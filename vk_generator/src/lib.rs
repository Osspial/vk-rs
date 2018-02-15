//! Automatically generates bindings for the Vulkan API. In order to use, first create a
//! [`VkRegistry`] struct with a valid Vulkan XML file, and then call either one of the two
//! generation functions ([`gen_global()`] or [`gen_struct()`]) or a custom one. Further
//! documentation can be found in [`VkRegistry`] and [`GenConfig`], and looking over the provided
//! [examples](https://github.com/Osspial/vk-rs/tree/master/vk_generator/examples) is encouraged.
//!
//! [`VkRegistry`]: ./struct.VkRegistry.html
//! [`GenConfig`]: ./struct.GenConfig.html
//! [`gen_global()`]: ./struct.VkRegistry.html#method.gen_global
//! [`gen_struct()`]: ./struct.VkRegistry.html#method.gen_struct

extern crate xml;
extern crate boolinator;

use std::mem;

#[cfg(feature = "unstable_generator_api")]
pub mod registry;
#[cfg(feature = "unstable_generator_api")]
pub mod generator;

#[cfg(not(feature = "unstable_generator_api"))]
mod registry;
#[cfg(not(feature = "unstable_generator_api"))]
mod generator;

pub use registry::{VkRegistry, VkVersion};
pub use generator::{GenConfig, VariantPaddingConfig};

#[inline]
fn to_option<'u>(s: *const str) -> Option<&'u str> {
    unsafe{ mem::transmute(s) }
}
