// Sources: vault/buildings/soldiers_center.md

use std::collections::BTreeMap;

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::SimChecksumState;

const SOLDIERS_CENTER_HP: I32F32 = I32F32::lit("800");
const SOLDIERS_CENTER_DEFENSES_LIFE: I32F32 = I32F32::lit("400");
const SOLDIERS_CENTER_WATCH_RANGE: I32F32 = I32F32::lit("7");

const SOLDIERS_CENTER_ENERGY_COST: i32 = 6;
const SOLDIERS_CENTER_WOOD_COST: i32 = 20;
const SOLDIERS_CENTER_STONE_COST: i32 = 20;
const SOLDIERS_CENTER_IRON_COST: i32 = 0;
const SOLDIERS_CENTER_OIL_COST: i32 = 0;
const SOLDIERS_CENTER_GOLD_COST: i32 = 450;
const SOLDIERS_CENTER_BUILD_TIME_SECONDS: i32 = 60;
const SOLDIERS_CENTER_WORKERS: i32 = 8;
const SOLDIERS_CENTER_MAINTENANCE_GOLD: i32 = 14;

const SOLDIERS_CENTER_SIZE_TILES: i32 = 3;
const SOLDIERS_CENTER_BUILD_AREA_TILES: i32 = 5;

const SIM_HZ: i32 = 25;

#[derive(Component, Default)]
pub struct SoldiersCenter;

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
            hp: SOLDIERS_CENTER_HP,
            defenses_life: SOLDIERS_CENTER_DEFENSES_LIFE,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct SoldiersCenterBuildState {
    pub build_ticks_remaining: i32,
    pub completed: bool,
}

#[derive(Component, Clone, Copy)]
pub struct SoldiersCenterEconomy {
    pub energy_cost: i32,
    pub wood_cost: i32,
    pub stone_cost: i32,
    pub iron_cost: i32,
    pub oil_cost: i32,
    pub gold_cost: i32,
    pub maintenance_gold: i32,
    pub workers: i32,
}

