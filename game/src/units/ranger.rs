// Sources: vault/units/ranger.md, vault/buildings/soldiers_center.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{Health, SimChecksumState, SimPosition, UnitStats};

const RANGER_HP: I32F32 = I32F32::lit("60");
const RANGER_MOVE_SPEED: I32F32 = I32F32::lit("4");
const RANGER_ATTACK_RANGE: I32F32 = I32F32::lit("6");
const RANGER_ATTACK_SPEED: I32F32 = I32F32::lit("1");
const RANGER_ATTACK_DAMAGE: I32F32 = I32F32::lit("10");
const RANGER_WATCH_RANGE: I32F32 = I32F32::lit("8");
const RANGER_BASE_ARMOR_REDUCTION: I32F32 = I32F32::lit("0.05");
const RANGER_BASE_ATTACK_NOISE: I32F32 = I32F32::lit("1");
const RANGER_VETERAN_ATTACK_RANGE: I32F32 = I32F32::lit("6.5");
const RANGER_VETERAN_ATTACK_SPEED: I32F32 = I32F32::lit("2");
const RANGER_VETERAN_ATTACK_DAMAGE: I32F32 = I32F32::lit("12");
const RANGER_VETERAN_XP_THRESHOLD: u32 = 60;

#[derive(Component, Default)]
pub struct Ranger;

#[derive(Component, Clone, Copy, Default)]
pub struct ReplicatedUnitId(pub u64);

#[derive(Component, Clone, Copy)]
pub struct ArmorReduction(pub I32F32);

impl Default for ArmorReduction {
    fn default() -> Self {
        Self(RANGER_BASE_ARMOR_REDUCTION)
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct AttackNoise(pub I32F32);

#[derive(Component, Clone, Copy, Default)]
pub struct Experience {
    pub value: u32,
}

#[derive(Component, Clone, Copy, Default)]
pub struct Veteran(pub bool);

#[derive(Bundle)]
pub struct RangerBundle {
    pub unit: Ranger,
    pub replicated_id: ReplicatedUnitId,
    pub position: SimPosition,
    pub health: Health,
    pub stats: UnitStats,
    pub armor_reduction: ArmorReduction,
    pub attack_noise: AttackNoise,
    pub experience: Experience,
    pub veteran: Veteran,
}

#[derive(Resource, Default)]
pub struct NextReplicatedUnitId(pub u64);

#[derive(Event, Clone, Copy)]
pub struct SpawnRangerEvent {
    pub position: SimPosition,
}

#[derive(Event, Clone, Copy)]
pub struct GainRangerExperienceEvent {
    pub entity: Entity,
    pub amount: u32,
}

#[derive(Event, Clone, Copy)]
pub struct DismissRangerEvent {
    pub entity: Entity,
}

impl Default for RangerBundle {
    fn default() -> Self {
        Self {
            unit: Ranger,
            replicated_id: ReplicatedUnitId::default(),
            position: SimPosition {
                x: I32F32::ZERO,
                y: I32F32::ZERO,
            },
            health: Health::full(RANGER_HP),
            stats: UnitStats {
                move_speed: RANGER_MOVE_SPEED,
                attack_range: RANGER_ATTACK_RANGE,
                attack_damage: RANGER_ATTACK_DAMAGE,
                attack_speed: RANGER_ATTACK_SPEED,
                watch_range: RANGER_WATCH_RANGE,
            },
            armor_reduction: ArmorReduction::default(),
            attack_noise: AttackNoise(RANGER_BASE_ATTACK_NOISE),
            experience: Experience::default(),
            veteran: Veteran::default(),
        }
    }
}

fn spawn_ranger_system(
    mut commands: Commands,
    mut events: EventReader<SpawnRangerEvent>,
    mut next_id: ResMut<NextReplicatedUnitId>,
) {
    for ev in events.read() {
        let mut bundle = RangerBundle::default();
        bundle.position = ev.position;
        bundle.replicated_id = ReplicatedUnitId(next_id.0);
        next_id.0 = next_id.0.wrapping_add(1);
        commands.spawn(bundle);
    }
}

fn gain_ranger_experience_system(
    mut events: EventReader<GainRangerExperienceEvent>,
    mut units: Query<(&mut Experience, &mut Veteran, &mut UnitStats), With<Ranger>>,
) {
    for ev in events.read() {
        let Ok((mut xp, mut veteran, mut stats)) = units.get_mut(ev.entity) else {
            continue;
        };

        xp.value = xp.value.saturating_add(ev.amount);

        if !veteran.0 && xp.value >= RANGER_VETERAN_XP_THRESHOLD {
            veteran.0 = true;
            stats.attack_range = RANGER_VETERAN_ATTACK_RANGE;
            stats.attack_speed = RANGER_VETERAN_ATTACK_SPEED;
            stats.attack_damage = RANGER_VETERAN_ATTACK_DAMAGE;
        }
    }
}

fn dismiss_ranger_system(
    mut commands: Commands,
    mut events: EventReader<DismissRangerEvent>,
    units: Query<(), With<Ranger>>,
) {
    for ev in events.read() {
        if units.get(ev.entity).is_ok() {
            commands.entity(ev.entity).despawn();
        }
    }
}

fn ranger_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    units: Query<
        (
            &ReplicatedUnitId,
            &SimPosition,
            &Health,
            &UnitStats,
            &ArmorReduction,
            &AttackNoise,
            &Experience,
            &Veteran,
        ),
        With<Ranger>,
    >,
) {
    for (replicated_id, pos, hp, stats, armor, noise, xp, veteran) in &units {
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
        checksum.accumulate(noise.0.to_bits() as u64);
        checksum.accumulate(xp.value as u64);
        checksum.accumulate(u64::from(veteran.0));
    }
}

pub struct RangerPlugin;

impl Plugin for RangerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NextReplicatedUnitId>()
            .add_event::<SpawnRangerEvent>()
            .add_event::<GainRangerExperienceEvent>()
            .add_event::<DismissRangerEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_ranger_system,
                    gain_ranger_experience_system,
                    dismiss_ranger_system,
                    ranger_checksum_system,
                )
                    .chain(),
            );
    }
}