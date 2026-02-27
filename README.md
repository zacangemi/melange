# Melange — "The Memory Must Flow"

A Dune-themed terminal tool that scans your hardware and local model files to tell you exactly what fits, how fast it'll run, and when you'll hit swap — so you never drown a model in memory again.

```
    ___
___/ o \____
   ___     \___________
  /   \___             \
           \___   ___  |
               \_/  \_|
```

## Why?

Running LLMs locally on Apple Silicon? Memory is everything. A model that looks perfect on paper can drown in swap at real prompt sizes (we learned this the hard way with GLM-4.7-Flash at 6-bit — 227 tok/s dropped to 5 tok/s).

**Melange scans your hardware, scans your model files, does the math, and shows you:**
- Will it fit in memory?
- How fast will it generate tokens?
- At what context length will it hit swap?
- KV cache growth at every context size

**No Ollama. No APIs. No cloud.** Pure local: scan hardware, scan model files on disk, do the math, show the results.

## Install

### Quick Install (macOS Apple Silicon)

```bash
curl -sSL https://raw.githubusercontent.com/zacangemi/melange/main/install.sh | sh
```

### Build from Source

Requires [Rust](https://rustup.rs/):

```bash
git clone https://github.com/zacangemi/melange.git
cd melange
cargo build --release
cp target/release/melange /usr/local/bin/
```

## Usage

```bash
# Launch the TUI dashboard
melange

# Scan a custom model directory
melange --scan /path/to/models

# Output as JSON (for scripting)
melange --json
```

### Controls

| Key | Action |
|-----|--------|
| `j` / `k` or arrows | Navigate models |
| `r` | Refresh hardware & models |
| `q` / `Esc` | Quit |

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

**Default model directory:** `~/AI_MODELS/models/`

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
