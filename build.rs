use cfg_aliases::cfg_aliases;

// generate config aliases, similar to the ones winit has to avoid long-winded cfg directives
fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    cfg_aliases! {
        android_platform: { target_os = "android" },
        macos_platform: { target_os = "macos" },
        ios_platform: { all(target_vendor = "apple", not(target_os = "macos")) },
        windows_platform: { target_os = "windows" },
        linux_platform: { target_os = "linux" },
    }
}
