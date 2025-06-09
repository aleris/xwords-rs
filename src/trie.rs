/*!
A data structure that provides efficient lookup of partially filled words.
*/

use crate::File;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::io::ErrorKind::InvalidInput;
use std::io::{BufRead, Error};
use std::path::PathBuf;
use std::{fmt, io};

#[derive(Clone, Serialize, Deserialize)]
pub struct TrieNode {
    contents: Option<char>,
    children: FxHashMap<char, TrieNode>,
    is_terminal: bool,
}

impl TrieNode {
    fn add_sequence(mut self, chars: &str) -> TrieNode {
        match chars.chars().next() {
            Some(val) => match self.children.remove_entry(&val) {
                Some((_, child)) => {
                    let rest: String = chars.chars().skip(1).collect();
                    self.children.insert(val, child.add_sequence(&rest));
                }
                None => {
                    let tmp = TrieNode {
                        children: FxHashMap::default(),
                        contents: Some(val),
                        is_terminal: false,
                    };
                    let rest: String = chars.chars().skip(1).collect();
                    self.children.insert(val, tmp.add_sequence(&rest));
                }
            },
            None => {
                self.is_terminal = true;
            }
        }
        self
    }

    fn display_helper(
        &self,
        f: &mut fmt::Formatter<'_>,
        depth: usize,
        first_child: bool,
    ) -> Result<(), fmt::Error> {
        if !first_child {
            for _ in 0..depth {
                write!(f, "\t")?;
            }
        } else {
            write!(f, "\t")?;
        }
        write!(f, "{}", self.contents.unwrap_or('*'))?;

        if self.is_terminal {
            write!(f, "'")?;
        }

        if self.children.is_empty() {
            return writeln!(f);
        }

        for (index, key) in self.children.keys().into_iter().enumerate() {
            self.children
                .get(key)
                .unwrap()
                .display_helper(f, depth + 1, index == 0)?;
        }

        Ok(())
    }

    fn words<T: Iterator<Item = char> + Clone>(
        &self,
        mut pattern: T,
        partial: &mut String,
        result: &mut Vec<String>,
    ) {
        if self.contents.is_some() {
            partial.push(self.contents.unwrap());
        }

        match pattern.next() {
            Some(new_char) => {
                if new_char == ' ' {
                    for child in self.children.values() {
                        child.words(pattern.clone(), partial, result);
                    }
                } else {
                    if let Some(child) = self.children.get(&new_char) {
                        child.words(pattern, partial, result);
                    }
                }
            }
            None => {
                if self.is_terminal {
                    result.push(partial.clone());
                }
            }
        }

        if self.contents.is_some() {
            partial.pop();
        }
    }

    pub fn is_viable<T: Iterator<Item = char> + Clone>(&self, mut chars: T) -> bool {
        match chars.next() {
            None => self.is_terminal,

            Some(c) => {
                if c == ' ' {
                    for child in self.children.values() {
                        if child.is_viable(chars.clone()) {
                            return true;
                        }
                    }
                    false
                } else {
                    match self.children.get(&c) {
                        None => false,
                        Some(child) => child.is_viable(chars),
                    }
                }
            }
        }
    }
}

impl fmt::Display for TrieNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        self.display_helper(f, 1, true)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Trie {
    pub root: TrieNode,
}

impl fmt::Display for Trie {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        self.root.fmt(f)
    }
}

impl Trie {
    pub fn load_default() -> Result<Trie, Error> {
        Trie::load("en")
    }

    pub fn load(name: &str) -> Result<Trie, Error> {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push(format!("words/{}.bincode", name));
        let file = File::open(path.clone())
            .map_err(|e| Error::new(e.kind(), format!("Could not open file {:?}", path)))?;
        bincode::deserialize_from::<File, Trie>(file)
            .map_err(|e| Error::new(InvalidInput, e.to_string()))
    }

    pub fn build(words: Vec<String>) -> Trie {
        let mut root = TrieNode {
            contents: None,
            children: FxHashMap::default(),
            is_terminal: false,
        };

        for word in words.iter() {
            root = root.add_sequence(&word);
        }

        Trie { root }
    }

