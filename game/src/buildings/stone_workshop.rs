// Sources: vault/buildings/stone_workshop.md

use std::collections::{BTreeMap, BTreeSet};

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{Health, SimChecksumState};

const STONE_WORKSHOP_HP: I32F32 = I32F32::lit("1000");
const STONE_WORKSHOP_DEFENSES_LIFE: I32F32 = I32F32::lit("250");
const STONE_WORKSHOP_WATCH_RANGE: I32F32 = I32F32::lit("7");
const STONE_WORKSHOP_ENERGY_COST: i32 = 20;
const STONE_WORKSHOP_WOOD_COST: i32 = 40;
const STONE_WORKSHOP_STONE_COST: i32 = 40;
const STONE_WORKSHOP_IRON_COST: i32 = 0;
const STONE_WORKSHOP_OIL_COST: i32 = 0;
const STONE_WORKSHOP_GOLD_COST: i32 = 1000;
const STONE_WORKSHOP_BUILD_TIME_SECONDS: i32 = 90;
const STONE_WORKSHOP_WORKERS: i32 = 20;
const STONE_WORKSHOP_SIZE_TILES: i32 = 4;
const STONE_WORKSHOP_GOLD_MAINTENANCE: i32 = 0;
const STONE_WORKSHOP_RESEARCH_SECONDS_UNSPECIFIED: i32 = 0;
const SIM_HZ: i32 = 25;

const STONE_HOUSE_GOLD_COST: i32 = 600;
const POWER_PLANT_GOLD_COST: i32 = 800;
const FOUNDRY_GOLD_COST: i32 = 1000;
const BANK_GOLD_COST: i32 = 800;
const STONE_WALL_GOLD_COST: i32 = 500;
const STONE_TOWER_GOLD_COST: i32 = 500;
const SHOCKING_TOWER_GOLD_COST: i32 = 900;
const WASP_GOLD_COST: i32 = 800;
const STONE_GATE_GOLD_COST: i32 = 0;
const THE_VICTORIOUS_GOLD_COST: i32 = 7000;
const THE_VICTORIOUS_WOOD_COST: i32 = 0;
const THE_VICTORIOUS_STONE_COST: i32 = 100;
const THE_VICTORIOUS_IRON_COST: i32 = 200;
const THE_VICTORIOUS_OIL_COST: i32 = 100;
const THE_ACADEMY_OF_IMMORTALS_GOLD_COST: i32 = 7000;
const THE_ACADEMY_OF_IMMORTALS_WOOD_COST: i32 = 200;
const THE_ACADEMY_OF_IMMORTALS_STONE_COST: i32 = 200;
const THE_ACADEMY_OF_IMMORTALS_IRON_COST: i32 = 100;
const THE_ACADEMY_OF_IMMORTALS_OIL_COST: i32 = 0;

const THE_VICTORIOUS_DWELLING_GOLD_MULTIPLIER: I32F32 = I32F32::lit("1.20");
const THE_VICTORIOUS_VICTORY_POINTS_BONUS: i32 = 2000;
const THE_VICTORIOUS_OIL_SUPPLY_DELTA: i32 = -10;

#[derive(Component, Default)]
pub struct StoneWorkshop;

#[derive(Component, Clone, Copy, Default)]
pub struct BuildingAnchor {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Clone, Copy)]
pub struct StoneWorkshopCoreStats {
    pub defenses_life: I32F32,
    pub watch_range: I32F32,
    pub health: Health,
}

