use crate::types::{DurabilityQosPolicy, InlineQos, LivelinessKind, ReliabilityQosPolicy};
use tracing::{Level, event, instrument};

use crate::DdsError;

#[derive(Debug)]
pub struct QosPolicyConsistencyChecker;

impl QosPolicyConsistencyChecker {
    #[instrument(level = Level::ERROR, "qos_match_checking")]
    pub fn check(writer_qos: &InlineQos, reader_qos: &InlineQos) -> Result<(), DdsError> {
        Self::check_topic_name(writer_qos, reader_qos)?;
        Self::check_topic_type(writer_qos, reader_qos)?;
        Self::check_durability(writer_qos, reader_qos)?;
        Self::check_deadline(writer_qos, reader_qos)?;
        Self::check_reliability(writer_qos, reader_qos)?;
        Self::check_liveness(writer_qos, reader_qos)?;
        Ok(())
    }

    fn check_topic_name(writer_qos: &InlineQos, reader_qos: &InlineQos) -> Result<(), DdsError> {
        if writer_qos.topic_name != reader_qos.topic_name {
            return Err(DdsError::InconsistentPolicy);
        }

        Ok(())
    }

    fn check_topic_type(writer_qos: &InlineQos, reader_qos: &InlineQos) -> Result<(), DdsError> {
        if writer_qos.type_name != reader_qos.type_name {
            return Err(DdsError::InconsistentPolicy);
        }

        Ok(())
    }

    fn check_durability(writer_qos: &InlineQos, reader_qos: &InlineQos) -> Result<(), DdsError> {
        let offered = writer_qos.durability;
        let requested = reader_qos.durability;

        match (offered, requested) {
            (DurabilityQosPolicy::Volatile, DurabilityQosPolicy::Volatile)
            | (DurabilityQosPolicy::TransientLocal, DurabilityQosPolicy::Volatile) => Ok(()),
            _ => Err(DdsError::InconsistentPolicy),
        }
    }

    fn check_deadline(writer_qos: &InlineQos, reader_qos: &InlineQos) -> Result<(), DdsError> {
        let offered = writer_qos.deadline;
        let requested = reader_qos.deadline;

        if offered <= requested {
            Ok(())
        } else {
            Err(DdsError::InconsistentPolicy)
        }
    }

    fn check_reliability(writer_qos: &InlineQos, reader_qos: &InlineQos) -> Result<(), DdsError> {
        let offered = writer_qos.reliability;
        let requested = reader_qos.reliability;

        match (offered, requested) {
            (ReliabilityQosPolicy::BestEffort, ReliabilityQosPolicy::Reliable { .. }) => {
                event!(Level::TRACE, "Reliability Qos doesn't match");
                Err(DdsError::InconsistentPolicy)
            }
            _ => Ok(()),
        }
    }

    fn check_liveness(writer_qos: &InlineQos, reader_qos: &InlineQos) -> Result<(), DdsError> {
        let offered = writer_qos.liveness;
        let requested = reader_qos.liveness;

        if offered.lease_duration > requested.lease_duration {
            return Err(DdsError::InconsistentPolicy);
        }

        match (offered.kind, requested.kind) {
            (LivelinessKind::Automatic, LivelinessKind::Automatic) => Ok(()),
            (_, LivelinessKind::ManualByParticipant) => Ok(()),
            (LivelinessKind::ManualByTopic, LivelinessKind::ManualByTopic) => Ok(()),
            _ => Err(DdsError::InconsistentPolicy),
        }
    }
}
