// Sources: vault/buildings/wood_tower.md, vault/buildings/stone_tower.md

use std::collections::BTreeMap;

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{Health, SimChecksumState};

const WOOD_TOWER_HP: I32F32 = I32F32::lit("800");
const WOOD_TOWER_DEFENSES_LIFE: I32F32 = I32F32::lit("200");
const WOOD_TOWER_WATCH_RANGE_BONUS: I32F32 = I32F32::lit("3");
const WOOD_TOWER_ATTACK_RANGE_BONUS: I32F32 = I32F32::lit("3");
const WOOD_TOWER_WOOD_COST: i32 = 10;
const WOOD_TOWER_STONE_COST: i32 = 0;
const WOOD_TOWER_IRON_COST: i32 = 0;
const WOOD_TOWER_OIL_COST: i32 = 0;
const WOOD_TOWER_GOLD_COST: i32 = 120;
const WOOD_TOWER_BUILD_TIME_SECONDS: i32 = 39;
const WOOD_TOWER_SIZE_TILES: i32 = 1;
const WOOD_TOWER_GARRISON_CAPACITY: i32 = 4;
const WOOD_TOWER_CONSTRUCTION_HP_PER_SECOND: I32F32 = I32F32::lit("20.5");
const SIM_HZ: i32 = 25;

#[derive(Component, Default)]
pub struct WoodTower;

#[derive(Component, Clone, Copy, Default)]
pub struct BuildingAnchor {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Clone, Copy)]
pub struct WoodTowerCore {
    pub defenses_life: I32F32,
    pub health: Health,
}

