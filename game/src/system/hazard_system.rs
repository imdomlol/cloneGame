// Sources: vault/map_hazard_pages/map_hazards.md, vault/gameplay_mechanics/employee.md
use bevy::prelude::*;
use fixed::types::I16F16;
use rand_core::RngCore;

use crate::sim::{tick_rng, GameSeed, Health, SimChecksumState, SimTick};

const LANDMINE_MAX_COUNT: u16 = 35;
const TURRET_MAX_COUNT: u16 = 35;
const SPIKE_TRAP_MAX_COUNT: u16 = 18;
const LANDMINE_DAMAGE: I16F16 = I16F16::from_bits(1000 << 16);
const TURRET_BULLET_DAMAGE: I16F16 = I16F16::from_bits(35 << 16);
const SPIKE_TRAP_DAMAGE: I16F16 = I16F16::from_bits(1000 << 16);
const TURRET_SPIN_FIRE_TICKS: u16 = 60;
const TERMINAL_DISABLE_TICKS: u16 = 100;
const SPIKE_TIMER_MIN_TENTHS: u32 = 8;
const SPIKE_TIMER_MAX_TENTHS: u32 = 262;
const SPIKE_INTERVAL_SALT: u64 = 0x6d61_705f_6861_7a31;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Default)]
pub enum MapHazardKind {
    #[default]
    Landmine,
    Turret,
    SpikeTrap,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Default)]
pub enum MapHazardTriggerEntity {
    #[default]
    Employee,
    DeadBody,
    Item,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Default)]
pub enum SpikeTrapMode {
    #[default]
    Timer,
    Detection,
}

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct MapHazard {
    pub kind: MapHazardKind,
    pub replicated_id: u64,
    pub disabled_ticks_remaining: u16,
}

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct LandmineHazard {
    pub armed: bool,
    pub occupied: bool,
    pub beep_ticks_remaining: u16,
}

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct TurretHazard {
    pub tracking_target: Option<Entity>,
    pub spin_fire_ticks_remaining: u16,
}

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct SpikeTrapHazard {
    pub mode: SpikeTrapMode,
    pub slam_ticks_remaining: u16,
    pub next_slam_ticks_remaining: u16,
}

#[derive(Event, Clone, Copy, Debug)]
pub struct SpawnMapHazardEvent {
    pub hazard: Entity,
    pub kind: MapHazardKind,
    pub replicated_id: u64,
}

#[derive(Event, Clone, Copy, Debug)]
pub struct MapHazardSpawnCountResolvedEvent {
    pub landmines: u16,
    pub turrets: u16,
    pub spike_traps: u16,
}

#[derive(Event, Clone, Copy, Debug)]
pub struct MapHazardContactEvent {
    pub hazard: Entity,
    pub target: Entity,
    pub trigger_entity: MapHazardTriggerEntity,
}

#[derive(Event, Clone, Copy, Debug)]
pub struct MapHazardMovedOffEvent {
    pub hazard: Entity,
    pub target: Entity,
    pub trigger_entity: MapHazardTriggerEntity,
}

#[derive(Event, Clone, Copy, Debug)]
pub struct MapHazardExplosionEvent {
    pub hazard: Entity,
    pub target: Entity,
    pub trigger_entity: MapHazardTriggerEntity,
}

#[derive(Event, Clone, Copy, Debug)]
pub struct MapHazardDetectedPlayerEvent {
    pub hazard: Entity,
    pub player: Entity,
}

#[derive(Event, Clone, Copy, Debug)]
pub struct MapHazardLineOfSightLostEvent {
    pub hazard: Entity,
    pub player: Entity,
}

#[derive(Event, Clone, Copy, Debug)]
pub struct MapHazardBulletBurstEvent {
    pub hazard: Entity,
    pub target: Entity,
}

#[derive(Event, Clone, Copy, Debug)]
pub struct MapHazardMeleeHitEvent {
    pub hazard: Entity,
    pub attacker: Entity,
}

#[derive(Event, Clone, Copy, Debug)]
pub struct MapHazardSpikeSlamEvent {
    pub hazard: Entity,
    pub target: Option<Entity>,
}

#[derive(Event, Clone, Copy, Debug)]
pub struct MapHazardTerminalDisableRequestedEvent {
    pub hazard: Entity,
}

#[derive(Event, Clone, Copy, Debug)]
pub struct MapHazardTemporarilyDisabledEvent {
    pub hazard: Entity,
    pub duration_ticks: u16,
}

