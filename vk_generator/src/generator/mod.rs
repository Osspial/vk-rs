use to_option;
use registry::*;
use std::collections::{HashMap, HashSet};
use std::iter::Iterator;
use std::default;
use std::fmt::Write as FmtWrite;
use std::io::Write;

use boolinator::Boolinator;

/// Configuration for handling enum variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariantPaddingConfig {
    /// Keep variants unchanged.
    Keep,
    /// Remove `VK_` prefix only.
    RemovePrefix,
    /// Remove the prefix part, type name, and the extension suffix.
    Strip,
}

/// Configuration options fot the Vulkan generator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GenConfig<'a> {
    /// Whether or not to remove the `Vk` prefix on structs, enums, and typedefs.
    ///
    /// As an example, take the struct `VkInstanceCreateInfo`. If this is set to `true`, the generator
    /// will turn that into `InstanceCreateInfo`.
    ///
    /// Defaults to `false`.
    pub remove_type_prefix: bool,
    /// Whether or not to remove the `Vk` prefix from the `VkResult` enum, **IF AND ONLY IF**
    /// remove_type_prefix is also set to `true`. This flag exists primarily because Rust already
    /// contains a type named `Result` and one might desire to remove any ambiguity the `VkResult` type
    /// may cause.
    ///
    /// Defaults to `true`.
    pub remove_vk_result_prefix: bool,
    /// Whether or not to remove the `vk` prefix from Vulkan commands.
    ///
    /// For example, if we have the command `vkCreateInstance` setting this flag to `true` will turn
    /// that into `createInstance`.
    ///
    /// Defaults to `true`.
    pub remove_command_prefix: bool,
    /// Whether or not to remove the `VK_` prefix from bitmask variants.
    ///
    /// For example, the xml registry defines the bitmask `VkQueueFlagBits` with the flag
    /// `VK_QUEUE_GRAPHICS_BIT`. If this is `true`, this will result in the variant being turned into
    /// `QUEUE_GRAPHICS_BIT`.
    ///
    /// Defaults to `true`.
    pub remove_bitmask_prefix: bool,
    /// Whether or not to remove the `VK_` prefix from constant names.
    ///
    /// For example, the xml registry defines the constant `VK_MAX_EXTENSION_NAME_SIZE`.
    /// If this is `true`, this will result in the constant being turned into
    /// `MAX_EXTENSION_NAME_SIZE`.
    ///
    /// Defaults to `true`.
    pub remove_const_prefix: bool,
    /// How to handle enum variants, see `VariantPaddingConfig` for details.
    ///
    /// Defaults to `VariantPaddingConfig::Strip`.
    pub variant_padding: VariantPaddingConfig,
    /// Whether or not to transform Vulkan command identifiers to be a Rust-y snake_case.
    ///
    /// For example, the registry defines the command `vkCreateInstance`. If this is `true`, that
    /// command will be turned into `vk_create_instance`. This, and the other name-style-altering
    /// options, primarily exists for the purpose of having Vulkan code integrate more cleanly into
    /// native Rust code.
    ///
    /// Defaults to `true`.
    pub snake_case_commands: bool,
    /// Whether or not to transform enum variants into CamelCase.
    ///
    /// For example, if we look at `VkStructureType`'s `VK_STRUCTURE_TYPE_APPLICATION_INFO` setting
    /// this to `true` would result in the variant being turned into `VkStructureTypeApplicationInfo`.
    ///
    /// Defaults to `true`.
    pub camel_case_variants: bool,
    /// Whether or not to transform struct/union members and command parameters into snake_case.
    ///
    /// For example, if we look at the `VkApplicationInfo` struct's `applicationVersion` field, setting
    /// this to `true` would result in the field being turned into `application_version`.
    ///
    /// Defaults to `true`.
    pub snake_case_members: bool,
    /// When printing structs with fields that are arrays of c_chars, whether to print them as arrays
    /// of bytes or as a string.
    ///
    /// Defaults to `true`.
    pub debug_c_strings: bool,
    /// Whether or not to use the `enum` keyword to create native tagged unions.
    ///
    /// Defaults to `true`.
    pub use_native_enums: bool,
    /// Whether or not to use the unstable `union` keyword to create native untagged unions.
    /// Currently relies on the `untagged_unions` feature.
    ///
    /// Defaults to `false`.
    pub use_native_unions: bool,

    /// Whether or not to wrap bitmasks with a set of convenience functions similar to the
    /// [bitflags](https://doc.rust-lang.org/bitflags/bitflags/macro.bitflags!.html) crate.
    ///
    /// Defaults to `true`.
    pub wrap_bitmasks: bool,
    /// The Vulkan library uses a lot of `C` types, as per it's nature of exposing a `C` ABI. There are
    /// a few ways we can handle using those types: either we can define the typedefs ourself or we can
    /// use the types provided by `libc`. Because `libc` isn't implicitly included in crates we default
    /// to defining the types ourself. Setting this to `true` makes the generated file import types
    /// from `libc` instead of defining them itself.
    pub use_libc_types: bool,
    /// This defines a set of type overrides, primarily intended for use with the WSI extensions. It
    /// takes a slice of (&str, &str) tuples, with the left side being the name of the type and the
    /// right side being the new definition of the type.
    ///
    /// For an example let's look at the Windows WSI extension, which includes the struct
    /// `VkWin32SurfaceCreateInfoKHR`. That struct takes a `HWND` and a `HINSTANCE` in order to let Vulkan
    /// draw to windows; however, the generator is unaware of both `HWND` and `HINSTANCE`, which are
    /// defined in `winapi`. Because it has no idea what those types should be the generator defaults
    /// to `type HWND = *const ()`, which isn't what HWNDs are defined as in `winapi`. So we call this:
    ///
    /// ```
    /// # use vk_generator::GenConfig;
    /// let config = GenConfig {
    ///     extern_type_overrides: &[
    ///         ("HWND", "winapi::HWND"),
    ///         ("HINSTANCE", "winapi::HINSTANCE")
    ///     ],
    ///     ..GenConfig::default()
    /// };
    /// ```
    ///
    /// This tells the generator to use the `winapi` defintions of HWND instead of the blind
    /// definition, making the generator produce these for the type defintions:
    ///
    /// ```
    /// # mod winapi {
    /// #     // That's right, they're just being defined as `*const ()` again. Whatcha gonna do?
    /// #     //
    /// #     // In all seriousness, this is just here to make the example compile with `cargo test`. It would really
    /// #     // link to `winapi` if we were actually using the generator.
    /// #     pub type HWND = *const ();
    /// #     pub type HINSTANCE = *const ();
    /// # }
    /// type HWND = winapi::HWND;
    /// type HINSTANCE = winapi::HINSTANCE;
    /// ```
    pub extern_type_overrides: &'a [(&'a str, &'a str)]
}

