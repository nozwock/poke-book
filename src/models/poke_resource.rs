use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;

mod imp {
    use std::cell::RefCell;

    use gtk::glib::Properties;

    use super::*;

    #[derive(Debug, Default, Properties)]
    #[properties(wrapper_type = super::NamedPokeResourceObject)]
    pub struct NamedPokeResourceObject {
        // https://github.com/gtk-rs/gtk-rs-core/issues/930
        // #[property(get, set, builder(ResourceGroup::default()))]
        // pub group: RefCell<ResourceGroup>,
        #[property(get, set)]
        pub name: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for NamedPokeResourceObject {
        const NAME: &'static str = "ResourceObject";
        type Type = super::NamedPokeResourceObject;
    }

    #[glib::derived_properties]
    impl ObjectImpl for NamedPokeResourceObject {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }
}

glib::wrapper! {
    pub struct NamedPokeResourceObject(ObjectSubclass<imp::NamedPokeResourceObject>);
}

impl NamedPokeResourceObject {
    pub fn new(name: String) -> Self {
        glib::Object::builder().property("name", name).build()
    }
}

impl Default for NamedPokeResourceObject {
    fn default() -> Self {
        glib::Object::builder().build()
    }
}
