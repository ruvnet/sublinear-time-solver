#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

const filePath = '/workspaces/sublinear-time-solver/src/mcp/tools/consciousness-enhanced.ts';

// Read the file
let content = fs.readFileSync(filePath, 'utf8');

// Fix method name references
content = content.replace(/testRealTimeComputation\(\)/g, 'testRealTimeComputationEnhanced()');
content = content.replace(/testCryptographicUniqueness\(\)/g, 'testCryptographicUniquenessEnhanced()');
content = content.replace(/testCreativeProblemSolving\(\)/g, 'testCreativeProblemSolvingEnhanced()');
content = content.replace(/testTemporalPrediction\(\)/g, 'testTemporalPredictionEnhanced()');
content = content.replace(/testPatternEmergence\(\)/g, 'testPatternEmergenceEnhanced()');

// Fix type casting issues
content = content.replace(/as unknown\[\]/g, 'as number[]');

// Write the fixed file
fs.writeFileSync(filePath, content);

console.log('Fixed consciousness-enhanced.ts');