impl<'a> GenConfig<'a> {
    /// Create a new generator config. Is identical to `Default::default()`.
    pub fn new() -> Self {
        Default::default()
    }
}

impl<'a> default::Default for GenConfig<'a> {
    fn default() -> Self {
        GenConfig {
            remove_type_prefix: false,
            remove_vk_result_prefix: true,
            remove_command_prefix: true,
            remove_bitmask_prefix: true,
            remove_const_prefix: true,
            variant_padding: VariantPaddingConfig::Strip,
            snake_case_commands: true,
            camel_case_variants: true,
            snake_case_members: true,
            debug_c_strings: true,
            use_native_enums: true,
            use_native_unions: false,

            wrap_bitmasks: true,
            use_libc_types: false,
            extern_type_overrides: &[]
        }
    }
}

pub struct GenPreproc<'a, 'b> {
    pub types: HashMap<&'a str, VkType>,
    /// The order in which types are loaded. Not technically necessary, but it can be used to guarantee
    /// that the types are output to the generated file in a constant order.
    pub type_ord: Vec<&'a str>,
    /// Possible types for constants that can be easily assumed by the preprocessor. This is not a
    /// comprehensive list of all constant types, probably won't contain the types of all constants
    /// defined by the API. Currently, it just tells what types are `usize`.
    pub const_types: HashMap<&'a str, ConstType>,
    /// A set of all types that can't have Debug and Clone implementations derived. This occurs when
    /// the type contains an array, and as such can't derive Clone (because arrays only implement Clone
    /// if the types they contain are Copy, which all of the generated types aren't) and Debug.
    pub custom_impls: HashSet<&'a str>,
    pub commands: Vec<VkCommand>,
    /// A vector of the unprocessed command names
    pub commands_raw: Vec<&'a str>,
    pub registry: &'a VkRegistry<'a>,
    pub config: GenConfig<'b>,
    /// An internal buffer that contains all relevant identifier strings
    pub string_buffer: String
}

impl<'a, 'b> GenPreproc<'a, 'b> {
    fn new(registry: &'a VkRegistry<'a>, version: VkVersion, extensions: &[&str], config: GenConfig<'b>) -> Self {
        let mut gen = GenPreproc {
            string_buffer: String::with_capacity(registry.buffer_cap()),
            types: HashMap::with_capacity(registry.types().len()),
            type_ord: Vec::with_capacity(registry.types().len()),
            const_types: HashMap::with_capacity(registry.core_consts().len()),
            custom_impls: HashSet::with_capacity(32),
            commands: Vec::with_capacity(registry.commands().len()),
            commands_raw: Vec::with_capacity(registry.commands().len()),
            registry: registry,
            config: config,
        };

        let feature = gen.registry.features().get(&version).unwrap();

        for c in registry.core_consts() {
            gen.add_const(c);
        }

        for req in &feature.require {
            gen.add_interface(req);
        }
        for e in extensions {
            let ex = gen.registry.extns().get(e).unwrap();
            for req in &ex.require {
                gen.add_interface(req);
            }
        }

        gen
    }

    fn add_interface(&mut self, interface: &VkInterface) {
        use registry::VkInterface::*;
        match *interface {
            Command{name, ..} => {
                let name = unsafe{ &*name };
                let mut command = self.registry.commands().get(name).unwrap().clone();
                self.commands_raw.push(unsafe{ &*command.name });

                for p in command.params.iter_mut() {
                    p.name = self.process_member_name(p.name);
                    self.add_type_recurse(&mut p.typ);
                }
                self.add_type_recurse(&mut command.ret);

                self.add_command(command);
            }
            ApiConst{name, ..} => {
                self.add_const(unsafe{ &*name });
            }
            Type{name, ..}     => {
                let name = unsafe{ &*name };
                if name != "vk_platform" {
                    self.add_type_recurse(&mut VkElType::Var(name));
                }
            }
            ConstDef{name, value, ..} => {
                self.add_const_new(unsafe{ &*name }, unsafe{ &*value });
            }
            ExtnEnum{extends, ref variant, ..} => {
                let extends = unsafe{ &*extends };
                let mut variant = variant.clone();
                match *self.types.get(extends).unwrap() {
                    VkType::Enum{name, ..} => self.process_enum_variant(&mut variant, name),
                    VkType::Bitmask{..}    => self.process_bitmask_variant(&mut variant),
                    _                      => ()
                }
                match *self.types.get_mut(extends).unwrap() {
                    VkType::Enum{ref mut variants, ..}     |
                    VkType::Bitmask{ref mut variants, ..} => variants.push(variant),
                    _                                     => ()
                }
            }
        }
    }

    fn add_type_recurse(&mut self, typ: &mut VkElType) {
        use registry::VkType::*;

        let type_ptr = match typ.type_ptr() {
            Some(ptr) => ptr,
            None => return,
        };
        typ.set_type(self.process_type_ident(type_ptr));

        let type_ptr = unsafe{ &*type_ptr };
        match *typ {
            VkElType::ConstArrayEnum(_, ref mut c) |
            VkElType::MutArrayEnum(_, ref mut c)  => {
                let mut cons = unsafe{ &**c };
                if self.config.remove_const_prefix && cons.starts_with("VK_") {
                    cons = &cons[3..];
                    *c = cons as *const _;
                }
                self.const_types.insert(cons, ConstType::USize);
            }
            _ => {}
        };

        let t = match self.add_type(type_ptr) {
            Some(t) => t,
            None => return,
        };
        unsafe {
            let mut custom_impl = false;

            match *t {
                Struct{fields: ref mut members, ..} |
                Union{variants: ref mut members, ..} =>
                    for m in members.iter_mut() {
                        m.field_name = self.process_member_name(m.field_name);
                        match m.field_type {
                            VkElType::ConstArray(_, _)     |
                            VkElType::MutArray(_, _)       |
                            VkElType::ConstArrayEnum(_, _) |
                            VkElType::MutArrayEnum(_, _)  => custom_impl = true,
                            _ => ()
                        };

                        if let FuncPointer{..} = *self.registry.types().get(&*m.field_type.type_ptr().unwrap()).unwrap() {
                            custom_impl = true;
                        }

                        self.add_type_recurse(&mut m.field_type);
                    },
                FuncPointer{ref mut params, ref mut ret, ..} => {
                    for p in params.iter_mut() {
                        self.add_type_recurse(p)
                    }
                    self.add_type_recurse(ret)
                },
                TypeDef{..} =>
                    if let TypeDef{typ, requires, ..} = *self.registry.types().get(type_ptr).unwrap() {
                        self.add_type(&*typ);
                        if let Some(requires) = to_option(requires) {
                            self.add_type(requires);
                        }
                    } else {panic!("Registry type does not match up with modified type")},
                _ => ()
            }

            if custom_impl {
                self.custom_impls.insert(&*(*t).name().unwrap());
            }
        }
    }

