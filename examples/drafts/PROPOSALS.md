# Draft specs: Klondike, Texas Hold'em, Gin Rummy and Pusoy Dos

Four games written in the cardlang we'd like to have, to find out what it's missing. None of them parse yet. Everything new is listed below once, with the drafts that use it.

| Draft | What it pushed on |
|---|---|
| [`klondike.card`](klondike.card) | face-down cards, several piles of the same kind, moving a card *and everything on top of it*, asking which pile a card is in |
| [`holdem.card`](holdem.card) | chips and betting, skipping players who've folded, moves that take a number, ranking poker hands |
| [`gin_rummy.card`](gin_rummy.card) | a turn made of two moves (take, then discard), melds that players lay out, remembering a card between moves |
| [`pusoy_dos.card`](pusoy_dos.card) | a game's own rank order (Twos high), suit order to break ties, rounds that end when everyone else passes, comparing combinations |
| [`pusoy_dos_pickering.card`](pusoy_dos_pickering.card) | a variant written as changes to another game, jokers as wildcards, a card order that turns round mid-game, any number of players |

The existing language held up better than expected: `move` with `/` groups fits Gin's `knock 2c / 7h 8h 9h / 5s 5d 5c` exactly, `after_move` plus setting `current_player` covers both a two-step turn and poker's turn order, and `every …: … or …` handles the poker table limit without anything new. Pusoy Dos needed only one new thing (18): its rank order is a `values` table, a round ending after three passes is a `number` and `after_move`, and the turn comes back to the last player who played without the game having to set it.

## Proposals

### 1. `let`: local names
```
let card = moving:top
let knocker = first player p: p:knocked
```
A name that lasts until the end of the function. It can hold any value (card, pile, player, number) and be changed with `=`. Without it, every draft repeated `moving:top` and `onto:top:pile` many times or split moves into helper functions just to name something. **All four.**

### 2. `none`
The value for "nothing": what `first pile c in columns: …` gives when no pile matches, the `:top` of an empty pile, and a `card` declaration that hasn't been set yet. You test it with `is none` / `is not none`. **Klondike, Gin.**

### 3. Face up and face down
```
stack stock face down
deck > column 3 face down
turn_up(column:top)
turn_up(p:hand)
```
- Every card is face up or face down. `deck` and every hand are face down; other piles are face up unless declared `face down`.
- A card turns to match the pile it lands on. A transfer can override that with `face up` or `face down` at the end.
- `turn_up(cards)` turns a card or a pile of cards face up. `c:face_up` says which way a card is.
- **Who sees what:** everyone sees face-up cards, and you can always see your own hand. Nobody sees other face-down cards.

That one rule gives Klondike its hidden columns, poker its hole cards (and the showdown is just `turn_up(p:hand)`), and Gin its face-down knock discard. It's also most of roadmap item 50 (visibility). **All three.**

### 4. Numbered piles
```
stack columns 7
player stack melds 3

columns:3
every pile column in columns { deck > column column:id face down }
```
A family of piles numbered from 1, read like `players:2`. `pile` joins `player`, `card` and `group` as something you can loop over, and a pile has an `:id`. Plural names read best (`columns`, `foundations`) because they match `players`. **Klondike, Gin.**

### 5. Pile and card fields: `:top`, `:pile`, `:face_up`
```
moving:top               .(the top card, as a card)
target:pile              .(the pile a card is in)
foundation:top:suit
```
A move's card group is a pile, so a one-card move needed `count(played) is 1` and then had no way to ask about *the* card. `:top` (and `:bottom`) fixes that. `card:pile` saves the player typing which pile they mean: `put 7h / 8s` finds both piles from the cards. **Klondike, Gin, Pusoy Dos.**

### 6. Transfers: `from`, and counts as expressions
```
card:pile > target:pile from card    .(the card and everything on top of it)
deck > column column:id              .(the count can be any number expression)
```
`taking` moves exactly the cards named, and `from` moves a card plus all the cards on top of it, which is how patience games move runs. **Klondike.**

