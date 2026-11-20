# Using the CLI

The `cardlang` binary has two modes.

## Check a file: `cardlang test <file>`

```sh
cardlang test examples/race.card
```

This parses the file and prints `Game loaded` and `success!`, or a parse error. It does **not** run the game. It's a quick syntax check that's easy to use in scripts or CI.

## Interactive mode: `cardlang`

With no arguments, `cardlang` opens a prompt (`>`) that takes one command per line.

| Command | What it does |
|---|---|
| `build <path>` | Loads a game file. Prints `Game loaded` or a parse error. Building again replaces the current game. |
| `start` | Starts the game: resets the table, then runs `setup`. Running it again starts a fresh game. |
| `move <n>` | Runs `player_move` for player `n`. Ignored before `start` and after the game has ended. |
| `show <what>` | Prints part of the game state (see below). |
| `exit` | Quits. |

Commands other than `build` and `exit` do nothing until a game has been built.

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

Cards are listed bottom first, so the **last** card shown is the top of the pile. An empty pile prints a blank line. An unknown name prints `<name> not found`.

### Commands that crash the interpreter

These are bugs, not intended behaviour. Avoid them for now:

- `move <n>` when the game has no `player_move` function
- `move 0`, or a player number above the number of players
- `show player` without a number, or with a number above the number of players
