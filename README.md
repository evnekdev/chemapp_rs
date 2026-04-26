# chemapp-rs

> A Rust interface for chemical thermodynamics calculations using ChemApp, providing safe and ergonomic access to native thermochemical equilibrium routines.

![Rust](https://img.shields.io/badge/language-Rust-orange)
![Status](https://img.shields.io/badge/status-active-green)

---

## 📌 Overview

**chemapp-rs** is a Rust library that wraps and interfaces with **ChemApp**, a widely used thermochemical calculation engine (provided by GTT Technologies).

This crate provides:

- Safe Rust abstractions over native ChemApp bindings
- Structured error handling
- High-level calculation utilities
- Modular design for extensibility

It is designed for:

- Thermodynamic equilibrium calculations
- Chemical process simulations
- Scientific computing workflows

---

## ✨ Features

- 🔬 Interface to ChemApp native library
- 🧮 Thermodynamic calculation utilities
- ⚙️ Safe wrapper around unsafe FFI calls
- 🧱 Modular architecture
- ❗ Robust error handling
- 🔗 Interaction modeling support

---

## 🛠️ Tech Stack

- Language: **Rust**
- FFI: Native bindings (ChemApp)
- Architecture: Modular Rust crate

---

## 📂 Project Structure

```
src/
├── lib.rs           # Library entry point
├── main.rs          # Example / CLI entry
├── calculator.rs    # Core calculation logic
├── interactions.rs  # Interaction modeling
├── native.rs        # FFI bindings to ChemApp
├── defs.rs          # Constants and definitions
├── error.rs         # Error handling
```

---

## 🚀 Getting Started

### 1. Clone the repository

```bash
git clone https://github.com/evnekdev/chemapp-rs.git
cd chemapp-rs
```

### 2. Build

```bash
cargo build
```

### 3. Run example

```bash
cargo run
```

---

## ⚠️ Requirements

This crate depends on **ChemApp**, which is:

- Proprietary software
- Must be installed separately
- Requires proper licensing

Ensure:

- ChemApp libraries are installed
- Environment variables / linker paths are configured correctly

---

## 📊 Usage

Example (conceptual):

```rust
use chemapp_rs::calculator::Calculator;

fn main() {
    let mut calculator = Calculator::from_library(r"ca_vc_e_local.dll", r"c:\_WORK\Continuous\diagrams\007_Al2O3-SiO2\BETA_Al-Si-O.dat").unwrap();
	calculator.set_transform(&["SiO2","Al2O3"]);
	calculator.calculate_isothermal_d(&dvector![0.1, 0.9], 2200.0);
	for idx in (0..100).components_valid(&calculator).components_names(&calculator) {
		println!("idx = {:?}", &idx);
	}
	for idx in (0..100).phases_valid(&calculator).phases_status_eliminated(&calculator).phases_names(&calculator) {
		println!("idx = {:?}", &idx);
	}
	
	for idx in (0..100).phases_valid(&calculator).phases_constituents(&calculator).constituents_hm(&calculator){
		println!("idx = {:?}", &idx);
	}
}
```

---

## 🧩 Core Modules

### `native.rs`
- Low-level FFI bindings
- Unsafe interface to ChemApp

---

### `calculator.rs`
- High-level API
- Encapsulates computation workflows

---

### `interactions.rs`
- Models interactions between components

---

### `error.rs`
- Custom error types
- Safe error propagation

---

### `defs.rs`
- Constants and shared definitions

---

## 🧪 Testing

```bash
cargo test
```

---

## ⚠️ Limitations

- Requires external ChemApp installation
- Limited by ChemApp API capabilities
- Platform-specific linking considerations

---

## 📈 Roadmap

- [ ] Improve documentation
- [ ] Add more high-level APIs
- [ ] Expand examples
- [ ] Improve error reporting
- [ ] Cross-platform support enhancements

---

## 🤝 Contributing

Contributions are welcome!

1. Fork the repository
2. Create a branch
3. Commit changes
4. Open a Pull Request

---

## 🐛 Issues

Report bugs or request features:

https://github.com/evnekdev/chemapp-rs/issues

---

## 📄 License

MIT

---

## 📬 Contact

**Evgenii Nekhoroshev**  
https://github.com/evnekdev
