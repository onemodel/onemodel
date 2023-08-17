/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003, 2004, 2010, 2011, 2013-2020 inclusive, and 2023-2023 inclusive, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
use crate::model::boolean_attribute::BooleanAttribute;
use crate::model::database::DataType;
use crate::model::database::Database;
use crate::model::entity::Entity;
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

/// An important thing to know about this code is that sqlx transactions automatically roll back
/// if they go out of scope before commit() is called.
#[derive(Debug)]
pub struct PostgreSQLDatabase {
    pub rt: tokio::runtime::Runtime,
    pub pool: PgPool,
    // When true, this means to override the usual settings and show the archived entities too (like a global temporary "un-archive"):
    pub include_archived_entities: bool,
}

impl PostgreSQLDatabase {
    const SCHEMA_VERSION: i32 = 7;
    const ENTITY_ONLY_SELECT_PART: &'static str = "SELECT e.id";

    fn db_name(db_name_without_prefix: &str) -> String {
        format!("{}{}", Util::DB_NAME_PREFIX, db_name_without_prefix)
    }

    //%%should this and other eventual callers of db_query take its advice and call the
    //ck method?
    //%%$%%I think this will have to return something with an option to deal with valid_on_date or observed_date things....  See callers..?
    fn db_query_wrapper_for_one_row(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        sql: &str,
        types: &str,
    ) -> Result<Vec<Option<DataType>>, anyhow::Error> {
        let results: Vec<Vec<Option<DataType>>> = self.db_query(transaction, sql, types)?;
        if results.len() != 1 {
            Err(anyhow!(format!(
                "Got {} instead of 1 result from sql \"{}\" ??",
                results.len(),
                sql
            )))
        } else {
            let oldrow = &results[0];
            let mut newrow = Vec::new();
            for x in oldrow {
                let z = match x {
                    //idea: surely there is some better way than what I am doing here? See other places similarly.  Maybe implement DataType.clone() ?
                    Some(DataType::Bigint(y)) => Some(DataType::Bigint(y.clone())),
                    // Some(DataType::UnsignedInt(y)) => DataType::UnsignedInt(y.clone()),
                    Some(DataType::Boolean(y)) => Some(DataType::Boolean(y.clone())),
                    Some(DataType::String(y)) => Some(DataType::String(y.clone())),
                    Some(DataType::Float(y)) => Some(DataType::Float(y.clone())),
                    Some(DataType::Smallint(y)) => Some(DataType::Smallint(y.clone())),
                    Some(DataType::Smallint(y)) => Some(DataType::Smallint(y.clone())),
                    None => None,
                    _ => {
                        return Err(anyhow!(format!(
                            "How did we get here for x of {:?} in {:?}?",
                            x, results[0]
                        )))
                    }
                };
                newrow.push(z);
            }
            Ok(newrow)
        }
    }

    /// Before calling this, the caller should have made sure that any parameters it received in the form of
    /// Strings should have been passed through escape_quotes_etc FIRST, and ONLY THE RESULT SENT HERE.
    /// Returns the results (a collection of rows, each row being its own collection).
    //%%should do that escape_quotes_etc here instead, so guaranteed? or comment why not?
    //%%$%%should the things in "types" parm be an enum or something like that? Or doc it here?
    fn db_query(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        sql: &str,
        types: &str,
    ) -> Result<Vec<Vec<Option<DataType>>>, anyhow::Error> {
        // Note: pgsql docs say "Under the JDBC specification, you should access a field only
        // once" (under the JDBC interface part).  Not sure if that applies now to sqlx in rust.
        debug!("In db_query, sql is: {}\n...and types: {:?} .", sql, types);

        Self::check_for_bad_sql(sql)?;
        let mut results: Vec<Vec<Option<DataType>>> = Vec::new();
        let types_vec: Vec<&str> = types.split_terminator(",").collect();
        let mut row_counter = 0;
        let query = sqlx::query(sql);
        let map = query
            .map(|sqlx_row: PgRow| {
                // (next line is 1-based -- intended to compare to the size of results, later)
                row_counter += 1;
                //was: let row: Vec<Option<DataType>> = new Vec<Option<DataType>>(types_vec.length);
                let mut row: Vec<Option<DataType>> = Vec::with_capacity(types_vec.len());
                let mut column_counter: usize = 0;
                for type_name in &types_vec {
                    // the for loop is to take us through all the columns in this row, as specified by the caller in the "types" parm.
                    //was: if rs.getObject(column_counter) == null) row(column_counter - 1) = None
                    //%%name?:
                    //%%how will these handle nulls? (like validOnDate or Entity.m_class_id??) how can they? See idea below under i64 & test, use that
                    //  elsewhere?
                    //%%what should error handling in this context be like? how should it work? see the open tabs re errs in closures,
                    // AND/OR the sqlx doc for this closure for what err it returns, or what caller expects...???
                    let col: &PgColumn = sqlx_row.try_column(column_counter).unwrap();
                    let value_ref = sqlx_row.try_get_raw(column_counter).unwrap();
                    let ti: &PgTypeInfo = col.type_info();
                    // %%let is_null: bool = ti.is_null();
                    let is_null: bool = value_ref.is_null();
                    //%%next 2 lines just for development:
                    let db_type_info = ti.to_string();
                    debug!("In fn db_query, is_null: {}, type_name={}, col={:?}, and db_type_info: {}", is_null, type_name, col, db_type_info);
                    if is_null {
                        row.push(None);
                    } else {
                        // When modifying: COMPARE TO AND SYNCHRONIZE WITH THE TYPES IN the for loop in RestDatabase.processArrayOptionAny .
                        if type_name == &"Float" {
                            //was: row(column_counter) = Some(rs.getFloat(column_counter))
                            let decode_mbe: Result<_, sqlx::Error> = sqlx_row.try_get(column_counter);
                            let x: f64 = decode_mbe.unwrap(); //%%???
                            debug!("in db_query1: x is {} .", x);
                            let y = DataType::Float(x);
                            row.push(Some(y));
                        } else if type_name == &"String" {
                            //%%$%
                            //     was: row(column_counter) = Some(PostgreSQLDatabase.unescape_quotes_etc(rs.getString(column_counter)))
                            let decode_mbe: Result<_, sqlx::Error> = sqlx_row.try_get(column_counter);
                            let x: String = decode_mbe.unwrap(); //%%???
                            debug!("in db_query3: x is {} .", x);
                            let y = DataType::String(Self::unescape_quotes_etc(x));
                            debug!("in db_query3: y is {:?} .", y);
                            row.push(Some(y));
                        } else if type_name == &"i64" {
                            //was: row(column_counter) = Some(rs.getLong(column_counter))
                            let decode_mbe = sqlx_row.try_get(column_counter);
                            // let decode_mbe = sqlx_row.try_column(column_counter);
                            let x: i64 = decode_mbe.unwrap(); //%%??? for all such.
                            // let x: i64 = match decode_mbe { //%%???
                            //     None => %
                            // };
                            debug!("in db_query4: x is {} .", x);
                            row.push(Some(DataType::Bigint(x)));
                        //u64 here unsupported by sqlx:
                        // } else if type_name == &"u64" {
                        //     let decode_mbe = sqlx_row.try_get(column_counter);
                        //     let x: u64 = decode_mbe.unwrap();
                        //     debug!("in db_query4a: x is {} .", x);
                        //     row.push(Some(DataType::UnsignedInt(x)));
                        } else if type_name == &"bool" {
                            //was: row(column_counter) = Some(rs.get_boolean(column_counter))
                            let decode_mbe: Result<_, sqlx::Error> = sqlx_row.try_get(column_counter);
                            let x: bool = decode_mbe.unwrap(); //%%???
                            debug!("in db_query5: x is {} .", x);
                            let y = DataType::Boolean(x);
                            row.push(Some(y));
                        } else if type_name == &"Int" {
                            //     row(column_counter) = Some(rs.getInt(column_counter))
                            let decode_mbe: Result<_, sqlx::Error> = sqlx_row.try_get(column_counter);
                            let x: i32 = decode_mbe.unwrap(); //%%???
                            debug!("in db_query6: x is {} .", x);
                            let y = DataType::Smallint(x);
                            row.push(Some(y));
                        } else {
                        //     %% make sure to address this? and that I know what happens when this line is hit: test it (mbe change some code so alw hit, just2see? or in a test?)
                        //     return Err(anyhow!("In db_query, Unexpected DataType value: '{}' at column: {}, with db_type_info={:?}.",
                            //     type_name, column_counter, db_type_info));
                            panic!("Unexpected DataType value: '{}' at column: {}, with db_type_info={:?}.",
                                type_name, column_counter, db_type_info);
                        }
                    }
                    column_counter += 1;
                }
                if row.len() != types_vec.len() {
                    //%% make sure to address this? and that I know what happens when this line is hit: test it (mbe change some code so alw hit, just2see? or in a test?)
                    // return Err(anyhow!("In db_query, Row length {} does not equal expected types list length {}.", row.len(), types_vec.len()));
                    panic!("Row length {} does not equal expected types list length {}.", row.len(), types_vec.len());
                }
                 // }
                results.push(row);
            });
        //PROBABLY IGNORE/DEL THIS
        // let future = map.fetch_all(&self.pool);
        // if let Some(tx) = transaction {
        // if transaction.is_some() {
        //     let mut  trans = *(transaction.unwrap());
        //     let future = map.fetch_all(&mut trans);
        //     self.rt.block_on(future).unwrap();
        // };

        if let Some(_tx) = transaction {
            // if transaction.is_some() {
            //     let mut trans = transaction.unwrap();
            //%%WHEN FIXING, PUT NEXT LINE BACK AND REMOVE THE ONE AFTER:
            //     let future = map.fetch_all(tx);
            let future = map.fetch_all(&self.pool);
            // let future = map.fetch_all(trans);
            self.rt.block_on(future).unwrap();
        } else {
            let future = map.fetch_all(&self.pool);
            self.rt.block_on(future).unwrap();
        }

        //%%JUST ANOTHER EXPERIMENT, probably can delete after other things working.
        // let future = match transaction {
        //     Some(tx) => map.fetch_all(tx),
        //     None => map.fetch_all(&(&self.pool)),
        // };
        /*let rows =*/
        // self.rt.block_on(future).unwrap();

        // idea: (see comment at other use in this class, of getWarnings)
        // idea: maybe both uses of getWarnings should be combined into a method.
        //%%how do this in rust/sqlx?:
        // let warnings = rs.getWarnings;
        // let warnings2 = st.getWarnings;
        // if warnings != null || warnings2 != null) throw new OmDatabaseException("Warnings from postgresql. Matters? Says: " + warnings + ", and " + warnings2)

        if row_counter != results.len() {
            return Err(anyhow!(
                "In db_query: Unexpected values in rowcounter ({}) and results.len ({})",
                row_counter,
                results.len()
            ));
        }
        Ok(results)
    }

    /// Convenience function. Error message it gives if > 1 found assumes that sql passed in will return only 1 row!
    /// Expects sql to be "select count(1)" from..."!  IDEA: make this fn provide all up to "where",
    /// to make it more ergonomic, and code less likely to be able to call it wrong?
    fn does_this_exist(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        sql_in: &str,
        fail_if_more_than_one_found: bool, /*%% = true*/
    ) -> Result<bool, anyhow::Error> {
        let row_count: u64 = self.extract_row_count_from_count_query(transaction, sql_in)?;
        if fail_if_more_than_one_found {
            if row_count == 1 {
                Ok(true)
            } else if row_count > 1 {
                Err(anyhow!(format!(
                    "Should there be > 1 entries for sql: {}?? ({} were found.)",
                    sql_in, row_count
                )))
            } else {
                assert!(row_count < 1);
                Ok(false)
            }
        } else {
            Ok(row_count >= 1)
        }
    }

    fn extract_row_count_from_count_query(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        sql_in: &str,
    ) -> Result<u64, anyhow::Error> {
        let results: Vec<Option<DataType>> =
            self.db_query_wrapper_for_one_row(transaction, sql_in, "i64")?;
        let result: i64 = match results[0] {
            Some(DataType::Bigint(x)) => x,
            _ => {
                return Err(anyhow!(
                    "In extract_row_count_from_count_query, Should never happen".to_string()
                ))
            }
        };
        let result_u64: u64 = result.try_into()?;
        Ok(result_u64)
    }

    /// Used, for example, when test code is finished with its test data. Be careful.
    pub fn destroy_tables(&self) -> Result<(), anyhow::Error> {
        //%%see comments at similar places elsewhere, re:  Search for related cmts w/ "isolation".
        // conn.setTransactionIsolation(Connection.TRANSACTION_SERIALIZABLE)

        /**** WHEN MAINTAINING THIS METHOD, SIMILARLY MAINTAIN THE SCRIPT*S*
        core/bin/purge-om-test-database* SO THEY DO THE SAME WORK. ****/

        // Doing these individually so that if one fails (not previously existing, such as
        // testing or a new installation), the others can proceed (drop method ignores that
        // exception).
        self.drop(&None, "table", "om_db_version")?;
        self.drop(&None, "table", Util::QUANTITY_TYPE)?;
        self.drop(&None, "table", Util::DATE_TYPE)?;
        self.drop(&None, "table", Util::BOOLEAN_TYPE)?;
        // The next line is to invoke the trigger that will clean out Large Objects
        // (FileAttributeContent...) from the table pg_largeobject.
        // The LO cleanup doesn't happen (trigger not invoked) w/ just a drop (or truncate),
        // but does on delete.  For more info see the wiki reference
        // link among those down in this file below "create table FileAttribute".
        let result: Result<u64, anyhow::Error> = self.db_action(
            &None,
            "delete from FileAttributeContent",
            /*%%caller_checks_row_count_etc =*/ true,
            false,
        );
        if let Err(msg) = result {
            if !msg.to_string().to_lowercase().contains("does not exist") {
                return Err(anyhow!(msg.to_string().clone()));
            }
        }
        self.drop(&None, "table", "FileAttributeContent")?;
        self.drop(&None, "table", Util::FILE_TYPE)?;
        self.drop(&None, "table", Util::TEXT_TYPE)?;
        self.drop(&None, "table", Util::RELATION_TO_LOCAL_ENTITY_TYPE)?;
        self.drop(&None, "table", Util::RELATION_TO_REMOTE_ENTITY_TYPE)?;
        self.drop(&None, "table", "EntitiesInAGroup")?;
        self.drop(&None, "table", Util::RELATION_TO_GROUP_TYPE)?;
        self.drop(&None, "table", "action")?;
        self.drop(&None, "table", "grupo")?;
        self.drop(&None, "table", Util::RELATION_TYPE_TYPE)?;
        self.drop(&None, "table", "AttributeSorting")?;
        self.drop(&None, "table", "omInstance")?;
        self.drop(&None, "table", Util::ENTITY_TYPE)?;
        self.drop(&None, "table", "class")?;
        self.drop(&None, "sequence", "EntityKeySequence")?;
        self.drop(&None, "sequence", "ClassKeySequence")?;
        self.drop(&None, "sequence", "TextAttributeKeySequence")?;
        self.drop(&None, "sequence", "QuantityAttributeKeySequence")?;
        self.drop(&None, "sequence", "RelationTypeKeySequence")?;
        self.drop(&None, "sequence", "ActionKeySequence")?;
        self.drop(&None, "sequence", "RelationToEntityKeySequence")?;
        self.drop(&None, "sequence", "RelationToRemoteEntityKeySequence")?;
        self.drop(&None, "sequence", "RelationToGroupKeySequence")?;
        self.drop(&None, "sequence", "RelationToGroupKeySequence2")?;
        self.drop(&None, "sequence", "DateAttributeKeySequence")?;
        self.drop(&None, "sequence", "BooleanAttributeKeySequence")?;
        self.drop(&None, "sequence", "FileAttributeKeySequence")
    }

    //idea: change sql_type to take an enum, not a string.
    // fn drop<'a, E>(&self, executor: Option<E>, sql_type: &str, name: &str) -> Result<(), String>
    fn drop(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        sql_type: &str,
        name: &str,
    ) -> Result<(), anyhow::Error> {
        let sql: String = format!(
            "DROP {} IF EXISTS {} CASCADE",
            Self::escape_quotes_etc(sql_type.to_string()),
            Self::escape_quotes_etc(name.to_string())
        );
        let result: Result<u64, anyhow::Error> =
            self.db_action(transaction, sql.as_str(), false, false);
        match result {
            Err(msg) => {
                // (Now that "IF EXISTS" is added in the above DROP statement, this check might
                // not be needed. No harm though?  If it does not exist pg replies with a
                // notification, per the pg docs.  Not sure at this writing how that is
                // reported by sqlx here though.)
                if !msg.to_string().contains("does not exist") {
                    Err(anyhow!(msg.to_string().clone()))
                } else {
                    Ok(())
                }
            }
            _ => Ok(()),
        }
    }

    /// For text fields (which by the way should be surrounded with single-quotes ').
    /// Best to use this with only one field at a time, so you don't escape the single-ticks
    /// that *surround* the field.
    fn escape_quotes_etc(s: String) -> String {
        let mut result: String = s;
        /*
        //both of these seem to work to embed a ' (single quote) in interactive testing w/ psql: the SQL standard
        //way (according to http://www.postgresql.org/docs/9.1/interactive/sql-syntax-lexical.html#SQL-SYNTAX-STRINGS )
        //    update entity set (name) = ROW('len''gth4') where id=-9223372036854775807;
        //...or the postgresql extension way (also works for: any char (\a is a), c-like (\b, \f, \n, \r, \t), or
        //hex (eg \x27), or "\u0027 (?) , \U0027 (?)  (x = 0 - 9, A - F)  16 or 32-bit
        //hexadecimal Unicode character value"; see same url above):
        //    update entity set (name) = ROW(E'len\'gth4') where id=-9223372036854775807;
         */
        // we don't have to do much: see the odd string that works ok, searching for "!@#$%" etc in PostgreSQLDatabaseTest.
        result = result.replace("'", "\\39");
        // there is probably a different/better/right way to do this, possibly using the psql functions quote_literal or quote_null,
        // or maybe using "escape" string constants (a postgresql extension to the sql standard). But it needs some thought, and maybe
        // this will work for now, unless someone needs to access the DB in another form. Kludgy, yes. It's on the fix list.
        result = result.replace(";", "\\59");
        result
    }

    fn unescape_quotes_etc(s: String) -> String {
        // don't have to do the single-ticks ("'") because the db does that automatically when returning data (see PostgreSQLDatabaseTest).

        let mut result: String = s;
        result = result.replace("\\39", "'");
        result = result.replace("\\59", ";");
        result
    }

    /// Returns the # of rows affected.
    /// @param skip_check_for_bad_sql_in  SET TO false EXCEPT *RARELY*, WITH CAUTION AND ONLY WHEN THE SQL HAS NO USER-PROVIDED STRING IN IT!!  SEE THE (hopefully
    ///                              still just one) PLACE USING IT NOW (in method create_attribute_sorting_deletion_trigger) AND PROBABLY LIMIT USE TO THAT!
    fn db_action(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        sql_in: &str,
        caller_checks_row_count_etc: bool, /*%% = false*/
        skip_check_for_bad_sql_in: bool,   /*%% = false*/
    ) -> Result<u64, anyhow::Error> {
        let mut rows_affected: u64 = 0;
        let is_create_drop_or_alter = sql_in.to_lowercase().starts_with("create ")
            || sql_in.to_lowercase().starts_with("drop ")
            || sql_in.to_lowercase().starts_with("alter ");
        if !skip_check_for_bad_sql_in {
            Self::check_for_bad_sql(sql_in)?;
        }
        let x: Result<PgQueryResult, sqlx::Error> = if let Some(_tx) = transaction {
            //%%WHEN FIXING, PUT NEXT LINE BACK AND REMOVE THE ONE AFTER, and remove comment
            //re this, in failing test, if also fixed in fn db_query.
            // let future = sqlx::query(sql_in).execute(tx); //a try
            // let future = sqlx::query(sql_in).execute(transaction.as_ref().unwrap()); //another try
            let future = sqlx::query(sql_in).execute(&self.pool); //the fallback, but loses transaction features
            self.rt.block_on(future)
        } else {
            let future = sqlx::query(sql_in).execute(&self.pool);
            self.rt.block_on(future)
        };
        debug!(
            "In db_action, sql is: {}\n... {:?} rows affected, w/ result: {:?}",
            sql_in, rows_affected, &x
        );
        match x {
            Err(e) => return Err(anyhow!(e.to_string())),
            Ok(res) => {
                rows_affected = res.rows_affected();
            }
        };

        //%%HOW DO THIS or whatever needed, w/ sqlx??:
        // idea: not sure whether these checks belong here really.  Might be worth research
        // to see how often warnings actually should be addressed, & how to routinely tell the difference. If so, do the same at the
        // other place(s) that use getWarnings.
        // let warnings = st.getWarnings;
        // if warnings != null
        //     && !warnings.toString.contains("NOTICE: CREATE TABLE / PRIMARY KEY will create implicit index")
        //     && !warnings.toString.contains("NOTICE: drop cascades to 2 other objects")
        //     && !warnings.toString.contains("NOTICE: drop cascades to constraint valid_related_to_entity_id on table class")
        // ) {
        //   throw new OmDatabaseException("Warnings from postgresql. Matters? Says: " + warnings)
        // }
        if !caller_checks_row_count_etc && !is_create_drop_or_alter && rows_affected != 1 {
            return Err(anyhow!(format!(
                "Affected {} rows instead of 1?? SQL was: {}",
                rows_affected, sql_in
            )));
        }
        Ok(rows_affected)
    }

    fn check_for_bad_sql(s: &str) -> Result<(), anyhow::Error> {
        if s.contains(";") {
            // it seems that could mean somehow an embedded sql is in a normal command, as an attack vector. We don't usually need
            // to write like that, nor accept it from outside. This & any similar needed checks should happen reliably
            // at the lowest level before the database for security.  If text needs the problematic character(s), it should
            // be escaped prior (see escape_quotes_etc for writing data, and where we read data).
            Err(anyhow!("Input can't contain ';'"))
        } else {
            Ok(())
        }
    }

    /* %%This is just to see what is different that causes compile errors, between code in
    // util.rs/ fn initialize_test_db and fn new here.
    // pub fn new_test(username: &str,
    //         password: &str,) -> Result<Box<dyn Database>, String> {
    // pub fn new_test() -> Result<PostgreSQLDatabase, &'static str> {
    pub fn new_test() -> Result<PostgreSQLDatabase, String> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        // let result = Self::connect(&rt, Util::TEST_USER, Util::TEST_USER, Util::TEST_PASS);
        // let pool: PgPool;
        // match result {
        //     Ok(x) => pool = x,
        // //     Err(e) => return Err(anyhow!(e.to_string())),
        // Err(e) => return Err(anyhow!(e.to_string())),
        // }
        let pool =
            PostgreSQLDatabase::connect(&rt, Util::TEST_USER, Util::TEST_USER, Util::TEST_PASS)
                .unwrap();
        let db: PostgreSQLDatabase = PostgreSQLDatabase {
            rt,
            pool,
            include_archived_entities: false,
        };
        //NEXT LINE IS WHAT LETS IT HAPPEN HERE! IS THAT OK, to do what we are doing in this?
        TEST_DB_INIT2.call_once(|| {
        //     db.destroy_tables().unwrap();
            let mut tx = db
                .begin_trans()
                .expect("Failure to begin transaction before creating test data.");
            // db.create_tables(&Some(&mut tx)).unwrap();
            // db.commit_trans(&mut tx)
            //     .expect("Failure to commit transaction after creating test data.");
        });
        Ok(db)
    }
    */

    /// Any code that would change when we change storage systems (like from postgresql to
    /// an object database or who knows), goes in this class.
    /// Note that any changes to the database structures (or constraints, etc) whatsoever should
    /// ALWAYS have the following: <ul>
    /// <li>Constraints, rules, functions, stored procedures, or triggers
    /// or something to enforce data integrity and referential integrity at the database level,
    /// whenever possible. When this is impossible, it should be discussed on the developer mailing
    /// so that we can consider putting it in the right place in the code, with the goal of
    /// greatest simplicity and reliability.</li>
    /// <li>Put these things in the auto-creation steps of the DB class. See create_base_data(), create_tables(), and doDatabaseUpgrades.</li>
    /// <li>Add comments to that part of the code, explaining the change or requirement, as needed.</li>
    /// <li>Any changes (as anywhere in this system) should be done in a test-first manner, for anything that
    /// could go wrong, along these lines: First write a test that demonstrates the issue and fails, then
    /// write code to correct the issue, then re-run the test to see the successful outcome. This helps keep our
    /// regression suite current, and could even help think through design issues without over-complicating things.
    /// </ul>
    ///
    /// This creates a new instance of Database. By default, auto-commit is on unless you explicitly
    /// open a transaction with begin_trans; then auto-commit will be off until you rollback_trans
    /// or commit_trans (or be rolled back upon the transaction going out of scope), at which
    /// point auto-commit is turned back on.  (At least I guess that is still true w/ sqlx as it
    /// was with jdbc.)
    ///
    /// In the scala code this was called login().
    pub fn new(
        /*%%hopefully del this cmt, was: &self, */ username: &str,
        password: &str,
    ) -> Result<Box<dyn Database>, anyhow::Error> {
        let include_archived_entities = false;
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let result = Self::connect(&rt, username, username, password);
        let pool: PgPool;
        match result {
            Ok(x) => pool = x,
            // Err(e) => return Err(e.to_string()),
            Err(e) => return Err(anyhow!(e.to_string())),
        }
        let new_db = PostgreSQLDatabase {
            rt,
            pool,
            include_archived_entities,
        };

        //%% why does having this here instead of in a separate fn setup_db cause
        //a compile error about using a moved value new_db (moved on the next line, returned in the
        //"Ok" line below)????  should I make a test and submit it, to learn or have it fixed or??
        //Note that *another* seeming solution is the line "TEST_DB_INIT2.call_once(|| {" (and the
        //part to end its block) found in experimental fn new_test, above.
        // // let x = new_db.begin_trans_test();
        // let mut tx = new_db.begin_trans();
        // let mut tx = match tx {
        // // let mut tx = match rt.block_on(pool.begin()) {
        //     Err(e) => {
        //         return Err(anyhow!(format!(
        //             "Unable to start a database transaction to set up database?: {}",
        //             e.to_string()
        //         )))
        //     }
        //     Ok(t) => t,
        // };
        // if !new_db.model_tables_exist(&Some(&mut tx))? {
        //     // //%%$% try to see what happens if pg down be4 & during this--does the err propagate ok?
        //     new_db.create_tables(&Some(&mut tx))?;
        //     //%%$% try to see what happens if pg down be4 & during this--does the err propagate ok?
        //     new_db.create_base_data(&Some(&mut tx))?;
        // }
        // //%% do_database_upgrades_if_needed()
        // new_db.create_and_check_expected_data(&Some(&mut tx))?;
        // match new_db.commit_trans(&mut tx) {
        //     Err(e) => {
        //         return Err(anyhow!(format!(
        //             "Unable to commit database transaction for db setup: {}",
        //             e.to_string()
        //         )))
        //     }
        //     Ok(t) => t,
        // }
        new_db.setup_db()?;

        Ok(Box::new(new_db))
    }
    //%%
    //Moved from fn new to see about addressing a compile error. See cmts there.
    //(removed next line to remove noise from debug output in test log)
    // #[tracing::instrument]
    pub fn setup_db(&self) -> Result<(), anyhow::Error> {
        // let x = new_db.begin_trans_test();
        let mut tx = self.begin_trans()?;
        if !self.model_tables_exist(&Some(&mut tx))? {
            // //%%$% try to see what happens if pg down be4 & during this--does the err propagate ok?
            self.create_tables(&Some(&mut tx))?;
            //%%$% try to see what happens if pg down be4 & during this--does the err propagate ok?
            self.create_base_data(&Some(&mut tx))?;
        }
        self.do_database_upgrades_if_needed(&Some(&mut tx))?;
        self.create_and_check_expected_data(&Some(&mut tx))?;
        self.commit_trans(tx)
    }

    /// For newly-assumed data in existing systems.  I.e., not a database schema change, and was added to the system (probably expected by the code somewhere),
    /// after an OM release was done.  This puts it into existing databases if needed.
    fn create_and_check_expected_data<'a>(
        &'a self,
        transaction: &Option<&mut Transaction<'a, Postgres>>,
    ) -> Result<(), anyhow::Error> {
        debug!("starting fn create_and_check_expected_data");
        //Idea: should this really be in the Controller then?  It wouldn't differ by which database type we are using.  Hmm, no, if there were multiple
        // database types, there would probably a parent class over them (of some kind) to hold this.
        let system_entity_id: i64 = self.get_system_entity_id(transaction)?;
        // Idea: surely there is some better way to manage ownership of these transactions?  It
        // boils down to what happens with them, inside db_query and db_action, and everything that
        // has to pass parameters around.  Unless we ignore the need to roll back (search for
        // "rollback" for related comments/explanation?).  See also the below tests
        // "test_rollback_and_commit" and "test_rollback_and_commit_with_less_helper_code".  See
        // also comments in delete_objects.
        //%%del above cmt? "transaction2".
        let type_id_of_the_has_relation: i64 =
            self.find_relation_type(transaction, Util::THE_HAS_RELATION_TYPE_NAME)?;

        let preferences_container_id: i64 = {
            let preferences_entity_id: Option<i64> = self.get_relation_to_local_entity_by_name(
                transaction,
                self.get_system_entity_id(transaction)?,
                Util::USER_PREFERENCES,
            )?;
            debug!("in create_and_check_expected_data: before 'match preferences_entity_id' for USER_PREFERENCES");
            match preferences_entity_id {
                Some(id) => id,
                None => {
                    // Since necessary, also create the entity that contains all the preferences:
                    let now = Utc::now().timestamp_millis();
                    debug!("in create_and_check_expected_data: in 'match preferences_entity_id' for USER_PREFERENCES, before create_entity_and_relation_to_local_entity");
                    let new_entity_id: i64 = self
                        .create_entity_and_relation_to_local_entity(
                            transaction,
                            system_entity_id,
                            type_id_of_the_has_relation,
                            Util::USER_PREFERENCES,
                            None,
                            Some(now),
                            now,
                            true,
                        )?
                        .0;
                    debug!("in create_and_check_expected_data: in 'match preferences_entity_id' for USER_PREFERENCES, after create_entity_and_relation_to_local_entity");
                    new_entity_id
                }
            }
        };
        debug!("in create_and_check_expected_data: after 'match preferences_entity_id' for USER_PREFERENCES");
        // (Not doing the default entity preference here also, because it might not be set by now and is not assumed to be.)
        if self
            .get_user_preference2(
                transaction,
                preferences_container_id,
                Util::SHOW_PUBLIC_PRIVATE_STATUS_PREFERENCE,
                Util::PREF_TYPE_BOOLEAN,
            )?
            .len()
            == 0
        {
            self.set_user_preference_boolean(
                transaction,
                Util::SHOW_PUBLIC_PRIVATE_STATUS_PREFERENCE,
                false,
            )?;
        }
        Ok(())
    }

    pub fn connect(
        rt: &tokio::runtime::Runtime,
        db_name_without_prefix: &str,
        username: &str,
        password: &str,
    ) -> Result<PgPool, sqlx::Error> {
        // (to connect to remote hosts, see logic in the connect() method and jdbcUrl(), in the older
        // PostgreSQLDatabase.scala file.  db_name() has replaced it here for now.)
        let connect_str = format!(
            "postgres://{}:{}@localhost/{}",
            username,
            password,
            Self::db_name(db_name_without_prefix)
        );
        let future = PgPoolOptions::new()
            // idea: the example had 5, could switch to not using pools, or use pools again now/later if it matters?
            // I had max_connections(10), but then a test fails with "pool timed out while waiting for an open connection",
            // or "PoolTimedOut" aka from sql docs at https://docs.rs/sqlx/latest/sqlx/enum.Error.html#variant.PoolTimedOut
            // "A Pool::acquire timed out due to connections not becoming available or because another task encountered too
            // many errors while trying to open a new connection."
            // even if it is the only test running (test_set_user_preference_and_get_user_preference).
            // %%Maybe raising just naively kicks the problem down the road; will see.  Could see also:
            // https://github.com/launchbadge/sqlx/issues/1199
            // and mbe  https://www.google.com/search?q=sqlx+%22PoolTimedOut%22&hl=eo&gbv=1&sei=y9mtZPjEB7XakPIP_qOi-Aw  .
            .max_connections(250)
            // .connect(connect_str.as_str())?;
            //%% be sure to test this by querying it, ad-hoc for now, later in a test, maybe something like:
            //     om_t1=> show transaction isolation level;
            //     transaction_isolation
            //         -----------------------
            //         read committed
            //         (1 row)
            // (to see the default, instead:   show default_transaction_isolation;
            // or more stuff:   show all;  ).
            //%%do this by sending a query like below per examples, and retrieve info: would work? Or, need to use PgConnectOptions instead of pool?
            //.options([("default_transaction_isolation","serializable")])
            //%%use .connect_with and pass options?? for transaction isolation levell...?  Is also
            // mentioned in one of the early parts of below "mod tests" below I think.
            // Search for related cmts w/ "isolation".
            .connect(connect_str.as_str());
        let pool = rt.block_on(future)?;
        // pool.options().
        // let pool = future;
        //%%$%just some testing, can delete after next commit, or use for a while for reference.
        // // let future = sqlx::query_as("SELECT $1")
        // let future = sqlx::query_as("SELECT count(1) from entity")
        //     .bind(150_i64)
        // OR: .bind("a new ticket (if the sql was insert...)?")
        //     .fetch_one(&pool);
        // let row: (i64, ) = rt.block_on(future).unwrap();
        // // assert_eq!(row.0, 150);
        // debug!("in connect: Result returned from sql!: {}  ******************************", row.0);

        //%%query examples at:
        //      https://gist.github.com/jeremychone/34d1e3daffc38eb602b1a9ab21298d10
        //      https://betterprogramming.pub/how-to-interact-with-postgresql-from-rust-using-sqlx-cfa2a7c758e7?gi=bfc149911f80
        //      from ddg/web search for:  rust sqlx examples postgres

        //%%the below does not show anything, and it is probably not set.  Maybe later if there is a
        // way to seek support or q/a for sqlx, ask how to set/check it?  Could maybe set it by the
        // options method when getting a single connection (but it seems not to be there for getting
        // a pool).  Search for related cmts w/ "isolation".
        let future = sqlx::query("show transaction isolation level").execute(&pool);
        let x = rt.block_on(future)?;
        debug!(
            "In connect: Query result re transaction isolation lvl?:  {:?}",
            x
        );

        Ok(pool)
    }
    /// Indicates whether the database setup has been done.
    fn model_tables_exist(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
    ) -> Result<bool, anyhow::Error> {
        self.does_this_exist(
            transaction,
            "select count(1) from pg_class where relname='entity'",
            true,
        )
    }

    fn create_version_table(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
    ) -> Result<u64, anyhow::Error> {
        // table has 1 row and 1 column, to say what db version we are on.
        self.db_action(
            transaction,
            // default 1 due to lack of a better idea.  See comment just below.
            "create table om_db_version (version integer DEFAULT 1) ",
            false,
            false,
        )?;
        self.db_action(
            transaction,
            // Initially 0 due to lack of a better idea.  The other setup code (fn create_tables
            // currently) should set it correctly to the updated version, once the schema with
            // that specific version has actually been created.
            "INSERT INTO om_db_version (version) values (0)",
            false,
            false,
        )
    }

    /// Does standard setup for a "OneModel" database, such as when starting up for the first time, or when creating a test system.
    /// Currently returns the # of rows affected by the last sql command (not interesting).
    /// NOTE: MAKE SURE everything this does is also covered elsewhere as needed: see the comment at
    /// the top of fn do_database_upgrades_if_needed.
    pub fn create_tables(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
    ) -> Result<(), anyhow::Error> {
        self.create_version_table(transaction)?;

        self.db_action(
            transaction,
            format!(
                "create sequence EntityKeySequence minvalue {}",
                self.min_id_value()
            )
            .as_str(),
            false,
            false,
        )?;

        // The id must be "unique not null" in ANY database used, because it is a primary key. "PRIMARY KEY" is the same.
        // 'archived' is only on Entity for now, to see if rows from related tables just don't show up because we
        // never link to them (never seeing the linking Entity rows), so they're effectively hidden/archived too.
        // At some point we could consider moving all those rows (entities & related...) to separate tables instead,
        // for performance/space if needed (including 'public').
        // The insertion_date is intended to be a readonly date: the (*java*-style numeric: milliseconds since 1970-1-1 or
        // such) when this row was inserted (ie, when the entity object was created in the db):
        // A null in the 'public' field means 'undecided' (effectively "false", but a different nuance,e.g. in case
        // user wants to remember to decide later)
        // The field new_entries_.... tells the UI that, with the highlight at the beginning of the list, attributes added
        // to an entity should become the new 1st entry, not 2nd.
        // (ie, grows from the top: convenient sometimes like for logs, but most of the time it is more convenient
        // for creating the 2nd entry after
        // the 1st one, such as when creating new lists).
        self.db_action(transaction, format!("create table Entity (\
            id bigint DEFAULT nextval('EntityKeySequence') PRIMARY KEY, \
            name varchar({}) NOT NULL, \
            class_id bigint, \
            archived boolean NOT NULL default false, \
            archived_date bigint check ((archived is false and archived_date is null) OR (archived and archived_date is not null)), \
            insertion_date bigint not null, \
            public boolean, \
            new_entries_stick_to_top boolean NOT NULL default false\
            ) ", Util::entity_name_length()).as_str(), false, false)?;

        // not unique, but for convenience/speed:
        self.db_action(
            transaction,
            "create index entity_lower_name on Entity (lower(NAME))",
            false,
            false,
        )?;

        self.db_action(
            transaction,
            format!(
                "create sequence ClassKeySequence minvalue {}",
                self.min_id_value()
            )
            .as_str(),
            false,
            false,
        )?;

        // The name here doesn't have to be the same name as in the related Entity record, (since it's not a key, and it might not make sense to match).
        // For additional comments on usage, see the Controller.askForInfoAndcreate_entity method.
        // Since in the code we can't call it class, the class that represents this in the model is called EntityClass.
        // The defining_entity_id is, in other words, template, aka class-defining entity.
        // The create_default_attributes means whether the user wants the program to create all
        // the attributes by default, using the defining_entity's attrs as a template.
        let sql = format!("create table Class (\
            id bigint DEFAULT nextval('ClassKeySequence') PRIMARY KEY, \
            name varchar({}) NOT NULL, \
            defining_entity_id bigint UNIQUE NOT NULL, \
            create_default_attributes boolean, \
            CONSTRAINT valid_related_to_entity_id FOREIGN KEY (defining_entity_id) REFERENCES entity (id) \
            )", Util::class_name_length());
        self.db_action(transaction, sql.as_str(), false, false)?;

        self.db_action(transaction, "alter table entity add CONSTRAINT valid_related_to_class_id FOREIGN KEY (class_id) REFERENCES class (id)", false, false)?;

        self.db_action(
            transaction,
            format!(
                "create sequence RelationTypeKeySequence minvalue {}",
                self.min_id_value()
            )
            .as_str(),
            false,
            false,
        )?;

        // this table "inherits" from Entity (each relation type is an Entity) but we use homegrown "inheritance" for that to make it
        // easier to port to databases that don't have postgresql-like inheritance built in. It inherits from Entity so that as Entity
        // expands (i.e., context-based naming or whatever), we'll automatically get the benefits, in objects based on this table (at least
        // that's the idea at this moment...) --Luke Call 8/2003.  Update:  That may have been a mistake--more of a nuisance to coordinate
        // them than having 2 tables (luke, 2013-11-1).
        // inherits from Entity; see RelationConnection for more info.
        // Note, 2014-07: At one point I considered whether this concept overlaps with that of class, but now I think they are quite separate.  This table
        // could fill the concept of an entity that *is* a relationship, containing e.g. the date a relationship began, or any other attributes that are not about
        // either participant, but about the relationship itself.  One such use could be: I "have" a physical object, I and the object being entities with
        // classes, and the "have" is not a regular generic "have" type (as defined by the system at first startup), but a particular one (maybe RelationType
        // should be renamed to "RelationEntity" or something: think about all this some more: more use cases etc).

        // Valid values for directionality are "BI ","UNI","NON"-directional for this relationship. example: parent/child is unidirectional. sibling is bidirectional,
        // and for nondirectional
        // see Controller's mention of "nondir" and/or elsewhere for comments
        self.db_action(transaction, format!("create table RelationType (\
            entity_id bigint PRIMARY KEY, \
            name_in_reverse_direction varchar({}), \
            directionality char(3) CHECK (directionality in ('BI','UNI','NON')), \
            CONSTRAINT valid_rel_entity_id FOREIGN KEY (entity_id) REFERENCES Entity (id) ON DELETE CASCADE \
            ) ", Util::relation_type_name_length()).as_str(), false, false)?;

        /* This table maintains the users' preferred display sorting information for entities' attributes (including relations to groups/entities).
        It might instead have been implemented by putting the sorting_index column on each attribute table, which would simplify some things, but that
        would have required writing a new way for placing & sorting the attributes and finding adjacent ones etc., and the first way was already
        mostly debugged, with much effort (for EntitiesInAGroup, and the hope is to reuse that way for interacting with this table).  But maybe that
        same effect could have been created by sorting the attributes in memory instead, adhoc when needed: not sure if that would be simpler
        */
        // The entity_id is the entity whose attribute this is.
        // The sorting_index is the reason for this table.
        // The attribute_form_id  is for which table the attribute is in.  Method getAttributeForm has details.
        // The constraint noDupSortingIndexes2 is to make it so the sorting_index must also be unique for each entity (otherwise we have sorting problems).
        // The constraint noDupSortingIndexes3 was required by the constraint valid_*_sorting on the tables that have a form_id column.
        self.db_action(transaction, "create table AttributeSorting (\
            entity_id bigint NOT NULL\
            , attribute_form_id smallint NOT NULL\
            , attribute_id bigint NOT NULL\
            , sorting_index bigint not null\
            , PRIMARY KEY (entity_id, attribute_form_id, attribute_id)\
            , CONSTRAINT valid_entity_id FOREIGN KEY (entity_id) REFERENCES entity (id) ON DELETE CASCADE\
            , CONSTRAINT valid_attribute_form_id CHECK (attribute_form_id >= 1 AND attribute_form_id <= 8)\
            , constraint noDupSortingIndexes2 unique (entity_id, sorting_index)\
            , constraint noDupSortingIndexes3 unique (attribute_form_id, attribute_id)\
            ) ", false, false)?;

        self.db_action(
            transaction,
            "create index AttributeSorting_sorted on AttributeSorting (entity_id, sorting_index)",
            false,
            false,
        )?;

        self.create_attribute_sorting_deletion_trigger(transaction)?;

        self.db_action(
            transaction,
            format!(
                "create sequence QuantityAttributeKeySequence minvalue {}",
                self.min_id_value()
            )
            .as_str(),
            false,
            false,
        )?;

        // The entity_id is the key for the entity on which this quantity info is recorded; for other meanings see comments on
        // Entity.addQuantityAttribute(...).
        // The id must be "unique not null" in ANY database used, because it is the primary key.
        // FOR COLUMN MEANINGS, SEE ALSO THE COMMENTS IN CREATEQUANTITYATTRIBUTE.
        // For form_id, see comment for this column under "create table RelationToGroup", below.
        // The unit_id refers to a unit (an entity), like "meters".
        // For quantity_number: eg, 50.0.
        // For attr_type_id eg, length (an entity).
        // For valid_on_date, see "create table RelationToEntity" for comments about dates' meanings.

        // For the constraint valid_qa_sorting: didn't use "on delete cascade",
        // because it didn't originally occur to me that instead of deleting the
        // sorting row (via triggers) when we delete the attribute, we could delete the attribute when deleting its sorting row, by instead
        // putting "ON DELETE CASCADE" on the attribute tables' constraints that reference this table, and where we
        // now delete attributes, instead deleting AttributeSorting rows, and so letting the attributes be deleted automatically.
        // But for now, see the trigger below instead.
        // (The same is true for all the attribute tables (including the 2 main RelationTo* tables).

        // (The "DEFERRABLE INITIALLY DEFERRED" is because otherwise when an attribute is deleted, it would
        // fail on this constraint before the trigger files to delete the row from
        // attributesorting.)
        let quantity_form_id: i32 = self.get_attribute_form_id(Util::QUANTITY_TYPE).unwrap();
        self.db_action(transaction, format!("create table QuantityAttribute (\
            form_id smallint DEFAULT {} \
                NOT NULL CHECK (form_id={}), \
            id bigint DEFAULT nextval('QuantityAttributeKeySequence') PRIMARY KEY, \
            entity_id bigint NOT NULL, \
            unit_id bigint NOT NULL, \
            quantity_number double precision not null, \
            attr_type_id bigint not null, \
            valid_on_date bigint, \
            observation_date bigint not null, \
            CONSTRAINT valid_unit_id FOREIGN KEY (unit_id) REFERENCES entity (id), \
            CONSTRAINT valid_attr_type_id FOREIGN KEY (attr_type_id) REFERENCES entity (id), \
            CONSTRAINT valid_parent_id FOREIGN KEY (entity_id) REFERENCES entity (id) ON DELETE CASCADE, \
            CONSTRAINT valid_qa_sorting FOREIGN KEY (entity_id, form_id, id) REFERENCES attributesorting (entity_id, attribute_form_id, attribute_id) \
              DEFERRABLE INITIALLY DEFERRED \
            )", quantity_form_id, quantity_form_id).as_str(), false, false)?;
        self.db_action(
            transaction,
            "create index quantity_parent_id on QuantityAttribute (entity_id)",
            false,
            false,
        )?;
        self.db_action(
            transaction,
            "CREATE TRIGGER qa_attribute_sorting_cleanup BEFORE DELETE ON QuantityAttribute \
            FOR EACH ROW EXECUTE PROCEDURE attribute_sorting_cleanup()",
            false,
            false,
        )?;

        self.db_action(
            transaction,
            format!(
                "create sequence DateAttributeKeySequence minvalue {}",
                self.min_id_value()
            )
            .as_str(),
            false,
            false,
        )?;
        // see comment for the form_id column under "create table RelationToGroup", below:
        let date_form_id = self.get_attribute_form_id(Util::DATE_TYPE).unwrap();
        // About the attr_type_id: e.g., due on, done on, should start on, started on on... (which would be an entity).
        self.db_action(transaction, format!("create table DateAttribute (\
            form_id smallint DEFAULT {} \
                NOT NULL CHECK (form_id={}), \
            id bigint DEFAULT nextval('DateAttributeKeySequence') PRIMARY KEY, \
            entity_id bigint NOT NULL, \
            attr_type_id bigint not null, \
            date bigint not null, \
            CONSTRAINT valid_attr_type_id FOREIGN KEY (attr_type_id) REFERENCES entity (id), \
            CONSTRAINT valid_parent_id FOREIGN KEY (entity_id) REFERENCES entity (id) ON DELETE CASCADE, \
            CONSTRAINT valid_da_sorting FOREIGN KEY (entity_id, form_id, id) REFERENCES attributesorting (entity_id, attribute_form_id, attribute_id) \
              DEFERRABLE INITIALLY DEFERRED \
            ) ", date_form_id, date_form_id).as_str(), false, false)?;
        self.db_action(
            transaction,
            "create index date_parent_id on DateAttribute (entity_id)",
            false,
            false,
        )?;
        self.db_action(
            transaction,
            "CREATE TRIGGER da_attribute_sorting_cleanup BEFORE DELETE ON DateAttribute \
            FOR EACH ROW EXECUTE PROCEDURE attribute_sorting_cleanup()",
            false,
            false,
        )?;

        self.db_action(
            transaction,
            format!(
                "create sequence BooleanAttributeKeySequence minvalue {}",
                self.min_id_value()
            )
            .as_str(),
            false,
            false,
        )?;
        let boolean_form_id = self.get_attribute_form_id(Util::BOOLEAN_TYPE).unwrap();
        // See comment for the form_id column under "create table RelationToGroup", below.
        // For the booleanValue column: allowing nulls because a template might not have \
        // value, and a task might not have a "done/not" setting yet (if unknown)?
        // Ex., isDone (where the task would be an entity).
        // See "create table RelationToEntity" for comments about dates' meanings.
        self.db_action(transaction, format!("create table BooleanAttribute (\
            form_id smallint DEFAULT {} \
                NOT NULL CHECK (form_id={}), \
            id bigint DEFAULT nextval('BooleanAttributeKeySequence') PRIMARY KEY, \
            entity_id bigint NOT NULL, \
            booleanValue boolean, \
            attr_type_id bigint not null, \
            valid_on_date bigint, \
            observation_date bigint not null, \
            CONSTRAINT valid_attr_type_id FOREIGN KEY (attr_type_id) REFERENCES entity (id), \
            CONSTRAINT valid_parent_id FOREIGN KEY (entity_id) REFERENCES entity (id) ON DELETE CASCADE, \
            CONSTRAINT valid_ba_sorting FOREIGN KEY (entity_id, form_id, id) REFERENCES attributesorting (entity_id, attribute_form_id, attribute_id) \
              DEFERRABLE INITIALLY DEFERRED \
            ) ", boolean_form_id, boolean_form_id).as_str(), false, false)?;
        self.db_action(
            transaction,
            "create index boolean_parent_id on BooleanAttribute (entity_id)",
            false,
            false,
        )?;
        self.db_action(
            transaction,
            "CREATE TRIGGER ba_attribute_sorting_cleanup BEFORE DELETE ON BooleanAttribute \
            FOR EACH ROW EXECUTE PROCEDURE attribute_sorting_cleanup()",
            false,
            false,
        )?;

        self.db_action(
            transaction,
            format!(
                "create sequence FileAttributeKeySequence minvalue {}",
                self.min_id_value()
            )
            .as_str(),
            false,
            false,
        )?;
        let file_form_id = self.get_attribute_form_id(Util::FILE_TYPE).unwrap();
        // See comment for form_id under "create table RelationToGroup", below.
        // About the attr_type_id: e.g., refers to a type like txt: i.e., could be like mime types, extensions, or mac fork info, etc (which would be an entity in any case).
        // Now that i already wrote this, maybe storing 'readable' is overkill since the system has to read it to store its content. Maybe there's a use.
        // Moved to other table:   contents bit varying NOT NULL,
        // The md5hash is the md5 hash in hex (just to see if doc has become corrupted; not intended for security/encryption)
        self.db_action(transaction, format!("create table FileAttribute (\
            form_id smallint DEFAULT {} \
                NOT NULL CHECK (form_id={}), \
            id bigint DEFAULT nextval('FileAttributeKeySequence') PRIMARY KEY, \
            entity_id bigint NOT NULL, \
            attr_type_id bigint NOT NULL, \
            description text NOT NULL, \
            original_file_date bigint NOT NULL, \
            stored_date bigint NOT NULL, \
            original_file_path text NOT NULL, \
            readable boolean not null, \
            writable boolean not null, \
            executable boolean not null, \
            size bigint NOT NULL, \
            md5hash char(32) NOT NULL, \
            CONSTRAINT valid_attr_type_id FOREIGN KEY (attr_type_id) REFERENCES entity (id), \
            CONSTRAINT valid_parent_id FOREIGN KEY (entity_id) REFERENCES entity (id) ON DELETE CASCADE, \
            CONSTRAINT valid_fa_sorting FOREIGN KEY (entity_id, form_id, id) REFERENCES attributesorting (entity_id, attribute_form_id, attribute_id) \
              DEFERRABLE INITIALLY DEFERRED \
            ) ", file_form_id, file_form_id).as_str(), false, false)?;
        self.db_action(
            transaction,
            "create index file_parent_id on FileAttribute (entity_id)",
            false,
            false,
        )?;
        self.db_action(
            transaction,
            "CREATE TRIGGER fa_attribute_sorting_cleanup BEFORE DELETE ON FileAttribute \
            FOR EACH ROW EXECUTE PROCEDURE attribute_sorting_cleanup()",
            false,
            false,
        )?;
        // about oids and large objects, blobs: here are some reference links (but consider also which version of postgresql is running):
        //  https://duckduckgo.com/?q=postgresql+large+binary+streams
        //  http://www.postgresql.org/docs/9.1/interactive/largeobjects.html
        //  https://wiki.postgresql.org/wiki/BinaryFilesInDB
        //  http://jdbc.postgresql.org/documentation/80/binary-data.html
        //  http://artofsystems.blogspot.com/2008/07/mysql-postgresql-and-blob-streaming.html
        //  http://stackoverflow.com/questions/2069541/postgresql-jdbc-and-streaming-blobs
        //  http://giswiki.hsr.ch/PostgreSQL_-_Binary_Large_Objects
        self.db_action(transaction, "CREATE TABLE FileAttributeContent (\
            file_attribute_id bigint PRIMARY KEY, \
            contents_oid lo NOT NULL, \
            CONSTRAINT valid_fileattr_id FOREIGN KEY (file_attribute_id) REFERENCES fileattribute (id) ON DELETE CASCADE \
            )", false, false)?;
        // This trigger exists because otherwise the binary data from large objects doesn't get cleaned up when the related rows are deleted. For details
        // see the links just above (especially the wiki one).
        // (The reason I PUT THE "UPDATE OR" in the "BEFORE UPDATE OR DELETE" is simply: that is how this page's example (at least as of 2016-06-01:
        //    http://www.postgresql.org/docs/current/static/lo.html
        // ...said to do it.
        //Idea: but we still might want more tests around it? and to use "vacuumlo" module, per that same url?
        self.db_action(transaction, "CREATE TRIGGER om_contents_oid_cleanup BEFORE UPDATE OR DELETE ON fileattributecontent \
            FOR EACH ROW EXECUTE PROCEDURE lo_manage(contents_oid)", false, false)?;

        self.db_action(
            transaction,
            format!(
                "create sequence TextAttributeKeySequence minvalue {}",
                self.min_id_value()
            )
            .as_str(),
            false,
            false,
        )?;
        // the entity_id is the key for the entity on which this text info is recorded; for other meanings see comments on
        // Entity.addQuantityAttribute(...).
        // id must be "unique not null" in ANY database used, because it is the primary key.
        let text_form_id = self.get_attribute_form_id(Util::TEXT_TYPE).unwrap();
        // See comment for column "form_id" under "create table RelationToGroup", below.
        // For attr_type_id:  eg, serial number (which would be an entity).
        // For valid_on_date, see "create table RelationToEntity" for comments about dates' meanings.
        self.db_action(transaction, format!("create table TextAttribute (\
            form_id smallint DEFAULT {} \
                NOT NULL CHECK (form_id={}), \
            id bigint DEFAULT nextval('TextAttributeKeySequence') PRIMARY KEY, \
            entity_id bigint NOT NULL, \
            textvalue text NOT NULL, \
            attr_type_id bigint not null, \
            valid_on_date bigint, \
            observation_date bigint not null, \
            CONSTRAINT valid_attr_type_id FOREIGN KEY (attr_type_id) REFERENCES entity (id), \
            CONSTRAINT valid_parent_id FOREIGN KEY (entity_id) REFERENCES entity (id) ON DELETE CASCADE, \
            CONSTRAINT valid_ta_sorting FOREIGN KEY (entity_id, form_id, id) REFERENCES attributesorting (entity_id, attribute_form_id, attribute_id) \
              DEFERRABLE INITIALLY DEFERRED \
            ) ", text_form_id, text_form_id).as_str(), false, false)?;
        self.db_action(
            transaction,
            "create index text_parent_id on TextAttribute (entity_id)",
            false,
            false,
        )?;
        self.db_action(
            transaction,
            "CREATE TRIGGER ta_attribute_sorting_cleanup BEFORE DELETE ON TextAttribute \
            FOR EACH ROW EXECUTE PROCEDURE attribute_sorting_cleanup()",
            false,
            false,
        )?;

        self.db_action(
            transaction,
            format!(
                "create sequence RelationToEntityKeySequence minvalue {}",
                self.min_id_value()
            )
            .as_str(),
            false,
            false,
        )?;
        //Example: a relationship between a state and various counties might be set up like this:
        // The state and each county are Entities. A RelationType (which is an Entity with some
        // additional columns) is bi- directional and indicates some kind of containment relationship, for example between
        // state & counties. In the RelationToEntity table there would be a row whose rel_type_id points to the described RelationType,
        // whose entity_id points to the state Entity, and whose entity_id_2 points to a given county Entity. There would be
        // additional rows for each county, varying only in the value in entity_id_2.
        // And example of something non(?)directional would be where the relationship is identical no matter which way you go, like
        // two human acquaintances). The relationship between a state and county is not the same in reverse. Haven't got a good
        // unidirectional example, so maybe it can be eliminated? (Or maybe it would be something where the "child" doesn't "know"
        // the "parent"--like an electron in an atom? -- revu notes or see what Mark Butler thinks.
        // --Luke Call 8/2003.
        let rle_form_id = self
            .get_attribute_form_id(Util::RELATION_TO_LOCAL_ENTITY_TYPE)
            .unwrap();
        // See comment for form_id, under "create table RelationToGroup", below.
        // The "id" column can be treated like a primary key (with the advantages of being
        // artificial) but the real one is a bit farther down. This one has the
        // slight or irrelevant disadvantage that it artificially limits the # of rows in this table, but it's still a big #.
        // The rel_type_id column is for lookup in RelationType table, eg "has".
        // About the entity_id column: what is related (see RelationConnection for "related to what" (related_to_entity_id).
        // For entity_id_2: the entity_id in RelAttr table is related to what other entity(ies).
        // The valid on date can be null (means no info), or 0 (means 'for all time', not 1970 or whatever that was. At least make it a 1 in that case),
        // or the date it first became valid/true. (The java/scala version of it put in System.currentTimeMillis() for "now"%%--ck if it
        // behaves correctly now when saving/reading/displaying, in milliseconds...? like the call in create_base_data()
        // to create_relation_to_local_entity ?)
        // The observation_date is: whenever first observed (in milliseconds?).
        self.db_action(transaction, format!("create table RelationToEntity (\
            form_id smallint DEFAULT {} \
                NOT NULL CHECK (form_id={}), \
            id bigint DEFAULT nextval('RelationToEntityKeySequence') UNIQUE NOT NULL, \
            rel_type_id bigint NOT NULL, \
            entity_id bigint NOT NULL, \
            entity_id_2 bigint NOT NULL, \
            valid_on_date bigint, \
            observation_date bigint not null, \
            PRIMARY KEY (rel_type_id, entity_id, entity_id_2), \
            CONSTRAINT valid_rel_type_id FOREIGN KEY (rel_type_id) REFERENCES RelationType (entity_id) ON DELETE CASCADE, \
            CONSTRAINT valid_related_to_entity_id_1 FOREIGN KEY (entity_id) REFERENCES entity (id) ON DELETE CASCADE, \
            CONSTRAINT valid_related_to_entity_id_2 FOREIGN KEY (entity_id_2) REFERENCES entity (id) ON DELETE CASCADE, \
            CONSTRAINT valid_reltoent_sorting FOREIGN KEY (entity_id, form_id, id) REFERENCES attributesorting (entity_id, attribute_form_id, attribute_id) \
              DEFERRABLE INITIALLY DEFERRED \
            ) ", rle_form_id, rle_form_id).as_str(), false, false)?;
        self.db_action(
            transaction,
            "create index entity_id_1 on RelationToEntity (entity_id)",
            false,
            false,
        )?;
        self.db_action(
            transaction,
            "create index entity_id_2 on RelationToEntity (entity_id_2)",
            false,
            false,
        )?;
        self.db_action(
            transaction,
            "CREATE TRIGGER rte_attribute_sorting_cleanup BEFORE DELETE ON RelationToEntity \
            FOR EACH ROW EXECUTE PROCEDURE attribute_sorting_cleanup()",
            false,
            false,
        )?;

        // Would rename this sequence to match the table it's used in now, but the cmd "alter sequence relationtogroupkeysequence rename to groupkeysequence;"
        // doesn't rename the name inside the sequence, and keeping the old name is easier for now than deciding whether to do something about that (more info
        // if you search the WWW for "postgresql bug 3619".
        self.db_action(
            transaction,
            format!(
                "create sequence RelationToGroupKeySequence minvalue {}",
                self.min_id_value()
            )
            .as_str(),
            false,
            false,
        )?;
        // This table is named "grupo" because otherwise some queries (like "drop table group") don't work unless "group" is quoted, which doesn't work
        // with mixed case; but forcing the dropped names to lowercase and quoted also prevented dropping class and entity in the same command, it seemed.
        // Avoiding the word "group" as a table in sql might prevent other errors too.
        // Insertion_date is intended to be a readonly date: the (*java*-style numeric: milliseconds
        // since 1970-1-1 or such) when this row was inserted (ie, when the object was created
        // in the db).
        // For new_entries... see comment at same field in Entity table.
        self.db_action(
            transaction,
            format!(
                "create table grupo (\
            id bigint DEFAULT nextval('RelationToGroupKeySequence') PRIMARY KEY, \
            name varchar({}) NOT NULL, \
            insertion_date bigint not null, \
            allow_mixed_classes boolean NOT NULL, \
            new_entries_stick_to_top boolean NOT NULL  default false\
            ) ",
                Util::entity_name_length()
            )
            .as_str(),
            false,
            false,
        )?;

        self.db_action(
            transaction,
            format!(
                "create sequence RelationToGroupKeySequence2 minvalue {}",
                self.min_id_value()
            )
            .as_str(),
            false,
            false,
        )?;
        // The form_id is always the same, and exists to enable the integrity constraint which references it, just below.
        // The id column can be treated like a primary key (with the advantages of being artificial)
        // but the real one is a bit farther down. This one has the slight or irrelevant
        // disadvantage that it artificially limits the # of rows in this table, but it's still a big #.
        // The entity_id is of the containing entity whose attribute (subgroup, RTG) this is.
        // Idea: Should the 2 dates be eliminated? The code is there, including in the parent class, and they might be useful,
        // maybe no harm while we wait & see.
        // See "create table RelationToEntity" for comments about dates' meanings.
        let rtg_form_id = self
            .get_attribute_form_id(Util::RELATION_TO_GROUP_TYPE)
            .unwrap();
        self.db_action(transaction, format!("create table RelationToGroup (\
            form_id smallint DEFAULT {} \
                NOT NULL CHECK (form_id={}), \
            id bigint DEFAULT nextval('RelationToGroupKeySequence2') UNIQUE NOT NULL, \
            entity_id bigint NOT NULL, \
            rel_type_id bigint NOT NULL, \
            group_id bigint NOT NULL, \
            valid_on_date bigint, \
            observation_date bigint not null, \
            PRIMARY KEY (entity_id, rel_type_id, group_id), \
            CONSTRAINT valid_reltogrp_entity_id FOREIGN KEY (entity_id) REFERENCES entity (id) ON DELETE CASCADE, \
            CONSTRAINT valid_reltogrp_rel_type_id FOREIGN KEY (rel_type_id) REFERENCES relationType (entity_id), \
            CONSTRAINT valid_reltogrp_group_id FOREIGN KEY (group_id) REFERENCES grupo (id) ON DELETE CASCADE, \
            CONSTRAINT valid_reltogrp_sorting FOREIGN KEY (entity_id, form_id, id) REFERENCES attributesorting (entity_id, attribute_form_id, attribute_id) \
              DEFERRABLE INITIALLY DEFERRED \
            ) ", rtg_form_id, rtg_form_id).as_str(), false, false)?;
        self.db_action(
            transaction,
            "create index RTG_entity_id on RelationToGroup (entity_id)",
            false,
            false,
        )?;
        self.db_action(
            transaction,
            "create index RTG_group_id on RelationToGroup (group_id)",
            false,
            false,
        )?;
        self.db_action(
            transaction,
            "CREATE TRIGGER rtg_attribute_sorting_cleanup BEFORE DELETE ON RelationToGroup \
            FOR EACH ROW EXECUTE PROCEDURE attribute_sorting_cleanup()",
            false,
            false,
        )?;

        /* This table maintains a 1-to-many connection between one entity, and many others in a particular group that it contains.
        Will this clarify terms?: the table below is a (1) "relationship table" (aka relationship entity--not an OM entity but at a lower layer) which tracks
        those entities which are part of a particular group.  The nature of the (2) "relation"-ship between that group of entities and the entity that "has"
        them (or other relationtype to them...) is described by the table RelationToGroup, which is instead of a regular old (3) "RelationToEntity" because #3
        just
        relates Entities to other Entities.  Or in other words, #2 (RelationToGroup) has notes about the tie from Entities to groups of Entities,
        where the specific entities in that group are listed in #1 (this table below).  And the type of relation between them (has, contains,
        is acquainted with...?) is in the 4) relationtogroup table's reference to the relationtype table (or its "rel_type_id"). Got it?
        (Good, then let's not confuse things by mentioning that postgresql refers to *every* table (and more?) as a "relation" because that's another
        context altogether, another use of the word.)
        */
        // The primary key is really the group_id + entity_id, and the sorting_index is just in an index so we can cheaply order query results.
        // When sorting_index was part of the key there were ongoing various problems because the rest of the system (like reordering results, but
        // probably also other issues) wasn't ready to handle two of the same entity in a group.
        // The onstraint noDupSortingIndexes is to make it so the sorting_index must also be unique for each group (otherwise we have sorting problems).
        self.db_action(transaction, "create table EntitiesInAGroup (\
            group_id bigint NOT NULL\
            , entity_id bigint NOT NULL\
            , sorting_index bigint not null\
            , PRIMARY KEY (group_id, entity_id)\
            , CONSTRAINT valid_group_id FOREIGN KEY (group_id) REFERENCES grupo (id) ON DELETE CASCADE\
            , CONSTRAINT valid_entity_id FOREIGN KEY (entity_id) REFERENCES entity (id)\
            , constraint noDupSortingIndexes unique (group_id, sorting_index)\
            ) ", false, false)?;
        self.db_action(
            transaction,
            "create index EntitiesInAGroup_id on EntitiesInAGroup (entity_id)",
            false,
            false,
        )?;
        self.db_action(transaction, "create index EntitiesInAGroup_sorted on EntitiesInAGroup (group_id, entity_id, sorting_index)", false, false)?;

        self.db_action(
            transaction,
            format!(
                "create sequence ActionKeySequence minvalue {}",
                self.min_id_value()
            )
            .as_str(),
            false,
            false,
        )?;
        self.db_action(transaction, format!("create table Action (\
            id bigint DEFAULT nextval('ActionKeySequence') PRIMARY KEY, \
            class_id bigint NOT NULL, \
            name varchar({}) NOT NULL, \
            action varchar({}) NOT NULL, \
            CONSTRAINT valid_related_to_class_id FOREIGN KEY (class_id) REFERENCES Class (id) ON DELETE CASCADE \
            ) ", Util::entity_name_length(), Util::entity_name_length()).as_str(), false, false)?;
        self.db_action(
            transaction,
            "create index action_class_id on Action (class_id)",
            false,
            false,
        )?;

        /* This current database is one OM instance, and known (remote or local) databases to which this one might refer are other instances.
          Design musings:
        This is being implemented in an explicit table instead of just with the features around EntityClass objects & the "class" table, to
        avoid a chicken/egg problem:
        imagine a new OM instance, or one where the user deleted via the UI the relevant entity class(es) for handling remote OM instances: how would the user
        retrieve those classes from others' shared OM data if the feature to connect to remote ones is broken?  Still, it is debatable whether it would have
        worked just as well to put this info in an entity under the .system entity, like user preferences are, and try to prevent deleting it or something,
        because other info might be needed on it in the future such as security settings, and using the entity_id field for links to that info could become
        just as awkward as having an entity to begin with.  But doing it the way it is now might make db-level constraints on such things
        more reliable, especially given that the OM-level constraints via classes/code on entities isn't developed yet.

        This might have some design overlap with the ".system" entity; maybe that should have been put here?
         */
        // The "local" field doesn't mean whether the instance is found on localhost, but rather whether the row is for *this* instance: the OneModel
        // instance whose database we are connected to right now.
        // See Controller.askForAndWriteOmInstanceInfo.askAndSave for more description for the address column.
        // Idea: Is it worth having to know future address formats, to enforce validity in a
        // constraint?  Problems seem likely to be infrequent & easy to fix.
        // See table "entity" for description of insertion_date.
        // The entity_id is to link to an entity with whatever details, such as a human-given
        // name for familiarity, security settings, other adhoc info, etc. NULL values are
        // intentionally allowed, in case user doesn't need to specify any extra info about an omInstance.
        // Idea: require a certain class for this entity, created at startup/db initialization? or a shared one? Waiting until use cases become clearer.
        self.db_action(
            transaction,
            format!(
                "create table OmInstance (\
            id uuid PRIMARY KEY\
            , local boolean NOT NULL\
            , address varchar({}) NOT NULL\
            , insertion_date bigint not null\
            , entity_id bigint REFERENCES entity (id) ON DELETE RESTRICT\
            ) ",
                self.om_instance_address_length()
            )
            .as_str(),
            false,
            false,
        )?;

        self.db_action(
            transaction,
            format!(
                "create sequence RelationToRemoteEntityKeySequence minvalue {}",
                self.min_id_value()
            )
            .as_str(),
            false,
            false,
        )?;
        // See comments on "create table RelationToEntity" above for comparison & some info, as well as class comments on RelationToRemoteEntity.
        // The difference here is (at least that) this has a field pointing
        // to a remote OM instance.  The Entity with id entity_id_2 is contained in that remote OM instance, not in the current one.
        // (About remote_instance_id: see comment just above.)
        // (See comment above about entity_id_2.)
        // About constraint valid_remote_instance_id below:
        // deletions of the referenced rows should warn the user that these will be deleted also.  The same should also be true for all
        // other uses of "ON DELETE CASCADE".
        let rtre_form_id = self
            .get_attribute_form_id(Util::RELATION_TO_REMOTE_ENTITY_TYPE)
            .unwrap();
        self.db_action(transaction, format! ("create table RelationToRemoteEntity (\
            form_id smallint DEFAULT {} \
                NOT NULL CHECK (form_id={}), \
            id bigint DEFAULT nextval('RelationToRemoteEntityKeySequence') UNIQUE NOT NULL, \
            rel_type_id bigint NOT NULL, \
            entity_id bigint NOT NULL, \
            remote_instance_id uuid NOT NULL, \
            entity_id_2 bigint NOT NULL, \
            valid_on_date bigint, \
            observation_date bigint not null, \
            PRIMARY KEY (rel_type_id, entity_id, remote_instance_id, entity_id_2), \
            CONSTRAINT valid_rel_to_local_type_id FOREIGN KEY (rel_type_id) REFERENCES RelationType (entity_id) ON DELETE CASCADE, \
            CONSTRAINT valid_rel_to_local_entity_id_1 FOREIGN KEY (entity_id) REFERENCES entity (id) ON DELETE CASCADE, \
            CONSTRAINT valid_remote_instance_id FOREIGN KEY (remote_instance_id) REFERENCES OmInstance (id) ON DELETE CASCADE, \
            CONSTRAINT remote_sorting FOREIGN KEY (entity_id, form_id, id) REFERENCES attributesorting (entity_id, attribute_form_id, attribute_id) \
              DEFERRABLE INITIALLY DEFERRED \
            ) ", rtre_form_id, rtre_form_id).as_str(), false, false)?;
        self.db_action(
            transaction,
            "create index rtre_entity_id_1 on RelationToRemoteEntity (entity_id)",
            false,
            false,
        )?;
        self.db_action(
            transaction,
            "create index rtre_entity_id_2 on RelationToRemoteEntity (entity_id_2)",
            false,
            false,
        )?;
        self.db_action(transaction, "CREATE TRIGGER rtre_attribute_sorting_cleanup BEFORE DELETE ON RelationToRemoteEntity \
            FOR EACH ROW EXECUTE PROCEDURE attribute_sorting_cleanup()", false, false)?;

        self.db_action(
            transaction,
            format!(
                "UPDATE om_db_version SET (version) = ROW({})",
                PostgreSQLDatabase::SCHEMA_VERSION
            )
            .as_str(),
            false,
            false,
        )?;

        Ok(())
    }

    fn create_attribute_sorting_deletion_trigger(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
    ) -> Result<u64, anyhow::Error> {
        // Each time an attribute (or rte/rtg) is deleted, the AttributeSorting row should be deleted too, in an enforced way (or it had sorting problems, for one).
        // I.e., an attempt to enforce (with triggers that call this procedure) that the AttributeSorting table's attribute_id value is found
        // in *one of the* 7 attribute tables' id column,  Doing it in application code is not as simple or as reliable as doing it at the DDL level.
        // (OLD is a special PL/pgsql variable of type RECORD, which contains the attribute row before the deletion.)
        let sql = "CREATE OR REPLACE FUNCTION attribute_sorting_cleanup() RETURNS trigger AS $attribute_sorting_cleanup$ \
          BEGIN \
                DELETE FROM AttributeSorting WHERE entity_id=OLD.entity_id and attribute_form_id=OLD.form_id and attribute_id=OLD.id; \
                RETURN OLD; \
              END;\
            $attribute_sorting_cleanup$ LANGUAGE plpgsql;";
        self.db_action(transaction, sql, false, true)
    }

    /// Creates data that must exist in a base system, and which is not re-created in an existing system.  If this data is deleted, the system might not work.
    fn create_base_data<'a>(
        &'a self,
        transaction: &Option<&mut Transaction<'a, Postgres>>,
    ) -> Result<(), anyhow::Error> {
        // idea: what tests are best, around this, vs. simply being careful in upgrade scripts?
        let ids: Vec<i64> =
            self.find_entity_only_ids_by_name(transaction, Util::SYSTEM_ENTITY_NAME.to_string())?;
        // will probably have to change the next line when things grow/change, and, maybe, we're doing upgrades not always a new system:
        assert!(ids.is_empty());

        // public=false, guessing at best value, since the world wants your modeled info, not
        // details about your system internals (which might be...unique & personal somehow)?:
        let system_entity_id =
            self.create_entity(transaction, Util::SYSTEM_ENTITY_NAME, None, Some(false))?;

        let existence_entity_id =
            self.create_entity(transaction, "existence", None, Some(false))?;
        //idea: as probably mentioned elsewhere, this "BI" (and other strings?) should be replaced with a constant somewhere (or enum?)!
        debug!("in create_base_data: after creating entity 'existence'.");
        let has_rel_type_id = self.create_relation_type(
            true,
            transaction,
            Util::THE_HAS_RELATION_TYPE_NAME,
            Util::THE_IS_HAD_BY_REVERSE_NAME,
            "BI",
        )?;
        debug!("in create_base_data: after creating relType 'has'.");
        //%%does this save/retrieve (comparing new data w/ this change, and old data from scala) accurately w/ what we want?:
        let current_time_millis = Utc::now().timestamp_millis();
        debug!("in create_base_data: after creating relType 'has'. 2");
        self.create_relation_to_local_entity(
            transaction,
            has_rel_type_id,
            system_entity_id,
            existence_entity_id,
            Some(current_time_millis),
            current_time_millis,
            None,
            true,
        )?;
        debug!("in create_base_data: after creating rte.");

        let editor_info_entity_id = self.create_entity(
            transaction,
            Util::EDITOR_INFO_ENTITY_NAME,
            None,
            Some(false),
        )?;
        debug!("in create_base_data: after creating entity 'editorinfo'.");
        self.create_relation_to_local_entity(
            transaction,
            has_rel_type_id,
            system_entity_id,
            editor_info_entity_id,
            Some(current_time_millis),
            current_time_millis,
            None,
            true,
        )?;
        let text_editor_info_entity_id = self.create_entity(
            transaction,
            Util::TEXT_EDITOR_INFO_ENTITY_NAME,
            None,
            Some(false),
        )?;
        self.create_relation_to_local_entity(
            transaction,
            has_rel_type_id,
            editor_info_entity_id,
            text_editor_info_entity_id,
            Some(current_time_millis),
            current_time_millis,
            None,
            true,
        )?;
        let text_editor_command_attribute_type_id = self.create_entity(
            transaction,
            Util::TEXT_EDITOR_COMMAND_ATTRIBUTE_TYPE_NAME,
            None,
            Some(false),
        )?;
        self.create_relation_to_local_entity(
            transaction,
            has_rel_type_id,
            text_editor_info_entity_id,
            text_editor_command_attribute_type_id,
            Some(current_time_millis),
            current_time_millis,
            None,
            true,
        )?;
        let editor_command: &str = {
            if Util::is_windows() {
                "notepad"
            } else {
                "vi"
            }
        };
        self.create_text_attribute(
            transaction,
            text_editor_info_entity_id,
            text_editor_command_attribute_type_id,
            editor_command,
            Some(current_time_millis),
            current_time_millis,
            true,
            None,
        )?;

        // the intent of this group is user convenience: the app shouldn't rely on this group to find classDefiningEntities (templates), but use the relevant table.
        // idea: REALLY, this should probably be replaced with a query to the class table: so, when queries as menu options are part of the OM
        // features, put them all there instead.
        // It is set to allowMixedClassesInGroup just because no current known reason not to; will be interesting to see what comes of it.
        self.create_group_and_relation_to_group(
            transaction,
            system_entity_id,
            has_rel_type_id,
            Util::CLASS_TEMPLATE_ENTITY_GROUP_NAME,
            /*%%allow_mixed_classes_in_group_in =*/ true,
            Some(current_time_millis),
            current_time_millis,
            None,
            true,
        )?;

        // NOTICE: code should not rely on this name, but on data in the tables.
        /*val (class_id, entity_id) = */
        self.create_class_and_its_template_entity(transaction, "person".to_string())?;
        // (should be same as the line in upgradeDbFrom3to4(), or when combined with later such methods, .)
        let uuid = uuid::Uuid::new_v4();
        debug!("in create_base_data: bytes: {:?}", uuid.as_bytes());
        debug!("in create_base_data: simple: {}", uuid.simple());
        debug!("in create_base_data: hyphenated: {}", uuid.hyphenated());
        debug!("in create_base_data: urn: {}", uuid.urn());
        debug!("in create_base_data: tostring: {}", uuid.to_string());
        self.create_om_instance(
            transaction,
            /*%%which from above!?*?*/ uuid.to_string(),
            true,
            Util::LOCAL_OM_INSTANCE_DEFAULT_DESCRIPTION.to_string(),
            None,
            false,
        )?;
        Ok(())
    }

    /// Case-insensitive.
    fn find_entity_only_ids_by_name(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        name_in: String,
    ) -> Result<Vec<i64>, anyhow::Error> {
        // idea: see if queries like this are using the expected index (run & ck the query plan). Tests around that, for benefit of future dbs? Or, just wait for
        // a performance issue then look at it?
        let include_archived: &str = if !self.include_archived_entities() {
            "(not archived) and "
        } else {
            ""
        };
        let the_rest = format!(
            "lower(name) = lower('{}') {} ",
            name_in,
            Self::limit_to_entities_only(Self::ENTITY_ONLY_SELECT_PART)
        );
        let rows: Vec<Vec<Option<DataType>>> = self.db_query(
            transaction,
            format!(
                "select id from entity where {}{}",
                include_archived, the_rest
            )
            .as_str(),
            "i64",
        )?;
        // if rows.isEmpty None
        // else {
        let mut results: Vec<i64> = Vec::new();
        for row in rows.iter() {
            // results = row(0).get.asInstanceOf[i64] :: results
            let id = match row[0] {
                Some(DataType::Bigint(x)) => x,
                // next line is intended to be impossible, based on the query
                _ => {
                    return Err(anyhow!(
                        "In find_entity_only_ids_by_name, this should never happen: {:?}.",
                        row[0]
                    ))
                }
            };
            results.push(id);
        }
        results.reverse();
        Ok(results)
        // }
    }

    /** Returns the class_id and entity_id, in a tuple. */
    fn create_class_and_its_template_entity2<'a>(
        &'a self,
        transaction_in: &Option<&mut Transaction<'a, Postgres>>,
        class_name_in: String,
        entity_name_in: String,
        // (See fn delete_objects for more about this parameter, and transaction above.)
        caller_manages_transactions_in: bool, /*%%= false*/
    ) -> Result<(i64, i64), anyhow::Error> {
        //BEGIN COPY/PASTED/DUPLICATED (except "in <fn_name>" below) BLOCK-----------------------------------
        // Try creating a local transaction whether we use it or not, to handle compiler errors
        // about variable moves. I'm not seeing a better way to get around them by just using
        // conditions and an Option (many errors):
        // (I tried putting this in a function, then a macro, but it gets compile errors.
        // So, copy/pasting this, unfortunately, until someone thinks of a better way. (You
        // can see the macro, and one of the compile errors, in the commit of 2023-05-18.
        // I didn't try a proc macro but based on some reading I think it would have the same
        // problem.)
        let mut local_tx: Transaction<Postgres> = {
            if transaction_in.is_none() {
                if caller_manages_transactions_in {
                    return Err(anyhow!("In create_class_and_its_template_entity2, inconsistent values for caller_manages_transactions_in \
                                and transaction_in: true and None??"
                    .to_string()));
                } else {
                    self.begin_trans()?
                }
            } else {
                if caller_manages_transactions_in {
                    // That means we have determined that the caller is to use the transaction_in .
                    // was just:  None
                    // But now instead, create it anyway, per comment above.
                    self.begin_trans()?
                } else {
                    return Err(anyhow!(
                        "In create_class_and_its_template_entity2, inconsistent values for caller_manages_transactions_in & transaction_in: \
                                false and Some??"
                            .to_string(),
                    ));
                }
            }
        };
        let local_tx_option = &Some(&mut local_tx);
        let transaction: &Option<&mut Transaction<Postgres>> = if caller_manages_transactions_in {
            transaction_in
        } else {
            local_tx_option
        };
        //END OF COPY/PASTED/DUPLICATED BLOCK----------------------------------

        // The name doesn't have to be the same on the entity and the template class, but why not for now.
        let class_name: String = Self::escape_quotes_etc(class_name_in);
        let entity_name: String = Self::escape_quotes_etc(entity_name_in);
        if class_name.len() == 0 {
            return Err(anyhow!(
                "In create_class_and_its_template_entity2, Class name must have a value."
                    .to_string()
            ));
        }
        if entity_name.len() == 0 {
            return Err(anyhow!(
                "In create_class_and_its_template_entity2, Entity name must have a value."
                    .to_string()
            ));
        }
        let class_id: i64 = self.get_new_key(transaction, "ClassKeySequence")?;
        let entity_id: i64 = self.get_new_key(transaction, "EntityKeySequence")?;
        // Start the entity w/ a NULL class_id so that it can be inserted w/o the class present, then update it afterward; constraints complain otherwise.
        // Idea: instead of doing in 3 steps, could specify 'deferred' on the 'not null'
        // constraint?: (see file:///usr/share/doc/postgresql-doc-9.1/html/sql-createtable.html).
        self.db_action(
            transaction,
            format!(
                "INSERT INTO Entity (id, insertion_date, name, class_id) VALUES ({},{},'{}', NULL)",
                entity_id,
                Utc::now().timestamp_millis(),
                entity_name
            )
            .as_str(),
            false,
            false,
        )?;
        self.db_action(
            transaction,
            format!(
                "INSERT INTO Class (id, name, defining_entity_id) VALUES ({},'{}', {})",
                class_id, class_name, entity_id
            )
            .as_str(),
            false,
            false,
        )?;
        self.db_action(
            transaction,
            format!(
                "update Entity set (class_id) = ROW({}) where id={}",
                class_id, entity_id
            )
            .as_str(),
            false,
            false,
        )?;
        let class_group_id: Option<i64> = self.get_system_entitys_class_group_id(transaction)?;
        if class_group_id.is_some() {
            self.add_entity_to_group(
                transaction,
                class_group_id.unwrap(),
                entity_id,
                None,
                caller_manages_transactions_in,
            )?;
        }

        //%%put this & similar places into a function like self.commit_or_err(tx)?;   ?  If so, include the rollback cmt from just above?
        if !caller_manages_transactions_in {
            // Using local_tx to make the compiler happy and because it is the one we need,
            // if !caller_manages_transactions_in. Ie, there is no transaction provided by
            // the caller.
            if let Err(e) = self.commit_trans(local_tx) {
                return Err(anyhow!(e.to_string()));
            }
        }

        Ok((class_id, entity_id))
    }

    /// Returns the id of a specific group under the system entity.  This group is the one that contains class-defining (template) entities.
    fn get_system_entitys_class_group_id(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
    ) -> Result<Option<i64>, anyhow::Error> {
        let system_entity_id: i64 = self.get_system_entity_id(transaction)?;

        // idea: maybe this stuff would be less breakable by the user if we put this kind of info in some system table
        // instead of in this group. (See also method create_base_data).  Or maybe it doesn't matter, since it's just a user convenience. Hmm.
        let class_template_group_id = self
            .find_relation_to_and_group_on_entity(
                transaction,
                system_entity_id,
                Some(Util::CLASS_TEMPLATE_ENTITY_GROUP_NAME.to_string()),
            )?
            .2;
        if class_template_group_id.is_none() {
            // no exception thrown here because really this group is a convenience for the user to see things, not a requirement. Maybe a user message would be best:
            // "Idea:: BAD SMELL! The UI should do all UI communication, no?"  Maybe, pass in a UI object instead and call some generic method that will handle
            // the info properly?  Or have logs?
            // (SEE ALSO comments and code at other places with the part on previous line in quotes).
            eprintln!(
                "Unable to find, from the entity {}({}), any connection to its \
            expected contained group {}.  If it was deleted, it could be replaced if you want the \
            convenience of finding template entities in it.",
                Util::SYSTEM_ENTITY_NAME,
                system_entity_id,
                Util::CLASS_TEMPLATE_ENTITY_GROUP_NAME,
            );
        }
        Ok(class_template_group_id)
    }

    /// Although the next sequence value would be set automatically as the default for a column (at least the
    /// way I have them defined so far in postgresql); we do it explicitly
    /// so we know what sequence value to return, and what the unique key is of the row we just created!
    fn get_new_key(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        sequence_name_in: &str,
    ) -> Result<i64, anyhow::Error> {
        let row: Vec<Option<DataType>> = self.db_query_wrapper_for_one_row(
            transaction,
            format!("SELECT nextval('{}')", sequence_name_in).as_str(),
            "i64",
        )?;
        if row.is_empty() {
            return Err(anyhow!(
                "In get_new_key, No elements found, in get_new_key().".to_string()
            ));
        } else {
            match row[0] {
                // None => return Err("None found, in get_new_key()."),
                Some(DataType::Bigint(new_id)) => Ok(new_id),
                // DataType::Bigint(new_id) => Ok(new_id),
                _ => {
                    return Err(anyhow!(
                        "In get_new_key() this should never happen".to_string()
                    ))
                }
            }
        }
    }

    fn are_mixed_classes_allowed(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        group_id: &i64,
    ) -> Result<bool, anyhow::Error> {
        let rows: Vec<Vec<Option<DataType>>> = self.db_query(
            transaction,
            format!(
                "select allow_mixed_classes from grupo where id ={}",
                group_id
            )
            .as_str(),
            "bool",
        )?;
        let mixed_classes_allowed: bool = match rows[0][0] {
            Some(DataType::Boolean(b)) => b,
            _ => {
                return Err(anyhow!(
                    "In are_mixed_classes_allowed, this should never happen".to_string()
                ))
            }
        };
        Ok(mixed_classes_allowed)
    }

    /* tmp/example from ~l 1080
           let mut results: Vec<i64> = Vec::new();
           for row in rows.iter() {
               // results = row(0).get.asInstanceOf[i64] :: results
               let id = match row[0] {
                   DataType::Bigint(x) => x,
                   // next line is intended to be impossible, based on the query
                   _ => return Err(anyhow!("In a tmp/example, This should never happen.")),
               };
               results.push(id);
           }
           Ok(results.reverse)
    */
    fn has_mixed_classes(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        group_id_in: &i64,
    ) -> Result<bool, anyhow::Error> {
        // Enforce that all entities in so-marked groups have the same class (or they all have no class; too bad).
        // (This could be removed or modified, but some user scripts attached to groups might (someday?) rely on their uniformity, so this
        // and the fact that you can have a group all of which don't have any class, is experimental.  This is optional, per
        // group.  I.e., trying it that way now to see whether it removes desired flexibility
        // at a cost higher than the benefit of uniformity for later user code operating on groups.  This might be better in a constraint,
        // but after trying for a while I hadn't made the syntax work right.

        // (Had to ask for them all and expect 1, instead of doing a count, because for some reason "select count(class_id) ... group by class_id" doesn't
        // group, and you get > 1 when I wanted just 1. This way it seems to work if I just check the # of rows returned.)
        let rows: Vec<Vec<Option<DataType>>> = self.db_query(
            transaction,
            format!(
                "select class_id from EntitiesInAGroup eiag, entity e \
            where eiag.entity_id=e.id and group_id={} and class_id is not null group by class_id",
                group_id_in
            )
            .as_str(),
            "i64",
        )?;
        let num_classes_in_group_entities = rows.len();
        // nulls don't show up in a count(class_id), so get those separately
        //%%but does it matter that we are not doing such a count(class_id)? have a test4this?
        let num_null_classes_in_group_entities = self.extract_row_count_from_count_query(
            transaction,
            format!(
                "select count(entity_id) from EntitiesInAGroup \
            eiag, entity e where eiag.entity_id=e.id and group_id={} and class_id is NULL ",
                group_id_in
            )
            .as_str(),
        )?;
        if num_classes_in_group_entities > 1
            || (num_classes_in_group_entities >= 1 && num_null_classes_in_group_entities > 0)
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn limit_to_entities_only(select_column_names: &str) -> String {
        // IN MAINTENANCE: compare to logic in method get_entities_used_as_attribute_types_sql, and related/similar logic near the top of
        // Controller.chooseOrCreateObject (if it is still there; as of
        // 2017-8-21 starts with "val (numObjectsAvailable: i64, showOnlyAttributeTypes: bool) = {".
        let mut sql: String = String::new();
        sql.push_str("except (");
        sql.push_str(select_column_names);
        sql.push_str(" from entity e, quantityattribute q where e.id=q.unit_id) ");
        sql.push_str("except (");
        sql.push_str(select_column_names);
        sql.push_str(" from entity e, quantityattribute q where e.id=q.attr_type_id) ");
        sql.push_str("except (");
        sql.push_str(select_column_names);
        sql.push_str(" from entity e, dateattribute t where e.id=t.attr_type_id) ");
        sql.push_str("except (");
        sql.push_str(select_column_names);
        sql.push_str(" from entity e, booleanattribute t where e.id=t.attr_type_id) ");
        sql.push_str("except (");
        sql.push_str(select_column_names);
        sql.push_str(" from entity e, fileattribute t where e.id=t.attr_type_id) ");
        sql.push_str("except (");
        sql.push_str(select_column_names);
        sql.push_str(" from entity e, textattribute t where e.id=t.attr_type_id) ");
        sql.push_str("except (");
        sql.push_str(select_column_names);
        sql.push_str(" from entity e, relationtype t where e.id=t.entity_id) ");
        sql
    }

    /// @param sorting_index_in is currently passed by callers with a default guess, not a guaranteed good value, so if it is in use, this ~tries to find a good one.
    ///                       An alternate approach could be to pass in a callback to code like in SortableEntriesMenu.placeEntryInPosition (or what it calls),
    ///                       which this can call if it thinks it
    ///                       is taking a long time to find a free value, to give the eventual caller chance to give up if needed.  Or just pass in a known
    ///                       good value or call the renumber_sorting_indexes method in SortableEntriesMenu.
    /// @return the sorting_index value that is actually used.
    fn add_attribute_sorting_row(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        entity_id_in: i64,
        attribute_form_id_in: i32,
        attribute_id_in: i64,
        sorting_index_in: Option<i64>, /*%% = None*/
    ) -> Result<i64, anyhow::Error> {
        // SEE COMMENTS IN SIMILAR METHOD: add_entity_to_group.  **AND DO MAINTENANCE. IN BOTH PLACES.
        // Should probably be called from inside a transaction (which isn't managed in this method, since all its current callers do it.)
        let sorting_index: i64 = {
            let index = {
                if sorting_index_in.is_some() {
                    sorting_index_in.unwrap()
                } else if self.get_attribute_count(transaction, entity_id_in, false)? == 0 {
                    // start with an increment off the min or max, so that later there is room to sort something before or after it, manually:
                    self.min_id_value() + 99999
                } else {
                    self.max_id_value() - 99999
                }
            };
            if self.is_attribute_sorting_index_in_use(transaction, entity_id_in, index)? {
                self.find_unused_attribute_sorting_index(transaction, entity_id_in, None)?
            } else {
                index
            }
        };
        self.db_action(transaction, format!("insert into AttributeSorting (entity_id, attribute_form_id, attribute_id, sorting_index) \
            values ({},{},{},{})", entity_id_in, attribute_form_id_in, attribute_id_in, sorting_index).as_str(),
                       false, false)?;
        Ok(sorting_index)
    }

    fn is_attribute_sorting_index_in_use(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        entity_id_in: i64,
        sorting_index_in: i64,
    ) -> Result<bool, anyhow::Error> {
        self.does_this_exist(
            transaction,
            format!(
                "SELECT count(1) from AttributeSorting where entity_id={} and sorting_index={}",
                entity_id_in, sorting_index_in
            )
            .as_str(),
            true,
        )
    }

    fn get_system_entity_id(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
    ) -> Result<i64, anyhow::Error> {
        let ids: Vec<i64> =
            self.find_entity_only_ids_by_name(transaction, Util::SYSTEM_ENTITY_NAME.to_string())?;
        if ids.is_empty() {
            return Err(anyhow!(format!(
                "No system entity id (named \"{}\") was \
                 found in the entity table.  Did a new data import fail partway through or \
                 something?",
                Util::SYSTEM_ENTITY_NAME
            )));
        }
        assert_eq!(ids.len(), 1);
        Ok(ids[0])
    }

    //%%$%%remove now? Or, can I find a way to do it in a fn again somehow? w/ online help?
    // fn confirm_which_transaction(
    //     &self,
    //     transaction_in: &Option<&mut Transaction<Postgres>>,
    //     caller_manages_transactions_in: bool,
    // ) -> Result<Option<Transaction<Postgres>>, anyhow::Error> {
    //     // Make sure we either have a good local_tx or good transaction_in, to use one correctly
    //     // further down.
    //     if transaction_in.is_none() {
    //         if caller_manages_transactions_in {
    //             Err("Inconsistent values for caller_manages_transactions_in \
    //             and transaction_in: true and None??"
    //                 .to_string())
    //         } else {
    //             let mut tx: Transaction<Postgres> = match self.begin_trans() {
    // //                 Err(e) => return Err(anyhow!e.to_string())),
    //                 Err(e) => return Err(anyhow!(e.to_string())),
    //                 Ok(t) => t,
    //             };
    //             Ok(Some(tx))
    //         }
    //     } else {
    //         if caller_manages_transactions_in {
    //             // that means we have determined that the caller is to use the transaction_in .
    //             Ok(None)
    //         } else {
    //             Err(
    //                 "Inconsistent values for caller_manages_transactions_in & transaction_in: \
    //             false and Some?? Not sure yet whether this happens, or if it should."
    //                     .to_string(),
    //             )
    //         }
    //     }
    // }

    // Cloned to archive_objects: CONSIDER UPDATING BOTH if updating one.  Returns the # of rows deleted.
    /// Unless the parameter rows_expected==-1, it will allow any # of rows to be deleted; otherwise if the # of rows is wrong it will abort tran & fail.
    fn delete_objects<'a>(
        &'a self,
        // The purpose of transaction_in is so that whenever a direct db call needs to be done in a
        // transaction, as opposed to just using the pool as Executor, it will be available.
        transaction_in: &Option<&mut Transaction<'a, Postgres>>,
        table_name_in: &str,
        where_clause_in: &str,
        rows_expected: u64, /*%%= 1*/
        // The purpose of transaction_in is for those times when this method does not know the
        // context in which it will be called: whether it should rollback itself on error
        // (automatically by creating a transaction and letting it go out of scope), or should allow
        // the caller only to manage that.
        caller_manages_transactions_in: bool, /*%%= false*/
    ) -> Result<u64, anyhow::Error> {
        //BEGIN COPY/PASTED/DUPLICATED (except "in <fn_name>" below) BLOCK-----------------------------------
        // Try creating a local transaction whether we use it or not, to handle compiler errors
        // about variable moves. I'm not seeing a better way to get around them by just using
        // conditions and an Option (many errors):
        // (I tried putting this in a function, then a macro, but it gets compile errors.
        // So, copy/pasting this, unfortunately, until someone thinks of a better way. (You
        // can see the macro, and one of the compile errors, in the commit of 2023-05-18.
        // I didn't try a proc macro but based on some reading I think it would have the same
        // problem.)
        let mut local_tx: Transaction<Postgres> = {
            if transaction_in.is_none() {
                if caller_manages_transactions_in {
                    return Err(anyhow!("In delete_objects, inconsistent values for caller_manages_transactions_in \
                                and transaction_in: true and None??"
                    .to_string()));
                } else {
                    self.begin_trans()?
                }
            } else {
                if caller_manages_transactions_in {
                    // That means we have determined that the caller is to use the transaction_in .
                    // was just:  None
                    // But now instead, create it anyway, per comment above.
                    self.begin_trans()?
                } else {
                    return Err(anyhow!(
                        "In delete_objects, inconsistent values for caller_manages_transactions_in & transaction_in: \
                                false and Some??"
                            .to_string(),
                    ));
                }
            }
        };
        let local_tx_option = &Some(&mut local_tx);
        let transaction: &Option<&mut Transaction<Postgres>> = if caller_manages_transactions_in {
            transaction_in
        } else {
            local_tx_option
        };
        //END OF COPY/PASTED/DUPLICATED BLOCK----------------------------------

        //idea: enhance this to also check & return the # of rows deleted, to the caller to just make sure? If so would have to let caller handle transactions.
        let sql = format!("DELETE FROM {} {}", table_name_in, where_clause_in);

        let rows_deleted = self.db_action(
            // match local_tx {
            //     //%%does this work? each arm when it should??
            //     Some(mut tx) => &Some(&mut tx),
            //     None => transaction_in,
            // },
            transaction,
            sql.as_str(),
            /*%%caller_checks_row_count_etc =*/ true,
            false,
        )?;
        if rows_expected > 0 && rows_deleted != rows_expected {
            // No need to explicitly roll back a locally created transaction aka tx, though we
            // definitely don't want to delete an unexpected # of rows,
            // because rollback is implicit whenever the transaction goes out of scope without a commit.
            // Caller should roll back (or fail to commit, same thing) in case of error.
            return Err(anyhow!(format!(
                "Delete command would have removed {} rows, but {} were expected! \
                Did not perform delete.  SQL is: \"{}\"",
                rows_deleted, rows_expected, sql
            )));
        } else {
            //%%put this & similar places into a function like self.commit_or_err(tx)?;   ?  If so, include the rollback cmt from just above?
            if !caller_manages_transactions_in {
                // Using local_tx to make the compiler happy and because it is the one we need,
                // if !caller_manages_transactions_in. Ie, there is no transaction provided by
                // the caller.
                if let Err(e) = self.commit_trans(local_tx) {
                    return Err(anyhow!(e.to_string()));
                }
            }
            Ok(rows_deleted)
        }
    }

    fn get_user_preference2<'a>(
        &'a self,
        transaction: &Option<&mut Transaction<'a, Postgres>>,
        preferences_container_id_in: i64,
        preference_name_in: &str,
        preference_type: &str,
    ) -> Result<Vec<DataType>, anyhow::Error> {
        // (Passing a smaller numeric parameter to find_contained_local_entity_ids for levels_remainingIn, so that in the (very rare) case where one does not
        // have a default entity set at the *top* level of the preferences under the system entity, and there are links there to entities with many links
        // to others, then it still won't take too long to traverse them all at startup when searching for the default entity.  But still allowing for
        // preferences to be nested up to that many levels (3 as of this writing).
        let mut set: HashSet<i64> = HashSet::new();
        let found_preferences: &mut HashSet<i64> = self.find_contained_local_entity_ids(
            transaction,
            &mut set,
            preferences_container_id_in,
            preference_name_in,
            3,
            true,
        )?;
        if found_preferences.len() == 0 {
            // let empty_vec: Vec<DataType> = Vec::new();
            // Ok(empty_vec)
            Ok(Vec::new())
        } else {
            if found_preferences.len() != 1 {
                let pref_container_entity_name =
                    match self.get_entity_name(transaction, preferences_container_id_in)? {
                        None => "(None)".to_string(),
                        Some(x) => x,
                    };
                return Err(anyhow!(format!("Under the entity \"{}\" ({}, possibly under {}), there \
                        are (eventually) more than one entity with the name \"{}\", so the program does not know which one to use for this.",
                                   pref_container_entity_name, preferences_container_id_in, Util::SYSTEM_ENTITY_NAME, preference_name_in)));
            }
            let mut preference_entity_id: i64 = 0;
            for x in found_preferences.iter() {
                // there is exactly one, as checked above
                preference_entity_id = *x;
            }
            let preference_entity = Entity::new2(Box::new(self), transaction, preference_entity_id);
            let relevant_attribute_rows: Vec<Vec<Option<DataType>>> = {
                if preference_type == Util::PREF_TYPE_BOOLEAN {
                    // (Using the preference_entity.get_id for attr_type_id, just for convenience since it seemed as good as any.  ALSO USED IN THE SAME WAY,
                    // IN setUserPreference METHOD CALL TO create_boolean_attribute!)
                    let sql2 = format!("select id, booleanValue from booleanattribute where entity_id={} and attr_type_id={}", preference_entity_id, preference_entity_id);
                    self.db_query(transaction, sql2.as_str(), "i64,bool")?
                } else if preference_type == Util::PREF_TYPE_ENTITY_ID {
                    let sql2 = format!("select rel_type_id, entity_id, entity_id_2 from relationtoentity where entity_id={}", preference_entity_id);
                    self.db_query(transaction, sql2.as_str(), "i64,i64,i64")?
                } else {
                    return Err(anyhow!(format!(
                        "Unexpected preference_type: {}",
                        preference_type
                    )));
                }
            };
            if relevant_attribute_rows.len() == 0 {
                // at this point we probably have a preference entity but not the expected attribute inside it that holds the actual useful information, so the
                // user needs to go delete the bad preference entity or re-create the attribute.
                // Idea: should there be a good way to *tell* them that, from here?
                // Or, just delete the bad preference (self-cleanup). If it was the public/private display toggle, its absence will cause errors (though it is a
                // very unlikely situation here), and it will be fixed on restarting the app (or starting another instance), via the create_and_check_expected_data
                // (or current equivalent?) method.
                self.delete_entity(transaction, preference_entity_id, false)?;
                Ok(Vec::new())
            } else {
                let attr_msg: String = if preference_type == Util::PREF_TYPE_BOOLEAN {
                    format!(
                        " BooleanAttributes with the relevant type ({},{}), ",
                        preference_name_in, preferences_container_id_in
                    )
                } else if preference_type == Util::PREF_TYPE_ENTITY_ID {
                    " RelationToEntity values ".to_string()
                } else {
                    return Err(anyhow!(format!(
                        "Unexpected preference_type: {}",
                        preference_type
                    )));
                };

                if relevant_attribute_rows.len() != 1 {
                    // ASSUMED it is 1, below!
                    // preference_entity.get_id()
                    let (pref_entity_name, id) = match preference_entity {
                        // Using 0 as a best-effort non-existent id (even though it does exists) because
                        // no better idea came to mind, at least for this error handling.
                        Err(e) => (format!("(Unknown/error: {})", e.to_string()), 0_i64),
                        Ok(mut entity) => (entity.get_name(transaction)?.clone(), entity.get_id()),
                    };
                    return Err(anyhow!(format!("Under the entity {} ({}), there are {}{}so the program does not know what to use for this.  There should be *one*.",
                                       pref_entity_name,
                                        id,
                                       relevant_attribute_rows.len(), attr_msg)));
                }
                if preference_type == Util::PREF_TYPE_BOOLEAN {
                    //PROVEN to have 1 row, just above!
                    // let DataType::Bigint(preference_id) = relevant_attribute_rows[0][0];
                    let preference_id: DataType/*i64*/ = match relevant_attribute_rows[0][0].clone() {
                        Some(x) => x.clone(),
                        None => return Err(anyhow!("In get_user_preference2, Did not expect null in retrieved column for preference_id value")),
                    };
                    // let DataType::Boolean(preference_value) = relevant_attribute_rows[0][1];
                    let preference_value: DataType/*bool*/ = match relevant_attribute_rows[0][1].clone() {
                        Some(x) => x.clone(),
                        None => return Err(anyhow!("In get_user_preference2, Did not expect null in retrieved column for boolean value")),
                    };
                    Ok(vec![preference_id, preference_value])
                } else if preference_type == Util::PREF_TYPE_ENTITY_ID {
                    //PROVEN to have 1 row, just above!
                    let rel_type_id: DataType/*i64*/ = match relevant_attribute_rows[0][0].clone() {
                        Some(x) => x.clone(),
                        None => return Err(anyhow!("In get_user_preference2, Did not expect null in retrieved column for rel_type_id value")),
                    };
                    let entity_id1: DataType/*i64*/ = match relevant_attribute_rows[0][1].clone() {
                        Some(x) => x.clone(),
                        None => return Err(anyhow!("In get_user_preference2, Did not expect null in retrieved column for entity_id1 value")),
                    };
                    let entity_id2: DataType/*i64*/ = match relevant_attribute_rows[0][2].clone() {
                        Some(x) => x.clone(),
                        None => return Err(anyhow!("In get_user_preference2, Did not expect null in retrieved column for entity_id2 value")),
                    };
                    Ok(vec![rel_type_id, entity_id1, entity_id2])
                } else {
                    return Err(anyhow!(
                        "In get_user_preference2, Unexpected preference_type: {}",
                        preference_type
                    ));
                }
            }
        }
    }

    fn get_relation_to_local_entity_by_name(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        containing_entity_id_in: i64,
        name_in: &str,
    ) -> Result<Option<i64>, anyhow::Error> {
        let if_not_archived = if !self.include_archived_entities {
            " and (not e.archived)"
        } else {
            ""
        };
        let sql = format!(
            "select rte.entity_id_2 from relationtoentity rte, entity e where \
            rte.entity_id={}{} and rte.entity_id_2=e.id and e.name='{}'",
            containing_entity_id_in, if_not_archived, name_in
        );
        let related_entity_id_rows = self.db_query(transaction, sql.as_str(), "i64")?;
        if related_entity_id_rows.len() == 0 {
            Ok(None)
        } else {
            if related_entity_id_rows.len() != 1 {
                let containing_entity_name =
                    match self.get_entity_name(transaction, containing_entity_id_in)? {
                        None => "(None)".to_string(),
                        Some(s) => s,
                    };
                return Err(anyhow!(format!("Under the entity {}({}), there is more one than entity with the name \"{}\", so the program does not know which one to use for this.",
                           containing_entity_name, containing_entity_id_in,
                    Util::USER_PREFERENCES)));
            }

            //idea: surely there is some better way than what I am doing here? See other places similarly.
            // let DataType::Bigint(id) = related_entity_id_rows[0][0];
            let id = match related_entity_id_rows[0][0] {
                Some(DataType::Bigint(x)) => x,
                _ => {
                    return Err(anyhow!(format!(
                        "How did we get here for {:?}?",
                        related_entity_id_rows[0][0]
                    )))
                }
            };
            Ok(Some(id))
        }
    }

    fn get_quantity_attribute_count(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        entity_id_in: i64,
    ) -> Result<u64, anyhow::Error> {
        self.extract_row_count_from_count_query(
            transaction,
            format!(
                "select count(1) from QuantityAttribute where entity_id={}",
                entity_id_in
            )
            .as_str(),
        )
    }

    fn get_text_attribute_count(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        entity_id_in: i64,
    ) -> Result<u64, anyhow::Error> {
        self.extract_row_count_from_count_query(
            transaction,
            format!(
                "select count(1) from TextAttribute where entity_id={}",
                entity_id_in
            )
            .as_str(),
        )
    }

    fn get_date_attribute_count(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        entity_id_in: i64,
    ) -> Result<u64, anyhow::Error> {
        self.extract_row_count_from_count_query(
            transaction,
            format!(
                "select count(1) from DateAttribute where entity_id={}",
                entity_id_in
            )
            .as_str(),
        )
    }

    fn get_boolean_attribute_count(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        entity_id_in: i64,
    ) -> Result<u64, anyhow::Error> {
        self.extract_row_count_from_count_query(
            transaction,
            format!(
                "select count(1) from BooleanAttribute where entity_id={}",
                entity_id_in
            )
            .as_str(),
        )
    }

    fn get_file_attribute_count(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        entity_id_in: i64,
    ) -> Result<u64, anyhow::Error> {
        self.extract_row_count_from_count_query(
            transaction,
            format!(
                "select count(1) from FileAttribute where entity_id={}",
                entity_id_in
            )
            .as_str(),
        )
    }
    /// Performs automatic database upgrades as required by evolving versions of OneModel.
    /// ******MAKE SURE*****:       ...that everything this does is also done in create_tables (and probably also
    /// the testing script integration/bin/purgue-om-test-database.psql) so that create_tables is a single reference
    /// point for a developer to go read about the database structure, and for testing!  I.e., a newly-created OM instance shouldn't have to be upgraded,
    /// because create_tables always provides the latest structure in a new system.  This method is just for updating older instances to what is in create_tables!
    fn do_database_upgrades_if_needed(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
    ) -> Result<(), anyhow::Error> {
        let version_table_exists: bool = self.does_this_exist(
            transaction,
            "select count(1) from pg_class where relname='om_db_version'",
            true,
        )?;
        if !version_table_exists {
            self.create_version_table(transaction)?;
        }
        let db_version_row: Vec<Option<DataType>> = self.db_query_wrapper_for_one_row(
            transaction,
            "select version from om_db_version",
            "Int",
        )?;
        let db_version = match db_version_row.get(0) {
            Some(Some(DataType::Smallint(i))) => i.clone(),
            _ => {
                return Err(anyhow!(
                    "In do_database_upgrades_if_needed, unexpected db_version: {:?}",
                    db_version_row
                ))
            }
        };
        // PUT THIS BACK & delete this cmt, when there is another upgrade, and similar ones to follow:
        // if db_version == 7 {
        //     db_version = upgradeDbFrom7to8()
        // }

        /* NOTE FOR FUTURE METHODS LIKE upgradeDbFrom0to1: methods like this should be designed carefully and very well-tested:
         0) make & test periodic backups of your live data to be safe!
         1) Consider designing it to be idempotent: so multiple runs on a production db (if by some mistake) will no harm (or at least will err out safely).
         2) Could run it against the test db (even though its tables already should have these changes, by being created from scratch), by not yet updating
            the table om_db_version (perhaps by temporarily commenting out the line with
            "UPDATE om_db_version ..." from create_tables while running tests).  AND,
         3) Could do a backup, open psql, start a transaction, paste the method's upgrade
            commands there, do manual verifications, then rollback.
         It doesn't seem to make sense to test methods like this with a unit test because the tests are run on a db created as a new
         system, so there is no upgrade to do on a new test, and no known need to call this method except on old systems being upgraded.
         (See also related comment above this do_database_upgrades_if_needed method.)  Better ideas?
        */

        // This at least makes sure all the upgrades ran to completion.
        // Idea: Should it be instead more specific to what versions of the db are compatible with
        // this version of the OM program, in case someone for example needs to restore old data but doesn't have an
        // older version of the OM program to go with it?
        if db_version as i32 != PostgreSQLDatabase::SCHEMA_VERSION {
            return Err(anyhow!("In do_database_upgrades_if_needed, db_version ({}) != PostgreSQLDatabase::SCHEMA_VERSION ({}).", db_version, PostgreSQLDatabase::SCHEMA_VERSION));
        }
        Ok(())
    }

    // See comment in ImportExport.processUriContent method which uses it, about where the
    // code should really go. Not sure if that idea includes this method or not.
    fn find_first_class_id_by_name(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        name_in: String,
        case_sensitive: bool, /*= false*/
    ) -> Result<Option<i64>, anyhow::Error> {
        // idea: see if queries like this are using the expected db index (run & ck the query
        // plan). Create tests around that, for benefit of future dbs? Or, just wait for
        // a performance issue then look at it?
        let name_clause = {
            if case_sensitive {
                format!("name = '{}'", name_in)
            } else {
                format!("lower(name) = lower('{}')", name_in)
            }
        };
        let sql = format!(
            "select id from class where {} order by id limit 1",
            name_clause
        );
        let rows = self.db_query(transaction, sql.as_str(), "i64")?;

        if rows.is_empty() {
            Ok(None)
        } else {
            let results = get_i64s_from_rows(&rows)?;
            if results.len() > 1 {
                return Err(anyhow!("In find_first_class_id_by_name, Expected 1 row (wanted just the first one), found {} rows.", results.len()));
            }
            Ok(Some(results[0]))
        }
    }
    /// @return the id of the new RTE
    fn add_has_relation_to_local_entity<'a>(
        &'a self,
        transaction: &Option<&mut Transaction<'a, Postgres>>,
        from_entity_id_in: i64,
        to_entity_id_in: i64,
        valid_on_date_in: Option<i64>,
        observation_date_in: i64,
        sorting_index_in: Option<i64>, /*= None*/
    ) -> Result<RelationToLocalEntity, anyhow::Error> {
        let relation_type_id: i64 =
            self.find_relation_type(transaction, Util::THE_HAS_RELATION_TYPE_NAME)?;
        let new_rte = self.create_relation_to_local_entity(
            transaction,
            relation_type_id,
            from_entity_id_in,
            to_entity_id_in,
            valid_on_date_in,
            observation_date_in,
            sorting_index_in,
            false,
        )?;
        Ok(new_rte)
    }

    fn update_class_name(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        id_in: i64,
        name_in: String,
    ) -> Result<u64, anyhow::Error> {
        let name: String = Self::escape_quotes_etc(name_in);
        self.db_action(
            transaction,
            format!(
                "update class set (name) = ROW('{}') where id={}",
                name, id_in
            )
            .as_str(),
            false,
            false,
        )
    }

    // This isn't really recursive and I don't immediately remember why.  Maybe it didn't make sense
    // or I was going to do it later.  It could use more thought.  Like how does that relate to
    // "deletions2" if at all.
    fn delete_relation_to_group_and_all_recursively<'a>(
        &'a self,
        transaction: &Option<&mut Transaction<'a, Postgres>>,
        group_id_in: i64,
    ) -> Result<(u64, u64), anyhow::Error> {
        let entity_ids: Vec<Vec<Option<DataType>>> = self.db_query(
            transaction,
            format!(
                "select entity_id from entitiesinagroup where group_id={}",
                group_id_in
            )
            .as_str(),
            "i64",
        )?;
        let num_e_ids: u64 = entity_ids.len().try_into()?;
        let deletions1 = self.delete_objects(
            transaction,
            "entitiesinagroup",
            format!("where group_id={}", group_id_in).as_str(),
            num_e_ids,
            true,
        )?;
        // Have to delete these 2nd because of a constraint on EntitiesInAGroup:
        // idea: is there a temp table somewhere that these could go into instead, for efficiency?
        // idea: batch these, would be much better performance.
        // idea: BUT: what is the length limit: should we do it it sets of N to not exceed sql command size limit?
        // idea: (also on task list i think but) we should not delete entities until dealing with their use as attrtypeids etc!
        for id_vec in entity_ids {
            match id_vec[0] {
                Some(DataType::Bigint(id)) => {
                    self.delete_objects(transaction, Util::ENTITY_TYPE,
                                format!("where id={}", id).as_str(), 1, true)?
                },
                None => return Err(anyhow!("In delete_relation_to_group_and_all_recursively, How did we get a null entity_id back from query?")),
                _ => return Err(anyhow!("In delete_relation_to_group_and_all_recursively, How did we get {:?} back from query?", id_vec)),
            };
        }

        let deletions2 = 0;
        //and finally:
        // (passing 0 for rows expected, because there either could be some, or none if the group is not contained in any entity.)
        self.delete_objects(
            transaction,
            Util::RELATION_TO_GROUP_TYPE,
            format!("where group_id={}", group_id_in).as_str(),
            0,
            true,
        )?;
        self.delete_objects(
            transaction,
            "grupo",
            format!("where id={}", group_id_in).as_str(),
            1,
            true,
        )?;
        Ok((deletions1, deletions2))
    }

    fn get_entity_attribute_sorting_data(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        entity_id_in: i64,
        limit_in: Option<i64>, /*= None*/
    ) -> Result<Vec<Vec<Option<DataType>>>, anyhow::Error> {
        // see comments in get_group_entries_data
        self.db_query(transaction, format!("select attribute_form_id, attribute_id, sorting_index from AttributeSorting where \
                                    entity_id = {} order by sorting_index limit {}", entity_id_in, Self::check_if_should_be_all_results(limit_in)).as_str(),
                    "Int,i64,i64")
    }

    fn check_if_should_be_all_results(max_vals_in: Option<i64>) -> String {
        match max_vals_in {
            None => "ALL".to_string(),
            Some(x) if x <= 0 => "1".to_string(),
            Some(x) => format!("{}", x),
        }
    }

    fn class_limit(
        limit_by_class: bool,
        class_id_in: Option<i64>,
    ) -> Result<String, anyhow::Error> {
        let res = if limit_by_class {
            match class_id_in {
                Some(x) => format!(" and e.class_id={} ", x).to_string(),
                _ => " and e.class_id is NULL ".to_string(),
            }
        } else {
            "".to_string()
        };
        Ok(res)
    }

    fn get_attribute_sorting_rows_count(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        entity_id_in: Option<i64>, /*= None*/
    ) -> Result<u64, anyhow::Error> {
        let where_entity_id = match entity_id_in {
            Some(x) => format!("where entity_id={}", x),
            _ => "".to_string(),
        };
        let sql = format!(
            "select count(1) from AttributeSorting {}",
            where_entity_id.as_str()
        );
        self.extract_row_count_from_count_query(transaction, sql.as_str())
    }

    fn get_relation_to_group_count_by_group(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        group_id_in: i64,
    ) -> Result<u64, anyhow::Error> {
        self.extract_row_count_from_count_query(
            transaction,
            format!(
                "select count(1) from relationtogroup where group_id={}",
                group_id_in
            )
            .as_str(),
        )
    }

    fn get_all_relation_to_local_entity_data_by_id(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        id_in: i64,
    ) -> Result<Vec<Option<DataType>>, anyhow::Error> {
        self.db_query_wrapper_for_one_row(transaction,
                                          format!("select form_id, id, rel_type_id, entity_id, entity_id_2, valid_on_date, observation_date from RelationToEntity where id={}", id_in).as_str(),
                                          "Int,i64,i64,i64,i64,i64,i64")
    }

    fn get_all_relation_to_remote_entity_data_by_id(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        id_in: i64,
    ) -> Result<Vec<Option<DataType>>, anyhow::Error> {
        self.db_query_wrapper_for_one_row(transaction,
                                          format!("select form_id, id, rel_type_id, entity_id, remote_instance_id, entity_id_2, valid_on_date, \
                                          observation_date from RelationToRemoteEntity where id={}", id_in).as_str(),
                                     "Int,i64,i64,i64,String,i64,i64,i64")
    }

    fn get_all_relation_to_group_data_by_id(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        id_in: i64,
    ) -> Result<Vec<Option<DataType>>, anyhow::Error> {
        self.db_query_wrapper_for_one_row(transaction,
                                          format!("select form_id, id, entity_id, rel_type_id, group_id, valid_on_date, observation_date from \
                                          RelationToGroup where id={}", id_in).as_str(),
                                     "Int,i64,i64,i64,i64,i64,i64")
    }

    // fn get_file_attribute_content(&self, transaction: &Option<&mut Transaction<Postgres>>, fileAttributeIdIn: i64, outputStreamIn: java.io.OutputStream) -> Result<(i64, String), anyhow::Error>  { {
    //     fn action(bufferIn: Array[Byte], starting_index_in: Int, numBytesIn: Int) {
    //         outputStreamIn.write(bufferIn, starting_index_in, numBytesIn)
    //     }
    //     let (fileSize, md5hash): (i64, String) = self.act_on_file_from_server(fileAttributeIdIn, action);
    //     (fileSize, md5hash)
    // }

    fn relation_to_group_keys_exist(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        entity_id: i64,
        relation_type_id: i64,
        group_id: i64,
    ) -> Result<bool, anyhow::Error> {
        self.does_this_exist(transaction,
                 format!("SELECT count(1) from RelationToGroup where entity_id={} and rel_type_id={} and group_id={}",
                         entity_id, relation_type_id, group_id).as_str(), true)
    }

    /// Excludes those entities that are really relationtypes, attribute types, or quantity units.
    fn entity_only_key_exists(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        id_in: i64,
    ) -> Result<bool, anyhow::Error> {
        let not_archived = if !self.include_archived_entities {
            "(not archived) and "
        } else {
            ""
        };
        let limit = Self::limit_to_entities_only(Self::ENTITY_ONLY_SELECT_PART);
        self.does_this_exist(
            transaction,
            format!(
                "SELECT count(1) from Entity where {} id={} and id in (select id from entity {})",
                not_archived, id_in, limit
            )
            .as_str(),
            true,
        )
    }

    fn relation_to_local_entity_exists(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        rel_type_id_in: i64,
        entity_id1_in: i64,
        entity_id2_in: i64,
    ) -> Result<bool, anyhow::Error> {
        self.does_this_exist(transaction, format!("SELECT count(1) from RelationToEntity where rel_type_id={} and entity_id={} and entity_id_2={}",
                                                  rel_type_id_in, entity_id1_in, entity_id2_in).as_str(), true)
    }

    fn relation_to_remote_entity_exists(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        rel_type_id_in: i64,
        entity_id1_in: i64,
        remote_instance_id_in: String,
        entity_id2_in: i64,
    ) -> Result<bool, anyhow::Error> {
        self.does_this_exist(transaction, format!("SELECT count(1) from RelationToRemoteEntity where rel_type_id={} and entity_id={} and \
                        remote_instance_id='{}' and entity_id_2={}",
                                                  rel_type_id_in, entity_id1_in, remote_instance_id_in, entity_id2_in).as_str(), true)
    }

    // fn add_new_entity_to_results(&self, transaction: &Option<&mut Transaction<Postgres>>, final_results: Vec<Entity>,
    //                              intermediate_result_in: Vec<Option<DataType>>) -> Result<bool, anyhow::Error> {
    //     let result = intermediate_result_in;
    //     // None of these values should be of "None" type. If they are it's a bug:
    //     let DataType::Bigint(id) = result.get(0)?;
    //     let DataType::String(name) = result.get(1)?;
    //     let Option(DataType::Bigint(class_id)) = result.get(2);
    //     let DataType::Bigint(insertion_date) = result.get(3)?;
    //     let Option(DataType::Boolean(public)) = result.get(4);
    //     let DataType::Boolean(archived) = result.get(5)?;
    //     let DataType::Boolean(new_entries_stick_to_top_) = result.get(6)?;
    //     final_results.add(Entity::new2(this, transaction, id, name, class_id, insertion_date, public, archived, new_entries_stick_to_top)
    // }

    fn get_containing_entities_helper(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        sql_in: &str,
    ) -> Result<Vec<(i64, Entity)>, anyhow::Error> {
        let early_results = self.db_query(transaction, sql_in, "i64,i64")?;
        let early_results_len = early_results.len();
        let mut final_results: Vec<(i64, Entity)> = Vec::new();
        // idea: should the remainder of this method be moved to Entity, so the persistence layer doesn't know anything about the Model? (helps avoid circular
        // dependencies? is a cleaner design?.)
        for result in early_results {
            // None of these values should be of "None" type, so not checking for that. If they are it's a bug:
            let rel_type_id = get_i64_from_row(&result, 0)?;
            let id = get_i64_from_row(&result, 1)?;
            let entity: Entity = Entity::new2(Box::new(self), transaction, id.clone()).unwrap();
            final_results.push((rel_type_id.clone(), entity))
        }

        if !(final_results.len() == early_results_len) {
            return Err(anyhow!("In get_containing_entities_helper, final_results.len() ({}) != early_results.len() ({}).", final_results.len(), early_results_len));
        }
        Ok(final_results)
    }
    //%%
    // fn get_containing_relation_to_groups_helper(&self, transaction: &Option<&mut Transaction<Postgres>>, sql_in: &str)  -> Result<Vec<RelationToGroup>, anyhow::Error>  {
    //     let early_results = self.db_query(transaction, sql_in, "i64")?;
    //     let mut group_id_results: Vec<i64> = Vec::new();
    //     // idea: should the remainder of this method be moved to Group, so the persistence layer doesn't know anything about the Model? (helps avoid circular
    //     // dependencies? is a cleaner design?)
    //     for result in early_results {
    //         //val group:Group = new Group(this, result(0).asInstanceOf[i64])
    //         let DataType::Bigint(id) = result.get(0)?;
    //         group_id_results.push(id.clone());
    //     }
    //     if group_id_results.len() != early_results.len() {
    //         return Err(anyhow!("In get_containing_relation_to_groups_helper, group_id_results.len() ({}) != early_results.len() ({})", group_id_results.len(), early_results.len()));
    //     }
    //     let mut containing_relations_to_group: Vec<RelationToGroup> = Vec::new();
    //     for gid in group_id_results {
    //         let rtgs: Vec<RelationToGroup> = self.get_relations_to_group_containing_this_group(transaction, gid, 0)?;
    //         for rtg in rtgs {
    //             containing_relations_to_group.push(rtg);
    //         }
    //     }
    //     Ok(containing_relations_to_group)
    // }
    fn get_entities_used_as_attribute_types_sql(
        &self,
        attribute_type_in: String,
        quantity_seeks_unit_not_type_in: bool,
    ) -> Result<String, anyhow::Error> {
        // whether it is archived doesn't seem relevant in the use case, but, it is debatable:
        //              (if !include_archived_entities) {
        //                "(not archived) and "
        //              } else {
        //                ""
        //              }) +
        let id_type = {
            // IN MAINTENANCE: compare to logic in method limit_to_entities_only.
            if Util::QUANTITY_TYPE == attribute_type_in && quantity_seeks_unit_not_type_in {
                "unit_id"
            } else if Util::NON_RELATION_ATTR_TYPE_NAMES.contains(&attribute_type_in.as_str()) {
                "attr_type_id"
            } else if Util::RELATION_TYPE_TYPE == attribute_type_in {
                "entity_id"
            } else if Util::RELATION_ATTR_TYPE_NAMES.contains(&attribute_type_in.as_str()) {
                "rel_type_id"
            } else {
                return Err(anyhow!(
                    "In get_entities_used_as_attribute_types_sql, unexpected attribute_type_in: {}",
                    attribute_type_in
                ));
            }
        };
        let mut sql: String = format!(" from Entity e where e.id in (select {} from ", id_type);
        if Util::NON_RELATION_ATTR_TYPE_NAMES.contains(&attribute_type_in.as_str())
            || Util::RELATION_ATTR_TYPE_NAMES.contains(&attribute_type_in.as_str())
        {
            // it happens to match the table name, which is convenient:
            sql = format!("{}{})", sql, attribute_type_in);
        } else {
            return Err(anyhow!(
                "In get_entities_used_as_attribute_types_sql, unexpected attribute_type_in: {}",
                attribute_type_in
            ));
        }
        Ok(sql)
    }

    // 1st parm is 0-based index to start with, 2nd parm is # of obj's to return (if None, means no limit).
    fn get_entities_generic(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        starting_object_index_in: i64,
        max_vals_in: Option<i64>,
        table_name_in: &str,
        class_id_in: Option<i64>,         /*= None*/
        limit_by_class: bool,             /*= false*/
        template_entity: Option<i64>,     /*= None*/
        group_to_omit_id_in: Option<i64>, /*= None*/
    ) -> Result<Vec<Entity>, anyhow::Error> {
        let some_sql = if table_name_in.eq_ignore_ascii_case(Util::RELATION_TYPE_TYPE) {
            ", r.name_in_reverse_direction, r.directionality "
        } else {
            ""
        };
        let more = " from Entity e ";
        let more2 = if table_name_in.eq_ignore_ascii_case(Util::RELATION_TYPE_TYPE) {
            // for RelationTypes, hit both tables since one "inherits", but limit it to those rows
            // for which a RelationType row also exists.
            ", RelationType r "
        } else {
            ""
        };
        let more3 = " where";
        let more4 = if !self.include_archived_entities {
            " (not archived) and"
        } else {
            ""
        };
        let more5 = Self::class_limit(limit_by_class, class_id_in)?;
        let more6 = if limit_by_class && template_entity.is_some() {
            format!(" and id != {}", template_entity.unwrap())
        } else {
            "".to_string()
        };
        let more7 = if table_name_in.eq_ignore_ascii_case(Util::RELATION_TYPE_TYPE) {
            // for RelationTypes, hit both tables since one "inherits", but limit it to those rows
            // for which a RelationType row also exists.
            " and e.id = r.entity_id "
        } else {
            ""
        };
        let more8 = if table_name_in.eq_ignore_ascii_case("EntityOnly") {
            Self::limit_to_entities_only(Util::SELECT_ENTITY_START)
        } else {
            "".to_string()
        };
        let more9 = if group_to_omit_id_in.is_some() {
            format!(" except ({} from entity e, EntitiesInAGroup eiag where e.id=eiag.entity_id and group_id={})",
                    Util::SELECT_ENTITY_START, group_to_omit_id_in.unwrap())
        } else {
            "".to_string()
        };
        let types = if table_name_in.eq_ignore_ascii_case(Util::RELATION_TYPE_TYPE) {
            "i64,String,i64,i64,bool,bool,String,String"
        } else {
            "i64,String,i64,i64,bool,bool,bool"
        };
        let sql = format!(
            "{}{}{}{}{}{} true {}{}{}{}{} order by id limit {} offset {}",
            Util::SELECT_ENTITY_START,
            some_sql,
            more,
            more2,
            more3,
            more4,
            more5,
            more6,
            more7,
            more8,
            more9,
            Self::check_if_should_be_all_results(max_vals_in),
            starting_object_index_in
        );
        let early_results = self.db_query(transaction, sql.as_str(), types)?;
        let early_results_len = early_results.len();

        let final_results: Vec<Entity> = Vec::new();
        // idea: should the remainder of this method be moved to Entity, so the persistence layer doesn't know anything about the Model? (helps avoid circular
        // dependencies; is a cleaner design?)  (and similar ones)
        for _result in early_results {
            // None of these values should be of "None" type. If they are it's a bug:
            if table_name_in.eq_ignore_ascii_case(Util::RELATION_TYPE_TYPE) {
                //%%$%%%
                // final_results.push(RelationType::new(&self, result(0).get.asInstanceOf[i64], result(1).get.asInstanceOf[String], result(6).get.asInstanceOf[String],
                //                                    result(7).get.asInstanceOf[String]))
            } else {
                //%%$%%%
                // add_new_entity_to_results(final_results, result)
            }
        }
        if final_results.len() != early_results_len {
            return Err(anyhow!(
                "In get_entities_generic, final_results.len() ({}) != early_results.len() ({})",
                final_results.len(),
                early_results_len
            ));
        }
        Ok(final_results)
    }

    //%%
    // fn get_text_editor_command(&self, transaction: &Option<&mut Transaction<Postgres>>) -> Result<String, anyhow::Error> {
    //     let system_entity_id = self.get_system_entity_id(transaction)?;
    //     let has_relation_type_id: i64 = self.find_relation_type(transaction, Util::THE_HAS_RELATION_TYPE_NAME)?;
    //     let editor_info_system_entities: Vec<Entity> = self.get_entities_from_relations_to_local_entity(transaction, system_entity_id,
    //                                                                                              Util::EDITOR_INFO_ENTITY_NAME,
    //                                                                                              Some(has_relation_type_id),
    //                                                                                                   Some(1))?;
    //     if editor_info_system_entities.len() < 1 {
    //         return Err(anyhow!("In get_text_editor_command, Unexpected # of results in get_text_editor_command a: {}", editor_info_system_entities.len()));
    //     }
    //     let id = editor_info_system_entities[0].get_id();
    //     let text_editor_info_system_entities: Vec<Entity> = self.get_entities_from_relations_to_local_entity(transaction, id,
    //                         Util::TEXT_EDITOR_INFO_ENTITY_NAME, Some(has_relation_type_id), Some(1))?;
    //     if text_editor_info_system_entities.len() < 1 {
    //         return Err(anyhow!("In get_text_editor_command, Unexpected # of results in get_text_editor_command b: {}", text_editor_info_system_entities.len()));
    //     }
    //     let text_editor_info_system_entity_id = text_editor_info_system_entities[0].get_id();
    //     let text_editor_command_name_attr_types: Vec<Entity> = self.get_entities_from_relations_to_local_entity(transaction,
    //                                                                            text_editor_info_system_entity_id,
    //                 Util::TEXT_EDITOR_COMMAND_ATTRIBUTE_TYPE_NAME, Some(has_relation_type_id), Some(1))?;
    //     if text_editor_command_name_attr_types.len() < 1 {
    //         return Err(anyhow!("In get_text_editor_command, Unexpected # of results in get_text_editor_command c: {}", text_editor_command_name_attr_types.len()));
    //     }
    //     let text_editor_command_name_attr_type_id = text_editor_command_name_attr_types[0].get_id();
    //     let tas: Vec<TextAttribute> = self.get_text_attribute_by_type_id(transaction, text_editor_info_system_entity_id,
    //                                                                      text_editor_command_name_attr_type_id, Some(1))?;
    //     if tas.len() < 1 {
    //         return Err(anyhow!("In get_text_editor_command, Unexpected # of results in get_text_editor_command d: {}", tas.len()));
    //     }
    //     tas[0].get_text()
    // }

    fn get_entities_from_relations_to_local_entity(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        parent_entity_id_in: i64,
        name_in: &str,
        rel_type_id_in: Option<i64>, /*= None*/
        expected_rows: Option<u64>,  /*= None*/
    ) -> Result<Vec<Entity>, anyhow::Error> {
        // (not getting all the attributes in this case, and doing another query to the entity table (less efficient), to save programming
        // time for the case that the entity table changes, we don't have to carefully update all the columns selected here & the mappings.  This is a more
        // likely change than for the TextAttribute table, below.
        let rel_type = match rel_type_id_in {
            Some(rtid) => format!(" and rel_type_id={}", rtid),
            _ => "".to_string(),
        };
        let query_results: Vec<Vec<Option<DataType>>> = self.db_query(transaction,
                                                                     format!("select id from entity where name='{}' and id in (select entity_id_2 from \
                                                                     relationToEntity where entity_id={} {})",
                                                                         name_in, parent_entity_id_in, rel_type).as_str(),
                                                        "i64")?;
        if let Some(expected_row_count) = expected_rows {
            let count = query_results.len();
            if count as u128 != expected_row_count as u128 {
                return Err(anyhow!("In get_entities_from_relations_to_local_entity, In get_entities_from_relations_to_local_entity, found {} rows in instead of expected {}", count, expected_row_count));
            }
        }
        let mut final_result: Vec<Entity> = Vec::with_capacity(query_results.len());
        // let mut index: usize = 0;
        for r in query_results {
            if r.len() == 0 {
                return Err(anyhow!("In get_entities_from_relations_to_local_entity, in get_entities_from_relations_to_local_entity, did not expect returned row to have 0 elements!: {:?}", r));
            }
            if let Some(DataType::Bigint(id)) = r[0] {
                final_result.push(Entity::new2(Box::new(self), transaction, id)?);
                // index += 1
            } else {
                return Err(anyhow!("In get_entities_from_relations_to_local_entity, in get_entities_from_relations_to_local_entity, did not expect this: {:?}", r[0]));
            }
        }
        Ok(final_result)
    }
    // //%%
    //     fn get_text_attribute_by_type_id(&self, transaction: &Option<&mut Transaction<Postgres>>,
    //                                      parent_entity_id_in: i64, type_id_in: i64,
    //                                      expected_rows: Option<usize> /*= None*/) -> Result<Vec<TextAttribute>, anyhow::Error> {
    //         let form_id: i32 = self.get_attribute_form_id(Util::TEXT_TYPE).unwrap();
    //         let sql = format!("select ta.id, ta.textvalue, ta.attr_type_id, ta.valid_on_date, ta.observation_date, asort.sorting_index from \
    //         textattribute ta, AttributeSorting asort where ta.entity_id={} and ta.attr_type_id={} and ta.entity_id=asort.entity_id and \
    //         asort.attribute_form_id={} and ta.id=asort.attribute_id",
    //             parent_entity_id_in, type_id_in, form_id).as_str();
    //         let query_results: Vec<Vec<Option<DataType>>> = self.db_query(transaction, sql, "i64,String,i64,i64,i64,i64")?;
    //         if let Some(expected_rows_len) = expected_rows {
    //             if query_results.len() != expected_rows_len {
    //                 return Err(anyhow!("In get_text_attribute_by_type_id, found {} rows instead of expected {}", query_results.len(), expected_rows_len));
    //             }
    //         }
    //         let final_result: Vec<TextAttribute> = Vec::with_capacity(query_results.len());
    //         for r in query_results {
    //             if r.len() < 6 {
    //                 return Err(anyhow!("In get_text_attribute_by_type_id, expected 6 elements in row returned, but found {}: {:?}", r.len(), r));
    //             }
    //             let Some(DataType::Bigint(text_attribute_id)) = r.get(0)?;
    //             let Some(DataType::String(textvalue)) = r.get(1)?;
    //             let Some(DataType::Bigint(attr_type_id)) = r.get(2)?;
    //             let valid_on_date = match r.get(3) {
    //                 None => None,
    //                 Some(DataType::Bigint(vod)) => Some(vod),
    //                 _ => return Err(anyhow!("In get_text_attribute_by_type_id, unexpected value in {:?}", r.get(3))),
    //             };
    //             let Some(DataType::Bigint(observation_date)) = r.get(4)?;
    //             let Some(DataType::Bigint(sorting_index)) = r.get(5)?;
    //             final_result.add(TextAttribute::new(Box::new(self), text_attribute_id, parent_entity_id_in, attr_type_id, textvalue, valid_on_date, observation_date, sorting_index));
    //         }
    //         Ok(final_result)
    //     }

    // %%
    // /// Returns an array of tuples, each of which is of (sorting_index, Attribute), and a i64 indicating the total # that could be returned with
    // /// infinite display space (total existing).
    // ///
    // /// The parameter max_vals_in can be 0 for 'all'.
    // ///
    // /// Idea to improve efficiency: make this able to query only those attributes needed to satisfy the max_vals_in parameter (by first checking
    // /// the AttributeSorting table).  In other words, no need to read all 1500 attributes to display on the screen, just to know which ones come first, if
    // /// only 10 can be displayed right now and the rest might not need to be displayed.  Because right now, we have to query all data from the AttributeSorting
    // /// table, then all attributes (since remember they might not *be* in the AttributeSorting table), then sort them with the best available information,
    // /// then decide which ones to return.  Maybe instead we could do that smartly, on just the needed subset.  But it still need to gracefully handle it
    // /// when a given attribute (or all) is not found in the sorting table.
    // fn get_sorted_attributes(&self, transaction: &Option<&mut Transaction<Postgres>>,
    //                          entity_id_in: i64, starting_object_index_in: usize /*= 0*/, max_vals_in: usize /*= 0*/,
    //                          only_public_entities_in: bool /*= true*/) -> Result<(Vec<(i64, Attribute)>, usize), anyhow::Error> {
    //     let allResults: java.util.ArrayList[(Option<i64>, Attribute)] = new java.util.ArrayList[(Option<i64>, Attribute)];
    //     // First select the counts from each table, keep a running total so we know when to select attributes (compared to inStartingObjectIndex)
    //     // and when to stop.
    //     let tables: Vec<String> = Array(Util.QUANTITY_TYPE, Util.BOOLEAN_TYPE, Util.DATE_TYPE, Util.TEXT_TYPE, Util.FILE_TYPE, Util.RELATION_TO_LOCAL_ENTITY_TYPE,;
    //     Util.RELATION_TO_GROUP_TYPE, Util.RELATION_TO_REMOTE_ENTITY_TYPE)
    //     let columnsSelectedByTable: Vec<String> = Array("id,entity_id,attr_type_id,unit_id,quantity_number,valid_on_date,observation_date",;
    //     "id,entity_id,attr_type_id,booleanValue,valid_on_date,observation_date",
    //     "id,entity_id,attr_type_id,date",
    //     "id,entity_id,attr_type_id,textvalue,valid_on_date,observation_date",
    //
    //     "id,entity_id,attr_type_id,description,original_file_date,stored_date,original_file_path,readable," +
    //     "writable,executable,size,md5hash",
    //
    //     "id,rel_type_id,entity_id,entity_id_2,valid_on_date,observation_date",
    //     "id,entity_id,rel_type_id,group_id,valid_on_date,observation_date",
    //     "id,rel_type_id,entity_id,remote_instance_id,entity_id_2,valid_on_date,observation_date")
    //     let typesByTable: Vec<String> = Array("i64,i64,i64,i64,i64,Float,i64,i64",;
    //     "i64,i64,i64,i64,bool,i64,i64",
    //     "i64,i64,i64,i64,i64",
    //     "i64,i64,i64,i64,String,i64,i64",
    //     "i64,i64,i64,i64,String,i64,i64,String,bool,bool,bool,i64,String",
    //     "i64,i64,i64,i64,i64,i64,i64",
    //     "i64,i64,i64,i64,i64,i64,i64",
    //     "i64,i64,i64,i64,String,i64,i64,i64")
    //     let where_clausesByTable: Vec<String> = Array(tables(0) + ".entity_id=" + entity_id_in, tables(1) + ".entity_id=" + entity_id_in,;
    //     tables(2) + ".entity_id=" + entity_id_in, tables(3) + ".entity_id=" + entity_id_in,
    //     tables(4) + ".entity_id=" + entity_id_in, tables(5) + ".entity_id=" + entity_id_in,
    //     tables(6) + ".entity_id=" + entity_id_in, tables(7) + ".entity_id=" + entity_id_in)
    //     let orderByClausesByTable: Vec<String> = Array("id", "id", "id", "id", "id", "entity_id", "group_id", "entity_id");
    //
    //     // *******************************************
    //     //****** NOTE **********: some logic here for counting & looping has been commented out because it is not yet updated to work with the sorting of
    //     // attributes on an entity.  But it is left here because it was so carefully debugged, once, and seems likely to be used again if we want to limit the
    //     // data queried and sorted to that amount which can be displayed at a given time.  For example,
    //     // we could query first from the AttributeSorting table, then based on that decide for which ones to get all the data. But maybe for now there's a small
    //     // enough amount of data that we can query all rows all the time.
    //     // *******************************************
    //
    //     // first just get a total row count for UI convenience later (to show how many left not viewed yet)
    //     // ABOUT THESE COMMENTED LINES: SEE "** NOTE **" ABOVE:
    //     //    let mut totalRowsAvailable: i64 = 0;
    //     //    let mut tableIndexForrow_counting = 0;
    //     //    while ((max_vals_in == 0 || totalRowsAvailable <= max_vals_in) && tableIndexForrow_counting < tables.length) {
    //     //      let table_name = tables(tableIndexForrow_counting);
    //     //      totalRowsAvailable += extract_row_count_from_count_query("select count(*) from " + table_name + " where " + where_clausesByTable(tableIndexForrow_counting))
    //     //      tableIndexForrow_counting += 1
    //     //    }
    //
    //     // idea: this could change to a let and be filled w/ a recursive helper method; other vars might go away then too.;
    //     let mut tableListIndex: i32 = 0;
    //
    //     // ABOUT THESE COMMENTED LINES: SEE "** NOTE **" ABOVE:
    //     //keeps track of where we are in getting rows >= inStartingObjectIndex and <= max_vals_in
    //     //    let mut counter: i64 = 0;
    //     //    while ((max_vals_in == 0 || counter - inStartingObjectIndex <= max_vals_in) && tableListIndex < tables.length) {
    //     while (tableListIndex < tables.length) {
    //     let table_name = tables(tableListIndex);
    //     // ABOUT THESE COMMENTED LINES: SEE "** NOTE **" ABOVE:
    //     //val thisTablesrow_count: i64 = extract_row_count_from_count_query("select count(*) from " + table_name + " where " + where_clausesByTable(tableListIndex))
    //     //if thisTablesrow_count > 0 && counter + thisTablesrow_count >= inStartingObjectIndex) {
    //     //try {
    //
    //     // Idea: could speed this query up in part? by doing on each query something like:
    //     //       limit max_vals_in+" offset "+ inStartingObjectIndex-counter;
    //     // ..and then incrementing the counters appropriately.
    //     // Idea: could do the sorting (currently done just before the end of this method) in sql? would have to combine all queries to all tables, though.
    //     let key = where_clausesByTable(tableListIndex).substring(0, where_clausesByTable(tableListIndex).indexOf("="));
    //     let columns = table_name + "." + columnsSelectedByTable(tableListIndex).replace(",", "," + table_name + ".");
    //     let mut sql: String = "select attributesorting.sorting_index, " + columns +;
    //     " from " +
    //     // idea: is the RIGHT JOIN really needed, or can it be a normal join? ie, given tables' setup can there really be
    //     // rows of any Attribute (or RelationTo*) table without a corresponding attributesorting row?  Going to assume not,
    //     // for some changes below adding the sortingindex parameter to the Attribute constructors, for now at least until this is studied
    //     // again.  Maybe it had to do with the earlier unreliability of always deleting rows from attributesorting when Attributes were
    //     // deleted (and in fact an attributesorting can in theory still be created without an Attribute row, and maybe other such problems).
    //     "   attributesorting RIGHT JOIN " + table_name +
    //     "     ON (attributesorting.attribute_form_id=" + Database.get_attribute_form_id(table_name) +
    //     "     and attributesorting.attribute_id=" + table_name + ".id )" +
    //     "   JOIN entity ON entity.id=" + key +
    //     " where " +
    //     (if !include_archived_entities) {
    //     "(not entity.archived) and "
    //     } else {
    //     ""
    //     }) +
    //     where_clausesByTable(tableListIndex)
    //     if table_name == Util.RELATION_TO_LOCAL_ENTITY_TYPE && !include_archived_entities) {
    //     sql += " and not exists(select 1 from entity e2, relationtoentity rte2 where e2.id=rte2.entity_id_2" +
    //     " and relationtoentity.entity_id_2=rte2.entity_id_2 and e2.archived)"
    //     }
    //     if table_name == Util.RELATION_TO_LOCAL_ENTITY_TYPE && only_public_entities_in) {
    //     sql += " and exists(select 1 from entity e2, relationtoentity rte2 where e2.id=rte2.entity_id_2" +
    //     " and relationtoentity.entity_id_2=rte2.entity_id_2 and e2.public)"
    //     }
    //     sql += " order by " + table_name + "." + orderByClausesByTable(tableListIndex)
    //     let results = db_query(sql, typesByTable(tableListIndex));
    //     for (result: Vec<Option<DataType>> <- results) {
    //     // skip past those that are outside the range to retrieve
    //     //idea: use some better scala/function construct here so we don't keep looping after counter hits the max (and to make it cleaner)?
    //     //idea: move it to the same layer of code that has the Attribute classes?
    //
    //     // ABOUT THESE COMMENTED LINES: SEE "** NOTE **" ABOVE:
    //     // Don't get it if it's not in the requested range:
    //     //            if counter >= inStartingObjectIndex && (max_vals_in == 0 || counter <= inStartingObjectIndex + max_vals_in)) {
    //     if table_name == Util.QUANTITY_TYPE) {
    //     allResults.add((if result(0).isEmpty) None else Some(result(0).get.asInstanceOf[i64]),
    //     new QuantityAttribute(this, result(1).get.asInstanceOf[i64], result(2).get.asInstanceOf[i64], result(3).get.asInstanceOf[i64],
    //     result(4).get.asInstanceOf[i64], result(5).get.asInstanceOf[Float],
    //     if result(6).isEmpty) None else Some(result(6).get.asInstanceOf[i64]), result(7).get.asInstanceOf[i64],
    //     result(0).get.asInstanceOf[i64])))
    //     } else if table_name == Util.TEXT_TYPE) {
    //     allResults.add((if result(0).isEmpty) None else Some(result(0).get.asInstanceOf[i64]),
    //     new TextAttribute(this, result(1).get.asInstanceOf[i64], result(2).get.asInstanceOf[i64], result(3).get.asInstanceOf[i64],
    //     result(4).get.asInstanceOf[String], if result(5).isEmpty) None else Some(result(5).get.asInstanceOf[i64]),
    //     result(6).get.asInstanceOf[i64], result(0).get.asInstanceOf[i64])))
    //     } else if table_name == Util.DATE_TYPE) {
    //     allResults.add((if result(0).isEmpty) None else Some(result(0).get.asInstanceOf[i64]),
    //     new DateAttribute(this, result(1).get.asInstanceOf[i64], result(2).get.asInstanceOf[i64], result(3).get.asInstanceOf[i64],
    //     result(4).get.asInstanceOf[i64], result(0).get.asInstanceOf[i64])))
    //     } else if table_name == Util.BOOLEAN_TYPE) {
    //     allResults.add((if result(0).isEmpty) None else Some(result(0).get.asInstanceOf[i64]),
    //     new BooleanAttribute(this, result(1).get.asInstanceOf[i64], result(2).get.asInstanceOf[i64], result(3).get.asInstanceOf[i64],
    //     result(4).get.asInstanceOf[Boolean], if result(5).isEmpty) None else Some(result(5).get.asInstanceOf[i64]),
    //     result(6).get.asInstanceOf[i64], result(0).get.asInstanceOf[i64])))
    //     } else if table_name == Util.FILE_TYPE) {
    //     allResults.add((if result(0).isEmpty) None else Some(result(0).get.asInstanceOf[i64]),
    //     new FileAttribute(this, result(1).get.asInstanceOf[i64], result(2).get.asInstanceOf[i64], result(3).get.asInstanceOf[i64],
    //     result(4).get.asInstanceOf[String], result(5).get.asInstanceOf[i64], result(6).get.asInstanceOf[i64],
    //     result(7).get.asInstanceOf[String], result(8).get.asInstanceOf[Boolean], result(9).get.asInstanceOf[Boolean],
    //     result(10).get.asInstanceOf[Boolean], result(11).get.asInstanceOf[i64], result(12).get.asInstanceOf[String],
    //     result(0).get.asInstanceOf[i64])))
    //     } else if table_name == Util.RELATION_TO_LOCAL_ENTITY_TYPE) {
    //     allResults.add((if result(0).isEmpty) None else Some(result(0).get.asInstanceOf[i64]),
    //     new RelationToLocalEntity(this, result(1).get.asInstanceOf[i64], result(2).get.asInstanceOf[i64], result(3).get.asInstanceOf[i64],
    //     result(4).get.asInstanceOf[i64],
    //     if result(5).isEmpty) None else Some(result(5).get.asInstanceOf[i64]), result(6).get.asInstanceOf[i64],
    //     result(0).get.asInstanceOf[i64])))
    //     } else if table_name == Util.RELATION_TO_GROUP_TYPE) {
    //     allResults.add((if result(0).isEmpty) None else Some(result(0).get.asInstanceOf[i64]),
    //     new RelationToGroup(this, result(1).get.asInstanceOf[i64], result(2).get.asInstanceOf[i64], result(3).get.asInstanceOf[i64],
    //     result(4).get.asInstanceOf[i64],
    //     if result(5).isEmpty) None else Some(result(5).get.asInstanceOf[i64]),
    //     result(6).get.asInstanceOf[i64], result(0).get.asInstanceOf[i64])))
    //     } else if table_name == Util.RELATION_TO_REMOTE_ENTITY_TYPE) {
    //     allResults.add((if result(0).isEmpty) None else Some(result(0).get.asInstanceOf[i64]),
    //     new RelationToRemoteEntity(this, result(1).get.asInstanceOf[i64], result(2).get.asInstanceOf[i64],
    //     result(3).get.asInstanceOf[i64],
    //     result(4).get.asInstanceOf[String], result(5).get.asInstanceOf[i64],
    //     if result(6).isEmpty) None else Some(result(6).get.asInstanceOf[i64]),
    //     result(7).get.asInstanceOf[i64],
    //     result(0).get.asInstanceOf[i64])))
    //     } else throw new OmDatabaseException("invalid table type?: '" + table_name + "'")
    //
    //     // ABOUT THESE COMMENTED LINES: SEE "** NOTE **" ABOVE:
    //     //}
    //     //            counter += 1
    //     }
    //
    //     // ABOUT THESE COMMENTED LINES: SEE "** NOTE **" ABOVE:
    //     //}
    //     //remove the try permanently, or, what should be here as a 'catch'? how interacts w/ 'throw' or anything related just above?
    //     //} else {
    //     //  counter += thisTablesrow_count
    //     //}
    //     tableListIndex += 1
    //     }
    //
    //     let allResultsArray: Array[(i64, Attribute)] = new Array[(i64, Attribute)](allResults.size);
    //     let mut index = -1;
    //     for (element: (Option<i64>, Attribute) <- allResults.toArray(new Array[(Option<i64>, Attribute)](0))) {
    //     index += 1
    //     // using max_id_value as the max value of a long so those w/o sorting information will just sort last:
    //     allResultsArray(index) = (element._1.getOrElse(self.max_id_value()), element._2)
    //     }
    //     // Per the scalaDocs for scala.math.Ordering, this sorts by the first element of the tuple (ie, .z_1) which at this point is attributesorting.sorting_index.
    //     // (The "getOrElse" on next line is to allow for the absence of a value in case the attributeSorting table doesn't have an entry for some attributes.
    //     Sorting.quickSort(allResultsArray)(Ordering[i64].on(x => x._1.asInstanceOf[i64]))
    //
    //     let from: i32 = starting_object_index_in;
    //     let numVals: i32 = if max_vals_in > 0) max_vals_in else allResultsArray.length;
    //     let until: i32 = Math.min(starting_object_index_in + numVals, allResultsArray.length);
    //     (allResultsArray.slice(from, until), allResultsArray.length)
    // }

    /// The inSelfIdToIgnore parameter is to avoid saying a class is a duplicate of itself: checks for all others only.
    fn is_duplicate_row(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        possible_duplicate_in: &str,
        table: &str,
        key_column_to_ignore_on: &str,
        column_to_check_for_dup_values: &str,
        extra_condition: Option<&str>,
        self_id_to_ignore_in: Option<String>, /*= None*/
    ) -> Result<bool, anyhow::Error> {
        let value_to_check: String = Self::escape_quotes_etc(possible_duplicate_in.to_string());

        let exception = match self_id_to_ignore_in {
            None => "".to_string(),
            Some(s) => format!("and not {}={}", key_column_to_ignore_on, s),
        };
        let ec = match extra_condition {
            Some(s) if s.len() > 0 => s,
            _ => "true",
        };
        self.does_this_exist(
            transaction,
            format!(
                "SELECT count({}) from {} where {} and lower({})=lower('{}') {}",
                key_column_to_ignore_on,
                table,
                ec,
                column_to_check_for_dup_values,
                value_to_check,
                exception
            )
            .as_str(),
            false,
        )
    }

    /// Cloned from delete_objects: CONSIDER UPDATING BOTH if updating one.
    /// Returns the # of rows affected (archived or un-archived).
    fn archive_objects<'a>(
        &'a self,
        transaction_in: &Option<&mut Transaction<'a, Postgres>>,
        table_name_in: &str,
        where_clause_in: &str,
        rows_expected: u64,                   /*= 1*/
        caller_manages_transactions_in: bool, /*= false*/
        unarchive: bool,                      /*= false*/
    ) -> Result<u64, anyhow::Error> {
        //idea: enhance this to also check & return the # of rows deleted, to the caller to just make sure? If so would have to let caller handle transactions.

        //BEGIN COPY/PASTED/DUPLICATED (except "in <fn_name>" below) BLOCK-----------------------------------
        // Try creating a local transaction whether we use it or not, to handle compiler errors
        // about variable moves. I'm not seeing a better way to get around them by just using
        // conditions and an Option (many errors):
        // (I tried putting this in a function, then a macro, but it gets compile errors.
        // So, copy/pasting this, unfortunately, until someone thinks of a better way. (You
        // can see the macro, and one of the compile errors, in the commit of 2023-05-18.
        // I didn't try a proc macro but based on some reading I think it would have the same
        // problem.)
        let mut local_tx: Transaction<Postgres> = {
            if transaction_in.is_none() {
                if caller_manages_transactions_in {
                    return Err(anyhow!("In archive_objects, inconsistent values for caller_manages_transactions_in \
                                and transaction_in: true and None??"
                    .to_string()));
                } else {
                    self.begin_trans()?
                }
            } else {
                if caller_manages_transactions_in {
                    // That means we have determined that the caller is to use the transaction_in .
                    // was just:  None
                    // But now instead, create it anyway, per comment above.
                    self.begin_trans()?
                } else {
                    return Err(anyhow!(
                        "In archive_objects, inconsistent values for caller_manages_transactions_in & transaction_in: \
                                false and Some??"
                            .to_string(),
                    ));
                }
            }
        };
        let local_tx_option = &Some(&mut local_tx);
        let transaction: &Option<&mut Transaction<Postgres>> = if caller_manages_transactions_in {
            transaction_in
        } else {
            local_tx_option
        };
        //END OF COPY/PASTED/DUPLICATED BLOCK----------------------------------

        let archive = if unarchive { "false" } else { "true" };
        let archived_date = if unarchive {
            "NULL".to_string()
        } else {
            Utc::now().timestamp_millis().to_string()
        };
        let sql = format!(
            "update {} set (archived, archived_date) = ({}, {}), {}",
            table_name_in, archive, archived_date, where_clause_in
        );
        let rows_affected = self.db_action(transaction, sql.as_str(), true, false)?;
        if rows_expected > 0 && rows_affected != rows_expected {
            // No need to explicitly roll back a locally created transaction aka tx, though we
            // definitely don't want to archive an unexpected # of rows,
            // because rollback is implicit whenever the transaction goes out of scope without a commit.
            // Caller should roll back (or fail to commit, same thing) in case of error.
            return Err(anyhow!(format!(
                            "In archive_objects, archive (or unarchive) would have affected {} rows, but {} were expected! \
                            Did not perform archive (or unarchive).  SQL is: \"{}\"",
                            rows_affected, rows_expected, sql)));
        } else {
            //%%put this & similar places into a function like self.commit_or_err(tx)?;   ?  If so, include the rollback cmt from just above?
            if !caller_manages_transactions_in {
                // Using local_tx to make the compiler happy and because it is the one we need,
                // if !caller_manages_transactions_in. Ie, there is no transaction provided by
                // the caller.
                if let Err(e) = self.commit_trans(local_tx) {
                    return Err(anyhow!(e.to_string()));
                }
            }
            Ok(rows_affected)
        }
    }

    fn delete_object_by_id<'a>(
        &'a self,
        transaction_in: &Option<&mut Transaction<'a, Postgres>>,
        table_name_in: &str,
        id_in: i64,
        caller_manages_transactions_in: bool, /*= false*/
    ) -> Result<u64, anyhow::Error> {
        self.delete_objects(
            transaction_in,
            table_name_in,
            format!("where id={}", id_in).as_str(),
            1,
            caller_manages_transactions_in,
        )
    }

    fn delete_object_by_id2<'a>(
        &'a self,
        transaction_in: &Option<&mut Transaction<'a, Postgres>>,
        table_name_in: &str,
        id_in: &str,
        caller_manages_transactions_in: bool, /*= false*/
    ) -> Result<u64, anyhow::Error> {
        self.delete_objects(
            transaction_in,
            table_name_in,
            format!("where id='{}'", id_in).as_str(),
            1,
            caller_manages_transactions_in,
        )
    }
    /*%%
                  // (idea: find out: why doesn't compiler (ide or cli) complain when the 'override' is removed from next line?)
                  // idea: see comment on findUnusedSortingIndex
                    fn findIdWhichIsNotKeyOfAnyEntity -> i64 {
                    //better idea?  This should be fast because we start in remote regions and return as soon as an unused id is found, probably
                    //only one iteration, ever.  (See similar comments elsewhere.)
                    let starting_id: i64 = self.max_id_value() - 1;

                    @tailrec fn findIdWhichIsNotKeyOfAnyEntity_helper(working_id: i64, counter: i64) -> i64 {
                      //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) w/in this method!
                      if entity_key_exists(working_id)) {
                        if working_id == self.max_id_value()) {
                          // means we did a full loop across all possible ids!?  Doubtful. Probably would turn into a performance problem long before. It's a bug.
                          throw new OmDatabaseException("No id found which is not a key of any entity in the system. How could all id's be used??")
                        }
                        // idea: this check assumes that the thing to get IDs will re-use deleted ones and wrap around the set of #'s. That fix is on the list (informally
                        // at this writing, 2013-11-18).
                        if counter > 1000) throw new OmDatabaseException("Very unexpected, but could it be that you are running out of available entity IDs?? Have someone check, " +
                                                                "before you need to create, for example, a thousand more entities.")
                        findIdWhichIsNotKeyOfAnyEntity_helper(working_id - 1, counter + 1)
                      } else working_id
                    }

                    findIdWhichIsNotKeyOfAnyEntity_helper(starting_id, 0)
                  }

                  // (see note in ImportExport's call to this, on this being better in the class and action *tables*, but here for now until those features are ready)
                    fn addUriEntityWithUriAttribute(containingEntityIn: Entity, new_entity_name_in: String, uriIn: String, observation_date_in: i64,
                                                   makeThem_publicIn: Option<bool>, caller_manages_transactions_in: bool,
                                                   quoteIn: Option<String> /*= None*/) -> (Entity, RelationToLocalEntity) {
                    if quoteIn.is_some()) require(!quoteIn.get.isEmpty, "It doesn't make sense to store a blank quotation; there was probably a program error.")
                          //rollbacketc%%FIX NEXT LINE AFTERI SEE HOW OTHERS DO!
                    // if !caller_manages_transactions_in { self.begin_trans() }
                    try {
                      // **idea: BAD SMELL: should this method be moved out of the db class, since it depends on higher-layer components, like EntityClass and
                      // those in the same package? It was in Controller, but moved here
                      // because it seemed like things that manage transactions should be in the db layer.  So maybe it needs un-mixing of layers.

                      let (uriClassId: i64, uriClassTemplateId: i64) = get_or_create_class_and_template_entity("URI", caller_manages_transactions_in);
                      let (_, quotationClassTemplateId: i64) = get_or_create_class_and_template_entity("quote", caller_manages_transactions_in);
                      let (newEntity: Entity, newRTLE: RelationToLocalEntity) = containingEntityIn.create_entityAndAddHASLocalRelationToIt(new_entity_name_in, observation_date_in,;
                                                                                                                               makeThem_publicIn, caller_manages_transactions_in)
                      update_entitys_class(newEntity.get_id, Some(uriClassId), caller_manages_transactions_in)
                      newEntity.addTextAttribute(uriClassTemplateId, uriIn, None, None, observation_date_in, caller_manages_transactions_in)
                      if quoteIn.is_some()) {
                        newEntity.addTextAttribute(quotationClassTemplateId, quoteIn.get, None, None, observation_date_in, caller_manages_transactions_in)
                      }
                          //rollbacketc%%FIX NEXT LINE AFTERI SEE HOW OTHERS DO!
                      // if !caller_manages_transactions_in {self.commit_trans() }
                      (newEntity, newRTLE)
                    } catch {
                      case e: Exception =>
                          //rollbacketc%%FIX NEXT LINE AFTERI SEE HOW OTHERS DO!
                        // if !caller_manages_transactions_in) rollback_trans()
                        throw e
                    }
                  }

              /// @return the OmInstance object that stands for *this*: the OmInstance to which this PostgreSQLDatabase class instance reads/writes directly.
            fn get_local_om_instance_data() -> OmInstance {
                let sql = "SELECT id, address, insertion_date, entity_id from omInstance where local=TRUE";
                let results = db_query(sql, "String,String,i64,i64");
                if results.size != 1) throw new OmDatabaseException("Got " + results.size + " instead of 1 result from sql " + sql +
                                                                     ".  Does the usage now warrant removing this check (ie, multiple locals stored)?")
                let result = results.head;
                new OmInstance(this, result(0).get.asInstanceOf[String], is_local_in = true,
                               result(1).get.asInstanceOf[String],
                               result(2).get.asInstanceOf[i64], if result(3).isEmpty) None else Some(result(3).get.asInstanceOf[i64]))
              }

    */
    //%%$%% moved methods that are not part of the Database trait go here
}

impl Database for PostgreSQLDatabase {
    fn is_remote(&self) -> bool {
        false
    }

    ///  This means whether to act on *all* entities (true), or only non-archived (false, the more typical use).  Needs clarification?
    fn include_archived_entities(&self) -> bool {
        self.include_archived_entities
    }

    /// Like jdbc's default, if you don't call begin/rollback/commit, sqlx will commit after every
    /// sql statement, but if you call begin/rollback/commit, it will let you manage
    /// explicitly and will automatically turn autocommit on/off as needed to allow that. (???)
    fn begin_trans(&self) -> Result<Transaction<Postgres>, anyhow::Error> {
        // let mut tx = self.rt.block_on(self.pool.begin())?;
        let tx: Transaction<Postgres> = match self.rt.block_on(self.pool.begin()) {
            Err(e) => return Err(anyhow!(e.to_string())),
            Ok(t) => t,
        };
        // %% see comments in fn connect() re this AND remove above method comment??
        // connection.setAutoCommit(false);
        Ok(tx)
    }
    /// Not needed when the transaction simply goes out of scope! Rollback is then automatic, per
    /// sqlx and a test I wrote to verify it, below.
    fn rollback_trans(&self, tx: Transaction<Postgres>) -> Result<(), anyhow::Error> {
        return match self.rt.block_on(tx.rollback()) {
            Err(e) => Err(anyhow!(e.to_string())),
            Ok(()) => Ok(()),
        };
        // so future work is auto- committed unless programmer explicitly opens another transaction
        //%% see comments in fn connect() re this
        // connection.setAutoCommit(true);
    }
    fn commit_trans(&self, tx: Transaction<Postgres>) -> Result<(), anyhow::Error> {
        if let Err(e) = self.rt.block_on(tx.commit()) {
            return Err(anyhow!(e.to_string()));
        }
        Ok(())
        // so future work is auto- committed unless programmer explicitly opens another transaction
        //%% see comments in fn connect() re this
        // connection.setAutoCommit(true);
    }

    fn find_all_entity_ids_by_name(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        name_in: String,
        case_sensitive: bool, /*= false*/
    ) -> Result<Vec<i64>, anyhow::Error> {
        // idea: see if queries like this are using the expected index (run & ck the query plan). Tests around that, for benefit of future dbs? Or, just wait for
        // a performance issue then look at it?
        let not_archived = if !self.include_archived_entities {
            "(not archived) and "
        } else {
            ""
        };
        let case: String = {
            if case_sensitive {
                format!("name = '{}'", name_in)
            } else {
                format!("lower(name) = lower('{})", name_in)
            }
        };
        let sql = format!("select id from entity where {}{}", not_archived, case);
        let rows = self.db_query(transaction, sql.as_str(), "i64")?;
        let results = get_i64s_from_rows(&rows)?;
        Ok(results)
    }

    /// @param search_string_in is case-insensitive.
    /// @param stop_after_any_found is to prevent a serious performance problem when searching for the default entity at startup, if that default entity
    ///                          eventually links to 1000's of others.  Alternatives included specifying a different levels_remaining parameter in that
    ///                          case, or not following any RelationTo[Local|Remote]Entity links (which defeats the ability to organize the preferences in a hierarchy),
    ///                          or flagging certain ones to skip by marking them as a preference (not a link to follow in the preferences hierarchy), but
    ///                          those all seemed more complicated.
    fn find_contained_local_entity_ids<'a>(
        &'a self,
        transaction: &Option<&mut Transaction<Postgres>>,
        results_in_out: &'a mut HashSet<i64>,
        from_entity_id_in: i64,
        search_string_in: &str,
        levels_remaining: i32,      /*%% = 20*/
        stop_after_any_found: bool, /*%% = true*/
    ) -> Result<&mut HashSet<i64>, anyhow::Error> {
        // Idea for optimizing: don't re-traverse dup ones (eg, circular links or entities in same two places).  But that has other complexities: see
        // comments on ImportExport.exportItsChildrenToHtmlFiles for more info.  But since we are limiting the # of levels total, it might not matter anyway
        // (ie, probably the current code is not optimized but is simpler and good enough for now).

        if levels_remaining <= 0 || (stop_after_any_found && results_in_out.len() > 0) {
            // do nothing: get out.
        } else {
            let condition = if !self.include_archived_entities {
                "and not e.archived"
            } else {
                ""
            };
            let sql = format!(
                "select rte.entity_id_2, e.name from entity e, RelationToEntity rte \
                  where rte.entity_id={} and rte.entity_id_2=e.id {}",
                from_entity_id_in, condition
            );
            let related_entity_id_rows = self.db_query(transaction, sql.as_str(), "i64,String")?;
            // let lower_cased_regex_pattern = Pattern.compile(".*" + search_string_in.to_lowercase() + ".*");
            let mut id: i64;
            let mut name: String;
            for row in related_entity_id_rows {
                // id = match row.get(0) {
                //     Some(Some(DataType::Bigint(x))) => *x,
                //     _ => {
                //         return Err(anyhow!(format!(
                //             "How did we get here for {:?}?",
                //             row.get(0)
                //         )))
                //     }
                // };
                id = get_i64_from_row(&row, 0)?;
                name = match row.get(1) {
                    Some(Some(DataType::String(x))) => x.clone(),
                    _ => {
                        return Err(anyhow!(format!(
                            "How did we get here for {:?}?",
                            row.get(1)
                        )))
                    }
                };

                // NOTE: this line, similar lines just below, and the prompt inside EntityMenu.entitySearchSubmenu __should all match__.
                if name
                    .to_lowercase()
                    .contains(&search_string_in.to_lowercase())
                {
                    // if lower_cased_regex_pattern.matcher(name.toLowerCase).find {
                    // have to do the name check here because we need to traverse all contained entities, so we need all those back from the sql, not just name matches.
                    results_in_out.insert(id);
                }
                self.find_contained_local_entity_ids(
                    transaction,
                    results_in_out,
                    id,
                    &search_string_in,
                    levels_remaining - 1,
                    stop_after_any_found,
                )?;
            }
            if !(stop_after_any_found && results_in_out.len() > 0) {
                let condition = if !self.include_archived_entities {
                    " and not e.archived"
                } else {
                    ""
                };
                let sql2 = format!("select eiag.entity_id, e.name from RelationToGroup rtg, EntitiesInAGroup eiag, entity e \
                    where rtg.entity_id={} and rtg.group_id=eiag.group_id and eiag.entity_id=e.id {}", from_entity_id_in, condition);
                let entities_in_groups = self.db_query(transaction, sql2.as_str(), "i64,String")?;
                for row in entities_in_groups {
                    // let id: i64 = row(0).get.asInstanceOf[i64];
                    // let name = row(1).get.asInstanceOf[String];
                    //idea: surely there is some better way than what I am doing here? See other places similarly.
                    //   DataType::Bigint(id) = *row.get(0).unwrap();
                    //   DataType::String(name) = *row.get(1).unwrap();
                    id = match row.get(0) {
                        Some(Some(DataType::Bigint(x))) => *x,
                        _ => {
                            return Err(anyhow!(format!(
                                "How did we get here for {:?}?",
                                row.get(0)
                            )))
                        }
                    };
                    // DataType::String(name) = *row.get(1).unwrap();
                    name = match row.get(1) {
                        Some(Some(DataType::String(x))) => x.clone(),
                        _ => {
                            return Err(anyhow!(format!(
                                "How did we get here for {:?}?",
                                row.get(1)
                            )))
                        }
                    };

                    // NOTE: this line, similar or related lines just above & below, and the prompt inside EntityMenu.entitySearchSubmenu __should all match__.
                    if name
                        .to_lowercase()
                        .contains(&search_string_in.to_lowercase())
                    {
                        // if lower_cased_regex_pattern.matcher(name.toLowerCase).find {
                        // have to do the name check here because we need to traverse all contained entities, so we need all those back from the sql, not just name matches.
                        results_in_out.insert(id);
                    }
                    self.find_contained_local_entity_ids(
                        transaction,
                        results_in_out,
                        id,
                        search_string_in,
                        levels_remaining - 1,
                        stop_after_any_found,
                    )?;
                }
            }
            // this part is doing a regex now:
            if !(stop_after_any_found && results_in_out.len() > 0) {
                let if_archived = if !self.include_archived_entities {
                    " and (not e.archived)"
                } else {
                    ""
                };
                // *NOTE*: this line about textvalue, similar lines just above (doing "matcher ..."), and the prompt
                // inside EntityMenu.entitySearchSubmenu __should all match__.
                let sql3 = format!(
                    "select ta.id from textattribute ta, entity e where \
                                entity_id=e.id{} and entity_id={} and textvalue ~* '{}'",
                    if_archived, from_entity_id_in, search_string_in
                );
                //idea: just select a count, instead of requesting all the data back?
                let text_attributes = self.db_query(transaction, sql3.as_str(), "i64")?;
                if text_attributes.len() > 0 {
                    results_in_out.insert(from_entity_id_in);
                }
            }
        }
        Ok(results_in_out)
    }

    fn create_class_and_its_template_entity<'a>(
        &'a self,
        transaction: &Option<&mut Transaction<'a, Postgres>>,
        class_name_in: String,
    ) -> Result<(i64, i64), anyhow::Error> {
        self.create_class_and_its_template_entity2(
            transaction,
            class_name_in.clone(),
            format!("{}{}", class_name_in.clone(), Util::TEMPLATE_NAME_SUFFIX),
            transaction.is_some(),
        )
    }

    fn delete_class_and_its_template_entity(&self, class_id_in: i64) -> Result<(), anyhow::Error> {
        let mut tx: Transaction<Postgres> = self.begin_trans()?;
        let transaction: &Option<&mut Transaction<Postgres>> = &Some(&mut tx);
        let template_entity_id_vec: Vec<Option<DataType>> =
            self.get_class_data(transaction, class_id_in)?;
        let template_entity_id: i64 = match template_entity_id_vec.get(1) {
            Some(Some(DataType::Bigint(n))) => *n,
            _ => {
                return Err(anyhow!(
                    "In delete_class_and_its_template_entity, Unexpected values for template: {:?}",
                    template_entity_id_vec
                ))
            }
        };
        let class_group_id: Option<i64> = self.get_system_entitys_class_group_id(transaction)?;
        if class_group_id.is_some() {
            self.remove_entity_from_group(
                transaction,
                class_group_id.unwrap(),
                template_entity_id,
                true,
            )?;
        }
        self.update_entitys_class(transaction, template_entity_id, None, true)?;
        self.delete_object_by_id2(transaction, "class", class_id_in.to_string().as_str(), true)?;
        self.delete_object_by_id2(
            transaction,
            Util::ENTITY_TYPE,
            template_entity_id.to_string().as_str(),
            true,
        )?;

        // if let Err(e) = self.commit_trans(tx) {
        //     return Err(anyhow!(e.to_string()));
        // }
        // Ok(())
        self.commit_trans(tx)
    }

    /// Returns at most 1 row's info (id, relation_type_id, group_id, name), and a boolean indicating if more were available.
    /// If 0 rows are found, returns (None, None, None, false), so this expects the caller
    /// to know there is only one or deal with the None.
    fn find_relation_to_and_group_on_entity(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        entity_id_in: i64,
        group_name_in: Option<String>, /*%% = None*/
    ) -> Result<(Option<i64>, Option<i64>, Option<i64>, Option<String>, bool), anyhow::Error> {
        let name_condition = match group_name_in {
            Some(gni) => {
                let name = Self::escape_quotes_etc(gni);
                format!("g.name='{}'", name)
            }
            __ => "true".to_string(),
        };

        // "limit 2", so we know and can return whether more were available:
        let rows: Vec<Vec<Option<DataType>>> = self.db_query(transaction, format!("select rtg.id, rtg.rel_type_id, g.id, g.name from relationtogroup rtg, grupo g where rtg.group_id=g.id \
                                       and rtg.entity_id={} and {} order by rtg.id limit 2",
                                        entity_id_in, name_condition).as_str(), "i64,i64,i64,String")?;
        // there could be none found, or more than one, but:
        if rows.is_empty() {
            return Ok((None, None, None, None, false));
        } else {
            let row: Vec<Option<DataType>> = rows[0].clone();
            let id: Option<i64> = {
                match row[0] {
                    Some(DataType::Bigint(x)) => Some(x),
                    _ => {
                        return Err(anyhow!(
                            "In find_relation_to_and_group_on_entity, should never happen 2"
                                .to_string()
                        ))
                    }
                }
            };
            let rel_type_id: Option<i64> = {
                match row[1] {
                    Some(DataType::Bigint(x)) => Some(x),
                    _ => {
                        return Err(anyhow!(
                            "In find_relation_to_and_group_on_entity, should never happen 3"
                                .to_string()
                        ))
                    }
                }
            };
            let group_id: Option<i64> = {
                match row[2] {
                    Some(DataType::Bigint(x)) => Some(x),
                    _ => {
                        return Err(anyhow!(
                            "In find_relation_to_and_group_on_entity, should never happen 4"
                                .to_string()
                        ))
                    }
                }
            };
            let name: Option<String> = {
                match row[3].clone() {
                    Some(DataType::String(x)) => Some(x),
                    _ => {
                        return Err(anyhow!(
                            "In find_relation_to_and_group_on_entity, should never happen 5"
                                .to_string()
                        ))
                    }
                }
            };
            return Ok((id, rel_type_id, group_id, name, rows.len() > 1));
        }
    }

    /// Returns at most 1 id (and a the ideas was?: boolean indicating if more were available?).
    /// If 0 rows are found, return(ed?) (None,false), so this expects the caller
    /// to know there is only one or deal with the None.
    fn find_relation_type(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        type_name_in: &str,
    ) -> Result<i64, anyhow::Error> {
        let name = Self::escape_quotes_etc(type_name_in.to_string());
        let rows = self.db_query(
            transaction,
            format!(
                "select entity_id from entity e, relationtype rt where \
                                 e.id=rt.entity_id and name='{}' order by id limit 2",
                name
            )
            .as_str(),
            "i64",
        )?;
        let count = rows.len();
        if count != 1 {
            return Err(anyhow!(format!(
                "Found {} rows instead of expected {}",
                count, 1
            )));
            //?: expected_rows.unwrap()));
        }
        // there could be none found, or more than one, but not after above check.
        //     let mut final_result: Vec<i64> = Vec::new();
        // for row in rows {
        let id: i64 = match rows[0].get(0) {
            Some(Some(DataType::Bigint(i))) => *i,
            _ => {
                return Err(anyhow!(format!(
                    "Found not 1 row with i64 but {:?} .",
                    rows
                )))
            }
        };
        // final_result.push(id);
        // }
        // Ok(final_result)
        Ok(id)
    }
    /// Saves data for a quantity attribute for a Entity (i.e., "6 inches length").<br>
    /// parent_id_in is the key of the Entity for which the info is being saved.<br>
    /// inUnitId represents a Entity; indicates the unit for this quantity (i.e., liters or inches).<br>
    /// inNumber represents "how many" of the given unit.<br>
    /// attr_type_id_in represents the attribute type and also is a Entity (i.e., "volume" or "length")<br>
    /// valid_on_date_in represents the date on which this began to be true (seems it could match the observation date if needed,
    /// or guess when it was definitely true);
    /// NULL means unknown, 0 means it is asserted true for all time. observation_date_in is the date the fact was observed. <br>
    /// <br>
    /// We store the dates in
    /// postgresql (at least) as bigint which should be the same size as a java long, with the understanding that we are
    /// talking about java-style dates here; it is my understanding that such long's can also be negative to represent
    /// dates long before 1970, or positive for dates long after 1970. <br>
    /// <br>
    /// In the case of inNumber, note
    /// that the postgresql docs give some warnings about the precision of its real and "double precision" types. Given those
    /// warnings and the fact that I haven't investigated carefully (as of 9/2002) how the data will be saved and read
    /// between the java float type and the postgresql types, I am using "double precision" as the postgresql data type,
    /// as a guess to try to lose as
    /// little information as possible, and I'm making this note to you the reader, so that if you care about the exactness
    /// of the data you can do some research and let us know what you find.
    /// <p/>
    /// Re dates' meanings: see usage notes elsewhere in code (like inside create_tables).
    fn create_quantity_attribute<'a>(
        &'a self,
        transaction_in: &Option<&mut Transaction<'a, Postgres>>,
        parent_id_in: i64,
        attr_type_id_in: i64,
        unit_id_in: i64,
        number_in: f64,
        valid_on_date_in: Option<i64>,
        observation_date_in: i64,
        caller_manages_transactions_in: bool, /*= false*/
        sorting_index_in: Option<i64>,        /*= None*/
    ) -> Result</*id*/ i64, anyhow::Error> {
        //BEGIN COPY/PASTED/DUPLICATED (except "in <fn_name>" below) BLOCK-----------------------------------
        // Try creating a local transaction whether we use it or not, to handle compiler errors
        // about variable moves. I'm not seeing a better way to get around them by just using
        // conditions and an Option (many errors):
        // (I tried putting this in a function, then a macro, but it gets compile errors.
        // So, copy/pasting this, unfortunately, until someone thinks of a better way. (You
        // can see the macro, and one of the compile errors, in the commit of 2023-05-18.
        // I didn't try a proc macro but based on some reading I think it would have the same
        // problem.)
        let mut local_tx: Transaction<Postgres> = {
            if transaction_in.is_none() {
                if caller_manages_transactions_in {
                    return Err(anyhow!("In create_quantity_attribute, Inconsistent values for caller_manages_transactions_in \
                                and transaction_in: true and None??"
                    .to_string()));
                } else {
                    self.begin_trans()?
                }
            } else {
                if caller_manages_transactions_in {
                    // That means we have determined that the caller is to use the transaction_in .
                    // was just:  None
                    // But now instead, create it anyway, per comment above.
                    self.begin_trans()?
                } else {
                    return Err(anyhow!(
                        "In create_quantity_attribute, Inconsistent values for caller_manages_transactions_in & transaction_in: \
                                false and Some??"
                            .to_string(),
                    ));
                }
            }
        };
        let local_tx_option = &Some(&mut local_tx);
        let transaction: &Option<&mut Transaction<Postgres>> = if caller_manages_transactions_in {
            transaction_in
        } else {
            local_tx_option
        };
        //END OF COPY/PASTED/DUPLICATED BLOCK----------------------------------

        let id: i64 = self.get_new_key(transaction, "QuantityAttributeKeySequence")?;
        let form_id = self.get_attribute_form_id(Util::QUANTITY_TYPE)?;
        self.add_attribute_sorting_row(transaction, parent_id_in, form_id, id, sorting_index_in)?;
        let valid_on = match valid_on_date_in {
            None => "NULL".to_string(),
            Some(d) => format!("{}", d),
        };
        self.db_action(transaction,
                                         format!("insert into QuantityAttribute (id, entity_id, unit_id, \
                                         quantity_number, attr_type_id, valid_on_date, observation_date) values ({},{},{},{},\
                                         {},{},{})", id, parent_id_in, unit_id_in, number_in, attr_type_id_in, valid_on, observation_date_in).as_str(),
                                         false, false)?;
        if !caller_manages_transactions_in {
            // see comments at similar location in delete_objects about local_tx
            if let Err(e) = self.commit_trans(local_tx) {
                // see comments in delete_objects about rollback
                return Err(anyhow!(e.to_string()));
            }
        }
        Ok(id)
    }

    fn update_quantity_attribute(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        id_in: i64,
        parent_id_in: i64,
        attr_type_id_in: i64,
        unit_id_in: i64,
        number_in: f64,
        valid_on_date_in: Option<i64>,
        observation_date_in: i64,
    ) -> Result<u64, anyhow::Error> {
        // NOTE: IF ADDING COLUMNS TO WHAT IS UPDATED, SIMILARLY UPDATE caller's update method! (else some fields don't get updated
        // in memory when the db updates, and the behavior gets weird.
        let valid_on = match valid_on_date_in {
            None => "NULL".to_string(),
            Some(d) => format!("{}", d),
        };
        self.db_action(transaction, format!("update QuantityAttribute set (unit_id, quantity_number, attr_type_id, valid_on_date, \
                        observation_date) = ({},{},{},{},{}) where id={} and  entity_id={}", unit_id_in, number_in, attr_type_id_in,
                            valid_on, observation_date_in, id_in, parent_id_in).as_str(),
                                       false, false)
    }

    fn update_text_attribute(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        id_in: i64,
        parent_id_in: i64,
        attr_type_id_in: i64,
        text_in: String,
        valid_on_date_in: Option<i64>,
        observation_date_in: i64,
    ) -> Result<u64, anyhow::Error> {
        let text: String = Self::escape_quotes_etc(text_in);
        let valid_on = match valid_on_date_in {
            None => "NULL".to_string(),
            Some(d) => format!("{}", d),
        };
        // NOTE: IF ADDING COLUMNS TO WHAT IS UPDATED, SIMILARLY UPDATE caller's update method! (else some fields don't get updated
        // in memory when the db updates, and the behavior gets weird.
        self.db_action(transaction, format!("update TextAttribute set (textvalue, attr_type_id, valid_on_date, observation_date) \
                        = ('{}',{},{},{}) where id={} and entity_id={}", text, attr_type_id_in,
                                 valid_on, observation_date_in, id_in, parent_id_in).as_str(),
                        false, false)
    }

    fn update_date_attribute(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        id_in: i64,
        parent_id_in: i64,
        date_in: i64,
        attr_type_id_in: i64,
    ) -> Result<u64, anyhow::Error> {
        // NOTE: IF ADDING COLUMNS TO WHAT IS UPDATED, SIMILARLY UPDATE caller's update method! (else some fields don't get updated
        // in memory when the db updates, and the behavior gets weird.
        self.db_action(
            transaction,
            format!(
                "update DateAttribute set (date, attr_type_id) \
                        = ({},{}) where id={} and entity_id={}",
                date_in, attr_type_id_in, id_in, parent_id_in
            )
            .as_str(),
            false,
            false,
        )
    }
    fn update_boolean_attribute(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        id_in: i64,
        parent_id_in: i64,
        attr_type_id_in: i64,
        boolean_in: bool,
        valid_on_date_in: Option<i64>,
        observation_date_in: i64,
    ) -> Result<(), anyhow::Error> {
        // NOTE: IF ADDING COLUMNS TO WHAT IS UPDATED, SIMILARLY UPDATE caller's update method! (else some fields don't get updated
        // in memory when the db updates, and the behavior gets weird.
        let if_valid_on_date = match valid_on_date_in {
            None => "NULL".to_string(),
            Some(date) => date.to_string(),
        };
        self.db_action(transaction, format!("update BooleanAttribute set (booleanValue, attr_type_id, valid_on_date, observation_date) \
                        = ({},{},{},{}) where id={} and entity_id={}",
                        boolean_in, attr_type_id_in, if_valid_on_date, observation_date_in, id_in, parent_id_in).as_str(),
                                   false, false)?;
        Ok(())
    }
    /// We don't update the dates, path, size, hash because we set those based on the file's own timestamp, path current date,
    /// & contents when it is written. So the only
    /// point to having an update method might be the attribute type & description.
    /// AND THAT: The valid_on_date for a file attr shouldn't ever be None/NULL like with other attrs, because it is the file date in the filesystem before it was
    /// read into OM.
    fn update_file_attribute(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        id_in: i64,
        parent_id_in: i64,
        attr_type_id_in: i64,
        description_in: String,
    ) -> Result<u64, anyhow::Error> {
        // NOTE: IF ADDING COLUMNS TO WHAT IS UPDATED, SIMILARLY UPDATE caller's update method! (else some fields don't get updated
        // in memory when the db updates, and the behavior gets weird.
        self.db_action(
            transaction,
            format!(
                "update FileAttribute set (description, attr_type_id) \
                   = ('{}',{}) where id={} and entity_id={}",
                description_in, attr_type_id_in, id_in, parent_id_in
            )
            .as_str(),
            false,
            false,
        )
    }

    /// first take on this: might have a use for it later.  It's tested, and didn't delete, but none known now. Remove?
    fn update_file_attribute2(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        id_in: i64,
        parent_id_in: i64,
        attr_type_id_in: i64,
        description_in: String,
        original_file_date_in: i64,
        stored_date_in: i64,
        original_file_path_in: String,
        readable_in: bool,
        writable_in: bool,
        executable_in: bool,
        size_in: i64,
        md5_hash_in: String,
    ) -> Result<u64, anyhow::Error> {
        // NOTE: IF ADDING COLUMNS TO WHAT IS UPDATED, SIMILARLY UPDATE caller's update method! (else some fields don't get updated
        // in memory when the db updates, and the behavior gets weird.
        self.db_action(transaction, format!("update FileAttribute set (description, attr_type_id, original_file_date, stored_date, \
                   original_file_path, readable, writable, executable, size, md5hash) = ('{}',{},{},{},'{}', {},{},{}, {}, '{}') where id={} and entity_id={}",
                       description_in, attr_type_id_in, original_file_date_in, stored_date_in, original_file_path_in, readable_in, writable_in, executable_in,
                            size_in, md5_hash_in, id_in, parent_id_in).as_str(),
                                  false, false)
    }

    fn update_entity_only_name(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        id_in: i64,
        name_in: &str,
    ) -> Result<u64, anyhow::Error> {
        let name: String = Self::escape_quotes_etc(name_in.to_string());
        self.db_action(
            transaction,
            format!(
                "update Entity set (name) = ROW('{}') where id={}",
                name, id_in
            )
            .as_str(),
            false,
            false,
        )
    }

    fn update_entity_only_public_status(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        id_in: i64,
        value_in: Option<bool>,
    ) -> Result<u64, anyhow::Error> {
        let value = match value_in {
            None => "NULL",
            Some(v) => {
                if v {
                    "true"
                } else {
                    "false"
                }
            }
        };
        self.db_action(
            transaction,
            format!(
                "update Entity set (public) = ROW({}) where id={}",
                value, id_in
            )
            .as_str(),
            false,
            false,
        )
    }

    fn update_entity_only_new_entries_stick_to_top(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        id_in: i64,
        new_entries_stick_to_top: bool,
    ) -> Result<u64, anyhow::Error> {
        self.db_action(
            transaction,
            format!(
                "update Entity set (new_entries_stick_to_top) = ROW('{}\
                   ') where id={}",
                new_entries_stick_to_top, id_in
            )
            .as_str(),
            false,
            false,
        )
    }

    // //%%put back after EntityClass fleshed out w/ ::new() ?
    //              fn update_class_and_template_entity_name(&self, class_id_in: i64,
    //                                                       name: &str) -> Result<i64, anyhow::Error> {
    //                let mut tx = self.begin_trans()?;
    //                  let transaction: &Option<&mut Transaction<Postgres>> = &Some(&mut tx);
    //                  self.update_class_name(transaction, class_id_in, name)?;
    //                  let mut entity_id: i64 = EntityClass::new(this, class_id_in).get_template_entity_id()?;
    //                  self.update_entity_only_name(transaction, entity_id, format!("{}{}", name, Util::TEMPLATE_NAME_SUFFIX)?;
    //                  if let Err(e) = self.commit_trans(tx) {
    //                      // see comments in delete_objects about rollback
    //                      return Err(anyhow!(e.to_string()));
    //                  }
    //                Ok(entity_id)
    //              }

    fn update_entitys_class<'a>(
        &'a self,
        transaction_in: &Option<&mut Transaction<'a, Postgres>>,
        entity_id: i64,
        class_id: Option<i64>,
        caller_manages_transactions_in: bool, /*= false*/
    ) -> Result<(), anyhow::Error> {
        //BEGIN COPY/PASTED/DUPLICATED (except "in <fn_name>" below) BLOCK-----------------------------------
        // Try creating a local transaction whether we use it or not, to handle compiler errors
        // about variable moves. I'm not seeing a better way to get around them by just using
        // conditions and an Option (many errors):
        // (I tried putting this in a function, then a macro, but it gets compile errors.
        // So, copy/pasting this, unfortunately, until someone thinks of a better way. (You
        // can see the macro, and one of the compile errors, in the commit of 2023-05-18.
        // I didn't try a proc macro but based on some reading I think it would have the same
        // problem.)
        let mut local_tx: Transaction<Postgres> = {
            if transaction_in.is_none() {
                if caller_manages_transactions_in {
                    return Err(anyhow!("In update_entitys_class, Inconsistent values for caller_manages_transactions_in \
                                and transaction_in: true and None??"
                    .to_string()));
                } else {
                    self.begin_trans()?
                }
            } else {
                if caller_manages_transactions_in {
                    // That means we have determined that the caller is to use the transaction_in .
                    // was just:  None
                    // But now instead, create it anyway, per comment above.
                    self.begin_trans()?
                } else {
                    return Err(anyhow!(
                        "In update_entitys_class, Inconsistent values for caller_manages_transactions_in & transaction_in: \
                                false and Some??"
                            .to_string(),
                    ));
                }
            }
        };
        let local_tx_option = &Some(&mut local_tx);
        let transaction: &Option<&mut Transaction<Postgres>> = if caller_manages_transactions_in {
            transaction_in
        } else {
            local_tx_option
        };
        //END OF COPY/PASTED/DUPLICATED BLOCK----------------------------------

        let ci = match class_id {
            None => "NULL".to_string(),
            Some(x) => format!("{}", x),
        };
        self.db_action(
            transaction,
            format!(
                "update Entity set (class_id) = ROW({}) where id={}",
                ci, entity_id
            )
            .as_str(),
            false,
            false,
        )?;
        let group_ids = self.db_query(
            transaction,
            format!(
                "select group_id from \
                        EntitiesInAGroup where entity_id={}",
                entity_id
            )
            .as_str(),
            "i64",
        )?;
        for row in group_ids {
            let group_id = match row.get(0) {
                Some(Some(DataType::Bigint(gid))) => *gid,
                _ => {
                    return Err(anyhow!(
                        "In update_entitys_class, unsure how got here for row {:?}",
                        row
                    ))
                }
            };
            let mixed_classes_allowed: bool =
                self.are_mixed_classes_allowed(transaction, &group_id)?;
            if !mixed_classes_allowed && self.has_mixed_classes(transaction, &group_id)? {
                return Err(anyhow!(
                    "In update_entitys_class: {}",
                    Util::MIXED_CLASSES_EXCEPTION
                ));
            }
        }
        if !caller_manages_transactions_in {
            // see comments at similar location in delete_objects about local_tx
            if let Err(e) = self.commit_trans(local_tx) {
                // see comments in delete_objects about rollback
                return Err(anyhow!("In update_entitys_class: {}", e.to_string()));
            }
        }
        Ok(())
    }

    fn update_relation_type(
        &self,
        id_in: i64,
        name_in: String,
        name_in_reverse_direction_in: String,
        directionality_in: String,
    ) -> Result<(), anyhow::Error> {
        assert!(name_in.len() > 0);
        assert!(name_in_reverse_direction_in.len() > 0);
        assert!(directionality_in.len() > 0);
        let name_in_reverse_direction: String =
            Self::escape_quotes_etc(name_in_reverse_direction_in);
        let name: String = Self::escape_quotes_etc(name_in);
        let directionality: String = Self::escape_quotes_etc(directionality_in);
        let mut tx = self.begin_trans()?;
        let transaction: &Option<&mut Transaction<Postgres>> = &Some(&mut tx);
        self.db_action(
            transaction,
            format!(
                "update Entity set (name) = ROW('{}') where id={}",
                name, id_in
            )
            .as_str(),
            false,
            false,
        )?;
        self.db_action(
            transaction,
            format!(
                "update RelationType set (name_in_reverse_direction, directionality) = \
                        ROW('{}', '{}') where entity_id={}",
                name_in_reverse_direction, directionality, id_in
            )
            .as_str(),
            false,
            false,
        )?;

        self.commit_trans(tx)
    }

    /// Re dates' meanings: see usage notes elsewhere in code (like inside create_tables).
    fn create_text_attribute<'a>(
        &'a self,
        // purpose: see comment in delete_objects
        transaction_in: &Option<&mut Transaction<'a, Postgres>>,
        parent_id_in: i64,
        attr_type_id_in: i64,
        text_in: &str,
        valid_on_date_in: Option<i64>, /*%%= None*/
        observation_date_in: i64,      /*%%= System.currentTimeMillis()*/
        // purpose: see comment in delete_objects
        caller_manages_transactions_in: bool, /*%% = false*/
        sorting_index_in: Option<i64>,        /*%%= None*/
    ) -> Result<i64, anyhow::Error> {
        //BEGIN COPY/PASTED/DUPLICATED (except "in <fn_name>" below) BLOCK-----------------------------------
        // Try creating a local transaction whether we use it or not, to handle compiler errors
        // about variable moves. I'm not seeing a better way to get around them by just using
        // conditions and an Option (many errors):
        // (I tried putting this in a function, then a macro, but it gets compile errors.
        // So, copy/pasting this, unfortunately, until someone thinks of a better way. (You
        // can see the macro, and one of the compile errors, in the commit of 2023-05-18.
        // I didn't try a proc macro but based on some reading I think it would have the same
        // problem.)
        let mut local_tx: Transaction<Postgres> = {
            if transaction_in.is_none() {
                if caller_manages_transactions_in {
                    return Err(anyhow!("In create_text_attribute, inconsistent values for caller_manages_transactions_in \
                                and transaction_in: true and None??"
                        .to_string()));
                } else {
                    self.begin_trans()?
                }
            } else {
                if caller_manages_transactions_in {
                    // That means we have determined that the caller is to use the transaction_in .
                    // was just:  None
                    // But now instead, create it anyway, per comment above.
                    self.begin_trans()?
                } else {
                    return Err(anyhow!(
                        "In create_text_attribute, inconsistent values for caller_manages_transactions_in & transaction_in: \
                                false and Some??"
                            .to_string(),
                    ));
                }
            }
        };
        let local_tx_option = &Some(&mut local_tx);
        let transaction: &Option<&mut Transaction<Postgres>> = if caller_manages_transactions_in {
            transaction_in
        } else {
            local_tx_option
        };
        //END OF COPY/PASTED/DUPLICATED BLOCK----------------------------------

        let text: String = Self::escape_quotes_etc(text_in.to_string());
        let id: i64 = self.get_new_key(&transaction, "TextAttributeKeySequence")?;
        let add_result = self.add_attribute_sorting_row(
            &transaction,
            parent_id_in,
            self.get_attribute_form_id(Util::TEXT_TYPE).unwrap(),
            id,
            sorting_index_in,
        );
        match add_result {
            Err(s) => {
                // see comments in delete_objects about rollback
                return Err(anyhow!(s.to_string()));
            }
            _ => {}
        }
        let result = self.db_action(
            &transaction,
            format!(
                "insert into TextAttribute (id, entity_id, textvalue, \
                  attr_type_id, valid_on_date, observation_date) values ({},{},'{}',{},{},{})",
                id,
                parent_id_in,
                text,
                attr_type_id_in,
                match valid_on_date_in {
                    None => "NULL".to_string(),
                    Some(vod) => vod.to_string(),
                },
                observation_date_in
            )
            .as_str(),
            false,
            false,
        );
        match result {
            Err(s) => {
                // see comments in delete_objects about rollback
                return Err(anyhow!(s.to_string()));
            }
            _ => {}
        };
        if !caller_manages_transactions_in {
            // see comments at similar location in delete_objects about local_tx
            if let Err(e) = self.commit_trans(local_tx) {
                // see comments in delete_objects about rollback
                return Err(anyhow!(e.to_string()));
            }
        }
        Ok(id)
    }

    fn create_date_attribute(
        &self,
        parent_id_in: i64,
        attr_type_id_in: i64,
        date_in: i64,
        sorting_index_in: Option<i64>, /*= None*/
    ) -> Result</*id*/ i64, anyhow::Error> {
        let mut tx = self.begin_trans()?;
        let transaction: &Option<&mut Transaction<Postgres>> = &Some(&mut tx);
        let id: i64 = self.get_new_key(transaction, "DateAttributeKeySequence")?;
        self.add_attribute_sorting_row(
            transaction,
            parent_id_in,
            self.get_attribute_form_id(Util::DATE_TYPE).unwrap(),
            id,
            sorting_index_in,
        )?;
        self.db_action(
            transaction,
            format!(
                "insert into DateAttribute (id, entity_id, attr_type_id, date) \
                    values ({},{},'{}',{})",
                id, parent_id_in, attr_type_id_in, date_in
            )
            .as_str(),
            false,
            false,
        )?;
        self.commit_trans(tx)?;
        Ok(id)
    }

    fn create_boolean_attribute(
        &self,
        parent_id_in: i64,
        attr_type_id_in: i64,
        boolean_in: bool,
        valid_on_date_in: Option<i64>,
        observation_date_in: i64,
        sorting_index_in: Option<i64>, /*%%= None*/
    ) -> Result<i64, anyhow::Error> {
        let mut tx: Transaction<Postgres> = self.begin_trans()?;
        let id: i64 = self.get_new_key(&Some(&mut tx), "BooleanAttributeKeySequence")?;
        // try {
        self.add_attribute_sorting_row(
            &Some(&mut tx),
            parent_id_in,
            self.get_attribute_form_id(Util::BOOLEAN_TYPE).unwrap(),
            id,
            sorting_index_in,
        )?;
        let vod = match valid_on_date_in {
            None => "NULL".to_string(),
            Some(date) => date.to_string(),
        };
        self.db_action(
            &Some(&mut tx),
            format!(
                "insert into BooleanAttribute (id, \
            entity_id, booleanvalue, attr_type_id, valid_on_date, observation_date) \
            values ({},{},'{}',{},{},{})",
                id, parent_id_in, boolean_in, attr_type_id_in, vod, observation_date_in
            )
            .as_str(),
            false,
            false,
        )?;
        self.commit_trans(tx)?;
        Ok(id)
    }

    //%%
    //   fn create_file_attribute(&self, parent_id_in: i64, attr_type_id_in: i64, description_in: String, original_file_date_in: i64, stored_date_in: i64,
    //                         original_file_path_in: String, readable_in: bool, writable_in: bool, executable_in: bool, size_in: i64,
    //                         md5_hash_in: String, inputStreamIn: java.io.FileInputStream, sorting_index_in: Option<i64> /*= None*/) -> Result</*id*/ i64, anyhow::Error> {
    //   let description: String = self.escape_quotes_etc(description_in);
    //   // (Next 2 for completeness but there shouldn't ever be a problem if other code is correct.)
    //   let original_file_path: String = self.escape_quotes_etc(original_file_path_in);
    //   // Escaping the md5hash string shouldn't ever matter, but security is more important than the md5hash:
    //   let md5hash: String = self.escape_quotes_etc(md5_hash_in);
    //   let mut obj: LargeObject = null;
    //   let mut id: i64 = 0;
    //   try {
    //     id = get_new_key("FileAttributeKeySequence")?;
    //     let mut tx = self.begin_trans()?;
    //       let transaction: &Option<&mut Transaction<Postgres>> = &Some(&mut tx);
    //     self.add_attribute_sorting_row(transaction, parent_id_in, self.get_attribute_form_id(Util::FILE_TYPE).unwrap(), id, sorting_index_in)?;
    //     self.db_action(transaction, format!("insert into FileAttribute (id, entity_id, attr_type_id, description, original_file_date, \
    //         stored_date, original_file_path, readable, writable, executable, size, md5hash)" +
    //          " values ({},{},{},'{}',{},{}, '{}', {}, {}, {}, {},'{}')", id, parent_id_in, attr_type_id_in, description, original_file_date_in,
    //             stored_date_in, original_file_path, readable_in, writable_in, executable_in, size_in, md5hash).as_str(),
    //                    false, false);
    //     // from the example at:   http://jdbc.postgresql.org/documentation/80/binary-data.html & info
    //     // at http://jdbc.postgresql.org/documentation/publicapi/org/postgresql/largeobject/LargeObjectManager.html & its links.
    //     let lobjManager: LargeObjectManager = connection.asInstanceOf[org.postgresql.PGConnection].getLargeObjectAPI;
    //     let oid: i64 = lobjManager.createLO();
    //     obj = lobjManager.open(oid, LargeObjectManager.WRITE)
    //     let buffer = new Array[Byte](2048);
    //     let mut numBytesRead = 0;
    //     let mut total: i64 = 0;
    //     @tailrec
    //     //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) w/in this method!
    //     fn saveFileToDb() {
    //       numBytesRead = inputStreamIn.read(buffer)
    //       // (intentional style violation, for readability):
    //       //noinspection ScalaUselessExpression
    //       if numBytesRead == -1) Unit
    //       else {
    //         // (just once by a subclass is enough to mess w/ the md5sum for testing:)
    //         if total == 0) damageBuffer(buffer)
    //
    //         obj.write(buffer, 0, numBytesRead)
    //         total += numBytesRead
    //         saveFileToDb()
    //       }
    //     }
    //     saveFileToDb()
    //     if total != size_in {
    //       return Err(anyhow!("In create_file_attribute, Transferred {} bytes instead of {}??", total, size_in));
    //     }
    //     self.db_action(transaction, format!("INSERT INTO FileAttributeContent (file_attribute_id, contents_oid) \
    //         VALUES ({},{})", id, oid).as_str(), false, false);
    //
    //     let (success, errMsgOption) = self.verify_file_attribute_content_integrity(id);
    //     if !success {
    //         let msg = errMsgOption.getOrElse("(verification provided no error message? How?)");
    //       return Err(anyhow!("In create_file_attribute, Failure to successfully upload file content: {}", msg));
    //     }
    //     self.commit_trans(tx)?;
    //     id
    //   } finally {
    //     if obj != null) {
    //           try {
    //             obj.close()
    //           } catch {
    //               case e: Exception =>
    //               // not sure why this fails sometimes, if it's a bad thing or not, but for now not going to be stuck on it.
    //               // idea: look at the source code that throws it..?.
    //           }
    //       }
    //   }
    // }

    /// Re dates' meanings: see usage notes elsewhere in code (like inside create_tables). */
    fn create_relation_to_local_entity<'a>(
        &'a self,
        // purpose: see comment in delete_objects
        transaction_in: &Option<&mut Transaction<'a, Postgres>>,
        relation_type_id_in: i64,
        entity_id1_in: i64,
        entity_id2_in: i64,
        valid_on_date_in: Option<i64>,
        observation_date_in: i64,
        sorting_index_in: Option<i64>, /*%% = None*/
        // purpose: see comment in delete_objects
        caller_manages_transactions_in: bool, /*%% = false*/
    ) -> Result<RelationToLocalEntity, anyhow::Error> {
        debug!("in create_relation_to_local_entity 0");
        //BEGIN COPY/PASTED/DUPLICATED (except "in <fn_name>" below) BLOCK-----------------------------------
        // Try creating a local transaction whether we use it or not, to handle compiler errors
        // about variable moves. I'm not seeing a better way to get around them by just using
        // conditions and an Option (many errors):
        // (I tried putting this in a function, then a macro, but it gets compile errors.
        // So, copy/pasting this, unfortunately, until someone thinks of a better way. (You
        // can see the macro, and one of the compile errors, in the commit of 2023-05-18.
        // I didn't try a proc macro but based on some reading I think it would have the same
        // problem.)
        let mut local_tx: Transaction<Postgres> = {
            if transaction_in.is_none() {
                if caller_manages_transactions_in {
                    return Err(anyhow!("In create_relation_to_local_entity, Inconsistent values for caller_manages_transactions_in \
                                and transaction_in: true and None??"
                        .to_string()));
                } else {
                    self.begin_trans()?
                }
            } else {
                if caller_manages_transactions_in {
                    // That means we have determined that the caller is to use the transaction_in .
                    // was just:  None
                    // But now instead, create it anyway, per comment above.
                    self.begin_trans()?
                } else {
                    return Err(anyhow!(
                        "Inconsistent values for caller_manages_transactions_in & transaction_in: \
                                false and Some??"
                            .to_string(),
                    ));
                }
            }
        };
        let local_tx_option = &Some(&mut local_tx);
        let transaction: &Option<&mut Transaction<Postgres>> = if caller_manages_transactions_in {
            transaction_in
        } else {
            local_tx_option
        };
        //END OF COPY/PASTED/DUPLICATED BLOCK----------------------------------

        debug!("in create_relation_to_local_entity 1");
        let rte_id: i64 = self.get_new_key(&transaction, "RelationToEntityKeySequence")?;
        let result: Result<i64, anyhow::Error> = self.add_attribute_sorting_row(
            &transaction,
            entity_id1_in,
            self.get_attribute_form_id(Util::RELATION_TO_LOCAL_ENTITY_TYPE)
                .unwrap(),
            rte_id,
            sorting_index_in,
        );
        if let Err(e) = result {
            // see comments in delete_objects about rollback
            return Err(anyhow!(e));
        }
        let valid_on_date_sql_str = match valid_on_date_in {
            Some(date) => date.to_string(),
            None => "NULL".to_string(),
        };
        debug!("in create_relation_to_local_entity 2");
        let result = self.db_action(&transaction, format!("INSERT INTO RelationToEntity (id, rel_type_id, entity_id, entity_id_2, valid_on_date, observation_date) \
                       VALUES ({},{},{},{}, {},{})", rte_id, relation_type_id_in, entity_id1_in, entity_id2_in,
                       valid_on_date_sql_str, observation_date_in).as_str(), false, false);
        debug!("in create_relation_to_local_entity 3");
        if let Err(e) = result {
            // see comments in delete_objects about rollback
            return Err(anyhow!(e));
        }
        debug!("in create_relation_to_local_entity 4");
        if !caller_manages_transactions_in {
            // see comments at similar location in delete_objects about local_tx
            if let Err(e) = self.commit_trans(local_tx) {
                // see comments in delete_objects about rollback
                return Err(anyhow!(e.to_string()));
            }
        }
        debug!("in create_relation_to_local_entity 5");
        Ok(RelationToLocalEntity {}) //%%$%%really: self, rte_id, relation_type_id_in, entity_id1_in, entity_id2_in})
    }

    /** Re dates' meanings: see usage notes elsewhere in code (like inside create_tables). */
    fn create_relation_to_remote_entity<'a>(
        &'a self,
        // purpose: see comment in delete_objects
        transaction_in: &Option<&mut Transaction<'a, Postgres>>,
        relation_type_id_in: i64,
        entity_id1_in: i64,
        entity_id2_in: i64,
        valid_on_date_in: Option<i64>,
        observation_date_in: i64,
        remote_instance_id_in: &str,
        sorting_index_in: Option<i64>, /*%% = None*/
        // purpose: see comment in delete_objects
        caller_manages_transactions_in: bool, /*%% = false*/
    ) -> Result<RelationToRemoteEntity, anyhow::Error> {
        //BEGIN COPY/PASTED/DUPLICATED (except "in <fn_name>" below) BLOCK-----------------------------------
        // Try creating a local transaction whether we use it or not, to handle compiler errors
        // about variable moves. I'm not seeing a better way to get around them by just using
        // conditions and an Option (many errors):
        // (I tried putting this in a function, then a macro, but it gets compile errors.
        // So, copy/pasting this, unfortunately, until someone thinks of a better way. (You
        // can see the macro, and one of the compile errors, in the commit of 2023-05-18.
        // I didn't try a proc macro but based on some reading I think it would have the same
        // problem.)
        let mut local_tx: Transaction<Postgres> = {
            if transaction_in.is_none() {
                if caller_manages_transactions_in {
                    return Err(anyhow!(
                        "Inconsistent values for caller_manages_transactions_in \
                                and transaction_in: true and None??"
                            .to_string()
                    ));
                } else {
                    self.begin_trans()?
                }
            } else {
                if caller_manages_transactions_in {
                    // That means we have determined that the caller is to use the transaction_in .
                    // was just:  None
                    // But now instead, create it anyway, per comment above.
                    self.begin_trans()?
                } else {
                    return Err(anyhow!(
                        "In create_relation_to_local_entity, inconsistent values for caller_manages_transactions_in & transaction_in: \
                                false and Some??"
                            .to_string(),
                    ));
                }
            }
        };
        let local_tx_option = &Some(&mut local_tx);
        let transaction: &Option<&mut Transaction<Postgres>> = if caller_manages_transactions_in {
            transaction_in
        } else {
            local_tx_option
        };
        //END OF COPY/PASTED/DUPLICATED BLOCK----------------------------------

        let rte_id: i64 = self.get_new_key(&transaction, "RelationToRemoteEntityKeySequence")?;
        // not creating anything in a remote DB, but a local record of a local relation to a remote entity.
        let result = self.add_attribute_sorting_row(
            &transaction,
            entity_id1_in,
            self.get_attribute_form_id(Util::RELATION_TO_REMOTE_ENTITY_TYPE)
                .unwrap(),
            rte_id,
            sorting_index_in,
        );
        if let Err(e) = result {
            // see comments in delete_objects about rollback
            return Err(anyhow!(e));
        }

        let valid_on_date_sql_str = match valid_on_date_in {
            Some(date) => date.to_string(),
            None => "NULL".to_string(),
        };
        let result = self.db_action(&transaction, format!("INSERT INTO RelationToRemoteEntity (id, rel_type_id, entity_id, \
                  entity_id_2, valid_on_date, observation_date, remote_instance_id) VALUES ({},{},{},{},{},{},'{}')",
                      rte_id, relation_type_id_in, entity_id1_in, entity_id2_in,
                      valid_on_date_sql_str, observation_date_in, remote_instance_id_in).as_str(), false, false);
        if let Err(e) = result {
            // see comments in delete_objects about rollback
            return Err(anyhow!(e));
        }
        if !caller_manages_transactions_in {
            // see comments at similar location in delete_objects about local_tx
            if let Err(e) = self.commit_trans(local_tx) {
                // see comments in delete_objects about rollback
                return Err(anyhow!(e.to_string()));
            }
        }
        Ok(RelationToRemoteEntity {}) //%%$%%really: self, rte_id, relation_type_id_in, entity_id1_in, remote_instance_id_in, entity_id2_in
    }

    /// Re dates' meanings: see usage notes elsewhere in code (like inside create_tables).
    fn update_relation_to_local_entity(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        old_relation_type_id_in: i64,
        entity_id1_in: i64,
        entity_id2_in: i64,
        new_relation_type_id_in: i64,
        valid_on_date_in: Option<i64>,
        observation_date_in: i64,
    ) -> Result<u64, anyhow::Error> {
        // NOTE: IF ADDING COLUMNS TO WHAT IS UPDATED, SIMILARLY UPDATE caller's update method! (else some fields don't get updated
        // in memory when the db updates, and the behavior gets weird.
        let valid = match valid_on_date_in {
            None => "NULL".to_string(),
            Some(v) => format!("{}", v),
        };
        self.db_action(
            transaction,
            format!(
                "UPDATE RelationToEntity SET (rel_type_id, valid_on_date, observation_date) \
                            = ({},{},{}) where rel_type_id={} and entity_id={} and entity_id_2={}",
                new_relation_type_id_in,
                valid,
                observation_date_in,
                old_relation_type_id_in,
                entity_id1_in,
                entity_id2_in
            )
            .as_str(),
            false,
            false,
        )
    }

    /// Re dates' meanings: see usage notes elsewhere in code (like inside create_tables).
    fn update_relation_to_remote_entity(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        old_relation_type_id_in: i64,
        entity_id1_in: i64,
        remote_instance_id_in: String,
        entity_id2_in: i64,
        new_relation_type_id_in: i64,
        valid_on_date_in: Option<i64>,
        observation_date_in: i64,
    ) -> Result<u64, anyhow::Error> {
        // NOTE: IF ADDING COLUMNS TO WHAT IS UPDATED, SIMILARLY UPDATE caller's update method! (else some fields don't get updated
        // in memory when the db updates, and the behavior gets weird.
        let valid = match valid_on_date_in {
            None => "NULL".to_string(),
            Some(v) => format!("{}", v),
        };
        self.db_action(transaction, format!("UPDATE RelationToRemoteEntity SET (rel_type_id, valid_on_date, observation_date) = \
                      ({},{},{}) where rel_type_id={} and entity_id={} and remote_instance_id='{}' and entity_id_2={}", new_relation_type_id_in,
                                      valid, observation_date_in, old_relation_type_id_in, entity_id1_in, remote_instance_id_in,
                             entity_id2_in).as_str(), false, false)
    }

    /// Takes an RTLE and unlinks it from one local entity, and links it under another instead.
    /// @param sorting_index_in Used because it seems handy (as done in calls to other move methods) to keep it in case one moves many entries: they stay in order.
    /// @return the new RelationToLocalEntity
    fn move_relation_to_local_entity_to_local_entity(
        &self,
        rtle_id_in: i64,
        to_containing_entity_id_in: i64,
        sorting_index_in: i64,
    ) -> Result<RelationToLocalEntity, anyhow::Error> {
        let mut tx = self.begin_trans()?;
        let transaction: &Option<&mut Transaction<Postgres>> = &Some(&mut tx);
        let rte_data: Vec<Option<DataType>> =
            self.get_all_relation_to_local_entity_data_by_id(transaction, rtle_id_in)?;
        // next lines are the same as in move_relation_to_remote_entity_to_local_entity and move_relation_to_group; could maintain them similarly.
        let old_rte_rel_type = get_i64_from_row(&rte_data, 2)?;
        let old_rte_entity_1 = get_i64_from_row(&rte_data, 3)?;
        let old_rte_entity_2 = get_i64_from_row(&rte_data, 4)?;
        let valid_on_date: Option<i64> = match rte_data.get(5) {
            //%%does this work in both cases?? (ie, from fn db_query, to here)
            Some(None) => None,
            Some(Some(DataType::Bigint(i))) => Some(i.clone()),
            _ => {
                return Err(anyhow!(
                "In move_relation_to_local_entity_to_local_entity, Unexpected valid_on_date: {:?}",
                rte_data.get(5)
            ))
            }
        };
        let observed_date = get_i64_from_row(&rte_data, 6)?;
        self.delete_relation_to_local_entity(
            transaction,
            old_rte_rel_type,
            old_rte_entity_1,
            old_rte_entity_2,
        )?;
        let new_rte: RelationToLocalEntity = self.create_relation_to_local_entity(
            transaction,
            old_rte_rel_type,
            to_containing_entity_id_in,
            old_rte_entity_2,
            valid_on_date,
            observed_date,
            Some(sorting_index_in),
            true,
        )?;
        //Something like the next line might have been more efficient than the above code to run, but not to write, given that it adds a complexity about updating
        //the attributesorting table, which might be more tricky in future when something is added to prevent those from being orphaned. The above avoids that or
        //centralizes the question to one place in the code.
        //db_action("UPDATE RelationToEntity SET (entity_id) = ROW(" + new_containing_entity_id_in + ")" + " where id=" + relationToLocalEntityIdIn)

        self.commit_trans(tx)?;
        Ok(new_rte)
    }

    /// See comments on & in method move_relation_to_local_entity_to_local_entity.  Only this one takes an RTRE (stored locally), and instead of linking it inside one local
    /// entity, links it inside another local entity.
    fn move_relation_to_remote_entity_to_local_entity(
        &self,
        remote_instance_id_in: &str,
        relation_to_remote_entity_id_in: i64,
        to_containing_entity_id_in: i64,
        sorting_index_in: i64,
    ) -> Result<RelationToRemoteEntity, anyhow::Error> {
        let mut tx = self.begin_trans()?;
        let transaction: &Option<&mut Transaction<Postgres>> = &Some(&mut tx);
        let rte_data: Vec<Option<DataType>> = self.get_all_relation_to_remote_entity_data_by_id(
            transaction,
            relation_to_remote_entity_id_in,
        )?;
        // next lines are the same as in move_relation_to_local_entity_to_local_entity; could maintain them similarly.
        let old_rte_rel_type = get_i64_from_row(&rte_data, 2)?;
        let old_rte_entity_1 = get_i64_from_row(&rte_data, 3)?;
        let old_rte_entity_2 = get_i64_from_row(&rte_data, 4)?;
        let valid_on_date: Option<i64> = match rte_data.get(5) {
            //%%does this work in both cases?? (ie, from fn db_query, to here)
            Some(None) => None,
            Some(Some(DataType::Bigint(i))) => Some(i.clone()),
            _ => {
                return Err(anyhow!(
                "In move_relation_to_local_entity_to_local_entity, Unexpected valid_on_date: {:?}",
                rte_data.get(5)
            ))
            }
        };
        let observed_date = get_i64_from_row(&rte_data, 6)?;
        self.delete_relation_to_remote_entity(
            transaction,
            old_rte_rel_type,
            old_rte_entity_1,
            remote_instance_id_in,
            old_rte_entity_2,
        )?;
        let new_rte: RelationToRemoteEntity = self.create_relation_to_remote_entity(
            transaction,
            old_rte_rel_type,
            to_containing_entity_id_in,
            old_rte_entity_2,
            valid_on_date,
            observed_date,
            remote_instance_id_in,
            Some(sorting_index_in),
            true,
        )?;
        self.commit_trans(tx)?;
        Ok(new_rte)
    }

    fn create_group(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        name_in: &str,
        allow_mixed_classes_in_group_in: bool, /*%%= false*/
    ) -> Result<i64, anyhow::Error> {
        let name: String = Self::escape_quotes_etc(name_in.to_string());
        let group_id: i64 = self.get_new_key(transaction, "RelationToGroupKeySequence")?;
        let allow_mixed = if allow_mixed_classes_in_group_in {
            "TRUE"
        } else {
            "FALSE"
        };
        self.db_action(
            transaction,
            format!(
                "INSERT INTO grupo (id, name, insertion_date, allow_mixed_classes) \
                         VALUES ({}, '{}', {}, {})",
                group_id,
                name,
                Utc::now().timestamp_millis(),
                allow_mixed
            )
            .as_str(),
            false,
            false,
        )?;
        Ok(group_id)
    }

    /// I.e., make it so the entity has a group in it, which can contain entities.
    // Re dates' meanings: see usage notes elsewhere in code (like inside create_tables).
    fn create_group_and_relation_to_group<'a>(
        &'a self,
        // purpose: see comment in delete_objects
        transaction_in: &Option<&mut Transaction<'a, Postgres>>,
        entity_id_in: i64,
        relation_type_id_in: i64,
        new_group_name_in: &str,
        allow_mixed_classes_in_group_in: bool, /*%%= false*/
        valid_on_date_in: Option<i64>,
        observation_date_in: i64,
        sorting_index_in: Option<i64>,
        // purpose: see comment in delete_objects
        caller_manages_transactions_in: bool, /*%%= false*/
    ) -> Result<(i64, i64), anyhow::Error> {
        //BEGIN COPY/PASTED/DUPLICATED (except "in <fn_name>" below) BLOCK-----------------------------------
        // Try creating a local transaction whether we use it or not, to handle compiler errors
        // about variable moves. I'm not seeing a better way to get around them by just using
        // conditions and an Option (many errors):
        // (I tried putting this in a function, then a macro, but it gets compile errors.
        // So, copy/pasting this, unfortunately, until someone thinks of a better way. (You
        // can see the macro, and one of the compile errors, in the commit of 2023-05-18.
        // I didn't try a proc macro but based on some reading I think it would have the same
        // problem.)
        let mut local_tx: Transaction<Postgres> = {
            if transaction_in.is_none() {
                if caller_manages_transactions_in {
                    return Err(anyhow!("In create_group_and_relation_to_group, inconsistent values for caller_manages_transactions_in \
                                and transaction_in: true and None??"
                        .to_string()));
                } else {
                    self.begin_trans()?
                }
            } else {
                if caller_manages_transactions_in {
                    // That means we have determined that the caller is to use the transaction_in .
                    // was just:  None
                    // But now instead, create it anyway, per comment above.
                    self.begin_trans()?
                } else {
                    return Err(anyhow!(
                        "In create_group_and_relation_to_group, inconsistent values for caller_manages_transactions_in & transaction_in: \
                                false and Some??"
                            .to_string(),
                    ));
                }
            }
        };
        let local_tx_option = &Some(&mut local_tx);
        let transaction: &Option<&mut Transaction<Postgres>> = if caller_manages_transactions_in {
            transaction_in
        } else {
            local_tx_option
        };
        //END OF COPY/PASTED/DUPLICATED BLOCK----------------------------------

        let group_id: i64 = self.create_group(
            transaction,
            new_group_name_in,
            allow_mixed_classes_in_group_in,
        )?;
        let (rtg_id, _) = self.create_relation_to_group(
            transaction,
            entity_id_in,
            relation_type_id_in,
            group_id,
            valid_on_date_in,
            observation_date_in,
            sorting_index_in,
            caller_manages_transactions_in,
        )?;
        if !caller_manages_transactions_in {
            // see comments at similar location in delete_objects about local_tx
            if let Err(e) = self.commit_trans(local_tx) {
                // see comments in delete_objects about rollback
                return Err(anyhow!(e.to_string()));
            }
        }
        Ok((group_id, rtg_id))
    }

    /// I.e., make it so the entity has a relation to a new entity in it.
    /// Re dates' meanings: see usage notes elsewhere in code (like inside create_tables).
    fn create_entity_and_relation_to_local_entity<'a>(
        &'a self,
        // purpose: see comment in delete_objects
        transaction_in: &Option<&mut Transaction<'a, Postgres>>,
        entity_id_in: i64,
        relation_type_id_in: i64,
        new_entity_name_in: &str,
        is_public_in: Option<bool>,
        valid_on_date_in: Option<i64>,
        observation_date_in: i64,
        // purpose: see comment in delete_objects
        caller_manages_transactions_in: bool, /*%% = false*/
    ) -> Result<(i64, i64), anyhow::Error> {
        let name: String = Self::escape_quotes_etc(new_entity_name_in.to_string());

        //BEGIN COPY/PASTED/DUPLICATED (except "in <fn_name>" below) BLOCK-----------------------------------
        // Try creating a local transaction whether we use it or not, to handle compiler errors
        // about variable moves. I'm not seeing a better way to get around them by just using
        // conditions and an Option (many errors):
        // (I tried putting this in a function, then a macro, but it gets compile errors.
        // So, copy/pasting this, unfortunately, until someone thinks of a better way. (You
        // can see the macro, and one of the compile errors, in the commit of 2023-05-18.
        // I didn't try a proc macro but based on some reading I think it would have the same
        // problem.)
        let mut local_tx: Transaction<Postgres> = {
            if transaction_in.is_none() {
                if caller_manages_transactions_in {
                    return Err(anyhow!("In create_entity_and_relation_to_local_entity, inconsistent values for caller_manages_transactions_in \
                                and transaction_in: true and None??"
                        .to_string()));
                } else {
                    self.begin_trans()?
                }
            } else {
                if caller_manages_transactions_in {
                    // That means we have determined that the caller is to use the transaction_in .
                    // was just:  None
                    // But now instead, create it anyway, per comment above.
                    self.begin_trans()?
                } else {
                    return Err(anyhow!(
                        "In create_entity_and_relation_to_local_entity, inconsistent values for caller_manages_transactions_in & transaction_in: \
                                false and Some??"
                            .to_string(),
                    ));
                }
            }
        };
        let local_tx_option = &Some(&mut local_tx);
        let transaction: &Option<&mut Transaction<Postgres>> = if caller_manages_transactions_in {
            transaction_in
        } else {
            local_tx_option
        };
        //END OF COPY/PASTED/DUPLICATED BLOCK----------------------------------

        let new_entity_id: i64 =
            self.create_entity(&transaction, name.as_str(), None, is_public_in)?;
        let _new_rte: RelationToLocalEntity = self.create_relation_to_local_entity(
            transaction_in,
            relation_type_id_in,
            entity_id_in,
            new_entity_id,
            valid_on_date_in,
            observation_date_in,
            None,
            caller_manages_transactions_in,
        )?;
        if !caller_manages_transactions_in {
            // see comments at similar location in delete_objects about local_tx
            if let Err(e) = self.commit_trans(local_tx) {
                // see comments in delete_objects about rollback
                return Err(anyhow!(
                    "In create_entity_and_relation_to_local_entity, {}: ",
                    e.to_string()
                ));
            }
        }
        //%%FIX NEXT LINE
        Ok((new_entity_id, 0)) //%%$%%really: , new_rte.get_id()))
    }

    /// I.e., make it so the entity has a group in it, which can contain entities.
    /// Re dates' meanings: see usage notes elsewhere in code (like inside create_tables).
    /// @return a tuple containing the id and new sorting_index: (id, sorting_index)
    fn create_relation_to_group<'a>(
        &'a self,
        // purpose: see comment in delete_objects
        transaction_in: &Option<&mut Transaction<'a, Postgres>>,
        entity_id_in: i64,
        relation_type_id_in: i64,
        group_id_in: i64,
        valid_on_date_in: Option<i64>,
        observation_date_in: i64,
        sorting_index_in: Option<i64>, /*%%= None*/
        // purpose: see comment in delete_objects
        caller_manages_transactions_in: bool, /*%%= false*/
    ) -> Result<(i64, i64), anyhow::Error> {
        //BEGIN COPY/PASTED/DUPLICATED (except "in <fn_name>" below) BLOCK-----------------------------------
        // Try creating a local transaction whether we use it or not, to handle compiler errors
        // about variable moves. I'm not seeing a better way to get around them by just using
        // conditions and an Option (many errors):
        // (I tried putting this in a function, then a macro, but it gets compile errors.
        // So, copy/pasting this, unfortunately, until someone thinks of a better way. (You
        // can see the macro, and one of the compile errors, in the commit of 2023-05-18.
        // I didn't try a proc macro but based on some reading I think it would have the same
        // problem.)
        let mut local_tx: Transaction<Postgres> = {
            if transaction_in.is_none() {
                if caller_manages_transactions_in {
                    return Err(anyhow!("In create_relation_to_group, inconsistent values for caller_manages_transactions_in \
                                and transaction_in: true and None??"
                        .to_string()));
                } else {
                    self.begin_trans()?
                }
            } else {
                if caller_manages_transactions_in {
                    // That means we have determined that the caller is to use the transaction_in .
                    // was just:  None
                    // But now instead, create it anyway, per comment above.
                    self.begin_trans()?
                } else {
                    return Err(anyhow!(
                        "In create_relation_to_group, inconsistent values for caller_manages_transactions_in & transaction_in: \
                                false and Some??"
                            .to_string(),
                    ));
                }
            }
        };
        let local_tx_option = &Some(&mut local_tx);
        let transaction: &Option<&mut Transaction<Postgres>> = if caller_manages_transactions_in {
            transaction_in
        } else {
            local_tx_option
        };
        //END OF COPY/PASTED/DUPLICATED BLOCK----------------------------------

        let id: i64 = self.get_new_key(transaction, "RelationToGroupKeySequence2")?;
        let sorting_index = {
            let sorting_index: i64 = self.add_attribute_sorting_row(
                transaction,
                entity_id_in,
                self.get_attribute_form_id(Util::RELATION_TO_GROUP_TYPE)
                    .unwrap(),
                id,
                sorting_index_in,
            )?;
            let valid_date = match valid_on_date_in {
                None => "NULL".to_string(),
                Some(d) => d.to_string(),
            };
            self.db_action(transaction, format!("INSERT INTO RelationToGroup (id, entity_id, rel_type_id, group_id, valid_on_date, observation_date) \
                             VALUES ({},{},{},{},{},{})", id, entity_id_in, relation_type_id_in, group_id_in, valid_date, observation_date_in).as_str(),
                                  false, false)?;
            sorting_index
        };
        if !caller_manages_transactions_in {
            // see comments at similar location in delete_objects about local_tx
            if let Err(e) = self.commit_trans(local_tx) {
                // see comments in delete_objects about rollback
                return Err(anyhow!(e.to_string()));
            }
        }
        Ok((id, sorting_index))
    }

    fn update_group(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        group_id_in: i64,
        name_in: String,
        allow_mixed_classes_in_group_in: bool, /*= false*/
        new_entries_stick_to_top_in: bool,     /*= false*/
    ) -> Result<u64, anyhow::Error> {
        let name: String = Self::escape_quotes_etc(name_in);
        let mixed = if allow_mixed_classes_in_group_in {
            "TRUE"
        } else {
            "FALSE"
        };
        let new_at_top = if new_entries_stick_to_top_in {
            "TRUE"
        } else {
            "FALSE"
        };
        self.db_action(
            transaction,
            format!(
                "UPDATE grupo SET (name, allow_mixed_classes, new_entries_stick_to_top) \
                            = ('{}', {}, {}) where id={}",
                name, mixed, new_at_top, group_id_in
            )
            .as_str(),
            false,
            false,
        )
    }

    /// Re dates' meanings: see usage notes elsewhere in code (like inside create_tables).
    fn update_relation_to_group(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        entity_id_in: i64,
        old_relation_type_id_in: i64,
        new_relation_type_id_in: i64,
        old_group_id_in: i64,
        new_group_id_in: i64,
        valid_on_date_in: Option<i64>,
        observation_date_in: i64,
    ) -> Result<u64, anyhow::Error> {
        // NOTE: IF ADDING COLUMNS TO WHAT IS UPDATED, SIMILARLY UPDATE caller's update method! (else some fields don't get updated
        // in memory when the db updates, and the behavior gets weird.
        let valid = match valid_on_date_in {
            None => "NULL".to_string(),
            Some(v) => v.to_string(),
        };
        self.db_action(transaction, format!("UPDATE RelationToGroup SET (rel_type_id, group_id, valid_on_date, observation_date) \
                        = ({}, {}, {},{}) where entity_id={} and rel_type_id={} and group_id={}", new_relation_type_id_in, new_group_id_in,
                        valid, observation_date_in, entity_id_in, old_relation_type_id_in, old_group_id_in).as_str(),
                              false, false)
    }

    /// @param sorting_index_in Used because it seems handy (as done in calls to other move methods) to keep it in case one moves many entries: they stay in order.
    /// @return the new RelationToGroup's id.
    fn move_relation_to_group(
        &self,
        relation_to_group_id_in: i64,
        new_containing_entity_id_in: i64,
        sorting_index_in: i64,
    ) -> Result<i64, anyhow::Error> {
        let mut tx = self.begin_trans()?;
        let transaction: &Option<&mut Transaction<Postgres>> = &Some(&mut tx);
        let rtg_data: Vec<Option<DataType>> =
            self.get_all_relation_to_group_data_by_id(transaction, relation_to_group_id_in)?;

        // next lines are the same as in move_relation_to_local_entity_to_local_entity and its sibling; could maintain them similarly.
        let old_rtg_entity_id = get_i64_from_row(&rtg_data, 2)?;
        let old_rtg_rel_type = get_i64_from_row(&rtg_data, 3)?;
        let old_rtg_group_id = get_i64_from_row(&rtg_data, 4)?;
        let valid_on_date: Option<i64> = match rtg_data.get(5) {
            //%%does this work in both cases?? (ie, from fn db_query, to here)
            Some(None) => None,
            Some(Some(DataType::Bigint(i))) => Some(i.clone()),
            _ => {
                return Err(anyhow!(
                    "In move_relation_to_group, unexpected valid_on_date: {:?}",
                    rtg_data.get(5)
                ))
            }
        };
        let observed_date = get_i64_from_row(&rtg_data, 6)?;

        self.delete_relation_to_group(
            transaction,
            old_rtg_entity_id,
            old_rtg_rel_type,
            old_rtg_group_id,
        )?;
        let (new_rtg_id, _) = self.create_relation_to_group(
            transaction,
            new_containing_entity_id_in,
            old_rtg_rel_type,
            old_rtg_group_id,
            valid_on_date,
            observed_date,
            Some(sorting_index_in),
            true,
        )?;

        // (see comment at similar commented line in move_relation_to_local_entity_to_local_entity)
        //db_action("UPDATE RelationToGroup SET (entity_id) = ROW(" + new_containing_entity_id_in + ")" + " where id=" + relation_to_group_id_in)

        self.commit_trans(tx)?;
        Ok(new_rtg_id)
    }

    /// Trying it out with the entity's previous sorting_index (or whatever is passed in) in case it's more convenient, say, when brainstorming a
    /// list then grouping them afterward, to keep them in the same order.  Might be better though just to put them all at the beginning or end; can see....
    fn move_local_entity_from_group_to_group(
        &self,
        from_group_id_in: i64,
        to_group_id_in: i64,
        move_entity_id_in: i64,
        sorting_index_in: i64,
    ) -> Result<(), anyhow::Error> {
        let mut tx = self.begin_trans()?;
        let transaction: &Option<&mut Transaction<Postgres>> = &Some(&mut tx);
        self.add_entity_to_group(
            transaction,
            to_group_id_in,
            move_entity_id_in,
            Some(sorting_index_in),
            true,
        )?;
        self.remove_entity_from_group(transaction, from_group_id_in, move_entity_id_in, true)?;
        if self.is_entity_in_group(transaction, to_group_id_in, move_entity_id_in)?
            && !self.is_entity_in_group(transaction, from_group_id_in, move_entity_id_in)?
        {
            self.commit_trans(tx)
        } else {
            return Err(anyhow!("In move_local_entity_from_group_to_group, Entity didn't get moved properly.  Retry: if predictably reproducible, it should be diagnosed."));
        }
    }

    /// (See comments on moveEntityFromGroupToGroup.)
    fn move_entity_from_group_to_local_entity(
        &self,
        from_group_id_in: i64,
        to_entity_id_in: i64,
        move_entity_id_in: i64,
        sorting_index_in: i64,
    ) -> Result<(), anyhow::Error> {
        let mut tx = self.begin_trans()?;
        let transaction: &Option<&mut Transaction<Postgres>> = &Some(&mut tx);
        self.add_has_relation_to_local_entity(
            transaction,
            to_entity_id_in,
            move_entity_id_in,
            None,
            Utc::now().timestamp_millis(),
            Some(sorting_index_in),
        )?;
        self.remove_entity_from_group(transaction, from_group_id_in, move_entity_id_in, true)?;
        self.commit_trans(tx)
    }
    // //%%$%%
    //          /// (See comments on moveEntityFromGroupToGroup.)
    //        fn move_local_entity_from_local_entity_to_group(&self, removing_rtle_in: RelationToLocalEntity, target_group_id_in: i64, sorting_index_in: i64)
    //              -> Result<(), anyhow::Error> {
    //            let mut tx = self.begin_trans()?;
    //              let transaction: &Option<&mut Transaction<Postgres>> = &Some(&mut tx);
    //            self.add_entity_to_group(transaction, target_group_id_in, removing_rtle_in.getRelatedId2,
    //                                     Some(sorting_index_in), true)?;
    //            self.delete_relation_to_local_entity(transaction, removing_rtle_in.get_attr_type_id(), removing_rtle_in.getRelatedId1,
    //                                                 removing_rtle_in.getRelatedId2)?;
    //            self.commit_trans()
    //          }

    // SEE ALSO METHOD find_unused_attribute_sorting_index **AND DO MAINTENANCE IN BOTH PLACES**
    // idea: this needs a test, and/or combining with findIdWhichIsNotKeyOfAnyEntity.
    // **ABOUT THE SORTINGINDEX:  SEE the related comment on method add_attribute_sorting_row.
    fn find_unused_group_sorting_index(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        group_id_in: i64,
        starting_with_in: Option<i64>, /*%% = None*/
    ) -> Result<i64, anyhow::Error> {
        //better idea?  This should be fast because we start in remote regions and return as soon as an unused id is found, probably
        //only one iteration, ever.  (See similar comments elsewhere.)
        // findUnusedSortingIndex_helper(group_id_in, starting_with_in.getOrElse(max_id_value - 1), 0)
        let g_id = group_id_in;
        let mut working_index = starting_with_in.unwrap_or(self.max_id_value() - 1);
        let mut counter = 0;

        loop {
            //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) w/in this method!
            if self.is_group_entry_sorting_index_in_use(transaction, g_id, working_index)? {
                if working_index == self.max_id_value() {
                    // means we did a full loop across all possible ids!?  Doubtful. Probably would turn into a performance problem long before. It's a bug.
                    return Err(anyhow!(Util::UNUSED_GROUP_ERR1.to_string()));
                }
                // idea: see comment at similar location in findIdWhichIsNotKeyOfAnyEntity
                if counter > 10_000 {
                    return Err(anyhow!(Util::UNUSED_GROUP_ERR2.to_string()));
                }
                working_index = working_index - 1;
                counter = counter + 1;
                continue;
            } else {
                return Ok(working_index);
            }
        }
    }

    // SEE COMMENTS IN find_unused_group_sorting_index **AND DO MAINTENANCE IN BOTH PLACES
    // **ABOUT THE SORTINGINDEX:  SEE the related comment on method add_attribute_sorting_row.
    fn find_unused_attribute_sorting_index(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        entity_id_in: i64,
        starting_with_in: Option<i64>, /*%%= None*/
    ) -> Result<i64, anyhow::Error> {
        let mut working_index = starting_with_in.unwrap_or(self.max_id_value() - 1);
        let mut counter = 0;
        loop {
            if self.is_attribute_sorting_index_in_use(transaction, entity_id_in, working_index)? {
                if working_index == self.max_id_value() {
                    return Err(anyhow!(Util::UNUSED_GROUP_ERR1.to_string()));
                }
                if counter > 10_000 {
                    return Err(anyhow!(Util::UNUSED_GROUP_ERR2.to_string()));
                }
                working_index -= 1;
                counter += 1;
                continue;
            } else {
                return Ok(working_index);
            }
        }
    }

    /// I.e., insert an entity into a group of entities. Using a default value for the sorting_index because user can set it if/as desired;
    /// the max (ie putting it at the end) might be the least often surprising if the user wonders where one went....
    /// **ABOUT THE SORTINGINDEX*:  SEE the related comment on method add_attribute_sorting_row.
    fn add_entity_to_group<'a>(
        &'a self,
        // purpose: see comment in delete_objects
        transaction_in: &Option<&mut Transaction<'a, Postgres>>,
        group_id_in: i64,
        contained_entity_id_in: i64,
        sorting_index_in: Option<i64>, /*%%= None*/
        // purpose: see comment in delete_objects
        caller_manages_transactions_in: bool, /*%%= false*/
    ) -> Result<(), anyhow::Error> {
        // IF THIS CHANGES ALSO DO MAINTENANCE IN SIMILAR METHOD add_attribute_sorting_row

        //BEGIN COPY/PASTED/DUPLICATED (except "anyhow!(\"in <fn_name>" below) BLOCK-----------------------------------
        // Try creating a local transaction whether we use it or not, to handle compiler errors
        // about variable moves. I'm not seeing a better way to get around them by just using
        // conditions and an Option (many errors):
        // (I tried putting this in a function, then a macro, but it gets compile errors.
        // So, copy/pasting this, unfortunately, until someone thinks of a better way. (You
        // can see the macro, and one of the compile errors, in the commit of 2023-05-18.
        // I didn't try a proc macro but based on some reading I think it would have the same
        // problem.)
        let mut local_tx: Transaction<Postgres> = {
            if transaction_in.is_none() {
                if caller_manages_transactions_in {
                    return Err(anyhow!("In add_entity_to_group, inconsistent values for caller_manages_transactions_in \
                                and transaction_in: true and None??"
                        .to_string()));
                } else {
                    self.begin_trans()?
                }
            } else {
                if caller_manages_transactions_in {
                    // That means we have determined that the caller is to use the transaction_in .
                    // was just:  None
                    // But now instead, create it anyway, per comment above.
                    self.begin_trans()?
                } else {
                    return Err(anyhow!(
                        "In add_entity_to_group, inconsistent values for caller_manages_transactions_in & transaction_in: \
                                false and Some??"
                            .to_string(),
                    ));
                }
            }
        };
        let local_tx_option = &Some(&mut local_tx);
        let transaction: &Option<&mut Transaction<Postgres>> = if caller_manages_transactions_in {
            transaction_in
        } else {
            local_tx_option
        };
        //END OF COPY/PASTED/DUPLICATED BLOCK----------------------------------

        // start from the beginning index, if it's the 1st record (otherwise later sorting/renumbering gets messed up if we start w/ the last #):
        let sorting_index: i64 = {
            let index = match sorting_index_in {
                Some(x) => x,
                // start with an increment off the min or max, so that later there is room to sort something before or after it, manually:
                None if self.get_group_size(transaction, group_id_in, 3)? == 0 => {
                    self.min_id_value() + 99999
                }
                _ => self.max_id_value() - 99999,
            };
            let is_in_use: bool =
                self.is_group_entry_sorting_index_in_use(transaction, group_id_in, index)?;
            if is_in_use {
                let find_unused_result: i64 =
                    self.find_unused_group_sorting_index(transaction, group_id_in, None)?;
                find_unused_result
            } else {
                index
            }
        };

        let result = self.db_action(transaction, format!("insert into EntitiesInAGroup (group_id, entity_id, sorting_index) values ({},{},{})",
                          group_id_in, contained_entity_id_in, sorting_index).as_str(), false, false);
        if let Err(s) = result {
            // see comments in delete_objects about rollback
            return Err(anyhow!(s));
        }
        // idea: do this check sooner in this method?:
        let mixed_classes_allowed: bool =
            self.are_mixed_classes_allowed(transaction, &group_id_in)?;
        if !mixed_classes_allowed && self.has_mixed_classes(transaction, &group_id_in)? {
            // see comments in delete_objects about rollback
            return Err(anyhow!(Util::MIXED_CLASSES_EXCEPTION.to_string()));
        }
        if !caller_manages_transactions_in {
            // see comments at similar location in delete_objects about local_tx
            if let Err(e) = self.commit_trans(local_tx) {
                // see comments in delete_objects about rollback
                return Err(anyhow!(e.to_string()));
            }
        }
        Ok(())
    }

    /// Returns the created row's id.
    fn create_entity(
        &self,
        // purpose: see comment in delete_objects
        transaction: &Option<&mut Transaction<Postgres>>,
        name_in: &str,
        class_id_in: Option<i64>,   /*%%= None*/
        is_public_in: Option<bool>, /*%%= None*/
    ) -> Result<i64, anyhow::Error> {
        let name: String = Self::escape_quotes_etc(name_in.to_string());
        if name.is_empty() {
            return Err(anyhow!(
                "In create_entity, name must have a value.".to_string()
            ));
        }
        let id: i64 = self.get_new_key(transaction, "EntityKeySequence")?;
        let maybe_class_id: &str = if class_id_in.is_some() {
            ", class_id"
        } else {
            ""
        };
        let maybe_is_public: &str = match is_public_in {
            None => "NULL",
            Some(b) => {
                if b {
                    "true"
                } else {
                    "false"
                }
            }
        };
        let maybe_class_id_val = match class_id_in {
            Some(id) => format!(",{}", id.to_string()),
            _ => "".to_string(),
        };
        let sql: String = format!(
            "INSERT INTO Entity (id, insertion_date, name, public{}) VALUES ({},{},'{}',{}{})",
            maybe_class_id,
            id,
            Utc::now().timestamp_millis(),
            name,
            maybe_is_public,
            maybe_class_id_val
        );
        self.db_action(transaction, sql.as_str(), false, false)?;
        Ok(id)
    }

    fn create_relation_type<'a>(
        &'a self,
        // purpose: see comment in delete_objects
        caller_manages_transactions_in: bool,
        // purpose: see comment in delete_objects
        transaction_in: &Option<&mut Transaction<'a, Postgres>>,
        name_in: &str,
        name_in_reverse_direction_in: &str,
        directionality_in: &str,
    ) -> Result<i64, anyhow::Error> {
        let name_in_reverse_direction: String =
            Self::escape_quotes_etc(name_in_reverse_direction_in.to_string());
        let name: String = Self::escape_quotes_etc(name_in.to_string());
        let directionality: String = Self::escape_quotes_etc(directionality_in.to_string());
        if name.len() == 0 {
            return Err(anyhow!(
                "In create_relation_type, name must have a value.".to_string()
            ));
        }

        //BEGIN COPY/PASTED/DUPLICATED (except "in <fn_name>" below) BLOCK-----------------------------------
        // Try creating a local transaction whether we use it or not, to handle compiler errors
        // about variable moves. I'm not seeing a better way to get around them by just using
        // conditions and an Option (many errors):
        // (I tried putting this in a function, then a macro, but it gets compile errors.
        // So, copy/pasting this, unfortunately, until someone thinks of a better way. (You
        // can see the macro, and one of the compile errors, in the commit of 2023-05-18.
        // I didn't try a proc macro but based on some reading I think it would have the same
        // problem.)
        let mut local_tx: Transaction<Postgres> = {
            if transaction_in.is_none() {
                if caller_manages_transactions_in {
                    return Err(anyhow!("In create_relation_type, inconsistent values for caller_manages_transactions_in \
                                and transaction_in: true and None??"
                        .to_string()));
                } else {
                    self.begin_trans()?
                }
            } else {
                if caller_manages_transactions_in {
                    // That means we have determined that the caller is to use the transaction_in .
                    // was just:  None
                    // But now instead, create it anyway, per comment above.
                    self.begin_trans()?
                } else {
                    return Err(anyhow!(
                        "In create_relation_type, inconsistent values for caller_manages_transactions_in & transaction_in: \
                                false and Some??"
                            .to_string(),
                    ));
                }
            }
        };
        let local_tx_option = &Some(&mut local_tx);
        let transaction: &Option<&mut Transaction<Postgres>> = if caller_manages_transactions_in {
            transaction_in
        } else {
            local_tx_option
        };
        //END OF COPY/PASTED/DUPLICATED BLOCK----------------------------------

        let mut result: Result<u64, anyhow::Error>;
        let mut id: i64 = 0;
        //see comment at loop in fn create_tables
        loop {
            id = match self.get_new_key(transaction, "EntityKeySequence") {
                Err(s) => {
                    result = Err(anyhow!(s.to_string()));
                    break;
                }
                Ok(i) => i,
            };
            result = self.db_action(
                transaction,
                format!(
                    "INSERT INTO Entity (id, insertion_date, name) VALUES ({},{},'{}')",
                    id,
                    Utc::now().timestamp_millis(),
                    name
                )
                .as_str(),
                false,
                false,
            );
            if result.is_err() {
                break;
            }
            result = self.db_action(transaction,
                                    format!("INSERT INTO RelationType (entity_id, name_in_reverse_direction, directionality) VALUES ({},'{}','{}')",
                                                  id, name_in_reverse_direction, directionality).as_str(), false, false);
            if result.is_err() {
                break;
            }
            if !caller_manages_transactions_in {
                // see comments at similar location in delete_objects about local_tx
                if let Err(e) = self.commit_trans(local_tx) {
                    // see comments in delete_objects about rollback
                    return Err(anyhow!("In create_relation_type (2), {}", e.to_string()));
                }
            }

            // see comment at top of loop
            // see comments in delete_objects about rollback
            break;
        }
        match result {
            Err(e) => Err(anyhow!("In create_relation_type, {}.", e)),
            _ => Ok(id),
        }
    }

    fn delete_entity<'a>(
        &'a self,
        // purpose: see comment in delete_objects
        transaction_in: &Option<&mut Transaction<'a, Postgres>>,
        id_in: i64,
        // purpose: see comment in delete_objects
        caller_manages_transactions_in: bool, /*%%= false*/
    ) -> Result<(), anyhow::Error> {
        // idea: (also on task list i think but) we should not delete entities until dealing with their use as attr_type_ids etc!
        // (or does the DB's integrity constraints do that for us?)

        //BEGIN COPY/PASTED/DUPLICATED (except "in <fn_name>" below) BLOCK-----------------------------------
        // Try creating a local transaction whether we use it or not, to handle compiler errors
        // about variable moves. I'm not seeing a better way to get around them by just using
        // conditions and an Option (many errors):
        // (I tried putting this in a function, then a macro, but it gets compile errors.
        // So, copy/pasting this, unfortunately, until someone thinks of a better way. (You
        // can see the macro, and one of the compile errors, in the commit of 2023-05-18.
        // I didn't try a proc macro but based on some reading I think it would have the same
        // problem.)
        let mut local_tx: Transaction<Postgres> = {
            if transaction_in.is_none() {
                if caller_manages_transactions_in {
                    return Err(anyhow!(
                        "In delete_entity, inconsistent values for caller_manages_transactions_in \
                                and transaction_in: true and None??"
                            .to_string()
                    ));
                } else {
                    self.begin_trans()?
                }
            } else {
                if caller_manages_transactions_in {
                    // That means we have determined that the caller is to use the transaction_in .
                    // was just:  None
                    // But now instead, create it anyway, per comment above.
                    self.begin_trans()?
                } else {
                    return Err(anyhow!(
                        "In delete_entity, inconsistent values for caller_manages_transactions_in & transaction_in: \
                                false and Some??"
                            .to_string(),
                    ));
                }
            }
        };
        let local_tx_option = &Some(&mut local_tx);
        let transaction: &Option<&mut Transaction<Postgres>> = if caller_manages_transactions_in {
            transaction_in
        } else {
            local_tx_option
        };
        //END OF COPY/PASTED/DUPLICATED BLOCK----------------------------------

        self.delete_objects(
            transaction,
            "EntitiesInAGroup",
            format!("where entity_id={}", id_in).as_str(),
            0,
            true,
        )?;
        self.delete_objects(
            transaction,
            Util::ENTITY_TYPE,
            format!("where id={}", id_in).as_str(),
            1,
            true,
        )?;
        self.delete_objects(
            transaction,
            "AttributeSorting",
            format!("where entity_id={}", id_in).as_str(),
            0,
            true,
        )?;
        if !caller_manages_transactions_in {
            // see comments at similar location in delete_objects about local_tx
            if let Err(e) = self.commit_trans(local_tx) {
                // see comments in delete_objects about rollback
                return Err(anyhow!(e.to_string()));
            }
        }
        Ok(())
    }

    fn archive_entity<'a>(
        &'a self,
        transaction: &Option<&mut Transaction<'a, Postgres>>,
        id_in: i64,
        caller_manages_transactions_in: bool, /*= false*/
    ) -> Result<u64, anyhow::Error> {
        self.archive_objects(
            transaction,
            Util::ENTITY_TYPE,
            format!("where id={}", id_in).as_str(),
            1,
            caller_manages_transactions_in,
            false,
        )
    }

    fn unarchive_entity<'a>(
        &'a self,
        transaction: &Option<&mut Transaction<'a, Postgres>>,
        id_in: i64,
        caller_manages_transactions_in: bool, /*= false*/
    ) -> Result<u64, anyhow::Error> {
        self.archive_objects(
            transaction,
            Util::ENTITY_TYPE,
            format!("where id={}", id_in).as_str(),
            1,
            caller_manages_transactions_in,
            true,
        )
    }

    fn delete_quantity_attribute<'a>(
        &'a self,
        transaction: &Option<&mut Transaction<'a, Postgres>>,
        id_in: i64,
    ) -> Result<u64, anyhow::Error> {
        self.delete_object_by_id(transaction, Util::QUANTITY_TYPE, id_in, false)
    }

    fn delete_text_attribute<'a>(
        &'a self,
        transaction: &Option<&mut Transaction<'a, Postgres>>,
        id_in: i64,
    ) -> Result<u64, anyhow::Error> {
        self.delete_object_by_id(transaction, Util::TEXT_TYPE, id_in, false)
    }

    fn delete_date_attribute<'a>(
        &'a self,
        transaction: &Option<&mut Transaction<'a, Postgres>>,
        id_in: i64,
    ) -> Result<u64, anyhow::Error> {
        self.delete_object_by_id(transaction, Util::DATE_TYPE, id_in, false)
    }

    fn delete_boolean_attribute<'a>(
        &'a self,
        transaction: &Option<&mut Transaction<'a, Postgres>>,
        id_in: i64,
    ) -> Result<u64, anyhow::Error> {
        self.delete_object_by_id(transaction, Util::BOOLEAN_TYPE, id_in, false)
    }

    fn delete_file_attribute<'a>(
        &'a self,
        transaction: &Option<&mut Transaction<'a, Postgres>>,
        id_in: i64,
    ) -> Result<u64, anyhow::Error> {
        self.delete_object_by_id(transaction, Util::FILE_TYPE, id_in, false)
    }

    fn delete_relation_to_local_entity<'a>(
        &'a self,
        transaction: &Option<&mut Transaction<'a, Postgres>>,
        rel_type_id_in: i64,
        entity_id1_in: i64,
        entity_id2_in: i64,
    ) -> Result<u64, anyhow::Error> {
        self.delete_objects(
            transaction,
            Util::RELATION_TO_LOCAL_ENTITY_TYPE,
            format!(
                "where rel_type_id={} and entity_id={} and entity_id_2={}",
                rel_type_id_in, entity_id1_in, entity_id2_in
            )
            .as_str(),
            1,
            false,
        )
    }

    fn delete_relation_to_remote_entity<'a>(
        &'a self,
        transaction: &Option<&mut Transaction<'a, Postgres>>,
        rel_type_id_in: i64,
        entity_id1_in: i64,
        remote_instance_id_in: &str,
        entity_id2_in: i64,
    ) -> Result<u64, anyhow::Error> {
        self.delete_objects(transaction, Util::RELATION_TO_REMOTE_ENTITY_TYPE,
                                    format!("where rel_type_id={} and entity_id={} and remote_instance_id='{}' and entity_id_2={}",
                                            rel_type_id_in, entity_id1_in, remote_instance_id_in, entity_id2_in).as_str(),
                                    1, false)
    }

    fn delete_relation_to_group<'a>(
        &'a self,
        transaction: &Option<&mut Transaction<'a, Postgres>>,
        entity_id_in: i64,
        rel_type_id_in: i64,
        group_id_in: i64,
    ) -> Result<u64, anyhow::Error> {
        self.delete_objects(
            transaction,
            Util::RELATION_TO_GROUP_TYPE,
            format!(
                "where entity_id={} and rel_type_id={} and group_id={}",
                entity_id_in, rel_type_id_in, group_id_in
            )
            .as_str(),
            1,
            false,
        )
    }

    fn delete_group_and_relations_to_it(&self, id_in: i64) -> Result<(), anyhow::Error> {
        let mut tx = self.begin_trans()?;
        let transaction: &Option<&mut Transaction<Postgres>> = &Some(&mut tx);
        let entity_count: u64 = self.get_group_size(transaction, id_in, 3)?;
        self.delete_objects(
            transaction,
            "EntitiesInAGroup",
            format!("where group_id={}", id_in).as_str(),
            entity_count,
            true,
        )?;
        let num_groups: u64 = self
            .get_relation_to_group_count_by_group(transaction, id_in)?
            .try_into()?;
        self.delete_objects(
            transaction,
            Util::RELATION_TO_GROUP_TYPE,
            format!("where group_id={}", id_in).as_str(),
            num_groups,
            true,
        )?;
        self.delete_objects(
            transaction,
            "grupo",
            format!("where id={}", id_in).as_str(),
            1,
            true,
        )?;
        self.commit_trans(tx)
    }

    fn remove_entity_from_group<'a>(
        &'a self,
        transaction: &Option<&mut Transaction<'a, Postgres>>,
        group_id_in: i64,
        contained_entity_id_in: i64,
        caller_manages_transactions_in: bool, /*= false*/
    ) -> Result<u64, anyhow::Error> {
        self.delete_objects(
            transaction,
            "EntitiesInAGroup",
            format!(
                "where group_id={} and entity_id={}",
                group_id_in, contained_entity_id_in
            )
            .as_str(),
            1,
            caller_manages_transactions_in,
        )
    }

    /// I hope you have a backup.
    fn delete_group_relations_to_it_and_its_entries(
        &self,
        group_id_in: i64,
    ) -> Result<(), anyhow::Error> {
        let mut tx = self.begin_trans()?;
        let transaction: &Option<&mut Transaction<Postgres>> = &Some(&mut tx);
        let entity_count = self.get_group_size(transaction, group_id_in, 3)?;
        let (deletions1, deletions2) =
            self.delete_relation_to_group_and_all_recursively(transaction, group_id_in)?;
        if deletions1.checked_add(deletions2).unwrap() != entity_count {
            return Err(anyhow!(
                "Not proceeding: deletions1 {} + deletions2 {} != entity_count {}.",
                deletions1,
                deletions2,
                entity_count
            ));
        }
        self.commit_trans(tx)
    }

    fn delete_relation_type<'a>(
        &'a self,
        transaction: &Option<&mut Transaction<'a, Postgres>>,
        id_in: i64,
    ) -> Result<u64, anyhow::Error> {
        // One possibility is that this should ALWAYS fail because it is done by deleting the entity, which cascades.
        // but that's more confusing to the programmer using the database layer's api calls, because they
        // have to know to delete an Entity instead of a RelationType. So we just do the desired thing here
        // instead, and the delete cascades.
        // Maybe those tables should be separated so this is its own thing? for performance/clarity?
        // like *attribute and relation don't have a parent 'attribute' table?  But see comments
        // in create_tables where this one is created.
        self.delete_objects(
            transaction,
            Util::ENTITY_TYPE,
            format!("where id={}", id_in).as_str(),
            1,
            false,
        )
    }

    /// Creates the preference if it doesn't already exist.
    fn set_user_preference_boolean<'a>(
        &'a self,
        transaction: &Option<&mut Transaction<'a, Postgres>>,
        name_in: &str,
        value_in: bool,
    ) -> Result<(), anyhow::Error> {
        let preferences_container_id: i64 = self.get_preferences_container_id(transaction)?;
        let result = self.get_user_preference2(
            transaction,
            preferences_container_id,
            name_in,
            Util::PREF_TYPE_BOOLEAN,
        )?;
        if result.len() > 0 {
            // let preferenceInfo: Option[(i64, Boolean)] = result.asInstanceOf[Option[(i64,Boolean)]];
            //idea: surely there is some better way than what I am doing here? See other places similarly.
            // let DataType::Bigint(preference_attribute_id) = result[0];
            let preference_attribute_id = match result[0] {
                DataType::Bigint(x) => x,
                _ => return Err(anyhow!(format!("How did we get here for {:?}?", result[0]))),
            };

            let mut attribute =
                BooleanAttribute::new2(Box::new(self), transaction, preference_attribute_id)?;
            // Now we have found a boolean attribute which already existed, and just need to
            // update its boolean value. The other values we read from the db inside the first call
            // to something like "get_parent_id()", and just write them back with the new boolean value,
            // to conveniently reuse existing methods.
            self.update_boolean_attribute(
                transaction,
                attribute.get_id(),
                attribute.get_parent_id(transaction)?,
                attribute.get_attr_type_id(transaction)?,
                value_in,
                attribute.get_valid_on_date(transaction)?,
                attribute.get_observation_date(transaction)?,
            )
        } else {
            let type_id_of_the_has_relation =
                self.find_relation_type(transaction, Util::THE_HAS_RELATION_TYPE_NAME)?;
            let preference_entity_id: i64 = self
                .create_entity_and_relation_to_local_entity(
                    transaction,
                    preferences_container_id,
                    type_id_of_the_has_relation,
                    name_in,
                    None,
                    Some(Utc::now().timestamp_millis()),
                    Utc::now().timestamp_millis(),
                    true,
                )?
                .0;
            // (For about the attr_type_id value (2nd parm), see comment about that field, in method get_user_preference_boolean2 below.)
            self.create_boolean_attribute(
                preference_entity_id,
                preference_entity_id,
                value_in,
                Some(Utc::now().timestamp_millis()),
                Utc::now().timestamp_millis(),
                None,
            )?;
            Ok(())
        }
    }
    fn get_user_preference_boolean<'a>(
        &'a self,
        transaction: &Option<&mut Transaction<'a, Postgres>>,
        preference_name_in: &str,
        default_value_in: Option<bool>, /*%%= None*/
    ) -> Result<Option<bool>, anyhow::Error> {
        let pref: Vec<DataType> = self.get_user_preference2(
            transaction,
            self.get_preferences_container_id(transaction)?,
            preference_name_in,
            Util::PREF_TYPE_BOOLEAN,
        )?;
        if pref.len() == 0 {
            Ok(default_value_in)
        } else {
            match pref.get(1) {
                Some(DataType::Boolean(b)) => Ok(Some(b.clone())),
                _ => {
                    return Err(anyhow!(
                        "In get_user_preference_boolean, This shouldn't happen: {:?}",
                        pref
                    ))
                }
            }
        }
    }
    /// Creates the preference if it doesn't already exist.
    fn set_user_preference_entity_id<'a>(
        &'a self,
        transaction: &Option<&mut Transaction<'a, Postgres>>,
        name_in: &str,
        entity_id_in: i64,
    ) -> Result<(), anyhow::Error> {
        let preferences_container_id: i64 = self.get_preferences_container_id(transaction)?;
        let pref: Vec<DataType> = self.get_user_preference2(
            transaction,
            preferences_container_id,
            name_in,
            Util::PREF_TYPE_ENTITY_ID,
        )?;
        if pref.len() == 3 {
            // let preferenceInfo: Option<(i64, i64, i64)> = pref.%%asInstanceOf[Option[(i64,i64,i64)]];
            let relation_type_id = get_i64_from_row_without_option(&pref, 0)?;
            let entity_id1 = get_i64_from_row_without_option(&pref, 1)?;
            let entity_id2 = get_i64_from_row_without_option(&pref, 2)?;
            // didn't bother to put these 2 calls in a transaction because this is likely to be so rarely used and easily fixed by user if
            // it fails (from default entity setting on any entity menu)
            self.delete_relation_to_local_entity(
                transaction,
                relation_type_id,
                entity_id1,
                entity_id2,
            )?;
            // (Using entity_id1 instead of (the likely identical) preferences_container_id, in case this RTE was originally found down among some
            // nested preferences (organized for user convenience) under here, in order to keep that organization.)
            self.create_relation_to_local_entity(
                transaction,
                relation_type_id,
                entity_id1,
                entity_id_in,
                Some(Utc::now().timestamp_millis()),
                Utc::now().timestamp_millis(),
                None,
                false,
            )?;
            Ok(())
        } else if pref.len() == 0 {
            let type_id_of_the_has_relation: i64 =
                self.find_relation_type(transaction, Util::THE_HAS_RELATION_TYPE_NAME)?;
            let preference_entity_id: i64 = self
                .create_entity_and_relation_to_local_entity(
                    transaction,
                    preferences_container_id,
                    type_id_of_the_has_relation,
                    name_in,
                    None,
                    Some(Utc::now().timestamp_millis()),
                    Utc::now().timestamp_millis(),
                    true,
                )?
                .0;
            self.create_relation_to_local_entity(
                transaction,
                type_id_of_the_has_relation,
                preference_entity_id,
                entity_id_in,
                Some(Utc::now().timestamp_millis()),
                Utc::now().timestamp_millis(),
                None,
                true,
            )?;
            Ok(())
        } else {
            Err(anyhow!("Expected 0 or 3, got {}: {:?}", pref.len(), pref))
        }
    }

    fn get_user_preference_entity_id<'a>(
        &'a self,
        transaction: &Option<&mut Transaction<'a, Postgres>>,
        preference_name_in: &str,
        default_value_in: Option<i64>, /*= None*/
    ) -> Result<Option<i64>, anyhow::Error> {
        let pref = self.get_user_preference2(
            transaction,
            self.get_preferences_container_id(transaction)?,
            preference_name_in,
            Util::PREF_TYPE_ENTITY_ID,
        )?;
        if pref.len() == 0 {
            Ok(default_value_in)
        } else if pref.len() == 3 {
            let id = get_i64_from_row_without_option(&pref, 2)?;
            Ok(Some(id))
        } else {
            Err(anyhow!("Unexpected vec size {}: {:?}", pref.len(), pref))
        }
    }

    /// This should never return None, except when method create_and_check_expected_data is called for the first time in a given database.
    fn get_preferences_container_id(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
    ) -> Result<i64, anyhow::Error> {
        let related_entity_id = self.get_relation_to_local_entity_by_name(
            transaction,
            self.get_system_entity_id(transaction)?,
            Util::USER_PREFERENCES,
        )?;
        match related_entity_id {
                    None => return Err(anyhow!("In get_preferences_container_id, This should never happen: method create_and_check_expected_data should be run at startup to create this part of the data.".to_string())),
                    Some(id) => Ok(id),
                }
    }
    fn get_entity_count(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
    ) -> Result<u64, anyhow::Error> {
        let archived = if !self.include_archived_entities {
            "where (not archived)"
        } else {
            ""
        };
        let count: u64 = self
            .extract_row_count_from_count_query(
                transaction,
                format!("SELECT count(1) from Entity {}", archived).as_str(),
            )?
            .try_into()?;
        Ok(count)
    }

    fn get_class_count(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        template_entity_id_in: Option<i64>, /*= None*/
    ) -> Result<u64, anyhow::Error> {
        let where_clause = match template_entity_id_in {
            Some(x) => format!(" where defining_entity_id={}", x),
            _ => "".to_string(),
        };
        let cnt: u64 = self
            .extract_row_count_from_count_query(
                transaction,
                format!("SELECT count(1) from class{}", where_clause).as_str(),
            )?
            .try_into()?;
        Ok(cnt)
    }

    fn get_group_entry_sorting_index(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        group_id_in: i64,
        entity_id_in: i64,
    ) -> Result<i64, anyhow::Error> {
        let row = self.db_query_wrapper_for_one_row(
            transaction,
            format!(
                "select sorting_index from EntitiesInAGroup where group_id={} and \
                                                            entity_id={}",
                group_id_in, entity_id_in
            )
            .as_str(),
            "i64",
        )?;
        match row.get(0) {
            Some(Some(DataType::Bigint(x))) => Ok(x.clone()),
            _ => Err(anyhow!(
                "Unexpected row in get_group_entry_sorting_index: {:?}",
                row
            )),
        }
    }

    fn get_entity_attribute_sorting_index(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        entity_id_in: i64,
        attribute_form_id_in: i64,
        attribute_id_in: i64,
    ) -> Result<i64, anyhow::Error> {
        let row = self.db_query_wrapper_for_one_row(transaction,
                                                            format!("select sorting_index from AttributeSorting where entity_id={} and \
                                                            attribute_form_id={} and attribute_id={}", entity_id_in, attribute_form_id_in,
                                                            attribute_id_in).as_str(),
                                                            "i64")?;
        match row.get(0) {
            Some(Some(DataType::Bigint(x))) => Ok(x.clone()),
            _ => Err(anyhow!(
                "Unexpected row in get_entity_attribute_sorting_index: {:?}",
                row
            )),
        }
    }

    fn get_highest_sorting_index_for_group(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        group_id_in: i64,
    ) -> Result<i64, anyhow::Error> {
        let rows: Vec<Vec<Option<DataType>>> = self.db_query(
            transaction,
            format!(
                "select max(sorting_index) from EntitiesInAGroup where group_id={}",
                group_id_in
            )
            .as_str(),
            "i64",
        )?;
        if rows.len() != 1 || rows[0].len() == 0 || rows[0][0].is_none() {
            return Err(anyhow!("In get_highest_sorting_index_for_group, Unexpected rows ({}) in get_highest_sorting_index_for_group: {:?}", rows.len(), rows));
        }
        match rows[0][0].clone() {
            Some(DataType::Bigint(x)) => Ok(x),
            _ => Err(anyhow!(
                "In get_highest_sorting_index_for_group, expected Some(i64), instead of {:?}.",
                rows[0][0]
            )),
        }
    }

    fn renumber_sorting_indexes<'a>(
        &'a self,
        transaction_in: &Option<&mut Transaction<'a, Postgres>>,
        entity_id_or_group_id_in: i64,
        caller_manages_transactions_in: bool,    /*= false*/
        is_entity_attrs_not_group_entries: bool, /*= true*/
    ) -> Result<(), anyhow::Error> {
        //This used to be called "renumberAttributeSortingIndexes" before it was merged with "renumberGroupSortingIndexes" (very similar).
        let number_of_entries: u64 = {
            if is_entity_attrs_not_group_entries {
                self.get_attribute_count(transaction_in, entity_id_or_group_id_in, true)?
            } else {
                self.get_group_size(transaction_in, entity_id_or_group_id_in, 3)?
                    .into()
            }
        };
        if number_of_entries != 0 {
            // (like a number line so + 1, then add 1 more (so + 2) in case we use up some room on the line due to "attributeSortingIndexInUse" (below))
            let number_of_segments = number_of_entries.checked_add(2).unwrap();
            // ( * 2 on next line, because the min_id_value is negative so there is a larger range to split up, but
            // doing so without exceeding the value of a i64 during the calculation.)
            let increment: i64 =
                (self.max_id_value() as f64 / number_of_segments as f64 * 2.0).round() as i64;
            // (start with an increment so that later there is room to sort something prior to it, manually)
            let mut next: i64 = self.min_id_value().checked_add(increment).unwrap();
            let mut previous: i64 = self.min_id_value();

            //BEGIN COPY/PASTED/DUPLICATED (except "in <fn_name>" below) BLOCK-----------------------------------
            // Try creating a local transaction whether we use it or not, to handle compiler errors
            // about variable moves. I'm not seeing a better way to get around them by just using
            // conditions and an Option (many errors):
            // (I tried putting this in a function, then a macro, but it gets compile errors.
            // So, copy/pasting this, unfortunately, until someone thinks of a better way. (You
            // can see the macro, and one of the compile errors, in the commit of 2023-05-18.
            // I didn't try a proc macro but based on some reading I think it would have the same
            // problem.)
            let mut local_tx: Transaction<Postgres> = {
                if transaction_in.is_none() {
                    if caller_manages_transactions_in {
                        return Err(anyhow!("In renumber_sorting_indexes, inconsistent values for caller_manages_transactions_in \
                                and transaction_in: true and None??"
                    .to_string()));
                    } else {
                        self.begin_trans()?
                    }
                } else {
                    if caller_manages_transactions_in {
                        // That means we have determined that the caller is to use the transaction_in .
                        // was just:  None
                        // But now instead, create it anyway, per comment above.
                        self.begin_trans()?
                    } else {
                        return Err(anyhow!(
                        "In renumber_sorting_indexes, inconsistent values for caller_manages_transactions_in & transaction_in: \
                                false and Some??"
                            .to_string(),
                    ));
                    }
                }
            };
            let local_tx_option = &Some(&mut local_tx);
            let transaction: &Option<&mut Transaction<Postgres>> = if caller_manages_transactions_in
            {
                transaction_in
            } else {
                local_tx_option
            };
            //END OF COPY/PASTED/DUPLICATED BLOCK----------------------------------

            let data: Vec<Vec<Option<DataType>>> = {
                if is_entity_attrs_not_group_entries {
                    self.get_entity_attribute_sorting_data(
                        transaction,
                        entity_id_or_group_id_in,
                        None,
                    )?
                } else {
                    self.get_group_entries_data(transaction, entity_id_or_group_id_in, None, true)?
                }
            };
            if data.len() as u128 != number_of_entries as u128 {
                // "Idea:: BAD SMELL! The UI should do all UI communication, no?"
                // (SEE ALSO comments and code at other places with the part on previous line in quotes).
                // Possible solution: pass a reference to the UI in to here, and use it?
                eprintln!();
                eprintln!();
                eprintln!();
                eprintln!("--------------------------------------");
                eprintln!("Unexpected state: data.size ({}) != number_of_entries ({}), when they should be equal. ", data.len(), number_of_entries);
                if data.len() as u128 > number_of_entries as u128 {
                    eprintln!("Possibly, the database trigger \"attribute_sorting_cleanup\" (created in method create_attribute_sorting_deletion_trigger) \
                            is not always cleaning up when it should or something. ");
                }
                eprintln!("If there is a consistent way to reproduce this from scratch (with attributes of a *new* entity), or other information \
                        to diagnose/improve the situation, please advise.  The program will attempt to continue anyway but a bug around sorting \
                        or placement in this set of entries might result.");
                eprintln!("--------------------------------------")
            }
            for entry in data {
                if is_entity_attrs_not_group_entries {
                    while self.is_attribute_sorting_index_in_use(
                        transaction,
                        entity_id_or_group_id_in,
                        next,
                    )? {
                        // Renumbering might choose already-used numbers, because it always uses the same algorithm.  This causes a constraint violation (unique index)
                        // , so
                        // get around that with a (hopefully quick & simple) increment to get the next unused one.  If they're all used...that's a surprise.
                        // Idea: also fix this bug in the case where it's near the end & the last #s are used: wrap around? when give err after too many loops: count?
                        next += 1
                    }
                } else {
                    while self.is_group_entry_sorting_index_in_use(
                        transaction,
                        entity_id_or_group_id_in,
                        next,
                    )? {
                        next += 1
                    }
                }
                // (make sure a bug didn't cause wraparound w/in the set of possible i64 values)
                if !(previous < next && next < self.max_id_value()) {
                    return Err(anyhow!("In renumber_sorting_indexes, Requirement failed for values previous, next, and max_id_value(): {}, {}, {}", previous, next,
                              self.max_id_value()));
                }
                if is_entity_attrs_not_group_entries {
                    if entry.len() < 2 {
                        return Err(anyhow!(
                            "In renumber_sorting_indexes, unexpected entry length < 2: {:?}",
                            entry
                        ));
                    }
                    let form_id: i64 = match entry[0] {
                        Some(DataType::Bigint(x)) => x,
                        _ => {
                            return Err(anyhow!(
                                "In renumber_sorting_indexes, unexpected entry[0]: {:?}",
                                entry[0]
                            ))
                        }
                    };
                    let attribute_id: i64 = match entry[1] {
                        Some(DataType::Bigint(x)) => x,
                        _ => {
                            return Err(anyhow!(
                                "In renumber_sorting_indexes, unexpected entry[1]: {:?}",
                                entry[1]
                            ))
                        }
                    };
                    self.update_attribute_sorting_index(
                        transaction,
                        entity_id_or_group_id_in,
                        form_id,
                        attribute_id,
                        next,
                    )?;
                } else {
                    // tried this, but no. Is there a smoother way than the way used below & above?
                    // let id: i64;
                    // let DataType::Bigint(id) = entry[0].unwrap_or_else(|| {
                    //     return Err(anyhow!("In renumber_sorting_indexes, another unexpected entry[0]: {:?}", entry[0]))
                    // });
                    let id: i64 = match entry[0] {
                        Some(DataType::Bigint(x)) => x,
                        _ => {
                            return Err(anyhow!(
                            "In renumber_sorting_indexes, yet another unexpected entry[0]: {:?}",
                            entry[0]
                        ))
                        }
                    };
                    self.update_sorting_index_in_a_group(
                        transaction,
                        entity_id_or_group_id_in,
                        id,
                        next,
                    )?;
                }
                previous = next;
                next += increment;
            }

            // assert: just to confirm that the generally expected behavior happened, not a requirement other than that:
            // (didn't happen in case of newly added entries w/ default values....
            // idea: could investigate further...does it matter or imply anything for adding entries to *brand*-newly created groups? Is it related
            // to the fact that when doing that, the 2nd entry goes above, not below the 1st, and to move it down you have to choose the down 1 option
            // *twice* for some reason (sometimes??)? And to the fact that deleting an entry selects the one above, not below, for next highlighting?)
            // (See also a comment somewhere else 4 poss. issue that refers, related, to this method name.)
            // But anyway, if used, do it with a condition and return an error, not panicking.
            //assert((maxIDValue - next) < (increment * 2))

            //%%put this & similar places into a function like self.commit_or_err(tx)?;   ?  If so, include the rollback cmt from just above?
            if !caller_manages_transactions_in {
                // Using local_tx to make the compiler happy and because it is the one we need,
                // if !caller_manages_transactions_in. Ie, there is no transaction provided by
                // the caller.
                if let Err(e) = self.commit_trans(local_tx) {
                    return Err(anyhow!(e.to_string()));
                }
            }
        }
        Ok(())
    }

    /// Excludes those entities that are really relationtypes, attribute types, or quantity units.
    /// The parameter limit_by_class decides whether any limiting is done at all: if true, the query is
    /// limited to entities having the class specified by inClassId (even if that is None).
    /// The parameter template_entity *further* limits, if limit_by_class is true, by omitting the template_entity from the results (ex., to help avoid
    /// counting that one when deciding whether it is OK to delete the class).
    fn get_entities_only_count(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        limit_by_class: bool,         /*= false*/
        class_id_in: Option<i64>,     /*= None*/
        template_entity: Option<i64>, /*= None*/
    ) -> Result<u64, anyhow::Error> {
        let archived = if !self.include_archived_entities {
            "(not archived) and "
        } else {
            ""
        };
        let limit = Self::class_limit(limit_by_class, class_id_in)?;
        let and_id_not = match template_entity {
            Some(s) if limit_by_class => format!(" and id != {}", s),
            _ => "".to_string(),
        };
        let limit2 = Self::limit_to_entities_only(Self::ENTITY_ONLY_SELECT_PART);
        self.extract_row_count_from_count_query(
            transaction,
            format!(
                "SELECT count(1) from Entity e where {} true {}{} \
                            and id in (select id from entity {})",
                archived, limit, and_id_not, limit2
            )
            .as_str(),
        )
    }

    fn get_relation_type_count(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
    ) -> Result<u64, anyhow::Error> {
        self.extract_row_count_from_count_query(transaction, "select count(1) from RelationType")
    }

    fn get_attribute_count(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        entity_id_in: i64,
        include_archived_entities_in: bool, /*%%= false*/
    ) -> Result<u64, anyhow::Error> {
        let total = self
            .get_quantity_attribute_count(transaction, entity_id_in)?
            .checked_add(self.get_text_attribute_count(transaction, entity_id_in)?)
            .unwrap()
            .checked_add(self.get_date_attribute_count(transaction, entity_id_in)?)
            .unwrap()
            .checked_add(self.get_boolean_attribute_count(transaction, entity_id_in)?)
            .unwrap()
            .checked_add(self.get_file_attribute_count(transaction, entity_id_in)?)
            .unwrap()
            .checked_add(self.get_relation_to_local_entity_count(
                transaction,
                entity_id_in,
                include_archived_entities_in,
            )?)
            .unwrap()
            .checked_add(self.get_relation_to_remote_entity_count(transaction, entity_id_in)?)
            .unwrap()
            .checked_add(self.get_relation_to_group_count(transaction, entity_id_in)?)
            .unwrap();
        Ok(total)
    }

    fn get_relation_to_local_entity_count(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        entity_id_in: i64,
        include_archived_entities: bool, /*= true*/
    ) -> Result<u64, anyhow::Error> {
        let appended = if !include_archived_entities && !include_archived_entities {
            " and (not eContained.archived)"
        } else {
            ""
        };
        let sql = format!("select count(1) from entity eContaining, RelationToEntity rte, entity eContained \
            where eContaining.id=rte.entity_id and rte.entity_id={} and rte.entity_id_2=eContained.id{}", entity_id_in, appended);

        self.extract_row_count_from_count_query(transaction, sql.as_str())
    }

    fn get_relation_to_remote_entity_count(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        entity_id_in: i64,
    ) -> Result<u64, anyhow::Error> {
        let sql = format!(
            "select count(1) from entity eContaining, RelationToRemoteEntity rtre \
            where eContaining.id=rtre.entity_id and rtre.entity_id={}",
            entity_id_in
        );
        self.extract_row_count_from_count_query(transaction, sql.as_str())
    }

    /// if 1st parm is None, gets all.
    fn get_relation_to_group_count(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        entity_id_in: i64,
    ) -> Result<u64, anyhow::Error> {
        self.extract_row_count_from_count_query(
            transaction,
            format!(
                "select count(1) from relationtogroup where entity_id={}",
                entity_id_in
            )
            .as_str(),
        )
    }

    //   // Idea: make starting_index_in and max_vals_in do something here.  How was that missed?  Is it needed?
    // fn get_relations_to_group_containing_this_group(&self, transaction: &Option<&mut Transaction<Postgres>>, group_id_in: i64,
    //                                                 starting_index_in: i64, max_vals_in: Option<u64> /*= None*/)
    //        -> Result<Vec<RelationToGroup>, anyhow::Error>  {
    //     let af_id = self.get_attribute_form_id(Util::RELATION_TO_GROUP_TYPE)?;
    //     let sql: &str = format!("select rtg.id, rtg.entity_id, rtg.rel_type_id, rtg.group_id, rtg.valid_on_date, rtg.observation_date, \
    //              asort.sorting_index from RelationToGroup rtg, AttributeSorting asort where group_id={} \
    //              and rtg.entity_id=asort.entity_id and asort.attribute_form_id={} \
    //              and rtg.id=asort.attribute_id", group_id_in, af_id).as_str();
    //     let early_results = self.db_query(transaction, sql, "i64,i64,i64,i64,i64,i64,i64")?;
    //     let mut final_results: Vec<RelationToGroup> = Vec::new();
    //     // idea: should the remainder of this method be moved to RelationToGroup, so the persistence layer doesn't know anything about the Model? (helps avoid
    //     // circular dependencies? is a cleaner design, at least if RTG were in a separate library?)
    //     for result in early_results {
    //       // None of these values should be of "None" type, so not checking for that. If they are it's a bug:
    //       //final_results.add(result(0).get.asInstanceOf[i64], new Entity(this, result(1).get.asInstanceOf[i64]))
    //       let rtg: RelationToGroup = new RelationToGroup(this, result(0).get.asInstanceOf[i64], result(1).get.asInstanceOf[i64],;
    //                                                      result(2).get.asInstanceOf[i64], result(3).get.asInstanceOf[i64],
    //                                                      if result(4).isEmpty) None else Some(result(4).get.asInstanceOf[i64]), result(5).get.asInstanceOf[i64],
    //                                                      result(6).get.asInstanceOf[i64])
    //       final_results.push(rtg)
    //     }
    //     if ! (final_results.len() == early_results.len()) {
    //         return Err(anyhow!("In get_relations_to_group_containing_this_group, Final results ({}) do not match count of early_results ({})", final_results.len(), early_results.len()));
    //     }
    //     Ok(final_results)
    //   }

    fn get_group_count(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
    ) -> Result<u64, anyhow::Error> {
        self.extract_row_count_from_count_query(transaction, "select count(1) from grupo")
    }

    /// @param group_id_in group_id
    /// @param include_which_entities_in 1/2/3 means select onlyNon-archived/onlyArchived/all entities, respectively.
    ///                                4 means "it depends on the value of include_archived_entities", which is what callers want in some cases.
    ///                                This param might be made more clear, but it is not yet clear how is best to do that.
    ///                                  Because the caller provides this switch specifically to the situation, the logic is not necessarily overridden
    ///                                internally based on the value of this.include_archived_entities.
    fn get_group_size(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        group_id_in: i64,
        include_which_entities_in: i32, /*%% = 3*/
    ) -> Result<u64, anyhow::Error> {
        //idea: convert this 1-4 to an enum?
        if include_which_entities_in <= 0 || include_which_entities_in >= 5 {
            return Err(anyhow!(format!("Variable include_which_entities_in ({}) is out of the expected range of 1-4; there is a bug.", include_which_entities_in)));
        }
        let archived_sql_condition: &str = match include_which_entities_in {
            1 => "(not archived)",
            2 => "archived",
            3 => "true",
            4 => {
                if self.include_archived_entities() {
                    "true"
                } else {
                    "(not archived)"
                }
            }
            _ => {
                return Err(anyhow!(format!(
                    "How did we get here? includeWhichEntities={}",
                    include_which_entities_in
                )))
            }
        };
        let count = self.extract_row_count_from_count_query(
            transaction,
            format!(
                "select count(1) from entity e, EntitiesInAGroup \
                eiag where e.id=eiag.entity_id and {} and eiag.group_id={}",
                archived_sql_condition, group_id_in
            )
            .as_str(),
        )?;
        Ok(count)
    }
    /// For all groups to which the parameter belongs, returns a collection of the *containing* RelationToGroups, in the form of "entity_name -> group_name"'s.
    /// This is useful for example when one is about
    /// to delete an entity and we want to warn first, showing where it is contained.
    fn get_containing_relation_to_group_descriptions(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        entity_id_in: i64,
        limit_in: Option<i64>, /*= None*/
    ) -> Result<Vec<String>, anyhow::Error> {
        let omit_archived = if !self.include_archived_entities {
            "(not archived) and "
        } else {
            ""
        };
        let limit = Self::check_if_should_be_all_results(limit_in);
        let rows: Vec<Vec<Option<DataType>>> = self.db_query(transaction,
                                                            format!("select e.name, grp.name, grp.id from entity e, relationtogroup rtg, \
                                                            grupo grp where {} e.id = rtg.entity_id and rtg.group_id = grp.id and rtg.group_id \
                                                            in (SELECT group_id from entitiesinagroup where entity_id={}) \
                                                            order by grp.id limit {}",
                                                            omit_archived, entity_id_in, limit).as_str(),
                                                            "String,String,i64")?;
        let mut results: Vec<String> = Vec::new();
        for row in rows {
            let entity_name = match row.get(0) {
                       Some(Some(DataType::String(x))) => x,
                       _ => return Err(anyhow!("In get_containing_relation_to_group_descriptions, expected an entity name at index 0 of {:?}", row)),
                   };
            let group_name = match row.get(1) {
                       Some(Some(DataType::String(x))) => x,
                       _ => return Err(anyhow!("In get_containing_relation_to_group_descriptions, expected a group name at index 1 of {:?}", row)),
                   };
            results.push(format!("{} -> {}", entity_name, group_name));
        }
        Ok(results)
    }

    /// For a given group, find all the RelationsToGroup that contain entities that contain the provided group id, and return their group_ids.
    /// What is really the best name for this method (concise but clear on what it does)?
    fn get_groups_containing_entitys_groups_ids(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        group_id_in: i64,
        limit_in: Option<i64>, /*= Some(5)*/
    ) -> Result<Vec<Vec<Option<DataType>>>, anyhow::Error> {
        //get every entity that contains a rtg that contains this group:
        let limit = Self::check_if_should_be_all_results(limit_in);
        let containing_entity_id_list: Vec<Vec<Option<DataType>>> =
                   self.db_query(transaction,
                            format!("SELECT entity_id from relationtogroup where group_id={} order by entity_id limit {}", group_id_in, limit).as_str(),
                            "i64")?;
        let mut containing_entity_ids: String = "".to_string();
        //for all those entity ids, get every rtg id containing that entity
        for row in containing_entity_id_list {
            let entity_id = match row.get(0) {
                        Some(Some(DataType::Bigint(x))) => x,
                        _ => return Err(anyhow!("In get_groups_containing_entitys_groups_ids, expected an entity id at index 0 of {:?}", row)),
                    };
            containing_entity_ids = format!("{}{},", containing_entity_ids, entity_id);
        }
        if containing_entity_ids.len() > 0 {
            // remove the last comma
            containing_entity_ids.pop();
            let rtg_rows: Vec<Vec<Option<DataType>>> = self.db_query(
                transaction,
                format!(
                    "SELECT group_id from entitiesinagroup where entity_id in ({}) order \
                                                             by group_id limit {}",
                    containing_entity_ids, limit
                )
                .as_str(),
                "i64",
            )?;
            Ok(rtg_rows)
        } else {
            Ok(Vec::new())
        }
    }

    /// Intended to show something like an activity log. Could be used for someone to show their personal journal or for other reporting.
    fn find_journal_entries(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        start_time_in: i64,
        end_time_in: i64,
        limit_in: Option<i64>, /*= None*/
    ) -> Result<Vec<(i64, String, i64)>, anyhow::Error> {
        let limit = Self::check_if_should_be_all_results(limit_in);
        let rows: Vec<Vec<Option<DataType>>> = self.db_query(transaction,
                                                        format!("select insertion_date, 'Added: ' || name, id from entity where insertion_date >= {}\
                                                         and insertion_date <= {} \
                                                         UNION \
                                                         select archived_date, 'Archived: ' || name, id from entity where archived \
                                                         and archived_date >= {} and archived_date <= {} order by 1 limit {}",
                                                            start_time_in, end_time_in, start_time_in, end_time_in, limit).as_str(),
                                                 "i64,String,i64")?;
        let mut results: Vec<(i64, String, i64)> = Vec::new();
        // let mut n: u64 = 0;
        for row in rows {
            // let DataType::Bigint(date) = row.get(0).ok_or(anyhow!("In find_journal_entries, unexpected date at index 0 in {:?}.", row))?;
            // let DataType::String(desc) = row.get(1).ok_or(anyhow!("In find_journal_entries, unexpected desc at index 1 in {:?}.", row))?;
            // let DataType::Bigint(id) = row.get(2).ok_or(anyhow!("In find_journal_entries, unexpected id at index 2 in {:?}.", row))?;
            let date = match row.get(0) {
                Some(Some(DataType::Bigint(x))) => x,
                _ => {
                    return Err(anyhow!(
                        "In find_journal_entries, expected a date at index 0 of {:?}",
                        row
                    ))
                }
            };
            let desc = match row.get(1) {
                Some(Some(DataType::String(x))) => x,
                _ => {
                    return Err(anyhow!(
                        "In find_journal_entries, expected a desc at index 1 of {:?}",
                        row
                    ))
                }
            };
            let id = match row.get(2) {
                Some(Some(DataType::Bigint(x))) => x,
                _ => {
                    return Err(anyhow!(
                        "In find_journal_entries, expected an id at index 2 of {:?}",
                        row
                    ))
                }
            };
            results.push((date.clone(), desc.clone(), id.clone()));
            // n += 1
        }
        Ok(results)
    }

    fn get_count_of_groups_containing_entity(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        entity_id_in: i64,
    ) -> Result<u64, anyhow::Error> {
        self.extract_row_count_from_count_query(
            transaction,
            format!(
                "select count(1) from EntitiesInAGroup where entity_id={}",
                entity_id_in
            )
            .as_str(),
        )
    }

    fn get_containing_groups_ids(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        entity_id_in: i64,
    ) -> Result<Vec<i64>, anyhow::Error> {
        let group_ids: Vec<Vec<Option<DataType>>> = self.db_query(
            transaction,
            format!(
                "select group_id from EntitiesInAGroup \
                                                                         where entity_id={}",
                entity_id_in
            )
            .as_str(),
            "i64",
        )?;
        let mut results: Vec<i64> = Vec::new();
        for row in group_ids {
            let id = match row.get(0) {
                       Some(Some(DataType::Bigint(id))) =>  id,
                       _ => return Err(anyhow!("In get_containing_groups_ids, expected an entity_id at index 0 instead of {:?}", row)),
                   };
            results.push(id.clone());
        }
        Ok(results)
    }

    fn is_entity_in_group(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        group_id_in: i64,
        entity_id_in: i64,
    ) -> Result<bool, anyhow::Error> {
        let not_archived = if !self.include_archived_entities {
            " and (not e.archived)"
        } else {
            ""
        };
        let num = self.extract_row_count_from_count_query(transaction,
                                                                 format!("select count(1) from EntitiesInAGroup eig, entity e \
                                                                 where eig.entity_id=e.id{} and group_id={} and entity_id={}",
                                                            not_archived, group_id_in, entity_id_in).as_str())?;
        if num > 1 {
            return Err(anyhow!(
                "In is_entity_in_group, Entity {} is in group {} {} times?? Should be 0 or 1.",
                entity_id_in,
                group_id_in,
                num
            ));
        }
        Ok(num == 1)
    }

    fn get_quantity_attribute_data(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        quantity_id_in: i64,
    ) -> Result<Vec<Option<DataType>>, anyhow::Error> {
        let af_id = self.get_attribute_form_id(Util::QUANTITY_TYPE)?;
        self.db_query_wrapper_for_one_row(transaction,
                                                 format!("select qa.entity_id, qa.unit_id, qa.quantity_number, qa.attr_type_id, qa.valid_on_date, \
                                                 qa.observation_date, asort.sorting_index \
                                       from QuantityAttribute qa, AttributeSorting asort where qa.id={} and qa.entity_id=asort.entity_id and \
                                       asort.attribute_form_id={} and qa.id=asort.attribute_id", quantity_id_in, af_id).as_str(),
                                       Util::GET_QUANTITY_ATTRIBUTE_DATA__RESULT_TYPES)
    }

    fn get_relation_to_local_entity_data(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        relation_type_id_in: i64,
        entity_id1_in: i64,
        entity_id2_in: i64,
    ) -> Result<Vec<Option<DataType>>, anyhow::Error> {
        let af_id = self.get_attribute_form_id(Util::RELATION_TO_LOCAL_ENTITY_TYPE)?;
        self.db_query_wrapper_for_one_row(transaction,
                                                 format!("select rte.id, rte.valid_on_date, rte.observation_date, asort.sorting_index \
                                                 from RelationToEntity rte, AttributeSorting asort where rte.rel_type_id={} \
                                                 and rte.entity_id={} and rte.entity_id_2={} and rte.entity_id=asort.entity_id \
                                                 and asort.attribute_form_id={} and rte.id=asort.attribute_id",
                                       relation_type_id_in, entity_id1_in, entity_id2_in, af_id).as_str(),
                                       Util::GET_RELATION_TO_LOCAL_ENTITY__RESULT_TYPES)
    }

    fn get_relation_to_local_entity_data_by_id(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        id_in: i64,
    ) -> Result<Vec<Option<DataType>>, anyhow::Error> {
        let af_id = self.get_attribute_form_id(Util::RELATION_TO_LOCAL_ENTITY_TYPE)?;
        self.db_query_wrapper_for_one_row(transaction,
                                                 format!("select rte.rel_type_id, rte.entity_id, rte.entity_id_2, rte.valid_on_date, \
                                                 rte.observation_date, asort.sorting_index from RelationToEntity rte, AttributeSorting asort \
                                                 where rte.id={} and rte.entity_id=asort.entity_id and asort.attribute_form_id={} and \
                                                 rte.id=asort.attribute_id", id_in, af_id).as_str(),
                                       format!("i64,i64,{}", Util::GET_RELATION_TO_LOCAL_ENTITY__RESULT_TYPES).as_str())
    }

    fn get_relation_to_remote_entity_data(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        relation_type_id_in: i64,
        entity_id1_in: i64,
        remote_instance_id_in: String,
        entity_id2_in: i64,
    ) -> Result<Vec<Option<DataType>>, anyhow::Error> {
        let af_id = self.get_attribute_form_id(Util::RELATION_TO_REMOTE_ENTITY_TYPE)?;
        self.db_query_wrapper_for_one_row(transaction,
                                     format!("select rte.id, rte.valid_on_date, rte.observation_date, asort.sorting_index from RelationToRemoteEntity rte, \
                                     AttributeSorting asort where rte.rel_type_id={} and rte.entity_id={} and rte.remote_instance_id='{}' and rte.entity_id_2={} \
                                      and rte.entity_id=asort.entity_id and asort.attribute_form_id={} and rte.id=asort.attribute_id",
                                             relation_type_id_in, entity_id1_in, remote_instance_id_in, entity_id2_in, af_id).as_str(),
                                       Util::GET_RELATION_TO_REMOTE_ENTITY__RESULT_TYPES)
    }

    fn get_group_data(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        id_in: i64,
    ) -> Result<Vec<Option<DataType>>, anyhow::Error> {
        self.db_query_wrapper_for_one_row(transaction,
                                     format!("select name, insertion_date, allow_mixed_classes, new_entries_stick_to_top from grupo where id={}",
                                             id_in).as_str(),
                                       Util::GET_GROUP_DATA__RESULT_TYPES)
    }

    fn get_relation_to_group_data_by_keys(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        entity_id: i64,
        rel_type_id: i64,
        group_id: i64,
    ) -> Result<Vec<Option<DataType>>, anyhow::Error> {
        let af_id = self.get_attribute_form_id(Util::RELATION_TO_GROUP_TYPE)?;
        self.db_query_wrapper_for_one_row(transaction,
                                  format!("select rtg.id, rtg.entity_id, rtg.rel_type_id, rtg.group_id, rtg.valid_on_date, rtg.observation_date, \
                                  asort.sorting_index from RelationToGroup rtg, AttributeSorting asort where rtg.entity_id={} \
                                   and rtg.rel_type_id={} and rtg.group_id={} and rtg.entity_id=asort.entity_id and asort.attribute_form_id={} \
                                   and rtg.id=asort.attribute_id",
                                   entity_id, rel_type_id, group_id, af_id).as_str(),
                                       Util::GET_RELATION_TO_GROUP_DATA_BY_KEYS__RESULT_TYPES)
    }

    fn get_relation_to_group_data(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        id_in: i64,
    ) -> Result<Vec<Option<DataType>>, anyhow::Error> {
        let af_id = self.get_attribute_form_id(Util::RELATION_TO_GROUP_TYPE)?;
        self.db_query_wrapper_for_one_row(transaction,
                     format!("select rtg.id, rtg.entity_id, rtg.rel_type_id, rtg.group_id, rtg.valid_on_date, rtg.observation_date, \
                     asort.sorting_index from RelationToGroup rtg, AttributeSorting asort where id={} and rtg.entity_id=asort.entity_id and \
                     asort.attribute_form_id={} and rtg.id=asort.attribute_id", id_in, af_id).as_str(),
                                       Util::GET_RELATION_TO_GROUP_DATA_BY_ID__RESULT_TYPES)
    }

    fn get_relation_type_data(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        id_in: i64,
    ) -> Result<Vec<Option<DataType>>, anyhow::Error> {
        let not_archived = if !self.include_archived_entities {
            "(not archived) and "
        } else {
            ""
        };
        self.db_query_wrapper_for_one_row(transaction,
                                format!("select name, name_in_reverse_direction, directionality from RelationType r, Entity e where {} \
                                    e.id=r.entity_id and r.entity_id={}",
                                       not_archived, id_in).as_str(),
                                       Util::GET_RELATION_TYPE_DATA__RESULT_TYPES)
    }

    // idea: combine all the methods that look like this (s.b. easier now, in scala, than java)
    fn get_text_attribute_data(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        text_id_in: i64,
    ) -> Result<Vec<Option<DataType>>, anyhow::Error> {
        let af_id = self.get_attribute_form_id(Util::TEXT_TYPE)?;
        self.db_query_wrapper_for_one_row(transaction,
                             format!("select ta.entity_id, ta.textvalue, ta.attr_type_id, ta.valid_on_date, ta.observation_date, \
                             asort.sorting_index from TextAttribute ta, AttributeSorting asort where id={} and ta.entity_id=asort.entity_id \
                             and asort.attribute_form_id={} and ta.id=asort.attribute_id",
                                 text_id_in, af_id).as_str(),
                                       Util::GET_TEXT_ATTRIBUTE_DATA__RESULT_TYPES)
    }

    fn get_date_attribute_data(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        date_id_in: i64,
    ) -> Result<Vec<Option<DataType>>, anyhow::Error> {
        let af_id = self.get_attribute_form_id(Util::DATE_TYPE)?;
        self.db_query_wrapper_for_one_row(transaction,
                                 format!("select da.entity_id, da.date, da.attr_type_id, asort.sorting_index from DateAttribute da, \
                                 AttributeSorting asort where da.id={} and da.entity_id=asort.entity_id and asort.attribute_form_id={} \
                                  and da.id=asort.attribute_id",
                                 date_id_in, af_id).as_str(),
                                       Util::GET_DATE_ATTRIBUTE_DATA__RESULT_TYPES)
    }

    fn get_boolean_attribute_data(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        boolean_id_in: i64,
    ) -> Result<Vec<Option<DataType>>, anyhow::Error> {
        let form_id = self.get_attribute_form_id(Util::BOOLEAN_TYPE)?;
        self.db_query_wrapper_for_one_row(transaction, format!("select ba.entity_id, ba.booleanValue, ba.attr_type_id, ba.valid_on_date, ba.observation_date, asort.sorting_index \
                                    from BooleanAttribute ba, AttributeSorting asort where id={} and ba.entity_id=asort.entity_id and asort.attribute_form_id={} \
                                     and ba.id=asort.attribute_id",
                                                      boolean_id_in, form_id).as_str(),
                                    Util::GET_BOOLEAN_ATTRIBUTE_DATA__RESULT_TYPES)
    }

    fn get_file_attribute_data(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        file_id_in: i64,
    ) -> Result<Vec<Option<DataType>>, anyhow::Error> {
        let af_id = self.get_attribute_form_id(Util::FILE_TYPE)?;
        self.db_query_wrapper_for_one_row(transaction,
                             format!("select fa.entity_id, fa.description, fa.attr_type_id, fa.original_file_date, fa.stored_date, \
                             fa.original_file_path, fa.readable, fa.writable, fa.executable, fa.size, fa.md5hash, asort.sorting_index \
                              from FileAttribute fa, AttributeSorting asort where id={} and fa.entity_id=asort.entity_id and asort.attribute_form_id={} \
                               and fa.id=asort.attribute_id",
                               file_id_in, af_id).as_str(),
                                   Util::GET_FILE_ATTRIBUTE_DATA__RESULT_TYPES)
    }

    fn update_sorting_index_in_a_group(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        group_id_in: i64,
        entity_id_in: i64,
        sorting_index_in: i64,
    ) -> Result<u64, anyhow::Error> {
        self.db_action(transaction,
                  format!("update EntitiesInAGroup set (sorting_index) = ROW({}) where group_id={} and entity_id={}",
                      sorting_index_in, group_id_in, entity_id_in).as_str(),
                          false, false)
    }

    fn update_attribute_sorting_index(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        entity_id_in: i64,
        attribute_form_id_in: i64,
        attribute_id_in: i64,
        sorting_index_in: i64,
    ) -> Result<u64, anyhow::Error> {
        self.db_action(transaction,
                  format!("update AttributeSorting set (sorting_index) = ROW({}) where entity_id={} and attribute_form_id={} and attribute_id={}",
                          sorting_index_in, entity_id_in, attribute_form_id_in, attribute_id_in).as_str(),
                          false, false)
    }

    //   /// Returns whether the stored and calculated md5hashes match, and an error message when they don't.
    // fn verify_file_attribute_content_integrity(fileAttributeIdIn: i64) -> (Boolean, Option<String>) {
    //     // Idea: combine w/ similar logic in FileAttribute.md5Hash?
    //     // Idea: compare actual/stored file sizes also? or does the check of md5 do enough as is?
    //     // Idea (tracked in tasks): switch to some SHA algorithm since they now say md5 is weaker?
    //     let messageDigest = java.security.MessageDigest.getInstance("MD5");
    //     fn action(bufferIn: Array[Byte], starting_index_in: Int, numBytesIn: Int) {
    //       messageDigest.update(bufferIn, starting_index_in, numBytesIn)
    //     }
    //     // Next line calls "action" (probably--see javadoc for java.security.MessageDigest for whatever i was thinking at the time)
    //     // to prepare messageDigest for the digest method to get the md5 value:
    //     let storedMd5Hash = act_on_file_from_server(fileAttributeIdIn, action)._2;
    //     //noinspection LanguageFeature ...It is a style violation (advanced feature) but it's what I found when searching for how to do it.
    //     // outputs same as command 'md5sum <file>'.
    //     let md5hash: String = messageDigest.digest.map(0xFF &).map {"%02x".format(_)}.foldLeft("") {_ + _};
    //     if md5hash == storedMd5Hash) (true, None)
    //     else {
    //       (false, Some("Mismatched md5hashes: " + storedMd5Hash + " (stored in the md5sum db column) != " + md5hash + "(calculated from stored file contents)"))
    //     }
    //   }

    // /** This is a no-op, called in act_on_file_from_server, that a test can customize to simulate a corrupted file on the server. */
    // //noinspection ScalaUselessExpression (...intentional style violation, for readability)
    //   fn damageBuffer(buffer: Array[Byte]) /* -> Unit = Unit%%*/
    //   /// Returns the file size (having confirmed it is the same as the # of bytes processed), and the md5hash that was stored with the document.
    // fn act_on_file_from_server(fileAttributeIdIn: i64, actionIn: (Array[Byte], Int, Int) => Unit) -> (i64, String) {
    //     let mut obj: LargeObject = null;
    //     try {
    //       // even though we're not storing data, the instructions (see create_tables re this...) said to have it in a transaction.
    //       self.begin_trans()
    //       let lobjManager: LargeObjectManager = connection.asInstanceOf[org.postgresql.PGConnection].getLargeObjectAPI;
    //       let oidOption: Option<i64> = db_query_wrapper_for_one_row("select contents_oid from FileAttributeContent where file_attribute_id=" + fileAttributeIdIn,;
    //                                                             "i64")(0).asInstanceOf[Option<i64>]
    //       if oidOption.isEmpty) throw new OmDatabaseException("No contents found for file attribute id " + fileAttributeIdIn)
    //       let oid: i64 = oidOption.get;
    //       obj = lobjManager.open(oid, LargeObjectManager.READ)
    //       // Using 4096 only because this url:
    //       //   https://commons.apache.org/proper/commons-io/javadocs/api-release/org/apache/commons/io/IOUtils.html
    //       // ...said, at least for that purpose, that: "The default buffer size of 4K has been shown to be efficient in tests." (retrieved 2016-12-05)
    //       let buffer = new Array[Byte](4096);
    //       let mut numBytesRead = 0;
    //       let mut total: i64 = 0;
    //       @tailrec
    //       fn readFileFromDbAndActOnIt() {
    //         //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) w/in this method!
    //         numBytesRead = obj.read(buffer, 0, buffer.length)
    //         // (intentional style violation, for readability):
    //         //noinspection ScalaUselessExpression
    //         if numBytesRead <= 0) Unit
    //         else {
    //           // just once by a test subclass is enough to mess w/ the md5sum.
    //           if total == 0) damageBuffer(buffer)
    //
    //           actionIn(buffer, 0, numBytesRead)
    //           total += numBytesRead
    //           readFileFromDbAndActOnIt()
    //         }
    //       }
    //       readFileFromDbAndActOnIt()
    //       let resultOption = db_query_wrapper_for_one_row("select size, md5hash from fileattribute where id=" + fileAttributeIdIn, "i64,String");
    //       if resultOption(0).isEmpty) throw new OmDatabaseException("No result from query for fileattribute for id " + fileAttributeIdIn + ".")
    //       let (contentSize, md5hash) = (resultOption(0).get.asInstanceOf[i64], resultOption(1).get.asInstanceOf[String]);
    //       if total != contentSize) {
    //         throw new OmFileTransferException("Transferred " + total + " bytes instead of " + contentSize + "??")
    //       }
    //       commit_trans()
    //       (total, md5hash)
    //     } catch {
    //       case e: Exception => throw rollbackWithCatch(e)
    //     } finally {
    //       try {
    //         obj.close()
    //       } catch {
    //         case e: Exception =>
    //         // not sure why this fails sometimes, if it's a bad thing or not, but for now not going to be stuck on it.
    //         // idea: look at the source code.
    //       }
    //     }
    //   }

    fn quantity_attribute_key_exists(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        id_in: i64,
    ) -> Result<bool, anyhow::Error> {
        self.does_this_exist(
            transaction,
            format!("SELECT count(1) from QuantityAttribute where id={}", id_in).as_str(),
            true,
        )
    }

    fn text_attribute_key_exists(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        id_in: i64,
    ) -> Result<bool, anyhow::Error> {
        self.does_this_exist(
            transaction,
            format!("SELECT count(1) from TextAttribute where id={}", id_in).as_str(),
            true,
        )
    }

    fn date_attribute_key_exists(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        id_in: i64,
    ) -> Result<bool, anyhow::Error> {
        self.does_this_exist(
            transaction,
            format!("SELECT count(1) from DateAttribute where id={}", id_in).as_str(),
            true,
        )
    }

    fn boolean_attribute_key_exists(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        id_in: i64,
    ) -> Result<bool, anyhow::Error> {
        self.does_this_exist(
            transaction,
            format!("SELECT count(1) from BooleanAttribute where id={}", id_in).as_str(),
            true,
        )
    }

    fn file_attribute_key_exists(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        id_in: i64,
    ) -> Result<bool, anyhow::Error> {
        self.does_this_exist(
            transaction,
            format!("SELECT count(1) from FileAttribute where id={}", id_in).as_str(),
            true,
        )
    }

    fn relation_to_local_entity_key_exists(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        id_in: i64,
    ) -> Result<bool, anyhow::Error> {
        self.does_this_exist(
            transaction,
            format!("SELECT count(1) from RelationToEntity where id={}", id_in).as_str(),
            true,
        )
    }

    fn relation_to_remote_entity_key_exists(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        id_in: i64,
    ) -> Result<bool, anyhow::Error> {
        self.does_this_exist(
            transaction,
            format!(
                "SELECT count(1) from RelationToRemoteEntity where id={}",
                id_in
            )
            .as_str(),
            true,
        )
    }

    fn relation_to_group_key_exists(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        id_in: i64,
    ) -> Result<bool, anyhow::Error> {
        self.does_this_exist(
            transaction,
            format!("SELECT count(1) from RelationToGroup where id={}", id_in).as_str(),
            true,
        )
    }

    fn attribute_key_exists(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        form_id_in: i64,
        id_in: i64,
    ) -> Result<bool, anyhow::Error> {
        //MAKE SURE THESE MATCH WITH THOSE IN get_attribute_form_id !
        match form_id_in {
            1 => self.relation_type_key_exists(transaction, id_in),
            2 => self.date_attribute_key_exists(transaction, id_in),
            3 => self.boolean_attribute_key_exists(transaction, id_in),
            4 => self.file_attribute_key_exists(transaction, id_in),
            5 => self.text_attribute_key_exists(transaction, id_in),
            6 => self.relation_to_local_entity_key_exists(transaction, id_in),
            7 => self.relation_to_group_key_exists(transaction, id_in),
            8 => self.relation_to_remote_entity_key_exists(transaction, id_in),
            _ => Err(anyhow!("unexpected")),
        }
    }

    /// @param include_archived See comment on similar parameter to method get_group_size.
    //idea: see if any callers should pass the include_archived parameter differently, now that the system can be used with archived entities displayed.
    fn entity_key_exists(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        id_in: i64,
        include_archived: bool,
    ) -> Result<bool, anyhow::Error> {
        let condition = if !include_archived {
            " and not archived"
        } else {
            ""
        };
        self.does_this_exist(
            transaction,
            format!(
                "SELECT count(1) from Entity where id={}{}",
                id_in, condition
            )
            .as_str(),
            true,
        )
    }

    fn is_group_entry_sorting_index_in_use(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        group_id_in: i64,
        sorting_index_in: i64,
    ) -> Result<bool, anyhow::Error> {
        self.does_this_exist(
            transaction,
            format!(
                "SELECT count(1) from Entitiesinagroup where group_id={} and sorting_index={}",
                group_id_in, sorting_index_in
            )
            .as_str(),
            true,
        )
    }

    fn class_key_exists(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        id_in: i64,
    ) -> Result<bool, anyhow::Error> {
        self.does_this_exist(
            transaction,
            format!("SELECT count(1) from class where id={}", id_in).as_str(),
            true,
        )
    }

    fn relation_type_key_exists(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        id_in: i64,
    ) -> Result<bool, anyhow::Error> {
        self.does_this_exist(
            transaction,
            format!(
                "SELECT count(1) from RelationType where entity_id={}",
                id_in
            )
            .as_str(),
            true,
        )
    }

    fn relation_to_local_entity_keys_exist_and_match(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        id_in: i64,
        rel_type_id_in: i64,
        entity_id1_in: i64,
        entity_id2_in: i64,
    ) -> Result<bool, anyhow::Error> {
        self.does_this_exist(
            transaction,
            format!(
                "SELECT count(1) from RelationToEntity where id={} and rel_type_id={} and \
                           entity_id={} and entity_id_2={}",
                id_in, rel_type_id_in, entity_id1_in, entity_id2_in
            )
            .as_str(),
            true,
        )
    }

    fn relation_to_remote_entity_keys_exist_and_match(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        id_in: i64,
        rel_type_id_in: i64,
        entity_id1_in: i64,
        remote_instance_id_in: String,
        entity_id2_in: i64,
    ) -> Result<bool, anyhow::Error> {
        self.does_this_exist(transaction, format!("SELECT count(1) from RelationToRemoteEntity where id={} and rel_type_id={} \
                                                     and entity_id={} and remote_instance_id='{}' and entity_id_2={}",
                                                         id_in, rel_type_id_in, entity_id1_in, remote_instance_id_in, entity_id2_in).as_str(),
                                                true)
    }

    fn group_key_exists(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        id_in: i64,
    ) -> Result<bool, anyhow::Error> {
        self.does_this_exist(
            transaction,
            format!("SELECT count(1) from grupo where id={}", id_in).as_str(),
            true,
        )
    }

    fn relation_to_group_keys_exist_and_match(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        id: i64,
        entity_id: i64,
        rel_type_id: i64,
        group_id: i64,
    ) -> Result<bool, anyhow::Error> {
        self.does_this_exist(transaction, format!("SELECT count(1) from RelationToGroup where id={} and entity_id={} and rel_type_id={} \
                             and group_id={}",
                                            id, entity_id, rel_type_id, group_id).as_str(), true)
    }

    /// Allows querying for a range of objects in the database; returns a java.util.Map with keys and names.
    /// 1st parm is index to start with (0-based), 2nd parm is # of obj's to return (if None, means no limit).
    fn get_entities(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        starting_object_index_in: i64,
        max_vals_in: Option<i64>, /*= None*/
    ) -> Result<Vec<Entity>, anyhow::Error> {
        self.get_entities_generic(
            transaction,
            starting_object_index_in,
            max_vals_in,
            Util::ENTITY_TYPE,
            None,
            false,
            None,
            None,
        )
    }

    /// Excludes those entities that are really relationtypes, attribute types, or quantity units. Otherwise similar to get_entities.
    /// *****NOTE*****: The limit_by_class:Boolean parameter is not redundant with the inClassId: inClassId could be None and we could still want
    /// to select only those entities whose class_id is NULL, such as when enforcing group uniformity (see method has_mixed_classes and its
    /// uses, for more info).
    ///
    /// The parameter omitEntity is (at this writing) used for the id of a class-defining (template) entity, which we shouldn't show for editing when showing all the
    /// entities in the class (editing that is a separate menu option), otherwise it confuses things.
    fn get_entities_only(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        starting_object_index_in: i64,
        max_vals_in: Option<i64>,         /*= None*/
        class_id_in: Option<i64>,         /*= None*/
        limit_by_class: bool,             /*= false*/
        template_entity: Option<i64>,     /*= None*/
        group_to_omit_id_in: Option<i64>, /*= None*/
    ) -> Result<Vec<Entity>, anyhow::Error> {
        self.get_entities_generic(
            transaction,
            starting_object_index_in,
            max_vals_in,
            "EntityOnly",
            class_id_in,
            limit_by_class,
            template_entity,
            group_to_omit_id_in,
        )
    }

    /// similar to get_entities
    fn get_relation_types(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        starting_object_index_in: i64,
        max_vals_in: Option<i64>, /*= None*/
    ) -> Result<Vec<Entity>, anyhow::Error> {
        self.get_entities_generic(
            transaction,
            starting_object_index_in,
            max_vals_in,
            Util::RELATION_TYPE_TYPE,
            None,
            false,
            None,
            None,
        )
    }

    fn get_matching_entities(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        starting_object_index_in: i64,
        max_vals_in: Option<i64>, /*= None*/
        omit_entity_id_in: Option<i64>,
        name_regex_in: String,
    ) -> Result<Vec<Entity>, anyhow::Error> {
        let select_columns = Util::SELECT_ENTITY_START;
        let name_regex = Self::escape_quotes_etc(name_regex_in);
        let omission_expression = match omit_entity_id_in {
            Some(id) => format!("(not id={})", id),
            None => "true".to_string(),
        };
        let not_archived = if !self.include_archived_entities {
            "not archived and "
        } else {
            ""
        };
        let limit = Self::check_if_should_be_all_results(max_vals_in);
        let sql = format!("{} from entity e where {}{} and name ~* '{}' \
                                UNION \
                                select id, name, class_id, insertion_date, public, archived, new_entries_stick_to_top from entity where {}{} \
                                 and id in (select entity_id from textattribute where textvalue ~* '{}') ORDER BY id limit {} offset {}",
                                 select_columns, not_archived, omission_expression, name_regex,
                                 not_archived, omission_expression, name_regex, limit, starting_object_index_in);
        let early_results = self.db_query(
            transaction,
            sql.as_str(),
            "i64,String,i64,i64,bool,bool,bool",
        )?;
        let early_results_len = early_results.len();
        let final_results: Vec<Entity> = Vec::new();
        // idea: (see get_entities_generic for an idea, see if it applies here)
        for _result in early_results {
            //%%$%% add_new_entity_to_results(final_results, result)
        }
        if !(final_results.len() == early_results_len) {
            return Err(anyhow!(
                "final_results.len() ({}) != early_results.len() ({}).",
                final_results.len(),
                early_results_len
            ));
        }
        Ok(final_results)
    }

    // fn get_matching_groups(&self, transaction: &Option<&mut Transaction<Postgres>>, starting_object_index_in: i64,
    //                        max_vals_in: Option<i64> /*= None*/, omit_group_id_in: Option<i64>,
    //                         name_regex_in: String) -> Result<Vec<Group>, anyhow::Error> {
    //     let name_regex = self.escape_quotes_etc(name_regex_in);
    //     let omission_expression = match omit_group_id_in {
    //         None => "true",
    //         Some(ogi) => format!("(not id={})", ogi).as_str(),
    //     };
    //     let sql = format!("select id, name, insertion_date, allow_mixed_classes, new_entries_stick_to_top from grupo where name ~* '{}' and {} \
    //                   order by id limit {} offset {}",
    //                     name_regex, omission_expression, Self::check_if_should_be_all_results(max_vals_in), starting_object_index_in).as_str();
    //     let early_results = self.db_query(transaction, sql, "i64,String,i64,bool,bool")?;
    //     let final_results: Vec<Group> = Vec::new();
    //     // idea: (see get_entities_generic for idea, see if applies here)
    //     for result in early_results {
    //       // None of these values should be of "None" type, so not checking for that. If they are it's a bug:
    //         //%%$%%
    //       // final_results.add(new Group(this, result(0).get.asInstanceOf[i64], result(1).get.asInstanceOf[String], result(2).get.asInstanceOf[i64],
    //       //                            result(3).get.asInstanceOf[Boolean], result(4).get.asInstanceOf[Boolean]))
    //     }
    //     if (final_results.len() != early_results.len()) {
    //         return Err(anyhow!("In get_matching_groups, final_results.len() ({}) != early_results.len() ({})", final_results.len(), early_results.len()));
    //     }
    //     Ok(final_results)
    //   }

    fn get_local_entities_containing_local_entity(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        entity_id_in: i64,
        starting_index_in: i64,
        max_vals_in: Option<i64>, /*= None*/
    ) -> Result<Vec<(i64, Entity)>, anyhow::Error> {
        let not_archived = if !self.include_archived_entities {
            " and (not e.archived)"
        } else {
            ""
        };
        let limit = Self::check_if_should_be_all_results(max_vals_in);
        let sql = format!("select rel_type_id, entity_id from relationtoentity rte, entity e where rte.entity_id=e.id and \
                            rte.entity_id_2={} {} order by entity_id limit {} offset {}",
                            entity_id_in, not_archived, limit, starting_index_in);
        //note/idea: this should be changed when we update relation stuff similarly, to go both ways in the relation (either entity_id or
        // entity_id_2: helpfully returned; & in UI?)
        self.get_containing_entities_helper(transaction, sql.as_str())
    }

    fn get_entities_containing_group(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        group_id_in: i64,
        starting_index_in: i64,
        max_vals_in: Option<i64>, /*= None*/
    ) -> Result<Vec<(i64, Entity)>, anyhow::Error> {
        let sql = format!("select rel_type_id, entity_id from relationtogroup where group_id={}  order by entity_id, rel_type_id \
                                    limit {} offset {}",
                                    group_id_in, Self::check_if_should_be_all_results(max_vals_in), starting_index_in);
        //note/idea: this should be changed when we update relation stuff similarly, to go both ways in the relation (either entity_id or
        // entity_id_2: helpfully returned; & in UI?)
        //And, perhaps changed to account for whether something is archived.
        // See get_count_of_entities_containing_group for example.
        self.get_containing_entities_helper(transaction, sql.as_str())
    }

    /// @return A tuple showing the # of non-archived entities and the # of archived entities that directly refer to this entity (IN *ONE* DIRECTION ONLY).
    fn get_count_of_local_entities_containing_local_entity(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        entity_id_in: i64,
    ) -> Result<(u64, u64), anyhow::Error> {
        let non_archived2 = self.extract_row_count_from_count_query(
            transaction,
            format!(
                "select count(1) from relationtoentity rte, entity e where e.id=rte.entity_id_2 \
                                                                and not e.archived and e.id={}",
                entity_id_in
            )
            .as_str(),
        )?;
        let archived2 = self.extract_row_count_from_count_query(transaction, format!("select count(1) from \
                                relationtoentity rte, entity e where e.id=rte.entity_id_2 and e.archived and e.id={}", entity_id_in).as_str())?;

        Ok((non_archived2, archived2))
    }

    /// @return A tuple showing the # of non-archived entities and the # of archived entities that directly refer to this group.
    fn get_count_of_entities_containing_group(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        group_id_in: i64,
    ) -> Result<(u64, u64), anyhow::Error> {
        let non_archived = self.extract_row_count_from_count_query(transaction, format!("select count(1) from \
                                relationtogroup rtg, entity e where e.id=rtg.entity_id and not e.archived and rtg.group_id={}", group_id_in).as_str())?;
        let archived = self.extract_row_count_from_count_query(transaction, format!("select count(1) from \
                                relationtogroup rtg, entity e where e.id=rtg.entity_id and e.archived and rtg.group_id={}", group_id_in).as_str())?;
        Ok((non_archived, archived))
    }

    // fn get_containing_relations_to_group(&self, transaction: &Option<&mut Transaction<Postgres>>, entity_id_in: i64, starting_index_in: i64,
    //                                      max_vals_in: Option<i64> /*= None*/)  -> Result<Vec<RelationToGroup>, anyhow::Error>  {
    //     // BUG (tracked in tasks): there is a disconnect here between this method and its _helper method, because one uses the eig table, the other the rtg table,
    //     // and there is no requirement/enforcement that all groups defined in eig are in an rtg, so they could get dif't/unexpected results.
    //     // So, could: see the expectation of the place(s) calling this method, if uniform, make these 2 methods more uniform in what they do in meeting that,
    //     // OR, could consider whether we really should have an enforcement between the 2 tables...?
    //     // THIS BUg currently prevents searching for then deleting the entity w/ this in name: "OTHER ENTITY NOTED IN A DELETION BUG" (see also other issue
    //     // in Controller.java where that same name is mentioned. Related, be cause in that case on the line:
    //     //    "descriptions = descriptions.substring(0, descriptions.length - delimiter.length) + ".  ""
    //     // ...one gets the below exception throw, probably for the same or related reason:
    //         /*
    //         ==============================================
    //         **CURRENT ENTITY:while at it, order a valentine's card on amazon asap (or did w/ cmas shopping?)
    //         No attributes have been assigned to this object, yet.
    //         1-Add attribute (quantity, true/false, date, text, external file, relation to entity or group: i.e., ownership of or "has" another entity, family ties, etc)...
    //         2-Import/Export...
    //         3-Edit name
    //         4-Delete or Archive...
    //         5-Go to...
    //         6-List next items
    //         7-Set current entity (while at it, order a valentine's card on amazon asap (or did w/ cmas shopping?)) as default (first to come up when launching this program.)
    //         8-Edit public/nonpublic status
    //         0/ESC - back/previous menu
    //         4
    //
    //
    //         ==============================================
    //         Choose a deletion or archiving option:
    //         1-Delete this entity
    //                  2-Archive this entity (remove from visibility but not permanent/total deletion)
    //         0/ESC - back/previous menu
    //         1
    //         An error occurred: "java.lang.StringIndexOutOfBoundsException: String index out of range: -2".  If you can provide simple instructions to reproduce it consistently, maybe it can be fixed.  Do you want to see the detailed output? (y/n):
    //           y
    //
    //
    //         ==============================================
    //         java.lang.StringIndexOutOfBoundsException: String index out of range: -2
    //         at java.lang.String.substring(String.java:1911)
    //         at org.onemodel.Controller.Controller.deleteOrArchiveEntity(Controller.scala:644)
    //         at org.onemodel.Controller.EntityMenu.entityMenu(EntityMenu.scala:232)
    //         at org.onemodel.Controller.EntityMenu.entityMenu(EntityMenu.scala:388)
    //         at org.onemodel.Controller.Controller.showInEntityMenuThenMainMenu(Controller.scala:277)
    //         at org.onemodel.Controller.MainMenu.mainMenu(MainMenu.scala:80)
    //         at org.onemodel.Controller.MainMenu.mainMenu(MainMenu.scala:98)
    //         at org.onemodel.Controller.MainMenu.mainMenu(MainMenu.scala:98)
    //         at org.onemodel.Controller.Controller.menuLoop$1(Controller.scala:140)
    //         at org.onemodel.Controller.Controller.start(Controller.scala:143)
    //         at org.onemodel.TextUI.launchUI(TextUI.scala:220)
    //         at org.onemodel.TextUI$.main(TextUI.scala:34)
    //         at org.onemodel.TextUI.main(TextUI.scala:1)
    //         */
    //
    //     let sql = format!("select group_id from entitiesinagroup where entity_id={} order by group_id limit {} offset {}",
    //                      entity_id_in, Self::check_if_should_be_all_results(max_vals_in), starting_index_in).as_str();
    //     get_containing_relation_to_groups_helper(transaction, sql)?;
    //   }

    fn get_count_of_entities_used_as_attribute_types(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        attribute_type_in: String,
        quantity_seeks_unit_not_type_in: bool,
    ) -> Result<u64, anyhow::Error> {
        let entities_sql = self.get_entities_used_as_attribute_types_sql(
            attribute_type_in,
            quantity_seeks_unit_not_type_in,
        )?;
        let sql = format!("SELECT count(1) {}", entities_sql);
        self.extract_row_count_from_count_query(transaction, sql.as_str())
    }

    // fn get_entities_used_as_attribute_types(&self, transaction: &Option<&mut Transaction<Postgres>>, attribute_type_in: String,
    //                                         starting_object_index_in: i64, max_vals_in: Option<i64> /*= None*/,
    //                                       quantity_seeks_unit_not_type_in: bool) -> Result<Vec<Entity>, anyhow::Error>  {
    //     let sql = format!("{}{}", Util::SELECT_ENTITY_START, self.get_entities_used_as_attribute_types_sql(attribute_type_in, quantity_seeks_unit_not_type_in)?).as_str();
    //     let early_results = self.db_query(transaction, sql, "i64,String,i64,i64,bool,bool,bool")?;
    //     let final_results: Vec<Entity> = Vec::new();
    //     // idea: should the remainder of this method be moved to Entity, so the persistence layer doesn't know anything about the Model? (helps avoid circular
    //     // dependencies; is a cleaner design.)  (and similar ones)
    //     for result in early_results {
    //       add_new_entity_to_results(final_results, result)
    //     }
    //     if final_results.len() != early_results.len() {
    //         return Err(anyhow!("In get_entities_used_as_attribute_types, final_results.len() ({}) != early_results.len() ({})", final_results.len(), early_results.len()));
    //     }
    //     Ok(final_results)
    //   }
    //%%:
    //   /// Allows querying for a range of objects in the database; returns a java.util.Map with keys and names.
    //   // 1st parm is index to start with (0-based), 2nd parm is # of obj's to return (if None, means no limit).
    // fn get_groups(&self, transaction: &Option<&mut Transaction<Postgres>>, starting_object_index_in: i64, max_vals_in: Option<i64> /*= None*/,
    //               group_to_omit_id_in: Option<i64> /*= None*/)  -> Result<Vec<Group>, anyhow::Error>  {
    //     let omission_expression: String = match group_to_omit_id_in {
    //       None => "true".to_string(),
    //       Some(gtoii) => format!("(not id={})", gtoii),
    //     };
    //     let sql = format!("SELECT id, name, insertion_date, allow_mixed_classes, new_entries_stick_to_top from grupo where {} \
    //                       order by id limit {} offset {}",
    //               omission_expression, Self::check_if_should_be_all_results(max_vals_in), starting_object_index_in).as_str();
    //     let early_results = self.db_query(transaction, sql, "i64,String,i64,bool,bool");
    //     let final_results: Vec<Group> = Vec::new();
    //     // idea: should the remainder of this method be moved to RTG, so the persistence layer doesn't know anything about the Model? (helps avoid circular
    //     // dependencies; is a cleaner design?)
    //     for result in early_results {
    //       // None of these values should be of "None" type. If they are it's a bug:
    //         //%%$%%%
    //       // final_results.add(new Group(this, result(0).get.asInstanceOf[i64], result(1).get.asInstanceOf[String], result(2).get.asInstanceOf[i64],
    //       //                            result(3).get.asInstanceOf[Boolean], result(4).get.asInstanceOf[Boolean]))
    //     }
    //     if final_results.len() != early_results.len() {
    //         return Err(anyhow!("In get_groups, final_results.len() ({}) != early_results.len() ({})", final_results.len(), early_results.len()));
    //     }
    //     Ok(final_results)
    //   }

    // fn get_classes(&self, transaction: &Option<&mut Transaction<Postgres>>, starting_object_index_in: i64, max_vals_in: Option<i64> /*= None*/)  -> Result<Vec<EntityClass>, anyhow::Error>  {
    //     let sql: String = format!("SELECT id, name, defining_entity_id, create_default_attributes from class order by id limit {} offset {}",
    //                       check_if_should_be_all_results(max_vals_in), starting_object_index_in);
    //     let early_results = self.db_query(transaction, sql.as_str(), "i64,String,i64,bool");
    //     let final_results: Vec<EntityClass> = Vec::new();
    //     // idea: should the remainder of this method be moved to EntityClass, so the persistence layer doesn't know anything about the Model? (helps avoid circular
    //     // dependencies; is a cleaner design?; see similar comment in get_entities_generic.)
    //     for result in early_results {
    //       // Only one of these values should be of "None" type.  If they are it's a bug:
    //         //%%$%%%
    //       // final_results.push(new EntityClass(this, result(0).get.asInstanceOf[i64], result(1).get.asInstanceOf[String], result(2).get.asInstanceOf[i64],
    //       //                                  if result(3).isEmpty) None else Some(result(3).get.asInstanceOf[Boolean])))
    //     }
    //     if final_results.len() != early_results.len() {
    //         return Err(anyhow!("In get_classes, final_results.len() ({}) != early_results.len() ({})", final_results.len(), early_results.len()));
    //     }
    //     Ok(final_results)
    //   }

    fn get_group_entries_data(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        group_id_in: i64,
        limit_in: Option<i64>,              /*= None*/
        include_archived_entities_in: bool, /*= true*/
    ) -> Result<Vec<Vec<Option<DataType>>>, anyhow::Error> {
        // LIKE THE OTHER 3 BELOW SIMILAR METHODS:
        // Need to make sure it gets the desired rows, rather than just some, so the order etc matters at each step, probably.
        // idea: needs automated tests (in task list also).
        let archived = if !include_archived_entities_in && !self.include_archived_entities {
            " and (not e.archived)"
        } else {
            ""
        };
        let sql = format!("select eiag.entity_id, eiag.sorting_index from entity e, entitiesinagroup eiag where e.id=eiag.entity_id \
                                    and eiag.group_id={}{} order by eiag.sorting_index, eiag.entity_id limit {}",
                                group_id_in, archived, Self::check_if_should_be_all_results(limit_in));
        self.db_query(
            transaction,
            sql.as_str(),
            Util::GET_GROUP_ENTRIES_DATA__RESULT_TYPES,
        )
    }

    fn get_adjacent_group_entries_sorting_indexes(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        group_id_in: i64,
        sorting_index_in: i64,
        limit_in: Option<i64>, /*= None*/
        forward_not_back_in: bool,
    ) -> Result<Vec<Vec<Option<DataType>>>, anyhow::Error> {
        // see comments in get_group_entries_data.
        // Doing "not e.archived", because the caller is probably trying to move entries up/down in the UI, and if we count archived entries but
        // are not showing them,
        // we could move relative to invisible entries only, and not make a visible move,  BUT: as of 2014-8-4, a comment was added, now gone, that said to ignore
        // archived entities while getting a new sorting_index is a bug. So if that bug is found again, we should cover all scenarios with automated
        // tests (showAllArchivedEntities is true and false, with archived entities present, and any other).
        let not_archived = if !self.include_archived_entities {
            " and (not e.archived)"
        } else {
            ""
        };
        let results = self.db_query(transaction, format!("select eiag.sorting_index from entity e, entitiesinagroup eiag where e.id=eiag.entity_id\
                                {} and eiag.group_id={} and eiag.sorting_index {}{} order by eiag.sorting_index {}, eiag.entity_id limit {}",
                                not_archived, group_id_in,
                                if forward_not_back_in { ">" } else { "<" },
                                sorting_index_in,
                                if forward_not_back_in { "ASC" } else { "DESC" },
                                Self::check_if_should_be_all_results(limit_in)).as_str(),
                                     "i64")?;
        Ok(results)
    }

    fn get_adjacent_attributes_sorting_indexes(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        entity_id_in: i64,
        sorting_index_in: i64,
        limit_in: Option<i64>,
        forward_not_back_in: bool,
    ) -> Result<Vec<Vec<Option<DataType>>>, anyhow::Error> {
        let results = self.db_query(transaction, format!("select sorting_index from AttributeSorting where \
                             entity_id={} and sorting_index {}{} order by sorting_index {} limit {}",
                             entity_id_in,
                             if forward_not_back_in { ">" } else { "<" },
                             sorting_index_in,
                             if forward_not_back_in {"ASC" } else { "DESC" },
                             Self::check_if_should_be_all_results(limit_in)).as_str(),
                                     "i64")?;
        Ok(results)
    }

    /// This one should explicitly NOT omit archived entities (unless parameterized for that later). See caller's comments for more, on purpose.
    fn get_nearest_group_entrys_sorting_index(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        group_id_in: i64,
        starting_point_sorting_index_in: i64,
        forward_not_back_in: bool,
    ) -> Result<Option<i64>, anyhow::Error> {
        let sql = format!(
            "select sorting_index from entitiesinagroup where group_id={} and sorting_index {}{} \
                                            order by sorting_index {} limit 1",
            group_id_in,
            (if forward_not_back_in { ">" } else { "<" }),
            starting_point_sorting_index_in,
            (if forward_not_back_in { "ASC" } else { "DESC" })
        );
        let results: Vec<Vec<Option<DataType>>> =
            self.db_query(transaction, sql.as_str(), "i64")?;
        if results.is_empty() {
            Ok(None)
        } else if results.len() > 1 {
            return Err(anyhow!("In get_nearest_group_entrys_sorting_index, probably the caller didn't expect this to get >1 results...Is that even meaningful? sql was: {}", sql));
        } else {
            let row = match results.get(0) {
                None => {
                    return Err(anyhow!(
                        "Expected a row result, got none for results at index 0. Results is {:?}",
                        results
                    ))
                }
                Some(x) => x,
            };
            match row.get(0) {
                Some(Some(DataType::Bigint(i))) => return Ok(Some(i.clone())),
                _ => {
                    return Err(anyhow!(
                    "In get_nearest_group_entrys_sorting_index, unexpected row {:?}, from sql: {}",
                    row,
                    sql
                ))
                }
            };
        }
    }

    fn get_nearest_attribute_entrys_sorting_index(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        entity_id_in: i64,
        starting_point_sorting_index_in: i64,
        forward_not_back_in: bool,
    ) -> Result<Option<i64>, anyhow::Error> {
        let results: Vec<Vec<Option<DataType>>> = self.get_adjacent_attributes_sorting_indexes(
            transaction,
            entity_id_in,
            starting_point_sorting_index_in,
            Some(1),
            forward_not_back_in,
        )?;
        if results.is_empty() {
            Ok(None)
        } else if results.len() > 1 {
            Err(anyhow!("Probably the caller didn't expect this to get >1 results...Is that even meaningful?: {:?}", results))
        } else {
            if results[0].len() != 1 {
                Err(anyhow!("Probably the caller didn't expect this to get != 1 columns...Is that even meaningful?: {:?}", results))
            } else {
                match results[0][0] {
                    None => Ok(None),
                    Some(DataType::Bigint(i)) => Ok(Some(i)),
                    _ => Err(anyhow!(
                        "Unexpected value in results[0][0]: {:?}",
                        results[0][0]
                    )),
                }
            }
        }
    }

    // 2nd parm is 0-based index to start with, 3rd parm is # of objs to return (if < 1 then it means "all"):
    fn get_group_entry_objects(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        group_id_in: i64,
        starting_object_index_in: i64,
        max_vals_in: Option<i64>, /*= None*/
    ) -> Result<Vec<Entity>, anyhow::Error> {
        let not_archived = if !self.include_archived_entities {
            " and (not e.archived) "
        } else {
            ""
        };
        // see comments in get_group_entries_data
        let sql = format!("select entity_id, sorting_index from entity e, EntitiesInAGroup eiag where e.id=eiag.entity_id\
                                    {} and eiag.group_id={} order by eiag.sorting_index, eiag.entity_id limit {} offset {}",
                                    not_archived, group_id_in, Self::check_if_should_be_all_results(max_vals_in), starting_object_index_in);
        let early_results = self.db_query(transaction, sql.as_str(), "i64,i64")?;
        let early_results_len = early_results.len();
        let mut final_results: Vec<Entity> = Vec::new();
        // idea: should the remainder of this method be moved to Entity, so the persistence layer doesn't know anything about the Model? (helps avoid circular
        // dependencies; is a cleaner design?  Or, maybe this class and all the object classes like Entity, etc, are all part of the same layer.)
        // And doing similarly elsewhere such as in get_om_instance_data().
        for result in early_results {
            if result.len() == 0 {
                return Err(anyhow!(
                    "In get_group_entry_objects, Unexpected 0-len() result: {:?}",
                    result
                ));
            }
            // None of these values should be of "None" type, so not checking for that. If they are it's a bug:
            match result[0] {
                None => {
                    return Err(anyhow!(
                        "In get_group_entry_objects, Unexpected None in result[0] {:?}",
                        result[0]
                    ))
                }
                Some(DataType::Bigint(i)) => {
                    final_results.push(Entity::new2(Box::new(self), transaction, i)?)
                }
                _ => {
                    return Err(anyhow!(
                        "In get_group_entry_objects, Unexpected value in result[0] {:?}",
                        result[0]
                    ))
                }
            };
        }
        if !(final_results.len() == early_results_len) {
            return Err(anyhow!(
                "In get_group_entry_objects, final_results.len() ({}) != early_results.len() ({}).",
                final_results.len(),
                early_results_len
            ));
        }
        Ok(final_results)
    }

    fn get_entity_data(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        id_in: i64,
    ) -> Result<Vec<Option<DataType>>, anyhow::Error> {
        self.db_query_wrapper_for_one_row(transaction,
                                          format!("SELECT name, class_id, insertion_date, public, \
                                          archived, new_entries_stick_to_top from Entity where id={}",
                                                  id_in).as_str(),
                                         Util::GET_ENTITY_DATA__RESULT_TYPES)
    }

    fn get_entity_name(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        id_in: i64,
    ) -> Result<Option<String>, anyhow::Error> {
        let name: Vec<Option<DataType>> = self.get_entity_data(transaction, id_in)?;
        match name.get(0) {
            None => Ok(None),
            Some(Some(DataType::String(x))) => Ok(Some(x.to_string())),
            _ => Err(anyhow!(format!("Unexpected value: {:?}", name))),
        }
    }

    fn get_class_data(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        id_in: i64,
    ) -> Result<Vec<Option<DataType>>, anyhow::Error> {
        self.db_query_wrapper_for_one_row(
            transaction,
            format!(
                "SELECT name, defining_entity_id, create_default_attributes from class where id={}",
                id_in
            )
            .as_str(),
            Util::GET_CLASS_DATA__RESULT_TYPES,
        )
    }

    fn get_class_name(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        id_in: i64,
    ) -> Result<Option<String>, anyhow::Error> {
        let columns = self.get_class_data(transaction, id_in)?;
        if columns.len() == 0 {
            return Err(anyhow!(
                "In get_class_name, No rows returned for class {} ?",
                id_in
            ));
        }
        let name: String = match columns[0].clone() {
            Some(DataType::String(s)) => s,
            _ => {
                return Err(anyhow!(
                    "In get_class_name, No name returned for class {} column 0? (columns: {:?})",
                    id_in,
                    columns
                ))
            }
        };
        // if name.isEmpty) None
        // else name.asInstanceOf[Option<String>]
        Ok(Some(name))
    }

    /// @return the create_default_attributes boolean value from a given class.
    fn update_class_create_default_attributes(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        class_id_in: i64,
        value: Option<bool>,
    ) -> Result<u64, anyhow::Error> {
        let value_sql = match value {
            None => "NULL",
            Some(true) => "true",
            _ => "false",
        };
        self.db_action(
            transaction,
            format!(
                "update class set (create_default_attributes) = ROW({}) where id={}",
                value_sql, class_id_in
            )
            .as_str(),
            false,
            false,
        )
    }

    /// The 2nd parameter is to avoid saying an entity is a duplicate of itself: checks for all others only.
    fn is_duplicate_entity_name(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        name_in: &str,
        self_id_to_ignore_in: Option<i64>, /*= None*/
    ) -> Result<bool, anyhow::Error> {
        let first = self.is_duplicate_row(
            transaction,
            name_in,
            Util::ENTITY_TYPE,
            "id",
            "name",
            if !self.include_archived_entities {
                Some("(not archived)")
            } else {
                None
            },
            match self_id_to_ignore_in {
                None => None,
                Some(id) => Some(format!("{}", id)),
            },
        )?;
        let second = self.is_duplicate_row(
            transaction,
            name_in,
            Util::RELATION_TYPE_TYPE,
            "entity_id",
            "name_in_reverse_direction",
            None,
            match self_id_to_ignore_in {
                None => None,
                Some(id) => Some(format!("{}", id)),
            },
        )?;
        Ok(first || second)
    }

    /// The 2nd parameter is to avoid saying a class is a duplicate of itself: checks for all others only. */
    fn is_duplicate_class_name(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        name_in: &str,
        self_id_to_ignore_in: Option<i64>, /*= None*/
    ) -> Result<bool, anyhow::Error> {
        self.is_duplicate_row(
            transaction,
            name_in,
            "class",
            "id",
            "name",
            None,
            match self_id_to_ignore_in {
                None => None,
                Some(id) => Some(format!("{}", id)),
            },
        )
    }

    /// The 2nd parameter is to avoid saying an instance is a duplicate of itself: checks for all others only.
    fn is_duplicate_om_instance_address(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        address_in: &str,
        self_id_to_ignore_in: Option<String>, /*= None*/
    ) -> Result<bool, anyhow::Error> {
        self.is_duplicate_row(
            transaction,
            address_in,
            "omInstance",
            "id",
            "address",
            None,
            self_id_to_ignore_in,
        )
    }

    // fn finalize() {
    //fn drop .. what form?
    //   //%%  super.finalize()
    //   // if connection != null) connection.close()
    //     self.pool.%%?
    // }

    /*%%
                fn get_or_create_class_and_template_entity(class_name_in: String, caller_manages_transactions_in: bool) -> (i64, i64) {
                    //(see note above re 'bad smell' in method addUriEntityWithUriAttribute.)
                          //rollbacketc%%FIX NEXT LINE AFTERI SEE HOW OTHERS DO!
                    // if !caller_manages_transactions_in { self.begin_trans() }
                    try {
                      let (class_id, entity_id) = {
                        let foundId = find_first_class_id_by_name(class_name_in, case_sensitive = true);
                        if foundId.is_some()) {
                          let entity_id: i64 = new EntityClass(this, foundId.get).get_template_entity_id;
                          (foundId.get, entity_id)
                        } else {
                          let (class_id: i64, entity_id: i64) = create_class_and_its_template_entity(class_name_in);
                          (class_id, entity_id)
                        }
                      }
                          //rollbacketc%%FIX NEXT LINE AFTERI SEE HOW OTHERS DO!
                      // if !caller_manages_transactions_in {self.commit_trans() }
                      (class_id, entity_id)
                    }
                    catch {
                      case e: Exception =>
                          //rollbacketc%%FIX NEXT LINE AFTERI SEE HOW OTHERS DO!
                        // if !caller_manages_transactions_in) rollback_trans()
                        throw e
                    }
                  }
    */

    fn set_include_archived_entities(&mut self, iae_in: bool) {
        self.include_archived_entities = iae_in;
    }

    fn get_om_instance_count(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
    ) -> Result<u64, anyhow::Error> {
        self.extract_row_count_from_count_query(transaction, "SELECT count(1) from omInstance")
    }

    fn create_om_instance(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        id_in: String,
        is_local_in: bool,
        address_in: String,
        entity_id_in: Option<i64>, /*%% = None*/
        old_table_name: bool,      /*%% = false*/
    ) -> Result<i64, anyhow::Error> {
        if id_in.len() == 0 {
            return Err(anyhow!(
                "In create_om_instance, ID must have a value.".to_string()
            ));
        }
        if address_in.len() == 0 {
            return Err(anyhow!(
                "In create_om_instance, Address must have a value.".to_string()
            ));
        }
        let id: String = Self::escape_quotes_etc(id_in.clone());
        let address: String = Self::escape_quotes_etc(address_in.clone());
        if id != id_in {
            return Err(anyhow!(format!(
                "In create_om_instance, Didn't expect quotes etc in the UUID provided: {}",
                id_in
            )));
        };
        if address != address_in {
            return Err(anyhow!(format!(
                "In create_om_instance, didn't expect quotes etc in the address provided: {}",
                address
            )));
        }
        let insertion_date: i64 = Utc::now().timestamp_millis();
        // next line is for the method upgradeDbFrom3to4 so it can work before upgrading 4to5:
        let table_name: &str = if old_table_name {
            "om_instance"
        } else {
            "omInstance"
        };
        let is_local = if is_local_in { "TRUE" } else { "FALSE" };
        let maybe_entity_id_value = match entity_id_in {
            None => "NULL".to_string(),
            Some(id) => id.to_string(),
        };
        let sql: String = format!(
            "INSERT INTO {} (id, local, address, insertion_date, entity_id) \
                                  VALUES ('{}',{},'{}',{},\
                                  {})",
            table_name, id, is_local, address, insertion_date, maybe_entity_id_value
        );
        self.db_action(transaction, sql.as_str(), false, false)?;
        Ok(insertion_date)
    }

    fn get_om_instance_data(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        id_in: String,
    ) -> Result<Vec<Option<DataType>>, anyhow::Error> {
        self.db_query_wrapper_for_one_row(
            transaction,
            format!(
                "SELECT local, address, insertion_date, entity_id from omInstance where id='{}'",
                id_in
            )
            .as_str(),
            Util::GET_OM_INSTANCE_DATA__RESULT_TYPES,
        )
    }

    /*%%$%%
                  lazy let id: String = {;
                    get_local_om_instance_data.get_id
                  }
    */
    fn om_instance_key_exists(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        id_in: String,
    ) -> Result<bool, anyhow::Error> {
        self.does_this_exist(
            transaction,
            format!("SELECT count(1) from omInstance where id='{}'", id_in).as_str(),
            true,
        )
    }

    //%%$%%
    /*
                fn get_om_instances(localIn: Option<bool> = None) -> java.util.ArrayList[OmInstance] {
                let sql = "select id, local, address, insertion_date, entity_id from omInstance" +;
                          (if localIn.is_some()) {
                            if localIn.get) {
                              " where local=TRUE"
                            } else {
                              " where local=FALSE"
                            }
                          } else {
                            ""
                          })
                let early_results = db_query(sql, "String,bool,String,i64,i64");
                let final_results = new java.util.ArrayList[OmInstance];
                // (Idea: See note in similar point in get_group_entry_objects.)
                for (result <- early_results) {
                  final_results.add(new OmInstance(this, result(0).get.asInstanceOf[String], is_local_in = result(1).get.asInstanceOf[Boolean],
                                                  result(2).get.asInstanceOf[String],
                                                  result(3).get.asInstanceOf[i64], if result(4).isEmpty) None else Some(result(4).get.asInstanceOf[i64])))
                }
                require(final_results.size == early_results.size)
                if localIn.is_some() && localIn.get && final_results.size == 0) {
                  let total = get_om_instance_count();
                  throw new OmDatabaseException("Unexpected: the # of rows omInstance where local=TRUE is 0, and there should always be at least one." +
                                                "(See insert at end of create_base_data and upgradeDbFrom3to4.)  Total # of rows: " + total)
                }
                final_results
              }

      "get_local_om_instance_data and friends" should "work" in {
        let oi: OmInstance = m_db.get_local_om_instance_data;
        let uuid: String = oi.get_id;
        assert(oi.getLocal)
        assert(m_db.om_instance_key_exists(uuid))
        let startingOmiCount = m_db.get_om_instance_count();
        assert(startingOmiCount > 0)
        let oiAgainAddress = m_db.get_om_instance_data(uuid)(1).get.asInstanceOf[String];
        assert(oiAgainAddress == Util.LOCAL_OM_INSTANCE_DEFAULT_DESCRIPTION)
        let omInstances: util.ArrayList[OmInstance] = m_db.get_om_instances();
        assert(omInstances.size == startingOmiCount)
        let sizeNowTrue = m_db.get_om_instances(Some(true)).size;
        assert(sizeNowTrue > 0)
        // Idea: fix: Next line fails at times, maybe due to code running in parallel between this and RestDatabaseTest, creating/deleting rows.  Only seems to happen
        // when all tests are run, never when the test classes are run separately.
        //    let sizeNowFalse = m_db.get_om_instances(Some(false)).size;
        //assert(sizeNowFalse < sizeNowTrue)
        assert(! m_db.om_instance_key_exists(java.util.UUID.randomUUID().toString))
        assert(new OmInstance(m_db, uuid).getAddress == Util.LOCAL_OM_INSTANCE_DEFAULT_DESCRIPTION)

        let uuid2 = java.util.UUID.randomUUID().toString;
        m_db.create_om_instance(uuid2, is_local_in = false, "om.example.com", Some(m_db.get_system_entity_id))
        // should have the local one created at db creation, and now the one for this test:
        assert(m_db.get_om_instance_count() == startingOmiCount + 1)
        let mut i2: OmInstance = new OmInstance(m_db, uuid2);
        assert(i2.getAddress == "om.example.com")
        m_db.update_om_instance(uuid2, "address", None)
        i2  = new OmInstance(m_db,uuid2)
        assert(i2.getAddress == "address")
        assert(!i2.getLocal)
        assert(i2.getEntityId.isEmpty)
        assert(i2.getCreationDate > 0)
        assert(i2.getCreationDateFormatted.length > 0)
        m_db.update_om_instance(uuid2, "address", Some(m_db.get_system_entity_id))
        i2  = new OmInstance(m_db,uuid2)
        assert(i2.getEntityId.get == m_db.get_system_entity_id)
        assert(m_db.is_duplicate_om_instance_address("address"))
        assert(m_db.is_duplicate_om_instance_address(Util.LOCAL_OM_INSTANCE_DEFAULT_DESCRIPTION))
        assert(!m_db.is_duplicate_om_instance_address("address", Some(uuid2)))
        assert(!m_db.is_duplicate_om_instance_address(Util.LOCAL_OM_INSTANCE_DEFAULT_DESCRIPTION, Some(uuid)))
        let uuid3 = java.util.UUID.randomUUID().toString;
        m_db.create_om_instance(uuid3, is_local_in = false, "address", Some(m_db.get_system_entity_id))
        assert(m_db.is_duplicate_om_instance_address("address", Some(uuid2)))
        assert(m_db.is_duplicate_om_instance_address("address", Some(uuid3)))
        i2.delete()
        assert(m_db.is_duplicate_om_instance_address("address"))
        assert(m_db.is_duplicate_om_instance_address("address", Some(uuid2)))
        assert(!m_db.is_duplicate_om_instance_address("address", Some(uuid3)))
        assert(intercept[Exception] {
                                      new OmInstance(m_db, uuid2)
                                    }.getMessage.contains("does not exist"))
      }
    */

    /*
        fn update_om_instance(id_in: String, address_in: String, entity_id_in: Option<i64>) {
        let address: String = self.escape_quotes_etc(address_in);
        let sql = format!("UPDATE omInstance SET (address, entity_id)" +;
                  " = ('" + address + "', " +
                  (if entity_id_in.is_some()) {
                    entity_id_in.get
                  } else {
                    "NULL"
                  }) +
                  ") where id='" + id_in + "'");
        self.db_action(sql.as_str(), false, false);
      }

        fn delete_om_instance(id_in: String) /* -> Unit%%*/ {
        delete_object_by_id2("omInstance", id_in)
      }

    */
}

fn get_i64s_from_rows(rows: &Vec<Vec<Option<DataType>>>) -> Result<Vec<i64>, anyhow::Error> {
    let mut results: Vec<i64> = Vec::new();
    for row in rows {
        let id = get_i64_from_row(&row, 0)?;
        results.push(id);
    }
    Ok(results)
}
fn get_i64_from_row(row: &Vec<Option<DataType>>, index: usize) -> Result<i64, anyhow::Error> {
    let id: i64 = match row.get(index) {
        Some(Some(DataType::Bigint(n))) => *n,
        _ => {
            return Err(anyhow!(
                "In get_i64_from_row for index {}, Unexpected row: {:?}",
                index,
                row
            ))
        }
    };
    Ok(id)
}
fn get_i64_from_row_without_option(
    row: &Vec<DataType>,
    index: usize,
) -> Result<i64, anyhow::Error> {
    let id: i64 = match row.get(index) {
        Some(DataType::Bigint(n)) => *n,
        _ => {
            return Err(anyhow!(
                "In get_i64_from_row for index {}, Unexpected row: {:?}",
                index,
                row
            ))
        }
    };
    Ok(id)
}

#[cfg(test)]
mod tests {
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
}
