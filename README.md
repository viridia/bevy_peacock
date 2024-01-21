# Peacock

**Peacock** is a styling library for Bevy UI. It provides a way to associate dynamic styles
with Bevy entities, similar in concept to the familiar Cascading Style Sheets (CSS).

Peacock is not intended to be a drop-in replacement for CSS, and intentionally leaves out
a number of CSS features. What is left is a carefully curated set of capabilities. The feature
set of Peacock has been shaped by a number of influences:

* Experience on large-scale projects has shown certain features of CSS tend to produce code
  that is less maintainable.
* Certain features of CSS are confusing for novice users.
* Some features of CSS are less performant than others; in particular, some types of selectors
  combinations require a backtracking search to evaluate.

Bear in mind that CSS was originally intended to style documents, not user interfaces, and
it's design reflects this. Styling panels and widgets does not require the kind of complex
rules needed for rich documents, but it does need a rich set of primitives for dynamic states.

Many other CSS frameworks have come to similar conclusions to the ones listed above; one can
see in the design of various popular CSS-in-JS frameworks a decision to deliberately constrain
the power of CSS in order to produce code that is simpler and easier to maintain.

<!-- ## Getting started

For now, you can run the examples. The "complex" example shows off multiple features of the
library:

```sh
cargo run --example complex
``` -->

## Aspirations / guiding principles:

* Works with regular Bevy UI components.
* Supports CSS-like styling and dynamic visuals.
* Supports dynamic selectors such as classes, hover states, and focus.

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

## Examples usages

## Philosophy

There are several different ways to approach styling in Bevy. One is "inline styles", meaning that
you explicitly create style components (`BackgroundColor`, `Outline` and so on) in the template
and pass them in as parameters to the presenter.

A disadvantage of this approach is that you have limited ability to compose styles from different
sources. Rust has one mechanism for inheriting struct values from another struct, which is the
`..` syntax; this supposes that both of the struct values are known at the point of declaration.

Another disadvantage is that any dynamic style properties are strictly the responsibility of the
widget. Transitory state changes such as "hover" and "focus" require updating the UI tree and
patching the entities. This means that widgets can only have whatever dynamic properties
are implemented in the widget itself; it's not possible for an artist to come along later and add
hover or focus effects unless the widget is designed with hover effects in mind, and if the widget
has multiple parts, only the parts which have explicit support for those effects can be dynamically
styled.

An alternative to inline styles is a rule-based approach. However, the rule-basd cascade of
CSS is *too* powerful, and can lead to code that is hard to reason about and hard to maintain.

Peacock is inspired by CSS, but it is not CSS. Styles are currently built either as constants,
using a fluent syntax, or dynamically inline. A future addition should allow styles
to be loaded from assets, using a CSS-like syntax, and in fact the data representation of styles
has been carefully designed to allow for future serialization. Right now, however, the main focus
is on "editor" use cases, which likely will want styles defined in code anyway.

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

## Usage

### Using StyleHandles

`StyleHandle`s are typically created using the `.build()` method, which accepts a closure that takes
a builder object. The builder methods are flexible in the type of arguments they accept: for
example, methods such as `.margin_right()` and `.row_gap()` accept an `impl Length`, which can be
an integer (i32), a float (f32), or a Bevy `ui::Val` object. In the case where no unit is specified,
pixels is the default unit, so for example `.border(2)` specifies a border width of 2 pixels.

Styles are applied to an element using the `.styled()` method, which accepts either a single style,
or a tuple of styles.

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

pub fn hoverable(cx: Cx) -> impl View {
    Element::new()
        .styled(STYLE_HOVERABLE.clone())
        .children(cx.props.children.clone())
}
```

An element can have multiple styles. Styles are applied in order, first-come, first-serve.

Conditional styles can be added via selectors. It supports a limited subset of CSS syntax (basically the parts of CSS that don't require backtracking):

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

#### Animated Transitions

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

<!-- ### Class names

The `class_names` method can add class names to an element. Class names can be added conditionally
using the `.if_true()` modifier.

```rust
pub fn classnames_example(cx: Cx<Props>) -> impl View {
    Element::new()
        .class_names(("vertical", "selected".if_true(cx.props.selected)))
}
``` -->
