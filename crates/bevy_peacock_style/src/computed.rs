use super::style::PointerEvents;
use super::transition::Transition;
use bevy::asset::AssetPath;
use bevy::prelude::*;
use bevy::text::BreakLineOn;
#[cfg(feature = "bevy_mod_picking")]
use bevy_mod_picking::prelude::Pickable;

/// A computed style represents the composition of one or more `ElementStyle`s.
#[derive(Default, Clone, Debug)]
#[doc(hidden)]
pub struct ComputedStyle {
    pub style: Style,

    // Text properties
    pub alignment: Option<TextAlignment>,
    pub color: Option<Color>,
    pub font_size: Option<f32>,
    pub font: Option<AssetPath<'static>>,
    pub font_handle: Option<Handle<Font>>,
    pub line_break: Option<BreakLineOn>,

    // pub text_style: TextStyle,
    pub border_color: Option<Color>,
    pub background_color: Option<Color>,
    pub outline_color: Option<Color>,
    pub outline_width: Val,
    pub outline_offset: Val,
    pub z_index: ZIndex,

    // Transform properties
    pub scale_x: Option<f32>,
    pub scale_y: Option<f32>,
    pub rotation: Option<f32>,
    pub translation: Option<Vec3>,

    // Image properties
    pub image: Option<AssetPath<'static>>,
    pub image_handle: Option<Handle<Image>>,
    pub flip_x: bool,
    pub flip_y: bool,

    // Picking properties
    pub pickable: Option<PointerEvents>,

    // Transitiions
    pub transitions: Vec<Transition>,
}

impl ComputedStyle {
    /// Construct a new, default style
    pub fn new() -> Self {
        Self { ..default() }
    }
}
