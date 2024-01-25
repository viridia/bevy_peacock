use bevy::{render::color::Color, ui};
use winnow::{
    ascii::{escaped_transform, multispace0},
    ascii::{float, space1},
    combinator::{alt, cut_err, delimited, eof, opt, preceded, repeat, separated, terminated},
    error::{
        ContextError, ErrMode, ErrorKind, FromExternalError, ParseError, StrContext,
        StrContextValue,
    },
    stream::AsChar,
    token::{none_of, one_of, take_while},
    PResult, Parser,
};

use crate::{selector_parser, style::SelectorEntry, StyleProp, StylePropList};

#[derive(Debug, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum StyleParsingError {
    InvalidPropertyName(String),
    InvalidPropertyType(String),
    InvalidPropertyValue(String),
}

impl std::fmt::Display for StyleParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StyleParsingError::InvalidPropertyName(name) => {
                write!(f, "invalid property name: '{}'", name)
            }
            StyleParsingError::InvalidPropertyType(ty) => {
                write!(f, "invalid property type: {}", ty)
            }
            StyleParsingError::InvalidPropertyValue(val) => {
                write!(f, "invalid property value: {}", val)
            }
        }
    }
}

impl std::error::Error for StyleParsingError {}

enum PropValue<'s> {
    Ident(&'s str),
    Number(f32),
    String(String),
    Length(ui::Val),
    List(Vec<PropValue<'s>>),
    Color(Color),
}

#[derive(Debug)]
enum StylePropOrSelector {
    StyleProp(StyleProp),
    Selector(SelectorEntry),
}

impl<'s> PropValue<'s> {
    fn type_name(&self) -> String {
        match self {
            PropValue::Ident(_) => "ident".to_string(),
            PropValue::Number(_) => "number".to_string(),
            PropValue::String(_) => "string".to_string(),
            PropValue::Length(_) => "length".to_string(),
            PropValue::List(_) => "list".to_string(),
            PropValue::Color(_) => "color".to_string(),
        }
    }
}

fn whitespace(input: &mut &str) -> PResult<()> {
    repeat(
        ..,
        alt((
            ("//", take_while(0.., |c| c != '\r' && c != '\n')).void(),
            one_of(['\n', '\r', '\t', ' ']).void(),
        )),
    )
    .parse_next(input)
}

fn prop_name<'s>(input: &mut &'s str) -> PResult<&'s str> {
    (
        one_of(AsChar::is_alpha),
        take_while(.., (AsChar::is_alphanum, '_')),
    )
        .recognize()
        .parse_next(input)
}

fn ident<'s>(input: &mut &'s str) -> PResult<PropValue<'s>> {
    (
        one_of(AsChar::is_alpha),
        take_while(.., (AsChar::is_alphanum, '_')),
    )
        .recognize()
        .map(|cls: &str| PropValue::Ident(cls))
        .parse_next(input)
}

fn string_chars(input: &mut &str) -> PResult<String> {
    escaped_transform(
        none_of(['"', '\\']).recognize(),
        '\\',
        alt(("\\".value("\\"), "\"".value("\""), "n".value("\n"))),
    )
    .parse_next(input)
}

fn string<'s>(input: &mut &'s str) -> PResult<PropValue<'s>> {
    (delimited('"', string_chars, '"'))
        .map(PropValue::String)
        .parse_next(input)
}

fn color_arg_sep(input: &mut &str) -> PResult<()> {
    (multispace0, opt((',', multispace0)))
        .void()
        .parse_next(input)
}

fn alpha_sep(input: &mut &str) -> PResult<()> {
    (multispace0, opt((one_of([',', '/']), multispace0)))
        .void()
        .parse_next(input)
}

fn f32_arg(input: &mut &str) -> PResult<f32> {
    float.parse_next(input)
}

