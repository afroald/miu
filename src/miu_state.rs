/// A representation of the Main Instrument Unit state.
#[derive(Clone, Copy, Debug, Default)]
pub struct MiuState {
    pub engine_speed: u16,
    pub engine_speed_fault: bool,
    pub vehicle_speed: u16,
    pub vehicle_speed_fault: bool,
    pub boost: u8,
    pub coolant_temperature: u8,
    pub coolant_temperature_fault: bool,
    pub fuel_level: u16,
    pub fuel_level_fault: bool,
    pub check_engine: bool,
    pub cruise: bool,
    pub gear_lever: u8,
    pub gear_lever_fault: bool,
    pub actual_gear: u8,
    pub actual_gear_fault: bool,
    pub sport: bool,
    pub winter: bool,
    pub check_gearbox: bool,
}

impl MiuState {
    pub fn get_boost_percentage(&self) -> f32 {
        f32::from(self.boost) / 255.0
    }

    #[allow(dead_code)]
    pub fn set_boost_percentage(&mut self, percentage: f32) {
        if percentage > 100.0 {
            // This could be handled nicer, but it's good enough for an unused method
            panic!("Received percentage above 100%: {}", percentage);
        }

        self.boost = (255.0 * percentage).floor() as u8;
    }
}

#[cfg(test)]
mod tests {
    use super::MiuState;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn it_gets_boost_as_percentage() {
        let mut state = MiuState::default();

        assert_eq!(state.boost, 0);
        assert_eq!(state.get_boost_percentage(), 0.0);

        state.boost = 255;
        assert_eq!(state.get_boost_percentage(), 1.0);

        state.boost = 100;
        assert_approx_eq!(state.get_boost_percentage(), 0.39215687);
    }

    #[test]
    fn it_sets_boost_as_percentage() {
        let mut state = MiuState::default();

        state.set_boost_percentage(0.0);
        assert_eq!(state.boost, 0);

        state.set_boost_percentage(1.0);
        assert_eq!(state.boost, 255);

        state.set_boost_percentage(0.1);
        assert_eq!(state.boost, 25);

        state.set_boost_percentage(0.5);
        assert_eq!(state.boost, 127);
    }
}
