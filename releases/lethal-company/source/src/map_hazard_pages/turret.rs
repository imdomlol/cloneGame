// Sources: vault/map_hazard_pages/turret.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{
    DamageType, GameSeed, IncomingDamageEvent, SimChecksumState, SimPosition, SimTick,
};

pub const TURRET_ID: &str = "turret";
pub const TURRET_NAME: &str = "Turret";
pub const TURRET_TYPE: &str = "map_hazard_pages";
pub const TURRET_SUBTYPE: &str = "hazard";
pub const TURRET_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Turret";
pub const TURRET_SOURCE_REVISION: u32 = 20476;
pub const TURRET_EXTRACTED_AT: &str = "2026-06-07T00:00:00Z";
pub const TURRET_CONFIDENCE_BASIS_POINTS: u16 = 93;

pub const TURRET_DAMAGE_PER_SHOT: I32F32 = I32F32::lit("50");
pub const TURRET_DAMAGE_TYPE_LABEL: &str = "hitscan_aoe";
pub const TURRET_MODE_COUNT: u8 = 5;

pub const TURRET_DEACTIVATED_TICKS: u32 = 90;
pub const TURRET_DEACTIVATION_COOLDOWN_TICKS: u32 = 90;
pub const TURRET_DETECTION_REVERSE_TICKS: u32 = 140;
pub const TURRET_DETECTION_CHECK_TICKS: u32 = 5;
pub const TURRET_CHARGE_TICKS: u32 = 30;
pub const TURRET_FIRE_INTERVAL_TICKS: u32 = 5;
pub const TURRET_FIRE_LOST_TARGET_HOLD_TICKS: u32 = 40;
pub const TURRET_BERSERK_WINDUP_TICKS: u32 = 26;
pub const TURRET_BERSERK_ACTIVE_TICKS: u32 = 180;

pub const TURRET_DETECTION_ROTATION_SPEED: I32F32 = I32F32::lit("28");
pub const TURRET_CHARGE_ROTATION_SPEED: I32F32 = I32F32::lit("95");
pub const TURRET_BERSERK_ROTATION_SPEED: I32F32 = I32F32::lit("77");

pub const TURRET_DEPENDS_ON: [&str; 8] = [
    "map_hazard",
    "lethal_company",
    "employee",
    "terminal",
    "shovel",
    "stop_sign",
    "yield_sign",
    "extension_ladder",
];

pub const TURRET_OCCURRENCES: [TurretOccurrence; 11] = [
    TurretOccurrence::new("experimentation", 0, 7, 2),
    TurretOccurrence::new("assurance", 0, 11, 3),
    TurretOccurrence::new("vow", 0, 7, 2),
    TurretOccurrence::new("offense", 0, 9, 3),
    TurretOccurrence::new("march", 0, 15, 4),
    TurretOccurrence::new("adamance", 0, 12, 3),
    TurretOccurrence::new("rend", 0, 0, 0),
    TurretOccurrence::new("dine", 0, 18, 3),
    TurretOccurrence::new("titan", 0, 35, 12),
    TurretOccurrence::new("artifice", 0, 10, 3),
    TurretOccurrence::new("embrion", 0, 9, 3),
];

