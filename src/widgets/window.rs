use crate::models::poke_resource::NamedPokeResourceObject;
use crate::pokeapi::rustemon_client;
use crate::widgets::pokemon::PokemonPageContent;
use crate::{pokeapi, skim_matcher, tokoi_runtime};
use adw::prelude::*;
use adw::subclass::prelude::*;
use fuzzy_matcher::FuzzyMatcher;
use gtk::glib::clone;
use gtk::{
    gdk, gio, glib, Expression, Label, PropertyExpression, SignalListItemFactory, SingleSelection,
};

use crate::application::ExampleApplication;
use crate::config::{APP_ID, PROFILE};

mod imp {
    use crate::widgets::pokemon::PokemonPageContent;

    use super::*;

    #[derive(Debug, gtk::CompositeTemplate)]
    #[template(resource = "/com/github/nozwock/PokeBook/ui/window.ui")]
    pub struct ExampleApplicationWindow {
        #[template_child]
        pub group_choice: TemplateChild<gtk::DropDown>,
        #[template_child]
        pub browse_entry: TemplateChild<gtk::SearchEntry>,
        #[template_child]
        pub browse_list: TemplateChild<gtk::ListView>,

        #[template_child]
        pub sidebar_split: TemplateChild<adw::NavigationSplitView>,
        #[template_child]
        pub sidebar_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub content_stack: TemplateChild<gtk::Stack>,

        #[template_child]
        pub pokemon_content: TemplateChild<PokemonPageContent>,

        pub settings: gio::Settings,
    }

    impl Default for ExampleApplicationWindow {
        fn default() -> Self {
            Self {
                settings: gio::Settings::new(APP_ID),
                group_choice: Default::default(),
                browse_entry: Default::default(),
                browse_list: Default::default(),
                sidebar_split: Default::default(),
                sidebar_stack: Default::default(),
                content_stack: Default::default(),
                pokemon_content: Default::default(),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ExampleApplicationWindow {
        const NAME: &'static str = "ExampleApplicationWindow";
        type Type = super::ExampleApplicationWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            PokemonPageContent::ensure_type();

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
            .browse_entry
            .downcast_ref::<gtk::SearchEntry>()
            .expect("Value has to be a SearchEntry");
        let sidebar_stack = imp.sidebar_stack.downcast_ref::<gtk::Stack>().unwrap();
        let content_stack = imp.content_stack.downcast_ref::<gtk::Stack>().unwrap();
        let group_choice = imp.group_choice.downcast_ref::<gtk::DropDown>().unwrap();

        let pokemon_content = imp
            .pokemon_content
            .downcast_ref::<PokemonPageContent>()
            .unwrap();
        let pokemon_content_imp = pokemon_content.imp();

        let group_model = adw::EnumListModel::new(pokeapi::ResourceGroup::static_type());
        group_choice.set_model(Some(&group_model));
        group_choice.set_expression(Some(PropertyExpression::new(
            group_model.item_type(),
            None::<Expression>,
            "name",
        )));
        group_choice
            .bind_property(
                "selected-item",
                imp.browse_entry
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

        // Sidebar
        let (tx, rx) = async_channel::unbounded::<anyhow::Result<Vec<String>>>();

        // make it "generic" later with enum variants
        let (content_tx, content_rx) = async_channel::unbounded::<anyhow::Result<ContentMessage>>();

        group_choice
            .connect_selected_item_notify(clone!(@weak sidebar_stack => move |choice| {
                // TODO: Show the loading page after figuring out how to defer it by 200-400ms?
                // sidebar_stack.set_visible_child_name("loading_page");
                let group = pokeapi::ResourceGroup::from(choice.selected());
                tracing::debug!(?group, "Entered group change handler");
                tokoi_runtime().spawn(clone!(@strong tx => async move {
                    _ = tx.send(get_all_entries(group).await).await.inspect_err(|e| tracing::error!(%e));
                }));
            }));
        group_choice.notify("selected-item");

        // Setting up ListView, Filters and stuff
        glib::spawn_future_local(
            clone!(@strong rx, @strong content_tx, @weak browse_list, @weak items_search_entry, @weak sidebar_stack, @weak content_stack, @weak pokemon_content_imp, @weak group_choice => async move {
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

                        // There seems to be some issue with search too.
                        // Sometimes, no entries are shown while the search entry is empty.
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

                        selection_model.connect_selected_item_notify(clone!(@strong content_tx, @weak content_stack, @weak group_choice, @weak pokemon_content_imp => move |model| {
                            content_stack.set_visible_child_name("loading_page");
                            // pokemon_content_imp.main_sprite.set_icon_name(Some("image-missing-symbolic"));

                            if let Some(it) = model.selected_item() {
                                let resource_name = it.downcast_ref::<NamedPokeResourceObject>().unwrap().name();
                                tracing::debug!(?resource_name, "Selected an item");

                                match pokeapi::ResourceGroup::from(group_choice.selected()) {
                                    pokeapi::ResourceGroup::Pokemon => {
                                        tokoi_runtime().spawn(clone!(@strong content_tx => async move {
                                            match rustemon::pokemon::pokemon::get_by_name(&resource_name, rustemon_client()).await {
                                                Ok(pokemon_model) => {
                                                    if let Some(ref it) = pokemon_model.sprites.other.official_artwork.front_default {
                                                        let bytes = glib::Bytes::from_owned(reqwest::get(it).await.unwrap().bytes().await.unwrap());
                                                        let texture = gtk::gdk::Texture::from_bytes(&bytes).unwrap();
                                                        _ = content_tx.send(Ok(ContentMessage::Pokemon((pokemon_model, texture)))).await;
                                                    };
                                                }
                                                Err(err) => {
                                                    // No idea why this fails? `err.map_err(anyhow::Error::new);`
                                                    _ = content_tx.send(Err(anyhow::anyhow!(err))).await;
                                                }
                                            };
                                        }));
                                    }
                                    _ => {}
                                };
                            }
                            // TODO: fetch resource and show it in its custom page
                        }));

                        browse_list.set_model(Some(&selection_model));
                        browse_list.set_factory(Some(&factory));

                        sidebar_stack.set_visible_child_name("browse_page");
                    } else {
                        sidebar_stack.set_visible_child_name("error_page");
                    }
                }
            }),
        );

        // Still not OK!
        // When multiple items are selected, only the latest one should be reflected on the page
        glib::spawn_future_local(
            clone!(@strong content_rx, @weak pokemon_content_imp, @weak content_stack => async move {
                while let Ok(it) = content_rx.recv().await {
                    match it {
                        Ok(msg) => match msg {
                            ContentMessage::Pokemon((model, texture)) => {
                                // Could also send model and texture in different message, that'd allow to load model first
                                // as fetching the sprite will probably take longer than that
                                pokemon_content_imp.name.set_label(&heck::AsTitleCase(model.name).to_string());
                                pokemon_content_imp.main_sprite.set_paintable(Some(&texture));
                                pokemon_content_imp.base_exp.set_label(&model.base_experience.unwrap().to_string());
                                pokemon_content_imp.height.set_label(&model.height.to_string());
                                pokemon_content_imp.weight.set_label(&model.weight.to_string());

                                content_stack.set_visible_child_name("pokemon_page");
                            }
                        }
                        Err(_err) => {}
                    }
                }
            }),
        );
    }
}

pub enum ContentMessage {
    Pokemon((rustemon::model::pokemon::Pokemon, gdk::Texture)),
}
