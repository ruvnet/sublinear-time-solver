//! Example nano-agents

use super::{NanoAgent, NanoBus, Message, TickResult, spin};
use super::bus::MessageData;
use crate::TemporalLeadPredictor;
use crate::quantum_container::QuantumContainer;
use crate::self_modifying::SelfModifyingLoop;

// Note: Some advanced agents are disabled due to module dependencies

/// Sensor agent that generates periodic signals
pub struct SensorAgent {
    counter: u64,
    period_ticks: u32,
    current_tick: u32,
}

impl SensorAgent {
    pub fn new(period_ticks: u32) -> Self {
        Self {
            counter: 0,
            period_ticks,
            current_tick: 0,
        }
    }
}

impl NanoAgent for SensorAgent {
    fn name(&self) -> &'static str { "sensor" }

    #[inline(always)]
    fn tick(&mut self, now_ns: u128, bus: &NanoBus) -> TickResult {
        self.current_tick = self.current_tick.wrapping_add(1);

        let mut messages_sent = 0;

        if self.current_tick % self.period_ticks == 0 {
            self.counter = self.counter.wrapping_add(1);
            bus.publish(Message {
                topic: "sensor:data",
                data: MessageData::U64(self.counter),
                timestamp_ns: now_ns,
            });
            messages_sent = 1;
        }

        TickResult {
            cycles: 100, // Minimal cycles
            messages_sent,
            messages_recv: 0,
            budget_used_ns: 50,
        }
    }

    fn budget_ns(&self) -> u128 { 1000 } // 1 microsecond
}

/// Debouncer agent that filters noisy signals
pub struct DebounceAgent {
    last_value: u64,
    stable_count: u32,
    stability_threshold: u32,
    spin_cycles: u32,
}

impl DebounceAgent {
    pub fn new(stability_threshold: u32) -> Self {
        Self {
            last_value: 0,
            stable_count: 0,
            stability_threshold,
            spin_cycles: 16,
        }
    }
}

impl NanoAgent for DebounceAgent {
    fn name(&self) -> &'static str { "debouncer" }

    #[inline(always)]
    fn tick(&mut self, now_ns: u128, bus: &NanoBus) -> TickResult {
        let mut messages_recv = 0;
        let mut messages_sent = 0;
        let mut changed = None;

        // Process up to 8 messages
        for _ in 0..8 {
            if let Some(msg) = bus.try_recv() {
                messages_recv += 1;

                if msg.topic == "sensor:data" {
                    if let MessageData::U64(value) = msg.data {
                        if value == self.last_value {
                            self.stable_count += 1;
                        } else {
                            self.stable_count = 0;
                            self.last_value = value;
                        }

                        if self.stable_count >= self.stability_threshold {
                            changed = Some(self.last_value);
                            self.stable_count = 0;
                        }
                    }
                }
            } else {
                break;
            }
        }

        // Publish stable signal
        if let Some(value) = changed {
            bus.publish(Message {
                topic: "signal:stable",
                data: MessageData::U64(value),
                timestamp_ns: now_ns,
            });
            messages_sent = 1;
        }

        // Controlled spin to maintain timing
        for _ in 0..self.spin_cycles {
            spin();
        }

        TickResult {
            cycles: 200,
            messages_sent,
            messages_recv,
            budget_used_ns: 100,
        }
    }

    fn budget_ns(&self) -> u128 { 2000 } // 2 microseconds
}

/// Quantum decision-making agent
pub struct QuantumDecisionAgent {
    quantum_state: QuantumContainer,
    decision_count: u64,
}

impl QuantumDecisionAgent {
    pub fn new() -> Self {
        Self {
            quantum_state: QuantumContainer::new(3), // 8 possible decisions
            decision_count: 0,
        }
    }
}

impl NanoAgent for QuantumDecisionAgent {
    fn name(&self) -> &'static str { "quantum_decision" }

