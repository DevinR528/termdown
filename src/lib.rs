use pulldown_cmark::{Options, Parser};
use syntect::{
    easy::HighlightLines,
    highlighting::{FontStyle, Style, Theme, ThemeSet},
    parsing::SyntaxSet,
    util::LinesWithEndings,
};

mod code_block;
mod error;
mod md_parse;

use code_block::{get_theme, write_as_ansi};
pub use error::{Error, Result};

pub fn markdown_terminal(input: &str) -> Result<String> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_STRIKETHROUGH);

    let parser = Parser::new_ext(&input, options);
    let mut escaped = vec![];
    for event in parser {
        md_parse::proccess_event(&mut escaped, event, get_theme())?;
    }

    // clear formatting
    escaped.extend(b"\x1b[0m");

    String::from_utf8(escaped).map_err(Into::into)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        println!(
            "{}",
            super::markdown_terminal(&std::fs::read_to_string("./test_data/code.md").unwrap(),)
                .unwrap()
        );
        println!(
            "{}",
            super::markdown_terminal("```rust\nfn heap_size_of_children(&self) -> usize;\n```")
                .unwrap()
        );
    }
}
