# VectorScreen

Touch screen UI for FlashForge Adventurer 5M 3D printers, built with [Slint](https://slint.dev/) and Rust.

VectorScreen connects to [Moonraker](https://moonraker.readthedocs.io/) (the Klipper API) to provide a full-screen touch interface for printer control — temperature management, axis movement, LED control, filament operations, and a G-code console.

## Features

- **Home Screen** — Print progress, temperatures, system status
- **Controls Screen** — Axis movement, homing, thermal control, filament load/unload
- **Console Screen** — Direct G-code input with command history
- **Emergency Stop** — One-tap M112 shutdown
- **Material Presets** — PLA / PETG / ABS temperature presets
- **LED Control** — Chamber light toggle and brightness
- **Auto-reconnection** — Exponential backoff on WebSocket disconnects
- **Panic Logging** — Crash logs written to `/tmp/vector-screen-panic.log`

## Prerequisites

- **Rust** (stable, 2021 edition) — [Install rustup](https://rustup.rs/)
- **ARM cross-compiler** (for target builds):
  ```bash
  # Debian/Ubuntu
  sudo apt install musl-tools

  # Arch
  sudo pacman -S musl
  ```
- **Moonraker** running on the printer (default port 7125)
- **Klipper** firmware on the printer

## Building

### Native debug build (for development)

```bash
make build
# or
cargo build
```

### Cross-compile for ARM (Adventurer 5M / Raspberry Pi)

```bash
make release
# or
cargo build --release --target armv7-unknown-linux-musleabihf
```

The ARM binary is at `target/armv7-unknown-linux-musleabihf/release/vector-screen`.

### Run tests

```bash
make test
```

### Check compilation (no output binary)

```bash
make check
```

## Installation

### Copy binary to target device

```bash
scp target/armv7-unknown-linux-musleabihf/release/vector-screen \
    user@printer-ip:/usr/local/bin/vector-screen
```

### Or use Make's install target (run on the target)

```bash
sudo make install
```

### Run directly

```bash
MOONRAKER_HOST=192.168.1.100 vector-screen
```

### Run as systemd service

```bash
sudo systemctl enable --now vector-screen
```

## Configuration

VectorScreen is configured via environment variables:

| Variable | Default | Description |
|---|---|---|
| `MOONRAKER_HOST` | `127.0.0.1` | IP address of the Moonraker instance |

### Example: systemd service with custom IP

```bash
# Edit the service file
sudo systemctl edit vector-screen

# Add:
[Service]
Environment="MOONRAKER_HOST=192.168.1.100"
```

## Project Structure

```
src/
  main.rs              — Application entry point, UI wiring
  ui/
    mod.rs             — AppState, Screen enum
    home.rs            — Home screen state
    controls.rs        — Controls screen state
    thermal.rs         — Thermal control logic
    motion.rs          — Axis movement control
    led.rs             — LED control
    console.rs         — Console input/history
    progress.rs        — Duration formatting
  moonraker/
    mod.rs             — WebSocket client, reconnection
    message.rs         — JSON-RPC message types
  klipper/
    mod.rs             — G-code command interface
    gcode.rs           — G-code formatting and validation

ui/
  app.slint            — Main app shell + sidebar
  theme.slint          — Design tokens
  main.slint           — Legacy entry (unused)
  screens/
    home.slint         — Home screen layout
    controls.slint     — Controls screen layout
    console.slint      — Console screen layout
  components/
    button.slint       — Button component
    card.slint         — Card component
    label.slint        — Label component
    toggle.slint       — Toggle switch

build.rs               — Slint compilation config
```

## License

MIT
