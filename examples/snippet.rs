use xwords::{crossword::Crossword, fill_crossword_with_default_wordlist};

fn main() -> Result<(), String> {
    let empty_crossword = Crossword::parse(String::from(
"
XXXXX
X.X.X
XXXXX
X.X.X
XXXXX
",

    ))?;
    let filled_crossword = fill_crossword_with_default_wordlist(&empty_crossword, false)?;
    println!("{}", filled_crossword);
    Ok(())
}
