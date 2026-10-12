use std::collections::BTreeMap;

use kdl::{KdlDocument, KdlNode};
use zellij_tile::prelude::*;

register_plugin!(State);

const TAB_BAR: &str = "zellij:tab-bar";
const STATUS_BAR: &str = "zellij:status-bar";

#[derive(Default)]
struct State {
    tabs: Vec<TabInfo>,
    bars_present: bool,
    // Whether zen mode is currently active (bars hidden). `None` until the first
    // toggle, at which point it is derived from the current panes.
    zen: Option<bool>,
}

impl ZellijPlugin for State {
    fn load(&mut self, _configuration: BTreeMap<String, String>) {
        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
        ]);
        subscribe(&[EventType::TabUpdate, EventType::PaneUpdate]);
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::TabUpdate(tabs) => {
                self.tabs = tabs;
            }
            Event::PaneUpdate(manifest) => {
                self.bars_present = manifest
                    .panes
                    .values()
                    .flatten()
                    .any(|pane| is_bar_url(pane.plugin_url.as_deref()));
            }
            _ => {}
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
        // The bars are toggled by re-applying a minimal layout per tab rather than
        // suppressing the bar panes: suppressed panes are dropped by Zellij's session
        // serializer, which used to make zen mode impossible to undo after a
        // session was saved/restored. The minimal layout only describes the bars (and
        // a placeholder pane); `retain_existing_terminal_panes` keeps the user's panes.
        //
        // We deliberately do not call `dump_session_layout` here: it deadlocks when
        // invoked from a plugin handler (the plugin's mutex is held for the duration).
        let was_zen = self.zen.unwrap_or(!self.bars_present);
        let zen = !was_zen;
        self.zen = Some(zen);

        if self.tabs.is_empty() {
            return;
        }

        let mut layout_children = KdlDocument::new();
        for tab in &self.tabs {
            layout_children.nodes_mut().push(tab_layout(&tab.name, zen));
        }
        let mut layout = KdlNode::new("layout");
        layout.set_children(layout_children);

        let mut document = KdlDocument::new();
        document.nodes_mut().push(layout);

        override_layout(
            LayoutInfo::Stringified(document.to_string()),
            true,  // retain existing terminal panes (and their contents)
            false, // drop the bar panes when hiding
            false, // apply to every tab
            BTreeMap::new(),
        );
    }
}

fn is_bar_url(url: Option<&str>) -> bool {
    matches!(
        url,
        Some(TAB_BAR) | Some(STATUS_BAR) | Some("tab-bar") | Some("status-bar")
    )
}

fn bar_pane(location: &str) -> KdlNode {
    let mut plugin = KdlNode::new("plugin");
    plugin.insert("location", location);

    let mut children = KdlDocument::new();
    children.nodes_mut().push(plugin);

    let mut pane = KdlNode::new("pane");
    pane.insert("size", 1i64);
    pane.insert("borderless", true);
    pane.set_children(children);
    pane
}

fn tab_layout(name: &str, zen: bool) -> KdlNode {
    let mut children = KdlDocument::new();
    if !zen {
        children.nodes_mut().push(bar_pane(TAB_BAR));
    }
    // placeholder pane; existing terminals are retained and matched against it
    children.nodes_mut().push(KdlNode::new("pane"));
    if !zen {
        children.nodes_mut().push(bar_pane(STATUS_BAR));
    }

    let mut tab = KdlNode::new("tab");
    tab.insert("name", name.to_string());
    tab.set_children(children);
    tab
}
