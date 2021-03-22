use std::cmp::Ordering;

use rand::{thread_rng, seq::SliceRandom};
use sha2::{Sha256, Digest};

use crate::util::{bytes_to_hex, gen_random_bytes};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Hand {
    Rock,
    Paper,
    Scissors,
}

impl Ord for Hand {
    fn cmp(&self, other: &Self) -> Ordering {
        use self::Hand::*;

        if self == other {
            return Ordering::Equal;
        }
        match (*self, *other) {
            (Rock, Paper) => Ordering::Less,
            (Paper, Scissors) => Ordering::Less,
            (Scissors, Rock) => Ordering::Less,

            _ => Ordering::Greater,
        }
    }
}

impl PartialOrd for Hand {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Hand {
    #[allow(dead_code)]
    pub fn cmp_as_str(lhs: Hand, rhs: Hand) -> &'static str {
        if lhs < rhs {
            "You lose"
        } else if lhs == rhs {
            "Draw"
        } else { // >
            "You win"
        }
    }

    // Abbreviation
    pub fn cmp_as_s(lhs: Hand, rhs: Hand) -> &'static str {
        if lhs < rhs {
            "L"
        } else if lhs == rhs {
            "D"
        } else { // >
            "W"
        }
    }

    pub fn as_icon(&self) -> &'static str {
        match *self {
            Hand::Rock => "✊🏼",
            Hand::Paper => "✋🏼",
            Hand::Scissors => "✌🏼",
        }
    }

    const CHOICES: [Hand; 3] = [Hand::Rock, Hand::Paper, Hand::Scissors];

    pub fn random() -> Hand {
        let mut rng = thread_rng();
        *Self::CHOICES.choose(&mut rng).unwrap()
    }
}

impl AsRef<str> for Hand {
    fn as_ref(&self) -> &str {
        match *self {
            Hand::Rock => "rock",
            Hand::Paper => "paper",
            Hand::Scissors => "scissors",
        }
    }
}

#[derive(Default)]
pub struct State {
    history: Vec<Round>,
    current_round: Option<Round>,
    selected_round: Option<usize>,

    win_count: usize,
    draw_count: usize,
    loss_count: usize,
}

impl State {
    pub fn new() -> State {
        let mut this = Self::default();
        this.new_round();
        this
    }

    pub fn history(&self) -> &[Round] {
        &self.history
    }

    pub fn num_rounds(&self) -> usize {
        self.history.len()
    }

    pub fn set_selected_round(&mut self, i: usize) {
        self.selected_round = Some(i);
    }

    pub fn selected_round(&self) -> Option<&Round> {
        self.selected_round
            .and_then(|i| {
                self.history.get(i)
            })
    }

    pub fn last_round(&self) -> Option<&Round> {
        self.history.last()
    }

    pub fn current_round(&self) -> Option<&Round> {
        self.current_round.as_ref()
    }

    fn new_round(&mut self) {
        if let Some(current_round) = self.current_round.take() {
            if current_round.human.is_some() {
                self.history.push(current_round);
            }
        }

        let next_i = self.num_rounds();
        self.current_round = Some(Round::random(next_i));
    }

    pub fn human_throw(&mut self, hand: Hand) {
        if let Some(current_round) = &mut self.current_round {
            if current_round.human.is_some() {
                panic!("WTF, throw again?");
            }

            current_round.human = Some(hand);
            match hand.cmp(&current_round.computer) {
                Ordering::Less => {
                    self.loss_count += 1;
                }
                Ordering::Equal => {
                    self.draw_count += 1;
                }
                Ordering::Greater => {
                    self.win_count += 1;
                }
            };
        }
        self.selected_round = None;
        self.new_round();
    }

    pub fn last_human_vs_computer(&self) -> Option<(Hand, Hand)> {
        match self.last_round() {
            Some(round) => match round.human {
                Some(human) => Some((human, round.computer)),
                None => None,
            },
            None => None,
        }

    }

    pub fn win_count(&self) -> usize { self.win_count }
    pub fn draw_count(&self) -> usize { self.draw_count }
    pub fn loss_count(&self) -> usize { self.loss_count }
}

pub struct Round {
    pub i: usize,
    pub computer: Hand,
    pub random_bytes: String,
    pub digest: String,
    pub human: Option<Hand>,
}

impl Round {
    /// Round number, 1-based
    pub fn num1(&self) -> usize {
        self.i + 1
    }

    pub fn random(i: usize) -> Round {
        let hand = Hand::random();
        let random_bytes = gen_random_bytes(32);
        let random_bytes_hex = bytes_to_hex(&random_bytes[..]);
        let concat_str = format!("{}_{}", random_bytes_hex, hand.as_ref());

        let digest = format!("{:x}", Sha256::digest(concat_str.as_bytes()));
        Round {
            i,
            computer: hand,
            random_bytes: random_bytes_hex,
            digest: digest,
            human: None,
        }
    }

    pub fn result_str(&self) -> Option<&str> {
        if let Some(human) = self.human {
            Some(Hand::cmp_as_str(human, self.computer))
        } else {
            None
        }
    }
}