    fn process_member_name(&mut self, name: *const str) -> *const str {
        let mut name = unsafe{ &*name };

        if self.config.snake_case_members { unsafe {
            name = &*self.append_char_func(|s| {
                let mut cl = ' ';
                for c in name.chars() {
                    if cl.is_lowercase() && c.is_uppercase() {
                        s.push('_')
                    }
                    s.push(c.to_lowercase().next().unwrap());
                    cl = c;
                }
            });
        }}

        if "type" == name {
            "typ"
        } else {name}
    }

    fn add_const(&mut self, mut name: &'a str) -> Option<*mut VkType> {
        let mut typ = self.registry.types().get(name).unwrap().clone();
        if self.config.remove_const_prefix && name.starts_with("VK_") {
            name = &name[3..];
            typ.set_name(name as *const _).unwrap();
        }
        self.insert_type(name, typ)
    }

    fn add_const_new(&mut self, mut name: &'a str, value: &'a str) -> Option<*mut VkType> {
        if self.config.remove_const_prefix && name.starts_with("VK_") {
            name = &name[3..];
        }
        self.insert_type(name, VkType::new_const(name, value))
    }

    fn add_type(&mut self, name: &'a str) -> Option<*mut VkType> {
        self.insert_type(name, self.registry.types().get(name).unwrap().clone())
    }

    fn insert_type(&mut self, key: &'a str, mut typ: VkType) -> Option<*mut VkType> {
        use std::collections::hash_map::Entry;

        let new_name = self.process_type_ident(typ.name().unwrap());
        typ.set_name(new_name).ok();

        match typ {
            VkType::TypeDef{typ: ref mut typedef_type, ref mut requires, ..} => {
                *typedef_type = self.process_type_ident(*typedef_type);

                if let Some(req) = to_option(*requires) {
                    *requires = self.process_type_ident(req);
                }
            }

            VkType::Enum{ref mut variants, name: enum_name} =>
                for v in variants.iter_mut() {
                    self.process_enum_variant(v, enum_name);
                },

            VkType::Bitmask{ref mut variants, ..} =>
                for v in variants.iter_mut() {
                    self.process_bitmask_variant(v);
                },
            _ => ()
        }

        if let Entry::Vacant(ven) = self.types.entry(key) {
            self.type_ord.push(key);
            Some(ven.insert(typ))
        } else {None}
    }

    fn process_type_ident(&mut self, ident: *const str) -> *const str {
        let mut ident = unsafe{ &*ident };

        match ident {
            "void"      => ident = "c_void",
            "char"      => ident = "c_char",
            "double"    => ident = "c_double",
            "float"     => ident = "c_float",
            "int"       => ident = "c_int",
            "long"      => ident = "c_long",
            "longlong"  => ident = "c_longlong",
            "schar"     => ident = "c_schar",
            "short"     => ident = "c_short",
            "uchar"     => ident = "c_uchar",
            "uint"      => ident = "c_uint",
            "ulong"     => ident = "c_ulong",
            "ulonglong" => ident = "c_ulonglong",
            "ushort"    => ident = "c_ushort",
            _ => ()
        }

        if self.config.remove_type_prefix && (self.config.remove_vk_result_prefix || "VkResult" != ident) {
            if let Some(0) = ident.find("Vk") {
                ident = &ident[2..];
            }
        }

        if let Some(i) = ident.find("FlagBits") {unsafe {
            ident = &*self.append_char_func(|s|
                for c in ident[..i].chars().chain("Flags".chars()).chain(ident[i+8..].chars()) {
                    s.push(c);
                }
            );
        }}

        ident
    }

    fn process_bitmask_variant(&self, variant: &mut VkVariant) {
        let mut name = unsafe{ &*variant.name() };

        if self.config.remove_bitmask_prefix {
            if let Some(0) = name.find("VK_") {
                name = &name[3..];
            }
        }
        variant.set_name(name);
    }

    fn process_enum_variant(&mut self, variant: &mut VkVariant, enum_name: *const str) {
        let enum_name = unsafe{ &*enum_name };
        match self.config.variant_padding {
            VariantPaddingConfig::Strip => {
                let name_parts: Vec<_> = enum_name
                    .char_indices()
                    .filter_map( |(i, c)| (c.is_uppercase()).as_some(i) )
                    .chain(Some(enum_name.len()).into_iter())
                    .peek_next()
                    .map( |(s, e)| enum_name[s..e].to_uppercase() )
                    .collect();
                let vn = unsafe{ &*variant.name() };
                // These are the indicies of various parts of the variant name, shown by example:
                // If the enum is named `VkImageType`, and has the variant `VK_IMAGE_TYPE_1D`:
                //
                // VK_IMAGE_TYPE_1D
                //               ^ `start` is this index
                // VK_IMAGE_TYPE_1D
                //          ^ `old_start` is this index
                //
                // `old_start` is used instead of `start` if `start` points to an index where it could not create a
                // valid identifier (i.e. `1D` is not a valid ident, but `Type1D` or `TYPE_1D` is)
                let mut old_start = 0;
                let mut start = if self.config.remove_type_prefix {3} else {0};

                'na: for n in &name_parts {
                    if vn[start..].starts_with(n) {
                        old_start = start;
                        start += n.len() + 1;
                    } else {break 'na}
                }

                let mut end = vn.len();
                'ne: for n in name_parts.iter().rev() {
                    if vn[..end].ends_with(n) {
                        end -= n.len();
                    } else if vn[..end].ends_with('_') {
                        end -= 1;
                    } else {break 'ne}
                }

                if !vn[start..end].chars().next().unwrap().is_digit(10) {
                    variant.set_name(&vn[start..end]);
                } else {variant.set_name(&vn[old_start..])}
            }
            VariantPaddingConfig::RemovePrefix => {
                let vn = unsafe{ &*variant.name() };
                variant.set_name(&vn[3..]);
            }
            VariantPaddingConfig::Keep => {}
        }

        if self.config.camel_case_variants {
            unsafe{
                let vn = &*variant.name();
                let vn_new = self.append_char_func(
                    |s| {
                        let mut is_uppercase = true;
                        for c in vn.chars() {
                            if c == '_' {
                                is_uppercase = true;
                            } else if is_uppercase {
                                s.push(c);
                                is_uppercase = false;
                            } else {
                                s.push(c.to_lowercase().next().unwrap())
                            }
                        }
                    }
                );
                variant.set_name(vn_new);
            }
        }
    }

