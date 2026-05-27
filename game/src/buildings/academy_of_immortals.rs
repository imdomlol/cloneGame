// Sources: vault/wonders/academy_of_immortals.md

use std::collections::BTreeMap;

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{Health, SimChecksumState, UnitStats};
use crate::units::{ranger, sniper, soldier};

const ACADEMY_HP: I32F32 = I32F32::lit("3000");
const ACADEMY_DEFENSES_LIFE: I32F32 = I32F32::lit("1500");
const ACADEMY_WATCH_RANGE: I32F32 = I32F32::lit("16");
const ACADEMY_BUILD_TIME_SECONDS: i32 = 0;
const ACADEMY_SIZE_TILES: i32 = 4;
const ACADEMY_VICTORY_POINTS_REWARD: i32 = 1000;

const ACADEMY_ENERGY_COST: i32 = 50;
const ACADEMY_WORKERS_COST: i32 = 50;
const ACADEMY_GOLD_MAINTENANCE: i32 = 120;

const ACADEMY_WOOD_COST: i32 = 200;
const ACADEMY_STONE_COST: i32 = 200;
const ACADEMY_IRON_COST: i32 = 100;
const ACADEMY_OIL_COST: i32 = 0;
const ACADEMY_GOLD_COST: i32 = 7000;

const ACADEMY_APPLIES_TO_RANGER: bool = true;
const ACADEMY_APPLIES_TO_SOLDIER: bool = true;
const ACADEMY_APPLIES_TO_SNIPER: bool = true;

const RANGER_VETERAN_ATTACK_RANGE: I32F32 = I32F32::lit("6.5");
const RANGER_VETERAN_ATTACK_SPEED: I32F32 = I32F32::lit("2");
const RANGER_VETERAN_ATTACK_DAMAGE: I32F32 = I32F32::lit("12");

const SOLDIER_VETERAN_ATTACK_RANGE: I32F32 = I32F32::lit("5.5");
const SOLDIER_VETERAN_ATTACK_SPEED: I32F32 = I32F32::lit("2.5");
const SOLDIER_VETERAN_ATTACK_DAMAGE: I32F32 = I32F32::lit("26");

const SNIPER_VETERAN_ATTACK_RANGE: I32F32 = I32F32::lit("8");
const SNIPER_VETERAN_ATTACK_SPEED: I32F32 = I32F32::lit("0.91");
const SNIPER_VETERAN_ATTACK_DAMAGE: I32F32 = I32F32::lit("110");

const SIM_HZ: i32 = 25;

#[derive(Component, Default)]
pub struct AcademyOfImmortals;

#[derive(Component, Clone, Copy, Default)]
pub struct BuildingAnchor {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Clone, Copy)]
pub struct AcademyCoreStats {
    pub defenses_life: I32F32,
    pub watch_range: I32F32,
    pub health: Health,
}

