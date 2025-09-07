use std::process::Command;
use std::path::Path;

fn main() {
    // Check for explicit library path first
    if let Ok(lib_path) = std::env::var("LIB_SYSTEMD_PATH") {
        println!("cargo:rustc-link-lib=systemd");
        println!("cargo:rustc-link-search=native={}", lib_path);
        
        // Also check for header path if provided
        if let Ok(include_path) = std::env::var("INCLUDE_SYSTEMD_PATH") {
            println!("cargo:include={}", include_path);
        }
        
        // Verify the library actually exists at the specified path
        let lib_file = Path::new(&lib_path).join("libsystemd.so");
        let lib_file_a = Path::new(&lib_path).join("libsystemd.a");
        
        if lib_file.exists() || lib_file_a.exists() {
            println!("cargo:warning=Using libsystemd from LIB_SYSTEMD_PATH: {}", lib_path);
        } else {
            println!("cargo:error=LIB_SYSTEMD_PATH specified but libsystemd not found at: {}", lib_path);
            println!("cargo:error=Expected libsystemd.so or libsystemd.a in the directory");
            println!("cargo:error=Set LIB_SYSTEMD_PATH correctly and try again");
            std::process::exit(1);
        }
        return;
    }
    // Check if libsystemd is available via pkg-config
    let output = Command::new("pkg-config")
        .args(&["--exists", "libsystemd"])
        .output();
    
    match output {
        Ok(result) if result.status.success() => {
            // libsystemd found via pkg-config
            println!("cargo:rustc-link-lib=systemd");
            
            // Get the library paths
            if let Ok(libs_output) = Command::new("pkg-config")
                .args(&["--libs", "libsystemd"])
                .output()
            {
                if let Ok(libs_str) = String::from_utf8(libs_output.stdout) {
                    for flag in libs_str.split_whitespace() {
                        if let Some(lib_path) = flag.strip_prefix("-L") {
                            println!("cargo:rustc-link-search=native={}", lib_path);
                        }
                    }
                }
            }
            
            println!("cargo:warning=Using libsystemd via pkg-config");
        }
        _ => {
            // Fallback: try to link directly
            println!("cargo:warning=libsystemd not found via pkg-config, trying direct link");
            println!("cargo:rustc-link-lib=systemd");
            
            // Common library paths
            println!("cargo:rustc-link-search=native=/usr/lib");
            println!("cargo:rustc-link-search=native=/usr/lib/x86_64-linux-gnu");
            println!("cargo:rustc-link-search=native=/lib");
            println!("cargo:rustc-link-search=native=/lib/x86_64-linux-gnu");
        }
    }
    // For development/testing, we can allow building without libsystemd
    // by setting an environment variable
    if std::env::var("JOURNALD_QUERY_NO_LINK").is_ok() {
        println!("cargo:warning=Building without libsystemd linking (tests will fail)");
    }
}
