use std::time::{Duration, Instant};

pub struct ExponentialMovingAverageFpsCounter {
    smoothing_factor: f64,

    last_transaction_time: Option<Instant>,

    measured_tps: Option<f64>,

    transaction_count: usize,
}

impl ExponentialMovingAverageFpsCounter {
    pub fn new(moving_average_weight: f64) -> Self {
        let mut ret = Self {
            smoothing_factor: moving_average_weight,

            last_transaction_time: None,

            measured_tps: None,

            transaction_count: 0,
        };

        ret.restart();

        ret
    }

    pub fn transaction_happened(&mut self) {
        let now = Instant::now();

        let last_transaction_time = self.last_transaction_time.replace(now).unwrap_or(now);

        let new_tps = compute_tps_from_duration(now - last_transaction_time);

        // if there was a value inside self.measured_tps, then it is used for computing the Exponential Moving Average (Smoothing Average)

        // else new_tps is used for self.measured_tps

        self.measured_tps = new_tps.map(|new_tps| {
            self.measured_tps.take().map_or_else(
                || new_tps,
                |old_tps| old_tps * (1.0 - self.smoothing_factor) + new_tps * self.smoothing_factor,
            )
        });

        self.transaction_count += 1;
    }

    pub fn get_average_tps(&self) -> Option<f64> {
        self.measured_tps
    }

    pub fn restart(&mut self) {
        self.last_transaction_time = None;

        self.measured_tps = None;

        self.transaction_count = 0;
    }
}

fn compute_tps_from_duration(duration: Duration) -> Option<f64> {
    if duration.as_micros() != 0 {
        Some((1000.0 * 1000.0) / duration.as_micros() as f64)
    } else {
        None
    }
}
