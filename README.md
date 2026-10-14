# Printable (`printable`)

<p align="center">
  <img src="logo.png" alt="Printable Logo" width="140" height="140" />
</p>

The official CLI build tool and package manager for the **[Blueprint (`.bl`)](https://github.com/LukeScarletTrial/bl)** programming language.

---

## Downloads

You can download the pre-compiled executable directly:
- **Windows (x64)**: [`printable.exe`](printable.exe)

Place `printable.exe` into any folder in your system `PATH` to use it anywhere in your terminal.

---

## Features

- **`printable init`**: Generates a standard `app.bls` (Blueprint Setup) configuration file.
- **`printable build <file.bl>`**: Compiles `.bl` source files into native standalone `.exe` executables.
- **`printable run <file.bl>`**: Type-checks and executes a Blueprint program directly.
- **`printable install`**: Inspects and validates dependency declarations from `app.bls`.

---

## Usage

```bash
# Initialize a Blueprint project
printable init

# Compile to standalone native binary (emits bin/<name>.exe)
printable build main.bl

# Run your program
printable run main.bl

# Verify dependencies
printable install
```

---

## Building from Source

Prerequisites: Rust (Cargo) 1.80+

```bash
cargo build --release
```

The resulting binary will be in `target/release/printable.exe`.
