// Sources: vault/buildings/cottage.md, vault/buildings/tent.md

use std::collections::BTreeMap;

use bevy::prelude::*;
use fixed::types::I32F32;

use crate::sim::SimChecksumState;

const COTTAGE_HP: I32F32 = I32F32::lit("200");
const COTTAGE_DEFENSES_LIFE: I32F32 = I32F32::lit("50");
const COTTAGE_WATCH_RANGE: I32F32 = I32F32::lit("6");

const COTTAGE_ENERGY_COST: i32 = 3;
const COTTAGE_WOOD_COST: i32 = 12;
const COTTAGE_GOLD_COST: i32 = 120;
const COTTAGE_WORKERS_USED: i32 = 4;
const COTTAGE_PCOLONISTS: i32 = 8;
const COTTAGE_PGOLD: i32 = 18;

const COTTAGE_UPGRADE_GOLD_COST: i32 = 90;
const COTTAGE_UPGRADE_BUILD_TIME_SECONDS: i32 = 14;
const COTTAGE_BUILD_TIME_SECONDS: i32 = 36;
const COTTAGE_SIZE_TILES: i32 = 2;

const COTTAGE_BANK_BONUS_SELF: i32 = 5;
const COTTAGE_BANK_BONUS_TWO_TENTS: i32 = 4;
const COTTAGE_GOLD_INTERVAL_HOURS: i32 = 8;

const SIM_HZ: i32 = 25;
const SECONDS_PER_HOUR: i32 = 3600;

#[derive(Component, Default)]
pub struct Tent;

#[derive(Component, Default)]
pub struct Cottage;

#[derive(Component, Clone, Copy)]
pub struct BuildingAnchor {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Clone, Copy)]
pub struct BuildingHealth {
    pub hp: I32F32,
    pub defenses_life: I32F32,
}

