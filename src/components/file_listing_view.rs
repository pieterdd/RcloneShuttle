use crate::client::RcloneFileListing;
use crate::globals::FILE_PICKER_MODE;
use crate::model::FilePickerMode;

use crate::icons::icon_names;
use relm4::adw::glib::clone;
use relm4::gtk::prelude::{BoxExt, WidgetExt};
use relm4::gtk::{self};
use relm4::typed_view::list::RelmListItem;
use relm4::RelmWidgetExt;

#[derive(Debug, PartialEq, Eq)]
pub struct FileListingView {
    pub(crate) model: RcloneFileListing,
}

impl FileListingView {
    pub fn new(model: RcloneFileListing) -> Self {
        Self { model }
    }
}

pub struct FileListingViewWidgets {
    image: gtk::Image,
    label: gtk::Label,
}

fn set_sensitivity(root: &gtk::Box, file_picker_mode: &FilePickerMode, is_dir: bool) -> bool {
    if let Some(parent) = root.parent() {
        parent.set_sensitive(matches!(file_picker_mode, FilePickerMode::Select) || is_dir);
    }
    false
}

impl RelmListItem for FileListingView {
    type Root = gtk::Box;
    type Widgets = FileListingViewWidgets;

    fn setup(_list_item: &gtk::ListItem) -> (Self::Root, Self::Widgets) {
        relm4::view! {
            root = gtk::Box {
                set_margin_all: 10,
                set_spacing: 10,

                #[name = "image"]
                gtk::Image {
                },

                #[name = "label"]
                gtk::Label {
                    set_halign: gtk::Align::Start,
                    set_ellipsize: gtk::pango::EllipsizeMode::End,
                },
            },
        }

        (root, Self::Widgets { label, image })
    }

    fn bind(&mut self, widgets: &mut Self::Widgets, root: &mut Self::Root) {
        let (tx, rx) = relm4::channel::<FilePickerMode>();
        let is_dir_copy = self.model.is_dir;
        let initial_mode = FILE_PICKER_MODE.read();
        relm4::spawn_local(clone!(
            #[strong]
            root,
            async move {
                set_sensitivity(&root, &initial_mode, is_dir_copy);
            }
        ));
        relm4::spawn_local(clone!(
            #[strong]
            root,
            async move {
                while let Some(mode) = rx.recv().await {
                    set_sensitivity(&root, &mode, is_dir_copy);
                }
            }
        ));
        FILE_PICKER_MODE.subscribe(&tx, |f| f.clone());

        widgets.image.set_icon_name(Some(match self.model.is_dir {
            true => icon_names::FOLDER_FILLED,
            false => icon_names::PAPER_FILLED,
        }));
        widgets.label.set_text(&self.model.name);
    }
}
