# Peacock

**Peacock** is a styling library for Bevy UI. It provides a way to associate dynamic styles
with Bevy entities, similar in concept to the familiar Cascading Style Sheets (CSS).

Although Peacock is "CSS-like", it is not exactly CSS, as it is designed specifically for styling
Bevy UI nodes and defines only those properties that are meaninful in that context. It also
intentionally leaves out a number of CSS features, leaving behind a carefully curated set of
functionality.

## Features:

* Works with regular Bevy UI components, and is framework-agnostic.
* "CSS-like" style properties which translate into Bevy component attributes.
* Multiple ways of defining styles:
  * In Rust code, using a fluent syntax.
  * import `.pss` files directly into Rust code.
  * (**Planned**) load as a Bevy asset.
* Shortcut syntax for many properties, e.g. `.border(10)` is the same as
  `.border(ui::UiRect::all(ui::Val::Px(10.0)))`
* Dynamic selectors such as class names, hover states, and focus.
* Animated transitions with interpolation curves ("easing").

## Getting started

To install Peacock, you need to install the `PeacockPlugin`, and you also need to schedule the
`PeacockSystemSet` to run after your UI has finished updating all element styles and class names.

```rust
app.add_systems(
    Update,
    my_ui_framework_system.before(PeacockSystemSet),
)
.add_plugins(PeacockPlugin)
.add_plugins(EventListenerPlugin::<ScrollWheel>::default())
```

### Enabling Hover

