# Tower Serve Static

Tower file services using [include_dir](https://crates.io/crates/include_dir/) and [include_bytes](https://doc.rust-lang.org/std/macro.include_bytes.html) to embed assets into the binary.

## Credits

The implementation is based on the [tower-http](https://crates.io/crates/tower-http) file services (more specifically [version 0.1.2](https://github.com/tower-rs/tower-http/tree/2c110d21ed6462d0ea9b7e1b1d3d3fb128736098)) and adapted to use [include_dir](https://crates.io/crates/include_dir/)/[include_bytes](https://doc.rust-lang.org/std/macro.include_bytes.html) instead of the filesystem.
