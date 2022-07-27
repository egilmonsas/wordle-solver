use itertools::Itertools;
use once_cell::sync::OnceCell;

use crate::{Correctness, Guess, Guesser, Word, DICTIONARY};
use std::borrow::Cow;

static INITIAL: OnceCell<Vec<(&'static Word, usize)>> = OnceCell::new();
static PATTERNS: OnceCell<Vec<[Correctness; 5]>> = OnceCell::new();

pub struct Solver {
    remaining: Cow<'static, Vec<(&'static Word, usize)>>,
    patterns: Cow<'static, Vec<[Correctness; 5]>>,
}

impl Solver {
    pub fn new() -> Self {
        Solver {
            remaining: Cow::Borrowed(INITIAL.get_or_init(|| {
                Vec::from_iter(DICTIONARY.lines().map(|line| {
                    let (word, count) = line
                        .split_once(' ')
                        .expect("Every line must be [word] [freq]");

                    let count: usize = count.parse().expect("every count must be a number");
                    let word = word
                        .as_bytes()
                        .try_into()
                        .expect("Every word must be 5 characters");
                    (word, count)
                }))
            })),
            patterns: Cow::Borrowed(PATTERNS.get_or_init(|| {
                Correctness::patterns().collect()
            })),
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct Candidate {
    word: &'static Word,
    goodness: f64,
}

impl Guesser for Solver {
    fn guess(&mut self, history: &[Guess]) -> Word {
        if history.is_empty() {
            self.patterns = Cow::Borrowed(PATTERNS.get().unwrap());
            return *b"tares";
        } else {
            assert!(!self.patterns.is_empty())
        }

        if let Some(last) = history.last() {
            // update self.remaining based on history
            self.remaining
                .to_mut()
                .retain(|(word, _)| last.matches(word));
        }
        let remaining_count: usize = self.remaining.iter().map(|&(_, c)| c).sum();
        let mut best: Option<Candidate> = None;
        let mut ccount: usize = 0;

        for &(word, _) in &*self.remaining {
            let mut sum = 0.0;
            let check_pattern = |pattern: &[Correctness; 5]| {
                let mut in_pattern_total: usize = 0;
                for (candidate, count) in &*self.remaining {
                    let g = Guess {
                        word: *word,
                        mask: *pattern,
                    };
                    if g.matches(candidate) {
                        in_pattern_total += count;
                    }
                    ccount = *count;
                }
                if in_pattern_total == 0 {
                    return false;
                };
                let p_of_this_pattern = in_pattern_total as f64 / remaining_count as f64;
                sum += p_of_this_pattern * p_of_this_pattern.log2();
                return true;
            };

            if matches!(self.patterns, Cow::Owned(_)) {
                self.patterns.to_mut().retain(check_pattern);
            } else {
                self.patterns = Cow::Owned(
                    self.patterns
                        .iter()
                        .copied()
                        .filter(check_pattern)
                        .collect(),
                );
            }

            let p_word = ccount as f64 / remaining_count as f64;
            let goodness = p_word * -sum;
            if let Some(c) = best {
                // Is this one better?
                if goodness > c.goodness {
                    best = Some(Candidate {
                        word,
                        goodness: goodness,
                    })
                }
            } else {
                best = Some(Candidate {
                    word,
                    goodness: goodness,
                })
            }
        }
        *best.unwrap().word
    }
}
