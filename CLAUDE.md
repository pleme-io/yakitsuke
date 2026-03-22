# yakitsuke — Safe ROM Flasher

Safe ROM flasher with pre-flight checks and rollback. Consumes FastbootTransport, VbmetaParser, SparseImageParser, UsbEnumerator traits.

## Build & Test

```bash
cargo build
cargo test
cargo run -- flash <image>
```

## Conventions

- Edition 2024, Rust 1.91.0+, MIT, clippy pedantic
- Release: codegen-units=1, lto=true, opt-level="z", strip=true
