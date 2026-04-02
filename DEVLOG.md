# Melange Development Log

> Running log of what was built, why, and key decisions made along the way.
> This will become the foundation for a blog post once we hit stable release.

---

## Project Summary

**Melange** is a Dune-themed CLI/TUI tool that analyzes local LLM memory requirements on Apple Silicon. It scans your hardware and model files, does the math, and tells you: will it fit, how fast will it run, and when will you hit swap.

**Why it exists:** Running LLMs locally on Apple Silicon means memory is everything. A model that looks great on paper can drown in swap at real context lengths. We learned this the hard way when GLM-4.7-Flash at 6-bit dropped from 227 tok/s to 5 tok/s. There was no tool that answered the simple question: "will this model actually work on my machine?" So we built one.

**Tech stack:** Rust, Ratatui (TUI), Crossterm, Clap, Serde, TOML config.

---

## Development Timeline

### Phase 0: Initial Build (v0.1.0-beta)

The first working version. Core functionality:
- Hardware detection (CPU, unified memory, GPU cores, Metal version, bandwidth, disk)
- Model scanner (reads config.json + model.safetensors.index.json — never touches weight files)
- Memory calculator with KV cache growth at multiple context lengths
- Spice Status system (Abundant Spice / Spice Thinning / Spice Scarcity / Desert Drought)
- Token generation speed estimation based on memory bandwidth
- Full TUI dashboard with Dune theme
- JSON output mode for scripting
- Splash screen with rotating Dune quotes

**Key design decisions:**
- Read-only analysis. Never modify model files. Only parse JSON metadata.
- Apple Silicon only. Unified memory architecture makes the math clean. Linux/x86 would need split VRAM vs system RAM — different problem entirely.
- Dune theming is not just cosmetic. "Spice" as a metaphor for memory makes the status labels immediately intuitive to the audience.

### Phase 1: Shipping Overhaul

**Problem:** v0.1.0-beta was on GitHub but had two blocking issues:
1. Install required sudo (installed to `/usr/local/bin`)
2. Model directory was hardcoded to `~/AI_MODELS/models/` — wouldn't exist on anyone else's machine

**What we shipped:**

#### Config System (`src/config.rs`)
- TOML config at `~/.config/melange/config.toml` (CLI convention, not macOS `~/Library/Application Support/`)
- Decision: used `~/.config/` because Melange is a terminal tool and its users live in the terminal. Every comparable CLI tool does this.
- First-run interactive prompt that asks where your models are, validates the path, saves to config
- `melange config` subcommand to view settings

#### Multi-Directory Model Support
- Realized single-directory was a prototype shortcut that would become tech debt immediately
- Every local AI user has models in multiple places (HuggingFace cache, manual downloads, external drives)
- Config stores `model_dirs` array, scanner loops over all registered directories
- `melange add /path` — register a directory with scan feedback (shows what models it found)
- `melange dirs` — list all registered directories
- `melange remove /path` — unregister a directory
- Duplicate detection via canonical path comparison
- Non-model files are naturally filtered (scanner only picks up dirs with valid config.json)

**Resolution priority chain:**
1. `--scan` flag (highest — explicit CLI override, not saved)
2. Config file
3. Default `~/AI_MODELS/models/` if it exists (auto-saved to config)
4. First-run interactive prompt (only if nothing else works)

#### No-Sudo Installer (`install.sh`)
- Installs to `~/.melange/bin/` instead of `/usr/local/bin/`
- Auto-detects shell (zsh/bash) and appends PATH to rc file
- Warns about old `/usr/local/bin/melange` installs
- Falls back to source build if no pre-built binary
- `--jobs 4` flag on cargo build to prevent SIGKILL on some machines (discovered during testing — macOS kills build processes when too many run in parallel)

#### UI Cleanup
- Model list title changed from full file path to `LOCAL MODELS (3)` — cleaner, works with multi-dir
- Removed redundant model name in detail panel (was showing the name twice)

### Testing & Verification

Tested by cloning fresh from GitHub into `/tmp/`, building from source, and running full test suite:
- 18 tests covering: help, version, config, add, dirs, remove, duplicate detection, invalid paths, JSON output, --scan override, auto-detect, config persistence, install script syntax
- Discovered the `--jobs` build issue during testing (SIGKILL on parallel compilation) — fixed in install.sh and README

---

## Architecture Notes

```
src/
├── main.rs              # CLI parsing, model dir resolution, TUI launch
├── config.rs            # Config load/save, add/remove/dirs commands
├── app.rs               # App state, event handling, refresh
├── dune/                # Quotes, terminology
├── hardware/            # CPU, memory, GPU, bandwidth, disk detection
├── models/
│   ├── scanner.rs       # Directory walking, multi-dir support
│   ├── config_parser.rs # Reads model config.json
│   ├── index_parser.rs  # Reads model.safetensors.index.json
│   └── memory_calc.rs   # Memory estimation, KV cache, tok/s
└── ui/
    ├── splash.rs        # Intro screen
    ├── header.rs        # Title bar
    ├── hardware_panel.rs
    ├── models_panel.rs  # Model list table
    ├── detail_panel.rs  # Selected model analysis
    ├── memory_panel.rs  # Memory usage visualization
    ├── footer.rs        # Quotes + controls
    └── theme.rs         # Color palette
```

---

## Key Metrics

- **Binary size:** ~4 MB (release build)
- **Build time:** ~10-30 seconds depending on parallelism
- **Dependencies:** 87 crates (Rust ecosystem)
- **Startup:** Instant hardware detection + model scan, 1.5s splash screen

---

## Decisions Log

| Decision | Choice | Why |
|----------|--------|-----|
| Config path | `~/.config/melange/` | CLI convention > macOS convention for a terminal tool |
| Single vs multi-dir | Multi-dir from day one | Every user has models in multiple places |
| Add dirs one-at-a-time | Yes | Less error surface, clear feedback per directory |
| Install location | `~/.melange/bin/` | No sudo, follows opencode pattern |
| Build parallelism | `--jobs 4` | Prevents SIGKILL on some machines |
| Model list title | Count, not path | Shorter, works with multi-dir, saves horizontal space |

---

## What's Next

- [ ] "So what should I do" recommendations (e.g., "Best at 8K context", "Drop to 4-bit to fit")
- [ ] Pre-built binary releases (GitHub Actions CI)
- [ ] More model architecture support as new models release
- [ ] Community feedback from initial users

---

*Last updated: March 2026*
