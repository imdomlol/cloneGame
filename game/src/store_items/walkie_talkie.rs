// Sources: vault/store_items/walkie_talkie.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::SimChecksumState;

pub const BUY_COST: u32 = 10;
pub const SELL_VALUE: u32 = 0;
pub const WEIGHT: u32 = 0;
pub const BATTERY_LIFE_MINUTES: I32F32 = I32F32::lit("13.6667");
pub const BATTERY_LIFE_TICKS: u32 = 16_400;
pub const CONDUCTIVE: bool = false;

#[derive(Event)]
pub struct WalkieTalkiePurchasedEvent;

#[derive(Event)]
pub struct WalkieTalkieToggledEvent {
    pub player_id: u64,
}

#[derive(Event)]
pub struct WalkieTalkieTransmitStartedEvent {
    pub player_id: u64,
}

#[derive(Event)]
pub struct WalkieTalkieTransmitStoppedEvent {
    pub player_id: u64,
}

#[derive(Event)]
pub struct WalkieTalkieTextBroadcastEvent {
    pub sender_id: u64,
}

#[derive(Event)]
pub struct WalkieTalkieZoomEvent {
    pub player_id: u64,
    pub active: bool,
}

#[derive(Event)]
pub struct WalkieTalkieAudioCutOutEvent {
    pub player_id: u64,
}

#[derive(Event)]
pub struct WalkieTalkieAudibleAlertEvent {
    pub player_id: u64,
}

#[derive(Resource)]
pub struct WalkieTalkieState {
    pub owned: bool,
    pub powered_on: bool,
    pub transmitting: bool,
    pub zooming: bool,
    pub battery_ticks_remaining: u32,
    pub last_user_id: u64,
}

impl Default for WalkieTalkieState {
    fn default() -> Self {
        Self {
            owned: false,
            powered_on: false,
            transmitting: false,
            zooming: false,
            battery_ticks_remaining: BATTERY_LIFE_TICKS,
            last_user_id: 0,
        }
    }
}

impl WalkieTalkieState {
    pub fn battery_fraction(&self) -> I32F32 {
        if BATTERY_LIFE_TICKS == 0 {
            return I32F32::ZERO;
        }

        I32F32::from_num(self.battery_ticks_remaining) / I32F32::from_num(BATTERY_LIFE_TICKS)
    }

    pub fn can_transmit(&self) -> bool {
        self.owned && self.powered_on && self.battery_ticks_remaining > 0
    }
}

pub struct WalkieTalkiePlugin;

impl Plugin for WalkieTalkiePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<WalkieTalkiePurchasedEvent>()
            .add_event::<WalkieTalkieToggledEvent>()
            .add_event::<WalkieTalkieTransmitStartedEvent>()
            .add_event::<WalkieTalkieTransmitStoppedEvent>()
            .add_event::<WalkieTalkieTextBroadcastEvent>()
            .add_event::<WalkieTalkieZoomEvent>()
            .add_event::<WalkieTalkieAudioCutOutEvent>()
            .add_event::<WalkieTalkieAudibleAlertEvent>()
            .init_resource::<WalkieTalkieState>()
            .add_systems(
                FixedUpdate,
                (
                    handle_purchase,
                    handle_toggle,
                    handle_transmit_start,
                    handle_transmit_stop,
                    handle_zoom,
                    handle_text_broadcast,
                    handle_audio_cut_out,
                    drain_battery,
                    walkie_talkie_checksum,
                )
                    .chain(),
            );
    }
}

fn handle_purchase(
    mut events: EventReader<WalkieTalkiePurchasedEvent>,
    mut state: ResMut<WalkieTalkieState>,
) {
    for _ in events.read() {
        if !state.owned {
            state.owned = true;
            state.battery_ticks_remaining = BATTERY_LIFE_TICKS;
        }
    }
}

fn handle_toggle(
    mut events: EventReader<WalkieTalkieToggledEvent>,
    mut alerts: EventWriter<WalkieTalkieAudibleAlertEvent>,
    mut state: ResMut<WalkieTalkieState>,
) {
    for event in events.read() {
        if state.owned && state.battery_ticks_remaining > 0 {
            state.powered_on = !state.powered_on;
            state.last_user_id = event.player_id;

            if !state.powered_on {
                state.transmitting = false;
            }

            alerts.send(WalkieTalkieAudibleAlertEvent {
                player_id: event.player_id,
            });
        }
    }
}

fn handle_transmit_start(
    mut events: EventReader<WalkieTalkieTransmitStartedEvent>,
    mut alerts: EventWriter<WalkieTalkieAudibleAlertEvent>,
    mut state: ResMut<WalkieTalkieState>,
) {
    for event in events.read() {
        if state.can_transmit() {
            state.transmitting = true;
            state.last_user_id = event.player_id;
            alerts.send(WalkieTalkieAudibleAlertEvent {
                player_id: event.player_id,
            });
        }
    }
}

fn handle_transmit_stop(
    mut events: EventReader<WalkieTalkieTransmitStoppedEvent>,
    mut state: ResMut<WalkieTalkieState>,
) {
    for event in events.read() {
        if state.transmitting && state.last_user_id == event.player_id {
            state.transmitting = false;
        }
    }
}

fn handle_zoom(
    mut events: EventReader<WalkieTalkieZoomEvent>,
    mut state: ResMut<WalkieTalkieState>,
) {
    for event in events.read() {
        if state.owned {
            state.zooming = event.active;
            state.last_user_id = event.player_id;
        }
    }
}

fn handle_text_broadcast(
    mut events: EventReader<WalkieTalkieTextBroadcastEvent>,
    mut state: ResMut<WalkieTalkieState>,
) {
    for event in events.read() {
        if state.can_transmit() {
            state.last_user_id = event.sender_id;
        }
    }
}

fn handle_audio_cut_out(
    mut events: EventReader<WalkieTalkieAudioCutOutEvent>,
    mut state: ResMut<WalkieTalkieState>,
) {
    for event in events.read() {
        if state.transmitting && state.last_user_id == event.player_id {
            state.transmitting = false;
        }
    }
}

fn drain_battery(mut state: ResMut<WalkieTalkieState>) {
    if state.powered_on && state.battery_ticks_remaining > 0 {
        state.battery_ticks_remaining -= 1;

        if state.battery_ticks_remaining == 0 {
            state.powered_on = false;
            state.transmitting = false;
            state.zooming = false;
        }
    }
}

fn walkie_talkie_checksum(state: Res<WalkieTalkieState>, mut cs: ResMut<SimChecksumState>) {
    cs.accumulate(state.owned as u64);
    cs.accumulate(state.powered_on as u64);
    cs.accumulate(state.transmitting as u64);
    cs.accumulate(state.zooming as u64);
    cs.accumulate(state.battery_ticks_remaining as u64);
    cs.accumulate(state.last_user_id);
}