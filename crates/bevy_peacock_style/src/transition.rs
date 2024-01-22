use std::fmt::Debug;

/// Represents an animation timing function such as 'ease-in'.
pub trait TimingFunction
where
    Self: Send + Sync + Debug,
{
    fn eval(&self, t: f32) -> f32;
}

/// Module containing various useful timing functions.
pub mod timing {
    use std::{f32::consts::PI, fmt::Debug};

    use super::TimingFunction;

    /// Linear easing function
    pub struct Linear {}

    impl TimingFunction for Linear {
        fn eval(&self, t: f32) -> f32 {
            t
        }
    }

    impl Debug for Linear {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("linear")
        }
    }

    /// Cubic ease-in function
    pub struct EaseIn {}

    impl TimingFunction for EaseIn {
        fn eval(&self, t: f32) -> f32 {
            t * t * t
        }
    }

    impl Debug for EaseIn {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("ease-in")
        }
    }

    /// Cubic ease-out function
    pub struct EaseOut {}

    impl TimingFunction for EaseOut {
        fn eval(&self, t: f32) -> f32 {
            1. - (1. - t).powf(3.)
        }
    }

    impl Debug for EaseOut {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("ease-out")
        }
    }

    /// Sinusoidal ease-in-out function
    pub struct EaseInOut {}

    impl TimingFunction for EaseInOut {
        fn eval(&self, t: f32) -> f32 {
            -((PI * t).cos() - 1.) / 2.
        }
    }

    impl Debug for EaseInOut {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("ease-in-out")
        }
    }

    /// Linear easing function
    pub const LINEAR: &Linear = &Linear {};

    /// "ease-in" animation function
    pub const EASE_IN: &EaseIn = &EaseIn {};

    /// "ease-out" animation function
    pub const EASE_OUT: &EaseOut = &EaseOut {};

    /// "ease-in-out" animation function
    pub const EASE_IN_OUT: &EaseInOut = &EaseInOut {};
}

/// Specifies which property is being animated.
#[derive(Clone, Debug, PartialEq, Eq, Copy, Hash)]
pub enum TransitionProperty {
    /// Animate the element's transform
    Transform,

    /// Animate the element's background color
    BackgroundColor,

    /// Animate the element's border color
    BorderColor,

    /// Animate left
    Left,

    /// Animate top
    Top,

    /// Animate bottom
    Bottom,

    /// Animate right
    Right,

    /// Animate height
    Height,

    /// Animate width
    Width,

    /// Animate border left
    BorderLeft,

    /// Animate border top
    BorderTop,

    /// Animate border right
    BorderRight,

    /// Animate border bottom
    BorderBottom,
}

/// Defines a CSS-like animated transition
#[derive(Clone, Debug)]
pub struct Transition {
    /// Which property is to be animated.
    pub property: TransitionProperty,

    /// Delay before animation starts
    pub delay: f32,

    /// How long the animation should last
    pub duration: f32,

    /// Easing function
    pub timing: &'static dyn TimingFunction,
}

impl Default for Transition {
    fn default() -> Self {
        Self {
            property: TransitionProperty::Transform,
            delay: 0.,
            duration: 0.,
            timing: timing::LINEAR,
        }
    }
}

#[doc(hidden)]
pub struct TransitionState {
    pub transition: Transition,
    // pub(crate) direction: f32,
    pub clock: f32,
}

impl TransitionState {
    pub fn advance(&mut self, delta: f32) {
        if self.transition.duration > 0. {
            self.clock = (self.clock + delta / self.transition.duration).clamp(0., 1.);
        } else {
            self.clock = 1.;
        }
    }

    // Return the current t parameter
    pub fn t(&self) -> f32 {
        self.transition.timing.eval(self.clock)
    }
}
