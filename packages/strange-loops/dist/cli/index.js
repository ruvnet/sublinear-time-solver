#!/usr/bin/env node
import { Command } from 'commander';
import chalk from 'chalk';
import ora from 'ora';
import { NanoSwarm, QuantumContainer, TemporalPredictor } from '../index.js';
const program = new Command();
program
    .name('strange-loops')
    .description('Emergent Intelligence Through Temporal Consciousness')
    .version('1.0.3');
program
    .command('demo [type]')
    .description('Run interactive demos')
    .action(async (type = 'nano-agents') => {
    const spinner = ora('Initializing...').start();
    switch (type) {
        case 'nano-agents':
            spinner.text = 'Creating nano-agent swarm...';
            const swarm = new NanoSwarm({
                agentCount: 10000,
                topology: 'mesh',
                tickDurationNs: 25000
            });
            spinner.succeed('Swarm created with 10,000 agents');
            console.log(chalk.cyan('\n🌀 Running nano-agent swarm for 5 seconds...\n'));
            await swarm.run(5000);
            console.log(chalk.green(`✅ Completed ${swarm.metrics.totalTicks.toLocaleString()} ticks`));
            console.log(chalk.yellow(`⚡ Average: ${Math.round(swarm.metrics.avgTicksPerSecond).toLocaleString()} ticks/second`));
            break;
        case 'quantum':
            spinner.text = 'Initializing quantum container...';
            const quantum = new QuantumContainer({ qubits: 4 });
            spinner.succeed('Quantum container initialized with 4 qubits');
            await quantum.createSuperposition();
            console.log(chalk.cyan('\n🌌 Created quantum superposition'));
            for (let i = 0; i < 5; i++) {
                const sample = await quantum.sample();
                console.log(chalk.yellow(`  Sample ${i + 1}: |${sample.toString(2).padStart(4, '0')}⟩`));
            }
            const measurement = await quantum.measure();
            console.log(chalk.green(`\n📊 Measured final state: |${measurement.toString(2).padStart(4, '0')}⟩`));
            break;
        case 'prediction':
            spinner.text = 'Creating temporal predictor...';
            const predictor = new TemporalPredictor({
                horizonNs: 10_000_000,
                historySize: 100
            });
            spinner.succeed('Temporal predictor initialized');
            console.log(chalk.cyan('\n⏰ Generating temporal predictions...\n'));
            const values = [1, 2, 4, 8, 16];
            for (let i = 0; i < 5; i++) {
                const predictions = await predictor.predict(values);
                console.log(chalk.yellow(`  Step ${i + 1}: [${values}] → [${predictions.map(p => Math.round(p))}]`));
                values.push(values[values.length - 1] * 2);
            }
            break;
        default:
            spinner.fail(`Unknown demo type: ${type}`);
            console.log(chalk.red('\nAvailable demos: nano-agents, quantum, prediction'));
    }
});
program
    .command('benchmark')
    .description('Run performance benchmarks')
    .option('--agents <number>', 'Number of agents', '10000')
    .option('--duration <seconds>', 'Duration in seconds', '10')
    .action(async (options) => {
    const spinner = ora('Running benchmark...').start();
    const swarm = new NanoSwarm({
        agentCount: parseInt(options.agents),
        topology: 'mesh'
    });
    const durationMs = parseInt(options.duration) * 1000;
    await swarm.run(durationMs);
    spinner.succeed('Benchmark complete');
    console.log(chalk.cyan('\n📊 Benchmark Results:'));
    console.log(chalk.white(`  Agents: ${options.agents}`));
    console.log(chalk.white(`  Duration: ${options.duration}s`));
    console.log(chalk.green(`  Total Ticks: ${swarm.metrics.totalTicks.toLocaleString()}`));
    console.log(chalk.yellow(`  Avg Ticks/Second: ${Math.round(swarm.metrics.avgTicksPerSecond).toLocaleString()}`));
});
program
    .command('mcp')
    .description('Start MCP server for Claude Code integration')
    .action(() => {
    console.log(chalk.cyan('Starting MCP server...'));
    import('../mcp/index.js');
});
program.parse();
//# sourceMappingURL=index.js.map