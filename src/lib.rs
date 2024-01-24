//! **Peacock** is a framework for CSS-like stylesheets that apply to Bevy UI nodes.

#![warn(missing_docs)]

mod animate;
mod classes;
mod plugin;
mod selector_matcher;
mod style_handle;
mod style_tuple;
pub(crate) mod update;
mod update_computed;

pub use animate::animate_bg_colors;
pub use animate::animate_border_colors;
pub use animate::animate_layout;
pub use animate::animate_transforms;
pub use bevy_peacock_style::timing;
pub use bevy_peacock_style::ComputedStyle;
pub use bevy_peacock_style::PointerEvents;
pub(crate) use bevy_peacock_style::Selector;
pub use bevy_peacock_style::StyleProp;
pub use bevy_peacock_style::StylePropList;
pub use bevy_peacock_style::Transition;
pub use bevy_peacock_style::TransitionProperty;
pub use classes::ClassNames;
pub use classes::ElementClasses;
pub use plugin::PeacockPlugin;
pub use plugin::PeacockSystemSet;
pub(crate) use selector_matcher::SelectorMatcher;
pub use style_handle::ElementStyles;
pub use style_handle::StyleHandle;
pub use style_handle::StyleSheet;
pub use style_tuple::StyleTuple;
pub use style_tuple::WithStyles;
pub use update_computed::UpdateComputedStyle;

pub use bevy_peacock_derive::import_stylesheet;
