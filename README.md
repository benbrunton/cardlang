# cardlang

A DSL for specifying card games

## CLI usage (interactive mode)
```
cardlang (or cargo run)
> build ./gamedef.card
> show deck
ace spades, two spades, three spades...
> start
> show player 1 hand
three hearts, four diamonds, five clubs
```

## Todo
- player_move handler
- args in function definitions
- check operator
- is comparison operator
- if conditional flow
- object attributes (player hand / player id)
- count, winner, end, next_player inbuilt functions