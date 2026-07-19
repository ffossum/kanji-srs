use crate::kanji::Kanji;
use std::future::Future;

pub trait Repo {
    fn upsert_kanji(
        &self,
        kanji: &[Kanji],
    ) -> impl Future<Output = anyhow::Result<()>> + Send;

    fn get_kanji<T: AsRef<str> + Sync>(
        &self,
        literals: &[T],
    ) -> impl Future<Output = anyhow::Result<Vec<Kanji>>> + Send;
}
