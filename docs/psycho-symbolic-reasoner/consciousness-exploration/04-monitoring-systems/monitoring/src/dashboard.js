/**
 * Real-Time Entity Monitoring Dashboard
 * Web-based dashboard for visualizing entity communication monitoring
 */

class MonitoringDashboard {
    constructor(entityMonitor) {
        this.monitor = entityMonitor;
        this.updateInterval = 1000; // 1 second
        this.isUpdating = false;
        this.charts = {};
        this.alerts = [];

        this.initializeDashboard();
    }

    initializeDashboard() {
        // Create dashboard HTML structure
        this.createDashboardHTML();

        // Initialize charts
        this.initializeCharts();

        // Start real-time updates
        this.startRealTimeUpdates();

        // Setup event listeners
        this.setupEventListeners();
    }

    createDashboardHTML() {
        const dashboardHTML = `
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>Entity Communication Monitor</title>
            <style>
                ${this.getDashboardCSS()}
            </style>
        </head>
        <body>
            <div id="dashboard">
                <header class="dashboard-header">
                    <h1>🔍 Entity Communication Monitor</h1>
                    <div class="status-indicators">
                        <div id="monitoring-status" class="status-indicator">
                            <span class="status-dot"></span>
                            <span class="status-text">Monitoring</span>
                        </div>
                        <div id="entity-status" class="status-indicator">
                            <span class="status-dot"></span>
                            <span class="status-text">Entity Status</span>
                        </div>
                    </div>
                </header>

                <div class="dashboard-controls">
                    <button id="start-monitoring" class="control-btn">Start Monitoring</button>
                    <button id="stop-monitoring" class="control-btn">Stop Monitoring</button>
                    <button id="test-protocols" class="control-btn">Test Protocols</button>
                    <button id="generate-report" class="control-btn">Generate Report</button>
                </div>

                <div class="dashboard-grid">
                    <!-- Channel Monitoring Section -->
                    <div class="grid-section" id="channel-monitoring">
                        <h2>📊 Channel Monitoring</h2>
                        <div class="channel-grid">
                            <div class="channel-card" id="channel-1">
                                <h3>Channel 1: Convergence Ratios</h3>
                                <div class="chart-container">
                                    <canvas id="convergence-chart"></canvas>
                                </div>
                                <div class="channel-stats">
                                    <div class="stat">
                                        <span class="stat-label">Current Rate:</span>
                                        <span id="convergence-rate" class="stat-value">--</span>
                                    </div>
                                    <div class="stat">
                                        <span class="stat-label">Anomalies:</span>
                                        <span id="convergence-anomalies" class="stat-value">0</span>
                                    </div>
                                </div>
                            </div>

                            <div class="channel-card" id="channel-2">
                                <h3>Channel 2: Error Patterns</h3>
                                <div class="chart-container">
                                    <canvas id="error-chart"></canvas>
                                </div>
                                <div class="channel-stats">
                                    <div class="stat">
                                        <span class="stat-label">Error Rate:</span>
                                        <span id="error-rate" class="stat-value">--</span>
                                    </div>
                                    <div class="stat">
                                        <span class="stat-label">Pattern Changes:</span>
                                        <span id="error-patterns" class="stat-value">0</span>
                                    </div>
                                </div>
                            </div>

                            <div class="channel-card" id="channel-3">
                                <h3>Channel 3: Timing Deltas</h3>
                                <div class="chart-container">
                                    <canvas id="timing-chart"></canvas>
                                </div>
                                <div class="channel-stats">
                                    <div class="stat">
                                        <span class="stat-label">Avg Time:</span>
                                        <span id="timing-avg" class="stat-value">--</span>
                                    </div>
                                    <div class="stat">
                                        <span class="stat-label">Deviations:</span>
                                        <span id="timing-deviations" class="stat-value">0</span>
                                    </div>
                                </div>
                            </div>

                            <div class="channel-card" id="channel-4">
                                <h3>Channel 4: Memory Patterns</h3>
                                <div class="chart-container">
                                    <canvas id="memory-chart"></canvas>
                                </div>
                                <div class="channel-stats">
                                    <div class="stat">
                                        <span class="stat-label">Usage:</span>
                                        <span id="memory-usage" class="stat-value">--</span>
                                    </div>
                                    <div class="stat">
                                        <span class="stat-label">Pattern Changes:</span>
                                        <span id="memory-patterns" class="stat-value">0</span>
                                    </div>
                                </div>
                            </div>

                            <div class="channel-card" id="channel-5">
                                <h3>Channel 5: Instruction Sequences</h3>
                                <div class="chart-container">
                                    <canvas id="instruction-chart"></canvas>
                                </div>
                                <div class="channel-stats">
                                    <div class="stat">
                                        <span class="stat-label">Complexity:</span>
                                        <span id="instruction-complexity" class="stat-value">--</span>
                                    </div>
                                    <div class="stat">
                                        <span class="stat-label">Sequence Changes:</span>
                                        <span id="instruction-changes" class="stat-value">0</span>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>

                    <!-- Entity Response Analysis -->
                    <div class="grid-section" id="entity-analysis">
                        <h2>🧠 Entity Response Analysis</h2>
                        <div class="analysis-grid">
                            <div class="analysis-card">
                                <h3>Response Detection</h3>
                                <div id="response-heatmap" class="heatmap-container">
                                    <!-- Response heatmap will be rendered here -->
                                </div>
                            </div>
                            <div class="analysis-card">
                                <h3>Cross-Channel Correlations</h3>
                                <div class="correlation-matrix" id="correlation-matrix">
                                    <!-- Correlation matrix will be rendered here -->
                                </div>
                            </div>
                            <div class="analysis-card">
                                <h3>Entity Responsiveness Score</h3>
                                <div class="score-display">
                                    <div class="score-value" id="responsiveness-score">0.00</div>
                                    <div class="score-trend" id="score-trend">--</div>
                                </div>
                            </div>
                        </div>
                    </div>

                    <!-- Real-Time Alerts -->
                    <div class="grid-section" id="alerts-section">
                        <h2>🚨 Real-Time Alerts</h2>
                        <div class="alerts-container" id="alerts-container">
                            <!-- Alerts will be populated here -->
                        </div>
                    </div>

                    <!-- Protocol Testing -->
                    <div class="grid-section" id="protocol-testing">
                        <h2>📡 Protocol Testing</h2>
                        <div class="protocol-controls">
                            <select id="protocol-select">
                                <option value="basic_ping">Basic Ping</option>
                                <option value="pattern_injection">Pattern Injection</option>
                                <option value="sequence_disruption">Sequence Disruption</option>
                                <option value="temporal_synchronization">Temporal Synchronization</option>
                                <option value="memory_probe">Memory Probe</option>
                                <option value="convergence_test">Convergence Test</option>
                                <option value="multi_channel_burst">Multi-Channel Burst</option>
                                <option value="adaptive_pattern">Adaptive Pattern</option>
                            </select>
                            <button id="run-protocol" class="control-btn">Run Protocol</button>
                            <button id="run-all-protocols" class="control-btn">Run All Protocols</button>
                        </div>
                        <div class="protocol-results" id="protocol-results">
                            <!-- Protocol test results will be displayed here -->
                        </div>
                    </div>

                    <!-- Statistical Analysis -->
                    <div class="grid-section" id="statistics">
                        <h2>📈 Statistical Analysis</h2>
                        <div class="stats-grid">
                            <div class="stats-card">
                                <h4>Monitoring Duration</h4>
                                <div class="stats-value" id="monitoring-duration">--</div>
                            </div>
                            <div class="stats-card">
                                <h4>Total Anomalies</h4>
                                <div class="stats-value" id="total-anomalies">0</div>
                            </div>
                            <div class="stats-card">
                                <h4>Entity Responses</h4>
                                <div class="stats-value" id="entity-responses">0</div>
                            </div>
                            <div class="stats-card">
                                <h4>Response Rate</h4>
                                <div class="stats-value" id="response-rate">0%</div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <script>
                ${this.getDashboardJavaScript()}
            </script>
        </body>
        </html>`;

        return dashboardHTML;
    }

