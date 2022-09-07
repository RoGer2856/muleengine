use std::time::{Duration, Instant};

pub struct MainLoop {
    desired_fps: f32,
}

impl MainLoop {
    pub fn new(desired_fps: f32) -> Self {
        Self { desired_fps }
    }

    pub fn iter(&self) -> MainLoopIterator {
        MainLoopIterator {
            desired_delta_time_in_secs: 1.0 / self.desired_fps,
            last_next_time: Instant::now(),
        }
    }
}

pub struct MainLoopIterator {
    desired_delta_time_in_secs: f32,
    last_next_time: Instant,
}

impl Iterator for MainLoopIterator {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let now = Instant::now();
        let last_loop_duration = now - self.last_next_time;
        self.last_next_time = now;

        let last_loop_duration_in_secs = last_loop_duration.as_secs_f32();
        let mut reduced_last_loop_duration_in_secs = last_loop_duration_in_secs;

        let count =
            f32::floor(reduced_last_loop_duration_in_secs / self.desired_delta_time_in_secs);
        reduced_last_loop_duration_in_secs -= count * self.desired_delta_time_in_secs;

        if reduced_last_loop_duration_in_secs < self.desired_delta_time_in_secs {
            let remaining_time_in_secs =
                self.desired_delta_time_in_secs - reduced_last_loop_duration_in_secs;
            std::thread::sleep(Duration::from_secs_f32(remaining_time_in_secs));
        }

        Some(self.desired_delta_time_in_secs)
    }
}
