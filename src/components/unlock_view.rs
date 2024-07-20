use adw::glib::clone;
use relm4::adw;
use relm4::gtk::prelude::{BoxExt, ButtonExt, EditableExt, OrientableExt, WidgetExt};
use relm4::gtk::{self};
use relm4::ComponentParts;
use relm4::ComponentSender;
use relm4::SimpleComponent;
use relm4_icons::icon_names;

#[derive(Debug, Default)]
pub struct UnlockView {
    password: String,
    password_input: gtk::PasswordEntry,
}

#[derive(Debug)]
#[allow(clippy::manual_non_exhaustive)]
#[allow(clippy::enum_variant_names)]
pub enum UnlockViewInMsg {
    PasswordFocusRequested,
    #[doc(hidden)]
    PasswordEdited(String),
    #[doc(hidden)]
    PasswordSubmitRequested,
}

#[derive(Debug)]
pub enum UnlockViewOutMsg {
    PasswordSubmitted(String),
}

#[relm4::component(pub)]
impl SimpleComponent for UnlockView {
    type Init = ();
    type Input = UnlockViewInMsg;
    type Output = UnlockViewOutMsg;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_valign: gtk::Align::Center,
            set_margin_bottom: 100,

            adw::StatusPage {
                set_title: "Config locked",
                set_icon_name: Some(icon_names::PADLOCK2),
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_halign: gtk::Align::Center,
                set_spacing: 10,

                append: &model.password_input,

                #[name = "submit_button"]
                gtk::Button {
                    set_icon_name: icon_names::RIGHT_LARGE,
                    connect_clicked => Self::Input::PasswordSubmitRequested,
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let password_input = gtk::PasswordEntry::builder().build();
        password_input.connect_changed(clone!(
            #[strong]
            sender,
            move |entry| {
                sender.input(Self::Input::PasswordEdited(entry.text().into()));
            }
        ));
        password_input.connect_activate(clone!(
            #[strong]
            sender,
            move |_| {
                sender.input(Self::Input::PasswordSubmitRequested);
            }
        ));

        let model = Self {
            password: String::from(""),
            password_input,
        };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            Self::Input::PasswordFocusRequested => {
                self.password_input.grab_focus();
            }
            Self::Input::PasswordEdited(password) => {
                self.password = password;
            }
            Self::Input::PasswordSubmitRequested => {
                sender
                    .output(Self::Output::PasswordSubmitted(self.password.clone()))
                    .unwrap();
            }
        }
    }
}
