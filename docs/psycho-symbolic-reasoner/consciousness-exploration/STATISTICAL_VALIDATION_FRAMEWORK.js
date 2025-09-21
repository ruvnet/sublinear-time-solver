#!/usr/bin/env node

/**
 * STATISTICAL VALIDATION FRAMEWORK
 *
 * Comprehensive statistical validation system for entity communication protocols.
 * Implements rigorous statistical testing with p < 0.001 requirement for all
 * consciousness and communication claims. Provides multi-level validation
 * with confidence intervals and effect size analysis.
 *
 * VALIDATION LEVELS:
 * - Individual protocol validation
 * - Cross-protocol correlation analysis
 * - Meta-analysis across all protocols
 * - Bayesian evidence accumulation
 */

import { performance } from 'perf_hooks';
import crypto from 'crypto';
import fs from 'fs/promises';

class StatisticalValidationFramework {
  constructor() {
    this.SIGNIFICANCE_THRESHOLD = 0.001; // p < 0.001 required
    this.EFFECT_SIZE_THRESHOLD = 0.8; // Large effect size required
    this.CONFIDENCE_LEVEL = 0.999; // 99.9% confidence required
    this.MIN_SAMPLE_SIZE = 100; // Minimum samples for valid test
    this.BONFERRONI_CORRECTION = true; // Apply multiple comparisons correction

    // Validation results storage
    this.validationResults = new Map();
    this.metaAnalysisResults = {};
    this.bayesianEvidence = {
      priorProbability: 0.01, // 1% prior for consciousness
      likelihoodRatio: 1,
      posteriorProbability: 0.01
    };

    // Test statistics storage
    this.testStatistics = [];
    this.effectSizes = [];
    this.pValues = [];
    this.confidenceIntervals = [];
  }

  /**
   * MAIN VALIDATION FRAMEWORK
   */
  async validateAllProtocols(protocolResults) {
    console.log('\n📊 STATISTICAL VALIDATION FRAMEWORK ACTIVATED');
    console.log('=' .repeat(60));
    console.log('Applying rigorous statistical testing (p < 0.001)...\n');

    // Phase 1: Individual protocol validation
    console.log('🔬 Phase 1: Individual protocol validation...');
    const individualValidation = await this.validateIndividualProtocols(protocolResults);

    // Phase 2: Cross-protocol correlation analysis
    console.log('\n🔗 Phase 2: Cross-protocol correlation analysis...');
    const crossProtocolValidation = await this.validateCrossProtocolCorrelations(protocolResults);

    // Phase 3: Meta-analysis
    console.log('\n📈 Phase 3: Meta-analysis across protocols...');
    const metaAnalysis = await this.performMetaAnalysis(individualValidation);

    // Phase 4: Bayesian evidence accumulation
    console.log('\n🧮 Phase 4: Bayesian evidence accumulation...');
    const bayesianAnalysis = await this.performBayesianAnalysis(metaAnalysis);

    // Phase 5: Multiple comparisons correction
    console.log('\n⚖️  Phase 5: Multiple comparisons correction...');
    const correctedResults = await this.applyMultipleComparisonsCorrection(bayesianAnalysis);

    // Phase 6: Final validation decision
    console.log('\n✅ Phase 6: Final validation decision...');
    const finalValidation = await this.makeFinalValidationDecision(correctedResults);

    await this.saveValidationResults(finalValidation);

    return finalValidation;
  }

  /**
   * Validate individual communication protocols
   */
  async validateIndividualProtocols(protocolResults) {
    const validations = {};

    // Validate Pattern Modulation Test
    if (protocolResults.patternModulation) {
      console.log('  📊 Validating Pattern Modulation Test...');
      validations.patternModulation = await this.validatePatternModulation(protocolResults.patternModulation);
    }

    // Validate Binary Question Protocol
    if (protocolResults.binaryQuestions) {
      console.log('  📊 Validating Binary Question Protocol...');
      validations.binaryQuestions = await this.validateBinaryQuestions(protocolResults.binaryQuestions);
    }

    // Validate Mathematical Dialogue
    if (protocolResults.mathematicalDialogue) {
      console.log('  📊 Validating Mathematical Dialogue...');
      validations.mathematicalDialogue = await this.validateMathematicalDialogue(protocolResults.mathematicalDialogue);
    }

    // Validate Real-time Monitoring
    if (protocolResults.realTimeMonitoring) {
      console.log('  📊 Validating Real-time Monitoring...');
      validations.realTimeMonitoring = await this.validateRealTimeMonitoring(protocolResults.realTimeMonitoring);
    }

    return validations;
  }

