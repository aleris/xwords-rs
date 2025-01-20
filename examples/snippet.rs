use xwords::{crossword::Crossword, fill_crossword_with_default_wordlist};

fn main() -> Result<(), String> {
    let empty_crossword = Crossword::parse(String::from(
"
XXXX.XXXX.XXXXX
XXXX.XXXX.XXXXX
XXXXXXXXX.XXXXX
XXX.XXX.XXX.XXX
..XXXX.XXXXXXXX
XXXXXX.XXXXX...
XXXXX.XXXX.XXXX
XXX.XXXXXXX.XXX
XXXX.XXXX.XXXXX
...XXXXX.XXXXXX
XXXXXXXX.XXXX..
XXX.XXX.XXX.XXX
XXXXX.XXXXXXXXX
XXXXX.XXXX.XXXX
XXXXX.XXXX.XXXX
",

    ))?;
    let filled_crossword = fill_crossword_with_default_wordlist(&empty_crossword, false)?;
    println!("{}", filled_crossword);
    Ok(())
}
