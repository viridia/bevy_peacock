use anyhow::Context as _;
use bevy::{render::color::Color, ui};
use bevy_peacock_style::{parse_stylesheet, Selector, SelectorEntry, StyleProp, StylePropList};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::{
    env,
    path::{Path, PathBuf},
};
use syn::{
    parse::{Parse, ParseStream, Result},
    parse_macro_input, Ident, LitStr, Token,
};

struct StylesheetInput {
    mod_name: Ident,
    path: String,
}

impl Parse for StylesheetInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let mod_name: Ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let path = input.parse::<LitStr>()?;
        Ok(StylesheetInput {
            mod_name,
            path: path.value(),
        })
    }
}

fn import_stylesheet_from_path(
    path_str: &String,
) -> anyhow::Result<(PathBuf, Vec<(String, StylePropList)>)> {
    let manifest_dir_env =
        env::var_os("CARGO_MANIFEST_DIR").context("CARGO_MANIFEST_DIR env var not found")?;
    let manifest_path = Path::new(&manifest_dir_env);
    let file_path = manifest_path.join(Path::new(&path_str));

    let stylesheet_src = std::fs::read_to_string(&file_path)
        .with_context(|| format!("failed to read stylesheet file: {}", file_path.display()))?;

    let stylesheet = match parse_stylesheet(&stylesheet_src) {
        Ok(stylesheet) => stylesheet,
        Err(err) => {
            return Err(anyhow::anyhow!(
                "failed to parse stylesheet file: {}\n{}",
                file_path.display(),
                err.to_string()
            ))
        }
    };

    Ok((file_path, stylesheet))
}

fn format_stylesheet_src(
    mod_name: &String,
    file_path: &Path,
    stylesheet: &[(String, StylePropList)],
) -> TokenStream {
    let mod_name = format_ident!("{}", mod_name);
    let rules = stylesheet.iter().map(|(name, style)| {
        let name = format_ident!("{}", name);
        let style = style.to_src();
        quote! {
            #[dynamic]
            pub static #name: StyleHandle = StyleHandle::new(#style);
        }
    });
    let path = file_path.to_str().unwrap();
    let output = quote! {
        mod #mod_name {
            use static_init::dynamic;
            use bevy_peacock::{StyleHandle, StyleProp, StylePropList, Selector};
            use bevy::{render::color::Color, ui};
            #[allow(dead_code)]
            const _: &str = include_str!(#path);
            #( #rules )*
        }
    };
    // println!("{}", output);
    output.into()
}

/// Import a Peacock stylesheet from a file path. The file path is relative to the Cargo.toml file.
///
/// Arguments:
/// mod_name - the name of the module to generate.
/// path - the path to the stylesheet file.
#[proc_macro]
pub fn import_stylesheet(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as StylesheetInput);
    match import_stylesheet_from_path(&input.path) {
        Ok((file_path, stylesheet)) => {
            format_stylesheet_src(&input.mod_name.to_string(), &file_path, &stylesheet)
        }
        Err(err) => syn::Error::new_spanned(&input.mod_name, err.to_string())
            .to_compile_error()
            .into(),
    }
}

trait ToSrc {
    fn to_src(&self) -> proc_macro2::TokenStream;
}

