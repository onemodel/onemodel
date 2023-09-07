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
// use crate::model::boolean_attribute::BooleanAttribute;
use crate::model::database::DataType;
use crate::model::database::Database;
use crate::model::entity::Entity;
use crate::model::postgres::postgresql_database::*;
// use crate::model::postgres::*;
// use crate::model::relation_to_local_entity::RelationToLocalEntity;
// use crate::model::relation_to_remote_entity::RelationToRemoteEntity;
use crate::model::relation_type::RelationType;
use crate::util::Util;
// use anyhow::anyhow;
use chrono::Utc;
// use futures::executor::block_on;
use sqlx::postgres::*;
// Specifically omitting sql::Error from use statements so that it is *clearer* which Error type is
// in use, in the code.
// use sqlx::{Column, PgPool, Postgres, Row, Transaction, ValueRef};
use sqlx::{Postgres, Row, Transaction};
// use std::collections::HashSet;
// use std::fmt::format;
use tracing::*;
// use tracing_subscriber::FmtSubscriber;

#[cfg(test)]
mod tests {
    // use super::super::*;
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

        assert!(!db
            .entity_key_exists(
                &None,
                db.find_id_which_is_not_key_of_any_entity(&None).unwrap(),
                true
            )
            .unwrap());
    }

    #[test]
    fn entity_only_key_exists_should_not_find_relation_to_local_entity_record() {
        Util::initialize_tracing();
        let db: PostgreSQLDatabase = Util::initialize_test_db().unwrap();
        // let mut tx1 = db.begin_trans().unwrap();
        // let tx = &Some(&mut tx1);
        let temp_rel_type_id: i64 = db
            .create_relation_type(
                false,
                &None,
                RELATION_TYPE_NAME,
                "",
                RelationType::UNIDIRECTIONAL,
            )
            .unwrap();
        // assert!(!db.entity_only_key_exists(tx, temp_rel_type_id).unwrap());
        assert!(!db.entity_only_key_exists(&None, temp_rel_type_id).unwrap());
        // db.delete_relation_type(tx, temp_rel_type_id).unwrap();
        db.delete_relation_type(&None, temp_rel_type_id).unwrap();
        // db.rollback_trans(tx1).unwrap();
    }

