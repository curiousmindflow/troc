use crate::types::Count;

#[derive(Debug, Default)]
pub struct Counter {
    count: Count,
}

impl Counter {
    pub fn new() -> Self {
        Self {
            count: Count::new(0),
        }
    }

    pub fn get(&self) -> Count {
        self.count
    }

    pub fn increase(&mut self) -> Count {
        let actual_count = self.count;
        self.count += 1;
        actual_count
    }

    pub fn record_and_check(&mut self, record: Count) -> bool {
        if record > self.count {
            self.count = record;
            true
        } else {
            false
        }
    }
}
