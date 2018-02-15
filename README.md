# vk-rs [![Build Status](https://travis-ci.org/Osspial/vk-rs.svg?branch=master)](https://travis-ci.org/Osspial/vk-rs)
This repository contiains the necessary starting blocks for building a wrapper around the Vulkan
API.


## [vk_generator](./vk_generator) [![Version](https://img.shields.io/crates/v/vk_generator.svg)](https://crates.io/crates/vk_generator)
Generate bindings to the Vulkan API.

```toml
[build-dependencies]
vk_generator = "0.3.0"
```


## [vk_api](./vk_api) [![Version](https://img.shields.io/crates/v/vk_api.svg)](https://crates.io/crates/vk_api)
The Vulkan Registry exposed as the bytestring `vk_api::VK_XML`.

```toml
[build-dependencies]
vk_api = "1.0"
```
