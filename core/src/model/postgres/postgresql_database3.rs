/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003, 2004, 2010, 2011, 2013-2020 inclusive, and 2023-2024 inclusive, Luke A. Call.
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
use crate::model::entity_class::EntityClass;
use crate::model::group::Group;
use crate::model::postgres::postgresql_database::*;
// use crate::model::postgres::*;
use crate::model::relation_to_entity::RelationToEntity;
use crate::model::relation_to_group::RelationToGroup;
use crate::model::relation_to_local_entity::RelationToLocalEntity;
use crate::model::relation_to_remote_entity::RelationToRemoteEntity;
use crate::model::text_attribute::TextAttribute;
use crate::util::Util;
use anyhow::anyhow;
use chrono::Utc;
// use futures::executor::block_on;
// use sqlx::postgres::*;
// Specifically omitting sql::Error from use statements so that it is *clearer* which Error type is
// in use, in the code.
// use sqlx::{Column, PgPool, Postgres, Row, Transaction, ValueRef};
use sqlx::{Postgres, Transaction};
use std::collections::HashSet;
// use std::fmt::format;
use crate::model::attribute::Attribute;
use crate::model::attribute_with_valid_and_observed_dates::AttributeWithValidAndObservedDates;
use std::cell::RefCell;
use std::rc::Rc;
use tracing::*;