fn length<'s>(input: &mut &'s str) -> PResult<PropValue<'s>> {
    alt((
        "auto".map(|_| PropValue::Length(ui::Val::Auto)),
        (f32_arg, alt(("%", "px", "vh", "vw", "vmin", "vmax"))).map(|(num, unit)| match unit {
            "%" => PropValue::Length(ui::Val::Percent(num)),
            "px" => PropValue::Length(ui::Val::Px(num)),
            "vh" => PropValue::Length(ui::Val::Vh(num)),
            "vw" => PropValue::Length(ui::Val::Vw(num)),
            "vmin" => PropValue::Length(ui::Val::VMin(num)),
            "vmax" => PropValue::Length(ui::Val::VMax(num)),
            _ => unreachable!(),
        }),
        f32_arg.map(PropValue::Number),
    ))
    .parse_next(input)
}

fn length_list<'s>(input: &mut &'s str) -> PResult<PropValue<'s>> {
    separated(2..4, length, space1)
        .map(|vals: Vec<PropValue<'s>>| PropValue::List(vals))
        .parse_next(input)
}

fn color_fn<'s>(input: &mut &'s str) -> PResult<PropValue<'s>> {
    (
        alt(("rgb", "rgba", "rgb_linear", "rgba_linear", "hsl", "hsla")),
        multispace0,
        '(',
        cut_err((
            multispace0,
            terminated(f32_arg, color_arg_sep),
            terminated(f32_arg, color_arg_sep),
            f32_arg,
            opt(preceded(alpha_sep, f32_arg)),
            multispace0,
            ')',
        )),
    )
        .map(|(f, _, _, (_, a1, a2, a3, a4, _, _))| match f {
            "rgb" | "rgba" => {
                if let Some(alpha) = a4 {
                    PropValue::Color(Color::rgba(a1, a2, a3, alpha))
                } else {
                    PropValue::Color(Color::rgb(a1, a2, a3))
                }
            }
            "rgb_linear" | "rgba_linear" => {
                if let Some(alpha) = a4 {
                    PropValue::Color(Color::rgba_linear(a1, a2, a3, alpha))
                } else {
                    PropValue::Color(Color::rgb_linear(a1, a2, a3))
                }
            }
            "hsl" | "hsla" => {
                if let Some(alpha) = a4 {
                    PropValue::Color(Color::hsla(a1, a2, a3, alpha))
                } else {
                    PropValue::Color(Color::hsl(a1, a2, a3))
                }
            }
            _ => unreachable!(),
        })
        .parse_next(input)
}

fn color<'s>(input: &mut &'s str) -> PResult<PropValue<'s>> {
    alt((
        ('#', take_while(1..8, AsChar::is_hex_digit))
            .recognize()
            .try_map(|s| Color::hex(s).map(PropValue::Color)),
        color_fn,
    ))
    .parse_next(input)
}

