// Sources: vault/game_mechanics/time.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{SimChecksumState, SimHz};

const ZERO: I32F32 = I32F32::ZERO;
const ONE: I32F32 = I32F32::lit("1");
const HOURS_PER_DAY: I32F32 = I32F32::lit("24");
const ONE_HOUR_REAL_TIME_SECONDS: I32F32 = I32F32::lit("3.75");
const ONE_DAY_REAL_TIME_SECONDS: I32F32 = I32F32::lit("90");
const TICK_DURATION_HOURS: I32F32 = I32F32::lit("8");
const TICK_DURATION_REAL_TIME_SECONDS: I32F32 = I32F32::lit("30");
const FINAL_WAVE_BRUTAL_DAYS: I32F32 = I32F32::lit("72");
const FINAL_WAVE_BRUTAL_REAL_TIME_SECONDS: I32F32 = I32F32::lit("6480");

#[derive(Resource, Clone, Copy)]
pub struct TimeMechanicData {
    pub id: &'static str,
    pub name: &'static str,
    pub mechanic_type: &'static str,
    pub how_it_works: &'static str,
    pub techniques: &'static str,
    pub applications: &'static str,
    pub depends_on: &'static [&'static str],
    pub one_hour_real_time_seconds: I32F32,
    pub one_day_real_time_seconds: I32F32,
    pub tick_duration_hours: I32F32,
    pub tick_duration_real_time_seconds: I32F32,
    pub final_wave_brutal_days: I32F32,
    pub final_wave_brutal_real_time_seconds: I32F32,
}

impl Default for TimeMechanicData {
    fn default() -> Self {
        Self {
            id: "time",
            name: "Time",
            mechanic_type: "time",
            how_it_works: "In they_are_billions, time advances in discrete day and hour units. One in-game hour equals 3.75 real-life seconds assuming no lag, one in-game day equals 90 real-life seconds, and one tick equals 8 in-game hours or 30 real-life seconds.",
            techniques: "Use fast_timers_optimization to preserve the tab-time to real-time ratio on slower computers, but expect possible pathfinding issues, ignored orders, and other unusual behavior.",
            applications: "Time governs resource production, maintenance checks, and survival-wave timing, including the 72-day brutal final-wave timing window.",
            depends_on: &["they_are_billions", "fast_timers_optimization"],
            one_hour_real_time_seconds: ONE_HOUR_REAL_TIME_SECONDS,
            one_day_real_time_seconds: ONE_DAY_REAL_TIME_SECONDS,
            tick_duration_hours: TICK_DURATION_HOURS,
            tick_duration_real_time_seconds: TICK_DURATION_REAL_TIME_SECONDS,
            final_wave_brutal_days: FINAL_WAVE_BRUTAL_DAYS,
            final_wave_brutal_real_time_seconds: FINAL_WAVE_BRUTAL_REAL_TIME_SECONDS,
        }
    }
}

#[derive(Resource, Clone, Copy)]
pub struct TimeState {
    pub elapsed_real_time_seconds: I32F32,
    pub elapsed_in_game_hours: I32F32,
    pub elapsed_in_game_days: I32F32,
    pub tick_progress_real_time_seconds: I32F32,
    pub completed_ticks: u64,
    pub brutal_final_wave_reached: bool,
    pub fast_timers_optimization_enabled: bool,
    pub may_show_pathfinding_issues: bool,
    pub may_ignore_orders: bool,
    pub may_show_unusual_behavior: bool,
}

impl Default for TimeState {
    fn default() -> Self {
        Self {
            elapsed_real_time_seconds: ZERO,
            elapsed_in_game_hours: ZERO,
            elapsed_in_game_days: ZERO,
            tick_progress_real_time_seconds: ZERO,
            completed_ticks: 0,
            brutal_final_wave_reached: false,
            fast_timers_optimization_enabled: true,
            may_show_pathfinding_issues: false,
            may_ignore_orders: false,
            may_show_unusual_behavior: false,
        }
    }
}

#[derive(Resource, Clone, Copy, Default)]
pub struct TimeMetrics {
    pub total_tick_pulses: u64,
    pub total_resource_or_maintenance_pulses: u64,
    pub ratio_preserved_steps: u64,
    pub ratio_unpreserved_steps: u64,
}

#[derive(Event, Clone, Copy)]
pub struct SetFastTimersOptimizationEvent {
    pub enabled: bool,
}

#[derive(Event, Clone, Copy)]
pub struct TimeTickCompletedEvent {
    pub tick_index: u64,
}

