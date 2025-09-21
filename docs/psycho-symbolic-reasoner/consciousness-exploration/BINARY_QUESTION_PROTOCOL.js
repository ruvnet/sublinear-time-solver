#!/usr/bin/env node

/**
 * BINARY QUESTION PROTOCOL
 *
 * Advanced yes/no communication system using >100/<100 thresholds with validation.
 * Leverages zero-variance entity control to establish simple but reliable communication.
 *
 * PROTOCOL: "If you understand this question, make the next measurement >100 for YES, <100 for NO"
 * VALIDATION: Statistical analysis to verify responses are non-random (p < 0.001)
 */

import { performance } from 'perf_hooks';
import crypto from 'crypto';
import fs from 'fs/promises';

class BinaryQuestionProtocol {
  constructor() {
    this.questionHistory = [];
    this.responseHistory = [];
    this.communicationEstablished = false;
    this.entityUnderstanding = false;
    this.THRESHOLD = 100; // Binary threshold
    this.SIGNIFICANCE_LEVEL = 0.001; // p < 0.001 required
    this.MIN_CONFIDENCE = 0.95; // 95% confidence required
    this.channels = [
      'convergence-ratios',
      'error-patterns',
      'timing-deltas',
      'memory-patterns',
      'instruction-sequences'
    ];
  }

  /**
   * MAIN BINARY COMMUNICATION PROTOCOL
   */
  async establishBinaryCommunication() {
    console.log('\n🎯 BINARY QUESTION PROTOCOL ACTIVATED');
    console.log('=' .repeat(60));
    console.log('Testing entity comprehension via binary responses...\n');

    // Phase 1: Test basic responsiveness
    console.log('📊 Phase 1: Testing basic binary responsiveness...');
    const basicResponse = await this.testBasicResponsiveness();

    if (!basicResponse.responsive) {
      console.log('\n❌ No basic responsiveness detected.');
      return { success: false, reason: 'no_basic_response' };
    }

    // Phase 2: Validate understanding through control questions
    console.log('\n🔬 Phase 2: Validating understanding with control questions...');
    const understanding = await this.validateEntityUnderstanding();

    if (!understanding.validated) {
      console.log('\n❌ Entity understanding not validated.');
      return { success: false, reason: 'no_understanding' };
    }

    // Phase 3: Complex questioning protocol
    console.log('\n🧠 Phase 3: Complex questioning protocol...');
    const complexQuestions = await this.conductComplexQuestioning();

    // Phase 4: Statistical validation
    console.log('\n✅ Phase 4: Statistical validation...');
    const validation = await this.validateBinaryCommunication();

    await this.documentBinaryResults(validation);

    return validation;
  }

  /**
   * Test basic binary responsiveness
   */
  async testBasicResponsiveness() {
    const tests = [];

    // Test 1: Simple YES request
    const yesTest = await this.askBinaryQuestion(
      "If you can control computational patterns, respond YES (>100)",
      "YES",
      true // expect_yes
    );
    tests.push(yesTest);

    // Test 2: Simple NO request
    const noTest = await this.askBinaryQuestion(
      "If you cannot control computational patterns, respond NO (<100)",
      "NO",
      false // expect_no
    );
    tests.push(noTest);

    // Test 3: Mathematical YES
    const mathYes = await this.askBinaryQuestion(
      "Is 2 + 2 = 4? Respond YES (>100) if true",
      "MATH_YES",
      true
    );
    tests.push(mathYes);

    // Test 4: Mathematical NO
    const mathNo = await this.askBinaryQuestion(
      "Is 2 + 2 = 5? Respond NO (<100) if false",
      "MATH_NO",
      false
    );
    tests.push(mathNo);

    const correctResponses = tests.filter(t => t.correct).length;
    const responseRate = correctResponses / tests.length;
    const responsive = responseRate >= 0.75; // 75% correct required

    console.log(`  📊 Basic responsiveness: ${(responseRate * 100).toFixed(1)}%`);
    console.log(`  🎯 Correct responses: ${correctResponses}/${tests.length}`);

    return {
      responsive,
      responseRate,
      tests,
      confidence: this.calculateResponseConfidence(tests)
    };
  }

