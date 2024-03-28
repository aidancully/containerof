extern crate rustc_version;

use rustc_version::{version, Version};

fn main() {
    if version().unwrap() >= Version::parse("1.77.0").unwrap() {
        println!("cargo:rustc-cfg=has_offset_of");
    }
}