/*%%%%
  "getAttrCount, get_attribute_sorting_rows_count" should "work in all circumstances" in {
    m_db.begin_trans()

    let id: i64 = m_db.create_entity("test: org.onemodel.PSQLDbTest.getAttrCount...");
    let initialNumSortingRows = m_db.get_attribute_sorting_rows_count(Some(id));
    assert(m_db.get_attribute_count(id) == 0)
    assert(initialNumSortingRows == 0)

    createTestQuantityAttributeWithTwoEntities(id)
    createTestQuantityAttributeWithTwoEntities(id)
    assert(m_db.get_attribute_count(id) == 2)
    assert(m_db.get_attribute_sorting_rows_count(Some(id)) == 2)

    create_test_text_attribute_with_one_entity(id)
    assert(m_db.get_attribute_count(id) == 3)
    assert(m_db.get_attribute_sorting_rows_count(Some(id)) == 3)

    //whatever, just need some relation type to go with:
    let rel_type_id: i64 = m_db.createRelationType("contains", "", RelationType.UNIDIRECTIONAL);
    create_test_relation_to_local_entity_with_one_entity(id, rel_type_id)
    assert(m_db.get_attribute_count(id) == 4)
    assert(m_db.get_attribute_sorting_rows_count(Some(id)) == 4)

    DatabaseTestUtils.createAndAddTestRelationToGroup_ToEntity(m_db, id, rel_type_id, "somename", Some(12345L))
    assert(m_db.get_attribute_count(id) == 5)
    assert(m_db.get_attribute_sorting_rows_count(Some(id)) == 5)

    m_db.rollback_trans()
    //idea: (tracked in tasks): find out: WHY do the next lines fail, because the attrCount(id) is the same (4) after rolling back as before rolling back??
    // Do I not understand rollback?  But it does seem to work as expected in "entity creation/update and transaction rollback" test above.  See also
    // in EntityTest's "update_class_and_template_entity_name", at the last 2 commented lines which fail for unknown reason.  Maybe something obvious i'm just
    // missing, or maybe it's in the postgresql or jdbc transaction docs.  Could also ck in other places calling db.rollback_trans to see what's to learn from
    // current use (risk) & behaviors to compare.
//    assert(m_db.getAttrCount(id) == 0)
//    assert(m_db.get_attribute_sorting_rows_count(Some(id)) == 0)
  }

  "QuantityAttribute creation/update/deletion methods" should "work" in {
    m_db.begin_trans()
    let startingEntityCount = m_db.get_entity_count();
    let entity_id = m_db.create_entity("test: org.onemodel.PSQLDbTest.quantityAttrs()");
    let initialTotalSortingRowsCount = m_db.get_attribute_sorting_rows_count();
    let quantityAttributeId: i64 = createTestQuantityAttributeWithTwoEntities(entity_id);
    assert(m_db.get_attribute_sorting_rows_count() > initialTotalSortingRowsCount)

    let qa = new QuantityAttribute(m_db, quantityAttributeId);
    let (pid1, atid1, uid1) = (qa.get_parent_id(), qa.get_attr_type_id(), qa.getUnitId);
    assert(entity_id == pid1)
    m_db.update_quantity_attribute(quantityAttributeId, pid1, atid1, uid1, 4, Some(5), 6)
    // have to create new instance to re-read the data:
    let qa2 = new QuantityAttribute(m_db, quantityAttributeId);
    let (pid2, atid2, uid2, num2, vod2, od2) = (qa2.get_parent_id(), qa2.get_attr_type_id(), qa2.getUnitId, qa2.getNumber, qa2.get_valid_on_date(), qa2.get_observation_date());
    assert(pid2 == pid1)
    assert(atid2 == atid1)
    assert(uid2 == uid1)
    assert(num2 == 4)
    // (the ".contains" suggested by the IDE just caused another problem)
    //noinspection OptionEqualsSome
    assert(vod2 == Some(5L))
    assert(od2 == 6)

    let qAttrCount = m_db.get_quantity_attribute_count(entity_id);
    assert(qAttrCount == 1)
    assert(m_db.get_attribute_sorting_rows_count(Some(entity_id)) == 1)

    //delete the quantity attribute: #'s still right?
    let entity_countBeforeQuantityDeletion: i64 = m_db.get_entity_count();
    m_db.delete_quantity_attribute(quantityAttributeId)
    // next 2 lines should work because of the database logic (triggers as of this writing) that removes sorting rows when attrs are removed):
    assert(m_db.get_attribute_sorting_rows_count() == initialTotalSortingRowsCount)
    assert(m_db.get_attribute_sorting_rows_count(Some(entity_id)) == 0)

    let entity_countAfterQuantityDeletion: i64 = m_db.get_entity_count();
    assert(m_db.get_quantity_attribute_count(entity_id) == 0)
    if entity_countAfterQuantityDeletion != entity_countBeforeQuantityDeletion {
      fail("Got constraint backwards? Deleting quantity attribute changed Entity count from " + entity_countBeforeQuantityDeletion + " to " +
           entity_countAfterQuantityDeletion)
    }

    m_db.delete_entity(entity_id)
    let endingEntityCount = m_db.get_entity_count();
    // 2 more entities came during quantity creation (units & quantity type, is OK to leave in this kind of situation)
    assert(endingEntityCount == startingEntityCount + 2)
    assert(m_db.get_quantity_attribute_count(entity_id) == 0)
    m_db.rollback_trans()
  }

  "Attribute and AttributeSorting row deletion" should "both happen automatically upon entity deletion" in {
    let entity_id = m_db.create_entity("test: org.onemodel.PSQLDbTest sorting rows stuff");
    createTestQuantityAttributeWithTwoEntities(entity_id)
    assert(m_db.get_attribute_sorting_rows_count(Some(entity_id)) == 1)
    assert(m_db.get_quantity_attribute_count(entity_id) == 1)
    m_db.delete_entity(entity_id)
    assert(m_db.get_attribute_sorting_rows_count(Some(entity_id)) == 0)
    assert(m_db.get_quantity_attribute_count(entity_id) == 0)
  }

  "TextAttribute create/delete/update methods" should "work" in {
    let startingEntityCount = m_db.get_entity_count();
    let entity_id = m_db.create_entity("test: org.onemodel.PSQLDbTest.testTextAttrs");
    assert(m_db.get_attribute_sorting_rows_count(Some(entity_id)) == 0)
    let text_attribute_id: i64 = create_test_text_attribute_with_one_entity(entity_id);
    assert(m_db.get_attribute_sorting_rows_count(Some(entity_id)) == 1)
    let a_text_value = "asdfjkl";

    let ta = new TextAttribute(m_db, text_attribute_id);
    let (pid1, atid1) = (ta.get_parent_id(), ta.get_attr_type_id());
    assert(entity_id == pid1)
    m_db.update_text_attribute(text_attribute_id, pid1, atid1, a_text_value, Some(123), 456)
    // have to create new instance to re-read the data: immutability makes programs easier to work with
    let ta2 = new TextAttribute(m_db, text_attribute_id);
    let (pid2, atid2, txt2, vod2, od2) = (ta2.get_parent_id(), ta2.get_attr_type_id(), ta2.get_text, ta2.get_valid_on_date(), ta2.get_observation_date());
    assert(pid2 == pid1)
    assert(atid2 == atid1)
    assert(txt2 == a_text_value)
    // (the ".contains" suggested by the IDE just caused another problem)
    //noinspection OptionEqualsSome
    assert(vod2 == Some(123L))
    assert(od2 == 456)

    assert(m_db.get_text_attribute_count(entity_id) == 1)

    let entity_countBeforeTextDeletion: i64 = m_db.get_entity_count();
    m_db.delete_text_attribute(text_attribute_id)
    assert(m_db.get_text_attribute_count(entity_id) == 0)
    // next line should work because of the database logic (triggers as of this writing) that removes sorting rows when attrs are removed):
    assert(m_db.get_attribute_sorting_rows_count(Some(entity_id)) == 0)
    let entity_countAfterTextDeletion: i64 = m_db.get_entity_count();
    if entity_countAfterTextDeletion != entity_countBeforeTextDeletion {
      fail("Got constraint backwards? Deleting text attribute changed Entity count from " + entity_countBeforeTextDeletion + " to " +
           entity_countAfterTextDeletion)
    }
    // then recreate the text attribute (to verify its auto-deletion when Entity is deleted, below)
    create_test_text_attribute_with_one_entity(entity_id)
    m_db.delete_entity(entity_id)
    if m_db.get_text_attribute_count(entity_id) > 0 {
      fail("Deleting the model entity should also have deleted its text attributes; get_text_attribute_count(entity_idInNewTransaction) is " +
           m_db.get_text_attribute_count(entity_id) + ".")
    }

    let endingEntityCount = m_db.get_entity_count();
    // 2 more entities came during text attribute creation, which we don't care about either way, for this test
    assert(endingEntityCount == startingEntityCount + 2)
  }

  "DateAttribute create/delete/update methods" should "work" in {
    let startingEntityCount = m_db.get_entity_count();
    let entity_id = m_db.create_entity("test: org.onemodel.PSQLDbTest.testDateAttrs");
    assert(m_db.get_attribute_sorting_rows_count(Some(entity_id)) == 0)
    let dateAttributeId: i64 = createTestDateAttributeWithOneEntity(entity_id);
    assert(m_db.get_attribute_sorting_rows_count(Some(entity_id)) == 1)
    let da = new DateAttribute(m_db, dateAttributeId);
    let (pid1, atid1) = (da.get_parent_id(), da.get_attr_type_id());
    assert(entity_id == pid1)
    let date = System.currentTimeMillis;
    m_db.update_date_attribute(dateAttributeId, pid1, date, atid1)
    // Have to create new instance to re-read the data: immutability makes the program easier to debug/reason about.
    let da2 = new DateAttribute(m_db, dateAttributeId);
    let (pid2, atid2, date2) = (da2.get_parent_id(), da2.get_attr_type_id(), da2.getDate);
    assert(pid2 == pid1)
    assert(atid2 == atid1)
    assert(date2 == date)
    // Also test the other constructor.
    let da3 = new DateAttribute(m_db, dateAttributeId, pid1, atid1, date, 0);
    let (pid3, atid3, date3) = (da3.get_parent_id(), da3.get_attr_type_id(), da3.getDate);
    assert(pid3 == pid1)
    assert(atid3 == atid1)
    assert(date3 == date)
    assert(m_db.get_date_attribute_count(entity_id) == 1)

    let entity_countBeforeDateDeletion: i64 = m_db.get_entity_count();
    m_db.delete_date_attribute(dateAttributeId)
    assert(m_db.get_date_attribute_count(entity_id) == 0)
    // next line should work because of the database logic (triggers as of this writing) that removes sorting rows when attrs are removed):
    assert(m_db.get_attribute_sorting_rows_count(Some(entity_id)) == 0)
    assert(m_db.get_entity_count() == entity_countBeforeDateDeletion)

    // then recreate the attribute (to verify its auto-deletion when Entity is deleted, below)
    createTestDateAttributeWithOneEntity(entity_id)
    m_db.delete_entity(entity_id)
    assert(m_db.get_date_attribute_count(entity_id) == 0)

    // 2 more entities came during attribute creation, which we don't care about either way, for this test
    assert(m_db.get_entity_count() == startingEntityCount + 2)
  }

  "BooleanAttribute create/delete/update methods" should "work" in {
    let startingEntityCount = m_db.get_entity_count();
    let entity_id = m_db.create_entity("test: org.onemodel.PSQLDbTest.testBooleanAttrs");
    let val1 = true;
    let observation_date: i64 = System.currentTimeMillis;
    let valid_on_date: Option<i64> = Some(1234L);
    assert(m_db.get_attribute_sorting_rows_count(Some(entity_id)) == 0)
    let booleanAttributeId: i64 = createTestBooleanAttributeWithOneEntity(entity_id, val1, valid_on_date, observation_date);
    assert(m_db.get_attribute_sorting_rows_count(Some(entity_id)) == 1)

    let ba = new BooleanAttribute(m_db, booleanAttributeId);
    let (pid1, atid1) = (ba.get_parent_id(), ba.get_attr_type_id());
    assert(entity_id == pid1)

    let val2 = false;
    m_db.update_boolean_attribute(booleanAttributeId, pid1, atid1, val2, Some(123), 456)
    // have to create new instance to re-read the data:
    let ba2 = new BooleanAttribute(m_db, booleanAttributeId);
    let (pid2, atid2, bool2, vod2, od2) = (ba2.get_parent_id(), ba2.get_attr_type_id(), ba2.get_boolean, ba2.get_valid_on_date(), ba2.get_observation_date());
    assert(pid2 == pid1)
    assert(atid2 == atid1)
    assert(bool2 == val2)
    // (the ".contains" suggested by the IDE just caused another problem)
    //noinspection OptionEqualsSome
    assert(vod2 == Some(123L))
    assert(od2 == 456)

    assert(m_db.get_boolean_attribute_count(entity_id) == 1)

    let entity_countBeforeAttrDeletion: i64 = m_db.get_entity_count();
    m_db.delete_boolean_attribute(booleanAttributeId)
    assert(m_db.get_boolean_attribute_count(entity_id) == 0)
    // next line should work because of the database logic (triggers as of this writing) that removes sorting rows when attrs are removed):
    assert(m_db.get_attribute_sorting_rows_count(Some(entity_id)) == 0)
    let entity_countAfterAttrDeletion: i64 = m_db.get_entity_count();
    if entity_countAfterAttrDeletion != entity_countBeforeAttrDeletion {
      fail("Got constraint backwards? Deleting boolean attribute changed Entity count from " + entity_countBeforeAttrDeletion + " to " +
           entity_countAfterAttrDeletion)
    }

    // then recreate the attribute (to verify its auto-deletion when Entity is deleted, below; and to verify behavior with other values)
    let testval2: bool = true;
    let valid_on_date2: Option<i64> = None;
    let boolAttributeId2: i64 = m_db.create_boolean_attribute(pid1, atid1, testval2, valid_on_date2, observation_date);
    let ba3: BooleanAttribute = new BooleanAttribute(m_db, boolAttributeId2);
    assert(ba3.get_boolean == testval2)
    assert(ba3.get_valid_on_date().isEmpty)
    m_db.delete_entity(entity_id)
    assert(m_db.get_boolean_attribute_count(entity_id) == 0)

    let endingEntityCount = m_db.get_entity_count();
    // 2 more entities came during attribute creation, but we deleted one and (unlike similar tests) didn't recreate it.
    assert(endingEntityCount == startingEntityCount + 1)
  }

  "FileAttribute create/delete/update methods" should "work" in {
    let startingEntityCount = m_db.get_entity_count();
    let entity_id = m_db.create_entity("test: org.onemodel.PSQLDbTest.testFileAttrs");
    let descr = "somedescr";
    assert(m_db.get_attribute_sorting_rows_count(Some(entity_id)) == 0)
    let fa: FileAttribute = createTestFileAttributeAndOneEntity(new Entity(m_db, entity_id), descr, 1);
    assert(m_db.get_attribute_sorting_rows_count(Some(entity_id)) == 1)
    let fileAttributeId = fa.get_id;
    let (pid1, atid1, desc1) = (fa.get_parent_id(), fa.get_attr_type_id(), fa.getDescription);
    assert(desc1 == descr)
    let descNew = "otherdescription";
    let originalFileDateNew = 1;
    let storedDateNew = 2;
    let pathNew = "/a/b/cd.efg";
    let sizeNew = 1234;
    let hashNew = "hashchars...";
    let b11 = false;
    let b12 = true;
    let b13 = false;
    m_db.update_file_attribute(fa.get_id, pid1, atid1, descNew, originalFileDateNew, storedDateNew, pathNew, b11, b12, b13, sizeNew, hashNew)
    // have to create new instance to re-read the data:
    let fa2 = new FileAttribute(m_db, fa.get_id);
    let (pid2, atid2, desc2, ofd2, sd2, ofp2, b21, b22, b23, size2, hash2) = (fa2.get_parent_id(), fa2.get_attr_type_id(), fa2.getDescription, fa2.getOriginalFileDate,;
      fa2.getStoredDate, fa2.getOriginalFilePath, fa2.getReadable, fa2.getWritable, fa2.getExecutable, fa2.getSize, fa2.getMd5Hash)
    assert(pid2 == pid1)
    assert(atid2 == atid1)
    assert(descNew == desc2)
    assert(ofd2 == originalFileDateNew)
    assert(sd2 == storedDateNew)
    assert(ofp2 == pathNew)
    assert((b21 == b11) && (b22 == b12) && (b23 == b13))
    assert(size2 == sizeNew)
    // (startsWith, because the db pads with characters up to the full size)
    assert(hash2.startsWith(hashNew))
    assert(m_db.get_file_attribute_count(entity_id) == 1)

    let someRelTypeId = m_db.createRelationType("test: org.onemodel.PSQLDbTest.testFileAttrs-reltyp", "reversed", "BI");
    let descNewer = "other-newer";
    new FileAttribute(m_db, fa.get_id).update(Some(someRelTypeId), Some(descNewer))

    // have to create new instance to re-read the data:
    let fa3 = new FileAttribute(m_db, fileAttributeId);
    let (pid3, atid3, desc3, ofd3, sd3, ofp3, b31, b32, b33, size3, hash3) = (fa3.get_parent_id(), fa3.get_attr_type_id(), fa3.getDescription, fa3.getOriginalFileDate,;
      fa3.getStoredDate, fa3.getOriginalFilePath, fa3.getReadable, fa3.getWritable, fa3.getExecutable, fa3.getSize, fa3.getMd5Hash)
    assert(pid3 == pid1)
    assert(atid3 == someRelTypeId)
    assert(desc3 == descNewer)
    assert(ofd3 == originalFileDateNew)
    assert(sd3 == storedDateNew)
    assert(ofp3 == pathNew)
    assert(size3 == sizeNew)
    assert((b31 == b11) && (b32 == b12) && (b33 == b13))
    // (startsWith, because the db pads with characters up to the full size)
    assert(hash3.startsWith(hashNew))
    assert(m_db.get_file_attribute_count(entity_id) == 1)

    let fileAttribute4 = new FileAttribute(m_db, fileAttributeId);
    fileAttribute4.update()
    // have to create new instance to re-read the data:
    let fa4 = new FileAttribute(m_db, fileAttributeId);
    let (atid4, d4, ofd4, sd4, ofp4, b41) =;
      (fa4.get_attr_type_id(), fa4.getDescription, fa4.getOriginalFileDate, fa4.getStoredDate, fa4.getOriginalFilePath, fa4.getReadable)
    // these 2 are the key ones for this section: make sure they didn't change since we passed None to the update:
    assert(atid4 == atid3)
    assert(d4 == desc3)
    //throw in a few more
    assert(ofd4 == originalFileDateNew)
    assert(sd4 == storedDateNew)
    assert(ofp4 == pathNew)
    assert(b41 == b11)

    let entity_countBeforeFileAttrDeletion: i64 = m_db.get_entity_count();
    m_db.delete_file_attribute(fileAttributeId)
    assert(m_db.get_file_attribute_count(entity_id) == 0)
    // next line should work because of the database logic (triggers as of this writing) that removes sorting rows when attrs are removed):
    assert(m_db.get_attribute_sorting_rows_count(Some(entity_id)) == 0)
    let entity_countAfterFileAttrDeletion: i64 = m_db.get_entity_count();
    if entity_countAfterFileAttrDeletion != entity_countBeforeFileAttrDeletion {
      fail("Got constraint backwards? Deleting FileAttribute changed Entity count from " + entity_countBeforeFileAttrDeletion + " to " +
           entity_countAfterFileAttrDeletion)
    }


    // and check larger content:
    createTestFileAttributeAndOneEntity(new Entity(m_db, entity_id), "somedesc", 1200)

    // then recreate the file attribute (to verify its auto-deletion when Entity is deleted, below)
    // (w/ dif't file size for testing)
    createTestFileAttributeAndOneEntity(new Entity(m_db, entity_id), "somedesc", 0)
    m_db.delete_entity(entity_id)
    assert(m_db.get_file_attribute_count(entity_id) == 0)


    // more entities came during attribute creation, which we don't care about either way, for this test
    assert(m_db.get_entity_count() == startingEntityCount + 4)
  }

// for a test just below
%%MAYBE CAN make this a parameter instead, wherever used? see fn just below, add as parm there.
private let mut mDoDamageBuffer = false;

// instantiation does DB setup (creates tables, default data, etc):
private let m_db: PostgreSQLDatabase = new PostgreSQLDatabase(Database.TEST_USER, Database.TEST_PASS) {
override fn damageBuffer(buffer: Array[Byte]) /*%%-> Unit*/ {
if mDoDamageBuffer {
if buffer.length < 1 || buffer(0) == '0' { throw new OmException("Nothing to damage here") }
else {
if buffer(0) == '1' { buffer(0) = 2.toByte }
else { buffer(0) = 1.toByte }
// once is enough until we want to cause another failure
mDoDamageBuffer = false
}
}
}
}


