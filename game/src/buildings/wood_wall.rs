// Sources: vault/buildings/wood_wall.md

use std::collections::BTreeMap;

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{Health, SimChecksumState};

const WOOD_WALL_HP: I32F32 = I32F32::lit("400");
const WOOD_WALL_DEFENSES_LIFE: I32F32 = I32F32::lit("0");
const WOOD_WALL_WATCH_RANGE: I32F32 = I32F32::lit("0");

const WOOD_WALL_ENERGY_COST: i32 = 0;
const WOOD_WALL_WOOD_COST: i32 = 3;
const WOOD_WALL_STONE_COST: i32 = 0;
const WOOD_WALL_IRON_COST: i32 = 0;
const WOOD_WALL_OIL_COST: i32 = 0;
const WOOD_WALL_GOLD_COST: i32 = 10;
const WOOD_WALL_BUILD_TIME_SECONDS: i32 = 15;
const WOOD_WALL_WORKERS: i32 = 0;

const WOOD_WALL_PFOOD: i32 = 0;
const WOOD_WALL_PWOOD: i32 = 0;
const WOOD_WALL_PSTONE: i32 = 0;
const WOOD_WALL_PIRON: i32 = 0;
const WOOD_WALL_POIL: i32 = 0;
const WOOD_WALL_PGOLD: i32 = 0;
const WOOD_WALL_PENERGY: i32 = 0;
const WOOD_WALL_PCOLONISTS: i32 = 0;

const WOOD_WALL_SIZE_X_TILES: i32 = 1;
const WOOD_WALL_SIZE_Y_TILES: i32 = 1;

const WOOD_WALL_CONSTRUCTION_HP_PER_SECOND: I32F32 = I32F32::lit("26.67");
const WOOD_WALL_DEMOLISH_REFUND_WOOD: i32 = 2;
const WOOD_WALL_DEMOLISH_REFUND_GOLD: i32 = 5;

const SIM_HZ: i32 = 25;

#[derive(Component, Default)]
pub struct WoodWall;

#[derive(Component, Clone, Copy, Default)]
pub struct BuildingAnchor {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Clone, Copy)]
pub struct WoodWallCore {
    pub defenses_life: I32F32,
    pub watch_range: I32F32,
    pub health: Health,
}

