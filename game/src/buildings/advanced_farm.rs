// Sources: vault/buildings/advanced_farm.md, vault/buildings/farm.md

use std::collections::BTreeMap;

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::buildings::farm::{Farm, FarmAnchor, FarmBuildState, FarmDefenseLife, FarmEconomy, FarmFootprint, FarmPlacementClaims, FarmPlots, FarmProduction};
use crate::sim::{Health, SimChecksumState};

const ADVANCED_FARM_HP: I32F32 = I32F32::lit("500");
const ADVANCED_FARM_DEFENSES_LIFE: I32F32 = I32F32::lit("125");
const ADVANCED_FARM_WATCH_RANGE: I32F32 = I32F32::lit("7");
const ADVANCED_FARM_BUILD_TIME_SECONDS: i32 = 60;
const ADVANCED_FARM_UPGRADE_TIME_SECONDS: i32 = 26;
const ADVANCED_FARM_SIZE_TILES: i32 = 2;
const ADVANCED_FARM_PLOT_RADIUS_TILES: i32 = 2;
const ADVANCED_FARM_MIN_SPACING_TILES: i32 = 4;
const ADVANCED_FARM_FOOD_PER_PLOT: I32F32 = I32F32::lit("4");
const ADVANCED_FARM_PFOOD_MIN: I32F32 = I32F32::lit("4");
const ADVANCED_FARM_PFOOD_MAX: I32F32 = I32F32::lit("128");
const ADVANCED_FARM_ENERGY_COST: i32 = 30;
const ADVANCED_FARM_WOOD_COST: i32 = 30;
const ADVANCED_FARM_STONE_COST: i32 = 20;
const ADVANCED_FARM_IRON_COST: i32 = 20;
const ADVANCED_FARM_OIL_COST: i32 = 20;
const ADVANCED_FARM_GOLD_COST: i32 = 1200;
const ADVANCED_FARM_PCOLONISTS: i32 = 24;
const SIM_HZ: i32 = 25;

#[derive(Component, Default)]
pub struct AdvancedFarm;

#[derive(Component, Clone, Copy)]
pub struct AdvancedFarmDefenseLife {
    pub current: I32F32,
    pub max: I32F32,
}

impl AdvancedFarmDefenseLife {
    pub fn full(max: I32F32) -> Self {
        Self { current: max, max }
    }
}

impl Default for AdvancedFarmDefenseLife {
    fn default() -> Self {
        Self::full(ADVANCED_FARM_DEFENSES_LIFE)
    }
}

#[derive(Component, Clone, Copy)]
pub struct AdvancedFarmBuildState {
    pub build_ticks_remaining: i32,
    pub upgrading_from_farm: bool,
    pub completed: bool,
}

#[derive(Component, Clone, Copy)]
pub struct AdvancedFarmEconomy {
    pub energy_cost: i32,
    pub wood_cost: i32,
    pub stone_cost: i32,
    pub iron_cost: i32,
    pub oil_cost: i32,
    pub gold_cost: i32,
    pub pcolonists: i32,
}

