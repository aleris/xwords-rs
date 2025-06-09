use crate::crossword::{Crossword, Direction};
use std::fmt;

/// Formats a Crossword into Across Puzzle V2 text file format.
/// See https://www.litsoft.com/across/docs/AcrossTextFormat.pdf
#[derive(PartialEq, Eq, Debug, Hash, Clone)]
pub struct AcrossFileFormat {
    pub(crate) crossword: Crossword,
    pub(crate) title: String,
    pub(crate) author: String,
    pub(crate) copyright: String,
}

impl AcrossFileFormat {
    pub fn new(crossword: Crossword, title: String, author: String, copyright: String) -> Self {
        AcrossFileFormat {
            crossword,
            title,
            author,
            copyright,
        }
    }

    fn indent(s: &str, spaces: usize) -> String {
        let indent = " ".repeat(spaces);
        s.lines()
            .map(|line| format!("{}{}", indent, line))
            .collect::<Vec<String>>()
            .join("\n")
    }
}

impl fmt::Display for AcrossFileFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent_spaces = 2;
        write!(
            f,
            "<ACROSS PUZZLE V2>
<TITLE>
{}
<AUTHOR>
{}
<COPYRIGHT>
{}
<SIZE>
{}
<GRID>
{}
<ACROSS>
{}
<DOWN>
{}",
            Self::indent(self.title.as_str(), indent_spaces),
            Self::indent(self.author.as_str(), indent_spaces),
            Self::indent(self.copyright.as_str(), indent_spaces),
            Self::indent(&format!("{}x{}", self.crossword.width, self.crossword.height), indent_spaces),
            Self::indent(&format!("{}", self.crossword), indent_spaces),
            Self::indent(
                &self.crossword.words(Direction::Across).join("\n"),
                indent_spaces
            ),
            Self::indent(
                &self.crossword.words(Direction::Down).join("\n"),
                indent_spaces
            ),
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::crossword::Crossword;

    #[test]
    fn format_works() {
        let c = Crossword::parse(String::from(
            "
SIAM
N.EM
RYAL
",
        ));
        let a = super::AcrossFileFormat::new(
            c.unwrap(),
            String::from("title"),
            String::from("author"),
            String::from("copyright"),
        );
        assert_eq!(
            format!("{}", a),
            "<ACROSS PUZZLE V2>
<TITLE>
  title
<AUTHOR>
  author
<COPYRIGHT>
  copyright
<SIZE>
  4x3
<GRID>
  SIAM
  N.EM
  RYAL
<ACROSS>
  SIAM
  EM
  RYAL
<DOWN>
  SNR
  AEA
  MML"
        );
    }
}
