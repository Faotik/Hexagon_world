use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();

    // let (lib_name, build_cmd, lib_path) = if target_os == "windows" {
    //     ("glfw3", "--build .", "./deps/glfw/src/Release")
    // } else {
    //     ("glfw", "-DBUILD_SHARED_LIBS=ON .", "./deps/glfw/src")
    // };

    //Cmake

    let glfw_lib = if target_os == "windows" {
        Path::new("./deps/glfw/src/Release/glfw3.dll")
    } else {
        Path::new("./deps/libglfw.so.3")
    };

    if !glfw_lib.exists() {
        println!("Building GLFW for {}...", target_os);

        let glfw_dir = "./deps/glfw";
        if !Path::new(glfw_dir).exists() {
            let status = Command::new("git")
                .args(&[
                    "clone",
                    "--depth",
                    "1",
                    "https://github.com/glfw/glfw.git",
                    glfw_dir,
                ])
                .status()
                .expect("Failed to clone GLFW");
            assert!(status.success());
        }

        let mut cmake_args = vec![];
        cmake_args.extend(build_cmd.clone());
        let status = Command::new("cmake")
            .current_dir(glfw_dir)
            .args(&cmake_args)
            .status()
            .expect("Failed to run CMake");
        assert!(status.success());

        Command::new("cmake")
            .current_dir(glfw_dir)
            .args(&["."])
            .status()
            .expect("Failed to build GLFW");

        assert!(status.success());
    }

    // // Tell Rust to link to the dynamic library
    // println!("cargo:rustc-link-search=native={}", lib_path);
    // println!("cargo:rustc-link-lib=dylib={}", lib_name);
}
