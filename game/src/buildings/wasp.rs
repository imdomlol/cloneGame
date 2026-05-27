// Sources: vault/buildings/wasp.md

use std::collections::{BTreeMap, BTreeSet};

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{Health, SimChecksumState};

const WASP_HP: I32F32 = I32F32::lit("300");
const WASP_DEFENSES_LIFE: I32F32 = I32F32::lit("0");
const WASP_WATCH_RANGE: I32F32 = I32F32::lit("8");
const WASP_ENERGY_COST: i32 = 1;
const WASP_WOOD_COST: i32 = 0;
const WASP_STONE_COST: i32 = 0;
const WASP_IRON_COST: i32 = 10;
const WASP_OIL_COST: i32 = 0;
const WASP_GOLD_COST: i32 = 320;
const WASP_BUILD_TIME_SECONDS: i32 = 32;
const WASP_MAINTENANCE_GOLD: i32 = 2;
const WASP_WORKERS: i32 = 0;
const WASP_SIZE_TILES: i32 = 1;
const WASP_ATTACK_DAMAGE: I32F32 = I32F32::lit("20");
const WASP_ATTACK_SPEED: I32F32 = I32F32::lit("3");
const WASP_ATTACK_RANGE: I32F32 = I32F32::lit("6");
const WASP_NOISE: i32 = 0;
const SIM_HZ: i32 = 25;

#[derive(Component, Default)]
pub struct Wasp;

#[derive(Component, Clone, Copy, Default)]
pub struct BuildingAnchor {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Clone, Copy)]
pub struct WaspCore {
    pub defenses_life: I32F32,
    pub watch_range: I32F32,
    pub health: Health,
}

