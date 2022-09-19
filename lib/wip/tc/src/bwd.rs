use std::sync::Arc;

#[repr(transparent)]
#[derive(Clone)]
pub struct Bwd<A>(Option<Arc<(Bwd<A>,A)>>);
impl <A> Bwd<A> {
  #[inline]
  #[must_use]
  pub const fn nil() -> Bwd<A> { Bwd(None) }
  #[inline]
  #[must_use]
  pub fn snoc(self, x:A) -> Bwd<A> { Bwd(Some(Arc::new((self,x)))) }
}

impl <A:Clone> Bwd<A> {
  #[inline]
  #[must_use]
  pub fn unsnoc(&self) -> Option<(Bwd<A>,A)> where A:Clone {
    Some(self.0.as_ref()?.as_ref().clone())
  }

  #[inline]
  #[must_use]
  pub fn last(&self) -> Option<A> where A:Clone {
    Some(self.0.as_ref()?.1.clone())
  }

  #[inline]
  #[must_use]
  pub fn init(&self) -> Option<Bwd<A>> {
    Some(self.0.as_ref()?.0.clone())
  }

  #[inline]
  #[must_use]
  pub fn append(&self,other: Bwd<A>) -> Bwd<A> { 
    panic!("TODO")
  }
}

// this is backwards
impl <A> Iterator for Bwd<A> {
  type Item = A;
  fn next(&mut self) -> Option<A> {
    panic!("TODO")
  }
}
