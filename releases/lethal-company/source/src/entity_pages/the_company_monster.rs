// Sources: vault/entity_pages/the_company_monster.md
use bevy::prelude::*;
use fixed::types::I16F16;
use rand_core::RngCore;

use crate::sim::{tick_rng, GameSeed, SimChecksumState, SimTick};

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

pub const THE_COMPANY_MONSTER_DEPENDS_ON: [&str; 4] = [
    "lethal_company",
    "company",
    "company_building",
    "moons",
];

pub const THE_COMPANY_MONSTER_FRONTMATTER_BEHAVIOR: [&str; 5] = [
    "Selects one of three moods at random when the ship lands on Gordion",
    "Detects noise from the bell, voice activity, and other sources",
    "Opens the counter door after enough noise is accumulated",
    "May attack when patience is low or negative",
    "Evaluates sold scrap and plays a voice line after the sale",
];

pub const COMPANY_MONSTER_BEHAVIORAL_MECHANICS: [CompanyMonsterBehaviorRule; 13] = [
    CompanyMonsterBehaviorRule {
        condition: "the ship lands on [[moons]]",
        outcome: "one of three company moods is selected at random, with the most aggressive mood able to appear before the first quota is fulfilled",
    },
    CompanyMonsterBehaviorRule {
        condition: "the mood is Agitated",
        outcome: "irritability is `0.8`, starting patience is `2`, sensitivity is `0.6`, judgement speed is `2`, and max player kills is `2`",
    },
    CompanyMonsterBehaviorRule {
        condition: "the mood is Silent Calm",
        outcome: "irritability is `0.4`, starting patience is `3`, sensitivity is `0.7`, judgement speed is `5`, max player kills is `1`, and the monster makes no sound at the counter",
    },
    CompanyMonsterBehaviorRule {
        condition: "the mood is Snoring Giant",
        outcome: "irritability is `0.5`, starting patience is `2`, sensitivity is `0.25`, judgement speed is `3`, max player kills is `1`, and the monster emits audible snoring at the counter",
    },
    CompanyMonsterBehaviorRule {
        condition: "a noise is detected from the bell, voice activity, or any other source",
        outcome: "the door timer decreases by the noise loudness divided by player count, and patience also decreases after a `1` second cooldown before the next patience drop",
    },
    CompanyMonsterBehaviorRule {
        condition: "patience drops below `1`",
        outcome: "the monster emits deep growling warnings before any attack",
    },
    CompanyMonsterBehaviorRule {
        condition: "patience drops below `0`",
        outcome: "every following noise has a `50%` chance to trigger an attack",
    },
    CompanyMonsterBehaviorRule {
        condition: "an attack triggers",
        outcome: "the attack lasts `3` seconds, can kill up to the mood's max player kills, adds `6` patience after retreating, and the door closes again `1` second after the attack",
    },
    CompanyMonsterBehaviorRule {
        condition: "scrap is taken",
        outcome: "item evaluation lasts for the mood's judgement speed",
    },
    CompanyMonsterBehaviorRule {
        condition: "average scrap value is less than or equal to `3` and patience is less than or equal to `2`",
        outcome: "there is a `30%` chance the monster attacks immediately after evaluation",
    },
    CompanyMonsterBehaviorRule {
        condition: "the immediate post-evaluation attack does not happen",
        outcome: "patience increases by `3`, the door closes, and a speaker voice line plays",
    },
    CompanyMonsterBehaviorRule {
        condition: "the total value of sold scrap is less than `25%` of the crew's current credit amount",
        outcome: "a bad-job sound plays; otherwise, a good-job sound plays",
    },
    CompanyMonsterBehaviorRule {
        condition: "the speaker is delivering a line",
        outcome: "there is about a `3%` chance a rare voice line replaces the normal one",
    },
];

