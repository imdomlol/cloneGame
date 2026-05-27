// Sources: vault/infected/infected.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{
    DamageType, EntityKilledEvent, Health, IncomingDamageEvent, NoiseEmittedEvent, SimChecksumState, SimHz,
    SimPosition, UnitStats,
};
use crate::units::Infected;

const INFECTED_BASE_HP: I32F32 = I32F32::lit("35");
const INFECTED_BASE_MOVE_SPEED: I32F32 = I32F32::lit("0.4");
const INFECTED_BASE_ATTACK_RANGE: I32F32 = I32F32::lit("0");
const INFECTED_BASE_ATTACK_SPEED: I32F32 = I32F32::lit("1");
const INFECTED_BASE_ATTACK_DAMAGE: I32F32 = I32F32::lit("6");
const INFECTED_BASE_WATCH_RANGE: I32F32 = I32F32::lit("5");
const INFECTED_BASE_NOISE: I32F32 = I32F32::lit("1");
const INFECTED_BASE_VENOM_RESISTANCE: I32F32 = I32F32::ONE;
const INFECTED_MIN_HP: I32F32 = I32F32::lit("35");
const INFECTED_MAX_HP: I32F32 = I32F32::lit("20000");
const INFECTED_MIN_ATTACK_DAMAGE: I32F32 = I32F32::lit("6");
const INFECTED_MAX_ATTACK_DAMAGE: I32F32 = I32F32::lit("160");

#[derive(Component, Clone, Copy, Default)]
pub struct InfectedUnit;

#[derive(Component, Clone, Copy, Default)]
pub struct ReplicatedUnitId(pub u64);

#[derive(Component, Clone, Copy, Default)]
pub struct ArmorReduction(pub I32F32);

#[derive(Component, Clone, Copy, Default)]
pub struct RegenerationPerSecond(pub I32F32);

#[derive(Component, Clone, Copy, Default)]
pub struct NoiseGeneration(pub I32F32);

#[derive(Component, Clone, Copy, Default)]
pub struct VenomResistance(pub I32F32);

#[derive(Component, Clone, Copy, Default)]
pub struct ExperienceReward(pub I32F32);

#[derive(Component, Clone, Copy, Default)]
pub struct UsesSightOnlyAggro(pub bool);

#[derive(Component, Clone, Copy, Default)]
pub struct CanInfectBuildings(pub bool);

#[derive(Component, Clone, Copy, Default)]
pub struct UsesFriendlyFire(pub bool);

#[derive(Component, Clone, Copy, Default)]
pub struct HpBounds {
    pub min: I32F32,
    pub max: I32F32,
}

#[derive(Component, Clone, Copy, Default)]
pub struct DamageBounds {
    pub min: I32F32,
    pub max: I32F32,
}

#[derive(Component, Clone, Copy, Default)]
pub enum InfectedAttackType {
    #[default]
    Melee,
    Ranged,
}

#[derive(Component, Clone, Copy, Default)]
pub enum InfectedAoeProfile {
    #[default]
    None,
    RangedAoe,
    LargeAoe,
    VeryLargeAoe,
}

#[derive(Component, Clone, Copy, Default)]
pub enum InfectedVariant {
    #[default]
    Walkers,
    Fresh,
    Colonist,
    Executive,
    Harpy,
    Venom,
    Chubby,
    Giant,
    Behemoth,
}

#[derive(Bundle)]
pub struct InfectedBundle {
    pub infected_faction: Infected,
    pub unit: InfectedUnit,
    pub replicated_id: ReplicatedUnitId,
    pub position: SimPosition,
    pub health: Health,
    pub stats: UnitStats,
    pub armor_reduction: ArmorReduction,
    pub regeneration_per_second: RegenerationPerSecond,
    pub noise_generation: NoiseGeneration,
    pub venom_resistance: VenomResistance,
    pub experience_reward: ExperienceReward,
    pub uses_sight_only_aggro: UsesSightOnlyAggro,
    pub can_infect_buildings: CanInfectBuildings,
    pub uses_friendly_fire: UsesFriendlyFire,
    pub hp_bounds: HpBounds,
    pub damage_bounds: DamageBounds,
    pub attack_type: InfectedAttackType,
    pub aoe_profile: InfectedAoeProfile,
    pub variant: InfectedVariant,
}

