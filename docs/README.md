# cardlang user guides

cardlang is a small language for describing card games. You write the rules of a game in a `.card` file, then load and play it with the `cardlang` interpreter.

| Guide | Read it when you want to… |
|---|---|
| [Getting started](getting-started.md) | build cardlang and write and play your first game |
| [Language reference](language-reference.md) | look up the syntax, built-in functions and how values behave |
| [Using the CLI](cli.md) | find every interpreter command and what it prints |
| [Known limitations](limitations.md) | understand an error, or check whether something is supported yet |
| [Roadmap](ROADMAP.md) | see known bugs and planned work, in priority order |

The language is still early and changing. The guides describe what the code on this branch actually does, including its rough edges. If the behaviour changes, update the guide too.

## Example games

- [`examples/race.card`](../examples/race.card): the game built in the getting started guide.
- [`examples/turns.card`](../examples/turns.card): a minimal turn-based game.
- [`examples/simple_scopa.card`](../examples/simple_scopa.card): Simple Scopa, a fuller game where players choose which cards to play. It shows named moves (`take 9d / 4c 5s`), card values, `taking` transfers and a pile for each player.