pub const TURRET_BEHAVIORAL_MECHANICS: [TurretBehaviorRule; 16] = [
    TurretBehaviorRule::new(
        "the turret is in detection mode",
        "it rotates at speed 28, reverses direction every 7 seconds, and checks for employees every 0.25 seconds",
    ),
    TurretBehaviorRule::new(
        "an employee is detected in detection mode",
        "the turret switches to charging mode",
    ),
    TurretBehaviorRule::new(
        "the turret is in charging mode",
        "it rotates at speed 95 toward the target and waits 1.5 seconds before entering firing mode",
    ),
    TurretBehaviorRule::new(
        "the target leaves line of sight during charging mode",
        "the turret returns to detection mode",
    ),
    TurretBehaviorRule::new(
        "the turret is in firing mode",
        "it fires every 0.21 seconds and deals 50 damage per shot",
    ),
    TurretBehaviorRule::new(
        "the turret is in firing mode",
        "its firing radius remains larger than its initial detection radius",
    ),
    TurretBehaviorRule::new(
        "the tracked employee leaves line of sight or dies during firing mode",
        "the turret keeps firing for 2.0 seconds",
    ),
    TurretBehaviorRule::new(
        "no employee remains in line of sight after the 2.0-second hold",
        "the turret returns to detection mode",
    ),
    TurretBehaviorRule::new(
        "the turret is deactivated from the ship's terminal",
        "it stays off for 4.5 seconds before turning back on",
    ),
    TurretBehaviorRule::new(
        "the deactivation timer ends",
        "the turret re-enters detection mode and starts a cooldown during which it cannot be turned off again",
    ),
    TurretBehaviorRule::new(
        "the turret is hit by a shovel, stop_sign, or yield_sign while it is not already firing",
        "it enters berserk mode",
    ),
    TurretBehaviorRule::new(
        "the turret enters berserk mode",
        "it waits 1.3 seconds, rotates at speed 77, fires every 0.21 seconds, and remains active for 9 seconds",
    ),
    TurretBehaviorRule::new(
        "the turret fires",
        "it targets employees only and does not target entities",
    ),
    TurretBehaviorRule::new(
        "turret spawn counts are generated",
        "the distribution uses a moon-specific custom curve and is not linear",
    ),
    TurretBehaviorRule::new(
        "an extension_ladder is placed in front of the turret",
        "the ladder can block shots from reaching anything behind it",
    ),
    TurretBehaviorRule::new(
        "you crouch under the bullet path",
        "the turret can still fire at you",
    ),
];

pub struct TurretPlugin;

impl Plugin for TurretPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnTurretEvent>()
            .add_event::<TurretSpawnCountResolvedEvent>()
            .add_event::<TurretEmployeeDetectedEvent>()
            .add_event::<TurretLineOfSightLostEvent>()
            .add_event::<TurretTerminalDeactivationRequestedEvent>()
            .add_event::<TurretTemporarilyDisabledEvent>()
            .add_event::<TurretMeleeHitEvent>()
            .add_event::<TurretModeChangedEvent>()
            .add_event::<TurretShotFiredEvent>()
            .add_event::<TurretShotBlockedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_turret,
                    resolve_turret_spawn_count,
                    turret_terminal_deactivation,
                    turret_disabled_timer,
                    turret_detection_sweep,
                    turret_employee_detection,
                    turret_charge_timer,
                    turret_line_of_sight_lost,
                    turret_firing_timer,
                    turret_berserk_from_melee_hit,
                    turret_berserk_timer,
                    turret_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TurretBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

