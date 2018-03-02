extern crate serde;

use std::borrow::ToOwned;
use std::hash::Hash;
use std::rc::Rc;
use std::borrow::Borrow;

use self::serde::{Serialize, Deserialize, Serializer, Deserializer};
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
