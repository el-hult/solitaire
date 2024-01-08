//! AI module
//! 
//! Defines the interface for the AI players and reexports them from their respective submodules.
//! 
pub mod simple;
pub mod greedy;

use crate::{view, game};
pub use simple::SimpleAi;
pub use greedy::GreedyAi;


pub enum AiType {
    Simple,
    Greedy,
}

pub trait Ai {
    fn make_move(&mut self, view: &view::SolitaireView) -> game::Action;
    fn name(&self) -> &'static str;
}

impl Ai for SimpleAi {
    fn make_move(&mut self, view: &view::SolitaireView) -> game::Action {
        self.calc_action(view)
    }
    fn name(&self) -> &'static str {
        "SimpleAi"
    }
}

impl Ai for GreedyAi {
    fn make_move(&mut self, view: &view::SolitaireView) -> game::Action {
        self.calc_action(view)
    }
    fn name(&self) -> &'static str {
        "GreedyAi"
    }
}
