use std::iter::Iterator;
use std::rc::Rc;
use enso_prelude::OptionOps;

use crate::bwd::Bwd;
use crate::fwd::Fwd;

#[derive(PartialEq,Eq,PartialOrd,Ord,Copy,Clone,Hash,Debug)]
pub struct TyName(u32);

// we're going to do too much copying for now because box patterns are too useful
#[derive(PartialEq,Eq,PartialOrd,Ord,Clone,Hash,Debug)]
pub enum Ty {
  V(TyName),
  Arr(Type,Type) // i hate everything about this
}

type Type = Rc<Ty>;

pub fn v(n: TyName) -> Type {
  Rc::new(Ty::V(n))
}

pub fn arr(arg: &Type, res: &Type) -> Type {
  Rc::new(Ty::Arr(arg.clone(),res.clone()))
}

pub trait FTV {
  fn contains(&self, name: TyName) -> bool;
}

impl FTV for TyName {
  fn contains(&self, name: TyName) -> bool {
    *self == name
  }
}

impl <T:FTV> FTV for Rc<T> {
  fn contains(&self, name: TyName) -> bool {
    self.as_ref().contains(name)
  }
}

impl FTV for Ty {
  fn contains(&self, name: TyName) -> bool {
    match self {
      Ty::V(x) => *x == name,
      Ty::Arr(l, r) => l.contains(name) || r.contains(name),
    }
  }
}

#[derive(Clone,Debug)]
pub struct TyEntry(TyName, Option<Type>);

impl <A:FTV> FTV for Option<A> {
  fn contains(&self, name: TyName) -> bool {
    self.map_ref(|x|x.contains(name)).unwrap_or(false)
  }
}

impl <A:FTV> FTV for Bwd<A> {
  fn contains(&self, name: TyName) -> bool {
    Bwd::visit(self,|x|x.contains(name))
  }
}

impl <A:FTV> FTV for Fwd<A> {
  fn contains(&self, name: TyName) -> bool {
    Fwd::visit(self,|x|x.contains(name))
  }
}

impl FTV for TyEntry {
  fn contains(&self, name: TyName) -> bool {
    match self {
      TyEntry(_,Some(x)) => x.contains(name),
      TyEntry(_,None) => false
    }
  }
}

#[derive(Debug,Clone)]
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

pub fn affix_mut(ctx: &mut Ctx, mut suffix: Suffix) {
  while let Some((te,new_suffix)) = suffix.uncons() {
    ctx.snoc_mut(Entry::TY(te));
    suffix = new_suffix;
  }
}



#[derive(Debug,Clone)]
pub enum Ext {
  Restore,
  Replace(Suffix)
}

#[allow(non_upper_case_globals)]
pub const restore : Ext = Ext::Restore;
pub fn replace(suffix: Suffix) -> Ext { Ext::Replace(suffix) }

impl Default for Ext {
    fn default() -> Self { restore }
}

pub trait AsNoContext {
  fn no_context() -> Self;
}

#[derive(Debug,Clone,PartialEq,Eq,PartialOrd,Ord)]
pub enum UnifyError {
  Occurs,
  Mismatch(Type,Type),
  NoContext
}

impl AsNoContext for UnifyError {
  fn no_context() -> Self { UnifyError::NoContext }
}

#[derive(Debug,Clone)]
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
    self.next += 1;
    let next = TyName(self.next);
    self.ctx.snoc_mut(Entry::TY(TyEntry(next,decl)));
    next
  }

  fn push(&mut self, suffix: Suffix) {
    affix_mut(&mut self.ctx, suffix)
  }

  #[allow(non_snake_case)]
  fn onTop<E : AsNoContext,F>(&mut self, f: F) -> Result<(),E> where
     F : FnOnce(&mut Context,TyEntry) -> Result<Ext,E> {
     let nuD = self.ctx.next().ok_or_else(|| E::no_context())?;
     Ok(match nuD {
      Entry::TY(aD) => {
        match f(self,aD.clone())? { // using a cheap Rc-based clone since this happens all the time
          Ext::Replace(suffix) => self.push(suffix),
          Ext::Restore => {
            self.ctx.snoc_mut(Entry::TY(aD))
          }
        }
      },
      #[allow(unreachable_patterns)]
      _ => {
        self.onTop(f)?;
        self.ctx.snoc_mut(nuD)
      }
    })
  }

  // TODO: borrow lhs and rhs
  fn unify(&mut self, lhs: &Type, rhs: &Type) -> Result<(),UnifyError> {
    match (lhs.clone().as_ref(),rhs.clone().as_ref()) {
      (Ty::Arr(t0,t1),Ty::Arr(v0,v1)) => {
        self.unify(t0,t1)?;
        self.unify(v0,v1)
      },
      (Ty::V(a), Ty::V(b)) => {
        self.onTop(|this,te| {
          let TyEntry(g, d) = te;
          Ok(
            match (g == *a, g == *b, d) {
              (true, true,  _) => restore,
              (true, false, None) => replace(Fwd::cons(TyEntry(*a,Some(v(*b))),Fwd::nil())),
              (false, true, None) => replace(Fwd::cons(TyEntry(*b,Some(v(*a))),Fwd::nil())),
              (true, false, Some(t)) => { this.unify(&v(*b),&t)?; restore }
              (false, true, Some(t)) => { this.unify(&v(*a),&t)?; restore }
              (false, false, _) => { this.unify(&v(*a),&v(*b))?; restore }
            }
          )
        })
      },
      (Ty::V(a), _t) => self.solve(*a,Fwd::nil(),rhs),
      (_t, Ty::V(b)) => self.solve(*b,Fwd::nil(),lhs)
    }
  }

  fn solve(&mut self, a: TyName, suffix: Suffix, t: &Type) -> Result<(),UnifyError> {
    self.onTop(|this,te| -> Result<Ext, UnifyError> {
      let TyEntry(g, d) = te;
      let occurs = t.contains(g) || suffix.contains(g);
      match (g == a, occurs, d) {
        (true, true, _) =>
          Err(UnifyError::Occurs),
        (true, false, None) =>
          Ok(replace(suffix.append(Fwd::singleton(TyEntry(a,Some(t.clone())))))),
        (true, false, Some(v)) => {
          this.push(suffix);
          this.unify(&v,t)?;
          Ok(restore)
        },
        (false, true, d) => {
          this.solve(a,Fwd::cons(TyEntry(g,d),suffix), t)?;
          Ok(replace(Fwd::nil()))
        }
        (false, false, _) => {
          this.solve(a,suffix,t)?;
          Ok(restore)
        }
      }
    })
  }
}