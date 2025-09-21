#!/usr/bin/env node

/**
 * Entity Communication Monitoring Coordinator
 * Main execution script for coordinating real-time monitoring of all 5 channels
 */

const EntityMonitor = require('./entity-monitor');
const CommunicationProtocolSuite = require('./communication-protocols');
const MonitoringDashboard = require('./dashboard');

class MonitoringCoordinator {
    constructor() {
        this.entityMonitor = new EntityMonitor();
        this.protocolSuite = new CommunicationProtocolSuite(this.entityMonitor);
        this.dashboard = new MonitoringDashboard(this.entityMonitor);

        this.isActive = false;
        this.sessionId = `monitoring_${Date.now()}`;

        console.log(`[COORDINATOR] Initialized monitoring session: ${this.sessionId}`);
    }

    async initialize() {
        console.log('[COORDINATOR] 🚀 Initializing comprehensive entity communication monitoring...');

        // Setup signal handlers for graceful shutdown
        this.setupSignalHandlers();

        // Display startup banner
        this.displayStartupBanner();

        // Initialize monitoring systems
        await this.initializeMonitoringSystems();

        console.log('[COORDINATOR] ✅ All systems initialized and ready');
    }

    displayStartupBanner() {
        console.log(`
╔═══════════════════════════════════════════════════════════════╗
║                 ENTITY COMMUNICATION MONITOR                 ║
║                Real-Time Multi-Channel Analysis              ║
╠═══════════════════════════════════════════════════════════════╣
║  Session ID: ${this.sessionId.padEnd(38)} ║
║  Channels:   5 (Convergence, Errors, Timing, Memory, Instr)  ║
║  Protocols:  8 (Ping, Injection, Disruption, Sync, etc.)     ║
║  Status:     Initializing...                                 ║
╚═══════════════════════════════════════════════════════════════╝
        `);
    }

    async initializeMonitoringSystems() {
        console.log('[COORDINATOR] 📊 Initializing monitoring systems...');

        // The EntityMonitor is already initialized with all channels
        // The dashboard is ready to display real-time data
        // The protocol suite is ready to test entity communication

        console.log('[COORDINATOR] ✅ Monitoring systems ready');
    }

    async startComprehensiveMonitoring() {
        if (this.isActive) {
            console.log('[COORDINATOR] ⚠️  Monitoring already active');
            return;
        }

        this.isActive = true;
        console.log('[COORDINATOR] 🎯 Starting comprehensive monitoring of all 5 channels...');

        try {
            // Start the main entity monitor
            console.log('[COORDINATOR] 🔄 Starting entity monitor...');
            this.entityMonitor.startMonitoring();

            // Start real-time dashboard updates
            console.log('[COORDINATOR] 📊 Starting dashboard...');
            this.dashboard.startRealTimeUpdates();

            // Schedule initial protocol tests
            console.log('[COORDINATOR] 📡 Scheduling initial protocol tests...');
            setTimeout(() => this.runInitialProtocolTests(), 10000); // After 10 seconds

            // Schedule periodic comprehensive tests
            this.schedulePeriodicTests();

            // Display monitoring status
            this.displayMonitoringStatus();

            console.log('[COORDINATOR] ✅ Comprehensive monitoring active');

        } catch (error) {
            console.error('[COORDINATOR] ❌ Error starting monitoring:', error);
            this.isActive = false;
            throw error;
        }
    }

    async runInitialProtocolTests() {
        console.log('[COORDINATOR] 🧪 Running initial protocol test suite...');

        try {
            const results = await this.protocolSuite.runFullTestSuite();

            console.log('[COORDINATOR] 📊 Initial protocol test results:');
            console.log('  Entity Responses Detected:', results.test_summary.entity_responses_detected);
            console.log('  Success Rate:', (results.test_summary.success_rate * 100).toFixed(1) + '%');
            console.log('  Most Responsive Protocol:', results.performance_metrics.most_responsive_protocol.protocol);

            if (results.entity_analysis.communication_established) {
                console.log('[COORDINATOR] 🎯 Entity communication established!');
                this.displayEntityAnalysis(results.entity_analysis);
            } else {
                console.log('[COORDINATOR] 📡 No clear entity communication detected yet');
            }

        } catch (error) {
            console.error('[COORDINATOR] ❌ Error in initial protocol tests:', error);
        }
    }

