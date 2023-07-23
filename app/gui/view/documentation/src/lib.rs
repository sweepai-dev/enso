//! Documentation view presenting the documentation in the Component Browser.

#![recursion_limit = "1024"]
// === Features ===
#![feature(drain_filter)]
#![feature(option_result_contains)]
// === Standard Linter Configuration ===
#![deny(non_ascii_idents)]
#![warn(unsafe_code)]
#![allow(clippy::bool_to_int_with_if)]
#![allow(clippy::let_and_return)]
// === Non-Standard Linter Configuration ===
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused_import_braces)]
#![warn(unused_qualifications)]

use ensogl::display::shape::*;
use ensogl::prelude::*;
use ensogl::system::web::traits::*;

use enso_frp as frp;
use enso_suggestion_database::documentation_ir::EntryDocumentation;
use enso_suggestion_database::documentation_ir::LinkedDocPage;
use ensogl::application::Application;
use ensogl::data::color;
use ensogl::display;
use ensogl::display::DomSymbol;
use ensogl::system::web;
use ensogl::Animation;
use ensogl_derive_theme::FromTheme;
use ensogl_hardcoded_theme::application::component_browser::documentation as theme;
use graph_editor::component::visualization;
use ide_view_graph_editor as graph_editor;


// ==============
// === Export ===
// ==============

pub mod html;

pub use ide_view_component_list_panel_breadcrumbs as breadcrumbs;



// =================
// === Constants ===
// =================



const INITIAL_SECTION_NAME: &str = "Popular";
/// Delay before updating the displayed documentation.
const DISPLAY_DELAY_MS: i32 = 0;


// === Style ===

#[derive(Debug, Clone, Copy, Default, FromTheme)]
#[base_path = "theme"]
#[allow(missing_docs)]
pub struct Style {
    width:              f32,
    height:             f32,
    background:         color::Rgba,
    corner_radius:      f32,
    #[theme_path = "theme::breadcrumbs::height"]
    breadcrumbs_height: f32,
}


// =============
// === Model ===
// =============

/// Model of Native visualization that generates documentation for given Enso code and embeds
/// it in a HTML container.
#[derive(Clone, CloneRef, Debug)]
#[allow(missing_docs)]
pub struct Model {
    style_container: DomSymbol,
    dom:             DomSymbol,
    pub breadcrumbs: breadcrumbs::Breadcrumbs,
    /// The purpose of this overlay is stop propagating mouse events under the documentation panel
    /// to EnsoGL shapes, and pass them to the DOM instead.
    overlay:         Rectangle,
    background:      Rectangle,
    display_object:  display::object::Instance,
    event_handlers:  Rc<RefCell<Vec<web::EventListenerHandle>>>,
}

impl Model {
    /// Constructor.
    fn new(app: &Application) -> Self {
        let scene = &app.display.default_scene;
        let display_object = display::object::Instance::new();
        let style_div = web::document.create_div_or_panic();
        let style_container = DomSymbol::new(&style_div);
        let div = web::document.create_div_or_panic();
        let dom = DomSymbol::new(&div);
        let background = Rectangle::new();
        let overlay = Rectangle::new().build(|r| {
            r.set_color(INVISIBLE_HOVER_COLOR);
        });

        let breadcrumbs = app.new_view::<breadcrumbs::Breadcrumbs>();
        breadcrumbs.set_base_layer(&app.display.default_scene.layers.node_searcher);
        display_object.add_child(&breadcrumbs);

        dom.dom().set_attribute_or_warn("class", "scrollable");
        dom.dom().set_style_or_warn("white-space", "normal");
        dom.dom().set_style_or_warn("overflow-y", "auto");
        dom.dom().set_style_or_warn("overflow-x", "auto");
        dom.dom().set_style_or_warn("pointer-events", "auto");

        display_object.add_child(&background);
        display_object.add_child(&dom);
        display_object.add_child(&overlay);
        scene.dom.layers.node_searcher.manage(&style_container);
        scene.dom.layers.node_searcher.manage(&dom);

        Model {
            style_container,
            dom,
            breadcrumbs,
            overlay,
            background,
            display_object,
            event_handlers: default(),
        }
        .init()
    }

    fn init(self) -> Self {
        self.load_css_stylesheet();
        self
    }

    /// Add `<style>` tag with the stylesheet to the `style_container`.
    fn load_css_stylesheet(&self) {
        let stylesheet = include_str!("../assets/styles.css");
        let element = web::document.create_element_or_panic("style");
        element.set_inner_html(stylesheet);
        self.style_container.append_or_warn(&element);
    }

    fn set_initial_breadcrumbs(&self) {
        let breadcrumb = breadcrumbs::Breadcrumb::new(INITIAL_SECTION_NAME);
        self.breadcrumbs.set_entries_from((vec![breadcrumb], 0));
        self.breadcrumbs.show_ellipsis(true);
    }

    /// Display the documentation and scroll to the qualified name if needed.
    fn display_doc(&self, docs: EntryDocumentation, display_doc: &frp::Source<EntryDocumentation>) {
        let linked_pages = docs.linked_doc_pages();
        let html = html::render(&docs);
        self.dom.dom().set_inner_html(&html);
        self.set_link_handlers(linked_pages, display_doc);
        // Scroll to the top of the page.
        self.dom.dom().set_scroll_top(0);
    }

    /// Setup event handlers for links on the documentation page.
    fn set_link_handlers(
        &self,
        linked_pages: Vec<LinkedDocPage>,
        display_doc: &frp::Source<EntryDocumentation>,
    ) {
        let new_handlers = linked_pages.into_iter().filter_map(|page| {
            let content = page.page.clone_ref();
            let anchor = html::anchor_name(&page.name);
            if let Some(element) = web::document.get_element_by_id(&anchor) {
                let closure: web::JsEventHandler = web::Closure::new(f_!([display_doc, content] {
                    display_doc.emit(content.clone_ref());
                }));
                Some(web::add_event_listener(&element, "click", closure))
            } else {
                None
            }
        });
        let _ = self.event_handlers.replace(new_handlers.collect());
    }

