/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003, 2004, 2010, 2011, 2013-2020 inclusive, and 2023-2023 inclusive, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
use std::collections::HashSet;
use chrono::{/*DateTime, NaiveDateTime,%%*/ Utc};
use crate::model::database::Database;
use crate::model::database::DataType;
use crate::model::entity::Entity;
use crate::model::boolean_attribute::BooleanAttribute;
use crate::model::relation_to_local_entity::RelationToLocalEntity;
use crate::model::relation_to_remote_entity::RelationToRemoteEntity;
use crate::util::Util;
use futures::executor::block_on;
use sqlx::postgres::*;
use sqlx::{Error, PgPool, Postgres, Row, Transaction};

pub struct PostgreSQLDatabase {
    //%% connection: Connection,
    pub pool: PgPool,
    // When true, this means to override the usual settings and show the archived entities too (like a global temporary "un-archive"):
    pub include_archived_entities: bool,
}

impl PostgreSQLDatabase {
    const SCHEMA_VERSION: i32 = 7;
    const ENTITY_ONLY_SELECT_PART: &'static str = "SELECT e.id";

    /*%%
        package org.onemodel.core.model

        import java.io.{PrintWriter, StringWriter}
        import java.sql.{Connection, DriverManager, ResultSet, Statement}
        import java.util.ArrayList
        import java.util.regex.Pattern

        import org.onemodel.core._
        import org.onemodel.core.model.Database._
        import org.postgresql.largeobject.{LargeObject, LargeObjectManager}

        import scala.annotation.tailrec
        import scala.collection.mutable
        import scala.util.Sorting

        ** Some methods are here on the object, so that PostgreSQLDatabaseTest can call destroy_tables on test data.
          *
        object PostgreSQLDatabase {
    */

    fn db_name(db_name_without_prefix: &str) -> String {
        format!("{}{}", Util::DB_NAME_PREFIX, db_name_without_prefix)
    }