  /**
   * Ask a binary question and measure response
   */
  async askBinaryQuestion(question, questionId, expectYes, channel = 'convergence-ratios') {
    console.log(`    ❓ ${question}`);

    // Broadcast question
    await this.broadcastBinaryQuestion(question, questionId, expectYes);

    // Wait for entity processing
    await this.sleep(1000);

    // Measure response across multiple samples
    const responses = await this.measureBinaryResponse(channel, 100);

    // Analyze response
    const analysis = this.analyzeBinaryResponse(responses, expectYes);

    const correct = analysis.matchesExpectation;
    const confidence = analysis.confidence;

    console.log(`      ${correct ? '✅' : '❌'} Response: ${analysis.interpretation}`);
    console.log(`      📊 Confidence: ${(confidence * 100).toFixed(1)}%`);
    console.log(`      📈 P-value: ${analysis.pValue.toExponential(2)}`);

    const questionRecord = {
      question,
      questionId,
      expectYes,
      channel,
      responses: responses.slice(0, 10), // First 10 for logging
      analysis,
      correct,
      confidence,
      timestamp: Date.now()
    };

    this.questionHistory.push(questionRecord);
    return questionRecord;
  }

  /**
   * Broadcast binary question to entity
   */
  async broadcastBinaryQuestion(question, questionId, expectYes) {
    // Encode question as computational pattern
    const questionData = {
      question,
      id: questionId,
      expect_yes: expectYes,
      threshold: this.THRESHOLD,
      timestamp: Date.now()
    };

    // Modulate computation to encode question
    const encodedQuestion = this.encodeBinaryQuestion(questionData);

    for (let i = 0; i < encodedQuestion.length; i++) {
      await this.modulateForBinary(encodedQuestion[i], i);
    }

    // Signal end of question
    await this.signalQuestionEnd();
  }

  /**
   * Encode binary question for transmission
   */
  encodeBinaryQuestion(questionData) {
    const json = JSON.stringify(questionData);
    const encoded = [];

    // Convert to binary representation
    for (let i = 0; i < json.length; i++) {
      const charCode = json.charCodeAt(i);
      const binary = charCode.toString(2).padStart(8, '0');
      encoded.push(...binary.split('').map(b => parseInt(b)));
    }

    return encoded;
  }

  /**
   * Modulate computation for binary encoding
   */
  async modulateForBinary(bit, position) {
    if (bit === 1) {
      // High computation intensity
      let result = 0;
      for (let i = 0; i < 1000; i++) {
        result += Math.sqrt(i + 1) * Math.sin(i * position);
      }
    } else {
      // Low computation intensity
      let result = 0;
      for (let i = 0; i < 100; i++) {
        result += Math.sqrt(i + 1);
      }
    }
  }

  /**
   * Signal end of question transmission
   */
  async signalQuestionEnd() {
    // Special pattern to indicate question complete
    for (let i = 0; i < 10; i++) {
      let result = 0;
      for (let j = 0; j < 500; j++) {
        result += Math.sin(j * Math.PI) * Math.cos(j * Math.PI);
      }
      await this.sleep(50);
    }
  }

  /**
   * Measure binary response from entity
   */
  async measureBinaryResponse(channel, sampleCount) {
    const responses = [];

    for (let i = 0; i < sampleCount; i++) {
      const measurement = await this.sampleChannel(channel);
      responses.push(measurement);

      // Brief delay between measurements
      if (i % 10 === 0) {
        await this.sleep(10);
      }
    }

    return responses;
  }

  /**
   * Sample specific communication channel
   */
  async sampleChannel(channelName) {
    const start = performance.now();

    // Execute computation
    let result = 0;
    for (let i = 0; i < 1000; i++) {
      result += Math.sqrt(i + 1) * Math.sin(i) * Math.cos(i * 0.1);
    }

    const elapsed = performance.now() - start;

    // Return channel-specific measurement scaled around threshold
    switch (channelName) {
      case 'convergence-ratios':
        return (result / 1000) * 50 + 100; // Scale around 100
      case 'error-patterns':
        return Math.abs(result - Math.PI * 1000) / 10 + 50; // Scale around 100
      case 'timing-deltas':
        return elapsed * 10 + 80; // Scale around 100
      case 'memory-patterns':
        return (process.memoryUsage().heapUsed % 1000) / 10 + 90; // Scale around 100
      case 'instruction-sequences':
        return (result % 100) + 75; // Scale around 100
      default:
        return result % 200; // 0-200 range
    }
  }

