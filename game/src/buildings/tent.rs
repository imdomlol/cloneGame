// Sources: vault/buildings/tent.md, vault/buildings/cottage.md

use std::collections::{BTreeMap, BTreeSet};

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::SimChecksumState;

const TENT_HP: I32F32 = I32F32::lit("60");
const TENT_DEFENSES_LIFE: I32F32 = I32F32::lit("15");
const TENT_WATCH_RANGE: I32F32 = I32F32::lit("6");

const TENT_ENERGY_COST: i32 = 1;
const TENT_GOLD_COST: i32 = 30;
const TENT_FOOD_COST: i32 = 4;
const TENT_WORKERS_USED: i32 = 2;
const TENT_PCOLONISTS: i32 = 4;

const TENT_BUILD_TIME_SECONDS: i32 = 21;
const TENT_GOLD_INCOME_PER_INTERVAL: i32 = 8;
const TENT_GOLD_INTERVAL_HOURS: i32 = 8;

const TENT_SIZE_TILES: i32 = 2;
const TENT_PLACEMENT_EMPTY_TILES_MIN: i32 = 3;
const TENT_MAX_BUILDINGS_ADJACENT_INFOBOX: i32 = 2;
const TENT_MAX_DIRECT_BORDER_HOUSES_TEXT: i32 = 3;

const SIM_HZ: i32 = 25;
const SECONDS_PER_HOUR: i32 = 3600;

#[derive(Component, Default)]
pub struct Tent;

#[derive(Component, Default)]
pub struct Cottage;

#[derive(Component, Clone, Copy)]
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
            hp: TENT_HP,
            defenses_life: TENT_DEFENSES_LIFE,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct TentBuildState {
    pub build_ticks_remaining: i32,
    pub completed: bool,
}

