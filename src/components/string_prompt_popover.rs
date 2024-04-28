use adw::glib::clone;
use adw::prelude::{BoxExt, ButtonExt, EditableExt, EntryExt, PopoverExt};
use relm4::adw;
use relm4::gtk::prelude::{OrientableExt, WidgetExt};
use relm4::gtk::{self};
use relm4::ComponentParts;
use relm4::ComponentSender;
use relm4::SimpleComponent;

#[derive(Debug, Default)]
pub struct StringPromptPopover {
    prompt: String,
    entry: gtk::Entry,
}

#[derive(Debug, Default)]
pub struct StringPromptPopoverInit {
    pub prompt: String,
}

#[derive(Debug)]
pub enum StringPromptPopoverInMsg {
    SubmitInput,
}

#[derive(Debug)]
pub enum StringPromptPopoverOutMsg {
    InputSubmitted(String),
}

#[relm4::component(pub)]
impl SimpleComponent for StringPromptPopover {
    type Init = StringPromptPopoverInit;
    type Input = StringPromptPopoverInMsg;
    type Output = StringPromptPopoverOutMsg;

    view! {
        #[root]
        gtk::Popover {
            #[wrap(Some)]
            set_child = &gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_valign: gtk::Align::Center,
                set_spacing: 10,

                gtk::Label {
                    #[watch]
                    set_text: &model.prompt,
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 10,

                    append: &model.entry,

                    gtk::Button {
                        set_label: "Submit",
                        connect_clicked => Self::Input::SubmitInput,
                    }
                }
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let entry = gtk::Entry::builder().build();
        entry.connect_activate(clone!(@strong sender => move |_| {
            sender.input(Self::Input::SubmitInput);
        }));
        let model = Self {
            prompt: init.prompt,
            entry,
        };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            Self::Input::SubmitInput => {
                sender
                    .output(Self::Output::InputSubmitted(String::from(
                        self.entry.text().as_str(),
                    )))
                    .expect("Could not broadcast submitted input");
                self.entry.set_text("");
            }
        }
    }
}
