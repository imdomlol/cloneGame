// Sources: vault/store_items/classic_painting.md, vault/item_index_pages/items.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::SimChecksumState;

pub const BUY_COST: u32 = 400;
pub const LUCK_BONUS: I32F32 = I32F32::lit("0.006");

#[derive(Event)]
pub struct ClassicPaintingPurchasedEvent;

#[derive(Resource, Default)]
pub struct ClassicPaintingState {
    pub owned: bool,
}

pub struct ClassicPaintingPlugin;

impl Plugin for ClassicPaintingPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ClassicPaintingPurchasedEvent>()
            .init_resource::<ClassicPaintingState>()
            .add_systems(
                FixedUpdate,
                (handle_purchase, classic_painting_checksum).chain(),
            );
    }
}

fn handle_purchase(
    mut events: EventReader<ClassicPaintingPurchasedEvent>,
    mut state: ResMut<ClassicPaintingState>,
) {
    for _ in events.read() {
        if !state.owned {
            state.owned = true;
        }
    }
}

fn classic_painting_checksum(state: Res<ClassicPaintingState>, mut cs: ResMut<SimChecksumState>) {
    cs.accumulate(state.owned as u64);
}