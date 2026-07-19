#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Radical {
    pub literal: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Kanji {
    pub literal: String,
    pub meanings: Vec<String>,
    pub radicals: Vec<Radical>,
    pub onyomi: Vec<String>,
    pub kunyomi: Vec<String>,
    pub jlpt: Option<Jlpt>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Jlpt {
    N5,
    N4,
    N3,
    N2,
    N1,
}