#[derive(Event, Clone, Copy, Debug)]
pub struct MapHazardWarningCueEvent {
    pub hazard: Entity,
    pub kind: MapHazardKind,
}

pub struct HazardSystemPlugin;

impl Plugin for HazardSystemPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnMapHazardEvent>()
            .add_event::<MapHazardSpawnCountResolvedEvent>()
            .add_event::<MapHazardContactEvent>()
            .add_event::<MapHazardMovedOffEvent>()
            .add_event::<MapHazardExplosionEvent>()
            .add_event::<MapHazardDetectedPlayerEvent>()
            .add_event::<MapHazardLineOfSightLostEvent>()
            .add_event::<MapHazardBulletBurstEvent>()
            .add_event::<MapHazardMeleeHitEvent>()
            .add_event::<MapHazardSpikeSlamEvent>()
            .add_event::<MapHazardTerminalDisableRequestedEvent>()
            .add_event::<MapHazardTemporarilyDisabledEvent>()
            .add_event::<MapHazardWarningCueEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_map_hazards,
                    resolve_terminal_disables,
                    tick_disabled_hazards,
                    handle_landmine_contacts,
                    handle_landmine_moved_off,
                    tick_landmine_warning_cues,
                    handle_turret_detection,
                    handle_turret_los_loss,
                    handle_turret_melee_hits,
                    tick_turrets,
                    handle_spike_detection,
                    tick_spike_traps,
                    hazard_system_checksum,
                )
                    .chain(),
            );
    }
}

fn spawn_map_hazards(
    mut commands: Commands,
    mut events: EventReader<SpawnMapHazardEvent>,
    mut counts: EventWriter<MapHazardSpawnCountResolvedEvent>,
) {
    let mut landmines = 0u16;
    let mut turrets = 0u16;
    let mut spike_traps = 0u16;

    for event in events.read() {
        match event.kind {
            MapHazardKind::Landmine => {
                if landmines >= LANDMINE_MAX_COUNT {
                    continue;
                }

                commands.entity(event.hazard).insert((
                    MapHazard {
                        kind: MapHazardKind::Landmine,
                        replicated_id: event.replicated_id,
                        disabled_ticks_remaining: 0,
                    },
                    LandmineHazard {
                        armed: true,
                        occupied: false,
                        beep_ticks_remaining: 20,
                    },
                ));
                landmines += 1;
            }
            MapHazardKind::Turret => {
                if turrets >= TURRET_MAX_COUNT {
                    continue;
                }

                commands.entity(event.hazard).insert((
                    MapHazard {
                        kind: MapHazardKind::Turret,
                        replicated_id: event.replicated_id,
                        disabled_ticks_remaining: 0,
                    },
                    TurretHazard::default(),
                ));
                turrets += 1;
            }
            MapHazardKind::SpikeTrap => {
                if spike_traps >= SPIKE_TRAP_MAX_COUNT {
                    continue;
                }

                commands.entity(event.hazard).insert((
                    MapHazard {
                        kind: MapHazardKind::SpikeTrap,
                        replicated_id: event.replicated_id,
                        disabled_ticks_remaining: 0,
                    },
                    SpikeTrapHazard {
                        mode: SpikeTrapMode::Timer,
                        slam_ticks_remaining: 0,
                        next_slam_ticks_remaining: 20,
                    },
                ));
                spike_traps += 1;
            }
        }
    }

    if landmines != 0 || turrets != 0 || spike_traps != 0 {
        counts.send(MapHazardSpawnCountResolvedEvent {
            landmines,
            turrets,
            spike_traps,
        });
    }
}

fn resolve_terminal_disables(
    mut events: EventReader<MapHazardTerminalDisableRequestedEvent>,
    mut hazards: Query<&mut MapHazard>,
    mut disabled: EventWriter<MapHazardTemporarilyDisabledEvent>,
) {
    for event in events.read() {
        if let Ok(mut hazard) = hazards.get_mut(event.hazard) {
            hazard.disabled_ticks_remaining = TERMINAL_DISABLE_TICKS;
            disabled.send(MapHazardTemporarilyDisabledEvent {
                hazard: event.hazard,
                duration_ticks: TERMINAL_DISABLE_TICKS,
            });
        }
    }
}

fn tick_disabled_hazards(mut hazards: Query<&mut MapHazard>) {
    for mut hazard in &mut hazards {
        hazard.disabled_ticks_remaining = hazard.disabled_ticks_remaining.saturating_sub(1);
    }
}

