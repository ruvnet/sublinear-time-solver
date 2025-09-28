/**
 * Anti-Hallucination and Factual Grounding Plugin
 * Ensures all claims are grounded with citations and implements verification schemas
 */

import { PluginContext, AdvancedPluginHooks } from '../../core/advanced-types.js';

export interface FactualClaim {
  claim: string;
  citations: string[];
  confidence: number;
  verified: boolean;
  groundingType: 'direct' | 'inferred' | 'synthesized';
}

export interface HallucinationCheck {
  totalClaims: number;
  groundedClaims: number;
  ungroundedClaims: string[];
  confidenceScore: number;
  hallucinationRisk: 'low' | 'medium' | 'high';
}

export class AntiHallucinationPlugin {
  name = 'anti-hallucination';
  version = '1.0.0';

  private factualClaims: FactualClaim[] = [];
  private hallucinationCheck: HallucinationCheck | null = null;
  private citationRequirement = 0.8; // 80% of claims must have citations

  hooks: AdvancedPluginHooks = {
    /**
     * Before search, set up grounding requirements
     */
    beforeSearch: async (context: PluginContext) => {
      console.log('🛡️ [Anti-Hallucination] Activating factual grounding requirements');

      // Enhance search to prioritize cited sources
      context.metadata = {
        ...context.metadata,
        groundingRequirements: {
          requireCitations: true,
          minimumCitationsPerClaim: 1,
          verificationLevel: 'strict'
        }
      };

      // Add citation-focused search parameters
      if (context.searchParams) {
        context.searchParams.return_citations = true;
        context.searchParams.citation_quality = 'high';
      }
    },

    /**
     * After search, extract and validate factual claims
     */
    afterSearch: async (results: any, context: PluginContext) => {
      console.log('🔍 [Anti-Hallucination] Extracting factual claims...');

      // Extract all factual claims from results
      this.factualClaims = this.extractFactualClaims(results);

      // Validate each claim against citations
      for (const claim of this.factualClaims) {
        claim.verified = this.verifyClaim(claim, results.citations || []);
      }

      // Calculate hallucination risk
      this.hallucinationCheck = this.assessHallucinationRisk(this.factualClaims);

      console.log(`📊 [Anti-Hallucination] Grounding rate: ${(this.hallucinationCheck.groundedClaims / this.hallucinationCheck.totalClaims * 100).toFixed(1)}%`);
      console.log(`⚠️ [Anti-Hallucination] Risk level: ${this.hallucinationCheck.hallucinationRisk}`);

      // Enhance results with grounding data
      results.grounding = {
        factualClaims: this.factualClaims,
        hallucinationCheck: this.hallucinationCheck
      };

      return results;
    },

    /**
     * Before synthesis, ensure grounding requirements
     */
    beforeSynthesize: async (context: PluginContext) => {
      if (!this.hallucinationCheck) return;

      // If high hallucination risk, modify synthesis approach
      if (this.hallucinationCheck.hallucinationRisk === 'high') {
        console.log('🚨 [Anti-Hallucination] High risk detected - enforcing strict grounding');

        context.synthesisParams = {
          ...context.synthesisParams,
          instruction: 'Only make claims that are directly supported by citations. Express uncertainty for any unverified information.',
          requireCitations: true,
          uncertaintyThreshold: 0.7
        };
      }
    },

    /**
     * After synthesis, validate final response
     */
    afterSynthesize: async (result: any, context: PluginContext) => {
      console.log('✅ [Anti-Hallucination] Validating synthesized response...');

      // Extract claims from synthesized response
      const responseClaims = this.extractResponseClaims(result.content);

      // Check each claim for grounding
      const validationResults = responseClaims.map(claim => ({
        claim,
        grounded: this.isClaimGrounded(claim, result.citations || []),
        requiresFlag: this.requiresUncertaintyFlag(claim)
      }));

      // Add uncertainty markers where needed
      let enhancedContent = result.content;
      for (const validation of validationResults) {
        if (!validation.grounded && validation.requiresFlag) {
          enhancedContent = this.addUncertaintyMarker(enhancedContent, validation.claim);
        }
      }

      result.content = enhancedContent;
      result.validation = {
        ...result.validation,
        antiHallucination: {
          totalClaims: validationResults.length,
          groundedClaims: validationResults.filter(v => v.grounded).length,
          uncertaintyMarkersAdded: validationResults.filter(v => v.requiresFlag && !v.grounded).length
        }
      };

      return result;
    },

    /**
     * Final verification against hallucination
     */
    verify: async (result: any, context: PluginContext) => {
      if (!this.hallucinationCheck) {
        return { valid: false, confidence: 0, method: 'no-hallucination-check' };
      }

      const groundingRate = this.hallucinationCheck.groundedClaims / Math.max(this.hallucinationCheck.totalClaims, 1);
      const meetsRequirement = groundingRate >= this.citationRequirement;

      // Additional checks
      const hasUnverifiedCritical = this.checkForCriticalUnverifiedClaims(result);
      const citationQuality = this.assessCitationQuality(result.citations || []);

      const overallScore = (groundingRate * 0.5) + (citationQuality * 0.3) + (hasUnverifiedCritical ? 0 : 0.2);

      return {
        valid: meetsRequirement && !hasUnverifiedCritical,
        confidence: overallScore,
        method: 'anti-hallucination-verification',
        details: {
          groundingRate,
          hallucinationRisk: this.hallucinationCheck.hallucinationRisk,
          ungroundedClaims: this.hallucinationCheck.ungroundedClaims.length,
          citationQuality
        }
      };
    }
  };