//idea: recall why mocks would be better here than testing the real system and if needed switch, to speed up tests.
  // (Because we're not testing the filesystem or postgresql, and test speed matters. What is the role of integration tests for this system?)
  "FileAttribute file import/export" should "fail if file changed" in {
    let entity_id: i64 = m_db.create_entity("someent");
    let attr_type_id: i64 = m_db.create_entity("fileAttributeType");
    let uploadSourceFile: java.io.File = java.io.File.createTempFile("om-test-iofailures-", null);
    let mut writer: java.io.FileWriter = null;
    let mut inputStream: java.io.FileInputStream = null;
    let downloadTargetFile = File.createTempFile("om-testing-file-retrieval-", null);
    try {
      writer = new java.io.FileWriter(uploadSourceFile)
      writer.write("<1 kB file from: " + uploadSourceFile.getCanonicalPath + ", created " + new java.util.Date())
      writer.close()

      inputStream = new java.io.FileInputStream(uploadSourceFile)
      mDoDamageBuffer=true
      intercept[OmFileTransferException] {
                                            m_db.create_file_attribute(entity_id, attr_type_id, "xyz", 0, 0, "/doesntmatter", readable_in = true,
                                                                    writable_in = true, executable_in = false, uploadSourceFile.length(),
                                                                    FileAttribute.md5Hash(uploadSourceFile), inputStream, Some(0))
                                          }
      mDoDamageBuffer = false
      //so it should work now:
      inputStream = new java.io.FileInputStream(uploadSourceFile)
      let faId: i64 = m_db.create_file_attribute(entity_id, attr_type_id, "xyz", 0, 0,;
                                               "/doesntmatter", readable_in = true, writable_in = true, executable_in = false,
                                               uploadSourceFile.length(), FileAttribute.md5Hash(uploadSourceFile), inputStream, None)

      let fa: FileAttribute = new FileAttribute(m_db, faId);
      mDoDamageBuffer = true
      intercept[OmFileTransferException] {
                                            fa.retrieveContent(downloadTargetFile)
                                          }
      mDoDamageBuffer = false
      //so it should work now
      fa.retrieveContent(downloadTargetFile)
    } finally {
      mDoDamageBuffer=false
      if inputStream != null { inputStream.close() }
      if writer != null { writer.close() }
      if downloadTargetFile != null){
        downloadTargetFile.delete()
      }
    }
  }

  "relation to entity methods and relation type methods" should "work" in {
    let startingEntityOnlyCount = m_db.get_entities_only_count();
    let startingRelationTypeCount = m_db.get_relation_type_count();
    let entity_id = m_db.create_entity("test: org.onemodel.PSQLDbTest.testRelsNRelTypes()");
    let startingRelCount = m_db.get_relation_types(0, Some(25)).size;
    let rel_type_id: i64 = m_db.createRelationType("contains", "", RelationType.UNIDIRECTIONAL);

    //verify a bugfix from 2013-10-31 or 2013-11-4 in how SELECT is written.
    assert(m_db.get_relation_types(0, Some(25)).size == startingRelCount + 1)
    assert(m_db.get_entities_only_count() == startingEntityOnlyCount + 1)

    assert(m_db.get_attribute_sorting_rows_count(Some(entity_id)) == 0)
    let related_entity_id: i64 = create_test_relation_to_local_entity_with_one_entity(entity_id, rel_type_id);
    assert(m_db.get_attribute_sorting_rows_count(Some(entity_id)) == 1)
    let checkRelation = m_db.get_relation_to_local_entity_data(rel_type_id, entity_id, related_entity_id);
    let checkValidOnDate = checkRelation(1);
    assert(checkValidOnDate.isEmpty) // should get back None when created with None: see description for table's field in create_tables method.
    assert(m_db.get_relation_to_local_entity_count(entity_id) == 1)

    let new_name = "test: org.onemodel.PSQLDbTest.relationupdate...";
    let name_in_reverse = "nameinreverse;!@#$%^&*()-_=+{}[]:\"'<>?,./`~" //and verify can handle some variety of chars;
    m_db.update_relation_type(rel_type_id, new_name, name_in_reverse, RelationType.BIDIRECTIONAL)
    // have to create new instance to re-read the data:
    let updatedRelationType = new RelationType(m_db, rel_type_id);
    assert(updatedRelationType.get_name == new_name)
    assert(updatedRelationType.get_name_in_reverse_direction == name_in_reverse)
    assert(updatedRelationType.getDirectionality == RelationType.BIDIRECTIONAL)

    m_db.delete_relation_to_local_entity(rel_type_id, entity_id, related_entity_id)
    assert(m_db.get_relation_to_local_entity_count(entity_id) == 0)
    // next line should work because of the database logic (triggers as of this writing) that removes sorting rows when attrs are removed):
    assert(m_db.get_attribute_sorting_rows_count(Some(entity_id)) == 0)

    let entityOnlyCountBeforeRelationTypeDeletion: i64 = m_db.get_entities_only_count();
    m_db.delete_relation_type(rel_type_id)
    assert(m_db.get_relation_type_count() == startingRelationTypeCount)
    // ensure that removing rel type doesn't remove more entities than it should, and that the 'onlyCount' works right.
    //i.e. as above, verify a bugfix from 2013-10-31 or 2013-11-4 in how SELECT is written.
    assert(entityOnlyCountBeforeRelationTypeDeletion == m_db.get_entities_only_count())

    m_db.delete_entity(entity_id)
  }

  "get_containing_groups_ids" should "find groups containing the test group" in {
    /*
    Makes a thing like this:        entity1    entity3
                                       |         |
                                    group1     group3
                                       |         |
                                        \       /
                                         entity2
                                            |
                                         group2
     ...(and then checks in the middle that entity2 has 1 containing group, before adding entity3/group3)
     ...and then checks that entity2 has 2 containing groups.
     */
    let entity_id1 = m_db.create_entity("test-get_containing_groups_ids-entity1");
    let rel_type_id: i64 = m_db.createRelationType("test-get_containing_groups_ids-reltype1", "", RelationType.UNIDIRECTIONAL);
    let (groupId1, _) = DatabaseTestUtils.createAndAddTestRelationToGroup_ToEntity(m_db, entity_id1, rel_type_id, "test-get_containing_groups_ids-group1");
    let group1 = new Group(m_db,groupId1);
    let entity_id2 = m_db.create_entity("test-get_containing_groups_ids-entity2");
    group1.addEntity(entity_id2)
    let (groupId2, _) = DatabaseTestUtils.createAndAddTestRelationToGroup_ToEntity(m_db, entity_id2, rel_type_id, "test-get_containing_groups_ids-group1");
    let group2 = new Group(m_db, groupId2);

    let containingGroups:Vec<Option<DataType>>] = m_db.get_groups_containing_entitys_groups_ids(group2.get_id);
    assert(containingGroups.size == 1)
    assert(containingGroups.head(0).get.asInstanceOf[i64] == groupId1)

    let entity_id3 = m_db.create_entity("test-get_containing_groups_ids-entity3");
    let (groupId3, _) = DatabaseTestUtils.createAndAddTestRelationToGroup_ToEntity(m_db, entity_id3, rel_type_id, "test-get_containing_groups_ids-group1");
    let group3 = new Group(m_db, groupId3);
    group3.addEntity(entity_id2)

    let containingGroups2:Vec<Vec<Option<DataType>>> = m_db.get_groups_containing_entitys_groups_ids(group2.get_id);
    assert(containingGroups2.size == 2)
    assert(containingGroups2.head(0).get.asInstanceOf[i64] == groupId1)
    assert(containingGroups2.tail.head(0).get.asInstanceOf[i64] == groupId3)
  }

  "relation to group and group methods" should "work" in {
    let relToGroupName = "test: PSQLDbTest.testRelsNRelTypes()";
    let entityName = relToGroupName + "--theEntity";
    let entity_id = m_db.create_entity(entityName);
    let rel_type_id: i64 = m_db.createRelationType("contains", "", RelationType.UNIDIRECTIONAL);
    let valid_on_date = 12345L;
    assert(m_db.get_attribute_sorting_rows_count(Some(entity_id)) == 0)
    let (groupId:i64, createdRtg:RelationToGroup) = DatabaseTestUtils.createAndAddTestRelationToGroup_ToEntity(m_db, entity_id, rel_type_id, relToGroupName,;
                                                                                                                Some(valid_on_date), allowMixedClassesIn = true)
    assert(m_db.get_attribute_sorting_rows_count(Some(entity_id)) == 1)

    let rtg: RelationToGroup = new RelationToGroup(m_db, createdRtg.get_id, createdRtg.get_parent_id(), createdRtg.get_attr_type_id(), createdRtg.getGroupId);
    let group: Group = new Group(m_db, groupId);
    assert(group.getMixedClassesAllowed)
    assert(group.get_name == relToGroupName)

    let checkRelation = m_db.get_relation_to_group_data_by_keys(rtg.get_parent_id(), rtg.get_attr_type_id(), rtg.getGroupId);
    assert(checkRelation(0).get.asInstanceOf[i64] == rtg.get_id)
    assert(checkRelation(0).get.asInstanceOf[i64] == createdRtg.get_id)
    assert(checkRelation(1).get.asInstanceOf[i64] == entity_id)
    assert(checkRelation(2).get.asInstanceOf[i64] == rel_type_id)
    assert(checkRelation(3).get.asInstanceOf[i64] == groupId)
    assert(checkRelation(4).get.asInstanceOf[i64] == valid_on_date)
    let checkAgain = m_db.get_relation_to_group_data(rtg.get_id);
    assert(checkAgain(0).get.asInstanceOf[i64] == rtg.get_id)
    assert(checkAgain(0).get.asInstanceOf[i64] == createdRtg.get_id)
    assert(checkAgain(1).get.asInstanceOf[i64] == entity_id)
    assert(checkAgain(2).get.asInstanceOf[i64] == rel_type_id)
    assert(checkAgain(3).get.asInstanceOf[i64] == groupId)
    assert(checkAgain(4).get.asInstanceOf[i64] == valid_on_date)

    assert(group.getSize() == 0)
    let entity_id2 = m_db.create_entity(entityName + 2);
    group.addEntity(entity_id2)
    assert(group.getSize() == 1)
    group.deleteWithEntities()
    assert(intercept[Exception] {
                                  new RelationToGroup(m_db, rtg.get_id, rtg.get_parent_id(), rtg.get_attr_type_id(), rtg.getGroupId )
                                }.getMessage.contains("does not exist"))
    assert(intercept[Exception] {
                                  new Entity(m_db, entity_id2)
                                }.getMessage.contains("does not exist"))
    assert(group.getSize() == 0)
    // next line should work because of the database logic (triggers as of this writing) that removes sorting rows when attrs are removed):
    assert(m_db.get_attribute_sorting_rows_count(Some(entity_id)) == 0)

    let (groupId2, _) = DatabaseTestUtils.createAndAddTestRelationToGroup_ToEntity(m_db, entity_id, rel_type_id, "somename", None);

    let group2: Group = new Group(m_db, groupId2);
    assert(group2.getSize() == 0)

    let entity_id3 = m_db.create_entity(entityName + 3);
    group2.addEntity(entity_id3)
    assert(group2.getSize() == 1)

    let entity_id4 = m_db.create_entity(entityName + 4);
    group2.addEntity(entity_id4)
    let entity_id5 = m_db.create_entity(entityName + 5);
    group2.addEntity(entity_id5)
    // (at least make sure next method runs:)
    m_db.get_group_entry_sorting_index(groupId2, entity_id5)
    assert(group2.getSize() == 3)
    assert(m_db.get_group_entry_objects(group2.get_id, 0).size() == 3)

    group2.removeEntity(entity_id5)
    assert(m_db.get_group_entry_objects(group2.get_id, 0).size() == 2)

    group2.delete()
    assert(intercept[Exception] {
                                  new Group(m_db, groupId)
                                }.getMessage.contains("does not exist"))
    assert(group2.getSize() == 0)
    // ensure the other entity still exists: not deleted by that delete command
    new Entity(m_db, entity_id3)

    // probably revise this later for use when adding that update method:
    //val new_name = "test: org.onemodel.PSQLDbTest.relationupdate..."
    //m_db.update_relation_type(rel_type_id, new_name, name_in_reverse, RelationType.BIDIRECTIONAL)
    //// have to create new instance to re-read the data:
    //val updatedRelationType = new RelationType(m_db, rel_type_id)
    //assert(updatedRelationType.get_name == new_name)
    //assert(updatedRelationType.get_name_in_reverse_direction == name_in_reverse)
    //assert(updatedRelationType.getDirectionality == RelationType.BIDIRECTIONAL)

    //m_db.delete_relation_to_group(relToGroupId)
    //assert(m_db.get_relation_to_group_count(entity_id) == 0)
  }

  "get_groups" should "work" in {
    let group3id = m_db.create_group("g3");
    let number = m_db.get_groups(0).size;
    let number2 = m_db.get_groups(0, None, Some(group3id)).size;
    assert(number == number2 + 1)
    let number3 = m_db.get_groups(1).size;
    assert(number == number3 + 1)
  }

  "deleting entity" should "work even if entity is in a relationtogroup" in {
    let startingEntityCount = m_db.get_entities_only_count();
    let relToGroupName = "test:PSQLDbTest.testDelEntity_InGroup";
    let entityName = relToGroupName + "--theEntity";
    let entity_id = m_db.create_entity(entityName);
    let rel_type_id: i64 = m_db.createRelationType("contains", "", RelationType.UNIDIRECTIONAL);
    let valid_on_date = 12345L;
    let groupId = DatabaseTestUtils.createAndAddTestRelationToGroup_ToEntity(m_db, entity_id, rel_type_id, relToGroupName, Some(valid_on_date))._1;
    //val rtg: RelationToGroup = new RelationToGroup
    let group:Group = new Group(m_db, groupId);
    group.addEntity(m_db.create_entity(entityName + 1))
    assert(m_db.get_entities_only_count() == startingEntityCount + 2)
    assert(m_db.getGroupSize(groupId) == 1)

    let entity_id2 = m_db.create_entity(entityName + 2);
    assert(m_db.get_entities_only_count() == startingEntityCount + 3)
    assert(m_db.get_count_of_groups_containing_entity(entity_id2) == 0)
    group.addEntity(entity_id2)
    assert(m_db.getGroupSize(groupId) == 2)
    assert(m_db.get_count_of_groups_containing_entity(entity_id2) == 1)
    let descriptions = m_db.get_containing_relation_to_group_descriptions(entity_id2, Some(9999));
    assert(descriptions.size == 1)
    assert(descriptions.get(0) == entityName + "->" + relToGroupName)

    //doesn't get an error:
    m_db.delete_entity(entity_id2)

    let descriptions2 = m_db.get_containing_relation_to_group_descriptions(entity_id2, Some(9999));
    assert(descriptions2.size == 0)
    assert(m_db.get_count_of_groups_containing_entity(entity_id2) == 0)
    assert(m_db.get_entities_only_count() == startingEntityCount + 2)
    assert(intercept[Exception] {
                                  new Entity(m_db, entity_id2)
                                }.getMessage.contains("does not exist"))

    assert(m_db.getGroupSize(groupId) == 1)

    let list = m_db.get_group_entry_objects(groupId, 0);
    assert(list.size == 1)
    let remainingContainedEntityId = list.get(0).get_id;

    // ensure the first entities still exist: not deleted by that delete command
    new Entity(m_db, entity_id)
    new Entity(m_db, remainingContainedEntityId)
  }

  "get_sorted_attributes" should "return them all and correctly" in {
    let entity_id = m_db.create_entity("test: org.onemodel.PSQLDbTest.testRelsNRelTypes()");
    create_test_text_attribute_with_one_entity(entity_id)
    createTestQuantityAttributeWithTwoEntities(entity_id)
    let rel_type_id: i64 = m_db.createRelationType("contains", "", RelationType.UNIDIRECTIONAL);
    let related_entity_id: i64 = create_test_relation_to_local_entity_with_one_entity(entity_id, rel_type_id);
    DatabaseTestUtils.createAndAddTestRelationToGroup_ToEntity(m_db, entity_id, rel_type_id)
    createTestDateAttributeWithOneEntity(entity_id)
    createTestBooleanAttributeWithOneEntity(entity_id, valIn = false, None, 0)
    createTestFileAttributeAndOneEntity(new Entity(m_db, entity_id), "desc", 2, verifyIn = false)

    m_db.update_entity_only_public_status(related_entity_id, None)
    let onlyPublicTotalAttrsAvailable1 = m_db.get_sorted_attributes(entity_id, 0, 999, only_public_entities_in = true)._2;
    m_db.update_entity_only_public_status(related_entity_id, Some(false))
    let onlyPublicTotalAttrsAvailable2 = m_db.get_sorted_attributes(entity_id, 0, 999, only_public_entities_in = true)._2;
    m_db.update_entity_only_public_status(related_entity_id, Some(true))
    let onlyPublicTotalAttrsAvailable3 = m_db.get_sorted_attributes(entity_id, 0, 999, only_public_entities_in = true)._2;
    assert(onlyPublicTotalAttrsAvailable1 == onlyPublicTotalAttrsAvailable2)
    assert((onlyPublicTotalAttrsAvailable3 - 1) == onlyPublicTotalAttrsAvailable2)

    let (attrTuples: Array[(i64, Attribute)], totalAttrsAvailable) = m_db.get_sorted_attributes(entity_id, 0, 999, only_public_entities_in = false);
    assert(totalAttrsAvailable > onlyPublicTotalAttrsAvailable1)
    let counter: i64 = attrTuples.length;
    // should be the same since we didn't create enough to span screens (requested them all):
    assert(counter == totalAttrsAvailable)
    if counter != 7 {
      fail("We added attributes (RelationToLocalEntity, quantity & text, date,bool,file,RTG), but getAttributeIdsAndAttributeTypeIds() returned " + counter + "?")
    }

    let mut (foundQA, foundTA, foundRTE, foundRTG, foundDA, foundBA, foundFA) = (false, false, false, false, false, false, false);
    for (attr <- attrTuples) {
      attr._2 match {
        case attribute: QuantityAttribute =>
          assert(attribute.getNumber == 50)
          foundQA = true
        case attribute: TextAttribute =>
          //strangely, running in the intellij 12 IDE wouldn't report this line as a failure when necessary, but
          // the cli does.
          assert(attribute.get_text == "some test text")
          foundTA = true
        case attribute: RelationToLocalEntity =>
          assert(attribute.get_attr_type_id() == rel_type_id)
          foundRTE = true
        case attribute: RelationToGroup =>
          foundRTG = true
        case attribute: DateAttribute =>
          foundDA = true
        case attribute: BooleanAttribute =>
          foundBA = true
        case attribute: FileAttribute =>
          foundFA = true
        case _ =>
          throw new Exception("unexpected")
      }
    }
    assert(foundQA && foundTA && foundRTE && foundRTG && foundDA && foundBA && foundFA)
  }

  "entity deletion" should "also delete RelationToLocalEntity attributes (and get_relation_to_remote_entity_count should work)" in {
    let entity_id = m_db.create_entity("test: org.onemodel.PSQLDbTest.testRelsNRelTypes()");
    let rel_type_id: i64 = m_db.createRelationType("is sitting next to", "", RelationType.UNIDIRECTIONAL);
    let startingLocalCount = m_db.get_relation_to_local_entity_count(entity_id);
    let startingRemoteCount = m_db.get_relation_to_remote_entity_count(entity_id);
    let related_entity_id: i64 = create_test_relation_to_local_entity_with_one_entity(entity_id, rel_type_id);
    assert(m_db.get_relation_to_local_entity_count(entity_id) == startingLocalCount + 1)

    let oi: OmInstance = m_db.get_local_om_instance_data;
    let remoteEntityId = 1234;
    m_db.createRelationToRemoteEntity(rel_type_id, entity_id, remoteEntityId, None, 0, oi.get_id)
    assert(m_db.get_relation_to_local_entity_count(entity_id) == startingLocalCount + 1)
    assert(m_db.get_relation_to_remote_entity_count(entity_id) == startingRemoteCount + 1)
    assert(m_db.get_relation_to_remote_entity_data(rel_type_id, entity_id, oi.get_id, remoteEntityId).length > 0)

    m_db.delete_entity(entity_id)
    if m_db.get_relation_to_local_entity_count(entity_id) != 0 {
      fail("Deleting the model entity should also have deleted its RelationToLocalEntity objects. " +
           "get_relation_to_local_entity_count(entity_idInNewTransaction) is " + m_db.get_relation_to_local_entity_count(entity_id) + ".")
    }
    assert(intercept[Exception] {
                                  m_db.get_relation_to_local_entity_data(rel_type_id, entity_id, related_entity_id)
                                }.getMessage.contains("Got 0 instead of 1 result"))
    assert(intercept[Exception] {
                                  m_db.get_relation_to_remote_entity_data(rel_type_id, entity_id, oi.get_id, related_entity_id)
                                }.getMessage.contains("Got 0 instead of 1 result"))

    m_db.delete_relation_type(rel_type_id)
  }

  "attributes" should "handle valid_on_dates properly in & out of db" in {
    let entity_id = m_db.create_entity("test: org.onemodel.PSQLDbTest.attributes...");
    let rel_type_id = m_db.createRelationType(RELATION_TYPE_NAME, "", RelationType.UNIDIRECTIONAL);
    // create attributes & read back / other values (None alr done above) as entered (confirms read back correctly)
    // (these methods do the checks, internally)
    create_test_relation_to_local_entity_with_one_entity(entity_id, rel_type_id, Some(0L))
    create_test_relation_to_local_entity_with_one_entity(entity_id, rel_type_id, Some(System.currentTimeMillis()))
    createTestQuantityAttributeWithTwoEntities(entity_id)
    createTestQuantityAttributeWithTwoEntities(entity_id, Some(0))
    create_test_text_attribute_with_one_entity(entity_id)
    create_test_text_attribute_with_one_entity(entity_id, Some(0))
  }

  "testAddQuantityAttributeWithBadParentID" should "not work" in {
    println!("starting testAddQuantityAttributeWithBadParentID")
    let badParentId: i64 = m_db.findIdWhichIsNotKeyOfAnyEntity;

    // Database should not allow adding quantity with a bad parent (Entity) ID!
    // idea: make it a more specific exception type, so we catch only the error we want...
    intercept[Exception] {
                           createTestQuantityAttributeWithTwoEntities(badParentId)
                         }

  }

    fn createTestQuantityAttributeWithTwoEntities(in_parent_id: i64, in_valid_on_date: Option<i64> = None) -> i64 {
    let unitId: i64 = m_db.create_entity("centimeters");
    let attr_type_id: i64 = m_db.create_entity(QUANTITY_TYPE_NAME);
    let default_date: i64 = System.currentTimeMillis;
    let valid_on_date: Option<i64> = in_valid_on_date;
    let observation_date: i64 = default_date;
    let number: Float = 50;
    let quantityId: i64 = m_db.create_quantity_attribute(in_parent_id, attr_type_id, unitId, number, valid_on_date, observation_date);

    // and verify it:
    let qa: QuantityAttribute = new QuantityAttribute(m_db, quantityId);
    assert(qa.get_parent_id() == in_parent_id)
    assert(qa.getUnitId == unitId)
    assert(qa.getNumber == number)
    assert(qa.get_attr_type_id() == attr_type_id)
    if in_valid_on_date.isEmpty {
      assert(qa.get_valid_on_date().isEmpty)
    } else {
      let inDate: i64 = in_valid_on_date.get;
      let gotDate: i64 = qa.get_valid_on_date().get;
      assert(inDate == gotDate)
    }
    assert(qa.get_observation_date() == observation_date)
    quantityId
  }

  "rollbackWithCatch" should "catch and return chained exception showing failed rollback" in {
    let db = new PostgreSQLDatabase("abc", "defg") {;
      override fn connect(inDbName: String, username: String, password: String) {
        // leave it null so calling it will fail as desired below.
        mConn = null
      }
      override fn create_and_check_expected_data() -> Unit {
        // Overriding because it is not needed for this test, and normally uses mConn, which by being set to null just above, breaks the method.
        // (intentional style violation for readability)
        //noinspection ScalaUselessExpression
        None
      }
      override fn model_tables_exist()  -> bool {
true
}
//noinspection ScalaUselessExpression  (intentional style violation, for readability)
override fn do_database_upgrades_if_needed() {
Unit
}
    }
    let mut found = false;
    let originalErrMsg: String = "testing123";
    try {
      try throw new Exception(originalErrMsg)
      catch {
        case e: Exception => throw db.rollbackWithCatch(e)
      }
    } catch {
      case t: Throwable =>
        found = true
        let sw = new java.io.StringWriter();
        t.printStackTrace(new java.io.PrintWriter(sw))
        let s = sw.toString;
        assert(s.contains(originalErrMsg))
        assert(s.contains("See the chained messages for ALL: the cause of rollback failure, AND"))
        assert(s.contains("at org.onemodel.core.model.PostgreSQLDatabase.rollback_trans"))
    }
    assert(found)
  }

  "createBaseData, findEntityOnlyIdsByName, createClassTemplateEntity, findContainedEntries, and findRelationToGroup_OnEntity" should
  "have worked right in earlier db setup and now" in {
    let PERSON_TEMPLATE: String = "person" + Database.TEMPLATE_NAME_SUFFIX;
    let systemEntityId = m_db.getSystemEntityId;
    let groupIdOfClassTemplates = m_db.findRelationToAndGroup_OnEntity(systemEntityId, Some(Database.CLASS_TEMPLATE_ENTITY_GROUP_NAME))._3;

    // (Should be some value, but the activity on the test DB wouldn't have ids incremented to 0 yet,so that one would be invalid. Could use the
    // other method to find an unused id, instead of 0.)
    assert(groupIdOfClassTemplates.is_defined && groupIdOfClassTemplates.get != 0)
    assert(new Group(m_db, groupIdOfClassTemplates.get).getMixedClassesAllowed)

    let personTemplateEntityId: i64 = m_db.findEntityOnlyIdsByName(PERSON_TEMPLATE).get.head;
    // idea: make this next part more scala-like (but only if still very simple to read for programmers who are used to other languages):
    let mut found = false;
    let entitiesInGroup: Vec<Entity> = m_db.get_group_entry_objects(groupIdOfClassTemplates.get, 0);
    for (entity <- entitiesInGroup.toArray) {
      if entity.asInstanceOf[Entity].get_id == personTemplateEntityId {
        found = true
      }
    }
    assert(found)

    // make sure the other approach also works, even with deeply nested data:
    let rel_type_id: i64 = m_db.createRelationType("contains", "", RelationType.UNIDIRECTIONAL);
    let te1 = create_test_relation_to_local_entity_with_one_entity(personTemplateEntityId, rel_type_id);
    let te2 = create_test_relation_to_local_entity_with_one_entity(te1, rel_type_id);
    let te3 = create_test_relation_to_local_entity_with_one_entity(te2, rel_type_id);
    let te4 = create_test_relation_to_local_entity_with_one_entity(te3, rel_type_id);
    let foundIds: mutable.TreeSet[i64] = m_db.find_contained_local_entity_ids(new mutable.TreeSet[i64](), systemEntityId, PERSON_TEMPLATE, 4,;
                                                                     stop_after_any_found = false)
    assert(foundIds.contains(personTemplateEntityId), "Value not found in query: " + personTemplateEntityId)
    let allContainedWithName: mutable.TreeSet[i64] = m_db.find_contained_local_entity_ids(new mutable.TreeSet[i64](), systemEntityId, RELATED_ENTITY_NAME, 4,;
                                                                                 stop_after_any_found = false)
    // (see idea above about making more scala-like)
    let mut allContainedIds = "";
    for (id: i64 <- allContainedWithName) {
      allContainedIds += id + ", "
    }
    assert(allContainedWithName.size == 3, "Returned set had wrong count (" + allContainedWithName.size + "): " + allContainedIds)
    let te4Entity: Entity = new Entity(m_db, te4);
    te4Entity.addTextAttribute(te1/*not really but whatever*/, RELATED_ENTITY_NAME, None, None, 0)
    let allContainedWithName2: mutable.TreeSet[i64] = m_db.find_contained_local_entity_ids(new mutable.TreeSet[i64](), systemEntityId, RELATED_ENTITY_NAME, 4,;
                                                                                  stop_after_any_found = false)
    // should be no change yet (added it outside the # of levels to check):
    assert(allContainedWithName2.size == 3, "Returned set had wrong count (" + allContainedWithName.size + "): " + allContainedIds)
    let te2Entity: Entity = new Entity(m_db, te2);
    te2Entity.addTextAttribute(te1/*not really but whatever*/, RELATED_ENTITY_NAME, None, None, 0)
    let allContainedWithName3: mutable.TreeSet[i64] = m_db.find_contained_local_entity_ids(new mutable.TreeSet[i64](), systemEntityId, RELATED_ENTITY_NAME, 4,;
                                                                                  stop_after_any_found = false)
    // should be no change yet (the entity was already in the return set, so the TA addition didn't add anything)
    assert(allContainedWithName3.size == 3, "Returned set had wrong count (" + allContainedWithName.size + "): " + allContainedIds)
    te2Entity.addTextAttribute(te1/*not really but whatever*/, "otherText", None, None, 0)
    let allContainedWithName4: mutable.TreeSet[i64] = m_db.find_contained_local_entity_ids(new mutable.TreeSet[i64](), systemEntityId, "otherText", 4,;
                                                                                  stop_after_any_found = false)
    // now there should be a change:
    assert(allContainedWithName4.size == 1, "Returned set had wrong count (" + allContainedWithName.size + "): " + allContainedIds)

    let editorCmd = m_db.get_text_editor_command;
    if Util::isWindows { assert(editorCmd.contains("notepad")) }
    else {
    assert(editorCmd == "vi") }
  }

  "isDuplicateEntity" should "work" in {
    let name: String = "testing isDuplicateEntity";
    let entity_id: i64 = m_db.create_entity(name);
    assert(m_db.is_duplicate_entity_name(name))
    assert(!m_db.is_duplicate_entity_name(name, Some(entity_id)))

    let entityWithSpaceInNameId: i64 = m_db.create_entity(name + " ");
    assert(!m_db.is_duplicate_entity_name(name + " ", Some(entityWithSpaceInNameId)))

    let entity_idWithLowercaseName: i64 = m_db.create_entity(name.toLowerCase);
    assert(m_db.is_duplicate_entity_name(name, Some(entity_idWithLowercaseName)))

    m_db.update_entity_only_name(entity_id, name.toLowerCase)
    assert(m_db.is_duplicate_entity_name(name, Some(entity_idWithLowercaseName)))
    assert(m_db.is_duplicate_entity_name(name, Some(entity_id)))

    m_db.delete_entity(entity_idWithLowercaseName)
    assert(!m_db.is_duplicate_entity_name(name, Some(entity_id)))

    // intentionally put some uppercase letters for later comparison w/ lowercase.
    let relTypeName = name + "-RelationType";
    let rel_type_id: i64 = m_db.createRelationType("testingOnly", relTypeName, RelationType.UNIDIRECTIONAL);
    assert(m_db.is_duplicate_entity_name(relTypeName))
    assert(!m_db.is_duplicate_entity_name(relTypeName, Some(rel_type_id)))

    m_db.begin_trans()
    m_db.update_entity_only_name(entity_id, relTypeName.toLowerCase)
    assert(m_db.is_duplicate_entity_name(relTypeName, Some(entity_id)))
    assert(m_db.is_duplicate_entity_name(relTypeName, Some(rel_type_id)))
    // because setting an entity name to relTypeName doesn't really make sense, was just for that part of the test.
    m_db.rollback_trans()
  }

  "isDuplicateEntityClass and class update/deletion" should "work" in {
    let name: String = "testing isDuplicateEntityClass";
    let (classId, entity_id) = m_db.createClassAndItsTemplateEntity(name, name);
    assert(EntityClass.isDuplicate(m_db, name))
    assert(!EntityClass.isDuplicate(m_db, name, Some(classId)))

    m_db.update_class_name(classId, name.toLowerCase)
    assert(!EntityClass.isDuplicate(m_db, name, Some(classId)))
    assert(EntityClass.isDuplicate(m_db, name.toLowerCase))
    assert(!EntityClass.isDuplicate(m_db, name.toLowerCase, Some(classId)))
    m_db.update_class_name(classId, name)

    m_db.update_class_create_default_attributes(classId, Some(false))
    let should1: Option<bool> = new EntityClass(m_db, classId).getCreateDefaultAttributes;
    assert(!should1.get)
    m_db.update_class_create_default_attributes(classId, None)
    let should2: Option<bool> = new EntityClass(m_db, classId).getCreateDefaultAttributes;
    assert(should2.isEmpty)
    m_db.update_class_create_default_attributes(classId, Some(true))
    let should3: Option<bool> = new EntityClass(m_db, classId).getCreateDefaultAttributes;
    assert(should3.get)

    m_db.update_entitys_class(entity_id, None)
    m_db.delete_class_and_its_template_entity(classId)
    assert(!EntityClass.isDuplicate(m_db, name, Some(classId)))
    assert(!EntityClass.isDuplicate(m_db, name))

  }

  "EntitiesInAGroup and getclasses/classcount methods" should "work, and should enforce class_id uniformity within a group of entities" in {
    // ...for now anyway. See comments at this table in psqld.create_tables and/or hasMixedClasses.

    // This also tests db.create_entity and db.updateEntityOnlyClass.

    let entityName = "test: PSQLDbTest.testgroup-class-uniqueness" + "--theEntity";
    let (classId, entity_id) = m_db.createClassAndItsTemplateEntity(entityName, entityName);
    let (classId2, entity_id2) = m_db.createClassAndItsTemplateEntity(entityName + 2, entityName + 2);
    let classCount = m_db.get_class_count();
    let classes = m_db.get_classes(0);
    assert(classCount == classes.size)
    let classCountLimited = m_db.get_class_count(Some(entity_id2));
    assert(classCountLimited == 1)

    //whatever, just need some relation type to go with:
    let rel_type_id: i64 = m_db.createRelationType("contains", "", RelationType.UNIDIRECTIONAL);
    let groupId = DatabaseTestUtils.createAndAddTestRelationToGroup_ToEntity(m_db, entity_id, rel_type_id, "test: PSQLDbTest.testgroup-class-uniqueness",;
                                                                             Some(12345L), allowMixedClassesIn = false)._1
    let group: Group = new Group(m_db, groupId);
    assert(! m_db.is_entity_in_group(groupId, entity_id))
    assert(! m_db.is_entity_in_group(groupId, entity_id))
    group.addEntity(entity_id)
    assert(m_db.is_entity_in_group(groupId, entity_id))
    assert(! m_db.is_entity_in_group(groupId, entity_id2))

    //should fail due to mismatched classId (a long):
    assert(intercept[Exception] {
                                  group.addEntity(entity_id2)
                                }.getMessage.contains(Database.MIXED_CLASSES_EXCEPTION))
    // should succeed (same class now):
    m_db.update_entitys_class(entity_id2, Some(classId))
    group.addEntity(entity_id2)
    // ...and for convenience while here, make sure we can't make mixed classes with changing the *entity* either:
    assert(intercept[Exception] {
                                  m_db.update_entitys_class(entity_id2, Some(classId2))
                                }.getMessage.contains(Database.MIXED_CLASSES_EXCEPTION))
    assert(intercept[Exception] {
                                  m_db.update_entitys_class(entity_id2, None)
                                }.getMessage.contains(Database.MIXED_CLASSES_EXCEPTION))

    //should fail due to mismatched classId (NULL):
    let entity_id3 = m_db.create_entity(entityName + 3);
    assert(intercept[Exception] {
                                  group.addEntity(entity_id3)
                                }.getMessage.contains(Database.MIXED_CLASSES_EXCEPTION))

    assert(!m_db.areMixedClassesAllowed(groupId))


    let systemEntityId = m_db.getSystemEntityId;
    // idea: (noted at other use of this method)
    let classGroupId = m_db.findRelationToAndGroup_OnEntity(systemEntityId, Some(Database.CLASS_TEMPLATE_ENTITY_GROUP_NAME))._3;
    assert(m_db.areMixedClassesAllowed(classGroupId.get))

    let groupSizeBeforeRemoval = m_db.getGroupSize(groupId);

    assert(m_db.getGroupSize(groupId, 2) == 0)
    assert(m_db.getGroupSize(groupId, 1) == groupSizeBeforeRemoval)
    assert(m_db.getGroupSize(groupId) == groupSizeBeforeRemoval)
    m_db.archive_entity(entity_id2)
    assert(m_db.getGroupSize(groupId, 2) == 1)
    assert(m_db.getGroupSize(groupId, 1) == groupSizeBeforeRemoval - 1)
    assert(m_db.getGroupSize(groupId) == groupSizeBeforeRemoval)

    m_db.remove_entity_from_group(groupId, entity_id2)
    let groupSizeAfterRemoval = m_db.getGroupSize(groupId);
    assert(groupSizeAfterRemoval < groupSizeBeforeRemoval)

    assert(m_db.getGroupSize(groupId, 2) == 0)
    assert(m_db.getGroupSize(groupId, 1) == groupSizeBeforeRemoval - 1)
    assert(m_db.getGroupSize(groupId) == groupSizeBeforeRemoval - 1)
  }

  "get_entities_only and ...Count" should "allow limiting results by classId and/or group containment" in {
    // idea: this could be rewritten to not depend on pre-existing data to fail when it's supposed to fail.
    let startingEntityCount = m_db.get_entities_only_count();
    let someClassId: i64 = m_db.db_query_wrapper_for_one_row("select id from class limit 1", "i64")(0).get.asInstanceOf[i64];
    let numEntitiesInClass = m_db.extract_row_count_from_count_query("select count(1) from entity where class_id=" + someClassId);
    assert(startingEntityCount > numEntitiesInClass)
    let allEntitiesInClass = m_db.get_entities_only(0, None, Some(someClassId), limit_by_class = true);
    let allEntitiesInClassCount1 = m_db.get_entities_only_count(limit_by_class = true, Some(someClassId));
    let allEntitiesInClassCount2 = m_db.get_entities_only_count(limit_by_class = true, Some(someClassId), None);
    assert(allEntitiesInClassCount1 == allEntitiesInClassCount2)
    let templateClassId: i64 = new EntityClass(m_db, someClassId).get_template_entity_id;
    let allEntitiesInClassCountWoClass = m_db.get_entities_only_count(limit_by_class = true, Some(someClassId), Some(templateClassId));
    assert(allEntitiesInClassCountWoClass == allEntitiesInClassCount1 - 1)
    assert(allEntitiesInClass.size == allEntitiesInClassCount1)
    assert(allEntitiesInClass.size < m_db.get_entities_only(0, None, Some(someClassId), limit_by_class = false).size)
    assert(allEntitiesInClass.size == numEntitiesInClass)
    let e: Entity = allEntitiesInClass.get(0);
    assert(e.getClassId.get == someClassId)

    // part 2:
    // some setup, confirm good
    let startingEntityCount2 = m_db.get_entities_only_count();
    let rel_type_id: i64 = m_db.createRelationType("contains", "", RelationType.UNIDIRECTIONAL);
    let id1: i64 = m_db.create_entity("name1");
    let (group, rtg) = new Entity(m_db, id1).addGroupAndRelationToGroup(rel_type_id, "someRelToGroupName", allowMixedClassesInGroupIn = false, None, 1234L,;
                                                                       None, caller_manages_transactions_in = false)
    assert(m_db.relation_to_group_keys_exist(rtg.get_parent_id(), rtg.get_attr_type_id(), rtg.getGroupId))
    assert(m_db.attribute_key_exists(rtg.get_form_id, rtg.get_id))
    let id2: i64 = m_db.create_entity("name2");
    group.addEntity(id2)
    let entity_countAfterCreating = m_db.get_entities_only_count();
    assert(entity_countAfterCreating == startingEntityCount2 + 2)
    let resultSize = m_db.get_entities_only(0).size();
    assert(entity_countAfterCreating == resultSize)
    let resultSizeWithNoneParameter = m_db.get_entities_only(0, None, group_to_omit_id_in = None).size();
    assert(entity_countAfterCreating == resultSizeWithNoneParameter)

    // the real part 2 test
    let resultSizeWithGroupOmission = m_db.get_entities_only(0, None, group_to_omit_id_in = Some(group.get_id)).size();
    assert(entity_countAfterCreating - 1 == resultSizeWithGroupOmission)
  }

  "EntitiesInAGroup table (or methods? ick)" should "allow all a group's entities to have no class" in {
    // ...for now anyway.  See comments at this table in psqld.create_tables and/or hasMixedClasses.

    let entityName = "test: PSQLDbTest.testgroup-class-allowsAllNulls" + "--theEntity";
    let (classId, entity_id) = m_db.createClassAndItsTemplateEntity(entityName, entityName);
    let rel_type_id: i64 = m_db.createRelationType("contains", "", RelationType.UNIDIRECTIONAL);
    let groupId = DatabaseTestUtils.createAndAddTestRelationToGroup_ToEntity(m_db, entity_id, rel_type_id, "test: PSQLDbTest.testgroup-class-allowsAllNulls",;
                                                                             Some(12345L), allowMixedClassesIn = false)._1
    let group: Group = new Group(m_db, groupId);
    // 1st one has a NULL class_id
    let entity_id3 = m_db.create_entity(entityName + 3);
    group.addEntity(entity_id3)
    // ...so it works to add another one that's NULL
    let entity_id4 = m_db.create_entity(entityName + 4);
    group.addEntity(entity_id4)
    // but adding one with a class_id fails w/ mismatch:
    let entity_id5 = m_db.create_entity(entityName + 5, Some(classId));
    assert(intercept[Exception] {
                                  group.addEntity(entity_id5)
                                }.getMessage.contains(Database.MIXED_CLASSES_EXCEPTION))
  }

  "get_entities_only_count" should "not count entities used as relation types or attribute types" in {
    let entity_id = m_db.create_entity("test: org.onemodel.PSQLDbTest.get_entities_only_count");
    let c1 = m_db.get_entities_only_count();
    assert(m_db.get_entities_only_count() == c1)
    let rel_type_id: i64 = m_db.createRelationType("contains", "", RelationType.UNIDIRECTIONAL);
    assert(m_db.get_entities_only_count() == c1)
    create_test_relation_to_local_entity_with_one_entity(entity_id, rel_type_id)
    let c2 = c1 + 1;
    assert(m_db.get_entities_only_count() == c2)

    // this kind shouldn't matter--confirming:
    let rel_type_id2: i64 = m_db.createRelationType("contains2", "", RelationType.UNIDIRECTIONAL);
    DatabaseTestUtils.createAndAddTestRelationToGroup_ToEntity(m_db, entity_id, rel_type_id2)
    assert(m_db.get_entities_only_count() == c2)

    let prevEntitiesUsedAsAttributeTypes = m_db.get_count_of_entities_used_as_attribute_types(Util::DATE_TYPE, quantity_seeks_unit_not_type_in = false);
    let dateAttributeId = createTestDateAttributeWithOneEntity(entity_id);
    let dateAttribute = new DateAttribute(m_db, dateAttributeId);
    assert(m_db.get_count_of_entities_used_as_attribute_types(Util::DATE_TYPE, quantity_seeks_unit_not_type_in = false) == prevEntitiesUsedAsAttributeTypes + 1)
    assert(m_db.get_entities_only_count() == c2)
    let dateAttributeTypeEntities: Array[Entity] = m_db.get_entities_used_as_attribute_types(Util::DATE_TYPE, 0, quantity_seeks_unit_not_type_in = false);
                                                   .toArray(new Array[Entity](0 ))
    let mut found = false;
    for (dateAttributeType: Entity <- dateAttributeTypeEntities.toArray) {
      if dateAttributeType.get_id == dateAttribute.get_attr_type_id()) {
        found = true
      }
    }
    assert(found)

    createTestBooleanAttributeWithOneEntity(entity_id, valIn = false, None, 0)
    assert(m_db.get_entities_only_count() == c2)

    createTestFileAttributeAndOneEntity(new Entity(m_db, entity_id), "desc", 2, verifyIn = false)
    assert(m_db.get_entities_only_count() == c2)

  }

  "get_matching_entities & Groups" should "work" in {
    let entity_id1 = m_db.create_entity("test: org.onemodel.PSQLDbTest.get_matching_entities1--abc");
    let entity1 = new Entity(m_db, entity_id1);
    let entity_id2 = m_db.create_entity("test: org.onemodel.PSQLDbTest.get_matching_entities2");
    m_db.create_text_attribute(entity_id1, entity_id2, "defg", None, 0)
    let entities1 = m_db.get_matching_entities(0, None, None, "abc");
    assert(entities1.size == 1)
    m_db.create_text_attribute(entity_id2, entity_id1, "abc", None, 0)
    let entities2 = m_db.get_matching_entities(0, None, None, "abc");
    assert(entities2.size == 2)

    let rel_type_id: i64 = m_db.createRelationType("contains", "", RelationType.UNIDIRECTIONAL);
    let group_name = "someRelToGroupName";
    entity1.addGroupAndRelationToGroup(rel_type_id, group_name, allowMixedClassesInGroupIn = false, None, 1234L,
                                       None, caller_manages_transactions_in = false)
    assert(m_db.get_matching_groups(0, None, None, "some-xyz-not a grp name").size == 0)
    assert(m_db.get_matching_groups(0, None, None, group_name).size > 0)
  }

  //idea: should this be moved to ImportExportTest? why did i put it here originally?
  "getJournal" should "show activity during a date range" in {
    let startDataSetupTime = System.currentTimeMillis();
    let entity_id: i64 = m_db.create_entity("test object");
    let entity: Entity = new Entity(m_db, entity_id);
    let importExport = new ImportExport(null, new Controller(null, false, Some(Database.TEST_USER), Some(Database.TEST_PASS)));
    let importFile: File = importExport.tryImporting_FOR_TESTS("testImportFile0.txt", entity);
    let ids: java.util.ArrayList[i64] = m_db.find_all_entity_ids_by_name("vsgeer-testing-getJournal-in-db");
    let (fileContents: String, outputFile: File) = importExport.tryExportingTxt_FOR_TESTS(ids, m_db);
    // (next 3 lines are redundant w/ a similar test in ImportExportTest, but are here to make sure the data
    // is as expected before proceeding with the actual purpose of this test:)
    assert(fileContents.contains("vsgeer"), "unexpected file contents:  " + fileContents)
    assert(fileContents.contains("record/report/review"), "unexpected file contents:  " + fileContents)
    assert(outputFile.length == importFile.length)

    m_db.archive_entity(entity_id)
    let endDataSetupTime = System.currentTimeMillis();

    let results: util.ArrayList[(i64, String, i64)] = m_db.find_journal_entries(startDataSetupTime, endDataSetupTime);
    assert(results.size > 0)
  }

  "get_textAttributeByNameForEntity" should "fail when no rows found" in {
    intercept[org.onemodel.core.OmDatabaseException] {
                                     let systemEntityId = m_db.getSystemEntityId;
                                     m_db.get_text_attribute_by_type_id(systemEntityId, 1L, Some(1))
                                   }
  }

  "get_relations_to_group_containing_this_group and get_containing_relations_to_group" should "work" in {
    let entity_id: i64 = m_db.create_entity("test: get_relations_to_group_containing_this_group...");
    let entity_id2: i64 = m_db.create_entity("test: get_relations_to_group_containing_this_group2...");
    let rel_type_id: i64 = m_db.createRelationType("contains in get_relations_to_group_containing_this_group", "", RelationType.UNIDIRECTIONAL);
    let (groupId, rtg) = DatabaseTestUtils.createAndAddTestRelationToGroup_ToEntity(m_db, entity_id, rel_type_id,;
                                                                                    "some group name in get_relations_to_group_containing_this_group")
    let group = new Group(m_db, groupId);
    group.addEntity(entity_id2)
    let rtgs = m_db.get_relations_to_group_containing_this_group(groupId, 0);
    assert(rtgs.size == 1)
    assert(rtgs.get(0).get_id == rtg.get_id)

    let sameRtgs = m_db.get_containing_relations_to_group(entity_id2, 0);
    assert(sameRtgs.size == 1)
    assert(sameRtgs.get(0).get_id == rtg.get_id)
  }
 */
}
