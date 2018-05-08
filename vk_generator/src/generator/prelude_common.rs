pub use self::__vk_generator_types::*;
#[allow(non_upper_case_globals)]
mod __vk_generator_types {

pub use std::os::raw::c_ulonglong;
pub use self::types::*;
pub use self::cmds::*;

#[doc(hidden)]
#[allow(dead_code)]
pub fn unloaded_function_panic() -> ! {
    panic!("Attempted to run unloaded vulkan function")
}

#[allow(unused_macros)]
macro_rules! handle_nondispatchable {
    ($name: ident) => {
        #[repr(C)]
        #[derive(Eq, PartialEq, Ord, PartialOrd, Clone, Copy, Hash)]
        pub struct $name (uint64_t);

        impl fmt::Pointer for $name {
            fn fmt(&self, f: &mut fmt::Formatter) -> ::std::result::Result<(), fmt::Error> {
                write!(f, "0x{:x}", self.0)
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter) -> ::std::result::Result<(), fmt::Error> {
                write!(f, "0x{:x}", self.0)
            }
        }
    }
}

// Only used if `wrap_bitmasks == true`.
//TODO: use bitflags crate?
#[allow(unused_macros)]
macro_rules! vk_bitflags_wrapped {
    ($name: ident, $all: expr, $flag_type: ty) => {
        #[repr(C)]
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name {flags: $flag_type}

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter) -> ::std::result::Result<(), fmt::Error> {
                write!(f, "{}({:b})", stringify!($name), self.flags)
            }
        }

        impl $name {
            #[inline]
            pub fn empty() -> Self {
                $name {flags: 0}
            }

            #[inline]
            pub fn all() -> Self {
                $name {flags: $all}
            }

            #[inline]
            pub fn flags(self) -> $flag_type {
                self.flags
            }

            #[inline]
            pub fn from_flags(flags: $flag_type) -> Option<Self> {
                if flags & !$all == 0 {
                    Some($name {flags: flags})
                } else {
                    None
                }
            }

            #[inline]
            pub fn from_flags_truncate(flags: $flag_type) -> Self {
                $name {flags: flags & $all}
            }

            #[inline]
            pub fn is_empty(self) -> bool {
                self == Self::empty()
            }

            #[inline]
            pub fn is_all(self) -> bool {
                self & Self::all() == Self::all()
            }

            #[inline]
            pub fn intersects(self, other: $name) -> bool {
                self & other != Self::empty()
            }

            /// Returns true of `other` is a subset of `self`
            #[inline]
            pub fn subset(self, other: $name) -> bool {
                self & other == other
            }
        }

        impl BitOr for $name {
            type Output = $name;

            #[inline]
            fn bitor(self, rhs: $name) -> Self {
                $name {flags: self.flags | rhs.flags }
            }
        }

        impl BitOrAssign for $name {
            #[inline]
            fn bitor_assign(&mut self, rhs: Self) {
                *self = *self | rhs
            }
        }

        impl BitAnd for $name {
            type Output = Self;

            #[inline]
            fn bitand(self, rhs: Self) -> Self {
                $name {flags: self.flags & rhs.flags}
            }
        }

        impl BitAndAssign for $name {
            #[inline]
            fn bitand_assign(&mut self, rhs: Self) {
                *self = *self & rhs
            }
        }

        impl BitXor for $name {
            type Output = $name;

            #[inline]
            fn bitxor(self, rhs: Self) -> Self {
                $name {flags: self.flags ^ rhs.flags}
            }
        }

        impl BitXorAssign for $name {
            #[inline]
            fn bitxor_assign(&mut self, rhs: Self) {
                *self = *self ^ rhs
            }
        }

        impl Sub for $name {
            type Output = Self;

            #[inline]
            fn sub(self, rhs: Self) -> Self {
                self & !rhs
            }
        }

        impl SubAssign for $name {
            #[inline]
            fn sub_assign(&mut self, rhs: Self) {
                *self = *self - rhs
            }
        }

        impl Not for $name {
            type Output = Self;

            #[inline]
            fn not(self) -> Self {
                self ^ Self::all()
            }
        }
    }
}