impl ToSrc for SelectorEntry {
    fn to_src(&self) -> proc_macro2::TokenStream {
        let (selector, props) = self;
        let selector = selector.to_src();
        let props = props.iter().map(|prop| prop.to_src()).collect::<Vec<_>>();
        quote! {
            (Box::new(Selector::parse(#selector).unwrap()), vec![#( #props ),*])
        }
    }
}

impl ToSrc for StylePropList {
    fn to_src(&self) -> proc_macro2::TokenStream {
        let props = self
            .get_props()
            .iter()
            .map(|prop| prop.to_src())
            .collect::<Vec<_>>();
        let selectors = self
            .get_selectors()
            .iter()
            .map(|selector| selector.to_src());
        quote! {
            StylePropList::from_raw(
                vec![#( #props ),*],
                vec![#( #selectors ),*],
            )
        }
    }
}

impl ToSrc for Selector {
    fn to_src(&self) -> proc_macro2::TokenStream {
        let s = format!("{}", self);
        quote! {#s}
    }
}

impl ToSrc for StyleProp {
    fn to_src(&self) -> proc_macro2::TokenStream {
        // println!("StyleProp: {:?}", self);
        match self {
            StyleProp::BackgroundImage(path) => {
                let path = path.to_src();
                quote! {
                    StyleProp::BackgroundImage(#path)
                }
            }
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
            StyleProp::ZIndex(z) => {
                let z = z.to_src();
                quote! {StyleProp::ZIndex(#z)}
            }
            StyleProp::Display(disp) => {
                let disp = disp.to_src();
                quote! {StyleProp::Display(#disp)}
            }
            StyleProp::Position(pos) => {
                let pos = pos.to_src();
                quote! {StyleProp::Position(#pos)}
            }
            StyleProp::Overflow(ov) => {
                let ov = ov.to_src();
                quote! {StyleProp::Overflow(#ov)}
            }
            StyleProp::OverflowX(ov) => {
                let ov = ov.to_src();
                quote! {StyleProp::OverflowX(#ov)}
            }
            StyleProp::OverflowY(ov) => {
                let ov = ov.to_src();
                quote! {StyleProp::OverflowY(#ov)}
            }
            StyleProp::Direction(dir) => {
                let dir = dir.to_src();
                quote! {StyleProp::Direction(#dir)}
            }
            StyleProp::Left(length) => {
                let length = length.to_src();
                quote! {StyleProp::Left(#length)}
            }
            StyleProp::Right(length) => {
                let length = length.to_src();
                quote! {StyleProp::Right(#length)}
            }
            StyleProp::Top(length) => {
                let length = length.to_src();
                quote! {StyleProp::Top(#length)}
            }
            StyleProp::Bottom(length) => {
                let length = length.to_src();
                quote! {StyleProp::Bottom(#length)}
            }
            StyleProp::Width(length) => {
                let length = length.to_src();
                quote! {StyleProp::Width(#length)}
            }
            StyleProp::Height(length) => {
                let length = length.to_src();
                quote! {StyleProp::Height(#length)}
            }
            StyleProp::MinWidth(length) => {
                let length = length.to_src();
                quote! {StyleProp::MinWidth(#length)}
            }
            StyleProp::MinHeight(length) => {
                let length = length.to_src();
                quote! {StyleProp::MinHeight(#length)}
            }
            StyleProp::MaxWidth(length) => {
                let length = length.to_src();
                quote! {StyleProp::MaxWidth(#length)}
            }
            StyleProp::MaxHeight(length) => {
                let length = length.to_src();
                quote! {StyleProp::MaxHeight(#length)}
            }
            StyleProp::AspectRatio(r) => {
                let r = r.to_src();
                quote! {StyleProp::AspectRatio(#r)}
            }
            StyleProp::Margin(length) => {
                let length = length.to_src();
                quote! {StyleProp::Margin(#length)}
            }
            StyleProp::MarginLeft(length) => {
                let length = length.to_src();
                quote! {StyleProp::MarginLeft(#length)}
            }
            StyleProp::MarginRight(length) => {
                let length = length.to_src();
                quote! {StyleProp::MarginRight(#length)}
            }
            StyleProp::MarginTop(length) => {
                let length = length.to_src();
                quote! {StyleProp::MarginTop(#length)}
            }
            StyleProp::MarginBottom(length) => {
                let length = length.to_src();
                quote! {StyleProp::MarginBottom(#length)}
            }
            StyleProp::Padding(length) => {
                let length = length.to_src();
                quote! {StyleProp::Padding(#length)}
            }
            StyleProp::PaddingLeft(length) => {
                let length = length.to_src();
                quote! {StyleProp::PaddingLeft(#length)}
            }
            StyleProp::PaddingRight(length) => {
                let length = length.to_src();
                quote! {StyleProp::PaddingRight(#length)}
            }
            StyleProp::PaddingTop(length) => {
                let length = length.to_src();
                quote! {StyleProp::PaddingTop(#length)}
            }
            StyleProp::PaddingBottom(length) => {
                let length = length.to_src();
                quote! {StyleProp::PaddingBottom(#length)}
            }
            StyleProp::Border(rect) => {
                let rect = rect.to_src();
                quote! {StyleProp::Border(#rect)}
            }
            StyleProp::BorderLeft(length) => {
                let length = length.to_src();
                quote! {StyleProp::BorderLeft(#length)}
            }
            StyleProp::BorderRight(length) => {
                let length = length.to_src();
                quote! {StyleProp::BorderRight(#length)}
            }
            StyleProp::BorderTop(length) => {
                let length = length.to_src();
                quote! {StyleProp::BorderTop(#length)}
            }
            StyleProp::BorderBottom(length) => {
                let length = length.to_src();
                quote! {StyleProp::BorderBottom(#length)}
            }
            StyleProp::FlexDirection(dir) => {
                let dir = dir.to_src();
                quote! {StyleProp::FlexDirection(#dir)}
            }
            StyleProp::FlexWrap(wrap) => {
                let wrap = wrap.to_src();
                quote! {StyleProp::FlexWrap(#wrap)}
            }
            StyleProp::FlexGrow(value) => {
                quote! {StyleProp::FlexGrow(#value)}
            }
            StyleProp::FlexShrink(value) => {
                quote! {StyleProp::FlexShrink(#value)}
            }
            StyleProp::FlexBasis(length) => {
                let length = length.to_src();
                quote! {StyleProp::FlexBasis(#length)}
            }
            StyleProp::RowGap(length) => {
                let length = length.to_src();
                quote! {StyleProp::RowGap(#length)}
            }
            StyleProp::ColumnGap(length) => {
                let length = length.to_src();
                quote! {StyleProp::ColumnGap(#length)}
            }
            StyleProp::Gap(length) => {
                let length = length.to_src();
                quote! {StyleProp::Gap(#length)}
            }
            StyleProp::AlignItems(value) => {
                let value = value.to_src();
                quote! {StyleProp::AlignItems(#value)}
            }
            StyleProp::AlignSelf(value) => {
                let value = value.to_src();
                quote! {StyleProp::AlignSelf(#value)}
            }
            StyleProp::AlignContent(value) => {
                let value = value.to_src();
                quote! {StyleProp::AlignContent(#value)}
            }
            StyleProp::JustifyItems(value) => {
                let value = value.to_src();
                quote! {StyleProp::JustifyItems(#value)}
            }
            StyleProp::JustifySelf(value) => {
                let value = value.to_src();
                quote! {StyleProp::JustifySelf(#value)}
            }
            StyleProp::JustifyContent(value) => {
                let value = value.to_src();
                quote! {StyleProp::JustifyContent(#value)}
            }
            StyleProp::GridAutoFlow(value) => {
                let value = value.to_src();
                quote! {StyleProp::GridAutoFlow(#value)}
            }
            StyleProp::GridTemplateRows(_) => todo!(),
            StyleProp::GridTemplateColumns(_) => todo!(),
            StyleProp::GridAutoRows(_) => todo!(),
            StyleProp::GridAutoColumns(_) => todo!(),
            StyleProp::GridRow(_) => todo!(),
            StyleProp::GridRowStart(value) => {
                quote! {StyleProp::GridRowStart(#value)}
            }
            StyleProp::GridRowSpan(value) => {
                quote! {StyleProp::GridRowSpan(#value)}
            }
            StyleProp::GridRowEnd(value) => {
                quote! {StyleProp::GridRowEnd(#value)}
            }
            StyleProp::GridColumn(_) => todo!(),
            StyleProp::GridColumnStart(value) => {
                quote! {StyleProp::GridColumnStart(#value)}
            }
            StyleProp::GridColumnSpan(value) => {
                quote! {StyleProp::GridColumnSpan(#value)}
            }
            StyleProp::GridColumnEnd(value) => {
                quote! {StyleProp::GridColumnEnd(#value)}
            }
            StyleProp::PointerEvents(_) => todo!(),
            StyleProp::Font(path) => {
                let path = path.to_src();
                quote! {
                    StyleProp::Font(#path)
                }
            }
            StyleProp::FontSize(value) => {
                quote! {StyleProp::FontSize(#value)}
            }
            StyleProp::OutlineColor(color) => {
                let color = color.to_src();
                quote! {
                    StyleProp::OutlineColor(#color)
                }
            }
            StyleProp::OutlineWidth(length) => {
                let length = length.to_src();
                quote! {StyleProp::OutlineWidth(#length)}
            }
            StyleProp::OutlineOffset(length) => {
                let length = length.to_src();
                quote! {StyleProp::OutlineOffset(#length)}
            }
            StyleProp::Cursor(_) => todo!(),
            StyleProp::CursorImage(_) => todo!(),
            StyleProp::CursorOffset(_) => todo!(),
            StyleProp::Scale(value) => {
                quote! {StyleProp::Scale(#value)}
            }
            StyleProp::ScaleX(value) => {
                quote! {StyleProp::ScaleX(#value)}
            }
            StyleProp::ScaleY(value) => {
                quote! {StyleProp::ScaleY(#value)}
            }
            StyleProp::Rotation(value) => {
                quote! {StyleProp::Rotation(#value)}
            }
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

impl ToSrc for f32 {
    fn to_src(&self) -> proc_macro2::TokenStream {
        quote! { #self }
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

impl ToSrc for bevy::asset::AssetPath<'static> {
    fn to_src(&self) -> proc_macro2::TokenStream {
        let path = self.path().to_str();
        quote! {bevy::asset::AssetPath::from(#path)}
    }
}

impl ToSrc for ui::Val {
    fn to_src(&self) -> proc_macro2::TokenStream {
        match self {
            ui::Val::Px(value) => quote! {ui::Val::Px(#value)},
            ui::Val::Percent(value) => quote! {ui::Val::Percent(#value)},
            ui::Val::Vh(value) => quote! {ui::Val::Vh(#value)},
            ui::Val::Vw(value) => quote! {ui::Val::Vw(#value)},
            ui::Val::VMin(value) => quote! {ui::Val::VMin(#value)},
            ui::Val::VMax(value) => quote! {ui::Val::VMax(#value)},
            ui::Val::Auto => quote! {ui::Val::Auto},
        }
    }
}

impl ToSrc for ui::UiRect {
    fn to_src(&self) -> proc_macro2::TokenStream {
        let left = self.left.to_src();
        let right = self.right.to_src();
        let top = self.top.to_src();
        let bottom = self.bottom.to_src();

        quote! {ui::UiRect::new(#left, #right, #top, #bottom)}
    }
}

impl ToSrc for ui::ZIndex {
    fn to_src(&self) -> proc_macro2::TokenStream {
        match self {
            ui::ZIndex::Local(n) => quote! {ui::ZIndex::Local(#n)},
            ui::ZIndex::Global(n) => quote! {ui::ZIndex::Global(#n)},
        }
    }
}

impl ToSrc for ui::Display {
    fn to_src(&self) -> proc_macro2::TokenStream {
        match self {
            ui::Display::None => quote! {ui::Display::None},
            ui::Display::Flex => quote! {ui::Display::Flex},
            ui::Display::Grid => quote! {ui::Display::Grid},
        }
    }
}

impl ToSrc for ui::PositionType {
    fn to_src(&self) -> proc_macro2::TokenStream {
        match self {
            ui::PositionType::Relative => quote! {ui::PositionType::Relative},
            ui::PositionType::Absolute => quote! {ui::PositionType::Absolute},
        }
    }
}

impl ToSrc for ui::OverflowAxis {
    fn to_src(&self) -> proc_macro2::TokenStream {
        match self {
            ui::OverflowAxis::Clip => quote! {ui::OverflowAxis::Clip},
            ui::OverflowAxis::Visible => quote! {ui::OverflowAxis::Visible},
        }
    }
}

impl ToSrc for ui::Direction {
    fn to_src(&self) -> proc_macro2::TokenStream {
        match self {
            ui::Direction::Inherit => quote! {ui::Direction::Inherit},
            ui::Direction::LeftToRight => quote! {ui::Direction::LeftToRight},
            ui::Direction::RightToLeft => quote! {ui::Direction::RightToLeft},
        }
    }
}

impl ToSrc for ui::FlexDirection {
    fn to_src(&self) -> proc_macro2::TokenStream {
        match self {
            ui::FlexDirection::Row => quote! {ui::FlexDirection::Row},
            ui::FlexDirection::RowReverse => quote! {ui::FlexDirection::RowReverse},
            ui::FlexDirection::Column => quote! {ui::FlexDirection::Column},
            ui::FlexDirection::ColumnReverse => quote! {ui::FlexDirection::ColumnReverse},
        }
    }
}

impl ToSrc for ui::FlexWrap {
    fn to_src(&self) -> proc_macro2::TokenStream {
        match self {
            ui::FlexWrap::Wrap => quote! {ui::FlexWrap::Wrap},
            ui::FlexWrap::WrapReverse => quote! {ui::FlexWrap::WrapReverse},
            ui::FlexWrap::NoWrap => quote! {ui::FlexWrap::NoWrap},
        }
    }
}

impl ToSrc for ui::AlignItems {
    fn to_src(&self) -> proc_macro2::TokenStream {
        match self {
            ui::AlignItems::Default => quote! {ui::AlignItems::Default},
            ui::AlignItems::Start => quote! {ui::AlignItems::Start},
            ui::AlignItems::End => quote! {ui::AlignItems::End},
            ui::AlignItems::FlexStart => quote! {ui::AlignItems::FlexStart},
            ui::AlignItems::FlexEnd => quote! {ui::AlignItems::FlexEnd},
            ui::AlignItems::Center => quote! {ui::AlignItems::Center},
            ui::AlignItems::Baseline => quote! {ui::AlignItems::Baseline},
            ui::AlignItems::Stretch => quote! {ui::AlignItems::Stretch},
        }
    }
}

impl ToSrc for ui::AlignContent {
    fn to_src(&self) -> proc_macro2::TokenStream {
        match self {
            ui::AlignContent::Default => quote! {ui::AlignContent::Default},
            ui::AlignContent::Start => quote! {ui::AlignContent::Start},
            ui::AlignContent::End => quote! {ui::AlignContent::End},
            ui::AlignContent::FlexStart => quote! {ui::AlignContent::FlexStart},
            ui::AlignContent::FlexEnd => quote! {ui::AlignContent::FlexEnd},
            ui::AlignContent::Center => quote! {ui::AlignContent::Center},
            ui::AlignContent::SpaceBetween => quote! {ui::AlignContent::SpaceBetween},
            ui::AlignContent::SpaceAround => quote! {ui::AlignContent::SpaceAround},
            ui::AlignContent::SpaceEvenly => quote! {ui::AlignContent::SpaceEvenly},
            ui::AlignContent::Stretch => quote! {ui::AlignContent::Stretch},
        }
    }
}

impl ToSrc for ui::AlignSelf {
    fn to_src(&self) -> proc_macro2::TokenStream {
        match self {
            ui::AlignSelf::Auto => quote! {ui::AlignSelf::Auto},
            ui::AlignSelf::Start => quote! {ui::AlignSelf::Start},
            ui::AlignSelf::End => quote! {ui::AlignSelf::End},
            ui::AlignSelf::FlexStart => quote! {ui::AlignSelf::FlexStart},
            ui::AlignSelf::FlexEnd => quote! {ui::AlignSelf::FlexEnd},
            ui::AlignSelf::Center => quote! {ui::AlignSelf::Center},
            ui::AlignSelf::Baseline => quote! {ui::AlignSelf::Baseline},
            ui::AlignSelf::Stretch => quote! {ui::AlignSelf::Stretch},
        }
    }
}

impl ToSrc for ui::JustifyItems {
    fn to_src(&self) -> proc_macro2::TokenStream {
        match self {
            ui::JustifyItems::Default => quote! {ui::JustifyItems::Default},
            ui::JustifyItems::Start => quote! {ui::JustifyItems::Start},
            ui::JustifyItems::End => quote! {ui::JustifyItems::End},
            ui::JustifyItems::Center => quote! {ui::JustifyItems::Center},
            ui::JustifyItems::Baseline => quote! {ui::JustifyItems::Baseline},
            ui::JustifyItems::Stretch => quote! {ui::JustifyItems::Stretch},
        }
    }
}

impl ToSrc for ui::JustifyContent {
    fn to_src(&self) -> proc_macro2::TokenStream {
        match self {
            ui::JustifyContent::Default => quote! {ui::JustifyContent::Default},
            ui::JustifyContent::Start => quote! {ui::JustifyContent::Start},
            ui::JustifyContent::End => quote! {ui::JustifyContent::End},
            ui::JustifyContent::FlexStart => quote! {ui::JustifyContent::FlexStart},
            ui::JustifyContent::FlexEnd => quote! {ui::JustifyContent::FlexEnd},
            ui::JustifyContent::Center => quote! {ui::JustifyContent::Center},
            ui::JustifyContent::SpaceBetween => quote! {ui::JustifyContent::SpaceBetween},
            ui::JustifyContent::SpaceAround => quote! {ui::JustifyContent::SpaceAround},
            ui::JustifyContent::SpaceEvenly => quote! {ui::JustifyContent::SpaceEvenly},
            ui::JustifyContent::Stretch => quote! {ui::JustifyContent::Stretch},
        }
    }
}

impl ToSrc for ui::JustifySelf {
    fn to_src(&self) -> proc_macro2::TokenStream {
        match self {
            ui::JustifySelf::Auto => quote! {ui::JustifySelf::Auto},
            ui::JustifySelf::Start => quote! {ui::JustifySelf::Start},
            ui::JustifySelf::End => quote! {ui::JustifySelf::End},
            ui::JustifySelf::Center => quote! {ui::JustifySelf::Center},
            ui::JustifySelf::Baseline => quote! {ui::JustifySelf::Baseline},
            ui::JustifySelf::Stretch => quote! {ui::JustifySelf::Stretch},
        }
    }
}

impl ToSrc for ui::GridAutoFlow {
    fn to_src(&self) -> proc_macro2::TokenStream {
        match self {
            ui::GridAutoFlow::Row => quote! {ui::GridAutoFlow::Row},
            ui::GridAutoFlow::RowDense => quote! {ui::GridAutoFlow::RowDense},
            ui::GridAutoFlow::Column => quote! {ui::GridAutoFlow::Column},
            ui::GridAutoFlow::ColumnDense => quote! {ui::GridAutoFlow::ColumnDense},
        }
    }
}