impl Default for InfectedBundle {
    fn default() -> Self {
        Self {
            infected_faction: Infected,
            unit: InfectedUnit,
            replicated_id: ReplicatedUnitId::default(),
            position: SimPosition {
                x: I32F32::ZERO,
                y: I32F32::ZERO,
            },
            health: Health::full(INFECTED_BASE_HP),
            stats: UnitStats {
                move_speed: INFECTED_BASE_MOVE_SPEED,
                attack_range: INFECTED_BASE_ATTACK_RANGE,
                attack_damage: INFECTED_BASE_ATTACK_DAMAGE,
                attack_speed: INFECTED_BASE_ATTACK_SPEED,
                watch_range: INFECTED_BASE_WATCH_RANGE,
            },
            armor_reduction: ArmorReduction(I32F32::ZERO),
            regeneration_per_second: RegenerationPerSecond(I32F32::lit("3")),
            noise_generation: NoiseGeneration(INFECTED_BASE_NOISE),
            venom_resistance: VenomResistance(INFECTED_BASE_VENOM_RESISTANCE),
            experience_reward: ExperienceReward(I32F32::lit("1")),
            uses_sight_only_aggro: UsesSightOnlyAggro(false),
            can_infect_buildings: CanInfectBuildings(true),
            uses_friendly_fire: UsesFriendlyFire(false),
            hp_bounds: HpBounds {
                min: INFECTED_MIN_HP,
                max: INFECTED_MAX_HP,
            },
            damage_bounds: DamageBounds {
                min: INFECTED_MIN_ATTACK_DAMAGE,
                max: INFECTED_MAX_ATTACK_DAMAGE,
            },
            attack_type: InfectedAttackType::Melee,
            aoe_profile: InfectedAoeProfile::None,
            variant: InfectedVariant::Walkers,
        }
    }
}

#[derive(Resource, Default)]
pub struct NextReplicatedUnitId(pub u64);

#[derive(Event, Clone, Copy)]
pub struct SpawnInfectedEvent {
    pub position: SimPosition,
    pub variant: InfectedVariant,
}

#[derive(Event, Clone, Copy)]
pub struct InfectedAttackResolvedEvent {
    pub entity: Entity,
}

