// Sources: vault/indoor_entity_pages/ghost_girl.md, vault/scrap_items/apparatus.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::entity_pages::entity::{EntityClassification, EntitySpawnCycleEvent, EntitySpawnTiming};
use crate::indoor_entity_pages::ghost_girl::{
    GHOST_GIRL_ABSENT_RETRY_SECONDS, GHOST_GIRL_INITIAL_ABSENT_DELAY_SECONDS,
};
use crate::scrap_items::apparatus::{
    ApparatusSpawnPressureRollEvent, APPARATUS_ENTITY_SPAWN_INCREASE_CHANCE_PERCENT,
    APPARATUS_MINIMUM_SPAWNED_ENTITY_COUNT_AFTER_ROLL,
};
use crate::sim::{SimChecksumState, SimHz, SimTick};
use crate::system::game_state_machine::{GameState, RequestStateChangeEvent};

pub const FACILITY_TIMER_ID: &str = "facility_timer";
pub const FACILITY_TIMER_NAME: &str = "Facility Timer";
pub const FACILITY_TIMER_TYPE: &str = "system";
pub const FACILITY_TIMER_SUBTYPE: &str = "time_pressure";

pub const FACILITY_TIMER_SOURCE_GHOST_GIRL_REVISION: u32 = 21476;
pub const FACILITY_TIMER_SOURCE_APPARATUS_REVISION: u32 = 21211;
pub const FACILITY_TIMER_MISSION_LIMIT_SECONDS: I32F32 = I32F32::lit("720");

pub const FACILITY_TIMER_RULES: [FacilityTimerRule; 6] = [
    FacilityTimerRule {
        condition: "the mission starts",
        outcome: "the timer arms the mission limit and schedules the first indoor pressure wave after the Ghost Girl absent delay",
    },
    FacilityTimerRule {
        condition: "the first indoor pressure delay elapses",
        outcome: "the timer emits indoor entity spawn-cycle pressure every Ghost Girl absent retry interval",
    },
    FacilityTimerRule {
        condition: "the Apparatus spawn-pressure roll succeeds",
        outcome: "the timer raises each due pressure wave to request at least 2 indoor spawn-cycle pulses",
    },
    FacilityTimerRule {
        condition: "the Apparatus spawn-pressure roll fails",
        outcome: "the timer records the roll and leaves the normal pressure-wave count unchanged",
    },
    FacilityTimerRule {
        condition: "the mission limit elapses",
        outcome: "the timer emits a mission-limit event and requests the lose state",
    },
    FacilityTimerRule {
        condition: "the mission has already expired",
        outcome: "the timer does not emit duplicate deadline events",
    },
];

pub struct FacilityTimerPlugin;

