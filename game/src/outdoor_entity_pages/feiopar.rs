// Sources: vault/outdoor_entity_pages/feiopar.md
use bevy::prelude::*;
use fixed::types::I32F32;
use rand_core::RngCore;

use crate::sim::{
    tick_rng, DamageType, GameSeed, Health, IncomingDamageEvent, NoiseEmittedEvent,
    SimChecksumState, SimHz, SimPosition, SimTick, UnitStats,
};

pub const FEIOPAR_ID: &str = "feiopar";
pub const FEIOPAR_NAME: &str = "Feiopar";
pub const FEIOPAR_TYPE: &str = "outdoor_entity_pages";
pub const FEIOPAR_SUBTYPE: &str = "bestiary";
pub const FEIOPAR_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Feiopar";
pub const FEIOPAR_SOURCE_REVISION: u32 = 21480;
pub const FEIOPAR_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const FEIOPAR_CONFIDENCE_BASIS_POINTS: u16 = 84;

pub const FEIOPAR_DWELLS: &str = "Outdoors";
pub const FEIOPAR_DANGER: &str = "Not mentioned by Sigurd";
pub const FEIOPAR_SCIENTIFIC_NAME: &str = "Felis arborum";
pub const FEIOPAR_HP: I32F32 = I32F32::lit("4");
pub const FEIOPAR_POWER_LEVEL: I32F32 = I32F32::lit("2");
pub const FEIOPAR_MAX_SPAWNED: usize = 6;
pub const FEIOPAR_ATTACK_DAMAGE: I32F32 = I32F32::lit("7");
pub const FEIOPAR_STUN_MULTIPLIER: I32F32 = I32F32::lit("0.3");
pub const FEIOPAR_CAN_SEE_THROUGH_FOG: bool = true;
pub const FEIOPAR_SPAWN_DELAY_SECONDS: I32F32 = I32F32::lit("15");
pub const FEIOPAR_DOOR_SPEED_MULTIPLIER: I32F32 = I32F32::lit("1.0");
pub const FEIOPAR_ZAP_GUN_DIFFICULTY: I32F32 = I32F32::lit("1.4");
pub const FEIOPAR_INTERNAL_NAME: &str = "Puma";
pub const FEIOPAR_PIP_SIZE: &str = "Tiny";

pub const FEIOPAR_ROAM_SPEED: I32F32 = I32F32::lit("14");
pub const FEIOPAR_TREE_EMPLOYEE_ALERT_RANGE: I32F32 = I32F32::lit("16");
pub const FEIOPAR_SEEN_TO_HIDE_SECONDS: I32F32 = I32F32::lit("2");
pub const FEIOPAR_HIDING_TIMER_SECONDS: I32F32 = I32F32::lit("6");
pub const FEIOPAR_LEAP_TREE_SEARCH_RANGE: I32F32 = I32F32::lit("26");
pub const FEIOPAR_TREE_CLOSE_STARE_RANGE: I32F32 = I32F32::lit("4.5");
pub const FEIOPAR_TREE_ANGER_DROP_THRESHOLD: I32F32 = I32F32::lit("0.45");
pub const FEIOPAR_TREE_UNWATCHED_RANGE: I32F32 = I32F32::lit("60");
pub const FEIOPAR_FAR_STALK_RANGE: I32F32 = I32F32::lit("10");
pub const FEIOPAR_MID_STALK_MIN_RANGE: I32F32 = I32F32::lit("4.5");
pub const FEIOPAR_MID_STALK_MAX_RANGE: I32F32 = I32F32::lit("7");
pub const FEIOPAR_CLOSE_STALK_RANGE: I32F32 = I32F32::lit("4.5");
pub const FEIOPAR_FAR_STALK_SPEED: I32F32 = I32F32::lit("15");
pub const FEIOPAR_MID_STALK_SPEED: I32F32 = I32F32::lit("6");
pub const FEIOPAR_CLOSE_STALK_SPEED: I32F32 = I32F32::lit("9");
pub const FEIOPAR_SEEN_CLOSE_STALK_SECONDS: I32F32 = I32F32::lit("6");
pub const FEIOPAR_SEEN_CLOSE_STALK_SPEED: I32F32 = I32F32::lit("12");
pub const FEIOPAR_THREAT_RANGE: I32F32 = I32F32::lit("7.5");
pub const FEIOPAR_FREEZE_FOV_DEGREES: I32F32 = I32F32::lit("67");
pub const FEIOPAR_FREEZE_VIEW_RANGE: I32F32 = I32F32::lit("90");
pub const FEIOPAR_FREEZE_LOOK_SECONDS: I32F32 = I32F32::lit("0.08");
pub const FEIOPAR_FROZEN_GIVE_UP_SEEN_METER: I32F32 = I32F32::lit("16");
pub const FEIOPAR_FROZEN_HIDING_SEEN_METER: I32F32 = I32F32::lit("2");
pub const FEIOPAR_STALK_FROZEN_RANGE: I32F32 = I32F32::lit("7.5");
pub const FEIOPAR_STALK_FROZEN_LOOK_DEGREES: I32F32 = I32F32::lit("25");
pub const FEIOPAR_STALK_FROZEN_SECONDS: I32F32 = I32F32::lit("1.5");
pub const FEIOPAR_EYE_CONTACT_DEGREES: I32F32 = I32F32::lit("25");
pub const FEIOPAR_EYE_CONTACT_RANGE: I32F32 = I32F32::lit("3.5");
pub const FEIOPAR_HIDING_RUN_SPEED: I32F32 = I32F32::lit("14");
pub const FEIOPAR_HIDING_TREE_RANGE: I32F32 = I32F32::lit("15");
pub const FEIOPAR_HIDING_TREE_MIN_DISTANCE: I32F32 = I32F32::lit("5");
pub const FEIOPAR_HIDING_TREE_MAX_DISTANCE: I32F32 = I32F32::lit("46");
pub const FEIOPAR_NO_HIDING_TREE_SECONDS: I32F32 = I32F32::lit("0.45");
pub const FEIOPAR_NOISE_DIRECT_RANGE: I32F32 = I32F32::lit("20");
pub const FEIOPAR_NOISE_FOCUSED_MIN_RANGE: I32F32 = I32F32::lit("8");
pub const FEIOPAR_NOISE_FOCUSED_MAX_RANGE: I32F32 = I32F32::lit("17");
pub const FEIOPAR_NOISE_SUPPRESS_SECONDS: I32F32 = I32F32::lit("2");
pub const FEIOPAR_LOUD_NOISE_RANGE: I32F32 = I32F32::lit("15");
pub const FEIOPAR_ATTACK_RANGE: I32F32 = I32F32::lit("6");
pub const FEIOPAR_ATTACK_TOUCH_COOLDOWN_SECONDS: I32F32 = I32F32::lit("2");
pub const FEIOPAR_ATTACK_GIVE_UP_SECONDS: I32F32 = I32F32::lit("5");
pub const FEIOPAR_SPRINT_SCARE_RANGE: I32F32 = I32F32::lit("9");
pub const FEIOPAR_SCARED_FILL_PER_SECOND: I32F32 = I32F32::lit("2");
pub const FEIOPAR_SCARED_FLEE_LOW: I32F32 = I32F32::lit("0.8");
pub const FEIOPAR_SCARED_FLEE_LOW_SECONDS: I32F32 = I32F32::lit("1.2");
pub const FEIOPAR_SCARED_FLEE_HIGH: I32F32 = I32F32::lit("2");
pub const FEIOPAR_SCARED_FLEE_HIGH_SECONDS: I32F32 = I32F32::lit("0.5");
pub const FEIOPAR_SCARED_DRAIN_MIN_PER_SECOND: I32F32 = I32F32::lit("3");
pub const FEIOPAR_SCARED_DRAIN_MAX_PER_SECOND: I32F32 = I32F32::lit("6");
pub const FEIOPAR_ATTACK_SPEED_SECONDS: I32F32 = I32F32::lit("2");
pub const FEIOPAR_WEAPON_HIT_DAMAGE: I32F32 = I32F32::lit("1");
pub const FEIOPAR_LEAP_SOUND_SALT: u64 = 0xFE10_FA00_0000_0001;

