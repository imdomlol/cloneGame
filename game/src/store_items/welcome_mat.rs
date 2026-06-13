// Sources: vault/store_items/welcome_mat.md, vault/item_index_pages/items.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::SimChecksumState;

pub const BUY_COST: u32 = 40;
pub const LUCK_BONUS: I32F32 = I32F32::lit("0.003");

#[derive(Event)]
pub struct WelcomeMatPurchasedEvent;

#[derive(Resource, Default)]
pub struct WelcomeMatState {
    pub owned: bool,
}

pub struct WelcomeMatPlugin;

impl Plugin for WelcomeMatPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<WelcomeMatPurchasedEvent>()
            .init_resource::<WelcomeMatState>()
            .add_systems(FixedUpdate, (handle_purchase, welcome_mat_checksum).chain());
    }
}

fn handle_purchase(
    mut events: EventReader<WelcomeMatPurchasedEvent>,
    mut state: ResMut<WelcomeMatState>,
) {
    for _ in events.read() {
        if !state.owned {
            state.owned = true;
        }
    }
}

fn welcome_mat_checksum(state: Res<WelcomeMatState>, mut cs: ResMut<SimChecksumState>) {
    cs.accumulate(state.owned as u64);
}