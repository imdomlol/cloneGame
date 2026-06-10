// Sources: vault/gameplay_mechanics/company_ranks.md
use bevy::prelude::*;

use crate::sim::{SimChecksumState, SimTick};

pub const COMPANY_RANKS_ID: &str = "company_ranks";
pub const COMPANY_RANKS_NAME: &str = "Company Ranks";
pub const COMPANY_RANKS_TYPE: &str = "gameplay_mechanics";
pub const COMPANY_RANKS_SUBTYPE: &str = "company_ranks";
pub const COMPANY_RANKS_SOURCE_URL: &str =
    "https://lethal-company.fandom.com/wiki/Company_Ranks";
pub const COMPANY_RANKS_SOURCE_REVISION: u32 = 9190;
pub const COMPANY_RANKS_EXTRACTED_AT: &str = "2026-06-07T00:00:00-07:00";
pub const COMPANY_RANKS_CONFIDENCE_PERCENT: u16 = 94;

pub const COMPANY_RANKS_INTERN_XP: i32 = 0;
pub const COMPANY_RANKS_PART_TIMER_XP: i32 = 50;
pub const COMPANY_RANKS_EMPLOYEE_XP: i32 = 100;
pub const COMPANY_RANKS_LEADER_XP: i32 = 200;
pub const COMPANY_RANKS_BOSS_XP: i32 = 500;
pub const COMPANY_RANKS_RESET_XP: i32 = 1500;

pub const COMPANY_RANKS_DEPENDS_ON: [&str; 1] = ["lethal_company"];

pub const COMPANY_RANKS_RULES: [CompanyRanksBehaviorRule; 6] = [
    CompanyRanksBehaviorRule {
        condition: "company XP has not reached 50",
        outcome: "the rank is Intern",
    },
    CompanyRanksBehaviorRule {
        condition: "company XP reaches 50",
        outcome: "the rank becomes Part-Timer",
    },
    CompanyRanksBehaviorRule {
        condition: "company XP reaches 100",
        outcome: "the rank becomes Employee",
    },
    CompanyRanksBehaviorRule {
        condition: "company XP reaches 200",
        outcome: "the rank becomes Leader",
    },
    CompanyRanksBehaviorRule {
        condition: "company XP reaches 500",
        outcome: "the rank becomes Boss",
    },
    CompanyRanksBehaviorRule {
        condition: "company XP surpasses 1500",
        outcome: "the rank resets to Intern and the ladder must be climbed again",
    },
];

pub const COMPANY_RANKS_MODIFIERS: [CompanyRanksBehaviorRule; 3] = [
    CompanyRanksBehaviorRule {
        condition: "a mission earns a higher grade",
        outcome: "XP is awarded and promotion progress increases",
    },
    CompanyRanksBehaviorRule {
        condition: "a mission earns a poor grade",
        outcome: "XP is deducted and demotion risk increases",
    },
    CompanyRanksBehaviorRule {
        condition: "a mission fails",
        outcome: "XP is deducted and demotion risk increases",
    },
];

pub const COMPANY_RANKS_NOTES: [CompanyRanksBehaviorRule; 1] = [CompanyRanksBehaviorRule {
    condition: "the rank is entry level",
    outcome: "the title is Intern",
}];

pub const COMPANY_RANKS_BEHAVIORAL_MECHANICS: [CompanyRanksBehaviorRule; 9] = [
    CompanyRanksBehaviorRule {
        condition: "a mission earns a higher grade",
        outcome: "more XP is awarded and promotion progress increases",
    },
    CompanyRanksBehaviorRule {
        condition: "a mission earns a poor grade",
        outcome: "XP is deducted and demotion risk increases",
    },
    CompanyRanksBehaviorRule {
        condition: "a mission fails",
        outcome: "XP is deducted and demotion risk increases",
    },
    CompanyRanksBehaviorRule {
        condition: "company XP has not reached 50",
        outcome: "the rank is Intern",
    },
    CompanyRanksBehaviorRule {
        condition: "company XP reaches 50",
        outcome: "the rank becomes Part-Timer",
    },
    CompanyRanksBehaviorRule {
        condition: "company XP reaches 100",
        outcome: "the rank becomes Employee",
    },
    CompanyRanksBehaviorRule {
        condition: "company XP reaches 200",
        outcome: "the rank becomes Leader",
    },
    CompanyRanksBehaviorRule {
        condition: "company XP reaches 500",
        outcome: "the rank becomes Boss",
    },
    CompanyRanksBehaviorRule {
        condition: "company XP surpasses 1500",
        outcome: "the rank resets to Intern and the rank ladder must be repeated",
    },
];

