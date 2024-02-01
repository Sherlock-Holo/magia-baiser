use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use sqlx::sqlite::SqliteConnectOptions;
use sqlx::{query, SqlitePool};
use tracing::instrument;

const DDL: &str = include_str!("../defeat_record.ddl");
const DB_PATH: &str = "mahou_syouzyo.db";

#[derive(Debug, Clone)]
pub struct MahouSyouzyoRecord {
    pool: SqlitePool,
}

impl MahouSyouzyoRecord {
    pub async fn new(dir: &Path) -> anyhow::Result<Self> {
        let pool = Self::init_db(dir).await?;

        Ok(Self { pool })
    }

    #[instrument(level = "debug", err(Debug))]
    pub async fn add_mahou_syouzyo(&self, user: &str, password: &str) -> anyhow::Result<()> {
        query(
            r#"
        INSERT INTO
        defeat_record
        (user, password, count)
        VALUES
        (?1, ?2, ?3)
        ON CONFLICT
        (user, password)
        DO UPDATE SET
        count=count+1
        "#,
        )
        .bind(user)
        .bind(password)
        .bind(1)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn init_db(dir: &Path) -> anyhow::Result<SqlitePool> {
        let path = dir.join(DB_PATH);
        match fs::metadata(&path) {
            Err(err) if err.kind() == ErrorKind::NotFound => {
                let pool = Self::create_pool(&path).await?;

                Self::create_table(&pool).await?;

                Ok(pool)
            }

            Err(err) => Err(err.into()),

            Ok(_) => {
                let options = SqliteConnectOptions::new().filename(path);
                let pool = SqlitePool::connect_with(options).await?;

                Ok(pool)
            }
        }
    }

    async fn create_pool(path: &Path) -> anyhow::Result<SqlitePool> {
        let options = SqliteConnectOptions::new().filename(path);
        let pool = SqlitePool::connect_with(options).await?;

        Ok(pool)
    }

    async fn create_table(pool: &SqlitePool) -> anyhow::Result<()> {
        query(DDL).execute(pool).await?;

        Ok(())
    }
}
