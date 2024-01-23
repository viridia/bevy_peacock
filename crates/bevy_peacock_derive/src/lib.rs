use bevy::render::color::Color;
use bevy_peacock_style::StyleProp;
use proc_macro::TokenStream;
use quote::{format_ident, quote};

#[proc_macro]
pub fn import_stylesheet(_item: TokenStream) -> TokenStream {
    let mod_name = format_ident!("styles");
    let rules = vec![quote! {
        #[dynamic]
        static MAIN: StyleHandle = StyleHandle::build(|ss| {
            ss.position(bevy::ui::PositionType::Absolute)
                .left(10.)
                .top(10.)
                .bottom(10.)
                .right(10.)
                .border(2)
                .border_color("#888")
                .display(bevy::ui::Display::Flex)
        });
    }];
    dbg!(rules[0].to_string());
    let output = quote! {
        mod #mod_name {
            use static_init::dynamic;
            use bevy_peacock::{StyleHandle, StyleProp, StylePropList};

            #( #rules )*
        }
    };
    dbg!(output.to_string());
    output.into()
}

trait ToSrc {
    fn to_src(&self) -> proc_macro2::TokenStream;
}

impl ToSrc for StyleProp {
    fn to_src(&self) -> proc_macro2::TokenStream {
        match self {
            StyleProp::BackgroundImage(_) => todo!(),
            StyleProp::BackgroundColor(color) => {
                let color = color.to_src();
                quote! {
                    StyleProp::BackgroundColor(#color)
                }
            }
            StyleProp::BorderColor(color) => {
                let color = color.to_src();
                quote! {
                    StyleProp::BorderColor(#color)
                }
            }
            StyleProp::Color(color) => {
                let color = color.to_src();
                quote! {
                    StyleProp::Color(#color)
                }
            }
            StyleProp::ZIndex(_) => todo!(),
            StyleProp::Display(_) => todo!(),
            StyleProp::Position(_) => todo!(),
            StyleProp::Overflow(_) => todo!(),
            StyleProp::OverflowX(_) => todo!(),
            StyleProp::OverflowY(_) => todo!(),
            StyleProp::Direction(_) => todo!(),
            StyleProp::Left(_) => todo!(),
            StyleProp::Right(_) => todo!(),
            StyleProp::Top(_) => todo!(),
            StyleProp::Bottom(_) => todo!(),
            StyleProp::Width(_) => todo!(),
            StyleProp::Height(_) => todo!(),
            StyleProp::MinWidth(_) => todo!(),
            StyleProp::MinHeight(_) => todo!(),
            StyleProp::MaxWidth(_) => todo!(),
            StyleProp::MaxHeight(_) => todo!(),
            StyleProp::Margin(_) => todo!(),
            StyleProp::MarginLeft(_) => todo!(),
            StyleProp::MarginRight(_) => todo!(),
            StyleProp::MarginTop(_) => todo!(),
            StyleProp::MarginBottom(_) => todo!(),
            StyleProp::Padding(_) => todo!(),
            StyleProp::PaddingLeft(_) => todo!(),
            StyleProp::PaddingRight(_) => todo!(),
            StyleProp::PaddingTop(_) => todo!(),
            StyleProp::PaddingBottom(_) => todo!(),
            StyleProp::Border(_) => todo!(),
            StyleProp::BorderLeft(_) => todo!(),
            StyleProp::BorderRight(_) => todo!(),
            StyleProp::BorderTop(_) => todo!(),
            StyleProp::BorderBottom(_) => todo!(),
            StyleProp::FlexDirection(_) => todo!(),
            StyleProp::FlexWrap(_) => todo!(),
            StyleProp::FlexGrow(_) => todo!(),
            StyleProp::FlexShrink(_) => todo!(),
            StyleProp::FlexBasis(_) => todo!(),
            StyleProp::RowGap(_) => todo!(),
            StyleProp::ColumnGap(_) => todo!(),
            StyleProp::Gap(_) => todo!(),
            StyleProp::AlignItems(_) => todo!(),
            StyleProp::AlignSelf(_) => todo!(),
            StyleProp::AlignContent(_) => todo!(),
            StyleProp::JustifyItems(_) => todo!(),
            StyleProp::JustifySelf(_) => todo!(),
            StyleProp::JustifyContent(_) => todo!(),
            StyleProp::GridAutoFlow(_) => todo!(),
            StyleProp::GridTemplateRows(_) => todo!(),
            StyleProp::GridTemplateColumns(_) => todo!(),
            StyleProp::GridAutoRows(_) => todo!(),
            StyleProp::GridAutoColumns(_) => todo!(),
            StyleProp::GridRow(_) => todo!(),
            StyleProp::GridRowStart(_) => todo!(),
            StyleProp::GridRowSpan(_) => todo!(),
            StyleProp::GridRowEnd(_) => todo!(),
            StyleProp::GridColumn(_) => todo!(),
            StyleProp::GridColumnStart(_) => todo!(),
            StyleProp::GridColumnSpan(_) => todo!(),
            StyleProp::GridColumnEnd(_) => todo!(),
            StyleProp::PointerEvents(_) => todo!(),
            StyleProp::Font(_) => todo!(),
            StyleProp::FontSize(_) => todo!(),
            StyleProp::OutlineColor(_) => todo!(),
            StyleProp::OutlineWidth(_) => todo!(),
            StyleProp::OutlineOffset(_) => todo!(),
            StyleProp::Cursor(_) => todo!(),
            StyleProp::CursorImage(_) => todo!(),
            StyleProp::CursorOffset(_) => todo!(),
            StyleProp::Scale(_) => todo!(),
            StyleProp::ScaleX(_) => todo!(),
            StyleProp::ScaleY(_) => todo!(),
            StyleProp::Rotation(_) => todo!(),
            StyleProp::Translation(_) => todo!(),
            StyleProp::Transition(_) => todo!(),
        }
    }
}

impl<T> ToSrc for Option<T>
where
    T: ToSrc,
{
    fn to_src(&self) -> proc_macro2::TokenStream {
        match self {
            Some(t) => {
                let src: proc_macro2::TokenStream = t.to_src();
                quote! { Some(#src) }
            }
            None => quote! { None },
        }
    }
}

impl ToSrc for Color {
    fn to_src(&self) -> proc_macro2::TokenStream {
        match self {
            Color::Rgba {
                red,
                green,
                blue,
                alpha,
            } => quote! {
                Color::rgba(#red, #green, #blue, #alpha)
            },
            Color::RgbaLinear {
                red,
                green,
                blue,
                alpha,
            } => quote! {
                Color::rgba_linear(#red, #green, #blue, #alpha)
            },
            Color::Hsla {
                hue,
                saturation,
                lightness,
                alpha,
            } => quote! {
                Color::hsla(#hue, #saturation, #lightness, #alpha)
            },
            Color::Lcha {
                lightness,
                chroma,
                hue,
                alpha,
            } => quote! {
                Color::lcha(#lightness, #chroma, #hue, #alpha)
            },
        }
    }
}

// impl ToTokens for Color {
//     fn to_tokens(&self, tokens: &mut TokenStream) {
//         match self {}
//     }
// }

// #[dynamic]
// static STYLE_VSPLITTER: StyleHandle = StyleHandle::new(StylePropList::default().with_props(&[
//     StyleProp::BackgroundColor(Some(Color::hex("#181818").unwrap())),
//     StyleProp::BackgroundColor(Some(Color::hex("#181818").unwrap())),
// ]));

// mod styles_compiled {
//     use static_init::dynamic;
//     use bevy_peacock::StyleHandle;

//     #[dynamic]
//     #[allow(dead_code)]
//     pub static MAIN: StyleHandle = StyleHandle::build(|ss| {
//         ss.position(bevy::ui::PositionType::Absolute)
//             .left(10.)
//             .top(10.)
//             .bottom(10.)
//             .right(10.)
//             .border(2)
//             .border_color("#888")
//             .display(bevy::ui::Display::Flex)
//     });
// }