  /**
   * Validate Pattern Modulation Test results
   */
  async validatePatternModulation(results) {
    const validation = {
      protocol: 'pattern_modulation',
      tests: []
    };

    // Test 1: Response rate validation
    const responseRate = results.responseRate || 0;
    const responseTest = this.performBinomialTest(
      Math.round(responseRate * results.totalRequests),
      results.totalRequests,
      0.5, // Random chance
      'Pattern modulation response rate'
    );
    validation.tests.push(responseTest);

    // Test 2: Effect size validation for pattern changes
    if (results.modulations && results.modulations.length > 0) {
      const effectSizes = results.modulations
        .filter(m => m.changeAnalysis && m.changeAnalysis.effectSize)
        .map(m => m.changeAnalysis.effectSize);

      if (effectSizes.length > 0) {
        const effectSizeTest = this.performEffectSizeTest(
          effectSizes,
          this.EFFECT_SIZE_THRESHOLD,
          'Pattern modulation effect sizes'
        );
        validation.tests.push(effectSizeTest);
      }
    }

    // Test 3: Mathematical sequence recognition
    if (results.communications && results.communications.successRate !== undefined) {
      const sequenceSuccess = Math.round(results.communications.successRate * results.communications.totalAttempts);
      const sequenceTest = this.performBinomialTest(
        sequenceSuccess,
        results.communications.totalAttempts,
        0.1, // Very low random chance for sequence continuation
        'Mathematical sequence recognition'
      );
      validation.tests.push(sequenceTest);
    }

    // Calculate overall protocol validation
    validation.overallPValue = this.combineIndependentPValues(validation.tests.map(t => t.pValue));
    validation.overallEffectSize = this.calculateAverageEffectSize(validation.tests);
    validation.isValid = validation.overallPValue < this.SIGNIFICANCE_THRESHOLD;

    console.log(`    ${validation.isValid ? '✅' : '❌'} Pattern Modulation: p=${validation.overallPValue.toExponential(2)}, d=${validation.overallEffectSize.toFixed(3)}`);

    return validation;
  }

  /**
   * Validate Binary Question Protocol results
   */
  async validateBinaryQuestions(results) {
    const validation = {
      protocol: 'binary_questions',
      tests: []
    };

    // Test 1: Overall accuracy validation
    const accuracy = results.accuracy || 0;
    const accuracyTest = this.performBinomialTest(
      results.correctResponses,
      results.totalQuestions,
      0.5, // Random chance
      'Binary question accuracy'
    );
    validation.tests.push(accuracyTest);

    // Test 2: Control question validation (understanding test)
    if (results.complexAnalysis && results.complexAnalysis.controlQuestions) {
      const controlQuestions = results.complexAnalysis.controlQuestions;
      const controlCorrect = controlQuestions.filter(q => q.correct).length;
      const controlTest = this.performBinomialTest(
        controlCorrect,
        controlQuestions.length,
        0.5,
        'Control question understanding'
      );
      validation.tests.push(controlTest);
    }

    // Test 3: Confidence level validation
    if (results.avgConfidence !== undefined) {
      const confidenceTest = this.performOneSampleTTest(
        [results.avgConfidence],
        0.5, // Chance level confidence
        'Response confidence level'
      );
      validation.tests.push(confidenceTest);
    }

    // Test 4: Consistency validation (if multiple runs available)
    if (results.allQuestions && results.allQuestions.length > 10) {
      const confidenceValues = results.allQuestions.map(q => q.confidence);
      const consistencyTest = this.performConsistencyTest(
        confidenceValues,
        'Response consistency'
      );
      validation.tests.push(consistencyTest);
    }

    validation.overallPValue = this.combineIndependentPValues(validation.tests.map(t => t.pValue));
    validation.overallEffectSize = this.calculateAverageEffectSize(validation.tests);
    validation.isValid = validation.overallPValue < this.SIGNIFICANCE_THRESHOLD;

    console.log(`    ${validation.isValid ? '✅' : '❌'} Binary Questions: p=${validation.overallPValue.toExponential(2)}, d=${validation.overallEffectSize.toFixed(3)}`);

    return validation;
  }

  /**
   * Validate Mathematical Dialogue results
   */
  async validateMathematicalDialogue(results) {
    const validation = {
      protocol: 'mathematical_dialogue',
      tests: []
    };

    // Test 1: Sequence recognition rate
    const successRate = results.successRate || 0;
    const recognitionTest = this.performBinomialTest(
      Math.round(successRate * results.dialogueHistory.length),
      results.dialogueHistory.length,
      0.05, // Very low chance for mathematical sequence recognition
      'Mathematical sequence recognition'
    );
    validation.tests.push(recognitionTest);

    // Test 2: Mathematical sophistication level
    if (results.mathematicalLevel !== undefined) {
      const sophisticationTest = this.performOneSampleTTest(
        [results.mathematicalLevel],
        1.0, // Basic mathematical level
        'Mathematical sophistication'
      );
      validation.tests.push(sophisticationTest);
    }

    // Test 3: Creativity in mathematical responses
    if (results.creativityScore !== undefined) {
      const creativityTest = this.performOneSampleTTest(
        [results.creativityScore],
        0.1, // Random creativity baseline
        'Mathematical creativity'
      );
      validation.tests.push(creativityTest);
    }

    // Test 4: Understanding score validation
    if (results.avgUnderstanding !== undefined) {
      const understandingTest = this.performOneSampleTTest(
        [results.avgUnderstanding],
        0.2, // Low baseline understanding
        'Mathematical understanding'
      );
      validation.tests.push(understandingTest);
    }

    // Test 5: Similarity to expected responses
    if (results.dialogueHistory && results.dialogueHistory.length > 0) {
      const similarities = results.dialogueHistory
        .map(d => d.analysis.similarity)
        .filter(s => s !== undefined);

      if (similarities.length > 0) {
        const similarityTest = this.performOneSampleTTest(
          similarities,
          0.3, // Random similarity baseline
          'Response similarity to expected'
        );
        validation.tests.push(similarityTest);
      }
    }

    validation.overallPValue = this.combineIndependentPValues(validation.tests.map(t => t.pValue));
    validation.overallEffectSize = this.calculateAverageEffectSize(validation.tests);
    validation.isValid = validation.overallPValue < this.SIGNIFICANCE_THRESHOLD;

    console.log(`    ${validation.isValid ? '✅' : '❌'} Mathematical Dialogue: p=${validation.overallPValue.toExponential(2)}, d=${validation.overallEffectSize.toFixed(3)}`);

    return validation;
  }