impl Default for AdvancedFarmEconomy {
    fn default() -> Self {
        Self {
            energy_cost: ADVANCED_FARM_ENERGY_COST,
            wood_cost: ADVANCED_FARM_WOOD_COST,
            stone_cost: ADVANCED_FARM_STONE_COST,
            iron_cost: ADVANCED_FARM_IRON_COST,
            oil_cost: ADVANCED_FARM_OIL_COST,
            gold_cost: ADVANCED_FARM_GOLD_COST,
            pcolonists: ADVANCED_FARM_PCOLONISTS,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct AdvancedFarmFootprint {
    pub size_tiles: i32,
    pub watch_range: I32F32,
    pub plot_radius_tiles: i32,
    pub min_spacing_tiles: i32,
}

impl Default for AdvancedFarmFootprint {
    fn default() -> Self {
        Self {
            size_tiles: ADVANCED_FARM_SIZE_TILES,
            watch_range: ADVANCED_FARM_WATCH_RANGE,
            plot_radius_tiles: ADVANCED_FARM_PLOT_RADIUS_TILES,
            min_spacing_tiles: ADVANCED_FARM_MIN_SPACING_TILES,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct AdvancedFarmProduction {
    pub food: I32F32,
    pub min_food: I32F32,
    pub max_food: I32F32,
}

impl Default for AdvancedFarmProduction {
    fn default() -> Self {
        Self {
            food: I32F32::ZERO,
            min_food: ADVANCED_FARM_PFOOD_MIN,
            max_food: ADVANCED_FARM_PFOOD_MAX,
        }
    }
}

#[derive(Resource, Default, Clone)]
pub struct AdvancedFarmPlacementClaims {
    pub claims: BTreeMap<Entity, FarmAnchor>,
}

#[derive(Event, Clone, Copy)]
pub struct PlaceAdvancedFarmEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct AdvancedFarmPlacementRejectedEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct UpgradeFarmToAdvancedFarmEvent {
    pub farm_entity: Entity,
}

#[derive(Event, Clone, Copy)]
pub struct UpgradeFarmToAdvancedFarmRejectedEvent {
    pub farm_entity: Entity,
}

#[derive(Event, Clone, Copy)]
pub struct DamageAdvancedFarmEvent {
    pub farm_entity: Entity,
    pub damage: I32F32,
}

fn seconds_to_ticks(seconds: i32) -> i32 {
    seconds * SIM_HZ
}

fn max_axis_delta(a: FarmAnchor, b: FarmAnchor) -> i32 {
    let dx = (a.x - b.x).abs();
    let dy = (a.y - b.y).abs();
    dx.max(dy)
}

fn is_anchor_free(anchor: FarmAnchor, farm_claims: &FarmPlacementClaims, advanced_claims: &AdvancedFarmPlacementClaims) -> bool {
    for existing in farm_claims.claims.values() {
        if existing.x == anchor.x && existing.y == anchor.y {
            return false;
        }
        if max_axis_delta(anchor, *existing) < ADVANCED_FARM_MIN_SPACING_TILES {
            return false;
        }
    }

    for existing in advanced_claims.claims.values() {
        if existing.x == anchor.x && existing.y == anchor.y {
            return false;
        }
        if max_axis_delta(anchor, *existing) < ADVANCED_FARM_MIN_SPACING_TILES {
            return false;
        }
    }

    true
}

fn place_advanced_farm_system(
    mut commands: Commands,
    mut events: EventReader<PlaceAdvancedFarmEvent>,
    mut rejected: EventWriter<AdvancedFarmPlacementRejectedEvent>,
    farm_claims: Res<FarmPlacementClaims>,
    mut advanced_claims: ResMut<AdvancedFarmPlacementClaims>,
) {
    for ev in events.read() {
        let anchor = FarmAnchor {
            x: ev.tile_x,
            y: ev.tile_y,
        };

        if !is_anchor_free(anchor, &farm_claims, &advanced_claims) {
            rejected.send(AdvancedFarmPlacementRejectedEvent {
                tile_x: ev.tile_x,
                tile_y: ev.tile_y,
            });
            continue;
        }

        let entity = commands
            .spawn((
                AdvancedFarm,
                anchor,
                Health::full(ADVANCED_FARM_HP),
                AdvancedFarmDefenseLife::default(),
                AdvancedFarmBuildState {
                    build_ticks_remaining: seconds_to_ticks(ADVANCED_FARM_BUILD_TIME_SECONDS),
                    upgrading_from_farm: false,
                    completed: false,
                },
                AdvancedFarmEconomy::default(),
                AdvancedFarmFootprint::default(),
                AdvancedFarmProduction::default(),
            ))
            .id();

        advanced_claims.claims.insert(entity, anchor);
    }
}

fn upgrade_farm_to_advanced_farm_system(
    mut commands: Commands,
    mut events: EventReader<UpgradeFarmToAdvancedFarmEvent>,
    mut rejected: EventWriter<UpgradeFarmToAdvancedFarmRejectedEvent>,
    farms: Query<(Entity, &FarmAnchor), With<Farm>>,
    mut farm_claims: ResMut<FarmPlacementClaims>,
    mut advanced_claims: ResMut<AdvancedFarmPlacementClaims>,
) {
    for ev in events.read() {
        let Ok((farm_entity, anchor)) = farms.get(ev.farm_entity) else {
            rejected.send(UpgradeFarmToAdvancedFarmRejectedEvent {
                farm_entity: ev.farm_entity,
            });
            continue;
        };

        for (other_entity, other_anchor) in &farm_claims.claims {
            if *other_entity == farm_entity {
                continue;
            }
            if max_axis_delta(*anchor, *other_anchor) < ADVANCED_FARM_MIN_SPACING_TILES {
                rejected.send(UpgradeFarmToAdvancedFarmRejectedEvent {
                    farm_entity: ev.farm_entity,
                });
                continue;
            }
        }

        commands.entity(farm_entity).remove::<Farm>();
        commands.entity(farm_entity).remove::<(
            FarmBuildState,
            FarmEconomy,
            FarmFootprint,
            FarmProduction,
            FarmDefenseLife,
            FarmPlots,
        )>();
        commands.entity(farm_entity).insert((
            AdvancedFarm,
            Health::full(ADVANCED_FARM_HP),
            AdvancedFarmDefenseLife::default(),
            AdvancedFarmBuildState {
                build_ticks_remaining: seconds_to_ticks(ADVANCED_FARM_UPGRADE_TIME_SECONDS),
                upgrading_from_farm: true,
                completed: false,
            },
            AdvancedFarmEconomy::default(),
            AdvancedFarmFootprint::default(),
            AdvancedFarmProduction::default(),
        ));

        farm_claims.claims.remove(&farm_entity);
        advanced_claims.claims.insert(farm_entity, *anchor);
    }
}

fn advanced_farm_build_tick_system(mut farms: Query<&mut AdvancedFarmBuildState, With<AdvancedFarm>>) {
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

fn refresh_advanced_farm_production_system(
    mut farms: Query<(&AdvancedFarmBuildState, &mut AdvancedFarmProduction), With<AdvancedFarm>>,
) {
    for (build, mut production) in &mut farms {
        if !build.completed {
            production.food = I32F32::ZERO;
            continue;
        }
        let plots_per_axis = ADVANCED_FARM_SIZE_TILES + ADVANCED_FARM_PLOT_RADIUS_TILES * 2;
        let total_plots = plots_per_axis * plots_per_axis;
        let raw_food = ADVANCED_FARM_FOOD_PER_PLOT * I32F32::from_num(total_plots);
        production.food = raw_food.min(production.max_food).max(production.min_food);
    }
}

fn damage_advanced_farm_system(
    mut commands: Commands,
    mut events: EventReader<DamageAdvancedFarmEvent>,
    mut farms: Query<(Entity, &mut Health, &mut AdvancedFarmDefenseLife), With<AdvancedFarm>>,
    mut claims: ResMut<AdvancedFarmPlacementClaims>,
) {
    for ev in events.read() {
        let Ok((entity, mut health, mut defense)) = farms.get_mut(ev.farm_entity) else {
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
            claims.claims.remove(&entity);
            commands.entity(entity).despawn();
        }
    }
}

fn advanced_farm_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    farms: Query<
        (
            Entity,
            &FarmAnchor,
            &Health,
            &AdvancedFarmDefenseLife,
            &AdvancedFarmBuildState,
            &AdvancedFarmEconomy,
            &AdvancedFarmFootprint,
            &AdvancedFarmProduction,
        ),
        With<AdvancedFarm>,
    >,
    claims: Res<AdvancedFarmPlacementClaims>,
) {
    for (entity, anchor, health, defense, build, economy, footprint, production) in &farms {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);

        checksum.accumulate(health.current.to_bits() as u64);
        checksum.accumulate(health.max.to_bits() as u64);
        checksum.accumulate(defense.current.to_bits() as u64);
        checksum.accumulate(defense.max.to_bits() as u64);

        checksum.accumulate(build.build_ticks_remaining as u64);
        checksum.accumulate(u64::from(build.upgrading_from_farm));
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
    }

    checksum.accumulate(claims.claims.len() as u64);
    for (entity, anchor) in &claims.claims {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);
    }
}

pub struct AdvancedFarmPlugin;

impl Plugin for AdvancedFarmPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AdvancedFarmPlacementClaims>()
            .add_event::<PlaceAdvancedFarmEvent>()
            .add_event::<AdvancedFarmPlacementRejectedEvent>()
            .add_event::<UpgradeFarmToAdvancedFarmEvent>()
            .add_event::<UpgradeFarmToAdvancedFarmRejectedEvent>()
            .add_event::<DamageAdvancedFarmEvent>()
            .add_systems(
                FixedUpdate,
                (
                    place_advanced_farm_system,
                    upgrade_farm_to_advanced_farm_system,
                    advanced_farm_build_tick_system,
                    refresh_advanced_farm_production_system,
                    damage_advanced_farm_system,
                    advanced_farm_checksum_system,
                )
                    .chain(),
            );
    }
}