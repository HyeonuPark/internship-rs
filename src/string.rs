use std::str::FromStr;
use std::string::ParseError;
use std::ops::Add;
use std::net::ToSocketAddrs;
use std::io;

use super::Intern;

impl FromStr for Intern<str> {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, ParseError> {
        Ok(Intern::new(s))
    }
}

impl<'a> Add<&'a str> for Intern<str> {
    type Output = Self;

    fn add(self, other: &'a str) -> Self {
        let mut buf = String::with_capacity(self.len() + other.len());
        buf.push_str(&self);
        buf.push_str(other);
        Intern::new(&buf)
    }
}

impl ToSocketAddrs for Intern<str> {
    type Iter = <str as ToSocketAddrs>::Iter;

    fn to_socket_addrs(&self) -> io::Result<Self::Iter> {
        (&**self).to_socket_addrs()
    }
}
