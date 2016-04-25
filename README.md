# s_app_dir

## Usage

Cargo.toml:

```toml
[package]
...

[dependencies]
s_app_dir = "*" # or semantic versioning
```

main.rc:

```rust
extern crate s_app_dir;

use s_app_dir::{AppDir, XdgDir};

fn main() {
    let app_dir = AppDir::new("foo-bar-app");
    println!("{:?}", app_dir.xdg_dir(XdgDir::Config));
}
```