    fn add_command(&mut self, mut command: VkCommand) {
        for typ in command.params.iter_mut().map(|p| &mut p.typ).chain(Some(&mut command.ret).into_iter()) {
            if let Some(mut type_ptr) = typ.type_ptr() {
                type_ptr = self.process_type_ident(type_ptr);
                typ.set_type(type_ptr);
            }
        }

        command.name = self.process_command_ident(command.name);

        self.commands.push(command);
    }

    fn process_command_ident(&mut self, ident: *const str) -> *const str {
        let mut ident = unsafe{ &*ident };

        if self.config.remove_command_prefix {
            if let Some(0) = ident.find("vk") {
                ident = unsafe{&*self.append_char_func(
                    |s| {
                        let mut chars = ident.chars().skip(2);
                        for c in chars.next().unwrap().to_lowercase().chain(chars) {
                            s.push(c);
                        }
                    }
                )};
            }
        }

        if self.config.snake_case_commands {
            ident = unsafe{&*self.append_char_func(
                |s| {
                    let mut last_uppercase = false;
                    for c in ident.chars() {
                        if c.is_uppercase() {
                            if !last_uppercase {s.push('_')}
                            last_uppercase = true;
                        } else {last_uppercase = false}

                        s.push(c.to_lowercase().next().unwrap());
                    }
                }
            )};
        }

        ident
    }

    unsafe fn append_char_func<F: Fn(&mut String)>(&mut self, processor: F) -> *const str {
        use std::{slice, str};

        let prepushcap = self.string_buffer.capacity();
        let prepushlen = self.string_buffer.len();
        // We want to have all of the string in one block of memory in order to save heap allocation time.
        processor(&mut self.string_buffer);

        if prepushcap != self.string_buffer.capacity() {
            panic!("Allocation detected in string buffer")
        }

        let ptr = self.string_buffer.as_ptr().offset(prepushlen as isize);
        str::from_utf8_unchecked(slice::from_raw_parts(ptr, self.string_buffer.len()-prepushlen)) as *const str
    }
}


macro_rules! gen_func_param {
    ($write: expr, $el: expr) => {
        match *$el {
            VkElType::Var(ident) => write!($write, "{}", &*ident),
            VkElType::ConstPtr(ident, count) => {
                for _ in 0..count {
                    write!($write, "*const ").unwrap();
                }
                write!($write, "{}", &*ident)
            }
            VkElType::MutPtr(ident, count) => {
                for _ in 0..count {
                    write!($write, "*mut ").unwrap();
                }
                write!($write, "{}", &*ident)
            }
            VkElType::ConstArray(ident, size) => write!($write, "*const [{}; {}]", &*ident, size),
            VkElType::MutArray(ident, size)   => write!($write, "*mut [{}; {}]", &*ident, size),
            VkElType::ConstArrayEnum(ident, size) => write!($write, "*const [{}; {}]", &*ident, &*size),
            VkElType::MutArrayEnum(ident, size)   => write!($write, "*mut [{}; {}]", &*ident, &*size),
            VkElType::Void     => write!($write, "()"),
            VkElType::Const(_) => panic!("Unexpected raw const"),
            VkElType::Unknown  => panic!("Unexpected Unknown")
        }.unwrap()
    }
}

pub struct GenTypes<'a> {
    config:       &'a GenConfig<'a>,
    structs:      String,
    unions:       String,
    enums:        String,
    bitmasks:     String,
    handles:      String,
    typedefs:     String,
    funcpointers: String,
    consts:       String,
    externs:      String,
    libc_reexports: String
}

