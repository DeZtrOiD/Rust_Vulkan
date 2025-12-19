
use std::process::Command;
use std::fs;
use std::path::Path;


fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            let dst_path = dst.as_ref().join(entry.file_name());
            println!("Copying model file: {:?} -> {:?}", entry.path(), dst_path);
            fs::copy(entry.path(), dst_path)?;
            println!("cargo:rerun-if-changed={}", entry.path().display());
        }
    }
    Ok(())
}

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

        let status = Command::new("glslc")
            .args([src.to_str().unwrap(), "-o", dst.to_str().unwrap()])
            .status()
            .expect("Failed to run glslc");

        if !status.success() {
            panic!("Shader compilation failed: {}", src.display());
        }

        println!("cargo:rerun-if-changed={}", src.display());
    }

    let out_dir = format!("target/{}/", profile);
    let model_src_dir = Path::new("obj_3d");
    let model_dst_dir = Path::new(&out_dir).join("obj_3d");

    if model_src_dir.exists() {
        copy_dir_all(model_src_dir, &model_dst_dir)
            .expect("Failed to copy 3D models recursively");
    } else {
        eprintln!("Warning: 3D model source directory not found: {:?}", model_src_dir);
    }


    // println!("cargo:rustc-env=SHADER_PATH=target/debug/shaders");
}
