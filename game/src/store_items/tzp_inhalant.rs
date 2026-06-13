// Sources: vault/store_items/tzp_inhalant.md

use bevy::prelude::*;
use fixed::types::I32F32;
use std::collections::BTreeMap;

use crate::sim::SimChecksumState;

pub const BUY_COST: u32 = 80;
pub const SELL_VALUE: u32 = 0;
pub const WEIGHT_LB: I32F32 = I32F32::lit("0");
pub const CONDUCTIVE: bool = true;
pub const TOTAL_USE_TICKS: u32 = 22 * 20;
pub const HIGH_PITCHED_VOICE_TICKS: u32 = 4 * 20;
pub const LIGHT_THRESHOLD_TICKS: u32 = 4 * 20;
pub const HEAVY_THRESHOLD_TICKS: u32 = 8 * 20;
pub const OVERDOSE_THRESHOLD_TICKS: u32 = 12 * 20;
pub const LIGHT_BUFF_TICKS: u32 = 6 * 20;
pub const LIGHT_DEBUFF_TICKS: u32 = 12 * 20;
pub const HEAVY_BUFF_TICKS: u32 = 24 * 20;
pub const HEAVY_DEBUFF_TICKS: u32 = 36 * 20;
pub const OVERDOSE_BUFF_TICKS: u32 = 36 * 20;
pub const OVERDOSE_DEBUFF_TICKS: u32 = 42 * 20;

pub const LIGHT_SPEED_BOOST: I32F32 = I32F32::lit("0.05");
pub const HEAVY_SPEED_BOOST: I32F32 = I32F32::lit("0.10");
pub const OVERDOSE_SPEED_BOOST: I32F32 = I32F32::lit("0.15");
pub const LIGHT_STAMINA_CONSUMPTION_MULTIPLIER: I32F32 = I32F32::lit("0.85");
pub const HEAVY_STAMINA_CONSUMPTION_MULTIPLIER: I32F32 = I32F32::lit("0.70");
pub const OVERDOSE_STAMINA_CONSUMPTION_MULTIPLIER: I32F32 = I32F32::lit("0.55");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TzpInhalantEffectState {
    None,
    Light,
    Heavy,
    Overdose,
}

impl Default for TzpInhalantEffectState {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct TzpInhalantUserState {
    pub total_use_ticks: u32,
    pub current_inhale_ticks: u32,
    pub buff_ticks_remaining: u32,
    pub debuff_ticks_remaining: u32,
    pub high_pitched_voice: bool,
    pub minor_visual_impairment: bool,
    pub visual_impairment: bool,
    pub heavy_visual_and_auditory_impairment: bool,
    pub slippery_controls: bool,
    pub screen_distortion: bool,
    pub depleted: bool,
    pub effect_state: TzpInhalantEffectState,
    pub speed_boost: I32F32,
    pub stamina_consumption_multiplier: I32F32,
}

#[derive(Event)]
pub struct TzpInhalantPurchasedEvent;

#[derive(Event)]
pub struct TzpInhalantInhaledEvent {
    pub player_id: u64,
    pub inhale_ticks: u32,
}

#[derive(Event)]
pub struct TzpInhalantStoppedEvent {
    pub player_id: u64,
}

#[derive(Event)]
pub struct TzpInhalantHighPitchedVoiceEvent {
    pub player_id: u64,
}

#[derive(Event)]
pub struct TzpInhalantEffectAppliedEvent {
    pub player_id: u64,
    pub state: TzpInhalantEffectState,
    pub buff_ticks: u32,
    pub debuff_ticks: u32,
}

#[derive(Event)]
pub struct TzpInhalantDepletedEvent {
    pub player_id: u64,
}

#[derive(Resource, Default)]
pub struct TzpInhalantState {
    pub owned: u32,
    pub users: BTreeMap<u64, TzpInhalantUserState>,
}

pub struct TzpInhalantPlugin;

impl Plugin for TzpInhalantPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TzpInhalantPurchasedEvent>()
            .add_event::<TzpInhalantInhaledEvent>()
            .add_event::<TzpInhalantStoppedEvent>()
            .add_event::<TzpInhalantHighPitchedVoiceEvent>()
            .add_event::<TzpInhalantEffectAppliedEvent>()
            .add_event::<TzpInhalantDepletedEvent>()
            .init_resource::<TzpInhalantState>()
            .add_systems(
                FixedUpdate,
                (
                    handle_purchase,
                    handle_inhaled,
                    handle_stopped,
                    tick_tzp_effects,
                    tzp_inhalant_checksum,
                )
                    .chain(),
            );
    }
}

