// Sources: vault/store_items/disco_ball.md, vault/item_index_pages/items.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::SimChecksumState;

pub const BUY_COST: u32 = 150;
pub const LUCK_BONUS: I32F32 = I32F32::lit("0.06");

#[derive(Event)]
pub struct DiscoBallPurchasedEvent;

/// Set `lights_on` from the ship-lights system; `active` is derived each tick.
#[derive(Resource, Default)]
pub struct DiscoBallState {
    pub owned: bool,
    /// True when the disco ball is placed and the room lights are off.
    pub active: bool,
    pub lights_on: bool,
}

pub struct DiscoBallPlugin;

impl Plugin for DiscoBallPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DiscoBallPurchasedEvent>()
            .init_resource::<DiscoBallState>()
            .add_systems(
                FixedUpdate,
                (handle_purchase, update_active_state, disco_ball_checksum).chain(),
            );
    }
}

fn handle_purchase(
    mut events: EventReader<DiscoBallPurchasedEvent>,
    mut state: ResMut<DiscoBallState>,
) {
    for _ in events.read() {
        if !state.owned {
            state.owned = true;
        }
    }
}

fn update_active_state(mut state: ResMut<DiscoBallState>) {
    state.active = state.owned && !state.lights_on;
}

fn disco_ball_checksum(state: Res<DiscoBallState>, mut cs: ResMut<SimChecksumState>) {
    cs.accumulate(state.owned as u64);
    cs.accumulate(state.active as u64);
    cs.accumulate(state.lights_on as u64);
}