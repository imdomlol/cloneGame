// Sources: vault/entity_pages/the_company_monster.md, vault/gameplay_mechanics/fear.md
use bevy::prelude::*;
use fixed::types::{I16F16, I32F32};
use std::collections::BTreeMap;

use crate::gameplay_mechanics::fear::{ApplyFearEvent, FearTrigger};
use crate::sim::{
    DamageType, Health, IncomingDamageEvent, SimChecksumState, SimPosition, SimTick, UnitStats,
};
use crate::system::game_state_machine::GameState;

pub const CREATURE_AI_ID: &str = "creature_ai";
pub const CREATURE_AI_NAME: &str = "Creature AI";
pub const CREATURE_AI_TYPE: &str = "system";
pub const CREATURE_AI_SUBTYPE: &str = "hostile_behavior";

pub const THE_COMPANY_MONSTER_ID: &str = "the_company_monster";
pub const THE_COMPANY_MONSTER_NAME: &str = "The Company Monster";
pub const THE_COMPANY_MONSTER_TYPE: &str = "entity_pages";
pub const THE_COMPANY_MONSTER_SUBTYPE: &str = "special_entity";
pub const THE_COMPANY_MONSTER_SOURCE_URL: &str =
    "https://lethal-company.fandom.com/wiki/The_Company_Monster";
pub const THE_COMPANY_MONSTER_SOURCE_REVISION: u32 = 21363;
pub const THE_COMPANY_MONSTER_EXTRACTED_AT: &str = "2026-06-07";
pub const THE_COMPANY_MONSTER_CONFIDENCE_BASIS_POINTS: u16 = 88;
pub const THE_COMPANY_MONSTER_DANGER: &str = "N/A";
pub const THE_COMPANY_MONSTER_SCIENTIFIC_NAME: &str = "N/A";
pub const THE_COMPANY_MONSTER_DWELLS: &str = "Inside the Company";
pub const THE_COMPANY_MONSTER_INTERNAL_NAME: &str = "CompanyMonster";
pub const THE_COMPANY_MONSTER_ATTACK_DAMAGE: &str = "Instant Kill";

pub const FEAR_ID: &str = "fear";
pub const FEAR_NAME: &str = "Fear";
pub const FEAR_TYPE: &str = "gameplay_mechanics";
pub const FEAR_SUBTYPE: &str = "status_effect";
pub const FEAR_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Fear";
pub const FEAR_SOURCE_REVISION: u32 = 18982;
pub const FEAR_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const FEAR_CONFIDENCE_BASIS_POINTS: u16 = 66;
pub const FEAR_OVERVIEW: &str = "Temporary status effect that distorts an employee's screen, obscures most of the UI, and adds a harsh synth-like sound when applied.";

pub const COMPANY_MONSTER_MOOD_COUNT: u8 = 3;
pub const COMPANY_MONSTER_AGITATED_IRRITABILITY: I16F16 = I16F16::lit("0.8");
pub const COMPANY_MONSTER_AGITATED_PATIENCE: I16F16 = I16F16::lit("2");
pub const COMPANY_MONSTER_AGITATED_SENSITIVITY: I16F16 = I16F16::lit("0.6");
pub const COMPANY_MONSTER_AGITATED_JUDGEMENT_SPEED_SECONDS: I16F16 = I16F16::lit("2");
pub const COMPANY_MONSTER_AGITATED_MAX_PLAYER_KILLS: u8 = 2;
pub const COMPANY_MONSTER_SILENT_CALM_IRRITABILITY: I16F16 = I16F16::lit("0.4");
pub const COMPANY_MONSTER_SILENT_CALM_PATIENCE: I16F16 = I16F16::lit("3");
pub const COMPANY_MONSTER_SILENT_CALM_SENSITIVITY: I16F16 = I16F16::lit("0.7");
pub const COMPANY_MONSTER_SILENT_CALM_JUDGEMENT_SPEED_SECONDS: I16F16 = I16F16::lit("5");
pub const COMPANY_MONSTER_SILENT_CALM_MAX_PLAYER_KILLS: u8 = 1;
pub const COMPANY_MONSTER_SNORING_GIANT_IRRITABILITY: I16F16 = I16F16::lit("0.5");
pub const COMPANY_MONSTER_SNORING_GIANT_PATIENCE: I16F16 = I16F16::lit("2");
pub const COMPANY_MONSTER_SNORING_GIANT_SENSITIVITY: I16F16 = I16F16::lit("0.25");
pub const COMPANY_MONSTER_SNORING_GIANT_JUDGEMENT_SPEED_SECONDS: I16F16 = I16F16::lit("3");
pub const COMPANY_MONSTER_SNORING_GIANT_MAX_PLAYER_KILLS: u8 = 1;
pub const COMPANY_MONSTER_NOISE_PATIENCE_COOLDOWN_SECONDS: I16F16 = I16F16::lit("1");
pub const COMPANY_MONSTER_ATTACK_CHANCE_PERCENT: u16 = 50;
pub const COMPANY_MONSTER_ATTACK_DURATION_SECONDS: I16F16 = I16F16::lit("3");
pub const COMPANY_MONSTER_ATTACK_RETREAT_PATIENCE_GAIN: I16F16 = I16F16::lit("6");
pub const COMPANY_MONSTER_DOOR_CLOSE_AFTER_ATTACK_SECONDS: I16F16 = I16F16::lit("1");
pub const COMPANY_MONSTER_LOW_SCRAP_AVERAGE_VALUE: u32 = 3;
pub const COMPANY_MONSTER_LOW_SCRAP_ATTACK_CHANCE_PERCENT: u16 = 30;
pub const COMPANY_MONSTER_SALE_PATIENCE_GAIN: I16F16 = I16F16::lit("3");
pub const COMPANY_MONSTER_BAD_JOB_CREDIT_PERCENT: u16 = 25;
pub const COMPANY_MONSTER_RARE_VOICE_LINE_PERCENT: u16 = 3;
pub const FEAR_CRITICAL_HEALTH_THRESHOLD: I32F32 = I32F32::lit("20");
pub const FEAR_BASE_INTENSITY: I16F16 = I16F16::lit("1");

