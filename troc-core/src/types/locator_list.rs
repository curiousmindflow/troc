#![allow(clippy::useless_conversion)]
#![allow(clippy::derivable_impls)]
use std::{
    cmp::Ordering,
    fmt::{Debug, Display},
    ops::{Deref, DerefMut},
};

use binrw::binrw;
use serde::{Deserialize, Serialize};

use crate::types::Locator;

#[binrw]
#[derive(Serialize, Deserialize)]
pub struct LocatorList {
    #[bw(try_calc(u32::try_from(locators.len())))]
    locators_len: u32,
    #[br(count = locators_len)]
    locators: Vec<Locator>,
}

impl LocatorList {
    pub fn new(locators: Vec<Locator>) -> Self {
        Self { locators }
    }

    pub fn merge(self, other: LocatorList) -> LocatorList {
        LocatorList {
            locators: [self.locators, other.locators].concat(),
        }
    }

    pub fn get_inner(&self) -> Vec<Locator> {
        self.locators.clone()
    }
}

impl Debug for LocatorList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LocatorList")
            .field("locators", &self.locators)
            .finish()
    }
}

impl Display for LocatorList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for elt in self.locators.iter() {
            f.write_str(&format!("{}", elt))?;
        }
        Ok(())
    }
}

impl From<&[Locator]> for LocatorList {
    fn from(value: &[Locator]) -> Self {
        LocatorList::new(value.to_vec())
    }
}

impl Default for LocatorList {
    fn default() -> Self {
        Self {
            locators: Vec::new(),
        }
    }
}

impl Clone for LocatorList {
    fn clone(&self) -> Self {
        Self {
            locators: self.locators.clone(),
        }
    }
}

impl PartialEq for LocatorList {
    fn eq(&self, other: &Self) -> bool {
        self.locators == other.locators
    }
}

impl Eq for LocatorList {
    fn assert_receiver_is_total_eq(&self) {}
}

impl PartialOrd for LocatorList {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LocatorList {
    fn cmp(&self, other: &Self) -> Ordering {
        self.locators.cmp(&other.locators)
    }
}

impl Deref for LocatorList {
    type Target = Vec<Locator>;

    fn deref(&self) -> &Self::Target {
        &self.locators
    }
}

impl DerefMut for LocatorList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.locators
    }
}