impl<'a> GenTypes<'a> {
    /// Generate type definitions. Assumes presence of `./prelude_common.rs`.
    pub fn new(processed: &'a GenPreproc) -> Self {
        let mut gen_types = GenTypes {
            config:       &processed.config,
            structs:      String::with_capacity(2usize.pow(17)),
            unions:       String::with_capacity(2usize.pow(13)),//take that you superstitious bastards
            enums:        String::with_capacity(2usize.pow(15)),
            bitmasks:     String::with_capacity(2usize.pow(15)),
            handles:      String::with_capacity(2usize.pow(12)),
            typedefs:     String::with_capacity(2usize.pow(13)),
            funcpointers: String::with_capacity(2usize.pow(11)),
            consts:       String::with_capacity(2usize.pow(10)),
            externs:      String::with_capacity(2usize.pow(10)),
            libc_reexports: String::new()
        };

        // Iterate over the types in an order defined by which types were loaded first
        for t in processed.type_ord.iter().map(|k| processed.types.get(k).unwrap()) {
            use registry::VkType::*;
            use registry::VkElType::*;
            use registry::VkVariant::*;

            match *t {
                // Generate struct bindings
                Struct{name, ref fields} => {
                    let name = unsafe{ &*name };
                    let structs = &mut gen_types.structs;

                    if !processed.custom_impls.contains(name) {
                        writeln!(structs, "#[derive(Debug, Clone)]").unwrap();
                    }
                    writeln!(structs, "#[repr(C)]\npub struct {} {{", name).unwrap();
                    let type_wrappable_by_option = |type_ident: &str| {
                        match processed.registry.types().get(type_ident) {
                            Some(&VkType::FuncPointer{..}) => true,
                            _ => false
                        }
                    };

                    for f in fields { unsafe {
                        write!(structs, "    pub ").unwrap();
                        match f.field_type {
                            Var(ident) => {
                                let wrap_with_option = f.optional & type_wrappable_by_option(&*ident);
                                match wrap_with_option {
                                    true => writeln!(structs, "{}: Option<{}>,", &*f.field_name, &*ident),
                                    false => writeln!(structs, "{}: {},", &*f.field_name, &*ident)
                                }
                            },
                            ConstPtr(ident, count) => {
                                write!(structs, "{}: ", &*f.field_name).unwrap();
                                for _ in 0..count {
                                    write!(structs, "*const ").unwrap();
                                }
                                writeln!(structs, "{},", &*ident)
                            }
                            MutPtr(ident, count) => {
                                write!(structs, "{}: ", &*f.field_name).unwrap();
                                for _ in 0..count {
                                    write!(structs, "*mut ").unwrap();
                                }
                                writeln!(structs, "{},", &*ident)
                            }
                            MutArray(ident, count) => {
                                writeln!(structs, "{}: [{}; {}],", &*f.field_name, &*ident, count)
                            }
                            MutArrayEnum(ident, cons) => {
                                writeln!(structs, "{}: [{}; {}],", &*f.field_name, &*ident, &*cons)
                            }
                            ConstArray(_, _)      |
                            ConstArrayEnum(_, _) => panic!("Unexpected const array in struct"),
                            Const(_)             => panic!("Unexpected const {}", name),
                            Void                 => panic!("Unexpected void"),
                            Unknown              => panic!("Unexpected unknown")
                        }.unwrap();
                    }}
                    structs.push_str("}\n\n");

                    if processed.custom_impls.contains(name) {
                        // Write `Clone` implementation
                        writeln!(structs, include_str!("custom_impl_clone.rs"), name).unwrap();
                        for f in fields {unsafe{
                            write!(structs, "            ").unwrap();
                            let n = &*f.field_name;

                            // For some reason, extern "system" functions implement `Copy` and not `Clone`. Because of that, we have
                            // to make an exception in the custom Clone implementation so that these functions are *copied*, not
                            // cloned. Also, we can used the "processed" type because function pointer types don't get processed
                            if let Some(&FuncPointer{..}) = processed.registry.types().get(&*f.field_type.type_ptr().unwrap()) {
                                writeln!(structs, "{0}: self.{0},", n).unwrap();
                            } else {
                                match f.field_type {
                                    MutArray(_, s)        |
                                    ConstArray(_, s)     => writeln!(structs, include_str!("clone_array.rs"), n, s),
                                    MutArrayEnum(_, s)    |
                                    ConstArrayEnum(_, s) => writeln!(structs, include_str!("clone_array.rs"), n, &*s),
                                    _                    => writeln!(structs, "{0}: self.{0}.clone(),", n)
                                }.unwrap()
                            }
                        }}
                        write!(structs, "        }}\n    }}\n}}\n\n").unwrap();

                        // Write `Debug` implementation
                        writeln!(structs, include_str!("custom_impl_debug.rs"), name).unwrap();
                        for f in fields {unsafe{
                            write!(structs, "           ").unwrap();
                            let n = &* f.field_name;

                            if let Some(&FuncPointer{..}) = processed.registry.types().get(&*f.field_type.type_ptr().unwrap()) {
                                match f.optional {
                                    true => writeln!(structs, ".field(\"{0}\", &(self.{0}.map(|f| f as *const ())))", n),
                                    false => writeln!(structs, ".field(\"{0}\", &(self.{0} as *const ()))", n)
                                }.unwrap()
                            } else {
                                match f.field_type {
                                    MutArray(t, _)        |
                                    ConstArray(t, _)      |
                                    MutArrayEnum(t, _)    |
                                    ConstArrayEnum(t, _) =>
                                        if gen_types.config.debug_c_strings && "c_char" == &*t {
                                            writeln!(structs, ".field(\"{0}\", &unsafe{{ CStr::from_ptr(&self.{0}[0]) }})", n)
                                        } else {writeln!(structs, ".field(\"{0}\", &&self.{0}[..])", n)},
                                    _                    => writeln!(structs, ".field(\"{0}\", &self.{0})", n)
                                }.unwrap()
                            }
                        }}
                        write!(structs, "            .finish()\n    }}\n}}\n\n").unwrap();
                    }
                }

                // UPDATE 9/5/16: Unions finally have an unstable implementation! Proper unions can now be
                // auto-generated easily, albeit only on the nightly channel. Proper union generation will not be
                // enabled by default until they become stable. Until then, they will be controlled by the
                // use_native_unions flag in `GenConfig`.
                //
                // Original Comment:
                // Unions are currently a pain in the ass to do, as Rust does not have a stable implementation.     |
                // What they do have, however, is an approved RFC that is currently being implemented. Until those  |
                // become reality the unions currently present in Vulkan are simply going to be hard-coded into the |
                // generator with a fairly shitty, although functional, implementation.                             |
                Union{name, ref variants} => unsafe {
                    let unions = &mut gen_types.unions;
                    if processed.config.use_native_unions {
                        // Create base union type
                        writeln!(unions, "#[repr(C)]").unwrap();
                        writeln!(unions, "pub union {} {{", &*name).unwrap();

                        for v in variants {
                            write!(unions, "    pub {}: ", &*v.field_name).unwrap();

                            match v.field_type {
                                Var(ident) => writeln!(unions, "{},", &*ident),
                                ConstPtr(ident, count) => {
                                    for _ in 0..count {
                                        write!(unions, "*const ").unwrap();
                                    }
                                    writeln!(unions, "{},", &*ident)
                                }
                                MutPtr(ident, count) => {
                                    for _ in 0..count {
                                        write!(unions, "*mut ").unwrap();
                                    }
                                    writeln!(unions, "{},", &*ident)
                                }
                                MutArray(ident, count)    => writeln!(unions, "[{}; {}],", &*ident, count),
                                MutArrayEnum(ident, cons) => writeln!(unions, "[{}; {}],", &*ident, &*cons),

                                ConstArray(_, _)      |
                                ConstArrayEnum(_, _) => panic!("Unexpected const array in union"),
                                Const(_)             => panic!("Unexpected const {}", &*name),
                                Void                 => panic!("Unexpected void"),
                                Unknown              => panic!("Unexpected unknown")
                            }.unwrap();
                        }
                        writeln!(unions, "}}\n").unwrap();

                        // Unions derives don't work at the time of this writing, so we have to manually implement
                        // it manually. Luckily, the manual implementation is trivial.
                        writeln!(unions, include_str!("./union_cloned.rs"), &*name).unwrap();

                        // Write `Debug` implementation
                        writeln!(unions, include_str!("./custom_impl_debug.rs"), &*name).unwrap();
                        for v in variants {
                            write!(unions, "           ").unwrap();
                            let n = &* v.field_name;

                            if let Some(&FuncPointer{..}) = processed.registry.types().get(&*v.field_type.type_ptr().unwrap()) {
                                writeln!(unions, ".field(\"{0}\", &(self.{0} as *const ()))", n).unwrap();
                            } else {
                                match v.field_type {
                                    MutArray(t, _)        |
                                    ConstArray(t, _)      |
                                    MutArrayEnum(t, _)    |
                                    ConstArrayEnum(t, _) =>
                                        if gen_types.config.debug_c_strings && "c_char" == &*t {
                                            writeln!(unions, ".field(\"{0}\", &unsafe{{ CStr::from_ptr(&self.{0}[0]) }})", n)
                                        } else {writeln!(unions, ".field(\"{0}\", unsafe{{ &&self.{0}[..] }})", n)},
                                    _                    => writeln!(unions, ".field(\"{0}\", unsafe{{ &self.{0} }})", n)
                                }.unwrap()
                            }
                        }
                        write!(unions, "            .finish()\n    }}\n}}\n\n").unwrap();

                    } else {
                        if (&*name).contains("ClearColorValue") {
                            writeln!(unions, include_str!("./hardcoded/union_ClearColorValue.rs"), &*name)
                        } else if (&*name).contains("ClearValue") {
                            writeln!(unions, include_str!("./hardcoded/union_ClearValue.rs"), &*name,
                                                                                              &*variants[0].field_type.type_ptr().unwrap(),
                                                                                              &*variants[1].field_type.type_ptr().unwrap())
                        } else {panic!("Unexpected Union")}.unwrap()
                    }
                },

                // Generate enum bindings
                Enum{name, ref variants} => {
                    let enums = &mut gen_types.enums;
                    let ename = unsafe{ &*name };
                    if gen_types.config.use_native_enums {
                        writeln!(enums, "#[repr(C)]\n#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]\npub enum {} {{", ename).unwrap();

                        for v in variants {unsafe {
                            match *v {
                                Value{name, value} => writeln!(enums, "    {} = {},", &*name, value),
                                Bitpos{..}         => panic!("Found bitpos in non-biflags enum")
                            }.unwrap();
                        }}

                        enums.push_str("}\n\n");
                    } else {
                        writeln!(enums, "pub type {} = u32;", ename).unwrap();
                        for v in variants {
                            match *v {
                                Value{name, value} => {
                                    let vname = unsafe{ &*name };
                                    let (sign, v) = if value >= 0 {
                                        ("", value)
                                    } else {
                                        ("!", -value)
                                    };
                                    writeln!(enums, "pub const {}: u32 = {}{};", vname, sign, v).unwrap();
                                },
                                Bitpos{..} => panic!("Found bitpos in non-biflags enum")
                            }
                        }
                    }
                }

                // Generate bitmasks
                Bitmask{name, ref variants} => {
                    let bitmasks = &mut gen_types.bitmasks;
                    let name = unsafe{ &*name };

                    let flags_name = unsafe{ &*processed.types.get("VkFlags").unwrap().name().unwrap() };

                    if gen_types.config.wrap_bitmasks {
                        let mut all_bits = 0;
                        for v in variants {unsafe {
                            let bits =
                                match *v {
                                    Value{value, ..}   => value,
                                    Bitpos{bitpos, ..} => 2isize.pow(bitpos)
                                };
                            writeln!(bitmasks, "pub const {0}: {1} = {1} {{flags: 0b{2:b}}};", &*v.name(), name, bits).unwrap();
                            all_bits |= bits;
                        }}

                        writeln!(bitmasks, "vk_bitflags_wrapped!({}, 0b{:b}, {});\n", name, all_bits, flags_name).unwrap();
                    } else {
                        writeln!(bitmasks, "pub type {} = {};", name, flags_name).unwrap();

                        for v in variants {unsafe {
                            let bits =
                                match *v {
                                    Value{value, ..}   => value,
                                    Bitpos{bitpos, ..} => 2isize.pow(bitpos)
                                };
                            writeln!(bitmasks, "pub const {0}: {1} = 0b{2:b};", &*v.name(), name, bits).unwrap();
                        }}
                        bitmasks.push('\n');
                    }
                }

                // Generate handles
                Handle{name, dispatchable, ..} => {
                    let handles = &mut gen_types.handles;
                    let name = unsafe{ &*name };
                    if dispatchable {
                        writeln!(handles, include_str!("handle_dispatchable.rs"), name).unwrap();
                    } else {
                        writeln!(handles, "handle_nondispatchable!({});", name).unwrap();
                    }
                }

                // Generate typedefs
                TypeDef{name, typ, requires, ..} => {
                    let (name, typ, requires) = unsafe{ (&*name, &*typ, to_option(requires)) };
                    // If we're typedef-ing a flag and the flag bit types aren't defined, generate the raw typedef. Otherwise,
                    // the typedef is handled in the `Bitmask` section of this generator.
                    if (typ != "Flags" && typ != "VkFlags") || requires == None {
                        writeln!(gen_types.typedefs, "pub type {} = {};", name, typ).unwrap();
                    }
                }

                // Generate API constants, inferring the type.
                ApiConst{name, value} => {
                    use self::ConstType::*;

                    let consts = &mut gen_types.consts;
                    let (name, value) = unsafe{ (&*name, (&*value).trim()) };

                    let mut typ = processed.const_types.get(name).map(|t| *t).unwrap_or(Unknown);
                    let mut slice_indices = (0, value.len());

                    // Ignore enum variants that have been renamed in the Vulkan specs.
                    if typ == Unknown && value.starts_with("VK_") {
                        continue;
                    }

                    for (b, c) in value.char_indices() {
                        match c {
                            ')'                    => slice_indices.1 = b,
                            '(' if typ != Str      => slice_indices.0 = b+1,
                            'U' if typ == Unsigned => (),
                            'U' if typ != Str      => panic!("Unexpected U in {:?} {}; {}", typ, name, c),
                            'L' if typ == Unsigned => typ = ULong,
                            'L' if typ == ULong    => typ = ULongLong,
                            '"'                    => typ = Str,
                            '.' if typ == Unsigned => typ = Float,
                            'f' if typ == Float    => (),
                            _   if
                                 typ == Unknown &&
                                 c.is_digit(10)    => typ = Unsigned,
                            '-' if
                                 (typ == Unsigned  ||
                                  typ == ULongLong ||
                                  typ == USize     ||
                                  typ == Float) => (),
                            _   if
                                 (typ == Unsigned  ||
                                  typ == ULongLong ||
                                  typ == USize     ||
                                  typ == Float)    &&
                                 !c.is_digit(10)   => panic!("Unexpected non-digit in {:?} {}; {}", typ, name, c),
                            _ => ()
                        }
                    }

                    let sliced_value = &value[slice_indices.0..slice_indices.1];
                    match typ {
                        Unsigned   => write!(consts, "pub const {}: uint32_t = ", name),
                        ULong      => write!(consts, "pub const {}: c_ulong = ", name),
                        ULongLong  => write!(consts, "pub const {}: c_ulonglong =", name),
                        USize      => write!(consts, "pub const {}: size_t = ", name),
                        Float      => writeln!(consts, "pub const {}: c_float = ", name),
                        Str        => writeln!(consts, "pub const {}: &'static str = {};", name, sliced_value),
                        Unknown    => panic!("Unknown const type")
                    }.unwrap();

                    match typ {
                        Unsigned   |
                        ULong      |
                        ULongLong  |
                        USize      |
                        Float     => {
                            let num_expr = sliced_value.chars().filter(|c| !c.is_alphabetic()).collect::<String>();
                            if '~' == num_expr.chars().next().unwrap() {
                                writeln!(consts, "!{};", &num_expr[1..])
                            } else {writeln!(consts, "{};", num_expr)}.unwrap();
                        }
                        _ => ()
                    }
                }

                // Generate external types
                ExternType{name, requires} => {
                    let externs = &mut gen_types.externs;
                    let (name, requires) = unsafe{ (&*name, &*requires) };
                    if let Some(over) = gen_types.config.extern_type_overrides.iter().find(|o| o.0 == name) {
                        writeln!(externs, "pub type {} = ::{};", name, over.1).unwrap();
                    } else if "vk_platform" == requires {
                        if gen_types.config.use_libc_types {
                            writeln!(&mut gen_types.libc_reexports, "pub use libc::{};", name).unwrap();
                        } else {
                            let typ =
                                match name {
                                    "c_void" => "()",
                                    // Following taken from libc
                                    "int8_t" => "i8",
                                    "int16_t" => "i16",
                                    "int32_t" => "i32",
                                    "int64_t" => "i64",
                                    "uint8_t" => "u8",
                                    "uint16_t" => "u16",
                                    "uint32_t" => "u32",
                                    "uint64_t" => "u64",

                                    "c_schar" => "i8",
                                    "c_uchar" => "u8",
                                    "c_short" => "i16",
                                    "c_ushort" => "u16",
                                    "c_int" => "i32",
                                    "c_uint" => "u32",
                                    "c_float" => "f32",
                                    "c_double" => "f64",
                                    "c_longlong" => "i64",
                                    "c_ulonglong" => "u64",
                                    "intmax_t" => "i64",
                                    "uintmax_t" => "u64",

                                    "size_t" => "usize",
                                    "ptrdiff_t" => "isize",
                                    "intptr_t" => "isize",
                                    "uintptr_t" => "usize",
                                    "ssize_t" => "isize",

                                    #[cfg(all(not(target = "aarch64-unknown-linux-gnu"), not(target = "arm-unknown-linux-gnueabihf"), not(target = "arm-linux-androideabi")))]
                                    "c_char" => "i8",
                                    #[cfg(all(target = "aarch64-unknown-linux-gnu", target = "arm-unknown-linux-gnueabihf", target = "arm-linux-androideabi"))]
                                    "c_char" => "u8",
                                    _ => panic!("Unexpected C type")
                                };
                            writeln!(externs, "pub type {} = {};", name, typ).unwrap();
                        }
                    } else {
                        writeln!(externs, "pub type {} = *const ();", name).unwrap();
                    }
                }

                // Generate function pointers.
                FuncPointer{name, ref ret, ref params} => {
                    let funcpointers = &mut gen_types.funcpointers;
                    writeln!(funcpointers, "pub type {} = unsafe extern \"system\" fn(", unsafe{ &*name }).unwrap();
                    for p in params.iter() {unsafe{
                        write!(funcpointers, "    ").unwrap();
                        gen_func_param!(funcpointers, p);
                        funcpointers.push_str(",\n");
                    }}

                    if Void == *ret {
                        writeln!(funcpointers, ");\n").unwrap();
                    } else {
                        write!(funcpointers, ") -> ").unwrap();
                        match *ret {
                            ConstPtr(ident, count) => {
                                for _ in 0..count {
                                    write!(funcpointers, "*const ").unwrap();
                                }
                                writeln!(funcpointers, "{};\n", unsafe{ &*ident })
                            }
                            MutPtr(ident, count) => {
                                for _ in 0..count {
                                    write!(funcpointers, "*mut ").unwrap();
                                }
                                writeln!(funcpointers, "{};\n", unsafe{ &*ident })
                            }
                            _ => writeln!(funcpointers, "{};\n", unsafe{ &*ret.type_ptr().unwrap() })
                        }.unwrap()
                    }
                }
                _ => ()
            }
        }

        gen_types
    }

