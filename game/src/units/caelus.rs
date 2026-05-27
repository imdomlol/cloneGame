// Sources: vault/units/caelus.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{Health, SimChecksumState, SimPosition, UnitStats};

const CAELUS_HP: I32F32 = I32F32::lit("120");
const CAELUS_MOVE_SPEED: I32F32 = I32F32::lit("2");
const CAELUS_ATTACK_RANGE: I32F32 = I32F32::lit("6");
const CAELUS_ATTACK_SPEED: I32F32 = I32F32::lit("0.71");
const CAELUS_ATTACK_DAMAGE: I32F32 = I32F32::lit("50");
const CAELUS_WATCH_RANGE: I32F32 = I32F32::lit("7");
const CAELUS_BASE_ARMOR_REDUCTION: I32F32 = I32F32::lit("0.10");
const CAELUS_BASE_EFFECT_RADIUS: I32F32 = I32F32::lit("1.0");

#[derive(Component, Default)]
pub struct Caelus;

#[derive(Component, Clone, Copy, Default)]
pub struct ReplicatedUnitId(pub u64);

#[derive(Component, Clone, Copy)]
pub struct ArmorReduction(pub I32F32);

impl Default for ArmorReduction {
    fn default() -> Self {
        Self(CAELUS_BASE_ARMOR_REDUCTION)
    }
}

#[derive(Component, Clone, Copy)]
pub struct EffectRadius(pub I32F32);

