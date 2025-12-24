use crate::model::{Action, Board};
use dioxus::prelude::*;

#[derive(Clone, Copy)]
pub struct BoardState {
    pub board: Signal<Board>,
    pub history: Signal<Vec<Action>>,
}

impl BoardState {
    pub fn new() -> Self {
        Self {
            board: Signal::new(Board::default()),
            history: Signal::new(Vec::new()),
        }
    }

    pub fn apply_action(&mut self, action: Action) {
        self.history.write().push(action.clone());
        match action {
            Action::Draw(shape) => {
                self.board.write().shapes.push(shape);
            }
            Action::Wipe => {
                self.board.write().shapes.clear();
            }
            Action::NewBoard => {
                self.board.write().shapes.clear();
                // Logic for new board ID could go here if we tracked multiple boards
            }
        }
    }
}
