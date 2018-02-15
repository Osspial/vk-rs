extern crate vk_generator;
extern crate vk_api;

use std::path::Path;
use std::fs::{File, DirBuilder};

use vk_generator::VkVersion;


fn main() {
    let out = env!("OUT_DIR");
    DirBuilder::new().recursive(true).create(&out).unwrap();

    let mut file = File::create(&Path::new(&out).join("gen_struct.rs")).unwrap();
    vk_generator::VkRegistry::new(vk_api::VK_XML).gen_struct(
        &mut file,
        VkVersion(1, 0),
        &["VK_KHR_surface"],
        Default::default()
    );
}
