// Sources: vault/buildings/great_ballista.md, vault/buildings/executor.md, vault/buildings/wood_workshop.md

use std::collections::{BTreeMap, BTreeSet};

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{Health, SimChecksumState};

const GREAT_BALLISTA_HP: I32F32 = I32F32::lit("1000");
const GREAT_BALLISTA_DEFENSES_LIFE: I32F32 = I32F32::lit("500");
const GREAT_BALLISTA_WATCH_RANGE: I32F32 = I32F32::lit("12");
const GREAT_BALLISTA_ENERGY_COST: i32 = 3;
const GREAT_BALLISTA_WOOD_COST: i32 = 20;
const GREAT_BALLISTA_STONE_COST: i32 = 0;
const GREAT_BALLISTA_IRON_COST: i32 = 0;
const GREAT_BALLISTA_OIL_COST: i32 = 0;
const GREAT_BALLISTA_GOLD_COST: i32 = 500;
const GREAT_BALLISTA_BUILD_TIME_SECONDS: i32 = 45;
const GREAT_BALLISTA_WORKERS: i32 = 3;
const GREAT_BALLISTA_SIZE_TILES: i32 = 2;
const GREAT_BALLISTA_ATTACK_DAMAGE: I32F32 = I32F32::lit("150");
const GREAT_BALLISTA_ATTACK_RANGE: I32F32 = I32F32::lit("9");
const GREAT_BALLISTA_ATTACK_SPEED: I32F32 = I32F32::lit("1");
const GREAT_BALLISTA_AOE_RADIUS: I32F32 = I32F32::lit("0.3");
const GREAT_BALLISTA_NOISE: i32 = 5;
const GREAT_BALLISTA_DIRECT_HIT_DAMAGE_MIN: I32F32 = I32F32::lit("100");
const GREAT_BALLISTA_DIRECT_HIT_DAMAGE_MAX: I32F32 = I32F32::lit("200");
const GREAT_BALLISTA_RESEARCH_COST_GOLD: i32 = 700;
const GREAT_BALLISTA_KNOCKBACK: I32F32 = I32F32::lit("1");
const GREAT_BALLISTA_EFFECTIVE_HIT_RANGE: I32F32 = I32F32::lit("8.75");
const SIM_HZ: i32 = 25;

#[derive(Component, Default)]
pub struct GreatBallista;

#[derive(Component, Clone, Copy, Default)]
pub struct BuildingAnchor {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Clone, Copy)]
pub struct GreatBallistaCore {
    pub defenses_life: I32F32,
    pub watch_range: I32F32,
    pub health: Health,
}

