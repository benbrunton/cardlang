# Known limitations

cardlang is early-stage. Everything listed here has been checked against the current code. When one of these is fixed, remove it from this page and update the [language reference](language-reference.md).

## Reading error messages

Errors are printed in their raw form:

```
parse error: LexError { error_type: ParseError, line_number: 33 }
parse error: ParseError { error_type: UnexpectedToken, line_number: 2 }
```

- **`LexError`**: the file contains a character cardlang doesn't understand. Common causes: a tab, a Windows (`\r\n`) line ending, `-`, `"`, or a decimal number.
- **`ParseError`**: the words are valid but in an order the parser doesn't expect. Common causes: a multi-word `name`, a declaration with the wrong kind of value (`players two`, `name 5`), a missing `(` after `check`, `return` or `if`, an unclosed `{`, an unfinished comparison (`check(a is)`), a transfer with no destination (`deck >`), or a trailing comma in a call.
- The line number is usually right, but can point slightly after the real problem. Some errors report line `0`.

## Language

| Limitation | What happens | What to do instead |
|---|---|---|
| A bare value or function call as a condition | `if(ready(p))` is always false | `if(ready(p) is true)` |
| `&` | Binds wrongly: `a is 1 & b is 2` is parsed as `a is (1 & (b is 2))`. `true & f()` is always false. Only `true`/`false` on the left with a single comparison on the right works (`true & a is 1`). | Use a helper function with one `check` per condition (example below) |
| `check` or `return` inside an `if` block | Only ends the `if` block. The function carries on, and the returned value is lost. | Keep `check` and `return` at the top level of the function body |
| No `else`, `or`, or arithmetic | Not supported | Two `if`s with opposite conditions (`is` / `is not`) |
| Transfer counts | `deck > middle 4` moves one card; the number is ignored | Repeat the transfer, or use `end` |
| `deck <word>` declaration | Ignored. Every game uses the full 52-card deck. | Use `filter` in your logic |
| `filter` results can't be moved | `filter` returns a new stack that isn't stored anywhere | Use it for counting or comparing |
| Stacks passed as arguments are copies | Transferring into a stack parameter doesn't change the original pile | Pass the player, or name the pile directly |
| `shuffle` argument | Always shuffles `deck` | none |
| Multi-word names | `name my game` fails to parse | `name my_game` |
| Unknown pile names in transfers | Silently ignored | Check spelling against your `stack` declarations |

### Combining conditions

Until `&` is fixed, use a helper function with a `check` per condition:

```
define both(a, b){
    check(a is 1)
    check(b is 2)
    return(true)
}

define setup(){
    if(both(x, y) is true){
        end()
    }
}
```

This works because a failing `check` makes the function return `false`. Keep the `check`s at the top level of the helper, not inside an `if`.

## Syntax used in `examples/simple_scopa.card` that doesn't exist yet

- `def` (use `define`)
- removing specific cards in a transfer: `a > b - move:cards`
- `valid_moves(...)`, `cards_in_stack(...)`
- `deck filter(StandardDeck, not_royal)` as a declaration
- a move argument for `player_move` (the CLI only passes the player)

## Interpreter

The interpreter should never crash. If you find a game file or command that makes it panic, that's a bug. Please report it, or add it to the [roadmap](ROADMAP.md).

See [Using the CLI](cli.md#when-a-move-is-ignored) for the messages printed when a move can't be made.
