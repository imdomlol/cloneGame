// Sources: vault/buildings/oil_platform.md

use std::collections::{BTreeMap, BTreeSet};

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{Health, SimChecksumState};

const OIL_PLATFORM_HP: I32F32 = I32F32::lit("500");
const OIL_PLATFORM_DEFENSES_LIFE: I32F32 = I32F32::lit("125");
const OIL_PLATFORM_WATCH_RANGE: I32F32 = I32F32::lit("7");

const OIL_PLATFORM_ENERGY_COST: i32 = 30;
const OIL_PLATFORM_WOOD_COST: i32 = 0;
const OIL_PLATFORM_STONE_COST: i32 = 20;
const OIL_PLATFORM_IRON_COST: i32 = 20;
const OIL_PLATFORM_OIL_COST: i32 = 0;
const OIL_PLATFORM_GOLD_COST: i32 = 1200;
const OIL_PLATFORM_WORKERS: i32 = 10;

const OIL_PLATFORM_PFOOD: i32 = 0;
const OIL_PLATFORM_PWOOD: i32 = 0;
const OIL_PLATFORM_PSTONE: i32 = 0;
const OIL_PLATFORM_PIRON: i32 = 0;
const OIL_PLATFORM_POIL: i32 = 10;
const OIL_PLATFORM_PGOLD: i32 = 0;
const OIL_PLATFORM_PENERGY: i32 = 0;
const OIL_PLATFORM_PCOLONISTS: i32 = 0;

const OIL_PLATFORM_BUILD_TIME_SECONDS: i32 = 60;
const OIL_PLATFORM_SIZE_TILES: i32 = 2;
const OIL_PLATFORM_PRODUCTION_CYCLE_HOURS: i32 = 8;
const SECONDS_PER_HOUR: i32 = 3600;
const SIM_HZ: i32 = 25;

#[derive(Component, Default)]
pub struct OilPlatform;

#[derive(Component, Clone, Copy, Default)]
pub struct BuildingAnchor {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Clone, Copy)]
pub struct OilPlatformCore {
    pub defenses_life: I32F32,
    pub watch_range: I32F32,
    pub health: Health,
}

