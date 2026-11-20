# Language reference

A cardlang file (`.card`) contains **declarations**, which set up the table, and **definitions**, which are functions holding the rules. Statements go inside function bodies.

- [Source text](#source-text)
- [Declarations](#declarations)
- [Functions](#functions)
- [Statements](#statements)
- [Transfers](#transfers)
- [Expressions and values](#expressions-and-values)
- [Built-in functions](#built-in-functions)
- [Objects: players and cards](#objects-players-and-cards)

## Source text

| Element | Rules |
|---|---|
| Comments | `.( any text )`. They can span lines, and nested parentheses are allowed: `.(a (b) c)`. |
| Words (symbols) | Start with a letter. Can contain letters, digits, `_` and `:`. Case-sensitive. |
| Numbers | Whole numbers only, such as `2` or `26`. |
| Whitespace | Spaces, tabs and newlines. Windows (`\r\n`) line endings are fine. |
| Keywords | `name`, `deck`, `players`, `current_player`, `stack`, `define`, `if`, `else`, `check`, `return`, `is`, `not`, `or`, `true`, `false` |

Write one statement per line.

## Declarations

Declarations go at the top level of the file, outside any function.

| Declaration | Meaning | Default |
|---|---|---|
| `name <word>` | The game's name, shown by `show name`. Must be a single word. | none |
| `players <number>` | Number of players. They are numbered `1` to `n` and start with empty hands. | `1` |
| `current_player <number>` | Which player's turn it is at the start. | `1` |
| `stack <word>` | Creates an empty pile with this name. Repeat it for more piles. | none |
| `deck <word>` | Accepted but currently **ignored**. | none |

Giving a declaration the wrong kind of value, such as `players two` or `name 5`, is a parse error.

Every game has a standard 52-card deck called `deck`. It starts unshuffled, in order: ace to king of spades, then hearts, then clubs, then diamonds. The **top** of the deck is the last card, the king of diamonds.

## Functions

```
define name(arg1, arg2){
    statements…
}
```

Parameters can be separated by commas or spaces. A body can be on one line: `define setup(){ next_player() }`.

Two names are special. The interpreter calls them for you:

| Function | Called when | Argument |
|---|---|---|
| `setup` | `start` is run | none |
| `player_move` | `move <n>` is run while the game is active | the [player object](#objects-players-and-cards) for player `n` |

Any other function runs only when you call it: `name(a, b)` as a statement, or inside an expression (`count(hand_of(player))`).

How calls work:

- **Arguments** are worked out in the calling function, then bound to the parameter names. Passing a player object passes the player itself. A transfer such as `p:hand > middle` inside the function changes that player's real hand.
- **Missing arguments** are `false`. **Extra arguments** are ignored.
- **Scope:** a function sees its own parameters, plus anything visible in the functions that called it. If two have the same name, the innermost one wins.
- **Return value:** the value from `return(...)`, or `false` if the function ends without returning or a `check` fails.
- **Recursion** is allowed, but stops at a depth of 256 calls (a message is printed to stderr).
- Calling an **unknown function** prints `unknown function '<name>'` to stderr and gives `false`.
- If your function has the same name as a built-in, the built-in is used.

## Statements

| Statement | Meaning |
|---|---|
| `from > to` / `from > to end` | Move cards. See [Transfers](#transfers). |
| `name(args…)` | Call a function and discard its result. |
| `if(condition){ … }` | Run the block when the condition is true. |
| `if(condition){ … } else { … }` | Run the first block when the condition is true, otherwise the second. `else` can go on the line after the `}`. |
| `if(a){ … } else if(b){ … } else { … }` | Run the block for the first condition that is true. |
| `check(condition)` | When the condition is false, stop the function and return `false`. This works inside `if` blocks too. |
| `return(expression)` | Stop the function and give back the value. This works inside `if` blocks too. |

A **condition** (in `if` and `check`) is true only when it evaluates to the boolean `true`. A function call or parameter can be used directly: `if(is_out(player))`. A number, text or stack is never true, so `if(count(deck))` is false. Write `if(count(deck) is not 0)`.

A failing `check` in `player_move` stops the move. The CLI doesn't print anything when that happens.

## Transfers

```
from > to
from > to end
```

A transfer takes cards one at a time from the top of `from` and puts them on top of `to`.

| Form | Cards moved |
|---|---|
| `a > b` | one card |
| `a > players` | one card **to each player** (players 1, 2, … in turn) |
| `a > b end` | every card in `a` |
| `a > players end` | every card in `a`, dealt round-robin starting with player 1 |

Valid names for `from` or `to`:

- `deck`
- a pile declared with `stack <name>`
- `p:hand`, where `p` is a player object, such as the `player` argument of `player_move`
- `players` (as a destination only)

Moving from an empty pile does nothing. If a name isn't a pile, the transfer is skipped and `transfer ignored: unknown stack '<name>'` is printed to stderr. Moving a set number of cards (`deck > middle 4`) is not supported yet and is a parse error.

## Expressions and values

Values are numbers, booleans (`true` / `false`), text, and stacks (lists of cards).

| Expression | Value |
|---|---|
| `26` | number |
| `true`, `false` | boolean |
| `current_player` | number of the player whose turn it is |
| `deck` / a pile name | that stack of cards |
| a parameter name | the value passed in |
| `p:id`, `p:hand`, `c:rank`… | a field of an [object](#objects-players-and-cards) |
| any other word, e.g. `Ace` | the word as text |
| `f(a, b)` | the function's return value |
| `a is b` | `true` if the values are equal |
| `a is not b` | `true` if the values differ |
| `a & b` | `true` if both are `true` |
| `a or b` | `true` if either is `true` |
| `( … )` | grouping |

**Precedence**, tightest first: `is` / `is not`, then `&`, then `or`. So `a is 1 & b is 2 or c is 3` means `((a is 1) & (b is 2)) or (c is 3)`. Use parentheses to group differently: `a is 1 & (b is 2 or c is 3)`. A comparison can't be chained: `a is b is c` is a parse error.

Equality compares type as well as value: the number `1` is not equal to the text `1`. Two stacks are equal when they hold the same cards in the same order.

## Built-in functions

| Function | Does | Returns |
|---|---|---|
| `shuffle(deck)` | Shuffles the main deck. It always shuffles `deck`, whatever you pass. | nothing |
| `count(stack)` | Counts cards. Gives `0` for anything that isn't a stack, or when called with no argument. | number |
| `filter(stack, f)` | Calls your function `f` with each card. Keeps the cards where `f` returns `true`. The original stack is not changed. | stack |
| `winner(id)` | Adds a player to the winners list. Call it more than once for several winners. Without a number it prints an error and does nothing. | nothing |
| `end()` | Ends the game. Later moves are ignored. | nothing |
| `next_player()` | Moves `current_player` to the next player, wrapping from the last back to 1. | nothing |

`filter` example:

```
define is_heart(card){
    return(card:suit is Hearts)
}

define setup(){
    if(count(filter(deck, is_heart)) is 13){
        end()
    }
}
```

## Objects: players and cards

**Player**: the argument passed to `player_move`.

| Field | Value |
|---|---|
| `p:id` | player number (1, 2, …) |
| `p:hand` | the player's current hand, as a stack |

**Card**: the argument passed to a `filter` function.

| Field | Values (text, case-sensitive) |
|---|---|
| `c:rank` | `Ace`, `Two`, `Three`, `Four`, `Five`, `Six`, `Seven`, `Eight`, `Nine`, `Ten`, `Jack`, `Queen`, `King` |
| `c:suit` | `Spades`, `Hearts`, `Clubs`, `Diamonds` |
