use relm4::factory::FactoryComponent;
use relm4::gtk::prelude::WidgetExt;
use relm4::gtk::{self};

use relm4::RelmWidgetExt;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct RemoteView {
    pub name: String,
}

#[relm4::factory(pub)]
impl FactoryComponent for RemoteView {
    type Init = String;
    type Input = ();
    type Output = ();
    type CommandOutput = ();
    type ParentWidget = gtk::ListBox;

    view! {
        gtk::Label {
            set_halign: gtk::Align::Start,
            set_margin_all: 10,
            set_text: &self.name,
        },
    }

    fn init_model(
        init: Self::Init,
        _index: &Self::Index,
        __sender: relm4::prelude::FactorySender<Self>,
    ) -> Self {
        Self { name: init }
    }
}
