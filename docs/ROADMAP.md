# Roadmap

All known bugs and planned work in one place, in priority order. It replaces the old todo list in the README.

**Keeping it current:** when you fix something, mark it ✅ here, then update [limitations.md](limitations.md) and the [language reference](language-reference.md) in the same PR.

Status: ✅ done · 🟡 partly done · ⬜ not started · ⚠️ needs a design decision first

## Milestone 1: never crash, never hang ✅

The interpreter now reports an error instead of panicking or looping. A randomised parser test and CLI tests guard against regressions.

| # | Item | Status |
|---|---|---|
| 1 | CI on GitHub Actions: build, test, check example games parse, clippy (warnings not yet fatal) | ✅ |
| 2 | End of input (Ctrl-D or a closed pipe) made the CLI loop forever printing `unrecognised command`. It now exits. | ✅ |
| 3 | CLI crashes: `move` with no `player_move`; `move 0` or an out-of-range player; `show player` with a missing or out-of-range number. `move x` silently moved player 1. All now print an error. | ✅ |
| 4 | Parser crashes: a declaration with nothing after it (`name`, `players`, `stack`, `current_player`); an unfinished comparison (`check(1 is)`, `check(1 &)`); a transfer with no destination (`x >`); `players > deck`. All are now parse errors. | ✅ |
| 4b | Runtime crashes: `winner()` and `count()` with no arguments; `unknown > deck end`. | ✅ |
| 5 | `cardlang test` exited `0` on failure. It now exits `1`. | ✅ |

## Milestone 2: correct behaviour ✅

Bugs that gave game authors wrong results without any error.

| # | Item | Status | Notes |
|---|---|---|---|
| 6 | `check` and `return` inside an `if` only ended the `if` block | ✅ | Statements now return a control-flow result that `if` passes up to the function |
| 7 | `&` bound wrongly: `a is 1 & b is 2` was parsed as `a is (1 & (b is 2))` | ✅ | New precedence-based expression parser: `is` > `&` > `or`, with parentheses for grouping. Added `or`, `else` and `else if`. |
| 8 | A bare boolean or function call used as a condition (`if(f(x))`) was always false | ✅ | A condition is true when it evaluates to the boolean `true` |
| 9 | Invalid input was silently accepted | ✅ | `players two`, `name 5`, stray `}`/`)`/numbers, a number after a transfer, `if` without `(`, a block without `{`, a missing `)` and unclosed comments are all errors now. `.test(...)` blocks are skipped explicitly. |
| 10 | Unknown pile names in transfers were silently ignored | 🟡 | Now reported on stderr when the transfer runs. Checking at parse time is still to do: it needs to know which names are parameters. |
| 11 | Error line numbers were wrong after multi-line comments, newline tokens had the next line's number, and some errors reported line `0` | ✅ | |
| 12 | Tabs and `\r` (Windows line endings) were lex errors | ✅ | Treated as whitespace |
| 13 | Error messages were raw Rust debug output | ✅ | Now `error: line 5: unexpected word or symbol`. They could still say what was expected; see item 13c. |
| 13b | `winner(x)` where `x` isn't a number recorded player `0` | ✅ | Prints an error instead |
| 13c | Error messages should say what was found and what was expected (`line 5: expected a number after 'players', found 'two'`) | ⬜ | Needs the token stored in `ParseError` |

## Milestone 3: Simple Scopa playable ✅

[`examples/simple_scopa.card`](../examples/simple_scopa.card) can be played from start to finish at the CLI. A test plays whole games through it.

| # | Item | Status | Notes |
|---|---|---|---|
| 14 | Transfer counts: `deck > middle 4`, `deck > players 3` (per player) | ✅ | |
| 16 | Move input: `[player] [name] [cards] [/ cards]` at the CLI, no `move` keyword, current player by default; `player_move(player, move)` with `move:name`, `move:cards`, `move:target` | ✅ | Cards as codes (`7h`) or names (`seven of hearts`) |
| 17 | Moving specific cards: `a > b taking cards`, all or nothing | ✅ | |
| 18 | `cards_in_stack(cards, stack)` built-in | ✅ | |
| 19 | `valid_moves(take, drop)`: run the named move, or the first that works | ✅ | |
| 19b | A failed move changes nothing: game state is restored | ✅ | |
| 20 | Card values: `card:value` (Ace 1 … King 13), a `values { }` table, `sum()`, `+ - < >` | ✅ | |
| 20b | Per-player piles (`stack collection per player`) and players by number (`players:2:hand`) | ✅ | Also fixed `p:<pile>` always meaning the hand |
| 23 | `def` replaces `define` | ✅ | |

