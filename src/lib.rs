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
//! Or, you can just use `internship` and ignore all the hassle. Why not?
//!
//! # What does this library provide?
//!
//! This crate exposes `IStr` type correspond to `Rc<str>`
//! but guaranteed to be unique over its value within thread.
//! Every instance of `IStr` are per-thread cached to archive this goal.
//!
//! Additionally, `IStr` does not heap-allocate small strings that can be fit on
//! stack. Size limit of inlined string is `2 * sizeof ptr - 1`, typically 15 byte
//! on 64bit machine.

#[cfg(feature = "serde-compat")]
extern crate serde;

mod handle;
mod istr;
mod ibytes;
mod icstr;
mod iosstr;
mod ipath;

pub use istr::IStr;
pub use ibytes::IBytes;

// TODO: implement other types
