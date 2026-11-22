//! ASCII art and the `Sprite` type that turns it into something drawable.
//!
//! All art is authored facing right. Left-facing versions are produced by
//! mirroring, which also swaps direction-sensitive glyphs like `/` and `\`.
//! `@` marks an eye and is drawn as a white `o`.

/// Marker for "nothing here, let whatever is behind show through".
pub const TRANSPARENT: char = '\0';

#[derive(Clone)]
pub struct Sprite {
    pub rows: Vec<Vec<char>>,
    pub w: i32,
    pub h: i32,
}

impl Sprite {
    /// Build a sprite from lines of art. Spaces *inside* a row (between its
    /// first and last visible glyph) are opaque so fish occlude what's behind
    /// them; spaces outside are transparent.
    pub fn new(lines: &[&str], mirror: bool) -> Self {
        let w = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
        let rows: Vec<Vec<char>> = lines
            .iter()
            .map(|line| {
                let mut row: Vec<char> = line.chars().collect();
                let first = row.iter().position(|&c| c != ' ');
                let last = row.iter().rposition(|&c| c != ' ');
                if let (Some(first), Some(last)) = (first, last) {
                    for (i, c) in row.iter_mut().enumerate() {
                        if i < first || i > last {
                            *c = TRANSPARENT;
                        }
                    }
                } else {
                    row.iter_mut().for_each(|c| *c = TRANSPARENT);
                }
                row.resize(w, TRANSPARENT);
                if mirror {
                    row.reverse();
                    row.iter_mut().for_each(|c| *c = flip(*c));
                }
                row
            })
            .collect();
        Sprite {
            h: rows.len() as i32,
            w: w as i32,
            rows,
        }
    }

    /// Number of visible cells. A fish's body size, which decides who can eat whom.
    pub fn area(&self) -> usize {
        self.rows.iter().flatten().filter(|&&c| c != TRANSPARENT).count()
    }
}

fn flip(c: char) -> char {
    match c {
        '/' => '\\',
        '\\' => '/',
        '(' => ')',
        ')' => '(',
        '<' => '>',
        '>' => '<',
        '[' => ']',
        ']' => '[',
        '{' => '}',
        '}' => '{',
        '`' => '\'',
        '\'' => '`',
        c => c,
    }
}

/// Fish designs with spawn weights. Bigger designs are rarer.
pub const FISH: &[(&[&str], u32)] = &[
    (&[r"><(@>"], 6),
    (&[r"  ,--.", r">=<  @ >", r"  `--'"], 3),
    (&[r"  __", r"><(__@)>"], 5),
    (&[r"    .-''-.", r"><=(  @    )>", r"    `-..-'"], 2),
    (&[r"   ,-.", r" ><__@>"], 5),
    (&[r"   /|", r"><<  @>", r"   \|"], 3),
    // The big one.
    (
        &[
            r"     _.-'''-._",
            r" \  ,'  ,--.  `.",
            r"  >=<   (@   )   >",
            r" /  `.  `--'  ,'",
            r"     `-.___.-'",
        ],
        1,
    ),
];

/// Newborns. They grow into their parent's design.
pub const BABY: &[&str] = &[r"><>"];

pub const SHARK: &[&str] = &[
    r"                __",
    r"               ( `\",
    r"  ,-----------)   `\____",
    r" /    |  |  |  |    @    `-.",
    r"<_.--------------..______.-'",
];

pub const WHALE_BODY: &[&str] = &[
    r"           _.-'''''-._",
    r"    ___.--'           `-._",
    r" .-'                  @   \",
    r"  `--.______....---.._____/",
];

/// Spout animation, three rows drawn above the whale's head.
pub const WHALE_SPOUT: &[&[&str]] = &[
    &["", "", ""],
    &["", "", "                     |"],
    &["", "                     .|.", "                      |"],
    &["                    . | .", "                     .|.", "                      |"],
    &["                   .  |  .", "                    . ' ' .", "                     ' | '"],
    &["                  .   .   .", "                     .   .", ""],
];

pub const SHIP: &[&str] = &[
    r"      |    |    |",
    r"     )_)  )_)  )_)",
    r"    )___))___))___)\",
    r"   )____)____)_____)\\",
    r" __|_____|____|_____|___\\\__",
    r" \                        /",
];

pub const DUCKS: &[&[&str]] = &[
    &[
        r"   _         _         _",
        r" _(@)>     _(@)>     _(@)>",
        r" \___/     \___/     \___/",
    ],
    &[
        r"   _         _         _",
        r" _(@)>     _(@)>     _(@)>",
        r" \_~_/     \_~_/     \_~_/",
    ],
];

pub const CASTLE: &[&str] = &[
    r"              T~~",
    r"              |",
    r"             /^\",
    r"            /   \",
    r" _   _   _ /     \ _   _   _",
    r"[ ]_[ ]_[ ]/ _   _ \[ ]_[ ]_[ ]",
    r"|_=__-_ =_|_[ ]_[ ]_|_=-___-__|",
    r" |=  []- |-  /| |\   |=_ =[] |",
    r" |- =_   | =| | | |  |- = -  |",
    r" |_______|__|_|_|_|__|_______|",
];

/// Rows of animated waterline, tiled and scrolled at draw time. Spaces are
/// gaps you can see through.
pub const WAVES: [&str; 4] = [
    "~~~~~~^~~~~~~~^~~~~~~~^~~~~~~",
    "^^^^ ^^^^^ ^^^^^^ ^^^^ ^^^^^ ",
    "^^^^^ ^^^^ ^^^^^^^ ^^^ ^^^^^^ ^^",
    "^^ ^^^^^^ ^^^^ ^^^^^ ^^^^ ^^^^ ",
];

pub const BUBBLE_FRAMES: [&str; 3] = [".", "o", "O"];
