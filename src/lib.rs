use std::collections::HashSet;

pub mod algorithms;

const DICTIONARY: &str = include_str!("../data/dictionary.txt");

pub struct Wordle {
    dictionary: HashSet<&'static str>,
}
impl Wordle {
    pub fn new() -> Self {
        Self {
            dictionary: HashSet::from_iter(DICTIONARY.lines().map(|line| {
                line.split_once(' ')
                    .expect("Every line should be [word] [freq]")
                    .0
            })),
        }
    }

    pub fn play<G: Guesser>(&self, answer: &'static str, mut guesser: G) -> Option<usize> {
        let mut history = Vec::new();
        for i in 1..=32 {
            let guess = guesser.guess(&history[..]);

            if guess == answer {
                return Some(i);
            }
            assert!(self.dictionary.contains(&*guess));
            let correctness = Correctness::compute(answer, &guess);

            history.push(Guess {
                word: guess,
                mask: correctness,
            });
        }
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Correctness {
    Correct,
    Misplaced,
    Wrong,
}
pub struct Guess {
    word: String,
    mask: [Correctness; 5],
}

pub trait Guesser {
    fn guess(&mut self, history: &[Guess]) -> String;
}

impl Correctness {
    fn compute(answer: &str, guess: &str) -> [Correctness; 5] {
        assert_eq!(answer.len(), 5);
        assert_eq!(guess.len(), 5);
        let mut c = [Correctness::Wrong; 5];

        //Mark fields green
        for (i, (a, g)) in answer.chars().zip(guess.chars()).enumerate() {
            if a == g {
                c[i] = Correctness::Correct;
            }
        }
        //Mark fields yellow
        let mut used = [false; 5];
        for (i, &c) in c.iter().enumerate() {
            if c == Correctness::Correct {
                used[i] = true;
            }
        }
        for (i, g) in guess.chars().enumerate() {
            if c[i] == Correctness::Correct {
                continue;
            }
            if answer.chars().enumerate().any(|(i, a)| {
                if a == g && !used[i] {
                    used[i] = true;
                    return true;
                }
                false
            }) {
                c[i] = Correctness::Misplaced;
            };
        }
        c
    }
}

impl Guesser for fn(history: &[Guess]) -> String {
    fn guess(&mut self, history: &[Guess]) -> String {
        (*self)(history)
    }
}

#[cfg(test)]
macro_rules! guesser {
    (|$history:ident| $impl:block) => {{
        struct G;
        impl Guesser for G {
            fn guess(&mut self, $history: &[Guess]) -> String {
                $impl
            }
        }
        G
    }};
}

#[cfg(test)]
mod tests {
    mod game {
        use crate::{Guess, Guesser, Wordle};

        #[test]
        fn genius() {
            let w = Wordle::new();
            let guesser = guesser!(|_history| { "right".to_string() });
            assert_eq!(w.play("right", guesser), Some(1));
        }
        #[test]
        fn magnificent() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 1 {
                    return "right".to_string();
                }
                return "wrong".to_string();
            });
            assert_eq!(w.play("right", guesser), Some(2));
        }
        #[test]
        fn impressive() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 2 {
                    return "right".to_string();
                }
                return "wrong".to_string();
            });
            assert_eq!(w.play("right", guesser), Some(3));
        }
        #[test]
        fn splendid() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 3 {
                    return "right".to_string();
                }
                return "wrong".to_string();
            });
            assert_eq!(w.play("right", guesser), Some(4));
        }
        #[test]
        fn great() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 4 {
                    return "right".to_string();
                }
                return "wrong".to_string();
            });
            assert_eq!(w.play("right", guesser), Some(5));
        }
        #[test]
        fn phew() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 5 {
                    return "right".to_string();
                }
                return "wrong".to_string();
            });
            assert_eq!(w.play("right", guesser), Some(6));
        }
        #[test]
        fn retarded() {
            let w = Wordle::new();
            let guesser = guesser!(|_history| {
                return "wrong".to_string();
            });
            assert_eq!(w.play("right", guesser), None);
        }
    }
    mod compute {
        use crate::Correctness;

        macro_rules!  mask {
            (c) => {Correctness::Correct};
            (m) => {Correctness::Misplaced};
            (w) => {Correctness::Wrong};
            ($($c:tt)+)=>{[
                $(mask!($c)),+
            ]}
        }

        #[test]
        fn all_correct_chars() {
            assert_eq!(Correctness::compute("abcde", "abcde"), mask![c c c c c])
        }
        #[test]
        fn all_misplaced() {
            assert_eq!(Correctness::compute("abcde", "bcdea"), mask![m m m m m])
        }
        #[test]
        fn all_wrong_chars() {
            assert_eq!(Correctness::compute("abcde", "xxxxx"), mask![w w w w w])
        }

        #[test]
        fn some_wrong_chars() {
            assert_eq!(Correctness::compute("abcde", "abcxx"), mask![c c c w w])
        }
        #[test]
        fn one_misplaced_char_rest_wrong() {
            assert_eq!(Correctness::compute("abcde", "xxxxa"), mask![w w w w m])
        }
    }
}
