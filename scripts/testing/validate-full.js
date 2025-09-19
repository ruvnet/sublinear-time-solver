#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

console.log('=== COMPREHENSIVE VALIDATION REPORT ===\n');

// Check file structure
console.log('1. FILE STRUCTURE CHECK:');
const requiredFiles = [
  '../../package.json',
  '../../bin/cli.js',
  '../../server/index.js',
  '../../server/streaming.js',
  '../../src/solver.js',
  '../../examples/basic-usage.js',
  '../../integrations/flow-nexus.js'
];

let fileCheckPassed = true;
requiredFiles.forEach(file => {
  const exists = fs.existsSync(file);
  console.log(`   ${exists ? '✓' : '✗'} ${file}`);
  if (!exists) fileCheckPassed = false;
});

console.log(`\nFile structure: ${fileCheckPassed ? 'PASSED ✓' : 'FAILED ✗'}\n`);

// Check package.json
console.log('2. PACKAGE.JSON VALIDATION:');
try {
  const pkg = JSON.parse(fs.readFileSync('../../package.json', 'utf8'));
  console.log(`   Name: ${pkg.name}`);
  console.log(`   Version: ${pkg.version}`);
  console.log(`   Main: ${pkg.main}`);
  console.log(`   Bin: ${JSON.stringify(pkg.bin)}`);
  console.log('   Package.json: VALID ✓\n');
} catch (e) {
  console.log(`   Package.json: INVALID ✗ - ${e.message}\n`);
}

// Check CLI executable
console.log('3. CLI EXECUTABLE CHECK:');
const { execSync } = require('child_process');
try {
  const version = execSync('node ../../bin/cli.js --version', { encoding: 'utf8' });
  console.log(`   Version: ${version.trim()}`);
  console.log('   CLI: WORKING ✓\n');
} catch (e) {
  console.log(`   CLI: FAILED ✗ - ${e.message}\n`);
}

// Test solver module
console.log('4. SOLVER MODULE TEST:');
try {
  const solverModule = require('../../src/solver.js');
  
  // Create small test system
  const A = [[4, 1], [1, 3]];
  const b = [1, 0];
  
  // Test if we can create solver instance
  if (solverModule.createSolver) {
    const solver = solverModule.createSolver();
    console.log('   Solver factory: FOUND ✓');
    
    // Try to solve
    if (solver.solve) {
      const result = solver.solve(A, b, 'jacobi');
      console.log(`   Jacobi solver: WORKING ✓ (${result.iterations} iterations)`);
    }
  } else if (solverModule.SublinearSolver) {
    const solver = new solverModule.SublinearSolver();
    console.log('   Solver class: FOUND ✓');
  } else {
    console.log('   Solver: STRUCTURE UNCLEAR ⚠');
  }
} catch (e) {
  console.log(`   Solver module: ERROR ✗ - ${e.message}`);
}

console.log('\n5. DEPENDENCIES CHECK:');
try {
  const deps = JSON.parse(fs.readFileSync('../../package.json', 'utf8')).dependencies;
  const installed = fs.existsSync('../../node_modules');
  console.log(`   Node modules: ${installed ? 'INSTALLED ✓' : 'NOT INSTALLED ✗'}`);
  console.log(`   Dependencies count: ${Object.keys(deps || {}).length}`);
  
  // Check critical deps
  const critical = ['express', 'commander', 'cors', 'ws'];
  critical.forEach(dep => {
    const hasIt = deps && deps[dep];
    console.log(`   ${hasIt ? '✓' : '✗'} ${dep}: ${hasIt || 'missing'}`);
  });
} catch (e) {
  console.log(`   Dependencies: ERROR ✗`);
}

console.log('\n6. RUST/WASM CHECK:');
try {
  const hasCargoToml = fs.existsSync('../../Cargo.toml');
  const hasSrcLib = fs.existsSync('../../src/lib.rs');
  const hasWasmIface = fs.existsSync('../../src/wasm_iface.rs');
  
  console.log(`   Cargo.toml: ${hasCargoToml ? 'EXISTS ✓' : 'MISSING ✗'}`);
  console.log(`   src/lib.rs: ${hasSrcLib ? 'EXISTS ✓' : 'MISSING ✗'}`);
  console.log(`   WASM interface: ${hasWasmIface ? 'EXISTS ✓' : 'MISSING ✗'}`);
  
  // Check if Rust is available
  try {
    execSync('rustc --version', { stdio: 'ignore' });
    console.log('   Rust compiler: AVAILABLE ✓');
  } catch {
    console.log('   Rust compiler: NOT FOUND ⚠');
  }
} catch (e) {
  console.log(`   Rust/WASM: ERROR ✗`);
}

console.log('\n=== VALIDATION COMPLETE ===\n');
