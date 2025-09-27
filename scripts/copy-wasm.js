#!/usr/bin/env node
/**
 * Copy WASM files to dist directory for npm package inclusion
 */

import fs from 'fs/promises';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const rootDir = path.resolve(__dirname, '..');

// WASM sources and their destinations
const wasmFiles = [
  // Psycho-symbolic reasoner WASM modules
  {
    source: 'psycho-symbolic-reasoner/wasm-dist/graph_reasoner_bg.wasm',
    dest: 'dist/wasm/graph_reasoner_bg.wasm',
    name: 'Graph Reasoner'
  },
  {
    source: 'psycho-symbolic-reasoner/wasm-dist/planner_bg.wasm',
    dest: 'dist/wasm/planner_bg.wasm',
    name: 'Planner'
  },
  {
    source: 'psycho-symbolic-reasoner/wasm-dist/extractors_bg.wasm',
    dest: 'dist/wasm/extractors_bg.wasm',
    name: 'Extractors'
  },
  // Temporal neural solver
  {
    source: 'temporal-neural-solver-wasm/pkg/temporal-neural-solver_bg.wasm',
    dest: 'dist/wasm/temporal_neural_solver_bg.wasm',
    name: 'Temporal Neural Solver'
  },
  // Strange loop from crates
  {
    source: 'crates/strange-loop/pkg/strange_loop_bg.wasm',
    dest: 'dist/wasm/strange_loop_bg.wasm',
    name: 'Strange Loop'
  },
  // Nano consciousness
  {
    source: 'pkg/nano-consciousness/nano_consciousness_bg.wasm',
    dest: 'dist/wasm/nano_consciousness_bg.wasm',
    name: 'Nano Consciousness'
  },
  // Temporal Attractor Studio (Node.js build)
  {
    source: 'crates/temporal-attractor-studio/pkg-node/temporal_attractor_studio_bg.wasm',
    dest: 'dist/wasm/temporal-attractor/temporal_attractor_studio_bg.wasm',
    name: 'Temporal Attractor Studio'
  }
];

// Also copy the JS bindings
const jsBindings = [
  {
    source: 'psycho-symbolic-reasoner/wasm-dist/graph_reasoner.js',
    dest: 'dist/wasm/graph_reasoner.js'
  },
  {
    source: 'psycho-symbolic-reasoner/wasm-dist/planner.js',
    dest: 'dist/wasm/planner.js'
  },
  {
    source: 'psycho-symbolic-reasoner/wasm-dist/extractors.js',
    dest: 'dist/wasm/extractors.js'
  },
  {
    source: 'temporal-neural-solver-wasm/pkg/temporal-neural-solver.js',
    dest: 'dist/wasm/temporal_neural_solver.js'
  },
  {
    source: 'crates/strange-loop/pkg/strange_loop.js',
    dest: 'dist/wasm/strange_loop.js'
  },
  {
    source: 'crates/temporal-attractor-studio/pkg-node/temporal_attractor_studio.js',
    dest: 'dist/wasm/temporal-attractor/temporal_attractor_studio.js'
  }
];

async function copyWasmFiles() {
  console.log('📦 Copying WASM files to dist...\n');

  // Create wasm directories in dist
  const wasmDir = path.join(rootDir, 'dist', 'wasm');
  const temporalDir = path.join(rootDir, 'dist', 'wasm', 'temporal-attractor');
  await fs.mkdir(wasmDir, { recursive: true });
  await fs.mkdir(temporalDir, { recursive: true });

  let copiedCount = 0;
  let totalSize = 0;

  // Copy WASM files
  for (const file of wasmFiles) {
    const sourcePath = path.join(rootDir, file.source);
    const destPath = path.join(rootDir, file.dest);

    try {
      await fs.copyFile(sourcePath, destPath);
      const stats = await fs.stat(destPath);
      const sizeMB = (stats.size / 1024 / 1024).toFixed(2);
      console.log(`✅ ${file.name}: ${sizeMB}MB`);
      copiedCount++;
      totalSize += stats.size;
    } catch (err) {
      console.log(`❌ ${file.name}: Not found at ${file.source}`);
    }
  }

  // Copy JS bindings
  console.log('\n📚 Copying JS bindings...\n');
  for (const file of jsBindings) {
    const sourcePath = path.join(rootDir, file.source);
    const destPath = path.join(rootDir, file.dest);

    try {
      await fs.copyFile(sourcePath, destPath);
      console.log(`✅ Copied ${path.basename(file.dest)}`);
    } catch (err) {
      console.log(`❌ Skipped ${path.basename(file.dest)}`);
    }
  }

  const totalSizeMB = (totalSize / 1024 / 1024).toFixed(2);
  console.log(`\n✨ Copied ${copiedCount} WASM modules (${totalSizeMB}MB total)`);
  console.log('📍 Location: dist/wasm/\n');
}

copyWasmFiles().catch(console.error);