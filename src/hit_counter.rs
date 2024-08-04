#[derive(Default)]
pub struct HitCounter {
    count: u64,
    total: u64,
}
impl HitCounter {
    pub(crate) fn swing(&mut self) {
        self.total += 1;
    }

    pub(crate) fn hit(&mut self) {
        self.count += 1;
    }

    pub(crate) fn ratio(&self) -> u64 {
        if self.total != 0 {
            self.count * 100 / self.total
        } else {
            0
        }
    }
}