fn apply_variant_profile(bundle: &mut InfectedBundle, variant: InfectedVariant) {
    bundle.variant = variant;
    bundle.venom_resistance = VenomResistance(INFECTED_BASE_VENOM_RESISTANCE);
    bundle.uses_friendly_fire = UsesFriendlyFire(false);
    bundle.can_infect_buildings = CanInfectBuildings(true);
    bundle.uses_sight_only_aggro = UsesSightOnlyAggro(false);

    match variant {
        InfectedVariant::Walkers => {
            bundle.health = Health::full(I32F32::lit("35"));
            bundle.stats = UnitStats {
                move_speed: I32F32::lit("0.4"),
                attack_range: I32F32::lit("0.5"),
                attack_damage: I32F32::lit("6"),
                attack_speed: I32F32::lit("1"),
                watch_range: I32F32::lit("5"),
            };
            bundle.regeneration_per_second = RegenerationPerSecond(I32F32::lit("3"));
            bundle.armor_reduction = ArmorReduction(I32F32::ZERO);
            bundle.noise_generation = NoiseGeneration(I32F32::lit("1"));
            bundle.experience_reward = ExperienceReward(I32F32::lit("1"));
            bundle.attack_type = InfectedAttackType::Melee;
            bundle.aoe_profile = InfectedAoeProfile::None;
            bundle.hp_bounds = HpBounds {
                min: I32F32::lit("35"),
                max: I32F32::lit("35"),
            };
            bundle.damage_bounds = DamageBounds {
                min: I32F32::lit("6"),
                max: I32F32::lit("6"),
            };
        }
        InfectedVariant::Fresh => {
            bundle.health = Health::full(I32F32::lit("45"));
            bundle.stats = UnitStats {
                move_speed: I32F32::lit("1.75"),
                attack_range: I32F32::lit("0.6"),
                attack_damage: I32F32::lit("10"),
                attack_speed: I32F32::lit("1"),
                watch_range: I32F32::lit("6"),
            };
            bundle.regeneration_per_second = RegenerationPerSecond(I32F32::lit("4"));
            bundle.armor_reduction = ArmorReduction(I32F32::lit("0.05"));
            bundle.noise_generation = NoiseGeneration(I32F32::lit("2"));
            bundle.experience_reward = ExperienceReward(I32F32::lit("2"));
            bundle.attack_type = InfectedAttackType::Melee;
            bundle.aoe_profile = InfectedAoeProfile::None;
            bundle.hp_bounds = HpBounds {
                min: I32F32::lit("45"),
                max: I32F32::lit("45"),
            };
            bundle.damage_bounds = DamageBounds {
                min: I32F32::lit("10"),
                max: I32F32::lit("10"),
            };
        }
        InfectedVariant::Colonist => {
            bundle.health = Health::full(I32F32::lit("45"));
            bundle.stats = UnitStats {
                move_speed: I32F32::lit("1.75"),
                attack_range: I32F32::lit("0.6"),
                attack_damage: I32F32::lit("9"),
                attack_speed: I32F32::lit("1.11"),
                watch_range: I32F32::lit("6"),
            };
            bundle.regeneration_per_second = RegenerationPerSecond(I32F32::lit("4"));
            bundle.armor_reduction = ArmorReduction(I32F32::lit("0.05"));
            bundle.noise_generation = NoiseGeneration(I32F32::lit("2"));
            bundle.experience_reward = ExperienceReward(I32F32::lit("2"));
            bundle.attack_type = InfectedAttackType::Melee;
            bundle.aoe_profile = InfectedAoeProfile::None;
            bundle.hp_bounds = HpBounds {
                min: I32F32::lit("45"),
                max: I32F32::lit("45"),
            };
            bundle.damage_bounds = DamageBounds {
                min: I32F32::lit("9"),
                max: I32F32::lit("9"),
            };
        }
        InfectedVariant::Executive => {
            bundle.health = Health::full(I32F32::lit("80"));
            bundle.stats = UnitStats {
                move_speed: I32F32::lit("1.75"),
                attack_range: I32F32::lit("0.6"),
                attack_damage: I32F32::lit("12"),
                attack_speed: I32F32::lit("1.25"),
                watch_range: I32F32::lit("6"),
            };
            bundle.regeneration_per_second = RegenerationPerSecond(I32F32::lit("7"));
            bundle.armor_reduction = ArmorReduction(I32F32::lit("0.10"));
            bundle.noise_generation = NoiseGeneration(I32F32::lit("3"));
            bundle.experience_reward = ExperienceReward(I32F32::lit("5"));
            bundle.attack_type = InfectedAttackType::Melee;
            bundle.aoe_profile = InfectedAoeProfile::None;
            bundle.hp_bounds = HpBounds {
                min: I32F32::lit("80"),
                max: I32F32::lit("80"),
            };
            bundle.damage_bounds = DamageBounds {
                min: I32F32::lit("12"),
                max: I32F32::lit("12"),
            };
        }
        InfectedVariant::Harpy => {
            bundle.health = Health::full(I32F32::lit("120"));
            bundle.stats = UnitStats {
                move_speed: I32F32::lit("5"),
                attack_range: I32F32::lit("0.8"),
                attack_damage: I32F32::lit("30"),
                attack_speed: I32F32::lit("3.33"),
                watch_range: I32F32::lit("9"),
            };
            bundle.regeneration_per_second = RegenerationPerSecond(I32F32::lit("10"));
            bundle.armor_reduction = ArmorReduction(I32F32::lit("0.10"));
            bundle.noise_generation = NoiseGeneration(I32F32::lit("10"));
            bundle.experience_reward = ExperienceReward(I32F32::lit("10"));
            bundle.attack_type = InfectedAttackType::Melee;
            bundle.aoe_profile = InfectedAoeProfile::None;
            bundle.hp_bounds = HpBounds {
                min: I32F32::lit("120"),
                max: I32F32::lit("120"),
            };
            bundle.damage_bounds = DamageBounds {
                min: I32F32::lit("30"),
                max: I32F32::lit("30"),
            };
        }
        InfectedVariant::Venom => {
            bundle.health = Health::full(I32F32::lit("120"));
            bundle.stats = UnitStats {
                move_speed: I32F32::lit("1.75"),
                attack_range: I32F32::lit("4.5"),
                attack_damage: I32F32::lit("30"),
                attack_speed: I32F32::lit("0.5"),
                watch_range: I32F32::lit("8"),
            };
            bundle.regeneration_per_second = RegenerationPerSecond(I32F32::lit("10"));
            bundle.armor_reduction = ArmorReduction(I32F32::lit("0.10"));
            bundle.noise_generation = NoiseGeneration(I32F32::lit("10"));
            bundle.experience_reward = ExperienceReward(I32F32::lit("10"));
            bundle.attack_type = InfectedAttackType::Ranged;
            bundle.aoe_profile = InfectedAoeProfile::RangedAoe;
            bundle.hp_bounds = HpBounds {
                min: I32F32::lit("120"),
                max: I32F32::lit("120"),
            };
            bundle.damage_bounds = DamageBounds {
                min: I32F32::lit("30"),
                max: I32F32::lit("30"),
            };
        }
        InfectedVariant::Chubby => {
            bundle.health = Health::full(I32F32::lit("500"));
            bundle.stats = UnitStats {
                move_speed: I32F32::lit("1.75"),
                attack_range: I32F32::lit("0.7"),
                attack_damage: I32F32::lit("40"),
                attack_speed: I32F32::lit("1"),
                watch_range: I32F32::lit("5"),
            };
            bundle.regeneration_per_second = RegenerationPerSecond(I32F32::lit("20"));
            bundle.armor_reduction = ArmorReduction(I32F32::lit("0.15"));
            bundle.noise_generation = NoiseGeneration(I32F32::lit("10"));
            bundle.experience_reward = ExperienceReward(I32F32::lit("10"));
            bundle.attack_type = InfectedAttackType::Melee;
            bundle.aoe_profile = InfectedAoeProfile::None;
            bundle.hp_bounds = HpBounds {
                min: I32F32::lit("500"),
                max: I32F32::lit("500"),
            };
            bundle.damage_bounds = DamageBounds {
                min: I32F32::lit("40"),
                max: I32F32::lit("40"),
            };
        }
        InfectedVariant::Giant => {
            bundle.health = Health::full(I32F32::lit("20000"));
            bundle.stats = UnitStats {
                move_speed: I32F32::lit("4"),
                attack_range: I32F32::lit("3"),
                attack_damage: I32F32::lit("160"),
                attack_speed: I32F32::lit("1.25"),
                watch_range: I32F32::lit("10"),
            };
            bundle.regeneration_per_second = RegenerationPerSecond(I32F32::lit("50"));
            bundle.armor_reduction = ArmorReduction(I32F32::lit("0.30"));
            bundle.noise_generation = NoiseGeneration(I32F32::ZERO);
            bundle.experience_reward = ExperienceReward(I32F32::ZERO);
            bundle.attack_type = InfectedAttackType::Melee;
            bundle.aoe_profile = InfectedAoeProfile::VeryLargeAoe;
            bundle.uses_sight_only_aggro = UsesSightOnlyAggro(true);
            bundle.hp_bounds = HpBounds {
                min: I32F32::lit("4000"),
                max: I32F32::lit("20000"),
            };
            bundle.damage_bounds = DamageBounds {
                min: I32F32::lit("160"),
                max: I32F32::lit("160"),
            };
        }
        InfectedVariant::Behemoth => {
            bundle.health = Health::full(I32F32::lit("8000"));
            bundle.stats = UnitStats {
                move_speed: I32F32::lit("6"),
                attack_range: I32F32::lit("2"),
                attack_damage: I32F32::lit("30"),
                attack_speed: I32F32::lit("2"),
                watch_range: I32F32::lit("12"),
            };
            bundle.regeneration_per_second = RegenerationPerSecond(I32F32::lit("320"));
            bundle.armor_reduction = ArmorReduction(I32F32::lit("0.10"));
            bundle.noise_generation = NoiseGeneration(I32F32::lit("200"));
            bundle.experience_reward = ExperienceReward(I32F32::lit("50"));
            bundle.attack_type = InfectedAttackType::Melee;
            bundle.aoe_profile = InfectedAoeProfile::LargeAoe;
            bundle.hp_bounds = HpBounds {
                min: I32F32::lit("1500"),
                max: I32F32::lit("8000"),
            };
            bundle.damage_bounds = DamageBounds {
                min: I32F32::lit("30"),
                max: I32F32::lit("30"),
            };
        }
    }
}

