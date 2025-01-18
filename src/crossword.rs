/*!
Core types to represent a crossword puzzle.
*/

use crate::parse::{parse_word_boundaries, WordBoundary};
use std::{fmt, fs, hash::Hash};
use std::path::Path;

/// The underlying representation of a crossword puzzle.
/// All the contents are stored in a string, and the dimensions of the grid are stored explicitly.
///
/// The contents use ACROSS PUZZLE V2 format to represent the grid.
/// See [the specs PDF](http://www.litsoft.com/across/docs/AcrossTextFormat.pdf)
/// for more information.
/// In the contents, `.` or `:` represents a black square,
/// and `X` represents a solution letter.
#[derive(PartialEq, Eq, Debug, Hash, Clone)]
pub struct Crossword {
    pub(crate) contents: String,
    pub(crate) width: usize,
    pub(crate) height: usize,
}

impl Crossword {
    /// Parses a crossword from a file.
    /// Err is returned of the file cannot be read or the contents cannot be parsed.
    pub fn parse_from_file<P>(file_path: P) -> Result<Crossword, String> where P: AsRef<Path>, {
        let name = file_path.as_ref().display().to_string();
        let contents = fs::read_to_string(file_path)
            .expect(format!("Could not read file {}", name).as_str());
        Crossword::parse(contents)
    }

    /// Parses a crossword from a string.
    /// Err is returned if the contents cannot be parsed.
    pub fn parse(contents: String) -> Result<Crossword, String> {
        let grid: Vec<Vec<char>> = contents
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| line.chars().collect())
            .collect();
        let height = grid.len();
        let width = grid[0].len();
        let contents = Crossword::clean(&contents);
        Ok(Crossword {
            contents,
            width,
            height,
        })
    }

    fn clean(contents: &String) -> String {
        let cleaned: String = contents.chars()
            .filter(|c| *c != '\n')
            .collect();
        cleaned
            .replace("X", " ") // internally use space for blank squares
    }

    /// Returns all words with at least two letters
    /// in the crossword for a given direction as a Vec of strings
    pub fn words(&self, direction: Direction) -> Vec<String> {
        let word_boundaries = parse_word_boundaries(self);
        word_boundaries
            .iter()
            .filter(|wb| wb.direction == direction)
            .map(|wb| {
                let iter = WordIterator::new(self, wb);
                iter.collect()
            })
            .filter(|word: &String| word.len() >= 2)
            .collect()
    }
}

/// An `Iterator<char>` that correctly traversing a Crossword, accounting for direction.
///
/// The length of the word is stored in the `word_boundary`.
#[derive(Clone, Debug)]
pub struct WordIterator<'s> {
    crossword: &'s Crossword,
    pub word_boundary: &'s WordBoundary,
    index: usize,
}

impl<'s> WordIterator<'s> {
    pub fn new(crossword: &'s Crossword, word_boundary: &'s WordBoundary) -> WordIterator<'s> {
        WordIterator {
            crossword,
            word_boundary,
            index: 0,
        }
    }
}

impl<'s> fmt::Display for WordIterator<'s> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for c in self.clone() {
            write!(f, "{}", c)?;
        }
        Ok(())
    }
}

impl<'s> Iterator for WordIterator<'s> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.word_boundary.length {
            return None;
        }

        match self.word_boundary.direction {
            Direction::Across => {
                let char_index = self.word_boundary.start_row * self.crossword.width
                    + self.word_boundary.start_col
                    + self.index;
                let result = self.crossword.contents.as_bytes()[char_index] as char;
                self.index += 1;
                Some(result)
            }
            Direction::Down => {
                let char_index = (self.word_boundary.start_row + self.index) * self.crossword.width
                    + self.word_boundary.start_col;
                let result = self.crossword.contents.as_bytes()[char_index] as char;
                self.index += 1;
                Some(result)
            }
        }
    }
}

impl Hash for WordIterator<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for c in (*self).clone() {
            c.hash(state);
        }
    }
}