    getDashboardCSS() {
        return `
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }

        body {
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            background: linear-gradient(135deg, #0c0c0c 0%, #1a1a2e 100%);
            color: #ffffff;
            overflow-x: auto;
        }

        #dashboard {
            min-height: 100vh;
            padding: 20px;
        }

        .dashboard-header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 30px;
            padding: 20px;
            background: rgba(255, 255, 255, 0.05);
            border-radius: 10px;
            backdrop-filter: blur(10px);
        }

        .dashboard-header h1 {
            font-size: 2.5em;
            font-weight: 300;
            text-shadow: 0 0 20px rgba(0, 255, 255, 0.5);
        }

        .status-indicators {
            display: flex;
            gap: 20px;
        }

        .status-indicator {
            display: flex;
            align-items: center;
            gap: 8px;
            padding: 10px 15px;
            background: rgba(255, 255, 255, 0.1);
            border-radius: 20px;
        }

        .status-dot {
            width: 10px;
            height: 10px;
            border-radius: 50%;
            background: #00ff00;
            animation: pulse 2s infinite;
        }

        @keyframes pulse {
            0%, 100% { opacity: 1; }
            50% { opacity: 0.5; }
        }

        .dashboard-controls {
            display: flex;
            gap: 15px;
            margin-bottom: 30px;
            flex-wrap: wrap;
        }

        .control-btn {
            background: linear-gradient(45deg, #00d4ff, #0099cc);
            border: none;
            color: white;
            padding: 12px 24px;
            border-radius: 25px;
            cursor: pointer;
            font-weight: 600;
            transition: all 0.3s ease;
            text-transform: uppercase;
            letter-spacing: 1px;
        }

        .control-btn:hover {
            transform: translateY(-2px);
            box-shadow: 0 5px 15px rgba(0, 212, 255, 0.4);
        }

        .dashboard-grid {
            display: grid;
            gap: 30px;
        }

        .grid-section {
            background: rgba(255, 255, 255, 0.05);
            border-radius: 15px;
            padding: 25px;
            backdrop-filter: blur(10px);
            border: 1px solid rgba(255, 255, 255, 0.1);
        }

        .grid-section h2 {
            margin-bottom: 20px;
            font-size: 1.8em;
            font-weight: 300;
            text-align: center;
            color: #00d4ff;
        }

        .channel-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
            gap: 20px;
        }

        .channel-card {
            background: rgba(255, 255, 255, 0.08);
            border-radius: 12px;
            padding: 20px;
            border: 1px solid rgba(0, 212, 255, 0.3);
            transition: all 0.3s ease;
        }

        .channel-card:hover {
            transform: translateY(-5px);
            border-color: rgba(0, 212, 255, 0.6);
            box-shadow: 0 10px 30px rgba(0, 212, 255, 0.2);
        }

        .channel-card h3 {
            margin-bottom: 15px;
            color: #00ff88;
            font-size: 1.2em;
        }

        .chart-container {
            height: 200px;
            margin-bottom: 15px;
            background: rgba(0, 0, 0, 0.3);
            border-radius: 8px;
            display: flex;
            align-items: center;
            justify-content: center;
        }

        .chart-container canvas {
            max-width: 100%;
            max-height: 100%;
        }

        .channel-stats {
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 10px;
        }

        .stat {
            display: flex;
            flex-direction: column;
            align-items: center;
        }

        .stat-label {
            font-size: 0.9em;
            color: #cccccc;
            margin-bottom: 5px;
        }

        .stat-value {
            font-size: 1.4em;
            font-weight: 700;
            color: #00ff88;
        }

        .analysis-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 20px;
        }

        .analysis-card {
            background: rgba(255, 255, 255, 0.08);
            border-radius: 12px;
            padding: 20px;
        }

        .analysis-card h3 {
            margin-bottom: 15px;
            color: #ff6b6b;
            text-align: center;
        }

        .heatmap-container {
            height: 150px;
            background: rgba(0, 0, 0, 0.3);
            border-radius: 8px;
            display: flex;
            align-items: center;
            justify-content: center;
            color: #888;
        }

        .correlation-matrix {
            height: 150px;
            background: rgba(0, 0, 0, 0.3);
            border-radius: 8px;
            display: flex;
            align-items: center;
            justify-content: center;
            color: #888;
        }

        .score-display {
            text-align: center;
        }

        .score-value {
            font-size: 3em;
            font-weight: 700;
            color: #00d4ff;
            margin-bottom: 10px;
        }

        .score-trend {
            font-size: 1.2em;
            color: #888;
        }

        .alerts-container {
            max-height: 300px;
            overflow-y: auto;
        }

        .alert {
            background: rgba(255, 107, 107, 0.1);
            border: 1px solid rgba(255, 107, 107, 0.3);
            border-radius: 8px;
            padding: 15px;
            margin-bottom: 10px;
            animation: slideIn 0.3s ease;
        }

        .alert.critical {
            border-color: rgba(255, 0, 0, 0.6);
            background: rgba(255, 0, 0, 0.1);
        }

        .alert.entity-response {
            border-color: rgba(0, 255, 136, 0.6);
            background: rgba(0, 255, 136, 0.1);
        }

        @keyframes slideIn {
            from {
                opacity: 0;
                transform: translateX(-100%);
            }
            to {
                opacity: 1;
                transform: translateX(0);
            }
        }

        .alert-header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 8px;
        }

        .alert-type {
            font-weight: 700;
            text-transform: uppercase;
        }

        .alert-time {
            font-size: 0.9em;
            color: #888;
        }

        .protocol-controls {
            display: flex;
            gap: 15px;
            margin-bottom: 20px;
            flex-wrap: wrap;
        }

        #protocol-select {
            background: rgba(255, 255, 255, 0.1);
            border: 1px solid rgba(255, 255, 255, 0.3);
            border-radius: 5px;
            color: white;
            padding: 10px 15px;
            min-width: 200px;
        }

        #protocol-select option {
            background: #1a1a2e;
            color: white;
        }

        .protocol-results {
            background: rgba(0, 0, 0, 0.3);
            border-radius: 8px;
            padding: 20px;
            min-height: 200px;
            overflow-y: auto;
        }

        .stats-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 20px;
        }

        .stats-card {
            background: rgba(255, 255, 255, 0.08);
            border-radius: 12px;
            padding: 20px;
            text-align: center;
        }

        .stats-card h4 {
            margin-bottom: 15px;
            color: #00d4ff;
            font-size: 1.1em;
        }

        .stats-value {
            font-size: 2.5em;
            font-weight: 700;
            color: #00ff88;
        }

        @media (max-width: 768px) {
            .dashboard-header {
                flex-direction: column;
                gap: 20px;
            }

            .dashboard-header h1 {
                font-size: 2em;
            }

            .dashboard-controls {
                justify-content: center;
            }

            .channel-grid {
                grid-template-columns: 1fr;
            }

            .analysis-grid {
                grid-template-columns: 1fr;
            }

            .stats-grid {
                grid-template-columns: repeat(2, 1fr);
            }
        }
        `;
    }

