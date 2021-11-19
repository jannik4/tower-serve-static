# Tower Serve Static

[![Build Status](https://github.com/jannik4/tower-serve-static/workflows/CI/badge.svg)](https://github.com/jannik4/tower-serve-static/actions)
[![dependency status](https://deps.rs/repo/github/jannik4/tower-serve-static/status.svg)](https://deps.rs/repo/github/jannik4/tower-serve-static)
[![codecov](https://codecov.io/gh/jannik4/tower-serve-static/branch/main/graph/badge.svg?token=Ah6sXPLFan)](https://codecov.io/gh/jannik4/tower-serve-static)
[![Lines Of Code](https://tokei.rs/b1/github/jannik4/tower-serve-static?category=code)](https://github.com/jannik4/tower-serve-static)

Tower file services using [include_dir](https://crates.io/crates/include_dir/) and [include_bytes](https://doc.rust-lang.org/std/macro.include_bytes.html) to embed assets into the binary.

## Usage

### Cargo.toml

```toml
tower-serve-static = { git = "https://github.com/jannik4/tower-serve-static", version = "0.1.0" }
include_dir = "0.7.0"
```

### Serve Static File

```rust
use tower_serve_static::{ServeFile, include_file};

// File is located relative to `CARGO_MANIFEST_DIR` (the directory containing the manifest of your package).
// This will embed and serve the `README.md` file.
let service = ServeFile::new(include_file!("/README.md"));

// Run our service using `hyper`
let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 3000));
hyper::Server::bind(&addr)
    .serve(tower::make::Shared::new(service))
    .await
    .expect("server error");
```

### Serve Static Directory

```rust
use tower_serve_static::{ServeDir};
use include_dir::{Dir, include_dir};

// Use `$CARGO_MANIFEST_DIR` to make path relative to your package.
// This will embed and serve files in the `src` directory and its subdirectories.
static ASSETS_DIR: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/src");
let service = ServeDir::new(&ASSETS_DIR);

// Run our service using `hyper`
let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 3000));
hyper::Server::bind(&addr)
    .serve(tower::make::Shared::new(service))
    .await
    .expect("server error");
```

## Credits

The implementation is based on the [tower-http](https://crates.io/crates/tower-http) file services (more specifically [version 0.1.2](https://github.com/tower-rs/tower-http/tree/2c110d21ed6462d0ea9b7e1b1d3d3fb128736098)) and adapted to use [include_dir](https://crates.io/crates/include_dir/)/[include_bytes](https://doc.rust-lang.org/std/macro.include_bytes.html) instead of the filesystem at runtime.