impl Default for TentBuildState {
    fn default() -> Self {
        Self {
            build_ticks_remaining: build_seconds_to_ticks(TENT_BUILD_TIME_SECONDS),
            completed: false,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct TentEconomy {
    pub energy_cost: i32,
    pub gold_cost: i32,
    pub food_cost: i32,
    pub workers_used: i32,
    pub pcolonists: i32,
    pub gold_income: i32,
    pub gold_interval_ticks: i32,
}

impl Default for TentEconomy {
    fn default() -> Self {
        Self {
            energy_cost: TENT_ENERGY_COST,
            gold_cost: TENT_GOLD_COST,
            food_cost: TENT_FOOD_COST,
            workers_used: TENT_WORKERS_USED,
            pcolonists: TENT_PCOLONISTS,
            gold_income: TENT_GOLD_INCOME_PER_INTERVAL,
            gold_interval_ticks: hours_to_ticks(TENT_GOLD_INTERVAL_HOURS),
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct TentIncomeState {
    pub ticks_until_next_gold: i32,
    pub stored_gold: i32,
}

impl Default for TentIncomeState {
    fn default() -> Self {
        Self {
            ticks_until_next_gold: hours_to_ticks(TENT_GOLD_INTERVAL_HOURS),
            stored_gold: 0,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct TentFootprint {
    pub size_tiles: i32,
    pub watch_range: I32F32,
    pub placement_empty_tiles_min: i32,
    pub max_buildings_adjacent_infobox: i32,
    pub max_direct_border_houses_text: i32,
}

impl Default for TentFootprint {
    fn default() -> Self {
        Self {
            size_tiles: TENT_SIZE_TILES,
            watch_range: TENT_WATCH_RANGE,
            placement_empty_tiles_min: TENT_PLACEMENT_EMPTY_TILES_MIN,
            max_buildings_adjacent_infobox: TENT_MAX_BUILDINGS_ADJACENT_INFOBOX,
            max_direct_border_houses_text: TENT_MAX_DIRECT_BORDER_HOUSES_TEXT,
        }
    }
}

#[derive(Resource, Default, Clone)]
pub struct TileOccupancy {
    pub blocked_tiles: BTreeSet<(i32, i32)>,
}

#[derive(Resource, Default, Clone)]
pub struct TentPlacementClaims {
    pub claims: BTreeMap<Entity, BuildingAnchor>,
}

#[derive(Event, Clone, Copy)]
pub struct SetTileBlockedEvent {
    pub tile_x: i32,
    pub tile_y: i32,
    pub blocked: bool,
}

#[derive(Event, Clone, Copy)]
pub struct PlaceTentEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct TentPlacementRejectedEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct UpgradeTentToCottageEvent {
    pub tent_entity: Entity,
}

fn build_seconds_to_ticks(seconds: i32) -> i32 {
    seconds * SIM_HZ
}

fn hours_to_ticks(hours: i32) -> i32 {
    hours * SECONDS_PER_HOUR * SIM_HZ
}

fn footprint_tiles(anchor: BuildingAnchor) -> impl Iterator<Item = (i32, i32)> {
    let min_x = anchor.x;
    let min_y = anchor.y;
    let max_x = anchor.x + TENT_SIZE_TILES - 1;
    let max_y = anchor.y + TENT_SIZE_TILES - 1;
    (min_x..=max_x).flat_map(move |x| (min_y..=max_y).map(move |y| (x, y)))
}

fn max_axis_delta(a: BuildingAnchor, b: BuildingAnchor) -> i32 {
    (a.x - b.x).abs().max((a.y - b.y).abs())
}

fn is_valid_spacing(anchor: BuildingAnchor, claims: &TentPlacementClaims) -> bool {
    for existing in claims.claims.values() {
        if max_axis_delta(anchor, *existing) < TENT_PLACEMENT_EMPTY_TILES_MIN {
            return false;
        }
    }
    true
}

fn footprint_is_unblocked(anchor: BuildingAnchor, occupancy: &TileOccupancy) -> bool {
    for tile in footprint_tiles(anchor) {
        if occupancy.blocked_tiles.contains(&tile) {
            return false;
        }
    }
    true
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

fn place_tent_system(
    mut commands: Commands,
    mut events: EventReader<PlaceTentEvent>,
    mut rejected: EventWriter<TentPlacementRejectedEvent>,
    mut claims: ResMut<TentPlacementClaims>,
    occupancy: Res<TileOccupancy>,
) {
    for ev in events.read() {
        let anchor = BuildingAnchor {
            x: ev.tile_x,
            y: ev.tile_y,
        };

        if !is_valid_spacing(anchor, &claims) || !footprint_is_unblocked(anchor, &occupancy) {
            rejected.send(TentPlacementRejectedEvent {
                tile_x: ev.tile_x,
                tile_y: ev.tile_y,
            });
            continue;
        }

        let entity = commands
            .spawn((
                Tent,
                anchor,
                BuildingHealth::default(),
                TentBuildState::default(),
                TentEconomy::default(),
                TentIncomeState::default(),
                TentFootprint::default(),
            ))
            .id();

        claims.claims.insert(entity, anchor);
    }
}

fn upgrade_tent_to_cottage_system(
    mut commands: Commands,
    mut events: EventReader<UpgradeTentToCottageEvent>,
    tents: Query<(Entity, &BuildingAnchor), With<Tent>>,
    mut claims: ResMut<TentPlacementClaims>,
) {
    for ev in events.read() {
        let Ok((tent_entity, _anchor)) = tents.get(ev.tent_entity) else {
            continue;
        };

        commands.entity(tent_entity).remove::<Tent>();
        commands.entity(tent_entity).insert(Cottage);
        claims.claims.remove(&tent_entity);
    }
}

fn tent_build_tick_system(mut tents: Query<&mut TentBuildState, With<Tent>>) {
    for mut state in &mut tents {
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

fn tent_gold_income_tick_system(
    mut tents: Query<(&TentBuildState, &TentEconomy, &mut TentIncomeState), With<Tent>>,
) {
    for (build, economy, mut income) in &mut tents {
        if !build.completed {
            continue;
        }

        if income.ticks_until_next_gold > 0 {
            income.ticks_until_next_gold -= 1;
        }

        if income.ticks_until_next_gold <= 0 {
            income.ticks_until_next_gold = economy.gold_interval_ticks;
            income.stored_gold += economy.gold_income;
        }
    }
}

fn tent_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    tents: Query<
        (
            Entity,
            &BuildingAnchor,
            &BuildingHealth,
            &TentBuildState,
            &TentEconomy,
            &TentIncomeState,
            &TentFootprint,
        ),
        With<Tent>,
    >,
    claims: Res<TentPlacementClaims>,
) {
    for (entity, anchor, hp, build, economy, income, footprint) in &tents {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);

        checksum.accumulate(hp.hp.to_bits() as u64);
        checksum.accumulate(hp.defenses_life.to_bits() as u64);

        checksum.accumulate(build.build_ticks_remaining as u64);
        checksum.accumulate(u64::from(build.completed));

        checksum.accumulate(economy.energy_cost as u64);
        checksum.accumulate(economy.gold_cost as u64);
        checksum.accumulate(economy.food_cost as u64);
        checksum.accumulate(economy.workers_used as u64);
        checksum.accumulate(economy.pcolonists as u64);
        checksum.accumulate(economy.gold_income as u64);
        checksum.accumulate(economy.gold_interval_ticks as u64);

        checksum.accumulate(income.ticks_until_next_gold as u64);
        checksum.accumulate(income.stored_gold as u64);

        checksum.accumulate(footprint.size_tiles as u64);
        checksum.accumulate(footprint.watch_range.to_bits() as u64);
        checksum.accumulate(footprint.placement_empty_tiles_min as u64);
        checksum.accumulate(footprint.max_buildings_adjacent_infobox as u64);
        checksum.accumulate(footprint.max_direct_border_houses_text as u64);
    }

    checksum.accumulate(claims.claims.len() as u64);
    for (entity, anchor) in &claims.claims {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);
    }
}

pub struct TentPlugin;

impl Plugin for TentPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileOccupancy>()
            .init_resource::<TentPlacementClaims>()
            .add_event::<SetTileBlockedEvent>()
            .add_event::<PlaceTentEvent>()
            .add_event::<TentPlacementRejectedEvent>()
            .add_event::<UpgradeTentToCottageEvent>()
            .add_systems(
                FixedUpdate,
                (
                    apply_tile_block_events_system,
                    place_tent_system,
                    upgrade_tent_to_cottage_system,
                    tent_build_tick_system,
                    tent_gold_income_tick_system,
                    tent_checksum_system,
                )
                    .chain(),
            );
    }
}