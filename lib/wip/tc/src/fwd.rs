use std::sync::Arc;

#[repr(transparent)]
#[derive(Clone)]
pub struct Fwd<A>(Option<Arc<(A,Fwd<A>)>>);
impl <A> Fwd<A> {
  #[inline]
  #[must_use]
  pub const fn nil() -> Fwd<A> { Fwd(None) }
  #[inline]
  #[must_use]
  pub fn cons(x:A, xs: Fwd<A>) -> Fwd<A> { Fwd(Some(Arc::new((x,xs)))) }

  pub fn singleton(x:A) -> Fwd<A> { Fwd::cons(x,Fwd::nil()) }
}


impl <A:Clone> Fwd<A> {
  //#[must_use]
  //pub fn peek(&self) -> Option<&(A,Self)> { self.0.as_ref() }

  // if you are going to clone the result, and are dropping this reference, you can use this
  #[inline]
  #[must_use]
  pub fn uncons(&self) -> Option<(A,Fwd<A>)> where A:Clone {
    Some(self.0.as_ref()?.as_ref().clone())
  }

  #[inline]
  #[must_use]
  pub fn head(&self) -> Option<A> where A:Clone {
    Some(self.0.as_ref()?.0.clone())
  }

  #[inline]
  #[must_use]
  pub fn tail(&self) -> Option<Fwd<A>> {
    Some(self.0.as_ref()?.1.clone())
  }

  pub fn append(&self, other: &Fwd<A>) -> Fwd<A> { 
    panic!("TODO")
  }
}

impl <A> Iterator for Fwd<A> {
  type Item = A;
  fn next(&mut self) -> Option<A> {
    panic!("TODO")
  }
}