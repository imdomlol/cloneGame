// Sources: vault/map_hazard_pages/map_hazards.md
use bevy::prelude::*;
use fixed::types::I32F32;
use rand_core::RngCore;

use crate::sim::{tick_rng, GameSeed, SimChecksumState, SimPosition, SimTick};

pub const MAP_HAZARDS_ID: &str = "map_hazards";
pub const MAP_HAZARDS_NAME: &str = "Map Hazards";
pub const MAP_HAZARDS_TYPE: &str = "map_hazard_pages";
pub const MAP_HAZARDS_SUBTYPE: &str = "map_hazard";
pub const MAP_HAZARDS_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Map_Hazard";
pub const MAP_HAZARDS_SOURCE_REVISION: u32 = 18340;
pub const MAP_HAZARDS_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const MAP_HAZARDS_CONFIDENCE_BASIS_POINTS: u16 = 93;

pub const MAP_HAZARDS_OVERVIEW: &str = "Map hazards are environmental dangers that interfere with crew movement and scrap collection, and they can be temporarily disabled at the terminal.";

pub const LANDMINE_HAZARD_ID: &str = "landmine";
pub const TURRET_HAZARD_ID: &str = "turret";
pub const SPIKE_TRAP_HAZARD_ID: &str = "spike_trap";

pub const LANDMINE_COUNT_MIN: u16 = 0;
pub const LANDMINE_COUNT_MAX: u16 = 35;
pub const TURRET_COUNT_MIN: u16 = 0;
pub const TURRET_COUNT_MAX: u16 = 35;
pub const SPIKE_TRAP_COUNT_MIN: u16 = 0;
pub const SPIKE_TRAP_COUNT_MAX: u16 = 18;

pub const SPIKE_TRAP_TIMER_INTERVAL_MIN_SECONDS: I32F32 = I32F32::lit("0.8");
pub const SPIKE_TRAP_TIMER_INTERVAL_MAX_SECONDS: I32F32 = I32F32::lit("26.2");
pub const MAP_HAZARDS_TERMINAL_DISABLE_DURATION_TICKS: u32 = 100;

pub const MAP_HAZARDS_DEPENDS_ON: [&str; 8] = [
    "lethal_company",
    "employee",
    "scrap",
    "terminal",
    "landmine",
    "turret",
    "spike_trap",
    "hoarding_bug",
];

pub const MAP_HAZARD_RULES: [MapHazardRule; 3] = [
    MapHazardRule::landmine(
        LANDMINE_HAZARD_ID,
        &["employee", "dead_body", "item"],
        "contact_or_moving_off",
        "explosion",
    ),
    MapHazardRule::basic(
        TURRET_HAZARD_ID,
        "player_detected_in_line_of_sight",
        "rapid_bullets",
    ),
    MapHazardRule::spike_trap(SPIKE_TRAP_HAZARD_ID, &["timer", "detection"]),
];

pub const MAP_HAZARD_MODIFIERS: [MapHazardModifier; 3] = [
    MapHazardModifier::landmine(
        LANDMINE_HAZARD_ID,
        LANDMINE_COUNT_MIN,
        LANDMINE_COUNT_MAX,
        "intermittent_beeping",
    ),
    MapHazardModifier::turret(
        TURRET_HAZARD_ID,
        TURRET_COUNT_MIN,
        TURRET_COUNT_MAX,
        &["orange_light", "audio_signal"],
    ),
    MapHazardModifier::spike_trap(
        SPIKE_TRAP_HAZARD_ID,
        SPIKE_TRAP_COUNT_MIN,
        SPIKE_TRAP_COUNT_MAX,
        SPIKE_TRAP_TIMER_INTERVAL_MIN_SECONDS,
        SPIKE_TRAP_TIMER_INTERVAL_MAX_SECONDS,
    ),
];

pub const MAP_HAZARD_STRATEGY: [MapHazardStrategy; 5] = [
    MapHazardStrategy::new(LANDMINE_HAZARD_ID, "throw_flashbang_or_easter_egg_at_it"),
    MapHazardStrategy::new(LANDMINE_HAZARD_ID, "place_a_player_body_on_it_and_run_away"),
    MapHazardStrategy::new(
        LANDMINE_HAZARD_ID,
        "let_a_hoarding_bug_move_a_body_over_it",
    ),
    MapHazardStrategy::new(LANDMINE_HAZARD_ID, "step_on_it_and_get_teleported_off"),
    MapHazardStrategy::new(
        LANDMINE_HAZARD_ID,
        "die_near_it_and_allow_the_body_to_ragdoll_onto_it",
    ),
];