impl Default for StoneWorkshopCoreStats {
    fn default() -> Self {
        Self {
            defenses_life: STONE_WORKSHOP_DEFENSES_LIFE,
            watch_range: STONE_WORKSHOP_WATCH_RANGE,
            health: Health::full(STONE_WORKSHOP_HP),
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct StoneWorkshopBuildState {
    pub build_ticks_remaining: i32,
    pub completed: bool,
}

#[derive(Component, Clone, Copy)]
pub struct StoneWorkshopEconomy {
    pub energy_cost: i32,
    pub wood_cost: i32,
    pub stone_cost: i32,
    pub iron_cost: i32,
    pub oil_cost: i32,
    pub gold_cost: i32,
    pub workers: i32,
    pub gold_maintenance: i32,
}

impl Default for StoneWorkshopEconomy {
    fn default() -> Self {
        Self {
            energy_cost: STONE_WORKSHOP_ENERGY_COST,
            wood_cost: STONE_WORKSHOP_WOOD_COST,
            stone_cost: STONE_WORKSHOP_STONE_COST,
            iron_cost: STONE_WORKSHOP_IRON_COST,
            oil_cost: STONE_WORKSHOP_OIL_COST,
            gold_cost: STONE_WORKSHOP_GOLD_COST,
            workers: STONE_WORKSHOP_WORKERS,
            gold_maintenance: STONE_WORKSHOP_GOLD_MAINTENANCE,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct StoneWorkshopFootprint {
    pub size_tiles: i32,
    pub watch_range: I32F32,
}

impl Default for StoneWorkshopFootprint {
    fn default() -> Self {
        Self {
            size_tiles: STONE_WORKSHOP_SIZE_TILES,
            watch_range: STONE_WORKSHOP_WATCH_RANGE,
        }
    }
}

#[derive(Clone, Copy)]
pub enum StoneWorkshopResearchOption {
    StoneHouse,
    PowerPlant,
    Foundry,
    Bank,
    StoneWall,
    StoneGate,
    StoneTower,
    ShockingTower,
    Wasp,
    TheVictorious,
    TheAcademyOfImmortals,
}

#[derive(Component, Clone, Copy)]
pub struct StoneWorkshopState {
    pub connected_to_grid: bool,
    pub destroyed: bool,
    pub demolished: bool,
    pub wood_workshop_prerequisite_met: bool,
}

impl Default for StoneWorkshopState {
    fn default() -> Self {
        Self {
            connected_to_grid: true,
            destroyed: false,
            demolished: false,
            wood_workshop_prerequisite_met: false,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct StoneWorkshopResearchState {
    pub active: Option<StoneWorkshopResearchOption>,
    pub ticks_remaining: i32,
}

impl Default for StoneWorkshopResearchState {
    fn default() -> Self {
        Self {
            active: None,
            ticks_remaining: 0,
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct StoneWorkshopUnlocked {
    pub stone_house: bool,
    pub power_plant: bool,
    pub foundry: bool,
    pub bank: bool,
    pub stone_wall: bool,
    pub stone_gate: bool,
    pub stone_tower: bool,
    pub shocking_tower: bool,
    pub wasp: bool,
    pub the_victorious: bool,
    pub the_academy_of_immortals: bool,
}

#[derive(Component, Clone, Copy)]
pub struct StoneWorkshopWonderEffects {
    pub victorious_dwellings_gold_multiplier: I32F32,
    pub victorious_victory_points_bonus: i32,
    pub victorious_oil_supply_delta: i32,
    pub academy_soldiers_center_units_become_veteran: bool,
    pub academy_applies_to_existing_and_new_units: bool,
}

impl Default for StoneWorkshopWonderEffects {
    fn default() -> Self {
        Self {
            victorious_dwellings_gold_multiplier: THE_VICTORIOUS_DWELLING_GOLD_MULTIPLIER,
            victorious_victory_points_bonus: THE_VICTORIOUS_VICTORY_POINTS_BONUS,
            victorious_oil_supply_delta: THE_VICTORIOUS_OIL_SUPPLY_DELTA,
            academy_soldiers_center_units_become_veteran: true,
            academy_applies_to_existing_and_new_units: true,
        }
    }
}

#[derive(Resource, Default, Clone)]
pub struct TileOccupancy {
    pub blocked_tiles: BTreeSet<(i32, i32)>,
}

#[derive(Resource, Default, Clone)]
pub struct StoneWorkshopPlacementClaims {
    pub claims: BTreeMap<Entity, BuildingAnchor>,
}

#[derive(Event, Clone, Copy)]
pub struct SetTileBlockedEvent {
    pub tile_x: i32,
    pub tile_y: i32,
    pub blocked: bool,
}

#[derive(Event, Clone, Copy)]
pub struct PlaceStoneWorkshopEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct StartStoneWorkshopResearchEvent {
    pub entity: Entity,
    pub option: StoneWorkshopResearchOption,
}

#[derive(Event, Clone, Copy)]
pub struct SetStoneWorkshopGridConnectionEvent {
    pub entity: Entity,
    pub connected: bool,
}

#[derive(Event, Clone, Copy)]
pub struct SetStoneWorkshopPrerequisiteEvent {
    pub entity: Entity,
    pub wood_workshop_built: bool,
}

#[derive(Event, Clone, Copy)]
pub struct DemolishStoneWorkshopEvent {
    pub entity: Entity,
}

#[derive(Event, Clone, Copy)]
pub struct AttackDestroyStoneWorkshopEvent {
    pub entity: Entity,
}

fn seconds_to_ticks(seconds: i32) -> i32 {
    seconds * SIM_HZ
}

fn footprint_tiles(anchor: BuildingAnchor) -> impl Iterator<Item = (i32, i32)> {
    let min_x = anchor.x;
    let min_y = anchor.y;
    let max_x = anchor.x + STONE_WORKSHOP_SIZE_TILES - 1;
    let max_y = anchor.y + STONE_WORKSHOP_SIZE_TILES - 1;
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

fn clear_unlocked(unlocked: &mut StoneWorkshopUnlocked) {
    *unlocked = StoneWorkshopUnlocked::default();
}

fn research_duration_seconds(_option: StoneWorkshopResearchOption) -> i32 {
    STONE_WORKSHOP_RESEARCH_SECONDS_UNSPECIFIED
}

fn research_total_cost(option: StoneWorkshopResearchOption) -> i32 {
    match option {
        StoneWorkshopResearchOption::StoneHouse => STONE_HOUSE_GOLD_COST,
        StoneWorkshopResearchOption::PowerPlant => POWER_PLANT_GOLD_COST,
        StoneWorkshopResearchOption::Foundry => FOUNDRY_GOLD_COST,
        StoneWorkshopResearchOption::Bank => BANK_GOLD_COST,
        StoneWorkshopResearchOption::StoneWall => STONE_WALL_GOLD_COST,
        StoneWorkshopResearchOption::StoneGate => STONE_GATE_GOLD_COST,
        StoneWorkshopResearchOption::StoneTower => STONE_TOWER_GOLD_COST,
        StoneWorkshopResearchOption::ShockingTower => SHOCKING_TOWER_GOLD_COST,
        StoneWorkshopResearchOption::Wasp => WASP_GOLD_COST,
        StoneWorkshopResearchOption::TheVictorious => {
            THE_VICTORIOUS_GOLD_COST
                + THE_VICTORIOUS_WOOD_COST
                + THE_VICTORIOUS_STONE_COST
                + THE_VICTORIOUS_IRON_COST
                + THE_VICTORIOUS_OIL_COST
        }
        StoneWorkshopResearchOption::TheAcademyOfImmortals => {
            THE_ACADEMY_OF_IMMORTALS_GOLD_COST
                + THE_ACADEMY_OF_IMMORTALS_WOOD_COST
                + THE_ACADEMY_OF_IMMORTALS_STONE_COST
                + THE_ACADEMY_OF_IMMORTALS_IRON_COST
                + THE_ACADEMY_OF_IMMORTALS_OIL_COST
        }
    }
}

fn is_unlocked(unlocked: &StoneWorkshopUnlocked, option: StoneWorkshopResearchOption) -> bool {
    match option {
        StoneWorkshopResearchOption::StoneHouse => unlocked.stone_house,
        StoneWorkshopResearchOption::PowerPlant => unlocked.power_plant,
        StoneWorkshopResearchOption::Foundry => unlocked.foundry,
        StoneWorkshopResearchOption::Bank => unlocked.bank,
        StoneWorkshopResearchOption::StoneWall => unlocked.stone_wall,
        StoneWorkshopResearchOption::StoneGate => unlocked.stone_gate,
        StoneWorkshopResearchOption::StoneTower => unlocked.stone_tower,
        StoneWorkshopResearchOption::ShockingTower => unlocked.shocking_tower,
        StoneWorkshopResearchOption::Wasp => unlocked.wasp,
        StoneWorkshopResearchOption::TheVictorious => unlocked.the_victorious,
        StoneWorkshopResearchOption::TheAcademyOfImmortals => unlocked.the_academy_of_immortals,
    }
}

fn unlock_option(unlocked: &mut StoneWorkshopUnlocked, option: StoneWorkshopResearchOption) {
    match option {
        StoneWorkshopResearchOption::StoneHouse => unlocked.stone_house = true,
        StoneWorkshopResearchOption::PowerPlant => unlocked.power_plant = true,
        StoneWorkshopResearchOption::Foundry => unlocked.foundry = true,
        StoneWorkshopResearchOption::Bank => unlocked.bank = true,
        StoneWorkshopResearchOption::StoneWall => unlocked.stone_wall = true,
        StoneWorkshopResearchOption::StoneGate => unlocked.stone_gate = true,
        StoneWorkshopResearchOption::StoneTower => unlocked.stone_tower = true,
        StoneWorkshopResearchOption::ShockingTower => unlocked.shocking_tower = true,
        StoneWorkshopResearchOption::Wasp => unlocked.wasp = true,
        StoneWorkshopResearchOption::TheVictorious => unlocked.the_victorious = true,
        StoneWorkshopResearchOption::TheAcademyOfImmortals => {
            unlocked.the_academy_of_immortals = true
        }
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

fn place_stone_workshop_system(
    mut commands: Commands,
    mut events: EventReader<PlaceStoneWorkshopEvent>,
    occupancy: Res<TileOccupancy>,
    mut claims: ResMut<StoneWorkshopPlacementClaims>,
) {
    for ev in events.read() {
        let anchor = BuildingAnchor {
            x: ev.tile_x,
            y: ev.tile_y,
        };

        if !footprint_is_unblocked(anchor, &occupancy) {
            continue;
        }

        let entity = commands
            .spawn((
                StoneWorkshop,
                anchor,
                StoneWorkshopCoreStats::default(),
                StoneWorkshopBuildState {
                    build_ticks_remaining: seconds_to_ticks(STONE_WORKSHOP_BUILD_TIME_SECONDS),
                    completed: false,
                },
                StoneWorkshopEconomy::default(),
                StoneWorkshopFootprint::default(),
                StoneWorkshopState::default(),
                StoneWorkshopResearchState::default(),
                StoneWorkshopUnlocked::default(),
                StoneWorkshopWonderEffects::default(),
            ))
            .id();

        claims.claims.insert(entity, anchor);
    }
}

fn set_stone_workshop_grid_connection_system(
    mut events: EventReader<SetStoneWorkshopGridConnectionEvent>,
    mut workshops: Query<&mut StoneWorkshopState, With<StoneWorkshop>>,
) {
    for ev in events.read() {
        let Ok(mut state) = workshops.get_mut(ev.entity) else {
            continue;
        };
        state.connected_to_grid = ev.connected;
    }
}

fn set_stone_workshop_prerequisite_system(
    mut events: EventReader<SetStoneWorkshopPrerequisiteEvent>,
    mut workshops: Query<&mut StoneWorkshopState, With<StoneWorkshop>>,
) {
    for ev in events.read() {
        let Ok(mut state) = workshops.get_mut(ev.entity) else {
            continue;
        };
        state.wood_workshop_prerequisite_met = ev.wood_workshop_built;
    }
}

fn stone_workshop_build_tick_system(
    mut workshops: Query<
        (
            &mut StoneWorkshopBuildState,
            &mut StoneWorkshopState,
            &mut StoneWorkshopResearchState,
            &mut StoneWorkshopUnlocked,
        ),
        With<StoneWorkshop>,
    >,
) {
    for (mut build, state, mut research, mut unlocked) in &mut workshops {
        if !build.completed {
            if build.build_ticks_remaining > 0 {
                build.build_ticks_remaining -= 1;
            }

            if build.build_ticks_remaining <= 0 {
                build.build_ticks_remaining = 0;
                build.completed = true;
            }
        }

        if state.destroyed || state.demolished {
            research.active = None;
            research.ticks_remaining = 0;
            clear_unlocked(&mut unlocked);
        }
    }
}

fn stone_workshop_start_research_system(
    mut events: EventReader<StartStoneWorkshopResearchEvent>,
    mut workshops: Query<
        (
            &StoneWorkshopBuildState,
            &StoneWorkshopState,
            &mut StoneWorkshopResearchState,
            &StoneWorkshopUnlocked,
        ),
        With<StoneWorkshop>,
    >,
) {
    for ev in events.read() {
        let Ok((build, state, mut research, unlocked)) = workshops.get_mut(ev.entity) else {
            continue;
        };

        if !build.completed
            || state.destroyed
            || state.demolished
            || !state.connected_to_grid
            || !state.wood_workshop_prerequisite_met
        {
            continue;
        }

        if research.active.is_some() || is_unlocked(unlocked, ev.option) {
            continue;
        }

        if matches!(ev.option, StoneWorkshopResearchOption::StoneGate) && !unlocked.stone_wall {
            continue;
        }

        research.active = Some(ev.option);
        research.ticks_remaining = seconds_to_ticks(research_duration_seconds(ev.option));
    }
}

fn stone_workshop_research_tick_and_unlock_system(
    mut workshops: Query<
        (
            &StoneWorkshopState,
            &mut StoneWorkshopResearchState,
            &mut StoneWorkshopUnlocked,
        ),
        With<StoneWorkshop>,
    >,
) {
    for (state, mut research, mut unlocked) in &mut workshops {
        if !state.connected_to_grid
            || state.destroyed
            || state.demolished
            || !state.wood_workshop_prerequisite_met
        {
            continue;
        }

        let Some(active) = research.active else {
            continue;
        };

        if research.ticks_remaining > 0 {
            research.ticks_remaining -= 1;
        }

        if research.ticks_remaining > 0 {
            continue;
        }

        research.ticks_remaining = 0;
        research.active = None;
        unlock_option(&mut unlocked, active);
    }
}

fn attack_destroy_stone_workshop_system(
    mut events: EventReader<AttackDestroyStoneWorkshopEvent>,
    mut workshops: Query<
        (
            &mut StoneWorkshopState,
            &mut StoneWorkshopResearchState,
            &mut StoneWorkshopUnlocked,
        ),
        With<StoneWorkshop>,
    >,
) {
    for ev in events.read() {
        let Ok((mut state, mut research, mut unlocked)) = workshops.get_mut(ev.entity) else {
            continue;
        };

        state.destroyed = true;
        research.active = None;
        research.ticks_remaining = 0;
        clear_unlocked(&mut unlocked);
    }
}

fn destroy_stone_workshop_at_zero_health_system(
    mut workshops: Query<
        (
            &StoneWorkshopCoreStats,
            &mut StoneWorkshopState,
            &mut StoneWorkshopResearchState,
            &mut StoneWorkshopUnlocked,
        ),
        With<StoneWorkshop>,
    >,
) {
    for (core, mut state, mut research, mut unlocked) in &mut workshops {
        if core.health.current > I32F32::ZERO {
            continue;
        }

        state.destroyed = true;
        research.active = None;
        research.ticks_remaining = 0;
        clear_unlocked(&mut unlocked);
    }
}

fn demolish_stone_workshop_system(
    mut events: EventReader<DemolishStoneWorkshopEvent>,
    mut workshops: Query<
        (
            &mut StoneWorkshopState,
            &mut StoneWorkshopResearchState,
            &mut StoneWorkshopUnlocked,
        ),
        With<StoneWorkshop>,
    >,
) {
    for ev in events.read() {
        let Ok((mut state, mut research, mut unlocked)) = workshops.get_mut(ev.entity) else {
            continue;
        };

        state.demolished = true;
        research.active = None;
        research.ticks_remaining = 0;
        clear_unlocked(&mut unlocked);
    }
}

fn stone_workshop_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    workshops: Query<
        (
            Entity,
            &BuildingAnchor,
            &StoneWorkshopCoreStats,
            &StoneWorkshopBuildState,
            &StoneWorkshopEconomy,
            &StoneWorkshopFootprint,
            &StoneWorkshopState,
            &StoneWorkshopResearchState,
            &StoneWorkshopUnlocked,
            &StoneWorkshopWonderEffects,
        ),
        With<StoneWorkshop>,
    >,
    claims: Res<StoneWorkshopPlacementClaims>,
    occupancy: Res<TileOccupancy>,
) {
    for (entity, anchor, core, build, eco, footprint, state, research, unlocked, effects) in
        &workshops
    {
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

        checksum.accumulate(u64::from(state.connected_to_grid));
        checksum.accumulate(u64::from(state.destroyed));
        checksum.accumulate(u64::from(state.demolished));
        checksum.accumulate(u64::from(state.wood_workshop_prerequisite_met));

        let active_research_id = match research.active {
            None => 0u64,
            Some(StoneWorkshopResearchOption::StoneHouse) => 1,
            Some(StoneWorkshopResearchOption::PowerPlant) => 2,
            Some(StoneWorkshopResearchOption::Foundry) => 3,
            Some(StoneWorkshopResearchOption::Bank) => 4,
            Some(StoneWorkshopResearchOption::StoneWall) => 5,
            Some(StoneWorkshopResearchOption::StoneGate) => 6,
            Some(StoneWorkshopResearchOption::StoneTower) => 7,
            Some(StoneWorkshopResearchOption::ShockingTower) => 8,
            Some(StoneWorkshopResearchOption::Wasp) => 9,
            Some(StoneWorkshopResearchOption::TheVictorious) => 10,
            Some(StoneWorkshopResearchOption::TheAcademyOfImmortals) => 11,
        };

        checksum.accumulate(active_research_id);
        checksum.accumulate(research.ticks_remaining as u64);
        checksum.accumulate(research.active.map(research_total_cost).unwrap_or(0) as u64);

        checksum.accumulate(u64::from(unlocked.stone_house));
        checksum.accumulate(u64::from(unlocked.power_plant));
        checksum.accumulate(u64::from(unlocked.foundry));
        checksum.accumulate(u64::from(unlocked.bank));
        checksum.accumulate(u64::from(unlocked.stone_wall));
        checksum.accumulate(u64::from(unlocked.stone_gate));
        checksum.accumulate(u64::from(unlocked.stone_tower));
        checksum.accumulate(u64::from(unlocked.shocking_tower));
        checksum.accumulate(u64::from(unlocked.wasp));
        checksum.accumulate(u64::from(unlocked.the_victorious));
        checksum.accumulate(u64::from(unlocked.the_academy_of_immortals));

        checksum.accumulate(effects.victorious_dwellings_gold_multiplier.to_bits() as u64);
        checksum.accumulate(effects.victorious_victory_points_bonus as u64);
        checksum.accumulate(effects.victorious_oil_supply_delta as u64);
        checksum.accumulate(u64::from(
            effects.academy_soldiers_center_units_become_veteran,
        ));
        checksum.accumulate(u64::from(
            effects.academy_applies_to_existing_and_new_units,
        ));
    }

    checksum.accumulate(claims.claims.len() as u64);
    for (entity, anchor) in &claims.claims {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);
    }

    checksum.accumulate(occupancy.blocked_tiles.len() as u64);
    for (x, y) in &occupancy.blocked_tiles {
        checksum.accumulate(*x as u64);
        checksum.accumulate(*y as u64);
    }
}

pub struct StoneWorkshopPlugin;

impl Plugin for StoneWorkshopPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileOccupancy>()
            .init_resource::<StoneWorkshopPlacementClaims>()
            .add_event::<SetTileBlockedEvent>()
            .add_event::<PlaceStoneWorkshopEvent>()
            .add_event::<StartStoneWorkshopResearchEvent>()
            .add_event::<SetStoneWorkshopGridConnectionEvent>()
            .add_event::<SetStoneWorkshopPrerequisiteEvent>()
            .add_event::<DemolishStoneWorkshopEvent>()
            .add_event::<AttackDestroyStoneWorkshopEvent>()
            .add_systems(
                FixedUpdate,
                (
                    apply_tile_block_events_system,
                    place_stone_workshop_system,
                    set_stone_workshop_grid_connection_system,
                    set_stone_workshop_prerequisite_system,
                    stone_workshop_build_tick_system,
                    stone_workshop_start_research_system,
                    stone_workshop_research_tick_and_unlock_system,
                    attack_destroy_stone_workshop_system,
                    destroy_stone_workshop_at_zero_health_system,
                    demolish_stone_workshop_system,
                    stone_workshop_checksum_system,
                )
                    .chain(),
            );
    }
}