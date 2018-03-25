use std::ops::{Deref, Index, Range, RangeFrom, RangeTo, RangeFull};
use std::borrow::{Cow, Borrow};
use std::string::ParseError;
use std::cmp::PartialEq;
use std::hash::{Hash, Hasher};
use std::str::{self, FromStr};
use std::fmt;
use std::net::ToSocketAddrs;

use handle::Handle;

/// Interned string type
///
/// `IStr` is designed for drop-in-replacement of immutable `String`.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct IStr(Handle);

// TODO: impl serde::{Serialize, Deserialize}

impl IStr {
    pub fn new(src: &str) -> Self {
        IStr(Handle::new(src.as_bytes()))
    }

    pub fn as_str(&self) -> &str {
        unsafe {
            str::from_utf8_unchecked(self.0.get())
        }
    }
}

impl Deref for IStr {
    type Target = str;

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
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl Index<Range<usize>> for IStr {
    type Output = str;

    fn index(&self, index: Range<usize>) -> &str {
        Index::index(self.as_str(), index)
    }
}

impl Index<RangeFrom<usize>> for IStr {
    type Output = str;

    fn index(&self, index: RangeFrom<usize>) -> &str {
        Index::index(self.as_str(), index)
    }
}

impl Index<RangeTo<usize>> for IStr {
    type Output = str;

    fn index(&self, index: RangeTo<usize>) -> &str {
        Index::index(self.as_str(), index)
    }
}

impl Index<RangeFull> for IStr {
    type Output = str;

    fn index(&self, _index: RangeFull) -> &str {
        self.as_str()
    }
}

impl FromStr for IStr {
    type Err = ParseError;

    fn from_str(src: &str) -> Result<Self, ParseError> {
        Ok(IStr::new(src))
    }
}

impl AsRef<str> for IStr {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<[u8]> for IStr {
    fn as_ref(&self) -> &[u8] {
        self.0.get()
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