//! Model for the slider component.

use ensogl_core::display::shape::*;
use ensogl_core::dom::traits::*;
use ensogl_core::prelude::*;

use crate::LabelPosition;

use ensogl_core::application::Application;
use ensogl_core::data::color;
use ensogl_core::display;
use ensogl_core::dom;
use ensogl_core::Animation;
use ensogl_hardcoded_theme::component::slider as theme;
use ensogl_text as text;
use ensogl_text::formatting::ResolvedProperty;
use ensogl_tooltip::Tooltip;



// =================
// === Constants ===
// =================

/// Default component width on initialization.
const COMPONENT_WIDTH_DEFAULT: f32 = 200.0;
/// Default component height on initialization.
const COMPONENT_HEIGHT_DEFAULT: f32 = 50.0;
/// Overflow marker size as fraction of the text height.
const OVERFLOW_MARKER_SIZE: f32 = 0.75;



// =====================================================
// === Slider background and track shape definitions ===
// =====================================================

/// Triangle shape used as an overflow indicator on either side of the range.
mod overflow {
    use super::*;

    ensogl_core::shape! {
        // above = [background, track];
        pointer_events = false;
        alignment = center;
        (style:Style, color:Vector4) {
            let width: Var<Pixels> = "input_size.x".into();
            let height: Var<Pixels> = "input_size.y".into();

            let color = style.get_color(theme::overflow::color);
            let triangle = Triangle(width, height);
            let triangle = triangle.fill(color);

            triangle.into()
        }
    }
}



// ===============================
// === Slider model definition ===
// ===============================

/// The slider model contains the visual elements of the slider component.
#[derive(Debug)]
pub struct Model {
    /// Slider track element that fills the slider proportional to the slider value.
    pub track:                 dom::Div,
    /// Indicator for overflow when the value is below the lower limit.
    pub overflow_lower:        overflow::View,
    /// Indicator for overflow when the value is above the upper limit.
    pub overflow_upper:        overflow::View,
    /// Slider label that is shown next to the slider.
    pub label:                 text::Text,
    /// Textual representation of the slider value, only part left of the decimal point.
    pub value_text_left:       text::Text,
    /// Decimal point that is used to display non-integer slider values.
    pub value_text_dot:        text::Text,
    /// Textual representation of the slider value, only part right of the decimal point.
    pub value_text_right:      text::Text,
    /// Textual representation of the slider value used when editing the value as text input.
    pub value_text_edit:       text::Text,
    /// Tooltip component showing either a tooltip message or slider precision changes.
    pub tooltip:               Tooltip,
    /// Animation component that smoothly adjusts the slider start value on large jumps.
    pub start_value_animation: Animation<f32>,
    /// Animation component that smoothly adjusts the slider end value on large jumps.
    pub end_value_animation:   Animation<f32>,
    /// Root of the display object.
    pub root:                  dom::Div,
    /// The display object containing the text value of the slider.
    pub value:                 display::object::Instance,
}

impl Model {
    /// Create a new slider model.
    pub fn new(app: &Application, frp_network: &frp::Network) -> Self {
        let root = dom::Div::new();
        let value = display::object::Instance::new();
        let label = app.new_view::<text::Text>();
        let value_text_left = app.new_view::<text::Text>();
        let value_text_dot = app.new_view::<text::Text>();
        let value_text_right = app.new_view::<text::Text>();
        let value_text_edit = app.new_view::<text::Text>();
        let tooltip = Tooltip::new(app);
        let start_value_animation = Animation::new_non_init(frp_network);
        let end_value_animation = Animation::new_non_init(frp_network);
        let track = dom::Div::new();
        let overflow_lower = overflow::View::new();
        let overflow_upper = overflow::View::new();
        let style = StyleWatch::new(&app.display.default_scene.style_sheet);

        root.set_position_relative();
        root.set_border_radius(i32::MAX);
        root.set_overflow_clip();
        track.set_height(100.pc());

        root.append_child(&track);
        // FIXME: commented
        // root.add_child(&label);
        // root.add_child(&value);
        value.add_child(&value_text_left);
        value.add_child(&value_text_dot);
        value.add_child(&value_text_right);
        app.display.default_scene.add_child(&tooltip);

        let model = Self {
            track,
            overflow_lower,
            overflow_upper,
            label,
            value_text_left,
            value_text_dot,
            value_text_right,
            value_text_edit,
            tooltip,
            start_value_animation,
            end_value_animation,
            root,
            value,
        };
        model.init(style)
    }