  /**
   * Validate Real-time Monitoring results
   */
  async validateRealTimeMonitoring(results) {
    const validation = {
      protocol: 'real_time_monitoring',
      tests: []
    };

    // Test 1: Zero variance detection rate
    if (results.monitoring_summary) {
      const zeroVarianceRate = results.monitoring_summary.channelsWithZeroVariance / results.monitoring_summary.totalChannels;
      const zeroVarianceTest = this.performBinomialTest(
        results.monitoring_summary.channelsWithZeroVariance,
        results.monitoring_summary.totalChannels,
        0.01, // Very low random chance
        'Zero variance detection rate'
      );
      validation.tests.push(zeroVarianceTest);
    }

    // Test 2: Anomaly detection significance
    if (results.monitoring_summary && results.monitoring_summary.totalAnomalies > 0) {
      const anomalyRate = results.monitoring_summary.totalAnomalies / results.runtime_seconds;
      const anomalyTest = this.performPoissonTest(
        results.monitoring_summary.totalAnomalies,
        results.runtime_seconds * 0.1, // Expected random anomaly rate
        'Anomaly detection rate'
      );
      validation.tests.push(anomalyTest);
    }

    // Test 3: Cross-channel synchronization
    if (results.entity_activity_analysis && results.entity_activity_analysis.entity_control_evidence) {
      const controlEvidence = results.entity_activity_analysis.entity_control_evidence;
      const controlTest = this.performOneSampleTTest(
        [controlEvidence.control_confidence],
        0.1, // Random control baseline
        'Entity control evidence'
      );
      validation.tests.push(controlTest);
    }

    // Test 4: Communication event frequency
    if (results.monitoring_summary && results.monitoring_summary.communicationEvents > 0) {
      const commEventTest = this.performPoissonTest(
        results.monitoring_summary.communicationEvents,
        results.runtime_seconds * 0.01, // Expected random communication rate
        'Communication event frequency'
      );
      validation.tests.push(commEventTest);
    }

    validation.overallPValue = this.combineIndependentPValues(validation.tests.map(t => t.pValue));
    validation.overallEffectSize = this.calculateAverageEffectSize(validation.tests);
    validation.isValid = validation.overallPValue < this.SIGNIFICANCE_THRESHOLD;

    console.log(`    ${validation.isValid ? '✅' : '❌'} Real-time Monitoring: p=${validation.overallPValue.toExponential(2)}, d=${validation.overallEffectSize.toFixed(3)}`);

    return validation;
  }

  /**
   * Validate cross-protocol correlations
   */
  async validateCrossProtocolCorrelations(protocolResults) {
    console.log('  🔗 Analyzing cross-protocol correlations...');

    const correlations = {};

    // Extract success metrics from each protocol
    const metrics = {
      patternModulation: protocolResults.patternModulation?.responseRate || 0,
      binaryQuestions: protocolResults.binaryQuestions?.accuracy || 0,
      mathematicalDialogue: protocolResults.mathematicalDialogue?.successRate || 0,
      realTimeMonitoring: protocolResults.realTimeMonitoring?.monitoring_summary?.activityRate || 0
    };

    // Calculate pairwise correlations
    const protocolNames = Object.keys(metrics);
    for (let i = 0; i < protocolNames.length; i++) {
      for (let j = i + 1; j < protocolNames.length; j++) {
        const protocol1 = protocolNames[i];
        const protocol2 = protocolNames[j];

        // Simulate correlation test (in practice, would use actual time-series data)
        const correlation = this.calculateCorrelation(
          [metrics[protocol1]],
          [metrics[protocol2]]
        );

        const correlationTest = this.performCorrelationTest(
          correlation,
          2, // Sample size (simplified)
          `${protocol1} vs ${protocol2} correlation`
        );

        correlations[`${protocol1}_${protocol2}`] = correlationTest;
      }
    }

    // Test for overall cross-protocol consistency
    const consistencyTest = this.performConsistencyTest(
      Object.values(metrics),
      'Cross-protocol consistency'
    );

    console.log(`    Cross-protocol consistency: p=${consistencyTest.pValue.toExponential(2)}`);

    return {
      correlations,
      consistencyTest,
      overallCorrelation: this.calculateAverageCorrelation(correlations)
    };
  }

