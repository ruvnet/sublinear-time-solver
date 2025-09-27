/**
 * Strange Loops - Emergent Intelligence Through Temporal Consciousness
 * @module strange-loops
 */
export interface NanoSwarmConfig {
    agentCount?: number;
    topology?: 'mesh' | 'hierarchical' | 'ring' | 'star';
    tickDurationNs?: number;
}
export interface QuantumContainerConfig {
    qubits?: number;
    entanglementPairs?: Array<[number, number]>;
    measurementBasis?: 'computational' | 'hadamard' | 'bell';
}
export interface TemporalPredictorConfig {
    horizonNs?: number;
    historySize?: number;
    learningRate?: number;
}
export declare class NanoSwarm {
    private agentCount;
    private topology;
    private tickDurationNs;
    metrics: {
        totalTicks: number;
        avgTicksPerSecond: number;
    };
    constructor(config?: NanoSwarmConfig);
    run(durationMs: number): Promise<void>;
}
export declare class QuantumContainer {
    private qubits;
    private state;
    constructor(config?: QuantumContainerConfig);
    createSuperposition(): Promise<void>;
    sample(): Promise<number>;
    measure(): Promise<number>;
    bias(state: number, fitness: number): Promise<void>;
}
export declare class TemporalPredictor {
    private horizonNs;
    private history;
    private historySize;
    constructor(config?: TemporalPredictorConfig);
    predict(currentValues: Array<number>): Promise<Array<number>>;
}
export declare class StrangeLoopSystem {
    private swarm?;
    private quantum?;
    private predictor?;
    constructor(config?: any);
    evolve(): Promise<void>;
}
declare const _default: {
    NanoSwarm: typeof NanoSwarm;
    QuantumContainer: typeof QuantumContainer;
    TemporalPredictor: typeof TemporalPredictor;
    StrangeLoopSystem: typeof StrangeLoopSystem;
};
export default _default;
//# sourceMappingURL=index.d.ts.map