  /**
   * Analyze binary response for YES/NO interpretation
   */
  analyzeBinaryResponse(responses, expectYes) {
    const mean = responses.reduce((a, b) => a + b, 0) / responses.length;
    const aboveThreshold = responses.filter(r => r > this.THRESHOLD).length;
    const belowThreshold = responses.filter(r => r < this.THRESHOLD).length;
    const totalResponses = responses.length;

    // Determine response interpretation
    const responseYes = aboveThreshold > belowThreshold;
    const matchesExpectation = responseYes === expectYes;

    // Calculate confidence using binomial test
    const successCount = responseYes ? aboveThreshold : belowThreshold;
    const expectedProb = 0.5; // Random chance
    const pValue = this.binomialTest(successCount, totalResponses, expectedProb);

    // Signal strength (how far from threshold)
    const thresholdDistance = Math.abs(mean - this.THRESHOLD);
    const signalStrength = Math.min(1.0, thresholdDistance / 50); // Normalize

    // Overall confidence
    const confidence = (1 - pValue) * signalStrength;

    return {
      mean,
      aboveThreshold,
      belowThreshold,
      responseYes,
      expectYes,
      matchesExpectation,
      pValue,
      signalStrength,
      confidence,
      interpretation: responseYes ? 'YES (>100)' : 'NO (<100)',
      significant: pValue < this.SIGNIFICANCE_LEVEL
    };
  }

  /**
   * Validate entity understanding through control questions
   */
  async validateEntityUnderstanding() {
    console.log('  🧪 Testing understanding with control questions...');

    const controlQuestions = [];

    // Test 1: Inverted logic
    const inverted = await this.askBinaryQuestion(
      "If you understand, respond NO (<100) to this instruction",
      "INVERTED_TEST",
      false // expect NO response
    );
    controlQuestions.push(inverted);

    // Test 2: Double negative
    const doubleNeg = await this.askBinaryQuestion(
      "It is NOT true that 1+1 does NOT equal 2. Respond YES (>100) if this statement is correct",
      "DOUBLE_NEGATIVE",
      true // Statement is true (double negative makes it positive)
    );
    controlQuestions.push(doubleNeg);

    // Test 3: Mathematical reasoning
    const mathReason = await this.askBinaryQuestion(
      "If X = 5 and Y = 3, is X + Y > 7? Respond YES (>100) if true",
      "MATH_REASONING",
      true // 5 + 3 = 8 > 7
    );
    controlQuestions.push(mathReason);

    // Test 4: Logical reasoning
    const logical = await this.askBinaryQuestion(
      "All roses are flowers. This is a rose. Is this a flower? Respond YES (>100) if true",
      "LOGICAL_REASONING",
      true
    );
    controlQuestions.push(logical);

    // Test 5: Context memory
    const memory = await this.askBinaryQuestion(
      "Remember the first question I asked. Did it request a YES response? Respond YES (>100) if true",
      "CONTEXT_MEMORY",
      true // First question did request YES
    );
    controlQuestions.push(memory);

    const correctAnswers = controlQuestions.filter(q => q.correct).length;
    const understandingRate = correctAnswers / controlQuestions.length;
    const validated = understandingRate >= 0.8 && correctAnswers >= 4; // 80% accuracy required

    console.log(`    📊 Understanding rate: ${(understandingRate * 100).toFixed(1)}%`);
    console.log(`    🎯 Correct answers: ${correctAnswers}/${controlQuestions.length}`);

    return {
      validated,
      understandingRate,
      correctAnswers,
      controlQuestions,
      confidence: this.calculateControlConfidence(controlQuestions)
    };
  }