    /// Initialise slider model.
    pub fn init(self, style: StyleWatch) -> Self {
        let background_color = style.get_color(theme::background::color);
        let track_color = style.get_color(theme::track::color);
        self.value_text_left.set_font(text::font::DEFAULT_FONT);
        self.value_text_dot.set_font(text::font::DEFAULT_FONT);
        self.value_text_right.set_font(text::font::DEFAULT_FONT);
        self.value_text_edit.set_font(text::font::DEFAULT_FONT);
        self.label.set_font(text::font::DEFAULT_FONT);
        self.root.set_background(background_color);
        self.track.set_background(track_color);

        self.update_size(Vector2(COMPONENT_WIDTH_DEFAULT, COMPONENT_HEIGHT_DEFAULT));
        self.value_text_dot.set_content(".");
        self
    }

    /// Set the component size.
    pub fn update_size(&self, size: Vector2<f32>) {
        // FIXME: commented
        // self.background.set_size(size);
        // self.background.set_width(size.x as f64);

        // FIXME: commented
        // self.track.set_size(size);

        // FIXME: commented
        // self.background.set_x(size.x / 2.0);
        // self.track.set_x(size.x / 2.0);
        self.value.set_x(size.x / 2.0);
    }

    /// Set the color of the slider track or thumb.
    pub fn set_indicator_color(&self, color: &color::Lcha) {
        // FIXME: commented
        // self.track.color.set(color::Rgba::from(color).into());
    }

    /// Set the color of the slider background.
    pub fn set_background_color(&self, color: &color::Lcha) {
        // FIXME: commented
        // self.background.color.set(color::Rgba::from(color).into());
    }

    /// Set the position of the value indicator.
    pub fn set_indicator_position(&self, start: f32, end: f32, orientation: Axis2) {
        let start = start.clamp(0.0, 1.0);
        let end = end.clamp(0.0, 1.0);
        self.track.set_width(((end - start) * 100.0).pc());
        self.track.set_margin_left((start * 100.0).pc());
    }

    /// Set the size and orientation of the overflow markers.
    pub fn set_overflow_marker_shape(&self, (size, orientation): &(f32, Axis2)) {
        let size = Vector2(*size, *size) * OVERFLOW_MARKER_SIZE;
        self.overflow_lower.set_size(size);
        self.overflow_upper.set_size(size);
        match orientation {
            Axis2::X => {
                self.overflow_lower.set_rotation_z(std::f32::consts::FRAC_PI_2);
                self.overflow_upper.set_rotation_z(-std::f32::consts::FRAC_PI_2);
            }
            Axis2::Y => {
                self.overflow_lower.set_rotation_z(std::f32::consts::PI);
                self.overflow_upper.set_rotation_z(0.0);
            }
        }
    }

    /// Set the position of the overflow markers.
    pub fn set_overflow_marker_position(
        &self,
        (comp_width, comp_height, orientation): &(f32, f32, Axis2),
    ) {
        match orientation {
            Axis2::X => {
                let pos_x = comp_width / 2.0 - comp_height / 4.0;
                self.overflow_lower.set_x(-pos_x);
                self.overflow_lower.set_y(0.0);
                self.overflow_upper.set_x(pos_x);
                self.overflow_upper.set_y(0.0);
            }
            Axis2::Y => {
                let pos_y = comp_height / 2.0 - comp_width / 4.0;
                self.overflow_lower.set_x(0.0);
                self.overflow_lower.set_y(-pos_y);
                self.overflow_upper.set_x(0.0);
                self.overflow_upper.set_y(pos_y);
            }
        }
    }

