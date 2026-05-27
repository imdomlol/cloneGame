// Sources: vault/buildings/engineering_center.md

use std::collections::BTreeMap;

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::{Health, SimChecksumState};

const ENGINEERING_CENTER_HP: I32F32 = I32F32::lit("1000");
const ENGINEERING_CENTER_DEFENSES_LIFE: I32F32 = I32F32::lit("500");
const ENGINEERING_CENTER_WATCH_RANGE: I32F32 = I32F32::lit("7");
const ENGINEERING_CENTER_ENERGY_COST: i32 = 50;
const ENGINEERING_CENTER_WOOD_COST: i32 = 0;
const ENGINEERING_CENTER_STONE_COST: i32 = 40;
const ENGINEERING_CENTER_IRON_COST: i32 = 40;
const ENGINEERING_CENTER_OIL_COST: i32 = 20;
const ENGINEERING_CENTER_GOLD_COST: i32 = 2000;
const ENGINEERING_CENTER_BUILD_TIME_SECONDS: i32 = 60;
const ENGINEERING_CENTER_WORKERS: i32 = 30;
const ENGINEERING_CENTER_MAINTENANCE_GOLD_PER_TICK: i32 = 100;
const ENGINEERING_CENTER_SIZE_X: i32 = 3;
const ENGINEERING_CENTER_SIZE_Y: i32 = 3;
const SIM_HZ: i32 = 25;

#[derive(Component, Default)]
pub struct EngineeringCenter;

#[derive(Component, Clone, Copy, Default)]
pub struct BuildingAnchor {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Clone, Copy)]
pub struct EngineeringCenterCore {
    pub defenses_life: I32F32,
    pub health: Health,
}

