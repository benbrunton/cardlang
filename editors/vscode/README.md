# cardlang for VS Code

Syntax highlighting for cardlang `.card` files. It covers:
- comments, `.( … )`, including nested brackets
- declarations: `name`, `players`, `stack … per player`, `values { … }`, and the proposed `var` and `deck … without`
- functions and calls, built-ins, and fields such as `p:hand` or `players:2:collection`
- `check(rule, message)`, with the message shown as text
- `every` loops and tests, and the proposed `any` and `group`
- card ranks and suits, and the proposed `[7d]` card literals

Brackets auto-close, and the "toggle block comment" command uses `.( … )`.

## Install

The extension isn't on the VS Code Marketplace. Build it from this folder and install the file:

```sh
cd editors/vscode
npx @vscode/vsce package --skip-license      # creates cardlang-0.1.0.vsix
code --install-extension cardlang-0.1.0.vsix
```

Then open any `.card` file. If `code` isn't on your PATH, use **Extensions → … → Install from VSIX…** in VS Code instead.

To update after changing the grammar, run the same two commands again and reload the window (**Developer: Reload Window**).

## Changing the grammar

The highlighting rules are in [`syntaxes/cardlang.tmLanguage.json`](syntaxes/cardlang.tmLanguage.json). The tests use the same highlighting engine as VS Code:

```sh
npm install
npm test
```

The tests check how specific pieces of syntax are highlighted. They also check that every game in `examples/` highlights without leaving a comment, `check` message or bracket open. When the language gains new syntax, update the grammar and add a test in the same PR.