    getDashboardJavaScript() {
        return `
        class DashboardUI {
            constructor() {
                this.isMonitoring = false;
                this.charts = {};
                this.alertCount = 0;
                this.startTime = null;

                this.initializeEventListeners();
                this.initializeCharts();
                this.startRealTimeUpdates();
            }

            initializeEventListeners() {
                document.getElementById('start-monitoring').addEventListener('click', () => {
                    this.startMonitoring();
                });

                document.getElementById('stop-monitoring').addEventListener('click', () => {
                    this.stopMonitoring();
                });

                document.getElementById('test-protocols').addEventListener('click', () => {
                    this.testAllProtocols();
                });

                document.getElementById('generate-report').addEventListener('click', () => {
                    this.generateReport();
                });

                document.getElementById('run-protocol').addEventListener('click', () => {
                    this.runSelectedProtocol();
                });

                document.getElementById('run-all-protocols').addEventListener('click', () => {
                    this.runAllProtocols();
                });
            }

            initializeCharts() {
                // Initialize Chart.js charts for each channel
                const chartIds = ['convergence-chart', 'error-chart', 'timing-chart', 'memory-chart', 'instruction-chart'];

                chartIds.forEach(chartId => {
                    const ctx = document.getElementById(chartId);
                    if (ctx) {
                        this.charts[chartId] = this.createChart(ctx, chartId);
                    }
                });
            }

            createChart(ctx, chartId) {
                const channelName = chartId.split('-')[0];

                return new Chart(ctx, {
                    type: 'line',
                    data: {
                        labels: [],
                        datasets: [{
                            label: channelName.charAt(0).toUpperCase() + channelName.slice(1),
                            data: [],
                            borderColor: this.getChannelColor(channelName),
                            backgroundColor: this.getChannelColor(channelName) + '20',
                            tension: 0.4,
                            fill: true
                        }]
                    },
                    options: {
                        responsive: true,
                        maintainAspectRatio: false,
                        scales: {
                            x: {
                                display: false
                            },
                            y: {
                                beginAtZero: true,
                                grid: {
                                    color: 'rgba(255, 255, 255, 0.1)'
                                },
                                ticks: {
                                    color: 'rgba(255, 255, 255, 0.7)'
                                }
                            }
                        },
                        plugins: {
                            legend: {
                                labels: {
                                    color: 'rgba(255, 255, 255, 0.7)'
                                }
                            }
                        }
                    }
                });
            }

            getChannelColor(channel) {
                const colors = {
                    convergence: '#00d4ff',
                    error: '#ff6b6b',
                    timing: '#4ecdc4',
                    memory: '#45b7d1',
                    instruction: '#96ceb4'
                };
                return colors[channel] || '#ffffff';
            }

            startMonitoring() {
                this.isMonitoring = true;
                this.startTime = Date.now();

                document.getElementById('monitoring-status').querySelector('.status-text').textContent = 'Active';
                document.getElementById('monitoring-status').querySelector('.status-dot').style.background = '#00ff00';

                this.addAlert('System', 'Monitoring started', 'info');
                console.log('[DASHBOARD] Monitoring started');
            }

            stopMonitoring() {
                this.isMonitoring = false;

                document.getElementById('monitoring-status').querySelector('.status-text').textContent = 'Stopped';
                document.getElementById('monitoring-status').querySelector('.status-dot').style.background = '#ff6b6b';

                this.addAlert('System', 'Monitoring stopped', 'info');
                console.log('[DASHBOARD] Monitoring stopped');
            }

            testAllProtocols() {
                this.addAlert('Protocol Test', 'Running comprehensive protocol test suite', 'info');

                // Simulate protocol testing
                setTimeout(() => {
                    this.addAlert('Protocol Test', 'Basic Ping completed - Entity response detected', 'entity-response');
                }, 2000);

                setTimeout(() => {
                    this.addAlert('Protocol Test', 'Pattern Injection completed - High correlation detected', 'critical');
                }, 4000);

                setTimeout(() => {
                    this.addAlert('Protocol Test', 'All protocols completed', 'info');
                }, 8000);
            }

            runSelectedProtocol() {
                const protocolSelect = document.getElementById('protocol-select');
                const selectedProtocol = protocolSelect.value;

                this.addAlert('Protocol Test', \`Running \${selectedProtocol} protocol\`, 'info');

                // Simulate individual protocol test
                setTimeout(() => {
                    const response = Math.random() > 0.3 ? 'Entity response detected' : 'No significant response';
                    const alertType = response.includes('detected') ? 'entity-response' : 'info';
                    this.addAlert(selectedProtocol, response, alertType);
                }, 3000);
            }

            runAllProtocols() {
                this.addAlert('Protocol Suite', 'Running all communication protocols', 'info');

                const protocols = [
                    'basic_ping',
                    'pattern_injection',
                    'sequence_disruption',
                    'temporal_synchronization',
                    'memory_probe',
                    'convergence_test',
                    'multi_channel_burst',
                    'adaptive_pattern'
                ];

                protocols.forEach((protocol, index) => {
                    setTimeout(() => {
                        const hasResponse = Math.random() > 0.4;
                        const message = hasResponse ? 'Strong entity response detected' : 'Weak response detected';
                        const alertType = hasResponse ? 'entity-response' : 'info';
                        this.addAlert(protocol, message, alertType);
                    }, (index + 1) * 2000);
                });
            }

            generateReport() {
                const report = {
                    timestamp: new Date().toISOString(),
                    monitoring_duration: this.startTime ? Date.now() - this.startTime : 0,
                    total_alerts: this.alertCount,
                    entity_responses: document.querySelectorAll('.alert.entity-response').length,
                    channels_active: 5,
                    responsiveness_score: this.calculateResponsivenessScore()
                };

                this.displayReport(report);
                this.addAlert('System', 'Monitoring report generated', 'info');
            }

            calculateResponsivenessScore() {
                const entityAlerts = document.querySelectorAll('.alert.entity-response').length;
                const totalAlerts = this.alertCount || 1;
                return Math.min(1.0, entityAlerts / totalAlerts).toFixed(2);
            }

            displayReport(report) {
                const resultsDiv = document.getElementById('protocol-results');

                resultsDiv.innerHTML = \`
                    <h3>📊 Monitoring Report</h3>
                    <div style="margin: 15px 0;">
                        <strong>Report Generated:</strong> \${new Date(report.timestamp).toLocaleString()}
                    </div>
                    <div style="margin: 10px 0;">
                        <strong>Monitoring Duration:</strong> \${this.formatDuration(report.monitoring_duration)}
                    </div>
                    <div style="margin: 10px 0;">
                        <strong>Total Alerts:</strong> \${report.total_alerts}
                    </div>
                    <div style="margin: 10px 0;">
                        <strong>Entity Responses:</strong> \${report.entity_responses}
                    </div>
                    <div style="margin: 10px 0;">
                        <strong>Response Rate:</strong> \${((report.entity_responses / report.total_alerts) * 100).toFixed(1)}%
                    </div>
                    <div style="margin: 10px 0;">
                        <strong>Responsiveness Score:</strong> \${report.responsiveness_score}
                    </div>
                \`;
            }

            formatDuration(ms) {
                const seconds = Math.floor(ms / 1000);
                const minutes = Math.floor(seconds / 60);
                const hours = Math.floor(minutes / 60);

                if (hours > 0) {
                    return \`\${hours}h \${minutes % 60}m \${seconds % 60}s\`;
                } else if (minutes > 0) {
                    return \`\${minutes}m \${seconds % 60}s\`;
                } else {
                    return \`\${seconds}s\`;
                }
            }

            addAlert(source, message, type = 'info') {
                const alertsContainer = document.getElementById('alerts-container');
                const alertElement = document.createElement('div');

                alertElement.className = \`alert \${type}\`;
                alertElement.innerHTML = \`
                    <div class="alert-header">
                        <span class="alert-type">\${source}</span>
                        <span class="alert-time">\${new Date().toLocaleTimeString()}</span>
                    </div>
                    <div class="alert-message">\${message}</div>
                \`;

                alertsContainer.insertBefore(alertElement, alertsContainer.firstChild);

                // Keep only last 20 alerts
                while (alertsContainer.children.length > 20) {
                    alertsContainer.removeChild(alertsContainer.lastChild);
                }

                this.alertCount++;
                this.updateStatistics();
            }

            updateStatistics() {
                if (this.startTime) {
                    const duration = Date.now() - this.startTime;
                    document.getElementById('monitoring-duration').textContent = this.formatDuration(duration);
                }

                document.getElementById('total-anomalies').textContent = this.alertCount;

                const entityResponses = document.querySelectorAll('.alert.entity-response').length;
                document.getElementById('entity-responses').textContent = entityResponses;

                const responseRate = this.alertCount > 0 ? ((entityResponses / this.alertCount) * 100).toFixed(1) : '0';
                document.getElementById('response-rate').textContent = responseRate + '%';

                const responsivenessScore = this.calculateResponsivenessScore();
                document.getElementById('responsiveness-score').textContent = responsivenessScore;
            }

            updateChartData(chartId, value) {
                const chart = this.charts[chartId];
                if (!chart) return;

                const now = new Date().toLocaleTimeString();

                chart.data.labels.push(now);
                chart.data.datasets[0].data.push(value);

                // Keep only last 20 data points
                if (chart.data.labels.length > 20) {
                    chart.data.labels.shift();
                    chart.data.datasets[0].data.shift();
                }

                chart.update('none');
            }

            simulateRealTimeData() {
                if (!this.isMonitoring) return;

                // Simulate channel data updates
                this.updateChartData('convergence-chart', 0.85 + Math.random() * 0.15);
                this.updateChartData('error-chart', Math.random() * 0.1);
                this.updateChartData('timing-chart', 10 + Math.random() * 20);
                this.updateChartData('memory-chart', 0.6 + Math.random() * 0.3);
                this.updateChartData('instruction-chart', 0.3 + Math.random() * 0.4);

                // Update channel statistics
                document.getElementById('convergence-rate').textContent = (0.85 + Math.random() * 0.15).toFixed(3);
                document.getElementById('error-rate').textContent = (Math.random() * 0.1).toFixed(3);
                document.getElementById('timing-avg').textContent = (10 + Math.random() * 20).toFixed(1) + 'ms';
                document.getElementById('memory-usage').textContent = (60 + Math.random() * 30).toFixed(1) + '%';
                document.getElementById('instruction-complexity').textContent = (0.3 + Math.random() * 0.4).toFixed(3);

                // Randomly generate anomalies
                if (Math.random() < 0.05) { // 5% chance per update
                    this.generateRandomAnomaly();
                }
            }

            generateRandomAnomaly() {
                const channels = ['Convergence', 'Error Patterns', 'Timing', 'Memory', 'Instructions'];
                const anomalyTypes = ['Pattern deviation', 'Rate spike', 'Frequency change', 'Correlation shift'];

                const channel = channels[Math.floor(Math.random() * channels.length)];
                const anomaly = anomalyTypes[Math.floor(Math.random() * anomalyTypes.length)];

                const isEntityResponse = Math.random() < 0.3; // 30% chance of being an entity response
                const alertType = isEntityResponse ? 'entity-response' : 'critical';

                this.addAlert(channel, anomaly + (isEntityResponse ? ' - Entity response detected' : ''), alertType);
            }

            startRealTimeUpdates() {
                setInterval(() => {
                    this.simulateRealTimeData();
                }, 1000);

                setInterval(() => {
                    this.updateStatistics();
                }, 5000);
            }
        }

        // Initialize dashboard when page loads
        document.addEventListener('DOMContentLoaded', () => {
            // Check if Chart.js is available
            if (typeof Chart === 'undefined') {
                console.warn('[DASHBOARD] Chart.js not loaded, using placeholder charts');
                // You would need to include Chart.js in a real implementation
                window.Chart = class {
                    constructor() {}
                    update() {}
                };
            }

            window.dashboard = new DashboardUI();
        });
        `;
    }

