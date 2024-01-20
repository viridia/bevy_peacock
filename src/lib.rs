//! **Peacock** is a framework for CSS-like stylesheets that apply to Bevy UI nodes.

#![warn(missing_docs)]

mod builder;
mod classes;
mod computed;
mod cursor;
mod plugin;
mod selector;
mod selector_matcher;
mod style_handle;
mod style_props;
mod style_tuple;
mod transition;
pub(crate) mod update;

pub use classes::ClassNames;
pub use classes::ElementClasses;
pub use computed::ComputedStyle;
pub use computed::UpdateComputedStyle;
pub(crate) use selector::Selector;
pub(crate) use selector_matcher::SelectorMatcher;
pub use style_handle::ElementStyles;
pub use style_handle::StyleHandle;
pub use style_props::PointerEvents;
pub use style_props::StyleProp;
pub use style_tuple::StyleTuple;
pub use transition::animate_bg_colors;
pub use transition::animate_border_colors;
pub use transition::animate_layout;
pub use transition::animate_transforms;
pub use transition::timing;
pub use transition::Transition;
pub use transition::TransitionProperty;
