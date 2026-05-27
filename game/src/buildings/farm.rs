// Sources: vault/buildings/farm.md, vault/buildings/advanced_farm.md

use std::collections::{BTreeMap, BTreeSet};

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{Health, SimChecksumState};

const FARM_HP: I32F32 = I32F32::lit("300");
const FARM_DEFENSES_LIFE: I32F32 = I32F32::lit("75");
const FARM_WATCH_RANGE: I32F32 = I32F32::lit("6");
const FARM_ENERGY_COST: i32 = 4;
const FARM_WOOD_COST: i32 = 30;
const FARM_STONE_COST: i32 = 0;
const FARM_IRON_COST: i32 = 0;
const FARM_OIL_COST: i32 = 0;
const FARM_GOLD_COST: i32 = 300;
const FARM_BUILD_TIME_SECONDS: i32 = 45;
const FARM_SIZE_TILES: i32 = 2;
const FARM_PLOT_RADIUS_TILES: i32 = 2;
const FARM_MIN_SPACING_TILES: i32 = 4;
const FARM_FOOD_PER_PLOT: I32F32 = I32F32::lit("2");
const FARM_PFOOD_MIN: I32F32 = I32F32::lit("2");
const FARM_PFOOD_MAX: I32F32 = I32F32::lit("64");
const FARM_PCOLONISTS: i32 = 12;
const FARM_INFECTED_ON_FALL: i32 = 12;
const BONUS_NUMERATOR_ONE_BONUS: i32 = 120;
const BONUS_NUMERATOR_BOTH_BONUSES: i32 = 140;
const BONUS_DENOMINATOR: i32 = 100;
const SIM_HZ: i32 = 25;

#[derive(Component, Default)]
pub struct Farm;

#[derive(Component, Clone, Copy, Default)]
pub struct FarmAnchor {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Clone, Copy)]
pub struct FarmDefenseLife {
    pub current: I32F32,
    pub max: I32F32,
}

impl FarmDefenseLife {
    pub fn full(max: I32F32) -> Self {
        Self { current: max, max }
    }
}

impl Default for FarmDefenseLife {
    fn default() -> Self {
        Self::full(FARM_DEFENSES_LIFE)
    }
}

#[derive(Component, Clone, Copy)]
pub struct FarmBuildState {
    pub build_ticks_remaining: i32,
    pub completed: bool,
}

#[derive(Component, Clone, Copy)]
pub struct FarmEconomy {
    pub energy_cost: i32,
    pub wood_cost: i32,
    pub stone_cost: i32,
    pub iron_cost: i32,
    pub oil_cost: i32,
    pub gold_cost: i32,
    pub pcolonists: i32,
}

