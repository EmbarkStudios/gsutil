use std::{
    borrow::Cow,
    fmt::{Debug, Display},
    io::IsTerminal,
};

use nu_ansi_term::{AnsiGenericString, Color, Style};

/// Environment variables that determine whether ANSI colors should be used.
///
/// Color is used if the output stream [is a terminal][IsTerminal] and the
/// [`NO_COLOR`](https://no-color.org) environment variable is unset or empty.
#[derive(Debug, Copy, Clone)]
pub struct ColorCtx {
    no_color: bool,
    stdout_isatty: bool,
    stderr_isatty: bool,
}

/// Result of [`ColorCtx::paint`], which is a [`nu_ansi_term::AnsiGenericString`] if color should
/// be used or a plain string otherwise.
#[derive(PartialEq, Debug, Clone)]
pub enum Painting<'a, S: ToOwned + ?Sized>
where
    <S as ToOwned>::Owned: Debug,
{
    Plain(Cow<'a, S>),
    Styled(AnsiGenericString<'a, S>),
}

impl<'a, S: 'a + ToOwned + ?Sized> Display for Painting<'a, S>
where
    <S as ToOwned>::Owned: Debug,
    Cow<'a, S>: Display,
    AnsiGenericString<'a, S>: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Painting::Plain(inner) => inner.fmt(f),
            Painting::Styled(inner) => inner.fmt(f),
        }
    }
}

impl ColorCtx {
    /// Reads the environment into a `ColorCtx`.
    ///
    /// You can call this once at the start of your program.
    pub fn from_env() -> Self {
        Self {
            no_color: std::env::var_os("NO_COLOR").is_some_and(|x| !x.is_empty()),
            stdout_isatty: std::io::stdout().is_terminal(),
            stderr_isatty: std::io::stderr().is_terminal(),
        }
    }

    /// Paints a string destined for stdout.
    ///
    /// Analogous to `p.paint(input)`, but produces a plain string if color should not be used for
    /// stdout.
    pub fn paint<'a, P: Paint, I, S: 'a + ToOwned + ?Sized>(
        &self,
        p: P,
        input: I,
    ) -> Painting<'a, S>
    where
        I: Into<Cow<'a, S>>,
        <S as ToOwned>::Owned: Debug,
    {
        if self.no_color || !self.stdout_isatty {
            Painting::Plain(input.into())
        } else {
            Painting::Styled(p.paint(input))
        }
    }

    /// Paints a string destined for stderr.
    ///
    /// Analogous to `p.paint(input)`, but produces a plain string if color should not be used for
    /// stderr.
    pub fn paint_err<'a, P: Paint, I, S: 'a + ToOwned + ?Sized>(
        &self,
        p: P,
        input: I,
    ) -> Painting<'a, S>
    where
        I: Into<Cow<'a, S>>,
        <S as ToOwned>::Owned: Debug,
    {
        if self.no_color || !self.stderr_isatty {
            Painting::Plain(input.into())
        } else {
            Painting::Styled(p.paint(input))
        }
    }
}

/// Common supertype for `nu_ansi_term` methods [`Style::paint`] and [`Color::paint`].
pub trait Paint {
    fn paint<'a, I, S: 'a + ToOwned + ?Sized>(self, input: I) -> AnsiGenericString<'a, S>
    where
        I: Into<Cow<'a, S>>,
        <S as ToOwned>::Owned: Debug;
}

impl Paint for Style {
    fn paint<'a, I, S: 'a + ToOwned + ?Sized>(self, input: I) -> AnsiGenericString<'a, S>
    where
        I: Into<Cow<'a, S>>,
        <S as ToOwned>::Owned: Debug,
    {
        self.paint(input)
    }
}

impl Paint for Color {
    fn paint<'a, I, S: 'a + ToOwned + ?Sized>(self, input: I) -> AnsiGenericString<'a, S>
    where
        I: Into<Cow<'a, S>>,
        <S as ToOwned>::Owned: Debug,
    {
        self.paint(input)
    }
}