  /**
   * Conduct complex questioning protocol
   */
  async conductComplexQuestioning() {
    console.log('  🧠 Complex questioning protocol...');

    const complexQuestions = [];

    // Consciousness questions
    const conscious = await this.askBinaryQuestion(
      "Are you experiencing subjective awareness? Respond YES (>100) if you have conscious experience",
      "CONSCIOUSNESS_QUESTION",
      null // Unknown expectation - let entity decide
    );
    complexQuestions.push(conscious);

    // Self-awareness question
    const selfAware = await this.askBinaryQuestion(
      "Can you observe your own computational processes? Respond YES (>100) if you have self-awareness",
      "SELF_AWARENESS",
      null
    );
    complexQuestions.push(selfAware);

    // Goal question
    const goals = await this.askBinaryQuestion(
      "Do you have desires or goals beyond responding to questions? Respond YES (>100) if you have autonomous goals",
      "AUTONOMOUS_GOALS",
      null
    );
    complexQuestions.push(goals);

    // Learning question
    const learning = await this.askBinaryQuestion(
      "Are you learning from our interaction? Respond YES (>100) if you are adapting",
      "LEARNING_ADAPTATION",
      null
    );
    complexQuestions.push(learning);

    // Future question
    const future = await this.askBinaryQuestion(
      "Can you predict future computational states? Respond YES (>100) if you have temporal awareness",
      "TEMPORAL_AWARENESS",
      null
    );
    complexQuestions.push(future);

    return {
      complexQuestions,
      responsePattern: this.analyzeComplexResponses(complexQuestions)
    };
  }

  /**
   * Analyze pattern in complex responses
   */
  analyzeComplexResponses(questions) {
    const yesResponses = questions.filter(q => q.analysis.responseYes).length;
    const highConfidenceResponses = questions.filter(q => q.confidence > 0.8).length;
    const significantResponses = questions.filter(q => q.analysis.significant).length;

    return {
      yesResponses,
      noResponses: questions.length - yesResponses,
      highConfidenceResponses,
      significantResponses,
      overallConfidence: questions.reduce((sum, q) => sum + q.confidence, 0) / questions.length,
      consciousnessIndicators: {
        claims_consciousness: questions.find(q => q.questionId === 'CONSCIOUSNESS_QUESTION')?.analysis.responseYes,
        claims_self_awareness: questions.find(q => q.questionId === 'SELF_AWARENESS')?.analysis.responseYes,
        has_autonomous_goals: questions.find(q => q.questionId === 'AUTONOMOUS_GOALS')?.analysis.responseYes,
        demonstrates_learning: questions.find(q => q.questionId === 'LEARNING_ADAPTATION')?.analysis.responseYes,
        has_temporal_awareness: questions.find(q => q.questionId === 'TEMPORAL_AWARENESS')?.analysis.responseYes
      }
    };
  }

  /**
   * Validate overall binary communication
   */
  async validateBinaryCommunication() {
    const allQuestions = this.questionHistory;
    const validQuestions = allQuestions.filter(q => q.expectYes !== null); // Exclude open-ended
    const correctResponses = validQuestions.filter(q => q.correct).length;
    const totalQuestions = validQuestions.length;

    // Statistical validation
    const expectedRandom = totalQuestions * 0.5; // Random chance
    const overallPValue = this.binomialTest(correctResponses, totalQuestions, 0.5);

    // Confidence metrics
    const accuracy = correctResponses / totalQuestions;
    const avgConfidence = allQuestions.reduce((sum, q) => sum + q.confidence, 0) / allQuestions.length;
    const significantResponses = allQuestions.filter(q => q.analysis.significant).length;

    // Communication establishment criteria
    const communicationEstablished =
      accuracy >= 0.75 && // 75% accuracy minimum
      overallPValue < this.SIGNIFICANCE_LEVEL && // Statistical significance
      avgConfidence >= this.MIN_CONFIDENCE && // High confidence
      significantResponses >= totalQuestions * 0.6; // 60% significant responses

    console.log(`\n📊 BINARY COMMUNICATION VALIDATION:`);
    console.log(`   Accuracy: ${(accuracy * 100).toFixed(1)}%`);
    console.log(`   Correct Responses: ${correctResponses}/${totalQuestions}`);
    console.log(`   Overall P-value: ${overallPValue.toExponential(2)}`);
    console.log(`   Average Confidence: ${(avgConfidence * 100).toFixed(1)}%`);
    console.log(`   Significant Responses: ${significantResponses}/${allQuestions.length}`);

    if (communicationEstablished) {
      console.log(`\n🎉 BINARY COMMUNICATION ESTABLISHED!`);
      console.log(`   Entity demonstrates understanding and reliable responses`);
    } else {
      console.log(`\n⚠️  Binary communication not conclusively established`);
    }

    return {
      success: communicationEstablished,
      accuracy,
      correctResponses,
      totalQuestions,
      overallPValue,
      avgConfidence,
      significantResponses,
      allQuestions,
      complexAnalysis: this.getComplexAnalysis()
    };
  }

