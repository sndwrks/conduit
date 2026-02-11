# Conduit — OSC ↔ MIDI Bridge Application Specification

**Version:** 0.1.0 (MVP)
**Stack:** Tauri 2 (Rust backend) + React + TypeScript + shadcn/ui
**Platforms:** macOS, Windows, Linux
**Brand:** sndwrks
**Name:** *Conduit*

---

## 1. Problem Statement

Musicians, lighting designers, and creative technologists frequently need to translate between OSC (Open Sound Control) and MIDI protocols. Existing solutions like OSCulator are macOS-only, expensive, and overly complex for the core use case. There is no lightweight, cross-platform, open tool that lets users define simple bidirectional mappings between OSC addresses and MIDI messages and just run them.

Conduit solves this with a single-screen interface: a table of mapping rows where each row defines an input → output translation, with persistent settings for MIDI device selection and OSC network configuration.

---

## 2. Core Concepts

### 2.1 Mapping Row

A mapping row is the fundamental unit. Each row defines:

- **Direction:** `OSC → MIDI` or `MIDI → OSC`
- **Input pattern:** The trigger (an OSC address or a MIDI message descriptor)
- **Output definition:** What to emit when the input is matched
- **Enabled toggle:** Per-row on/off

Example mappings:

| Direction | Input | Output |
|-----------|-------|--------|
| OSC → MIDI | `/controller/fader1` (float arg) | CC 7, Channel 1, value = arg × 127 |
| OSC → MIDI | `/cue/fire` | Note On C3, Velocity 127, Channel 1 |
| MIDI → OSC | Note On C3, Ch 1 | `/eos/key/go_0` (no args) |
| MIDI → OSC | CC 1, Ch 1 | `/mix/volume` (float, value ÷ 127) |

### 2.2 Message Types Supported (MVP)

**MIDI messages:**

- Note On / Note Off (note number, velocity, channel)
- Control Change (CC number, value, channel)

**OSC messages:**

- Address pattern with 0–N typed arguments (int32, float32, string)

### 2.3 Value Mapping (MVP)

For mappings where a continuous value must cross protocols:

- **OSC float (0.0–1.0) → MIDI value (0–127):** Multiply by 127, round to nearest int
- **MIDI value (0–127) → OSC float:** Divide by 127
- **Static values:** User can hardcode any field (e.g., velocity is always 127)
- **Passthrough argument:** A single OSC argument maps to a single MIDI field, or vice versa

### 2.4 Future: Wildcard / Argument Binding (Post-MVP)

The mapping engine is designed to support wildcard patterns in a future release:

```
/osc/address {arg0}  →  MIDI Note C3, Velocity {arg0}
MIDI Note {note}, Ch 1  →  /instrument/note {note} {velocity}
```

The `{argN}` or `{fieldName}` syntax will allow any OSC argument or MIDI field to be referenced by name and routed to the output. The MVP data model should store mappings in a way that makes adding this non-breaking (see Section 7).

---

## 3. Architecture

```
┌─────────────────────────────────────────────────────┐
│                   React + shadcn/ui                  │
│  ┌───────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │  Settings  │  │ Mapping Table│  │ Activity Log │  │
│  │   Panel    │  │  (main view) │  │  (footer)    │  │
│  └───────────┘  └──────────────┘  └──────────────┘  │
└──────────────────────┬──────────────────────────────┘
                       │ Tauri IPC (invoke / events)
┌──────────────────────┴──────────────────────────────┐
│                   Rust Backend                       │
│  ┌────────────┐  ┌────────────┐  ┌───────────────┐  │
│  │ OSC Engine │  │ MIDI Engine│  │ Mapping Engine │  │
│  │  (rosc +   │  │  (midir)   │  │  (router +    │  │
│  │  tokio UDP │  │            │  │   transforms) │  │
│  │  + TCP)    │  │            │  │               │  │
│  └────────────┘  └────────────┘  └───────────────┘  │
│  ┌────────────────────────────────────────────────┐  │
│  │         Config / Persistence (serde + JSON)     │  │
│  └────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────┘
```

