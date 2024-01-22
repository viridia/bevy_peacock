use bevy::ecs::entity::Entity;
use winnow::Parser;

use std::fmt;

use crate::selector_parser;

/// Represents a predicate which can be used to conditionally style a node.
/// Selectors support a subset of CSS grammar:
///
/// * Current element (`&`)
/// * Classname matching
/// * Psuedo-classes: `:hover`, `:focus`, `:focus-within`, `:focus-visible`, `:first-child`,
///   `:last-child`.
/// * Parent element (`>`) pattern
/// * Multiple patterns can be specified by commas.
///
/// Examples:
/// ```css
///   &
///   &.name
///   :hover
///   .state > &
///   .state > * > &.name
/// ```
///
/// Selectors must target the "current element": this means that the "`&`" selector is
/// required, and it can only appear on the last term of the selector expression. This means
/// that parent elements cannot implicitly style their children; child elements must have styles
/// explicitly specified (although those styles can be conditional on the state of their parents).
#[derive(Debug, PartialEq, Clone)]
pub enum Selector {
    /// If we reach this state, it means the match was successful
    Accept,

    /// Match an element with a specific class name.
    Class(String, Box<Selector>),

    /// Element that is being hovered.
    Hover(Box<Selector>),

    /// Element that currently has keyboard focus.
    Focus(Box<Selector>),

    /// Element that currently has keyboard focus, or contains a descendant that does.
    FocusWithin(Box<Selector>),

    /// Element that currently has keyboard focus, when focus is shown.
    FocusVisible(Box<Selector>),

    /// Element is the first child of its parent.
    FirstChild(Box<Selector>),

    /// Element is the last child of its parent.
    LastChild(Box<Selector>),

    /// Reference to the current element.
    Current(Box<Selector>),

    /// Reference to the parent of this element.
    Parent(Box<Selector>),

    /// List of alternate choices.
    #[allow(clippy::vec_box)]
    Either(Vec<Box<Selector>>),
}

impl Selector {
    /// Returns a number indicating how many levels up the entity ancestor hierarchy we might
    /// have to search to look for classes.
    pub fn depth(&self) -> usize {
        match self {
            Selector::Accept => 1,
            Selector::Class(_, next) => next.depth(),
            Selector::Hover(next)
            | Selector::Focus(next)
            | Selector::FocusWithin(next)
            | Selector::FocusVisible(next)
            | Selector::FirstChild(next)
            | Selector::LastChild(next) => next.depth(),
            Selector::Current(next) => next.depth(),
            Selector::Parent(next) => next.depth() + 1,
            Selector::Either(opts) => opts.iter().map(|next| next.depth()).max().unwrap_or(0),
        }
    }

    /// Returns whether this selector uses the hover pseudo-class.
    pub fn uses_hover(&self) -> bool {
        match self {
            Selector::Accept => false,
            Selector::Class(_, next) => next.uses_hover(),
            Selector::Hover(_) => true,
            Selector::Focus(next)
            | Selector::FocusWithin(next)
            | Selector::FocusVisible(next)
            | Selector::FirstChild(next)
            | Selector::LastChild(next)
            | Selector::Current(next) => next.uses_hover(),
            Selector::Parent(next) => next.uses_hover(),
            Selector::Either(opts) => opts
                .iter()
                .map(|next| next.uses_hover())
                .max()
                .unwrap_or(false),
        }
    }

    /// Returns whether this selector uses the hover pseudo-class.
    pub fn uses_focus_within(&self) -> bool {
        match self {
            Selector::Accept => false,
            Selector::Class(_, next) => next.uses_hover(),
            Selector::FocusWithin(_) => true,
            Selector::Hover(next)
            | Selector::Focus(next)
            | Selector::FocusVisible(next)
            | Selector::FirstChild(next)
            | Selector::LastChild(next)
            | Selector::Current(next) => next.uses_hover(),
            Selector::Parent(next) => next.uses_hover(),
            Selector::Either(opts) => opts
                .iter()
                .map(|next| next.uses_hover())
                .max()
                .unwrap_or(false),
        }
    }
}

impl std::str::FromStr for Selector {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        selector_parser::selector_parser
            .parse(input.trim())
            .map(|a| *a)
            .map_err(|e| e.to_string())
    }
}

impl fmt::Display for Selector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Selector::Accept => Ok(()),
            Selector::Current(prev) => {
                // Because 'current' comes first, reverse order
                let mut str = String::with_capacity(64);
                let mut p = prev.as_ref();
                while let Selector::Class(name, desc) = p {
                    str.insert_str(0, name);
                    str.insert(0, '.');
                    p = desc.as_ref()
                }
                str.insert(0, '&');
                write!(f, "{}{}", p, str)
            }

            Selector::Class(name, prev) => write!(f, "{}.{}", prev, name),
            Selector::Hover(prev) => write!(f, "{}:hover", prev),
            Selector::Focus(prev) => write!(f, "{}:focus", prev),
            Selector::FocusWithin(prev) => write!(f, "{}:focus-within", prev),
            Selector::FocusVisible(prev) => write!(f, "{}:focus-visible", prev),
            Selector::FirstChild(prev) => write!(f, "{}:first-child", prev),
            Selector::LastChild(prev) => write!(f, "{}:last-child", prev),
            Selector::Parent(prev) => match prev.as_ref() {
                Selector::Parent(_) => write!(f, "{}* > ", prev),
                _ => write!(f, "{} > ", prev),
            },
            Selector::Either(items) => {
                for (index, item) in items.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }
                    item.fmt(f)?
                }
                Ok(())
            }
        }
    }
}

/// A trait for matching a selector expression against an element.
pub trait SelectorMatcher {
    /// Given an array of match params representing the element's ancestor chain, match the
    /// selector expression with the params.
    fn selector_match(&self, selector: &Selector, entity: &Entity) -> bool;
}