impl Default for GreatBallistaCore {
    fn default() -> Self {
        Self {
            defenses_life: GREAT_BALLISTA_DEFENSES_LIFE,
            watch_range: GREAT_BALLISTA_WATCH_RANGE,
            health: Health::full(GREAT_BALLISTA_HP),
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct GreatBallistaEconomy {
    pub energy_cost: i32,
    pub wood_cost: i32,
    pub stone_cost: i32,
    pub iron_cost: i32,
    pub oil_cost: i32,
    pub gold_cost: i32,
    pub workers: i32,
    pub research_cost_gold: i32,
    pub upgrade_target_executor: bool,
}

impl Default for GreatBallistaEconomy {
    fn default() -> Self {
        Self {
            energy_cost: GREAT_BALLISTA_ENERGY_COST,
            wood_cost: GREAT_BALLISTA_WOOD_COST,
            stone_cost: GREAT_BALLISTA_STONE_COST,
            iron_cost: GREAT_BALLISTA_IRON_COST,
            oil_cost: GREAT_BALLISTA_OIL_COST,
            gold_cost: GREAT_BALLISTA_GOLD_COST,
            workers: GREAT_BALLISTA_WORKERS,
            research_cost_gold: GREAT_BALLISTA_RESEARCH_COST_GOLD,
            upgrade_target_executor: true,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct GreatBallistaFootprint {
    pub size_tiles: i32,
    pub watch_range: I32F32,
}

impl Default for GreatBallistaFootprint {
    fn default() -> Self {
        Self {
            size_tiles: GREAT_BALLISTA_SIZE_TILES,
            watch_range: GREAT_BALLISTA_WATCH_RANGE,
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct GreatBallistaBuildState {
    pub build_ticks_remaining: i32,
    pub completed: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GreatBallistaTargetingMode {
    Nearest,
    HighestThreat,
}

#[derive(Component, Clone, Copy)]
pub struct GreatBallistaAttackProfile {
    pub attack_damage: I32F32,
    pub attack_range: I32F32,
    pub attack_speed: I32F32,
    pub aoe_radius: I32F32,
    pub direct_hit_damage_min: I32F32,
    pub direct_hit_damage_max: I32F32,
    pub effective_hit_range: I32F32,
    pub knockback_impulse: I32F32,
    pub noise: i32,
    pub targeting_mode: GreatBallistaTargetingMode,
}

impl Default for GreatBallistaAttackProfile {
    fn default() -> Self {
        Self {
            attack_damage: GREAT_BALLISTA_ATTACK_DAMAGE,
            attack_range: GREAT_BALLISTA_ATTACK_RANGE,
            attack_speed: GREAT_BALLISTA_ATTACK_SPEED,
            aoe_radius: GREAT_BALLISTA_AOE_RADIUS,
            direct_hit_damage_min: GREAT_BALLISTA_DIRECT_HIT_DAMAGE_MIN,
            direct_hit_damage_max: GREAT_BALLISTA_DIRECT_HIT_DAMAGE_MAX,
            effective_hit_range: GREAT_BALLISTA_EFFECTIVE_HIT_RANGE,
            knockback_impulse: GREAT_BALLISTA_KNOCKBACK,
            noise: GREAT_BALLISTA_NOISE,
            targeting_mode: GreatBallistaTargetingMode::Nearest,
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct GreatBallistaAttackState {
    pub cooldown_ticks_remaining: i32,
    pub total_shots_fired: i64,
    pub total_shots_hit: i64,
    pub total_noise_generated: i64,
    pub current_target: Option<Entity>,
}

#[derive(Component, Clone, Copy, Default)]
pub struct GreatBallistaUpgradeState {
    pub executor_research_completed: bool,
    pub can_upgrade_to_executor: bool,
}

#[derive(Component, Clone, Copy, Default)]
pub struct GreatBallistaCombatLedger {
    pub total_direct_hit_damage_applied: I32F32,
    pub total_splash_damage_applied: I32F32,
    pub total_knockback_applied: I32F32,
}

#[derive(Component, Clone, Copy, Default)]
pub struct GreatBallistaTargetProfile {
    pub threat_level: i32,
    pub is_village_of_doom: bool,
    pub is_large_or_stationary: bool,
    pub is_fast_or_airborne: bool,
}

#[derive(Component, Clone, Copy, Default)]
pub struct GreatBallistaTargetPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Resource, Clone, Copy, Default)]
pub struct GreatBallistaResearchState {
    pub unlocked: bool,
}

#[derive(Resource, Default, Clone)]
pub struct TileOccupancy {
    pub blocked_tiles: BTreeSet<(i32, i32)>,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LineOfFireBlockerKind {
    Mountain,
    Forest,
    LargeRock,
}

#[derive(Resource, Default, Clone)]
pub struct LineOfFireBlockers {
    pub blockers: BTreeMap<(i32, i32), LineOfFireBlockerKind>,
}

#[derive(Resource, Default, Clone)]
pub struct GreatBallistaPlacementClaims {
    pub claims: BTreeMap<Entity, BuildingAnchor>,
}

#[derive(Event, Clone, Copy)]
pub struct SetTileBlockedEvent {
    pub tile_x: i32,
    pub tile_y: i32,
    pub blocked: bool,
}

#[derive(Event, Clone, Copy)]
pub struct SetLineOfFireBlockerEvent {
    pub tile_x: i32,
    pub tile_y: i32,
    pub blocker: Option<LineOfFireBlockerKind>,
}

#[derive(Event, Clone, Copy)]
pub struct SetGreatBallistaResearchUnlockedEvent {
    pub unlocked: bool,
}

#[derive(Event, Clone, Copy)]
pub struct PlaceGreatBallistaEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct GreatBallistaPlacementRejectedEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct SetGreatBallistaTargetingModeEvent {
    pub entity: Entity,
    pub mode: GreatBallistaTargetingMode,
}

#[derive(Event, Clone, Copy)]
pub struct SetGreatBallistaExecutorResearchCompletedEvent {
    pub entity: Entity,
    pub researched: bool,
}

#[derive(Event, Clone, Copy)]
pub struct SetGreatBallistaTargetProfileEvent {
    pub entity: Entity,
    pub profile: GreatBallistaTargetProfile,
    pub x: i32,
    pub y: i32,
}

fn seconds_to_ticks(seconds: i32) -> i32 {
    seconds * SIM_HZ
}

fn attack_cooldown_ticks(profile: &GreatBallistaAttackProfile) -> i32 {
    let ticks = I32F32::from_num(SIM_HZ) / profile.attack_speed;
    let rounded = ticks.ceil().to_num::<i32>();
    rounded.max(1)
}

fn footprint_tiles(anchor: BuildingAnchor) -> impl Iterator<Item = (i32, i32)> {
    let min_x = anchor.x;
    let min_y = anchor.y;
    let max_x = anchor.x + GREAT_BALLISTA_SIZE_TILES - 1;
    let max_y = anchor.y + GREAT_BALLISTA_SIZE_TILES - 1;
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

fn line_of_fire_blocked(
    anchor: BuildingAnchor,
    target: GreatBallistaTargetPosition,
    blockers: &LineOfFireBlockers,
) -> bool {
    let center_x = anchor.x;
    let center_y = anchor.y;
    let min_x = center_x.min(target.x);
    let max_x = center_x.max(target.x);
    let min_y = center_y.min(target.y);
    let max_y = center_y.max(target.y);

    for x in min_x..=max_x {
        for y in min_y..=max_y {
            let Some(kind) = blockers.blockers.get(&(x, y)) else {
                continue;
            };
            if *kind == LineOfFireBlockerKind::Mountain || *kind == LineOfFireBlockerKind::Forest {
                return true;
            }
        }
    }

    false
}

fn distance_squared(a: BuildingAnchor, b: GreatBallistaTargetPosition) -> i64 {
    let dx = (a.x - b.x) as i64;
    let dy = (a.y - b.y) as i64;
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

fn apply_line_of_fire_blockers_system(
    mut events: EventReader<SetLineOfFireBlockerEvent>,
    mut blockers: ResMut<LineOfFireBlockers>,
) {
    for ev in events.read() {
        let tile = (ev.tile_x, ev.tile_y);
        if let Some(blocker) = ev.blocker {
            blockers.blockers.insert(tile, blocker);
        } else {
            blockers.blockers.remove(&tile);
        }
    }
}

fn apply_research_unlock_event_system(
    mut events: EventReader<SetGreatBallistaResearchUnlockedEvent>,
    mut research: ResMut<GreatBallistaResearchState>,
) {
    for ev in events.read() {
        research.unlocked = ev.unlocked;
    }
}

fn place_great_ballista_system(
    mut commands: Commands,
    mut events: EventReader<PlaceGreatBallistaEvent>,
    mut rejected: EventWriter<GreatBallistaPlacementRejectedEvent>,
    occupancy: Res<TileOccupancy>,
    research: Res<GreatBallistaResearchState>,
    mut claims: ResMut<GreatBallistaPlacementClaims>,
) {
    for ev in events.read() {
        if !research.unlocked {
            rejected.send(GreatBallistaPlacementRejectedEvent {
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
            rejected.send(GreatBallistaPlacementRejectedEvent {
                tile_x: ev.tile_x,
                tile_y: ev.tile_y,
            });
            continue;
        }

        let entity = commands
            .spawn((
                GreatBallista,
                anchor,
                GreatBallistaCore::default(),
                GreatBallistaEconomy::default(),
                GreatBallistaFootprint::default(),
                GreatBallistaBuildState {
                    build_ticks_remaining: seconds_to_ticks(GREAT_BALLISTA_BUILD_TIME_SECONDS),
                    completed: false,
                },
                GreatBallistaAttackProfile::default(),
                GreatBallistaAttackState::default(),
                GreatBallistaUpgradeState::default(),
                GreatBallistaCombatLedger::default(),
            ))
            .id();

        claims.claims.insert(entity, anchor);
    }
}

fn apply_target_profile_events_system(
    mut commands: Commands,
    mut events: EventReader<SetGreatBallistaTargetProfileEvent>,
) {
    for ev in events.read() {
        commands.entity(ev.entity).insert((ev.profile, GreatBallistaTargetPosition { x: ev.x, y: ev.y }));
    }
}

fn set_targeting_mode_system(
    mut events: EventReader<SetGreatBallistaTargetingModeEvent>,
    mut ballistas: Query<&mut GreatBallistaAttackProfile, With<GreatBallista>>,
) {
    for ev in events.read() {
        let Ok(mut attack) = ballistas.get_mut(ev.entity) else {
            continue;
        };
        attack.targeting_mode = ev.mode;
    }
}

fn set_upgrade_research_system(
    mut events: EventReader<SetGreatBallistaExecutorResearchCompletedEvent>,
    mut ballistas: Query<&mut GreatBallistaUpgradeState, With<GreatBallista>>,
) {
    for ev in events.read() {
        let Ok(mut upgrade) = ballistas.get_mut(ev.entity) else {
            continue;
        };
        upgrade.executor_research_completed = ev.researched;
    }
}

fn great_ballista_build_tick_system(
    mut ballistas: Query<&mut GreatBallistaBuildState, With<GreatBallista>>,
) {
    for mut build in &mut ballistas {
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

fn great_ballista_upgrade_gate_system(
    mut ballistas: Query<(&GreatBallistaBuildState, &mut GreatBallistaUpgradeState), With<GreatBallista>>,
) {
    for (build, mut upgrade) in &mut ballistas {
        upgrade.can_upgrade_to_executor = build.completed && upgrade.executor_research_completed;
    }
}

fn great_ballista_target_and_attack_system(
    mut ballistas: Query<
        (
            &BuildingAnchor,
            &GreatBallistaBuildState,
            &GreatBallistaAttackProfile,
            &mut GreatBallistaAttackState,
            &mut GreatBallistaCombatLedger,
        ),
        With<GreatBallista>,
    >,
    targets: Query<(Entity, &GreatBallistaTargetProfile, &GreatBallistaTargetPosition)>,
    blockers: Res<LineOfFireBlockers>,
) {
    for (anchor, build, attack_profile, mut attack_state, mut ledger) in &mut ballistas {
        if !build.completed {
            continue;
        }

        if attack_state.cooldown_ticks_remaining > 0 {
            attack_state.cooldown_ticks_remaining -= 1;
            continue;
        }

        let max_range_squared = attack_profile.effective_hit_range * attack_profile.effective_hit_range;
        let mut selected: Option<(Entity, GreatBallistaTargetProfile, GreatBallistaTargetPosition, i64)> = None;

        for (target_entity, profile, position) in &targets {
            let distance_sq = distance_squared(*anchor, *position);
            if I32F32::from_num(distance_sq) > max_range_squared {
                continue;
            }

            if line_of_fire_blocked(*anchor, *position, &blockers) {
                continue;
            }

            match selected {
                None => {
                    selected = Some((target_entity, *profile, *position, distance_sq));
                }
                Some((_, selected_profile, _, selected_distance_sq)) => match attack_profile.targeting_mode {
                    GreatBallistaTargetingMode::Nearest => {
                        if distance_sq < selected_distance_sq {
                            selected = Some((target_entity, *profile, *position, distance_sq));
                        }
                    }
                    GreatBallistaTargetingMode::HighestThreat => {
                        if profile.threat_level > selected_profile.threat_level
                            || (profile.threat_level == selected_profile.threat_level && distance_sq < selected_distance_sq)
                        {
                            selected = Some((target_entity, *profile, *position, distance_sq));
                        }
                    }
                },
            }
        }

        let Some((target_entity, profile, _, _)) = selected else {
            attack_state.current_target = None;
            continue;
        };

        attack_state.current_target = Some(target_entity);
        attack_state.total_shots_fired += 1;
        attack_state.total_noise_generated += attack_profile.noise as i64;
        attack_state.cooldown_ticks_remaining = attack_cooldown_ticks(attack_profile);

        let misses_this_shot = profile.is_fast_or_airborne && (attack_state.total_shots_fired % 2 == 0);
        if misses_this_shot {
            continue;
        }

        attack_state.total_shots_hit += 1;

        let direct_hit_damage = if profile.is_village_of_doom || profile.is_large_or_stationary {
            attack_profile.direct_hit_damage_max
        } else {
            attack_profile.direct_hit_damage_min
        };

        ledger.total_direct_hit_damage_applied += direct_hit_damage;
        ledger.total_splash_damage_applied += attack_profile.attack_damage;
        ledger.total_knockback_applied += attack_profile.knockback_impulse;
    }
}

fn great_ballista_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    ballistas: Query<
        (
            Entity,
            &BuildingAnchor,
            &GreatBallistaCore,
            &GreatBallistaEconomy,
            &GreatBallistaFootprint,
            &GreatBallistaBuildState,
            &GreatBallistaAttackProfile,
            &GreatBallistaAttackState,
            &GreatBallistaUpgradeState,
            &GreatBallistaCombatLedger,
        ),
        With<GreatBallista>,
    >,
    target_entities: Query<(Entity, &GreatBallistaTargetProfile, &GreatBallistaTargetPosition)>,
    research: Res<GreatBallistaResearchState>,
    occupancy: Res<TileOccupancy>,
    blockers: Res<LineOfFireBlockers>,
    claims: Res<GreatBallistaPlacementClaims>,
) {
    for (entity, anchor, core, economy, footprint, build, attack_profile, attack_state, upgrade, ledger) in &ballistas {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);

        checksum.accumulate(core.defenses_life.to_bits() as u64);
        checksum.accumulate(core.watch_range.to_bits() as u64);
        checksum.accumulate(core.health.current.to_bits() as u64);
        checksum.accumulate(core.health.max.to_bits() as u64);

        checksum.accumulate(economy.energy_cost as u64);
        checksum.accumulate(economy.wood_cost as u64);
        checksum.accumulate(economy.stone_cost as u64);
        checksum.accumulate(economy.iron_cost as u64);
        checksum.accumulate(economy.oil_cost as u64);
        checksum.accumulate(economy.gold_cost as u64);
        checksum.accumulate(economy.workers as u64);
        checksum.accumulate(economy.research_cost_gold as u64);
        checksum.accumulate(u64::from(economy.upgrade_target_executor));

        checksum.accumulate(footprint.size_tiles as u64);
        checksum.accumulate(footprint.watch_range.to_bits() as u64);

        checksum.accumulate(build.build_ticks_remaining as u64);
        checksum.accumulate(u64::from(build.completed));

        checksum.accumulate(attack_profile.attack_damage.to_bits() as u64);
        checksum.accumulate(attack_profile.attack_range.to_bits() as u64);
        checksum.accumulate(attack_profile.attack_speed.to_bits() as u64);
        checksum.accumulate(attack_profile.aoe_radius.to_bits() as u64);
        checksum.accumulate(attack_profile.direct_hit_damage_min.to_bits() as u64);
        checksum.accumulate(attack_profile.direct_hit_damage_max.to_bits() as u64);
        checksum.accumulate(attack_profile.effective_hit_range.to_bits() as u64);
        checksum.accumulate(attack_profile.knockback_impulse.to_bits() as u64);
        checksum.accumulate(attack_profile.noise as u64);
        checksum.accumulate(match attack_profile.targeting_mode {
            GreatBallistaTargetingMode::Nearest => 1,
            GreatBallistaTargetingMode::HighestThreat => 2,
        });

        checksum.accumulate(attack_state.cooldown_ticks_remaining as u64);
        checksum.accumulate(attack_state.total_shots_fired as u64);
        checksum.accumulate(attack_state.total_shots_hit as u64);
        checksum.accumulate(attack_state.total_noise_generated as u64);
        checksum.accumulate(attack_state.current_target.map(Entity::to_bits).unwrap_or(0));

        checksum.accumulate(u64::from(upgrade.executor_research_completed));
        checksum.accumulate(u64::from(upgrade.can_upgrade_to_executor));

        checksum.accumulate(ledger.total_direct_hit_damage_applied.to_bits() as u64);
        checksum.accumulate(ledger.total_splash_damage_applied.to_bits() as u64);
        checksum.accumulate(ledger.total_knockback_applied.to_bits() as u64);
    }

    checksum.accumulate(u64::from(research.unlocked));

    checksum.accumulate(occupancy.blocked_tiles.len() as u64);
    for (x, y) in &occupancy.blocked_tiles {
        checksum.accumulate(*x as u64);
        checksum.accumulate(*y as u64);
    }

    checksum.accumulate(blockers.blockers.len() as u64);
    for ((x, y), kind) in &blockers.blockers {
        checksum.accumulate(*x as u64);
        checksum.accumulate(*y as u64);
        checksum.accumulate(match kind {
            LineOfFireBlockerKind::Mountain => 1,
            LineOfFireBlockerKind::Forest => 2,
            LineOfFireBlockerKind::LargeRock => 3,
        });
    }

    checksum.accumulate(claims.claims.len() as u64);
    for (entity, anchor) in &claims.claims {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);
    }

    for (entity, profile, pos) in &target_entities {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(profile.threat_level as u64);
        checksum.accumulate(u64::from(profile.is_village_of_doom));
        checksum.accumulate(u64::from(profile.is_large_or_stationary));
        checksum.accumulate(u64::from(profile.is_fast_or_airborne));
        checksum.accumulate(pos.x as u64);
        checksum.accumulate(pos.y as u64);
    }
}

pub struct GreatBallistaPlugin;

impl Plugin for GreatBallistaPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GreatBallistaResearchState>()
            .init_resource::<TileOccupancy>()
            .init_resource::<LineOfFireBlockers>()
            .init_resource::<GreatBallistaPlacementClaims>()
            .add_event::<SetTileBlockedEvent>()
            .add_event::<SetLineOfFireBlockerEvent>()
            .add_event::<SetGreatBallistaResearchUnlockedEvent>()
            .add_event::<PlaceGreatBallistaEvent>()
            .add_event::<GreatBallistaPlacementRejectedEvent>()
            .add_event::<SetGreatBallistaTargetingModeEvent>()
            .add_event::<SetGreatBallistaExecutorResearchCompletedEvent>()
            .add_event::<SetGreatBallistaTargetProfileEvent>()
            .add_systems(
                FixedUpdate,
                (
                    apply_tile_block_events_system,
                    apply_line_of_fire_blockers_system,
                    apply_research_unlock_event_system,
                    place_great_ballista_system,
                    apply_target_profile_events_system,
                    set_targeting_mode_system,
                    set_upgrade_research_system,
                    great_ballista_build_tick_system,
                    great_ballista_upgrade_gate_system,
                    great_ballista_target_and_attack_system,
                    great_ballista_checksum_system,
                )
                    .chain(),
            );
    }
}