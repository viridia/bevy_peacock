use bevy::{prelude::*, ui, utils::HashMap};
use bevy_peacock_style::TransitionState;

use crate::TransitionProperty;

#[derive(Component)]
#[doc(hidden)]
pub struct AnimatedTransform {
    pub(crate) state: TransitionState,
    pub(crate) origin: Transform,
    pub(crate) target: Transform,
}

#[derive(Component)]
#[doc(hidden)]
pub struct AnimatedBackgroundColor {
    pub(crate) state: TransitionState,
    pub(crate) origin: Color,
    pub(crate) target: Color,
}

#[derive(Component)]
#[doc(hidden)]
pub struct AnimatedBorderColor {
    pub(crate) state: TransitionState,
    pub(crate) origin: Color,
    pub(crate) target: Color,
}

pub struct AnimatedLayoutProp {
    pub(crate) state: TransitionState,
    pub(crate) origin: f32,
    pub(crate) target: f32,
}

impl AnimatedLayoutProp {
    pub fn new(state: TransitionState) -> Self {
        Self {
            state,
            origin: 0.,
            target: 0.,
        }
    }

    /// Update the [`Style`] component with the current animation value.
    pub fn update(&mut self, prop: TransitionProperty, style: &mut Style, delta: f32, force: bool) {
        let t_old = self.state.clock;
        self.state.advance(delta);
        let t = self.state.transition.timing.eval(self.state.clock);
        if t != t_old || force {
            let value = self.target * t + self.origin * (1. - t);
            match prop {
                TransitionProperty::Width => style.width = ui::Val::Px(value),
                TransitionProperty::Height => style.height = ui::Val::Px(value),
                TransitionProperty::Left => style.left = ui::Val::Px(value),
                TransitionProperty::Top => style.top = ui::Val::Px(value),
                TransitionProperty::Bottom => style.bottom = ui::Val::Px(value),
                TransitionProperty::Right => style.right = ui::Val::Px(value),
                TransitionProperty::BorderLeft => style.border.left = ui::Val::Px(value),
                TransitionProperty::BorderTop => style.border.top = ui::Val::Px(value),
                TransitionProperty::BorderRight => style.border.right = ui::Val::Px(value),
                TransitionProperty::BorderBottom => style.border.bottom = ui::Val::Px(value),
                TransitionProperty::Transform
                | TransitionProperty::BackgroundColor
                | TransitionProperty::BorderColor => panic!("Invalid style transition prop"),
            }
        }
    }

    /// Restart the animation with a new target if the target changed.
    pub fn restart_if_changed(
        &mut self,
        prop: TransitionProperty,
        prev_style: &Style, // The current style values
        next_style: &Style, // The targets we are going for
    ) {
        let (next, prev) = match prop {
            TransitionProperty::Width => (next_style.width, prev_style.width),
            TransitionProperty::Height => (next_style.height, prev_style.height),
            TransitionProperty::Left => (next_style.left, prev_style.left),
            TransitionProperty::Top => (next_style.top, prev_style.top),
            TransitionProperty::Bottom => (next_style.bottom, prev_style.bottom),
            TransitionProperty::Right => (next_style.right, prev_style.right),
            TransitionProperty::BorderLeft => (next_style.border.left, prev_style.border.left),
            TransitionProperty::BorderTop => (next_style.border.top, prev_style.border.top),
            TransitionProperty::BorderRight => (next_style.border.right, prev_style.border.right),
            TransitionProperty::BorderBottom => {
                (next_style.border.bottom, prev_style.border.bottom)
            }
            TransitionProperty::Transform
            | TransitionProperty::BackgroundColor
            | TransitionProperty::BorderColor => panic!("Invalid style transition prop"),
        };

        // Assume that all values are in pixels, we don't try and animate in other units.
        if let (ui::Val::Px(next_value), ui::Val::Px(prev_value)) = (next, prev) {
            if self.target != next_value {
                self.origin = prev_value;
                self.target = next_value;
                self.state.clock = 0.;
            }
        }
    }
}

#[derive(Component)]
#[doc(hidden)]
pub struct AnimatedLayout(pub HashMap<TransitionProperty, AnimatedLayoutProp>);

#[doc(hidden)]
pub fn animate_transforms(
    mut query: Query<(&mut Transform, &mut AnimatedTransform)>,
    time: Res<Time>,
) {
    for (mut trans, mut at) in query.iter_mut() {
        let t_old = at.state.clock;
        at.state.advance(time.delta_seconds());
        let t = at.state.transition.timing.eval(at.state.clock);
        if t != t_old {
            trans.scale = at.origin.scale.lerp(at.target.scale, t);
            trans.translation = at.origin.translation.lerp(at.target.translation, t);
            trans.rotation = at.origin.rotation.lerp(at.target.rotation, t);
        }
    }
}

#[doc(hidden)]
pub fn animate_bg_colors(
    mut query: Query<(
        Entity,
        Option<&mut BackgroundColor>,
        &mut AnimatedBackgroundColor,
    )>,
    time: Res<Time>,
) {
    #![allow(unused)]
    for (e, mut bg, mut at) in query.iter_mut() {
        let t_old = at.state.clock;
        at.state.advance(time.delta_seconds());
        let t = at.state.transition.timing.eval(at.state.clock);
        let origin = at.origin.as_rgba_linear();
        let target = at.target.as_rgba_linear();
        todo!("Finish color space interpolation!");
    }
}

#[doc(hidden)]
pub fn animate_border_colors(
    mut query: Query<(Entity, Option<&mut BorderColor>, &mut AnimatedBorderColor)>,
    time: Res<Time>,
) {
    #![allow(unused)]
    for (e, mut bg, mut at) in query.iter_mut() {
        let t_old = at.state.clock;
        at.state.advance(time.delta_seconds());
        let t = at.state.transition.timing.eval(at.state.clock);
        let origin = at.origin.as_rgba_linear();
        let target = at.target.as_rgba_linear();
        todo!("Finish color space interpolation!");
    }
}

#[doc(hidden)]
pub fn animate_layout(mut query: Query<(&mut Style, &mut AnimatedLayout)>, time: Res<Time>) {
    let delta = time.delta_seconds();
    for (mut style, mut anim) in query.iter_mut() {
        for (prop, trans) in anim.0.iter_mut() {
            trans.update(*prop, &mut style, delta, false);
        }
    }
}