impl Default for WoodTowerCore {
    fn default() -> Self {
        Self {
            defenses_life: WOOD_TOWER_DEFENSES_LIFE,
            health: Health {
                current: I32F32::ZERO,
                max: WOOD_TOWER_HP,
            },
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct WoodTowerEconomy {
    pub wood_cost: i32,
    pub stone_cost: i32,
    pub iron_cost: i32,
    pub oil_cost: i32,
    pub gold_cost: i32,
}

impl Default for WoodTowerEconomy {
    fn default() -> Self {
        Self {
            wood_cost: WOOD_TOWER_WOOD_COST,
            stone_cost: WOOD_TOWER_STONE_COST,
            iron_cost: WOOD_TOWER_IRON_COST,
            oil_cost: WOOD_TOWER_OIL_COST,
            gold_cost: WOOD_TOWER_GOLD_COST,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct WoodTowerBuildState {
    pub build_ticks_remaining: i32,
    pub completed: bool,
}

impl Default for WoodTowerBuildState {
    fn default() -> Self {
        Self {
            build_ticks_remaining: WOOD_TOWER_BUILD_TIME_SECONDS * SIM_HZ,
            completed: false,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct WoodTowerFootprint {
    pub size_tiles: i32,
    pub watch_range_bonus: I32F32,
    pub attack_range_bonus: I32F32,
    pub garrison_capacity: i32,
}

impl Default for WoodTowerFootprint {
    fn default() -> Self {
        Self {
            size_tiles: WOOD_TOWER_SIZE_TILES,
            watch_range_bonus: WOOD_TOWER_WATCH_RANGE_BONUS,
            attack_range_bonus: WOOD_TOWER_ATTACK_RANGE_BONUS,
            garrison_capacity: WOOD_TOWER_GARRISON_CAPACITY,
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct WoodTowerBehavior {
    pub occupants_immobile: bool,
    pub occupants_untargetable: bool,
    pub can_upgrade_to_stone_tower: bool,
    pub hostile_prefer_tower_over_wall_or_gate: bool,
}

#[derive(Clone, Copy)]
pub struct GarrisonedUnit {
    pub unit_id: i32,
    pub base_attack_range: I32F32,
    pub base_watch_range: I32F32,
    pub tower_attack_range: I32F32,
    pub tower_watch_range: I32F32,
    pub immobile: bool,
    pub untargetable: bool,
}

#[derive(Component, Default, Clone)]
pub struct WoodTowerGarrison {
    pub units: BTreeMap<i32, GarrisonedUnit>,
}

#[derive(Resource, Default, Clone)]
pub struct WoodTowerPlacementClaims {
    pub claims: BTreeMap<Entity, BuildingAnchor>,
}

#[derive(Event, Clone, Copy)]
pub struct PlaceWoodTowerEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct WoodTowerPlacementRejectedEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct GarrisonUnitInWoodTowerEvent {
    pub tower_entity: Entity,
    pub unit_id: i32,
    pub base_attack_range: I32F32,
    pub base_watch_range: I32F32,
}

#[derive(Event, Clone, Copy)]
pub struct GarrisonUnitInWoodTowerRejectedEvent {
    pub tower_entity: Entity,
    pub unit_id: i32,
}

#[derive(Event, Clone, Copy)]
pub struct UngarrisonUnitFromWoodTowerEvent {
    pub tower_entity: Entity,
    pub unit_id: i32,
}

#[derive(Event, Clone, Copy)]
pub struct DamageWoodTowerEvent {
    pub tower_entity: Entity,
    pub damage: I32F32,
}

#[derive(Event, Clone, Copy)]
pub struct WoodTowerDestroyedDepositedUnitEvent {
    pub unit_id: i32,
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct UpgradeWoodTowerToStoneTowerEvent {
    pub wood_tower_entity: Entity,
}

fn place_wood_tower_system(
    mut commands: Commands,
    mut events: EventReader<PlaceWoodTowerEvent>,
    mut rejected: EventWriter<WoodTowerPlacementRejectedEvent>,
    mut claims: ResMut<WoodTowerPlacementClaims>,
) {
    for ev in events.read() {
        let anchor = BuildingAnchor {
            x: ev.tile_x,
            y: ev.tile_y,
        };

        if claims
            .claims
            .values()
            .any(|existing| existing.x == anchor.x && existing.y == anchor.y)
        {
            rejected.send(WoodTowerPlacementRejectedEvent {
                tile_x: ev.tile_x,
                tile_y: ev.tile_y,
            });
            continue;
        }

        let entity = commands
            .spawn((
                WoodTower,
                anchor,
                WoodTowerCore::default(),
                WoodTowerEconomy::default(),
                WoodTowerBuildState::default(),
                WoodTowerFootprint::default(),
                WoodTowerBehavior {
                    occupants_immobile: true,
                    occupants_untargetable: true,
                    can_upgrade_to_stone_tower: true,
                    hostile_prefer_tower_over_wall_or_gate: true,
                },
                WoodTowerGarrison::default(),
            ))
            .id();

        claims.claims.insert(entity, anchor);
    }
}

fn wood_tower_build_tick_system(
    mut towers: Query<(&mut WoodTowerCore, &mut WoodTowerBuildState), With<WoodTower>>,
) {
    let hp_per_tick = WOOD_TOWER_CONSTRUCTION_HP_PER_SECOND / I32F32::from_num(SIM_HZ);

    for (mut core, mut build) in &mut towers {
        if build.completed {
            continue;
        }

        if build.build_ticks_remaining > 0 {
            build.build_ticks_remaining -= 1;
        }

        core.health.current = (core.health.current + hp_per_tick).min(core.health.max);

        if build.build_ticks_remaining <= 0 {
            build.build_ticks_remaining = 0;
            build.completed = true;
        }
    }
}

fn garrison_unit_in_wood_tower_system(
    mut events: EventReader<GarrisonUnitInWoodTowerEvent>,
    mut rejected: EventWriter<GarrisonUnitInWoodTowerRejectedEvent>,
    mut towers: Query<
        (
            &WoodTowerBuildState,
            &WoodTowerFootprint,
            &WoodTowerBehavior,
            &mut WoodTowerGarrison,
        ),
        With<WoodTower>,
    >,
) {
    for ev in events.read() {
        let Ok((build, footprint, behavior, mut garrison)) = towers.get_mut(ev.tower_entity) else {
            rejected.send(GarrisonUnitInWoodTowerRejectedEvent {
                tower_entity: ev.tower_entity,
                unit_id: ev.unit_id,
            });
            continue;
        };

        if !build.completed {
            rejected.send(GarrisonUnitInWoodTowerRejectedEvent {
                tower_entity: ev.tower_entity,
                unit_id: ev.unit_id,
            });
            continue;
        }

        if garrison.units.contains_key(&ev.unit_id) {
            continue;
        }

        if garrison.units.len() as i32 >= footprint.garrison_capacity {
            rejected.send(GarrisonUnitInWoodTowerRejectedEvent {
                tower_entity: ev.tower_entity,
                unit_id: ev.unit_id,
            });
            continue;
        }

        let tower_attack_range = ev.base_attack_range + footprint.attack_range_bonus;
        let tower_watch_range = ev.base_watch_range + footprint.watch_range_bonus;

        garrison.units.insert(
            ev.unit_id,
            GarrisonedUnit {
                unit_id: ev.unit_id,
                base_attack_range: ev.base_attack_range,
                base_watch_range: ev.base_watch_range,
                tower_attack_range,
                tower_watch_range,
                immobile: behavior.occupants_immobile,
                untargetable: behavior.occupants_untargetable,
            },
        );
    }
}

fn ungarrison_unit_from_wood_tower_system(
    mut events: EventReader<UngarrisonUnitFromWoodTowerEvent>,
    mut towers: Query<&mut WoodTowerGarrison, With<WoodTower>>,
) {
    for ev in events.read() {
        let Ok(mut garrison) = towers.get_mut(ev.tower_entity) else {
            continue;
        };

        garrison.units.remove(&ev.unit_id);
    }
}

fn damage_wood_tower_system(
    mut commands: Commands,
    mut events: EventReader<DamageWoodTowerEvent>,
    mut deposited_units: EventWriter<WoodTowerDestroyedDepositedUnitEvent>,
    mut towers: Query<(Entity, &BuildingAnchor, &mut WoodTowerCore, &WoodTowerGarrison), With<WoodTower>>,
    mut claims: ResMut<WoodTowerPlacementClaims>,
) {
    for ev in events.read() {
        let Ok((tower_entity, anchor, mut core, garrison)) = towers.get_mut(ev.tower_entity) else {
            continue;
        };

        let mut incoming = ev.damage;
        if incoming <= I32F32::ZERO {
            continue;
        }

        if core.defenses_life > I32F32::ZERO {
            let absorbed = incoming.min(core.defenses_life);
            core.defenses_life -= absorbed;
            incoming -= absorbed;
        }

        if incoming > I32F32::ZERO {
            core.health.current -= incoming;
        }

        if core.health.current > I32F32::ZERO {
            continue;
        }

        core.health.current = I32F32::ZERO;
        for unit in garrison.units.values() {
            deposited_units.send(WoodTowerDestroyedDepositedUnitEvent {
                unit_id: unit.unit_id,
                tile_x: anchor.x,
                tile_y: anchor.y,
            });
        }

        claims.claims.remove(&tower_entity);
        commands.entity(tower_entity).despawn();
    }
}

fn wood_tower_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    towers: Query<
        (
            Entity,
            &BuildingAnchor,
            &WoodTowerCore,
            &WoodTowerEconomy,
            &WoodTowerBuildState,
            &WoodTowerFootprint,
            &WoodTowerBehavior,
            &WoodTowerGarrison,
        ),
        With<WoodTower>,
    >,
    claims: Res<WoodTowerPlacementClaims>,
) {
    for (entity, anchor, core, economy, build, footprint, behavior, garrison) in &towers {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);

        checksum.accumulate(core.defenses_life.to_bits() as u64);
        checksum.accumulate(core.health.current.to_bits() as u64);
        checksum.accumulate(core.health.max.to_bits() as u64);

        checksum.accumulate(economy.wood_cost as u64);
        checksum.accumulate(economy.stone_cost as u64);
        checksum.accumulate(economy.iron_cost as u64);
        checksum.accumulate(economy.oil_cost as u64);
        checksum.accumulate(economy.gold_cost as u64);

        checksum.accumulate(build.build_ticks_remaining as u64);
        checksum.accumulate(u64::from(build.completed));

        checksum.accumulate(footprint.size_tiles as u64);
        checksum.accumulate(footprint.watch_range_bonus.to_bits() as u64);
        checksum.accumulate(footprint.attack_range_bonus.to_bits() as u64);
        checksum.accumulate(footprint.garrison_capacity as u64);

        checksum.accumulate(u64::from(behavior.occupants_immobile));
        checksum.accumulate(u64::from(behavior.occupants_untargetable));
        checksum.accumulate(u64::from(behavior.can_upgrade_to_stone_tower));
        checksum.accumulate(u64::from(behavior.hostile_prefer_tower_over_wall_or_gate));

        checksum.accumulate(garrison.units.len() as u64);
        for (unit_id, unit) in &garrison.units {
            checksum.accumulate(*unit_id as u64);
            checksum.accumulate(unit.base_attack_range.to_bits() as u64);
            checksum.accumulate(unit.base_watch_range.to_bits() as u64);
            checksum.accumulate(unit.tower_attack_range.to_bits() as u64);
            checksum.accumulate(unit.tower_watch_range.to_bits() as u64);
            checksum.accumulate(u64::from(unit.immobile));
            checksum.accumulate(u64::from(unit.untargetable));
        }
    }

    checksum.accumulate(claims.claims.len() as u64);
    for (entity, anchor) in &claims.claims {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);
    }
}

pub struct WoodTowerPlugin;

impl Plugin for WoodTowerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WoodTowerPlacementClaims>()
            .add_event::<PlaceWoodTowerEvent>()
            .add_event::<WoodTowerPlacementRejectedEvent>()
            .add_event::<GarrisonUnitInWoodTowerEvent>()
            .add_event::<GarrisonUnitInWoodTowerRejectedEvent>()
            .add_event::<UngarrisonUnitFromWoodTowerEvent>()
            .add_event::<DamageWoodTowerEvent>()
            .add_event::<WoodTowerDestroyedDepositedUnitEvent>()
            .add_event::<UpgradeWoodTowerToStoneTowerEvent>()
            .add_systems(
                FixedUpdate,
                (
                    place_wood_tower_system,
                    wood_tower_build_tick_system,
                    garrison_unit_in_wood_tower_system,
                    ungarrison_unit_from_wood_tower_system,
                    damage_wood_tower_system,
                    wood_tower_checksum_system,
                )
                    .chain(),
            );
    }
}