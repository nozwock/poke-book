use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::glib;

use crate::pokeapi::ResourceGroup;

mod imp {
    use std::cell::RefCell;

    use gtk::glib::Properties;

    use crate::pokeapi::ResourceGroup;

    use super::*;

    #[derive(Debug, Default, Properties)]
    #[properties(wrapper_type = super::ResourceObject)]
    pub struct ResourceObject {
        // https://github.com/gtk-rs/gtk-rs-core/issues/930
        #[property(get, set, builder(ResourceGroup::default()))]
        pub group: RefCell<ResourceGroup>,
        #[property(get, set)]
        pub names: RefCell<Vec<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ResourceObject {
        const NAME: &'static str = "ResourceObject";
        type Type = super::ResourceObject;
    }

    #[glib::derived_properties]
    impl ObjectImpl for ResourceObject {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }
}

glib::wrapper! {
    pub struct ResourceObject(ObjectSubclass<imp::ResourceObject>);
}

impl ResourceObject {
    pub fn new(resource_group: ResourceGroup, names: Vec<String>) -> Self {
        glib::Object::builder()
            .property("group", resource_group)
            .property("names", names)
            .build()
    }
}

impl Default for ResourceObject {
    fn default() -> Self {
        glib::Object::builder().build()
    }
}
