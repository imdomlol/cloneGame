// Sources: vault/buildings/inn.md

use std::collections::BTreeMap;

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::SimChecksumState;

const INN_HP: I32F32 = I32F32::lit("1200");
const INN_DEFENSES_LIFE: I32F32 = I32F32::lit("300");
const INN_WATCH_RANGE: I32F32 = I32F32::lit("9");
const INN_ENERGY_COST: i32 = 50;
const INN_WOOD_COST: i32 = 100;
const INN_STONE_COST: i32 = 100;
const INN_GOLD_COST: i32 = 2000;
const INN_BUILD_TIME_SECONDS: i32 = 0;
const INN_SIZE_TILES: i32 = 4;
const INN_WORKERS_GRANTED: i32 = 100;
const INN_FOOD_REQUIRED: i32 = 100;
const INN_COLONISTS_REQUIRED_TO_BUILD: i32 = 200;
const INN_GOLD_GENERATION_BONUS_PERCENT: i32 = 10;
const INN_EFFECT_AREA_WIDTH: i32 = 52;
const INN_EFFECT_AREA_HEIGHT: i32 = 52;
const INN_EFFECT_RADIUS_BLOCKS: i32 = 24;
const INN_UPKEEP_WOOD: i32 = 30;
const INN_UPKEEP_INTERVAL_HOURS: i32 = 8;
const INN_MERCENARY_REFRESH_DAYS: i32 = 5;
const INN_PRESTIGE_THRESHOLD_LUCIFER: i32 = 55;
const INN_PRESTIGE_THRESHOLD_THANATOS: i32 = 60;
const INN_PRESTIGE_THRESHOLD_TITAN: i32 = 65;
const SIM_HZ: i32 = 25;
const SECONDS_PER_HOUR: i32 = 3600;
const HOURS_PER_DAY: i32 = 24;

#[derive(Component, Default)]
pub struct Inn;

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
            hp: INN_HP,
            defenses_life: INN_DEFENSES_LIFE,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct InnBuildState {
    pub build_ticks_remaining: i32,
    pub completed: bool,
}

