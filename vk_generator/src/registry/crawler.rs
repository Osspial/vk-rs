//! Module that contains a crawler function that crawls through the vulkan xml and returns a
//! Registry struct
use xml::reader::{Events, XmlEvent};
use xml::name::OwnedName;
use xml::attribute::OwnedAttribute;
use std::io::Read;
use std::slice::Iter;
use std::num::ParseIntError;
use super::{VkRegistry, VkType, VkMember, VkVariant, VkCommand, VkParam, VkFeature, VkVersion, VkReqRem, VkExtn, VkElType};

pub fn crawl<R: Read>(xml_events: Events<R>, registry: &mut VkRegistry) {
    use self::XmlElement::*;

    // Everything that we're going to put in the registry get stored in temporary buffers before it's   |
    // placed on the registry.                                                                          |
    let mut type_buffer = VkType::Unhandled;
    let mut command_buffer: Option<VkCommand> = None;
    let mut feature_buffer: Option<VkFeature> = None;
    let mut interface_reqrem = VkReqRem::None;
    let mut extn_buffer: Option<VkExtn> = None;
    let mut cur_block = VkBlock::None;
    // A variable that contains what the index of the element in vk_elements that vk_elements was
    // popped to. Used to prevent the element iterator from going over elements that have already
    // been checked.
    let mut popped_to = 0;

    // A stack of the relevant elements in the vulkan xml
    let mut vk_elements = Vec::with_capacity(10);

    for event in xml_events {
        let event = event.unwrap();
        match event {
            XmlEvent::StartElement{ name, attributes, .. } => {
                vk_elements.push(XmlElement::new_tag(name, attributes));
            }

            XmlEvent::EndElement{..} => {
                // We don't have to go through the entire element stack - only the elements that have changed since
                // the last access. Also, we only have to process the stack when an element is about to be removed.
                for el in &vk_elements[popped_to..] {
                    match *el {
                        // This branch handles the various tags present in the vulkan xml file. Because each tag is only    |
                        // seen once, when it's pushed onto vk_elements, we don't have to have any checking inside to make  |
                        // sure we aren't looking at a duplicate tag.                                                       |
                        Tag{name: ref tag_name, attributes: ref tag_attrs} =>
                            match &tag_name[..] {
                                // Handle enum registration
                                "enums"      =>
                                    if let Some(name) = find_attribute(tag_attrs, "name") {
                                        cur_block = VkBlock::Enums;
                                        registry.push_type(type_buffer).ok();
                                        if "API Constants" == name {
                                            type_buffer = VkType::new_enum("API Constants");
                                        } else {
                                            match find_attribute(tag_attrs, "type") {
                                                Some("enum")    => type_buffer = VkType::new_enum(registry.append_str(name)),
                                                Some("bitmask") => type_buffer = VkType::new_bitmask(registry.append_str(name)),
                                                t               => panic!(format!("Unexpected enum type {:?} {}", t, name))
                                            }
                                        }
                                    } else {panic!("Could not find enum name")},
                                "enum"
                                    if VkBlock::Enums == cur_block =>
                                    if let Some(name) = find_attribute(tag_attrs, "name") {
                                        match type_buffer {
                                            VkType::Enum{name: enum_name, ref mut variants} |
                                            VkType::Bitmask{name: enum_name, ref mut variants} => {
                                                let name = registry.append_str(name);

                                                if "API Constants" == unsafe{ &*enum_name } {
                                                    if let Some(value) = find_attribute(tag_attrs, "value") {
                                                        let value = registry.append_str(value);
                                                        registry.push_type(VkType::new_const(name, value)).ok();
                                                    } else {panic!("Could not find value in API Constant")}
                                                } else {
                                                    variants.push(
                                                        if let Some(value) = find_attribute(tag_attrs, "value") {
                                                            VkVariant::new_value(name, to_number(value).unwrap())
                                                        } else if let Some(bitpos) = find_attribute(tag_attrs, "bitpos") {
                                                            VkVariant::new_bitpos(name, to_number(bitpos).unwrap() as u32)
                                                        } else {panic!("Could not find value or bitpos in enum")}
                                                    );
                                                }
                                            }
                                            _ => ()
                                        }
                                    } else {panic!("Could not find enum variant name")},


                                // Handle type (struct, union, typedef, etc.) registration
                                "types"      => cur_block = VkBlock::Types,
                                "type"
                                    if VkBlock::Types == cur_block =>
                                    if let Some(category) = find_attribute(tag_attrs, "category") {
                                        registry.push_type(type_buffer).ok();
                                        match category {
                                            "basetype"       |
                                            "bitmask"       => type_buffer = VkType::new_typedef(find_attribute(tag_attrs, "requires").map(|r| registry.append_str(r))),
                                            "define"        =>
                                                if let Some(name) = find_attribute(tag_attrs, "name") {
                                                    type_buffer = VkType::new_define(registry.append_str(name));
                                                } else {type_buffer = VkType::empty_define()},
                                            "enum"          => type_buffer = VkType::Unhandled,
                                            "funcpointer"   => type_buffer = VkType::empty_funcpointer(),
                                            "group"         => type_buffer = VkType::Unhandled,
                                            "handle"        => type_buffer = VkType::empty_handle(),
                                            "include"       => type_buffer = VkType::Unhandled,
                                            "struct"        =>
                                                type_buffer = VkType::new_struct(registry.append_str(find_attribute(tag_attrs, "name").unwrap())),
                                            "union"         =>
                                                type_buffer = VkType::new_union(registry.append_str(find_attribute(tag_attrs, "name").unwrap())),
                                            _               => panic!("Unexpected category")
                                        }
                                    } else if let Some(requires) = find_attribute(tag_attrs, "requires") {
                                        registry.push_type(type_buffer).ok();
                                        let requires = registry.append_str(requires);
                                        if let Some(name) = find_attribute(tag_attrs, "name") {
                                            let name = registry.append_str(name);
                                            type_buffer = VkType::new_extern(name, requires);
                                        } else {panic!("Expected external type name; found nothing")}
                                    },
                                // Members of structs and unions
                                "member"
                                    if VkBlock::Types == cur_block =>
                                    match type_buffer {
                                        VkType::Struct{fields: ref mut members, ..}   |
                                        VkType::Union{variants: ref mut members, ..} =>
                                            if let Some("true") = find_attribute(tag_attrs, "optional") {
                                                members.push(VkMember::empty(true))
                                            } else {members.push(VkMember::empty(false))},
                                        _                                            => panic!("Unexpected \"member\" tag found")
                                    },
                                "member"     => panic!("\"member\" tag found outside of \"types\" block"),


                                // Register commands. Most of the relevant code is in the `Characters` blocks
                                "commands"   => cur_block = VkBlock::Commands,
                                "command"
                                    if VkBlock::Commands == cur_block => {
                                        registry.push_command(command_buffer).ok();
                                        command_buffer = Some(VkCommand::empty());
                                    }
                                "param"
                                    if VkBlock::Commands == cur_block => command_buffer.as_mut().unwrap().params.push(VkParam::empty()),


                                // Register features. Features are the function/type declarations for the specific API version.
                                "feature"    =>
                                    if let Some(name) = find_attribute(tag_attrs, "name") {
                                        if let Some(version) = find_attribute(tag_attrs, "number") {
                                            cur_block = VkBlock::Feature;
                                            registry.push_feature(feature_buffer).ok();
                                            feature_buffer = Some(VkFeature::new(registry.append_str(name), VkVersion::from_str(version)));
                                        } else {panic!("Could not find feature number")}
                                    } else {panic!("Could not find feature name")},
                                "require"
                                    if VkBlock::Feature == cur_block =>
                                    interface_reqrem = VkReqRem::Require(find_attribute(tag_attrs, "profile").map(|s| registry.append_str(s))),
                                "remove"
                                    if VkBlock::Feature == cur_block =>
                                    interface_reqrem = VkReqRem::Remove(find_attribute(tag_attrs, "profile").map(|s| registry.append_str(s))),
                                "command"
                                    if VkBlock::Feature == cur_block =>
                                    if let Some(name) = find_attribute(tag_attrs, "name") {
                                        feature_buffer.as_mut().unwrap().push_command(registry.append_str(name), &interface_reqrem);
                                    } else {panic!("Could not find feature name")},
                                "enum"
                                    if VkBlock::Feature == cur_block =>
                                    if let Some(name) = find_attribute(tag_attrs, "name") {
                                        feature_buffer.as_mut().unwrap().push_const(registry.append_str(name), &interface_reqrem);
                                    } else {panic!("Could not find feature name")},
                                "type"
                                    if VkBlock::Feature == cur_block =>
                                    if let Some(name) = find_attribute(tag_attrs, "name") {
                                        feature_buffer.as_mut().unwrap().push_type(registry.append_str(name), &interface_reqrem);
                                    } else {panic!("Could not find feature name")},


                                // Register extensions.
                                "extensions" => cur_block = VkBlock::Extensions,
                                "extension"
                                    if VkBlock::Extensions == cur_block =>
                                    if let Some(name) = find_attribute(tag_attrs, "name") {
                                        if let Some(num) = find_attribute(tag_attrs, "number") {
                                            registry.push_extn(extn_buffer).ok();
                                            extn_buffer = Some(VkExtn::new(registry.append_str(name), isize::from_str_radix(num, 10).unwrap()))
                                        } else {panic!("Could not find extension number")}
                                    } else {panic!("Could not find extension name")},
                                "command"
                                    if VkBlock::Extensions == cur_block =>
                                    if let Some(name) = find_attribute(tag_attrs, "name") {
                                        extn_buffer.as_mut().unwrap().push_command(registry.append_str(name), &interface_reqrem);
                                    } else {panic!("Could not find extension command name")},
                                "type"
                                    if VkBlock::Extensions == cur_block =>
                                    if let Some(name) = find_attribute(tag_attrs, "name") {
                                        extn_buffer.as_mut().unwrap().push_type(registry.append_str(name), &interface_reqrem);
                                    } else {panic!("Could not find extension type name")},
                                "enum"
                                    if VkBlock::Extensions == cur_block =>
                                    if let Some(name) = find_attribute(tag_attrs, "name") {
                                        let name = registry.append_str(name);

                                        if let Some(extends) = find_attribute(tag_attrs, "extends") {
                                            let variant =
                                                if let Some(offset) = find_attribute(tag_attrs, "offset") {
                                                    // Determine enumerant value, as defined in the "Layers & Extensions" appendix of the spec
                                                    let offset = isize::from_str_radix(offset, 10).unwrap();
                                                    let extn_num = extn_buffer.as_ref().unwrap().num;
                                                    let mut value = BASE_VALUE + (extn_num - 1) * RANGE_SIZE + offset;

                                                    if let Some("-") = find_attribute(tag_attrs, "dir") {
                                                        value = -value;
                                                    }

                                                    VkVariant::new_value(name, value)
                                                } else if let Some(value) = find_attribute(tag_attrs, "value") {
                                                    let value = isize::from_str_radix(value, 10).unwrap();
                                                    VkVariant::new_value(name, value)
                                                } else if let Some(bitpos) = find_attribute(tag_attrs, "bitpos") {
                                                    let bitpos = u32::from_str_radix(bitpos, 10).unwrap();
                                                    VkVariant::new_bitpos(name, bitpos)
                                                } else {panic!("Invalid enum extension; missing \"offset\" or \"bitpos\"")};

                                            extn_buffer.as_mut().unwrap().push_enum(
                                                variant,
                                                Some(registry.append_str(extends)),
                                                &interface_reqrem);
                                        } else if let Some(value) = find_attribute(tag_attrs, "value") {
                                            extn_buffer.as_mut().unwrap().push_const(
                                                name, registry.append_str(value),
                                                &interface_reqrem);
                                        }
                                    } else {panic!("Could not find enum name")},
                                _ => ()
                            },

                        // In the registry, some of the tags contain characters which need to be processed. That happens    |
                        // here. Each Characters branch is specific to a certain block type, and what block we're in is     |
                        // stored in the `cur_block` variable.                                                              |
                        Characters{ref chars, tags: (tag, tag1)}
                            if VkBlock::Types == cur_block &&
                               "usage" != tag => {
                            let chars = &chars[..];
                            // Not every type contains the same tags; this match statement differentiates them so that they can |
                            // be processed in the manner that they require.
                            match type_buffer {
                                VkType::Struct{fields: ref mut members, ..} |
                                VkType::Union{variants: ref mut members, ..} => {
                                    if let Some(member) = members.last_mut() {
                                        match tag {
                                            "member" => process_type(chars, &mut member.field_type),
                                            "type"   => member.field_type.set_type(registry.append_str(chars)),
                                            "name"   =>
                                                // This exists as an `if let` block and isn't just `member.set_name(registry.append_str(chars))`    |
                                                // because someone decided to have two fields in the entire vulkan xml contain the array size       |
                                                // *inside* of the <name> tag. This is terribly inconvenient, and now I have to document exactly    |
                                                // what is means and why it exists.                                                                 |
                                                //                                                                                                  |
                                                // `size` is just the size of the the array. `name_len` is how long the name (without [\d+]) is.    |
                                                // For example, if the contents of <name> are "srcOffsets[2]", `name_len` is equal to 10 because    |
                                                // "srcOffsets" is 10 characters long.                                                              |
                                                if let Some((size, name_len)) = parse_array_index(chars) {
                                                    member.field_type.make_array(size);
                                                    member.set_name(registry.append_str(&chars[..name_len]));
                                                } else {member.set_name(registry.append_str(chars))},
                                            // Some arrays have their length defined as a constant value. This adds the name of that constant   |
                                            // to the type buffer.
                                            "enum"   => member.field_type.set_array_const(registry.append_str(chars)),
                                            _        => ()
                                        }
                                    }
                                }
                                VkType::TypeDef{ref mut typ,
                                                ref mut name, ..} =>
                                    match tag {
                                        "type" =>
                                            if "typedef" != chars &&
                                               ";"       != chars {
                                                *typ = registry.append_str(chars)
                                            },
                                        "name" => *name = registry.append_str(chars),
                                        "comment" => (),
                                        _ => panic!("Unexpected tag")
                                    },
                                VkType::Handle{ref mut name,
                                               ref mut dispatchable} =>
                                    match tag {
                                        "type" =>
                                            match chars {
                                                // Dispatchable defaults to true
                                                "VK_DEFINE_HANDLE"                  =>  (),
                                                "VK_DEFINE_NON_DISPATCHABLE_HANDLE" => *dispatchable = false,
                                                "("                                  |
                                                ")"                                 => (),
                                                _                                   => panic!("Unexpected handle")
                                            },
                                        "name" => *name = registry.append_str(chars),
                                        _ => ()
                                    },
                                VkType::Define{ref mut name} =>
                                    match tag {
                                        "name" => *name = registry.append_str(chars),
                                        "comment" |
                                        "type" => (),
                                        _      => panic!("Unexpected define tag")
                                    },
                                VkType::FuncPointer{ref mut name, ref mut ret, ref mut params} =>
                                    match tag {
                                        "name" => *name = registry.append_str(chars),
                                        "type" =>
                                            if let Some("type") = tag1 {
                                                if let Some(&VkElType::Const(_)) = params.last() {
                                                    params.last_mut().unwrap().set_type(registry.append_str(chars))
                                                } else {
                                                    params.push(VkElType::Var(registry.append_str(chars)))
                                                }
                                            } else if let Some("types") = tag1 {
                                                if *ret != VkElType::Unknown {
                                                    if let Some(p) = params.last_mut() {
                                                        let mut ptr_count = 0;
                                                        // Indicies of array size slice
                                                        let mut indices = (0, 0);
                                                        for (b, c) in chars.char_indices() {
                                                            match c {
                                                                '*' => ptr_count += 1,
                                                                '[' => indices.0 = b,
                                                                ']' => indices.1 = b+1,
                                                                _   => ()
                                                            }
                                                        }

                                                        if 0 < ptr_count {
                                                            p.make_ptr(ptr_count)
                                                        } else if (0, 0) != indices {
                                                            p.make_array(parse_array_index(&chars[indices.0..indices.1]).unwrap().0)
                                                        }
                                                    }

                                                    if chars.ends_with("const") {
                                                        params.push(VkElType::empty_const())
                                                    }
                                                } else {
                                                    // The type starts 8 chars into the funcpointer, after `typedef`
                                                    let start = 8;
                                                    if let Some(end) = chars.find(" (VKAPI_PTR") {
                                                        let mut new_end = end;
                                                        let mut ptr_count = 0;
                                                        for (b, c) in chars[..end].char_indices() {
                                                            if '*' == c {
                                                                ptr_count += 1;
                                                                if new_end == end {
                                                                    new_end = b;
                                                                }
                                                            }
                                                        }

                                                        // If the type is *just* void (NOT a void pointer)
                                                        if "void" == &chars[start..end] {
                                                            ret.make_void()
                                                        } else if 0 != ptr_count {
                                                            *ret = VkElType::MutPtr(registry.append_str(&chars[start..new_end]), ptr_count)
                                                        } else {
                                                            *ret = VkElType::Var(registry.append_str(&chars[start..end]))
                                                        }
                                                    }
                                                }
                                            },
                                        "comment" => (),
                                        _      => panic!("Unexpected define tag")
                                    },
                                _ => ()
                            }
                        }

                        // Perform character processing for commands
                        Characters{ref chars, tags: (tag, Some(tag1))}
                            if VkBlock::Commands == cur_block &&
                               "usage" != tag => {
                                let chars = &chars[..];
                                let command_buffer = command_buffer.as_mut().unwrap();

                                // The vulkan xml specification guarantees that the <proto> tag, where the command return type and  |
                                // name are defined, comes first. As such, if the command_buffer params list contains any value     |
                                // the characters *must* match up with a param, and appropriate action is taken. Otherwise, the tag |
                                // is going to be a <proto>. In the case that it isn't some shit has gone down and it should        |
                                // probably be reported.                                                                            |
                                if let Some(last_param) = command_buffer.params.last_mut() {
                                    process_type(chars, &mut last_param.typ);
                                    match tag1 {
                                        "proto" => panic!("Unexpected proto tag"),
                                        "param" =>
                                            match tag {
                                                "type" => last_param.typ.set_type(registry.append_str(chars)),
                                                "name" => last_param.name = registry.append_str(chars),
                                                _      => panic!("Unexpected tag")
                                            },
                                        _ => ()
                                    }
                                } else if tag1 == "proto" {
                                    match tag {
                                        "type" =>
                                            if chars == "void" {
                                                command_buffer.ret.make_void();
                                            } else {command_buffer.ret.set_type(registry.append_str(chars))},
                                        "name" => command_buffer.name = registry.append_str(chars),
                                        _      => panic!("Unexpected tag")
                                    }
                                } else {panic!("Some shit went down in a function tag, and you should probably go and report it.")}
                            }
                        Characters{..} => ()
                    }
                }

                pop_element_stack(&mut vk_elements);
                popped_to = vk_elements.len();
            }

            XmlEvent::Characters(chars) => {
                let tags = get_tags(&vk_elements);

                unsafe{ vk_elements.push(XmlElement::new_chars(chars, (&*tags.0, tags.1.map(|t| &*t)))) }
            }

            _ => ()
        }
    }

    // The loop doesn't push the last type/command, so that's handled here.
    registry.push_type(type_buffer).ok();
    registry.push_command(command_buffer).ok();
    registry.push_feature(feature_buffer).ok();
    registry.push_extn(extn_buffer).ok();
}