fn handle_landmine_contacts(
    mut events: EventReader<MapHazardContactEvent>,
    mut hazards: Query<(&MapHazard, &mut LandmineHazard)>,
    mut explosions: EventWriter<MapHazardExplosionEvent>,
    mut health: Query<&mut Health>,
) {
    for event in events.read() {
        let Ok((hazard, mut mine)) = hazards.get_mut(event.hazard) else {
            continue;
        };

        if hazard.disabled_ticks_remaining != 0 || !mine.armed {
            continue;
        }

        mine.occupied = true;
        apply_hazard_damage(event.target, LANDMINE_DAMAGE, &mut health);
        explosions.send(MapHazardExplosionEvent {
            hazard: event.hazard,
            target: event.target,
            trigger_entity: event.trigger_entity,
        });
        mine.armed = false;
    }
}

fn handle_landmine_moved_off(
    mut events: EventReader<MapHazardMovedOffEvent>,
    mut hazards: Query<(&MapHazard, &mut LandmineHazard)>,
    mut explosions: EventWriter<MapHazardExplosionEvent>,
    mut health: Query<&mut Health>,
) {
    for event in events.read() {
        let Ok((hazard, mut mine)) = hazards.get_mut(event.hazard) else {
            continue;
        };

        if hazard.disabled_ticks_remaining != 0 || !mine.armed {
            continue;
        }

        mine.occupied = false;
        apply_hazard_damage(event.target, LANDMINE_DAMAGE, &mut health);
        explosions.send(MapHazardExplosionEvent {
            hazard: event.hazard,
            target: event.target,
            trigger_entity: event.trigger_entity,
        });
        mine.armed = false;
    }
}

fn tick_landmine_warning_cues(
    mut hazards: Query<(Entity, &MapHazard, &mut LandmineHazard)>,
    mut cues: EventWriter<MapHazardWarningCueEvent>,
) {
    for (entity, hazard, mut mine) in &mut hazards {
        if hazard.disabled_ticks_remaining != 0 || !mine.armed {
            continue;
        }

        mine.beep_ticks_remaining = mine.beep_ticks_remaining.saturating_sub(1);
        if mine.beep_ticks_remaining == 0 {
            mine.beep_ticks_remaining = 20;
            cues.send(MapHazardWarningCueEvent {
                hazard: entity,
                kind: MapHazardKind::Landmine,
            });
        }
    }
}

fn handle_turret_detection(
    mut events: EventReader<MapHazardDetectedPlayerEvent>,
    mut hazards: Query<(&MapHazard, &mut TurretHazard)>,
    mut bursts: EventWriter<MapHazardBulletBurstEvent>,
    mut cues: EventWriter<MapHazardWarningCueEvent>,
    mut health: Query<&mut Health>,
) {
    for event in events.read() {
        let Ok((hazard, mut turret)) = hazards.get_mut(event.hazard) else {
            continue;
        };

        if hazard.disabled_ticks_remaining != 0 {
            continue;
        }

        turret.tracking_target = Some(event.player);
        apply_hazard_damage(event.player, TURRET_BULLET_DAMAGE, &mut health);
        cues.send(MapHazardWarningCueEvent {
            hazard: event.hazard,
            kind: MapHazardKind::Turret,
        });
        bursts.send(MapHazardBulletBurstEvent {
            hazard: event.hazard,
            target: event.player,
        });
    }
}

fn handle_turret_los_loss(
    mut events: EventReader<MapHazardLineOfSightLostEvent>,
    mut hazards: Query<&mut TurretHazard>,
) {
    for event in events.read() {
        if let Ok(mut turret) = hazards.get_mut(event.hazard) {
            if turret.tracking_target == Some(event.player) {
                turret.tracking_target = None;
            }
        }
    }
}

fn handle_turret_melee_hits(
    mut events: EventReader<MapHazardMeleeHitEvent>,
    mut hazards: Query<(&MapHazard, &mut TurretHazard)>,
) {
    for event in events.read() {
        let Ok((hazard, mut turret)) = hazards.get_mut(event.hazard) else {
            continue;
        };

        if hazard.disabled_ticks_remaining == 0 {
            turret.spin_fire_ticks_remaining = TURRET_SPIN_FIRE_TICKS;
        }
    }
}

