use cfg_aliases::cfg_aliases;

// generate config aliases, similar to the ones winit has to avoid long-winded cfg directives
fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    cfg_aliases! {
        android: { target_os = "android" },
        macos: { target_os = "macos" },
        ios: { all(target_vendor = "apple", not(target_os = "macos")) },
        windows: { target_os = "windows" },
        linux: { target_os = "linux" },
    }
}
