use std::collections::BTreeMap;

use adw::prelude::{AdwDialogExt, BoxExt};
use relm4::factory::FactoryVecDeque;
use relm4::gtk::prelude::{ButtonExt, OrientableExt, WidgetExt};
use relm4::gtk::{self};
use relm4::ComponentSender;
use relm4::{adw, RelmWidgetExt};
use relm4::{Component, ComponentParts};
use relm4_icons::icon_names;

use crate::globals::JOBS;
use crate::model::{RcloneJob, RcloneJobStatus};

use super::queue_detail_view::QueueDetailView;

#[derive(Debug)]
pub struct QueueDialog {
    queue_detail_views_wrapper: FactoryVecDeque<QueueDetailView>,
}

#[derive(Debug)]
pub enum QueueDialogInput {
    JobsUpdated,
    CleanNonOngoingJobs,
}

impl QueueDialog {
    fn propagate_jobs_update(queue_detail_views_wrapper: &mut FactoryVecDeque<QueueDetailView>) {
        let mut inner = queue_detail_views_wrapper.guard();
        inner.clear();

        let mut ordered_jobs = JOBS.read().values().cloned().collect::<Vec<RcloneJob>>();
        ordered_jobs.sort_by(|a, b| b.started_at.cmp(&a.started_at));

        for job in ordered_jobs {
            inner.push_back(job.uuid);
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
                add_top_bar = &adw::HeaderBar {
                    pack_start = &gtk::Button {
                        set_icon_name: icon_names::BRUSH,
                        #[watch]
                        set_sensitive: !model.queue_detail_views_wrapper.is_empty(),
                        set_tooltip_text: Some("Clear terminated jobs"),
                        connect_clicked => Self::Input::CleanNonOngoingJobs,
                    }
                },

                #[wrap(Some)]
                set_content = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_hexpand: true,
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Start,
                    set_margin_all: 10,
                    set_margin_bottom: 20,
                    set_spacing: 20,

                    #[local_ref]
                    queue_detail_list_view -> gtk::Box {
                        set_hexpand: true,
                        set_orientation: gtk::Orientation::Vertical,
                        #[watch]
                        set_visible: !model.queue_detail_views_wrapper.is_empty(),
                        set_spacing: 10,
                        set_margin_all: 5,
                    },

                    gtk::Label {
                        set_text: "Queue is empty",
                        #[watch]
                        set_visible: model.queue_detail_views_wrapper.is_empty().clone(),
                        set_margin_all: 5,
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
        let mut queue_detail_views_wrapper = FactoryVecDeque::builder()
            .launch(gtk::Box::default())
            .detach();
        Self::propagate_jobs_update(&mut queue_detail_views_wrapper);
        let model = Self {
            queue_detail_views_wrapper,
        };
        let queue_detail_list_view: &gtk::Box = model.queue_detail_views_wrapper.widget();
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
                Self::propagate_jobs_update(&mut self.queue_detail_views_wrapper);
            }
            Self::Input::JobsUpdated => {
                Self::propagate_jobs_update(&mut self.queue_detail_views_wrapper);
            }
        }
    }
}
