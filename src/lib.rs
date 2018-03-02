//! Interned string and more for rust.
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
//! The core of the API is `Intern<T>` where `T` is `str`-like types in `std`.
//! You can think of it as `Rc<T>`, but guaranteed uniqueness over value within thread.
//!
//! # Example
//!
//!   ```
//!   # extern crate internship;
//!   # use std::collections::HashMap;
//!   use internship::{Intern, intern};
//!
//!   # fn main() {
//!   let foo = intern("foo"); // type is Intern<str>
//!   let foo2 = intern("foo"); // reuse foo's buffer
//!
//!   let mut map = HashMap::new();
//!   map.insert(intern("key"), 42);
//!   assert_eq!(map.get(&intern("key")), Some(&42));
//!   # }
//!   ```
//!
//! # How is `Intern<T>` better then `Rc<T>`?
//!
//! `Intern<T>` has some advantages over `Rc<T>`
//!
//! 1. Space efficient
//!
//!   As only single allocation is happen per unique value,
//!   you can even span `intern()` without worrying about memory bloat.
//!
//! 1. Cheap equality check
//!
//!   As only one copy of unique value can be exist,
//!   comparing two `Intern<T>` can be done with just single pointer comparison
//!   instead comparing every bytes of strings.
//!
//! 1. Cheap hash calculation
//!
//!   Again, as only one copy of unique value can be exist,
//!   its allocated memory address can represent underlying value
//!   so calculating hash over its pointer makes perfect sense to hash `Intern<T>`.
//!   Now you can perform blazingly-fast hashmap lookup with arbitrary string key!
//!
//! # What types can be interned?
//!
//! Currently these types are supported.
//!
//! - `str`
//! - `[u8]`
//! - `CStr`
//! - `OsStr`
//! - `Path`
//!
//! You can find interned type of them as re-export, like `InternStr`.
//!
//! > For now, only `str` and `[u8]` are supported by default.
//! > This limitation should be removed after docs.rs update their rustc from v1.22.0
//! > I think it's more important to show proper docs on docs.rs
//! > than make this feature works out of the box.
//! > If you want to use others at now, turn on cargo feature "shared_from_slice2".
//!

use std::rc::Rc;
use std::hash::{Hash, Hasher};
use std::cell::RefCell;
use std::collections::HashSet;
use std::borrow::Borrow;
use std::thread::LocalKey;
use std::ops::{Deref, Drop};
use std::fmt;

#[cfg(feature = "shared_from_slice2")]
use std::ffi::{CStr, OsStr};
#[cfg(feature = "shared_from_slice2")]
use std::path::Path;

pub type InternStr = Intern<str>;
pub type InternBytes = Intern<[u8]>;
#[cfg(feature = "shared_from_slice2")]
pub type InternCStr = Intern<CStr>;
#[cfg(feature = "shared_from_slice2")]
pub type InternOsStr = Intern<OsStr>;
#[cfg(feature = "shared_from_slice2")]
pub type InternPath = Intern<Path>;

/// Interned data
///
/// `Intern<T>` is conceptually same as `Rc<T>` but unique over its value within thread.
///
#[derive(Debug, Clone, Eq, PartialOrd, Ord, Default)]
pub struct Intern<T>(Rc<T>) where T: AllowIntern + ?Sized;

mod private {
    pub trait Guard {}
}

pub trait AllowIntern: Eq + Hash + ToOwned + private::Guard + 'static {

    /// Provide thread-local interned pool for this type.
    ///
    /// This is necessary as Rust doesn't allow static variables with generic type,
    /// as this can't be monomorphized trivially.
    ///
    /// This function is private as `Intern` relies on assumption that
    /// provided pool never change except for the construction/destruction of the `Intern`.
    ///
    unsafe fn provide_per_thread_intern_pool() -> &'static LocalKey<RefCell<HashSet<Rc<Self>>>>;
}

impl<T> Intern<T> where T: AllowIntern + ?Sized, for<'a> &'a T: Into<Rc<T>> {