pub struct CompanyRanksPlugin;

impl Plugin for CompanyRanksPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CompanyRanksState>()
            .add_event::<CompanyRankXpChangedEvent>()
            .add_event::<CompanyRankChangedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    company_ranks_apply_xp_changes,
                    company_ranks_refresh_rank,
                    company_ranks_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompanyRanksBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CompanyRank {
    #[default]
    Intern,
    PartTimer,
    Employee,
    Leader,
    Boss,
}

impl CompanyRank {
    pub fn title(self) -> &'static str {
        match self {
            Self::Intern => "Intern",
            Self::PartTimer => "Part-Timer",
            Self::Employee => "Employee",
            Self::Leader => "Leader",
            Self::Boss => "Boss",
        }
    }

    pub fn threshold_xp(self) -> i32 {
        match self {
            Self::Intern => COMPANY_RANKS_INTERN_XP,
            Self::PartTimer => COMPANY_RANKS_PART_TIMER_XP,
            Self::Employee => COMPANY_RANKS_EMPLOYEE_XP,
            Self::Leader => COMPANY_RANKS_LEADER_XP,
            Self::Boss => COMPANY_RANKS_BOSS_XP,
        }
    }

    pub fn checksum_code(self) -> u64 {
        match self {
            Self::Intern => 0,
            Self::PartTimer => 1,
            Self::Employee => 2,
            Self::Leader => 3,
            Self::Boss => 4,
        }
    }
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct CompanyRanksState {
    pub company_xp: i32,
    pub current_rank: CompanyRank,
    pub xp_changes_applied: u64,
    pub rank_changes: u64,
    pub ladder_resets: u64,
    pub promotion_progress_increases: u64,
    pub demotion_risk_increases: u64,
    pub last_change_context_id: u64,
}