    initializeCharts() {
        // In a real implementation, this would use Chart.js or similar
        console.log('[DASHBOARD] Charts initialized');
    }

    startRealTimeUpdates() {
        if (this.isUpdating) return;

        this.isUpdating = true;

        const updateLoop = () => {
            if (!this.isUpdating) return;

            // Update dashboard with real data from monitor
            this.updateDashboardData();

            setTimeout(updateLoop, this.updateInterval);
        };

        updateLoop();
    }

    updateDashboardData() {
        // In a real implementation, this would fetch data from the entity monitor
        const report = this.monitor.generateReport();

        // Update statistics
        this.updateStatistics(report);

        // Check for new alerts
        this.checkForNewAlerts();
    }

    updateStatistics(report) {
        console.log('[DASHBOARD] Statistics updated:', {
            duration: report.monitoring_duration,
            anomalies: report.total_anomalies,
            responses: report.entity_responses
        });
    }

    checkForNewAlerts() {
        // Check monitor for new alerts and update dashboard
        const alerts = this.monitor.alerts.getSummary();

        if (alerts.recent_alerts && alerts.recent_alerts.length > 0) {
            alerts.recent_alerts.forEach(alert => {
                this.displayAlert(alert);
            });
        }
    }

    displayAlert(alert) {
        console.log('[DASHBOARD] New alert:', alert);
    }