    /// Create new `Intern<T>` from given value if matching cache not found.
    ///
    pub fn new(value: &T) -> Self {
        let pool = unsafe {
            T::provide_per_thread_intern_pool()
        };
        pool.with(|pool| {
            let mut pool = pool.borrow_mut();
            let cached = pool.get(value.borrow()).cloned();

            match cached {
                Some(v) => Intern(v),
                None => {
                    let v = value.into();
                    pool.insert(Rc::clone(&v));
                    Intern(v)
                }
            }
        })
    }
}

/// Create new `Intern<T>` from given value if matching cache not found.
///
/// This function is a thin wrapper over `Intern::new()` for convenience.
///
pub fn intern<T>(value: &T) -> Intern<T> where
    T: AllowIntern + ?Sized,
    for<'a> &'a T: Into<Rc<T>>,
{
    Intern::<T>::new(value)
}

impl<'a, T> From<&'a T> for Intern<T> where
    T: AllowIntern + ?Sized,
    for<'aa> &'aa T: Into<Rc<T>>
{
    fn from(value: &T) -> Self {
        Self::new(value)
    }
}

impl<T> Deref for Intern<T> where T: AllowIntern + ?Sized {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl<T> Drop for Intern<T> where T: AllowIntern + ?Sized {
    fn drop(&mut self) {
        // strong count == 2 means no other copies of this interned value exist
        // other then the `self` which will be dropped and the one in the pool,
        // so it's not really being used.
        //
        if Rc::strong_count(&self.0) == 2 {
            let pool = unsafe {
                T::provide_per_thread_intern_pool()
            };
            pool.with(|pool| pool.borrow_mut().remove(&self.0));
        }
    }
}

impl<T> PartialEq<Self> for Intern<T> where
    T: AllowIntern + ?Sized {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl<T> Hash for Intern<T> where T: AllowIntern + ?Sized {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        let ptr = Rc::into_raw(Rc::clone(&self.0));
        unsafe { Rc::from_raw(ptr) };
        ptr.hash(hasher)
    }
}

impl<T> AsRef<T> for Intern<T> where T: AllowIntern + ?Sized {
    fn as_ref(&self) -> &T {
        &*self.0
    }
}

impl<T> fmt::Display for Intern<T> where T: AllowIntern + ?Sized + fmt::Display {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.as_ref().fmt(f)
    }
}

/// Provide a interned pool so your custom type can be interned.
///
/// # Example
///
/// ```
/// #[macro_use]
/// extern crate internship;
///
/// use internship::intern;
///
/// #[derive(Clone, Hash, PartialEq, Eq)] // required
/// struct CustomData(u32, bool);
///
/// allow_intern!(CustomData);
///
/// // Now you can use `Intern<CustomData>`
/// # fn main() {
/// let _ = intern(CustomData(3, true));
/// # }
/// ```
macro_rules! allow_intern {
    ($T:ty) => (
        impl private::Guard for $T {}
        impl AllowIntern for $T {
            unsafe fn provide_per_thread_intern_pool() ->
                &'static ::std::thread::LocalKey<
                        ::std::cell::RefCell<
                            ::std::collections::HashSet<
                                ::std::rc::Rc<$T>
                            >
                        >
                    >
            {
                thread_local! {
                    static POOL: (
                        ::std::cell::RefCell<
                            ::std::collections::HashSet<
                                ::std::rc::Rc<$T>
                            >
                        >
                    ) = Default::default();
                }

                &POOL
            }
        }
    );
}

allow_intern!{str}
allow_intern!{[u8]}

#[cfg(feature = "shared_from_slice2")]
allow_intern!{CStr}
#[cfg(feature = "shared_from_slice2")]
allow_intern!{OsStr}
#[cfg(feature = "shared_from_slice2")]
allow_intern!{Path}

#[cfg(feature = "serde-compat")]
mod serde_support;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eq_check() {
        assert_eq!(intern("foo"), Intern::new("foo"));
        assert_eq!(&*Intern::new("bar"), "bar");
    }
}
