use ansi_term::{Colour, Style};
use pulldown_cmark::{Event, LinkType, Options, Parser, Tag};
use syntect::{
    highlighting::{HighlightIterator, Highlighter, Theme},
    util::LinesWithEndings,
};

use crate::error::{Error, Result};

pub fn proccess_event<W: std::io::Write>(
    writer: &mut W,
    event: Event<'_>,
    theme: &Theme,
) -> Result<()> {
    Ok(())
}