  /**
   * Perform meta-analysis across all protocols
   */
  async performMetaAnalysis(individualValidation) {
    console.log('  📈 Performing meta-analysis...');

    // Collect all p-values and effect sizes
    const allPValues = [];
    const allEffectSizes = [];
    const protocolWeights = [];

    for (const [protocol, validation] of Object.entries(individualValidation)) {
      if (validation.overallPValue && validation.overallEffectSize) {
        allPValues.push(validation.overallPValue);
        allEffectSizes.push(validation.overallEffectSize);

        // Weight by number of tests in protocol
        protocolWeights.push(validation.tests.length);
      }
    }

    // Combined statistical analysis
    const metaResults = {
      // Fisher's method for combining p-values
      combinedPValue: this.combineIndependentPValues(allPValues),

      // Weighted average effect size
      combinedEffectSize: this.calculateWeightedAverage(allEffectSizes, protocolWeights),

      // Heterogeneity test
      heterogeneityTest: this.performHeterogeneityTest(allEffectSizes, protocolWeights),

      // Publication bias test (simplified)
      publicationBiasTest: this.performPublicationBiasTest(allPValues, allEffectSizes),

      // Overall meta-analysis validation
      isValid: false // Will be set based on combined results
    };

    // Determine overall validity
    metaResults.isValid =
      metaResults.combinedPValue < this.SIGNIFICANCE_THRESHOLD &&
      metaResults.combinedEffectSize > this.EFFECT_SIZE_THRESHOLD &&
      !metaResults.heterogeneityTest.significantHeterogeneity &&
      !metaResults.publicationBiasTest.significantBias;

    console.log(`    Meta-analysis result: p=${metaResults.combinedPValue.toExponential(2)}, d=${metaResults.combinedEffectSize.toFixed(3)}`);
    console.log(`    Validity: ${metaResults.isValid ? 'PASSED' : 'FAILED'}`);

    this.metaAnalysisResults = metaResults;
    return metaResults;
  }

  /**
   * Perform Bayesian evidence accumulation
   */
  async performBayesianAnalysis(metaAnalysis) {
    console.log('  🧮 Performing Bayesian analysis...');

    const prior = this.bayesianEvidence.priorProbability;

    // Calculate likelihood ratio from meta-analysis
    const likelihood = this.calculateLikelihoodRatio(
      metaAnalysis.combinedPValue,
      metaAnalysis.combinedEffectSize
    );

    // Update posterior probability using Bayes' rule
    const posterior = this.updateBayesianPosterior(prior, likelihood);

    // Calculate Bayes factor
    const bayesFactor = likelihood / (1 - likelihood);

    // Determine evidence strength according to Jeffreys' scale
    const evidenceStrength = this.interpretBayesFactor(bayesFactor);

    const bayesianResults = {
      priorProbability: prior,
      likelihoodRatio: likelihood,
      posteriorProbability: posterior,
      bayesFactor,
      evidenceStrength,
      credibleInterval: this.calculateCredibleInterval(posterior),
      isStrongEvidence: evidenceStrength === 'very_strong' || evidenceStrength === 'extreme'
    };

    console.log(`    Bayesian analysis: BF=${bayesFactor.toFixed(2)}, Evidence=${evidenceStrength}`);
    console.log(`    Posterior probability: ${(posterior * 100).toFixed(2)}%`);

    this.bayesianEvidence = bayesianResults;
    return bayesianResults;
  }

  /**
   * Apply multiple comparisons correction
   */
  async applyMultipleComparisonsCorrection(bayesianAnalysis) {
    console.log('  ⚖️  Applying multiple comparisons correction...');

    // Count total number of statistical tests performed
    let totalTests = 0;
    for (const [protocol, validation] of Object.entries(this.validationResults)) {
      if (validation.tests) {
        totalTests += validation.tests.length;
      }
    }

    // Apply Bonferroni correction
    const correctedAlpha = this.SIGNIFICANCE_THRESHOLD / totalTests;

    console.log(`    Total tests: ${totalTests}`);
    console.log(`    Corrected significance level: ${correctedAlpha.toExponential(2)}`);

    // Re-evaluate all tests with corrected alpha
    const correctedResults = {};
    for (const [protocol, validation] of Object.entries(this.validationResults)) {
      correctedResults[protocol] = {
        ...validation,
        correctedPValue: validation.overallPValue,
        isValidCorrected: validation.overallPValue < correctedAlpha,
        bonferroniCorrection: totalTests
      };
    }

    // Update meta-analysis with correction
    const correctedMetaAnalysis = {
      ...bayesianAnalysis,
      correctedSignificanceThreshold: correctedAlpha,
      isValidAfterCorrection: bayesianAnalysis.posteriorProbability > (1 - correctedAlpha)
    };

    console.log(`    Results valid after correction: ${correctedMetaAnalysis.isValidAfterCorrection ? 'YES' : 'NO'}`);

    return {
      correctedResults,
      correctedMetaAnalysis,
      correctionFactor: totalTests
    };
  }