  /**
   * Extract factual claims from search results
   */
  private extractFactualClaims(results: any): FactualClaim[] {
    const claims: FactualClaim[] = [];
    const text = typeof results === 'string' ? results : JSON.stringify(results);

    // Pattern matching for factual statements
    const claimPatterns = [
      /(?:is|are|was|were|has|have|will|can|does|do)\s+[^.?!]+[.!]/gi,
      /\d+(?:\.\d+)?%?\s+(?:of|in|from|to|by)[^.?!]+[.!]/gi,
      /(?:according to|research shows|studies indicate)[^.?!]+[.!]/gi
    ];

    for (const pattern of claimPatterns) {
      const matches = text.match(pattern) || [];
      for (const match of matches) {
        claims.push({
          claim: match.trim(),
          citations: [],
          confidence: 0,
          verified: false,
          groundingType: 'direct'
        });
      }
    }

    return claims;
  }

  /**
   * Verify a claim against available citations
   */
  private verifyClaim(claim: FactualClaim, citations: string[]): boolean {
    // Check if claim content appears in any citation
    const claimKeywords = this.extractKeywords(claim.claim);

    for (const citation of citations) {
      const citationKeywords = this.extractKeywords(citation);
      const overlap = this.calculateKeywordOverlap(claimKeywords, citationKeywords);

      if (overlap > 0.3) {
        claim.citations.push(citation);
        claim.confidence = Math.max(claim.confidence, overlap);
      }
    }

    return claim.citations.length > 0;
  }

  /**
   * Extract keywords from text
   */
  private extractKeywords(text: string): Set<string> {
    return new Set(
      text.toLowerCase()
        .replace(/[^a-z0-9\s]/g, '')
        .split(/\s+/)
        .filter(word => word.length > 3)
    );
  }

  /**
   * Calculate keyword overlap between two sets
   */
  private calculateKeywordOverlap(set1: Set<string>, set2: Set<string>): number {
    const intersection = new Set([...set1].filter(x => set2.has(x)));
    const union = new Set([...set1, ...set2]);
    return intersection.size / union.size;
  }