impl Default for SoldiersCenterEconomy {
    fn default() -> Self {
        Self {
            energy_cost: SOLDIERS_CENTER_ENERGY_COST,
            wood_cost: SOLDIERS_CENTER_WOOD_COST,
            stone_cost: SOLDIERS_CENTER_STONE_COST,
            iron_cost: SOLDIERS_CENTER_IRON_COST,
            oil_cost: SOLDIERS_CENTER_OIL_COST,
            gold_cost: SOLDIERS_CENTER_GOLD_COST,
            maintenance_gold: SOLDIERS_CENTER_MAINTENANCE_GOLD,
            workers: SOLDIERS_CENTER_WORKERS,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct SoldiersCenterFootprint {
    pub size_tiles: i32,
    pub build_area_tiles: i32,
    pub watch_range: I32F32,
}

impl Default for SoldiersCenterFootprint {
    fn default() -> Self {
        Self {
            size_tiles: SOLDIERS_CENTER_SIZE_TILES,
            build_area_tiles: SOLDIERS_CENTER_BUILD_AREA_TILES,
            watch_range: SOLDIERS_CENTER_WATCH_RANGE,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SoldiersCenterTrainableUnit {
    Ranger = 1,
    Soldier = 2,
    Sniper = 3,
}

#[derive(Component, Clone, Copy)]
pub struct SoldiersCenterUnlocks {
    pub ranger_unlocked: bool,
    pub soldier_unlocked: bool,
    pub sniper_unlocked: bool,
}

impl Default for SoldiersCenterUnlocks {
    fn default() -> Self {
        Self {
            ranger_unlocked: true,
            soldier_unlocked: false,
            sniper_unlocked: false,
        }
    }
}

#[derive(Clone, Copy)]
pub struct QueuedUnitTraining {
    pub queue_slot: i32,
    pub unit_instance_id: i32,
    pub unit_kind: SoldiersCenterTrainableUnit,
    pub ticks_remaining: i32,
}

#[derive(Component, Default, Clone)]
pub struct SoldiersCenterTrainingQueue {
    pub entries: BTreeMap<i32, QueuedUnitTraining>,
    pub next_queue_slot: i32,
}

#[derive(Resource, Default, Clone)]
pub struct SoldiersCenterPlacementClaims {
    pub claims: BTreeMap<Entity, BuildingAnchor>,
}

#[derive(Event, Clone, Copy)]
pub struct PlaceSoldiersCenterEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct SoldiersCenterPlacementRejectedEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct SetSoldiersCenterUnitUnlockedEvent {
    pub center_entity: Entity,
    pub unit_kind: SoldiersCenterTrainableUnit,
    pub unlocked: bool,
}

#[derive(Event, Clone, Copy)]
pub struct QueueSoldiersCenterTrainingEvent {
    pub center_entity: Entity,
    pub unit_instance_id: i32,
    pub unit_kind: SoldiersCenterTrainableUnit,
    pub training_time_seconds: i32,
}

#[derive(Event, Clone, Copy)]
pub struct QueueSoldiersCenterTrainingRejectedEvent {
    pub center_entity: Entity,
    pub unit_instance_id: i32,
    pub unit_kind: SoldiersCenterTrainableUnit,
}

#[derive(Event, Clone, Copy)]
pub struct SoldiersCenterUnitTrainedEvent {
    pub center_entity: Entity,
    pub unit_instance_id: i32,
    pub unit_kind: SoldiersCenterTrainableUnit,
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct DismissUnitNearSoldiersCenterEvent {
    pub center_entity: Entity,
    pub unit_instance_id: i32,
}

#[derive(Event, Clone, Copy)]
pub struct DismissUnitNearSoldiersCenterAllowedEvent {
    pub center_entity: Entity,
    pub unit_instance_id: i32,
}

#[derive(Event, Clone, Copy)]
pub struct DismissUnitNearSoldiersCenterRejectedEvent {
    pub center_entity: Entity,
    pub unit_instance_id: i32,
}

#[derive(Event, Clone, Copy)]
pub struct DamageSoldiersCenterEvent {
    pub center_entity: Entity,
    pub damage: I32F32,
}

fn seconds_to_ticks(seconds: i32) -> i32 {
    seconds * SIM_HZ
}

fn is_unit_unlocked(unlocks: &SoldiersCenterUnlocks, unit: SoldiersCenterTrainableUnit) -> bool {
    match unit {
        SoldiersCenterTrainableUnit::Ranger => unlocks.ranger_unlocked,
        SoldiersCenterTrainableUnit::Soldier => unlocks.soldier_unlocked,
        SoldiersCenterTrainableUnit::Sniper => unlocks.sniper_unlocked,
    }
}

fn place_soldiers_center_system(
    mut commands: Commands,
    mut events: EventReader<PlaceSoldiersCenterEvent>,
    mut rejected: EventWriter<SoldiersCenterPlacementRejectedEvent>,
    mut claims: ResMut<SoldiersCenterPlacementClaims>,
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
            rejected.send(SoldiersCenterPlacementRejectedEvent {
                tile_x: ev.tile_x,
                tile_y: ev.tile_y,
            });
            continue;
        }

        let entity = commands
            .spawn((
                SoldiersCenter,
                anchor,
                BuildingHealth::default(),
                SoldiersCenterBuildState {
                    build_ticks_remaining: seconds_to_ticks(SOLDIERS_CENTER_BUILD_TIME_SECONDS),
                    completed: false,
                },
                SoldiersCenterEconomy::default(),
                SoldiersCenterFootprint::default(),
                SoldiersCenterUnlocks::default(),
                SoldiersCenterTrainingQueue::default(),
            ))
            .id();

        claims.claims.insert(entity, anchor);
    }
}

fn soldiers_center_build_tick_system(mut centers: Query<&mut SoldiersCenterBuildState, With<SoldiersCenter>>) {
    for mut state in &mut centers {
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

fn set_soldiers_center_unlock_system(
    mut events: EventReader<SetSoldiersCenterUnitUnlockedEvent>,
    mut centers: Query<&mut SoldiersCenterUnlocks, With<SoldiersCenter>>,
) {
    for ev in events.read() {
        let Ok(mut unlocks) = centers.get_mut(ev.center_entity) else {
            continue;
        };

        match ev.unit_kind {
            SoldiersCenterTrainableUnit::Ranger => unlocks.ranger_unlocked = ev.unlocked,
            SoldiersCenterTrainableUnit::Soldier => unlocks.soldier_unlocked = ev.unlocked,
            SoldiersCenterTrainableUnit::Sniper => unlocks.sniper_unlocked = ev.unlocked,
        }
    }
}

fn queue_soldiers_center_training_system(
    mut events: EventReader<QueueSoldiersCenterTrainingEvent>,
    mut rejected: EventWriter<QueueSoldiersCenterTrainingRejectedEvent>,
    mut centers: Query<
        (
            &SoldiersCenterBuildState,
            &SoldiersCenterUnlocks,
            &mut SoldiersCenterTrainingQueue,
        ),
        With<SoldiersCenter>,
    >,
) {
    for ev in events.read() {
        let Ok((build, unlocks, mut queue)) = centers.get_mut(ev.center_entity) else {
            rejected.send(QueueSoldiersCenterTrainingRejectedEvent {
                center_entity: ev.center_entity,
                unit_instance_id: ev.unit_instance_id,
                unit_kind: ev.unit_kind,
            });
            continue;
        };

        if !build.completed || !is_unit_unlocked(unlocks, ev.unit_kind) || ev.training_time_seconds <= 0 {
            rejected.send(QueueSoldiersCenterTrainingRejectedEvent {
                center_entity: ev.center_entity,
                unit_instance_id: ev.unit_instance_id,
                unit_kind: ev.unit_kind,
            });
            continue;
        }

        let slot = queue.next_queue_slot;
        queue.next_queue_slot += 1;
        queue.entries.insert(
            slot,
            QueuedUnitTraining {
                queue_slot: slot,
                unit_instance_id: ev.unit_instance_id,
                unit_kind: ev.unit_kind,
                ticks_remaining: seconds_to_ticks(ev.training_time_seconds),
            },
        );
    }
}

fn soldiers_center_training_tick_system(
    mut centers: Query<(Entity, &BuildingAnchor, &mut SoldiersCenterTrainingQueue), With<SoldiersCenter>>,
    mut trained_writer: EventWriter<SoldiersCenterUnitTrainedEvent>,
) {
    for (entity, anchor, mut queue) in &mut centers {
        let Some(front_slot) = queue.entries.keys().next().copied() else {
            continue;
        };

        let mut complete = false;
        if let Some(front_entry) = queue.entries.get_mut(&front_slot) {
            if front_entry.ticks_remaining > 0 {
                front_entry.ticks_remaining -= 1;
            }
            complete = front_entry.ticks_remaining <= 0;
        }

        if !complete {
            continue;
        }

        if let Some(finished) = queue.entries.remove(&front_slot) {
            trained_writer.send(SoldiersCenterUnitTrainedEvent {
                center_entity: entity,
                unit_instance_id: finished.unit_instance_id,
                unit_kind: finished.unit_kind,
                tile_x: anchor.x,
                tile_y: anchor.y,
            });
        }
    }
}

fn dismiss_unit_near_soldiers_center_system(
    mut events: EventReader<DismissUnitNearSoldiersCenterEvent>,
    mut allowed: EventWriter<DismissUnitNearSoldiersCenterAllowedEvent>,
    mut rejected: EventWriter<DismissUnitNearSoldiersCenterRejectedEvent>,
    centers: Query<(), With<SoldiersCenter>>,
) {
    for ev in events.read() {
        if centers.get(ev.center_entity).is_ok() {
            allowed.send(DismissUnitNearSoldiersCenterAllowedEvent {
                center_entity: ev.center_entity,
                unit_instance_id: ev.unit_instance_id,
            });
        } else {
            rejected.send(DismissUnitNearSoldiersCenterRejectedEvent {
                center_entity: ev.center_entity,
                unit_instance_id: ev.unit_instance_id,
            });
        }
    }
}

fn damage_soldiers_center_system(
    mut commands: Commands,
    mut events: EventReader<DamageSoldiersCenterEvent>,
    mut centers: Query<(Entity, &mut BuildingHealth), With<SoldiersCenter>>,
    mut claims: ResMut<SoldiersCenterPlacementClaims>,
) {
    for ev in events.read() {
        let Ok((entity, mut hp)) = centers.get_mut(ev.center_entity) else {
            continue;
        };

        if ev.damage <= I32F32::ZERO {
            continue;
        }

        if hp.hp > ev.damage {
            hp.hp -= ev.damage;
            continue;
        }

        hp.hp = I32F32::ZERO;
        claims.claims.remove(&entity);
        commands.entity(entity).despawn();
    }
}

fn soldiers_center_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    centers: Query<
        (
            &BuildingAnchor,
            &BuildingHealth,
            &SoldiersCenterBuildState,
            &SoldiersCenterEconomy,
            &SoldiersCenterFootprint,
            &SoldiersCenterUnlocks,
            &SoldiersCenterTrainingQueue,
        ),
        With<SoldiersCenter>,
    >,
    claims: Res<SoldiersCenterPlacementClaims>,
) {
    for (anchor, health, build, economy, footprint, unlocks, queue) in &centers {
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);

        checksum.accumulate(health.hp.to_bits() as u64);
        checksum.accumulate(health.defenses_life.to_bits() as u64);

        checksum.accumulate(build.build_ticks_remaining as u64);
        checksum.accumulate(u64::from(build.completed));

        checksum.accumulate(economy.energy_cost as u64);
        checksum.accumulate(economy.wood_cost as u64);
        checksum.accumulate(economy.stone_cost as u64);
        checksum.accumulate(economy.iron_cost as u64);
        checksum.accumulate(economy.oil_cost as u64);
        checksum.accumulate(economy.gold_cost as u64);
        checksum.accumulate(economy.maintenance_gold as u64);
        checksum.accumulate(economy.workers as u64);

        checksum.accumulate(footprint.size_tiles as u64);
        checksum.accumulate(footprint.build_area_tiles as u64);
        checksum.accumulate(footprint.watch_range.to_bits() as u64);

        checksum.accumulate(u64::from(unlocks.ranger_unlocked));
        checksum.accumulate(u64::from(unlocks.soldier_unlocked));
        checksum.accumulate(u64::from(unlocks.sniper_unlocked));

        checksum.accumulate(queue.entries.len() as u64);
        checksum.accumulate(queue.next_queue_slot as u64);
        for (slot, entry) in &queue.entries {
            checksum.accumulate(*slot as u64);
            checksum.accumulate(entry.queue_slot as u64);
            checksum.accumulate(entry.unit_instance_id as u64);
            checksum.accumulate(entry.unit_kind as u64);
            checksum.accumulate(entry.ticks_remaining as u64);
        }
    }

    checksum.accumulate(claims.claims.len() as u64);
    for (entity, anchor) in &claims.claims {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);
    }
}

pub struct SoldiersCenterPlugin;

impl Plugin for SoldiersCenterPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SoldiersCenterPlacementClaims>()
            .add_event::<PlaceSoldiersCenterEvent>()
            .add_event::<SoldiersCenterPlacementRejectedEvent>()
            .add_event::<SetSoldiersCenterUnitUnlockedEvent>()
            .add_event::<QueueSoldiersCenterTrainingEvent>()
            .add_event::<QueueSoldiersCenterTrainingRejectedEvent>()
            .add_event::<SoldiersCenterUnitTrainedEvent>()
            .add_event::<DismissUnitNearSoldiersCenterEvent>()
            .add_event::<DismissUnitNearSoldiersCenterAllowedEvent>()
            .add_event::<DismissUnitNearSoldiersCenterRejectedEvent>()
            .add_event::<DamageSoldiersCenterEvent>()
            .add_systems(
                FixedUpdate,
                (
                    place_soldiers_center_system,
                    soldiers_center_build_tick_system,
                    set_soldiers_center_unlock_system,
                    queue_soldiers_center_training_system,
                    soldiers_center_training_tick_system,
                    dismiss_unit_near_soldiers_center_system,
                    damage_soldiers_center_system,
                    soldiers_center_checksum_system,
                )
                    .chain(),
            );
    }
}