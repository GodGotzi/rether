# rether

Rether is a Rust rendering library built on top of wgpu, designed to simplify the process of using wgpu for rendering tasks. Rether provides easy-to-use abstractions for handling picking, textures, meshes, and cameras, making it a powerful tool for both beginners and experienced developers in the graphics programming community.

## Features

- **Easy Integration**: Simplified setup and integration with wgpu.
- **Picking Support**: Easily implement object picking in your 3D scenes.
- **Texture Management**: Handle textures efficiently with built-in support.
- **Mesh Handling**: Load and manage 3D meshes with ease.
- **Camera Control**: Flexible and intuitive camera management for various use cases.

## Getting Started

### Prerequisites

To use Rether, ensure you have the following installed:

- [Rust](https://www.rust-lang.org/tools/install)
- [wgpu](https://github.com/gfx-rs/wgpu)

### Installation

Add Rether to your `Cargo.toml`:

```toml
[dependencies]
rether = "0.1.0"
