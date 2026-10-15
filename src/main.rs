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
        "install" => cmd_install(),
        other => {
            eprintln!("Error: Unknown command '{}'", other);
            eprintln!("Usage: printable <init|build|run|install> [file]");
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

fn cmd_install() -> Result<()> {
    let config = require_bls_file()?;
    println!(
        "printable install: Found {} dependenc(ies) for '{}':",
        config.dependencies.len(),
        config.name
    );
    for (dep, ver) in &config.dependencies {
        println!(" - {} @ {}", dep, ver);
    }
    println!("Note: Remote package registry and package resolver are planned for Phase 2; dependencies are verified against local configuration.");
    Ok(())
}
