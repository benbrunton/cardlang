# Using the CLI

The `cardlang` binary has two modes.

## Check a file: `cardlang test <file>`

```sh
cardlang test examples/race.card
```

This parses the file and prints `Game loaded` and `success!`, or a parse error. It does **not** run the game. It exits with status `0` when the file is valid, and `1` when it fails to parse, can't be read, or no file was given. That makes it usable in scripts and CI.

## Interactive mode: `cardlang`

With no arguments, `cardlang` opens a prompt (`>`) that takes one command per line.

| Command | What it does |
|---|---|
| `build <path>` | Loads a game file. Prints `Game loaded` or a parse error. Building again replaces the current game. |
| `start` | Starts the game: resets the table, then runs `setup`. Running it again starts a fresh game. |
| `move <n>` | Runs `player_move` for player `n`. If the move can't happen, it prints `move ignored: <reason>` (see below). |
| `show <what>` | Prints part of the game state (see below). |
| `exit` | Quits. End of input (Ctrl-D, or the end of a piped script) also quits. |

Empty lines are ignored. Until a game has been built, `start`, `move` and `show` print `no game loaded - use 'build <file>' first`.

### `show`

| Command | Prints | Example output |
|---|---|---|
| `show game` | status (`pending`, `active` or `game over`) and any winners | `game over`<br>`winners: 1` |
| `show name` | the game's `name` | `race` |
| `show players` | every player and how many cards they hold | `player 1 (cards: 26), player 2 (cards: 26)` |
| `show player <n>` | player `n`'s hand. `show player <n> hand` also works. | `king diamonds, jack clubs` |
| `show current_player` | whose turn it is | `2` |
| `show deck` | the cards in the deck | `ace spades, two spades, …` |
| `show <pile>` | the cards in a pile declared with `stack` | `four hearts` |

Cards are listed bottom first, so the **last** card shown is the top of the pile. An empty pile prints a blank line. An unknown name prints `<name> not found`. `show player` without a valid number prints `usage: show player <number>`, and a player that doesn't exist prints `there is no player <n>`.

### When a move is ignored

| Situation | Message |
|---|---|
| `move` without a number, or with something that isn't a number | `usage: move <player number>` |
| before `start` | `move ignored: the game has not started - use 'start'` |
| after the game has ended | `move ignored: the game is over` |
| the game has no `player_move` function | `move ignored: this game does not define player_move` |
| no such player (e.g. `move 0`, or `move 3` with 2 players) | `move ignored: there is no player 3 (players: 2)` |

A `check` that fails inside `player_move` also stops the move, but it prints nothing.