fn handle_purchase(
    mut events: EventReader<TzpInhalantPurchasedEvent>,
    mut state: ResMut<TzpInhalantState>,
) {
    for _ in events.read() {
        state.owned = state.owned.saturating_add(1);
    }
}

fn handle_inhaled(
    mut events: EventReader<TzpInhalantInhaledEvent>,
    mut state: ResMut<TzpInhalantState>,
    mut voice_events: EventWriter<TzpInhalantHighPitchedVoiceEvent>,
    mut applied_events: EventWriter<TzpInhalantEffectAppliedEvent>,
    mut depleted_events: EventWriter<TzpInhalantDepletedEvent>,
) {
    for event in events.read() {
        let user = state.users.entry(event.player_id).or_default();

        if user.depleted {
            continue;
        }

        let previous_total = user.total_use_ticks;
        let previous_inhale = user.current_inhale_ticks;
        user.current_inhale_ticks = user.current_inhale_ticks.saturating_add(event.inhale_ticks);
        user.total_use_ticks = user.total_use_ticks.saturating_add(event.inhale_ticks);

        if previous_inhale < HIGH_PITCHED_VOICE_TICKS
            && user.current_inhale_ticks >= HIGH_PITCHED_VOICE_TICKS
        {
            user.high_pitched_voice = true;
            voice_events.send(TzpInhalantHighPitchedVoiceEvent {
                player_id: event.player_id,
            });
        }

        if previous_inhale < LIGHT_THRESHOLD_TICKS && user.current_inhale_ticks >= LIGHT_THRESHOLD_TICKS
        {
            apply_tzp_state(
                event.player_id,
                user,
                TzpInhalantEffectState::Light,
                &mut applied_events,
            );
        }

        if previous_inhale < HEAVY_THRESHOLD_TICKS && user.current_inhale_ticks >= HEAVY_THRESHOLD_TICKS
        {
            apply_tzp_state(
                event.player_id,
                user,
                TzpInhalantEffectState::Heavy,
                &mut applied_events,
            );
        }

        if previous_inhale < OVERDOSE_THRESHOLD_TICKS
            && user.current_inhale_ticks >= OVERDOSE_THRESHOLD_TICKS
        {
            apply_tzp_state(
                event.player_id,
                user,
                TzpInhalantEffectState::Overdose,
                &mut applied_events,
            );
        }

        if previous_total < TOTAL_USE_TICKS && user.total_use_ticks >= TOTAL_USE_TICKS {
            user.total_use_ticks = TOTAL_USE_TICKS;
            user.current_inhale_ticks = 0;
            user.depleted = true;
            depleted_events.send(TzpInhalantDepletedEvent {
                player_id: event.player_id,
            });
        }
    }
}

fn handle_stopped(
    mut events: EventReader<TzpInhalantStoppedEvent>,
    mut state: ResMut<TzpInhalantState>,
) {
    for event in events.read() {
        if let Some(user) = state.users.get_mut(&event.player_id) {
            user.current_inhale_ticks = 0;
        }
    }
}

