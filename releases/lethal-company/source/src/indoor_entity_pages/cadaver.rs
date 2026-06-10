// Sources: vault/indoor_entity_pages/cadaver.md
use bevy::prelude::*;
use fixed::types::I32F32;
use rand_core::RngCore;

use crate::sim::{
    tick_rng, DamageType, GameSeed, Health, IncomingDamageEvent, NoiseEmittedEvent,
    SimChecksumState, SimHz, SimPosition, SimTick, UnitStats,
};

pub const CADAVER_ID: &str = "cadaver";
pub const CADAVER_NAME: &str = "Cadaver";
pub const CADAVER_TYPE: &str = "indoor_entity_pages";
pub const CADAVER_SUBTYPE: &str = "entity";
pub const CADAVER_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Cadaver";
pub const CADAVER_SOURCE_REVISION: u32 = 21465;
pub const CADAVER_EXTRACTED_AT: &str = "2026-06-07T00:00:00Z";
pub const CADAVER_CONFIDENCE_BASIS_POINTS: u16 = 92;

pub const CADAVER_HP: &str = "Infinite";
pub const CADAVER_POWER_LEVEL: I32F32 = I32F32::lit("2");
pub const CADAVER_MAX_SPAWNED: usize = 1;
pub const CADAVER_ATTACK_DAMAGE: &str = "Instant Kill (Burst Phase)";
pub const CADAVER_SPAWN_DELAY_SECONDS: u32 = 15;
pub const CADAVER_DWELLS: &str = "Inside";
pub const CADAVER_DANGER: &str = "100%";
pub const CADAVER_CAN_SEE_THROUGH_FOG: bool = false;
pub const CADAVER_INTERNAL_NAME: &str = "Cadaver Growth";

pub const CADAVER_INFINITE_HEALTH: I32F32 = I32F32::lit("1000000");
pub const CADAVER_BURST_DAMAGE: I32F32 = I32F32::lit("1000000");
pub const CADAVER_WATCH_RANGE: I32F32 = I32F32::lit("10");
pub const CADAVER_SPORE_RANGE: I32F32 = I32F32::lit("10");
pub const CADAVER_COUGH_INFECTION_RANGE: I32F32 = I32F32::lit("6");
pub const CADAVER_NEARBY_ACCELERATION_RANGE: I32F32 = I32F32::lit("15");
pub const CADAVER_CHAIN_BURST_RANGE: I32F32 = I32F32::lit("14");
pub const CADAVER_SOLO_EXPOSURE_SECONDS: I32F32 = I32F32::lit("7");
pub const CADAVER_MULTIPLAYER_EXPOSURE_SECONDS: I32F32 = I32F32::lit("4");
pub const CADAVER_SYMPTOMLESS_MAX: I32F32 = I32F32::lit("0.08");
pub const CADAVER_MILD_MAX: I32F32 = I32F32::lit("0.55");
pub const CADAVER_BLOOM_DEATH_THRESHOLD: I32F32 = I32F32::lit("0.35");
pub const CADAVER_SPORE_TALK_THRESHOLD: I32F32 = I32F32::lit("0.60");
pub const CADAVER_FEVER_THRESHOLD: I32F32 = I32F32::lit("0.80");
pub const CADAVER_NEARBY_ACCELERATION_THRESHOLD: I32F32 = I32F32::lit("0.925");
pub const CADAVER_CRITICAL_CHAIN_THRESHOLD: I32F32 = I32F32::lit("0.85");
pub const CADAVER_COMPLETE_INFECTION: I32F32 = I32F32::lit("1");
pub const CADAVER_BURST_WARNING_THRESHOLD: I32F32 = I32F32::lit("0.90");
pub const CADAVER_WEED_KILLER_NORMAL_SECONDS: I32F32 = I32F32::lit("0.33");
pub const CADAVER_WEED_KILLER_CRITICAL_SECONDS: I32F32 = I32F32::lit("0.53");
pub const CADAVER_WEED_KILLER_INFECTION_REDUCTION: I32F32 = I32F32::lit("0.1");
pub const CADAVER_WEED_KILLER_PLANT_TIME_REDUCTION: I32F32 = I32F32::lit("0.25");
pub const CADAVER_WEED_KILLER_DAMAGE: I32F32 = I32F32::lit("8");
pub const CADAVER_SHOWER_SLOW_MULTIPLIER: I32F32 = I32F32::lit("0.85");
pub const CADAVER_FULL_HEALTH_CHANCE_MULTIPLIER: I32F32 = I32F32::lit("0.75");
pub const CADAVER_LOW_HEALTH_CHANCE_MULTIPLIER: I32F32 = I32F32::lit("1.2");
pub const CADAVER_CRITICAL_DENSE_CHANCE_MULTIPLIER: I32F32 = I32F32::lit("1.5");
pub const CADAVER_FULL_HEALTH_SPEED_MULTIPLIER: I32F32 = I32F32::lit("0.85");
pub const CADAVER_LOW_HEALTH_SPEED_MULTIPLIER: I32F32 = I32F32::lit("1.15");
pub const CADAVER_SOLO_SPEED_MULTIPLIER: I32F32 = I32F32::lit("0.45");
pub const CADAVER_SUNLIGHT_SPEED_MULTIPLIER: I32F32 = I32F32::lit("1.15");
pub const CADAVER_LOBBY_FOUR_ONE_INFECTED_SPEED: I32F32 = I32F32::lit("0.9625");
pub const CADAVER_LOBBY_FOUR_FOUR_INFECTED_SPEED: I32F32 = I32F32::lit("0.85");
pub const CADAVER_SEVERE_CASE_PERCENT: u32 = 76;
pub const CADAVER_NON_SEVERE_TALK_SPORE_PERCENT: u32 = 1;
pub const CADAVER_SECONDS_BEFORE_SUNLIGHT_SPEEDUP: u64 = 33120;
pub const CADAVER_MULTIPLAYER_FINAL_BURST_TICKS_AT_25HZ: u32 = 43;
pub const CADAVER_SOLO_FINAL_BURST_TICKS_AT_25HZ: u32 = 450;
pub const CADAVER_NEAR_EMPLOYEE_BURST_TICKS_AT_25HZ: u32 = 75;
pub const CADAVER_FAR_EMPLOYEE_BURST_TICKS_AT_25HZ: u32 = 6825;
pub const CADAVER_RNG_SALT_INFECTION: u64 = 0xCA_DA_00_01;
pub const CADAVER_RNG_SALT_SEVERE: u64 = 0xCA_DA_00_02;
pub const CADAVER_RNG_SALT_TALK: u64 = 0xCA_DA_00_03;

