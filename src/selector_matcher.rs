use bevy::a11y::Focus;
use bevy::ecs::entity::Entity;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
#[cfg(feature = "bevy_mod_picking")]
use bevy_mod_picking::{
    focus::{HoverMap, PreviousHoverMap},
    pointer::PointerId,
};
#[cfg(not(feature = "bevy_mod_picking"))]
use std::marker::PhantomData;

use crate::update::PreviousFocus;
use crate::{ElementClasses, ElementStyles, Selector};

#[derive(SystemParam)]
pub struct SelectorMatcher<'w, 's> {
    classes_query: Query<'w, 's, Ref<'static, ElementClasses>>,
    parent_query: Query<'w, 's, &'static Parent, (With<Node>, With<Visibility>)>,
    children_query: Query<'w, 's, &'static Children, (With<Node>, With<Visibility>)>,

    #[cfg(feature = "bevy_mod_picking")]
    hover_map: Res<'w, HoverMap>,
    #[cfg(feature = "bevy_mod_picking")]
    hover_map_prev: Res<'w, PreviousHoverMap>,

    #[cfg(not(feature = "bevy_mod_picking"))]
    hover_map: PhantomData<i32>,
    #[cfg(not(feature = "bevy_mod_picking"))]
    hover_map_prev: PhantomData<i32>,

    focus: Res<'w, Focus>,
    focus_prev: Res<'w, PreviousFocus>,
}

impl<'w, 's> SelectorMatcher<'w, 's> {
    /// Detects whether the given entity's styles have changed, or whether any of its ancestors
    /// have changed in a way that would affect the computation of styles (either because
    /// of class list changes or hovering).
    pub(crate) fn is_changed(
        &self,
        entity: Entity,
        element_styles: &Ref<'_, ElementStyles>,
    ) -> bool {
        // Style changes only affect current element, not children.
        let mut changed = element_styles.is_changed();

        // Search ancestors to see if any have changed.
        // We want to know if either the class list or the hover state has changed.
        if !changed && element_styles.selector_depth > 0 {
            let mut e = entity;
            for _ in 0..element_styles.selector_depth {
                if let Ok(a_classes) = self.classes_query.get(e) {
                    if element_styles.uses_hover && self.is_hovering(&e) != self.was_hovering(&e) {
                        changed = true;
                        break;
                    }

                    if self.is_focused(&e) != self.was_focused(&e) {
                        changed = true;
                        break;
                    }

                    if self.is_focus_visible(&e) != self.was_focus_visible(&e) {
                        changed = true;
                        break;
                    }

                    if element_styles.uses_focus_within
                        && self.is_focus_within(&e) != self.was_focus_within(&e)
                    {
                        changed = true;
                        break;
                    }

                    if a_classes.is_changed() {
                        changed = true;
                        break;
                    }
                }

                match self.parent_query.get(e) {
                    Ok(parent) => e = **parent,
                    _ => break,
                }
            }
        }
        changed
    }

    /// True if the given entity, or an ancestor of it, is in the hover map for PointerId::Mouse.
    ///
    /// This is used to determine whether to apply the :hover pseudo-class.
    #[cfg(feature = "bevy_mod_picking")]
    pub fn is_hovering(&self, e: &Entity) -> bool {
        match self.hover_map.get(&PointerId::Mouse) {
            Some(map) => map.iter().any(|(ha, _)| self.is_descendant(ha, e)),
            None => false,
        }
    }

    #[cfg(not(feature = "bevy_mod_picking"))]
    pub fn is_hovering(&self, _e: &Entity) -> bool {
        false
    }

    /// True if the given entity, or an ancestor of it, was previously in the hover map for
    /// PointerId::Mouse.
    #[cfg(feature = "bevy_mod_picking")]
    fn was_hovering(&self, e: &Entity) -> bool {
        match self.hover_map_prev.get(&PointerId::Mouse) {
            Some(map) => map.iter().any(|(ha, _)| self.is_descendant(ha, e)),
            None => false,
        }
    }

    #[cfg(not(feature = "bevy_mod_picking"))]
    fn was_hovering(&self, _e: &Entity) -> bool {
        false
    }