fn tick_tzp_effects(mut state: ResMut<TzpInhalantState>) {
    for user in state.users.values_mut() {
        user.buff_ticks_remaining = user.buff_ticks_remaining.saturating_sub(1);
        user.debuff_ticks_remaining = user.debuff_ticks_remaining.saturating_sub(1);

        if user.buff_ticks_remaining == 0 {
            user.speed_boost = I32F32::ZERO;
            user.stamina_consumption_multiplier = I32F32::ONE;
        }

        if user.debuff_ticks_remaining == 0 {
            user.high_pitched_voice = false;
            user.minor_visual_impairment = false;
            user.visual_impairment = false;
            user.heavy_visual_and_auditory_impairment = false;
            user.slippery_controls = false;
            user.screen_distortion = false;
            user.effect_state = TzpInhalantEffectState::None;
        }
    }
}

fn apply_tzp_state(
    player_id: u64,
    user: &mut TzpInhalantUserState,
    effect_state: TzpInhalantEffectState,
    applied_events: &mut EventWriter<TzpInhalantEffectAppliedEvent>,
) {
    user.effect_state = effect_state;
    user.high_pitched_voice = true;

    match effect_state {
        TzpInhalantEffectState::None => {}
        TzpInhalantEffectState::Light => {
            user.buff_ticks_remaining = LIGHT_BUFF_TICKS;
            user.debuff_ticks_remaining = LIGHT_DEBUFF_TICKS;
            user.minor_visual_impairment = true;
            user.speed_boost = LIGHT_SPEED_BOOST;
            user.stamina_consumption_multiplier = LIGHT_STAMINA_CONSUMPTION_MULTIPLIER;
        }
        TzpInhalantEffectState::Heavy => {
            user.buff_ticks_remaining = HEAVY_BUFF_TICKS;
            user.debuff_ticks_remaining = HEAVY_DEBUFF_TICKS;
            user.visual_impairment = true;
            user.slippery_controls = true;
            user.speed_boost = HEAVY_SPEED_BOOST;
            user.stamina_consumption_multiplier = HEAVY_STAMINA_CONSUMPTION_MULTIPLIER;
        }
        TzpInhalantEffectState::Overdose => {
            user.buff_ticks_remaining = OVERDOSE_BUFF_TICKS;
            user.debuff_ticks_remaining = OVERDOSE_DEBUFF_TICKS;
            user.heavy_visual_and_auditory_impairment = true;
            user.slippery_controls = true;
            user.screen_distortion = true;
            user.speed_boost = OVERDOSE_SPEED_BOOST;
            user.stamina_consumption_multiplier = OVERDOSE_STAMINA_CONSUMPTION_MULTIPLIER;
        }
    }

    applied_events.send(TzpInhalantEffectAppliedEvent {
        player_id,
        state: effect_state,
        buff_ticks: user.buff_ticks_remaining,
        debuff_ticks: user.debuff_ticks_remaining,
    });
}

fn tzp_inhalant_checksum(state: Res<TzpInhalantState>, mut cs: ResMut<SimChecksumState>) {
    cs.accumulate(state.owned as u64);

    for (player_id, user) in state.users.iter() {
        cs.accumulate(*player_id);
        cs.accumulate(user.total_use_ticks as u64);
        cs.accumulate(user.current_inhale_ticks as u64);
        cs.accumulate(user.buff_ticks_remaining as u64);
        cs.accumulate(user.debuff_ticks_remaining as u64);
        cs.accumulate(user.high_pitched_voice as u64);
        cs.accumulate(user.minor_visual_impairment as u64);
        cs.accumulate(user.visual_impairment as u64);
        cs.accumulate(user.heavy_visual_and_auditory_impairment as u64);
        cs.accumulate(user.slippery_controls as u64);
        cs.accumulate(user.screen_distortion as u64);
        cs.accumulate(user.depleted as u64);
        cs.accumulate(user.effect_state as u64);
        cs.accumulate(user.speed_boost.to_bits() as u64);
        cs.accumulate(user.stamina_consumption_multiplier.to_bits() as u64);
    }
}