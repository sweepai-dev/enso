#![feature(trusted_len)]
#![feature(exact_size_is_empty)]
#![feature(fn_traits)]
#![feature(unboxed_closures)]
#![allow(dead_code)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(trivial_numeric_casts)]

use std::borrow::Borrow;
use std::boxed::Box;
use std::cell::UnsafeCell;
use std::fmt::{self, Debug, Formatter};
use std::{mem, panic};
use std::ops::{Deref, Fn, FnMut, FnOnce};
use std::rc::Rc;
//use crate::sync::Lrc;

// almost has the semantics of a scala lazy val
// but requires mutation. attempts to generally
// follow the poison semantics of mutexes and 
// Once blocks.
pub enum Closure<T> {
  Delayed(Box<dyn FnOnce() -> T>),
  Forced(T)
}

impl<T: Debug> Debug for Closure<T> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      Closure::Delayed(_) => f.write_str("<closure>"),
      Closure::Forced(t) => Debug::fmt(&t, f)
    }
  }
}

impl <T> Closure<T> {
  #[inline]
  pub fn new<F: 'static + FnOnce() -> T>(f: F) -> Self {
    Closure::Delayed(Box::new(f))
  }

  #[inline]
  pub const fn ready(&self) -> bool {
    matches!(self,Closure::Forced(_))
  }


  #[inline]
  pub fn seq(&mut self) {
    let f = match self {
      Closure::Delayed(ref mut fr) => mem::replace(fr, detail::blackhole()),
      Closure::Forced(_) => return
    };
    *self = Closure::Forced(f());
  }

  #[inline]
  pub fn get(&mut self) -> &T {
    self.seq();
    match self {
      Closure::Forced(ref t) => &t,
      Closure::Delayed(_) => unreachable!()
    }
  }

  #[inline]
  pub const fn try_get(&self) -> Option<&T> {
    if let Closure::Forced(ref t) = self {
      Some(&t)
    } else {
      None
    }
  }
  #[inline]
  pub fn consume(self) -> T {
    match self {
      Closure::Delayed(f) => f(),
      Closure::Forced(t) => t
    }
  }

  #[inline]
  pub fn try_consume(self) -> Result<T,Self> {
    match self {
      Closure::Forced(t) => Ok(t),
      _ => Err(self)
    }
  }
  #[inline]
  pub fn map_consume<U, F>(self, f: F) -> Closure<U>
  where
    T: 'static,
    F: 'static + FnOnce(T) -> U, {
      Closure::new(move || f(self.consume()))
  }

  pub fn promote(&mut self) -> Lazy<T>
  where T: 'static + Clone, {
    if let Closure::Forced(value) = self {
      Lazy::from(value.clone())
    } else {
      let placeholder = Closure::new(|| unreachable!());
      let old_guts = mem::replace(self, placeholder);
      let result = Lazy(Rc::new(LazyVal(UnsafeCell::new(old_guts))));
      let clone = result.clone();
      let new_guts = Closure::new(move || clone.get().clone());
      let _ = mem::replace(self, new_guts);
      result
    }
  }
}

impl<T> From<T> for Closure<T> {
  #[inline]
  fn from(that: T) -> Self {
    Closure::Forced(that)
  }
}

impl<T: Default> Default for Closure<T> {
  fn default() -> Self {
    Closure::new(|| T::default())
  }
}

impl<T> IntoIterator for Closure<T> {
  type Item = T;
  type IntoIter = detail::ClosureIterator<T>;
  fn into_iter(self) -> Self::IntoIter {
    detail::ClosureIterator(Some(self))
  }
}

// this is a scala-style 'lazy val'. with all the upsides
// and downsides that would entail
#[derive(Debug)]
pub struct LazyVal<T>(UnsafeCell<Closure<T>>);