impl Database for PostgreSQLDatabase {
    //%%do the lifetimes used with these parameters make sense? Or should there be a 'b? a 'c?
    fn add_uri_entity_with_uri_attribute<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        //%%why is there a warning (per top of main.rs) if the '_ is not here? What does it really mean?
        containing_entity_in: &'a Entity<'_>,
        new_entity_name_in: &str,
        uri_in: &str,
        observation_date_in: i64,
        make_them_public_in: Option<bool>,
        caller_manages_transactions_in: bool,
        quote_in: Option<&str>, /*= None*/
                                // ('a are per warnings from top of main.rs)
    ) -> Result<(Entity<'a>, RelationToLocalEntity<'a>), anyhow::Error> /*%%where 'a: 'b*/ {
        //) -> Result<(Entity, RelationToLocalEntity), anyhow::Error> {
        if quote_in.is_some() {
            if quote_in.unwrap().is_empty() {
                return Err(anyhow!("It doesn't make sense to store a blank quotation; there was probably a program error."));
            }
        }
        //%%put here lines like where I have "duplicated code" ie other plcs using
        //caller_manages_transaction_in ?:  (and for the commit at end?)
        //
        //rollbacketc%%FIX NEXT LINE AFTERI SEE HOW OTHERS DO!
        // if !caller_manages_transactions_in { self.begin_trans() }
        // try {
        // **idea: BAD SMELL: should this method be moved out of the db class, since it depends on higher-layer components, like EntityClass and
        // those in the same package? It was in Controller, but moved here
        // because it seemed like things that manage transactions should be in the db layer.  So maybe it needs un-mixing of layers.

        let (uri_class_id, uri_class_template_id) = self.get_or_create_class_and_template_entity(
            transaction.clone(),
            "URI",
            caller_manages_transactions_in,
        )?;
        let (_, quotation_class_template_id) = self.get_or_create_class_and_template_entity(
            transaction.clone(),
            "quote",
            caller_manages_transactions_in,
        )?;
        let (new_entity, new_rtle) = containing_entity_in
            .create_entity_and_add_has_local_relation_to_it(
                transaction.clone(),
                new_entity_name_in,
                observation_date_in,
                make_them_public_in,
                caller_manages_transactions_in,
            )?;
        self.update_entitys_class(
            transaction.clone(),
            new_entity.get_id(),
            Some(uri_class_id),
            caller_manages_transactions_in,
        )?;
        //(attempts to handle the transaction and lifetime compiler errors:)
        //let new_entity2: Entity<'a> = new_entity.clone();
        //let new_entity2 = Rc::new(RefCell::new(new_entity));
        //let new_entity3 = Rc::into_inner(new_entity2).unwrap().into_inner();
        new_entity.add_text_attribute2(
            //%%latertrans1 fix this (and next, similar one) to use the transaction again. How, given lifetime issues?
            //could make it so the above transaction parameter to the fn doesn't require a
            //lifetime, by changing that in postgresql_database.rs fn db_action, or such? Or
            //just fix things here somehow?
            //How is this call different from other db calls that don't have the problem?
            //some discussion was in:
            //https://users.rust-lang.org/t/error-e0597-new-entity2-does-not-live-long-enough/115725
            None, //transaction.clone(),
            uri_class_template_id,
            uri_in,
            None,
            None,
            observation_date_in,
            caller_manages_transactions_in,
        )?;
        if quote_in.is_some() {
            new_entity.add_text_attribute2(
                //%%latertrans1 fix this to use the transaction again. How, given lifetime issues?
                None, //transaction.clone(),
                quotation_class_template_id,
                quote_in.unwrap(),
                None,
                None,
                observation_date_in,
                caller_manages_transactions_in,
            )?;
        };
        //rollbacketc%%FIX NEXT LINE AFTERI SEE HOW OTHERS DO!
        // if !caller_manages_transactions_in {self.commit_trans() }
        //Ok((new_entity.clone(), new_rtle))
        Ok((new_entity, new_rtle))
        //  } catch {
        //    case e: Exception =>
        //rollbacketc%%FIX NEXT LINE AFTERI SEE HOW OTHERS DO!
        // if !caller_manages_transactions_in) rollback_trans()
    }

    fn get_text_attribute_by_type_id(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        parent_entity_id_in: i64,
        type_id_in: i64,
        expected_rows: Option<usize>, /*= None*/
    ) -> Result<Vec<TextAttribute>, anyhow::Error> {
        let form_id: i32 = self.get_attribute_form_id(Util::TEXT_TYPE).unwrap();
        let sql: String = format!("select ta.id, ta.textvalue, ta.attr_type_id, ta.valid_on_date, ta.observation_date, asort.sorting_index from \
             textattribute ta, AttributeSorting asort where ta.entity_id={} and ta.attr_type_id={} and ta.entity_id=asort.entity_id and \
             asort.attribute_form_id={} and ta.id=asort.attribute_id",
                 parent_entity_id_in, type_id_in, form_id);
        let query_results: Vec<Vec<Option<DataType>>> =
            self.db_query(transaction, sql.as_str(), "i64,String,i64,i64,i64,i64")?;
        if let Some(expected_rows_len) = expected_rows {
            if query_results.len() != expected_rows_len {
                return Err(anyhow!(
                    "In get_text_attribute_by_type_id, found {} rows instead of expected {}",
                    query_results.len(),
                    expected_rows_len
                ));
            }
        }
        let mut final_result: Vec<TextAttribute> = Vec::with_capacity(query_results.len());
        for r in query_results {
            if r.len() < 6 {
                return Err(anyhow!("In get_text_attribute_by_type_id, expected 6 elements in row returned, but found {}: {:?}", r.len(), r));
            }
            let err_msg = format!("Unexpected None from get_text_attribute_by_type_id from sql: \"{}\", at resulting vec element ", sql);
            let Some(DataType::Bigint(text_attribute_id)) =
                r.get(0).ok_or(anyhow!("{}{}", err_msg, 0))?
            else {
                return Err(anyhow!(
                    "Unexpected &None from sql/0: {}: {:?}",
                    sql.as_str(),
                    r.get(0)
                ));
            };
            let Some(DataType::String(textvalue)) = r.get(1).ok_or(anyhow!("{}{}", err_msg, 1))?
            else {
                return Err(anyhow!(
                    "Unexpected &None from sql/1: {}: {:?}",
                    sql.as_str(),
                    r.get(1)
                ));
            };
            let Some(DataType::Bigint(attr_type_id)) =
                r.get(2).ok_or(anyhow!("{}{}", err_msg, 2))?
            else {
                return Err(anyhow!(
                    "Unexpected &None from sql/2: {}: {:?}",
                    sql.as_str(),
                    r.get(2)
                ));
            };
            let valid_on_date = match r.get(3) {
                None => None,
                Some(Some(DataType::Bigint(vod))) => Some(*vod),
                _ => {
                    return Err(anyhow!(
                        "In get_text_attribute_by_type_id, unexpected value in {:?}",
                        r.get(3)
                    ))
                }
            };
            let Some(DataType::Bigint(observation_date)) =
                r.get(4).ok_or(anyhow!("{}{}", err_msg, 4))?
            else {
                return Err(anyhow!(
                    "Unexpected &None from sql/4: {}: {:?}",
                    sql,
                    r.get(4)
                ));
            };
            let Some(DataType::Bigint(sorting_index)) =
                r.get(5).ok_or(anyhow!("{}{}", err_msg, 5))?
            else {
                return Err(anyhow!(
                    "Unexpected &None from sql/5: {}: {:?}",
                    sql,
                    r.get(5)
                ));
            };
            final_result.push(TextAttribute::new(
                self as &dyn Database,
                *text_attribute_id,
                parent_entity_id_in,
                *attr_type_id,
                textvalue,
                valid_on_date,
                *observation_date,
                *sorting_index,
            ));
        }
        Ok(final_result)
    }

    fn is_attribute_sorting_index_in_use(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        //Util::print_backtrace();
        //panic!("%%latertrans2?: exiting for a test/tmp only to show what is calling this that causes deadlock?");
        //%%%%%%%
        let tx: Transaction<Postgres> = match self.rt.block_on(self.pool.begin()) {
            Err(e) => return Err(anyhow!(e.to_string())),
            Ok(t) => t,
        };
        // %% see comments in fn connect() re this AND remove above method comment??
        // connection.setAutoCommit(false);
        Ok(tx)
        //
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        results_in_out: &'a mut HashSet<i64>,
        from_entity_id_in: i64,
        search_string_in: &str,
        levels_remaining: i32,      /*= 20*/
        stop_after_any_found: bool, /*= true*/
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
            let related_entity_id_rows =
                self.db_query(transaction.clone(), sql.as_str(), "i64,String")?;
            // let lower_cased_regex_pattern = Pattern.compile(".*" + search_string_in.to_lowercase() + ".*");
            let mut id: i64;
            let mut name: String;
            for row in related_entity_id_rows {
                // id = match row.get(0) {
                //     Some(Some(DataType::Bigint(x))) => *x,
                //     _ => {
                //         return Err(anyhow!(
                //             "How did we get here for {:?}?",
                //             row.get(0)
                //         ))
                //     }
                // };
                id = get_i64_from_row(&row, 0)?;
                name = match row.get(1) {
                    Some(Some(DataType::String(x))) => x.clone(),
                    _ => return Err(anyhow!("How did we get here for {:?}?", row.get(1))),
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
                    transaction.clone(),
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
                let entities_in_groups =
                    self.db_query(transaction.clone(), sql2.as_str(), "i64,String")?;
                for row in entities_in_groups {
                    // let id: i64 = row(0).get.asInstanceOf[i64];
                    // let name = row(1).get.asInstanceOf[String];
                    //idea: surely there is some better way than what I am doing here? See other places similarly.
                    //   DataType::Bigint(id) = *row.get(0).unwrap();
                    //   DataType::String(name) = *row.get(1).unwrap();
                    id = match row.get(0) {
                        Some(Some(DataType::Bigint(x))) => *x,
                        _ => return Err(anyhow!("How did we get here for {:?}?", row.get(0))),
                    };
                    // DataType::String(name) = *row.get(1).unwrap();
                    name = match row.get(1) {
                        Some(Some(DataType::String(x))) => x.clone(),
                        _ => return Err(anyhow!("How did we get here for {:?}?", row.get(1))),
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
                        transaction.clone(),
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
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        class_name_in: &str,
    ) -> Result<(i64, i64), anyhow::Error> {
        self.create_class_and_its_template_entity2(
            transaction.clone(),
            class_name_in.to_string(),
            format!("{}{}", class_name_in.clone(), Util::TEMPLATE_NAME_SUFFIX),
            transaction.is_some(),
        )
    }

    fn delete_class_and_its_template_entity(&self, class_id_in: i64) -> Result<(), anyhow::Error> {
        let tx: Transaction<Postgres> = self.begin_trans()?;
        //let transaction: &Option<&mut Transaction<Postgres>> = &Some(&mut tx);
        let transaction = Some(Rc::new(RefCell::new(tx)));
        let template_entity_id_vec: Vec<Option<DataType>> =
            self.get_class_data(transaction.clone(), class_id_in)?;
        let template_entity_id: i64 = match template_entity_id_vec.get(1) {
            Some(Some(DataType::Bigint(n))) => *n,
            _ => {
                return Err(anyhow!(
                    "In delete_class_and_its_template_entity, Unexpected values for template: {:?}",
                    template_entity_id_vec
                ))
            }
        };
        let class_group_id: Option<i64> =
            self.get_system_entitys_class_group_id(transaction.clone())?;
        if class_group_id.is_some() {
            self.remove_entity_from_group(
                transaction.clone(),
                class_group_id.unwrap(),
                template_entity_id,
                true,
            )?;
        }
        self.update_entitys_class(transaction.clone(), template_entity_id, None, true)?;
        self.delete_object_by_id2(
            transaction.clone(),
            "class",
            class_id_in.to_string().as_str(),
            true,
        )?;
        self.delete_object_by_id2(
            transaction.clone(),
            Util::ENTITY_TYPE,
            template_entity_id.to_string().as_str(),
            true,
        )?;

        let local_tx_cell: Option<RefCell<Transaction<Postgres>>> =
            Rc::into_inner(transaction.unwrap());
        match local_tx_cell {
            Some(t) => {
                let unwrapped_local_tx = t.into_inner();
                if let Err(e) = self.commit_trans(unwrapped_local_tx) {
                    return Err(anyhow!(e.to_string()));
                }
                Ok(())
            }
            None => {
                return Err(anyhow!(
                    "Unexpectedly found None instead of Some<RefCell<Transaction<Postgres>>>. How?"
                ));
            }
        }
    }

    /// Returns at most 1 row's info (id, relation_type_id, group_id, name), and a boolean indicating if more were available.
    /// If 0 rows are found, returns (None, None, None, false), so this expects the caller
    /// to know there is only one or deal with the None.
    fn find_relation_to_and_group_on_entity(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        entity_id_in: i64,
        group_name_in: Option<String>, /*= None*/
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
            return Err(anyhow!("Found {} rows instead of expected {}", count, 1));
            //?: expected_rows.unwrap()));
        }
        // there could be none found, or more than one, but not after above check.
        //     let mut final_result: Vec<i64> = Vec::new();
        // for row in rows {
        let id: i64 = match rows[0].get(0) {
            Some(Some(DataType::Bigint(i))) => *i,
            _ => return Err(anyhow!("Found not 1 row with i64 but {:?} .", rows)),
        };
        // final_result.push(id);
        // }
        // Ok(final_result)
        Ok(id)
    }
    /// Saves data for a quantity attribute for a Entity (i.e., "6 inches length").<br>
    /// parent_id_in is the key of the Entity for which the info is being saved.<br>
    /// in_unit_id represents a Entity; indicates the unit for this quantity (i.e., liters or inches).<br>
    /// in_number represents "how many" of the given unit.<br>
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
    /// In the case of in_number, note
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
        transaction_in: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        parent_id_in: i64,
        attr_type_id_in: i64,
        unit_id_in: i64,
        number_in: f64,
        valid_on_date_in: Option<i64>,
        observation_date_in: i64,
        caller_manages_transactions_in: bool, /*= false*/
        sorting_index_in: Option<i64>,        /*= None*/
    ) -> Result</*id*/ i64, anyhow::Error> {
        /*For the duplicated code & comments just below, would ideas from these help?:
            The weird of function-local types in Rust
            https://elastio.github.io/bon/blog/the-weird-of-function-local-types-in-rust
            https://news.ycombinator.com/item?id=41272893
        */
        //BEGIN COPY/PASTED/DUPLICATED (except "in <fn_name>" in 2 Err msgs below) BLOCK-----------------------------------
        // Try creating a local transaction whether we use it or not, to handle compiler errors
        // about variable moves. I'm not seeing a better way to get around them by just using
        // conditions and an Option (many errors):
        // (I tried putting this in a function, then a macro, but it gets compile errors.
        // So, copy/pasting this, unfortunately, until someone thinks of a better way. (You
        // can see the macro, and one of the compile errors, in the commit of 2023-05-18.
        // I didn't try a proc macro but based on some reading I think it would have the same
        // problem.)
        let local_tx: Transaction<Postgres> = {
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
        //let local_tx_option = &Some(&mut local_tx);
        let local_tx_option = Some(Rc::new(RefCell::new(local_tx)));
        //let transaction: &Option<&mut Transaction<Postgres>> = if caller_manages_transactions_in {
        let transaction = if caller_manages_transactions_in {
            transaction_in
        } else {
            local_tx_option
        };
        //END OF COPY/PASTED/DUPLICATED BLOCK----------------------------------

        let id: i64 = self.get_new_key(transaction.clone(), "QuantityAttributeKeySequence")?;
        let form_id = self.get_attribute_form_id(Util::QUANTITY_TYPE)?;
        self.add_attribute_sorting_row(
            transaction.clone(),
            parent_id_in,
            form_id,
            id,
            sorting_index_in,
        )?;
        let valid_on = match valid_on_date_in {
            None => "NULL".to_string(),
            Some(d) => format!("{}", d),
        };
        self.db_action(transaction.clone(),
                       format!("insert into QuantityAttribute (id, entity_id, unit_id, \
                                         quantity_number, attr_type_id, valid_on_date, observation_date) values ({},{},{},{},\
                                         {},{},{})", id, parent_id_in, unit_id_in, number_in, attr_type_id_in, valid_on, observation_date_in).as_str(),
                       false, false)?;
        if !caller_manages_transactions_in {
            // see comments at similar location in delete_objects about local_tx
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
        Ok(id)
    }

    fn update_quantity_attribute(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id_in: i64,
        parent_id_in: i64,
        attr_type_id_in: i64,
        text_in: &str,
        valid_on_date_in: Option<i64>,
        observation_date_in: i64,
    ) -> Result<u64, anyhow::Error> {
        let text: String = Self::escape_quotes_etc(text_in.to_string());
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        self.db_action(transaction, format!("update BooleanAttribute set (boolean_value, attr_type_id, valid_on_date, observation_date) \
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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

    fn update_class_and_template_entity_name<'a>(
        &'a self,
        transaction_in: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        class_id_in: i64,
        name: &str,
        caller_manages_transactions_in: bool, /*= false*/
    ) -> Result<i64, anyhow::Error> {
        // let mut tx = self.begin_trans()?;
        //   let transaction: &Option<&mut Transaction<Postgres>> = &Some(&mut tx);
        //BEGIN COPY/PASTED/DUPLICATED (except "in <fn_name>" in 2 Err msgs below) BLOCK-----------------------------------
        // Try creating a local transaction whether we use it or not, to handle compiler errors
        // about variable moves. I'm not seeing a better way to get around them by just using
        // conditions and an Option (many errors):
        // (I tried putting this in a function, then a macro, but it gets compile errors.
        // So, copy/pasting this, unfortunately, until someone thinks of a better way. (You
        // can see the macro, and one of the compile errors, in the commit of 2023-05-18.
        // I didn't try a proc macro but based on some reading I think it would have the same
        // problem.)
        let local_tx: Transaction<Postgres> = {
            if transaction_in.is_none() {
                if caller_manages_transactions_in {
                    return Err(anyhow!("In update_class_and_template_entity_name, Inconsistent values for caller_manages_transactions_in \
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
                        "In update_class_and_template_entity_name, Inconsistent values for caller_manages_transactions_in & transaction_in: \
                                false and Some??"
                            .to_string(),
                    ));
                }
            }
        };
        let local_tx_option = Some(Rc::new(RefCell::new(local_tx)));
        let transaction = if caller_manages_transactions_in {
            transaction_in
        } else {
            local_tx_option
        };
        //END OF COPY/PASTED/DUPLICATED BLOCK----------------------------------

        self.update_class_name(transaction.clone(), class_id_in, name.to_string())?;
        let entity_id: i64 =
            EntityClass::new2(self as &dyn Database, transaction.clone(), class_id_in)?
                .get_template_entity_id(transaction.clone())?;
        self.update_entity_only_name(
            transaction.clone(),
            entity_id,
            format!("{}{}", name, Util::TEMPLATE_NAME_SUFFIX).as_str(),
        )?;
        // if let Err(e) = self.commit_trans(tx) {
        //     see comments in delete_objects about rollback
        // return Err(anyhow!(e.to_string()));
        // }
        if !caller_manages_transactions_in {
            // see comments at similar location in delete_objects about local_tx
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
        Ok(entity_id)
    }

    fn update_entitys_class<'a>(
        &'a self,
        transaction_in: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        entity_id: i64,
        class_id: Option<i64>,
        caller_manages_transactions_in: bool, /*= false*/
    ) -> Result<(), anyhow::Error> {
        //BEGIN COPY/PASTED/DUPLICATED (except "in <fn_name>" in 2 Err msgs below) BLOCK-----------------------------------
        // Try creating a local transaction whether we use it or not, to handle compiler errors
        // about variable moves. I'm not seeing a better way to get around them by just using
        // conditions and an Option (many errors):
        // (I tried putting this in a function, then a macro, but it gets compile errors.
        // So, copy/pasting this, unfortunately, until someone thinks of a better way. (You
        // can see the macro, and one of the compile errors, in the commit of 2023-05-18.
        // I didn't try a proc macro but based on some reading I think it would have the same
        // problem.)
        let local_tx: Transaction<Postgres> = {
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
        let local_tx_option = Some(Rc::new(RefCell::new(local_tx)));
        let transaction = if caller_manages_transactions_in {
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
            transaction.clone(),
            format!(
                "update Entity set (class_id) = ROW({}) where id={}",
                ci, entity_id
            )
            .as_str(),
            false,
            false,
        )?;
        let group_ids = self.db_query(
            transaction.clone(),
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
                self.are_mixed_classes_allowed(transaction.clone(), &group_id)?;
            if !mixed_classes_allowed && self.has_mixed_classes(transaction.clone(), &group_id)? {
                return Err(anyhow!(
                    "In update_entitys_class: {}",
                    Util::MIXED_CLASSES_EXCEPTION
                ));
            }
        }
        if !caller_manages_transactions_in {
            // see comments at similar location in delete_objects about local_tx
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
        let tx = self.begin_trans()?;
        let transaction = Some(Rc::new(RefCell::new(tx)));
        self.db_action(
            transaction.clone(),
            format!(
                "update Entity set (name) = ROW('{}') where id={}",
                name, id_in
            )
            .as_str(),
            false,
            false,
        )?;
        self.db_action(
            transaction.clone(),
            format!(
                "update RelationType set (name_in_reverse_direction, directionality) = \
                        ROW('{}', '{}') where entity_id={}",
                name_in_reverse_direction, directionality, id_in
            )
            .as_str(),
            false,
            false,
        )?;

        let local_tx_cell: Option<RefCell<Transaction<Postgres>>> =
            Rc::into_inner(transaction.unwrap());
        match local_tx_cell {
            Some(t) => {
                let unwrapped_local_tx = t.into_inner();
                if let Err(e) = self.commit_trans(unwrapped_local_tx) {
                    return Err(anyhow!(e.to_string()));
                }
                Ok(())
            }
            None => {
                return Err(anyhow!(
                    "Unexpectedly found None instead of Some<RefCell<Transaction<Postgres>>>. How?"
                ));
            }
        }
    }

    /// Re dates' meanings: see usage notes elsewhere in code (like inside create_tables).
    fn create_text_attribute<'a>(
        &'a self,
        // purpose: see comment in delete_objects
        transaction_in: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        parent_id_in: i64,
        attr_type_id_in: i64,
        text_in: &str,
        valid_on_date_in: Option<i64>, /*= None*/
        observation_date_in: i64,      /*= System.currentTimeMillis()*/
        // purpose: see comment in delete_objects
        caller_manages_transactions_in: bool, /*= false*/
        sorting_index_in: Option<i64>, /*(%%how comment places like this to show what I mean by it for readers? maybe search for "/\*=" and "/\* ="? :) = None*/
                                       // The "where..." on the next line means "where 'a outlives (or is >=) 'b" and is explained in
                                       // the Rust reference (as quoted by) and in chapter 7 of the helpful site:
                                       // https://tfpk.github.io/lifetimekata/chapter_7.html .
                                       //) -> Result<i64, anyhow::Error> where 'a: 'b {
    ) -> Result<i64, anyhow::Error> {
        //BEGIN COPY/PASTED/DUPLICATED (except "in <fn_name>" in 2 Err msgs below) BLOCK-----------------------------------
        // Try creating a local transaction whether we use it or not, to handle compiler errors
        // about variable moves. I'm not seeing a better way to get around them by just using
        // conditions and an Option (many errors):
        // (I tried putting this in a function, then a macro, but it gets compile errors.
        // So, copy/pasting this, unfortunately, until someone thinks of a better way. (You
        // can see the macro, and one of the compile errors, in the commit of 2023-05-18.
        // I didn't try a proc macro but based on some reading I think it would have the same
        // problem.)
        let local_tx: Transaction<'a, Postgres> = {
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
        let local_tx_option = Some(Rc::new(RefCell::new(local_tx)));
        let transaction = if caller_manages_transactions_in {
            transaction_in
        } else {
            local_tx_option
        };
        //END OF COPY/PASTED/DUPLICATED BLOCK----------------------------------

        let text: String = Self::escape_quotes_etc(text_in.to_string());
        let id: i64 = self.get_new_key(transaction.clone(), "TextAttributeKeySequence")?;
        let add_result = self.add_attribute_sorting_row(
            transaction.clone(),
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
            transaction.clone(),
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
            let local_tx_cell: Option<RefCell<Transaction<'a, Postgres>>> =
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
        Ok(id)
    }

    fn create_date_attribute<'a>(
        &'a self,
        transaction_in: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        parent_id_in: i64,
        attr_type_id_in: i64,
        date_in: i64,
        sorting_index_in: Option<i64>,        /*= None*/
        caller_manages_transactions_in: bool, /*= false*/
    ) -> Result</*id*/ i64, anyhow::Error> {
        //BEGIN COPY/PASTED/DUPLICATED (except "in <fn_name>" in 2 Err msgs below) BLOCK-----------------------------------
        // Try creating a local transaction whether we use it or not, to handle compiler errors
        // about variable moves. I'm not seeing a better way to get around them by just using
        // conditions and an Option (many errors):
        // (I tried putting this in a function, then a macro, but it gets compile errors.
        // So, copy/pasting this, unfortunately, until someone thinks of a better way. (You
        // can see the macro, and one of the compile errors, in the commit of 2023-05-18.
        // I didn't try a proc macro but based on some reading I think it would have the same
        // problem.)
        let local_tx: Transaction<Postgres> = {
            if transaction_in.is_none() {
                if caller_manages_transactions_in {
                    return Err(anyhow!("In create_date_attribute, inconsistent values for caller_manages_transactions_in \
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
                        "In create_date_attribute, inconsistent values for caller_manages_transactions_in & transaction_in: \
                                false and Some??"
                            .to_string(),
                    ));
                }
            }
        };
        //let local_tx_option = &Some(&mut local_tx);
        let local_tx_option = Some(Rc::new(RefCell::new(local_tx)));
        let transaction = if caller_manages_transactions_in {
            transaction_in
        } else {
            local_tx_option
        };
        //END OF COPY/PASTED/DUPLICATED BLOCK----------------------------------

        let id: i64 = self.get_new_key(transaction.clone(), "DateAttributeKeySequence")?;
        self.add_attribute_sorting_row(
            transaction.clone(),
            parent_id_in,
            self.get_attribute_form_id(Util::DATE_TYPE).unwrap(),
            id,
            sorting_index_in,
        )?;
        self.db_action(
            transaction.clone(),
            format!(
                "insert into DateAttribute (id, entity_id, attr_type_id, date) \
                    values ({},{},'{}',{})",
                id, parent_id_in, attr_type_id_in, date_in
            )
            .as_str(),
            false,
            false,
        )?;
        if !caller_manages_transactions_in {
            // Using local_tx to make the compiler happy and because it is the one we need,
            // if !caller_manages_transactions_in. Ie, there is no transaction provided by
            // the caller.
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
            };
        }
        Ok(id)
    }

    fn create_boolean_attribute<'a>(
        &'a self,
        transaction_in: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        parent_id_in: i64,
        attr_type_id_in: i64,
        boolean_in: bool,
        valid_on_date_in: Option<i64>,
        observation_date_in: i64,
        sorting_index_in: Option<i64>,        /*= None*/
        caller_manages_transactions_in: bool, /*= false*/
    ) -> Result<i64, anyhow::Error> {
        //BEGIN COPY/PASTED/DUPLICATED (except "in <fn_name>" in 2 Err msgs below) BLOCK-----------------------------------
        // Try creating a local transaction whether we use it or not, to handle compiler errors
        // about variable moves. I'm not seeing a better way to get around them by just using
        // conditions and an Option (many errors):
        // (I tried putting this in a function, then a macro, but it gets compile errors.
        // So, copy/pasting this, unfortunately, until someone thinks of a better way. (You
        // can see the macro, and one of the compile errors, in the commit of 2023-05-18.
        // I didn't try a proc macro but based on some reading I think it would have the same
        // problem.)
        let local_tx: Transaction<Postgres> = {
            if transaction_in.is_none() {
                if caller_manages_transactions_in {
                    return Err(anyhow!("In create_boolean_attribute, inconsistent values for caller_manages_transactions_in \
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
                        "In create_boolean_attribute, inconsistent values for caller_manages_transactions_in & transaction_in: \
                                false and Some??"
                            .to_string(),
                    ));
                }
            }
        };
        //let local_tx_option = &Some(&mut local_tx);
        let local_tx_option = Some(Rc::new(RefCell::new(local_tx)));
        let transaction = if caller_manages_transactions_in {
            transaction_in
        } else {
            local_tx_option
        };
        //END OF COPY/PASTED/DUPLICATED BLOCK----------------------------------

        let id: i64 = self.get_new_key(transaction.clone(), "BooleanAttributeKeySequence")?;
        // try {
        self.add_attribute_sorting_row(
            transaction.clone(),
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
            transaction.clone(),
            format!(
                "insert into BooleanAttribute (id, \
            entity_id, boolean_value, attr_type_id, valid_on_date, observation_date) \
            values ({},{},'{}',{},{},{})",
                id, parent_id_in, boolean_in, attr_type_id_in, vod, observation_date_in
            )
            .as_str(),
            false,
            false,
        )?;

        if !caller_manages_transactions_in {
            // Using local_tx to make the compiler happy and because it is the one we need,
            // if !caller_manages_transactions_in. Ie, there is no transaction provided by
            // the caller.
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
            };
        }
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
        transaction_in: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        relation_type_id_in: i64,
        entity_id1_in: i64,
        entity_id2_in: i64,
        valid_on_date_in: Option<i64>,
        observation_date_in: i64,
        sorting_index_in: Option<i64>, /*= None*/
        // purpose: see comment in delete_objects
        caller_manages_transactions_in: bool, /*= false*/
    ) -> Result<RelationToLocalEntity, anyhow::Error> {
        debug!("in create_relation_to_local_entity 0");
        //BEGIN COPY/PASTED/DUPLICATED (except "in <fn_name>" in 2 Err msgs below) BLOCK-----------------------------------
        // Try creating a local transaction whether we use it or not, to handle compiler errors
        // about variable moves. I'm not seeing a better way to get around them by just using
        // conditions and an Option (many errors):
        // (I tried putting this in a function, then a macro, but it gets compile errors.
        // So, copy/pasting this, unfortunately, until someone thinks of a better way. (You
        // can see the macro, and one of the compile errors, in the commit of 2023-05-18.
        // I didn't try a proc macro but based on some reading I think it would have the same
        // problem.)
        let local_tx: Transaction<Postgres> = {
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
        let local_tx_option = Some(Rc::new(RefCell::new(local_tx)));
        let transaction = if caller_manages_transactions_in {
            transaction_in
        } else {
            local_tx_option
        };
        //END OF COPY/PASTED/DUPLICATED BLOCK----------------------------------

        debug!("in create_relation_to_local_entity 1");
        let rte_id: i64 = self.get_new_key(transaction.clone(), "RelationToEntityKeySequence")?;
        let result: Result<i64, anyhow::Error> = self.add_attribute_sorting_row(
            transaction.clone(),
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
        let result = self.db_action(transaction.clone(), format!("INSERT INTO RelationToEntity (id, rel_type_id, entity_id, entity_id_2, valid_on_date, observation_date) \
                       VALUES ({},{},{},{}, {},{})", rte_id, relation_type_id_in, entity_id1_in, entity_id2_in,
                                                          valid_on_date_sql_str, observation_date_in).as_str(), false, false);
        debug!("in create_relation_to_local_entity 3");
        if let Err(e) = result {
            // see comments in delete_objects about rollback
            return Err(anyhow!(e));
        }
        debug!("in create_relation_to_local_entity 4");
        let rtle = RelationToLocalEntity::new2(
            self,
            transaction.clone(),
            rte_id,
            relation_type_id_in,
            entity_id1_in,
            entity_id2_in,
        )?;
        if !caller_manages_transactions_in {
            // see comments at similar location in delete_objects about local_tx
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
        debug!("in create_relation_to_local_entity 5");
        Ok(rtle)
    }

    /** Re dates' meanings: see usage notes elsewhere in code (like inside create_tables). */
    fn create_relation_to_remote_entity<'a>(
        &'a self,
        // purpose: see comment in delete_objects
        transaction_in: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        relation_type_id_in: i64,
        entity_id1_in: i64,
        entity_id2_in: i64,
        valid_on_date_in: Option<i64>,
        observation_date_in: i64,
        remote_instance_id_in: &str,
        sorting_index_in: Option<i64>, /*= None*/
        // purpose: see comment in delete_objects
        caller_manages_transactions_in: bool, /*= false*/
    ) -> Result<RelationToRemoteEntity, anyhow::Error> {
        //BEGIN COPY/PASTED/DUPLICATED (except "in <fn_name>" in 2 Err msgs below) BLOCK-----------------------------------
        // Try creating a local transaction whether we use it or not, to handle compiler errors
        // about variable moves. I'm not seeing a better way to get around them by just using
        // conditions and an Option (many errors):
        // (I tried putting this in a function, then a macro, but it gets compile errors.
        // So, copy/pasting this, unfortunately, until someone thinks of a better way. (You
        // can see the macro, and one of the compile errors, in the commit of 2023-05-18.
        // I didn't try a proc macro but based on some reading I think it would have the same
        // problem.)
        let local_tx: Transaction<Postgres> = {
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
        let local_tx_option = Some(Rc::new(RefCell::new(local_tx)));
        let transaction = if caller_manages_transactions_in {
            transaction_in
        } else {
            local_tx_option
        };
        //END OF COPY/PASTED/DUPLICATED BLOCK----------------------------------

        let rte_id: i64 =
            self.get_new_key(transaction.clone(), "RelationToRemoteEntityKeySequence")?;
        // not creating anything in a remote DB, but a local record of a local relation to a remote entity.
        let result = self.add_attribute_sorting_row(
            transaction.clone(),
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
        let result = self.db_action(transaction.clone(), format!("INSERT INTO RelationToRemoteEntity (id, rel_type_id, entity_id, \
                  entity_id_2, valid_on_date, observation_date, remote_instance_id) VALUES ({},{},{},{},{},{},'{}')",
                                                          rte_id, relation_type_id_in, entity_id1_in, entity_id2_in,
                                                          valid_on_date_sql_str, observation_date_in, remote_instance_id_in).as_str(), false, false);
        if let Err(e) = result {
            // see comments in delete_objects about rollback
            return Err(anyhow!(e));
        }
        if !caller_manages_transactions_in {
            // see comments at similar location in delete_objects about local_tx
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
        Ok(RelationToRemoteEntity {}) //%%%%really: self, rte_id, relation_type_id_in, entity_id1_in, remote_instance_id_in, entity_id2_in
    }

    /// Re dates' meanings: see usage notes elsewhere in code (like inside create_tables).
    fn update_relation_to_local_entity(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
    fn move_relation_to_local_entity_into_local_entity(
        &self,
        rtle_id_in: i64,
        to_containing_entity_id_in: i64,
        sorting_index_in: i64,
    ) -> Result<RelationToLocalEntity, anyhow::Error> {
        let tx = self.begin_trans()?;
        //let transaction: &Option<&mut Transaction<Postgres>> = &Some(&mut tx);
        let transaction = Some(Rc::new(RefCell::new(tx)));
        let rte_data: Vec<Option<DataType>> =
            self.get_all_relation_to_local_entity_data_by_id(transaction.clone(), rtle_id_in)?;
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
                "In move_relation_to_local_entity_into_local_entity, Unexpected valid_on_date: {:?}",
                rte_data.get(5)
            ))
            }
        };
        let observed_date = get_i64_from_row(&rte_data, 6)?;
        self.delete_relation_to_local_entity(
            transaction.clone(),
            old_rte_rel_type,
            old_rte_entity_1,
            old_rte_entity_2,
        )?;
        let new_rte: RelationToLocalEntity = self.create_relation_to_local_entity(
            transaction.clone(),
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
                return Err(anyhow!(
                    "Unexpectedly found None instead of Some<RefCell<Transaction<Postgres>>>. How?"
                ));
            }
        };
        Ok(new_rte)
    }

    /// See comments on & in method move_relation_to_local_entity_into_local_entity.  Only this one takes an RTRE (stored locally), and instead of linking it inside one local
    /// entity, links it inside another local entity.
    fn move_relation_to_remote_entity_to_local_entity(
        &self,
        remote_instance_id_in: &str,
        relation_to_remote_entity_id_in: i64,
        to_containing_entity_id_in: i64,
        sorting_index_in: i64,
    ) -> Result<RelationToRemoteEntity, anyhow::Error> {
        let tx = self.begin_trans()?;
        let transaction = Some(Rc::new(RefCell::new(tx)));
        let rte_data: Vec<Option<DataType>> = self.get_all_relation_to_remote_entity_data_by_id(
            transaction.clone(),
            relation_to_remote_entity_id_in,
        )?;
        // next lines are the same as in move_relation_to_local_entity_into_local_entity; could maintain them similarly.
        let old_rte_rel_type = get_i64_from_row(&rte_data, 2)?;
        let old_rte_entity_1 = get_i64_from_row(&rte_data, 3)?;
        let old_rte_entity_2 = get_i64_from_row(&rte_data, 4)?;
        let valid_on_date: Option<i64> = match rte_data.get(5) {
            //%%does this work in both cases?? (ie, from fn db_query, to here)
            Some(None) => None,
            Some(Some(DataType::Bigint(i))) => Some(i.clone()),
            _ => {
                return Err(anyhow!(
                "In move_relation_to_local_entity_into_local_entity, Unexpected valid_on_date: {:?}",
                rte_data.get(5)
            ))
            }
        };
        let observed_date = get_i64_from_row(&rte_data, 6)?;
        self.delete_relation_to_remote_entity(
            transaction.clone(),
            old_rte_rel_type,
            old_rte_entity_1,
            remote_instance_id_in,
            old_rte_entity_2,
        )?;
        let new_rte: RelationToRemoteEntity = self.create_relation_to_remote_entity(
            transaction.clone(),
            old_rte_rel_type,
            to_containing_entity_id_in,
            old_rte_entity_2,
            valid_on_date,
            observed_date,
            remote_instance_id_in,
            Some(sorting_index_in),
            true,
        )?;
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
                return Err(anyhow!(
                    "Unexpectedly found None instead of Some<RefCell<Transaction<Postgres>>>. How?"
                ));
            }
        }
        Ok(new_rte)
    }

    fn create_group(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        name_in: &str,
        allow_mixed_classes_in_group_in: bool, /*= false*/
    ) -> Result<i64, anyhow::Error> {
        let name: String = Self::escape_quotes_etc(name_in.to_string());
        let group_id: i64 = self.get_new_key(transaction.clone(), "RelationToGroupKeySequence")?;
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
        transaction_in: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        entity_id_in: i64,
        relation_type_id_in: i64,
        new_group_name_in: &str,
        allow_mixed_classes_in_group_in: bool, /*= false*/
        valid_on_date_in: Option<i64>,
        observation_date_in: i64,
        sorting_index_in: Option<i64>,
        // purpose: see comment in delete_objects
        caller_manages_transactions_in: bool, /*= false*/
    ) -> Result<(i64, i64), anyhow::Error> {
        //%%latertrans: fix/simplify these blocks??  can dup in a simple enough thing to post on URLO?
        //BEGIN COPY/PASTED/DUPLICATED (except "in <fn_name>" in 2 Err msgs below) BLOCK-----------------------------------
        // Try creating a local transaction whether we use it or not, to handle compiler errors
        // about variable moves. I'm not seeing a better way to get around them by just using
        // conditions and an Option (many errors):
        // (I tried putting this in a function, then a macro, but it gets compile errors.
        // So, copy/pasting this, unfortunately, until someone thinks of a better way. (You
        // can see the macro, and one of the compile errors, in the commit of 2023-05-18.
        // I didn't try a proc macro but based on some reading I think it would have the same
        // problem.)
        let local_tx: Transaction<Postgres> = {
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
        let local_tx_option = Some(Rc::new(RefCell::new(local_tx)));
        let transaction = if caller_manages_transactions_in {
            transaction_in
        } else {
            local_tx_option
        };
        //END OF COPY/PASTED/DUPLICATED BLOCK----------------------------------

        let group_id: i64 = self.create_group(
            transaction.clone(),
            new_group_name_in,
            allow_mixed_classes_in_group_in,
        )?;
        //%%%%%%this gets the deadlock from the test:
        let (rtg_id, _) = self.create_relation_to_group(
            transaction.clone(),
            entity_id_in,
            relation_type_id_in,
            group_id,
            valid_on_date_in,
            observation_date_in,
            sorting_index_in,
            true,
        )?;
        /*%%%%%%
        if !caller_manages_transactions_in {
            // see comments at similar location in delete_objects about local_tx
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
        Ok((group_id, rtg_id))
        %%%%%%*/
        Ok((0 as i64, 0 as i64))
    }

    /// I.e., make it so the entity has a relation to a new entity in it.
    /// Re dates' meanings: see usage notes elsewhere in code (like inside create_tables).
    fn create_entity_and_relation_to_local_entity<'a>(
        &'a self,
        // purpose: see comment in delete_objects
        transaction_in: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        entity_id_in: i64,
        relation_type_id_in: i64,
        new_entity_name_in: &str,
        is_public_in: Option<bool>,
        valid_on_date_in: Option<i64>,
        observation_date_in: i64,
        // purpose: see comment in delete_objects
        caller_manages_transactions_in: bool, /*= false*/
    ) -> Result<(i64, i64), anyhow::Error> {
        let name: String = Self::escape_quotes_etc(new_entity_name_in.to_string());

        //BEGIN COPY/PASTED/DUPLICATED (except "in <fn_name>" in 2 Err msgs below) BLOCK-----------------------------------
        // Try creating a local transaction whether we use it or not, to handle compiler errors
        // about variable moves. I'm not seeing a better way to get around them by just using
        // conditions and an Option (many errors):
        // (I tried putting this in a function, then a macro, but it gets compile errors.
        // So, copy/pasting this, unfortunately, until someone thinks of a better way. (You
        // can see the macro, and one of the compile errors, in the commit of 2023-05-18.
        // I didn't try a proc macro but based on some reading I think it would have the same
        // problem.)
        let local_tx: Transaction<Postgres> = {
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
        let local_tx_option = Some(Rc::new(RefCell::new(local_tx)));
        let transaction = if caller_manages_transactions_in {
            transaction_in
        } else {
            local_tx_option
        };
        //END OF COPY/PASTED/DUPLICATED BLOCK----------------------------------

        let new_entity_id: i64 =
            self.create_entity(transaction.clone(), name.as_str(), None, is_public_in)?;
        let _new_rte: RelationToLocalEntity = self.create_relation_to_local_entity(
            //%%should this not be "_in"?:
            transaction.clone(),
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
            // see comments in delete_objects about rollback
            let local_tx_cell: Option<RefCell<Transaction<Postgres>>> =
                Rc::into_inner(transaction.unwrap());
            match local_tx_cell {
                Some(t) => {
                    let unwrapped_local_tx = t.into_inner();
                    if let Err(e) = self.commit_trans(unwrapped_local_tx) {
                        return Err(anyhow!(
                            "In create_entity_and_relation_to_local_entity, {}: ",
                            e.to_string()
                        ));
                    }
                }
                None => {
                    return Err(anyhow!("Unexpectedly found None instead of Some<RefCell<Transaction<Postgres>>>. How?"));
                }
            }
        }
        //%%FIX NEXT LINE
        Ok((new_entity_id, 0)) //%%%%really: , new_rte.get_id()))
    }

    /// I.e., make it so the entity has a group in it, which can contain entities.
    /// Re dates' meanings: see usage notes elsewhere in code (like inside create_tables).
    /// @return a tuple containing the id and new sorting_index: (id, sorting_index)
    fn create_relation_to_group<'a>(
        &'a self,
        // purpose: see comment in delete_objects
        transaction_in: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        entity_id_in: i64,
        relation_type_id_in: i64,
        group_id_in: i64,
        valid_on_date_in: Option<i64>,
        observation_date_in: i64,
        sorting_index_in: Option<i64>, /*= None*/
        // purpose: see comment in delete_objects
        caller_manages_transactions_in: bool, /*= false*/
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
        let local_tx: Transaction<Postgres> = {
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
        let local_tx_option = Some(Rc::new(RefCell::new(local_tx)));
        let transaction = if caller_manages_transactions_in {
            transaction_in
        } else {
            local_tx_option
        };
        //END OF COPY/PASTED/DUPLICATED BLOCK----------------------------------

        let id: i64 = self.get_new_key(transaction.clone(), "RelationToGroupKeySequence2")?;
        let sorting_index = {
            let sorting_index: i64 = self.add_attribute_sorting_row(
                transaction.clone(),
                entity_id_in,
                self.get_attribute_form_id(Util::RELATION_TO_GROUP_TYPE)
                    .unwrap(),
                id,
                sorting_index_in,
            )?;
            //%%%%%%
            //%%%%%%%this gets the deadlock from the test:
            let valid_date = match valid_on_date_in {
                None => "NULL".to_string(),
                Some(d) => d.to_string(),
            };
            self.db_action(transaction.clone(), format!("INSERT INTO RelationToGroup (id, entity_id, rel_type_id, group_id, valid_on_date, observation_date) \
                             VALUES ({},{},{},{},{},{})", id, entity_id_in, relation_type_id_in, group_id_in, valid_date, observation_date_in).as_str(),
                           false, false)?;
            sorting_index
            //%%%%%%//
            //0
        };
        /*%%%%%%
        if !caller_manages_transactions_in {
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
        Ok((id, sorting_index))
            %%%%%%*/
        Ok((0 as i64, 0 as i64))
    }

    fn update_group(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        let tx = self.begin_trans()?;
        let transaction = Some(Rc::new(RefCell::new(tx)));
        let rtg_data: Vec<Option<DataType>> = self
            .get_all_relation_to_group_data_by_id(transaction.clone(), relation_to_group_id_in)?;

        // next lines are the same as in move_relation_to_local_entity_into_local_entity and its sibling; could maintain them similarly.
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
            transaction.clone(),
            old_rtg_entity_id,
            old_rtg_rel_type,
            old_rtg_group_id,
        )?;
        let (new_rtg_id, _) = self.create_relation_to_group(
            transaction.clone(),
            new_containing_entity_id_in,
            old_rtg_rel_type,
            old_rtg_group_id,
            valid_on_date,
            observed_date,
            Some(sorting_index_in),
            true,
        )?;

        // (see comment at similar commented line in move_relation_to_local_entity_into_local_entity)
        //db_action("UPDATE RelationToGroup SET (entity_id) = ROW(" + new_containing_entity_id_in + ")" + " where id=" + relation_to_group_id_in)

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
                return Err(anyhow!(
                    "Unexpectedly found None instead of Some<RefCell<Transaction<Postgres>>>. How?"
                ));
            }
        }
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
        let tx = self.begin_trans()?;
        let transaction = Some(Rc::new(RefCell::new(tx)));
        self.add_entity_to_group(
            transaction.clone(),
            to_group_id_in,
            move_entity_id_in,
            Some(sorting_index_in),
            true,
        )?;
        self.remove_entity_from_group(
            transaction.clone(),
            from_group_id_in,
            move_entity_id_in,
            true,
        )?;
        if self.is_entity_in_group(transaction.clone(), to_group_id_in, move_entity_id_in)?
            && !self.is_entity_in_group(transaction.clone(), from_group_id_in, move_entity_id_in)?
        {
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
            Ok(())
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
        let tx = self.begin_trans()?;
        let transaction = Some(Rc::new(RefCell::new(tx)));
        self.add_has_relation_to_local_entity(
            transaction.clone(),
            to_entity_id_in,
            move_entity_id_in,
            None,
            Utc::now().timestamp_millis(),
            Some(sorting_index_in),
        )?;
        self.remove_entity_from_group(
            transaction.clone(),
            from_group_id_in,
            move_entity_id_in,
            true,
        )?;
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
                return Err(anyhow!(
                    "Unexpectedly found None instead of Some<RefCell<Transaction<Postgres>>>. How?"
                ));
            }
        }
        Ok(())
    }
    /// (See comments on moveEntityFromGroupToGroup.)
    fn move_local_entity_from_local_entity_to_group(
        &self,
        removing_rtle_in: &mut RelationToLocalEntity,
        target_group_id_in: i64,
        sorting_index_in: i64,
    ) -> Result<(), anyhow::Error> {
        //%%latertrans1: why dont the lines below compile as expected? (the commit_trans unwrap call
        //moves transaction.):
        //let mut tx = self.begin_trans()?;
        //let transaction: &Option<&mut Transaction<Postgres>> = &Some(&mut tx);
        ////let trans: &mut Transaction<Postgres> = &mut tx;
        self.add_entity_to_group(
            //%%latertrans1
            //transaction,
            ////&Some(trans),
            ////%%latertrans1 down to here also (and just below)
            None,
            target_group_id_in,
            removing_rtle_in.get_related_id2(),
            Some(sorting_index_in),
            true,
        )?;
        self.delete_relation_to_local_entity(
            //%%latertrans1: here too:  transaction,
            ////&Some(trans),
            None,
            //%%latertrans1 here too: removing_rtle_in.get_attr_type_id(transaction)?,
            ////removing_rtle_in.get_attr_type_id(&Some(trans))?,
            removing_rtle_in.get_attr_type_id(None)?,
            removing_rtle_in.get_related_id1(),
            removing_rtle_in.get_related_id2(),
        )?;
        //%%latertrans: self.commit_trans(*(transaction.unwrap()))
        ////%%latertrans: self.commit_trans(*trans)
        //%%latertrans:
        Ok(())
    }

    // SEE ALSO METHOD find_unused_attribute_sorting_index **AND DO MAINTENANCE IN BOTH PLACES**
    // idea: this needs a test, and/or combining with findIdWhichIsNotKeyOfAnyEntity.
    // **ABOUT THE SORTINGINDEX:  SEE the related comment on method add_attribute_sorting_row.
    fn find_unused_group_sorting_index(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        group_id_in: i64,
        starting_with_in: Option<i64>, /*= None*/
    ) -> Result<i64, anyhow::Error> {
        //better idea?  This should be fast because we start in remote regions and return as soon as an unused id is found, probably
        //only one iteration, ever.  (See similar comments elsewhere.)
        // find_unused_sorting_index_helper(group_id_in, starting_with_in.getOrElse(max_id_value - 1), 0)
        let g_id = group_id_in;
        let mut working_index = starting_with_in.unwrap_or(self.max_id_value() - 1);
        let mut counter = 0;

        loop {
            //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) w/in this method!
            if self.is_group_entry_sorting_index_in_use(transaction.clone(), g_id, working_index)? {
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        entity_id_in: i64,
        starting_with_in: Option<i64>, /*= None*/
    ) -> Result<i64, anyhow::Error> {
        let mut working_index = starting_with_in.unwrap_or(self.max_id_value() - 1);
        let mut counter = 0;
        loop {
            if self.is_attribute_sorting_index_in_use(
                transaction.clone(),
                entity_id_in,
                working_index,
            )? {
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
        transaction_in: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        group_id_in: i64,
        contained_entity_id_in: i64,
        sorting_index_in: Option<i64>, /*= None*/
        // purpose: see comment in delete_objects
        caller_manages_transactions_in: bool, /*= false*/
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
        let local_tx: Transaction<Postgres> = {
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
        let local_tx_option = Some(Rc::new(RefCell::new(local_tx)));
        let transaction = if caller_manages_transactions_in {
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
                None if self.get_group_size(transaction.clone(), group_id_in, 3)? == 0 => {
                    self.min_id_value() + 99999
                }
                _ => self.max_id_value() - 99999,
            };
            let is_in_use: bool =
                self.is_group_entry_sorting_index_in_use(transaction.clone(), group_id_in, index)?;
            if is_in_use {
                let find_unused_result: i64 =
                    self.find_unused_group_sorting_index(transaction.clone(), group_id_in, None)?;
                find_unused_result
            } else {
                index
            }
        };

        let result = self.db_action(transaction.clone(), format!("insert into EntitiesInAGroup (group_id, entity_id, sorting_index) values ({},{},{})",
                                                         group_id_in, contained_entity_id_in, sorting_index).as_str(), false, false);
        if let Err(s) = result {
            // see comments in delete_objects about rollback
            return Err(anyhow!(s));
        }
        // idea: do this check sooner in this method?:
        let mixed_classes_allowed: bool =
            self.are_mixed_classes_allowed(transaction.clone(), &group_id_in)?;
        if !mixed_classes_allowed && self.has_mixed_classes(transaction.clone(), &group_id_in)? {
            // see comments in delete_objects about rollback
            return Err(anyhow!(Util::MIXED_CLASSES_EXCEPTION.to_string()));
        }
        if !caller_manages_transactions_in {
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
        Ok(())
    }

    /// Returns the created row's id.
    fn create_entity(
        &self,
        // purpose: see comment in delete_objects
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        name_in: &str,
        class_id_in: Option<i64>,   /*= None*/
        is_public_in: Option<bool>, /*= None*/
    ) -> Result<i64, anyhow::Error> {
        let name: String = Self::escape_quotes_etc(name_in.to_string());
        if name.is_empty() {
            return Err(anyhow!(
                "In create_entity, name must have a value.".to_string()
            ));
        }
        let id: i64 = self.get_new_key(transaction.clone(), "EntityKeySequence")?;
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
        self.db_action(transaction.clone(), sql.as_str(), false, false)?;
        Ok(id)
    }

    fn create_relation_type<'a>(
        &'a self,
        // purpose: see comment in delete_objects
        caller_manages_transactions_in: bool,
        // purpose: see comment in delete_objects
        transaction_in: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
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

        //BEGIN COPY/PASTED/DUPLICATED (except "in <fn_name>" in 2 Err msgs below) BLOCK-----------------------------------
        // Try creating a local transaction whether we use it or not, to handle compiler errors
        // about variable moves. I'm not seeing a better way to get around them by just using
        // conditions and an Option (many errors):
        // (I tried putting this in a function, then a macro, but it gets compile errors.
        // So, copy/pasting this, unfortunately, until someone thinks of a better way. (You
        // can see the macro, and one of the compile errors, in the commit of 2023-05-18.
        // I didn't try a proc macro but based on some reading I think it would have the same
        // problem.)
        let local_tx: Transaction<Postgres> = {
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
        let local_tx_option = Some(Rc::new(RefCell::new(local_tx)));
        let transaction = if caller_manages_transactions_in {
            transaction_in
        } else {
            local_tx_option
        };
        //END OF COPY/PASTED/DUPLICATED BLOCK----------------------------------

        let mut result: Result<u64, anyhow::Error>;
        let mut id: i64 = 0;
        //see comment at loop in fn create_tables
        loop {
            id = match self.get_new_key(transaction.clone(), "EntityKeySequence") {
                Err(s) => {
                    result = Err(anyhow!(s.to_string()));
                    break;
                }
                Ok(i) => i,
            };
            result = self.db_action(
                transaction.clone(),
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
            result = self.db_action(transaction.clone(),
                                    format!("INSERT INTO RelationType (entity_id, name_in_reverse_direction, directionality) VALUES ({},'{}','{}')",
                                            id, name_in_reverse_direction, directionality).as_str(), false, false);
            if result.is_err() {
                break;
            }
            if !caller_manages_transactions_in {
                // see comments at similar location in delete_objects about local_tx
                // see comments in delete_objects about rollback
                let local_tx_cell: Option<RefCell<Transaction<Postgres>>> =
                    Rc::into_inner(transaction.unwrap());
                match local_tx_cell {
                    Some(t) => {
                        let unwrapped_local_tx = t.into_inner();
                        if let Err(e) = self.commit_trans(unwrapped_local_tx) {
                            return Err(anyhow!("In create_relation_type (2), {}", e.to_string()));
                        }
                    }
                    None => {
                        return Err(anyhow!("Unexpectedly found None instead of Some<RefCell<Transaction<Postgres>>>. How?"));
                    }
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
        transaction_in: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        id_in: i64,
        // purpose: see comment in delete_objects
        caller_manages_transactions_in: bool, /*= false*/
    ) -> Result<(), anyhow::Error> {
        // idea: (also on task list i think but) we should not delete entities until dealing with their use as attr_type_ids etc!
        // (or does the DB's integrity constraints do that for us?)

        //BEGIN COPY/PASTED/DUPLICATED (except "in <fn_name>" in 2 Err msgs below) BLOCK-----------------------------------
        // Try creating a local transaction whether we use it or not, to handle compiler errors
        // about variable moves. I'm not seeing a better way to get around them by just using
        // conditions and an Option (many errors):
        // (I tried putting this in a function, then a macro, but it gets compile errors.
        // So, copy/pasting this, unfortunately, until someone thinks of a better way. (You
        // can see the macro, and one of the compile errors, in the commit of 2023-05-18.
        // I didn't try a proc macro but based on some reading I think it would have the same
        // problem.)
        let local_tx: Transaction<Postgres> = {
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
        let local_tx_option = Some(Rc::new(RefCell::new(local_tx)));
        let transaction = if caller_manages_transactions_in {
            transaction_in
        } else {
            local_tx_option
        };
        //END OF COPY/PASTED/DUPLICATED BLOCK----------------------------------

        self.delete_objects(
            transaction.clone(),
            "EntitiesInAGroup",
            format!("where entity_id={}", id_in).as_str(),
            0,
            true,
        )?;
        self.delete_objects(
            transaction.clone(),
            Util::ENTITY_TYPE,
            format!("where id={}", id_in).as_str(),
            1,
            true,
        )?;
        self.delete_objects(
            transaction.clone(),
            "AttributeSorting",
            format!("where entity_id={}", id_in).as_str(),
            0,
            true,
        )?;
        if !caller_manages_transactions_in {
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
        Ok(())
    }

    fn archive_entity<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        id_in: i64,
    ) -> Result<u64, anyhow::Error> {
        self.delete_object_by_id(transaction, Util::QUANTITY_TYPE, id_in, false)
    }

    fn delete_text_attribute<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        id_in: i64,
    ) -> Result<u64, anyhow::Error> {
        self.delete_object_by_id(transaction, Util::TEXT_TYPE, id_in, false)
    }

    fn delete_date_attribute<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        id_in: i64,
    ) -> Result<u64, anyhow::Error> {
        self.delete_object_by_id(transaction, Util::DATE_TYPE, id_in, false)
    }

    fn delete_boolean_attribute<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        id_in: i64,
    ) -> Result<u64, anyhow::Error> {
        self.delete_object_by_id(transaction, Util::BOOLEAN_TYPE, id_in, false)
    }

    fn delete_file_attribute<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        id_in: i64,
    ) -> Result<u64, anyhow::Error> {
        self.delete_object_by_id(transaction, Util::FILE_TYPE, id_in, false)
    }

    fn delete_relation_to_local_entity<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
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
        let tx = self.begin_trans()?;
        let transaction = Some(Rc::new(RefCell::new(tx)));
        let entity_count: u64 = self.get_group_size(transaction.clone(), id_in, 3)?;
        self.delete_objects(
            transaction.clone(),
            "EntitiesInAGroup",
            format!("where group_id={}", id_in).as_str(),
            entity_count,
            true,
        )?;
        let num_groups: u64 = self
            .get_relation_to_group_count_by_group(transaction.clone(), id_in)?
            .try_into()?;
        self.delete_objects(
            transaction.clone(),
            Util::RELATION_TO_GROUP_TYPE,
            format!("where group_id={}", id_in).as_str(),
            num_groups,
            true,
        )?;
        self.delete_objects(
            transaction.clone(),
            "grupo",
            format!("where id={}", id_in).as_str(),
            1,
            true,
        )?;

        let local_tx_cell: Option<RefCell<Transaction<Postgres>>> =
            Rc::into_inner(transaction.unwrap());
        match local_tx_cell {
            Some(t) => {
                let unwrapped_local_tx = t.into_inner();
                if let Err(e) = self.commit_trans(unwrapped_local_tx) {
                    return Err(anyhow!(e.to_string()));
                }
                Ok(())
            }
            None => {
                return Err(anyhow!(
                    "Unexpectedly found None instead of Some<RefCell<Transaction<Postgres>>>. How?"
                ));
            }
        }
    }

    fn remove_entity_from_group<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
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
        let tx = self.begin_trans()?;
        let transaction = Some(Rc::new(RefCell::new(tx)));
        let entity_count = self.get_group_size(transaction.clone(), group_id_in, 3)?;
        let (deletions1, deletions2) =
            self.delete_relation_to_group_and_all_recursively(transaction.clone(), group_id_in)?;
        if deletions1.checked_add(deletions2).unwrap() != entity_count {
            return Err(anyhow!(
                "Not proceeding: deletions1 {} + deletions2 {} != entity_count {}.",
                deletions1,
                deletions2,
                entity_count
            ));
        }
        let local_tx_cell: Option<RefCell<Transaction<Postgres>>> =
            Rc::into_inner(transaction.unwrap());
        match local_tx_cell {
            Some(t) => {
                let unwrapped_local_tx = t.into_inner();
                if let Err(e) = self.commit_trans(unwrapped_local_tx) {
                    return Err(anyhow!(e.to_string()));
                }
                Ok(())
            }
            None => {
                return Err(anyhow!(
                    "Unexpectedly found None instead of Some<RefCell<Transaction<Postgres>>>. How?"
                ));
            }
        }
    }

    fn delete_relation_type<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        name_in: &str,
        value_in: bool,
    ) -> Result<(), anyhow::Error> {
        let preferences_container_id: i64 =
            self.get_preferences_container_id(transaction.clone())?;
        let result = self.get_user_preference2(
            transaction.clone(),
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
                _ => return Err(anyhow!("How did we get here for {:?}?", result[0])),
            };

            let mut attribute = BooleanAttribute::new2(
                self as &dyn Database,
                transaction.clone(),
                preference_attribute_id,
            )?;
            // Now we have found a boolean attribute which already existed, and just need to
            // update its boolean value. The other values we read from the db inside the first call
            // to something like "get_parent_id()", and just write them back with the new boolean value,
            // to conveniently reuse existing methods.
            self.update_boolean_attribute(
                transaction.clone(),
                attribute.get_id(),
                attribute.get_parent_id(transaction.clone())?,
                attribute.get_attr_type_id(transaction.clone())?,
                value_in,
                attribute.get_valid_on_date(transaction.clone())?,
                attribute.get_observation_date(transaction.clone())?,
            )
        } else {
            let type_id_of_the_has_relation =
                self.find_relation_type(transaction.clone(), Util::THE_HAS_RELATION_TYPE_NAME)?;
            let preference_entity_id: i64 = self
                .create_entity_and_relation_to_local_entity(
                    transaction.clone(),
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
                transaction.clone(),
                preference_entity_id,
                preference_entity_id,
                value_in,
                Some(Utc::now().timestamp_millis()),
                Utc::now().timestamp_millis(),
                None,
                true,
            )?;
            Ok(())
        }
    }
    fn get_user_preference_boolean<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        preference_name_in: &str,
        default_value_in: Option<bool>, /*= None*/
    ) -> Result<Option<bool>, anyhow::Error> {
        let pref: Vec<DataType> = self.get_user_preference2(
            transaction.clone(),
            self.get_preferences_container_id(transaction.clone())?,
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
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        name_in: &str,
        entity_id_in: i64,
    ) -> Result<(), anyhow::Error> {
        let preferences_container_id: i64 =
            self.get_preferences_container_id(transaction.clone())?;
        let pref: Vec<DataType> = self.get_user_preference2(
            transaction.clone(),
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
                transaction.clone(),
                relation_type_id,
                entity_id1,
                entity_id2,
            )?;
            // (Using entity_id1 instead of (the likely identical) preferences_container_id, in case this RTE was originally found down among some
            // nested preferences (organized for user convenience) under here, in order to keep that organization.)
            self.create_relation_to_local_entity(
                transaction.clone(),
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
                self.find_relation_type(transaction.clone(), Util::THE_HAS_RELATION_TYPE_NAME)?;
            let preference_entity_id: i64 = self
                .create_entity_and_relation_to_local_entity(
                    transaction.clone(),
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
                transaction.clone(),
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
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        preference_name_in: &str,
        default_value_in: Option<i64>, /*= None*/
    ) -> Result<Option<i64>, anyhow::Error> {
        let pref = self.get_user_preference2(
            transaction.clone(),
            self.get_preferences_container_id(transaction.clone())?,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<i64, anyhow::Error> {
        let related_entity_id = self.get_relation_to_local_entity_by_name(
            transaction.clone(),
            self.get_system_entity_id(transaction.clone())?,
            Util::USER_PREFERENCES,
        )?;
        match related_entity_id {
            None => return Err(anyhow!("In get_preferences_container_id, This should never happen: method create_and_check_expected_data should be run at startup to create this part of the data.".to_string())),
            Some(id) => Ok(id),
        }
    }
    fn get_entity_count(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction_in: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        entity_id_or_group_id_in: i64,
        caller_manages_transactions_in: bool,    /*= false*/
        is_entity_attrs_not_group_entries: bool, /*= true*/
    ) -> Result<(), anyhow::Error> {
        //This used to be called "renumberAttributeSortingIndexes" before it was merged with "renumberGroupSortingIndexes" (very similar).
        let number_of_entries: u64 = {
            if is_entity_attrs_not_group_entries {
                self.get_attribute_count(transaction_in.clone(), entity_id_or_group_id_in, true)?
            } else {
                self.get_group_size(transaction_in.clone(), entity_id_or_group_id_in, 3)?
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

            //BEGIN COPY/PASTED/DUPLICATED (except "in <fn_name>" in 2 Err msgs below) BLOCK-----------------------------------
            // Try creating a local transaction whether we use it or not, to handle compiler errors
            // about variable moves. I'm not seeing a better way to get around them by just using
            // conditions and an Option (many errors):
            // (I tried putting this in a function, then a macro, but it gets compile errors.
            // So, copy/pasting this, unfortunately, until someone thinks of a better way. (You
            // can see the macro, and one of the compile errors, in the commit of 2023-05-18.
            // I didn't try a proc macro but based on some reading I think it would have the same
            // problem.)
            let local_tx: Transaction<Postgres> = {
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
            let local_tx_option = Some(Rc::new(RefCell::new(local_tx)));
            let transaction = if caller_manages_transactions_in {
                transaction_in
            } else {
                local_tx_option
            };
            //END OF COPY/PASTED/DUPLICATED BLOCK----------------------------------

            let data: Vec<Vec<Option<DataType>>> = {
                if is_entity_attrs_not_group_entries {
                    self.get_entity_attribute_sorting_data(
                        transaction.clone(),
                        entity_id_or_group_id_in,
                        None,
                    )?
                } else {
                    self.get_group_entries_data(
                        transaction.clone(),
                        entity_id_or_group_id_in,
                        None,
                        true,
                    )?
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
                        transaction.clone(),
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
                        transaction.clone(),
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
                        transaction.clone(),
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
                        transaction.clone(),
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
        }
        Ok(())
    }

    /// Excludes those entities that are really relationtypes, attribute types, or quantity units.
    /// The parameter limit_by_class decides whether any limiting is done at all: if true, the query is
    /// limited to entities having the class specified by in_class_id (even if that is None).
    /// The parameter template_entity *further* limits, if limit_by_class is true, by omitting the template_entity from the results (ex., to help avoid
    /// counting that one when deciding whether it is OK to delete the class).
    fn get_entities_only_count(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<u64, anyhow::Error> {
        self.extract_row_count_from_count_query(transaction, "select count(1) from RelationType")
    }

    /// @return the id of the new RTE
    fn add_has_relation_to_local_entity(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        from_entity_id_in: i64,
        to_entity_id_in: i64,
        valid_on_date_in: Option<i64>,
        observation_date_in: i64,
        sorting_index_in: Option<i64>, /*= None*/
    ) -> Result<RelationToLocalEntity, anyhow::Error> {
        let relation_type_id: i64 =
            self.find_relation_type(transaction.clone(), Util::THE_HAS_RELATION_TYPE_NAME)?;
        let new_rte = self.create_relation_to_local_entity(
            //%%latertrans1
            //transaction.clone(),
            None,
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
    fn get_attribute_count(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        entity_id_in: i64,
        include_archived_entities_in: bool, /*= false*/
    ) -> Result<u64, anyhow::Error> {
        let total = self
            .get_quantity_attribute_count(transaction.clone(), entity_id_in)?
            .checked_add(self.get_text_attribute_count(transaction.clone(), entity_id_in)?)
            .unwrap()
            .checked_add(self.get_date_attribute_count(transaction.clone(), entity_id_in)?)
            .unwrap()
            .checked_add(self.get_boolean_attribute_count(transaction.clone(), entity_id_in)?)
            .unwrap()
            .checked_add(self.get_file_attribute_count(transaction.clone(), entity_id_in)?)
            .unwrap()
            .checked_add(self.get_relation_to_local_entity_count(
                transaction.clone(),
                entity_id_in,
                include_archived_entities_in,
            )?)
            .unwrap()
            .checked_add(
                self.get_relation_to_remote_entity_count(transaction.clone(), entity_id_in)?,
            )
            .unwrap()
            .checked_add(self.get_relation_to_group_count(transaction.clone(), entity_id_in)?)
            .unwrap();
        Ok(total)
    }

    fn get_relation_to_local_entity_count(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
    fn get_relations_to_group_containing_this_group(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        group_id_in: i64,
        _starting_index_in: i64,
        _max_vals_in: Option<u64>, /*= None*/
    ) -> Result<Vec<RelationToGroup>, anyhow::Error> {
        let af_id = self.get_attribute_form_id(Util::RELATION_TO_GROUP_TYPE)?;
        let sql = format!("select rtg.id, rtg.entity_id, rtg.rel_type_id, rtg.group_id, rtg.valid_on_date, rtg.observation_date, \
                 asort.sorting_index from RelationToGroup rtg, AttributeSorting asort where group_id={} \
                 and rtg.entity_id=asort.entity_id and asort.attribute_form_id={} \
                 and rtg.id=asort.attribute_id", group_id_in, af_id);
        let early_results =
            self.db_query(transaction, sql.as_str(), "i64,i64,i64,i64,i64,i64,i64")?;
        let mut final_results: Vec<RelationToGroup> = Vec::new();
        // idea: should the remainder of this method be moved to RelationToGroup, so the persistence layer doesn't know anything about the Model? (helps avoid
        // circular dependencies? is a cleaner design, at least if RTG were in a separate library?)
        let early_results_len = early_results.len();
        for result in early_results {
            // None of these values should be of "None" type, so not checking for that. If they are it's a bug:
            //final_results.add(result(0).get.asInstanceOf[i64], new Entity(this, result(1).get.asInstanceOf[i64]))
            let id = match result[0] {
                Some(DataType::Bigint(x)) => x,
                _ => return Err(anyhow!("How did we get here for {:?}?", result[0])),
            };
            let entity_id = match result[1] {
                Some(DataType::Bigint(x)) => x,
                _ => return Err(anyhow!("How did we get here for {:?}?", result[1])),
            };
            let rel_type_id = match result[2] {
                Some(DataType::Bigint(x)) => x,
                _ => return Err(anyhow!("How did we get here for {:?}?", result[2])),
            };
            let group_id = match result[3] {
                Some(DataType::Bigint(x)) => x,
                _ => return Err(anyhow!("How did we get here for {:?}?", result[3])),
            };
            //%%%%% fix this next part after figuring out about what happens when querying a null back, in pg.db_query etc!
            // valid_on_date: Option<i64> /*%%= None*/,
            /*DataType::Bigint(%%)*/
            let valid_on_date = None; //match result[4] {
                                      //     DataType::Bigint(x) => x,
                                      //     _ => return Err(anyhow!("How did we get here for {:?}?", result[4])),
                                      // };
            let observation_date = match result[5] {
                Some(DataType::Bigint(x)) => x,
                _ => return Err(anyhow!("How did we get here for {:?}?", result[5])),
            };
            let sorting_index = match result[6] {
                Some(DataType::Bigint(x)) => x,
                _ => return Err(anyhow!("How did we get here for {:?}?", result[6])),
            };
            let rtg: RelationToGroup = RelationToGroup::new(
                self,
                id,
                entity_id,
                rel_type_id,
                group_id,
                valid_on_date,
                observation_date,
                sorting_index,
            );
            final_results.push(rtg)
        }
        if !(final_results.len() == early_results_len) {
            return Err(anyhow!("In get_relations_to_group_containing_this_group, Final results ({}) do not match count of early_results ({})", final_results.len(), early_results_len));
        }
        Ok(final_results)
    }

    fn get_group_count(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<u64, anyhow::Error> {
        self.extract_row_count_from_count_query(transaction, "select count(1) from grupo")
    }

    /// @param group_id_in group_id
    /// @param include_which_entities_in 1/2/3 means select onlyNon-archived/onlyArchived/all entities, respectively.
    ///                                4 means "it depends on the value of include_archived_entities", which is what callers want in some cases.
    ///                                This param might be made more clear, but it is not yet clear how is best to do that.
    ///                                  Because the caller provides this switch specifically to the situation, the logic is not necessarily overridden
    ///                                internally based on the value of this.include_archived_entities.
    /// Idea: maybe it should be turned into an enum.  Probably.
    fn get_group_size(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        group_id_in: i64,
        include_which_entities_in: i32, /*= 3*/
    ) -> Result<u64, anyhow::Error> {
        //idea: convert this 1-4 to an enum?
        if include_which_entities_in <= 0 || include_which_entities_in >= 5 {
            return Err(anyhow!("Variable include_which_entities_in ({}) is out of the expected range of 1-4; there is a bug.", include_which_entities_in));
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
                return Err(anyhow!(
                    "How did we get here? include_which_entities={}",
                    include_which_entities_in
                ))
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        group_id_in: i64,
        limit_in: Option<i64>, /*= Some(5)*/
    ) -> Result<Vec<Vec<Option<DataType>>>, anyhow::Error> {
        //get every entity that contains a rtg that contains this group:
        let limit = Self::check_if_should_be_all_results(limit_in);
        let containing_entity_id_list: Vec<Vec<Option<DataType>>> =
            self.db_query(transaction.clone(),
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
                transaction.clone(),
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        quantity_id_in: i64,
    ) -> Result<Vec<Option<DataType>>, anyhow::Error> {
        let af_id = self.get_attribute_form_id(Util::QUANTITY_TYPE)?;
        self.db_query_wrapper_for_one_row(transaction,
                                          format!("select qa.entity_id, qa.unit_id, qa.attr_type_id, asort.sorting_index, \
                                          qa.valid_on_date, qa.observation_date, qa.quantity_number \
                                       from QuantityAttribute qa, AttributeSorting asort where qa.id={} and qa.entity_id=asort.entity_id and \
                                       asort.attribute_form_id={} and qa.id=asort.attribute_id", quantity_id_in, af_id).as_str(),
                                          Util::GET_QUANTITY_ATTRIBUTE_DATA__RESULT_TYPES)
    }

    fn get_relation_to_local_entity_data(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id_in: i64,
    ) -> Result<Vec<Option<DataType>>, anyhow::Error> {
        self.db_query_wrapper_for_one_row(transaction,
                                          format!("select name, insertion_date, allow_mixed_classes, new_entries_stick_to_top from grupo where id={}",
                                                  id_in).as_str(),
                                          Util::GET_GROUP_DATA__RESULT_TYPES)
    }

    fn get_relation_to_group_data_by_keys(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        text_id_in: i64,
    ) -> Result<Vec<Option<DataType>>, anyhow::Error> {
        let af_id = self.get_attribute_form_id(Util::TEXT_TYPE)?;
        self.db_query_wrapper_for_one_row(transaction,
                                          format!("select ta.entity_id, ta.textvalue, ta.attr_type_id, asort.sorting_index, \
                                          ta.valid_on_date, ta.observation_date \
                             from TextAttribute ta, AttributeSorting asort where id={} and ta.entity_id=asort.entity_id \
                             and asort.attribute_form_id={} and ta.id=asort.attribute_id",
                                                  text_id_in, af_id).as_str(),
                                          Util::GET_TEXT_ATTRIBUTE_DATA__RESULT_TYPES)
    }

    fn get_date_attribute_data(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        boolean_id_in: i64,
    ) -> Result<Vec<Option<DataType>>, anyhow::Error> {
        let form_id = self.get_attribute_form_id(Util::BOOLEAN_TYPE)?;
        self.db_query_wrapper_for_one_row(transaction, format!("select ba.entity_id, ba.boolean_value, ba.attr_type_id, asort.sorting_index, ba.valid_on_date, ba.observation_date \
                                    from BooleanAttribute ba, AttributeSorting asort where id={} and ba.entity_id=asort.entity_id and asort.attribute_form_id={} \
                                     and ba.id=asort.attribute_id",
                                                               boolean_id_in, form_id).as_str(),
                                          Util::GET_BOOLEAN_ATTRIBUTE_DATA__RESULT_TYPES)
    }

    fn get_file_attribute_data(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        file_id_in: i64,
    ) -> Result<Vec<Option<DataType>>, anyhow::Error> {
        let af_id = self.get_attribute_form_id(Util::FILE_TYPE)?;
        self.db_query_wrapper_for_one_row(transaction,
                                          format!("select fa.entity_id, fa.description, fa.attr_type_id, asort.sorting_index, fa.original_file_date, fa.stored_date, \
                             fa.original_file_path, fa.readable, fa.writable, fa.executable, fa.size, fa.md5hash \
                              from FileAttribute fa, AttributeSorting asort where id={} and fa.entity_id=asort.entity_id and asort.attribute_form_id={} \
                               and fa.id=asort.attribute_id",
                                                  file_id_in, af_id).as_str(),
                                          Util::GET_FILE_ATTRIBUTE_DATA__RESULT_TYPES)
    }

    fn update_sorting_index_in_a_group(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
    //     // Idea: combine w/ similar logic in FileAttribute::md5_hash?
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id_in: i64,
        include_archived: bool,
    ) -> Result<bool, anyhow::Error> {
        let condition = if !include_archived {
            " and not archived"
        } else {
            ""
        };
        Util::print_backtrace();
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
    /// *****NOTE*****: The limit_by_class:Boolean parameter is not redundant with the in_class_id: in_class_id could be None and we could still want
    /// to select only those entities whose class_id is NULL, such as when enforcing group uniformity (see method has_mixed_classes and its
    /// uses, for more info).
    ///
    /// The parameter omitEntity is (at this writing) used for the id of a class-defining (template) entity, which we shouldn't show for editing when showing all the
    /// entities in the class (editing that is a separate menu option), otherwise it confuses things.
    fn get_entities_only(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
            //%%%% add_new_entity_to_results(final_results, result)
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

    fn get_matching_groups(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        starting_object_index_in: i64,
        max_vals_in: Option<i64>, /*= None*/
        omit_group_id_in: Option<i64>,
        name_regex_in: String,
    ) -> Result<Vec<Group>, anyhow::Error> {
        let name_regex = Self::escape_quotes_etc(name_regex_in);
        let omission_expression = match omit_group_id_in {
            None => "true".to_string(),
            Some(ogi) => format!("(not id={})", ogi),
        };
        let sql = format!("select id, name, insertion_date, allow_mixed_classes, new_entries_stick_to_top from grupo where name ~* '{}' and {} \
                      order by id limit {} offset {}",
                        name_regex, omission_expression, Self::check_if_should_be_all_results(max_vals_in), starting_object_index_in);
        let early_results = self.db_query(transaction, sql.as_str(), "i64,String,i64,bool,bool")?;
        let early_results_len = early_results.len();
        let final_results: Vec<Group> = Vec::new();
        // idea: (see get_entities_generic for idea, see if applies here)
        for _result in early_results {
            // None of these values should be of "None" type, so not checking for that. If they are it's a bug:
            //%%%%
            // final_results.add(new Group(this, result(0).get.asInstanceOf[i64], result(1).get.asInstanceOf[String], result(2).get.asInstanceOf[i64],
            //                            result(3).get.asInstanceOf[Boolean], result(4).get.asInstanceOf[Boolean]))
        }
        if final_results.len() != early_results_len {
            return Err(anyhow!("In get_matching_groups, final_results.len() ({}) != early_results.len() ({}), with sql: {}", final_results.len(), early_results_len, sql));
        }
        Ok(final_results)
    }

    fn get_local_entities_containing_local_entity(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        entity_id_in: i64,
    ) -> Result<(u64, u64), anyhow::Error> {
        let non_archived2 = self.extract_row_count_from_count_query(
            transaction.clone(),
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        group_id_in: i64,
    ) -> Result<(u64, u64), anyhow::Error> {
        let non_archived = self.extract_row_count_from_count_query(transaction.clone(), format!("select count(1) from \
                                relationtogroup rtg, entity e where e.id=rtg.entity_id and not e.archived and rtg.group_id={}", group_id_in).as_str())?;
        let archived = self.extract_row_count_from_count_query(transaction, format!("select count(1) from \
                                relationtogroup rtg, entity e where e.id=rtg.entity_id and e.archived and rtg.group_id={}", group_id_in).as_str())?;
        Ok((non_archived, archived))
    }

    fn get_containing_relations_to_group(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        entity_id_in: i64,
        starting_index_in: i64,
        max_vals_in: Option<i64>, /*= None*/
    ) -> Result<Vec<RelationToGroup>, anyhow::Error> {
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

        let sql = format!("select group_id from entitiesinagroup where entity_id={} order by group_id limit {} offset {}",
                         entity_id_in, Self::check_if_should_be_all_results(max_vals_in), starting_index_in);
        self.get_containing_relation_to_groups_helper(transaction, sql.as_str())
    }

    fn get_count_of_entities_used_as_attribute_types(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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

    // fn get_entities_used_as_attribute_types(&self,
    //    transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    // attribute_type_in: String,
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
    // fn get_groups(&self,
    //    transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    // starting_object_index_in: i64, max_vals_in: Option<i64> /*= None*/,
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
    //         //%%%%%
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
    //         //%%%%%
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        let results = self.db_query(transaction, format!("select eiag.sorting_index from entity e, entitiesinagroup eiag \
                                where e.id=eiag.entity_id \
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        entity_id_in: i64,
        sorting_index_in: i64,
        limit_in: Option<i64>,
        forward_not_back_in: bool,
    ) -> Result<Vec<Vec<Option<DataType>>>, anyhow::Error> {
        // (See comments in getAdjacentGroupEntriesSortingIndexes, at least about the "...archived..." stuff.)
        let rtle_form_id = self.get_attribute_form_id(Util::RELATION_TO_LOCAL_ENTITY_TYPE)?;
        // IDEA: would the query be faster on larger data volumes, if the
        //      not in (select id from relationtoentity rte where entity_id_2 in (select id from entity where archived))
        // ...were replaced with:
        //      not in (select rte.id from relationtoentity rte, entity e where rte.entity_id_2=e.id and e.archived)
        // ...and are those truly equivalent? They yielded the same results in an ad-hoc test like:
        /*  select sorting_index from AttributeSorting asort where attribute_form_id = 6 and asort.entity_id=-9223372036854567954 and asort.sorting_index>-7142999829835153408
            and asort.attribute_id not in (select rte.id from relationtoentity rte, entity e where rte.entity_id_2=e.id and e.archived)
        */
        // ...vs. this (but I did not easily, interactively, observe a performance difference:
        /*
           select sorting_index from AttributeSorting asort where attribute_form_id = 6 and asort.entity_id=-9223372036854567954 and asort.sorting_index>-7142999829835153408
           and asort.attribute_id not in (select id from relationtoentity rte where entity_id_2 in (select id from entity where archived))
        */
        let not_archived = if !self.include_archived_entities {
            "and asort.attribute_id not in \
                (select id from relationtoentity rte where entity_id_2 in (select id from entity where archived)) "
        } else {
            " "
        };
        let results = self.db_query(transaction,
        // NOTE: the 2 main (UNION-ed) sql sections differ by the attribute_form_id and presence/absence of the "not in" stuff.
        // Next query could be faster in the infrequent case of showing archived entities, if we combined the two selects,
        // since it is just doing a UNION of two things where we could remove the condition. But not
        // so for the more likely case of hiding archived entities (and maintenance seems easier as-is).
       format!("select sorting_index from AttributeSorting asort where asort.attribute_form_id={} \
           and asort.entity_id={} and asort.sorting_index {}{} \
           \
           {}
           \
           UNION \
           \
           select sorting_index from AttributeSorting asort where asort.attribute_form_id != {} \
           and asort.entity_id={} and asort.sorting_index {}{} \
           \
           order by sorting_index {} limit {}",
           rtle_form_id,
           entity_id_in,
           if forward_not_back_in { ">" } else { "<" },
           not_archived,
           sorting_index_in,
           rtle_form_id,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        let early_results = self.db_query(transaction.clone(), sql.as_str(), "i64,i64")?;
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
                    final_results.push(Entity::new2(self as &dyn Database, transaction.clone(), i)?)
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id_in: i64,
    ) -> Result<Option<String>, anyhow::Error> {
        let name: Vec<Option<DataType>> = self.get_entity_data(transaction, id_in)?;
        match name.get(0) {
            None => Ok(None),
            Some(Some(DataType::String(x))) => Ok(Some(x.to_string())),
            _ => Err(anyhow!("Unexpected value: {:?}", name)),
        }
    }

    fn get_class_data(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        name_in: &str,
        self_id_to_ignore_in: Option<i64>, /*= None*/
    ) -> Result<bool, anyhow::Error> {
        let first = self.is_duplicate_row(
            transaction.clone(),
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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

    fn get_or_create_class_and_template_entity<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        class_name_in: &str,
        _caller_manages_transactions_in: bool,
    ) -> Result<(i64, i64), anyhow::Error> {
        //(see note above re 'bad smell' in method add_uri_entity_with_uri_attribute.)
        //rollbacketc%%FIX NEXT LINE AFTERI SEE HOW OTHERS DO!
        // if !caller_manages_transactions_in { self.begin_trans() }
        //                    try {
        let (class_id, entity_id) = {
            let found_id: Option<i64> =
                self.find_first_class_id_by_name(transaction.clone(), class_name_in, true)?;
            if found_id.is_some() {
                let entity_id: i64 = EntityClass::new2(
                    self as &dyn Database,
                    transaction.clone(),
                    found_id.unwrap(),
                )?
                //.get_template_entity_id(found_id.get, entity_id)?;
                .get_template_entity_id(transaction.clone())?;
                (found_id.unwrap(), entity_id)
            } else {
                let (class_id, entity_id) =
                    self.create_class_and_its_template_entity(transaction, class_name_in)?;
                (class_id, entity_id)
            }
        };
        //rollbacketc%%FIX NEXT LINE AFTERI SEE HOW OTHERS DO!
        // if !caller_manages_transactions_in {self.commit_trans() }
        Ok((class_id, entity_id))
        //                  }
        //                catch {
        //                case e: Exception =>
        //rollbacketc%%FIX NEXT LINE AFTERI SEE HOW OTHERS DO!
        // if !caller_manages_transactions_in) rollback_trans()
        //                throw e
        //          }
    }

    fn set_include_archived_entities(&mut self, iae_in: bool) {
        self.include_archived_entities = iae_in;
    }

    fn get_om_instance_count(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<u64, anyhow::Error> {
        self.extract_row_count_from_count_query(transaction, "SELECT count(1) from omInstance")
    }

    fn create_om_instance(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id_in: String,
        is_local_in: bool,
        address_in: String,
        entity_id_in: Option<i64>, /*= None*/
        old_table_name: bool,      /*= false*/
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
            return Err(anyhow!(
                "In create_om_instance, Didn't expect quotes etc in the UUID provided: {}",
                id_in
            ));
        };
        if address != address_in {
            return Err(anyhow!(
                "In create_om_instance, didn't expect quotes etc in the address provided: {}",
                address
            ));
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
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
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

    fn id(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<String, anyhow::Error> {
        self.get_local_om_instance_data(transaction)?.get_id()
    }

    fn om_instance_key_exists(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id_in: &str,
    ) -> Result<bool, anyhow::Error> {
        self.does_this_exist(
            transaction,
            format!("SELECT count(1) from omInstance where id='{}'", id_in).as_str(),
            true,
        )
    }

    //%%%%
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
        let oi: OmInstance = db.get_local_om_instance_data;
        let uuid: String = oi.get_id;
        assert(oi.get_local)
        assert(db.om_instance_key_exists(uuid))
        let startingOmiCount = db.get_om_instance_count();
        assert(startingOmiCount > 0)
        let oiAgainAddress = db.get_om_instance_data(uuid)(1).get.asInstanceOf[String];
        assert(oiAgainAddress == Util.LOCAL_OM_INSTANCE_DEFAULT_DESCRIPTION)
        let omInstances: util.ArrayList[OmInstance] = db.get_om_instances();
        assert(omInstances.size == startingOmiCount)
        let sizeNowTrue = db.get_om_instances(Some(true)).size;
        assert(sizeNowTrue > 0)
        // Idea: fix: Next line fails at times, maybe due to code running in parallel between this and RestDatabaseTest, creating/deleting rows.  Only seems to happen
        // when all tests are run, never when the test classes are run separately.
        //    let sizeNowFalse = db.get_om_instances(Some(false)).size;
        //assert(sizeNowFalse < sizeNowTrue)
        assert(! db.om_instance_key_exists(java.util.UUID.randomUUID().toString))
        assert(new OmInstance(db, uuid).get_address == Util.LOCAL_OM_INSTANCE_DEFAULT_DESCRIPTION)

        let uuid2 = java.util.UUID.randomUUID().toString;
        db.create_om_instance(uuid2, is_local_in = false, "om.example.com", Some(db.get_system_entity_id))
        // should have the local one created at db creation, and now the one for this test:
        assert(db.get_om_instance_count() == startingOmiCount + 1)
        let mut i2: OmInstance = new OmInstance(db, uuid2);
        assert(i2.get_address == "om.example.com")
        db.update_om_instance(uuid2, "address", None)
        i2  = new OmInstance(db,uuid2)
        assert(i2.get_address == "address")
        assert(!i2.get_local)
        assert(i2.get_entity_id.isEmpty)
        assert(i2.get_creation_date > 0)
        assert(i2.get_creation_date_formatted.length > 0)
        db.update_om_instance(uuid2, "address", Some(db.get_system_entity_id))
        i2  = new OmInstance(db,uuid2)
        assert(i2.get_entity_id.get == db.get_system_entity_id)
        assert(db.is_duplicate_om_instance_address("address"))
        assert(db.is_duplicate_om_instance_address(Util.LOCAL_OM_INSTANCE_DEFAULT_DESCRIPTION))
        assert(!db.is_duplicate_om_instance_address("address", Some(uuid2)))
        assert(!db.is_duplicate_om_instance_address(Util.LOCAL_OM_INSTANCE_DEFAULT_DESCRIPTION, Some(uuid)))
        let uuid3 = java.util.UUID.randomUUID().toString;
        db.create_om_instance(uuid3, is_local_in = false, "address", Some(db.get_system_entity_id))
        assert(db.is_duplicate_om_instance_address("address", Some(uuid2)))
        assert(db.is_duplicate_om_instance_address("address", Some(uuid3)))
        i2.delete()
        assert(db.is_duplicate_om_instance_address("address"))
        assert(db.is_duplicate_om_instance_address("address", Some(uuid2)))
        assert(!db.is_duplicate_om_instance_address("address", Some(uuid3)))
        assert(intercept[Exception] {
                                      new OmInstance(db, uuid2)
                                    }.getMessage.contains("does not exist"))
      }
    */

    fn update_om_instance(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id_in: String,
        address_in: String,
        entity_id_in: Option<i64>,
    ) -> Result<u64, anyhow::Error> {
        let address: String = Self::escape_quotes_etc(address_in);
        let eid_or_null = match entity_id_in {
            Some(eid) => eid.to_string(),
            _ => "NULL".to_string(),
        };
        let sql = format!(
            "UPDATE omInstance SET (address, entity_id) = ('{}', {}) where id='{}'",
            address, eid_or_null, id_in
        );
        self.db_action(transaction, sql.as_str(), false, false)
    }

    fn delete_om_instance<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        id_in: &str,
    ) -> Result<u64, anyhow::Error> {
        self.delete_object_by_id2(transaction, "omInstance", id_in, false)
    }
}
