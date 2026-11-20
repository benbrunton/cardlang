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

### Setting up

The `setup` function runs when the game starts. Add:

```
def setup(){
    shuffle(deck)
    deck > players end
}
```

- `shuffle(deck)` shuffles the deck.
- `deck > players end` is a **transfer**. The `>` moves cards from the left into the right. Moving into `players` deals one card at a time to each player in turn. `end` means "keep going until the deck is empty". Each player ends up with 26 cards.

### Taking a turn

A **move** is something a player can do on their turn. Race has just one, `play`:

```
def is_out(p){
    return(count(p:hand) is 0)
}

move play(player){
    player:hand > middle
    if(is_out(player)){
        winner(player:id)
        end()
    }
}
```

`move` works like `def`, but it marks this as something a player can do. Players type its name, `play`, to make the move, and it receives the player making it.

Line by line:

1. `player:hand > middle`: move the top card of the player's hand onto the middle pile.
2. `is_out` is an ordinary function (`def`). It returns `true` when the player's hand is empty. `if(is_out(player))` calls it and runs the block when it returns `true`.
3. `winner(player:id)` records the winner and `end()` finishes the game.

You don't need to check whose turn it is, or pass the turn on: the interpreter does both. Only the current player can make a move. After a move, the turn passes to the next player, and after the last player it goes back to player 1.

The full file is in [`examples/race.card`](../examples/race.card).

## 3. Check it parses

```sh
cardlang test race.card
```

prints `Game loaded` and `success!` when the file is valid. If it isn't, it prints an error with the line number, such as `error: line 12: unexpected word or symbol`. See [Known limitations](limitations.md#reading-error-messages) for common causes.

## 4. Play it

Start the interpreter with no arguments and type commands at the `>` prompt:

```
$ cardlang
Cardlang interpreter
> build ./race.card
Game loaded
> start
player 1 to play
  middle: empty
  your hand (player 1): ad, 6h, …
> play
player 2 to play
  middle: 9h
  your hand (player 2): qc, 3c, …
> 1 play
move ignored: it's not your turn
> play
player 1 to play
  middle: 9h, 3h
  your hand (player 1): ad, 6h, …
```

`start` deals the cards and shows the table: whose turn it is, the piles, and the current player's hand (shortened here; a hand is listed bottom first, so the card played is the last one). Cards are shown as short codes: `9h` is the nine of hearts, `ad` the ace of diamonds. Any line that isn't one of the CLI's own commands is a move. `play` makes Race's only move for the current player. After each move the table is shown again.

`1 play` tries to move as player 1. It's rejected because it's player 2's turn. Your cards will differ because the deck is shuffled.

Keep typing `play`. When player 1 plays their last card, the table shows:

```
game over - winners: 1
```

Moves made after the game is over are ignored. Type `start` to deal a fresh game, `help` to list the commands, or `exit` to quit. All the commands are in [Using the CLI](cli.md).

Two shortcuts:
- `cardlang play race.card` builds and starts the game in one step.
- Press **Tab** to complete commands, file names, moves and cards. For example, `build ex<Tab>` completes to `build examples/`. The up arrow repeats earlier lines.

## Next steps

- Read the [language reference](language-reference.md) for every statement and built-in function.
- Try changing Race: deal only part of the deck (`deck > players 5`), add a second pile, or use `filter` to remove the picture cards before dealing.
- Look at [`examples/simple_scopa.card`](../examples/simple_scopa.card). It's a fuller game where players choose which cards to play: named moves, card values, and capturing cards into a pile each player owns.
- Then [`examples/scopa.card`](../examples/scopa.card): full Scopa, with scoring over several hands. Its opening comment explains the rules in plain English.
