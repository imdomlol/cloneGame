// Sources: vault/buildings/executor.md

use std::collections::{BTreeMap, BTreeSet};

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::buildings::great_ballista::{BuildingAnchor as GreatBallistaAnchor, GreatBallista};
use crate::sim::{Health, SimChecksumState, UnitStats};

const EXECUTOR_HP: I32F32 = I32F32::lit("2000");
const EXECUTOR_DEFENSES_LIFE: I32F32 = I32F32::lit("1000");
const EXECUTOR_WATCH_RANGE: I32F32 = I32F32::lit("12");
const EXECUTOR_ENERGY_COST: i32 = 10;
const EXECUTOR_WOOD_COST: i32 = 0;
const EXECUTOR_STONE_COST: i32 = 0;
const EXECUTOR_IRON_COST: i32 = 20;
const EXECUTOR_OIL_COST: i32 = 10;
const EXECUTOR_GOLD_COST: i32 = 1200;
const EXECUTOR_BUILD_TIME_SECONDS: i32 = 45;
const EXECUTOR_WORKERS: i32 = 5;
const EXECUTOR_SIZE_TILES: i32 = 2;
const EXECUTOR_MAINTENANCE_GOLD: i32 = 50;
const EXECUTOR_ATTACK_DAMAGE: I32F32 = I32F32::lit("100");
const EXECUTOR_ATTACK_RANGE: I32F32 = I32F32::lit("12");
const EXECUTOR_ATTACK_SPEED: I32F32 = I32F32::lit("2");
const EXECUTOR_AOE_RADIUS: I32F32 = I32F32::lit("0.6");
const EXECUTOR_NOISE: i32 = 20;
const EXECUTOR_UPGRADE_GOLD_COST: i32 = 700;
const EXECUTOR_UPGRADE_BUILD_TIME_SECONDS: i32 = 23;
const EXECUTOR_RESEARCH_COST_GOLD: i32 = 3000;
const SIM_HZ: i32 = 25;

#[derive(Component, Default)]
pub struct Executor;

