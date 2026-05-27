// Sources: vault/buildings/foundry.md

use std::collections::{BTreeMap, BTreeSet};

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{Health, SimChecksumState};

const FOUNDRY_HP: I32F32 = I32F32::lit("1500");
const FOUNDRY_DEFENSES_LIFE: I32F32 = I32F32::lit("375");
const FOUNDRY_WATCH_RANGE: I32F32 = I32F32::lit("7");

const FOUNDRY_ENERGY_COST: i32 = 50;
const FOUNDRY_WOOD_COST: i32 = 30;
const FOUNDRY_STONE_COST: i32 = 50;
const FOUNDRY_IRON_COST: i32 = 50;
const FOUNDRY_OIL_COST: i32 = 0;
const FOUNDRY_GOLD_COST: i32 = 3000;
const FOUNDRY_WORKERS: i32 = 40;

const FOUNDRY_BUILD_TIME_SECONDS: i32 = 105;
const FOUNDRY_SIZE_TILES: i32 = 4;
const SIM_HZ: i32 = 25;

const UNLOCK_ADVANCED_FARM: &str = "advanced_farm";
const UNLOCK_ADVANCED_QUARRY: &str = "advanced_quarry";
const UNLOCK_ADVANCED_MILL: &str = "advanced_mill";
const UNLOCK_OIL_PLATFORM: &str = "oil_platform";
const UNLOCK_ENGINEERING_CENTER: &str = "engineering_center";
const UNLOCK_RADAR_TOWER: &str = "radar_tower";
const UNLOCK_EXECUTOR: &str = "executor";
const UNLOCK_WIRE_FENCE_TRAP: &str = "wire_fence_trap";
const UNLOCK_THANATOS: &str = "thanatos";
const UNLOCK_TITAN: &str = "titan";
const UNLOCK_MUTANT: &str = "mutant";
const UNLOCK_ATLAS_TRANSMUTATOR: &str = "atlas_transmutator";
const UNLOCK_LIGHTNING_SPIRE: &str = "lightning_spire";

#[derive(Component, Default)]
pub struct Foundry;

#[derive(Component, Clone, Copy, Default)]
pub struct BuildingAnchor {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Clone, Copy)]
pub struct FoundryCore {
    pub defenses_life: I32F32,
    pub watch_range: I32F32,
    pub health: Health,
}