    fn tick(&mut self, now_ns: u128, bus: &NanoBus) -> TickResult {
        let mut messages_sent = 0;

        // Make quantum decision periodically
        if self.decision_count % 100 == 0 {
            // Create superposition of decisions
            let amplitude = crate::types::QuantumAmplitude::new(
                1.0 / (8.0_f64).sqrt(), 0.0
            );
            for i in 0..8 {
                self.quantum_state.set_superposition_state(i, amplitude);
            }

            // Collapse to decision
            let decision = self.quantum_state.measure();

            bus.publish(Message {
                topic: "quantum:decision",
                data: MessageData::U64(decision as u64),
                timestamp_ns: now_ns,
            });
            messages_sent = 1;
        }

        self.decision_count += 1;

        TickResult {
            cycles: 500,
            messages_sent,
            messages_recv: 0,
            budget_used_ns: 300,
        }
    }

    fn budget_ns(&self) -> u128 { 5000 } // 5 microseconds
}

/// Temporal prediction agent
pub struct TemporalPredictorAgent {
    predictor: TemporalLeadPredictor,
    prediction_count: u64,
}

impl TemporalPredictorAgent {
    pub fn new() -> Self {
        Self {
            predictor: TemporalLeadPredictor::new(100_000, 50),
            prediction_count: 0,
        }
    }
}

impl NanoAgent for TemporalPredictorAgent {
    fn name(&self) -> &'static str { "temporal_predictor" }

    fn tick(&mut self, now_ns: u128, bus: &NanoBus) -> TickResult {
        let mut messages_recv = 0;
        let mut messages_sent = 0;

        // Collect sensor data for prediction
        for _ in 0..4 {
            if let Some(msg) = bus.try_recv() {
                messages_recv += 1;

                if msg.topic == "sensor:data" {
                    if let MessageData::U64(value) = msg.data {
                        // Feed to predictor
                        let state = vec![value as f64];
                        let future = self.predictor.predict_future(state);

                        // Publish prediction
                        if self.prediction_count % 10 == 0 {
                            bus.publish(Message {
                                topic: "prediction:future",
                                data: MessageData::F64(future[0]),
                                timestamp_ns: now_ns,
                            });
                            messages_sent = 1;
                        }
                    }
                }
            } else {
                break;
            }
        }

        self.prediction_count += 1;

        TickResult {
            cycles: 1000,
            messages_sent,
            messages_recv,
            budget_used_ns: 500,
        }
    }

    fn budget_ns(&self) -> u128 { 10_000 } // 10 microseconds
}

/// Self-modifying agent that evolves its behavior
// // pub struct EvolvingAgent {
//     evolution_loop: SelfModifyingLoop,
//     generation: u64,
// }

pub struct EvolvingAgent {\n    evolution_loop: SelfModifyingLoop,\n    generation: u64,\n}\n\nimpl EvolvingAgent {
    pub fn new() -> Self {
        Self {
            evolution_loop: SelfModifyingLoop::new(0.1),
            generation: 0,
        }
    }
}

impl NanoAgent for EvolvingAgent {
    fn name(&self) -> &'static str { "evolving" }

    fn tick(&mut self, now_ns: u128, bus: &NanoBus) -> TickResult {
        let mut messages_sent = 0;

        // Execute evolved function
        let input = (self.generation as f64) * 0.01;
        let output = self.evolution_loop.execute(input);

        // Calculate fitness based on output
        let target = 1.618033988749; // Golden ratio
        let fitness = 1.0 / (1.0 + (output - target).abs());

        // Evolve periodically
        if self.generation % 100 == 0 {
            self.evolution_loop.evolve(fitness);

            // Publish evolution metrics
            let metrics = self.evolution_loop.get_metrics();
            bus.publish(Message {
                topic: "evolution:fitness",
                data: MessageData::F64(metrics.current_fitness),
                timestamp_ns: now_ns,
            });
            messages_sent = 1;
        }

        self.generation += 1;

        TickResult {
            cycles: 2000,
            messages_sent,
            messages_recv: 0,
            budget_used_ns: 1000,
        }
    }

    fn budget_ns(&self) -> u128 { 20_000 } // 20 microseconds
}