pub const FEIOPAR_DEPENDS_ON: [&str; 3] = ["lethal_company", "vow", "march"];

pub const FEIOPAR_FRONTMATTER_BEHAVIOR: [&str; 5] = [
    "Roaming",
    "Stalking",
    "Hiding",
    "Fleeing",
    "Attacking",
];

pub const FEIOPAR_BEHAVIORAL_MECHANICS: [FeioparBehaviorRule; 42] = [
    FeioparBehaviorRule {
        condition: "it is roaming",
        outcome: "it runs at 14 units per second toward the nearest suitable tree, avoids trees that are closer to an employee than to itself, and climbs into idle",
    },
    FeioparBehaviorRule {
        condition: "it is climbing a tree and an employee is within 16 units",
        outcome: "the camera shakes slightly",
    },
    FeioparBehaviorRule {
        condition: "it is idle and an employee has seen it for more than 2 seconds",
        outcome: "it sets hiding_timer to 6 and prioritizes trees hidden from all employees",
    },
    FeioparBehaviorRule {
        condition: "hiding_timer is greater than 0",
        outcome: "it leaps toward trees that employees cannot see",
    },
    FeioparBehaviorRule {
        condition: "hiding_timer is 0",
        outcome: "it leaps toward trees that are closer to an employee",
    },
    FeioparBehaviorRule {
        condition: "it is searching for a leap target",
        outcome: "it checks trees within 26 units and expands that search after each failed leap attempt",
    },
    FeioparBehaviorRule {
        condition: "it leaps while an employee is facing it",
        outcome: "one leaf particle effect and one random leap sound play",
    },
    FeioparBehaviorRule {
        condition: "an employee is within 4.5 units of the tree it is on and is staring directly at it",
        outcome: "the anger meter fills and at 0.45 it forces a drop from the tree",
    },
    FeioparBehaviorRule {
        condition: "it stays on its current tree long enough and no employee is looking at it within 60 units",
        outcome: "it drops down and enters state 1",
    },
    FeioparBehaviorRule {
        condition: "stalking",
        outcome: "it moves toward the closest AI node near the closest employee while avoiding employee line of sight",
    },
    FeioparBehaviorRule {
        condition: "the distance to the employee is greater than 10 units",
        outcome: "stalking speed can reach 15 units per second",
    },
    FeioparBehaviorRule {
        condition: "the distance to the employee is between 4.5 units and 7 units",
        outcome: "stalking speed can reach 6 units per second",
    },
    FeioparBehaviorRule {
        condition: "the distance to the employee is within 4.5 units",
        outcome: "stalking speed can reach 9 units per second",
    },
    FeioparBehaviorRule {
        condition: "an employee has seen it while it was close for more than 6 seconds",
        outcome: "stalking speed becomes 12 units per second",
    },
    FeioparBehaviorRule {
        condition: "a threatening player with high threat level or within 7.5 units is detected while it is not already startled",
        outcome: "it becomes startled, freezes momentarily, and then either resumes stalking or flees to state 0 based on StartledTimer",
    },
    FeioparBehaviorRule {
        condition: "it is startled and stunned",
        outcome: "it immediately flees to state 0",
    },
    FeioparBehaviorRule {
        condition: "an employee looks at the Puma within a 67 degree field of view and within 90 units for 0.08 seconds",
        outcome: "it enters frozen state",
    },
    FeioparBehaviorRule {
        condition: "it is frozen",
        outcome: "it stops growling and moving",
    },
    FeioparBehaviorRule {
        condition: "the seen meter reaches 16 or higher while frozen",
        outcome: "it gives up and flees",
    },
    FeioparBehaviorRule {
        condition: "the seen meter is greater than 2 while frozen",
        outcome: "it enters hiding mode",
    },
    FeioparBehaviorRule {
        condition: "it is frozen and an employee is within 7.5 units with look angle above 25 degrees",
        outcome: "the stalkFrozen timer fills",
    },
    FeioparBehaviorRule {
        condition: "the stalkFrozen timer reaches 1.5 seconds",
        outcome: "it exits freeze and enters state 2",
    },
    FeioparBehaviorRule {
        condition: "an employee makes direct eye contact within 25 degrees and under 3.5 units while it is frozen",
        outcome: "it immediately enters state 2",
    },
    FeioparBehaviorRule {
        condition: "hiding mode is triggered by a seen meter greater than 2",
        outcome: "it runs at 14 units per second toward the nearest tree within 15 units that is not in direct line of sight of any employee",
    },
    FeioparBehaviorRule {
        condition: "it is choosing a hiding tree",
        outcome: "it prefers trees that are between 5 units and 46 units from the employee, not in any employee line of sight, and reachable",
    },
    FeioparBehaviorRule {
        condition: "the selected tree keeps getting it spotted",
        outcome: "it looks for the next one",
    },
    FeioparBehaviorRule {
        condition: "no valid hiding tree is found for 0.45 seconds",
        outcome: "it enters state 0",
    },
    FeioparBehaviorRule {
        condition: "a noise occurs within 20 units or within 8 units to 17 units when already focused and lacks direct line of sight",
        outcome: "it reacts",
    },
    FeioparBehaviorRule {
        condition: "the last noise was within 2 seconds",
        outcome: "noise reactions are suppressed",
    },
    FeioparBehaviorRule {
        condition: "a noise comes from a known employee",
        outcome: "it is ignored",
    },
    FeioparBehaviorRule {
        condition: "a close loud noise within 15 units has loudness divided by distance greater than distance",
        outcome: "it triggers startled state with a high focus level",
    },
    FeioparBehaviorRule {
        condition: "it is attacking and within 6 units",
        outcome: "it attacks the employee and plays a scream sound",
    },
    FeioparBehaviorRule {
        condition: "it collides with an employee during an attack",
        outcome: "it deals 7 damage with a scratching attack, applies small knockback, and leaves bloody scratches",
    },
    FeioparBehaviorRule {
        condition: "it has no valid target",
        outcome: "attacking stops",
    },
    FeioparBehaviorRule {
        condition: "it has not touched the employee in the last 2 seconds after 5 seconds of attacking",
        outcome: "attacking stops",
    },
    FeioparBehaviorRule {
        condition: "the employee enters and closes the ship doors while the puma is outside",
        outcome: "attacking stops",
    },
    FeioparBehaviorRule {
        condition: "the scared meter threshold is reached",
        outcome: "attacking stops",
    },
    FeioparBehaviorRule {
        condition: "it is hit while in state 1 or state 2",
        outcome: "it immediately enters state 0",
    },
    FeioparBehaviorRule {
        condition: "an employee is sprinting toward it within 9 units while looking at it",
        outcome: "the scared meter fills at 2 per second",
    },
    FeioparBehaviorRule {
        condition: "the scared meter is at least 0.8 after 1.2 seconds of attacking",
        outcome: "it flees to state 0",
    },
    FeioparBehaviorRule {
        condition: "the scared meter is at least 2 after 0.5 seconds of attacking",
        outcome: "it flees to state 0",
    },
    FeioparBehaviorRule {
        condition: "the employee is not sprinting toward it",
        outcome: "the scared meter drains at 3 to 6 per second",
    },
];

pub struct FeioparPlugin;

