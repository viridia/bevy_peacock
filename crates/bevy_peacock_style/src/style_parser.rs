use bevy::{render::color::Color, ui};
use winnow::{
    ascii::{float, space1},
    ascii::{multispace0, space0},
    combinator::{alt, cut_err, opt, preceded, repeat, separated, terminated},
    error::StrContext,
    stream::AsChar,
    token::{one_of, take_while},
    PResult, Parser,
};

use crate::{style::SelectorEntry, Selector, StyleProp, StylePropList};

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum StyleParsingError {
    InvalidProperty,
    InvalidPropertyType(String),
}

impl std::fmt::Display for StyleParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StyleParsingError::InvalidProperty => write!(f, "invalid style property"),
            StyleParsingError::InvalidPropertyType(ty) => {
                write!(f, "invalid property type: {}", ty)
            }
        }
    }
}

impl std::error::Error for StyleParsingError {}

enum PropValue<'s> {
    Ident(&'s str),
    Number(f32),
    String(&'s str),
    Length(ui::Val),
    List(Vec<PropValue<'s>>),
    Color(Color),
}

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

fn prop_name<'s>(input: &mut &'s str) -> PResult<&'s str> {
    (
        one_of(AsChar::is_alpha),
        take_while(0.., (AsChar::is_alphanum, '_')),
    )
        .recognize()
        .parse_next(input)
}

fn ident<'s>(input: &mut &'s str) -> PResult<PropValue<'s>> {
    (
        one_of(AsChar::is_alpha),
        take_while(0.., (AsChar::is_alphanum, '_')),
    )
        .recognize()
        .map(|cls: &str| PropValue::Ident(cls))
        .parse_next(input)
}

fn color_arg_sep(input: &mut &str) -> PResult<()> {
    (space0, opt((',', space0))).void().parse_next(input)
}

fn alpha_sep(input: &mut &str) -> PResult<()> {
    (space0, opt((one_of([',', '/']), space0)))
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
        space0,
        '(',
        cut_err(space0),
        terminated(f32_arg, color_arg_sep),
        terminated(f32_arg, color_arg_sep),
        f32_arg,
        opt(preceded(alpha_sep, f32_arg)),
        space0,
        ')',
    )
        .map(|(f, _, _, _, a1, a2, a3, a4, _, _)| match f {
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

fn style_prop(input: &mut &str) -> PResult<StyleProp> {
    (
        prop_name,
        space0,
        ':',
        cut_err(space0).context(StrContext::Label("property value")),
        alt((color, ident, length_list, length)),
    )
        .try_map(|(name, _, _, _, value)| match name {
            "background_color" => Ok(StyleProp::BackgroundColor(value.coerce()?)),
            "border_color" => Ok(StyleProp::BackgroundColor(value.coerce().unwrap())),
            "color" => Ok(StyleProp::Color(value.coerce().unwrap())),
            "width" => Ok(StyleProp::Width(value.coerce().unwrap())),
            "height" => Ok(StyleProp::Height(value.coerce().unwrap())),
            "min_width" => Ok(StyleProp::MinWidth(value.coerce().unwrap())),
            "min_height" => Ok(StyleProp::MinHeight(value.coerce().unwrap())),
            "max_width" => Ok(StyleProp::MaxWidth(value.coerce().unwrap())),
            "max_height" => Ok(StyleProp::MaxHeight(value.coerce().unwrap())),
            _ => Err(StyleParsingError::InvalidProperty),
        })
        .parse_next(input)
}

pub(crate) fn style_prop_parser(input: &mut &str) -> PResult<StyleProp> {
    style_prop.parse_next(input)
}

fn style_prop_list_items(input: &mut &str) -> PResult<Vec<StylePropOrSelector>> {
    repeat(
        0..,
        terminated(style_prop, (space0, ';', multispace0))
            .map(|s| StylePropOrSelector::StyleProp(s)),
    )
    .parse_next(input)
}

fn style_prop_list(input: &mut &str) -> PResult<StylePropList> {
    ('{', cut_err(multispace0), style_prop_list_items, '}')
        .map(|(_, _, mut items, _)| {
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
            StylePropList { props, selectors }
        })
        .parse_next(input)
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

#[cfg(test)]
mod tests {
    use winnow::error::ContextError;

    use super::*;

    fn run_parser<'s, T>(mut parser: impl Parser<&'s str, T, ContextError>, input: &'s str) -> T {
        let result = parser.parse(input);
        match result {
            Ok(val) => val,
            Err(e) => panic!("{}", e.to_string()),
        }
    }

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
        let err = run_parser_err(style_prop, "foo: #fff");
        assert_eq!(err, "foo: #fff\n^\ninvalid style property");
    }

    #[test]
    fn test_style_invalid_prop_value() {
        let err = run_parser_err(style_prop, "background_color: 1");
        assert_eq!(err, "background_color: 1\n^\ninvalid property type: number");
    }

    #[test]
    fn test_parse_color() {
        let result = run_parser(style_prop, "background_color: #fff");
        match result {
            StyleProp::BackgroundColor(Some(color)) => {
                assert_eq!(color, Color::rgb(1.0, 1.0, 1.0));
            }
            _ => panic!("incorrect result: {:?}", result),
        }

        let result = run_parser(style_prop, "background_color: rgb(1, 1, 1)");
        match result {
            StyleProp::BackgroundColor(Some(color)) => {
                assert_eq!(color, Color::rgb(1.0, 1.0, 1.0));
            }
            _ => panic!("incorrect result: {:?}", result),
        }
    }

    #[test]
    fn test_style_parser_background_color() {
        let result = run_parser(style_prop, "background_color: #fff");
        match result {
            StyleProp::BackgroundColor(Some(color)) => {
                assert_eq!(color, Color::rgb(1.0, 1.0, 1.0));
            }
            _ => panic!("incorrect result: {:?}", result),
        }
    }

    #[test]
    fn test_style_parser_length() {
        let result = run_parser(style_prop, "width: 10");
        match result {
            StyleProp::Width(val) => {
                assert_eq!(val, ui::Val::Px(10.0));
            }
            _ => panic!("incorrect result: {:?}", result),
        }

        let result = run_parser(style_prop, "width: 10px");
        match result {
            StyleProp::Width(val) => {
                assert_eq!(val, ui::Val::Px(10.0));
            }
            _ => panic!("incorrect result: {:?}", result),
        }

        let result = run_parser(style_prop, "width: 10%");
        match result {
            StyleProp::Width(val) => {
                assert_eq!(val, ui::Val::Percent(10.0));
            }
            _ => panic!("incorrect result: {:?}", result),
        }

        let result = run_parser(style_prop, "width: 10vh");
        match result {
            StyleProp::Width(val) => {
                assert_eq!(val, ui::Val::Vh(10.0));
            }
            _ => panic!("incorrect result: {:?}", result),
        }

        let result = run_parser(style_prop, "width: auto");
        match result {
            StyleProp::Width(val) => {
                assert_eq!(val, ui::Val::Auto);
            }
            _ => panic!("incorrect result: {:?}", result),
        }
    }

    #[test]
    fn test_style_list_parser() {
        let result = run_parser(style_prop_list, "{}");
        assert_eq!(result.props.len(), 0);

        let result = run_parser(
            style_prop_list,
            "{
                width: 10px;
                height: 10px;
             }",
        );
        assert_eq!(result.props.len(), 2);
    }
}
