# Getting started

This guide takes you from a fresh checkout to playing a game you wrote yourself.

## 1. Build cardlang

You need a Rust toolchain ([rustup.rs](https://rustup.rs)). From the repository root:

```sh
cargo build
cargo test        # optional: check everything passes
```

You can then run the interpreter with `cargo run`, or use the binary directly at `target/debug/cardlang`. The examples below use `cardlang`. Replace it with `cargo run --` (or the binary path) if it isn't on your `PATH`.

## 2. Write a game

We'll build **Race**. Two players take turns playing the top card of their hand onto a pile in the middle. The first player to empty their hand wins.

Create `race.card`:

```
.(Race to empty: take turns playing your top card.
  First player with no cards left wins.)
name race
players 2
current_player 1
stack middle
```

So far this only describes the table:

- `.( ... )` is a comment. It can span several lines.
- `name race` gives the game a name. It must be a single word.
- `players 2` creates two players. Each has an empty hand.
- `current_player 1` says player 1 goes first.
- `stack middle` creates an empty pile of cards called `middle`.

Every game also starts with a standard 52-card deck called `deck`.

> **Indent with spaces, not tabs**, and save the file with Unix (LF) line endings. Tabs and Windows line endings currently cause a lex error.

### Setting up

The `setup` function runs when the game starts. Add:

```
define setup(){
    shuffle(deck)
    deck > players end
}
```

- `shuffle(deck)` shuffles the deck.
- `deck > players end` is a **transfer**. The `>` moves cards from the left into the right. Moving into `players` deals one card at a time to each player in turn. `end` means "keep going until the deck is empty". Each player ends up with 26 cards.

### Taking a turn

The `player_move` function runs every time a player makes a move. It receives that player as its argument.

```
define is_out(p){
    return(count(p:hand) is 0)
}

define player_move(player){
    check(player:id is current_player)
    player:hand > middle
    if(is_out(player) is true){
        winner(player:id)
        end()
    }
    next_player()
}
```

Line by line:

1. `check(player:id is current_player)`: if it isn't this player's turn, stop here and ignore the move.
2. `player:hand > middle`: move the top card of the player's hand onto the middle pile.
3. `is_out` is a function you wrote. It returns `true` when the player's hand is empty. `if(is_out(player) is true)` calls it.
4. `winner(player:id)` records the winner and `end()` finishes the game.
5. `next_player()` passes the turn to the next player. After the last player it goes back to player 1.

> Write `if(is_out(player) is true)`, not `if(is_out(player))`. A condition has to be a comparison (`is` / `is not`), `true`, `false`, or an `&` of those. A bare function call is always treated as false. See [Known limitations](limitations.md).

The full file is in [`examples/race.card`](../examples/race.card).

## 3. Check it parses

```sh
cardlang test race.card
```

prints `Game loaded` and `success!` when the file is valid. If it isn't, it prints a parse error with a line number. See [Known limitations](limitations.md#reading-error-messages) for how to read it.

## 4. Play it

Start the interpreter with no arguments and type commands at the `>` prompt:

```
$ cardlang
Cardlang interpreter
> build ./race.card
Game loaded
> start
> show players
player 1 (cards: 26), player 2 (cards: 26)
> move 1
> show current_player
2
> move 1
> show players
player 1 (cards: 25), player 2 (cards: 26)
> move 2
> show middle
four hearts, jack clubs
> show game
active
```

The second `move 1` did nothing because it was player 2's turn, so the `check` stopped it. Your cards will differ because the deck is shuffled.

Keep alternating `move 1` and `move 2`. When player 1 plays their last card, `show game` prints:

```
game over
winners: 1
```

Moves made after the game is over are ignored. Type `start` to deal a fresh game, or `exit` to quit. All the commands are in [Using the CLI](cli.md).

## Next steps

- Read the [language reference](language-reference.md) for every statement and built-in function.
- Try changing Race: deal only part of the deck, add a second pile, or use `filter` to remove the picture cards before dealing.
