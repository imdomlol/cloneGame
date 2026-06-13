// Sources: vault/store_items/cozy_lights.md, vault/item_index_pages/decor.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::gameplay_mechanics::the_ship::TheShipLightSwitchToggledEvent;
use crate::sim::SimChecksumState;

pub const BUY_COST: u32 = 140;
pub const LUCK_BONUS: I32F32 = I32F32::lit("0.005");

#[derive(Event)]
pub struct CozyLightsPurchasedEvent;

#[derive(Resource, Default)]
pub struct CozyLightsState {
    pub owned: bool,
    pub active: bool,
}

pub struct CozyLightsPlugin;

impl Plugin for CozyLightsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CozyLightsPurchasedEvent>()
            .init_resource::<CozyLightsState>()
            .add_systems(
                FixedUpdate,
                (handle_purchase, handle_ship_light_toggle, cozy_lights_checksum).chain(),
            );
    }
}

fn handle_purchase(
    mut events: EventReader<CozyLightsPurchasedEvent>,
    mut state: ResMut<CozyLightsState>,
) {
    for _ in events.read() {
        if !state.owned {
            state.owned = true;
            state.active = true;
        }
    }
}

fn handle_ship_light_toggle(
    mut events: EventReader<TheShipLightSwitchToggledEvent>,
    mut state: ResMut<CozyLightsState>,
    mut ship_lights_on: Local<bool>,
) {
    for _ in events.read() {
        *ship_lights_on = !*ship_lights_on;
        state.active = state.owned && !*ship_lights_on;
    }
}

fn cozy_lights_checksum(state: Res<CozyLightsState>, mut cs: ResMut<SimChecksumState>) {
    cs.accumulate(state.owned as u64);
    cs.accumulate(state.active as u64);
}