fn to_number(source: &str) -> Result<isize, ParseIntError> {
    if source.len() < 2 || &source[0..2] != "0x" {
        isize::from_str_radix(source, 10)
    } else {
        isize::from_str_radix(&source[2..], 16)
    }
}

fn find_attribute<'v>(source: &'v Vec<OwnedAttribute>, query: &str) -> Option<&'v str> {
    source.into_iter().skip_while(|attr| &attr.name.local_name != query).next().map(|res| &*res.value)
}

/// Takes a mutable reference to a XmlElement stack, popping the stack of Character elements until
/// it pops a Tag
fn pop_element_stack(vk_elements: &mut Vec<XmlElement>) {
    if let Some(el) = vk_elements.pop() {
        if let XmlElement::Characters{..} = el {
            pop_element_stack(vk_elements)
        }
    } else {panic!("Invalid xml; Unexpected closing tag")}
}

/// Takes a stack of XmlElements and returns a tuple containing the tag names of the last two tags.
/// If the stack is only one tag deep, returns None for the second element.
fn get_tags(vk_elements: &Vec<XmlElement>) -> (*const str, Option<*const str>) {
    fn recursive(mut vk_elements: Iter<XmlElement>, mut tags: [Option<*const str>; 2], mut index: usize) -> (*const str, Option<*const str>) {
        match vk_elements.next_back() {
            Some(el) if index < 2 => {
                if let XmlElement::Tag{ref name, ..} = *el {
                    tags[index] = Some(&name[..] as *const str);
                    index += 1;
                }
                recursive(vk_elements, tags, index)
            }
            _                     => (tags[0].unwrap(), tags[1])
        }
    }

    recursive(vk_elements.iter(), [None; 2], 0)
}

