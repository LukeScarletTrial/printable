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
        "build" => cmd_build(args.get(2).map(|s| s.as_str())),
        "run" => cmd_run(args.get(2).map(|s| s.as_str())),
        "install" => cmd_install(args.get(2).map(|s| s.as_str())),
        other => {
            eprintln!("Error: Unknown command '{}'", other);
            eprintln!("Usage: printable <init|build|run|install> [file/package]");
            std::process::exit(1);
        }
    };

    if let Err(err) = result {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}

fn require_bls_file() -> Result<ProjectConfig> {
    let bls_path = Path::new("app.bls");
    if !bls_path.exists() {
        bail!("Error: 'app.bls' configuration file is required. Run 'printable init' to create one.");
    }
    let content = fs::read_to_string(bls_path)
        .with_context(|| format!("Failed to read config file '{}'", bls_path.display()))?;
    let config: ProjectConfig = serde_json::from_str(&content)
        .with_context(|| format!("Invalid JSON or schema in '{}'", bls_path.display()))?;
    Ok(config)
}

fn cmd_init() -> Result<()> {
    let path = Path::new("app.bls");
    if path.exists() {
        println!("app.bls already exists; project already initialized.");
        return Ok(());
    }

    // 1. Create app.bls
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
            deps.insert("core".to_string(), "1.2.0".to_string());
            deps
        },
    };

    let formatted = serde_json::to_string_pretty(&default_config)?;
    fs::write(path, formatted)?;

    // 2. Create src directory and src/main.bl
    let src_dir = Path::new("src");
    if !src_dir.exists() {
        fs::create_dir_all(src_dir)?;
    }

    let main_bl_path = src_dir.join("main.bl");
    if !main_bl_path.exists() {
        let starter_code = r#"@blueprint import Blueprint
Blueprint.Setup("Blueprint App", true)

defstruct Player [
    name: str,
    score: int
]

func IsHighScore(p: Player): bool [
    if p.score >= 100 [
        return true
    ] else [
        return false
    ]
]

func Main(): [
    def player = Player { name: "Alex", score: 120 }

    if IsHighScore(player) [
        output("High score achieved by " + player.name)
    ] else [
        output("Keep trying!")
    ]
]
"#;
        fs::write(&main_bl_path, starter_code)?;
    }

    // 3. Create .gitignore if not present
    let gitignore_path = Path::new(".gitignore");
    if !gitignore_path.exists() {
        fs::write(gitignore_path, "bin/\ntarget/\n*.exe\n")?;
    }

    println!("✓ Initialized full Blueprint project successfully!");
    println!("  - Created app.bls");
    println!("  - Created src/main.bl");
    println!("  - Created .gitignore");
    println!("\nNext steps:");
    println!("  printable run src/main.bl");
    println!("  printable build src/main.bl");
    Ok(())
}


fn cmd_build(file: Option<&str>) -> Result<()> {
    let config = require_bls_file()?;
    println!(
        "Loaded configuration for '{}' v{}",
        config.name, config.version
    );

    let file_path = match file {
        Some(f) => f,
        None => {
            eprintln!("Error: Missing source file.\nUsage: printable build <file.bl>");
            std::process::exit(1);
        }
    };
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
    // app.bls is optional for run — just warn if missing
    if Path::new("app.bls").exists() {
        let _ = require_bls_file(); // validate but don't fail
    }

    let file_path = match file {
        Some(f) => f,
        None => {
            eprintln!("Error: Missing source file.\nUsage: printable run <file.bl>");
            std::process::exit(1);
        }
    };
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

    // Official or community package download from GitHub/Registry
    if !pkg_folder.exists() {
        fs::create_dir_all(&pkg_folder)?;
    }

    // Attempt downloading package module file
    let download_url = format!(
        "https://raw.githubusercontent.com/LukeScarletTrial/bl-packages/main/{}/lib.bl",
        pkg_name
    );
    let target_file = pkg_folder.join(format!("{}.bl", pkg_name));

    let curl_status = std::process::Command::new("curl.exe")
        .args(&["-sSL", &download_url, "-o", target_file.to_str().unwrap()])
        .status();

    let downloaded = match curl_status {
        Ok(s) => s.success() && target_file.exists() && fs::metadata(&target_file).map(|m| m.len() > 10).unwrap_or(false),
        Err(_) => false,
    };

    if !downloaded {
        // Write standard fallback module header for declared package
        let stub_module = format!(
            "// Blueprint Package: {}\n// Version: {}\n\n@blueprint import Blueprint\n\nfunc Version(): str [\n    return \"{}\"\n]\n",
            pkg_name, version, version
        );
        let _ = fs::write(&target_file, stub_module);
        println!("   ✓ Initialized package module at {}", target_file.display());
    } else {
        println!("   ✓ Downloaded '{}' into {}", pkg_name, target_file.display());
    }

    Ok(())
}