pub const COMPANY_MONSTER_ATTACK_DURATION_TICKS: u32 = 60;
pub const COMPANY_MONSTER_ATTACK_RETREAT_PATIENCE_GAIN: i32 = 6;
pub const COMPANY_MONSTER_DOOR_CLOSE_AFTER_ATTACK_TICKS: u32 = 20;
pub const COMPANY_MONSTER_PATIENCE_COOLDOWN_TICKS: u32 = 20;
pub const COMPANY_MONSTER_POST_EVALUATION_ATTACK_PERCENT: u32 = 30;
pub const COMPANY_MONSTER_NEGATIVE_PATIENCE_ATTACK_PERCENT: u32 = 50;
pub const COMPANY_MONSTER_RARE_VOICE_LINE_PERCENT: u32 = 3;
pub const COMPANY_MONSTER_BAD_JOB_CREDIT_PERCENT: u32 = 25;

pub struct TheCompanyMonsterPlugin;

impl Plugin for TheCompanyMonsterPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CompanyMonsterState>()
            .add_event::<CompanyShipLandedEvent>()
            .add_event::<CompanyMonsterNoiseEvent>()
            .add_event::<CompanyMonsterScrapTakenEvent>()
            .add_event::<CompanyMonsterAttackEvent>()
            .add_event::<CompanyMonsterVoiceLineEvent>()
            .add_systems(
                FixedUpdate,
                (
                    company_monster_handle_ship_landed,
                    company_monster_handle_noise,
                    company_monster_handle_scrap_taken,
                    company_monster_tick_timers,
                    company_monster_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompanyMonsterBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompanyMonsterMood {
    Agitated,
    SilentCalm,
    SnoringGiant,
}

impl CompanyMonsterMood {
    pub fn profile(self) -> CompanyMonsterMoodProfile {
        match self {
            Self::Agitated => CompanyMonsterMoodProfile {
                mood: self,
                irritability: I16F16::from_num(0.8),
                starting_patience: I16F16::from_num(2),
                sensitivity: I16F16::from_num(0.6),
                judgement_speed_ticks: 40,
                max_player_kills: 2,
                counter_sound: CompanyMonsterCounterSound::Standard,
            },
            Self::SilentCalm => CompanyMonsterMoodProfile {
                mood: self,
                irritability: I16F16::from_num(0.4),
                starting_patience: I16F16::from_num(3),
                sensitivity: I16F16::from_num(0.7),
                judgement_speed_ticks: 100,
                max_player_kills: 1,
                counter_sound: CompanyMonsterCounterSound::Silent,
            },
            Self::SnoringGiant => CompanyMonsterMoodProfile {
                mood: self,
                irritability: I16F16::from_num(0.5),
                starting_patience: I16F16::from_num(2),
                sensitivity: I16F16::from_num(0.25),
                judgement_speed_ticks: 60,
                max_player_kills: 1,
                counter_sound: CompanyMonsterCounterSound::Snoring,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompanyMonsterMoodProfile {
    pub mood: CompanyMonsterMood,
    pub irritability: I16F16,
    pub starting_patience: I16F16,
    pub sensitivity: I16F16,
    pub judgement_speed_ticks: u32,
    pub max_player_kills: u8,
    pub counter_sound: CompanyMonsterCounterSound,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompanyMonsterCounterSound {
    Standard,
    Silent,
    Snoring,
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct CompanyMonsterState {
    pub mood: Option<CompanyMonsterMood>,
    pub patience: I16F16,
    pub door_timer: I16F16,
    pub patience_cooldown_ticks: u32,
    pub attack_ticks_remaining: u32,
    pub door_close_ticks_remaining: u32,
    pub evaluation_ticks_remaining: u32,
    pub pending_evaluation: Option<CompanyMonsterScrapEvaluation>,
    pub speaker_ticks_remaining: u32,
    pub warning_emitted: bool,
    pub door_open: bool,
}

impl Default for CompanyMonsterState {
    fn default() -> Self {
        Self {
            mood: None,
            patience: I16F16::from_num(0),
            door_timer: I16F16::from_num(10),
            patience_cooldown_ticks: 0,
            attack_ticks_remaining: 0,
            door_close_ticks_remaining: 0,
            evaluation_ticks_remaining: 0,
            pending_evaluation: None,
            speaker_ticks_remaining: 0,
            warning_emitted: false,
            door_open: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompanyMonsterScrapEvaluation {
    pub total_value: u32,
    pub item_count: u32,
    pub crew_credits: u32,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompanyShipLandedEvent {
    pub landed_on_moons: bool,
    pub first_quota_fulfilled: bool,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompanyMonsterNoiseEvent {
    pub source: CompanyMonsterNoiseSource,
    pub loudness: I16F16,
    pub player_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompanyMonsterNoiseSource {
    Bell,
    VoiceActivity,
    Other,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompanyMonsterScrapTakenEvent {
    pub total_value: u32,
    pub item_count: u32,
    pub crew_credits: u32,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompanyMonsterAttackEvent {
    pub duration_ticks: u32,
    pub max_player_kills: u8,
    pub cause: CompanyMonsterAttackCause,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompanyMonsterAttackCause {
    NoiseAfterNegativePatience,
    ScrapEvaluation,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompanyMonsterVoiceLineEvent {
    pub quality: CompanyMonsterSaleQuality,
    pub rare: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompanyMonsterSaleQuality {
    BadJob,
    GoodJob,
}

fn company_monster_handle_ship_landed(
    mut landed_events: EventReader<CompanyShipLandedEvent>,
    game_seed: Res<GameSeed>,
    tick: Res<SimTick>,
    mut state: ResMut<CompanyMonsterState>,
) {
    for event in landed_events.read() {
        if !event.landed_on_moons {
            continue;
        }

        let mood = select_company_monster_mood(game_seed.0, tick.0, event.first_quota_fulfilled);
        let profile = mood.profile();

        state.mood = Some(mood);
        state.patience = profile.starting_patience;
        state.door_timer = I16F16::from_num(10);
        state.patience_cooldown_ticks = 0;
        state.attack_ticks_remaining = 0;
        state.door_close_ticks_remaining = 0;
        state.evaluation_ticks_remaining = 0;
        state.pending_evaluation = None;
        state.speaker_ticks_remaining = 0;
        state.warning_emitted = false;
        state.door_open = false;
    }
}

fn company_monster_handle_noise(
    mut noise_events: EventReader<CompanyMonsterNoiseEvent>,
    mut attack_events: EventWriter<CompanyMonsterAttackEvent>,
    game_seed: Res<GameSeed>,
    tick: Res<SimTick>,
    mut state: ResMut<CompanyMonsterState>,
) {
    let Some(mood) = state.mood else {
        return;
    };

    let profile = mood.profile();

    for event in noise_events.read() {
        let player_count = event.player_count.max(1);
        let player_count_fixed = I16F16::from_num(player_count);
        let loudness_per_player = event.loudness / player_count_fixed;

        state.door_timer -= loudness_per_player * profile.sensitivity;

        if state.door_timer <= I16F16::from_num(0) {
            state.door_open = true;
            state.door_timer = I16F16::from_num(0);
        }

        if state.patience_cooldown_ticks == 0 {
            state.patience -= profile.irritability;
            state.patience_cooldown_ticks = COMPANY_MONSTER_PATIENCE_COOLDOWN_TICKS;
        }

        if state.patience < I16F16::from_num(1) {
            state.warning_emitted = true;
        }

        if state.patience < I16F16::from_num(0)
            && percent_roll(
                game_seed.0,
                tick.0,
                0x636f_6d70_616e_7901,
                COMPANY_MONSTER_NEGATIVE_PATIENCE_ATTACK_PERCENT,
            )
        {
            trigger_company_monster_attack(
                &mut state,
                &mut attack_events,
                CompanyMonsterAttackCause::NoiseAfterNegativePatience,
            );
        }
    }
}

fn company_monster_handle_scrap_taken(
    mut scrap_events: EventReader<CompanyMonsterScrapTakenEvent>,
    mut attack_events: EventWriter<CompanyMonsterAttackEvent>,
    mut voice_line_events: EventWriter<CompanyMonsterVoiceLineEvent>,
    game_seed: Res<GameSeed>,
    tick: Res<SimTick>,
    mut state: ResMut<CompanyMonsterState>,
) {
    let Some(mood) = state.mood else {
        return;
    };

    for event in scrap_events.read() {
        state.pending_evaluation = Some(CompanyMonsterScrapEvaluation {
            total_value: event.total_value,
            item_count: event.item_count,
            crew_credits: event.crew_credits,
        });
        state.evaluation_ticks_remaining = mood.profile().judgement_speed_ticks;
    }

    if state.evaluation_ticks_remaining != 0 {
        return;
    }

    let Some(evaluation) = state.pending_evaluation.take() else {
        return;
    };

    let item_count = evaluation.item_count.max(1);
    let average_scrap_value = evaluation.total_value / item_count;
    let low_value_scrap = average_scrap_value <= 3;
    let low_patience = state.patience <= I16F16::from_num(2);

    if low_value_scrap
        && low_patience
        && percent_roll(
            game_seed.0,
            tick.0,
            0x636f_6d70_616e_7902,
            COMPANY_MONSTER_POST_EVALUATION_ATTACK_PERCENT,
        )
    {
        trigger_company_monster_attack(
            &mut state,
            &mut attack_events,
            CompanyMonsterAttackCause::ScrapEvaluation,
        );
        return;
    }

    state.patience += I16F16::from_num(3);
    state.door_open = false;
    state.speaker_ticks_remaining = 20;

    voice_line_events.send(CompanyMonsterVoiceLineEvent {
        quality: sale_quality(evaluation.total_value, evaluation.crew_credits),
        rare: percent_roll(
            game_seed.0,
            tick.0,
            0x636f_6d70_616e_7903,
            COMPANY_MONSTER_RARE_VOICE_LINE_PERCENT,
        ),
    });
}

fn company_monster_tick_timers(mut state: ResMut<CompanyMonsterState>) {
    state.patience_cooldown_ticks = state.patience_cooldown_ticks.saturating_sub(1);

    if state.attack_ticks_remaining > 0 {
        state.attack_ticks_remaining -= 1;

        if state.attack_ticks_remaining == 0 {
            state.patience += I16F16::from_num(COMPANY_MONSTER_ATTACK_RETREAT_PATIENCE_GAIN);
            state.door_close_ticks_remaining = COMPANY_MONSTER_DOOR_CLOSE_AFTER_ATTACK_TICKS;
        }
    }

    if state.door_close_ticks_remaining > 0 {
        state.door_close_ticks_remaining -= 1;

        if state.door_close_ticks_remaining == 0 {
            state.door_open = false;
        }
    }

    if state.evaluation_ticks_remaining > 0 {
        state.evaluation_ticks_remaining -= 1;
    }

    state.speaker_ticks_remaining = state.speaker_ticks_remaining.saturating_sub(1);
}

fn company_monster_checksum(
    mut checksum: ResMut<SimChecksumState>,
    tick: Res<SimTick>,
    state: Res<CompanyMonsterState>,
) {
    checksum.accumulate(tick.0);
    checksum.accumulate(THE_COMPANY_MONSTER_SOURCE_REVISION as u64);
    checksum.accumulate(THE_COMPANY_MONSTER_CONFIDENCE_BASIS_POINTS as u64);

    accumulate_str(&mut checksum, 0x1000, THE_COMPANY_MONSTER_ID);
    accumulate_str(&mut checksum, 0x1001, THE_COMPANY_MONSTER_NAME);
    accumulate_str(&mut checksum, 0x1002, THE_COMPANY_MONSTER_TYPE);
    accumulate_str(&mut checksum, 0x1003, THE_COMPANY_MONSTER_SUBTYPE);
    accumulate_str(&mut checksum, 0x1004, THE_COMPANY_MONSTER_SOURCE_URL);
    accumulate_str(&mut checksum, 0x1005, THE_COMPANY_MONSTER_EXTRACTED_AT);
    accumulate_str(&mut checksum, 0x1006, THE_COMPANY_MONSTER_DANGER);
    accumulate_str(&mut checksum, 0x1007, THE_COMPANY_MONSTER_SCIENTIFIC_NAME);
    accumulate_str(&mut checksum, 0x1008, THE_COMPANY_MONSTER_DWELLS);
    accumulate_str(&mut checksum, 0x1009, THE_COMPANY_MONSTER_INTERNAL_NAME);
    accumulate_str(&mut checksum, 0x100a, THE_COMPANY_MONSTER_ATTACK_DAMAGE);

    for dependency in THE_COMPANY_MONSTER_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x2000, dependency);
    }

    for behavior in THE_COMPANY_MONSTER_FRONTMATTER_BEHAVIOR {
        accumulate_str(&mut checksum, 0x3000, behavior);
    }

    for rule in COMPANY_MONSTER_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x4000, rule.condition);
        accumulate_str(&mut checksum, 0x4001, rule.outcome);
    }

    if let Some(mood) = state.mood {
        checksum.accumulate(0x5000 ^ mood_index(mood) as u64);
    }

    checksum.accumulate(state.patience.to_bits() as u64);
    checksum.accumulate(state.door_timer.to_bits() as u64);
    checksum.accumulate(state.patience_cooldown_ticks as u64);
    checksum.accumulate(state.attack_ticks_remaining as u64);
    checksum.accumulate(state.door_close_ticks_remaining as u64);
    checksum.accumulate(state.evaluation_ticks_remaining as u64);
    checksum.accumulate(state.speaker_ticks_remaining as u64);
    checksum.accumulate(state.warning_emitted as u64);
    checksum.accumulate(state.door_open as u64);

    if let Some(evaluation) = state.pending_evaluation {
        checksum.accumulate(0x6000);
        checksum.accumulate(evaluation.total_value as u64);
        checksum.accumulate(evaluation.item_count as u64);
        checksum.accumulate(evaluation.crew_credits as u64);
    }
}

pub fn select_company_monster_mood(
    game_seed: u64,
    tick: u64,
    first_quota_fulfilled: bool,
) -> CompanyMonsterMood {
    let mut rng = tick_rng(game_seed, tick, 0x636f_6d70_616e_7900);
    let roll = rng.next_u32() % 3;

    match roll {
        0 => CompanyMonsterMood::SilentCalm,
        1 => CompanyMonsterMood::SnoringGiant,
        _ => {
            let _agitated_can_appear_before_first_quota = !first_quota_fulfilled;
            CompanyMonsterMood::Agitated
        }
    }
}

fn trigger_company_monster_attack(
    state: &mut CompanyMonsterState,
    attack_events: &mut EventWriter<CompanyMonsterAttackEvent>,
    cause: CompanyMonsterAttackCause,
) {
    let Some(mood) = state.mood else {
        return;
    };

    let profile = mood.profile();
    state.attack_ticks_remaining = COMPANY_MONSTER_ATTACK_DURATION_TICKS;
    state.door_open = true;

    attack_events.send(CompanyMonsterAttackEvent {
        duration_ticks: COMPANY_MONSTER_ATTACK_DURATION_TICKS,
        max_player_kills: profile.max_player_kills,
        cause,
    });
}

fn sale_quality(total_value: u32, crew_credits: u32) -> CompanyMonsterSaleQuality {
    if total_value.saturating_mul(100) < crew_credits.saturating_mul(COMPANY_MONSTER_BAD_JOB_CREDIT_PERCENT) {
        return CompanyMonsterSaleQuality::BadJob;
    }

    CompanyMonsterSaleQuality::GoodJob
}

fn percent_roll(game_seed: u64, tick: u64, salt: u64, percent: u32) -> bool {
    let mut rng = tick_rng(game_seed, tick, salt);
    rng.next_u32() % 100 < percent
}

fn mood_index(mood: CompanyMonsterMood) -> u8 {
    match mood {
        CompanyMonsterMood::Agitated => 0,
        CompanyMonsterMood::SilentCalm => 1,
        CompanyMonsterMood::SnoringGiant => 2,
    }
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt ^ value.len() as u64);

    for (index, byte) in value.bytes().enumerate() {
        checksum.accumulate(salt ^ ((index as u64) << 8) ^ byte as u64);
    }
}