use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib::clone;
use gtk::glib::translate::FromGlib;
use gtk::{
    gio, glib, Expression, Label, PropertyExpression, SignalListItemFactory, SingleSelection,
};
use poke_book::pokeapi::rustemon_client;
use poke_book::resource_object::NamedPokeResourceObject;
use poke_book::{pokeapi, tokoi_runtime};

use crate::application::ExampleApplication;
use crate::config::{APP_ID, PROFILE};

mod imp {
    use super::*;

    #[derive(Debug, gtk::CompositeTemplate)]
    #[template(resource = "/com/github/nozwock/PokeBook/ui/window.ui")]
    pub struct ExampleApplicationWindow {
        // #[template_child]
        // pub headerbar: TemplateChild<adw::HeaderBar>,
        #[template_child]
        pub group_choice: TemplateChild<gtk::DropDown>,
        #[template_child]
        pub items_search_entry: TemplateChild<gtk::SearchEntry>,
        #[template_child]
        pub browse_list: TemplateChild<gtk::ListView>,
        pub settings: gio::Settings,
    }

    impl Default for ExampleApplicationWindow {
        fn default() -> Self {
            Self {
                settings: gio::Settings::new(APP_ID),
                group_choice: Default::default(),
                items_search_entry: Default::default(),
                browse_list: Default::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ExampleApplicationWindow {
        const NAME: &'static str = "ExampleApplicationWindow";
        type Type = super::ExampleApplicationWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        // You must call `Widget`'s `init_template()` within `instance_init()`.
        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ExampleApplicationWindow {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            // Devel Profile
            if PROFILE == "Devel" {
                obj.add_css_class("devel");
            }

            // Load latest window state
            obj.load_window_size();
            obj.setup_ui();
        }
    }

    impl WidgetImpl for ExampleApplicationWindow {}
    impl WindowImpl for ExampleApplicationWindow {
        // Save window state on delete event
        fn close_request(&self) -> glib::Propagation {
            _ = self
                .obj()
                .save_window_size()
                .inspect_err(|e| tracing::warn!(%e, "Failed to save window state"));

            // Pass close request on to the parent
            self.parent_close_request()
        }
    }

    impl ApplicationWindowImpl for ExampleApplicationWindow {}
    impl AdwApplicationWindowImpl for ExampleApplicationWindow {}
}

glib::wrapper! {
    pub struct ExampleApplicationWindow(ObjectSubclass<imp::ExampleApplicationWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionMap, gio::ActionGroup, gtk::Root;
}

impl ExampleApplicationWindow {
    pub fn new(app: &ExampleApplication) -> Self {
        glib::Object::builder().property("application", app).build()
    }

    fn save_window_size(&self) -> Result<(), glib::BoolError> {
        let imp = self.imp();

        let (width, height) = self.default_size();

        imp.settings.set_int("window-width", width)?;
        imp.settings.set_int("window-height", height)?;

        imp.settings
            .set_boolean("is-maximized", self.is_maximized())?;

        Ok(())
    }

    fn load_window_size(&self) {
        let imp = self.imp();

        let width = imp.settings.int("window-width");
        let height = imp.settings.int("window-height");
        let is_maximized = imp.settings.boolean("is-maximized");

        self.set_default_size(width, height);

        if is_maximized {
            self.maximize();
        }
    }

    fn setup_ui(&self) {
        let imp = self.imp();

        let group_model = adw::EnumListModel::new(pokeapi::ResourceGroup::static_type());
        imp.group_choice.set_model(Some(&group_model));
        imp.group_choice
            .set_expression(Some(PropertyExpression::new(
                group_model.item_type(),
                None::<Expression>,
                "name",
            )));

        // TODO: Bind DropDown label to SearchEntry's
        // .bind_property(
        //     "name",
        //     imp.items_search_entry
        //         .downcast_ref::<gtk::SearchEntry>()
        //         .expect("The value is of type gtk::SearchEntry"),
        //     "placeholder-text",
        // )
        // .transform_to(|_, s: String| Some(format!("Search in {s}")))
        // .sync_create()
        // .build();

        async fn get_entries(group: pokeapi::ResourceGroup) -> anyhow::Result<Vec<String>> {
            // TODO: Time to macro?
            Ok(match group {
                pokeapi::ResourceGroup::Pokemon => {
                    rustemon::pokemon::pokemon::get_all_entries(rustemon_client())
                        .await?
                        .into_iter()
                        .map(|it| it.name)
                        .collect()
                }
                pokeapi::ResourceGroup::Moves => {
                    rustemon::moves::move_::get_all_entries(rustemon_client())
                        .await?
                        .into_iter()
                        .map(|it| it.name)
                        .collect()
                }
            })
        }

        let (tx, rx) = async_channel::unbounded::<anyhow::Result<Vec<String>>>();

        imp.group_choice
            .connect_selected_item_notify(move |choice| {
                let group = unsafe { pokeapi::ResourceGroup::from_glib(choice.selected() as i32) };
                tokoi_runtime().spawn(clone!(@strong tx => async move {
                    _ = tx.send(get_entries(group).await).await.inspect_err(|e| tracing::error!(%e));
                }));
            });
        imp.group_choice.notify("selected-item");

        let browse_list = imp
            .browse_list
            .downcast_ref::<gtk::ListView>()
            .expect("Value has to be a ListView");
        glib::spawn_future_local(clone!(@strong rx, @weak browse_list => async move {
            while let Ok(it) = rx.recv().await {
                if let Ok(it) = it {
                    // dbg!(&it);
                    let objs = it.into_iter().map(|it| NamedPokeResourceObject::new(it)).collect::<Vec<_>>();
                    let browse_model = gio::ListStore::new::<NamedPokeResourceObject>();
                    browse_model.extend_from_slice(&objs);

                    let factory = SignalListItemFactory::new();
                    factory.connect_setup(move |_, list_item| {
                        let label = Label::builder().css_classes(["osd", "pad-8"]).build();
                        list_item
                            .downcast_ref::<gtk::ListItem>()
                            .expect("Value has to be a ListItem")
                            .set_child(Some(&label));
                    });
                    factory.connect_bind(move |_, list_item| {
                        let resource = list_item
                            .downcast_ref::<gtk::ListItem>()
                            .expect("Value has to be a ListItem")
                            .item()
                            .and_downcast::<NamedPokeResourceObject>()
                            .expect("Value has to be a NamedPokeResourceObject");

                        let label = list_item
                            .downcast_ref::<gtk::ListItem>()
                            .expect("Value has to be a ListItem")
                            .child()
                            .and_downcast::<Label>()
                            .expect("Value has to be a Label");

                        label.set_label(&resource.name());
                    });

                    let selection_model = SingleSelection::new(Some(browse_model));
                    browse_list.set_model(Some(&selection_model));
                    browse_list.set_factory(Some(&factory));
                }
            }
        }));
    }
}
