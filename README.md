# BoltFFI - a fast multi-language bindings generator for Rust

BoltFFI (Pronounced bolt-eff-eff-eye) generates native bindings from Rust libraries. Write your code once in Rust, ship it everywhere.

Quick links: [User Guide](https://boltffi.dev/docs/overview) | [Tutorial](https://boltffi.dev/docs/tutorial) | [Getting Started](https://boltffi.dev/docs/getting-started)

## Why BoltFFI?

BoltFFI was built because serialization-based FFI is slow. Tools like UniFFI serialize every value to a byte buffer on each call. That overhead shows up when you're making thousands of FFI calls per second.

BoltFFI uses zero-copy where possible. Primitives pass as raw values. Structs with primitive fields pass as pointers to memory both sides can read directly. Only strings and collections go through encoding.

The result:

| Operation | BoltFFI | UniFFI | Speedup |
|-----------|---------|--------|---------|
| Primitives | <1 ns | 625 ns | ∞ |
| 1,000 structs | 1,958 ns | 1,195,354 ns | 611x |
| 10,000 i32 values | 1,291 ns | 1,991,146 ns | 1,542x |

Full benchmark code in [bench_demo](./bench_demo).

## What it does

Mark your Rust types with `#[data]` and functions, objects or traits with `#[export]`:

```rust
use boltffi::*;

#[data]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

#[export]
pub fn distance(a: Point, b: Point) -> f64 {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    (dx * dx + dy * dy).sqrt()
}
```

Run `boltffi pack` and get native libraries ready to import:

```bash
boltffi pack apple      # XCFramework for Swift
boltffi pack android    # JNI libraries for Kotlin
```

The generated bindings use each language's idioms. Swift gets async/await. Kotlin gets coroutines. Errors become native exceptions.

## Supported languages

| Language | Status |
|----------|--------|
| Swift | Full support |
| Kotlin | Full support |
| WASM | In progress |
| Python | Soon |
| C# | Soon |

Want another language? [Open an issue](https://github.com/boltffi/boltffi/issues).

## Installation

```bash
cargo install boltffi_cli
```

Add to your `Cargo.toml`:

```toml
[dependencies]
boltffi = "0.1"

[lib]
crate-type = ["staticlib", "cdylib"]
```

## Documentation

- [Overview](https://boltffi.dev/docs/overview)
- [Getting Started](https://boltffi.dev/docs/getting-started)
- [Tutorial](https://boltffi.dev/docs/tutorial)
- [Types](https://boltffi.dev/docs/types)
- [Async](https://boltffi.dev/docs/async)
- [Streaming](https://boltffi.dev/docs/streaming)

## Alternative tools

Other tools that solve similar problems:

- [UniFFI](https://github.com/mozilla/uniffi-rs) - Mozilla's binding generator, uses serialization-based approach
- [Diplomat](https://github.com/rust-diplomat/diplomat) - Focused on C/C++ interop
- [cxx](https://github.com/dtolnay/cxx) - Safe C++/Rust interop

## Contributing
Contributions are warmly welcomed 🙌

- [File an issue](https://github.com/boltffi/boltffi/issues)
- [Submit a PR](https://github.com/boltffi/boltffi/pulls)

## License
BOLTFFI is released under the MIT license. See [LICENSE](./LICENSE) for more information.
