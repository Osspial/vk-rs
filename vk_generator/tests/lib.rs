extern crate vk_generator;
extern crate vk_api;

use std::path::Path;
use std::fs::{File, DirBuilder};
use std::process::Command;
use std::io::Write;
use std::str;

use vk_generator::{VkVersion, GenConfig};

#[test]
fn default_global() {
    let out = env!("OUT_DIR");
    DirBuilder::new().recursive(true).create(&out).unwrap();

    let mut file = File::create(&Path::new(&out).join("default_global.rs")).unwrap();
    writeln!(file, "fn main() {{}} mod vk {{").unwrap();
    vk_generator::VkRegistry::new(vk_api::VK_XML)
        .gen_global(&mut file,
                    VkVersion(1, 0), 
                    &["VK_KHR_surface", "VK_EXT_debug_report"],
                    Default::default());
    writeln!(file, "}}").unwrap();

    assert_eq!("", str::from_utf8(&Command::new("rustc").current_dir(&out).arg("default_global.rs").output().unwrap().stderr).unwrap());
}

#[test]
fn default_struct() {
    let out = env!("OUT_DIR");
    DirBuilder::new().recursive(true).create(&out).unwrap();

    let mut file = File::create(&Path::new(&out).join("default_struct.rs")).unwrap();
    writeln!(file, "fn main() {{}} mod vk {{").unwrap();
    vk_generator::VkRegistry::new(vk_api::VK_XML)
        .gen_struct(&mut file,
                    VkVersion(1, 0), 
                    &["VK_KHR_surface", "VK_EXT_debug_report"],
                    Default::default());
    writeln!(file, "}}").unwrap();

    assert_eq!("", str::from_utf8(&Command::new("rustc").current_dir(&out).arg("default_struct.rs").output().unwrap().stderr).unwrap());
}

#[test]
fn nondefault_global() {
    let out = env!("OUT_DIR");
    DirBuilder::new().recursive(true).create(&out).unwrap();

    let mut file = File::create(&Path::new(&out).join("nondefault_global.rs")).unwrap();
    writeln!(file, "{}", include_str!("./libc_dummy.rs")).unwrap();
    writeln!(file, "fn main() {{}} mod vk {{").unwrap();
    vk_generator::VkRegistry::new(vk_api::VK_XML)
        .gen_global(&mut file,
                    VkVersion(1, 0), 
                    &["VK_KHR_surface", "VK_EXT_debug_report"],
                    GenConfig::new()
                        .remove_type_prefix(true)
                        .remove_vk_result_prefix(false)
                        .remove_command_prefix(false)
                        .remove_variant_padding(false)
                        .remove_bitmask_prefix(false)
                        .snake_case_commands(false)
                        .camel_case_variants(false)
                        .snake_case_members(false)
                        .wrap_bitmasks(false)
                        .use_libc_types(true));
    writeln!(file, "}}").unwrap();

    assert_eq!("", str::from_utf8(&Command::new("rustc").current_dir(&out).arg("nondefault_global.rs").output().unwrap().stderr).unwrap());
}

#[test]
fn nondefault_struct() {
    let out = env!("OUT_DIR");
    DirBuilder::new().recursive(true).create(&out).unwrap();

    let mut file = File::create(&Path::new(&out).join("nondefault_struct.rs")).unwrap();
    writeln!(file, "{}", include_str!("./libc_dummy.rs")).unwrap();
    writeln!(file, "fn main() {{}} mod vk {{").unwrap();
    vk_generator::VkRegistry::new(vk_api::VK_XML)
        .gen_struct(&mut file,
                    VkVersion(1, 0), 
                    &["VK_KHR_surface", "VK_EXT_debug_report"],
                    GenConfig::new()
                        .remove_type_prefix(true)
                        .remove_vk_result_prefix(false)
                        .remove_command_prefix(false)
                        .remove_variant_padding(false)
                        .remove_bitmask_prefix(false)
                        .snake_case_commands(false)
                        .camel_case_variants(false)
                        .snake_case_members(false)
                        .wrap_bitmasks(false)
                        .use_libc_types(true));
    writeln!(file, "}}").unwrap();

    assert_eq!("", str::from_utf8(&Command::new("rustc").current_dir(&out).arg("nondefault_struct.rs").output().unwrap().stderr).unwrap());
}
