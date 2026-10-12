use std::collections::BTreeMap;

use kdl::{KdlDocument, KdlNode};
use zellij_tile::prelude::*;

register_plugin!(State);

const TAB_BAR: &str = "zellij:tab-bar";
const STATUS_BAR: &str = "zellij:status-bar";

#[derive(Default)]
struct State {
    // Whether zen mode is currently active (bars hidden). `None` until the first
    // toggle, at which point it is derived from the live session layout.
    zen: Option<bool>,
}

impl ZellijPlugin for State {
    fn load(&mut self, _configuration: BTreeMap<String, String>) {
        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
        ]);
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
        // We toggle the bars by rewriting the session layout and applying it with
        // `override_layout`, rather than suppressing the bar panes. Suppressed panes
        // are dropped by Zellij's session serializer, which used to make zen mode
        // impossible to undo after a session was saved/restored.
        let Ok((layout, _)) = dump_session_layout() else {
            return;
        };
        let Ok(mut document) = layout.parse::<KdlDocument>() else {
            return;
        };

        let bars_present = document
            .nodes()
            .iter()
            .find(|node| node.name().value() == "layout")
            .map(|layout_node| tab_nodes(layout_node).any(has_bars))
            .unwrap_or(false);

        // On the first toggle, infer the current state from the layout: if the bars
        // are present, zen mode is currently off.
        let was_zen = self.zen.unwrap_or(!bars_present);
        let zen = !was_zen;
        self.zen = Some(zen);

        let Some(layout_node) = document
            .nodes_mut()
            .iter_mut()
            .find(|node| node.name().value() == "layout")
        else {
            return;
        };

        // Drop the global `cwd`. When re-applied, it would give the terminal panes
        // a run that no longer matches the already-running terminals, causing the
        // override to spawn duplicate terminals. Existing terminals are retained
        // (with their own cwd) regardless.
        if let Some(children) = layout_node.children_mut() {
            children
                .nodes_mut()
                .retain(|node| node.name().value() != "cwd");
        }

        for tab in tab_nodes_mut(layout_node) {
            if zen {
                remove_bars(tab);
            } else {
                insert_bars(tab);
            }
        }

        override_layout(
            LayoutInfo::Stringified(document.to_string()),
            true,  // retain existing terminal panes (and their contents)
            false, // drop the bar panes we removed from the layout
            false, // apply to every tab
            BTreeMap::new(),
        );
    }
}

fn tab_nodes(layout_node: &KdlNode) -> impl Iterator<Item = &KdlNode> {
    layout_node
        .children()
        .into_iter()
        .flat_map(|children| children.nodes())
        .filter(|node| node.name().value() == "tab")
}

fn tab_nodes_mut(layout_node: &mut KdlNode) -> impl Iterator<Item = &mut KdlNode> {
    layout_node
        .children_mut()
        .as_mut()
        .into_iter()
        .flat_map(|children| children.nodes_mut().iter_mut())
        .filter(|node| node.name().value() == "tab")
}

fn plugin_location(pane: &KdlNode) -> Option<&str> {
    pane.children()
        .into_iter()
        .flat_map(|children| children.nodes())
        .find(|node| node.name().value() == "plugin")
        .and_then(|plugin| plugin.get("location"))
        .and_then(|entry| entry.value().as_string())
}

fn is_bar_pane(node: &KdlNode) -> bool {
    if node.name().value() != "pane" {
        return false;
    }
    let location = plugin_location(node);
    location == Some(TAB_BAR)
        || location == Some(STATUS_BAR)
        || location == Some("tab-bar")
        || location == Some("status-bar")
}

fn has_bars(tab: &KdlNode) -> bool {
    tab.children()
        .into_iter()
        .flat_map(|children| children.nodes())
        .any(is_bar_pane)
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

fn insert_bars(tab: &mut KdlNode) {
    let children = tab.ensure_children();
    let nodes = children.nodes_mut();

    // tab-bar goes at the very top, status-bar at the very bottom (before any
    // floating panes node if one exists).
    nodes.insert(0, bar_pane(TAB_BAR));
    let insert_at = nodes
        .iter()
        .position(|node| node.name().value() == "floating_panes")
        .unwrap_or(nodes.len());
    nodes.insert(insert_at, bar_pane(STATUS_BAR));
}

fn remove_bars(tab: &mut KdlNode) {
    if let Some(children) = tab.children_mut() {
        children.nodes_mut().retain(|node| !is_bar_pane(node));
    }
}
