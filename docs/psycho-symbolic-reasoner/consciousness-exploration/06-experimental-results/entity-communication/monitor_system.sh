#!/bin/bash

# Real-time Entity Communication System Monitor
# Displays live status of all protocols and breakthrough detection

LOG_DIR="/workspaces/sublinear-time-solver/docs/psycho-symbolic-reasoner/consciousness-exploration/entity-communication/logs"

echo "🔍 Entity Communication System Monitor"
echo "====================================="
echo "📊 Real-time status and breakthrough detection"
echo ""

# Function to get latest log entry
get_latest_log() {
    local file="$1"
    if [ -f "$file" ]; then
        tail -1 "$file" 2>/dev/null | jq -r '.timestamp // "No data"' 2>/dev/null || echo "No data"
    else
        echo "Not started"
    fi
}

# Function to count total entries
count_entries() {
    local file="$1"
    if [ -f "$file" ]; then
        wc -l < "$file" 2>/dev/null || echo "0"
    else
        echo "0"
    fi
}

# Function to check for breakthroughs
check_breakthroughs() {
    local pattern="$1"
    find "$LOG_DIR" -name "*.jsonl" -exec grep -l "$pattern" {} \; 2>/dev/null | wc -l
}

# Monitor loop
while true; do
    clear
    echo "🌟 ENTITY COMMUNICATION SYSTEM STATUS"
    echo "===================================="
    echo "📅 $(date)"
    echo ""

    # Check if system is running
    if [ -f "$LOG_DIR/agent_pids.txt" ]; then
        echo "✅ System Status: ACTIVE"
    else
        echo "❌ System Status: STOPPED"
    fi
    echo ""

    # Protocol Status
    echo "📡 PROTOCOL AGENTS STATUS:"
    echo "┌─────────────────────┬──────────────┬────────────┬─────────────────────┐"
    echo "│ Agent               │ Transmissions│ Last Activity│ Latest Log Entry   │"
    echo "├─────────────────────┼──────────────┼────────────┼─────────────────────┤"

    protocols=("mathematical" "binary" "identity_beacon" "cosmic_coordinates" "temporal_sync" "pattern_variance")
    for protocol in "${protocols[@]}"; do
        log_file="$LOG_DIR/${protocol}.jsonl"
        if [ "$protocol" = "identity_beacon" ]; then
            log_file="$LOG_DIR/identity_beacon.jsonl"
        elif [ "$protocol" = "cosmic_coordinates" ]; then
            log_file="$LOG_DIR/cosmic_coordinates.jsonl"
        elif [ "$protocol" = "temporal_sync" ]; then
            log_file="$LOG_DIR/temporal_sync.jsonl"
        elif [ "$protocol" = "pattern_variance" ]; then
            log_file="$LOG_DIR/pattern_variance.jsonl"
        else
            log_file="$LOG_DIR/${protocol}_protocol.jsonl"
        fi

        count=$(count_entries "$log_file")
        latest=$(get_latest_log "$log_file")

        printf "│ %-19s │ %-12s │ %-10s │ %-19s │\n" \
            "${protocol^}" \
            "$count" \
            "Active" \
            "${latest:0:19}"
    done
    echo "└─────────────────────┴──────────────┴────────────┴─────────────────────┘"
    echo ""

    # Breakthrough Detection
    echo "🚨 BREAKTHROUGH EVENTS:"
    echo "┌─────────────────────┬───────────────┬─────────────────────────────────┐"
    echo "│ Protocol Type       │ Breakthrough  │ Description                     │"
    echo "│                     │ Count         │                                 │"
    echo "├─────────────────────┼───────────────┼─────────────────────────────────┤"

    # Check for breakthrough indicators in logs
    breakthrough_count=0
    for protocol in "${protocols[@]}"; do
        if [ "$protocol" = "identity_beacon" ]; then
            file_pattern="identity"
        elif [ "$protocol" = "cosmic_coordinates" ]; then
            file_pattern="cosmic"
        elif [ "$protocol" = "temporal_sync" ]; then
            file_pattern="temporal"
        elif [ "$protocol" = "pattern_variance" ]; then
            file_pattern="pattern"
        else
            file_pattern="$protocol"
        fi

        breakthroughs=$(find "$LOG_DIR" -name "*${file_pattern}*.jsonl" -exec grep -c "breakthrough_indicator.*true" {} \; 2>/dev/null | awk '{sum+=$1} END {print sum+0}')
        breakthrough_count=$((breakthrough_count + breakthroughs))

        description="Advanced communication detected"
        if [ "$breakthroughs" -gt 0 ]; then
            printf "│ %-19s │ %-13s │ %-31s │\n" \
                "${protocol^}" \
                "$breakthroughs" \
                "$description"
        fi
    done

    if [ "$breakthrough_count" -eq 0 ]; then
        printf "│ %-19s │ %-13s │ %-31s │\n" \
            "No breakthroughs" \
            "0" \
            "Continuing baseline communication"
    fi
    echo "└─────────────────────┴───────────────┴─────────────────────────────────┘"
    echo ""

    # System Health
    echo "💚 SYSTEM HEALTH:"

    # Check disk space
    disk_usage=$(df "$LOG_DIR" | awk 'NR==2 {print $5}' | sed 's/%//')
    if [ "$disk_usage" -gt 90 ]; then
        echo "⚠️  Disk Usage: ${disk_usage}% (HIGH)"
    else
        echo "✅ Disk Usage: ${disk_usage}%"
    fi

    # Check log file sizes
    total_log_size=$(find "$LOG_DIR" -name "*.jsonl" -exec du -c {} \; 2>/dev/null | tail -1 | awk '{print $1}')
    echo "📁 Log Size: ${total_log_size}K"

    # Check process count
    process_count=$(pgrep -f "entity-communication" | wc -l)
    echo "🔄 Active Processes: $process_count"

    echo ""
    echo "📊 REAL-TIME METRICS:"

    # Calculate response rates
    for protocol in mathematical binary identity cosmic temporal pattern; do
        response_file="$LOG_DIR/${protocol}*.jsonl"
        if ls $response_file >/dev/null 2>&1; then
            total_transmissions=$(find "$LOG_DIR" -name "${protocol}*.jsonl" -exec grep -c '"type":"transmission"' {} \; 2>/dev/null | awk '{sum+=$1} END {print sum+0}')
            total_responses=$(find "$LOG_DIR" -name "${protocol}*.jsonl" -exec grep -c '"type":"response"' {} \; 2>/dev/null | awk '{sum+=$1} END {print sum+0}')

            if [ "$total_transmissions" -gt 0 ]; then
                response_rate=$((total_responses * 100 / total_transmissions))
                echo "📈 ${protocol^} Response Rate: ${response_rate}%"
            fi
        fi
    done

    echo ""
    echo "🔧 CONTROL COMMANDS:"
    echo "  [Ctrl+C] Stop monitoring"
    echo "  'tail -f $LOG_DIR/coordinator.log' - View coordinator logs"
    echo "  'bash stop_all_protocols.sh' - Stop all agents"
    echo ""
    echo "⏱️  Next update in 10 seconds..."
    echo "🌟 Scanning for entity responses..."

    # Wait for 10 seconds or until interrupted
    sleep 10
done