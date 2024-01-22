use winnow::{
    ascii::space0,
    combinator::{alt, opt, preceded, repeat, separated},
    stream::AsChar,
    token::{one_of, take_while},
    PResult, Parser,
};

use crate::Selector;

enum SelectorToken<'s> {
    Class(&'s str),
    Hover,
    FirstChild,
    LastChild,
    Focus,
    FocusWithin,
    FocusVisible,
}

fn parent(input: &mut &str) -> PResult<()> {
    (space0, '>', space0).void().parse_next(input)
}

fn class_name<'s>(input: &mut &'s str) -> PResult<SelectorToken<'s>> {
    preceded(
        '.',
        (
            one_of(AsChar::is_alpha),
            take_while(0.., (AsChar::is_alphanum, '-', '_')),
        ),
    )
    .recognize()
    .map(|cls: &str| SelectorToken::Class(&cls[1..]))
    .parse_next(input)
}

fn hover<'s>(input: &mut &'s str) -> PResult<SelectorToken<'s>> {
    ":hover"
        .recognize()
        .map(|_| SelectorToken::Hover)
        .parse_next(input)
}

fn focus<'s>(input: &mut &'s str) -> PResult<SelectorToken<'s>> {
    ":focus"
        .recognize()
        .map(|_| SelectorToken::Focus)
        .parse_next(input)
}

fn focus_within<'s>(input: &mut &'s str) -> PResult<SelectorToken<'s>> {
    ":focus-within"
        .recognize()
        .map(|_| SelectorToken::FocusWithin)
        .parse_next(input)
}

fn focus_visible<'s>(input: &mut &'s str) -> PResult<SelectorToken<'s>> {
    ":focus-visible"
        .recognize()
        .map(|_| SelectorToken::FocusVisible)
        .parse_next(input)
}

fn first_child<'s>(input: &mut &'s str) -> PResult<SelectorToken<'s>> {
    ":first-child"
        .recognize()
        .map(|_| SelectorToken::FirstChild)
        .parse_next(input)
}

fn last_child<'s>(input: &mut &'s str) -> PResult<SelectorToken<'s>> {
    ":last-child"
        .recognize()
        .map(|_| SelectorToken::LastChild)
        .parse_next(input)
}

fn simple_selector<'s>(input: &mut &'s str) -> PResult<(Option<char>, Vec<SelectorToken<'s>>)> {
    (
        opt(alt(('*', '&'))),
        repeat(
            0..,
            alt((
                class_name,
                hover,
                first_child,
                last_child,
                focus,
                focus_within,
                focus_visible,
            )),
        ),
    )
        .parse_next(input)
}

fn combo_selector(input: &mut &str) -> PResult<Box<Selector>> {
    let mut sel = Box::new(Selector::Accept);
    let (prefix, classes) = simple_selector.parse_next(input)?;
    for tok in classes {
        match tok {
            SelectorToken::Class(cls) => {
                sel = Box::new(Selector::Class(cls.into(), sel));
            }
            SelectorToken::Hover => {
                sel = Box::new(Selector::Hover(sel));
            }
            SelectorToken::FirstChild => {
                sel = Box::new(Selector::FirstChild(sel));
            }
            SelectorToken::LastChild => {
                sel = Box::new(Selector::LastChild(sel));
            }
            SelectorToken::Focus => {
                sel = Box::new(Selector::Focus(sel));
            }
            SelectorToken::FocusWithin => {
                sel = Box::new(Selector::FocusWithin(sel));
            }
            SelectorToken::FocusVisible => {
                sel = Box::new(Selector::FocusVisible(sel));
            }
        }
    }
    if let Some(ch) = prefix {
        if ch == '&' {
            sel = Box::new(Selector::Current(sel));
        }
    }
    Ok(sel)
}

fn desc_selector(input: &mut &str) -> PResult<Box<Selector>> {
    let mut sel = combo_selector.parse_next(input)?;
    while parent.parse_next(input).is_ok() {
        sel = Box::new(Selector::Parent(sel));
        let (prefix, classes) = simple_selector.parse_next(input)?;
        for tok in classes {
            match tok {
                SelectorToken::Class(cls) => {
                    sel = Box::new(Selector::Class(cls.into(), sel));
                }
                SelectorToken::Hover => {
                    sel = Box::new(Selector::Hover(sel));
                }
                SelectorToken::FirstChild => {
                    sel = Box::new(Selector::FirstChild(sel));
                }
                SelectorToken::LastChild => {
                    sel = Box::new(Selector::LastChild(sel));
                }
                SelectorToken::Focus => {
                    sel = Box::new(Selector::Focus(sel));
                }
                SelectorToken::FocusWithin => {
                    sel = Box::new(Selector::FocusWithin(sel));
                }
                SelectorToken::FocusVisible => {
                    sel = Box::new(Selector::FocusVisible(sel));
                }
            }
        }
        if let Some(ch) = prefix {
            if ch == '&' {
                sel = Box::new(Selector::Current(sel));
            }
        }
    }

    Ok(sel)
}

