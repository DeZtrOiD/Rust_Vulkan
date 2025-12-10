
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
        ("sphere.vert", "vert_sphere.spv"),
        ("sphere.frag", "frag_sphere.spv"),
        ("light.vert", "vert_light.spv"),
        ("light.frag", "frag_light.spv"),
        ("imgui.vert", "imgui_vert.spv"),
        ("imgui.frag", "imgui_frag.spv"),
        ("shadows.vert", "vert_shadows.spv"),
        ("shadows.frag", "frag_shadows.spv"),
        ("light_shadows.frag", "frag_light_shadows.spv"),
        ("light_shadows.vert", "vert_light_shadows.spv"),
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
