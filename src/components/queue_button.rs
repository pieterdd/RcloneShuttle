use crate::globals::JOBS;
use crate::model::{get_ongoing_jobs, has_failed_jobs};
use relm4::adw::prelude::AdwDialogExt;
use relm4::gtk::prelude::{BoxExt, ButtonExt, WidgetExt};
use relm4::gtk::{self};
use relm4::SimpleComponent;
use relm4::{Component, ComponentSender};
use relm4::{ComponentController, Controller};
use relm4::{ComponentParts, RelmWidgetExt};
use relm4_icons::icon_names;

use super::queue_dialog::QueueDialog;

#[derive(Debug)]
pub struct QueueButton {
    root: gtk::Box,
    dialog: Controller<QueueDialog>,
}

#[derive(Debug)]
pub enum QueueViewInMsg {
    DialogRequested,
    JobsUpdated,
}

#[relm4::component(pub)]
impl SimpleComponent for QueueButton {
    type Init = ();
    type Input = QueueViewInMsg;
    type Output = ();

    view! {
        #[root]
        gtk::Box {
            gtk::Button {
                #[watch]
                set_hexpand: true,
                set_tooltip_text: Some("View job queue"),
                set_height_request: 50,
                inline_css: "border-radius: 0",
                connect_clicked => QueueViewInMsg::DialogRequested,

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
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        JOBS.subscribe(sender.input_sender(), |_| Self::Input::JobsUpdated);
        let widgets = view_output!();
        let dialog = QueueDialog::builder().launch(()).detach();
        let model = Self { root, dialog };

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            Self::Input::JobsUpdated => {}
            Self::Input::DialogRequested => {
                self.dialog.widget().present(&self.root);
            }
        }
    }
}
