// Sources: vault/item_index_pages/items.md, vault/version_pages/version_4.md

use bevy::prelude::*;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Menu,
    Playing,
    Paused,
    Win,
    Lose,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct RequestStateChangeEvent(pub GameState);

pub struct GameStateMachinePlugin;

impl Plugin for GameStateMachinePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy::state::app::StatesPlugin)
            .init_state::<GameState>()
            .add_event::<RequestStateChangeEvent>()
            .add_systems(Update, apply_state_transitions);
    }
}

fn is_transition_allowed(from: GameState, to: GameState) -> bool {
    matches!(
        (from, to),
        (GameState::Menu, GameState::Playing)
            | (GameState::Playing, GameState::Paused)
            | (GameState::Playing, GameState::Win)
            | (GameState::Playing, GameState::Lose)
            | (GameState::Paused, GameState::Playing)
            | (GameState::Paused, GameState::Menu)
            | (GameState::Win, GameState::Menu)
            | (GameState::Lose, GameState::Menu)
    )
}

fn apply_state_transitions(
    mut events: EventReader<RequestStateChangeEvent>,
    current: Res<State<GameState>>,
    mut next: ResMut<NextState<GameState>>,
) {
    for RequestStateChangeEvent(target) in events.read() {
        if is_transition_allowed(*current.get(), *target) {
            next.set(*target);
        }
    }
}