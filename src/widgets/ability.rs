use adw::subclass::prelude::*;
use gtk::glib;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/com/github/nozwock/PokeBook/ui/content-pages/ability.ui")]
    pub struct AbilityPageContent {
        #[template_child]
        pub name: TemplateChild<gtk::Label>,
        #[template_child]
        pub effect: TemplateChild<gtk::Label>,
        #[template_child]
        pub pokemon_list: TemplateChild<gtk::FlowBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for AbilityPageContent {
        const NAME: &'static str = "AbilityPageContent";
        type ParentType = gtk::Box;
        type Type = super::AbilityPageContent;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    // #[glib::derived_properties]
    impl ObjectImpl for AbilityPageContent {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }
    impl WidgetImpl for AbilityPageContent {}
    impl BoxImpl for AbilityPageContent {}
}

glib::wrapper! {
    pub struct AbilityPageContent(ObjectSubclass<imp::AbilityPageContent>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl AbilityPageContent {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }
}

impl Default for AbilityPageContent {
    fn default() -> Self {
        glib::Object::builder().build()
    }
}
