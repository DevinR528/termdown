use pulldown_cmark::{Options, Parser};

mod code_block;
mod error;
mod md_parse;

pub use error::{Error, Result};
pub use md_parse::StateStack;

pub fn markdown_terminal(input: &str) -> Result<String> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_STRIKETHROUGH);

    let parser = Parser::new_ext(&input, options);
    let mut escaped = vec![];
    let mut state = StateStack::default();
    for event in parser {
        state = md_parse::process_event(&mut escaped, event, state)?;
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
    }

    #[test]
    fn second() {
        println!(
            "{}",
            super::markdown_terminal("```rust\nfn heap_size_of_children(&self) -> usize;\n```")
                .unwrap()
        );
    }

    #[test]
    fn third() {
        println!(
            "{}",
            super::markdown_terminal(&std::fs::read_to_string("./test_data/links.md").unwrap(),)
                .unwrap()
        );
    }
}
