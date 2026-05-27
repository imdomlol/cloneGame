// Sources: vault/game_mechanics/animation_canceling.md, vault/game_mechanics/noise.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{SimChecksumState, SimHz};

const ONE_THOUSAND_MS: I32F32 = I32F32::lit("1000");
const ONE_MS: I32F32 = I32F32::lit("1");
const ZERO_MS: I32F32 = I32F32::ZERO;

// Vault timing examples for ranger animation cancel windows.
const RANGER_ANIMATION_TOTAL_MS: I32F32 = I32F32::lit("1000");
const RANGER_PROJECTILE_RELEASE_MS: I32F32 = I32F32::lit("300");
const RANGER_SKIPPABLE_MS: I32F32 = I32F32::lit("700");

#[derive(Component, Clone, Copy, Default)]
pub struct AnimationCancelCapable;

#[derive(Component, Clone, Copy, Default)]
pub struct AttackTimingProfile {
    pub time_pre_action_ms: I32F32,
    pub time_action_ms: I32F32,
    pub time_post_action_ms: I32F32,
    pub time_loop_action_ms: Option<I32F32>,
    pub time_load_ms: Option<I32F32>,
    pub time_unload_ms: Option<I32F32>,
}

#[derive(Component, Clone, Copy, Default)]
pub struct AttackCycleState {
    pub action_elapsed_ms: I32F32,
    pub projectile_released: bool,
    pub skipped_animation_ms_total: I32F32,
    pub cancel_count: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationCancelTechnique {
    ShiftAttacking,
    HoldStopping,
}

#[derive(Event, Clone, Copy)]
pub struct AnimationCancelIntentEvent {
    pub entity: Entity,
    pub technique: AnimationCancelTechnique,
    pub rapid_input_loop: bool,
}

#[derive(Event, Clone, Copy)]
pub struct AnimationCancelAppliedEvent {
    pub entity: Entity,
    pub skipped_ms: I32F32,
    pub displayed_attacks_per_second: I32F32,
    pub bonus_noise_activity: I32F32,
}

#[derive(Resource, Clone, Copy)]
pub struct AnimationCancelMechanicData {
    pub id: &'static str,
    pub name: &'static str,
    pub mechanic_type: &'static str,
    pub how_it_works: &'static str,
    pub techniques: &'static str,
    pub applications: &'static str,
    pub depends_on: &'static [&'static str],
}

impl Default for AnimationCancelMechanicData {
    fn default() -> Self {
        Self {
            id: "animation_canceling",
            name: "Animation canceling",
            mechanic_type: "animation_canceling",
            how_it_works: "Interrupt the attack animation after the projectile is released so the remaining animation frames are skipped and the next action can begin sooner.",
            techniques: "Shift-attacking and hold-stopping are the two manual methods for forcing the interruption window.",
            applications: "Manual unit micro for higher damage per second, especially on units with short release windows or looping attack cycles.",
            depends_on: &[
                "noise",
                "ranger",
                "caelus",
                "calliope",
                "lucifer",
                "infected_venom",
                "soldier",
                "titan",
            ],
        }
    }
}

#[derive(Resource, Clone, Copy)]
pub struct AnimationCancelMetrics {
    pub total_manual_cancels: u64,
    pub total_skipped_ms: I32F32,
    pub total_bonus_noise_activity: I32F32,
    pub ranger_window_reference_ms: I32F32,
}

impl Default for AnimationCancelMetrics {
    fn default() -> Self {
        Self {
            total_manual_cancels: 0,
            total_skipped_ms: ZERO_MS,
            total_bonus_noise_activity: ZERO_MS,
            ranger_window_reference_ms: RANGER_SKIPPABLE_MS,
        }
    }
}

fn displayed_attacks_per_second(profile: &AttackTimingProfile) -> I32F32 {
    if let Some(loop_ms) = profile.time_loop_action_ms {
        if loop_ms > ZERO_MS {
            return ONE_THOUSAND_MS / loop_ms;
        }
        return ZERO_MS;
    }

    let cycle_ms = profile.time_pre_action_ms + profile.time_action_ms + profile.time_post_action_ms;
    if cycle_ms > ZERO_MS {
        ONE_THOUSAND_MS / cycle_ms
    } else {
        ZERO_MS
    }
}

