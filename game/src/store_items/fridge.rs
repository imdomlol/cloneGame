// Sources: vault/store_items/fridge.md, vault/item_index_pages/items.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::SimChecksumState;

pub const BUY_COST: u32 = 225;
pub const LUCK_BONUS: I32F32 = I32F32::lit("0.01");
pub const STORAGE_SLOTS: usize = 2;

#[derive(Event)]
pub struct FridgePurchasedEvent;

/// Emitted when the fridge door is opened or closed (render-side audio uses this).
#[derive(Event)]
pub struct FridgeDoorToggledEvent {
    pub open: bool,
}

#[derive(Resource, Default)]
pub struct FridgeState {
    pub owned: bool,
    pub is_open: bool,
    /// Game entity ids (not Bevy Entity bits) of items stored in the two compartments.
    pub slots: [Option<u64>; STORAGE_SLOTS],
}

impl FridgeState {
    pub fn store_item(&mut self, game_entity_id: u64) -> bool {
        for slot in self.slots.iter_mut() {
            if slot.is_none() {
                *slot = Some(game_entity_id);
                return true;
            }
        }
        false
    }

    pub fn remove_item(&mut self, game_entity_id: u64) -> bool {
        for slot in self.slots.iter_mut() {
            if *slot == Some(game_entity_id) {
                *slot = None;
                return true;
            }
        }
        false
    }
}

pub struct FridgePlugin;

impl Plugin for FridgePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<FridgePurchasedEvent>()
            .add_event::<FridgeDoorToggledEvent>()
            .init_resource::<FridgeState>()
            .add_systems(
                FixedUpdate,
                (handle_purchase, handle_door_toggle, fridge_checksum).chain(),
            );
    }
}

fn handle_purchase(
    mut events: EventReader<FridgePurchasedEvent>,
    mut state: ResMut<FridgeState>,
) {
    for _ in events.read() {
        if !state.owned {
            state.owned = true;
        }
    }
}

fn handle_door_toggle(
    mut events: EventReader<FridgeDoorToggledEvent>,
    mut state: ResMut<FridgeState>,
) {
    for ev in events.read() {
        state.is_open = ev.open;
    }
}

fn fridge_checksum(state: Res<FridgeState>, mut cs: ResMut<SimChecksumState>) {
    cs.accumulate(state.owned as u64);
    cs.accumulate(state.is_open as u64);
    for slot in &state.slots {
        cs.accumulate(slot.unwrap_or(0));
    }
}