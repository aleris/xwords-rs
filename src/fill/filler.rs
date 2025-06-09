/*!
An algorithm that composes algorithms and data structures throughout this
crate. This is where the magic happens.
*/

use rand::seq::SliceRandom;
use std::{
    collections::HashSet,
    hash::BuildHasherDefault,
    time::Instant,
};

use rustc_hash::FxHasher;

use crate::{
    crossword::{Crossword, WordIterator},
    parse::parse_word_boundaries,
    trie::Trie,
};

use super::{
    build_square_word_boundary_lookup,
    cache::{CachedIsViable, CachedWords},
    fill_one_word, is_viable_reuse, words_orthogonal_to_word, Fill,
};

pub struct Filler<'s> {
    word_cache: CachedWords,
    is_viable_cache: CachedIsViable,

    trie: &'s Trie,
    random: bool,
    max_time_seconds: u64,
}

impl<'s> Filler<'s> {
    pub fn new(trie: &'s Trie, random: bool, max_time_seconds: Option<u64>) -> Filler<'s> {
        Filler {
            word_cache: CachedWords::default(),
            is_viable_cache: CachedIsViable::default(),
            trie,
            random,
            max_time_seconds: max_time_seconds.unwrap_or(120),
        }
    }
}

impl<'s> Fill for Filler<'s> {
    fn fill(&mut self, initial_crossword: &Crossword) -> Result<Crossword, String> {
        let start_time = Instant::now();
        let mut candidate_count = 0;

        let word_boundaries = parse_word_boundaries(&initial_crossword);
        let mut already_used = HashSet::with_capacity_and_hasher(
            word_boundaries.len(),
            BuildHasherDefault::<FxHasher>::default(),
        );

        let mut candidates = vec![initial_crossword.to_owned()];

        let word_boundary_lookup = build_square_word_boundary_lookup(&word_boundaries);

        while let Some(candidate) = candidates.pop() {
            candidate_count += 1;

            let elapsed_secs = start_time.elapsed().as_secs();
            if elapsed_secs > self.max_time_seconds {
                eprintln!(
                    "[DEBUG] No solution found after time limit of {} seconds reached, {} candidates",
                    self.max_time_seconds, candidate_count
                );
                return Err(format!(
                    "Time limit of {} seconds reached after {} candidates",
                    self.max_time_seconds, candidate_count
                ));
            }

            if candidate_count % 10_000 == 0 {
                eprintln!("[DEBUG] Current candidate:\n{}", candidate);
                eprintln!(
                    "[DEBUG] Throughput: {} candidates/ms",
                    candidate_count as f32 / start_time.elapsed().as_millis() as f32
                );
            }

            let to_fill = word_boundaries
                .iter()
                .map(|word_boundary| WordIterator::new(&candidate, word_boundary))
                .filter(|iter| iter.clone().any(|c| c == ' '))
                .min_by_key(|iter| {
                    let words = self.word_cache.words(iter.clone(), self.trie);
                    (
                        words.len(),
                        iter.word_boundary.start_row,
                        iter.word_boundary.start_col,
                    )
                })
                .ok_or_else(|| "No fillable words found".to_string())?;

            let orthogonals =
                words_orthogonal_to_word(&to_fill.word_boundary, &word_boundary_lookup);

            let mut potential_fills = self.word_cache.words(to_fill.clone(), self.trie).to_vec();

            if self.random {
                potential_fills.shuffle(&mut rand::rng());
            }

            for potential_fill in potential_fills {
                let new_candidate = fill_one_word(&candidate, &to_fill.clone(), &potential_fill);

                let (viable, tmp) = is_viable_reuse(
                    &new_candidate,
                    &orthogonals,
                    self.trie,
                    already_used,
                    &mut self.is_viable_cache,
                );
                already_used = tmp;
                already_used.clear();

                if viable {
                    if !new_candidate.contents.contains(&' ') {
                        eprintln!(
                            "[DEBUG] Ok, total candidates: {}, time taken: {} ms",
                            candidate_count,
                            start_time.elapsed().as_millis()
                        );
                        return Ok(new_candidate);
                    }
                    candidates.push(new_candidate);
                }
            }
        }

        Err("No valid solution found".to_string())
    }
}

#[cfg(test)]
mod tests {
    use crate::{fill::Fill, Trie};

    use crate::Crossword;

    use std::{cmp::Ordering, time::Instant};

    use super::Filler;

    #[test]
    fn test() {
        assert_eq!((1, 2).cmp(&(3, 4)), Ordering::Less)
    }

    #[test]
    fn medium_grid() {
        let grid = Crossword::parse(String::from(
            "
XXXX...
XXXX...
XXXX...
XXXXXXX
...XXXX
...XXXX
...XXXX
",
        ))
        .unwrap();

        let now = Instant::now();
        let trie = Trie::load_default().expect("Failed to load trie");
        let mut filler = Filler::new(&trie, false, None);
        let filled_puz = filler.fill(&grid).unwrap();
        println!("Filled in {} seconds.", now.elapsed().as_secs());
        println!("{}", filled_puz);
    }

    #[test]
    fn medium_grid_ro() {
        let grid = Crossword::parse(String::from(
            "
XXXX...
XXXX...
XXXX...
XXXXXXX
...XXXX
...XXXX
...XXXX
",
        ))
        .unwrap();

        let now = Instant::now();
        let trie = Trie::load("ro_dex_080").expect("Failed to load trie");
        let mut filler = Filler::new(&trie, true, None);
        let filled_puz = filler.fill(&grid).unwrap();
        println!("Filled in {} seconds.", now.elapsed().as_secs());
        println!("{}", filled_puz);
    }

    #[test]
    fn waffle_grid_ro_dex_000() {
        let grid = Crossword::parse(String::from(
            "
XXXXX
X.X.X
XXXXX
X.X.X
XXXXX
",
        ))
        .unwrap();

        let now = Instant::now();
        let trie = Trie::load("ro_dex_000").expect("Failed to load trie");
        let mut filler = Filler::new(&trie, true, None);
        let filled_puz = filler.fill(&grid).unwrap();
        println!("Filled in {} seconds.", now.elapsed().as_secs());
        println!("{}", filled_puz);
    }
}
