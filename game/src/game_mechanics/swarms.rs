// Sources: vault/game_mechanics/swarms.md, vault/organizations/the_new_empire.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::SimChecksumState;

const ZERO: I32F32 = I32F32::ZERO;
const ONE_HUNDRED: I32F32 = I32F32::lit("100");
const HOURS_BIG_SWARM_WARNING: I32F32 = I32F32::lit("8");
const HOURS_FINAL_SWARM_WARNING: I32F32 = I32F32::lit("24");
const BIG_SWARMS_PER_SURVIVAL_MISSION: u32 = 10;
const RAIDERS_START_DAY_SURVIVAL: u32 = 20;
const RAIDERS_START_DAY_NEW_EMPIRE: u32 = 12;
const GUARANTEED_VOD_RAIDER_INTERVAL_DAYS: u32 = 5;
const EARLY_WALKER_GROUP_DAY: u32 = 4;
const LONELY_FOREST_FINAL_SWARM_DAY: u32 = 40;
const FINAL_FORTIFICATION_LAYERS_MIN: u32 = 2;
const FINAL_FORTIFICATION_LAYERS_MAX: u32 = 3;

#[derive(Component, Clone, Copy, Default)]
pub struct SwarmTargetsCommandCenter;

#[derive(Component, Clone, Copy, Default)]
pub struct SwarmNeverSpawnsFromCorner;

