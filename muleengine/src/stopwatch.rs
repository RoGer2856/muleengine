use std::time::{Duration, Instant};

pub struct Stopwatch {
    start_time: Instant,
}

impl Stopwatch {
    pub fn start_new() -> Self {
        Self {
            start_time: Instant::now(),
        }
    }

    pub fn elapsed(&self) -> Duration {
        Instant::now() - self.start_time
    }

    pub fn restart(&mut self) -> Duration {
        let elapsed_time = Instant::now() - self.start_time;
        self.start_time = Instant::now();
        elapsed_time
    }
}
