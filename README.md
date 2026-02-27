# Melange — "The Memory Must Flow"

A Dune-themed terminal tool that scans your Apple Silicon hardware and local model files to tell you exactly what fits, how fast it'll run, and when you'll hit swap.

```
    ___
___/ o \____
   ___     \___________
  /   \___             \
           \___   ___  |
               \_/  \_|
```

## Why?

Running LLMs locally on Apple Silicon? Memory is everything. A model that looks perfect on paper can drown in swap at real prompt sizes.

Melange scans your hardware, reads your model metadata, does the math, and shows you:
- Will it fit in memory?
- How fast will it generate tokens?
- At what context length will it hit swap?
- KV cache growth at every context size

No Ollama. No APIs. No cloud. Pure local analysis.

## Install

### One-liner (macOS Apple Silicon)

```bash
curl -sSL https://raw.githubusercontent.com/zacangemi/melange/main/install.sh | sh
```

This installs to `~/.melange/bin/` and updates your shell PATH. No sudo required.

### Build from Source

Requires [Rust](https://rustup.rs/):

```bash
git clone https://github.com/zacangemi/melange.git
cd melange
cargo build --release
cp target/release/melange ~/.melange/bin/
```

## First Run

The first time you run `melange`, it will ask where your models live:

```
  Welcome to Melange — "The memory must flow"

  First-time setup: I need to know where your models live.

  Model directory path: ~/AI_MODELS/models

  Saved to ~/.config/melange/config.toml
```

Your choice is saved and used for all future runs. If `~/AI_MODELS/models/` already exists on your machine, Melange will detect it automatically and skip the prompt.

## Usage

```bash
melange                  # Launch the TUI dashboard
melange config           # Show or change configuration
melange --scan /path     # Override model directory for this run
melange --json           # Output as JSON (for scripting)
```

### Controls

| Key | Action |
|-----|--------|
| `j` / `k` or arrows | Navigate models |
| `r` | Refresh hardware & models |
| `q` / `Esc` | Quit |

## Configuration

Config file: `~/.config/melange/config.toml`

```toml
# Melange configuration
# Run `melange config` to change these settings

model_dir = "/Users/you/AI_MODELS/models"
```

Model directory resolution order:
1. `--scan` flag (highest priority — explicit CLI override)
2. Config file (`~/.config/melange/config.toml`)
3. Default `~/AI_MODELS/models/` if it exists (auto-saved to config)
4. First-run interactive prompt (only if nothing else works)

Run `melange config` at any time to view or change your settings.

## What It Scans

**Hardware:**
- CPU (brand, P/E core split)
- Unified memory (total, used, available)
- GPU cores + Metal version
- Memory bandwidth (for tok/s estimation)
- Disk space

**Models** (reads JSON metadata only — never touches weight files):
- `config.json` — architecture, layers, attention heads, MoE experts, quantization
- `model.safetensors.index.json` — exact parameter count and byte size

## Spice Status

| Status | Meaning |
|--------|---------|
| **Abundant Spice** | > 4 GB headroom, runs great |
| **Spice Thinning** | 1-4 GB headroom, watch it |
| **Spice Scarcity** | Tight fit, limited context |
| **Desert Drought** | Will hit swap — don't run this |

## Requirements

- macOS with Apple Silicon (M1/M2/M3/M4)
- Model files in safetensors format with `config.json`

## License

MIT
