# cardlang for VS Code

Syntax highlighting for cardlang `.card` files. It covers:
- comments, `.( … )`, including nested brackets
- declarations: `name`, `players`, `deck standard without …`, `stack`, `number`, `player stack`, `player number`, `values` and `values <name>`
- functions and calls, built-ins, assignment (`p:score = …`), and fields such as `p:hand` or `players:2:collection`
- `check(rule, message)`, with the message shown as text
- `every`, `any` and `most`, including `group`
- card ranks and suits, and card literals such as `[7d]`

Brackets auto-close, and the "toggle block comment" command uses `.( … )`.

## Install

The extension isn't on the VS Code Marketplace. You build it from this folder and install it in one command. It needs Node.js, and `code` on your PATH:

```sh
cd editors/vscode
npm install                    # once
npm run install-extension      # build cardlang.vsix and install it
```

Run `npm run install-extension` again whenever the grammar changes, then reload VS Code (**Developer: Reload Window**). From the repo root, the same command is `npm --prefix editors/vscode run install-extension`.

If `code` isn't on your PATH, run `npm run package` to build `cardlang.vsix`, then use **Extensions → … → Install from VSIX…** in VS Code. On a Mac, Cmd+Shift+P → **Shell Command: Install 'code' command in PATH** adds `code`.

## Changing the grammar

The highlighting rules are in [`syntaxes/cardlang.tmLanguage.json`](syntaxes/cardlang.tmLanguage.json). The tests use the same highlighting engine as VS Code:

```sh
npm install
npm test
```

The tests check how specific pieces of syntax are highlighted. They also check that every game in `examples/` highlights without leaving a comment, `check` message or bracket open. When the language gains new syntax, update the grammar and add a test in the same PR.
