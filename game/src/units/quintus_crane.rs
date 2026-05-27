// Sources: vault/characters/quintus_crane.md, vault/organizations/the_new_empire.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{Health, SimChecksumState, SimPosition, UnitStats};

const QUINTUS_CRANE_HP: I32F32 = I32F32::lit("0");
const QUINTUS_CRANE_MOVE_SPEED: I32F32 = I32F32::lit("0");
const QUINTUS_CRANE_ATTACK_RANGE: I32F32 = I32F32::lit("0");
const QUINTUS_CRANE_ATTACK_SPEED: I32F32 = I32F32::lit("0");
const QUINTUS_CRANE_ATTACK_DAMAGE: I32F32 = I32F32::lit("0");
const QUINTUS_CRANE_WATCH_RANGE: I32F32 = I32F32::lit("0");
const QUINTUS_CRANE_ARMOR_REDUCTION: I32F32 = I32F32::lit("0");

#[derive(Component, Default)]
pub struct QuintusCrane;

#[derive(Component, Clone, Copy, Default)]
pub struct ReplicatedUnitId(pub u64);

#[derive(Component, Clone, Copy)]
pub struct ArmorReduction(pub I32F32);

impl Default for ArmorReduction {
    fn default() -> Self {
        Self(QUINTUS_CRANE_ARMOR_REDUCTION)
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct NewEmpireFaction;

#[derive(Component, Clone, Copy, Default)]
pub struct CongratulatedHero(pub bool);

#[derive(Component, Clone, Copy, Default)]
pub struct AdditionalTerritoriesWillBeConquered(pub bool);

#[derive(Bundle)]
pub struct QuintusCraneBundle {
    pub unit: QuintusCrane,
    pub replicated_id: ReplicatedUnitId,
    pub position: SimPosition,
    pub health: Health,
    pub stats: UnitStats,
    pub armor_reduction: ArmorReduction,
    pub new_empire_faction: NewEmpireFaction,
    pub congratulated_hero: CongratulatedHero,
    pub additional_territories_will_be_conquered: AdditionalTerritoriesWillBeConquered,
}

impl Default for QuintusCraneBundle {
    fn default() -> Self {
        Self {
            unit: QuintusCrane,
            replicated_id: ReplicatedUnitId::default(),
            position: SimPosition {
                x: I32F32::ZERO,
                y: I32F32::ZERO,
            },
            health: Health::full(QUINTUS_CRANE_HP),
            stats: UnitStats {
                move_speed: QUINTUS_CRANE_MOVE_SPEED,
                attack_range: QUINTUS_CRANE_ATTACK_RANGE,
                attack_damage: QUINTUS_CRANE_ATTACK_DAMAGE,
                attack_speed: QUINTUS_CRANE_ATTACK_SPEED,
                watch_range: QUINTUS_CRANE_WATCH_RANGE,
            },
            armor_reduction: ArmorReduction::default(),
            new_empire_faction: NewEmpireFaction,
            congratulated_hero: CongratulatedHero::default(),
            additional_territories_will_be_conquered: AdditionalTerritoriesWillBeConquered::default(),
        }
    }
}

#[derive(Resource, Default)]
pub struct NextReplicatedUnitId(pub u64);

#[derive(Event, Clone, Copy)]
pub struct SpawnQuintusCraneEvent {
    pub position: SimPosition,
}

#[derive(Event, Clone, Copy)]
pub struct HeroConqueredEasternTerritoriesEvent;

fn spawn_quintus_crane_system(
    mut commands: Commands,
    mut events: EventReader<SpawnQuintusCraneEvent>,
    mut next_id: ResMut<NextReplicatedUnitId>,
) {
    for ev in events.read() {
        let mut bundle = QuintusCraneBundle::default();
        bundle.position = ev.position;
        bundle.replicated_id = ReplicatedUnitId(next_id.0);
        next_id.0 = next_id.0.wrapping_add(1);
        commands.spawn(bundle);
    }
}

fn process_eastern_territory_conquest_system(
    mut events: EventReader<HeroConqueredEasternTerritoriesEvent>,
    mut units: Query<
        (
            &mut CongratulatedHero,
            &mut AdditionalTerritoriesWillBeConquered,
        ),
        With<QuintusCrane>,
    >,
) {
    for _ in events.read() {
        for (mut congratulated, mut more_territories) in &mut units {
            congratulated.0 = true;
            more_territories.0 = true;
        }
    }
}

fn quintus_crane_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    units: Query<
        (
            &ReplicatedUnitId,
            &SimPosition,
            &Health,
            &UnitStats,
            &ArmorReduction,
            &CongratulatedHero,
            &AdditionalTerritoriesWillBeConquered,
        ),
        With<QuintusCrane>,
    >,
) {
    for (replicated_id, position, health, stats, armor, congratulated, more_territories) in &units {
        checksum.accumulate(replicated_id.0);
        checksum.accumulate(position.x.to_bits() as u64);
        checksum.accumulate(position.y.to_bits() as u64);
        checksum.accumulate(health.current.to_bits() as u64);
        checksum.accumulate(health.max.to_bits() as u64);
        checksum.accumulate(stats.move_speed.to_bits() as u64);
        checksum.accumulate(stats.attack_range.to_bits() as u64);
        checksum.accumulate(stats.attack_damage.to_bits() as u64);
        checksum.accumulate(stats.attack_speed.to_bits() as u64);
        checksum.accumulate(stats.watch_range.to_bits() as u64);
        checksum.accumulate(armor.0.to_bits() as u64);
        checksum.accumulate(u64::from(congratulated.0));
        checksum.accumulate(u64::from(more_territories.0));
    }
}

pub struct QuintusCranePlugin;

impl Plugin for QuintusCranePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NextReplicatedUnitId>()
            .add_event::<SpawnQuintusCraneEvent>()
            .add_event::<HeroConqueredEasternTerritoriesEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_quintus_crane_system,
                    process_eastern_territory_conquest_system,
                    quintus_crane_checksum_system,
                )
                    .chain(),
            );
    }
}