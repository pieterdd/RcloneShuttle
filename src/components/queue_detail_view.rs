use crate::globals::JOBS;
use crate::model::{RcloneJob, RcloneJobStatus, RcloneJobType};
use relm4::adw;
use relm4::adw::prelude::{ActionRowExt, ButtonExt, ListBoxRowExt, PreferencesRowExt, WidgetExt};
use relm4::factory::{DynamicIndex, FactoryComponent};
use relm4::{gtk, FactorySender};
use relm4_icons::icon_names;
use uuid::Uuid;

#[derive(Debug)]
pub struct QueueDetailView {
    uuid: Uuid,
    job_description: String,
    job_copy: RcloneJob,
}

#[derive(Debug)]
pub enum QueueDetailInMsg {
    #[doc(hidden)]
    JobUpdated,
    #[doc(hidden)]
    JobDeletionRequested,
}

impl QueueDetailView {
    fn make_job_description(job: &RcloneJob) -> String {
        match job.r#type.clone() {
            RcloneJobType::Upload {
                local_path: source,
                remote_path: dest,
            } => {
                format!(
                    "Upload {} to {}",
                    source.filename(),
                    dest.resolve_to_parent()
                )
            }
            RcloneJobType::Move {
                source_path,
                target_path,
            } => {
                format!(
                    "Move {} to {}",
                    source_path.filename(),
                    target_path.resolve_to_parent()
                )
            }
            RcloneJobType::Copy {
                source_path,
                target_path,
            } => {
                format!(
                    "Copy {} to {}",
                    source_path.filename(),
                    target_path.resolve_to_parent()
                )
            }
            RcloneJobType::Rename(path) => {
                format!("Rename {} in {}", path.filename(), path.resolve_to_parent())
            }
            RcloneJobType::Download {
                local_path,
                remote_path,
            } => {
                format!(
                    "Download {} to {}",
                    remote_path.filename(),
                    local_path.resolve_to_parent()
                )
            }
            RcloneJobType::Delete(path) => {
                format!(
                    "Delete {} from {}",
                    path.filename(),
                    path.resolve_to_parent()
                )
            }
            RcloneJobType::Open { remote_path, .. } => {
                format!(
                    "Open {} from {}",
                    remote_path.filename(),
                    remote_path.resolve_to_parent()
                )
            }
        }
    }
}

#[relm4::factory(pub)]
impl FactoryComponent for QueueDetailView {
    type Init = Uuid;
    type Input = QueueDetailInMsg;
    type Output = ();
    type CommandOutput = ();
    type Widgets = CounterWidgets;
    type ParentWidget = adw::PreferencesGroup;

    view! {
        #[root]
        adw::PreferencesRow {
            #[wrap(Some)]
            set_child = &adw::ActionRow {
                set_title: self.job_description.as_str(),
                set_activatable: false,
                set_selectable: false,
                add_prefix = if matches!(self.job_copy.status, RcloneJobStatus::Ongoing) {
                    gtk::Spinner {
                        set_spinning: true,
                        set_height_request: 20,
                        set_valign: gtk::Align::Center,
                    }
                } else {
                    gtk::Image {
                        set_pixel_size: 20,
                        set_icon_name: match self.job_copy.status {
                            RcloneJobStatus::Ongoing => None,
                            RcloneJobStatus::Finished => Some(icon_names::CHECK_ROUND_OUTLINE),
                            RcloneJobStatus::Failed(_) => Some(icon_names::ERROR_OUTLINE)
                        },
                        set_tooltip_text: match self.job_copy.status {
                            RcloneJobStatus::Ongoing => Some("Ongoing"),
                            RcloneJobStatus::Finished => Some("Finished"),
                            RcloneJobStatus::Failed(_) => Some("Failed")
                        },
                    }
                },
                add_suffix = &gtk::Button {
                    set_halign: gtk::Align::End,
                    set_valign: gtk::Align::Center,
                    set_has_frame: false,
                    set_icon_name: icon_names::MINUS_CIRCLE_FILLED,
                    set_tooltip_text: Some("Remove from queue"),
                    set_visible: self.job_copy.status != RcloneJobStatus::Ongoing,
                    connect_clicked => Self::Input::JobDeletionRequested,
                }
            }
        }
    }

    fn init_model(value: Self::Init, _index: &DynamicIndex, sender: FactorySender<Self>) -> Self {
        JOBS.subscribe(sender.input_sender(), |_| Self::Input::JobUpdated);
        let job_copy = JOBS.read().get(&value).unwrap().clone();
        Self {
            uuid: value,
            job_copy: job_copy.clone(),
            job_description: Self::make_job_description(&job_copy),
        }
    }

    fn update(&mut self, message: Self::Input, _sender: FactorySender<Self>) {
        match message {
            Self::Input::JobUpdated => {
                if let Some(job) = JOBS.read().get(&self.uuid) {
                    self.job_copy = job.clone();
                    self.job_description = Self::make_job_description(&self.job_copy);
                }
            }
            Self::Input::JobDeletionRequested => {
                JOBS.write().remove(&self.uuid);
            }
        }
    }
}