### 7. `in`: is it there?
```
played in player:hand          .(replaces cards_in_stack(played, player:hand))
card:pile in columns           .(is this pile one of the columns?)
```
`cards_in_stack` would be removed. **Klondike, Gin, Pusoy Dos.**

### 8. Adding and taking away piles: `+` and `-`
```
poker_rank(p:hand + board)
is_meld(meld + cards)
```
Both give a new pile and leave the originals alone. **Hold'em, Gin.**

### 9. The quantifier family
Today there are `every`, `any` and `most`. The drafts needed a few more. They all have the same shape, `<word> player p: …` or `<word> card c in <pile>: …`, so it's one idea, not six:

| Word | Gives | Example |
|---|---|---|
| `every` / `any` | true or false | `every pile f in foundations: count(f) is 13` |
| `count` | how many match | `count player p: in_hand(p)` |
| `sum` / `min` / `max` | a number | `sum card c in p:hand: c:deadwood`, `max player p: strength(p)` |
| `first` | the first one that matches, or `none` | `first pile column in columns: count(column) is 0` |
| `most` | the one with the highest value, or `none` for a tie | `most player p: p:score` |
| `next` | the next player round the table that matches, or `none` | `next player p after dealer: in_hand(p)` |

- `next` starts after the current player, or after the one named with `after`. Its condition is optional: `next player after dealer`.
- The functions `count(pile)`, `sum(pile)`, `min(pile)` and `max(pile)` stay, as short forms that use the card values. `min` and `max` also take two numbers: `min(2, big:chips)`.

`next` replaces the need for a "skip folded players" feature. **All four.**

The condition still runs to the end of the brackets, so `count player p: in_hand(p) is 1` doesn't mean what it looks like. The drafts put each count in a small function (`players_in_hand() is 1`), which reads better anyway. If that gets annoying, the alternative is to make the condition stop at `is`, `<` and `>` when the quantifier gives a number.

### 10. Players are their numbers
```
dealer = next player after dealer
current_player = player
winner(first player p: p:chips > 0)
```
A player and their number are interchangeable: `p is 2` is true for player 2, a player can be stored in a `number`, and `winner` and `current_player` accept either. That lets `first`, `next` and `most` give back players you can use straight away (`survivor:chips`) while staying compatible with today's `most`, which gives a number. Scopa keeps working. **Hold'em, Gin.**

### 11. More declared types: `flag` and `card`
```
player flag folded        .(true or false; starts false)
player card taken         .(a card, or none; starts none)
```
`p:folded = true` and `if(p:folded)` read better than `p:folded is 1`. Gin needs `card` to remember which card a player took from the discard pile until their discard. These follow the same pattern as `stack` and `number`. **Hold'em and Gin use flag; Gin uses card.**

### 12. `not` (roadmap 65), and `and` for `&`
```
must(not player:drawn, …)
```
Every draft wanted `not`. Also, `or` is a word and `&` is a symbol, and in the drafts `&` is the odd one out. Suggest `and`, keeping `&` working for a while. **All three.**

### 13. Number parameters on moves
```
move raise(player, number amount)     .(typed as: raise 10)
```
Move parameters are card groups unless marked `number`, matching how declarations are typed. This avoids a clash with cards, since a bare `10` reads as the ten. **Hold'em.**

### 14. `*` and `/` (roadmap 39b)
Splitting a pot needs `/`: `let share = pot / winners`. Division rounds down. **Hold'em.** Pusoy Dos uses `*` to rank every card in the deck: `card:pusoy * 4 + card:suit_order`.

### 15. `use`: libraries, and poker hands as a built-in
```
use poker      .(gives poker_rank(cards): higher is better)
```
Most of poker ranking is easy to write in cardlang: sets, flushes and straights are a few lines each, the same as Gin's `is_set` and `is_run`. The hard part is breaking ties, where kickers are compared one by one. That needs sorting and comparing lists, and fitting it into a single number overflows. Adding sorting and lists to the language for one game would make it bigger for everyone. Instead, `poker_rank` would be a library written in Rust that gives each hand a number (kind × 15⁵ plus the deciding cards in order), and a game uses it with `use poker`. The same `use` could later load libraries written in cardlang. Lots of games use poker hands (Stud, Draw, Pai Gow, video poker), so it's a reasonable thing to provide. **Hold'em.**

