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
//! The core of `Internship` is `struct Intern<T>`. You can think of it as `Rc<T>`,
//! but guaranteed uniqueness over value and thread.
//!
//! # Example
//!
//! 1. Interning string
//!
//!   ```
//!   extern crate internship;
//!   use internship::Intern;
//!   # use std::collections::HashMap;
//!
//!   # fn main() {
//!   let foo = Intern::new("foo"); // type is Intern<str>
//!   let foo2 = Intern::new("foo"); // reuse foo's buffer
//!
//!   let mut map = HashMap::new();
//!   map.insert(Intern::new("key"), 42);
//!   assert_eq!(map.get(&Intern::new("key")), Some(&42));
//!   # }
//!   ```
//!
//! 1. Interning custom type
//!
//!   ```
//!   #[macro_use]
//!   extern crate internship;
//!   use internship::Intern;
//!
//!   #[derive(Clone, Hash, PartialEq, Eq)] // required
//!   struct CustomData(u32, bool);
//!
//!   intern!(CustomData); // now it's ready for interning
//!
//!   # fn main() {
//!   let my_data = Intern::from(CustomData(3, true));
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
//!   you can even span `Intern::new()` without worrying about memory bloat.
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
//!   its allocated memory address can represent underlying value.
//!   So you can perform blazingly-fast hashmap lookup with string value.
//!
//! # What primitive types can be interned?
//!
//! Currently these types below are supported.
//!
//! - `str`
//! - `CStr`
//! - `OsStr`
//! - `Path`
//! - `[u8]`
//!
//! You can find interned type of them as re-export, like `InternStr`.

use std::rc::Rc;
use std::hash::{Hash, Hasher};
use std::cell::RefCell;
use std::collections::HashSet;
use std::thread::LocalKey;
use std::ops::{Deref, Drop};

use std::ffi::{CStr, OsStr};
use std::path::Path;

pub type InternStr = Intern<str>;
pub type InternByteStr = Intern<[u8]>;
pub type InternCStr = Intern<CStr>;
pub type InternOsStr = Intern<OsStr>;
pub type InternPath = Intern<Path>;

/// Interned data
///
/// `Intern<T>` is conceptually same as `Rc<T>` but unique per value per thread.
///
/// # Advantages
///
/// `Intern<T>` has some advantages over `Rc<T>` that..
///
/// 1. Space efficient
///
///   As only single allocation is happen per unique value,
///   you can span `Intern::new()` without worring memory bloat.
///
/// 1. Cheap equality check
///
///   As only single copy of unique value can be exists,
///   comparing two `Intern`s is done with just single pointer-comparison
///   instead comparing all bytes of strings.
///
/// 1. Cheap hash calculation
///
///   Again, as only single copy of unique value can be exists,
///   its allocated memory address can represent underlying value.
///   So you can perform blazingly-fast hashmap lookup for string keys.
#[derive(Debug, Clone, Eq)]
pub struct Intern<T: InternContent + ?Sized>(Rc<T>);

/// Intern-able data
///
/// Generally, use provided `intern!` macro instead to allow your type to be interned.
pub unsafe trait InternContent: Eq + Hash + 'static {
    /// Provide thread-local interned pool for this type.
    ///
    /// This is necessary as Rust doesn't allow static variables with generic type,
    /// as this can't be monomorphized trivially.
    ///
    /// Calling this function is `unsafe` because `Intern` relies on assumption that
    /// provided pool never change except for the construction/destruction of the `Intern`.
    unsafe fn provide_interned_pool() -> &'static LocalKey<RefCell<HashSet<Rc<Self>>>>;
}

impl<'a, T> Intern<T> where T: InternContent + ?Sized, &'a T: Into<Rc<T>> {
    /// Create new `Intern` from `str` like type, if cached data not found.
    ///
    /// This function always perform thread-local hashmap lookup.
    /// So `Intern::clone()` is still more efficient then
    /// repeated `Intern::new()` with same data.
    pub fn new(content: &'a T) -> Self {
        unsafe {
            T::provide_interned_pool().with(|pool| {
                let mut pool = pool.borrow_mut();
                let cached = pool.get(content).cloned();

                match cached {
                    Some(value) => Intern(value),
                    None => {
                        let value = content.into();
                        pool.insert(Rc::clone(&value));
                        Intern(value)
                    }
                }
            })
        }
    }
}

impl<T: InternContent> From<T> for Intern<T> {
    fn from(content: T) -> Self {
        unsafe {
            T::provide_interned_pool().with(|pool| {
                let mut pool = pool.borrow_mut();
                let cached = pool.get(&content).cloned();

                match cached {
                    Some(value) => Intern(value),
                    None => {
                        let value = Rc::new(content);
                        pool.insert(Rc::clone(&value));
                        Intern(value)
                    }
                }
            })
        }
    }
}

impl<'a, T> From<&'a T> for Intern<T> where T: InternContent + ?Sized, &'a T: Into<Rc<T>> {
    fn from(data: &'a T) -> Self {
        Intern::new(data)
    }
}

impl<T: InternContent + ?Sized> Deref for Intern<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl<T: InternContent + ?Sized> Drop for Intern<T> {
    fn drop(&mut self) {
        // strong count == 2 means
        // no other copies of this interned value exist
        // other then the `self` which will be dropped
        // and the one in the pool.
        if Rc::strong_count(&self.0) == 2 {
            unsafe {
                T::provide_interned_pool().with(|pool| {
                    pool.borrow_mut().remove(&self.0);
                });
            }
        }
    }
}

impl<T: InternContent + ?Sized> PartialEq<Self> for Intern<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl<T: InternContent + ?Sized> Hash for Intern<T> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        let ptr = Rc::into_raw(Rc::clone(&self.0));
        unsafe { Rc::from_raw(ptr) };
        ptr.hash(hasher)
    }
}

/// Provide a interned pool so your custom data can be interned.
///
/// # Example
///
/// ```
/// #[macro_use]
/// extern crate internship;
///
/// use internship::Intern;
///
/// #[derive(Clone, Hash, PartialEq, Eq)]
/// struct CustomData(u32, bool);
///
/// intern!(CustomData);
///
/// // Now you can use `Intern<CustomData>`
/// # fn main() {
/// let _ = Intern::from(CustomData(3, true));
/// # }
/// ```
#[macro_export]
macro_rules! intern {
    ($T:ty) => (
        unsafe impl $crate::InternContent for $T {
            unsafe fn provide_interned_pool() ->
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

intern!{str}
intern!{[u8]}
intern!{CStr}
intern!{OsStr}
intern!{Path}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eq_check() {
        assert_eq!(InternStr::new("foo"), InternStr::new("foo"));
        assert_eq!(&*InternStr::new("bar"), "bar");
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    struct Dummy(usize);

    intern!(Dummy);

    #[test]
    fn test_remove_cache_on_destruct() {
        let pool_size = || unsafe {
            Dummy::provide_interned_pool().with(|pool| {
                pool.borrow().len()
            })
        };

        assert_eq!(pool_size(), 0);
        let d1 = Intern::<Dummy>::from(Dummy(7));
        assert_eq!(pool_size(), 1);
        drop(d1);
        assert_eq!(pool_size(), 0);
    }
}