impl Plugin for FeioparPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnFeioparEvent>()
            .add_event::<FeioparStateChangedEvent>()
            .add_event::<FeioparTreeCameraShakeEvent>()
            .add_event::<FeioparLeapVisualEvent>()
            .add_event::<FeioparStartledEvent>()
            .add_event::<FeioparFrozenEvent>()
            .add_event::<FeioparNoiseReactedEvent>()
            .add_event::<FeioparAttackScreamEvent>()
            .add_event::<FeioparScratchHitEvent>()
            .add_event::<FeioparScaredMeterChangedEvent>()
            .add_event::<FeioparStunAdjustedEvent>()
            .add_event::<FeioparDoorAttemptEvent>()
            .add_event::<FeioparDoorAttemptResolvedEvent>()
            .add_event::<FeioparZapGunTargetedEvent>()
            .add_event::<FeioparZapGunDifficultyEvent>()
            .add_event::<FeioparDamageTakenEvent>()
            .add_event::<FeioparDefeatedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    feiopar_spawn_from_events,
                    feiopar_climb_camera_shake,
                    feiopar_roam_to_tree,
                    feiopar_idle_and_leap_logic,
                    feiopar_stalk_closest_employee,
                    feiopar_threat_and_freeze_logic,
                    feiopar_noise_reactions,
                    feiopar_attack_target,
                    feiopar_update_scared_meter,
                    feiopar_apply_stun_and_damage,
                    feiopar_door_attempt_speed,
                    feiopar_zap_gun_difficulty,
                    feiopar_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FeioparBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Feiopar {
    pub stable_id: u64,
    pub spawn_delay_ticks_remaining: u32,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FeioparEmployeeSensor {
    pub stable_id: u64,
    pub is_alive: bool,
    pub visible_to_feiopar: bool,
    pub looking_at_feiopar: bool,
    pub direct_eye_contact: bool,
    pub staring_directly: bool,
    pub sprinting_toward_feiopar: bool,
    pub high_threat_level: bool,
    pub in_ship_with_doors_closed: bool,
    pub look_angle_degrees: I32F32,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FeioparTreeSensor {
    pub stable_id: u64,
    pub reachable: bool,
    pub hidden_from_all_employees: bool,
    pub in_any_employee_line_of_sight: bool,
    pub spotted_repeatedly: bool,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FeioparTarget {
    pub has_target: bool,
    pub target_entity: Entity,
    pub target_stable_id: u64,
    pub last_known_position: SimPosition,
    pub target_visible: bool,
}

impl Default for FeioparTarget {
    fn default() -> Self {
        Self {
            has_target: false,
            target_entity: Entity::PLACEHOLDER,
            target_stable_id: 0,
            last_known_position: SimPosition {
                x: I32F32::ZERO,
                y: I32F32::ZERO,
            },
            target_visible: false,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FeioparTreeTarget {
    pub has_tree: bool,
    pub tree_entity: Entity,
    pub tree_stable_id: u64,
    pub position: SimPosition,
    pub leap_search_range: I32F32,
    pub failed_leap_attempts: u32,
}

impl Default for FeioparTreeTarget {
    fn default() -> Self {
        Self {
            has_tree: false,
            tree_entity: Entity::PLACEHOLDER,
            tree_stable_id: 0,
            position: SimPosition {
                x: I32F32::ZERO,
                y: I32F32::ZERO,
            },
            leap_search_range: FEIOPAR_LEAP_TREE_SEARCH_RANGE,
            failed_leap_attempts: 0,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FeioparTimers {
    pub seen_ticks: u32,
    pub hiding_ticks: u32,
    pub tree_idle_ticks: u32,
    pub freeze_look_ticks: u32,
    pub seen_close_ticks: u32,
    pub stalk_frozen_ticks: u32,
    pub no_hiding_tree_ticks: u32,
    pub last_noise_ticks: u32,
    pub attack_ticks: u32,
    pub last_touch_ticks: u32,
    pub low_scared_attack_ticks: u32,
    pub high_scared_attack_ticks: u32,
    pub startled_ticks: u32,
}

impl Default for FeioparTimers {
    fn default() -> Self {
        Self {
            seen_ticks: 0,
            hiding_ticks: 0,
            tree_idle_ticks: 0,
            freeze_look_ticks: 0,
            seen_close_ticks: 0,
            stalk_frozen_ticks: 0,
            no_hiding_tree_ticks: 0,
            last_noise_ticks: u32::MAX,
            attack_ticks: 0,
            last_touch_ticks: 0,
            low_scared_attack_ticks: 0,
            high_scared_attack_ticks: 0,
            startled_ticks: 0,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FeioparMeters {
    pub anger: I32F32,
    pub seen: I32F32,
    pub scared: I32F32,
    pub focus: I32F32,
}

impl Default for FeioparMeters {
    fn default() -> Self {
        Self {
            anger: I32F32::ZERO,
            seen: I32F32::ZERO,
            scared: I32F32::ZERO,
            focus: I32F32::ZERO,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FeioparState {
    #[default]
    Roaming,
    Climbing,
    TreeIdle,
    Hiding,
    Stalking,
    Startled,
    Frozen,
    Attacking,
    Dead,
}

#[derive(Bundle)]
pub struct FeioparBundle {
    pub name: Name,
    pub feiopar: Feiopar,
    pub position: SimPosition,
    pub health: Health,
    pub stats: UnitStats,
    pub state: FeioparState,
    pub target: FeioparTarget,
    pub tree_target: FeioparTreeTarget,
    pub timers: FeioparTimers,
    pub meters: FeioparMeters,
}

impl FeioparBundle {
    pub fn new(event: SpawnFeioparEvent, sim_hz: I32F32) -> Self {
        Self {
            name: Name::new(FEIOPAR_NAME),
            feiopar: Feiopar {
                stable_id: event.stable_id,
                spawn_delay_ticks_remaining: fixed_seconds_to_ticks(
                    FEIOPAR_SPAWN_DELAY_SECONDS,
                    sim_hz,
                ),
            },
            position: event.position,
            health: Health::full(FEIOPAR_HP),
            stats: UnitStats {
                move_speed: FEIOPAR_ROAM_SPEED,
                attack_range: FEIOPAR_ATTACK_RANGE,
                attack_damage: FEIOPAR_ATTACK_DAMAGE,
                attack_speed: FEIOPAR_ATTACK_SPEED_SECONDS,
                watch_range: FEIOPAR_FREEZE_VIEW_RANGE,
            },
            state: FeioparState::Roaming,
            target: FeioparTarget {
                last_known_position: event.position,
                ..Default::default()
            },
            tree_target: FeioparTreeTarget::default(),
            timers: FeioparTimers::default(),
            meters: FeioparMeters::default(),
        }
    }
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpawnFeioparEvent {
    pub stable_id: u64,
    pub position: SimPosition,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FeioparStateChangedEvent {
    pub feiopar: Entity,
    pub from: FeioparState,
    pub to: FeioparState,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FeioparTreeCameraShakeEvent {
    pub feiopar: Entity,
    pub employee: Entity,
    pub tree: Entity,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FeioparLeapVisualEvent {
    pub feiopar: Entity,
    pub tree: Entity,
    pub sound_index: u32,
    pub leaf_particle_count: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FeioparStartledEvent {
    pub feiopar: Entity,
    pub source: Entity,
    pub high_focus: bool,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FeioparFrozenEvent {
    pub feiopar: Entity,
    pub observer: Entity,
    pub seen_meter: I32F32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FeioparNoiseReactedEvent {
    pub feiopar: Entity,
    pub noise_source: Entity,
    pub sound_position: SimPosition,
    pub high_focus: bool,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FeioparAttackScreamEvent {
    pub feiopar: Entity,
    pub target: Entity,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FeioparScratchHitEvent {
    pub feiopar: Entity,
    pub target: Entity,
    pub damage: I32F32,
    pub small_knockback: bool,
    pub bloody_scratches: bool,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FeioparScaredMeterChangedEvent {
    pub feiopar: Entity,
    pub target: Entity,
    pub scared_meter: I32F32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FeioparStunAdjustedEvent {
    pub feiopar: Entity,
    pub source: Entity,
    pub stun_multiplier: I32F32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FeioparDoorAttemptEvent {
    pub feiopar: Entity,
    pub door: Entity,
    pub base_open_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FeioparDoorAttemptResolvedEvent {
    pub feiopar: Entity,
    pub door: Entity,
    pub adjusted_open_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FeioparZapGunTargetedEvent {
    pub feiopar: Entity,
    pub wielder: Entity,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FeioparZapGunDifficultyEvent {
    pub feiopar: Entity,
    pub wielder: Entity,
    pub difficulty: I32F32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FeioparDamageTakenEvent {
    pub feiopar: Entity,
    pub source: Entity,
    pub damage: I32F32,
    pub remaining_health: I32F32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FeioparDefeatedEvent {
    pub feiopar: Entity,
    pub source: Entity,
    pub fell_from_tree: bool,
}

fn feiopar_spawn_from_events(
    mut commands: Commands,
    sim_hz: Res<SimHz>,
    mut events: EventReader<SpawnFeioparEvent>,
    feiopars: Query<(), With<Feiopar>>,
) {
    let mut spawned_count = feiopars.iter().count();

    for event in events.read() {
        if spawned_count >= FEIOPAR_MAX_SPAWNED {
            break;
        }

        commands.spawn(FeioparBundle::new(*event, sim_hz.0));
        spawned_count += 1;
    }
}

fn feiopar_climb_camera_shake(
    mut events: EventWriter<FeioparTreeCameraShakeEvent>,
    feiopars: Query<(Entity, &SimPosition, &FeioparState, &FeioparTreeTarget), With<Feiopar>>,
    employees: Query<(Entity, &SimPosition, &FeioparEmployeeSensor), Without<Feiopar>>,
) {
    for (feiopar_entity, position, state, tree_target) in feiopars.iter() {
        if *state != FeioparState::Climbing || !tree_target.has_tree {
            continue;
        }

        let Some((employee_entity, _employee_position, _sensor)) =
            nearest_employee(*position, FEIOPAR_TREE_EMPLOYEE_ALERT_RANGE, &employees)
        else {
            continue;
        };

        events.send(FeioparTreeCameraShakeEvent {
            feiopar: feiopar_entity,
            employee: employee_entity,
            tree: tree_target.tree_entity,
        });
    }
}

fn feiopar_roam_to_tree(
    sim_hz: Res<SimHz>,
    mut state_events: EventWriter<FeioparStateChangedEvent>,
    mut feiopars: Query<
        (
            Entity,
            &mut Feiopar,
            &mut SimPosition,
            &mut FeioparState,
            &mut FeioparTreeTarget,
            &mut UnitStats,
        ),
        With<Feiopar>,
    >,
    trees: Query<(Entity, &SimPosition, &FeioparTreeSensor), Without<Feiopar>>,
    employees: Query<(Entity, &SimPosition, &FeioparEmployeeSensor), Without<Feiopar>>,
) {
    for (entity, mut feiopar, mut position, mut state, mut tree_target, mut stats) in
        feiopars.iter_mut()
    {
        if feiopar.spawn_delay_ticks_remaining > 0 {
            feiopar.spawn_delay_ticks_remaining -= 1;
            continue;
        }

        if *state != FeioparState::Roaming {
            continue;
        }

        let Some((tree_entity, tree_position, tree_sensor)) =
            nearest_suitable_tree(*position, &trees, &employees)
        else {
            stats.move_speed = FEIOPAR_ROAM_SPEED;
            continue;
        };

        tree_target.has_tree = true;
        tree_target.tree_entity = tree_entity;
        tree_target.tree_stable_id = tree_sensor.stable_id;
        tree_target.position = tree_position;
        stats.move_speed = FEIOPAR_ROAM_SPEED;

        move_axis_toward(&mut position, tree_position, FEIOPAR_ROAM_SPEED / sim_hz.0);

        if fixed_distance_sq(*position, tree_position) <= fixed_square(I32F32::ONE) {
            set_feiopar_state(entity, &mut state, FeioparState::Climbing, &mut state_events);
        }
    }
}

fn feiopar_idle_and_leap_logic(
    game_seed: Res<GameSeed>,
    sim_tick: Res<SimTick>,
    sim_hz: Res<SimHz>,
    mut state_events: EventWriter<FeioparStateChangedEvent>,
    mut leap_events: EventWriter<FeioparLeapVisualEvent>,
    mut feiopars: Query<
        (
            Entity,
            &Feiopar,
            &mut SimPosition,
            &mut FeioparState,
            &mut FeioparTreeTarget,
            &mut FeioparTimers,
            &mut FeioparMeters,
            &mut UnitStats,
        ),
        With<Feiopar>,
    >,
    trees: Query<(Entity, &SimPosition, &FeioparTreeSensor), Without<Feiopar>>,
    employees: Query<(Entity, &SimPosition, &FeioparEmployeeSensor), Without<Feiopar>>,
) {
    for (
        entity,
        feiopar,
        mut position,
        mut state,
        mut tree_target,
        mut timers,
        mut meters,
        mut stats,
    ) in feiopars.iter_mut()
    {
        if *state == FeioparState::Climbing {
            set_feiopar_state(entity, &mut state, FeioparState::TreeIdle, &mut state_events);
        }

        if *state != FeioparState::TreeIdle && *state != FeioparState::Hiding {
            continue;
        }

        timers.tree_idle_ticks = timers.tree_idle_ticks.saturating_add(1);

        let visible_employee =
            nearest_looking_employee(*position, FEIOPAR_FREEZE_VIEW_RANGE, &employees);
        if visible_employee.is_some() {
            timers.seen_ticks = timers.seen_ticks.saturating_add(1);
            meters.seen += I32F32::ONE / sim_hz.0;
        } else {
            timers.seen_ticks = 0;
            if meters.seen > I32F32::ZERO {
                meters.seen -= I32F32::ONE / sim_hz.0;
            }
        }

        if fixed_ticks_to_seconds(timers.seen_ticks, sim_hz.0) > FEIOPAR_SEEN_TO_HIDE_SECONDS {
            timers.hiding_ticks = fixed_seconds_to_ticks(FEIOPAR_HIDING_TIMER_SECONDS, sim_hz.0);
            set_feiopar_state(entity, &mut state, FeioparState::Hiding, &mut state_events);
        }

        if let Some((_employee_entity, _employee_position, sensor)) =
            nearest_looking_employee(*position, FEIOPAR_TREE_CLOSE_STARE_RANGE, &employees)
        {
            if sensor.staring_directly {
                meters.anger += I32F32::ONE / sim_hz.0;
            }
        }

        if meters.anger >= FEIOPAR_TREE_ANGER_DROP_THRESHOLD {
            set_feiopar_state(entity, &mut state, FeioparState::Stalking, &mut state_events);
            stats.move_speed = FEIOPAR_CLOSE_STALK_SPEED;
            continue;
        }

        if visible_employee.is_none()
            && fixed_ticks_to_seconds(timers.tree_idle_ticks, sim_hz.0)
                >= FEIOPAR_HIDING_TIMER_SECONDS
            && nearest_looking_employee(*position, FEIOPAR_TREE_UNWATCHED_RANGE, &employees)
                .is_none()
        {
            set_feiopar_state(entity, &mut state, FeioparState::Stalking, &mut state_events);
            continue;
        }

        if *state == FeioparState::Hiding && timers.hiding_ticks > 0 {
            timers.hiding_ticks -= 1;
        }

        let prefer_hidden = timers.hiding_ticks > 0;
        let next_tree =
            leap_tree_target(*position, tree_target.leap_search_range, prefer_hidden, &trees, &employees);

        let Some((tree_entity, tree_position, tree_sensor)) = next_tree else {
            tree_target.failed_leap_attempts += 1;
            tree_target.leap_search_range += FEIOPAR_LEAP_TREE_SEARCH_RANGE;
            timers.no_hiding_tree_ticks = timers.no_hiding_tree_ticks.saturating_add(1);

            if fixed_ticks_to_seconds(timers.no_hiding_tree_ticks, sim_hz.0)
                >= FEIOPAR_NO_HIDING_TREE_SECONDS
            {
                set_feiopar_state(entity, &mut state, FeioparState::Roaming, &mut state_events);
            }
            continue;
        };

        if tree_sensor.spotted_repeatedly {
            tree_target.failed_leap_attempts += 1;
            continue;
        }

        tree_target.has_tree = true;
        tree_target.tree_entity = tree_entity;
        tree_target.tree_stable_id = tree_sensor.stable_id;
        tree_target.position = tree_position;
        tree_target.leap_search_range = FEIOPAR_LEAP_TREE_SEARCH_RANGE;
        timers.no_hiding_tree_ticks = 0;
        position.x = tree_position.x;
        position.y = tree_position.y;

        if visible_employee.is_some() {
            let mut rng = tick_rng(game_seed.0, sim_tick.0, FEIOPAR_LEAP_SOUND_SALT ^ feiopar.stable_id);
            leap_events.send(FeioparLeapVisualEvent {
                feiopar: entity,
                tree: tree_entity,
                sound_index: rng.next_u32() % 4,
                leaf_particle_count: 1,
            });
        }
    }
}

fn feiopar_stalk_closest_employee(
    sim_hz: Res<SimHz>,
    mut state_events: EventWriter<FeioparStateChangedEvent>,
    mut feiopars: Query<
        (
            Entity,
            &mut SimPosition,
            &mut FeioparState,
            &mut FeioparTarget,
            &mut FeioparTimers,
            &mut UnitStats,
        ),
        With<Feiopar>,
    >,
    employees: Query<(Entity, &SimPosition, &FeioparEmployeeSensor), Without<Feiopar>>,
) {
    for (entity, mut position, mut state, mut target, mut timers, mut stats) in feiopars.iter_mut() {
        if *state != FeioparState::Stalking {
            continue;
        }

        let Some((employee_entity, employee_position, sensor)) =
            nearest_employee(*position, FEIOPAR_FREEZE_VIEW_RANGE, &employees)
        else {
            target.has_target = false;
            continue;
        };

        target.has_target = true;
        target.target_entity = employee_entity;
        target.target_stable_id = sensor.stable_id;
        target.last_known_position = employee_position;
        target.target_visible = sensor.visible_to_feiopar;

        let distance_sq = fixed_distance_sq(*position, employee_position);
        if distance_sq > fixed_square(FEIOPAR_FAR_STALK_RANGE) {
            stats.move_speed = FEIOPAR_FAR_STALK_SPEED;
        } else if distance_sq >= fixed_square(FEIOPAR_MID_STALK_MIN_RANGE)
            && distance_sq <= fixed_square(FEIOPAR_MID_STALK_MAX_RANGE)
        {
            stats.move_speed = FEIOPAR_MID_STALK_SPEED;
        } else if distance_sq <= fixed_square(FEIOPAR_CLOSE_STALK_RANGE) {
            stats.move_speed = FEIOPAR_CLOSE_STALK_SPEED;
        }

        if sensor.looking_at_feiopar && distance_sq <= fixed_square(FEIOPAR_CLOSE_STALK_RANGE) {
            timers.seen_close_ticks = timers.seen_close_ticks.saturating_add(1);
        } else {
            timers.seen_close_ticks = 0;
        }

        if fixed_ticks_to_seconds(timers.seen_close_ticks, sim_hz.0)
            > FEIOPAR_SEEN_CLOSE_STALK_SECONDS
        {
            stats.move_speed = FEIOPAR_SEEN_CLOSE_STALK_SPEED;
        }

        move_axis_toward(&mut position, employee_position, stats.move_speed / sim_hz.0);

        if distance_sq <= fixed_square(FEIOPAR_ATTACK_RANGE) {
            set_feiopar_state(entity, &mut state, FeioparState::Attacking, &mut state_events);
        }
    }
}

fn feiopar_threat_and_freeze_logic(
    sim_hz: Res<SimHz>,
    mut state_events: EventWriter<FeioparStateChangedEvent>,
    mut startled_events: EventWriter<FeioparStartledEvent>,
    mut frozen_events: EventWriter<FeioparFrozenEvent>,
    mut feiopars: Query<
        (
            Entity,
            &SimPosition,
            &mut FeioparState,
            &mut FeioparTarget,
            &mut FeioparTimers,
            &mut FeioparMeters,
            &mut UnitStats,
        ),
        With<Feiopar>,
    >,
    employees: Query<(Entity, &SimPosition, &FeioparEmployeeSensor), Without<Feiopar>>,
) {
    for (entity, position, mut state, mut target, mut timers, mut meters, mut stats) in
        feiopars.iter_mut()
    {
        if *state == FeioparState::Dead {
            continue;
        }

        let Some((employee_entity, employee_position, sensor)) =
            nearest_employee(*position, FEIOPAR_FREEZE_VIEW_RANGE, &employees)
        else {
            continue;
        };

        let distance_sq = fixed_distance_sq(*position, employee_position);
        let threatening = sensor.high_threat_level || distance_sq <= fixed_square(FEIOPAR_THREAT_RANGE);

        if threatening && *state != FeioparState::Startled {
            target.has_target = true;
            target.target_entity = employee_entity;
            target.target_stable_id = sensor.stable_id;
            target.last_known_position = employee_position;
            stats.move_speed = I32F32::ZERO;
            timers.startled_ticks = fixed_seconds_to_ticks(FEIOPAR_FREEZE_LOOK_SECONDS, sim_hz.0);
            startled_events.send(FeioparStartledEvent {
                feiopar: entity,
                source: employee_entity,
                high_focus: sensor.high_threat_level,
            });
            set_feiopar_state(entity, &mut state, FeioparState::Startled, &mut state_events);
            continue;
        }

        if *state == FeioparState::Startled {
            if timers.startled_ticks > 0 {
                timers.startled_ticks -= 1;
                continue;
            }

            stats.move_speed = FEIOPAR_CLOSE_STALK_SPEED;
            set_feiopar_state(entity, &mut state, FeioparState::Stalking, &mut state_events);
        }

        if sensor.looking_at_feiopar
            && sensor.look_angle_degrees <= FEIOPAR_FREEZE_FOV_DEGREES
            && distance_sq <= fixed_square(FEIOPAR_FREEZE_VIEW_RANGE)
        {
            timers.freeze_look_ticks = timers.freeze_look_ticks.saturating_add(1);
        } else {
            timers.freeze_look_ticks = 0;
        }

        if fixed_ticks_to_seconds(timers.freeze_look_ticks, sim_hz.0) >= FEIOPAR_FREEZE_LOOK_SECONDS {
            meters.seen += I32F32::ONE;
            stats.move_speed = I32F32::ZERO;
            frozen_events.send(FeioparFrozenEvent {
                feiopar: entity,
                observer: employee_entity,
                seen_meter: meters.seen,
            });
            set_feiopar_state(entity, &mut state, FeioparState::Frozen, &mut state_events);
        }

        if *state != FeioparState::Frozen {
            continue;
        }

        stats.move_speed = I32F32::ZERO;

        if meters.seen >= FEIOPAR_FROZEN_GIVE_UP_SEEN_METER {
            set_feiopar_state(entity, &mut state, FeioparState::Roaming, &mut state_events);
            continue;
        }

        if meters.seen > FEIOPAR_FROZEN_HIDING_SEEN_METER {
            stats.move_speed = FEIOPAR_HIDING_RUN_SPEED;
            set_feiopar_state(entity, &mut state, FeioparState::Hiding, &mut state_events);
            continue;
        }

        if distance_sq <= fixed_square(FEIOPAR_STALK_FROZEN_RANGE)
            && sensor.look_angle_degrees > FEIOPAR_STALK_FROZEN_LOOK_DEGREES
        {
            timers.stalk_frozen_ticks = timers.stalk_frozen_ticks.saturating_add(1);
        }

        if fixed_ticks_to_seconds(timers.stalk_frozen_ticks, sim_hz.0)
            >= FEIOPAR_STALK_FROZEN_SECONDS
            || (sensor.direct_eye_contact
                && sensor.look_angle_degrees <= FEIOPAR_EYE_CONTACT_DEGREES
                && distance_sq < fixed_square(FEIOPAR_EYE_CONTACT_RANGE))
        {
            stats.move_speed = FEIOPAR_CLOSE_STALK_SPEED;
            set_feiopar_state(entity, &mut state, FeioparState::Stalking, &mut state_events);
        }
    }
}

fn feiopar_noise_reactions(
    sim_hz: Res<SimHz>,
    mut noise_events: EventReader<NoiseEmittedEvent>,
    mut reaction_events: EventWriter<FeioparNoiseReactedEvent>,
    mut startled_events: EventWriter<FeioparStartledEvent>,
    mut state_events: EventWriter<FeioparStateChangedEvent>,
    mut feiopars: Query<
        (
            Entity,
            &SimPosition,
            &mut FeioparState,
            &mut FeioparTimers,
            &mut FeioparMeters,
            &mut UnitStats,
        ),
        With<Feiopar>,
    >,
    employees: Query<(Entity, &FeioparEmployeeSensor), Without<Feiopar>>,
) {
    for event in noise_events.read() {
        if employees.get(event.source).is_ok() {
            continue;
        }

        for (entity, position, mut state, mut timers, mut meters, mut stats) in feiopars.iter_mut() {
            if *state == FeioparState::Dead {
                continue;
            }

            if timers.last_noise_ticks < fixed_seconds_to_ticks(FEIOPAR_NOISE_SUPPRESS_SECONDS, sim_hz.0) {
                continue;
            }

            let distance_sq = fixed_distance_sq(*position, event.position);
            let direct = distance_sq <= fixed_square(FEIOPAR_NOISE_DIRECT_RANGE);
            let focused = meters.focus > I32F32::ZERO
                && distance_sq >= fixed_square(FEIOPAR_NOISE_FOCUSED_MIN_RANGE)
                && distance_sq <= fixed_square(FEIOPAR_NOISE_FOCUSED_MAX_RANGE);

            if !direct && !focused {
                continue;
            }

            let loud_close = distance_sq <= fixed_square(FEIOPAR_LOUD_NOISE_RANGE)
                && event.amount * event.amount > distance_sq * distance_sq;

            meters.focus = if loud_close { I32F32::lit("2") } else { I32F32::ONE };
            timers.last_noise_ticks = 0;
            reaction_events.send(FeioparNoiseReactedEvent {
                feiopar: entity,
                noise_source: event.source,
                sound_position: event.position,
                high_focus: loud_close,
            });

            if loud_close {
                stats.move_speed = I32F32::ZERO;
                startled_events.send(FeioparStartledEvent {
                    feiopar: entity,
                    source: event.source,
                    high_focus: true,
                });
                set_feiopar_state(entity, &mut state, FeioparState::Startled, &mut state_events);
            }
        }
    }

    for (_entity, _position, _state, mut timers, _meters, _stats) in feiopars.iter_mut() {
        if timers.last_noise_ticks != u32::MAX {
            timers.last_noise_ticks = timers.last_noise_ticks.saturating_add(1);
        }
    }
}

fn feiopar_attack_target(
    sim_hz: Res<SimHz>,
    mut state_events: EventWriter<FeioparStateChangedEvent>,
    mut scream_events: EventWriter<FeioparAttackScreamEvent>,
    mut scratch_events: EventWriter<FeioparScratchHitEvent>,
    mut damage_events: EventWriter<IncomingDamageEvent>,
    mut feiopars: Query<
        (
            Entity,
            &SimPosition,
            &mut FeioparState,
            &mut FeioparTarget,
            &mut FeioparTimers,
            &FeioparMeters,
        ),
        With<Feiopar>,
    >,
    employees: Query<(Entity, &SimPosition, &FeioparEmployeeSensor), Without<Feiopar>>,
) {
    for (entity, position, mut state, mut target, mut timers, meters) in feiopars.iter_mut() {
        if *state != FeioparState::Attacking {
            continue;
        }

        timers.attack_ticks = timers.attack_ticks.saturating_add(1);
        timers.last_touch_ticks = timers.last_touch_ticks.saturating_add(1);

        let Some((employee_position, sensor)) =
            employee_position_by_stable_id(target.target_stable_id, &employees)
        else {
            target.has_target = false;
            set_feiopar_state(entity, &mut state, FeioparState::Stalking, &mut state_events);
            continue;
        };

        if !sensor.is_alive || sensor.in_ship_with_doors_closed {
            target.has_target = false;
            set_feiopar_state(entity, &mut state, FeioparState::Stalking, &mut state_events);
            continue;
        }

        if meters.scared >= FEIOPAR_SCARED_FLEE_LOW {
            set_feiopar_state(entity, &mut state, FeioparState::Roaming, &mut state_events);
            continue;
        }

        if fixed_distance_sq(*position, employee_position) <= fixed_square(FEIOPAR_ATTACK_RANGE) {
            timers.last_touch_ticks = 0;
            scream_events.send(FeioparAttackScreamEvent {
                feiopar: entity,
                target: target.target_entity,
            });
            scratch_events.send(FeioparScratchHitEvent {
                feiopar: entity,
                target: target.target_entity,
                damage: FEIOPAR_ATTACK_DAMAGE,
                small_knockback: true,
                bloody_scratches: true,
            });
            damage_events.send(IncomingDamageEvent {
                target: target.target_entity,
                raw_amount: FEIOPAR_ATTACK_DAMAGE,
                damage_type: DamageType::Standard,
                source: entity,
            });
        }

        if fixed_ticks_to_seconds(timers.attack_ticks, sim_hz.0) >= FEIOPAR_ATTACK_GIVE_UP_SECONDS
            && fixed_ticks_to_seconds(timers.last_touch_ticks, sim_hz.0)
                >= FEIOPAR_ATTACK_TOUCH_COOLDOWN_SECONDS
        {
            set_feiopar_state(entity, &mut state, FeioparState::Stalking, &mut state_events);
        }
    }
}

fn feiopar_update_scared_meter(
    sim_hz: Res<SimHz>,
    mut state_events: EventWriter<FeioparStateChangedEvent>,
    mut meter_events: EventWriter<FeioparScaredMeterChangedEvent>,
    mut feiopars: Query<
        (
            Entity,
            &SimPosition,
            &mut FeioparState,
            &FeioparTarget,
            &mut FeioparTimers,
            &mut FeioparMeters,
        ),
        With<Feiopar>,
    >,
    employees: Query<(Entity, &SimPosition, &FeioparEmployeeSensor), Without<Feiopar>>,
) {
    for (entity, position, mut state, target, mut timers, mut meters) in feiopars.iter_mut() {
        if *state != FeioparState::Attacking || !target.has_target {
            continue;
        }

        let Some((employee_position, sensor)) =
            employee_position_by_stable_id(target.target_stable_id, &employees)
        else {
            continue;
        };

        if sensor.sprinting_toward_feiopar
            && sensor.looking_at_feiopar
            && fixed_distance_sq(*position, employee_position) <= fixed_square(FEIOPAR_SPRINT_SCARE_RANGE)
        {
            meters.scared += FEIOPAR_SCARED_FILL_PER_SECOND / sim_hz.0;
        } else if meters.scared > I32F32::ZERO {
            meters.scared -= FEIOPAR_SCARED_DRAIN_MIN_PER_SECOND / sim_hz.0;
            if meters.scared < I32F32::ZERO {
                meters.scared = I32F32::ZERO;
            }
        }

        meter_events.send(FeioparScaredMeterChangedEvent {
            feiopar: entity,
            target: target.target_entity,
            scared_meter: meters.scared,
        });

        if meters.scared >= FEIOPAR_SCARED_FLEE_LOW {
            timers.low_scared_attack_ticks = timers.low_scared_attack_ticks.saturating_add(1);
        } else {
            timers.low_scared_attack_ticks = 0;
        }

        if meters.scared >= FEIOPAR_SCARED_FLEE_HIGH {
            timers.high_scared_attack_ticks = timers.high_scared_attack_ticks.saturating_add(1);
        } else {
            timers.high_scared_attack_ticks = 0;
        }

        if fixed_ticks_to_seconds(timers.low_scared_attack_ticks, sim_hz.0)
            >= FEIOPAR_SCARED_FLEE_LOW_SECONDS
            || fixed_ticks_to_seconds(timers.high_scared_attack_ticks, sim_hz.0)
                >= FEIOPAR_SCARED_FLEE_HIGH_SECONDS
        {
            set_feiopar_state(entity, &mut state, FeioparState::Roaming, &mut state_events);
        }
    }
}

fn feiopar_apply_stun_and_damage(
    mut damage_events: EventReader<IncomingDamageEvent>,
    mut taken_events: EventWriter<FeioparDamageTakenEvent>,
    mut defeated_events: EventWriter<FeioparDefeatedEvent>,
    mut stun_events: EventWriter<FeioparStunAdjustedEvent>,
    mut state_events: EventWriter<FeioparStateChangedEvent>,
    mut feiopars: Query<
        (
            Entity,
            &mut Health,
            &mut FeioparState,
            &mut UnitStats,
            &FeioparTreeTarget,
        ),
        With<Feiopar>,
    >,
) {
    for event in damage_events.read() {
        let Ok((entity, mut health, mut state, mut stats, tree_target)) =
            feiopars.get_mut(event.target)
        else {
            continue;
        };

        if *state == FeioparState::Dead {
            continue;
        }

        let damage = if event.raw_amount <= I32F32::ZERO {
            FEIOPAR_WEAPON_HIT_DAMAGE
        } else {
            event.raw_amount
        };

        health.current -= damage;
        if health.current < I32F32::ZERO {
            health.current = I32F32::ZERO;
        }

        taken_events.send(FeioparDamageTakenEvent {
            feiopar: entity,
            source: event.source,
            damage,
            remaining_health: health.current,
        });

        stun_events.send(FeioparStunAdjustedEvent {
            feiopar: entity,
            source: event.source,
            stun_multiplier: FEIOPAR_STUN_MULTIPLIER,
        });

        if *state == FeioparState::Stalking || *state == FeioparState::Startled {
            stats.move_speed = FEIOPAR_ROAM_SPEED;
            set_feiopar_state(entity, &mut state, FeioparState::Roaming, &mut state_events);
        }

        if health.current <= I32F32::ZERO {
            let fell_from_tree = tree_target.has_tree
                && (*state == FeioparState::Climbing || *state == FeioparState::TreeIdle);
            *state = FeioparState::Dead;
            defeated_events.send(FeioparDefeatedEvent {
                feiopar: entity,
                source: event.source,
                fell_from_tree,
            });
        }
    }
}

fn feiopar_door_attempt_speed(
    mut events: EventReader<FeioparDoorAttemptEvent>,
    mut resolved_events: EventWriter<FeioparDoorAttemptResolvedEvent>,
    feiopars: Query<(), With<Feiopar>>,
) {
    for event in events.read() {
        if feiopars.get(event.feiopar).is_err() {
            continue;
        }

        resolved_events.send(FeioparDoorAttemptResolvedEvent {
            feiopar: event.feiopar,
            door: event.door,
            adjusted_open_ticks: fixed_ticks_scaled(
                event.base_open_ticks,
                FEIOPAR_DOOR_SPEED_MULTIPLIER,
            ),
        });
    }
}

fn feiopar_zap_gun_difficulty(
    mut events: EventReader<FeioparZapGunTargetedEvent>,
    mut difficulty_events: EventWriter<FeioparZapGunDifficultyEvent>,
    feiopars: Query<(), With<Feiopar>>,
) {
    for event in events.read() {
        if feiopars.get(event.feiopar).is_err() {
            continue;
        }

        difficulty_events.send(FeioparZapGunDifficultyEvent {
            feiopar: event.feiopar,
            wielder: event.wielder,
            difficulty: FEIOPAR_ZAP_GUN_DIFFICULTY,
        });
    }
}

fn feiopar_checksum(
    mut checksum: ResMut<SimChecksumState>,
    tick: Res<SimTick>,
    sim_hz: Res<SimHz>,
    feiopars: Query<
        (
            &Feiopar,
            &SimPosition,
            &Health,
            &UnitStats,
            &FeioparState,
            &FeioparTarget,
            &FeioparTreeTarget,
            &FeioparTimers,
            &FeioparMeters,
        ),
        With<Feiopar>,
    >,
) {
    checksum.accumulate(tick.0);
    checksum.accumulate(sim_hz.0.to_bits() as u64);
    checksum.accumulate(FEIOPAR_SOURCE_REVISION as u64);
    checksum.accumulate(FEIOPAR_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(FEIOPAR_HP.to_bits() as u64);
    checksum.accumulate(FEIOPAR_POWER_LEVEL.to_bits() as u64);
    checksum.accumulate(FEIOPAR_MAX_SPAWNED as u64);
    checksum.accumulate(FEIOPAR_ATTACK_DAMAGE.to_bits() as u64);
    checksum.accumulate(FEIOPAR_STUN_MULTIPLIER.to_bits() as u64);
    checksum.accumulate(FEIOPAR_CAN_SEE_THROUGH_FOG as u64);
    checksum.accumulate(FEIOPAR_SPAWN_DELAY_SECONDS.to_bits() as u64);
    checksum.accumulate(FEIOPAR_DOOR_SPEED_MULTIPLIER.to_bits() as u64);
    checksum.accumulate(FEIOPAR_ZAP_GUN_DIFFICULTY.to_bits() as u64);

    accumulate_str(&mut checksum, 0x1000, FEIOPAR_ID);
    accumulate_str(&mut checksum, 0x1001, FEIOPAR_NAME);
    accumulate_str(&mut checksum, 0x1002, FEIOPAR_TYPE);
    accumulate_str(&mut checksum, 0x1003, FEIOPAR_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, FEIOPAR_SOURCE_URL);
    accumulate_str(&mut checksum, 0x1005, FEIOPAR_EXTRACTED_AT);
    accumulate_str(&mut checksum, 0x1006, FEIOPAR_DWELLS);
    accumulate_str(&mut checksum, 0x1007, FEIOPAR_DANGER);
    accumulate_str(&mut checksum, 0x1008, FEIOPAR_SCIENTIFIC_NAME);
    accumulate_str(&mut checksum, 0x1009, FEIOPAR_INTERNAL_NAME);
    accumulate_str(&mut checksum, 0x100A, FEIOPAR_PIP_SIZE);

    for dependency in FEIOPAR_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for behavior in FEIOPAR_FRONTMATTER_BEHAVIOR {
        accumulate_str(&mut checksum, 0x3000, behavior);
    }

    for rule in FEIOPAR_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x4000, rule.condition);
        accumulate_str(&mut checksum, 0x4001, rule.outcome);
    }

    for (feiopar, position, health, stats, state, target, tree_target, timers, meters) in
        feiopars.iter()
    {
        checksum.accumulate(feiopar.stable_id);
        checksum.accumulate(feiopar.spawn_delay_ticks_remaining as u64);
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(health.current.to_bits() as u64);
        checksum.accumulate(health.max.to_bits() as u64);
        checksum.accumulate(stats.move_speed.to_bits() as u64);
        checksum.accumulate(stats.attack_range.to_bits() as u64);
        checksum.accumulate(stats.attack_damage.to_bits() as u64);
        checksum.accumulate(stats.attack_speed.to_bits() as u64);
        checksum.accumulate(stats.watch_range.to_bits() as u64);
        checksum.accumulate(feiopar_state_bits(*state));
        checksum.accumulate(target.has_target as u64);
        checksum.accumulate(target.target_stable_id);
        checksum.accumulate(target.last_known_position.x.to_bits() as u64);
        checksum.accumulate(target.last_known_position.y.to_bits() as u64);
        checksum.accumulate(target.target_visible as u64);
        checksum.accumulate(tree_target.has_tree as u64);
        checksum.accumulate(tree_target.tree_stable_id);
        checksum.accumulate(tree_target.position.x.to_bits() as u64);
        checksum.accumulate(tree_target.position.y.to_bits() as u64);
        checksum.accumulate(tree_target.leap_search_range.to_bits() as u64);
        checksum.accumulate(tree_target.failed_leap_attempts as u64);
        checksum.accumulate(timers.seen_ticks as u64);
        checksum.accumulate(timers.hiding_ticks as u64);
        checksum.accumulate(timers.tree_idle_ticks as u64);
        checksum.accumulate(timers.freeze_look_ticks as u64);
        checksum.accumulate(timers.seen_close_ticks as u64);
        checksum.accumulate(timers.stalk_frozen_ticks as u64);
        checksum.accumulate(timers.no_hiding_tree_ticks as u64);
        checksum.accumulate(timers.last_noise_ticks as u64);
        checksum.accumulate(timers.attack_ticks as u64);
        checksum.accumulate(timers.last_touch_ticks as u64);
        checksum.accumulate(timers.low_scared_attack_ticks as u64);
        checksum.accumulate(timers.high_scared_attack_ticks as u64);
        checksum.accumulate(timers.startled_ticks as u64);
        checksum.accumulate(meters.anger.to_bits() as u64);
        checksum.accumulate(meters.seen.to_bits() as u64);
        checksum.accumulate(meters.scared.to_bits() as u64);
        checksum.accumulate(meters.focus.to_bits() as u64);
    }
}

fn nearest_employee(
    origin: SimPosition,
    range: I32F32,
    employees: &Query<(Entity, &SimPosition, &FeioparEmployeeSensor), Without<Feiopar>>,
) -> Option<(Entity, SimPosition, FeioparEmployeeSensor)> {
    let mut best: Option<(Entity, SimPosition, FeioparEmployeeSensor, I32F32)> = None;
    let range_sq = fixed_square(range);

    for (entity, position, sensor) in employees.iter() {
        if !sensor.is_alive {
            continue;
        }

        let distance_sq = fixed_distance_sq(origin, *position);
        if distance_sq > range_sq {
            continue;
        }

        match best {
            Some((_best_entity, _best_position, _best_sensor, best_distance_sq))
                if distance_sq >= best_distance_sq => {}
            _ => best = Some((entity, *position, *sensor, distance_sq)),
        }
    }

    best.map(|(entity, position, sensor, _distance_sq)| (entity, position, sensor))
}

fn nearest_looking_employee(
    origin: SimPosition,
    range: I32F32,
    employees: &Query<(Entity, &SimPosition, &FeioparEmployeeSensor), Without<Feiopar>>,
) -> Option<(Entity, SimPosition, FeioparEmployeeSensor)> {
    let mut best: Option<(Entity, SimPosition, FeioparEmployeeSensor, I32F32)> = None;
    let range_sq = fixed_square(range);

    for (entity, position, sensor) in employees.iter() {
        if !sensor.is_alive || !sensor.looking_at_feiopar {
            continue;
        }

        let distance_sq = fixed_distance_sq(origin, *position);
        if distance_sq > range_sq {
            continue;
        }

        match best {
            Some((_best_entity, _best_position, _best_sensor, best_distance_sq))
                if distance_sq >= best_distance_sq => {}
            _ => best = Some((entity, *position, *sensor, distance_sq)),
        }
    }

    best.map(|(entity, position, sensor, _distance_sq)| (entity, position, sensor))
}

fn nearest_suitable_tree(
    origin: SimPosition,
    trees: &Query<(Entity, &SimPosition, &FeioparTreeSensor), Without<Feiopar>>,
    employees: &Query<(Entity, &SimPosition, &FeioparEmployeeSensor), Without<Feiopar>>,
) -> Option<(Entity, SimPosition, FeioparTreeSensor)> {
    let mut best: Option<(Entity, SimPosition, FeioparTreeSensor, I32F32)> = None;

    for (tree_entity, tree_position, tree_sensor) in trees.iter() {
        if !tree_sensor.reachable {
            continue;
        }

        let feiopar_distance_sq = fixed_distance_sq(origin, *tree_position);
        if employee_closer_to_tree(*tree_position, feiopar_distance_sq, employees) {
            continue;
        }

        match best {
            Some((_best_entity, _best_position, _best_sensor, best_distance_sq))
                if feiopar_distance_sq >= best_distance_sq => {}
            _ => best = Some((tree_entity, *tree_position, *tree_sensor, feiopar_distance_sq)),
        }
    }

    best.map(|(entity, position, sensor, _distance_sq)| (entity, position, sensor))
}

fn leap_tree_target(
    origin: SimPosition,
    range: I32F32,
    prefer_hidden: bool,
    trees: &Query<(Entity, &SimPosition, &FeioparTreeSensor), Without<Feiopar>>,
    employees: &Query<(Entity, &SimPosition, &FeioparEmployeeSensor), Without<Feiopar>>,
) -> Option<(Entity, SimPosition, FeioparTreeSensor)> {
    let mut best: Option<(Entity, SimPosition, FeioparTreeSensor, I32F32)> = None;
    let range_sq = fixed_square(range);

    for (tree_entity, tree_position, tree_sensor) in trees.iter() {
        if !tree_sensor.reachable {
            continue;
        }

        let distance_sq = fixed_distance_sq(origin, *tree_position);
        if distance_sq > range_sq {
            continue;
        }

        if prefer_hidden && !tree_sensor.hidden_from_all_employees {
            continue;
        }

        if !prefer_hidden && !employee_closer_to_tree(*tree_position, distance_sq, employees) {
            continue;
        }

        if tree_sensor.in_any_employee_line_of_sight {
            continue;
        }

        match best {
            Some((_best_entity, _best_position, _best_sensor, best_distance_sq))
                if distance_sq >= best_distance_sq => {}
            _ => best = Some((tree_entity, *tree_position, *tree_sensor, distance_sq)),
        }
    }

    best.map(|(entity, position, sensor, _distance_sq)| (entity, position, sensor))
}

fn employee_closer_to_tree(
    tree_position: SimPosition,
    feiopar_distance_sq: I32F32,
    employees: &Query<(Entity, &SimPosition, &FeioparEmployeeSensor), Without<Feiopar>>,
) -> bool {
    for (_entity, employee_position, sensor) in employees.iter() {
        if !sensor.is_alive {
            continue;
        }

        if fixed_distance_sq(tree_position, *employee_position) < feiopar_distance_sq {
            return true;
        }
    }

    false
}

fn employee_position_by_stable_id(
    stable_id: u64,
    employees: &Query<(Entity, &SimPosition, &FeioparEmployeeSensor), Without<Feiopar>>,
) -> Option<(SimPosition, FeioparEmployeeSensor)> {
    for (_entity, position, sensor) in employees.iter() {
        if sensor.stable_id == stable_id {
            return Some((*position, *sensor));
        }
    }

    None
}

fn set_feiopar_state(
    feiopar: Entity,
    state: &mut FeioparState,
    next: FeioparState,
    events: &mut EventWriter<FeioparStateChangedEvent>,
) {
    if *state == next {
        return;
    }

    let previous = *state;
    *state = next;
    events.send(FeioparStateChangedEvent {
        feiopar,
        from: previous,
        to: next,
    });
}

fn move_axis_toward(position: &mut SimPosition, target: SimPosition, max_step: I32F32) {
    position.x = move_scalar_toward(position.x, target.x, max_step);
    position.y = move_scalar_toward(position.y, target.y, max_step);
}

fn move_scalar_toward(current: I32F32, target: I32F32, max_step: I32F32) -> I32F32 {
    if current < target {
        let next = current + max_step;
        if next > target {
            target
        } else {
            next
        }
    } else if current > target {
        let next = current - max_step;
        if next < target {
            target
        } else {
            next
        }
    } else {
        current
    }
}

fn fixed_distance_sq(a: SimPosition, b: SimPosition) -> I32F32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    dx * dx + dy * dy
}

fn fixed_square(value: I32F32) -> I32F32 {
    value * value
}

fn fixed_seconds_to_ticks(seconds: I32F32, sim_hz: I32F32) -> u32 {
    let ticks = seconds * sim_hz;
    if ticks <= I32F32::ZERO {
        0
    } else {
        ticks.ceil().to_num::<u32>()
    }
}

fn fixed_ticks_to_seconds(ticks: u32, sim_hz: I32F32) -> I32F32 {
    if sim_hz <= I32F32::ZERO {
        I32F32::ZERO
    } else {
        I32F32::from_num(ticks) / sim_hz
    }
}

fn fixed_ticks_scaled(base_ticks: u32, multiplier: I32F32) -> u32 {
    let ticks = I32F32::from_num(base_ticks) / multiplier;
    if ticks <= I32F32::ONE {
        1
    } else {
        ticks.ceil().to_num::<u32>()
    }
}

fn feiopar_state_bits(state: FeioparState) -> u64 {
    match state {
        FeioparState::Roaming => 0,
        FeioparState::Climbing => 1,
        FeioparState::TreeIdle => 2,
        FeioparState::Hiding => 3,
        FeioparState::Stalking => 4,
        FeioparState::Startled => 5,
        FeioparState::Frozen => 6,
        FeioparState::Attacking => 7,
        FeioparState::Dead => 8,
    }
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt ^ value.len() as u64);

    for (index, byte) in value.bytes().enumerate() {
        checksum.accumulate(salt ^ ((index as u64) << 8) ^ byte as u64);
    }
}