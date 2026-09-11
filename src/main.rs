use std::collections::BTreeMap;
use zellij_tile::prelude::*;

register_plugin!(State);

#[derive(Default)]
struct State {
    toggled: bool,
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
        if self.toggled {
            return false;
        }
        if let Event::PaneUpdate(manifest) = event {
            self.toggled = true;

            let mut tab_bar: Option<PaneId> = None;
            let mut status_bar: Option<PaneId> = None;
            let mut hidden = false;

            for panes in manifest.panes.values() {
                for pane in panes {
                    match pane.plugin_url.as_deref() {
                        Some("tab-bar") | Some("zellij:tab-bar") => {
                            tab_bar = Some(PaneId::Plugin(pane.id));
                            hidden = pane.is_suppressed;
                        },
                        Some("status-bar") | Some("zellij:status-bar") => {
                            status_bar = Some(PaneId::Plugin(pane.id));
                        },
                        _ => {},
                    }
                }
            }

            if hidden {
                if let Some(id) = tab_bar {
                    show_pane_with_id(id, false, false);
                }
                if let Some(id) = status_bar {
                    show_pane_with_id(id, false, false);
                }
            } else {
                if let Some(id) = tab_bar {
                    hide_pane_with_id(id);
                }
                if let Some(id) = status_bar {
                    hide_pane_with_id(id);
                }
            }

            close_self();
        }
        false
    }

    fn render(&mut self, _rows: usize, _cols: usize) {}
}
