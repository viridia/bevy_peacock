//! Cursor definitions (not done yet)

/// 2D Cursor type - subset of standard CSS cursor types.
/// Note that in some cases, it may be desirable to have a 3d cursor in the world. In such
/// cases, set the 2D cursor to "none" and use a 3d cursor instead.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cursor {
    /// No cursor
    None,

    /// Cursor is a custom image (set cursor image and cursor offset seperately)
    CustomImage,

    /// A pointing arrow
    Default,

    /// A hand with a pointing finger
    Pointer,

    /// Hourglass
    Wait,

    /// Crosshair
    Crosshair,

    /// I-beam
    Text,

    /// Vertical I-beam
    VerticalText,

    /// 4-way arrow
    Move,

    /// "forbidden" symbol
    NotAllowed,

    /// Grabbing hand
    Grab,

    /// Column resize
    ColResize,

    /// Row resize
    RowResize,

    /// Magnifying Glass with Plus
    ZoomIn,

    /// Magnifying Glass with Minus
    ZoomOut,
}