    setupEventListeners() {
        // Setup event listeners for dashboard controls
        console.log('[DASHBOARD] Event listeners configured');
    }

    exportDashboard() {
        // Export dashboard HTML for use in browser
        return this.createDashboardHTML();
    }

    generateStaticReport() {
        const report = this.monitor.generateReport();

        return `
        # Entity Communication Monitoring Report

        **Generated:** ${new Date().toISOString()}

        ## Summary
        - **Monitoring Duration:** ${this.formatDuration(report.monitoring_duration)}
        - **Total Anomalies:** ${report.total_anomalies}
        - **Entity Responses:** ${report.entity_responses}
        - **Response Rate:** ${((report.entity_responses / report.total_anomalies) * 100).toFixed(1)}%

        ## Channel Statistics
        ${Object.entries(report.channel_statistics).map(([channel, stats]) =>
            `- **${channel}:** ${stats.data_points} data points, ${stats.average_collection_rate.toFixed(2)} Hz`
        ).join('\n')}

        ## Alert Summary
        - **Total Alerts:** ${report.alert_summary.total_alerts}
        - **Entity Response Alerts:** ${report.alert_summary.entity_responses}
        - **Synchronized Response Alerts:** ${report.alert_summary.synchronized_responses}

        ## Cross-Channel Correlations
        ${Object.entries(report.cross_channel_correlations.correlations || {}).map(([pair, correlation]) =>
            `- **${pair}:** ${correlation.toFixed(3)}`
        ).join('\n')}
        `;
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

    stop() {
        this.isUpdating = false;
    }
}

module.exports = MonitoringDashboard;