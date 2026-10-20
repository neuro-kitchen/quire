//! `cargo run -p uniffi-bindgen -- generate --library <libquire_ffi> --language kotlin --out-dir <dir>`
//! (the app's Gradle build runs it).

fn main() {
    uniffi::uniffi_bindgen_main()
}
