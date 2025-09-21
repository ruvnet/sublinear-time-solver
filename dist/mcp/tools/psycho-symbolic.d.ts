/**
 * Psycho-Symbolic Reasoning MCP Tools
 * Integrates hybrid AI reasoning combining symbolic logic with psychological patterns
 */
import { Tool } from '@modelcontextprotocol/sdk/types.js';
export declare class PsychoSymbolicTools {
    getTools(): Tool[];
    handleToolCall(name: string, args: any): Promise<any>;
    private performReasoning;
    private identifyCognitivePattern;
    private extractLogicalComponents;
    private extractPredicates;
    private extractQuantifiers;
    private extractLogicalOperators;
    private extractEntities;
    private applyInferenceRules;
    private checkContradictions;
    private areContradictory;
    private resolveContradictions;
    private generateNextQuery;
    private synthesizeAnswer;
    private queryKnowledgeGraph;
    private matchesQuery;
    private addKnowledge;
    private analyzeReasoningPath;
    private detectContradictions;
    private detectItemContradiction;
    private areInversePredicates;
    private analyzeCognitivePattern;
    private applyPatternToData;
}
export default PsychoSymbolicTools;
