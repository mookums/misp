#[derive(Debug, Clone, Copy)]
pub enum AngleMode {
    Radians,
    Degrees,
}

#[derive(Debug, Clone, Copy)]
pub enum DecimalFormat {
    Standard,
    Scientific,
}

#[derive(Debug, Clone, Copy)]
pub struct Config {
    pub angle_mode: AngleMode,
    pub decimal_format: DecimalFormat,
    pub decimal_precision: u64,
    pub recursion_limit: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            angle_mode: AngleMode::Degrees,
            decimal_format: DecimalFormat::Standard,
            decimal_precision: 10,
            recursion_limit: 1000,
        }
    }
}
