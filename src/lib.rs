//! Interned string and bytes for rust.
//!
//! # What is interning?
//!
//! Interning is a method to store exactly one copy of immutable data.
//!
//! Imagine your program holds lots of string values, mostly same value in it,
//! and does not mutate them at all. If you use `String` to store them,
//! lots of memories are wasted just for storing identical texts.
//!
//! Interning efficiently eliminate this problem by managing global pool of cache,
//! in the case above the type of the pool can be `HashSet<Rc<str>>`.
//! When you need a new owned string, first you should lookup global pool for it.
//! If desired string is found then use it.
//! If not, just create a new one and put them also to the pool.
//!
//! # What does this library provide?
//!
//! This crate exposes `Intern<T>` type correspond to `Rc<T>`
//! but guaranteed to be unique over its value within thread
//! and provide fast O(1) comparison and hashing.
//!
//! # Example
//!
//!   ```
//!   # extern crate internship;
//!   # use std::collections::HashMap;
//!   use internship::IStr;
//!
//!   # fn main() {
//!   let foo = IStr::new("foo"); // type is IStr
//!   let foo2 = IStr::new("foo"); // reuse foo's buffer
//!
//!   let mut map = HashMap::new();
//!   map.insert(IStr::new("key"), 42);
//!   assert_eq!(map.get(&IStr::new("key")), Some(&42));
//!   # }
//!   ```
//!
//! # Why should I use `IStr` over `Rc<str>`?
//!
//! `IStr` has some advantages over `Rc<str>`
//!
//! 1. Space efficient
//!
//!   As only single allocation is happen per unique value,
//!   you can even spam `IStr::new()` without worrying about memory bloat.
//!
//! 1. O(1) equality check
//!
//!   As only one copy of unique value can be exist,
//!   comparing two `IStr` can be done with just single pointer comparison
//!   instead comparing entire contents of strings.
//!
//! 1. O(1) hash calculation
//!
//!   Again, as only one copy of unique value can be exist,
//!   its allocated memory address can represent underlying value
//!   so calculating hash over its pointer makes perfect sense to hash `IStr`.
//!   Now you can perform blazingly-fast hashmap lookup with arbitrary string key!
//!

use std::rc::Rc;
use std::ffi::{CStr, OsStr};
use std::path::Path;

pub type IStr = Intern<str>;
pub type IBytes = Intern<[u8]>;
pub type ICStr = Intern<CStr>;
pub type IOsStr = Intern<OsStr>;
pub type IPath = Intern<Path>;

/// Interned data
///
/// `Intern<T>` is conceptually same as `Rc<T>` but unique over its value within thread.
///
#[derive(Debug, PartialOrd, Ord, Default)]
pub struct Intern<T>(Rc<T>) where T: private::IntoIntern + ?Sized;

mod private {
    use super::*;
    use std::hash::{Hash, Hasher};
    use std::cell::RefCell;
    use std::collections::HashSet;
    use std::thread::LocalKey;
    use std::ops::{Deref, Drop};
    use std::fmt;
    use std::path::PathBuf;
    use std::ffi::{CString, OsString};

