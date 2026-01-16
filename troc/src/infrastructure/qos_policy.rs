use troc_core::{
    DeadlineQosPolicy, DurabilityQosPolicy, HistoryQosPolicy, InlineQos, LifespanQosPolicy,
    LivelinessQosPolicy, ParameterId, ReliabilityQosPolicy,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct QosPolicy {
    durability: DurabilityQosPolicy,
    deadline: DeadlineQosPolicy,
    reliability: ReliabilityQosPolicy,
    lifespan: LifespanQosPolicy,
    history: HistoryQosPolicy,
    liveness: LivelinessQosPolicy,
}

impl QosPolicy {
    pub fn to_inline_qos(&self) -> ParameterId {
        unimplemented!()
    }

    pub fn durability(&self) -> DurabilityQosPolicy {
        self.durability
    }

    pub fn deadline(&self) -> DeadlineQosPolicy {
        self.deadline
    }

    pub fn reliability(&self) -> ReliabilityQosPolicy {
        self.reliability
    }

    pub fn lifespan(&self) -> LifespanQosPolicy {
        self.lifespan
    }

    pub fn history(&self) -> HistoryQosPolicy {
        self.history
    }

    pub fn liveness(&self) -> LivelinessQosPolicy {
        self.liveness
    }
}

impl From<InlineQos> for QosPolicy {
    fn from(value: InlineQos) -> Self {
        let InlineQos {
            durability,
            deadline,
            reliability,
            lifespan,
            history,
            liveness,
            ..
        } = value;
        QosPolicy {
            durability,
            deadline,
            reliability,
            lifespan,
            history,
            liveness,
        }
    }
}

impl From<QosPolicy> for InlineQos {
    fn from(value: QosPolicy) -> Self {
        let QosPolicy {
            durability,
            deadline,
            reliability,
            lifespan,
            history,
            liveness,
        } = value;
        InlineQos {
            durability,
            deadline,
            reliability,
            lifespan,
            history,
            liveness,
            ..Default::default()
        }
    }
}

#[derive(Debug, Default)]
pub struct QosPolicyBuilder {
    durability: Option<DurabilityQosPolicy>,
    deadline: Option<DeadlineQosPolicy>,
    reliability: Option<ReliabilityQosPolicy>,
    lifespan: Option<LifespanQosPolicy>,
    history: Option<HistoryQosPolicy>,
    liveness: Option<LivelinessQosPolicy>,
}

impl QosPolicyBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn durability(mut self, durability: DurabilityQosPolicy) -> Self {
        self.durability.replace(durability);
        self
    }

    pub fn reliability(mut self, reliability: ReliabilityQosPolicy) -> Self {
        self.reliability.replace(reliability);
        self
    }

    pub fn deadline(mut self, deadline: DeadlineQosPolicy) -> Self {
        self.deadline.replace(deadline);
        self
    }

    pub fn lifespan(mut self, lifespan: LifespanQosPolicy) -> Self {
        self.lifespan.replace(lifespan);
        self
    }

    pub fn history(mut self, history: HistoryQosPolicy) -> Self {
        self.history.replace(history);
        self
    }

    pub fn liveness(mut self, liveness: LivelinessQosPolicy) -> Self {
        self.liveness.replace(liveness);
        self
    }

    pub fn build(self) -> QosPolicy {
        QosPolicy {
            durability: self.durability.unwrap_or_default(),
            deadline: self.deadline.unwrap_or_default(),
            reliability: self.reliability.unwrap_or_default(),
            lifespan: self.lifespan.unwrap_or_default(),
            history: self.history.unwrap_or_default(),
            liveness: self.liveness.unwrap_or_default(),
        }
    }
}