pub const COMPANY_MONSTER_BEHAVIORAL_MECHANICS: [CreatureAiRule; 13] = [
    CreatureAiRule {
        condition: "the ship lands on moons",
        outcome: "one of three company moods is selected at random, with the most aggressive mood able to appear before the first quota is fulfilled",
    },
    CreatureAiRule {
        condition: "the mood is Agitated",
        outcome: "irritability is 0.8, starting patience is 2, sensitivity is 0.6, judgement speed is 2, and max player kills is 2",
    },
    CreatureAiRule {
        condition: "the mood is Silent Calm",
        outcome: "irritability is 0.4, starting patience is 3, sensitivity is 0.7, judgement speed is 5, max player kills is 1, and the monster makes no sound at the counter",
    },
    CreatureAiRule {
        condition: "the mood is Snoring Giant",
        outcome: "irritability is 0.5, starting patience is 2, sensitivity is 0.25, judgement speed is 3, max player kills is 1, and the monster emits audible snoring at the counter",
    },
    CreatureAiRule {
        condition: "a noise is detected from the bell, voice activity, or any other source",
        outcome: "the door timer decreases by the noise loudness divided by player count, and patience also decreases after a 1 second cooldown before the next patience drop",
    },
    CreatureAiRule {
        condition: "patience drops below 1",
        outcome: "the monster emits deep growling warnings before any attack",
    },
    CreatureAiRule {
        condition: "patience drops below 0",
        outcome: "every following noise has a 50% chance to trigger an attack",
    },
    CreatureAiRule {
        condition: "an attack triggers",
        outcome: "the attack lasts 3 seconds, can kill up to the mood's max player kills, adds 6 patience after retreating, and the door closes again 1 second after the attack",
    },
    CreatureAiRule {
        condition: "scrap is taken",
        outcome: "item evaluation lasts for the mood's judgement speed",
    },
    CreatureAiRule {
        condition: "average scrap value is less than or equal to 3 and patience is less than or equal to 2",
        outcome: "there is a 30% chance the monster attacks immediately after evaluation",
    },
    CreatureAiRule {
        condition: "the immediate post-evaluation attack does not happen",
        outcome: "patience increases by 3, the door closes, and a speaker voice line plays",
    },
    CreatureAiRule {
        condition: "the total value of sold scrap is less than 25% of the crew's current credit amount",
        outcome: "a bad-job sound plays; otherwise, a good-job sound plays",
    },
    CreatureAiRule {
        condition: "the speaker is delivering a line",
        outcome: "there is about a 3% chance a rare voice line replaces the normal one",
    },
];

