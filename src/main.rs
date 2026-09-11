use std::collections::BTreeMap;
use zellij_tile::prelude::*;

register_plugin!(State);

#[derive(Default)]
struct State;

impl ZellijPlugin for State {
    fn load(&mut self, _configuration: BTreeMap<String, String>) {}
}
