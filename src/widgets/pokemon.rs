use adw::subclass::prelude::*;
use gtk::glib;

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/com/github/nozwock/PokeBook/ui/content-pages/pokemon.ui")]
    pub struct PokemonPageContent {
        #[template_child]
        pub main_sprite: TemplateChild<gtk::Image>,
        #[template_child]
        pub name: TemplateChild<gtk::Label>,
        #[template_child]
        pub base_exp: TemplateChild<gtk::Label>,
        #[template_child]
        pub height: TemplateChild<gtk::Label>,
        #[template_child]
        pub weight: TemplateChild<gtk::Label>,
        #[template_child]
        pub abilities_list: TemplateChild<gtk::FlowBox>,
        #[template_child]
        pub moves_list: TemplateChild<gtk::FlowBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PokemonPageContent {
        const NAME: &'static str = "PokemonPageContent";
        type ParentType = gtk::Box;
        type Type = super::PokemonPageContent;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    // #[glib::derived_properties]
    impl ObjectImpl for PokemonPageContent {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }
    impl WidgetImpl for PokemonPageContent {}
    impl BoxImpl for PokemonPageContent {}
}

glib::wrapper! {
    pub struct PokemonPageContent(ObjectSubclass<imp::PokemonPageContent>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl PokemonPageContent {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }
}

impl Default for PokemonPageContent {
    fn default() -> Self {
        glib::Object::builder().build()
    }
}
