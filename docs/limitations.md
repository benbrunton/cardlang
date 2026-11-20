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
| `a list of cards is never closed` | a `[` card list with no `]` on the same line |
| `a check message is never closed` | a `check(rule, message` with no matching `)`, or an unbalanced `(` in the message |
| `unexpected word or symbol` | a deck other than `standard`, or `without` followed by something that isn't a rank; the old `stack <name> per player` (now `player stack <name>`); `player stack hand`; `most` with anything but `player`; `every`/`any` without `player`, `card … in` or `group … in`, a block with a colon (`every player p: { }`) or a test without one; a multi-word `name`; a declaration with the wrong kind of value (`players two`, `name 5`); a missing `(` after `check`, `return` or `if`; a chained comparison (`a is b is c`, `a < b < c`); `else` without an `if`; a function named `build`, `start`, `show`, `help` or `exit`; a `values` entry that isn't a rank name and a number; anything left over at the top level of the file |
| `unexpected end of statement` | something is missing: an unclosed `{` or `(`, an unfinished comparison (`check(a is)`), or a transfer with no destination (`deck >`) |
| `expected a name` | `def` not followed by a function name, or something other than names in a function's parameter list |

The message says what the parser found, but not yet what it expected.

## Language

| Limitation | What happens | What to do instead |
|---|---|---|
| Only `+` and `-` | No `*` or `/` | none |
| No `not` for a whole condition | You can't write `not (a & b)` | `(a & b) is false` |
| Numbers only | `number` holds whole numbers; there are no text or true/false variables | Use 0 and 1 |
| `group` needs small piles | A pile of more than 20 cards is refused | Filter the pile first |
| `filter` results can't be stored | `filter` returns a new stack that isn't kept anywhere | Use it directly: count it, sum it, or move it with `taking` |
| Stacks passed as arguments are copies | Transferring into a stack parameter doesn't change the original pile | Pass the player, or name the pile directly |
| `shuffle` argument | Always shuffles `deck` | none |
| Multi-word names | `name my game` fails to parse | `name my_game` |
| Unknown pile names in transfers | Only noticed when the transfer runs: it fails, with a message on stderr | Check spelling against your `stack` declarations |
| `.test(...)` blocks | Skipped; spec tests aren't run yet | none |

## Moves

| Limitation | What happens |
|---|---|
| Checks without a message show the rule | e.g. `check failed in take: count(move:cards) is 1`. Add a message: `check(count(move:cards) is 1, play one card)`. |
| At most two groups of cards | A move has `move:cards` and `move:target`; a third `/` group is rejected |
| Some words can't be move names | Anything that reads as a card (`as`, `seven`, `10`) and the CLI's own commands |

## Interpreter

The interpreter should never crash. If you find a game file or command that makes it panic, that's a bug. Please report it, or add it to the [roadmap](ROADMAP.md).

See [Using the CLI](cli.md#when-a-move-is-ignored) for the messages printed when a move can't be made.
