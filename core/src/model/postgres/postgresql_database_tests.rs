/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003, 2004, 2010, 2011, 2013-2020 inclusive, and 2023-2023 inclusive, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
/// Created this file to reduce the size of postgresql_database.rs, so the IDE can process things
/// faster.
use crate::model::boolean_attribute::BooleanAttribute;
use crate::model::database::DataType;
use crate::model::database::Database;
use crate::model::entity::Entity;
use crate::model::postgres::postgresql_database::*;
use crate::model::postgres::*;
use crate::model::relation_to_local_entity::RelationToLocalEntity;
use crate::model::relation_to_remote_entity::RelationToRemoteEntity;
use crate::util::Util;
use anyhow::anyhow;
use chrono::Utc;
// use futures::executor::block_on;
use sqlx::postgres::*;
// Specifically omitting sql::Error from use statements so that it is *clearer* which Error type is
// in use, in the code.
use sqlx::{Column, PgPool, Postgres, Row, Transaction, ValueRef};
use std::collections::HashSet;
// use std::fmt::format;
use tracing::*;
// use tracing_subscriber::FmtSubscriber;

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;

    const QUANTITY_TYPE_NAME: &str = "length";
    const RELATION_TYPE_NAME: &str = "someRelationToEntityTypeName";
    const RELATED_ENTITY_NAME: &str = "someRelatedEntityName";

    /// This fn is used in important (informative) commented lines elsewhere.
    fn db_query_for_test1<'a>(
        rt: &tokio::runtime::Runtime,
        pool: &sqlx::Pool<Postgres>,
        transaction: Option<&'a mut Transaction<'a, Postgres>>,
        sql: &str, // ) -> Result<(Vec<Vec<DataType>>, Option<&mut Transaction<Postgres>>), String> {
    ) -> Result<(), String> {
        // let mut results: Vec<Vec<DataType>> = Vec::new();
        // let types_vec: Vec<&str> = types.split_terminator(",").collect();
        // let mut row_counter = 0;
        // let future = sqlx::query(format!(sql).as_str()).execute(&pool);
        // let future = sqlx::query(sql).execute(&pool);
        // let result = rt.block_on(future)?;
        // debug!("Query result:  {:?}", result);
        let query = sqlx::query(sql);

        let map = query.map(|_sqlx_row: PgRow| {
            //do stuff to capture results
        });
        if transaction.is_some() {
            let tx = transaction.unwrap();
            let future = map.fetch_all(tx);
            /*self.*/
            rt.block_on(future).unwrap();
        } else {
            let future = map.fetch_all(/*&self.*/ pool);
            /*self.*/
            rt.block_on(future).unwrap();
        }

        // Ok((results, transaction))
        // Ok(results)
        Ok(())
    }

    // THOUGH COMMENTED, THIS IS KEPT HERE because it would be used when uncommenting lines in
    // the following method (test_compile_problem_with_non_reference_transaction_parameters)
    // per its comments, to demonstrate some compilation errors.
    // fn db_query_for_test2<'a>(
    //     // &'a self,
    //     rt: &tokio::runtime::Runtime,
    //     pool: &sqlx::Pool<Postgres>,
    //     transaction: Option<&'a mut sqlx::Transaction<'a, sqlx::Postgres>>,
    //     // transaction: &Option<&Transaction<Postgres>>,
    //     sql: &str
    //     // ) -> Result<(Vec<Vec<DataType>>, Option<&mut Transaction<Postgres>>), String> {
    // ) -> Result<Option<&'a mut sqlx::Transaction<'a, sqlx::Postgres>>, String> {
    //     // let mut results: Vec<Vec<DataType>> = Vec::new();
    //     // let types_vec: Vec<&str> = types.split_terminator(",").collect();
    //     // let mut row_counter = 0;
    //     // let future = sqlx::query(format!(sql).as_str()).execute(&pool);
    //     // let future = sqlx::query(sql).execute(&pool);
    //     // let result = rt.block_on(future)?;
    //     // debug!("Query result:  {:?}", result);
    //     let query = sqlx::query(sql);
    //
    //     let map = query
    //         .map(|sqlx_row: PgRow| {
    //             //do stuff to capture results
    //         });
    //     if transaction.is_some() {
    //         let mut tx = transaction.unwrap();
    //         let future = map.fetch_all(tx);
    //         /*self.*/rt.block_on(future).unwrap();
    //     } else {
    //         let future = map.fetch_all(/*&self.*/pool);
    //         /*self.*/rt.block_on(future).unwrap();
    //     }
    //
    //     // Ok((results, transaction))
    //     // Ok(results)
    //     Ok(transaction)
    // }

    #[test]
    /// Some lines have to be uncommented, to see the compile errors that this is meant to
    /// demonstrate.  See comments below for details.
    //%%
    fn test_compile_problem_with_non_reference_transaction_parameters() {
        Util::initialize_tracing();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let connect_str = format!(
            "postgres://{}:{}@localhost/{}",
            Util::TEST_USER,
            Util::TEST_PASS,
            "om_t1"
        );
        let future = PgPoolOptions::new()
            .max_connections(10)
            .connect(connect_str.as_str());
        let pool = rt.block_on(future).unwrap();
        // let name: String =
        //     format!("test_rollback_temporary_entity").to_string(); //_{}", rand_num.to_string()).to_string();
        // db.drop(None, "table", name.as_str()).unwrap();
        // (Using the _ in _transaction only to get rid of compiler warning. Remove it as needed.)
        let _transaction = rt.block_on(pool.begin()).unwrap();
        // let mut id = db
        //     .create_entity(Some(&mut tx), name.as_str(), None, None)
        //     .expect(format!("Failed to create entity with name: {name}").as_str());

        // %%$%% Uncommenting these 2 lines gets one kind of transaction error (something about the
        // transaction being moved in the first line, and so not available to the second line).
        // db_query_for_test1(&rt, &pool, Some(&mut transaction), "select count(*) from pg_aggregate");
        // db_query_for_test1(&rt, &pool, None, "select count(*) from pg_views");

        // %%$%%Commenting out the 2 lines just above, and un-commenting these, gets these errors. But I
        // can't use .as_ref() or .as_mut() because that violates trait constraints or something.
        // Unless one of those (or Copy?) is added to the struct Transaction later?
        /*
            error[E0382]: use of moved value: `transaction`
            --> src/model/postgresql_database.rs:6243:12
                |
                6214 |         transaction: Option<&'a mut sqlx::Transaction<'a, sqlx::Postgres>>,
            |         ----------- move occurs because `transaction` has type `Option<&mut sqlx::Transaction<'_, sqlx::Postgres>>`, which doe
                ...
                6233 |             let mut tx = transaction.unwrap();
            |                          ----------- -------- `transaction` moved due to this method call
                |                          |
            |                          help: consider calling `.as_ref()` or `.as_mut()` to borrow the type's contents
                ...
                6243 |         Ok((transaction))
                    |            ^^^^^^^^^^^^^ value used here after move
            |
            note: `Option::<T>::unwrap` takes ownership of the receiver `self`, which moves `transaction`
            --> /usr/obj/ports/rust-1.68.0/rustc-1.68.0-src/library/core/src/option.rs:820:25

            error[E0597]: `transaction` does not live long enough
                --> src/model/postgresql_database.rs:6275:49
                |
                6275 |             db_query_for_test2(&rt, &pool, Some(&mut transaction), "select count(*) from pg_aggregate").unwrap();
            |                                                 ^^^^^^^^^^^^^^^^ borrowed value does not live long enough
            6276 |         db_query_for_test2(&rt, &pool, None, "select count(*) from pg_views");
            6277 |     }
        |     -
        |     |
        |     `transaction` dropped here while still borrowed
        |     borrow might be used here, when `transaction` is dropped and runs the `Drop` code for type `sqlx::Transaction`
             */
        // let transaction: Option<&mut sqlx::Transaction<sqlx::Postgres>> =
        //     db_query_for_test2(&rt, &pool, Some(&mut transaction), "select count(*) from pg_aggregate").unwrap();
        // db_query_for_test2(&rt, &pool, None, "select count(*) from pg_views");
    }

    #[test]
    fn test_basic_sql_connectivity_with_async_and_tokio() {
        // To reproduce and fix hangs, by using either 1) rt.block_on(); or 2) #[tokio::main] or
        // #[tokio::test], with async fn and future.await, but not mixing the 2 approaches!
        Util::initialize_tracing();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let connect_str = format!(
            "postgres://{}:{}@localhost/{}",
            Util::TEST_USER,
            Util::TEST_PASS,
            "om_t1"
        );
        let future = PgPoolOptions::new()
            .max_connections(1)
            // .connect(connect_str.as_str()).await?;
            .connect(connect_str.as_str());
        let pool = rt.block_on(future).unwrap();

        // idea: could get the next lines to work also and show something useful re the current setting??:
        // let future = sqlx::query("show transaction isolation level").execute(&pool);
        // let x = rt.block_on(future).unwrap();
        // debug!("in test_basic_sql_connectivity_with_async_and_tokio: Query result re transaction isolation lvl?:  {:?}", x);
        // %%Search for related cmts w/ "isolation".

        for c in 1..=150 {
            debug!(
                "in test_basic_sql_connectivity_with_async_and_tokio: before, {}",
                c
            );

            // hung after 1-4 iterations, when block_on didn't have "rt.":
            let sql: String = "DROP table IF EXISTS test_doesnt_exist CASCADE".to_string();
            let future = sqlx::query(sql.as_str()).execute(&pool);
            let x: Result<PgQueryResult, sqlx::Error> = /*%%: i32 asking compiler or println below*/ rt.block_on(future);
            //using next line instead avoided the problem!
            // let x: Result<PgQueryResult, sqlx::Error> = /*%%: i32 asking compiler or println below*/ future.await;
            if let Err(e) = x {
                panic!("FAILURE 1: {}", e.to_string());
            } else {
                debug!(
                    "in test_basic_sql_connectivity_with_async_and_tokio: ok {}: {}, {:?}",
                    c,
                    &sql,
                    x.unwrap()
                );
            }

            // Hung similarly to above section, after ~ 6 iterations.  Has valid output.
            // let sql: String = "select count(*) from entity".to_string();
            let sql: String = "select count(1) from pg_catalog.pg_user;".to_string();
            let query = sqlx::query(sql.as_str());
            let future = query
                .map(|sqlx_row: PgRow| {
                    let decode_mbe = sqlx_row.try_get(0);
                    let val: i64 = decode_mbe.unwrap();
                    debug!("in test_basic_sql_connectivity_with_async_and_tokio: in db_query {}: val is {} .", c, val);
                })
                .fetch_all(&pool);
            let res = rt.block_on(future).unwrap();
            debug!(
                "in test_basic_sql_connectivity_with_async_and_tokio: result vec (length {}): {:?}",
                res.len(),
                res
            );
            for (c, e) in res.iter().enumerate() {
                debug!(
                    "in test_basic_sql_connectivity_with_async_and_tokio:  vec element {}: {:?}",
                    c, e
                );
            }

            // hung at 8-9 iterations, similarly:
            let future = sqlx::query_as("select count(*) from pg_catalog.pg_user;")
                .bind(150_i64)
                .fetch_one(&pool);
            let row: (i64,) = rt.block_on(future).unwrap();
            //using next line instead avoids the problem, before I used rt! (see more detailed comment above.)
            // let row: (i64, ) = future.await.unwrap();
            debug!(
                "in test_basic_sql_connectivity_with_async_and_tokio: in query {}: {:?} and {}",
                c, row, row.0
            );
        }
    }

    #[test]
    fn test_set_user_preference_and_get_user_preference() {
        Util::initialize_tracing();
        let db: PostgreSQLDatabase = Util::initialize_test_db().unwrap();

        let mut tx = db.begin_trans().unwrap();
        let tx = &Some(&mut tx);
        let pref_name = "xyznevercreatemeinreallife";
        assert!(db
            .get_user_preference_boolean(tx, pref_name, None)
            .unwrap()
            .is_none());

        assert_eq!(
            db.get_user_preference_boolean(tx, pref_name, Some(true))
                .unwrap(),
            Some(true)
        );
        db.set_user_preference_boolean(tx, pref_name, false)
            .unwrap();
        assert_eq!(
            db.get_user_preference_boolean(tx, pref_name, Some(true))
                .unwrap(),
            Some(false)
        );

        let pref_name2 = "xyz2";
        assert!(db
            .get_user_preference_entity_id(tx, pref_name2, None)
            .unwrap()
            .is_none());
        assert_eq!(
            db.get_user_preference_entity_id(tx, pref_name2, Some(0))
                .unwrap(),
            Some(0)
        );
        db.set_user_preference_entity_id(tx, pref_name2, db.get_system_entity_id(tx).unwrap())
            .unwrap();
        assert_eq!(
            db.get_user_preference_entity_id(tx, pref_name2, Some(0))
                .unwrap(),
            Some(db.get_system_entity_id(tx).unwrap())
        );
        // no need to db.rollback_trans(), because that is automatic when tx goes out of scope, per sqlx docs.
    }

    #[test]
    /// yes it actually was failing when written, in my use of Sqlx somehow, before I learned
    /// that you have to pass the transaction as the executor (ie, instead of the pool), for a sql
    /// operation to be included in a transaction. Not just start the transaction.
    ///
    /// As of 2023-05-22, it is still failing because transactions are not yet used correctly
    /// inside fn db_action and fn db_query.  Need to uncomment a line and recomment another, but
    /// that gets compiler errors. Hmm.
    fn test_rollback_and_commit() {
        Util::initialize_tracing();
        let db: PostgreSQLDatabase = Util::initialize_test_db().unwrap();
        let rand_num = randlib::Rand::new().rand_u32();
        let name: String =
            format!("test_rollback_temporary_entity_{}", rand_num.to_string()).to_string();
        // (make sure to avoid confusion with another test or earlier run somehow using the same table name?)
        db.drop(&None, "table", name.as_str()).unwrap();
        let mut tx = db.begin_trans().unwrap();
        let mut id = db
            .create_entity(&Some(&mut tx), name.as_str(), None, None)
            .expect(format!("Failed to create entity with name: {}", name).as_str());
        assert!(db
            .entity_key_exists(&Some(&mut tx), id, true)
            .expect(format!("Found: {}", id).as_str()));
        db.rollback_trans(tx).unwrap();
        assert!(!db
            .entity_key_exists(&None, id, true)
            .expect(format!("Found: {}", id).as_str()));

        // this time with an implied rollback, as sqlx docs say when a transaction goes out of scope
        // without a commit, it is implicitly rolled back.
        {
            let mut tx = db.begin_trans().unwrap();
            id = db
                .create_entity(&Some(&mut tx), name.as_str(), None, None)
                .expect(format!("Failed to create: {}", name).as_str());
        }
        assert!(!db
            .entity_key_exists(&None, id, true)
            .expect(format!("Found: {}", id).as_str()));

        // this time with a commit, not a rollback
        let mut tx = db.begin_trans().unwrap();
        id = db
            .create_entity(&Some(&mut tx), name.as_str(), None, None)
            .expect(format!("Failed to create entity w name: {}", name).as_str());
        assert!(db
            .entity_key_exists(&Some(&mut tx), id, true)
            .expect(format!("Failed to find: {}", id).as_str()));
        db.commit_trans(tx).unwrap();
        assert!(db
            .entity_key_exists(&None, id, true)
            .expect(format!("Failed to find: {}", id).as_str()));
    }

    /// Intended for use in tests that I might want to send in to reproduce a bug.
    /// Takes sql like "select count(1) from <table>".
    // fn sqlx_get_int(pool: &sqlx::Pool<Postgres>, rt: &tokio::runtime::Runtime, sql: &str) -> i64 {
    // fn sqlx_get_int(tx: &mut sqlx::Executor, rt: &tokio::runtime::Runtime, sql: &str) -> i64 {
    // fn sqlx_do_query<'a, E>(executor: E, rt: &tokio::runtime::Runtime, sql: &str)
    //     where E: sqlx::Executor<'a, Database = Postgres>,
    // fn sqlx_get_int(tx: &mut Transaction<Postgres>, rt: &tokio::runtime::Runtime, sql: &str) -> i64 {
    // see comment (question) on below method sqlx_do_query.
    fn sqlx_get_int<'a, E>(executor: E, rt: &tokio::runtime::Runtime, sql: &str) -> i64
    where
        E: sqlx::Executor<'a, Database = Postgres>,
    {
        // let future = sqlx::query_as(sql.as_str()).bind(150_i64).fetch_one(&pool);
        // let row: (i64,) = rt.block_on(future).expect(format!("Failed sql: {count_sql}").as_str());
        // let count: i64 = row.0;
        let mut count: i64 = -1;
        let future = sqlx::query(sql)
            .map(|sqlx_row: PgRow| {
                count = sqlx_row
                    .try_get(0)
                    .expect(format!("Failed at: {} and getting val.", sql).as_str());
                debug!("in sqlx_get_int: in db_query {}: val is {} .", sql, count);
            })
            // .fetch_all(pool);
            .fetch_all(executor);
        rt.block_on(future)
            .expect(format!("Failed sql: {}", sql).as_str());
        count
    }
    // /// Intended for use in tests that I might want to send in to reproduce a bug.
    // /// Takes sql like "select count(1) from <table>".
    // fn sqlx_get_int_no_tx(executor: &sqlx::Pool<Postgres>, rt: &tokio::runtime::Runtime, sql: &str) -> i64 {
    // // fn sqlx_get_int_no_tx(executor: &mut dyn sqlx::Executor<Database = Postgres>, rt: &tokio::runtime::Runtime, sql: &str) -> i64 {
    //     // let future = sqlx::query_as(sql.as_str()).bind(150_i64).fetch_one(&pool);
    //     // let row: (i64,) = rt.block_on(future).expect(format!("Failed sql: {count_sql}").as_str());
    //     // let count: i64 = row.0;
    //     let mut count: i64 = -1;
    //     let future = sqlx::query(sql)
    //         .map(|sqlx_row: PgRow| {
    //             count = sqlx_row
    //                 .try_get(0)
    //                 .expect(format!("Failed at: {sql} and getting val.").as_str());
    //             debug!("in sqlx_get_int_no_tx: in db_query {sql}: val is {} .", count);
    //         })
    //         .fetch_all(executor);
    //         // .fetch_all(tx);
    //     rt.block_on(future)
    //         .expect(format!("Failed sql: {sql}").as_str());
    //     count
    // }
    /// For a test that does an insert statement.  The executor should be either a db pool
    /// or transaction.
    // fn sqlx_do_query(pool: &sqlx::Pool<Postgres>, rt: &tokio::runtime::Runtime, sql: &str) {
    // fn sqlx_do_query(executor: &mut Transaction<Postgres>, rt: &tokio::runtime::Runtime, sql: &str) {
    // fn sqlx_do_query(executor: Box<&mut dyn sqlx::Executor<Database = Postgres>>, rt: &tokio::runtime::Runtime, sql: &str) {
    // fn sqlx_do_query(executor: &mut sqlx::Executor<Database = Postgres>, rt: &tokio::runtime::Runtime, sql: &str) {
    // Why does below line not work (compile errors), but the 2 lines below it do work (as mimicked from sqlx:;query.execute(...))?
    // fn sqlx_do_query<'a>(executor: sqlx::Executor<'a, Database = Postgres>, rt: &tokio::runtime::Runtime, sql: &str) {
    fn sqlx_do_query<'a, E>(executor: E, rt: &tokio::runtime::Runtime, sql: &str)
    where
        E: sqlx::Executor<'a, Database = Postgres>,
    {
        let x: PgQueryResult = rt
            .block_on(sqlx::query(sql).execute(executor))
            .expect(format!("Failed sql: {}", sql).as_str());
        debug!("in sqlx_do_query: inserted: {}: {:?}", sql, x);
    }

    #[test]
    ///yes it actually was failing when written, in my use of Sqlx somehow.%%finish cmt--what fixed?
    fn test_rollback_and_commit_with_less_helper_code() {
        Util::initialize_tracing();
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let connect_str = format!(
            "postgres://{}:{}@localhost/{}",
            Util::TEST_USER,
            Util::TEST_PASS,
            "om_t1"
        );
        //%%$% why does the insert sql get "PoolTimedOut" if .max_connections is 1 instead of 10??
        //(Is similar to similar problem w/ .max_connections noted elsewhere?)
        let future = PgPoolOptions::new()
            .max_connections(10)
            .connect(connect_str.as_str());
        let pool = rt.block_on(future).unwrap();

        // let rand_num = randlib::Rand::new().rand_u16();
        // let table_name: String = format!("test_rollback_temp_{}", rand_num.to_string()).to_string();
        let table_name: String = format!("test_rollback_temp").to_string();

        let sql = format!("DROP table IF EXISTS {}", table_name);
        let mut x: PgQueryResult = rt
            .block_on(sqlx::query(sql.as_str()).execute(&pool))
            .expect(format!("Error from sql: {}", sql).as_str());
        debug!(
            "in test_rollback_and_commit_with_less_helper_code: dropped table if exists: {}: {:?}",
            &sql, x
        );

        let sql = format!("create table {} (datum varchar(99) NOT NULL) ", table_name);
        x = rt
            .block_on(sqlx::query(sql.as_str()).execute(&pool))
            .expect(format!("Error from sql: {}", sql).as_str());
        debug!(
            "in test_rollback_and_commit_with_less_helper_code: created table: {}: {:?}",
            &sql, x
        );

        let count_sql = format!("select count(*) from {}", table_name);
        let mut count: i64 = sqlx_get_int(&pool, &rt, count_sql.as_str());
        // let mut count = 0;
        // let mut count: i64 = sqlx_get_int(&mut tx, &rt, count_sql.as_str());
        assert_eq!(count, 0);
        debug!(
            "test_rollback_and_commit_with_less_helper_code: count before insertion: {}",
            count
        );

        // now we have a table w/ no rows, so see about a transaction and its effects.
        let mut tx: Transaction<Postgres> = rt.block_on(pool.begin()).unwrap();

        let insert_sql = format!("insert into {} (datum) VALUES ('something')", table_name);
        // sqlx_do_query(&pool, &rt, insert_sql.as_str());
        // sqlx_do_query(Box::new(&mut tx), &rt, insert_sql.as_str());
        sqlx_do_query(&mut tx, &rt, insert_sql.as_str());
        // count = sqlx_get_int(&pool, &rt, count_sql.as_str());
        count = sqlx_get_int(&mut tx, &rt, count_sql.as_str());
        assert_eq!(count, 1);
        debug!(
            "in test_rollback_and_commit_with_less_helper_code: count after insertion: {}",
            count
        );
        rt.block_on(tx.rollback()).unwrap();

        // count = sqlx_get_int(&pool, &rt, count_sql.as_str());
        count = sqlx_get_int(&pool, &rt, count_sql.as_str());
        debug!("in test_rollback_and_commit_with_less_helper_code: count after rollback should be 0: {}", count);

        //%%$%this fails, so try?: xnew version of sqlx w what changes, xmore web searches, reddit?, file an issue (filed 20230406)?
        //%%$%why doesnt the rollback, implied OR explicit, do anything? due to xactn isolation or...??
        //AFTER FIXING, see all the places with "rollbacketc%%" (2) and address them.
        //could: Search for related cmts w/ "isolation".
        assert_eq!(count, 0);

        // this time with an implied rollback, as sqlx docs say when a transaction goes out of scope
        // without a commit, it is implicitly rolled back.
        {
            // now we have a table w/ no rows, so see about a transaction and its effects.
            let mut tx = rt.block_on(pool.begin()).unwrap();
            // sqlx_do_query(&pool, &rt, insert_sql.as_str());
            // sqlx_do_query(Box::new(&mut tx), &rt, insert_sql.as_str());
            sqlx_do_query(&mut tx, &rt, insert_sql.as_str());
            // count = sqlx_get_int(&pool, &rt, count_sql.as_str());
            count = sqlx_get_int(&mut tx, &rt, count_sql.as_str());
            assert_eq!(count, 1);
            debug!(
                "in test_rollback_and_commit_with_less_helper_code: count after insert: {}",
                count
            );

            rt.block_on(tx.rollback()).unwrap();
        }
        // count = sqlx_get_int(&pool, &rt, count_sql.as_str());
        count = sqlx_get_int(&pool, &rt, count_sql.as_str());
        debug!("in test_rollback_and_commit_with_less_helper_code: count after implied rollback should be 0: {}", count);
        assert_eq!(count, 0);

        // this time with a commit, not a rollback
        let mut tx = rt.block_on(pool.begin()).unwrap();

        // sqlx_do_query(&pool, &rt, insert_sql.as_str());
        // sqlx_do_query(Box::new(&mut tx), &rt, insert_sql.as_str());
        sqlx_do_query(&mut tx, &rt, insert_sql.as_str());
        // count = sqlx_get_int(&pool, &rt, count_sql.as_str());
        count = sqlx_get_int(&mut tx, &rt, count_sql.as_str());
        assert_eq!(count, 1);
        debug!(
            "in test_rollback_and_commit_with_less_helper_code: count after insert: {}",
            count
        );

        rt.block_on(tx.commit()).unwrap();

        // count = sqlx_get_int(&pool, &rt, count_sql.as_str());
        count = sqlx_get_int(&pool, &rt, count_sql.as_str());
        debug!("in test_rollback_and_commit_with_less_helper_code: count after commit should still be 1: {}", count);
        assert_eq!(count, 1);
    }

    #[test]
    fn test_database_version_table_has_right_data() {
        Util::initialize_tracing();
        let db: PostgreSQLDatabase = Util::initialize_test_db().unwrap();
        let version_table_exists: bool = db
            .does_this_exist(
                &None,
                "select count(1) from pg_class where relname='om_db_version'",
                true,
            )
            .unwrap();
        assert!(version_table_exists);
        let results = db
            .db_query_wrapper_for_one_row(&None, "select version from om_db_version", "Int")
            .unwrap();
        assert_eq!(results.len(), 1);
        if let Some(DataType::Smallint(db_ver)) = results.get(0).unwrap() {
            assert_eq!(
                *db_ver,
                PostgreSQLDatabase::SCHEMA_VERSION,
                "db_ver ({}) != PostgreSQLDatabase.SCHEMA_VERSION ({}).",
                db_ver,
                PostgreSQLDatabase::SCHEMA_VERSION
            );
        } else {
            panic!("Unexpected value: {:?}.", results.get(0).unwrap());
        }
    }

    //%%
    // fn create_test_text_attribute_with_one_entity(db: PostgreSQLDatabase, in_parent_id: i64, in_valid_on_date: Option<i64> /*= None*/) -> i64 {
    //     let attr_type_id: i64 = db.create_entity("textAttributeTypeLikeSsn");
    //     let default_date: i64 = Utc::now().timestamp_millis();
    //     let valid_on_date: Option<i64> = in_valid_on_date;
    //     let observation_date: i64 = default_date;
    //     let text = "some test text";
    //     let text_attribute_id: i64 = db.create_text_attribute(&None, in_parent_id, attr_type_id, text, valid_on_date, observation_date).unwrap();
    //     // and verify it:
    //     let ta: TextAttribute = new TextAttribute(m_db, text_attribute_id);
    //     assert(ta.get_parent_id() == in_parent_id)
    //     assert(ta.get_text == text)
    //     assert(ta.get_attr_type_id() == attr_type_id)
    //     if in_valid_on_date.isEmpty {
    //         assert(ta.get_valid_on_date().isEmpty)
    //     } else {
    //         assert(ta.get_valid_on_date().get == in_valid_on_date.get)
    //     }
    //     assert(ta.get_observation_date() == observation_date)
    //
    //     text_attribute_id
    // }
    //
    // fn createTestDateAttributeWithOneEntity(in_parent_id: i64) -> i64 {
    //     let attr_type_id: i64 = m_db.create_entity("dateAttributeType--likeDueOn");
    //     let date: i64 = System.currentTimeMillis;
    //     let dateAttributeId: i64 = m_db.create_date_attribute(in_parent_id, attr_type_id, date);
    //     let ba: DateAttribute = new DateAttribute(m_db, dateAttributeId);
    //     assert(ba.get_parent_id() == in_parent_id)
    //     assert(ba.getDate == date)
    //     assert(ba.get_attr_type_id() == attr_type_id)
    //     dateAttributeId
    // }
    //
    // fn createTestBooleanAttributeWithOneEntity(in_parent_id: i64, valIn: bool, in_valid_on_date: Option<i64> = None, observation_date_in: i64) -> i64 {
    //     let attr_type_id: i64 = m_db.create_entity("boolAttributeType-like-isDone");
    //     let booleanAttributeId: i64 = m_db.create_boolean_attribute(in_parent_id, attr_type_id, valIn, in_valid_on_date, observation_date_in);
    //     let ba = new BooleanAttribute(m_db, booleanAttributeId);
    //     assert(ba.get_attr_type_id() == attr_type_id)
    //     assert(ba.get_boolean == valIn)
    //     assert(ba.get_valid_on_date() == in_valid_on_date)
    //     assert(ba.get_parent_id() == in_parent_id)
    //     assert(ba.get_observation_date() == observation_date_in)
    //     booleanAttributeId
    // }
    //
    // fn createTestFileAttributeAndOneEntity(inParentEntity: Entity, inDescr: String, addedKiloBytesIn: Int, verifyIn: bool = true) -> FileAttribute {
    //     let attr_type_id: i64 = m_db.create_entity("fileAttributeType");
    //     let file: java.io.File = java.io.File.createTempFile("om-test-file-attr-", null);
    //     let mut writer: java.io.FileWriter = null;
    //     let mut verificationFile: java.io.File = null;
    //     try {
    //         writer = new java.io.FileWriter(file)
    //         writer.write(addedKiloBytesIn + "+ kB file from: " + file.getCanonicalPath + ", created " + new java.util.Date())
    //         let mut nextInteger: i64 = 1;
    //         for (i: Int <- 1 to (1000 * addedKiloBytesIn)) {
    //             // there's a bug here: files aren't the right size (not single digits being added) but oh well it's just to make some file.
    //             writer.write(nextInteger.toString)
    //             if i % 1000 == 0 { nextInteger += 1 }
    //         }
    //         writer.close();
    //
    //         // sleep is so we can see a difference between the 2 dates to be saved, in later assertion.
    //         let sleepPeriod = 5;
    //         Thread.sleep(sleepPeriod);
    //         let size = file.length();
    //         let mut inputStream: java.io.FileInputStream = null;
    //         let mut fa: FileAttribute = null;
    //         try {
    //             inputStream = new java.io.FileInputStream(file)
    //             fa = inParentEntity.addFileAttribute(attr_type_id, inDescr, file)
    //         } finally {
    //             if inputStream != null { inputStream.close() }
    //     }
    //
    //     if verifyIn {
    //         // this first part is just testing DB consistency from add to retrieval, not the actual file:
    //         assert(fa.get_parent_id() == inParentEntity.get_id)
    //         assert(fa.get_attr_type_id() == attr_type_id)
    //         assert((fa.getStoredDate - (sleepPeriod - 1)) > fa.getOriginalFileDate)
    //         // (easily fails if the program pauses when debugging):
    //         assert((fa.getStoredDate - 10000) < fa.getOriginalFileDate)
    //         assert(file.lastModified() == fa.getOriginalFileDate)
    //         assert(file.length() == fa.getSize)
    //         assert(file.getCanonicalPath == fa.getOriginalFilePath)
    //         assert(fa.getDescription == inDescr)
    //         assert(fa.getSize == size)
    //         // (startsWith, because the db pads with characters up to the full size)
    //         assert(fa.getReadable && fa.getWritable && !fa.getExecutable)
    //
    //         // now ck the content itself
    //         verificationFile = File.createTempFile("om-fileattr-retrieved-content-", null)
    //         fa.retrieveContent(verificationFile)
    //         assert(verificationFile.canRead == fa.getReadable)
    //         assert(verificationFile.canWrite == fa.getWritable)
    //         assert(verificationFile.canExecute == fa.getExecutable)
    //     }
    //     fa
    // } finally {
    //     if verificationFile != null { verificationFile.delete() }
    //     if writer != null { writer.close() }
    //     if file != null { file.delete() }
    //     }
    // }

    fn create_test_relation_to_local_entity_with_one_entity(
        in_entity_id: i64,
        in_rel_type_id: i64,
        in_valid_on_date: Option<i64>, /*= None*/
    ) -> i64 {
        Util::initialize_tracing();
        let db: PostgreSQLDatabase = Util::initialize_test_db().unwrap();
        // idea: could use here instead: db.create_entityAndRelationToLocalEntity
        let related_entity_id: i64 = db
            .create_entity(&None, RELATED_ENTITY_NAME, None, None)
            .unwrap();
        // let valid_on_date: Option<i64> = if in_valid_on_date.isEmpty { None } else { in_valid_on_date };
        let observation_date: i64 = Utc::now().timestamp_millis();
        0_i64

        //%%finish when attrs in place again:
        // let id = db.create_relation_to_local_entity(&None, in_rel_type_id,
        //                                             in_entity_id, related_entity_id,
        //                                             in_valid_on_date, observation_date).get_id;
        //
        // // and verify it:
        // let rtle: RelationToLocalEntity = new RelationToLocalEntity(m_db, id, in_rel_type_id, in_entity_id, related_entity_id);
        // if in_valid_on_date.isEmpty {
        //     assert(rtle.get_valid_on_date().isEmpty)
        // } else {
        //     let inDt: i64 = in_valid_on_date.get;
        //     let gotDt: i64 = rtle.get_valid_on_date().get;
        //     assert(inDt == gotDt)
        // }
        // assert(rtle.get_observation_date() == observation_date)
        // related_entity_id
    }

    #[test]
    fn escape_quotes_etc_allow_updating_db_with_single_quotes() {
        Util::initialize_tracing();
        let db: PostgreSQLDatabase = Util::initialize_test_db().unwrap();
        let name = "This ' name contains a single-quote.";
        let mut tx = db.begin_trans().unwrap();
        let tx = &Some(&mut tx);
        //on a create:
        let entity_id: i64 = db.create_entity(tx, name, None, None).unwrap();
        let new_name = db.get_entity_name(tx, entity_id);
        assert_eq!(name, new_name.unwrap().unwrap().as_str());

        //and on an update:
        //%%FINISH WHEN ATTRS and other above cmted fns are in place
        // let text_attribute_id: i64 = create_test_text_attribute_with_one_entity(db entity_id);
        // let a_text_value = "as'dfjkl";
        // let ta = new TextAttribute(m_db, text_attribute_id);
        // let (pid1, atid1) = (ta.get_parent_id(), ta.get_attr_type_id());
        // m_db.update_text_attribute(text_attribute_id, pid1, atid1, a_text_value, Some(123), 456)
        // // have to create new instance to re-read the data:
        // let ta2 = new TextAttribute(m_db, text_attribute_id);
        // let txt2 = ta2.get_text;
        //
        // assert(txt2 == a_text_value)
    }

    #[test]
    /// With transaction rollback, this should create one new entity, work right, then have none.
    fn test_entity_creation_and_update() {
        Util::initialize_tracing();
        let db: PostgreSQLDatabase = Util::initialize_test_db().unwrap();
        let name = "test: org.onemodel.PSQLDbTest.entitycreation...";
        let mut tx1 = db.begin_trans().unwrap();
        let tx = &Some(&mut tx1);

        let entity_count_before_creating: u64 = db.get_entity_count(tx).unwrap();
        let entities_only_first_count: u64 =
            db.get_entities_only_count(tx, false, None, None).unwrap();

        let id: i64 = db.create_entity(tx, name, None, None).unwrap();
        let new_name = db.get_entity_name(tx, id);
        assert_eq!(name, new_name.unwrap().unwrap().as_str());
        let entity_count_after_1st_create = db.get_entity_count(tx).unwrap();
        let entities_only_new_count = db.get_entities_only_count(tx, false, None, None).unwrap();
        if entity_count_before_creating + 1 != entity_count_after_1st_create
            || entities_only_first_count + 1 != entities_only_new_count
        {
            panic!("get_entity_count() after adding doesn't match prior count+1! Before: {} and {}, after: {} and {}.",
                   entity_count_before_creating,  entities_only_new_count, entity_count_after_1st_create, entities_only_new_count);
        }
        assert!(db.entity_key_exists(tx, id, true).unwrap());

        let new_name = "test: ' org.onemodel.PSQLDbTest.entityupdate...";
        db.update_entity_only_name(tx, id, new_name).unwrap();
        // have to create new instance to re-read the data:
        let mut updated_entity = Entity::new2(Box::new(&db as &dyn Database), tx, id).unwrap();
        let name3 = updated_entity.get_name(tx).unwrap().as_str();
        assert_eq!(name3, new_name);

        assert!(db.entity_only_key_exists(tx, id).unwrap());
        db.rollback_trans(tx1).unwrap();

        // now should not exist
        let entity_count_after_rollback = db.get_entity_count(&None).unwrap();
        assert_eq!(entity_count_after_rollback, entity_count_before_creating);
        assert!(!db.entity_key_exists(&None, id, true).unwrap());
    }

    #[test]
    fn find_id_which_is_not_key_of_any_entity() {
        Util::initialize_tracing();
        let db: PostgreSQLDatabase = Util::initialize_test_db().unwrap();
        // let mut tx1 = db.begin_trans().unwrap();
        // let tx = &Some(&mut tx1);

        assert!(!db.entity_key_exists(
            &None,
            db.find_id_which_is_not_key_of_any_entity(&None).unwrap(),
            true
        ).unwrap());
    }
}
