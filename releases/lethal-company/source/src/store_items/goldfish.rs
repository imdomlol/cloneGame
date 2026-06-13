// Sources: vault/store_items/goldfish.md, vault/item_index_pages/items.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::SimChecksumState;

pub const BUY_COST: u32 = 50;
pub const LUCK_BONUS: I32F32 = I32F32::lit("0.006");

#[derive(Event)]
pub struct GoldfishPurchasedEvent;

#[derive(Resource, Default)]
pub struct GoldfishState {
    pub owned: bool,
}

pub struct GoldfishPlugin;

impl Plugin for GoldfishPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<GoldfishPurchasedEvent>()
            .init_resource::<GoldfishState>()
            .add_systems(
                FixedUpdate,
                (handle_purchase, goldfish_checksum).chain(),
            );
    }
}

fn handle_purchase(
    mut events: EventReader<GoldfishPurchasedEvent>,
    mut state: ResMut<GoldfishState>,
) {
    for _ in events.read() {
        if !state.owned {
            state.owned = true;
        }
    }
}

fn goldfish_checksum(state: Res<GoldfishState>, mut cs: ResMut<SimChecksumState>) {
    cs.accumulate(state.owned as u64);
}