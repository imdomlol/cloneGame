// Sources: vault/units/thanatos.md, vault/buildings/engineering_center.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{Health, NoiseEmittedEvent, SimChecksumState, SimPosition, UnitStats};

const THANATOS_HP: I32F32 = I32F32::lit("250");
const THANATOS_MOVE_SPEED: I32F32 = I32F32::lit("1.8");
const THANATOS_ATTACK_RANGE: I32F32 = I32F32::lit("10");
const THANATOS_ATTACK_SPEED: I32F32 = I32F32::lit("0.33");
const THANATOS_ATTACK_DAMAGE: I32F32 = I32F32::lit("70");
const THANATOS_WATCH_RANGE: I32F32 = I32F32::lit("12");
const THANATOS_BASE_ARMOR_REDUCTION: I32F32 = I32F32::lit("0.25");

const THANATOS_MINIMUM_RANGE: I32F32 = I32F32::lit("3");
const THANATOS_RANGED_AOE_RADIUS: I32F32 = I32F32::lit("1.8");
const THANATOS_RANGED_DIRECT_IMPACT_DAMAGE: I32F32 = I32F32::lit("93");
const THANATOS_RANGED_DIRECT_DIAGONAL_DAMAGE: I32F32 = I32F32::lit("41");

const THANATOS_MELEE_RANGE: I32F32 = I32F32::lit("1.5");
const THANATOS_MELEE_ATTACK_SPEED: I32F32 = I32F32::lit("1");
const THANATOS_MELEE_ATTACK_DAMAGE: I32F32 = I32F32::lit("20");

const THANATOS_RANGED_NOISE: I32F32 = I32F32::lit("500");
const THANATOS_MELEE_NOISE: I32F32 = I32F32::lit("3");
const THANATOS_RESEARCH_COST_GOLD: I32F32 = I32F32::lit("2000");

#[derive(Component, Default)]
pub struct Thanatos;

#[derive(Component, Clone, Copy, Default)]
pub struct ReplicatedUnitId(pub u64);

#[derive(Component, Clone, Copy)]
pub struct ArmorReduction(pub I32F32);

