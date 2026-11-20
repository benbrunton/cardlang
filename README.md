# cardlang

A DSL for specifying card games

New here? Start with the [user guides](docs/README.md): [getting started](docs/getting-started.md), [language reference](docs/language-reference.md), [CLI](docs/cli.md) and [known limitations](docs/limitations.md). Planned work is in the [roadmap](docs/ROADMAP.md).

## CLI usage
### (interactive mode)
```
cardlang play examples/simple_scopa.card   # (or cargo run -- play ...)
> build examples/simple_scopa.card
Game loaded
> start
> show middle
jack diamonds, ace spades, two spades, nine clubs
> show player 1
six hearts, seven spades, two clubs
> take 2c / 2s
> show player 1 collection
two spades, two clubs
> help
```
Press Tab to complete commands, file names, moves and cards.

## check a game file parses
```
cardlang test ./gamedef.card   # exits 1 if the file doesn't parse
```
(Running spec tests written inside a game file is planned; see the roadmap.)

## Editor support
Syntax highlighting for VS Code is in [`editors/vscode`](editors/vscode/README.md).

## Roadmap
Known bugs and planned work are tracked in [docs/ROADMAP.md](docs/ROADMAP.md).
