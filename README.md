# zellij-zen

A tiny [Zellij](https://zellij.dev) plugin that toggles the visibility of the
`tab-bar` (top) and `status-bar` (bottom) panes, giving you a distraction-free
"zen" mode on demand.

Press a single keybind to hide both bars, press it again to bring them back.

## Install

No manual download needed: Zellij fetches the prebuilt `.wasm` from the release
URL below and caches it. Just reference it from a keybind or at startup.

### Via a keybind

Add a binding (e.g. `Alt v`) to your Zellij config:

```kdl
bind "Alt v" {
    LaunchPlugin "https://github.com/kxrur/zellij-zen/releases/latest/download/zellij-zen.wasm" {
        floating true
    }
}
```

### Load on startup

```kdl
load_plugins {
    "https://github.com/kxrur/zellij-zen/releases/latest/download/zellij-zen.wasm"
}
```

## Permissions

The **first time you press your keybind**, Zellij will ask you to grant two
permissions:

- `ReadApplicationState` — to inspect the current panes
- `ChangeApplicationState` — to hide/show the bar panes

Answer `y` once. The grant is cached, so every later press (and future Zellij
sessions) will run the toggle without asking again.

## Build from source

```sh
rustup target add wasm32-wasip1
cargo build --release --target wasm32-wasip1
```

The plugin is written to `target/wasm32-wasip1/release/zellij-zen.wasm`.

## License

[MIT](LICENSE)
