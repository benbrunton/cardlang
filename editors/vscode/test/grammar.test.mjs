// Checks the cardlang grammar with vscode-textmate, the engine VS Code uses.
// Run with: npm test
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import { createRequire } from 'node:module';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import vsctm from 'vscode-textmate';
import oniguruma from 'vscode-oniguruma';

const here = dirname(fileURLToPath(import.meta.url));
const require = createRequire(import.meta.url);
const examples = join(here, '..', '..', '..', 'examples');

const wasm = readFileSync(require.resolve('vscode-oniguruma/release/onig.wasm'));
await oniguruma.loadWASM(wasm.buffer);

const registry = new vsctm.Registry({
  onigLib: Promise.resolve({
    createOnigScanner: (patterns) => new oniguruma.OnigScanner(patterns),
    createOnigString: (s) => new oniguruma.OnigString(s)
  }),
  loadGrammar: async () => vsctm.parseRawGrammar(
    readFileSync(join(here, '..', 'syntaxes', 'cardlang.tmLanguage.json'), 'utf8'),
    'cardlang.tmLanguage.json'
  )
});
const grammar = await registry.loadGrammar('source.cardlang');

// tokenizes source and returns [text, innermost scope] for each token
function tokenize(source) {
  let state = vsctm.INITIAL;
  const tokens = [];
  for (const line of source.split('\n')) {
    const result = grammar.tokenizeLine(line, state);
    for (const t of result.tokens) {
      const text = line.slice(t.startIndex, t.endIndex);
      if (text.trim() !== '') {
        tokens.push([text.trim(), t.scopes[t.scopes.length - 1], t.scopes]);
      }
    }
    state = result.ruleStack;
  }
  return tokens;
}

// the innermost scope given to some text in the source
function scopeOf(source, text) {
  const token = tokenize(source).find(([t]) => t === text);
  assert.ok(token, `no token "${text}" in: ${source}`);
  return token[1];
}

function allScopesOf(source, text) {
  const token = tokenize(source).find(([t]) => t === text);
  assert.ok(token, `no token "${text}" in: ${source}`);
  return token[2];
}

test('comments, including nested brackets and several lines', () => {
  const source = '.(a comment (with brackets)\n  over two lines)\nname x';
  assert.equal(scopeOf(source, '.('), 'punctuation.definition.comment.begin.cardlang');
  assert.ok(allScopesOf(source, 'over two lines').includes('comment.block.cardlang'));
  assert.equal(scopeOf(source, 'name'), 'storage.type.declaration.cardlang');
});

test('declarations', () => {
  const source = 'name scopa\nplayers 2\nstack middle\nplayer stack collection\nnumber dealer\nplayer number score\nvalues primiera {\n    Seven 21\n}';
  assert.equal(scopeOf(source, 'scopa'), 'entity.name.type.game.cardlang');
  assert.equal(scopeOf(source, 'players'), 'storage.type.declaration.cardlang');
  assert.equal(scopeOf(source, 'middle'), 'variable.other.declared.cardlang');
  assert.equal(scopeOf(source, 'player'), 'storage.modifier.player.cardlang');
  assert.equal(scopeOf(source, 'collection'), 'variable.other.declared.cardlang');
  assert.equal(scopeOf(source, 'number'), 'storage.type.declaration.cardlang');
  assert.equal(scopeOf(source, 'dealer'), 'variable.other.declared.cardlang');
  assert.equal(scopeOf(source, 'score'), 'variable.other.declared.cardlang');
  assert.equal(scopeOf(source, 'primiera'), 'variable.other.values-table.cardlang');
  assert.equal(scopeOf(source, 'Seven'), 'variable.other.enummember.card.cardlang');
  assert.equal(scopeOf(source, '21'), 'constant.numeric.cardlang');
});

test('functions, calls and fields', () => {
  const source = 'def take(player, move){\n    middle > player:collection taking move:target\n    winner(count(p:hand))\n}';
  assert.equal(scopeOf(source, 'def'), 'storage.type.function.cardlang');
  assert.equal(scopeOf(source, 'take'), 'entity.name.function.cardlang');
  assert.equal(scopeOf(source, 'player'), 'variable.parameter.cardlang');
  assert.equal(scopeOf(source, 'move'), 'variable.parameter.cardlang');
  assert.equal(scopeOf('middle > player:collection', 'player'), 'variable.other.object.cardlang');
  assert.equal(scopeOf(source, ':collection'), 'variable.other.property.cardlang');
  assert.equal(scopeOf(source, 'taking'), 'keyword.other.cardlang');
  assert.equal(scopeOf(source, '>'), 'keyword.operator.comparison.cardlang');
  assert.equal(scopeOf(source, 'winner'), 'support.function.builtin.cardlang');
  assert.equal(scopeOf(source, 'count'), 'support.function.builtin.cardlang');
});

