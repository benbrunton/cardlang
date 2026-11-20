# Using the CLI

The `cardlang` binary has three modes.

## Check a file: `cardlang test <file>`

```sh
cardlang test examples/race.card
```

This parses the file and prints `Game loaded` and `success!`, or an error. It does **not** run the game. It exits with status `0` when the file is valid, and `1` when it fails to parse, can't be read, or no file was given. That makes it usable in scripts and CI.

## Play a game: `cardlang play <file>`

```sh
cardlang play examples/simple_scopa.card
```

This loads the game, starts it, and leaves you at the prompt, ready to make the first move. It's the same as running `cardlang` and typing `build <file>` then `start`.

## Interactive mode: `cardlang`

With no arguments, `cardlang` opens a prompt (`>`) that takes one line at a time. A line is either a command or a move.

### Tab completion and history

In a terminal, press **Tab** to complete what you're typing. Press it twice to list the options.

| You type | Tab offers |
|---|---|
| the first word | the commands, and the game's move names (e.g. `take`, `drop`) |
| `build ` | folders and `.card` files, e.g. `build ex<Tab>` → `build examples/` |
| `show ` | what can be shown: `game`, `players`, the game's piles … then player numbers and their piles |
| a move | cards in the current player's hand, then after a `/` the cards on the table, as short codes (`7h`) |

Move names come from the game's `valid_moves(...)` calls.

**Up** and **Down** go through the lines you've typed, so you can repeat or edit a move. **Ctrl-C** clears the line and **Ctrl-D** quits.

When input is piped in (for example `cardlang < moves.txt`), lines are read plainly, without completion or history.

### Commands

| Command | What it does |
|---|---|
| `build <path>` | Loads a game file. Prints `Game loaded` or an error. Building again replaces the current game. |
| `start` | Starts the game: resets the table, runs `setup`, then shows [the table](#the-table). Running it again starts a fresh game. |
| `show <what>` | Prints part of the game state (see below). |
| `help` | Lists the commands, the move syntax and the keys. |
| `exit` | Quits. End of input (Ctrl-D, or the end of a piped script) also quits. |

Empty lines are ignored. Until a game has been built, moves, `start` and `show` print `no game loaded - use 'build <file>' first`. After `build` but before `start`, nothing has been dealt yet, so `show` prints `the game hasn't started - type 'start' to deal` (except `show game` and `show name`).

These five words are reserved. A game can't define a function called `build`, `start`, `show`, `help` or `exit`.

### Moves

Any other line is a move:

```
[player] [name] [cards] [/ cards]
```

| Part | Meaning | Example |
|---|---|---|
| player | Optional. The number of the player making the move. Without it, the move is for the current player. | `2 drop 9d` |
| name | Optional. Which move to make; the game's `valid_moves` decides what's allowed. | `take`, `drop` |
| cards | Optional. The cards being played. They become `move:cards`. | `9d` |
| `/ cards` | Optional. A second group of cards, such as what you're capturing. They become `move:target`. | `/ 4c 5s` |

Examples from [Simple Scopa](../examples/simple_scopa.card):

```
> take 9d / 4c 5s        capture the 4 and 5 with the 9 of diamonds
> drop nine of diamonds  put a card in the middle
> 9d / 4c 5s             no name: the game tries take first, then drop
> 2 drop 8h              make a move as player 2
```

For games where a move needs no name or cards, such as [Race](../examples/race.card), type any word: `move`, or `2 move` for player 2.

**Writing cards.** Separate cards with spaces or commas. Each card can be written as:
- a short code: rank then suit. Ranks are `a`, `2`–`10` (or `t`), `j`, `q`, `k`; suits are `s`, `h`, `c`, `d`. For example `7h`, `10d`, `ks`, `AS`.
- a name: `seven hearts`, `seven of hearts`, `7 hearts`, `queen of spades`.

Case doesn't matter. A word that looks like a card (`as`, `seven`, `10`) is always read as a card, so it can't be used as a move name.

### The table

After `start` and after every accepted move, the CLI shows the state of the game:

```
player 2 to play
  deck: 42 cards
  middle: three hearts, ten clubs, ace clubs, ten diamonds
  player 2 hand: six clubs, ace diamonds, eight hearts
  collection: player 1: 0 cards, player 2: 2 cards
```

- the first line says whose turn it is, or `game over - winners: …`
- the deck shows only how many cards are left, because it's face down
- every pile declared with `stack` is listed; a pile with more than 10 cards shows its size and top card
- the current player's hand is shown in full, so the CLI works for players passing the keyboard round
- piles declared with `stack <name> per player` show how many cards each player has

The `show` commands below give more detail.

### When a move is ignored

A move either happens completely or not at all. If it's rejected, nothing in the game changes, and the CLI says why.

| Situation | Message |
|---|---|
| before `start` | `move ignored: the game has not started - use 'start'` |
| after the game has ended | `move ignored: the game is over` |
| the game has no `player_move` function | `move ignored: this game does not define player_move` |
| no such player (e.g. `0 move`, or `3 move` with 2 players) | `move ignored: there is no player 3 (players: 2)` |
| a `check` with a [message](language-reference.md#check-messages) fails | `move ignored: put a / before the cards you're capturing, e.g. take 9d / 4c 5s` |
| a `check` without a message fails | `move ignored: not allowed: check failed in take: sum(move:target) is sum(move:cards)` |
| a transfer can't happen | `move ignored: not allowed: transfer failed in drop: player:hand > middle taking move:cards (king clubs is not in player:hand)` |
| no move name given, and no move works | `move ignored: not allowed: no valid move - check failed in take: …; check failed in drop: …` (one reason per move tried) |
| an unknown move name | `move ignored: not allowed: unknown move 'play' - valid moves: take, drop` |
| something that isn't a card | `'7x' is not a card (try 7h, 10d, ks or 'seven hearts')` |
| the old `move 2` syntax | `to move as player 2, put the number first: '2 move'` |

Games can give their own messages, as Simple Scopa does. Otherwise the reason shows the game's rule as written in the `.card` file.

### `show`

| Command | Prints | Example output |
|---|---|---|
| `show game` | status (`pending`, `active` or `game over`) and any winners | `game over`<br>`winners: 1` |
| `show name` | the game's `name` | `race` |
| `show players` | every player and how many cards are in their hand | `player 1 (cards: 26), player 2 (cards: 26)` |
| `show player <n>` | player `n`'s hand | `king diamonds, jack clubs` |
| `show player <n> <stack>` | one of player `n`'s own stacks (`hand`, or one declared with `stack <name> per player`) | `jack spades, jack diamonds` |
| `show current_player` | whose turn it is | `2` |
| `show deck` | the cards in the deck | `ace spades, two spades, …` |
| `show <pile>` | the cards in a pile declared with `stack` | `four hearts` |

Cards are listed bottom first, so the **last** card shown is the top of the pile. An empty pile prints a blank line. An unknown name prints `<name> not found`. `show player` without a valid number prints `usage: show player <number>`. A player that doesn't exist prints `there is no player <n>`, and an unknown stack prints `player 1 has no stack '<name>'`.
