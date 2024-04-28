use std::collections::BTreeMap;

use relm4::SharedState;
use uuid::Uuid;

use crate::model::{FilePickerMode, RcloneJob};

pub(crate) static JOBS: SharedState<BTreeMap<Uuid, RcloneJob>> = SharedState::new();

pub(crate) static FILE_PICKER_MODE: SharedState<FilePickerMode> = SharedState::new();
