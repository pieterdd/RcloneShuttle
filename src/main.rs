use crate::client::MkdirError;
use crate::components::file_listing_view::FileListingView;
use crate::components::remote_view::RemoteView;
use crate::components::unlock_view::{UnlockView, UnlockViewInMsg, UnlockViewOutMsg};
use crate::globals::FILE_PICKER_MODE;
use crate::gtk::DropTarget;
use crate::model::{FilePickerMode, RcloneJobStatus};
use adw::gio::Cancellable;
use adw::glib::clone;
use adw::gtk::ffi::GTK_INVALID_LIST_POSITION;
use adw::prelude::{ButtonExt, EditableExt, AdwDialogExt};
use client::{RcloneClient, RcloneFileListing};
use components::queue_button::QueueButton;
use components::string_prompt_dialog::{StringPromptDialog, StringPromptDialogInit, StringPromptDialogOutMsg};
use config::AppConfig;
use dirs::cache_dir;
use globals::JOBS;
use model::{RcloneJob, RcloneJobType};
use path_tools::RclonePath;
use relm4::actions::{AccelsPlus, RelmAction, RelmActionGroup};
use relm4::adw::prelude::{NavigationPageExt, AlertDialogExt};
use relm4::adw::ToolbarStyle;
use relm4::factory::FactoryVecDeque;
use relm4::gtk::gdk::{DragAction, FileList};
use relm4::gtk::prelude::{
    BoxExt, EntryExt, FileExt, GtkWindowExt, OrientableExt, StaticType, WidgetExt,
};
use relm4::gtk::{self};
use relm4::typed_view::list::TypedListView;
use relm4::ComponentParts;
use relm4::ComponentSender;
use relm4::Controller;
use relm4::RelmApp;
use relm4::RelmListBoxExt;
use relm4::{adw, ComponentController};
use relm4::{Component, RelmWidgetExt};
use relm4_icons::icon_names;
use std::ffi::OsString;
use std::ops::Deref;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

mod config;
mod client;
mod components;
mod globals;
mod model;
mod path_tools;

relm4::new_action_group!(FileListingsViewGroup, "file_listings_view");
relm4::new_stateless_action!(PathRefreshAction, FileListingsViewGroup, "path_refresh");
relm4::new_stateless_action!(MoveAction, FileListingsViewGroup, "move");
relm4::new_stateless_action!(CopyAction, FileListingsViewGroup, "copy");
relm4::new_stateless_action!(RenameAction, FileListingsViewGroup, "rename");
relm4::new_stateless_action!(DeleteAction, FileListingsViewGroup, "delete");
relm4::new_stateless_action!(PathParentAction, FileListingsViewGroup, "path_parent");
relm4::new_stateless_action!(PathUndoAction, FileListingsViewGroup, "path_undo");
relm4::new_stateless_action!(PathRedoAction, FileListingsViewGroup, "path_redo");

relm4::new_action_group!(MainWindowMenuViewGroup, "main_window");
relm4::new_stateless_action!(
    RemotesRefreshAction,
    MainWindowMenuViewGroup,
    "remotes_refresh"
);
relm4::new_stateless_action!(
    RemotesConfigureAction,
    MainWindowMenuViewGroup,
    "remotes_configure"
);

#[derive(Debug)]
pub enum AppInMsg {
    PasswordReceived(String),
    ClientConnectionFailed,
    ClientConnected(RcloneClient),
    RemotesRefreshRequested,
    RemotesConfigurationRequested,
    RemoteSelectionChanged(usize),
    PathRefreshRequested,
    PathParentRequested,
    PathUndoRequested,
    PathRedoRequested,
    PathEntered(RclonePath),
    PathChanged(RclonePath),
    OpenRequested(RclonePath),
    UploadRequested(RclonePath, RclonePath),
    FileListingSelectionChanged,
    FileListingPositionActivated(u32),
    FileListingErrorDetailRequested,
    FilesDropped(Vec<PathBuf>),
    ReturnToSelectModeRequested,
    CreateFolderRequested,
    CreateFolderConfirmed(String),
    MoveKeyPressed,
    CopyKeyPressed,
    RenameKeyPressed,
    MoveSelectionRequested,
    MoveTargetConfirmed,
    CopySelectionRequested,
    CopyTargetConfirmed,
    RenameSelectionRequested,
    RenameConfirmed(RclonePath, String),
    DeleteSelectionRequested,
    DeleteConfirmed(RclonePath, bool),
    TriggerGenericError(String, String, bool),
    FilePickerModeChange(FilePickerMode),
}

#[derive(Debug)]
enum AppOutCmd {
    FileListingAvailable(Vec<RcloneFileListing>),
    CommandFailed(String),
    JobUpdated(Uuid, RcloneJobStatus),
}

#[derive(Debug, Clone)]
enum FileListingViewState {
    Loading,
    Loaded,
    Error(String),
}

struct App {
    unlock_widget: Controller<UnlockView>,
    remotes_view_wrapper: FactoryVecDeque<RemoteView>,
    file_listing_view_wrapper: TypedListView<FileListingView, gtk::SingleSelection>,
    file_listing_view_state: FileListingViewState,
    queue_button: Controller<QueueButton>,
    path: RclonePath,
    undoable_paths: Vec<RclonePath>,
    redoable_paths: Vec<RclonePath>,
    client: Option<RcloneClient>,
    requires_password: bool,
    selected_file_listing_copy: Option<RcloneFileListing>,
    active_string_prompt: Option<Controller<StringPromptDialog>>,
}

