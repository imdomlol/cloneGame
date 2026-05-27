// Sources: vault/buildings/warehouse.md

use std::collections::BTreeMap;

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::SimChecksumState;

const WAREHOUSE_HP: I32F32 = I32F32::lit("1000");
const WAREHOUSE_DEFENSES_LIFE: I32F32 = I32F32::lit("250");
const WAREHOUSE_WATCH_RANGE: I32F32 = I32F32::lit("7");

const WAREHOUSE_ENERGY_COST: i32 = 8;
const WAREHOUSE_WOOD_COST: i32 = 40;
const WAREHOUSE_STONE_COST: i32 = 40;
const WAREHOUSE_IRON_COST: i32 = 0;
const WAREHOUSE_OIL_COST: i32 = 0;
const WAREHOUSE_GOLD_COST: i32 = 400;
const WAREHOUSE_WORKER_COST: i32 = 10;
const WAREHOUSE_GOLD_MAINTENANCE: i32 = 30;
const WAREHOUSE_BUILD_TIME_SECONDS: i32 = 75;

const WAREHOUSE_SIZE_TILES: i32 = 4;
const WAREHOUSE_ZONE_SIZE_TILES: i32 = 28;
const WAREHOUSE_RESOURCE_STORAGE_BONUS: i32 = 50;
const WAREHOUSE_GOLD_STORAGE_BONUS: i32 = 2000;
const WAREHOUSE_PRODUCTION_BONUS_PERCENT: i32 = 20;

const SIM_HZ: i32 = 25;

#[derive(Component, Default)]
pub struct Warehouse;

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
            hp: WAREHOUSE_HP,
            defenses_life: WAREHOUSE_DEFENSES_LIFE,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct WarehouseBuildState {
    pub build_ticks_remaining: i32,
    pub completed: bool,
}

