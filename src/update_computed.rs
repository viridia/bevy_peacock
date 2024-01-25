use super::animate::{
    AnimatedBackgroundColor, AnimatedBorderColor, AnimatedLayout, AnimatedLayoutProp,
    AnimatedTransform,
};
use bevy::ecs::system::Command;
use bevy::prelude::*;
use bevy::ui::widget::UiImageSize;
use bevy::ui::ContentSize;
use bevy::utils::HashMap;
#[cfg(feature = "bevy_mod_picking")]
use bevy_mod_picking::prelude::Pickable;
use bevy_peacock_style::{ComputedStyle, PointerEvents};
use bevy_peacock_style::{TransitionProperty, TransitionState};

/// Custom command that updates the style of an entity.
pub struct UpdateComputedStyle {
    pub(crate) entity: Entity,
    pub(crate) computed: ComputedStyle,
}

impl Command for UpdateComputedStyle {
    fn apply(self, world: &mut World) {
        let Some(mut e) = world.get_entity_mut(self.entity) else {
            return;
        };

        let mut is_animated_bg_color = false;
        let mut is_animated_border_color = false;
        let mut is_animated_transform = false;
        let mut is_animated_layout = false;

        let mut next_style = self.computed.style;

        self.computed
            .transitions
            .iter()
            .for_each(|tr| match tr.property {
                TransitionProperty::Transform => is_animated_transform = true,
                TransitionProperty::BackgroundColor => is_animated_bg_color = true,
                TransitionProperty::BorderColor => is_animated_border_color = true,
                TransitionProperty::Height
                | TransitionProperty::Width
                | TransitionProperty::Left
                | TransitionProperty::Top
                | TransitionProperty::Bottom
                | TransitionProperty::Right
                | TransitionProperty::BorderLeft
                | TransitionProperty::BorderTop
                | TransitionProperty::BorderRight
                | TransitionProperty::BorderBottom => is_animated_layout = true,
            });

        let bg_image = self.computed.image_handle;

        // If any layout properties are animated, insert animation components and mutate
        // the style that's going to get inserted
        if is_animated_layout {
            // Get the current style
            let prev_style: Style = match e.get::<Style>() {
                Some(st) => st.clone(),
                None => next_style.clone(),
            };

            // TODO: Make sure the set of transitions hasn't changed.

            // If there's already animations
            if let Some(mut anim) = e.get_mut::<AnimatedLayout>() {
                for (prop, trans) in anim.0.iter_mut() {
                    trans.restart_if_changed(*prop, &prev_style, &next_style);
                    trans.update(*prop, &mut next_style, 0., true);
                }
            } else {
                let mut anim =
                    AnimatedLayout(HashMap::with_capacity(self.computed.transitions.len()));
                self.computed
                    .transitions
                    .iter()
                    .for_each(|tr| match tr.property {
                        TransitionProperty::Left
                        | TransitionProperty::Top
                        | TransitionProperty::Right
                        | TransitionProperty::Bottom
                        | TransitionProperty::Height
                        | TransitionProperty::Width
                        | TransitionProperty::BorderLeft
                        | TransitionProperty::BorderTop
                        | TransitionProperty::BorderRight
                        | TransitionProperty::BorderBottom => {
                            let mut ap = AnimatedLayoutProp::new(TransitionState {
                                transition: tr.clone(),
                                clock: 0.,
                            });
                            ap.update(tr.property, &mut next_style, 0., true);
                            anim.0.insert(tr.property, ap);
                        }
                        _ => (),
                    });
                e.insert(anim);
            }
        }

        if let Some(mut existing_style) = e.get_mut::<Style>() {
            // Update the existing style
            if !existing_style.eq(&next_style) {
                *existing_style = next_style;
            }
        } else {
            // Insert a new Style component
            e.insert(next_style);
        }

        if let Some(mut text) = e.get_mut::<Text>() {
            // White is the default.
            let color = self.computed.color.unwrap_or(Color::WHITE);
            for section in text.sections.iter_mut() {
                if section.style.color != color {
                    section.style.color = color;
                }
            }

            if let Some(ws) = self.computed.line_break {
                if text.linebreak_behavior != ws {
                    text.linebreak_behavior = ws;
                }
            }

            if let Some(font_size) = self.computed.font_size {
                for section in text.sections.iter_mut() {
                    if section.style.font_size != font_size {
                        section.style.font_size = font_size;
                    }
                }
            }

            if let Some(ref font) = self.computed.font_handle {
                for section in text.sections.iter_mut() {
                    if section.style.font != *font {
                        section.style.font = font.clone();
                    }
                }
            }
        }

        if is_animated_bg_color {
            match e.get_mut::<AnimatedBackgroundColor>() {
                Some(_) => todo!(),
                None => todo!(),
            }
        } else {
            e.remove::<AnimatedBackgroundColor>();
            match e.get_mut::<BackgroundColor>() {
                Some(mut bg_comp) => {
                    if self.computed.background_color.is_none() {
                        if bg_image.is_none() {
                            // Remove the background
                            e.remove::<BackgroundColor>();
                        }
                    } else {
                        let color = self.computed.background_color.unwrap();
                        // Mutate the background
                        if bg_comp.0 != color {
                            bg_comp.0 = color
                        }
                    }
                }

                None => {
                    if self.computed.background_color.is_some() {
                        // Insert a new background
                        e.insert(BackgroundColor(self.computed.background_color.unwrap()));
                    } else if bg_image.is_some() {
                        // Images require a background color to be set.
                        e.insert(BackgroundColor::DEFAULT);
                    }
                }
            }
        }

        if is_animated_border_color {
            match e.get_mut::<AnimatedBorderColor>() {
                Some(_) => todo!(),
                None => todo!(),
            }
        } else {
            e.remove::<AnimatedBorderColor>();
            match e.get_mut::<BorderColor>() {
                Some(mut bc_comp) => {
                    if self.computed.border_color.is_none() {
                        // Remove the border color
                        e.remove::<BorderColor>();
                    } else {
                        let color = self.computed.border_color.unwrap();
                        if bc_comp.0 != color {
                            bc_comp.0 = color
                        }
                    }
                }

                None => {
                    if self.computed.border_color.is_some() {
                        // Insert a new background color
                        e.insert(BorderColor(self.computed.border_color.unwrap()));
                    }
                }
            }
        }

        match e.get_mut::<UiImage>() {
            Some(mut img) => {
                match bg_image {
                    Some(src) => {
                        if img.texture != src {
                            img.texture = src;
                        }
                        if img.flip_x != self.computed.flip_x {
                            img.flip_x = self.computed.flip_x;
                        }
                        if img.flip_y != self.computed.flip_y {
                            img.flip_y = self.computed.flip_y;
                        }
                    }
                    None => {
                        // Remove the image.
                        e.remove::<UiImage>();
                    }
                }
            }

            None => {
                if let Some(src) = bg_image {
                    // Create image component
                    e.insert((
                        UiImage {
                            texture: src,
                            flip_x: self.computed.flip_x,
                            flip_y: self.computed.flip_y,
                        },
                        ContentSize::default(),
                        UiImageSize::default(),
                    ));
                }
            }
        }

        // Update outline
        match (self.computed.outline_color, e.get_mut::<Outline>()) {
            (Some(color), Some(mut outline)) => {
                outline.width = self.computed.outline_width;
                outline.offset = self.computed.outline_offset;
                outline.color = color;
            }
            (None, Some(_)) => {
                e.remove::<Outline>();
            }
            (Some(color), None) => {
                e.insert(Outline {
                    width: self.computed.outline_width,
                    offset: self.computed.outline_offset,
                    color,
                });
            }
            (None, None) => {}
        }

        // Update Z-Index
        match (self.computed.z_index, e.get::<ZIndex>()) {
            // Don't change if value is the same. Also, local(0) is the same as not at all.
            (ZIndex::Local(zi), Some(ZIndex::Local(zo))) if zi != 0 && zi == *zo => {}
            (ZIndex::Global(zi), Some(ZIndex::Global(zo))) if zi == *zo => {}
            (ZIndex::Local(zi), Some(_)) if zi != 0 => {
                e.insert(ZIndex::Local(zi));
            }
            (ZIndex::Global(zi), Some(_)) => {
                e.insert(ZIndex::Global(zi));
            }
            (_, Some(_)) => {
                e.remove::<ZIndex>();
            }
            (_, None) => {}
        }

        UpdateComputedStyle::update_picking(self.computed.pickable, &mut e);

        let mut transform = Transform::default();
        transform.translation = self.computed.translation.unwrap_or(transform.translation);
        transform.scale.x = self.computed.scale_x.unwrap_or(1.);
        transform.scale.y = self.computed.scale_y.unwrap_or(1.);
        transform.rotate_z(self.computed.rotation.unwrap_or(0.));
        if is_animated_transform {
            let prev_transform = *e.get_mut::<Transform>().unwrap();
            let transition = self
                .computed
                .transitions
                .iter()
                .find(|t| t.property == TransitionProperty::Transform)
                .unwrap();
            match e.get_mut::<AnimatedTransform>() {
                Some(at) => {
                    if at.target.translation != transform.translation
                        || at.target.scale != transform.scale
                        || at.target.rotation != transform.rotation
                    {
                        e.insert(AnimatedTransform {
                            state: TransitionState {
                                transition: transition.clone(),
                                clock: 0.,
                            },
                            origin: prev_transform,
                            target: transform,
                        });
                    }
                }
                None => {
                    e.insert(AnimatedTransform {
                        state: TransitionState {
                            transition: transition.clone(),
                            clock: 0.,
                        },
                        origin: transform,
                        target: transform,
                    });
                }
            }
        } else {
            match e.get_mut::<Transform>() {
                Some(tr) => {
                    if tr.translation != transform.translation
                        || tr.scale != transform.scale
                        || tr.rotation != transform.rotation
                    {
                        e.insert(transform);
                    }
                }
                None => {
                    panic!("Element has no transform!")
                }
            }
        }
    }
}

impl UpdateComputedStyle {
    #[cfg(feature = "bevy_mod_picking")]
    fn update_picking(pickable: Option<PointerEvents>, e: &mut EntityWorldMut<'_>) {
        // Update Pickable
        match (pickable, e.get_mut::<Pickable>()) {
            (Some(pe), Some(mut pickable)) => {
                pickable.should_block_lower = pe == PointerEvents::All;
                pickable.should_emit_events = pe == PointerEvents::All;
            }
            (None, Some(_)) => {
                e.remove::<Pickable>();
            }
            (Some(pe), None) => {
                e.insert(Pickable {
                    should_block_lower: pe == PointerEvents::All,
                    should_emit_events: pe == PointerEvents::All,
                });
            }
            (None, None) => {}
        }
    }

    #[cfg(not(feature = "bevy_mod_picking"))]
    fn update_picking(_pickable: Option<PointerEvents>, _e: &mut EntityWorldMut<'_>) {}
}
