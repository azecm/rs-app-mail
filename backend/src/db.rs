use std::sync::Arc;

use deadpool_postgres::{Config, ManagerConfig, Object, Pool, PoolError, RecyclingMethod, Runtime};
use once_cell::sync::Lazy;
use postgres_types::ToSql;
use serde::Deserialize;
use tokio_postgres::{NoTls, Row};

const DB_PARAMS: &str = include_str!("../../env.json");

#[derive(Deserialize, Debug)]
struct DBParams {
    db: String,
    user: String,
    password: String,
    host: String,
    port: u16,
}

static DB_POOL: Lazy<Arc<Pool>> = Lazy::new(|| {
    let p = serde_json::from_str::<DBParams>(DB_PARAMS).unwrap();

    let mut cfg = Config::new();
    cfg.dbname = Some(p.db);
    cfg.user = Some(p.user);
    cfg.password = Some(p.password);
    cfg.host = Some(p.host);
    cfg.port = Some(p.port);

    cfg.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });
    let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();

    Arc::new(pool)
});


pub async fn db_conn() -> Result<Object, PoolError> {
    DB_POOL.get().await
}

pub async fn db_query<R>(data_from: fn(Row) -> R, statement: &str, params: &[&(dyn ToSql + Sync)]) -> Vec<R>
{
    match db_conn().await {
        Ok(db) => {
            match db.query(statement, params).await {
                Ok(result) => {
                    result.into_iter().map(data_from).collect::<Vec<_>>()
                }
                Err(err) => {
                    tracing::error!("db_query [statement]: {:?}", statement);
                    tracing::error!("db_query [params]: {:?}", params);
                    tracing::error!("db_query [error]: {:?}", err);
                    vec![]
                }
            }
        }
        Err(err) => {
            tracing::error!("db_conn: {:?}", err);
            vec![]
        }
    }
}

pub async fn db_update_query(statement: &str, params: &[&(dyn ToSql + Sync)]) -> bool
{
    match db_conn().await {
        Ok(db) => {
            match db.query(statement, params).await {
                Ok(_) => {
                    true
                }
                Err(err) => {
                    tracing::error!("db_query [statement]: {:?}", statement);
                    tracing::error!("db_query [params]: {:?}", params);
                    tracing::error!("db_query [error]: {:?}", err);
                    false
                }
            }
        }
        Err(err) => {
            tracing::error!("db_conn: {:?}", err);
            false
        }
    }
}