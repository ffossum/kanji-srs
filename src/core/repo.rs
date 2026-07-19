use crate::core::kanji::{Jlpt, Kanji, Radical};
use sqlx::PgPool;

pub trait Repo {
    async fn upsert_kanji(&self, kanji: &[Kanji]) -> anyhow::Result<()>;
    async fn get_kanji<T: AsRef<str>>(&self, literals: &[T]) -> anyhow::Result<Vec<Kanji>>;
}

/// Postgres-backed [`Repo`] implementation.
#[derive(Clone)]
pub struct PgRepo {
    pool: PgPool,
}

impl PgRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Connect to `url` and run pending migrations.
    pub async fn connect(url: &str) -> anyhow::Result<Self> {
        let pool = PgPool::connect(url).await?;
        sqlx::migrate!().run(&pool).await?;
        Ok(Self::new(pool))
    }
}

/// Flat row shape returned by [`PgRepo::get_kanji`]; radicals are aggregated
/// into an array by the query so a single round-trip yields the full aggregate.
#[derive(sqlx::FromRow)]
struct KanjiRow {
    literal: String,
    meanings: Vec<String>,
    onyomi: Vec<String>,
    kunyomi: Vec<String>,
    jlpt: Option<Jlpt>,
    radicals: Vec<String>,
}

impl From<KanjiRow> for Kanji {
    fn from(row: KanjiRow) -> Self {
        Kanji {
            literal: row.literal,
            meanings: row.meanings,
            onyomi: row.onyomi,
            kunyomi: row.kunyomi,
            jlpt: row.jlpt,
            radicals: row
                .radicals
                .into_iter()
                .map(|literal| Radical { literal })
                .collect(),
        }
    }
}

impl Repo for PgRepo {
    async fn upsert_kanji(&self, kanji: &[Kanji]) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;

        for k in kanji {
            let kanji_id: i16 = sqlx::query_scalar(
                "INSERT INTO kanji (literal, meanings, kunyomi, onyomi, jlpt)
                 VALUES ($1, $2, $3, $4, $5)
                 ON CONFLICT (literal) DO UPDATE
                 SET meanings = EXCLUDED.meanings,
                     kunyomi  = EXCLUDED.kunyomi,
                     onyomi   = EXCLUDED.onyomi,
                     jlpt     = EXCLUDED.jlpt
                 RETURNING id",
            )
            .bind(&k.literal)
            .bind(&k.meanings)
            .bind(&k.kunyomi)
            .bind(&k.onyomi)
            .bind(k.jlpt)
            .fetch_one(&mut *tx)
            .await?;

            // Replace the radical links so removed radicals don't linger.
            sqlx::query("DELETE FROM kanji_radical WHERE kanji_id = $1")
                .bind(kanji_id)
                .execute(&mut *tx)
                .await?;

            for radical in &k.radicals {
                // DO UPDATE (not DO NOTHING) so RETURNING yields the id on conflict.
                let radical_id: i16 = sqlx::query_scalar(
                    "INSERT INTO radical (literal) VALUES ($1)
                     ON CONFLICT (literal) DO UPDATE SET literal = EXCLUDED.literal
                     RETURNING id",
                )
                .bind(&radical.literal)
                .fetch_one(&mut *tx)
                .await?;

                sqlx::query(
                    "INSERT INTO kanji_radical (kanji_id, radical_id)
                     VALUES ($1, $2) ON CONFLICT DO NOTHING",
                )
                .bind(kanji_id)
                .bind(radical_id)
                .execute(&mut *tx)
                .await?;
            }
        }

        tx.commit().await?;
        Ok(())
    }

    async fn get_kanji<T: AsRef<str>>(&self, literals: &[T]) -> anyhow::Result<Vec<Kanji>> {
        let literals: Vec<&str> = literals.iter().map(AsRef::as_ref).collect();

        let rows: Vec<KanjiRow> = sqlx::query_as(
            "SELECT k.literal,
                    k.meanings,
                    k.onyomi,
                    k.kunyomi,
                    k.jlpt,
                    COALESCE(
                        array_agg(r.literal ORDER BY r.literal)
                            FILTER (WHERE r.id IS NOT NULL),
                        ARRAY[]::text[]
                    ) AS radicals
             FROM kanji k
             LEFT JOIN kanji_radical kr ON kr.kanji_id = k.id
             LEFT JOIN radical r ON r.id = kr.radical_id
             WHERE k.literal = ANY($1)
             GROUP BY k.id",
        )
        .bind(&literals)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(Kanji::from).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pglite_oxide::PgliteServer;
    use sqlx::postgres::PgPoolOptions;

    /// Spin up a throwaway Postgres, migrate it, and return a repo.
    /// The server is returned so the caller keeps it alive for the test.
    ///
    /// PGlite is a single embedded backend, so the pool is capped at one
    /// connection — the default (10) exhausts the backend and times out.
    async fn setup() -> (PgliteServer, PgRepo) {
        let server = PgliteServer::temporary_tcp().expect("start pglite");
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(&server.database_url())
            .await
            .expect("connect");
        sqlx::migrate!().run(&pool).await.expect("migrate");
        (server, PgRepo::new(pool))
    }

    fn sample() -> Kanji {
        Kanji {
            literal: "明".into(),
            meanings: vec!["bright".into(), "light".into()],
            radicals: vec![
                Radical {
                    literal: "日".into(),
                },
                Radical {
                    literal: "月".into(),
                },
            ],
            onyomi: vec!["メイ".into(), "ミョウ".into()],
            kunyomi: vec!["あか".into()],
            jlpt: Some(Jlpt::N4),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn upsert_then_get_roundtrips() {
        let (_server, repo) = setup().await;
        let kanji = sample();

        repo.upsert_kanji(std::slice::from_ref(&kanji))
            .await
            .unwrap();
        let got = repo.get_kanji(&["明"]).await.unwrap();

        assert_eq!(got, vec![kanji]);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn upsert_updates_existing_and_replaces_radicals() {
        let (_server, repo) = setup().await;
        repo.upsert_kanji(&[sample()]).await.unwrap();

        let mut updated = sample();
        updated.meanings = vec!["clear".into()];
        updated.radicals = vec![Radical {
            literal: "日".into(),
        }];
        repo.upsert_kanji(std::slice::from_ref(&updated))
            .await
            .unwrap();

        let got = repo.get_kanji(&["明"]).await.unwrap();
        assert_eq!(got, vec![updated]);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn get_kanji_ignores_missing_literals() {
        let (_server, repo) = setup().await;
        repo.upsert_kanji(&[sample()]).await.unwrap();

        let got = repo.get_kanji(&["明", "存"]).await.unwrap();
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].literal, "明");
    }
}
