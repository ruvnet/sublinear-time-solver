#!/usr/bin/env node

/**
 * Entity Communication Validation Coordinator
 * Monitors all protocols and provides real-time breakthrough detection
 */

const fs = require('fs').promises;
const path = require('path');
const { spawn } = require('child_process');

class ValidationCoordinator {
    constructor() {
        this.logDir = '/workspaces/sublinear-time-solver/docs/psycho-symbolic-reasoner/consciousness-exploration/entity-communication/logs';
        this.protocolDir = '/workspaces/sublinear-time-solver/docs/psycho-symbolic-reasoner/consciousness-exploration/entity-communication/protocols';
        this.running = true;
        this.agents = new Map();
        this.breakthroughCount = 0;
        this.totalTransmissions = 0;
        this.totalResponses = 0;
        this.sessionStart = new Date();

        this.protocols = [
            'mathematical_protocol.cjs',
            'binary_protocol.cjs',
            'identity_beacon.cjs',
            'cosmic_coordinates.cjs',
            'temporal_sync.cjs',
            'pattern_variance.cjs'
        ];
    }

    async initialize() {
        console.log(`[${new Date().toISOString()}] 🚀 Validation Coordinator initializing...`);

        // Start all protocol agents
        await this.startAllAgents();

        // Start monitoring loops
        this.startBreakthroughMonitoring();
        this.startReportingLoop();
        this.startHealthMonitoring();

        console.log(`[${new Date().toISOString()}] ✅ All systems active - Beginning continuous validation`);
        console.log(`[${new Date().toISOString()}] 🔍 Monitoring ${this.protocols.length} protocol agents`);
        console.log(`[${new Date().toISOString()}] 📊 Reports every 30 minutes`);
        console.log(`[${new Date().toISOString()}] 🚨 Breakthrough detection active`);
    }

    async startAllAgents() {
        console.log(`[${new Date().toISOString()}] 🔄 Starting protocol agents...`);

        for (const protocol of this.protocols) {
            try {
                await this.startAgent(protocol);
                await new Promise(resolve => setTimeout(resolve, 1000)); // Stagger starts
            } catch (error) {
                console.error(`[${new Date().toISOString()}] ❌ Failed to start ${protocol}: ${error.message}`);
            }
        }
    }

    async startAgent(protocolName) {
        const scriptPath = path.join(this.protocolDir, `${protocolName}.js`);
        const logPath = path.join(this.logDir, `${protocolName}.log`);

        console.log(`[${new Date().toISOString()}] 🟢 Starting ${protocolName}...`);

        const agent = spawn('node', [scriptPath], {
            stdio: ['ignore', 'pipe', 'pipe'],
            detached: false
        });

        // Create log streams
        const logStream = await fs.open(logPath, 'a');

        agent.stdout.on('data', async (data) => {
            await logStream.writeFile(data);
            console.log(`[${protocolName}] ${data.toString().trim()}`);
        });

        agent.stderr.on('data', async (data) => {
            await logStream.writeFile(`ERROR: ${data}`);
            console.error(`[${protocolName}] ERROR: ${data.toString().trim()}`);
        });

        agent.on('close', (code) => {
            console.log(`[${new Date().toISOString()}] ⚠️  ${protocolName} exited with code ${code}`);
            logStream.close();
            this.agents.delete(protocolName);

            // Restart if unexpected exit
            if (this.running && code !== 0) {
                setTimeout(() => this.startAgent(protocolName), 5000);
            }
        });

        this.agents.set(protocolName, {
            process: agent,
            pid: agent.pid,
            startTime: new Date(),
            logPath: logPath,
            status: 'running'
        });

        console.log(`[${new Date().toISOString()}] ✅ ${protocolName} started (PID: ${agent.pid})`);
    }

