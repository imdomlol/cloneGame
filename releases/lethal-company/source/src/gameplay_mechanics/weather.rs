// Sources: vault/gameplay_mechanics/weather.md, vault/destination_pages/8_titan.md
use bevy::prelude::*;
use fixed::types::I16F16;
use rand_core::RngCore;

use crate::sim::{tick_rng, GameSeed, SimChecksumState, SimTick};

pub const WEATHER_ID: &str = "weather";
pub const WEATHER_NAME: &str = "Weather";
pub const WEATHER_TYPE: &str = "gameplay_mechanics";
pub const WEATHER_SUBTYPE: &str = "weather";
pub const WEATHER_SOURCE_URL: &str = "https://lethal-company.fandom.com/wiki/Weather";
pub const WEATHER_SOURCE_REVISION: u32 = 21461;
pub const WEATHER_EXTRACTED_AT: &str = "2026-06-07T00:00:00Z";
pub const WEATHER_CONFIDENCE_BASIS_POINTS: u16 = 87;

pub const WEATHER_DEPENDS_ON: [&str; 0] = [];

pub const WEATHER_OVERVIEW: &str = "Weather is a daily environmental system that changes outdoor conditions on a moon. It can add hazards, visibility loss, or entity pressure, but it does not change scrap amount or value.";

pub const WEATHER_RULES: [&str; 4] = [
    "At the start of each in-game day, every moon rolls weather independently.",
    "The Exomoons catalogue shows the forecast for each moon.",
    "Weather can affect multiple moons on the same day.",
    "Available weather states include Clear, Rainy, Stormy, Foggy, Flooded, Eclipsed, and the Meteor Shower event.",
];

pub const WEATHER_MODIFIERS: [&str; 6] = [
    "Rainy can create outdoor quicksand patches that slow and can sink crew members within a few seconds.",
    "Stormy can let conductive outdoor items build charge and be struck by lightning after a few moments.",
    "Foggy can reduce outdoor visibility and make navigation harder.",
    "Flooded can raise water levels, slow movement, drain stamina, and drown fully submerged crew members.",
    "Eclipsed can increase early-day entity pressure by spawning outdoor entities immediately and starting indoor spawns at night levels.",
    "Meteor Shower can add a separate surface hazard that can occur during any current weather state.",
];

pub const WEATHER_STRATEGY: [&str; 5] = [
    "Check forecasts before landing and avoid risky moons when the crew is inexperienced.",
    "Keep movement routes simple during Rainy or Flooded conditions so retreat paths stay reliable.",
    "Minimize carried conductive loot during Stormy weather so the crew can drop fewer items if lightning targets them.",
    "Treat Foggy weather as a navigation problem first and memorize routes before committing deep into the map.",
    "Delay landing on Eclipsed moons unless the crew can coordinate tightly and keep one person safe.",
];

pub const WEATHER_NOTES: [&str; 8] = [
    "Weather does not affect scrap amount or scrap value.",
    "The Meteor Shower event is not a normal daily weather roll and can begin at a random time during the day.",
    "The weather selection count uses a random value from 0.0 to 1.0 and may be multiplied by a factor from 1.5 to 2.5 when the crew has at least 2 employees and the no-death streak is a multiple of 3 days.",
    "The selected moon count is converted to an integer and clamped between 0 and the total number of moons.",
    "A moon's specific weather is chosen by random index from that moon's available weather list.",
    "Conductive items bought on a Stormy moon do not attract lightning for the rest of that purchase day.",
    "Some moons use reversed flood behavior, with water starting high and falling over time.",
    "Titan supports Foggy, Stormy, and Eclipsed weather.",
];