    /// Load an HTML file into the documentation view when user is waiting for data to be received.
    /// TODO(#5214): This should be replaced with a EnsoGL spinner.
    fn load_waiting_screen(&self) {
        let spinner = include_str!("../assets/spinner.html");
        self.dom.dom().set_inner_html(spinner)
    }

    fn update_layout(&self, style: Style, width_factor: f32) {
        let size = Vector2(style.width * width_factor, style.height);
        self.overlay.set_size(size);
        let dom_size = Vector2(size.x, size.y - style.breadcrumbs_height);
        self.dom.set_dom_size(dom_size);
        self.dom.set_xy(dom_size / 2.0);

        self.background.set_color(style.background);
        self.background.set_corner_radius(style.corner_radius);
        self.background.set_size(size);

        self.overlay.set_corner_radius(style.corner_radius);
        self.dom.set_style_or_warn("border-radius", format!("{}px", style.corner_radius));

        self.breadcrumbs.set_xy(Vector2(0.0, size.y));
        self.breadcrumbs.frp().set_size(Vector2(size.x, style.breadcrumbs_height));
    }
}



// ===========
// === FRP ===
// ===========

ensogl::define_endpoints! {
    Input {
        /// Display documentation of the specific entry from the suggestion database.
        display_documentation (EntryDocumentation),
        /// Open documentation panel with animation.
        show(),
        /// Close documentation panel with animation.
        hide(),
        /// Skip show/hide animation.
        skip_animation(),
    }
    Output {
        /// Indicates whether the documentation panel has been selected through clicking into
        /// it, or deselected by clicking somewhere else.
        is_selected(bool),
        /// Indicates whether the documentation panel has been hovered.
        is_hovered(bool),
    }
}


// ============
// === View ===
// ============

/// View of the visualization that renders the given documentation as a HTML page.
///
/// The documentation can be provided in two formats: it can be code of the entity (type, method,
/// function etc) with doc comments, or the docstring only - in the latter case
/// however we're unable to summarize methods and atoms of types.
///
/// The default format is the docstring.
#[derive(Clone, CloneRef, Debug, Deref)]
#[allow(missing_docs)]
pub struct View {
    #[deref]
    pub model:             Model,
    pub visualization_frp: visualization::instance::Frp,
    pub frp:               Frp,
}

impl View {
    /// Constructor.
    pub fn new(app: &Application) -> Self {
        let frp = Frp::new();
        let visualization_frp = visualization::instance::Frp::new(&frp.network);
        let model = Model::new(app);
        model.load_waiting_screen();
        Self { model, visualization_frp, frp }.init(app)
    }

    fn init(self, app: &Application) -> Self {
        let network = &self.frp.network;
        let model = &self.model;
        let scene = &app.display.default_scene;
        let overlay = &model.overlay;
        let visualization = &self.visualization_frp;
        let frp = &self.frp;
        let display_delay = frp::io::timer::Timeout::new(network);
        let style_frp = StyleWatchFrp::new(&scene.style_sheet);
        let style = Style::from_theme(network, &style_frp);
        let width_anim = Animation::new(network);
        frp::extend! { network
            init <- source_();


            // === Breadcrumbs ===

            eval_ init(model.set_initial_breadcrumbs());


            // === Displaying documentation ===

            docs <- any(...);
            docs <+ frp.display_documentation;
            display_delay.restart <+ frp.display_documentation.constant(DISPLAY_DELAY_MS);
            display_docs <- display_delay.on_expired.map2(&docs,|_,docs| docs.clone_ref());
            display_docs_callback <- source();
            display_docs <- any(&display_docs, &display_docs_callback);
            eval display_docs([model, display_docs_callback]
                (docs) model.display_doc(docs.clone_ref(), &display_docs_callback)
            );


            // === Show/hide animation ===

            width_anim.target <+ frp.show.constant(1.0);
            width_anim.target <+ frp.hide.constant(0.0);
            width_anim.skip <+ frp.skip_animation;
            width_anim.target <+ init.constant(1.0);


            // === Layout update ===

            style_and_width_factor <- all(style.update, width_anim.target);
            eval style_and_width_factor(((style, width_factor)) model.update_layout(*style, *width_factor));


            // === Activation ===

            mouse_down_target <- scene.mouse.frp_deprecated.down.map(f_!(scene.mouse.target.get()));
            selected <- mouse_down_target.map(f!([model,visualization] (target){
                if !model.overlay.is_this_target(*target) {
                    visualization.deactivate.emit(());
                    false
                } else {
                    visualization.activate.emit(());
                    true
                }
            }));
            is_selected_changed <= selected.map2(&frp.output.is_selected, |&new,&old| {
                (new != old).as_some(new)
            });
            frp.source.is_selected <+ is_selected_changed;


            // === Mouse Cursor ===

            app.frp.show_system_cursor <+ overlay.events_deprecated.mouse_over;
            app.frp.hide_system_cursor <+ overlay.events_deprecated.mouse_out;


            // === Hover ===

            frp.source.is_hovered <+ model.overlay.events_deprecated.mouse_over.constant(true);
            frp.source.is_hovered <+ model.overlay.events_deprecated.mouse_out.constant(false);
        }
        init.emit(());
        style.init.emit(());
        self
    }
}

impl display::Object for View {
    fn display_object(&self) -> &display::object::Instance {
        &self.model.display_object
    }
}
