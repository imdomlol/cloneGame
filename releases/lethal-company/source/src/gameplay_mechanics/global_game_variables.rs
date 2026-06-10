// Sources: vault/gameplay_mechanics/global_game_variables.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{SimChecksumState, SimTick};

pub const GLOBAL_GAME_VARIABLES_ID: &str = "global_game_variables";
pub const GLOBAL_GAME_VARIABLES_NAME: &str = "Global Game Variables";
pub const GLOBAL_GAME_VARIABLES_TYPE: &str = "gameplay_mechanics";
pub const GLOBAL_GAME_VARIABLES_SUBTYPE: &str = "global_variables";
pub const GLOBAL_GAME_VARIABLES_SOURCE_URL: &str =
    "https://lethal-company.fandom.com/wiki/Mechanics";
pub const GLOBAL_GAME_VARIABLES_SOURCE_REVISION: u32 = 19089;
pub const GLOBAL_GAME_VARIABLES_EXTRACTED_AT: &str = "2026-06-07T00:00:00Z";
pub const GLOBAL_GAME_VARIABLES_CONFIDENCE_BASIS_POINTS: u16 = 82;

pub const GLOBAL_GAME_VARIABLES_HOUR_LENGTH: u32 = 60;
pub const GLOBAL_GAME_VARIABLES_HOUR_COUNT: u32 = 18;
pub const GLOBAL_GAME_VARIABLES_TIME_SPEED_MULTIPLIER_NUMERATOR: i64 = 14;
pub const GLOBAL_GAME_VARIABLES_TIME_SPEED_MULTIPLIER_DENOMINATOR: i64 = 10;
pub const GLOBAL_GAME_VARIABLES_SHIP_LEAVE_TIME: &str = "12 AM";
pub const GLOBAL_GAME_VARIABLES_SCRAP_VALUE_MULTIPLIER_NUMERATOR: i64 = 4;
pub const GLOBAL_GAME_VARIABLES_SCRAP_VALUE_MULTIPLIER_DENOMINATOR: i64 = 10;
pub const GLOBAL_GAME_VARIABLES_SCRAP_AMOUNT_MULTIPLIER_NUMERATOR: i64 = 10;
pub const GLOBAL_GAME_VARIABLES_SCRAP_AMOUNT_MULTIPLIER_DENOMINATOR: i64 = 10;
pub const GLOBAL_GAME_VARIABLES_MAP_SIZE_MULTIPLIER_NUMERATOR: i64 = 15;
pub const GLOBAL_GAME_VARIABLES_MAP_SIZE_MULTIPLIER_DENOMINATOR: i64 = 10;
pub const GLOBAL_GAME_VARIABLES_SPAWNING_COOLDOWN_HOURS: u32 = 2;

pub const GLOBAL_GAME_VARIABLES_DEPENDS_ON: [&str; 2] = ["entity", "scrap"];

pub const GLOBAL_GAME_VARIABLES_RULES: [&str; 3] = [
    "Variables are static for every player and do not change during a round.",
    "Normalized time is computed as `current_time / (hour_length * hour_count)`.",
    "Spawn-cycle timing is driven by `spawning_cooldown`.",
];

pub const GLOBAL_GAME_VARIABLES_BEHAVIORAL_MECHANICS: [GlobalGameVariablesBehaviorRule; 7] = [
    GlobalGameVariablesBehaviorRule {
        condition: "time is converted into normalized form",
        outcome: "divide current time by `hour_length * hour_count`, where `hour_length` is 60 and `hour_count` is 18",
    },
    GlobalGameVariablesBehaviorRule {
        condition: "game speed is applied",
        outcome: "use `time_speed_multiplier` at 1.4 to scale elapsed time behavior",
    },
    GlobalGameVariablesBehaviorRule {
        condition: "the ship departure cutoff is checked",
        outcome: "use `ship_leave_time` at 12 AM as the leave threshold",
    },
    GlobalGameVariablesBehaviorRule {
        condition: "scrap value is calculated",
        outcome: "multiply base scrap value by `scrap_value_multiplier` at 0.4",
    },
    GlobalGameVariablesBehaviorRule {
        condition: "scrap quantity is calculated",
        outcome: "multiply base scrap amount by `scrap_amount_multiplier` at 1.0",
    },
    GlobalGameVariablesBehaviorRule {
        condition: "map scale is calculated",
        outcome: "multiply base map size by `map_size_multiplier` at 1.5",
    },
    GlobalGameVariablesBehaviorRule {
        condition: "spawning cycles are scheduled",
        outcome: "repeat them every `spawning_cooldown` of 2 hours",
    },
];

pub struct GlobalGameVariablesPlugin;

impl Plugin for GlobalGameVariablesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GlobalGameVariablesState>()
            .add_systems(FixedUpdate, global_game_variables_checksum);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GlobalGameVariablesBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct GlobalGameVariablesState {
    pub hour_length: u32,
    pub hour_count: u32,
    pub time_speed_multiplier: I32F32,
    pub ship_leave_time: &'static str,
    pub scrap_value_multiplier: I32F32,
    pub scrap_amount_multiplier: I32F32,
    pub map_size_multiplier: I32F32,
    pub spawning_cooldown_hours: u32,
}

