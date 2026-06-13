// Sources: vault/store_items/shower.md, vault/item_index_pages/items.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::SimChecksumState;

pub const BUY_COST: u32 = 180;
pub const LUCK_BONUS: I32F32 = I32F32::lit("0.015");

#[derive(Event)]
pub struct ShowerPurchasedEvent;

#[derive(Event)]
pub struct ShowerHeadInteractedEvent;

#[derive(Event)]
pub struct ShowerWaterSoundEffectEvent;

#[derive(Event)]
pub struct ShowerWashedPaintedPlayerEvent {
    pub player_id: u64,
}

#[derive(Event)]
pub struct ShowerSprayPaintRemovedEvent {
    pub player_id: u64,
}

#[derive(Event)]
pub struct ShowerGhostGirlSightlineEvent {
    pub player_id: u64,
    pub ghost_girl_id: u64,
}

#[derive(Event)]
pub struct ShowerGhostGirlSolidOcclusionEvent {
    pub player_id: u64,
    pub ghost_girl_id: u64,
}

#[derive(Resource, Default)]
pub struct ShowerState {
    pub owned: bool,
    pub water_running: bool,
}

pub struct ShowerPlugin;

impl Plugin for ShowerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ShowerPurchasedEvent>()
            .add_event::<ShowerHeadInteractedEvent>()
            .add_event::<ShowerWaterSoundEffectEvent>()
            .add_event::<ShowerWashedPaintedPlayerEvent>()
            .add_event::<ShowerSprayPaintRemovedEvent>()
            .add_event::<ShowerGhostGirlSightlineEvent>()
            .add_event::<ShowerGhostGirlSolidOcclusionEvent>()
            .init_resource::<ShowerState>()
            .add_systems(
                FixedUpdate,
                (
                    handle_purchase,
                    handle_shower_head_interaction,
                    handle_painted_player_wash,
                    handle_ghost_girl_sightline,
                    shower_checksum,
                )
                    .chain(),
            );
    }
}

fn handle_purchase(
    mut events: EventReader<ShowerPurchasedEvent>,
    mut state: ResMut<ShowerState>,
) {
    for _ in events.read() {
        if !state.owned {
            state.owned = true;
        }
    }
}

fn handle_shower_head_interaction(
    mut events: EventReader<ShowerHeadInteractedEvent>,
    mut water_sounds: EventWriter<ShowerWaterSoundEffectEvent>,
    mut state: ResMut<ShowerState>,
) {
    for _ in events.read() {
        if state.owned {
            state.water_running = true;
            water_sounds.send(ShowerWaterSoundEffectEvent);
        }
    }
}

fn handle_painted_player_wash(
    mut events: EventReader<ShowerWashedPaintedPlayerEvent>,
    mut removed: EventWriter<ShowerSprayPaintRemovedEvent>,
    state: Res<ShowerState>,
) {
    for event in events.read() {
        if state.owned && state.water_running {
            removed.send(ShowerSprayPaintRemovedEvent {
                player_id: event.player_id,
            });
        }
    }
}

fn handle_ghost_girl_sightline(
    mut events: EventReader<ShowerGhostGirlSightlineEvent>,
    mut occluded: EventWriter<ShowerGhostGirlSolidOcclusionEvent>,
    state: Res<ShowerState>,
) {
    for event in events.read() {
        if state.owned {
            occluded.send(ShowerGhostGirlSolidOcclusionEvent {
                player_id: event.player_id,
                ghost_girl_id: event.ghost_girl_id,
            });
        }
    }
}

fn shower_checksum(state: Res<ShowerState>, mut cs: ResMut<SimChecksumState>) {
    cs.accumulate(state.owned as u64);
    cs.accumulate(state.water_running as u64);
}