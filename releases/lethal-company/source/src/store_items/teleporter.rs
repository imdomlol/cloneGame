// Sources: vault/store_items/teleporter.md

use bevy::prelude::*;

use crate::gameplay_mechanics::the_ship::TheShipTeleporterUsedEvent;
use crate::sim::{SimChecksumState, SimHz};

pub const BUY_COST: u32 = 375;
pub const COOLDOWN_SECONDS: u32 = 10;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TeleporterTargetCondition {
    Alive,
    PossessedByMasked,
    ConsumedByEarthLeviathan,
    ConsumedByForestKeeper,
    ConsumedByCompanyMonster,
    SuffocatedInQuicksand,
    DrownedOn71Gordion,
    CarriedByBaboonHawk,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TeleporterFailureReason {
    NotOwned,
    Recharging,
    ConsumedByEarthLeviathan,
    ConsumedByForestKeeper,
    ConsumedByCompanyMonster,
    SuffocatedInQuicksand,
    DrownedOn71Gordion,
    CarriedByBaboonHawk,
}

#[derive(Event)]
pub struct TeleporterPurchasedEvent;

#[derive(Event)]
pub struct TeleporterActivateEvent {
    pub monitored_employee_id: u64,
    pub target_condition: TeleporterTargetCondition,
}

#[derive(Event)]
pub struct TeleporterHeldItemsDroppedEvent {
    pub employee_id: u64,
}

#[derive(Event)]
pub struct TeleporterTeleportSucceededEvent {
    pub employee_id: u64,
}

#[derive(Event)]
pub struct TeleporterTeleportFailedEvent {
    pub employee_id: u64,
    pub reason: TeleporterFailureReason,
}

#[derive(Resource, Default)]
pub struct TeleporterState {
    pub owned: bool,
    pub cooldown_ticks_remaining: u32,
    pub successful_teleports: u64,
    pub failed_teleports: u64,
    pub dropped_held_item_batches: u64,
    pub last_target_employee_id: u64,
    pub last_failure_reason: Option<TeleporterFailureReason>,
}

pub struct TeleporterPlugin;

impl Plugin for TeleporterPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TeleporterPurchasedEvent>()
            .add_event::<TeleporterActivateEvent>()
            .add_event::<TeleporterHeldItemsDroppedEvent>()
            .add_event::<TeleporterTeleportSucceededEvent>()
            .add_event::<TeleporterTeleportFailedEvent>()
            .init_resource::<TeleporterState>()
            .add_systems(
                FixedUpdate,
                (
                    handle_purchase,
                    tick_cooldown,
                    handle_activation,
                    teleporter_checksum,
                )
                    .chain(),
            );
    }
}

fn handle_purchase(
    mut events: EventReader<TeleporterPurchasedEvent>,
    mut state: ResMut<TeleporterState>,
) {
    for _ in events.read() {
        if !state.owned {
            state.owned = true;
        }
    }
}

fn tick_cooldown(mut state: ResMut<TeleporterState>) {
    if state.cooldown_ticks_remaining > 0 {
        state.cooldown_ticks_remaining -= 1;
    }
}

