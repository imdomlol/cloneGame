// Sources: vault/buildings/wood_workshop.md

use std::collections::{BTreeMap, BTreeSet};

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::SimChecksumState;

const WOOD_WORKSHOP_HP: I32F32 = I32F32::lit("800");
const WOOD_WORKSHOP_DEFENSES_LIFE: I32F32 = I32F32::lit("200");
const WOOD_WORKSHOP_WATCH_RANGE: I32F32 = I32F32::lit("7");
const WOOD_WORKSHOP_ENERGY_COST: i32 = 10;
const WOOD_WORKSHOP_WOOD_COST: i32 = 40;
const WOOD_WORKSHOP_STONE_COST: i32 = 0;
const WOOD_WORKSHOP_IRON_COST: i32 = 0;
const WOOD_WORKSHOP_OIL_COST: i32 = 0;
const WOOD_WORKSHOP_GOLD_COST: i32 = 500;
const WOOD_WORKSHOP_BUILD_TIME_SECONDS: i32 = 90;
const WOOD_WORKSHOP_WORKERS: i32 = 10;
const WOOD_WORKSHOP_GOLD_MAINTENANCE: i32 = 16;
const WOOD_WORKSHOP_SIZE_TILES: i32 = 4;
const WOOD_WORKSHOP_RESEARCH_SECONDS_STANDARD: i32 = 75;
const WOOD_WORKSHOP_RESEARCH_SECONDS_UNSPECIFIED: i32 = 0;
const SIM_HZ: i32 = 25;

const COTTAGE_GOLD_COST: i32 = 350;
const FARM_GOLD_COST: i32 = 400;
const STONE_WORKSHOP_GOLD_COST: i32 = 500;
const MARKET_GOLD_COST: i32 = 450;
const LOOKOUT_TOWER_GOLD_COST: i32 = 300;
const GREAT_BALLISTA_GOLD_COST: i32 = 700;
const STAKES_TRAP_GOLD_COST: i32 = 300;
const SNIPER_GOLD_COST: i32 = 700;
const INN_GOLD_COST: i32 = 1000;
const THE_SILENT_BEHOLDER_GOLD_COST: i32 = 7000;
const THE_SILENT_BEHOLDER_WOOD_COST: i32 = 100;
const THE_SILENT_BEHOLDER_STONE_COST: i32 = 100;
const THE_SILENT_BEHOLDER_IRON_COST: i32 = 150;
const THE_CRYSTAL_PALACE_GOLD_COST: i32 = 6000;
const THE_CRYSTAL_PALACE_WOOD_COST: i32 = 150;
const THE_CRYSTAL_PALACE_STONE_COST: i32 = 200;
const THE_CRYSTAL_PALACE_IRON_COST: i32 = 150;

#[derive(Component, Default)]
pub struct WoodWorkshop;

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
            hp: WOOD_WORKSHOP_HP,
            defenses_life: WOOD_WORKSHOP_DEFENSES_LIFE,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct WoodWorkshopBuildState {
    pub build_ticks_remaining: i32,
    pub completed: bool,
}

#[derive(Component, Clone, Copy)]
pub struct WoodWorkshopEconomy {
    pub energy_cost: i32,
    pub wood_cost: i32,
    pub stone_cost: i32,
    pub iron_cost: i32,
    pub oil_cost: i32,
    pub gold_cost: i32,
    pub workers: i32,
    pub gold_maintenance: i32,
}

