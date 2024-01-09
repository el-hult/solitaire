//! A simple AI player that can play solitaire
//!
use super::{Action, CardView, SolitaireObserver};
use crate::core::{Addr, Value};

/// A simple AI player that can play solitaire
///
pub struct SimpleAi {
    seen_state_action_combos: std::collections::HashSet<(SolitaireObserver, Action)>,
    // have we made passes through the deck?
    number_of_passes: u64,
    view: SolitaireObserver,
}

impl SimpleAi {
    pub fn new(view: SolitaireObserver) -> Self {
        SimpleAi {
            seen_state_action_combos: std::collections::HashSet::new(),
            number_of_passes: 0,
            view,
        }
    }

    /// Produce all valid moves that we potentially would like to make in a prioritized order
    ///
    /// Some of the simplest advice from <https://solitaired.com/ultimate-solitaire-strategy-guide> are implemented
    fn suggest_actions(&mut self) -> Vec<Action> {
        let mut actions = vec![];
        if self.view.is_won() {
            actions.push(Action::Quit);
            return actions;
        }

        // Build on foundations
        for from_addr in Addr::DEPOTS_AND_WASTE.iter() {
            if let Some(CardView::FaceUp(suit, value)) = self.view.card_at(from_addr, 1) {
                for to_addr in Addr::FOUNDATIONS {
                    match self.view.card_at(&to_addr, 1) {
                        None => {
                            if value.is_ace() {
                                actions.push(Action::Move(*from_addr, to_addr, 1));
                            }
                        }
                        // increase by one
                        Some(CardView::FaceUp(to_suit, to_value)) => {
                            if suit == to_suit
                                && value.numeric_value() == to_value.numeric_value() + 1
                            {
                                actions.push(Action::Move(*from_addr, to_addr, 1));
                            }
                        }
                        Some(CardView::FaceDown) => {
                            continue;
                        }
                    }
                }
            }
        }

        // Try to reveal a card
        for (idx, a) in self.view.depots.iter().enumerate() {
            if let Some(CardView::FaceDown) = a.last() {
                actions.push(Action::Reveal(Addr::DEPOTS[idx]));
            }
        }

        // Try to increase the sequences in the tableaux
        for from in Addr::DEPOTS_AND_WASTE {
            let max_cards_to_move = self.view.n_takeable_cards(&from);
            if max_cards_to_move == 0 {
                continue;
            }
            for to in Addr::DEPOTS.into_iter().filter(|to| to != &from) {
                if from.is_waste() && matches!(self.view.waste.last(), Some((_, Value::TWO))) {
                    // Don't move 2's from the hand to the tableaux - they can only ever block other cards
                    continue;
                }
                if from.is_waste() && to.is_depot() {
                    // Dont move low values from the hand to the tableaux too early
                    if let Some((_, value)) = self.view.waste.last() {
                        if value.numeric_value() < 5 && self.number_of_passes == 0 {
                            continue;
                        }
                    }
                }
                for n_moves in 1..=max_cards_to_move {
                    if let Some(CardView::FaceUp(suit, value)) = self.view.card_at(&from, n_moves) {
                        match self.view.card_at(&to, 1) {
                            None => {
                                if value == Value::KING {
                                    actions.push(Action::Move(from, to, n_moves));
                                }
                            }
                            Some(CardView::FaceUp(suit2, value2)) => {
                                let is_valid_move = suit.color() != suit2.color()
                                    && value.numeric_value() == value2.numeric_value() - 1;
                                if is_valid_move {
                                    actions.push(Action::Move(from, to, n_moves));
                                }
                            }
                            Some(CardView::FaceDown) => { /* do nothing */ }
                        }
                    }
                }
            }
        }

        // Take from the talon
        if self.view.talon_size != 0 {
            actions.push(Action::Take);
        }

        // Turn over the talon
        if self.view.waste.last().is_some() && self.view.talon_size == 0 {
            actions.push(Action::Turnover);
        }

        // Give up
        actions.push(Action::Quit);
        actions
    }
}

impl super::Ai for SimpleAi {
    fn make_move(&mut self) -> Action {
        let actions = self.suggest_actions();
        // dbg!(&actions);
        for action in actions {
            if self
                .seen_state_action_combos
                .contains(&(self.view.clone(), action.clone()))
            {
                continue;
            }
            self.seen_state_action_combos
                .insert((self.view.clone(), action.clone()));
            if action == Action::Turnover {
                self.number_of_passes += 1;
            }
            return action;
        }
        panic!("No action found");
    }
    fn name(&self) -> &'static str {
        "SimpleAi"
    }
    fn update(&mut self, action: Action, res: Option<(crate::core::Suit, Value)>) {
        self.view.update(action, res)
    }
}

#[cfg(test)]
mod tests {
    use crate::core::Suit;

    use super::*;

    #[test]
    fn test_ai_can_win() {
        let view = SolitaireObserver {
            talon_size: 0,
            waste: vec![],
            foundation_tops: [None; 4],
            depots: [
                vec![CardView::FaceUp(Suit::Hearts, Value::KING)],
                vec![CardView::FaceUp(Suit::Clubs, Value::QUEEN)],
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
            ],
        };
        let mut ai = SimpleAi::new(view);
        let actions = ai.suggest_actions();
        assert!(
            actions.contains(&Action::Move(Addr::Depot2, Addr::Depot1, 1)),
            "Should be able to move queen of clubs to king of hearts"
        );
    }
}