Hovering (The ":hover" selector) is an optional feature which requires
[`bevy_mod_picking`](https://github.com/aevyrie/bevy_mod_picking). The feature flag
`bevy_mod_picking` enables this behavior and the dependency.

## Example usages

Here's an example of how to attach a style handle to an entity:

```rust
use bevy_peacock::*;
use static_init::dynamic;

#[dynamic]
static ROOT: StyleHandle = StyleHandle::build(|ss| {
    ss.background_color("#444")
        .font_size(14.)
        .font(Some(AssetPath::from("fonts/Ubuntu/Ubuntu-Medium.ttf")))
});

fn setup_view_root(mut commands: Commands) {
    commands
        .spawn(NodeBundle::default())
        .with_styles(ROOT.clone());
}
```

## Philosophy

There are several different ways to approach styling in Bevy. One is "inline styles", meaning that
you explicitly create style components (`BackgroundColor`, `Outline` and so on) and attach
them to the UI entity.

A disadvantage of this approach is that you have limited ability to compose styles from different
sources. Style composition is important because it enables certain kinds of creative workflows,
as well as a rich ecosystem in which artists and coders can contribute reusable visual elements and
shared styles without having to tightly coordinate their labor.

Another disadvantage is that any dynamic style properties are strictly the responsibility of the
widget. Transitory state changes such as "hover" and "focus" require updating the UI tree and
patching the entities. This means that widgets can only have whatever dynamic properties
are implemented in the widget itself; it's not possible for an artist to come along later and add
hover or focus effects unless the widget is designed with hover effects in mind, and if the widget
has multiple parts, only the parts which have explicit support for those effects can be dynamically
styled.

Finally, implementing dynamic variations (hover, focus, etc.) and animated transitions using
inline styles requires a lot of complex boilerplate code.

An alternative to inline styles is a rule-based approach that forms the basis of CSS. However,
the "cascade" logic of CSS is *too* powerful, and can lead to code that is hard to reason about
and hard to maintain.

CSS was originally intended to style documents, not user interfaces, and it's design reflects
this. Styling panels and widgets does not require the kind of complex rules needed for rich
documents, but it does need a rich set of primitives for dynamic states.

Many other CSS frameworks have come to similar conclusions to the ones listed above; one can
see in the design of various popular CSS-in-JS frameworks a decision to deliberately constrain
the power of CSS in order to produce code that is simpler and easier to maintain.

Because of this, Peacock's design resembles CSS but deliberately leaves out many CSS features.

## Usage

### Defining Styles

In Peacock, styles are Rust objects, called `StyleHandle`s which can be attached to a Bevy UI node.
More than one style can be assigned to the same entity, in which case the style properties are
algorithmically combined.

`StyleHandles` resemble CSS in the following ways:

* Style attributes are sparsely represented, meaning that only those properties that you actually
  declare are stored in the style handle.
* Styles are composable, meaning that you can "merge" multiple styles together to produce a union
  of all of them.
* Styles support both "long-form" and "shortcut" syntax variations. For example, the following are
  all equivalent:
  * `.border(ui::UiRect::all(ui::Val::Px(10.)))` -- a border of 10px on all sides
  * `.border(ui::Val::Px(10.))` -- Scalar is automatically converted to a rect
  * `.border(10.)` -- `Px` is assumed to be the default unit
  * `.border(10)` -- Integers are automatically converted to f32 type.
* Styles allow dynamism by defining "selectors", dynamic matching rules. These rules execute
  in their own dedicated ECS system, and use `Commands` to update the entity's style components.
* Styles allow for animated transitions of many properties.

However, they also differ from CSS in a number of important ways:

* There is no prioritization or "cascade", as this tends to be a source of confusion for web
  developers (Even CSS itself is moving away from this with the new "CSS layers" feature.) Instead
  styles are merged strictly in the order that they appear on the element.
* The syntax for selectors is limited, and certain CSS features which are (a) expensive to compute
  and (b) not needed for widget development have been left out.
* Styles can only affect the element they are assigned to, not their children. Styles can query
  the state of parent elements, but cannot affect them. This idea is borrowed from
  some popular CSS-in-JS frameworks, which have similar restrictions. The idea is to increase
  maintainability by making styles more deterministic and predictable.
* There is no support for CSS variables. This may be somewhat surprising because this is a very
  powerful feature of CSS which makes a lot of other, older CSS features unnecessary. However,
  it is anticipated that a comprehensive Bevy UI framework will have other ways to implement
  "inheritable" properties, such as scoped variables or contexts, and there's no need to have two
  separate ways of inheriting values in the UI hierarchy.

### Ordering and Priority

The algorithm for composing styles is very simple:

* Each style is merged in the order it appears on the element.
* Each style property is merged in the order it appears in the style definition.
* Dynamic selectors are evaluated after the base properties have been applied.

In some cases, you may want to build widgets that take styles as parameters, and merge those
styles with other styles that are built-in to the widget. For example, "unstyled" widget libraries
are a growing trend on the web: libraries which implement only widget behaviors (including
accessibility), allowing those widgets to be restyled by the app. These "unstyled" widgets will
actually have *some* built-in style rules (popup menus need absolute positioning for example), but
things like color and border are left up to the user of the library.

For cases like this, the recommendation is to establish a convention about the order in which
styles are merged, and then attach those styles to the entity in that order. One such convention
might be:

* **(lowest) Implementation Layer**: Styles which are needed by the widget in order to function at all.
* **Design Layer**: Styles which provide a common look and feel to the widgets. This includes
  widget variants (primary, secondary) and dynamic states (hover, focus).
* **Theme Layer**: Styles which override the appearance based on the current theme or mode.
* **(highest) App Layer**: Application-specific overrides, such as adding additional margins or alignment
  styles.

### Defining Styles 1: Constant Styles

In most cases, styles will be defined as constants. Styles are both immutable and sharable, so
there is no need to constantly re-create them as the app runs. The easiest way to create
a style is via the `.build()` method:

```rust
#[dynamic]
static STYLE_BUTTON: StyleHandle = StyleHandle::build(|ss| {
    ss.background_color(COLOR_GRAY_500)
        .border_color(COLOR_GRAY_700)
        .color(COLOR_GRAY_900)
        .selector(".pressed", |ss| ss.background_color(COLOR_GRAY_300))
        .selector(":hover", |ss| ss.background_color(COLOR_GRAY_400))
        .selector(":hover.pressed", |ss| ss.background_color(COLOR_GRAY_200))
        .selector(":focus", |ss| {
            ss.outline_color(COLOR_GRAY_400)
                .outline_width(2)
                .outline_offset(1)
        })
});
```

(The `dynamic` attribute is required for lazy initialization. You can also use `lazy_static`
if you prefer).

The builder methods are flexible in the type of arguments they accept: for example, methods such as `.margin_right()` and `.row_gap()` accept an `impl Length`, which can be an integer (i32),
a float (f32), or a Bevy `ui::Val` object. In the case where no unit is specified, pixels is the
default unit, so for example `.border(2)` specifies a border width of 2 pixels.

### Defining Styles 2: Inline Styles

Although styles are most often constants, there are cases where you may want to create a style
dynamically inline. For example, when implementing a slider widget, you might want to position
the slider thumb based on the current slider value. In cases like this, you can simply call
the same `.build()` method to create a new style handle in your event handler or reactive
update function.

### Defining Styles 3: Stylesheets

Peacock styles can be written in a separate stylesheet file with the extension `.pss`. The syntax
is similar to CSS, except that the property names don't use kebab-case, but instead match the
names of the Bevy style properties:

```
ROOT {
    display: flex;
    position_type: absolute;
}

BUTTON {
    display: flex;
    border: 3;
    border_color: #00ffff;
    flex_direction: row;
    height: 32;
    padding: 0 12;
    align_items: center;
}
```

To import the styles into your code, you can use the `import_stylesheet` macro:

```rust
import_stylesheet!(test_styles, "path/to/styles.pss");
```

The first argument is the module name which will contain the styles. This path is crate-relative:
that is, relative to the location of the Cargo.toml file (this is a limitation of proc macros).

The macro creates a Rust module that contains a static constant for each style rule.

### Defining Styles 4: Styles as Assets

(Currently planned)

### Using StyleHandles

To attach style handles to a UI node, create a new `ElementStyles` component and insert it into
the entity.

Here's an example of a widget which changes its border color when hovered:

```rust
use bevy::{prelude::*, ui};
use bevy_peacock::prelude::*;
use static_init::dynamic;

#[dynamic]
static STYLE_HOVERABLE: StyleHandle = StyleHandle::build(|ss| {
    ss.border_color("#383838")
        .border(1)
        .selector(":hover", |ss| {
            ss.border_color("#444")
        })
});

fn setup_view_root(mut commands: Commands) {
    commands
        .spawn(NodeBundle::default())
        .insert(ElementStyles::new(STYLE_HOVERABLE.clone()));
}
```
There's also a `.with_styles()` trait helper which is slightly less verbose:

```rust
fn setup_view_root(mut commands: Commands) {
    commands
        .spawn(NodeBundle::default())
        .with_styles((
            STYLE_1.clone(),
            STYLE_2.clone(),
            STYLE_3.clone(),
        ));
}
```
An element can have multiple styles. Styles are applied in order, first-come, first-serve.

Note on cloning: StyleHandles are `Arc`s, so cloning is relatively cheap.

### Selectors

Conditional styles can be added via selectors. It supports a limited subset of CSS syntax
(basically the parts of CSS that don't require backtracking):

* `.classname`
* `:hover`
* `:focus`, `:focus-within` and `:focus-visible`
* `:first-child` and `:last-child`
* `>` (parent combinator, e.g. `:hover > &`)
* `&` (current element)
* `,` (logical-or)
* **Planned:** `:not()`.

As stated previously, selectors only support styling the *current* node - that is, the node that
the style handle is attached to. Selectors can't affect child nodes - they need to have their own styles.

So for example, `".bg:hover > &"` is a valid selector expression, but `"&:hover > .bg"` is not valid.
The `&` must always be on the last term. The reason for this is performance - Peacock only supports those features of CSS that are lightning-fast.

### Class Names

You can add "class names" to an entity using the "ElementClasses" component. Class names can be
added conditionally using the `.if_true()` modifier.

```rust
pub fn classnames_example(is_selected: bool) -> impl View {
    commands
        .spawn(NodeBundle::default())
        .insert(ElementClasses::new((
            "vertical",
            "dark",
            "selected".if_true(is_selected)
        )))
}
```

There's also a `class_names()` trait helper:

```rust
pub fn classnames_example(is_selected: bool) -> impl View {
    commands
        .spawn(NodeBundle::default())
        .class_names((
            "vertical",
            "dark",
            "selected".if_true(is_selected)
        ))
}
```

### Animated Transitions

Peacock `StyleHandle`s support CSS-like transitions for some properties (mostly layout properties
like `width`, `height`, `left` and so on, as well as transform properties like `scale` and `rotation`.
Eventually color once we get lerping figured out.)

The `transition` style attribute indicates which properties you want to be animated. Here's an
example of how to animate a rotation:

```rust
#[dynamic]
static STYLE_DISCLOSURE_TRIANGLE: StyleHandle = StyleHandle::build(|ss| {
    ss.display(ui::Display::Flex)
        .transition(&vec![Transition {
            property: TransitionProperty::Transform,
            duration: 0.3,
            timing: timing::EASE_IN_OUT,
            ..default()
        }])
        .selector(".expanded", |ss| ss.rotation(PI / 2.))
});
```
How this works: when the styling system sees that a particular property is to be animated,
instead of modifying that style attribute directly, it injects an animation component that
contains a timer and an easing function. A separate ECS system updates the timer clock and
adjusts the style attribute.

Easing functions are just functions, so you can define whatever kind of easing you want.
