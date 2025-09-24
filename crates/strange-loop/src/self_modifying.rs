//! Self-modifying strange loops that evolve their own parameters

use std::sync::{Arc, RwLock};

/// Self-modifying loop that evolves its own code and parameters
pub struct SelfModifyingLoop {
    /// Current loop function (mutable)
    loop_fn: Arc<RwLock<Box<dyn Fn(f64) -> f64 + Send + Sync>>>,
    /// Evolution parameters
    mutation_rate: f64,
    /// Fitness history
    fitness_history: Vec<f64>,
    /// Current generation
    generation: usize,
    /// Best performing parameters
    best_params: Vec<f64>,
}

impl SelfModifyingLoop {
    pub fn new(mutation_rate: f64) -> Self {
        let initial_fn = Box::new(|x: f64| x.sin() * x.cos()) as Box<dyn Fn(f64) -> f64 + Send + Sync>;

        Self {
            loop_fn: Arc::new(RwLock::new(initial_fn)),
            mutation_rate,
            fitness_history: Vec::new(),
            generation: 0,
            best_params: vec![1.0, 0.0, 0.0],
        }
    }

    /// Evolve the loop function based on performance
    pub fn evolve(&mut self, fitness: f64) {
        self.fitness_history.push(fitness);
        self.generation += 1;

        // Only evolve if we have enough history
        if self.fitness_history.len() < 10 {
            return;
        }

        // Calculate fitness trend
        let recent_avg = self.fitness_history.iter().rev().take(5).sum::<f64>() / 5.0;
        let past_avg = self.fitness_history.iter().rev().skip(5).take(5).sum::<f64>() / 5.0;

        // If performance is declining, mutate more aggressively
        let mutation_factor = if recent_avg < past_avg {
            self.mutation_rate * 2.0
        } else {
            self.mutation_rate
        };

        // Mutate parameters
        self.mutate_parameters(mutation_factor);

        // Update the loop function with new parameters
        self.update_loop_function();
    }

    fn mutate_parameters(&mut self, factor: f64) {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        for param in &mut self.best_params {
            if rng.gen::<f64>() < factor {
                // Add Gaussian noise
                let noise = rng.gen_range(-0.1..0.1);
                *param += noise;
                *param = param.clamp(-10.0, 10.0);
            }
        }
    }

    fn update_loop_function(&mut self) {
        let a = self.best_params[0];
        let b = self.best_params[1];
        let c = self.best_params[2];

        let new_fn = Box::new(move |x: f64| {
            a * x.sin() + b * x.cos() + c * x.tan().tanh()
        }) as Box<dyn Fn(f64) -> f64 + Send + Sync>;

        if let Ok(mut fn_lock) = self.loop_fn.write() {
            *fn_lock = new_fn;
        }
    }

    /// Execute the current loop function
    pub fn execute(&self, input: f64) -> f64 {
        if let Ok(fn_lock) = self.loop_fn.read() {
            fn_lock(input)
        } else {
            input
        }
    }

    /// Self-replicate with mutations
    pub fn replicate(&self) -> Self {
        let mut child = Self::new(self.mutation_rate);
        child.best_params = self.best_params.clone();
        child.mutate_parameters(self.mutation_rate * 0.5);
        child.update_loop_function();
        child
    }

    /// Get evolutionary metrics
    pub fn get_metrics(&self) -> EvolutionMetrics {
        EvolutionMetrics {
            generation: self.generation,
            current_fitness: self.fitness_history.last().copied().unwrap_or(0.0),
            best_fitness: self.fitness_history.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).copied().unwrap_or(0.0),
            mutation_rate: self.mutation_rate,
            parameters: self.best_params.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EvolutionMetrics {
    pub generation: usize,
    pub current_fitness: f64,
    pub best_fitness: f64,
    pub mutation_rate: f64,
    pub parameters: Vec<f64>,
}