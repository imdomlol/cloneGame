// Sources: vault/indoor_entity_pages/maneater.md
use bevy::prelude::*;
use fixed::types::I32F32;
use rand_core::RngCore;

use crate::sim::{
    tick_rng, DamageType, GameSeed, Health, IncomingDamageEvent, SimChecksumState, SimHz,
    SimPosition, SimTick, UnitStats,
};

pub const MANEATER_ID: &str = "maneater";
pub const MANEATER_NAME: &str = "Maneater";
pub const MANEATER_TYPE: &str = "indoor_entity_pages";
pub const MANEATER_SUBTYPE: &str = "creature";
pub const MANEATER_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Maneater";
pub const MANEATER_SOURCE_REVISION: u32 = 21401;
pub const MANEATER_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const MANEATER_CONFIDENCE_BASIS_POINTS: u16 = 94;

pub const MANEATER_DWELLS: &str = "Indoor (can be brought outdoors)";
pub const MANEATER_DANGER: &str = "1000%";
pub const MANEATER_SCIENTIFIC_NAME: &str = "Periplaneta clamorus";
pub const MANEATER_HP: I32F32 = I32F32::lit("5");
pub const MANEATER_BABY_HP: &str = "invulnerable";
pub const MANEATER_POWER_LEVEL: I32F32 = I32F32::lit("2");
pub const MANEATER_MAX_SPAWNED: usize = 1;
pub const MANEATER_ATTACK_DAMAGE: &str = "Instant Kill";
pub const MANEATER_ATTACK_SPEED_MIN: I32F32 = I32F32::lit("0.5");
pub const MANEATER_ATTACK_SPEED_MAX: I32F32 = I32F32::lit("1");
pub const MANEATER_STUN_MULTIPLIER: I32F32 = I32F32::lit("0.25");
pub const MANEATER_ZAP_GUN_DIFFICULTY: I32F32 = I32F32::lit("0.8");
pub const MANEATER_INTERNAL_NAME: &str = "CaveDweller";
pub const MANEATER_PIP_SIZE: &str = "Small in baby phases; Big as an adult";
pub const MANEATER_DOOR_SPEED_MULTIPLIER: I32F32 = I32F32::lit("1.25");
pub const MANEATER_BABY_LIKING_FIRST_PLAYER_CHANCE: I32F32 = I32F32::lit("1.0");
pub const MANEATER_BABY_LIKING_OTHER_PLAYER_CHANCE: I32F32 = I32F32::lit("0.5");
pub const MANEATER_BABY_CRYING_TRANSFORM_SECONDS: I32F32 = I32F32::lit("17");
pub const MANEATER_BABY_FOLLOW_AVERAGE_SECONDS: I32F32 = I32F32::lit("22");
pub const MANEATER_BABY_INITIAL_WANDER_SECONDS: I32F32 = I32F32::lit("26");
pub const MANEATER_BABY_SEEK_PLAYER_SECONDS_MIN: I32F32 = I32F32::lit("14");
pub const MANEATER_BABY_SEEK_PLAYER_SECONDS_MAX: I32F32 = I32F32::lit("26");
pub const MANEATER_ITEM_EAT_CHECK_INTERVAL_SECONDS: I32F32 = I32F32::lit("0.2");
pub const MANEATER_ITEM_EAT_CHANCE: I32F32 = I32F32::lit("0.08");
pub const MANEATER_ITEM_EAT_COOLDOWN_SECONDS: I32F32 = I32F32::lit("25");
pub const MANEATER_ADULT_TRANSFORM_SECONDS: I32F32 = I32F32::lit("3");
pub const MANEATER_ADULT_ATTACK_ROUND_SECONDS_MIN: I32F32 = I32F32::lit("2.5");
pub const MANEATER_ADULT_ATTACK_ROUND_SECONDS_MAX: I32F32 = I32F32::lit("3");
pub const MANEATER_ADULT_ATTACK_SPRINT_SECONDS_MIN: I32F32 = I32F32::lit("0.5");
pub const MANEATER_ADULT_ATTACK_SPRINT_SECONDS_MAX: I32F32 = I32F32::lit("1");
pub const MANEATER_ADULT_ATTACK_PAUSE_SECONDS_MIN: I32F32 = I32F32::lit("1.5");
pub const MANEATER_ADULT_ATTACK_PAUSE_SECONDS_MAX: I32F32 = I32F32::lit("2");
pub const MANEATER_TERRITORY_INITIAL_UNITS: I32F32 = I32F32::lit("30");
pub const MANEATER_TERRITORY_MAX_UNITS: I32F32 = I32F32::lit("100");
pub const MANEATER_TERRITORY_GROWTH_SECONDS: I32F32 = I32F32::lit("45");
pub const MANEATER_STALKING_ATTACK_TRIGGER_NODES: u16 = 7;
pub const MANEATER_OUTDOOR_ATTACK_RADIUS_UNITS: I32F32 = I32F32::lit("23");
pub const MANEATER_OUTDOOR_LUNGE_SPEED_MULTIPLIER: I32F32 = I32F32::lit("2");
pub const MANEATER_ADULT_NON_INSTANT_KILL_DAMAGE: I32F32 = I32F32::lit("1");
pub const MANEATER_SPAWN_MINESHAFT_MULTIPLIER: I32F32 = I32F32::lit("1.7");

pub const MANEATER_NO_PLAYER_SECONDS: I32F32 = I32F32::lit("9");
pub const MANEATER_DISLIKED_HOLD_SECONDS: I32F32 = I32F32::lit("30");
pub const MANEATER_CRYING_DISLIKED_HELD_MULTIPLIER: u32 = 3;
pub const MANEATER_CRYING_RUNNING_MULTIPLIER: u32 = 2;
pub const MANEATER_INDOOR_LUNGE_DISTANCE: I32F32 = I32F32::lit("4");
pub const MANEATER_ATTACK_EXIT_DISTANCE: I32F32 = I32F32::lit("14");
pub const MANEATER_INSTANT_KILL_DAMAGE: I32F32 = I32F32::lit("1000000");

pub const MANEATER_DEPENDS_ON: [&str; 0] = [];
pub const MANEATER_FRONTMATTER_BEHAVIOR: [&str; 6] = [
    "Baby phases are wandering, following, impatient, and crying.",
    "Adults use guarding, stalking, and attacking phases.",
    "The first player seen is imprinted on; later players can still be liked, but only with a lower chance.",
    "A baby can eat ground items, but repeated eating is throttled by a cooldown.",
    "Sustained crying is the main path from baby form to adulthood.",
    "Outdoor adults are more aggressive because they move faster while lunging and can threaten everyone nearby.",
];

