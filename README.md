# cardlang

A DSL for specifying card games

New here? Start with the [user guides](docs/README.md): [getting started](docs/getting-started.md), [language reference](docs/language-reference.md), [CLI](docs/cli.md) and [known limitations](docs/limitations.md).

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
- filter deck in declaration
- limit on stack transfer
- switch statements
- specify cards on transfer
- cards_in_stack inbuilt function
- spec test