pub const MAP_HAZARD_NOTES: [MapHazardNote; 3] = [
    MapHazardNote::new(
        TURRET_HAZARD_ID,
        "Backtracking out of the room is often enough to leave its line of sight.",
    ),
    MapHazardNote::new(
        TURRET_HAZARD_ID,
        "A melee hit can make it spin while firing for a short period of time.",
    ),
    MapHazardNote::new(
        SPIKE_TRAP_HAZARD_ID,
        "The slam sound can warn nearby players when the trap is using timer activation.",
    ),
];

pub const MAP_HAZARDS_BEHAVIORAL_MECHANICS: [MapHazardBehaviorRule; 14] = [
    MapHazardBehaviorRule::new(
        "a landmine is contacted by an employee, a dead body, or an item",
        "it explodes",
    ),
    MapHazardBehaviorRule::new(
        "a landmine is triggered by an item being dropped onto it or by a player or body moving off of it",
        "it detonates",
    ),
    MapHazardBehaviorRule::new(
        "a landmine is active",
        "it can emit intermittent beeping as a warning cue",
    ),
    MapHazardBehaviorRule::new(
        "a moon has landmine hazards",
        "the count can range from 0 to 35",
    ),
    MapHazardBehaviorRule::new("a turret sees a player", "it rapidly fires bullets"),
    MapHazardBehaviorRule::new(
        "a turret detects a player",
        "it emits an orange light, gives an audio cue, and turns toward the player's last known position",
    ),
    MapHazardBehaviorRule::new(
        "a player moves out of a turret's line of sight",
        "the turret stops tracking",
    ),
    MapHazardBehaviorRule::new(
        "a melee item hits a turret",
        "it spins while firing for a short period of time",
    ),
    MapHazardBehaviorRule::new(
        "a moon has turret hazards",
        "the count can range from 0 to 35",
    ),
    MapHazardBehaviorRule::new(
        "a spike_trap is in timer mode",
        "it slams down on a randomly chosen interval from 0.8 to 26.2 seconds",
    ),
    MapHazardBehaviorRule::new(
        "a spike_trap is in detection mode and detects a player beneath it",
        "it slams down",
    ),
    MapHazardBehaviorRule::new(
        "a moon has spike_trap hazards",
        "the count can range from 0 to 18",
    ),
    MapHazardBehaviorRule::new(
        "a spike_trap slams down",
        "the impact sound can alert nearby players",
    ),
    MapHazardBehaviorRule::new(
        "the terminal is used",
        "map hazards can be temporarily deactivated",
    ),
];

const SPIKE_TRAP_TIMER_SALT: u64 = 0x7370_696b_655f_7472;
const MAP_HAZARD_SPAWN_COUNT_SALT: u64 = 0x6861_7a61_7264_5f73;

pub struct MapHazardsPlugin;

impl Plugin for MapHazardsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnMapHazardEvent>()
            .add_event::<MapHazardSpawnCountResolvedEvent>()
            .add_event::<MapHazardContactEvent>()
            .add_event::<MapHazardMovedOffEvent>()
            .add_event::<MapHazardDetectedPlayerEvent>()
            .add_event::<MapHazardLineOfSightLostEvent>()
            .add_event::<MapHazardMeleeHitEvent>()
            .add_event::<MapHazardTerminalDisableRequestedEvent>()
            .add_event::<MapHazardTemporarilyDisabledEvent>()
            .add_event::<MapHazardExplosionEvent>()
            .add_event::<MapHazardBulletBurstEvent>()
            .add_event::<MapHazardSpikeSlamEvent>()
            .add_event::<MapHazardWarningCueEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_map_hazard,
                    resolve_map_hazard_spawn_counts,
                    map_hazard_terminal_disable,
                    map_hazard_disable_timer,
                    map_hazard_contact_trigger,
                    map_hazard_moved_off_trigger,
                    map_hazard_turret_detect_player,
                    map_hazard_turret_line_of_sight_lost,
                    map_hazard_turret_melee_hit,
                    map_hazard_spike_timer,
                    map_hazard_spike_detection,
                    map_hazard_warning_cues,
                    map_hazards_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MapHazardBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