pub const FEAR_BEHAVIORAL_MECHANICS: [CreatureAiRule; 20] = [
    CreatureAiRule {
        condition: "fear is active",
        outcome: "the affected employee's screen becomes distorted and blurry, most of the UI is harder to read, and a jarring synth-like sound plays for a few seconds after application",
    },
    CreatureAiRule {
        condition: "an employee's HP drops below 20",
        outcome: "the employee automatically enters critical health and becomes feared until health regenerates back to 20",
    },
    CreatureAiRule {
        condition: "an employee directly views another employee's player_body",
        outcome: "the viewer becomes feared",
    },
    CreatureAiRule {
        condition: "a teleporter returns an employee corpse to the ship",
        outcome: "anyone inside the ship or otherwise within vision of the body becomes feared",
    },
    CreatureAiRule {
        condition: "an employee only sees another employee fall into weather water or march quicksand",
        outcome: "the fear condition does not trigger because the moment of death is not witnessed",
    },
    CreatureAiRule {
        condition: "an employee sees another employee die to the company_monster, transform into a masked, or get eaten by the earth_leviathan",
        outcome: "the fear condition does not trigger",
    },
    CreatureAiRule {
        condition: "a hoarding_bug flies near an employee",
        outcome: "the employee becomes feared",
    },
    CreatureAiRule {
        condition: "a snare_flea drops and attempts to ensnare an employee",
        outcome: "the employee becomes feared",
    },
    CreatureAiRule {
        condition: "a snare_flea successfully ensnares an employee",
        outcome: "the employee remains feared until freed and far enough away from the flea",
    },
    CreatureAiRule {
        condition: "a snare_flea fails to ensnare an employee",
        outcome: "the employee remains feared until the flea stops pursuing them",
    },
    CreatureAiRule {
        condition: "a thumper, eyeless_dog, or giant_sapsucker targets an employee",
        outcome: "the employee becomes feared",
    },
    CreatureAiRule {
        condition: "a coil_head freezes close enough to an employee",
        outcome: "the employee becomes feared",
    },
    CreatureAiRule {
        condition: "a bracken is spotted during its hunt",
        outcome: "the employee becomes feared, and the effect intensity scales with the distance to the bracken",
    },
    CreatureAiRule {
        condition: "a forest_keeper looks in an employee's direction",
        outcome: "the employee hears ambience similar to the fear effect",
    },
    CreatureAiRule {
        condition: "a forest_keeper spots an employee",
        outcome: "the full fear effect applies",
    },
    CreatureAiRule {
        condition: "a masked is interrupted while attacking an employee",
        outcome: "the employee becomes feared",
    },
    CreatureAiRule {
        condition: "a jester pops",
        outcome: "all nearby employees become feared",
    },
    CreatureAiRule {
        condition: "the entity that caused the fear effect dies",
        outcome: "the effect dissipates",
    },
    CreatureAiRule {
        condition: "a bunker_spider, mask_hornets, hygrodere, ghost_girl, spore_lizard, nutcracker, baboon_hawk, circuit_bee, tulip_snake, manticoil, or roaming_locust is involved",
        outcome: "the page lists no unique fear interaction for that entity",
    },
    CreatureAiRule {
        condition: "line of sight is poor on a moon such as rend or inside the ship",
        outcome: "the fear ambience can act as a warning for nearby threats",
    },
];

const DEFAULT_ATTACK_COOLDOWN_TICKS: u16 = 25;

pub struct CreatureAiPlugin;

