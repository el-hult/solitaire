use crate::ai::SimpleAi;

mod ai;
mod ai_greedy;
mod game;
mod view;

enum AiType {
    Simple,
    Greedy,
}

trait Ai {
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

impl Ai for ai_greedy::GreedyAi {
    fn make_move(&mut self, view: &view::SolitaireView) -> game::Action {
        self.calc_action(view)
    }
    fn name(&self) -> &'static str {
        "GreedyAi"
    }
}

/// The main function.
fn main() -> Result<(), std::io::Error> {
    let n_games_to_play = 10;
    let mut game_statistics = Vec::new();

    for k in 0..n_games_to_play {
        for ai_type in [AiType::Simple, AiType::Greedy] {
        let mut gs = game::GameEngine::deal(k);
        let t_begin = std::time::Instant::now();
        let mut ai: Box<dyn Ai> = match ai_type {
            AiType::Simple => Box::from(ai::SimpleAi::new()),
            AiType::Greedy => Box::from(ai_greedy::GreedyAi::new())
        };
        let mut n_actions_taken = 0;
        while gs.is_running() {
            let action = ai.make_move(&gs.observe());
            gs.act(&action)
                .unwrap_or_else(|_| panic!("The AI suggested {:?} an illegal move!", action));
            n_actions_taken += 1;
        }
        let t_end = std::time::Instant::now();
        let stats = (ai.name(), k, gs.score(), gs.is_won(),n_actions_taken, t_end - t_begin);
        game_statistics.push(stats);
        println!("{:?}", stats);
    }
    }
    println!("{:#?}", game_statistics);
    Ok(())
}
