struct Radical {
    literal: String,
}

pub struct Kanji {
    literal: String,
    meanings: Vec<String>,
    radicals: Vec<Radical>,
    onyomi: Vec<String>,
    kunyomi: Vec<String>,
    jlpt: Option<JLPT>,
}

enum JLPT {
    N5,
    N4,
    N3,
    N2,
    N1,
}
