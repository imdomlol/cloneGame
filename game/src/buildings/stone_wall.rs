// Sources: vault/buildings/stone_wall.md, vault/buildings/wood_wall.md

use std::collections::BTreeMap;

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::SimChecksumState;

const STONE_WALL_HP: I32F32 = I32F32::lit("1000");
const STONE_WALL_DEFENSES_LIFE: I32F32 = I32F32::lit("0");
const STONE_WALL_WATCH_RANGE: I32F32 = I32F32::lit("0");

const STONE_WALL_WOOD_COST_DIRECT: i32 = 0;
const STONE_WALL_STONE_COST_DIRECT: i32 = 3;
const STONE_WALL_GOLD_COST_DIRECT: i32 = 30;
const STONE_WALL_BUILD_TIME_SECONDS_DIRECT: i32 = 30;

const STONE_WALL_BUILD_TIME_SECONDS_FROM_WOOD_WALL: i32 = 26;
const STONE_WALL_WOOD_PATH_EXTRA_COST: i32 = 3;

const WOOD_WALL_HP: I32F32 = I32F32::lit("400");
const WOOD_WALL_WOOD_COST: i32 = 3;
const WOOD_WALL_GOLD_COST: i32 = 10;
const WOOD_WALL_BUILD_TIME_SECONDS: i32 = 15;

const WOOD_WALL_DEMOLISH_REFUND_WOOD: i32 = 2;
const WOOD_WALL_DEMOLISH_REFUND_GOLD: i32 = 5;

const SIM_HZ: i32 = 25;

#[derive(Component, Default)]
pub struct WoodWall;

#[derive(Component, Default)]
pub struct StoneWall;

#[derive(Component, Clone, Copy, Default)]
pub struct BuildingAnchor {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Clone, Copy)]
pub struct BuildingHealth {
    pub hp: I32F32,
    pub defenses_life: I32F32,
}

