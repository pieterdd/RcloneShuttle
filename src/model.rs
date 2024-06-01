use std::iter::Iterator;

use time::OffsetDateTime;
use uuid::Uuid;

use crate::{globals::JOBS, path_tools::RclonePath};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd)]
pub enum RcloneJobType {
    Upload {
        local_path: RclonePath,
        remote_path: RclonePath,
    },
    Move {
        source_path: RclonePath,
        target_path: RclonePath,
    },
    Copy {
        source_path: RclonePath,
        target_path: RclonePath,
    },
    Rename(RclonePath),
    Delete(RclonePath),
    Open {
        remote_path: RclonePath,
        tmp_local_path: RclonePath,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum RcloneJobStatus {
    Ongoing,
    Finished,
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct RcloneJob {
    pub uuid: Uuid,
    pub status: RcloneJobStatus,
    pub r#type: RcloneJobType,
    pub started_at: OffsetDateTime,
}

impl RcloneJob {
    pub fn new(r#type: RcloneJobType) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            status: RcloneJobStatus::Ongoing,
            r#type,
            started_at: OffsetDateTime::now_utc(),
        }
    }

    pub fn set_status(&mut self, new_status: RcloneJobStatus) {
        self.status = new_status;
    }
}

pub fn get_ongoing_jobs() -> Vec<RcloneJob> {
    JOBS.read()
        .values()
        .filter(|&j| j.status == RcloneJobStatus::Ongoing)
        .cloned()
        .collect::<Vec<RcloneJob>>()
}

pub fn has_failed_jobs() -> bool {
    JOBS.read()
        .values()
        .filter(|&j| matches!(j.status, RcloneJobStatus::Failed(_)))
        .cloned()
        .count()
        > 0
}

#[derive(Debug, Clone, Default)]
pub enum FilePickerMode {
    #[default]
    Select,
    Move(RclonePath),
    Copy(RclonePath),
}
