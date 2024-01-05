//! An AI player that plays greedy
//! 
use crate::view::{SolitaireView, CardView, DEPOTS_AND_WASTE, Addr, Value};
use super::game::Action;

/// An AI player that plays greedy
/// 
pub struct GreedyAi {
    seen_state_action_combos: std::collections::HashSet<(SolitaireView, Action)>,
    // have we made passes through the deck?
    number_of_passes: u64,
}

struct PrioritizedAction {
    priority: i64,
    action: Action,
}

impl From<(i64, Action)> for PrioritizedAction {
    fn from((priority, action): (i64, Action)) -> Self {
        PrioritizedAction { priority, action }
    }
}

impl PartialOrd for PrioritizedAction {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.priority.partial_cmp(&other.priority)
    }
}

impl Ord for PrioritizedAction {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.priority.cmp(&other.priority)
    }
}

impl PartialEq for PrioritizedAction {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

/// Actually, the elements are not equal, but they are equally prioritized
impl Eq for PrioritizedAction {}

impl GreedyAi {
    pub fn new() -> Self {
        GreedyAi {
            seen_state_action_combos: std::collections::HashSet::new(),
            number_of_passes: 0,
        }
    }

    /// Provide a single move,
    pub fn calc_action(&mut self, view: &SolitaireView) -> Action {
        let actions = self.suggest_actions(view);
        // dbg!(&actions);
        for action in actions {
            if self
                .seen_state_action_combos
                .contains(&(view.clone(), action.clone()))
            {
                continue;
            }
            self.seen_state_action_combos
                .insert((view.clone(), action.clone()));
            if action == Action::Turnover {
                self.number_of_passes += 1;
            } 
            return action;
        }
        panic!("No action found");
    }



    /// Produce all valid moves that we potentially would like to make in a prioritized order
    /// 
    /// Prioritizes moves that give more score
    fn suggest_actions(&mut self, view: &SolitaireView) -> Vec<Action> {
        if view.is_won() {
            return vec![Action::Quit];
        }
        let mut actions: std::collections::BinaryHeap<PrioritizedAction>  = std::collections::BinaryHeap::new();
        
        // Build on foundations
        for from_addr in DEPOTS_AND_WASTE.iter() {
            if let Some(CardView::FaceUp(suit,value)) = view.card_at(from_addr, 1) {
                for to_addr in Addr::FOUNDATIONS {
                    match view.card_at(&to_addr, 1) {
                        None => {
                            if value.is_ace() {
                                actions.push(
                                    (10,Action::Move(*from_addr, to_addr, 1)).into()
                                );
                            }
                        }
                        // increase by one
                        Some(CardView::FaceUp(to_suit,to_value)) => {
                            if suit == to_suit
                                && value.numeric_value()
                                    == to_value.numeric_value() + 1
                            {
                                actions.push((10,Action::Move(*from_addr, to_addr, 1)).into());
                            }
                        }
                        Some(CardView::FaceDown) => {continue;}
                    }
                }
            }
        }

        // Try to reveal a card
        for (idx,a) in view.depots.iter().enumerate() {
            if let Some(CardView::FaceDown) = a.last() {
                actions.push((5,Action::Reveal(Addr::DEPOTS[idx])).into());
            }
        }

        // Try to increase the sequences in the tableaux
        for from in DEPOTS_AND_WASTE {
            let max_cards_to_move = view.n_takeable_cards(&from);
            if max_cards_to_move == 0 {
                continue;
            }
            for to in Addr::DEPOTS.into_iter().filter(|to| to != &from) {

                let score = if from.is_foundation() && to.is_depot() { -15} 
                else if from.is_waste() && to.is_foundation() {10}
                else if from.is_waste() && to.is_depot() {5}
                else {0};

                for n_moves in 1..=max_cards_to_move {
                    if let Some(CardView::FaceUp(suit, value)) =
                        view.card_at(&from, n_moves)
                    {
                        match view.card_at(&to, 1) {
                            None => {
                                if value == Value::KING {
                                    actions.push((score,Action::Move(from, to, n_moves)).into());
                                }
                            }
                            Some(CardView::FaceUp(suit2,value2)) => {
                                let is_valid_move = suit.color() != suit2.color()
                                    && value.numeric_value() == value2.numeric_value() - 1;
                                if is_valid_move {
                                    actions.push((score,Action::Move(from, to, n_moves)).into());
                                }
                            }
                            Some(CardView::FaceDown) => { /* do nothing */ }
                        }
                    }
                }
            }
        }

        // Take from the talon
        if view.talon_size != 0 {
            actions.push((0,Action::Take).into());
        }

        // Turn over the talon
        if view.waste_top.is_some() && view.talon_size == 0 {
            actions.push((-100,Action::Turnover).into());
        }

        // Give up
        actions.push((0,Action::Quit).into());
        actions.into_sorted_vec().into_iter().map(|a| a.action).collect()
    }
}


#[cfg(test)]
mod tests {
    use crate::view::Suit;

    use super::*;

    #[test]
    fn test_ai_can_win() {
        let mut ai = GreedyAi::new();
        let view = SolitaireView{
            talon_size: 0,
            waste_top: None,
            foundation_tops: [None; 4],
            depots: [
                vec![CardView::FaceUp(Suit::Hearts, Value::KING)],
                vec![CardView::FaceUp(Suit::Clubs, Value::QUEEN)],
                vec![], vec![], vec![], vec![], vec![]
            ],
        };
        let actions = ai.suggest_actions(&view);
        assert!(actions.contains(&Action::Move(Addr::Depot2, Addr::Depot1, 1)), "Should be able to move queen of clubs to king of hearts");
    }
}