    /// True if the given entity has keyboard focus.
    ///
    /// This is used to determine whether to apply the :focus pseudo-class.
    pub fn is_focused(&self, e: &Entity) -> bool {
        Some(e) == self.focus.0.as_ref()
    }

    /// True if the given entity, or a descendant of it has keyboard focus.
    ///
    /// This is used to determine whether to apply the :focus-within pseudo-class.
    pub fn is_focus_within(&self, e: &Entity) -> bool {
        match self.focus.0 {
            Some(focus) => self.is_descendant(&focus, e),
            None => false,
        }
    }

    /// True if the given entity has focus and focus visibility is enabled.
    ///
    /// This is used to determine whether to apply the :focus-visible pseudo-class.
    pub fn is_focus_visible(&self, e: &Entity) -> bool {
        // TODO: Add configuration flag for whether focus should be visible.
        Some(e) == self.focus.0.as_ref()
    }

    /// True if the given entity had keyboard focus in the previous frame.
    fn was_focused(&self, e: &Entity) -> bool {
        Some(e) == self.focus_prev.0.as_ref()
    }

    /// True if the given entity, or a descendant of it had keyboard focus in the previous frame.
    fn was_focus_within(&self, e: &Entity) -> bool {
        match self.focus_prev.0 {
            Some(focus) => self.is_descendant(&focus, e),
            None => false,
        }
    }

    /// True if the given entity had focus and focus visibility is enabled.
    fn was_focus_visible(&self, e: &Entity) -> bool {
        // TODO: Add configuration flag for whether focus should be visible.
        Some(e) == self.focus_prev.0.as_ref()
    }

    /// True if this entity is the first child of its parent.
    pub fn is_first_child(&self, entity: &Entity) -> bool {
        match self.parent_query.get(*entity) {
            Ok(parent) => match self.children_query.get(parent.get()) {
                Ok(children) => children.first() == Some(entity),
                _ => false,
            },
            _ => false,
        }
    }

    /// True if this entity is the last child of its parent.
    pub fn is_last_child(&self, entity: &Entity) -> bool {
        match self.parent_query.get(*entity) {
            Ok(parent) => match self.children_query.get(parent.get()) {
                Ok(children) => children.last() == Some(entity),
                _ => false,
            },
            _ => false,
        }
    }

    /// Given an array of match params representing the element's ancestor chain, match the
    /// selector expression with the params.
    pub(crate) fn selector_match(&self, selector: &Selector, entity: &Entity) -> bool {
        match selector {
            Selector::Accept => true,
            Selector::Class(cls, next) => match self.classes_query.get(*entity) {
                Ok(classes) => classes.0.contains(cls) && self.selector_match(next, entity),
                _ => false,
            },
            Selector::Hover(next) => self.is_hovering(entity) && self.selector_match(next, entity),
            Selector::Focus(next) => self.is_focused(entity) && self.selector_match(next, entity),
            Selector::FocusWithin(next) => {
                self.is_focus_within(entity) && self.selector_match(next, entity)
            }
            Selector::FocusVisible(next) => {
                self.is_focus_visible(entity) && self.selector_match(next, entity)
            }
            Selector::FirstChild(next) => {
                self.is_first_child(entity) && self.selector_match(next, entity)
            }
            Selector::LastChild(next) => {
                self.is_last_child(entity) && self.selector_match(next, entity)
            }
            Selector::Current(next) => self.selector_match(next, entity),
            Selector::Parent(next) => match self.parent_query.get(*entity) {
                Ok(parent) => self.selector_match(next, &parent.get()),
                _ => false,
            },
            Selector::Either(opts) => opts.iter().any(|next| self.selector_match(next, entity)),
        }
    }

    /// True if the given entity is a descendant of the given ancestor.
    fn is_descendant(&self, e: &Entity, ancestor: &Entity) -> bool {
        let mut ha = e;
        loop {
            if ha == ancestor {
                return true;
            }
            match self.parent_query.get(*ha) {
                Ok(parent) => ha = parent,
                _ => return false,
            }
        }
    }
}

impl bevy_peacock_style::SelectorMatcher for SelectorMatcher<'_, '_> {
    fn selector_match(&self, selector: &Selector, entity: &Entity) -> bool {
        SelectorMatcher::selector_match(self, selector, entity)
    }
}