pub const WEATHER_BEHAVIORAL_MECHANICS: [WeatherBehaviorRule; 22] = [
    WeatherBehaviorRule {
        condition: "the day starts",
        outcome: "each moon rolls weather from a random input value between 0.0 and 1.0",
    },
    WeatherBehaviorRule {
        condition: "the crew has at least 2 employees and the no-death streak is a multiple of 3 days",
        outcome: "the weather-affected moon count is multiplied by a random factor from 1.5 to 2.5",
    },
    WeatherBehaviorRule {
        condition: "the calculated moon count is below 0 or above the total moon count",
        outcome: "it is clamped to the 0-to-total range",
    },
    WeatherBehaviorRule {
        condition: "a moon is selected for weather",
        outcome: "one weather type is chosen by random index from that moon's valid weather list",
    },
    WeatherBehaviorRule {
        condition: "weather is Rainy",
        outcome: "dark quicksand patches can appear outside and can sink an unresponsive crew member within a few seconds",
    },
    WeatherBehaviorRule {
        condition: "a crew member hears the mud audio cue during Rainy weather",
        outcome: "backing out quickly can let them escape before sinking",
    },
    WeatherBehaviorRule {
        condition: "weather is Stormy and an outdoor conductive item starts buzzing and arcing",
        outcome: "it is charging toward a lightning strike",
    },
    WeatherBehaviorRule {
        condition: "a crew member is holding a sparking conductive item during Stormy weather",
        outcome: "dropping it immediately and moving away can prevent death",
    },
    WeatherBehaviorRule {
        condition: "lightning strikes a conductive object",
        outcome: "it can strike again multiple times in quick succession",
    },
    WeatherBehaviorRule {
        condition: "a conductive item is under cover during Stormy weather",
        outcome: "lightning is more likely to hit the cover or roof instead of the item",
    },
    WeatherBehaviorRule {
        condition: "weather is Foggy",
        outcome: "visibility for outdoor entities that cannot see through fog is limited to 30 units",
    },
    WeatherBehaviorRule {
        condition: "weather is Foggy",
        outcome: "navigation becomes harder and route memory matters more than sight",
    },
    WeatherBehaviorRule {
        condition: "weather is Flooded",
        outcome: "low areas become harder and eventually impossible to traverse as water rises",
    },
    WeatherBehaviorRule {
        condition: "crew members wade through Flooded terrain",
        outcome: "their movement slows, stamina drains, and running is disabled",
    },
    WeatherBehaviorRule {
        condition: "a crew member is fully underwater in Flooded weather for long enough",
        outcome: "they drown",
    },
    WeatherBehaviorRule {
        condition: "weather is Eclipsed",
        outcome: "outdoor entities can spawn immediately and indoor spawn pressure begins at night levels",
    },
    WeatherBehaviorRule {
        condition: "weather is Eclipsed and the crew is inexperienced",
        outcome: "the moon is a high-risk landing choice",
    },
    WeatherBehaviorRule {
        condition: "a Meteor Shower is imminent",
        outcome: "an alert appears on the HUD before the waves arrive",
    },
    WeatherBehaviorRule {
        condition: "a meteor lands on a tile occupied by a target",
        outcome: "that target dies and nearby crew members are knocked back without damage",
    },
    WeatherBehaviorRule {
        condition: "a meteor impact occurs",
        outcome: "the impact leaves only a char decal and no crater",
    },
    WeatherBehaviorRule {
        condition: "a Meteor Shower starts",
        outcome: "it lasts 12 in-game hours, which is about 8.5 real-time minutes",
    },
    WeatherBehaviorRule {
        condition: "a conductive item was ordered on a Stormy moon",
        outcome: "it does not attract lightning for the rest of that purchase day",
    },
];

pub const TITAN_ID: &str = "8_titan";
pub const TITAN_NAME: &str = "8-Titan";
pub const TITAN_SOURCE_REVISION: u32 = 21454;
pub const TITAN_VALID_WEATHER: [WeatherKind; 3] = [
    WeatherKind::Foggy,
    WeatherKind::Stormy,
    WeatherKind::Eclipsed,
];

pub const FOGGY_OUTDOOR_VISIBILITY_UNITS: i32 = 30;
pub const METEOR_SHOWER_DURATION_INGAME_HOURS: i32 = 12;
pub const METEOR_SHOWER_DURATION_REALTIME_SECONDS_TENTHS: u32 = 5100;
pub const TITAN_ECLIPSED_START_INDOOR_ENTITIES: u8 = 3;
pub const TITAN_ECLIPSED_START_OUTDOOR_ENTITIES: u8 = 3;

pub struct WeatherPlugin;