fn either(input: &mut &str) -> PResult<Box<Selector>> {
    separated(1.., desc_selector, (space0, ',', space0))
        .map(|mut items: Vec<Box<Selector>>| {
            if items.len() == 1 {
                items.pop().unwrap()
            } else {
                Box::new(Selector::Either(items))
            }
        })
        .parse_next(input)
}

pub fn selector_parser(input: &mut &str) -> PResult<Box<Selector>> {
    either.parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_current() {
        assert_eq!(
            "&".parse::<Selector>().unwrap(),
            Selector::Current(Box::new(Selector::Accept))
        );
    }

    #[test]
    fn test_serialize() {
        assert_eq!(
            Selector::Current(Box::new(Selector::Accept)).to_string(),
            "&",
        );
        assert_eq!(
            Selector::Class("x".into(), Box::new(Selector::Accept)).to_string(),
            ".x",
        );
        assert_eq!(
            ".foo > &.bar".parse::<Selector>().unwrap().to_string(),
            ".foo > &.bar",
        );
        assert_eq!(
            ".foo > .bar.baz".parse::<Selector>().unwrap().to_string(),
            ".foo > .bar.baz",
        );
        assert_eq!(
            ".foo > * > .bar".parse::<Selector>().unwrap().to_string(),
            ".foo > * > .bar",
        );
        assert_eq!(
            ".foo > &.bar.baz".parse::<Selector>().unwrap().to_string(),
            ".foo > &.bar.baz",
        );
        assert_eq!(
            ".a.b.c > .d.e.f > &.g.h.i"
                .parse::<Selector>()
                .unwrap()
                .to_string(),
            ".a.b.c > .d.e.f > &.g.h.i",
        );
        assert_eq!(
            ".foo, .bar".parse::<Selector>().unwrap().to_string(),
            ".foo, .bar",
        );
    }

    #[test]
    fn test_parse_current_class() {
        assert_eq!(
            "&.foo".parse::<Selector>().unwrap(),
            Selector::Current(Box::new(Selector::Class(
                "foo".into(),
                Box::new(Selector::Accept)
            )))
        );
    }

    #[test]
    fn test_parse_class() {
        assert_eq!(
            ".foo".parse::<Selector>().unwrap(),
            Selector::Class("foo".into(), Box::new(Selector::Accept))
        );
    }

    #[test]
    fn test_parse_hover() {
        assert_eq!(
            ":hover".parse::<Selector>().unwrap(),
            Selector::Hover(Box::new(Selector::Accept))
        );
        assert_eq!(
            ".foo:hover".parse::<Selector>().unwrap(),
            Selector::Hover(Box::new(Selector::Class(
                "foo".into(),
                Box::new(Selector::Accept)
            )))
        );
    }

    #[test]
    fn test_parse_first_last_child() {
        assert_eq!(
            ":first-child".parse::<Selector>().unwrap(),
            Selector::FirstChild(Box::new(Selector::Accept))
        );
        assert_eq!(
            ".foo:first-child".parse::<Selector>().unwrap(),
            Selector::FirstChild(Box::new(Selector::Class(
                "foo".into(),
                Box::new(Selector::Accept)
            )))
        );
        assert_eq!(
            ":last-child".parse::<Selector>().unwrap(),
            Selector::LastChild(Box::new(Selector::Accept))
        );
        assert_eq!(
            ".foo:last-child".parse::<Selector>().unwrap(),
            Selector::LastChild(Box::new(Selector::Class(
                "foo".into(),
                Box::new(Selector::Accept)
            )))
        );
    }

    #[test]
    fn test_parse_parent() {
        assert_eq!(
            "&.foo > .bar".parse::<Selector>().unwrap(),
            Selector::Class(
                "bar".into(),
                Box::new(Selector::Parent(Box::new(Selector::Current(Box::new(
                    Selector::Class("foo".into(), Box::new(Selector::Accept))
                )))))
            )
        );

        assert_eq!(
            ".foo > &.bar".parse::<Selector>().unwrap(),
            Selector::Current(Box::new(Selector::Class(
                "bar".into(),
                Box::new(Selector::Parent(Box::new(Selector::Class(
                    "foo".into(),
                    Box::new(Selector::Accept)
                ))))
            )))
        );
    }

    #[test]
    fn test_either() {
        assert_eq!(
            "&.foo, .bar".parse::<Selector>().unwrap(),
            Selector::Either(vec!(
                Box::new(Selector::Current(Box::new(Selector::Class(
                    "foo".into(),
                    Box::new(Selector::Accept)
                )))),
                Box::new(Selector::Class("bar".into(), Box::new(Selector::Accept)))
            ))
        );
    }
}
