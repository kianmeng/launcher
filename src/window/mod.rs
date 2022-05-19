use super::consts::*;
use crate::crab_row::CrabRow;
use gtk::glib::{clone, Object};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{CustomFilter, FilterListModel, gdk, gio, Inhibit, SignalListItemFactory, SingleSelection};
use gtk::{glib, Application, FilterChange};
use gtk::gio::{AppInfo};
use gtk::gdk::AppLaunchContext;

mod imp;

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Window {
    pub fn new(app: &Application) -> Self {
        Object::new(&[("application", app)]).expect("Failed to create `Window`.")
    }

    fn current_items(&self) -> gio::ListStore {
        self.imp()
            .current_items
            .borrow()
            .clone()
            .expect(ERROR_ITEMS)
    }

    fn setup_window(&self) {
        let model = gio::ListStore::new(gio::AppInfo::static_type());
        gio::AppInfo::all().iter().for_each(|app_info| {
            model.append(app_info);
        });

        self.imp().current_items.replace(Some(model));

        let filter = CustomFilter::new(clone!(@strong self as window => move |obj| {
            let crab_entry = obj.downcast_ref::<gio::AppInfo>().unwrap();
            let search = window.imp().entry.buffer().text();

            if !search.is_empty() { crab_entry.name().to_lowercase().contains(&search.as_str().to_lowercase()) } else { true }
        }));

        let filter_model = FilterListModel::new(Some(&self.current_items()), Some(&filter));

        let sorter = gtk::CustomSorter::new(move |obj1, obj2| {
            let app_info1 = obj1.downcast_ref::<gio::AppInfo>().unwrap();
            let app_info2 = obj2.downcast_ref::<gio::AppInfo>().unwrap();

            app_info1
                .name()
                .to_lowercase()
                .cmp(&app_info2.name().to_lowercase())
                .into()
        });
        let sorted_model = gtk::SortListModel::new(Some(&filter_model), Some(&sorter));

        let selection_model = SingleSelection::new(Some(&sorted_model));
        self.imp().crab_items_list.set_model(Some(&selection_model));

        self.imp().entry.connect_changed(clone!(@strong self as window => move |_| {
            filter.changed(FilterChange::Different);
        }));

        self.imp().entry.connect_activate(clone!(@weak self as window => move |_| {
            let mut selected_index = 0;

            for i in 0..selection_model.n_items() {
                let is_selected = selection_model.selection().contains(i);

                if is_selected {
                    break;
                }

                selected_index += 1;
            }

            let list_view = &window.imp().crab_items_list;

            let model = list_view.model().unwrap();
            let app_info = model
                .item(selected_index)
                .unwrap()
                .downcast::<gio::AppInfo>()
                .unwrap();

            let parent_window = list_view.root().unwrap().downcast::<gtk::Window>().unwrap();

            open_app(&app_info, &parent_window);

            window.close();
        }));

        self.imp().crab_items_list.connect_activate(clone!(@weak self as window => move |list_view, position| {
            let model = list_view.model().unwrap();
            let app_info = model
                .item(position)
                .unwrap()
                .downcast::<gio::AppInfo>()
                .unwrap();

            let parent_window = list_view.root().unwrap().downcast::<gtk::Window>().unwrap();

            open_app(&app_info, &parent_window);

            window.close();
        }));

        let controller = gtk::EventControllerKey::new();
        controller.connect_key_pressed(clone!(@strong self as window => move |_, key, keycode, _| {
            match keycode {
                116 => {
                    window.imp().crab_items_list.model().unwrap().select_item(1, false);
                }
                9 => {
                    window.close();
                }
                _ => {
                    if keycode != 50 && keycode != 62 {
                        window.imp().entry.grab_focus();
                    }

                    if !(key.is_lower() && key.is_upper()) {
                        if let Some(key_name) = key.name() {
                            let buffer = window.imp().entry.buffer();

                            let mut content = buffer.text();
                            content.push(key_name.chars().next().unwrap());

                            buffer.set_text(content.as_str());
                            window.imp().entry.set_position((content.len()) as i32);
                            window.imp().entry.set_placeholder_text(None);
                        }
                    }
                }
            }

            Inhibit(false)
        }));

        self.add_controller(&controller);
    }

    fn setup_factory(&self) {
        let factory = SignalListItemFactory::new();

        factory.connect_setup(move |_, list_item| {
            let task_row = CrabRow::new();
            list_item.set_child(Some(&task_row));
        });

        factory.connect_bind(move |_, list_item| {
            let app_info = list_item
                .item()
                .unwrap()
                .downcast::<gio::AppInfo>()
                .unwrap();

            let task_row = list_item.child().unwrap().downcast::<CrabRow>().unwrap();

            task_row.set_app_info(&app_info);
        });

        self.imp().crab_items_list.set_factory(Some(&factory));
    }
}

fn open_app(app_info: &AppInfo, parent_window: &gtk::Window) {
    let context = parent_window.display().app_launch_context();

    if let Err(err) = app_info.launch(&[], Some(&context)) {
        gtk::MessageDialog::builder()
            .text(&format!("Failed to start {}", app_info.name()))
            .secondary_text(&err.to_string())
            .message_type(gtk::MessageType::Error)
            .modal(true)
            .transient_for(parent_window)
            .build()
            .show();
    }
}