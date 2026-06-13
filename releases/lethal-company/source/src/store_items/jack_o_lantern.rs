// Sources: vault/store_items/jack_o_lantern.md, vault/item_index_pages/items.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::SimChecksumState;

pub const BUY_COST: u32 = 50;
pub const LUCK_BONUS: I32F32 = I32F32::lit("0.012");

#[derive(Event)]
pub struct JackOLanternPurchasedEvent;

#[derive(Resource, Default)]
pub struct JackOLanternState {
    pub owned: bool,
}

pub struct JackOLanternPlugin;

impl Plugin for JackOLanternPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<JackOLanternPurchasedEvent>()
            .init_resource::<JackOLanternState>()
            .add_systems(
                FixedUpdate,
                (handle_purchase, jack_o_lantern_checksum).chain(),
            );
    }
}

fn handle_purchase(
    mut events: EventReader<JackOLanternPurchasedEvent>,
    mut state: ResMut<JackOLanternState>,
) {
    for _ in events.read() {
        if !state.owned {
            state.owned = true;
        }
    }
}

fn jack_o_lantern_checksum(state: Res<JackOLanternState>, mut cs: ResMut<SimChecksumState>) {
    cs.accumulate(state.owned as u64);
}