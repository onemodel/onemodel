/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003, 2004, 2010, 2011, 2013-2020 inclusive, and 2023-2025 inclusive, Luke A. Call.
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
use crate::model::entity_class::EntityClass;
use crate::model::postgres::postgresql_database::*;
// use crate::model::postgres::*;
use crate::model::group::Group;
use crate::model::relation_to_group::RelationToGroup;
use crate::model::relation_to_local_entity::RelationToLocalEntity;
// use crate::model::relation_to_remote_entity::RelationToRemoteEntity;
use crate::model::boolean_attribute::BooleanAttribute;
use crate::model::date_attribute::DateAttribute;
use crate::model::om_instance::OmInstance;
use crate::model::relation_type::RelationType;
use crate::model::file_attribute::FileAttribute;
use crate::model::quantity_attribute::QuantityAttribute;
use crate::model::text_attribute::TextAttribute;
use crate::util::Util;
use anyhow::anyhow;
use chrono::Utc;
// use futures::executor::block_on;
use sqlx::postgres::*;
// Specifically omitting sql::Error from use statements so that it is *clearer* which Error type is
// in use, in the code.
// use sqlx::{Column, PgPool, Postgres, Row, Transaction, ValueRef};
use sqlx::{Postgres, Row, Transaction};
// use std::collections::HashSet;
// use std::fmt::format;
use std::any::Any;
use std::cell::{RefCell, RefMut};
use std::rc::Rc;
use tracing::*;
// use tracing_subscriber::FmtSubscriber;

#[cfg(test)]
mod test {
    use super::*;
    use crate::model::attribute::Attribute;
    use crate::model::attribute_with_valid_and_observed_dates::AttributeWithValidAndObservedDates;
    //use crate::model::relation_to_group::RelationToGroup;
    use std::any::Any;

    const QUANTITY_TYPE_NAME: &str = "length";
    const RELATION_TYPE_NAME: &str = "someRelationToEntityTypeName";
    const RELATED_ENTITY_NAME: &str = "someRelatedEntityName";

    /// This fn is used in important (informative) commented lines elsewhere.
    fn db_query_for_test1(
        rt: &tokio::runtime::Runtime,
        pool: &sqlx::Pool<Postgres>,
        shared_tx: Option<Rc<RefCell<Transaction<Postgres>>>>,
        sql: &str,
    ) -> Result<bool, String> {
        let query = sqlx::query(sql);
        let map = query.map(|_sqlx_row: PgRow| {
            //do stuff to capture results
        });
        let using_transaction;
        match shared_tx {
            Some(transaction) => {
                // next 2 lines compile.  Trying to do it in more lines just below, to compare to other code that
                // is not compiling successfully.
                //let mut tx_mut: RefMut<'_, _> = transaction.borrow_mut();
                //let future = map.fetch_all(&mut *tx_mut);

                let mut trans: RefMut<'_, Transaction<'_, Postgres>> = transaction.borrow_mut();
                //let mut trans2: Transaction<'_, Postgres>  = *trans;
                //let mut trans2  = *trans;
                let future = map.fetch_all(&mut *trans);

                rt.block_on(future).unwrap();
                using_transaction = true;
            }
            None => {
                let future = map.fetch_all(pool);
                rt.block_on(future).unwrap();
                using_transaction = false;
            }
        }
        Ok(using_transaction)
    }

