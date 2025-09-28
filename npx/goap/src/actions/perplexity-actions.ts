/**
 * Perplexity API Integration Actions
 * Implements search and synthesis capabilities using Perplexity API
 */

import axios from 'axios';
import { GoapAction, WorldState, Effect, Precondition } from '../core/types.js';

export interface PerplexitySearchParams {
  query: string | string[];
  mode?: 'web' | 'academic';
  recency?: 'hour' | 'day' | 'week' | 'month' | 'year';
  domains?: string[];
  maxResults?: number;
}

export interface PerplexityChatParams {
  messages: Array<{ role: string; content: string }>;
  model?: string;
  maxTokens?: number;
  temperature?: number;
  searchDomainFilter?: string[];
  searchRecencyFilter?: 'hour' | 'day' | 'week' | 'month' | 'year';
  searchMode?: 'web' | 'academic';
}

export class PerplexityClient {
  private apiKey: string;
  private baseURL = 'https://api.perplexity.ai';

  constructor(apiKey: string) {
    this.apiKey = apiKey;
  }

  /**
   * Perform web search using Perplexity Search API
   */
  async search(params: PerplexitySearchParams) {
    const searchBody: any = {
      query: Array.isArray(params.query) ? params.query.join(' ') : params.query,
      return_citations: true
    };

    // Only add optional parameters if provided
    if (params.recency) searchBody.search_recency_filter = params.recency;
    if (params.domains) searchBody.search_domain_filter = params.domains;
    if (params.maxResults) searchBody.max_results = params.maxResults;
    // Note: search_mode is not supported by the Search API

    const response = await axios.post(`${this.baseURL}/search`, searchBody, {
      headers: {
        'Authorization': `Bearer ${this.apiKey}`,
        'Content-Type': 'application/json'
      }
    });

    return response.data;
  }

  /**
   * Perform chat completion using Perplexity Sonar models
   */
  async chat(params: PerplexityChatParams) {
    const response = await axios.post(`${this.baseURL}/chat/completions`, {
      model: params.model || 'sonar-pro',
      messages: params.messages,
      max_tokens: params.maxTokens || 2000,
      temperature: params.temperature || 0.1,
      search_domain_filter: params.searchDomainFilter,
      search_recency_filter: params.searchRecencyFilter,
      search_mode: params.searchMode || 'web'
    }, {
      headers: {
        'Authorization': `Bearer ${this.apiKey}`,
        'Content-Type': 'application/json'
      }
    });

    return response.data;
  }
}

// Lazy initialization of Perplexity client
let client: PerplexityClient | null = null;

function getPerplexityClient(): PerplexityClient {
  if (!client) {
    const apiKey = process.env.PERPLEXITY_API_KEY;

    if (!apiKey || apiKey === '') {
      console.error('\n❌ ERROR: PERPLEXITY_API_KEY is not set');
      console.error('💡 Get your API key from: https://www.perplexity.ai/settings/api');
      console.error('📝 Set it with: export PERPLEXITY_API_KEY="your-key"');
      console.error('   Or add it to your .env file\n');
    }

    client = new PerplexityClient(apiKey || 'MISSING_API_KEY');
  }

  return client;
}

/**
 * Action: Compose search queries from user input
 */
export const composeQueriesAction: GoapAction = {
  name: 'compose_queries',
  cost: 1,
  preconditions: [
    { key: 'user_query', value: true, operator: 'exists' }
  ],
  effects: [
    { key: 'queries_composed', value: true, operation: 'set' },
    { key: 'search_queries', value: [], operation: 'set' }
  ],
  async execute(state: WorldState, params?: any) {
    try {
      const userQuery = state.user_query as string;
      const domains = params?.domains || [];
      const queryVariants = params?.queryVariants || [];

      // Base queries
      const queries = [userQuery];

      // Add domain-specific variants if domains specified
      if (domains && domains.length > 0) {
        queries.push(`${userQuery} site:${domains[0]}`);
      }

      // Add any plugin-generated variants
      if (queryVariants && queryVariants.length > 0) {
        queries.push(...queryVariants.slice(0, 3)); // Limit to 3 additional variants
      }

      // Add context-aware variants
      queries.push(
        `${userQuery} research`,
        `${userQuery} latest developments`
      );

      const newState = { ...state };
      newState.queries_composed = true;
      newState.search_queries = queries.slice(0, 5); // Limit total queries

      return {
        success: true,
        newState,
        data: { queries: newState.search_queries }
      };
    } catch (error) {
      return {
        success: false,
        newState: state,
        error: error instanceof Error ? error.message : 'Failed to compose queries'
      };
    }
  }
};

/**
 * Action: Search information using Perplexity Search API
 */
export const searchInformationAction: GoapAction = {
  name: 'search_information',
  cost: 3,
  preconditions: [
    { key: 'queries_composed', value: true, operator: 'equals' }
  ],
  effects: [
    { key: 'information_searched', value: true, operation: 'set' },
    { key: 'search_results', value: [], operation: 'set' }
  ],
  async execute(state: WorldState, params?: any) {
    try {
      const queries = state.search_queries as string[];
      console.log('🔍 Executing search with queries:', queries);

      const searchParams: PerplexitySearchParams = {
        query: queries,
        mode: params?.mode || 'web',
        recency: params?.recency,
        domains: params?.domains,
        maxResults: params?.maxResults || 10
      };

      const searchResults = await getPerplexityClient().search(searchParams);
      console.log('✅ Search completed successfully');

      const newState = { ...state };
      newState.information_searched = true;
      newState.search_results = searchResults.results || [];
      newState.search_metadata = {
        query_count: queries.length,
        result_count: searchResults.results?.length || 0,
        timestamp: new Date().toISOString()
      };

      return {
        success: true,
        newState,
        data: searchResults
      };
    } catch (error: any) {
      console.error('❌ Search failed:', error.response?.data || error.message);
      return {
        success: false,
        newState: state,
        error: error.response?.data?.error?.message || error.message || 'Search failed'
      };
    }
  }
};

