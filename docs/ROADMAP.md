# Roadmap

All known bugs and planned work in one place, in priority order. It replaces the old todo list in the README.

**Keeping it current:** when you fix something, mark it ✅ here, then update [limitations.md](limitations.md) and the [language reference](language-reference.md) in the same PR.

Status: ✅ done · 🟡 partly done · ⬜ not started · ⚠️ needs a design decision first · ➡️ moved

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
| 58 | Syntax highlighting for VS Code (`editors/vscode`) | ✅ | Tested with the same engine as VS Code; also covers syntax proposed for Scopa. Getting GitHub itself to highlight `.card` files needs the language added to GitHub's Linguist, which only accepts widely used languages. |

## Milestone 7: web and embedding

Not started. The aim is to use cardlang as the rules engine inside other programs, such as a web app, and not only the CLI. There are two ways to run it:
- **On a server**, which acts as the referee for networked play. Players can't cheat, and each sees only what they're allowed to.
- **In the browser via WebAssembly**, for pass-and-play, offline play, or a playground for writing games. The engine can see every card there, so it's not enough on its own for play between people who don't trust each other.

| # | Item | Status | Notes |
|---|---|---|---|
| 48 | Split into a library crate plus the CLI binary | ➡️ | Moved to Milestone 9 |
| 49 | Game state as data (e.g. JSON) instead of display text | ⬜ | `show` and the table are text for the terminal |
| 50 | Visibility: who can see which cards, and a view of the game per player | ⬜ | ⚠️ Needs a design decision; probably declared in the game file (face up/down, owner-only hands) |
| 51 | Event log of what happened in a move, e.g. "7h moved from player 1's hand to their collection" | ⬜ | For animation and history |
| 52 | List the legal moves, or check one without making it | ⬜ | For highlighting playable cards; the engine can already try a move and undo it. Also unblocks compulsory capture (38). |
| 53 | Seeded shuffles | ➡️ | Moved to Milestone 9 |
| 54 | Save and load a game in progress | ⬜ | Between requests, or across restarts |
| 55 | Limits on how much work one move can do | ➡️ | Moved to Milestone 9 |
| 56 | Return all errors and messages to the caller instead of printing to stderr | ➡️ | Moved to Milestone 9 |
| 57 | Proof of concept: a small WebAssembly page to play Simple Scopa by clicking cards | ⬜ | Builds on 48–50. Do this before committing to a server design. |

## Milestone 8: full Scopa ✅

[`examples/scopa.card`](../examples/scopa.card) plays the full game to 11 points: a 40-card deck, single-match priority, compulsory capture, scopas, the last capturer taking the rest, and scoring for most cards, most diamonds, the settebello and the primiera. Its header is a plain-English rulebook. Tests play whole games and check scoring with known collections.

| # | Item | Status | Notes |
|---|---|---|---|
| 15 | `deck standard without Eight Nine Ten` | ✅ | |
| 36 | Numbers: `number dealer`, `player number score`, assigned with `=` | ✅ | Declarations by type: `stack` / `number`, and `player stack` / `player number` for every player. `player stack` replaces `stack … per player`. |
| 37 | Hands within a game: `gather()` puts every card back in the deck | ✅ | The game decides when a hand ends and the next starts |
| 38 | Compulsory capture | ✅ | Written in the game with `any` and `group` |
| 47 | `any` | ✅ | `not` for a whole condition is still missing |
| 59 | `group g in <pile>`: every combination of cards | ✅ | Piles over 20 cards are refused |
| 60 | Card literals: `[7d, kh]` | ✅ | |
| 61 | `most player p: …`: the leader's id, or 0 for a tie | ✅ | Instead of passing function names around |
| 62 | Named value tables: `values primiera { … }` → `card:primiera` | ✅ | |
| 63 | A line break inside `( … )` continues the statement | ✅ | |
| 64 | The table grouped by player, with their numbers and piles, and cards as codes | ✅ | |
| 66 | Moves declared with `move` instead of `player_move` + `valid_moves` | ✅ | `move take(player, played, captured)`: the card groups are named parameters. The engine checks it's the player's turn, runs the named move (or tries each in file order), calls `after_move()`, and passes the turn on unless the move set `current_player`. |
| 67 | `npm run install-extension` builds and installs the VS Code extension | ✅ | |

### Design decisions made

1. Declarations by type: `stack` and `number` are shared by the whole game; `player stack` and `player number` give each player their own.
2. `most player p: …` gives the leader's id, or 0 for a tie, instead of passing function names around.
3. Moves are declared with `move`, with the card groups as named parameters. Turn order is enforced by the engine, with no opt-out for now: games where anyone can act at any time (like Snap) will need a design decision when they come up.
4. `after_move()` is the one hook for what happens after a move.

### Plan from here

