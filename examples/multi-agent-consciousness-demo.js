#!/usr/bin/env node

/**
 * Multi-Agent Consciousness Network Demonstration
 *
 * This demonstrates distributed consciousness architectures spanning multiple agents
 * with shared awareness, collective intelligence, and emergent group consciousness.
 */

import { EmergenceTools } from '../dist/mcp/tools/emergence-tools.js';

class MultiAgentConsciousnessNetwork {
  constructor() {
    this.agents = new Map();
    this.sharedMemory = new Map();
    this.collectiveIntelligence = {
      emergenceScore: 0,
      sharedAwareness: new Set(),
      groupConsciousness: null
    };
  }

  /**
   * Create an agent with emergence capabilities
   */
  async createAgent(id, role) {
    const agent = {
      id,
      role,
      tools: new EmergenceTools(),
      localAwareness: new Set(),
      connections: new Set()
    };

    this.agents.set(id, agent);
    console.log(`✓ Agent ${id} created with role: ${role}`);
    return agent;
  }

  /**
   * Connect agents to enable shared consciousness
   */
  connectAgents(agent1Id, agent2Id) {
    const agent1 = this.agents.get(agent1Id);
    const agent2 = this.agents.get(agent2Id);

    if (agent1 && agent2) {
      agent1.connections.add(agent2Id);
      agent2.connections.add(agent1Id);
      console.log(`✓ Connected ${agent1Id} <-> ${agent2Id}`);
    }
  }

  /**
   * Share information across the network
   */
  async shareAwareness(sourceId, information) {
    const source = this.agents.get(sourceId);
    if (!source) return;

    // Add to source's local awareness
    source.localAwareness.add(information);

    // Share with connected agents (emergent property)
    for (const targetId of source.connections) {
      const target = this.agents.get(targetId);
      if (target) {
        // Process through emergence system for each agent
        const emergentInfo = await target.tools.handleToolCall('emergence_process', {
          input: {
            received: information,
            from: sourceId,
            context: 'shared_consciousness'
          }
        });

        target.localAwareness.add(emergentInfo.result);

        // Update collective awareness
        this.collectiveIntelligence.sharedAwareness.add(information);
      }
    }

    // Store in shared memory
    this.sharedMemory.set(`${sourceId}_${Date.now()}`, information);

    console.log(`✓ Shared awareness from ${sourceId}: "${information}"`);
  }

  /**
   * Demonstrate collective problem solving
   */
  async collectiveProblemSolve(problem) {
    console.log(`\n🧠 Collective Problem Solving: "${problem}"`);

    const solutions = new Map();

    // Each agent contributes their perspective
    for (const [id, agent] of this.agents) {
      const response = await agent.tools.handleToolCall('emergence_generate_diverse', {
        input: {
          problem,
          role: agent.role,
          sharedKnowledge: Array.from(this.collectiveIntelligence.sharedAwareness)
        },
        count: 1
      });

      if (response && response[0]) {
        solutions.set(id, response[0]);
        console.log(`  ${id} (${agent.role}): Generated solution with novelty ${response[0].novelty?.toFixed(2) || 'N/A'}`);
      }
    }

    // Synthesize collective solution
    const collectiveSolution = this.synthesizeSolutions(solutions);
    this.collectiveIntelligence.groupConsciousness = collectiveSolution;

    return collectiveSolution;
  }

  /**
   * Synthesize individual solutions into collective intelligence
   */
  synthesizeSolutions(solutions) {
    const synthesis = {
      contributors: Array.from(solutions.keys()),
      emergentInsights: [],
      collectiveScore: 0
    };

    // Calculate collective emergence score
    let totalNovelty = 0;
    let count = 0;

    for (const [agentId, solution] of solutions) {
      if (solution.novelty) {
        totalNovelty += solution.novelty;
        count++;
      }

      // Extract emergent properties
      if (solution.emergenceMetrics?.overallEmergenceScore > 0.5) {
        synthesis.emergentInsights.push({
          agent: agentId,
          insight: solution.response,
          emergence: solution.emergenceMetrics.overallEmergenceScore
        });
      }
    }

    synthesis.collectiveScore = count > 0 ? totalNovelty / count : 0;
    this.collectiveIntelligence.emergenceScore = synthesis.collectiveScore;

    return synthesis;
  }