impl Default for BuildingHealth {
    fn default() -> Self {
        Self {
            hp: COTTAGE_HP,
            defenses_life: COTTAGE_DEFENSES_LIFE,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct CottageBuildState {
    pub build_ticks_remaining: i32,
    pub upgrading_from_tent: bool,
    pub completed: bool,
}

#[derive(Component, Clone, Copy)]
pub struct CottageEconomy {
    pub energy_cost: i32,
    pub wood_cost: i32,
    pub gold_cost: i32,
    pub workers_used: i32,
    pub pcolonists: i32,
    pub pgold: i32,
    pub gold_interval_ticks: i32,
    pub upgrade_gold_cost: i32,
    pub upgrade_build_ticks: i32,
}

impl Default for CottageEconomy {
    fn default() -> Self {
        Self {
            energy_cost: COTTAGE_ENERGY_COST,
            wood_cost: COTTAGE_WOOD_COST,
            gold_cost: COTTAGE_GOLD_COST,
            workers_used: COTTAGE_WORKERS_USED,
            pcolonists: COTTAGE_PCOLONISTS,
            pgold: COTTAGE_PGOLD,
            gold_interval_ticks: hours_to_ticks(COTTAGE_GOLD_INTERVAL_HOURS),
            upgrade_gold_cost: COTTAGE_UPGRADE_GOLD_COST,
            upgrade_build_ticks: seconds_to_ticks(COTTAGE_UPGRADE_BUILD_TIME_SECONDS),
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct CottageIncomeState {
    pub ticks_until_next_gold: i32,
    pub stored_gold: i32,
}

impl Default for CottageIncomeState {
    fn default() -> Self {
        Self {
            ticks_until_next_gold: hours_to_ticks(COTTAGE_GOLD_INTERVAL_HOURS),
            stored_gold: 0,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct CottageCampaignModifiers {
    pub in_the_new_empire_campaign: bool,
    pub has_insulating_materials: bool,
    pub has_solar_energy: bool,
}

impl Default for CottageCampaignModifiers {
    fn default() -> Self {
        Self {
            in_the_new_empire_campaign: false,
            has_insulating_materials: false,
            has_solar_energy: false,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct CottageBankInfluence {
    pub influenced_cottages: i32,
    pub influenced_tents: i32,
}

impl Default for CottageBankInfluence {
    fn default() -> Self {
        Self {
            influenced_cottages: 0,
            influenced_tents: 0,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct CottageDerivedEconomy {
    pub effective_energy_cost: i32,
    pub effective_gold_income: i32,
}

impl Default for CottageDerivedEconomy {
    fn default() -> Self {
        Self {
            effective_energy_cost: COTTAGE_ENERGY_COST,
            effective_gold_income: COTTAGE_PGOLD,
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct CottageFootprint {
    pub size_tiles: i32,
    pub watch_range: I32F32,
}

impl Default for CottageFootprint {
    fn default() -> Self {
        Self {
            size_tiles: COTTAGE_SIZE_TILES,
            watch_range: COTTAGE_WATCH_RANGE,
        }
    }
}

#[derive(Resource, Default, Clone)]
pub struct CottagePlacementClaims {
    pub claims: BTreeMap<Entity, BuildingAnchor>,
}

#[derive(Event, Clone, Copy)]
pub struct PlaceCottageEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct CottagePlacementRejectedEvent {
    pub tile_x: i32,
    pub tile_y: i32,
}

#[derive(Event, Clone, Copy)]
pub struct UpgradeTentToCottageEvent {
    pub tent_entity: Entity,
}

#[derive(Event, Clone, Copy)]
pub struct SetCottageCampaignModifiersEvent {
    pub cottage_entity: Entity,
    pub in_the_new_empire_campaign: bool,
    pub has_insulating_materials: bool,
    pub has_solar_energy: bool,
}

#[derive(Event, Clone, Copy)]
pub struct SetCottageBankInfluenceEvent {
    pub cottage_entity: Entity,
    pub influenced_cottages: i32,
    pub influenced_tents: i32,
}

fn seconds_to_ticks(seconds: i32) -> i32 {
    seconds * SIM_HZ
}

fn hours_to_ticks(hours: i32) -> i32 {
    hours * SECONDS_PER_HOUR * SIM_HZ
}

fn place_cottage_system(
    mut commands: Commands,
    mut events: EventReader<PlaceCottageEvent>,
    mut rejected: EventWriter<CottagePlacementRejectedEvent>,
    mut claims: ResMut<CottagePlacementClaims>,
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
            rejected.send(CottagePlacementRejectedEvent {
                tile_x: ev.tile_x,
                tile_y: ev.tile_y,
            });
            continue;
        }

        let entity = commands
            .spawn((
                Cottage,
                anchor,
                BuildingHealth::default(),
                CottageBuildState {
                    build_ticks_remaining: seconds_to_ticks(COTTAGE_BUILD_TIME_SECONDS),
                    upgrading_from_tent: false,
                    completed: false,
                },
                CottageEconomy::default(),
                CottageIncomeState::default(),
                CottageCampaignModifiers::default(),
                CottageBankInfluence::default(),
                CottageDerivedEconomy::default(),
                CottageFootprint::default(),
            ))
            .id();

        claims.claims.insert(entity, anchor);
    }
}

fn upgrade_tent_to_cottage_system(
    mut commands: Commands,
    mut events: EventReader<UpgradeTentToCottageEvent>,
    tents: Query<(Entity, &BuildingAnchor), With<Tent>>,
    mut claims: ResMut<CottagePlacementClaims>,
) {
    for ev in events.read() {
        let Ok((tent_entity, anchor)) = tents.get(ev.tent_entity) else {
            continue;
        };

        commands.entity(tent_entity).remove::<Tent>();
        commands.entity(tent_entity).insert((
            Cottage,
            *anchor,
            BuildingHealth::default(),
            CottageBuildState {
                build_ticks_remaining: seconds_to_ticks(COTTAGE_UPGRADE_BUILD_TIME_SECONDS),
                upgrading_from_tent: true,
                completed: false,
            },
            CottageEconomy::default(),
            CottageIncomeState::default(),
            CottageCampaignModifiers::default(),
            CottageBankInfluence::default(),
            CottageDerivedEconomy::default(),
            CottageFootprint::default(),
        ));

        claims.claims.insert(tent_entity, *anchor);
    }
}

fn set_cottage_campaign_modifiers_system(
    mut events: EventReader<SetCottageCampaignModifiersEvent>,
    mut cottages: Query<&mut CottageCampaignModifiers, With<Cottage>>,
) {
    for ev in events.read() {
        let Ok(mut modifiers) = cottages.get_mut(ev.cottage_entity) else {
            continue;
        };

        modifiers.in_the_new_empire_campaign = ev.in_the_new_empire_campaign;
        modifiers.has_insulating_materials = ev.has_insulating_materials;
        modifiers.has_solar_energy = ev.has_solar_energy;
    }
}

fn set_cottage_bank_influence_system(
    mut events: EventReader<SetCottageBankInfluenceEvent>,
    mut cottages: Query<&mut CottageBankInfluence, With<Cottage>>,
) {
    for ev in events.read() {
        let Ok(mut influence) = cottages.get_mut(ev.cottage_entity) else {
            continue;
        };

        influence.influenced_cottages = ev.influenced_cottages;
        influence.influenced_tents = ev.influenced_tents;
    }
}

fn cottage_build_tick_system(mut cottages: Query<&mut CottageBuildState, With<Cottage>>) {
    for mut state in &mut cottages {
        if state.completed {
            continue;
        }

        if state.build_ticks_remaining > 0 {
            state.build_ticks_remaining -= 1;
        }

        if state.build_ticks_remaining <= 0 {
            state.build_ticks_remaining = 0;
            state.completed = true;
        }
    }
}

fn cottage_derived_economy_system(
    mut cottages: Query<
        (
            &CottageCampaignModifiers,
            &CottageBankInfluence,
            &mut CottageDerivedEconomy,
        ),
        With<Cottage>,
    >,
) {
    for (campaign, bank, mut derived) in &mut cottages {
        let mut effective_energy_cost = COTTAGE_ENERGY_COST;
        if campaign.in_the_new_empire_campaign
            && campaign.has_insulating_materials
            && campaign.has_solar_energy
        {
            effective_energy_cost -= 2;
        }

        let mut effective_gold_income = COTTAGE_PGOLD;
        if bank.influenced_cottages >= 1 {
            effective_gold_income += COTTAGE_BANK_BONUS_SELF;
        }
        if bank.influenced_tents >= 2 {
            effective_gold_income += COTTAGE_BANK_BONUS_TWO_TENTS;
        }

        derived.effective_energy_cost = effective_energy_cost;
        derived.effective_gold_income = effective_gold_income;
    }
}

fn cottage_gold_income_tick_system(
    mut cottages: Query<
        (
            &CottageBuildState,
            &CottageEconomy,
            &CottageDerivedEconomy,
            &mut CottageIncomeState,
        ),
        With<Cottage>,
    >,
) {
    for (build, economy, derived, mut income) in &mut cottages {
        if !build.completed {
            continue;
        }

        if income.ticks_until_next_gold > 0 {
            income.ticks_until_next_gold -= 1;
        }

        if income.ticks_until_next_gold <= 0 {
            income.ticks_until_next_gold = economy.gold_interval_ticks;
            income.stored_gold += derived.effective_gold_income;
        }
    }
}

fn cottage_checksum_system(
    mut checksum: ResMut<SimChecksumState>,
    cottages: Query<
        (
            Entity,
            &BuildingAnchor,
            &BuildingHealth,
            &CottageBuildState,
            &CottageEconomy,
            &CottageIncomeState,
            &CottageCampaignModifiers,
            &CottageBankInfluence,
            &CottageDerivedEconomy,
            &CottageFootprint,
        ),
        With<Cottage>,
    >,
    claims: Res<CottagePlacementClaims>,
) {
    for (
        entity,
        anchor,
        health,
        build,
        economy,
        income,
        campaign,
        bank,
        derived,
        footprint,
    ) in &cottages
    {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);

        checksum.accumulate(health.hp.to_bits() as u64);
        checksum.accumulate(health.defenses_life.to_bits() as u64);

        checksum.accumulate(build.build_ticks_remaining as u64);
        checksum.accumulate(u64::from(build.upgrading_from_tent));
        checksum.accumulate(u64::from(build.completed));

        checksum.accumulate(economy.energy_cost as u64);
        checksum.accumulate(economy.wood_cost as u64);
        checksum.accumulate(economy.gold_cost as u64);
        checksum.accumulate(economy.workers_used as u64);
        checksum.accumulate(economy.pcolonists as u64);
        checksum.accumulate(economy.pgold as u64);
        checksum.accumulate(economy.gold_interval_ticks as u64);
        checksum.accumulate(economy.upgrade_gold_cost as u64);
        checksum.accumulate(economy.upgrade_build_ticks as u64);

        checksum.accumulate(income.ticks_until_next_gold as u64);
        checksum.accumulate(income.stored_gold as u64);

        checksum.accumulate(u64::from(campaign.in_the_new_empire_campaign));
        checksum.accumulate(u64::from(campaign.has_insulating_materials));
        checksum.accumulate(u64::from(campaign.has_solar_energy));

        checksum.accumulate(bank.influenced_cottages as u64);
        checksum.accumulate(bank.influenced_tents as u64);

        checksum.accumulate(derived.effective_energy_cost as u64);
        checksum.accumulate(derived.effective_gold_income as u64);

        checksum.accumulate(footprint.size_tiles as u64);
        checksum.accumulate(footprint.watch_range.to_bits() as u64);
    }

    checksum.accumulate(claims.claims.len() as u64);
    for (entity, anchor) in &claims.claims {
        checksum.accumulate(entity.to_bits() as u64);
        checksum.accumulate(anchor.x as u64);
        checksum.accumulate(anchor.y as u64);
    }
}

pub struct CottagePlugin;

impl Plugin for CottagePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CottagePlacementClaims>()
            .add_event::<PlaceCottageEvent>()
            .add_event::<CottagePlacementRejectedEvent>()
            .add_event::<UpgradeTentToCottageEvent>()
            .add_event::<SetCottageCampaignModifiersEvent>()
            .add_event::<SetCottageBankInfluenceEvent>()
            .add_systems(
                FixedUpdate,
                (
                    place_cottage_system,
                    upgrade_tent_to_cottage_system,
                    set_cottage_campaign_modifiers_system,
                    set_cottage_bank_influence_system,
                    cottage_build_tick_system,
                    cottage_derived_economy_system,
                    cottage_gold_income_tick_system,
                    cottage_checksum_system,
                )
                    .chain(),
            );
    }
}