impl Default for InnBuildState {
    fn default() -> Self {
        let ticks = seconds_to_ticks(INN_BUILD_TIME_SECONDS);
        Self {
            build_ticks_remaining: ticks,
            completed: ticks == 0,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct InnEconomy {
    pub energy_cost: i32,
    pub wood_cost: i32,
    pub stone_cost: i32,
    pub gold_cost: i32,
    pub food_required: i32,
    pub workers_granted: i32,
}

impl Default for InnEconomy {
    fn default() -> Self {
        Self {
            energy_cost: INN_ENERGY_COST,
            wood_cost: INN_WOOD_COST,
            stone_cost: INN_STONE_COST,
            gold_cost: INN_GOLD_COST,
            food_required: INN_FOOD_REQUIRED,
            workers_granted: INN_WORKERS_GRANTED,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct InnFootprint {
    pub size_tiles: i32,
    pub watch_range: I32F32,
}

impl Default for InnFootprint {
    fn default() -> Self {
        Self {
            size_tiles: INN_SIZE_TILES,
            watch_range: INN_WATCH_RANGE,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct InnInfluenceArea {
    pub width_tiles: i32,
    pub height_tiles: i32,
    pub radius_blocks: i32,
    pub gold_bonus_percent: i32,
}

impl Default for InnInfluenceArea {
    fn default() -> Self {
        Self {
            width_tiles: INN_EFFECT_AREA_WIDTH,
            height_tiles: INN_EFFECT_AREA_HEIGHT,
            radius_blocks: INN_EFFECT_RADIUS_BLOCKS,
            gold_bonus_percent: INN_GOLD_GENERATION_BONUS_PERCENT,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct InnUpkeepState {
    pub ticks_until_upkeep: i32,
    pub upkeep_ticks_interval: i32,
    pub wood_per_upkeep: i32,
    pub total_wood_consumed: i32,
}

impl Default for InnUpkeepState {
    fn default() -> Self {
        let interval = hours_to_ticks(INN_UPKEEP_INTERVAL_HOURS);
        Self {
            ticks_until_upkeep: interval,
            upkeep_ticks_interval: interval,
            wood_per_upkeep: INN_UPKEEP_WOOD,
            total_wood_consumed: 0,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct InnMercenaryState {
    pub ticks_until_refresh: i32,
    pub refresh_ticks_interval: i32,
    pub lucifer_unlocked: bool,
    pub thanatos_unlocked: bool,
    pub titan_unlocked: bool,
}

impl Default for InnMercenaryState {
    fn default() -> Self {
        let interval = days_to_ticks(INN_MERCENARY_REFRESH_DAYS);
        Self {
            ticks_until_refresh: interval,
            refresh_ticks_interval: interval,
            lucifer_unlocked: false,
            thanatos_unlocked: false,
            titan_unlocked: false,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct InnOwnership {
    pub colony_id: i32,
}

#[derive(Clone, Copy, Default)]
pub struct ColonyInnState {
    pub colonists: i32,
    pub inn_count: i32,
    pub prestige_percent: i32,
    pub campaign_mode: bool,
    pub disabled_dwellings: i32,
    pub inn_colonists_count_toward_objectives: bool,
    pub wood_workshop_required: bool,
}

#[derive(Resource, Default, Clone)]
pub struct ColonyInnLedger {
    pub by_colony: BTreeMap<i32, ColonyInnState>,
}

#[derive(Event, Clone, Copy)]
pub struct SetColonyInnStateEvent {
    pub colony_id: i32,
    pub colonists: i32,
    pub prestige_percent: i32,
    pub campaign_mode: bool,
    pub disabled_dwellings: i32,
}

#[derive(Event, Clone, Copy)]
pub struct PlaceInnEvent {
    pub colony_id: i32,
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct InnPlacementRejectedEvent {
    pub colony_id: i32,
    pub tile_x: i32,
    pub tile_y: i32,
}

fn seconds_to_ticks(seconds: i32) -> i32 {
    seconds * SIM_HZ
}

fn hours_to_ticks(hours: i32) -> i32 {
    seconds_to_ticks(hours * SECONDS_PER_HOUR)
}

fn days_to_ticks(days: i32) -> i32 {
    hours_to_ticks(days * HOURS_PER_DAY)
}

fn apply_colony_inn_state_events_system(
    mut events: EventReader<SetColonyInnStateEvent>,
    mut ledger: ResMut<ColonyInnLedger>,
) {
    for ev in events.read() {
        let entry = ledger.by_colony.entry(ev.colony_id).or_default();
        entry.colonists = ev.colonists;
        entry.prestige_percent = ev.prestige_percent;
        entry.campaign_mode = ev.campaign_mode;
        entry.disabled_dwellings = ev.disabled_dwellings;
        entry.inn_colonists_count_toward_objectives = !ev.campaign_mode;
        entry.wood_workshop_required = !ev.campaign_mode;
    }
}

fn place_inn_system(
    mut commands: Commands,
    mut events: EventReader<PlaceInnEvent>,
    mut rejected: EventWriter<InnPlacementRejectedEvent>,
    mut ledger: ResMut<ColonyInnLedger>,
) {
    for ev in events.read() {
        let colony_state = ledger.by_colony.entry(ev.colony_id).or_default();

        if colony_state.colonists < INN_COLONISTS_REQUIRED_TO_BUILD || colony_state.inn_count >= 1 {
            rejected.send(InnPlacementRejectedEvent {
                colony_id: ev.colony_id,
                tile_x: ev.tile_x,
                tile_y: ev.tile_y,
            });
            continue;
        }

        commands.spawn((
            Inn,
            BuildingAnchor {
                x: ev.tile_x,
                y: ev.tile_y,
            },
            BuildingHealth::default(),
            InnBuildState::default(),
            InnEconomy::default(),
            InnFootprint::default(),
            InnInfluenceArea::default(),
            InnUpkeepState::default(),
            InnMercenaryState::default(),
            InnOwnership {
                colony_id: ev.colony_id,
            },
        ));

        colony_state.inn_count += 1;
    }
}

fn inn_build_tick_system(mut query: Query<&mut InnBuildState, With<Inn>>) {
    for mut state in &mut query {
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

fn inn_upkeep_system(mut query: Query<(&InnBuildState, &mut InnUpkeepState), With<Inn>>) {
    for (build, mut upkeep) in &mut query {
        if !build.completed {
            continue;
        }

        upkeep.ticks_until_upkeep -= 1;
        if upkeep.ticks_until_upkeep <= 0 {
            upkeep.total_wood_consumed += upkeep.wood_per_upkeep;
            upkeep.ticks_until_upkeep = upkeep.upkeep_ticks_interval;
        }
    }
}

fn inn_mercenary_refresh_and_unlocks_system(
    mut query: Query<(&InnBuildState, &InnOwnership, &mut InnMercenaryState), With<Inn>>,
    ledger: Res<ColonyInnLedger>,
) {
    for (build, ownership, mut mercenary) in &mut query {
        if !build.completed {
            continue;
        }

        let prestige = ledger
            .by_colony
            .get(&ownership.colony_id)
            .map(|s| s.prestige_percent)
            .unwrap_or(0);

        mercenary.lucifer_unlocked = prestige >= INN_PRESTIGE_THRESHOLD_LUCIFER;
        mercenary.thanatos_unlocked = prestige >= INN_PRESTIGE_THRESHOLD_THANATOS;
        mercenary.titan_unlocked = prestige >= INN_PRESTIGE_THRESHOLD_TITAN;

        mercenary.ticks_until_refresh -= 1;
        if mercenary.ticks_until_refresh <= 0 {
            mercenary.ticks_until_refresh = mercenary.refresh_ticks_interval;
        }
    }
}

fn inn_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    inns: Query<
        (
            Entity,
            &BuildingAnchor,
            &BuildingHealth,
            &InnBuildState,
            &InnEconomy,
            &InnFootprint,
            &InnInfluenceArea,
            &InnUpkeepState,
            &InnMercenaryState,
            &InnOwnership,
        ),
        With<Inn>,
    >,
    ledger: Res<ColonyInnLedger>,
) {
    for (entity, anchor, hp, build, eco, footprint, influence, upkeep, mercenary, ownership) in &inns {
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
        checksum.accumulate(eco.gold_cost as u64);
        checksum.accumulate(eco.food_required as u64);
        checksum.accumulate(eco.workers_granted as u64);

        checksum.accumulate(footprint.size_tiles as u64);
        checksum.accumulate(footprint.watch_range.to_bits() as u64);

        checksum.accumulate(influence.width_tiles as u64);
        checksum.accumulate(influence.height_tiles as u64);
        checksum.accumulate(influence.radius_blocks as u64);
        checksum.accumulate(influence.gold_bonus_percent as u64);

        checksum.accumulate(upkeep.ticks_until_upkeep as u64);
        checksum.accumulate(upkeep.upkeep_ticks_interval as u64);
        checksum.accumulate(upkeep.wood_per_upkeep as u64);
        checksum.accumulate(upkeep.total_wood_consumed as u64);

        checksum.accumulate(mercenary.ticks_until_refresh as u64);
        checksum.accumulate(mercenary.refresh_ticks_interval as u64);
        checksum.accumulate(u64::from(mercenary.lucifer_unlocked));
        checksum.accumulate(u64::from(mercenary.thanatos_unlocked));
        checksum.accumulate(u64::from(mercenary.titan_unlocked));

        checksum.accumulate(ownership.colony_id as u64);
    }

    checksum.accumulate(ledger.by_colony.len() as u64);
    for (colony_id, state) in &ledger.by_colony {
        checksum.accumulate(*colony_id as u64);
        checksum.accumulate(state.colonists as u64);
        checksum.accumulate(state.inn_count as u64);
        checksum.accumulate(state.prestige_percent as u64);
        checksum.accumulate(u64::from(state.campaign_mode));
        checksum.accumulate(state.disabled_dwellings as u64);
        checksum.accumulate(u64::from(state.inn_colonists_count_toward_objectives));
        checksum.accumulate(u64::from(state.wood_workshop_required));
    }
}

pub struct InnPlugin;

impl Plugin for InnPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ColonyInnLedger>()
            .add_event::<SetColonyInnStateEvent>()
            .add_event::<PlaceInnEvent>()
            .add_event::<InnPlacementRejectedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    apply_colony_inn_state_events_system,
                    place_inn_system,
                    inn_build_tick_system,
                    inn_upkeep_system,
                    inn_mercenary_refresh_and_unlocks_system,
                    inn_checksum_system,
                )
                    .chain(),
            );
    }
}