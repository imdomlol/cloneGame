// Sources: vault/buildings/bank.md

use std::collections::{BTreeMap, BTreeSet};

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{Health, SimChecksumState};

const BANK_HP: I32F32 = I32F32::lit("1000");
const BANK_DEFENSES_LIFE: I32F32 = I32F32::lit("250");
const BANK_WATCH_RANGE: I32F32 = I32F32::lit("7");
const BANK_ENERGY_COST: i32 = 20;
const BANK_WOOD_COST: i32 = 50;
const BANK_STONE_COST: i32 = 50;
const BANK_IRON_COST: i32 = 0;
const BANK_OIL_COST: i32 = 0;
const BANK_GOLD_COST: i32 = 1000;
const BANK_BUILD_TIME_SECONDS: i32 = 60;
const BANK_SIZE_TILES: i32 = 3;
const BANK_WORKERS: i32 = 12;
const BANK_GOLD_MAINTENANCE: i32 = 0;
const BANK_BUFF_RADIUS_TILES: i32 = 12;
const BANK_BUFF_PERCENT: I32F32 = I32F32::lit("0.30");
const BANK_AOE_WIDTH_TILES: i32 = 27;
const BANK_AOE_HEIGHT_TILES: i32 = 27;
const BANK_POPULATION_UNLOCK_1: i32 = 400;
const BANK_POPULATION_UNLOCK_2: i32 = 800;
const BANK_POPULATION_UNLOCK_3: i32 = 1200;
const BANK_MAX_COUNT: usize = 3;
const SIM_HZ: i32 = 25;

#[derive(Component, Default)]
pub struct Bank;

#[derive(Component, Clone, Copy, Default)]
pub struct BuildingAnchor {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Clone, Copy)]
pub struct BankCoreStats {
    pub defenses_life: I32F32,
    pub watch_range: I32F32,
    pub health: Health,
}