fn apply_animation_cancel_intents_system(
    sim_hz: Res<SimHz>,
    mut metrics: ResMut<AnimationCancelMetrics>,
    mut intents: EventReader<AnimationCancelIntentEvent>,
    mut applied_writer: EventWriter<AnimationCancelAppliedEvent>,
    mut units: Query<(&AttackTimingProfile, &mut AttackCycleState), With<AnimationCancelCapable>>,
) {
    for ev in intents.read() {
        let Ok((profile, mut cycle)) = units.get_mut(ev.entity) else {
            continue;
        };

        if !cycle.projectile_released {
            continue;
        }

        match ev.technique {
            AnimationCancelTechnique::ShiftAttacking | AnimationCancelTechnique::HoldStopping => {}
        }

        let cycle_total_ms = if let Some(loop_ms) = profile.time_loop_action_ms {
            loop_ms
        } else {
            profile.time_pre_action_ms + profile.time_action_ms + profile.time_post_action_ms
        };

        let remaining_ms = if cycle_total_ms > cycle.action_elapsed_ms {
            cycle_total_ms - cycle.action_elapsed_ms
        } else {
            ZERO_MS
        };

        let per_frame_ms = if sim_hz.0 > I32F32::ZERO {
            ONE_THOUSAND_MS / sim_hz.0
        } else {
            ONE_MS
        };

        let skipped_ms = if profile.time_loop_action_ms.is_some() && ev.rapid_input_loop {
            if remaining_ms > per_frame_ms {
                remaining_ms - per_frame_ms
            } else {
                ZERO_MS
            }
        } else {
            remaining_ms
        };

        if skipped_ms <= ZERO_MS {
            continue;
        }

        cycle.action_elapsed_ms = ZERO_MS;
        cycle.projectile_released = false;
        cycle.skipped_animation_ms_total = cycle.skipped_animation_ms_total + skipped_ms;
        cycle.cancel_count = cycle.cancel_count.saturating_add(1);

        let aps = displayed_attacks_per_second(profile);

        // Animation canceling increases effective noise pressure by increasing attack cadence.
        let bonus_noise = if cycle_total_ms > ZERO_MS {
            skipped_ms / cycle_total_ms
        } else {
            ZERO_MS
        };

        metrics.total_manual_cancels = metrics.total_manual_cancels.saturating_add(1);
        metrics.total_skipped_ms = metrics.total_skipped_ms + skipped_ms;
        metrics.total_bonus_noise_activity = metrics.total_bonus_noise_activity + bonus_noise;

        applied_writer.send(AnimationCancelAppliedEvent {
            entity: ev.entity,
            skipped_ms,
            displayed_attacks_per_second: aps,
            bonus_noise_activity: bonus_noise,
        });
    }
}

fn animation_canceling_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    metrics: Res<AnimationCancelMetrics>,
    units: Query<(&AttackTimingProfile, &AttackCycleState), With<AnimationCancelCapable>>,
) {
    checksum.accumulate(metrics.total_manual_cancels);
    checksum.accumulate(metrics.total_skipped_ms.to_bits() as u64);
    checksum.accumulate(metrics.total_bonus_noise_activity.to_bits() as u64);
    checksum.accumulate(metrics.ranger_window_reference_ms.to_bits() as u64);

    checksum.accumulate(RANGER_ANIMATION_TOTAL_MS.to_bits() as u64);
    checksum.accumulate(RANGER_PROJECTILE_RELEASE_MS.to_bits() as u64);
    checksum.accumulate(RANGER_SKIPPABLE_MS.to_bits() as u64);

    for (profile, cycle) in &units {
        checksum.accumulate(profile.time_pre_action_ms.to_bits() as u64);
        checksum.accumulate(profile.time_action_ms.to_bits() as u64);
        checksum.accumulate(profile.time_post_action_ms.to_bits() as u64);

        if let Some(v) = profile.time_loop_action_ms {
            checksum.accumulate(v.to_bits() as u64);
        } else {
            checksum.accumulate(0);
        }

        if let Some(v) = profile.time_load_ms {
            checksum.accumulate(v.to_bits() as u64);
        } else {
            checksum.accumulate(0);
        }

        if let Some(v) = profile.time_unload_ms {
            checksum.accumulate(v.to_bits() as u64);
        } else {
            checksum.accumulate(0);
        }

        checksum.accumulate(cycle.action_elapsed_ms.to_bits() as u64);
        checksum.accumulate(u64::from(cycle.projectile_released));
        checksum.accumulate(cycle.skipped_animation_ms_total.to_bits() as u64);
        checksum.accumulate(cycle.cancel_count as u64);
    }
}

pub struct AnimationCancelingPlugin;

impl Plugin for AnimationCancelingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AnimationCancelMechanicData>()
            .init_resource::<AnimationCancelMetrics>()
            .add_event::<AnimationCancelIntentEvent>()
            .add_event::<AnimationCancelAppliedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    apply_animation_cancel_intents_system,
                    animation_canceling_checksum_system,
                )
                    .chain(),
            );
    }
}