pub const MANEATER_BEHAVIORAL_MECHANICS: [ManeaterBehaviorRule; 25] = [
    ManeaterBehaviorRule {
        condition: "spawned",
        outcome: "start in baby form and imprint on the first player seen, with a 100% liking chance for that player and a 50% liking chance for later players",
    },
    ManeaterBehaviorRule {
        condition: "in the first wandering pass",
        outcome: "wander randomly for 26 seconds, then move toward the nearest player for 14 to 26 seconds, repeating until a player is found",
    },
    ManeaterBehaviorRule {
        condition: "following a liked player",
        outcome: "remain in Following for about 22 seconds on average",
    },
    ManeaterBehaviorRule {
        condition: "following a disliked player",
        outcome: "the follow timer is shorter",
    },
    ManeaterBehaviorRule {
        condition: "a baby cannot find a player for about 9 seconds",
        outcome: "it enters Crying",
    },
    ManeaterBehaviorRule {
        condition: "a baby is exposed to loud non-movement sounds, melee hits, dead bodies, shaking, falls, facility exits, or other listed triggers",
        outcome: "it enters Crying",
    },
    ManeaterBehaviorRule {
        condition: "a baby is rocked while crying indoors",
        outcome: "it can return to Following",
    },
    ManeaterBehaviorRule {
        condition: "a baby is rocked while outdoors",
        outcome: "it cannot return to Following until it is brought back inside",
    },
    ManeaterBehaviorRule {
        condition: "crying accumulates to 17 total seconds indoors",
        outcome: "the baby transforms into an adult irreversibly",
    },
    ManeaterBehaviorRule {
        condition: "a baby is held by a player it does not like for 30 seconds",
        outcome: "it enters Impatient",
    },
    ManeaterBehaviorRule {
        condition: "a baby is crying because it is being held by a disliked player",
        outcome: "the path to adulthood advances 3x faster",
    },
    ManeaterBehaviorRule {
        condition: "a baby is running while crying",
        outcome: "the transform timer advances 2x faster than normal",
    },
    ManeaterBehaviorRule {
        condition: "a baby is struck by instant-kill damage",
        outcome: "it transforms into an adult instead of dying",
    },
    ManeaterBehaviorRule {
        condition: "a baby sees a ground item",
        outcome: "it checks every 0.2 seconds with an 8% chance to eat it",
    },
    ManeaterBehaviorRule {
        condition: "a baby has eaten within the last 25 seconds",
        outcome: "it does not attempt to eat again",
    },
    ManeaterBehaviorRule {
        condition: "an adult enters Guarding",
        outcome: "it selects a nest node, prefers cave nodes, and begins territory play from 30 units wide",
    },
    ManeaterBehaviorRule {
        condition: "players remain inside while the adult is guarding",
        outcome: "its territory expands from 30 units to 100 units over 45 seconds",
    },
    ManeaterBehaviorRule {
        condition: "no players are inside",
        outcome: "the adult's territory shrinks back to 30 units",
    },
    ManeaterBehaviorRule {
        condition: "an adult is in Stalking and a player comes within 7 nodes",
        outcome: "it enters Attacking",
    },
    ManeaterBehaviorRule {
        condition: "an adult is attacking indoors",
        outcome: "each attack round lasts 2.5 to 3.0 seconds, with a 0.5 to 1.0 second sprint and a 1.5 to 2.0 second pause",
    },
    ManeaterBehaviorRule {
        condition: "an adult makes contact during an attack",
        outcome: "the target dies instantly",
    },
    ManeaterBehaviorRule {
        condition: "an indoor target is farther than 4 units",
        outcome: "the adult chases until it can lunge",
    },
    ManeaterBehaviorRule {
        condition: "all players are more than 14 units away or leave the adult's territory",
        outcome: "it exits Attacking",
    },
    ManeaterBehaviorRule {
        condition: "outdoors",
        outcome: "the adult lunges at 2x speed and attacks every employee within 23 units",
    },
    ManeaterBehaviorRule {
        condition: "struck by non-instant-kill damage as an adult",
        outcome: "it takes 1 damage per hit",
    },
];

pub struct ManeaterPlugin;

