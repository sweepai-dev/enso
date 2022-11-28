use std::mem;

#[repr(transparent)]
#[derive(Debug,Clone)]
pub struct Fwd<A>(Option<Box<(A,Fwd<A>)>>);
impl <A> Fwd<A> {
  #[inline]
  #[must_use]
  pub const fn nil() -> Fwd<A> { Fwd(None) }

  #[inline]
  #[must_use]
  pub fn cons(x:A, xs: Fwd<A>) -> Fwd<A> { Fwd(Some(box (x,xs))) }

  #[must_use]
  pub fn singleton(x:A) -> Fwd<A> { Fwd::cons(x,Fwd::nil()) }
}


impl <A> Fwd<A> {
  #[inline]
  #[must_use]
  pub fn uncons(self) -> Option<(A,Fwd<A>)> {
    Some(*self.0?)
  }

  #[inline]
  #[must_use]
  pub fn head(&self) -> Option<A> where A:Clone {
    Some(self.0.as_ref()?.0.clone())
  }

  #[inline]
  #[must_use]
  pub fn tail(self) -> Option<Fwd<A>> {
    Some(self.0?.1)
  }

  #[must_use]
  pub fn append(self, _other: Fwd<A>) -> Fwd<A> {
    match self {
      Fwd(None) => _other,
      Fwd(Some(box (x, xs))) => Fwd::cons(x,xs.append(_other))
    }
  }

  pub fn visit<F>(mut it : &Fwd<A>,f : F) -> bool where
      F : Fn (&A) -> bool {
    let mut found = false;
    while let Some(box (x,xs)) = &it.0 {
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
impl <A> Iterator for Fwd<A> {
  type Item = A;
  fn next(&mut self) -> Option<A> {
    let m = mem::replace(&mut self.0,None);
    match m {
      None => None,
      Some(box(x, xs)) => { self.0 = xs.0; Some(x) }
    }
  }
}