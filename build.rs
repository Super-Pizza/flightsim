use std::{env, fs::read_dir, path::PathBuf, process::Command};
fn main() {
    println!("cargo:rerun-if-changed=src/shaders/");
    let root_dir = &env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut shader_dir = PathBuf::from(root_dir);
    shader_dir.push("src/shaders");
    read_dir(shader_dir).unwrap().for_each(|shader| {
        let shader = shader.unwrap().path();
        let mut output = PathBuf::from(env::var("OUT_DIR").unwrap()).join("out");
        output.set_file_name(shader.file_name().unwrap());
        output.set_extension("spv");
        #[cfg(feature = "debuginfo")]
        let args = [
            "-g",
            shader.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
        ];
        #[cfg(not(feature = "debuginfo"))]
        let args = [
            "-O",
            shader.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
        ];
        let mut cmd = Command::new(
            PathBuf::from(env::var("VULKAN_SDK").unwrap_or_else(|_| "".into()))
                .join("Bin")
                .join("glslc.exe"),
        )
        .args(args)
        .output();
        cmd = cmd.or_else(|_| Command::new("glslc").args(args).output());
        let cmd = cmd.unwrap();
        if !cmd.status.success() {
            panic!(
                "Command failed with:
[stdout] {}
[stderr] {}",
                String::from_utf8(cmd.stdout).unwrap(),
                String::from_utf8(cmd.stderr).unwrap()
            )
        }
    });
}
