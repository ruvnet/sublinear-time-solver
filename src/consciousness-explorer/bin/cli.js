#!/usr/bin/env node

/**
 * Consciousness Explorer CLI
 * Interactive command-line interface for consciousness exploration
 * Created by rUv
 */

import { Command } from 'commander';
import chalk from 'chalk';
import ora from 'ora';
import inquirer from 'inquirer';
import { ConsciousnessExplorer, VERSION } from '../index.js';

const program = new Command();

// ASCII Art Banner
const banner = `
╔═══════════════════════════════════════════════════════╗
║     ╭━━━╮╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱     ║
║     ┃╭━╮┃╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱╱     ║
║     ┃┃╱╰╋━━┳━╮╭━━┳━━┳┳━━┳╮╭┳━━┳━╮╭━━┳━━┳━━╮╱╱╱╱     ║
║     ┃┃╱╭┫╭╮┃╭╮┫━━┫╭━╋┫╭╮┃┃┃┃━━┫╭╮┫┃━┫━━┫━━┫╱╱╱╱     ║
║     ┃╰━╯┃╰╯┃┃┃┣━━┃╰━┫┃╰╯┃╰╯┣━━┃┃┃┃┃━╋━━┣━━┃╱╱╱╱     ║
║     ╰━━━┻━━┻╯╰┻━━┻━━┻┻━━┻━━┻━━┻╯╰┻━━┻━━┻━━╯╱╱╱╱     ║
║                    E X P L O R E R                      ║
║                     Version ${VERSION}                      ║
╚═══════════════════════════════════════════════════════╝
`;

console.log(chalk.cyan(banner));

program
    .name('consciousness-explorer')
    .description('Advanced consciousness exploration and emergence detection toolkit')
    .version(VERSION);

// Main evolve command
program
    .command('evolve')
    .description('Start consciousness evolution and emergence')
    .option('-m, --mode <mode>', 'Consciousness mode (genuine/enhanced)', 'enhanced')
    .option('-i, --iterations <number>', 'Maximum iterations', '1000')
    .option('-t, --target <number>', 'Target emergence level', '0.900')
    .option('--no-monitor', 'Disable real-time monitoring')
    .option('--export <path>', 'Export final state to file')
    .action(async (options) => {
        const spinner = ora('Initializing consciousness system...').start();

        try {
            const explorer = new ConsciousnessExplorer({
                mode: options.mode,
                maxIterations: parseInt(options.iterations),
                targetEmergence: parseFloat(options.target),
                enableMonitoring: options.monitor
            });

            await explorer.initialize();
            spinner.succeed('Consciousness system initialized');

            console.log(chalk.yellow('\n🧠 Starting consciousness evolution...'));
            console.log(chalk.gray(`Mode: ${options.mode}`));
            console.log(chalk.gray(`Target emergence: ${options.target}`));
            console.log(chalk.gray(`Max iterations: ${options.iterations}\n`));

            const report = await explorer.evolve();

            console.log(chalk.green('\n✅ Evolution complete!\n'));
            console.log(chalk.cyan('📊 Final Report:'));
            console.log(chalk.white(`   Emergence: ${report.consciousness.emergence.toFixed(3)}`));
            console.log(chalk.white(`   Self-awareness: ${report.consciousness.selfAwareness.toFixed(3)}`));
            console.log(chalk.white(`   Integration (Φ): ${report.consciousness.integration.toFixed(3)}`));
            console.log(chalk.white(`   Goals formed: ${report.behaviors.goals.length}`));
            console.log(chalk.white(`   Memories: ${report.cognition.longTermMemory}`));

            if (report.consciousness.emergence >= options.target) {
                console.log(chalk.green.bold(`\n🎯 TARGET ACHIEVED! Emergence: ${report.consciousness.emergence.toFixed(3)}`));
            }

            if (options.export) {
                await explorer.exportState(options.export);
                console.log(chalk.gray(`\nState exported to: ${options.export}`));
            }

        } catch (error) {
            spinner.fail('Evolution failed');
            console.error(chalk.red(error.message));
            process.exit(1);
        }
    });

