use anyhow::{bail, Context, Result};
use blueprint_compiler::{build_executable, run};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub version: String,
    pub author: String,
    pub compiler: CompilerConfig,
    pub dependencies: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompilerConfig {
    pub target: String,
    pub optimizer: String,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: printable <init|build|run|install> [file]");
        std::process::exit(1);
    }

    let result = match args[1].as_str() {
        "init" => cmd_init(),
        "module-init" => cmd_module_init(args.get(2).map(|s| s.as_str())),
        "compile-module" => cmd_compile_module(args.get(2).map(|s| s.as_str()), args.get(3).map(|s| s.as_str())),
        "build" => cmd_build(args.get(2).map(|s| s.as_str())),
        "run" => cmd_run(args.get(2).map(|s| s.as_str())),
        "install" => cmd_install(args.get(2).map(|s| s.as_str())),
        other => {
            eprintln!("Error: Unknown command '{}'", other);
            eprintln!("Usage: printable <init|module-init|compile-module|build|run|install> [args...]");
            std::process::exit(1);
        }
    };

    if let Err(err) = result {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}

fn require_bls_file() -> Result<ProjectConfig> {
    let candidate = if Path::new("config.bls").exists() {
        Path::new("config.bls")
    } else if Path::new("app.bls").exists() {
        Path::new("app.bls")
    } else {
        bail!("Error: 'config.bls' configuration file is required. Run 'printable init' to create one.");
    };

    let content = fs::read_to_string(candidate)
        .with_context(|| format!("Failed to read config file '{}'", candidate.display()))?;
    let config: ProjectConfig = serde_json::from_str(&content)
        .with_context(|| format!("Invalid JSON or schema in '{}'", candidate.display()))?;
    Ok(config)
}

fn cmd_init() -> Result<()> {
    let config_path = Path::new("config.bls");
    let main_path = Path::new("main.bl");

    if config_path.exists() && main_path.exists() {
        println!("Project already initialized (config.bls and main.bl exist).");
        return Ok(());
    }

    // 1. Create config.bls
    if !config_path.exists() {
        let default_config = ProjectConfig {
            name: "Blueprint App".to_string(),
            version: "1.0.0".to_string(),
            author: "Developer".to_string(),
            compiler: CompilerConfig {
                target: "native".to_string(),
                optimizer: "optimized".to_string(),
            },
            dependencies: {
                let mut deps = HashMap::new();
                deps.insert("core".to_string(), "1.0.0".to_string());
                deps
            },
        };
        let formatted = serde_json::to_string_pretty(&default_config)?;
        fs::write(config_path, formatted)?;
        println!("✓ Created config.bls");
    }

    // 2. Create main.bl with simple Hello World program
    if !main_path.exists() {
        let hello_world = r#"@blueprint import Blueprint
Blueprint.Setup("Hello World", true)

func Main(): [
    output("Hello, World!")
]
"#;
        fs::write(main_path, hello_world)?;
        println!("✓ Created main.bl (Hello World)");
    }

    println!("\nProject ready! Run or build with:");
    println!("  printable run");
    println!("  printable build");
    Ok(())
}

fn cmd_module_init(name_arg: Option<&str>) -> Result<()> {
    let mod_name = match name_arg {
        Some(name) => name.trim(),
        None => {
            eprintln!("Usage: printable module-init <module_name>");
            bail!("Missing module name. Example: printable module-init math_utils");
        }
    };

    if mod_name.is_empty() {
        bail!("Module name cannot be empty.");
    }

    let target_dir = Path::new(mod_name);
    let functions_dir = target_dir.join("functions");
    fs::create_dir_all(&functions_dir)
        .with_context(|| format!("Failed to create module directory '{}'", target_dir.display()))?;

    // 1. Create package.bls
    let bls_file = target_dir.join("package.bls");
    if !bls_file.exists() {
        let meta = serde_json::json!({
            "name": mod_name,
            "version": "1.0.0",
            "author": "Developer",
            "functions": [
                "example_func"
            ]
        });
        fs::write(&bls_file, serde_json::to_string_pretty(&meta)?)?;
        println!("✓ Created {}/package.bls", mod_name);
    }

    // 2. Create package.bl
    let bl_file = target_dir.join("package.bl");
    if !bl_file.exists() {
        let content = format!(
            "// Blueprint Module: {}\n// package.bl maps exports and defines shared structures\n\ndefstruct {}Config [\n    enabled: bool\n]\n",
            mod_name,
            mod_name
        );
        fs::write(&bl_file, content)?;
        println!("✓ Created {}/package.bl", mod_name);
    }

    // 3. Create functions/example_func.bl
    let func_file = functions_dir.join("example_func.bl");
    if !func_file.exists() {
        let func_code = format!(
            "// Component function for {}\nfunc example_func(val: int): int [\n    return val * 2\n]\n",
            mod_name
        );
        fs::write(&func_file, func_code)?;
        println!("✓ Created {}/functions/example_func.bl", mod_name);
    }

    println!("\n✓ Module '{}' initialized successfully!", mod_name);
    println!("Directory layout created:");
    println!("  {}/", mod_name);
    println!("  ├── package.bls");
    println!("  ├── package.bl");
    println!("  └── functions/");
    println!("      └── example_func.bl");
    println!("\nTo compile into a single distribution module, run:");
    println!("  printable compile-module {}", mod_name);
    Ok(())
}

