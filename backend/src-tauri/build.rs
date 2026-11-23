fn main() {
  tauri_build::build();

  #[cfg(target_os = "windows")]
  {
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("gnu") {
      if let Ok(out_dir) = std::env::var("OUT_DIR") {
        let out_path = std::path::Path::new(&out_dir);
        let rc_path = out_path.join("resource.rc");
        let obj_path = out_path.join("mam_app.o");
        if rc_path.exists() {
          for tool in ["windres", "x86_64-w64-mingw32-windres", "llvm-windres"] {
            if let Ok(status) = std::process::Command::new(tool)
              .arg(&rc_path)
              .arg("-O")
              .arg("coff")
              .arg("-o")
              .arg(&obj_path)
              .status()
            {
              if status.success() && obj_path.exists() {
                println!("cargo:rustc-link-arg-bins={}", obj_path.display());
                break;
              }
            }
          }
        }
      }
    }
  }
}
