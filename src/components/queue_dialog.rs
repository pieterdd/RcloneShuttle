use adw::prelude::{AdwDialogExt, BoxExt};
use relm4::factory::FactoryVecDeque;
use relm4::gtk::prelude::{OrientableExt, WidgetExt};
use relm4::gtk::{self};
use relm4::ComponentSender;
use relm4::{adw, RelmWidgetExt};
use relm4::{Component, ComponentParts};

use crate::globals::JOBS;
use crate::model::RcloneJob;

use super::queue_detail_view::QueueDetailView;

#[derive(Debug)]
pub struct QueueDialog {
    queue_detail_views_wrapper: FactoryVecDeque<QueueDetailView>,
}

#[derive(Debug)]
pub enum QueueDialogInput {
    JobsUpdated,
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
                add_top_bar = &adw::HeaderBar {},

                #[wrap(Some)]
                set_content = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_width_request: 400,
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
            Self::Input::JobsUpdated => {
                Self::propagate_jobs_update(&mut self.queue_detail_views_wrapper);
            }
        }
    }
}