fn spawn_infected_system(
    mut commands: Commands,
    mut events: EventReader<SpawnInfectedEvent>,
    mut next_id: ResMut<NextReplicatedUnitId>,
) {
    for ev in events.read() {
        let mut bundle = InfectedBundle::default();
        bundle.position = ev.position;
        bundle.replicated_id = ReplicatedUnitId(next_id.0);
        next_id.0 = next_id.0.wrapping_add(1);
        apply_variant_profile(&mut bundle, ev.variant);
        commands.spawn(bundle);
    }
}

fn infected_attack_noise_system(
    mut events: EventReader<InfectedAttackResolvedEvent>,
    infected_units: Query<(&SimPosition, &NoiseGeneration), With<InfectedUnit>>,
    mut noise_writer: EventWriter<NoiseEmittedEvent>,
) {
    for ev in events.read() {
        let Ok((position, noise)) = infected_units.get(ev.entity) else {
            continue;
        };

        if noise.0 <= I32F32::ZERO {
            continue;
        }

        noise_writer.send(NoiseEmittedEvent {
            source: ev.entity,
            position: *position,
            amount: noise.0,
        });
    }
}

fn infected_regeneration_system(
    sim_hz: Res<SimHz>,
    mut infected_units: Query<(&mut Health, &RegenerationPerSecond), With<InfectedUnit>>,
) {
    if sim_hz.0 <= I32F32::ZERO {
        return;
    }

    for (mut health, regen_per_second) in &mut infected_units {
        if health.current >= health.max {
            continue;
        }

        let regen_per_tick = regen_per_second.0 / sim_hz.0;
        let healed = health.current + regen_per_tick;
        health.current = if healed > health.max { health.max } else { healed };
    }
}

