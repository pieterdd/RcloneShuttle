use crate::globals::JOBS;
use crate::model::{get_ongoing_jobs, has_failed_jobs, RcloneJob};
use relm4::factory::FactoryVecDeque;
use relm4::gtk::prelude::{BoxExt, ButtonExt, OrientableExt, PopoverExt, WidgetExt};
use relm4::gtk::{self};
use relm4::ComponentSender;
use relm4::SimpleComponent;
use relm4::{ComponentParts, RelmWidgetExt};
use relm4_icons::icon_names;

use super::queue_detail_view::QueueDetailView;

#[derive(Debug)]
pub struct QueueView {
    popover: gtk::Popover,
    queue_detail_views_wrapper: FactoryVecDeque<QueueDetailView>,
}

#[derive(Debug)]
pub enum QueueViewInMsg {
    JobsUpdated,
    PopoverRequested,
}

#[relm4::component(pub)]
impl SimpleComponent for QueueView {
    type Init = ();
    type Input = QueueViewInMsg;
    type Output = ();

    view! {
        #[root]
        gtk::Box {
            gtk::Button {
                #[watch]
                set_visible: JOBS.read().len() > 0,
                set_hexpand: true,
                set_tooltip_text: Some("View job queue"),
                set_height_request: 50,
                inline_css: "border-radius: 0",
                connect_clicked => QueueViewInMsg::PopoverRequested,

                gtk::Box {
                    set_valign: gtk::Align::Center,
                    set_spacing: 10,

                    if !get_ongoing_jobs().is_empty() {
                        gtk::Spinner {
                            set_spinning: true,
                        }
                    } else {
                        gtk::Image {
                            #[watch]
                            set_icon_name: if has_failed_jobs() {
                                Some(icon_names::ERROR_OUTLINE)
                            } else {
                                Some(icon_names::CHECK_ROUND_OUTLINE)
                            }
                        }
                    },

                    gtk::Label {
                        #[watch]
                        set_text: if !get_ongoing_jobs().is_empty() {
                            "Working"
                        } else if has_failed_jobs() {
                            "Error"
                        } else {
                            "Ready"
                        },
                        set_halign: gtk::Align::Start,
                    }
                },
            },

            #[local_ref]
            popover -> gtk::Popover {
                set_position: gtk::PositionType::Top,

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,

                    #[local_ref]
                    queue_detail_list_view -> gtk::Box {
                        set_hexpand: true,
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 10,
                        set_margin_all: 5,
                    }
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
        let queue_detail_views_wrapper = FactoryVecDeque::builder()
            .launch(gtk::Box::default())
            .detach();
        let popover = gtk::Popover::builder().build();
        let queue_detail_list_view: &gtk::Box = queue_detail_views_wrapper.widget();
        let widgets = view_output!();
        let model = Self {
            popover,
            queue_detail_views_wrapper,
        };

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            Self::Input::JobsUpdated => {
                self.queue_detail_views_wrapper.guard().clear();

                let mut ordered_jobs = JOBS.read().values().cloned().collect::<Vec<RcloneJob>>();
                ordered_jobs.sort_by(|a, b| b.started_at.cmp(&a.started_at));

                for job in ordered_jobs {
                    self.queue_detail_views_wrapper.guard().push_back(job.uuid);
                }

                if JOBS.read().len() == 0 {
                    self.popover.set_visible(false);
                }
            }
            Self::Input::PopoverRequested => {
                if self.popover.is_visible() {
                    self.popover.set_visible(false);
                } else {
                    self.popover.set_visible(true);
                    self.popover.present();
                }
            }
        }
    }
}
