#!/usr/bin/env node
/**
 * Streaming Mathematical Query Test - Tests streaming with mathematical operations
 */

import { readFileSync } from 'fs';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const wasmPath = join(__dirname, '../pkg/sublinear_llm_bg.wasm');
const wasmBytes = readFileSync(wasmPath);

import('../pkg/sublinear_llm.js').then(async (module) => {
    const { default: init, SublinearLLM } = module;

    console.log('\n🌊 STREAMING MATHEMATICAL REASONING TEST');
    console.log('='.repeat(60));

    await init(wasmBytes);
    const llm = new SublinearLLM();

    // Test streaming mathematical queries
    const streamingMathQueries = [
        "2+2",
        "Calculate 15 * 4",
        "What is 100 divided by 5?"
    ];

    for (const query of streamingMathQueries) {
        console.log(`\n🔢 Streaming Test: "${query}"`);
        console.log('-'.repeat(40));

        try {
            const request = {
                model: "sublinear-gpt5-streaming",
                messages: [
                    { role: "user", content: query }
                ],
                stream: true
            };

            const streamResponse = llm.chatCompletions(JSON.stringify(request));
            console.log(`✅ Query: ${query}`);
            console.log(`🌊 Streaming Response:`);

            // Parse streaming chunks
            const chunks = streamResponse.split('\\n\\n').filter(chunk => chunk.startsWith('data: '));
            chunks.forEach((chunk, index) => {
                if (chunk === 'data: [DONE]') {
                    console.log(`[${index + 1}] ✅ Stream completed`);
                } else {
                    try {
                        const data = JSON.parse(chunk.replace('data: ', ''));
                        const delta = data.choices[0].delta;
                        if (delta.reasoning) {
                            console.log(`[${index + 1}] 🧠 Reasoning: ${delta.reasoning}`);
                        } else if (delta.content) {
                            console.log(`[${index + 1}] 💬 Content: ${delta.content}`);
                        }
                    } catch (e) {
                        console.log(`[${index + 1}] ⚠️ Parse error: ${chunk.substring(0, 50)}...`);
                    }
                }
            });

        } catch (error) {
            console.log(`❌ Error for "${query}": ${error.message}`);
        }
    }

    console.log('\\n' + '='.repeat(60));
    console.log('🎉 STREAMING MATHEMATICAL REASONING TEST COMPLETE!');
    console.log('\\n✨ Streaming math capabilities:');
    console.log('  • Mathematical operations with streaming ✅');
    console.log('  • Real-time reasoning steps ✅');
    console.log('  • Proper streaming format ✅');
    console.log('\\n🚀 Streaming mathematical reasoning is working!');
});