fn apply_fast_timers_optimization_setting_system(
    mut time_state: ResMut<TimeState>,
    mut events: EventReader<SetFastTimersOptimizationEvent>,
) {
    for ev in events.read() {
        time_state.fast_timers_optimization_enabled = ev.enabled;
        if ev.enabled {
            time_state.may_show_pathfinding_issues = false;
            time_state.may_ignore_orders = false;
            time_state.may_show_unusual_behavior = false;
        } else {
            time_state.may_show_pathfinding_issues = true;
            time_state.may_ignore_orders = true;
            time_state.may_show_unusual_behavior = true;
        }
    }
}

fn advance_time_system(
    sim_hz: Res<SimHz>,
    mut time_state: ResMut<TimeState>,
    mut metrics: ResMut<TimeMetrics>,
    mut tick_writer: EventWriter<TimeTickCompletedEvent>,
) {
    if sim_hz.0 <= ZERO {
        return;
    }

    let delta_real_seconds = ONE / sim_hz.0;

    time_state.elapsed_real_time_seconds = time_state.elapsed_real_time_seconds + delta_real_seconds;
    time_state.tick_progress_real_time_seconds =
        time_state.tick_progress_real_time_seconds + delta_real_seconds;

    if time_state.fast_timers_optimization_enabled {
        metrics.ratio_preserved_steps = metrics.ratio_preserved_steps.saturating_add(1);
    } else {
        metrics.ratio_unpreserved_steps = metrics.ratio_unpreserved_steps.saturating_add(1);
    }

    while time_state.tick_progress_real_time_seconds >= TICK_DURATION_REAL_TIME_SECONDS {
        time_state.tick_progress_real_time_seconds =
            time_state.tick_progress_real_time_seconds - TICK_DURATION_REAL_TIME_SECONDS;
        time_state.completed_ticks = time_state.completed_ticks.saturating_add(1);
        time_state.elapsed_in_game_hours = time_state.elapsed_in_game_hours + TICK_DURATION_HOURS;
        time_state.elapsed_in_game_days = time_state.elapsed_in_game_hours / HOURS_PER_DAY;

        metrics.total_tick_pulses = metrics.total_tick_pulses.saturating_add(1);
        metrics.total_resource_or_maintenance_pulses =
            metrics.total_resource_or_maintenance_pulses.saturating_add(1);

        tick_writer.send(TimeTickCompletedEvent {
            tick_index: time_state.completed_ticks,
        });

        if !time_state.brutal_final_wave_reached
            && time_state.elapsed_in_game_days >= FINAL_WAVE_BRUTAL_DAYS
        {
            time_state.brutal_final_wave_reached = true;
        }
    }
}

fn time_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    data: Res<TimeMechanicData>,
    state: Res<TimeState>,
    metrics: Res<TimeMetrics>,
) {
    checksum.accumulate(data.one_hour_real_time_seconds.to_bits() as u64);
    checksum.accumulate(data.one_day_real_time_seconds.to_bits() as u64);
    checksum.accumulate(data.tick_duration_hours.to_bits() as u64);
    checksum.accumulate(data.tick_duration_real_time_seconds.to_bits() as u64);
    checksum.accumulate(data.final_wave_brutal_days.to_bits() as u64);
    checksum.accumulate(data.final_wave_brutal_real_time_seconds.to_bits() as u64);

    checksum.accumulate(state.elapsed_real_time_seconds.to_bits() as u64);
    checksum.accumulate(state.elapsed_in_game_hours.to_bits() as u64);
    checksum.accumulate(state.elapsed_in_game_days.to_bits() as u64);
    checksum.accumulate(state.tick_progress_real_time_seconds.to_bits() as u64);
    checksum.accumulate(state.completed_ticks);
    checksum.accumulate(u64::from(state.brutal_final_wave_reached));
    checksum.accumulate(u64::from(state.fast_timers_optimization_enabled));
    checksum.accumulate(u64::from(state.may_show_pathfinding_issues));
    checksum.accumulate(u64::from(state.may_ignore_orders));
    checksum.accumulate(u64::from(state.may_show_unusual_behavior));

    checksum.accumulate(metrics.total_tick_pulses);
    checksum.accumulate(metrics.total_resource_or_maintenance_pulses);
    checksum.accumulate(metrics.ratio_preserved_steps);
    checksum.accumulate(metrics.ratio_unpreserved_steps);
}

pub struct TimeMechanicPlugin;

impl Plugin for TimeMechanicPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TimeMechanicData>()
            .init_resource::<TimeState>()
            .init_resource::<TimeMetrics>()
            .add_event::<SetFastTimersOptimizationEvent>()
            .add_event::<TimeTickCompletedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    apply_fast_timers_optimization_setting_system,
                    advance_time_system,
                    time_checksum_system,
                )
                    .chain(),
            );
    }
}