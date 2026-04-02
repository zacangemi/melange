# Melange — "The Memory Must Flow"

A Dune-themed terminal tool that scans your Apple Silicon hardware and local model files to tell you exactly what fits, how fast it'll run, and when you'll hit swap.

```
                          .  '  .
            \  ' .  /    ' .  . '   .  '                                        ___
         --  \  |  / --      '                                                 / _ \
        '  ---\|//---  .        .                                             | / \ |
          -====(●)====-                        .          .                    |  _  |
        .  ---/|\---  '     .          ___---~~~~---___                        \ \_/ |
         --  /  |  \ --          ___--~~  //||||\\  ~~--___                     \___/
            /  ' .  \     ___--~~   //  ||||||||  \\   ~~--___                   |
          '    .    ___--~~ ///  ||||  ||||||||  ||||  \\\ ~~--___              /
           ___..--~~ // ||||||||  ||||   ||||||   ||||  |||||||| \\ ~~--..__/
     __--~~  ///  |||| ||||||||  ||||    ||||    ||||  |||||||| ||||  \\\  ~~--
   ~~  //  ||||||  |||| ||||||||  ||||    ||||    ||||  |||||||| ||||  ||||||  \\
    \\  ||||  ||||  ||||  ||||||  ||||    ||||    ||||  ||||||  ||||  ||||  //
      \\  ||||  ||||  \\  ||||  ||||    ||||    ||||  ||||  //  ||||  //
  ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
  ~~.~~~  ~~~~.~~~~  ~~~~.~~~~  ~~~~.~~~~  ~~~~.~~~~  ~~~~.~~~~  ~~~~.~~~~  ~~~~.~~~  ~~
      .  i i .    i i .    i i .    i i      i i .    i i .    i i .    i i .
```

## Why?

Running LLMs locally on Apple Silicon? Memory is everything. A model that looks perfect on paper can drown in swap at real prompt sizes.

Melange scans your hardware, reads your model metadata, does the math, and shows you:
- Will it fit in memory?
- How fast will it generate tokens? (both prefill and decode speed)
- At what context length will it hit swap?
- KV cache growth at every context size (with per-step memory deltas)
- Known engine bugs for your specific models

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
cargo build --release --jobs 4
cp target/release/melange ~/.melange/bin/
```

> **Note:** The `--jobs 4` flag prevents build failures on some machines where full parallelism causes processes to get killed by macOS. You can omit it if your build succeeds without it.

## First Run

The first time you run `melange`, it will ask where your models live:

```
  Welcome to Melange — "The memory must flow"

  First-time setup: I need to know where your models live.

  Model directory path: ~/AI_MODELS/models

  Found 5 models:
    Qwen3-30B-A3B (30.5B params, 4-bit)
    GLM-4-9B (9.4B params, 6-bit)
    ...

  Saved to ~/.config/melange/config.toml
  Add more directories later with `melange add /path`.
```

If `~/AI_MODELS/models/` already exists, Melange detects it automatically and skips the prompt.

## Usage

```bash
melange                      # Launch the TUI dashboard
melange add ~/more/models    # Register another model directory
melange dirs                 # List all registered directories
melange remove ~/old/path    # Unregister a directory
melange config               # Show configuration
melange --scan /one-off      # Override for this run (not saved)
melange --json               # Output as JSON (for scripting)
```

### Multiple Model Directories

Most people have models in more than one place. Melange supports this natively:

```bash
melange add ~/AI_MODELS/models
melange add ~/.cache/huggingface/hub
melange add /Volumes/external/models
```

All registered directories are scanned every time you launch the TUI. The panel title shows the directory count when you have more than one.

### Controls

| Key | Action |
|-----|--------|
| `j` / `k` or arrows | Navigate models |
| `Tab` | Switch between Local / Catalog tabs |
| `r` | Refresh hardware & models |
| `v` | Toggle VPN info |
| `w` | Engine warning detail |
| `?` | Show help overlay |
| `q` / `Esc` | Quit |

## Dashboard

The TUI has two tabs:

- **Local** — Scans your model directories and analyzes every model against your real hardware. Models are sorted by fit status (Fits first, OOM last) so you see what you can run at a glance.
- **Catalog** — Pre-configured reference models (Llama 3.3, DeepSeek R1, Mixtral, etc.) analyzed against your hardware, even if you haven't downloaded them yet. Useful for deciding what to download.

Both tabs show the same analysis: speed estimates, KV cache growth, memory breakdown, fit status, and engine warnings.

## Configuration

Config file: `~/.config/melange/config.toml`

```toml
# Melange configuration
# Manage directories with: melange add, melange dirs, melange remove

model_dirs = [
    "/Users/you/AI_MODELS/models",
    "/Users/you/.cache/huggingface/hub",
]
```

Model directory resolution order:
1. `--scan` flag (highest priority — one-time override, not saved)
2. Config file (`~/.config/melange/config.toml`)
3. Default `~/AI_MODELS/models/` if it exists (auto-saved to config)
4. First-run interactive prompt (only if nothing else works)

## What It Scans

**Hardware:**
- CPU (brand, P/E core split)
- Unified memory (total, used, available, top memory consumers)
- GPU cores + Metal version
- Memory bandwidth (for tok/s estimation)
- Disk space
- Installed inference engines (llama.cpp, MLX, Ollama, vLLM, ExLlamaV2)
- VPN status (Tailscale)

**Models** (reads JSON metadata only — never touches weight files):
- `config.json` — architecture, layers, attention heads, MoE experts, quantization
- `model.safetensors.index.json` — exact parameter count and byte size

Non-model files in your directories are ignored. Melange only picks up subdirectories containing valid model metadata.

## Fit Status

| Status | Meaning |
|--------|---------|
| **✓ Fits** | > 4 GB headroom, runs great |
| **△ Tight** | 1-4 GB headroom, watch it |
| **△ Limited** | Tight fit, limited context |
| **✗ OOM** | Will hit swap — don't run this |

## Engine Warnings

Melange ships a built-in compatibility knowledge base of 50+ documented bugs across llama.cpp, Ollama, MLX, vLLM, and ExLlamaV2. Warnings are matched per-model based on architecture, family, and which engines you have installed.

**Three layers of progressive disclosure:**

1. **Badge in model table** — A severity icon + count appears in the `Warn` column next to models with known issues. No badge = clean model.
2. **Inline in detail panel** — The top 3 warnings show in the detail panel with a hint to press `w` for more.
3. **Full overlay on `w`** — A popup with severity labels, summaries, workarounds, version fix info, and detail text for every matched warning.

Each warning includes:
- **Severity** — Breaking (model won't work) or Caution (works with caveats)
- **Summary** — One-line description of the issue
- **Detail** — Full explanation with context
- **Workaround** — Flags or settings to fix it (when available)
- **Fixed in** — Engine version where the bug was resolved

You can add your own warnings or override built-in ones by creating `~/.config/melange/compat_warnings.toml`.

## Requirements

- macOS with Apple Silicon (M1/M2/M3/M4)
- Model files in safetensors format with `config.json`

## License

MIT