impl Default for FarmEconomy {
    fn default() -> Self {
        Self {
            energy_cost: FARM_ENERGY_COST,
            wood_cost: FARM_WOOD_COST,
            stone_cost: FARM_STONE_COST,
            iron_cost: FARM_IRON_COST,
            oil_cost: FARM_OIL_COST,
            gold_cost: FARM_GOLD_COST,
            pcolonists: FARM_PCOLONISTS,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct FarmFootprint {
    pub size_tiles: i32,
    pub watch_range: I32F32,
    pub plot_radius_tiles: i32,
    pub min_spacing_tiles: i32,
}

impl Default for FarmFootprint {
    fn default() -> Self {
        Self {
            size_tiles: FARM_SIZE_TILES,
            watch_range: FARM_WATCH_RANGE,
            plot_radius_tiles: FARM_PLOT_RADIUS_TILES,
            min_spacing_tiles: FARM_MIN_SPACING_TILES,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct FarmProduction {
    pub food: I32F32,
    pub min_food: I32F32,
    pub max_food: I32F32,
}

impl Default for FarmProduction {
    fn default() -> Self {
        Self {
            food: I32F32::ZERO,
            min_food: FARM_PFOOD_MIN,
            max_food: FARM_PFOOD_MAX,
        }
    }
}

#[derive(Component, Clone, Default)]
pub struct FarmPlots {
    pub wheat_tiles: BTreeSet<(i32, i32)>,
}

#[derive(Resource, Default, Clone)]
pub struct FarmGrassTiles {
    pub tiles: BTreeSet<(i32, i32)>,
}

#[derive(Resource, Default, Clone)]
pub struct FarmBlockedTiles {
    pub tiles: BTreeSet<(i32, i32)>,
}

#[derive(Resource, Default, Clone)]
pub struct FarmPlacementClaims {
    pub claims: BTreeMap<Entity, FarmAnchor>,
}

#[derive(Resource, Default, Clone, Copy)]
pub struct FarmBonuses {
    pub warehouse_bonus: bool,
    pub mayor_bonus: bool,
}

#[derive(Event, Clone, Copy)]
pub struct SetFarmGrassTileEvent {
    pub tile_x: i32,
    pub tile_y: i32,
    pub is_grass: bool,
}

#[derive(Event, Clone, Copy)]
pub struct SetFarmTileBlockedEvent {
    pub tile_x: i32,
    pub tile_y: i32,
    pub blocked: bool,
}

#[derive(Event, Clone, Copy)]
pub struct SetFarmBonusStateEvent {
    pub warehouse_bonus: bool,
    pub mayor_bonus: bool,
}

#[derive(Event, Clone, Copy)]
pub struct PlaceFarmEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct FarmPlacementRejectedEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct DamageFarmEvent {
    pub farm_entity: Entity,
    pub damage: I32F32,
}

#[derive(Event, Clone, Copy)]
pub struct FarmInfectedOnFallEvent {
    pub farm_entity: Entity,
    pub infected_count: i32,
    pub tile_x: i32,
    pub tile_y: i32,
}

fn seconds_to_ticks(seconds: i32) -> i32 {
    seconds * SIM_HZ
}

fn max_axis_delta(a: FarmAnchor, b: FarmAnchor) -> i32 {
    let dx = (a.x - b.x).abs();
    let dy = (a.y - b.y).abs();
    dx.max(dy)
}

fn is_valid_spacing(anchor: FarmAnchor, claims: &FarmPlacementClaims) -> bool {
    for existing in claims.claims.values() {
        if max_axis_delta(anchor, *existing) < FARM_MIN_SPACING_TILES {
            return false;
        }
    }
    true
}

fn footprint_tiles(anchor: FarmAnchor) -> impl Iterator<Item = (i32, i32)> {
    let min_x = anchor.x;
    let min_y = anchor.y;
    let max_x = anchor.x + FARM_SIZE_TILES - 1;
    let max_y = anchor.y + FARM_SIZE_TILES - 1;
    (min_x..=max_x).flat_map(move |x| (min_y..=max_y).map(move |y| (x, y)))
}

fn footprint_is_not_blocked(anchor: FarmAnchor, blocked: &FarmBlockedTiles) -> bool {
    for tile in footprint_tiles(anchor) {
        if blocked.tiles.contains(&tile) {
            return false;
        }
    }
    true
}

fn apply_farm_grass_tile_events_system(
    mut events: EventReader<SetFarmGrassTileEvent>,
    mut grass: ResMut<FarmGrassTiles>,
) {
    for ev in events.read() {
        let tile = (ev.tile_x, ev.tile_y);
        if ev.is_grass {
            grass.tiles.insert(tile);
        } else {
            grass.tiles.remove(&tile);
        }
    }
}

fn apply_farm_tile_block_events_system(
    mut events: EventReader<SetFarmTileBlockedEvent>,
    mut blocked: ResMut<FarmBlockedTiles>,
) {
    for ev in events.read() {
        let tile = (ev.tile_x, ev.tile_y);
        if ev.blocked {
            blocked.tiles.insert(tile);
        } else {
            blocked.tiles.remove(&tile);
        }
    }
}

fn apply_farm_bonus_events_system(
    mut events: EventReader<SetFarmBonusStateEvent>,
    mut bonuses: ResMut<FarmBonuses>,
) {
    for ev in events.read() {
        bonuses.warehouse_bonus = ev.warehouse_bonus;
        bonuses.mayor_bonus = ev.mayor_bonus;
    }
}

fn place_farm_system(
    mut commands: Commands,
    mut events: EventReader<PlaceFarmEvent>,
    mut rejected: EventWriter<FarmPlacementRejectedEvent>,
    mut claims: ResMut<FarmPlacementClaims>,
    blocked: Res<FarmBlockedTiles>,
) {
    for ev in events.read() {
        let anchor = FarmAnchor {
            x: ev.tile_x,
            y: ev.tile_y,
        };

        if !is_valid_spacing(anchor, &claims) || !footprint_is_not_blocked(anchor, &blocked) {
            rejected.send(FarmPlacementRejectedEvent {
                tile_x: ev.tile_x,
                tile_y: ev.tile_y,
            });
            continue;
        }

        let entity = commands
            .spawn((
                Farm,
                anchor,
                Health::full(FARM_HP),
                FarmDefenseLife::default(),
                FarmBuildState {
                    build_ticks_remaining: seconds_to_ticks(FARM_BUILD_TIME_SECONDS),
                    completed: false,
                },
                FarmEconomy::default(),
                FarmFootprint::default(),
                FarmProduction::default(),
                FarmPlots::default(),
            ))
            .id();

        claims.claims.insert(entity, anchor);
    }
}

fn farm_build_tick_system(mut farms: Query<&mut FarmBuildState, With<Farm>>) {
    for mut build in &mut farms {
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

fn farm_wheat_plot_and_output_system(
    mut farms: Query<(&FarmAnchor, &FarmBuildState, &mut FarmPlots, &mut FarmProduction), With<Farm>>,
    grass: Res<FarmGrassTiles>,
    blocked: Res<FarmBlockedTiles>,
    bonuses: Res<FarmBonuses>,
) {
    let bonus_numerator = if bonuses.warehouse_bonus && bonuses.mayor_bonus {
        BONUS_NUMERATOR_BOTH_BONUSES
    } else if bonuses.warehouse_bonus || bonuses.mayor_bonus {
        BONUS_NUMERATOR_ONE_BONUS
    } else {
        BONUS_DENOMINATOR
    };

    for (anchor, build, mut plots, mut production) in &mut farms {
        if !build.completed {
            plots.wheat_tiles.clear();
            production.food = I32F32::ZERO;
            continue;
        }

        let mut assigned = BTreeSet::new();
        let min_x = anchor.x - FARM_PLOT_RADIUS_TILES;
        let max_x = anchor.x + FARM_SIZE_TILES - 1 + FARM_PLOT_RADIUS_TILES;
        let min_y = anchor.y - FARM_PLOT_RADIUS_TILES;
        let max_y = anchor.y + FARM_SIZE_TILES - 1 + FARM_PLOT_RADIUS_TILES;

        for x in min_x..=max_x {
            for y in min_y..=max_y {
                let tile = (x, y);
                if !grass.tiles.contains(&tile) {
                    continue;
                }

                let adjacent_to_footprint = x >= anchor.x - 1
                    && x <= anchor.x + FARM_SIZE_TILES
                    && y >= anchor.y - 1
                    && y <= anchor.y + FARM_SIZE_TILES;

                if !adjacent_to_footprint && blocked.tiles.contains(&tile) {
                    continue;
                }

                assigned.insert(tile);
            }
        }

        plots.wheat_tiles = assigned;

        let raw_food = FARM_FOOD_PER_PLOT * I32F32::from_num(plots.wheat_tiles.len() as i32);
        let with_bonuses =
            raw_food * I32F32::from_num(bonus_numerator) / I32F32::from_num(BONUS_DENOMINATOR);
        production.food = with_bonuses.min(production.max_food).max(production.min_food);
    }
}

fn damage_farm_system(
    mut commands: Commands,
    mut events: EventReader<DamageFarmEvent>,
    mut infected_events: EventWriter<FarmInfectedOnFallEvent>,
    mut farms: Query<(Entity, &FarmAnchor, &mut Health, &mut FarmDefenseLife), With<Farm>>,
    mut claims: ResMut<FarmPlacementClaims>,
) {
    for ev in events.read() {
        let Ok((entity, anchor, mut health, mut defense)) = farms.get_mut(ev.farm_entity) else {
            continue;
        };

        if ev.damage <= I32F32::ZERO {
            continue;
        }

        let mut remaining = ev.damage;
        if defense.current > I32F32::ZERO {
            let absorbed = if defense.current > remaining {
                remaining
            } else {
                defense.current
            };
            defense.current -= absorbed;
            remaining -= absorbed;
        }

        if remaining > I32F32::ZERO {
            if health.current > remaining {
                health.current -= remaining;
            } else {
                health.current = I32F32::ZERO;
            }
        }

        if health.current == I32F32::ZERO {
            infected_events.send(FarmInfectedOnFallEvent {
                farm_entity: entity,
                infected_count: FARM_INFECTED_ON_FALL,
                tile_x: anchor.x,
                tile_y: anchor.y,
            });
            claims.claims.remove(&entity);
            commands.entity(entity).despawn();
        }
    }
}

fn farm_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    farms: Query<
        (
            Entity,
            &FarmAnchor,
            &Health,
            &FarmDefenseLife,
            &FarmBuildState,
            &FarmEconomy,
            &FarmFootprint,
            &FarmProduction,
            &FarmPlots,
        ),
        With<Farm>,
    >,
    claims: Res<FarmPlacementClaims>,
    bonuses: Res<FarmBonuses>,
) {
    for (entity, anchor, health, defense, build, economy, footprint, production, plots) in &farms {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);

        checksum.accumulate(health.current.to_bits() as u64);
        checksum.accumulate(health.max.to_bits() as u64);
        checksum.accumulate(defense.current.to_bits() as u64);
        checksum.accumulate(defense.max.to_bits() as u64);

        checksum.accumulate(build.build_ticks_remaining as u64);
        checksum.accumulate(u64::from(build.completed));

        checksum.accumulate(economy.energy_cost as u64);
        checksum.accumulate(economy.wood_cost as u64);
        checksum.accumulate(economy.stone_cost as u64);
        checksum.accumulate(economy.iron_cost as u64);
        checksum.accumulate(economy.oil_cost as u64);
        checksum.accumulate(economy.gold_cost as u64);
        checksum.accumulate(economy.pcolonists as u64);

        checksum.accumulate(footprint.size_tiles as u64);
        checksum.accumulate(footprint.watch_range.to_bits() as u64);
        checksum.accumulate(footprint.plot_radius_tiles as u64);
        checksum.accumulate(footprint.min_spacing_tiles as u64);

        checksum.accumulate(production.food.to_bits() as u64);
        checksum.accumulate(production.min_food.to_bits() as u64);
        checksum.accumulate(production.max_food.to_bits() as u64);

        checksum.accumulate(plots.wheat_tiles.len() as u64);
        for (x, y) in &plots.wheat_tiles {
            checksum.accumulate(*x as u64);
            checksum.accumulate(*y as u64);
        }
    }

    checksum.accumulate(claims.claims.len() as u64);
    for (entity, anchor) in &claims.claims {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);
    }

    checksum.accumulate(u64::from(bonuses.warehouse_bonus));
    checksum.accumulate(u64::from(bonuses.mayor_bonus));
}

pub struct FarmPlugin;

impl Plugin for FarmPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FarmGrassTiles>()
            .init_resource::<FarmBlockedTiles>()
            .init_resource::<FarmPlacementClaims>()
            .init_resource::<FarmBonuses>()
            .add_event::<SetFarmGrassTileEvent>()
            .add_event::<SetFarmTileBlockedEvent>()
            .add_event::<SetFarmBonusStateEvent>()
            .add_event::<PlaceFarmEvent>()
            .add_event::<FarmPlacementRejectedEvent>()
            .add_event::<DamageFarmEvent>()
            .add_event::<FarmInfectedOnFallEvent>()
            .add_systems(
                FixedUpdate,
                (
                    apply_farm_grass_tile_events_system,
                    apply_farm_tile_block_events_system,
                    apply_farm_bonus_events_system,
                    place_farm_system,
                    farm_build_tick_system,
                    farm_wheat_plot_and_output_system,
                    damage_farm_system,
                    farm_checksum_system,
                )
                    .chain(),
            );
    }
}