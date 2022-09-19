use std::sync::Arc;
use std::iter::Iterator;
use crate::bwd::Bwd;
use crate::fwd::Fwd;

#[derive(PartialEq,Eq,PartialOrd,Ord,Copy,Clone,Hash)]
pub struct TyName(u32);

#[derive(PartialEq,Eq,PartialOrd,Ord,Clone,Hash)]
pub enum Ty {
  V(TyName),
  Arr(Type,Type)
}

pub type Type = Arc<Ty>;

pub fn v(n: TyName) -> Type {
  Arc::new(Ty::V(n))
}
pub fn arr(arg: Type, res: Type) -> Type {
  Arc::new(Ty::Arr(arg,res))
}

pub trait FTV {
  fn contains(&self, name: TyName) -> bool;
}

impl FTV for TyName {
  fn contains(&self, name: TyName) -> bool { 
    *self == name 
  }
}

pub struct TyEntry(TyName, Option<Type>);

#[macro_export]
macro_rules! into_iter_ftv {
  () => {
    fn contains(&self, name: TyName) -> bool {
      self.clone().into_iter().any(|x|x.contains(name))
    }
  }
}


#[macro_export]
macro_rules! iter_ftv {
  () => {
    fn contains(&self, name: TyName) -> bool {
      <Self as Iterator>::any(&mut self,|x|x.contains(name))
    }
  }
}
  
impl <A:FTV> FTV for Option<A> {
  into_iter_ftv!{}
  // fn contains(&self, name: TyName) -> bool {  
  //   if let Some(a) = self {
  //     a.contains(name)
  //   } else {
  //     false
  //   }
  // }
}
impl <A:FTV> FTV for Bwd<A> { iter_ftv!{} }
impl <A:FTV> FTV for Fwd<A> { iter_ftv!{} }


impl FTV for TyEntry {
  fn contains(&self, name: TyName) -> bool {
    match self {
      TyEntry(_,b) => b.contains(name)
    }
  }
}

#[derive(Clone)]
pub enum Entry {
  TY(TyEntry) // | ...
}

pub type Ctx = Bwd<Entry>;
pub type Suffix = Fwd<TyEntry>;

pub fn affix(mut ctx: Ctx, mut suffix: Suffix) -> Ctx {
  while let Some((te,new_suffix)) = suffix.uncons() {
    ctx = ctx.snoc(Entry::TY(te));
    suffix = new_suffix;
  }
  ctx
}

#[derive(Clone)]
pub enum Ext {
  Restore,
  Replace(Suffix)
}

pub const restore : Ext = Ext::Restore;
pub fn replace(suffix: Suffix) -> Ext { Ext::Replace(suffix) }

impl Default for Ext {
    fn default() -> Self { restore }
}

pub trait AsNoContext {
  fn no_context() -> Self;
}

#[derive(Clone,PartialEq,Eq,PartialOrd,Ord)]
pub enum UnifyError {
  Occurs,
  Mismatch(Type,Type),
  NoContext
}

impl AsNoContext for UnifyError {
  fn no_context() -> Self { UnifyError::NoContext }
}

pub struct Context {
  next: u32,
  ctx: Ctx
}

impl Default for Context {
  fn default() -> Self { 
    Context { 
      next: 0, 
      ctx: Bwd::nil() 
    }
  }
}


impl Context {
  fn fresh (&mut self, decl: Option<Type>) -> TyName {
    let next = TyName(self.next += 1);
    self.ctx = self.ctx.snoc(Entry::TY(TyEntry(next,decl)));
    next
  }

  fn push(&mut self, suffix: Suffix) {
    self.ctx = affix(self.ctx, suffix)
  }

  fn onTop<E : AsNoContext,F>(&self, f: F) -> Result<(),E> where
     F : FnOnce(TyEntry) -> Result<Ext,E> {
     let (new_ctx, nuD) = self.ctx.last.ok_or_else(|| E::no_context() )?;
     self.ctx = new_ctx;
     Ok(match nuD {
      Entry::TY(aD) => {
        match f(aD)? {
          Ext::Replace(suffix) => self.push(suffix),
          Ext::Restore => self.ctx = self.ctx.snoc(nuD)
        }
      },
      _ => { 
        self.onTop(f)?;
        self.ctx.snoc(nuD)
      }
    })
  }

  // TODO: borrow lhs and rhs 
  fn unify(&mut self, lhs: &Type, rhs: &Type) -> Result<(),UnifyError> {
    match (*lhs,*rhs) {
      (Ty::Arr(t0,t1),Ty::Arr(v0,v1)) => { self.unify(t0,t1)?; self.unify(v0,v1) },
      (Ty::V(a), Ty::V(b)) => {
        self.onTop(|te| {
          let TyEntry(g, d) = te; 
          Ok(
            match (g == a, g == b, d) {
              (true, true,  ) => restore,
              (true, false, None) => replace(Fwd::cons(TyEntry(a,Some(v(b))),Fwd::nil())),
              (false, true, None) => replace(Fwd::cons(TyEntry(b,Some(v(a))),Fwd::nil())),
              (true, false, Some(t)) => { self.unify(v(b),t)?; restore }
              (false, true, Some(t)) => { self.unify(v(a),t)?; restore }
              (false, false, _) => { self.unify(v(a),v(b))?; restore }
            }          
          )
        })
      },
      (Ty::V(a), t) => self.solve(a,Fwd::nil(),t),
      (t, Ty::V(b)) => self.solve(b,Fwd::nil(),t)
    }
  }

  // todo generalize error type to AsUnifyError?
  fn solve(&mut self, a: TyName, suffix: Suffix, t: Type) -> Result<(),UnifyError> {
    self.onTop(|te| {
      let TyEntry(g, d) = te;
      let occurs = t.contains(g) || suffix.contains(g);
      match (g == a, occurs, d) {
        (true, true, _) => 
          Err(UnifyError::Occurs),
        (true, false, None) => 
          // requires 
          Ok(replace(suffix.append(Fwd::singleton(TyEntry(a,Some(t)))))),
        (true, false, Some(v)) => {
          self.push(suffix);
          self.unify(v,t)?;
          Ok(restore)
        },
        (false, true, _) => {
          // TODO: fully understand this branch
          self.solve(a,Fwd::cons(te,suffix), t)?;
          Ok(replace(Fwd::nil())) 
        }
        (false, false, _) => {
          self.solve(a,suffix,t)?;
          Ok(restore)
        }
      }
    })
  }
}