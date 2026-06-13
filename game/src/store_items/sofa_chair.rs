// Sources: vault/store_items/sofa_chair.md, vault/item_index_pages/decor.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::SimChecksumState;

pub const BUY_COST: u32 = 150;
pub const LUCK_BONUS: I32F32 = I32F32::lit("0.008");

#[derive(Event)]
pub struct SofaChairPurchasedEvent;

#[derive(Event)]
pub struct SofaChairPlacedEvent;

#[derive(Event)]
pub struct SofaChairSatOnEvent;

#[derive(Resource, Default)]
pub struct SofaChairState {
    pub owned: bool,
    pub placed: bool,
    pub occupied: bool,
}

pub struct SofaChairPlugin;

impl Plugin for SofaChairPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SofaChairPurchasedEvent>()
            .add_event::<SofaChairPlacedEvent>()
            .add_event::<SofaChairSatOnEvent>()
            .init_resource::<SofaChairState>()
            .add_systems(
                FixedUpdate,
                (
                    handle_purchase,
                    handle_placement,
                    handle_interaction,
                    sofa_chair_checksum,
                )
                    .chain(),
            );
    }
}

fn handle_purchase(
    mut events: EventReader<SofaChairPurchasedEvent>,
    mut state: ResMut<SofaChairState>,
) {
    for _ in events.read() {
        if !state.owned {
            state.owned = true;
        }
    }
}

fn handle_placement(
    mut events: EventReader<SofaChairPlacedEvent>,
    mut state: ResMut<SofaChairState>,
) {
    for _ in events.read() {
        if state.owned {
            state.placed = true;
        }
    }
}

fn handle_interaction(
    mut events: EventReader<SofaChairSatOnEvent>,
    mut state: ResMut<SofaChairState>,
) {
    for _ in events.read() {
        if state.owned && state.placed {
            state.occupied = true;
        }
    }
}

fn sofa_chair_checksum(state: Res<SofaChairState>, mut cs: ResMut<SimChecksumState>) {
    cs.accumulate(state.owned as u64);
    cs.accumulate(state.placed as u64);
    cs.accumulate(state.occupied as u64);
}