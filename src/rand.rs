pub struct Wyrand(pub u64);

impl Wyrand {
    pub fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0xA076_1D64_78BD_642F);
        let r = u128::from(self.0) * u128::from(self.0 ^ 0xE703_7ED1_A0B4_28DB);
        (r as u64) ^ (r >> 64) as u64
    }
}