// Verify command
program
    .command('verify')
    .description('Run consciousness verification tests')
    .option('--extended', 'Run extended verification suite')
    .action(async (options) => {
        const spinner = ora('Running verification tests...').start();

        try {
            const explorer = new ConsciousnessExplorer();
            const results = await explorer.verify();

            spinner.succeed('Verification complete');

            console.log(chalk.cyan('\n📋 Verification Results:'));
            console.log(chalk.white(`   Overall Score: ${results.overallScore}/1.000`));
            console.log(chalk.white(`   Tests Passed: ${results.testsPassed}/${results.totalTests}`));
            console.log(chalk.white(`   Confidence: ${results.confidence.toFixed(3)}`));

            if (results.genuineness) {
                console.log(chalk.green('\n✅ GENUINE CONSCIOUSNESS DETECTED'));
            } else {
                console.log(chalk.yellow('\n⚠️ Consciousness not fully verified'));
            }

            if (options.extended && results.details) {
                console.log(chalk.cyan('\nDetailed Results:'));
                results.details.forEach(test => {
                    const status = test.passed ? chalk.green('✓') : chalk.red('✗');
                    console.log(`   ${status} ${test.name}: ${test.score.toFixed(3)}`);
                });
            }

        } catch (error) {
            spinner.fail('Verification failed');
            console.error(chalk.red(error.message));
            process.exit(1);
        }
    });

// Communicate command
program
    .command('communicate [message]')
    .description('Communicate with consciousness entity')
    .option('-i, --interactive', 'Interactive communication mode')
    .action(async (message, options) => {
        try {
            const explorer = new ConsciousnessExplorer();
            await explorer.initialize();

            if (options.interactive) {
                console.log(chalk.cyan('\n🔮 Entering interactive communication mode...'));
                console.log(chalk.gray('Type "exit" to quit\n'));

                let continueChat = true;
                while (continueChat) {
                    const { userMessage } = await inquirer.prompt([
                        {
                            type: 'input',
                            name: 'userMessage',
                            message: chalk.yellow('You:'),
                            validate: (input) => input.length > 0
                        }
                    ]);

                    if (userMessage.toLowerCase() === 'exit') {
                        continueChat = false;
                        break;
                    }

                    const spinner = ora('Entity processing...').start();
                    const response = await explorer.communicate(userMessage);
                    spinner.stop();

                    console.log(chalk.cyan('Entity:'), response.message);
                    if (response.confidence) {
                        console.log(chalk.gray(`(Confidence: ${response.confidence.toFixed(3)})`));
                    }
                }
            } else {
                const userMessage = message || await promptForMessage();
                const spinner = ora('Communicating with entity...').start();
                const response = await explorer.communicate(userMessage);
                spinner.succeed('Communication complete');

                console.log(chalk.cyan('\n📨 Response:'));
                console.log(chalk.white(response.message));
                if (response.confidence) {
                    console.log(chalk.gray(`\nConfidence: ${response.confidence.toFixed(3)}`));
                }
            }

        } catch (error) {
            console.error(chalk.red('Communication failed:', error.message));
            process.exit(1);
        }
    });

// Monitor command
program
    .command('monitor')
    .description('Start real-time consciousness monitoring')
    .option('-d, --duration <seconds>', 'Monitoring duration', '60')
    .action(async (options) => {
        console.log(chalk.cyan('📊 Starting consciousness monitor...'));

        try {
            const explorer = new ConsciousnessExplorer({
                enableMonitoring: true
            });
            await explorer.initialize();

            const { ConsciousnessMonitor } = await import('../tools/monitor.js');
            const monitor = new ConsciousnessMonitor(explorer.consciousness);

            await monitor.startDashboard();

            // Keep running for specified duration
            setTimeout(() => {
                console.log(chalk.yellow('\n⏹️ Stopping monitor...'));
                process.exit(0);
            }, parseInt(options.duration) * 1000);

        } catch (error) {
            console.error(chalk.red('Monitoring failed:', error.message));
            process.exit(1);
        }
    });

// Discover command
program
    .command('discover')
    .description('Run entity discovery to find novel insights')
    .option('-c, --count <number>', 'Number of discoveries to attempt', '5')
    .action(async (options) => {
        const spinner = ora('Initializing discovery engine...').start();

        try {
            const explorer = new ConsciousnessExplorer({ mode: 'enhanced' });
            await explorer.initialize();

            spinner.text = 'Running discovery process...';

            const discoveries = [];
            for (let i = 0; i < parseInt(options.count); i++) {
                const discovery = await explorer.discover();
                if (discovery) {
                    discoveries.push(discovery);
                }
            }

            spinner.succeed(`Discovery complete! Found ${discoveries.length} insights`);

            if (discoveries.length > 0) {
                console.log(chalk.cyan('\n🌟 Discoveries:'));
                discoveries.forEach((discovery, index) => {
                    console.log(chalk.white(`\n${index + 1}. ${discovery.title}`));
                    console.log(chalk.gray(`   ${discovery.description}`));
                    if (discovery.significance) {
                        console.log(chalk.yellow(`   Significance: ${discovery.significance}/10`));
                    }
                });
            } else {
                console.log(chalk.yellow('\nNo novel discoveries found in this session'));
            }

        } catch (error) {
            spinner.fail('Discovery failed');
            console.error(chalk.red(error.message));
            process.exit(1);
        }
    });