impl Plugin for ManeaterPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnManeaterEvent>()
            .add_event::<ManeaterStateChangedEvent>()
            .add_event::<ManeaterImprintRollEvent>()
            .add_event::<ManeaterCryingTriggeredEvent>()
            .add_event::<ManeaterRockedEvent>()
            .add_event::<ManeaterTransformStartedEvent>()
            .add_event::<ManeaterTransformCompletedEvent>()
            .add_event::<ManeaterItemEatCheckEvent>()
            .add_event::<ManeaterItemEatenEvent>()
            .add_event::<ManeaterDoorAttemptEvent>()
            .add_event::<ManeaterDoorAttemptResolvedEvent>()
            .add_event::<ManeaterStunAppliedEvent>()
            .add_event::<ManeaterStunAdjustedEvent>()
            .add_event::<ManeaterStunIgnoredEvent>()
            .add_event::<ManeaterZapGunTargetedEvent>()
            .add_event::<ManeaterZapGunDifficultyEvent>()
            .add_event::<ManeaterContactKillEvent>()
            .add_event::<ManeaterDamageTakenEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_maneater,
                    maneater_find_and_imprint_player,
                    maneater_wander_seek_or_follow,
                    maneater_trigger_crying_from_sensors,
                    maneater_rock_crying_baby,
                    maneater_advance_crying_transform,
                    maneater_complete_adult_transform,
                    maneater_item_eating,
                    maneater_guarding_territory,
                    maneater_stalking_trigger_attack,
                    maneater_indoor_attack_timing,
                    maneater_outdoor_lunge_attack,
                    maneater_contact_instant_kill,
                    maneater_damage_rules,
                    maneater_door_attempt_speed,
                    maneater_apply_stun_rules,
                    maneater_report_zap_gun_difficulty,
                    maneater_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ManeaterBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Maneater;

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ManeaterForm {
    #[default]
    Baby,
    Transforming,
    Adult,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ManeaterBabyPhase {
    #[default]
    Wandering,
    SeekingPlayer,
    Following,
    Impatient,
    Crying,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ManeaterAdultPhase {
    #[default]
    Guarding,
    Stalking,
    Attacking,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ManeaterEmployeeSensor {
    pub stable_id: u64,
    pub can_be_seen_by_baby: bool,
    pub is_liked_by_baby: bool,
    pub holding_baby: bool,
    pub inside_facility: bool,
    pub within_stalking_nodes: u16,
    pub within_territory: bool,
    pub touching_maneater: bool,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ManeaterStimuli {
    pub loud_non_movement_sound: bool,
    pub melee_hit: bool,
    pub dead_body_nearby: bool,
    pub shaking: bool,
    pub fall: bool,
    pub facility_exit: bool,
    pub running_while_crying: bool,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ManeaterEnvironment {
    pub outdoors: bool,
    pub players_inside: bool,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ManeaterImprint {
    pub has_seen_first_player: bool,
    pub target_stable_id: u64,
    pub liked_target: bool,
    pub disliked_hold_ticks: u32,
    pub no_player_ticks: u32,
}

impl Default for ManeaterImprint {
    fn default() -> Self {
        Self {
            has_seen_first_player: false,
            target_stable_id: 0,
            liked_target: false,
            disliked_hold_ticks: 0,
            no_player_ticks: 0,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ManeaterBabyTiming {
    pub phase_ticks: u32,
    pub follow_ticks: u32,
    pub seek_ticks: u32,
    pub crying_ticks: u32,
    pub item_check_ticks: u32,
    pub item_cooldown_ticks: u32,
}

impl Default for ManeaterBabyTiming {
    fn default() -> Self {
        Self {
            phase_ticks: 0,
            follow_ticks: 0,
            seek_ticks: 0,
            crying_ticks: 0,
            item_check_ticks: 0,
            item_cooldown_ticks: 0,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ManeaterItemSensor {
    pub ground_item: Option<Entity>,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ManeaterAdultTerritory {
    pub has_nest_node: bool,
    pub nest_node: u64,
    pub prefers_cave_nodes: bool,
    pub radius_units: I32F32,
}

impl Default for ManeaterAdultTerritory {
    fn default() -> Self {
        Self {
            has_nest_node: false,
            nest_node: 0,
            prefers_cave_nodes: true,
            radius_units: MANEATER_TERRITORY_INITIAL_UNITS,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ManeaterAdultTiming {
    pub transform_ticks: u32,
    pub attack_round_ticks: u32,
    pub sprint_ticks: u32,
    pub pause_ticks: u32,
}

impl Default for ManeaterAdultTiming {
    fn default() -> Self {
        Self {
            transform_ticks: 0,
            attack_round_ticks: 0,
            sprint_ticks: 0,
            pause_ticks: 0,
        }
    }
}

#[derive(Bundle)]
pub struct ManeaterBundle {
    pub name: Name,
    pub maneater: Maneater,
    pub position: SimPosition,
    pub health: Health,
    pub stats: UnitStats,
    pub form: ManeaterForm,
    pub baby_phase: ManeaterBabyPhase,
    pub adult_phase: ManeaterAdultPhase,
    pub imprint: ManeaterImprint,
    pub baby_timing: ManeaterBabyTiming,
    pub item_sensor: ManeaterItemSensor,
    pub territory: ManeaterAdultTerritory,
    pub adult_timing: ManeaterAdultTiming,
    pub stimuli: ManeaterStimuli,
    pub environment: ManeaterEnvironment,
}

impl ManeaterBundle {
    pub fn new(event: SpawnManeaterEvent) -> Self {
        Self {
            name: Name::new(MANEATER_NAME),
            maneater: Maneater,
            position: event.position,
            health: Health::full(MANEATER_HP),
            stats: UnitStats {
                move_speed: event.base_move_speed,
                attack_range: MANEATER_INDOOR_LUNGE_DISTANCE,
                attack_damage: MANEATER_INSTANT_KILL_DAMAGE,
                attack_speed: MANEATER_ATTACK_SPEED_MIN,
                watch_range: event.watch_range,
            },
            form: ManeaterForm::Baby,
            baby_phase: ManeaterBabyPhase::Wandering,
            adult_phase: ManeaterAdultPhase::Guarding,
            imprint: ManeaterImprint::default(),
            baby_timing: ManeaterBabyTiming::default(),
            item_sensor: ManeaterItemSensor::default(),
            territory: ManeaterAdultTerritory::default(),
            adult_timing: ManeaterAdultTiming::default(),
            stimuli: ManeaterStimuli::default(),
            environment: ManeaterEnvironment {
                outdoors: event.outdoors,
                players_inside: true,
            },
        }
    }
}

#[derive(Event, Clone, Copy, Debug)]
pub struct SpawnManeaterEvent {
    pub position: SimPosition,
    pub base_move_speed: I32F32,
    pub watch_range: I32F32,
    pub outdoors: bool,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ManeaterStateChangedEvent {
    pub maneater: Entity,
    pub from_form: ManeaterForm,
    pub to_form: ManeaterForm,
    pub from_baby_phase: ManeaterBabyPhase,
    pub to_baby_phase: ManeaterBabyPhase,
    pub from_adult_phase: ManeaterAdultPhase,
    pub to_adult_phase: ManeaterAdultPhase,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ManeaterImprintRollEvent {
    pub maneater: Entity,
    pub employee: Entity,
    pub stable_id: u64,
    pub first_player_seen: bool,
    pub liked: bool,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ManeaterCryingTriggeredEvent {
    pub maneater: Entity,
    pub reason: ManeaterCryingReason,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ManeaterCryingReason {
    NoPlayerFound,
    LoudNonMovementSound,
    MeleeHit,
    DeadBody,
    Shaking,
    Fall,
    FacilityExit,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ManeaterRockedEvent {
    pub maneater: Entity,
    pub indoors: bool,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ManeaterTransformStartedEvent {
    pub maneater: Entity,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ManeaterTransformCompletedEvent {
    pub maneater: Entity,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ManeaterItemEatCheckEvent {
    pub maneater: Entity,
    pub item: Entity,
    pub ate_item: bool,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ManeaterItemEatenEvent {
    pub maneater: Entity,
    pub item: Entity,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ManeaterDoorAttemptEvent {
    pub maneater: Entity,
    pub door: Entity,
    pub base_open_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ManeaterDoorAttemptResolvedEvent {
    pub maneater: Entity,
    pub door: Entity,
    pub adjusted_open_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ManeaterStunAppliedEvent {
    pub maneater: Entity,
    pub base_ticks: u32,
    pub stun_kind: ManeaterStunKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ManeaterStunKind {
    Standard,
    StunGrenade,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ManeaterStunAdjustedEvent {
    pub maneater: Entity,
    pub base_ticks: u32,
    pub adjusted_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ManeaterStunIgnoredEvent {
    pub maneater: Entity,
    pub stun_kind: ManeaterStunKind,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ManeaterZapGunTargetedEvent {
    pub maneater: Entity,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ManeaterZapGunDifficultyEvent {
    pub maneater: Entity,
    pub difficulty_modifier: I32F32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ManeaterContactKillEvent {
    pub maneater: Entity,
    pub employee: Entity,
    pub employee_stable_id: u64,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ManeaterDamageTakenEvent {
    pub maneater: Entity,
    pub source: Entity,
    pub applied_damage: I32F32,
}

fn spawn_maneater(
    mut commands: Commands,
    mut events: EventReader<SpawnManeaterEvent>,
    maneaters: Query<(), With<Maneater>>,
) {
    let mut spawned_count = maneaters.iter().count();

    for event in events.read() {
        if spawned_count >= MANEATER_MAX_SPAWNED {
            break;
        }

        commands.spawn(ManeaterBundle::new(*event));
        spawned_count += 1;
    }
}

fn maneater_find_and_imprint_player(
    game_seed: Res<GameSeed>,
    sim_tick: Res<SimTick>,
    mut imprint_events: EventWriter<ManeaterImprintRollEvent>,
    mut state_events: EventWriter<ManeaterStateChangedEvent>,
    mut maneaters: Query<
        (
            Entity,
            &mut ManeaterBabyPhase,
            &mut ManeaterImprint,
            &mut ManeaterBabyTiming,
            &ManeaterForm,
        ),
        With<Maneater>,
    >,
    employees: Query<(Entity, &ManeaterEmployeeSensor)>,
) {
    for (maneater_entity, mut baby_phase, mut imprint, mut timing, form) in maneaters.iter_mut() {
        if *form != ManeaterForm::Baby {
            continue;
        }

        let Some((employee_entity, sensor)) = first_seen_employee(&employees) else {
            continue;
        };

        let first_player_seen = !imprint.has_seen_first_player;
        let chance = if first_player_seen {
            MANEATER_BABY_LIKING_FIRST_PLAYER_CHANCE
        } else {
            MANEATER_BABY_LIKING_OTHER_PLAYER_CHANCE
        };
        let salt = 0x6d61_6e65_6174_0001_u64 ^ sensor.stable_id;
        let mut rng = tick_rng(game_seed.0, sim_tick.0, salt);
        let liked = fixed_chance_from_u32(rng.next_u32(), chance);

        imprint.has_seen_first_player = true;
        imprint.target_stable_id = sensor.stable_id;
        imprint.liked_target = liked || sensor.is_liked_by_baby;
        imprint.no_player_ticks = 0;
        timing.follow_ticks = 0;

        imprint_events.send(ManeaterImprintRollEvent {
            maneater: maneater_entity,
            employee: employee_entity,
            stable_id: sensor.stable_id,
            first_player_seen,
            liked: imprint.liked_target,
        });

        set_maneater_baby_phase(
            maneater_entity,
            &mut baby_phase,
            ManeaterBabyPhase::Following,
            &mut state_events,
        );
    }
}

fn maneater_wander_seek_or_follow(
    sim_hz: Res<SimHz>,
    mut state_events: EventWriter<ManeaterStateChangedEvent>,
    mut crying_events: EventWriter<ManeaterCryingTriggeredEvent>,
    mut maneaters: Query<
        (
            Entity,
            &mut ManeaterBabyPhase,
            &mut ManeaterImprint,
            &mut ManeaterBabyTiming,
            &ManeaterForm,
        ),
        With<Maneater>,
    >,
    employees: Query<&ManeaterEmployeeSensor>,
) {
    let wander_ticks = fixed_seconds_to_ticks(MANEATER_BABY_INITIAL_WANDER_SECONDS, sim_hz.0);
    let seek_ticks = fixed_seconds_to_ticks(MANEATER_BABY_SEEK_PLAYER_SECONDS_MIN, sim_hz.0);
    let follow_ticks = fixed_seconds_to_ticks(MANEATER_BABY_FOLLOW_AVERAGE_SECONDS, sim_hz.0);
    let no_player_ticks = fixed_seconds_to_ticks(MANEATER_NO_PLAYER_SECONDS, sim_hz.0);
    let disliked_hold_ticks = fixed_seconds_to_ticks(MANEATER_DISLIKED_HOLD_SECONDS, sim_hz.0);

    for (maneater_entity, mut baby_phase, mut imprint, mut timing, form) in maneaters.iter_mut() {
        if *form != ManeaterForm::Baby {
            continue;
        }

        timing.phase_ticks = timing.phase_ticks.saturating_add(1);

        match *baby_phase {
            ManeaterBabyPhase::Wandering => {
                if timing.phase_ticks >= wander_ticks {
                    timing.phase_ticks = 0;
                    timing.seek_ticks = seek_ticks;
                    set_maneater_baby_phase(
                        maneater_entity,
                        &mut baby_phase,
                        ManeaterBabyPhase::SeekingPlayer,
                        &mut state_events,
                    );
                }
            }
            ManeaterBabyPhase::SeekingPlayer => {
                if timing.phase_ticks >= timing.seek_ticks {
                    timing.phase_ticks = 0;
                    set_maneater_baby_phase(
                        maneater_entity,
                        &mut baby_phase,
                        ManeaterBabyPhase::Wandering,
                        &mut state_events,
                    );
                }
            }
            ManeaterBabyPhase::Following => {
                let following_target = employees
                    .iter()
                    .any(|sensor| sensor.stable_id == imprint.target_stable_id && sensor.can_be_seen_by_baby);

                if following_target {
                    imprint.no_player_ticks = 0;
                } else {
                    imprint.no_player_ticks = imprint.no_player_ticks.saturating_add(1);
                }

                if imprint.no_player_ticks >= no_player_ticks {
                    set_maneater_baby_phase(
                        maneater_entity,
                        &mut baby_phase,
                        ManeaterBabyPhase::Crying,
                        &mut state_events,
                    );
                    crying_events.send(ManeaterCryingTriggeredEvent {
                        maneater: maneater_entity,
                        reason: ManeaterCryingReason::NoPlayerFound,
                    });
                    continue;
                }

                if timing.phase_ticks >= follow_ticks || (!imprint.liked_target && timing.phase_ticks >= follow_ticks / 2) {
                    timing.phase_ticks = 0;
                    set_maneater_baby_phase(
                        maneater_entity,
                        &mut baby_phase,
                        ManeaterBabyPhase::SeekingPlayer,
                        &mut state_events,
                    );
                }

                let disliked_holder = employees.iter().any(|sensor| {
                    sensor.stable_id == imprint.target_stable_id
                        && sensor.holding_baby
                        && !imprint.liked_target
                });

                if disliked_holder {
                    imprint.disliked_hold_ticks = imprint.disliked_hold_ticks.saturating_add(1);
                } else {
                    imprint.disliked_hold_ticks = 0;
                }

                if imprint.disliked_hold_ticks >= disliked_hold_ticks {
                    set_maneater_baby_phase(
                        maneater_entity,
                        &mut baby_phase,
                        ManeaterBabyPhase::Impatient,
                        &mut state_events,
                    );
                }
            }
            ManeaterBabyPhase::Impatient | ManeaterBabyPhase::Crying => {}
        }
    }
}

fn maneater_trigger_crying_from_sensors(
    mut crying_events: EventWriter<ManeaterCryingTriggeredEvent>,
    mut state_events: EventWriter<ManeaterStateChangedEvent>,
    mut maneaters: Query<
        (
            Entity,
            &mut ManeaterBabyPhase,
            &ManeaterForm,
            &ManeaterStimuli,
        ),
        With<Maneater>,
    >,
) {
    for (maneater_entity, mut baby_phase, form, stimuli) in maneaters.iter_mut() {
        if *form != ManeaterForm::Baby || *baby_phase == ManeaterBabyPhase::Crying {
            continue;
        }

        let reason = if stimuli.loud_non_movement_sound {
            Some(ManeaterCryingReason::LoudNonMovementSound)
        } else if stimuli.melee_hit {
            Some(ManeaterCryingReason::MeleeHit)
        } else if stimuli.dead_body_nearby {
            Some(ManeaterCryingReason::DeadBody)
        } else if stimuli.shaking {
            Some(ManeaterCryingReason::Shaking)
        } else if stimuli.fall {
            Some(ManeaterCryingReason::Fall)
        } else if stimuli.facility_exit {
            Some(ManeaterCryingReason::FacilityExit)
        } else {
            None
        };

        let Some(reason) = reason else {
            continue;
        };

        set_maneater_baby_phase(
            maneater_entity,
            &mut baby_phase,
            ManeaterBabyPhase::Crying,
            &mut state_events,
        );
        crying_events.send(ManeaterCryingTriggeredEvent {
            maneater: maneater_entity,
            reason,
        });
    }
}

fn maneater_rock_crying_baby(
    mut events: EventReader<ManeaterRockedEvent>,
    mut state_events: EventWriter<ManeaterStateChangedEvent>,
    mut maneaters: Query<
        (
            Entity,
            &mut ManeaterBabyPhase,
            &ManeaterForm,
            &ManeaterEnvironment,
        ),
        With<Maneater>,
    >,
) {
    for event in events.read() {
        let Ok((maneater_entity, mut baby_phase, form, environment)) = maneaters.get_mut(event.maneater) else {
            continue;
        };

        if maneater_entity != event.maneater
            || *form != ManeaterForm::Baby
            || *baby_phase != ManeaterBabyPhase::Crying
            || environment.outdoors
            || !event.indoors
        {
            continue;
        }

        set_maneater_baby_phase(
            event.maneater,
            &mut baby_phase,
            ManeaterBabyPhase::Following,
            &mut state_events,
        );
    }
}

fn maneater_advance_crying_transform(
    sim_hz: Res<SimHz>,
    mut transform_events: EventWriter<ManeaterTransformStartedEvent>,
    mut state_events: EventWriter<ManeaterStateChangedEvent>,
    mut maneaters: Query<
        (
            Entity,
            &mut ManeaterForm,
            &ManeaterBabyPhase,
            &mut ManeaterBabyTiming,
            &ManeaterImprint,
            &ManeaterStimuli,
            &ManeaterEnvironment,
            &mut ManeaterAdultTiming,
        ),
        With<Maneater>,
    >,
) {
    let transform_ticks = fixed_seconds_to_ticks(MANEATER_BABY_CRYING_TRANSFORM_SECONDS, sim_hz.0);
    let adult_transform_ticks = fixed_seconds_to_ticks(MANEATER_ADULT_TRANSFORM_SECONDS, sim_hz.0);

    for (
        maneater_entity,
        mut form,
        baby_phase,
        mut baby_timing,
        imprint,
        stimuli,
        environment,
        mut adult_timing,
    ) in maneaters.iter_mut()
    {
        if *form != ManeaterForm::Baby || *baby_phase != ManeaterBabyPhase::Crying || environment.outdoors {
            continue;
        }

        let mut increment = 1;
        if !imprint.liked_target && imprint.disliked_hold_ticks > 0 {
            increment *= MANEATER_CRYING_DISLIKED_HELD_MULTIPLIER;
        }
        if stimuli.running_while_crying {
            increment *= MANEATER_CRYING_RUNNING_MULTIPLIER;
        }

        baby_timing.crying_ticks = baby_timing.crying_ticks.saturating_add(increment);
        if baby_timing.crying_ticks < transform_ticks {
            continue;
        }

        let previous = *form;
        *form = ManeaterForm::Transforming;
        adult_timing.transform_ticks = adult_transform_ticks;
        transform_events.send(ManeaterTransformStartedEvent {
            maneater: maneater_entity,
        });
        state_events.send(ManeaterStateChangedEvent {
            maneater: maneater_entity,
            from_form: previous,
            to_form: ManeaterForm::Transforming,
            from_baby_phase: *baby_phase,
            to_baby_phase: *baby_phase,
            from_adult_phase: ManeaterAdultPhase::Guarding,
            to_adult_phase: ManeaterAdultPhase::Guarding,
        });
    }
}

fn maneater_complete_adult_transform(
    mut completed_events: EventWriter<ManeaterTransformCompletedEvent>,
    mut state_events: EventWriter<ManeaterStateChangedEvent>,
    mut maneaters: Query<
        (
            Entity,
            &mut ManeaterForm,
            &ManeaterBabyPhase,
            &mut ManeaterAdultPhase,
            &mut ManeaterAdultTiming,
            &mut ManeaterAdultTerritory,
            &ManeaterEnvironment,
        ),
        With<Maneater>,
    >,
) {
    for (
        maneater_entity,
        mut form,
        baby_phase,
        mut adult_phase,
        mut adult_timing,
        mut territory,
        environment,
    ) in maneaters.iter_mut()
    {
        if *form != ManeaterForm::Transforming {
            continue;
        }

        if adult_timing.transform_ticks > 0 {
            adult_timing.transform_ticks -= 1;
            continue;
        }

        let previous_form = *form;
        *form = ManeaterForm::Adult;
        let previous_adult_phase = *adult_phase;
        *adult_phase = if environment.outdoors {
            ManeaterAdultPhase::Attacking
        } else {
            territory.has_nest_node = true;
            territory.prefers_cave_nodes = true;
            territory.radius_units = MANEATER_TERRITORY_INITIAL_UNITS;
            ManeaterAdultPhase::Guarding
        };

        completed_events.send(ManeaterTransformCompletedEvent {
            maneater: maneater_entity,
        });
        state_events.send(ManeaterStateChangedEvent {
            maneater: maneater_entity,
            from_form: previous_form,
            to_form: ManeaterForm::Adult,
            from_baby_phase: *baby_phase,
            to_baby_phase: *baby_phase,
            from_adult_phase: previous_adult_phase,
            to_adult_phase: *adult_phase,
        });
    }
}

fn maneater_item_eating(
    game_seed: Res<GameSeed>,
    sim_tick: Res<SimTick>,
    sim_hz: Res<SimHz>,
    mut check_events: EventWriter<ManeaterItemEatCheckEvent>,
    mut eaten_events: EventWriter<ManeaterItemEatenEvent>,
    mut maneaters: Query<
        (
            Entity,
            &ManeaterForm,
            &ManeaterItemSensor,
            &mut ManeaterBabyTiming,
        ),
        With<Maneater>,
    >,
) {
    let check_ticks = fixed_seconds_to_ticks(MANEATER_ITEM_EAT_CHECK_INTERVAL_SECONDS, sim_hz.0);
    let cooldown_ticks = fixed_seconds_to_ticks(MANEATER_ITEM_EAT_COOLDOWN_SECONDS, sim_hz.0);

    for (maneater_entity, form, item_sensor, mut timing) in maneaters.iter_mut() {
        if *form != ManeaterForm::Baby {
            continue;
        }

        if timing.item_cooldown_ticks > 0 {
            timing.item_cooldown_ticks -= 1;
            continue;
        }

        timing.item_check_ticks = timing.item_check_ticks.saturating_add(1);
        if timing.item_check_ticks < check_ticks {
            continue;
        }
        timing.item_check_ticks = 0;

        let Some(item) = item_sensor.ground_item else {
            continue;
        };

        let mut rng = tick_rng(
            game_seed.0,
            sim_tick.0,
            0x6d61_6e65_6174_0002_u64 ^ maneater_entity.index() as u64,
        );
        let ate_item = fixed_chance_from_u32(rng.next_u32(), MANEATER_ITEM_EAT_CHANCE);

        check_events.send(ManeaterItemEatCheckEvent {
            maneater: maneater_entity,
            item,
            ate_item,
        });

        if ate_item {
            timing.item_cooldown_ticks = cooldown_ticks;
            eaten_events.send(ManeaterItemEatenEvent {
                maneater: maneater_entity,
                item,
            });
        }
    }
}

fn maneater_guarding_territory(
    sim_hz: Res<SimHz>,
    mut maneaters: Query<
        (
            &ManeaterForm,
            &ManeaterAdultPhase,
            &ManeaterEnvironment,
            &mut ManeaterAdultTerritory,
        ),
        With<Maneater>,
    >,
) {
    let growth_step = (MANEATER_TERRITORY_MAX_UNITS - MANEATER_TERRITORY_INITIAL_UNITS)
        / (MANEATER_TERRITORY_GROWTH_SECONDS * sim_hz.0);

    for (form, adult_phase, environment, mut territory) in maneaters.iter_mut() {
        if *form != ManeaterForm::Adult || *adult_phase != ManeaterAdultPhase::Guarding {
            continue;
        }

        if environment.outdoors {
            territory.has_nest_node = false;
            territory.radius_units = MANEATER_TERRITORY_INITIAL_UNITS;
            continue;
        }

        if environment.players_inside {
            territory.radius_units = fixed_clamp(
                territory.radius_units + growth_step,
                MANEATER_TERRITORY_INITIAL_UNITS,
                MANEATER_TERRITORY_MAX_UNITS,
            );
        } else {
            territory.radius_units = MANEATER_TERRITORY_INITIAL_UNITS;
        }
    }
}

fn maneater_stalking_trigger_attack(
    mut state_events: EventWriter<ManeaterStateChangedEvent>,
    mut maneaters: Query<
        (
            Entity,
            &ManeaterForm,
            &ManeaterBabyPhase,
            &mut ManeaterAdultPhase,
        ),
        With<Maneater>,
    >,
    employees: Query<&ManeaterEmployeeSensor>,
) {
    let player_within_trigger = employees
        .iter()
        .any(|sensor| sensor.within_stalking_nodes <= MANEATER_STALKING_ATTACK_TRIGGER_NODES);

    if !player_within_trigger {
        return;
    }

    for (maneater_entity, form, baby_phase, mut adult_phase) in maneaters.iter_mut() {
        if *form != ManeaterForm::Adult || *adult_phase != ManeaterAdultPhase::Stalking {
            continue;
        }

        set_maneater_adult_phase(
            maneater_entity,
            *baby_phase,
            &mut adult_phase,
            ManeaterAdultPhase::Attacking,
            &mut state_events,
        );
    }
}

fn maneater_indoor_attack_timing(
    game_seed: Res<GameSeed>,
    sim_tick: Res<SimTick>,
    sim_hz: Res<SimHz>,
    mut state_events: EventWriter<ManeaterStateChangedEvent>,
    mut maneaters: Query<
        (
            Entity,
            &ManeaterForm,
            &ManeaterBabyPhase,
            &mut ManeaterAdultPhase,
            &ManeaterEnvironment,
            &mut ManeaterAdultTiming,
        ),
        With<Maneater>,
    >,
    employees: Query<&ManeaterEmployeeSensor>,
) {
    for (maneater_entity, form, baby_phase, mut adult_phase, environment, mut timing) in
        maneaters.iter_mut()
    {
        if *form != ManeaterForm::Adult
            || *adult_phase != ManeaterAdultPhase::Attacking
            || environment.outdoors
        {
            continue;
        }

        let target_available = employees.iter().any(|sensor| {
            sensor.within_territory
                && sensor.within_stalking_nodes > 0
                && sensor.within_stalking_nodes <= MANEATER_ATTACK_EXIT_DISTANCE.to_num::<u16>()
        });

        if !target_available {
            set_maneater_adult_phase(
                maneater_entity,
                *baby_phase,
                &mut adult_phase,
                ManeaterAdultPhase::Stalking,
                &mut state_events,
            );
            continue;
        }

        if timing.attack_round_ticks > 0 {
            timing.attack_round_ticks -= 1;
        }
        if timing.sprint_ticks > 0 {
            timing.sprint_ticks -= 1;
        } else if timing.pause_ticks > 0 {
            timing.pause_ticks -= 1;
        }

        if timing.attack_round_ticks == 0 {
            let mut rng = tick_rng(
                game_seed.0,
                sim_tick.0,
                0x6d61_6e65_6174_0003_u64 ^ maneater_entity.index() as u64,
            );
            timing.attack_round_ticks = fixed_lerp_ticks(
                rng.next_u32(),
                MANEATER_ADULT_ATTACK_ROUND_SECONDS_MIN,
                MANEATER_ADULT_ATTACK_ROUND_SECONDS_MAX,
                sim_hz.0,
            );
            timing.sprint_ticks = fixed_lerp_ticks(
                rng.next_u32(),
                MANEATER_ADULT_ATTACK_SPRINT_SECONDS_MIN,
                MANEATER_ADULT_ATTACK_SPRINT_SECONDS_MAX,
                sim_hz.0,
            );
            timing.pause_ticks = fixed_lerp_ticks(
                rng.next_u32(),
                MANEATER_ADULT_ATTACK_PAUSE_SECONDS_MIN,
                MANEATER_ADULT_ATTACK_PAUSE_SECONDS_MAX,
                sim_hz.0,
            );
        }
    }
}

fn maneater_outdoor_lunge_attack(
    mut maneaters: Query<(&ManeaterForm, &mut UnitStats, &ManeaterEnvironment), With<Maneater>>,
) {
    for (form, mut stats, environment) in maneaters.iter_mut() {
        if *form != ManeaterForm::Adult || !environment.outdoors {
            continue;
        }

        stats.move_speed *= MANEATER_OUTDOOR_LUNGE_SPEED_MULTIPLIER;
        stats.attack_range = MANEATER_OUTDOOR_ATTACK_RADIUS_UNITS;
    }
}

fn maneater_contact_instant_kill(
    mut damage_events: EventWriter<IncomingDamageEvent>,
    mut contact_events: EventWriter<ManeaterContactKillEvent>,
    maneaters: Query<(Entity, &ManeaterForm, &ManeaterAdultPhase), With<Maneater>>,
    employees: Query<(Entity, &ManeaterEmployeeSensor)>,
) {
    for (maneater_entity, form, adult_phase) in maneaters.iter() {
        if *form != ManeaterForm::Adult || *adult_phase != ManeaterAdultPhase::Attacking {
            continue;
        }

        for (employee_entity, sensor) in employees.iter() {
            if !sensor.touching_maneater {
                continue;
            }

            contact_events.send(ManeaterContactKillEvent {
                maneater: maneater_entity,
                employee: employee_entity,
                employee_stable_id: sensor.stable_id,
            });
            damage_events.send(IncomingDamageEvent {
                target: employee_entity,
                raw_amount: MANEATER_INSTANT_KILL_DAMAGE,
                damage_type: DamageType::Standard,
                source: maneater_entity,
            });
        }
    }
}

fn maneater_damage_rules(
    mut damage_events: EventReader<IncomingDamageEvent>,
    mut transform_events: EventWriter<ManeaterTransformStartedEvent>,
    mut taken_events: EventWriter<ManeaterDamageTakenEvent>,
    mut state_events: EventWriter<ManeaterStateChangedEvent>,
    mut maneaters: Query<
        (
            Entity,
            &mut ManeaterForm,
            &ManeaterBabyPhase,
            &ManeaterAdultPhase,
            &mut ManeaterAdultTiming,
            &mut Health,
        ),
        With<Maneater>,
    >,
    sim_hz: Res<SimHz>,
) {
    for event in damage_events.read() {
        let Ok((maneater_entity, mut form, baby_phase, adult_phase, mut timing, mut health)) =
            maneaters.get_mut(event.target)
        else {
            continue;
        };

        if *form == ManeaterForm::Baby && event.raw_amount >= MANEATER_INSTANT_KILL_DAMAGE {
            let previous = *form;
            *form = ManeaterForm::Transforming;
            timing.transform_ticks = fixed_seconds_to_ticks(MANEATER_ADULT_TRANSFORM_SECONDS, sim_hz.0);
            transform_events.send(ManeaterTransformStartedEvent {
                maneater: maneater_entity,
            });
            state_events.send(ManeaterStateChangedEvent {
                maneater: maneater_entity,
                from_form: previous,
                to_form: ManeaterForm::Transforming,
                from_baby_phase: *baby_phase,
                to_baby_phase: *baby_phase,
                from_adult_phase: *adult_phase,
                to_adult_phase: *adult_phase,
            });
            continue;
        }

        if *form == ManeaterForm::Baby {
            continue;
        }

        let applied_damage = if event.raw_amount >= MANEATER_INSTANT_KILL_DAMAGE {
            MANEATER_INSTANT_KILL_DAMAGE
        } else {
            MANEATER_ADULT_NON_INSTANT_KILL_DAMAGE
        };
        health.current = fixed_clamp(
            health.current - applied_damage,
            I32F32::lit("0"),
            health.max,
        );
        taken_events.send(ManeaterDamageTakenEvent {
            maneater: maneater_entity,
            source: event.source,
            applied_damage,
        });
    }
}

fn maneater_door_attempt_speed(
    mut events: EventReader<ManeaterDoorAttemptEvent>,
    mut resolved_events: EventWriter<ManeaterDoorAttemptResolvedEvent>,
    maneaters: Query<(), With<Maneater>>,
) {
    for event in events.read() {
        if maneaters.get(event.maneater).is_err() {
            continue;
        }

        resolved_events.send(ManeaterDoorAttemptResolvedEvent {
            maneater: event.maneater,
            door: event.door,
            adjusted_open_ticks: fixed_ticks_scaled(
                event.base_open_ticks,
                MANEATER_DOOR_SPEED_MULTIPLIER,
            ),
        });
    }
}

fn maneater_apply_stun_rules(
    mut events: EventReader<ManeaterStunAppliedEvent>,
    mut adjusted_events: EventWriter<ManeaterStunAdjustedEvent>,
    mut ignored_events: EventWriter<ManeaterStunIgnoredEvent>,
    maneaters: Query<&ManeaterForm, With<Maneater>>,
) {
    for event in events.read() {
        let Ok(form) = maneaters.get(event.maneater) else {
            continue;
        };

        if *form == ManeaterForm::Adult && event.stun_kind == ManeaterStunKind::StunGrenade {
            ignored_events.send(ManeaterStunIgnoredEvent {
                maneater: event.maneater,
                stun_kind: event.stun_kind,
            });
            continue;
        }

        adjusted_events.send(ManeaterStunAdjustedEvent {
            maneater: event.maneater,
            base_ticks: event.base_ticks,
            adjusted_ticks: fixed_ticks_scaled(event.base_ticks, MANEATER_STUN_MULTIPLIER),
        });
    }
}

fn maneater_report_zap_gun_difficulty(
    mut events: EventReader<ManeaterZapGunTargetedEvent>,
    mut difficulty_events: EventWriter<ManeaterZapGunDifficultyEvent>,
    maneaters: Query<(), With<Maneater>>,
) {
    for event in events.read() {
        if maneaters.get(event.maneater).is_err() {
            continue;
        }

        difficulty_events.send(ManeaterZapGunDifficultyEvent {
            maneater: event.maneater,
            difficulty_modifier: MANEATER_ZAP_GUN_DIFFICULTY,
        });
    }
}

fn maneater_checksum(
    mut checksum: ResMut<SimChecksumState>,
    maneaters: Query<
        (
            &SimPosition,
            &Health,
            &UnitStats,
            &ManeaterForm,
            &ManeaterBabyPhase,
            &ManeaterAdultPhase,
            &ManeaterImprint,
            &ManeaterBabyTiming,
            &ManeaterAdultTerritory,
            &ManeaterAdultTiming,
            &ManeaterStimuli,
            &ManeaterEnvironment,
        ),
        With<Maneater>,
    >,
) {
    for (
        position,
        health,
        stats,
        form,
        baby_phase,
        adult_phase,
        imprint,
        baby_timing,
        territory,
        adult_timing,
        stimuli,
        environment,
    ) in maneaters.iter()
    {
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(health.current.to_bits() as u64);
        checksum.accumulate(health.max.to_bits() as u64);
        checksum.accumulate(stats.move_speed.to_bits() as u64);
        checksum.accumulate(stats.attack_range.to_bits() as u64);
        checksum.accumulate(stats.attack_damage.to_bits() as u64);
        checksum.accumulate(stats.attack_speed.to_bits() as u64);
        checksum.accumulate(stats.watch_range.to_bits() as u64);
        checksum.accumulate(maneater_form_bits(*form));
        checksum.accumulate(maneater_baby_phase_bits(*baby_phase));
        checksum.accumulate(maneater_adult_phase_bits(*adult_phase));
        checksum.accumulate(imprint.has_seen_first_player as u64);
        checksum.accumulate(imprint.target_stable_id);
        checksum.accumulate(imprint.liked_target as u64);
        checksum.accumulate(imprint.disliked_hold_ticks as u64);
        checksum.accumulate(imprint.no_player_ticks as u64);
        checksum.accumulate(baby_timing.phase_ticks as u64);
        checksum.accumulate(baby_timing.follow_ticks as u64);
        checksum.accumulate(baby_timing.seek_ticks as u64);
        checksum.accumulate(baby_timing.crying_ticks as u64);
        checksum.accumulate(baby_timing.item_check_ticks as u64);
        checksum.accumulate(baby_timing.item_cooldown_ticks as u64);
        checksum.accumulate(territory.has_nest_node as u64);
        checksum.accumulate(territory.nest_node);
        checksum.accumulate(territory.prefers_cave_nodes as u64);
        checksum.accumulate(territory.radius_units.to_bits() as u64);
        checksum.accumulate(adult_timing.transform_ticks as u64);
        checksum.accumulate(adult_timing.attack_round_ticks as u64);
        checksum.accumulate(adult_timing.sprint_ticks as u64);
        checksum.accumulate(adult_timing.pause_ticks as u64);
        checksum.accumulate(stimuli.loud_non_movement_sound as u64);
        checksum.accumulate(stimuli.melee_hit as u64);
        checksum.accumulate(stimuli.dead_body_nearby as u64);
        checksum.accumulate(stimuli.shaking as u64);
        checksum.accumulate(stimuli.fall as u64);
        checksum.accumulate(stimuli.facility_exit as u64);
        checksum.accumulate(stimuli.running_while_crying as u64);
        checksum.accumulate(environment.outdoors as u64);
        checksum.accumulate(environment.players_inside as u64);
    }
}

fn first_seen_employee(
    employees: &Query<(Entity, &ManeaterEmployeeSensor)>,
) -> Option<(Entity, ManeaterEmployeeSensor)> {
    let mut best: Option<(Entity, ManeaterEmployeeSensor)> = None;

    for (entity, sensor) in employees.iter() {
        if !sensor.can_be_seen_by_baby {
            continue;
        }

        if let Some((_best_entity, best_sensor)) = best {
            if sensor.stable_id >= best_sensor.stable_id {
                continue;
            }
        }

        best = Some((entity, *sensor));
    }

    best
}

fn set_maneater_baby_phase(
    maneater: Entity,
    phase: &mut ManeaterBabyPhase,
    next: ManeaterBabyPhase,
    events: &mut EventWriter<ManeaterStateChangedEvent>,
) {
    if *phase == next {
        return;
    }

    let previous = *phase;
    *phase = next;
    events.send(ManeaterStateChangedEvent {
        maneater,
        from_form: ManeaterForm::Baby,
        to_form: ManeaterForm::Baby,
        from_baby_phase: previous,
        to_baby_phase: next,
        from_adult_phase: ManeaterAdultPhase::Guarding,
        to_adult_phase: ManeaterAdultPhase::Guarding,
    });
}

fn set_maneater_adult_phase(
    maneater: Entity,
    baby_phase: ManeaterBabyPhase,
    phase: &mut ManeaterAdultPhase,
    next: ManeaterAdultPhase,
    events: &mut EventWriter<ManeaterStateChangedEvent>,
) {
    if *phase == next {
        return;
    }

    let previous = *phase;
    *phase = next;
    events.send(ManeaterStateChangedEvent {
        maneater,
        from_form: ManeaterForm::Adult,
        to_form: ManeaterForm::Adult,
        from_baby_phase: baby_phase,
        to_baby_phase: baby_phase,
        from_adult_phase: previous,
        to_adult_phase: next,
    });
}

fn fixed_chance_from_u32(draw: u32, chance: I32F32) -> bool {
    let threshold: u32 = (chance * I32F32::from_num(10_000_u32)).to_num();
    draw % 10_000 < threshold
}

fn fixed_lerp_ticks(draw: u32, min_seconds: I32F32, max_seconds: I32F32, sim_hz: I32F32) -> u32 {
    let span_basis = draw % 10_000;
    let span = max_seconds - min_seconds;
    let seconds = min_seconds + (span * I32F32::from_num(span_basis) / I32F32::from_num(10_000_u32));
    fixed_seconds_to_ticks(seconds, sim_hz)
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

fn fixed_clamp(value: I32F32, min: I32F32, max: I32F32) -> I32F32 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

fn maneater_form_bits(form: ManeaterForm) -> u64 {
    match form {
        ManeaterForm::Baby => 0,
        ManeaterForm::Transforming => 1,
        ManeaterForm::Adult => 2,
    }
}

fn maneater_baby_phase_bits(phase: ManeaterBabyPhase) -> u64 {
    match phase {
        ManeaterBabyPhase::Wandering => 0,
        ManeaterBabyPhase::SeekingPlayer => 1,
        ManeaterBabyPhase::Following => 2,
        ManeaterBabyPhase::Impatient => 3,
        ManeaterBabyPhase::Crying => 4,
    }
}

fn maneater_adult_phase_bits(phase: ManeaterAdultPhase) -> u64 {
    match phase {
        ManeaterAdultPhase::Guarding => 0,
        ManeaterAdultPhase::Stalking => 1,
        ManeaterAdultPhase::Attacking => 2,
    }
}