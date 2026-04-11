use sqlx::Row;
use sqlx::sqlite::SqlitePool;
use std::future::Future;
use suitcase::{RunConfig, Suite, run, suite_methods};
use tokio::runtime::{Handle, Runtime};

struct DbSuite {
    handle: Handle,
    pool: Option<SqlitePool>,
    before_each_calls: u32,
}

impl DbSuite {
    fn new(handle: Handle) -> Self {
        Self {
            handle,
            pool: None,
            before_each_calls: 0,
        }
    }

    fn pool(&self) -> &SqlitePool {
        self.pool
            .as_ref()
            .expect("pool must be initialized by setup_suite before hooks or cases run")
    }

    fn block_on<F>(&self, fut: F) -> F::Output
    where
        F: Future + Send,
        F::Output: Send,
    {
        self.handle.block_on(fut)
    }

    async fn count_rows(pool: &SqlitePool, table: &str) -> i64 {
        let q = format!("SELECT COUNT(*) AS c FROM {table}");
        let row = sqlx::query(&q).fetch_one(pool).await.unwrap();
        row.get::<i64, _>("c")
    }

    fn test_insert_alice(&mut self) {
        self.block_on(async {
            sqlx::query("INSERT INTO users (name, version) VALUES (?, ?)")
                .bind("alice")
                .bind(1i64)
                .execute(self.pool())
                .await
                .unwrap();
        });
    }

    fn test_insert_bob(&mut self) {
        self.block_on(async {
            sqlx::query("INSERT INTO users (name, version) VALUES (?, ?)")
                .bind("bob")
                .bind(1i64)
                .execute(self.pool())
                .await
                .unwrap();
            let n = DbSuite::count_rows(self.pool(), "users").await;
            assert_eq!(n, 2, "alice then bob");
        });
    }

    fn test_bump_alice_version(&mut self) {
        self.block_on(async {
            sqlx::query("UPDATE users SET version = ? WHERE name = ?")
                .bind(2i64)
                .bind("alice")
                .execute(self.pool())
                .await
                .unwrap();
            let row = sqlx::query("SELECT version FROM users WHERE name = ?")
                .bind("alice")
                .fetch_one(self.pool())
                .await
                .unwrap();
            assert_eq!(row.get::<i64, _>("version"), 2);
        });
    }

    fn test_assert_final_counts(&mut self) {
        self.block_on(async {
            assert_eq!(DbSuite::count_rows(self.pool(), "users").await, 2);
            let logs = DbSuite::count_rows(self.pool(), "op_log").await;
            assert!(logs >= 1, "at least setup + hooks + cases");
        });
    }
}

impl Suite for DbSuite {
    fn setup_suite(&mut self) {
        self.pool = Some(self.block_on(async {
            let pool = SqlitePool::connect("sqlite::memory:")
                .await
                .expect("sqlite connect");

            sqlx::migrate!("tests/sqlx_sqlite_migrations")
                .run(&pool)
                .await
                .expect("apply migrations");

            sqlx::query("INSERT INTO op_log (message) VALUES ('setup_suite')")
                .execute(&pool)
                .await
                .unwrap();

            pool
        }));
    }

    fn teardown_suite(&mut self) {
        self.block_on(async {
            sqlx::query("INSERT INTO op_log (message) VALUES ('teardown_suite')")
                .execute(self.pool())
                .await
                .unwrap();
        });
    }

    fn before_each(&mut self) {
        self.before_each_calls += 1;
        let n = self.before_each_calls;
        self.block_on(async {
            sqlx::query("INSERT INTO op_log (message) VALUES (?)")
                .bind(format!("before_each #{n}"))
                .execute(self.pool())
                .await
                .unwrap();
        });
    }

    fn after_each(&mut self) {
        self.block_on(async {
            sqlx::query("INSERT INTO op_log (message) VALUES ('after_each')")
                .execute(self.pool())
                .await
                .unwrap();
        });
    }
}

#[test]
fn sqlx_sqlite_suite_mutates_db_between_cases() -> Result<(), Box<dyn std::error::Error>> {
    let rt = Runtime::new()?;

    let mut suite = DbSuite::new(rt.handle().clone());

    run(
        &mut suite,
        suite_methods![
            DbSuite, s =>
                test_insert_alice,
                test_insert_bob,
                test_bump_alice_version,
                test_assert_final_counts
        ],
        RunConfig::all(),
    );

    let users = rt.block_on(DbSuite::count_rows(suite.pool(), "users"));
    let ops = rt.block_on(DbSuite::count_rows(suite.pool(), "op_log"));
    assert_eq!(users, 2);
    assert!(ops >= 10, "expected hook + case log rows, got {ops}");
    println!("done: users={users}, op_log rows={ops}");

    Ok(())
}
