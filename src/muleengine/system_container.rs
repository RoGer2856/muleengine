pub trait System {
    fn tick(&mut self, delta_time_in_secs: f32);
}

pub struct SystemContainer {
    systems: Vec<Box<dyn System>>,
}

impl SystemContainer {
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
        }
    }

    pub fn tick(&mut self, delta_time_in_secs: f32) {
        for system in self.systems.iter_mut() {
            system.tick(delta_time_in_secs);
        }
    }

    pub fn add_system(&mut self, system: impl System + 'static) {
        self.systems.push(Box::new(system));
    }
}
