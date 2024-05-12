use adw::glib::clone;
use adw::prelude::{AdwDialogExt, BoxExt, ButtonExt, EditableExt, EntryExt};
use relm4::gtk::prelude::{OrientableExt, WidgetExt};
use relm4::gtk::{self};
use relm4::ComponentSender;
use relm4::{adw, RelmWidgetExt};
use relm4::{Component, ComponentParts};

#[derive(Debug, Default)]
pub struct StringPromptDialog {
    title: String,
    prompt: String,
    submit_label: String,
    entry: gtk::Entry,
}

#[derive(Debug, Default)]
pub struct StringPromptDialogInit {
    pub title: String,
    pub prompt: String,
    pub submit_label: String,
}

#[derive(Debug)]
pub enum StringPromptDialogInMsg {
    SubmitInput,
}

#[derive(Debug)]
pub enum StringPromptDialogOutMsg {
    InputSubmitted(String),
}

#[relm4::component(pub)]
impl Component for StringPromptDialog {
    type Init = StringPromptDialogInit;
    type Input = StringPromptDialogInMsg;
    type Output = StringPromptDialogOutMsg;
    type CommandOutput = ();

    view! {
        #[root]
        adw::Dialog {
            set_title: &model.title,
            set_can_close: true,
            #[wrap(Some)]
            set_child = &adw::ToolbarView {
                add_top_bar = &adw::HeaderBar {},

                #[wrap(Some)]
                set_content = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Center,
                    set_margin_all: 10,
                    set_spacing: 20,

                    gtk::Label {
                        #[watch]
                        set_text: &model.prompt,
                    },

                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 10,

                        append: &model.entry,

                        gtk::Button {
                            set_label: &model.submit_label,
                            connect_clicked => Self::Input::SubmitInput,
                        }
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
            title: init.title,
            prompt: init.prompt,
            submit_label: init.submit_label,
            entry,
        };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, root: &Self::Root) {
        match message {
            Self::Input::SubmitInput => {
                sender
                    .output(Self::Output::InputSubmitted(String::from(
                        self.entry.text().as_str(),
                    )))
                    .expect("Could not broadcast submitted input");
                self.entry.set_text("");
                root.close();
            }
        }
    }
}