    fn db_query_wrapper_for_one_row(&self, sql: String, types: &str) -> Result<Vec<DataType>, String> {
        let results: Vec<Vec<DataType>> = self.db_query(sql.as_str(), types)?;
        if results.len() != 1 {
            Err(format!("Got {} instead of 1 result from sql \"{}\" ??", results.len(), sql))
        } else {
            let oldrow = &results[0];
            let mut newrow = Vec::new();
            for x in oldrow {
                let z = match &x {
                    //idea: surely there is some better way than what I am doing here? See other places similarly.  Maybe implement DataType.clone() ?
                    DataType::Bigint(y) => DataType::Bigint(y.clone()),
                    DataType::Boolean(y) => DataType::Boolean(y.clone()),
                    DataType::String(y) => DataType::String(y.clone()),
                    DataType::Float(y) => DataType::Float(y.clone()),
                    DataType::Smallint(y) => DataType::Smallint(y.clone()),
                    // _ => return Err(format!("How did we get here for {:?}?", results[0])),
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
        fn db_query(&self, sql: &str, types: &str) -> Result<Vec<Vec<DataType>>, String> {
            // Note: pgsql docs say "Under the JDBC specification, you should access a field only
            // once" (under the JDBC interface part).  Not sure if that applies now to sqlx in rust.

            Self::check_for_bad_sql(sql)?;
            let mut results: Vec<Vec<DataType>> = Vec::new();
            let types_vec: Vec<&str> = types.split_terminator(",").collect();
            let mut row_counter = 0;
            let query = sqlx::query(sql);
            let future = query.map(|sqlx_row: PgRow| {
            //%%$%%% add back this but how return the err? see exs more or figure it out?
            //     if row.len() != types_vec.len() {
            //         Err(format!("Row length {} does not equal expected types list length {}.", row.len(), types_vec.len()))
            //     } else {
                    // (next line is 1-based -- intended to compare to the size of results, later)
                    row_counter += 1;
                    //was: let row: Array[Option[Any]] = new Array[Option[Any]](types_vec.length);
                    let mut row: Vec<DataType> = Vec::new();
                    let mut column_counter: usize = 0;
                    for type_name in &types_vec {
                        // the for loop is to take is through all the columns in this row, as specified by the caller in the "types" parm.
                        //was: if rs.getObject(column_counter) == null) row(column_counter - 1) = None
                        //%%name?:
                        // else {
                        //%%$%%%%how will these handle nulls? (like validOnDate or Entity.m_class_id??) how can they?
                            // When modifying: COMPARE TO AND SYNCHRONIZE WITH THE TYPES IN the for loop in RestDatabase.processArrayOptionAny .
                            if type_name == &"Float" {
                                //was: row(column_counter) = Some(rs.getFloat(column_counter))
                                let decode_mbe: Result<_, sqlx::Error> = sqlx_row.try_get(column_counter);
                                let x: f64 = decode_mbe.unwrap(); //%%???
                                println!("in db_query1: x is {} .", x);
                                let y = DataType::Float(x);
                                row.push(y);
                            } else if type_name == &"String" {
                            //%%$%%%%%
                            //     was: row(column_counter) = Some(PostgreSQLDatabase.unescape_quotes_etc(rs.getString(column_counter)))
                                let decode_mbe: Result<_, sqlx::Error> = sqlx_row.try_get(column_counter);
                                let x: String = decode_mbe.unwrap(); //%%???
                                println!("in db_query3: x is {x} .");
                                let y = DataType::String(Self::unescape_quotes_etc(x));
                                println!("in db_query3: y is {:?} .", y);
                                row.push(y);
                            } else if type_name == &"i64" {
                                //was: row(column_counter) = Some(rs.getLong(column_counter))
                                let decode_mbe = sqlx_row.try_get(column_counter);
                                let x: i64 = decode_mbe.unwrap(); //%%???
                                println!("in db_query4: x is {} .", x);
                                row.push(DataType::Bigint(x));
                            } else if type_name == &"Boolean" {
                                //was: row(column_counter) = Some(rs.get_boolean(column_counter))
                                let decode_mbe: Result<_, sqlx::Error> = sqlx_row.try_get(column_counter);
                                let x: bool = decode_mbe.unwrap(); //%%???
                                println!("in db_query5: x is {} .", x);
                                let y = DataType::Boolean(x);
                                row.push(y);
                            } else if type_name == &"Int" {
                            //     row(column_counter) = Some(rs.getInt(column_counter))
                                let decode_mbe: Result<_, sqlx::Error> = sqlx_row.try_get(column_counter);
                                let x: i16 = decode_mbe.unwrap(); //%%???
                                println!("in db_query6: x is {} .", x);
                                let y = DataType::Smallint(x);
                                row.push(y);
                            } else {
                                // how to make this just return an error from the fn that calls this
                                // closure, instead?  Should never happen though.
                                // was: throw new OmDatabaseException("unexpected value: '" + type_name + "'")
                                panic!("Unexpected DataType value: '{}'.", type_name);
                            }
                        // }
                        column_counter += 1;
                    }
                // }
                results.push(row);
            }).fetch_all(&self.pool);
            /*let rows =*/ block_on(future).unwrap();

                // idea: (see comment at other use in this class, of getWarnings)
                // idea: maybe both uses of getWarnings should be combined into a method.
                //%%how do this in rust/sqlx?:
                // let warnings = rs.getWarnings;
                // let warnings2 = st.getWarnings;
                // if warnings != null || warnings2 != null) throw new OmDatabaseException("Warnings from postgresql. Matters? Says: " + warnings + ", and " + warnings2)

            //%%change to return an err here?:
            assert_eq!(row_counter, results.len());
            Ok(results)
        }

        /// Convenience function. Error message it gives if > 1 found assumes that sql passed in will return only 1 row!
        fn does_this_exist(&self, sql_in: &str, fail_if_more_than_one_found: bool /*%% = true*/) -> Result<bool, String> {
            let row_count: i64 = self.extract_row_count_from_count_query(sql_in)?;
            if fail_if_more_than_one_found {
                if row_count == 1 {
                    Ok(true)
                } else if row_count > 1 {
                    Err(format!("Should there be > 1 entries for sql: {}?? ({} were found.)", sql_in, row_count))
                } else {
                    assert!(row_count < 1);
                    Ok(false)
                }
            } else {
                Ok(row_count >= 1)
            }
        }

        fn extract_row_count_from_count_query(&self, sql_in: &str) -> Result<i64, String> {
            let results: Vec<DataType> = self.db_query_wrapper_for_one_row(sql_in.to_string(), "i64")?;
            let result: i64 = match results[0] {
                DataType::Bigint(x) => x,
                _ => return Err("Should never happen".to_string()),
            };
            Ok(result)
        }

    pub fn destroy_tables(&self) -> Result<(), String> {
        //%%see comments at similar places elsewhere, re:
        // conn.setTransactionIsolation(Connection.TRANSACTION_SERIALIZABLE)

        /**** WHEN MAINTAINING THIS METHOD, SIMILARLY MAINTAIN THE SCRIPT core/bin/purge-om-test-database*
        SO IT DOES THE SAME WORK. ****/

        // Doing these individually so that if one fails (not previously existing, such as
        // testing or a new installation), the others can proceed (drop method ignores that
        // exception).
        self.drop("table", "om_db_version")?;
        self.drop("table", Util::QUANTITY_TYPE)?;
        self.drop("table", Util::DATE_TYPE)?;
        self.drop("table", Util::BOOLEAN_TYPE)?;
        // The next line is to invoke the trigger that will clean out Large Objects
        // (FileAttributeContent...) from the table pg_largeobject.
        // The LO cleanup doesn't happen (trigger not invoked) w/ just a drop (or truncate),
        // but does on delete.  For more info see the wiki reference
        // link among those down in this file below "create table FileAttribute".
        let result = self.db_action("delete from FileAttributeContent", /*%%caller_checks_row_count_etc =*/ true, false);
        if let Err(msg) = result {
            if !msg.to_lowercase().contains("does not exist") {
                return Err(msg.clone());
            }
        }
        self.drop("table", "FileAttributeContent")?;
        self.drop("table", Util::FILE_TYPE)?;
        self.drop("table", Util::TEXT_TYPE)?;
        self.drop("table", Util::RELATION_TO_LOCAL_ENTITY_TYPE)?;
        self.drop("table", Util::RELATION_TO_REMOTE_ENTITY_TYPE)?;
        self.drop("table", "EntitiesInAGroup")?;
        self.drop("table", Util::RELATION_TO_GROUP_TYPE)?;
        self.drop("table", "action")?;
        self.drop("table", "grupo")?;
        self.drop("table", Util::RELATION_TYPE_TYPE)?;
        self.drop("table", "AttributeSorting")?;
        self.drop("table", "omInstance")?;
        self.drop("table", Util::ENTITY_TYPE)?;
        self.drop("table", "class")?;
        self.drop("sequence", "EntityKeySequence")?;
        self.drop("sequence", "ClassKeySequence")?;
        self.drop("sequence", "TextAttributeKeySequence")?;
        self.drop("sequence", "QuantityAttributeKeySequence")?;
        self.drop("sequence", "RelationTypeKeySequence")?;
        self.drop("sequence", "ActionKeySequence")?;
        self.drop("sequence", "RelationToEntityKeySequence")?;
        self.drop("sequence", "RelationToRemoteEntityKeySequence")?;
        self.drop("sequence", "RelationToGroupKeySequence")?;
        self.drop("sequence", "RelationToGroupKeySequence2")?;
        self.drop("sequence", "DateAttributeKeySequence")?;
        self.drop("sequence", "BooleanAttributeKeySequence")?;
        self.drop("sequence", "FileAttributeKeySequence")
          }

          fn drop(&self, sql_type: &str, name: &str) -> Result<(), String> {
              let sql: String = format!("DROP {} IF EXISTS {} CASCADE",
                                        Self::escape_quotes_etc(sql_type.to_string()),
                                        Self::escape_quotes_etc(name.to_string()));
              let result: Result<i64, String> = self.db_action(sql.as_str(), false, false);
              match result {
                  Err(msg) => {
                      // (Now that "IF EXISTS" is added in the above DROP statement, this check might
                      // not be needed. No harm though?  If it does not exist pg replies with a
                      // notification, per the pg docs.  Not sure at this writing how that is
                      // reported by sqlx here though.)
                      if !msg.contains("does not exist") {
                          Err(msg.clone())
                      } else {
                          Ok(())
                      }
                  }
                  _ => Ok(())
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
        fn db_action(&self, sql_in: &str, caller_checks_row_count_etc: bool/*%% = false*/,
                       skip_check_for_bad_sql_in: bool/*%% = false*/) -> Result<i64, String> {
            let mut rows_affected = -1;
            //%%let mut st: Statement = null;
            let is_create_drop_or_alter = sql_in.to_lowercase().starts_with("create ") || sql_in.to_lowercase().starts_with("drop ") ||
                                               sql_in.to_lowercase().starts_with("alter ");
              //%% st = connIn.createStatement
              if ! skip_check_for_bad_sql_in {
                Self::check_for_bad_sql(sql_in)?;
              }
              let future = sqlx::query(sql_in).execute(&self.pool);

              let x: Result<PgQueryResult, sqlx::Error> = /*%%: i32 asking compiler or println below*/ block_on(future);
              // /*let y: PgQueryResult = */match x {
              //     Err(e) => return Err(e.to_string()),
              //     Ok(r) => r,
              // };
              if let Err(e) = x {
                  return Err(e.to_string());
              }
                //%%$%%% case e: Exception =>
                //   let msg = "Exception while processing sql: ";
                //   throw new OmDatabaseException(msg + sql_in, e)

                //%%$%%%how get rows_affected, per above??:
              //let rows_affected = st.executeUpdate(sql_in);
                println!("Query result?:  {:?}", &x);

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
                return Err(format!("Affected {} rows instead of 1?? SQL was: {}", rows_affected, sql_in));
              }
              Ok(rows_affected)
        }

    fn check_for_bad_sql(s: &str) -> Result<(), &'static str> {
        if s.contains(";") {
            // it seems that could mean somehow an embedded sql is in a normal command, as an attack vector. We don't usually need
            // to write like that, nor accept it from outside. This & any similar needed checks should happen reliably
            // at the lowest level before the database for security.  If text needs the problematic character(s), it should
            // be escaped prior (see escape_quotes_etc for writing data, and where we read data).
            Err("Input can't contain ';'")
        } else {
            Ok(())
        }
    }

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
    /// open a transaction; then auto-commit will be off until you rollback_trans()
    /// or commit_trans() (or be rolled back upon going out of scope), at which point auto-commit is
    /// turned back on.
    ///
    /// In the scala code this was called login().
    pub fn new(/*%%hopefully del this cmt, was: &self, */username: &str, password: &str) -> Result<Box<dyn Database>, String> {
        let include_archived_entities = false;
        let r = Self::connect(username, username, password);
        let pool: PgPool;
        match r {
            Ok(x) => pool = x,
            Err(e) => return Err(e.to_string()),
        }
        //%% del? works as 'this'?   if !self.model_tables_exist() {
        //   self.create_tables()?;
        // //%%$%%%%% try to see what happens if pg down be4 & during this--does the err propagate ok?
        //   self.create_base_data()?;
        // }
        // //%% doDatabaseUpgradesIfNeeded()
        // self.create_and_check_expected_data()?;
        //
        // Ok(Box::new(PostgreSQLDatabase {
        //     include_archived_entities,
        //     pool,
        // }))
        let this = PostgreSQLDatabase {
            include_archived_entities,
            pool,
        };
        if !this.model_tables_exist()? {
            this.create_tables()?;
            //%%$%%%%% try to see what happens if pg down be4 & during this--does the err propagate ok?
            this.create_base_data()?;
        }
        //%% doDatabaseUpgradesIfNeeded()
        this.create_and_check_expected_data()?;

        Ok(Box::new(this))
    }

          /// For newly-assumed data in existing systems.  I.e., not a database schema change, and was added to the system (probably expected by the code somewhere),
          /// after an OM release was done.  This puts it into existing databases if needed.
          fn create_and_check_expected_data(&self) -> Result<(), String> {
            //Idea: should this really be in the Controller then?  It wouldn't differ by which database type we are using.  Hmm, no, if there were multiple
            // database types, there would probably a parent class over them (of some kind) to hold this.
            let system_entity_id: i64 = self.get_system_entity_id()?;
            let type_id_of_the_has_relation: i64 = self.find_relation_type(Util::THE_HAS_RELATION_TYPE_NAME.to_string())?;

            let preferences_container_id: i64 = {
              let preferences_entity_id: Option<i64> = self.get_relation_to_local_entity_by_name(self.get_system_entity_id()?, Util::USER_PREFERENCES)?;
              match preferences_entity_id {
                  Some(id) => id,
                  None => {
                      // Since necessary, also create the entity that contains all the preferences:
                      //%%$%%%%D?OES THE .0 here get the first one? USED TO BE _1 in scala!
                      let now = Utc::now().timestamp_millis();
                      let new_entity_id: i64 = self.create_entity_and_relation_to_local_entity(system_entity_id, type_id_of_the_has_relation, Util::USER_PREFERENCES, None,
                                                 Some(now), now, false)?.0;
                      new_entity_id
                  }
              }
            };
            // (Not doing the default entity preference here also, because it might not be set by now and is not assumed to be.)
            if self.get_user_preference2(preferences_container_id, Util::SHOW_PUBLIC_PRIVATE_STATUS_PREFERENCE, Util::PREF_TYPE_BOOLEAN)?
                .len() == 0 {
              self.set_user_preference_boolean(Util::SHOW_PUBLIC_PRIVATE_STATUS_PREFERENCE, false);
            }
              Ok(())
          }

    pub fn connect(
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
            .max_connections(1)
            // .connect(connect_str.as_str()).await?;
            //%%$%%% be sure to test this by querying it, ad-hoc for now, later in a test, maybe something like:
            //     om_t1=> show transaction isolation level;
            //     transaction_isolation
            //         -----------------------
            //         read committed
            //         (1 row)
            // (to see the default, instead:   show default_transaction_isolation;
            // or more stuff:   show all;  ).
            //%%do this by sending a query like below per examples, and retrieve info: would work? Or, need to use PgConnectOptions instead of pool?
            //.options([("default_transaction_isolation","serializable")])
            .connect(connect_str.as_str());
        let pool = block_on(future)?;
        //%%$%just some testing, can delete after next commit, or use for a while for reference.
        // // let future = sqlx::query_as("SELECT $1")
        // let future = sqlx::query_as("SELECT count(1) from entity")
        //     .bind(150_i64)
            // OR: .bind("a new ticket (if the sql was insert...)?")
        //     .fetch_one(&pool);
        // let row: (i64, ) = block_on(future).unwrap();
        // // assert_eq!(row.0, 150);
        // println!("Result returned from sql!: {}  ******************************", row.0);

        //%%query examples at:
        //      https://gist.github.com/jeremychone/34d1e3daffc38eb602b1a9ab21298d10
        //      https://betterprogramming.pub/how-to-interact-with-postgresql-from-rust-using-sqlx-cfa2a7c758e7?gi=bfc149911f80
        //      from ddg/web search for:  rust sqlx examples postgres

        //%%the below does not show anything, and it is probably not set.  Maybe later if there is a
        // way to seek support or q/a for sqlx, ask how to set/check it?  Could maybe set it by the
        // options method when getting a single connection (but it seems not to be there for getting
        // a pool).
        let future = sqlx::query("show transaction isolation level").execute(&pool);
        let x = block_on(future)?;
        println!("Query result re transaction isolation lvl?:  {:?}", x);

        Ok(pool)
    }

    /// Indicates whether the database setup has been done.
    fn model_tables_exist(&self) -> Result<bool, String>  {
        self.does_this_exist("select count(1) from pg_class where relname='entity'", true)
    }

    fn create_version_table(&self) -> Result<i64, String> {
        // table has 1 row and 1 column, to say what db version we are on.
        self.db_action("create table om_db_version (version integer DEFAULT 1) ", false, false)?;
        self.db_action("INSERT INTO om_db_version (version) values (0)", false, false)
    }

    /// Does standard setup for a "OneModel" database, such as when starting up for the first time, or when creating a test system.
    /// Currently returns the # of rows affected by the last sql command (not interesting).
    pub fn create_tables(&self) -> Result<i64, String> {
        let tx: Transaction<Postgres> = match self.begin_trans() {
            Err(e) => return Err(e.to_string()),
            Ok(t) => t,
        };

        let mut result: Result<i64, String>;
        // This loop is just to execute once, but allows dropping down to the rollback immediately
        // if necessary.  Maybe there is a cleaner way to do the error handling besides all these
        // breaks!?
        loop {
            result = self.create_version_table();
            if result.is_err() {break;}

            result = self.db_action(format!("create sequence EntityKeySequence minvalue {}", self.min_id_value()).as_str(), false, false);
            if result.is_err() {break;}

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
            result = self.db_action(format!("create table Entity (\
                id bigint DEFAULT nextval('EntityKeySequence') PRIMARY KEY, \
                name varchar({}) NOT NULL, \
                class_id bigint, \
                archived boolean NOT NULL default false, \
                archived_date bigint check ((archived is false and archived_date is null) OR (archived and archived_date is not null)), \
                insertion_date bigint not null, \
                public boolean, \
                new_entries_stick_to_top boolean NOT NULL default false\
                ) ", Util::entity_name_length()).as_str(), false, false);
            if result.is_err() {break;}

            // not unique, but for convenience/speed:
            result = self.db_action("create index entity_lower_name on Entity (lower(NAME))", false, false);
            if result.is_err() {break;}

            result = self.db_action(format!("create sequence ClassKeySequence minvalue {}", self.min_id_value()).as_str(), false, false);
            if result.is_err() {break;}

            // The name here doesn't have to be the same name as in the related Entity record, (since it's not a key, and it might not make sense to match).
            // For additional comments on usage, see the Controller.askForInfoAndcreate_entity method.
            // Since in the code we can't call it class, the class that represents this in the model is called EntityClass.
            result = self.db_action(format!("create table Class (\
                id bigint DEFAULT nextval('ClassKeySequence') PRIMARY KEY, \
                name varchar({}) NOT NULL, \
                // In other words, template, aka class-defining entity:
                defining_entity_id bigint UNIQUE NOT NULL, \
                // this means whether the user wants the program to create all the attributes by default, using the defining_entity's attrs as a template:
                create_default_attributes boolean, \
                CONSTRAINT valid_related_to_entity_id FOREIGN KEY (defining_entity_id) REFERENCES entity (id) \
                ", Util::class_name_length()).as_str(), false, false);
            if result.is_err() {break;}

            result = self.db_action("alter table entity add CONSTRAINT valid_related_to_class_id FOREIGN KEY (class_id) REFERENCES class (id)", false, false);
            if result.is_err() {break;}


            result = self.db_action(format!("create sequence RelationTypeKeySequence minvalue {}", self.min_id_value()).as_str(), false, false);
            if result.is_err() {break;}

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
            result = self.db_action(format!("create table RelationType (\
                entity_id bigint PRIMARY KEY, \
                name_in_reverse_direction varchar({}), \
                directionality char(3) CHECK (directionality in ('BI','UNI','NON')), \
                CONSTRAINT valid_rel_entity_id FOREIGN KEY (entity_id) REFERENCES Entity (id) ON DELETE CASCADE \
                ) ", Util::relation_type_name_length()).as_str(), false, false);
            if result.is_err() {break;}


            /* This table maintains the users' preferred display sorting information for entities' attributes (including relations to groups/entities).

               It might instead have been implemented by putting the sorting_index column on each attribute table, which would simplify some things, but that
               would have required writing a new way for placing & sorting the attributes and finding adjacent ones etc., and the first way was already
               mostly debugged, with much effort (for EntitiesInAGroup, and the hope is to reuse that way for interacting with this table).  But maybe that
               same effect could have been created by sorting the attributes in memory instead, adhoc when needed: not sure if that would be simpler
            */
            result = self.db_action("create table AttributeSorting (\
                // the entity whose attribute this is:
                entity_id bigint NOT NULL\
                // next field is for which table the attribute is in.  Method getAttributeForm has details.
                , attribute_form_id smallint NOT NULL\
                , attribute_id bigint NOT NULL\
                // the reason for this table:
                , sorting_index bigint not null\
                , PRIMARY KEY (entity_id, attribute_form_id, attribute_id)\
                , CONSTRAINT valid_entity_id FOREIGN KEY (entity_id) REFERENCES entity (id) ON DELETE CASCADE\
                , CONSTRAINT valid_attribute_form_id CHECK (attribute_form_id >= 1 AND attribute_form_id <= 8)\
                // make it so the sorting_index must also be unique for each entity (otherwise we have sorting problems):
                , constraint noDupSortingIndexes2 unique (entity_id, sorting_index)\
                // this one was required by the constraint valid_*_sorting on the tables that have a form_id column:
                , constraint noDupSortingIndexes3 unique (attribute_form_id, attribute_id)\
                ) ", false, false);
            if result.is_err() {break;}

            result = self.db_action("create index AttributeSorting_sorted on AttributeSorting (entity_id, sorting_index)", false, false);
            if result.is_err() {break;}

            result = self.create_attribute_sorting_deletion_trigger();
            if result.is_err() {break;}

            result = self.db_action(format!("create sequence QuantityAttributeKeySequence minvalue {}", self.min_id_value()).as_str(), false, false);
            if result.is_err() {break;}

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
            result = self.db_action(format!("create table QuantityAttribute (\
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
                )", quantity_form_id, quantity_form_id).as_str(), false, false);
            if result.is_err() {break;}
            result = self.db_action("create index quantity_parent_id on QuantityAttribute (entity_id)", false, false);
            if result.is_err() {break;}
            result = self.db_action("CREATE TRIGGER qa_attribute_sorting_cleanup BEFORE DELETE ON QuantityAttribute \
                FOR EACH ROW EXECUTE PROCEDURE attribute_sorting_cleanup()", false, false);
            if result.is_err() {break;}

            result = self.db_action(format!("create sequence DateAttributeKeySequence minvalue {}", self.min_id_value()).as_str(), false, false);
            if result.is_err() {break;}
            // see comment for the form_id column under "create table RelationToGroup", below:
            let date_form_id = self.get_attribute_form_id(Util::DATE_TYPE).unwrap();
            result = self.db_action(format!("create table DateAttribute (\
                form_id smallint DEFAULT {} \
                    NOT NULL CHECK (form_id={}), \
                id bigint DEFAULT nextval('DateAttributeKeySequence') PRIMARY KEY, \
                entity_id bigint NOT NULL, \
                //eg, due on, done on, should start on, started on on... (which would be an entity)
                attr_type_id bigint not null, \
                date bigint not null, \
                CONSTRAINT valid_attr_type_id FOREIGN KEY (attr_type_id) REFERENCES entity (id), \
                CONSTRAINT valid_parent_id FOREIGN KEY (entity_id) REFERENCES entity (id) ON DELETE CASCADE, \
                CONSTRAINT valid_da_sorting FOREIGN KEY (entity_id, form_id, id) REFERENCES attributesorting (entity_id, attribute_form_id, attribute_id) \
                  DEFERRABLE INITIALLY DEFERRED \
                ) ", date_form_id, date_form_id).as_str(), false, false);
            if result.is_err() {break;}
            result = self.db_action("create index date_parent_id on DateAttribute (entity_id)", false, false);
            if result.is_err() {break;}
            result = self.db_action("CREATE TRIGGER da_attribute_sorting_cleanup BEFORE DELETE ON DateAttribute \
                FOR EACH ROW EXECUTE PROCEDURE attribute_sorting_cleanup()", false, false);
            if result.is_err() {break;}

            result = self.db_action(format!("create sequence BooleanAttributeKeySequence minvalue {}", self.min_id_value()).as_str(), false, false);
            if result.is_err() {break;}
            let boolean_form_id = self.get_attribute_form_id(Util::BOOLEAN_TYPE).unwrap();
            // See comment for the form_id column under "create table RelationToGroup", below.
            // For the booleanValue column: allowing nulls because a template might not have \
            // value, and a task might not have a "done/not" setting yet (if unknown)?
            // Ex., isDone (where the task would be an entity).
            // See "create table RelationToEntity" for comments about dates' meanings.
            result = self.db_action(format!("create table BooleanAttribute (\
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
                ) ", boolean_form_id, boolean_form_id).as_str(), false, false);
            if result.is_err() {break;}
            //%%try not checking one to see if compiler catches it:
            /*result =*/ self.db_action("create index boolean_parent_id on BooleanAttribute (entity_id)", false, false);
            if result.is_err() {break;}
            result = self.db_action("CREATE TRIGGER ba_attribute_sorting_cleanup BEFORE DELETE ON BooleanAttribute \
                FOR EACH ROW EXECUTE PROCEDURE attribute_sorting_cleanup()", false, false);
            if result.is_err() {break;}

            result = self.db_action(format!("create sequence FileAttributeKeySequence minvalue {}", self.min_id_value()).as_str(), false, false);
            if result.is_err() {break;}
            let file_form_id = self.get_attribute_form_id(Util::FILE_TYPE).unwrap();
            // see comment for form_id under "create table RelationToGroup", below:
            result = self.db_action(format!("create table FileAttribute (\
                form_id smallint DEFAULT {} \
                    NOT NULL CHECK (form_id={}), \
                id bigint DEFAULT nextval('FileAttributeKeySequence') PRIMARY KEY, \
                entity_id bigint NOT NULL, \
                //eg, refers to a type like txt: i.e., could be like mime types, extensions, or mac fork info, etc (which would be an entity in any case).
                attr_type_id bigint NOT NULL, \
                description text NOT NULL, \
                original_file_date bigint NOT NULL, \
                stored_date bigint NOT NULL, \
                original_file_path text NOT NULL, \
                // now that i already wrote this, maybe storing 'readable' is overkill since the system has to read it to store its content. Maybe there's a use.
                readable boolean not null, \
                writable boolean not null, \
                executable boolean not null, \
                //moved to other table:   contents bit varying NOT NULL,
                size bigint NOT NULL, \
                // this is the md5 hash in hex (just to see if doc has become corrupted; not intended for security/encryption)
                md5hash char(32) NOT NULL, \
                CONSTRAINT valid_attr_type_id FOREIGN KEY (attr_type_id) REFERENCES entity (id), \
                CONSTRAINT valid_parent_id FOREIGN KEY (entity_id) REFERENCES entity (id) ON DELETE CASCADE, \
                CONSTRAINT valid_fa_sorting FOREIGN KEY (entity_id, form_id, id) REFERENCES attributesorting (entity_id, attribute_form_id, attribute_id) \
                  DEFERRABLE INITIALLY DEFERRED \
                ) ", file_form_id, file_form_id).as_str(), false, false);
            if result.is_err() {break;}
            result = self.db_action("create index file_parent_id on FileAttribute (entity_id)", false, false);
            if result.is_err() {break;}
            result = self.db_action("CREATE TRIGGER fa_attribute_sorting_cleanup BEFORE DELETE ON FileAttribute \
                FOR EACH ROW EXECUTE PROCEDURE attribute_sorting_cleanup()", false, false);
            if result.is_err() {break;}
            // about oids and large objects, blobs: here are some reference links (but consider also which version of postgresql is running):
            //  https://duckduckgo.com/?q=postgresql+large+binary+streams
            //  http://www.postgresql.org/docs/9.1/interactive/largeobjects.html
            //  https://wiki.postgresql.org/wiki/BinaryFilesInDB
            //  http://jdbc.postgresql.org/documentation/80/binary-data.html
            //  http://artofsystems.blogspot.com/2008/07/mysql-postgresql-and-blob-streaming.html
            //  http://stackoverflow.com/questions/2069541/postgresql-jdbc-and-streaming-blobs
            //  http://giswiki.hsr.ch/PostgreSQL_-_Binary_Large_Objects
            result = self.db_action("CREATE TABLE FileAttributeContent (\
                file_attribute_id bigint PRIMARY KEY, \
                contents_oid lo NOT NULL, \
                CONSTRAINT valid_fileattr_id FOREIGN KEY (file_attribute_id) REFERENCES fileattribute (id) ON DELETE CASCADE \
                )", false, false);
            if result.is_err() {break;}
            // This trigger exists because otherwise the binary data from large objects doesn't get cleaned up when the related rows are deleted. For details
            // see the links just above (especially the wiki one).
            // (The reason I PUT THE "UPDATE OR" in the "BEFORE UPDATE OR DELETE" is simply: that is how this page's example (at least as of 2016-06-01:
            //    http://www.postgresql.org/docs/current/static/lo.html
            // ...said to do it.
            //Idea: but we still might want more tests around it? and to use "vacuumlo" module, per that same url?
            result = self.db_action("CREATE TRIGGER om_contents_oid_cleanup BEFORE UPDATE OR DELETE ON fileattributecontent \
                FOR EACH ROW EXECUTE PROCEDURE lo_manage(contents_oid)", false, false);
            if result.is_err() {break;}

            result = self.db_action(format!("create sequence TextAttributeKeySequence minvalue {}", self.min_id_value()).as_str(), false, false);
            if result.is_err() {break;}
            // the entity_id is the key for the entity on which this text info is recorded; for other meanings see comments on
            // Entity.addQuantityAttribute(...).
            // id must be "unique not null" in ANY database used, because it is the primary key.
            let text_form_id = self.get_attribute_form_id(Util::TEXT_TYPE).unwrap();
            // See comment for column "form_id" under "create table RelationToGroup", below.
            // For attr_type_id:  eg, serial number (which would be an entity).
            // For valid_on_date, see "create table RelationToEntity" for comments about dates' meanings.
            result = self.db_action(format!("create table TextAttribute (\
                form_id smallint DEFAULT {} \
                    NOT NULL CHECK (form_id={}), \
                id bigint DEFAULT nextval('TextAttributeKeySequence') PRIMARY KEY, \
                entity_id bigint NOT NULL, \
                textValue text NOT NULL, \
                attr_type_id bigint not null, \
                valid_on_date bigint, \
                observation_date bigint not null, \
                CONSTRAINT valid_attr_type_id FOREIGN KEY (attr_type_id) REFERENCES entity (id), \
                CONSTRAINT valid_parent_id FOREIGN KEY (entity_id) REFERENCES entity (id) ON DELETE CASCADE, \
                CONSTRAINT valid_ta_sorting FOREIGN KEY (entity_id, form_id, id) REFERENCES attributesorting (entity_id, attribute_form_id, attribute_id) \
                  DEFERRABLE INITIALLY DEFERRED \
                ) ", text_form_id, text_form_id).as_str(), false, false);
            if result.is_err() {break;}
            result = self.db_action("create index text_parent_id on TextAttribute (entity_id)", false, false);
            if result.is_err() {break;}
            result = self.db_action("CREATE TRIGGER ta_attribute_sorting_cleanup BEFORE DELETE ON TextAttribute \
                FOR EACH ROW EXECUTE PROCEDURE attribute_sorting_cleanup()", false, false);
            if result.is_err() {break;}

            result = self.db_action(format!("create sequence RelationToEntityKeySequence minvalue {}", self.min_id_value()).as_str(), false, false);
            if result.is_err() {break;}
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
            let rle_form_id = self.get_attribute_form_id(Util::RELATION_TO_LOCAL_ENTITY_TYPE).unwrap();
            // See comment for form_id, under "create table RelationToGroup", below.
            // The "id" column can be treated like a primary key (with the advantages of being artificial) but the real one is a bit farther down. This one has the
            // slight or irrelevant disadvantage that it artificially limits the # of rows in this table, but it's still a big #.
            // The rel_type_id column is for lookup in RelationType table, eg "has".
            // About the entity_id column: what is related (see RelationConnection for "related to what" (related_to_entity_id).
            // For entity_id_2: the entity_id in RelAttr table is related to what other entity(ies).
            // The valid on date can be null (means no info), or 0 (means 'for all time', not 1970 or whatever that was. At least make it a 1 in that case),
            // or the date it first became valid/true. (The java/scala version of it put in System.currentTimeMillis() for "now"%%--ck if it
            // behaves correctly now when saving/reading/displaying, in milliseconds...? like the call in create_base_data()
            // to create_relation_to_local_entity ?)
            // The observation_date is: whenever first observed (in milliseconds?).
            result = self.db_action(format!("create table RelationToEntity (\
                form_id smallint DEFAULT {}\
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
                ) ", rle_form_id, rle_form_id).as_str(), false, false);
            if result.is_err() {break;}
            result = self.db_action("create index entity_id_1 on RelationToEntity (entity_id)", false, false);
            if result.is_err() {break;}
            result = self.db_action("create index entity_id_2 on RelationToEntity (entity_id_2)", false, false);
            if result.is_err() {break;}
            result = self.db_action("CREATE TRIGGER rte_attribute_sorting_cleanup BEFORE DELETE ON RelationToEntity \
                FOR EACH ROW EXECUTE PROCEDURE attribute_sorting_cleanup()", false, false);
            if result.is_err() {break;}

            // Would rename this sequence to match the table it's used in now, but the cmd "alter sequence relationtogroupkeysequence rename to groupkeysequence;"
            // doesn't rename the name inside the sequence, and keeping the old name is easier for now than deciding whether to do something about that (more info
            // if you search the WWW for "postgresql bug 3619".
            result = self.db_action(format!("create sequence RelationToGroupKeySequence minvalue {}", self.min_id_value()).as_str(), false, false);
            if result.is_err() {break;}
            // This table is named "grupo" because otherwise some queries (like "drop table group") don't work unless "group" is quoted, which doesn't work
            // with mixed case; but forcing the dropped names to lowercase and quoted also prevented dropping class and entity in the same command, it seemed.
            // Avoiding the word "group" as a table in sql might prevent other errors too.
            // Insertion_date is intended to be a readonly date: the (*java*-style numeric: milliseconds
            // since 1970-1-1 or such) when this row was inserted (ie, when the object was created
            // in the db).
            // For new_entries... see comment at same field in Entity table.
            result = self.db_action(format!("create table grupo (\
                id bigint DEFAULT nextval('RelationToGroupKeySequence') PRIMARY KEY, \
                name varchar({}) NOT NULL, \
                insertion_date bigint not null, \
                allow_mixed_classes boolean NOT NULL, \
                new_entries_stick_to_top boolean NOT NULL  default false\
                ) ", Util::entity_name_length()).as_str(), false, false);
            if result.is_err() {break;}

            result = self.db_action(format!("create sequence RelationToGroupKeySequence2 minvalue {}", self.min_id_value()).as_str(), false, false);
            if result.is_err() {break;}
            // The form_id is always the same, and exists to enable the integrity constraint which references it, just below.
            // The id column can be treated like a primary key (with the advantages of being artificial)
            // but the real one is a bit farther down. This one has the slight or irrelevant
            // disadvantage that it artificially limits the # of rows in this table, but it's still a big #.
            // The entity_id is of the containing entity whose attribute (subgroup, RTG) this is.
            // Idea: Should the 2 dates be eliminated? The code is there, including in the parent class, and they might be useful,
            // maybe no harm while we wait & see.
            // See "create table RelationToEntity" for comments about dates' meanings.
            let rtg_form_id = self.get_attribute_form_id(Util::RELATION_TO_GROUP_TYPE).unwrap();
            result = self.db_action(format!("create table RelationToGroup (\
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
                ) ", rtg_form_id, rtg_form_id).as_str(), false, false);
            if result.is_err() {break;}
            result = self.db_action("create index RTG_entity_id on RelationToGroup (entity_id)", false, false);
            if result.is_err() {break;}
            result = self.db_action("create index RTG_group_id on RelationToGroup (group_id)", false, false);
            if result.is_err() {break;}
            result = self.db_action("CREATE TRIGGER rtg_attribute_sorting_cleanup BEFORE DELETE ON RelationToGroup \
                FOR EACH ROW EXECUTE PROCEDURE attribute_sorting_cleanup()", false, false);
            if result.is_err() {break;}

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
            result = self.db_action("create table EntitiesInAGroup (\
                group_id bigint NOT NULL\
                , entity_id bigint NOT NULL\
                , sorting_index bigint not null\
                // the key is really the group_id + entity_id, and the sorting_index is just in an index so we can cheaply order query results
                // When sorting_index was part of the key there were ongoing various problems because the rest of the system (like reordering results, but
                // probably also other issues) wasn't ready to handle two of the same entity in a group.
                , PRIMARY KEY (group_id, entity_id)\
                , CONSTRAINT valid_group_id FOREIGN KEY (group_id) REFERENCES grupo (id) ON DELETE CASCADE\
                , CONSTRAINT valid_entity_id FOREIGN KEY (entity_id) REFERENCES entity (id)\
                // make it so the sorting_index must also be unique for each group (otherwise we have sorting problems):
                , constraint noDupSortingIndexes unique (group_id, sorting_index)\
                ) ", false, false);
            if result.is_err() {break;}
            result = self.db_action("create index EntitiesInAGroup_id on EntitiesInAGroup (entity_id)", false, false);
            if result.is_err() {break;}
            result = self.db_action("create index EntitiesInAGroup_sorted on EntitiesInAGroup (group_id, entity_id, sorting_index)", false, false);
            if result.is_err() {break;}

            result = self.db_action(format!("create sequence ActionKeySequence minvalue {}", self.min_id_value()).as_str(), false, false);
            if result.is_err() {break;}
            result = self.db_action(format!("create table Action (\
                id bigint DEFAULT nextval('ActionKeySequence') PRIMARY KEY, \
                class_id bigint NOT NULL, \
                name varchar({}) NOT NULL, \
                action varchar({}) NOT NULL, \
                CONSTRAINT valid_related_to_class_id FOREIGN KEY (class_id) REFERENCES Class (id) ON DELETE CASCADE \
                ) ", Util::entity_name_length(), Util::entity_name_length()).as_str(), false, false);
            if result.is_err() {break;}
            result = self.db_action("create index action_class_id on Action (class_id)", false, false);
            if result.is_err() {break;}

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
            result = self.db_action(format!("create table OmInstance (\
                id uuid PRIMARY KEY\
                , local boolean NOT NULL\
                , address varchar({}) NOT NULL\
                , insertion_date bigint not null\
                , entity_id bigint REFERENCES entity (id) ON DELETE RESTRICT\
                ) ", self.om_instance_address_length()).as_str(), false, false);
            if result.is_err() {break;}

            result = self.db_action(format!("create sequence RelationToRemoteEntityKeySequence minvalue {}", self.min_id_value()).as_str(), false, false);
            if result.is_err() {break;}
            // See comments on "create table RelationToEntity" above for comparison & some info, as well as class comments on RelationToRemoteEntity.
            // The difference here is (at least that) this has a field pointing
            // to a remote OM instance.  The Entity with id entity_id_2 is contained in that remote OM instance, not in the current one.
            // (About remote_instance_id: see comment just above.)
            // (See comment above about entity_id_2.)
            // About constraint valid_remote_instance_id below:
            // deletions of the referenced rows should warn the user that these will be deleted also.  The same should also be true for all
            // other uses of "ON DELETE CASCADE".
            let rtre_form_id = self.get_attribute_form_id(Util::RELATION_TO_REMOTE_ENTITY_TYPE).unwrap();
            result = self.db_action(format! ("create table RelationToRemoteEntity (\
                form_id smallint DEFAULT {}\
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
                ) ", rtre_form_id, rtre_form_id).as_str(), false, false);
            if result.is_err() {break;}
            result = self.db_action("create index rtre_entity_id_1 on RelationToRemoteEntity (entity_id)", false, false);
            if result.is_err() {break;}
            result = self.db_action("create index rtre_entity_id_2 on RelationToRemoteEntity (entity_id_2)", false, false);
            if result.is_err() {break;}
            result = self.db_action("CREATE TRIGGER rtre_attribute_sorting_cleanup BEFORE DELETE ON RelationToRemoteEntity \
                FOR EACH ROW EXECUTE PROCEDURE attribute_sorting_cleanup()", false, false);
            if result.is_err() {break;}

            result = self.db_action(format!("UPDATE om_db_version SET (version) = ROW({})", PostgreSQLDatabase::SCHEMA_VERSION).as_str(), false, false);
            if result.is_err() {break;}

            match self.commit_trans(tx) {
                Err(e) => return Err(e.to_string()),
                _ => {},
            }

            // see comment at top of loop
            break;
        }
        if result.is_err() {
            //%% see "rollback problem" prefixed with %%, for comments on this.
            //%%$%%%%%%NOTE: IN ALL USES OF THIS, IS THERE ANY POINT, GIVEN THAT ROLLBACK HAPPENS WHEN xactn goes OUT OF SCOPE?
            // CLEAN UP ALL OF THEM, carefully??
            // let _rollback_results = self.rollback_trans(tx);
            println!("set brkpt here to see if it rolls back when going out of scope shortly?%%");
        }
        result
    }

    fn create_attribute_sorting_deletion_trigger(&self) -> Result<i64, String> {
        // Each time an attribute (or rte/rtg) is deleted, the AttributeSorting row should be deleted too, in an enforced way (or it had sorting problems, for one).
        // I.e., an attempt to enforce (with triggers that call this procedure) that the AttributeSorting table's attribute_id value is found
        // in *one of the* 7 attribute tables' id column,  Doing it in application code is not as simple or as reliable as doing it at the DDL level.
        let sql = "CREATE OR REPLACE FUNCTION attribute_sorting_cleanup() RETURNS trigger AS $attribute_sorting_cleanup$ \
          BEGIN\
            // (OLD is a special PL/pgsql variable of type RECORD, which contains the attribute row before the deletion.)
                DELETE FROM AttributeSorting WHERE entity_id=OLD.entity_id and attribute_form_id=OLD.form_id and attribute_id=OLD.id; \
                RETURN OLD; \
              END;\
            $attribute_sorting_cleanup$ LANGUAGE plpgsql;";
        self.db_action(sql, false, true)
    }

    /// Creates data that must exist in a base system, and which is not re-created in an existing system.  If this data is deleted, the system might not work.
    fn create_base_data(&self) -> Result<(), String> {
        // idea: what tests are best, around this, vs. simply being careful in upgrade scripts?
        let ids: Vec<i64> = self.find_entity_only_ids_by_name(Util::SYSTEM_ENTITY_NAME.to_string())?;
        // will probably have to change the next line when things grow/change, and, maybe, we're doing upgrades not always a new system:
        assert!(ids.is_empty());

        // public=false, guessing at best value, since the world wants your modeled info, not
        // details about your system internals (which might be...unique & personal somehow)?:
        let system_entity_id = self.create_entity(Util::SYSTEM_ENTITY_NAME, None, Some(false))?;

        let existence_entity_id = self.create_entity("existence", None, Some(false))?;
        //idea: as probably mentioned elsewhere, this "BI" (and other strings?) should be replaced with a constant somewhere (or enum?)!
        let has_rel_type_id = self.create_relation_type(Util::THE_HAS_RELATION_TYPE_NAME, Util::THE_IS_HAD_BY_REVERSE_NAME, "BI")?;
        //%%$%%%%
        //%%does this save/retrieve (comparing new data w/ this change, and old data from scala) accurately w/ what we want?:
        let current_time_millis = Utc::now().timestamp_millis();
        self.create_relation_to_local_entity(has_rel_type_id, system_entity_id, existence_entity_id,
                                             Some(current_time_millis), current_time_millis, None, false)?;

        let editor_info_entity_id = self.create_entity(Util::EDITOR_INFO_ENTITY_NAME, None, Some(false))?;
        self.create_relation_to_local_entity(has_rel_type_id, system_entity_id, editor_info_entity_id, Some(current_time_millis), current_time_millis, None, false)?;
        let text_editor_info_entity_id = self.create_entity(Util::TEXT_EDITOR_INFO_ENTITY_NAME, None, Some(false))?;
        self.create_relation_to_local_entity(has_rel_type_id, editor_info_entity_id, text_editor_info_entity_id, Some(current_time_millis), current_time_millis, None, false)?;
        let text_editor_command_attribute_type_id = self.create_entity(Util::TEXT_EDITOR_COMMAND_ATTRIBUTE_TYPE_NAME, None, Some(false))?;
        self.create_relation_to_local_entity(has_rel_type_id, text_editor_info_entity_id, text_editor_command_attribute_type_id, Some(current_time_millis), current_time_millis, None, false)?;
        let editor_command: &str = {
            if Util::is_windows() {
                "notepad"
            } else {
                "vi"
            }
        };
        self.create_text_attribute(text_editor_info_entity_id, text_editor_command_attribute_type_id, editor_command, Some(current_time_millis), current_time_millis, false, None)?;

        // the intent of this group is user convenience: the app shouldn't rely on this group to find classDefiningEntities (templates), but use the relevant table.
        // idea: REALLY, this should probably be replaced with a query to the class table: so, when queries as menu options are part of the OM
        // features, put them all there instead.
        // It is set to allowMixedClassesInGroup just because no current known reason not to; will be interesting to see what comes of it.
        self.create_group_and_relation_to_group(system_entity_id, has_rel_type_id, Util::CLASS_TEMPLATE_ENTITY_GROUP_NAME, /*%%allow_mixed_classes_in_group_in =*/ true,
                                      Some(current_time_millis), current_time_millis, None, false)?;

        // NOTICE: code should not rely on this name, but on data in the tables.
        /*val (class_id, entity_id) = */ self.create_class_and_its_template_entity("person".to_string());
        // (should be same as the line in upgradeDbFrom3to4(), or when combined with later such methods, .)
        let uuid = uuid::Uuid::new_v4();
        println!("bytes: {:?}", uuid.as_bytes());
        println!("simple: {}", uuid.simple());
        println!("hyphenated: {}", uuid.hyphenated());
        println!("urn: {}", uuid.urn());
        println!("tostring: {}", uuid.to_string());
        self.create_om_instance(/*%%which from above!?*?*/uuid.to_string(), true, Util::LOCAL_OM_INSTANCE_DEFAULT_DESCRIPTION.to_string(),
                                None, false)?;
        Ok(())
    }

    /// Case-insensitive.
    fn find_entity_only_ids_by_name(&self, name_in: String) -> Result<Vec<i64>, String> {
        // idea: see if queries like this are using the expected index (run & ck the query plan). Tests around that, for benefit of future dbs? Or, just wait for
        // a performance issue then look at it?
        let include_archived: &str = if !self.include_archived_entities() {
            "(not archived) and "
        } else {
            ""
        };
        let the_rest = format!("lower(name) = lower('{}') {} ", name_in, Self::limit_to_entities_only(Self::ENTITY_ONLY_SELECT_PART.to_string()));
        let rows: Vec<Vec<DataType>> = self.db_query(format!("select id from entity where {}{}", include_archived, the_rest).as_str(), "i64")?;
        // if rows.isEmpty None
        // else {
        let mut results: Vec<i64> = Vec::new();
        for row in rows.iter() {
            // results = row(0).get.asInstanceOf[i64] :: results
            let id = match row[0] {
                DataType::Bigint(x) => x,
                // next line is intended to be impossible, based on the query
                _ => return Err("This should never happen.".to_string()),
            };
            results.push(id);
        }
        results.reverse();
        Ok(results)
        // }
    }

    /** Returns the class_id and entity_id, in a tuple. */
    fn create_class_and_its_template_entity2(&self, class_name_in: String, entity_name_in: String) -> Result<(i64, i64), String> {
        // The name doesn't have to be the same on the entity and the template class, but why not for now.
        let class_name: String = Self::escape_quotes_etc(class_name_in);
        let entity_name: String = Self::escape_quotes_etc(entity_name_in);
        if class_name.len() == 0 {
            return Err("Class name must have a value.".to_string());
        }
        if entity_name.len() == 0 {
            return Err("Entity name must have a value.".to_string());
        }
        let class_id: i64 = self.get_new_key("ClassKeySequence")?;
        let entity_id: i64 = self.get_new_key("EntityKeySequence")?;
        let tx: Transaction<Postgres> = match self.begin_trans() {
            Err(e) => return Err(e.to_string()),
            Ok(t) => t,
        };
        // Start the entity w/ a NULL class_id so that it can be inserted w/o the class present, then update it afterward; constraints complain otherwise.
        // Idea: instead of doing in 3 steps, could specify 'deferred' on the 'not null'
        // constraint?: (see file:///usr/share/doc/postgresql-doc-9.1/html/sql-createtable.html).
        self.db_action(format!("INSERT INTO Entity (id, insertion_date, name, class_id) VALUES ({},{},'{}', NULL)", entity_id, Utc::now().timestamp_millis(), entity_name).as_str(), false, false)?;
        self.db_action(format!("INSERT INTO Class (id, name, defining_entity_id) VALUES ({},'{}', {})", class_id, class_name, entity_id).as_str(), false, false)?;
        self.db_action(format!("update Entity set (class_id) = ROW({}) where id={}", class_id, entity_id).as_str(), false, false)?;
        //%%should move this transaction down a few lines?? why didn't, before?
        match self.commit_trans(tx) {
            Err(e) => return Err(e.to_string()),
            _ => {},
        }

        let class_group_id: Option<i64> = self.get_system_entitys_class_group_id()?;
        if class_group_id.is_some() {
            self.add_entity_to_group(class_group_id.unwrap(), entity_id, None, false)?;
        }

        Ok((class_id, entity_id))
    }

    /// Returns the id of a specific group under the system entity.  This group is the one that contains class-defining (template) entities.
    fn get_system_entitys_class_group_id(&self) -> Result<Option<i64>, String> {
        let system_entity_id: i64 = self.get_system_entity_id()?;

        // idea: maybe this stuff would be less breakable by the user if we put this kind of info in some system table
        // instead of in this group. (See also method create_base_data).  Or maybe it doesn't matter, since it's just a user convenience. Hmm.
        let class_template_group_id = self.find_relation_to_and_group_on_entity(system_entity_id, Some(Util::CLASS_TEMPLATE_ENTITY_GROUP_NAME.to_string()))?.2;
        if class_template_group_id.is_none() {
            // no exception thrown here because really this group is a convenience for the user to see things, not a requirement. Maybe a user message would be best:
            // "Idea:: BAD SMELL! The UI should do all UI communication, no?"  Maybe, pass in a UI object instead and call some generic method that will handle
            // the info properly?  Or have logs?
            // (SEE ALSO comments and code at other places with the part on previous line in quotes).
            eprintln!("Unable to find, from the entity {}({}), any connection to its \
            expected contained group {}.  If it was deleted, it could be replaced if you want the \
            convenience of finding template entities in it.",
                Util::SYSTEM_ENTITY_NAME, system_entity_id, Util::CLASS_TEMPLATE_ENTITY_GROUP_NAME,
                );
        }
        Ok(class_template_group_id)
    }

    /// Although the next sequence value would be set automatically as the default for a column (at least the
    /// way I have them defined so far in postgresql); we do it explicitly
    /// so we know what sequence value to return, and what the unique key is of the row we just created!
    fn get_new_key(&self, sequenceNameIn: &str)  -> /*id*/ Result<i64, String>  {
        let row: Vec<DataType> = self.db_query_wrapper_for_one_row(format!("SELECT nextval('{}')", sequenceNameIn), "i64")?;
        if row.is_empty() {
            return Err("No elements found, in get_new_key().".to_string());
        } else {
            match row[0] {
                // None => return Err("None found, in get_new_key()."),
                // Some(DataType::Bigint(new_id)) => Ok(new_id),
                DataType::Bigint(new_id) => Ok(new_id),
                _ => return Err("In get_new_key() this should never happen".to_string()),
            }
        }
    }

    fn are_mixed_classes_allowed(&self, group_id: i64) -> Result<bool, String> {
        let rows: Vec<Vec<DataType>> = self.db_query(format!("select allow_mixed_classes from grupo where id ={}", group_id).as_str(), "Boolean")?;
        let mixed_classes_allowed: bool = match rows[0][0] {
            DataType::Boolean(b) => b,
            _ => return Err("This should never happen".to_string()),
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
                _ => return Err("This should never happen."),
            };
            results.push(id);
        }
        Ok(results.reverse)
 */
    fn has_mixed_classes(&self, group_id_in: i64) -> Result<bool, String> {
        // Enforce that all entities in so-marked groups have the same class (or they all have no class; too bad).
        // (This could be removed or modified, but some user scripts attached to groups might (someday?) rely on their uniformity, so this
        // and the fact that you can have a group all of which don't have any class, is experimental.  This is optional, per
        // group.  I.e., trying it that way now to see whether it removes desired flexibility
        // at a cost higher than the benefit of uniformity for later user code operating on groups.  This might be better in a constraint,
        // but after trying for a while I hadn't made the syntax work right.

        // (Had to ask for them all and expect 1, instead of doing a count, because for some reason "select count(class_id) ... group by class_id" doesn't
        // group, and you get > 1 when I wanted just 1. This way it seems to work if I just check the # of rows returned.)
        let rows: Vec<Vec<DataType>> = self.db_query(format!("select class_id from EntitiesInAGroup eiag, entity e \
            where eiag.entity_id=e.id and group_id={} and class_id is not null group by class_id",
                                                        group_id_in).as_str(), "i64")?;
        let num_classes_in_group_entities = rows.len();
        // nulls don't show up in a count(class_id), so get those separately
        let num_null_classes_in_group_entities = self.extract_row_count_from_count_query(format!("select count(entity_id) from EntitiesInAGroup \
            eiag, entity e where eiag.entity_id=e.id and group_id={} and class_id is NULL ", group_id_in).as_str())?;
        if num_classes_in_group_entities > 1 ||
            (num_classes_in_group_entities >= 1 && num_null_classes_in_group_entities > 0) {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn limit_to_entities_only(selectColumnNames: String) -> String {
        // IN MAINTENANCE: compare to logic in method getEntitiesUsedAsAttributeTypes_sql, and related/similar logic near the top of
        // Controller.chooseOrCreateObject (if it is still there; as of
        // 2017-8-21 starts with "val (numObjectsAvailable: i64, showOnlyAttributeTypes: bool) = {".
        let mut sql: String = String::new();
        sql.push_str("except (");
        sql.push_str(selectColumnNames.as_str());
        sql.push_str(" from entity e, quantityattribute q where e.id=q.unit_id) ");
        sql.push_str("except (");
        sql.push_str(selectColumnNames.as_str());
        sql.push_str(" from entity e, quantityattribute q where e.id=q.attr_type_id) ");
        sql.push_str("except (");
        sql.push_str(selectColumnNames.as_str());
        sql.push_str(" from entity e, dateattribute t where e.id=t.attr_type_id) ");
        sql.push_str("except (");
        sql.push_str(selectColumnNames.as_str());
        sql.push_str(" from entity e, booleanattribute t where e.id=t.attr_type_id) ");
        sql.push_str("except (");
        sql.push_str(selectColumnNames.as_str());
        sql.push_str(" from entity e, fileattribute t where e.id=t.attr_type_id) ");
        sql.push_str("except (");
        sql.push_str(selectColumnNames.as_str());
        sql.push_str(" from entity e, textattribute t where e.id=t.attr_type_id) ");
        sql.push_str("except (");
        sql.push_str(selectColumnNames.as_str());
        sql.push_str(" from entity e, relationtype t where e.id=t.entity_id) ");
        sql
    }

    /// @param sorting_index_in is currently passed by callers with a default guess, not a guaranteed good value, so if it is in use, this ~tries to find a good one.
    ///                       An alternate approach could be to pass in a callback to code like in SortableEntriesMenu.placeEntryInPosition (or what it calls),
    ///                       which this can call if it thinks it
    ///                       is taking a long time to find a free value, to give the eventual caller chance to give up if needed.  Or just pass in a known
    ///                       good value or call the renumberSortingIndexes method in SortableEntriesMenu.
    /// @return the sorting_index value that is actually used.
    fn add_attribute_sorting_row(&self, entity_id_in: i64, attribute_form_id_in: i32,
                                 attribute_id_in: i64, sorting_index_in: Option<i64>/*%% = None*/)
        -> Result<i64, String> {
        // SEE COMMENTS IN SIMILAR METHOD: add_entity_to_group.  **AND DO MAINTENANCE. IN BOTH PLACES.
        // Should probably be called from inside a transaction (which isn't managed in this method, since all its current callers do it.)
        let sorting_index: i64 = {
            let index = {
                if sorting_index_in.is_some() {
                    sorting_index_in.unwrap()
                } else if self.get_attribute_count(entity_id_in, false)? == 0 {
                    // start with an increment off the min or max, so that later there is room to sort something before or after it, manually:
                    self.min_id_value() + 99999
                } else {
                    self.max_id_value() - 99999
                }
            };
            if self.is_attribute_sorting_index_in_use(entity_id_in, index)? {
                self.find_unused_attribute_sorting_index(entity_id_in, None)?
            } else {
                index
            }
        };
        self.db_action(format!("insert into AttributeSorting (entity_id, attribute_form_id, attribute_id, sorting_index) \
            values ({},{},{},{})", entity_id_in, attribute_form_id_in, attribute_id_in, sorting_index).as_str(),
                       false, false)?;
        Ok(sorting_index)
    }

    fn is_attribute_sorting_index_in_use(&self, entity_id_in: i64, sorting_index_in: i64) -> Result<bool, String> {
        self.does_this_exist(format!("SELECT count(1) from AttributeSorting where entity_id={} and sorting_index={}",
            entity_id_in, sorting_index_in).as_str(), true)
    }

    fn get_system_entity_id(&self) -> Result<i64, String> {
        let ids: Vec<i64> = self.find_entity_only_ids_by_name(Util::SYSTEM_ENTITY_NAME.to_string())?;
        if ids.is_empty() {
            return Err(format!("No system entity id (named \"{}\") was \
                 found in the entity table.  Did a new data import fail partway through or \
                 something?", Util::SYSTEM_ENTITY_NAME));
        }
        assert_eq!(ids.len(), 1);
        Ok(ids[0])
    }

    // Cloned to archiveObjects: CONSIDER UPDATING BOTH if updating one.  Returns the # of rows deleted.
    /// Unless the parameter rows_expected==-1, it will allow any # of rows to be deleted; otherwise if the # of rows is wrong it will abort tran & fail.
    fn delete_objects(&self, table_name_in: &str, where_clause_in: &str, rows_expected: i64 /*%%= 1*/, caller_manages_transactions_in: bool /*%%= false*/)
        -> Result<i64, String> {
        //idea: enhance this to also check & return the # of rows deleted, to the caller to just make sure? If so would have to let caller handle transactions.
        let sql = format!("DELETE FROM {} {}", table_name_in, where_clause_in);

        let tx: Option<Transaction<Postgres>> = if !caller_manages_transactions_in {
            match self.begin_trans() {
                Err(e) => return Err(e.to_string()),
                Ok(t) => Some(t),
            }
        } else { None };

        let rows_deleted = self.db_action(sql.as_str(), /*%%caller_checks_row_count_etc =*/ true, false)?;
        if rows_expected >= 0 && rows_deleted != rows_expected {
            // Roll back, as we definitely don't want to delete an unexpected # of rows.
            // Do it ***EVEN THOUGH callerManagesTransaction IS true***: seems cleaner/safer this way.
            //%%see cmts at "rollback problem" for ideas on this?
            // throw rollbackWithCatch(new OmDatabaseException("Delete command would have removed " + rows_deleted + " rows, but " +
            //     rows_expected + " were expected! Did not perform delete.  SQL is: \"" + sql + "\""))
            //No, don't roll back as we don't have the transaction here -- the caller needs to handle it.
            // match self.rollback_trans(tx_vec[0]) {
            //     Err(e) => return Err(e.to_string()),
            //     _ => (),
            // }
            //%%$%%%%make sure caller rolls back!:
            return Err(format!("Delete command would have removed {} rows, but {} were expected! Did not perform delete.  SQL is: \"{}\"",
                               rows_deleted, rows_expected, sql));
        } else {
            if !caller_manages_transactions_in {
                if let Err(e) = self.commit_trans(tx.unwrap()) {
                    return Err(e.to_string());
                }
            }
            Ok(rows_deleted)
        }
    }

    fn get_user_preference2(&self, preferences_container_id_in: i64, preference_name_in: &str, preference_type: &str) -> Result<Vec<DataType>, String> {
        // (Passing a smaller numeric parameter to find_contained_local_entity_ids for levels_remainingIn, so that in the (very rare) case where one does not
        // have a default entity set at the *top* level of the preferences under the system entity, and there are links there to entities with many links
        // to others, then it still won't take too long to traverse them all at startup when searching for the default entity.  But still allowing for
        // preferences to be nested up to that many levels (3 as of this writing).
        let mut set: HashSet<i64> = HashSet::new();
        let found_preferences: &mut HashSet<i64> = self.find_contained_local_entity_ids(&mut set, preferences_container_id_in, preference_name_in, 3, true)?;
        if found_preferences.len() == 0 {
            // let empty_vec: Vec<DataType> = Vec::new();
            // Ok(empty_vec)
            Ok(Vec::new())
        } else {
            if found_preferences.len() != 1 {
                let pref_container_entity_name = match self.get_entity_name(preferences_container_id_in)? {
                    None => "(None)".to_string(),
                    Some(x) => x,
                };
                return Err(format!("Under the entity \"{}\" ({}, possibly under {}), there \
                        are (eventually) more than one entity with the name \"{}\", so the program does not know which one to use for this.",
                                   pref_container_entity_name, preferences_container_id_in, Util::SYSTEM_ENTITY_NAME, preference_name_in));
            }
            let mut preference_entity_id: i64 = 0;
            for x in found_preferences.iter() {
                // there is exactly one, as checked above
                preference_entity_id = *x;
            }
            let preference_entity = Entity::new2(Box::new(self), preference_entity_id);
            let relevant_attribute_rows: Vec<Vec<DataType>> = {
                if preference_type == Util::PREF_TYPE_BOOLEAN {
                    // (Using the preference_entity.get_id for attr_type_id, just for convenience since it seemed as good as any.  ALSO USED IN THE SAME WAY,
                    // IN setUserPreference METHOD CALL TO create_boolean_attribute!)
                    let sql2 = format!("select id, booleanValue from booleanattribute where entity_id={} and attr_type_id={}", preference_entity_id, preference_entity_id);
                    self.db_query(sql2.as_str(), "i64,Boolean")?
                } else if preference_type == Util::PREF_TYPE_ENTITY_ID {
                    let sql2 = format!("select rel_type_id, entity_id, entity_id_2 from relationtoentity where entity_id={}", preference_entity_id);
                    self.db_query(sql2.as_str(), "i64,i64,i64")?
                } else {
                    return Err(format!("Unexpected preference_type: {}", preference_type));
                }
            };
            if relevant_attribute_rows.len() == 0 {
                // at this point we probably have a preference entity but not the expected attribute inside it that holds the actual useful information, so the
                // user needs to go delete the bad preference entity or re-create the attribute.
                // Idea: should there be a good way to *tell* them that, from here?
                // Or, just delete the bad preference (self-cleanup). If it was the public/private display toggle, its absence will cause errors (though it is a
                // very unlikely situation here), and it will be fixed on restarting the app (or starting another instance), via the createExpectedData
                // (or current equivalent?) method.
                self.delete_entity(preference_entity_id, false)?;
                Ok(Vec::new())
            } else {
                let attr_msg: String = if preference_type == Util::PREF_TYPE_BOOLEAN {
                    format!(" BooleanAttributes with the relevant type ({},{}), ", preference_name_in, preferences_container_id_in)
                } else if preference_type == Util::PREF_TYPE_ENTITY_ID {
                    " RelationToEntity values ".to_string()
                } else {
                    return Err(format! ("Unexpected preference_type: {}", preference_type));
                };

                if relevant_attribute_rows.len() != 1 {
                    // ASSUMED it is 1, below!
                    // preference_entity.get_id()
                    let (pref_entity_name, id) = match preference_entity {
                        Err(e) => (format!("(Unknown/error: {})", e.to_string()), 0_i64),
                        Ok(mut entity) => (entity.get_name()?.clone(), entity.get_id()),
                    };
                    //delme
                    // let pref_entity_name = match self.get_entity_name(id)? {
                    //     None => "(None)".to_string(),
                    //     Some(s) => s,
                    // };
                    return Err(format!("Under the entity {} ({}), there are {}{}so the program does not know what to use for this.  There should be *one*.",
                                       pref_entity_name,
                                        id,
                                       relevant_attribute_rows.len(), attr_msg));
                }
                if preference_type == Util::PREF_TYPE_BOOLEAN {
                    //PROVEN to have 1 row, just above!
                    // let DataType::Bigint(preferenceId) = relevant_attribute_rows[0][0];
                    let preferenceId: DataType/*i64*/ = relevant_attribute_rows[0][0].clone();
                    // let DataType::Boolean(preferenceValue) = relevant_attribute_rows[0][1];
                    let preferenceValue: DataType/*bool*/ = relevant_attribute_rows[0][1].clone();
                    Ok(vec![preferenceId, preferenceValue])
                } else if preference_type == Util::PREF_TYPE_ENTITY_ID {
                    //PROVEN to have 1 row, just above!
                    let relTypeId: DataType/*i64*/ = relevant_attribute_rows[0][0].clone();
                    let entity_id1: DataType/*i64*/ = relevant_attribute_rows[0][1].clone();
                    let entity_id2: DataType/*i64*/ = relevant_attribute_rows[0][2].clone();
                    Ok(vec![relTypeId, entity_id1, entity_id2])
                } else {
                    return Err(format!("Unexpected preference_type: {}", preference_type));
                }
            }
        }
    }

    fn get_relation_to_local_entity_by_name(&self, containing_entity_id_in: i64, name_in: &str) -> Result<Option<i64>, String> {
        let if_not_archived = if !self.include_archived_entities {
            " and (not e.archived)"
        } else { "" };
        let sql = format!("select rte.entity_id_2 from relationtoentity rte, entity e where \
            rte.entity_id={}{} and rte.entity_id_2=e.id and e.name='{}'",
            containing_entity_id_in, if_not_archived, name_in);
        let related_entity_id_rows = self.db_query(sql.as_str(), "i64")?;
        if related_entity_id_rows.len() == 0 {
            Ok(None)
        } else {
            if related_entity_id_rows.len() != 1 {
                let containing_entity_name = match self.get_entity_name(containing_entity_id_in)? {
                    None => "(None)".to_string(),
                    Some(s) => s,
                };
                return Err(format!("Under the entity {}({}), there is more one than entity with the name \"{}\", so the program does not know which one to use for this.",
                           containing_entity_name, containing_entity_id_in,
                    Util::USER_PREFERENCES));
            }

            //idea: surely there is some better way than what I am doing here? See other places similarly.
            // let DataType::Bigint(id) = related_entity_id_rows[0][0];
            let id = match related_entity_id_rows[0][0] {
                DataType::Bigint(x) => x,
                _ => return Err(format!("How did we get here for {:?}?", related_entity_id_rows[0][0])),
            };
            Ok(Some(id))
        }
    }

    fn get_quantity_attribute_count(&self, entity_id_in: i64) -> Result<i64, String> {
        self.extract_row_count_from_count_query(format!("select count(1) from QuantityAttribute where entity_id={}", entity_id_in).as_str())
    }

    fn get_text_attribute_count(&self, entity_id_in: i64) -> Result<i64, String> {
        self.extract_row_count_from_count_query(format!("select count(1) from TextAttribute where entity_id={}", entity_id_in).as_str())
    }

    fn get_date_attribute_count(&self, entity_id_in: i64) -> Result<i64, String> {
        self.extract_row_count_from_count_query(format!("select count(1) from DateAttribute where entity_id={}", entity_id_in).as_str())
    }

    fn get_boolean_attribute_count(&self, entity_id_in: i64) -> Result<i64, String> {
        self.extract_row_count_from_count_query(format!("select count(1) from BooleanAttribute where entity_id={}", entity_id_in).as_str())
    }

    fn get_file_attribute_count(&self, entity_id_in: i64) -> Result<i64, String> {
        self.extract_row_count_from_count_query(format!("select count(1) from FileAttribute where entity_id={}", entity_id_in).as_str())
    }

    /// Used for example after one has been deleted, to put the highlight on right next one:
    /// idea: This feels overcomplicated.  Make it better?  Fixing bad smells in general (large classes etc etc) is on the task list.
    /**%%fix doc formatting:
         * @param object_set_size  # of all the possible entries, not reduced by what fits in the available display space (I think).
         * @param objects_to_display_in  Only those that have been chosen to display (ie, smaller list to fit in display size size) (I think).
         * @return
     */
    fn find_entity_to_highlight_next<'a>(
        &'a self,
        object_set_size: usize,
        objects_to_display_in: Vec<Entity>,
        removed_one_in: bool,
        previously_highlighted_index_in_obj_list_in: usize,
        previously_highlighted_entry_in: Entity<'a>,
    ) -> Result<Option<Entity<'a>>, String> {
        //NOTE: SIMILAR TO find_attribute_to_highlight_next: WHEN MAINTAINING ONE, DO SIMILARLY ON THE OTHER, until they are merged maybe by using the type
        //system better.

        // Here of course, previously_highlighted_index_in_obj_list_in and obj_ids.size were calculated prior to the deletion.

        if removed_one_in {
            if object_set_size <= 1 {
                return Ok(None);
            }
            let new_obj_list_size: usize = object_set_size - 1;
            if new_obj_list_size == 0 {
                //(redundant with above test/None, but for clarity in reading)
                Ok(None)
            } else {
                let mut new_index_to_highlight = std::cmp::min(
                    new_obj_list_size - 1,
                    previously_highlighted_index_in_obj_list_in,
                );
                // if new_index_to_highlight != previously_highlighted_index_in_obj_list_in {
                //     // %%why doesn't Rust know the element is an Entity, vs. <Unknown>? why can't just return
                //     // objects_to_display_in.get(new_index_to_highlight)? Maybe rustc would do OK but the IDE doesn't? try changing at first 1 of the
                //     // 3 below places back, and see if rustc gets it right? or am I mistaken?
                //     match objects_to_display_in.get(new_index_to_highlight) {
                //         None => Ok(None),
                //         //%%$%%does the next line actually work?? ie, unknown how clone would work w/ its db. If not, remove derive clone fr entity?
                //         //%%$%%might have to create a new instance of the entity, instead, with new2()?
                //         // Some(&e) => Some(e.to_owned()),
                //         Some(&e) => {
                //             // create a new instance of this entity, to avoid compiler errors
                //             let new_same_entity = match Entity::new2(Box::new(self), e.get_id()) {
                //                 Err(e) => return Err(e.to_string()),
                //                 Ok(entity) => entity,
                //             };
                //             Ok(Some(new_same_entity))
                //         },
                //     }
                // } else {
                //     if new_index_to_highlight + 1 < new_obj_list_size - 1 {
                //         match objects_to_display_in.get(new_index_to_highlight + 1) {
                //             //%%$%%%%%%%%%%should this containing method be moved back to util?--what db to use to create a new entity--get from the old db or??
                //             //%%$%%%%%%%%%%then, refactor this to call the just-above once, w/ parm for which entity to highlight?
                //             None => Ok(None),
                //             Some(&e) => Some(e),
                //         }
                //     } else if new_index_to_highlight >= 1 {
                //         match objects_to_display_in.get(new_index_to_highlight - 1) {
                //             None => None,
                //             Some(&e) => Some(e),
                //         }
                //     } else {
                //         None
                //     }
                // }
                //%%replace/del cmted part above w/ below?
                new_index_to_highlight = if new_index_to_highlight != previously_highlighted_index_in_obj_list_in {
                    new_index_to_highlight
                } else {
                    if new_index_to_highlight + 1 < new_obj_list_size - 1 {
                        new_index_to_highlight + 1
                            //%%$%%%%%%%%%%should this containing method be moved back to util?--what db to use to create a new entity--get from the old db or??
                            //%%$%%%%%%%%%%then, refactor this to call the just-above once, w/ parm for which entity to highlight?
                            // None => Ok(None),
                            // Some(&e) => Some(e),
                    } else if new_index_to_highlight >= 1 {
                        new_index_to_highlight - 1
                    } else {
                        return Ok(None)
                    }
                };
                // if new_index_to_highlight == -1 {
                //     Ok(None)
                // } else {
                    // %%why doesn't Rust know the element is an Entity, vs. <Unknown>? why can't just return
                    // objects_to_display_in.get(new_index_to_highlight)? Maybe rustc would do OK but the IDE doesn't? try changing at first 1 of the
                    // 3 below places back, and see if rustc gets it right? or am I mistaken?
                    match objects_to_display_in.get(new_index_to_highlight) {
                        None => Ok(None),
                        //%%$%%does the next line actually work?? ie, unknown how clone would work w/ its db. If not, remove derive clone fr entity?
                        //%%$%%might have to create a new instance of the entity, instead, with new2()?
                        // Some(&e) => Some(e.to_owned()),
                        Some(e) => {
                            // create a new instance of this entity, to avoid compiler errors
                            let new_same_entity = match Entity::new2(Box::new(self), e.get_id()) {
                                Err(e) => return Err(e.to_string()),
                                Ok(entity) => entity,
                            };
                            Ok(Some(new_same_entity))
                        },
                    }
                // }
            }
        } else {
            Ok(Some(previously_highlighted_entry_in))
        }
    }

}

impl Database for PostgreSQLDatabase {
    fn is_remote(&self) -> bool {
        false
    }

    ///  This means whether to act on *all* entities (true), or only non-archived (false, the more typical use).  Needs clarification?
    fn include_archived_entities(&self) -> bool {
        self.include_archived_entities
    }

    /// Like jdbc's default, if you don't call begin/rollback/commit, it will commit after every stmt,
    /// using the default behavior of jdbc; but if you call begin/rollback/commit, it will let you manage
    /// explicitly and will automatically turn autocommit on/off as needed to allow that.
    fn begin_trans(&self) -> Result<Transaction<Postgres>, sqlx::Error> {
        let tx = block_on(self.pool.begin())?;
        //%% see comments in fn connect() re this
        // connection.setAutoCommit(false);
        Ok(tx)
    }

    /// might not be needed when the transaction simply goes out of scope! ?
    fn rollback_trans(&self, tx: Transaction<Postgres>) -> Result<(), sqlx::Error> {
        block_on(tx.rollback())
        // so future work is auto- committed unless programmer explicitly opens another transaction
        //%% see comments in fn connect() re this
        // connection.setAutoCommit(true);
    }

    fn commit_trans(&self, tx: Transaction<Postgres>) -> Result<(), sqlx::Error> {
        block_on(tx.commit())
        // so future work is auto- committed unless programmer explicitly opens another transaction
        //%% see comments in fn connect() re this
        // connection.setAutoCommit(true);
    }

              // /** @param skip_check_for_bad_sql_in   Avoid using this parameter! See comment on PostgreSQLDatabase.db_action.
              //   */
              //   fn db_action(sql_in: String, caller_checks_row_count_etc: bool = false, skip_check_for_bad_sql_in: bool = false) -> i64 {
              //   PostgreSQLDatabase.db_action(sql_in, caller_checks_row_count_etc, connection, skip_check_for_bad_sql_in)
              // }


    /*
              /** Performs automatic database upgrades as required by evolving versions of OneModel.
                *
                * ******MAKE SURE*****:       ...that everything this does is also done in create_tables so that create_tables is a single reference
                * point for a developer to go read about the database structure, and for testing!  I.e., a newly-created OM instance shouldn't have to be upgraded,
                * because create_tables always provides the latest structure in a new system.  This method is just for updating older instances to what is in create_tables!
                */
                fn doDatabaseUpgradesIfNeeded() /* -> Unit%%*/ {
                let versionTableExists: bool = does_this_exist("select count(1) from pg_class where relname='om_db_version'");
                if ! versionTableExists) {
                  create_version_table()
                }
                let mut dbVersion: i32 = db_query_wrapper_for_one_row("select version from om_db_version", "Int")(0).get.asInstanceOf[Int];
                if dbVersion == 0) {
                  dbVersion = upgradeDbFrom0to1()
                }
                if dbVersion == 1) {
                  dbVersion = upgradeDbFrom1to2()
                }
                if dbVersion == 2) {
                  dbVersion = upgradeDbFrom2to3()
                }
                if dbVersion == 3) {
                  dbVersion = upgradeDbFrom3to4()
                }
                if dbVersion == 4) {
                  dbVersion = upgradeDbFrom4to5()
                }
                if dbVersion == 5) {
                  dbVersion = upgradeDbFrom5to6()
                }
                if dbVersion == 6) {
                  dbVersion = upgradeDbFrom6to7()
                }
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
                   (See also related comment above this doDatabaseUpgradesIfNeeded method.)  Better ideas?
                  */

                // This at least makes sure all the upgrades ran to completion.
                // Idea: Should it be instead more specific to what versions of the db are compatible with
                // this .jar, in case someone for example needs to restore old data but doesn't have an older .jar to go with it?
                require(dbVersion == PostgreSQLDatabase.SCHEMA_VERSION)
              }

                fn findAllEntityIdsByName(name_in: String, caseSensitive: bool = false) -> java.util.ArrayList[i64] {
                // idea: see if queries like this are using the expected index (run & ck the query plan). Tests around that, for benefit of future dbs? Or, just wait for
                // a performance issue then look at it?
                let sql = "select id from entity where " +;
                          (if !include_archived_entities) {
                            "(not archived) and "
                          } else {
                            ""
                          }) +
                          {
                            if caseSensitive) "name = '" + name_in + "'"
                            else "lower(name) = lower('" + name_in + "'" + ")"
                          }
                let rows = db_query(sql, "i64");
                let results = new java.util.ArrayList[i64]();
                for (row <- rows) {
                  results.add(row(0).get.asInstanceOf[i64])
                }
                results
              }

              // See comment in ImportExport.processUriContent method which uses it, about where the code should really go. Not sure if that idea includes this
              // method or not.
                fn findFIRSTClassIdByName(name_in: String, caseSensitive: bool = false) -> Option<i64> {
                // idea: see if queries like this are using the expected index (run & ck the query plan). Tests around that, for benefit of future dbs? Or, just wait for
                // a performance issue then look at it?
                let nameClause = {;
                  if caseSensitive) "name = '" + name_in + "'"
                  else "lower(name) = lower('" + name_in + "'" + ")"
                }
                let sql = "select id from class where " + nameClause + " order by id limit 1";
                let rows = db_query(sql, "i64");

                if rows.isEmpty) None
                else {
                  let mut results: List[i64] = Nil;
                  for (row <- rows) {
                    results = row(0).get.asInstanceOf[i64] :: results
                  }
                  if results.size > 1) throw new OmDatabaseException("Expected 1 row (wanted just the first one), found " + results.size + " rows.")
                  Some(results.head)
                }
              }


