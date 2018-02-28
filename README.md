Internship
===========

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

The core of `Internship` is `struct Intern<T>`. You can think of it as `Rc<T>`,
but guaranteed uniqueness over value and thread.

# Example

1. Interning string
```
extern crate internship;
use internship::Intern;

let foo = Intern::new("foo"); // type is Intern<str>
let foo2 = Intern::new("foo"); // reuse foo's buffer

let mut map = HashMap::new();
map.insert(Intern::new("key"), 42);
assert_eq!(map.get(&Intern::new("key")), Some(&42));
```

1. Interning custom type
```
#[macro_use]
extern crate internship;
use internship::Intern;

struct CustomData(u32, bool);
intern!(CustomData); // this is all you need to interning

let my_data = Intern::new(&CustomData(3, true))
```

# How is `Intern<T>` better then `Rc<T>`?

`Intern<T>` has some advantages over `Rc<T>`

1. Space efficient

  As only single allocation is happen per unique value,
  you can even span `Intern::new()` without worrying about memory bloat.

1. Cheap equality check

  As only one copy of unique value can be exist,
  comparing two `Intern<T>` can be done with just single pointer comparison
  instead comparing every bytes of strings.

1. Cheap hash calculation

  Again, as only one copy of unique value can be exist,
  its allocated memory address can represent underlying value.
  So you can perform blazingly-fast hashmap lookup with string value.

# License

This repository is dual-licensed under the [MIT license][license-mit]
and [Apache license 2.0][license-apl] at your option.
By contributing to Nal you agree that your contributions will be licensed
under these two licenses.

<!-- links -->

[license-mit]: ./LICENSE-MIT
[license-apl]: ./LICENSE-APACHE