### Design decisions made

1. `def` replaces `define` everywhere.
2. Moves are typed without a `move` keyword: `[player] [name] [cards] [/ cards]`. The player defaults to the current player. Without a name, `valid_moves` tries each move in order. The CLI's command words are reserved.
3. Specific cards are moved with a trailing `taking`: `middle > player:collection taking move:target`.
4. Card values have built-in defaults (Ace 1 … King 13), an optional `values` table, `sum()`, and `+ - < >`.

## Milestone 4: code health

Spread across the other milestones rather than done in one go.

| # | Item | Status |
|---|---|---|
| 27 | Fix the 2 compiler warnings (unused `Peekable` import, unused `Game.ast` field) | ⬜ |
| 28 | Fix the ~40 clippy warnings (mostly mechanical: `?`, `if let`, `&[T]` instead of `&Vec<T>`), then make clippy warnings fatal in CI | ⬜ |
| 29 | Move from Rust edition 2018 to 2021 | ⬜ |
| 30 | Remove the duplication in `Callbacks`: `setup` and `player_move` are stored separately from `definitions` | ⬜ |
| 31 | Replace the remaining `.expect()` calls in the lexer and `main.rs` with error handling (none can currently be triggered, but they're fragile) | ⬜ |
| 32 | CLI tests | ✅ `tests/cli.rs` |
| 33 | One list of known work: this file replaces the README todo list | ✅ |

## Milestone 5: towards full Scopa and multi-round games

Not started. These came up while building Simple Scopa, plus the leftovers from the old Milestone 3.

| # | Item | Status | Notes |
|---|---|---|---|
| 34 | Say why a move was rejected | ✅ | e.g. `not allowed: check failed in take: sum(move:target) is sum(move:cards)` |
| 34b | Let a game attach its own message to a `check`, e.g. `check(count(move:target) > 0, "put a / before the cards you're capturing")` | ⬜ | The built-in reason shows the rule, not what the player should do |
| 35 | Loops, e.g. `for each player` | ⬜ | Simple Scopa compares `players:1` and `players:2` by hand, so it only works for 2 players |
| 36 | Variables or game state, e.g. who captured last (in Scopa, they take the cards left at the end) | ⬜ | |
| 37 | Rounds and scores: play several hands, keep points between them | ⬜ | Needed for full Scopa scoring |
| 38 | Compulsory capture (Scopa rule): only allow `drop` when no `take` is possible | ⬜ | Needs a way to ask "would this move work?" without making it |
| 15 | A real `deck` declaration (currently ignored), e.g. a 40-card deck | ⬜ | README "filter deck in declaration". `filter` + `taking` in `setup` works today. |
| 21 | Stacks as real references. Passing a pile to a function copies it, and `filter` results can't be stored. | ⬜ | |
| 22 | Spec tests: `.test(...)` blocks in a game file, run by `cardlang test` | ⬜ | README "spec test"; sketched in `examples/turns.card` |
| 24 | `shuffle(x)` should shuffle `x`, not always `deck` | ⬜ | |
| 25 | Multi-word names and quoted strings | ⬜ | |
| 26 | Switch statements | ⬜ | README; `else if` covers most uses |
| 39 | `*`, `/`, `<=`, `>=` | ⬜ | |

## Milestone 6: CLI experience

Making the interpreter pleasant to play at.

| # | Item | Status | Notes |
|---|---|---|---|
| 40 | **Tab completion**: press Tab to complete `build` file paths (e.g. `build ex<Tab>` → `build examples/`), command names, `show` targets, move names and cards in hand | ⬜ | Needs a line-editing library such as `rustyline` |
| 41 | Command history: up arrow to repeat or edit a previous line | ⬜ | Comes with the same library |
| 42 | Show the table after `start` and after each move | ✅ | Whose turn, deck size, shared piles, current hand, each player's own piles |
| 43 | `show` before `start` says to type `start` | ✅ | It used to print empty piles |
| 44 | Let a game choose what the table shows, e.g. a `display` function | ⬜ | The built-in table is the same for every game |
| 45 | `cardlang play <file>`: build and start in one step | ⬜ | |

