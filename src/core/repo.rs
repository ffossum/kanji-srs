use crate::core::kanji::Kanji;

trait Repo {
    async fn upsert_kanji(kanji: &[Kanji]) -> ();
    async fn get_kanji<T: AsRef<str>>(literals: &[T]) -> Vec<Kanji>;
}