impl Plugin for CreatureAiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CreatureAiState>()
            .add_event::<CreatureAiTargetChangedEvent>()
            .add_event::<CreatureAiMovedEvent>()
            .add_event::<CreatureAiAttackEvent>()
            .add_systems(
                FixedUpdate,
                (
                    creature_ai_acquire_targets,
                    creature_ai_move_toward_targets,
                    creature_ai_attack_targets,
                    creature_ai_decay_aggression,
                    creature_ai_checksum,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CreatureAiRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum CreatureAiKind {
    #[default]
    GenericHostile = 0,
    HoardingBug = 1,
    SnareFlea = 2,
    Thumper = 3,
    EyelessDog = 4,
    GiantSapsucker = 5,
    CoilHead = 6,
    Bracken = 7,
    ForestKeeper = 8,
    Masked = 9,
    Jester = 10,
    CompanyMonster = 11,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CreatureAi {
    pub stable_id: u64,
    pub kind: CreatureAiKind,
    pub hostile: bool,
    pub aggression: I16F16,
    pub max_aggression: I16F16,
    pub attack_cooldown_ticks: u16,
    pub current_attack_cooldown_ticks: u16,
}

impl Default for CreatureAi {
    fn default() -> Self {
        Self {
            stable_id: 0,
            kind: CreatureAiKind::GenericHostile,
            hostile: true,
            aggression: I16F16::ZERO,
            max_aggression: I16F16::lit("1"),
            attack_cooldown_ticks: DEFAULT_ATTACK_COOLDOWN_TICKS,
            current_attack_cooldown_ticks: 0,
        }
    }
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CreatureAiTarget {
    pub target: Option<Entity>,
    pub target_stable_id: u64,
    pub distance_squared: I32F32,
    pub last_seen_tick: u64,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EmployeeAiTarget {
    pub stable_id: u64,
}

#[derive(Resource, Debug, Clone, PartialEq, Eq, Default)]
pub struct CreatureAiState {
    pub target_changes: u64,
    pub movement_steps: u64,
    pub attacks: u64,
    pub fear_applications_requested: u64,
    pub last_creature_stable_id: u64,
    pub last_target_stable_id: u64,
    pub active_targets_by_creature: BTreeMap<u64, u64>,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreatureAiTargetChangedEvent {
    pub creature: Entity,
    pub creature_stable_id: u64,
    pub old_target_stable_id: u64,
    pub new_target: Option<Entity>,
    pub new_target_stable_id: u64,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreatureAiMovedEvent {
    pub creature: Entity,
    pub creature_stable_id: u64,
    pub target_stable_id: u64,
    pub new_position: SimPosition,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreatureAiAttackEvent {
    pub creature: Entity,
    pub creature_stable_id: u64,
    pub target: Entity,
    pub target_stable_id: u64,
    pub damage: I32F32,
}

fn creature_ai_acquire_targets(
    tick: Res<SimTick>,
    mut state: ResMut<CreatureAiState>,
    mut creatures: Query<(Entity, &CreatureAi, &SimPosition, &UnitStats, &mut CreatureAiTarget)>,
    employees: Query<(Entity, &EmployeeAiTarget, &SimPosition, Option<&Health>)>,
    mut changed_events: EventWriter<CreatureAiTargetChangedEvent>,
    mut fear_events: EventWriter<ApplyFearEvent>,
) {
    let mut changed = Vec::new();
    let mut fear_requests = Vec::new();

    for (creature_entity, creature, creature_position, stats, mut target) in creatures.iter_mut() {
        if !creature.hostile {
            continue;
        }

        let watch_range_squared = stats.watch_range * stats.watch_range;
        let mut best_target: Option<(Entity, u64, I32F32)> = None;

        for (employee_entity, employee, employee_position, health) in employees.iter() {
            if health.is_some_and(|hp| hp.current <= I32F32::ZERO) {
                continue;
            }

            let distance_squared = distance_squared(*creature_position, *employee_position);
            if distance_squared > watch_range_squared {
                continue;
            }

            let replace = match best_target {
                None => true,
                Some((_, best_id, best_distance)) => {
                    distance_squared < best_distance
                        || (distance_squared == best_distance && employee.stable_id < best_id)
                }
            };

            if replace {
                best_target = Some((employee_entity, employee.stable_id, distance_squared));
            }
        }

        let old_target_stable_id = target.target_stable_id;

        if let Some((target_entity, target_stable_id, target_distance_squared)) = best_target {
            target.target = Some(target_entity);
            target.target_stable_id = target_stable_id;
            target.distance_squared = target_distance_squared;
            target.last_seen_tick = tick.0;
        } else {
            target.target = None;
            target.target_stable_id = 0;
            target.distance_squared = I32F32::ZERO;
        }

        if old_target_stable_id == target.target_stable_id {
            continue;
        }

        changed.push(CreatureAiTargetChangedEvent {
            creature: creature_entity,
            creature_stable_id: creature.stable_id,
            old_target_stable_id,
            new_target: target.target,
            new_target_stable_id: target.target_stable_id,
        });

        if target.target_stable_id != 0 {
            if let Some(trigger) = fear_trigger_for_creature(creature.kind) {
                fear_requests.push(ApplyFearEvent {
                    employee_id: target.target_stable_id,
                    trigger,
                    source_entity_id: source_entity_id_for_creature(creature.kind),
                    intensity: FEAR_BASE_INTENSITY,
                    persists_until_cleared: false,
                });
            }
        }
    }

    changed.sort_by_key(|event| event.creature_stable_id);
    for event in changed {
        state.target_changes = state.target_changes.wrapping_add(1);
        state.last_creature_stable_id = event.creature_stable_id;
        state.last_target_stable_id = event.new_target_stable_id;

        if event.new_target_stable_id == 0 {
            state.active_targets_by_creature.remove(&event.creature_stable_id);
        } else {
            state
                .active_targets_by_creature
                .insert(event.creature_stable_id, event.new_target_stable_id);
        }

        changed_events.send(event);
    }

    fear_requests.sort_by_key(|event| event.employee_id);
    for event in fear_requests {
        state.fear_applications_requested = state.fear_applications_requested.wrapping_add(1);
        fear_events.send(event);
    }
}

fn creature_ai_move_toward_targets(
    mut state: ResMut<CreatureAiState>,
    mut creatures: Query<(Entity, &CreatureAi, &mut SimPosition, &UnitStats, &CreatureAiTarget)>,
    employees: Query<&SimPosition, (With<EmployeeAiTarget>, Without<CreatureAi>)>,
    mut moved_events: EventWriter<CreatureAiMovedEvent>,
) {
    let mut moved = Vec::new();

    for (creature_entity, creature, mut creature_position, stats, target) in creatures.iter_mut() {
        if !creature.hostile {
            continue;
        }

        let Some(target_entity) = target.target else {
            continue;
        };

        let Ok(target_position) = employees.get(target_entity) else {
            continue;
        };

        let attack_range_squared = stats.attack_range * stats.attack_range;
        if distance_squared(*creature_position, *target_position) <= attack_range_squared {
            continue;
        }

        let old_position = *creature_position;
        step_axis_toward(&mut creature_position.x, target_position.x, stats.move_speed);
        step_axis_toward(&mut creature_position.y, target_position.y, stats.move_speed);

        if *creature_position == old_position {
            continue;
        }

        moved.push(CreatureAiMovedEvent {
            creature: creature_entity,
            creature_stable_id: creature.stable_id,
            target_stable_id: target.target_stable_id,
            new_position: *creature_position,
        });
    }

    moved.sort_by_key(|event| event.creature_stable_id);
    for event in moved {
        state.movement_steps = state.movement_steps.wrapping_add(1);
        state.last_creature_stable_id = event.creature_stable_id;
        state.last_target_stable_id = event.target_stable_id;
        moved_events.send(event);
    }
}

fn creature_ai_attack_targets(
    mut state: ResMut<CreatureAiState>,
    mut creatures: Query<(
        Entity,
        &mut CreatureAi,
        &SimPosition,
        &UnitStats,
        &CreatureAiTarget,
    )>,
    employees: Query<&SimPosition, With<EmployeeAiTarget>>,
    mut attack_events: EventWriter<CreatureAiAttackEvent>,
    mut damage_events: EventWriter<IncomingDamageEvent>,
) {
    let mut attacks = Vec::new();

    for (creature_entity, mut creature, creature_position, stats, target) in creatures.iter_mut() {
        if !creature.hostile {
            continue;
        }

        if creature.current_attack_cooldown_ticks > 0 {
            creature.current_attack_cooldown_ticks -= 1;
            continue;
        }

        let Some(target_entity) = target.target else {
            continue;
        };

        let Ok(target_position) = employees.get(target_entity) else {
            continue;
        };

        let attack_range_squared = stats.attack_range * stats.attack_range;
        if distance_squared(*creature_position, *target_position) > attack_range_squared {
            continue;
        }

        creature.current_attack_cooldown_ticks = creature.attack_cooldown_ticks;
        creature.aggression = (creature.aggression + I16F16::lit("0.1")).min(creature.max_aggression);

        attacks.push(CreatureAiAttackEvent {
            creature: creature_entity,
            creature_stable_id: creature.stable_id,
            target: target_entity,
            target_stable_id: target.target_stable_id,
            damage: stats.attack_damage,
        });
    }

    attacks.sort_by_key(|event| event.creature_stable_id);
    for event in attacks {
        state.attacks = state.attacks.wrapping_add(1);
        state.last_creature_stable_id = event.creature_stable_id;
        state.last_target_stable_id = event.target_stable_id;

        damage_events.send(IncomingDamageEvent {
            target: event.target,
            raw_amount: event.damage,
            damage_type: DamageType::Standard,
            source: event.creature,
        });
        attack_events.send(event);
    }
}

fn creature_ai_decay_aggression(mut creatures: Query<&mut CreatureAi>) {
    for mut creature in creatures.iter_mut() {
        if creature.aggression <= I16F16::ZERO {
            continue;
        }

        creature.aggression = (creature.aggression - I16F16::lit("0.02")).max(I16F16::ZERO);
    }
}

fn creature_ai_checksum(
    tick: Res<SimTick>,
    mut checksum: ResMut<SimChecksumState>,
    state: Res<CreatureAiState>,
    creatures: Query<(&CreatureAi, &CreatureAiTarget, &SimPosition, &UnitStats)>,
    employees: Query<(&EmployeeAiTarget, &SimPosition)>,
) {
    checksum.accumulate(tick.0);
    checksum.accumulate(state.target_changes);
    checksum.accumulate(state.movement_steps);
    checksum.accumulate(state.attacks);
    checksum.accumulate(state.fear_applications_requested);
    checksum.accumulate(state.last_creature_stable_id);
    checksum.accumulate(state.last_target_stable_id);

    for (creature_id, target_id) in state.active_targets_by_creature.iter() {
        checksum.accumulate(*creature_id);
        checksum.accumulate(*target_id);
    }

    for (creature, target, position, stats) in creatures.iter() {
        checksum.accumulate(creature.stable_id);
        checksum.accumulate(creature.kind as u64);
        checksum.accumulate(creature.hostile as u64);
        checksum.accumulate(creature.aggression.to_bits() as u64);
        checksum.accumulate(creature.max_aggression.to_bits() as u64);
        checksum.accumulate(creature.attack_cooldown_ticks as u64);
        checksum.accumulate(creature.current_attack_cooldown_ticks as u64);
        checksum.accumulate(target.target_stable_id);
        checksum.accumulate(target.distance_squared.to_bits() as u64);
        checksum.accumulate(target.last_seen_tick);
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(stats.move_speed.to_bits() as u64);
        checksum.accumulate(stats.attack_range.to_bits() as u64);
        checksum.accumulate(stats.attack_damage.to_bits() as u64);
        checksum.accumulate(stats.attack_speed.to_bits() as u64);
        checksum.accumulate(stats.watch_range.to_bits() as u64);
    }

    for (employee, position) in employees.iter() {
        checksum.accumulate(employee.stable_id);
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
    }
}

fn distance_squared(a: SimPosition, b: SimPosition) -> I32F32 {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    dx * dx + dy * dy
}

fn step_axis_toward(current: &mut I32F32, target: I32F32, step: I32F32) {
    let delta = target - *current;

    if delta > step {
        *current += step;
    } else if delta < -step {
        *current -= step;
    } else {
        *current = target;
    }
}

fn fear_trigger_for_creature(kind: CreatureAiKind) -> Option<FearTrigger> {
    match kind {
        CreatureAiKind::HoardingBug => Some(FearTrigger::HoardingBugNearFlight),
        CreatureAiKind::SnareFlea => Some(FearTrigger::SnareFleaAttemptedEnsnare),
        CreatureAiKind::Thumper
        | CreatureAiKind::EyelessDog
        | CreatureAiKind::GiantSapsucker => Some(FearTrigger::ThreatTargetedEmployee),
        CreatureAiKind::CoilHead => Some(FearTrigger::CoilHeadFrozenNearby),
        CreatureAiKind::Bracken => Some(FearTrigger::BrackenSpottedDuringHunt),
        CreatureAiKind::ForestKeeper => Some(FearTrigger::ForestKeeperSpottedEmployee),
        CreatureAiKind::Masked => Some(FearTrigger::MaskedInterruptedAttack),
        CreatureAiKind::Jester => Some(FearTrigger::JesterPoppedNearby),
        CreatureAiKind::GenericHostile | CreatureAiKind::CompanyMonster => None,
    }
}

fn source_entity_id_for_creature(kind: CreatureAiKind) -> &'static str {
    match kind {
        CreatureAiKind::GenericHostile => "entity",
        CreatureAiKind::HoardingBug => "hoarding_bug",
        CreatureAiKind::SnareFlea => "snare_flea",
        CreatureAiKind::Thumper => "thumper",
        CreatureAiKind::EyelessDog => "eyeless_dog",
        CreatureAiKind::GiantSapsucker => "giant_sapsucker",
        CreatureAiKind::CoilHead => "coil_head",
        CreatureAiKind::Bracken => "bracken",
        CreatureAiKind::ForestKeeper => "forest_keeper",
        CreatureAiKind::Masked => "masked",
        CreatureAiKind::Jester => "jester",
        CreatureAiKind::CompanyMonster => "company_monster",
    }
}