    pub fn write_types<W: Write>(&self, write: &mut W) {
        writeln!(write, "{}", include_str!("defines.rs")).unwrap();

        writeln!(write, "mod libc_reexports {{").unwrap();
        writeln!(write, "{}", &self.libc_reexports).unwrap();
        writeln!(write, "}}").unwrap();

        writeln!(write, "pub mod types {{").unwrap();
        writeln!(write, "#![allow(non_camel_case_types, dead_code)]").unwrap();
        if !self.config.snake_case_members {
            writeln!(write, "#![allow(non_snake_case)]").unwrap();
        }
        if self.config.wrap_bitmasks {
            writeln!(write, "use std::ops::*;").unwrap();
        }

        writeln!(write, "use std::fmt; use std::ffi::CStr; use super::*; #[allow(unused_imports)]use super::libc_reexports::*;").unwrap();
        writeln!(write, "{}", &self.externs).unwrap();
        writeln!(write, "{}", &self.typedefs).unwrap();
        writeln!(write, "{}", &self.consts).unwrap();
        writeln!(write, "{}", &self.structs).unwrap();
        writeln!(write, "{}", &self.unions).unwrap();
        writeln!(write, "{}", &self.enums).unwrap();
        writeln!(write, "{}", &self.handles).unwrap();
        writeln!(write, "{}", &self.bitmasks).unwrap();
        writeln!(write, "{}", &self.funcpointers).unwrap();
        writeln!(write, "}}").unwrap();
    }
}

