extern crate serde;

use std::borrow::ToOwned;
use std::hash::Hash;
use std::rc::Rc;
use std::borrow::Borrow;
use std::fmt;

use self::serde::{Serialize, Deserialize, Serializer, Deserializer};
use self::serde::de::{Visitor, Error};
use super::{Intern, AllowIntern};

impl<T> Serialize for Intern<T> where T: AllowIntern + ?Sized + Serialize {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        (&**self).serialize(ser)
    }
}

impl<'de, T> Deserialize<'de> for Intern<T> where
    T: AllowIntern + ToOwned,
    for<'a> &'a T: Into<Rc<T>>,
    <T as ToOwned>::Owned: Deserialize<'de> + Into<Rc<T>> + Hash + Eq,
{
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        <T as ToOwned>::Owned::deserialize(de).map(|o| Self::new(o.borrow()))
    }
}

pub struct StrVisitor;

impl<'de> Visitor<'de> for StrVisitor {
    type Value = Intern<str>;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("string slice")
    }

    fn visit_str<E: Error>(self, value: &str) -> Result<Intern<str>, E> {
        Ok(Intern::new(value))
    }
}

impl<'de> Deserialize<'de> for Intern<str> {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        de.deserialize_str(StrVisitor)
    }
}

pub struct BytesVisitor;

impl<'de> Visitor<'de> for BytesVisitor {
    type Value = Intern<[u8]>;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("byte slice")
    }

    fn visit_bytes<E: Error>(self, value: &[u8]) -> Result<Intern<[u8]>, E> {
        Ok(Intern::new(value))
    }
}

impl<'de> Deserialize<'de> for Intern<[u8]> {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        de.deserialize_bytes(BytesVisitor)
    }
}
