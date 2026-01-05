use std::collections::BTreeSet;

use troc_core::types::EntityKey;

#[derive(Debug, Default)]
pub struct EntityIdentifier {
    history: BTreeSet<EntityKey>,
}

impl EntityIdentifier {
    pub fn new() -> Self {
        Self {
            history: Default::default(),
        }
    }

    pub fn get_new_key(&mut self) -> EntityKey {
        for possible_key in (EntityKey::MIN..=EntityKey::MAX).map(EntityKey::from_u32) {
            if !self.history.contains(&possible_key) {
                self.history.insert(possible_key);
                return possible_key;
            }
        }
        unreachable!()
    }

    pub fn remove_key(&mut self, key: &EntityKey) {
        self.history.remove(key);
    }
}
