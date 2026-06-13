// Sources: vault/store_items/record_player.md, vault/item_index_pages/items.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::SimChecksumState;

pub const BUY_COST: u32 = 120;
pub const LUCK_BONUS: I32F32 = I32F32::lit("0.005");

#[derive(Event)]
pub struct RecordPlayerPurchasedEvent;

#[derive(Event)]
pub struct RecordPlayerActivatedEvent;

#[derive(Resource, Default)]
pub struct RecordPlayerState {
    pub owned: bool,
    /// True while music is playing; inaudible to entities while active.
    pub active: bool,
}

pub struct RecordPlayerPlugin;

impl Plugin for RecordPlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<RecordPlayerPurchasedEvent>()
            .add_event::<RecordPlayerActivatedEvent>()
            .init_resource::<RecordPlayerState>()
            .add_systems(
                FixedUpdate,
                (handle_purchase, handle_activation, record_player_checksum).chain(),
            );
    }
}

fn handle_purchase(
    mut events: EventReader<RecordPlayerPurchasedEvent>,
    mut state: ResMut<RecordPlayerState>,
) {
    for _ in events.read() {
        if !state.owned {
            state.owned = true;
        }
    }
}

fn handle_activation(
    mut events: EventReader<RecordPlayerActivatedEvent>,
    mut state: ResMut<RecordPlayerState>,
) {
    for _ in events.read() {
        if state.owned {
            state.active = !state.active;
        }
    }
}

fn record_player_checksum(state: Res<RecordPlayerState>, mut cs: ResMut<SimChecksumState>) {
    cs.accumulate(state.owned as u64);
    cs.accumulate(state.active as u64);
}