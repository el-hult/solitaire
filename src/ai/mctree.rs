///! AI that implements a Monte Carlo Tree Search.
/// 

use super::SolitaireObserver;

/// The AI class that implements my Monte Carlo Tree Search
/// 
/// Honestly, I think this is a misnomer. I don't really do <https://en.wikipedia.org/wiki/Monte_Carlo_tree_search>
/// but I do make a tree search, and I use random sampling to evaluate the score on moves whose score I don't know.
/// So I will call it MCTS for the time being.
pub struct MonteCarloTreeSearchAI {
    current_game_state: (),
}

impl MonteCarloTreeSearchAI {
    pub fn new(_obs:SolitaireObserver) -> Self {
        Self {
            current_game_state: (),
        }
    }
}

impl super::Ai for MonteCarloTreeSearchAI {
    fn make_move(&mut self) -> super::Action {
        todo!()
    }

    fn name(&self) -> &'static str {
        "Monte Carlo Tree Search"
    }

    fn update(&mut self, _action: super::Action, _res: Option<(super::Suit, super::Value)>) {
        todo!()
    }
}

