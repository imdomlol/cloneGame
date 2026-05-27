// Sources: vault/game_mechanics/explosive_barrel.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{
    DamageType, EntityKilledEvent, Health, IncomingDamageEvent, SimChecksumState, SimPosition,
};

const BARREL_HP: I32F32 = I32F32::lit("5");
const BARREL_AOE_TILES: I32F32 = I32F32::lit("3");
const BARREL_DAMAGE: I32F32 = I32F32::lit("500");
const BARREL_NOISE: I32F32 = I32F32::lit("0");

#[derive(Component, Clone, Copy, Default)]
pub struct ExplosiveBarrel;

#[derive(Component, Clone, Copy, Default)]
pub struct BarrelCarrierEligible;

#[derive(Component, Clone, Copy)]
pub struct BarrelCarried {
    pub carrier: Entity,
}

#[derive(Component, Clone, Copy, Default)]
pub struct PendingBarrelExplosion;

#[derive(Component, Clone, Copy, Default)]
pub struct BarrelPayload {
    pub hp: I32F32,
    pub damage: I32F32,
    pub aoe_tiles: I32F32,
    pub noise: I32F32,
}

#[derive(Event, Clone, Copy)]
pub struct BarrelCarryIntentEvent {
    pub barrel: Entity,
    pub carrier: Entity,
}

#[derive(Event, Clone, Copy)]
pub struct BarrelPlaceIntentEvent {
    pub barrel: Entity,
    pub position: SimPosition,
}

#[derive(Event, Clone, Copy)]
pub struct BarrelExplosionAppliedEvent {
    pub barrel: Entity,
    pub damage: I32F32,
    pub aoe_tiles: I32F32,
    pub affected_targets: u32,
    pub suppress_reinforcements_against_villages_of_doom: bool,
}

#[derive(Resource, Clone, Copy)]
pub struct ExplosiveBarrelMechanicData {
    pub id: &'static str,
    pub name: &'static str,
    pub mechanic_type: &'static str,
    pub how_it_works: &'static str,
    pub techniques: &'static str,
    pub applications: &'static str,
    pub depends_on: &'static [&'static str],
}

impl Default for ExplosiveBarrelMechanicData {
    fn default() -> Self {
        Self {
            id: "explosive_barrel",
            name: "Explosive barrel",
            mechanic_type: "explosive scenery item",
            how_it_works: "When shot or when the carrier dies, the barrel explodes and deals 500 damage in a 3-tile area.",
            techniques: "Selected units can carry the barrel, move it, and place it in a different location.",
            applications: "Area damage against clustered enemies, units, and buildings.",
            depends_on: &[
                "resources",
                "infected",
                "soldier",
                "lucifer",
                "titan",
                "caelus",
                "calliope",
                "villages_of_doom",
            ],
        }
    }
}

#[derive(Resource, Clone, Copy)]
pub struct ExplosiveBarrelMetrics {
    pub barrels_exploded: u64,
    pub total_targets_hit: u64,
    pub total_damage_dispatched: I32F32,
    pub total_noise_emitted: I32F32,
}

impl Default for ExplosiveBarrelMetrics {
    fn default() -> Self {
        Self {
            barrels_exploded: 0,
            total_targets_hit: 0,
            total_damage_dispatched: I32F32::ZERO,
            total_noise_emitted: I32F32::ZERO,
        }
    }
}

fn initialize_explosive_barrels_system(
    mut commands: Commands,
    barrels: Query<(Entity, Option<&Health>, Option<&BarrelPayload>), Added<ExplosiveBarrel>>,
) {
    for (entity, health, payload) in &barrels {
        if health.is_none() {
            commands.entity(entity).insert(Health::full(BARREL_HP));
        } else {
            commands.entity(entity).insert(Health::full(BARREL_HP));
        }

        if payload.is_none() {
            commands.entity(entity).insert(BarrelPayload {
                hp: BARREL_HP,
                damage: BARREL_DAMAGE,
                aoe_tiles: BARREL_AOE_TILES,
                noise: BARREL_NOISE,
            });
        }
    }
}

fn handle_barrel_shot_triggers_system(
    mut commands: Commands,
    mut incoming_damage: EventReader<IncomingDamageEvent>,
    barrels: Query<(), With<ExplosiveBarrel>>,
    already_pending: Query<(), With<PendingBarrelExplosion>>,
) {
    for ev in incoming_damage.read() {
        if barrels.get(ev.target).is_err() {
            continue;
        }

        if already_pending.get(ev.target).is_ok() {
            continue;
        }

        commands.entity(ev.target).insert(PendingBarrelExplosion);
    }
}

fn handle_barrel_carrier_death_triggers_system(
    mut commands: Commands,
    mut killed_events: EventReader<EntityKilledEvent>,
    carried_barrels: Query<(Entity, &BarrelCarried), With<ExplosiveBarrel>>,
    already_pending: Query<(), With<PendingBarrelExplosion>>,
) {
    for killed in killed_events.read() {
        for (barrel_entity, carried) in &carried_barrels {
            if carried.carrier != killed.entity {
                continue;
            }

            if already_pending.get(barrel_entity).is_ok() {
                continue;
            }

            commands.entity(barrel_entity).insert(PendingBarrelExplosion);
        }
    }
}

