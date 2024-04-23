use crate::drust_std::alloc::LOCAL_ALLOCATOR;

use super::{*, dvec::*};

pub(crate) type DString = DVec<u8>;
pub(crate) type DStringRef<'a> = DVecRef<'a, u8>;
pub(crate) type DStringMut<'a> = DVecMutRef<'a, u8>;