### 15b. Not every game with poker hands needs `use poker`
Pusoy Dos has straights, flushes, full houses and so on, but writes them in about 40 lines of cardlang without the library. Hands of the same kind are compared by one card (the highest, or the rank of the three or the four), so each hand fits in one number: `kind × 100 + that card`. Hold'em needs the library because of kickers, where several cards are compared one after another. That's a reason to keep `use poker` small, and Hold'em the only game that needs it.

### 16. `must` replaces `check`
```
must(count(played) is 1, play one card, e.g. drop 9d)
must(card:colour is not target:colour, it must be the other colour)
```
Same behaviour as `check`, with a better name. A rule reads as a rule: "the card must be the other colour". It also frees up `check` as a move name, so Hold'em has a real `check` move alongside `call`. `check` would be kept working for a while and then removed. **All four.**

### 17. The language knows about cards: `card:colour`
```
card:colour        .(Red for hearts and diamonds, Black for spades and clubs)
```
Facts that are true of a card in every game belong to the language, not the game spec. Today a card has `:rank`, `:suit` and `:value`; Klondike also needed its colour, and without this every patience game would define the same `red(card)` function. Other candidates to add when a game needs them: `:face` (true for Jack, Queen and King) and the next or previous rank. Things that change from game to game, such as what a card is worth, stay in `values` tables. **Klondike.**

### 18. Suits in value tables
```
values suit_order { Clubs 1, Spades 2, Hearts 3, Diamonds 4 }

card:suit_order
```
A `values` table can list suits instead of ranks. Pusoy Dos breaks ties between cards of the same rank by suit, and so do other climbing games (Big Two, Tiến lên) and many trick-taking games with a trump order. A table lists either ranks or suits, not both. Together with `*`, it gives each card a single place in the deck: `card:pusoy * 4 + card:suit_order`. **Pusoy Dos.**

### 19. Variants: `extends` and `base()`
```
name pickering_pusoy_dos
extends pusoy_dos

values suit_order { Clubs 1, Hearts 2, Diamonds 3, Spades 4 }

def card_strength(card){
    if(reversed){
        return(61 - base(card))
    }
    return(base(card))
}
```
House rules and variants are how most card games are actually played, so a variant should be a short list of changes, not a copy of the whole game.
- `extends pusoy_dos` starts from everything in `pusoy_dos.card` (in the same folder).
- Anything the variant defines with the same name replaces the original: a `def`, a `move`, a `values` table, a declaration such as `players`. A replaced move keeps its place in the move order. Everything else is added.
- Inside a replacement, `base(…)` runs the original. That's what keeps variants short: Pickering's jokers are one line (`max version v in cards: base(v)`), and its reversal wraps `card_strength` without repeating it.
- The variant's header only describes what's different.

It also gives a rule for writing games: **the functions a game names are the places a variant can change it.** Pickering only works in three short functions because the standard game sends every comparison through `strength` and every card through `card_strength`. It's another reason for principle 7 below. **Pusoy Dos (Pickering).**