  /**
   * Assess overall hallucination risk
   */
  private assessHallucinationRisk(claims: FactualClaim[]): HallucinationCheck {
    const totalClaims = claims.length;
    const groundedClaims = claims.filter(c => c.verified).length;
    const ungroundedClaims = claims.filter(c => !c.verified).map(c => c.claim);

    const groundingRate = totalClaims > 0 ? groundedClaims / totalClaims : 1;

    let hallucinationRisk: 'low' | 'medium' | 'high';
    if (groundingRate >= 0.8) hallucinationRisk = 'low';
    else if (groundingRate >= 0.6) hallucinationRisk = 'medium';
    else hallucinationRisk = 'high';

    return {
      totalClaims,
      groundedClaims,
      ungroundedClaims,
      confidenceScore: groundingRate,
      hallucinationRisk
    };
  }

  /**
   * Extract claims from synthesized response
   */
  private extractResponseClaims(content: string): string[] {
    // Split into sentences and filter for factual claims
    return content.split(/[.!?]/)
      .filter(sentence => sentence.trim().length > 10)
      .filter(sentence => /\b(?:is|are|was|were|has|have|will|can)\b/i.test(sentence));
  }

  /**
   * Check if a claim is grounded in citations
   */
  private isClaimGrounded(claim: string, citations: string[]): boolean {
    const claimKeywords = this.extractKeywords(claim);

    for (const citation of citations) {
      const overlap = this.calculateKeywordOverlap(
        claimKeywords,
        this.extractKeywords(citation)
      );

      if (overlap > 0.3) return true;
    }

    return false;
  }

  /**
   * Determine if claim requires uncertainty flag
   */
  private requiresUncertaintyFlag(claim: string): boolean {
    // Check for definitive language that needs qualification
    const definitivePatterns = [
      /\b(?:always|never|every|all|none|must|definitely|certainly)\b/i,
      /\b\d+(?:\.\d+)?%\b/, // Specific percentages
      /\b(?:proven|confirmed|established|guaranteed)\b/i
    ];

    return definitivePatterns.some(pattern => pattern.test(claim));
  }

  /**
   * Add uncertainty marker to content
   */
  private addUncertaintyMarker(content: string, claim: string): string {
    // Add qualifier before ungrounded claims
    const qualifiers = [
      'Based on available information, ',
      'It appears that ',
      'Evidence suggests that ',
      'While not fully verified, '
    ];

    const qualifier = qualifiers[Math.floor(Math.random() * qualifiers.length)];

    // Try to replace the claim with qualified version
    if (content.includes(claim)) {
      return content.replace(claim, qualifier.toLowerCase() + claim);
    }

    return content;
  }

  /**
   * Check for critical unverified claims
   */
  private checkForCriticalUnverifiedClaims(result: any): boolean {
    // Critical patterns that must be verified
    const criticalPatterns = [
      /\b(?:medical|health|safety|legal|financial)\b.*\b(?:advice|recommendation|must|should)\b/i,
      /\b(?:fatal|deadly|dangerous|toxic|harmful)\b/i,
      /\b(?:guaranteed|proven|cure|treatment)\b/i
    ];

    const content = result.content || '';
    const hasCritical = criticalPatterns.some(pattern => pattern.test(content));

    if (hasCritical) {
      // Check if critical claims are grounded
      const criticalClaims = this.extractResponseClaims(content)
        .filter(claim => criticalPatterns.some(p => p.test(claim)));

      return criticalClaims.some(claim =>
        !this.isClaimGrounded(claim, result.citations || [])
      );
    }

    return false;
  }

  /**
   * Assess citation quality
   */
  private assessCitationQuality(citations: string[]): number {
    if (citations.length === 0) return 0;

    // Check for quality indicators
    let qualityScore = 0;

    const qualityDomains = [
      'arxiv.org', 'nature.com', 'science.org', 'ieee.org',
      'acm.org', 'pubmed', '.edu', '.gov'
    ];

    for (const citation of citations) {
      const hasQualityDomain = qualityDomains.some(domain =>
        citation.toLowerCase().includes(domain)
      );

      if (hasQualityDomain) qualityScore += 1;
    }

    return Math.min(qualityScore / citations.length, 1.0);
  }
}

export default new AntiHallucinationPlugin();