use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct ResourceLimitsQosPolicy {
    max_samples: u32,
    max_instances: u32,
    max_samples_per_instance: u32,
}
