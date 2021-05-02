use std::fmt;

mod player;
pub use player::*;

#[derive(Debug, Copy, Clone)]
pub enum Suit {
    Spades,
    Hearts,
    Clubs,
    Diamonds
}

#[derive(Debug, Copy, Clone)]
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

pub struct Card {
    suit: Suit,
    rank: Rank
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
    let mut cards = vec!();
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