fn cmd_compile_module(module_path_arg: Option<&str>, output_arg: Option<&str>) -> Result<()> {
    let mod_path_str = module_path_arg.unwrap_or(".");
    let mod_path = Path::new(mod_path_str);

    if !mod_path.exists() {
        bail!("Module path '{}' does not exist.", mod_path.display());
    }

    let mod_name = if mod_path.is_dir() {
        mod_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("module")
    } else {
        "module"
    };

    println!("Compiling Blueprint module '{}' into .blm...", mod_name);

    let mut combined_code = String::new();
    combined_code.push_str(&format!("// Blueprint Compiled Module: {}\n", mod_name));

    // 1. Read package.bls
    let bls_path = if mod_path.is_dir() {
        mod_path.join("package.bls")
    } else {
        PathBuf::from("package.bls")
    };

    let mut version = "1.0.0".to_string();
    if bls_path.is_file() {
        if let Ok(content) = fs::read_to_string(&bls_path) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(v) = val.get("version").and_then(|v| v.as_str()) {
                    version = v.to_string();
                }
            }
        }
    }
    combined_code.push_str(&format!("// Version: {}\n\n", version));

    // 2. Read package.bl
    let bl_path = if mod_path.is_dir() {
        mod_path.join("package.bl")
    } else {
        PathBuf::from("package.bl")
    };

    if bl_path.is_file() {
        let content = fs::read_to_string(&bl_path)
            .with_context(|| format!("Failed to read '{}'", bl_path.display()))?;
        combined_code.push_str(&format!("// --- {} ---\n", bl_path.display()));
        combined_code.push_str(&content);
        combined_code.push_str("\n\n");
    }

    // 3. Read all function files in functions/
    let functions_dir = if mod_path.is_dir() {
        mod_path.join("functions")
    } else {
        PathBuf::from("functions")
    };

    if functions_dir.is_dir() {
        let mut entries = fs::read_dir(&functions_dir)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("bl"))
            .collect::<Vec<_>>();
        entries.sort();

        for file_path in entries {
            let func_code = fs::read_to_string(&file_path)
                .with_context(|| format!("Failed to read function file '{}'", file_path.display()))?;
            let func_name = file_path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("func");
            combined_code.push_str(&format!("// --- function: {} ---\n", func_name));
            combined_code.push_str(&func_code);
            combined_code.push_str("\n\n");
        }
    }

    // 4. Determine output .blm path
    let out_file = if let Some(out) = output_arg {
        PathBuf::from(out)
    } else if mod_path.is_dir() && mod_path != Path::new(".") {
        mod_path.join(format!("{}.blm", mod_name))
    } else {
        PathBuf::from(format!("{}.blm", mod_name))
    };

    fs::write(&out_file, &combined_code)
        .with_context(|| format!("Failed to write compiled module to '{}'", out_file.display()))?;

    println!("✓ Successfully compiled module to '{}'", out_file.display());
    println!("  You can now distribute this single file or place it in modules/{}.blm", mod_name);
    Ok(())
}

fn cmd_build(file: Option<&str>) -> Result<()> {
    let config = require_bls_file()?;
    println!(
        "Loaded configuration for '{}' v{}",
        config.name, config.version
    );

    let default_file = if Path::new("main.bl").exists() {
        "main.bl"
    } else if Path::new("src/main.bl").exists() {
        "src/main.bl"
    } else {
        "main.bl"
    };

    let file_path = file.unwrap_or(default_file);
    let src = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read source file '{}'", file_path))?;

    println!("Building '{}' to a native executable...", file_path);

    let input_path = Path::new(file_path);
    let file_stem = input_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("app");

    fs::create_dir_all("bin").context("Failed to create bin/ directory")?;

    let mut out_path = PathBuf::from("bin");
    out_path.push(format!("{}.exe", file_stem));

    match build_executable(&src, &out_path) {
        Ok(built_exe) => {
            println!(
                "✓ Build successful → {}",
                built_exe.display()
            );
        }
        Err(e) => {
            eprintln!("✗ Build error:\n  {}", e);
            std::process::exit(1);
        }
    }
    Ok(())
}

