// Sources: vault/units/titan.md, vault/buildings/engineering_center.md

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{DamageType, Health, IncomingDamageEvent, SimChecksumState, SimHz, SimPosition, UnitStats};
use crate::units::Infected;

const TITAN_HP: I32F32 = I32F32::lit("800");
const TITAN_MOVE_SPEED: I32F32 = I32F32::lit("3");
const TITAN_ATTACK_RANGE: I32F32 = I32F32::lit("9");
const TITAN_ATTACK_SPEED: I32F32 = I32F32::lit("5");
const TITAN_ATTACK_DAMAGE: I32F32 = I32F32::lit("32");
const TITAN_WATCH_RANGE: I32F32 = I32F32::lit("12");

const TITAN_ARMOR_REDUCTION: I32F32 = I32F32::lit("0.40");
const TITAN_ATTACK_NOISE: I32F32 = I32F32::lit("20");
const TITAN_MEDIUM_AOE_RADIUS: I32F32 = I32F32::lit("1.5");
const TITAN_NARROW_TERRAIN_SPEED_MULT: I32F32 = I32F32::lit("0.60");

const TITAN_COST_GOLD: u32 = 2000;
const TITAN_COST_FOOD: u32 = 2;
const TITAN_COST_WORKERS: u32 = 1;
const TITAN_COST_IRON: u32 = 40;
const TITAN_COST_OIL: u32 = 40;
const TITAN_MAINTENANCE_GOLD: u32 = 40;
const TITAN_MAINTENANCE_OIL: u32 = 2;
const TITAN_BUILD_TIME_SECONDS: u32 = 45;
const TITAN_RESEARCH_COST_GOLD: u32 = 6000;

#[derive(Component, Default)]
pub struct Titan;

#[derive(Component, Clone, Copy)]
pub struct InfectedVenom;

#[derive(Component, Clone, Copy)]
pub struct InfectedGiant;

#[derive(Component, Clone, Copy)]
pub struct TitanAttackProfile {
    pub armor_reduction: I32F32,
    pub attack_noise: I32F32,
    pub aoe_radius: I32F32,
}