impl Default for WaspCore {
    fn default() -> Self {
        Self {
            defenses_life: WASP_DEFENSES_LIFE,
            watch_range: WASP_WATCH_RANGE,
            health: Health::full(WASP_HP),
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct WaspEconomy {
    pub energy_cost: i32,
    pub wood_cost: i32,
    pub stone_cost: i32,
    pub iron_cost: i32,
    pub oil_cost: i32,
    pub gold_cost: i32,
    pub maintenance_gold: i32,
    pub workers: i32,
}

impl Default for WaspEconomy {
    fn default() -> Self {
        Self {
            energy_cost: WASP_ENERGY_COST,
            wood_cost: WASP_WOOD_COST,
            stone_cost: WASP_STONE_COST,
            iron_cost: WASP_IRON_COST,
            oil_cost: WASP_OIL_COST,
            gold_cost: WASP_GOLD_COST,
            maintenance_gold: WASP_MAINTENANCE_GOLD,
            workers: WASP_WORKERS,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct WaspFootprint {
    pub size_tiles: i32,
    pub blocks_ground_movement: bool,
    pub harpy_can_jump_over: bool,
}

impl Default for WaspFootprint {
    fn default() -> Self {
        Self {
            size_tiles: WASP_SIZE_TILES,
            blocks_ground_movement: true,
            harpy_can_jump_over: true,
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct WaspBuildState {
    pub build_ticks_remaining: i32,
    pub completed: bool,
    pub powered: bool,
}

#[derive(Component, Clone, Copy)]
pub struct WaspAttackProfile {
    pub attack_damage: I32F32,
    pub attack_speed: I32F32,
    pub attack_range: I32F32,
    pub noise: i32,
}

impl Default for WaspAttackProfile {
    fn default() -> Self {
        Self {
            attack_damage: WASP_ATTACK_DAMAGE,
            attack_speed: WASP_ATTACK_SPEED,
            attack_range: WASP_ATTACK_RANGE,
            noise: WASP_NOISE,
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct WaspAttackState {
    pub cooldown_ticks_remaining: i32,
    pub shots_fired: i64,
    pub total_noise_generated: i64,
    pub current_target: Option<Entity>,
}

#[derive(Component, Clone, Copy, Default)]
pub struct WaspState {
    pub is_non_living_object: bool,
    pub irreversibly_destroyed: bool,
    pub counts_as_attack_tower: bool,
}

#[derive(Component, Clone, Copy, Default)]
pub struct WaspCombatLedger {
    pub total_damage_dealt: I32F32,
    pub infected_retarged_count: i64,
}

#[derive(Component, Clone, Copy, Default)]
pub struct WaspTargetPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Clone, Copy, Default)]
pub struct WaspTargetProfile {
    pub is_living: bool,
    pub is_infected: bool,
    pub is_harpy: bool,
    pub attack_damage: I32F32,
}

#[derive(Component, Clone, Copy)]
pub struct WaspTargetHealth {
    pub health: Health,
}

impl Default for WaspTargetHealth {
    fn default() -> Self {
        Self {
            health: Health::full(I32F32::lit("1")),
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct WaspTargetAggro {
    pub original_target: Option<Entity>,
    pub current_target: Option<Entity>,
}

#[derive(Resource, Clone, Copy, Default)]
pub struct WaspResearchState {
    pub unlocked: bool,
}

#[derive(Resource, Default, Clone)]
pub struct TileOccupancy {
    pub blocked_tiles: BTreeSet<(i32, i32)>,
}

#[derive(Resource, Default, Clone)]
pub struct WaspPlacementClaims {
    pub claims: BTreeMap<Entity, BuildingAnchor>,
}

#[derive(Event, Clone, Copy)]
pub struct SetWaspResearchUnlockedEvent {
    pub unlocked: bool,
}

#[derive(Event, Clone, Copy)]
pub struct SetTileBlockedEvent {
    pub tile_x: i32,
    pub tile_y: i32,
    pub blocked: bool,
}

#[derive(Event, Clone, Copy)]
pub struct PlaceWaspEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct WaspPlacementRejectedEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct SetWaspPoweredEvent {
    pub entity: Entity,
    pub powered: bool,
}

#[derive(Event, Clone, Copy)]
pub struct DamageWaspEvent {
    pub entity: Entity,
    pub damage: I32F32,
    pub attacker_is_infected: bool,
}

#[derive(Event, Clone, Copy)]
pub struct SetWaspTargetEvent {
    pub entity: Entity,
    pub x: i32,
    pub y: i32,
    pub hp: I32F32,
    pub is_living: bool,
    pub is_infected: bool,
    pub is_harpy: bool,
    pub attack_damage: I32F32,
    pub current_target: Option<Entity>,
    pub original_target: Option<Entity>,
}

fn seconds_to_ticks(seconds: i32) -> i32 {
    seconds * SIM_HZ
}

fn attack_cooldown_ticks(profile: &WaspAttackProfile) -> i32 {
    let ticks = I32F32::from_num(SIM_HZ) / profile.attack_speed;
    ticks.ceil().to_num::<i32>().max(1)
}

fn footprint_tiles(anchor: BuildingAnchor) -> impl Iterator<Item = (i32, i32)> {
    let min_x = anchor.x;
    let min_y = anchor.y;
    let max_x = anchor.x + WASP_SIZE_TILES - 1;
    let max_y = anchor.y + WASP_SIZE_TILES - 1;
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

fn distance_squared(a: BuildingAnchor, b: WaspTargetPosition) -> i64 {
    let dx = (a.x - b.x) as i64;
    let dy = (a.y - b.y) as i64;
    dx * dx + dy * dy
}

fn apply_research_unlock_event_system(
    mut events: EventReader<SetWaspResearchUnlockedEvent>,
    mut research: ResMut<WaspResearchState>,
) {
    for ev in events.read() {
        research.unlocked = ev.unlocked;
    }
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

fn place_wasp_system(
    mut commands: Commands,
    mut events: EventReader<PlaceWaspEvent>,
    mut rejected: EventWriter<WaspPlacementRejectedEvent>,
    research: Res<WaspResearchState>,
    occupancy: Res<TileOccupancy>,
    mut claims: ResMut<WaspPlacementClaims>,
) {
    for ev in events.read() {
        if !research.unlocked {
            rejected.send(WaspPlacementRejectedEvent {
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
            rejected.send(WaspPlacementRejectedEvent {
                tile_x: ev.tile_x,
                tile_y: ev.tile_y,
            });
            continue;
        }

        let entity = commands
            .spawn((
                Wasp,
                anchor,
                WaspCore::default(),
                WaspEconomy::default(),
                WaspFootprint::default(),
                WaspBuildState {
                    build_ticks_remaining: seconds_to_ticks(WASP_BUILD_TIME_SECONDS),
                    completed: false,
                    powered: true,
                },
                WaspAttackProfile::default(),
                WaspAttackState::default(),
                WaspState {
                    is_non_living_object: true,
                    irreversibly_destroyed: false,
                    counts_as_attack_tower: true,
                },
                WaspCombatLedger::default(),
            ))
            .id();

        claims.claims.insert(entity, anchor);
    }
}

fn set_wasp_powered_system(
    mut events: EventReader<SetWaspPoweredEvent>,
    mut wasps: Query<&mut WaspBuildState, With<Wasp>>,
) {
    for ev in events.read() {
        let Ok(mut build) = wasps.get_mut(ev.entity) else {
            continue;
        };
        build.powered = ev.powered;
    }
}

fn set_wasp_target_event_system(
    mut commands: Commands,
    mut events: EventReader<SetWaspTargetEvent>,
) {
    for ev in events.read() {
        commands.entity(ev.entity).insert((
            WaspTargetPosition { x: ev.x, y: ev.y },
            WaspTargetProfile {
                is_living: ev.is_living,
                is_infected: ev.is_infected,
                is_harpy: ev.is_harpy,
                attack_damage: ev.attack_damage.max(I32F32::ZERO),
            },
            WaspTargetHealth {
                health: Health::full(ev.hp.max(I32F32::ZERO)),
            },
            WaspTargetAggro {
                original_target: ev.original_target,
                current_target: ev.current_target,
            },
        ));
    }
}

fn wasp_build_tick_system(
    mut wasps: Query<&mut WaspBuildState, With<Wasp>>,
) {
    for mut build in &mut wasps {
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

fn wasp_attack_system(
    mut wasps: Query<
        (
            Entity,
            &BuildingAnchor,
            &WaspBuildState,
            &WaspState,
            &WaspAttackProfile,
            &mut WaspAttackState,
            &mut WaspCombatLedger,
        ),
        With<Wasp>,
    >,
    mut targets: Query<
        (
            Entity,
            &WaspTargetPosition,
            &WaspTargetProfile,
            &mut WaspTargetHealth,
            &mut WaspTargetAggro,
        ),
        Without<Wasp>,
    >,
) {
    for (wasp_entity, anchor, build, state, profile, mut attack_state, mut ledger) in &mut wasps {
        if !build.completed || !build.powered || state.irreversibly_destroyed {
            continue;
        }

        if attack_state.cooldown_ticks_remaining > 0 {
            attack_state.cooldown_ticks_remaining -= 1;
            continue;
        }

        let max_range_sq = profile.attack_range * profile.attack_range;
        let mut selected_living: Option<(Entity, i64)> = None;
        let mut selected_non_living: Option<(Entity, i64)> = None;

        for (target_entity, position, target_profile, _, _) in &mut targets {
            let dist_sq = distance_squared(*anchor, *position);
            if I32F32::from_num(dist_sq) > max_range_sq {
                continue;
            }

            if target_profile.is_living {
                match selected_living {
                    None => selected_living = Some((target_entity, dist_sq)),
                    Some((_, selected_dist_sq)) => {
                        if dist_sq < selected_dist_sq {
                            selected_living = Some((target_entity, dist_sq));
                        }
                    }
                }
            } else {
                match selected_non_living {
                    None => selected_non_living = Some((target_entity, dist_sq)),
                    Some((_, selected_dist_sq)) => {
                        if dist_sq < selected_dist_sq {
                            selected_non_living = Some((target_entity, dist_sq));
                        }
                    }
                }
            }
        }

        let selected_target = selected_living.or(selected_non_living).map(|v| v.0);
        let Some(target_entity) = selected_target else {
            attack_state.current_target = None;
            continue;
        };

        let Ok((_, _, target_profile, mut target_health, mut target_aggro)) = targets.get_mut(target_entity) else {
            attack_state.current_target = None;
            continue;
        };

        attack_state.current_target = Some(target_entity);
        attack_state.shots_fired += 1;
        attack_state.total_noise_generated += profile.noise as i64;
        attack_state.cooldown_ticks_remaining = attack_cooldown_ticks(profile);

        if target_health.health.current <= I32F32::ZERO {
            continue;
        }

        target_health.health.current = (target_health.health.current - profile.attack_damage).max(I32F32::ZERO);
        ledger.total_damage_dealt += profile.attack_damage;

        if target_profile.is_infected
            && target_profile.attack_damage > I32F32::ZERO
            && target_aggro.current_target != Some(wasp_entity)
        {
            target_aggro.current_target = Some(wasp_entity);
            ledger.infected_retarged_count += 1;
        }
    }
}

fn apply_damage_wasp_system(
    mut events: EventReader<DamageWaspEvent>,
    mut wasps: Query<(&mut WaspCore, &mut WaspState), With<Wasp>>,
) {
    for ev in events.read() {
        let Ok((mut core, mut state)) = wasps.get_mut(ev.entity) else {
            continue;
        };

        if state.irreversibly_destroyed {
            continue;
        }

        let mut remaining = ev.damage.max(I32F32::ZERO);
        if remaining <= I32F32::ZERO {
            continue;
        }

        if core.defenses_life > I32F32::ZERO {
            let absorbed = remaining.min(core.defenses_life);
            core.defenses_life -= absorbed;
            remaining -= absorbed;
        }

        if remaining > I32F32::ZERO {
            core.health.current -= remaining;
        }

        if ev.attacker_is_infected && core.health.current <= I32F32::ZERO {
            core.health.current = I32F32::ZERO;
            state.irreversibly_destroyed = true;
        }
    }
}

fn wasp_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    wasps: Query<
        (
            Entity,
            &BuildingAnchor,
            &WaspCore,
            &WaspEconomy,
            &WaspFootprint,
            &WaspBuildState,
            &WaspAttackProfile,
            &WaspAttackState,
            &WaspState,
            &WaspCombatLedger,
        ),
        With<Wasp>,
    >,
    targets: Query<
        (
            Entity,
            &WaspTargetPosition,
            &WaspTargetProfile,
            &WaspTargetHealth,
            &WaspTargetAggro,
        ),
        Without<Wasp>,
    >,
    research: Res<WaspResearchState>,
    occupancy: Res<TileOccupancy>,
    claims: Res<WaspPlacementClaims>,
) {
    for (entity, anchor, core, economy, footprint, build, profile, attack, state, ledger) in &wasps {
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
        checksum.accumulate(economy.maintenance_gold as u64);
        checksum.accumulate(economy.workers as u64);

        checksum.accumulate(footprint.size_tiles as u64);
        checksum.accumulate(u64::from(footprint.blocks_ground_movement));
        checksum.accumulate(u64::from(footprint.harpy_can_jump_over));

        checksum.accumulate(build.build_ticks_remaining as u64);
        checksum.accumulate(u64::from(build.completed));
        checksum.accumulate(u64::from(build.powered));

        checksum.accumulate(profile.attack_damage.to_bits() as u64);
        checksum.accumulate(profile.attack_speed.to_bits() as u64);
        checksum.accumulate(profile.attack_range.to_bits() as u64);
        checksum.accumulate(profile.noise as u64);

        checksum.accumulate(attack.cooldown_ticks_remaining as u64);
        checksum.accumulate(attack.shots_fired as u64);
        checksum.accumulate(attack.total_noise_generated as u64);
        checksum.accumulate(attack.current_target.map(Entity::to_bits).unwrap_or(0));

        checksum.accumulate(u64::from(state.is_non_living_object));
        checksum.accumulate(u64::from(state.irreversibly_destroyed));
        checksum.accumulate(u64::from(state.counts_as_attack_tower));

        checksum.accumulate(ledger.total_damage_dealt.to_bits() as u64);
        checksum.accumulate(ledger.infected_retarged_count as u64);
    }

    for (entity, pos, profile, health, aggro) in &targets {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(pos.x as u64);
        checksum.accumulate(pos.y as u64);
        checksum.accumulate(u64::from(profile.is_living));
        checksum.accumulate(u64::from(profile.is_infected));
        checksum.accumulate(u64::from(profile.is_harpy));
        checksum.accumulate(profile.attack_damage.to_bits() as u64);
        checksum.accumulate(health.health.current.to_bits() as u64);
        checksum.accumulate(health.health.max.to_bits() as u64);
        checksum.accumulate(aggro.original_target.map(Entity::to_bits).unwrap_or(0));
        checksum.accumulate(aggro.current_target.map(Entity::to_bits).unwrap_or(0));
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

pub struct WaspPlugin;

impl Plugin for WaspPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WaspResearchState>()
            .init_resource::<TileOccupancy>()
            .init_resource::<WaspPlacementClaims>()
            .add_event::<SetWaspResearchUnlockedEvent>()
            .add_event::<SetTileBlockedEvent>()
            .add_event::<PlaceWaspEvent>()
            .add_event::<WaspPlacementRejectedEvent>()
            .add_event::<SetWaspPoweredEvent>()
            .add_event::<DamageWaspEvent>()
            .add_event::<SetWaspTargetEvent>()
            .add_systems(
                FixedUpdate,
                (
                    apply_research_unlock_event_system,
                    apply_tile_block_events_system,
                    place_wasp_system,
                    set_wasp_powered_system,
                    set_wasp_target_event_system,
                    wasp_build_tick_system,
                    wasp_attack_system,
                    apply_damage_wasp_system,
                    wasp_checksum_system,
                )
                    .chain(),
            );
    }
}