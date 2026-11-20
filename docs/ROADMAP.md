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

## Milestone 4: code health ✅

| # | Item | Status |
|---|---|---|
| 27 | Fix the compiler warnings (unused `Peekable` import, unused `Game.ast` field) | ✅ |
| 28 | Fix the clippy warnings, then make clippy warnings fatal in CI | ✅ `cargo clippy --all-targets -- -D warnings` |
| 29 | Move from Rust edition 2018 to 2021 | ✅ |
| 30 | Remove the duplication in `Callbacks`: `setup` and `player_move` were stored separately from `definitions` | ✅ |
| 31 | Replace the remaining check-then-`expect()` patterns in the lexer, `main.rs` and transfers | ✅ |
| 32 | CLI tests | ✅ `tests/cli.rs` |
| 33 | One list of known work: this file replaces the README todo list | ✅ |

The Rust version is pinned in `rust-toolchain.toml` (currently 1.100.0), and both CI and local builds use it. This stops a new Rust release from adding clippy warnings that fail the build. To upgrade, change the version there, run `cargo clippy --all-targets -- -D warnings`, and fix anything new (`cargo clippy --fix` handles most of it).

## Milestone 5: messages and loops ✅

Simple Scopa now explains mistakes in plain words and works for 2 to 4 players.

| # | Item | Status | Notes |
|---|---|---|---|
| 34 | Say why a move was rejected | ✅ | e.g. `not allowed: check failed in take: sum(move:target) is sum(move:cards)` |
| 34b | A game's own message on `check`: `check(rule, message)` | ✅ | No quotes; everything after the first comma is the message |
| 35 | Loops: `every player p { … }`, `every card c in <pile> { … }` | ✅ | |
| 35b | "For all" tests: `every player p: <condition>` | ✅ | Lets a game compare players without variables, e.g. who has the most cards |
| 39 | `>=` and `<=` | ✅ | `*` and `/` moved to the backlog |

### Design decisions made

1. `check(rule, message)`: the message is free text with no quotes. Everything after the first top-level comma up to the matching `)` is the message, so it can contain commas. Brackets inside must be balanced.
2. `every` has two forms. With a block it runs once per player or card. With a colon it's a true/false test that holds for all of them. The types are `player` and `card c in <pile>`.
3. Variables, rounds and scoring, compulsory capture, and a game-chosen table display are deferred until a game needs them.

## Milestone 6: CLI experience ✅

Making the interpreter pleasant to play at.

| # | Item | Status | Notes |
|---|---|---|---|
| 40 | **Tab completion**: `build` file paths (`build ex<Tab>` → `build examples/`), commands, `show` targets, move names and the cards you can play | ✅ | Uses `rustyline`, only when running in a terminal |
| 41 | Command history: up arrow to repeat or edit a previous line | ✅ | |
| 42 | Show the table after `start` and after each move | ✅ | Whose turn, deck size, shared piles, current hand, each player's own piles |
| 43 | `show` before `start` says to type `start` | ✅ | It used to print empty piles |
| 45 | `cardlang play <file>`: build and start in one step | ✅ | |
| 44 | Let a game choose what the table shows, e.g. a `display` function | ⚠️ | Needs a design decision; see Milestone 5 |
| 46 | Save history between sessions | ⬜ | History lasts for one session today |

## Milestone 7: web and embedding

Not started. The aim is to use cardlang as the rules engine inside other programs, such as a web app, and not only the CLI. There are two ways to run it:
- **On a server**, which acts as the referee for networked play. Players can't cheat, and each sees only what they're allowed to.
- **In the browser via WebAssembly**, for pass-and-play, offline play, or a playground for writing games. The engine can see every card there, so it's not enough on its own for play between people who don't trust each other.

| # | Item | Status | Notes |
|---|---|---|---|
| 48 | Split into a library crate plus the CLI binary | ⬜ | So a server or WebAssembly build can call the engine directly |
| 49 | Game state as data (e.g. JSON) instead of display text | ⬜ | `show` and the table are text for the terminal |
| 50 | Visibility: who can see which cards, and a view of the game per player | ⬜ | ⚠️ Needs a design decision; probably declared in the game file (face up/down, owner-only hands) |
| 51 | Event log of what happened in a move, e.g. "7h moved from player 1's hand to their collection" | ⬜ | For animation and history |
| 52 | List the legal moves, or check one without making it | ⬜ | For highlighting playable cards; the engine can already try a move and undo it. Also unblocks compulsory capture (38). |
| 53 | Seeded shuffles | ⬜ | Replayable and testable games; server and browser agree on the deck |
| 54 | Save and load a game in progress | ⬜ | Between requests, or across restarts |
| 55 | Limits on how much work one move can do | ⬜ | Needed before running rules uploaded by players. Recursion depth is capped today, but not total work. |
| 56 | Return all errors and messages to the caller instead of printing to stderr | ⬜ | |
| 57 | Proof of concept: a small WebAssembly page to play Simple Scopa by clicking cards | ⬜ | Builds on 48–50. Do this before committing to a server design. |

## Backlog

Deferred or not yet needed. Each needs a design decision before it starts.

| # | Item | Status | Notes |
|---|---|---|---|
| 36 | Variables or game state, e.g. who captured last (in Scopa, they take the cards left at the end) | ⏸️ deferred | |
| 37 | Rounds and scores: play several hands, keep points between them | ⏸️ deferred | Needed for full Scopa scoring; builds on 36 |
| 38 | Compulsory capture (Scopa rule): only allow `drop` when no `take` is possible | ⏸️ deferred | Needs a way to ask "would this move work?" without making it |
| 44 | Let a game choose what the table shows, e.g. a `display` function | ⏸️ deferred | |
| 47 | "Any" tests and `not` for a whole condition | ⬜ | Today: `(every player p: …) is false` |
| 15 | A real `deck` declaration (currently ignored), e.g. a 40-card deck | ⬜ | README "filter deck in declaration". `filter` + `taking` in `setup` works today. |
| 21 | Stacks as real references. Passing a pile to a function copies it, and `filter` results can't be stored. | ⬜ | |
| 22 | Spec tests: `.test(...)` blocks in a game file, run by `cardlang test` | ⏸️ deferred | README "spec test"; sketched in `examples/turns.card` |
| 24 | `shuffle(x)` should shuffle `x`, not always `deck` | ⬜ | |
| 25 | Multi-word names | ⬜ | Could reuse the free-text approach from check messages |
| 26 | Switch statements | ⬜ | README; `else if` covers most uses |
| 39b | `*` and `/` | ⬜ | |
