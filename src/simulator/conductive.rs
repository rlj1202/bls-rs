pub trait Conductive {
    fn is_conductive(&self) -> bool;
}

const BRIGHTNESS_MIN: u8 = 223;

impl Conductive for &mut [u8] {
    fn is_conductive(&self) -> bool {
        self[..self.len() - 1]
            .iter()
            .any(|&value| value > BRIGHTNESS_MIN)
    }
}

impl Conductive for &[u8] {
    fn is_conductive(&self) -> bool {
        self[..self.len() - 1]
            .iter()
            .any(|&value| value > BRIGHTNESS_MIN)
    }
}
