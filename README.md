# xwords

<img src="https://raw.githubusercontent.com/szunami/xwords-rs/main/xwords.png" width="48">

![](https://github.com/szunami/xwords-rs/workflows/Build/badge.svg)
[![](http://meritbadge.herokuapp.com/xwords)](https://crates.io/crates/xwords)

`xwords` is a fast library that fills crossword puzzles. 
This repo also contains a lightweight CLI for invoking the library.

### Caveat Emptor

This is foremost a hobbyist project for me to learn a bit about profiling and optimizing rust. 
I am more than happy to accept contributions or to consider feature requests, 
but please be aware that the future of this project is somewhat uncertain.


## CLI

This command fills a grid that is stored in a local file using a default wordlist.

```text
USAGE:
    xwords [FLAGS] [OPTIONS] --input <FILE>

FLAGS:
    -h, --help       Prints help information
    -p, --profile    Profile the program. Default is false.
    -r, --random     Randomize word fill. Default is false.
    -V, --version    Prints version information

OPTIONS:
    -a, --author <AUTHOR>            Author name across output. Defaults to `xwords-rs`.
    -c, --copyright <COPYRIGHT>      Copyright text for across output. Defaults to `<YEAR> Public domain.`
    -f, --format <FORMAT>            Output format. Can be `grid` for simple grid or `across` for Across Puzzle V2 text.
                                     Default is `grid`.
    -i, --input <FILE>               Input crossword file location.
    -t, --title <TITLE>              Puzzle title for across output. Defaults to title case file name.
    -w, --words <WORDS_FILE_NAME>    File name from /words without extension to use for filling. Default is `en`.

```

Example:

```bash
$ xwords --input grids/20201005_empty.txt
```

```text
CFS.ANGELI.ORDU
AIA.DEEPAS.SEIN
SCLAVONIAN.MFAS
IKANTLETGO.ALLE
OLDY..ROE.ANOSE
.YOOHOOMRSBLOOM
...FUGUE.IRIDAL
FAA.LIS.ECO.SPY
IMPROV.ACOOK...
BILLIEJEANKING.
SEUSS.IAD..CEIL
..STTIC..ALKALI
CITI.CACOMISTLE
CORN.OMOLON.LIS
CLEE.NATURA.YST
```

This command runs in about 2 seconds on my machine.

Example with random word fill:
    
```bash
$ xwords --input grids/waffle.txt --random
```

```text
FLUKY
T.W.U
ETAAC
N.I.C
SITKA
```

To output an Across Puzzle V2 text file, use the `--format` flag:

```bash
$ xwords --input grids/waffle.txt --format across --title "Waffle" --author "Adi" --copyright "2025 Adi"
```

```text
<ACROSS PUZZLE V2>
<TITLE>
Waffle
<AUTHOR>
Adi
<COPYRIGHT>
2025 Adi
<SIZE>
5x5
<GRID>
SHIAS
L.S.K
IFIDO
P.T.R
TOSET
<ACROSS>
SHIAS
IFIDO
TOSET
<DOWN>
SLIPT
ISITS
SKORT
```

The <ACROSS> and <DOWN> are the word placeholders for clues.

To use other word lists, you can specify the file name from the `/words` directory without the extension:

```bash
$ xwords --input grids/waffle.txt --random --words ro_dex_095
```

```text
DINȚA
E.O.L
PĂTAT
U.A.U
SĂTUL
```

## Library

```rust
use xwords::{crossword::Crossword, fill_crossword_with_default_wordlist};

fn main() -> Result<(), String> {
    let empty_crossword = Crossword::new(String::from(
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

/*
ZETA.TWIT.VOWEL
ETAT.IANA.EVOKE
RINTINTIN.REVIE
OCT.TIE.TUI.ENR
..ATHA.TASTINGS
TOLEAN.ILIES...
ISIAC.TEAN.STEM
ZAT.ACHATES.HRA
AYES.SETE.TYEES
...TUTSI.URALIC
VENERATE.SEWA..
ORA.TRO.UES.TOA
WISHI.NETASSETS
ETHIC.EVIL.USTO
RUEDA.SWAL.OTSU
*/
```
On my machine, the above snippet runs in about 3 seconds.

Behind the scenes, this snippet loads an indexed wordlist, and iteratively fills the input with valid words.