/**
 * Action: Synthesize results using Perplexity Sonar chat
 */
export const synthesizeResultsAction: GoapAction = {
  name: 'synthesize_results',
  cost: 5,
  preconditions: [
    { key: 'information_searched', value: true, operator: 'equals' }
  ],
  effects: [
    { key: 'results_synthesized', value: true, operation: 'set' },
    { key: 'final_answer', value: '', operation: 'set' },
    { key: 'citations', value: [], operation: 'set' }
  ],
  async execute(state: WorldState, params?: any) {
    try {
      const userQuery = state.user_query as string;
      const searchResults = state.search_results as any[];

      // Prepare context from search results
      const context = searchResults.map((result, index) =>
        `[${index + 1}] ${result.title}\n${result.snippet}\nURL: ${result.url}\n`
      ).join('\n');

      const messages = [
        {
          role: 'system',
          content: 'You are a research assistant. Provide a comprehensive answer based on the search results. Include specific citations using [number] format. Be factual and well-structured.'
        },
        {
          role: 'user',
          content: `Question: ${userQuery}\n\nSearch Results:\n${context}\n\nPlease provide a detailed answer with proper citations.`
        }
      ];

      const chatParams: PerplexityChatParams = {
        messages,
        model: params?.model || 'sonar-pro',
        maxTokens: params?.maxTokens || 2000,
        temperature: params?.temperature || 0.1,
        searchDomainFilter: params?.domains,
        searchRecencyFilter: params?.recency,
        searchMode: params?.mode || 'web'
      };

      const chatResponse = await getPerplexityClient().chat(chatParams);

      // Extract citations from the response
      const answer = chatResponse.choices[0]?.message?.content || '';
      const citations = extractCitations(answer, searchResults);

      const newState = { ...state };
      newState.results_synthesized = true;
      newState.final_answer = answer;
      newState.citations = citations;
      newState.usage = {
        tokens: chatResponse.usage?.total_tokens || 0,
        cost: calculateCost(chatResponse.usage?.total_tokens || 0)
      };

      return {
        success: true,
        newState,
        data: {
          answer,
          citations,
          usage: newState.usage
        }
      };
    } catch (error) {
      return {
        success: false,
        newState: state,
        error: error instanceof Error ? error.message : 'Synthesis failed'
      };
    }
  }
};

/**
 * Extract citations from synthesized answer
 */
function extractCitations(answer: string, searchResults: any[]) {
    const citations: any[] = [];
    const citationRegex = /\[(\d+)\]/g;
    let match;

    while ((match = citationRegex.exec(answer)) !== null) {
      const index = parseInt(match[1]) - 1;
      if (index >= 0 && index < searchResults.length) {
        const result = searchResults[index];
        citations.push({
          index: index + 1,
          title: result.title,
          url: result.url,
          snippet: result.snippet,
          publishDate: result.published_date
        });
      }
    }

    return citations;
}

/**
 * Calculate estimated cost based on token usage
 */
function calculateCost(tokens: number): number {
    // Rough estimate: $5 per 1M tokens for Sonar Pro
    return (tokens / 1000000) * 5;
}

/**
 * Action: Verify citations and answer quality
 */
export const verifyAnswerAction: GoapAction = {
  name: 'verify_answer',
  cost: 2,
  preconditions: [
    { key: 'results_synthesized', value: true, operator: 'equals' }
  ],
  effects: [
    { key: 'answer_verified', value: true, operation: 'set' },
    { key: 'verification_notes', value: [], operation: 'set' }
  ],
  async execute(state: WorldState, params?: any) {
    try {
      const answer = state.final_answer as string;
      const citations = state.citations as any[];
      const notes: string[] = [];

      // Check citation coverage
      const citationCount = citations.length;
      if (citationCount === 0) {
        notes.push('⚠️ No citations found in answer');
      } else if (citationCount < 3) {
        notes.push(`ℹ️ Limited citations (${citationCount})`);
      } else {
        notes.push(`✅ Good citation coverage (${citationCount})`);
      }

      // Check answer length
      const wordCount = answer.split(/\s+/).length;
      if (wordCount < 50) {
        notes.push('⚠️ Answer may be too brief');
      } else if (wordCount > 1000) {
        notes.push('ℹ️ Very comprehensive answer');
      } else {
        notes.push('✅ Appropriate answer length');
      }

      // Check for unique sources
      const uniqueDomains = new Set(
        citations.map(c => new URL(c.url).hostname)
      );
      if (uniqueDomains.size >= 3) {
        notes.push('✅ Diverse source coverage');
      } else {
        notes.push('ℹ️ Limited source diversity');
      }

      const newState = { ...state };
      newState.answer_verified = true;
      newState.verification_notes = notes;

      return {
        success: true,
        newState,
        data: { notes }
      };
    } catch (error) {
      return {
        success: false,
        newState: state,
        error: error instanceof Error ? error.message : 'Verification failed'
      };
    }
  }
};

// Export all Perplexity actions
export const perplexityActions = [
  composeQueriesAction,
  searchInformationAction,
  synthesizeResultsAction,
  verifyAnswerAction
];