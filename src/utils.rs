#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Wrapping<const SIZE: u32>(u32);

impl<const SIZE: u32> Wrapping<SIZE> {
    pub fn new(num: u32) -> Self {
        Self(num)
    }

    pub fn get(self) -> u32 {
        self.0
    }

    pub fn next(&mut self) {
        self.0 = (self.get() + 1) % SIZE;
    }

    pub fn prev(&mut self) {
        if self.get() == 0 {
            self.0 = SIZE - 1;
        } else {
            self.0 -= 1;
        }
    }
}

impl<const SIZE: u32> PartialEq<u32> for Wrapping<SIZE> {
    fn eq(&self, other: &u32) -> bool {
        self.get() == *other
    }
}

impl<const SIZE: u32> Default for Wrapping<SIZE> {
    fn default() -> Self {
        Self::new(0)
    }
}
