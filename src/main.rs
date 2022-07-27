use std::time::Instant;
use wordle_rs::GAMES;

fn main() {
    let mut hits = 0.0;
    let mut trials = 0.0;
    println!("prog\ttrials\ttime");

    let game_count = GAMES.split_whitespace().count() as f64;
    let mut progress_ticker = 0.0;
    let start_time = Instant::now();
    let w = wordle_rs::Wordle::new();
    for game in GAMES.split_whitespace() {
        let game = game
            .as_bytes()
            .try_into()
            .expect("Guess game must be a 5 character word");

        let guesser = wordle_rs::solver::Solver::new();
        if let Some(score) = w.play(game, guesser) {
            hits += 1.0;
            trials += score as f64;
            
            progress_ticker+=1.0 / game_count;
        }
        let avg_trials = trials / hits;
        let avg_time = start_time.elapsed().as_secs() as f64 / hits;

        if progress_ticker>=0.1{
            progress_ticker=0.0;
            println!(
                "{:.1}% \t{:.2}\t{:.2}",
                hits / game_count * 100.0,
                avg_trials,
                avg_time
            );
        }
    }
    
    println!(
        "\n\n{:.1}% \t{:.2}\t{:.2}",
        hits / game_count * 100.0,
        trials / hits,
        start_time.elapsed().as_secs() as f64
    );
}