    pub fn build_bin_code(file_path: &PathBuf) -> Result<PathBuf, Error> {
        let name = file_path.display().to_string();
        let file_name = file_path
            .file_stem()
            .ok_or_else(|| Error::new(InvalidInput, "File has no stem"))?
            .to_str()
            .ok_or_else(|| Error::new(InvalidInput, "File stem is not valid"))?;
        let out_path = PathBuf::from(format!("words/{}.bincode", file_name));
        let file = File::open(file_path)
            .map_err(|e| Error::new(e.kind(), format!("Could not open file {}", name)))?;
        let extension = file_path
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| Error::new(InvalidInput, "File has no extension"))?;
        let words = match extension {
            "json" => Trie::load_words_from_json(&file),
            "txt" => Trie::load_words_from_text(&file),
            ext => Err(Error::new(
                InvalidInput,
                format!("Unsupported file format: {}", ext),
            ))?,
        };
        let words = Trie::make_words_uppercase(words);
        let trie = Trie::build(words);
        let trie_file = File::create(&out_path)?;
        bincode::serialize_into(trie_file, &trie)
            .map_err(|e| Error::new(InvalidInput, e.to_string()))?;
        Ok(out_path)
    }

    pub fn words<T: Iterator<Item = char> + Clone>(&self, pattern: T) -> Vec<String> {
        let mut result = Vec::with_capacity(4);
        let mut partial = String::with_capacity(4);
        self.root.words(pattern, &mut partial, &mut result);
        result
    }

    pub fn is_viable<T: Iterator<Item = char> + Clone>(&self, chars: T) -> bool {
        self.root.is_viable(chars)
    }

    fn load_words_from_json(file: &File) -> Vec<String> {
        let words = serde_json::from_reader(file).expect("JSON was not well-formatted");
        words
    }

    fn load_words_from_text(file: &File) -> Vec<String> {
        let words = io::BufReader::new(file)
            .lines()
            .flatten()
            .filter(|s| !s.is_empty() && !s.starts_with("#"))
            .collect::<Vec<String>>();
        words
    }

    fn make_words_uppercase(words: Vec<String>) -> Vec<String> {
        words.into_iter().map(|s| s.to_uppercase()).collect()
    }
}

#[cfg(test)]
mod tests {
    use rustc_hash::FxHashMap;

    use super::{Trie, TrieNode};
    use std::collections::HashSet;
    use std::path::PathBuf;

    #[test]
    #[ignore]
    fn rebuild_serialized_trie_en() {
        let result = Trie::build_bin_code(&PathBuf::from("words/en.json"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_trie_load_en() {
        let trie = Trie::load("en");
        assert!(trie.is_ok());
    }

    #[test]
    #[ignore]
    fn rebuild_serialized_trie_ro_dex() {
        let result = Trie::build_bin_code(&PathBuf::from("words/ro_dex_000.txt"));
        if let Err(e) = result {
            panic!("{}", e);
        }
        let result = Trie::build_bin_code(&PathBuf::from("words/ro_dex_025.txt"));
        if let Err(e) = result {
            panic!("{}", e);
        }
        let result = Trie::build_bin_code(&PathBuf::from("words/ro_dex_050.txt"));
        if let Err(e) = result {
            panic!("{}", e);
        }
        let result = Trie::build_bin_code(&PathBuf::from("words/ro_dex_060.txt"));
        if let Err(e) = result {
            panic!("{}", e);
        }
        let result = Trie::build_bin_code(&PathBuf::from("words/ro_dex_070.txt"));
        if let Err(e) = result {
            panic!("{}", e);
        }
        let result = Trie::build_bin_code(&PathBuf::from("words/ro_dex_080.txt"));
        if let Err(e) = result {
            panic!("{}", e);
        }
    }

    #[test]
    fn test_trie_load_ro_dex() {
        let trie = Trie::load("ro_dex_000");
        if let Err(e) = trie {
            panic!("{}", e);
        }
    }

    #[test]
    fn display_works() {
        let mut root = TrieNode {
            contents: None,
            children: FxHashMap::default(),
            is_terminal: false,
        };

        root.children.insert(
            'b',
            TrieNode {
                contents: Some('b'),
                children: FxHashMap::default(),
                is_terminal: false,
            },
        );

        let mut c = TrieNode {
            contents: Some('c'),
            children: FxHashMap::default(),
            is_terminal: false,
        };

        c.children.insert(
            'd',
            TrieNode {
                contents: Some('d'),
                children: FxHashMap::default(),
                is_terminal: false,
            },
        );

        root.children.insert('c', c);

        println!("{}", root);
    }

    #[test]
    fn add_sequence_works() {
        let root = TrieNode {
            contents: Some('a'),
            children: FxHashMap::default(),
            is_terminal: false,
        };

        let new_root = root.add_sequence("itsyaboi");

        println!("{}", new_root);

        let another_root = new_root.add_sequence("wereallyouthere");

        println!("{}", another_root)
    }

    #[test]
    fn build_works() {
        println!(
            "{}",
            Trie::build(vec![
                String::from("asdf"),
                String::from("asset"),
                String::from("bass"),
                String::from("baseball"),
                String::from("bassooon"),
                String::from("basset"),
            ])
        );
    }

    #[test]
    fn words_works() {
        let trie = Trie::build(vec![
            String::from("bass"),
            String::from("bats"),
            String::from("bess"),
            String::from("be"),
        ]);

        let expected: HashSet<String> = vec![String::from("bass"), String::from("bess")]
            .iter()
            .cloned()
            .collect();

        let iter = String::from("b ss");
        let actual: HashSet<String> = trie.words(iter.chars()).iter().cloned().collect();
        assert_eq!(expected, actual,)
    }
}
