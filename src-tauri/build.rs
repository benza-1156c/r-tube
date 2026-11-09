use std::{env, fs, path::Path};

fn main() {
    generate_commands_macro();
    tauri_build::build();
}

fn generate_commands_macro() {
    let mut commands = Vec::new();
    scan_dir(Path::new("src"), "", &mut commands);

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("auto_commands.rs");
    let macro_code = format!(
        "#[macro_export]\nmacro_rules! app_commands {{\n    () => {{\n        tauri_specta::collect_commands![\n            {}\n        ]\n    }};\n}}",
        commands.join(",\n            ")
    );
    fs::write(dest_path, macro_code).unwrap();
}

fn scan_dir(dir: &Path, mod_prefix: &str, commands: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = entry.file_name().to_string_lossy().to_string();
            let next_prefix = if mod_prefix.is_empty() {
                name
            } else {
                format!("{mod_prefix}::{name}")
            };
            scan_dir(&path, &next_prefix, commands);
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            println!("cargo:rerun-if-changed={}", path.display());
            let file_stem = path.file_stem().unwrap().to_string_lossy();
            let current_mod = if file_stem == "mod" || file_stem == "lib" || file_stem == "main" {
                mod_prefix.to_string()
            } else if mod_prefix.is_empty() {
                file_stem.to_string()
            } else {
                format!("{mod_prefix}::{file_stem}")
            };

            if let Ok(content) = fs::read_to_string(&path) {
                let mut has_command = false;
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.contains("#[app_command]")
                        || trimmed.contains("#[rtube_macros::app_command]")
                        || trimmed.contains("#[specta::specta]")
                        || trimmed.contains("#[specta]")
                    {
                        has_command = true;
                    } else if has_command {
                        if trimmed.starts_with("pub ") && trimmed.contains("fn ") {
                            if let Some(fn_name) = trimmed
                                .split("fn ")
                                .nth(1)
                                .and_then(|s| s.split('(').next())
                                .and_then(|s| s.split('<').next())
                            {
                                let fn_name = fn_name.trim();
                                if !fn_name.is_empty() {
                                    let full_path = if current_mod.is_empty() {
                                        format!("$crate::{}", fn_name)
                                    } else {
                                        format!("$crate::{}::{}", current_mod, fn_name)
                                    };
                                    commands.push(full_path);
                                }
                            }
                            has_command = false;
                        } else if !trimmed.starts_with("//")
                            && !trimmed.starts_with("#[")
                            && !trimmed.is_empty()
                        {
                            has_command = false;
                        }
                    }
                }
            }
        }
    }
}