fn tick_turrets(
    mut hazards: Query<(Entity, &MapHazard, &mut TurretHazard)>,
    mut bursts: EventWriter<MapHazardBulletBurstEvent>,
    mut health: Query<&mut Health>,
) {
    for (entity, hazard, mut turret) in &mut hazards {
        if hazard.disabled_ticks_remaining != 0 {
            continue;
        }

        if turret.spin_fire_ticks_remaining != 0 {
            turret.spin_fire_ticks_remaining -= 1;
            if let Some(target) = turret.tracking_target {
                apply_hazard_damage(target, TURRET_BULLET_DAMAGE, &mut health);
                bursts.send(MapHazardBulletBurstEvent {
                    hazard: entity,
                    target,
                });
            }
        }
    }
}

fn handle_spike_detection(
    mut events: EventReader<MapHazardDetectedPlayerEvent>,
    mut hazards: Query<(&MapHazard, &mut SpikeTrapHazard)>,
    mut slams: EventWriter<MapHazardSpikeSlamEvent>,
    mut health: Query<&mut Health>,
) {
    for event in events.read() {
        let Ok((hazard, mut spike)) = hazards.get_mut(event.hazard) else {
            continue;
        };

        if hazard.disabled_ticks_remaining != 0 || spike.mode != SpikeTrapMode::Detection {
            continue;
        }

        spike.slam_ticks_remaining = 10;
        apply_hazard_damage(event.player, SPIKE_TRAP_DAMAGE, &mut health);
        slams.send(MapHazardSpikeSlamEvent {
            hazard: event.hazard,
            target: Some(event.player),
        });
    }
}

fn tick_spike_traps(
    seed: Res<GameSeed>,
    tick: Res<SimTick>,
    mut hazards: Query<(Entity, &MapHazard, &mut SpikeTrapHazard)>,
    mut slams: EventWriter<MapHazardSpikeSlamEvent>,
    mut cues: EventWriter<MapHazardWarningCueEvent>,
) {
    for (entity, hazard, mut spike) in &mut hazards {
        if hazard.disabled_ticks_remaining != 0 {
            continue;
        }

        spike.slam_ticks_remaining = spike.slam_ticks_remaining.saturating_sub(1);

        if spike.mode != SpikeTrapMode::Timer {
            continue;
        }

        spike.next_slam_ticks_remaining = spike.next_slam_ticks_remaining.saturating_sub(1);
        if spike.next_slam_ticks_remaining == 0 {
            spike.slam_ticks_remaining = 10;
            spike.next_slam_ticks_remaining =
                next_spike_interval_ticks(seed.0, tick.0, hazard.replicated_id);
            cues.send(MapHazardWarningCueEvent {
                hazard: entity,
                kind: MapHazardKind::SpikeTrap,
            });
            slams.send(MapHazardSpikeSlamEvent {
                hazard: entity,
                target: None,
            });
        }
    }
}

fn next_spike_interval_ticks(game_seed: u64, tick: u64, replicated_id: u64) -> u16 {
    let mut rng = tick_rng(game_seed, tick, SPIKE_INTERVAL_SALT ^ replicated_id);
    let range = SPIKE_TIMER_MAX_TENTHS - SPIKE_TIMER_MIN_TENTHS + 1;
    let tenths = SPIKE_TIMER_MIN_TENTHS + (rng.next_u32() % range);
    ((tenths * 20 + 9) / 10) as u16
}

fn apply_hazard_damage(target: Entity, amount: I16F16, health: &mut Query<&mut Health>) {
    let Ok(mut target_health) = health.get_mut(target) else {
        return;
    };

    target_health.current = target_health.current.saturating_sub(amount.into());
}

fn hazard_system_checksum(
    hazards: Query<(
        &MapHazard,
        Option<&LandmineHazard>,
        Option<&TurretHazard>,
        Option<&SpikeTrapHazard>,
    )>,
    mut checksum: ResMut<SimChecksumState>,
) {
    for (hazard, landmine, turret, spike) in &hazards {
        checksum.accumulate(hazard.kind as u64);
        checksum.accumulate(hazard.replicated_id);
        checksum.accumulate(hazard.disabled_ticks_remaining as u64);

        if let Some(landmine) = landmine {
            checksum.accumulate(landmine.armed as u64);
            checksum.accumulate(landmine.occupied as u64);
            checksum.accumulate(landmine.beep_ticks_remaining as u64);
        }

        if let Some(turret) = turret {
            checksum.accumulate(turret.tracking_target.is_some() as u64);
            checksum.accumulate(turret.spin_fire_ticks_remaining as u64);
        }

        if let Some(spike) = spike {
            checksum.accumulate(spike.mode as u64);
            checksum.accumulate(spike.slam_ticks_remaining as u64);
            checksum.accumulate(spike.next_slam_ticks_remaining as u64);
        }
    }
}