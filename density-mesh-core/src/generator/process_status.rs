use serde::{Deserialize, Serialize};

/// Live density mesh processing status.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ProcessStatus {
    /// There are no pending changes.
    Idle,
    /// There are changes waiting for processing.
    InProgress,
    /// Inner mesh was updated during last processing.
    MeshChanged,
}
