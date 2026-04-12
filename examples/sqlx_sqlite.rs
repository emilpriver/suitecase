//! sqlx + SQLite + embedded migrations. Run: `cargo test --example sqlx_sqlite`

#![allow(dead_code)]

use sqlx::Row;
use sqlx::sqlite::SqlitePool;
use std::future::Future;
use suitecase::{cargo_case_tests_with_hooks, cases_fn, Case, HookFns};
use tokio::runtime::{Handle, Runtime};

struct DbSuite {
    _rt: Runtime,
    handle: Handle,
    pool: Option<SqlitePool>,
    before_each_calls: u32,
}

impl DbSuite {
    fn new() -> Self {
        let _rt = Runtime::new().expect("tokio Runtime");
        let handle = _rt.handle().clone();
        Self {
            _rt,
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
}

impl Default for DbSuite {
    fn default() -> Self {
        Self::new()
    }
}

fn case_insert_alice(s: &mut DbSuite) {
    s.block_on(async {
        sqlx::query("INSERT INTO users (name, version) VALUES (?, ?)")
            .bind("alice")
            .bind(1i64)
            .execute(s.pool())
            .await
            .unwrap();
    });
}

fn case_insert_bob(s: &mut DbSuite) {
    s.block_on(async {
        let pool = s.pool();
        let has_alice = sqlx::query("SELECT 1 AS x FROM users WHERE name = ?")
            .bind("alice")
            .fetch_optional(pool)
            .await
            .unwrap()
            .is_some();
        if !has_alice {
            sqlx::query("INSERT INTO users (name, version) VALUES (?, ?)")
                .bind("alice")
                .bind(1i64)
                .execute(pool)
                .await
                .unwrap();
        }
        sqlx::query("INSERT INTO users (name, version) VALUES (?, ?)")
            .bind("bob")
            .bind(1i64)
            .execute(pool)
            .await
            .unwrap();
        let n = DbSuite::count_rows(pool, "users").await;
        assert_eq!(n, 2, "alice then bob");
    });
}

fn case_bump_alice_version(s: &mut DbSuite) {
    s.block_on(async {
        let pool = s.pool();
        let has_alice = sqlx::query("SELECT 1 AS x FROM users WHERE name = ?")
            .bind("alice")
            .fetch_optional(pool)
            .await
            .unwrap()
            .is_some();
        if !has_alice {
            sqlx::query("INSERT INTO users (name, version) VALUES (?, ?)")
                .bind("alice")
                .bind(1i64)
                .execute(pool)
                .await
                .unwrap();
        }
        sqlx::query("UPDATE users SET version = ? WHERE name = ?")
            .bind(2i64)
            .bind("alice")
            .execute(pool)
            .await
            .unwrap();
        let row = sqlx::query("SELECT version FROM users WHERE name = ?")
            .bind("alice")
            .fetch_one(pool)
            .await
            .unwrap();
        assert_eq!(row.get::<i64, _>("version"), 2);
    });
}

fn case_assert_final_counts(s: &mut DbSuite) {
    s.block_on(async {
        let pool = s.pool();
        let u = DbSuite::count_rows(pool, "users").await;
        if u < 2 {
            sqlx::query("INSERT INTO users (name, version) VALUES (?, ?)")
                .bind("alice")
                .bind(1i64)
                .execute(pool)
                .await
                .unwrap();
            sqlx::query("INSERT INTO users (name, version) VALUES (?, ?)")
                .bind("bob")
                .bind(1i64)
                .execute(pool)
                .await
                .unwrap();
        }
        assert_eq!(DbSuite::count_rows(pool, "users").await, 2);
        let logs = DbSuite::count_rows(pool, "op_log").await;
        assert!(logs >= 1, "at least setup + hooks + cases");
    });
}

fn db_setup_suite(s: &mut DbSuite) {
    s.pool = Some(s.block_on(async {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("sqlite connect");

        sqlx::migrate!("examples/sqlx_sqlite_migrations")
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

fn db_teardown_suite(s: &mut DbSuite) {
    s.block_on(async {
        sqlx::query("INSERT INTO op_log (message) VALUES ('teardown_suite')")
            .execute(s.pool())
            .await
            .unwrap();
    });
}

fn db_before_each(s: &mut DbSuite) {
    s.before_each_calls += 1;
    let n = s.before_each_calls;
    s.block_on(async {
        sqlx::query("INSERT INTO op_log (message) VALUES (?)")
            .bind(format!("before_each #{n}"))
            .execute(s.pool())
            .await
            .unwrap();
    });
}

fn db_after_each(s: &mut DbSuite) {
    s.block_on(async {
        sqlx::query("INSERT INTO op_log (message) VALUES ('after_each')")
            .execute(s.pool())
            .await
            .unwrap();
    });
}

static MY_HOOKS: HookFns<DbSuite> = HookFns {
    setup_suite: Some(db_setup_suite),
    teardown_suite: Some(db_teardown_suite),
    before_each: Some(db_before_each),
    after_each: Some(db_after_each),
};

static MY_CASES: &[Case<DbSuite>] = cases_fn![
    DbSuite =>
    test_insert_alice => case_insert_alice,
    test_insert_bob => case_insert_bob,
    test_bump_alice_version => case_bump_alice_version,
    test_assert_final_counts => case_assert_final_counts,
];

cargo_case_tests_with_hooks!(
    DbSuite::default(),
    MY_CASES,
    MY_HOOKS,
    [test_insert_alice, test_insert_bob, test_bump_alice_version, test_assert_final_counts]
);

fn main() {}