pub const CADAVER_DEPENDS_ON: [&str; 4] = [
    "lethal_company",
    "weed_killer",
    "cadaver_bloom",
    "masked",
];

pub const CADAVER_FRONTMATTER_BEHAVIOR: [&str; 8] = [
    "IF an employee remains in the spore cloud for 7 seconds solo or 4 seconds multiplayer, THEN infection can begin.",
    "IF infection is acquired, THEN the timer pauses outside the source and resumes on re-entry for the rest of the run.",
    "IF infection is at 0% to 8%, THEN there are no visible symptoms and a shower can cleanse it.",
    "IF infection is at 8% to 55%, THEN a shower only slows progression by 15% while the employee remains inside it.",
    "IF infection reaches 35% during the mild stage and the employee dies, THEN a cadaver_bloom can burst from the body unless no body is left.",
    "IF infection reaches 100%, THEN a burst meter begins and can convert the employee into a burst event.",
    "IF another employee uses weed_killer for at least 0.33 seconds, or 0.53 seconds at critical health, THEN infection is reduced by 0.1 and 25% of total plant time per tick while 8 damage is dealt.",
    "IF the target is already in the burst phase, THEN weed_killer triggers an immediate burst instead of curing them.",
];

pub const CADAVER_SPAWN_CHANCES: [CadaverSpawnChance; 4] = [
    CadaverSpawnChance {
        moon: "Adamance",
        chance: I32F32::lit("37.66"),
    },
    CadaverSpawnChance {
        moon: "Dine",
        chance: I32F32::lit("11.54"),
    },
    CadaverSpawnChance {
        moon: "Artifice",
        chance: I32F32::lit("2.7"),
    },
    CadaverSpawnChance {
        moon: "Rend",
        chance: I32F32::lit("1.41"),
    },
];

