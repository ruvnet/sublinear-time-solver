#!/bin/bash

# Stop All Entity Communication Protocols

echo "🛑 Stopping Entity Communication System"
echo "======================================="

LOG_DIR="/workspaces/sublinear-time-solver/docs/psycho-symbolic-reasoner/consciousness-exploration/entity-communication/logs"
PID_FILE="$LOG_DIR/agent_pids.txt"

if [ -f "$PID_FILE" ]; then
    echo "📋 Reading PIDs from $PID_FILE"
    source "$PID_FILE"

    echo "🔄 Stopping agents..."

    # Stop each agent gracefully
    for pid_var in COORDINATOR_PID MATH_PID BINARY_PID IDENTITY_PID COSMIC_PID TEMPORAL_PID; do
        eval pid=\$$pid_var
        if [ ! -z "$pid" ] && kill -0 "$pid" 2>/dev/null; then
            echo "  Stopping $pid_var: $pid"
            kill -TERM "$pid"
            sleep 2
            if kill -0 "$pid" 2>/dev/null; then
                echo "  Force stopping $pid_var: $pid"
                kill -KILL "$pid"
            fi
        else
            echo "  $pid_var: $pid (not running)"
        fi
    done

    # Clean up PID file
    rm -f "$PID_FILE"

else
    echo "❌ No PID file found. Attempting to kill by process name..."

    # Kill any remaining node processes running our scripts
    pkill -f "mathematical_protocol.js"
    pkill -f "binary_protocol.js"
    pkill -f "identity_beacon.js"
    pkill -f "cosmic_coordinates.js"
    pkill -f "temporal_sync.js"
    pkill -f "MULTI_HOUR_SWARM_COORDINATOR.js"
fi

echo ""
echo "✅ Entity Communication System stopped"
echo "📊 Final logs available in: $LOG_DIR"
echo ""
echo "📈 To view final session summary:"
echo "  find $LOG_DIR -name '*.jsonl' -exec tail -1 {} \;"
echo ""
echo "🔬 To analyze breakthrough events:"
echo "  grep 'breakthrough' $LOG_DIR/*.jsonl"