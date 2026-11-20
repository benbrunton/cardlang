# Known limitations

cardlang is early-stage. Everything listed here has been checked against the current code. When one of these is fixed, remove it from this page, update the [language reference](language-reference.md), and mark it done in the [roadmap](ROADMAP.md).

## Reading error messages

Errors give a line number and a short description:

```
error: line 33: unrecognised character (for example '-', '"' or a decimal number)
error: line 2: unexpected word or symbol
```

| Message | Common causes |
|---|---|
| `unrecognised character` | `-`, `"`, a decimal number such as `1.5`, or a word starting with a digit |
| `a comment is never closed` | a `.(` comment with no matching `)` |
| `unexpected word or symbol` | a multi-word `name`; a declaration with the wrong kind of value (`players two`, `name 5`); a missing `(` after `check`, `return` or `if`; a chained comparison (`a is b is c`); `else` without an `if`; a number after a transfer (`deck > middle 4`); anything left over at the top level of the file |
| `unexpected end of statement` | something is missing: an unclosed `{` or `(`, an unfinished comparison (`check(a is)`), or a transfer with no destination (`deck >`) |
| `expected a name` | `define` not followed by a function name, or something other than names in a function's parameter list |

The message says what the parser found, but not yet what it expected.

## Language

| Limitation | What happens | What to do instead |
|---|---|---|
| No arithmetic | `+`, `-` etc. aren't supported | none yet; see the [roadmap](ROADMAP.md) (item 20) |
| Transfer counts | `deck > middle 4` is a parse error | Repeat the transfer, or use `end` |
| `deck <word>` declaration | Ignored. Every game uses the full 52-card deck. | Use `filter` in your logic |
| `filter` results can't be moved | `filter` returns a new stack that isn't stored anywhere | Use it for counting or comparing |
| Stacks passed as arguments are copies | Transferring into a stack parameter doesn't change the original pile | Pass the player, or name the pile directly |
| `shuffle` argument | Always shuffles `deck` | none |
| Multi-word names | `name my game` fails to parse | `name my_game` |
| Unknown pile names in transfers | Only noticed when the transfer runs. It's skipped, with a message on stderr. | Check spelling against your `stack` declarations |
| `.test(...)` blocks | Skipped; spec tests aren't run yet | none |

## Syntax used in `examples/simple_scopa.card` that doesn't exist yet

- `def` (use `define`)
- removing specific cards in a transfer: `a > b - move:cards`
- `valid_moves(...)`, `cards_in_stack(...)`
- `deck filter(StandardDeck, not_royal)` as a declaration
- a move argument for `player_move` (the CLI only passes the player)

## Interpreter

The interpreter should never crash. If you find a game file or command that makes it panic, that's a bug. Please report it, or add it to the [roadmap](ROADMAP.md).

See [Using the CLI](cli.md#when-a-move-is-ignored) for the messages printed when a move can't be made.
