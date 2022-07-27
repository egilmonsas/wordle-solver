use std::{collections::HashSet};
pub mod solver;

pub const DICTIONARY: &str = include_str!("../data/dictionary.txt");
pub const GAMES: &str = include_str!("../data/answers.txt");

pub type Word = [u8;5];

pub struct Wordle {
    dictionary: HashSet<&'static Word>,
}
impl Wordle {
    pub fn new() -> Self {
        Self {
            dictionary: HashSet::from_iter(DICTIONARY.lines().map(|line| {
                line.split_once(' ')
                    .expect("Every line must be [word] [freq]")
                    .0
                    .as_bytes()
                    .try_into()
                    .expect("Every word must be 5 character utf8")
            })),
        }
    }

    pub fn play<G: Guesser>(&self, answer: &'static Word, mut guesser: G) -> Option<usize> {
        let mut history = Vec::new();
        for i in 1..=32 {
            let guess = guesser.guess(&history[..]);
            if &guess == answer {
                return Some(i);
            }
            assert!(self.dictionary.contains(&guess));
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
    word: Word,
    mask: [Correctness; 5],
}
impl Guess{
    pub fn matches(&self, word: &Word) -> bool {
        Correctness::compute(word, &self.word) == self.mask
    }
    pub fn matches_(&self, word: &Word) -> bool {

        assert_eq!(self.word.len(), 5);
        assert_eq!(word.len(), 5);

        let mut used = [false; 5];


        //Check correct letters
        for (i, ((g, &m), w)) in self
            .word
            .iter()
            .zip(&self.mask)
            .zip(word.iter())
            .enumerate()
        {
            if m == Correctness::Correct {
                if g != w {
                    return false;
                } else {
                    used[i] = true;
                    continue;
                }
            }
        }

        //Check misplaced letters
        for (i, (w,&m)) in
            word
            .iter()
            .zip(&self.mask)
            .enumerate()
        {
            if m==Correctness::Correct{
                //Must be correct, or wed have returned in the earlier loop
                continue;
            }
            let mut plausible=true;
            if  self.word
                .iter()
                .zip(&self.mask)
                .enumerate()
                .any(|(j, (g, m))| {
                    if g != w {
                        return false;
                    }

                    if used[j] {
                        return false;
                    }
                    // We are now looking at an x in both words, the color status will tell us if using this x is ok
                    match m {
                        Correctness::Correct => unreachable!("All correct should have resulted in return or being used at this point"),
                        Correctness::Misplaced if j==i => {
                            // 'w' was yellow in this same position last time around, so it must be 
                            plausible=true; 
                            return true
                        },
                        Correctness::Misplaced => {
                            // 'w' was yellow in this same position last time around, so it must be 
                            used[j]=true; 
                            return true
                        },
                        Correctness::Wrong => {
                            //Todo: early return
                            plausible=false;
                            return false;
                        }
                    }
                })
                &&plausible
            {
                // The character 'w' was yellow in a previous guess
                assert!(plausible)
            } else if !plausible{
                return false;
            } else {
                // We have no information about 'w', so word might still match
                return true;
            }
                        
        }
        true
    }
}

pub trait Guesser {
    fn guess(&mut self, history: &[Guess]) -> Word;
}

impl Correctness {
    fn compute(answer: &Word, guess: &Word) -> [Correctness; 5] {
        assert_eq!(answer.len(), 5);
        assert_eq!(guess.len(), 5);
        let mut c = [Correctness::Wrong; 5];

        //Mark fields green
        for (i, (a, g)) in answer.iter().zip(guess.iter()).enumerate() {
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
        for (i, g) in guess.iter().enumerate() {
            if c[i] == Correctness::Correct {
                continue;
            }
            if answer.iter().enumerate().any(|(i, a)| {
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
    pub fn patterns() -> impl Iterator<Item=[Self;5]>{
        itertools::iproduct!(
            [Self::Correct, Self::Misplaced,Self::Wrong],
            [Self::Correct, Self::Misplaced,Self::Wrong],
            [Self::Correct, Self::Misplaced,Self::Wrong],
            [Self::Correct, Self::Misplaced,Self::Wrong],
            [Self::Correct, Self::Misplaced,Self::Wrong]
        )
        .map(|(a,b,c,d,e)| [a,b,c,d,e])
    }
}

impl Guesser for fn(history: &[Guess]) -> Word {
    fn guess(&mut self, history: &[Guess]) -> Word {
        (*self)(history)
    }
}

#[cfg(test)]
macro_rules! guesser {
    (|$history:ident| $impl:block) => {{
        struct G;
        impl Guesser for G {
            fn guess(&mut self, $history: &[Guess]) -> $crate::Word {
                $impl
            }
        }
        G
    }};
}

#[cfg(test)]
macro_rules!  mask {
    (c) => {$crate::Correctness::Correct};
    (m) => {$crate::Correctness::Misplaced};
    (w) => {$crate::Correctness::Wrong};
    ($($c:tt)+)=>{[
        $(mask!($c)),+
    ]}
}

#[cfg(test)]
mod tests {
    mod guess_matcher{
        use crate::Guess;

        macro_rules! check {
            ($prev:literal + [$($mask:tt)+] allows $next:literal) => {
                assert!(Guess{
                    word:*$prev,
                    mask:mask![$($mask )+]
                }
                .matches($next));
                assert_eq!($crate::Correctness::compute($next,$prev), mask![$($mask )+]);
            };
            ($prev:literal + [$($mask:tt)+] disallows $next:literal) => {
                assert!(!Guess{
                    word:*$prev,
                    mask:mask![$($mask )+]
                }
                .matches($next));  
                assert_ne!($crate::Correctness::compute($next,$prev), mask![$($mask )+]);

            };
        }
        #[test]
        fn matches(){
            check!(b"abcde" + [c c c c c] allows b"abcde");
            check!(b"abcde" + [c c c c c] disallows b"abcdf");
            check!(b"abcde" + [w w w w w] allows b"xxxxx");
            check!(b"abcde" + [m m m m m] allows b"eabcd");
            check!(b"abcde" + [w m m m m] disallows b"abcde");
            check!(b"baaaa" + [w c m w w] allows b"aaccc");
        }
    }
    mod game {

        use crate::{Guess, Guesser, Wordle};

        #[test]
        fn genius() {
            let w = Wordle::new();
            let guesser = guesser!(|_history| { *b"right" });
            assert_eq!(w.play(b"right", guesser), Some(1));
        }
        #[test]
        fn magnificent() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 1 {
                    return *b"right";
                }
                return *b"wrong";
            });
            assert_eq!(w.play(b"right", guesser), Some(2));
        }
        #[test]
        fn impressive() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 2 {
                    return *b"right";
                }
                return *b"wrong";
            });
            assert_eq!(w.play(b"right", guesser), Some(3));
        }
        #[test]
        fn splendid() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 3 {
                    return *b"right";
                }
                return *b"wrong";
            });
            assert_eq!(w.play(b"right", guesser), Some(4));
        }
        #[test]
        fn great() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 4 {
                    return *b"right";
                }
                return *b"wrong";
            });
            assert_eq!(w.play(b"right", guesser), Some(5));
        }
        #[test]
        fn phew() {
            let w = Wordle::new();
            let guesser = guesser!(|history| {
                if history.len() == 5 {
                    return *b"right";
                }
                return *b"wrong";
            });
            assert_eq!(w.play(b"right", guesser), Some(6));
        }
        #[test]
        fn retarded() {
            let w = Wordle::new();
            let guesser = guesser!(|_history| {
                return *b"wrong";
            });
            assert_eq!(w.play(b"right", guesser), None);
        }
    }
    mod compute {
        use crate::Correctness;

        #[test]
        fn all_correct_chars() {
            assert_eq!(Correctness::compute(b"abcde", b"abcde"), mask![c c c c c])
        }
        #[test]
        fn all_misplaced() {
            assert_eq!(Correctness::compute(b"abcde", b"bcdea"), mask![m m m m m])
        }
        #[test]
        fn all_wrong_chars() {
            assert_eq!(Correctness::compute(b"abcde", b"xxxxx"), mask![w w w w w])
        }

        #[test]
        fn some_wrong_chars() {
            assert_eq!(Correctness::compute(b"abcde", b"abcxx"), mask![c c c w w])
        }
        #[test]
        fn one_misplaced_char_rest_wrong() {
            assert_eq!(Correctness::compute(b"abcde", b"xxxxa"), mask![w w w w m])
        }
    }
}
