use itertools::Itertools;

mod ai;
mod engine;
mod core;

/// The main function.
fn main() -> Result<(), std::io::Error> {
    let n_games_to_play = 10;
    let mut game_statistics = Vec::new();

    for k in 0..n_games_to_play {
        let make_greedy: fn(ai::SolitaireObserver) -> Box<dyn ai::Ai> = |obs| Box::from(ai::GreedyAi::new(obs)); 
        let make_simple: fn(ai::SolitaireObserver) -> Box<dyn ai::Ai> = |obs| Box::from(ai::SimpleAi::new(obs));
        let ai_makers  = [make_simple, make_greedy];
        for make_ai in ai_makers {
            let mut gs = engine::GameEngine::deal(k);
            let t_begin = std::time::Instant::now();
            let mut ai: Box<dyn ai::Ai> = make_ai(gs.observe());
            let mut n_actions_taken = 0;
            while gs.is_running() {
                let action = ai.make_move();
                let res = gs.act(&action)
                    .unwrap_or_else(|_| panic!("The AI suggested {:?} an illegal move!", action));
                ai.update(action, res);
                n_actions_taken += 1;
            }
            let t_end = std::time::Instant::now();
            let stats = (
                ai.name(),
                k,
                gs.score(),
                gs.is_won(),
                n_actions_taken,
                t_end - t_begin,
            );
            game_statistics.push(stats);
            println!("{:?}", stats);
        }
    }
    game_statistics
        .iter()
        .sorted()
        .group_by(|x| x.0)
        .into_iter()
        .for_each(|(key, group)| {
            let group = group.collect::<Vec<_>>();
            let wins = group.iter().fold(0u8, |acc, tup| acc + tup.3 as u8);
            let score = group.iter().fold(0, |acc, tup| acc + tup.2);
            println!("{key}: {wins} wins. Total score {score}");
        });
    Ok(())
}
