Internship
===========

[![Docs.rs](https://docs.rs/internship/badge.svg)](https://docs.rs/internship)
[![Build Status](https://travis-ci.org/HyeonuPark/internship-rs.svg?branch=master)](https://travis-ci.org/HyeonuPark/internship-rs)
[![codecov](https://codecov.io/gh/HyeonuPark/internship-rs/branch/master/graph/badge.svg)](https://codecov.io/gh/HyeonuPark/internship-rs)

Interned string and more for rust.

# What is interning?

Interning is a method to store exactly one copy of immutable data.

Imagine your program holds lots of string values, mostly same value in it,
and does not mutate them at all. If you use `String` to store them,
lots of memories are wasted just for storing identical texts.

Interning efficiently eliminate this problem by managing global pool of cache,
in the case above the type of the pool can be `HashSet<Rc<str>>`.
When you need a new owned string, first you should lookup global pool for it.
If desired string is found then use it.
If not, just create a new one and put them also to the pool.

# What does this library provide?

This crate exposes `IStr` type correspond to `Rc<str>`
but guaranteed to be unique over its value within thread
and provide fast O(1) comparison and hashing.

You can also find `IBytes`, `ICStr`, `IOsStr` and `IPath` in this crate,
each correspond to `Rc` of `[u8]`, `CStr`, `OsStr`, and `Path` respectively.

# Why should I use `IStr` over `Rc<str>`?

`IStr` has some advantages over `Rc<str>`

1. Space efficient

  As only single allocation is happen per unique value,
  you can even spam `IStr::new()` without worrying about memory bloat.

1. Cheap equality check

  As only one copy of unique value can be exist,
  comparing two `IStr` can be done with just single pointer comparison
  instead comparing entire contents of strings.

1. Cheap hash calculation

  Again, as only one copy of unique value can be exist,
  its allocated memory address can represent underlying value
  so calculating hash over its pointer makes perfect sense to hash `IStr`.
  Now you can perform blazingly-fast hash map lookup with arbitrary string key!

# License

This repository is dual-licensed under the [MIT license][license-mit]
and [Apache license 2.0][license-apl] at your option.
By contributing to Internship you agree that your contributions will be licensed
under these two licenses.

<!-- links -->

[license-mit]: ./LICENSE-MIT
[license-apl]: ./LICENSE-APACHE
