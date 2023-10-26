use bytifex_utils::sync::types::{arc_mutex_new, ArcMutex};

#[derive(Clone)]
pub struct VirtualClock {
    frac_1_speed_multiplier_tx: ArcMutex<tokio::sync::watch::Sender<f64>>,
    frac_1_speed_multiplier_rx: tokio::sync::watch::Receiver<f64>,
}

impl Default for VirtualClock {
    fn default() -> Self {
        Self::new()
    }
}

impl VirtualClock {
    pub fn new() -> Self {
        let (tx, rx) = tokio::sync::watch::channel(1.0);
        Self {
            frac_1_speed_multiplier_tx: arc_mutex_new(tx),
            frac_1_speed_multiplier_rx: rx,
        }
    }

    pub fn now() -> tokio::time::Instant {
        tokio::time::Instant::now()
    }

    pub async fn sleep_for(&self, mut timeout: std::time::Duration) {
        let mut rx = self.frac_1_speed_multiplier_rx.clone();
        let mut frac_1_speed = *rx.borrow();
        let mut real_time_instance = tokio::time::Instant::now() + timeout.mul_f64(frac_1_speed);

        loop {
            tokio::select!(
                _ = tokio::time::sleep_until(real_time_instance) => {
                    break;
                }
                res = rx.changed() => {
                    if res.is_ok() {
                        timeout = real_time_instance - tokio::time::Instant::now();

                        let last_frac_1_speed = frac_1_speed;
                        frac_1_speed = *rx.borrow();
                        real_time_instance = tokio::time::Instant::now() + timeout.mul_f64(frac_1_speed / last_frac_1_speed);
                    }
                }
            );
        }
    }

    pub fn set_speed_multiplier(&mut self, multiplier: f64) {
        let _ = self
            .frac_1_speed_multiplier_tx
            .lock()
            .send(1.0 / multiplier);
    }

    pub fn virtual_to_real_seconds_f32(&self, virtual_seconds: f32) -> f32 {
        virtual_seconds * *self.frac_1_speed_multiplier_rx.borrow() as f32
    }

    pub fn real_to_virtual_seconds_f32(&self, real_seconds: f32) -> f32 {
        real_seconds / *self.frac_1_speed_multiplier_rx.borrow() as f32
    }

    pub fn virtual_to_real_seconds_f64(&self, virtual_seconds: f64) -> f64 {
        virtual_seconds * *self.frac_1_speed_multiplier_rx.borrow()
    }

    pub fn real_to_virtual_seconds_f64(&self, real_seconds: f64) -> f64 {
        real_seconds / *self.frac_1_speed_multiplier_rx.borrow()
    }

    pub fn virtual_to_real_duration(
        &self,
        virtual_duration: std::time::Duration,
    ) -> std::time::Duration {
        virtual_duration.mul_f64(*self.frac_1_speed_multiplier_rx.borrow())
    }

    pub fn real_to_virtual_duration(
        &self,
        real_duration: std::time::Duration,
    ) -> std::time::Duration {
        real_duration.mul_f64(1.0 / *self.frac_1_speed_multiplier_rx.borrow())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const THRESHOLD_MILLIS: u128 = 50;

    #[test]
    fn conversions() {
        let mut virtual_clock = VirtualClock::new();
        virtual_clock.set_speed_multiplier(2.0);

        assert_eq!(virtual_clock.real_to_virtual_seconds_f32(6.0), 12.0);
        assert_eq!(virtual_clock.virtual_to_real_seconds_f32(6.0), 3.0);

        assert_eq!(virtual_clock.real_to_virtual_seconds_f64(6.0), 12.0);
        assert_eq!(virtual_clock.virtual_to_real_seconds_f64(6.0), 3.0);

        assert_eq!(
            virtual_clock.real_to_virtual_duration(std::time::Duration::from_secs_f64(6.0)),
            std::time::Duration::from_secs_f64(12.0)
        );
        assert_eq!(
            virtual_clock.virtual_to_real_duration(std::time::Duration::from_secs_f64(6.0)),
            std::time::Duration::from_secs_f64(3.0)
        );
    }

    #[test]
    fn sleep_for() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            let virtual_clock = VirtualClock::new();

            let start_time = std::time::Instant::now();

            virtual_clock
                .sleep_for(std::time::Duration::from_millis(1000))
                .await;

            let time_elapsed_millis = start_time.elapsed().as_millis();
            assert!(time_elapsed_millis >= 1000);
            assert!(time_elapsed_millis - 1000 < THRESHOLD_MILLIS);
        });
    }

    #[test]
    fn sleep_for_change_multiplier() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            {
                let mut virtual_clock = VirtualClock::new();
                virtual_clock.set_speed_multiplier(0.5);

                let start_time = std::time::Instant::now();

                virtual_clock
                    .sleep_for(std::time::Duration::from_millis(1000))
                    .await;

                let time_elapsed_millis = start_time.elapsed().as_millis();
                assert!(time_elapsed_millis >= 2000);
                assert!(time_elapsed_millis - 2000 < THRESHOLD_MILLIS);
            }

            {
                let mut virtual_clock = VirtualClock::new();
                virtual_clock.set_speed_multiplier(2.0);

                let start_time = std::time::Instant::now();

                virtual_clock
                    .sleep_for(std::time::Duration::from_millis(1000))
                    .await;

                let time_elapsed_millis = start_time.elapsed().as_millis();
                assert!(time_elapsed_millis >= 500);
                assert!(time_elapsed_millis - 500 < THRESHOLD_MILLIS);
            }
        });
    }

    #[test]
    fn sleep_for_change_multiplier_on_the_fly() {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.block_on(async {
            let virtual_clock = VirtualClock::new();

            let start_time = std::time::Instant::now();

            {
                let mut virtual_clock = virtual_clock.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

                    virtual_clock.set_speed_multiplier(2.0);
                });
            }

            virtual_clock
                .sleep_for(std::time::Duration::from_millis(1000))
                .await;

            let time_elapsed_millis = start_time.elapsed().as_millis();
            assert!(time_elapsed_millis >= 750);
            assert!(time_elapsed_millis - 750 < THRESHOLD_MILLIS);
        });
    }
}
