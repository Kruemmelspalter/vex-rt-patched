# vex-rt

[![](https://img.shields.io/crates/v/vex-rt)](https://crates.io/crates/vex-rt)
[![docs.rs](https://docs.rs/vex-rt/badge.svg)](https://docs.rs/vex-rt/)

A Rust runtime for the Vex V5 built on top of [PROS](https://pros.cs.purdue.edu/).

## Disclamer

vex-rt is an open-source community project. vex-rt is neither endorsed by or affiliated with Innovation First, Inc. VEX and VEX Robotics are trademarks or service marks of Innovation First, Inc.

## Quickstart

you will need:
1. A Rust toolchain managed with `rustup`
2. An `arm-none-eabi` toolchain
3. `pros-cli`


```shell
# Simply plug in a V5 and run:
cargo run --example hello-world
```

## Versions

| Versions starting with... | ...use PROS kernel version... |
| ------------------------- | ----------------------------- |
| 0.10.1                    | 3.5.2                         |
| 0.4.1                     | 3.4.0                         |
| 0.1.0                     | 3.3.1                         |
