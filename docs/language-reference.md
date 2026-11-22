# Language reference

A cardlang file (`.card`) contains **declarations**, which set up the table, and **definitions**, which are functions holding the rules. Statements go inside function bodies.

- [Source text](#source-text)
- [Declarations](#declarations)
- [Functions](#functions)
- [Moves](#moves)
- [Statements](#statements)
- [Loops: every, any and most](#loops-every-any-and-most)
- [Transfers](#transfers)
- [Expressions and values](#expressions-and-values)
- [Built-in functions](#built-in-functions)
- [Objects: players and cards](#objects-players-and-cards)

## Source text

| Element | Rules |
|---|---|
| Comments | `.( any text )`. They can span lines, and nested parentheses are allowed: `.(a (b) c)`. |
| Words (symbols) | Start with a letter. Can contain letters, digits, `_` and `:`. Case-sensitive. |
| Numbers | Whole numbers only, such as `2` or `26`. Write negative numbers as `-1`. |
| Operators | `>` (transfer, or greater than inside an expression), `<`, `>=`, `<=`, `+`, `-`, `&`, `=` (assignment), `:` (after `every …`, `any …`, `most …`) |
| Card literals | `[7d]`, `[7d, kh, as]`: specific cards, written as at the CLI. See [Expressions](#expressions-and-values). |
| Whitespace | Spaces, tabs and newlines. Windows (`\r\n`) line endings are fine. |
| Keywords | `name`, `deck`, `players`, `current_player`, `stack`, `number`, `values`, `def`, `move`, `if`, `else`, `every`, `any`, `most`, `in`, `check`, `return`, `is`, `not`, `or`, `true`, `false` |

Write one statement per line. A line break inside `( … )` doesn't end the statement, so a long expression or call can continue on the next line:

```
return(best_in_suit(pile, Spades) + best_in_suit(pile, Hearts)
    + best_in_suit(pile, Clubs) + best_in_suit(pile, Diamonds))
```

## Declarations

Declarations go at the top level of the file, outside any function.

| Declaration | Meaning | Default |
|---|---|---|
| `name <word>` | The game's name, shown by `show name`. Must be a single word. | none |
| `players <number>` | Number of players. They are numbered `1` to `n` and start with empty hands. | `1` |
| `current_player <number>` | Which player's turn it is at the start. | `1` |
| `deck standard` | The deck: the standard 52 cards. | `standard` |
| `deck standard without Eight Nine Ten` | The standard deck with those ranks left out (here, 40 cards). | |
| `stack <word>` | A shared pile, empty at the start. | none |
| `player stack <word>` | A pile for **every player**, such as `player stack collection`, used as `p:collection`. (`hand` already exists.) | none |
| `number <word>` | A number for the whole game, such as `number dealer`. It starts at 0 and is changed with [`=`](#assignment). | none |
| `player number <word>` | A number for **every player**, such as `player number score`, used as `p:score`. It starts at 0. | none |
| `values { … }` | Changes card values; see [card values](#card-values). | Ace 1 … King 13 |
| `values <word> { … }` | A second set of card values, used as a card field; see [card values](#card-values). | |

Giving a declaration the wrong kind of value, such as `players two` or `name 5`, is a parse error.

`stack`, `number` and `player` declare **types**: `stack` / `number` on their own are shared by the whole game, and with `player` in front each player gets their own.

Every game has a deck called `deck`, the standard 52 cards unless `deck … without` leaves some out. It starts unshuffled, in order: ace to king of spades, then hearts, then clubs, then diamonds. The **top** of the deck is the last card, the king of diamonds.

### Card values

Each card has a value: Ace 1, Two 2 … Ten 10, Jack 11, Queen 12, King 13. A `values` table changes the value of any rank:

```
values {
    Jack 8, Queen 9, King 10
}
```

Entries are a rank name (as in `card:rank`) and a whole number. Commas and new lines between entries are both fine. Ranks you don't list keep their default value. Values are used by `card:value` and `sum(cards)`.

A **named** table gives every card an extra field with that name. Ranks it doesn't list are 0. Scopa uses one for the primiera:

```
values primiera {
    Seven 21, Six 18, Ace 16, Five 15, Four 14, Three 13, Two 12
    Jack 10, Queen 10, King 10
}
```

`card:primiera` is then 21 for a seven, and so on. The name can't be `rank`, `suit` or `value`.

## Functions

```
def name(arg1, arg2){
    statements…
}
```

Parameters can be separated by commas or spaces. A body can be on one line: `def setup(){ next_player() }`. The CLI's command words (`build`, `start`, `show`, `help`, `exit`) can't be used as function names.

Two function names are special. If a game defines them, the interpreter calls them for you:

| Function | Called when | Arguments |
|---|---|---|
| `setup` | `start` is run | none |
| `after_move` | after every successful [move](#moves), before the turn passes on | none |

Any other function runs only when you call it: `name(a, b)` as a statement, or inside an expression (`count(hand_of(player))`).

How calls work:

- **Arguments** are worked out in the calling function, then bound to the parameter names. Passing a player object passes the player itself. A transfer such as `p:hand > middle` inside the function changes that player's real hand.
- **Missing arguments** are `false`. **Extra arguments** are ignored.
- **Scope:** a function sees its own parameters, plus anything visible in the functions that called it. If two have the same name, the innermost one wins.
- **Return value:** the value from `return(...)`. A function that ends without returning, or that a `check` or transfer stops, gives `false`.
- **Recursion** is allowed, but stops at a depth of 256 calls (a message is printed to stderr).
- **Too much work:** setup or a single move that runs more than 1,000,000 statements and expressions is stopped. A move that's stopped is undone and rejected with `the move stopped: it did too much work`; setup stops with a message on stderr. The example games use a few thousand at most, so this only catches mistakes like a function that calls itself twice. A program using cardlang as a library can change the limit.
- Calling an **unknown function** prints `unknown function '<name>'` to stderr and gives `false`.
- If your function has the same name as a built-in, the built-in is used.

## Moves

A **move** is something a player can do on their turn. Declare it with `move` instead of `def`:

```
move take(player, played, captured){
    check(sum(captured) is sum(played), the cards you capture must add up to the card you play)
    middle > player:collection taking captured
    player:hand > player:collection taking played
}

move drop(player, played){
    player:hand > middle taking played
}
```

**Parameters:** the first is the [player](#objects-players-and-cards) making the move. The rest receive the groups of cards the player typed, in order, as piles. In `take 9d / 4c 5s`, `played` is `[9d]` and `captured` is `[4c, 5s]`.
- A group the player didn't type is an empty pile.
- Typing more groups than a move has parameters for rejects it, e.g. `drop takes one group of cards, without a /`.

**Which move runs:**
- **A named move** like `take 9d / 4c` runs `take`. A name that isn't a move is rejected, listing the moves there are.
- **No name** (`9d / 4c`): the moves are tried in the order they appear in the file, and the first one that succeeds is used.
- **A move without cards**, like Race's `move play(player)`, is typed by its name: `play`.

**Turns are handled for you:**
- **Only the current player can move.** Anyone else is told `it's not your turn`.
- **After a successful move,** `after_move()` runs if the game defines it, for example to deal more cards or end a hand.
- **Then the turn passes** to the next player, wrapping from the last back to 1. If the move or `after_move` sets `current_player` itself (or calls `next_player()`), that decides who plays next instead. This works even when it sets the same player, which gives them another turn.

**A move succeeds unless it fails.** It fails when a `check` fails, a transfer can't happen, an assignment can't be made, it returns `false`, or `after_move` fails.

**A failed move changes nothing.** Every card and number goes back to how it was before the move, including changes made before the failure. The turn doesn't pass. The CLI says why the move was rejected.

Move names can't be words that read as cards (`as`, `seven`, `10`), because the CLI would read them as cards.

## Statements

| Statement | Meaning |
|---|---|
| `from > to …` | Move cards. See [Transfers](#transfers). If the transfer can't happen, the function stops and returns `false`, like a failed `check`. |
| `name(args…)` | Call a function and discard its result. |
| `if(condition){ … }` | Run the block when the condition is true. |
| `if(condition){ … } else { … }` | Run the first block when the condition is true, otherwise the second. `else` can go on the line after the `}`. |
| `if(a){ … } else if(b){ … } else { … }` | Run the block for the first condition that is true. |
| `check(condition)` | When the condition is false, stop the function and return `false`. This works inside `if` and `every` blocks too. |
| `check(condition, message)` | The same, and if this rejects a move, the player sees `message` instead of the rule. See [check messages](#check-messages). |
| `<name> = <expression>` | Change a number. See [Assignment](#assignment). |
| `every player p { … }` | Run the block once for each player. See [Loops](#loops-every-any-and-most). |
| `return(expression)` | Stop the function and give back the value. This works inside `if` blocks too. |

A **condition** (in `if` and `check`) is true only when it evaluates to the boolean `true`. A function call or parameter can be used directly: `if(is_out(player))`. A number, text or stack is never true, so `if(count(deck))` is false. Write `if(count(deck) > 0)`.

### Assignment

```
dealer = 2
p:score = p:score + 1
players:2:score = 0
current_player = 1
```

`=` changes a number declared with `number` or `player number`, or `current_player`. The value must be a number. Assigning anything else, to something that isn't declared, or making `current_player` a player who doesn't exist makes the function stop, like a failed `check`, with the reason. Changes made by a move that then fails are undone along with everything else.

### Check messages

```
check(count(captured) > 0, put a / before the cards you're capturing, e.g. take 9d / 4c 5s)
```

Everything after the **first** comma, up to the `)` that closes the `check`, is the message, exactly as written. No quotes are needed:
- later commas are part of the message
- any characters can be used, including `/`, apostrophes and words like `is` or `or`
- brackets inside the message must be balanced: `(like this)` is fine, but a lone `)` ends the message
- a message can continue onto the next line; extra spaces and line breaks become single spaces

When a move is rejected by a `check` with a message, the CLI prints `move ignored: <message>`. Without a message, it prints the rule that failed.

## Loops: every, any and most

`every` goes through each player, each card in a pile, or each group of cards from a pile. It has two forms.

**Run a block for each one:**

```
every player p {
    deck > p:hand 3
}

every card c in middle {
    winner(c:value)
}
```

Players are visited in order, 1, 2, 3 …. A `check`, `return` or failed transfer inside the block ends the whole function, just as it would outside the loop.

**Test something** with a colon:

```
return(every player p: count(p:hand) is 0)        .(true if it holds for all of them)
return(any card c in middle: c:value is 7)        .(true if it holds for at least one)
```

`every …: <condition>` is `true` when the condition holds for all of them (and for none at all). `any …: <condition>` is `true` when it holds for at least one (so it's `false` for none). Both can be used anywhere a value is expected: in `if`, `check`, `return`, or combined with `&` and `or`. The condition runs to the end of the enclosing brackets, so `every player p: a & b` tests `a & b` for each player.

**Find the leader** with `most`:

```
winner(most player p: p:score)
```

`most player p: <number>` gives the id of the one player with the highest value, or `0` if two or more share it. A `0` matches no player, so it's handy for "a tie scores nothing":

```
def give_point(id){
    every player p {
        if(p:id is id){
            p:score = p:score + 1
        }
    }
}

give_point(most player p: count(p:collection))
```

| Form | Goes through |
|---|---|
| `player p` | each player, as a [player object](#objects-players-and-cards) |
| `card c in <pile>` | each card in the pile, as a [card object](#objects-players-and-cards), bottom first |
| `group g in <pile>` | every combination of one or more cards from the pile, each as a pile (so `sum(g)`, `count(g)` work) |

`<pile>` can be any expression that gives cards: `middle`, `p:hand`, `players:2:collection`, `filter(deck, is_ace)`, `[7d, kh]`. `most` only works with `player`.

`group` is how Scopa checks whether any cards in the middle add up to a card:

```
any card c in player:hand: any group g in middle: sum(g) is c:value
```

A pile of n cards has 2ⁿ − 1 groups, so `group` refuses piles of more than 20 cards. 20 cards is about a million groups and takes up to half a second.

The colon can touch the word before it or stand alone: `every player p: …` and `every card c in middle : …` both work.

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

`taking` takes any expression that gives cards, such as a move's `played` parameter, `[7d]` or `filter(middle, is_ace)`. It's all or nothing: if any of the cards isn't in `a`, nothing moves and the transfer fails. `taking` needs a single pile on each side, so not `players`.

Valid names for `from` or `to`:

| Name | Pile |
|---|---|
| `deck` | the deck |
| `[7d, kh]` | specific cards, as the source of `taking` |
| a name declared with `stack <name>` | that shared pile |
| `p:hand`, `p:collection` … | a pile of the player object `p`, such as a move's `player` parameter |
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
| a name declared with `number` | its value |
| `p:id`, `p:hand`, `p:score`, `c:value`… | a field of an [object](#objects-players-and-cards) |
| `players:2:id`, `players:2:collection`, `players:2:score`… | a field of a player chosen by number |
| `[7d]`, `[7d, kh, as]` | those cards, as a pile. Cards are written as at the CLI: `7h`, `10d`, `ks` or names such as `seven hearts`. |
| any other word, e.g. `Ace` | the word as text |
| `f(a, b)` | the function's return value |
| `a + b`, `a - b` | arithmetic on numbers |
| `a < b`, `a > b`, `a <= b`, `a >= b` | compare numbers |
| `every … : …`, `any … : …` | `true` if the condition holds for all / at least one; see [Loops](#loops-every-any-and-most) |
| `most player p: …` | the id of the player with the highest value, or `0` if tied |
| `a is b` | `true` if the values are equal |
| `a is not b` | `true` if the values differ |
| `a & b` | `true` if both are `true` |
| `a or b` | `true` if either is `true` |
| `( … )` | grouping |

**Precedence**, tightest first: `+` `-`, then `is` / `is not` / `<` / `>` / `<=` / `>=`, then `&`, then `or`. So `a + 1 > b & c is 2 or d` means `(((a + 1) > b) & (c is 2)) or d`. Use parentheses to group differently. Comparisons can't be chained: `a < b < c` is a parse error.

Arithmetic and `<` / `>` / `<=` / `>=` need numbers. With anything else they print a message to stderr and give `false`.

Equality compares type as well as value: the number `1` is not equal to the text `1`. Two stacks are equal when they hold the same cards in the same order.

## Built-in functions

| Function | Does | Returns |
|---|---|---|
| `shuffle(deck)` | Shuffles the main deck. It always shuffles `deck`, whatever you pass. A game started with a [seed](cli.md#interactive-mode-cardlang) shuffles the same way every time. | nothing |
| `count(stack)` | Counts cards. Gives `0` for anything that isn't a stack, or when called with no argument. | number |
| `sum(stack)` | Adds up the cards' [values](#card-values). | number |
| `cards_in_stack(cards, stack)` | Checks every card in `cards` is in `stack`. | boolean |
| `filter(stack, f)` | Calls your function `f` with each card. Keeps the cards where `f` returns `true`. The original stack is not changed. | stack |
| `winner(id)` | Adds a player to the winners list. Call it more than once for several winners. Without a number it prints an error and does nothing. | nothing |
| `end()` | Ends the game. Later moves are ignored. | nothing |
| `next_player()` | Moves `current_player` to the next player, wrapping from the last back to 1. | nothing |
| `gather()` | Puts every card from every pile and hand back into the deck, for starting a new hand. Shuffle afterwards. | nothing |

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

## Objects: players and cards

**Player**: the first parameter of a move, or the `p` in `every player p`.

| Field | Value |
|---|---|
| `p:id` | player number (1, 2, …) |
| `p:hand` | the player's hand, as a stack |
| `p:<name>` | a pile declared with `player stack <name>`, or a number declared with `player number <name>` |

Player fields always show the current state, including transfers made earlier in the same move. Any player can also be reached by number: `players:2:hand`.

**Card**: the argument passed to a `filter` function, or the `c` in `every card c in …`.

| Field | Values |
|---|---|
| `c:rank` | `Ace`, `Two`, `Three`, `Four`, `Five`, `Six`, `Seven`, `Eight`, `Nine`, `Ten`, `Jack`, `Queen`, `King` (text, case-sensitive) |
| `c:suit` | `Spades`, `Hearts`, `Clubs`, `Diamonds` (text, case-sensitive) |
| `c:value` | the card's [value](#card-values) (number) |
| `c:<name>` | its value in a [named values table](#card-values), such as `c:primiera` |