fn create_prop(name: &str, value: &PropValue) -> Result<StyleProp, StyleParsingError> {
    match name {
        "background_color" => Ok(StyleProp::BackgroundColor(value.coerce()?)),
        "border_color" => Ok(StyleProp::BorderColor(value.coerce()?)),
        "color" => Ok(StyleProp::Color(value.coerce()?)),
        "display" => Ok(StyleProp::Display(value.coerce()?)),
        "position_type" => Ok(StyleProp::Position(value.coerce()?)),

        "left" => Ok(StyleProp::Left(value.coerce()?)),
        "right" => Ok(StyleProp::Right(value.coerce()?)),
        "top" => Ok(StyleProp::Top(value.coerce()?)),
        "bottom" => Ok(StyleProp::Bottom(value.coerce()?)),

        "width" => Ok(StyleProp::Width(value.coerce()?)),
        "height" => Ok(StyleProp::Height(value.coerce()?)),
        "min_width" => Ok(StyleProp::MinWidth(value.coerce()?)),
        "min_height" => Ok(StyleProp::MinHeight(value.coerce()?)),
        "max_width" => Ok(StyleProp::MaxWidth(value.coerce()?)),
        "max_height" => Ok(StyleProp::MaxHeight(value.coerce()?)),

        "border" => Ok(StyleProp::Border(value.coerce()?)),
        "border_left" => Ok(StyleProp::BorderLeft(value.coerce()?)),
        "border_right" => Ok(StyleProp::BorderRight(value.coerce()?)),
        "border_top" => Ok(StyleProp::BorderTop(value.coerce()?)),
        "border_bottom" => Ok(StyleProp::BorderBottom(value.coerce()?)),

        "padding" => Ok(StyleProp::Padding(value.coerce()?)),
        "padding_left" => Ok(StyleProp::PaddingLeft(value.coerce()?)),
        "padding_right" => Ok(StyleProp::PaddingRight(value.coerce()?)),
        "padding_top" => Ok(StyleProp::PaddingTop(value.coerce()?)),
        "padding_bottom" => Ok(StyleProp::PaddingBottom(value.coerce()?)),

        "margin" => Ok(StyleProp::Margin(value.coerce()?)),
        "margin_left" => Ok(StyleProp::MarginLeft(value.coerce()?)),
        "margin_right" => Ok(StyleProp::MarginRight(value.coerce()?)),
        "margin_top" => Ok(StyleProp::MarginTop(value.coerce()?)),
        "margin_bottom" => Ok(StyleProp::MarginBottom(value.coerce()?)),

        "flex_direction" => Ok(StyleProp::FlexDirection(value.coerce()?)),
        "flex_basis" => Ok(StyleProp::FlexBasis(value.coerce()?)),
        "row_gap" => Ok(StyleProp::RowGap(value.coerce()?)),
        "column_gap" => Ok(StyleProp::ColumnGap(value.coerce()?)),
        "gap" => Ok(StyleProp::Gap(value.coerce()?)),

        "align_items" => Ok(StyleProp::AlignItems(value.coerce()?)),
        "align_content" => Ok(StyleProp::AlignContent(value.coerce()?)),
        "align_self" => Ok(StyleProp::AlignSelf(value.coerce()?)),

        "justify_items" => Ok(StyleProp::JustifyItems(value.coerce()?)),
        "justify_content" => Ok(StyleProp::JustifyContent(value.coerce()?)),
        "justify_self" => Ok(StyleProp::JustifySelf(value.coerce()?)),

        _ => Err(StyleParsingError::InvalidPropertyName(name.to_owned())),
    }
}

fn style_prop_inner(input: &mut &str) -> PResult<StyleProp> {
    let (name, _, _, _, value, _, _) = (
        prop_name,
        multispace0,
        ':',
        multispace0,
        alt((color, ident, length_list, length, string))
            .context(StrContext::Label("property value")),
        multispace0,
        cut_err(
            ';'.context(StrContext::Expected(StrContextValue::Description(
                "semicolon",
            ))),
        ),
    )
        .parse_next(input)?;

    create_prop(name, &value)
        .map_err(|err| ErrMode::from_external_error(input, ErrorKind::Fail, err).cut())
}

fn style_prop(input: &mut &str) -> PResult<StyleProp> {
    (style_prop_inner, whitespace)
        .map(|(prop, _)| Ok(prop))
        .parse_next(input)?
}

fn selector_prop_list_items(input: &mut &str) -> PResult<Vec<StyleProp>> {
    repeat(.., style_prop).parse_next(input)
}

fn selector(input: &mut &str) -> PResult<SelectorEntry> {
    (
        selector_parser::selector_parser,
        whitespace,
        '{',
        cut_err((whitespace, selector_prop_list_items, '}', whitespace)),
    )
        .map(|(sel, _, _, (_, props, _, _))| (sel, props))
        .context(StrContext::Expected(StrContextValue::Description(
            "selector",
        )))
        .parse_next(input)
}

