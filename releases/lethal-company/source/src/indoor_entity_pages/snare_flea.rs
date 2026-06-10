// Sources: vault/indoor_entity_pages/snare_flea.md
use bevy::prelude::*;
use fixed::types::I32F32;

use crate::gameplay_mechanics::scanner::ScannerActivationEvent;
use crate::sim::{
    DamageType, Health, IncomingDamageEvent, SimChecksumState, SimHz, SimPosition, UnitStats,
};

pub const SNARE_FLEA_ID: &str = "snare_flea";
pub const SNARE_FLEA_NAME: &str = "Snare Flea";
pub const SNARE_FLEA_TYPE: &str = "indoor_entity_pages";
pub const SNARE_FLEA_SUBTYPE: &str = "creature";
pub const SNARE_FLEA_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Snare_Flea";
pub const SNARE_FLEA_SOURCE_REVISION: u32 = 20526;
pub const SNARE_FLEA_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const SNARE_FLEA_CONFIDENCE_BASIS_POINTS: u16 = 86;

pub const SNARE_FLEA_DWELLS: &str = "Inside";
pub const SNARE_FLEA_SCIENTIFIC_NAME: &str = "Dolus-scolopendra";
pub const SNARE_FLEA_ATTACK_DAMAGE: I32F32 = I32F32::lit("10");
pub const SNARE_FLEA_ATTACK_SPEED: I32F32 = I32F32::lit("0.5");
pub const SNARE_FLEA_DPS: I32F32 = I32F32::lit("5");
pub const SNARE_FLEA_POWER_LEVEL: I32F32 = I32F32::lit("1");
pub const SNARE_FLEA_MAX_SPAWNED: usize = 4;
pub const SNARE_FLEA_SHOVEL_HP: I32F32 = I32F32::lit("3");
pub const SNARE_FLEA_DOOR_OPEN_SPEED: I32F32 = I32F32::lit("0.23");
pub const SNARE_FLEA_STUN_MULTIPLIER: I32F32 = I32F32::lit("3");
pub const SNARE_FLEA_RADAR_PIP_SIZE: &str = "Small";
pub const SNARE_FLEA_SHOCK_RESPONSE: &str = "Susceptible";
pub const SNARE_FLEA_ZAP_GUN_DIFFICULTY: I32F32 = I32F32::lit("0.6");
pub const SNARE_FLEA_INTERNAL_NAME: &str = "Centipede";

pub const SNARE_FLEA_ROAM_SPEED: I32F32 = I32F32::lit("1");
pub const SNARE_FLEA_ATTACH_RANGE: I32F32 = I32F32::lit("1");
pub const SNARE_FLEA_WATCH_RANGE: I32F32 = I32F32::lit("8");

pub const SNARE_FLEA_DEPENDS_ON: [&str; 11] = [
    "lethal_company",
    "employee",
    "scanner",
    "shovel",
    "stop_sign",
    "yield_sign",
    "double_barrel",
    "kitchen_knife",
    "stun_grenade",
    "zap_gun",
    "teleporter",
];

pub const SNARE_FLEA_FRONTMATTER_BEHAVIOR: [&str; 2] = ["Roaming", "Trapping"];