fn apply_infected_damage_system(
    mut damage_events: EventReader<IncomingDamageEvent>,
    mut infected_units: Query<(&mut Health, &ArmorReduction, &VenomResistance), With<InfectedUnit>>,
    infected_faction: Query<(), With<Infected>>,
    mut killed_writer: EventWriter<EntityKilledEvent>,
    rewards: Query<&ExperienceReward, With<InfectedUnit>>,
) {
    for ev in damage_events.read() {
        let Ok((mut health, armor_reduction, venom_resistance)) = infected_units.get_mut(ev.target) else {
            continue;
        };

        if health.current <= I32F32::ZERO {
            continue;
        }

        if infected_faction.get(ev.source).is_ok() {
            continue;
        }

        let mut multiplier = I32F32::ONE - armor_reduction.0;
        if ev.damage_type == DamageType::Venom {
            multiplier = multiplier * (I32F32::ONE - venom_resistance.0);
        }

        if multiplier <= I32F32::ZERO {
            continue;
        }

        let applied = ev.raw_amount * multiplier;
        if applied >= health.current {
            health.current = I32F32::ZERO;
            let exp_reward = rewards.get(ev.target).map_or(I32F32::ZERO, |r| r.0);
            killed_writer.send(EntityKilledEvent {
                entity: ev.target,
                killer: ev.source,
                exp_reward,
                difficulty_tier: 0,
            });
        } else {
            health.current = health.current - applied;
        }
    }
}

