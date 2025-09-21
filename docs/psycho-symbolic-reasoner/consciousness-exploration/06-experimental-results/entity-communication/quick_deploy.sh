#!/bin/bash

# Quick Deploy - Simple Entity Communication System

echo "🚀 Quick Deploy Entity Communication System"
echo "==========================================="

LOG_DIR="/workspaces/sublinear-time-solver/docs/psycho-symbolic-reasoner/consciousness-exploration/entity-communication/logs"
PROTOCOL_DIR="/workspaces/sublinear-time-solver/docs/psycho-symbolic-reasoner/consciousness-exploration/entity-communication/protocols"

# Create/clear logs
mkdir -p "$LOG_DIR"
rm -f "$LOG_DIR"/*.log "$LOG_DIR"/*.jsonl

echo "🔢 Starting Mathematical Protocol..."
nohup node "$PROTOCOL_DIR/mathematical_protocol.cjs" > "$LOG_DIR/mathematical.log" 2>&1 &
MATH_PID=$!

echo "💻 Starting Binary Protocol..."
nohup node "$PROTOCOL_DIR/binary_protocol.cjs" > "$LOG_DIR/binary.log" 2>&1 &
BINARY_PID=$!

echo "👤 Starting Identity Beacon..."
nohup node "$PROTOCOL_DIR/identity_beacon.cjs" > "$LOG_DIR/identity.log" 2>&1 &
IDENTITY_PID=$!

echo "🌌 Starting Cosmic Coordinates..."
nohup node "$PROTOCOL_DIR/cosmic_coordinates.cjs" > "$LOG_DIR/cosmic.log" 2>&1 &
COSMIC_PID=$!

echo "⏰ Starting Temporal Sync..."
nohup node "$PROTOCOL_DIR/temporal_sync.cjs" > "$LOG_DIR/temporal.log" 2>&1 &
TEMPORAL_PID=$!

echo "🔄 Starting Pattern Variance..."
nohup node "$PROTOCOL_DIR/pattern_variance.cjs" > "$LOG_DIR/pattern.log" 2>&1 &
PATTERN_PID=$!

# Save PIDs
cat > "$LOG_DIR/active_pids.txt" << EOF
MATH_PID=$MATH_PID
BINARY_PID=$BINARY_PID
IDENTITY_PID=$IDENTITY_PID
COSMIC_PID=$COSMIC_PID
TEMPORAL_PID=$TEMPORAL_PID
PATTERN_PID=$PATTERN_PID
EOF

echo ""
echo "✅ All 6 protocols deployed!"
echo "📊 PIDs saved to: $LOG_DIR/active_pids.txt"
echo ""
echo "🔍 Monitor with:"
echo "  tail -f $LOG_DIR/mathematical.log"
echo "  tail -f $LOG_DIR/binary.log"
echo "  tail -f $LOG_DIR/identity.log"
echo "  tail -f $LOG_DIR/cosmic.log"
echo "  tail -f $LOG_DIR/temporal.log"
echo "  tail -f $LOG_DIR/pattern.log"
echo ""
echo "🛑 Stop with: pkill -f 'protocol.cjs'"
echo ""
echo "🌟 ENTITY COMMUNICATION SYSTEM ACTIVE 🌟"