use std::collections::BTreeMap;
use zellij_tile::prelude::*;

register_plugin!(State);

#[derive(Default)]
struct State {
    manifest: Option<PaneManifest>,
}

impl ZellijPlugin for State {
    fn load(&mut self, _configuration: BTreeMap<String, String>) {
        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
        ]);
        subscribe(&[EventType::PaneUpdate]);
    }

    fn update(&mut self, event: Event) -> bool {
        if let Event::PaneUpdate(manifest) = event {
            self.manifest = Some(manifest);
        }
        false
    }

    fn pipe(&mut self, message: PipeMessage) -> bool {
        if message.name == "toggle" {
            self.toggle_bars();
        }
        false
    }

    fn render(&mut self, _rows: usize, _cols: usize) {}
}

impl State {
    fn toggle_bars(&mut self) {
        let Some(manifest) = self.manifest.as_ref() else {
            return;
        };

        // Keep tab-bars and status-bars separate so we can always (un)suppress
        // them in a stable order: tab-bar(s) first, then status-bar(s). The
        // order the manifest reports them in is not stable after suppression,
        // and restoring the status-bar before the tab-bar misplaces them.
        let mut tab_bars: Vec<PaneId> = Vec::new();
        let mut status_bars: Vec<PaneId> = Vec::new();
        let mut any_hidden = false;

        for panes in manifest.panes.values() {
            for pane in panes {
                match pane.plugin_url.as_deref() {
                    Some("tab-bar") | Some("zellij:tab-bar") => {
                        tab_bars.push(PaneId::Plugin(pane.id));
                        any_hidden = any_hidden || pane.is_suppressed;
                    },
                    Some("status-bar") | Some("zellij:status-bar") => {
                        status_bars.push(PaneId::Plugin(pane.id));
                        any_hidden = any_hidden || pane.is_suppressed;
                    },
                    _ => {},
                }
            }
        }

        let hide = !any_hidden;

        for id in tab_bars.into_iter().chain(status_bars) {
            if hide {
                hide_pane_with_id(id);
            } else {
                show_pane_with_id(id, false, false);
            }
        }
    }
}
