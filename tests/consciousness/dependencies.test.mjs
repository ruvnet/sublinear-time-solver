// consciousness-explorer release hygiene (weaver ReleaseFix, 2026-11-10).
//
// 1.1.2 shipped with `blessed` + `blessed-contrib` declared but never
// imported; blessed-contrib alone pulled 3 high + 2 moderate advisories
// (lodash, xml2js) into every install. Its `repository` pointed at
// ruvnet/consciousness-explorer, a repo that does not exist; the package
// is built and published from this repo. These tests pin all of that.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, readdirSync, statSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const pkgDir = join(here, '..', '..', 'src', 'consciousness-explorer');
const pkg = JSON.parse(readFileSync(join(pkgDir, 'package.json'), 'utf8'));
const deps = pkg.dependencies ?? {};

// Directories and files that end up in the published tarball and run.
const SHIPPED = ['index.js', 'bin', 'lib', 'mcp', 'tools'];

function jsFiles(path) {
  const st = statSync(path);
  if (st.isFile()) return path.endsWith('.js') ? [path] : [];
  return readdirSync(path).flatMap((name) => jsFiles(join(path, name)));
}

function importedPackages() {
  const seen = new Set();
  const re = /(?:from\s+|import\s*\(\s*|require\s*\(\s*)['"]([^'"./][^'"]*)['"]/g;
  for (const file of SHIPPED.flatMap((p) => jsFiles(join(pkgDir, p)))) {
    for (const [, spec] of readFileSync(file, 'utf8').matchAll(re)) {
      if (spec.startsWith('node:')) continue;
      const parts = spec.split('/');
      seen.add(spec.startsWith('@') ? parts.slice(0, 2).join('/') : parts[0]);
    }
  }
  return seen;
}

test('no ^0.x, ~0.x or wildcard dependency ranges', () => {
  const bad = Object.entries(deps).filter(([, r]) => /^[\^~]0\.|^\*$|^latest$|^x$/.test(r.trim()));
  assert.deepEqual(bad, [], `open 0.x / wildcard ranges: ${JSON.stringify(bad)}`);
});

test('every declared dependency is imported by shipped code', () => {
  const used = importedPackages();
  const unused = Object.keys(deps).filter((d) => !used.has(d));
  assert.deepEqual(unused, [], `declared but never imported: ${unused.join(', ')}`);
});

test('@modelcontextprotocol/sdk floor is past known advisories (>=1.32.0)', () => {
  const range = deps['@modelcontextprotocol/sdk'];
  assert.ok(range, '@modelcontextprotocol/sdk must be declared (mcp/server.js imports it)');
  const [maj, min] = range.replace(/^[\^~>=\s]+/, '').split('.').map(Number);
  assert.ok(maj > 1 || (maj === 1 && min >= 32), `range ${range} admits SDK < 1.32.0`);
});

test('repository metadata points at the repo the package is published from', () => {
  const repo = typeof pkg.repository === 'string' ? pkg.repository : pkg.repository?.url;
  assert.match(repo ?? '', /github\.com\/ruvnet\/sublinear-time-solver(\.git)?$/);
  assert.equal(pkg.repository?.directory, 'src/consciousness-explorer');
  assert.match(pkg.bugs?.url ?? '', /github\.com\/ruvnet\/sublinear-time-solver\/issues$/);
});
