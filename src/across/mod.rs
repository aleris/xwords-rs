use crate::crossword::{Crossword, Direction};
use std::fmt;

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
}

impl fmt::Display for AcrossFileFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
{}x{}
<GRID>
{}
<ACROSS>
{}
<DOWN>
{}",
            self.title,
            self.author,
            self.copyright,
            self.crossword.width,
            self.crossword.height,
            self.crossword,
            self.crossword.words(Direction::Across).join("\n"),
            self.crossword.words(Direction::Down).join("\n"),
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