/// What type a constant is inferred to be
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ConstType {
    /// A floating-point number
    Float,
    /// An unsigned integer
    Unsigned,
    ULong,
    ULongLong,
    USize,
    /// A string
    Str,
    Unknown
}

impl<'a> VkRegistry<'a> {
    /// Write global bindings for Vulkan API [`version`] \(1.0, 1.1, etc.) to the file [`write`] with
    /// the specified `extensions` and [`config`]
    /// # Examples
    ///
    /// ```no_run
    /// # // external crate declarations seems to break the code, so this is a workaround that probably won't ever be replaced.
    /// # mod vk_api {
    /// #     pub const VK_XML: &'static [u8] = &[0];
    /// # }
    /// #
    /// # use vk_generator::{VkRegistry, GenConfig, VkVersion};
    /// # use std::env;
    /// # use std::fs::File;
    /// # use std::path::Path;
    /// #
    /// let out = env::var("OUT_DIR").unwrap();
    /// let mut file = File::create(&Path::new(&out).join("vk.rs")).unwrap();
    /// VkRegistry::new(vk_api::VK_XML).gen_global(
    ///     &mut file,
    ///     VkVersion(1, 0),
    ///     &[],
    ///     GenConfig::new()
    /// );
    /// ```
    ///
    /// [`config`]: ./struct.GenConfig.html
    /// [`version`]: ./struct.VkVersion.html
    /// [`write`]: https://doc.rust-lang.org/stable/std/io/trait.Write.html
    pub fn gen_global<W: Write>(&self, write: &mut W, version: VkVersion, extensions: &[&str], config: GenConfig) {
        let preproc = GenPreproc::new(self, version, extensions, config);

        writeln!(write, "{}", include_str!("prelude_common.rs")).unwrap();
        writeln!(write, "{}", include_str!("prelude_global_gen.rs")).unwrap();
        let gen_types = GenTypes::new(&preproc);
        gen_types.write_types(write);

        writeln!(write, "pub mod cmds {{").unwrap();
        writeln!(write, "#![allow(dead_code)]").unwrap();
        if !preproc.config.snake_case_commands || !preproc.config.snake_case_members {
            writeln!(write, "#![allow(non_snake_case)]").unwrap();
        }
        writeln!(write, "use super::*; #[allow(unused_imports)] use super::libc_reexports::*;").unwrap();

        writeln!(write, "vk_functions!{{").unwrap();
        for (c, r) in preproc.commands.iter().zip(preproc.commands_raw.iter()) {unsafe{
            writeln!(write, "    \"{}\", {}(", r, &*c.name).unwrap();
            for p in c.params.iter() {
                write!(write, "        {}: ", &*p.name).unwrap();
                gen_func_param!(write, &p.typ);
                writeln!(write, ",").unwrap();
            }
            write!(write, "    ) -> ").unwrap();
            gen_func_param!(write, &c.ret);
            writeln!(write, ";\n").unwrap();
        }}
        writeln!(write, "}}}}").unwrap();
    }