     */
              /// @param search_string_in is case-insensitive.
              /// @param stop_after_any_found is to prevent a serious performance problem when searching for the default entity at startup, if that default entity
              ///                          eventually links to 1000's of others.  Alternatives included specifying a different levels_remaining parameter in that
              ///                          case, or not following any RelationTo[Local|Remote]Entity links (which defeats the ability to organize the preferences in a hierarchy),
              ///                          or flagging certain ones to skip by marking them as a preference (not a link to follow in the preferences hierarchy), but
              ///                          those all seemed more complicated.
                fn find_contained_local_entity_ids<'a>(&'a self, results_in_out: &'a mut HashSet<i64>, from_entity_id_in: i64, search_string_in: &str,
                                              levels_remaining: i32/*%% = 20*/, stop_after_any_found: bool/*%% = true*/) -> Result<&mut HashSet<i64>, String> {
                // Idea for optimizing: don't re-traverse dup ones (eg, circular links or entities in same two places).  But that has other complexities: see
                // comments on ImportExport.exportItsChildrenToHtmlFiles for more info.  But since we are limiting the # of levels total, it might not matter anyway
                // (ie, probably the current code is not optimized but is simpler and good enough for now).

                if levels_remaining <= 0 || (stop_after_any_found && results_in_out.len() > 0) {
                  // do nothing: get out.
                } else {
                    let condition = if !self.include_archived_entities {
                                      "and not e.archived"
                                    } else { "" };
                  let sql = format!("select rte.entity_id_2, e.name from entity e, RelationToEntity rte \
                  where rte.entity_id={} and rte.entity_id_2=e.id {}", from_entity_id_in, condition);
                  let related_entity_id_rows = self.db_query(sql.as_str(), "i64,String")?;
                  // let lower_cased_regex_pattern = Pattern.compile(".*" + search_string_in.to_lowercase() + ".*");
                    let mut id: i64 = 0;
                    let mut name: String = "".to_string();
                  for row in related_entity_id_rows {
                      //%%$%%%%does this work?? or have to use a match to pull it out? see other places calling or inside db_query?
                      // or a let statement w/ same names + type, just before?
                      //idea: surely there is some better way than what I am doing here? See other places similarly.
                    // DataType::Bigint(id) = *row.get(0).unwrap();
                      id = match row.get(0).unwrap() {
                          DataType::Bigint(x) => *x,
                          _ => return Err(format!("How did we get here for {:?}?", row.get(0))),
                      };
                    // DataType::String(name) = *row.get(1).unwrap();
                      name = match row.get(1).unwrap() {
                          DataType::String(x) => x.clone(),
                          _ => return Err(format!("How did we get here for {:?}?", row.get(1))),
                      };

                    // NOTE: this line, similar lines just below, and the prompt inside EntityMenu.entitySearchSubmenu __should all match__.
                      if name.to_lowercase().contains(&search_string_in.to_lowercase()) {
                    // if lower_cased_regex_pattern.matcher(name.toLowerCase).find {
                      // have to do the name check here because we need to traverse all contained entities, so we need all those back from the sql, not just name matches.
                      results_in_out.insert(id);
                    }
                    self.find_contained_local_entity_ids(results_in_out, id, &search_string_in, levels_remaining - 1, stop_after_any_found);
                  }
                  if ! (stop_after_any_found && results_in_out.len() > 0) {
                      let condition = if !self.include_archived_entities {
                          " and not e.archived"
                      } else { "" };
                    let sql2 = format!("select eiag.entity_id, e.name from RelationToGroup rtg, EntitiesInAGroup eiag, entity e \
                    where rtg.entity_id={} and rtg.group_id=eiag.group_id and eiag.entity_id=e.id {}", from_entity_id_in, condition);
                    let entities_in_groups = self.db_query(sql2.as_str(), "i64,String")?;
                    for row in entities_in_groups {

                      // let id: i64 = row(0).get.asInstanceOf[i64];
                      // let name = row(1).get.asInstanceOf[String];
                        //idea: surely there is some better way than what I am doing here? See other places similarly.
                      //   DataType::Bigint(id) = *row.get(0).unwrap();
                      //   DataType::String(name) = *row.get(1).unwrap();
                        id = match row.get(0).unwrap() {
                            DataType::Bigint(x) => *x,
                            _ => return Err(format!("How did we get here for {:?}?", row.get(0))),
                        };
                        // DataType::String(name) = *row.get(1).unwrap();
                        name = match row.get(1).unwrap() {
                            DataType::String(x) => x.clone(),
                            _ => return Err(format!("How did we get here for {:?}?", row.get(1))),
                        };

                        // NOTE: this line, similar or related lines just above & below, and the prompt inside EntityMenu.entitySearchSubmenu __should all match__.
                        if name.to_lowercase().contains(&search_string_in.to_lowercase()) {
                      // if lower_cased_regex_pattern.matcher(name.toLowerCase).find {
                        // have to do the name check here because we need to traverse all contained entities, so we need all those back from the sql, not just name matches.
                        results_in_out.insert(id);
                      }
                      self.find_contained_local_entity_ids(results_in_out, id, search_string_in, levels_remaining - 1, stop_after_any_found);
                    }
                  }
                  // this part is doing a regex now:
                  if ! (stop_after_any_found && results_in_out.len() > 0) {
                      let if_archived = if !self.include_archived_entities {
                          " and (not e.archived)"
                      } else {
                          ""
                      };
                      // *NOTE*: this line about textValue, similar lines just above (doing "matcher ..."), and the prompt
                      // inside EntityMenu.entitySearchSubmenu __should all match__.
                      let sql3 = format!("select ta.id from textattribute ta, entity e where \
                                entity_id=e.id{} and entity_id={} and textValue ~* '{}'",
                                if_archived, from_entity_id_in, search_string_in);
                      //idea: just select a count, instead of requesting all the data back?
                    let textAttributes = self.db_query(sql3.as_str(), "i64")?;
                    if textAttributes.len() > 0 {
                      results_in_out.insert(from_entity_id_in);
                    }
                  }
                }
                Ok(results_in_out)
              }

              fn create_class_and_its_template_entity(&self, class_name_in: String) -> Result<(i64, i64), String> {
                self.create_class_and_its_template_entity2(class_name_in.clone(), format!("{}{}", class_name_in.clone(), Util::TEMPLATE_NAME_SUFFIX))
              }

    /*
                    fn deleteClassAndItsTemplateEntity(class_id_in: i64) {
                    self.begin_trans()
                    try {
                      let templateEntityId: i64 = getClassData(class_id_in)(1).get.asInstanceOf[i64];
                      let class_group_id = get_system_entitys_class_group_id;
                      if class_group_id.is_some()) {
                        removeEntityFromGroup(class_group_id.get, templateEntityId, caller_manages_transactions_in = true)
                      }
                      updateEntitysClass(templateEntityId, None, caller_manages_transactions_in = true)
                      deleteObjectById("class", class_id_in, caller_manages_transactions_in = true)
                      deleteObjectById(Util::ENTITY_TYPE, templateEntityId, caller_manages_transactions_in = true)
                    } catch {
                      case e: Exception => throw rollbackWithCatch(e)
                    }
                    commit_trans()
                  }
*/

                  /// Returns at most 1 row's info (id, relationTypeId, group_id, name), and a boolean indicating if more were available.
                  /// If 0 rows are found, returns (None, None, None, false), so this expects the caller
                  /// to know there is only one or deal with the None.
                  fn find_relation_to_and_group_on_entity(&self, entity_id_in: i64,
                                                      group_name_in: Option<String>/*%% = None*/)
                      -> Result<(Option<i64>, Option<i64>, Option<i64>, Option<String>, bool), String> {
                    let nameCondition = match group_name_in {
                      Some(gni) => {
                            let name = Self::escape_quotes_etc(gni);
                            format!("g.name='{}'", name)
                      },
                        __ => "true".to_string(),
                    };

                    // "limit 2", so we know and can return whether more were available:
                    let rows: Vec<Vec<DataType>> = self.db_query(format!("select rtg.id, rtg.rel_type_id, g.id, g.name from relationtogroup rtg, grupo g where rtg.group_id=g.id \
                                       and rtg.entity_id={} and {} order by rtg.id limit 2",
                                        entity_id_in, nameCondition).as_str(), "i64,i64,i64,String")?;
                    // there could be none found, or more than one, but:
                    if rows.is_empty() {
                        return Ok((None, None, None, None, false));
                    } else {
                      let row: Vec<DataType> = rows[0].clone();
                      let id: Option<i64> = {
                          match row[0] {
                              DataType::Bigint(x) => Some(x),
                              _ => return Err("should never happen 2".to_string()),
                          }
                      };
                      let relTypeId: Option<i64> = {
                          match row[1] {
                              DataType::Bigint(x) => Some(x),
                              _ => return Err("should never happen 3".to_string()),
                          }
                      };
                      let group_id: Option<i64> = {
                          match row[2] {
                              DataType::Bigint(x) => Some(x),
                              _ => return Err("should never happen 4".to_string()),
                          }
                      };
                      let name: Option<String> = {
                          match row[3].clone() {
                              DataType::String(x) => Some(x),
                              _ => return Err("should never happen 5".to_string()),
                          }
                      };
                      return Ok((id, relTypeId, group_id, name, rows.len() > 1));
                    }
                  }

    /*
                  ///
                  // @return the id of the new RTE
                    fn addHASRelationToLocalEntity(from_entity_id_in: i64, toEntityIdIn: i64, valid_on_date_in: Option<i64>, observation_date_in: i64,
                                                  sorting_index_in: Option<i64> = None):  -> RelationToLocalEntity {
                    let relationTypeId = find_relation_type(Database.THE_HAS_RELATION_TYPE_NAME, Some(1)).get(0);
                    let newRte = create_relation_to_local_entity(relationTypeId, from_entity_id_in, toEntityIdIn, valid_on_date_in, observation_date_in, sorting_index_in);
                    newRte
                  }
*/

                  /// Returns at most 1 id (and a the ideas was?: boolean indicating if more were available?).
                  /// If 0 rows are found, return(ed?) (None,false), so this expects the caller
                  /// to know there is only one or deal with the None.
                  fn find_relation_type(&self, type_name_in: String) -> Result<i64, String> {
                    let name = Self::escape_quotes_etc(type_name_in);
                    let rows = self.db_query(format!("select entity_id from entity e, relationtype rt where e.id=rt.entity_id and name='{}' order by id limit 2", name).as_str(), "i64")?;
                  let count = rows.len();
                  if count != 1 {
                      return Err(format!("Found {} rows instead of expected {}", count, 1)); //?: expected_rows.unwrap()));
                  }
                // there could be none found, or more than one, but not after above check.
                //     let mut final_result: Vec<i64> = Vec::new();
                    // for row in rows {
                      let id: i64 = match rows[0].get(0){
                          Some(DataType::Bigint(i)) => *i,
                          _ => return Err(format!("Found not 1 row with i64 but {:?} .", rows)),
                      };
                      // final_result.push(id);
                    // }
                    // Ok(final_result)
                      Ok(id)
                  }

    /*
                  // /** Used, for example, when test code is finished with its test data. Be careful. */
                  //   fn destroy_tables() {
                  //   PostgreSQLDatabase.destroy_tables(connection)
                  // }

                  /**
                   * Saves data for a quantity attribute for a Entity (i.e., "6 inches length").<br>
                   * parent_id_in is the key of the Entity for which the info is being saved.<br>
                   * inUnitId represents a Entity; indicates the unit for this quantity (i.e., liters or inches).<br>
                   * inNumber represents "how many" of the given unit.<br>
                   * attr_type_id_in represents the attribute type and also is a Entity (i.e., "volume" or "length")<br>
                   * valid_on_date_in represents the date on which this began to be true (seems it could match the observation date if needed,
                   * or guess when it was definitely true);
                   * NULL means unknown, 0 means it is asserted true for all time. inObservationDate is the date the fact was observed. <br>
                   * <br>
                   * We store the dates in
                   * postgresql (at least) as bigint which should be the same size as a java long, with the understanding that we are
                   * talking about java-style dates here; it is my understanding that such long's can also be negative to represent
                   * dates long before 1970, or positive for dates long after 1970. <br>
                   * <br>
                   * In the case of inNumber, note
                   * that the postgresql docs give some warnings about the precision of its real and "double precision" types. Given those
                   * warnings and the fact that I haven't investigated carefully (as of 9/2002) how the data will be saved and read
                   * between the java float type and the postgresql types, I am using "double precision" as the postgresql data type,
                   * as a guess to try to lose as
                   * little information as possible, and I'm making this note to you the reader, so that if you care about the exactness
                   * of the data you can do some research and let us know what you find.
                   * <p/>
                   * Re dates' meanings: see usage notes elsewhere in code (like inside create_tables).
                   */
                    fn createQuantityAttribute(parent_id_in: i64, attr_type_id_in: i64, unitIdIn: i64, numberIn: Float, valid_on_date_in: Option<i64>,
                                              inObservationDate: i64, caller_manages_transactions_in: bool = false, sorting_index_in: Option<i64> = None) -> /*id*/ i64 {
                    if !caller_manages_transactions_in { self.begin_trans() }
                    let mut id: i64 = 0L;
                    try {
                      id = get_new_key("QuantityAttributeKeySequence")
                      add_attribute_sorting_row(parent_id_in, Database.get_attribute_form_id(Util::QUANTITY_TYPE), id, sorting_index_in)
                      self.db_action(format!"insert into QuantityAttribute (id, entity_id, unit_id, quantity_number, attr_type_id, valid_on_date, observation_date) " +
                               "values (" + id + "," + parent_id_in + "," + unitIdIn + "," + numberIn + "," + attr_type_id_in + "," +
                               (if valid_on_date_in.isEmpty) "NULL" else valid_on_date_in.get) + "," + inObservationDate + ")").as_str(), false, false);
                    }
                    catch {
                      case e: Exception =>
                        if !caller_manages_transactions_in) rollback_trans()
                        throw e
                    }
                    if !caller_manages_transactions_in {self.commit_trans() }
                    id
                  }

                    fn escape_quotes_etc(s: String) -> String {
                    PostgreSQLDatabase.escape_quotes_etc(s)
                  }

                    fn check_for_bad_sql(s: String) {
                    PostgreSQLDatabase.check_for_bad_sql(s)
                  }

                    fn updateQuantityAttribute(id_in: i64, parent_id_in: i64, attr_type_id_in: i64, unitIdIn: i64, numberIn: Float, valid_on_date_in: Option<i64>,
                                              inObservationDate: i64) {
                    // NOTE: IF ADDING COLUMNS TO WHAT IS UPDATED, SIMILARLY UPDATE caller's update method! (else some fields don't get updated
                    // in memory when the db updates, and the behavior gets weird.
                    self.(format!("update QuantityAttribute set (unit_id, quantity_number, attr_type_id, valid_on_date, observation_date) = (" + unitIdIn + "," +
                             "" + numberIn + "," + attr_type_id_in + "," + (if valid_on_date_in.isEmpty) "NULL" else valid_on_date_in.get) + "," +
                             "" + inObservationDate + ") where id=" + id_in + " and  entity_id=" + parent_id_in).as_str, false, false);
                  }

                    fn updateTextAttribute(id_in: i64, parent_id_in: i64, attr_type_id_in: i64, text_in: String, valid_on_date_in: Option<i64>, observation_date_in: i64) {
                    let text: String = self.escape_quotes_etc(text_in);
                    // NOTE: IF ADDING COLUMNS TO WHAT IS UPDATED, SIMILARLY UPDATE caller's update method! (else some fields don't get updated
                    // in memory when the db updates, and the behavior gets weird.
                    db_action("update TextAttribute set (textValue, attr_type_id, valid_on_date, observation_date) = ('" + text + "'," + attr_type_id_in + "," +
                             "" + (if valid_on_date_in.isEmpty) "NULL" else valid_on_date_in.get) + "," + observation_date_in + ") where id=" + id_in + " and  " +
                             "entity_id=" + parent_id_in)
                  }

                    fn updateDateAttribute(id_in: i64, parent_id_in: i64, date_in: i64, attr_type_id_in: i64) {
                    // NOTE: IF ADDING COLUMNS TO WHAT IS UPDATED, SIMILARLY UPDATE caller's update method! (else some fields don't get updated
                    // in memory when the db updates, and the behavior gets weird.
                    self.db_action(format!("update DateAttribute set (date, attr_type_id) = (" + date_in + "," + attr_type_id_in + ") where id=" + id_in + " and  " +
                             "entity_id=" + parent_id_in).as_str(), false, false);
                  }
