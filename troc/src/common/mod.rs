use std::collections::HashMap;

use protocol::types::Locator;

use crate::wires::Wire;

#[derive(Debug)]
pub struct EmissionInfosStorage {
    store: HashMap<Locator, Wire>,
}

impl EmissionInfosStorage {
    pub fn new() -> Self {
        Self {
            store: Default::default(),
        }
    }

    pub fn get(&mut self, locator: &Locator) -> &mut Wire {
        if let Some(wire) = self.store.get_mut(locator) {
            wire
        } else {
            panic!()
        }
    }

    pub fn store(&mut self, locator: Locator, wire: Wire) {
        let old = self.store.insert(locator, wire);
        assert!(old.is_none())
    }

    pub fn remove(&mut self, locator: &Locator) {
        let removed = self.store.remove(locator);
        assert!(removed.is_some())
    }
}
