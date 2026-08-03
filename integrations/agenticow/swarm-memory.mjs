/**
 * SwarmMemory — copy-on-write collective memory for multi-agent coordination,
 * backed by agenticow (ruvnet's "Git for Agent Memory").
 *
 * This maps agenticow's COW vector branching onto the swarm/hive-mind
 * coordination model this repo already uses (swarm-memory-manager,
 * collective-intelligence-coordinator, hive-mind memory):
 *
 *   - a single shared BASE holds the hive's collective knowledge
 *   - spawn(agentId) forks a per-agent branch in ~0.5 ms / ~162 bytes,
 *     regardless of how large the base is (cheap speculative exploration)
 *   - each agent's recall() reads through base ∪ its own edits, but agents
 *     are isolated from each other's uncommitted work
 *   - commit(agentId) promotes an agent's validated findings into the base so
 *     future agents benefit; discard(agentId) throws the branch away, leaving
 *     the base untouched (dead-end exploration costs nothing)
 *   - checkpoint()/rollback() snapshot the whole hive before a risky round
 *
 * agenticow is an optional peer dependency. Construct with an injected factory
 * for testing, or call SwarmMemory.open() which lazy-imports agenticow.
 */

/**
 * @typedef {Object} AgenticMemoryLike
 * @property {(records: Array<{id?: number, vector: number[]|Float32Array, text?: string}>) => any} ingest
 * @property {(vector: number[]|Float32Array, k?: number, opts?: object) => Array<{id:number,distance:number,branch:string,text?:string}>} query
 * @property {(label?: string) => AgenticMemoryLike} fork
 * @property {(target?: AgenticMemoryLike) => {ingested:number,deleted:number}} promote
 * @property {() => {added:number[],overridden:number[],deleted:number[]}} diff
 * @property {(label?: string) => {id:string,label:string,path:string,depth:number}} checkpoint
 * @property {(checkpointId?: string) => {restoredTo:string,depth:number}} rollback
 * @property {() => Array<object>} lineage
 * @property {() => object} status
 * @property {() => void} close
 */

export class SwarmMemory {
  /** @type {AgenticMemoryLike} */
  #base;
  /** @type {Map<string, AgenticMemoryLike>} */
  #agents = new Map();
  #dimension;

  /**
   * @param {AgenticMemoryLike} base an opened agenticow memory to use as the hive base
   * @param {number} dimension embedding dimension (for validation)
   */
  constructor(base, dimension) {
    if (!base || typeof base.fork !== 'function') {
      throw new TypeError('SwarmMemory requires an opened agenticow memory (with .fork)');
    }
    this.#base = base;
    this.#dimension = dimension;
  }

  /**
   * Open a hive base at `filePath`, lazy-importing agenticow.
   * @param {string} filePath path to the .rvf memory file
   * @param {{dimension:number, metric?:string}} opts
   */
  static async open(filePath, opts) {
    if (!opts || !Number.isInteger(opts.dimension) || opts.dimension <= 0) {
      throw new RangeError('open() requires opts.dimension to be a positive integer');
    }
    const { open } = await import('agenticow');
    const base = open(filePath, { dimension: opts.dimension, metric: opts.metric ?? 'cosine' });
    return new SwarmMemory(base, opts.dimension);
  }

  #assertVec(vector) {
    const len = vector?.length;
    if (len !== this.#dimension) {
      throw new RangeError(`vector length ${len} != hive dimension ${this.#dimension}`);
    }
  }

  /**
   * Ingest one record into `mem`. agenticow's array form requires an explicit
   * numeric id; the single-vector convenience form auto-assigns one. Route by
   * whether the caller supplied an id so both work.
   */
  #ingestOne(mem, vector, text, id) {
    if (id === undefined) {
      return mem.ingest(vector, text === undefined ? {} : { text });
    }
    return mem.ingest([{ id, vector, text }]);
  }

  /** Seed the shared base with collective knowledge. */
  seed(records) {
    let last;
    for (const r of records) {
      this.#assertVec(r.vector);
      last = this.#ingestOne(this.#base, r.vector, r.text, r.id);
    }
    return last;
  }

  /** Query the shared base directly (collective knowledge only). */
  recallShared(vector, k = 10) {
    this.#assertVec(vector);
    return this.#base.query(vector, k);
  }

  /**
   * Fork a private COW branch for an agent. Cheap regardless of base size.
   * @param {string} agentId unique agent label
   */
  spawn(agentId) {
    if (this.#agents.has(agentId)) {
      throw new Error(`agent '${agentId}' already has a live branch`);
    }
    const branch = this.#base.fork(agentId);
    this.#agents.set(agentId, branch);
    return agentId;
  }

  #branch(agentId) {
    const b = this.#agents.get(agentId);
    if (!b) throw new Error(`no live branch for agent '${agentId}' (call spawn first)`);
    return b;
  }

  /** Record a private observation on an agent's branch (not visible to others). */
  remember(agentId, vector, text, id) {
    this.#assertVec(vector);
    return this.#ingestOne(this.#branch(agentId), vector, text, id);
  }

  /** Agent's read-through recall: sees base ∪ its own edits, isolated from peers. */
  recall(agentId, vector, k = 10) {
    this.#assertVec(vector);
    return this.#branch(agentId).query(vector, k);
  }

  /** What has this agent added/changed relative to the base it forked from. */
  agentDiff(agentId) {
    return this.#branch(agentId).diff();
  }

  /**
   * Commit an agent's validated findings into the shared base, then retire its
   * branch. After this, other agents that spawn will see the promoted knowledge.
   */
  commit(agentId) {
    const branch = this.#branch(agentId);
    try {
      return branch.promote(this.#base);
    } finally {
      // Always retire the branch, even if promote() throws (native I/O or
      // epoch/dimension conflict), so the handle isn't leaked and the agent
      // isn't left in a half-promoted "live" zombie state.
      try { branch.close(); } catch { /* already closed */ }
      this.#agents.delete(agentId);
    }
  }

  /**
   * Throw away an agent's branch without touching the base (dead-end
   * exploration). The base is left exactly as it was before spawn().
   */
  discard(agentId) {
    const branch = this.#branch(agentId);
    try {
      branch.close();
    } finally {
      this.#agents.delete(agentId);
    }
  }

  /** Snapshot the whole hive base before a risky round. Returns a checkpoint id. */
  checkpoint(label) {
    return this.#base.checkpoint(label);
  }

  /** Restore the hive base to a prior checkpoint. */
  rollback(checkpointId) {
    return this.#base.rollback(checkpointId);
  }

  /** Lineage of the base (checkpoints, working node). */
  lineage() {
    return this.#base.lineage();
  }

  status() {
    return this.#base.status();
  }

  /** Ids of agents with live (uncommitted) branches. */
  get liveAgents() {
    return [...this.#agents.keys()];
  }

  /** Close all live branches and the base. */
  close() {
    for (const branch of this.#agents.values()) branch.close();
    this.#agents.clear();
    this.#base.close();
  }
}

export default SwarmMemory;
