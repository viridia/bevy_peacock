use bevy::{
    a11y::Focus,
    prelude::*,
    render::texture::{ImageLoaderSettings, ImageSampler},
};

use crate::{
    ElementStyles, SelectorMatcher, {ComputedStyle, UpdateComputedStyle},
};

use super::style_handle::TextStyles;

#[derive(Resource, Default)]
pub(crate) struct PreviousFocus(pub(crate) Option<Entity>);

#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub(crate) fn update_styles(
    mut commands: Commands,
    query_root: Query<Entity, (With<Node>, Without<Parent>)>,
    query_styles: Query<
        (
            Ref<Style>,
            Option<Ref<ElementStyles>>,
            Option<&TextStyles>,
            Option<Ref<Text>>,
        ),
        With<Node>,
    >,
    matcher: SelectorMatcher<'_, '_>,
    query_children: Query<&'static Children, (With<Node>, With<Visibility>)>,
    assets: Res<AssetServer>,
) {
    for root_node in &query_root {
        update_element_styles(
            &mut commands,
            &query_styles,
            &query_children,
            &matcher,
            &assets,
            root_node,
            &TextStyles::default(),
            false,
        )
    }
}

pub(crate) fn update_focus(focus: Res<Focus>, mut focus_prev: ResMut<PreviousFocus>) {
    focus_prev.0 = focus.0;
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
fn update_element_styles(
    commands: &mut Commands,
    query_styles: &Query<
        (
            Ref<Style>,
            Option<Ref<ElementStyles>>,
            Option<&TextStyles>,
            Option<Ref<Text>>,
        ),
        With<Node>,
    >,
    children_query: &Query<'_, '_, &Children, (With<Node>, With<Visibility>)>,
    matcher: &SelectorMatcher<'_, '_>,
    assets: &Res<AssetServer>,
    entity: Entity,
    inherited_styles: &TextStyles,
    mut inherited_styles_changed: bool,
) {
    let mut text_styles = inherited_styles.clone();

    if let Ok((style, elt_styles, prev_text_styles, txt)) = query_styles.get(entity) {
        // Check if the element styles or ancestor classes have changed.
        let mut changed = match elt_styles {
            Some(ref element_style) => matcher.is_changed(entity, element_style),
            None => false,
        };

        if let Some(ref text_node) = txt {
            if text_node.is_changed() {
                changed = true;
            }
        }

        if changed || inherited_styles_changed {
            // Compute computed style. Initialize to the current state.
            let mut computed = ComputedStyle::new();
            computed.style = style.clone();

            // Inherited properties
            computed.font_handle = inherited_styles.font.clone();
            computed.font_size = inherited_styles.font_size;
            computed.color = inherited_styles.color;

            // Apply element styles to computed
            if let Some(ref element_styles) = elt_styles {
                for ss in element_styles.styles.iter() {
                    ss.apply_to(&mut computed, matcher, &entity);
                }
                // Load font asset if non-null.
                if let Some(ref font_path) = computed.font {
                    computed.font_handle = Some(assets.load(font_path));
                }
            }

            // Update inherited text styles
            text_styles.font = computed.font_handle.clone();
            text_styles.font_size = computed.font_size;
            text_styles.color = computed.color;

            if text_styles == *inherited_styles && txt.is_none() {
                // No change from parent, so we can remove the cached styles and rely on inherited
                // styles only. Note that for text nodes, we always want to store the inherited
                // styles, even if they are the same as the parent.
                inherited_styles_changed = prev_text_styles.is_some();
                if inherited_styles_changed {
                    changed = true;
                    commands.entity(entity).remove::<TextStyles>();
                }
            } else {
                // Text styles are different from parent, so we need to store a cached copy.
                inherited_styles_changed = prev_text_styles != Some(&text_styles);
                if inherited_styles_changed {
                    changed = true;
                    commands.entity(entity).insert(text_styles.clone());
                }
            }

            if changed {
                computed.image_handle = computed.image.as_ref().map(|path| {
                    assets.load_with_settings(path, |s: &mut ImageLoaderSettings| {
                        s.sampler = ImageSampler::linear()
                    })
                });

                commands.add(UpdateComputedStyle { entity, computed });
            }
        } else if let Some(prev) = prev_text_styles {
            // Styles didn't change, but we need to pass inherited text styles to children.
            text_styles = prev.clone();
        }
    }

    if let Ok(children) = children_query.get(entity) {
        for child in children.iter() {
            update_element_styles(
                commands,
                query_styles,
                // classes_query,
                // parent_query,
                children_query,
                matcher,
                assets,
                *child,
                &text_styles,
                inherited_styles_changed,
            );
        }
    }
}