pub const SNARE_FLEA_BEHAVIORAL_MECHANICS: [SnareFleaBehaviorRule; 10] = [
    SnareFleaBehaviorRule {
        condition: "an employee moves beneath or near the Snare Flea",
        outcome: "it drops from the ceiling and attempts to attach to the head",
    },
    SnareFleaBehaviorRule {
        condition: "the Snare Flea is attached",
        outcome: "it muffles the victim's voice and suffocates them at attack_damage 10, attack_speed 0.5, and dps 5",
    },
    SnareFleaBehaviorRule {
        condition: "the victim takes damage, leaves the facility, or dies",
        outcome: "the Snare Flea detaches and returns to the ceiling",
    },
    SnareFleaBehaviorRule {
        condition: "the player is in singleplayer and reaches critical health",
        outcome: "the Snare Flea detaches automatically",
    },
    SnareFleaBehaviorRule {
        condition: "the Snare Flea interacts with a door",
        outcome: "it opens the door with a door_open_speed of 0.23",
    },
    SnareFleaBehaviorRule {
        condition: "a player hits it with a shovel, stop sign, yield sign, double barrel, kitchen knife, or stun grenade",
        outcome: "it can be knocked off the target",
    },
    SnareFleaBehaviorRule {
        condition: "a player uses a zap gun",
        outcome: "it can be dislodged from a coworker's head, with a zap_gun_difficulty of 0.6",
    },
    SnareFleaBehaviorRule {
        condition: "the target exits the facility or is moved by a teleporter",
        outcome: "the Snare Flea dies immediately",
    },
    SnareFleaBehaviorRule {
        condition: "the scanner is used",
        outcome: "the Snare Flea's ceiling position can be identified",
    },
    SnareFleaBehaviorRule {
        condition: "the Snare Flea is stunned",
        outcome: "its stun_multiplier is 3 and its hitbox may become unreliable, making it harder to kill",
    },
];

pub struct SnareFleaPlugin;

