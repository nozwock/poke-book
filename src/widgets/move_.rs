use adw::subclass::prelude::*;
use gtk::glib;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/com/github/nozwock/PokeBook/ui/content-pages/move.ui")]
    pub struct MovePageContent {
        // #[template_child]
        // pub main_sprite: TemplateChild<gtk::Image>,
        #[template_child]
        pub name: TemplateChild<gtk::Label>,
        #[template_child]
        pub effect: TemplateChild<gtk::Label>,
        #[template_child]
        pub power: TemplateChild<gtk::Label>,
        #[template_child]
        pub accuracy: TemplateChild<gtk::Label>,
        #[template_child]
        pub pp: TemplateChild<gtk::Label>,
        #[template_child]
        pub type_: TemplateChild<gtk::Label>,
        // #[template_child]
        // pub moves_list: TemplateChild<gtk::FlowBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MovePageContent {
        const NAME: &'static str = "MovePageContent";
        type ParentType = gtk::Box;
        type Type = super::MovePageContent;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    // #[glib::derived_properties]
    impl ObjectImpl for MovePageContent {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }
    impl WidgetImpl for MovePageContent {}
    impl BoxImpl for MovePageContent {}
}

glib::wrapper! {
    pub struct MovePageContent(ObjectSubclass<imp::MovePageContent>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl MovePageContent {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }
}

impl Default for MovePageContent {
    fn default() -> Self {
        glib::Object::builder().build()
    }
}
