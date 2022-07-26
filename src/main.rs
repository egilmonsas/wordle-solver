const GAMES: &str = include_str!("../data/answers.txt");

fn main() {
    let w = wordle_rs::Wordle::new();
    for game in GAMES.split_whitespace() {
        let guesser = wordle_rs::algorithms::naive::Naive::new();
        w.play(game, guesser);
    }
}