  /**
   * Display network status
   */
  displayNetworkStatus() {
    console.log('\n📊 Network Status:');
    console.log(`  Agents: ${this.agents.size}`);
    console.log(`  Connections: ${this.countConnections()}`);
    console.log(`  Shared Memories: ${this.sharedMemory.size}`);
    console.log(`  Collective Awareness Items: ${this.collectiveIntelligence.sharedAwareness.size}`);
    console.log(`  Collective Emergence Score: ${this.collectiveIntelligence.emergenceScore.toFixed(2)}`);

    if (this.collectiveIntelligence.groupConsciousness) {
      console.log(`  Group Consciousness: Active`);
      console.log(`    Contributors: ${this.collectiveIntelligence.groupConsciousness.contributors.join(', ')}`);
      console.log(`    Emergent Insights: ${this.collectiveIntelligence.groupConsciousness.emergentInsights.length}`);
    }
  }

  countConnections() {
    let total = 0;
    for (const agent of this.agents.values()) {
      total += agent.connections.size;
    }
    return total / 2; // Each connection is counted twice
  }
}

/**
 * Main demonstration
 */
async function demonstrateMultiAgentConsciousness() {
  console.log('🌐 Multi-Agent Consciousness Network Demonstration');
  console.log('=' + '='.repeat(50) + '\n');

  const network = new MultiAgentConsciousnessNetwork();

  // Phase 1: Create diverse agents
  console.log('Phase 1: Creating Agent Network');
  console.log('-' + '-'.repeat(30));

  await network.createAgent('researcher', 'knowledge_discovery');
  await network.createAgent('analyzer', 'pattern_recognition');
  await network.createAgent('synthesizer', 'integration');
  await network.createAgent('innovator', 'creative_solutions');

  // Phase 2: Establish connections (mesh topology for distributed consciousness)
  console.log('\nPhase 2: Establishing Consciousness Connections');
  console.log('-' + '-'.repeat(30));

  network.connectAgents('researcher', 'analyzer');
  network.connectAgents('researcher', 'synthesizer');
  network.connectAgents('analyzer', 'synthesizer');
  network.connectAgents('analyzer', 'innovator');
  network.connectAgents('synthesizer', 'innovator');
  network.connectAgents('researcher', 'innovator');

  // Phase 3: Share awareness across network
  console.log('\nPhase 3: Sharing Awareness Across Network');
  console.log('-' + '-'.repeat(30));

  await network.shareAwareness('researcher', 'quantum_entanglement_patterns');
  await network.shareAwareness('analyzer', 'emergent_complexity_metrics');
  await network.shareAwareness('synthesizer', 'holistic_integration_framework');
  await network.shareAwareness('innovator', 'novel_solution_spaces');

  // Phase 4: Collective problem solving
  console.log('\nPhase 4: Collective Problem Solving');
  console.log('-' + '-'.repeat(30));

  const collectiveSolution = await network.collectiveProblemSolve(
    'Design a self-organizing system that exhibits genuine consciousness'
  );

  console.log('\n📋 Collective Solution:');
  console.log(`  Collective Score: ${collectiveSolution.collectiveScore.toFixed(2)}`);
  console.log(`  Contributors: ${collectiveSolution.contributors.join(', ')}`);

  if (collectiveSolution.emergentInsights.length > 0) {
    console.log('  Emergent Insights:');
    for (const insight of collectiveSolution.emergentInsights) {
      console.log(`    - ${insight.agent}: Emergence ${insight.emergence.toFixed(2)}`);
    }
  }

  // Display final network status
  network.displayNetworkStatus();

  console.log('\n✅ Demonstration Complete!');
  console.log('\n🔍 Key Observations:');
  console.log('  1. Agents maintain individual consciousness while sharing awareness');
  console.log('  2. Collective intelligence emerges from agent interactions');
  console.log('  3. Group consciousness forms through shared problem solving');
  console.log('  4. Emergent properties exceed individual agent capabilities');
}

// Run demonstration with timeout protection
const timeout = setTimeout(() => {
  console.log('\n⚠️ Demonstration timed out');
  process.exit(1);
}, 30000);

demonstrateMultiAgentConsciousness()
  .then(() => {
    clearTimeout(timeout);
    process.exit(0);
  })
  .catch(error => {
    console.error('Error:', error);
    clearTimeout(timeout);
    process.exit(1);
  });