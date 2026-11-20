# Language reference

A cardlang file (`.card`) contains **declarations**, which set up the table, and **definitions**, which are functions holding the rules. Statements go inside function bodies.

- [Source text](#source-text)
- [Declarations](#declarations)
- [Functions](#functions)
- [Moves](#moves)
- [Statements](#statements)
- [Transfers](#transfers)
- [Expressions and values](#expressions-and-values)
- [Built-in functions](#built-in-functions)
- [Objects: players, cards and moves](#objects-players-cards-and-moves)

## Source text

| Element | Rules |
|---|---|
| Comments | `.( any text )`. They can span lines, and nested parentheses are allowed: `.(a (b) c)`. |
| Words (symbols) | Start with a letter. Can contain letters, digits, `_` and `:`. Case-sensitive. |
| Numbers | Whole numbers only, such as `2` or `26`. Write negative numbers as `-1`. |
| Operators | `>` (transfer, or greater than inside an expression), `<`, `+`, `-`, `&` |
| Whitespace | Spaces, tabs and newlines. Windows (`\r\n`) line endings are fine. |
| Keywords | `name`, `deck`, `players`, `current_player`, `stack`, `values`, `def`, `if`, `else`, `check`, `return`, `is`, `not`, `or`, `true`, `false` |

Write one statement per line.

## Declarations

Declarations go at the top level of the file, outside any function.

| Declaration | Meaning | Default |
|---|---|---|
| `name <word>` | The game's name, shown by `show name`. Must be a single word. | none |
| `players <number>` | Number of players. They are numbered `1` to `n` and start with empty hands. | `1` |
| `current_player <number>` | Which player's turn it is at the start. | `1` |
| `stack <word>` | Creates an empty shared pile with this name. Repeat it for more piles. | none |
| `stack <word> per player` | Gives **every player** their own empty pile with this name, such as `stack collection per player`. It's used as `p:collection`. (`hand` already exists.) | none |
| `values { … }` | Changes card values; see [card values](#card-values). | Ace 1 … King 13 |
| `deck <word>` | Accepted but currently **ignored**. | none |

Giving a declaration the wrong kind of value, such as `players two` or `name 5`, is a parse error.

Every game has a standard 52-card deck called `deck`. It starts unshuffled, in order: ace to king of spades, then hearts, then clubs, then diamonds. The **top** of the deck is the last card, the king of diamonds.

### Card values

Each card has a value: Ace 1, Two 2 … Ten 10, Jack 11, Queen 12, King 13. A `values` table changes the value of any rank:

```
values {
    Jack 8, Queen 9, King 10
}
```

Entries are a rank name (as in `card:rank`) and a whole number. Commas and new lines between entries are both fine. Ranks you don't list keep their default value. Values are used by `card:value` and `sum(cards)`.

## Functions

```
def name(arg1, arg2){
    statements…
}
```

Parameters can be separated by commas or spaces. A body can be on one line: `def setup(){ next_player() }`. The CLI's command words (`build`, `start`, `show`, `help`, `exit`) can't be used as function names.

Two names are special. The interpreter calls them for you:

| Function | Called when | Arguments |
|---|---|---|
| `setup` | `start` is run | none |
| `player_move` | a player makes a move at the CLI (see [Moves](#moves)) | the [player](#objects-players-cards-and-moves) making the move, then the [move](#objects-players-cards-and-moves) |

Any other function runs only when you call it: `name(a, b)` as a statement, or inside an expression (`count(hand_of(player))`).

How calls work:

- **Arguments** are worked out in the calling function, then bound to the parameter names. Passing a player object passes the player itself. A transfer such as `p:hand > middle` inside the function changes that player's real hand.
- **Missing arguments** are `false`. **Extra arguments** are ignored.
- **Scope:** a function sees its own parameters, plus anything visible in the functions that called it. If two have the same name, the innermost one wins.
- **Return value:** the value from `return(...)`. A function that ends without returning, or that a `check` or transfer stops, gives `false`.
- **Recursion** is allowed, but stops at a depth of 256 calls (a message is printed to stderr).
- Calling an **unknown function** prints `unknown function '<name>'` to stderr and gives `false`.
- If your function has the same name as a built-in, the built-in is used.

## Moves

When a player types a move at the CLI, such as `take 9d / 4c 5s` ([syntax](cli.md#moves)), the interpreter calls `player_move(player, move)`:

```
def player_move(player, move){
    check(player:id is current_player)
    valid_moves(take, drop)
    next_player()
}

def take(player, move){
    check(sum(move:target) is sum(move:cards))
    middle > player:collection taking move:target
    player:hand > player:collection taking move:cards
}

def drop(player, move){
    player:hand > middle taking move:cards
}
```

- **`valid_moves(take, drop)`** runs the move the player named. For example, `take 9d / 4c` runs `take`; a name that isn't in the list is rejected. If the player didn't give a name (`9d / 4c`), it tries each move in order and uses the first one that succeeds. Each move function receives the same `player` and `move` arguments.
- **A move succeeds unless it fails.** It fails when a `check` fails, a transfer can't happen, it returns `false`, or `valid_moves` finds no valid move.
- **A failed move changes nothing.** Every card and value goes back to how it was before the move, including transfers made before the failure. The CLI prints `move ignored: not allowed`.

`player_move` can also take just the player (`def player_move(player)`), for games where a move needs no input, like [Race](../examples/race.card).

## Statements

| Statement | Meaning |
|---|---|
| `from > to …` | Move cards. See [Transfers](#transfers). If the transfer can't happen, the function stops and returns `false`, like a failed `check`. |
| `name(args…)` | Call a function and discard its result. `valid_moves(...)` is the exception: if no move is valid, the function stops and returns `false`. |
| `if(condition){ … }` | Run the block when the condition is true. |
| `if(condition){ … } else { … }` | Run the first block when the condition is true, otherwise the second. `else` can go on the line after the `}`. |
| `if(a){ … } else if(b){ … } else { … }` | Run the block for the first condition that is true. |
| `check(condition)` | When the condition is false, stop the function and return `false`. This works inside `if` blocks too. |
| `return(expression)` | Stop the function and give back the value. This works inside `if` blocks too. |

A **condition** (in `if` and `check`) is true only when it evaluates to the boolean `true`. A function call or parameter can be used directly: `if(is_out(player))`. A number, text or stack is never true, so `if(count(deck))` is false. Write `if(count(deck) > 0)`.

## Transfers

```
from > to
from > to <number>
from > to end
from > to taking <cards>
```

A transfer takes cards from the top of `from` and puts them on top of `to`.

| Form | Cards moved |
|---|---|
| `a > b` | one card |
| `a > b 4` | four cards (or as many as `a` has) |
| `a > b end` | every card in `a` |
| `a > players` | one card **to each player** (players 1, 2, … in turn) |
| `a > players 3` | three cards to each player, dealt one at a time round the table |
| `a > players end` | every card in `a`, dealt round the table starting with player 1 |
| `a > b taking cards` | exactly these cards, wherever they are in `a`, in the order given |

`taking` takes any expression that gives cards, such as `move:cards` or `filter(middle, is_ace)`. It's all or nothing: if any of the cards isn't in `a`, nothing moves and the transfer fails. `taking` needs a single pile on each side, so not `players`.

Valid names for `from` or `to`:

| Name | Pile |
|---|---|
| `deck` | the deck |
| a name declared with `stack <name>` | that shared pile |
| `p:hand`, `p:collection` … | a pile of the player object `p`, such as the `player` argument of `player_move` |
| `players:2:hand`, `players:1:collection` … | a pile of a player chosen by number |
| `players` | every player's hand (destination only, not with `taking`) |

Moving from an empty pile does nothing and isn't a failure. A name that isn't a pile makes the transfer fail, and `transfer failed: unknown stack '<name>'` is printed to stderr.

## Expressions and values

Values are numbers, booleans (`true` / `false`), text, and stacks (lists of cards).

| Expression | Value |
|---|---|
| `26`, `-1` | number |
| `true`, `false` | boolean |
| `current_player` | number of the player whose turn it is |
| `deck` / a pile name | that stack of cards |
| a parameter name | the value passed in |
| `p:id`, `p:hand`, `c:value`, `move:cards`… | a field of an [object](#objects-players-cards-and-moves) |
| `players:2:id`, `players:2:collection`… | a field of a player chosen by number |
| any other word, e.g. `Ace` | the word as text |
| `f(a, b)` | the function's return value |
| `a + b`, `a - b` | arithmetic on numbers |
| `a < b`, `a > b` | `true` if `a` is less / greater than `b` (numbers only) |
| `a is b` | `true` if the values are equal |
| `a is not b` | `true` if the values differ |
| `a & b` | `true` if both are `true` |
| `a or b` | `true` if either is `true` |
| `( … )` | grouping |

**Precedence**, tightest first: `+` `-`, then `is` / `is not` / `<` / `>`, then `&`, then `or`. So `a + 1 > b & c is 2 or d` means `(((a + 1) > b) & (c is 2)) or d`. Use parentheses to group differently. Comparisons can't be chained: `a < b < c` is a parse error.

Arithmetic and `<` / `>` need numbers. With anything else they print a message to stderr and give `false`.

Equality compares type as well as value: the number `1` is not equal to the text `1`. Two stacks are equal when they hold the same cards in the same order.

## Built-in functions

| Function | Does | Returns |
|---|---|---|
| `shuffle(deck)` | Shuffles the main deck. It always shuffles `deck`, whatever you pass. | nothing |
| `count(stack)` | Counts cards. Gives `0` for anything that isn't a stack, or when called with no argument. | number |
| `sum(stack)` | Adds up the cards' [values](#card-values). | number |
| `cards_in_stack(cards, stack)` | Checks every card in `cards` is in `stack`. | boolean |
| `filter(stack, f)` | Calls your function `f` with each card. Keeps the cards where `f` returns `true`. The original stack is not changed. | stack |
| `valid_moves(f, g, …)` | Runs the player's chosen move; see [Moves](#moves). Only works inside `player_move`. | boolean |
| `winner(id)` | Adds a player to the winners list. Call it more than once for several winners. Without a number it prints an error and does nothing. | nothing |
| `end()` | Ends the game. Later moves are ignored. | nothing |
| `next_player()` | Moves `current_player` to the next player, wrapping from the last back to 1. | nothing |

`filter` example:

```
def is_heart(card){
    return(card:suit is Hearts)
}

def setup(){
    if(count(filter(deck, is_heart)) is 13){
        end()
    }
}
```

## Objects: players, cards and moves

**Player**: the first argument of `player_move` and of move functions.

| Field | Value |
|---|---|
| `p:id` | player number (1, 2, …) |
| `p:hand` | the player's hand, as a stack |
| `p:<name>` | a pile declared with `stack <name> per player` |

Player fields always show the current state, including transfers made earlier in the same move. Any player can also be reached by number: `players:2:hand`.

**Card**: the argument passed to a `filter` function.

| Field | Values |
|---|---|
| `c:rank` | `Ace`, `Two`, `Three`, `Four`, `Five`, `Six`, `Seven`, `Eight`, `Nine`, `Ten`, `Jack`, `Queen`, `King` (text, case-sensitive) |
| `c:suit` | `Spades`, `Hearts`, `Clubs`, `Diamonds` (text, case-sensitive) |
| `c:value` | the card's [value](#card-values) (number) |

**Move**: the second argument of `player_move` and of move functions.

| Field | Value |
|---|---|
| `move:name` | the move name the player typed, such as `take`, or empty text if they didn't give one |
| `move:cards` | the first group of cards typed (stack, maybe empty) |
| `move:target` | the group after the `/` (stack, maybe empty) |
