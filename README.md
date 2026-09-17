# zellij-zen

A tiny [Zellij](https://zellij.dev) plugin that toggles the visibility of the
`tab-bar` (top) and `status-bar` (bottom) panes, giving you a distraction-free
"zen" mode on demand.

Press a single keybind to hide every tab's bars, press it again to bring them
back.

![zellij-zen demo](assets/zen.gif)

## Install

No manual download needed: Zellij fetches the prebuilt `.wasm` from the release
URL below and caches it.

> **Note:** Zellij caches `https://` plugins **by URL**, so a `…/latest/…`
> URL keeps serving the first version it downloaded. The snippets below use a
> *versioned* URL — bump the version to pick up a new release (or clear the
> Zellij plugin cache).

`zellij-zen` runs as a **background plugin**: load it once at startup, then
trigger it from a keybind. This keeps the toggle instant and avoids spawning a
pane on every press.

### 1. Load it at startup

```kdl
load_plugins {
    "https://github.com/kxrur/zellij-zen/releases/download/v0.2.0/zellij-zen.wasm"
}
```

### 2. Bind a key to toggle

```kdl
bind "Alt v" {
    MessagePlugin "https://github.com/kxrur/zellij-zen/releases/download/v0.2.0/zellij-zen.wasm" {
        name "toggle"
    }
}
```

## Permissions

The first time the plugin loads (i.e. when you start a session after adding it),
Zellij will ask you to grant two permissions:

- `ReadApplicationState` — to inspect the current panes
- `ChangeApplicationState` — to hide/show the bar panes

Answer `y` once. The grant is cached, so future sessions run the toggle without
asking again.

## Build from source

```sh
rustup target add wasm32-wasip1
cargo build --release --target wasm32-wasip1
```

The plugin is written to `target/wasm32-wasip1/release/zellij-zen.wasm`.

## Known issues

- After toggling, Zellij may reset a tab's auto-derived name (e.g. the current
  directory) back to its default (`Tab #1`). This happens in Zellij itself when
  bar panes are suppressed/restored and isn't controlled by the plugin.

## License

[MIT](LICENSE)