fn style_prop_list_items(input: &mut &str) -> PResult<Vec<StylePropOrSelector>> {
    repeat(
        ..,
        alt((
            style_prop.map(StylePropOrSelector::StyleProp),
            selector.map(StylePropOrSelector::Selector),
        )),
    )
    .parse_next(input)
}

fn style_prop_list(input: &mut &str) -> PResult<(String, StylePropList)> {
    (
        ident.context(StrContext::Label("style name")).recognize(),
        whitespace,
        '{',
        cut_err((whitespace, style_prop_list_items, '}')),
    )
        .map(|(name, _, _, (_, mut items, _))| {
            let mut props: Vec<StyleProp> = Vec::new();
            let mut selectors: Vec<SelectorEntry> = Vec::new();
            for item in items.drain(..) {
                match item {
                    StylePropOrSelector::StyleProp(prop) => {
                        props.push(prop);
                    }
                    StylePropOrSelector::Selector(sel) => {
                        selectors.push(sel);
                    }
                }
            }
            (name.to_owned(), StylePropList { props, selectors })
        })
        .parse_next(input)
}

fn stylesheet(input: &mut &str) -> PResult<Vec<(String, StylePropList)>> {
    (
        repeat(.., (whitespace, style_prop_list).map(|(_, l)| l)),
        whitespace,
        eof,
    )
        .map(|(entries, _, _)| entries)
        .parse_next(input)
}

/// Parse a stylesheet from a string.
pub fn parse_stylesheet(
    input: &str,
) -> Result<Vec<(String, StylePropList)>, ParseError<&str, ContextError>> {
    stylesheet.parse(input)
}

trait CoercePropValue<T> {
    fn coerce(&self) -> Result<T, StyleParsingError>;
}

impl<'s> CoercePropValue<Option<Color>> for PropValue<'s> {
    fn coerce(&self) -> Result<Option<Color>, StyleParsingError> {
        match self {
            PropValue::Color(color) => Ok(Some(*color)),
            PropValue::Ident("transparent") => Ok(None),
            PropValue::Ident(id) => Err(StyleParsingError::InvalidPropertyType(format!(
                "\"{}\"",
                id
            ))),
            _ => Err(StyleParsingError::InvalidPropertyType(
                PropValue::type_name(self),
            )),
        }
    }
}

impl<'s> CoercePropValue<ui::Val> for PropValue<'s> {
    fn coerce(&self) -> Result<ui::Val, StyleParsingError> {
        match self {
            PropValue::Number(p) => Ok(ui::Val::Px(*p)),
            PropValue::Length(l) => Ok(*l),
            PropValue::Ident("auto") => Ok(ui::Val::Auto),
            PropValue::Ident(id) => Err(StyleParsingError::InvalidPropertyValue(format!(
                "\"{}\"",
                id
            ))),
            _ => Err(StyleParsingError::InvalidPropertyType(
                PropValue::type_name(self),
            )),
        }
    }
}

impl<'s> CoercePropValue<ui::UiRect> for PropValue<'s> {
    fn coerce(&self) -> Result<ui::UiRect, StyleParsingError> {
        match self {
            PropValue::Number(p) => Ok(ui::UiRect::all(ui::Val::Px(*p))),
            PropValue::Length(l) => Ok(ui::UiRect::all(*l)),
            PropValue::List(vals) => match vals.len() {
                2 => Ok(ui::UiRect::axes(vals[1].coerce()?, vals[0].coerce()?)),
                3 => Ok(ui::UiRect::new(
                    // CSS order: top, right/left, bottom
                    vals[1].coerce()?,
                    vals[1].coerce()?,
                    vals[0].coerce()?,
                    vals[2].coerce()?,
                )),
                4 => Ok(ui::UiRect::new(
                    // CSS order: top, right, bottom, left
                    vals[3].coerce()?,
                    vals[1].coerce()?,
                    vals[0].coerce()?,
                    vals[2].coerce()?,
                )),
                _ => unreachable!(),
            },
            PropValue::Ident("auto") => Ok(ui::UiRect::all(ui::Val::Auto)),
            PropValue::Ident(id) => Err(StyleParsingError::InvalidPropertyValue(format!(
                "\"{}\"",
                id
            ))),
            _ => Err(StyleParsingError::InvalidPropertyType(
                PropValue::type_name(self),
            )),
        }
    }
}