impl Default for OilPlatformCore {
    fn default() -> Self {
        Self {
            defenses_life: OIL_PLATFORM_DEFENSES_LIFE,
            watch_range: OIL_PLATFORM_WATCH_RANGE,
            health: Health::full(OIL_PLATFORM_HP),
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct OilPlatformEconomy {
    pub energy_cost: i32,
    pub wood_cost: i32,
    pub stone_cost: i32,
    pub iron_cost: i32,
    pub oil_cost: i32,
    pub gold_cost: i32,
    pub workers: i32,
    pub pfood: i32,
    pub pwood: i32,
    pub pstone: i32,
    pub piron: i32,
    pub poil: i32,
    pub pgold: i32,
    pub penergy: i32,
    pub pcolonists: i32,
}

impl Default for OilPlatformEconomy {
    fn default() -> Self {
        Self {
            energy_cost: OIL_PLATFORM_ENERGY_COST,
            wood_cost: OIL_PLATFORM_WOOD_COST,
            stone_cost: OIL_PLATFORM_STONE_COST,
            iron_cost: OIL_PLATFORM_IRON_COST,
            oil_cost: OIL_PLATFORM_OIL_COST,
            gold_cost: OIL_PLATFORM_GOLD_COST,
            workers: OIL_PLATFORM_WORKERS,
            pfood: OIL_PLATFORM_PFOOD,
            pwood: OIL_PLATFORM_PWOOD,
            pstone: OIL_PLATFORM_PSTONE,
            piron: OIL_PLATFORM_PIRON,
            poil: OIL_PLATFORM_POIL,
            pgold: OIL_PLATFORM_PGOLD,
            penergy: OIL_PLATFORM_PENERGY,
            pcolonists: OIL_PLATFORM_PCOLONISTS,
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct OilPlatformBuildState {
    pub build_ticks_remaining: i32,
    pub completed: bool,
}

#[derive(Component, Clone, Copy)]
pub struct OilPlatformFootprint {
    pub size_tiles: i32,
    pub requires_oil_pool: bool,
}

impl Default for OilPlatformFootprint {
    fn default() -> Self {
        Self {
            size_tiles: OIL_PLATFORM_SIZE_TILES,
            requires_oil_pool: true,
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct OilPlatformOutputState {
    pub cycle_ticks_remaining: i32,
    pub oil_generated_total: i64,
}

#[derive(Component, Clone, Copy, Default)]
pub struct OilPlatformRulesState {
    pub requires_foundry_building: bool,
    pub requires_foundry_research: bool,
}

#[derive(Resource, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum ColonyMode {
    #[default]
    Standard,
    TheNewEmpire,
}

#[derive(Resource, Clone, Copy, Default)]
pub struct OilPlatformModeState {
    pub mode: ColonyMode,
}

#[derive(Resource, Clone, Copy, Default)]
pub struct FoundryAccessState {
    pub foundry_built: bool,
    pub foundry_research_complete: bool,
}

#[derive(Resource, Clone, Default)]
pub struct OilPoolTiles {
    pub tiles: BTreeSet<(i32, i32)>,
}

#[derive(Resource, Clone, Default)]
pub struct OilPlatformPlacementClaims {
    pub claims: BTreeMap<Entity, BuildingAnchor>,
}

#[derive(Resource, Clone, Copy, Default)]
pub struct EngineeringCenterOilQueuePolicy {
    pub queue_can_consume_oil: bool,
}

#[derive(Resource, Clone, Copy, Default)]
pub struct ColonyOilIncomeLedger {
    pub oil_income_per_cycle: i32,
}

#[derive(Event, Clone, Copy)]
pub struct SetOilPoolTileEvent {
    pub tile_x: i32,
    pub tile_y: i32,
    pub is_oil_pool: bool,
}

#[derive(Event, Clone, Copy)]
pub struct SetFoundryAccessEvent {
    pub foundry_built: bool,
    pub foundry_research_complete: bool,
}

#[derive(Event, Clone, Copy)]
pub struct SetColonyModeEvent {
    pub mode: ColonyMode,
}

#[derive(Event, Clone, Copy)]
pub struct PlaceOilPlatformEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct OilPlatformPlacementRejectedEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

fn build_seconds_to_ticks(seconds: i32) -> i32 {
    seconds * SIM_HZ
}

fn production_cycle_ticks() -> i32 {
    OIL_PLATFORM_PRODUCTION_CYCLE_HOURS * SECONDS_PER_HOUR * SIM_HZ
}

fn footprint_min_max(anchor: BuildingAnchor) -> (i32, i32, i32, i32) {
    let min_x = anchor.x;
    let min_y = anchor.y;
    let max_x = anchor.x + OIL_PLATFORM_SIZE_TILES - 1;
    let max_y = anchor.y + OIL_PLATFORM_SIZE_TILES - 1;
    (min_x, max_x, min_y, max_y)
}

fn footprint_tiles(anchor: BuildingAnchor) -> impl Iterator<Item = (i32, i32)> {
    let (min_x, max_x, min_y, max_y) = footprint_min_max(anchor);
    (min_x..=max_x).flat_map(move |x| (min_y..=max_y).map(move |y| (x, y)))
}

fn footprints_overlap(a: BuildingAnchor, b: BuildingAnchor) -> bool {
    let (a_min_x, a_max_x, a_min_y, a_max_y) = footprint_min_max(a);
    let (b_min_x, b_max_x, b_min_y, b_max_y) = footprint_min_max(b);
    !(a_max_x < b_min_x || b_max_x < a_min_x || a_max_y < b_min_y || b_max_y < a_min_y)
}

fn footprint_is_on_oil_pool(anchor: BuildingAnchor, oil_pool: &OilPoolTiles) -> bool {
    for tile in footprint_tiles(anchor) {
        if !oil_pool.tiles.contains(&tile) {
            return false;
        }
    }
    true
}

fn can_build_with_foundry_requirements(mode: ColonyMode, foundry: FoundryAccessState) -> bool {
    if !foundry.foundry_built {
        return false;
    }

    match mode {
        ColonyMode::Standard => foundry.foundry_research_complete,
        ColonyMode::TheNewEmpire => true,
    }
}

fn apply_oil_pool_tile_events_system(
    mut events: EventReader<SetOilPoolTileEvent>,
    mut oil_pool: ResMut<OilPoolTiles>,
) {
    for ev in events.read() {
        let tile = (ev.tile_x, ev.tile_y);
        if ev.is_oil_pool {
            oil_pool.tiles.insert(tile);
        } else {
            oil_pool.tiles.remove(&tile);
        }
    }
}

fn apply_foundry_access_events_system(
    mut events: EventReader<SetFoundryAccessEvent>,
    mut foundry: ResMut<FoundryAccessState>,
) {
    for ev in events.read() {
        foundry.foundry_built = ev.foundry_built;
        foundry.foundry_research_complete = ev.foundry_research_complete;
    }
}

fn apply_colony_mode_events_system(
    mut events: EventReader<SetColonyModeEvent>,
    mut mode: ResMut<OilPlatformModeState>,
) {
    for ev in events.read() {
        mode.mode = ev.mode;
    }
}

fn place_oil_platform_system(
    mut commands: Commands,
    mut events: EventReader<PlaceOilPlatformEvent>,
    mut rejected: EventWriter<OilPlatformPlacementRejectedEvent>,
    mut claims: ResMut<OilPlatformPlacementClaims>,
    oil_pool: Res<OilPoolTiles>,
    mode_state: Res<OilPlatformModeState>,
    foundry: Res<FoundryAccessState>,
) {
    for ev in events.read() {
        let anchor = BuildingAnchor {
            x: ev.tile_x,
            y: ev.tile_y,
        };

        let mut overlaps_existing_claim = false;
        for existing_anchor in claims.claims.values() {
            if footprints_overlap(anchor, *existing_anchor) {
                overlaps_existing_claim = true;
                break;
            }
        }

        if overlaps_existing_claim
            || !footprint_is_on_oil_pool(anchor, &oil_pool)
            || !can_build_with_foundry_requirements(mode_state.mode, *foundry)
        {
            rejected.send(OilPlatformPlacementRejectedEvent {
                tile_x: ev.tile_x,
                tile_y: ev.tile_y,
            });
            continue;
        }

        let rules = match mode_state.mode {
            ColonyMode::Standard => OilPlatformRulesState {
                requires_foundry_building: true,
                requires_foundry_research: true,
            },
            ColonyMode::TheNewEmpire => OilPlatformRulesState {
                requires_foundry_building: true,
                requires_foundry_research: false,
            },
        };

        let entity = commands
            .spawn((
                OilPlatform,
                anchor,
                OilPlatformCore::default(),
                OilPlatformEconomy::default(),
                OilPlatformBuildState {
                    build_ticks_remaining: build_seconds_to_ticks(OIL_PLATFORM_BUILD_TIME_SECONDS),
                    completed: false,
                },
                OilPlatformFootprint::default(),
                OilPlatformOutputState {
                    cycle_ticks_remaining: production_cycle_ticks(),
                    oil_generated_total: 0,
                },
                rules,
            ))
            .id();

        claims.claims.insert(entity, anchor);
    }
}

fn oil_platform_build_tick_system(
    mut platforms: Query<(&mut OilPlatformBuildState, &mut OilPlatformOutputState), With<OilPlatform>>,
) {
    for (mut build, mut output) in &mut platforms {
        if build.completed {
            continue;
        }

        if build.build_ticks_remaining > 0 {
            build.build_ticks_remaining -= 1;
        }

        if build.build_ticks_remaining <= 0 {
            build.build_ticks_remaining = 0;
            build.completed = true;
            output.cycle_ticks_remaining = production_cycle_ticks();
        }
    }
}

fn oil_platform_output_tick_system(
    mut platforms: Query<
        (&OilPlatformBuildState, &OilPlatformEconomy, &mut OilPlatformOutputState),
        With<OilPlatform>,
    >,
    mut ledger: ResMut<ColonyOilIncomeLedger>,
) {
    ledger.oil_income_per_cycle = 0;

    for (build, economy, mut output) in &mut platforms {
        if !build.completed {
            continue;
        }

        ledger.oil_income_per_cycle += economy.poil;

        if output.cycle_ticks_remaining > 0 {
            output.cycle_ticks_remaining -= 1;
        }

        if output.cycle_ticks_remaining <= 0 {
            output.oil_generated_total += i64::from(economy.poil);
            output.cycle_ticks_remaining = production_cycle_ticks();
        }
    }
}

fn engineering_center_oil_queue_policy_system(
    platforms: Query<&OilPlatformBuildState, With<OilPlatform>>,
    mut policy: ResMut<EngineeringCenterOilQueuePolicy>,
) {
    let mut has_operational_platform = false;
    for build in &platforms {
        if build.completed {
            has_operational_platform = true;
            break;
        }
    }

    policy.queue_can_consume_oil = has_operational_platform;
}

fn oil_platform_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    platforms: Query<
        (
            Entity,
            &BuildingAnchor,
            &OilPlatformCore,
            &OilPlatformEconomy,
            &OilPlatformBuildState,
            &OilPlatformFootprint,
            &OilPlatformOutputState,
            &OilPlatformRulesState,
        ),
        With<OilPlatform>,
    >,
    oil_pool: Res<OilPoolTiles>,
    claims: Res<OilPlatformPlacementClaims>,
    mode_state: Res<OilPlatformModeState>,
    foundry: Res<FoundryAccessState>,
    policy: Res<EngineeringCenterOilQueuePolicy>,
    ledger: Res<ColonyOilIncomeLedger>,
) {
    for (entity, anchor, core, economy, build, footprint, output, rules) in &platforms {
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
        checksum.accumulate(economy.pfood as u64);
        checksum.accumulate(economy.pwood as u64);
        checksum.accumulate(economy.pstone as u64);
        checksum.accumulate(economy.piron as u64);
        checksum.accumulate(economy.poil as u64);
        checksum.accumulate(economy.pgold as u64);
        checksum.accumulate(economy.penergy as u64);
        checksum.accumulate(economy.pcolonists as u64);

        checksum.accumulate(build.build_ticks_remaining as u64);
        checksum.accumulate(u64::from(build.completed));

        checksum.accumulate(footprint.size_tiles as u64);
        checksum.accumulate(u64::from(footprint.requires_oil_pool));

        checksum.accumulate(output.cycle_ticks_remaining as u64);
        checksum.accumulate(output.oil_generated_total as u64);

        checksum.accumulate(u64::from(rules.requires_foundry_building));
        checksum.accumulate(u64::from(rules.requires_foundry_research));
    }

    checksum.accumulate(oil_pool.tiles.len() as u64);
    for (x, y) in &oil_pool.tiles {
        checksum.accumulate(*x as u64);
        checksum.accumulate(*y as u64);
    }

    checksum.accumulate(claims.claims.len() as u64);
    for (entity, anchor) in &claims.claims {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);
    }

    match mode_state.mode {
        ColonyMode::Standard => checksum.accumulate(0),
        ColonyMode::TheNewEmpire => checksum.accumulate(1),
    }

    checksum.accumulate(u64::from(foundry.foundry_built));
    checksum.accumulate(u64::from(foundry.foundry_research_complete));
    checksum.accumulate(u64::from(policy.queue_can_consume_oil));
    checksum.accumulate(ledger.oil_income_per_cycle as u64);
}

pub struct OilPlatformPlugin;

impl Plugin for OilPlatformPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OilPlatformModeState>()
            .init_resource::<FoundryAccessState>()
            .init_resource::<OilPoolTiles>()
            .init_resource::<OilPlatformPlacementClaims>()
            .init_resource::<EngineeringCenterOilQueuePolicy>()
            .init_resource::<ColonyOilIncomeLedger>()
            .add_event::<SetOilPoolTileEvent>()
            .add_event::<SetFoundryAccessEvent>()
            .add_event::<SetColonyModeEvent>()
            .add_event::<PlaceOilPlatformEvent>()
            .add_event::<OilPlatformPlacementRejectedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    apply_oil_pool_tile_events_system,
                    apply_foundry_access_events_system,
                    apply_colony_mode_events_system,
                    place_oil_platform_system,
                    oil_platform_build_tick_system,
                    oil_platform_output_tick_system,
                    engineering_center_oil_queue_policy_system,
                    oil_platform_checksum_system,
                )
                    .chain(),
            );
    }
}