use std::fmt;

mod player;
pub use player::*;
mod notation;
pub use notation::{card_code, parse_cards, starts_a_card};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Suit {
    Spades,
    Hearts,
    Clubs,
    Diamonds
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Rank {
    Ace,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Card {
    suit: Suit,
    rank: Rank
}

impl Card {
    pub fn get_suit_str(&self) -> String {
        format!("{:?}", self.suit)
    }

    pub fn get_rank_str(&self) -> String {
        format!("{:?}", self.rank)
    }

    // Ace 1, Two 2 ... Jack 11, Queen 12, King 13
    pub fn default_value(&self) -> f64 {
        (self.rank as usize + 1) as f64
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let debug_str = format!("{:?} {:?}", self.rank, self.suit).to_lowercase();
        write!(f, "{}", debug_str)
    }
}



fn get_suit_array() -> [Suit; 4] {
    [Suit::Spades, Suit::Hearts, Suit::Clubs, Suit::Diamonds]
}

fn get_rank_array() -> [Rank; 13] {
    [
        Rank::Ace,
        Rank::Two,
        Rank::Three,
        Rank::Four,
        Rank::Five,
        Rank::Six,
        Rank::Seven,
        Rank::Eight,
        Rank::Nine,
        Rank::Ten,
        Rank::Jack,
        Rank::Queen,
        Rank::King,
    ]
}

pub fn standard_deck() -> Vec<Card> {
    let suits = get_suit_array();
    let ranks = get_rank_array();
    let mut cards = vec![];
    for suit in &suits {
        for rank in &ranks {
            let card = Card {
                rank: *rank,
                suit: *suit,
            };
            cards.push(card);
        }
    }
    cards
}