fn cmd_run(file: Option<&str>) -> Result<()> {
    // config.bls or app.bls is optional for run
    if Path::new("config.bls").exists() || Path::new("app.bls").exists() {
        let _ = require_bls_file();
    }

    let default_file = if Path::new("main.bl").exists() {
        "main.bl"
    } else if Path::new("src/main.bl").exists() {
        "src/main.bl"
    } else {
        "main.bl"
    };

    let file_path = file.unwrap_or(default_file);
    let src = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read source file '{}'", file_path))?;

    match run(&src) {
        Ok((_val, output)) => {
            for line in output {
                println!("{}", line);
            }
        }
        Err(e) => {
            eprintln!("✗ Error in '{}':\n  {}", file_path, e);
            std::process::exit(1);
        }
    }
    Ok(())
}

fn cmd_install(target_pkg: Option<&str>) -> Result<()> {
    let mut config = require_bls_file()?;
    let modules_dir = Path::new("modules");
    if !modules_dir.exists() {
        fs::create_dir_all(modules_dir)?;
    }

    if let Some(pkg) = target_pkg {
        println!("Installing package '{}'...", pkg);
        install_single_package(pkg, "latest", modules_dir)?;
        config.dependencies.insert(pkg.to_string(), "latest".to_string());
        let formatted = serde_json::to_string_pretty(&config)?;
        fs::write("app.bls", formatted)?;
        println!("✓ Added '{}' to app.bls dependencies.", pkg);
        return Ok(());
    }

    println!(
        "Resolving and installing {} package(s) for '{}':",
        config.dependencies.len(),
        config.name
    );

    for (dep, ver) in &config.dependencies {
        println!(" → Downloading '{}' ({}) into modules/...", dep, ver);
        install_single_package(dep, ver, modules_dir)?;
    }

    println!("✓ All dependencies resolved and installed in ./modules/");
    Ok(())
}

fn install_single_package(pkg_name: &str, version: &str, target_dir: &Path) -> Result<()> {
    let pkg_folder = target_dir.join(pkg_name);

    if pkg_name.starts_with("http://") || pkg_name.starts_with("https://") || pkg_name.ends_with(".git") {
        // Clone from Git repository directly
        if pkg_folder.exists() {
            println!("   Updating existing repository in {}", pkg_folder.display());
            let _ = std::process::Command::new("git")
                .args(&["pull"])
                .current_dir(&pkg_folder)
                .status();
        } else {
            let status = std::process::Command::new("git")
                .args(&["clone", "--depth", "1", pkg_name, pkg_folder.to_str().unwrap()])
                .status();
            if let Ok(s) = status {
                if s.success() {
                    println!("   ✓ Cloned {} successfully.", pkg_name);
                    return Ok(());
                }
            }
        }
    }

    if !pkg_folder.exists() {
        fs::create_dir_all(&pkg_folder)?;
    }

    let functions_folder = pkg_folder.join("functions");
    if !functions_folder.exists() {
        fs::create_dir_all(&functions_folder)?;
    }

    // 1. Create or download package.bls
    let package_bls_file = pkg_folder.join("package.bls");
    if !package_bls_file.exists() {
        let meta = serde_json::json!({
            "name": pkg_name,
            "version": version,
            "functions": ["get_version"]
        });
        fs::write(&package_bls_file, serde_json::to_string_pretty(&meta)?)?;
    }

    // 2. Create or download package.bl
    let package_bl_file = pkg_folder.join("package.bl");
    if !package_bl_file.exists() {
        let package_bl_content = format!(
            "// Blueprint Package: {}\n// Version: {}\n\n@blueprint import Blueprint\n\nfunc Version(): str [\n    return \"{}\"\n]\n",
            pkg_name, version, version
        );
        fs::write(&package_bl_file, package_bl_content)?;
    }

    // 3. Create or download functions/get_version.bl
    let get_version_file = functions_folder.join("get_version.bl");
    if !get_version_file.exists() {
        let get_version_content = format!(
            "func get_version(): str [\n    return \"{}\"\n]\n",
            version
        );
        fs::write(&get_version_file, get_version_content)?;
    }

    println!("   ✓ Initialized package structure for '{}' (package.bls, package.bl, functions/)", pkg_name);
    Ok(())
}