impl Default for AcademyCoreStats {
    fn default() -> Self {
        Self {
            defenses_life: ACADEMY_DEFENSES_LIFE,
            watch_range: ACADEMY_WATCH_RANGE,
            health: Health::full(ACADEMY_HP),
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct AcademyBuildState {
    pub build_ticks_remaining: i32,
    pub completed: bool,
}

#[derive(Component, Clone, Copy)]
pub struct AcademyEconomy {
    pub energy_cost: i32,
    pub workers_cost: i32,
    pub gold_maintenance: i32,
    pub wood_cost: i32,
    pub stone_cost: i32,
    pub iron_cost: i32,
    pub oil_cost: i32,
    pub gold_cost: i32,
}

impl Default for AcademyEconomy {
    fn default() -> Self {
        Self {
            energy_cost: ACADEMY_ENERGY_COST,
            workers_cost: ACADEMY_WORKERS_COST,
            gold_maintenance: ACADEMY_GOLD_MAINTENANCE,
            wood_cost: ACADEMY_WOOD_COST,
            stone_cost: ACADEMY_STONE_COST,
            iron_cost: ACADEMY_IRON_COST,
            oil_cost: ACADEMY_OIL_COST,
            gold_cost: ACADEMY_GOLD_COST,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct AcademyFootprint {
    pub size_tiles: i32,
    pub watch_range: I32F32,
}

impl Default for AcademyFootprint {
    fn default() -> Self {
        Self {
            size_tiles: ACADEMY_SIZE_TILES,
            watch_range: ACADEMY_WATCH_RANGE,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct AcademyState {
    pub connected_to_grid: bool,
    pub supplied_energy: bool,
    pub supplied_workers: bool,
    pub supplied_gold_maintenance: bool,
    pub destroyed: bool,
    pub demolished: bool,
    pub research_tier_stone_workshop_met: bool,
}

impl Default for AcademyState {
    fn default() -> Self {
        Self {
            connected_to_grid: true,
            supplied_energy: true,
            supplied_workers: true,
            supplied_gold_maintenance: true,
            destroyed: false,
            demolished: false,
            research_tier_stone_workshop_met: true,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct AcademyEffects {
    pub applies_to_ranger: bool,
    pub applies_to_soldier: bool,
    pub applies_to_sniper: bool,
    pub victory_points_reward: i32,
}

impl Default for AcademyEffects {
    fn default() -> Self {
        Self {
            applies_to_ranger: ACADEMY_APPLIES_TO_RANGER,
            applies_to_soldier: ACADEMY_APPLIES_TO_SOLDIER,
            applies_to_sniper: ACADEMY_APPLIES_TO_SNIPER,
            victory_points_reward: ACADEMY_VICTORY_POINTS_REWARD,
        }
    }
}

#[derive(Resource, Default, Clone)]
pub struct AcademyPlacementClaims {
    pub claims: BTreeMap<Entity, BuildingAnchor>,
}

#[derive(Event, Clone, Copy)]
pub struct PlaceAcademyOfImmortalsEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct SetAcademyGridConnectionEvent {
    pub entity: Entity,
    pub connected: bool,
}

#[derive(Event, Clone, Copy)]
pub struct SetAcademySupplyStateEvent {
    pub entity: Entity,
    pub supplied_energy: bool,
    pub supplied_workers: bool,
    pub supplied_gold_maintenance: bool,
}

#[derive(Event, Clone, Copy)]
pub struct SetAcademyResearchTierEvent {
    pub entity: Entity,
    pub stone_workshop_met: bool,
}

#[derive(Event, Clone, Copy)]
pub struct DemolishAcademyEvent {
    pub entity: Entity,
}

#[derive(Event, Clone, Copy)]
pub struct AttackDestroyAcademyEvent {
    pub entity: Entity,
}

fn seconds_to_ticks(seconds: i32) -> i32 {
    seconds * SIM_HZ
}

fn academy_is_active(build: &AcademyBuildState, state: &AcademyState) -> bool {
    build.completed
        && state.connected_to_grid
        && state.supplied_energy
        && state.supplied_workers
        && state.supplied_gold_maintenance
        && state.research_tier_stone_workshop_met
        && !state.destroyed
        && !state.demolished
}

fn place_academy_system(
    mut commands: Commands,
    mut events: EventReader<PlaceAcademyOfImmortalsEvent>,
    mut claims: ResMut<AcademyPlacementClaims>,
) {
    for ev in events.read() {
        let anchor = BuildingAnchor {
            x: ev.tile_x,
            y: ev.tile_y,
        };

        let entity = commands
            .spawn((
                AcademyOfImmortals,
                anchor,
                AcademyCoreStats::default(),
                AcademyBuildState {
                    build_ticks_remaining: seconds_to_ticks(ACADEMY_BUILD_TIME_SECONDS),
                    completed: false,
                },
                AcademyEconomy::default(),
                AcademyFootprint::default(),
                AcademyState::default(),
                AcademyEffects::default(),
            ))
            .id();

        claims.claims.insert(entity, anchor);
    }
}

fn set_academy_grid_connection_system(
    mut events: EventReader<SetAcademyGridConnectionEvent>,
    mut academies: Query<&mut AcademyState, With<AcademyOfImmortals>>,
) {
    for ev in events.read() {
        let Ok(mut state) = academies.get_mut(ev.entity) else {
            continue;
        };
        state.connected_to_grid = ev.connected;
    }
}

fn set_academy_supply_state_system(
    mut events: EventReader<SetAcademySupplyStateEvent>,
    mut academies: Query<&mut AcademyState, With<AcademyOfImmortals>>,
) {
    for ev in events.read() {
        let Ok(mut state) = academies.get_mut(ev.entity) else {
            continue;
        };
        state.supplied_energy = ev.supplied_energy;
        state.supplied_workers = ev.supplied_workers;
        state.supplied_gold_maintenance = ev.supplied_gold_maintenance;
    }
}

fn set_academy_research_tier_system(
    mut events: EventReader<SetAcademyResearchTierEvent>,
    mut academies: Query<&mut AcademyState, With<AcademyOfImmortals>>,
) {
    for ev in events.read() {
        let Ok(mut state) = academies.get_mut(ev.entity) else {
            continue;
        };
        state.research_tier_stone_workshop_met = ev.stone_workshop_met;
    }
}

fn academy_build_tick_system(mut academies: Query<&mut AcademyBuildState, With<AcademyOfImmortals>>) {
    for mut build in &mut academies {
        if build.completed {
            continue;
        }

        if build.build_ticks_remaining > 0 {
            build.build_ticks_remaining -= 1;
        }

        if build.build_ticks_remaining <= 0 {
            build.build_ticks_remaining = 0;
            build.completed = true;
        }
    }
}

fn attack_destroy_academy_system(
    mut events: EventReader<AttackDestroyAcademyEvent>,
    mut academies: Query<&mut AcademyState, With<AcademyOfImmortals>>,
) {
    for ev in events.read() {
        let Ok(mut state) = academies.get_mut(ev.entity) else {
            continue;
        };
        state.destroyed = true;
    }
}

fn destroy_academy_at_zero_health_system(
    mut academies: Query<(&AcademyCoreStats, &mut AcademyState), With<AcademyOfImmortals>>,
) {
    for (core, mut state) in &mut academies {
        if core.health.current <= I32F32::ZERO {
            state.destroyed = true;
        }
    }
}

fn demolish_academy_system(
    mut events: EventReader<DemolishAcademyEvent>,
    mut academies: Query<&mut AcademyState, With<AcademyOfImmortals>>,
) {
    for ev in events.read() {
        let Ok(mut state) = academies.get_mut(ev.entity) else {
            continue;
        };
        state.demolished = true;
    }
}

fn apply_academy_veteran_effect_system(
    academies: Query<(&AcademyBuildState, &AcademyState, &AcademyEffects), With<AcademyOfImmortals>>,
    mut rangers: Query<(&mut ranger::Veteran, &mut UnitStats), With<ranger::Ranger>>,
    mut soldiers: Query<(&mut soldier::Veteran, &mut UnitStats), With<soldier::Soldier>>,
    mut snipers: Query<(&mut sniper::IsVeteran, &mut UnitStats), With<sniper::Sniper>>,
) {
    let mut applies_to_ranger = false;
    let mut applies_to_soldier = false;
    let mut applies_to_sniper = false;

    for (build, state, effects) in &academies {
        if !academy_is_active(build, state) {
            continue;
        }

        applies_to_ranger |= effects.applies_to_ranger;
        applies_to_soldier |= effects.applies_to_soldier;
        applies_to_sniper |= effects.applies_to_sniper;
    }

    if applies_to_ranger {
        for (mut veteran, mut stats) in &mut rangers {
            if veteran.0 {
                continue;
            }
            veteran.0 = true;
            stats.attack_range = RANGER_VETERAN_ATTACK_RANGE;
            stats.attack_speed = RANGER_VETERAN_ATTACK_SPEED;
            stats.attack_damage = RANGER_VETERAN_ATTACK_DAMAGE;
        }
    }

    if applies_to_soldier {
        for (mut veteran, mut stats) in &mut soldiers {
            if veteran.0 {
                continue;
            }
            veteran.0 = true;
            stats.attack_range = SOLDIER_VETERAN_ATTACK_RANGE;
            stats.attack_speed = SOLDIER_VETERAN_ATTACK_SPEED;
            stats.attack_damage = SOLDIER_VETERAN_ATTACK_DAMAGE;
        }
    }

    if applies_to_sniper {
        for (mut veteran, mut stats) in &mut snipers {
            if veteran.0 {
                continue;
            }
            veteran.0 = true;
            stats.attack_range = SNIPER_VETERAN_ATTACK_RANGE;
            stats.attack_speed = SNIPER_VETERAN_ATTACK_SPEED;
            stats.attack_damage = SNIPER_VETERAN_ATTACK_DAMAGE;
        }
    }
}

fn academy_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    academies: Query<
        (
            Entity,
            &BuildingAnchor,
            &AcademyCoreStats,
            &AcademyBuildState,
            &AcademyEconomy,
            &AcademyFootprint,
            &AcademyState,
            &AcademyEffects,
        ),
        With<AcademyOfImmortals>,
    >,
    claims: Res<AcademyPlacementClaims>,
) {
    for (entity, anchor, core, build, eco, footprint, state, effects) in &academies {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);

        checksum.accumulate(core.defenses_life.to_bits() as u64);
        checksum.accumulate(core.watch_range.to_bits() as u64);
        checksum.accumulate(core.health.current.to_bits() as u64);
        checksum.accumulate(core.health.max.to_bits() as u64);

        checksum.accumulate(build.build_ticks_remaining as u64);
        checksum.accumulate(u64::from(build.completed));

        checksum.accumulate(eco.energy_cost as u64);
        checksum.accumulate(eco.workers_cost as u64);
        checksum.accumulate(eco.gold_maintenance as u64);
        checksum.accumulate(eco.wood_cost as u64);
        checksum.accumulate(eco.stone_cost as u64);
        checksum.accumulate(eco.iron_cost as u64);
        checksum.accumulate(eco.oil_cost as u64);
        checksum.accumulate(eco.gold_cost as u64);

        checksum.accumulate(footprint.size_tiles as u64);
        checksum.accumulate(footprint.watch_range.to_bits() as u64);

        checksum.accumulate(u64::from(state.connected_to_grid));
        checksum.accumulate(u64::from(state.supplied_energy));
        checksum.accumulate(u64::from(state.supplied_workers));
        checksum.accumulate(u64::from(state.supplied_gold_maintenance));
        checksum.accumulate(u64::from(state.destroyed));
        checksum.accumulate(u64::from(state.demolished));
        checksum.accumulate(u64::from(state.research_tier_stone_workshop_met));

        checksum.accumulate(u64::from(effects.applies_to_ranger));
        checksum.accumulate(u64::from(effects.applies_to_soldier));
        checksum.accumulate(u64::from(effects.applies_to_sniper));
        checksum.accumulate(effects.victory_points_reward as u64);
    }

    checksum.accumulate(claims.claims.len() as u64);
    for (entity, anchor) in &claims.claims {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);
    }
}

pub struct AcademyOfImmortalsPlugin;

impl Plugin for AcademyOfImmortalsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AcademyPlacementClaims>()
            .add_event::<PlaceAcademyOfImmortalsEvent>()
            .add_event::<SetAcademyGridConnectionEvent>()
            .add_event::<SetAcademySupplyStateEvent>()
            .add_event::<SetAcademyResearchTierEvent>()
            .add_event::<DemolishAcademyEvent>()
            .add_event::<AttackDestroyAcademyEvent>()
            .add_systems(
                FixedUpdate,
                (
                    place_academy_system,
                    set_academy_grid_connection_system,
                    set_academy_supply_state_system,
                    set_academy_research_tier_system,
                    academy_build_tick_system,
                    attack_destroy_academy_system,
                    destroy_academy_at_zero_health_system,
                    demolish_academy_system,
                    apply_academy_veteran_effect_system,
                    academy_checksum_system,
                )
                    .chain(),
            );
    }
}