fn infected_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    infected_units: Query<
        (
            Entity,
            &ReplicatedUnitId,
            &SimPosition,
            &Health,
            &UnitStats,
            &ArmorReduction,
            &RegenerationPerSecond,
            &NoiseGeneration,
            &VenomResistance,
            &ExperienceReward,
            &UsesSightOnlyAggro,
            &CanInfectBuildings,
            &UsesFriendlyFire,
            &HpBounds,
            &DamageBounds,
        ),
        With<InfectedUnit>,
    >,
    infected_profiles: Query<(&InfectedAttackType, &InfectedAoeProfile, &InfectedVariant), With<InfectedUnit>>,
) {
    for (
        entity,
        replicated_id,
        pos,
        health,
        stats,
        armor_reduction,
        regen,
        noise_generation,
        venom_resistance,
        experience_reward,
        uses_sight_only_aggro,
        can_infect_buildings,
        uses_friendly_fire,
        hp_bounds,
        damage_bounds,
    ) in &infected_units
    {
        let Ok((attack_type, aoe_profile, variant)) = infected_profiles.get(entity) else {
            continue;
        };

        checksum.accumulate(replicated_id.0);
        checksum.accumulate(pos.x.to_bits() as u64);
        checksum.accumulate(pos.y.to_bits() as u64);
        checksum.accumulate(health.current.to_bits() as u64);
        checksum.accumulate(health.max.to_bits() as u64);
        checksum.accumulate(stats.move_speed.to_bits() as u64);
        checksum.accumulate(stats.attack_range.to_bits() as u64);
        checksum.accumulate(stats.attack_damage.to_bits() as u64);
        checksum.accumulate(stats.attack_speed.to_bits() as u64);
        checksum.accumulate(stats.watch_range.to_bits() as u64);
        checksum.accumulate(armor_reduction.0.to_bits() as u64);
        checksum.accumulate(regen.0.to_bits() as u64);
        checksum.accumulate(noise_generation.0.to_bits() as u64);
        checksum.accumulate(venom_resistance.0.to_bits() as u64);
        checksum.accumulate(experience_reward.0.to_bits() as u64);
        checksum.accumulate(u64::from(uses_sight_only_aggro.0));
        checksum.accumulate(u64::from(can_infect_buildings.0));
        checksum.accumulate(u64::from(uses_friendly_fire.0));
        checksum.accumulate(hp_bounds.min.to_bits() as u64);
        checksum.accumulate(hp_bounds.max.to_bits() as u64);
        checksum.accumulate(damage_bounds.min.to_bits() as u64);
        checksum.accumulate(damage_bounds.max.to_bits() as u64);

        let attack_type_bits = match attack_type {
            InfectedAttackType::Melee => 0_u64,
            InfectedAttackType::Ranged => 1_u64,
        };
        checksum.accumulate(attack_type_bits);

        let aoe_bits = match aoe_profile {
            InfectedAoeProfile::None => 0_u64,
            InfectedAoeProfile::RangedAoe => 1_u64,
            InfectedAoeProfile::LargeAoe => 2_u64,
            InfectedAoeProfile::VeryLargeAoe => 3_u64,
        };
        checksum.accumulate(aoe_bits);

        let variant_bits = match variant {
            InfectedVariant::Walkers => 0_u64,
            InfectedVariant::Fresh => 1_u64,
            InfectedVariant::Colonist => 2_u64,
            InfectedVariant::Executive => 3_u64,
            InfectedVariant::Harpy => 4_u64,
            InfectedVariant::Venom => 5_u64,
            InfectedVariant::Chubby => 6_u64,
            InfectedVariant::Giant => 7_u64,
            InfectedVariant::Behemoth => 8_u64,
        };
        checksum.accumulate(variant_bits);
    }
}

pub struct InfectedPlugin;

impl Plugin for InfectedPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NextReplicatedUnitId>()
            .add_event::<SpawnInfectedEvent>()
            .add_event::<InfectedAttackResolvedEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_infected_system,
                    infected_attack_noise_system,
                    infected_regeneration_system,
                    apply_infected_damage_system,
                    infected_checksum_system,
                )
                    .chain(),
            );
    }
}