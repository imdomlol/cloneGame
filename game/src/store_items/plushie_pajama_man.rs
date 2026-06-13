// Sources: vault/store_items/plushie_pajama_man.md, vault/item_index_pages/decor.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::SimChecksumState;

pub const BUY_COST: u32 = 100;
pub const LUCK_BONUS: I32F32 = I32F32::lit("0.003");

#[derive(Event)]
pub struct PlushiePajamaManPurchasedEvent;

/// Fired when a player interacts with the plushie. Does not propagate as
/// NoiseEmittedEvent — the squeak explicitly does not attract eyeless_dog.
#[derive(Event)]
pub struct PlushiePajamaManSqueakedEvent;

#[derive(Resource, Default)]
pub struct PlushiePajamaManState {
    pub owned: bool,
}

pub struct PlushiePajamaManPlugin;

impl Plugin for PlushiePajamaManPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlushiePajamaManPurchasedEvent>()
            .add_event::<PlushiePajamaManSqueakedEvent>()
            .init_resource::<PlushiePajamaManState>()
            .add_systems(
                FixedUpdate,
                (handle_purchase, handle_squeak, plushie_pajama_man_checksum).chain(),
            );
    }
}

fn handle_purchase(
    mut events: EventReader<PlushiePajamaManPurchasedEvent>,
    mut state: ResMut<PlushiePajamaManState>,
) {
    for _ in events.read() {
        if !state.owned {
            state.owned = true;
        }
    }
}

fn handle_squeak(
    mut events: EventReader<PlushiePajamaManSqueakedEvent>,
    state: Res<PlushiePajamaManState>,
) {
    if !state.owned {
        events.clear();
        return;
    }
    // Consume without emitting NoiseEmittedEvent — squeak never alerts eyeless_dog.
    for _ in events.read() {}
}

fn plushie_pajama_man_checksum(
    state: Res<PlushiePajamaManState>,
    mut cs: ResMut<SimChecksumState>,
) {
    cs.accumulate(state.owned as u64);
}