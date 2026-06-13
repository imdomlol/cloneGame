// Sources: vault/store_items/dog_house.md, vault/item_index_pages/decor.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::SimChecksumState;

pub const BUY_COST: u32 = 80;
pub const LUCK_BONUS: I32F32 = I32F32::lit("0.007");

#[derive(Event)]
pub struct DogHousePurchasedEvent;

#[derive(Resource, Default)]
pub struct DogHouseState {
    pub owned: bool,
}

pub struct DogHousePlugin;

impl Plugin for DogHousePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DogHousePurchasedEvent>()
            .init_resource::<DogHouseState>()
            .add_systems(
                FixedUpdate,
                (handle_purchase, dog_house_checksum).chain(),
            );
    }
}

fn handle_purchase(
    mut events: EventReader<DogHousePurchasedEvent>,
    mut state: ResMut<DogHouseState>,
) {
    for _ in events.read() {
        if !state.owned {
            state.owned = true;
        }
    }
}

fn dog_house_checksum(state: Res<DogHouseState>, mut cs: ResMut<SimChecksumState>) {
    cs.accumulate(state.owned as u64);
}