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
- line numbers in parser error messaging
- check operator
- next_player inbuilt func