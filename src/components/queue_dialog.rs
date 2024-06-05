use std::collections::BTreeMap;

use adw::prelude::{AdwDialogExt, BoxExt};
use relm4::adw;
use relm4::adw::prelude::{ListBoxRowExt, PreferencesGroupExt};
use relm4::factory::FactoryVecDeque;
use relm4::gtk::prelude::{ButtonExt, OrientableExt, WidgetExt};
use relm4::gtk::{self};
use relm4::ComponentSender;
use relm4::RelmWidgetExt;
use relm4::{Component, ComponentParts};

use crate::globals::JOBS;
use crate::model::{RcloneJob, RcloneJobStatus};

use super::queue_detail_view::QueueDetailView;

#[derive(Debug)]
pub struct QueueDialog {
    ongoing_queue_wrapper: FactoryVecDeque<QueueDetailView>,
    terminated_queue_wrapper: FactoryVecDeque<QueueDetailView>,
}

#[derive(Debug)]
pub enum QueueDialogInput {
    JobsUpdated,
    CleanNonOngoingJobs,
}

impl QueueDialog {
    fn propagate_jobs_update(
        ongoing_queue_wrapper: &mut FactoryVecDeque<QueueDetailView>,
        terminated_queue_wrapper: &mut FactoryVecDeque<QueueDetailView>,
    ) {
        ongoing_queue_wrapper.guard().clear();
        terminated_queue_wrapper.guard().clear();

        let mut ordered_jobs = JOBS.read().values().cloned().collect::<Vec<RcloneJob>>();
        ordered_jobs.sort_by(|a, b| b.started_at.cmp(&a.started_at));

        for job in ordered_jobs {
            if job.status == RcloneJobStatus::Ongoing {
                ongoing_queue_wrapper.guard().push_back(job.uuid);
            } else {
                terminated_queue_wrapper.guard().push_back(job.uuid);
            }
        }
    }
}

#[relm4::component(pub)]
impl Component for QueueDialog {
    type Init = ();
    type Input = QueueDialogInput;
    type Output = ();
    type CommandOutput = ();

    view! {
        #[root]
        adw::Dialog {
            set_title: "Job queue",
            set_can_close: true,
            #[wrap(Some)]
            set_child = &adw::ToolbarView {
                add_top_bar = &adw::HeaderBar {},

                #[wrap(Some)]
                set_content = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_hexpand: true,
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Start,
                    set_margin_all: 20,
                    set_margin_top: 0,
                    set_spacing: 20,
                    set_width_request: 500,

                    #[local_ref]
                    ongoing_queue_view -> adw::PreferencesGroup {
                        set_title: "Ongoing jobs",

                        adw::PreferencesRow {
                            #[watch]
                            set_visible: model.ongoing_queue_wrapper.is_empty(),

                            #[wrap(Some)]
                            set_child = &gtk::Box {
                                set_orientation: gtk::Orientation::Horizontal,
                                set_margin_all: 10,

                                gtk::Label {
                                    set_text: "Empty",
                                    set_opacity: 0.4,
                                },
                            }
                        }
                    },

                    #[local_ref]
                    terminated_queue_view -> adw::PreferencesGroup {
                        set_title: "Terminated jobs",
                        #[wrap(Some)]
                        set_header_suffix = &gtk::Button {
                            set_label: "Clear",
                            #[watch]
                            set_sensitive: !model.terminated_queue_wrapper.is_empty(),
                            set_tooltip_text: Some("Clear terminated jobs"),
                            connect_clicked => Self::Input::CleanNonOngoingJobs,
                        },

                        add = &adw::PreferencesRow {
                            #[watch]
                            set_visible: model.terminated_queue_wrapper.is_empty(),

                            #[wrap(Some)]
                            set_child = &gtk::Box {
                                set_orientation: gtk::Orientation::Horizontal,
                                set_margin_all: 10,

                                gtk::Label {
                                    set_text: "Empty",
                                    set_opacity: 0.4,
                                },
                            }
                        }
                    },
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        JOBS.subscribe(sender.input_sender(), |_| Self::Input::JobsUpdated);
        let mut ongoing_queue_wrapper = FactoryVecDeque::builder()
            .launch(adw::PreferencesGroup::default())
            .detach();
        let mut terminated_queue_wrapper = FactoryVecDeque::builder()
            .launch(adw::PreferencesGroup::default())
            .detach();
        Self::propagate_jobs_update(&mut ongoing_queue_wrapper, &mut terminated_queue_wrapper);
        let model = Self {
            ongoing_queue_wrapper,
            terminated_queue_wrapper,
        };
        let ongoing_queue_view = model.ongoing_queue_wrapper.widget();
        let terminated_queue_view = model.terminated_queue_wrapper.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            Self::Input::CleanNonOngoingJobs => {
                let mut new_jobs = BTreeMap::new();
                for job in JOBS.read().iter() {
                    if job.1.status == RcloneJobStatus::Ongoing {
                        new_jobs.insert(job.0.clone(), job.1.clone());
                    }
                }
                *JOBS.write() = new_jobs;
                Self::propagate_jobs_update(
                    &mut self.ongoing_queue_wrapper,
                    &mut self.terminated_queue_wrapper,
                );
            }
            Self::Input::JobsUpdated => {
                Self::propagate_jobs_update(
                    &mut self.ongoing_queue_wrapper,
                    &mut self.terminated_queue_wrapper,
                );
            }
        }
    }
}