impl Default for BankCoreStats {
    fn default() -> Self {
        Self {
            defenses_life: BANK_DEFENSES_LIFE,
            watch_range: BANK_WATCH_RANGE,
            health: Health::full(BANK_HP),
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct BankBuildState {
    pub build_ticks_remaining: i32,
    pub completed: bool,
}

#[derive(Component, Clone, Copy)]
pub struct BankEconomy {
    pub energy_cost: i32,
    pub wood_cost: i32,
    pub stone_cost: i32,
    pub iron_cost: i32,
    pub oil_cost: i32,
    pub gold_cost: i32,
    pub workers: i32,
    pub gold_maintenance: i32,
}

impl Default for BankEconomy {
    fn default() -> Self {
        Self {
            energy_cost: BANK_ENERGY_COST,
            wood_cost: BANK_WOOD_COST,
            stone_cost: BANK_STONE_COST,
            iron_cost: BANK_IRON_COST,
            oil_cost: BANK_OIL_COST,
            gold_cost: BANK_GOLD_COST,
            workers: BANK_WORKERS,
            gold_maintenance: BANK_GOLD_MAINTENANCE,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct BankFootprint {
    pub size_tiles: i32,
    pub watch_range: I32F32,
    pub aoe_width_tiles: i32,
    pub aoe_height_tiles: i32,
}

impl Default for BankFootprint {
    fn default() -> Self {
        Self {
            size_tiles: BANK_SIZE_TILES,
            watch_range: BANK_WATCH_RANGE,
            aoe_width_tiles: BANK_AOE_WIDTH_TILES,
            aoe_height_tiles: BANK_AOE_HEIGHT_TILES,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct BankAura {
    pub radius_tiles: i32,
    pub bonus_percent: I32F32,
}

impl Default for BankAura {
    fn default() -> Self {
        Self {
            radius_tiles: BANK_BUFF_RADIUS_TILES,
            bonus_percent: BANK_BUFF_PERCENT,
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct ColonistBuilding {
    pub base_gold_output: I32F32,
}

#[derive(Component, Clone, Copy)]
pub struct ColonistFootprint {
    pub width_tiles: i32,
    pub height_tiles: i32,
}

impl Default for ColonistFootprint {
    fn default() -> Self {
        Self {
            width_tiles: 1,
            height_tiles: 1,
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct ColonistGoldBonus {
    pub bank_entity: Option<Entity>,
    pub bonus_percent: I32F32,
    pub effective_gold_output: I32F32,
}

#[derive(Resource, Default, Clone, Copy)]
pub struct ColonyPopulation {
    pub total: i32,
}

#[derive(Resource, Default, Clone)]
pub struct TileOccupancy {
    pub blocked_tiles: BTreeSet<(i32, i32)>,
}

#[derive(Resource, Default, Clone)]
pub struct BankPlacementClaims {
    pub claims: BTreeMap<Entity, BuildingAnchor>,
}

#[derive(Event, Clone, Copy)]
pub struct SetTileBlockedEvent {
    pub tile_x: i32,
    pub tile_y: i32,
    pub blocked: bool,
}

#[derive(Event, Clone, Copy)]
pub struct SetColonyPopulationEvent {
    pub total: i32,
}

#[derive(Event, Clone, Copy)]
pub struct PlaceBankEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct BankPlacementRejectedEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

fn build_seconds_to_ticks(seconds: i32) -> i32 {
    seconds * SIM_HZ
}

fn footprint_tiles(anchor: BuildingAnchor) -> impl Iterator<Item = (i32, i32)> {
    let min_x = anchor.x;
    let min_y = anchor.y;
    let max_x = anchor.x + BANK_SIZE_TILES - 1;
    let max_y = anchor.y + BANK_SIZE_TILES - 1;
    (min_x..=max_x).flat_map(move |x| (min_y..=max_y).map(move |y| (x, y)))
}

fn bank_center(anchor: BuildingAnchor) -> (i32, i32) {
    (anchor.x + 1, anchor.y + 1)
}

fn max_axis_delta(a: (i32, i32), b: (i32, i32)) -> i32 {
    (a.0 - b.0).abs().max((a.1 - b.1).abs())
}

fn colonist_fully_inside_bank_aura(
    colonist_anchor: BuildingAnchor,
    colonist_footprint: ColonistFootprint,
    bank_anchor: BuildingAnchor,
    radius_tiles: i32,
) -> bool {
    let (cx, cy) = bank_center(bank_anchor);
    let min_x = colonist_anchor.x;
    let min_y = colonist_anchor.y;
    let max_x = colonist_anchor.x + colonist_footprint.width_tiles - 1;
    let max_y = colonist_anchor.y + colonist_footprint.height_tiles - 1;

    let corners = [(min_x, min_y), (min_x, max_y), (max_x, min_y), (max_x, max_y)];

    for corner in corners {
        if max_axis_delta(corner, (cx, cy)) > radius_tiles {
            return false;
        }
    }

    true
}

fn max_banks_for_population(population: i32) -> usize {
    let mut max_count = 0usize;

    if population >= BANK_POPULATION_UNLOCK_1 {
        max_count += 1;
    }
    if population >= BANK_POPULATION_UNLOCK_2 {
        max_count += 1;
    }
    if population >= BANK_POPULATION_UNLOCK_3 {
        max_count += 1;
    }

    max_count.min(BANK_MAX_COUNT)
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

fn apply_population_events_system(
    mut events: EventReader<SetColonyPopulationEvent>,
    mut population: ResMut<ColonyPopulation>,
) {
    for ev in events.read() {
        population.total = ev.total;
    }
}

fn place_bank_system(
    mut commands: Commands,
    mut events: EventReader<PlaceBankEvent>,
    mut rejected: EventWriter<BankPlacementRejectedEvent>,
    mut claims: ResMut<BankPlacementClaims>,
    occupancy: Res<TileOccupancy>,
    population: Res<ColonyPopulation>,
) {
    let allowed = max_banks_for_population(population.total);

    for ev in events.read() {
        let anchor = BuildingAnchor {
            x: ev.tile_x,
            y: ev.tile_y,
        };

        if claims.claims.len() >= allowed || !footprint_is_unblocked(anchor, &occupancy) {
            rejected.send(BankPlacementRejectedEvent {
                tile_x: ev.tile_x,
                tile_y: ev.tile_y,
            });
            continue;
        }

        let entity = commands
            .spawn((
                Bank,
                anchor,
                BankCoreStats::default(),
                BankBuildState {
                    build_ticks_remaining: build_seconds_to_ticks(BANK_BUILD_TIME_SECONDS),
                    completed: false,
                },
                BankEconomy::default(),
                BankFootprint::default(),
                BankAura::default(),
            ))
            .id();

        claims.claims.insert(entity, anchor);
    }
}

fn bank_build_tick_system(mut banks: Query<&mut BankBuildState, With<Bank>>) {
    for mut state in &mut banks {
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

fn bank_colonist_bonus_system(
    banks: Query<(Entity, &BuildingAnchor, &BankBuildState, &BankAura), With<Bank>>,
    mut colonists: Query<
        (&BuildingAnchor, &ColonistFootprint, &ColonistBuilding, &mut ColonistGoldBonus),
        With<ColonistBuilding>,
    >,
) {
    for (colonist_anchor, colonist_footprint, colonist, mut bonus) in &mut colonists {
        let mut selected_bank: Option<Entity> = None;
        let mut selected_percent = I32F32::ZERO;

        for (bank_entity, bank_anchor, bank_state, aura) in &banks {
            if !bank_state.completed {
                continue;
            }

            if !colonist_fully_inside_bank_aura(
                *colonist_anchor,
                *colonist_footprint,
                *bank_anchor,
                aura.radius_tiles,
            ) {
                continue;
            }

            match selected_bank {
                None => {
                    selected_bank = Some(bank_entity);
                    selected_percent = aura.bonus_percent;
                }
                Some(existing) => {
                    if bank_entity.to_bits() < existing.to_bits() {
                        selected_bank = Some(bank_entity);
                        selected_percent = aura.bonus_percent;
                    }
                }
            }
        }

        bonus.bank_entity = selected_bank;
        bonus.bonus_percent = selected_percent;
        bonus.effective_gold_output = colonist.base_gold_output * (I32F32::ONE + selected_percent);
    }
}

fn bank_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    banks: Query<
        (
            Entity,
            &BuildingAnchor,
            &BankCoreStats,
            &BankBuildState,
            &BankEconomy,
            &BankFootprint,
            &BankAura,
        ),
        With<Bank>,
    >,
    claims: Res<BankPlacementClaims>,
    colonists: Query<
        (&BuildingAnchor, &ColonistFootprint, &ColonistBuilding, &ColonistGoldBonus),
        With<ColonistBuilding>,
    >,
    population: Res<ColonyPopulation>,
) {
    for (entity, anchor, core, build, eco, footprint, aura) in &banks {
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
        checksum.accumulate(eco.wood_cost as u64);
        checksum.accumulate(eco.stone_cost as u64);
        checksum.accumulate(eco.iron_cost as u64);
        checksum.accumulate(eco.oil_cost as u64);
        checksum.accumulate(eco.gold_cost as u64);
        checksum.accumulate(eco.workers as u64);
        checksum.accumulate(eco.gold_maintenance as u64);

        checksum.accumulate(footprint.size_tiles as u64);
        checksum.accumulate(footprint.watch_range.to_bits() as u64);
        checksum.accumulate(footprint.aoe_width_tiles as u64);
        checksum.accumulate(footprint.aoe_height_tiles as u64);

        checksum.accumulate(aura.radius_tiles as u64);
        checksum.accumulate(aura.bonus_percent.to_bits() as u64);
    }

    checksum.accumulate(claims.claims.len() as u64);
    for (entity, anchor) in &claims.claims {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);
    }

    checksum.accumulate(population.total as u64);

    for (anchor, footprint, colonist, bonus) in &colonists {
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);
        checksum.accumulate(footprint.width_tiles as u64);
        checksum.accumulate(footprint.height_tiles as u64);
        checksum.accumulate(colonist.base_gold_output.to_bits() as u64);
        checksum.accumulate(bonus.bonus_percent.to_bits() as u64);
        checksum.accumulate(bonus.effective_gold_output.to_bits() as u64);
        checksum.accumulate(bonus.bank_entity.map(Entity::to_bits).unwrap_or(0));
    }
}

pub struct BankPlugin;

impl Plugin for BankPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ColonyPopulation>()
            .init_resource::<TileOccupancy>()
            .init_resource::<BankPlacementClaims>()
            .add_event::<SetTileBlockedEvent>()
            .add_event::<SetColonyPopulationEvent>()
            .add_event::<PlaceBankEvent>()
            .add_event::<BankPlacementRejectedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    apply_tile_block_events_system,
                    apply_population_events_system,
                    place_bank_system,
                    bank_build_tick_system,
                    bank_colonist_bonus_system,
                    bank_checksum_system,
                )
                    .chain(),
            );
    }
}