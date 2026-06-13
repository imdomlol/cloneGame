// Sources: vault/item_index_pages/items.md, vault/version_pages/version_4.md

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy::state::state::StateTransition;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Menu,
    Playing,
    Paused,
    Win,
    Lose,
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameStateMachineState {
    pub current: GameState,
    pub transition_count: u64,
}

impl Default for GameStateMachineState {
    fn default() -> Self {
        Self {
            current: GameState::Menu,
            transition_count: 0,
        }
    }
}

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct RequestStateChangeEvent(pub GameState);

#[derive(Event, Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameStateChangedEvent {
    pub from: GameState,
    pub to: GameState,
    pub transition_count: u64,
}

pub struct GameStateMachinePlugin;

impl Plugin for GameStateMachinePlugin {
    fn build(&self, app: &mut App) {
        let has_state_transition_schedule = app
            .world()
            .resource::<Schedules>()
            .contains(StateTransition);

        if !has_state_transition_schedule {
            app.add_plugins(StatesPlugin);
        }

        app.init_state::<GameState>()
            .init_resource::<GameStateMachineState>()
            .add_event::<RequestStateChangeEvent>()
            .add_event::<GameStateChangedEvent>()
            .add_systems(
                Update,
                (
                    apply_state_transition_requests,
                    observe_menu_state.run_if(in_state(GameState::Menu)),
                    observe_playing_state.run_if(in_state(GameState::Playing)),
                    observe_paused_state.run_if(in_state(GameState::Paused)),
                    observe_win_state.run_if(in_state(GameState::Win)),
                    observe_lose_state.run_if(in_state(GameState::Lose)),
                )
                    .chain(),
            );
    }
}

pub const fn is_transition_allowed(from: GameState, to: GameState) -> bool {
    matches!(
        (from, to),
        (GameState::Menu, GameState::Playing)
            | (GameState::Playing, GameState::Paused)
            | (GameState::Playing, GameState::Win)
            | (GameState::Playing, GameState::Lose)
            | (GameState::Playing, GameState::Menu)
            | (GameState::Paused, GameState::Playing)
            | (GameState::Paused, GameState::Menu)
            | (GameState::Win, GameState::Menu)
            | (GameState::Lose, GameState::Menu)
    )
}

fn apply_state_transition_requests(
    mut requests: EventReader<RequestStateChangeEvent>,
    current: Res<State<GameState>>,
    mut next: ResMut<NextState<GameState>>,
) {
    let from = *current.get();

    for RequestStateChangeEvent(target) in requests.read() {
        if *target == from {
            continue;
        }

        if is_transition_allowed(from, *target) {
            next.set(*target);
            break;
        }
    }
}

fn observe_menu_state(
    machine: ResMut<GameStateMachineState>,
    changed: EventWriter<GameStateChangedEvent>,
) {
    observe_state(GameState::Menu, machine, changed);
}

fn observe_playing_state(
    machine: ResMut<GameStateMachineState>,
    changed: EventWriter<GameStateChangedEvent>,
) {
    observe_state(GameState::Playing, machine, changed);
}

fn observe_paused_state(
    machine: ResMut<GameStateMachineState>,
    changed: EventWriter<GameStateChangedEvent>,
) {
    observe_state(GameState::Paused, machine, changed);
}

fn observe_win_state(
    machine: ResMut<GameStateMachineState>,
    changed: EventWriter<GameStateChangedEvent>,
) {
    observe_state(GameState::Win, machine, changed);
}

fn observe_lose_state(
    machine: ResMut<GameStateMachineState>,
    changed: EventWriter<GameStateChangedEvent>,
) {
    observe_state(GameState::Lose, machine, changed);
}

fn observe_state(
    state: GameState,
    mut machine: ResMut<GameStateMachineState>,
    mut changed: EventWriter<GameStateChangedEvent>,
) {
    if machine.current == state {
        return;
    }

    let previous = machine.current;
    machine.current = state;
    machine.transition_count = machine.transition_count.wrapping_add(1);

    changed.send(GameStateChangedEvent {
        from: previous,
        to: state,
        transition_count: machine.transition_count,
    });
}