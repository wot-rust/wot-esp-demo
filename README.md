# wot-esp-demo

[![LICENSE][license badge apache]][license apache]
[![LICENSE][license badge mit]][license mit]

Demo Hygro-Thermometer based on the [esp-rust-board](https://github.com/esp-rs/esp-rust-board).

- [x] http version based on [std-training](https://github.com/esp-rs/std-training)


# Deploy

## Rust prerequisites
- Install `espflash`, `ldproxy` and `cargo-espflash`
```
$ cargo install espflash ldproxy cargo-espflash
```
- Install a `nightly` rustc at least for now.
```
$ rustup install nightly --component rust-src
```

## Distribution specific prerequisites
- Install `clang` and `llvm` with support for RISC-V
- Install `libuv`
Depending on your distribution the package may be `{pkgname}-dev` or `{pkgname}-devel`.

## Building and running
- Make sure to connect the board and that its serial/jtag gets detected by your system.
- Populate the `cfg.toml` with the wifi credentials.

If the toolchain is correctly installed the usual `cargo build` and `cargo run` will work.

<!-- Links -->
[license apache]: LICENSES/Apache-2.0.txt
[license mit]: LICENSES/MIT.txt

<!-- Badges -->
[license badge apache]: https://img.shields.io/badge/license-Apache_2.0-blue.svg
[license badge mit]: https://img.shields.io/badge/license-MIT-blue.svg
