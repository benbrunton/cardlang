# Known limitations

cardlang is early-stage. Everything listed here has been checked against the current code. When one of these is fixed, remove it from this page, update the [language reference](language-reference.md), and mark it done in the [roadmap](ROADMAP.md).

## Reading error messages

Errors give a line number and a short description:

```
error: line 3: unrecognised character (for example '"' or a decimal number)
error: line 2: unexpected word or symbol
```

| Message | Common causes |
|---|---|
| `unrecognised character` | `"`, a decimal number such as `1.5`, or a word starting with a digit |
| `a comment is never closed` | a `.(` comment with no matching `)` |
| `unexpected word or symbol` | a multi-word `name`; a declaration with the wrong kind of value (`players two`, `name 5`); a missing `(` after `check`, `return` or `if`; a chained comparison (`a is b is c`, `a < b < c`); `else` without an `if`; a function named `build`, `start`, `show`, `help` or `exit`; a `values` entry that isn't a rank name and a number; anything left over at the top level of the file |
| `unexpected end of statement` | something is missing: an unclosed `{` or `(`, an unfinished comparison (`check(a is)`), or a transfer with no destination (`deck >`) |
| `expected a name` | `def` not followed by a function name, or something other than names in a function's parameter list |

The message says what the parser found, but not yet what it expected.

## Language

| Limitation | What happens | What to do instead |
|---|---|---|
| No loops | You can't repeat something for each player or card | Address players by number (`players:1:hand`, `players:2:hand`); use `filter`, `count` and `sum` for cards |
| No variables | You can't store a value, such as who captured last | Keep state in piles, or work it out from the cards |
| Only `+` and `-` | No `*`, `/`, `<=` or `>=` | `a < b or a is b` for `<=` |
| `filter` results can't be stored | `filter` returns a new stack that isn't kept anywhere | Use it directly: count it, sum it, or move it with `taking` |
| Stacks passed as arguments are copies | Transferring into a stack parameter doesn't change the original pile | Pass the player, or name the pile directly |
| `deck <word>` declaration | Ignored. Every game uses the full 52-card deck. | Use `filter` and `taking` in `setup` to remove cards |
| `shuffle` argument | Always shuffles `deck` | none |
| Multi-word names | `name my game` fails to parse | `name my_game` |
| Unknown pile names in transfers | Only noticed when the transfer runs: it fails, with a message on stderr | Check spelling against your `stack` declarations |
| `.test(...)` blocks | Skipped; spec tests aren't run yet | none |

## Moves

| Limitation | What happens |
|---|---|
| Rejection reasons show the rule, not advice | e.g. `check failed in take: count(move:cards) is 1` when you forgot the `/`. A game can't give its own message yet. |
| At most two groups of cards | A move has `move:cards` and `move:target`; a third `/` group is rejected |
| Some words can't be move names | Anything that reads as a card (`as`, `seven`, `10`) and the CLI's own commands |

## Interpreter

The interpreter should never crash. If you find a game file or command that makes it panic, that's a bug. Please report it, or add it to the [roadmap](ROADMAP.md).

See [Using the CLI](cli.md#when-a-move-is-ignored) for the messages printed when a move can't be made.
