mod ai;
mod game;
mod view;

/// The main function.
fn main() -> Result<(), std::io::Error> {
    let n_games_to_play = 10;
    let mut game_statistics = Vec::new();

    for k in 0..n_games_to_play {
        let mut gs = game::GameEngine::deal(k);
        let t_begin = std::time::Instant::now();
        let mut ai = ai::SimpleAi::new();
        let mut n_actions_taken = 0;
        while gs.is_running() {
            let action = ai.calc_action(&gs.observe());
            gs.act(&action)
                .unwrap_or_else(|_| panic!("The AI suggested {:?} an illegal move!", action));
            n_actions_taken += 1;
        }
        let t_end = std::time::Instant::now();
        let stats = ("SimpleAi", k, gs.score(), gs.is_won(),n_actions_taken, t_end - t_begin);
        game_statistics.push(stats);
        println!("{:?}", stats);
    }
    println!("{:#?}", game_statistics);
    Ok(())
}
