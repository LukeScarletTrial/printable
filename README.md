# Printable (`printable`)

<p align="center">
  <img src="logo.png" alt="Printable Logo" width="140" height="140" />
</p>

<h3 align="center">The official CLI build tool and package manager for the <a href="https://github.com/LukeScarletTrial/bl">Blueprint (.bl)</a> programming language.</h3>

---

## Downloads

Download the pre-compiled binary directly — no Rust or build tools required:

| Platform | Binary |
|----------|--------|
| Windows (x64) | [`printable.exe`](printable.exe) |

Place `printable.exe` anywhere in your system `PATH` and you're ready to go.

---

## Features

| Command | Description |
|---------|-------------|
| `printable init` | Creates an `app.bls` project configuration file |
| `printable build <file.bl>` | Compiles `.bl` source to a standalone native `.exe` in `bin/` |
| `printable run <file.bl>` | Runs a Blueprint program directly |
| `printable install` | Validates dependency declarations in `app.bls` |

---

## Quick Start

```bash
# 1. Initialize your Blueprint project
printable init

# 2. Create your source file (main.bl)
#    ... write your Blueprint code ...

# 3. Build to a native executable
printable build main.bl
# -> bin/main.exe created!

# 4. Or run it directly
printable run main.bl
```

---

## Building from Source

Prerequisites: Rust (Cargo) 1.80+

```bash
cargo build --release
```

The resulting binary will be at `target/release/printable.exe`.

---

## Blueprint Language

Blueprint source files (`.bl`) use square brackets `[...]` for code blocks:

```blueprint
@blueprint import Blueprint
Blueprint.Setup("My App", true)

func Main(): [
    output("Hello, Blueprint!")
]
```

→ See the full language at [LukeScarletTrial/bl](https://github.com/LukeScarletTrial/bl)
