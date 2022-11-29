use std::iter::Iterator;
use std::rc::Rc;
use std::string::String;
use enso_prelude::OptionOps;

use crate::bwd::Bwd;
use crate::fwd::Fwd;

#[derive(PartialEq,Eq,PartialOrd,Ord,Clone,Hash,Debug)]
pub enum TyName {
  B(u32),
  F(String)
}

// we're going to do too much copying for now because box patterns are too useful
#[derive(PartialEq,Eq,PartialOrd,Ord,Clone,Hash,Debug)]
pub enum Ty {
  V(TyName),     // free variable
  Arr(Type,Type) // i hate everything about this
}

type Type = Rc<Ty>;

pub fn v(n: TyName) -> Type {
  Rc::new(Ty::V(n))
}

pub fn arr(arg: Type, res: Type) -> Type {
  Rc::new(Ty::Arr(arg,res))
}

// instantiate the next bound variable (replace it with a given type)
// TODO: retain sharing if possible
pub fn instantiate(x: TyName, t: Type) -> Type {
  match t.as_ref() {
    Ty::V(TyName::B(0)) => t,
    Ty::V(TyName::B(n)) => v(TyName::B(n-1)),
    Ty::V(TyName::F(y)) => v(TyName::F(*y)),
    Ty::Arr(arg,res) => arr(instantiate(x,*arg),instantiate(x,*res))
  }
}


// instantiate a bound variable (replace it with a given type)
// TODO: retain sharing if possible
pub fn instantiate_n(n: u32, x: TyName, t: Type) -> Type {
  match t.as_ref() {
    Ty::V(TyName::B(m)) if n == *m => t,
    Ty::V(TyName::B(m)) if n < *m => v(TyName::B(m-1)), // decrease all larger numbers
    Ty::V(TyName::B(m)) /* if n > *m */ => v(TyName::B(*m)),
    Ty::V(TyName::F(y)) => v(TyName::F(*y)),
    Ty::Arr(arg,res) => arr(instantiate(x,*arg),instantiate(x,*res))
  }
}

// abstract a free variable (capture it as bound)
// TODO: retain sharing if possible
pub fn bind_type(x: String, t: Type) -> Type {
  match t.as_ref() {
    Ty::V(TyName::B(n)) => v(TyName::B(n+1)),
    Ty::V(TyName::F(y)) => v(if *y == x { TyName::B(0) } else { TyName::F(*y) }),
    Ty::Arr(arg,res) => arr(bind_type(x,*arg),bind_type(x,*res))
  }
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
  TY(TyEntry), // type level context entries
  TM(TmEntry), // term level context entries
  SEMI
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

  fn specialize(&mut self, s: Scheme) -> Type {
    match s.as_ref() { 
      Sc::Type(t) => t.clone(),
      Sc::Forall(sp) => {
        let beta = self.fresh(None);
        self.specialize(instantiate_scheme(beta,sp))
      }
      Sc::Let(t,sp) => {
        let beta = self.fresh(Some(t.clone()));
        self.specialize(instantiate_scheme(beta,sp))
      }
    }
  }
}


// type schemas (pre-boxy types)
#[derive(PartialEq,Eq,PartialOrd,Ord,Clone,Hash,Debug)]
pub enum Sc {
  Type(Type),
  Forall(Scheme),
  Let(Type,Scheme)
}


type Scheme = Rc<Sc>;

impl Sc {
  pub fn ty(t: Type) -> Scheme {
    Rc::new(Sc::Type(t))
  }

  // TODO: smart constructor that captures a name
  pub fn forall(s: Scheme) -> Scheme {
    Rc::new(Sc::Forall(s))
  }

  // TODO: smart constructor that captures a name
  pub fn let_(t: Type, s: Scheme) -> Scheme {
    Rc::new(Sc::Let(t,s))
  }
}

pub fn instantiate_scheme_n(n: u32, x: TyName, s: Scheme) -> Scheme {
  match s.as_ref() {
    Sc::Type(t) => Sc::ty(instantiate_n(n,x,*t)),
    Sc::Forall(sp) => Sc::forall(instantiate_scheme_n(n+1,x,*sp)),
    Sc::Let(t,sp) => Sc::let_(instantiate_n(n,x,*t),instantiate_scheme_n(n+1,x,*sp))
  }
}

// TODO: build a scheme binding all at once rather than work debruijn/bound style
pub fn instantiate_scheme(x: TyName, s: Scheme) -> Scheme {
  instantiate_scheme_n(0,x,s)
}

pub fn bind_suffix(e : Suffix, t : Type) -> Scheme {
  panic!("ok")
}


#[derive(PartialEq,Eq,PartialOrd,Ord,Clone,Hash,Debug)]
pub struct TmName(String);

#[derive(PartialEq,Eq,PartialOrd,Ord,Clone,Hash,Debug)]
pub enum Tm {
  V(TmName),
  App(Term,Term),
  Lam(TmName,Term),
  Let(TmName,Term,Term)
}

type Term = Rc<Tm>;

#[derive(PartialEq,Eq,PartialOrd,Ord,Clone,Hash,Debug)]
pub struct TmEntry(TmName,Scheme);

