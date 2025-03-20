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
use crate::model::attribute::Attribute;
use crate::model::database::DataType;
use crate::model::database::Database;
use crate::model::entity::Entity;
use crate::model::om_instance::OmInstance;
use crate::model::postgres::postgresql_database::*;
use crate::model::relation_to_group::RelationToGroup;
use crate::model::text_attribute::TextAttribute;
// use crate::model::postgres::*;
use crate::util::Util;
use anyhow::anyhow;
use chrono::Utc;
// use futures::executor::block_on;
// use sqlx::postgres::*;
// Specifically omitting sql::Error from use statements so that it is *clearer* which Error type is
// in use, in the code.
// use sqlx::{Column, PgPool, Postgres, Row, Transaction, ValueRef};
use sqlx::{Postgres, Transaction};
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
// use std::fmt::format;
// use tracing::*;

impl PostgreSQLDatabase {
    // Moved methods that are not part of the Database trait go here
    // or in postgresql_database.rs (they are split to make smaller files,
    // for parsing speed during intellij editing).

    pub fn limit_to_entities_only(select_column_names: &str) -> String {
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
    pub fn add_attribute_sorting_row(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        entity_id_in: i64,
        attribute_form_id_in: i32,
        attribute_id_in: i64,
        sorting_index_in: Option<i64>, /*= None*/
    ) -> Result<i64, anyhow::Error> {
        // SEE COMMENTS IN SIMILAR METHOD: add_entity_to_group.  **AND DO MAINTENANCE. IN BOTH PLACES.
        // Should probably be called from inside a transaction (which isn't managed in this method, since all its current callers do it.)
        let sorting_index: i64 = {
            let index = {
                if sorting_index_in.is_some() {
                    sorting_index_in.unwrap()
                } else if self.get_attribute_count(transaction.clone(), entity_id_in, false)? == 0 {
                    // start with an increment off the min or max, so that later there is room to sort something before or after it, manually:
                    self.min_id_value() + 99999
                } else {
                    self.max_id_value() - 99999
                }
            };
            if self.is_attribute_sorting_index_in_use(transaction.clone(), entity_id_in, index)? {
                self.find_unused_attribute_sorting_index(transaction.clone(), entity_id_in, None)?
            } else {
                index
            }
        };
        self.db_action(transaction, format!("insert into AttributeSorting (entity_id, attribute_form_id, attribute_id, sorting_index) \
            values ({},{},{},{})", entity_id_in, attribute_form_id_in, attribute_id_in, sorting_index).as_str(),
                       false, false)?;
        Ok(sorting_index)
    }

    pub fn get_system_entity_id(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<i64, anyhow::Error> {
        let ids: Vec<i64> =
            self.find_entity_only_ids_by_name(transaction, Util::SYSTEM_ENTITY_NAME.to_string())?;
        if ids.is_empty() {
            return Err(anyhow!(
                "No system entity id (named \"{}\") was \
                 found in the entity table.  Did a new data import fail partway through or \
                 something?",
                Util::SYSTEM_ENTITY_NAME
            ));
        }
        assert_eq!(ids.len(), 1);
        Ok(ids[0])
    }

    // Cloned to archive_objects: CONSIDER UPDATING BOTH if updating one.  Returns the # of rows deleted.
    /// Unless the parameter rows_expected==-1, it will allow any # of rows to be deleted; otherwise if the # of rows is wrong it will abort tran & fail.
    pub fn delete_objects<'a>(
        &'a self,
        // The purpose of transaction_in is so that whenever a direct db call needs to be done in a
        // transaction, as opposed to just using the pool as Executor, it will be available.
        // And (it being None vs. Some) for those times when this method does not know the
        // context in which it will be called: whether it should rollback itself on error
        // (automatically by creating a transaction and letting it go out of scope), or should allow
        // the caller only to manage that.
        transaction_in: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        table_name_in: &str,
        where_clause_in: &str,
        rows_expected: u64, /*= 1*/
    ) -> Result<u64, anyhow::Error> {
        //BEGIN COPY/PASTED/DUPLICATED BLOCK-----------------------------------
        // Try creating a local transaction whether we use it or not, to handle compiler errors
        // about variable moves. I'm not seeing a better way to get around them by just using
        // conditions and an Option (many errors):
        // (I tried putting this in a function, then a macro, but it gets compile errors.
        // So, copy/pasting this, unfortunately, until someone thinks of a better way. (You
        // can see the macro, and one of the compile errors, in the commit of 2023-05-18.
        // I didn't try a proc macro but based on some reading I think it would have the same
        // problem.)
        let local_tx: Transaction<Postgres> = self.begin_trans()?;
        let local_tx_option = Some(Rc::new(RefCell::new(local_tx)));
        let transaction = if transaction_in.clone().is_some() {
            transaction_in.clone()
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
            transaction.clone(),
            sql.as_str(),
            /*caller_checks_row_count_etc =*/ true,
            false,
        )?;
        if rows_expected > 0 && rows_deleted != rows_expected {
            // No need to explicitly roll back a locally created transaction aka tx, though we
            // definitely don't want to delete an unexpected # of rows,
            // because rollback is implicit whenever the transaction goes out of scope without a commit.
            // Caller should roll back (or fail to commit, same thing) in case of error.
            return Err(anyhow!(
                "Delete command  have removed {} rows, but {} were expected! \
                Did not perform delete.  SQL is: \"{}\"",
                rows_deleted,
                rows_expected,
                sql
            ));
        } else {
            //%%put this & similar places into a function like self.commit_or_err(tx)?;   ?  If so, include the rollback cmt from just above?
            if transaction_in.is_none() && transaction.is_some() {
                // Using local_tx to make the compiler happy and because it is the one we need,
                // Ie, there is no transaction provided by the caller.
                let local_tx_cell: Option<RefCell<Transaction<Postgres>>> =
                    Rc::into_inner(transaction.unwrap());
                match local_tx_cell {
                    Some(t) => {
                        let unwrapped_local_tx = t.into_inner();
                        if let Err(e) = self.commit_trans(unwrapped_local_tx) {
                            return Err(anyhow!(e.to_string()));
                        }
                    }
                    None => {
                        return Err(anyhow!("Unexpectedly found None instead of Some<RefCell<Transaction<Postgres>>>. How?"));
                    }
                }
            }
            Ok(rows_deleted)
        }
    }

    pub fn get_user_preference2<'a, 'b>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'b, Postgres>>>>,
        preferences_container_id_in: i64,
        preference_name_in: &str,
        preference_type: &str,
    ) -> Result<Vec<DataType>, anyhow::Error>
    where
        'a: 'b,
    {
        // (Passing a smaller numeric parameter to find_contained_local_entity_ids for levels_remainingIn, so that in the (very rare) case where one does not
        // have a default entity set at the *top* level of the preferences under the system entity, and there are links there to entities with many links
        // to others, then it still won't take too long to traverse them all at startup when searching for the default entity.  But still allowing for
        // preferences to be nested up to that many levels (3 as of this writing).
        let mut set: HashSet<i64> = HashSet::new();
        let found_preferences: &mut HashSet<i64> = self.find_contained_local_entity_ids(
            transaction.clone(),
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
                return Err(anyhow!("Under the entity \"{}\" ({}, possibly under {}), there \
                        are (eventually) more than one entity with the name \"{}\", so the program does not know which one to use for this.",
                                   pref_container_entity_name, preferences_container_id_in, Util::SYSTEM_ENTITY_NAME, preference_name_in));
            }
            let mut preference_entity_id: i64 = 0;
            for x in found_preferences.iter() {
                // there is exactly one, as checked above
                preference_entity_id = *x;
            }
            let relevant_attribute_rows: Vec<Vec<Option<DataType>>> = {
                if preference_type == Util::PREF_TYPE_BOOLEAN {
                    // (Using the preference_entity.get_id for attr_type_id, just for convenience since it seemed as good as any.  ALSO USED IN THE SAME WAY,
                    // IN setUserPreference METHOD CALL TO create_boolean_attribute!)
                    let sql2 = format!("select id, boolean_value from booleanattribute where entity_id={} and attr_type_id={}", preference_entity_id, preference_entity_id);
                    self.db_query(transaction.clone(), sql2.as_str(), "i64,bool")?
                } else if preference_type == Util::PREF_TYPE_ENTITY_ID {
                    let sql2 = format!("select rel_type_id, entity_id, entity_id_2 from relationtoentity where entity_id={}", preference_entity_id);
                    self.db_query(transaction.clone(), sql2.as_str(), "i64,i64,i64")?
                } else {
                    return Err(anyhow!("Unexpected preference_type: {}", preference_type));
                }
            };
            if relevant_attribute_rows.len() == 0 {
                // at this point we probably have a preference entity but not the expected attribute inside it that holds the actual useful information, so the
                // user needs to go delete the bad preference entity or re-create the attribute.
                // Idea: should there be a good way to *tell* them that, from here?
                // Or, just delete the bad preference (self-cleanup). If it was the public/private display toggle, its absence will cause errors (though it is a
                // very unlikely situation here), and it will be fixed on restarting the app (or starting another instance), via the create_and_check_expected_data
                // (or current equivalent?) method.
                self.delete_entity(transaction.clone(), preference_entity_id)?;
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
                    return Err(anyhow!("Unexpected preference_type: {}", preference_type));
                };

                if relevant_attribute_rows.len() != 1 {
                    // ASSUMED it is 1, below!
                    return Err(anyhow!("Under the entity {}, there are {}{}so the program does not know what to use for this.  There should be *one*.",
                                        preference_entity_id,
                                       relevant_attribute_rows.len(), attr_msg));
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

    pub fn get_relation_to_local_entity_by_name(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        let related_entity_id_rows = self.db_query(transaction.clone(), sql.as_str(), "i64")?;
        if related_entity_id_rows.len() == 0 {
            Ok(None)
        } else {
            if related_entity_id_rows.len() != 1 {
                let containing_entity_name =
                    match self.get_entity_name(transaction, containing_entity_id_in)? {
                        None => "(None)".to_string(),
                        Some(s) => s,
                    };
                return Err(anyhow!("Under the entity {}({}), there is more one than entity with the name \"{}\", so the program does not know which one to use for this.",
                           containing_entity_name, containing_entity_id_in,
                    Util::USER_PREFERENCES));
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

    pub fn get_quantity_attribute_count(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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

    pub fn get_text_attribute_count(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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

    pub fn get_date_attribute_count(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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

    pub fn get_boolean_attribute_count(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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

    pub fn get_file_attribute_count(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
    pub fn do_database_upgrades_if_needed(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<(), anyhow::Error> {
        let version_table_exists: bool = self.does_this_exist(
            transaction.clone(),
            "select count(1) from pg_class where relname='odb_version'",
            true,
        )?;
        if !version_table_exists {
            self.create_version_table(transaction.clone())?;
        }
        let db_version_row: Vec<Option<DataType>> = self.db_query_wrapper_for_one_row(
            transaction,
            "select version from odb_version",
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
            the table odb_version (perhaps by temporarily commenting out the line with
            "UPDATE odb_version ..." from create_tables while running tests).  AND,
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
    pub fn find_first_class_id_by_name(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        name_in: &str,
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

    pub fn update_class_name(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
    pub fn delete_relation_to_group_and_all_recursively<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        group_id_in: i64,
    ) -> Result<(u64, u64), anyhow::Error> {
        let entity_ids: Vec<Vec<Option<DataType>>> = self.db_query(
            transaction.clone(),
            format!(
                "select entity_id from entitiesinagroup where group_id={}",
                group_id_in
            )
            .as_str(),
            "i64",
        )?;
        let num_e_ids: u64 = entity_ids.len().try_into()?;
        let deletions1 = self.delete_objects(
            transaction.clone(),
            "entitiesinagroup",
            format!("where group_id={}", group_id_in).as_str(),
            num_e_ids,
        )?;
        // Have to delete these 2nd because of a constraint on EntitiesInAGroup:
        // idea: is there a temp table somewhere that these could go into instead, for efficiency?
        // idea: batch these, would be much better performance.
        // idea: BUT: what is the length limit: should we do it it sets of N to not exceed sql command size limit?
        // idea: (also on task list i think but) we should not delete entities until dealing with their use as attrtypeids etc!
        for id_vec in entity_ids {
            match id_vec[0] {
                Some(DataType::Bigint(id)) => {
                    self.delete_objects(transaction.clone(), Util::ENTITY_TYPE,
                                        format!("where id={}", id).as_str(), 1)?
                },
                None => return Err(anyhow!("In delete_relation_to_group_and_all_recursively, How did we get a null entity_id back from query?")),
                _ => return Err(anyhow!("In delete_relation_to_group_and_all_recursively, How did we get {:?} back from query?", id_vec)),
            };
        }

        let deletions2 = 0;
        //and finally:
        // (passing 0 for rows expected, because there either could be some, or none if the group is not contained in any entity.)
        self.delete_objects(
            transaction.clone(),
            Util::RELATION_TO_GROUP_TYPE,
            format!("where group_id={}", group_id_in).as_str(),
            0,
        )?;
        self.delete_objects(
            transaction,
            "grupo",
            format!("where id={}", group_id_in).as_str(),
            1,
        )?;
        Ok((deletions1, deletions2))
    }

    pub fn get_entity_attribute_sorting_data(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        entity_id_in: i64,
        limit_in: Option<i64>, /*= None*/
    ) -> Result<Vec<Vec<Option<DataType>>>, anyhow::Error> {
        // see comments in get_group_entries_data
        self.db_query(transaction, format!("select attribute_form_id, attribute_id, sorting_index from AttributeSorting where \
                                    entity_id = {} order by sorting_index limit {}", entity_id_in, Self::check_if_should_be_all_results(limit_in)).as_str(),
                      "Int,i64,i64")
    }

    pub fn check_if_should_be_all_results(max_vals_in: Option<i64>) -> String {
        match max_vals_in {
            None => "ALL".to_string(),
            Some(x) if x <= 0 => "1".to_string(),
            Some(x) => format!("{}", x),
        }
    }

    pub fn class_limit(
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

    pub fn get_attribute_sorting_rows_count(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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

    pub fn get_relation_to_group_count_by_group(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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

    pub fn get_all_relation_to_local_entity_data_by_id(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id_in: i64,
    ) -> Result<Vec<Option<DataType>>, anyhow::Error> {
        self.db_query_wrapper_for_one_row(transaction,
                                          format!("select form_id, id, rel_type_id, entity_id, entity_id_2, valid_on_date, observation_date from RelationToEntity where id={}", id_in).as_str(),
                                          "Int,i64,i64,i64,i64,i64,i64")
    }

    pub fn get_all_relation_to_remote_entity_data_by_id(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id_in: i64,
    ) -> Result<Vec<Option<DataType>>, anyhow::Error> {
        self.db_query_wrapper_for_one_row(transaction,
                                          format!("select form_id, id, rel_type_id, entity_id, remote_instance_id, entity_id_2, valid_on_date, \
                                          observation_date from RelationToRemoteEntity where id={}", id_in).as_str(),
                                          "Int,i64,i64,i64,String,i64,i64,i64")
    }

    pub fn get_all_relation_to_group_data_by_id(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        entity_id: i64,
        relation_type_id: i64,
        group_id: i64,
    ) -> Result<bool, anyhow::Error> {
        self.does_this_exist(transaction,
                             format!("SELECT count(1) from RelationToGroup where entity_id={} and rel_type_id={} and group_id={}",
                                     entity_id, relation_type_id, group_id).as_str(), true)
    }

    /// Excludes those entities that are really relationtypes, attribute types, or quantity units.
    pub fn entity_only_key_exists(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        rel_type_id_in: i64,
        entity_id1_in: i64,
        entity_id2_in: i64,
    ) -> Result<bool, anyhow::Error> {
        self.does_this_exist(transaction, format!("SELECT count(1) from RelationToEntity where rel_type_id={} and entity_id={} and entity_id_2={}",
                                                  rel_type_id_in, entity_id1_in, entity_id2_in).as_str(), true)
    }

    fn relation_to_remote_entity_exists(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        rel_type_id_in: i64,
        entity_id1_in: i64,
        remote_instance_id_in: String,
        entity_id2_in: i64,
    ) -> Result<bool, anyhow::Error> {
        self.does_this_exist(transaction, format!("SELECT count(1) from RelationToRemoteEntity where rel_type_id={} and entity_id={} and \
                        remote_instance_id='{}' and entity_id_2={}",
                                                  rel_type_id_in, entity_id1_in, remote_instance_id_in, entity_id2_in).as_str(), true)
    }

    /// This takes a db parameter for the same reasons as in the comment on fn get_entities_generic.
    fn add_new_entity_to_results(
        &self,
        db: Rc<dyn Database>,
        //transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        final_results: &mut Vec<Entity>,
        intermediate_result_in: Vec<Option<DataType>>,
    ) -> Result<(), anyhow::Error> {
        let result = intermediate_result_in;
        let id: i64 = match result.get(0) {
            Some(val) => match val {
                Some(DataType::Bigint(x)) => *x,
                _ => {
                    return Err(anyhow!(
                        "Expected Some(DataType::Bigint(id)), got {:?}",
                        val
                    ))
                }
            },
            _ => {
                return Err(anyhow!(
                    "Expected Some(Some(DataType::Bigint(id))), got {:?}",
                    result.get(0)
                ))
            }
        };
        //let DataType::String(name) = result.get(1)?;
        let name: String = match result.get(1) {
            Some(val) => match val {
                Some(DataType::String(name)) => name.clone(),
                _ => {
                    return Err(anyhow!(
                        "Expected Some(DataType::String(name)), got {:?}",
                        val
                    ))
                }
            },
            _ => {
                return Err(anyhow!(
                    "Expected Some(Some(DataType::String(name))), got {:?}",
                    result.get(1)
                ))
            }
        };
        //let Option(DataType::Bigint(class_id)) = result.get(2);
        let class_id: Option<i64> = match result.get(2) {
            Some(val) => match val {
                Some(DataType::Bigint(id)) => Some(*id),
                None => None,
                _ => {
                    return Err(anyhow!(
                        "Expected Option<DataType::Bigint(class_id)>, got {:?}",
                        val
                    ))
                }
            },
            _ => {
                return Err(anyhow!(
                    "Expected Some(Option<DataType::Bigint(class_id)>), got {:?}",
                    result.get(2)
                ))
            }
        };
        //let DataType::Bigint(insertion_date) = result.get(3)?;
        let insertion_date: i64 = match result.get(3) {
            Some(val) => match val {
                Some(DataType::Bigint(x)) => *x,
                _ => {
                    return Err(anyhow!(
                        "Expected Some(DataType::Bigint(insertion_date)), got {:?}",
                        val
                    ))
                }
            },
            _ => {
                return Err(anyhow!(
                    "Expected Some(Some(DataType::Bigint(insertion_date))), got {:?}",
                    result.get(3)
                ))
            }
        };
        //let Option(DataType::Boolean(public)) = result.get(4);
        let public: Option<bool> = match result.get(4) {
            Some(val) => match val {
                Some(DataType::Boolean(x)) => Some(*x),
                None => None,
                _ => {
                    return Err(anyhow!(
                        "Expected Option<DataType::Boolean(public)>, got {:?}",
                        val
                    ))
                }
            },
            _ => {
                return Err(anyhow!(
                    "Expected Some(Option<DataType::Bigint(public)>), got {:?}",
                    result.get(4)
                ))
            }
        };
        //let DataType::Boolean(archived) = result.get(5)?;
        let archived: bool = match result.get(5) {
            Some(val) => match val {
                Some(DataType::Boolean(x)) => *x,
                _ => {
                    return Err(anyhow!(
                        "Expected Some(DataType::Boolean(archived)), got {:?}",
                        val
                    ))
                }
            },
            _ => {
                return Err(anyhow!(
                    "Expected Some(Some(DataType::Bigint(archived))), got {:?}",
                    result.get(5)
                ))
            }
        };
        //let DataType::Boolean(new_entries_stick_to_top) = result.get(6)?;
        let new_entries_stick_to_top: bool = match result.get(6) {
            Some(val) => match val {
                Some(DataType::Boolean(x)) => *x,
                _ => {
                    return Err(anyhow!(
                        "Expected Some(DataType::Boolean(new_entries_stick_to_top)), got {:?}",
                        val
                    ))
                }
            },
            _ => {
                return Err(anyhow!(
                    "Expected Some(Some(DataType::Bigint(new_entries_stick_to_top))), got {:?}",
                    result.get(6)
                ))
            }
        };

        let e: Entity = Entity::new(
            db,
            id,
            name,
            class_id,
            insertion_date,
            public,
            archived,
            new_entries_stick_to_top,
        );
        final_results.push(e);
        Ok(())
    }

    /// @return a Vec of (relation_type_id, entity_id) tuples.
    pub fn get_containing_entities_helper(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        sql_in: &str,
        //) -> Result<Vec<(i64, Entity)>, anyhow::Error> {
    ) -> Result<Vec<(i64, i64)>, anyhow::Error> {
        let early_results = self.db_query(transaction.clone(), sql_in, "i64,i64")?;
        let early_results_len = early_results.len();
        //let mut final_results: Vec<(i64, Entity)> = Vec::new();
        let mut final_results: Vec<(i64, i64)> = Vec::new();
        // idea: should the remainder of this method be moved to Entity, so the persistence layer doesn't know anything about the Model? (helps avoid circular
        // dependencies? is a cleaner design?.)
        for result in early_results {
            // None of these values should be of "None" type, so not checking for that. If they are it's a bug:
            let rel_type_id = get_i64_from_row(&result, 0)?;
            let id = get_i64_from_row(&result, 1)?;
            //let entity: Entity =
            //    Entity::new2(self as &dyn Database, transaction.clone(), id.clone()).unwrap();
            final_results.push((rel_type_id.clone(), id))
        }

        if !(final_results.len() == early_results_len) {
            return Err(anyhow!("In get_containing_entities_helper, final_results.len() ({}) != early_results.len() ({}).", final_results.len(), early_results_len));
        }
        Ok(final_results)
    }

    pub fn get_entities_used_as_attribute_types_sql(
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

    /// This also takes a db *parameter*, to handle lifetime issues that arose. There is probably a better
    /// way, but I tried and considered various things and it got complicated (like making this
    /// an associated function, or returning a collection of
    /// values to build an Entity or RelationType rather than an Entity or RelationType as such),
    /// and I didn't want to make do_query part of the Database trait,
    /// preferring to keep it internal so only the database code can call it, and thus calls to db_query can be more
    /// controlled to do it right per its comment, and anything else; I also didn't want to put the
    /// knowledge about handling returned column data from the database, outside the database and model code
    /// into whoever calls it (even though I have done that elsewhere, like in
    /// fn get_relations_to_group_containing_this_group).
    /// The starting_object_index_in is 0-based index to start with.
    /// The max_vals_in is # of obj's to return (if None, means no limit).
    // This fn used to be integrated with get_relation_types. Probably want to check both when
    // making changes?
    pub fn get_entities_generic(
        &self,
        db: Rc<dyn Database>,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        starting_object_index_in: i64,
        max_vals_in: Option<i64>,
        table_name_in: &str,
        class_id_in: Option<i64>,         /*= None*/
        limit_by_class: bool,             /*= false*/
        template_entity: Option<i64>,     /*= None*/
        group_to_omit_id_in: Option<i64>, /*= None*/
    ) -> Result<Vec<Entity>, anyhow::Error> {
        let some_sql = "";
        let more = " from Entity e ";
        let more3 = " where";
        let more4 = if !self.include_archived_entities() {
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
        //let more7 = "";
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
        let types = "i64,String,i64,i64,bool,bool,bool";
        let sql = format!(
            //"{}{}{}{}{}{} true {}{}{}{}{} order by id limit {} offset {}",
            "{}{}{}{}{} true {}{}{}{} order by id limit {} offset {}",
            Util::SELECT_ENTITY_START,
            some_sql,
            more,
            more3,
            more4,
            more5,
            more6,
            //more7,
            more8,
            more9,
            Self::check_if_should_be_all_results(max_vals_in),
            starting_object_index_in
        );
        let early_results: Vec<Vec<Option<DataType>>> =
            self.db_query(transaction.clone(), sql.as_str(), types)?;
        let early_results_len = early_results.len();

        let mut final_results: Vec<Entity> = Vec::new();
        // idea: should the remainder of this method be moved to Entity, so the persistence layer doesn't know anything about the Model? (helps avoid circular
        // dependencies; is a cleaner design?)  (and similar ones)
        for result in early_results {
            // None of these values should be of "None" type. If they are it's a bug:
            self.add_new_entity_to_results(db.clone(), &mut final_results, result)?;
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

    pub fn get_text_editor_command(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<String, anyhow::Error> {
        let system_entity_id = self.get_system_entity_id(transaction.clone())?;
        let has_relation_type_id: i64 = self.find_relation_type(
            transaction.clone(),
            Util::THE_HAS_RELATION_TYPE_NAME,
        )?;
        let editor_info_system_entities: Vec<i64> = self.get_entities_from_relations_to_local_entity(
            transaction.clone(),
            system_entity_id,
            Util::EDITOR_INFO_ENTITY_NAME,
            Some(has_relation_type_id),
            Some(1),
        )?;
        if editor_info_system_entities.len() < 1 {
            return Err(anyhow!(
                "In get_text_editor_command, Unexpected # of results in get_text_editor_command a: {}",
                editor_info_system_entities.len()
            ));
        }
        let id = editor_info_system_entities[0];
        let text_editor_info_system_entities: Vec<i64> = self.get_entities_from_relations_to_local_entity(
            transaction.clone(),
            id,
            Util::TEXT_EDITOR_INFO_ENTITY_NAME,
            Some(has_relation_type_id),
            Some(1),
        )?;
        if text_editor_info_system_entities.len() < 1 {
            return Err(anyhow!(
                "In get_text_editor_command, Unexpected # of results in get_text_editor_command b: {}",
                text_editor_info_system_entities.len()
            ));
        }
        let text_editor_info_system_entity_id = text_editor_info_system_entities[0];
        let text_editor_command_name_attr_types: Vec<i64> = self.get_entities_from_relations_to_local_entity(
            transaction.clone(),
            text_editor_info_system_entity_id,
            Util::TEXT_EDITOR_COMMAND_ATTRIBUTE_TYPE_NAME,
            Some(has_relation_type_id),
            Some(1),
        )?;
        if text_editor_command_name_attr_types.len() < 1 {
            return Err(anyhow!(
                "In get_text_editor_command, Unexpected # of results in get_text_editor_command c: {}",
                text_editor_command_name_attr_types.len()
            ));
        }
        let text_editor_command_name_attr_type_id = text_editor_command_name_attr_types[0];
        let tas: Vec<(i64, i64, i64, String, Option<i64>, i64, i64)> = self.get_text_attribute_by_type_id(
            transaction.clone(),
            text_editor_info_system_entity_id,
            text_editor_command_name_attr_type_id,
            Some(1),
        )?;
        if tas.len() < 1 {
            return Err(anyhow!(
                "In get_text_editor_command, Unexpected # of results in get_text_editor_command d: {}",
                tas.len()
            ));
        }
        //return the text value
        Ok(tas[0].3.clone())
    }
    
    fn get_entities_from_relations_to_local_entity(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        parent_entity_id_in: i64,
        name_in: &str,
        rel_type_id_in: Option<i64>, /*= None*/
        expected_rows: Option<u64>,  /*= None*/
    ) -> Result<Vec<i64>, anyhow::Error> {
        // (not getting all the attributes in this case, and doing another query to the entity table (less efficient), to save programming
        // time for the case that the entity table changes, we don't have to carefully update all the columns selected here & the mappings.  This is a more
        // likely change than for the TextAttribute table, below.
        let rel_type = match rel_type_id_in {
            Some(rtid) => format!(" and rel_type_id={}", rtid),
            _ => "".to_string(),
        };
        let query_results: Vec<Vec<Option<DataType>>> = self.db_query(transaction.clone(),
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
        let mut final_result: Vec<i64> = Vec::with_capacity(query_results.len());
        // let mut index: usize = 0;
        for r in query_results {
            if r.len() == 0 {
                return Err(anyhow!("In get_entities_from_relations_to_local_entity, in get_entities_from_relations_to_local_entity, did not expect returned row to have 0 elements!: {:?}", r));
            }
            if let Some(DataType::Bigint(id)) = r[0] {
                //final_result.push(Entity::new2( self as &dyn Database, transaction.clone(), id,)?);
                final_result.push(id);
                // index += 1
            } else {
                return Err(anyhow!("In get_entities_from_relations_to_local_entity, in get_entities_from_relations_to_local_entity, did not expect this: {:?}", r[0]));
            }
        }
        Ok(final_result)
    }

    /// The in_self_id_to_ignore parameter is to avoid saying a class is a duplicate of itself: checks for all others only.
    pub fn is_duplicate_row(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
    pub fn archive_objects<'a>(
        &'a self,
        transaction_in: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        table_name_in: &str,
        where_clause_in: &str,
        rows_expected: u64, /*= 1*/
        unarchive: bool,    /*= false*/
    ) -> Result<u64, anyhow::Error> {
        //idea: enhance this to also check & return the # of rows deleted, to the caller to just make sure? If so would have to let caller handle transactions.

        //BEGIN COPY/PASTED/DUPLICATED BLOCK-----------------------------------
        // Try creating a local transaction whether we use it or not, to handle compiler errors
        // about variable moves. I'm not seeing a better way to get around them by just using
        // conditions and an Option (many errors):
        // (I tried putting this in a function, then a macro, but it gets compile errors.
        // So, copy/pasting this, unfortunately, until someone thinks of a better way. (You
        // can see the macro, and one of the compile errors, in the commit of 2023-05-18.
        // I didn't try a proc macro but based on some reading I think it would have the same
        // problem.)
        let local_tx: Transaction<Postgres> = self.begin_trans()?;
        let local_tx_option = Some(Rc::new(RefCell::new(local_tx)));
        let transaction = if transaction_in.clone().is_some() {
            transaction_in.clone()
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
        let rows_affected = self.db_action(transaction.clone(), sql.as_str(), true, false)?;
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
            if transaction_in.is_none() && transaction.is_some() {
                // see comments at similar location in delete_objects about local_tx
                // see comments in delete_objects about rollback
                let local_tx_cell: Option<RefCell<Transaction<Postgres>>> =
                    Rc::into_inner(transaction.unwrap());
                match local_tx_cell {
                    Some(t) => {
                        let unwrapped_local_tx = t.into_inner();
                        if let Err(e) = self.commit_trans(unwrapped_local_tx) {
                            return Err(anyhow!(e.to_string()));
                        }
                    }
                    None => {
                        return Err(anyhow!("Unexpectedly found None instead of Some<RefCell<Transaction<Postgres>>>. How?"));
                    }
                }
            }
            Ok(rows_affected)
        }
    }

    pub fn delete_object_by_id<'a>(
        &'a self,
        transaction_in: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        table_name_in: &str,
        id_in: i64,
    ) -> Result<u64, anyhow::Error> {
        self.delete_objects(
            transaction_in,
            table_name_in,
            format!("where id={}", id_in).as_str(),
            1,
        )
    }

    pub fn delete_object_by_id2<'a>(
        &'a self,
        transaction_in: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        table_name_in: &str,
        id_in: &str,
    ) -> Result<u64, anyhow::Error> {
        self.delete_objects(
            transaction_in,
            table_name_in,
            format!("where id='{}'", id_in).as_str(),
            1,
        )
    }
    // (idea: find out: why doesn't compiler (ide or cli) complain when the 'override' is removed from next line?)
    // idea: see comment on find_unused_sorting_index
    pub fn find_id_which_is_not_key_of_any_entity(
        &self,
        transaction_in: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<i64, anyhow::Error> {
        //better idea?  This should be fast because we start in remote regions and return as soon as an unused id is found, probably
        //only one iteration, ever.  (See similar comments elsewhere.)
        let mut working_id: i64 = self.max_id_value() - 1;
        let mut counter: i64 = 0;
        loop {
            if self.entity_key_exists(transaction_in.clone(), working_id, true)? {
                if working_id == self.max_id_value() {
                    // means we did a full loop across all possible ids!?  Doubtful. Probably would turn into a
                    // performance problem long before. It's a bug.
                    return Err(anyhow!("In find_id_which_is_not_key_of_any_entity: No id found \
                          which is not a key of any entity in the system. How could all id's be used??"));
                }
                // idea: this check assumes that the thing to get IDs will re-use deleted ones
                // and wrap around the set of #'s. That fix is on the list (informally
                // at this writing, 2013-11-18).
                if counter > 1000 {
                    return Err(anyhow!("In find_id_which_is_not_key_of_any_entity: Very unexpected, \
                            but could it be that you are running out of available entity IDs?? Have someone \
                            check, before you need to create, for example, a thousand more entities."));
                }
                working_id -= 1;
                counter += 1;
                continue;
            } else {
                return Ok(working_id);
            }
        }
    }

    /// @return the OmInstance object that stands for *this*: the OmInstance to which this PostgreSQLDatabase class instance reads/writes directly.
    pub fn get_local_om_instance_data(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        //) -> Result<OmInstance, anyhow::Error> {
    ) -> Result<(String, bool, String, i64, Option<i64>), anyhow::Error> {
        let sql = "SELECT id, address, insertion_date, entity_id from omInstance where local=TRUE";
        let results = self.db_query(transaction, sql, "UUID,String,i64,i64")?;
        if results.len() != 1 {
            return Err(anyhow!(
                "Got {} instead of 1 result from sql {}.  Does the usage now \
                            warrant removing this check (ie, multiple locals stored)?",
                results.len(),
                sql
            ));
        }
        let result = results.get(0).unwrap();
        let DataType::String(id) = result[0].clone().unwrap() else {
            return Err(anyhow!(
                "In pgdb2.get_local_om_instance_data, unexpected value: {:?}",
                result[0]
            ));
        };
        let DataType::String(address) = result[1].clone().unwrap() else {
            return Err(anyhow!(
                "In pgdb2.get_local_om_instance_data, unexpected value: {:?}",
                result[1]
            ));
        };
        let DataType::Bigint(insertion_date) = result[2].clone().unwrap() else {
            return Err(anyhow!(
                "In pgdb2.get_local_om_instance_data, unexpected value: {:?}",
                result[2]
            ));
        };
        let entity_id = match result[3] {
            None => None,
            Some(DataType::Bigint(x)) => Some(x),
            _ => {
                return Err(anyhow!(
                    "Unexpected value {:?} from sql \"{}\".",
                    result[3],
                    sql
                ))
            }
        };
        // return a tuple instead of an OmInstance because I don't know how to construct one with
        // "self" as a parameter, rather than an owned db parameter. The caller can deal with it.
        //Ok(OmInstance::new(
        //    self,
        Ok((id, true, address, insertion_date, entity_id))
    }
}