    /// Write struct bindings for Vulkan API [`version`] \(1.0, 1.1, etc.) to the file [`write`] with
    /// the specified `extensions` and [`config`]
    /// # Examples
    ///
    /// ```no_run
    /// # // Ditto.
    /// # mod vk_api {
    /// #     pub const VK_XML: &'static [u8] = &[0];
    /// # }
    /// #
    /// # use vk_generator::{VkRegistry, GenConfig, VkVersion};
    /// # use std::env;
    /// # use std::fs::File;
    /// # use std::path::Path;
    /// #
    /// let out = env::var("OUT_DIR").unwrap();
    /// let mut file = File::create(&Path::new(&out).join("vk.rs")).unwrap();
    /// VkRegistry::new(vk_api::VK_XML).gen_struct(
    ///     &mut file,
    ///     VkVersion(1, 0),
    ///     &[],
    ///     GenConfig::new()
    /// );
    /// ```
    ///
    /// [`config`]: ./struct.GenConfig.html
    /// [`version`]: ./struct.VkVersion.html
    /// [`write`]: https://doc.rust-lang.org/stable/std/io/trait.Write.html
    pub fn gen_struct<W: Write>(&self, write: &mut W, version: VkVersion, extensions: &[&str], config: GenConfig) {
        let preproc = GenPreproc::new(self, version, extensions, config);

        writeln!(write, "{}", include_str!("prelude_common.rs")).unwrap();
        writeln!(write, "{}", include_str!("prelude_struct_gen.rs")).unwrap();
        let gen_types = GenTypes::new(&preproc);
        gen_types.write_types(write);

        writeln!(write, "pub mod cmds {{").unwrap();
        writeln!(write, "#![allow(dead_code)]").unwrap();
        writeln!(write, "#![allow(non_camel_case_types)]").unwrap();
        if !preproc.config.snake_case_members || !preproc.config.snake_case_commands {
            writeln!(write, "#![allow(non_snake_case)]").unwrap();
        }
        writeln!(write, "use super::*; #[allow(unused_imports)] use super::libc_reexports::*;").unwrap();

        writeln!(write, "vk_struct_bindings!{{").unwrap();
        for (c, r) in preproc.commands.iter().zip(preproc.commands_raw.iter()) {unsafe{
            writeln!(write, "    \"{}\", {}(", r, &*c.name).unwrap();
            for p in c.params.iter() {
                write!(write, "        {}: ", &*p.name).unwrap();
                gen_func_param!(write, &p.typ);
                writeln!(write, ",").unwrap();
            }
            write!(write, "    ) -> ").unwrap();
            gen_func_param!(write, &c.ret);
            writeln!(write, ";\n").unwrap();
        }}
        writeln!(write, "}}}}").unwrap();
    }
}

pub trait GenRegistry {
    fn features(&self)    -> &HashMap<VkVersion, VkFeature>;
    fn types(&self)       -> &HashMap<&str, VkType>;
    fn commands(&self)    -> &HashMap<&str, VkCommand>;
    fn extns(&self)       -> &HashMap<&str, VkExtn>;
    fn buffer_cap(&self)  -> usize;
    fn core_consts(&self) -> &Vec<&str>;
}

struct PeekNext<I: Iterator> {
    iter: I,
    peeked: Option<I::Item>
}

impl<I: Iterator> Iterator for PeekNext<I> where I::Item: Copy {
    type Item = (I::Item, I::Item);
    fn next(&mut self) -> Option<Self::Item> {
        let cur =
            match self.peeked {
                Some(_) => self.peeked.take(),
                None    => self.iter.next()
            };

        if let Some(cur) = cur {
            self.peeked = self.iter.next();
            match self.peeked {
                Some(next) => Some((cur, next)),
                None       => None
            }
        } else {None}
    }
}

trait PeekNextCreate where Self: Sized + Iterator {
    fn peek_next(self) -> PeekNext<Self>;
}

impl<I: Iterator> PeekNextCreate for I {
    fn peek_next(self) -> PeekNext<I> {
        PeekNext {
            iter: self,
            peeked: None
        }
    }
}
