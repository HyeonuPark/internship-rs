use std::ops::{Deref, Index, Range, RangeFrom, RangeTo, RangeFull};
use std::borrow::{Cow, Borrow};
use std::string::ParseError;
use std::cmp::PartialEq;
use std::hash::{Hash, Hasher};
use std::str::{self, FromStr, from_utf8, Utf8Error};
use std::fmt;
use std::net::ToSocketAddrs;

use handle::Handle;
use ibytes::IBytes;

/// Interned string type
///
/// `IStr` is designed for drop-in-replacement of immutable `String`.
///
/// Conceptually, `IStr` is similar to `Rc<str>` as both are immutable and zero-copy shareable.
/// But `IStr` is interned, and inlined if small enough.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct IStr(pub(crate) Handle);

impl IStr {
    pub fn new(src: &str) -> Self {
        IStr(Handle::new(src.as_bytes()))
    }

    pub fn from_utf8(src: &[u8]) -> Result<Self, Utf8Error> {
        from_utf8(src).map(IStr::new)
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        unsafe {
            str::from_utf8_unchecked(self.0.get())
        }
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        self.0.get()
    }

    #[inline]
    pub fn to_ibytes(&self) -> IBytes {
        IBytes(self.0.clone())
    }
}

impl Deref for IStr {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl From<String> for IStr {
    fn from(v: String) -> Self {
        IStr::new(&v)
    }
}

impl<'a> From<&'a str> for IStr {
    fn from(v: &str) -> Self {
        IStr::new(v)
    }
}

impl From<Box<str>> for IStr {
    fn from(v: Box<str>) -> Self {
        IStr::new(&v)
    }
}

impl<'a> From<Cow<'a, str>> for IStr {
    fn from(v: Cow<str>) -> Self {
        IStr::new(&v)
    }
}

impl<'a> PartialEq<Cow<'a, str>> for IStr {
    fn eq(&self, other: &Cow<str>) -> bool {
        PartialEq::eq(self.as_str(), other)
    }
}

impl PartialEq<String> for IStr {
    fn eq(&self, other: &String) -> bool {
        PartialEq::eq(self.as_str(), other)
    }
}

impl<'a> PartialEq<&'a str> for IStr {
    fn eq(&self, other: &&str) -> bool {
        PartialEq::eq(self.as_str(), *other)
    }
}

impl PartialEq<str> for IStr {
    fn eq(&self, other: &str) -> bool {
        PartialEq::eq(self.as_str(), other)
    }
}

impl Default for IStr {
    #[inline]
    fn default() -> Self {
        IStr::new("")
    }
}

impl Hash for IStr {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        Hash::hash(self.as_str(), hasher)
    }
}

impl Borrow<str> for IStr {
    #[inline]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl Index<Range<usize>> for IStr {
    type Output = str;

    #[inline]
    fn index(&self, index: Range<usize>) -> &str {
        &self.as_str()[index]
    }
}

impl Index<RangeFrom<usize>> for IStr {
    type Output = str;

    #[inline]
    fn index(&self, index: RangeFrom<usize>) -> &str {
        &self.as_str()[index]
    }
}

impl Index<RangeTo<usize>> for IStr {
    type Output = str;

    #[inline]
    fn index(&self, index: RangeTo<usize>) -> &str {
        &self.as_str()[index]
    }
}

impl Index<RangeFull> for IStr {
    type Output = str;

    #[inline]
    fn index(&self, index: RangeFull) -> &str {
        &self.as_str()[index]
    }
}

impl FromStr for IStr {
    type Err = ParseError;

    fn from_str(src: &str) -> Result<Self, ParseError> {
        Ok(IStr::new(src))
    }
}

impl AsRef<str> for IStr {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<[u8]> for IStr {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl fmt::Debug for IStr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl fmt::Display for IStr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

impl ToSocketAddrs for IStr {
    type Iter = <str as ToSocketAddrs>::Iter;

    fn to_socket_addrs(&self) -> ::std::io::Result<Self::Iter> {
        ToSocketAddrs::to_socket_addrs(self.as_str())
    }
}

#[cfg(feature = "serde-compat")]
mod serde_compat {
    use super::*;
    use serde::{Serialize, Serializer, Deserialize, Deserializer, de};

    impl Serialize for IStr {
        fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
            Serialize::serialize(self.as_str(), s)
        }
    }

    impl<'d> Deserialize<'d> for IStr {
        fn deserialize<D: Deserializer<'d>>(d: D) -> Result<IStr, D::Error> {
            d.deserialize_str(Visitor)
        }
    }

    pub struct Visitor;

    impl<'d> de::Visitor<'d> for Visitor {
        type Value = IStr;

        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("string slice")
        }

        fn visit_str<E: de::Error>(self, value: &str) -> Result<IStr, E> {
            Ok(IStr::new(value))
        }
    }
}
