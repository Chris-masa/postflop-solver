pub use crate::bindings::exports::holdem_solver::host::game_manager::{
    WitAction, WitActionHistoryDetail, WitActionRatio, WitGameStatus, WitStreet,
};
use std::cmp::Ordering;

use solver_core::{Action, ActionHistoryDetail, BoardState, GameStatus};

impl From<Action> for WitAction {
    fn from(action: Action) -> Self {
        match action {
            Action::None => WitAction::None,
            Action::Fold => WitAction::Fold,
            Action::Check => WitAction::Check,
            Action::Call => WitAction::Call,
            Action::Bet(x) => WitAction::Bet(x as u32),
            Action::Raise(x) => WitAction::Raise(x as u32),
            Action::AllIn(x) => WitAction::AllIn(x as u32),
            Action::Chance(card) => WitAction::Chance(card as u32),
        }
    }
}

impl WitAction {
    pub fn sort_order(&self) -> u8 {
        match self {
            WitAction::None => 0,
            WitAction::Fold => 1,
            WitAction::Check => 2,
            WitAction::Call => 3,
            WitAction::Bet(_) => 4,
            WitAction::Raise(_) => 5,
            WitAction::AllIn(_) => 6,
            WitAction::Chance(_) => 7,
        }
    }

    pub fn inner_value(&self) -> u32 {
        match self {
            WitAction::Bet(x)
            | WitAction::Raise(x)
            | WitAction::AllIn(x)
            | WitAction::Chance(x) => *x,
            _ => 0,
        }
    }
}

impl Ord for WitAction {
    fn cmp(&self, other: &Self) -> Ordering {
        self.sort_order()
            .cmp(&other.sort_order())
            .then_with(|| self.inner_value().cmp(&other.inner_value()))
    }
}

impl PartialOrd for WitAction {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for WitAction {
    fn eq(&self, other: &Self) -> bool {
        self.sort_order() == other.sort_order() && self.inner_value() == other.inner_value()
    }
}

impl Eq for WitAction {}

impl From<(Action, f32)> for WitActionRatio {
    fn from(value: (Action, f32)) -> Self {
        let (action, ratio) = value;
        let action_converted = WitAction::from(action);
        WitActionRatio {
            action: action_converted,
            ratio: ratio,
        }
    }
}

impl From<GameStatus> for WitGameStatus {
    fn from(status: GameStatus) -> Self {
        match status {
            GameStatus::Chance => WitGameStatus::Chance,
            GameStatus::Ip => WitGameStatus::IpAction,
            GameStatus::Oop => WitGameStatus::OopAction,
            GameStatus::Terminal => WitGameStatus::Terminal,
            GameStatus::Unknown => panic!("Invalid game status"),
        }
    }
}

impl From<ActionHistoryDetail> for WitActionHistoryDetail {
    fn from(detail: ActionHistoryDetail) -> Self {
        let mut tmp_list: Vec<WitActionRatio> = detail
            .actions
            .into_iter()
            .map(|(action, ratio)| WitActionRatio::from((action, ratio)))
            .collect();

        // sort_byを実行（WitActionにOrd実装があるので、sortでもOK）
        tmp_list.sort_by(|a, b| a.action.cmp(&b.action));
        WitActionHistoryDetail {
            action_ratio_list: tmp_list,
            game_status: WitGameStatus::from(detail.game_status),
            pot: detail.pot_without_current_bet as u32,
            opponent_bet_size: detail.opponent_bet_size.and_then(|v| u32::try_from(v).ok()),
            street: match detail.street {
                BoardState::Flop => WitStreet::Flop,
                BoardState::Turn => WitStreet::Turn,
                BoardState::River => WitStreet::River,
            },
        }
    }
}
