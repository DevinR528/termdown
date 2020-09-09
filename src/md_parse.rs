use ansi_term::{Colour, Style};
use pulldown_cmark::{CodeBlockKind, CowStr, Event, Tag};

// const OSC: &str = "\x1B[";
// const BEL: &str = "7";
// const SEP: &str = ";";

// write_osc(writer, &format!("8;;{}", destination))

fn link(text: &str, url: &str) -> String {
    // [OSC, "8", SEP, SEP, url, BEL, text, OSC, "8", SEP, SEP, BEL].join("")
    // format!("[{}]: {}8{}{}{}", text, OSC, SEP, SEP, url)
    format!("{} ({})", text, url)
}

// pub fn write_rule<W: std::io::Write>(writer: &mut W, length: usize) -> Result<()> {
//     let rule = "\u{2550}".repeat(length);
//     let style = Style::new().fg(Colour::Green);
//     write_styled(writer, &style, rule)
// }

use crate::{
    code_block::codeblock_ansi,
    error::{Error, Result},
};

#[derive(Clone, Debug)]
pub enum State<'a> {
    Link {
        style: Style,
        url: CowStr<'a>,
        title: Option<CowStr<'a>>,
    },
    Strong {
        style: Style,
    },
    Emphasis {
        style: Style,
    },
    Paragraph {
        style: Style,
    },
    Code {
        lang: Option<CowStr<'a>>,
        indented: bool,
    },
    List {
        style: Style,
        first: Option<u64>,
    },
    ListItem {
        style: Style,
    },
    Heading {
        style: Style,
        level: u32,
    },
}

// #[allow(unused)]
// #[derive(Clone, Debug)]
// pub enum StateText<'a> {
//     Link {
//         style: Style,
//         body: CowStr<'a>,
//     },
//     Text {
//         body: CowStr<'a>,
//     },
//     Strong {
//         style: Style,
//         body: CowStr<'a>,
//     },
//     Emphasis {
//         style: Style,
//         body: CowStr<'a>,
//     },
//     Paragraph {
//         style: Style,
//         body: CowStr<'a>,
//     },
//     Code {
//         lang: &'a Option<CowStr<'a>>,
//         body: CowStr<'a>,
//         indented: bool,
//     },
//     List {
//         style: Style,
//         body: CowStr<'a>,
//     },
//     ListItem {
//         style: Style,
//         body: CowStr<'a>,
//     },
// }

#[derive(Debug, Default)]
pub struct StateStack<'a> {
    stack: Vec<State<'a>>,
}

impl<'a> StateStack<'a> {
    pub fn push_tag(&mut self, tag: Tag<'a>) {
        match tag {
            Tag::CodeBlock(CodeBlockKind::Fenced(lang)) => {
                self.stack.push(State::Code {
                    indented: false,
                    lang: Some(lang.to_owned()),
                });
            }
            Tag::CodeBlock(CodeBlockKind::Indented) => self.stack.push(State::Code {
                indented: true,
                lang: None,
            }),
            Tag::Link(_type, url, title) => self.stack.push(State::Link {
                style: Style::new().fg(Colour::Green),
                url,
                title: if title.is_empty() { None } else { Some(title) },
            }),
            Tag::Strong => self.stack.push(State::Strong {
                style: Style::new().bold(),
            }),
            Tag::Emphasis => self.stack.push(State::Emphasis {
                style: Style::new().italic(),
            }),
            Tag::Paragraph => self.stack.push(State::Paragraph {
                style: Style::new(),
            }),
            Tag::List(first) => self.stack.push(State::List {
                first,
                style: Style::new(),
            }),
            Tag::Item => self.stack.push(State::ListItem {
                style: Style::new(),
            }),
            // TODO is there a way to render a different font size in terminal
            Tag::Heading(level) => self.stack.push(State::Heading {
                style: Style::new().bold(),
                level,
            }),
            ev => println!("{:?}", ev),
        }
    }

    pub fn pop_tag(&mut self, tag: Tag<'a>) -> Result<()> {
        match (tag, self.stack.pop()) {
            (Tag::Paragraph, Some(State::Paragraph { .. }))
            | (Tag::Emphasis, Some(State::Emphasis { .. }))
            | (Tag::Strong, Some(State::Strong { .. }))
            | (Tag::Link(_, _, _), Some(State::Link { .. }))
            | (Tag::List(_), Some(State::List { .. }))
            | (Tag::Item, Some(State::ListItem { .. }))
            | (Tag::CodeBlock(_), Some(State::Code { .. })) => Ok(()),
            (t, s) => {
                println!("tag: {:?} state: {:?}", t, s);
                Err(Error::Unmatched("Unmatched pair found on stack".into()))
            }
        }
    }

    pub fn write_text<W: std::io::Write>(
        &mut self,
        body: CowStr<'a>,
        writer: &mut W,
    ) -> Result<()> {
        match self.stack.last() {
            Some(state) => write_terminal(writer, body, state),
            None => panic!("Text event before Start event"),
        }
    }
}

/// Turn `pulldown_cmark::Event`s into ansi escaped text written to `writer`.
///
/// Returns the given `state` with the `event` applied (if an `Event::Start(_)` we append,
/// if `Event::End(_)` we pop and check the events match).
pub fn process_event<'a, W: std::io::Write>(
    writer: &mut W,
    event: Event<'a>,
    mut state: StateStack<'a>,
) -> Result<StateStack<'a>> {
    match event {
        // A list is always rendered with "\n\n" before the items
        Event::Start(tag) if matches!(tag, Tag::List(_)) => {
            writeln!(writer)?;
            writeln!(writer)?;
            state.push_tag(tag);
        }
        // An extra newline at the end of a list
        Event::End(tag) if matches!(tag, Tag::List(_)) => {
            writeln!(writer)?;
            state.pop_tag(tag)?;
        }
        Event::Start(tag) => state.push_tag(tag),
        Event::Text(text) => state.write_text(text, writer)?,
        Event::End(tag) => state.pop_tag(tag)?,
        // Code like `foo && bar`
        Event::Code(text) => write_styled(writer, &Style::new().fg(Colour::Yellow), text)?,
        ev => println!("{:?}", ev),
    }
    Ok(state)
}

pub fn write_terminal<W: std::io::Write>(
    writer: &mut W,
    body: CowStr<'_>,
    state: &State<'_>,
) -> Result<()> {
    match state {
        State::Strong { style } => write_styled(writer, style, body),
        State::Emphasis { style } => write_styled(writer, style, body),
        State::Paragraph { style } => write_styled(writer, style, body),
        State::List { .. } => Ok(()), // do nothing this is handled by `process_event`
        State::ListItem { style } => write_styled(writer, style, format!(" * {}\n", body)),
        // TODO use style and check for title
        // since ansi_term has no link ability we write our own escape codes
        State::Link {
            style,
            url,
            title: _,
        } => write_styled(writer, style, link(&body, url)).map_err(Into::into),
        State::Code { lang, indented: _ } => {
            if let Some(lang) = lang {
                codeblock_ansi(&body, lang, writer).map_err(Into::into)
            } else {
                // TODO some sensible default?
                codeblock_ansi(&body, "js", writer).map_err(Into::into)
            }
        }
        _ => Ok(()),
    }
}

fn write_styled<W: std::io::Write, S: AsRef<str>>(
    writer: &mut W,
    style: &Style,
    text: S,
) -> Result<()> {
    write!(writer, "{}", style.to_owned().paint(text.as_ref())).map_err(Into::into)
}
