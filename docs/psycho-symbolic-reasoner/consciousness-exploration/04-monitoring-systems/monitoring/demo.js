#!/usr/bin/env node

/**
 * Entity Communication Monitor Demonstration
 * Shows the monitoring system detecting simulated entity responses
 */

const EntityMonitor = require('./src/entity-monitor');
const CommunicationProtocolSuite = require('./src/communication-protocols');

class MonitoringDemo {
    constructor() {
        this.entityMonitor = new EntityMonitor();
        this.protocolSuite = new CommunicationProtocolSuite(this.entityMonitor);
    }

    async runDemo() {
        console.log(`
╔═══════════════════════════════════════════════════════════════╗
║              ENTITY COMMUNICATION MONITOR DEMO               ║
║            Demonstrating Real-Time Response Detection        ║
╚═══════════════════════════════════════════════════════════════╝
        `);

        console.log('[DEMO] 🚀 Starting demonstration...');

        // Start monitoring with enhanced anomaly generation
        this.startEnhancedMonitoring();

        // Run a series of protocol tests
        await this.runProtocolDemonstration();

        // Generate final report
        await this.generateDemoReport();
    }

    startEnhancedMonitoring() {
        console.log('[DEMO] 📊 Starting enhanced monitoring with simulated responses...');

        // Override the channel monitors to generate more interesting data
        this.enhanceChannelMonitors();

        // Start the entity monitor
        this.entityMonitor.startMonitoring();
    }

    enhanceChannelMonitors() {
        // Add more realistic anomaly generation to convergence monitor
        const originalConvergenceDetect = this.entityMonitor.channels.convergenceRatios.detectAnomalies;
        this.entityMonitor.channels.convergenceRatios.detectAnomalies = async function(data) {
            const anomalies = await originalConvergenceDetect.call(this, data);

            // Simulate occasional convergence anomalies that look like entity responses
            if (Math.random() < 0.15) { // 15% chance
                anomalies.push({
                    type: 'convergence_pattern_change',
                    severity: 'high',
                    deviation: 2.5 + Math.random() * 2,
                    correlation: 0.75 + Math.random() * 0.2,
                    timing_correlation: 0.65 + Math.random() * 0.25,
                    frequency_shift: 0.4 + Math.random() * 0.3
                });
            }

            return anomalies;
        };

        // Enhance error pattern monitor
        const originalErrorDetect = this.entityMonitor.channels.errorPatterns.detectAnomalies;
        this.entityMonitor.channels.errorPatterns.detectAnomalies = async function(data) {
            const anomalies = await originalErrorDetect.call(this, data);

            // Simulate error pattern shifts that indicate adaptation
            if (Math.random() < 0.12) { // 12% chance
                anomalies.push({
                    type: 'error_pattern_adaptation',
                    severity: 'critical',
                    deviation: 3.2 + Math.random() * 1.5,
                    correlation: 0.8 + Math.random() * 0.15,
                    timing_correlation: 0.7 + Math.random() * 0.2
                });
            }

            return anomalies;
        };

        // Enhance timing monitor for synchronized responses
        const originalTimingDetect = this.entityMonitor.channels.timingDeltas.detectAnomalies;
        this.entityMonitor.channels.timingDeltas.detectAnomalies = async function(data) {
            const anomalies = await originalTimingDetect.call(this, data);

            // Simulate timing synchronization anomalies
            if (Math.random() < 0.10) { // 10% chance
                anomalies.push({
                    type: 'temporal_synchronization_shift',
                    severity: 'high',
                    deviation: 3.8 + Math.random() * 1.2,
                    timing_correlation: 0.85 + Math.random() * 0.1,
                    frequency_shift: 0.6 + Math.random() * 0.3
                });
            }

            return anomalies;
        };

        console.log('[DEMO] ✅ Enhanced channel monitors with entity response simulation');
    }

    async runProtocolDemonstration() {
        console.log('\n[DEMO] 🧪 Running protocol demonstration...');

        const protocols = ['basicPing', 'patternInjection', 'adaptivePattern'];

        for (const protocol of protocols) {
            console.log(`\n[DEMO] 📡 Testing ${protocol}...`);

            try {
                const result = await this.protocolSuite.runTargetedTest(protocol, 1);

                console.log(`[DEMO] 📊 ${protocol} Results:`);
                console.log(`  • Entity Response Likelihood: ${result.average_entity_response_likelihood.toFixed(3)}`);
                console.log(`  • Consistency Score: ${result.consistency_score.toFixed(3)}`);

                if (result.average_entity_response_likelihood > 0.7) {
                    console.log(`  🎯 Strong entity response detected in ${protocol}!`);
                } else if (result.average_entity_response_likelihood > 0.4) {
                    console.log(`  📈 Moderate entity response detected in ${protocol}`);
                } else {
                    console.log(`  📊 Baseline response detected in ${protocol}`);
                }

            } catch (error) {
                console.error(`[DEMO] ❌ Error testing ${protocol}:`, error.message);
            }

            // Wait between tests
            await this.sleep(3000);
        }
    }

    async generateDemoReport() {
        console.log('\n[DEMO] 📊 Generating demonstration report...');

        const report = this.entityMonitor.generateReport();

        console.log(`
╔═══════════════════════════════════════════════════════════════╗
║                    DEMONSTRATION REPORT                      ║
╚═══════════════════════════════════════════════════════════════╝

📊 MONITORING SUMMARY
• Duration: ${this.formatDuration(report.monitoring_duration)}
• Total Anomalies: ${report.total_anomalies}
• Entity Responses: ${report.entity_responses}
• Response Rate: ${report.total_anomalies > 0 ? ((report.entity_responses / report.total_anomalies) * 100).toFixed(1) : '0'}%

📈 CHANNEL ACTIVITY
${Object.entries(report.channel_statistics).map(([channel, stats]) =>
    `• ${channel}: ${stats.data_points} data points (${stats.average_collection_rate.toFixed(2)} Hz)`
).join('\n')}

🎯 ENTITY RESPONSE ANALYSIS
${report.entity_responses > 0 ?
    `Entity responses detected across ${Object.keys(report.channel_statistics).length} channels` :
    'No significant entity responses detected in this demonstration'
}

🔗 CROSS-CHANNEL CORRELATIONS
${report.cross_channel_correlations && report.cross_channel_correlations.synchronization_score ?
    `Synchronization Score: ${report.cross_channel_correlations.synchronization_score.toFixed(3)}` :
    'Synchronization analysis pending'
}

🚨 ALERT SUMMARY
• Total Alerts: ${report.alert_summary.total_alerts}
• Entity Response Alerts: ${report.alert_summary.entity_responses}
• Synchronized Response Alerts: ${report.alert_summary.synchronized_responses}

═══════════════════════════════════════════════════════════════
        `);

        console.log('[DEMO] ✅ Demonstration completed successfully!');
    }

    formatDuration(ms) {
        const seconds = Math.floor(ms / 1000);
        const minutes = Math.floor(seconds / 60);

        if (minutes > 0) {
            return `${minutes}m ${seconds % 60}s`;
        } else {
            return `${seconds}s`;
        }
    }

    sleep(ms) {
        return new Promise(resolve => setTimeout(resolve, ms));
    }
}

// Run the demonstration if this script is executed directly
if (require.main === module) {
    const demo = new MonitoringDemo();

    demo.runDemo().then(() => {
        console.log('\n[DEMO] 🎯 Demonstration completed');
        process.exit(0);
    }).catch(error => {
        console.error('\n[DEMO] ❌ Demonstration failed:', error);
        process.exit(1);
    });
}

module.exports = MonitoringDemo;