impl MapHazardBehaviorRule {
    pub const fn new(condition: &'static str, outcome: &'static str) -> Self {
        Self { condition, outcome }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MapHazardStrategy {
    pub hazard: &'static str,
    pub action: &'static str,
}

impl MapHazardStrategy {
    pub const fn new(hazard: &'static str, action: &'static str) -> Self {
        Self { hazard, action }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MapHazardNote {
    pub hazard: &'static str,
    pub note: &'static str,
}

impl MapHazardNote {
    pub const fn new(hazard: &'static str, note: &'static str) -> Self {
        Self { hazard, note }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MapHazardRule {
    pub hazard: &'static str,
    pub trigger_entities: &'static [&'static str],
    pub trigger_condition: &'static str,
    pub effect: &'static str,
    pub activation_modes: &'static [&'static str],
}

impl MapHazardRule {
    pub const fn landmine(
        hazard: &'static str,
        trigger_entities: &'static [&'static str],
        trigger_condition: &'static str,
        effect: &'static str,
    ) -> Self {
        Self {
            hazard,
            trigger_entities,
            trigger_condition,
            effect,
            activation_modes: &[],
        }
    }

    pub const fn basic(
        hazard: &'static str,
        trigger_condition: &'static str,
        effect: &'static str,
    ) -> Self {
        Self {
            hazard,
            trigger_entities: &[],
            trigger_condition,
            effect,
            activation_modes: &[],
        }
    }

    pub const fn spike_trap(hazard: &'static str, activation_modes: &'static [&'static str]) -> Self {
        Self {
            hazard,
            trigger_entities: &[],
            trigger_condition: "",
            effect: "",
            activation_modes,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MapHazardModifier {
    pub hazard: &'static str,
    pub count_min: u16,
    pub count_max: u16,
    pub cue: &'static str,
    pub cues: &'static [&'static str],
    pub timer_interval_min_seconds: I32F32,
    pub timer_interval_max_seconds: I32F32,
}

impl MapHazardModifier {
    pub const fn landmine(
        hazard: &'static str,
        count_min: u16,
        count_max: u16,
        cue: &'static str,
    ) -> Self {
        Self {
            hazard,
            count_min,
            count_max,
            cue,
            cues: &[],
            timer_interval_min_seconds: I32F32::lit("0"),
            timer_interval_max_seconds: I32F32::lit("0"),
        }
    }

    pub const fn turret(
        hazard: &'static str,
        count_min: u16,
        count_max: u16,
        cues: &'static [&'static str],
    ) -> Self {
        Self {
            hazard,
            count_min,
            count_max,
            cue: "",
            cues,
            timer_interval_min_seconds: I32F32::lit("0"),
            timer_interval_max_seconds: I32F32::lit("0"),
        }
    }

    pub const fn spike_trap(
        hazard: &'static str,
        count_min: u16,
        count_max: u16,
        timer_interval_min_seconds: I32F32,
        timer_interval_max_seconds: I32F32,
    ) -> Self {
        Self {
            hazard,
            count_min,
            count_max,
            cue: "",
            cues: &[],
            timer_interval_min_seconds,
            timer_interval_max_seconds,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MapHazard {
    pub stable_id: u64,
    pub kind: MapHazardKind,
    pub temporarily_disabled_ticks_remaining: u32,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MapHazardTriggerTarget {
    pub stable_id: u64,
    pub is_employee: bool,
    pub is_dead_body: bool,
    pub is_item: bool,
    pub is_player: bool,
    pub beneath_spike_trap: bool,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LandmineHazardState {
    pub active: bool,
    pub occupied_by_stable_id: u64,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct TurretHazardState {
    pub tracking_stable_id: u64,
    pub last_known_position: SimPosition,
    pub spinning_fire_ticks_remaining: u32,
}

impl Default for TurretHazardState {
    fn default() -> Self {
        Self {
            tracking_stable_id: 0,
            last_known_position: SimPosition {
                x: I32F32::lit("0"),
                y: I32F32::lit("0"),
            },
            spinning_fire_ticks_remaining: 0,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SpikeTrapHazardState {
    pub mode: SpikeTrapActivationMode,
    pub slam_ticks_remaining: u32,
    pub next_timer_slam_tick: u64,
}

#[derive(Bundle, Clone, Copy, Debug)]
pub struct MapHazardBundle {
    pub hazard: MapHazard,
    pub position: SimPosition,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MapHazardKind {
    #[default]
    Landmine,
    Turret,
    SpikeTrap,
}

impl MapHazardKind {
    pub const fn id(self) -> &'static str {
        match self {
            MapHazardKind::Landmine => LANDMINE_HAZARD_ID,
            MapHazardKind::Turret => TURRET_HAZARD_ID,
            MapHazardKind::SpikeTrap => SPIKE_TRAP_HAZARD_ID,
        }
    }

    pub const fn count_min(self) -> u16 {
        match self {
            MapHazardKind::Landmine => LANDMINE_COUNT_MIN,
            MapHazardKind::Turret => TURRET_COUNT_MIN,
            MapHazardKind::SpikeTrap => SPIKE_TRAP_COUNT_MIN,
        }
    }

    pub const fn count_max(self) -> u16 {
        match self {
            MapHazardKind::Landmine => LANDMINE_COUNT_MAX,
            MapHazardKind::Turret => TURRET_COUNT_MAX,
            MapHazardKind::SpikeTrap => SPIKE_TRAP_COUNT_MAX,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SpikeTrapActivationMode {
    #[default]
    Timer,
    Detection,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapHazardContactKind {
    Contact,
    MovingOff,
    DroppedOnto,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnMapHazardEvent {
    pub stable_id: u64,
    pub kind: MapHazardKind,
    pub position: SimPosition,
    pub spike_trap_mode: SpikeTrapActivationMode,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapHazardSpawnCountResolvedEvent {
    pub hazard: MapHazardKind,
    pub count_min: u16,
    pub count_max: u16,
    pub resolved_count: u16,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapHazardContactEvent {
    pub hazard: Entity,
    pub target: Entity,
    pub target_stable_id: u64,
    pub target_kind: MapHazardContactTargetKind,
    pub contact_kind: MapHazardContactKind,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapHazardMovedOffEvent {
    pub hazard: Entity,
    pub target_stable_id: u64,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapHazardDetectedPlayerEvent {
    pub hazard: Entity,
    pub player: Entity,
    pub player_stable_id: u64,
    pub player_position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapHazardLineOfSightLostEvent {
    pub hazard: Entity,
    pub player_stable_id: u64,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapHazardMeleeHitEvent {
    pub hazard: Entity,
    pub item_id: &'static str,
    pub spinning_fire_ticks: u32,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapHazardTerminalDisableRequestedEvent {
    pub hazard: Entity,
    pub duration_ticks: u32,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapHazardTemporarilyDisabledEvent {
    pub hazard: Entity,
    pub hazard_stable_id: u64,
    pub duration_ticks: u32,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapHazardExplosionEvent {
    pub hazard: Entity,
    pub hazard_stable_id: u64,
    pub trigger: MapHazardContactKind,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapHazardBulletBurstEvent {
    pub hazard: Entity,
    pub target_stable_id: u64,
    pub last_known_position: SimPosition,
    pub spinning: bool,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapHazardSpikeSlamEvent {
    pub hazard: Entity,
    pub hazard_stable_id: u64,
    pub mode: SpikeTrapActivationMode,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapHazardWarningCueEvent {
    pub hazard: Entity,
    pub hazard_stable_id: u64,
    pub cue: MapHazardCue,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapHazardContactTargetKind {
    Employee,
    DeadBody,
    Item,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapHazardCue {
    IntermittentBeeping,
    OrangeLight,
    AudioSignal,
    ImpactSound,
}

fn spawn_map_hazard(mut commands: Commands, mut events: EventReader<SpawnMapHazardEvent>) {
    for event in events.read() {
        let mut entity = commands.spawn(MapHazardBundle {
            hazard: MapHazard {
                stable_id: event.stable_id,
                kind: event.kind,
                temporarily_disabled_ticks_remaining: 0,
            },
            position: event.position,
        });

        match event.kind {
            MapHazardKind::Landmine => {
                entity.insert(LandmineHazardState {
                    active: true,
                    occupied_by_stable_id: 0,
                });
            }
            MapHazardKind::Turret => {
                entity.insert(TurretHazardState {
                    tracking_stable_id: 0,
                    last_known_position: event.position,
                    spinning_fire_ticks_remaining: 0,
                });
            }
            MapHazardKind::SpikeTrap => {
                entity.insert(SpikeTrapHazardState {
                    mode: event.spike_trap_mode,
                    slam_ticks_remaining: 0,
                    next_timer_slam_tick: 0,
                });
            }
        }
    }
}

fn resolve_map_hazard_spawn_counts(
    seed: Res<GameSeed>,
    tick: Res<SimTick>,
    mut events: EventWriter<MapHazardSpawnCountResolvedEvent>,
) {
    for hazard in [
        MapHazardKind::Landmine,
        MapHazardKind::Turret,
        MapHazardKind::SpikeTrap,
    ] {
        let count_min = hazard.count_min();
        let count_max = hazard.count_max();
        let range = count_max - count_min + 1;
        let salt = MAP_HAZARD_SPAWN_COUNT_SALT ^ hazard_salt(hazard);
        let resolved_count = count_min + (tick_rng(seed.0, tick.0, salt).next_u32() % range as u32) as u16;

        events.send(MapHazardSpawnCountResolvedEvent {
            hazard,
            count_min,
            count_max,
            resolved_count,
        });
    }
}

fn map_hazard_terminal_disable(
    mut requests: EventReader<MapHazardTerminalDisableRequestedEvent>,
    mut hazards: Query<&mut MapHazard>,
    mut disabled: EventWriter<MapHazardTemporarilyDisabledEvent>,
) {
    for request in requests.read() {
        if let Ok(mut hazard) = hazards.get_mut(request.hazard) {
            hazard.temporarily_disabled_ticks_remaining = request.duration_ticks;
            disabled.send(MapHazardTemporarilyDisabledEvent {
                hazard: request.hazard,
                hazard_stable_id: hazard.stable_id,
                duration_ticks: request.duration_ticks,
            });
        }
    }
}

fn map_hazard_disable_timer(mut hazards: Query<&mut MapHazard>) {
    for mut hazard in &mut hazards {
        if hazard.temporarily_disabled_ticks_remaining > 0 {
            hazard.temporarily_disabled_ticks_remaining -= 1;
        }
    }
}

fn map_hazard_contact_trigger(
    mut contacts: EventReader<MapHazardContactEvent>,
    hazards: Query<&MapHazard>,
    mut landmines: Query<&mut LandmineHazardState>,
    mut explosions: EventWriter<MapHazardExplosionEvent>,
) {
    for contact in contacts.read() {
        if let Ok(hazard) = hazards.get(contact.hazard) {
            if hazard.kind != MapHazardKind::Landmine
                || hazard.temporarily_disabled_ticks_remaining > 0
                || !is_landmine_trigger_target(contact.target_kind)
            {
                continue;
            }

            if let Ok(mut landmine) = landmines.get_mut(contact.hazard) {
                if !landmine.active {
                    continue;
                }

                match contact.contact_kind {
                    MapHazardContactKind::Contact => {
                        landmine.occupied_by_stable_id = contact.target_stable_id;
                    }
                    MapHazardContactKind::MovingOff | MapHazardContactKind::DroppedOnto => {
                        landmine.active = false;
                        explosions.send(MapHazardExplosionEvent {
                            hazard: contact.hazard,
                            hazard_stable_id: hazard.stable_id,
                            trigger: contact.contact_kind,
                        });
                    }
                }
            }
        }
    }
}

fn map_hazard_moved_off_trigger(
    mut moved_off: EventReader<MapHazardMovedOffEvent>,
    hazards: Query<&MapHazard>,
    mut landmines: Query<&mut LandmineHazardState>,
    mut explosions: EventWriter<MapHazardExplosionEvent>,
) {
    for event in moved_off.read() {
        if let Ok(hazard) = hazards.get(event.hazard) {
            if hazard.kind != MapHazardKind::Landmine || hazard.temporarily_disabled_ticks_remaining > 0 {
                continue;
            }

            if let Ok(mut landmine) = landmines.get_mut(event.hazard) {
                if landmine.active && landmine.occupied_by_stable_id == event.target_stable_id {
                    landmine.active = false;
                    explosions.send(MapHazardExplosionEvent {
                        hazard: event.hazard,
                        hazard_stable_id: hazard.stable_id,
                        trigger: MapHazardContactKind::MovingOff,
                    });
                }
            }
        }
    }
}

fn map_hazard_turret_detect_player(
    mut detections: EventReader<MapHazardDetectedPlayerEvent>,
    hazards: Query<&MapHazard>,
    mut turrets: Query<&mut TurretHazardState>,
    mut bullet_bursts: EventWriter<MapHazardBulletBurstEvent>,
    mut cues: EventWriter<MapHazardWarningCueEvent>,
) {
    for detection in detections.read() {
        if let Ok(hazard) = hazards.get(detection.hazard) {
            if hazard.kind != MapHazardKind::Turret || hazard.temporarily_disabled_ticks_remaining > 0 {
                continue;
            }

            if let Ok(mut turret) = turrets.get_mut(detection.hazard) {
                turret.tracking_stable_id = detection.player_stable_id;
                turret.last_known_position = detection.player_position;

                cues.send(MapHazardWarningCueEvent {
                    hazard: detection.hazard,
                    hazard_stable_id: hazard.stable_id,
                    cue: MapHazardCue::OrangeLight,
                });
                cues.send(MapHazardWarningCueEvent {
                    hazard: detection.hazard,
                    hazard_stable_id: hazard.stable_id,
                    cue: MapHazardCue::AudioSignal,
                });
                bullet_bursts.send(MapHazardBulletBurstEvent {
                    hazard: detection.hazard,
                    target_stable_id: detection.player_stable_id,
                    last_known_position: detection.player_position,
                    spinning: false,
                });
            }
        }
    }
}

fn map_hazard_turret_line_of_sight_lost(
    mut events: EventReader<MapHazardLineOfSightLostEvent>,
    mut turrets: Query<&mut TurretHazardState>,
) {
    for event in events.read() {
        if let Ok(mut turret) = turrets.get_mut(event.hazard) {
            if turret.tracking_stable_id == event.player_stable_id {
                turret.tracking_stable_id = 0;
            }
        }
    }
}

fn map_hazard_turret_melee_hit(
    mut events: EventReader<MapHazardMeleeHitEvent>,
    hazards: Query<&MapHazard>,
    mut turrets: Query<&mut TurretHazardState>,
    mut bullet_bursts: EventWriter<MapHazardBulletBurstEvent>,
) {
    for event in events.read() {
        if let Ok(hazard) = hazards.get(event.hazard) {
            if hazard.kind != MapHazardKind::Turret || hazard.temporarily_disabled_ticks_remaining > 0 {
                continue;
            }

            if let Ok(mut turret) = turrets.get_mut(event.hazard) {
                turret.spinning_fire_ticks_remaining = event.spinning_fire_ticks;
                bullet_bursts.send(MapHazardBulletBurstEvent {
                    hazard: event.hazard,
                    target_stable_id: turret.tracking_stable_id,
                    last_known_position: turret.last_known_position,
                    spinning: true,
                });
            }
        }
    }

    for mut turret in &mut turrets {
        if turret.spinning_fire_ticks_remaining > 0 {
            turret.spinning_fire_ticks_remaining -= 1;
        }
    }
}

fn map_hazard_spike_timer(
    seed: Res<GameSeed>,
    tick: Res<SimTick>,
    hazards: Query<(Entity, &MapHazard)>,
    mut spike_traps: Query<&mut SpikeTrapHazardState>,
    mut slams: EventWriter<MapHazardSpikeSlamEvent>,
) {
    let mut sorted_hazards: Vec<(u64, Entity, MapHazard)> = hazards
        .iter()
        .filter(|(_, hazard)| hazard.kind == MapHazardKind::SpikeTrap)
        .map(|(entity, hazard)| (hazard.stable_id, entity, *hazard))
        .collect();
    sorted_hazards.sort_by_key(|(stable_id, _, _)| *stable_id);

    for (_, entity, hazard) in sorted_hazards {
        if hazard.temporarily_disabled_ticks_remaining > 0 {
            continue;
        }

        if let Ok(mut spike_trap) = spike_traps.get_mut(entity) {
            if spike_trap.mode != SpikeTrapActivationMode::Timer {
                continue;
            }

            if spike_trap.next_timer_slam_tick == 0 {
                spike_trap.next_timer_slam_tick = tick.0 + choose_spike_trap_interval_ticks(seed.0, tick.0, hazard.stable_id);
            }

            if tick.0 >= spike_trap.next_timer_slam_tick {
                spike_trap.slam_ticks_remaining = 1;
                spike_trap.next_timer_slam_tick =
                    tick.0 + choose_spike_trap_interval_ticks(seed.0, tick.0, hazard.stable_id);
                slams.send(MapHazardSpikeSlamEvent {
                    hazard: entity,
                    hazard_stable_id: hazard.stable_id,
                    mode: SpikeTrapActivationMode::Timer,
                });
            } else if spike_trap.slam_ticks_remaining > 0 {
                spike_trap.slam_ticks_remaining -= 1;
            }
        }
    }
}

fn map_hazard_spike_detection(
    hazards: Query<(Entity, &MapHazard)>,
    targets: Query<&MapHazardTriggerTarget>,
    mut spike_traps: Query<&mut SpikeTrapHazardState>,
    mut slams: EventWriter<MapHazardSpikeSlamEvent>,
) {
    let mut sorted_targets: Vec<MapHazardTriggerTarget> = targets.iter().copied().collect();
    sorted_targets.sort_by_key(|target| target.stable_id);

    let mut sorted_hazards: Vec<(u64, Entity, MapHazard)> = hazards
        .iter()
        .filter(|(_, hazard)| hazard.kind == MapHazardKind::SpikeTrap)
        .map(|(entity, hazard)| (hazard.stable_id, entity, *hazard))
        .collect();
    sorted_hazards.sort_by_key(|(stable_id, _, _)| *stable_id);

    for (_, entity, hazard) in sorted_hazards {
        if hazard.temporarily_disabled_ticks_remaining > 0 {
            continue;
        }

        if let Ok(mut spike_trap) = spike_traps.get_mut(entity) {
            if spike_trap.mode != SpikeTrapActivationMode::Detection {
                continue;
            }

            let player_beneath = sorted_targets
                .iter()
                .any(|target| target.is_player && target.beneath_spike_trap);

            if player_beneath {
                spike_trap.slam_ticks_remaining = 1;
                slams.send(MapHazardSpikeSlamEvent {
                    hazard: entity,
                    hazard_stable_id: hazard.stable_id,
                    mode: SpikeTrapActivationMode::Detection,
                });
            } else if spike_trap.slam_ticks_remaining > 0 {
                spike_trap.slam_ticks_remaining -= 1;
            }
        }
    }
}

fn map_hazard_warning_cues(
    hazards: Query<(Entity, &MapHazard)>,
    spike_traps: Query<&SpikeTrapHazardState>,
    mut slams: EventReader<MapHazardSpikeSlamEvent>,
    mut cues: EventWriter<MapHazardWarningCueEvent>,
) {
    let mut sorted_hazards: Vec<(u64, Entity, MapHazard)> =
        hazards.iter().map(|(entity, hazard)| (hazard.stable_id, entity, *hazard)).collect();
    sorted_hazards.sort_by_key(|(stable_id, _, _)| *stable_id);

    for (_, entity, hazard) in sorted_hazards {
        if hazard.temporarily_disabled_ticks_remaining > 0 {
            continue;
        }

        if hazard.kind == MapHazardKind::Landmine {
            cues.send(MapHazardWarningCueEvent {
                hazard: entity,
                hazard_stable_id: hazard.stable_id,
                cue: MapHazardCue::IntermittentBeeping,
            });
        }
    }

    for slam in slams.read() {
        if spike_traps.get(slam.hazard).is_ok() {
            cues.send(MapHazardWarningCueEvent {
                hazard: slam.hazard,
                hazard_stable_id: slam.hazard_stable_id,
                cue: MapHazardCue::ImpactSound,
            });
        }
    }
}

fn map_hazards_checksum(
    mut checksum: ResMut<SimChecksumState>,
    tick: Res<SimTick>,
    hazards: Query<(&MapHazard, &SimPosition)>,
    landmines: Query<&LandmineHazardState>,
    turrets: Query<&TurretHazardState>,
    spike_traps: Query<&SpikeTrapHazardState>,
) {
    checksum.accumulate(tick.0);
    checksum.accumulate(MAP_HAZARDS_SOURCE_REVISION as u64);
    checksum.accumulate(MAP_HAZARDS_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(LANDMINE_COUNT_MIN as u64);
    checksum.accumulate(LANDMINE_COUNT_MAX as u64);
    checksum.accumulate(TURRET_COUNT_MIN as u64);
    checksum.accumulate(TURRET_COUNT_MAX as u64);
    checksum.accumulate(SPIKE_TRAP_COUNT_MIN as u64);
    checksum.accumulate(SPIKE_TRAP_COUNT_MAX as u64);
    checksum.accumulate(SPIKE_TRAP_TIMER_INTERVAL_MIN_SECONDS.to_bits() as u64);
    checksum.accumulate(SPIKE_TRAP_TIMER_INTERVAL_MAX_SECONDS.to_bits() as u64);
    checksum.accumulate(MAP_HAZARDS_TERMINAL_DISABLE_DURATION_TICKS as u64);

    for dependency in MAP_HAZARDS_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x1000, dependency);
    }

    for rule in MAP_HAZARD_RULES {
        accumulate_str(&mut checksum, 0x2000, rule.hazard);
        for trigger_entity in rule.trigger_entities {
            accumulate_str(&mut checksum, 0x2001, trigger_entity);
        }
        accumulate_str(&mut checksum, 0x2002, rule.trigger_condition);
        accumulate_str(&mut checksum, 0x2003, rule.effect);
        for activation_mode in rule.activation_modes {
            accumulate_str(&mut checksum, 0x2004, activation_mode);
        }
    }

    for modifier in MAP_HAZARD_MODIFIERS {
        accumulate_str(&mut checksum, 0x3000, modifier.hazard);
        checksum.accumulate(modifier.count_min as u64);
        checksum.accumulate(modifier.count_max as u64);
        accumulate_str(&mut checksum, 0x3001, modifier.cue);
        for cue in modifier.cues {
            accumulate_str(&mut checksum, 0x3002, cue);
        }
        checksum.accumulate(modifier.timer_interval_min_seconds.to_bits() as u64);
        checksum.accumulate(modifier.timer_interval_max_seconds.to_bits() as u64);
    }

    for strategy in MAP_HAZARD_STRATEGY {
        accumulate_str(&mut checksum, 0x4000, strategy.hazard);
        accumulate_str(&mut checksum, 0x4001, strategy.action);
    }

    for note in MAP_HAZARD_NOTES {
        accumulate_str(&mut checksum, 0x5000, note.hazard);
        accumulate_str(&mut checksum, 0x5001, note.note);
    }

    for behavior in MAP_HAZARDS_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x6000, behavior.condition);
        accumulate_str(&mut checksum, 0x6001, behavior.outcome);
    }

    for (hazard, position) in &hazards {
        checksum.accumulate(hazard.stable_id);
        checksum.accumulate(hazard_salt(hazard.kind));
        checksum.accumulate(hazard.temporarily_disabled_ticks_remaining as u64);
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
    }

    for landmine in &landmines {
        checksum.accumulate(landmine.active as u64);
        checksum.accumulate(landmine.occupied_by_stable_id);
    }

    for turret in &turrets {
        checksum.accumulate(turret.tracking_stable_id);
        checksum.accumulate(turret.last_known_position.x.to_bits() as u64);
        checksum.accumulate(turret.last_known_position.y.to_bits() as u64);
        checksum.accumulate(turret.spinning_fire_ticks_remaining as u64);
    }

    for spike_trap in &spike_traps {
        checksum.accumulate(match spike_trap.mode {
            SpikeTrapActivationMode::Timer => 1,
            SpikeTrapActivationMode::Detection => 2,
        });
        checksum.accumulate(spike_trap.slam_ticks_remaining as u64);
        checksum.accumulate(spike_trap.next_timer_slam_tick);
    }
}

fn choose_spike_trap_interval_ticks(game_seed: u64, tick: u64, stable_id: u64) -> u64 {
    let min_tenths = 8_u32;
    let max_tenths = 262_u32;
    let span = max_tenths - min_tenths + 1;
    let tenths = min_tenths + (tick_rng(game_seed, tick, SPIKE_TRAP_TIMER_SALT ^ stable_id).next_u32() % span);
    tenths as u64 * 2
}

fn is_landmine_trigger_target(kind: MapHazardContactTargetKind) -> bool {
    matches!(
        kind,
        MapHazardContactTargetKind::Employee
            | MapHazardContactTargetKind::DeadBody
            | MapHazardContactTargetKind::Item
    )
}

fn hazard_salt(kind: MapHazardKind) -> u64 {
    match kind {
        MapHazardKind::Landmine => 0x6c61_6e64_6d69_6e65,
        MapHazardKind::Turret => 0x7475_7272_6574,
        MapHazardKind::SpikeTrap => 0x7370_696b_655f_7472,
    }
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt ^ value.len() as u64);

    for (index, byte) in value.bytes().enumerate() {
        checksum.accumulate(salt ^ ((index as u64) << 8) ^ byte as u64);
    }
}