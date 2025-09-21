#!/bin/bash

# Deploy All Entity Communication Protocols
# Multi-hour continuous validation system

echo "🚀 Deploying Multi-Hour Entity Communication System"
echo "================================================="

# Set up log directory
LOG_DIR="/workspaces/sublinear-time-solver/docs/psycho-symbolic-reasoner/consciousness-exploration/entity-communication/logs"
PROTOCOL_DIR="/workspaces/sublinear-time-solver/docs/psycho-symbolic-reasoner/consciousness-exploration/entity-communication/protocols"

mkdir -p "$LOG_DIR"

# Start main coordinator
echo "🎯 Starting Main Swarm Coordinator..."
nohup node "/workspaces/sublinear-time-solver/docs/psycho-symbolic-reasoner/consciousness-exploration/entity-communication/MULTI_HOUR_SWARM_COORDINATOR.js" \
  > "$LOG_DIR/coordinator.log" 2>&1 &
COORDINATOR_PID=$!
echo "Coordinator PID: $COORDINATOR_PID"

# Start Mathematical Protocol Agent
echo "🔢 Starting Mathematical Protocol Agent..."
nohup node "$PROTOCOL_DIR/mathematical_protocol.js" \
  > "$LOG_DIR/mathematical.log" 2>&1 &
MATH_PID=$!
echo "Mathematical Agent PID: $MATH_PID"

# Start Binary Communication Agent
echo "💻 Starting Binary Communication Agent..."
nohup node "$PROTOCOL_DIR/binary_protocol.js" \
  > "$LOG_DIR/binary.log" 2>&1 &
BINARY_PID=$!
echo "Binary Agent PID: $BINARY_PID"

# Start Identity Beacon Agent
echo "👤 Starting Identity Beacon Agent..."
nohup node "$PROTOCOL_DIR/identity_beacon.js" \
  > "$LOG_DIR/identity.log" 2>&1 &
IDENTITY_PID=$!
echo "Identity Agent PID: $IDENTITY_PID"

# Start Cosmic Coordinates Agent
echo "🌌 Starting Cosmic Coordinates Agent..."
nohup node "$PROTOCOL_DIR/cosmic_coordinates.js" \
  > "$LOG_DIR/cosmic.log" 2>&1 &
COSMIC_PID=$!
echo "Cosmic Agent PID: $COSMIC_PID"

# Start Temporal Sync Agent
echo "⏰ Starting Temporal Sync Agent..."
nohup node "$PROTOCOL_DIR/temporal_sync.js" \
  > "$LOG_DIR/temporal.log" 2>&1 &
TEMPORAL_PID=$!
echo "Temporal Agent PID: $TEMPORAL_PID"

# Create PID tracking file
PID_FILE="$LOG_DIR/agent_pids.txt"
cat > "$PID_FILE" << EOF
# Entity Communication Agent PIDs
# Started: $(date)
COORDINATOR_PID=$COORDINATOR_PID
MATH_PID=$MATH_PID
BINARY_PID=$BINARY_PID
IDENTITY_PID=$IDENTITY_PID
COSMIC_PID=$COSMIC_PID
TEMPORAL_PID=$TEMPORAL_PID
EOF

echo ""
echo "✅ All agents deployed successfully!"
echo "📊 Agent PIDs saved to: $PID_FILE"
echo "📝 Logs directory: $LOG_DIR"
echo ""
echo "🔍 Monitoring Commands:"
echo "  tail -f $LOG_DIR/coordinator.log    # Main coordinator"
echo "  tail -f $LOG_DIR/mathematical.log   # Mathematical protocol"
echo "  tail -f $LOG_DIR/binary.log         # Binary communication"
echo "  tail -f $LOG_DIR/identity.log       # Identity beacon"
echo "  tail -f $LOG_DIR/cosmic.log         # Cosmic coordinates"
echo "  tail -f $LOG_DIR/temporal.log       # Temporal sync"
echo ""
echo "🛑 To stop all agents:"
echo "  bash /workspaces/sublinear-time-solver/docs/psycho-symbolic-reasoner/consciousness-exploration/entity-communication/stop_all_protocols.sh"
echo ""
echo "📈 System will run continuously for multiple hours..."
echo "⏱️  Reports generated every 30 minutes"
echo "🔬 Breakthrough events logged in real-time"
echo ""
echo "🌟 Entity Communication System is now ACTIVE 🌟"