impl Default for ArmorReduction {
    fn default() -> Self {
        Self(THANATOS_BASE_ARMOR_REDUCTION)
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct MinimumRange(pub I32F32);

#[derive(Component, Clone, Copy, Default)]
pub struct RangedAoeRadius(pub I32F32);

#[derive(Component, Clone, Copy, Default)]
pub struct RangedDirectImpactDamage(pub I32F32);

#[derive(Component, Clone, Copy, Default)]
pub struct RangedDirectDiagonalDamage(pub I32F32);

#[derive(Component, Clone, Copy, Default)]
pub struct RangedAttackNoise(pub I32F32);

#[derive(Component, Clone, Copy, Default)]
pub struct MeleeAttackNoise(pub I32F32);

#[derive(Component, Clone, Copy, Default)]
pub struct MeleeStats {
    pub attack_range: I32F32,
    pub attack_speed: I32F32,
    pub attack_damage: I32F32,
}

#[derive(Component, Clone, Copy, Default)]
pub struct HoldMode(pub bool);

#[derive(Component, Clone, Copy, Default)]
pub struct HoldModeDisablesMelee(pub bool);

#[derive(Component, Clone, Copy, Default)]
pub struct RetreatFromMinimumRange(pub bool);

#[derive(Component, Clone, Copy, Default)]
pub struct FoundryResearchRequired(pub bool);

#[derive(Component, Clone, Copy, Default)]
pub struct ResearchCostGold(pub I32F32);

#[derive(Component, Clone, Copy, Default)]
pub struct MeleeAoeIsCone120Degrees(pub bool);

#[derive(Component, Clone, Copy, Default)]
pub struct RangedAoeIsCircle(pub bool);

#[derive(Bundle)]
pub struct ThanatosBundle {
    pub unit: Thanatos,
    pub replicated_id: ReplicatedUnitId,
    pub position: SimPosition,
    pub health: Health,
    pub stats: UnitStats,
    pub armor_reduction: ArmorReduction,
    pub minimum_range: MinimumRange,
    pub ranged_aoe_radius: RangedAoeRadius,
    pub ranged_direct_impact_damage: RangedDirectImpactDamage,
    pub ranged_direct_diagonal_damage: RangedDirectDiagonalDamage,
    pub ranged_attack_noise: RangedAttackNoise,
    pub melee_attack_noise: MeleeAttackNoise,
    pub melee_stats: MeleeStats,
    pub hold_mode: HoldMode,
    pub hold_mode_disables_melee: HoldModeDisablesMelee,
    pub retreat_from_minimum_range: RetreatFromMinimumRange,
    pub foundry_research_required: FoundryResearchRequired,
    pub research_cost_gold: ResearchCostGold,
    pub melee_aoe_is_cone_120_degrees: MeleeAoeIsCone120Degrees,
    pub ranged_aoe_is_circle: RangedAoeIsCircle,
}

impl Default for ThanatosBundle {
    fn default() -> Self {
        Self {
            unit: Thanatos,
            replicated_id: ReplicatedUnitId::default(),
            position: SimPosition {
                x: I32F32::ZERO,
                y: I32F32::ZERO,
            },
            health: Health::full(THANATOS_HP),
            stats: UnitStats {
                move_speed: THANATOS_MOVE_SPEED,
                attack_range: THANATOS_ATTACK_RANGE,
                attack_damage: THANATOS_ATTACK_DAMAGE,
                attack_speed: THANATOS_ATTACK_SPEED,
                watch_range: THANATOS_WATCH_RANGE,
            },
            armor_reduction: ArmorReduction::default(),
            minimum_range: MinimumRange(THANATOS_MINIMUM_RANGE),
            ranged_aoe_radius: RangedAoeRadius(THANATOS_RANGED_AOE_RADIUS),
            ranged_direct_impact_damage: RangedDirectImpactDamage(THANATOS_RANGED_DIRECT_IMPACT_DAMAGE),
            ranged_direct_diagonal_damage: RangedDirectDiagonalDamage(
                THANATOS_RANGED_DIRECT_DIAGONAL_DAMAGE,
            ),
            ranged_attack_noise: RangedAttackNoise(THANATOS_RANGED_NOISE),
            melee_attack_noise: MeleeAttackNoise(THANATOS_MELEE_NOISE),
            melee_stats: MeleeStats {
                attack_range: THANATOS_MELEE_RANGE,
                attack_speed: THANATOS_MELEE_ATTACK_SPEED,
                attack_damage: THANATOS_MELEE_ATTACK_DAMAGE,
            },
            hold_mode: HoldMode::default(),
            hold_mode_disables_melee: HoldModeDisablesMelee(true),
            retreat_from_minimum_range: RetreatFromMinimumRange(true),
            foundry_research_required: FoundryResearchRequired(true),
            research_cost_gold: ResearchCostGold(THANATOS_RESEARCH_COST_GOLD),
            melee_aoe_is_cone_120_degrees: MeleeAoeIsCone120Degrees(true),
            ranged_aoe_is_circle: RangedAoeIsCircle(true),
        }
    }
}

#[derive(Resource, Default)]
pub struct NextReplicatedUnitId(pub u64);

#[derive(Event, Clone, Copy)]
pub struct SpawnThanatosEvent {
    pub position: SimPosition,
}

#[derive(Event, Clone, Copy)]
pub struct SetThanatosHoldModeEvent {
    pub entity: Entity,
    pub enabled: bool,
}

#[derive(Event, Clone, Copy)]
pub struct ThanatosFiredRangedEvent {
    pub entity: Entity,
}

#[derive(Event, Clone, Copy)]
pub struct ThanatosPerformedMeleeEvent {
    pub entity: Entity,
}

fn spawn_thanatos_system(
    mut commands: Commands,
    mut events: EventReader<SpawnThanatosEvent>,
    mut next_id: ResMut<NextReplicatedUnitId>,
) {
    for ev in events.read() {
        let mut bundle = ThanatosBundle::default();
        bundle.position = ev.position;
        bundle.replicated_id = ReplicatedUnitId(next_id.0);
        next_id.0 = next_id.0.wrapping_add(1);
        commands.spawn(bundle);
    }
}

fn set_thanatos_hold_mode_system(
    mut events: EventReader<SetThanatosHoldModeEvent>,
    mut units: Query<&mut HoldMode, With<Thanatos>>,
) {
    for ev in events.read() {
        let Ok(mut hold_mode) = units.get_mut(ev.entity) else {
            continue;
        };
        hold_mode.0 = ev.enabled;
    }
}

fn thanatos_ranged_noise_system(
    mut events: EventReader<ThanatosFiredRangedEvent>,
    units: Query<(&SimPosition, &RangedAttackNoise), With<Thanatos>>,
    mut noise_writer: EventWriter<NoiseEmittedEvent>,
) {
    for ev in events.read() {
        let Ok((position, noise)) = units.get(ev.entity) else {
            continue;
        };

        noise_writer.send(NoiseEmittedEvent {
            source: ev.entity,
            position: *position,
            amount: noise.0,
        });
    }
}

fn thanatos_melee_noise_system(
    mut events: EventReader<ThanatosPerformedMeleeEvent>,
    units: Query<(&SimPosition, &MeleeAttackNoise, &HoldMode, &HoldModeDisablesMelee), With<Thanatos>>,
    mut noise_writer: EventWriter<NoiseEmittedEvent>,
) {
    for ev in events.read() {
        let Ok((position, noise, hold_mode, hold_disables_melee)) = units.get(ev.entity) else {
            continue;
        };

        if hold_mode.0 && hold_disables_melee.0 {
            continue;
        }

        noise_writer.send(NoiseEmittedEvent {
            source: ev.entity,
            position: *position,
            amount: noise.0,
        });
    }
}

fn thanatos_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    core_units: Query<
        (
            &ReplicatedUnitId,
            &SimPosition,
            &Health,
            &UnitStats,
            &ArmorReduction,
            &MinimumRange,
            &RangedAoeRadius,
            &RangedDirectImpactDamage,
            &RangedDirectDiagonalDamage,
            &RangedAttackNoise,
            &MeleeAttackNoise,
        ),
        With<Thanatos>,
    >,
    extra_units: Query<
        (
            &MeleeStats,
            &HoldMode,
            &HoldModeDisablesMelee,
            &RetreatFromMinimumRange,
            &FoundryResearchRequired,
            &ResearchCostGold,
            &MeleeAoeIsCone120Degrees,
            &RangedAoeIsCircle,
        ),
        With<Thanatos>,
    >,
) {
    for (
        replicated_id,
        pos,
        hp,
        stats,
        armor,
        minimum_range,
        ranged_aoe_radius,
        ranged_impact_damage,
        ranged_diagonal_damage,
        ranged_noise,
        melee_noise,
    ) in &core_units
    {
        checksum.accumulate(replicated_id.0);
        checksum.accumulate(pos.x.to_bits() as u64);
        checksum.accumulate(pos.y.to_bits() as u64);
        checksum.accumulate(hp.current.to_bits() as u64);
        checksum.accumulate(hp.max.to_bits() as u64);
        checksum.accumulate(stats.move_speed.to_bits() as u64);
        checksum.accumulate(stats.attack_range.to_bits() as u64);
        checksum.accumulate(stats.attack_damage.to_bits() as u64);
        checksum.accumulate(stats.attack_speed.to_bits() as u64);
        checksum.accumulate(stats.watch_range.to_bits() as u64);
        checksum.accumulate(armor.0.to_bits() as u64);
        checksum.accumulate(minimum_range.0.to_bits() as u64);
        checksum.accumulate(ranged_aoe_radius.0.to_bits() as u64);
        checksum.accumulate(ranged_impact_damage.0.to_bits() as u64);
        checksum.accumulate(ranged_diagonal_damage.0.to_bits() as u64);
        checksum.accumulate(ranged_noise.0.to_bits() as u64);
        checksum.accumulate(melee_noise.0.to_bits() as u64);
    }

    for (
        melee_stats,
        hold_mode,
        hold_disables_melee,
        retreat_minimum_range,
        research_required,
        research_cost_gold,
        melee_aoe_cone,
        ranged_aoe_circle,
    ) in &extra_units
    {
        checksum.accumulate(melee_stats.attack_range.to_bits() as u64);
        checksum.accumulate(melee_stats.attack_speed.to_bits() as u64);
        checksum.accumulate(melee_stats.attack_damage.to_bits() as u64);
        checksum.accumulate(u64::from(hold_mode.0));
        checksum.accumulate(u64::from(hold_disables_melee.0));
        checksum.accumulate(u64::from(retreat_minimum_range.0));
        checksum.accumulate(u64::from(research_required.0));
        checksum.accumulate(research_cost_gold.0.to_bits() as u64);
        checksum.accumulate(u64::from(melee_aoe_cone.0));
        checksum.accumulate(u64::from(ranged_aoe_circle.0));
    }
}

pub struct ThanatosPlugin;

impl Plugin for ThanatosPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NextReplicatedUnitId>()
            .add_event::<SpawnThanatosEvent>()
            .add_event::<SetThanatosHoldModeEvent>()
            .add_event::<ThanatosFiredRangedEvent>()
            .add_event::<ThanatosPerformedMeleeEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_thanatos_system,
                    set_thanatos_hold_mode_system,
                    thanatos_ranged_noise_system,
                    thanatos_melee_noise_system,
                    thanatos_checksum_system,
                )
                    .chain(),
            );
    }
}