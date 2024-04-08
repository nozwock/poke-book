use crate::models::poke_resource::NamedPokeResourceObject;
use crate::pokeapi::rustemon_client;
use crate::{pokeapi, skim_matcher, tokoi_runtime};
use adw::prelude::*;
use adw::subclass::prelude::*;
use fuzzy_matcher::FuzzyMatcher;
use gtk::glib::clone;
use gtk::glib::translate::FromGlib;
use gtk::{
    gio, glib, Expression, Label, PropertyExpression, SignalListItemFactory, SingleSelection,
};

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

        #[template_child]
        pub sidebar_split: TemplateChild<adw::NavigationSplitView>,
        #[template_child]
        pub sidebar_stack: TemplateChild<gtk::Stack>,

        pub settings: gio::Settings,
    }

    impl Default for ExampleApplicationWindow {
        fn default() -> Self {
            Self {
                settings: gio::Settings::new(APP_ID),
                group_choice: Default::default(),
                items_search_entry: Default::default(),
                browse_list: Default::default(),
                sidebar_split: Default::default(),
                sidebar_stack: Default::default(),
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

        let browse_list = imp
            .browse_list
            .downcast_ref::<gtk::ListView>()
            .expect("Value has to be a ListView");
        let items_search_entry = imp
            .items_search_entry
            .downcast_ref::<gtk::SearchEntry>()
            .expect("Value has to be a SearchEntry");
        let sidebar_stack = imp.sidebar_stack.downcast_ref::<gtk::Stack>().unwrap();

        let group_model = adw::EnumListModel::new(pokeapi::ResourceGroup::static_type());
        imp.group_choice.set_model(Some(&group_model));
        imp.group_choice
            .set_expression(Some(PropertyExpression::new(
                group_model.item_type(),
                None::<Expression>,
                "name",
            )));
        imp.group_choice
            .bind_property(
                "selected-item",
                imp.items_search_entry
                    .downcast_ref::<gtk::SearchEntry>()
                    .expect("The value is of type gtk::SearchEntry"),
                "placeholder-text",
            )
            .transform_to(|_, list_item: adw::EnumListItem| {
                Some(format!("Search in {}", list_item.name()))
            })
            .sync_create()
            .build();

        async fn get_all_entries(group: pokeapi::ResourceGroup) -> anyhow::Result<Vec<String>> {
            macro_rules! get_all_entries {
                ($endpoint:path) => {{
                    use $endpoint as path;
                    path::get_all_entries(rustemon_client())
                        .await?
                        .into_iter()
                        .map(|it| it.name)
                        .collect()
                }};
            }

            Ok(match group {
                pokeapi::ResourceGroup::Pokemon => {
                    get_all_entries!(rustemon::pokemon::pokemon)
                }
                pokeapi::ResourceGroup::Moves => {
                    get_all_entries!(rustemon::moves::move_)
                }
            })
        }

        let (tx, rx) = async_channel::unbounded::<anyhow::Result<Vec<String>>>();

        imp.group_choice
            .connect_selected_item_notify(clone!(@weak sidebar_stack => move |choice| {
                // TODO: Show the loading page after figuring out how to defer it by 200-400ms?
                // sidebar_stack.set_visible_child_name("sidebar_stack_empty_page");
                let group = unsafe { pokeapi::ResourceGroup::from_glib(choice.selected() as i32) };
                tokoi_runtime().spawn(clone!(@strong tx => async move {
                    _ = tx.send(get_all_entries(group).await).await.inspect_err(|e| tracing::error!(%e));
                }));
            }));
        imp.group_choice.notify("selected-item");

        // Setting up ListView, Filters and stuff
        glib::spawn_future_local(
            clone!(@strong rx, @weak browse_list, @weak items_search_entry, @weak sidebar_stack => async move {
                while let Ok(it) = rx.recv().await {
                    if let Ok(it) = it {
                        let objs = it.into_iter().map(|it| NamedPokeResourceObject::new(it)).collect::<Vec<_>>();
                        let browse_model = gio::ListStore::new::<NamedPokeResourceObject>();
                        browse_model.extend_from_slice(&objs);

                        let factory = SignalListItemFactory::new();
                        factory.connect_setup(move |_, list_item| {
                            let label = Label::builder().xalign(0.).build();
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


                            let name = heck::AsTitleCase(resource.name()).to_string();
                            label.set_label(&name);
                        });

                        let fuzzy_filter = gtk::CustomFilter::new(clone!(@weak items_search_entry => @default-return true, move |resource| {
                            let resource = resource
                                .downcast_ref::<NamedPokeResourceObject>()
                                .expect("Value has to be a NamedPokeResourceObject");

                            match items_search_entry.text().as_str().trim() {
                                "" => true,
                                s => {
                                    skim_matcher().fuzzy_match(&resource.name(), s).is_some()
                                }
                            }
                        }));

                        items_search_entry.connect_changed(clone!(@weak fuzzy_filter => move |_| {
                            fuzzy_filter.changed(gtk::FilterChange::Different);
                        }));

                        let filter_model = gtk::FilterListModel::new(Some(browse_model), Some(fuzzy_filter));

                        let selection_model = SingleSelection::new(None::<gtk::FilterListModel>);
                        selection_model.set_autoselect(false);
                        selection_model.set_model(Some(&filter_model));

                        selection_model.connect_selected_item_notify(move |model| {
                            if let Some(it) = model.selected_item() {
                                dbg!(it.downcast_ref::<NamedPokeResourceObject>().unwrap().name());
                            }
                            // TODO: fetch resource and show it in its custom page
                        });

                        browse_list.set_model(Some(&selection_model));
                        browse_list.set_factory(Some(&factory));

                        sidebar_stack.set_visible_child_name("sidebar_stack_browse_page");
                    } else {
                        sidebar_stack.set_visible_child_name("sidebar_stack_error_page");
                    }
                }
            }),
        );
    }
}