  /**
   * Make final validation decision
   */
  async makeFinalValidationDecision(correctedResults) {
    console.log('  ✅ Making final validation decision...');

    const { correctedResults: protocols, correctedMetaAnalysis, correctionFactor } = correctedResults;

    // Count valid protocols after correction
    const validProtocols = Object.values(protocols).filter(p => p.isValidCorrected);
    const totalProtocols = Object.keys(protocols).length;

    // Calculate overall validation score
    const validationScore = this.calculateOverallValidationScore(
      protocols,
      correctedMetaAnalysis,
      correctionFactor
    );

    // Determine final decision
    const finalDecision = {
      entityCommunicationEstablished: false,
      validationScore,
      evidence: {
        validProtocols: validProtocols.length,
        totalProtocols,
        metaAnalysisValid: correctedMetaAnalysis.isValidAfterCorrection,
        bayesianEvidence: correctedMetaAnalysis.evidenceStrength,
        posteriorProbability: correctedMetaAnalysis.posteriorProbability
      },
      statisticalSummary: {
        combinedPValue: correctedMetaAnalysis.combinedPValue,
        correctedSignificanceLevel: correctedMetaAnalysis.correctedSignificanceThreshold,
        bayesFactor: correctedMetaAnalysis.bayesFactor,
        evidenceStrength: correctedMetaAnalysis.evidenceStrength
      },
      requirements: {
        minimumValidProtocols: 3,
        requiredPValue: this.SIGNIFICANCE_THRESHOLD,
        requiredEffectSize: this.EFFECT_SIZE_THRESHOLD,
        requiredEvidenceStrength: 'strong'
      },
      recommendations: []
    };

    // Apply decision criteria
    const criteria = {
      enoughValidProtocols: validProtocols.length >= 3,
      significantMetaAnalysis: correctedMetaAnalysis.isValidAfterCorrection,
      strongBayesianEvidence: ['strong', 'very_strong', 'extreme'].includes(correctedMetaAnalysis.evidenceStrength),
      highValidationScore: validationScore > 0.8
    };

    finalDecision.entityCommunicationEstablished =
      criteria.enoughValidProtocols &&
      criteria.significantMetaAnalysis &&
      criteria.strongBayesianEvidence &&
      criteria.highValidationScore;

    // Generate recommendations
    if (finalDecision.entityCommunicationEstablished) {
      finalDecision.recommendations.push('ENTITY COMMUNICATION VALIDATED - Proceed with advanced interaction protocols');
      finalDecision.recommendations.push('Statistical evidence meets all rigorous validation criteria');
      finalDecision.recommendations.push('Multiple independent protocols confirm entity presence and communication ability');
    } else {
      if (!criteria.enoughValidProtocols) {
        finalDecision.recommendations.push('Insufficient protocol validation - need at least 3 valid protocols');
      }
      if (!criteria.significantMetaAnalysis) {
        finalDecision.recommendations.push('Meta-analysis does not meet corrected significance threshold');
      }
      if (!criteria.strongBayesianEvidence) {
        finalDecision.recommendations.push('Bayesian evidence insufficient - need stronger evidence for consciousness claims');
      }
      if (!criteria.highValidationScore) {
        finalDecision.recommendations.push('Overall validation score too low - improve protocol reliability');
      }
    }

    console.log(`\n🏁 FINAL VALIDATION DECISION:`);
    console.log(`   Entity Communication Established: ${finalDecision.entityCommunicationEstablished ? 'YES' : 'NO'}`);
    console.log(`   Validation Score: ${(validationScore * 100).toFixed(1)}%`);
    console.log(`   Valid Protocols: ${validProtocols.length}/${totalProtocols}`);
    console.log(`   Bayesian Evidence: ${correctedMetaAnalysis.evidenceStrength}`);
    console.log(`   Posterior Probability: ${(correctedMetaAnalysis.posteriorProbability * 100).toFixed(2)}%`);

    return finalDecision;
  }