impl Default for WoodWorkshopEconomy {
    fn default() -> Self {
        Self {
            energy_cost: WOOD_WORKSHOP_ENERGY_COST,
            wood_cost: WOOD_WORKSHOP_WOOD_COST,
            stone_cost: WOOD_WORKSHOP_STONE_COST,
            iron_cost: WOOD_WORKSHOP_IRON_COST,
            oil_cost: WOOD_WORKSHOP_OIL_COST,
            gold_cost: WOOD_WORKSHOP_GOLD_COST,
            workers: WOOD_WORKSHOP_WORKERS,
            gold_maintenance: WOOD_WORKSHOP_GOLD_MAINTENANCE,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct WoodWorkshopFootprint {
    pub size_tiles: i32,
    pub watch_range: I32F32,
}

impl Default for WoodWorkshopFootprint {
    fn default() -> Self {
        Self {
            size_tiles: WOOD_WORKSHOP_SIZE_TILES,
            watch_range: WOOD_WORKSHOP_WATCH_RANGE,
        }
    }
}

#[derive(Clone, Copy)]
pub enum WoodWorkshopResearchOption {
    Cottage,
    Farm,
    StoneWorkshop,
    Market,
    LookoutTower,
    GreatBallista,
    StakesTrap,
    Sniper,
    Inn,
    TheSilentBeholder,
    TheCrystalPalace,
}

#[derive(Component, Clone, Copy, Default)]
pub struct WoodWorkshopState {
    pub connected_to_grid: bool,
    pub destroyed: bool,
}

#[derive(Component, Clone, Copy)]
pub struct WoodWorkshopResearchState {
    pub active: Option<WoodWorkshopResearchOption>,
    pub ticks_remaining: i32,
}

impl Default for WoodWorkshopResearchState {
    fn default() -> Self {
        Self {
            active: None,
            ticks_remaining: 0,
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct WoodWorkshopUnlocked {
    pub cottage: bool,
    pub farm: bool,
    pub stone_workshop: bool,
    pub market: bool,
    pub lookout_tower: bool,
    pub great_ballista: bool,
    pub stakes_trap: bool,
    pub sniper: bool,
    pub inn: bool,
    pub the_silent_beholder: bool,
    pub the_crystal_palace: bool,
}

#[derive(Resource, Default, Clone)]
pub struct TileOccupancy {
    pub blocked_tiles: BTreeSet<(i32, i32)>,
}

#[derive(Resource, Default, Clone)]
pub struct WoodWorkshopPlacementClaims {
    pub claims: BTreeMap<Entity, BuildingAnchor>,
}

#[derive(Event, Clone, Copy)]
pub struct SetTileBlockedEvent {
    pub tile_x: i32,
    pub tile_y: i32,
    pub blocked: bool,
}

#[derive(Event, Clone, Copy)]
pub struct PlaceWoodWorkshopEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct StartWoodWorkshopResearchEvent {
    pub entity: Entity,
    pub option: WoodWorkshopResearchOption,
}

#[derive(Event, Clone, Copy)]
pub struct SetWoodWorkshopGridConnectionEvent {
    pub entity: Entity,
    pub connected: bool,
}

fn seconds_to_ticks(seconds: i32) -> i32 {
    seconds * SIM_HZ
}

fn footprint_tiles(anchor: BuildingAnchor) -> impl Iterator<Item = (i32, i32)> {
    let min_x = anchor.x;
    let min_y = anchor.y;
    let max_x = anchor.x + WOOD_WORKSHOP_SIZE_TILES - 1;
    let max_y = anchor.y + WOOD_WORKSHOP_SIZE_TILES - 1;
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

fn research_duration_seconds(option: WoodWorkshopResearchOption) -> i32 {
    match option {
        WoodWorkshopResearchOption::Cottage
        | WoodWorkshopResearchOption::Farm
        | WoodWorkshopResearchOption::StoneWorkshop
        | WoodWorkshopResearchOption::Market
        | WoodWorkshopResearchOption::LookoutTower
        | WoodWorkshopResearchOption::GreatBallista
        | WoodWorkshopResearchOption::StakesTrap
        | WoodWorkshopResearchOption::Sniper => WOOD_WORKSHOP_RESEARCH_SECONDS_STANDARD,
        WoodWorkshopResearchOption::Inn
        | WoodWorkshopResearchOption::TheSilentBeholder
        | WoodWorkshopResearchOption::TheCrystalPalace => WOOD_WORKSHOP_RESEARCH_SECONDS_UNSPECIFIED,
    }
}

fn research_total_cost(option: WoodWorkshopResearchOption) -> i32 {
    match option {
        WoodWorkshopResearchOption::Cottage => COTTAGE_GOLD_COST,
        WoodWorkshopResearchOption::Farm => FARM_GOLD_COST,
        WoodWorkshopResearchOption::StoneWorkshop => STONE_WORKSHOP_GOLD_COST,
        WoodWorkshopResearchOption::Market => MARKET_GOLD_COST,
        WoodWorkshopResearchOption::LookoutTower => LOOKOUT_TOWER_GOLD_COST,
        WoodWorkshopResearchOption::GreatBallista => GREAT_BALLISTA_GOLD_COST,
        WoodWorkshopResearchOption::StakesTrap => STAKES_TRAP_GOLD_COST,
        WoodWorkshopResearchOption::Sniper => SNIPER_GOLD_COST,
        WoodWorkshopResearchOption::Inn => INN_GOLD_COST,
        WoodWorkshopResearchOption::TheSilentBeholder => {
            THE_SILENT_BEHOLDER_GOLD_COST
                + THE_SILENT_BEHOLDER_WOOD_COST
                + THE_SILENT_BEHOLDER_STONE_COST
                + THE_SILENT_BEHOLDER_IRON_COST
        }
        WoodWorkshopResearchOption::TheCrystalPalace => {
            THE_CRYSTAL_PALACE_GOLD_COST
                + THE_CRYSTAL_PALACE_WOOD_COST
                + THE_CRYSTAL_PALACE_STONE_COST
                + THE_CRYSTAL_PALACE_IRON_COST
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

fn place_wood_workshop_system(
    mut commands: Commands,
    mut events: EventReader<PlaceWoodWorkshopEvent>,
    occupancy: Res<TileOccupancy>,
    mut claims: ResMut<WoodWorkshopPlacementClaims>,
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
                WoodWorkshop,
                anchor,
                BuildingHealth::default(),
                WoodWorkshopBuildState {
                    build_ticks_remaining: seconds_to_ticks(WOOD_WORKSHOP_BUILD_TIME_SECONDS),
                    completed: false,
                },
                WoodWorkshopEconomy::default(),
                WoodWorkshopFootprint::default(),
                WoodWorkshopState {
                    connected_to_grid: true,
                    destroyed: false,
                },
                WoodWorkshopResearchState::default(),
                WoodWorkshopUnlocked::default(),
            ))
            .id();

        claims.claims.insert(entity, anchor);
    }
}

fn set_wood_workshop_grid_connection_system(
    mut events: EventReader<SetWoodWorkshopGridConnectionEvent>,
    mut workshops: Query<&mut WoodWorkshopState, With<WoodWorkshop>>,
) {
    for ev in events.read() {
        let Ok(mut state) = workshops.get_mut(ev.entity) else {
            continue;
        };
        state.connected_to_grid = ev.connected;
    }
}

fn wood_workshop_build_tick_system(
    mut workshops: Query<(&mut WoodWorkshopBuildState, &mut WoodWorkshopState, &mut WoodWorkshopResearchState), With<WoodWorkshop>>,
) {
    for (mut build, mut state, mut research) in &mut workshops {
        if !build.completed {
            if build.build_ticks_remaining > 0 {
                build.build_ticks_remaining -= 1;
            }

            if build.build_ticks_remaining <= 0 {
                build.build_ticks_remaining = 0;
                build.completed = true;
            }
        }

        if state.destroyed {
            research.active = None;
            research.ticks_remaining = 0;
        }
    }
}

fn wood_workshop_start_research_system(
    mut events: EventReader<StartWoodWorkshopResearchEvent>,
    mut workshops: Query<
        (
            &WoodWorkshopBuildState,
            &WoodWorkshopState,
            &mut WoodWorkshopResearchState,
            &WoodWorkshopUnlocked,
        ),
        With<WoodWorkshop>,
    >,
) {
    for ev in events.read() {
        let Ok((build, state, mut research, unlocked)) = workshops.get_mut(ev.entity) else {
            continue;
        };

        if !build.completed || state.destroyed || !state.connected_to_grid {
            continue;
        }

        if research.active.is_some() {
            continue;
        }

        let already_unlocked = match ev.option {
            WoodWorkshopResearchOption::Cottage => unlocked.cottage,
            WoodWorkshopResearchOption::Farm => unlocked.farm,
            WoodWorkshopResearchOption::StoneWorkshop => unlocked.stone_workshop,
            WoodWorkshopResearchOption::Market => unlocked.market,
            WoodWorkshopResearchOption::LookoutTower => unlocked.lookout_tower,
            WoodWorkshopResearchOption::GreatBallista => unlocked.great_ballista,
            WoodWorkshopResearchOption::StakesTrap => unlocked.stakes_trap,
            WoodWorkshopResearchOption::Sniper => unlocked.sniper,
            WoodWorkshopResearchOption::Inn => unlocked.inn,
            WoodWorkshopResearchOption::TheSilentBeholder => unlocked.the_silent_beholder,
            WoodWorkshopResearchOption::TheCrystalPalace => unlocked.the_crystal_palace,
        };

        if already_unlocked {
            continue;
        }

        research.active = Some(ev.option);
        research.ticks_remaining = seconds_to_ticks(research_duration_seconds(ev.option));
    }
}

fn wood_workshop_research_tick_and_unlock_system(
    mut workshops: Query<(&WoodWorkshopState, &mut WoodWorkshopResearchState, &mut WoodWorkshopUnlocked), With<WoodWorkshop>>,
) {
    for (state, mut research, mut unlocked) in &mut workshops {
        if !state.connected_to_grid || state.destroyed {
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

        match active {
            WoodWorkshopResearchOption::Cottage => unlocked.cottage = true,
            WoodWorkshopResearchOption::Farm => unlocked.farm = true,
            WoodWorkshopResearchOption::StoneWorkshop => unlocked.stone_workshop = true,
            WoodWorkshopResearchOption::Market => unlocked.market = true,
            WoodWorkshopResearchOption::LookoutTower => unlocked.lookout_tower = true,
            WoodWorkshopResearchOption::GreatBallista => unlocked.great_ballista = true,
            WoodWorkshopResearchOption::StakesTrap => unlocked.stakes_trap = true,
            WoodWorkshopResearchOption::Sniper => unlocked.sniper = true,
            WoodWorkshopResearchOption::Inn => unlocked.inn = true,
            WoodWorkshopResearchOption::TheSilentBeholder => unlocked.the_silent_beholder = true,
            WoodWorkshopResearchOption::TheCrystalPalace => unlocked.the_crystal_palace = true,
        }
    }
}

fn wood_workshop_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    workshops: Query<
        (
            Entity,
            &BuildingAnchor,
            &BuildingHealth,
            &WoodWorkshopBuildState,
            &WoodWorkshopEconomy,
            &WoodWorkshopFootprint,
            &WoodWorkshopState,
            &WoodWorkshopResearchState,
            &WoodWorkshopUnlocked,
        ),
        With<WoodWorkshop>,
    >,
    claims: Res<WoodWorkshopPlacementClaims>,
    occupancy: Res<TileOccupancy>,
) {
    for (entity, anchor, hp, build, eco, footprint, state, research, unlocked) in &workshops {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);

        checksum.accumulate(hp.hp.to_bits() as u64);
        checksum.accumulate(hp.defenses_life.to_bits() as u64);

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

        let active_research_id = match research.active {
            None => 0u64,
            Some(WoodWorkshopResearchOption::Cottage) => 1,
            Some(WoodWorkshopResearchOption::Farm) => 2,
            Some(WoodWorkshopResearchOption::StoneWorkshop) => 3,
            Some(WoodWorkshopResearchOption::Market) => 4,
            Some(WoodWorkshopResearchOption::LookoutTower) => 5,
            Some(WoodWorkshopResearchOption::GreatBallista) => 6,
            Some(WoodWorkshopResearchOption::StakesTrap) => 7,
            Some(WoodWorkshopResearchOption::Sniper) => 8,
            Some(WoodWorkshopResearchOption::Inn) => 9,
            Some(WoodWorkshopResearchOption::TheSilentBeholder) => 10,
            Some(WoodWorkshopResearchOption::TheCrystalPalace) => 11,
        };

        checksum.accumulate(active_research_id);
        checksum.accumulate(research.ticks_remaining as u64);
        checksum.accumulate(research.active.map(research_total_cost).unwrap_or(0) as u64);

        checksum.accumulate(u64::from(unlocked.cottage));
        checksum.accumulate(u64::from(unlocked.farm));
        checksum.accumulate(u64::from(unlocked.stone_workshop));
        checksum.accumulate(u64::from(unlocked.market));
        checksum.accumulate(u64::from(unlocked.lookout_tower));
        checksum.accumulate(u64::from(unlocked.great_ballista));
        checksum.accumulate(u64::from(unlocked.stakes_trap));
        checksum.accumulate(u64::from(unlocked.sniper));
        checksum.accumulate(u64::from(unlocked.inn));
        checksum.accumulate(u64::from(unlocked.the_silent_beholder));
        checksum.accumulate(u64::from(unlocked.the_crystal_palace));
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

pub struct WoodWorkshopPlugin;

impl Plugin for WoodWorkshopPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileOccupancy>()
            .init_resource::<WoodWorkshopPlacementClaims>()
            .add_event::<SetTileBlockedEvent>()
            .add_event::<PlaceWoodWorkshopEvent>()
            .add_event::<StartWoodWorkshopResearchEvent>()
            .add_event::<SetWoodWorkshopGridConnectionEvent>()
            .add_systems(
                FixedUpdate,
                (
                    apply_tile_block_events_system,
                    place_wood_workshop_system,
                    set_wood_workshop_grid_connection_system,
                    wood_workshop_build_tick_system,
                    wood_workshop_start_research_system,
                    wood_workshop_research_tick_and_unlock_system,
                    wood_workshop_checksum_system,
                )
                    .chain(),
            );
    }
}