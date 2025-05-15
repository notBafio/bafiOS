pub struct LcgRng {
    state: u64,
    multiplier: u64,
    increment: u64,
}

pub static mut GLOBAL_RNG: u64 = 15746565656558969;

impl LcgRng {
    const MULTIPLIER: u64 = 6364136223846793005;
    const INCREMENT: u64 = 1442695040888963407;

    pub fn new(seed: u64) -> Self {
        LcgRng {
            state: seed,
            multiplier: Self::MULTIPLIER,
            increment: Self::INCREMENT,
        }
    }

    pub fn global_new() -> Self {
        LcgRng {
            state: 0,
            multiplier: Self::MULTIPLIER,
            increment: Self::INCREMENT,
        }
    }

    #[inline]
    pub fn next(&mut self) -> u64 {
        if self.state == 0 {
            unsafe {
                GLOBAL_RNG =
                    GLOBAL_RNG
                        .wrapping_mul(self.multiplier)
                        .wrapping_add(self.increment);

                let mut x = GLOBAL_RNG;
                x ^= x >> 32;
                x ^= x >> 16;
                x ^= x >> 8;
                x
            }
        }
        else {
            self.state = self
                .state
                .wrapping_mul(self.multiplier)
                .wrapping_add(self.increment);

            let mut x = self.state;
            x ^= x >> 32;
            x ^= x >> 16;
            x ^= x >> 8;
            x
        }
    }

    pub fn range(&mut self, min: u64, max: u64) -> u64 {
        assert!(min < max, "min must be less than max");
        let range = max - min;
        min + (self.next() % range)
    }
}
