use std::time::Instant;

pub struct FpsCounter {
    first_draw_time: Option<Instant>,
    draw_count: usize,
}

impl Default for FpsCounter {
    fn default() -> Self {
        Self::new()
    }
}

impl FpsCounter {
    pub fn new() -> Self {
        Self {
            first_draw_time: None,
            draw_count: 0,
        }
    }

    pub fn draw_happened(&mut self) {
        if self.first_draw_time.is_none() {
            self.first_draw_time = Some(Instant::now());
        }

        self.draw_count += 1;
    }

    pub fn get_average_fps(&self) -> Option<f64> {
        self.first_draw_time.map(|first_draw_time| {
            let time_elapsed = Instant::now() - first_draw_time;
            self.draw_count as f64 / time_elapsed.as_secs_f64()
        })
    }

    pub fn restart(&mut self) {
        self.first_draw_time = Some(Instant::now());
        self.draw_count = 0;
    }
}
