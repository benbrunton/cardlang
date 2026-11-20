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

## Milestone 2: correct behaviour

Bugs that give game authors wrong results without any error. These matter most for the language being trustworthy.

| # | Item | Status | Notes |
|---|---|---|---|
| 6 | `check` and `return` inside an `if` only end the `if` block. The function carries on, and the returned value is lost. | ⬜ | Make `handle_statements` return a control-flow result (continue / return value) and pass it up through `if` |
| 7 | `&` binds wrongly: `a is 1 & b is 2` is parsed as `a is (1 & (b is 2))`. `true & f()` is always false. | ⬜ | Give comparisons higher precedence than `&` in the expression parser. Add `or`, and `else` for `if`, at the same time. |
| 8 | A bare boolean value or function call used as a condition (`if(f(x))`) is always false | ⬜ | `resolve_to_bool` should evaluate the expression and use the result when it's a boolean |
| 9 | Invalid input is accepted | 🟡 | `players two` and `name 5` are now rejected. A stray `}`, `)` or number at the top level is still silently ignored. |
| 10 | Unknown pile names in transfers are silently ignored | ⬜ | Report at runtime, or better, check at parse time against the `stack` declarations |
| 11 | Error line numbers are wrong after multi-line comments (off by the comment's line count). Some errors report line `0`. Newline tokens carry the next line's number. | ⬜ | Count newlines inside comments in the lexer |
| 12 | Tabs and `\r` (Windows line endings) are lex errors | ⬜ | Treat them as whitespace |
| 13 | Error messages are raw Rust debug output (`ParseError { error_type: … }`) | ⬜ | Implement `Display`: `line 5: expected a number after 'players'` |
| 13b | `winner(x)` where `x` isn't a number records player `0` | ⬜ | Report an error instead |

## Milestone 3: features to play Scopa

The old README todo list, plus what [`examples/simple_scopa.card`](../examples/simple_scopa.card) relies on, ordered by dependency. Settle the design decisions below before starting the ⚠️ items.

| # | Item | Status | Source |
|---|---|---|---|
| 14 | Transfer counts: `deck > middle 4` (the number is currently ignored) | ⬜ | README "limit on stack transfer" |
| 15 | A real `deck` declaration (currently ignored), including `deck filter(StandardDeck, not_royal)` | ⬜ | README "filter deck in declaration" |
| 16 | Move input: `player_move(player, move)`, plus a CLI way to choose cards | ⚠️ | Scopa |
| 17 | Moving specific cards in a transfer: `a > b - move:cards` | ⚠️ | README "specify cards on transfer" |
| 18 | `cards_in_stack(cards, stack)` built-in | ⬜ | README |
| 19 | `valid_moves(take, drop)`: try each move function in turn | ⬜ | Scopa |
| 20 | Card values and arithmetic (Scopa's `get_value` needs sums) | ⚠️ | Scopa |
| 21 | Stacks as real references. Passing a pile to a function copies it, and `filter` results can't be stored. | ⬜ | limitations.md |
| 22 | Spec tests: `.test(...)` blocks in a game file, run by `cardlang test` | ⬜ | README "spec test"; sketched in `examples/turns.card` |
| 23 | `def` as an alias for `define`, or update the Scopa example | ⚠️ | Scopa |
| 24 | `shuffle(x)` should shuffle `x`, not always `deck` | ⬜ | limitations.md |
| 25 | Multi-word names and quoted strings | ⬜ | limitations.md |
| 26 | Switch statements | ⬜ | README |

### Design decisions needed

1. **`def` or `define`?** Make `def` an alias, or update the Scopa example? (item 23)
2. **How does a player choose cards on a move?** The CLI syntax (e.g. `move 1 ace hearts, two clubs`) and the shape of the `move` object passed to `player_move`. (item 16)
3. **Syntax for moving specific cards:** keep `a > b - cards`, or something else? (item 17)
4. **Card values:** a fixed value table in the game declaration, or a function the game defines? (item 20)

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