impl Plugin for SnareFleaPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnSnareFleaEvent>()
            .add_event::<SnareFleaStateChangedEvent>()
            .add_event::<SnareFleaAttachedEvent>()
            .add_event::<SnareFleaDetachedEvent>()
            .add_event::<SnareFleaMuffledVoiceEvent>()
            .add_event::<SnareFleaDoorAttemptEvent>()
            .add_event::<SnareFleaDoorAttemptResolvedEvent>()
            .add_event::<SnareFleaWeaponKnockoffEvent>()
            .add_event::<SnareFleaZapGunTargetedEvent>()
            .add_event::<SnareFleaZapGunDifficultyEvent>()
            .add_event::<SnareFleaKilledByFacilityTransitionEvent>()
            .add_event::<SnareFleaScannerIdentifiedEvent>()
            .add_event::<SnareFleaStunAppliedEvent>()
            .add_event::<SnareFleaStunAdjustedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_snare_flea,
                    snare_flea_drop_and_attach,
                    snare_flea_suffocate_attached,
                    snare_flea_detach_on_victim_damage_exit_or_death,
                    snare_flea_singleplayer_critical_detach,
                    snare_flea_door_attempt_speed,
                    snare_flea_weapon_knockoff,
                    snare_flea_zap_gun_dislodge,
                    snare_flea_kill_on_facility_exit_or_teleporter,
                    snare_flea_scanner_identifies_ceiling_position,
                    snare_flea_apply_stun_multiplier,
                    snare_flea_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SnareFleaBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SnareFlea;

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SnareFleaEmployeeSensor {
    pub stable_id: u64,
    pub is_beneath_or_near: bool,
    pub victim_took_damage: bool,
    pub victim_left_facility: bool,
    pub victim_died: bool,
    pub is_singleplayer: bool,
    pub is_critical_health: bool,
    pub moved_by_teleporter: bool,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SnareFleaHitSensor {
    pub hit_by_shovel: bool,
    pub hit_by_stop_sign: bool,
    pub hit_by_yield_sign: bool,
    pub hit_by_double_barrel: bool,
    pub hit_by_kitchen_knife: bool,
    pub hit_by_stun_grenade: bool,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SnareFleaCeilingAnchor {
    pub position: SimPosition,
}

impl Default for SnareFleaCeilingAnchor {
    fn default() -> Self {
        Self {
            position: SimPosition {
                x: I32F32::lit("0"),
                y: I32F32::lit("0"),
            },
        }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SnareFleaAttachment {
    pub has_victim: bool,
    pub victim: Entity,
    pub victim_stable_id: u64,
    pub damage_timer_ticks: u32,
    pub voice_muffled: bool,
}

impl Default for SnareFleaAttachment {
    fn default() -> Self {
        Self {
            has_victim: false,
            victim: Entity::PLACEHOLDER,
            victim_stable_id: 0,
            damage_timer_ticks: 1,
            voice_muffled: false,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SnareFleaState {
    #[default]
    Roaming,
    Trapping,
}

#[derive(Bundle)]
pub struct SnareFleaBundle {
    pub name: Name,
    pub snare_flea: SnareFlea,
    pub position: SimPosition,
    pub health: Health,
    pub stats: UnitStats,
    pub state: SnareFleaState,
    pub ceiling_anchor: SnareFleaCeilingAnchor,
    pub attachment: SnareFleaAttachment,
    pub hit_sensor: SnareFleaHitSensor,
}

impl SnareFleaBundle {
    pub fn new(event: SpawnSnareFleaEvent) -> Self {
        Self {
            name: Name::new(SNARE_FLEA_NAME),
            snare_flea: SnareFlea,
            position: event.ceiling_position,
            health: Health::full(SNARE_FLEA_SHOVEL_HP),
            stats: UnitStats {
                move_speed: SNARE_FLEA_ROAM_SPEED,
                attack_range: SNARE_FLEA_ATTACH_RANGE,
                attack_damage: SNARE_FLEA_ATTACK_DAMAGE,
                attack_speed: SNARE_FLEA_ATTACK_SPEED,
                watch_range: SNARE_FLEA_WATCH_RANGE,
            },
            state: SnareFleaState::Roaming,
            ceiling_anchor: SnareFleaCeilingAnchor {
                position: event.ceiling_position,
            },
            attachment: SnareFleaAttachment::default(),
            hit_sensor: SnareFleaHitSensor::default(),
        }
    }
}

#[derive(Event, Clone, Copy, Debug)]
pub struct SpawnSnareFleaEvent {
    pub ceiling_position: SimPosition,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SnareFleaStateChangedEvent {
    pub snare_flea: Entity,
    pub from: SnareFleaState,
    pub to: SnareFleaState,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SnareFleaAttachedEvent {
    pub snare_flea: Entity,
    pub victim: Entity,
    pub victim_stable_id: u64,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SnareFleaDetachedEvent {
    pub snare_flea: Entity,
    pub victim: Entity,
    pub victim_stable_id: u64,
    pub reason: SnareFleaDetachReason,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SnareFleaMuffledVoiceEvent {
    pub snare_flea: Entity,
    pub victim: Entity,
    pub victim_stable_id: u64,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SnareFleaDoorAttemptEvent {
    pub snare_flea: Entity,
    pub door: Entity,
    pub base_open_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SnareFleaDoorAttemptResolvedEvent {
    pub snare_flea: Entity,
    pub door: Entity,
    pub adjusted_open_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SnareFleaWeaponKnockoffEvent {
    pub snare_flea: Entity,
    pub victim: Entity,
    pub victim_stable_id: u64,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SnareFleaZapGunTargetedEvent {
    pub snare_flea: Entity,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SnareFleaZapGunDifficultyEvent {
    pub snare_flea: Entity,
    pub difficulty_modifier: I32F32,
    pub dislodged: bool,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SnareFleaKilledByFacilityTransitionEvent {
    pub snare_flea: Entity,
    pub victim: Entity,
    pub victim_stable_id: u64,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SnareFleaScannerIdentifiedEvent {
    pub snare_flea: Entity,
    pub ceiling_position: SimPosition,
    pub radar_pip_size: &'static str,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SnareFleaStunAppliedEvent {
    pub snare_flea: Entity,
    pub base_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SnareFleaStunAdjustedEvent {
    pub snare_flea: Entity,
    pub base_ticks: u32,
    pub adjusted_ticks: u32,
    pub hitbox_unreliable: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SnareFleaDetachReason {
    VictimDamaged,
    VictimLeftFacility,
    VictimDied,
    SingleplayerCriticalHealth,
    WeaponKnockoff,
    ZapGun,
}

fn spawn_snare_flea(
    mut commands: Commands,
    mut events: EventReader<SpawnSnareFleaEvent>,
    snare_fleas: Query<(), With<SnareFlea>>,
) {
    let mut spawned_count = snare_fleas.iter().count();

    for event in events.read() {
        if spawned_count >= SNARE_FLEA_MAX_SPAWNED {
            break;
        }

        commands.spawn(SnareFleaBundle::new(*event));
        spawned_count += 1;
    }
}

fn snare_flea_drop_and_attach(
    mut state_events: EventWriter<SnareFleaStateChangedEvent>,
    mut attached_events: EventWriter<SnareFleaAttachedEvent>,
    mut muffled_events: EventWriter<SnareFleaMuffledVoiceEvent>,
    mut snare_fleas: Query<
        (
            Entity,
            &mut SimPosition,
            &mut SnareFleaState,
            &mut SnareFleaAttachment,
        ),
        With<SnareFlea>,
    >,
    employees: Query<(Entity, &SimPosition, &SnareFleaEmployeeSensor), Without<SnareFlea>>,
) {
    let Some((employee, employee_position, employee_sensor)) = nearest_triggering_employee(&employees) else {
        return;
    };

    for (snare_flea, mut position, mut state, mut attachment) in snare_fleas.iter_mut() {
        if *state != SnareFleaState::Roaming || attachment.has_victim {
            continue;
        }

        position.x = employee_position.x;
        position.y = employee_position.y;
        attachment.has_victim = true;
        attachment.victim = employee;
        attachment.victim_stable_id = employee_sensor.stable_id;
        attachment.voice_muffled = true;

        set_snare_flea_state(
            snare_flea,
            &mut state,
            SnareFleaState::Trapping,
            &mut state_events,
        );

        attached_events.send(SnareFleaAttachedEvent {
            snare_flea,
            victim: employee,
            victim_stable_id: employee_sensor.stable_id,
        });
        muffled_events.send(SnareFleaMuffledVoiceEvent {
            snare_flea,
            victim: employee,
            victim_stable_id: employee_sensor.stable_id,
        });
    }
}

fn snare_flea_suffocate_attached(
    sim_hz: Res<SimHz>,
    mut damage_events: EventWriter<IncomingDamageEvent>,
    mut snare_fleas: Query<(Entity, &mut SnareFleaAttachment), With<SnareFlea>>,
) {
    let interval_ticks = fixed_seconds_to_ticks(SNARE_FLEA_ATTACK_SPEED, sim_hz.0);

    for (snare_flea, mut attachment) in snare_fleas.iter_mut() {
        if !attachment.has_victim {
            continue;
        }

        if attachment.damage_timer_ticks > 0 {
            attachment.damage_timer_ticks -= 1;
            continue;
        }

        damage_events.send(IncomingDamageEvent {
            target: attachment.victim,
            raw_amount: SNARE_FLEA_ATTACK_DAMAGE,
            damage_type: DamageType::Standard,
            source: snare_flea,
        });
        attachment.damage_timer_ticks = interval_ticks;
    }
}

fn snare_flea_detach_on_victim_damage_exit_or_death(
    mut state_events: EventWriter<SnareFleaStateChangedEvent>,
    mut detached_events: EventWriter<SnareFleaDetachedEvent>,
    mut snare_fleas: Query<
        (
            Entity,
            &mut SimPosition,
            &mut SnareFleaState,
            &SnareFleaCeilingAnchor,
            &mut SnareFleaAttachment,
        ),
        With<SnareFlea>,
    >,
    employees: Query<&SnareFleaEmployeeSensor>,
) {
    for (snare_flea, mut position, mut state, anchor, mut attachment) in snare_fleas.iter_mut() {
        if !attachment.has_victim {
            continue;
        }

        let Some(sensor) = employee_sensor_by_stable_id(attachment.victim_stable_id, &employees) else {
            continue;
        };

        let reason = if sensor.victim_took_damage {
            Some(SnareFleaDetachReason::VictimDamaged)
        } else if sensor.victim_left_facility {
            Some(SnareFleaDetachReason::VictimLeftFacility)
        } else if sensor.victim_died {
            Some(SnareFleaDetachReason::VictimDied)
        } else {
            None
        };

        if let Some(detach_reason) = reason {
            detach_snare_flea(
                snare_flea,
                &mut position,
                &mut state,
                anchor,
                &mut attachment,
                detach_reason,
                &mut state_events,
                &mut detached_events,
            );
        }
    }
}

fn snare_flea_singleplayer_critical_detach(
    mut state_events: EventWriter<SnareFleaStateChangedEvent>,
    mut detached_events: EventWriter<SnareFleaDetachedEvent>,
    mut snare_fleas: Query<
        (
            Entity,
            &mut SimPosition,
            &mut SnareFleaState,
            &SnareFleaCeilingAnchor,
            &mut SnareFleaAttachment,
        ),
        With<SnareFlea>,
    >,
    employees: Query<&SnareFleaEmployeeSensor>,
) {
    for (snare_flea, mut position, mut state, anchor, mut attachment) in snare_fleas.iter_mut() {
        if !attachment.has_victim {
            continue;
        }

        let Some(sensor) = employee_sensor_by_stable_id(attachment.victim_stable_id, &employees) else {
            continue;
        };

        if !sensor.is_singleplayer || !sensor.is_critical_health {
            continue;
        }

        detach_snare_flea(
            snare_flea,
            &mut position,
            &mut state,
            anchor,
            &mut attachment,
            SnareFleaDetachReason::SingleplayerCriticalHealth,
            &mut state_events,
            &mut detached_events,
        );
    }
}

fn snare_flea_door_attempt_speed(
    mut events: EventReader<SnareFleaDoorAttemptEvent>,
    mut resolved_events: EventWriter<SnareFleaDoorAttemptResolvedEvent>,
    snare_fleas: Query<(), With<SnareFlea>>,
) {
    for event in events.read() {
        if snare_fleas.get(event.snare_flea).is_err() {
            continue;
        }

        resolved_events.send(SnareFleaDoorAttemptResolvedEvent {
            snare_flea: event.snare_flea,
            door: event.door,
            adjusted_open_ticks: fixed_ticks_scaled(event.base_open_ticks, SNARE_FLEA_DOOR_OPEN_SPEED),
        });
    }
}

fn snare_flea_weapon_knockoff(
    mut state_events: EventWriter<SnareFleaStateChangedEvent>,
    mut detached_events: EventWriter<SnareFleaDetachedEvent>,
    mut knockoff_events: EventWriter<SnareFleaWeaponKnockoffEvent>,
    mut snare_fleas: Query<
        (
            Entity,
            &mut SimPosition,
            &mut SnareFleaState,
            &SnareFleaCeilingAnchor,
            &mut SnareFleaAttachment,
            &SnareFleaHitSensor,
        ),
        With<SnareFlea>,
    >,
) {
    for (snare_flea, mut position, mut state, anchor, mut attachment, hit_sensor) in snare_fleas.iter_mut() {
        if !attachment.has_victim || !snare_flea_was_knocked_by_weapon(hit_sensor) {
            continue;
        }

        knockoff_events.send(SnareFleaWeaponKnockoffEvent {
            snare_flea,
            victim: attachment.victim,
            victim_stable_id: attachment.victim_stable_id,
        });

        detach_snare_flea(
            snare_flea,
            &mut position,
            &mut state,
            anchor,
            &mut attachment,
            SnareFleaDetachReason::WeaponKnockoff,
            &mut state_events,
            &mut detached_events,
        );
    }
}

fn snare_flea_zap_gun_dislodge(
    mut events: EventReader<SnareFleaZapGunTargetedEvent>,
    mut difficulty_events: EventWriter<SnareFleaZapGunDifficultyEvent>,
    mut state_events: EventWriter<SnareFleaStateChangedEvent>,
    mut detached_events: EventWriter<SnareFleaDetachedEvent>,
    mut snare_fleas: Query<
        (
            &mut SimPosition,
            &mut SnareFleaState,
            &SnareFleaCeilingAnchor,
            &mut SnareFleaAttachment,
        ),
        With<SnareFlea>,
    >,
) {
    for event in events.read() {
        let Ok((mut position, mut state, anchor, mut attachment)) = snare_fleas.get_mut(event.snare_flea) else {
            continue;
        };

        let dislodged = attachment.has_victim;
        difficulty_events.send(SnareFleaZapGunDifficultyEvent {
            snare_flea: event.snare_flea,
            difficulty_modifier: SNARE_FLEA_ZAP_GUN_DIFFICULTY,
            dislodged,
        });

        if !dislodged {
            continue;
        }

        detach_snare_flea(
            event.snare_flea,
            &mut position,
            &mut state,
            anchor,
            &mut attachment,
            SnareFleaDetachReason::ZapGun,
            &mut state_events,
            &mut detached_events,
        );
    }
}

fn snare_flea_kill_on_facility_exit_or_teleporter(
    mut commands: Commands,
    mut killed_events: EventWriter<SnareFleaKilledByFacilityTransitionEvent>,
    snare_fleas: Query<(Entity, &SnareFleaAttachment), With<SnareFlea>>,
    employees: Query<&SnareFleaEmployeeSensor>,
) {
    for (snare_flea, attachment) in snare_fleas.iter() {
        if !attachment.has_victim {
            continue;
        }

        let Some(sensor) = employee_sensor_by_stable_id(attachment.victim_stable_id, &employees) else {
            continue;
        };

        if !sensor.victim_left_facility && !sensor.moved_by_teleporter {
            continue;
        }

        killed_events.send(SnareFleaKilledByFacilityTransitionEvent {
            snare_flea,
            victim: attachment.victim,
            victim_stable_id: attachment.victim_stable_id,
        });
        commands.entity(snare_flea).despawn();
    }
}

fn snare_flea_scanner_identifies_ceiling_position(
    mut events: EventReader<ScannerActivationEvent>,
    mut identified_events: EventWriter<SnareFleaScannerIdentifiedEvent>,
    snare_fleas: Query<(Entity, &SnareFleaCeilingAnchor), With<SnareFlea>>,
) {
    for _event in events.read() {
        for (snare_flea, anchor) in snare_fleas.iter() {
            identified_events.send(SnareFleaScannerIdentifiedEvent {
                snare_flea,
                ceiling_position: anchor.position,
                radar_pip_size: SNARE_FLEA_RADAR_PIP_SIZE,
            });
        }
    }
}

fn snare_flea_apply_stun_multiplier(
    mut events: EventReader<SnareFleaStunAppliedEvent>,
    mut adjusted_events: EventWriter<SnareFleaStunAdjustedEvent>,
) {
    for event in events.read() {
        adjusted_events.send(SnareFleaStunAdjustedEvent {
            snare_flea: event.snare_flea,
            base_ticks: event.base_ticks,
            adjusted_ticks: fixed_ticks_scaled(event.base_ticks, SNARE_FLEA_STUN_MULTIPLIER),
            hitbox_unreliable: true,
        });
    }
}

fn snare_flea_checksum(
    mut checksum: ResMut<SimChecksumState>,
    snare_fleas: Query<
        (
            &SimPosition,
            &Health,
            &UnitStats,
            &SnareFleaState,
            &SnareFleaCeilingAnchor,
            &SnareFleaAttachment,
            &SnareFleaHitSensor,
        ),
        With<SnareFlea>,
    >,
) {
    for (position, health, stats, state, anchor, attachment, hit_sensor) in snare_fleas.iter() {
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(health.current.to_bits() as u64);
        checksum.accumulate(health.max.to_bits() as u64);
        checksum.accumulate(stats.move_speed.to_bits() as u64);
        checksum.accumulate(stats.attack_range.to_bits() as u64);
        checksum.accumulate(stats.attack_damage.to_bits() as u64);
        checksum.accumulate(stats.attack_speed.to_bits() as u64);
        checksum.accumulate(stats.watch_range.to_bits() as u64);
        checksum.accumulate(snare_flea_state_bits(*state));
        checksum.accumulate(anchor.position.x.to_bits() as u64);
        checksum.accumulate(anchor.position.y.to_bits() as u64);
        checksum.accumulate(attachment.has_victim as u64);
        checksum.accumulate(attachment.victim_stable_id);
        checksum.accumulate(attachment.damage_timer_ticks as u64);
        checksum.accumulate(attachment.voice_muffled as u64);
        checksum.accumulate(hit_sensor.hit_by_shovel as u64);
        checksum.accumulate(hit_sensor.hit_by_stop_sign as u64);
        checksum.accumulate(hit_sensor.hit_by_yield_sign as u64);
        checksum.accumulate(hit_sensor.hit_by_double_barrel as u64);
        checksum.accumulate(hit_sensor.hit_by_kitchen_knife as u64);
        checksum.accumulate(hit_sensor.hit_by_stun_grenade as u64);
    }
}

fn nearest_triggering_employee(
    employees: &Query<(Entity, &SimPosition, &SnareFleaEmployeeSensor), Without<SnareFlea>>,
) -> Option<(Entity, SimPosition, SnareFleaEmployeeSensor)> {
    let mut best: Option<(Entity, SimPosition, SnareFleaEmployeeSensor)> = None;

    for (entity, position, sensor) in employees.iter() {
        if !sensor.is_beneath_or_near {
            continue;
        }

        if let Some((_best_entity, _best_position, best_sensor)) = best {
            if sensor.stable_id >= best_sensor.stable_id {
                continue;
            }
        }

        best = Some((entity, *position, *sensor));
    }

    best
}

fn employee_sensor_by_stable_id(
    stable_id: u64,
    employees: &Query<&SnareFleaEmployeeSensor>,
) -> Option<SnareFleaEmployeeSensor> {
    for sensor in employees.iter() {
        if sensor.stable_id == stable_id {
            return Some(*sensor);
        }
    }

    None
}

fn detach_snare_flea(
    snare_flea: Entity,
    position: &mut SimPosition,
    state: &mut SnareFleaState,
    anchor: &SnareFleaCeilingAnchor,
    attachment: &mut SnareFleaAttachment,
    reason: SnareFleaDetachReason,
    state_events: &mut EventWriter<SnareFleaStateChangedEvent>,
    detached_events: &mut EventWriter<SnareFleaDetachedEvent>,
) {
    let victim = attachment.victim;
    let victim_stable_id = attachment.victim_stable_id;

    attachment.has_victim = false;
    attachment.victim = Entity::PLACEHOLDER;
    attachment.victim_stable_id = 0;
    attachment.voice_muffled = false;
    position.x = anchor.position.x;
    position.y = anchor.position.y;

    set_snare_flea_state(
        snare_flea,
        state,
        SnareFleaState::Roaming,
        state_events,
    );

    detached_events.send(SnareFleaDetachedEvent {
        snare_flea,
        victim,
        victim_stable_id,
        reason,
    });
}

fn set_snare_flea_state(
    snare_flea: Entity,
    state: &mut SnareFleaState,
    next: SnareFleaState,
    events: &mut EventWriter<SnareFleaStateChangedEvent>,
) {
    if *state == next {
        return;
    }

    let previous = *state;
    *state = next;
    events.send(SnareFleaStateChangedEvent {
        snare_flea,
        from: previous,
        to: next,
    });
}

fn snare_flea_was_knocked_by_weapon(hit_sensor: &SnareFleaHitSensor) -> bool {
    hit_sensor.hit_by_shovel
        || hit_sensor.hit_by_stop_sign
        || hit_sensor.hit_by_yield_sign
        || hit_sensor.hit_by_double_barrel
        || hit_sensor.hit_by_kitchen_knife
        || hit_sensor.hit_by_stun_grenade
}

fn fixed_seconds_to_ticks(seconds: I32F32, sim_hz: I32F32) -> u32 {
    let ticks = seconds * sim_hz;
    let whole_ticks: u32 = ticks.to_num();

    if whole_ticks == 0 {
        1
    } else {
        whole_ticks
    }
}

fn fixed_ticks_scaled(ticks: u32, scale: I32F32) -> u32 {
    let scaled = I32F32::from_num(ticks) * scale;
    let whole_ticks: u32 = scaled.to_num();

    if whole_ticks == 0 {
        1
    } else {
        whole_ticks
    }
}

fn snare_flea_state_bits(state: SnareFleaState) -> u64 {
    match state {
        SnareFleaState::Roaming => 0,
        SnareFleaState::Trapping => 1,
    }
}