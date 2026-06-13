// Sources: vault/store_items/toilet.md, vault/item_index_pages/items.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::SimChecksumState;

pub const BUY_COST: u32 = 150;
pub const LUCK_BONUS: I32F32 = I32F32::lit("0.01");

#[derive(Event)]
pub struct ToiletPurchasedEvent;

#[derive(Event)]
pub struct ToiletInteractedEvent;

#[derive(Event)]
pub struct ToiletFlushingSoundEvent;

#[derive(Resource, Default)]
pub struct ToiletState {
    pub owned: bool,
    pub flush_count: u32,
}

pub struct ToiletPlugin;

impl Plugin for ToiletPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ToiletPurchasedEvent>()
            .add_event::<ToiletInteractedEvent>()
            .add_event::<ToiletFlushingSoundEvent>()
            .init_resource::<ToiletState>()
            .add_systems(
                FixedUpdate,
                (handle_purchase, handle_interaction, toilet_checksum).chain(),
            );
    }
}

fn handle_purchase(
    mut events: EventReader<ToiletPurchasedEvent>,
    mut state: ResMut<ToiletState>,
) {
    for _ in events.read() {
        if !state.owned {
            state.owned = true;
        }
    }
}

fn handle_interaction(
    mut events: EventReader<ToiletInteractedEvent>,
    mut flushing_sounds: EventWriter<ToiletFlushingSoundEvent>,
    mut state: ResMut<ToiletState>,
) {
    for _ in events.read() {
        if state.owned {
            state.flush_count = state.flush_count.saturating_add(1);
            flushing_sounds.send(ToiletFlushingSoundEvent);
        }
    }
}

fn toilet_checksum(state: Res<ToiletState>, mut cs: ResMut<SimChecksumState>) {
    cs.accumulate(state.owned as u64);
    cs.accumulate(state.flush_count as u64);
}