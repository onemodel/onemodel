/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003, 2004, 2010, 2011, 2013-2020 inclusive, and 2023-2023 inclusive, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
// use crate::model::boolean_attribute::BooleanAttribute;
use crate::model::database::DataType;
use crate::model::database::Database;
// use crate::model::entity::Entity;
// use crate::model::postgres::postgresql_database2::*;
// use crate::model::postgres::*;
// use crate::model::relation_to_local_entity::RelationToLocalEntity;
// use crate::model::relation_to_remote_entity::RelationToRemoteEntity;
use crate::util::Util;
use anyhow::anyhow;
use chrono::Utc;
// use futures::executor::block_on;
use sqlx::postgres::*;
// Specifically omitting sql::Error from use statements so that it is *clearer* which Error type is
// in use, in the code.
use sqlx::{Column, PgPool, Postgres, Row, Transaction, ValueRef};
// use std::collections::HashSet;
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
    // Moved methods that are not part of the Database trait go here
    // or in postgresql_database2.rs (split up to make smaller files,
    // for parsing speed during intellij editing).

    pub const SCHEMA_VERSION: i32 = 7;
    pub const ENTITY_ONLY_SELECT_PART: &'static str = "SELECT e.id";

    fn db_name(db_name_without_prefix: &str) -> String {
        format!("{}{}", Util::DB_NAME_PREFIX, db_name_without_prefix)
    }

    //%%should this and other eventual callers of db_query take its advice and call the ck method?
    pub fn db_query_wrapper_for_one_row(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        sql: &str,
        types: &str,
    ) -> Result<Vec<Option<DataType>>, anyhow::Error> {
        let results: Vec<Vec<Option<DataType>>> = self.db_query(transaction, sql, types)?;
        if results.len() != 1 {
            Err(anyhow!(
                "Got {} instead of 1 result from sql \"{}\" ??",
                results.len(),
                sql
            ))
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
                    None => None,
                    // _ => {
                    //     return Err(anyhow!(
                    //         "How did we get here for x of {:?} in {:?}?",
                    //         x, results[0]
                    //     ))
                    // }
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
    //%%%%should the things in "types" parm be an enum or something like that? Or doc it here?
    pub fn db_query(
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
                            //%%%
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
    pub fn does_this_exist(
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
                Err(anyhow!(
                    "Should there be > 1 entries for sql: {}?? ({} were found.)",
                    sql_in,
                    row_count
                ))
            } else {
                assert!(row_count < 1);
                Ok(false)
            }
        } else {
            Ok(row_count >= 1)
        }
    }

    pub fn extract_row_count_from_count_query(
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
        self.drop(&None, "table", "odb_version")?;
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
    pub fn drop(
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
    pub fn escape_quotes_etc(s: String) -> String {
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
    pub fn db_action(
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
            return Err(anyhow!(
                "Affected {} rows instead of 1?? SQL was: {}",
                rows_affected,
                sql_in
            ));
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
        //         return Err(anyhow!(
        //             "Unable to start a database transaction to set up database?: {}",
        //             e.to_string()
        //         ))
        //     }
        //     Ok(t) => t,
        // };
        // if !new_db.model_tables_exist(&Some(&mut tx))? {
        //     // //%%% try to see what happens if pg down be4 & during this--does the err propagate ok?
        //     new_db.create_tables(&Some(&mut tx))?;
        //     //%%% try to see what happens if pg down be4 & during this--does the err propagate ok?
        //     new_db.create_base_data(&Some(&mut tx))?;
        // }
        // //%% do_database_upgrades_if_needed()
        // new_db.create_and_check_expected_data(&Some(&mut tx))?;
        // match new_db.commit_trans(&mut tx) {
        //     Err(e) => {
        //         return Err(anyhow!(
        //             "Unable to commit database transaction for db setup: {}",
        //             e.to_string()
        //         ))
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
            // //%%% try to see what happens if pg down be4 & during this--does the err propagate ok?
            self.create_tables(&Some(&mut tx))?;
            //%%% try to see what happens if pg down be4 & during this--does the err propagate ok?
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
        //%%%just some testing, can delete after next commit, or use for a while for reference.
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

    pub fn create_version_table(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
    ) -> Result<u64, anyhow::Error> {
        // table has 1 row and 1 column, to say what db version we are on.
        self.db_action(
            transaction,
            // default 1 due to lack of a better idea.  See comment just below.
            "create table odb_version (version integer DEFAULT 1) ",
            false,
            false,
        )?;
        self.db_action(
            transaction,
            // Initially 0 due to lack of a better idea.  The other setup code (fn create_tables
            // currently) should set it correctly to the updated version, once the schema with
            // that specific version has actually been created.
            "INSERT INTO odb_version (version) values (0)",
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
        // Entity.add_quantity_attribute(...).
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
        // For the boolean_value column: allowing nulls because a template might not have \
        // value, and a task might not have a "done/not" setting yet (if unknown)?
        // Ex., isDone (where the task would be an entity).
        // See "create table RelationToEntity" for comments about dates' meanings.
        self.db_action(transaction, format!("create table BooleanAttribute (\
            form_id smallint DEFAULT {} \
                NOT NULL CHECK (form_id={}), \
            id bigint DEFAULT nextval('BooleanAttributeKeySequence') PRIMARY KEY, \
            entity_id bigint NOT NULL, \
            boolean_value boolean, \
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
        // Entity.add_quantity_attribute(...).
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
                "UPDATE odb_version SET (version) = ROW({})",
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
    pub fn find_entity_only_ids_by_name(
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
    pub fn create_class_and_its_template_entity2<'a>(
        &'a self,
        transaction_in: &Option<&mut Transaction<'a, Postgres>>,
        class_name_in: String,
        entity_name_in: String,
        // (See fn delete_objects for more about this parameter, and transaction above.)
        caller_manages_transactions_in: bool, /*%%= false*/
    ) -> Result<(i64, i64), anyhow::Error> {
        //BEGIN COPY/PASTED/DUPLICATED (except "in <fn_name>" in 2 Err msgs below) BLOCK-----------------------------------
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
    pub fn get_system_entitys_class_group_id(
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
    pub fn get_new_key(
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

    pub fn are_mixed_classes_allowed(
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
    pub fn has_mixed_classes(
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
}

pub fn get_i64s_from_rows(rows: &Vec<Vec<Option<DataType>>>) -> Result<Vec<i64>, anyhow::Error> {
    let mut results: Vec<i64> = Vec::new();
    for row in rows {
        let id = get_i64_from_row(&row, 0)?;
        results.push(id);
    }
    Ok(results)
}
pub fn get_i64_from_row(row: &Vec<Option<DataType>>, index: usize) -> Result<i64, anyhow::Error> {
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

pub fn get_i64_from_row_without_option(
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