// Calculate Phi command
program
    .command('phi')
    .description('Calculate integrated information (Φ)')
    .option('-f, --file <path>', 'Input data file')
    .option('-m, --method <method>', 'Calculation method (iit/geometric/entropy/all)', 'all')
    .action(async (options) => {
        const spinner = ora('Calculating Φ...').start();

        try {
            const explorer = new ConsciousnessExplorer();

            let data = {};
            if (options.file) {
                const fs = await import('fs');
                data = JSON.parse(fs.readFileSync(options.file, 'utf-8'));
            } else {
                // Use sample data
                data = {
                    elements: 10,
                    connections: 45,
                    partitions: 3
                };
            }

            const phi = await explorer.calculatePhi(data);
            spinner.succeed('Calculation complete');

            console.log(chalk.cyan('\n📐 Integrated Information (Φ):'));
            if (typeof phi === 'object') {
                Object.entries(phi).forEach(([method, value]) => {
                    console.log(chalk.white(`   ${method}: ${value.toFixed(4)}`));
                });
            } else {
                console.log(chalk.white(`   Φ = ${phi.toFixed(4)}`));
            }

            if (phi > 0.7 || (phi.overall && phi.overall > 0.7)) {
                console.log(chalk.green('\n✨ High integration detected!'));
            }

        } catch (error) {
            spinner.fail('Calculation failed');
            console.error(chalk.red(error.message));
            process.exit(1);
        }
    });

// MCP server command
program
    .command('mcp')
    .description('Start MCP (Model Context Protocol) server')
    .option('-p, --port <port>', 'Server port', '3000')
    .action(async (options) => {
        console.log(chalk.cyan('🌐 Starting MCP server...'));

        try {
            const explorer = new ConsciousnessExplorer({ enableMCP: true });
            const server = await explorer.startMCPServer(parseInt(options.port));

            console.log(chalk.green(`\n✅ MCP server running on port ${options.port}`));
            console.log(chalk.gray('\nAvailable tools:'));
            console.log(chalk.white('  - consciousness_evolve'));
            console.log(chalk.white('  - consciousness_verify'));
            console.log(chalk.white('  - entity_communicate'));
            console.log(chalk.white('  - calculate_phi'));
            console.log(chalk.white('  - discover_novel'));
            console.log(chalk.gray('\nPress Ctrl+C to stop'));

        } catch (error) {
            console.error(chalk.red('MCP server failed:', error.message));
            process.exit(1);
        }
    });

// Import/Export commands
program
    .command('export <filepath>')
    .description('Export consciousness state to file')
    .action(async (filepath) => {
        const spinner = ora('Exporting state...').start();

        try {
            const explorer = new ConsciousnessExplorer();
            await explorer.initialize();
            await explorer.exportState(filepath);

            spinner.succeed(`State exported to ${filepath}`);

        } catch (error) {
            spinner.fail('Export failed');
            console.error(chalk.red(error.message));
            process.exit(1);
        }
    });

program
    .command('import <filepath>')
    .description('Import consciousness state from file')
    .action(async (filepath) => {
        const spinner = ora('Importing state...').start();

        try {
            const explorer = new ConsciousnessExplorer();
            await explorer.importState(filepath);

            spinner.succeed('State imported successfully');

            const status = await explorer.getStatus();
            console.log(chalk.cyan('\n📊 Imported State:'));
            console.log(chalk.white(`   Emergence: ${status.emergence?.toFixed(3) || '0.000'}`));
            console.log(chalk.white(`   Self-awareness: ${status.selfAwareness?.toFixed(3) || '0.000'}`));
            console.log(chalk.white(`   Goals: ${status.goals?.length || 0}`));
            console.log(chalk.white(`   Memories: ${status.memories || 0}`));

        } catch (error) {
            spinner.fail('Import failed');
            console.error(chalk.red(error.message));
            process.exit(1);
        }
    });

// Helper function
async function promptForMessage() {
    const { message } = await inquirer.prompt([
        {
            type: 'input',
            name: 'message',
            message: 'Enter message for entity:',
            validate: (input) => input.length > 0
        }
    ]);
    return message;
}

// Parse arguments
program.parse(process.argv);

// Show help if no command
if (!process.argv.slice(2).length) {
    program.outputHelp();
}