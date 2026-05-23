// Sources: vault/unit/soldier.md

//! Soldier — Empire primary infantry, produced at soldiers_center.

use bevy::prelude::*;
use fixed::types::I32F32;
use rand_chacha::ChaCha8Rng;
use rand_core::{RngCore, SeedableRng};

use crate::sim::{
    DamageType, EntityKilledEvent, GameSeed, IncomingDamageEvent, NoiseEmittedEvent, SimHz,
    SimPosition, SimTick,
};
use crate::units::Infected;

// ── Frontmatter constants (vault/unit/soldier.md) ────────────────────────────

const SOLDIER_HP: I32F32             = I32F32::lit("120");
const SOLDIER_MS: I32F32             = I32F32::lit("2.4");
const SOLDIER_AR: I32F32             = I32F32::lit("5");
const SOLDIER_AS: I32F32             = I32F32::lit("2");
const SOLDIER_AD: I32F32             = I32F32::lit("16");
const SOLDIER_WR: I32F32             = I32F32::lit("6");
const SOLDIER_NOISE_ON_FIRE: I32F32  = I32F32::lit("3");
const SOLDIER_ARMOR_STD: I32F32      = I32F32::lit("0.4");  // 40% standard damage reduction
const SOLDIER_ARMOR_FIRE: I32F32     = I32F32::lit("0.5");  // 50% fire resistance
const SOLDIER_ARMOR_VENOM: I32F32    = I32F32::lit("0.5");  // 50% venom resistance

// Veteran stats (arv / asv / adv)
const SOLDIER_AR_VET: I32F32         = I32F32::lit("5.5");
const SOLDIER_AS_VET: I32F32         = I32F32::lit("2.5");
const SOLDIER_AD_VET: I32F32         = I32F32::lit("26");

// Build cost
const SOLDIER_GOLD_COST: i32         = 240;
const SOLDIER_BUILD_TIME: i32        = 30;   // seconds; caller converts to ticks
const SOLDIER_FOOD_UPKEEP: i32       = 1;
const SOLDIER_WORKER_SLOTS: i32      = 1;
const SOLDIER_IRON_COST: i32         = 2;
const SOLDIER_GOLD_MAINTENANCE: i32  = 3;

// Experience
const SOLDIER_EXP_REWARD: I32F32     = I32F32::lit("90");

// Gain rates per kill by difficulty tier (behavioral mechanic #8).
const SOLDIER_EXP_GAIN_BASIC: I32F32  = I32F32::lit("0.0134");  // 1.34%
const SOLDIER_EXP_GAIN_MEDIUM: I32F32 = I32F32::lit("0.0268");  // 2.68%
const SOLDIER_EXP_GAIN_HARD: I32F32   = I32F32::lit("0.067");   // 6.7%
const SOLDIER_EXP_GAIN_ELITE: I32F32  = I32F32::lit("0.134");   // 13.4%

// Not specified in vault/unit/soldier.md; engine constant.
const SOLDIER_VETERAN_THRESHOLD: I32F32 = I32F32::lit("100");

// ── Components ────────────────────────────────────────────────────────────────

/// Marks an entity as a Soldier unit (Empire primary infantry).
#[derive(Component)]
pub struct Soldier;

/// Marks a Soldier that has reached veteran status.
#[derive(Component)]
pub struct Veteran;

/// Fixed-point HP state for a sim entity.
#[derive(Component, Clone, Copy)]
pub struct SimHealth {
    pub current: I32F32,
    pub max: I32F32,
}

impl SimHealth {
    pub fn full(max: I32F32) -> Self {
        Self { current: max, max }
    }

    pub fn is_dead(self) -> bool {
        self.current <= I32F32::ZERO
    }
}

/// Ticks until this unit can fire again.
#[derive(Component)]
pub struct AttackCooldown {
    pub ticks_remaining: I32F32,
    pub ticks_per_attack: I32F32,
}

impl AttackCooldown {
    pub fn from_rate(attacks_per_second: I32F32, sim_hz: I32F32) -> Self {
        Self {
            ticks_remaining: I32F32::ZERO,
            ticks_per_attack: sim_hz / attacks_per_second,
        }
    }

    pub fn is_ready(&self) -> bool {
        self.ticks_remaining <= I32F32::ZERO
    }

    pub fn reset(&mut self) {
        self.ticks_remaining = self.ticks_per_attack;
    }

    pub fn tick(&mut self) {
        if self.ticks_remaining > I32F32::ZERO {
            self.ticks_remaining -= I32F32::ONE;
        }
    }
}

/// Combat stats for a unit. Updated on veteran promotion.
#[derive(Component, Clone, Copy)]
pub struct AttackStats {
    pub range: I32F32,
    pub speed: I32F32,       // attacks per second
    pub damage: I32F32,
    pub watch_range: I32F32,
}

/// Movement speed in sim tiles per second.
#[derive(Component, Clone, Copy)]
pub struct SimMovementSpeed(pub I32F32);