1. ~~Specify Scopa~~ ✅
2. ~~Draft the next games to find what the language is missing~~ ✅ [Klondike, Hold'em, Gin Rummy, Pusoy Dos and Pusoy Dos with Pickering rules](../examples/drafts/PROPOSALS.md)
3. Groundwork: the parts of Milestone 7 that don't depend on the language (Milestone 9)
4. Language 1.0 (Milestone 10)
5. The rest of web and embedding (Milestone 7)
6. Build a lightweight client

The language comes before the rest of Milestone 7. Game state as data (49), visibility (50), the event log (51), listing legal moves (52) and saving games (54) all depend on what a game's state looks like, and the proposals change it: face up and face down, numbered piles, `none`, `flag` and `card`, players as numbers. Face up and face down *is* the visibility model. Building those first would mean building them twice. The groundwork in Milestone 9 isn't affected, and it makes the language work quicker: seeded shuffles give the new games repeatable tests, and returned errors let tests check messages.

## Milestone 9: groundwork for 1.0 and embedding

Taken from Milestone 7 because they don't depend on the language.

| # | Item | Status | Notes |
|---|---|---|---|
| 48 | Split into a library crate plus the CLI binary | ⬜ | Lets the drafts run as tests against the engine directly |
| 53 | Seeded shuffles | ⬜ | Today the Scopa test removes `shuffle(deck)` from the source to get a known deal |
| 56 | Return all errors and messages to the caller instead of printing to stderr | ⬜ | So tests can check them |
| 55 | Limits on how much work one move can do | ⬜ | The new quantifiers make runaway work easier to write |

## Milestone 10: language 1.0

**1.0 is done when** [Klondike](../examples/drafts/klondike.card), [Texas Hold'em](../examples/drafts/holdem.card), [Gin Rummy](../examples/drafts/gin_rummy.card) [Pusoy Dos](../examples/drafts/pusoy_dos.card) and [Pusoy Dos with Pickering rules](../examples/drafts/pusoy_dos_pickering.card), written as changes to Pusoy Dos, play from start to finish, each with tests that play whole games, and Scopa still passes. After 1.0, changes that break existing games need a very good reason.

The work is the proposals in [PROPOSALS.md](../examples/drafts/PROPOSALS.md). Suggested order: renames first, while there are few games to update; then the changes to values; then face up and face down.

| # | Proposal | Status | Games |
|---|---|---|---|
| P16 | `must` replaces `check` | ⬜ | all |
| P12 | `not` (was 65) and `and` for `&` | ⬜ | Klondike, Hold'em, Gin, Pickering |
| P14 | `*` and `/` (was 39b) | ⬜ | Hold'em, both Pusoy Dos |
| P17 | `card:colour` | ⬜ | Klondike |
| P18 | Suits in value tables | ⬜ | Pusoy Dos |
| P19 | Variants: `extends` and `base()` | ⬜ | Pickering |
| P20 | Jokers and wildcards (`with jokers`, `version`) | ⬜ | Pickering |
| P21 | A range of players (`players 2 to 6`, `start 3`, `count player`) | ⬜ | both Pusoy Dos. Also changes how the CLI and an embedding program start a game |
| P1 | `let` | ⬜ | all |
| P2 | `none` | ⬜ | Klondike, Gin |
| P10 | Players are their numbers | ⬜ | Hold'em, Gin |
| P11 | `flag` and `card` declarations | ⬜ | Hold'em, Gin, Pickering |
| P5 | `:top`, `:pile` | ⬜ | Klondike, Gin, Pusoy Dos |
| P7 | `in` (replaces `cards_in_stack`) | ⬜ | Klondike, Gin, Pusoy Dos |
| P8 | Pile `+` and `-` | ⬜ | Hold'em, Gin |
| P9 | Quantifiers: `count`, `sum`, `min`, `max`, `first`, `next` | ⬜ | all |
| P4 | Numbered piles | ⬜ | Klondike, Gin. May need stacks as real references (21) |
| P6 | `from`, and counts as expressions | ⬜ | Klondike |
| P3 | Face up and face down | ⬜ | all. Designed together with visibility (50) |
| P13 | Number parameters on moves | ⬜ | Hold'em. Affects listing legal moves (52) |
| P15 | `use poker` | ⬜ | Hold'em |

The riskiest are `use poker` and number parameters on moves, and only Hold'em needs them. If they hold things up, Hold'em can move to 1.1 rather than delaying the rest.

## Backlog

Deferred or not yet needed. Each needs a design decision before it starts.

| # | Item | Status | Notes |
|---|---|---|---|
| 44 | Let a game choose what the table shows, e.g. a `display` function | ⏸️ deferred | |
| 21 | Stacks as real references. Passing a pile to a function copies it, and `filter` results can't be stored. | ⬜ | |
| 22 | Spec tests: `.test(...)` blocks in a game file, run by `cardlang test` | ⏸️ deferred | README "spec test"; sketched in `examples/turns.card` |
| 24 | `shuffle(x)` should shuffle `x`, not always `deck` | ⬜ | |
| 25 | Multi-word names | ⬜ | Could reuse the free-text approach from check messages |
| 26 | Switch statements | ⬜ | README; `else if` covers most uses |
| 39b | `*` and `/` | ⬜ | |
| 65 | `not` for a whole condition | ⬜ | Today: `(…) is false` |