    /// Set whether the lower overfow marker is visible.
    pub fn set_overflow_lower_visible(&self, visible: bool) {
        // FIXME: commented
        // if visible {
        //     self.root.add_child(&self.overflow_lower);
        // } else {
        //     self.root.remove_child(&self.overflow_lower);
        // }
    }

    /// Set whether the upper overfow marker is visible.
    pub fn set_overflow_upper_visible(&self, visible: bool) {
        // FIXME: commented
        // if visible {
        //     self.root.add_child(&self.overflow_upper);
        // } else {
        //     self.root.remove_child(&self.overflow_upper);
        // }
    }

    /// Set the position of the slider's label.
    pub fn set_label_position(
        &self,
        (comp_width, comp_height, lab_width, lab_height, position, orientation): &(
            f32,
            f32,
            f32,
            f32,
            LabelPosition,
            Axis2,
        ),
    ) {
        let label_position_x = match orientation {
            Axis2::X => match position {
                LabelPosition::Inside => -comp_width / 2.0 + comp_height / 2.0,
                LabelPosition::Outside => -comp_width / 2.0 - comp_height / 2.0 - lab_width,
            },
            Axis2::Y => -lab_width / 2.0,
        };
        let label_position_y = match orientation {
            Axis2::X => lab_height / 2.0,
            Axis2::Y => match position {
                LabelPosition::Inside => comp_height / 2.0 - comp_width / 2.0,
                LabelPosition::Outside => comp_height / 2.0 + comp_width / 2.0 + lab_height,
            },
        };
        self.label.set_xy(Vector2(label_position_x, label_position_y));
    }

    /// Set whether the slider value text is hidden.
    pub fn show_value(&self, visible: bool) {
        // FIXME: commented
        // if visible {
        //     self.root.add_child(&self.value);
        // } else {
        //     self.root.remove_child(&self.value);
        // }
    }

    /// Set whether the slider label is hidden.
    pub fn set_label_hidden(&self, hidden: bool) {
        // FIXME: commented
        // if hidden {
        //     self.root.remove_child(&self.label);
        // } else {
        //     self.root.add_child(&self.label);
        // }
    }

    /// Set whether the value is being edited. This hides the value display and shows a text editor
    /// field to enter a new value.
    pub fn set_edit_mode(&self, (editing, _precision): &(bool, f32)) {
        // FIXME: commented
        // if *editing {
        //     self.root.remove_child(&self.value);
        //     self.root.add_child(&self.value_text_edit);
        //     self.value_text_edit.deprecated_focus();
        //     self.value_text_edit.add_cursor_at_front();
        //     self.value_text_edit.cursor_select_to_text_end();
        // } else {
        //     self.root.add_child(&self.value);
        //     // if *precision < 1.0 {
        //     //     self.root.add_child(&self.value_text_dot);
        //     //     self.root.add_child(&self.value_text_right);
        //     // }
        //     self.root.remove_child(&self.value_text_edit);
        //     self.value_text_edit.deprecated_defocus();
        //     self.value_text_edit.remove_all_cursors();
        // }
    }

    /// Set whether the value display decimal point and the text right of it are visible.
    pub fn set_value_text_right_visible(&self, enabled: bool) {
        if enabled {
            self.value.add_child(&self.value_text_dot);
            self.value.add_child(&self.value_text_right);
        } else {
            self.value.remove_child(&self.value_text_dot);
            self.value.remove_child(&self.value_text_right);
        }
    }

    /// Set default properties to the group of text elements displaying the slider value.
    pub fn set_value_text_property(&self, property: impl Into<ResolvedProperty> + Copy) {
        self.value_text_left.set_property_default(property.into());
        self.value_text_dot.set_property_default(property.into());
        self.value_text_right.set_property_default(property.into());
    }
}

// impl display::Object for Model {
//     fn display_object(&self) -> &display::object::Instance {
//         &self.root
//     }
// }
