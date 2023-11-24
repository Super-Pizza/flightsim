# vk-alloc

[![Latest version](https://img.shields.io/crates/v/vk-alloc.svg)](https://crates.io/crates/vk-alloc)
[![Documentation](https://docs.rs/vk-alloc/badge.svg)](https://docs.rs/vk-alloc)
![ZLIB](https://img.shields.io/badge/license-zlib-blue.svg)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)
![Apache](https://img.shields.io/badge/license-Apache-blue.svg)

A segregated list memory allocator for Vulkan written in Rust.

Targets Vulkan 1.2+ using `ash`.

## Features

All features are optional by default.

* `vk-buffer-device-address`: Enables the usage of "vkGetBufferDeviceAddress". Needs the Vulkan
  1.2 "bufferDeviceAddress" device feature enabled.

## License

Licensed under MIT or Apache-2.0 or ZLIB.