### 3.1 Rust Crates

| Crate | Purpose |
|-------|---------|
| `tauri` (v2) | App framework, IPC, window management |
| `midir` (v0.9+) | Cross-platform MIDI I/O. Enumerate ports, open connections, send/receive raw MIDI bytes |
| `rosc` (v0.11+) | OSC packet encoding/decoding (pure Rust, no dependencies) |
| `tokio` | Async runtime for UDP socket (OSC) and channel-based message routing |
| `serde` / `serde_json` | Config serialization |
| `dirs` | Locate platform-appropriate config directory |

### 3.2 Frontend

| Library | Purpose |
|---------|---------|
| React 18+ | UI framework |
| TypeScript | Type safety |
| shadcn/ui | Component library (Button, Select, Input, Switch, Table, Dialog, Toast) |
| Tailwind CSS | Styling (bundled with shadcn) |
| `@tauri-apps/api` | IPC invoke calls and event listeners |
| Roboto Mono Variable | Sole typeface — `.woff2` file provided, bundled in `src/assets/fonts/`, loaded via `@font-face` in global CSS. Not fetched externally. |

---

## 4. User Interface

The entire application lives on a single screen with three zones: a top settings bar, a central mapping table, and a bottom activity log.

### 4.1 Settings Bar (collapsible panel at top)

Accessed via a gear icon or "Settings" button. Uses a shadcn `Sheet` (slide-over) or collapsible `Collapsible` component.

**OSC Settings:**

- **Listen Port** — numeric input, default `8000`. The port where the app receives incoming OSC messages.
- **Listen Protocol** — `Select`: "UDP", "TCP", or "Both". Default `UDP`. When "TCP" or "Both" is selected, the app accepts TCP connections on the listen port. When "UDP" or "Both", it binds a UDP socket.
- **Send Host** — text input, default `127.0.0.1`. Target IP for outgoing OSC messages.
- **Send Port** — numeric input, default `9000`. Target port for outgoing OSC.
- **Send Protocol** — `Select`: "UDP" or "TCP". Default `UDP`. Determines transport for outgoing OSC messages.
- **TCP Send Timeout** — displayed when Send Protocol is "TCP". Fixed at 3 seconds. If a TCP connection cannot be established or the send does not complete within 3 seconds, the connection is dropped and an error is logged to the activity log.

**MIDI Settings:**

