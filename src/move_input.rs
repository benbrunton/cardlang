// Reading a move typed at the CLI: `[player] [name] [cards] [/ cards]`
// e.g. `take 7h / 3c 4s`, `2 drop 9d`, `7h / 3c 4s`, `move`
use crate::cards::{parse_cards, starts_a_card};
use crate::runtime::MoveInput;

// words the CLI uses itself, so a game can't use them as move names
pub const RESERVED_WORDS: [&str; 5] = ["build", "start", "show", "exit", "help"];

pub fn parse_move(line: &str) -> Result<MoveInput, String> {
    let mut rest = line.trim();
    let mut input = MoveInput::default();

    // optional player number
    let first = rest.split_whitespace().next().unwrap_or("");
    if !first.is_empty() && first.chars().all(|c| c.is_ascii_digit()) {
        let n = first.parse::<usize>().map_err(|_| format!("'{}' is not a player number", first))?;
        input.player = Some(n);
        rest = rest[first.len()..].trim_start();
    }

    // optional move name: a word that isn't the start of a card
    let word = rest.split(|c: char| c.is_whitespace() || c == '/').next().unwrap_or("");
    if !word.is_empty() && !starts_a_card(word) {
        if !word.chars().next().map_or(false, |c| c.is_ascii_alphabetic()) {
            return Err(format!("'{}' is not a move name or a card", word));
        }
        input.name = Some(word.to_string());
        rest = rest[word.len()..].trim_start();
    }

    // `move 2` was the old way to move as player 2
    if input.name.as_deref() == Some("move") && input.player.is_none() {
        let next = rest.split_whitespace().next().unwrap_or("");
        if !next.is_empty() && next.chars().all(|c| c.is_ascii_digit()) {
            return Err(format!("to move as player {0}, put the number first: '{0} move'", next));
        }
    }

    if rest.is_empty() {
        return Ok(input);
    }

    for group in rest.split('/') {
        let cards = parse_cards(group)?;
        if cards.is_empty() {
            return Err("there is an empty group of cards around a '/'".to_string());
        }
        input.groups.push(cards);
    }
    Ok(input)
}

#[cfg(test)]
mod test {
    use super::*;

    fn card_names(input: &MoveInput) -> Vec<Vec<String>> {
        input.groups.iter().map(|g| g.iter().map(|c| c.to_string()).collect()).collect()
    }

    #[test]
    fn a_named_move_with_two_groups() {
        let input = parse_move("take 7h / 3c 4s").unwrap();
        assert_eq!(input.player, None);
        assert_eq!(input.name, Some("take".to_string()));
        assert_eq!(card_names(&input), vec!(
            vec!("seven hearts".to_string()),
            vec!("three clubs".to_string(), "four spades".to_string())
        ));
    }

    #[test]
    fn a_player_number_comes_first() {
        let input = parse_move("2 drop nine of diamonds").unwrap();
        assert_eq!(input.player, Some(2));
        assert_eq!(input.name, Some("drop".to_string()));
        assert_eq!(card_names(&input), vec!(vec!("nine diamonds".to_string())));
    }

    #[test]
    fn the_name_is_optional() {
        let input = parse_move("7h/3c").unwrap();
        assert_eq!(input.name, None);
        assert_eq!(input.groups.len(), 2);

        let input = parse_move("seven hearts").unwrap();
        assert_eq!(input.name, None);
        assert_eq!(input.groups.len(), 1);
    }

    #[test]
    fn a_move_can_have_no_cards() {
        assert_eq!(parse_move("move").unwrap(), MoveInput{ name: Some("move".to_string()), ..MoveInput::default() });
        assert_eq!(parse_move("2 move").unwrap().player, Some(2));
    }

    #[test]
    fn mistakes_are_reported() {
        assert!(parse_move("take 7x").is_err());
        assert!(parse_move("take 7h /").is_err());
        assert!(parse_move("take / 7h").is_err());
        assert!(parse_move("_take").is_err());
        assert_eq!(
            parse_move("move 2"),
            Err("to move as player 2, put the number first: '2 move'".to_string())
        );
    }
}