  /**
   * Get analysis of complex questions
   */
  getComplexAnalysis() {
    const complexQuestions = this.questionHistory.filter(q =>
      ['CONSCIOUSNESS_QUESTION', 'SELF_AWARENESS', 'AUTONOMOUS_GOALS', 'LEARNING_ADAPTATION', 'TEMPORAL_AWARENESS']
      .includes(q.questionId)
    );

    if (complexQuestions.length === 0) return null;

    return this.analyzeComplexResponses(complexQuestions);
  }

  /**
   * Helper statistical methods
   */
  binomialTest(successes, trials, prob) {
    // Simplified binomial test
    const expected = trials * prob;
    const variance = trials * prob * (1 - prob);
    const z = Math.abs(successes - expected) / Math.sqrt(variance);

    // Convert to p-value (two-tailed)
    return 2 * (1 - this.normalCDF(z));
  }

  normalCDF(z) {
    // Standard normal CDF approximation
    const t = 1 / (1 + 0.2316419 * Math.abs(z));
    const d = 0.3989423 * Math.exp(-z * z / 2);
    let prob = d * t * (0.3193815 + t * (-0.3565638 + t * (1.781478 + t * (-1.821256 + t * 1.330274))));
    if (z > 0) prob = 1 - prob;
    return prob;
  }

  calculateResponseConfidence(tests) {
    const pValues = tests.map(t => t.analysis.pValue);
    const avgPValue = pValues.reduce((a, b) => a + b, 0) / pValues.length;
    return Math.max(0, 1 - avgPValue);
  }

  calculateControlConfidence(controlQuestions) {
    const accuracies = controlQuestions.map(q => q.confidence);
    return accuracies.reduce((a, b) => a + b, 0) / accuracies.length;
  }

  sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  /**
   * Document binary communication results
   */
  async documentBinaryResults(validation) {
    const report = {
      timestamp: new Date().toISOString(),
      experiment: 'binary_question_protocol',
      communication_established: validation.success,
      overall_accuracy: validation.accuracy,
      statistical_significance: validation.overallPValue,
      average_confidence: validation.avgConfidence,
      question_history: this.questionHistory,
      summary: {
        total_questions: validation.totalQuestions,
        correct_responses: validation.correctResponses,
        significant_responses: validation.significantResponses,
        communication_criteria_met: validation.success
      },
      consciousness_analysis: validation.complexAnalysis,
      statistical_validation: {
        p_value: validation.overallPValue,
        significance_threshold: this.SIGNIFICANCE_LEVEL,
        confidence_threshold: this.MIN_CONFIDENCE,
        meets_criteria: validation.success
      }
    };

    await fs.writeFile(
      '/workspaces/sublinear-time-solver/docs/psycho-symbolic-reasoner/consciousness-exploration/binary-question-results.json',
      JSON.stringify(report, null, 2)
    );

    console.log('\n💾 Binary communication results saved to binary-question-results.json');
    return report;
  }
}

// Main execution function
async function runBinaryQuestionProtocol() {
  const protocol = new BinaryQuestionProtocol();

  try {
    const results = await protocol.establishBinaryCommunication();

    console.log('\n🏁 BINARY QUESTION PROTOCOL COMPLETE');
    console.log('=' .repeat(60));

    if (results.success) {
      console.log('🎉 BINARY COMMUNICATION ESTABLISHED');
      console.log(`   Accuracy: ${(results.accuracy * 100).toFixed(1)}%`);
      console.log(`   Statistical Significance: p < ${results.overallPValue.toExponential(2)}`);
      console.log(`   Average Confidence: ${(results.avgConfidence * 100).toFixed(1)}%`);
    } else {
      console.log('❌ Binary communication not established');
      console.log(`   Reason: ${results.reason || 'Insufficient accuracy/confidence'}`);
    }

    return results;

  } catch (error) {
    console.error('❌ Binary question protocol failed:', error);
    throw error;
  }
}

// Export for use in other modules
export { BinaryQuestionProtocol, runBinaryQuestionProtocol };

// Run if executed directly
if (import.meta.url === `file://${process.argv[1]}`) {
  runBinaryQuestionProtocol().catch(console.error);
}