/// Noise emitted at the unit's current tile.
#[derive(Component, Clone, Copy)]
pub struct NoiseOutput(pub I32F32);

/// Experience reward granted to whoever kills this entity.
#[derive(Component, Clone, Copy)]
pub struct ExpReward(pub I32F32);

/// Per-damage-type armor for the Soldier.
#[derive(Component, Clone, Copy)]
pub struct SoldierArmor {
    pub standard: I32F32,
    pub fire: I32F32,
    pub venom: I32F32,
}

/// Accumulated kills experience tracking progress toward veteran status.
#[derive(Component)]
pub struct SoldierExperience {
    pub accumulated: I32F32,
}

/// Immutable build-cost record for upkeep deduction and dismissal refunds.
#[derive(Component, Clone, Copy)]
pub struct SoldierBuildCost {
    pub gold: i32,
    pub iron: i32,
    pub food: i32,
    pub workers: i32,
    pub build_time_seconds: i32,
    pub gold_maintenance: i32,
}

// ── Events ────────────────────────────────────────────────────────────────────

/// Emitted each time a Soldier fires one shot.
#[derive(Event, Debug, Clone, Copy)]
pub struct SoldierFiredEvent {
    pub entity: Entity,
}

// ── Plugin ────────────────────────────────────────────────────────────────────

pub struct SoldierPlugin;

impl Plugin for SoldierPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SoldierFiredEvent>().add_systems(
            FixedUpdate,
            (
                soldier_cooldown_system,
                soldier_attack_system,
                soldier_fire_noise_system,
                soldier_damage_receive_system,
                soldier_experience_system,
                soldier_veteran_promotion_system,
            )
                .chain(),
        );
    }
}

// ── Systems ───────────────────────────────────────────────────────────────────

/// Decrements each Soldier's attack cooldown by one tick.
fn soldier_cooldown_system(mut query: Query<&mut AttackCooldown, With<Soldier>>) {
    for mut cooldown in &mut query {
        cooldown.tick();
    }
}

/// Fires at the nearest Infected target within attack range.
///
/// Tie-breaking among equidistant targets uses a per-tick ChaCha8Rng seeded
/// from `(game_seed XOR tick)` to satisfy the determinism contract.
fn soldier_attack_system(
    mut fired_events: EventWriter<SoldierFiredEvent>,
    mut damage_events: EventWriter<IncomingDamageEvent>,
    mut soldiers: Query<(Entity, &SimPosition, &AttackStats, &mut AttackCooldown), With<Soldier>>,
    targets: Query<(Entity, &SimPosition, &SimHealth), With<Infected>>,
    tick: Res<SimTick>,
    seed: Res<GameSeed>,
) {
    for (soldier_entity, soldier_pos, stats, mut cooldown) in &mut soldiers {
        if !cooldown.is_ready() {
            continue;
        }

        let range_sq = stats.range * stats.range;

        let mut in_range: Vec<(Entity, I32F32)> = targets
            .iter()
            .filter_map(|(entity, pos, health)| {
                if health.is_dead() {
                    return None;
                }
                let dx = pos.x - soldier_pos.x;
                let dy = pos.y - soldier_pos.y;
                let dist_sq = dx * dx + dy * dy;
                (dist_sq <= range_sq).then_some((entity, dist_sq))
            })
            .collect();

        if in_range.is_empty() {
            continue;
        }

        // Sort ascending by distance-squared for deterministic prefix selection.
        in_range.sort_by(|a, b| a.1.cmp(&b.1));

        let target_entity = if in_range.len() == 1 || in_range[0].1 < in_range[1].1 {
            in_range[0].0
        } else {
            // Seeded RNG breaks ties among equidistant nearest targets.
            let tie_count = in_range.partition_point(|e| e.1 == in_range[0].1);
            let mut rng = ChaCha8Rng::seed_from_u64(seed.0 ^ tick.0);
            let idx = (rng.next_u64() as usize) % tie_count;
            in_range[idx].0
        };

        cooldown.reset();
        fired_events.send(SoldierFiredEvent { entity: soldier_entity });
        damage_events.send(IncomingDamageEvent {
            target: target_entity,
            raw_amount: stats.damage,
            damage_type: DamageType::Standard,
            source: soldier_entity,
        });
    }
}

/// Emits noise = 3 at the Soldier's tile each time it fires.
///
/// The villages_of_doom noise accumulation system consumes NoiseEmittedEvents
/// and is responsible for the trigger threshold (behavioral mechanic #2).
fn soldier_fire_noise_system(
    mut fired_events: EventReader<SoldierFiredEvent>,
    mut noise_events: EventWriter<NoiseEmittedEvent>,
    soldiers: Query<&SimPosition, With<Soldier>>,
) {
    for fired in fired_events.read() {
        if let Ok(pos) = soldiers.get(fired.entity) {
            noise_events.send(NoiseEmittedEvent {
                source: fired.entity,
                position: *pos,
                amount: SOLDIER_NOISE_ON_FIRE,
            });
        }
    }
}