impl Default for GlobalGameVariablesState {
    fn default() -> Self {
        Self {
            hour_length: GLOBAL_GAME_VARIABLES_HOUR_LENGTH,
            hour_count: GLOBAL_GAME_VARIABLES_HOUR_COUNT,
            time_speed_multiplier: global_game_variables_time_speed_multiplier(),
            ship_leave_time: GLOBAL_GAME_VARIABLES_SHIP_LEAVE_TIME,
            scrap_value_multiplier: global_game_variables_scrap_value_multiplier(),
            scrap_amount_multiplier: global_game_variables_scrap_amount_multiplier(),
            map_size_multiplier: global_game_variables_map_size_multiplier(),
            spawning_cooldown_hours: GLOBAL_GAME_VARIABLES_SPAWNING_COOLDOWN_HOURS,
        }
    }
}

pub fn global_game_variables_total_round_time() -> u32 {
    GLOBAL_GAME_VARIABLES_HOUR_LENGTH * GLOBAL_GAME_VARIABLES_HOUR_COUNT
}

pub fn global_game_variables_time_speed_multiplier() -> I32F32 {
    fixed_ratio(
        GLOBAL_GAME_VARIABLES_TIME_SPEED_MULTIPLIER_NUMERATOR,
        GLOBAL_GAME_VARIABLES_TIME_SPEED_MULTIPLIER_DENOMINATOR,
    )
}

pub fn global_game_variables_scrap_value_multiplier() -> I32F32 {
    fixed_ratio(
        GLOBAL_GAME_VARIABLES_SCRAP_VALUE_MULTIPLIER_NUMERATOR,
        GLOBAL_GAME_VARIABLES_SCRAP_VALUE_MULTIPLIER_DENOMINATOR,
    )
}

pub fn global_game_variables_scrap_amount_multiplier() -> I32F32 {
    fixed_ratio(
        GLOBAL_GAME_VARIABLES_SCRAP_AMOUNT_MULTIPLIER_NUMERATOR,
        GLOBAL_GAME_VARIABLES_SCRAP_AMOUNT_MULTIPLIER_DENOMINATOR,
    )
}

pub fn global_game_variables_map_size_multiplier() -> I32F32 {
    fixed_ratio(
        GLOBAL_GAME_VARIABLES_MAP_SIZE_MULTIPLIER_NUMERATOR,
        GLOBAL_GAME_VARIABLES_MAP_SIZE_MULTIPLIER_DENOMINATOR,
    )
}

pub fn global_game_variables_normalized_time(current_time: I32F32) -> I32F32 {
    current_time / I32F32::from_num(global_game_variables_total_round_time())
}

pub fn global_game_variables_apply_game_speed(elapsed_time: I32F32) -> I32F32 {
    elapsed_time * global_game_variables_time_speed_multiplier()
}

pub fn global_game_variables_apply_scrap_value_multiplier(base_scrap_value: I32F32) -> I32F32 {
    base_scrap_value * global_game_variables_scrap_value_multiplier()
}

pub fn global_game_variables_apply_scrap_amount_multiplier(base_scrap_amount: I32F32) -> I32F32 {
    base_scrap_amount * global_game_variables_scrap_amount_multiplier()
}

pub fn global_game_variables_apply_map_size_multiplier(base_map_size: I32F32) -> I32F32 {
    base_map_size * global_game_variables_map_size_multiplier()
}

fn fixed_ratio(numerator: i64, denominator: i64) -> I32F32 {
    I32F32::from_num(numerator) / I32F32::from_num(denominator)
}

fn global_game_variables_checksum(
    mut checksum: ResMut<SimChecksumState>,
    tick: Res<SimTick>,
    state: Res<GlobalGameVariablesState>,
) {
    checksum.accumulate(tick.0);
    checksum.accumulate(GLOBAL_GAME_VARIABLES_SOURCE_REVISION as u64);
    checksum.accumulate(GLOBAL_GAME_VARIABLES_CONFIDENCE_BASIS_POINTS as u64);

    for dependency in GLOBAL_GAME_VARIABLES_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x1000, dependency);
    }

    for rule in GLOBAL_GAME_VARIABLES_RULES {
        accumulate_str(&mut checksum, 0x2000, rule);
    }

    for rule in GLOBAL_GAME_VARIABLES_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x3000, rule.condition);
        accumulate_str(&mut checksum, 0x3001, rule.outcome);
    }

    checksum.accumulate(state.hour_length as u64);
    checksum.accumulate(state.hour_count as u64);
    checksum.accumulate(state.time_speed_multiplier.to_bits() as u64);
    accumulate_str(&mut checksum, 0x4000, state.ship_leave_time);
    checksum.accumulate(state.scrap_value_multiplier.to_bits() as u64);
    checksum.accumulate(state.scrap_amount_multiplier.to_bits() as u64);
    checksum.accumulate(state.map_size_multiplier.to_bits() as u64);
    checksum.accumulate(state.spawning_cooldown_hours as u64);
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt ^ value.len() as u64);

    for (index, byte) in value.bytes().enumerate() {
        checksum.accumulate(salt ^ ((index as u64) << 8) ^ byte as u64);
    }
}