*/

                  fn update_boolean_attribute(&self, id_in: i64, parent_id_in: i64, attr_type_id_in: i64, boolean_in: bool,
                                              valid_on_date_in: Option<i64>, observation_date_in: i64) -> Result<(), String> {
                    // NOTE: IF ADDING COLUMNS TO WHAT IS UPDATED, SIMILARLY UPDATE caller's update method! (else some fields don't get updated
                    // in memory when the db updates, and the behavior gets weird.
                      let if_valid_on_date = match valid_on_date_in {
                          None => "NULL".to_string(),
                          Some(date) => date.to_string(),
                      };
                    self.db_action(format!("update BooleanAttribute set (booleanValue, attr_type_id, valid_on_date, observation_date) \
                        = ({},{},{},{}) where id={} and entity_id={}",
                        boolean_in, attr_type_id_in, if_valid_on_date, observation_date_in, id_in, parent_id_in).as_str(),
                                   false, false)?;
                      Ok(())
                  }

    /*
                  // We don't update the dates, path, size, hash because we set those based on the file's own timestamp, path current date,
                  // & contents when it is written. So the only
                  // point to having an update method might be the attribute type & description.
                  // AND THAT: The valid_on_date for a file attr shouldn't ever be None/NULL like with other attrs, because it is the file date in the filesystem before it was
                  // read into OM.
                    fn updateFileAttribute(id_in: i64, parent_id_in: i64, attr_type_id_in: i64, descriptionIn: String) {
                    // NOTE: IF ADDING COLUMNS TO WHAT IS UPDATED, SIMILARLY UPDATE caller's update method! (else some fields don't get updated
                    // in memory when the db updates, and the behavior gets weird.
                    self.db_action(format!("update FileAttribute set (description, attr_type_id) = ('" + descriptionIn + "'," + attr_type_id_in + ")" +
                             " where id=" + id_in + " and entity_id=" + parent_id_in).as_str(), false, false);
                  }

                  // first take on this: might have a use for it later.  It's tested, and didn't delete, but none known now. Remove?
                    fn updateFileAttribute(id_in: i64, parent_id_in: i64, attr_type_id_in: i64, descriptionIn: String, originalFileDateIn: i64, storedDateIn: i64,
                                          original_file_path_in: String, readableIn: bool, writableIn: bool, executableIn: bool, sizeIn: i64, md5hashIn: String) {
                    // NOTE: IF ADDING COLUMNS TO WHAT IS UPDATED, SIMILARLY UPDATE caller's update method! (else some fields don't get updated
                    // in memory when the db updates, and the behavior gets weird.
                    self.db_action(format!("update FileAttribute set " +
                             " (description, attr_type_id, original_file_date, stored_date, original_file_path, readable, writable, executable, size, md5hash) =" +
                             " ('" + descriptionIn + "'," + attr_type_id_in + "," + originalFileDateIn + "," + storedDateIn + ",'" + original_file_path_in + "'," +
                             " " + readableIn + "," + writableIn + "," + executableIn + "," +
                             " " + sizeIn + "," +
                             " '" + md5hashIn + "')" +
                             " where id=" + id_in + " and entity_id=" + parent_id_in).as_str(), false, false);
                  }

                    fn updateEntityOnlyName(id_in: i64, name_in: String) {
                    let name: String = self.escape_quotes_etc(name_in);
                    self.db_action(format!("update Entity set (name) = ROW('" + name + "') where id=" + id_in).as_str(), false, false);
                  }

                    fn updateEntityOnlyPublicStatus(id_in: i64, value: Option<bool>) {
                    self.db_action(format!("update Entity set (public) = ROW(" +
                             (if value.isEmpty) "NULL" else if value.get) "true" else "false") +
                             ") where id=" + id_in).as_str(), false, false);
                  }

                    fn updateEntityOnlyNewEntriesStickToTop(id_in: i64, newEntriesStickToTop: bool) {
                    self.db_action(format!("update Entity set (new_entries_stick_to_top) = ROW('" + newEntriesStickToTop + "') where id=" + id_in).as_str(), false, false);
                  }

                    fn updateClassAndTemplateEntityName(class_id_in: i64, name: String) -> i64 {
                    let mut entity_id: i64 = 0;
                    self.begin_trans()
                    try {
                      updateClassName(class_id_in, name)
                      entity_id = new EntityClass(this, class_id_in).getTemplateEntityId
                      updateEntityOnlyName(entity_id, name  + Database.TEMPLATE_NAME_SUFFIX)
                    }
                    catch {
                      case e: Exception => throw rollbackWithCatch(e)
                    }
                    commit_trans()
                    entity_id
                  }

                    fn updateClassName(id_in: i64, name_in: String) {
                    let name: String = self.escape_quotes_etc(name_in);
                    self.db_action(format!("update class set (name) = ROW('" + name + "') where id=" + id_in).as_str(), false, false);
                  }

                    fn updateEntitysClass(entity_id: i64, class_id: Option<i64>, caller_manages_transactions_in: bool = false) {
                    if !caller_manages_transactions_in) self.begin_trans()
                    self.db_action(format!("update Entity set (class_id) = ROW(" +
                             (if class_id.isEmpty) "NULL" else class_id.get) +
                             ") where id=" + entity_id).as_str(), false, false);
                    let group_ids = db_query("select group_id from EntitiesInAGroup where entity_id=" + entity_id, "i64");
                    for (row <- group_ids) {
                      let group_id = row(0).get.asInstanceOf[i64];
                      let mixed_classes_allowed: bool = are_mixed_classes_allowed(group_id);
                      if (!mixed_classes_allowed) && has_mixed_classes(group_id)) {
                        throw rollbackWithCatch(new OmDatabaseException(Database.MIXED_CLASSES_EXCEPTION))
                      }
                    }
                    if !caller_manages_transactions_in) commit_trans()
                  }

                    fn updateRelationType(id_in: i64, name_in: String, name_in_reverseDirectionIn: String, directionalityIn: String) {
                    require(name_in != null)
                    require(name_in.length > 0)
                    require(name_in_reverseDirectionIn != null)
                    require(name_in_reverseDirectionIn.length > 0)
                    require(directionalityIn != null)
                    require(directionalityIn.length > 0)
                    let name_in_reverseDirection: String = self.escape_quotes_etc(name_in_reverseDirectionIn);
                    let name: String = self.escape_quotes_etc(name_in);
                    let directionality: String = self.escape_quotes_etc(directionalityIn);
                    self.begin_trans()
                    try {
                      self.db_action(format!("update Entity set (name) = ROW('" + name + "') where id=" + id_in).as_str(), false, false);
                      self.db_action(format!("update RelationType set (name_in_reverse_direction, directionality) = ROW('" + name_in_reverseDirection + "', " +
                               "'" + directionality + "') where entity_id=" + id_in).as_str(), false, false);
                    } catch {
                      case e: Exception => throw rollbackWithCatch(e)
                    }
                    commit_trans()
                  }


     */
                  /// Re dates' meanings: see usage notes elsewhere in code (like inside create_tables).
                  fn create_text_attribute(&self, parent_id_in: i64, attr_type_id_in: i64, text_in: &str, valid_on_date_in: Option<i64> /*%%= None*/,
                                          observation_date_in: i64 /*%%= System.currentTimeMillis()*/, caller_manages_transactions_in: bool/*%% = false*/,
                                          sorting_index_in: Option<i64> /*%%= None*/) -> /*id*/ Result<i64, String> {
                    let text: String = Self::escape_quotes_etc(text_in.to_string());
                    let id: i64 = self.get_new_key("TextAttributeKeySequence")?;
                      //%%$%%%%%%%FIX NEXT LINE AFTERI SEE HOW OTHERS DO!
                    // if !caller_manages_transactions_in { self.begin_trans()?; }
                    let add_result = self.add_attribute_sorting_row(parent_id_in, self.get_attribute_form_id(Util::TEXT_TYPE).unwrap(), id, sorting_index_in);
                    match add_result {
                        Err(s) => {
                            //%%$%%%%%%%FIX NEXT LINE AFTERI SEE HOW OTHERS DO!
                            // if !caller_manages_transactions_in { self.rollback_trans()?; }
                            return Err(s.to_string());
                        }
                        _ => {}
                    }
                  let result = self.db_action(format!("insert into TextAttribute (id, entity_id, textvalue, \
                  attr_type_id, valid_on_date, observation_date) values ({id},{parent_id_in},'{text}',{attr_type_id_in},{},{})",
                           match valid_on_date_in {
                               None => "NULL".to_string(),
                               Some(vod) => vod.to_string(),
                           },
                           observation_date_in).as_str(), false, false);
                    match result {
                        Err(s) => {
                            //%%$%%%%%%%FIX NEXT LINE AFTERI SEE HOW OTHERS DO!
                             // if !caller_manages_transactions_in { self.rollback_trans()?; }
                            return Err(s);
                        }
                        _ => {}
                    };
                      //%%$%%%%%%%FIX NEXT LINE AFTERI SEE HOW OTHERS DO!
                // if !caller_manages_transactions_in { self.commit_trans()?; }
                Ok(id)
                  }
/*
                    fn createDateAttribute(parent_id_in: i64, attr_type_id_in: i64, date_in: i64, sorting_index_in: Option<i64> = None) -> /*id*/ i64 {
                    let id: i64 = get_new_key("DateAttributeKeySequence");
                    self.begin_trans()
                    try {
                      add_attribute_sorting_row(parent_id_in, Database.get_attribute_form_id(Util::DATE_TYPE), id, sorting_index_in)
                      self.db_action(format!("insert into DateAttribute (id, entity_id, attr_type_id, date) " +
                               "values (" + id + "," + parent_id_in + ",'" + attr_type_id_in + "'," + date_in + ")").as_str(), false, false);
                    }
                    catch {
                      case e: Exception => throw rollbackWithCatch(e)
                    }
                    commit_trans()
                    id
                  }

 */

                  fn create_boolean_attribute(&self, parent_id_in: i64, attr_type_id_in: i64, boolean_in: bool, valid_on_date_in: Option<i64>,
                                              observation_date_in: i64, sorting_index_in: Option<i64> /*%%= None*/) -> /*id*/ Result<i64, String> {
                    let id: i64 = self.get_new_key("BooleanAttributeKeySequence")?;
                      let tx: Transaction<Postgres> = match self.begin_trans() {
                          Err(e) => return Err(e.to_string()),
                          Ok(t) => t,
                      };
                    // try {
                      self.add_attribute_sorting_row(parent_id_in, self.get_attribute_form_id(Util::BOOLEAN_TYPE).unwrap(), id, sorting_index_in)?;
                      let vod = match valid_on_date_in {
                          None => "NULL".to_string(),
                          Some(date) => date.to_string(),
                      };
                      self.db_action(format!("insert into BooleanAttribute (id, entity_id, booleanvalue, attr_type_id, valid_on_date, observation_date) \
                               values ({},{},'{}',{},{},{})", id, parent_id_in, boolean_in, attr_type_id_in,
                                             vod, observation_date_in).as_str(), false, false)?;
                    // }
                    // catch {
                        //see cmts at "%%rollback problem" for ideas on this?  Or, is ok? was:
                      // case e: Exception => throw rollbackWithCatch(e)
                    // }
                      match self.commit_trans(tx) {
                          Err(e) => return Err(e.to_string()),
                          _ => {},
                      }
                    Ok(id)
                  }
/*

                    fn createFileAttribute(parent_id_in: i64, attr_type_id_in: i64, descriptionIn: String, originalFileDateIn: i64, storedDateIn: i64,
                                          original_file_path_in: String, readableIn: bool, writableIn: bool, executableIn: bool, sizeIn: i64,
                                          md5hashIn: String, inputStreamIn: java.io.FileInputStream, sorting_index_in: Option<i64> = None) -> /*id*/ i64 {
                    let description: String = self.escape_quotes_etc(descriptionIn);
                    // (Next 2 for completeness but there shouldn't ever be a problem if other code is correct.)
                    let original_file_path: String = self.escape_quotes_etc(original_file_path_in);
                    // Escaping the md5hash string shouldn't ever matter, but security is more important than the md5hash:
                    let md5hash: String = self.escape_quotes_etc(md5hashIn);
                    let mut obj: LargeObject = null;
                    let mut id: i64 = 0;
                    try {
                      id = get_new_key("FileAttributeKeySequence")
                      self.begin_trans()
                      add_attribute_sorting_row(parent_id_in, Database.get_attribute_form_id(Util::FILE_TYPE), id, sorting_index_in)
                      self.db_action(format!("insert into FileAttribute (id, entity_id, attr_type_id, description, original_file_date, stored_date, original_file_path, readable, writable," +
                               " executable, size, md5hash)" +
                               " values (" + id + "," + parent_id_in + "," + attr_type_id_in + ",'" + description + "'," + originalFileDateIn + "," + storedDateIn + "," +
                               " '" + original_file_path + "', " + readableIn + ", " + writableIn + ", " + executableIn + ", " + sizeIn + ",'" + md5hash + "')").as_str(), false, false);
                      // from the example at:   http://jdbc.postgresql.org/documentation/80/binary-data.html & info
                      // at http://jdbc.postgresql.org/documentation/publicapi/org/postgresql/largeobject/LargeObjectManager.html & its links.
                      let lobjManager: LargeObjectManager = connection.asInstanceOf[org.postgresql.PGConnection].getLargeObjectAPI;
                      let oid: i64 = lobjManager.createLO();
                      obj = lobjManager.open(oid, LargeObjectManager.WRITE)
                      let buffer = new Array[Byte](2048);
                      let mut numBytesRead = 0;
                      let mut total: i64 = 0;
                      @tailrec
                      //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) w/in this method!
                      fn saveFileToDb() {
                        numBytesRead = inputStreamIn.read(buffer)
                        // (intentional style violation, for readability):
                        //noinspection ScalaUselessExpression
                        if numBytesRead == -1) Unit
                        else {
                          // (just once by a subclass is enough to mess w/ the md5sum for testing:)
                          if total == 0) damageBuffer(buffer)

                          obj.write(buffer, 0, numBytesRead)
                          total += numBytesRead
                          saveFileToDb()
                        }
                      }
                      saveFileToDb()
                      if total != sizeIn) {
                        throw new OmDatabaseException("Transferred " + total + " bytes instead of " + sizeIn + "??")
                      }
                      self.db_action(format!("INSERT INTO FileAttributeContent (file_attribute_id, contents_oid) VALUES (" + id + "," + oid + ")").as_str(), false, false);

                      let (success, errMsgOption) = verifyFileAttributeContentIntegrity(id);
                      if !success) {
                        throw new OmFileTransferException("Failure to successfully upload file content: " + errMsgOption.getOrElse("(verification provided no error message? " +
                                                                                                                                   "how?)"))
                      }
                      commit_trans()
                      id
                    } catch {
                      case e: Exception => throw rollbackWithCatch(e)
                    } finally {
                      if obj != null)
                        try {
                          obj.close()
                        } catch {
                          case e: Exception =>
                          // not sure why this fails sometimes, if it's a bad thing or not, but for now not going to be stuck on it.
                          // idea: look at the source code.
                        }
                    }
                  }

                  /// Re dates' meanings: see usage notes elsewhere in code (like inside create_tables). */
              fn create_relation_to_local_entity(&self, relation_type_id_in: i64, entity_id1_in: i64,
                                                 entity_id2_in: i64, valid_on_date_in: Option<i64>,
                                                 observation_date_in: i64, sorting_index_in: Option<i64>/*%% = None*/,
                                                 caller_manages_transactions_in: bool/*%% = false*/)
                  -> Result<RelationToLocalEntity, String> {
                let rte_id: i64 = self.get_new_key("RelationToEntityKeySequence")?;
                  //%%$%%%%%%%FIX NEXT LINE AFTERI SEE HOW OTHERS DO!
                // if !caller_manages_transactions_in { self.begin_trans()?; }
                let result: Result<i64, String> = self.add_attribute_sorting_row(entity_id1_in, self.get_attribute_form_id(Util::RELATION_TO_LOCAL_ENTITY_TYPE).unwrap(), rte_id, sorting_index_in);
                if let Err(e) = result {
                    //%%$%%%%%%%FIX NEXT LINE AFTERI SEE HOW OTHERS DO!
                    // if !caller_manages_transactions_in { self.rollback_trans()?; }
                    return Err(e);
                }
                let valid_on_date_sql_str = match valid_on_date_in {
                    Some(date) => date.to_string(),
                    None => "NULL".to_string(),
                };
                let result = self.db_action(format!("INSERT INTO RelationToEntity (id, rel_type_id, entity_id, entity_id_2, valid_on_date, observation_date) \
                       VALUES ({},{},{},{}, {},{})", rte_id, relation_type_id_in, entity_id1_in, entity_id2_in,
                       valid_on_date_sql_str, observation_date_in).as_str(), false, false);
                if let Err(e) = result {
                    //%%$%%%%%%%FIX NEXT LINE AFTERI SEE HOW OTHERS DO!
                    // if !caller_manages_transactions_in { self.rollback_trans()?; }
                    return Err(e);
                }
                  //%%$%%%%%%%FIX NEXT LINE AFTERI SEE HOW OTHERS DO!
                // if !caller_manages_transactions_in { self.commit_trans()?; }

                Ok(RelationToLocalEntity{}) //%%$%%really: self, rte_id, relation_type_id_in, entity_id1_in, entity_id2_in})
              }

              /** Re dates' meanings: see usage notes elsewhere in code (like inside create_tables). */
                fn create_relation_to_remote_entity(&self, relation_type_id_in: i64, entity_id1_in: i64, entity_id2_in: i64, valid_on_date_in: Option<i64>, observation_date_in: i64,
                                               remote_instance_id_in: String, sorting_index_in: Option<i64>/*%% = None*/,
                                               caller_manages_transactions_in: bool/*%% = false*/) -> Result<RelationToRemoteEntity, String> {
                  //%%$%%%%%%%FIX NEXT LINE AFTERI SEE HOW OTHERS DO!
                // if !caller_manages_transactions_in { self.begin_trans()?; }
                let rte_id: i64 = self.get_new_key("RelationToRemoteEntityKeySequence")?;
                  // not creating anything in a remote DB, but a local record of a local relation to a remote entity.
                  let result = self.add_attribute_sorting_row(entity_id1_in,
                                            self.get_attribute_form_id(Util::RELATION_TO_REMOTE_ENTITY_TYPE).unwrap(),
                                            rte_id, sorting_index_in);
                    if let Err(e) = result {
                        //%%$%%%%%%%FIX NEXT LINE AFTERI SEE HOW OTHERS DO!
                        // if !caller_manages_transactions_in { self.rollback_trans()?; }
                        return Err(e);
                    }

                  let valid_on_date_sql_str = match valid_on_date_in {
                        Some(date) => date.to_string(),
                        None => "NULL".to_string(),
                    };
                  let result = self.db_action(format!("INSERT INTO RelationToRemoteEntity (id, rel_type_id, entity_id, \
                  entity_id_2, valid_on_date, observation_date, remote_instance_id) VALUES ({},{},{},{},{},{},'{}')",
                      rte_id, relation_type_id_in, entity_id1_in, entity_id2_in,
                      valid_on_date_sql_str, observation_date_in, remote_instance_id_in).as_str(), false, false);
                    if let Err(e) = result {
                        //%%$%%%%%%%FIX NEXT LINE AFTERI SEE HOW OTHERS DO!
                        // if !caller_manages_transactions_in { self.rollback_trans()?; }
                        return Err(e);
                    }
                  //%%$%%%%%%%FIX NEXT LINE AFTERI SEE HOW OTHERS DO!
                // if !caller_manages_transactions_in { self.commit_trans()?; }
                Ok(RelationToRemoteEntity{}) //%%$%%really: self, rte_id, relation_type_id_in, entity_id1_in, remote_instance_id_in, entity_id2_in
              }

    /*
              /** Re dates' meanings: see usage notes elsewhere in code (like inside create_tables). */
                fn updateRelationToLocalEntity(oldRelationTypeIdIn: i64, entity_id1_in: i64, entity_id2_in: i64,
                                         newRelationTypeIdIn: i64, valid_on_date_in: Option<i64>, observation_date_in: i64) {
                // NOTE: IF ADDING COLUMNS TO WHAT IS UPDATED, SIMILARLY UPDATE caller's update method! (else some fields don't get updated
                // in memory when the db updates, and the behavior gets weird.
                self.db_action(format!("UPDATE RelationToEntity SET (rel_type_id, valid_on_date, observation_date)" +
                         " = (" + newRelationTypeIdIn + "," + (if valid_on_date_in.isEmpty) "NULL" else valid_on_date_in.get) + "," + observation_date_in + ")" +
                         " where rel_type_id=" + oldRelationTypeIdIn + " and entity_id=" + entity_id1_in + " and entity_id_2=" + entity_id2_in).as_str(), false, false);
              }

              /** Re dates' meanings: see usage notes elsewhere in code (like inside create_tables). */
                fn updateRelationToRemoteEntity(oldRelationTypeIdIn: i64, entity_id1_in: i64, remote_instance_id_in: String, entity_id2_in: i64,
                                         newRelationTypeIdIn: i64, valid_on_date_in: Option<i64>, observation_date_in: i64) {
                // NOTE: IF ADDING COLUMNS TO WHAT IS UPDATED, SIMILARLY UPDATE caller's update method! (else some fields don't get updated
                // in memory when the db updates, and the behavior gets weird.
                self.db_action(format!("UPDATE RelationToRemoteEntity SET (rel_type_id, valid_on_date, observation_date)" +
                         " = (" + newRelationTypeIdIn + "," + (if valid_on_date_in.isEmpty) "NULL" else valid_on_date_in.get) + "," + observation_date_in + ")" +
                         " where rel_type_id=" + oldRelationTypeIdIn + " and entity_id=" + entity_id1_in + " and remote_instance_id='" + remote_instance_id_in
                         + "' and entity_id_2=" + entity_id2_in).as_str(), false, false);
              }

              /**
               * Takes an RTLE and unlinks it from one local entity, and links it under another instead.
               * @param sorting_index_in Used because it seems handy (as done in calls to other move methods) to keep it in case one moves many entries: they stay in order.
               * @return the new RelationToLocalEntity
               */
                fn moveRelationToLocalEntityToLocalEntity(rtleIdIn: i64, toContainingEntityIdIn: i64, sorting_index_in: i64) -> RelationToLocalEntity {
                self.begin_trans();
                try {
                  let rteData: Array[Option[Any]] = getAllRelationToLocalEntityDataById(rtleIdIn);
                  let oldRteRelType: i64 = rteData(2).get.asInstanceOf[i64];
                  let oldRteEntity1: i64 = rteData(3).get.asInstanceOf[i64];
                  let oldRteEntity2: i64 = rteData(4).get.asInstanceOf[i64];
                  let valid_on_date: Option<i64> = rteData(5).asInstanceOf[Option<i64>];
                  let observed_date: i64 = rteData(6).get.asInstanceOf[i64];
                  deleteRelationToLocalEntity(oldRteRelType, oldRteEntity1, oldRteEntity2)
                  let newRTE: RelationToLocalEntity = create_relation_to_local_entity(oldRteRelType, toContainingEntityIdIn, oldRteEntity2, valid_on_date, observed_date,;
                                                                                  Some(sorting_index_in), caller_manages_transactions_in = true)
                  //Something like the next line might have been more efficient than the above code to run, but not to write, given that it adds a complexity about updating
                  //the attributesorting table, which might be more tricky in future when something is added to prevent those from being orphaned. The above avoids that or
                  //centralizes the question to one place in the code.
                  //db_action("UPDATE RelationToEntity SET (entity_id) = ROW(" + newContainingEntityIdIn + ")" + " where id=" + relationToLocalEntityIdIn)

                  self.commit_trans();
                  newRTE
                } catch {
                  case e: Exception => throw rollbackWithCatch(e)
                }
              }

              /**
               * See comments on & in method moveRelationToLocalEntityToLocalEntity.  Only this one takes an RTRE (stored locally), and instead of linking it inside one local
               * entity, links it inside another local entity.
               */
                fn moveRelationToRemoteEntityToLocalEntity(remote_instance_id_in: String, relationToRemoteEntityIdIn: i64, toContainingEntityIdIn: i64,
                                                          sorting_index_in: i64) -> RelationToRemoteEntity {
                self.begin_trans()
                try {
                  let rteData: Array[Option[Any]] = getAllRelationToRemoteEntityDataById(relationToRemoteEntityIdIn);
                  let oldRteRelType: i64 = rteData(2).get.asInstanceOf[i64];
                  let oldRteEntity1: i64 = rteData(3).get.asInstanceOf[i64];
                  let oldRteEntity2: i64 = rteData(4).get.asInstanceOf[i64];
                  let valid_on_date: Option<i64> = rteData(5).asInstanceOf[Option<i64>];
                  let observed_date: i64 = rteData(6).get.asInstanceOf[i64];
                  deleteRelationToRemoteEntity(oldRteRelType, oldRteEntity1, remote_instance_id_in, oldRteEntity2)
                  let newRTE: RelationToRemoteEntity = create_relation_to_remote_entity(oldRteRelType, toContainingEntityIdIn, oldRteEntity2, valid_on_date, observed_date,;
                                                                                  remote_instance_id_in, Some(sorting_index_in), caller_manages_transactions_in = true)
                  commit_trans()
                  newRTE
                } catch {
                  case e: Exception => throw rollbackWithCatch(e)
                }
              }

