// Sources: vault/units/calliope.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{Health, SimChecksumState, SimPosition, UnitStats};

const CALLIOPE_HP: I32F32 = I32F32::lit("60");
const CALLIOPE_MOVE_SPEED: I32F32 = I32F32::lit("3.2");
const CALLIOPE_ATTACK_RANGE: I32F32 = I32F32::lit("4.5");
const CALLIOPE_ATTACK_SPEED: I32F32 = I32F32::lit("5");
const CALLIOPE_ATTACK_DAMAGE: I32F32 = I32F32::lit("15");
const CALLIOPE_WATCH_RANGE: I32F32 = I32F32::lit("7");
const CALLIOPE_BASE_ARMOR_REDUCTION: I32F32 = I32F32::lit("0.10");

#[derive(Component, Default)]
pub struct Calliope;

#[derive(Component, Clone, Copy, Default)]
pub struct ReplicatedUnitId(pub u64);

#[derive(Component, Clone, Copy)]
pub struct ArmorReduction(pub I32F32);

impl Default for ArmorReduction {
    fn default() -> Self {
        Self(CALLIOPE_BASE_ARMOR_REDUCTION)
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct AttackNoise(pub I32F32);

#[derive(Component, Clone, Copy, Default)]
pub struct ReloadSpeedBonus(pub I32F32);

#[derive(Component, Clone, Copy, Default)]
pub struct PerkPoints {
    pub available: i32,
}

#[derive(Component, Clone, Copy, Default)]
pub struct PerkRefundLocked(pub bool);

#[derive(Component, Clone, Copy, Default)]
pub struct ScoutingMissionRewarded(pub bool);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CalliopePerk {
    AimI,
    AimII,
    AimIII,
    AimIV,
    ProtectionI,
    ProtectionII,
    ProtectionIII,
    StrengthI,
    StrengthII,
    StrengthIII,
    StrengthIV,
    SpeedI,
    SpeedII,
    SpeedIII,
    DexterityI,
    DexterityII,
    DexterityIII,
    SilentI,
    SilentII,
    QuickReflexesI,
    QuickReflexesII,
    ImprovedVisionI,
    ImprovedVisionII,
    ImprovedVisionIII,
}

#[derive(Component, Clone, Default)]
pub struct CalliopePerks {
    pub purchased: Vec<CalliopePerk>,
}

#[derive(Bundle)]
pub struct CalliopeBundle {
    pub unit: Calliope,
    pub replicated_id: ReplicatedUnitId,
    pub position: SimPosition,
    pub health: Health,
    pub stats: UnitStats,
    pub armor_reduction: ArmorReduction,
    pub attack_noise: AttackNoise,
    pub reload_speed_bonus: ReloadSpeedBonus,
    pub perk_points: PerkPoints,
    pub perks: CalliopePerks,
    pub perk_refund_locked: PerkRefundLocked,
    pub scouting_mission_rewarded: ScoutingMissionRewarded,
}

#[derive(Resource, Default)]
pub struct NextReplicatedUnitId(pub u64);

#[derive(Event, Clone, Copy)]
pub struct SpawnCalliopeEvent {
    pub position: SimPosition,
}

#[derive(Event, Clone, Copy)]
pub struct CompleteScoutingMissionFirstTimeEvent {
    pub entity: Entity,
}

#[derive(Event, Clone, Copy)]
pub struct PurchaseCalliopePerkEvent {
    pub entity: Entity,
    pub perk: CalliopePerk,
}

#[derive(Event, Clone, Copy)]
pub struct RefundCalliopePerkEvent {
    pub entity: Entity,
    pub perk: CalliopePerk,
}

#[derive(Event, Clone, Copy)]
pub struct LockCalliopePerkRefundEvent {
    pub entity: Entity,
}

#[derive(Event, Clone, Copy)]
pub struct UnlockCalliopePerkRefundEvent {
    pub entity: Entity,
}

impl Default for CalliopeBundle {
    fn default() -> Self {
        Self {
            unit: Calliope,
            replicated_id: ReplicatedUnitId::default(),
            position: SimPosition {
                x: I32F32::ZERO,
                y: I32F32::ZERO,
            },
            health: Health::full(CALLIOPE_HP),
            stats: UnitStats {
                move_speed: CALLIOPE_MOVE_SPEED,
                attack_range: CALLIOPE_ATTACK_RANGE,
                attack_damage: CALLIOPE_ATTACK_DAMAGE,
                attack_speed: CALLIOPE_ATTACK_SPEED,
                watch_range: CALLIOPE_WATCH_RANGE,
            },
            armor_reduction: ArmorReduction::default(),
            attack_noise: AttackNoise(I32F32::ONE),
            reload_speed_bonus: ReloadSpeedBonus::default(),
            perk_points: PerkPoints::default(),
            perks: CalliopePerks::default(),
            perk_refund_locked: PerkRefundLocked::default(),
            scouting_mission_rewarded: ScoutingMissionRewarded::default(),
        }
    }
}

fn perk_position(perk: CalliopePerk) -> (i32, i32) {
    match perk {
        CalliopePerk::AimI => (-3, -1),
        CalliopePerk::AimII => (-2, -1),
        CalliopePerk::AimIII => (-1, -1),
        CalliopePerk::AimIV => (0, -1),
        CalliopePerk::ProtectionI => (2, -2),
        CalliopePerk::ProtectionII => (2, -1),
        CalliopePerk::ProtectionIII => (2, 0),
        CalliopePerk::StrengthI => (-2, 2),
        CalliopePerk::StrengthII => (-1, 2),
        CalliopePerk::StrengthIII => (0, 2),
        CalliopePerk::StrengthIV => (1, 2),
        CalliopePerk::SpeedI => (-2, 0),
        CalliopePerk::SpeedII => (-1, 0),
        CalliopePerk::SpeedIII => (0, 0),
        CalliopePerk::DexterityI => (-2, -2),
        CalliopePerk::DexterityII => (-1, -2),
        CalliopePerk::DexterityIII => (0, -2),
        CalliopePerk::SilentI => (1, -1),
        CalliopePerk::SilentII => (1, 0),
        CalliopePerk::QuickReflexesI => (0, 1),
        CalliopePerk::QuickReflexesII => (1, 1),
        CalliopePerk::ImprovedVisionI => (-1, 1),
        CalliopePerk::ImprovedVisionII => (-2, 1),
        CalliopePerk::ImprovedVisionIII => (-3, 1),
    }
}

fn is_center_perk(perk: CalliopePerk) -> bool {
    matches!(
        perk,
        CalliopePerk::SpeedII
            | CalliopePerk::SpeedIII
            | CalliopePerk::QuickReflexesI
            | CalliopePerk::ImprovedVisionI
    )
}

fn has_adjacent_unlocked_perk(target: CalliopePerk, purchased: &[CalliopePerk]) -> bool {
    let (tx, ty) = perk_position(target);
    for perk in purchased {
        let (px, py) = perk_position(*perk);
        let dx = (tx - px).abs();
        let dy = (ty - py).abs();
        if dx <= 1 && dy <= 1 {
            return true;
        }
    }
    false
}

fn can_purchase_perk(target: CalliopePerk, purchased: &[CalliopePerk]) -> bool {
    if purchased.contains(&target) {
        return false;
    }

    if purchased.is_empty() {
        return is_center_perk(target);
    }

    has_adjacent_unlocked_perk(target, purchased)
}

fn apply_perk_effect(
    stats: &mut UnitStats,
    health: &mut Health,
    armor: &mut ArmorReduction,
    noise: &mut AttackNoise,
    reload: &mut ReloadSpeedBonus,
    perk: CalliopePerk,
) {
    match perk {
        CalliopePerk::AimI => stats.attack_damage = stats.attack_damage * I32F32::lit("1.30"),
        CalliopePerk::AimII => stats.attack_damage = stats.attack_damage * I32F32::lit("1.30"),
        CalliopePerk::AimIII => stats.attack_damage = stats.attack_damage * I32F32::lit("1.40"),
        CalliopePerk::AimIV => stats.attack_damage = stats.attack_damage * I32F32::lit("1.50"),
        CalliopePerk::ProtectionI => armor.0 = armor.0 + I32F32::lit("0.15"),
        CalliopePerk::ProtectionII => armor.0 = armor.0 + I32F32::lit("0.15"),
        CalliopePerk::ProtectionIII => armor.0 = armor.0 + I32F32::lit("0.15"),
        CalliopePerk::StrengthI => {
            health.max = health.max * I32F32::lit("1.30");
            health.current = health.current * I32F32::lit("1.30");
        }
        CalliopePerk::StrengthII => {
            health.max = health.max * I32F32::lit("1.30");
            health.current = health.current * I32F32::lit("1.30");
        }
        CalliopePerk::StrengthIII => {
            health.max = health.max * I32F32::lit("1.40");
            health.current = health.current * I32F32::lit("1.40");
        }
        CalliopePerk::StrengthIV => {
            health.max = health.max * I32F32::lit("1.50");
            health.current = health.current * I32F32::lit("1.50");
        }
        CalliopePerk::SpeedI => stats.move_speed = stats.move_speed * I32F32::lit("1.20"),
        CalliopePerk::SpeedII => stats.move_speed = stats.move_speed * I32F32::lit("1.20"),
        CalliopePerk::SpeedIII => stats.move_speed = stats.move_speed * I32F32::lit("1.30"),
        CalliopePerk::DexterityI => stats.attack_speed = stats.attack_speed * I32F32::lit("1.20"),
        CalliopePerk::DexterityII => stats.attack_speed = stats.attack_speed * I32F32::lit("1.20"),
        CalliopePerk::DexterityIII => stats.attack_speed = stats.attack_speed * I32F32::lit("1.25"),
        CalliopePerk::SilentI => noise.0 = noise.0 * I32F32::lit("0.60"),
        CalliopePerk::SilentII => noise.0 = noise.0 * I32F32::lit("0.60"),
        CalliopePerk::QuickReflexesI => reload.0 = reload.0 + I32F32::lit("0.40"),
        CalliopePerk::QuickReflexesII => reload.0 = reload.0 + I32F32::lit("0.40"),
        CalliopePerk::ImprovedVisionI => {
            stats.attack_range = stats.attack_range + I32F32::ONE;
            stats.watch_range = stats.watch_range + I32F32::ONE;
        }
        CalliopePerk::ImprovedVisionII => {
            stats.attack_range = stats.attack_range + I32F32::ONE;
            stats.watch_range = stats.watch_range + I32F32::ONE;
        }
        CalliopePerk::ImprovedVisionIII => {
            stats.attack_range = stats.attack_range + I32F32::ONE;
            stats.watch_range = stats.watch_range + I32F32::ONE;
        }
    }
}

fn reset_to_base_stats(
    stats: &mut UnitStats,
    health: &mut Health,
    armor: &mut ArmorReduction,
    noise: &mut AttackNoise,
    reload: &mut ReloadSpeedBonus,
) {
    *stats = UnitStats {
        move_speed: CALLIOPE_MOVE_SPEED,
        attack_range: CALLIOPE_ATTACK_RANGE,
        attack_damage: CALLIOPE_ATTACK_DAMAGE,
        attack_speed: CALLIOPE_ATTACK_SPEED,
        watch_range: CALLIOPE_WATCH_RANGE,
    };
    *health = Health::full(CALLIOPE_HP);
    *armor = ArmorReduction::default();
    *noise = AttackNoise(I32F32::ONE);
    *reload = ReloadSpeedBonus::default();
}

fn spawn_calliope_system(
    mut commands: Commands,
    mut events: EventReader<SpawnCalliopeEvent>,
    mut next_id: ResMut<NextReplicatedUnitId>,
) {
    for ev in events.read() {
        let mut bundle = CalliopeBundle::default();
        bundle.position = ev.position;
        bundle.replicated_id = ReplicatedUnitId(next_id.0);
        next_id.0 = next_id.0.wrapping_add(1);
        commands.spawn(bundle);
    }
}

fn award_scouting_mission_perk_point_system(
    mut events: EventReader<CompleteScoutingMissionFirstTimeEvent>,
    mut units: Query<(&mut PerkPoints, &mut ScoutingMissionRewarded), With<Calliope>>,
) {
    for ev in events.read() {
        let Ok((mut points, mut rewarded)) = units.get_mut(ev.entity) else {
            continue;
        };

        if !rewarded.0 {
            points.available += 1;
            rewarded.0 = true;
        }
    }
}

fn purchase_calliope_perk_system(
    mut events: EventReader<PurchaseCalliopePerkEvent>,
    mut units: Query<
        (
            &mut PerkPoints,
            &mut CalliopePerks,
            &mut UnitStats,
            &mut Health,
            &mut ArmorReduction,
            &mut AttackNoise,
            &mut ReloadSpeedBonus,
        ),
        With<Calliope>,
    >,
) {
    for ev in events.read() {
        let Ok((mut points, mut perks, mut stats, mut health, mut armor, mut noise, mut reload)) =
            units.get_mut(ev.entity)
        else {
            continue;
        };

        if points.available <= 0 || !can_purchase_perk(ev.perk, &perks.purchased) {
            continue;
        }

        points.available -= 1;
        perks.purchased.push(ev.perk);
        apply_perk_effect(&mut stats, &mut health, &mut armor, &mut noise, &mut reload, ev.perk);
    }
}

fn refund_calliope_perk_system(
    mut events: EventReader<RefundCalliopePerkEvent>,
    mut units: Query<
        (
            &mut PerkPoints,
            &mut CalliopePerks,
            &mut UnitStats,
            &mut Health,
            &mut ArmorReduction,
            &mut AttackNoise,
            &mut ReloadSpeedBonus,
            &PerkRefundLocked,
        ),
        With<Calliope>,
    >,
) {
    for ev in events.read() {
        let Ok((
            mut points,
            mut perks,
            mut stats,
            mut health,
            mut armor,
            mut noise,
            mut reload,
            locked,
        )) = units.get_mut(ev.entity)
        else {
            continue;
        };

        if locked.0 {
            continue;
        }

        let mut removed = false;
        perks.purchased.retain(|perk| {
            if !removed && *perk == ev.perk {
                removed = true;
                false
            } else {
                true
            }
        });

        if !removed {
            continue;
        }

        points.available += 1;
        reset_to_base_stats(&mut stats, &mut health, &mut armor, &mut noise, &mut reload);
        for perk in perks.purchased.iter().copied() {
            apply_perk_effect(&mut stats, &mut health, &mut armor, &mut noise, &mut reload, perk);
        }
    }
}

fn lock_refund_system(
    mut events: EventReader<LockCalliopePerkRefundEvent>,
    mut units: Query<&mut PerkRefundLocked, With<Calliope>>,
) {
    for ev in events.read() {
        let Ok(mut locked) = units.get_mut(ev.entity) else {
            continue;
        };
        locked.0 = true;
    }
}

fn unlock_refund_system(
    mut events: EventReader<UnlockCalliopePerkRefundEvent>,
    mut units: Query<&mut PerkRefundLocked, With<Calliope>>,
) {
    for ev in events.read() {
        let Ok(mut locked) = units.get_mut(ev.entity) else {
            continue;
        };
        locked.0 = false;
    }
}

fn calliope_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    units: Query<
        (
            &ReplicatedUnitId,
            &SimPosition,
            &Health,
            &UnitStats,
            &ArmorReduction,
            &AttackNoise,
            &ReloadSpeedBonus,
            &PerkPoints,
            &CalliopePerks,
            &PerkRefundLocked,
            &ScoutingMissionRewarded,
        ),
        With<Calliope>,
    >,
) {
    for (replicated_id, pos, hp, stats, armor, noise, reload, points, perks, locked, rewarded) in
        &units
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
        checksum.accumulate(noise.0.to_bits() as u64);
        checksum.accumulate(reload.0.to_bits() as u64);
        checksum.accumulate(points.available as u64);
        checksum.accumulate(perks.purchased.len() as u64);
        for perk in &perks.purchased {
            checksum.accumulate(*perk as u64);
        }
        checksum.accumulate(u64::from(locked.0));
        checksum.accumulate(u64::from(rewarded.0));
    }
}

pub struct CalliopePlugin;

impl Plugin for CalliopePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NextReplicatedUnitId>()
            .add_event::<SpawnCalliopeEvent>()
            .add_event::<CompleteScoutingMissionFirstTimeEvent>()
            .add_event::<PurchaseCalliopePerkEvent>()
            .add_event::<RefundCalliopePerkEvent>()
            .add_event::<LockCalliopePerkRefundEvent>()
            .add_event::<UnlockCalliopePerkRefundEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_calliope_system,
                    award_scouting_mission_perk_point_system,
                    purchase_calliope_perk_system,
                    refund_calliope_perk_system,
                    lock_refund_system,
                    unlock_refund_system,
                    calliope_checksum_system,
                )
                    .chain(),
            );
    }
}