impl<T> LazyVal<T> {
  #[inline]
  pub fn new<F: 'static + FnOnce () -> T>(f: F) -> Self {
    LazyVal(UnsafeCell::new(Closure::new(f)))
  }
  #[inline]
  pub fn seq(&self) {
    unsafe { &mut *self.0.get() }.seq()
  }
  #[inline]
  pub fn ready(&self) -> bool {
    unsafe { &*self.0.get() }.ready()
  }
  #[inline]
  pub fn get(&self) -> &T {
    unsafe { &mut *self.0.get() }.get()
  }
  #[inline]
  pub fn try_get(&self) -> Option<&T> {
    unsafe { &*self.0.get() }.try_get()
  }
  #[inline]
  pub fn consume(self) -> T {
    self.0.into_inner().consume()
  }
  #[inline]
  pub fn try_consume(self) -> Result<T,Self> {
    self.0.into_inner().try_consume().map_err(|e| LazyVal(UnsafeCell::new(e)))
  }
  #[inline]
  pub fn map_consume<U, F>(self, f: F) -> LazyVal<U> where
    T: 'static,
    F: 'static + FnOnce(&T) -> U, {
      LazyVal::new(move || f(self.get()))
  }
  #[inline]
  pub fn promote(&self) -> Lazy<T> where
    T: 'static + Clone, {
      unsafe { &mut *self.0.get() }.promote()
  }
}

impl<T: Default> Default for LazyVal<T> {
  fn default() -> Self {
    LazyVal::new(|| T::default())
  }
}

impl<T> From<Closure<T>> for LazyVal<T> {
  fn from(that: Closure<T>) -> Self {
    LazyVal(UnsafeCell::new(that))
  }
}

impl<T> From<T> for LazyVal<T> {
  fn from(that: T) -> Self {
    LazyVal::from(Closure::from(that))
  }
}

impl<T> From<LazyVal<T>> for Closure<T> {
  fn from(that: LazyVal<T>) -> Self {
    that.0.into_inner()
  }
}

impl<T> Borrow<T> for LazyVal<T> {
  fn borrow(&self) -> &T {
    self.get()
  }
}

impl<T> AsRef<T> for LazyVal<T> {
  fn as_ref(&self) -> &T {
    self.get()
  }
}

impl<T> Deref for LazyVal<T> {
  type Target = T;
  fn deref(&self) -> &T {
    self.get()
  }
}

impl<T> IntoIterator for LazyVal<T> {
  type Item = T;
  type IntoIter = detail::ClosureIterator<T>;
  fn into_iter(self) -> Self::IntoIter {
    self.0.into_inner().into_iter()
  }
}

// a haskell-style thunk, single threaded
#[derive(Debug)]
#[repr(transparent)]
pub struct Lazy<T>(pub Rc<LazyVal<T>>);

impl<T> Clone for Lazy<T> {
  fn clone(&self) -> Self {
    Lazy(self.0.clone())
  }

  fn clone_from(&mut self, source: &Self) {
    self.0.clone_from(&source.0)
  }
}

impl<T> Lazy<T> {
  pub fn new<F: 'static + FnOnce() -> T>(f: F) -> Self {
    Lazy(Rc::new(LazyVal::new(f)))
  }
  pub fn new_strict(value: T) -> Self {
    Lazy(Rc::new(LazyVal::from(value)))
  }
  pub fn seq(&self) {
    self.0.as_ref().seq()
  }
  pub fn ready(&self) -> bool {
    self.0.as_ref().ready()
  }
  pub fn get(&self) -> &T {
    self.0.as_ref().get()
  }
  pub fn try_get(&self) -> Option<&T> {
    self.0.as_ref().try_get()
  }
  pub fn map<U, F: 'static + FnOnce(&T) -> U>(&self, f: F) -> Lazy<U> where
    T: 'static {
    let me = self.clone();
    Lazy::new(move || f(me.get()))
  }
  pub fn map2<U, V, F>(
    this: &Lazy<T>,
    that: &Lazy<U>,
    f: F,
  ) -> Lazy<V> where 
    U: 'static, 
    V: 'static, 
    T: 'static, 
    F: 'static + FnOnce(&T, &U) -> V  
  {
    let a = this.0.clone();
    let b = that.0.clone();
    Lazy::new(move || f(a.get(), b.get()))
  }

  // consumes this lazy value in an effort to try to avoid cloning the contents
  pub fn consume(self) -> T where 
    T: Clone, 
  {
    match Rc::try_unwrap(self.0) {
      Result::Ok(lval) => lval.consume(),
      Result::Err(this) => this.get().clone(), // other references to this thunk exist
    }
  }

  pub fn try_consume(self) -> Result<T,Self> where T: Clone, {
    match Rc::try_unwrap(self.0) {
      Result::Ok(lval) => lval.try_consume().map_err(|e| Lazy(Rc::new(e))),
      Result::Err(this) => match this.try_get() {
        Some(x) => Ok(x.clone()),
        None => Result::Err(Lazy(this))
      },
    }
  }
}