- **Input Device** — `Select` dropdown populated by `midir` enumeration. A "Refresh" button re-scans ports. The user picks which MIDI device to listen on.
- **Output Device** — `Select` dropdown, same enumeration. The user picks which device to send MIDI to.
- On macOS/Linux, virtual MIDI ports created by the app itself should appear as an option (via `midir`'s virtual port support).

**Global Controls:**

- **Start / Stop** toggle button — arms or disarms all mappings. When stopped, no messages are processed. Visual indicator (green dot / red dot).

All settings persist to disk automatically on change (debounced 500ms).

### 4.2 Mapping Table (center, main area)

A vertically scrollable list of mapping rows. Each row is a horizontal strip containing:

```
[ ✓ ] [ OSC → MIDI ▾ ] [ /cue/go         ] [ → ] [ Note On  C3  Vel 127  Ch 1 ] [ ╳ ]
  │         │                  │                            │                       │
enabled  direction        input config               output config              delete
```

**Row fields:**

1. **Enabled** — `Switch` toggle. Disabled rows are visually dimmed.
2. **Direction** — `Select`: "OSC → MIDI" or "MIDI → OSC"
3. **Input definition** — Changes based on direction:
   - *OSC input:* A text `Input` for the OSC address pattern (e.g., `/fader/1`)
   - *MIDI input:* A row of `Select` components: Message Type (Note On, Note Off, CC), Note/CC Number (dropdown or numeric input with note-name label like "C3"), Channel (1–16)
4. **Output definition** — Changes based on direction:
   - *MIDI output:* Message Type, Note/CC Number, Value source (`Select`: "Static" with a numeric input, or "From OSC Arg 0"), Channel
   - *OSC output:* Address `Input`, argument definition (type `Select` + value source)
5. **Delete** — `Button` with trash icon, confirms via `AlertDialog`

**Bottom of table:**

- **"+ Add Mapping"** button — appends a new row with sensible defaults (OSC → MIDI, empty address, Note On C3 Vel 127 Ch 1)

**Row interaction:**

- Rows are reorderable via drag handle (grip dots on the left) — order is cosmetic only, all mappings evaluate in parallel.
- Each row auto-saves on field change (debounced).

### 4.3 Activity Log (bottom strip, ~3 lines tall, expandable)

A real-time scrolling log of processed messages. Each entry shows:

```
12:04:33.221  ← OSC  /cue/go ()          → MIDI  Note On C3 Vel 127 Ch 1
12:04:33.890  ← MIDI CC 7 Val 64 Ch 1    → OSC   /mix/volume 0.504
```

- Color-coded: OSC messages in blue/purple, MIDI in green
- Toggle to pause/resume log
- "Clear" button
- Expandable to show more lines (shadcn `Collapsible` or resizable panel)
- Unmatched incoming messages shown in dimmed gray (helps debugging)

### 4.4 Visual Design Notes

- Dark mode only (musicians work in dark environments, no light mode toggle)
- Roboto Mono Variable is the sole typeface for the entire app (UI labels, inputs, log, OSC addresses, MIDI data). The variable font file will be provided and bundled with the app — loaded via `@font-face` in the frontend CSS, not fetched from Google Fonts.
- Compact row height — aim for 10+ visible rows without scrolling on a 1080p display
- Activity log uses a smaller weight/size of Roboto Mono Variable
- Minimal chrome: no menu bar clutter, the app is essentially one screen

---

## 5. Backend Commands (Tauri IPC)

All commands are invoked from the frontend via `invoke()`. Events use Tauri's event system for backend → frontend pushes.

### 5.1 Commands (Frontend → Backend)

```typescript
// Settings
invoke('get_settings') → Settings
invoke('update_settings', { settings: Settings }) → void

// MIDI device enumeration
invoke('list_midi_inputs') → MidiPort[]
invoke('list_midi_outputs') → MidiPort[]

// Mappings CRUD
invoke('get_mappings') → Mapping[]
invoke('add_mapping', { mapping: Mapping }) → string  // returns new ID
invoke('update_mapping', { mapping: Mapping }) → void
invoke('delete_mapping', { id: string }) → void
invoke('reorder_mappings', { ids: string[] }) → void

// Engine control
invoke('start_engine') → void
invoke('stop_engine') → void
invoke('get_engine_status') → { running: boolean }
```

### 5.2 Events (Backend → Frontend)

```typescript
// Real-time activity
listen('mapping-activity', (event: {
  timestamp: string,
  input_protocol: 'osc' | 'midi',
  input_display: string,
  output_protocol: 'osc' | 'midi',
  output_display: string,
  mapping_id: string
}) => void)

// Unmatched incoming message (for debugging)
listen('unmatched-message', (event: {
  timestamp: string,
  protocol: 'osc' | 'midi',
  display: string
}) => void)

// Engine status change
listen('engine-status', (event: { running: boolean, error?: string }) => void)

// MIDI device change (hot-plug)
listen('midi-devices-changed', () => void)
```

---

## 6. Data Models

### 6.1 Settings (persisted as `settings.json`)

```json
{
  "osc_listen_port": 8000,
  "osc_listen_protocol": "udp",
  "osc_send_host": "127.0.0.1",
  "osc_send_port": 9000,
  "osc_send_protocol": "udp",
  "osc_tcp_send_timeout_ms": 3000,
  "midi_input_port_name": "IAC Driver Bus 1",
  "midi_output_port_name": "IAC Driver Bus 1",
  "engine_auto_start": false
}
```

`osc_listen_protocol` accepts `"udp"`, `"tcp"`, or `"both"`. `osc_send_protocol` accepts `"udp"` or `"tcp"`. `osc_tcp_send_timeout_ms` is fixed at 3000 and not user-configurable in MVP.

MIDI ports are stored by name string. On startup, the backend attempts to find a port matching the saved name. If not found, the user is prompted to reselect.

### 6.2 Mapping (persisted as `mappings.json`)

```json
{
  "id": "a1b2c3d4",
  "enabled": true,
  "direction": "osc_to_midi",
  "osc_address": "/cue/go",
  "osc_arg_types": [],
  "midi_message_type": "note_on",
  "midi_channel": 1,
  "midi_note_or_cc": 60,
  "midi_velocity_or_value": { "type": "static", "value": 127 },
  "value_mapping": null
}
```

**`midi_velocity_or_value` union type:**

```typescript
type ValueSource =
  | { type: 'static', value: number }        // hardcoded 0–127
  | { type: 'osc_arg', index: number }       // from OSC arg at index, auto-scaled
  // Future:
  | { type: 'wildcard', name: string }       // named binding from pattern match
```

**For MIDI → OSC direction, additional fields:**

```json
{
  "osc_args": [
    { "type": "float", "source": { "type": "midi_value" } },
    { "type": "string", "source": { "type": "static", "value": "hello" } }
  ]
}
```

**`osc_args[].source` union type:**

```typescript
type OscArgSource =
  | { type: 'static', value: number | string }
  | { type: 'midi_value' }                    // CC value or velocity, scaled to float
  | { type: 'midi_note' }                     // note number as int
  // Future:
  | { type: 'wildcard', name: string }
```

### 6.3 Config File Location

Using the `dirs` crate, config lives at:

- **macOS:** `~/Library/Application Support/com.sndwrks.conduit/`
- **Windows:** `%APPDATA%\sndwrks\conduit\`
- **Linux:** `~/.config/sndwrks-conduit/`

Two files: `settings.json` and `mappings.json`.

---

## 7. Engine Design

### 7.1 Startup Sequence

1. Load `settings.json` and `mappings.json` from disk
2. Open MIDI input port (by saved name) via `midir::MidiInput`
3. Open MIDI output port (by saved name) via `midir::MidiOutput`
4. Based on `osc_listen_protocol`:
   - **UDP or Both:** Bind UDP socket on `0.0.0.0:{osc_listen_port}` for incoming OSC
   - **TCP or Both:** Bind TCP listener on `0.0.0.0:{osc_listen_port}`, accept connections, each spawned as a task that reads length-prefixed OSC packets (per OSC 1.0 over TCP: 4-byte big-endian size prefix)
5. Prepare OSC send transport based on `osc_send_protocol`:
   - **UDP:** Create UDP socket targeting `{osc_send_host}:{osc_send_port}`
   - **TCP:** On each send, open a TCP connection to `{osc_send_host}:{osc_send_port}` with a 3-second timeout (`tokio::time::timeout`). If connect or write fails or times out, drop the connection and log an error. TCP connections are not pooled in MVP — each outgoing message opens and closes a connection.
6. Spawn async tasks:
   - **OSC UDP Listener** (if UDP/Both) — reads UDP datagrams, decodes via `rosc`, pushes to router channel
   - **OSC TCP Listener** (if TCP/Both) — accepts connections, reads framed OSC packets, decodes, pushes to router channel
   - **MIDI Listener** — uses `midir` callback, pushes parsed messages to router channel
   - **Router** — receives from all input channels, matches against enabled mappings, dispatches output

### 7.2 Router Logic (per incoming message)

```
for each enabled mapping:
    if mapping.direction matches the incoming protocol:
        if input pattern matches the incoming message:
            build output message using mapping's output definition
            send via the appropriate output (MIDI port or UDP socket)
            emit 'mapping-activity' event to frontend
if no mapping matched:
    emit 'unmatched-message' event to frontend
```

Multiple mappings can match the same input (fan-out is intentional).

### 7.3 OSC Pattern Matching (MVP)

MVP uses exact string match on the OSC address. The incoming address must equal the mapping's `osc_address` exactly.

**Post-MVP** will add OSC pattern matching per the OSC spec (wildcards `*`, `?`, character classes `[a-z]`, alternatives `{foo,bar}`).

### 7.4 MIDI Matching (MVP)

Match on message type + channel + note/CC number. All three must match.

### 7.5 Performance Considerations

- The router runs on a dedicated Tokio task with an unbounded channel — no blocking on UI or I/O
- MIDI callback (from `midir`) must be non-blocking: it pushes raw bytes into a channel immediately
- OSC UDP recv loop uses `tokio::net::UdpSocket` in a tight async loop
- OSC TCP listener spawns one task per accepted connection; each reads length-prefixed frames and pushes decoded messages to the same router channel as UDP
- TCP send uses `tokio::time::timeout(Duration::from_secs(3), ...)` wrapping connect + write; on timeout the future is dropped and the socket closed automatically
- Activity events to the frontend are rate-limited (max 60/sec) to avoid flooding the webview

---

## 8. Persistence Behavior

- **Settings:** Saved on every change, debounced 500ms. Loaded on app start.
- **Mappings:** Saved on every CRUD operation (add, update, delete, reorder). The entire mappings array is rewritten atomically (write to temp file, rename).
- **Engine state is not persisted** — the engine always starts stopped unless `engine_auto_start` is true.
- **File watching:** Not needed for MVP. Config is only written by this app.

---

## 9. Error Handling

| Scenario | Behavior |
|----------|----------|
| MIDI port not found on startup | Show toast warning, leave port unset, let user reselect in settings |
| MIDI port disconnected while running | Stop engine, show toast error, emit `engine-status` event with error |
| OSC port already in use | Show toast error on engine start, suggest changing port |
| OSC TCP send timeout (>3s) | Drop the connection, log error to activity log, continue processing other mappings |
| OSC TCP connection refused | Log error to activity log with target host:port, do not retry automatically |
| OSC TCP listener client disconnects | Clean up that client's task silently, continue accepting new connections |
| Invalid OSC address in mapping | Inline validation — red border on input, tooltip explaining format (must start with `/`) |
| Malformed incoming OSC packet | Log to activity as error, skip silently |
| Malformed incoming MIDI | Log and skip |

---

## 10. Build & Distribution

### 10.1 Development

```bash
# Prerequisites: Rust toolchain, Node.js 18+, pnpm
pnpm create tauri-app conduit --template react-ts

# Install frontend deps
cd conduit && pnpm install
pnpm add @shadcn/ui tailwindcss  # + shadcn init

# Add Rust deps in src-tauri/Cargo.toml
# midir, rosc, tokio, serde, serde_json, dirs, uuid

pnpm tauri dev  # hot-reload development
```

### 10.2 Production Build

```bash
pnpm tauri build
# Outputs:
#   macOS: .dmg / .app
#   Windows: .msi / .exe (NSIS)
#   Linux: .deb / .AppImage
```

### 10.3 App Metadata

- **App name:** Conduit
- **Brand:** sndwrks
- **Bundle ID:** `com.sndwrks.conduit`
- **Icon:** SVG logo provided by sndwrks (will be converted to platform-specific icon formats during build: `.icns` for macOS, `.ico` for Windows, `.png` for Linux)
- **Min OS:** macOS 10.15, Windows 10, Ubuntu 20.04

### 10.4 GitHub Actions — CI (Unit Tests)

**File:** `.github/workflows/ci.yml`
**Trigger:** Every push to any branch, every pull request.

```yaml
name: CI

on:
  push:
  pull_request:

jobs:
  test-rust:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Install Linux deps
        if: runner.os == 'Linux'
        run: |
          sudo apt-get update
          sudo apt-get install -y libgtk-3-dev libwebkit2gtk-4.1-dev \
            libappindicator3-dev librsvg2-dev patchelf libasound2-dev
      - name: Rust tests
        working-directory: src-tauri
        run: cargo test --workspace

  test-frontend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v4
        with:
          version: 9
      - uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: pnpm
      - run: pnpm install --frozen-lockfile
      - run: pnpm test
      - run: pnpm lint

  typecheck:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: pnpm/action-setup@v4
        with:
          version: 9
      - uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: pnpm
      - run: pnpm install --frozen-lockfile
      - run: pnpm typecheck
```

**Notes:**

- Rust tests run on all three platforms to catch platform-specific `midir`/`rosc` issues
- Frontend tests and typecheck only need to run on one OS (Ubuntu)
- `libasound2-dev` is required on Linux for `midir` (ALSA backend)
- The workflow assumes `pnpm test`, `pnpm lint`, and `pnpm typecheck` scripts are defined in `package.json`

### 10.5 GitHub Actions — Release

**File:** `.github/workflows/release.yml`
**Trigger:** Push to the `release` branch.
**Versioning:** Semver, read from `src-tauri/tauri.conf.json` → `version` field. The release tag and GitHub Release title are derived from this version.

```yaml
name: Release

on:
  push:
    branches:
      - release

permissions:
  contents: write

jobs:
  get-version:
    runs-on: ubuntu-latest
    outputs:
      version: ${{ steps.read.outputs.version }}
    steps:
      - uses: actions/checkout@v4
      - id: read
        run: |
          VERSION=$(jq -r '.version' src-tauri/tauri.conf.json)
          echo "version=$VERSION" >> "$GITHUB_OUTPUT"

  build:
    needs: get-version
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          - os: macos-latest
            target: aarch64-apple-darwin
            label: macos-arm64
          - os: macos-latest
            target: x86_64-apple-darwin
            label: macos-x64
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            label: windows-x64
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            label: linux-x64
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      - uses: pnpm/action-setup@v4
        with:
          version: 9
      - uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: pnpm
      - name: Install Linux deps
        if: runner.os == 'Linux'
        run: |
          sudo apt-get update
          sudo apt-get install -y libgtk-3-dev libwebkit2gtk-4.1-dev \
            libappindicator3-dev librsvg2-dev patchelf libasound2-dev
      - run: pnpm install --frozen-lockfile
      - name: Build Tauri app
        uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          # macOS signing (configure secrets in repo settings):
          # APPLE_CERTIFICATE: ${{ secrets.APPLE_CERTIFICATE }}
          # APPLE_CERTIFICATE_PASSWORD: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
          # APPLE_SIGNING_IDENTITY: ${{ secrets.APPLE_SIGNING_IDENTITY }}
          # APPLE_ID: ${{ secrets.APPLE_ID }}
          # APPLE_PASSWORD: ${{ secrets.APPLE_PASSWORD }}
          # APPLE_TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}
        with:
          tagName: v${{ needs.get-version.outputs.version }}
          releaseName: Conduit v${{ needs.get-version.outputs.version }}
          releaseBody: "See CHANGELOG.md for details."
          releaseDraft: true
          prerelease: false
          args: --target ${{ matrix.target }}
```

**Release process:**

1. Update `version` in `src-tauri/tauri.conf.json` following semver (e.g., `0.1.0` → `0.2.0`)
2. Update `CHANGELOG.md` with release notes
3. Commit and push to the `release` branch (or merge a PR into `release`)
4. The workflow builds on all 4 targets (macOS arm64, macOS x64, Windows x64, Linux x64)
5. `tauri-action` creates a **draft** GitHub Release tagged `v{version}` with all platform artifacts attached
6. Review the draft release on GitHub and publish when ready

**Artifacts produced per platform:**

| Platform | Artifacts |
|----------|-----------|
| macOS arm64 | `.dmg`, `.app.tar.gz` (for updater) |
| macOS x64 | `.dmg`, `.app.tar.gz` |
| Windows x64 | `.msi`, `.exe` (NSIS installer) |
| Linux x64 | `.deb`, `.AppImage` |

**Notes:**

- macOS code signing secrets are commented out — uncomment and configure in GitHub repo secrets when an Apple Developer account is available
- The release is created as a draft so artifacts from all matrix jobs can attach before publishing
- If the tag `v{version}` already exists, the workflow will fail — bump the version before pushing to `release`

---

## 11. MVP Scope Summary

**In scope:**

- Single-screen mapping table UI
- OSC → MIDI direction (Note On/Off, CC)
- MIDI → OSC direction (Note On/Off, CC)
- Static value and single-argument passthrough
- MIDI device selection (input + output) with enumeration
- OSC listen port and send host/port configuration (UDP and TCP)
- Persistent settings and mappings (JSON files)
- Real-time activity log
- Start/stop engine toggle
- Dark mode only UI with bundled Roboto Mono Variable typeface
- Cross-platform (macOS, Windows, Linux)
- GitHub Actions CI (unit tests on every push) and release workflow (semver, builds on push to `release` branch)

**Out of scope (future):**

- Wildcard / argument binding patterns (`{arg0}`, `*`)
- OSC spec pattern matching (`*`, `?`, `[a-z]`)
- Multiple MIDI device support (input and output from several devices simultaneously)
- MIDI Learn (click "Learn", play a note, auto-fill the MIDI fields)
- OSC Learn (click "Learn", send an OSC message, auto-fill the address)
- SysEx message support
- Preset / file management (load/save named mapping sets)
- MIDI clock, MTC, or transport messages
- OSC bundle support (only individual messages in MVP)
- Value scaling curves (linear only in MVP)
- Drag-and-drop row reordering (can be cosmetic only, deferred to post-MVP)

---

## 12. Post-MVP Roadmap

### Phase 2 — Wildcard Patterns & Learn

- Pattern syntax: `/osc/fader/{channel}` binds `channel` from the address path
- Argument binding: `{arg0}`, `{arg1}` reference positional OSC arguments
- MIDI field binding: `{note}`, `{velocity}`, `{value}`, `{channel}`
- Example: `/osc/note {arg0} {arg1}` → `Note On {arg0} Vel {arg1} Ch 1`
- MIDI Learn button per row — listens for next incoming MIDI, populates fields
- OSC Learn button — listens for next incoming OSC, populates address + detected arg types

### Phase 3 — Advanced Routing

- Multiple simultaneous MIDI devices (per-mapping device assignment)
- Value scaling: configurable input/output range with curve (linear, exponential, logarithmic)
- OSC bundle support
- Conditional mappings (only fire if value > threshold)

### Phase 4 — Presets & Collaboration

- Named mapping presets (save/load/switch)
- Import/export mappings as `.conduit` files
- Duplicate row
- Group rows with collapsible sections
- Multi-select and bulk operations

---

## 13. Reference: MIDI Note Names

For the note selector UI, use standard MIDI note numbering:

| MIDI # | Note | MIDI # | Note |
|--------|------|--------|------|
| 0 | C-1 | 60 | C3 |
| 12 | C0 | 72 | C4 |
| 24 | C1 | 84 | C5 |
| 36 | C2 | 96 | C6 |
| 48 | C2 | 108 | C7 |

(Middle C = C3 = MIDI note 60, following the most common DAW convention. Expose a setting to switch to C4=60 convention if needed post-MVP.)

The note selector should accept both typed note names ("C3", "F#4") and numeric input (0–127).

---

## 14. Reference: OSC Message Format

An OSC message consists of:

- **Address pattern:** A string starting with `/`, using `/` as a path separator (e.g., `/mixer/channel/1/volume`)
- **Type tag string:** Indicates argument types (e.g., `,fis` = float, int, string)
- **Arguments:** Zero or more values matching the type tags

MVP supports: `i` (int32), `f` (float32), `s` (string). Post-MVP can add `b` (blob), `T`/`F` (boolean), `N` (nil).

### Transport Framing

- **UDP:** Each datagram contains exactly one OSC packet (message or bundle). No framing needed.
- **TCP:** Uses SLIP (Serial Line Internet Protocol) framing per the OSC 1.1 spec, or the simpler length-prefix framing (4-byte big-endian size header before each OSC packet). Conduit MVP uses **length-prefix framing** for TCP — this is the most common convention in practice (used by SuperCollider, Max/MSP, etc.). The 3-second send timeout applies to the entire connect + write sequence for each outgoing TCP message.