*/
              fn create_group(&self, name_in: &str, allow_mixed_classes_in_group_in: bool /*%%= false*/) -> Result<i64, String> {
                let name: String = Self::escape_quotes_etc(name_in.to_string());
                let group_id: i64 = self.get_new_key("RelationToGroupKeySequence")?;
                  let allow_mixed = if allow_mixed_classes_in_group_in {
                      "TRUE"
                  } else {
                      "FALSE"
                  };
                self.db_action(format!("INSERT INTO grupo (id, name, insertion_date, allow_mixed_classes) \
                         VALUES ({}, '{}', {}, {})",
                         group_id, name, Utc::now().timestamp_millis(), allow_mixed).as_str(), false, false)?;
                Ok(group_id)
              }

              /// I.e., make it so the entity has a group in it, which can contain entities.
              // Re dates' meanings: see usage notes elsewhere in code (like inside create_tables).
              fn create_group_and_relation_to_group(&self, entity_id_in: i64, relation_type_id_in: i64, new_group_name_in: &str,
                                                    allow_mixed_classes_in_group_in: bool /*%%= false*/,
                                                valid_on_date_in: Option<i64>, observation_date_in: i64,
                                                sorting_index_in: Option<i64>, caller_manages_transactions_in: bool /*%%= false*/)
                  -> Result<(i64, i64), String> {
                  //%%$%%%%%%%FIX NEXT LINE AFTERI SEE HOW OTHERS DO!
                // if !caller_manages_transactions_in { self.begin_trans() }
                let group_id: i64 = self.create_group(new_group_name_in, allow_mixed_classes_in_group_in)?;
                let (rtg_id, _) = self.create_relation_to_group(entity_id_in, relation_type_id_in, group_id, valid_on_date_in, observation_date_in, sorting_index_in, caller_manages_transactions_in)?;
                  //%%$%%%%%%%FIX NEXT LINE AFTERI SEE HOW OTHERS DO!
                // if !caller_manages_transactions_in {self.commit_trans() }
                Ok((group_id, rtg_id))
              }

              /// I.e., make it so the entity has a relation to a new entity in it.
              /// Re dates' meanings: see usage notes elsewhere in code (like inside create_tables).
              fn create_entity_and_relation_to_local_entity(&self, entity_id_in: i64, relation_type_id_in: i64, new_entity_name_in: &str, is_public_in: Option<bool>,
                                                       valid_on_date_in: Option<i64>, observation_date_in: i64, caller_manages_transactions_in: bool/*%% = false*/)
                  -> Result<(i64, i64), String> {
                let name: String = Self::escape_quotes_etc(new_entity_name_in.to_string());
                  //%%$%%%%%%%FIX NEXT LINE AFTERI SEE HOW OTHERS DO!
                // if !caller_manages_transactions_in { self.begin_trans() }
                let new_entity_id: i64 = self.create_entity(name.as_str(), None, is_public_in)?;
                let newRte: RelationToLocalEntity = self.create_relation_to_local_entity(relation_type_id_in, entity_id_in, new_entity_id, valid_on_date_in, observation_date_in, None,
                                                                                caller_manages_transactions_in)?;
                  //%%$%%%%%%%FIX NEXT LINE AFTERI SEE HOW OTHERS DO!
                // if !caller_manages_transactions_in {self.commit_trans() }
                  //%%$%%%%%%%FIX NEXT LINE AFTERI SEE HOW OTHERS DO!
                Ok((new_entity_id, 0)) //really: , newRte.get_id()))
              }

              /// I.e., make it so the entity has a group in it, which can contain entities.
              /// Re dates' meanings: see usage notes elsewhere in code (like inside create_tables).
              /// @return a tuple containing the id and new sorting_index: (id, sorting_index)
              fn create_relation_to_group(&self, entity_id_in: i64, relation_type_id_in: i64, group_id_in: i64, valid_on_date_in: Option<i64>, observation_date_in: i64,
                                        sorting_index_in: Option<i64> /*%%= None*/, caller_manages_transactions_in: bool /*%%= false*/) -> Result<(i64, i64), String> {
                  //%%$%%%%%%%FIX NEXT LINE AFTERI SEE HOW OTHERS DO!
                // if !caller_manages_transactions_in { self.begin_trans() }
                let id: i64 = self.get_new_key("RelationToGroupKeySequence2")?;
                let sorting_index = {
                    let sorting_index: i64 = self.add_attribute_sorting_row(entity_id_in, self.get_attribute_form_id(Util::RELATION_TO_GROUP_TYPE).unwrap(), id, sorting_index_in)?;
                  let valid_date = match valid_on_date_in {
                      None => "NULL".to_string(),
                      Some(d) => d.to_string(),
                  };
                   self.db_action(format!("INSERT INTO RelationToGroup (id, entity_id, rel_type_id, group_id, valid_on_date, observation_date) \
                             VALUES ({},{},{},{},{},{})", id, entity_id_in, relation_type_id_in, group_id_in, valid_date, observation_date_in).as_str(),
                                  false, false)?;
                    sorting_index
                };
                  //%%$%%%%%%%FIX NEXT LINE AFTERI SEE HOW OTHERS DO!
                // if !caller_manages_transactions_in {self.commit_trans() }
                Ok((id, sorting_index))
              }

    /*
                fn updateGroup(group_id_in: i64, name_in: String, allow_mixed_classes_in_group_in: bool = false, newEntriesStickToTopIn: bool = false) {
                let name: String = self.escape_quotes_etc(name_in);
                self.db_action(format!("UPDATE grupo SET (name, allow_mixed_classes, new_entries_stick_to_top)" +
                         " = ('" + name + "', " + (if allow_mixed_classes_in_group_in) "TRUE" else "FALSE") + ", " + (if newEntriesStickToTopIn) "TRUE" else "FALSE") +
                         ") where id=" + group_id_in).as_str(), false, false);
              }

              /** Re dates' meanings: see usage notes elsewhere in code (like inside create_tables).
                */
                fn updateRelationToGroup(entity_id_in: i64, oldRelationTypeIdIn: i64, newRelationTypeIdIn: i64, oldGroupIdIn: i64, newGroupIdIn: i64,
                                        valid_on_date_in: Option<i64>, observation_date_in: i64) {
                // NOTE: IF ADDING COLUMNS TO WHAT IS UPDATED, SIMILARLY UPDATE caller's update method! (else some fields don't get updated
                // in memory when the db updates, and the behavior gets weird.
                self.db_action(format!("UPDATE RelationToGroup SET (rel_type_id, group_id, valid_on_date, observation_date)" +
                         " = (" + newRelationTypeIdIn + ", " + newGroupIdIn + ", " +
                         (if valid_on_date_in.isEmpty) "NULL" else valid_on_date_in.get) + "," + observation_date_in + ")" +
                         " where entity_id=" + entity_id_in + " and rel_type_id=" + oldRelationTypeIdIn + " and group_id=" + oldGroupIdIn).as_str(), false, false);
              }

              /**
               * @param sorting_index_in Used because it seems handy (as done in calls to other move methods) to keep it in case one moves many entries: they stay in order.
               * @return the new RelationToGroup's id.
               */
                fn moveRelationToGroup(relationToGroupIdIn: i64, newContainingEntityIdIn: i64, sorting_index_in: i64) -> i64 {
                self.begin_trans()
                try {
                  let rtgData: Array[Option[Any]] = getAllRelationToGroupDataById(relationToGroupIdIn);
                  let oldRtgEntityId: i64 = rtgData(2).get.asInstanceOf[i64];
                  let oldRtgRelType: i64 = rtgData(3).get.asInstanceOf[i64];
                  let oldRtgGroupId: i64 = rtgData(4).get.asInstanceOf[i64];
                  let valid_on_date: Option<i64> = rtgData(5).asInstanceOf[Option<i64>];
                  let observed_date: i64 = rtgData(6).get.asInstanceOf[i64];
                  deleteRelationToGroup(oldRtgEntityId, oldRtgRelType, oldRtgGroupId)
                  let (newRtg_id: i64,_) = create_relation_to_group(newContainingEntityIdIn, oldRtgRelType, oldRtgGroupId, valid_on_date, observed_date, Some(sorting_index_in),;
                                                             caller_manages_transactions_in = true)

                  // (see comment at similar commented line in moveRelationToLocalEntityToLocalEntity)
                  //db_action("UPDATE RelationToGroup SET (entity_id) = ROW(" + newContainingEntityIdIn + ")" + " where id=" + relationToGroupIdIn)

                  self.commit_trans();
                  newRtg_id
                } catch {
                  case e: Exception => throw rollbackWithCatch(e)
                }
              }

              /** Trying it out with the entity's previous sorting_index (or whatever is passed in) in case it's more convenient, say, when brainstorming a
                * list then grouping them afterward, to keep them in the same order.  Might be better though just to put them all at the beginning or end; can see....
                */
                fn moveLocalEntityFromGroupToGroup(fromGroupIdIn: i64, toGroupIdIn: i64, moveEntityIdIn: i64, sorting_index_in: i64) {
                self.begin_trans()
                add_entity_to_group(toGroupIdIn, moveEntityIdIn, Some(sorting_index_in), caller_manages_transactions_in = true)
                removeEntityFromGroup(fromGroupIdIn, moveEntityIdIn, caller_manages_transactions_in = true)
                if isEntityInGroup(toGroupIdIn, moveEntityIdIn) && !isEntityInGroup(fromGroupIdIn, moveEntityIdIn)) {
                  commit_trans()
                } else {
                  throw rollbackWithCatch(new OmDatabaseException("Entity didn't get moved properly.  Retry: if predictably reproducible, it should be diagnosed."))
                }
              }

              /** (See comments on moveEntityFromGroupToGroup.)
                */
                fn moveEntityFromGroupToLocalEntity(fromGroupIdIn: i64, toEntityIdIn: i64, moveEntityIdIn: i64, sorting_index_in: i64) {
                self.begin_trans()
                addHASRelationToLocalEntity(toEntityIdIn, moveEntityIdIn, None, System.currentTimeMillis(), Some(sorting_index_in))
                removeEntityFromGroup(fromGroupIdIn, moveEntityIdIn, caller_manages_transactions_in = true)
                commit_trans()
              }

              /** (See comments on moveEntityFromGroupToGroup.)
                */
                fn moveLocalEntityFromLocalEntityToGroup(removingRtleIn: RelationToLocalEntity, targetGroupIdIn: i64, sorting_index_in: i64) {
                self.begin_trans()
                add_entity_to_group(targetGroupIdIn, removingRtleIn.getRelatedId2, Some(sorting_index_in), caller_manages_transactions_in = true)
                deleteRelationToLocalEntity(removingRtleIn.get_attr_type_id(), removingRtleIn.getRelatedId1, removingRtleIn.getRelatedId2)
                commit_trans()
              }

     */

              // SEE ALSO METHOD find_unused_attribute_sorting_index **AND DO MAINTENANCE IN BOTH PLACES**
              // idea: this needs a test, and/or combining with findIdWhichIsNotKeyOfAnyEntity.
              // **ABOUT THE SORTINGINDEX:  SEE the related comment on method add_attribute_sorting_row.
              fn find_unused_group_sorting_index(&self, group_id_in: i64, starting_with_in: Option<i64>/*%% = None*/) -> Result<i64, String> {
                  //better idea?  This should be fast because we start in remote regions and return as soon as an unused id is found, probably
                  //only one iteration, ever.  (See similar comments elsewhere.)
                  // findUnusedSortingIndex_helper(group_id_in, starting_with_in.getOrElse(max_id_value - 1), 0)
                  let g_id = group_id_in;
                  let mut working_index = starting_with_in.unwrap_or(self.max_id_value() - 1);
                  let mut counter = 0;

                loop {
                  //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) w/in this method!
                  if self.is_group_entry_sorting_index_in_use(g_id, working_index)? {
                    if working_index == self.max_id_value() {
                      // means we did a full loop across all possible ids!?  Doubtful. Probably would turn into a performance problem long before. It's a bug.
                      return Err(Util::UNUSED_GROUP_ERR1.to_string());
                    }
                    // idea: see comment at similar location in findIdWhichIsNotKeyOfAnyEntity
                    if counter > 10_000 {
                          return Err(Util::UNUSED_GROUP_ERR2.to_string());
                      }
                      working_index = working_index - 1;
                      counter = counter + 1;
                      continue;
                  } else {
                      return Ok(working_index)
                  }
                }
              }

              // SEE COMMENTS IN find_unused_group_sorting_index **AND DO MAINTENANCE IN BOTH PLACES
              // **ABOUT THE SORTINGINDEX:  SEE the related comment on method add_attribute_sorting_row.
              fn find_unused_attribute_sorting_index(&self, entity_id_in: i64, starting_with_in: Option<i64> /*%%= None*/) -> Result<i64, String> {
                  let mut working_index = starting_with_in.unwrap_or(self.max_id_value() - 1);
                  let mut counter = 0;
                  loop {
                      if self.is_attribute_sorting_index_in_use(entity_id_in, working_index)? {
                          if working_index == self.max_id_value() {
                              return Err(Util::UNUSED_GROUP_ERR1.to_string());
                          }
                          if counter > 10_000 {
                              return Err(Util::UNUSED_GROUP_ERR2.to_string());
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
              fn add_entity_to_group(&self, group_id_in: i64, contained_entity_id_in: i64, sorting_index_in: Option<i64> /*%%= None*/, caller_manages_transactions_in: bool /*%%= false*/)
              -> Result<(), String> {
                // IF THIS CHANGES ALSO DO MAINTENANCE IN SIMILAR METHOD add_attribute_sorting_row
                  //%%$%%%%%%%FIX NEXT LINE AFTERI SEE HOW OTHERS DO!
                // if !caller_manages_transactions_in { self.begin_trans()?; }

                // start from the beginning index, if it's the 1st record (otherwise later sorting/renumbering gets messed up if we start w/ the last #):
                let sorting_index: i64 = {
                  let index = match sorting_index_in {
                      Some(x) => x,
                      // start with an increment off the min or max, so that later there is room to sort something before or after it, manually:
                      None if self.get_group_size(group_id_in, 3)? == 0 => {
                          self.min_id_value() + 99999
                      }
                      _ => self.max_id_value() - 99999,
                  };
                  let is_in_use: bool = self.is_group_entry_sorting_index_in_use(group_id_in, index)?;
                  if is_in_use {
                      let find_unused_result: i64 = self.find_unused_group_sorting_index(group_id_in, None)?;
                      // let unused_index: i64 = match find_unused_result {
                      //     Ok(i) => i,
                      //     Err(s) => {
                      //         %%$%%%%%%%FIX NEXT LINE AFTERI SEE HOW OTHERS DO!
                      //         if !caller_manages_transactions_in { self.rollback_trans()?; }
                              // return Err(s.to_string());
                          // },
                      // };
                      find_unused_result
                  } else {
                      index
                  }
                };

                  let x=1;let y=2;if x==1 {println!("gh1")};let z=3;//this is a test %% to see what formatter does w/ cmt and MAINLY if compiler allows 2 statements on a line (to help decide about how t o debug later MAYBE if changes could move lines around?)

                let result = self.db_action(format!("insert into EntitiesInAGroup (group_id, entity_id, sorting_index) values ({},{},{})",
                          group_id_in, contained_entity_id_in, sorting_index).as_str(), false, false);
                  if let Err(s) = result {
                      //%%$%%%%%%%FIX NEXT LINE AFTERI SEE HOW OTHERS DO!
                      // if !caller_manages_transactions_in { self.rollback_trans()?; }
                      return Err(s)
                  }
                // idea: do this check sooner in this method?:
                let mixed_classes_allowed: bool = self.are_mixed_classes_allowed(group_id_in)?;
                if !mixed_classes_allowed && self.has_mixed_classes(group_id_in)? {
                    //%%$%%%%%%%FIX NEXT LINE AFTERI SEE HOW OTHERS DO!
                  // if !caller_manages_transactions_in { self.rollback_trans()?; }
                  return Err(Util::MIXED_CLASSES_EXCEPTION.to_string());
                }
                  //%%$%%%%%%%FIX NEXT LINE AFTERI SEE HOW OTHERS DO!
                // if !caller_manages_transactions_in { self.commit_trans()?; }
                  Ok(())
              }

            fn create_entity(&self, name_in: &str, class_id_in: Option<i64> /*%%= None*/,
                             is_public_in: Option<bool> /*%%= None*/) -> /*id*/ Result<i64, String> {
                let name: String = Self::escape_quotes_etc(name_in.to_string());
                if name.is_empty() {
                    return Err("Name must have a value.".to_string());
                }
                let id: i64 = self.get_new_key("EntityKeySequence")?;
                let maybe_class_id: &str = if class_id_in.is_some() { ", class_id" } else { "" };
                let maybe_is_public: &str = match is_public_in {
                    None => "NULL",
                    Some(b) => if b { "true" } else { "false" }
                };
                let maybe_class_id_val = match class_id_in {
                    Some(id) => format!(",{}", id.to_string()),
                    _ => "".to_string(),
                };
                let sql: String = format!("INSERT INTO Entity (id, insertion_date, name, public{}) VALUES ({},{},'{}',{}{})",
                    maybe_class_id, id, Utc::now().timestamp_millis(), name, maybe_is_public, maybe_class_id_val);
                self.db_action(sql.as_str(), false, false)?;
                Ok(id)
              }

              fn create_relation_type(&self, name_in: &str, name_in_reverseDirectionIn: &str, directionalityIn: &str) -> /*id*/ Result<i64, String> {
                let name_in_reverseDirection: String = Self::escape_quotes_etc(name_in_reverseDirectionIn.to_string());
                let name: String = Self::escape_quotes_etc(name_in.to_string());
                let directionality: String = Self::escape_quotes_etc(directionalityIn.to_string());
                if name.len() == 0 {
                    return Err("Name must have a value.".to_string());
                }
                  let tx: Transaction<Postgres> = match self.begin_trans() {
                      Err(e) => return Err(e.to_string()),
                      Ok(t) => t,
                  };

                  let mut result: Result<i64, String>;
                  let mut id: i64 = 0;
                //see comment at loop in create_tables()
                loop {
                  id = match self.get_new_key("EntityKeySequence") {
                      Err(s) => {
                          result = Err(s.to_string());
                          break;
                      },
                      Ok(i) => i,
                  };
                  result = self.db_action(format!("INSERT INTO Entity (id, insertion_date, name) VALUES ({},{},'{}')",
                                                  id, Utc::now().timestamp_millis(), name).as_str(), false, false);
                    if result.is_err() {break;}
                  result = self.db_action(format!("INSERT INTO RelationType (entity_id, name_in_reverse_direction, directionality) VALUES ({},'{}','{}')",
                                                  id, name_in_reverseDirection, directionality).as_str(), false, false);
                    if result.is_err() {break;}
                    match self.commit_trans(tx) {
                        Err(e) => return Err(e.to_string()),
                        _ => {},
                    }

                    // see comment at top of loop
                    break;
                  }
                  //%%$%%%%%debug/verify all parts of this?:
                  if result.is_err() {
                      //%%$%%%%%%%FIX NEXT LINE AFTERI SEE HOW OTHERS DO!
                      // let _rollback_result = self.rollback_trans();
                  // if let Err(e1) = result {
                      //%%rollback problem:  see if i can figure out a way to get rollback to work like the old def rollbackWithCatch
                      // in ~/proj/om/core-scala/src/main/scala/org/onemodel/core/model/PostgreSQLDatabase.scala .
                      // maybe see:  https://docs.rs/sqlx/latest/sqlx/enum.Error.html to see if IJ can work now w/ its to_string() or fmt/Display that
                      // it says are there but IJ says as of 2023-03-08 is not there? Or something?
                      // Does a newer IJ or sqlx help at all?

                      // let rollback_result: Result<(), sqlx::Error> = self.rollback_trans();
                      // if rollback_result.is_err() {
                      //     let rollback_err: sqlx::Error = rollback_result.expect_err("Failure can never happen.");
                      //     let combined_error = format!("Error '{}' occurred while rolling back transaction due to original error '{}'.",
                      //                                  rollback_err, e1);
                      // }
                      // if let Err(e2) = rollback_result {
                      //     Err(e2) => {
                      //         let combined_error = format!("Error '{}' occurred while rolling back transaction due to original error '{}'.",
                      //                                      e2, e1);
                      //         result = Err(combined_error);
                      //     },
                      //     _ => {},
                      // }
                      result
                  } else {
                      Ok(id)
                  }
              }
    /*

                    create_attribute_sorting_deletion_trigger(t: Throwable) -> Throwable {
                    let mut rollbackException: Option[Throwable] = None;
                    try {
                      rollback_trans()
                    } catch {
                      case e: Exception =>
                        rollbackException = Some(e)
                    }
                    if rollbackException.isEmpty) t
                    else {
                      rollbackException.get.addSuppressed(t)
                      let exc = new OmDatabaseException("See the chained messages for ALL: the cause of rollback failure, AND for the original failure(s).",;
                                                        rollbackException.get)
                      exc
                    }
                  }

     */
                  fn delete_entity(&self, id_in: i64, caller_manages_transactions_in: bool /*%%= false*/) -> Result<(), String> {
                    // idea: (also on task list i think but) we should not delete entities until dealing with their use as attrtypeids etc!
                    // (or does the DB's integrity constraints do that for us?)
                      //%%$%%%%%%%FIX NEXT LINE AFTERI SEE HOW OTHERS DO!
                    // if !caller_manages_transactions_in { self.begin_trans()?; }
                    self.delete_objects("EntitiesInAGroup", format!("where entity_id={}", id_in).as_str(), -1, true)?;
                      self.delete_objects(Util::ENTITY_TYPE, format!("where id={}", id_in).as_str(), 1, true)?;
                      self.delete_objects("AttributeSorting", format!("where entity_id={}", id_in).as_str(), -1, true)?;
                      //%%$%%%%%%%FIX NEXT LINE AFTERI SEE HOW OTHERS DO!
                    // if !caller_manages_transactions_in {self.commit_trans()?; }
                    Ok(())
                  }

    /*
                    fn archiveEntity(id_in: i64, caller_manages_transactions_in: bool = false) /* -> Unit%%*/ {
                    archiveObjects(Util::ENTITY_TYPE, "where id=" + id_in, 1, caller_manages_transactions_in)
                  }

                    fn unarchiveEntity(id_in: i64, caller_manages_transactions_in: bool = false) /* -> Unit%%*/ {
                    archiveObjects(Util::ENTITY_TYPE, "where id=" + id_in, 1, caller_manages_transactions_in, unarchive = true)
                  }

                    fn deleteQuantityAttribute(id_in: i64) /* -> %%Unit*/ {
                        deleteObjectById(Util::QUANTITY_TYPE, id_in);
                        }

                    fn deleteTextAttribute(id_in: i64) /*%% -> Unit*/ {
                        deleteObjectById(Util::TEXT_TYPE, id_in);
                    }

                    fn deleteDateAttribute(id_in: i64) /* -> %%Unit*/ {
                    deleteObjectById(Util::DATE_TYPE, id_in);
                    }

                    fn deleteBooleanAttribute(id_in: i64) /*%% -> Unit*/ {
                    deleteObjectById(Util::BOOLEAN_TYPE, id_in);
                    }

                    fn deleteFileAttribute(id_in: i64) /*%% ->  Unit*/ {
                    deleteObjectById(Util::FILE_TYPE, id_in);
                    }

                    fn deleteRelationToLocalEntity(relTypeIdIn: i64, entity_id1_in: i64, entity_id2_in: i64) {
                    delete_objects(Util::RELATION_TO_LOCAL_ENTITY_TYPE, "where rel_type_id=" + relTypeIdIn + " and entity_id=" + entity_id1_in + " and entity_id_2=" + entity_id2_in)
                  }

                    fn deleteRelationToRemoteEntity(relTypeIdIn: i64, entity_id1_in: i64, remote_instance_id_in: String, entity_id2_in: i64) {
                    delete_objects(Util::RELATION_TO_REMOTE_ENTITY_TYPE, "where rel_type_id=" + relTypeIdIn + " and entity_id=" + entity_id1_in + " and remote_instance_id='" +
                                                                remote_instance_id_in + "' and entity_id_2=" + entity_id2_in)
                  }

                    fn deleteRelationToGroup(entity_id_in: i64, relTypeIdIn: i64, group_id_in: i64) {
                    delete_objects(Util::RELATION_TO_GROUP_TYPE, "where entity_id=" + entity_id_in + " and rel_type_id=" + relTypeIdIn + " and group_id=" + group_id_in)
                  }

                    fn deleteGroupAndRelationsToIt(id_in: i64) {
                    self.begin_trans();
                    try {
                      let entityCount: i64 = get_group_size(id_in);
                      delete_objects("EntitiesInAGroup", "where group_id=" + id_in, entityCount, caller_manages_transactions_in = true)
                      let numGroups = get_relation_to_group_countByGroup(id_in);
                      delete_objects(Util::RELATION_TO_GROUP_TYPE, "where group_id=" + id_in, numGroups, caller_manages_transactions_in = true)
                      delete_objects("grupo", "where id=" + id_in, 1, caller_manages_transactions_in = true)
                    }
                    catch {
                      case e: Exception => throw rollbackWithCatch(e)
                    }
                    commit_trans()
                  }

                    fn removeEntityFromGroup(group_id_in: i64, contained_entity_id_in: i64, caller_manages_transactions_in: bool = false) {
                    delete_objects("EntitiesInAGroup", "where group_id=" + group_id_in + " and entity_id=" + contained_entity_id_in,
                                  caller_manages_transactions_in = caller_manages_transactions_in)
                  }

                  /** I hope you have a backup. */
                    fn deleteGroupRelationsToItAndItsEntries(group_id_in: i64) {
                    self.begin_trans()
                    try {
                      let entityCount = get_group_size(group_id_in);

                      fn deleteRelationToGroupAndALL_recursively(group_id_in: i64) -> (i64, i64) {
                        let entity_ids: List[Array[Option[Any]]] = db_query("select entity_id from entitiesinagroup where group_id=" + group_id_in, "i64");
                        let deletions1 = delete_objects("entitiesinagroup", "where group_id=" + group_id_in, entityCount, caller_manages_transactions_in = true);
                        // Have to delete these 2nd because of a constraint on EntitiesInAGroup:
                        // idea: is there a temp table somewhere that these could go into instead, for efficiency?
                        // idea: batch these, would be much better performance.
                        // idea: BUT: what is the length limit: should we do it it sets of N to not exceed sql command size limit?
                        // idea: (also on task list i think but) we should not delete entities until dealing with their use as attrtypeids etc!
                        for (id <- entity_ids) {
                          delete_objects(Util::ENTITY_TYPE, "where id=" + id(0).get.asInstanceOf[i64], 1, caller_manages_transactions_in = true)
                        }

                        let deletions2 = 0;
                        //and finally:
                        // (passing -1 for rows expected, because there either could be some, or none if the group is not contained in any entity.)
                        delete_objects(Util::RELATION_TO_GROUP_TYPE, "where group_id=" + group_id_in, -1, caller_manages_transactions_in = true)
                        delete_objects("grupo", "where id=" + group_id_in, 1, caller_manages_transactions_in = true)
                        (deletions1, deletions2)
                      }
                      let (deletions1, deletions2) = deleteRelationToGroupAndALL_recursively(group_id_in);
                      require(deletions1 + deletions2 == entityCount)
                    }
                    catch {
                      case e: Exception => throw rollbackWithCatch(e)
                    }
                    commit_trans()
                  }

                    fn deleteRelationType(id_in: i64) {
                    // One possibility is that this should ALWAYS fail because it is done by deleting the entity, which cascades.
                    // but that's more confusing to the programmer using the database layer's api calls, because they
                    // have to know to delete an Entity instead of a RelationType. So we just do the desired thing here
                    // instead, and the delete cascades.
                    // Maybe those tables should be separated so this is its own thing? for performance/clarity?
                    // like *attribute and relation don't have a parent 'attribute' table?  But see comments
                    // in create_tables where this one is created.
                    delete_objects(Util::ENTITY_TYPE, "where id=" + id_in)
                  }
        */
    //%%$%%%

                  /// Creates the preference if it doesn't already exist.
                  fn set_user_preference_boolean(&self, name_in: &str, value_in: bool) -> Result<(), String> {
                    let preferences_container_id: i64 = self.get_preferences_container_id()?;
                    let result = self.get_user_preference2(preferences_container_id, name_in, Util::PREF_TYPE_BOOLEAN)?;
                    if result.len() > 0 {

                        // let preferenceInfo: Option[(i64, Boolean)] = result.asInstanceOf[Option[(i64,Boolean)]];
                        //idea: surely there is some better way than what I am doing here? See other places similarly.
                        // let DataType::Bigint(preference_attribute_id) = result[0];
                        let preference_attribute_id = match result[0] {
                            DataType::Bigint(x) => x,
                            _ => return Err(format!("How did we get here for {:?}?", result[0])),
                        };

                        let mut attribute = BooleanAttribute::new2(Box::new(self), preference_attribute_id)?;
                        // Now we have found a boolean attribute which already existed, and just need to
                        // update its boolean value. The other values we read from the db inside the first call
                        // to something like "get_parent_id()", and just write them back with the new boolean value,
                        // to conveniently reuse existing methods.
                        self.update_boolean_attribute(attribute.get_id(), attribute.get_parent_id()?,
                                                      attribute.get_attr_type_id()?, value_in,
                                                      attribute.get_valid_on_date()?, attribute.get_observation_date()?)
                    } else {
                      let type_id_of_the_has_relation = self.find_relation_type(Util::THE_HAS_RELATION_TYPE_NAME.to_string()/*??:, Some(1)).get(0)*/)?;
                      let preference_entity_id: i64 = self.create_entity_and_relation_to_local_entity(preferences_container_id,
                                                                                                      type_id_of_the_has_relation,
                                                                                                      name_in, None,
                                                                                          Some(Utc::now().timestamp_millis()),
                                                                                                      Utc::now().timestamp_millis(),
                                                                                                    false)?.0;
                      // (For about the attr_type_id value (2nd parm), see comment about that field, in method get_user_preference_boolean2 below.)
                      self.create_boolean_attribute(preference_entity_id, preference_entity_id, value_in,
                                                    Some(Utc::now().timestamp_millis()),
                                                    Utc::now().timestamp_millis(),
                                                    None)?;
                      Ok(())
                    }
                  }

    fn get_user_preference_boolean(
        &self,
        preference_name_in: &str,
        default_value_in: Option<bool>, /*%%= None*/
    ) -> Option<bool> {
        return None;
        //%%cont
        //     let pref = get_user_preference2(get_preferences_container_id, preference_name_in, Database.PREF_TYPE_BOOLEAN);
        //     if pref.isEmpty) {
        //       default_value_in
        //     } else {
        //       Some(pref.get.asInstanceOf[(i64,Boolean)]._2)
        //     }
    }

    /*
              /** Creates the preference if it doesn't already exist.  */
                fn setUserPreference_EntityId(name_in: String, entity_id_in: i64) /* -> Unit%%*/ {
                let preferences_container_id: i64 = get_preferences_container_id;
                let result = get_user_preference2(preferences_container_id, name_in, Database.PREF_TYPE_ENTITY_ID);
                let preferenceInfo: Option[(i64, i64, i64)] = result.asInstanceOf[Option[(i64,i64,i64)]];
                if preferenceInfo.is_some()) {
                  let relationTypeId: i64 = preferenceInfo.get._1;
                  let entity_id1: i64 = preferenceInfo.get._2;
                  let entity_id2: i64 = preferenceInfo.get._3;
                  // didn't bother to put these 2 calls in a transaction because this is likely to be so rarely used and easily fixed by user if it fails (from default
                  // entity setting on any entity menu)
                  deleteRelationToLocalEntity(relationTypeId, entity_id1, entity_id2)
                  // (Using entity_id1 instead of (the likely identical) preferences_container_id, in case this RTE was originally found down among some
                  // nested preferences (organized for user convenience) under here, in order to keep that organization.)
                  create_relation_to_local_entity(relationTypeId, entity_id1, entity_id_in, Some(System.currentTimeMillis()), System.currentTimeMillis())
                } else {
                  let type_id_of_the_has_relation = find_relation_type(Database.THE_HAS_RELATION_TYPE_NAME, Some(1)).get(0);
                  let preference_entity_id: i64 = create_entity_and_relation_to_local_entity(preferences_container_id, type_id_of_the_has_relation, name_in, None,;
                                                                                      Some(System.currentTimeMillis()), System.currentTimeMillis())._1
                  create_relation_to_local_entity(type_id_of_the_has_relation, preference_entity_id, entity_id_in, Some(System.currentTimeMillis()), System.currentTimeMillis())
                }
              }
    */

    //%%$%%
    /*
                fn getUserPreference_EntityId(preference_name_in: String, default_value_in: Option<i64> = None) -> Option<i64> {
                let pref = get_user_preference2(get_preferences_container_id, preference_name_in, Database.PREF_TYPE_ENTITY_ID);
                if pref.isEmpty) {
                  default_value_in
                } else {
                  Some(pref.get.asInstanceOf[(i64,i64,i64)]._3)
                }
              }

*/

              /// This should never return None, except when method createExpectedData is called for the first time in a given database.
              fn get_preferences_container_id(&self) -> Result<i64, String> {
                let related_entity_id = self.get_relation_to_local_entity_by_name(self.get_system_entity_id()?, Util::USER_PREFERENCES)?;
                match related_entity_id {
                    None => return Err("This should never happen: method createExpectedData should be run at startup to create this part of the data.".to_string()),
                    Some(id) => Ok(id),
                }
              }
    //%%$%%
    /*
            fn getEntityCount() ->  i64 {
            extract_row_count_from_count_query("SELECT count(1) from Entity " +
                                                                   (if !include_archived_entities) {
                                                                     "where (not archived)"
                                                                   } else {
                                                                     ""
                                                                   })
                                                                  );
                                                                  }

            fn getClassCount(templateEntityIdIn: Option<i64> = None) -> i64 {
            let whereClause = if templateEntityIdIn.is_some()) " where defining_entity_id=" + templateEntityIdIn.get else "";
            extract_row_count_from_count_query("SELECT count(1) from class" + whereClause)
          }

            fn getGroupEntrySortingIndex(group_id_in: i64, entity_id_in: i64) -> i64 {
            let row = db_query_wrapper_for_one_row("select sorting_index from EntitiesInAGroup where group_id=" + group_id_in + " and entity_id=" + entity_id_in, "i64");
            row(0).get.asInstanceOf[i64]
          }

            fn getEntityAttributeSortingIndex(entity_id_in: i64, attribute_form_id_in: i64, attribute_id_in: i64) -> i64 {
            let row = db_query_wrapper_for_one_row("select sorting_index from AttributeSorting where entity_id=" + entity_id_in + " and attribute_form_id=" +;
                                              attribute_form_id_in + " and attribute_id=" + attribute_id_in, "i64")
            row(0).get.asInstanceOf[i64]
          }

            fn getHighestSortingIndexForGroup(group_id_in: i64) -> i64 {
            let rows: List[Array[Option[Any]]] = db_query("select max(sorting_index) from EntitiesInAGroup where group_id=" + group_id_in, "i64");
            require(rows.size == 1)
            rows.head(0).get.asInstanceOf[i64]
          }

            fn renumberSortingIndexes(entity_idOrGroupIdIn: i64, caller_manages_transactions_in: bool = false, isEntityAttrsNotGroupEntries: bool = true) {
            //This used to be called "renumberAttributeSortingIndexes" before it was merged with "renumberGroupSortingIndexes" (very similar).
            let numberOfEntries: i64 = {;
              if isEntityAttrsNotGroupEntries) get_attribute_count(entity_idOrGroupIdIn, include_archived_entitiesIn = true)
              else get_group_size(entity_idOrGroupIdIn)
            }
            if numberOfEntries != 0) {
              // (like a number line so + 1, then add 1 more (so + 2) in case we use up some room on the line due to "attributeSortingIndexInUse" (below))
              let numberOfSegments = numberOfEntries + 2;
              // ( * 2 on next line, because the min_id_value is negative so there is a larger range to split up, but
              // doing so without exceeding the value of a i64 during the calculation.)
              let increment: i64 = (max_id_value.asInstanceOf[Float] / numberOfSegments * 2).asInstanceOf[i64];
              // (start with an increment so that later there is room to sort something prior to it, manually)
              let mut next: i64 = self.min_id_value() + increment;
              let mut previous: i64 = self.min_id_value();
                      //%%$%%%%%%%FIX NEXT LINE AFTERI SEE HOW OTHERS DO!
              // if !caller_manages_transactions_in { self.begin_trans() }
              try {
                let data: List[Array[Option[Any]]] = {;
                  if isEntityAttrsNotGroupEntries) getEntityAttributeSortingData(entity_idOrGroupIdIn)
                  else getGroupEntriesData(entity_idOrGroupIdIn)
                }
                if data.size != numberOfEntries) {
                  // "Idea:: BAD SMELL! The UI should do all UI communication, no?"
                  // (SEE ALSO comments and code at other places with the part on previous line in quotes).
                  eprintln!()
                  eprintln!()
                  eprintln!()
                  eprintln!("--------------------------------------")
                  eprintln!("Unexpected state: data.size (" + data.size +  ") != numberOfEntries (" + numberOfEntries +  "), when they should be equal. ")
                  if data.size > numberOfEntries) {
                    eprintln!("Possibly, the database trigger \"attribute_sorting_cleanup\" (created in method create_attribute_sorting_deletion_trigger) is" +
                    " not always cleaning up when it should or something. ")
                  }
                  eprintln!("If there is a consistent way to reproduce this from scratch (with attributes of a *new* entity), or other information" +
                                     " to diagnose/improve the situation, please advise.  The program will attempt to continue anyway but a bug around sorting" +
                                     " or placement in this set of entries might result.")
                  eprintln!("--------------------------------------")
                }
                for (entry <- data) {
                  if isEntityAttrsNotGroupEntries) {
                    while (is_attribute_sorting_index_in_use(entity_idOrGroupIdIn, next)) {
                      // Renumbering might choose already-used numbers, because it always uses the same algorithm.  This causes a constraint violation (unique index)
                      // , so
                      // get around that with a (hopefully quick & simple) increment to get the next unused one.  If they're all used...that's a surprise.
                      // Idea: also fix this bug in the case where it's near the end & the last #s are used: wrap around? when give err after too many loops: count?
                      next += 1
                    }
                  } else {
                    while (is_group_entry_sorting_index_in_use(entity_idOrGroupIdIn, next)) {
                      next += 1
                    }
                  }
                  // (make sure a bug didn't cause wraparound w/in the set of possible i64 values)
                  require(previous < next && next < self.max_id_value(), "Requirement failed for values previous, next, and max_id_value(): " + previous + ", " + next + ", " +
                                                                self.max_id_value())
                  if isEntityAttrsNotGroupEntries) {
                    let form_id: i64 = entry(0).get.asInstanceOf[Int];
                    let attributeId: i64 = entry(1).get.asInstanceOf[i64];
                    updateAttributeSortingIndex(entity_idOrGroupIdIn, form_id, attributeId, next)
                  } else {
                    let id: i64 = entry(0).get.asInstanceOf[i64];
                    updateSortingIndexInAGroup(entity_idOrGroupIdIn, id, next)
                  }
                  previous = next
                  next += increment
                }
              }
              catch {
                case e: Exception =>
                      //%%$%%%%%%%FIX NEXT LINE AFTERI SEE HOW OTHERS DO!
                  // if !caller_manages_transactions_in) rollback_trans()
                  throw e
              }

              // require: just to confirm that the generally expected behavior happened, not a requirement other than that:
              // (didn't happen in case of newly added entries w/ default values....
              // idea: could investigate further...does it matter or imply anything for adding entries to *brand*-newly created groups? Is it related
              // to the fact that when doing that, the 2nd entry goes above, not below the 1st, and to move it down you have to choose the down 1 option
              // *twice* for some reason (sometimes??)? And to the fact that deleting an entry selects the one above, not below, for next highlighting?)
              // (See also a comment somewhere else 4 poss. issue that refers, related, to this method name.)
              //require((maxIDValue - next) < (increment * 2))

                      //%%$%%%%%%%FIX NEXT LINE AFTERI SEE HOW OTHERS DO!
              // if !caller_manages_transactions_in {self.commit_trans() }
            }
          }

            fn classLimit(limitByClass: bool, class_id_in: Option<i64>) -> String {
            if limitByClass) {
              if class_id_in.is_some()) {
                " and e.class_id=" + class_id_in.get + " "
              } else {
                " and e.class_id is NULL "
              }
            } else ""
          }

          /** Excludes those entities that are really relationtypes, attribute types, or quantity units.
            *
            * The parameter limitByClass decides whether any limiting is done at all: if true, the query is
            * limited to entities having the class specified by inClassId (even if that is None).
            *
            * The parameter templateEntity *further* limits, if limitByClass is true, by omitting the templateEntity from the results (ex., to help avoid
            * counting that one when deciding whether it is OK to delete the class).
            * */
            fn getEntitiesOnlyCount(limitByClass: bool = false, class_id_in: Option<i64> = None,
                                   templateEntity: Option<i64> = None) -> i64 {
            extract_row_count_from_count_query("SELECT count(1) from Entity e where " +
                                          (if !include_archived_entities) {
                                            "(not archived) and "
                                          } else {
                                            ""
                                          }) +
                                          "true " +
                                          classLimit(limitByClass, class_id_in) +
                                          (if limitByClass && templateEntity.is_some()) " and id != " + templateEntity.get else "") +
                                          " and id in " +
                                          "(select id from entity " + limit_to_entities_only(Self::ENTITY_ONLY_SELECT_PART) +
                                          ")")
          }

            fn getRelationTypeCount -> i64 {
            extract_row_count_from_count_query("select count(1) from RelationType")
            }
*/

          fn get_attribute_count(&self, entity_id_in: i64, include_archived_entitiesIn: bool /*%%= false*/) -> Result<i64, String> {
              let total = self.get_quantity_attribute_count(entity_id_in)? +
                  self.get_text_attribute_count(entity_id_in)? +
                  self.get_date_attribute_count(entity_id_in)? +
                  self.get_boolean_attribute_count(entity_id_in)? +
                  self.get_file_attribute_count(entity_id_in)? +
                  self.get_relation_to_local_entity_count(entity_id_in, include_archived_entitiesIn)? +
                  self.get_relation_to_remote_entity_count(entity_id_in)? +
                  self.get_relation_to_group_count(entity_id_in)?;
              Ok(total)
          }

    fn get_relation_to_local_entity_count(&self, entity_id_in: i64, include_archived_entities: bool /*= true*/) -> Result<i64, String> {
        let appended = if !include_archived_entities && !include_archived_entities {
            " and (not eContained.archived)"
        } else {
            ""
        };
        let sql = format!("select count(1) from entity eContaining, RelationToEntity rte, entity eContained \
            where eContaining.id=rte.entity_id and rte.entity_id={} and rte.entity_id_2=eContained.id{}", entity_id_in, appended);

        self.extract_row_count_from_count_query(sql.as_str())
    }

    fn get_relation_to_remote_entity_count(&self, entity_id_in: i64) ->  Result<i64, String> {
        let sql = format!("select count(1) from entity eContaining, RelationToRemoteEntity rtre \
            where eContaining.id=rtre.entity_id and rtre.entity_id={}", entity_id_in);
        self.extract_row_count_from_count_query(sql.as_str())
    }

    /** if 1st parm is None, gets all. */
    fn get_relation_to_group_count(&self, entity_id_in: i64) ->  Result<i64, String> {
        self.extract_row_count_from_count_query(format!("select count(1) from relationtogroup where entity_id={}", entity_id_in).as_str())
    }
    /*

          fn getAttributeSortingRowsCount(entity_id_in: Option<i64> = None) -> Result<i64, String> {
            let sql = "select count(1) from AttributeSorting " + (if entity_id_in.is_some()) "where entity_id=" + entity_id_in.get else "");
            extract_row_count_from_count_query(sql)
          }

            fn get_relation_to_group_countByGroup(group_id_in: i64) -> i64 {
            extract_row_count_from_count_query("select count(1) from relationtogroup where group_id=" + group_id_in)
          }

          // Idea: make maxValsIn do something here.  How was that missed?  Is it needed?
            fn getRelationsToGroupContainingThisGroup(group_id_in: i64, startingIndexIn: i64, maxValsIn: Option<i64> = None) -> java.util.ArrayList[RelationToGroup] {
            let sql: String = "select rtg.id, rtg.entity_id, rtg.rel_type_id, rtg.group_id, rtg.valid_on_date, rtg.observation_date, asort.sorting_index" +;
                              " from RelationToGroup rtg, AttributeSorting asort where group_id=" + group_id_in +
                              " and rtg.entity_id=asort.entity_id and asort.attribute_form_id=" + Database.get_attribute_form_id(Util::RELATION_TO_GROUP_TYPE) +
                              " and rtg.id=asort.attribute_id"
            let earlyResults = db_query(sql, "i64,i64,i64,i64,i64,i64,i64");
            let final_results = new java.util.ArrayList[RelationToGroup];
            // idea: should the remainder of this method be moved to RelationToGroup, so the persistence layer doesn't know anything about the Model? (helps avoid
            // circular dependencies? is a cleaner design?)
            for (result <- earlyResults) {
              // None of these values should be of "None" type, so not checking for that. If they are it's a bug:
              //final_results.add(result(0).get.asInstanceOf[i64], new Entity(this, result(1).get.asInstanceOf[i64]))
              let rtg: RelationToGroup = new RelationToGroup(this, result(0).get.asInstanceOf[i64], result(1).get.asInstanceOf[i64],;
                                                             result(2).get.asInstanceOf[i64], result(3).get.asInstanceOf[i64],
                                                             if result(4).isEmpty) None else Some(result(4).get.asInstanceOf[i64]), result(5).get.asInstanceOf[i64],
                                                             result(6).get.asInstanceOf[i64])
              final_results.add(rtg)
            }
            require(final_results.size == earlyResults.size)
            final_results
          }

            fn getGroupCount -> i64 {
            extract_row_count_from_count_query("select count(1) from grupo")
          }

     */

          /// @param group_id_in group_id
          /// @param includeWhichEntitiesIn 1/2/3 means select onlyNon-archived/onlyArchived/all entities, respectively.
          ///                                4 means "it depends on the value of include_archived_entities", which is what callers want in some cases.
          ///                                This param might be made more clear, but it is not yet clear how is best to do that.
          ///                                  Because the caller provides this switch specifically to the situation, the logic is not necessarily overridden
          ///                                internally based on the value of this.include_archived_entities.
          fn get_group_size(&self, group_id_in: i64, includeWhichEntitiesIn: i32/*%% = 3*/) -> Result<i64, String> {
              //idea: convert this 1-4 to an enum?
            if includeWhichEntitiesIn <= 0 || includeWhichEntitiesIn >= 5 {
                return Err(format!("Variable includeWhichEntitiesIn ({}) is out of the expected range of 1-4; there is a bug.", includeWhichEntitiesIn));
            }
            let archivedSqlCondition: &str = match includeWhichEntitiesIn {
              1 => "(not archived)",
              2 => "archived",
              3 => "true",
              4 => {
                if self.include_archived_entities() { "true" } else { "(not archived)" }
              }
              _ => return Err(format!("How did we get here? includeWhichEntities={}", includeWhichEntitiesIn)),
            };
            let count = self.extract_row_count_from_count_query(format!("select count(1) from entity e, EntitiesInAGroup \
                eiag where e.id=eiag.entity_id and {} and eiag.group_id={}", archivedSqlCondition, group_id_in).as_str())?;
            Ok(count)
          }

/*
          /** For all groups to which the parameter belongs, returns a collection of the *containing* RelationToGroups, in the form of "entity_name -> groupName"'s.
            * This is useful for example when one is about
            * to delete an entity and we want to warn first, showing where it is contained.
            */
            fn getContainingRelationToGroupDescriptions(entity_id_in: i64, limitIn: Option<i64> = None) -> ArrayList[String] {
            let rows: List[Array[Option[Any]]] = db_query("select e.name, grp.name, grp.id from entity e, relationtogroup rtg, " +;
                                                         "grupo grp where " +
                                                         (if !include_archived_entities) {
                                                           "(not archived) and "
                                                         } else {
                                                           ""
                                                         }) +
                                                         "e.id = rtg.entity_id" +
                                                         " and rtg.group_id = grp.id and rtg.group_id in (SELECT group_id from entitiesinagroup where entity_id=" +
                                                         entity_id_in + ")" +
                                                         " order by grp.id limit " + checkIfShouldBeAllResults(limitIn), "String,String,i64")
            let results: ArrayList[String] = new ArrayList(rows.size);
            for (row <- rows) {
              let entity_name = row(0).get.asInstanceOf[String];
              let groupName = row(1).get.asInstanceOf[String];
              results.add(entity_name + "->" + groupName)
            }
            results
          }

          /** For a given group, find all the RelationsToGroup that contain entities that contain the provided group id, and return their group_ids.
            * What is really the best name for this method (concise but clear on what it does)?
            */
            fn getGroupsContainingEntitysGroupsIds(group_id_in: i64, limitIn: Option<i64> = Some(5)) -> List[Array[Option[Any]]] {
            //get every entity that contains a rtg that contains this group:
            let containingEntityIdList: List[Array[Option[Any]]] = db_query("SELECT entity_id from relationtogroup where group_id=" + group_id_in +;
                                                                           " order by entity_id limit " + checkIfShouldBeAllResults(limitIn), "i64")
            let mut containingEntityIds: String = "";
            //for all those entity ids, get every rtg id containing that entity
            for (row <- containingEntityIdList) {
              let entity_id: i64 = row(0).get.asInstanceOf[i64];
              containingEntityIds += entity_id
              containingEntityIds += ","
            }
            if containingEntityIds.nonEmpty) {
              // remove the last comma
              containingEntityIds = containingEntityIds.substring(0, containingEntityIds.length - 1)
              let rtgRows: List[Array[Option[Any]]] = db_query("SELECT group_id from entitiesinagroup" +;
                                                              " where entity_id in (" + containingEntityIds + ") order by group_id limit " +
                                                              checkIfShouldBeAllResults(limitIn), "i64")
              rtgRows
            } else Nil
          }

          /** Intended to show something like an activity log. Could be used for someone to show their personal journal or for other reporting.
            */
            fn findJournalEntries(startTimeIn: i64, endTimeIn: i64, limitIn: Option<i64> = None) -> ArrayList[(i64, String, i64)] {
            let rows: List[Array[Option[Any]]] = db_query("select insertion_date, 'Added: ' || name, id from entity where insertion_date >= " + startTimeIn +;
                                                                " and insertion_date <= " + endTimeIn +
                                                         " UNION " +
                                                         "select archived_date, 'Archived: ' || name, id from entity where archived and archived_date >= " + startTimeIn +
                                                                " and archived_date <= " + endTimeIn +
                                                         " order by 1 limit " + checkIfShouldBeAllResults(limitIn), "i64,String,i64")
            let results = new ArrayList[(i64, String, i64)];
            let mut n = 0;
            for (row <- rows) {
              results.add((row(0).get.asInstanceOf[i64], row(1).get.asInstanceOf[String], row(2).get.asInstanceOf[i64]))
              n += 1
            }
            results
          }

          override fn getCountOfGroupsContainingEntity(entity_id_in: i64) -> i64 {
            extract_row_count_from_count_query("select count(1) from EntitiesInAGroup where entity_id=" + entity_id_in)
          }

            fn getContainingGroupsIds(entity_id_in: i64) -> ArrayList[i64] {
            let group_ids: List[Array[Option[Any]]] = db_query("select group_id from EntitiesInAGroup where entity_id=" + entity_id_in,;
                                                             "i64")
            let results = new ArrayList[i64];
            for (row <- group_ids) {
              results.add(row(0).get.asInstanceOf[i64])
            }
            results
          }

            fn isEntityInGroup(group_id_in: i64, entity_id_in: i64) -> bool {
            let num = extract_row_count_from_count_query("select count(1) from EntitiesInAGroup eig, entity e where eig.entity_id=e.id" +;
                                                    (if !include_archived_entities) {
                                                      " and (not e.archived)"
                                                    } else {
                                                      ""
                                                    }) +
                                                    " and group_id=" + group_id_in + " and entity_id=" + entity_id_in)
            if num > 1) throw new OmDatabaseException("Entity " + entity_id_in + " is in group " + group_id_in + " " + num + " times?? Should be 0 or 1.")
            num == 1
          }

          fn getQuantityAttributeData(quantityIdIn: i64) -> Array[Option[Any]] {
            db_query_wrapper_for_one_row("select qa.entity_id, qa.unit_id, qa.quantity_number, qa.attr_type_id, qa.valid_on_date, qa.observation_date, asort.sorting_index " +
                                    "from QuantityAttribute qa, AttributeSorting asort where qa.id=" + quantityIdIn +
                                    " and qa.entity_id=asort.entity_id and asort.attribute_form_id=" + Database.get_attribute_form_id(Util::QUANTITY_TYPE) +
                                    " and qa.id=asort.attribute_id",
                                    GET_QUANTITY_ATTRIBUTE_DATA__RESULT_TYPES)
          }

            fn getRelationToLocalEntityData(relation_type_id_in: i64, entity_id1_in: i64, entity_id2_in: i64) -> Array[Option[Any]] {
            db_query_wrapper_for_one_row("select rte.id, rte.valid_on_date, rte.observation_date, asort.sorting_index" +
                                    " from RelationToEntity rte, AttributeSorting asort" +
                                    " where rte.rel_type_id=" + relation_type_id_in + " and rte.entity_id=" + entity_id1_in + " and rte.entity_id_2=" + entity_id2_in +
                                    " and rte.entity_id=asort.entity_id and asort.attribute_form_id=" + Database.get_attribute_form_id(Util.RELATION_TO_LOCAL_ENTITY_TYPE) +
                                    " and rte.id=asort.attribute_id",
                                    Database.GET_RELATION_TO_LOCAL_ENTITY__RESULT_TYPES)
          }

            fn getRelationToLocalEntityDataById(id_in: i64) -> Array[Option[Any]] {
            db_query_wrapper_for_one_row("select rte.rel_type_id, rte.entity_id, rte.entity_id_2, rte.valid_on_date, rte.observation_date, asort.sorting_index" +
                                    " from RelationToEntity rte, AttributeSorting asort" +
                                    " where rte.id=" + id_in +
                                    " and rte.entity_id=asort.entity_id and asort.attribute_form_id=" + Database.get_attribute_form_id(Util.RELATION_TO_LOCAL_ENTITY_TYPE) +
                                    " and rte.id=asort.attribute_id",
                                    "i64,i64," + Database.GET_RELATION_TO_LOCAL_ENTITY__RESULT_TYPES)
          }

            fn getRelationToRemoteEntityData(relation_type_id_in: i64, entity_id1_in: i64, remote_instance_id_in: String, entity_id2_in: i64) -> Array[Option[Any]] {
            db_query_wrapper_for_one_row("select rte.id, rte.valid_on_date, rte.observation_date, asort.sorting_index" +
                                    " from RelationToRemoteEntity rte, AttributeSorting asort" +
                                    " where rte.rel_type_id=" + relation_type_id_in + " and rte.entity_id=" + entity_id1_in +
                                    " and rte.remote_instance_id='" + remote_instance_id_in + "' and rte.entity_id_2=" + entity_id2_in +
                                    " and rte.entity_id=asort.entity_id and asort.attribute_form_id=" + Database.get_attribute_form_id(Util.RELATION_TO_REMOTE_ENTITY_TYPE) +
                                    " and rte.id=asort.attribute_id",
                                    GET_RELATION_TO_REMOTE_ENTITY__RESULT_TYPES)
          }

            fn getAllRelationToLocalEntityDataById(id_in: i64) -> Array[Option[Any]] {
            db_query_wrapper_for_one_row("select form_id, id, rel_type_id, entity_id, entity_id_2, valid_on_date, observation_date from RelationToEntity where id=" + id_in,
                                    "Int,i64,i64,i64,i64,i64,i64")
          }

            fn getAllRelationToRemoteEntityDataById(id_in: i64) -> Array[Option[Any]] {
            db_query_wrapper_for_one_row("select form_id, id, rel_type_id, entity_id, remote_instance_id, entity_id_2, valid_on_date, observation_date" +
                                    " from RelationToRemoteEntity where id=" + id_in,
                                    "Int,i64,i64,i64,String,i64,i64,i64")
          }

            fn getGroupData(id_in: i64) -> Array[Option[Any]] {
            db_query_wrapper_for_one_row("select name, insertion_date, allow_mixed_classes, new_entries_stick_to_top from grupo where id=" + id_in,
                                    GET_GROUP_DATA__RESULT_TYPES)
          }

            fn getRelationToGroupDataByKeys(entity_id: i64, relTypeId: i64, group_id: i64) -> Array[Option[Any]] {
            db_query_wrapper_for_one_row("select rtg.id, rtg.entity_id, rtg.rel_type_id, rtg.group_id, rtg.valid_on_date, rtg.observation_date, asort.sorting_index " +
                                    "from RelationToGroup rtg, AttributeSorting asort" +
                                    " where rtg.entity_id=" + entity_id + " and rtg.rel_type_id=" + relTypeId + " and rtg.group_id=" + group_id +
                                    " and rtg.entity_id=asort.entity_id and asort.attribute_form_id=" + Database.get_attribute_form_id(Util.RELATION_TO_GROUP_TYPE) +
                                    " and rtg.id=asort.attribute_id",
                                    GET_RELATION_TO_GROUP_DATA_BY_KEYS__RESULT_TYPES)
          }

            fn getAllRelationToGroupDataById(id_in: i64) -> Array[Option[Any]] {
            db_query_wrapper_for_one_row("select form_id, id, entity_id, rel_type_id, group_id, valid_on_date, observation_date from RelationToGroup " +
                                    " where id=" + id_in,
                                    "Int,i64,i64,i64,i64,i64,i64")
          }


            fn getRelationToGroupData(id_in: i64) -> Array[Option[Any]] {
            db_query_wrapper_for_one_row("select rtg.id, rtg.entity_id, rtg.rel_type_id, rtg.group_id, rtg.valid_on_date, rtg.observation_date, asort.sorting_index " +
                                    "from RelationToGroup rtg, AttributeSorting asort" +
                                    " where id=" + id_in +
                                    " and rtg.entity_id=asort.entity_id and asort.attribute_form_id=" + Database.get_attribute_form_id(Util.RELATION_TO_GROUP_TYPE) +
                                    " and rtg.id=asort.attribute_id",
                                    GET_RELATION_TO_GROUP_DATA_BY_ID__RESULT_TYPES)
          }

            fn getRelationTypeData(id_in: i64) -> Array[Option[Any]] {
            db_query_wrapper_for_one_row("select name, name_in_reverse_direction, directionality from RelationType r, Entity e where " +
                                    (if !include_archived_entities) {
                                      "(not archived) and "
                                    } else {
                                      ""
                                    }) +
                                    "e.id=r.entity_id " +
                                    "and r.entity_id=" +
                                    id_in,
                                    Database.GET_RELATION_TYPE_DATA__RESULT_TYPES)
          }

          // idea: combine all the methods that look like this (s.b. easier now, in scala, than java)
            fn getTextAttributeData(textIdIn: i64) -> Array[Option[Any]] {
            db_query_wrapper_for_one_row("select ta.entity_id, ta.textValue, ta.attr_type_id, ta.valid_on_date, ta.observation_date, asort.sorting_index" +
                                    " from TextAttribute ta, AttributeSorting asort where id=" + textIdIn +
                                    " and ta.entity_id=asort.entity_id and asort.attribute_form_id=" + Database.get_attribute_form_id(Util.TEXT_TYPE) +
                                    " and ta.id=asort.attribute_id",
                                    GET_TEXT_ATTRIBUTE_DATA__RESULT_TYPES)
          }

            fn getDateAttributeData(dateIdIn: i64) -> Array[Option[Any]] {
            db_query_wrapper_for_one_row("select da.entity_id, da.date, da.attr_type_id, asort.sorting_index " +
                                    "from DateAttribute da, AttributeSorting asort where da.id=" + dateIdIn +
                                    " and da.entity_id=asort.entity_id and asort.attribute_form_id=" + Database.get_attribute_form_id(Util.DATE_TYPE) +
                                    " and da.id=asort.attribute_id",
                                    Database.GET_DATE_ATTRIBUTE_DATA__RESULT_TYPES)
          }

 */
          fn get_boolean_attribute_data(&self, boolean_id_in: i64) -> Result<Vec<DataType>, String> {
              let form_id = match self.get_attribute_form_id(Util::BOOLEAN_TYPE) {
                  None => return Err(format!("No form_id found for {}", Util::BOOLEAN_TYPE)),
                  Some(id) => id,
              };
            self.db_query_wrapper_for_one_row(format!("select ba.entity_id, ba.booleanValue, ba.attr_type_id, ba.valid_on_date, ba.observation_date, asort.sorting_index \
                                    from BooleanAttribute ba, AttributeSorting asort where id={} and ba.entity_id=asort.entity_id and asort.attribute_form_id={} \
                                     and ba.id=asort.attribute_id",
                                                      boolean_id_in, form_id),
                                    Util::GET_BOOLEAN_ATTRIBUTE_DATA__RESULT_TYPES)
          }

    /*
            fn getFileAttributeData(fileIdIn: i64) -> Array[Option[Any]] {
            db_query_wrapper_for_one_row("select fa.entity_id, fa.description, fa.attr_type_id, fa.original_file_date, fa.stored_date, fa.original_file_path, fa.readable, " +
                                    "fa.writable, fa.executable, fa.size, fa.md5hash, asort.sorting_index " +
                                    " from FileAttribute fa, AttributeSorting asort where id=" + fileIdIn +
                                    " and fa.entity_id=asort.entity_id and asort.attribute_form_id=" + Database.get_attribute_form_id(Util.FILE_TYPE) +
                                    " and fa.id=asort.attribute_id",
                                    GET_FILE_ATTRIBUTE_DATA__RESULT_TYPES)
          }

            fn getFileAttributeContent(fileAttributeIdIn: i64, outputStreamIn: java.io.OutputStream) -> (i64, String) {
                fn action(bufferIn: Array[Byte], startingIndexIn: Int, numBytesIn: Int) {
                  outputStreamIn.write(bufferIn, startingIndexIn, numBytesIn)
                }
            let (fileSize, md5hash): (i64, String) = actOnFileFromServer(fileAttributeIdIn, action);
            (fileSize, md5hash)
          }

            fn updateSortingIndexInAGroup(group_id_in: i64, entity_id_in: i64, sorting_index_in: i64) {
            self.db_action(format!("update EntitiesInAGroup set (sorting_index) = ROW(" + sorting_index_in + ") where group_id=" + group_id_in + " and  " +
                     "entity_id=" + entity_id_in).as_str(), false, false);
          }

            fn updateAttributeSortingIndex(entity_id_in: i64, attribute_form_id_in: i64, attribute_id_in: i64, sorting_index_in: i64) {
            self.db_action(format!("update AttributeSorting set (sorting_index) = ROW(" + sorting_index_in + ") where entity_id=" + entity_id_in + " and  " +
                     "attribute_form_id=" + attribute_form_id_in + " and attribute_id=" + attribute_id_in).as_str(), false, false);
          }

          /** Returns whether the stored and calculated md5hashes match, and an error message when they don't.
            */
            fn verifyFileAttributeContentIntegrity(fileAttributeIdIn: i64) -> (Boolean, Option<String>) {
            // Idea: combine w/ similar logic in FileAttribute.md5Hash?
            // Idea: compare actual/stored file sizes also? or does the check of md5 do enough as is?
            // Idea (tracked in tasks): switch to some SHA algorithm since they now say md5 is weaker?
            let messageDigest = java.security.MessageDigest.getInstance("MD5");
            fn action(bufferIn: Array[Byte], startingIndexIn: Int, numBytesIn: Int) {
              messageDigest.update(bufferIn, startingIndexIn, numBytesIn)
            }
            // Next line calls "action" (probably--see javadoc for java.security.MessageDigest for whatever i was thinking at the time)
            // to prepare messageDigest for the digest method to get the md5 value:
            let storedMd5Hash = actOnFileFromServer(fileAttributeIdIn, action)._2;
            //noinspection LanguageFeature ...It is a style violation (advanced feature) but it's what I found when searching for how to do it.
            // outputs same as command 'md5sum <file>'.
            let md5hash: String = messageDigest.digest.map(0xFF &).map {"%02x".format(_)}.foldLeft("") {_ + _};
            if md5hash == storedMd5Hash) (true, None)
            else {
              (false, Some("Mismatched md5hashes: " + storedMd5Hash + " (stored in the md5sum db column) != " + md5hash + "(calculated from stored file contents)"))
            }
          }

          /** This is a no-op, called in actOnFileFromServer, that a test can customize to simulate a corrupted file on the server. */
          //noinspection ScalaUselessExpression (...intentional style violation, for readability)
            fn damageBuffer(buffer: Array[Byte]) /* -> Unit = Unit%%*/

          /** Returns the file size (having confirmed it is the same as the # of bytes processed), and the md5hash that was stored with the document.
            */
            fn actOnFileFromServer(fileAttributeIdIn: i64, actionIn: (Array[Byte], Int, Int) => Unit) -> (i64, String) {
            let mut obj: LargeObject = null;
            try {
              // even though we're not storing data, the instructions (see create_tables re this...) said to have it in a transaction.
              self.begin_trans()
              let lobjManager: LargeObjectManager = connection.asInstanceOf[org.postgresql.PGConnection].getLargeObjectAPI;
              let oidOption: Option<i64> = db_query_wrapper_for_one_row("select contents_oid from FileAttributeContent where file_attribute_id=" + fileAttributeIdIn,;
                                                                    "i64")(0).asInstanceOf[Option<i64>]
              if oidOption.isEmpty) throw new OmDatabaseException("No contents found for file attribute id " + fileAttributeIdIn)
              let oid: i64 = oidOption.get;
              obj = lobjManager.open(oid, LargeObjectManager.READ)
              // Using 4096 only because this url:
              //   https://commons.apache.org/proper/commons-io/javadocs/api-release/org/apache/commons/io/IOUtils.html
              // ...said, at least for that purpose, that: "The default buffer size of 4K has been shown to be efficient in tests." (retrieved 2016-12-05)
              let buffer = new Array[Byte](4096);
              let mut numBytesRead = 0;
              let mut total: i64 = 0;
              @tailrec
              fn readFileFromDbAndActOnIt() {
                //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) w/in this method!
                numBytesRead = obj.read(buffer, 0, buffer.length)
                // (intentional style violation, for readability):
                //noinspection ScalaUselessExpression
                if numBytesRead <= 0) Unit
                else {
                  // just once by a test subclass is enough to mess w/ the md5sum.
                  if total == 0) damageBuffer(buffer)

                  actionIn(buffer, 0, numBytesRead)
                  total += numBytesRead
                  readFileFromDbAndActOnIt()
                }
              }
              readFileFromDbAndActOnIt()
              let resultOption = db_query_wrapper_for_one_row("select size, md5hash from fileattribute where id=" + fileAttributeIdIn, "i64,String");
              if resultOption(0).isEmpty) throw new OmDatabaseException("No result from query for fileattribute for id " + fileAttributeIdIn + ".")
              let (contentSize, md5hash) = (resultOption(0).get.asInstanceOf[i64], resultOption(1).get.asInstanceOf[String]);
              if total != contentSize) {
                throw new OmFileTransferException("Transferred " + total + " bytes instead of " + contentSize + "??")
              }
              commit_trans()
              (total, md5hash)
            } catch {
              case e: Exception => throw rollbackWithCatch(e)
            } finally {
              try {
                obj.close()
              } catch {
                case e: Exception =>
                // not sure why this fails sometimes, if it's a bad thing or not, but for now not going to be stuck on it.
                // idea: look at the source code.
              }
            }
          }

            fn quantityAttributeKeyExists(id_in: i64) -> bool {
             does_this_exist("SELECT count(1) from QuantityAttribute where id=" + id_in)
             }

            fn textAttributeKeyExists(id_in: i64) -> bool {
             does_this_exist("SELECT count(1) from TextAttribute where id=" + id_in)
             }

            fn dateAttributeKeyExists(id_in: i64) -> bool {
             does_this_exist("SELECT count(1) from DateAttribute where id=" + id_in)
             }

     */
            fn boolean_attribute_key_exists(&self, id_in: i64) -> Result<bool, String> {
             self.does_this_exist(format!("SELECT count(1) from BooleanAttribute where id={}", id_in).as_str(), true)
             }

    /*
            fn fileAttributeKeyExists(id_in: i64) -> bool {
            does_this_exist("SELECT count(1) from FileAttribute where id=" + id_in)
            }

            fn relationToLocal_entity_key_exists(id_in: i64) -> bool {
             does_this_exist("SELECT count(1) from RelationToEntity where id=" + id_in)
             }

            fn relationToRemote_entity_key_exists(id_in: i64) -> bool {
            does_this_exist("SELECT count(1) from RelationToRemoteEntity where id=" + id_in)
            }

            fn relationToGroupKeyExists(id_in: i64) -> bool {
            does_this_exist("SELECT count(1) from RelationToGroup where id=" + id_in)
            }

            fn relationToGroupKeysExist(entity_id: i64, relationTypeId: i64, group_id: i64) -> bool {
            does_this_exist("SELECT count(1) from RelationToGroup where entity_id=" + entity_id + " and rel_type_id=" + relationTypeId + " and group_id=" + group_id)
            }

            fn attribute_key_exists(form_id_in: i64, id_in: i64) -> bool {
              //MAKE SURE THESE MATCH WITH THOSE IN get_attribute_form_id !
              form_id_in match {
                case 1 => quantityAttributeKeyExists(id_in)
                case 2 => dateAttributeKeyExists(id_in)
                case 3 => boolean_attribute_key_exists(id_in)
                case 4 => fileAttributeKeyExists(id_in)
                case 5 => textAttributeKeyExists(id_in)
                case 6 => relationToLocal_entity_key_exists(id_in)
                case 7 => relationToGroupKeyExists(id_in)
                case 8 => relationToRemote_entity_key_exists(id_in)
                case _ => throw new OmDatabaseException("unexpected")
              }
          }

          /** Excludes those entities that are really relationtypes, attribute types, or quantity units. */
            fn entityOnlyKeyExists(id_in: i64) -> bool {
            does_this_exist("SELECT count(1) from Entity where " +
                          (if !include_archived_entities) "(not archived) and " else "") +
                          "id=" + id_in + " and id in (select id from entity " + limit_to_entities_only(Self::ENTITY_ONLY_SELECT_PART) + ")")
          }
    */
     /// @param include_archived See comment on similar parameter to method get_group_size.
     //idea: see if any callers should pass the include_archived parameter differently, now that the system can be used with archived entities displayed.
     fn entity_key_exists(&self, id_in: i64, include_archived: bool) -> Result<bool, String> {
       let condition = if !include_archived { " and not archived"
       } else {
           ""
       };
       self.does_this_exist(format!("SELECT count(1) from Entity where id={}{}", id_in, condition).as_str(), true)
     }

                fn is_group_entry_sorting_index_in_use(&self, group_id_in: i64, sorting_index_in: i64) -> Result<bool, String> {
                 self.does_this_exist(format!("SELECT count(1) from Entitiesinagroup where group_id={} and sorting_index={}", group_id_in, sorting_index_in).as_str(), true)
                }

    /*
                fn classKeyExists(id_in: i64) -> bool {
                does_this_exist("SELECT count(1) from class where id=" + id_in)
                }

                fn relationTypeKeyExists(id_in: i64) -> bool {
                does_this_exist("SELECT count(1) from RelationType where entity_id=" + id_in)
                }

                fn relationToLocalEntityKeysExistAndMatch(id_in: i64, relTypeIdIn: i64, entity_id1_in: i64, entity_id2_in: i64) -> bool {
                does_this_exist("SELECT count(1) from RelationToEntity where id=" + id_in + " and rel_type_id=" + relTypeIdIn + " and entity_id=" + entity_id1_in +
                              " and entity_id_2=" + entity_id2_in)
              }

                fn relationToRemoteEntityKeysExistAndMatch(id_in: i64, relTypeIdIn: i64, entity_id1_in: i64, remote_instance_id_in: String, entity_id2_in: i64) -> bool {
                does_this_exist("SELECT count(1) from RelationToRemoteEntity where id=" + id_in + " and rel_type_id=" + relTypeIdIn + " and entity_id=" + entity_id1_in +
                              " and remote_instance_id='" + remote_instance_id_in + "' and entity_id_2=" + entity_id2_in)
              }

                fn relationToLocalEntityExists(relTypeIdIn: i64, entity_id1_in: i64, entity_id2_in: i64) -> bool {
                does_this_exist("SELECT count(1) from RelationToEntity where rel_type_id=" + relTypeIdIn + " and entity_id=" + entity_id1_in +
                              " and entity_id_2=" + entity_id2_in)
              }

                fn relationToRemoteEntityExists(relTypeIdIn: i64, entity_id1_in: i64, remote_instance_id_in: String, entity_id2_in: i64) -> bool {
                does_this_exist("SELECT count(1) from RelationToRemoteEntity where rel_type_id=" + relTypeIdIn + " and entity_id=" + entity_id1_in +
                              " and remote_instance_id='" + remote_instance_id_in + "' and entity_id_2=" + entity_id2_in)
              }

                fn groupKeyExists(id_in: i64) -> bool {
                does_this_exist("SELECT count(1) from grupo where id=" + id_in)
              }

                fn relationToGroupKeysExistAndMatch(id: i64, entity_id: i64, relTypeId: i64, group_id: i64) -> bool {
                does_this_exist("SELECT count(1) from RelationToGroup where id=" + id + " and entity_id=" + entity_id + " and rel_type_id=" + relTypeId +
                              " and group_id=" + group_id)
              }

              /**
               * Allows querying for a range of objects in the database; returns a java.util.Map with keys and names.
               * 1st parm is index to start with (0-based), 2nd parm is # of obj's to return (if None, means no limit).
               */
                fn getEntities(startingObjectIndexIn: i64, maxValsIn: Option<i64> = None) -> Vec<Entity> {
                getEntitiesGeneric(startingObjectIndexIn, maxValsIn, Util.ENTITY_TYPE)
              }

              /** Excludes those entities that are really relationtypes, attribute types, or quantity units. Otherwise similar to getEntities.
                *
                * *****NOTE*****: The limitByClass:Boolean parameter is not redundant with the inClassId: inClassId could be None and we could still want
                * to select only those entities whose class_id is NULL, such as when enforcing group uniformity (see method has_mixed_classes and its
                * uses, for more info).
                *
                * The parameter omitEntity is (at this writing) used for the id of a class-defining (template) entity, which we shouldn't show for editing when showing all the
                * entities in the class (editing that is a separate menu option), otherwise it confuses things.
                * */
                fn getEntitiesOnly(startingObjectIndexIn: i64, maxValsIn: Option<i64> = None, class_id_in: Option<i64> = None,
                                  limitByClass: bool = false, templateEntity: Option<i64> = None,
                                  groupToOmitIdIn: Option<i64> = None) -> Vec<Entity> {
                getEntitiesGeneric(startingObjectIndexIn, maxValsIn, "EntityOnly", class_id_in, limitByClass, templateEntity, groupToOmitIdIn)
              }

              /** similar to getEntities */
                fn getRelationTypes(startingObjectIndexIn: i64, maxValsIn: Option<i64> = None) -> Vec<Entity> {
                getEntitiesGeneric(startingObjectIndexIn, maxValsIn, Util.RELATION_TYPE_TYPE)
              }

              let selectEntityStart = "SELECT e.id, e.name, e.class_id, e.insertion_date, e.public, e.archived, e.new_entries_stick_to_top ";

                fn addNewEntityToResults(final_results: Vec<Entity>, intermediateResultIn: Array[Option[Any]]) -> bool {
                let result = intermediateResultIn;
                // None of these values should be of "None" type, so not checking for that. If they are it's a bug:
                final_results.add(new Entity(this, result(0).get.asInstanceOf[i64], result(1).get.asInstanceOf[String], result(2).asInstanceOf[Option<i64>],
                                            result(3).get.asInstanceOf[i64], result(4).asInstanceOf[Option<bool>], result(5).get.asInstanceOf[Boolean],
                                            result(6).get.asInstanceOf[Boolean]))
              }

                fn getMatchingEntities(startingObjectIndexIn: i64, maxValsIn: Option<i64> = None, omitEntityIdIn: Option<i64>,
                                      nameRegexIn: String) -> Vec<Entity> {
                let nameRegex = self.escape_quotes_etc(nameRegexIn);
                let omissionExpression: String = if omitEntityIdIn.isEmpty) "true" else "(not id=" + omitEntityIdIn.get + ")";
                let sql: String = selectEntityStart + " from entity e where " +;
                                  (if !include_archived_entities) {
                                    "not archived and "
                                  } else {
                                    ""
                                  }) +
                                  omissionExpression +
                                  " and name ~* '" + nameRegex + "'" +
                                  " UNION " +
                                  "select id, name, class_id, insertion_date, public, archived, new_entries_stick_to_top from entity where " +
                                  (if !include_archived_entities) {
                                    "not archived and "
                                  } else {
                                    ""
                                  }) +
                                  omissionExpression +
                                  " and id in (select entity_id from textattribute where textValue ~* '" + nameRegex + "')" +
                                  " ORDER BY" +
                                  " id limit " + checkIfShouldBeAllResults(maxValsIn) + " offset " + startingObjectIndexIn
                let earlyResults = db_query(sql, "i64,String,i64,i64,Boolean,Boolean,Boolean");
                let final_results = new Vec<Entity>;
                // idea: (see getEntitiesGeneric for idea, see if applies here)
                for (result <- earlyResults) {
                  addNewEntityToResults(final_results, result)
                }
                require(final_results.size == earlyResults.size)
                final_results
              }

                fn getMatchingGroups(startingObjectIndexIn: i64, maxValsIn: Option<i64> = None, omitGroupIdIn: Option<i64>,
                                    nameRegexIn: String) -> java.util.ArrayList[Group] {
                let nameRegex = self.escape_quotes_etc(nameRegexIn);
                let omissionExpression: String = if omitGroupIdIn.isEmpty) "true" else "(not id=" + omitGroupIdIn.get + ")";
                let sql: String = s"select id, name, insertion_date, allow_mixed_classes, new_entries_stick_to_top from grupo where name ~* '$nameRegex'" +;
                                  " and " + omissionExpression + " order by id limit " + checkIfShouldBeAllResults(maxValsIn) + " offset " + startingObjectIndexIn
                let earlyResults = db_query(sql, "i64,String,i64,Boolean,Boolean");
                let final_results = new java.util.ArrayList[Group];
                // idea: (see getEntitiesGeneric for idea, see if applies here)
                for (result <- earlyResults) {
                  // None of these values should be of "None" type, so not checking for that. If they are it's a bug:
                  final_results.add(new Group(this, result(0).get.asInstanceOf[i64], result(1).get.asInstanceOf[String], result(2).get.asInstanceOf[i64],
                                             result(3).get.asInstanceOf[Boolean], result(4).get.asInstanceOf[Boolean]))
                }
                require(final_results.size == earlyResults.size)
                final_results
              }

                fn getContainingEntities_helper(sql_in: String) -> java.util.ArrayList[(i64, Entity)] {
                let earlyResults = db_query(sql_in, "i64,i64");
                let final_results = new java.util.ArrayList[(i64, Entity)];
                // idea: should the remainder of this method be moved to Entity, so the persistence layer doesn't know anything about the Model? (helps avoid circular
                // dependencies? is a cleaner design?.)
                for (result <- earlyResults) {
                  // None of these values should be of "None" type, so not checking for that. If they are it's a bug:
                  let rel_type_id: i64 = result(0).get.asInstanceOf[i64];
                  let entity: Entity = new Entity(this, result(1).get.asInstanceOf[i64]);
                  final_results.add((rel_type_id, entity))
                }

                require(final_results.size == earlyResults.size)
                final_results
              }

                fn getLocalEntitiesContainingLocalEntity(entity_id_in: i64, startingIndexIn: i64, maxValsIn: Option<i64> = None) -> java.util.ArrayList[(i64, Entity)] {
                let sql: String = "select rel_type_id, entity_id from relationtoentity rte, entity e where rte.entity_id=e.id and rte.entity_id_2=" + entity_id_in +;
                                  (if !include_archived_entities) {
                                    " and (not e.archived)"
                                  } else {
                                    ""
                                  }) +
                                  " order by entity_id limit " + checkIfShouldBeAllResults(maxValsIn) + " offset " + startingIndexIn
                //note/idea: this should be changed when we update relation stuff similarly, to go both ways in the relation (either entity_id or
                // entity_id_2: helpfully returned; & in UI?)
                getContainingEntities_helper(sql)
              }

                fn getEntitiesContainingGroup(group_id_in: i64, startingIndexIn: i64, maxValsIn: Option<i64> = None) -> java.util.ArrayList[(i64, Entity)] {
                let sql: String = "select rel_type_id, entity_id from relationtogroup where group_id=" + group_id_in +;
                                  " order by entity_id, rel_type_id limit " +
                                  checkIfShouldBeAllResults(maxValsIn) + " offset " + startingIndexIn
                //note/idea: this should be changed when we update relation stuff similarly, to go both ways in the relation (either entity_id or
                // entity_id_2: helpfully returned; & in UI?)
                //And, perhaps changed to account for whether something is archived.
                // See getCountOfEntitiesContainingGroup for example.
                getContainingEntities_helper(sql)
              }

              /**
               * @return A tuple showing the # of non-archived entities and the # of archived entities that directly refer to this entity (IN *ONE* DIRECTION ONLY).
               */
                fn getCountOfLocalEntitiesContainingLocalEntity(entity_id_in: i64) -> (i64, i64) {
                let nonArchived2 = extract_row_count_from_count_query("select count(1) from relationtoentity rte, entity e where e.id=rte.entity_id_2 and not e.archived" +;
                                                                 " and e.id=" + entity_id_in)
                let archived2 = extract_row_count_from_count_query("select count(1) from relationtoentity rte, entity e where e.id=rte.entity_id_2 and e.archived" +;
                                                              " and e.id=" + entity_id_in)

                (nonArchived2, archived2)
              }

              /**
               * @return A tuple showing the # of non-archived entities and the # of archived entities that directly refer to this group.
               */
                fn getCountOfEntitiesContainingGroup(group_id_in: i64) -> (i64, i64) {
                let nonArchived = extract_row_count_from_count_query("select count(1) from relationtogroup rtg, entity e where e.id=rtg.entity_id and not e.archived" +;
                                                                " and rtg.group_id=" + group_id_in)
                let archived = extract_row_count_from_count_query("select count(1) from relationtogroup rtg, entity e where e.id=rtg.entity_id and e.archived" +;
                                                             " and rtg.group_id=" + group_id_in)
                (nonArchived, archived)
              }

                fn getContainingRelationsToGroup(entity_id_in: i64, startingIndexIn: i64, maxValsIn: Option<i64> = None) -> java.util.ArrayList[RelationToGroup] {
                // BUG (tracked in tasks): there is a disconnect here between this method and its _helper method, because one uses the eig table, the other the rtg table,
                // and there is no requirement/enforcement that all groups defined in eig are in an rtg, so they could get dif't/unexpected results.
                // So, could: see the expectation of the place(s) calling this method, if uniform, make these 2 methods more uniform in what they do in meeting that,
                // OR, could consider whether we really should have an enforcement between the 2 tables...?
                // THIS BUg currently prevents searching for then deleting the entity w/ this in name: "OTHER ENTITY NOTED IN A DELETION BUG" (see also other issue
                // in Controller.java where that same name is mentioned. Related, be cause in that case on the line:
                //    "descriptions = descriptions.substring(0, descriptions.length - delimiter.length) + ".  ""
                // ...one gets the below exception throw, probably for the same or related reason:
                    /*
                    ==============================================
                    **CURRENT ENTITY:while at it, order a valentine's card on amazon asap (or did w/ cmas shopping?)
                    No attributes have been assigned to this object, yet.
                    1-Add attribute (quantity, true/false, date, text, external file, relation to entity or group: i.e., ownership of or "has" another entity, family ties, etc)...
                    2-Import/Export...
                    3-Edit name
                    4-Delete or Archive...
                    5-Go to...
                    6-List next items
                    7-Set current entity (while at it, order a valentine's card on amazon asap (or did w/ cmas shopping?)) as default (first to come up when launching this program.)
                    8-Edit public/nonpublic status
                    0/ESC - back/previous menu
                    4


                    ==============================================
                    Choose a deletion or archiving option:
                    1-Delete this entity
                             2-Archive this entity (remove from visibility but not permanent/total deletion)
                    0/ESC - back/previous menu
                    1
                    An error occurred: "java.lang.StringIndexOutOfBoundsException: String index out of range: -2".  If you can provide simple instructions to reproduce it consistently, maybe it can be fixed.  Do you want to see the detailed output? (y/n):
                      y


                    ==============================================
                    java.lang.StringIndexOutOfBoundsException: String index out of range: -2
                    at java.lang.String.substring(String.java:1911)
                    at org.onemodel.Controller.Controller.deleteOrArchiveEntity(Controller.scala:644)
                    at org.onemodel.Controller.EntityMenu.entityMenu(EntityMenu.scala:232)
                    at org.onemodel.Controller.EntityMenu.entityMenu(EntityMenu.scala:388)
                    at org.onemodel.Controller.Controller.showInEntityMenuThenMainMenu(Controller.scala:277)
                    at org.onemodel.Controller.MainMenu.mainMenu(MainMenu.scala:80)
                    at org.onemodel.Controller.MainMenu.mainMenu(MainMenu.scala:98)
                    at org.onemodel.Controller.MainMenu.mainMenu(MainMenu.scala:98)
                    at org.onemodel.Controller.Controller.menuLoop$1(Controller.scala:140)
                    at org.onemodel.Controller.Controller.start(Controller.scala:143)
                    at org.onemodel.TextUI.launchUI(TextUI.scala:220)
                    at org.onemodel.TextUI$.main(TextUI.scala:34)
                    at org.onemodel.TextUI.main(TextUI.scala:1)
                    */

                let sql: String = "select group_id from entitiesinagroup where entity_id=" + entity_id_in + " order by group_id limit " +;
                                  checkIfShouldBeAllResults(maxValsIn) + " offset " + startingIndexIn
                getContainingRelationToGroups_helper(sql)
              }

                fn getContainingRelationToGroups_helper(sql_in: String) -> java.util.ArrayList[RelationToGroup] {
                let earlyResults = db_query(sql_in, "i64");
                let group_idResults = new java.util.ArrayList[i64];
                // idea: should the remainder of this method be moved to Group, so the persistence layer doesn't know anything about the Model? (helps avoid circular
                // dependencies? is a cleaner design?)
                for (result <- earlyResults) {
                  //val group:Group = new Group(this, result(0).asInstanceOf[i64])
                  group_idResults.add(result(0).get.asInstanceOf[i64])
                }
                require(group_idResults.size == earlyResults.size)
                let containingRelationsToGroup: java.util.ArrayList[RelationToGroup] = new java.util.ArrayList[RelationToGroup];
                for (gid <- group_idResults.toArray) {
                  let rtgs = getRelationsToGroupContainingThisGroup(gid.asInstanceOf[i64], 0);
                  for (rtg <- rtgs.toArray) containingRelationsToGroup.add(rtg.asInstanceOf[RelationToGroup])
                }
                containingRelationsToGroup
              }

                fn getEntitiesUsedAsAttributeTypes_sql(attributeTypeIn: String, quantitySeeksUnitNotTypeIn: bool) -> String {
                let mut sql: String = " from Entity e where " +;
                                  // whether it is archived doesn't seem relevant in the use case, but, it is debatable:
                                  //              (if !include_archived_entities) {
                                  //                "(not archived) and "
                                  //              } else {
                                  //                ""
                                  //              }) +
                                  " e.id in (select " +
                                  {
                                    // IN MAINTENANCE: compare to logic in method limit_to_entities_only.
                                    if Util.QUANTITY_TYPE == attributeTypeIn && quantitySeeksUnitNotTypeIn) "unit_id"
                                    else if Util.NON_RELATION_ATTR_TYPE_NAMES.contains(attributeTypeIn)) "attr_type_id"
                                    else if Util.RELATION_TYPE_TYPE == attributeTypeIn) "entity_id"
                                    else if Util.RELATION_ATTR_TYPE_NAMES.contains(attributeTypeIn)) "rel_type_id"
                                    else throw new Exception("unexpected attributeTypeIn: " + attributeTypeIn)
                                  } +
                                  " from "
                if Util.NON_RELATION_ATTR_TYPE_NAMES.contains(attributeTypeIn) || Util.RELATION_ATTR_TYPE_NAMES.contains(attributeTypeIn)) {
                  // it happens to match the table name, which is convenient:
                  sql = sql + attributeTypeIn + ")"
                } else {
                  throw new Exception("unexpected attributeTypeIn: " + attributeTypeIn)
                }
                sql
              }

                fn getCountOfEntitiesUsedAsAttributeTypes(attributeTypeIn: String, quantitySeeksUnitNotTypeIn: bool) -> i64 {
                let sql = "SELECT count(1) " + getEntitiesUsedAsAttributeTypes_sql(attributeTypeIn, quantitySeeksUnitNotTypeIn);
                extract_row_count_from_count_query(sql)
              }

                fn getEntitiesUsedAsAttributeTypes(attributeTypeIn: String, startingObjectIndexIn: i64, maxValsIn: Option<i64> = None,
                                                  quantitySeeksUnitNotTypeIn: bool) -> Vec<Entity> {
                let sql: String = selectEntityStart + getEntitiesUsedAsAttributeTypes_sql(attributeTypeIn, quantitySeeksUnitNotTypeIn);
                let earlyResults = db_query(sql, "i64,String,i64,i64,Boolean,Boolean,Boolean");
                let final_results = new Vec<Entity>;
                // idea: should the remainder of this method be moved to Entity, so the persistence layer doesn't know anything about the Model? (helps avoid circular
                // dependencies; is a cleaner design.)  (and similar ones)
                for (result <- earlyResults) {
                  addNewEntityToResults(final_results, result)
                }
                require(final_results.size == earlyResults.size)
                final_results
              }

              // 1st parm is 0-based index to start with, 2nd parm is # of obj's to return (if None, means no limit).
                fn getEntitiesGeneric(startingObjectIndexIn: i64, maxValsIn: Option<i64>, table_name_in: String,
                                             class_id_in: Option<i64> = None, limitByClass: bool = false,
                                             templateEntity: Option<i64> = None, groupToOmitIdIn: Option<i64> = None) -> Vec<Entity> {
                let sql: String = selectEntityStart +;
                                  (if table_name_in.compareToIgnoreCase(Util.RELATION_TYPE_TYPE) == 0) ", r.name_in_reverse_direction, r.directionality " else "") +
                                  " from Entity e " +
                                  (if table_name_in.compareToIgnoreCase(Util.RELATION_TYPE_TYPE) == 0) {
                                    // for RelationTypes, hit both tables since one "inherits", but limit it to those rows
                                    // for which a RelationType row also exists.
                                    ", RelationType r "
                                  } else "") +
                                  " where" +
                                  (if !include_archived_entities) {
                                    " (not archived) and"
                                  } else {
                                    ""
                                  }) +
                                  " true " +
                                  classLimit(limitByClass, class_id_in) +
                                  (if limitByClass && templateEntity.is_some()) " and id != " + templateEntity.get else "") +
                                  (if table_name_in.compareToIgnoreCase(Util.RELATION_TYPE_TYPE) == 0) {
                                    // for RelationTypes, hit both tables since one "inherits", but limit it to those rows
                                    // for which a RelationType row also exists.
                                    " and e.id = r.entity_id "
                                  } else "") +
                                  (if table_name_in.compareToIgnoreCase("EntityOnly") == 0) limit_to_entities_only(selectEntityStart) else "") +
                                  (if groupToOmitIdIn.is_some()) " except (" + selectEntityStart + " from entity e, " +
                                                                "EntitiesInAGroup eiag where e.id=eiag.entity_id and " +
                                                                "group_id=" + groupToOmitIdIn.get + ")"
                                  else "") +
                                  " order by id limit " + checkIfShouldBeAllResults(maxValsIn) + " offset " + startingObjectIndexIn
                let earlyResults = db_query(sql,;
                                           if table_name_in.compareToIgnoreCase(Util.RELATION_TYPE_TYPE) == 0) {
                                             "i64,String,i64,i64,Boolean,Boolean,String,String"
                                           } else {
                                             "i64,String,i64,i64,Boolean,Boolean,Boolean"
                                           })
                let final_results = new Vec<Entity>;
                // idea: should the remainder of this method be moved to Entity, so the persistence layer doesn't know anything about the Model? (helps avoid circular
                // dependencies; is a cleaner design.)  (and similar ones)
                for (result <- earlyResults) {
                  // None of these values should be of "None" type, so not checking for that. If they are it's a bug:
                  if table_name_in.compareToIgnoreCase(Util.RELATION_TYPE_TYPE) == 0) {
                    final_results.add(new RelationType(this, result(0).get.asInstanceOf[i64], result(1).get.asInstanceOf[String], result(6).get.asInstanceOf[String],
                                                      result(7).get.asInstanceOf[String]))
                  } else {
                    addNewEntityToResults(final_results, result)
                  }
                }
                require(final_results.size == earlyResults.size)
                final_results
              }

              /** Allows querying for a range of objects in the database; returns a java.util.Map with keys and names.
                * 1st parm is index to start with (0-based), 2nd parm is # of obj's to return (if None, means no limit).
                */
                fn getGroups(startingObjectIndexIn: i64, maxValsIn: Option<i64> = None, groupToOmitIdIn: Option<i64> = None) -> java.util.ArrayList[Group] {
                let omissionExpression: String = {;
                  if groupToOmitIdIn.isEmpty) {
                    "true"
                  } else {
                    "(not id=" + groupToOmitIdIn.get + ")"
                  }
                }
                let sql = "SELECT id, name, insertion_date, allow_mixed_classes, new_entries_stick_to_top from grupo " +;
                          " where " + omissionExpression +
                          " order by id limit " + checkIfShouldBeAllResults(maxValsIn) + " offset " + startingObjectIndexIn
                let earlyResults = db_query(sql, "i64,String,i64,Boolean,Boolean");
                let final_results = new java.util.ArrayList[Group];
                // idea: should the remainder of this method be moved to RTG, so the persistence layer doesn't know anything about the Model? (helps avoid circular
                // dependencies; is a cleaner design.)
                for (result <- earlyResults) {
                  // None of these values should be of "None" type, so not checking for that. If they are it's a bug:
                  final_results.add(new Group(this, result(0).get.asInstanceOf[i64], result(1).get.asInstanceOf[String], result(2).get.asInstanceOf[i64],
                                             result(3).get.asInstanceOf[Boolean], result(4).get.asInstanceOf[Boolean]))
                }
                require(final_results.size == earlyResults.size)
                final_results
              }


                fn getClasses(startingObjectIndexIn: i64, maxValsIn: Option<i64> = None) -> java.util.ArrayList[EntityClass] {
                let sql: String = "SELECT id, name, defining_entity_id, create_default_attributes from class order by id limit " +;
                                  checkIfShouldBeAllResults(maxValsIn) + " offset " + startingObjectIndexIn
                let earlyResults = db_query(sql, "i64,String,i64,Boolean");
                let final_results = new java.util.ArrayList[EntityClass];
                // idea: should the remainder of this method be moved to EntityClass, so the persistence layer doesn't know anything about the Model? (helps avoid circular
                // dependencies; is a cleaner design; see similar comment in getEntitiesGeneric.)
                for (result <- earlyResults) {
                  // Only one of these values should be of "None" type, so not checking the others for that. If they are it's a bug:
                  final_results.add(new EntityClass(this, result(0).get.asInstanceOf[i64], result(1).get.asInstanceOf[String], result(2).get.asInstanceOf[i64],
                                                   if result(3).isEmpty) None else Some(result(3).get.asInstanceOf[Boolean])))
                }
                require(final_results.size == earlyResults.size)
                final_results
              }

                fn checkIfShouldBeAllResults(maxValsIn: Option<i64>) -> String {
                if maxValsIn.isEmpty) "ALL"
                else if maxValsIn.get <= 0) "1"
                else maxValsIn.get.toString
              }

                fn getGroupEntriesData(group_id_in: i64, limitIn: Option<i64> = None, include_archived_entitiesIn: bool = true) -> List[Array[Option[Any]]] {
                // LIKE THE OTHER 3 BELOW SIMILAR METHODS:
                // Need to make sure it gets the desired rows, rather than just some, so the order etc matters at each step, probably.
                // idea: needs automated tests (in task list also).
                let mut sql: String = "select eiag.entity_id, eiag.sorting_index from entity e, entitiesinagroup eiag where e.id=eiag.entity_id" +;
                                      " and eiag.group_id=" + group_id_in
                if !include_archived_entitiesIn && !include_archived_entities) sql += " and (not e.archived)"
                sql += " order by eiag.sorting_index, eiag.entity_id limit " + checkIfShouldBeAllResults(limitIn)
                let results = db_query(sql, GET_GROUP_ENTRIES_DATA__RESULT_TYPES);
                results
              }

                fn getEntityAttributeSortingData(entity_id_in: i64, limitIn: Option<i64> = None) -> List[Array[Option[Any]]] {
                // see comments in getGroupEntriesData
                let results = db_query("select attribute_form_id, attribute_id, sorting_index from AttributeSorting where entity_id = " + entity_id_in +;
                                      " order by sorting_index limit " + checkIfShouldBeAllResults(limitIn),
                                      "Int,i64,i64")
                results
              }

                fn getAdjacentGroupEntriesSortingIndexes(group_id_in: i64, sorting_index_in: i64, limitIn: Option<i64> = None,
                                                        forwardNotBackIn: bool) -> ListArray[Option[Any]]] {
                // see comments in getGroupEntriesData.
                // Doing "not e.archived", because the caller is probably trying to move entries up/down in the UI, and if we count archived entries but
                // are not showing them,
                // we could move relative to invisible entries only, and not make a visible move,  BUT: as of 2014-8-4, a comment was added, now gone, that said to ignore
                // archived entities while getting a new sorting_index is a bug. So if that bug is found again, we should cover all scenarios with automated
                // tests (showAllArchivedEntities is true and false, with archived entities present, and any other).
                let results = db_query("select eiag.sorting_index from entity e, entitiesinagroup eiag where e.id=eiag.entity_id" +;
                                      (if !include_archived_entities) {
                                        " and (not e.archived)"
                                      } else {
                                        ""
                                      }) +
                                      " and eiag.group_id=" + group_id_in + " and eiag.sorting_index " + (if forwardNotBackIn) ">" else "<") + sorting_index_in +
                                      " order by eiag.sorting_index " + (if forwardNotBackIn) "ASC" else "DESC") + ", eiag.entity_id " +
                                      " limit " + checkIfShouldBeAllResults(limitIn),
                                      "i64")
                results
              }

                fn getAdjacentAttributesSortingIndexes(entity_id_in: i64, sorting_index_in: i64, limitIn: Option<i64>, forwardNotBackIn: bool) -> ListArray[Option[Any]]] {
                let results = db_query("select sorting_index from AttributeSorting where entity_id=" + entity_id_in +;
                                      " and sorting_index" + (if forwardNotBackIn) ">" else "<") + sorting_index_in +
                                      " order by sorting_index " + (if forwardNotBackIn) "ASC" else "DESC") +
                                      " limit " + checkIfShouldBeAllResults(limitIn),
                                      "i64")
                results
              }

              /** This one should explicitly NOT omit archived entities (unless parameterized for that later). See caller's comments for more, on purpose.
                */
                fn getNearestGroupEntrysSortingIndex(group_id_in: i64, startingPointSortingIndexIn: i64, forwardNotBackIn: bool) -> Option<i64> {
                let results = db_query("select sorting_index from entitiesinagroup where group_id=" + group_id_in + " and sorting_index " +;
                                      (if forwardNotBackIn) ">" else "<") + startingPointSortingIndexIn +
                                      " order by sorting_index " + (if forwardNotBackIn) "ASC" else "DESC") +
                                      " limit 1",
                                      "i64")
                if results.isEmpty) {
                  None
                } else {
                  if results.size > 1) throw new OmDatabaseException("Probably the caller didn't expect this to get >1 results...Is that even meaningful?")
                  else results.head(0).asInstanceOf[Option<i64>]
                }
              }

                fn getNearestAttributeEntrysSortingIndex(entity_id_in: i64, startingPointSortingIndexIn: i64, forwardNotBackIn: bool) -> Option<i64> {
                let results: List[Array[Option[Any]]] = getAdjacentAttributesSortingIndexes(entity_id_in, startingPointSortingIndexIn, Some(1), forwardNotBackIn = forwardNotBackIn);
                if results.isEmpty) {
                  None
                } else {
                  if results.size > 1) throw new OmDatabaseException("Probably the caller didn't expect this to get >1 results...Is that even meaningful?")
                  else results.head(0).asInstanceOf[Option<i64>]
                }
              }

              // 2nd parm is 0-based index to start with, 3rd parm is # of obj's to return (if < 1 then it means "all"):
                fn getGroupEntryObjects(group_id_in: i64, startingObjectIndexIn: i64, maxValsIn: Option<i64> = None) -> Vec<Entity> {
                // see comments in getGroupEntriesData
                let sql = "select entity_id, sorting_index from entity e, EntitiesInAGroup eiag where e.id=eiag.entity_id" +;
                          (if !include_archived_entities) {
                            " and (not e.archived) "
                          } else {
                            ""
                          }) +
                          " and eiag.group_id=" + group_id_in +
                          " order by eiag.sorting_index, eiag.entity_id limit " + checkIfShouldBeAllResults(maxValsIn) + " offset " + startingObjectIndexIn
                let earlyResults = db_query(sql, "i64,i64");
                let final_results = new Vec<Entity>;
                // idea: should the remainder of this method be moved to Entity, so the persistence layer doesn't know anything about the Model? (helps avoid circular
                // dependencies; is a cleaner design. Or, maybe this class and all the object classes like Entity, etc, are all part of the same layer.) And
                // doing similarly elsewhere such as in getOmInstanceData().
                for (result <- earlyResults) {
                  // None of these values should be of "None" type, so not checking for that. If they are it's a bug:
                  final_results.add(new Entity(this, result(0).get.asInstanceOf[i64]))
                }
                require(final_results.size == earlyResults.size)
                final_results
              }

     */
              fn get_entity_data(&self, id_in: i64) -> Result<Vec<DataType>, String> {
                   self.db_query_wrapper_for_one_row(format!("SELECT name, class_id, insertion_date, public, archived, new_entries_stick_to_top from Entity where id={}", id_in),
                                         Util::GET_ENTITY_DATA__RESULT_TYPES)
              }

              fn get_entity_name(&self, id_in: i64) -> Result<Option<String>, String> {
                  let name: Vec<DataType> = self.get_entity_data(id_in)?;
                  match name.get(0) {
                      None => Ok(None),
                      Some(DataType::String(x)) => Ok(Some(x.to_string())),
                      _ => Err(format!("Unexpected value: {:?}", name)),
                  }
              }

    /*
              fn getClassData(id_in: i64) -> Array[Option[Any]] {
                db_query_wrapper_for_one_row("SELECT name, defining_entity_id, create_default_attributes from class where id=" + id_in, Database.GET_CLASS_DATA__RESULT_TYPES)
              }

                fn getClassName(id_in: i64) -> Option<String> {
                let name: Option[Any] = getClassData(id_in)(0);
                if name.isEmpty) None
                else name.asInstanceOf[Option<String>]
              }

              /**
               * @return the create_default_attributes boolean value from a given class.
               */
                fn updateClassCreateDefaultAttributes(class_id_in: i64, value: Option<bool>) {
                self.db_action(format!("update class set (create_default_attributes) = ROW(" +
                         (if value.isEmpty) "NULL" else if value.get) "true" else "false") +
                         ") where id=" + class_id_in).as_str(), false, false);
              }

                fn getTextEditorCommand -> String {
                let system_entity_id = get_system_entity_id;
                let hasRelationTypeId: i64 = find_relation_type(Database.THE_HAS_RELATION_TYPE_NAME, Some(1)).get(0);
                let editorInfoSystemEntity: Entity = getEntitiesFromRelationsToLocalEntity(system_entity_id, Database.EDITOR_INFO_ENTITY_NAME,;
                                                                                      Some(hasRelationTypeId), Some(1))(0)
                let textEditorInfoSystemEntity: Entity = getEntitiesFromRelationsToLocalEntity(editorInfoSystemEntity.get_id,;
                                                                                          Database.TEXT_EDITOR_INFO_ENTITY_NAME, Some(hasRelationTypeId),
                                                                                          Some(1))(0)
                let textEditorCommandNameAttrType: Entity = getEntitiesFromRelationsToLocalEntity(textEditorInfoSystemEntity.get_id,;
                                                                                     Database.TEXT_EDITOR_COMMAND_ATTRIBUTE_TYPE_NAME, Some(hasRelationTypeId),
                                                                                     Some(1))(0)
                let ta: TextAttribute = getTextAttributeByTypeId(textEditorInfoSystemEntity.get_id, textEditorCommandNameAttrType.get_id, Some(1)).get(0);
                ta.getText
              }

                fn getEntitiesFromRelationsToLocalEntity(parentEntityIdIn: i64, name_in: String, relTypeIdIn: Option<i64> = None,
                                                 expected_rows: Option[Int] = None) -> Array[Entity] {
                // (not getting all the attributes in this case, and doing another query to the entity table (less efficient), to save programming
                // time for the case that the entity table changes, we don't have to carefully update all the columns selected here & the mappings.  This is a more
                // likely change than for the TextAttribute table, below.
                let queryResults: List[Array[Option[Any]]] = db_query("select id from entity where name='" + name_in + "' and id in " +;
                                                                 "(select entity_id_2 from relationToEntity where entity_id=" + parentEntityIdIn +
                                                                (if relTypeIdIn.is_some()) " and rel_type_id=" + relTypeIdIn.get + " " else "") + ")",
                                                                "i64")
                if expected_rows.is_some()) {
                  let count = queryResults.size;
                  if count != expected_rows.get) throw new OmDatabaseException("Found " + count + " rows instead of expected " + expected_rows.get)
                }
                let final_result = new Array[Entity](queryResults.size);
                let mut index = 0;
                for (r <- queryResults) {
                  let id: i64 = r(0).get.asInstanceOf[i64];
                  final_result(index) = new Entity(this, id)
                  index += 1
                }
                final_result
              }

                fn getTextAttributeByTypeId(parentEntityIdIn: i64, typeIdIn: i64, expected_rows: Option[Int] = None) -> ArrayList[TextAttribute] {
                let sql = "select ta.id, ta.textValue, ta.attr_type_id, ta.valid_on_date, ta.observation_date, asort.sorting_index " +;
                          " from textattribute ta, AttributeSorting asort where ta.entity_id=" + parentEntityIdIn + " and ta.attr_type_id="+typeIdIn +
                          " and ta.entity_id=asort.entity_id and asort.attribute_form_id=" + Database.get_attribute_form_id(Util.TEXT_TYPE) +
                          " and ta.id=asort.attribute_id"
                let queryResults: List[Array[Option[Any]]] = db_query(sql, "i64,String,i64,i64,i64,i64");
                if expected_rows.is_some()) {
                  let count = queryResults.size;
                  if count != expected_rows.get) throw new OmDatabaseException("Found " + count + " rows instead of expected " + expected_rows.get)
                }
                let final_result = new ArrayList[TextAttribute](queryResults.size);
                for (r <- queryResults) {
                  let textAttributeId: i64 = r(0).get.asInstanceOf[i64];
                  let textValue: String = r(1).get.asInstanceOf[String];
                  let attrTypeId: i64 = r(2).get.asInstanceOf[i64];
                  let valid_on_date: Option<i64> = if r(3).isEmpty) None else Some(r(3).get.asInstanceOf[i64]);
                  let observationDate: i64 = r(4).get.asInstanceOf[i64];
                  let sorting_index: i64 = r(5).get.asInstanceOf[i64];
                  final_result.add(new TextAttribute(this, textAttributeId, parentEntityIdIn, attrTypeId, textValue, valid_on_date, observationDate, sorting_index))
                }
                final_result
              }

              /** Returns an array of tuples, each of which is of (sorting_index, Attribute), and a i64 indicating the total # that could be returned with
                * infinite display space (total existing).
                *
                * The parameter maxValsIn can be 0 for 'all'.
                *
                * Idea to improve efficiency: make this able to query only those attributes needed to satisfy the maxValsIn parameter (by first checking
                * the AttributeSorting table).  In other words, no need to read all 1500 attributes to display on the screen, just to know which ones come first, if
                * only 10 can be displayed right now and the rest might not need to be displayed.  Because right now, we have to query all data from the AttributeSorting
                * table, then all attributes (since remember they might not *be* in the AttributeSorting table), then sort them with the best available information,
                * then decide which ones to return.  Maybe instead we could do that smartly, on just the needed subset.  But it still need to gracefully handle it
                * when a given attribute (or all) is not found in the sorting table.
                */
                fn getSortedAttributes(entity_id_in: i64, startingObjectIndexIn: Int = 0, maxValsIn: Int = 0,
                                      onlyPublicEntitiesIn: bool = true): (Array[(i64, Attribute)], Int) {
                let allResults: java.util.ArrayList[(Option<i64>, Attribute)] = new java.util.ArrayList[(Option<i64>, Attribute)];
                // First select the counts from each table, keep a running total so we know when to select attributes (compared to inStartingObjectIndex)
                // and when to stop.
                let tables: Vec<String> = Array(Util.QUANTITY_TYPE, Util.BOOLEAN_TYPE, Util.DATE_TYPE, Util.TEXT_TYPE, Util.FILE_TYPE, Util.RELATION_TO_LOCAL_ENTITY_TYPE,;
                                                  Util.RELATION_TO_GROUP_TYPE, Util.RELATION_TO_REMOTE_ENTITY_TYPE)
                let columnsSelectedByTable: Vec<String> = Array("id,entity_id,attr_type_id,unit_id,quantity_number,valid_on_date,observation_date",;
                                                                  "id,entity_id,attr_type_id,booleanValue,valid_on_date,observation_date",
                                                                  "id,entity_id,attr_type_id,date",
                                                                  "id,entity_id,attr_type_id,textValue,valid_on_date,observation_date",

                                                                  "id,entity_id,attr_type_id,description,original_file_date,stored_date,original_file_path,readable," +
                                                                  "writable,executable,size,md5hash",

                                                                  "id,rel_type_id,entity_id,entity_id_2,valid_on_date,observation_date",
                                                                  "id,entity_id,rel_type_id,group_id,valid_on_date,observation_date",
                                                                  "id,rel_type_id,entity_id,remote_instance_id,entity_id_2,valid_on_date,observation_date")
                let typesByTable: Vec<String> = Array("i64,i64,i64,i64,i64,Float,i64,i64",;
                                                        "i64,i64,i64,i64,Boolean,i64,i64",
                                                        "i64,i64,i64,i64,i64",
                                                        "i64,i64,i64,i64,String,i64,i64",
                                                        "i64,i64,i64,i64,String,i64,i64,String,Boolean,Boolean,Boolean,i64,String",
                                                        "i64,i64,i64,i64,i64,i64,i64",
                                                        "i64,i64,i64,i64,i64,i64,i64",
                                                        "i64,i64,i64,i64,String,i64,i64,i64")
                let whereClausesByTable: Vec<String> = Array(tables(0) + ".entity_id=" + entity_id_in, tables(1) + ".entity_id=" + entity_id_in,;
                                                               tables(2) + ".entity_id=" + entity_id_in, tables(3) + ".entity_id=" + entity_id_in,
                                                               tables(4) + ".entity_id=" + entity_id_in, tables(5) + ".entity_id=" + entity_id_in,
                                                               tables(6) + ".entity_id=" + entity_id_in, tables(7) + ".entity_id=" + entity_id_in)
                let orderByClausesByTable: Vec<String> = Array("id", "id", "id", "id", "id", "entity_id", "group_id", "entity_id");

                // *******************************************
                //****** NOTE **********: some logic here for counting & looping has been commented out because it is not yet updated to work with the sorting of
                // attributes on an entity.  But it is left here because it was so carefully debugged, once, and seems likely to be used again if we want to limit the
                // data queried and sorted to that amount which can be displayed at a given time.  For example,
                // we could query first from the AttributeSorting table, then based on that decide for which ones to get all the data. But maybe for now there's a small
                // enough amount of data that we can query all rows all the time.
                // *******************************************

                // first just get a total row count for UI convenience later (to show how many left not viewed yet)
                // ABOUT THESE COMMENTED LINES: SEE "** NOTE **" ABOVE:
            //    let mut totalRowsAvailable: i64 = 0;
            //    let mut tableIndexForrow_counting = 0;
            //    while ((maxValsIn == 0 || totalRowsAvailable <= maxValsIn) && tableIndexForrow_counting < tables.length) {
            //      let table_name = tables(tableIndexForrow_counting);
            //      totalRowsAvailable += extract_row_count_from_count_query("select count(*) from " + table_name + " where " + whereClausesByTable(tableIndexForrow_counting))
            //      tableIndexForrow_counting += 1
            //    }

                // idea: this could change to a let and be filled w/ a recursive helper method; other vars might go away then too.;
                let mut tableListIndex: i32 = 0;

                // ABOUT THESE COMMENTED LINES: SEE "** NOTE **" ABOVE:
                //keeps track of where we are in getting rows >= inStartingObjectIndex and <= maxValsIn
                //    let mut counter: i64 = 0;
                //    while ((maxValsIn == 0 || counter - inStartingObjectIndex <= maxValsIn) && tableListIndex < tables.length) {
                while (tableListIndex < tables.length) {
                  let table_name = tables(tableListIndex);
                  // ABOUT THESE COMMENTED LINES: SEE "** NOTE **" ABOVE:
                  //val thisTablesrow_count: i64 = extract_row_count_from_count_query("select count(*) from " + table_name + " where " + whereClausesByTable(tableListIndex))
                  //if thisTablesrow_count > 0 && counter + thisTablesrow_count >= inStartingObjectIndex) {
                  //try {

                      // Idea: could speed this query up in part? by doing on each query something like:
                      //       limit maxValsIn+" offset "+ inStartingObjectIndex-counter;
                      // ..and then incrementing the counters appropriately.
                      // Idea: could do the sorting (currently done just before the end of this method) in sql? would have to combine all queries to all tables, though.
                      let key = whereClausesByTable(tableListIndex).substring(0, whereClausesByTable(tableListIndex).indexOf("="));
                      let columns = table_name + "." + columnsSelectedByTable(tableListIndex).replace(",", "," + table_name + ".");
                      let mut sql: String = "select attributesorting.sorting_index, " + columns +;
                                        " from " +
                                        // idea: is the RIGHT JOIN really needed, or can it be a normal join? ie, given tables' setup can there really be
                                        // rows of any Attribute (or RelationTo*) table without a corresponding attributesorting row?  Going to assume not,
                                        // for some changes below adding the sortingindex parameter to the Attribute constructors, for now at least until this is studied
                                        // again.  Maybe it had to do with the earlier unreliability of always deleting rows from attributesorting when Attributes were
                                        // deleted (and in fact an attributesorting can in theory still be created without an Attribute row, and maybe other such problems).
                                        "   attributesorting RIGHT JOIN " + table_name +
                                        "     ON (attributesorting.attribute_form_id=" + Database.get_attribute_form_id(table_name) +
                                        "     and attributesorting.attribute_id=" + table_name + ".id )" +
                                        "   JOIN entity ON entity.id=" + key +
                                        " where " +
                                        (if !include_archived_entities) {
                                          "(not entity.archived) and "
                                        } else {
                                          ""
                                        }) +
                                        whereClausesByTable(tableListIndex)
                      if table_name == Util.RELATION_TO_LOCAL_ENTITY_TYPE && !include_archived_entities) {
                        sql += " and not exists(select 1 from entity e2, relationtoentity rte2 where e2.id=rte2.entity_id_2" +
                               " and relationtoentity.entity_id_2=rte2.entity_id_2 and e2.archived)"
                      }
                      if table_name == Util.RELATION_TO_LOCAL_ENTITY_TYPE && onlyPublicEntitiesIn) {
                        sql += " and exists(select 1 from entity e2, relationtoentity rte2 where e2.id=rte2.entity_id_2" +
                               " and relationtoentity.entity_id_2=rte2.entity_id_2 and e2.public)"
                      }
                      sql += " order by " + table_name + "." + orderByClausesByTable(tableListIndex)
                      let results = db_query(sql, typesByTable(tableListIndex));
                      for (result: Array[Option[Any]] <- results) {
                        // skip past those that are outside the range to retrieve
                        //idea: use some better scala/function construct here so we don't keep looping after counter hits the max (and to make it cleaner)?
                        //idea: move it to the same layer of code that has the Attribute classes?

                        // ABOUT THESE COMMENTED LINES: SEE "** NOTE **" ABOVE:
                        // Don't get it if it's not in the requested range:
            //            if counter >= inStartingObjectIndex && (maxValsIn == 0 || counter <= inStartingObjectIndex + maxValsIn)) {
                          if table_name == Util.QUANTITY_TYPE) {
                            allResults.add((if result(0).isEmpty) None else Some(result(0).get.asInstanceOf[i64]),
                                       new QuantityAttribute(this, result(1).get.asInstanceOf[i64], result(2).get.asInstanceOf[i64], result(3).get.asInstanceOf[i64],
                                                             result(4).get.asInstanceOf[i64], result(5).get.asInstanceOf[Float],
                                                             if result(6).isEmpty) None else Some(result(6).get.asInstanceOf[i64]), result(7).get.asInstanceOf[i64],
                                                             result(0).get.asInstanceOf[i64])))
                          } else if table_name == Util.TEXT_TYPE) {
                            allResults.add((if result(0).isEmpty) None else Some(result(0).get.asInstanceOf[i64]),
                                       new TextAttribute(this, result(1).get.asInstanceOf[i64], result(2).get.asInstanceOf[i64], result(3).get.asInstanceOf[i64],
                                                         result(4).get.asInstanceOf[String], if result(5).isEmpty) None else Some(result(5).get.asInstanceOf[i64]),
                                                         result(6).get.asInstanceOf[i64], result(0).get.asInstanceOf[i64])))
                          } else if table_name == Util.DATE_TYPE) {
                            allResults.add((if result(0).isEmpty) None else Some(result(0).get.asInstanceOf[i64]),
                                       new DateAttribute(this, result(1).get.asInstanceOf[i64], result(2).get.asInstanceOf[i64], result(3).get.asInstanceOf[i64],
                                                         result(4).get.asInstanceOf[i64], result(0).get.asInstanceOf[i64])))
                          } else if table_name == Util.BOOLEAN_TYPE) {
                            allResults.add((if result(0).isEmpty) None else Some(result(0).get.asInstanceOf[i64]),
                                       new BooleanAttribute(this, result(1).get.asInstanceOf[i64], result(2).get.asInstanceOf[i64], result(3).get.asInstanceOf[i64],
                                                            result(4).get.asInstanceOf[Boolean], if result(5).isEmpty) None else Some(result(5).get.asInstanceOf[i64]),
                                                            result(6).get.asInstanceOf[i64], result(0).get.asInstanceOf[i64])))
                          } else if table_name == Util.FILE_TYPE) {
                            allResults.add((if result(0).isEmpty) None else Some(result(0).get.asInstanceOf[i64]),
                                       new FileAttribute(this, result(1).get.asInstanceOf[i64], result(2).get.asInstanceOf[i64], result(3).get.asInstanceOf[i64],
                                                         result(4).get.asInstanceOf[String], result(5).get.asInstanceOf[i64], result(6).get.asInstanceOf[i64],
                                                         result(7).get.asInstanceOf[String], result(8).get.asInstanceOf[Boolean], result(9).get.asInstanceOf[Boolean],
                                                         result(10).get.asInstanceOf[Boolean], result(11).get.asInstanceOf[i64], result(12).get.asInstanceOf[String],
                                                         result(0).get.asInstanceOf[i64])))
                          } else if table_name == Util.RELATION_TO_LOCAL_ENTITY_TYPE) {
                            allResults.add((if result(0).isEmpty) None else Some(result(0).get.asInstanceOf[i64]),
                                       new RelationToLocalEntity(this, result(1).get.asInstanceOf[i64], result(2).get.asInstanceOf[i64], result(3).get.asInstanceOf[i64],
                                                            result(4).get.asInstanceOf[i64],
                                                            if result(5).isEmpty) None else Some(result(5).get.asInstanceOf[i64]), result(6).get.asInstanceOf[i64],
                                                            result(0).get.asInstanceOf[i64])))
                          } else if table_name == Util.RELATION_TO_GROUP_TYPE) {
                            allResults.add((if result(0).isEmpty) None else Some(result(0).get.asInstanceOf[i64]),
                                       new RelationToGroup(this, result(1).get.asInstanceOf[i64], result(2).get.asInstanceOf[i64], result(3).get.asInstanceOf[i64],
                                                           result(4).get.asInstanceOf[i64],
                                                           if result(5).isEmpty) None else Some(result(5).get.asInstanceOf[i64]),
                                                           result(6).get.asInstanceOf[i64], result(0).get.asInstanceOf[i64])))
                          } else if table_name == Util.RELATION_TO_REMOTE_ENTITY_TYPE) {
                            allResults.add((if result(0).isEmpty) None else Some(result(0).get.asInstanceOf[i64]),
                                             new RelationToRemoteEntity(this, result(1).get.asInstanceOf[i64], result(2).get.asInstanceOf[i64],
                                                                        result(3).get.asInstanceOf[i64],
                                                                        result(4).get.asInstanceOf[String], result(5).get.asInstanceOf[i64],
                                                                        if result(6).isEmpty) None else Some(result(6).get.asInstanceOf[i64]),
                                                                        result(7).get.asInstanceOf[i64],
                                                                  result(0).get.asInstanceOf[i64])))
                          } else throw new OmDatabaseException("invalid table type?: '" + table_name + "'")

                        // ABOUT THESE COMMENTED LINES: SEE "** NOTE **" ABOVE:
                        //}
            //            counter += 1
                      }

                  // ABOUT THESE COMMENTED LINES: SEE "** NOTE **" ABOVE:
                    //}
                    //remove the try permanently, or, what should be here as a 'catch'? how interacts w/ 'throw' or anything related just above?
                  //} else {
                  //  counter += thisTablesrow_count
                  //}
                  tableListIndex += 1
                }

                let allResultsArray: Array[(i64, Attribute)] = new Array[(i64, Attribute)](allResults.size);
                let mut index = -1;
                for (element: (Option<i64>, Attribute) <- allResults.toArray(new Array[(Option<i64>, Attribute)](0))) {
                  index += 1
                  // using max_id_value as the max value of a long so those w/o sorting information will just sort last:
                  allResultsArray(index) = (element._1.getOrElse(self.max_id_value()), element._2)
                }
                // Per the scalaDocs for scala.math.Ordering, this sorts by the first element of the tuple (ie, .z_1) which at this point is attributesorting.sorting_index.
                // (The "getOrElse" on next line is to allow for the absence of a value in case the attributeSorting table doesn't have an entry for some attributes.
                Sorting.quickSort(allResultsArray)(Ordering[i64].on(x => x._1.asInstanceOf[i64]))

                let from: i32 = startingObjectIndexIn;
                let numVals: i32 = if maxValsIn > 0) maxValsIn else allResultsArray.length;
                let until: i32 = Math.min(startingObjectIndexIn + numVals, allResultsArray.length);
                (allResultsArray.slice(from, until), allResultsArray.length)
              }

              /// The 2nd parameter is to avoid saying an entity is a duplicate of itself: checks for all others only.
                fn isDuplicateEntityName(name_in: String, selfIdToIgnoreIn: Option<i64> = None) -> bool {
                let first = isDuplicateRow(name_in, Util.ENTITY_TYPE, "id", "name",;
                                           if !include_archived_entities) {
                                             Some("(not archived)")
                                           } else {
                                             None
                                           },
                                           selfIdToIgnoreIn)
                let second = isDuplicateRow(name_in, Util.RELATION_TYPE_TYPE, "entity_id", "name_in_reverse_direction", None, selfIdToIgnoreIn);
                first || second
              }

              /// The inSelfIdToIgnore parameter is to avoid saying a class is a duplicate of itself: checks for all others only.
                fn isDuplicateRow[T](possibleDuplicateIn: String, table: String, keyColumnToIgnoreOn: String, columnToCheckForDupValues: String, extraCondition: Option<String>,
                                 selfIdToIgnoreIn: Option[T] = None) -> bool {
                let valueToCheck: String = self.escape_quotes_etc(possibleDuplicateIn);

                let exception: String =;
                  if selfIdToIgnoreIn.isEmpty) {
                    ""
                  } else {
                    "and not " + keyColumnToIgnoreOn + "=" + selfIdToIgnoreIn.get.toString
                  }

                does_this_exist("SELECT count(" + keyColumnToIgnoreOn + ") from " + table + " where " +
                              (if extraCondition.is_some() && extraCondition.get.nonEmpty) extraCondition.get else "true") +
                              " and lower(" + columnToCheckForDupValues + ")=lower('" + valueToCheck + "') " + exception,
                              fail_if_more_than_one_found = false)
              }


              /** The 2nd parameter is to avoid saying a class is a duplicate of itself: checks for all others only. */
                fn isDuplicateClassName(name_in: String, selfIdToIgnoreIn: Option<i64> = None) -> bool {
                isDuplicateRow[i64](name_in, "class", "id", "name", None, selfIdToIgnoreIn)
              }

              /// The 2nd parameter is to avoid saying an instance is a duplicate of itself: checks for all others only.
                fn isDuplicateOmInstanceAddress(address_in: String, selfIdToIgnoreIn: Option<String> = None) -> bool {
                isDuplicateRow[String](address_in, "omInstance", "id", "address", None,
                                       if selfIdToIgnoreIn.isEmpty) None else Some("'" + selfIdToIgnoreIn.get + "'"))
              }
              protected override fn finalize() {
                super.finalize()
                if connection != null) connection.close()
              }


              /** Cloned from delete_objects: CONSIDER UPDATING BOTH if updating one.
                */
                fn archiveObjects(table_name_in: String, where_clause_in: String, rows_expected: i64 = 1, caller_manages_transactions_in: bool = false,
                                         unarchive: bool = false) {
                //idea: enhance this to also check & return the # of rows deleted, to the caller to just make sure? If so would have to let caller handle transactions.
                      //%%$%%%%%%%FIX NEXT LINE AFTERI SEE HOW OTHERS DO!
                // if !caller_manages_transactions_in) self.begin_trans()
                try {
                  let archive = if unarchive) "false" else "true";
                  let archivedDate = if unarchive) {;
                    "NULL"
                  } else {
                    "" + System.currentTimeMillis()
                  }
                  let rows_affected = self.db_action(format!("update " + table_name_in + " set (archived, archived_date) = (" + archive + ", " + archivedDate + ") " + where_clause_in).as_str(), false, false);
                  if rows_expected >= 0 && rows_affected != rows_expected) {
                    // Roll back, as we definitely don't want to affect an unexpected # of rows.
                    // Do it ***EVEN THOUGH callerManagesTransaction IS true***: seems cleaner/safer this way.
                    throw rollbackWithCatch(new OmDatabaseException("Archive command would have updated " + rows_affected + "rows, but " +
                                                          rows_expected + " were expected! Did not perform archive."))
                  } else {
                      //%%$%%%%%%%FIX NEXT LINE AFTERI SEE HOW OTHERS DO!
                    // if !caller_manages_transactions_in) commit_trans()
                  }
                } catch {
                  case e: Exception => throw rollbackWithCatch(e)
                }
              }

                fn deleteObjectById(table_name_in: String, id_in: i64, caller_manages_transactions_in: bool = false) /* -> Unit%%*/ {
                delete_objects(table_name_in, "where id=" + id_in, caller_manages_transactions_in = caller_manages_transactions_in)
              }

                fn deleteObjectById2(table_name_in: String, id_in: String, caller_manages_transactions_in: bool = false) /* -> Unit%%*/ {
                delete_objects(table_name_in, "where id='" + id_in + "'", caller_manages_transactions_in = caller_manages_transactions_in)
              }

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
                                               quoteIn: Option<String> = None) -> (Entity, RelationToLocalEntity) {
                if quoteIn.is_some()) require(!quoteIn.get.isEmpty, "It doesn't make sense to store a blank quotation; there was probably a program error.")
                      //%%$%%%%%%%FIX NEXT LINE AFTERI SEE HOW OTHERS DO!
                // if !caller_manages_transactions_in { self.begin_trans() }
                try {
                  // **idea: BAD SMELL: should this method be moved out of the db class, since it depends on higher-layer components, like EntityClass and
                  // those in the same package? It was in Controller, but moved here
                  // because it seemed like things that manage transactions should be in the db layer.  So maybe it needs un-mixing of layers.

                  let (uriClassId: i64, uriClassTemplateId: i64) = getOrCreateClassAndTemplateEntity("URI", caller_manages_transactions_in);
                  let (_, quotationClassTemplateId: i64) = getOrCreateClassAndTemplateEntity("quote", caller_manages_transactions_in);
                  let (newEntity: Entity, newRTLE: RelationToLocalEntity) = containingEntityIn.create_entityAndAddHASLocalRelationToIt(new_entity_name_in, observation_date_in,;
                                                                                                                           makeThem_publicIn, caller_manages_transactions_in)
                  updateEntitysClass(newEntity.get_id, Some(uriClassId), caller_manages_transactions_in)
                  newEntity.addTextAttribute(uriClassTemplateId, uriIn, None, None, observation_date_in, caller_manages_transactions_in)
                  if quoteIn.is_some()) {
                    newEntity.addTextAttribute(quotationClassTemplateId, quoteIn.get, None, None, observation_date_in, caller_manages_transactions_in)
                  }
                      //%%$%%%%%%%FIX NEXT LINE AFTERI SEE HOW OTHERS DO!
                  // if !caller_manages_transactions_in {self.commit_trans() }
                  (newEntity, newRTLE)
                } catch {
                  case e: Exception =>
                      //%%$%%%%%%%FIX NEXT LINE AFTERI SEE HOW OTHERS DO!
                    // if !caller_manages_transactions_in) rollback_trans()
                    throw e
                }
              }

                fn getOrCreateClassAndTemplateEntity(class_name_in: String, caller_manages_transactions_in: bool) -> (i64, i64) {
                //(see note above re 'bad smell' in method addUriEntityWithUriAttribute.)
                      //%%$%%%%%%%FIX NEXT LINE AFTERI SEE HOW OTHERS DO!
                // if !caller_manages_transactions_in { self.begin_trans() }
                try {
                  let (class_id, entity_id) = {;
                    let foundId = findFIRSTClassIdByName(class_name_in, caseSensitive = true);
                    if foundId.is_some()) {
                      let entity_id: i64 = new EntityClass(this, foundId.get).getTemplateEntityId;
                      (foundId.get, entity_id)
                    } else {
                      let (class_id: i64, entity_id: i64) = create_class_and_its_template_entity(class_name_in);
                      (class_id, entity_id)
                    }
                  }
                      //%%$%%%%%%%FIX NEXT LINE AFTERI SEE HOW OTHERS DO!
                  // if !caller_manages_transactions_in {self.commit_trans() }
                  (class_id, entity_id)
                }
                catch {
                  case e: Exception =>
                      //%%$%%%%%%%FIX NEXT LINE AFTERI SEE HOW OTHERS DO!
                    // if !caller_manages_transactions_in) rollback_trans()
                    throw e
                }
              }
              fn set_include_archived_entities(in: bool) /* -> Unit%%*/ {
                include_archived_entities = in
              }

                fn getOmInstanceCount() -> i64 {
                extract_row_count_from_count_query("SELECT count(1) from omInstance")
              }