impl Default for BuildingHealth {
    fn default() -> Self {
        Self {
            hp: STONE_WALL_HP,
            defenses_life: STONE_WALL_DEFENSES_LIFE,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct StoneWallStats {
    pub watch_range: I32F32,
}

impl Default for StoneWallStats {
    fn default() -> Self {
        Self {
            watch_range: STONE_WALL_WATCH_RANGE,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct StoneWallBuildState {
    pub build_ticks_remaining: i32,
    pub upgrading_from_wood_wall: bool,
    pub completed: bool,
}

#[derive(Component, Clone, Copy, Default)]
pub struct StoneWallUnderAttackState {
    pub attacked_this_tick: bool,
    pub near_uncleared_zombie_area_this_tick: bool,
}

#[derive(Component, Clone, Copy)]
pub struct StoneWallActivityNoise {
    pub noise: I32F32,
}

impl Default for StoneWallActivityNoise {
    fn default() -> Self {
        Self {
            noise: I32F32::ZERO,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct StoneWallEconomy {
    pub wood_cost_direct: i32,
    pub stone_cost_direct: i32,
    pub gold_cost_direct: i32,
    pub build_time_seconds_direct: i32,
    pub build_time_seconds_upgrade_path_total: i32,
    pub wood_upgrade_path_extra_total_cost: i32,
}

impl Default for StoneWallEconomy {
    fn default() -> Self {
        Self {
            wood_cost_direct: STONE_WALL_WOOD_COST_DIRECT,
            stone_cost_direct: STONE_WALL_STONE_COST_DIRECT,
            gold_cost_direct: STONE_WALL_GOLD_COST_DIRECT,
            build_time_seconds_direct: STONE_WALL_BUILD_TIME_SECONDS_DIRECT,
            build_time_seconds_upgrade_path_total: STONE_WALL_BUILD_TIME_SECONDS_FROM_WOOD_WALL,
            wood_upgrade_path_extra_total_cost: STONE_WALL_WOOD_PATH_EXTRA_COST,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct WoodWallEconomy {
    pub wood_cost: i32,
    pub gold_cost: i32,
    pub build_time_seconds: i32,
    pub demolish_refund_wood: i32,
    pub demolish_refund_gold: i32,
}

impl Default for WoodWallEconomy {
    fn default() -> Self {
        Self {
            wood_cost: WOOD_WALL_WOOD_COST,
            gold_cost: WOOD_WALL_GOLD_COST,
            build_time_seconds: WOOD_WALL_BUILD_TIME_SECONDS,
            demolish_refund_wood: WOOD_WALL_DEMOLISH_REFUND_WOOD,
            demolish_refund_gold: WOOD_WALL_DEMOLISH_REFUND_GOLD,
        }
    }
}

#[derive(Bundle, Default)]
pub struct WoodWallBundle {
    pub wall: WoodWall,
    pub anchor: BuildingAnchor,
    pub health: BuildingHealth,
    pub economy: WoodWallEconomy,
}

#[derive(Bundle, Default)]
pub struct StoneWallBundle {
    pub wall: StoneWall,
    pub anchor: BuildingAnchor,
    pub health: BuildingHealth,
    pub stats: StoneWallStats,
    pub build_state: StoneWallBuildState,
    pub attack_state: StoneWallUnderAttackState,
    pub activity_noise: StoneWallActivityNoise,
    pub economy: StoneWallEconomy,
}

#[derive(Resource, Default, Clone)]
pub struct WallPlacementClaims {
    pub claims: BTreeMap<Entity, BuildingAnchor>,
}

#[derive(Event, Clone, Copy)]
pub struct PlaceStoneWallEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct UpgradeWoodWallToStoneWallEvent {
    pub wood_wall_entity: Entity,
}

#[derive(Event, Clone, Copy)]
pub struct DamageStoneWallEvent {
    pub wall_entity: Entity,
    pub damage: I32F32,
    pub near_uncleared_zombie_area: bool,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StoneWallPlacementRejectionReason {
    TooManyAdjacentWalls,
    TooManyWallsInLine,
}

#[derive(Event, Clone, Copy)]
pub struct StoneWallPlacementRejectedEvent {
    pub tile_x: i32,
    pub tile_y: i32,
    pub reason: StoneWallPlacementRejectionReason,
}

impl Default for StoneWallBuildState {
    fn default() -> Self {
        Self {
            build_ticks_remaining: build_seconds_to_ticks(STONE_WALL_BUILD_TIME_SECONDS_DIRECT),
            upgrading_from_wood_wall: false,
            completed: false,
        }
    }
}

fn build_seconds_to_ticks(seconds: i32) -> i32 {
    seconds * SIM_HZ
}

fn has_wall_at(claims: &WallPlacementClaims, x: i32, y: i32) -> bool {
    for anchor in claims.claims.values() {
        if anchor.x == x && anchor.y == y {
            return true;
        }
    }
    false
}

fn direct_neighbor_count(claims: &WallPlacementClaims, x: i32, y: i32) -> i32 {
    let mut count = 0;
    if has_wall_at(claims, x - 1, y) {
        count += 1;
    }
    if has_wall_at(claims, x + 1, y) {
        count += 1;
    }
    if has_wall_at(claims, x, y - 1) {
        count += 1;
    }
    if has_wall_at(claims, x, y + 1) {
        count += 1;
    }
    count
}

fn would_create_three_wall_line(claims: &WallPlacementClaims, x: i32, y: i32) -> bool {
    (has_wall_at(claims, x - 1, y) && has_wall_at(claims, x + 1, y))
        || (has_wall_at(claims, x, y - 1) && has_wall_at(claims, x, y + 1))
}

fn validate_placement(
    claims: &WallPlacementClaims,
    x: i32,
    y: i32,
) -> Result<(), StoneWallPlacementRejectionReason> {
    if has_wall_at(claims, x, y) {
        return Err(StoneWallPlacementRejectionReason::TooManyAdjacentWalls);
    }

    if direct_neighbor_count(claims, x, y) > 2 {
        return Err(StoneWallPlacementRejectionReason::TooManyAdjacentWalls);
    }

    if would_create_three_wall_line(claims, x, y) {
        return Err(StoneWallPlacementRejectionReason::TooManyWallsInLine);
    }

    Ok(())
}

fn place_stone_wall_system(
    mut commands: Commands,
    mut events: EventReader<PlaceStoneWallEvent>,
    mut rejected: EventWriter<StoneWallPlacementRejectedEvent>,
    mut claims: ResMut<WallPlacementClaims>,
) {
    for ev in events.read() {
        let validation = validate_placement(&claims, ev.tile_x, ev.tile_y);
        if let Err(reason) = validation {
            rejected.send(StoneWallPlacementRejectedEvent {
                tile_x: ev.tile_x,
                tile_y: ev.tile_y,
                reason,
            });
            continue;
        }

        let entity = commands
            .spawn(StoneWallBundle {
                anchor: BuildingAnchor {
                    x: ev.tile_x,
                    y: ev.tile_y,
                },
                ..Default::default()
            })
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

fn upgrade_wood_wall_to_stone_wall_system(
    mut commands: Commands,
    mut events: EventReader<UpgradeWoodWallToStoneWallEvent>,
    wood_walls: Query<(Entity, &BuildingAnchor), With<WoodWall>>,
    mut claims: ResMut<WallPlacementClaims>,
) {
    for ev in events.read() {
        let Ok((entity, anchor)) = wood_walls.get(ev.wood_wall_entity) else {
            continue;
        };

        let validation = validate_placement(&claims, anchor.x, anchor.y);
        if validation.is_err() {
            continue;
        }

        commands.entity(entity).remove::<WoodWall>();
        commands.entity(entity).insert((
            StoneWall,
            BuildingHealth::default(),
            StoneWallStats::default(),
            StoneWallBuildState {
                build_ticks_remaining: build_seconds_to_ticks(
                    STONE_WALL_BUILD_TIME_SECONDS_FROM_WOOD_WALL,
                ),
                upgrading_from_wood_wall: true,
                completed: false,
            },
            StoneWallUnderAttackState::default(),
            StoneWallActivityNoise::default(),
            StoneWallEconomy::default(),
        ));

        claims.claims.insert(entity, *anchor);
    }
}

fn stone_wall_build_tick_system(mut walls: Query<&mut StoneWallBuildState, With<StoneWall>>) {
    for mut state in &mut walls {
        if state.completed {
            continue;
        }

        if state.build_ticks_remaining > 0 {
            state.build_ticks_remaining -= 1;
        }

        if state.build_ticks_remaining <= 0 {
            state.build_ticks_remaining = 0;
            state.completed = true;
        }
    }
}

fn damage_stone_wall_system(
    mut events: EventReader<DamageStoneWallEvent>,
    mut walls: Query<(&mut BuildingHealth, &mut StoneWallUnderAttackState), With<StoneWall>>,
) {
    for ev in events.read() {
        let Ok((mut health, mut attack_state)) = walls.get_mut(ev.wall_entity) else {
            continue;
        };

        if ev.damage > I32F32::ZERO {
            health.hp -= ev.damage;
            if health.hp < I32F32::ZERO {
                health.hp = I32F32::ZERO;
            }
            attack_state.attacked_this_tick = true;
            if ev.near_uncleared_zombie_area {
                attack_state.near_uncleared_zombie_area_this_tick = true;
            }
        }
    }
}

fn stone_wall_activity_noise_system(
    mut walls: Query<
        (
            &StoneWallBuildState,
            &mut StoneWallUnderAttackState,
            &mut StoneWallActivityNoise,
        ),
        With<StoneWall>,
    >,
) {
    for (build_state, mut attack_state, mut activity_noise) in &mut walls {
        if !build_state.completed {
            activity_noise.noise = I32F32::ZERO;
            attack_state.attacked_this_tick = false;
            attack_state.near_uncleared_zombie_area_this_tick = false;
            continue;
        }

        if attack_state.attacked_this_tick && attack_state.near_uncleared_zombie_area_this_tick {
            activity_noise.noise = I32F32::lit("1");
        } else {
            activity_noise.noise = I32F32::ZERO;
        }

        attack_state.attacked_this_tick = false;
        attack_state.near_uncleared_zombie_area_this_tick = false;
    }
}

fn demolish_wood_wall_system(
    mut commands: Commands,
    mut events: EventReader<DemolishWoodWallEvent>,
    mut refunds: EventWriter<WoodWallDemolishedRefundEvent>,
    walls: Query<&WoodWallEconomy, With<WoodWall>>,
    mut claims: ResMut<WallPlacementClaims>,
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

fn stone_wall_cleanup_claims_system(
    existing_walls: Query<Entity, Or<(With<StoneWall>, With<WoodWall>)>>,
    mut claims: ResMut<WallPlacementClaims>,
) {
    claims
        .claims
        .retain(|entity, _| existing_walls.get(*entity).is_ok());
}

fn stone_wall_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    walls: Query<
        (
            Entity,
            &BuildingAnchor,
            Option<&StoneWall>,
            Option<&WoodWall>,
            &BuildingHealth,
            Option<&StoneWallStats>,
            Option<&StoneWallBuildState>,
            Option<&StoneWallUnderAttackState>,
            Option<&StoneWallActivityNoise>,
            Option<&StoneWallEconomy>,
            Option<&WoodWallEconomy>,
        ),
    >,
    claims: Res<WallPlacementClaims>,
) {
    for (
        entity,
        anchor,
        stone_wall,
        wood_wall,
        health,
        stats,
        build_state,
        attack_state,
        activity_noise,
        stone_eco,
        wood_eco,
    ) in &walls
    {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);

        checksum.accumulate(u64::from(stone_wall.is_some()));
        checksum.accumulate(u64::from(wood_wall.is_some()));

        checksum.accumulate(health.hp.to_bits() as u64);
        checksum.accumulate(health.defenses_life.to_bits() as u64);

        if let Some(s) = stats {
            checksum.accumulate(s.watch_range.to_bits() as u64);
        }

        if let Some(build) = build_state {
            checksum.accumulate(build.build_ticks_remaining as u64);
            checksum.accumulate(u64::from(build.upgrading_from_wood_wall));
            checksum.accumulate(u64::from(build.completed));
        }

        if let Some(attack) = attack_state {
            checksum.accumulate(u64::from(attack.attacked_this_tick));
            checksum.accumulate(u64::from(attack.near_uncleared_zombie_area_this_tick));
        }

        if let Some(noise) = activity_noise {
            checksum.accumulate(noise.noise.to_bits() as u64);
        }

        if let Some(eco) = stone_eco {
            checksum.accumulate(eco.wood_cost_direct as u64);
            checksum.accumulate(eco.stone_cost_direct as u64);
            checksum.accumulate(eco.gold_cost_direct as u64);
            checksum.accumulate(eco.build_time_seconds_direct as u64);
            checksum.accumulate(eco.build_time_seconds_upgrade_path_total as u64);
            checksum.accumulate(eco.wood_upgrade_path_extra_total_cost as u64);
        }

        if let Some(eco) = wood_eco {
            checksum.accumulate(eco.wood_cost as u64);
            checksum.accumulate(eco.gold_cost as u64);
            checksum.accumulate(eco.build_time_seconds as u64);
            checksum.accumulate(eco.demolish_refund_wood as u64);
            checksum.accumulate(eco.demolish_refund_gold as u64);
        }
    }

    checksum.accumulate(claims.claims.len() as u64);
    for (entity, anchor) in &claims.claims {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);
    }
}

pub struct StoneWallPlugin;

impl Plugin for StoneWallPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WallPlacementClaims>()
            .add_event::<PlaceStoneWallEvent>()
            .add_event::<UpgradeWoodWallToStoneWallEvent>()
            .add_event::<DamageStoneWallEvent>()
            .add_event::<DemolishWoodWallEvent>()
            .add_event::<WoodWallDemolishedRefundEvent>()
            .add_event::<StoneWallPlacementRejectedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    place_stone_wall_system,
                    upgrade_wood_wall_to_stone_wall_system,
                    stone_wall_build_tick_system,
                    damage_stone_wall_system,
                    stone_wall_activity_noise_system,
                    demolish_wood_wall_system,
                    stone_wall_cleanup_claims_system,
                    stone_wall_checksum_system,
                )
                    .chain(),
            );
    }
}