impl Default for WoodWallCore {
    fn default() -> Self {
        Self {
            defenses_life: WOOD_WALL_DEFENSES_LIFE,
            watch_range: WOOD_WALL_WATCH_RANGE,
            health: Health {
                current: I32F32::ZERO,
                max: WOOD_WALL_HP,
            },
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct WoodWallEconomy {
    pub energy_cost: i32,
    pub wood_cost: i32,
    pub stone_cost: i32,
    pub iron_cost: i32,
    pub oil_cost: i32,
    pub gold_cost: i32,
    pub build_time_seconds: i32,
    pub workers: i32,
    pub pfood: i32,
    pub pwood: i32,
    pub pstone: i32,
    pub piron: i32,
    pub poil: i32,
    pub pgold: i32,
    pub penergy: i32,
    pub pcolonists: i32,
    pub demolish_refund_wood: i32,
    pub demolish_refund_gold: i32,
}

impl Default for WoodWallEconomy {
    fn default() -> Self {
        Self {
            energy_cost: WOOD_WALL_ENERGY_COST,
            wood_cost: WOOD_WALL_WOOD_COST,
            stone_cost: WOOD_WALL_STONE_COST,
            iron_cost: WOOD_WALL_IRON_COST,
            oil_cost: WOOD_WALL_OIL_COST,
            gold_cost: WOOD_WALL_GOLD_COST,
            build_time_seconds: WOOD_WALL_BUILD_TIME_SECONDS,
            workers: WOOD_WALL_WORKERS,
            pfood: WOOD_WALL_PFOOD,
            pwood: WOOD_WALL_PWOOD,
            pstone: WOOD_WALL_PSTONE,
            piron: WOOD_WALL_PIRON,
            poil: WOOD_WALL_POIL,
            pgold: WOOD_WALL_PGOLD,
            penergy: WOOD_WALL_PENERGY,
            pcolonists: WOOD_WALL_PCOLONISTS,
            demolish_refund_wood: WOOD_WALL_DEMOLISH_REFUND_WOOD,
            demolish_refund_gold: WOOD_WALL_DEMOLISH_REFUND_GOLD,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct WoodWallFootprint {
    pub size_x_tiles: i32,
    pub size_y_tiles: i32,
}

impl Default for WoodWallFootprint {
    fn default() -> Self {
        Self {
            size_x_tiles: WOOD_WALL_SIZE_X_TILES,
            size_y_tiles: WOOD_WALL_SIZE_Y_TILES,
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct WoodWallBuildState {
    pub build_ticks_remaining: i32,
    pub completed: bool,
}

#[derive(Component, Clone, Copy, Default)]
pub struct WoodWallCombatState {
    pub infected_in_contact: bool,
    pub defended_by_friendly_units: bool,
    pub noise_from_attacks: I32F32,
}

#[derive(Component, Clone, Copy, Default)]
pub struct WoodWallBehavior {
    pub infected_prefer_attacking_wall: bool,
    pub most_units_can_fire_over_wall: bool,
    pub can_upgrade_to_stone_wall: bool,
    pub lucifer_cannot_attack_over_wall: bool,
}

#[derive(Resource, Default, Clone)]
pub struct WoodWallPlacementClaims {
    pub claims: BTreeMap<Entity, BuildingAnchor>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WoodWallPlacementRejectionReason {
    Occupied,
    CreatesThreeByThreeWallBlock,
    CreatesRightAngleThreeRowsJunction,
}

#[derive(Event, Clone, Copy)]
pub struct PlaceWoodWallEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct DamageWoodWallEvent {
    pub wall_entity: Entity,
    pub damage: I32F32,
    pub attacker_is_lucifer: bool,
    pub attack_over_wall: bool,
}

#[derive(Event, Clone, Copy)]
pub struct SetWoodWallInfectedInContactEvent {
    pub wall_entity: Entity,
    pub infected_in_contact: bool,
}

#[derive(Event, Clone, Copy)]
pub struct SetWoodWallDefendedByUnitsEvent {
    pub wall_entity: Entity,
    pub defended_by_friendly_units: bool,
}

#[derive(Event, Clone, Copy)]
pub struct DemolishWoodWallEvent {
    pub wall_entity: Entity,
}

#[derive(Event, Clone, Copy)]
pub struct WoodWallDemolishedRefundEvent {
    pub wood: i32,
    pub gold: i32,
}

#[derive(Event, Clone, Copy)]
pub struct UpgradeWoodWallToStoneWallEvent {
    pub wall_entity: Entity,
}

#[derive(Event, Clone, Copy)]
pub struct WoodWallPlacementRejectedEvent {
    pub tile_x: i32,
    pub tile_y: i32,
    pub reason: WoodWallPlacementRejectionReason,
}

fn seconds_to_ticks(seconds: i32) -> i32 {
    seconds * SIM_HZ
}

fn has_wall_at(claims: &WoodWallPlacementClaims, x: i32, y: i32) -> bool {
    for anchor in claims.claims.values() {
        if anchor.x == x && anchor.y == y {
            return true;
        }
    }
    false
}

fn creates_full_three_by_three(claims: &WoodWallPlacementClaims, x: i32, y: i32) -> bool {
    for center_x in (x - 1)..=(x + 1) {
        for center_y in (y - 1)..=(y + 1) {
            let mut full_block = true;
            for check_x in (center_x - 1)..=(center_x + 1) {
                for check_y in (center_y - 1)..=(center_y + 1) {
                    if check_x == x && check_y == y {
                        continue;
                    }
                    if !has_wall_at(claims, check_x, check_y) {
                        full_block = false;
                        break;
                    }
                }
                if !full_block {
                    break;
                }
            }
            if full_block {
                return true;
            }
        }
    }
    false
}

fn creates_right_angle_three_rows_junction(claims: &WoodWallPlacementClaims, x: i32, y: i32) -> bool {
    let has_left = has_wall_at(claims, x - 1, y);
    let has_right = has_wall_at(claims, x + 1, y);
    let has_up = has_wall_at(claims, x, y + 1);
    let has_down = has_wall_at(claims, x, y - 1);

    (has_left && has_right && (has_up || has_down)) || (has_up && has_down && (has_left || has_right))
}

fn validate_wood_wall_placement(
    claims: &WoodWallPlacementClaims,
    x: i32,
    y: i32,
) -> Result<(), WoodWallPlacementRejectionReason> {
    if has_wall_at(claims, x, y) {
        return Err(WoodWallPlacementRejectionReason::Occupied);
    }

    if creates_full_three_by_three(claims, x, y) {
        return Err(WoodWallPlacementRejectionReason::CreatesThreeByThreeWallBlock);
    }

    if creates_right_angle_three_rows_junction(claims, x, y) {
        return Err(WoodWallPlacementRejectionReason::CreatesRightAngleThreeRowsJunction);
    }

    Ok(())
}

fn place_wood_wall_system(
    mut commands: Commands,
    mut events: EventReader<PlaceWoodWallEvent>,
    mut rejected: EventWriter<WoodWallPlacementRejectedEvent>,
    mut claims: ResMut<WoodWallPlacementClaims>,
) {
    for ev in events.read() {
        let validation = validate_wood_wall_placement(&claims, ev.tile_x, ev.tile_y);
        if let Err(reason) = validation {
            rejected.send(WoodWallPlacementRejectedEvent {
                tile_x: ev.tile_x,
                tile_y: ev.tile_y,
                reason,
            });
            continue;
        }

        let entity = commands
            .spawn((
                WoodWall,
                BuildingAnchor {
                    x: ev.tile_x,
                    y: ev.tile_y,
                },
                WoodWallCore::default(),
                WoodWallEconomy::default(),
                WoodWallFootprint::default(),
                WoodWallBuildState {
                    build_ticks_remaining: seconds_to_ticks(WOOD_WALL_BUILD_TIME_SECONDS),
                    completed: false,
                },
                WoodWallCombatState::default(),
                WoodWallBehavior {
                    infected_prefer_attacking_wall: true,
                    most_units_can_fire_over_wall: true,
                    can_upgrade_to_stone_wall: true,
                    lucifer_cannot_attack_over_wall: true,
                },
            ))
            .id();

        claims.claims.insert(
            entity,
            BuildingAnchor {
                x: ev.tile_x,
                y: ev.tile_y,
            },
        );
    }
}

fn wood_wall_build_tick_system(
    mut walls: Query<(&mut WoodWallCore, &mut WoodWallBuildState), With<WoodWall>>,
) {
    let hp_per_tick = WOOD_WALL_CONSTRUCTION_HP_PER_SECOND / I32F32::from_num(SIM_HZ);

    for (mut core, mut build) in &mut walls {
        if build.completed {
            continue;
        }

        if build.build_ticks_remaining > 0 {
            build.build_ticks_remaining -= 1;
        }

        core.health.current = (core.health.current + hp_per_tick).min(core.health.max);

        if build.build_ticks_remaining <= 0 {
            build.build_ticks_remaining = 0;
            build.completed = true;
        }
    }
}

fn set_wood_wall_infected_contact_system(
    mut events: EventReader<SetWoodWallInfectedInContactEvent>,
    mut walls: Query<(&WoodWallBuildState, &mut WoodWallCombatState), With<WoodWall>>,
) {
    for ev in events.read() {
        let Ok((build, mut combat)) = walls.get_mut(ev.wall_entity) else {
            continue;
        };

        if !build.completed {
            combat.infected_in_contact = false;
            continue;
        }

        combat.infected_in_contact = ev.infected_in_contact;
    }
}

fn set_wood_wall_defended_by_units_system(
    mut events: EventReader<SetWoodWallDefendedByUnitsEvent>,
    mut walls: Query<(&WoodWallBuildState, &mut WoodWallCombatState), With<WoodWall>>,
) {
    for ev in events.read() {
        let Ok((build, mut combat)) = walls.get_mut(ev.wall_entity) else {
            continue;
        };

        if !build.completed {
            combat.defended_by_friendly_units = false;
            continue;
        }

        combat.defended_by_friendly_units = ev.defended_by_friendly_units;
    }
}

fn wood_wall_noise_from_attack_activity_system(
    mut walls: Query<(&WoodWallBuildState, &mut WoodWallCombatState), With<WoodWall>>,
) {
    for (build, mut combat) in &mut walls {
        if !build.completed {
            combat.noise_from_attacks = I32F32::ZERO;
            continue;
        }

        if combat.infected_in_contact && !combat.defended_by_friendly_units {
            combat.noise_from_attacks = I32F32::lit("1");
        } else {
            combat.noise_from_attacks = I32F32::ZERO;
        }
    }
}

fn damage_wood_wall_system(
    mut commands: Commands,
    mut events: EventReader<DamageWoodWallEvent>,
    mut walls: Query<(Entity, &WoodWallBehavior, &mut WoodWallCore), With<WoodWall>>,
    mut claims: ResMut<WoodWallPlacementClaims>,
) {
    for ev in events.read() {
        let Ok((entity, behavior, mut core)) = walls.get_mut(ev.wall_entity) else {
            continue;
        };

        if behavior.lucifer_cannot_attack_over_wall && ev.attacker_is_lucifer && ev.attack_over_wall {
            continue;
        }

        let mut incoming = ev.damage;
        if incoming <= I32F32::ZERO {
            continue;
        }

        if core.defenses_life > I32F32::ZERO {
            let absorbed = incoming.min(core.defenses_life);
            core.defenses_life -= absorbed;
            incoming -= absorbed;
        }

        if incoming > I32F32::ZERO {
            core.health.current -= incoming;
            if core.health.current <= I32F32::ZERO {
                claims.claims.remove(&entity);
                commands.entity(entity).despawn();
            }
        }
    }
}

fn demolish_wood_wall_system(
    mut commands: Commands,
    mut events: EventReader<DemolishWoodWallEvent>,
    mut refunds: EventWriter<WoodWallDemolishedRefundEvent>,
    walls: Query<&WoodWallEconomy, With<WoodWall>>,
    mut claims: ResMut<WoodWallPlacementClaims>,
) {
    for ev in events.read() {
        let Ok(economy) = walls.get(ev.wall_entity) else {
            continue;
        };

        refunds.send(WoodWallDemolishedRefundEvent {
            wood: economy.demolish_refund_wood,
            gold: economy.demolish_refund_gold,
        });

        claims.claims.remove(&ev.wall_entity);
        commands.entity(ev.wall_entity).despawn();
    }
}

fn wood_wall_cleanup_claims_system(
    existing_walls: Query<Entity, With<WoodWall>>,
    mut claims: ResMut<WoodWallPlacementClaims>,
) {
    claims
        .claims
        .retain(|entity, _| existing_walls.get(*entity).is_ok());
}

fn wood_wall_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    walls: Query<
        (
            Entity,
            &BuildingAnchor,
            &WoodWallCore,
            &WoodWallEconomy,
            &WoodWallFootprint,
            &WoodWallBuildState,
            &WoodWallCombatState,
            &WoodWallBehavior,
        ),
        With<WoodWall>,
    >,
    claims: Res<WoodWallPlacementClaims>,
) {
    for (entity, anchor, core, economy, footprint, build, combat, behavior) in &walls {
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
        checksum.accumulate(economy.build_time_seconds as u64);
        checksum.accumulate(economy.workers as u64);
        checksum.accumulate(economy.pfood as u64);
        checksum.accumulate(economy.pwood as u64);
        checksum.accumulate(economy.pstone as u64);
        checksum.accumulate(economy.piron as u64);
        checksum.accumulate(economy.poil as u64);
        checksum.accumulate(economy.pgold as u64);
        checksum.accumulate(economy.penergy as u64);
        checksum.accumulate(economy.pcolonists as u64);
        checksum.accumulate(economy.demolish_refund_wood as u64);
        checksum.accumulate(economy.demolish_refund_gold as u64);

        checksum.accumulate(footprint.size_x_tiles as u64);
        checksum.accumulate(footprint.size_y_tiles as u64);

        checksum.accumulate(build.build_ticks_remaining as u64);
        checksum.accumulate(u64::from(build.completed));

        checksum.accumulate(u64::from(combat.infected_in_contact));
        checksum.accumulate(u64::from(combat.defended_by_friendly_units));
        checksum.accumulate(combat.noise_from_attacks.to_bits() as u64);

        checksum.accumulate(u64::from(behavior.infected_prefer_attacking_wall));
        checksum.accumulate(u64::from(behavior.most_units_can_fire_over_wall));
        checksum.accumulate(u64::from(behavior.can_upgrade_to_stone_wall));
        checksum.accumulate(u64::from(behavior.lucifer_cannot_attack_over_wall));
    }

    checksum.accumulate(claims.claims.len() as u64);
    for (entity, anchor) in &claims.claims {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);
    }
}

pub struct WoodWallPlugin;

impl Plugin for WoodWallPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WoodWallPlacementClaims>()
            .add_event::<PlaceWoodWallEvent>()
            .add_event::<DamageWoodWallEvent>()
            .add_event::<SetWoodWallInfectedInContactEvent>()
            .add_event::<SetWoodWallDefendedByUnitsEvent>()
            .add_event::<DemolishWoodWallEvent>()
            .add_event::<WoodWallDemolishedRefundEvent>()
            .add_event::<UpgradeWoodWallToStoneWallEvent>()
            .add_event::<WoodWallPlacementRejectedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    place_wood_wall_system,
                    wood_wall_build_tick_system,
                    set_wood_wall_infected_contact_system,
                    set_wood_wall_defended_by_units_system,
                    wood_wall_noise_from_attack_activity_system,
                    damage_wood_wall_system,
                    demolish_wood_wall_system,
                    wood_wall_cleanup_claims_system,
                    wood_wall_checksum_system,
                )
                    .chain(),
            );
    }
}