impl Plugin for FacilityTimerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FacilityTimerState>()
            .add_event::<FacilityTimerEntityWaveEvent>()
            .add_event::<FacilityTimerMissionLimitExpiredEvent>()
            .add_event::<FacilityTimerGhostGirlAbsentWindowEvent>()
            .add_systems(
                FixedUpdate,
                (
                    facility_timer_initialize,
                    facility_timer_absorb_apparatus_pressure,
                    facility_timer_advance,
                    facility_timer_emit_due_waves,
                    facility_timer_enforce_mission_limit,
                    facility_timer_checksum,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FacilityTimerRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum FacilityPressureStage {
    #[default]
    Low = 0,
    Medium = 1,
    High = 2,
    Deadline = 3,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum FacilityTimerTargetArea {
    #[default]
    Facility = 0,
    Ship = 1,
    OutsideFacilityAndShip = 2,
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct FacilityTimerState {
    pub initialized: bool,
    pub elapsed_ticks: u64,
    pub mission_limit_ticks: u64,
    pub next_entity_wave_tick: u64,
    pub next_ghost_girl_absent_window_tick: u64,
    pub entity_wave_interval_ticks: u64,
    pub ghost_girl_absent_retry_ticks: u64,
    pub entity_waves_emitted: u64,
    pub entity_spawn_cycle_pulses_emitted: u64,
    pub ghost_girl_absent_windows_emitted: u64,
    pub apparatus_pressure_rolls_seen: u64,
    pub apparatus_pressure_successes: u64,
    pub apparatus_pressure_active: bool,
    pub minimum_spawned_entity_count: u8,
    pub target_area: FacilityTimerTargetArea,
    pub pressure_stage: FacilityPressureStage,
    pub mission_limit_expired: bool,
}

impl Default for FacilityTimerState {
    fn default() -> Self {
        Self {
            initialized: false,
            elapsed_ticks: 0,
            mission_limit_ticks: 0,
            next_entity_wave_tick: 0,
            next_ghost_girl_absent_window_tick: 0,
            entity_wave_interval_ticks: 0,
            ghost_girl_absent_retry_ticks: 0,
            entity_waves_emitted: 0,
            entity_spawn_cycle_pulses_emitted: 0,
            ghost_girl_absent_windows_emitted: 0,
            apparatus_pressure_rolls_seen: 0,
            apparatus_pressure_successes: 0,
            apparatus_pressure_active: false,
            minimum_spawned_entity_count: 1,
            target_area: FacilityTimerTargetArea::Facility,
            pressure_stage: FacilityPressureStage::Low,
            mission_limit_expired: false,
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct FacilityTimerEntityWaveEvent {
    pub wave_index: u64,
    pub elapsed_ticks: u64,
    pub classification: EntityClassification,
    pub timing: EntitySpawnTiming,
    pub requested_spawn_cycle_pulses: u8,
    pub pressure_stage: FacilityPressureStage,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct FacilityTimerMissionLimitExpiredEvent {
    pub elapsed_ticks: u64,
    pub mission_limit_ticks: u64,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct FacilityTimerGhostGirlAbsentWindowEvent {
    pub window_index: u64,
    pub elapsed_ticks: u64,
    pub target_area: FacilityTimerTargetArea,
    pub spawn_attempt_can_select_location: bool,
}

fn facility_timer_initialize(sim_hz: Res<SimHz>, mut state: ResMut<FacilityTimerState>) {
    if state.initialized {
        return;
    }

    let initial_delay_ticks = fixed_seconds_to_ticks(GHOST_GIRL_INITIAL_ABSENT_DELAY_SECONDS, sim_hz.0);
    let retry_ticks = fixed_seconds_to_ticks(GHOST_GIRL_ABSENT_RETRY_SECONDS, sim_hz.0);

    state.initialized = true;
    state.mission_limit_ticks = fixed_seconds_to_ticks(FACILITY_TIMER_MISSION_LIMIT_SECONDS, sim_hz.0);
    state.next_entity_wave_tick = initial_delay_ticks;
    state.next_ghost_girl_absent_window_tick = initial_delay_ticks;
    state.entity_wave_interval_ticks = retry_ticks.max(1);
    state.ghost_girl_absent_retry_ticks = retry_ticks.max(1);
}

fn facility_timer_absorb_apparatus_pressure(
    mut events: EventReader<ApparatusSpawnPressureRollEvent>,
    mut state: ResMut<FacilityTimerState>,
) {
    for event in events.read() {
        state.apparatus_pressure_rolls_seen = state.apparatus_pressure_rolls_seen.wrapping_add(1);

        if !event.succeeded {
            continue;
        }

        state.apparatus_pressure_successes = state.apparatus_pressure_successes.wrapping_add(1);
        state.apparatus_pressure_active = true;
        state.minimum_spawned_entity_count = state
            .minimum_spawned_entity_count
            .max(event.minimum_spawned_entity_count)
            .max(APPARATUS_MINIMUM_SPAWNED_ENTITY_COUNT_AFTER_ROLL);
    }
}

fn facility_timer_advance(mut state: ResMut<FacilityTimerState>) {
    if !state.initialized || state.mission_limit_expired {
        return;
    }

    state.elapsed_ticks = state.elapsed_ticks.wrapping_add(1);
    state.pressure_stage = pressure_stage_for_ticks(state.elapsed_ticks, state.mission_limit_ticks);
}

fn facility_timer_emit_due_waves(
    mut state: ResMut<FacilityTimerState>,
    mut wave_events: EventWriter<FacilityTimerEntityWaveEvent>,
    mut ghost_window_events: EventWriter<FacilityTimerGhostGirlAbsentWindowEvent>,
    mut spawn_cycle_events: EventWriter<EntitySpawnCycleEvent>,
) {
    if !state.initialized || state.mission_limit_expired {
        return;
    }

    if state.elapsed_ticks >= state.next_entity_wave_tick {
        let requested_spawn_cycle_pulses = state.minimum_spawned_entity_count.max(1);
        state.entity_waves_emitted = state.entity_waves_emitted.wrapping_add(1);

        for _ in 0..requested_spawn_cycle_pulses {
            spawn_cycle_events.send(EntitySpawnCycleEvent {
                classification: EntityClassification::Indoor,
                timing: EntitySpawnTiming::IndoorDelayed,
            });
            state.entity_spawn_cycle_pulses_emitted =
                state.entity_spawn_cycle_pulses_emitted.wrapping_add(1);
        }

        wave_events.send(FacilityTimerEntityWaveEvent {
            wave_index: state.entity_waves_emitted,
            elapsed_ticks: state.elapsed_ticks,
            classification: EntityClassification::Indoor,
            timing: EntitySpawnTiming::IndoorDelayed,
            requested_spawn_cycle_pulses,
            pressure_stage: state.pressure_stage,
        });

        state.next_entity_wave_tick = state
            .elapsed_ticks
            .saturating_add(state.entity_wave_interval_ticks.max(1));
    }

    if state.elapsed_ticks >= state.next_ghost_girl_absent_window_tick {
        state.ghost_girl_absent_windows_emitted =
            state.ghost_girl_absent_windows_emitted.wrapping_add(1);

        ghost_window_events.send(FacilityTimerGhostGirlAbsentWindowEvent {
            window_index: state.ghost_girl_absent_windows_emitted,
            elapsed_ticks: state.elapsed_ticks,
            target_area: state.target_area,
            spawn_attempt_can_select_location: state.target_area
                != FacilityTimerTargetArea::OutsideFacilityAndShip,
        });

        state.next_ghost_girl_absent_window_tick = state
            .elapsed_ticks
            .saturating_add(state.ghost_girl_absent_retry_ticks.max(1));
    }
}

fn facility_timer_enforce_mission_limit(
    mut state: ResMut<FacilityTimerState>,
    mut expired_events: EventWriter<FacilityTimerMissionLimitExpiredEvent>,
    mut state_requests: EventWriter<RequestStateChangeEvent>,
) {
    if !state.initialized || state.mission_limit_expired {
        return;
    }

    if state.elapsed_ticks < state.mission_limit_ticks {
        return;
    }

    state.mission_limit_expired = true;
    state.pressure_stage = FacilityPressureStage::Deadline;

    expired_events.send(FacilityTimerMissionLimitExpiredEvent {
        elapsed_ticks: state.elapsed_ticks,
        mission_limit_ticks: state.mission_limit_ticks,
    });

    state_requests.send(RequestStateChangeEvent(GameState::Lose));
}

fn pressure_stage_for_ticks(elapsed_ticks: u64, mission_limit_ticks: u64) -> FacilityPressureStage {
    if mission_limit_ticks == 0 || elapsed_ticks >= mission_limit_ticks {
        return FacilityPressureStage::Deadline;
    }

    if elapsed_ticks.saturating_mul(4) >= mission_limit_ticks.saturating_mul(3) {
        FacilityPressureStage::High
    } else if elapsed_ticks.saturating_mul(2) >= mission_limit_ticks {
        FacilityPressureStage::Medium
    } else {
        FacilityPressureStage::Low
    }
}

fn fixed_seconds_to_ticks(seconds: I32F32, hz: I32F32) -> u64 {
    let ticks = seconds * hz;
    ticks.max(I32F32::ONE).to_num::<u64>()
}

fn facility_timer_checksum(
    tick: Res<SimTick>,
    state: Res<FacilityTimerState>,
    mut checksum: ResMut<SimChecksumState>,
) {
    checksum.accumulate(tick.0);
    accumulate_str(&mut checksum, 0x1000, FACILITY_TIMER_ID);
    accumulate_str(&mut checksum, 0x1001, FACILITY_TIMER_NAME);
    accumulate_str(&mut checksum, 0x1002, FACILITY_TIMER_TYPE);
    accumulate_str(&mut checksum, 0x1003, FACILITY_TIMER_SUBTYPE);

    checksum.accumulate(FACILITY_TIMER_SOURCE_GHOST_GIRL_REVISION as u64);
    checksum.accumulate(FACILITY_TIMER_SOURCE_APPARATUS_REVISION as u64);
    checksum.accumulate(FACILITY_TIMER_MISSION_LIMIT_SECONDS.to_bits() as u64);
    checksum.accumulate(GHOST_GIRL_INITIAL_ABSENT_DELAY_SECONDS.to_bits() as u64);
    checksum.accumulate(GHOST_GIRL_ABSENT_RETRY_SECONDS.to_bits() as u64);
    checksum.accumulate(APPARATUS_ENTITY_SPAWN_INCREASE_CHANCE_PERCENT as u64);
    checksum.accumulate(APPARATUS_MINIMUM_SPAWNED_ENTITY_COUNT_AFTER_ROLL as u64);

    for rule in FACILITY_TIMER_RULES {
        accumulate_str(&mut checksum, 0x2000, rule.condition);
        accumulate_str(&mut checksum, 0x2001, rule.outcome);
    }

    checksum.accumulate(state.initialized as u64);
    checksum.accumulate(state.elapsed_ticks);
    checksum.accumulate(state.mission_limit_ticks);
    checksum.accumulate(state.next_entity_wave_tick);
    checksum.accumulate(state.next_ghost_girl_absent_window_tick);
    checksum.accumulate(state.entity_wave_interval_ticks);
    checksum.accumulate(state.ghost_girl_absent_retry_ticks);
    checksum.accumulate(state.entity_waves_emitted);
    checksum.accumulate(state.entity_spawn_cycle_pulses_emitted);
    checksum.accumulate(state.ghost_girl_absent_windows_emitted);
    checksum.accumulate(state.apparatus_pressure_rolls_seen);
    checksum.accumulate(state.apparatus_pressure_successes);
    checksum.accumulate(state.apparatus_pressure_active as u64);
    checksum.accumulate(state.minimum_spawned_entity_count as u64);
    checksum.accumulate(target_area_code(state.target_area));
    checksum.accumulate(pressure_stage_code(state.pressure_stage));
    checksum.accumulate(state.mission_limit_expired as u64);
}

fn target_area_code(target_area: FacilityTimerTargetArea) -> u64 {
    match target_area {
        FacilityTimerTargetArea::Facility => 1,
        FacilityTimerTargetArea::Ship => 2,
        FacilityTimerTargetArea::OutsideFacilityAndShip => 3,
    }
}

fn pressure_stage_code(stage: FacilityPressureStage) -> u64 {
    match stage {
        FacilityPressureStage::Low => 1,
        FacilityPressureStage::Medium => 2,
        FacilityPressureStage::High => 3,
        FacilityPressureStage::Deadline => 4,
    }
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt ^ value.len() as u64);

    for (index, byte) in value.bytes().enumerate() {
        checksum.accumulate(salt ^ ((index as u64) << 8) ^ byte as u64);
    }
}