impl Default for WarehouseBuildState {
    fn default() -> Self {
        Self {
            build_ticks_remaining: build_seconds_to_ticks(WAREHOUSE_BUILD_TIME_SECONDS),
            completed: false,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct WarehouseEconomy {
    pub energy_cost: i32,
    pub wood_cost: i32,
    pub stone_cost: i32,
    pub iron_cost: i32,
    pub oil_cost: i32,
    pub gold_cost: i32,
    pub worker_cost: i32,
    pub gold_maintenance: i32,
}

impl Default for WarehouseEconomy {
    fn default() -> Self {
        Self {
            energy_cost: WAREHOUSE_ENERGY_COST,
            wood_cost: WAREHOUSE_WOOD_COST,
            stone_cost: WAREHOUSE_STONE_COST,
            iron_cost: WAREHOUSE_IRON_COST,
            oil_cost: WAREHOUSE_OIL_COST,
            gold_cost: WAREHOUSE_GOLD_COST,
            worker_cost: WAREHOUSE_WORKER_COST,
            gold_maintenance: WAREHOUSE_GOLD_MAINTENANCE,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct WarehouseFootprint {
    pub size_tiles: i32,
    pub watch_range: I32F32,
    pub zone_size_tiles: i32,
}

impl Default for WarehouseFootprint {
    fn default() -> Self {
        Self {
            size_tiles: WAREHOUSE_SIZE_TILES,
            watch_range: WAREHOUSE_WATCH_RANGE,
            zone_size_tiles: WAREHOUSE_ZONE_SIZE_TILES,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct WarehouseStorageBonus {
    pub resource_storage: i32,
    pub gold_storage: i32,
}

impl Default for WarehouseStorageBonus {
    fn default() -> Self {
        Self {
            resource_storage: WAREHOUSE_RESOURCE_STORAGE_BONUS,
            gold_storage: WAREHOUSE_GOLD_STORAGE_BONUS,
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct WarehousePower {
    pub connected: bool,
}

#[derive(Component, Clone, Copy)]
pub struct WarehouseProductionBonus {
    pub percent: i32,
}

impl Default for WarehouseProductionBonus {
    fn default() -> Self {
        Self {
            percent: WAREHOUSE_PRODUCTION_BONUS_PERCENT,
        }
    }
}

#[derive(Component, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Default)]
pub enum WarehouseProducerCategory {
    #[default]
    Food,
    Wood,
    Stone,
    Iron,
    Oil,
    GoldQuarry,
    WorkerOutput,
    EnergyOutput,
    HousingGold,
}

impl WarehouseProducerCategory {
    fn is_bonus_eligible(self) -> bool {
        matches!(
            self,
            WarehouseProducerCategory::Food
                | WarehouseProducerCategory::Wood
                | WarehouseProducerCategory::Stone
                | WarehouseProducerCategory::Iron
                | WarehouseProducerCategory::Oil
                | WarehouseProducerCategory::GoldQuarry
        )
    }
}

#[derive(Component, Clone, Copy)]
pub struct WarehouseProducer {
    pub anchor: BuildingAnchor,
    pub category: WarehouseProducerCategory,
    pub base_output: I32F32,
    pub final_output: I32F32,
}

impl Default for WarehouseProducer {
    fn default() -> Self {
        Self {
            anchor: BuildingAnchor { x: 0, y: 0 },
            category: WarehouseProducerCategory::Food,
            base_output: I32F32::ZERO,
            final_output: I32F32::ZERO,
        }
    }
}

#[derive(Resource, Default, Clone)]
pub struct WarehousePlacementClaims {
    pub claims: BTreeMap<Entity, BuildingAnchor>,
}

#[derive(Resource, Default, Clone, Copy)]
pub struct WarehouseGlobalStorageBonuses {
    pub resource_storage: i32,
    pub gold_storage: i32,
}

#[derive(Event, Clone, Copy)]
pub struct PlaceWarehouseEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct SetWarehousePowerEvent {
    pub warehouse_entity: Entity,
    pub connected: bool,
}

#[derive(Event, Clone, Copy)]
pub struct RegisterWarehouseProducerEvent {
    pub producer_entity: Entity,
    pub tile_x: i32,
    pub tile_y: i32,
    pub category: WarehouseProducerCategory,
    pub base_output: I32F32,
}

#[derive(Event, Clone, Copy)]
pub struct SetWarehouseProducerBaseOutputEvent {
    pub producer_entity: Entity,
    pub base_output: I32F32,
}

#[derive(Event, Clone, Copy)]
pub struct UnregisterWarehouseProducerEvent {
    pub producer_entity: Entity,
}

fn build_seconds_to_ticks(seconds: i32) -> i32 {
    seconds * SIM_HZ
}

fn abs_i32(v: i32) -> i32 {
    if v < 0 {
        -v
    } else {
        v
    }
}

fn in_warehouse_zone(warehouse_anchor: BuildingAnchor, target_anchor: BuildingAnchor) -> bool {
    let warehouse_center_x_num = warehouse_anchor.x * 2 + (WAREHOUSE_SIZE_TILES - 1);
    let warehouse_center_y_num = warehouse_anchor.y * 2 + (WAREHOUSE_SIZE_TILES - 1);
    let target_center_x_num = target_anchor.x * 2;
    let target_center_y_num = target_anchor.y * 2;

    let half_zone_num = WAREHOUSE_ZONE_SIZE_TILES;
    abs_i32(warehouse_center_x_num - target_center_x_num) <= half_zone_num
        && abs_i32(warehouse_center_y_num - target_center_y_num) <= half_zone_num
}

fn place_warehouse_system(
    mut commands: Commands,
    mut events: EventReader<PlaceWarehouseEvent>,
    mut claims: ResMut<WarehousePlacementClaims>,
) {
    for ev in events.read() {
        let anchor = BuildingAnchor {
            x: ev.tile_x,
            y: ev.tile_y,
        };

        let entity = commands
            .spawn((
                Warehouse,
                anchor,
                BuildingHealth::default(),
                WarehouseBuildState::default(),
                WarehouseEconomy::default(),
                WarehouseFootprint::default(),
                WarehouseStorageBonus::default(),
                WarehousePower::default(),
                WarehouseProductionBonus::default(),
            ))
            .id();

        claims.claims.insert(entity, anchor);
    }
}

fn set_warehouse_power_system(
    mut events: EventReader<SetWarehousePowerEvent>,
    mut warehouses: Query<&mut WarehousePower, With<Warehouse>>,
) {
    for ev in events.read() {
        let Ok(mut power) = warehouses.get_mut(ev.warehouse_entity) else {
            continue;
        };
        power.connected = ev.connected;
    }
}

fn warehouse_build_tick_system(mut warehouses: Query<&mut WarehouseBuildState, With<Warehouse>>) {
    for mut state in &mut warehouses {
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

fn register_warehouse_producer_system(
    mut commands: Commands,
    mut events: EventReader<RegisterWarehouseProducerEvent>,
) {
    for ev in events.read() {
        commands.entity(ev.producer_entity).insert(WarehouseProducer {
            anchor: BuildingAnchor {
                x: ev.tile_x,
                y: ev.tile_y,
            },
            category: ev.category,
            base_output: ev.base_output,
            final_output: ev.base_output,
        });
    }
}

fn set_warehouse_producer_base_output_system(
    mut events: EventReader<SetWarehouseProducerBaseOutputEvent>,
    mut producers: Query<&mut WarehouseProducer>,
) {
    for ev in events.read() {
        let Ok(mut producer) = producers.get_mut(ev.producer_entity) else {
            continue;
        };
        producer.base_output = ev.base_output;
    }
}

fn unregister_warehouse_producer_system(
    mut commands: Commands,
    mut events: EventReader<UnregisterWarehouseProducerEvent>,
) {
    for ev in events.read() {
        commands.entity(ev.producer_entity).remove::<WarehouseProducer>();
    }
}

fn warehouse_apply_production_bonus_system(
    warehouses: Query<
        (
            &BuildingAnchor,
            &WarehouseBuildState,
            &WarehousePower,
            &WarehouseProductionBonus,
        ),
        With<Warehouse>,
    >,
    mut producers: Query<&mut WarehouseProducer>,
) {
    for mut producer in &mut producers {
        if !producer.category.is_bonus_eligible() {
            producer.final_output = producer.base_output;
            continue;
        }

        let mut has_bonus = false;
        let mut bonus_percent = 0;
        for (warehouse_anchor, build_state, power, bonus) in &warehouses {
            if !build_state.completed || !power.connected {
                continue;
            }

            if in_warehouse_zone(*warehouse_anchor, producer.anchor) {
                has_bonus = true;
                bonus_percent = bonus.percent;
                break;
            }
        }

        if has_bonus {
            let multiplier_numerator = I32F32::from_num(100 + bonus_percent);
            let multiplier_denominator = I32F32::from_num(100);
            producer.final_output = producer.base_output * multiplier_numerator / multiplier_denominator;
        } else {
            producer.final_output = producer.base_output;
        }
    }
}

fn warehouse_global_storage_bonus_system(
    warehouses: Query<(&WarehouseBuildState, &WarehouseStorageBonus), With<Warehouse>>,
    mut storage: ResMut<WarehouseGlobalStorageBonuses>,
) {
    let mut resource_storage = 0;
    let mut gold_storage = 0;

    for (build_state, bonus) in &warehouses {
        if !build_state.completed {
            continue;
        }

        resource_storage += bonus.resource_storage;
        gold_storage += bonus.gold_storage;
    }

    storage.resource_storage = resource_storage;
    storage.gold_storage = gold_storage;
}

fn warehouse_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    warehouses: Query<
        (
            Entity,
            &BuildingAnchor,
            &BuildingHealth,
            &WarehouseBuildState,
            &WarehouseEconomy,
            &WarehouseFootprint,
            &WarehouseStorageBonus,
            &WarehousePower,
            &WarehouseProductionBonus,
        ),
        With<Warehouse>,
    >,
    producers: Query<(Entity, &WarehouseProducer)>,
    claims: Res<WarehousePlacementClaims>,
    storage: Res<WarehouseGlobalStorageBonuses>,
) {
    for (entity, anchor, hp, build, eco, footprint, bonus_storage, power, bonus_prod) in &warehouses {
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
        checksum.accumulate(eco.worker_cost as u64);
        checksum.accumulate(eco.gold_maintenance as u64);

        checksum.accumulate(footprint.size_tiles as u64);
        checksum.accumulate(footprint.watch_range.to_bits() as u64);
        checksum.accumulate(footprint.zone_size_tiles as u64);

        checksum.accumulate(bonus_storage.resource_storage as u64);
        checksum.accumulate(bonus_storage.gold_storage as u64);

        checksum.accumulate(u64::from(power.connected));
        checksum.accumulate(bonus_prod.percent as u64);
    }

    for (entity, producer) in &producers {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(producer.anchor.x as u64);
        checksum.accumulate(producer.anchor.y as u64);
        checksum.accumulate(producer.category as u64);
        checksum.accumulate(producer.base_output.to_bits() as u64);
        checksum.accumulate(producer.final_output.to_bits() as u64);
    }

    checksum.accumulate(claims.claims.len() as u64);
    for (entity, anchor) in &claims.claims {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);
    }

    checksum.accumulate(storage.resource_storage as u64);
    checksum.accumulate(storage.gold_storage as u64);
}

pub struct WarehousePlugin;

impl Plugin for WarehousePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WarehousePlacementClaims>()
            .init_resource::<WarehouseGlobalStorageBonuses>()
            .add_event::<PlaceWarehouseEvent>()
            .add_event::<SetWarehousePowerEvent>()
            .add_event::<RegisterWarehouseProducerEvent>()
            .add_event::<SetWarehouseProducerBaseOutputEvent>()
            .add_event::<UnregisterWarehouseProducerEvent>()
            .add_systems(
                FixedUpdate,
                (
                    place_warehouse_system,
                    set_warehouse_power_system,
                    warehouse_build_tick_system,
                    register_warehouse_producer_system,
                    set_warehouse_producer_base_output_system,
                    unregister_warehouse_producer_system,
                    warehouse_apply_production_bonus_system,
                    warehouse_global_storage_bonus_system,
                    warehouse_checksum_system,
                )
                    .chain(),
            );
    }
}