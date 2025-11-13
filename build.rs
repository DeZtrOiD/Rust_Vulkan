
use std::process::Command;
use std::fs;
use std::path::Path;

fn main() {
    let shader_dir = Path::new("shaders");
    let profile = std::env::var("PROFILE").unwrap(); // debug или release
    let shdr = format!("target/{}/shaders", profile); // путь для шейдеров
    let bin_dir = Path::new(&shdr); 

    fs::create_dir_all(&bin_dir).unwrap();

    let shaders = [
        ("vert.vert", "vert.spv"),
        ("frag.frag", "frag.spv"),
    ];

    for (src_name, dst_name) in shaders {
        let src = shader_dir.join(src_name);
        let dst = bin_dir.join(dst_name);

        // 1. Компиляция через glslc
        let status = Command::new("glslc")
            .args([src.to_str().unwrap(), "-o", dst.to_str().unwrap()])
            .status()
            .expect("Failed to run glslc");

        if !status.success() {
            panic!("Shader compilation failed: {}", src.display());
        }

        // 2. Сообщить Cargo, что если шейдер изменился — пересобрать
        println!("cargo:rerun-if-changed={}", src.display());
    }

    // 3. Чтобы бинарник мог найти шейдеры
    println!("cargo:rustc-env=SHADER_PATH=target/debug/shaders");
}