impl Default for CompanyRanksState {
    fn default() -> Self {
        Self {
            company_xp: 0,
            current_rank: CompanyRank::Intern,
            xp_changes_applied: 0,
            rank_changes: 0,
            ladder_resets: 0,
            promotion_progress_increases: 0,
            demotion_risk_increases: 0,
            last_change_context_id: 0,
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompanyRankXpChangedEvent {
    pub context_id: u64,
    pub xp_delta: i32,
    pub mission_result: CompanyRankMissionResult,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompanyRankChangedEvent {
    pub context_id: u64,
    pub previous_rank: CompanyRank,
    pub new_rank: CompanyRank,
    pub company_xp: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompanyRankMissionResult {
    HigherGrade,
    PoorGrade,
    Failed,
}

pub fn company_rank_for_xp(company_xp: i32) -> CompanyRank {
    if company_xp >= COMPANY_RANKS_BOSS_XP {
        CompanyRank::Boss
    } else if company_xp >= COMPANY_RANKS_LEADER_XP {
        CompanyRank::Leader
    } else if company_xp >= COMPANY_RANKS_EMPLOYEE_XP {
        CompanyRank::Employee
    } else if company_xp >= COMPANY_RANKS_PART_TIMER_XP {
        CompanyRank::PartTimer
    } else {
        CompanyRank::Intern
    }
}

pub fn company_rank_title_for_xp(company_xp: i32) -> &'static str {
    company_rank_for_xp(company_xp).title()
}

fn company_ranks_apply_xp_changes(
    mut xp_events: EventReader<CompanyRankXpChangedEvent>,
    mut state: ResMut<CompanyRanksState>,
) {
    for event in xp_events.read() {
        state.company_xp = state.company_xp.saturating_add(event.xp_delta);
        if state.company_xp < 0 {
            state.company_xp = 0;
        }

        state.xp_changes_applied = state.xp_changes_applied.wrapping_add(1);
        state.last_change_context_id = event.context_id;

        match event.mission_result {
            CompanyRankMissionResult::HigherGrade => {
                state.promotion_progress_increases =
                    state.promotion_progress_increases.wrapping_add(1);
            }
            CompanyRankMissionResult::PoorGrade | CompanyRankMissionResult::Failed => {
                state.demotion_risk_increases = state.demotion_risk_increases.wrapping_add(1);
            }
        }

        if state.company_xp > COMPANY_RANKS_RESET_XP {
            state.company_xp = 0;
            state.ladder_resets = state.ladder_resets.wrapping_add(1);
        }
    }
}

fn company_ranks_refresh_rank(
    mut rank_events: EventWriter<CompanyRankChangedEvent>,
    mut state: ResMut<CompanyRanksState>,
) {
    let previous_rank = state.current_rank;
    let new_rank = company_rank_for_xp(state.company_xp);

    if previous_rank != new_rank {
        state.current_rank = new_rank;
        state.rank_changes = state.rank_changes.wrapping_add(1);

        rank_events.send(CompanyRankChangedEvent {
            context_id: state.last_change_context_id,
            previous_rank,
            new_rank,
            company_xp: state.company_xp,
        });
    }
}

fn company_ranks_checksum(
    mut checksum: ResMut<SimChecksumState>,
    tick: Res<SimTick>,
    state: Res<CompanyRanksState>,
) {
    checksum.accumulate(tick.0);
    checksum.accumulate(COMPANY_RANKS_SOURCE_REVISION as u64);
    checksum.accumulate(COMPANY_RANKS_CONFIDENCE_PERCENT as u64);
    checksum.accumulate(COMPANY_RANKS_INTERN_XP as u64);
    checksum.accumulate(COMPANY_RANKS_PART_TIMER_XP as u64);
    checksum.accumulate(COMPANY_RANKS_EMPLOYEE_XP as u64);
    checksum.accumulate(COMPANY_RANKS_LEADER_XP as u64);
    checksum.accumulate(COMPANY_RANKS_BOSS_XP as u64);
    checksum.accumulate(COMPANY_RANKS_RESET_XP as u64);

    for dependency in COMPANY_RANKS_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x1000, dependency);
    }

    for rule in COMPANY_RANKS_RULES {
        accumulate_str(&mut checksum, 0x2000, rule.condition);
        accumulate_str(&mut checksum, 0x2001, rule.outcome);
    }

    for modifier in COMPANY_RANKS_MODIFIERS {
        accumulate_str(&mut checksum, 0x3000, modifier.condition);
        accumulate_str(&mut checksum, 0x3001, modifier.outcome);
    }

    for note in COMPANY_RANKS_NOTES {
        accumulate_str(&mut checksum, 0x4000, note.condition);
        accumulate_str(&mut checksum, 0x4001, note.outcome);
    }

    for mechanic in COMPANY_RANKS_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x5000, mechanic.condition);
        accumulate_str(&mut checksum, 0x5001, mechanic.outcome);
    }

    checksum.accumulate(state.company_xp as u64);
    checksum.accumulate(state.current_rank.checksum_code());
    checksum.accumulate(state.xp_changes_applied);
    checksum.accumulate(state.rank_changes);
    checksum.accumulate(state.ladder_resets);
    checksum.accumulate(state.promotion_progress_increases);
    checksum.accumulate(state.demotion_risk_increases);
    checksum.accumulate(state.last_change_context_id);
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt ^ value.len() as u64);

    for (index, byte) in value.bytes().enumerate() {
        checksum.accumulate(salt ^ ((index as u64) << 8) ^ byte as u64);
    }
}