use bevy::prelude::*;

use crate::{
    animate_bg_colors, animate_border_colors, animate_layout, animate_transforms,
    update::{update_focus, update_styles, PreviousFocus},
};

/// Plugin which initializes the Quill library.
pub struct PeacockPlugin;

impl Plugin for PeacockPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PreviousFocus>().add_systems(
            Update,
            (
                update_styles,
                (
                    update_focus,
                    animate_transforms,
                    animate_bg_colors,
                    animate_border_colors,
                    animate_layout,
                ),
            )
                .chain(),
        );
    }
}
