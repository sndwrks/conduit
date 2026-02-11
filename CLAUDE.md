# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Conduit is a cross-platform desktop application that bridges OSC (Open Sound Control) and MIDI protocols. It provides a single-screen interface with a mapping table where each row defines an input→output translation between protocols.

**Stack:** Tauri 2 (Rust backend) + React 18+ + TypeScript + shadcn/ui + Tailwind CSS
**Brand:** sndwrks | **Bundle ID:** `com.sndwrks.conduit`

## Commands

```bash
# Development (hot-reload)
npm tauri dev

# Build for production
npm tauri build

# Frontend
npm install              # install dependencies
npm test                 # run frontend tests
npm lint                 # lint frontend
npm typecheck            # TypeScript type checking

# Rust backend (run from src-tauri/)
cargo test --workspace    # run all Rust tests
cargo test test_name      # run a single Rust test
```

**Prerequisites:** Rust toolchain, Node.js 18+, pnpm 9
**Linux deps:** `libgtk-3-dev libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf libasound2-dev`

## Architecture

```
src/                    # React frontend (TypeScript)
  components/           # shadcn/ui components
resources/              # Logos & Roboto Mono Variable font (bundled, sole typeface)
src-tauri/              # Rust backend
  src/
    main.rs             # Tauri app entry
    commands/           # IPC command handlers
    osc_engine.rs       # OSC listener (rosc + tokio UDP/TCP)
    midi_engine.rs      # MIDI I/O (midir)
    router.rs           # Message routing/matching engine
    config.rs           # Settings persistence (serde + JSON)
    models.rs           # Shared data types
```

### Backend (Rust) — Key crates

- **tauri v2** — app framework, IPC, window management
- **midir** — cross-platform MIDI I/O (ALSA on Linux, CoreMIDI on macOS, WinMM on Windows)
- **rosc** — OSC packet encoding/decoding
- **tokio** — async runtime for UDP/TCP sockets and channel-based routing
- **serde/serde_json** — config serialization
- **dirs** — platform config directory resolution

### Frontend ↔ Backend IPC

Frontend uses `invoke()` for commands and `listen()` for backend-pushed events. Key commands: `get_settings`, `update_settings`, `list_midi_inputs`, `list_midi_outputs`, `get_mappings`, `add_mapping`, `update_mapping`, `delete_mapping`, `start_engine`, `stop_engine`. Key events: `mapping-activity`, `unmatched-message`, `engine-status`, `midi-devices-changed`.

### Engine Design

The router runs on a dedicated Tokio task with an unbounded channel. Incoming messages (from OSC UDP/TCP listeners and MIDI callback) are pushed to the router channel. The router matches against all enabled mappings (exact string match for OSC addresses, type+channel+note/CC for MIDI). Multiple mappings can match the same input (fan-out). Activity events to frontend are rate-limited at 60/sec.

### Data Persistence

Config stored as JSON in platform-specific directories:
- macOS: `~/Library/Application Support/com.sndwrks.conduit/`
- Windows: `%APPDATA%\sndwrks\conduit\`
- Linux: `~/.config/sndwrks-conduit/`

Two files: `settings.json` (OSC/MIDI config) and `mappings.json` (mapping rows). Settings auto-save debounced at 500ms. Mappings save on every CRUD op via atomic write (temp file + rename).

## Key Design Decisions

- **Dark mode only** — no light mode toggle (musicians work in dark environments)
- **Single screen UI** — settings panel (top, collapsible), mapping table (center), activity log (bottom)
- **MVP message types:** MIDI Note On/Off and CC only; OSC with int32, float32, string args
- **Value scaling:** OSC float 0.0–1.0 ↔ MIDI 0–127 (linear, multiply/divide by 127)
- **MIDI note convention:** Middle C = C3 = MIDI note 60
- **OSC TCP framing:** SLIP (RFC 1055) — matches QLab and OSC 1.1 spec
- **TCP send timeout:** 3 seconds, not pooled — each outgoing message opens/closes connection

## CI/CD

- **CI** (`.github/workflows/ci.yml`): Rust tests on macOS/Windows/Linux; frontend tests, lint, typecheck on Ubuntu. Triggered on every push/PR.
- **Release** (`.github/workflows/release.yml`): Triggered by push to `release` branch. Reads version from `src-tauri/tauri.conf.json`. Builds for macOS arm64, macOS x64, Windows x64, Linux x64. Creates draft GitHub Release.

## Spec Reference

The full application specification is in `spec.md`. Consult it for detailed data models (Section 6), IPC command signatures (Section 5), error handling table (Section 9), and post-MVP roadmap (Section 12).