    #[test]
    fn test_compile_problem_with_non_reference_transaction_parameters() -> Result<(), String> {
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
        let tx = rt.block_on(pool.begin()).unwrap();
        let shared_tx = Rc::new(RefCell::new(tx));
        let using_transaction = db_query_for_test1(
            &rt,
            &pool,
            Some(shared_tx.clone()),
            "select count(*) from pg_aggregate",
        )?;
        assert!(using_transaction);
        // confirm this can be done twice
        let using_transaction = db_query_for_test1(
            &rt,
            &pool,
            Some(shared_tx.clone()),
            "select count(*) from pg_aggregate",
        )?;
        assert!(using_transaction);
        // confirm this can be done w/o a transaction
        let using_transaction =
            db_query_for_test1(&rt, &pool, None, "select count(*) from pg_views")?;
        assert!(!using_transaction);
        Ok(())
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

        //idea: could get the next lines to work also and show something useful re the current setting??:
        //let future = sqlx::query("show transaction isolation level").execute(&pool);
        //let x = rt.block_on(future).unwrap();
        //debug!("in test_basic_sql_connectivity_with_async_and_tokio: Query result re transaction isolation lvl?:  {:?}", x);
        //%%later: Search for related cmts w/ "isolation".

        for c in 1..=150 {
            debug!(
                "in test_basic_sql_connectivity_with_async_and_tokio: before, {}",
                c
            );

            // hung after 1-4 iterations, when block_on didn't have "rt.":
            let sql: String = "DROP table IF EXISTS test_doesnt_exist CASCADE".to_string();
            let future = sqlx::query(sql.as_str()).execute(&pool);
            let x: Result<PgQueryResult, sqlx::Error> = rt.block_on(future);
            //using next line instead avoided the problem!
            // let x: Result<PgQueryResult, sqlx::Error> = future.await;
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
        let db: Rc<PostgreSQLDatabase> = Rc::new(Util::initialize_test_db().unwrap());
        let tx = db.begin_trans().unwrap();
        let tx = Some(Rc::new(RefCell::new(tx)));
        let pref_name = "xyznevercreatemeinreallife";
        assert!(db
            .get_user_preference_boolean(tx.clone(), pref_name, None)
            .unwrap()
            .is_none());

        assert_eq!(
            db.get_user_preference_boolean(tx.clone(), pref_name, Some(true))
                .unwrap(),
            Some(true)
        );
        db.set_user_preference_boolean(tx.clone(), pref_name, false)
            .unwrap();
        assert_eq!(
            db.get_user_preference_boolean(tx.clone(), pref_name, Some(true))
                .unwrap(),
            Some(false)
        );

        let pref_name2 = "xyz2";
        assert!(db
            .get_user_preference_entity_id(tx.clone(), pref_name2, None)
            .unwrap()
            .is_none());
        assert_eq!(
            db.get_user_preference_entity_id(tx.clone(), pref_name2, Some(0))
                .unwrap(),
            Some(0)
        );
        db.set_user_preference_entity_id(
            tx.clone(),
            pref_name2,
            db.get_system_entity_id(tx.clone()).unwrap(),
        )
        .unwrap();
        assert_eq!(
            db.get_user_preference_entity_id(tx.clone(), pref_name2, Some(0))
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
        db.drop(None, "table", name.as_str()).unwrap();
        let tx = db.begin_trans().unwrap();
        let transaction = Some(Rc::new(RefCell::new(tx)));
        let mut id = db
            .create_entity(transaction.clone(), name.as_str(), None, None)
            .expect(format!("Failed to create entity with name: {}", name).as_str());
        assert!(db
            .entity_key_exists(transaction.clone(), id, true)
            .expect(format!("Found: {}", id).as_str()));

        //%%later: Can make every place like this call common fns instead of dup code? Note that this one
        //does *rollback*, most do commit.  It also differs because self is not db.
        let local_tx_cell: Option<RefCell<Transaction<Postgres>>> =
            Rc::into_inner(transaction.unwrap());
        //match local_tx_cell {
        //  Some(t) => {
        let unwrapped_local_tx = local_tx_cell.unwrap().into_inner();
        db.rollback_trans(unwrapped_local_tx).unwrap();
        //},
        //None => {
        //    return Err(anyhow!("Unexpectedly found None instead of Some<RefCell<Transaction<Postgres>>>. How?"));
        //},
        //}
        assert!(!db
            .entity_key_exists(None, id, true)
            .expect(format!("Found: {}", id).as_str()));

        // this time with an implied rollback, as sqlx docs say when a transaction goes out of scope
        // without a commit, it is implicitly rolled back.
        {
            let tx = db.begin_trans().unwrap();
            let transaction = Some(Rc::new(RefCell::new(tx)));
            id = db
                .create_entity(transaction.clone(), name.as_str(), None, None)
                .expect(format!("Failed to create: {}", name).as_str());
        }
        assert!(!db
            .entity_key_exists(None, id, true)
            .expect(format!("Found: {}", id).as_str()));

        // this time with a commit, not a rollback
        let tx = db.begin_trans().unwrap();
        let transaction = Some(Rc::new(RefCell::new(tx)));
        id = db
            .create_entity(transaction.clone(), name.as_str(), None, None)
            .expect(format!("Failed to create entity w name: {}", name).as_str());
        assert!(db
            .entity_key_exists(transaction.clone(), id, true)
            .expect(format!("Failed to find: {}", id).as_str()));

        let local_tx_cell: Option<RefCell<Transaction<Postgres>>> =
            Rc::into_inner(transaction.unwrap());
        let unwrapped_local_tx = local_tx_cell.unwrap().into_inner();
        db.commit_trans(unwrapped_local_tx).unwrap();
        assert!(db
            .entity_key_exists(None, id, true)
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
    ///yes it actually was failing when written, in my use of Sqlx somehow.
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
        //%%later: Why does the insert sql get "PoolTimedOut" if .max_connections is 1 instead of 10??
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

        //%%later: AFTER FIXING (is fixed now, right?), see all the places with "rollbacketc%%" (2) and address them.
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
                None,
                "select count(1) from pg_class where relname='odb_version'",
                true,
            )
            .unwrap();
        assert!(version_table_exists);
        let results = db
            .db_query_wrapper_for_one_row(None, "select version from odb_version", "Int")
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

    fn create_test_text_attribute_with_one_entity<'a>(
        db: &'a Rc<PostgreSQLDatabase>,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        in_parent_id: i64,
        in_valid_on_date: Option<i64>, /*= None*/
    ) -> i64 {
        let attr_type_id: i64 = db
            .create_entity(None, "textAttributeTypeLikeSsn", None, None)
            .unwrap();
        let default_date: i64 = Utc::now().timestamp_millis();
        let valid_on_date: Option<i64> = in_valid_on_date;
        let observation_date: i64 = default_date;
        let text = "some test text";
        let text_attribute_id: i64 = db
            .create_text_attribute(
                transaction.clone(),
                in_parent_id,
                attr_type_id,
                &text,
                valid_on_date,
                observation_date,
                None,
            )
            .unwrap();
        // and verify it:
        let mut ta: TextAttribute =
            TextAttribute::new2(db.clone(), transaction.clone(), text_attribute_id).unwrap();
        assert!(ta.get_parent_id(transaction.clone()).unwrap() == in_parent_id);
        assert!(ta.get_text(transaction.clone()).unwrap() == text);
        assert!(ta.get_attr_type_id(transaction.clone()).unwrap() == attr_type_id);
        if in_valid_on_date.is_none() {
            assert!(ta.get_valid_on_date(transaction.clone()).unwrap().is_none());
        } else {
            assert!(ta.get_valid_on_date(transaction.clone()).unwrap() == in_valid_on_date);
        }
        assert!(ta.get_observation_date(transaction.clone()).unwrap() == observation_date);

        text_attribute_id
    }

    fn create_test_date_attribute_with_one_entity(
        db: &Rc<PostgreSQLDatabase>,
        in_parent_id: i64,
    ) -> i64 {
        let attr_type_id: i64 = db
            .create_entity(None, "dateAttributeType--likeDueOn", None, None)
            .unwrap();
        let date: i64 = Utc::now().timestamp_millis();
        let date_attribute_id: i64 = db
            .create_date_attribute(None, in_parent_id, attr_type_id, date, None)
            .unwrap();
        let mut ba: DateAttribute =
            DateAttribute::new2(db.clone(), None, date_attribute_id).unwrap();
        assert!(ba.get_parent_id(None).unwrap() == in_parent_id);
        assert!(ba.get_date(None).unwrap() == date);
        assert!(ba.get_attr_type_id(None).unwrap() == attr_type_id);
        date_attribute_id
    }

    fn create_test_boolean_attribute_with_one_entity<'a, 'b>(
        db: &'a Rc<PostgreSQLDatabase>,
        // purpose: see comment in delete_objects
        transaction: Option<Rc<RefCell<Transaction<'b, Postgres>>>>,
        in_parent_id: i64,
        val_in: bool,
        in_valid_on_date: Option<i64>, /*= None*/
        observation_date_in: i64,
    ) -> i64
    where
        'a: 'b,
    {
        let attr_type_id: i64 = db
            .create_entity(
                transaction.clone(),
                "boolAttributeType-like-isDone",
                None,
                None,
            )
            .unwrap();
        let boolean_attribute_id: i64 = db
            .create_boolean_attribute(
                transaction.clone(),
                in_parent_id,
                attr_type_id,
                val_in,
                in_valid_on_date,
                observation_date_in,
                None,
            )
            .unwrap();
        let mut ba = BooleanAttribute::new2(
            db.clone(),
            transaction.clone(),
            boolean_attribute_id,
        )
        .unwrap();
        assert!(ba.get_attr_type_id(transaction.clone()).unwrap() == attr_type_id);
        assert!(ba.get_boolean(transaction.clone()).unwrap() == val_in);
        assert!(ba.get_valid_on_date(transaction.clone()).unwrap() == in_valid_on_date);
        assert!(ba.get_parent_id(transaction.clone()).unwrap() == in_parent_id);
        assert!(ba.get_observation_date(transaction.clone()).unwrap() == observation_date_in);
        boolean_attribute_id
    }

    /*%%file_attr latertests after FileAttribute is more completed.
    fn create_test_file_attribute_and_one_entity(in_parent_entity: Entity, in_descr: String, added_kilobytes_in: i32, verify_in: bool /*= true*/) -> FileAttribute {
        let attr_type_id: i64 = db.create_entity("fileAttributeType");
        let file: java.io.File = java.io.File.createTempFile("om-test-file-attr-", null);
        let mut writer: java.io.FileWriter = null;
        let mut verificationFile: java.io.File = null;
        try {
            writer = new java.io.FileWriter(file)
            writer.write(added_kilobytes_in + "+ kB file from: " + file.getCanonicalPath + ", created " + new java.util.Date())
            let mut nextInteger: i64 = 1;
            for (i: Int <- 1 to (1000 * added_kilobytes_in)) {
                // there's a bug here: files aren't the right size (not single digits being added) but oh well it's just to make some file.
                writer.write(nextInteger.toString)
                if i % 1000 == 0 { nextInteger += 1 }
            }
            writer.close();

            // sleep is so we can see a difference between the 2 dates to be saved, in later assertion.
            let sleepPeriod = 5;
            Thread.sleep(sleepPeriod);
            let size = file.length();
            let mut inputStream: java.io.FileInputStream = null;
            let mut fa: FileAttribute = null;
            try {
                inputStream = new java.io.FileInputStream(file)
                fa = in_parent_entity.add_file_attribute(attr_type_id, in_descr, file)
            } finally {
                if inputStream != null { inputStream.close() }
        }

        if verify_in {
            // this first part is just testing DB consistency from add to retrieval, not the actual file:
            assert(fa.get_parent_id() == in_parent_entity.get_id)
            assert(fa.get_attr_type_id() == attr_type_id)
            assert((fa.get_stored_date() - (sleepPeriod - 1)) > fa.get_original_file_date())
            // (easily fails if the program pauses when debugging):
            assert((fa.get_stored_date() - 10000) < fa.get_original_file_date())
            assert(file.lastModified() == fa.get_original_file_date())
            assert(file.length() == fa.get_size())
            assert(file.getCanonicalPath == fa.get_original_file_path())
            assert(fa.get_description() == in_descr)
            assert(fa.get_size() == size)
            // (startsWith, because the db pads with characters up to the full size)
            assert(fa.get_readable() && fa.get_writeable() && !fa.get_executable())

            // now ck the content itself
            verificationFile = File.createTempFile("om-fileattr-retrieved-content-", null)
            fa.retrieveContent(verificationFile)
            assert(verificationFile.canRead == fa.get_readable())
            assert(verificationFile.canWrite == fa.get_writeable())
            assert(verificationFile.canExecute == fa.get_executable())
        }
        fa
    } finally {
        if verificationFile != null { verificationFile.delete() }
        if writer != null { writer.close() }
        if file != null { file.delete() }
        }
    }
    */

    fn create_test_relation_to_local_entity_with_one_entity<'a, 'b>(
        db: &'a Rc<PostgreSQLDatabase>,
        tx: Option<Rc<RefCell<Transaction<'b, Postgres>>>>,
        in_entity_id: i64,
        in_rel_type_id: i64,
        in_valid_on_date: Option<i64>, /*= None*/
    ) -> i64 
    where
        'a: 'b
    {
        //Util::initialize_tracing();
        //let db: PostgreSQLDatabase = Util::initialize_test_db().unwrap();
        // idea: could use here instead: db.create_entityAndRelationToLocalEntity
        let related_entity_id: i64 = db
            .create_entity(tx.clone(), RELATED_ENTITY_NAME, None, None)
            .unwrap();
        // let valid_on_date: Option<i64> = if in_valid_on_date.isEmpty { None } else { in_valid_on_date };
        let observation_date: i64 = Utc::now().timestamp_millis();

        //let ref rc_db = &db;
        //let ref cloned = rc_db.clone();
        //let (id, _new_sorting_index) = db.clone()
        //let (id, _new_sorting_index) = cloned
        let (id, _new_sorting_index) = db
            .create_relation_to_local_entity(
                tx.clone(),
                in_rel_type_id,
                in_entity_id,
                related_entity_id,
                in_valid_on_date,
                observation_date,
                None,
            )
            .unwrap();

        // and verify it:
        let mut rtle: RelationToLocalEntity = RelationToLocalEntity::new2(
            db.clone(),
            tx.clone(),
            id,
            in_rel_type_id,
            in_entity_id,
            related_entity_id,
        )
        .unwrap();
        match in_valid_on_date {
            None => assert!(rtle.get_valid_on_date(tx.clone()).unwrap().is_none()),
            Some(d) => {
                let in_dt: i64 = d;
                let got_dt: i64 = rtle.get_valid_on_date(tx.clone()).unwrap().unwrap();
                assert!(in_dt == got_dt);
            }
        }
        assert!(rtle.get_observation_date(tx).unwrap() == observation_date);
        related_entity_id
    }

    #[test]
    fn escape_quotes_etc_allow_updating_db_with_single_quotes() {
        Util::initialize_tracing();
        let db: Rc<PostgreSQLDatabase> = Rc::new(Util::initialize_test_db().unwrap());
        let name = "This ' name contains a single-quote.";
        let tx = db.begin_trans().unwrap();
        let tx = Some(Rc::new(RefCell::new(tx)));
        //on a create:
        let entity_id: i64 = db.create_entity(tx.clone(), name, None, None).unwrap();
        let new_name = db.get_entity_name(tx.clone(), entity_id);
        assert_eq!(name, new_name.unwrap().unwrap().as_str());

        //and on an update:
        let text_attribute_id: i64 =
            create_test_text_attribute_with_one_entity(&db, tx.clone(), entity_id, None);
        let a_text_value = "as'dfjkl";
        let mut ta =
            TextAttribute::new2(db.clone(), tx.clone(), text_attribute_id).unwrap();
        let (pid1, atid1) = (
            ta.get_parent_id(tx.clone()).unwrap(),
            ta.get_attr_type_id(tx.clone()).unwrap(),
        );
        db.update_text_attribute(
            tx.clone(),
            text_attribute_id,
            pid1,
            atid1,
            a_text_value,
            Some(123),
            456,
        )
        .unwrap();
        // have to create new instance to re-read the data:
        let mut ta2 =
            TextAttribute::new2(db.clone(), tx.clone(), text_attribute_id).unwrap();
        let txt2 = ta2.get_text(tx.clone()).unwrap();

        assert!(txt2 == a_text_value);
    }

    #[test]
    /// With transaction rollback, this should create one new entity, work right, then have none.
    fn test_entity_creation_and_update() {
        Util::initialize_tracing();
        let db: Rc<PostgreSQLDatabase> = Rc::new(Util::initialize_test_db().unwrap());
        let name = "test: org.onemodel.PSQLDbTest.entitycreation...";
        let temp = db.clone();
        let tx1 = temp.begin_trans().unwrap();
        let tx = Some(Rc::new(RefCell::new(tx1)));

        let entity_count_before_creating: u64 = db.get_entity_count(tx.clone()).unwrap();
        let entities_only_first_count: u64 = db
            .get_entities_only_count(tx.clone(), false, None, None)
            .unwrap();

        let id: i64 = db.create_entity(tx.clone(), name, None, None).unwrap();
        let new_name = db.get_entity_name(tx.clone(), id);
        assert_eq!(name, new_name.unwrap().unwrap().as_str());
        let entity_count_after_1st_create = db.get_entity_count(tx.clone()).unwrap();
        let entities_only_new_count = db
            .get_entities_only_count(tx.clone(), false, None, None)
            .unwrap();

        // Next condition fails when run concurrently with other tests, because the other tests
        // also manipulate data: apparently counting rows is not isolated by a transaction?
        if entity_count_before_creating + 1 != entity_count_after_1st_create
            || entities_only_first_count + 1 != entities_only_new_count
        {
            panic!("get_entity_count() after adding doesn't match prior count+1! Before: {} and {}, after: {} and {}.",
                   entity_count_before_creating,  entities_only_first_count, entity_count_after_1st_create, entities_only_new_count);
        }

        assert!(db.entity_key_exists(tx.clone(), id, true).unwrap());

        let new_name = "test: ' org.onemodel.PSQLDbTest.entityupdate...";
        db.update_entity_only_name(tx.clone(), id, new_name)
            .unwrap();
        // have to create new instance to re-read the data:
        let mut updated_entity = Entity::new2(db.clone(), tx.clone(), id).unwrap();
        let name3 = updated_entity.get_name(tx.clone()).unwrap().as_str();
        assert_eq!(name3, new_name);

        assert!(db.entity_only_key_exists(tx.clone(), id).unwrap());
        let local_tx_cell: Option<RefCell<Transaction<Postgres>>> = Rc::into_inner(tx.unwrap());
        let unwrapped_local_tx = local_tx_cell.unwrap().into_inner();
        db.rollback_trans(unwrapped_local_tx).unwrap();

        // now should not exist

        // Next assert_eq fails when run concurrently with other tests, because the other tests
        // create data:
        let entity_count_after_rollback = db.get_entity_count(None).unwrap();
        assert_eq!(entity_count_after_rollback, entity_count_before_creating);

        assert!(!db.entity_key_exists(None, id, true).unwrap());
    }

    #[test]
    fn find_id_which_is_not_key_of_any_entity() {
        Util::initialize_tracing();
        let db: PostgreSQLDatabase = Util::initialize_test_db().unwrap();

        assert!(!db
            .entity_key_exists(
                None,
                db.find_id_which_is_not_key_of_any_entity(None).unwrap(),
                true
            )
            .unwrap());
    }

    #[test]
    fn entity_only_key_exists_should_not_find_relation_to_local_entity_record() {
        Util::initialize_tracing();
        let db: PostgreSQLDatabase = Util::initialize_test_db().unwrap();
        let tx = db.begin_trans().unwrap();
        let tx1 = Some(Rc::new(RefCell::new(tx)));
        let temp_rel_type_id: i64 = db
            .create_relation_type(
                tx1.clone(),
                RELATION_TYPE_NAME,
                "",
                RelationType::UNIDIRECTIONAL,
            )
            .unwrap();
        assert!(!db
            .entity_only_key_exists(tx1.clone(), temp_rel_type_id)
            .unwrap());
        //no need to delete if we are shortly afterward rolling back
        //db.delete_relation_type(tx1.clone(), temp_rel_type_id).unwrap();
        let local_tx_cell: Option<RefCell<Transaction<Postgres>>> = Rc::into_inner(tx1.unwrap());
        let unwrapped_local_tx = local_tx_cell.unwrap().into_inner();
        db.rollback_trans(unwrapped_local_tx).unwrap();
    }

    #[test]
    fn get_attr_count_and_get_attribute_sorting_rows_count() {
        Util::initialize_tracing();
        let db: Rc<PostgreSQLDatabase> = Rc::new(Util::initialize_test_db().unwrap());
        let id: i64 = db
            .create_entity(
                None, //tx.clone(),
                "test: org.onemodel.PSQLDbTest.getAttrCount...",
                None,
                None,
            )
            .unwrap();
        let entity: Entity = Entity::new2(db.clone(), None /*tx.clone*/, id).unwrap();
        let temp = db.clone();
        let tx = temp.begin_trans().unwrap();
        let tx: Option<Rc<RefCell<Transaction<Postgres>>>> = Some(Rc::new(RefCell::new(tx)));

        let initial_num_sorting_rows: u64 =
            db.get_attribute_sorting_rows_count(None, Some(id)).unwrap();
        assert!(db.get_attribute_count(tx.clone(), id, false).unwrap() == 0);
        assert!(initial_num_sorting_rows == 0);

        create_test_quantity_attribute_with_two_entities(&db, tx.clone(), id, None);
        create_test_quantity_attribute_with_two_entities(&db, tx.clone(), id, None);
        assert!(db.clone().get_attribute_count(tx.clone(), id, false).unwrap() == 2);
        assert!(
            db.clone().get_attribute_sorting_rows_count(tx.clone(), Some(id))
                .unwrap()
                == 2
        );

        create_test_text_attribute_with_one_entity(&db, tx.clone(), id, None);
        assert!(db.get_attribute_count(tx.clone(), id, false).unwrap() == 3);
        assert!(
            db.get_attribute_sorting_rows_count(tx.clone(), Some(id))
                .unwrap()
                == 3
        );

        //whatever, just need some relation type to go with:
        let rel_type_id: i64 = db
            .create_relation_type(tx.clone(), "contains", "", RelationType::UNIDIRECTIONAL)
            .unwrap();
        create_test_relation_to_local_entity_with_one_entity(
            //db.clone(),
            &db,
            tx.clone(),
            id,
            rel_type_id,
            None,
        );
        assert!(db.clone().get_attribute_count(tx.clone(), id, false).unwrap() == 4);
        assert!(
            db.get_attribute_sorting_rows_count(tx.clone(), Some(id))
                .unwrap()
                == 4
        );

        create_and_add_test_relation_to_group_on_to_entity(
            db.clone(),
            tx.clone(),
            &entity,
            rel_type_id,
            "somename",
            Some(12345 as i64),
            true,
        )
        .unwrap();
        assert_eq!(db.get_attribute_count(tx.clone(), id, false).unwrap(), 5);
        assert_eq!(
            db.get_attribute_sorting_rows_count(tx.clone(), Some(id))
                .unwrap(),
            5
        );

        let unwrapped_local_tx = Rc::into_inner(tx.unwrap()).unwrap().into_inner();
        db.rollback_trans(unwrapped_local_tx).unwrap();

        //%%idea: (tracked in tasks): find out: WHY do the next lines fail, because the attrCount(id) is 
        //the same (4) after rolling back as before rolling back??
        // Do I not understand rollback?  But it does seem to work as expected in "entity creation/update 
        // and transaction rollback" test above.  See also
        // in EntityTest's "update_class_and_template_entity_name", at the last 2 commented lines which fail 
        // for unknown reason.  Maybe something obvious i'm just
        // missing, or maybe it's in the postgresql or jdbc transaction docs.  Could also ck in other places 
        // calling db.rollback_trans to see what's to learn from
        // current use (risk) & behaviors to compare.
        //    assert(db.getAttrCount(id) == 0)
        //    assert(db.get_attribute_sorting_rows_count(Some(id)) == 0)
    }

    /// Returns the group_id, and the rtg_id.
    /// In scala, this file was in the core package (not in the test directory), so that by being included in the .jar,
    /// it is available for use by the integration module (in RestDatabaseTest.scala).
    /// (It was in core/src/model/database_test_utils.rs before that was converted to Rust, and in scala
    /// it was in core-scala/src/main/scala/org/onemodel/core/model/DatabaseTestUtils.scala.
    //fn create_and_add_test_relation_to_group_on_to_entity<'a, 'b>(db_in: &'a dyn Database,
    //fn create_and_add_test_relation_to_group_on_to_entity<'a, 'b, 'c>(
    fn create_and_add_test_relation_to_group_on_to_entity<'b, 'c>(
        db_in: Rc<dyn Database>,
        transaction: Option<Rc<RefCell<Transaction<'b, Postgres>>>>,
        in_entity: &'c Entity,
        in_rel_type_id: i64,
        in_group_name: &str,           /*= "something"*/
        in_valid_on_date: Option<i64>, /*= None*/
        allow_mixed_classes_in: bool,  /*= true*/
    ) -> Result<(i64, i64), anyhow::Error>
    where
        'c: 'b,
    {
        let observation_date: i64 = Utc::now().timestamp_millis();
        let (group_id, rtg_id) = in_entity.add_group_and_relation_to_group(
            transaction.clone(),
            in_rel_type_id,
            in_group_name,
            allow_mixed_classes_in,
            in_valid_on_date,
            observation_date,
            None,
        )?;
        let mut group = Group::new2(db_in.clone(), transaction.clone(), group_id)?;
        debug!("new group id = {}", group.get_id());

        let mut rtg =
            RelationToGroup::new3(db_in, transaction.clone(), rtg_id)?;

        // and verify it:
        match in_valid_on_date {
            None => assert!(rtg.get_valid_on_date(transaction.clone())?.is_none()),
            Some(vod) => {
                let in_dt: i64 = vod;
                let got_dt: i64 = rtg.get_valid_on_date(transaction.clone())?.unwrap();
                assert!(in_dt == got_dt);
            }
        }
        assert!(
            group
                .get_mixed_classes_allowed(transaction.clone())
                .unwrap()
                == allow_mixed_classes_in
        );
        assert!(group.get_name(transaction.clone()).unwrap() == in_group_name);
        assert!(rtg.get_observation_date(transaction.clone()).unwrap() == observation_date);
        Ok((group.get_id(), rtg.get_id()))
    }

    #[test]
    fn quantity_create_update_delete_methods() {
        Util::initialize_tracing();
        let db: Rc<PostgreSQLDatabase> = Rc::new(Util::initialize_test_db().unwrap());

        // Begin transaction
        let tx = db.begin_trans().unwrap();
        let tx: Option<Rc<RefCell<Transaction<Postgres>>>> = Some(Rc::new(RefCell::new(tx)));

        let starting_entity_count = db.get_entity_count(tx.clone()).unwrap();
        let entity_id = db
            .create_entity(
                tx.clone(),
                "test: org.onemodel.PSQLDbTest.quantityAttrs()",
                None,
                None,
            )
            .unwrap();
        let initial_total_sorting_rows_count = db
            .get_attribute_sorting_rows_count(tx.clone(), None)
            .unwrap();

        let quantity_attribute_id: i64 =
            create_test_quantity_attribute_with_two_entities(&db, tx.clone(), entity_id, None);
        assert!(
            db.get_attribute_sorting_rows_count(tx.clone(), None)
                .unwrap()
                > initial_total_sorting_rows_count
        );

        let mut qa = QuantityAttribute::new2(db.clone(), tx.clone(), quantity_attribute_id).unwrap();
        let (pid1, atid1, uid1) = (
            qa.get_parent_id(tx.clone()).unwrap(),
            qa.get_attr_type_id(tx.clone()).unwrap(),
            qa.get_unit_id(tx.clone()).unwrap(),
        );
        assert!(entity_id == pid1);

        db.update_quantity_attribute(
            tx.clone(),
            quantity_attribute_id,
            pid1,
            atid1,
            uid1,
            4.0,
            Some(5),
            6,
        )
        .unwrap();

        // Re-read the data by creating a new instance
        let mut qa2 = QuantityAttribute::new2(db.clone(), tx.clone(), quantity_attribute_id).unwrap();
        let (pid2, atid2, uid2, num2, vod2, od2) = (
            qa2.get_parent_id(tx.clone()).unwrap(),
            qa2.get_attr_type_id(tx.clone()).unwrap(),
            qa2.get_unit_id(tx.clone()).unwrap(),
            qa2.get_number(tx.clone()).unwrap(),
            qa2.get_valid_on_date(tx.clone()).unwrap(),
            qa2.get_observation_date(tx.clone()).unwrap(),
        );

        assert_eq!(pid2, pid1);
        assert_eq!(atid2, atid1);
        assert_eq!(uid2, uid1);
        assert_eq!(num2, 4.0);
        assert_eq!(vod2, Some(5 as i64));
        assert_eq!(od2, 6);

        let q_attr_count = db
            .get_quantity_attribute_count(tx.clone(), entity_id)
            .unwrap();
        assert_eq!(q_attr_count, 1);
        assert_eq!(
            db.get_attribute_sorting_rows_count(tx.clone(), Some(entity_id))
                .unwrap()
               , 1
        );

        // Delete the quantity attribute and check correctness
        let entity_count_before_quantity_deletion: u64 = db.get_entity_count(tx.clone()).unwrap();
        db.delete_quantity_attribute(tx.clone(), quantity_attribute_id)
            .unwrap();

        // next 2 assert! lines should work because of the database logic (triggers as of this writing)
        // that removes sorting rows when attrs are removed):
        assert_eq!(
            db.get_attribute_sorting_rows_count(tx.clone(), None)
                .unwrap()
               , initial_total_sorting_rows_count
        );
        assert_eq!(
            db.get_attribute_sorting_rows_count(tx.clone(), Some(entity_id))
                .unwrap()
               , 0
        );

        let entity_count_after_quantity_deletion: u64 = db.get_entity_count(tx.clone()).unwrap();
        assert_eq!(
            db.get_quantity_attribute_count(tx.clone(), entity_id)
                .unwrap()
               , 0
        );

        if entity_count_after_quantity_deletion != entity_count_before_quantity_deletion {
            panic!(
            "Got constraint backwards? Deleting quantity attribute changed Entity count from {} to {}",
            entity_count_before_quantity_deletion, entity_count_after_quantity_deletion
        );
        }

        db.delete_entity(tx.clone(), entity_id).unwrap();
        let ending_entity_count = db.get_entity_count(tx.clone()).unwrap();
        // 2 more entities came during quantity creation (units & quantity type), it's OK to leave 
        // them in this kind of situation)
        assert_eq!(ending_entity_count, starting_entity_count + 2);
        assert_eq!(
            db.get_quantity_attribute_count(tx.clone(), entity_id)
                .unwrap()
               , 0
        );

        // Rollback transaction (handled automatically when tx goes out of scope)
        // No explicit rollback needed, as per sqlx docs.
    }

    #[test]
    fn attribute_and_attribute_sorting_row_deletion_both_happen_automatically_upon_entity_deletion() {
        Util::initialize_tracing();
        let db: Rc<PostgreSQLDatabase> = Rc::new(Util::initialize_test_db().unwrap());
        //Not needing to deal w/ complexity of a tx in this method.
        //let tx = db.begin_trans().unwrap();
        //let tx: Option<Rc<RefCell<Transaction<Postgres>>>> = Some(Rc::new(RefCell::new(tx)));
        let entity_id = db.create_entity(None, "test: org.onemodel.PSQLDbTest sorting rows stuff", None, None)
            .unwrap();
        create_test_quantity_attribute_with_two_entities(&db, None, entity_id, None);
        assert_eq!(
            db.get_attribute_sorting_rows_count(None, Some(entity_id)).unwrap(),
            1
        );
        assert_eq!(
            db.get_quantity_attribute_count(None, entity_id).unwrap(),
            1
        );
        db.delete_entity(None, entity_id).unwrap();
        assert_eq!(
            db.get_attribute_sorting_rows_count(None, Some(entity_id)).unwrap(),
            0
        );
        assert_eq!(
            db.get_quantity_attribute_count(None, entity_id).unwrap(),
            0
        );
        // no need to db.rollback_trans(), because that is automatic when tx goes out of scope, per sqlx docs.
    }

    #[test]
    fn text_attribute_create_delete_update_methods() {
        Util::initialize_tracing();
        let db: Rc<PostgreSQLDatabase> = Rc::new(Util::initialize_test_db().unwrap());
        let tx = db.begin_trans().unwrap();
        let tx: Option<Rc<RefCell<Transaction<Postgres>>>> = Some(Rc::new(RefCell::new(tx)));

        let starting_entity_count = db.get_entity_count(tx.clone()).unwrap();
        let entity_id = db.create_entity(tx.clone(), "test: org.onemodel.PSQLDbTest.testTextAttrs", None, None).unwrap();
        assert_eq!(db.get_attribute_sorting_rows_count(tx.clone(), Some(entity_id)).unwrap(), 0);
        let text_attribute_id: i64 = create_test_text_attribute_with_one_entity(&db, tx.clone(), entity_id, None);
        assert_eq!(db.get_attribute_sorting_rows_count(tx.clone(), Some(entity_id)).unwrap(), 1);
        let a_text_value = "asdfjkl";

        let mut ta = TextAttribute::new2(db.clone(), tx.clone(), text_attribute_id).unwrap();
        let (pid1, atid1) = (ta.get_parent_id(tx.clone()).unwrap(), ta.get_attr_type_id(tx.clone()).unwrap());
        assert_eq!(entity_id, pid1);
        db.update_text_attribute(tx.clone(), text_attribute_id, pid1, atid1, a_text_value, Some(123), 456).unwrap();
        // have to create new instance to re-read the data: immutability makes programs easier to work with
        let mut ta2 = TextAttribute::new2(db.clone(), tx.clone(), text_attribute_id).unwrap();
        let pid2 = ta2.get_parent_id(tx.clone()).unwrap();
        let atid2 = ta2.get_attr_type_id(tx.clone()).unwrap();
        {
            let txt2 = ta2.get_text(tx.clone()).unwrap();
            assert_eq!(txt2, a_text_value);
        //}
        //{
            let vod2 = ta2.get_valid_on_date(tx.clone()).unwrap();
            assert_eq!(vod2, Some(123i64));
        }
        let od2 = ta2.get_observation_date(tx.clone()).unwrap();
        assert_eq!(pid2, pid1);
        assert_eq!(atid2, atid1);
        assert_eq!(od2, 456);
        assert_eq!(db.get_text_attribute_count(tx.clone(), entity_id).unwrap(), 1);

        let entity_count_before_text_deletion: u64 = db.get_entity_count(tx.clone()).unwrap();
        db.delete_text_attribute(tx.clone(), text_attribute_id).unwrap();
        assert_eq!(db.get_text_attribute_count(tx.clone(), entity_id).unwrap(), 0);
        // next line should work because of the database logic (triggers as of this writing) 
        // that removes sorting rows when attrs are removed):
        assert_eq!(db.get_attribute_sorting_rows_count(tx.clone(), Some(entity_id)).unwrap(), 0);
        let entity_count_after_text_deletion: u64 = db.get_entity_count(tx.clone(), ).unwrap();
        if entity_count_after_text_deletion != entity_count_before_text_deletion {
            panic!("Got constraint backwards? Deleting text attribute changed Entity count from {} to {}",
                entity_count_before_text_deletion, 
                entity_count_after_text_deletion);
        }
        // then recreate the text attribute (to verify its auto-deletion when Entity is deleted, below)
        create_test_text_attribute_with_one_entity(&db, tx.clone(), entity_id, None);
        db.delete_entity(tx.clone(), entity_id).unwrap();
        if db.get_text_attribute_count(tx.clone(), entity_id).unwrap() > 0 {
            panic!("Deleting the model entity should also have deleted its text \
                attributes; get_text_attribute_count(entity_idInNewTransaction) is {}.", 
                db.get_text_attribute_count(tx.clone(), entity_id).unwrap());
        }

        let ending_entity_count = db.get_entity_count(tx.clone()).unwrap();
        // 2 more entities came during text attribute creation, which we don't care about either way, for this test
        assert_eq!(ending_entity_count, starting_entity_count + 2);
      }

    #[test]
    fn date_attribute_create_delete_update_methods() {
        Util::initialize_tracing();
        let db: Rc<PostgreSQLDatabase> = Rc::new(Util::initialize_test_db().unwrap());
        
        // If tests are run in parallel, we probably need to start using transactions in this test.
        // See other examples, and parameters to all fns. Some might need a transaction parameter
        // to be added.
        let starting_entity_count = db.get_entity_count(None).unwrap();
        let entity_id = db
            .create_entity(
                None,
                "test: org.onemodel.PSQLDbTest.testDateAttrs",
                None,
                None,
            )
            .unwrap();
        assert_eq!(db.get_attribute_sorting_rows_count(None, Some(entity_id)).unwrap(), 0);
        let date_attribute_id: i64 = create_test_date_attribute_with_one_entity(&db, entity_id);
        assert_eq!(db.get_attribute_sorting_rows_count(None, Some(entity_id)).unwrap(), 1);
        
        let mut da = DateAttribute::new2(db.clone(), None, date_attribute_id).unwrap();
        let (pid1, atid1) = (da.get_parent_id(None).unwrap(), da.get_attr_type_id(None).unwrap());
        assert_eq!(entity_id, pid1);
        
        let date = Utc::now().timestamp_millis();
        db.update_date_attribute(None, date_attribute_id, pid1, date, atid1).unwrap();
        
        // Have to create new instance to re-read the data: immutability makes the program easier to debug/reason about.
        let mut da2 = DateAttribute::new2(db.clone(), None, date_attribute_id).unwrap();
        let (pid2, atid2, date2) = (
            da2.get_parent_id(None).unwrap(),
            da2.get_attr_type_id(None).unwrap(),
            da2.get_date(None).unwrap()
        );
        assert_eq!(pid2, pid1);
        assert_eq!(atid2, atid1);
        assert_eq!(date2, date);
        
        // Also test the other constructor.
        let mut da3 = DateAttribute::new(db.clone(), date_attribute_id, pid1, atid1, date, 0);
        let (pid3, atid3, date3) = (
            da3.get_parent_id(None).unwrap(),
            da3.get_attr_type_id(None).unwrap(),
            da3.get_date(None).unwrap()
        );
        assert_eq!(pid3, pid1);
        assert_eq!(atid3, atid1);
        assert_eq!(date3, date);
        
        assert_eq!(db.clone().get_date_attribute_count(None, entity_id).unwrap(), 1);
        
        let entity_count_before_date_deletion: u64 = db.get_entity_count(None).unwrap();
        db.delete_date_attribute(None, date_attribute_id).unwrap();
        assert_eq!(db.get_date_attribute_count(None, entity_id).unwrap(), 0);
        
        // next line should work because of the database logic (triggers as of this writing) that removes sorting rows when attrs are removed):
        assert_eq!(db.get_attribute_sorting_rows_count(None, Some(entity_id)).unwrap(), 0);
        assert_eq!(db.get_entity_count(None).unwrap(), entity_count_before_date_deletion);
        
        // then recreate the attribute (to verify its auto-deletion when Entity is deleted, below)
        create_test_date_attribute_with_one_entity(&db, entity_id);
        db.delete_entity(None, entity_id).unwrap();
        assert_eq!(db.get_date_attribute_count(None, entity_id).unwrap(), 0);
        
        // 2 more entities came during attribute creation, which we don't care about either way, for this test
        assert_eq!(db.get_entity_count(None).unwrap(), starting_entity_count + 2);
    }
    
    #[test]
    fn boolean_create_delete_update_methods() {
        Util::initialize_tracing();
        let db: Rc<PostgreSQLDatabase> = Rc::new(Util::initialize_test_db().unwrap());
        let tx = db.begin_trans().unwrap();
        let tx: Option<Rc<RefCell<Transaction<Postgres>>>> = Some(Rc::new(RefCell::new(tx)));

        let starting_entity_count = db.get_entity_count(tx.clone()).unwrap();
        let entity_id = db
            .create_entity(
                tx.clone(),
                "test: org.onemodel.PSQLDbTest.testBooleanAttrs",
                None,
                None,
            )
            .unwrap();
        let val1 = true;
        let observation_date: i64 = Utc::now().timestamp_millis();
        let valid_on_date: Option<i64> = Some(1234);
        assert!(
            db.get_attribute_sorting_rows_count(tx.clone(), Some(entity_id))
                .unwrap()
                == 0
        );
        let boolean_attribute_id: i64 = create_test_boolean_attribute_with_one_entity(
            &db,
            tx.clone(),
            entity_id,
            val1,
            valid_on_date,
            observation_date,
        );
        assert!(
            db.get_attribute_sorting_rows_count(tx.clone(), Some(entity_id))
                .unwrap()
                == 1
        );

        let mut ba = BooleanAttribute::new2(db.clone(), tx.clone(), boolean_attribute_id).unwrap();
        let (pid1, atid1) = (
            ba.get_parent_id(tx.clone()).unwrap(),
            ba.get_attr_type_id(tx.clone()).unwrap(),
        );
        assert!(entity_id == pid1);

        let val2 = false;
        db.update_boolean_attribute(
            tx.clone(),
            boolean_attribute_id,
            pid1,
            atid1,
            val2,
            Some(123),
            456,
        )
        .unwrap();
        // have to create new instance to re-read the data:
        let mut ba2 = BooleanAttribute::new2(db.clone(), tx.clone(), boolean_attribute_id).unwrap();
        let (pid2, atid2, bool2, vod2, od2) = (
            ba2.get_parent_id(tx.clone()).unwrap(),
            ba2.get_attr_type_id(tx.clone()).unwrap(),
            ba2.get_boolean(tx.clone()).unwrap(),
            ba2.get_valid_on_date(tx.clone()).unwrap(),
            ba2.get_observation_date(tx.clone()).unwrap(),
        );
        assert!(pid2 == pid1);
        assert!(atid2 == atid1);
        assert!(bool2 == val2);
        assert!(vod2 == Some(123));
        assert!(od2 == 456);

        assert!(
            db.get_boolean_attribute_count(tx.clone(), entity_id)
                .unwrap()
                == 1
        );

        let entity_count_before_attr_deletion: u64 = db.get_entity_count(tx.clone()).unwrap();
        db.delete_boolean_attribute(tx.clone(), boolean_attribute_id)
            .unwrap();
        assert!(
            db.get_boolean_attribute_count(tx.clone(), entity_id)
                .unwrap()
                == 0
        );
        // Next line should work because of the database logic (triggers as of this writing)
        // that removes sorting rows when attrs are removed):
        assert!(
            db.get_attribute_sorting_rows_count(tx.clone(), Some(entity_id))
                .unwrap()
                == 0
        );
        let entity_count_after_attr_deletion: u64 = db.get_entity_count(tx.clone()).unwrap();
        if entity_count_after_attr_deletion != entity_count_before_attr_deletion {
            panic!("Got constraint backwards? Deleting boolean attribute changed Entity count from {} to {}",
               entity_count_before_attr_deletion, entity_count_after_attr_deletion);
        }

        // then recreate the attribute (to verify its auto-deletion when Entity is deleted, below; and to verify behavior with other values)
        let testval2: bool = true;
        let valid_on_date2: Option<i64> = None;
        let bool_attribute_id2: i64 = db
            .create_boolean_attribute(
                tx.clone(),
                pid1,
                atid1,
                testval2,
                valid_on_date2,
                observation_date,
                None,
            )
            .unwrap();
        let mut ba3: BooleanAttribute =
            BooleanAttribute::new2(db.clone(), tx.clone(), bool_attribute_id2).unwrap();
        assert!(ba3.get_boolean(tx.clone()).unwrap() == testval2);
        assert!(ba3.get_valid_on_date(tx.clone()).unwrap().is_none());
        db.delete_entity(tx.clone(), entity_id).unwrap();
        assert!(
            db.get_boolean_attribute_count(tx.clone(), entity_id)
                .unwrap()
                == 0
        );

        let ending_entity_count = db.get_entity_count(tx).unwrap();
        // 2 more entities came during attribute creation, but we deleted one and (unlike similar tests) didn't recreate it.
        assert!(ending_entity_count == starting_entity_count + 1)

        // no need to db.rollback_trans(), because that is automatic when tx goes out of scope, per sqlx docs.
        // (if the transaction is even needed here; arguably not.)
    }

    /*%%
    // for a test just below
    %%file_attr stuff:
    MAYBE CAN make this a parameter instead, wherever used? see fn just below, add as parm there.
    private let mut mDoDamageBuffer = false;

    // instantiation does DB setup (creates tables, default data, etc):
    private let db: PostgreSQLDatabase = new PostgreSQLDatabase(Database.TEST_USER, Database.TEST_PASS) {
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
        let entity_id: i64 = db.create_entity("someent");
        let attr_type_id: i64 = db.create_entity("fileAttributeType");
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
                                                db.create_file_attribute(entity_id, attr_type_id, "xyz", 0, 0, "/doesntmatter", readable_in = true,
                                                                        writable_in = true, executable_in = false, uploadSourceFile.length(),
                                                                        FileAttribute::md5_hash(uploadSourceFile), inputStream, Some(0))
                                              }
          mDoDamageBuffer = false
          //so it should work now:
          inputStream = new java.io.FileInputStream(uploadSourceFile)
          let faId: i64 = db.create_file_attribute(entity_id, attr_type_id, "xyz", 0, 0,;
                                                   "/doesntmatter", readable_in = true, writable_in = true, executable_in = false,
                                                   uploadSourceFile.length(), FileAttribute::md5_hash(uploadSourceFile), inputStream, None)

          let fa: FileAttribute = new FileAttribute(db, faId);
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
*/
#[test]
fn relation_to_entity_methods_and_relation_type_methods() {
    Util::initialize_tracing();
    let db: Rc<PostgreSQLDatabase> = Rc::new(Util::initialize_test_db().unwrap());
    let tx = None;
    
    let starting_entity_only_count = db.get_entities_only_count(tx.clone(), false, None, None).unwrap();
    let starting_relation_type_count = db.get_relation_type_count(tx.clone()).unwrap();
    let entity_id = db.create_entity(tx.clone(), "test: org.onemodel.PSQLDbTest.testRelsNRelTypes()", None, None).unwrap();
    let starting_rel_count = db.get_relation_types(db.clone(), tx.clone(), 0, Some(25)).unwrap().len();
    let rel_type_id: i64 = db.create_relation_type(tx.clone(), "contains", "", RelationType::UNIDIRECTIONAL).unwrap();

    // Verify a bugfix from 2013-10-31 or 2013-11-4 in how SELECT is written.
    assert_eq!(db.get_relation_types(db.clone(), tx.clone(), 0, Some(25)).unwrap().len(), starting_rel_count + 1);
    assert_eq!(db.get_entities_only_count(tx.clone(), false, None, None).unwrap(), starting_entity_only_count + 1);

    assert_eq!(db.get_attribute_sorting_rows_count(tx.clone(), Some(entity_id)).unwrap(), 0);

    let related_entity_id: i64 = create_test_relation_to_local_entity_with_one_entity(&db, tx.clone(), entity_id, rel_type_id, None);
    assert_eq!(db.get_attribute_sorting_rows_count(tx.clone(), Some(entity_id)).unwrap(), 1);

    let check_relation = db.get_relation_to_local_entity_data(tx.clone(), rel_type_id, entity_id, related_entity_id).unwrap();
    let check_valid_on_date = check_relation.get(1).unwrap();
    // should get back None when created with None: see description for table's field in create_tables method.
    assert!(check_valid_on_date.is_none());
    assert_eq!(db.get_relation_to_local_entity_count(tx.clone(), entity_id, true).unwrap(), 1);

    let new_name = "test: org.onemodel.PSQLDbTest.relationupdate...";
    let name_in_reverse = "nameinreverse;!@#$%^&*()-_=+{}[]:\"'<>?,./`~"; // And verify can handle some variety of chars;
    
    db.update_relation_type(rel_type_id, new_name, name_in_reverse, RelationType::BIDIRECTIONAL).unwrap();
    
    // Have to create new instance to re-read the data:
    let mut updated_relation_type = RelationType::new2(db.clone(), tx.clone(), rel_type_id).unwrap();
    assert_eq!(updated_relation_type.get_name(tx.clone()).unwrap(), new_name);
    assert_eq!(updated_relation_type.get_name_in_reverse_direction(tx.clone()).unwrap(), name_in_reverse);
    assert_eq!(updated_relation_type.get_directionality(tx.clone()).unwrap(), RelationType::BIDIRECTIONAL);

    db.delete_relation_to_local_entity(tx.clone(), rel_type_id, entity_id, related_entity_id).unwrap();
    assert_eq!(db.get_relation_to_local_entity_count(tx.clone(), entity_id, true).unwrap(), 0);
    // Next line should work because of the database logic (triggers as of this writing) that removes sorting rows when attrs are removed:
    assert_eq!(db.get_attribute_sorting_rows_count(tx.clone(), Some(entity_id)).unwrap(), 0);

    let entity_only_count_before_relation_type_deletion: u64 = db.get_entities_only_count(tx.clone(), false, None, None).unwrap();
    db.delete_relation_type(tx.clone(), rel_type_id).unwrap();
    assert_eq!(db.get_relation_type_count(tx.clone()).unwrap(), starting_relation_type_count);
    // Ensure that removing rel type doesn't remove more entities than it should, and that the 'onlyCount' works right.
    // i.e. as above, verify a bugfix from 2013-10-31 or 2013-11-4 in how SELECT is written.
    assert_eq!(entity_only_count_before_relation_type_deletion, db.get_entities_only_count(tx.clone(), false, None, None).unwrap());

    db.delete_entity(tx.clone(), entity_id).unwrap();
}
    #[test]
    fn get_containing_groups_ids_finds_groups_containing_the_test_group() {
        Util::initialize_tracing();
        let db: Rc<PostgreSQLDatabase> = Rc::new(Util::initialize_test_db().unwrap());
        // Using None for simplicity for now but if tests run in parallel, might
        // need a real one, like in other examples.
        //let tx = None; 
        /*This makes a thing like this:   entity1    entity3
                                             |         |
                                          group1     group3
                                             |         |
                                              \       /
                                               entity2
                                                  |
                                               group2
         ...(and then checks in the middle that entity2 has 1 containing group, before adding entity3/group3)
         ...and then checks that entity2 has 2 containing groups. */
        let entity_id1 = db
            .create_entity(
                None, //tx.clone(),
                "test-get_containing_groups_ids-entity1",
                None,
                None,
            )
            .unwrap();
        let entity1 = Entity::new2(db.clone(), None /*tx.clone()*/, entity_id1).unwrap();
        let rel_type_id: i64 = db.clone()
            .create_relation_type(
                None, //tx.clone(),
                "test-get_containing_groups_ids-reltype1",
                "",
                RelationType::UNIDIRECTIONAL,
            )
            .unwrap();
        let (group_id1, _) = create_and_add_test_relation_to_group_on_to_entity(
            db.clone(),
            None, //tx.clone(),
            &entity1,
            rel_type_id,
            "test-get_containing_groups_ids-group1",
            None,
            true,
        ).unwrap();
        let group1 = Group::new2(db.clone(), None /*tx.clone()*/, group_id1).unwrap();
        
        let entity_id2 = db.clone()
            .create_entity(
                None, //tx.clone(),
                "test-get_containing_groups_ids-entity2",
                None,
                None,
            )
            .unwrap();
        let entity2 = Entity::new2(db.clone(), None /*tx.clone()*/, entity_id2).unwrap();
        group1.add_entity(None /*tx.clone()*/, entity_id2, None).unwrap();
        let (group_id2, _) = create_and_add_test_relation_to_group_on_to_entity(
            db.clone(),
            None, //tx.clone(),
            &entity2,
            rel_type_id,
            "test-get_containing_groups_ids-group2",
            None,
            true,
        ).unwrap();
        let group2 = Group::new2(db.clone(), None /*tx.clone()*/, group_id2).unwrap();
        // this is a list of rows (query results) of (in this case) one element in each row.
        let containing_groups: Vec<Vec<Option<DataType>>> = 
            db.get_groups_containing_entitys_groups_ids(None /*tx.clone()*/, group2.get_id(), Some(5)).unwrap();
        assert_eq!(containing_groups.len(), 1);
        let first_group_id: i64 = match containing_groups[0].clone()[0].clone().unwrap() {
            DataType::Bigint(x) => x,
            _ => panic!("Unexpected value from query: {:?}", containing_groups),
        };
        assert_eq!(first_group_id, group_id1);
        
        let entity_id3 = db
            .create_entity(
                None, //tx.clone(),
                "test-get_containing_groups_ids-entity3",
                None,
                None,
            )
            .unwrap();
        let entity3 = Entity::new2(db.clone(), None /*tx.clone()*/, entity_id3).unwrap();
        let (group_id3, _) = create_and_add_test_relation_to_group_on_to_entity(
            db.clone(),
            None, //tx.clone(),
            &entity3,
            rel_type_id,
            "test-get_containing_groups_ids-group3",
            None,
            true,
        ).unwrap();
        
        let group3 = Group::new2(db.clone(), None /*tx.clone()*/, group_id3).unwrap();
        group3.add_entity(None /*tx.clone()*/, entity_id2, None).unwrap();
        
        let containing_groups2: Vec<Vec<Option<DataType>>> = 
            db.get_groups_containing_entitys_groups_ids(None /*tx.clone()*/, group2.get_id(), Some(5)).unwrap();
        assert_eq!(containing_groups2.len(), 2);
        let first_group_id: i64 = match containing_groups2[0].clone()[0].clone().unwrap() {
            DataType::Bigint(x) => x,
            _ => panic!("Unexpected value from row 0: {:?}", containing_groups2),
        };
        let second_group_id: i64 = match containing_groups2[1].clone()[0].clone().unwrap() {
            DataType::Bigint(x) => x,
            _ => panic!("Unexpected value from row 1: {:?}", containing_groups2),
        };
        assert_eq!(first_group_id, group_id1);
        assert_eq!(second_group_id, group_id3);
    }

#[test]
fn relation_to_group_and_group_methods() -> Result<(), Box<dyn std::error::Error>> {
    Util::initialize_tracing();
    let db: Rc<PostgreSQLDatabase> = Rc::new(Util::initialize_test_db()?);
    let rel_to_group_name = "test: PSQLDbTest.testRelsNRelTypes()";
    let entity_name = format!("{}--theEntity", rel_to_group_name);
    
    //Creating things without a transaction in the several lines just below, seemingly because
    //the compiler forced me to specify lifetimes that force these things to be created before
    //their transaction, otherwise I get error 0597, saying vars are dropped in order they are
    //created. To reproduce, move one of the lines, like the last one, to below the "let tx = None"
    //line.
    //let entity_id = db.create_entity(tx.clone(), entity_name.as_str(), None, None)?;
    let entity_id = db.clone().create_entity(None, entity_name.as_str(), None, None)?;
    //let entity = Entity::new2(db.clone(), tx.clone(), entity_id).unwrap();
    let entity = Entity::new2(db.clone(), None, entity_id).unwrap();
    //let rel_type_id = cloned.create_relation_type(tx.clone(), "contains", "", RelationType::UNIDIRECTIONAL)?;
    let rel_type_id = db.clone().create_relation_type(None, "contains", "", RelationType::UNIDIRECTIONAL)?;
    let valid_on_date = 12345;
    let (group_id, created_rtg_id) = create_and_add_test_relation_to_group_on_to_entity(
        //db.clone(), tx.clone(), &entity, rel_type_id, rel_to_group_name, Some(valid_on_date), true)?;
        db.clone(), None, &entity, rel_type_id, rel_to_group_name, Some(valid_on_date), true)?;
    //let mut group = Group::new2(db.clone(), tx.clone(), group_id).unwrap();
    let mut group = Group::new2(db.clone(), None, group_id).unwrap();
    let (group_id2, _) = create_and_add_test_relation_to_group_on_to_entity(
        //db.clone(), tx.clone(), &entity, rel_type_id, "somename", None, false
        db.clone(), None, &entity, rel_type_id, "somename", None, false
    )?;
    //let group2 = Group::new2(db.clone(), tx.clone(), group_id2).unwrap();
    let group2 = Group::new2(db.clone(), None, group_id2).unwrap();

    // Was set to None, but variable exists in case we want to test with a transaction.
    // Other tests also check transactions. But having the transaction exposed a db deadlock
    // in call to group.delete_with_entities, below.
    let tx = db.begin_trans().unwrap();
    let tx: Option<Rc<RefCell<Transaction<Postgres>>>> = Some(Rc::new(RefCell::new(tx)));

    assert_eq!(db.get_attribute_sorting_rows_count(tx.clone(), Some(entity_id))?, 2);

    let mut rtg = RelationToGroup::new2(db.clone(), tx.clone(), created_rtg_id, entity_id, rel_type_id, group_id).unwrap();

    assert!(group.get_mixed_classes_allowed(tx.clone()).unwrap());
    assert_eq!(group.get_name(tx.clone()).unwrap(), rel_to_group_name);

    let check_relation = db.get_relation_to_group_data_by_keys(tx.clone(), rtg.get_parent_id(tx.clone()).unwrap(), rtg.get_attr_type_id(tx.clone()).unwrap(), rtg.get_group_id(tx.clone()).unwrap()).unwrap();
    if let DataType::Bigint(x) = check_relation[0].as_ref().unwrap() {
        assert_eq!(*x, rtg.get_id());
    } else {
        panic!("How did we get here with {:?}?", check_relation[0]);
    }
    if let DataType::Bigint(x) = check_relation[1].as_ref().unwrap() {
        assert_eq!(*x, entity_id);
    } else {
        panic!("How did we get here with {:?}?", check_relation[1]);
    }
    if let DataType::Bigint(x) = check_relation[2].as_ref().unwrap() {
        assert_eq!(*x, rel_type_id);
    } else {
        panic!("How did we get here with {:?}?", check_relation[2]);
    }
    if let DataType::Bigint(x) = check_relation[3].as_ref().unwrap() {
        assert_eq!(*x, group_id);
    } else {
        panic!("How did we get here with {:?}?", check_relation[3]);
    }
    if let DataType::Bigint(x) = check_relation[4].as_ref().unwrap() {
        assert_eq!(*x, valid_on_date);
    } else {
        panic!("How did we get here with {:?}?", check_relation[4]);
    }
    let check_again = db.get_relation_to_group_data(tx.clone(), rtg.get_id())?;
    if let DataType::Bigint(x) = check_again[0].as_ref().unwrap() {
        assert_eq!(*x, rtg.get_id());
    } else {
        panic!("How did we get here with {:?}?", check_again[0]);
    }
    if let DataType::Bigint(x) = check_again[1].as_ref().unwrap() {
        assert_eq!(*x, entity_id);
    } else {
        panic!("How did we get here with {:?}?", check_again[1]);
    }
    if let DataType::Bigint(x) = check_again[2].as_ref().unwrap() {
        assert_eq!(*x, rel_type_id);
    } else {
        panic!("How did we get here with {:?}?", check_again[2]);
    }
    if let DataType::Bigint(x) = check_again[3].as_ref().unwrap() {
        assert_eq!(*x, group_id);
    } else {
        panic!("How did we get here with {:?}?", check_again[3]);
    }
    if let DataType::Bigint(x) = check_again[4].as_ref().unwrap() {
        assert_eq!(*x, valid_on_date);
    } else {
        panic!("How did we get here with {:?}?", check_again[4]);
    }
    assert_eq!(group.get_size(tx.clone(), 3).unwrap(), 0);

    let entity_id2 = db.create_entity(tx.clone(), format!("{}2", entity_name).as_str(), None, None).unwrap();
    group.add_entity(tx.clone(), entity_id2, None).unwrap();
    //group.add_entity(None, entity_id2, None).unwrap();
    assert_eq!(group.get_size(tx.clone(), 3)?, 1);

    group.delete_with_entities(tx.clone()).unwrap();

    let result = RelationToGroup::new2(db.clone(), tx.clone(), rtg.get_id(), rtg.get_parent_id(tx.clone()).unwrap(), rtg.get_attr_type_id(tx.clone()).unwrap(), rtg.get_group_id(tx.clone()).unwrap());
    assert!(result.is_err() && result.err().unwrap().to_string().contains("does not exist"));

    let result = Entity::new2(db.clone(), tx.clone(), entity_id2);
    assert!(result.is_err() && result.err().unwrap().to_string().contains("does not exist"));

    assert_eq!(group.get_size(tx.clone(), 3).unwrap(), 0);

    // next line should work because of the database logic (triggers as of this writing) that 
    // removes sorting rows when attrs are removed):
    assert_eq!(db.clone().get_attribute_sorting_rows_count(tx.clone(), Some(entity_id)).unwrap(), 1);
    
    assert_eq!(group2.get_size(tx.clone(), 3).unwrap(), 0);

    let entity_id3 = db.create_entity(tx.clone(), format!("{}3", entity_name).as_str(), None, None).unwrap();
    group2.add_entity(tx.clone(), entity_id3, None).unwrap();
    assert_eq!(group2.get_size(tx.clone(), 3).unwrap(), 1);

    let entity_id4 = db.clone().create_entity(tx.clone(), format!("{}4", entity_name).as_str(), None, None).unwrap();
    group2.add_entity(tx.clone(), entity_id4, None).unwrap();

    let entity_id5 = db.create_entity(tx.clone(), format!("{}5", entity_name).as_str(), None, None).unwrap();
    group2.add_entity(tx.clone(), entity_id5, None).unwrap();

    db.get_group_entry_sorting_index(tx.clone(), group_id2, entity_id5).unwrap();

    assert_eq!(group2.get_size(tx.clone(), 3).unwrap(), 3);
    assert_eq!(db.get_group_entry_ids(tx.clone(), group2.get_id(), 0, None).unwrap().len(), 3);

    group2.remove_entity(tx.clone(), entity_id5).unwrap();
    assert_eq!(db.get_group_entry_ids(tx.clone(), group2.get_id(), 0, None).unwrap().len(), 2);

    group2.delete(tx.clone()).unwrap();
    let result = Group::new2(db.clone(), tx.clone(), group_id2);
    assert!(result.is_err() && result.err().unwrap().to_string().contains("does not exist"));
    assert_eq!(group2.get_size(tx.clone(), 3).unwrap(), 0);

    // ensure the other entity still exists: not deleted by that delete command
    let _entity6 = Entity::new2(db.clone(), tx.clone(), entity_id3).unwrap();

    // Idea?: old comments: 
    // probably revise this later for use when adding that update method:
                               //val new_name = "test: org.onemodel.PSQLDbTest.relationupdate..."
                               //db.update_relation_type(rel_type_id, new_name, name_in_reverse, RelationType.BIDIRECTIONAL)
                               //// have to create new instance to re-read the data:
                               //val updatedRelationType = new RelationType(db, rel_type_id)
                               //assert(updatedRelationType.get_name == new_name)
                               //assert(updatedRelationType.get_name_in_reverse_direction == name_in_reverse)
                               //assert(updatedRelationType.get_directionality == RelationType.BIDIRECTIONAL)

    assert_eq!(db.get_relation_to_group_count(tx.clone(), entity_id)?, 0);
 
    Ok(())
}

    /*Just some leftover experimental test code once used to help isolate an issue:
#[test]
fn test_lifetime_issue() -> Result<(), Box<dyn std::error::Error>> {
    //why does the order of next 2 lines matter?? Or only sometimes?
    let tx = None;
    //{
    //let db: PostgreSQLDatabase = Util::initialize_test_db()?;
    let db: String = "1234".to_string();
    //{
    //let test_struct_instance = TestStruct::new2(&db, tx.clone(), entity_id).unwrap();
    let test_struct_instance = TestStruct::new2(&db, tx.clone()).unwrap();
    let (_group_id, _created_rtg_id) = fn4(&db, tx.clone(), &test_struct_instance).unwrap();
    //}}
    //let test_struct_instance = TestStruct{};
    //test_struct_instance.fn2(tx.clone());
    Ok(())
}
struct TestStruct<'a>{
    //db: &'a dyn Database,
    //db: &'a dyn String,
    db: &'a String,
}
impl TestStruct<'_>{
    pub fn new2<'a, 'b>(
        //db: &'a dyn Database,
        db: &'a String,
        _transaction: Option<Rc<RefCell<Transaction<'b, Postgres>>>>,
    ) -> Result<TestStruct<'a>, anyhow::Error> 
    where
        'a: 'b
    {
        Ok(TestStruct{
            db
        })
    }
}
// */



    #[test]
    fn get_groups_works() {
        Util::initialize_tracing();
        let db: Rc<PostgreSQLDatabase> = Rc::new(Util::initialize_test_db().unwrap());
        //Using None instead of tx here for simplicity, but probably would have to change if 
        //running tests in parallel.
        //let tx = db.begin_trans().unwrap();
        //let tx = Some(Rc::new(RefCell::new(tx)));
        let group3id = db.create_group(None /*tx.clone()*/, "g3", false).unwrap();
        let number = db.get_groups(db.clone(), None /*tx.clone()*/, 0, None, None).unwrap().len();
        let number2 = db.get_groups(db.clone(), None /*tx.clone()*/, 0, None, Some(group3id)).unwrap().len();
        assert_eq!(number, number2 + 1);
        let number3 = db.get_groups(db.clone(), None /*tx.clone()*/, 1, None, None).unwrap().len();
        assert_eq!(number, number3 + 1);
    }

    #[test]
    fn deleting_entity_works_even_if_entity_is_in_a_relationtogroup() {
        Util::initialize_tracing();
        let db: Rc<PostgreSQLDatabase> = Rc::new(Util::initialize_test_db().unwrap());
        //Using None instead of tx here for simplicity, but might have to change if 
        //running tests in parallel.
        //let tx = db.begin_trans().unwrap();
        //let tx = Some(Rc::new(RefCell::new(tx)));
        
        let starting_entity_count = db.get_entities_only_count(None, false, None, None).unwrap();
        let rel_to_group_name = "test:PSQLDbTest.testDelEntity_InGroup";
        let entity_name = format!("{}--theEntity", rel_to_group_name);
        let entity_id = db
            .create_entity(
                None,
                &entity_name.as_str(),
                None,
                None,
            )
            .unwrap();
        let entity = Entity::new2(db.clone(), None, entity_id).unwrap();
        let rel_type_id: i64 = db
            .create_relation_type(
                None,
                "contains",
                "",
                RelationType::UNIDIRECTIONAL,
            )
            .unwrap();
        let valid_on_date = Some(12345i64);
        let (group_id, _) = create_and_add_test_relation_to_group_on_to_entity(
            db.clone(),
            None,
            &entity,
            rel_type_id,
            rel_to_group_name,
            valid_on_date,
            true,
        ).unwrap();
        let group = Group::new2(db.clone(), None, group_id).unwrap();
        let entity_id1 = db
            .create_entity(
                None,
                &format!("{}{}", entity_name, 1),
                None,
                None,
            )
            .unwrap();
        group.add_entity(None, entity_id1, None).unwrap();
        assert_eq!(db.get_entities_only_count(None, false, None, None).unwrap(), starting_entity_count + 2);
        assert_eq!(db.get_group_size(None, group_id, 3).unwrap(), 1);
        
        let entity_id2 = db
            .create_entity(
                None,
                &format!("{}{}", entity_name, 2),
                None,
                None,
            )
            .unwrap();
        assert_eq!(db.get_entities_only_count(None, false, None, None).unwrap(), starting_entity_count + 3);
        assert_eq!(db.get_count_of_groups_containing_entity(None, entity_id2).unwrap(), 0);
        group.add_entity(None, entity_id2, None).unwrap();
        assert_eq!(db.get_group_size(None, group_id, 3).unwrap(), 2);
        assert_eq!(db.get_count_of_groups_containing_entity(None, entity_id2).unwrap(), 1);
        
        let descriptions = db.get_containing_relation_to_group_descriptions(None, entity_id2, Some(9999)).unwrap();
        assert_eq!(descriptions.len(), 1);
        assert_eq!(descriptions[0], format!("{}->{}", entity_name, rel_to_group_name));
        // Doesn't get an error
        db.delete_entity(None, entity_id2).unwrap();
        
        let descriptions2 = db.get_containing_relation_to_group_descriptions(None, entity_id2, Some(9999)).unwrap();
        assert_eq!(descriptions2.len(), 0);
        assert_eq!(db.get_count_of_groups_containing_entity(None, entity_id2).unwrap(), 0);
        assert_eq!(db.get_entities_only_count(None, false, None, None).unwrap(), starting_entity_count + 2);
        // Check that creating an Entity with deleted id returns error
        let result = Entity::new2(db.clone(), None, entity_id2);
        assert!(result.is_err());
        let error_message = format!("{}", result.err().unwrap());
        assert!(error_message.contains("does not exist"));
        
        assert_eq!(db.get_group_size(None, group_id, 3).unwrap(), 1);
        
        let list = db.get_group_entry_ids(None, group_id, 0, None).unwrap();
        assert_eq!(list.len(), 1);
        let remaining_contained_entity_id = list[0];
        //Ensure the first entities still exist: not deleted by that delete command
        let result = Entity::new2(db.clone(), None, entity_id);
        assert!(result.is_ok());
        let result = Entity::new2(db.clone(), None, remaining_contained_entity_id);
        assert!(result.is_ok());
    }

    #[test]
    fn get_sorted_attributes_returns_them_all_and_correctly() {
        Util::initialize_tracing();
        let db: Rc<PostgreSQLDatabase> = Rc::new(Util::initialize_test_db().unwrap());
        //Using None instead of tx here for simplicity, but might have to change if 
        //running tests in parallel. 
        //let tx = db.begin_trans().unwrap();
        //let tx = Some(Rc::new(RefCell::new(tx)));

        let entity_id = db
            .create_entity(None, "test: org.onemodel.PSQLDbTest.testRelsNRelTypes()", None, None)
            .unwrap();
        let entity = Entity::new2(db.clone(), None, entity_id).unwrap();
        create_test_text_attribute_with_one_entity(&db.clone(), None, entity_id, None);
        create_test_quantity_attribute_with_two_entities(&db.clone(), None, entity_id, None);
        let rel_type_id: i64 = db
            .create_relation_type(
                None,
                "contains",
                "",
                RelationType::UNIDIRECTIONAL,
            )
            .unwrap();
        let related_entity_id: i64 = create_test_relation_to_local_entity_with_one_entity(
            &db.clone(),
            None,
            entity_id,
            rel_type_id,
            None,
        );
        let (_, _) = create_and_add_test_relation_to_group_on_to_entity(
            db.clone(), None, &entity, rel_type_id, "test-relation-to-group", None, true,
        ).unwrap();
        // if using transactions, might have to add such a parameter to the next-called function:
        create_test_date_attribute_with_one_entity(&db.clone(), entity_id);
        create_test_boolean_attribute_with_one_entity(&db.clone(), None, entity_id, false, None, 0);
        //%%file_attr: latertests after FileAttribute is more completed.
        //create_test_file_attribute_and_one_entity(&entity, "desc", 2, false);
        db.update_entity_only_public_status(None, related_entity_id, None).unwrap();
        let (_, only_public_total_attrs_available1) = db.get_sorted_attributes(db.clone(), None, entity_id, 0, 999, true).unwrap();
        db.update_entity_only_public_status(None, related_entity_id, Some(false)).unwrap();
        let (_, only_public_total_attrs_available2) = db.get_sorted_attributes(db.clone(), None, entity_id, 0, 999, true).unwrap();
        db.update_entity_only_public_status(None, related_entity_id, Some(true)).unwrap();
        let (_, only_public_total_attrs_available3) = db.get_sorted_attributes(db.clone(), None, entity_id, 0, 999, true).unwrap();
        assert_eq!(only_public_total_attrs_available1, only_public_total_attrs_available2);
        assert_eq!((only_public_total_attrs_available3 - 1), only_public_total_attrs_available2);
        
        let (mut attr_tuples, total_attrs_available) = db.get_sorted_attributes(db.clone(), None, entity_id, 0, 999, false).unwrap();
        assert!(total_attrs_available > only_public_total_attrs_available1);
        let counter = attr_tuples.len();
        // Should be the same since we didn't create enough to span screens (requested them all)
        assert_eq!(counter, total_attrs_available);
        //if counter != 7 {
        //%%file_attr: latertests: at 6 until code for FileAttr is added to get_sorted_attributes etc
        if counter != 6 {
            panic!("We added attributes (RelationToLocalEntity, quantity & text, date, bool, file, RTG), but get_sorted_attributes() returned {}?", counter);
        }

        let mut found_qa = false;
        let mut found_ta = false;
        let mut found_rte = false;
        let mut found_rtg = false;
        let mut found_da = false;
        let mut found_ba = false;
        let mut found_fa = false;
        for (_, mut attr) in attr_tuples {
            let form_id: i32 = attr.get_form_id().unwrap();
            //Hopefully, there is some better way to do the below group of "if" expressions, such
            //as I was a attempting with "downcast_ref" and "is". See experiments 
            //in (and call to) get_type_info().
            //match attr {
                //if let Some(qa) = <attr as Any>::downcast_ref::<QuantityAttribute>() {
                //if let Some(qa) = (*attr).downcast_ref::<QuantityAttribute>() {
                //if let Some(qa) = (*attr).is::<QuantityAttribute>() {
                if db.get_attribute_form_name(form_id).unwrap() == Util::QUANTITY_TYPE {
                    let mut qa: QuantityAttribute = QuantityAttribute::new2(db.clone(), None, attr.get_id()).unwrap();
                    assert_eq!(qa.get_number(None).unwrap(), 50.0);
                    found_qa = true;
                }
                //else if let Some(ta) = attr.downcast_ref::<TextAttribute>() {
                else if db.get_attribute_form_name(form_id).unwrap() == Util::TEXT_TYPE {
                    let mut ta: TextAttribute = TextAttribute::new2(db.clone(), None, attr.get_id()).unwrap();
                    assert_eq!(ta.get_text(None).unwrap(), "some test text");
                    found_ta = true;
                }
                //else if let Some(rtle) = attr.downcast_ref::<RelationToLocalEntity>() {
                else if db.get_attribute_form_name(form_id).unwrap() == Util::RELATION_TO_LOCAL_ENTITY_TYPE {
                    let mut rtle: RelationToLocalEntity = RelationToLocalEntity::new3(db.clone(), None, attr.get_id()).unwrap().unwrap(); 
                    assert_eq!(rtle.get_attr_type_id(None).unwrap(), rel_type_id);
                    found_rte = true;
                }
                //Attribute::RelationToGroup(_) => {
                //else if let Some(a) = attr.downcast_ref::<RelationToGroup>() {
                else if db.get_attribute_form_name(form_id).unwrap() == Util::RELATION_TO_GROUP_TYPE {
                    found_rtg = true;
                }
                //else if let Some(a) = attr.downcast_ref::<DateAttribute>() {
                else if db.get_attribute_form_name(form_id).unwrap() == Util::DATE_TYPE {
                    found_da = true;
                }
                //else if let Some(a) = attr.downcast_ref::<BooleanAttribute>() {
                else if db.get_attribute_form_name(form_id).unwrap() == Util::BOOLEAN_TYPE {
                    found_ba = true;
                }
                //else if let Some(a) = attr.downcast_ref::<FileAttribute>() {
                else if db.get_attribute_form_name(form_id).unwrap() == Util::FILE_TYPE {
                    found_fa = true;
                }
                else { 
                    panic!("unexpected attribute type");
                }
                //_ => panic!("unexpected attribute type"),
            //}
            
            //let dbg_form_id: i32 = get_type_info(Box::new(attr.clone())).unwrap();
            //let parent_id = attr.get_parent_id(None).unwrap();
            //let sorting_index = attr.get_sorting_index(None).unwrap();
            //let attr_type_id = attr.get_attr_type_id(None).unwrap();
            //debug!("parent id is: {:?}, form id is: {:?}/{:?}, sorting_index is: {:?}, attr_type_id is: {:?}", parent_id, form_id_basic, form_id, sorting_index, attr_type_id);
            //debug!("form id is: {:?}/{:?}", form_id, dbg_form_id);
        }
        assert!(found_qa);
        assert!(found_ta);
        assert!(found_rte);
        assert!(found_rtg);
        assert!(found_da);
        assert!(found_ba);
        //%%file_attr: put back next line after file_attribute things are more implemented
        //assert!(found_fa);
    }

    fn get_type_info(attr: Box<dyn Any>) -> Result<i32, anyhow::Error> {
        //I *really* hope there is a better (or any successful??) way to do this.
        
        if (&*attr).is::<QuantityAttribute>() {
            debug!("is a quantityAttr");
        } else {
            debug!("is a: {:?}", attr.type_id());
            debug!("inner, is a: {:?}", (&*attr).type_id());
            debug!("{:?}", attr);
        }
        match attr.downcast_ref::<QuantityAttribute>() {
            Some(x) => debug!("x is a quantityAttr!: {:?}", x),
            None => debug!("attr is: {:?}", attr),
        }
        match attr.downcast_ref::<Rc<dyn Attribute>>() {
            Some(x) => {
                debug!("Attribute x is: {:?}", x);
                Ok(x.get_form_id().unwrap())
            },
            None => {
                debug!("Attribute is: {:?}", attr);
                return Err(anyhow!("Unable to determine form_id for attribute(?) {:?}", attr));
            },
        }
    }

    #[test]
    fn om_instance_read_data_from_db() {
        Util::initialize_tracing();
        let db: Rc<PostgreSQLDatabase> = Rc::new(Util::initialize_test_db().unwrap());
        let trans = None;

        let entity_id = db.create_entity(trans.clone(), "test: om_instance_read_data_from_db", None, None).unwrap();
        let uuid = uuid::Uuid::new_v4();
        let omi = OmInstance::create(db.clone(), trans.clone(), uuid.to_string().as_str(), "address", Some(entity_id)).unwrap();
        let mut omi_retrieved = OmInstance::new2(db, trans.clone(), omi.get_id().unwrap()).unwrap();
        let retrieved_id = omi_retrieved.get_entity_id(trans.clone()).unwrap();
        assert_eq!(retrieved_id, Some(entity_id));
    }

    // (and_get_relation_to_remote_entity_count_should_work.)
    #[test]
    fn entity_deletion_should_also_delete_relation_to_local_entity_attributes() {
        Util::initialize_tracing();
        let db: Rc<PostgreSQLDatabase> = Rc::new(Util::initialize_test_db().unwrap());
        let trans = None;

        let entity_id = db.create_entity(trans.clone(), "test: org.onemodel.PSQLDbTest.testRelsNRelTypes()", None, None).unwrap();
        let rel_type_id = db.create_relation_type(trans.clone(), "is sitting next to", "", RelationType::UNIDIRECTIONAL).unwrap();
        let starting_local_count = db.get_relation_to_local_entity_count(trans.clone(), entity_id, true).unwrap();
        let starting_remote_count = db.get_relation_to_remote_entity_count(trans.clone(), entity_id).unwrap();
        let related_entity_id = create_test_relation_to_local_entity_with_one_entity(&db, trans.clone(), entity_id, rel_type_id, None);
        assert_eq!(
            db.get_relation_to_local_entity_count(trans.clone(), entity_id, true).unwrap(),
            starting_local_count + 1
        );

        let oi_info = db.get_local_om_instance_data(trans.clone()).unwrap();
        let oi_id: String = oi_info.0;
        let remote_entity_id = 1234;
        db.create_relation_to_remote_entity(trans.clone(), rel_type_id, entity_id, remote_entity_id, None, 0, oi_id.as_str(), None).unwrap();
        assert_eq!(
            db.get_relation_to_local_entity_count(trans.clone(), entity_id, true).unwrap(),
            starting_local_count + 1
        );
        assert_eq!(
            db.get_relation_to_remote_entity_count(trans.clone(), entity_id).unwrap(),
            starting_remote_count + 1
        );
        assert!(!db
            .get_relation_to_remote_entity_data(trans.clone(), rel_type_id, entity_id, oi_id.clone(), remote_entity_id)
            .unwrap()
            .is_empty());

        db.delete_entity(trans.clone(), entity_id).unwrap();
        if db.get_relation_to_local_entity_count(trans.clone(), entity_id, true).unwrap() != 0 {
            panic!(
                "Deleting the model entity should also have deleted its RelationToLocalEntity objects. \
                 get_relation_to_local_entity_count(entity_idInNewTransaction) is {}",
                db.get_relation_to_local_entity_count(trans.clone(), entity_id, true).unwrap()
            );
        }
        let local_entity_result = 
            db.get_relation_to_local_entity_data(trans.clone(), rel_type_id, entity_id, related_entity_id);
        assert!(local_entity_result.is_err());
        assert!(local_entity_result
            .unwrap_err()
            .to_string()
            .contains("Got 0 instead of 1 result"));
        let remote_entity_result = 
            db.get_relation_to_remote_entity_data(trans.clone(), rel_type_id, entity_id, oi_id.clone(), remote_entity_id);
        assert!(remote_entity_result.is_err());
        assert!(remote_entity_result
            .unwrap_err()
            .to_string()
            .contains("Got 0 instead of 1 result"));
        db.delete_relation_type(trans.clone(), rel_type_id).unwrap();
    }

#[test]
fn attributes_handle_valid_on_dates_properly_in_and_out_of_db() {
    let db: Rc<PostgreSQLDatabase> = Rc::new(Util::initialize_test_db().unwrap());
    Util::initialize_tracing();
    let tx=None;

    let entity_id = db.create_entity(tx.clone(), "test: org.onemodel.PSQLDbTest.attributes...", None, None).unwrap();
    let rel_type_id = db.create_relation_type(tx.clone(), RELATION_TYPE_NAME, "", RelationType::UNIDIRECTIONAL).unwrap();

    // Create attributes & read back / other values (None already done above) as entered (confirms read back correctly)
    // (These methods do the checks, internally)
    create_test_relation_to_local_entity_with_one_entity(&db, tx.clone(), entity_id, rel_type_id, Some(0));
    create_test_relation_to_local_entity_with_one_entity(&db, tx.clone(), entity_id, rel_type_id, Some(Utc::now().timestamp_millis()));
    create_test_quantity_attribute_with_two_entities(&db, tx.clone(), entity_id, None);
    create_test_quantity_attribute_with_two_entities(&db, tx.clone(), entity_id, Some(0));
    create_test_text_attribute_with_one_entity(&db, tx.clone(), entity_id, None);
    create_test_text_attribute_with_one_entity(&db, tx.clone(), entity_id, Some(0));
}

    #[test]
    #[should_panic]
    fn test_add_quantity_attribute_with_bad_parent_id_does_not_work() {
        Util::initialize_tracing();
        let db: Rc<PostgreSQLDatabase> = Rc::new(Util::initialize_test_db().unwrap());
        //Using None instead of tx here for simplicity, but might have to change if
        //running tests in parallel.
        //let tx = db.begin_trans().unwrap();
        //let tx = Some(Rc::new(RefCell::new(tx)));
        println!("starting test_add_quantity_attribute_with_bad_parent_id");
        // Database should not allow adding quantity with a bad parent (Entity) ID!
        let bad_parent_id: i64 = db.find_id_which_is_not_key_of_any_entity(None).unwrap();
        // idea: make it a more specific failure type, so we catch only the error we want...?
        let _quantity_id = create_test_quantity_attribute_with_two_entities(&db.clone(), None, bad_parent_id, None);
    }

    fn create_test_quantity_attribute_with_two_entities<'a, 'b>(
        db: &'a Rc<PostgreSQLDatabase>,
        transaction: Option<Rc<RefCell<Transaction<'b, Postgres>>>>,
        in_parent_id: i64,
        in_valid_on_date: Option<i64>, /*= None*/
    ) -> i64 
    where
        'a: 'b
    {
        let unit_id: i64 = db
            .create_entity(transaction.clone(), "centimeters", None, None)
            .unwrap();
        let attr_type_id: i64 = db
            .create_entity(transaction.clone(), QUANTITY_TYPE_NAME, None, None)
            .unwrap();
        let default_date: i64 = Utc::now().timestamp_millis();
        let valid_on_date: Option<i64> = in_valid_on_date;
        let observation_date: i64 = default_date;
        let number: f64 = 50.0;

        //let ref rc_db = &db.clone();
        //let ref cloned = rc_db.clone();
        let quantity_id: i64 = db
            .create_quantity_attribute(
                transaction.clone(),
                in_parent_id,
                attr_type_id,
                unit_id,
                number,
                valid_on_date,
                observation_date,
                None,
            )
            .unwrap();
        // and verify it:
        let mut qa: QuantityAttribute =
            QuantityAttribute::new2(db.clone(), transaction.clone(), quantity_id).unwrap();
        assert!(qa.get_parent_id(transaction.clone()).unwrap() == in_parent_id);
        assert!(qa.get_unit_id(transaction.clone()).unwrap() == unit_id);
        assert!(qa.get_number(transaction.clone()).unwrap() == number);
        assert!(qa.get_attr_type_id(transaction.clone()).unwrap() == attr_type_id);
        match in_valid_on_date {
            None => assert!(qa.get_valid_on_date(transaction.clone()).unwrap().is_none()),
            Some(d) => {
                let in_date: i64 = d;
                let got_date: i64 = qa.get_valid_on_date(transaction.clone()).unwrap().unwrap();
                assert!(in_date == got_date);
            }
        }
        assert!(qa.get_observation_date(transaction.clone()).unwrap() == observation_date);
        quantity_id
    }

    /*%%later: reread the test, and consider: do I even want to be this detailed in Rust, with
     * errors when a rollback fails? Claude did try to convert it, in detail (below).
      "rollbackWithCatch" should "catch and return chained exception showing failed rollback" in {
        let db = new PostgreSQLDatabase("abc", "defg") {;
          override fn connect(inDbName: String, username: String, password: String) {
        // leave it null so calling it will fail as desired below.
        mConn = null
          }
          override fn create_and_check_expected_data() -> Unit { // Overriding because it is not needed for this test, and normally uses mConn, which by being set to null just above, breaks the method.
                                                                 // (intentional style violation for readability)
                                                                 //noinspection ScalaUselessExpression
        None
          }
          override fn model_tables_exist()  -> bool {
    true
    } //noinspection ScalaUselessExpression  (intentional style violation, for readability)
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

Here is claude's version of the above, and comments:

I've converted the "rollbackWithCatch" test from Scala to Rust. This was a more complex conversion because it involves:
1) Nested Exception Handling: Scala's nested try-catch blocks had to be converted to Rust's Result pattern with nested closures.
2) Mock Database Creation: In Scala, an anonymous subclass was used to override methods. In Rust, I created a separate MockPostgreSQLDatabase structure with the necessary methods.
3) Exception Chaining: I implemented the error chaining logic to mimic Scala's exception chaining behavior using Rust's anyhow error handling.

Key differences in the conversion:
1) Error Flow: Instead of throwing and catching exceptions, the Rust version uses Result types and the ? operator or explicit match statements to handle errors.
2) Stack Traces: Scala's explicit stack trace gathering was replaced with Rust's error formatting using {:?} in the format! macro.
3) Method Override Pattern: Rather than inline anonymous class definition with method overrides as in Scala, Rust uses a separate implementation block.

The test still verifies that:
- The original error message is preserved
- The chain of errors includes the message about rollback failure
- The stack trace contains references to the rollback operation

This conversion maintains the test's intent: verifying that when an error occurs and the rollback also fails, a properly chained error is returned that includes both the original error and the rollback error information.

    #[test]
    fn rollback_with_catch_catches_and_returns_chained_exception_showing_failed_rollback() {
        Util::initialize_tracing();
        
        //Using None instead of tx here for simplicity, but might have to change if
        //running tests in parallel.
        //let tx = db.begin_trans().unwrap();
        //let tx = Some(Rc::new(RefCell::new(tx)));
        
        // Create a mock database with overridden methods
        let db = MockPostgreSQLDatabase::new("abc", "defg");
        
        let mut found = false;
        let original_err_msg: String = "testing123".to_string();
        
        // In Rust, we'll simulate the nested try-catch with Result handling
        let result = (|| -> Result<(), anyhow::Error> {
            // Simulate the inner try-catch
            let inner_result = (|| -> Result<(), anyhow::Error> {
                // Throw the original exception
                Err(anyhow::anyhow!(original_err_msg.clone()))
            })();
            
            // Handle the inner error with rollback_with_catch
            match inner_result {
                Ok(_) => Ok(()),
                Err(e) => Err(db.rollback_with_catch(e)),
            }
        })();
        
        // Handle the outer error and verify the exception chain
        match result {
            Ok(_) => panic!("Expected error did not occur"),
            Err(t) => {
                found = true;
                let error_string = format!("{:?}", t);
                
                assert!(error_string.contains(&original_err_msg));
                assert!(error_string.contains("See the chained messages for ALL: the cause of rollback failure, AND"));
                assert!(error_string.contains("at org.onemodel.core.model.PostgreSQLDatabase.rollback_trans"));
            }
        }
        
        assert!(found);
    }

    // Mock implementation of PostgreSQLDatabase for testing
    struct MockPostgreSQLDatabase {
        conn: Option<Connection>,
    }

    impl MockPostgreSQLDatabase {
        fn new(db_name: &str, username: &str) -> Self {
            MockPostgreSQLDatabase {
                conn: None,
            }
        }
        
        fn connect(&mut self, db_name: &str, username: &str, password: &str) {
            // Leave conn as None so calling it will fail as desired
            self.conn = None;
        }
        
        fn create_and_check_expected_data(&self) -> () {
            // Overriding because it is not needed for this test, and normally uses conn,
            // which by being set to None just above, breaks the method.
            // (intentional style violation for readability)
            ()
        }
        
        fn model_tables_exist(&self) -> bool {
            true
        }
        
        fn do_database_upgrades_if_needed(&self) {
            ()
        }
        
        fn rollback_with_catch(&self, original_error: anyhow::Error) -> anyhow::Error {
            // Try to rollback, which will fail since conn is None
            match self.rollback_trans() {
                Ok(_) => original_error,
                Err(rollback_error) => {
                    // Chain the errors together
                    let chained_message = format!(
                        "See the chained messages for ALL: the cause of rollback failure, AND at org.onemodel.core.model.PostgreSQLDatabase.rollback_trans"
                    );
                    anyhow::anyhow!("{}\nOriginal error: {}", chained_message, original_error)
                }
            }
        }
        
        fn rollback_trans(&self) -> Result<(), anyhow::Error> {
            // This will fail since conn is None
            Err(anyhow::anyhow!("Connection is null"))
        }
    }
*/

    #[test]
    fn test_create_base_data_etc() {
        // and find_entity_only_ids_by_name and create_class_template_entity and 
        // find_contained_entries and find_relation_to_group_on_entity, that they all worked right 
        // in earlier db setup and now.
        Util::initialize_tracing();
        let db: Rc<PostgreSQLDatabase> = Rc::new(Util::initialize_test_db().unwrap());
        //Using None instead of tx here for simplicity, but might have to change if
        //running tests in parallel.
        //let tx = db.begin_trans().unwrap();
        //let tx = Some(Rc::new(RefCell::new(tx)));
        
        let person_template: String = format!("person{}", Util::TEMPLATE_NAME_SUFFIX);
        let system_entity_id = db.get_system_entity_id(None).unwrap();
        let group_id_of_class_templates = db.find_relation_to_and_group_on_entity(
            None,
            system_entity_id,
            Some(Util::CLASS_TEMPLATE_ENTITY_GROUP_NAME.to_string()),
        ).unwrap().2;
        // (Should be some value, but the activity on the test DB wouldn't have ids incremented to 0 yet,
        // so that one would be invalid. Could use the other method (find_id_which_is_not_key_of_any_entity()?)
        // to find an unused id, instead of 0.)
        assert!(group_id_of_class_templates.is_some() && group_id_of_class_templates.unwrap() != 0);
        let mut group = Group::new2(db.clone(), None, group_id_of_class_templates.unwrap()).unwrap();
        assert!(group.get_mixed_classes_allowed(None).unwrap());
        
        let person_template_entity_id: i64 = db.find_entity_only_ids_by_name(None, person_template.clone()).unwrap()[0];
        // idea: make this next part more idiomatic Rust? (but only if still very simple to read 
        // for programmers who are used to other languages?):
        let mut found = false;
        let entity_ids_in_group: Vec<i64> = db.get_group_entry_ids(None, group_id_of_class_templates.unwrap(), 0, None)
            .unwrap();
        for entity_id in &entity_ids_in_group {
            if *entity_id == person_template_entity_id {
                found = true;
            }
        }
        assert!(found); 
        // make sure the other approach also works, even with deeply nested data:
        let rel_type_id: i64 = db
            .create_relation_type(
                None,
                "contains",
                "",
                RelationType::UNIDIRECTIONAL,
            )
            .unwrap();
        let te1 = create_test_relation_to_local_entity_with_one_entity(
            &db.clone(),
            None,
            person_template_entity_id,
            rel_type_id,
            None,
        );
        
        let te2 = create_test_relation_to_local_entity_with_one_entity(
            &db.clone(),
            None,
            te1,
            rel_type_id,
            None,
        );
        
        let te3 = create_test_relation_to_local_entity_with_one_entity(
            &db.clone(),
            None,
            te2,
            rel_type_id,
            None,
        );
        
        let te4 = create_test_relation_to_local_entity_with_one_entity(
            &db.clone(),
            None,
            te3,
            rel_type_id,
            None,
        );
        let mut found_ids = std::collections::HashSet::new();
        db.find_contained_local_entity_ids(
            None,
            &mut found_ids,
            system_entity_id,
            &person_template.clone(),
            4,
            false, // stop_after_any_found
        ).unwrap();
        
        assert!(
            found_ids.contains(&person_template_entity_id),
            "Value not found in query: {}",
            person_template_entity_id
        );
        let mut all_contained_with_name = std::collections::HashSet::new();
        db.find_contained_local_entity_ids(
            None,
            &mut all_contained_with_name,
            system_entity_id,
            RELATED_ENTITY_NAME,
            4,
            false, // stop_after_any_found
        ).unwrap();
        
        // (see idea above about making more idiomatic Rust)
        let mut all_contained_ids = String::new();
        for id in &all_contained_with_name {
            all_contained_ids.push_str(&format!("{}, ", id));
        }
        assert_eq!(
            all_contained_with_name.len(),
            3,
            "Returned set had wrong count ({}): {}",
            all_contained_with_name.len(),
            all_contained_ids
        );
        
        let te4_entity = Entity::new2(db.clone(), None, te4).unwrap();
        te4_entity.add_text_attribute(
            None,
            te1, // not really but whatever
            RELATED_ENTITY_NAME,
            Some(0),
        ).unwrap();
        let mut all_contained_with_name2 = std::collections::HashSet::new();
        db.find_contained_local_entity_ids(
            None,
            &mut all_contained_with_name2,
            system_entity_id,
            RELATED_ENTITY_NAME,
            4,
            false, // stop_after_any_found
        ).unwrap();
        // should be no change yet (added it outside the # of levels to check):
        assert_eq!(
            all_contained_with_name2.len(),
            3,
            "Returned set had wrong count ({}): {}",
            all_contained_with_name.len(),
            all_contained_ids
        );

        let te2_entity = Entity::new2(db.clone(), None, te2).unwrap();
        te2_entity.add_text_attribute(
            None,
            te1, // not really but whatever
            RELATED_ENTITY_NAME,
            Some(0),
        ).unwrap();
        let mut all_contained_with_name3 = std::collections::HashSet::new();
        db.find_contained_local_entity_ids(
            None,
            &mut all_contained_with_name3,
            system_entity_id,
            RELATED_ENTITY_NAME,
            4,
            false, // stop_after_any_found
        ).unwrap();
        // should be no change yet (the entity was already in the return set, so the TA addition didn't add anything)
        assert_eq!(
            all_contained_with_name3.len(),
            3,
            "Returned set had wrong count ({}): {}",
            all_contained_with_name.len(),
            all_contained_ids
        );

        te2_entity.add_text_attribute(
            None,
            te1, // not really but whatever
            "otherText",
            Some(0),
        ).unwrap();
        let mut all_contained_with_name4 = std::collections::HashSet::new();
        db.find_contained_local_entity_ids(
            None,
            &mut all_contained_with_name4,
            system_entity_id,
            "otherText",
            4,
            false, // stop_after_any_found
        ).unwrap();
        // now there should be a change:
        assert_eq!(
            all_contained_with_name4.len(),
            1,
            "Returned set had wrong count ({}): {}",
            all_contained_with_name.len(),
            all_contained_ids
        );
        
        // make sure this was also set up probably by the db set code, as with some other things
        // above.
        let editor_cmd = db.get_text_editor_command(None).unwrap();
        if Util::is_windows() {
            assert!(editor_cmd.contains("notepad"));
        } else {
            assert_eq!(editor_cmd, "vi");
        }
    }

    #[test]
    fn test_is_duplicate_entity() {
        Util::initialize_tracing();
        let db: Rc<PostgreSQLDatabase> = Rc::new(Util::initialize_test_db().unwrap());
        //Using None instead of tx here for simplicity, but might have to change if
        //running tests in parallel.
        //let tx = db.begin_trans().unwrap();
        //let tx = Some(Rc::new(RefCell::new(tx)));
        
        let name: String = "testing is_duplicateEntity".to_string();
        let entity_id: i64 = db
            .create_entity(None, &name, None, None,)
            .unwrap();
        assert!(db.is_duplicate_entity_name(None, &name, None).unwrap());
        assert!(!db.is_duplicate_entity_name(None, &name, Some(entity_id)).unwrap());
        
        let entity_with_space_in_name_id: i64 = db
            .create_entity(None, &format!("{} ", name), None, None,)
            .unwrap();
        assert!(!db.is_duplicate_entity_name(
            None,
            &format!("{} ", name),
            Some(entity_with_space_in_name_id)
        ).unwrap());

        let entity_id_with_lowercase_name: i64 = db
            .create_entity(None, &name.to_lowercase(), None, None,)
            .unwrap();
        assert!(db.is_duplicate_entity_name(
            None,
            &name,
            Some(entity_id_with_lowercase_name)
        ).unwrap());
        
        db.update_entity_only_name(None, entity_id, &name.to_lowercase()).unwrap();
        assert!(db.is_duplicate_entity_name(
            None,
            &name,
            Some(entity_id_with_lowercase_name)
        ).unwrap());
        assert!(db.is_duplicate_entity_name(
            None,
            &name,
            Some(entity_id)
        ).unwrap());
        
        db.delete_entity(None, entity_id_with_lowercase_name).unwrap();
        assert!(!db.is_duplicate_entity_name(
            None,
            &name,
            Some(entity_id)
        ).unwrap());

        // Intentionally put some uppercase letters for later comparison w/ lowercase.
        let rel_type_name = format!("{}-RelationType", name);
        let rel_type_id: i64 = db.create_relation_type(
                None,
                "testingOnly",
                &rel_type_name,
                RelationType::UNIDIRECTIONAL,
            )
            .unwrap();
        assert!(db.is_duplicate_entity_name(None, &rel_type_name, None).unwrap());
        assert!(!db.is_duplicate_entity_name(None, &rel_type_name, Some(rel_type_id)).unwrap());
        
        //let tx = db.begin_trans().unwrap();
        //let tx = Some(Rc::new(RefCell::new(tx)));
        db.update_entity_only_name(
            None, //tx.clone(),
            entity_id,
            &rel_type_name.to_lowercase()
        ).unwrap();
        assert!(db.is_duplicate_entity_name(
            None, //tx.clone(),
            &rel_type_name,
            Some(entity_id)
        ).unwrap());
        
        assert!(db.is_duplicate_entity_name(
            None, //tx.clone(),
            &rel_type_name,
            Some(rel_type_id)
        ).unwrap()); 
        // Maybe the rollback (and "tx.clone()" in 3 places just above) was because setting an 
        // entity name to rel_type_name (above) doesn't really make sense, 
        // but was just for that part of the test. But maybe it doesnt matter, since it is just a test.
        //db.rollback_trans(tx).unwrap();
    }

    #[test]
    fn is_duplicate_entity_class_and_class_update_deletion_works() {
        Util::initialize_tracing();
        let db: Rc<PostgreSQLDatabase> = Rc::new(Util::initialize_test_db().unwrap());
        //Using None instead of tx here for simplicity, but might have to change if
        //running tests in parallel.
        //let tx = db.begin_trans().unwrap();
        //let tx = Some(Rc::new(RefCell::new(tx)));
        
        let name: String = "testing is_duplicateEntityClass".to_string();
        let (class_id, entity_id) = db
            .create_class_and_its_template_entity(None, &name)
            .unwrap();
        assert!(EntityClass::is_duplicate(db.clone(), None, &name, None).unwrap());
        assert!(!EntityClass::is_duplicate(db.clone(), None, &name, Some(class_id)).unwrap());
        
        db.update_class_name(None, class_id, name.to_lowercase()).unwrap();
        assert!(!EntityClass::is_duplicate(db.clone(), None, &name, Some(class_id)).unwrap());
        assert!(EntityClass::is_duplicate(db.clone(), None, &name.to_lowercase(), None).unwrap());
        assert!(!EntityClass::is_duplicate(db.clone(), None, &name.to_lowercase(), Some(class_id)).unwrap());
        
        db.update_class_name(None, class_id, name.clone()).unwrap();
        db.update_class_create_default_attributes(None, class_id, Some(false)).unwrap();
        let mut entity_class = EntityClass::new2(db.clone(), None, class_id).unwrap();
        let should1: Option<bool> = entity_class.get_create_default_attributes(None).unwrap();
        assert!(!should1.unwrap());
        
        db.update_class_create_default_attributes(None, class_id, None).unwrap();
        let mut entity_class = EntityClass::new2(db.clone(), None, class_id).unwrap();
        let should2: Option<bool> = entity_class.get_create_default_attributes(None).unwrap();
        assert!(should2.is_none());
        
        db.update_class_create_default_attributes(None, class_id, Some(true)).unwrap();
        let mut entity_class = EntityClass::new2(db.clone(), None, class_id).unwrap();
        let should3: Option<bool> = entity_class.get_create_default_attributes(None).unwrap();
        assert!(should3.unwrap());
        
        db.update_entitys_class(None, entity_id, None).unwrap();
        db.delete_class_and_its_template_entity(class_id).unwrap();
        assert!(!EntityClass::is_duplicate(db.clone(), None, &name, Some(class_id)).unwrap());
        assert!(!EntityClass::is_duplicate(db.clone(), None, &name, None).unwrap());
    }
/*%%%%
      "EntitiesInAGroup and getclasses/classcount methods" should "work, and should enforce class_id uniformity within a group of entities" in {
        // ...for now anyway. See comments at this table in psqld.create_tables and/or hasMixedClasses.

        // This also tests db.create_entity and db.updateEntityOnlyClass.
        let entityName = "test: PSQLDbTest.testgroup-class-uniqueness" + "--theEntity";
        let (classId, entity_id) = db.createClassAndItsTemplateEntity(entityName, entityName);
        let (classId2, entity_id2) = db.createClassAndItsTemplateEntity(entityName + 2, entityName + 2);
        let classCount = db.get_class_count();
        let classes = db.get_classes(0);
        assert(classCount == classes.size)
        let classCountLimited = db.get_class_count(Some(entity_id2));
        assert(classCountLimited == 1) //whatever, just need some relation type to go with:
        let rel_type_id: i64 = db.create_relation_type("contains", "", RelationType.UNIDIRECTIONAL);
        let groupId = DatabaseTestUtils.create_and_add_test_relation_to_group_on_to_entity(db, entity_id, rel_type_id, "test: PSQLDbTest.testgroup-class-uniqueness",;
                                                                                 Some(12345L), allow_mixed_classes_in = false)._1
        let group: Group = new Group(db, groupId);
        assert(! db.is_entity_in_group(groupId, entity_id))
        assert(! db.is_entity_in_group(groupId, entity_id))
        group.add_entity(entity_id)
        assert(db.is_entity_in_group(groupId, entity_id))
        assert(! db.is_entity_in_group(groupId, entity_id2)) //should fail due to mismatched classId (a long):
        assert(intercept[Exception] {
                                      group.add_entity(entity_id2)
                                    }.getMessage.contains(Database.MIXED_CLASSES_EXCEPTION))
        // should succeed (same class now):
        db.update_entitys_class(entity_id2, Some(classId))
        group.add_entity(entity_id2) // ...and for convenience while here, make sure we can't make mixed classes with changing the *entity* either:
        assert(intercept[Exception] {
                                      db.update_entitys_class(entity_id2, Some(classId2))
                                    }.getMessage.contains(Database.MIXED_CLASSES_EXCEPTION))
        assert(intercept[Exception] {
                                      db.update_entitys_class(entity_id2, None)
                                    }.getMessage.contains(Database.MIXED_CLASSES_EXCEPTION))
        //should fail due to mismatched classId (NULL):
        let entity_id3 = db.create_entity(entityName + 3);
        assert(intercept[Exception] {
                                      group.add_entity(entity_id3)
                                    }.getMessage.contains(Database.MIXED_CLASSES_EXCEPTION))

        assert(!db.areMixedClassesAllowed(groupId))


        let system_entity_id = db.getSystemEntityId; // idea: (noted at other use of this method)
        let classGroupId = db.find_relation_to_and_group_OnEntity(system_entity_id, Some(Database.CLASS_TEMPLATE_ENTITY_GROUP_NAME))._3;
        assert(db.areMixedClassesAllowed(classGroupId.get))

        let groupSizeBeforeRemoval = db.get_group_size(groupId);

        assert(db.get_group_size(groupId, 2) == 0)
        assert(db.get_group_size(groupId, 1) == groupSizeBeforeRemoval)
        assert(db.get_group_size(groupId) == groupSizeBeforeRemoval)
        db.archive_entity(entity_id2)
        assert(db.get_group_size(groupId, 2) == 1)
        assert(db.get_group_size(groupId, 1) == groupSizeBeforeRemoval - 1)
        assert(db.get_group_size(groupId) == groupSizeBeforeRemoval)

        db.remove_entity_from_group(groupId, entity_id2)
        let groupSizeAfterRemoval = db.get_group_size(groupId);
        assert(groupSizeAfterRemoval < groupSizeBeforeRemoval)

        assert(db.get_group_size(groupId, 2) == 0)
        assert(db.get_group_size(groupId, 1) == groupSizeBeforeRemoval - 1)
        assert(db.get_group_size(groupId) == groupSizeBeforeRemoval - 1)
      }

      "get_entities_only and ...Count" should "allow limiting results by classId and/or group containment" in {
        // idea: this could be rewritten to not depend on pre-existing data to fail when it's supposed to fail.
        let starting_entity_count = db.get_entities_only_count();
        let someClassId: i64 = db.db_query_wrapper_for_one_row("select id from class limit 1", "i64")(0).get.asInstanceOf[i64];
        let numEntitiesInClass = db.extract_row_count_from_count_query("select count(1) from entity where class_id=" + someClassId);
        assert(starting_entity_count > numEntitiesInClass)
        let allEntitiesInClass = db.get_entities_only(0, None, Some(someClassId), limit_by_class = true);
        let allEntitiesInClassCount1 = db.get_entities_only_count(limit_by_class = true, Some(someClassId));
        let allEntitiesInClassCount2 = db.get_entities_only_count(limit_by_class = true, Some(someClassId), None);
        assert(allEntitiesInClassCount1 == allEntitiesInClassCount2)
        let templateClassId: i64 = new EntityClass(db, someClassId).get_template_entity_id;
        let allEntitiesInClassCountWoClass = db.get_entities_only_count(limit_by_class = true, Some(someClassId), Some(templateClassId));
        assert(allEntitiesInClassCountWoClass == allEntitiesInClassCount1 - 1)
        assert(allEntitiesInClass.size == allEntitiesInClassCount1)
        assert(allEntitiesInClass.size < db.get_entities_only(0, None, Some(someClassId), limit_by_class = false).size)
        assert(allEntitiesInClass.size == numEntitiesInClass)
        let e: Entity = allEntitiesInClass.get(0);
        assert(e.get_class_id.get == someClassId) // part 2:
                                                  // some setup, confirm good
        let starting_entity_count2 = db.get_entities_only_count();
        let rel_type_id: i64 = db.create_relation_type("contains", "", RelationType.UNIDIRECTIONAL);
        let id1: i64 = db.create_entity("name1");
        let (group, rtg) = new Entity(db, id1).add_group_and_relation_to_group(rel_type_id, "someRelToGroupName", allow_mixed_classes_inGroupIn = false, None, 1234L,;
                                                                           None)
        assert(db.relation_to_group_keys_exist(rtg.get_parent_id(), rtg.get_attr_type_id(), rtg.get_group_id))
        assert(db.attribute_key_exists(rtg.get_form_id, rtg.get_id))
        let id2: i64 = db.create_entity("name2");
        group.add_entity(id2)
        let entity_countAfterCreating = db.get_entities_only_count();
        assert(entity_countAfterCreating == starting_entity_count2 + 2)
        let resultSize = db.get_entities_only(0).size();
        assert(entity_countAfterCreating == resultSize)
        let resultSizeWithNoneParameter = db.get_entities_only(0, None, group_to_omit_id_in = None).size();
        assert(entity_countAfterCreating == resultSizeWithNoneParameter) // the real part 2 test
        let resultSizeWithGroupOmission = db.get_entities_only(0, None, group_to_omit_id_in = Some(group.get_id)).size();
        assert(entity_countAfterCreating - 1 == resultSizeWithGroupOmission)
      }

      "EntitiesInAGroup table (or methods? ick)" should "allow all a group's entities to have no class" in {
        // ...for now anyway.  See comments at this table in psqld.create_tables and/or hasMixedClasses.
        let entityName = "test: PSQLDbTest.testgroup-class-allowsAllNulls" + "--theEntity";
        let (classId, entity_id) = db.createClassAndItsTemplateEntity(entityName, entityName);
        let rel_type_id: i64 = db.create_relation_type("contains", "", RelationType.UNIDIRECTIONAL);
        let groupId = DatabaseTestUtils.create_and_add_test_relation_to_group_on_to_entity(db, entity_id, rel_type_id, "test: PSQLDbTest.testgroup-class-allowsAllNulls",;
                                                                                 Some(12345L), allow_mixed_classes_in = false)._1
        let group: Group = new Group(db, groupId); // 1st one has a NULL class_id
        let entity_id3 = db.create_entity(entityName + 3);
        group.add_entity(entity_id3) // ...so it works to add another one that's NULL
        let entity_id4 = db.create_entity(entityName + 4);
        group.add_entity(entity_id4) // but adding one with a class_id fails w/ mismatch:
        let entity_id5 = db.create_entity(entityName + 5, Some(classId));
        assert(intercept[Exception] {
                                      group.add_entity(entity_id5)
                                    }.getMessage.contains(Database.MIXED_CLASSES_EXCEPTION))
      }

      "get_entities_only_count" should "not count entities used as relation types or attribute types" in {
        let entity_id = db.create_entity("test: org.onemodel.PSQLDbTest.get_entities_only_count");
        let c1 = db.get_entities_only_count();
        assert(db.get_entities_only_count() == c1)
        let rel_type_id: i64 = db.create_relation_type("contains", "", RelationType.UNIDIRECTIONAL);
        assert(db.get_entities_only_count() == c1)
        create_test_relation_to_local_entity_with_one_entity(entity_id, rel_type_id)
        let c2 = c1 + 1;
        assert(db.get_entities_only_count() == c2) // this kind shouldn't matter--confirming:
        let rel_type_id2: i64 = db.create_relation_type("contains2", "", RelationType.UNIDIRECTIONAL);
        DatabaseTestUtils.create_and_add_test_relation_to_group_on_to_entity(db, entity_id, rel_type_id2)
        assert(db.get_entities_only_count() == c2)

        let prevEntitiesUsedAsAttributeTypes = db.get_count_of_entities_used_as_attribute_types(Util::DATE_TYPE, quantity_seeks_unit_not_type_in = false);
        let date_attribute_id = create_test_date_attribute_with_one_entity(entity_id);
        let dateAttribute = new DateAttribute(db, date_attribute_id);
        assert(db.get_count_of_entities_used_as_attribute_types(Util::DATE_TYPE, quantity_seeks_unit_not_type_in = false) == prevEntitiesUsedAsAttributeTypes + 1)
        assert(db.get_entities_only_count() == c2)
        let dateAttributeTypeEntities: Array[Entity] = db.get_entities_used_as_attribute_types(Util::DATE_TYPE, 0, quantity_seeks_unit_not_type_in = false);
                                                       .toArray(new Array[Entity](0 ))
        let mut found = false;
        for (dateAttributeType: Entity <- dateAttributeTypeEntities.toArray) {
          if dateAttributeType.get_id == dateAttribute.get_attr_type_id()) {
            found = true
          }
        }
        assert(found)

        create_test_boolean_attribute_with_one_entity(entity_id, val_in = false, None, 0)
        assert(db.get_entities_only_count() == c2)

        create_test_file_attribute_and_one_entity(new Entity(db, entity_id), "desc", 2, verify_in = false)
        assert(db.get_entities_only_count() == c2)

      }

      "get_matching_entities & Groups" should "work" in {
        let entity_id1 = db.create_entity("test: org.onemodel.PSQLDbTest.get_matching_entities1--abc");
        let entity1 = new Entity(db, entity_id1);
        let entity_id2 = db.create_entity("test: org.onemodel.PSQLDbTest.get_matching_entities2");
        db.create_text_attribute(entity_id1, entity_id2, "defg", None, 0)
        let entities1 = db.get_matching_entities(0, None, None, "abc");
        assert(entities1.size == 1)
        db.create_text_attribute(entity_id2, entity_id1, "abc", None, 0)
        let entities2 = db.get_matching_entities(0, None, None, "abc");
        assert(entities2.size == 2)

        let rel_type_id: i64 = db.create_relation_type("contains", "", RelationType.UNIDIRECTIONAL);
        let group_name = "someRelToGroupName";
        entity1.add_group_and_relation_to_group(rel_type_id, group_name, allow_mixed_classes_inGroupIn = false, None, 1234L,
                                           None)
        assert(db.get_matching_groups(0, None, None, "some-xyz-not a grp name").size == 0)
        assert(db.get_matching_groups(0, None, None, group_name).size > 0)
      } //idea: should this be moved to ImportExportTest? why did i put it here originally?
        "getJournal" should "show activity during a date range" in {
        let startDataSetupTime = System.currentTimeMillis();
        let entity_id: i64 = db.create_entity("test object");
        let entity: Entity = new Entity(db, entity_id);
        let importExport = new ImportExport(null, new Controller(null, false, Some(Database.TEST_USER), Some(Database.TEST_PASS)));
        let importFile: File = importExport.tryImporting_FOR_TESTS("testImportFile0.txt", entity);
        let ids: java.util.ArrayList[i64] = db.find_all_entity_ids_by_name("vsgeer-testing-getJournal-in-db");
        let (fileContents: String, outputFile: File) = importExport.tryExportingTxt_FOR_TESTS(ids, db);
        // (next 3 lines are redundant w/ a similar test in ImportExportTest, but are here to make sure the data
        // is as expected before proceeding with the actual purpose of this test:)
        assert(fileContents.contains("vsgeer"), "unexpected file contents:  " + fileContents)
        assert(fileContents.contains("record/report/review"), "unexpected file contents:  " + fileContents)
        assert(outputFile.length == importFile.length)

        db.archive_entity(entity_id)
        let endDataSetupTime = System.currentTimeMillis();

        let results: util.ArrayList[(i64, String, i64)] = db.find_journal_entries(startDataSetupTime, endDataSetupTime);
        assert(results.size > 0)
      }

      "get_textAttributeByNameForEntity" should "fail when no rows found" in {
        intercept[org.onemodel.core.OmDatabaseException] {
                                         let system_entity_id = db.getSystemEntityId;
                                         db.get_text_attribute_by_type_id(system_entity_id, 1L, Some(1))
                                       }
      }
     %%%%*/

    #[test]
    fn get_relations_to_group_containing_this_group_and_get_containing_relations_to_group() {
        Util::initialize_tracing();
        let db: Rc<PostgreSQLDatabase> = Rc::new(Util::initialize_test_db().unwrap());
        let entity_id: i64 = db
            .create_entity(
                None, 
                "test: get_relations_to_group_containing_this_group...",
                None,
                None,
            )
            .unwrap();
        let entity_id2: i64 = db
            .create_entity(
                None,
                "test: get_relations_to_group_containing_this_group2...",
                None,
                None,
            )
            .unwrap();
        let rel_type_id: i64 = db
            .create_relation_type(
                None,
                "contains in get_relations_to_group_containing_this_group", 
                "", 
                RelationType::UNIDIRECTIONAL,
            )
            .unwrap();
        let entity = Entity::new2(db.clone(), None, entity_id).unwrap();
        let (group_id, rtg_id) = create_and_add_test_relation_to_group_on_to_entity(
            db.clone(), 
            None,
            &entity,
            rel_type_id,
            "some group name in get_relations_to_group_containing_this_group",
            Some(1),
            true,
        ).unwrap();
        let group = Group::new2(db.clone(), None, group_id).unwrap();
        group.add_entity(None, entity_id2, None).unwrap();
        
        let rtg_data = db.get_relations_to_group_containing_this_group(None, group_id, 0, None).unwrap();
        assert_eq!(rtg_data.len(), 1);
        assert_eq!(rtg_data[0].0, rtg_id);
        assert_eq!(rtg_data[0].4, Some(1));
        
        let same_rtgs = db.get_containing_relations_to_group(None, entity_id2, 0, None).unwrap();
        assert_eq!(same_rtgs.len(), 1);
        assert_eq!(same_rtgs[0].0, rtg_id);
        // no need to db.rollback_trans(), because that is automatic when tx goes out of scope, per sqlx docs.
    }

}
