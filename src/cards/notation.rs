// Reading cards typed by a player: short codes (`7h`, `10d`, `td`, `ks`)
// or names (`seven hearts`, `seven of hearts`, `7 hearts`).
use super::{Card, Rank, Suit};

fn rank_from_word(word: &str) -> Option<Rank> {
    let rank = match word {
        "a" | "ace" | "1" => Rank::Ace,
        "2" | "two" => Rank::Two,
        "3" | "three" => Rank::Three,
        "4" | "four" => Rank::Four,
        "5" | "five" => Rank::Five,
        "6" | "six" => Rank::Six,
        "7" | "seven" => Rank::Seven,
        "8" | "eight" => Rank::Eight,
        "9" | "nine" => Rank::Nine,
        "t" | "10" | "ten" => Rank::Ten,
        "j" | "jack" => Rank::Jack,
        "q" | "queen" => Rank::Queen,
        "k" | "king" => Rank::King,
        _ => return None
    };
    Some(rank)
}

fn suit_from_word(word: &str) -> Option<Suit> {
    let suit = match word {
        "s" | "spade" | "spades" => Suit::Spades,
        "h" | "heart" | "hearts" => Suit::Hearts,
        "c" | "club" | "clubs" => Suit::Clubs,
        "d" | "diamond" | "diamonds" => Suit::Diamonds,
        _ => return None
    };
    Some(suit)
}

// `7h`, `10d`, `ks`
fn card_from_code(word: &str) -> Option<Card> {
    if word.len() < 2 || !word.is_ascii() {
        return None;
    }
    let (rank, suit) = word.split_at(word.len() - 1);
    Some(Card { rank: rank_from_word(rank)?, suit: suit_from_word(suit)? })
}

// true if a word could start a card, so it can't be a move name
pub fn starts_a_card(word: &str) -> bool {
    let word = word.to_lowercase();
    // single letters like `a` or `k` are rank codes only when followed by a suit
    let is_rank_word = word.len() > 1 && rank_from_word(&word).is_some();
    let is_number = word.chars().all(|c| c.is_ascii_digit());
    is_rank_word || is_number || card_from_code(&word).is_some()
}

// Parses a list of cards separated by spaces and/or commas.
pub fn parse_cards(input: &str) -> Result<Vec<Card>, String> {
    let lowered = input.to_lowercase();
    let words: Vec<&str> = lowered
        .split(|c: char| c.is_whitespace() || c == ',')
        .filter(|w| !w.is_empty())
        .collect();

    let mut cards = vec!();
    let mut i = 0;
    while i < words.len() {
        let word = words[i];
        let card = if let Some(card) = card_from_code(word) {
            i += 1;
            card
        } else if let Some(rank) = rank_from_word(word) {
            // `seven hearts` or `seven of hearts`
            let mut next = i + 1;
            if words.get(next) == Some(&"of") {
                next += 1;
            }
            let suit = words.get(next)
                .and_then(|w| suit_from_word(w))
                .ok_or_else(|| format!("expected a suit after '{}'", word))?;
            i = next + 1;
            Card { rank, suit }
        } else {
            return Err(format!("'{}' is not a card (try 7h, 10d, ks or 'seven hearts')", word));
        };

        if cards.contains(&card) {
            return Err(format!("{} is listed twice", card));
        }
        cards.push(card);
    }
    Ok(cards)
}

#[cfg(test)]
mod test {
    use super::*;

    fn names(input: &str) -> Vec<String> {
        parse_cards(input).unwrap().iter().map(|c| c.to_string()).collect()
    }

    #[test]
    fn it_reads_short_codes() {
        assert_eq!(names("7h 10d ks AS 2c"), vec!(
            "seven hearts", "ten diamonds", "king spades", "ace spades", "two clubs"
        ));
        assert_eq!(names("td"), vec!("ten diamonds"));
    }

    #[test]
    fn it_reads_names_with_or_without_of() {
        assert_eq!(names("seven hearts, Queen of Spades 10 clubs"), vec!(
            "seven hearts", "queen spades", "ten clubs"
        ));
    }

    #[test]
    fn it_accepts_commas_and_spaces_between_cards() {
        assert_eq!(names("3c, 4s,5d  6h"), vec!("three clubs", "four spades", "five diamonds", "six hearts"));
    }

    #[test]
    fn an_empty_list_is_no_cards() {
        assert!(parse_cards("  ").unwrap().is_empty());
    }

    #[test]
    fn it_rejects_things_that_are_not_cards() {
        assert!(parse_cards("7x").is_err());
        assert!(parse_cards("seven").is_err());
        assert!(parse_cards("seven of").is_err());
        assert!(parse_cards("joker").is_err());
        assert!(parse_cards("11h").is_err());
    }

    #[test]
    fn it_rejects_the_same_card_twice() {
        assert!(parse_cards("10d td").is_err());
    }

    #[test]
    fn it_knows_which_words_start_a_card() {
        assert!(starts_a_card("7h"));
        assert!(starts_a_card("seven"));
        assert!(starts_a_card("10"));
        assert!(starts_a_card("KS"));
        assert!(!starts_a_card("take"));
        assert!(!starts_a_card("drop"));
        assert!(!starts_a_card("move"));
    }
}