impl<T: Default> Default for Lazy<T> {
  fn default() -> Self {
    Lazy::new(|| T::default())
  }
}

impl<T> From<T> for Lazy<T> {
  #[inline]
  fn from(that: T) -> Self {
    Lazy::new_strict(that)
  }
}

impl<T:Clone> FnOnce<()> for Lazy<T> {
  type Output = T;
  extern "rust-call" fn call_once(self, _args: ()) -> T {
    self.0.as_ref().get().clone()
  }
}

impl<T:Clone> FnMut<()> for Lazy<T> {
  extern "rust-call" fn call_mut(&mut self, _args: ()) -> T {
    self.0.as_ref().get().clone()
  }
}

impl<T:Clone> Fn<()> for Lazy<T> {
  extern "rust-call" fn call(&self, _args: ()) -> T {
    self.0.as_ref().get().clone()
  }
}

impl<T> Borrow<T> for Lazy<T> {
  fn borrow(&self) -> &T {
    self.get()
  }
}

impl<T> AsRef<T> for Lazy<T> {
  fn as_ref(&self) -> &T {
    self.get()
  }
}

impl<T:Clone> IntoIterator for Lazy<T> {
  type Item = T;
  type IntoIter = detail::LazyIterator<T>;
  fn into_iter(self) -> Self::IntoIter {
    detail::LazyIterator(Some(self))
  }
}

pub mod detail {
  use std::iter::{ExactSizeIterator,TrustedLen,FusedIterator};
  use super::*;

  pub fn blackhole<T>() -> Box<dyn FnOnce() -> T> {
    Box::new(|| panic!("<infinite loop>"))
  }

  pub fn promoting<T>() -> Box<dyn FnOnce() -> T> {
    Box::new(|| unreachable!())
  }

  #[derive(Debug)]
  #[repr(transparent)]
  pub struct ClosureIterator<T>(pub Option<Closure<T>>);

  impl<T> Iterator for ClosureIterator<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
      Some(self.0.take()?.consume())
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
      let n = self.len();
      (n, Some(n))
    }
    fn last(self) -> Option<Self::Item> {
      Some(self.0?.consume())
    }
    fn count(self) -> usize {
      if self.0.is_some() {
          1
      } else {
          0
      }
    }
  }

  impl <T> FusedIterator for ClosureIterator<T> {}
  unsafe impl<T> TrustedLen for ClosureIterator<T> {}
  impl <T> ExactSizeIterator for ClosureIterator<T> {
    fn len(&self) -> usize {
      if self.0.is_some() { 1 } else { 0 }
    }

    fn is_empty(&self) -> bool {
      self.0.is_none()
    }
  }

  #[derive(Debug)]
  #[repr(transparent)]
  pub struct LazyIterator<T>(pub Option<Lazy<T>>);

  impl<T> Clone for LazyIterator<T> {
    #[inline]
    fn clone(&self) -> Self {
      LazyIterator(self.0.clone())
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
      self.0.clone_from(&source.0)
    }
  }
  impl<T:Clone> Iterator for LazyIterator<T> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
      Some(self.0.take()?.consume())
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
      let n = if self.0.is_some() { 1 } else { 0 };
      (n, Some(n))
    }

    fn last(self) -> Option<Self::Item> {
      Some(self.0?.consume())
    }

    fn count(self) -> usize {
      if self.0.is_some() {
        1
      } else {
        0
      }
    }
  }

  impl <T: Clone> FusedIterator for LazyIterator<T> {}
  unsafe impl<T: Clone> TrustedLen for LazyIterator<T> {}
  impl <T: Clone> ExactSizeIterator for LazyIterator<T> {
    fn len(&self) -> usize {
      if self.0.is_some() { 1 } else { 0 }
    }

    fn is_empty(&self) -> bool {
      self.0.is_none()
    }
  }

}

pub fn main() {
  let y = 12;
  println!("{}", y);
  let x = Lazy::new(move || {
    println!("x forced");
    y * 10
  });
  let w = x.map(|r| r + 1);
  println!("{}", w());
  println!("{}", w.get());
  for z in w {
    println!("{}", z);
  }
}
