#!/usr/bin/env node
/**
 * Training Demo - Shows how to improve the LLM with training and feedback
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

    console.log('\n🎓 SUBLINEAR LLM - TRAINING & LEARNING DEMO');
    console.log('='.repeat(60));

    await init(wasmBytes);
    const llm = new SublinearLLM();

    console.log('\n📊 INITIAL STATE');
    console.log('-'.repeat(40));
    const initialStats = JSON.parse(llm.trainingStats());
    console.log('Learned triples:', initialStats.learned_triples);
    console.log('Pattern types:', initialStats.pattern_types);
    console.log('Feedback count:', initialStats.feedback_count);

    // Test initial performance
    console.log('\n🔍 TESTING INITIAL KNOWLEDGE');
    console.log('-'.repeat(40));

    const testQuery = "What are the security risks with blockchain technology?";

    const request = {
        model: "sublinear-gpt5",
        messages: [{ role: "user", content: testQuery }]
    };

    const beforeResponse = JSON.parse(llm.chatCompletions(JSON.stringify(request)));
    console.log('Query:', testQuery);
    console.log('Initial Response:', beforeResponse.choices[0].message.content);

    // Training Phase 1: Add new knowledge about blockchain security
    console.log('\n📚 TRAINING PHASE 1: Teaching Blockchain Security');
    console.log('-'.repeat(40));

    const blockchainTraining = {
        input: "What are blockchain security risks?",
        expected_output: "Blockchain faces several security risks including smart contract vulnerabilities, 51% attacks, private key theft, and consensus mechanism exploits.",
        knowledge_triples: [
            {
                subject: "blockchain",
                predicate: "vulnerable_to",
                object: "51_percent_attacks",
                confidence: 0.9,
                source: "training",
                usage_count: 0,
                success_rate: 0.0
            },
            {
                subject: "smart_contracts",
                predicate: "contain",
                object: "vulnerabilities",
                confidence: 0.85,
                source: "training",
                usage_count: 0,
                success_rate: 0.0
            },
            {
                subject: "private_keys",
                predicate: "risk_of",
                object: "theft",
                confidence: 0.95,
                source: "training",
                usage_count: 0,
                success_rate: 0.0
            },
            {
                subject: "consensus_mechanisms",
                predicate: "exploitable_by",
                object: "attackers",
                confidence: 0.8,
                source: "training",
                usage_count: 0,
                success_rate: 0.0
            }
        ],
        feedback_score: 0.9
    };

    const trainingResult1 = JSON.parse(llm.train(JSON.stringify(blockchainTraining)));
    console.log('Training Result:', trainingResult1);

    // Test after training
    const afterTraining1 = JSON.parse(llm.chatCompletions(JSON.stringify(request)));
    console.log('Post-training Response:', afterTraining1.choices[0].message.content);

    // Training Phase 2: Add web3 and DeFi knowledge
    console.log('\n📚 TRAINING PHASE 2: Teaching Web3 & DeFi');
    console.log('-'.repeat(40));

    const web3Training = [
        {
            input: "How do DeFi protocols work?",
            expected_output: "DeFi protocols use automated market makers, liquidity pools, and smart contracts to provide decentralized financial services.",
            knowledge_triples: [
                {
                    subject: "defi",
                    predicate: "uses",
                    object: "automated_market_makers",
                    confidence: 0.9,
                    source: "training",
                    usage_count: 0,
                    success_rate: 0.0
                },
                {
                    subject: "liquidity_pools",
                    predicate: "enable",
                    object: "decentralized_trading",
                    confidence: 0.85,
                    source: "training",
                    usage_count: 0,
                    success_rate: 0.0
                }
            ],
            feedback_score: 0.85
        },
        {
            input: "What are flash loan attacks?",
            expected_output: "Flash loans allow borrowing without collateral for single transactions, enabling sophisticated DeFi exploits.",
            knowledge_triples: [
                {
                    subject: "flash_loans",
                    predicate: "enable",
                    object: "arbitrage_attacks",
                    confidence: 0.9,
                    source: "training",
                    usage_count: 0,
                    success_rate: 0.0
                },
                {
                    subject: "flash_loans",
                    predicate: "require_no",
                    object: "collateral",
                    confidence: 1.0,
                    source: "training",
                    usage_count: 0,
                    success_rate: 0.0
                }
            ],
            feedback_score: 0.92
        }
    ];

    const fineTuneResult = JSON.parse(llm.fineTune("blockchain_security", JSON.stringify(web3Training)));
    console.log('Fine-tuning Result:', fineTuneResult);

    // Test with more specific queries
    console.log('\n🎯 TESTING IMPROVED KNOWLEDGE');
    console.log('-'.repeat(40));

    const advancedQueries = [
        "What are flash loan attacks?",
        "How do smart contracts get exploited?",
        "What is a 51% attack?"
    ];

    for (const query of advancedQueries) {
        const testRequest = {
            model: "sublinear-o1",
            messages: [{ role: "user", content: query }],
            reasoning_effort: "high"
        };

        const response = JSON.parse(llm.responsesAPI(JSON.stringify(testRequest)));
        console.log(`\nQ: ${query}`);
        console.log(`A: ${response.output[0].content[0].text}`);
    }

    // Feedback Learning
    console.log('\n📝 FEEDBACK LEARNING');
    console.log('-'.repeat(40));

    // Give positive feedback for good responses
    const feedbackResult1 = JSON.parse(llm.feedback(
        "What are flash loan attacks?",
        "Flash loans enable arbitrage attacks and require no collateral",
        0.95
    ));
    console.log('Positive feedback result:', feedbackResult1);

    // Give negative feedback for poor responses
    const feedbackResult2 = JSON.parse(llm.feedback(
        "What is quantum computing?",
        "I don't have enough information about quantum computing",
        0.3
    ));
    console.log('Negative feedback result:', feedbackResult2);

    // Final statistics
    console.log('\n📈 FINAL TRAINING STATISTICS');
    console.log('-'.repeat(40));

    const finalStats = JSON.parse(llm.trainingStats());
    console.log('Learned triples:', finalStats.learned_triples);
    console.log('Pattern types:', finalStats.pattern_types);
    console.log('Semantic clusters:', finalStats.semantic_clusters);
    console.log('Feedback count:', finalStats.feedback_count);
    console.log('Average confidence:', finalStats.average_confidence.toFixed(3));

    // Export knowledge
    console.log('\n💾 KNOWLEDGE EXPORT/IMPORT');
    console.log('-'.repeat(40));

    const exportedKnowledge = llm.exportKnowledge();
    console.log('Exported knowledge size:', exportedKnowledge.length, 'characters');

    // Create a new instance and import knowledge
    const llm2 = new SublinearLLM();
    const importResult = JSON.parse(llm2.importKnowledge(exportedKnowledge));
    console.log('Import result:', importResult);

    const imported_stats = JSON.parse(llm2.trainingStats());
    console.log('Imported learned triples:', imported_stats.learned_triples);

    // Test the imported knowledge
    const testImported = {
        model: "sublinear-gpt5",
        messages: [{ role: "user", content: "What are flash loan attacks?" }]
    };

    const importedResponse = JSON.parse(llm2.chatCompletions(JSON.stringify(testImported)));
    console.log('\nImported knowledge test response:');
    console.log(importedResponse.choices[0].message.content);

    console.log('\n' + '='.repeat(60));
    console.log('✨ TRAINING DEMO COMPLETE!');
    console.log('\n🎯 KEY BENEFITS OF TRAINING:');
    console.log('  • Learn new domains (blockchain security)');
    console.log('  • Improve accuracy through feedback');
    console.log('  • Build domain-specific expertise');
    console.log('  • Persistent knowledge across sessions');
    console.log('  • Fine-tune for specific use cases');
    console.log('\n🚀 The model can now answer questions about:');
    console.log('  • Blockchain security risks');
    console.log('  • DeFi protocol mechanics');
    console.log('  • Smart contract vulnerabilities');
    console.log('  • Flash loan attacks');
    console.log('\n💡 And it gets better with more training data!');
});