*/
*/

              fn create_om_instance(&self, id_in: String, is_local_in: bool, address_in: String, entity_id_in: Option<i64>/*%% = None*/,
                                   old_table_name: bool/*%% = false*/) -> Result<i64, String> {
                if id_in.len() == 0 { return Err("ID must have a value.".to_string()); }
                if address_in.len() == 0 { return Err("Address must have a value.".to_string()); }
                let id: String = Self::escape_quotes_etc(id_in.clone());
                let address: String = Self::escape_quotes_etc(address_in.clone());
                if id != id_in { return Err(format!("Didn't expect quotes etc in the UUID provided: {}", id_in)) };
                if address != address_in { return Err(format!("Didn't expect quotes etc in the address provided: {}", address)); }
                let insertion_date: i64 = Utc::now().timestamp_millis();
                // next line is for the method upgradeDbFrom3to4 so it can work before upgrading 4to5:
                let table_name: &str = if old_table_name {"om_instance"} else {"omInstance"};
                let is_local = if is_local_in {"TRUE"} else {"FALSE"};
                let maybe_entity_id_value = match entity_id_in {
                    None => "NULL".to_string(),
                    Some(id) => id.to_string(),
                };
                let sql: String = format!("INSERT INTO {table_name} (id, local, address, insertion_date, entity_id) \
                                  VALUES ('{id}',{is_local},'{address}',{insertion_date},\
                                  {maybe_entity_id_value})");
                self.db_action(sql.as_str(), false, false)?;
                Ok(insertion_date)
              }

     /*
                fn getOmInstanceData(id_in: String) -> Array[Option[Any]] {
                let row: Array[Option[Any]] = db_query_wrapper_for_one_row("SELECT local, address, insertion_date, entity_id from omInstance" +;
                                                                      " where id='" + id_in + "'", Database.GET_OM_INSTANCE_DATA__RESULT_TYPES)
                row
              }

              lazy let id: String = {;
                getLocalOmInstanceData.get_id
              }

              /// @return the OmInstance object that stands for *this*: the OmInstance to which this PostgreSQLDatabase class instance reads/writes directly.
                fn getLocalOmInstanceData -> OmInstance {
                let sql = "SELECT id, address, insertion_date, entity_id from omInstance where local=TRUE";
                let results = db_query(sql, "String,String,i64,i64");
                if results.size != 1) throw new OmDatabaseException("Got " + results.size + " instead of 1 result from sql " + sql +
                                                                     ".  Does the usage now warrant removing this check (ie, multiple locals stored)?")
                let result = results.head;
                new OmInstance(this, result(0).get.asInstanceOf[String], is_local_in = true,
                               result(1).get.asInstanceOf[String],
                               result(2).get.asInstanceOf[i64], if result(3).isEmpty) None else Some(result(3).get.asInstanceOf[i64]))
              }

                fn omInstanceKeyExists(id_in: String) -> bool {
                does_this_exist("SELECT count(1) from omInstance where id='" + id_in + "'")
              }
    */

    //%%$%%
    /*
                fn getOmInstances(localIn: Option<bool> = None) -> java.util.ArrayList[OmInstance] {
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
                let earlyResults = db_query(sql, "String,Boolean,String,i64,i64");
                let final_results = new java.util.ArrayList[OmInstance];
                // (Idea: See note in similar point in getGroupEntryObjects.)
                for (result <- earlyResults) {
                  final_results.add(new OmInstance(this, result(0).get.asInstanceOf[String], is_local_in = result(1).get.asInstanceOf[Boolean],
                                                  result(2).get.asInstanceOf[String],
                                                  result(3).get.asInstanceOf[i64], if result(4).isEmpty) None else Some(result(4).get.asInstanceOf[i64])))
                }
                require(final_results.size == earlyResults.size)
                if localIn.is_some() && localIn.get && final_results.size == 0) {
                  let total = getOmInstanceCount;
                  throw new OmDatabaseException("Unexpected: the # of rows omInstance where local=TRUE is 0, and there should always be at least one." +
                                                "(See insert at end of create_base_data and upgradeDbFrom3to4.)  Total # of rows: " + total)
                }
                final_results
              }

      "getLocalOmInstanceData and friends" should "work" in {
        let oi: OmInstance = m_db.getLocalOmInstanceData;
        let uuid: String = oi.get_id;
        assert(oi.getLocal)
        assert(m_db.omInstanceKeyExists(uuid))
        let startingOmiCount = m_db.getOmInstanceCount;
        assert(startingOmiCount > 0)
        let oiAgainAddress = m_db.getOmInstanceData(uuid)(1).get.asInstanceOf[String];
        assert(oiAgainAddress == Util.LOCAL_OM_INSTANCE_DEFAULT_DESCRIPTION)
        let omInstances: util.ArrayList[OmInstance] = m_db.getOmInstances();
        assert(omInstances.size == startingOmiCount)
        let sizeNowTrue = m_db.getOmInstances(Some(true)).size;
        assert(sizeNowTrue > 0)
        // Idea: fix: Next line fails at times, maybe due to code running in parallel between this and RestDatabaseTest, creating/deleting rows.  Only seems to happen
        // when all tests are run, never when the test classes are run separately.
        //    let sizeNowFalse = m_db.getOmInstances(Some(false)).size;
        //assert(sizeNowFalse < sizeNowTrue)
        assert(! m_db.omInstanceKeyExists(java.util.UUID.randomUUID().toString))
        assert(new OmInstance(m_db, uuid).getAddress == Util.LOCAL_OM_INSTANCE_DEFAULT_DESCRIPTION)

        let uuid2 = java.util.UUID.randomUUID().toString;
        m_db.create_om_instance(uuid2, is_local_in = false, "om.example.com", Some(m_db.get_system_entity_id))
        // should have the local one created at db creation, and now the one for this test:
        assert(m_db.getOmInstanceCount == startingOmiCount + 1)
        let mut i2: OmInstance = new OmInstance(m_db, uuid2);
        assert(i2.getAddress == "om.example.com")
        m_db.updateOmInstance(uuid2, "address", None)
        i2  = new OmInstance(m_db,uuid2)
        assert(i2.getAddress == "address")
        assert(!i2.getLocal)
        assert(i2.getEntityId.isEmpty)
        assert(i2.getCreationDate > 0)
        assert(i2.getCreationDateFormatted.length > 0)
        m_db.updateOmInstance(uuid2, "address", Some(m_db.get_system_entity_id))
        i2  = new OmInstance(m_db,uuid2)
        assert(i2.getEntityId.get == m_db.get_system_entity_id)
        assert(m_db.isDuplicateOmInstanceAddress("address"))
        assert(m_db.isDuplicateOmInstanceAddress(Util.LOCAL_OM_INSTANCE_DEFAULT_DESCRIPTION))
        assert(!m_db.isDuplicateOmInstanceAddress("address", Some(uuid2)))
        assert(!m_db.isDuplicateOmInstanceAddress(Util.LOCAL_OM_INSTANCE_DEFAULT_DESCRIPTION, Some(uuid)))
        let uuid3 = java.util.UUID.randomUUID().toString;
        m_db.create_om_instance(uuid3, is_local_in = false, "address", Some(m_db.get_system_entity_id))
        assert(m_db.isDuplicateOmInstanceAddress("address", Some(uuid2)))
        assert(m_db.isDuplicateOmInstanceAddress("address", Some(uuid3)))
        i2.delete()
        assert(m_db.isDuplicateOmInstanceAddress("address"))
        assert(m_db.isDuplicateOmInstanceAddress("address", Some(uuid2)))
        assert(!m_db.isDuplicateOmInstanceAddress("address", Some(uuid3)))
        assert(intercept[Exception] {
                                      new OmInstance(m_db, uuid2)
                                    }.getMessage.contains("does not exist"))
      }
    */

    /*
        fn updateOmInstance(id_in: String, address_in: String, entity_id_in: Option<i64>) {
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

        fn deleteOmInstance(id_in: String) /* -> Unit%%*/ {
        deleteObjectById2("omInstance", id_in)
      }

    */
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_set_user_preference_and_get_user_preference() {
        let db: PostgreSQLDatabase = Util::initialize_test_db().unwrap();
        assert!(false);
        //%%$%%%%
        //fix red things, see other markers, can make it compile then (ckin and?) reformat?
        //then step thru w/ a debugger?
        //then cont here/below?
        /*
        assert(m_db.get_user_preference_boolean("xyznevercreatemeinreallife").isEmpty)
        // (intentional style violation for readability - the ".contains" suggested by the IDE just caused another problem)
        //noinspection OptionEqualsSome
        assert(m_db.get_user_preference_boolean("xyznevercreatemeinreallife", Some(true)) == Some(true))
        m_db.set_user_preference_boolean("xyznevercreatemeinreallife", value_in = false)
        //noinspection OptionEqualsSome
        assert(m_db.get_user_preference_boolean("xyznevercreatemeinreallife", Some(true)) == Some(false))

        assert(m_db.getUserPreference_EntityId("xyz2").isEmpty)
        // (intentional style violation for readability - the ".contains" suggested by the IDE just caused another problem)
        //noinspection OptionEqualsSome
        assert(m_db.getUserPreference_EntityId("xyz2", Some(0L)) == Some(0L))
        m_db.setUserPreference_EntityId("xyz2", m_db.get_system_entity_id)
        //noinspection OptionEqualsSome
        assert(m_db.getUserPreference_EntityId("xyz2", Some(0L)) == Some(m_db.get_system_entity_id))
                 */
    }
}