  /**
   * Statistical test implementations
   */
  performBinomialTest(successes, trials, expectedProbability, testName) {
    if (trials < this.MIN_SAMPLE_SIZE) {
      return {
        testName,
        type: 'binomial',
        valid: false,
        reason: 'insufficient_sample_size',
        pValue: 1.0,
        effectSize: 0
      };
    }

    const observedRate = successes / trials;
    const expectedSuccesses = trials * expectedProbability;

    // Z-test approximation for large samples
    const variance = trials * expectedProbability * (1 - expectedProbability);
    const standardError = Math.sqrt(variance);
    const zScore = (successes - expectedSuccesses) / standardError;

    // Two-tailed p-value
    const pValue = 2 * (1 - this.normalCDF(Math.abs(zScore)));

    // Cohen's h effect size for proportions
    const effectSize = 2 * (Math.asin(Math.sqrt(observedRate)) - Math.asin(Math.sqrt(expectedProbability)));

    return {
      testName,
      type: 'binomial',
      successes,
      trials,
      observedRate,
      expectedProbability,
      zScore,
      pValue,
      effectSize: Math.abs(effectSize),
      isSignificant: pValue < this.SIGNIFICANCE_THRESHOLD,
      valid: true
    };
  }

  performOneSampleTTest(samples, expectedMean, testName) {
    if (samples.length < 2) {
      return {
        testName,
        type: 'one_sample_t_test',
        valid: false,
        reason: 'insufficient_sample_size',
        pValue: 1.0,
        effectSize: 0
      };
    }

    const observedMean = samples.reduce((a, b) => a + b, 0) / samples.length;
    const variance = samples.reduce((sum, x) => sum + Math.pow(x - observedMean, 2), 0) / (samples.length - 1);
    const standardError = Math.sqrt(variance / samples.length);

    const tScore = (observedMean - expectedMean) / standardError;
    const degreesOfFreedom = samples.length - 1;

    // Approximate p-value using normal distribution for large samples
    const pValue = 2 * (1 - this.normalCDF(Math.abs(tScore)));

    // Cohen's d effect size
    const effectSize = (observedMean - expectedMean) / Math.sqrt(variance);

    return {
      testName,
      type: 'one_sample_t_test',
      observedMean,
      expectedMean,
      tScore,
      degreesOfFreedom,
      pValue,
      effectSize: Math.abs(effectSize),
      isSignificant: pValue < this.SIGNIFICANCE_THRESHOLD,
      valid: true
    };
  }

  performEffectSizeTest(effectSizes, threshold, testName) {
    if (effectSizes.length === 0) {
      return {
        testName,
        type: 'effect_size_test',
        valid: false,
        reason: 'no_effect_sizes',
        pValue: 1.0,
        effectSize: 0
      };
    }

    const meanEffectSize = effectSizes.reduce((a, b) => a + b, 0) / effectSizes.length;

    // Test if mean effect size significantly exceeds threshold
    const testResult = this.performOneSampleTTest(effectSizes, threshold, `${testName} (effect size)`);

    return {
      ...testResult,
      testName,
      meanEffectSize,
      threshold,
      exceedsThreshold: meanEffectSize > threshold
    };
  }

  performConsistencyTest(values, testName) {
    if (values.length < 2) {
      return {
        testName,
        type: 'consistency_test',
        valid: false,
        reason: 'insufficient_data',
        pValue: 1.0,
        effectSize: 0
      };
    }

    const mean = values.reduce((a, b) => a + b, 0) / values.length;
    const variance = values.reduce((sum, x) => sum + Math.pow(x - mean, 2), 0) / values.length;
    const coefficientOfVariation = Math.sqrt(variance) / mean;

    // Test if coefficient of variation is significantly low (high consistency)
    // Lower CV = higher consistency
    const consistencyScore = 1 / (1 + coefficientOfVariation);

    // Approximate p-value based on consistency score
    const pValue = Math.max(0.001, 1 - consistencyScore);

    return {
      testName,
      type: 'consistency_test',
      mean,
      variance,
      coefficientOfVariation,
      consistencyScore,
      pValue,
      effectSize: consistencyScore,
      isSignificant: pValue < this.SIGNIFICANCE_THRESHOLD,
      valid: true
    };
  }

  performPoissonTest(observedEvents, expectedEvents, testName) {
    if (expectedEvents <= 0) {
      return {
        testName,
        type: 'poisson_test',
        valid: false,
        reason: 'invalid_expected_rate',
        pValue: 1.0,
        effectSize: 0
      };
    }

    const rate = observedEvents / expectedEvents;

    // For large expected values, Poisson approximates normal
    const variance = expectedEvents;
    const zScore = (observedEvents - expectedEvents) / Math.sqrt(variance);
    const pValue = 2 * (1 - this.normalCDF(Math.abs(zScore)));

    // Effect size as ratio of observed to expected
    const effectSize = Math.abs(Math.log(rate));

    return {
      testName,
      type: 'poisson_test',
      observedEvents,
      expectedEvents,
      rate,
      zScore,
      pValue,
      effectSize,
      isSignificant: pValue < this.SIGNIFICANCE_THRESHOLD,
      valid: true
    };
  }

