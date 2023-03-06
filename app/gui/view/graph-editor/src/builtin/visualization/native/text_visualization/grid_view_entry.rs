//! This module contains the `Entry` used in the `TextGrid` visualizations well as associated
//! structs.

use crate::builtin::visualization::native::text_visualization::*;
use ensogl::prelude::*;

use crate::display;
use crate::web;
use crate::Application;

use ensogl::data::color;
use ensogl_component::grid_view;
use ensogl_component::grid_view::entry::EntryFrp;
use std::fmt::Write;



// =============
// === Model ===
// =============

/// Model that contains the data that is required to create and render an `Entry`.
#[derive(Clone, Debug, Default)]
pub struct Model {
    pub content:  Content,
    pub bg_color: color::Rgba,
}

impl From<String> for Model {
    fn from(content: String) -> Self {
        let content = content.into();
        let bg_color = color::Rgba::transparent();
        Self { content, bg_color }
    }
}

/// Content that is shown in an `Entry`.
#[derive(Clone, Debug)]
pub enum Content {
    Text { content: String },
    Divider { top: bool, bottom: bool, left: bool, right: bool },
}

impl Default for Content {
    fn default() -> Self {
        Self::Text { content: String::default() }
    }
}

impl From<String> for Content {
    fn from(content: String) -> Self {
        Self::Text { content }
    }
}

impl Model {
    /// Get the content of the model.
    pub fn text(&self, width: usize) -> String {
        match &self.content {
            Content::Text { content, .. } => content.to_string(),
            _ => " ".repeat(width),
        }
    }
}



// ==============
// === Params ===
// ==============

/// Parameters that are required to set up an `Entry`.
#[derive(Clone, Debug, Default)]
pub struct Params {
    /// DOM parent of the Entry. The text element in the `Entry` must be a child of the
    /// `parent` to appear correctly.
    pub parent:    Option<web::HtmlDivElement>,
    /// Name of the font to be used in the `Entry`.
    pub font_name: ImString,
    /// Font size in pixels.
    pub font_size: f32,
}



// =============
// === Entry ===
// =============


/// Entry for use in GridView. Contains a dom element with a text, the Entry frp, and a dummy
/// display object for compatibility with `GridView`. The `dummy_root` is not used for
/// displaying anything, all that is visible is the `text` element, which is updates through
/// the FRP.
#[derive(Clone, CloneRef, Debug)]
pub struct Entry {
    // Needed to provide a dummy display object for the `display::Object` trait. Not used, as the
    // text element is created as HTML Element and positioned manually in `set_position_and_size`.
    dummy_root: display::object::Instance,
    content:    Rc<web::HtmlDivElement>,
    frp:        Rc<EntryFrp<Self>>,
}

impl Entry {
    fn set_model(&self, model: &Model) {
        self.content.set_style_or_warn("background-color", model.bg_color.to_javascript_string());
        self.content.set_style_or_warn("border-width", "1px");
        self.content.set_style_or_warn("border-color", model.bg_color.to_javascript_string());

        self.content.set_inner_text(&model.text(CHARS_PER_CHUNK));
    }

    fn set_params(&self, params: &Params) {
        if let Some(parent) = &params.parent {
            parent.append_or_warn(&self.content);
        }
    }

    fn set_position_and_size(&self, pos: &Vector2, size: &Vector2) {
        let left = pos.x - size.x / 2.0;
        let top = -pos.y - size.y / 2.0;
        let width = size.x as u32 + 1; // Prevents aliasing artifacts between adjacent cells.
        let height = size.y as u32;
        let mut style = "position: absolute; white-space: pre; pointer-events: auto;".to_string();
        write!(style, "left: {left}px; top: {top}px;").ok();
        write!(style, "width: {width}px; height: {height}px;").ok();
        self.content.set_attribute_or_warn("style", style);

        // Properly hide zero sized elements.
        if size.x <= 0.0 {
            self.content.set_style_or_warn("display", "none");
        } else {
            self.content.set_style_or_warn("display", "block");
        }
    }
}

impl display::Object for Entry {
    fn display_object(&self) -> &display::object::Instance {
        &self.dummy_root
    }
}

impl grid_view::Entry for Entry {
    type Model = Model;
    type Params = Params;

    fn new(_app: &Application, _text_layer: Option<&display::scene::Layer>) -> Self {
        let text = web::document.create_div_or_panic();
        let dummy_root = display::object::Instance::new();
        let content = Rc::new(text);
        let new_entry = Self { dummy_root, content, frp: default() };

        let input = &new_entry.frp.private().input;
        let network = new_entry.frp.network();
        enso_frp::extend! { network
            init <- source_();
            eval input.set_model((model) new_entry.set_model(model));
            eval input.set_params((params) new_entry.set_params(params));

            pos_size <- all(&input.position_set, &input.set_size);
            eval pos_size (((pos, size)) new_entry.set_position_and_size(pos, size));
        }
        init.emit(());
        new_entry
    }

    fn frp(&self) -> &EntryFrp<Self> {
        &self.frp
    }
}