impl Default for EffectRadius {
    fn default() -> Self {
        Self(CAELUS_BASE_EFFECT_RADIUS)
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct LoadingWeaponSpeedBonus(pub I32F32);

#[derive(Component, Clone, Copy, Default)]
pub struct PerkPoints {
    pub available: i32,
}

#[derive(Component, Clone, Copy, Default)]
pub struct PerkRefundLocked(pub bool);

#[derive(Component, Clone, Copy, Default)]
pub struct ScoutingMissionRewarded(pub bool);

#[derive(Component, Clone, Copy, Default)]
pub struct ReissuedAttackCommand(pub bool);

#[derive(Component, Clone, Copy, Default)]
pub struct OneShotExecutiveUnlocked(pub bool);

#[derive(Component, Clone, Copy, Default)]
pub struct OneShotSeveralExecutivesUnlocked(pub bool);

#[derive(Component, Clone, Copy, Default)]
pub struct OneShotHarpyOrVenomUnlocked(pub bool);

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CaelusPerk {
    AimI,
    AimII,
    AimIII,
    StrengthI,
    StrengthII,
    StrengthIII,
    StrengthIV,
    DexterityI,
    DexterityII,
    DexterityIII,
    QuickReflexesI,
    QuickReflexesII,
    QuickReflexesIII,
    ProtectionI,
    ProtectionII,
    ProtectionIII,
    ImprovedVisionI,
    ImprovedVisionIII,
    SpeedI,
    SpeedII,
    SpeedIII,
    EffectRadiusI,
    EffectRadiusII,
}

#[derive(Component, Clone, Default)]
pub struct CaelusPerks {
    pub purchased: Vec<CaelusPerk>,
}

#[derive(Bundle)]
pub struct CaelusBundle {
    pub unit: Caelus,
    pub replicated_id: ReplicatedUnitId,
    pub position: SimPosition,
    pub health: Health,
    pub stats: UnitStats,
    pub armor_reduction: ArmorReduction,
    pub effect_radius: EffectRadius,
    pub loading_weapon_speed_bonus: LoadingWeaponSpeedBonus,
    pub perk_points: PerkPoints,
    pub perks: CaelusPerks,
    pub perk_refund_locked: PerkRefundLocked,
    pub scouting_mission_rewarded: ScoutingMissionRewarded,
    pub reissued_attack_command: ReissuedAttackCommand,
    pub one_shot_executive_unlocked: OneShotExecutiveUnlocked,
    pub one_shot_several_executives_unlocked: OneShotSeveralExecutivesUnlocked,
    pub one_shot_harpy_or_venom_unlocked: OneShotHarpyOrVenomUnlocked,
}

#[derive(Resource, Default)]
pub struct NextReplicatedUnitId(pub u64);

#[derive(Event, Clone, Copy)]
pub struct SpawnCaelusEvent {
    pub position: SimPosition,
}

#[derive(Event, Clone, Copy)]
pub struct CompleteCaelusScoutingMissionFirstTimeEvent {
    pub entity: Entity,
}

#[derive(Event, Clone, Copy)]
pub struct PurchaseCaelusPerkEvent {
    pub entity: Entity,
    pub perk: CaelusPerk,
}

#[derive(Event, Clone, Copy)]
pub struct RefundCaelusPerkEvent {
    pub entity: Entity,
    pub perk: CaelusPerk,
}

#[derive(Event, Clone, Copy)]
pub struct LockCaelusPerkRefundEvent {
    pub entity: Entity,
}

#[derive(Event, Clone, Copy)]
pub struct UnlockCaelusPerkRefundEvent {
    pub entity: Entity,
}

#[derive(Event, Clone, Copy)]
pub struct ReissueCaelusAttackCommandEvent {
    pub entity: Entity,
}

impl Default for CaelusBundle {
    fn default() -> Self {
        Self {
            unit: Caelus,
            replicated_id: ReplicatedUnitId::default(),
            position: SimPosition {
                x: I32F32::ZERO,
                y: I32F32::ZERO,
            },
            health: Health::full(CAELUS_HP),
            stats: UnitStats {
                move_speed: CAELUS_MOVE_SPEED,
                attack_range: CAELUS_ATTACK_RANGE,
                attack_damage: CAELUS_ATTACK_DAMAGE,
                attack_speed: CAELUS_ATTACK_SPEED,
                watch_range: CAELUS_WATCH_RANGE,
            },
            armor_reduction: ArmorReduction::default(),
            effect_radius: EffectRadius::default(),
            loading_weapon_speed_bonus: LoadingWeaponSpeedBonus::default(),
            perk_points: PerkPoints::default(),
            perks: CaelusPerks::default(),
            perk_refund_locked: PerkRefundLocked::default(),
            scouting_mission_rewarded: ScoutingMissionRewarded::default(),
            reissued_attack_command: ReissuedAttackCommand::default(),
            one_shot_executive_unlocked: OneShotExecutiveUnlocked::default(),
            one_shot_several_executives_unlocked: OneShotSeveralExecutivesUnlocked::default(),
            one_shot_harpy_or_venom_unlocked: OneShotHarpyOrVenomUnlocked::default(),
        }
    }
}

fn perk_position(perk: CaelusPerk) -> (i32, i32) {
    match perk {
        CaelusPerk::AimI => (-3, -1),
        CaelusPerk::AimII => (-2, -1),
        CaelusPerk::AimIII => (-1, -1),
        CaelusPerk::DexterityI => (-2, -2),
        CaelusPerk::DexterityII => (-1, -2),
        CaelusPerk::DexterityIII => (0, -2),
        CaelusPerk::QuickReflexesI => (0, 1),
        CaelusPerk::QuickReflexesII => (1, 1),
        CaelusPerk::QuickReflexesIII => (2, 1),
        CaelusPerk::ProtectionI => (2, -2),
        CaelusPerk::ProtectionII => (2, -1),
        CaelusPerk::ProtectionIII => (2, 0),
        CaelusPerk::StrengthI => (-2, 2),
        CaelusPerk::StrengthII => (-1, 2),
        CaelusPerk::StrengthIII => (0, 2),
        CaelusPerk::StrengthIV => (1, 2),
        CaelusPerk::SpeedI => (-2, 0),
        CaelusPerk::SpeedII => (-1, 0),
        CaelusPerk::SpeedIII => (0, 0),
        CaelusPerk::ImprovedVisionI => (-1, 1),
        CaelusPerk::ImprovedVisionIII => (-2, 1),
        CaelusPerk::EffectRadiusI => (1, -1),
        CaelusPerk::EffectRadiusII => (1, 0),
    }
}

fn is_center_perk(perk: CaelusPerk) -> bool {
    matches!(
        perk,
        CaelusPerk::SpeedII
            | CaelusPerk::DexterityII
            | CaelusPerk::QuickReflexesI
            | CaelusPerk::ImprovedVisionI
    )
}

fn has_adjacent_unlocked_perk(target: CaelusPerk, purchased: &[CaelusPerk]) -> bool {
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

fn can_purchase_perk(target: CaelusPerk, purchased: &[CaelusPerk]) -> bool {
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
    effect_radius: &mut EffectRadius,
    loading_weapon_speed_bonus: &mut LoadingWeaponSpeedBonus,
    perk: CaelusPerk,
) {
    match perk {
        CaelusPerk::AimI => stats.attack_damage = stats.attack_damage * I32F32::lit("1.40"),
        CaelusPerk::AimII => stats.attack_damage = stats.attack_damage * I32F32::lit("1.50"),
        CaelusPerk::AimIII => stats.attack_damage = stats.attack_damage * I32F32::lit("1.70"),
        CaelusPerk::StrengthI => {
            health.max = health.max * I32F32::lit("1.40");
            health.current = health.current * I32F32::lit("1.40");
        }
        CaelusPerk::StrengthII => {
            health.max = health.max * I32F32::lit("1.40");
            health.current = health.current * I32F32::lit("1.40");
        }
        CaelusPerk::StrengthIII => {
            health.max = health.max * I32F32::lit("1.50");
            health.current = health.current * I32F32::lit("1.50");
        }
        CaelusPerk::StrengthIV => {
            health.max = health.max * I32F32::lit("1.60");
            health.current = health.current * I32F32::lit("1.60");
        }
        CaelusPerk::DexterityI => stats.attack_speed = stats.attack_speed * I32F32::lit("1.20"),
        CaelusPerk::DexterityII => stats.attack_speed = stats.attack_speed * I32F32::lit("1.20"),
        CaelusPerk::DexterityIII => stats.attack_speed = stats.attack_speed * I32F32::lit("1.20"),
        CaelusPerk::QuickReflexesI => {
            loading_weapon_speed_bonus.0 = loading_weapon_speed_bonus.0 + I32F32::lit("0.25")
        }
        CaelusPerk::QuickReflexesII => {
            loading_weapon_speed_bonus.0 = loading_weapon_speed_bonus.0 + I32F32::lit("0.25")
        }
        CaelusPerk::QuickReflexesIII => {
            loading_weapon_speed_bonus.0 = loading_weapon_speed_bonus.0 + I32F32::lit("0.30")
        }
        CaelusPerk::ProtectionI => armor.0 = armor.0 + I32F32::lit("0.15"),
        CaelusPerk::ProtectionII => armor.0 = armor.0 + I32F32::lit("0.15"),
        CaelusPerk::ProtectionIII => armor.0 = armor.0 + I32F32::lit("0.15"),
        CaelusPerk::ImprovedVisionI => {
            stats.watch_range = stats.watch_range + I32F32::ONE;
            stats.attack_range = stats.attack_range + I32F32::ONE;
        }
        CaelusPerk::ImprovedVisionIII => {
            stats.watch_range = stats.watch_range + I32F32::ONE;
            stats.attack_range = stats.attack_range + I32F32::ONE;
        }
        CaelusPerk::SpeedI => stats.move_speed = stats.move_speed * I32F32::lit("1.20"),
        CaelusPerk::SpeedII => stats.move_speed = stats.move_speed * I32F32::lit("1.20"),
        CaelusPerk::SpeedIII => stats.move_speed = stats.move_speed * I32F32::lit("1.30"),
        CaelusPerk::EffectRadiusI => effect_radius.0 = effect_radius.0 + I32F32::lit("0.2"),
        CaelusPerk::EffectRadiusII => effect_radius.0 = effect_radius.0 + I32F32::lit("0.2"),
    }
}

fn reset_to_base_stats(
    stats: &mut UnitStats,
    health: &mut Health,
    armor: &mut ArmorReduction,
    effect_radius: &mut EffectRadius,
    loading_weapon_speed_bonus: &mut LoadingWeaponSpeedBonus,
) {
    *stats = UnitStats {
        move_speed: CAELUS_MOVE_SPEED,
        attack_range: CAELUS_ATTACK_RANGE,
        attack_damage: CAELUS_ATTACK_DAMAGE,
        attack_speed: CAELUS_ATTACK_SPEED,
        watch_range: CAELUS_WATCH_RANGE,
    };
    *health = Health::full(CAELUS_HP);
    *armor = ArmorReduction::default();
    *effect_radius = EffectRadius::default();
    *loading_weapon_speed_bonus = LoadingWeaponSpeedBonus::default();
}

fn refresh_one_shot_flags_system(
    mut units: Query<
        (
            &CaelusPerks,
            &mut OneShotExecutiveUnlocked,
            &mut OneShotSeveralExecutivesUnlocked,
            &mut OneShotHarpyOrVenomUnlocked,
        ),
        With<Caelus>,
    >,
) {
    for (perks, mut one_shot_exec, mut one_shot_multi_exec, mut one_shot_harpy_or_venom) in &mut units
    {
        let has_aim_i = perks.purchased.contains(&CaelusPerk::AimI);
        let has_aim_ii = perks.purchased.contains(&CaelusPerk::AimII);
        let has_aim_iii = perks.purchased.contains(&CaelusPerk::AimIII);

        one_shot_exec.0 = has_aim_i;
        one_shot_multi_exec.0 = has_aim_ii;
        one_shot_harpy_or_venom.0 = has_aim_i && has_aim_iii;
    }
}

fn spawn_caelus_system(
    mut commands: Commands,
    mut events: EventReader<SpawnCaelusEvent>,
    mut next_id: ResMut<NextReplicatedUnitId>,
) {
    for ev in events.read() {
        let mut bundle = CaelusBundle::default();
        bundle.position = ev.position;
        bundle.replicated_id = ReplicatedUnitId(next_id.0);
        next_id.0 = next_id.0.wrapping_add(1);
        commands.spawn(bundle);
    }
}

fn award_scouting_mission_perk_point_system(
    mut events: EventReader<CompleteCaelusScoutingMissionFirstTimeEvent>,
    mut units: Query<(&mut PerkPoints, &mut ScoutingMissionRewarded), With<Caelus>>,
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

fn purchase_caelus_perk_system(
    mut events: EventReader<PurchaseCaelusPerkEvent>,
    mut units: Query<
        (
            &mut PerkPoints,
            &mut CaelusPerks,
            &mut UnitStats,
            &mut Health,
            &mut ArmorReduction,
            &mut EffectRadius,
            &mut LoadingWeaponSpeedBonus,
        ),
        With<Caelus>,
    >,
) {
    for ev in events.read() {
        let Ok((
            mut points,
            mut perks,
            mut stats,
            mut health,
            mut armor,
            mut effect_radius,
            mut loading_weapon_speed_bonus,
        )) = units.get_mut(ev.entity)
        else {
            continue;
        };

        if points.available <= 0 || !can_purchase_perk(ev.perk, &perks.purchased) {
            continue;
        }

        points.available -= 1;
        perks.purchased.push(ev.perk);
        apply_perk_effect(
            &mut stats,
            &mut health,
            &mut armor,
            &mut effect_radius,
            &mut loading_weapon_speed_bonus,
            ev.perk,
        );
    }
}

fn refund_caelus_perk_system(
    mut events: EventReader<RefundCaelusPerkEvent>,
    mut units: Query<
        (
            &mut PerkPoints,
            &mut CaelusPerks,
            &mut UnitStats,
            &mut Health,
            &mut ArmorReduction,
            &mut EffectRadius,
            &mut LoadingWeaponSpeedBonus,
            &PerkRefundLocked,
        ),
        With<Caelus>,
    >,
) {
    for ev in events.read() {
        let Ok((
            mut points,
            mut perks,
            mut stats,
            mut health,
            mut armor,
            mut effect_radius,
            mut loading_weapon_speed_bonus,
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
        reset_to_base_stats(
            &mut stats,
            &mut health,
            &mut armor,
            &mut effect_radius,
            &mut loading_weapon_speed_bonus,
        );
        for perk in perks.purchased.iter().copied() {
            apply_perk_effect(
                &mut stats,
                &mut health,
                &mut armor,
                &mut effect_radius,
                &mut loading_weapon_speed_bonus,
                perk,
            );
        }
    }
}

fn lock_refund_system(
    mut events: EventReader<LockCaelusPerkRefundEvent>,
    mut units: Query<&mut PerkRefundLocked, With<Caelus>>,
) {
    for ev in events.read() {
        let Ok(mut locked) = units.get_mut(ev.entity) else {
            continue;
        };
        locked.0 = true;
    }
}

fn unlock_refund_system(
    mut events: EventReader<UnlockCaelusPerkRefundEvent>,
    mut units: Query<&mut PerkRefundLocked, With<Caelus>>,
) {
    for ev in events.read() {
        let Ok(mut locked) = units.get_mut(ev.entity) else {
            continue;
        };
        locked.0 = false;
    }
}

fn reissue_attack_command_system(
    mut events: EventReader<ReissueCaelusAttackCommandEvent>,
    mut units: Query<&mut ReissuedAttackCommand, With<Caelus>>,
) {
    for ev in events.read() {
        let Ok(mut reissued) = units.get_mut(ev.entity) else {
            continue;
        };
        reissued.0 = true;
    }
}

fn caelus_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    units: Query<
        (
            &ReplicatedUnitId,
            &SimPosition,
            &Health,
            &UnitStats,
            &ArmorReduction,
            &EffectRadius,
            &LoadingWeaponSpeedBonus,
            &PerkPoints,
            &CaelusPerks,
            &PerkRefundLocked,
            &ScoutingMissionRewarded,
            &ReissuedAttackCommand,
            &OneShotExecutiveUnlocked,
            &OneShotSeveralExecutivesUnlocked,
            &OneShotHarpyOrVenomUnlocked,
        ),
        With<Caelus>,
    >,
) {
    for (
        replicated_id,
        pos,
        hp,
        stats,
        armor,
        effect_radius,
        loading_weapon_speed_bonus,
        points,
        perks,
        refund_locked,
        scouting_rewarded,
        reissued_attack,
        one_shot_exec,
        one_shot_multi_exec,
        one_shot_harpy_or_venom,
    ) in &units
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
        checksum.accumulate(effect_radius.0.to_bits() as u64);
        checksum.accumulate(loading_weapon_speed_bonus.0.to_bits() as u64);
        checksum.accumulate(points.available as u64);
        checksum.accumulate(perks.purchased.len() as u64);
        for perk in &perks.purchased {
            checksum.accumulate(*perk as u64);
        }
        checksum.accumulate(u64::from(refund_locked.0));
        checksum.accumulate(u64::from(scouting_rewarded.0));
        checksum.accumulate(u64::from(reissued_attack.0));
        checksum.accumulate(u64::from(one_shot_exec.0));
        checksum.accumulate(u64::from(one_shot_multi_exec.0));
        checksum.accumulate(u64::from(one_shot_harpy_or_venom.0));
    }
}

pub struct CaelusPlugin;

impl Plugin for CaelusPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NextReplicatedUnitId>()
            .add_event::<SpawnCaelusEvent>()
            .add_event::<CompleteCaelusScoutingMissionFirstTimeEvent>()
            .add_event::<PurchaseCaelusPerkEvent>()
            .add_event::<RefundCaelusPerkEvent>()
            .add_event::<LockCaelusPerkRefundEvent>()
            .add_event::<UnlockCaelusPerkRefundEvent>()
            .add_event::<ReissueCaelusAttackCommandEvent>()
            .add_systems(
                FixedUpdate,
                (
                    spawn_caelus_system,
                    award_scouting_mission_perk_point_system,
                    purchase_caelus_perk_system,
                    refund_caelus_perk_system,
                    lock_refund_system,
                    unlock_refund_system,
                    reissue_attack_command_system,
                    refresh_one_shot_flags_system,
                    caelus_checksum_system,
                )
                    .chain(),
            );
    }
}