impl Plugin for WeatherPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WeatherState>()
            .add_event::<WeatherDayStartedEvent>()
            .add_event::<WeatherForecastResolvedEvent>()
            .add_event::<MeteorShowerImminentEvent>()
            .add_event::<MeteorImpactEvent>()
            .add_systems(
                FixedUpdate,
                (
                    weather_roll_day_forecasts,
                    weather_resolve_meteor_impacts,
                    weather_checksum,
                )
                    .chain(),
            );
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WeatherBehaviorRule {
    pub condition: &'static str,
    pub outcome: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WeatherKind {
    Clear,
    Rainy,
    Stormy,
    Foggy,
    Flooded,
    Eclipsed,
}

impl WeatherKind {
    pub const fn id(self) -> &'static str {
        match self {
            Self::Clear => "clear",
            Self::Rainy => "rainy",
            Self::Stormy => "stormy",
            Self::Foggy => "foggy",
            Self::Flooded => "flooded",
            Self::Eclipsed => "eclipsed",
        }
    }

    pub const fn checksum_value(self) -> u64 {
        match self {
            Self::Clear => 0,
            Self::Rainy => 1,
            Self::Stormy => 2,
            Self::Foggy => 3,
            Self::Flooded => 4,
            Self::Eclipsed => 5,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MoonWeatherForecast {
    pub moon_id: &'static str,
    pub weather: WeatherKind,
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
pub struct WeatherState {
    pub current_day: u32,
    pub employee_count: u32,
    pub no_death_streak_days: u32,
    pub total_moon_count: u32,
    pub affected_moon_count: u32,
    pub titan_forecast: WeatherKind,
    pub meteor_shower_active: bool,
    pub meteor_shower_started_tick: u64,
    pub meteor_shower_imminent_alerts: u64,
    pub meteor_impacts: u64,
    pub meteor_kills: u64,
    pub meteor_knockbacks: u64,
    pub conductive_purchase_protection_day: u32,
}

impl Default for WeatherState {
    fn default() -> Self {
        Self {
            current_day: 0,
            employee_count: 1,
            no_death_streak_days: 0,
            total_moon_count: 1,
            affected_moon_count: 0,
            titan_forecast: WeatherKind::Clear,
            meteor_shower_active: false,
            meteor_shower_started_tick: 0,
            meteor_shower_imminent_alerts: 0,
            meteor_impacts: 0,
            meteor_kills: 0,
            meteor_knockbacks: 0,
            conductive_purchase_protection_day: 0,
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct WeatherDayStartedEvent {
    pub day: u32,
    pub employee_count: u32,
    pub no_death_streak_days: u32,
    pub total_moon_count: u32,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct WeatherForecastResolvedEvent {
    pub day: u32,
    pub affected_moon_count: u32,
    pub titan_forecast: WeatherKind,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct MeteorShowerImminentEvent {
    pub day: u32,
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct MeteorImpactEvent {
    pub tile_id: u64,
    pub occupied_target_id: Option<u64>,
    pub nearby_crew_count: u32,
}

pub fn weather_valid_states() -> [WeatherKind; 6] {
    [
        WeatherKind::Clear,
        WeatherKind::Rainy,
        WeatherKind::Stormy,
        WeatherKind::Foggy,
        WeatherKind::Flooded,
        WeatherKind::Eclipsed,
    ]
}

pub fn titan_valid_weather() -> &'static [WeatherKind] {
    &TITAN_VALID_WEATHER
}

pub fn foggy_outdoor_visibility_units() -> I16F16 {
    I16F16::from_num(FOGGY_OUTDOOR_VISIBILITY_UNITS)
}

pub fn meteor_shower_duration_ingame_hours() -> I16F16 {
    I16F16::from_num(METEOR_SHOWER_DURATION_INGAME_HOURS)
}

pub fn meteor_shower_duration_realtime_seconds() -> I16F16 {
    I16F16::from_num(METEOR_SHOWER_DURATION_REALTIME_SECONDS_TENTHS)
        / I16F16::from_num(10)
}

pub fn stormy_purchase_is_lightning_protected(
    purchase_day: u32,
    current_day: u32,
    weather: WeatherKind,
) -> bool {
    weather == WeatherKind::Stormy && purchase_day == current_day
}

fn weather_roll_day_forecasts(
    mut day_events: EventReader<WeatherDayStartedEvent>,
    mut forecast_events: EventWriter<WeatherForecastResolvedEvent>,
    seed: Res<GameSeed>,
    tick: Res<SimTick>,
    mut state: ResMut<WeatherState>,
) {
    for event in day_events.read() {
        state.current_day = event.day;
        state.employee_count = event.employee_count;
        state.no_death_streak_days = event.no_death_streak_days;
        state.total_moon_count = event.total_moon_count;

        let mut count_rng = tick_rng(seed.0, tick.0, 0x7765_6174_6865_7201);
        let random_unit = fixed_unit_from_u32(count_rng.next_u32());
        let mut selected_count = random_unit * I16F16::from_num(event.total_moon_count);

        if event.employee_count >= 2
            && event.no_death_streak_days != 0
            && event.no_death_streak_days % 3 == 0
        {
            let multiplier_roll = fixed_unit_from_u32(count_rng.next_u32());
            let multiplier = I16F16::from_num(3) / I16F16::from_num(2) + multiplier_roll;
            selected_count *= multiplier;
        }

        let clamped_count = clamp_fixed_to_moon_count(selected_count, event.total_moon_count);
        state.affected_moon_count = clamped_count;

        let titan_forecast = if clamped_count > 0 {
            let mut titan_rng = tick_rng(seed.0, tick.0, 0x7765_6174_6865_7202 ^ stable_id(TITAN_ID));
            let index = titan_rng.next_u32() as usize % TITAN_VALID_WEATHER.len();
            TITAN_VALID_WEATHER[index]
        } else {
            WeatherKind::Clear
        };

        state.titan_forecast = titan_forecast;

        forecast_events.send(WeatherForecastResolvedEvent {
            day: event.day,
            affected_moon_count: state.affected_moon_count,
            titan_forecast,
        });
    }
}

fn weather_resolve_meteor_impacts(
    mut impact_events: EventReader<MeteorImpactEvent>,
    mut state: ResMut<WeatherState>,
) {
    for event in impact_events.read() {
        state.meteor_impacts = state.meteor_impacts.wrapping_add(1);

        if event.occupied_target_id.is_some() {
            state.meteor_kills = state.meteor_kills.wrapping_add(1);
        }

        state.meteor_knockbacks = state
            .meteor_knockbacks
            .wrapping_add(event.nearby_crew_count as u64);
    }
}

fn weather_checksum(
    mut checksum: ResMut<SimChecksumState>,
    tick: Res<SimTick>,
    state: Res<WeatherState>,
) {
    checksum.accumulate(tick.0);
    checksum.accumulate(WEATHER_SOURCE_REVISION as u64);
    checksum.accumulate(WEATHER_CONFIDENCE_BASIS_POINTS as u64);
    checksum.accumulate(TITAN_SOURCE_REVISION as u64);
    checksum.accumulate(FOGGY_OUTDOOR_VISIBILITY_UNITS as u64);
    checksum.accumulate(METEOR_SHOWER_DURATION_INGAME_HOURS as u64);
    checksum.accumulate(METEOR_SHOWER_DURATION_REALTIME_SECONDS_TENTHS as u64);
    checksum.accumulate(TITAN_ECLIPSED_START_INDOOR_ENTITIES as u64);
    checksum.accumulate(TITAN_ECLIPSED_START_OUTDOOR_ENTITIES as u64);

    for dependency in WEATHER_DEPENDS_ON {
        accumulate_str(&mut checksum, 0x1000, dependency);
    }

    for rule in WEATHER_RULES {
        accumulate_str(&mut checksum, 0x1100, rule);
    }

    for modifier in WEATHER_MODIFIERS {
        accumulate_str(&mut checksum, 0x1200, modifier);
    }

    for strategy in WEATHER_STRATEGY {
        accumulate_str(&mut checksum, 0x1300, strategy);
    }

    for note in WEATHER_NOTES {
        accumulate_str(&mut checksum, 0x1400, note);
    }

    for rule in WEATHER_BEHAVIORAL_MECHANICS {
        accumulate_str(&mut checksum, 0x2000, rule.condition);
        accumulate_str(&mut checksum, 0x2001, rule.outcome);
    }

    for weather in TITAN_VALID_WEATHER {
        checksum.accumulate(0x3000 ^ weather.checksum_value());
        accumulate_str(&mut checksum, 0x3001, weather.id());
    }

    accumulate_str(&mut checksum, 0x4000, WEATHER_OVERVIEW);
    accumulate_str(&mut checksum, 0x4001, TITAN_ID);
    accumulate_str(&mut checksum, 0x4002, TITAN_NAME);

    checksum.accumulate(state.current_day as u64);
    checksum.accumulate(state.employee_count as u64);
    checksum.accumulate(state.no_death_streak_days as u64);
    checksum.accumulate(state.total_moon_count as u64);
    checksum.accumulate(state.affected_moon_count as u64);
    checksum.accumulate(state.titan_forecast.checksum_value());
    checksum.accumulate(state.meteor_shower_active as u64);
    checksum.accumulate(state.meteor_shower_started_tick);
    checksum.accumulate(state.meteor_shower_imminent_alerts);
    checksum.accumulate(state.meteor_impacts);
    checksum.accumulate(state.meteor_kills);
    checksum.accumulate(state.meteor_knockbacks);
    checksum.accumulate(state.conductive_purchase_protection_day as u64);
}

fn fixed_unit_from_u32(value: u32) -> I16F16 {
    I16F16::from_num(value) / I16F16::from_num(u32::MAX)
}

fn clamp_fixed_to_moon_count(value: I16F16, total_moon_count: u32) -> u32 {
    if value <= I16F16::from_num(0) {
        return 0;
    }

    let max = I16F16::from_num(total_moon_count);
    if value >= max {
        return total_moon_count;
    }

    value.to_num::<u32>()
}

fn stable_id(value: &str) -> u64 {
    let mut id = value.len() as u64;

    for (index, byte) in value.bytes().enumerate() {
        id = id.wrapping_add(((index as u64) + 1) * byte as u64);
    }

    id
}

fn accumulate_str(checksum: &mut SimChecksumState, salt: u64, value: &str) {
    checksum.accumulate(salt ^ value.len() as u64);

    for (index, byte) in value.bytes().enumerate() {
        checksum.accumulate(salt ^ ((index as u64) << 8) ^ byte as u64);
    }
}