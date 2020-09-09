use ansi_term::Colour;
use std::io::Write;
use syntect::{
    easy::HighlightLines,
    highlighting::{FontStyle, Style, Theme, ThemeSet},
    parsing::{SyntaxReference, SyntaxSet},
    util::LinesWithEndings,
};

use std::sync::Once;

use crate::error::Result;

static INIT_THEME: Once = Once::new();
static INIT_SYNTAX: Once = Once::new();
static mut THEME: Option<Theme> = None;
static mut SYNTAX: Option<SyntaxSet> = None;

/// We avoid loading the entire ThemeSet binary every time we want to convert some code
/// to ansi escaped text.
///
/// We use a static mut, `THEME` to hold a syntax set that will define how the tokens are colored.
pub(crate) fn get_theme<'a>() -> &'a Theme {
    unsafe {
        INIT_THEME.call_once(|| {
            let mut ts = ThemeSet::load_defaults();
            THEME = ts.themes.remove("Solarized (dark)");
        });
        THEME.as_ref().unwrap()
    }
}

/// The `SyntaxSet` defines the tokens that are possible.
///
/// A static mut is used to avoid parsing and storing the binary file and resulting struct every
/// time `markdown_terminal` is called.
pub(crate) fn get_syntax<'a>(token: &str) -> &'a SyntaxReference {
    unsafe {
        INIT_SYNTAX.call_once(|| {
            SYNTAX = Some(SyntaxSet::load_defaults_newlines());
        });
        SYNTAX
            .as_ref()
            .unwrap()
            .find_syntax_by_token(token)
            .unwrap_or_else(|| SYNTAX.as_ref().unwrap().find_syntax_plain_text())
    }
}

// TODO re-write
/// Write regions as ANSI 8-bit coloured text.
///
/// We use this function to simplify syntax highlighting to 8-bit ANSI values
/// which every theme provides.  Contrary to 24 bit colours this gives us a good
/// guarantee that highlighting works with any terminal colour theme, whether
/// light or dark, and saves us all the hassle of mismatching colours.
///
/// We assume Solarized colours here: Solarized cleanly maps to 8-bit ANSI
/// colours so we can safely map its RGB colour values back to ANSI colours.  We
/// do so for all accent colours, but leave "base*" colours alone: Base colours
/// change depending on light or dark Solarized; to address both light and dark
/// backgrounds we must map all base colours to the default terminal colours.
///
/// Furthermore we completely ignore any background colour settings, to avoid
/// conflicts with the terminal colour themes.
pub(crate) fn write_as_ansi<'a, W: Write, I: Iterator<Item = (Style, &'a str)>>(
    writer: &mut W,
    tokens: I,
) -> Result<()> {
    for (style, text) in tokens {
        let rgb = {
            let fg = style.foreground;
            (fg.r, fg.g, fg.b)
        };
        let mut ansi_style = ansi_term::Style::new();
        match rgb {
            // base03, base02, base01, base00, base0, base1, base2, and base3
            (0x00, 0x2b, 0x36)
            | (0x07, 0x36, 0x42)
            | (0x58, 0x6e, 0x75)
            | (0x65, 0x7b, 0x83)
            | (0x83, 0x94, 0x96)
            | (0x93, 0xa1, 0xa1)
            | (0xee, 0xe8, 0xd5)
            | (0xfd, 0xf6, 0xe3) => ansi_style.foreground = None,
            (0xb5, 0x89, 0x00) => ansi_style.foreground = Some(Colour::Yellow),
            (0xcb, 0x4b, 0x16) => ansi_style.foreground = Some(Colour::Fixed(9)), // Bright red
            (0xdc, 0x32, 0x2f) => ansi_style.foreground = Some(Colour::Red),
            (0xd3, 0x36, 0x82) => ansi_style.foreground = Some(Colour::Purple),
            (0x6c, 0x71, 0xc4) => ansi_style.foreground = Some(Colour::Fixed(13)), // Bright purple
            (0x26, 0x8b, 0xd2) => ansi_style.foreground = Some(Colour::Blue),
            (0x2a, 0xa1, 0x98) => ansi_style.foreground = Some(Colour::Cyan),
            (0x85, 0x99, 0x00) => ansi_style.foreground = Some(Colour::Green),
            (r, g, b) => unreachable!("Unexpected RGB colour: #{:2>0x}{:2>0x}{:2>0x}", r, g, b),
        };
        let font = style.font_style;
        ansi_style.is_bold = font.contains(FontStyle::BOLD);
        ansi_style.is_italic = font.contains(FontStyle::ITALIC);
        ansi_style.is_underline = font.contains(FontStyle::UNDERLINE);

        write!(writer, "{}", ansi_style.paint(text))?;
    }
    Ok(())
}

pub fn codeblock_ansi<W: Write>(input: &str, lang: &str, writer: &mut W) -> Result<()> {
    let ps = SyntaxSet::load_defaults_newlines();
    let syntax = get_syntax(lang);

    let mut h = HighlightLines::new(&syntax, get_theme());

    for line in LinesWithEndings::from(input) {
        let ranges: Vec<(Style, &str)> = h.highlight(line, &ps);
        write_as_ansi(writer, ranges.into_iter()).unwrap();
    }

    // clear formatting
    write!(writer, "\x1b[0m")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    fn block(input: &str, lang: &str) -> String {
        let mut out = vec![];
        super::codeblock_ansi(input, lang, &mut out).unwrap();
        String::from_utf8(out).unwrap()
    }
    #[test]
    fn code_blocks() {
        println!(
            "{}",
            block(
                &std::fs::read_to_string("./test_data/code.md").unwrap(),
                "rust",
            )
        );
        println!(
            "{}",
            block("fn heap_size_of_children(&self) -> usize;", "rs")
        );
    }
}