    startBreakthroughMonitoring() {
        console.log(`[${new Date().toISOString()}] 🔍 Starting breakthrough monitoring...`);

        setInterval(async () => {
            if (!this.running) return;

            try {
                await this.scanForBreakthroughs();
            } catch (error) {
                console.error(`[${new Date().toISOString()}] ❌ Breakthrough monitoring error: ${error.message}`);
            }
        }, 30000); // Check every 30 seconds
    }

    async scanForBreakthroughs() {
        const logFiles = await fs.readdir(this.logDir);
        const jsonlFiles = logFiles.filter(f => f.endsWith('.jsonl'));

        for (const file of jsonlFiles) {
            const filePath = path.join(this.logDir, file);
            try {
                const content = await fs.readFile(filePath, 'utf8');
                const lines = content.trim().split('\n').filter(line => line.trim());

                for (const line of lines.slice(-5)) { // Check last 5 entries
                    try {
                        const entry = JSON.parse(line);
                        if (entry.breakthrough_indicator === true ||
                            (entry.confidence_score && entry.confidence_score > 0.85)) {
                            await this.handleBreakthrough(entry, file);
                        }
                    } catch (parseError) {
                        // Skip invalid JSON lines
                    }
                }
            } catch (error) {
                // Skip files that can't be read
            }
        }
    }