pub const CADAVER_BEHAVIORAL_MECHANICS: [CadaverBehaviorRule; 34] = [
    CadaverBehaviorRule {
        condition: "an employee stays in a Cadaver spore cloud for 7 seconds solo or 4 seconds multiplayer",
        outcome: "infection can begin",
    },
    CadaverBehaviorRule {
        condition: "an employee is near an infected coughing employee",
        outcome: "infection can begin without direct contact with the spore cloud",
    },
    CadaverBehaviorRule {
        condition: "infection starts",
        outcome: "the exposure timer pauses when the employee leaves the source and resumes when they re-enter it",
    },
    CadaverBehaviorRule {
        condition: "the employee is in solo",
        outcome: "remaining exposure time is shown on the HUD as Filter quality: X%",
    },
    CadaverBehaviorRule {
        condition: "Cadaver density within 10 units increases or the employee is closer to the nearest plant",
        outcome: "infection chance increases",
    },
    CadaverBehaviorRule {
        condition: "more employees are already infected",
        outcome: "infection chance decreases slightly",
    },
    CadaverBehaviorRule {
        condition: "the employee is at full health",
        outcome: "infection chance is multiplied by 0.75",
    },
    CadaverBehaviorRule {
        condition: "the employee is at or below 60 health",
        outcome: "infection chance is multiplied by 1.2",
    },
    CadaverBehaviorRule {
        condition: "the employee is at critical health in dense plants",
        outcome: "infection chance is multiplied by 1.5",
    },
    CadaverBehaviorRule {
        condition: "infection is at 0% to 8%",
        outcome: "there are no visible symptoms and a shower can cleanse it",
    },
    CadaverBehaviorRule {
        condition: "infection is at 8% to 55%",
        outcome: "a shower only slows progression by 15% while the employee remains inside it",
    },
    CadaverBehaviorRule {
        condition: "infection reaches 35% and the employee dies",
        outcome: "a cadaver_bloom bursts from the body unless no body remains",
    },
    CadaverBehaviorRule {
        condition: "infection is at 55% to 80%",
        outcome: "spore emission while talking can occur after 60% exposure",
    },
    CadaverBehaviorRule {
        condition: "the infection roll is one of the 76% severe cases",
        outcome: "talking after 60% exposure always emits spores",
    },
    CadaverBehaviorRule {
        condition: "the infection roll is one of the 24% non-severe cases",
        outcome: "talking after 60% exposure emits spores with a 1% chance per talk",
    },
    CadaverBehaviorRule {
        condition: "an infected employee coughs",
        outcome: "nearby employees within 6 units see a health-risk warning on the HUD",
    },
    CadaverBehaviorRule {
        condition: "infection is above 80%",
        outcome: "fever warnings begin in solo, HUD temperature shows about 101F to 112F, visual poison builds up, and vine-in-head audio can play",
    },
    CadaverBehaviorRule {
        condition: "infection is above 92.5%",
        outcome: "nearby players within 15 units accelerate progression to 150%",
    },
    CadaverBehaviorRule {
        condition: "infection reaches 100%",
        outcome: "a burst meter begins",
    },
    CadaverBehaviorRule {
        condition: "the burst meter is between 0% and 90%",
        outcome: "fill speed depends on exposure and distance to nearby employees",
    },
    CadaverBehaviorRule {
        condition: "the burst meter is 15 units or less from employees",
        outcome: "it fills in under 3 seconds",
    },
    CadaverBehaviorRule {
        condition: "the burst meter is 30 units or more from employees",
        outcome: "it fills in about 4 minutes 33 seconds",
    },
    CadaverBehaviorRule {
        condition: "the burst meter reaches 90%",
        outcome: "it takes about 1.7 seconds to reach 100% in multiplayer or about 18 seconds in solo",
    },
    CadaverBehaviorRule {
        condition: "the burst meter is at 90% or higher",
        outcome: "movement is hindered, vision reddens, and a ringing sound plays",
    },
    CadaverBehaviorRule {
        condition: "the infected employee is on the ship during takeoff",
        outcome: "the burst meter stops at 90% and they can leave without dying",
    },
    CadaverBehaviorRule {
        condition: "another employee sprays the infected employee with weed_killer for at least 0.33 seconds or 0.53 seconds at critical health",
        outcome: "0.1 infection and 25% of total plant-time are removed per tick while 8 damage is dealt",
    },
    CadaverBehaviorRule {
        condition: "the sprayed employee has already reached the burst phase",
        outcome: "weed_killer triggers an immediate burst instead of curing them",
    },
    CadaverBehaviorRule {
        condition: "the lobby has 4 employees and infection count rises from 1 to 4",
        outcome: "infection speed scales from 96.25% to 85%",
    },
    CadaverBehaviorRule {
        condition: "only one employee is alive",
        outcome: "burst buildup slows down",
    },
    CadaverBehaviorRule {
        condition: "the infected employee is in solo",
        outcome: "final infection speed is multiplied by 0.45",
    },
    CadaverBehaviorRule {
        condition: "the infected employee is at full health",
        outcome: "infection progress speed is multiplied by 0.85",
    },
    CadaverBehaviorRule {
        condition: "the infected employee is at or below 40 health",
        outcome: "infection progress speed is multiplied by 1.15",
    },
    CadaverBehaviorRule {
        condition: "the infected employee is outside before about 5:12 pm in direct sunlight",
        outcome: "infection speed increases to 115%",
    },
    CadaverBehaviorRule {
        condition: "a burst occurs and another player is within 14 units with infection above 85%",
        outcome: "that player chain-bursts simultaneously",
    },
];

pub struct CadaverPlugin;

