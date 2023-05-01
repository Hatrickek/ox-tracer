extern crate shaderc;

use std::error::Error;
use std::fs::File;
use shaderc::{IncludeCallbackResult, OptimizationLevel, ResolvedInclude, ShaderKind, SpirvVersion, TargetEnv};
use std::path::Path;
use std::io::{Read, Write};
use shaderc::EnvVersion::Vulkan1_2;


fn compile_shader(path: &Path, kind: ShaderKind, output: String) {
  let compiler = shaderc::Compiler::new().unwrap();
  let mut options = shaderc::CompileOptions::new().unwrap();
  options.set_optimization_level(OptimizationLevel::Zero);
  options.set_target_env(TargetEnv::Vulkan, Vulkan1_2 as u32);
  options.set_target_spirv(SpirvVersion::V1_5);
  options.set_include_callback(|name, _include_type, _containing, _depth| -> IncludeCallbackResult {
    let mut out = String::new();
    // only support includes within the same directory of the file for now.
    let path = format!("resources/shaders/{}", name);
    File::open(path).unwrap().read_to_string(&mut out).unwrap();

    if out.is_empty() {
      return Err("Could not read the include file".to_string());
    }
    
    let resolved_include = ResolvedInclude { resolved_name: name.to_string(), content: out.to_string() };
    return Ok(resolved_include);
  });
  let binary = compiler
    .compile_into_spirv(&load_file(path), kind, path.as_os_str().to_str().unwrap(), "main", Some(&options))
    .unwrap();
  save_file(output, binary.as_binary_u8());
}

fn load_file(path: &Path) -> String {
  let mut out = String::new();
  File::open(path).unwrap().read_to_string(&mut out).unwrap();
  out
}

fn save_file(path: String, binary: &[u8]) {
  File::create(path).unwrap().write_all(binary).unwrap();
}

fn to_string(kind: ShaderKind) -> &'static str {
  match kind {
    ShaderKind::Vertex => "vert",
    ShaderKind::Fragment => "frag",
    ShaderKind::Compute => "comp",
    ShaderKind::Geometry => "geom",
    ShaderKind::RayGeneration => "rgen",
    ShaderKind::ClosestHit => "rchit",
    ShaderKind::Miss => "rmiss",
    _ => "empty"
  }
}

fn main() -> Result<(), Box<dyn Error>> {
  println!("cargo:rerun-if-changed=resources/shaders");

  for entry in std::fs::read_dir("resources/shaders")? {
    let entry = entry?;

    if entry.file_type()?.is_file() {
      let in_path = entry.path();
      let shader_type = in_path
        .extension()
        .and_then(|ext| match ext.to_string_lossy().as_ref() {
          "vert" => Some(ShaderKind::Vertex),
          "geom" => Some(ShaderKind::Geometry),
          "frag" => Some(ShaderKind::Fragment),
          "comp" => Some(ShaderKind::Compute),
          "rgen" => Some(ShaderKind::RayGeneration),
          "rchit" => Some(ShaderKind::ClosestHit),
          "rmiss" => Some(ShaderKind::Miss),
          _ => None,
        });

      if let Some(shader_type) = shader_type {
        let out_path = format!("resources/shaders/{}_{}.spv", in_path.file_stem().unwrap().to_string_lossy(), to_string(shader_type));
        compile_shader(&in_path, shader_type, out_path);
      }
    }
  }
  Ok(())
}