### 20. Jokers and wildcards
```
deck standard with jokers      .(adds two jokers, typed jk)

max version v in cards: strength(v)
```
- `with jokers` adds two jokers to the deck. A joker's `:rank` and `:suit` are `Joker`.
- `version` joins the quantifiers (proposal 9): each version of a pile is the pile with every joker replaced by a real card. A pile without jokers has one version, itself. So "a joker is whatever card makes your hand best" is `max version v in cards: …`, and "the move is allowed if the jokers can be read some way that works" is `any version v in cards: …`.
- Two jokers give 1,378 versions (the order of the jokers doesn't matter). That's fine for checking a move, but it's the kind of work the per-move limit (roadmap 55) needs to allow for.

Wildcards in other games (Canasta's twos, Rummy's jokers) fit the same pattern. **Pusoy Dos (Pickering).**

### 21. A range of players
```
players 2 to 6
players 2 or more

count player        .(how many are playing)
```
Many games take a range of players: Pickering Pusoy Dos takes two or more, Hold'em anything from two to ten. Writing one file per count would be silly.
- `players 4` works as now. `players 2 to 6` and `players 2 or more` give a range.
- The number is chosen when the game starts: `start 3` at the CLI, `cardlang play pickering.card 3`, or a number passed to the engine when it's embedded. For a game with a range, `start` on its own says which numbers it takes.
- `count player` is the number playing: proposal 9's `count` with no condition, like `next player after dealer`. Pusoy Dos ends a round when `passes is (count player) - 1`.
- Most of the language already copes: `player stack` and `player number` give every player their own, `deck > players end` deals everything round the table, and `every`, `next` and `most` go through however many there are. What doesn't cope is a number written into the game, such as `passes is 3`, or a deal of a fixed size (`deck > players 13`) when there aren't enough cards for everyone.
- A game that can't work above some number should give it with `to`. `or more` really means any number; with enough players some hands are empty, and that's the game's problem.

A variant can change the number with its own `players` declaration (proposal 19), which is how Pickering takes two or more players when the standard game takes four. **Pusoy Dos (both).**

## Smaller observations

- **A turn made of two moves** (Gin's take, then discard) works today: the first move sets `current_player = player` and a flag. `keep_turn()` would say it more clearly, but it doesn't seem worth a new built-in yet.
- **A flag shared by the game** (`flag reversed`) follows from proposal 11, the same way `number` goes with `player number`.
- **`>` means two things**: a transfer as a statement, "greater than" inside an expression. Statement position separates them and the drafts never got confused, but it's worth keeping in mind before adding more symbols.
- **`:` also means two things**: a field (`p:score`) and the start of a condition (`every player p: …`). `every player p: p:score > 0` has both. Spacing helps; it hasn't been a problem yet.

## Functions, operators, fields or quantifiers?

The drafts suggest a rule of thumb based on how you'd say it in English:

| You'd say… | Write it as | Examples |
|---|---|---|
| "the *x* of *y*" | a **field**, `y:x` | `card:suit`, `card:colour`, `pile:top`, `card:pile`, `p:chips` |
| "*a* is/is in/plus/beats *b*" | an **operator** between two values | `a is b`, `card in hand`, `hand + board`, `a < b` |
| "do *x*" | a **function** used as a statement | `shuffle(deck)`, `turn_up(c)`, `end()`, `winner(p)` |
| "*x* must be so" | a **rule**, `must(…)` | `must(count(played) is 1, play one card)` |
| "the *x* of these" / "how many" | a **function** that gives a value | `count(pile)`, `sum(pile)`, `poker_rank(cards)` |
| "for every / any / the first / the most…" | a **quantifier** | `every player p: …`, `first pile c in columns: …`, `max version v in cards: …` |
| "move these cards there" | a **transfer** | `deck > players 10` |

Some principles behind it:

1. **Operators are for relations and combinations between exactly two values** that read as a sentence left to right: `played in player:hand`. If it needs a third input, a choice of how, or has a side effect, it's a function.
2. **A small, fixed set of operators.** No operators defined by games. Every operator should be guessable by someone who has never read the docs, and the precedence table should stay short enough to remember.
3. **Words before symbols** when a plain word exists: `is`, `not`, `or`, `in`, `and`. Symbols only where they're universal: `+ - * / < >`.
4. **Fields for things a value has, not things you work out.** `pile:top` and `card:colour` are fields because the pile has a top card and the card has a colour. `count(pile)` stays a function because it's a calculation, and so does anything a game defines for itself (`can_lift(card)`), since games can't add fields to cards except through value tables.
5. **The language provides facts about cards; the game provides rules.** A card's colour is the same in every game, so a spec shouldn't have to define it. What a card is worth isn't, so that stays in the spec.
6. **One way to say each thing.** When `in` arrives, `cards_in_stack` goes. The short forms `count(pile)` and `sum(pile)` are fine next to the quantifiers because they use the same word for the same idea.
7. **Name a function when an expression gets clever.** `players_in_hand() is 1`, `can_lift(card)` and `goes_home(card, f)` are what make the drafts readable. A function name is free documentation, and the language should keep defining them easy.
