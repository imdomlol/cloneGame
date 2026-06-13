// Sources: vault/store_items/table.md, vault/item_index_pages/items.md, vault/store_items/record_player.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::SimChecksumState;

pub const BUY_COST: u32 = 70;
pub const LUCK_BONUS: I32F32 = I32F32::lit("0.004");
pub const CAN_PLACE_OBJECTS_ON_SURFACE: bool = true;
pub const CAN_FUNCTION_AS_STORAGE: bool = true;
pub const STACKS_WITH_ROMANTIC_TABLE: bool = true;

#[derive(Event)]
pub struct TablePurchasedEvent;

#[derive(Event)]
pub struct TableObjectPlacedEvent {
    pub object_id: u32,
}

#[derive(Event)]
pub struct TableUsedAsStorageEvent;

#[derive(Resource, Default)]
pub struct TableState {
    pub owned: bool,
    pub orientation_offset_applied: bool,
    pub objects_on_surface: u32,
    pub used_as_storage: bool,
}

pub struct TablePlugin;

impl Plugin for TablePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TablePurchasedEvent>()
            .add_event::<TableObjectPlacedEvent>()
            .add_event::<TableUsedAsStorageEvent>()
            .init_resource::<TableState>()
            .add_systems(
                FixedUpdate,
                (
                    handle_purchase,
                    handle_surface_placement,
                    handle_storage_use,
                    table_checksum,
                )
                    .chain(),
            );
    }
}

fn handle_purchase(
    mut events: EventReader<TablePurchasedEvent>,
    mut state: ResMut<TableState>,
) {
    for _ in events.read() {
        if !state.owned {
            state.owned = true;
            state.orientation_offset_applied = true;
        }
    }
}

fn handle_surface_placement(
    mut events: EventReader<TableObjectPlacedEvent>,
    mut state: ResMut<TableState>,
) {
    for _ in events.read() {
        if state.owned {
            state.objects_on_surface = state.objects_on_surface.saturating_add(1);
        }
    }
}

fn handle_storage_use(
    mut events: EventReader<TableUsedAsStorageEvent>,
    mut state: ResMut<TableState>,
) {
    for _ in events.read() {
        if state.owned {
            state.used_as_storage = true;
        }
    }
}

fn table_checksum(state: Res<TableState>, mut cs: ResMut<SimChecksumState>) {
    cs.accumulate(state.owned as u64);
    cs.accumulate(state.orientation_offset_applied as u64);
    cs.accumulate(state.objects_on_surface as u64);
    cs.accumulate(state.used_as_storage as u64);
    cs.accumulate(LUCK_BONUS.to_bits() as u64);
}