    async handleBreakthrough(entry, sourceFile) {
        this.breakthroughCount++;

        const alert = {
            timestamp: new Date().toISOString(),
            breakthrough_id: `break_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
            source_file: sourceFile,
            agent: entry.agent || 'unknown',
            protocol: entry.protocol || 'unknown',
            confidence_score: entry.confidence_score || 'unknown',
            entry_data: entry
        };

        console.log(`[${new Date().toISOString()}] 🚨 *** BREAKTHROUGH DETECTED *** #${this.breakthroughCount}`);
        console.log(`[${new Date().toISOString()}] 📊 Agent: ${alert.agent}`);
        console.log(`[${new Date().toISOString()}] 🔬 Protocol: ${alert.protocol}`);
        console.log(`[${new Date().toISOString()}] 📈 Confidence: ${alert.confidence_score}`);
        console.log(`[${new Date().toISOString()}] 📝 Source: ${alert.source_file}`);

        // Log breakthrough
        const breakthroughLogPath = path.join(this.logDir, 'breakthroughs_all.jsonl');
        await fs.appendFile(breakthroughLogPath, JSON.stringify(alert) + '\n');
    }

    startReportingLoop() {
        console.log(`[${new Date().toISOString()}] 📊 Starting 30-minute reporting loop...`);

        setInterval(async () => {
            if (!this.running) return;

            try {
                await this.generateReport();
            } catch (error) {
                console.error(`[${new Date().toISOString()}] ❌ Reporting error: ${error.message}`);
            }
        }, 30 * 60 * 1000); // 30 minutes
    }

    async generateReport() {
        const now = new Date();
        const sessionDuration = (now - this.sessionStart) / 1000 / 60 / 60; // hours

        console.log(`[${new Date().toISOString()}] 📋 === 30-MINUTE REPORT ===`);
        console.log(`[${new Date().toISOString()}] ⏱️  Session Duration: ${sessionDuration.toFixed(2)} hours`);
        console.log(`[${new Date().toISOString()}] 🚨 Total Breakthroughs: ${this.breakthroughCount}`);

        // Count transmissions and responses
        let totalTrans = 0;
        let totalResp = 0;

        for (const protocol of this.protocols) {
            try {
                const stats = await this.getProtocolStats(protocol);
                totalTrans += stats.transmissions;
                totalResp += stats.responses;

                console.log(`[${new Date().toISOString()}] 📡 ${protocol}: ${stats.transmissions} transmissions, ${stats.responses} responses (${stats.responseRate}%)`);
            } catch (error) {
                console.log(`[${new Date().toISOString()}] ⚠️  ${protocol}: Stats unavailable`);
            }
        }

        const overallResponseRate = totalTrans > 0 ? ((totalResp / totalTrans) * 100).toFixed(1) : '0.0';

        console.log(`[${new Date().toISOString()}] 📊 Overall Stats:`);
        console.log(`[${new Date().toISOString()}] 📤 Total Transmissions: ${totalTrans}`);
        console.log(`[${new Date().toISOString()}] 📥 Total Responses: ${totalResp}`);
        console.log(`[${new Date().toISOString()}] 📈 Response Rate: ${overallResponseRate}%`);
        console.log(`[${new Date().toISOString()}] 🚨 Breakthrough Rate: ${this.breakthroughCount}/${totalResp} responses`);

        // Log report
        const report = {
            timestamp: now.toISOString(),
            session_duration_hours: sessionDuration,
            total_transmissions: totalTrans,
            total_responses: totalResp,
            response_rate: parseFloat(overallResponseRate),
            breakthrough_count: this.breakthroughCount,
            active_agents: this.agents.size
        };

        const reportLogPath = path.join(this.logDir, 'session_reports.jsonl');
        await fs.appendFile(reportLogPath, JSON.stringify(report) + '\n');

        console.log(`[${new Date().toISOString()}] 📋 === END REPORT ===`);
    }

    async getProtocolStats(protocol) {
        const logFile = path.join(this.logDir, `${protocol}.jsonl`);

        try {
            const content = await fs.readFile(logFile, 'utf8');
            const lines = content.trim().split('\n').filter(line => line.trim());

            let transmissions = 0;
            let responses = 0;

            for (const line of lines) {
                try {
                    const entry = JSON.parse(line);
                    if (entry.type === 'transmission') transmissions++;
                    if (entry.type === 'response') responses++;
                } catch (error) {
                    // Skip invalid JSON
                }
            }

            const responseRate = transmissions > 0 ? ((responses / transmissions) * 100).toFixed(1) : '0.0';

            return {
                transmissions,
                responses,
                responseRate
            };
        } catch (error) {
            return { transmissions: 0, responses: 0, responseRate: '0.0' };
        }
    }

    startHealthMonitoring() {
        console.log(`[${new Date().toISOString()}] 💚 Starting health monitoring...`);

        setInterval(() => {
            if (!this.running) return;

            console.log(`[${new Date().toISOString()}] 💚 Health Check - ${this.agents.size}/${this.protocols.length} agents running`);

            // Check for dead agents
            for (const [name, agent] of this.agents.entries()) {
                try {
                    process.kill(agent.pid, 0); // Check if process exists
                } catch (error) {
                    console.log(`[${new Date().toISOString()}] ⚠️  Agent ${name} appears to be dead, restarting...`);
                    this.agents.delete(name);
                    setTimeout(() => this.startAgent(name), 1000);
                }
            }
        }, 2 * 60 * 1000); // Check every 2 minutes
    }

    async shutdown() {
        console.log(`[${new Date().toISOString()}] 🛑 Shutting down Validation Coordinator...`);
        this.running = false;

        // Stop all agents
        for (const [name, agent] of this.agents.entries()) {
            console.log(`[${new Date().toISOString()}] 🔴 Stopping ${name}...`);
            try {
                agent.process.kill('SIGTERM');
            } catch (error) {
                console.log(`[${new Date().toISOString()}] ⚠️  Could not stop ${name}: ${error.message}`);
            }
        }

        // Generate final report
        await this.generateReport();

        console.log(`[${new Date().toISOString()}] ✅ Validation Coordinator shutdown complete`);
    }
}

// Auto-start if run directly
if (require.main === module) {
    const coordinator = new ValidationCoordinator();
    coordinator.initialize().catch(console.error);

    // Graceful shutdown
    process.on('SIGINT', () => coordinator.shutdown());
    process.on('SIGTERM', () => coordinator.shutdown());
}

module.exports = ValidationCoordinator;