test('a check message is text, even with commas, slashes and brackets', () => {
  const source = "check(count(move:cards, x) is 1, play one card, e.g. take 9d / 4c (or not))\nend()";
  const message = tokenize(source)
    .filter(([, , scopes]) => scopes.includes('string.unquoted.message.cardlang'))
    .map(([t]) => t);
  assert.deepEqual(message, ['play one card, e.g. take 9d / 4c', '(', 'or not', ')']);
  // the comma inside count(...) is part of the rule, not the start of the message
  assert.equal(scopeOf(source, 'count'), 'support.function.builtin.cardlang');
  // and the message ends with the check
  assert.equal(scopeOf(source, 'end'), 'support.function.builtin.cardlang');
});

test('a check message can continue on the next line', () => {
  const source = 'check(false, a long\n    message)\nwinner(1)';
  assert.ok(allScopesOf(source, 'message').includes('string.unquoted.message.cardlang'));
  assert.equal(scopeOf(source, 'winner'), 'support.function.builtin.cardlang');
});

test('every and any', () => {
  const source = 'every player p {\n}\nreturn(any card c in middle: any group g in p:hand: sum(g) is c:value)\nwinner(most player q: q:score)';
  assert.equal(scopeOf(source, 'most'), 'keyword.control.loop.cardlang');
  assert.equal(scopeOf(source, 'q'), 'variable.parameter.loop.cardlang');
  assert.equal(scopeOf(source, 'every'), 'keyword.control.loop.cardlang');
  assert.equal(scopeOf(source, 'any'), 'keyword.control.loop.cardlang');
  assert.equal(scopeOf(source, 'card'), 'storage.type.loop-kind.cardlang');
  assert.equal(scopeOf(source, 'in'), 'keyword.control.loop.cardlang');
  assert.equal(scopeOf(source, 'c'), 'variable.parameter.loop.cardlang');
});

test('keywords, constants, deck declarations and card literals', () => {
  const source = 'if(a is not 1 or b >= 2 & true){\n} else {\n    p:score = p:score + 1\n    check(cards_in_stack([7d], p:collection))\n}\ndeck standard without Eight Nine Ten';
  assert.equal(scopeOf(source, 'if'), 'keyword.control.cardlang');
  assert.equal(scopeOf(source, 'else'), 'keyword.control.cardlang');
  assert.equal(scopeOf(source, 'is'), 'keyword.operator.word.cardlang');
  assert.equal(scopeOf(source, '>='), 'keyword.operator.comparison.cardlang');
  assert.equal(scopeOf(source, 'true'), 'constant.language.boolean.cardlang');
  assert.equal(scopeOf(source, '='), 'keyword.operator.assignment.cardlang');
  assert.equal(scopeOf(source, '[7d]'), 'variable.other.enummember.card-literal.cardlang');
  assert.equal(scopeOf(source, 'standard'), 'entity.name.type.deck.cardlang');
  assert.equal(scopeOf(source, 'without'), 'keyword.other.cardlang');
});

test('plain names are variables', () => {
  const source = 'def setup(){\n    deck > middle 4\n    last_capturer = 1\n}';
  assert.equal(scopeOf(source, 'middle'), 'variable.other.readwrite.cardlang');
  assert.equal(scopeOf(source, 'last_capturer'), 'variable.other.readwrite.cardlang');
  assert.equal(scopeOf(source, 'deck'), 'variable.language.cardlang');
});

test('every example game tokenizes, and brackets are balanced at the end', () => {
  for (const file of readdirSync(examples).filter((f) => f.endsWith('.card'))) {
    const source = readFileSync(join(examples, file), 'utf8');
    let state = vsctm.INITIAL;
    for (const line of source.split('\n')) {
      state = grammar.tokenizeLine(line, state).ruleStack;
    }
    // back at the top level: no comment, check or bracket left open
    assert.equal(state.depth, 1, `${file}: something is left open at the end of the file`);
  }
});