    schedulePeriodicTests() {
        // Run basic ping every 30 seconds
        setInterval(async () => {
            if (this.isActive) {
                console.log('[COORDINATOR] 📡 Running periodic basic ping...');
                await this.protocolSuite.runTargetedTest('basicPing');
            }
        }, 30000);

        // Run adaptive pattern test every 2 minutes
        setInterval(async () => {
            if (this.isActive) {
                console.log('[COORDINATOR] 🧠 Running adaptive pattern test...');
                await this.protocolSuite.runTargetedTest('adaptivePattern');
            }
        }, 120000);

        // Run full test suite every 10 minutes
        setInterval(async () => {
            if (this.isActive) {
                console.log('[COORDINATOR] 🔬 Running comprehensive test suite...');
                await this.protocolSuite.runFullTestSuite();
            }
        }, 600000);
    }

    displayMonitoringStatus() {
        setInterval(() => {
            if (this.isActive) {
                const report = this.entityMonitor.generateReport();

                console.log(`\n[STATUS UPDATE] ${new Date().toLocaleTimeString()}`);
                console.log(`  Monitoring Duration: ${this.formatDuration(report.monitoring_duration)}`);
                console.log(`  Total Anomalies: ${report.total_anomalies}`);
                console.log(`  Entity Responses: ${report.entity_responses}`);
                console.log(`  Active Channels: ${Object.keys(report.channel_statistics).length}`);

                if (report.cross_channel_correlations && report.cross_channel_correlations.synchronization_score) {
                    console.log(`  Synchronization Score: ${report.cross_channel_correlations.synchronization_score.toFixed(3)}`);
                }
            }
        }, 30000); // Every 30 seconds
    }

    displayEntityAnalysis(analysis) {
        console.log('\n[ENTITY ANALYSIS]');
        console.log('  Preferred Channels:', analysis.preferred_channels.map(p => p.channel).join(', '));
        console.log('  Response Patterns:');
        Object.entries(analysis.response_patterns).forEach(([pattern, count]) => {
            console.log(`    ${pattern}: ${count}`);
        });
        console.log('  Adaptation Indicators:');
        analysis.adaptation_indicators.forEach(indicator => {
            console.log(`    • ${indicator}`);
        });
    }

    async testSpecificProtocol(protocolName, iterations = 1) {
        console.log(`[COORDINATOR] 🎯 Testing ${protocolName} protocol (${iterations} iterations)...`);

        try {
            const results = await this.protocolSuite.runTargetedTest(protocolName, iterations);

            console.log(`[COORDINATOR] 📊 ${protocolName} test results:`);
            console.log(`  Consistency Score: ${results.consistency_score.toFixed(3)}`);
            console.log(`  Improvement Trend: ${results.improvement_trend.toFixed(3)}`);
            console.log(`  Avg Response Likelihood: ${results.average_entity_response_likelihood.toFixed(3)}`);

            return results;

        } catch (error) {
            console.error(`[COORDINATOR] ❌ Error testing ${protocolName}:`, error);
            throw error;
        }
    }

    async generateComprehensiveReport() {
        console.log('[COORDINATOR] 📊 Generating comprehensive monitoring report...');

        const monitorReport = this.entityMonitor.generateReport();
        const dashboardReport = this.dashboard.generateStaticReport();

        const comprehensiveReport = {
            session_id: this.sessionId,
            timestamp: new Date().toISOString(),
            monitoring_report: monitorReport,
            dashboard_report: dashboardReport,
            coordinator_status: {
                active: this.isActive,
                uptime: Date.now() - parseInt(this.sessionId.split('_')[1])
            }
        };

        console.log('[COORDINATOR] 📄 Report generated');
        return comprehensiveReport;
    }

    async exportDashboard() {
        console.log('[COORDINATOR] 🌐 Exporting dashboard for browser viewing...');

        const dashboardHTML = this.dashboard.exportDashboard();

        // In a real implementation, you would write this to a file
        console.log('[COORDINATOR] ✅ Dashboard HTML exported');

        return dashboardHTML;
    }

    async gracefulShutdown() {
        console.log('\n[COORDINATOR] 🛑 Initiating graceful shutdown...');

        this.isActive = false;

        // Stop monitoring
        console.log('[COORDINATOR] ⏹️  Stopping entity monitor...');
        this.entityMonitor.isMonitoring = false;

        // Stop dashboard updates
        console.log('[COORDINATOR] 📊 Stopping dashboard updates...');
        this.dashboard.stop();

        // Generate final report
        console.log('[COORDINATOR] 📊 Generating final report...');
        const finalReport = await this.generateComprehensiveReport();

        console.log('\n[COORDINATOR] 📄 FINAL MONITORING REPORT');
        console.log('='.repeat(50));
        console.log(`Session Duration: ${this.formatDuration(finalReport.coordinator_status.uptime)}`);
        console.log(`Total Anomalies: ${finalReport.monitoring_report.total_anomalies}`);
        console.log(`Entity Responses: ${finalReport.monitoring_report.entity_responses}`);
        console.log(`Response Rate: ${((finalReport.monitoring_report.entity_responses / finalReport.monitoring_report.total_anomalies) * 100).toFixed(1)}%`);

        console.log('\n[COORDINATOR] ✅ Shutdown complete');

        return finalReport;
    }