impl Default for TitanAttackProfile {
    fn default() -> Self {
        Self {
            armor_reduction: TITAN_ARMOR_REDUCTION,
            attack_noise: TITAN_ATTACK_NOISE,
            aoe_radius: TITAN_MEDIUM_AOE_RADIUS,
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct TitanAttackCooldown {
    pub ticks_remaining: u32,
}

#[derive(Component, Clone, Copy)]
pub struct TitanEconomy {
    pub cost_gold: u32,
    pub cost_food: u32,
    pub cost_workers: u32,
    pub cost_iron: u32,
    pub cost_oil: u32,
    pub maintenance_gold: u32,
    pub maintenance_oil: u32,
    pub build_time_seconds: u32,
    pub research_cost_gold: u32,
}

impl Default for TitanEconomy {
    fn default() -> Self {
        Self {
            cost_gold: TITAN_COST_GOLD,
            cost_food: TITAN_COST_FOOD,
            cost_workers: TITAN_COST_WORKERS,
            cost_iron: TITAN_COST_IRON,
            cost_oil: TITAN_COST_OIL,
            maintenance_gold: TITAN_MAINTENANCE_GOLD,
            maintenance_oil: TITAN_MAINTENANCE_OIL,
            build_time_seconds: TITAN_BUILD_TIME_SECONDS,
            research_cost_gold: TITAN_RESEARCH_COST_GOLD,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct TitanResearchGate {
    pub researched_in_foundry: bool,
}

impl Default for TitanResearchGate {
    fn default() -> Self {
        Self {
            researched_in_foundry: false,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct TitanRecruitmentGate {
    pub engineering_center_online: bool,
    pub oil_income_non_negative: bool,
}

impl Default for TitanRecruitmentGate {
    fn default() -> Self {
        Self {
            engineering_center_online: false,
            oil_income_non_negative: true,
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct TitanTerrainState {
    pub in_narrow_terrain: bool,
}

#[derive(Component, Clone, Copy)]
pub struct TitanMovementState {
    pub base_move_speed: I32F32,
    pub effective_move_speed: I32F32,
}

impl Default for TitanMovementState {
    fn default() -> Self {
        Self {
            base_move_speed: TITAN_MOVE_SPEED,
            effective_move_speed: TITAN_MOVE_SPEED,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TitanRolePreference {
    General,
    AvoidPrimaryTankVsVenom,
    TankOrKiteVsGiant,
}

#[derive(Component, Clone, Copy)]
pub struct TitanTacticalState {
    pub preference: TitanRolePreference,
}

impl Default for TitanTacticalState {
    fn default() -> Self {
        Self {
            preference: TitanRolePreference::General,
        }
    }
}

#[derive(Event, Clone, Copy)]
pub struct SetTitanResearchStateEvent {
    pub target: Entity,
    pub researched_in_foundry: bool,
}

#[derive(Event, Clone, Copy)]
pub struct SetTitanRecruitmentStateEvent {
    pub target: Entity,
    pub engineering_center_online: bool,
    pub oil_income_non_negative: bool,
}

#[derive(Event, Clone, Copy)]
pub struct SetTitanNarrowTerrainEvent {
    pub target: Entity,
    pub in_narrow_terrain: bool,
}

pub fn titan_base_health() -> Health {
    Health::full(TITAN_HP)
}

pub fn titan_base_stats() -> UnitStats {
    UnitStats {
        move_speed: TITAN_MOVE_SPEED,
        attack_range: TITAN_ATTACK_RANGE,
        attack_damage: TITAN_ATTACK_DAMAGE,
        attack_speed: TITAN_ATTACK_SPEED,
        watch_range: TITAN_WATCH_RANGE,
    }
}

pub fn spawn_titan(commands: &mut Commands, position: SimPosition) -> Entity {
    commands
        .spawn((
            Titan,
            position,
            titan_base_health(),
            titan_base_stats(),
            TitanAttackProfile::default(),
            TitanAttackCooldown::default(),
            TitanEconomy::default(),
            TitanResearchGate::default(),
            TitanRecruitmentGate::default(),
            TitanTerrainState::default(),
            TitanMovementState::default(),
            TitanTacticalState::default(),
        ))
        .id()
}

pub fn titan_attack_tick_system(
    sim_hz: Res<SimHz>,
    mut attackers: Query<
        (
            Entity,
            &SimPosition,
            &UnitStats,
            &TitanAttackProfile,
            &TitanResearchGate,
            &TitanRecruitmentGate,
            &mut TitanAttackCooldown,
        ),
        With<Titan>,
    >,
    infected_positions: Query<(Entity, &SimPosition), With<Infected>>,
    mut outgoing_damage: EventWriter<IncomingDamageEvent>,
) {
    for (titan_entity, titan_pos, stats, profile, research_gate, recruit_gate, mut cooldown) in &mut attackers {
        if !research_gate.researched_in_foundry {
            continue;
        }
        if !recruit_gate.engineering_center_online || !recruit_gate.oil_income_non_negative {
            continue;
        }

        if cooldown.ticks_remaining > 0 {
            cooldown.ticks_remaining -= 1;
            continue;
        }

        let attack_range_sq = stats.attack_range * stats.attack_range;
        let mut best_target: Option<(SimPosition, I32F32)> = None;

        for (_, infected_pos) in &infected_positions {
            let dx = infected_pos.x - titan_pos.x;
            let dy = infected_pos.y - titan_pos.y;
            let dist_sq = dx * dx + dy * dy;
            if dist_sq > attack_range_sq {
                continue;
            }

            match best_target {
                None => best_target = Some((*infected_pos, dist_sq)),
                Some((best_pos, best_dist_sq)) => {
                    if dist_sq < best_dist_sq
                        || (dist_sq == best_dist_sq
                            && (infected_pos.x < best_pos.x
                                || (infected_pos.x == best_pos.x && infected_pos.y < best_pos.y)))
                    {
                        best_target = Some((*infected_pos, dist_sq));
                    }
                }
            }
        }

        let Some((impact_pos, _)) = best_target else {
            continue;
        };

        let aoe_sq = profile.aoe_radius * profile.aoe_radius;
        for (infected_entity, infected_pos) in &infected_positions {
            let dx = infected_pos.x - impact_pos.x;
            let dy = infected_pos.y - impact_pos.y;
            let dist_sq = dx * dx + dy * dy;
            if dist_sq <= aoe_sq {
                outgoing_damage.send(IncomingDamageEvent {
                    target: infected_entity,
                    raw_amount: stats.attack_damage,
                    damage_type: DamageType::Standard,
                    source: titan_entity,
                });
            }
        }

        let mut ticks = (sim_hz.0 / stats.attack_speed).to_num::<u32>();
        if ticks == 0 {
            ticks = 1;
        }
        cooldown.ticks_remaining = ticks;
    }
}

pub fn titan_receive_damage_system(
    mut events: EventReader<IncomingDamageEvent>,
    mut units: Query<(&mut Health, &TitanAttackProfile), With<Titan>>,
) {
    for ev in events.read() {
        if let Ok((mut hp, profile)) = units.get_mut(ev.target) {
            let applied = ev.raw_amount * (I32F32::ONE - profile.armor_reduction);
            hp.current = (hp.current - applied).max(I32F32::ZERO);
        }
    }
}

pub fn titan_research_gate_system(
    mut events: EventReader<SetTitanResearchStateEvent>,
    mut units: Query<&mut TitanResearchGate, With<Titan>>,
) {
    for ev in events.read() {
        if let Ok(mut gate) = units.get_mut(ev.target) {
            gate.researched_in_foundry = ev.researched_in_foundry;
        }
    }
}

pub fn titan_recruitment_gate_system(
    mut events: EventReader<SetTitanRecruitmentStateEvent>,
    mut units: Query<&mut TitanRecruitmentGate, With<Titan>>,
) {
    for ev in events.read() {
        if let Ok(mut gate) = units.get_mut(ev.target) {
            gate.engineering_center_online = ev.engineering_center_online;
            gate.oil_income_non_negative = ev.oil_income_non_negative;
        }
    }
}

pub fn titan_narrow_terrain_state_system(
    mut events: EventReader<SetTitanNarrowTerrainEvent>,
    mut units: Query<&mut TitanTerrainState, With<Titan>>,
) {
    for ev in events.read() {
        if let Ok(mut terrain) = units.get_mut(ev.target) {
            terrain.in_narrow_terrain = ev.in_narrow_terrain;
        }
    }
}

pub fn titan_collision_penalty_system(
    mut titans: Query<
        (
            Entity,
            &SimPosition,
            &TitanTerrainState,
            &mut TitanMovementState,
            &mut UnitStats,
        ),
        With<Titan>,
    >,
) {
    let positions: Vec<(Entity, SimPosition, bool)> = titans
        .iter()
        .map(|(entity, pos, terrain, _, _)| (entity, *pos, terrain.in_narrow_terrain))
        .collect();

    let cluster_radius_sq = I32F32::ONE;

    for (entity, pos, terrain, mut movement, mut stats) in &mut titans {
        if !terrain.in_narrow_terrain {
            movement.effective_move_speed = movement.base_move_speed;
            stats.move_speed = movement.effective_move_speed;
            continue;
        }

        let mut nearby_titans = 0_u32;
        for (other_entity, other_pos, other_narrow) in &positions {
            if !*other_narrow || *other_entity == entity {
                continue;
            }
            let dx = other_pos.x - pos.x;
            let dy = other_pos.y - pos.y;
            if dx * dx + dy * dy <= cluster_radius_sq {
                nearby_titans = nearby_titans.saturating_add(1);
            }
        }

        if nearby_titans > 0 {
            movement.effective_move_speed = movement.base_move_speed * TITAN_NARROW_TERRAIN_SPEED_MULT;
        } else {
            movement.effective_move_speed = movement.base_move_speed;
        }

        stats.move_speed = movement.effective_move_speed;
    }
}

pub fn titan_tactical_role_system(
    mut titans: Query<(&SimPosition, &UnitStats, &mut TitanTacticalState), With<Titan>>,
    venoms: Query<&SimPosition, (With<Infected>, With<InfectedVenom>)>,
    giants: Query<&SimPosition, (With<Infected>, With<InfectedGiant>)>,
) {
    for (titan_pos, stats, mut tactical) in &mut titans {
        let watch_sq = stats.watch_range * stats.watch_range;

        let mut venom_nearby = false;
        for venom_pos in &venoms {
            let dx = venom_pos.x - titan_pos.x;
            let dy = venom_pos.y - titan_pos.y;
            if dx * dx + dy * dy <= watch_sq {
                venom_nearby = true;
                break;
            }
        }

        let mut giant_nearby = false;
        for giant_pos in &giants {
            let dx = giant_pos.x - titan_pos.x;
            let dy = giant_pos.y - titan_pos.y;
            if dx * dx + dy * dy <= watch_sq {
                giant_nearby = true;
                break;
            }
        }

        tactical.preference = if venom_nearby {
            TitanRolePreference::AvoidPrimaryTankVsVenom
        } else if giant_nearby {
            TitanRolePreference::TankOrKiteVsGiant
        } else {
            TitanRolePreference::General
        };
    }
}

pub fn titan_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    units: Query<
        (
            &Health,
            &UnitStats,
            &TitanAttackProfile,
            &TitanAttackCooldown,
            &TitanEconomy,
            &TitanResearchGate,
            &TitanRecruitmentGate,
            &TitanTerrainState,
            &TitanMovementState,
            &TitanTacticalState,
        ),
        With<Titan>,
    >,
) {
    for (health, stats, profile, cooldown, economy, research, recruit, terrain, movement, tactical) in &units {
        checksum.accumulate(health.current.to_bits() as u64);
        checksum.accumulate(health.max.to_bits() as u64);

        checksum.accumulate(stats.move_speed.to_bits() as u64);
        checksum.accumulate(stats.attack_range.to_bits() as u64);
        checksum.accumulate(stats.attack_damage.to_bits() as u64);
        checksum.accumulate(stats.attack_speed.to_bits() as u64);
        checksum.accumulate(stats.watch_range.to_bits() as u64);

        checksum.accumulate(profile.armor_reduction.to_bits() as u64);
        checksum.accumulate(profile.attack_noise.to_bits() as u64);
        checksum.accumulate(profile.aoe_radius.to_bits() as u64);

        checksum.accumulate(cooldown.ticks_remaining as u64);

        checksum.accumulate(economy.cost_gold as u64);
        checksum.accumulate(economy.cost_food as u64);
        checksum.accumulate(economy.cost_workers as u64);
        checksum.accumulate(economy.cost_iron as u64);
        checksum.accumulate(economy.cost_oil as u64);
        checksum.accumulate(economy.maintenance_gold as u64);
        checksum.accumulate(economy.maintenance_oil as u64);
        checksum.accumulate(economy.build_time_seconds as u64);
        checksum.accumulate(economy.research_cost_gold as u64);

        checksum.accumulate(u64::from(research.researched_in_foundry));
        checksum.accumulate(u64::from(recruit.engineering_center_online));
        checksum.accumulate(u64::from(recruit.oil_income_non_negative));
        checksum.accumulate(u64::from(terrain.in_narrow_terrain));

        checksum.accumulate(movement.base_move_speed.to_bits() as u64);
        checksum.accumulate(movement.effective_move_speed.to_bits() as u64);

        let tactical_bits = match tactical.preference {
            TitanRolePreference::General => 0_u64,
            TitanRolePreference::AvoidPrimaryTankVsVenom => 1_u64,
            TitanRolePreference::TankOrKiteVsGiant => 2_u64,
        };
        checksum.accumulate(tactical_bits);
    }
}

pub struct TitanPlugin;

impl Plugin for TitanPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SetTitanResearchStateEvent>()
            .add_event::<SetTitanRecruitmentStateEvent>()
            .add_event::<SetTitanNarrowTerrainEvent>()
            .add_systems(
                FixedUpdate,
                (
                    titan_attack_tick_system,
                    titan_receive_damage_system,
                    titan_research_gate_system,
                    titan_recruitment_gate_system,
                    titan_narrow_terrain_state_system,
                    titan_collision_penalty_system,
                    titan_tactical_role_system,
                    titan_checksum_system,
                )
                    .chain(),
            );
    }
}