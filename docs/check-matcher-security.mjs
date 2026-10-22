import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createRequire } from 'node:module';

const generator = new URL('./node_modules/starlight-llms-txt/generator.ts', import.meta.url);
const fromPlugin = createRequire(generator);
const matcher = fromPlugin('micromatch');
const resolvedPackage = fromPlugin('micromatch/package.json');
assert.equal(resolvedPackage.name, 'picomatch');
assert.equal(resolvedPackage.version, '4.0.4');

// This override is valid only at the plugin's existing isMatch-only seam.
const source = readFileSync(generator, 'utf8');
assert.match(source, /import micromatch from 'micromatch'/);
assert.doesNotMatch(source, /\bmicromatch\s*\(/);
const usedMethods = [...source.matchAll(/\bmicromatch\.([a-zA-Z]+)/g)].map((match) => match[1]);
assert.ok(usedMethods.length > 0);
assert.deepEqual([...new Set(usedMethods)], ['isMatch']);
const lock = readFileSync(new URL('./bun.lock', import.meta.url), 'utf8');
assert.doesNotMatch(lock, /"[^"]*"\s*:\s*\["braces@/);

// Expected results were captured from the prior Micromatch 4.0.8 implementation.
const cases = [
  {
    "input": "index",
    "patterns": "index*",
    "expected": true
  },
  {
    "input": "guides/intro",
    "patterns": "guides/**",
    "expected": true
  },
  {
    "input": "guides/intro",
    "patterns": [
      "index*",
      "guides/**"
    ],
    "expected": true
  },
  {
    "input": "guides/intro",
    "patterns": "guides/{intro,setup}",
    "expected": true
  },
  {
    "input": "guides/setup",
    "patterns": "guides/{intro,setup}",
    "expected": true
  },
  {
    "input": "guide.txt",
    "patterns": "**/*.md",
    "expected": false
  },
  {
    "input": "draft/hidden",
    "patterns": "!draft/**",
    "expected": false
  },
  {
    "input": "sync/setup",
    "patterns": "!draft/**",
    "expected": true
  },
  {
    "input": "guide",
    "patterns": "@(guide|intro)",
    "expected": true
  },
  {
    "input": ".hidden",
    "patterns": "**",
    "expected": false
  },
  {
    "input": "sync/setup",
    "patterns": [],
    "expected": false
  },
  {
    "input": "sync/setup",
    "patterns": [
      "index*",
      "sync/*"
    ],
    "expected": true
  }
];
for (const { input, patterns, expected } of cases) {
  assert.equal(matcher.isMatch(input, patterns), expected, JSON.stringify({ input, patterns }));
}
assert.throws(() => matcher.isMatch('guide', 'x'.repeat(65537)), /length|exceed/i);
console.log('Matcher API, glob compatibility, missing Braces and oversized-input rejection passed.');