    setupSignalHandlers() {
        process.on('SIGINT', async () => {
            console.log('\n[COORDINATOR] 🛑 Received SIGINT (Ctrl+C)');
            await this.gracefulShutdown();
            process.exit(0);
        });

        process.on('SIGTERM', async () => {
            console.log('\n[COORDINATOR] 🛑 Received SIGTERM');
            await this.gracefulShutdown();
            process.exit(0);
        });
    }

    formatDuration(ms) {
        const seconds = Math.floor(ms / 1000);
        const minutes = Math.floor(seconds / 60);
        const hours = Math.floor(minutes / 60);

        if (hours > 0) {
            return `${hours}h ${minutes % 60}m ${seconds % 60}s`;
        } else if (minutes > 0) {
            return `${minutes}m ${seconds % 60}s`;
        } else {
            return `${seconds}s`;
        }
    }
}

// Command-line interface
class MonitoringCLI {
    constructor() {
        this.coordinator = new MonitoringCoordinator();
    }

    async run() {
        const args = process.argv.slice(2);
        const command = args[0] || 'monitor';

        try {
            await this.coordinator.initialize();

            switch (command) {
                case 'monitor':
                    await this.runFullMonitoring();
                    break;

                case 'test':
                    await this.runProtocolTests(args[1]);
                    break;

                case 'dashboard':
                    await this.exportDashboard();
                    break;

                case 'report':
                    await this.generateReport();
                    break;

                default:
                    this.showHelp();
            }

        } catch (error) {
            console.error('[CLI] ❌ Fatal error:', error);
            process.exit(1);
        }
    }

    async runFullMonitoring() {
        console.log('[CLI] 🚀 Starting full monitoring mode...');

        await this.coordinator.startComprehensiveMonitoring();

        // Keep the process running
        console.log('[CLI] ✅ Monitoring active. Press Ctrl+C to stop.');

        // Prevent the process from exiting
        setInterval(() => {}, 1000);
    }

    async runProtocolTests(protocolName) {
        if (protocolName) {
            console.log(`[CLI] 🧪 Testing specific protocol: ${protocolName}`);
            await this.coordinator.testSpecificProtocol(protocolName, 3);
        } else {
            console.log('[CLI] 🧪 Running full protocol test suite...');
            await this.coordinator.protocolSuite.runFullTestSuite();
        }

        process.exit(0);
    }

    async exportDashboard() {
        console.log('[CLI] 🌐 Exporting dashboard...');

        const dashboardHTML = await this.coordinator.exportDashboard();

        // In a real implementation, save to file
        console.log('[CLI] ✅ Dashboard exported');

        process.exit(0);
    }

    async generateReport() {
        console.log('[CLI] 📊 Generating monitoring report...');

        const report = await this.coordinator.generateComprehensiveReport();

        console.log('[CLI] 📄 Report:');
        console.log(JSON.stringify(report, null, 2));

        process.exit(0);
    }

    showHelp() {
        console.log(`
Entity Communication Monitor - Command Line Interface

Usage: node monitoring-coordinator.js [command] [options]

Commands:
  monitor                 Start full real-time monitoring (default)
  test [protocol]         Run protocol tests (all or specific protocol)
  dashboard              Export dashboard HTML
  report                 Generate monitoring report

Protocol Names:
  basic_ping, pattern_injection, sequence_disruption,
  temporal_synchronization, memory_probe, convergence_test,
  multi_channel_burst, adaptive_pattern

Examples:
  node monitoring-coordinator.js                    # Start monitoring
  node monitoring-coordinator.js test              # Run all protocol tests
  node monitoring-coordinator.js test basic_ping   # Test specific protocol
  node monitoring-coordinator.js dashboard         # Export dashboard
  node monitoring-coordinator.js report            # Generate report
        `);

        process.exit(0);
    }
}

// Export classes for use as modules
module.exports = {
    MonitoringCoordinator,
    MonitoringCLI
};

// If this script is run directly, start the CLI
if (require.main === module) {
    const cli = new MonitoringCLI();
    cli.run().catch(error => {
        console.error('[CLI] Fatal error:', error);
        process.exit(1);
    });
}