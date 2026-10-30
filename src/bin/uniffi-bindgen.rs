//! UniFFI's binding generator, at the library's UniFFI version:
//! `cargo run --features uniffi-cli --bin uniffi-bindgen -- generate --library <libacs> --language swift --out-dir <dir>`.

fn main() {
    uniffi::uniffi_bindgen_main()
}
