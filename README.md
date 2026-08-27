# mkproject

`mkproject` is a CLI tool written in Rust designed to automate project scaffolding. It places new projects into structured workspace subdirectories using configurable templates, default authors, and automatic license handling.

## Features

- 📁 **Workspace Routing**: Automatically routes projects into subdirectories (e.g., `apps/`, `tools/`) based on `config.toml`.
- 🚀 **Template Integration**: Wraps `cargo-generate` to apply default or per-type templates.
- 📜 **License Automation**: Generates license files (`MIT`, `Apache-2.0`, `Unlicense`, or custom) and cleans up `Cargo.toml` automatically.
- 👤 **Author Auto-detection**: Falls back to global `git config user.name` if no authors are specified.
- 🏠 **Path Expansion**: Supports `~` home directory resolution across configuration and CLI flags.

## Prerequisites

- [Rust & Cargo](https://rustup.rs/) (edition 2024)
- [`cargo-generate`](https://github.com/cargo-generate/cargo-generate) installed on your system:
  ```bash
  cargo install cargo-generate
  ```

## Installation

### From Source

```bash
git clone https://github.com/vincentdesiree/mkproject.git
cd mkproject
cargo install --path .
```

This installs `mkproject` directly into `~/.cargo/bin`.

## Configuration

On first run, `mkproject` automatically creates a default `config.toml` at the standard OS configuration path:

- **Linux**: `~/.config/mkproject/config.toml`
- **macOS**: `~/Library/Application Support/mkproject/config.toml` (or `~/.config/mkproject/config.toml`)
- **Windows**: `%APPDATA%\mkproject\config.toml`

### Example Configuration

```toml
workspace_root = "~/dev/workspace"
default_template = "https://github.com/your-username/rust_project_template.git"

default_authors = "Your Name"
default_license = "MIT"

[types.app]
path = "apps"
template = "https://github.com/your-username/rust_app_template.git"

[types.tool]
path = "tools"
```

## Usage

```bash
# Basic creation in workspace root (~/dev/workspace/my_app)
mkproject my_app

# Using positional project type (~/dev/workspace/apps/my_app)
mkproject my_app app

# Equivalent syntax using combined 'type/name'
mkproject app/my_app

# License, author, and path overrides
mkproject my_tool tool -l "MIT OR Apache-2.0" -a "Alice <alice@example.com>"

# Custom licence override (same syntax if you want to put it as default in config file)
mkproject my_tool tool -l "custom_licence_name:path/to/licence"
```

### CLI Reference
```text
Usage: mkproject [OPTIONS] <NAME> [PROJECT_TYPE]

Arguments:
  <NAME>          Project name (e.g., 'my_app') or combined 'type/name' (e.g., 'app/my_app')
  [PROJECT_TYPE]  Optional project type defined in config (e.g., app, lib, tool)

Options:
  -t, --template <TEMPLATE>  Override template repository URL
  -p, --path <PATH>          Override target destination path
  -a, --authors <AUTHORS>    Author(s) name(s)
  -l, --license <LICENSE>    License SPDX or custom (ex: "MIT", "Apache-2.0", "MIT OR Apache-2.0", "Unlicense", "license_name:path_to_license_file")
  -h, --help                 Print help
  -V, --version              Print version
```

## License

This project is licensed under the [MIT License](LICENSE-MIT) OR [Apache-2.0](LICENSE-APACHE).