impl App {
    fn refresh_remotes(&mut self, sender: &ComponentSender<App>) {
        let remotes = self.client.as_ref().unwrap().list_remotes().unwrap();
        self.remotes_view_wrapper.guard().clear();
        for (i, remote) in remotes.into_iter().enumerate() {
            if i == 0 {
                sender.input(AppInMsg::PathChanged(RclonePath::from(&remote)));
            }
            self.remotes_view_wrapper.guard().push_back(remote);
        }
        if let Some(list_box_row) = self.remotes_view_wrapper.widget().row_at_index(0) {
            self.remotes_view_wrapper
                .widget()
                .select_row(Some(&list_box_row));
        }
    }
}

#[relm4::component]
impl Component for App {
    type Init = ();
    type Input = AppInMsg;
    type Output = ();
    type CommandOutput = AppOutCmd;

    view! {
        #[name = "window"]
        adw::ApplicationWindow {
            set_title: Some("Rclone Shuttle"),
            set_default_size: (800, 600),

            adw::ToolbarView {
                set_top_bar_style: ToolbarStyle::Raised,
                add_top_bar = &adw::HeaderBar {
                    pack_start = &gtk::MenuButton {
                        set_icon_name: icon_names::MENU,
                        set_tooltip_text: Some("Menu"),
                        #[wrap(Some)]
                        set_popover = &gtk::PopoverMenu::from_model(Some(&main_menu)) {}
                    }
                },

                #[wrap(Some)]
                set_content = &gtk::Box {
                    set_hexpand: true,
                    set_vexpand: true,

                    if model.client.is_none() && model.requires_password {
                        adw::Clamp {
                            set_hexpand: true,
                            set_vexpand: true,
                            #[wrap(Some)]
                            set_child = model.unlock_widget.widget(),
                        }
                    } else if model.client.is_some() && model.remotes_view_wrapper.is_empty() {
                        adw::StatusPage {
                            set_title: "No remotes configured",
                            set_description: "Add a remote via 'rclone config'\nand then come back to browse it.".into(),
                            set_icon_name: Some(icon_names::INFO_OUTLINE),
                        }
                    } else {
                        adw::NavigationSplitView {
                            #[wrap(Some)]
                            set_sidebar = &adw::NavigationPage {
                                set_title: "Remotes",

                                #[wrap(Some)]
                                set_child = &gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,

                                    gtk::ScrolledWindow {
                                        #[local_ref]
                                        remotes_view -> gtk::ListBox {
                                            set_hexpand: true,
                                            set_vexpand: true,
                                            add_css_class: "navigation-sidebar",
                                            connect_row_activated[sender] => move |list_box, row| {
                                                if let Some(i) = list_box.index_of_child(row) {
                                                    sender.input(Self::Input::RemoteSelectionChanged(i as usize));
                                                }
                                            },
                                        },
                                    },

                                    model.queue_button.widget(),
                                }
                            },
                            #[wrap(Some)]
                            set_content = &adw::NavigationPage {
                                set_title: "Files",

                                #[wrap(Some)]
                                set_child = &adw::ToolbarView {
                                    set_vexpand: true,

                                    set_top_bar_style: ToolbarStyle::Raised,
                                    add_top_bar = &gtk::Box {
                                        set_height_request: 50,
                                        set_css_classes: &["toolbar"],

                                        gtk::Button {
                                            set_icon_name: icon_names::LEFT_LARGE,
                                            set_tooltip_text: Some("Previous (Alt+Left)"),
                                            connect_clicked => Self::Input::PathUndoRequested,
                                            #[watch]
                                            set_sensitive: !model.undoable_paths.is_empty(),
                                        },

                                        gtk::Button {
                                            set_icon_name: icon_names::RIGHT_LARGE,
                                            set_tooltip_text: Some("Next (Alt+Right)"),
                                            connect_clicked => Self::Input::PathRedoRequested,
                                            #[watch]
                                            set_sensitive: !model.redoable_paths.is_empty(),
                                        },

                                        gtk::Entry {
                                            set_hexpand: true,
                                            #[watch]
                                            set_text: &model.path.to_string(),
                                            set_margin_horizontal: 5,
                                            connect_activate[sender] => move |entry| {
                                                sender.input(Self::Input::PathEntered(RclonePath::from(entry.text().as_ref())));
                                            },
                                        },

                                        gtk::Button {
                                            set_icon_name: icon_names::ARROW_CIRCULAR_TOP_RIGHT,
                                            set_tooltip_text: Some("Refresh (F5)"),
                                            connect_clicked => Self::Input::PathRefreshRequested,
                                        },

                                        gtk::Button {
                                            set_icon_name: icon_names::UP_LARGE,
                                            set_tooltip_text: Some("Up one folder (Alt+Up)"),
                                            #[watch]
                                            set_sensitive: model.path.path_has_parent(),
                                            connect_clicked => Self::Input::PathParentRequested,
                                        },
                                    },

                                    #[wrap(Some)]
                                    set_content = &gtk::ScrolledWindow {
                                        set_vexpand: true,
                                        set_hscrollbar_policy: gtk::PolicyType::Never,
                                        #[wrap(Some)]
                                        set_child = match model.file_listing_view_state {
                                            FileListingViewState::Loading => {
                                                &gtk::Box {
                                                    set_halign: gtk::Align::Center,
                                                    set_valign: gtk::Align::Center,

                                                    gtk::Spinner {
                                                        set_spinning: true,
                                                        set_height_request: 30,
                                                        set_width_request: 30,
                                                    },
                                                }
                                            }
                                            FileListingViewState::Loaded => {
                                                &gtk::Box {
                                                    #[local_ref]
                                                    file_listing_view -> gtk::ListView {
                                                        set_hexpand: true,
                                                        inline_css: "background-color: transparent",
                                                        connect_activate[sender] => move |_, position| {
                                                            sender.input(Self::Input::FileListingPositionActivated(position));
                                                        },
                                                    }
                                                }
                                            }
                                            FileListingViewState::Error(_) => {
                                                &gtk::Box {
                                                    set_orientation: gtk::Orientation::Vertical,
                                                    set_halign: gtk::Align::Center,
                                                    set_valign: gtk::Align::Center,
                                                    set_hexpand: true,

                                                    adw::StatusPage {
                                                        set_height_request: 100,
                                                        set_hexpand: true,
                                                        set_title: "Error",
                                                        set_icon_name: Some(icon_names::WARNING_OUTLINE),
                                                    },

                                                    gtk::Button {
                                                        set_label: "Details",
                                                        connect_clicked => Self::Input::FileListingErrorDetailRequested,
                                                    }
                                                }
                                            }
                                        },
                                    },

                                    add_bottom_bar = &gtk::Box {
                                        set_orientation: gtk::Orientation::Horizontal,
                                        set_height_request: 50,
                                        set_css_classes: &["toolbar"],
                                        set_margin_horizontal: 5,

                                        gtk::Box {
                                            set_orientation: gtk::Orientation::Horizontal,
                                            set_halign: gtk::Align::Start,
                                            set_hexpand: true,

                                            gtk::Label {
                                                set_ellipsize: gtk::pango::EllipsizeMode::End,
                                                set_hexpand: true,
                                                #[watch]
                                                set_text: &match &FILE_PICKER_MODE.read().deref() {
                                                    FilePickerMode::Select => {
                                                        if let Some(listing) = &model.selected_file_listing_copy {
                                                            format!(
                                                                "\"{}\" selected{}",
                                                                listing.name,
                                                                match listing.formatted_size() {
                                                                    Some(size) => format!(" ({}B)", size),
                                                                    None => String::from(""),
                                                                }
                                                            )
                                                        } else {
                                                            String::from("")
                                                        }
                                                    }
                                                    FilePickerMode::Move(path) | FilePickerMode::Copy(path) => {
                                                        format!("Select folder for \"{}\"", path.filename().clone())
                                                    }
                                                }
                                            }
                                        },

                                        match FILE_PICKER_MODE.read().deref() {
                                            FilePickerMode::Select => {
                                                &gtk::Box {
                                                    set_orientation: gtk::Orientation::Horizontal,
                                                    set_halign: gtk::Align::End,
                                                    set_hexpand: true,
                                                    set_spacing: 5,

                                                    gtk::Button {
                                                        connect_clicked => AppInMsg::CreateFolderRequested,
                                                        add_css_class: "flat",

                                                        gtk::Box {
                                                            gtk::Label {
                                                                set_text: "New folder",
                                                            },
                                                        }
                                                    },
                                                    
                                                    gtk::MenuButton {
                                                        set_label: "Edit",
                                                        set_menu_model: Some(&file_listing_actions),
                                                    },

                                                }
                                            }
                                            FilePickerMode::Move(_) => {
                                                &gtk::Box {
                                                    set_orientation: gtk::Orientation::Horizontal,
                                                    set_halign: gtk::Align::End,
                                                    set_hexpand: true,
                                                    set_spacing: 5,

                                                    gtk::Button {
                                                        set_label: "Move here",
                                                        set_tooltip_text: Some("Confirm move action (F7)"),
                                                        connect_clicked => Self::Input::MoveTargetConfirmed,
                                                    },

                                                    gtk::Button {
                                                        set_label: "Cancel",
                                                        set_tooltip_text: Some("Cancel"),
                                                        connect_clicked => Self::Input::ReturnToSelectModeRequested,
                                                    },
                                                }
                                            }
                                            FilePickerMode::Copy(_) => {
                                                &gtk::Box {
                                                    set_orientation: gtk::Orientation::Horizontal,
                                                    set_halign: gtk::Align::End,
                                                    set_hexpand: true,
                                                    set_spacing: 5,

                                                    gtk::Button {
                                                        set_label: "Copy here",
                                                        set_tooltip_text: Some("Confirm copy action (F7)"),
                                                        connect_clicked => Self::Input::CopyTargetConfirmed,
                                                    },

                                                    gtk::Button {
                                                        set_label: "Cancel",
                                                        set_tooltip_text: Some("Cancel"),
                                                        connect_clicked => Self::Input::ReturnToSelectModeRequested,
                                                    },
                                                }
                                            }
                                        },
                                    }
                                }
                            }
                        }
                    },
                }
            }
        }
    }

    menu! {
        main_menu: {
            "Refresh remotes" => RemotesRefreshAction,
            "Configure remotes" => RemotesConfigureAction,
        },
        file_listing_actions: {
            "Rename" => RenameAction,
            "Move" => MoveAction,
            "Copy" => CopyAction,
            "Delete" => DeleteAction,
        }
    }

    fn init(
        _: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let unlock_widget =
            UnlockView::builder()
                .launch(())
                .forward(sender.input_sender(), |msg| match msg {
                    UnlockViewOutMsg::PasswordSubmitted(password) => {
                        AppInMsg::PasswordReceived(password)
                    }
                });
        let queue_button = QueueButton::builder().launch(()).detach();

        let remotes_view_wrapper = FactoryVecDeque::builder()
            .launch(gtk::ListBox::new())
            .detach();
        let file_listing_view_wrapper: TypedListView<FileListingView, gtk::SingleSelection> =
            TypedListView::new();
        file_listing_view_wrapper
            .selection_model
            .set_can_unselect(true);
        file_listing_view_wrapper
            .selection_model
            .connect_selected_item_notify(clone!(#[strong] sender, move |_| {
                sender.input(Self::Input::FileListingSelectionChanged);
            }));

        let drop_target = DropTarget::new(FileList::static_type(), DragAction::COPY);
        drop_target.connect_drop(clone!(#[strong] sender, move |_drop_target, value, _, _| {
            let file_list = value.get::<FileList>().expect("Non-file dropped");
            let file_paths: Vec<PathBuf> =
                file_list.files().iter().filter_map(|f| f.path()).collect();
            sender.input(Self::Input::FilesDropped(file_paths));
            true
        }));

        let rclone_config_file = std::env::var("RCLONE_CONFIG_FILE").ok();
        let mut requires_password = true;
        match RcloneClient::is_password_required(&rclone_config_file) {
            Ok(outcome) => {
                requires_password = outcome;
                if requires_password {
                    unlock_widget.emit(UnlockViewInMsg::PasswordFocusRequested);
                } else {
                    match RcloneClient::new(None, rclone_config_file) {
                        Ok(client) => sender.input(Self::Input::ClientConnected(client)),
                        Err(_) => sender.input(Self::Input::ClientConnectionFailed),
                    }
                }
            }
            Err(_) => {
                sender.input(Self::Input::ClientConnectionFailed);
            }
        }

        let model = App {
            unlock_widget,
            remotes_view_wrapper,
            file_listing_view_wrapper,
            file_listing_view_state: FileListingViewState::Loading,
            queue_button,
            path: RclonePath::from(""),
            undoable_paths: vec![],
            redoable_paths: vec![],
            client: None,
            requires_password,
            selected_file_listing_copy: None,
            active_string_prompt: None,
        };
        let remotes_view = model.remotes_view_wrapper.widget();
        let file_listing_view = &model.file_listing_view_wrapper.view;
        file_listing_view.add_controller(drop_target);
        let widgets = view_output!();

        let app = relm4::main_application();
        let rename_action: RelmAction<RenameAction> = {
            RelmAction::new_stateless(clone!(#[strong] sender, move |_| {
                sender.input(Self::Input::RenameKeyPressed);
            }))
        };
        let move_action: RelmAction<MoveAction> = {
            RelmAction::new_stateless(clone!(#[strong] sender, move |_| {
                sender.input(Self::Input::MoveKeyPressed);
            }))
        };
        let copy_action: RelmAction<CopyAction> = {
            RelmAction::new_stateless(clone!(#[strong] sender, move |_| {
                sender.input(Self::Input::CopyKeyPressed);
            }))
        };
        let delete_action: RelmAction<DeleteAction> = {
            RelmAction::new_stateless(
                clone!(#[strong] sender, move |_| {
                    sender.input(Self::Input::DeleteSelectionRequested);
                }),
            )
        };
        let path_refresh_action: RelmAction<PathRefreshAction> = {
            RelmAction::new_stateless(clone!(#[strong] sender, move |_| {
                sender.input(Self::Input::PathRefreshRequested);
            }))
        };
        let path_parent_action: RelmAction<PathParentAction> = {
            RelmAction::new_stateless(clone!(#[strong] sender, move |_| {
                sender.input(Self::Input::PathParentRequested);
            }))
        };
        let path_undo_action: RelmAction<PathUndoAction> = {
            RelmAction::new_stateless(clone!(#[strong] sender, move |_| {
                sender.input(Self::Input::PathUndoRequested);
            }))
        };
        let path_redo_action: RelmAction<PathRedoAction> = {
            RelmAction::new_stateless(clone!(#[strong] sender, move |_| {
                sender.input(Self::Input::PathRedoRequested);
            }))
        };
        let remotes_refresh_action: RelmAction<RemotesRefreshAction> = {
            RelmAction::new_stateless(clone!(#[strong] sender, move |_| {
                sender.input(Self::Input::RemotesRefreshRequested);
            }))
        };
        let remotes_configure_action: RelmAction<RemotesConfigureAction> = {
            RelmAction::new_stateless(clone!(#[strong] sender, move |_| {
                sender.input(Self::Input::RemotesConfigurationRequested);
            }))
        };

        app.set_accelerators_for_action::<RenameAction>(&["F2"]);
        app.set_accelerators_for_action::<MoveAction>(&["F6"]);
        app.set_accelerators_for_action::<CopyAction>(&["F7"]);
        app.set_accelerators_for_action::<DeleteAction>(&["<Shift>Delete"]);
        app.set_accelerators_for_action::<PathRefreshAction>(&["F5"]);
        app.set_accelerators_for_action::<PathParentAction>(&["<Alt>Up"]);
        app.set_accelerators_for_action::<PathUndoAction>(&["<Alt>Left"]);
        app.set_accelerators_for_action::<PathRedoAction>(&["<Alt>Right"]);

        let mut file_listings_view_group = RelmActionGroup::<FileListingsViewGroup>::new();
        file_listings_view_group.add_action(rename_action);
        file_listings_view_group.add_action(move_action);
        file_listings_view_group.add_action(copy_action);
        file_listings_view_group.add_action(delete_action);
        file_listings_view_group.add_action(path_refresh_action);
        file_listings_view_group.add_action(path_parent_action);
        file_listings_view_group.add_action(path_undo_action);
        file_listings_view_group.add_action(path_redo_action);
        file_listings_view_group.register_for_widget(&widgets.window);

        let mut main_menu_group = RelmActionGroup::<MainWindowMenuViewGroup>::new();
        main_menu_group.add_action(remotes_refresh_action);
        main_menu_group.add_action(remotes_configure_action);
        main_menu_group.register_for_widget(&widgets.window);

        FILE_PICKER_MODE.subscribe(sender.input_sender(), |new_mode| {
            AppInMsg::FilePickerModeChange(new_mode.clone())
        });

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, root: &Self::Root) {
        match message {
            Self::Input::PasswordReceived(password) => {
                let rclone_config_file = std::env::var("RCLONE_CONFIG_FILE").ok();
                if let Ok(client) = RcloneClient::new(Some(password), rclone_config_file) {
                    sender.input(Self::Input::ClientConnected(client));
                } else {
                    gtk::AlertDialog::builder()
                        .modal(true)
                        .message("Password incorrect")
                        .detail("Please try again.")
                        .build()
                        .show(Some(root));
                    self.unlock_widget
                        .emit(UnlockViewInMsg::PasswordFocusRequested);
                }
            }
            Self::Input::ClientConnectionFailed => {
                sender.input(AppInMsg::TriggerGenericError(String::from("Could not initialize rclone"), String::from("Please make sure rclone v1.66 or higher is installed and available from your system path."), true))
            }
            Self::Input::ClientConnected(client) => {
                // Assign the client and the remotes in one go to ensure the first remote is
                // preselected properly in the GUI
                self.client = Some(client);
                self.refresh_remotes(&sender);
                
                let mut config = AppConfig::load();
                
                if !config.skip_overwrite_disclaimer {
                    let dialog = gtk::AlertDialog::builder()
                        .buttons(["OK", "Don't show again"])
                        .message("Just so you know")
                        .detail("Rclone does not warn before overwriting files. If you upload, move or copy a file over another, Rclone will replace it without confirmation.")
                        .default_button(0)
                        .build();

                    dialog.choose(Some(root), Some(&Cancellable::default()), move |selection| {
                        if let Ok(1) = selection {
                            config.skip_overwrite_disclaimer = true;
                            config.save();
                        }
                    });
                }
            }
            Self::Input::RemotesRefreshRequested => {
                self.refresh_remotes(&sender);
            }
            Self::Input::RemotesConfigurationRequested => {
                let dialog = gtk::AlertDialog::builder()
                    .modal(true)
                    .buttons(["Got it"])
                    .message("Terminal time!")
                    .detail("The Rclone CLI can be used to configure a wide variety of cloud storage providers and protocols. Open up your favorite terminal and enter 'rclone config' to add, edit or delete remotes.")
                    .build();
                dialog.show(Some(root));
            }
            Self::Input::RemoteSelectionChanged(row) => {
                let raw_path = self.remotes_view_wrapper.get(row).unwrap().name.clone();
                let path = RclonePath::from(&raw_path);
                if path != self.path {
                    self.undoable_paths.push(self.path.clone());
                    self.redoable_paths.clear();
                }
                sender.input(Self::Input::PathChanged(path));
            }
            Self::Input::PathRefreshRequested => {
                sender.input(Self::Input::PathChanged(self.path.clone()));
            }
            Self::Input::PathParentRequested => {
                self.undoable_paths.push(self.path.clone());
                self.redoable_paths.clear();
                sender.input(Self::Input::PathChanged(self.path.resolve_to_parent()));
            }
            Self::Input::PathUndoRequested => {
                if let Some(path) = self.undoable_paths.pop() {
                    self.redoable_paths.push(self.path.clone());
                    sender.input(Self::Input::PathChanged(path));
                }
            }
            Self::Input::PathRedoRequested => {
                if let Some(path) = self.redoable_paths.pop() {
                    self.undoable_paths.push(self.path.clone());
                    sender.input(Self::Input::PathChanged(path));
                }
            }
            Self::Input::PathEntered(path) => {
                self.undoable_paths.push(self.path.clone());
                self.redoable_paths.clear();
                sender.input(Self::Input::PathChanged(path));
            }
            Self::Input::PathChanged(path) => {
                self.selected_file_listing_copy = None;
                self.path = path.clone();
                self.file_listing_view_state = FileListingViewState::Loading;
                self.file_listing_view_wrapper.clear();
                if let Some(remote) = path.remote() {
                    if let Some(position_of_match) = self
                        .remotes_view_wrapper
                        .iter()
                        .position(|item| item.name == remote)
                    {
                        let widget_to_select = self
                            .remotes_view_wrapper
                            .widget()
                            .row_at_index(position_of_match as i32)
                            .expect("Mismatched remote view");
                        self.remotes_view_wrapper
                            .widget()
                            .select_row(Some(&widget_to_select));
                    }
                }
                let client = self.client.clone();
                let path = path.clone();
                sender.spawn_oneshot_command(move || match client.as_ref().unwrap().ls(&path) {
                    Ok(listings) => AppOutCmd::FileListingAvailable(listings),
                    Err(error_str) => AppOutCmd::CommandFailed(error_str),
                });
            }
            Self::Input::OpenRequested(remote_path) => {
                let client = self.client.clone();

                let cache_dir = cache_dir()
                    .expect("Cannot find cache folder")
                    .join("rclone-shuttle");
                std::fs::create_dir_all(&cache_dir)
                    .expect("Could not make temporary download folder");
                let tmp_filename = format!(
                    "tmp{}-{}",
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_millis(),
                    remote_path.filename()
                );
                let tmp_local_path =
                    RclonePath::from(&cache_dir.into_os_string().into_string().unwrap())
                        .join(&tmp_filename);

                let job = RcloneJob::new(RcloneJobType::Open {
                    remote_path: remote_path.clone(),
                    tmp_local_path: tmp_local_path.clone(),
                });
                let uuid = job.uuid;
                JOBS.write().insert(job.uuid, job);

                sender.spawn_oneshot_command(move || {
                    let result = match client.as_ref().unwrap().copy(&remote_path, &tmp_local_path)
                    {
                        Ok(()) => AppOutCmd::JobUpdated(uuid, RcloneJobStatus::Finished),
                        Err(error_str) => {
                            AppOutCmd::JobUpdated(uuid, RcloneJobStatus::Failed(error_str))
                        }
                    };
                    result
                })
            }
            Self::Input::UploadRequested(local_path, remote_path) => {
                let client = self.client.clone();
                let job = RcloneJob::new(RcloneJobType::Upload {
                    local_path: local_path.clone(),
                    remote_path: remote_path.clone(),
                });
                let uuid = job.uuid;
                JOBS.write().insert(job.uuid, job);
                sender.spawn_oneshot_command(move || {
                    let result = match client.as_ref().unwrap().copy(&local_path, &remote_path) {
                        Ok(()) => AppOutCmd::JobUpdated(uuid, RcloneJobStatus::Finished),
                        Err(error_str) => {
                            AppOutCmd::JobUpdated(uuid, RcloneJobStatus::Failed(error_str))
                        }
                    };
                    result
                })
            }
            Self::Input::FileListingSelectionChanged => {
                let position = self.file_listing_view_wrapper.selection_model.selected();
                if let Some(item) = self.file_listing_view_wrapper.get(position) {
                    let listing_copy = item.borrow().model.clone();
                    self.selected_file_listing_copy = Some(listing_copy);
                }
            }
            Self::Input::FileListingPositionActivated(position) => {
                if let Some(item) = &self.file_listing_view_wrapper.get(position) {
                    let listing = &item.borrow().model;
                    if listing.is_dir {
                        self.undoable_paths.push(self.path.clone());
                        self.redoable_paths.clear();
                        sender.input(Self::Input::PathChanged(listing.path.clone()));
                    } else {
                        sender.input(Self::Input::OpenRequested(listing.path.clone()));
                    }
                }
            }
            Self::Input::FileListingErrorDetailRequested => {
                if let FileListingViewState::Error(error_str) = self.file_listing_view_state.clone()
                {
                    gtk::AlertDialog::builder()
                        .modal(true)
                        .message("Something went wrong")
                        .detail(error_str)
                        .build()
                        .show(Some(root));
                }
            }
            Self::Input::FilesDropped(file_paths) => {
                for file_path in file_paths {
                    let local_path =
                        RclonePath::from(&file_path.into_os_string().into_string().unwrap());
                    let local_filename = local_path.filename();
                    sender.input(Self::Input::UploadRequested(
                        local_path,
                        self.path.clone().join(&local_filename),
                    ));
                }
            }
            Self::Input::ReturnToSelectModeRequested => {
                *FILE_PICKER_MODE.write() = FilePickerMode::Select;
            }
            Self::Input::MoveKeyPressed => {
                match FILE_PICKER_MODE.read().deref() {
                    FilePickerMode::Select => sender.input(Self::Input::MoveSelectionRequested),
                    FilePickerMode::Move(_) => sender.input(Self::Input::MoveTargetConfirmed),
                    _ => {},
                };
            }
            Self::Input::CopyKeyPressed => {
                match FILE_PICKER_MODE.read().deref() {
                    FilePickerMode::Select => sender.input(Self::Input::CopySelectionRequested),
                    FilePickerMode::Copy(_) => sender.input(Self::Input::CopyTargetConfirmed),
                    _ => {},
                };
            }
            Self::Input::RenameKeyPressed => {
                if let FilePickerMode::Select = FILE_PICKER_MODE.read().deref() {
                    sender.input(Self::Input::RenameSelectionRequested);
                }
            }
            Self::Input::CreateFolderRequested => {
                let dialog = StringPromptDialog::builder().launch(StringPromptDialogInit {
                    title: String::from("New folder"),
                    prompt: String::from("Enter a name for the new folder."),
                    default_value: None,
                    submit_label: String::from("Create"),
                }).forward(sender.input_sender(), |msg| match msg {
                    StringPromptDialogOutMsg::InputSubmitted(folder_name) => AppInMsg::CreateFolderConfirmed(folder_name),
                });
                dialog.widget().present(Some(root));
                self.active_string_prompt = Some(dialog);
            }
            Self::Input::CreateFolderConfirmed(folder_name) => {
                if let Some(client) = &self.client {
                    let path = self.path.join(&folder_name);
                    match client.mkdir(&path) {
                        Ok(()) => {
                            sender.input(Self::Input::PathRefreshRequested);
                        }
                        Err(MkdirError::NotAvailableHere) => {
                            sender.input(Self::Input::TriggerGenericError(
                                String::from("Cannot create empty folder"),
                                String::from("This may be a technical limitation of your storage provider, typically with object storage like Amazon S3.\n\nTo persist the folder, upload a file to it before leaving."),
                                false,
                            ));
                            if path != self.path {
                                self.undoable_paths.push(self.path.clone());
                                self.redoable_paths.clear();
                            }
                            sender.input(Self::Input::PathChanged(path.clone()));
                        }
                        Err(MkdirError::Generic(error_str)) => {
                            sender.input(Self::Input::TriggerGenericError(
                                String::from("Something went wrong"),
                                error_str,
                                false,
                            ));
                        }
                    }
                }
            }
            Self::Input::MoveSelectionRequested => {
                let position = self.file_listing_view_wrapper.selection_model.selected();
                if let Some(item) = self.file_listing_view_wrapper.get(position) {
                    let path = item.borrow().model.path.clone();
                    *FILE_PICKER_MODE.write() = FilePickerMode::Move(path.clone());
                }
            }
            Self::Input::CopySelectionRequested => {
                let position = self.file_listing_view_wrapper.selection_model.selected();
                if let Some(item) = self.file_listing_view_wrapper.get(position) {
                    let path = item.borrow().model.path.clone();
                    *FILE_PICKER_MODE.write() = FilePickerMode::Copy(path.clone());
                }
            }
            Self::Input::MoveTargetConfirmed => {
                if let FilePickerMode::Move(path) = &FILE_PICKER_MODE.read().deref() {
                    let source_path = path.clone();
                    let target_path = self.path.join(&path.filename());
                    relm4::spawn_local(async {
                        // Don't read and write in the same cycle to avoid deadlock
                        *FILE_PICKER_MODE.write() = FilePickerMode::Select;
                    });

                    let client = self.client.clone();
                    let job = RcloneJob::new(RcloneJobType::Move {
                        source_path: source_path.clone(),
                        target_path: target_path.clone(),
                    });
                    let uuid = job.uuid;
                    JOBS.write().insert(job.uuid, job);
                    sender.spawn_oneshot_command(move || {
                        let result = match client.as_ref().unwrap().mv(&source_path, &target_path)
                        {
                            Ok(()) => AppOutCmd::JobUpdated(uuid, RcloneJobStatus::Finished),
                            Err(error_str) => {
                                AppOutCmd::JobUpdated(uuid, RcloneJobStatus::Failed(error_str))
                            }
                        };
                        result
                    })
                }
            }
            Self::Input::CopyTargetConfirmed => {
                if let FilePickerMode::Copy(path) = &FILE_PICKER_MODE.read().deref() {
                    let source_path = path.clone();
                    let target_path = self.path.join(&path.filename());
                    relm4::spawn_local(async {
                        // Don't read and write in the same cycle to avoid deadlock
                        *FILE_PICKER_MODE.write() = FilePickerMode::Select;
                    });

                    let client = self.client.clone();
                    let job = RcloneJob::new(RcloneJobType::Copy {
                        source_path: source_path.clone(),
                        target_path: target_path.clone(),
                    });
                    let uuid = job.uuid;
                    JOBS.write().insert(job.uuid, job);
                    sender.spawn_oneshot_command(move || {
                        let result = match client.as_ref().unwrap().copy(&source_path, &target_path)
                        {
                            Ok(()) => AppOutCmd::JobUpdated(uuid, RcloneJobStatus::Finished),
                            Err(error_str) => {
                                AppOutCmd::JobUpdated(uuid, RcloneJobStatus::Failed(error_str))
                            }
                        };
                        result
                    })
                }
            }
            Self::Input::RenameSelectionRequested => {
                let position = self.file_listing_view_wrapper.selection_model.selected();
                if let Some(item) = self.file_listing_view_wrapper.get(position) {
                    let path = item.borrow().model.path.clone();
                    let dialog = StringPromptDialog::builder().launch(StringPromptDialogInit {
                        title: format!("Rename '{}'", path.filename()),
                        prompt: String::from("Enter a new name to proceed."),
                        default_value: Some(path.filename()),
                        submit_label: String::from("Confirm"), 
                    }).forward(sender.input_sender(), move |msg| match msg {
                        StringPromptDialogOutMsg::InputSubmitted(new_filename) => Self::Input::RenameConfirmed(path.clone(), new_filename),
                    });
                    dialog.widget().present(Some(root));
                    self.active_string_prompt = Some(dialog);
                }
            }
            Self::Input::RenameConfirmed(path, new_filename) => {
                let client = self.client.clone();
                let job = RcloneJob::new(RcloneJobType::Rename(path.resolve_to_parent().join(&new_filename)));
                let uuid = job.uuid;
                JOBS.write().insert(job.uuid, job);
                sender.spawn_oneshot_command(move || {
                    let result = match client.as_ref().unwrap().rename(&path, &new_filename) {
                        Ok(()) => AppOutCmd::JobUpdated(uuid, RcloneJobStatus::Finished),
                        Err(error_str) => {
                            AppOutCmd::JobUpdated(uuid, RcloneJobStatus::Failed(error_str))
                        }
                    };
                    result
                });
            }
            Self::Input::DeleteSelectionRequested => {
                let position = self.file_listing_view_wrapper.selection_model.selected();
                if let Some(item) = self.file_listing_view_wrapper.get(position) {
                    let path = item.borrow().model.path.clone();
                    let is_dir = item.borrow().model.is_dir;
                    let alert = adw::AlertDialog::builder()
                        .heading(format!("Deleting '{}'", path.filename()))
                        .body(match is_dir {
                            true => "Are you sure? This will permanently delete the entire folder.",
                            false => "Are you sure? This is permanent.",
                        }).build();
                    alert.add_response("delete", "Delete");
                    alert.add_response("cancel", "Cancel");
                    alert.set_response_appearance("delete", adw::ResponseAppearance::Destructive);
                    alert.connect_response(Some("delete"), move |_, _| {
                        sender.input(Self::Input::DeleteConfirmed(path.clone(), is_dir));
                    });
                    alert.present(Some(root));
                }
            }
            Self::Input::DeleteConfirmed(path, is_dir) => {
                let client = self.client.clone();
                let job = RcloneJob::new(RcloneJobType::Delete(path.clone()));
                let uuid = job.uuid;
                JOBS.write().insert(job.uuid, job);
                sender.spawn_oneshot_command(move || {
                    let result = match client.as_ref().unwrap().rm(&path, is_dir) {
                        Ok(()) => AppOutCmd::JobUpdated(uuid, RcloneJobStatus::Finished),
                        Err(error_str) => {
                            AppOutCmd::JobUpdated(uuid, RcloneJobStatus::Failed(error_str))
                        }
                    };
                    result
                })
            }
            Self::Input::TriggerGenericError(error_title, error_description, fatal) => {
                let alert = gtk::AlertDialog::builder()
                    .modal(true)
                    .message(error_title)
                    .detail(error_description)
                    .build();
                if fatal {
                    alert.choose(Some(root), Some(&Cancellable::default()), |_| {
                        std::process::exit(1);
                    })
                } else {
                    alert.show(Some(root));
                }
            }
            Self::Input::FilePickerModeChange(new_mode) => match new_mode {
                FilePickerMode::Select => {}
                _ => {
                    self.file_listing_view_wrapper
                        .selection_model
                        .set_selected(GTK_INVALID_LIST_POSITION);
                }
            },
        }
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            AppOutCmd::FileListingAvailable(listings) => {
                self.file_listing_view_wrapper.clear();
                let mut listings_copy = listings.clone();
                listings_copy.sort_by(|a, b| {
                    if a.is_dir && !b.is_dir {
                        return std::cmp::Ordering::Less;
                    }
                    if !a.is_dir && b.is_dir {
                        return std::cmp::Ordering::Greater;
                    }
                    a.name.cmp(&b.name)
                });

                for (index, listing) in listings_copy.into_iter().enumerate() {
                    let is_dir = listing.is_dir;
                    self.file_listing_view_wrapper
                        .append(FileListingView::new(listing));
                    if index == 0 && !is_dir {
                        if let FilePickerMode::Copy(_) = &FILE_PICKER_MODE.read().deref() {
                            // Avoid selecting grayed out files
                            self.file_listing_view_wrapper
                                .selection_model
                                .set_selected(GTK_INVALID_LIST_POSITION);
                        }
                    }
                }
                self.file_listing_view_state = FileListingViewState::Loaded;
            }
            AppOutCmd::CommandFailed(error_str) => {
                self.file_listing_view_state = FileListingViewState::Error(error_str.clone());
            }
            AppOutCmd::JobUpdated(uuid, status) => {
                if let Some(job) = JOBS.write().get_mut(&uuid) {
                    job.set_status(status.clone());

                    if let RcloneJobType::Open { tmp_local_path, .. } = &job.r#type {
                        open::that_in_background(OsString::from(&tmp_local_path.to_string()));
                    } else if status != RcloneJobStatus::Ongoing {
                        sender.input(Self::Input::PathChanged(self.path.clone()));
                    }
                }
            }
        }
    }
}

fn main() {
    let app = RelmApp::new("io.github.pieterdd.RcloneShuttle");
    relm4_icons::initialize_icons();
    app.run::<App>(());
}