    pub trait IntoIntern: Eq + Hash + 'static {
        fn provide_intern_pool() -> &'static LocalKey<RefCell<HashSet<Rc<Self>>>>;
        fn to_rc(&self) -> Rc<Self>;
    }

    macro_rules! impl_intern {
        ($($T:ty),*) => ($(
            impl IntoIntern for $T {
                fn provide_intern_pool() -> &'static LocalKey<RefCell<HashSet<Rc<Self>>>> {
                    thread_local! {
                        static POOL: RefCell<HashSet<Rc<$T>>> = Default::default();
                    }
                    &POOL
                }

                fn to_rc(&self) -> Rc<Self> {
                    self.into()
                }
            }
        )*);
    }

    impl_intern!(str, [u8], CStr, OsStr, Path);

    impl<T: IntoIntern + ?Sized> Intern<T> {
        /// Create new `Intern<T>` from given value, if matching cache is not found.
        pub fn new(value: &T) -> Self {
            T::provide_intern_pool().with(|pool| {
                let mut pool = pool.borrow_mut();
                let cached = pool.get(value).cloned();

                match cached {
                    Some(v) => Intern(v),
                    None => {
                        let v = value.to_rc();
                        pool.insert(Rc::clone(&v));
                        Intern(v)
                    }
                }
            })
        }
    }

    impl<'a, T: IntoIntern + ?Sized> From<&'a T> for Intern<T> {
        fn from(v: &'a T) -> Self {
            Intern::new(v)
        }
    }

    macro_rules! impl_from_owned {
        ($($T:ty : $Owned:ty),*) => ($(
            impl From<$Owned> for Intern<$T> {
                fn from(v: $Owned) -> Self {
                    Intern::new(&v)
                }
            }
        )*);
    }

    impl_from_owned!(str:String, [u8]:Vec<u8>, CStr:CString, OsStr:OsString, Path:PathBuf);

    impl<T: IntoIntern + ?Sized> Clone for Intern<T> {
        fn clone(&self) -> Self {
            Intern(self.0.clone())
        }
    }

    impl<T: IntoIntern + ?Sized> Deref for Intern<T> {
        type Target = T;

        fn deref(&self) -> &T {
            &*self.0
        }
    }

    impl<T: IntoIntern + ?Sized> Drop for Intern<T> {
        fn drop(&mut self) {
            // strong count == 2 means no other copies of this interned value exist
            // other then the `self` which will be dropped and the one in the pool,
            // so it's not really being used.
            //
            if Rc::strong_count(&self.0) == 2 {
                T::provide_intern_pool().with(|pool| {
                    pool.borrow_mut().remove(&self.0)
                });
            }
        }
    }

    impl<T: IntoIntern + ?Sized> PartialEq for Intern<T> {
        fn eq(&self, other: &Self) -> bool {
            Rc::ptr_eq(&self.0, &other.0)
        }
    }

    impl<T: IntoIntern + ?Sized> Eq for Intern<T> {}

    impl<T: IntoIntern + ?Sized> Hash for Intern<T> {
        fn hash<H: Hasher>(&self, hasher: &mut H) {
            let ptr = Rc::into_raw(Rc::clone(&self.0));
            unsafe { Rc::from_raw(ptr) };
            ptr.hash(hasher)
        }
    }

    impl<T: IntoIntern + ?Sized> AsRef<T> for Intern<T> {
        fn as_ref(&self) -> &T {
            &*self.0
        }
    }

    macro_rules! impl_fmt {
        ($($Trait:ident),*) => ($(
            impl<T: IntoIntern + ?Sized + fmt::$Trait> fmt::$Trait for Intern<T> {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    self.0.fmt(f)
                }
            }
        )*);
    }

    impl_fmt!(Binary, Display, LowerExp, LowerHex, Octal, UpperExp, UpperHex);

    impl<T: IntoIntern + ?Sized> fmt::Pointer for Intern<T> {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            self.0.fmt(f)
        }
    }
}

// more trait impl for `Intern<str>`
mod string;

#[cfg(feature = "serde-compat")]
mod serde_support;

#[cfg(test)]
mod tests {
    use super::*;
    use super::private::IntoIntern;

    #[test]
    fn test_eq_check() {
        assert_eq!(IStr::new("foo"), IStr::new("foo"));
        assert_eq!(&*Intern::new("bar"), "bar");
    }

    #[test]
    fn test_pool_and_rc_count() {
        use std::mem::replace;

        let prev_pool = <[u8]>::provide_intern_pool().with(|pool| {
            replace(&mut *pool.borrow_mut(), Default::default())
        });

        let pool_size = || <[u8]>::provide_intern_pool().with(|pool| {
            pool.borrow().len()
        });

        assert_eq!(pool_size(), 0);
        let b1 = IBytes::new(&b"foo"[..]);
        assert_eq!(pool_size(), 1);
        assert_eq!(Rc::strong_count(&b1.0), 2);
        let b2 = IBytes::new(&b"bar"[..]);
        assert_eq!(pool_size(), 2);
        let b3 = IBytes::new(&b"foo"[..]);
        assert_eq!(pool_size(), 2);
        assert_eq!(Rc::strong_count(&b1.0), 3);
        let b4 = Intern::clone(&b3);
        assert_eq!(Rc::strong_count(&b1.0), 4);
        drop(b2);
        assert_eq!(pool_size(), 1);
        drop(b1);
        drop(b4);
        assert_eq!(pool_size(), 1);
        assert_eq!(Rc::strong_count(&b3.0), 2);
        drop(b3);
        assert_eq!(pool_size(), 0);

        <[u8]>::provide_intern_pool().with(|pool| {
            replace(&mut *pool.borrow_mut(), prev_pool);
        })
    }
}