/// Takes a string slice and extracts x from [x], as long as [x] is at the end.
///
/// Returns (x, length of `chars` without [x]) if "[x]" is detected.
/// Returns (0, 0) if "[" is detected at end instead of "]"
fn parse_array_index(chars: &str) -> Option<(usize, usize)> {
    let mut chariter = chars.chars().rev();
    match chariter.next().unwrap() {
        ']' => {
            // The size of the array
            let mut size = 0;
            for (i, digit) in chariter.clone().take_while(|c| c.is_digit(10)).enumerate() {
                size += digit.to_digit(10).unwrap() as usize * (10usize.pow(i as u32));
            }

            // The length of the actual name, dropping the array length. Is decremented in the iterator below.
            let mut name_len = chars.len()-1;
            match chariter.skip_while(|c| {name_len -= 1;
                                           c.is_digit(10) | c.is_whitespace()}
                                           ).next() {
                Some('[') => (),
                Some(c)   => panic!(format!("Expected '['; found '{}'", c)),
                None      => panic!("Expected '['; found nothing")
            }
            Some((size, name_len))
        }
        '[' => Some((0, 0)),
        _   => None
    }
}

fn process_type(chars: &str, field: &mut VkElType) {
    match chars {
        "const" => field.make_const(),
        "*"     => field.make_ptr(1),
        _
            if &chars[0..1] == "[" =>
            match parse_array_index(chars) {
                Some((size, _)) => {field.make_array(size);}
                None => panic!(format!("Unexpected characters after name: {}", chars))
            },
        _       => {
            let ptr_count = chars.chars().fold(0, |acc, x| if x == '*' {acc + 1} else {acc});
            if ptr_count > 0 {
                field.make_ptr(ptr_count);
            }
        }
    }
}

#[derive(Debug)]
enum XmlElement<'a> {
    Tag {
        /// The name of the tag e.g. "hello" in <hello>
        name: String,
        /// The attributes contained within the specification xml.
        attributes: Vec<OwnedAttribute>
    },
    Characters{
        chars: String,
        tags: (&'a str, Option<&'a str>)
    }
}

impl<'a> XmlElement<'a> {
    fn new_tag(name: OwnedName, attributes: Vec<OwnedAttribute>) -> XmlElement<'a> {
        XmlElement::Tag {
            name: name.local_name,
            attributes: attributes
        }
    }

    fn new_chars(chars: String, tags: (&'a str, Option<&'a str>)) -> XmlElement<'a> {
        XmlElement::Characters {
            chars: chars,
            tags: tags
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum VkBlock {
    Types,
    Enums,
    Commands,
    Extensions,
    Feature,
    None
}

const BASE_VALUE: isize = 1000000000;
const RANGE_SIZE: isize = 1000;