fn apply_barrel_carry_intents_system(
    mut commands: Commands,
    mut intents: EventReader<BarrelCarryIntentEvent>,
    barrels: Query<(), With<ExplosiveBarrel>>,
    eligible_carriers: Query<(), With<BarrelCarrierEligible>>,
) {
    for ev in intents.read() {
        if barrels.get(ev.barrel).is_err() {
            continue;
        }

        if eligible_carriers.get(ev.carrier).is_err() {
            continue;
        }

        commands
            .entity(ev.barrel)
            .insert(BarrelCarried { carrier: ev.carrier });
    }
}

fn apply_barrel_place_intents_system(
    mut commands: Commands,
    mut intents: EventReader<BarrelPlaceIntentEvent>,
    carried_barrels: Query<(), (With<ExplosiveBarrel>, With<BarrelCarried>)>,
) {
    for ev in intents.read() {
        if carried_barrels.get(ev.barrel).is_err() {
            continue;
        }

        commands
            .entity(ev.barrel)
            .insert(ev.position)
            .remove::<BarrelCarried>();
    }
}

fn explode_pending_barrels_system(
    mut commands: Commands,
    mut metrics: ResMut<ExplosiveBarrelMetrics>,
    mut incoming_damage_writer: EventWriter<IncomingDamageEvent>,
    mut exploded_writer: EventWriter<BarrelExplosionAppliedEvent>,
    barrels: Query<(Entity, &BarrelPayload, Option<&SimPosition>), With<PendingBarrelExplosion>>,
    targets: Query<(Entity, &SimPosition), With<Health>>,
) {
    for (barrel_entity, payload, barrel_pos) in &barrels {
        let Some(origin) = barrel_pos else {
            commands.entity(barrel_entity).despawn();
            metrics.barrels_exploded = metrics.barrels_exploded.saturating_add(1);
            continue;
        };

        let radius_sq = payload.aoe_tiles * payload.aoe_tiles;
        let mut affected: u32 = 0;

        for (target_entity, target_pos) in &targets {
            let dx = target_pos.x - origin.x;
            let dy = target_pos.y - origin.y;
            let dist_sq = dx * dx + dy * dy;
            if dist_sq > radius_sq {
                continue;
            }

            incoming_damage_writer.send(IncomingDamageEvent {
                target: target_entity,
                raw_amount: payload.damage,
                damage_type: DamageType::Standard,
                source: barrel_entity,
            });

            affected = affected.saturating_add(1);
        }

        metrics.barrels_exploded = metrics.barrels_exploded.saturating_add(1);
        metrics.total_targets_hit = metrics.total_targets_hit.saturating_add(affected as u64);
        metrics.total_damage_dispatched =
            metrics.total_damage_dispatched + payload.damage * I32F32::from_num(affected);
        metrics.total_noise_emitted = metrics.total_noise_emitted + payload.noise;

        exploded_writer.send(BarrelExplosionAppliedEvent {
            barrel: barrel_entity,
            damage: payload.damage,
            aoe_tiles: payload.aoe_tiles,
            affected_targets: affected,
            suppress_reinforcements_against_villages_of_doom: true,
        });

        commands.entity(barrel_entity).despawn();
    }
}

fn explosive_barrel_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    metrics: Res<ExplosiveBarrelMetrics>,
    barrels: Query<
        (
            &BarrelPayload,
            Option<&Health>,
            Option<&SimPosition>,
            Option<&BarrelCarried>,
            Option<&PendingBarrelExplosion>,
        ),
        With<ExplosiveBarrel>,
    >,
) {
    checksum.accumulate(BARREL_HP.to_bits() as u64);
    checksum.accumulate(BARREL_AOE_TILES.to_bits() as u64);
    checksum.accumulate(BARREL_DAMAGE.to_bits() as u64);
    checksum.accumulate(BARREL_NOISE.to_bits() as u64);

    checksum.accumulate(metrics.barrels_exploded);
    checksum.accumulate(metrics.total_targets_hit);
    checksum.accumulate(metrics.total_damage_dispatched.to_bits() as u64);
    checksum.accumulate(metrics.total_noise_emitted.to_bits() as u64);

    for (payload, health, pos, carried, pending) in &barrels {
        checksum.accumulate(payload.hp.to_bits() as u64);
        checksum.accumulate(payload.damage.to_bits() as u64);
        checksum.accumulate(payload.aoe_tiles.to_bits() as u64);
        checksum.accumulate(payload.noise.to_bits() as u64);

        if let Some(hp) = health {
            checksum.accumulate(hp.current.to_bits() as u64);
            checksum.accumulate(hp.max.to_bits() as u64);
        } else {
            checksum.accumulate(0);
            checksum.accumulate(0);
        }

        if let Some(p) = pos {
            checksum.accumulate(p.x.to_bits() as u64);
            checksum.accumulate(p.y.to_bits() as u64);
        } else {
            checksum.accumulate(0);
            checksum.accumulate(0);
        }

        checksum.accumulate(u64::from(carried.is_some()));
        checksum.accumulate(u64::from(pending.is_some()));
    }
}

pub struct ExplosiveBarrelPlugin;

impl Plugin for ExplosiveBarrelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ExplosiveBarrelMechanicData>()
            .init_resource::<ExplosiveBarrelMetrics>()
            .add_event::<BarrelCarryIntentEvent>()
            .add_event::<BarrelPlaceIntentEvent>()
            .add_event::<BarrelExplosionAppliedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    initialize_explosive_barrels_system,
                    handle_barrel_shot_triggers_system,
                    handle_barrel_carrier_death_triggers_system,
                    apply_barrel_carry_intents_system,
                    apply_barrel_place_intents_system,
                    explode_pending_barrels_system,
                    explosive_barrel_checksum_system,
                )
                    .chain(),
            );
    }
}