impl Default for EngineeringCenterCore {
    fn default() -> Self {
        Self {
            defenses_life: ENGINEERING_CENTER_DEFENSES_LIFE,
            health: Health::full(ENGINEERING_CENTER_HP),
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct EngineeringCenterBuildState {
    pub build_ticks_remaining: i32,
    pub completed: bool,
}

impl Default for EngineeringCenterBuildState {
    fn default() -> Self {
        Self {
            build_ticks_remaining: ENGINEERING_CENTER_BUILD_TIME_SECONDS * SIM_HZ,
            completed: false,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct EngineeringCenterEconomy {
    pub energy_cost: i32,
    pub wood_cost: i32,
    pub stone_cost: i32,
    pub iron_cost: i32,
    pub oil_cost: i32,
    pub gold_cost: i32,
    pub workers: i32,
    pub maintenance_gold_per_tick: i32,
}

impl Default for EngineeringCenterEconomy {
    fn default() -> Self {
        Self {
            energy_cost: ENGINEERING_CENTER_ENERGY_COST,
            wood_cost: ENGINEERING_CENTER_WOOD_COST,
            stone_cost: ENGINEERING_CENTER_STONE_COST,
            iron_cost: ENGINEERING_CENTER_IRON_COST,
            oil_cost: ENGINEERING_CENTER_OIL_COST,
            gold_cost: ENGINEERING_CENTER_GOLD_COST,
            workers: ENGINEERING_CENTER_WORKERS,
            maintenance_gold_per_tick: ENGINEERING_CENTER_MAINTENANCE_GOLD_PER_TICK,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct EngineeringCenterFootprint {
    pub size_x: i32,
    pub size_y: i32,
    pub watch_range: I32F32,
}

impl Default for EngineeringCenterFootprint {
    fn default() -> Self {
        Self {
            size_x: ENGINEERING_CENTER_SIZE_X,
            size_y: ENGINEERING_CENTER_SIZE_Y,
            watch_range: ENGINEERING_CENTER_WATCH_RANGE,
        }
    }
}

#[derive(Component, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EngineeringCenterUnit {
    Lucifer,
    Thanatos,
    Titan,
    Mutant,
}

#[derive(Component, Default, Clone)]
pub struct EngineeringCenterResearch {
    pub unlocked: BTreeMap<EngineeringCenterUnit, bool>,
}

#[derive(Component, Clone, Copy, Default)]
pub struct EngineeringCenterBehavior {
    pub blocks_hiring_on_negative_oil_income: bool,
    pub units_do_not_desert_on_negative_oil_income: bool,
}

#[derive(Resource, Clone, Copy)]
pub struct EngineeringCenterOilIncome {
    pub positive_income: bool,
}

impl Default for EngineeringCenterOilIncome {
    fn default() -> Self {
        Self {
            positive_income: true,
        }
    }
}

#[derive(Resource, Default, Clone)]
pub struct EngineeringCenterPlacementClaims {
    pub claims: BTreeMap<Entity, BuildingAnchor>,
}

#[derive(Event, Clone, Copy)]
pub struct PlaceEngineeringCenterEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct EngineeringCenterPlacementRejectedEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct SetEngineeringCenterResearchEvent {
    pub center_entity: Entity,
    pub unit: EngineeringCenterUnit,
    pub researched: bool,
}

#[derive(Event, Clone, Copy)]
pub struct SetEngineeringCenterOilIncomeEvent {
    pub positive_income: bool,
}

#[derive(Event, Clone, Copy)]
pub struct TryRecruitFromEngineeringCenterEvent {
    pub center_entity: Entity,
    pub unit: EngineeringCenterUnit,
}

#[derive(Event, Clone, Copy)]
pub struct EngineeringCenterRecruitmentApprovedEvent {
    pub center_entity: Entity,
    pub unit: EngineeringCenterUnit,
}

#[derive(Event, Clone, Copy)]
pub struct EngineeringCenterRecruitmentRejectedEvent {
    pub center_entity: Entity,
    pub unit: EngineeringCenterUnit,
    pub reason_code: i32,
}

#[derive(Event, Clone, Copy)]
pub struct DamageEngineeringCenterEvent {
    pub center_entity: Entity,
    pub damage: I32F32,
}

fn place_engineering_center_system(
    mut commands: Commands,
    mut events: EventReader<PlaceEngineeringCenterEvent>,
    mut rejected: EventWriter<EngineeringCenterPlacementRejectedEvent>,
    mut claims: ResMut<EngineeringCenterPlacementClaims>,
) {
    for ev in events.read() {
        let anchor = BuildingAnchor {
            x: ev.tile_x,
            y: ev.tile_y,
        };

        if claims
            .claims
            .values()
            .any(|existing| existing.x == anchor.x && existing.y == anchor.y)
        {
            rejected.send(EngineeringCenterPlacementRejectedEvent {
                tile_x: ev.tile_x,
                tile_y: ev.tile_y,
            });
            continue;
        }

        let mut unlocked = BTreeMap::new();
        unlocked.insert(EngineeringCenterUnit::Lucifer, true);
        unlocked.insert(EngineeringCenterUnit::Thanatos, false);
        unlocked.insert(EngineeringCenterUnit::Titan, false);
        unlocked.insert(EngineeringCenterUnit::Mutant, false);

        let entity = commands
            .spawn((
                EngineeringCenter,
                anchor,
                EngineeringCenterCore::default(),
                EngineeringCenterBuildState::default(),
                EngineeringCenterEconomy::default(),
                EngineeringCenterFootprint::default(),
                EngineeringCenterResearch { unlocked },
                EngineeringCenterBehavior {
                    blocks_hiring_on_negative_oil_income: true,
                    units_do_not_desert_on_negative_oil_income: true,
                },
            ))
            .id();

        claims.claims.insert(entity, anchor);
    }
}

fn engineering_center_build_tick_system(
    mut centers: Query<&mut EngineeringCenterBuildState, With<EngineeringCenter>>,
) {
    for mut build in &mut centers {
        if build.completed {
            continue;
        }

        if build.build_ticks_remaining > 0 {
            build.build_ticks_remaining -= 1;
        }

        if build.build_ticks_remaining <= 0 {
            build.build_ticks_remaining = 0;
            build.completed = true;
        }
    }
}

fn set_engineering_center_research_system(
    mut events: EventReader<SetEngineeringCenterResearchEvent>,
    mut centers: Query<&mut EngineeringCenterResearch, With<EngineeringCenter>>,
) {
    for ev in events.read() {
        let Ok(mut research) = centers.get_mut(ev.center_entity) else {
            continue;
        };
        research.unlocked.insert(ev.unit, ev.researched);
    }
}

fn set_engineering_center_oil_income_system(
    mut events: EventReader<SetEngineeringCenterOilIncomeEvent>,
    mut oil_income: ResMut<EngineeringCenterOilIncome>,
) {
    for ev in events.read() {
        oil_income.positive_income = ev.positive_income;
    }
}

fn try_recruit_from_engineering_center_system(
    mut attempts: EventReader<TryRecruitFromEngineeringCenterEvent>,
    mut approved: EventWriter<EngineeringCenterRecruitmentApprovedEvent>,
    mut rejected: EventWriter<EngineeringCenterRecruitmentRejectedEvent>,
    oil_income: Res<EngineeringCenterOilIncome>,
    centers: Query<
        (
            &EngineeringCenterBuildState,
            &EngineeringCenterResearch,
            &EngineeringCenterBehavior,
        ),
        With<EngineeringCenter>,
    >,
) {
    for ev in attempts.read() {
        let Ok((build, research, behavior)) = centers.get(ev.center_entity) else {
            rejected.send(EngineeringCenterRecruitmentRejectedEvent {
                center_entity: ev.center_entity,
                unit: ev.unit,
                reason_code: 1,
            });
            continue;
        };

        if !build.completed {
            rejected.send(EngineeringCenterRecruitmentRejectedEvent {
                center_entity: ev.center_entity,
                unit: ev.unit,
                reason_code: 2,
            });
            continue;
        }

        if behavior.blocks_hiring_on_negative_oil_income && !oil_income.positive_income {
            rejected.send(EngineeringCenterRecruitmentRejectedEvent {
                center_entity: ev.center_entity,
                unit: ev.unit,
                reason_code: 3,
            });
            continue;
        }

        let unlocked = research.unlocked.get(&ev.unit).copied().unwrap_or(false);
        if !unlocked {
            rejected.send(EngineeringCenterRecruitmentRejectedEvent {
                center_entity: ev.center_entity,
                unit: ev.unit,
                reason_code: 4,
            });
            continue;
        }

        approved.send(EngineeringCenterRecruitmentApprovedEvent {
            center_entity: ev.center_entity,
            unit: ev.unit,
        });
    }
}

fn damage_engineering_center_system(
    mut commands: Commands,
    mut events: EventReader<DamageEngineeringCenterEvent>,
    mut centers: Query<(Entity, &mut EngineeringCenterCore), With<EngineeringCenter>>,
    mut claims: ResMut<EngineeringCenterPlacementClaims>,
) {
    for ev in events.read() {
        let Ok((entity, mut core)) = centers.get_mut(ev.center_entity) else {
            continue;
        };

        let mut incoming = ev.damage;
        if incoming <= I32F32::ZERO {
            continue;
        }

        if core.defenses_life > I32F32::ZERO {
            let absorbed = incoming.min(core.defenses_life);
            core.defenses_life -= absorbed;
            incoming -= absorbed;
        }

        if incoming > I32F32::ZERO {
            core.health.current -= incoming;
        }

        if core.health.current > I32F32::ZERO {
            continue;
        }

        core.health.current = I32F32::ZERO;
        claims.claims.remove(&entity);
        commands.entity(entity).despawn();
    }
}

fn engineering_center_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    centers: Query<
        (
            Entity,
            &BuildingAnchor,
            &EngineeringCenterCore,
            &EngineeringCenterBuildState,
            &EngineeringCenterEconomy,
            &EngineeringCenterFootprint,
            &EngineeringCenterResearch,
            &EngineeringCenterBehavior,
        ),
        With<EngineeringCenter>,
    >,
    claims: Res<EngineeringCenterPlacementClaims>,
    oil_income: Res<EngineeringCenterOilIncome>,
) {
    for (entity, anchor, core, build, economy, footprint, research, behavior) in &centers {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);

        checksum.accumulate(core.defenses_life.to_bits() as u64);
        checksum.accumulate(core.health.current.to_bits() as u64);
        checksum.accumulate(core.health.max.to_bits() as u64);

        checksum.accumulate(build.build_ticks_remaining as u64);
        checksum.accumulate(u64::from(build.completed));

        checksum.accumulate(economy.energy_cost as u64);
        checksum.accumulate(economy.wood_cost as u64);
        checksum.accumulate(economy.stone_cost as u64);
        checksum.accumulate(economy.iron_cost as u64);
        checksum.accumulate(economy.oil_cost as u64);
        checksum.accumulate(economy.gold_cost as u64);
        checksum.accumulate(economy.workers as u64);
        checksum.accumulate(economy.maintenance_gold_per_tick as u64);

        checksum.accumulate(footprint.size_x as u64);
        checksum.accumulate(footprint.size_y as u64);
        checksum.accumulate(footprint.watch_range.to_bits() as u64);

        checksum.accumulate(research.unlocked.len() as u64);
        for (unit, unlocked) in &research.unlocked {
            let unit_id = match unit {
                EngineeringCenterUnit::Lucifer => 1_u64,
                EngineeringCenterUnit::Thanatos => 2_u64,
                EngineeringCenterUnit::Titan => 3_u64,
                EngineeringCenterUnit::Mutant => 4_u64,
            };
            checksum.accumulate(unit_id);
            checksum.accumulate(u64::from(*unlocked));
        }

        checksum.accumulate(u64::from(
            behavior.blocks_hiring_on_negative_oil_income,
        ));
        checksum.accumulate(u64::from(
            behavior.units_do_not_desert_on_negative_oil_income,
        ));
    }

    checksum.accumulate(claims.claims.len() as u64);
    for (entity, anchor) in &claims.claims {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);
    }

    checksum.accumulate(u64::from(oil_income.positive_income));
}

pub struct EngineeringCenterPlugin;

impl Plugin for EngineeringCenterPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EngineeringCenterPlacementClaims>()
            .init_resource::<EngineeringCenterOilIncome>()
            .add_event::<PlaceEngineeringCenterEvent>()
            .add_event::<EngineeringCenterPlacementRejectedEvent>()
            .add_event::<SetEngineeringCenterResearchEvent>()
            .add_event::<SetEngineeringCenterOilIncomeEvent>()
            .add_event::<TryRecruitFromEngineeringCenterEvent>()
            .add_event::<EngineeringCenterRecruitmentApprovedEvent>()
            .add_event::<EngineeringCenterRecruitmentRejectedEvent>()
            .add_event::<DamageEngineeringCenterEvent>()
            .add_systems(
                FixedUpdate,
                (
                    place_engineering_center_system,
                    engineering_center_build_tick_system,
                    set_engineering_center_research_system,
                    set_engineering_center_oil_income_system,
                    try_recruit_from_engineering_center_system,
                    damage_engineering_center_system,
                    engineering_center_checksum_system,
                )
                    .chain(),
            );
    }
}