impl Default for FoundryCore {
    fn default() -> Self {
        Self {
            defenses_life: FOUNDRY_DEFENSES_LIFE,
            watch_range: FOUNDRY_WATCH_RANGE,
            health: Health::full(FOUNDRY_HP),
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct FoundryBuildState {
    pub build_ticks_remaining: i32,
    pub completed: bool,
}

#[derive(Component, Clone, Copy)]
pub struct FoundryEconomy {
    pub energy_cost: i32,
    pub wood_cost: i32,
    pub stone_cost: i32,
    pub iron_cost: i32,
    pub oil_cost: i32,
    pub gold_cost: i32,
    pub workers: i32,
}

impl Default for FoundryEconomy {
    fn default() -> Self {
        Self {
            energy_cost: FOUNDRY_ENERGY_COST,
            wood_cost: FOUNDRY_WOOD_COST,
            stone_cost: FOUNDRY_STONE_COST,
            iron_cost: FOUNDRY_IRON_COST,
            oil_cost: FOUNDRY_OIL_COST,
            gold_cost: FOUNDRY_GOLD_COST,
            workers: FOUNDRY_WORKERS,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct FoundryFootprint {
    pub size_tiles: i32,
    pub watch_range: I32F32,
}

impl Default for FoundryFootprint {
    fn default() -> Self {
        Self {
            size_tiles: FOUNDRY_SIZE_TILES,
            watch_range: FOUNDRY_WATCH_RANGE,
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct FoundryPolicy {
    pub refund_allowed: bool,
}

#[derive(Component, Clone, Default)]
pub struct FoundryUnlockSet {
    pub ids: BTreeSet<&'static str>,
}

impl FoundryUnlockSet {
    pub fn from_default_unlocks() -> Self {
        let mut ids = BTreeSet::new();
        ids.insert(UNLOCK_ADVANCED_FARM);
        ids.insert(UNLOCK_ADVANCED_QUARRY);
        ids.insert(UNLOCK_ADVANCED_MILL);
        ids.insert(UNLOCK_OIL_PLATFORM);
        ids.insert(UNLOCK_ENGINEERING_CENTER);
        ids.insert(UNLOCK_RADAR_TOWER);
        ids.insert(UNLOCK_EXECUTOR);
        ids.insert(UNLOCK_WIRE_FENCE_TRAP);
        ids.insert(UNLOCK_THANATOS);
        ids.insert(UNLOCK_TITAN);
        ids.insert(UNLOCK_MUTANT);
        ids.insert(UNLOCK_ATLAS_TRANSMUTATOR);
        ids.insert(UNLOCK_LIGHTNING_SPIRE);
        Self { ids }
    }
}

#[derive(Resource, Clone, Default)]
pub struct FoundryPlacementClaims {
    pub claims: BTreeMap<Entity, BuildingAnchor>,
}

#[derive(Resource, Clone, Default)]
pub struct FoundryTechAccessState {
    pub technologies_accessible: bool,
    pub active_completed_foundries: i32,
    pub unlocked_ids: BTreeSet<&'static str>,
}

#[derive(Event, Clone, Copy)]
pub struct PlaceFoundryEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct FoundryPlacementRejectedEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct RemoveFoundryEvent {
    pub foundry_entity: Entity,
}

#[derive(Event, Clone, Copy)]
pub struct RefundFoundryEvent {
    pub foundry_entity: Entity,
}

#[derive(Event, Clone, Copy)]
pub struct FoundryRefundRejectedEvent {
    pub foundry_entity: Entity,
}

#[derive(Event, Clone, Copy)]
pub struct DamageFoundryEvent {
    pub foundry_entity: Entity,
    pub damage: I32F32,
}

fn seconds_to_ticks(seconds: i32) -> i32 {
    seconds * SIM_HZ
}

fn place_foundry_system(
    mut commands: Commands,
    mut events: EventReader<PlaceFoundryEvent>,
    mut rejected: EventWriter<FoundryPlacementRejectedEvent>,
    mut claims: ResMut<FoundryPlacementClaims>,
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
            rejected.send(FoundryPlacementRejectedEvent {
                tile_x: ev.tile_x,
                tile_y: ev.tile_y,
            });
            continue;
        }

        let entity = commands
            .spawn((
                Foundry,
                anchor,
                FoundryCore::default(),
                FoundryBuildState {
                    build_ticks_remaining: seconds_to_ticks(FOUNDRY_BUILD_TIME_SECONDS),
                    completed: false,
                },
                FoundryEconomy::default(),
                FoundryFootprint::default(),
                FoundryPolicy {
                    refund_allowed: false,
                },
                FoundryUnlockSet::from_default_unlocks(),
            ))
            .id();

        claims.claims.insert(entity, anchor);
    }
}

fn foundry_build_tick_system(mut foundries: Query<&mut FoundryBuildState, With<Foundry>>) {
    for mut build in &mut foundries {
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

fn remove_foundry_system(
    mut commands: Commands,
    mut events: EventReader<RemoveFoundryEvent>,
    foundries: Query<(), With<Foundry>>,
    mut claims: ResMut<FoundryPlacementClaims>,
) {
    for ev in events.read() {
        if foundries.get(ev.foundry_entity).is_err() {
            continue;
        }

        claims.claims.remove(&ev.foundry_entity);
        commands.entity(ev.foundry_entity).despawn();
    }
}

fn refund_foundry_system(
    mut events: EventReader<RefundFoundryEvent>,
    mut rejected: EventWriter<FoundryRefundRejectedEvent>,
    foundries: Query<&FoundryPolicy, With<Foundry>>,
) {
    for ev in events.read() {
        let Ok(policy) = foundries.get(ev.foundry_entity) else {
            continue;
        };

        if !policy.refund_allowed {
            rejected.send(FoundryRefundRejectedEvent {
                foundry_entity: ev.foundry_entity,
            });
        }
    }
}

fn damage_foundry_system(
    mut commands: Commands,
    mut events: EventReader<DamageFoundryEvent>,
    mut foundries: Query<(Entity, &mut FoundryCore), With<Foundry>>,
    mut claims: ResMut<FoundryPlacementClaims>,
) {
    for ev in events.read() {
        let Ok((entity, mut core)) = foundries.get_mut(ev.foundry_entity) else {
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
            if core.health.current <= I32F32::ZERO {
                core.health.current = I32F32::ZERO;
                claims.claims.remove(&entity);
                commands.entity(entity).despawn();
            }
        }
    }
}

fn foundry_technology_access_system(
    foundries: Query<(&FoundryBuildState, &FoundryUnlockSet), With<Foundry>>,
    mut state: ResMut<FoundryTechAccessState>,
) {
    let mut completed_count = 0_i32;
    let mut unlocked = BTreeSet::new();

    for (build, unlocks) in &foundries {
        if !build.completed {
            continue;
        }

        completed_count += 1;
        for id in &unlocks.ids {
            unlocked.insert(*id);
        }
    }

    state.active_completed_foundries = completed_count;
    state.technologies_accessible = completed_count > 0;
    state.unlocked_ids = unlocked;
}

fn foundry_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    foundries: Query<
        (
            Entity,
            &BuildingAnchor,
            &FoundryCore,
            &FoundryBuildState,
            &FoundryEconomy,
            &FoundryFootprint,
            &FoundryPolicy,
            &FoundryUnlockSet,
        ),
        With<Foundry>,
    >,
    claims: Res<FoundryPlacementClaims>,
    tech_state: Res<FoundryTechAccessState>,
) {
    for (entity, anchor, core, build, economy, footprint, policy, unlocks) in &foundries {
        checksum.accumulate(entity.to_bits() as u64);

        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);

        checksum.accumulate(core.defenses_life.to_bits() as u64);
        checksum.accumulate(core.watch_range.to_bits() as u64);
        checksum.accumulate(core.health.current.to_bits() as u64);
        checksum.accumulate(core.health.max.to_bits() as u64);

        checksum.accumulate(build.build_ticks_remaining as u64);
        checksum.accumulate(u64::from(build.completed));

        checksum.accumulate(economy.energy_cost as u64);
        checksum.accumulate(economy.wood_cost as u64);
        checksum.accumulate(economy.stone_cost as u64);
        checksum.accumulate(economy.iron_cost as u64);
        checksum.accumulate(economy.oil_cost as u64);
        checksum.accumulate(economy.gold_cost as u64);
        checksum.accumulate(economy.workers as u64);

        checksum.accumulate(footprint.size_tiles as u64);
        checksum.accumulate(footprint.watch_range.to_bits() as u64);

        checksum.accumulate(u64::from(policy.refund_allowed));

        checksum.accumulate(unlocks.ids.len() as u64);
        for id in &unlocks.ids {
            for byte in id.as_bytes() {
                checksum.accumulate(*byte as u64);
            }
        }
    }

    checksum.accumulate(claims.claims.len() as u64);
    for (entity, anchor) in &claims.claims {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);
    }

    checksum.accumulate(u64::from(tech_state.technologies_accessible));
    checksum.accumulate(tech_state.active_completed_foundries as u64);

    checksum.accumulate(tech_state.unlocked_ids.len() as u64);
    for id in &tech_state.unlocked_ids {
        for byte in id.as_bytes() {
            checksum.accumulate(*byte as u64);
        }
    }
}

pub struct FoundryPlugin;

impl Plugin for FoundryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FoundryPlacementClaims>()
            .init_resource::<FoundryTechAccessState>()
            .add_event::<PlaceFoundryEvent>()
            .add_event::<FoundryPlacementRejectedEvent>()
            .add_event::<RemoveFoundryEvent>()
            .add_event::<RefundFoundryEvent>()
            .add_event::<FoundryRefundRejectedEvent>()
            .add_event::<DamageFoundryEvent>()
            .add_systems(
                FixedUpdate,
                (
                    place_foundry_system,
                    foundry_build_tick_system,
                    remove_foundry_system,
                    refund_foundry_system,
                    damage_foundry_system,
                    foundry_technology_access_system,
                    foundry_checksum_system,
                )
                    .chain(),
            );
    }
}