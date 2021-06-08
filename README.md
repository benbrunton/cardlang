# cardlang

A DSL for specifying card games

## CLI usage
### (interactive mode)
```
cardlang # (or cargo run)
> build ./gamedef.card
> show deck
ace spades, two spades, three spades...
> start
> show player 1 hand
three hearts, four diamonds, five clubs
```

## run spec tests
```
cardlang test ./gamedef.card
```

## Todo
- is not comparison modifier
- filter deck in declaration
- limit on stack transfer
- user defined functions
- switch statements
- specify cards on transfer
- cards_in_stack inbuilt function
- spec test