impl<'s> CoercePropValue<ui::Display> for PropValue<'s> {
    fn coerce(&self) -> Result<ui::Display, StyleParsingError> {
        match self {
            PropValue::Ident("flex") => Ok(ui::Display::Flex),
            PropValue::Ident("grid") => Ok(ui::Display::Grid),
            PropValue::Ident("none") => Ok(ui::Display::None),
            PropValue::Ident(id) => Err(StyleParsingError::InvalidPropertyValue(format!(
                "\"{}\"",
                id
            ))),
            _ => Err(StyleParsingError::InvalidPropertyType(
                PropValue::type_name(self),
            )),
        }
    }
}

impl<'s> CoercePropValue<ui::PositionType> for PropValue<'s> {
    fn coerce(&self) -> Result<ui::PositionType, StyleParsingError> {
        match self {
            PropValue::Ident("relative") => Ok(ui::PositionType::Relative),
            PropValue::Ident("absolute") => Ok(ui::PositionType::Absolute),
            PropValue::Ident(id) => Err(StyleParsingError::InvalidPropertyValue(format!(
                "\"{}\"",
                id
            ))),
            _ => Err(StyleParsingError::InvalidPropertyType(
                PropValue::type_name(self),
            )),
        }
    }
}

impl<'s> CoercePropValue<ui::FlexDirection> for PropValue<'s> {
    fn coerce(&self) -> Result<ui::FlexDirection, StyleParsingError> {
        match self {
            PropValue::Ident("row") => Ok(ui::FlexDirection::Row),
            PropValue::Ident("row_reverse") => Ok(ui::FlexDirection::RowReverse),
            PropValue::Ident("column") => Ok(ui::FlexDirection::Column),
            PropValue::Ident("column_reverse") => Ok(ui::FlexDirection::ColumnReverse),
            PropValue::Ident(id) => Err(StyleParsingError::InvalidPropertyValue(format!(
                "\"{}\"",
                id
            ))),
            _ => Err(StyleParsingError::InvalidPropertyType(
                PropValue::type_name(self),
            )),
        }
    }
}

impl<'s> CoercePropValue<ui::AlignItems> for PropValue<'s> {
    fn coerce(&self) -> Result<ui::AlignItems, StyleParsingError> {
        match self {
            PropValue::Ident("default") => Ok(ui::AlignItems::Default),
            PropValue::Ident("start") => Ok(ui::AlignItems::Start),
            PropValue::Ident("end") => Ok(ui::AlignItems::End),
            PropValue::Ident("flex_start") => Ok(ui::AlignItems::FlexStart),
            PropValue::Ident("flex_end") => Ok(ui::AlignItems::FlexEnd),
            PropValue::Ident("center") => Ok(ui::AlignItems::Center),
            PropValue::Ident("baseline") => Ok(ui::AlignItems::Baseline),
            PropValue::Ident("stretch") => Ok(ui::AlignItems::Stretch),
            PropValue::Ident(id) => Err(StyleParsingError::InvalidPropertyValue(format!(
                "\"{}\"",
                id
            ))),
            _ => Err(StyleParsingError::InvalidPropertyType(
                PropValue::type_name(self),
            )),
        }
    }
}