/// Applies Soldier armor to incoming damage and emits a kill event on death.
///
/// - Standard damage:              40% reduction (SOLDIER_ARMOR_STD  = 0.4)
/// - Fire damage:                  50% reduction (SOLDIER_ARMOR_FIRE = 0.5)
/// - Venom damage (infected_venom): 50% reduction (SOLDIER_ARMOR_VENOM = 0.5)
fn soldier_damage_receive_system(
    mut damage_events: EventReader<IncomingDamageEvent>,
    mut kill_events: EventWriter<EntityKilledEvent>,
    mut soldiers: Query<(&SoldierArmor, &mut SimHealth, &ExpReward), With<Soldier>>,
) {
    for event in damage_events.read() {
        let Ok((armor, mut health, exp_reward)) = soldiers.get_mut(event.target) else {
            continue;
        };

        let absorption = match event.damage_type {
            DamageType::Standard => armor.standard,
            DamageType::Fire     => armor.fire,
            DamageType::Venom    => armor.venom,
        };
        health.current -= event.raw_amount * (I32F32::ONE - absorption);

        if health.is_dead() {
            kill_events.send(EntityKilledEvent {
                entity: event.target,
                killer: event.source,
                exp_reward: exp_reward.0,
                difficulty_tier: 0,
            });
        }
    }
}

/// Accumulates experience on the Soldier-killer for each EntityKilledEvent.
///
/// Gain rates by difficulty tier of the killed target (behavioral mechanic #8):
///   0 (basic)  → 1.34%   1 (medium) → 2.68%
///   2 (hard)   → 6.7%    3+ (elite) → 13.4%
fn soldier_experience_system(
    mut kill_events: EventReader<EntityKilledEvent>,
    mut soldiers: Query<&mut SoldierExperience, With<Soldier>>,
) {
    for event in kill_events.read() {
        let Ok(mut exp) = soldiers.get_mut(event.killer) else {
            continue;
        };
        let rate = match event.difficulty_tier {
            0 => SOLDIER_EXP_GAIN_BASIC,
            1 => SOLDIER_EXP_GAIN_MEDIUM,
            2 => SOLDIER_EXP_GAIN_HARD,
            _ => SOLDIER_EXP_GAIN_ELITE,
        };
        exp.accumulated += event.exp_reward * rate;
    }
}

/// Promotes a Soldier to veteran once accumulated experience reaches the threshold.
///
/// On promotion (behavioral mechanic #9):
///   damage → adv = 26   attack speed → asv = 2.5   range → arv = 5.5
fn soldier_veteran_promotion_system(
    mut commands: Commands,
    mut soldiers: Query<
        (Entity, &SoldierExperience, &mut AttackStats, &mut AttackCooldown),
        (With<Soldier>, Without<Veteran>),
    >,
    sim_hz: Res<SimHz>,
) {
    for (entity, exp, mut stats, mut cooldown) in &mut soldiers {
        if exp.accumulated < SOLDIER_VETERAN_THRESHOLD {
            continue;
        }
        stats.damage      = SOLDIER_AD_VET;
        stats.speed       = SOLDIER_AS_VET;
        stats.range       = SOLDIER_AR_VET;
        cooldown.ticks_per_attack = sim_hz.0 / SOLDIER_AS_VET;
        commands.entity(entity).insert(Veteran);
    }
}

// ── Spawn helper ──────────────────────────────────────────────────────────────

/// Spawns a Soldier entity at `position` and returns its `Entity` id.
pub fn spawn_soldier(commands: &mut Commands, position: SimPosition, sim_hz: I32F32) -> Entity {
    commands
        .spawn((
            Soldier,
            position,
            SimHealth::full(SOLDIER_HP),
            SimMovementSpeed(SOLDIER_MS),
            AttackStats {
                range:       SOLDIER_AR,
                speed:       SOLDIER_AS,
                damage:      SOLDIER_AD,
                watch_range: SOLDIER_WR,
            },
            AttackCooldown::from_rate(SOLDIER_AS, sim_hz),
            SoldierArmor {
                standard: SOLDIER_ARMOR_STD,
                fire:     SOLDIER_ARMOR_FIRE,
                venom:    SOLDIER_ARMOR_VENOM,
            },
            SoldierExperience { accumulated: I32F32::ZERO },
            NoiseOutput(I32F32::ZERO),
            ExpReward(SOLDIER_EXP_REWARD),
            SoldierBuildCost {
                gold:               SOLDIER_GOLD_COST,
                iron:               SOLDIER_IRON_COST,
                food:               SOLDIER_FOOD_UPKEEP,
                workers:            SOLDIER_WORKER_SLOTS,
                build_time_seconds: SOLDIER_BUILD_TIME,
                gold_maintenance:   SOLDIER_GOLD_MAINTENANCE,
            },
        ))
        .id()
}
