use std::mem;

#[repr(transparent)]
#[derive(Debug,Clone)]
pub struct Bwd<A>(Option<Box<(Bwd<A>,A)>>);
impl <A> Bwd<A> {
  #[inline]
  #[must_use]
  pub const fn nil() -> Bwd<A> { Bwd(None) }
  #[inline]
  #[must_use]
  pub fn snoc(self, x:A) -> Bwd<A> { Bwd(Some(box (self,x))) }

  // there has to be a nicer way to say this
  pub fn snoc_mut(&mut self, x:A) {
    let old = mem::replace(self,Bwd(None));
    self.0 = Bwd::snoc(old, x).0;
  }

  #[must_use]
  pub fn singleton(x:A) -> Bwd<A> { Bwd::snoc(Bwd::nil(),x) }
}

impl <A> Bwd<A> {
  #[inline]
  #[must_use]
  pub fn unsnoc(self) -> Option<(Bwd<A>,A)> {
    Some(*self.0?)
  }

  #[inline]
  #[must_use]
  pub fn last(&self) -> Option<A> where A:Clone {
    Some(self.0.as_ref()?.1.clone())
  }

  #[inline]
  #[must_use]
  pub fn init(self) -> Option<Bwd<A>> {
    Some(self.0?.0)
  }

  #[inline]
  #[must_use]
  pub fn append(self, other: Bwd<A>) -> Bwd<A> {
    match other {
      Bwd(None) => self,
      Bwd(Some(box (xs, x))) => Bwd::snoc(self.append(xs),x)
    }
  }

  pub fn visit<F>(mut it : &Bwd<A>,f : F) -> bool where
      F : Fn (&A) -> bool {
    let mut found = false;
    while let Some(box (xs,x)) = &it.0 {
      if f(&x) {
        found = true;
        break;
      }
      it = &xs
    }
    found
  }
}

// this iterates backwards
impl <A> Iterator for Bwd<A> {
  type Item = A;
  fn next(&mut self) -> Option<A> {
    let m = mem::replace(&mut self.0,None);
    match m {
      None => None,
      Some(box(xs, x)) => { self.0 = xs.0; Some(x) }
    }
  }
}
