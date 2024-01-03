mod ai;
mod game;
mod view;

/// The main function.
fn main() -> Result<(), std::io::Error> {
    let mut n_runs = 0;
    let mut n_wins = 0;
    let n_games_to_play = 10;

    for k in 0..n_games_to_play {
        let mut gs = game::GameEngine::deal(k);
        let mut ai = ai::AiPlayer::new();
        let mut n_actions_taken = 0;
        while gs.is_running() {
            let action = ai.calc_action(&gs.observe());
            gs.act(&action)
                .unwrap_or_else(|_| panic!("The AI suggested {:?} an illegal move!", action));
            n_actions_taken += 1;
        }
        if gs.is_won() {
            println!("Game {} won in {} moves", k, n_actions_taken);
        } else {
            println!("Game {} lost in {} moves", k, n_actions_taken);
        }
        n_runs += 1;
        if gs.is_won() {
            n_wins += 1;
        }
    }
    println!("Won {} out of {} games", n_wins, n_runs);
    Ok(())
}
