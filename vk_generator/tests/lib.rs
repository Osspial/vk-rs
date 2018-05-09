extern crate vk_generator;
extern crate vk_api;

use std::path::Path;
use std::fs::{File, DirBuilder};
use std::process::Command;
use std::io::Write;

use vk_generator::{VkVersion, GenConfig, VariantPaddingConfig};

#[test]
fn default_global() {
    let out = env!("OUT_DIR");
    DirBuilder::new().recursive(true).create(&out).unwrap();

    let mut file = File::create(&Path::new(&out).join("default_global.rs")).unwrap();
    writeln!(file, "fn main() {{}} mod vk {{").unwrap();
    vk_generator::VkRegistry::new(vk_api::VK_XML)
        .gen_global(
            &mut file,
            VkVersion(1, 0),
            &["VK_KHR_surface", "VK_EXT_debug_report"],
            Default::default()
        );
    writeln!(file, "}}").unwrap();

    let error = String::from_utf8(Command::new("rustc").current_dir(&out).arg("default_global.rs").output().unwrap().stderr).unwrap();
    if error != "" {
        panic!("{}", error);
    }
}

#[test]
fn default_struct() {
    let out = env!("OUT_DIR");
    DirBuilder::new().recursive(true).create(&out).unwrap();

    let mut file = File::create(&Path::new(&out).join("default_struct.rs")).unwrap();
    writeln!(file, "fn main() {{}} mod vk {{").unwrap();
    vk_generator::VkRegistry::new(vk_api::VK_XML)
        .gen_struct(
            &mut file,
            VkVersion(1, 0),
            &["VK_KHR_surface", "VK_EXT_debug_report"],
            Default::default()
        );
    writeln!(file, "}}").unwrap();

    let error = String::from_utf8(Command::new("rustc").current_dir(&out).arg("default_struct.rs").output().unwrap().stderr).unwrap();
    if error != "" {
        panic!("{}", error);
    }
}

#[test]
fn nondefault_global() {
    let out = env!("OUT_DIR");
    DirBuilder::new().recursive(true).create(&out).unwrap();

    let mut file = File::create(&Path::new(&out).join("nondefault_global.rs")).unwrap();
    writeln!(file, "{}", include_str!("./libc_dummy.rs")).unwrap();
    writeln!(file, "fn main() {{}} mod vk {{").unwrap();
    vk_generator::VkRegistry::new(vk_api::VK_XML)
        .gen_global(
            &mut file,
            VkVersion(1, 0),
            &["VK_KHR_surface", "VK_EXT_debug_report"],
            GenConfig {
                remove_type_prefix: true,
                remove_vk_result_prefix: false,
                remove_command_prefix: false,
                remove_bitmask_prefix: false,
                remove_const_prefix: false,
                variant_padding: VariantPaddingConfig::RemovePrefix,
                snake_case_commands: false,
                camel_case_variants: false,
                snake_case_members: false,
                use_native_enums: false,
                use_native_unions: true,
                wrap_bitmasks: false,
                wrap_non_dispatchable_handles: false,
                use_libc_types: true,
                ..GenConfig::default()
            },
        );
    writeln!(file, "}}").unwrap();

    let error = String::from_utf8(Command::new("rustc").current_dir(&out).arg("nondefault_global.rs").output().unwrap().stderr).unwrap();
    if error != "" {
        panic!("{}", error);
    }
}

#[test]
fn nondefault_struct() {
    let out = env!("OUT_DIR");
    DirBuilder::new().recursive(true).create(&out).unwrap();

    let mut file = File::create(&Path::new(&out).join("nondefault_struct.rs")).unwrap();
    writeln!(file, "{}", include_str!("./libc_dummy.rs")).unwrap();
    writeln!(file, "fn main() {{}} mod vk {{").unwrap();
    vk_generator::VkRegistry::new(vk_api::VK_XML)
        .gen_struct(
            &mut file,
            VkVersion(1, 0),
            &["VK_KHR_surface", "VK_EXT_debug_report"],
            GenConfig {
                remove_type_prefix: true,
                remove_vk_result_prefix: false,
                remove_command_prefix: false,
                remove_bitmask_prefix: false,
                remove_const_prefix: false,
                variant_padding: VariantPaddingConfig::Keep,
                snake_case_commands: false,
                camel_case_variants: false,
                snake_case_members: false,
                use_native_enums: false,
                use_native_unions: true,
                wrap_bitmasks: false,
                wrap_non_dispatchable_handles: false,
                use_libc_types: true,
                ..GenConfig::default()
            }
        );
    writeln!(file, "}}").unwrap();

    let error = String::from_utf8(Command::new("rustc").current_dir(&out).arg("nondefault_struct.rs").output().unwrap().stderr).unwrap();
    if error != "" {
        panic!("{}", error);
    }
}