#[derive(Component, Clone, Copy, Default)]
pub struct SwarmGroup {
    pub runner_count: u32,
    pub special_count: u32,
    pub redirected: bool,
    pub split_fronts: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SwarmDifficulty {
    Easiest,
    Easy,
    Accessible,
    Challenging,
    Brutal,
}

impl Default for SwarmDifficulty {
    fn default() -> Self {
        Self::Challenging
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SwarmType {
    Normal,
    BigAnnounced,
    RaiderUnannounced,
    FinalAllEdges,
}

impl Default for SwarmType {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Event, Clone, Copy, Default)]
pub struct SwarmWarningIssuedEvent {
    pub swarm_type: SwarmType,
    pub warning_hours_before_spawn: I32F32,
    pub minimap_yellow_skull: bool,
}

#[derive(Event, Clone, Copy, Default)]
pub struct SwarmSpawnedEvent {
    pub swarm_type: SwarmType,
    pub base_population: u32,
    pub difficulty: SwarmDifficulty,
    pub map_is_survival: bool,
    pub map_is_new_empire: bool,
    pub map_is_villages_of_doom: bool,
    pub map_is_the_resistance: bool,
    pub map_is_cape_storm: bool,
    pub day_number: u32,
    pub noise_triggered_raider_event: bool,
    pub structure_infected_during_final: bool,
    pub infected_structure_worker_count: u32,
}

#[derive(Resource, Clone, Copy)]
pub struct SwarmsMechanicData {
    pub id: &'static str,
    pub name: &'static str,
    pub mechanic_type: &'static str,
    pub how_it_works: &'static str,
    pub techniques: &'static str,
    pub applications: &'static str,
    pub depends_on: &'static [&'static str],
}

impl Default for SwarmsMechanicData {
    fn default() -> Self {
        Self {
            id: "swarms",
            name: "Swarms",
            mechanic_type: "enemy_wave_system",
            how_it_works: "Swarms are map-edge attack waves that path toward the colony's main base, scale with difficulty, and include announced big waves, unannounced raiders, and a final all-edges assault.",
            techniques: "Use layered perimeter defenses, keep all directions covered, clear corners early, and reserve warehouses, walls, and traps for the final defense.",
            applications: "This mechanic controls colony pressure in survival missions and applies to edge raids on campaign maps and Village of Doom scenarios.",
            depends_on: &[
                "command_center",
                "infected",
                "survival_maps",
                "the_new_empire",
                "infected_behemoth",
                "infected_harpy",
                "infected_venom",
                "infected_chubby",
                "villages_of_doom",
                "the_resistance",
                "cape_storm",
                "stone_wall",
                "stakes_trap",
                "wire_fence_trap",
                "warehouse",
            ],
        }
    }
}

#[derive(Resource, Clone, Copy)]
pub struct SwarmRules {
    pub big_swarm_warning_hours: I32F32,
    pub final_swarm_warning_hours: I32F32,
    pub big_swarms_per_survival_mission: u32,
    pub easiest_population_percent: I32F32,
    pub easy_population_percent: I32F32,
    pub accessible_population_percent: I32F32,
    pub challenging_population_percent: I32F32,
    pub brutal_population_percent: I32F32,
    pub raiders_start_day_survival: u32,
    pub raiders_start_day_new_empire: u32,
    pub guaranteed_vod_raider_interval_days: u32,
    pub early_walker_group_day: u32,
    pub lonely_forest_final_swarm_day: u32,
    pub final_fortification_layers_min: u32,
    pub final_fortification_layers_max: u32,
}

impl Default for SwarmRules {
    fn default() -> Self {
        Self {
            big_swarm_warning_hours: HOURS_BIG_SWARM_WARNING,
            final_swarm_warning_hours: HOURS_FINAL_SWARM_WARNING,
            big_swarms_per_survival_mission: BIG_SWARMS_PER_SURVIVAL_MISSION,
            easiest_population_percent: I32F32::lit("20"),
            easy_population_percent: I32F32::lit("50"),
            accessible_population_percent: I32F32::lit("80"),
            challenging_population_percent: I32F32::lit("100"),
            brutal_population_percent: I32F32::lit("120"),
            raiders_start_day_survival: RAIDERS_START_DAY_SURVIVAL,
            raiders_start_day_new_empire: RAIDERS_START_DAY_NEW_EMPIRE,
            guaranteed_vod_raider_interval_days: GUARANTEED_VOD_RAIDER_INTERVAL_DAYS,
            early_walker_group_day: EARLY_WALKER_GROUP_DAY,
            lonely_forest_final_swarm_day: LONELY_FOREST_FINAL_SWARM_DAY,
            final_fortification_layers_min: FINAL_FORTIFICATION_LAYERS_MIN,
            final_fortification_layers_max: FINAL_FORTIFICATION_LAYERS_MAX,
        }
    }
}

#[derive(Resource, Clone, Copy)]
pub struct SwarmRuntimeState {
    pub total_waves_spawned: u64,
    pub announced_big_waves_spawned: u32,
    pub raider_waves_spawned: u32,
    pub final_wave_spawned: bool,
    pub total_spawned_units_scaled: u64,
    pub total_structure_spawned_units: u64,
    pub warning_events_seen: u64,
    pub yellow_skull_warnings_seen: u64,
    pub redirected_waves_seen: u64,
    pub split_front_waves_seen: u64,
    pub never_corner_spawn_rule_active: bool,
}

impl Default for SwarmRuntimeState {
    fn default() -> Self {
        Self {
            total_waves_spawned: 0,
            announced_big_waves_spawned: 0,
            raider_waves_spawned: 0,
            final_wave_spawned: false,
            total_spawned_units_scaled: 0,
            total_structure_spawned_units: 0,
            warning_events_seen: 0,
            yellow_skull_warnings_seen: 0,
            redirected_waves_seen: 0,
            split_front_waves_seen: 0,
            never_corner_spawn_rule_active: true,
        }
    }
}

fn population_percent_for_difficulty(rules: &SwarmRules, difficulty: SwarmDifficulty) -> I32F32 {
    match difficulty {
        SwarmDifficulty::Easiest => rules.easiest_population_percent,
        SwarmDifficulty::Easy => rules.easy_population_percent,
        SwarmDifficulty::Accessible => rules.accessible_population_percent,
        SwarmDifficulty::Challenging => rules.challenging_population_percent,
        SwarmDifficulty::Brutal => rules.brutal_population_percent,
    }
}

fn scaled_population(base_population: u32, percent: I32F32) -> u32 {
    let base_fp = I32F32::from_num(base_population);
    let scaled_fp = (base_fp * percent) / ONE_HUNDRED;
    if scaled_fp <= ZERO {
        0
    } else {
        scaled_fp.to_num::<u32>()
    }
}

fn apply_swarm_warning_system(
    rules: Res<SwarmRules>,
    mut runtime: ResMut<SwarmRuntimeState>,
    mut warnings: EventReader<SwarmWarningIssuedEvent>,
) {
    for warning in warnings.read() {
        runtime.warning_events_seen = runtime.warning_events_seen.saturating_add(1);

        if warning.minimap_yellow_skull {
            runtime.yellow_skull_warnings_seen = runtime.yellow_skull_warnings_seen.saturating_add(1);
        }

        let expected_hours = match warning.swarm_type {
            SwarmType::BigAnnounced => rules.big_swarm_warning_hours,
            SwarmType::FinalAllEdges => rules.final_swarm_warning_hours,
            SwarmType::Normal | SwarmType::RaiderUnannounced => ZERO,
        };

        if expected_hours > ZERO && warning.warning_hours_before_spawn > expected_hours {
            runtime.redirected_waves_seen = runtime.redirected_waves_seen.saturating_add(1);
        }
    }
}

fn apply_swarm_spawns_system(
    rules: Res<SwarmRules>,
    mut runtime: ResMut<SwarmRuntimeState>,
    mut spawns: EventReader<SwarmSpawnedEvent>,
    mut swarm_groups: Query<&mut SwarmGroup>,
) {
    for spawn in spawns.read() {
        runtime.total_waves_spawned = runtime.total_waves_spawned.saturating_add(1);

        let percent = population_percent_for_difficulty(&rules, spawn.difficulty);
        let mut total_scaled = scaled_population(spawn.base_population, percent) as u64;

        match spawn.swarm_type {
            SwarmType::Normal => {}
            SwarmType::BigAnnounced => {
                if spawn.map_is_survival
                    && runtime.announced_big_waves_spawned < rules.big_swarms_per_survival_mission
                {
                    runtime.announced_big_waves_spawned =
                        runtime.announced_big_waves_spawned.saturating_add(1);
                }
            }
            SwarmType::RaiderUnannounced => {
                let survival_raider_ok =
                    spawn.map_is_survival && spawn.day_number >= rules.raiders_start_day_survival;
                let empire_raider_ok =
                    spawn.map_is_new_empire && spawn.day_number >= rules.raiders_start_day_new_empire;
                let vod_raider_ok = spawn.map_is_villages_of_doom
                    && spawn.day_number >= rules.raiders_start_day_survival
                    && ((spawn.day_number - rules.raiders_start_day_survival)
                        % rules.guaranteed_vod_raider_interval_days
                        == 0);

                if survival_raider_ok || empire_raider_ok || vod_raider_ok || spawn.noise_triggered_raider_event {
                    runtime.raider_waves_spawned = runtime.raider_waves_spawned.saturating_add(1);
                }
            }
            SwarmType::FinalAllEdges => {
                runtime.final_wave_spawned = true;

                if spawn.structure_infected_during_final {
                    total_scaled = total_scaled.saturating_add(spawn.infected_structure_worker_count as u64);
                    runtime.total_structure_spawned_units = runtime
                        .total_structure_spawned_units
                        .saturating_add(spawn.infected_structure_worker_count as u64);
                }
            }
        }

        if spawn.map_is_the_resistance || spawn.map_is_cape_storm {
            if spawn.day_number >= rules.early_walker_group_day {
                runtime.split_front_waves_seen = runtime.split_front_waves_seen.saturating_add(1);
            }
        }

        runtime.total_spawned_units_scaled =
            runtime.total_spawned_units_scaled.saturating_add(total_scaled);

        for mut group in &mut swarm_groups {
            if group.redirected {
                runtime.redirected_waves_seen = runtime.redirected_waves_seen.saturating_add(1);
            }
            if group.split_fronts {
                runtime.split_front_waves_seen = runtime.split_front_waves_seen.saturating_add(1);
            }
        }
    }
}

fn swarms_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    data: Res<SwarmsMechanicData>,
    rules: Res<SwarmRules>,
    runtime: Res<SwarmRuntimeState>,
    groups: Query<&SwarmGroup>,
) {
    checksum.accumulate(data.id.len() as u64);
    checksum.accumulate(data.name.len() as u64);
    checksum.accumulate(data.mechanic_type.len() as u64);
    checksum.accumulate(data.how_it_works.len() as u64);
    checksum.accumulate(data.techniques.len() as u64);
    checksum.accumulate(data.applications.len() as u64);
    checksum.accumulate(data.depends_on.len() as u64);

    checksum.accumulate(rules.big_swarm_warning_hours.to_bits() as u64);
    checksum.accumulate(rules.final_swarm_warning_hours.to_bits() as u64);
    checksum.accumulate(rules.big_swarms_per_survival_mission as u64);
    checksum.accumulate(rules.easiest_population_percent.to_bits() as u64);
    checksum.accumulate(rules.easy_population_percent.to_bits() as u64);
    checksum.accumulate(rules.accessible_population_percent.to_bits() as u64);
    checksum.accumulate(rules.challenging_population_percent.to_bits() as u64);
    checksum.accumulate(rules.brutal_population_percent.to_bits() as u64);
    checksum.accumulate(rules.raiders_start_day_survival as u64);
    checksum.accumulate(rules.raiders_start_day_new_empire as u64);
    checksum.accumulate(rules.guaranteed_vod_raider_interval_days as u64);
    checksum.accumulate(rules.early_walker_group_day as u64);
    checksum.accumulate(rules.lonely_forest_final_swarm_day as u64);
    checksum.accumulate(rules.final_fortification_layers_min as u64);
    checksum.accumulate(rules.final_fortification_layers_max as u64);

    checksum.accumulate(runtime.total_waves_spawned);
    checksum.accumulate(runtime.announced_big_waves_spawned as u64);
    checksum.accumulate(runtime.raider_waves_spawned as u64);
    checksum.accumulate(u64::from(runtime.final_wave_spawned));
    checksum.accumulate(runtime.total_spawned_units_scaled);
    checksum.accumulate(runtime.total_structure_spawned_units);
    checksum.accumulate(runtime.warning_events_seen);
    checksum.accumulate(runtime.yellow_skull_warnings_seen);
    checksum.accumulate(runtime.redirected_waves_seen);
    checksum.accumulate(runtime.split_front_waves_seen);
    checksum.accumulate(u64::from(runtime.never_corner_spawn_rule_active));

    for group in &groups {
        checksum.accumulate(group.runner_count as u64);
        checksum.accumulate(group.special_count as u64);
        checksum.accumulate(u64::from(group.redirected));
        checksum.accumulate(u64::from(group.split_fronts));
    }
}

pub struct SwarmsPlugin;

impl Plugin for SwarmsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SwarmsMechanicData>()
            .init_resource::<SwarmRules>()
            .init_resource::<SwarmRuntimeState>()
            .add_event::<SwarmWarningIssuedEvent>()
            .add_event::<SwarmSpawnedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    apply_swarm_warning_system,
                    apply_swarm_spawns_system,
                    swarms_checksum_system,
                )
                    .chain(),
            );
    }
}