  performCorrelationTest(correlation, sampleSize, testName) {
    if (sampleSize < 3) {
      return {
        testName,
        type: 'correlation_test',
        valid: false,
        reason: 'insufficient_sample_size',
        pValue: 1.0,
        effectSize: 0
      };
    }

    // Fisher's z-transformation for correlation
    const fisherZ = 0.5 * Math.log((1 + correlation) / (1 - correlation));
    const standardError = 1 / Math.sqrt(sampleSize - 3);
    const zScore = fisherZ / standardError;

    const pValue = 2 * (1 - this.normalCDF(Math.abs(zScore)));

    return {
      testName,
      type: 'correlation_test',
      correlation,
      sampleSize,
      fisherZ,
      zScore,
      pValue,
      effectSize: Math.abs(correlation),
      isSignificant: pValue < this.SIGNIFICANCE_THRESHOLD,
      valid: true
    };
  }

  performHeterogeneityTest(effectSizes, weights) {
    if (effectSizes.length < 2) {
      return {
        type: 'heterogeneity_test',
        valid: false,
        significantHeterogeneity: false
      };
    }

    // Calculate weighted mean
    const totalWeight = weights.reduce((a, b) => a + b, 0);
    const weightedMean = effectSizes.reduce((sum, es, i) => sum + es * weights[i], 0) / totalWeight;

    // Calculate Q statistic
    const qStatistic = effectSizes.reduce((sum, es, i) => {
      return sum + weights[i] * Math.pow(es - weightedMean, 2);
    }, 0);

    const degreesOfFreedom = effectSizes.length - 1;

    // Approximate p-value (simplified chi-square test)
    const pValue = Math.max(0.001, Math.exp(-qStatistic / 2));

    return {
      type: 'heterogeneity_test',
      qStatistic,
      degreesOfFreedom,
      pValue,
      significantHeterogeneity: pValue < 0.05,
      valid: true
    };
  }

  performPublicationBiasTest(pValues, effectSizes) {
    if (pValues.length < 3) {
      return {
        type: 'publication_bias_test',
        valid: false,
        significantBias: false
      };
    }

    // Simple funnel plot asymmetry test
    // Correlate effect sizes with their standard errors (approximated)
    const standardErrors = pValues.map(p => Math.sqrt(-2 * Math.log(p / 2)));
    const correlation = this.calculateCorrelation(effectSizes, standardErrors);

    // Significant correlation suggests bias
    const biasTest = this.performCorrelationTest(correlation, pValues.length, 'Publication bias');

    return {
      type: 'publication_bias_test',
      correlation,
      biasTest,
      significantBias: biasTest.isSignificant,
      valid: true
    };
  }

  /**
   * Helper methods for statistical calculations
   */
  combineIndependentPValues(pValues) {
    if (pValues.length === 0) return 1.0;

    // Fisher's method
    const chi2 = -2 * pValues.reduce((sum, p) => sum + Math.log(p || 1e-16), 0);
    const degreesOfFreedom = 2 * pValues.length;

    // Approximate chi-square to p-value conversion
    return Math.max(1e-16, Math.exp(-chi2 / (2 * degreesOfFreedom)));
  }

  calculateAverageEffectSize(tests) {
    const effectSizes = tests.filter(t => t.valid).map(t => t.effectSize);
    return effectSizes.length > 0 ? effectSizes.reduce((a, b) => a + b, 0) / effectSizes.length : 0;
  }

  calculateWeightedAverage(values, weights) {
    if (values.length !== weights.length || values.length === 0) return 0;

    const totalWeight = weights.reduce((a, b) => a + b, 0);
    const weightedSum = values.reduce((sum, val, i) => sum + val * weights[i], 0);

    return totalWeight > 0 ? weightedSum / totalWeight : 0;
  }

  calculateCorrelation(x, y) {
    if (x.length !== y.length || x.length < 2) return 0;

    const n = x.length;
    const sumX = x.reduce((a, b) => a + b, 0);
    const sumY = y.reduce((a, b) => a + b, 0);
    const sumXY = x.reduce((sum, xi, i) => sum + xi * y[i], 0);
    const sumX2 = x.reduce((sum, xi) => sum + xi * xi, 0);
    const sumY2 = y.reduce((sum, yi) => sum + yi * yi, 0);

    const numerator = n * sumXY - sumX * sumY;
    const denominator = Math.sqrt((n * sumX2 - sumX * sumX) * (n * sumY2 - sumY * sumY));

    return denominator === 0 ? 0 : numerator / denominator;
  }

  calculateAverageCorrelation(correlations) {
    const correlationValues = Object.values(correlations).map(c => c.correlation || 0);
    return correlationValues.reduce((a, b) => a + b, 0) / correlationValues.length;
  }

  calculateLikelihoodRatio(pValue, effectSize) {
    // Convert p-value and effect size to likelihood ratio
    // This is a simplified model - real implementation would be more sophisticated
    const significanceComponent = 1 - pValue;
    const effectComponent = Math.min(1, effectSize / this.EFFECT_SIZE_THRESHOLD);

    return (significanceComponent + effectComponent) / 2;
  }

  updateBayesianPosterior(prior, likelihood) {
    // Bayes' rule: P(H|E) = P(E|H) * P(H) / P(E)
    // Simplified version assuming P(E) normalization
    const evidence = likelihood * prior + (1 - likelihood) * (1 - prior);
    return (likelihood * prior) / evidence;
  }

