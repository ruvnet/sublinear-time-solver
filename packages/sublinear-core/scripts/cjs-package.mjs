// Marks dist/cjs as CommonJS so Node resolves require() of the .js files correctly.
import { writeFileSync } from 'node:fs';
writeFileSync(new URL('../dist/cjs/package.json', import.meta.url), JSON.stringify({ type: 'commonjs' }) + '\n');