impl Plugin for CadaverPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnCadaverEvent>()
            .add_event::<CadaverInfectionRollEvent>()
            .add_event::<CadaverInfectionStartedEvent>()
            .add_event::<CadaverInfectionStageChangedEvent>()
            .add_event::<CadaverFilterQualityHudEvent>()
            .add_event::<CadaverCoughWarningEvent>()
            .add_event::<CadaverFeverWarningEvent>()
            .add_event::<CadaverBloomBurstRequestEvent>()
            .add_event::<CadaverBurstStartedEvent>()
            .add_event::<CadaverBurstWarningEvent>()
            .add_event::<CadaverBurstEvent>()
            .add_event::<CadaverChainBurstEvent>()
            .add_event::<CadaverWeedKillerSprayEvent>()
            .add_event::<CadaverWeedKillerCureTickEvent>()
            .add_event::<CadaverWeedKillerImmediateBurstEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_cadaver,
                    cadaver_track_spore_exposure,
                    cadaver_roll_infection_start,
                    cadaver_progress_infection,
                    cadaver_update_stages,
                    cadaver_emit_hud_warnings,
                    cadaver_emit_talking_spores,
                    cadaver_request_bloom_on_death,
                    cadaver_start_burst_phase,
                    cadaver_progress_burst_meter,
                    cadaver_apply_weed_killer,
                    cadaver_execute_burst,
                    cadaver_chain_burst,
                    cadaver_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CadaverBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CadaverSpawnChance {
    pub moon: &'static str,
    pub chance: I32F32,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Cadaver;

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CadaverSporeCloud {
    pub range: I32F32,
    pub density: I32F32,
}

impl Default for CadaverSporeCloud {
    fn default() -> Self {
        Self {
            range: CADAVER_SPORE_RANGE,
            density: I32F32::lit("1"),
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CadaverEmployeeSensor {
    pub stable_id: u64,
    pub in_spore_cloud: bool,
    pub near_coughing_infected: bool,
    pub talking: bool,
    pub coughing: bool,
    pub in_shower: bool,
    pub outside_in_direct_sunlight: bool,
    pub on_ship_during_takeoff: bool,
    pub body_left_after_death: bool,
    pub is_dead: bool,
    pub is_solo: bool,
    pub alive_employee_count: u8,
    pub infected_employee_count: u8,
    pub cadaver_density_within_10: u8,
    pub nearest_plant_distance_units: I32F32,
    pub nearest_employee_distance_units: I32F32,
    pub current_health: I32F32,
    pub max_health: I32F32,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CadaverInfection {
    pub infected: bool,
    pub severe_case: bool,
    pub exposure_ticks: u32,
    pub exposure_required_ticks: u32,
    pub plant_time_ticks: u32,
    pub progress: I32F32,
    pub stage: CadaverInfectionStage,
    pub burst_meter: I32F32,
    pub burst_phase: bool,
    pub burst_complete: bool,
    pub bloom_requested: bool,
}

impl Default for CadaverInfection {
    fn default() -> Self {
        Self {
            infected: false,
            severe_case: false,
            exposure_ticks: 0,
            exposure_required_ticks: 175,
            plant_time_ticks: 0,
            progress: I32F32::lit("0"),
            stage: CadaverInfectionStage::None,
            burst_meter: I32F32::lit("0"),
            burst_phase: false,
            burst_complete: false,
            bloom_requested: false,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CadaverInfectionStage {
    #[default]
    None,
    Symptomless,
    Mild,
    SporeTalk,
    Fever,
    Burst,
}

#[derive(Bundle)]
pub struct CadaverBundle {
    pub name: Name,
    pub cadaver: Cadaver,
    pub position: SimPosition,
    pub health: Health,
    pub stats: UnitStats,
    pub spore_cloud: CadaverSporeCloud,
}

impl CadaverBundle {
    pub fn new(event: SpawnCadaverEvent) -> Self {
        Self {
            name: Name::new(CADAVER_NAME),
            cadaver: Cadaver,
            position: event.position,
            health: Health::full(CADAVER_INFINITE_HEALTH),
            stats: UnitStats {
                move_speed: I32F32::lit("0"),
                attack_range: CADAVER_SPORE_RANGE,
                attack_damage: CADAVER_BURST_DAMAGE,
                attack_speed: I32F32::lit("0"),
                watch_range: CADAVER_WATCH_RANGE,
            },
            spore_cloud: CadaverSporeCloud {
                range: CADAVER_SPORE_RANGE,
                density: event.density,
            },
        }
    }
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpawnCadaverEvent {
    pub position: SimPosition,
    pub density: I32F32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CadaverInfectionRollEvent {
    pub employee: Entity,
    pub employee_stable_id: u64,
    pub chance_basis_points: u32,
    pub roll_basis_points: u32,
    pub infected: bool,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CadaverInfectionStartedEvent {
    pub employee: Entity,
    pub employee_stable_id: u64,
    pub severe_case: bool,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CadaverInfectionStageChangedEvent {
    pub employee: Entity,
    pub employee_stable_id: u64,
    pub from: CadaverInfectionStage,
    pub to: CadaverInfectionStage,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CadaverFilterQualityHudEvent {
    pub employee: Entity,
    pub employee_stable_id: u64,
    pub remaining_percent: u8,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CadaverCoughWarningEvent {
    pub source_employee: Entity,
    pub warned_employee_stable_id: u64,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CadaverFeverWarningEvent {
    pub employee: Entity,
    pub employee_stable_id: u64,
    pub temperature_fahrenheit_tenths: u16,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CadaverBloomBurstRequestEvent {
    pub employee: Entity,
    pub employee_stable_id: u64,
    pub bloom_id: &'static str,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CadaverBurstStartedEvent {
    pub employee: Entity,
    pub employee_stable_id: u64,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CadaverBurstWarningEvent {
    pub employee: Entity,
    pub employee_stable_id: u64,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CadaverBurstEvent {
    pub employee: Entity,
    pub employee_stable_id: u64,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CadaverChainBurstEvent {
    pub source_employee: Entity,
    pub chained_employee: Entity,
    pub chained_stable_id: u64,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CadaverWeedKillerSprayEvent {
    pub sprayer: Entity,
    pub target: Entity,
    pub spray_ticks: u32,
    pub target_critical_health: bool,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CadaverWeedKillerCureTickEvent {
    pub sprayer: Entity,
    pub target: Entity,
    pub infection_removed: I32F32,
    pub plant_time_removed_ticks: u32,
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CadaverWeedKillerImmediateBurstEvent {
    pub sprayer: Entity,
    pub target: Entity,
}

fn spawn_cadaver(
    mut commands: Commands,
    mut events: EventReader<SpawnCadaverEvent>,
    cadavers: Query<(), With<Cadaver>>,
) {
    let mut spawned_count = cadavers.iter().count();

    for event in events.read() {
        if spawned_count >= CADAVER_MAX_SPAWNED {
            break;
        }

        commands.spawn(CadaverBundle::new(*event));
        spawned_count += 1;
    }
}

fn cadaver_track_spore_exposure(
    sim_hz: Res<SimHz>,
    mut employees: Query<(&CadaverEmployeeSensor, &mut CadaverInfection)>,
) {
    for (sensor, mut infection) in employees.iter_mut() {
        infection.exposure_required_ticks = exposure_required_ticks(sensor.is_solo, sim_hz.0);

        if sensor.in_spore_cloud || sensor.near_coughing_infected {
            infection.exposure_ticks = infection.exposure_ticks.saturating_add(1);
            infection.plant_time_ticks = infection.plant_time_ticks.saturating_add(1);
        }
    }
}

fn cadaver_roll_infection_start(
    sim_tick: Res<SimTick>,
    game_seed: Res<GameSeed>,
    mut roll_events: EventWriter<CadaverInfectionRollEvent>,
    mut start_events: EventWriter<CadaverInfectionStartedEvent>,
    mut employees: Query<(Entity, &CadaverEmployeeSensor, &mut CadaverInfection)>,
) {
    for (employee, sensor, mut infection) in employees.iter_mut() {
        if infection.infected || infection.exposure_ticks < infection.exposure_required_ticks {
            continue;
        }

        let chance_basis_points = infection_chance_basis_points(sensor);
        let mut rng = tick_rng(
            game_seed.0,
            sim_tick.0,
            CADAVER_RNG_SALT_INFECTION ^ sensor.stable_id,
        );
        let roll_basis_points = rng.next_u32() % 10000;
        let infected = roll_basis_points < chance_basis_points;

        roll_events.send(CadaverInfectionRollEvent {
            employee,
            employee_stable_id: sensor.stable_id,
            chance_basis_points,
            roll_basis_points,
            infected,
        });

        if !infected {
            continue;
        }

        let mut severe_rng = tick_rng(
            game_seed.0,
            sim_tick.0,
            CADAVER_RNG_SALT_SEVERE ^ sensor.stable_id,
        );

        infection.infected = true;
        infection.severe_case = (severe_rng.next_u32() % 100) < CADAVER_SEVERE_CASE_PERCENT;
        infection.stage = CadaverInfectionStage::Symptomless;

        start_events.send(CadaverInfectionStartedEvent {
            employee,
            employee_stable_id: sensor.stable_id,
            severe_case: infection.severe_case,
        });
    }
}

fn cadaver_progress_infection(
    sim_tick: Res<SimTick>,
    sim_hz: Res<SimHz>,
    mut employees: Query<(&CadaverEmployeeSensor, &mut CadaverInfection)>,
) {
    for (sensor, mut infection) in employees.iter_mut() {
        if !infection.infected || infection.burst_phase || infection.burst_complete {
            continue;
        }

        let mut speed = infection_speed_multiplier(sensor);

        if sensor.in_shower && infection.progress >= CADAVER_SYMPTOMLESS_MAX {
            speed *= CADAVER_SHOWER_SLOW_MULTIPLIER;
        }

        if sensor.in_shower && infection.progress < CADAVER_SYMPTOMLESS_MAX {
            infection.infected = false;
            infection.progress = I32F32::lit("0");
            infection.stage = CadaverInfectionStage::None;
            infection.exposure_ticks = 0;
            continue;
        }

        if sensor.outside_in_direct_sunlight && sim_tick.0 < CADAVER_SECONDS_BEFORE_SUNLIGHT_SPEEDUP {
            speed *= CADAVER_SUNLIGHT_SPEED_MULTIPLIER;
        }

        let step = speed / (sim_hz.0 * I32F32::lit("120"));
        infection.progress = fixed_clamp(
            infection.progress + step,
            I32F32::lit("0"),
            CADAVER_COMPLETE_INFECTION,
        );
    }
}

fn cadaver_update_stages(
    mut stage_events: EventWriter<CadaverInfectionStageChangedEvent>,
    mut employees: Query<(Entity, &CadaverEmployeeSensor, &mut CadaverInfection)>,
) {
    for (employee, sensor, mut infection) in employees.iter_mut() {
        let next = infection_stage_for_progress(infection.progress, infection.burst_phase);

        if infection.stage == next {
            continue;
        }

        let previous = infection.stage;
        infection.stage = next;
        stage_events.send(CadaverInfectionStageChangedEvent {
            employee,
            employee_stable_id: sensor.stable_id,
            from: previous,
            to: next,
        });
    }
}

fn cadaver_emit_hud_warnings(
    mut filter_events: EventWriter<CadaverFilterQualityHudEvent>,
    mut cough_events: EventWriter<CadaverCoughWarningEvent>,
    mut fever_events: EventWriter<CadaverFeverWarningEvent>,
    employees: Query<(Entity, &CadaverEmployeeSensor, &CadaverInfection)>,
) {
    for (employee, sensor, infection) in employees.iter() {
        if sensor.is_solo && !infection.infected {
            let remaining_percent = exposure_remaining_percent(infection.exposure_ticks, infection.exposure_required_ticks);
            filter_events.send(CadaverFilterQualityHudEvent {
                employee,
                employee_stable_id: sensor.stable_id,
                remaining_percent,
            });
        }

        if infection.infected && sensor.coughing {
            for (_other_employee, other_sensor, _other_infection) in employees.iter() {
                if other_sensor.stable_id == sensor.stable_id {
                    continue;
                }

                if other_sensor.nearest_employee_distance_units <= CADAVER_COUGH_INFECTION_RANGE {
                    cough_events.send(CadaverCoughWarningEvent {
                        source_employee: employee,
                        warned_employee_stable_id: other_sensor.stable_id,
                    });
                }
            }
        }

        if sensor.is_solo && infection.progress > CADAVER_FEVER_THRESHOLD {
            fever_events.send(CadaverFeverWarningEvent {
                employee,
                employee_stable_id: sensor.stable_id,
                temperature_fahrenheit_tenths: fever_temperature_tenths(infection.progress),
            });
        }
    }
}

fn cadaver_emit_talking_spores(
    sim_tick: Res<SimTick>,
    game_seed: Res<GameSeed>,
    mut noise_events: EventWriter<NoiseEmittedEvent>,
    employees: Query<(Entity, &CadaverEmployeeSensor, &CadaverInfection, &SimPosition)>,
) {
    for (employee, sensor, infection, position) in employees.iter() {
        if !infection.infected || !sensor.talking || infection.progress < CADAVER_SPORE_TALK_THRESHOLD {
            continue;
        }

        let emits = if infection.severe_case {
            true
        } else {
            let mut rng = tick_rng(
                game_seed.0,
                sim_tick.0,
                CADAVER_RNG_SALT_TALK ^ sensor.stable_id,
            );
            (rng.next_u32() % 100) < CADAVER_NON_SEVERE_TALK_SPORE_PERCENT
        };

        if !emits {
            continue;
        }

        noise_events.send(NoiseEmittedEvent {
            source: employee,
            position: *position,
            amount: CADAVER_COUGH_INFECTION_RANGE,
        });
    }
}

fn cadaver_request_bloom_on_death(
    mut bloom_events: EventWriter<CadaverBloomBurstRequestEvent>,
    mut employees: Query<(Entity, &CadaverEmployeeSensor, &mut CadaverInfection)>,
) {
    for (employee, sensor, mut infection) in employees.iter_mut() {
        if infection.bloom_requested {
            continue;
        }

        if !sensor.is_dead || !sensor.body_left_after_death {
            continue;
        }

        if infection.progress < CADAVER_BLOOM_DEATH_THRESHOLD {
            continue;
        }

        infection.bloom_requested = true;
        bloom_events.send(CadaverBloomBurstRequestEvent {
            employee,
            employee_stable_id: sensor.stable_id,
            bloom_id: "cadaver_bloom",
        });
    }
}

fn cadaver_start_burst_phase(
    mut burst_started_events: EventWriter<CadaverBurstStartedEvent>,
    mut employees: Query<(Entity, &CadaverEmployeeSensor, &mut CadaverInfection)>,
) {
    for (employee, sensor, mut infection) in employees.iter_mut() {
        if !infection.infected || infection.burst_phase {
            continue;
        }

        if infection.progress < CADAVER_COMPLETE_INFECTION {
            continue;
        }

        infection.burst_phase = true;
        infection.stage = CadaverInfectionStage::Burst;
        burst_started_events.send(CadaverBurstStartedEvent {
            employee,
            employee_stable_id: sensor.stable_id,
        });
    }
}

fn cadaver_progress_burst_meter(
    sim_hz: Res<SimHz>,
    mut warning_events: EventWriter<CadaverBurstWarningEvent>,
    mut employees: Query<(Entity, &CadaverEmployeeSensor, &mut CadaverInfection)>,
) {
    for (employee, sensor, mut infection) in employees.iter_mut() {
        if !infection.burst_phase || infection.burst_complete {
            continue;
        }

        if sensor.on_ship_during_takeoff {
            infection.burst_meter = fixed_clamp(
                infection.burst_meter,
                I32F32::lit("0"),
                CADAVER_BURST_WARNING_THRESHOLD,
            );
            continue;
        }

        let target_ticks = burst_fill_ticks(sensor, sim_hz.0);
        let step = CADAVER_COMPLETE_INFECTION / I32F32::from_num(target_ticks);
        infection.burst_meter = fixed_clamp(
            infection.burst_meter + step,
            I32F32::lit("0"),
            CADAVER_COMPLETE_INFECTION,
        );

        if infection.burst_meter >= CADAVER_BURST_WARNING_THRESHOLD {
            warning_events.send(CadaverBurstWarningEvent {
                employee,
                employee_stable_id: sensor.stable_id,
            });
        }
    }
}

fn cadaver_apply_weed_killer(
    sim_hz: Res<SimHz>,
    mut spray_events: EventReader<CadaverWeedKillerSprayEvent>,
    mut cure_events: EventWriter<CadaverWeedKillerCureTickEvent>,
    mut immediate_burst_events: EventWriter<CadaverWeedKillerImmediateBurstEvent>,
    mut damage_events: EventWriter<IncomingDamageEvent>,
    mut employees: Query<&mut CadaverInfection>,
) {
    for event in spray_events.read() {
        let Ok(mut infection) = employees.get_mut(event.target) else {
            continue;
        };

        let required_ticks = if event.target_critical_health {
            fixed_seconds_to_ticks(CADAVER_WEED_KILLER_CRITICAL_SECONDS, sim_hz.0)
        } else {
            fixed_seconds_to_ticks(CADAVER_WEED_KILLER_NORMAL_SECONDS, sim_hz.0)
        };

        if event.spray_ticks < required_ticks {
            continue;
        }

        damage_events.send(IncomingDamageEvent {
            target: event.target,
            raw_amount: CADAVER_WEED_KILLER_DAMAGE,
            damage_type: DamageType::Standard,
            source: event.sprayer,
        });

        if infection.burst_phase {
            infection.burst_meter = CADAVER_COMPLETE_INFECTION;
            immediate_burst_events.send(CadaverWeedKillerImmediateBurstEvent {
                sprayer: event.sprayer,
                target: event.target,
            });
            continue;
        }

        let plant_time_removed_ticks = fixed_ticks_scaled(
            infection.plant_time_ticks,
            CADAVER_WEED_KILLER_PLANT_TIME_REDUCTION,
        );
        infection.progress = fixed_clamp(
            infection.progress - CADAVER_WEED_KILLER_INFECTION_REDUCTION,
            I32F32::lit("0"),
            CADAVER_COMPLETE_INFECTION,
        );
        infection.plant_time_ticks = infection.plant_time_ticks.saturating_sub(plant_time_removed_ticks);

        cure_events.send(CadaverWeedKillerCureTickEvent {
            sprayer: event.sprayer,
            target: event.target,
            infection_removed: CADAVER_WEED_KILLER_INFECTION_REDUCTION,
            plant_time_removed_ticks,
        });
    }
}

fn cadaver_execute_burst(
    mut burst_events: EventWriter<CadaverBurstEvent>,
    mut damage_events: EventWriter<IncomingDamageEvent>,
    mut employees: Query<(Entity, &CadaverEmployeeSensor, &mut CadaverInfection)>,
) {
    for (employee, sensor, mut infection) in employees.iter_mut() {
        if !infection.burst_phase || infection.burst_complete {
            continue;
        }

        if infection.burst_meter < CADAVER_COMPLETE_INFECTION {
            continue;
        }

        infection.burst_complete = true;
        burst_events.send(CadaverBurstEvent {
            employee,
            employee_stable_id: sensor.stable_id,
        });
        damage_events.send(IncomingDamageEvent {
            target: employee,
            raw_amount: CADAVER_BURST_DAMAGE,
            damage_type: DamageType::Standard,
            source: employee,
        });
    }
}

fn cadaver_chain_burst(
    mut burst_events: EventReader<CadaverBurstEvent>,
    mut chain_events: EventWriter<CadaverChainBurstEvent>,
    mut employees: Query<(Entity, &CadaverEmployeeSensor, &mut CadaverInfection)>,
) {
    for burst in burst_events.read() {
        for (employee, sensor, mut infection) in employees.iter_mut() {
            if employee == burst.employee || infection.burst_complete {
                continue;
            }

            if infection.progress <= CADAVER_CRITICAL_CHAIN_THRESHOLD {
                continue;
            }

            if sensor.nearest_employee_distance_units > CADAVER_CHAIN_BURST_RANGE {
                continue;
            }

            infection.burst_phase = true;
            infection.burst_meter = CADAVER_COMPLETE_INFECTION;

            chain_events.send(CadaverChainBurstEvent {
                source_employee: burst.employee,
                chained_employee: employee,
                chained_stable_id: sensor.stable_id,
            });
        }
    }
}

fn cadaver_checksum(
    mut checksum: ResMut<SimChecksumState>,
    cadavers: Query<(&SimPosition, &Health, &UnitStats, &CadaverSporeCloud), With<Cadaver>>,
    infections: Query<(&CadaverEmployeeSensor, &CadaverInfection)>,
) {
    for (position, health, stats, spore_cloud) in cadavers.iter() {
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(health.current.to_bits() as u64);
        checksum.accumulate(health.max.to_bits() as u64);
        checksum.accumulate(stats.move_speed.to_bits() as u64);
        checksum.accumulate(stats.attack_range.to_bits() as u64);
        checksum.accumulate(stats.attack_damage.to_bits() as u64);
        checksum.accumulate(stats.attack_speed.to_bits() as u64);
        checksum.accumulate(stats.watch_range.to_bits() as u64);
        checksum.accumulate(spore_cloud.range.to_bits() as u64);
        checksum.accumulate(spore_cloud.density.to_bits() as u64);
    }

    for (sensor, infection) in infections.iter() {
        checksum.accumulate(sensor.stable_id);
        checksum.accumulate(infection.infected as u64);
        checksum.accumulate(infection.severe_case as u64);
        checksum.accumulate(infection.exposure_ticks as u64);
        checksum.accumulate(infection.exposure_required_ticks as u64);
        checksum.accumulate(infection.plant_time_ticks as u64);
        checksum.accumulate(infection.progress.to_bits() as u64);
        checksum.accumulate(cadaver_stage_bits(infection.stage));
        checksum.accumulate(infection.burst_meter.to_bits() as u64);
        checksum.accumulate(infection.burst_phase as u64);
        checksum.accumulate(infection.burst_complete as u64);
        checksum.accumulate(infection.bloom_requested as u64);
    }
}

fn exposure_required_ticks(is_solo: bool, sim_hz: I32F32) -> u32 {
    if is_solo {
        fixed_seconds_to_ticks(CADAVER_SOLO_EXPOSURE_SECONDS, sim_hz)
    } else {
        fixed_seconds_to_ticks(CADAVER_MULTIPLAYER_EXPOSURE_SECONDS, sim_hz)
    }
}

fn infection_chance_basis_points(sensor: &CadaverEmployeeSensor) -> u32 {
    let mut chance = I32F32::lit("5000");

    chance += I32F32::from_num(sensor.cadaver_density_within_10) * I32F32::lit("250");

    if sensor.nearest_plant_distance_units <= I32F32::lit("5") {
        chance += I32F32::lit("1000");
    }

    chance -= I32F32::from_num(sensor.infected_employee_count) * I32F32::lit("150");

    if sensor.current_health >= sensor.max_health {
        chance *= CADAVER_FULL_HEALTH_CHANCE_MULTIPLIER;
    }

    if sensor.current_health <= I32F32::lit("60") {
        chance *= CADAVER_LOW_HEALTH_CHANCE_MULTIPLIER;
    }

    if sensor.current_health <= I32F32::lit("40") && sensor.cadaver_density_within_10 > 1 {
        chance *= CADAVER_CRITICAL_DENSE_CHANCE_MULTIPLIER;
    }

    fixed_clamp(chance, I32F32::lit("0"), I32F32::lit("10000")).to_num()
}

fn infection_speed_multiplier(sensor: &CadaverEmployeeSensor) -> I32F32 {
    let mut speed = I32F32::lit("1");

    if sensor.alive_employee_count == 4 && sensor.infected_employee_count > 0 {
        let infected = fixed_clamp(
            I32F32::from_num(sensor.infected_employee_count),
            I32F32::lit("1"),
            I32F32::lit("4"),
        );
        let span = CADAVER_LOBBY_FOUR_ONE_INFECTED_SPEED - CADAVER_LOBBY_FOUR_FOUR_INFECTED_SPEED;
        speed *= CADAVER_LOBBY_FOUR_ONE_INFECTED_SPEED - (span * (infected - I32F32::lit("1")) / I32F32::lit("3"));
    }

    if sensor.is_solo {
        speed *= CADAVER_SOLO_SPEED_MULTIPLIER;
    }

    if sensor.current_health >= sensor.max_health {
        speed *= CADAVER_FULL_HEALTH_SPEED_MULTIPLIER;
    }

    if sensor.current_health <= I32F32::lit("40") {
        speed *= CADAVER_LOW_HEALTH_SPEED_MULTIPLIER;
    }

    if sensor.nearest_employee_distance_units <= CADAVER_NEARBY_ACCELERATION_RANGE {
        speed = fixed_max(speed, I32F32::lit("1.5"));
    }

    speed
}

fn infection_stage_for_progress(progress: I32F32, burst_phase: bool) -> CadaverInfectionStage {
    if burst_phase {
        CadaverInfectionStage::Burst
    } else if progress == I32F32::lit("0") {
        CadaverInfectionStage::None
    } else if progress < CADAVER_SYMPTOMLESS_MAX {
        CadaverInfectionStage::Symptomless
    } else if progress < CADAVER_MILD_MAX {
        CadaverInfectionStage::Mild
    } else if progress < CADAVER_FEVER_THRESHOLD {
        CadaverInfectionStage::SporeTalk
    } else {
        CadaverInfectionStage::Fever
    }
}

fn exposure_remaining_percent(exposure_ticks: u32, required_ticks: u32) -> u8 {
    if required_ticks == 0 {
        return 0;
    }

    let remaining = required_ticks.saturating_sub(exposure_ticks);
    ((remaining * 100) / required_ticks) as u8
}

fn fever_temperature_tenths(progress: I32F32) -> u16 {
    let clamped = fixed_clamp(progress, CADAVER_FEVER_THRESHOLD, CADAVER_COMPLETE_INFECTION);
    let fever_progress = (clamped - CADAVER_FEVER_THRESHOLD) / (CADAVER_COMPLETE_INFECTION - CADAVER_FEVER_THRESHOLD);
    let temperature = I32F32::lit("1010") + fever_progress * I32F32::lit("110");
    temperature.to_num()
}

fn burst_fill_ticks(sensor: &CadaverEmployeeSensor, sim_hz: I32F32) -> u32 {
    if sensor.burst_meter_should_finish_fast() {
        scale_25hz_ticks(CADAVER_NEAR_EMPLOYEE_BURST_TICKS_AT_25HZ, sim_hz)
    } else if sensor.nearest_employee_distance_units >= I32F32::lit("30") || sensor.alive_employee_count <= 1 {
        scale_25hz_ticks(CADAVER_FAR_EMPLOYEE_BURST_TICKS_AT_25HZ, sim_hz)
    } else if sensor.is_solo {
        scale_25hz_ticks(CADAVER_SOLO_FINAL_BURST_TICKS_AT_25HZ, sim_hz)
    } else {
        scale_25hz_ticks(CADAVER_MULTIPLAYER_FINAL_BURST_TICKS_AT_25HZ, sim_hz)
    }
}

impl CadaverEmployeeSensor {
    fn burst_meter_should_finish_fast(&self) -> bool {
        self.nearest_employee_distance_units <= CADAVER_NEARBY_ACCELERATION_RANGE
    }
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

    if whole_ticks == 0 && ticks > 0 {
        1
    } else {
        whole_ticks
    }
}

fn scale_25hz_ticks(ticks_at_25hz: u32, sim_hz: I32F32) -> u32 {
    let scaled = I32F32::from_num(ticks_at_25hz) * sim_hz / I32F32::lit("25");
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

fn fixed_max(left: I32F32, right: I32F32) -> I32F32 {
    if left > right {
        left
    } else {
        right
    }
}

fn cadaver_stage_bits(stage: CadaverInfectionStage) -> u64 {
    match stage {
        CadaverInfectionStage::None => 0,
        CadaverInfectionStage::Symptomless => 1,
        CadaverInfectionStage::Mild => 2,
        CadaverInfectionStage::SporeTalk => 3,
        CadaverInfectionStage::Fever => 4,
        CadaverInfectionStage::Burst => 5,
    }
}