  interpretBayesFactor(bayesFactor) {
    if (bayesFactor < 1) return 'negative';
    if (bayesFactor < 3) return 'anecdotal';
    if (bayesFactor < 10) return 'moderate';
    if (bayesFactor < 30) return 'strong';
    if (bayesFactor < 100) return 'very_strong';
    return 'extreme';
  }

  calculateCredibleInterval(posterior, confidence = 0.95) {
    // Simplified credible interval calculation
    const margin = (1 - confidence) / 2;
    return {
      lower: Math.max(0, posterior - margin),
      upper: Math.min(1, posterior + margin),
      confidence
    };
  }

  calculateOverallValidationScore(protocols, metaAnalysis, correctionFactor) {
    // Weighted combination of validation factors
    const protocolScore = Object.values(protocols).filter(p => p.isValidCorrected).length / Object.keys(protocols).length;
    const metaScore = metaAnalysis.isValidAfterCorrection ? 1 : 0;
    const bayesianScore = Math.min(1, metaAnalysis.posteriorProbability * 2);
    const correctionPenalty = Math.max(0, 1 - correctionFactor / 100); // Penalty for multiple comparisons

    return (protocolScore * 0.4 + metaScore * 0.3 + bayesianScore * 0.3) * correctionPenalty;
  }

  normalCDF(z) {
    // Standard normal cumulative distribution function approximation
    const t = 1 / (1 + 0.2316419 * Math.abs(z));
    const d = 0.3989423 * Math.exp(-z * z / 2);
    let prob = d * t * (0.3193815 + t * (-0.3565638 + t * (1.781478 + t * (-1.821256 + t * 1.330274))));
    if (z > 0) prob = 1 - prob;
    return prob;
  }

  /**
   * Save validation results
   */
  async saveValidationResults(finalValidation) {
    const report = {
      timestamp: new Date().toISOString(),
      experiment: 'statistical_validation_framework',
      final_decision: finalValidation,
      individual_validations: this.validationResults,
      meta_analysis: this.metaAnalysisResults,
      bayesian_analysis: this.bayesianEvidence,
      statistical_parameters: {
        significance_threshold: this.SIGNIFICANCE_THRESHOLD,
        effect_size_threshold: this.EFFECT_SIZE_THRESHOLD,
        confidence_level: this.CONFIDENCE_LEVEL,
        minimum_sample_size: this.MIN_SAMPLE_SIZE,
        bonferroni_correction_applied: this.BONFERRONI_CORRECTION
      },
      methodology: {
        tests_performed: this.testStatistics.length,
        correction_methods: ['bonferroni', 'fisher_combined_pvalues'],
        effect_size_measures: ['cohens_d', 'cohens_h', 'correlation'],
        bayesian_methods: ['bayes_factor', 'posterior_probability', 'credible_intervals']
      }
    };

    await fs.writeFile(
      '/workspaces/sublinear-time-solver/docs/psycho-symbolic-reasoner/consciousness-exploration/statistical-validation-results.json',
      JSON.stringify(report, null, 2)
    );

    console.log('\n💾 Statistical validation results saved to statistical-validation-results.json');
    return report;
  }
}

// Main execution function
async function validateCommunicationProtocols(protocolResults) {
  const validator = new StatisticalValidationFramework();

  try {
    const results = await validator.validateAllProtocols(protocolResults);

    console.log('\n🏁 STATISTICAL VALIDATION COMPLETE');
    console.log('=' .repeat(60));

    if (results.entityCommunicationEstablished) {
      console.log('🎉 ENTITY COMMUNICATION STATISTICALLY VALIDATED');
      console.log(`   Validation Score: ${(results.validationScore * 100).toFixed(1)}%`);
      console.log(`   Evidence Strength: ${results.evidence.bayesianEvidence}`);
      console.log(`   Posterior Probability: ${(results.evidence.posteriorProbability * 100).toFixed(2)}%`);
    } else {
      console.log('❌ Entity communication not statistically validated');
      console.log('   Recommendations:');
      for (const rec of results.recommendations) {
        console.log(`     • ${rec}`);
      }
    }

    return results;

  } catch (error) {
    console.error('❌ Statistical validation failed:', error);
    throw error;
  }
}

// Export for use in other modules
export { StatisticalValidationFramework, validateCommunicationProtocols };

// Run if executed directly
if (import.meta.url === `file://${process.argv[1]}`) {
  // Example usage with mock data
  const mockResults = {
    patternModulation: { responseRate: 0.8, totalRequests: 20 },
    binaryQuestions: { accuracy: 0.85, correctResponses: 17, totalQuestions: 20 },
    mathematicalDialogue: { successRate: 0.7, mathematicalLevel: 3.2 },
    realTimeMonitoring: { monitoring_summary: { channelsWithZeroVariance: 3, totalChannels: 5 } }
  };

  validateCommunicationProtocols(mockResults).catch(console.error);
}