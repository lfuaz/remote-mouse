fn main() {
    // include_dir! does not track files: rebuild when the embedded client changes.
    println!("cargo:rerun-if-changed=dist");
}
