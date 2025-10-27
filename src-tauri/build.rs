fn main() {
    // Configure for Windows 11 compatibility
    tauri_build::build();

    // Set Windows-specific build flags for modern Windows APIs
    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-arg=/SUBSYSTEM:WINDOWS");
        println!("cargo:rustc-link-arg=/ENTRY:mainCRTStartup");

        // Ensure compatibility with Windows 11 TaskDialog APIs
        println!("cargo:rustc-link-lib=comctl32");
        println!("cargo:rustc-link-lib=user32");
        println!("cargo:rustc-link-lib=kernel32");

        // Set minimum Windows version to Windows 10 1809 (RS5)
        println!("cargo:rustc-cfg=windows_subsystem");
    }

    println!("cargo:rustc-cfg=desktop");
}