impl PartialEq for WordIterator<'_> {
    fn eq(&self, other: &Self) -> bool {
        if self.word_boundary.length != other.word_boundary.length {
            return false;
        }

        self.clone().zip(other.clone()).all(|(a, b)| a == b)
    }
}

impl Eq for WordIterator<'_> {}

impl fmt::Display for Crossword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in 0..self.height {
            for col in 0..self.width {
                let char = self.contents.as_bytes()[row * self.width + col] as char;
                // for unsolved cells, put back standard across file format X
                // for an omitted solution letter instead of space which is used internally
                let char = if char == ' ' { 'X' } else { char };
                write!(f, "{}", char)?;
            }
            if row < self.height - 1 {
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

/// The direction of a word in a Crossword.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Direction {
    Across,
    Down,
}

#[cfg(test)]
mod tests {
    use super::Crossword;
    use crate::{crossword::WordIterator, parse::WordBoundary};
    use std::collections::HashSet;

    use super::Direction;

    #[test]

    fn parse_from_string_works() {
        let result = Crossword::parse(String::from(
            "
abc
def
ghi
",
        ));

        assert!(result.is_ok());

        let c = result.unwrap();
        assert_eq!(String::from("abcdefghi"), c.contents);
        assert_eq!(3, c.width);
        assert_eq!(3, c.height);
        println!("{}", c);
    }

    #[test]
    fn crossword_iterator_works() {
        let input = Crossword::parse(String::from("
ABC
DEF
GHI
")).unwrap();
        let word_boundary = WordBoundary {
            start_col: 0,
            start_row: 0,
            direction: Direction::Across,
            length: 3,
        };

        let t = WordIterator {
            crossword: &input,
            word_boundary: &word_boundary,
            index: 0,
        };

        let s: String = t.collect();

        assert_eq!(String::from("ABC"), s);

        let word_boundary = WordBoundary {
            start_col: 0,
            start_row: 0,
            direction: Direction::Down,
            length: 3,
        };

        let t = WordIterator {
            crossword: &input,
            word_boundary: &word_boundary,
            index: 0,
        };

        let s: String = t.collect();

        assert_eq!(String::from("ADG"), s);
    }

    #[test]
    fn crossword_iterator_eq_works() {
        let input = Crossword::parse(String::from("
ABC
BXX
CXX
")).unwrap();
        let a = WordBoundary {
            start_col: 0,
            start_row: 0,
            direction: Direction::Across,
            length: 3,
        };
        let b = WordBoundary {
            start_col: 0,
            start_row: 0,
            direction: Direction::Down,
            length: 3,
        };

        let a_iter = WordIterator {
            crossword: &input,
            word_boundary: &a,
            index: 0,
        };

        let b_iter = WordIterator {
            crossword: &input,
            word_boundary: &b,
            index: 0,
        };

        assert_eq!(a_iter, b_iter);
    }

    #[test]
    fn crossword_iterator_hash_works() {
        let input = Crossword::parse(String::from("
ABC
BXX
CXX
")).unwrap();
        let a = WordBoundary {
            start_col: 0,
            start_row: 0,
            direction: Direction::Across,
            length: 3,
        };
        let b = WordBoundary {
            start_col: 0,
            start_row: 0,
            direction: Direction::Down,
            length: 3,
        };

        let a_iter = WordIterator {
            crossword: &input,
            word_boundary: &a,
            index: 0,
        };

        let b_iter = WordIterator {
            crossword: &input,
            word_boundary: &b,
            index: 0,
        };

        let mut set = HashSet::new();

        set.insert(a_iter);

        assert!(set.contains(&b_iter));
    }

    #[test]
    fn words_in_direction_works() {
        let input = Crossword::parse(String::from("
SIAM
N.EM
RYAL
")).unwrap();

        let across_words = input.words(Direction::Across);
        let down_words = input.words(Direction::Down);

        assert_eq!(vec!["SIAM", "EM", "RYAL"], across_words);
        assert_eq!(vec!["SNR", "AEA", "MML"], down_words);
    }
}