#[derive(Component, Clone, Copy, Default)]
pub struct BuildingAnchor {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExecutorAttackType {
    Shot,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExecutorTargetingMode {
    Nearest,
    HighestThreat,
}

#[derive(Component, Clone, Copy)]
pub struct ExecutorCore {
    pub defenses_life: I32F32,
    pub health: Health,
}

impl Default for ExecutorCore {
    fn default() -> Self {
        Self {
            defenses_life: EXECUTOR_DEFENSES_LIFE,
            health: Health::full(EXECUTOR_HP),
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct ExecutorEconomy {
    pub energy_cost: i32,
    pub wood_cost: i32,
    pub stone_cost: i32,
    pub iron_cost: i32,
    pub oil_cost: i32,
    pub gold_cost: i32,
    pub workers: i32,
    pub gold_maintenance: i32,
    pub upgrade_gold_cost: i32,
    pub research_cost_gold: i32,
}

impl Default for ExecutorEconomy {
    fn default() -> Self {
        Self {
            energy_cost: EXECUTOR_ENERGY_COST,
            wood_cost: EXECUTOR_WOOD_COST,
            stone_cost: EXECUTOR_STONE_COST,
            iron_cost: EXECUTOR_IRON_COST,
            oil_cost: EXECUTOR_OIL_COST,
            gold_cost: EXECUTOR_GOLD_COST,
            workers: EXECUTOR_WORKERS,
            gold_maintenance: EXECUTOR_MAINTENANCE_GOLD,
            upgrade_gold_cost: EXECUTOR_UPGRADE_GOLD_COST,
            research_cost_gold: EXECUTOR_RESEARCH_COST_GOLD,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct ExecutorFootprint {
    pub size_tiles: i32,
    pub watch_range: I32F32,
}

impl Default for ExecutorFootprint {
    fn default() -> Self {
        Self {
            size_tiles: EXECUTOR_SIZE_TILES,
            watch_range: EXECUTOR_WATCH_RANGE,
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct ExecutorBuildState {
    pub build_ticks_remaining: i32,
    pub upgrading_from_great_ballista: bool,
    pub completed: bool,
}

#[derive(Component, Clone, Copy)]
pub struct ExecutorAttackProfile {
    pub stats: UnitStats,
    pub aoe_radius: I32F32,
    pub noise: i32,
    pub attack_type: ExecutorAttackType,
    pub targeting_mode: ExecutorTargetingMode,
}

impl Default for ExecutorAttackProfile {
    fn default() -> Self {
        Self {
            stats: UnitStats {
                move_speed: I32F32::ZERO,
                attack_range: EXECUTOR_ATTACK_RANGE,
                attack_damage: EXECUTOR_ATTACK_DAMAGE,
                attack_speed: EXECUTOR_ATTACK_SPEED,
                watch_range: EXECUTOR_WATCH_RANGE,
            },
            aoe_radius: EXECUTOR_AOE_RADIUS,
            noise: EXECUTOR_NOISE,
            attack_type: ExecutorAttackType::Shot,
            targeting_mode: ExecutorTargetingMode::Nearest,
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct ExecutorAttackState {
    pub cooldown_ticks_remaining: i32,
    pub total_shots_fired: i64,
    pub total_shots_hit: i64,
    pub total_noise_generated: i64,
    pub current_target: Option<Entity>,
}

#[derive(Component, Clone, Copy, Default)]
pub struct ExecutorCombatLedger {
    pub total_direct_damage_applied: I32F32,
    pub total_splash_damage_applied: I32F32,
}

#[derive(Component, Clone, Copy, Default)]
pub struct ExecutorTargetProfile {
    pub threat_level: i32,
    pub is_large_or_stationary: bool,
    pub is_fast_or_airborne: bool,
}

#[derive(Component, Clone, Copy, Default)]
pub struct ExecutorTargetPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Resource, Clone, Copy, Default)]
pub struct ExecutorResearchState {
    pub unlocked: bool,
}

#[derive(Resource, Default, Clone)]
pub struct TileOccupancy {
    pub blocked_tiles: BTreeSet<(i32, i32)>,
}

#[derive(Resource, Default, Clone)]
pub struct ExecutorPlacementClaims {
    pub claims: BTreeMap<Entity, BuildingAnchor>,
}

#[derive(Event, Clone, Copy)]
pub struct SetTileBlockedEvent {
    pub tile_x: i32,
    pub tile_y: i32,
    pub blocked: bool,
}

#[derive(Event, Clone, Copy)]
pub struct SetExecutorResearchUnlockedEvent {
    pub unlocked: bool,
}

#[derive(Event, Clone, Copy)]
pub struct PlaceExecutorEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct UpgradeGreatBallistaToExecutorEvent {
    pub great_ballista_entity: Entity,
}

#[derive(Event, Clone, Copy)]
pub struct ExecutorPlacementRejectedEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct SetExecutorTargetingModeEvent {
    pub entity: Entity,
    pub mode: ExecutorTargetingMode,
}

#[derive(Event, Clone, Copy)]
pub struct SetExecutorTargetProfileEvent {
    pub entity: Entity,
    pub profile: ExecutorTargetProfile,
    pub x: i32,
    pub y: i32,
}

fn seconds_to_ticks(seconds: i32) -> i32 {
    seconds * SIM_HZ
}

fn attack_cooldown_ticks(profile: &ExecutorAttackProfile) -> i32 {
    let ticks = I32F32::from_num(SIM_HZ) / profile.stats.attack_speed;
    ticks.ceil().to_num::<i32>().max(1)
}

fn footprint_tiles(anchor: BuildingAnchor) -> impl Iterator<Item = (i32, i32)> {
    let min_x = anchor.x;
    let min_y = anchor.y;
    let max_x = anchor.x + EXECUTOR_SIZE_TILES - 1;
    let max_y = anchor.y + EXECUTOR_SIZE_TILES - 1;
    (min_x..=max_x).flat_map(move |x| (min_y..=max_y).map(move |y| (x, y)))
}

fn footprint_is_unblocked(anchor: BuildingAnchor, occupancy: &TileOccupancy) -> bool {
    for tile in footprint_tiles(anchor) {
        if occupancy.blocked_tiles.contains(&tile) {
            return false;
        }
    }
    true
}

fn distance_squared(anchor: BuildingAnchor, target: ExecutorTargetPosition) -> i64 {
    let dx = (anchor.x - target.x) as i64;
    let dy = (anchor.y - target.y) as i64;
    dx * dx + dy * dy
}

fn apply_tile_block_events_system(
    mut events: EventReader<SetTileBlockedEvent>,
    mut occupancy: ResMut<TileOccupancy>,
) {
    for ev in events.read() {
        let tile = (ev.tile_x, ev.tile_y);
        if ev.blocked {
            occupancy.blocked_tiles.insert(tile);
        } else {
            occupancy.blocked_tiles.remove(&tile);
        }
    }
}

fn apply_research_unlock_event_system(
    mut events: EventReader<SetExecutorResearchUnlockedEvent>,
    mut research: ResMut<ExecutorResearchState>,
) {
    for ev in events.read() {
        research.unlocked = ev.unlocked;
    }
}

fn place_executor_system(
    mut commands: Commands,
    mut events: EventReader<PlaceExecutorEvent>,
    mut rejected: EventWriter<ExecutorPlacementRejectedEvent>,
    occupancy: Res<TileOccupancy>,
    research: Res<ExecutorResearchState>,
    mut claims: ResMut<ExecutorPlacementClaims>,
) {
    for ev in events.read() {
        if !research.unlocked {
            rejected.send(ExecutorPlacementRejectedEvent {
                tile_x: ev.tile_x,
                tile_y: ev.tile_y,
            });
            continue;
        }

        let anchor = BuildingAnchor {
            x: ev.tile_x,
            y: ev.tile_y,
        };

        if !footprint_is_unblocked(anchor, &occupancy) {
            rejected.send(ExecutorPlacementRejectedEvent {
                tile_x: ev.tile_x,
                tile_y: ev.tile_y,
            });
            continue;
        }

        let entity = commands
            .spawn((
                Executor,
                anchor,
                ExecutorCore::default(),
                ExecutorEconomy::default(),
                ExecutorFootprint::default(),
                ExecutorBuildState {
                    build_ticks_remaining: seconds_to_ticks(EXECUTOR_BUILD_TIME_SECONDS),
                    upgrading_from_great_ballista: false,
                    completed: false,
                },
                ExecutorAttackProfile::default(),
                ExecutorAttackState::default(),
                ExecutorCombatLedger::default(),
            ))
            .id();

        claims.claims.insert(entity, anchor);
    }
}

fn upgrade_great_ballista_to_executor_system(
    mut commands: Commands,
    mut events: EventReader<UpgradeGreatBallistaToExecutorEvent>,
    great_ballistas: Query<&GreatBallistaAnchor, With<GreatBallista>>,
    mut claims: ResMut<ExecutorPlacementClaims>,
) {
    for ev in events.read() {
        let Ok(ballista_anchor) = great_ballistas.get(ev.great_ballista_entity) else {
            continue;
        };

        let anchor = BuildingAnchor {
            x: ballista_anchor.x,
            y: ballista_anchor.y,
        };

        commands
            .entity(ev.great_ballista_entity)
            .remove::<GreatBallista>()
            .remove::<GreatBallistaAnchor>()
            .insert((
                Executor,
                anchor,
                ExecutorCore::default(),
                ExecutorEconomy::default(),
                ExecutorFootprint::default(),
                ExecutorBuildState {
                    build_ticks_remaining: seconds_to_ticks(EXECUTOR_UPGRADE_BUILD_TIME_SECONDS),
                    upgrading_from_great_ballista: true,
                    completed: false,
                },
                ExecutorAttackProfile::default(),
                ExecutorAttackState::default(),
                ExecutorCombatLedger::default(),
            ));

        claims.claims.insert(ev.great_ballista_entity, anchor);
    }
}

fn apply_target_profile_events_system(
    mut commands: Commands,
    mut events: EventReader<SetExecutorTargetProfileEvent>,
) {
    for ev in events.read() {
        commands.entity(ev.entity).insert((ev.profile, ExecutorTargetPosition { x: ev.x, y: ev.y }));
    }
}

fn set_targeting_mode_system(
    mut events: EventReader<SetExecutorTargetingModeEvent>,
    mut executors: Query<&mut ExecutorAttackProfile, With<Executor>>,
) {
    for ev in events.read() {
        let Ok(mut profile) = executors.get_mut(ev.entity) else {
            continue;
        };
        profile.targeting_mode = ev.mode;
    }
}

fn executor_build_tick_system(
    mut executors: Query<&mut ExecutorBuildState, With<Executor>>,
) {
    for mut build in &mut executors {
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

fn executor_target_and_attack_system(
    mut executors: Query<
        (
            Entity,
            &BuildingAnchor,
            &ExecutorBuildState,
            &ExecutorAttackProfile,
            &mut ExecutorAttackState,
            &mut ExecutorCombatLedger,
        ),
        With<Executor>,
    >,
    targets: Query<(Entity, &ExecutorTargetProfile, &ExecutorTargetPosition)>,
) {
    for (executor_entity, anchor, build, attack_profile, mut attack_state, mut ledger) in &mut executors {
        if !build.completed {
            continue;
        }

        if attack_state.cooldown_ticks_remaining > 0 {
            attack_state.cooldown_ticks_remaining -= 1;
            continue;
        }

        let max_range_sq = attack_profile.stats.attack_range * attack_profile.stats.attack_range;
        let mut selected: Option<(Entity, ExecutorTargetProfile, ExecutorTargetPosition, i64)> = None;

        for (target_entity, profile, position) in &targets {
            if target_entity == executor_entity {
                continue;
            }

            let dist_sq = distance_squared(*anchor, *position);
            if I32F32::from_num(dist_sq) > max_range_sq {
                continue;
            }

            match selected {
                None => selected = Some((target_entity, *profile, *position, dist_sq)),
                Some((_, selected_profile, _, selected_dist_sq)) => match attack_profile.targeting_mode {
                    ExecutorTargetingMode::Nearest => {
                        if dist_sq < selected_dist_sq {
                            selected = Some((target_entity, *profile, *position, dist_sq));
                        }
                    }
                    ExecutorTargetingMode::HighestThreat => {
                        if profile.threat_level > selected_profile.threat_level
                            || (profile.threat_level == selected_profile.threat_level && dist_sq < selected_dist_sq)
                        {
                            selected = Some((target_entity, *profile, *position, dist_sq));
                        }
                    }
                },
            }
        }

        let Some((target_entity, target_profile, _, _)) = selected else {
            attack_state.current_target = None;
            continue;
        };

        attack_state.current_target = Some(target_entity);
        attack_state.total_shots_fired += 1;
        attack_state.total_noise_generated += attack_profile.noise as i64;
        attack_state.cooldown_ticks_remaining = attack_cooldown_ticks(attack_profile);

        if target_profile.is_fast_or_airborne && (attack_state.total_shots_fired % 2 == 0) {
            continue;
        }

        attack_state.total_shots_hit += 1;
        ledger.total_direct_damage_applied += attack_profile.stats.attack_damage;
        ledger.total_splash_damage_applied += attack_profile.stats.attack_damage;
    }
}

fn executor_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    executors: Query<
        (
            Entity,
            &BuildingAnchor,
            &ExecutorCore,
            &ExecutorEconomy,
            &ExecutorFootprint,
            &ExecutorBuildState,
            &ExecutorAttackProfile,
            &ExecutorAttackState,
            &ExecutorCombatLedger,
        ),
        With<Executor>,
    >,
    targets: Query<(Entity, &ExecutorTargetProfile, &ExecutorTargetPosition)>,
    research: Res<ExecutorResearchState>,
    occupancy: Res<TileOccupancy>,
    claims: Res<ExecutorPlacementClaims>,
) {
    for (entity, anchor, core, economy, footprint, build, attack_profile, attack_state, ledger) in &executors {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);

        checksum.accumulate(core.defenses_life.to_bits() as u64);
        checksum.accumulate(core.health.current.to_bits() as u64);
        checksum.accumulate(core.health.max.to_bits() as u64);

        checksum.accumulate(economy.energy_cost as u64);
        checksum.accumulate(economy.wood_cost as u64);
        checksum.accumulate(economy.stone_cost as u64);
        checksum.accumulate(economy.iron_cost as u64);
        checksum.accumulate(economy.oil_cost as u64);
        checksum.accumulate(economy.gold_cost as u64);
        checksum.accumulate(economy.workers as u64);
        checksum.accumulate(economy.gold_maintenance as u64);
        checksum.accumulate(economy.upgrade_gold_cost as u64);
        checksum.accumulate(economy.research_cost_gold as u64);

        checksum.accumulate(footprint.size_tiles as u64);
        checksum.accumulate(footprint.watch_range.to_bits() as u64);

        checksum.accumulate(build.build_ticks_remaining as u64);
        checksum.accumulate(u64::from(build.upgrading_from_great_ballista));
        checksum.accumulate(u64::from(build.completed));

        checksum.accumulate(attack_profile.stats.move_speed.to_bits() as u64);
        checksum.accumulate(attack_profile.stats.attack_range.to_bits() as u64);
        checksum.accumulate(attack_profile.stats.attack_damage.to_bits() as u64);
        checksum.accumulate(attack_profile.stats.attack_speed.to_bits() as u64);
        checksum.accumulate(attack_profile.stats.watch_range.to_bits() as u64);
        checksum.accumulate(attack_profile.aoe_radius.to_bits() as u64);
        checksum.accumulate(attack_profile.noise as u64);
        checksum.accumulate(match attack_profile.attack_type {
            ExecutorAttackType::Shot => 1,
        });
        checksum.accumulate(match attack_profile.targeting_mode {
            ExecutorTargetingMode::Nearest => 1,
            ExecutorTargetingMode::HighestThreat => 2,
        });

        checksum.accumulate(attack_state.cooldown_ticks_remaining as u64);
        checksum.accumulate(attack_state.total_shots_fired as u64);
        checksum.accumulate(attack_state.total_shots_hit as u64);
        checksum.accumulate(attack_state.total_noise_generated as u64);
        checksum.accumulate(attack_state.current_target.map(Entity::to_bits).unwrap_or(0));

        checksum.accumulate(ledger.total_direct_damage_applied.to_bits() as u64);
        checksum.accumulate(ledger.total_splash_damage_applied.to_bits() as u64);
    }

    for (entity, profile, pos) in &targets {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(profile.threat_level as u64);
        checksum.accumulate(u64::from(profile.is_large_or_stationary));
        checksum.accumulate(u64::from(profile.is_fast_or_airborne));
        checksum.accumulate(pos.x as u64);
        checksum.accumulate(pos.y as u64);
    }

    checksum.accumulate(u64::from(research.unlocked));

    checksum.accumulate(occupancy.blocked_tiles.len() as u64);
    for (x, y) in &occupancy.blocked_tiles {
        checksum.accumulate(*x as u64);
        checksum.accumulate(*y as u64);
    }

    checksum.accumulate(claims.claims.len() as u64);
    for (entity, anchor) in &claims.claims {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);
    }
}

pub struct ExecutorPlugin;

impl Plugin for ExecutorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ExecutorResearchState>()
            .init_resource::<TileOccupancy>()
            .init_resource::<ExecutorPlacementClaims>()
            .add_event::<SetTileBlockedEvent>()
            .add_event::<SetExecutorResearchUnlockedEvent>()
            .add_event::<PlaceExecutorEvent>()
            .add_event::<UpgradeGreatBallistaToExecutorEvent>()
            .add_event::<ExecutorPlacementRejectedEvent>()
            .add_event::<SetExecutorTargetingModeEvent>()
            .add_event::<SetExecutorTargetProfileEvent>()
            .add_systems(
                FixedUpdate,
                (
                    apply_tile_block_events_system,
                    apply_research_unlock_event_system,
                    place_executor_system,
                    upgrade_great_ballista_to_executor_system,
                    apply_target_profile_events_system,
                    set_targeting_mode_system,
                    executor_build_tick_system,
                    executor_target_and_attack_system,
                    executor_checksum_system,
                )
                    .chain(),
            );
    }
}