impl TurretBehaviorRule {
    pub const fn new(condition: &'static str, outcome: &'static str) -> Self {
        Self { condition, outcome }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TurretOccurrence {
    pub moon: &'static str,
    pub min_turrets: u16,
    pub max_turrets: u16,
    pub average_turrets: u16,
}

impl TurretOccurrence {
    pub const fn new(
        moon: &'static str,
        min_turrets: u16,
        max_turrets: u16,
        average_turrets: u16,
    ) -> Self {
        Self {
            moon,
            min_turrets,
            max_turrets,
            average_turrets,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TurretMode {
    #[default]
    Detection,
    Charging,
    Firing,
    Deactivated,
    Berserk,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Turret {
    pub stable_id: u64,
    pub mode: TurretMode,
    pub tracked_employee_stable_id: u64,
    pub rotation_direction: i8,
    pub rotation_speed: I32F32,
    pub reverse_ticks_remaining: u32,
    pub detection_check_ticks_remaining: u32,
    pub charge_ticks_remaining: u32,
    pub fire_ticks_remaining: u32,
    pub lost_target_hold_ticks_remaining: u32,
    pub deactivated_ticks_remaining: u32,
    pub deactivation_cooldown_ticks_remaining: u32,
    pub berserk_windup_ticks_remaining: u32,
    pub berserk_active_ticks_remaining: u32,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TurretEmployeeTarget {
    pub stable_id: u64,
    pub alive: bool,
    pub in_detection_line_of_sight: bool,
    pub in_firing_line_of_sight: bool,
    pub crouching: bool,
    pub blocked_by_extension_ladder: bool,
}

#[derive(Bundle, Clone, Copy, Debug)]
pub struct TurretBundle {
    pub turret: Turret,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnTurretEvent {
    pub stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TurretSpawnCountResolvedEvent {
    pub moon: &'static str,
    pub min_turrets: u16,
    pub max_turrets: u16,
    pub average_turrets: u16,
    pub resolved_turrets: u16,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TurretEmployeeDetectedEvent {
    pub turret: Entity,
    pub employee: Entity,
    pub employee_stable_id: u64,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TurretLineOfSightLostEvent {
    pub turret: Entity,
    pub employee_stable_id: u64,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TurretTerminalDeactivationRequestedEvent {
    pub turret: Entity,
    pub source_id: &'static str,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TurretTemporarilyDisabledEvent {
    pub turret: Entity,
    pub duration_ticks: u32,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TurretMeleeHitEvent {
    pub turret: Entity,
    pub weapon_id: &'static str,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TurretModeChangedEvent {
    pub turret: Entity,
    pub stable_id: u64,
    pub from: TurretMode,
    pub to: TurretMode,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TurretShotFiredEvent {
    pub turret: Entity,
    pub target: Entity,
    pub target_stable_id: u64,
    pub damage: I32F32,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TurretShotBlockedEvent {
    pub turret: Entity,
    pub target: Entity,
    pub blocker_id: &'static str,
}

fn spawn_turret(mut commands: Commands, mut events: EventReader<SpawnTurretEvent>) {
    for event in events.read() {
        commands.spawn(TurretBundle {
            turret: Turret {
                stable_id: event.stable_id,
                mode: TurretMode::Detection,
                tracked_employee_stable_id: 0,
                rotation_direction: 1,
                rotation_speed: TURRET_DETECTION_ROTATION_SPEED,
                reverse_ticks_remaining: TURRET_DETECTION_REVERSE_TICKS,
                detection_check_ticks_remaining: TURRET_DETECTION_CHECK_TICKS,
                charge_ticks_remaining: 0,
                fire_ticks_remaining: 0,
                lost_target_hold_ticks_remaining: 0,
                deactivated_ticks_remaining: 0,
                deactivation_cooldown_ticks_remaining: 0,
                berserk_windup_ticks_remaining: 0,
                berserk_active_ticks_remaining: 0,
            },
            position: event.position,
        });
    }
}

fn resolve_turret_spawn_count(mut events: EventWriter<TurretSpawnCountResolvedEvent>) {
    for occurrence in TURRET_OCCURRENCES {
        events.send(TurretSpawnCountResolvedEvent {
            moon: occurrence.moon,
            min_turrets: occurrence.min_turrets,
            max_turrets: occurrence.max_turrets,
            average_turrets: occurrence.average_turrets,
            resolved_turrets: occurrence.average_turrets,
        });
    }
}

fn turret_terminal_deactivation(
    mut requests: EventReader<TurretTerminalDeactivationRequestedEvent>,
    mut turrets: Query<&mut Turret>,
    mut disabled: EventWriter<TurretTemporarilyDisabledEvent>,
    mut changed: EventWriter<TurretModeChangedEvent>,
) {
    for request in requests.read() {
        if request.source_id != "terminal" {
            continue;
        }

        if let Ok(mut turret) = turrets.get_mut(request.turret) {
            if turret.mode == TurretMode::Firing
                || turret.mode == TurretMode::Deactivated
                || turret.deactivation_cooldown_ticks_remaining > 0
            {
                continue;
            }

            let from = turret.mode;
            turret.mode = TurretMode::Deactivated;
            turret.tracked_employee_stable_id = 0;
            turret.deactivated_ticks_remaining = TURRET_DEACTIVATED_TICKS;
            turret.charge_ticks_remaining = 0;
            turret.fire_ticks_remaining = 0;
            turret.lost_target_hold_ticks_remaining = 0;

            disabled.send(TurretTemporarilyDisabledEvent {
                turret: request.turret,
                duration_ticks: TURRET_DEACTIVATED_TICKS,
            });
            changed.send(TurretModeChangedEvent {
                turret: request.turret,
                stable_id: turret.stable_id,
                from,
                to: TurretMode::Deactivated,
            });
        }
    }
}

fn turret_disabled_timer(
    mut turrets: Query<(Entity, &mut Turret)>,
    mut changed: EventWriter<TurretModeChangedEvent>,
) {
    for (entity, mut turret) in &mut turrets {
        if turret.deactivation_cooldown_ticks_remaining > 0 {
            turret.deactivation_cooldown_ticks_remaining -= 1;
        }

        if turret.mode != TurretMode::Deactivated {
            continue;
        }

        if turret.deactivated_ticks_remaining > 0 {
            turret.deactivated_ticks_remaining -= 1;
        }

        if turret.deactivated_ticks_remaining == 0 {
            let from = turret.mode;
            turret.mode = TurretMode::Detection;
            turret.rotation_speed = TURRET_DETECTION_ROTATION_SPEED;
            turret.reverse_ticks_remaining = TURRET_DETECTION_REVERSE_TICKS;
            turret.detection_check_ticks_remaining = TURRET_DETECTION_CHECK_TICKS;
            turret.deactivation_cooldown_ticks_remaining = TURRET_DEACTIVATION_COOLDOWN_TICKS;

            changed.send(TurretModeChangedEvent {
                turret: entity,
                stable_id: turret.stable_id,
                from,
                to: TurretMode::Detection,
            });
        }
    }
}

fn turret_detection_sweep(mut turrets: Query<&mut Turret>) {
    for mut turret in &mut turrets {
        if turret.mode != TurretMode::Detection {
            continue;
        }

        turret.rotation_speed = TURRET_DETECTION_ROTATION_SPEED;

        if turret.reverse_ticks_remaining > 0 {
            turret.reverse_ticks_remaining -= 1;
        }

        if turret.reverse_ticks_remaining == 0 {
            turret.rotation_direction *= -1;
            turret.reverse_ticks_remaining = TURRET_DETECTION_REVERSE_TICKS;
        }

        if turret.detection_check_ticks_remaining > 0 {
            turret.detection_check_ticks_remaining -= 1;
        }
    }
}

fn turret_employee_detection(
    mut turrets: Query<(Entity, &mut Turret)>,
    employees: Query<(Entity, &TurretEmployeeTarget)>,
    mut detected: EventWriter<TurretEmployeeDetectedEvent>,
    mut changed: EventWriter<TurretModeChangedEvent>,
) {
    let mut sorted_employees: Vec<(u64, Entity, TurretEmployeeTarget)> = employees
        .iter()
        .map(|(entity, target)| (target.stable_id, entity, *target))
        .collect();
    sorted_employees.sort_by_key(|(stable_id, _, _)| *stable_id);

    for (turret_entity, mut turret) in &mut turrets {
        if turret.mode != TurretMode::Detection || turret.detection_check_ticks_remaining > 0 {
            continue;
        }

        turret.detection_check_ticks_remaining = TURRET_DETECTION_CHECK_TICKS;

        for (_, employee_entity, target) in &sorted_employees {
            if target.alive && target.in_detection_line_of_sight {
                let from = turret.mode;
                turret.mode = TurretMode::Charging;
                turret.tracked_employee_stable_id = target.stable_id;
                turret.rotation_speed = TURRET_CHARGE_ROTATION_SPEED;
                turret.charge_ticks_remaining = TURRET_CHARGE_TICKS;

                detected.send(TurretEmployeeDetectedEvent {
                    turret: turret_entity,
                    employee: *employee_entity,
                    employee_stable_id: target.stable_id,
                });
                changed.send(TurretModeChangedEvent {
                    turret: turret_entity,
                    stable_id: turret.stable_id,
                    from,
                    to: TurretMode::Charging,
                });
                break;
            }
        }
    }
}

fn turret_charge_timer(
    mut turrets: Query<(Entity, &mut Turret)>,
    employees: Query<&TurretEmployeeTarget>,
    mut changed: EventWriter<TurretModeChangedEvent>,
) {
    let mut sorted_targets: Vec<TurretEmployeeTarget> =
        employees.iter().map(|target| *target).collect();
    sorted_targets.sort_by_key(|target| target.stable_id);

    for (entity, mut turret) in &mut turrets {
        if turret.mode != TurretMode::Charging {
            continue;
        }

        let target_visible = sorted_targets.iter().any(|target| {
            target.stable_id == turret.tracked_employee_stable_id
                && target.alive
                && target.in_detection_line_of_sight
        });

        if !target_visible {
            let from = turret.mode;
            turret.mode = TurretMode::Detection;
            turret.tracked_employee_stable_id = 0;
            turret.rotation_speed = TURRET_DETECTION_ROTATION_SPEED;
            turret.charge_ticks_remaining = 0;
            changed.send(TurretModeChangedEvent {
                turret: entity,
                stable_id: turret.stable_id,
                from,
                to: TurretMode::Detection,
            });
            continue;
        }

        if turret.charge_ticks_remaining > 0 {
            turret.charge_ticks_remaining -= 1;
        }

        if turret.charge_ticks_remaining == 0 {
            let from = turret.mode;
            turret.mode = TurretMode::Firing;
            turret.fire_ticks_remaining = 0;
            turret.lost_target_hold_ticks_remaining = TURRET_FIRE_LOST_TARGET_HOLD_TICKS;
            changed.send(TurretModeChangedEvent {
                turret: entity,
                stable_id: turret.stable_id,
                from,
                to: TurretMode::Firing,
            });
        }
    }
}

fn turret_line_of_sight_lost(
    mut turrets: Query<(Entity, &mut Turret)>,
    employees: Query<&TurretEmployeeTarget>,
    mut lost: EventWriter<TurretLineOfSightLostEvent>,
) {
    let mut sorted_targets: Vec<TurretEmployeeTarget> =
        employees.iter().map(|target| *target).collect();
    sorted_targets.sort_by_key(|target| target.stable_id);

    for (entity, mut turret) in &mut turrets {
        if turret.mode != TurretMode::Firing {
            continue;
        }

        let target_visible = sorted_targets.iter().any(|target| {
            target.stable_id == turret.tracked_employee_stable_id
                && target.alive
                && target.in_firing_line_of_sight
        });

        if target_visible {
            turret.lost_target_hold_ticks_remaining = TURRET_FIRE_LOST_TARGET_HOLD_TICKS;
        } else if turret.lost_target_hold_ticks_remaining > 0 {
            turret.lost_target_hold_ticks_remaining -= 1;
            lost.send(TurretLineOfSightLostEvent {
                turret: entity,
                employee_stable_id: turret.tracked_employee_stable_id,
            });
        }
    }
}

fn turret_firing_timer(
    mut turrets: Query<(Entity, &mut Turret)>,
    employees: Query<(Entity, &TurretEmployeeTarget)>,
    mut damage: EventWriter<IncomingDamageEvent>,
    mut fired: EventWriter<TurretShotFiredEvent>,
    mut blocked: EventWriter<TurretShotBlockedEvent>,
    mut changed: EventWriter<TurretModeChangedEvent>,
) {
    let mut sorted_targets: Vec<(u64, Entity, TurretEmployeeTarget)> = employees
        .iter()
        .map(|(entity, target)| (target.stable_id, entity, *target))
        .collect();
    sorted_targets.sort_by_key(|(stable_id, _, _)| *stable_id);

    for (turret_entity, mut turret) in &mut turrets {
        if turret.mode != TurretMode::Firing {
            continue;
        }

        if turret.fire_ticks_remaining > 0 {
            turret.fire_ticks_remaining -= 1;
        }

        if turret.fire_ticks_remaining == 0 {
            turret.fire_ticks_remaining = TURRET_FIRE_INTERVAL_TICKS;

            for (_, employee_entity, target) in &sorted_targets {
                if target.stable_id != turret.tracked_employee_stable_id
                    || !target.alive
                    || !target.in_firing_line_of_sight
                {
                    continue;
                }

                if target.blocked_by_extension_ladder {
                    blocked.send(TurretShotBlockedEvent {
                        turret: turret_entity,
                        target: *employee_entity,
                        blocker_id: "extension_ladder",
                    });
                } else {
                    damage.send(IncomingDamageEvent {
                        target: *employee_entity,
                        raw_amount: TURRET_DAMAGE_PER_SHOT,
                        damage_type: DamageType::Standard,
                        source: turret_entity,
                    });
                    fired.send(TurretShotFiredEvent {
                        turret: turret_entity,
                        target: *employee_entity,
                        target_stable_id: target.stable_id,
                        damage: TURRET_DAMAGE_PER_SHOT,
                    });
                }

                break;
            }
        }

        if turret.lost_target_hold_ticks_remaining == 0 {
            let from = turret.mode;
            turret.mode = TurretMode::Detection;
            turret.tracked_employee_stable_id = 0;
            turret.rotation_speed = TURRET_DETECTION_ROTATION_SPEED;
            turret.reverse_ticks_remaining = TURRET_DETECTION_REVERSE_TICKS;
            turret.detection_check_ticks_remaining = TURRET_DETECTION_CHECK_TICKS;
            changed.send(TurretModeChangedEvent {
                turret: turret_entity,
                stable_id: turret.stable_id,
                from,
                to: TurretMode::Detection,
            });
        }
    }
}

fn turret_berserk_from_melee_hit(
    mut hits: EventReader<TurretMeleeHitEvent>,
    mut turrets: Query<&mut Turret>,
    mut changed: EventWriter<TurretModeChangedEvent>,
) {
    for hit in hits.read() {
        if hit.weapon_id != "shovel" && hit.weapon_id != "stop_sign" && hit.weapon_id != "yield_sign"
        {
            continue;
        }

        if let Ok(mut turret) = turrets.get_mut(hit.turret) {
            if turret.mode == TurretMode::Firing || turret.mode == TurretMode::Berserk {
                continue;
            }

            let from = turret.mode;
            turret.mode = TurretMode::Berserk;
            turret.tracked_employee_stable_id = 0;
            turret.rotation_speed = TURRET_BERSERK_ROTATION_SPEED;
            turret.berserk_windup_ticks_remaining = TURRET_BERSERK_WINDUP_TICKS;
            turret.berserk_active_ticks_remaining = TURRET_BERSERK_ACTIVE_TICKS;
            turret.fire_ticks_remaining = TURRET_FIRE_INTERVAL_TICKS;

            changed.send(TurretModeChangedEvent {
                turret: hit.turret,
                stable_id: turret.stable_id,
                from,
                to: TurretMode::Berserk,
            });
        }
    }
}

fn turret_berserk_timer(
    mut turrets: Query<(Entity, &mut Turret)>,
    employees: Query<(Entity, &TurretEmployeeTarget)>,
    mut damage: EventWriter<IncomingDamageEvent>,
    mut fired: EventWriter<TurretShotFiredEvent>,
    mut blocked: EventWriter<TurretShotBlockedEvent>,
    mut changed: EventWriter<TurretModeChangedEvent>,
) {
    let mut sorted_targets: Vec<(u64, Entity, TurretEmployeeTarget)> = employees
        .iter()
        .map(|(entity, target)| (target.stable_id, entity, *target))
        .collect();
    sorted_targets.sort_by_key(|(stable_id, _, _)| *stable_id);

    for (turret_entity, mut turret) in &mut turrets {
        if turret.mode != TurretMode::Berserk {
            continue;
        }

        turret.rotation_speed = TURRET_BERSERK_ROTATION_SPEED;

        if turret.berserk_windup_ticks_remaining > 0 {
            turret.berserk_windup_ticks_remaining -= 1;
            continue;
        }

        if turret.berserk_active_ticks_remaining > 0 {
            turret.berserk_active_ticks_remaining -= 1;
        }

        if turret.fire_ticks_remaining > 0 {
            turret.fire_ticks_remaining -= 1;
        }

        if turret.fire_ticks_remaining == 0 {
            turret.fire_ticks_remaining = TURRET_FIRE_INTERVAL_TICKS;

            for (_, employee_entity, target) in &sorted_targets {
                if !target.alive || !target.in_firing_line_of_sight {
                    continue;
                }

                if target.blocked_by_extension_ladder {
                    blocked.send(TurretShotBlockedEvent {
                        turret: turret_entity,
                        target: *employee_entity,
                        blocker_id: "extension_ladder",
                    });
                } else {
                    damage.send(IncomingDamageEvent {
                        target: *employee_entity,
                        raw_amount: TURRET_DAMAGE_PER_SHOT,
                        damage_type: DamageType::Standard,
                        source: turret_entity,
                    });
                    fired.send(TurretShotFiredEvent {
                        turret: turret_entity,
                        target: *employee_entity,
                        target_stable_id: target.stable_id,
                        damage: TURRET_DAMAGE_PER_SHOT,
                    });
                }

                break;
            }
        }

        if turret.berserk_active_ticks_remaining == 0 {
            let from = turret.mode;
            turret.mode = TurretMode::Detection;
            turret.tracked_employee_stable_id = 0;
            turret.rotation_speed = TURRET_DETECTION_ROTATION_SPEED;
            turret.reverse_ticks_remaining = TURRET_DETECTION_REVERSE_TICKS;
            turret.detection_check_ticks_remaining = TURRET_DETECTION_CHECK_TICKS;
            changed.send(TurretModeChangedEvent {
                turret: turret_entity,
                stable_id: turret.stable_id,
                from,
                to: TurretMode::Detection,
            });
        }
    }
}

fn turret_checksum(
    mut checksum: ResMut<SimChecksumState>,
    tick: Res<SimTick>,
    seed: Res<GameSeed>,
    turrets: Query<(&Turret, &SimPosition)>,
) {
    checksum.accumulate(tick.0);
    checksum.accumulate(seed.0);
    checksum.accumulate(TURRET_SOURCE_REVISION as u64);
    checksum.accumulate(TURRET_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(TURRET_DAMAGE_PER_SHOT.to_bits() as u64);
    checksum.accumulate(TURRET_MODE_COUNT as u64);
    checksum.accumulate(TURRET_DEACTIVATED_TICKS as u64);
    checksum.accumulate(TURRET_DEACTIVATION_COOLDOWN_TICKS as u64);
    checksum.accumulate(TURRET_DETECTION_REVERSE_TICKS as u64);
    checksum.accumulate(TURRET_DETECTION_CHECK_TICKS as u64);
    checksum.accumulate(TURRET_CHARGE_TICKS as u64);
    checksum.accumulate(TURRET_FIRE_INTERVAL_TICKS as u64);
    checksum.accumulate(TURRET_FIRE_LOST_TARGET_HOLD_TICKS as u64);
    checksum.accumulate(TURRET_BERSERK_WINDUP_TICKS as u64);
    checksum.accumulate(TURRET_BERSERK_ACTIVE_TICKS as u64);
    checksum.accumulate(TURRET_DETECTION_ROTATION_SPEED.to_bits() as u64);
    checksum.accumulate(TURRET_CHARGE_ROTATION_SPEED.to_bits() as u64);
    checksum.accumulate(TURRET_BERSERK_ROTATION_SPEED.to_bits() as u64);

    for dependency in TURRET_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x1000, dependency);
    }

    for rule in TURRET_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x2000, rule.condition);
        accumulate_str(&mut checksum, 0x2001, rule.outcome);
    }

    for occurrence in TURRET_OCCURRENCES {
        accumulate_str(&mut checksum, 0x3000, occurrence.moon);
        checksum.accumulate(occurrence.min_turrets as u64);
        checksum.accumulate(occurrence.max_turrets as u64);
        checksum.accumulate(occurrence.average_turrets as u64);
    }

    for (turret, position) in &turrets {
        checksum.accumulate(turret.stable_id);
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(turret.mode as u64);
        checksum.accumulate(turret.tracked_employee_stable_id);
        checksum.accumulate(turret.rotation_direction as u64);
        checksum.accumulate(turret.rotation_speed.to_bits() as u64);
        checksum.accumulate(turret.reverse_ticks_remaining as u64);
        checksum.accumulate(turret.detection_check_ticks_remaining as u64);
        checksum.accumulate(turret.charge_ticks_remaining as u64);
        checksum.accumulate(turret.fire_ticks_remaining as u64);
        checksum.accumulate(turret.lost_target_hold_ticks_remaining as u64);
        checksum.accumulate(turret.deactivated_ticks_remaining as u64);
        checksum.accumulate(turret.deactivation_cooldown_ticks_remaining as u64);
        checksum.accumulate(turret.berserk_windup_ticks_remaining as u64);
        checksum.accumulate(turret.berserk_active_ticks_remaining as u64);
    }
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt ^ value.len() as u64);

    for (index, byte) in value.bytes().enumerate() {
        checksum.accumulate(salt ^ ((index as u64) << 8) ^ byte as u64);
    }
}