fn handle_activation(
    mut activations: EventReader<TeleporterActivateEvent>,
    mut state: ResMut<TeleporterState>,
    mut dropped: EventWriter<TeleporterHeldItemsDroppedEvent>,
    mut succeeded: EventWriter<TeleporterTeleportSucceededEvent>,
    mut failed: EventWriter<TeleporterTeleportFailedEvent>,
    mut ship_used: EventWriter<TheShipTeleporterUsedEvent>,
    sim_hz: Res<SimHz>,
) {
    for event in activations.read() {
        state.last_target_employee_id = event.monitored_employee_id;

        if !state.owned {
            emit_failure(
                &mut state,
                &mut failed,
                event.monitored_employee_id,
                TeleporterFailureReason::NotOwned,
            );
            continue;
        }

        if state.cooldown_ticks_remaining > 0 {
            emit_failure(
                &mut state,
                &mut failed,
                event.monitored_employee_id,
                TeleporterFailureReason::Recharging,
            );
            continue;
        }

        state.cooldown_ticks_remaining = COOLDOWN_SECONDS * sim_hz.0.to_num::<u32>();

        if let Some(reason) = failure_reason(event.target_condition) {
            emit_failure(&mut state, &mut failed, event.monitored_employee_id, reason);
            continue;
        }

        state.dropped_held_item_batches = state.dropped_held_item_batches.wrapping_add(1);
        state.successful_teleports = state.successful_teleports.wrapping_add(1);
        state.last_failure_reason = None;

        dropped.send(TeleporterHeldItemsDroppedEvent {
            employee_id: event.monitored_employee_id,
        });
        succeeded.send(TeleporterTeleportSucceededEvent {
            employee_id: event.monitored_employee_id,
        });
        ship_used.send(TheShipTeleporterUsedEvent {
            monitored_employee_id: event.monitored_employee_id,
        });
    }
}

fn failure_reason(condition: TeleporterTargetCondition) -> Option<TeleporterFailureReason> {
    match condition {
        TeleporterTargetCondition::Alive | TeleporterTargetCondition::PossessedByMasked => None,
        TeleporterTargetCondition::ConsumedByEarthLeviathan => {
            Some(TeleporterFailureReason::ConsumedByEarthLeviathan)
        }
        TeleporterTargetCondition::ConsumedByForestKeeper => {
            Some(TeleporterFailureReason::ConsumedByForestKeeper)
        }
        TeleporterTargetCondition::ConsumedByCompanyMonster => {
            Some(TeleporterFailureReason::ConsumedByCompanyMonster)
        }
        TeleporterTargetCondition::SuffocatedInQuicksand => {
            Some(TeleporterFailureReason::SuffocatedInQuicksand)
        }
        TeleporterTargetCondition::DrownedOn71Gordion => {
            Some(TeleporterFailureReason::DrownedOn71Gordion)
        }
        TeleporterTargetCondition::CarriedByBaboonHawk => {
            Some(TeleporterFailureReason::CarriedByBaboonHawk)
        }
    }
}

fn emit_failure(
    state: &mut TeleporterState,
    failed: &mut EventWriter<TeleporterTeleportFailedEvent>,
    employee_id: u64,
    reason: TeleporterFailureReason,
) {
    state.failed_teleports = state.failed_teleports.wrapping_add(1);
    state.last_failure_reason = Some(reason);
    failed.send(TeleporterTeleportFailedEvent {
        employee_id,
        reason,
    });
}

fn teleporter_checksum(state: Res<TeleporterState>, mut cs: ResMut<SimChecksumState>) {
    cs.accumulate(state.owned as u64);
    cs.accumulate(state.cooldown_ticks_remaining as u64);
    cs.accumulate(state.successful_teleports);
    cs.accumulate(state.failed_teleports);
    cs.accumulate(state.dropped_held_item_batches);
    cs.accumulate(state.last_target_employee_id);
    cs.accumulate(failure_reason_bits(state.last_failure_reason));
}

fn failure_reason_bits(reason: Option<TeleporterFailureReason>) -> u64 {
    match reason {
        None => 0,
        Some(TeleporterFailureReason::NotOwned) => 1,
        Some(TeleporterFailureReason::Recharging) => 2,
        Some(TeleporterFailureReason::ConsumedByEarthLeviathan) => 3,
        Some(TeleporterFailureReason::ConsumedByForestKeeper) => 4,
        Some(TeleporterFailureReason::ConsumedByCompanyMonster) => 5,
        Some(TeleporterFailureReason::SuffocatedInQuicksand) => 6,
        Some(TeleporterFailureReason::DrownedOn71Gordion) => 7,
        Some(TeleporterFailureReason::CarriedByBaboonHawk) => 8,
    }
}