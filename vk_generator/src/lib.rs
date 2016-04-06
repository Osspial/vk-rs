extern crate vk_api;
extern crate xml;

mod crawler;

use xml::EventReader;
use xml::attribute::OwnedAttribute;

use std::os::raw::c_char;
use std::ffi::CStr;
use std::ptr;
use std::fmt;

pub fn load_xml() {
    let vk_xml = EventReader::new(vk_api::VK_XML);
    // The vulkan registry
    let registry = crawler::crawl(vk_xml.into_iter(), VkRegistry::new(vk_api::VK_XML.len()));
}

pub struct VkRegistry {
    string_buffer: String,
    types: Vec<VkType>
}

impl<'a> VkRegistry {
    fn new(capacity: usize) -> VkRegistry {
        VkRegistry {
            string_buffer: String::with_capacity(capacity),
            types: Vec::with_capacity(256)
        }
    }

    fn push_type(&mut self, vk_type: VkType) -> Result<(), ()> {
        match vk_type {
            VkType::Unhandled => Err(()),
            vk_type           => {self.types.push(vk_type); Ok(())}
        }
    }

    /// Append a given attribute to the internal string buffer and return a pointer to a CStr that represents the attribute
    fn append_str(&mut self, string: &str) -> *const c_char {
        // We want to have all of the string in one block of memory in order to save heap allocation time. 
        self.string_buffer.push_str(string);
        self.string_buffer.push('\0');

        (self.string_buffer.as_ptr() as usize + self.string_buffer.len() - string.len() - 1) as *const c_char
    }
}

enum VkMemberType {
    Var(*const c_char),
    Ptr(*const c_char),
    PtrMut(*const c_char),
    /// Default value to initialize with.
    Unknown
}

impl fmt::Debug for VkMemberType {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use VkMemberType::*;
        if let Unknown = *self {
            write!(fmt, "Unknown")
        } else {
            let pt;
            fmt.debug_tuple(
                match *self {
                    Var(p) => {pt = p; "Var"}
                    Ptr(p) => {pt = p; "Ptr"}
                    PtrMut(p) => {pt = p; "PtrMut"}
                    Unknown => unreachable!()
                })
                .field(unsafe{ &if pt != ptr::null() {CStr::from_ptr(pt).to_str().unwrap()} else {"Error: Null Ptr"} })
                .finish()
        }
    }
}

#[derive(Debug)]
struct VkMember {
    field_type: VkMemberType,
    field_name: *const c_char,
}

impl VkMember {
    fn empty() -> VkMember {
        VkMember {
            field_type: VkMemberType::Unknown,
            field_name: ptr::null()
        }
    }

    fn type_ptr(&self) -> Option<*const c_char> {
        use VkMemberType::*;
        match self.field_type {
            Var(s) |
            Ptr(s) |
            PtrMut(s) => Some(s),
            Unknown => None
        }
    }

    fn set_type(&mut self, field_type: *const c_char) {
        use VkMemberType::*;
        match self.field_type {
            Var(ref mut s) |
            Ptr(ref mut s) |
            PtrMut(ref mut s) => 
                if *s == ptr::null() {
                    *s = field_type
                } else {panic!("Field type already set")},
            Unknown => self.field_type = Var(field_type)

        }
    }

    fn change_type_var(&mut self) {
        use VkMemberType::*;
        match self.field_type {
            Var(s) |
            Ptr(s) |
            PtrMut(s) => self.field_type = Var(s),
            Unknown   => self.field_type = Var(ptr::null())
        }
    }

    fn change_type_ptr(&mut self) {
        use VkMemberType::*;
        match self.field_type {
            Var(s) |
            Ptr(s) |
            PtrMut(s) => self.field_type = Ptr(s),
            Unknown   => self.field_type = Ptr(ptr::null())
        }
    }

    fn change_type_ptr_mut(&mut self) {
        use VkMemberType::*;
        match self.field_type {
            Var(s) |
            Ptr(s) |
            PtrMut(s) => self.field_type = PtrMut(s),
            Unknown   => self.field_type = PtrMut(ptr::null())
        }
    }

    fn set_name(&mut self, field_name: *const c_char) {
        if field_name != ptr::null() {
            panic!("Unexpected \"name\" tag");
        } else {
            self.field_name = field_name;
        }
    }
}

/// A variant of a vulkan enum
#[derive(Debug)]
struct VkVariant {
    name: *const c_char,
    value: isize
}

#[derive(Debug)]
enum VkType {
    Struct {
        name: *const c_char,
        fields: Vec<VkMember>
    },

    Union {
        name: *const c_char,
        variants: Vec<VkMember>
    },

    Enum {
        name: *const c_char,
        variants: Vec<VkVariant>
    },

    BaseType {
        /// The type that is being aliased
        ty: *const c_char,
        /// The name of the new type
        name: *const c_char
    },

    Unhandled
}