impl<'s> CoercePropValue<ui::AlignContent> for PropValue<'s> {
    fn coerce(&self) -> Result<ui::AlignContent, StyleParsingError> {
        match self {
            PropValue::Ident("default") => Ok(ui::AlignContent::Default),
            PropValue::Ident("start") => Ok(ui::AlignContent::Start),
            PropValue::Ident("end") => Ok(ui::AlignContent::End),
            PropValue::Ident("flex_start") => Ok(ui::AlignContent::FlexStart),
            PropValue::Ident("flex_end") => Ok(ui::AlignContent::FlexEnd),
            PropValue::Ident("center") => Ok(ui::AlignContent::Center),
            PropValue::Ident("stretch") => Ok(ui::AlignContent::Stretch),
            PropValue::Ident("space_between") => Ok(ui::AlignContent::SpaceBetween),
            PropValue::Ident("space_evenly") => Ok(ui::AlignContent::SpaceEvenly),
            PropValue::Ident("space_around") => Ok(ui::AlignContent::SpaceAround),
            PropValue::Ident(id) => Err(StyleParsingError::InvalidPropertyValue(format!(
                "\"{}\"",
                id
            ))),
            _ => Err(StyleParsingError::InvalidPropertyType(
                PropValue::type_name(self),
            )),
        }
    }
}

impl<'s> CoercePropValue<ui::AlignSelf> for PropValue<'s> {
    fn coerce(&self) -> Result<ui::AlignSelf, StyleParsingError> {
        match self {
            PropValue::Ident("auto") => Ok(ui::AlignSelf::Auto),
            PropValue::Ident("start") => Ok(ui::AlignSelf::Start),
            PropValue::Ident("end") => Ok(ui::AlignSelf::End),
            PropValue::Ident("flex_start") => Ok(ui::AlignSelf::FlexStart),
            PropValue::Ident("flex_end") => Ok(ui::AlignSelf::FlexEnd),
            PropValue::Ident("center") => Ok(ui::AlignSelf::Center),
            PropValue::Ident("baseline") => Ok(ui::AlignSelf::Baseline),
            PropValue::Ident("stretch") => Ok(ui::AlignSelf::Stretch),
            PropValue::Ident(id) => Err(StyleParsingError::InvalidPropertyValue(format!(
                "\"{}\"",
                id
            ))),
            _ => Err(StyleParsingError::InvalidPropertyType(
                PropValue::type_name(self),
            )),
        }
    }
}

impl<'s> CoercePropValue<ui::JustifyItems> for PropValue<'s> {
    fn coerce(&self) -> Result<ui::JustifyItems, StyleParsingError> {
        match self {
            PropValue::Ident("default") => Ok(ui::JustifyItems::Default),
            PropValue::Ident("start") => Ok(ui::JustifyItems::Start),
            PropValue::Ident("end") => Ok(ui::JustifyItems::End),
            PropValue::Ident("center") => Ok(ui::JustifyItems::Center),
            PropValue::Ident("baseline") => Ok(ui::JustifyItems::Baseline),
            PropValue::Ident("stretch") => Ok(ui::JustifyItems::Stretch),
            PropValue::Ident(id) => Err(StyleParsingError::InvalidPropertyValue(format!(
                "\"{}\"",
                id
            ))),
            _ => Err(StyleParsingError::InvalidPropertyType(
                PropValue::type_name(self),
            )),
        }
    }
}

impl<'s> CoercePropValue<ui::JustifyContent> for PropValue<'s> {
    fn coerce(&self) -> Result<ui::JustifyContent, StyleParsingError> {
        match self {
            PropValue::Ident("default") => Ok(ui::JustifyContent::Default),
            PropValue::Ident("start") => Ok(ui::JustifyContent::Start),
            PropValue::Ident("end") => Ok(ui::JustifyContent::End),
            PropValue::Ident("flex_start") => Ok(ui::JustifyContent::FlexStart),
            PropValue::Ident("flex_end") => Ok(ui::JustifyContent::FlexEnd),
            PropValue::Ident("center") => Ok(ui::JustifyContent::Center),
            PropValue::Ident("stretch") => Ok(ui::JustifyContent::Stretch),
            PropValue::Ident("space_between") => Ok(ui::JustifyContent::SpaceBetween),
            PropValue::Ident("space_evenly") => Ok(ui::JustifyContent::SpaceEvenly),
            PropValue::Ident("space_around") => Ok(ui::JustifyContent::SpaceAround),
            PropValue::Ident(id) => Err(StyleParsingError::InvalidPropertyValue(format!(
                "\"{}\"",
                id
            ))),
            _ => Err(StyleParsingError::InvalidPropertyType(
                PropValue::type_name(self),
            )),
        }
    }
}

