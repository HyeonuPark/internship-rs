use std::ffi::{CStr, CString};
use std::ops::{Deref, Index, RangeFull};
use std::hash::{Hash, Hasher};
use std::borrow::Borrow;
use std::str::{from_utf8, Utf8Error};

use handle::Handle;
use ibytes::IBytes;
use istr::IStr;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ICStr(pub(crate) Handle);

impl ICStr {
    #[inline]
    pub fn new(src: &CStr) -> Self {
        ICStr(Handle::new(src.to_bytes_with_nul()))
    }

    #[inline]
    pub fn as_cstr(&self) -> &CStr {
        unsafe {
            CStr::from_bytes_with_nul_unchecked(self.0.get())
        }
    }

    /// result slice does *not* contains trailing nul terminator.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        let length = self.0.get().len();
        &self.0.get()[..length - 1]
    }

    #[inline]
    pub fn as_bytes_with_nul(&self) -> &[u8] {
        self.0.get()
    }

    #[inline]
    pub fn to_ibytes_with_nul(&self) -> IBytes {
        IBytes(self.0.clone())
    }

    #[inline]
    pub fn to_istr(&self) -> Result<IStr, Utf8Error> {
        from_utf8(self.as_bytes()).map(|_| IStr(self.0.clone()))
    }
}

impl Deref for ICStr {
    type Target = CStr;

    #[inline]
    fn deref(&self) -> &CStr {
        self.as_cstr()
    }
}

impl From<CString> for ICStr {
    #[inline]
    fn from(v: CString) -> Self {
        ICStr::new(&v)
    }
}

impl<'a> From<&'a CStr> for ICStr {
    #[inline]
    fn from(v: &'a CStr) -> Self {
        ICStr::new(v)
    }
}

impl From<Box<CStr>> for ICStr {
    #[inline]
    fn from(v: Box<CStr>) -> Self {
        ICStr::new(&v)
    }
}

impl Default for ICStr {
    #[inline]
    fn default() -> Self {
        ICStr::new(Default::default())
    }
}

impl Hash for ICStr {
    #[inline]
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.as_cstr().hash(hasher)
    }
}

impl Borrow<CStr> for ICStr {
    #[inline]
    fn borrow(&self) -> &CStr {
        self.as_cstr()
    }
}

impl Index<RangeFull> for ICStr {
    type Output = CStr;

    #[inline]
    fn index(&self, _index: RangeFull) -> &CStr {
        self.as_cstr()
    }
}

impl AsRef<CStr> for ICStr {
    #[inline]
    fn as_ref(&self) -> &CStr {
        self.as_cstr()
    }
}