impl<'s> CoercePropValue<ui::JustifySelf> for PropValue<'s> {
    fn coerce(&self) -> Result<ui::JustifySelf, StyleParsingError> {
        match self {
            PropValue::Ident("auto") => Ok(ui::JustifySelf::Auto),
            PropValue::Ident("start") => Ok(ui::JustifySelf::Start),
            PropValue::Ident("end") => Ok(ui::JustifySelf::End),
            PropValue::Ident("center") => Ok(ui::JustifySelf::Center),
            PropValue::Ident("baseline") => Ok(ui::JustifySelf::Baseline),
            PropValue::Ident("stretch") => Ok(ui::JustifySelf::Stretch),
            PropValue::Ident(id) => Err(StyleParsingError::InvalidPropertyValue(format!(
                "\"{}\"",
                id
            ))),
            _ => Err(StyleParsingError::InvalidPropertyType(
                PropValue::type_name(self),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use winnow::error::ContextError;

    use super::*;

    #[track_caller]
    fn run_parser<'s, T>(mut parser: impl Parser<&'s str, T, ContextError>, input: &'s str) -> T {
        match parser.parse(input) {
            Ok(val) => val,
            Err(e) => {
                panic!("{}", e.to_string());
            }
        }
    }

    #[track_caller]
    fn run_parser_err<'s, T>(
        mut parser: impl Parser<&'s str, T, ContextError>,
        input: &'s str,
    ) -> String
    where
        T: std::fmt::Debug,
    {
        parser.parse(input).map_err(|e| e.to_string()).unwrap_err()
    }

    #[test]
    fn test_style_invalid_prop() {
        let err = run_parser_err(style_prop, "foo: #fff;");
        assert!(err.contains("invalid property name:"));
        assert!(err.contains("foo: #fff"));
    }

    #[test]
    fn test_style_invalid_prop_value() {
        let err = run_parser_err(style_prop, "background_color: 1;");
        assert!(err.contains("invalid property type:"));
        assert!(err.contains("background_color:"));
    }

    #[test]
    fn test_parse_color() {
        let result = run_parser(style_prop, "background_color: #fff;");
        match result {
            StyleProp::BackgroundColor(Some(color)) => {
                assert_eq!(color, Color::rgb(1.0, 1.0, 1.0));
            }
            _ => panic!("incorrect result: {:?}", result),
        }

        let result = run_parser(style_prop, "background_color: rgb(1, 1, 1);");
        match result {
            StyleProp::BackgroundColor(Some(color)) => {
                assert_eq!(color, Color::rgb(1.0, 1.0, 1.0));
            }
            _ => panic!("incorrect result: {:?}", result),
        }
    }

    #[test]
    fn test_style_parser_background_color() {
        let result = run_parser(style_prop, "background_color: #fff;");
        match result {
            StyleProp::BackgroundColor(Some(color)) => {
                assert_eq!(color, Color::rgb(1.0, 1.0, 1.0));
            }
            _ => panic!("incorrect result: {:?}", result),
        }
    }

    #[test]
    fn test_style_parser_length() {
        let result = run_parser(style_prop, "width: 10;");
        match result {
            StyleProp::Width(val) => {
                assert_eq!(val, ui::Val::Px(10.0));
            }
            _ => panic!("incorrect result: {:?}", result),
        }

        let result = run_parser(style_prop, "width: 10px;");
        match result {
            StyleProp::Width(val) => {
                assert_eq!(val, ui::Val::Px(10.0));
            }
            _ => panic!("incorrect result: {:?}", result),
        }

        let result = run_parser(style_prop, "width: 10%;");
        match result {
            StyleProp::Width(val) => {
                assert_eq!(val, ui::Val::Percent(10.0));
            }
            _ => panic!("incorrect result: {:?}", result),
        }

        let result = run_parser(style_prop, "width: 10vh;");
        match result {
            StyleProp::Width(val) => {
                assert_eq!(val, ui::Val::Vh(10.0));
            }
            _ => panic!("incorrect result: {:?}", result),
        }

        let result = run_parser(style_prop, "width: auto;");
        match result {
            StyleProp::Width(val) => {
                assert_eq!(val, ui::Val::Auto);
            }
            _ => panic!("incorrect result: {:?}", result),
        }
    }

    #[test]
    fn test_style_list_parser_empty() {
        let result = run_parser(style_prop_list, "MAIN {}");
        assert_eq!(result.0, "MAIN");
        assert_eq!(result.1.props.len(), 0);
    }

    #[test]
    fn test_style_list_parser_basic() {
        let result = run_parser(
            style_prop_list,
            "MAIN {
                width: 10px;
                height: 10px;
             }",
        );
        assert_eq!(result.0, "MAIN");
        assert_eq!(result.1.props.len(), 2);
    }

    #[test]
    fn test_stylesheet_parser_empty() {
        let result = run_parser(stylesheet, "");
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_stylesheet_parser_single() {
        let result = run_parser(
            stylesheet,
            "MAIN {
                width: 10px;
                height: 10px;
             }",
        );
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_stylesheet_parser_with_selector() {
        let result = run_parser(
            stylesheet,
            "MAIN {
                width: 10px;
                height: 10px;

                .enabled {
                    width: 10px;
                    height: 10px;
                }
             }",
        );
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].1.props.len(), 2);
        assert_eq!(result[0].1.selectors.len(), 1);
    }

    #[test]
    fn test_stylesheet_parser_with_space() {
        let result = run_parser(
            stylesheet,
            "
            MAIN {
                width: 10px;
                height: 10px;
             }

             SIDE {
                width: 10px;
                height: 10px;
             }  ",
        );
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_stylesheet_comment() {
        let result = run_parser(
            stylesheet,
            "
            // Comment1
            MAIN {
                // Comment2
                width: 10px;

                // Comment3
                height: 10px;
             }

             // Comment4
             SIDE {
                width: 10px;
                height: 10px;
             }  ",
        );
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_stylesheet_parser_err_bad_prop() {
        let err = run_parser_err(
            stylesheet,
            "MAIN {
                bad: 10px;
                height: 10px;
             }",
        );
        assert!(err.contains("invalid property name: 'bad'"), "{err}");
    }

    #[test]
    fn test_stylesheet_parser_err_bad_value() {
        let err = run_parser_err(
            stylesheet,
            "MAIN {
                height: #fff;
             }",
        );
        assert!(err.contains("invalid property type: color"), "{err}");
    }

    #[test]
    fn test_stylesheet_parser_err_bad_value2() {
        let err = run_parser_err(
            stylesheet,
            "MAIN {
                color: #fff#;
             }",
        );
        assert!(err.contains("expected semicolon"), "{err}");
    }

    #[test]
    fn test_stylesheet_parser_err_bad_prop2() {
        let err = run_parser_err(style_prop, "colorr: fff;");
        assert!(err.contains("invalid property name:"), "{err}");

        let result = run_parser_err(style_prop_list_items, "colorr: fff;");
        assert!(result.contains("invalid property name:"), "{err}");
    }
}
