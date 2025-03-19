/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2016-2017 inclusive, 2020, and 2023-2025 inclusive, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
use crate::model::attribute::Attribute;
use crate::model::entity::Entity;
use crate::model::group::Group;
use crate::model::relation_to_group::RelationToGroup;
use crate::model::relation_to_local_entity::RelationToLocalEntity;
use crate::model::relation_to_remote_entity::RelationToRemoteEntity;
use crate::model::relation_type::RelationType;
use crate::model::text_attribute::TextAttribute;
use crate::util::Util;
use anyhow::anyhow;
//use mockall::{automock, mock, predicate::*};
use sqlx::{Postgres, Transaction};
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
// use std::string::ToString;
// use crate::model::postgresql_database::PostgreSQLDatabase;

#[derive(Debug, Clone)]
pub enum DataType {
    Float(f64),
    String(String),
    // not supported in return values from sqlx: see db_query in postgresql_database.rs 
    // for another, related comment:
    // UnsignedInt(u64),
    Bigint(i64),
    Boolean(bool),
    Smallint(i32),
}

impl std::fmt::Debug for dyn Database {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // to enhance, see std lib docs for Debug or fmt.
        let id_all: String = match self.id_all(None) {
            Ok(s) => s,
            Err(e) => e.to_string(),
        };
        write!(f, "Database [is_remote: {}, id details: {}]", self.is_remote(), id_all)
    }
}

pub trait Database {
    fn is_remote(&self) -> bool;
    fn id<'a>(
        &self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
    ) -> Result<String, anyhow::Error>;
    fn id_all<'a>(
        &self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
    ) -> Result<String, anyhow::Error>;
    // fn setup_db(&self) -> Result<(), String>;

    fn get_remote_address(&self) -> Option<String> {
        None
    }
    fn include_archived_entities(&self) -> bool;
    fn begin_trans(&self) -> Result<Transaction<Postgres>, anyhow::Error>;
    fn rollback_trans(&self, tx: Transaction<Postgres>) -> Result<(), anyhow::Error>;
    fn commit_trans(&self, tx: Transaction<Postgres>) -> Result<(), anyhow::Error>;

    // where we create the table also calls this.
    // Longer than the old 60 (needed), and a likely familiar length to many people (for ease in knowing when done), seems a decent balance. If any longer
    // is needed, maybe it should be put in a TextAttribute and make those more convenient to use, instead.
    // (See usages. The DNS hostname max size seems to be 255 plus 1 null, but the ":<port>" part could add 6 more chars (they seem to go up to :65535).
    // Idea: Maybe someday we will have to move to a larger or flexible size in case it changes or uses unicode or I don't know what.)
    fn om_instance_address_length(&self) -> i32 {
        262
    }

    /// This has &self as a parameter to avoid a compiler error about Database not being able to
    /// be made into an object, unless it is there.
    fn get_attribute_form_id(&self, key: &str) -> Result<i32, anyhow::Error> {
        //MAKE SURE THESE MATCH WITH THOSE IN attribute_key_exists and get_attribute_form_name, and the range in the db constraint valid_attribute_form_id ,
        // and in RestDatabase.process_array_of_tuples_and_int !
        let res = match key {
            Util::QUANTITY_TYPE => 1,
            Util::DATE_TYPE => 2,
            Util::BOOLEAN_TYPE => 3,
            Util::FILE_TYPE => 4,
            Util::TEXT_TYPE => 5,
            Util::RELATION_TO_LOCAL_ENTITY_TYPE => 6,
            "RelationToLocalEntity" => 6,
            Util::RELATION_TO_GROUP_TYPE => 7,
            Util::RELATION_TO_REMOTE_ENTITY_TYPE => 8,
            _ => {
                return Err(anyhow!(
                    "Unexpected key name in get_attribute_form_id: {}",
                    key
                ))
            }
        };
        Ok(res)
    }
    fn get_attribute_form_name(&self, key: i32) -> Result<&str, anyhow::Error> {
        // MAKE SURE THESE MATCH WITH THOSE IN get_attribute_form_id !
        //idea: put these values in a structure that is looked up both ways, instead of duplicating them?
        let res = match key {
            1 => Util::QUANTITY_TYPE,
            2 => Util::DATE_TYPE,
            3 => Util::BOOLEAN_TYPE,
            4 => Util::FILE_TYPE,
            5 => Util::TEXT_TYPE,
            6 => Util::RELATION_TO_LOCAL_ENTITY_TYPE,
            7 => Util::RELATION_TO_GROUP_TYPE,
            8 => Util::RELATION_TO_REMOTE_ENTITY_TYPE,
            _ => {
                return Err(anyhow!(
                    "Unexpected key value in get_attribute_form_name: {}",
                    key
                ))
            }
        };
        Ok(res)
    }

    /// This has &self as a parameter to avoid a compiler error about Database not being able to
    /// be made into an object, unless it is there.
    fn max_id_value(&self) -> i64 {
        // Max size for a Java long type, a Rust i64, and for a postgresql 7.2.1 bigint type (which is being used, at the moment, for the id value in Entity table.
        // (these values are from file:///usr/share/doc/postgresql-doc-9.1/html/datatype-numeric.html)
        // 9223372036854775807L I think: confirm it & below.
        i64::MAX
    }

    /// This has &self as a parameter to avoid a compiler error about Database not being able to
    /// be made into an object, unless it is there.
    fn min_id_value(&self) -> i64 {
        //-9223372036854775808L I think: confirm it & above.
        i64::MIN
    }

    fn create_boolean_attribute<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        parent_id_in: i64,
        attr_type_id_in: i64,
        boolean_in: bool,
        valid_on_date_in: Option<i64>,
        observation_date_in: i64,
        sorting_index_in: Option<i64>, /*= None, ie, default or value to pass if irrelevant.  Was default for no parm, in scala version.*/
    ) -> Result<i64, anyhow::Error>;
    fn create_text_attribute<'a, 'b>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'b, Postgres>>>>,
        parent_id_in: i64,
        attr_type_id_in: i64,
        text_in: &str,
        valid_on_date_in: Option<i64>, /*= None*/
        observation_date_in: i64,      /*= System.currentTimeMillis()*/
        sorting_index_in: Option<i64>, /*= None*/
    ) -> Result<i64, anyhow::Error>
    where
        'a: 'b;
    fn create_relation_to_local_entity<'a, 'b>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'b, Postgres>>>>,
        relation_type_id_in: i64,
        entity_id1_in: i64,
        entity_id2_in: i64,
        valid_on_date_in: Option<i64>,
        observation_date_in: i64,
        sorting_index_in: Option<i64>, /* = None*/
    ) -> Result<(i64, i64), anyhow::Error>
    where
        'a: 'b;
    fn create_relation_to_remote_entity<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        relation_type_id_in: i64,
        entity_id1_in: i64,
        entity_id2_in: i64,
        valid_on_date_in: Option<i64>,
        observation_date_in: i64,
        remote_instance_id_in: &str,
        sorting_index_in: Option<i64>, /* = None*/
    ) -> Result<RelationToRemoteEntity, anyhow::Error>;
    fn create_group_and_relation_to_group<'a, 'b>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'b, Postgres>>>>,
        entity_id_in: i64,
        relation_type_id_in: i64,
        new_group_name_in: &str,
        allow_mixed_classes_in_group_in: bool, /*= false*/
        valid_on_date_in: Option<i64>,
        observation_date_in: i64,
        sorting_index_in: Option<i64>,
    ) -> Result<(i64, i64), anyhow::Error>
    where
        'a: 'b;

    fn create_entity<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        name_in: &str,
        class_id_in: Option<i64>,   /*= None*/
        is_public_in: Option<bool>, /* = None*/
    ) -> Result<i64, anyhow::Error>;
    fn create_entity_and_relation_to_local_entity<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        entity_id_in: i64,
        relation_type_id_in: i64,
        new_entity_name_in: &str,
        is_public_in: Option<bool>,
        valid_on_date_in: Option<i64>,
        observation_date_in: i64,
    ) -> Result<(i64, i64), anyhow::Error>;
    fn create_relation_to_group<'a, 'b>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'b, Postgres>>>>,
        entity_id_in: i64,
        relation_type_id_in: i64,
        group_id_in: i64,
        valid_on_date_in: Option<i64>,
        observation_date_in: i64,
        sorting_index_in: Option<i64>, /*= None*/
    ) -> Result<(i64, i64), anyhow::Error>
    where
        'a: 'b;
    fn add_entity_to_group<'a, 'b>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'b, Postgres>>>>,
        group_id_in: i64,
        contained_entity_id_in: i64,
        sorting_index_in: Option<i64>, /*= None*/
    ) -> Result<(), anyhow::Error>
    where
        'a: 'b;
    fn create_om_instance<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id_in: String,
        is_local_in: bool,
        address_in: String,
        entity_id_in: Option<i64>, /*= None*/
        old_table_name: bool,      /* = false*/
    ) -> Result<i64, anyhow::Error>;
    fn create_relation_type<'a, 'b>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'b, Postgres>>>>,
        name_in: &str,
        name_in_reverse_direction_in: &str,
        directionality_in: &str,
    ) -> Result<i64, anyhow::Error>
    where
        'a: 'b;
    fn create_class_and_its_template_entity<'a, 'b>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'b, Postgres>>>>,
        class_name_in: &str,
    ) -> Result<(i64, i64), anyhow::Error>
    where
        'a: 'b;
    fn find_contained_local_entity_ids<'a, 'b>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        results_in_out: &'b mut HashSet<i64>,
        from_entity_id_in: i64,
        search_string_in: &str,
        levels_remaining: i32,      /* = 20*/
        stop_after_any_found: bool, /* = true*/
    ) -> Result<&'b mut HashSet<i64>, anyhow::Error>
    where
        'b: 'a;
    fn entity_key_exists(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id_in: i64,
        include_archived: bool, /*= true*/
    ) -> Result<bool, anyhow::Error>;
    fn boolean_attribute_key_exists(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id_in: i64,
    ) -> Result<bool, anyhow::Error>;
    fn get_entity_data(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id_in: i64,
    ) -> Result<Vec<Option<DataType>>, anyhow::Error>;
    fn get_entity_name(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id_in: i64,
    ) -> Result<Option<String>, anyhow::Error>;
    fn find_relation_type(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        type_name_in: &str,
    ) -> Result<i64, anyhow::Error>;
    fn get_boolean_attribute_data(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id_in: i64,
    ) -> Result<Vec<Option<DataType>>, anyhow::Error>;
    fn get_group_size(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        group_id_in: i64,
        include_which_entities_in: i32, /* = 3*/
    ) -> Result<u64, anyhow::Error>;

    /*%%
          pub fn getRestDatabase(remoteAddressIn: String) -> RestDatabase {
            new RestDatabase(remoteAddressIn)
          }

          //%%should this be here or somewhere else, given what it depends on? See where used.
          pub fn currentOrRemoteDb(relationToEntityIn: Attribute, currentDb: Database) -> Database {
            require(relationToEntityIn.isInstanceOf[RelationToLocalEntity] || relationToEntityIn.isInstanceOf[RelationToRemoteEntity])

            // Can't use ".is_remote" here because a RelationToRemoteEntity is stored locally (so would say false),
            // but refers to an entity which is remote (so we want the next line to be true in that case):
            //noinspection TypeCheckCanBeMatch
            if relationToEntityIn.isInstanceOf[RelationToRemoteEntity]) {
              relationToEntityIn.asInstanceOf[RelationToRemoteEntity].getRemoteDatabase
            } else if relationToEntityIn.isInstanceOf[RelationToLocalEntity]) {
              currentDb
            } else throw new OmDatabaseException("Unexpected type: " + relationToEntityIn.getClass.getCanonicalName)
          }
    */

    /// %%Many of these methods were marked "protected[model]" in scala, for 2 reasons:
    ///      1) to minimize the risk of calling db.<method> on the wrong db, when the full model object (like Entity) would contain the right db for itself
    ///         (ie, what if one called db.delete and the same entity id # exists in both databases), and
    ///      2) to generally manage the coupling between the Controller and model package, since it seems cleaner to go through model objects when they can
    ///         call the db for themselves, rather than everything touching the db entrails directly.
    ///    ...but should be avoided when going through the model object (like Entity) causes enough more db hits to not be worth it (performance vs.
    ///    clarity & ease of maintenance).
    fn create_quantity_attribute<'a, 'b>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'b, Postgres>>>>,
        parent_id_in: i64,
        attr_type_id_in: i64,
        unit_id_in: i64,
        number_in: f64,
        valid_on_date_in: Option<i64>,
        observation_date_in: i64,
        sorting_index_in: Option<i64>, /*= None*/
    ) -> Result<i64, anyhow::Error>
    where
        'a: 'b;
    fn create_date_attribute<'a>(
        &'a self,
        transaction_in: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        parent_id_in: i64,
        attr_type_id_in: i64,
        date_in: i64,
        sorting_index_in: Option<i64>, /*= None*/
    ) -> Result<i64, anyhow::Error>;
    //%%
    // fn create_file_attribute(&self,
    //                          parent_id_in: i64, attr_type_id_in: i64, description_in: String,
    //                          original_file_date_in: i64, stored_date_in: i64,
    //                         original_file_path_in: String, readable_in: bool, writable_in: bool,
    //                          executable_in: bool, size_in: i64, md5_hash_in: String,
    //                          inputStreamIn: java.io.FileInputStream,
    //                          sorting_index_in: Option<i64> /*= None*/) -> /*id*/ Result<i64, anyhow::Error>;
    fn add_has_relation_to_local_entity<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        from_entity_id_in: i64,
        to_entity_id_in: i64,
        valid_on_date_in: Option<i64>,
        observation_date_in: i64,
        sorting_index_in: Option<i64>, /*= None*/
    ) -> Result<(i64, i64, i64), anyhow::Error>;
    fn get_or_create_class_and_template_entity<'a, 'b>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'b, Postgres>>>>,
        class_name_in: &str,
    ) -> Result<(i64, i64), anyhow::Error>
    where
        'a: 'b;
    fn add_uri_entity_with_uri_attribute<'a, 'b>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'b, Postgres>>>>,
        containing_entity_id_in: i64,
        new_entity_name_in: &str,
        uri_in: &str,
        observation_date_in: i64,
        make_them_public_in: Option<bool>,
        quote_in: Option<&str>, /*= None*/
    ) -> Result<(i64, i64), anyhow::Error>
    where
        'a: 'b;
    fn attribute_key_exists(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        form_id_in: i64,
        id_in: i64,
    ) -> Result<bool, anyhow::Error>;
    fn relation_type_key_exists(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id_in: i64,
    ) -> Result<bool, anyhow::Error>;
    fn quantity_attribute_key_exists(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id_in: i64,
    ) -> Result<bool, anyhow::Error>;
    fn date_attribute_key_exists(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id_in: i64,
    ) -> Result<bool, anyhow::Error>;
    fn file_attribute_key_exists(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id_in: i64,
    ) -> Result<bool, anyhow::Error>;
    fn text_attribute_key_exists(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id_in: i64,
    ) -> Result<bool, anyhow::Error>;
    fn relation_to_local_entity_key_exists(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id_in: i64,
    ) -> Result<bool, anyhow::Error>;
    fn group_key_exists(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id_in: i64,
    ) -> Result<bool, anyhow::Error>;
    fn relation_to_group_keys_exist_and_match(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id: i64,
        entity_id: i64,
        rel_type_id: i64,
        group_id: i64,
    ) -> Result<bool, anyhow::Error>;
    fn class_key_exists(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id_in: i64,
    ) -> Result<bool, anyhow::Error>;
    fn om_instance_key_exists(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id_in: &str,
    ) -> Result<bool, anyhow::Error>;
    fn is_duplicate_entity_name(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        name_in: &str,
        self_id_to_ignore_in: Option<i64>, /*= None*/
    ) -> Result<bool, anyhow::Error>;
    fn get_sorted_attributes(
        &self,
        db: Rc<dyn Database>,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        entity_id_in: i64,
        starting_object_index_in: usize /*= 0*/, 
        max_vals_in: usize /*= 0*/,
        only_public_entities_in: bool /*= true*/
    ) -> Result<(Vec<(i64, Rc<dyn Attribute>)>, usize), anyhow::Error>;
    fn get_relation_type_data(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id_in: i64,
    ) -> Result<Vec<Option<DataType>>, anyhow::Error>;
    fn get_quantity_attribute_data(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id_in: i64,
    ) -> Result<Vec<Option<DataType>>, anyhow::Error>;
    fn get_date_attribute_data(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id_in: i64,
    ) -> Result<Vec<Option<DataType>>, anyhow::Error>;
    fn get_file_attribute_data(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id_in: i64,
    ) -> Result<Vec<Option<DataType>>, anyhow::Error>;
    //%%
    // fn get_file_attribute_content(&self, transaction: &Option<&mut Transaction<Postgres>>, fileAttributeIdIn: i64, outputStreamIn: java.io.OutputStream) -> -> Result<(i64, String), anyhow::Error>
    fn get_text_attribute_data(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id_in: i64,
    ) -> Result<Vec<Option<DataType>>, anyhow::Error>;
    fn relation_to_local_entity_keys_exist_and_match(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id_in: i64,
        rel_type_id_in: i64,
        entity_id1_in: i64,
        entity_id2_in: i64,
    ) -> Result<bool, anyhow::Error>;
    fn relation_to_remote_entity_key_exists(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id_in: i64,
    ) -> Result<bool, anyhow::Error>;
    fn relation_to_remote_entity_keys_exist_and_match(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id_in: i64,
        rel_type_id_in: i64,
        entity_id1_in: i64,
        remote_instance_id_in: String,
        entity_id2_in: i64,
    ) -> Result<bool, anyhow::Error>;
    fn get_relation_to_local_entity_data(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        rel_type_id_in: i64,
        entity_id1_in: i64,
        entity_id2_in: i64,
    ) -> Result<Vec<Option<DataType>>, anyhow::Error>;
    fn get_relation_to_local_entity_data_by_id(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id_in: i64,
    ) -> Result<Vec<Option<DataType>>, anyhow::Error>;
    fn get_relation_to_remote_entity_data(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        rel_type_id_in: i64,
        entity_id1_in: i64,
        remote_instance_id_in: String,
        entity_id2_in: i64,
    ) -> Result<Vec<Option<DataType>>, anyhow::Error>;
    fn get_group_data(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id_in: i64,
    ) -> Result<Vec<Option<DataType>>, anyhow::Error>;
    fn get_group_entry_ids(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        group_id_in: i64,
        starting_object_index_in: i64,
        max_vals_in: Option<i64>, /*= None*/
                                  //) -> Result<Vec<Entity>, anyhow::Error>;
    ) -> Result<Vec<i64>, anyhow::Error>;
    fn get_highest_sorting_index_for_group(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        group_id_in: i64,
    ) -> Result<i64, anyhow::Error>;
    fn get_relation_to_group_data_by_keys(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        entity_id: i64,
        rel_type_id: i64,
        group_id: i64,
    ) -> Result<Vec<Option<DataType>>, anyhow::Error>;
    fn get_relation_to_group_data(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id_in: i64,
    ) -> Result<Vec<Option<DataType>>, anyhow::Error>;
    fn get_group_entries_data(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        group_id_in: i64,
        limit_in: Option<i64>,              /*= None*/
        include_archived_entities_in: bool, /*= true*/
    ) -> Result<Vec<Vec<Option<DataType>>>, anyhow::Error>;
    fn find_relation_to_and_group_on_entity(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        entity_id_in: i64,
        group_name_in: Option<String>, /* = None*/
    ) -> Result<(Option<i64>, Option<i64>, Option<i64>, Option<String>, bool), anyhow::Error>;
    fn get_entities_containing_group<'a>(
        &self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        group_id_in: i64,
        starting_index_in: i64,
        max_vals_in: Option<i64>, /*= None*/
                                  //) -> Result<Vec<(i64, Entity)>, anyhow::Error>;
    ) -> Result<Vec<(i64, i64)>, anyhow::Error>;
    fn get_count_of_entities_containing_group(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        group_id_in: i64,
    ) -> Result<(u64, u64), anyhow::Error>;
    fn get_class_data(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id_in: i64,
    ) -> Result<Vec<Option<DataType>>, anyhow::Error>;
    fn get_attribute_count(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        entity_id_in: i64,
        include_archived_entities_in: bool, /*= false*/
    ) -> Result<u64, anyhow::Error>;
    fn get_relation_to_local_entity_count(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        entity_id_in: i64,
        include_archived_entities: bool, /*= false*/
    ) -> Result<u64, anyhow::Error>;
    fn get_relation_to_remote_entity_count(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        entity_id_in: i64,
    ) -> Result<u64, anyhow::Error>;
    fn get_relation_to_group_count(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        entity_id_in: i64,
    ) -> Result<u64, anyhow::Error>;
    fn get_class_count(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        entity_id_in: Option<i64>, /*= None*/
    ) -> Result<u64, anyhow::Error>;
    fn get_class_name(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id_in: i64,
    ) -> Result<Option<String>, anyhow::Error>;
    fn get_om_instance_data(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id_in: String,
    ) -> Result<Vec<Option<DataType>>, anyhow::Error>;
    fn is_duplicate_om_instance_address(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        address_in: &str,
        _self_id_to_ignore_in: Option<String>, /*= None*/
    ) -> Result<bool, anyhow::Error>;
    fn get_groups_containing_entitys_groups_ids(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        group_id_in: i64,
        limit_in: Option<i64>, /*= Some(5)*/
    ) -> Result<Vec<Vec<Option<DataType>>>, anyhow::Error>;
    fn is_entity_in_group(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        group_id_in: i64,
        entity_id_in: i64,
    ) -> Result<bool, anyhow::Error>;
    fn get_adjacent_group_entries_sorting_indexes(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        group_id_in: i64,
        sorting_index_in: i64,
        limit_in: Option<i64>, /*= None*/
        forward_not_back_in: bool,
    ) -> Result<Vec<Vec<Option<DataType>>>, anyhow::Error>;
    fn get_nearest_group_entrys_sorting_index(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        group_id_in: i64,
        starting_point_sorting_index_in: i64,
        forward_not_back_in: bool,
    ) -> Result<Option<i64>, anyhow::Error>;
    fn get_adjacent_attributes_sorting_indexes(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        entity_id_in: i64,
        sorting_index_in: i64,
        limit_in: Option<i64>,
        forward_not_back_in: bool,
    ) -> Result<Vec<Vec<Option<DataType>>>, anyhow::Error>;
    fn get_nearest_attribute_entrys_sorting_index(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        entity_id_in: i64,
        starting_point_sorting_index_in: i64,
        forward_not_back_in: bool,
    ) -> Result<Option<i64>, anyhow::Error>;
    fn get_entity_attribute_sorting_index(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        entity_id_in: i64,
        attribute_form_id_in: i64,
        attribute_id_in: i64,
    ) -> Result<i64, anyhow::Error>;
    fn get_group_entry_sorting_index(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        group_id_in: i64,
        entity_id_in: i64,
    ) -> Result<i64, anyhow::Error>;
    fn is_group_entry_sorting_index_in_use(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        group_id_in: i64,
        sorting_index_in: i64,
    ) -> Result<bool, anyhow::Error>;
    fn is_attribute_sorting_index_in_use(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        entity_id_in: i64,
        sorting_index_in: i64,
    ) -> Result<bool, anyhow::Error>;
    fn find_unused_attribute_sorting_index(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        entity_id_in: i64,
        starting_with_in: Option<i64>, /*= None*/
    ) -> Result<i64, anyhow::Error>;
    fn find_all_entity_ids_by_name(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        name_in: String,
        case_sensitive: bool, /*= false*/
    ) -> Result<Vec<i64>, anyhow::Error>;
    fn find_unused_group_sorting_index(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        group_id_in: i64,
        starting_with_in: Option<i64>, /* = None*/
    ) -> Result<i64, anyhow::Error>;
    fn get_text_attribute_by_type_id(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        parent_entity_id_in: i64,
        type_id_in: i64,
        expected_rows: Option<usize>, /*= None*/
    ) -> Result<Vec<(i64, i64, i64, String, Option<i64>, i64, i64)>, anyhow::Error>;
    fn get_local_entities_containing_local_entity(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        entity_id_in: i64,
        starting_index_in: i64,
        max_vals_in: Option<i64>, /*= None*/
                                  //) -> Result<Vec<(i64, Entity)>, anyhow::Error>;
    ) -> Result<Vec<(i64, i64)>, anyhow::Error>;
    fn get_count_of_groups_containing_entity(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        entity_id_in: i64,
        //) -> Result<u64, anyhow::Error>;
    ) -> Result<u64, anyhow::Error>;
    fn get_containing_groups_ids(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        entity_id_in: i64,
    ) -> Result<Vec<i64>, anyhow::Error>;
    fn get_containing_relations_to_group(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        entity_id_in: i64,
        starting_index_in: i64,
        max_vals_in: Option<i64>, /*= None*/
    //) -> Result<Vec<RelationToGroup>, anyhow::Error>;
    ) -> Result<Vec<(i64, i64, i64, i64, Option<i64>, i64, i64)>, anyhow::Error>;
    //%% fn get_should_create_default_attributes(&self, transaction: &Option<&mut Transaction<Postgres>>, class_id_in: i64) -> Result<Option<bool>, anyhow::Error>;
    fn update_class_create_default_attributes(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        class_id_in: i64,
        value: Option<bool>,
    ) -> Result<u64, anyhow::Error>;
    fn get_entities_only_count(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        limit_by_class: bool,         /*= false*/
        class_id_in: Option<i64>,     /*= None*/
        template_entity: Option<i64>, /*= None*/
    ) -> Result<u64, anyhow::Error>;
    fn get_count_of_local_entities_containing_local_entity(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        entity_id_in: i64,
    ) -> Result<(u64, u64), anyhow::Error>;
    //idea (tracked): make "*duplicate*" methods just be ... called "search"? combine w/ search, or rename? makes sense for callers?
    fn is_duplicate_class_name(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        name_in: &str,
        self_id_to_ignore_in: Option<i64>, /*= None*/
    ) -> Result<bool, anyhow::Error>;
    fn get_containing_relation_to_group_descriptions(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        entity_id_in: i64,
        limit_in: Option<i64>, /*= None*/
    ) -> Result<Vec<String>, anyhow::Error>;
    fn get_matching_entities(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        starting_object_index_in: i64,
        max_vals_in: Option<i64>, /*= None*/
        omit_entity_id_in: Option<i64>,
        name_regex_in: String,
    ) -> Result<Vec<Entity>, anyhow::Error>;
    fn get_matching_groups(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        starting_object_index_in: i64,
        max_vals_in: Option<i64>, /*= None*/
        omit_group_id_in: Option<i64>,
        name_regex_in: String,
    ) -> Result<Vec<Group>, anyhow::Error>;
    fn get_relations_to_group_containing_this_group(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        group_id_in: i64,
        starting_index_in: i64,
        max_vals_in: Option<u64>, /*= None*/
    //) -> Result<Vec<RelationToGroup>, anyhow::Error>;
    ) -> Result<Vec<(i64, i64, i64, i64, Option<i64>, i64, i64)>, anyhow::Error>;
    fn get_entities(
        &self,
        db: Rc<dyn Database>,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        starting_object_index_in: i64,
        max_vals_in: Option<i64>, /*= None*/
    ) -> Result<Vec<Entity>, anyhow::Error>;
    fn get_entities_only(
        &self,
        db: Rc<dyn Database>,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        starting_object_index_in: i64,
        max_vals_in: Option<i64>,         /*= None*/
        class_id_in: Option<i64>,         /*= None*/
        limit_by_class: bool,             /*= false*/
        template_entity: Option<i64>,     /*= None*/
        group_to_omit_id_in: Option<i64>, /*= None*/
    ) -> Result<Vec<Entity>, anyhow::Error>;
    fn get_count_of_entities_used_as_attribute_types(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        object_type_in: String,
        quantity_seeks_unit_not_type_in: bool,
    ) -> Result<u64, anyhow::Error>;
    // fn get_entities_used_as_attribute_types(&self,
    //transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    // object_type_in: String, starting_object_index_in: i64, max_vals_in: Option<i64> /*= None*/,
    //                                     quantity_seeks_unit_not_type_in: bool) -> Result<Vec<Entity>, anyhow::Error>;
    fn get_relation_types(
        &self,
        db: Rc<dyn Database>,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        starting_object_index_in: i64,
        max_vals_in: Option<i64>, /*= None*/
    ) -> Result<Vec<RelationType>, anyhow::Error>;
    //%% fn get_classes(&self, transaction: &Option<&mut Transaction<Postgres>>, starting_object_index_in: i64, max_vals_in: Option<i64> /*= None*/) -> Result<Vec<EntityClass>, anyhow::Error>;
    fn get_relation_type_count(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<u64, anyhow::Error>;
    fn get_om_instance_count(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<u64, anyhow::Error>;
    fn get_entity_count(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<u64, anyhow::Error>;
    fn find_journal_entries(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        start_time_in: i64,
        end_time_in: i64,
        limit_in: Option<i64>, /*= None*/
    ) -> Result<Vec<(i64, String, i64)>, anyhow::Error>;
    fn get_group_count(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<u64, anyhow::Error>;
    fn get_groups(
        &self,
        db: Rc<dyn Database>, 
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        starting_object_index_in: i64, 
        max_vals_in: Option<i64> /*= None*/, 
        group_to_omit_id_in: Option<i64> /*= None*/
    ) -> Result<Vec<Group>, anyhow::Error>;
    fn create_group(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        name_in: &str,
        allow_mixed_classes_in_group_in: bool, /*= false*/
    ) -> Result<i64, anyhow::Error>;
    fn relation_to_group_key_exists(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id_in: i64,
    ) -> Result<bool, anyhow::Error>;
    fn update_entitys_class<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        entity_id: i64,
        class_id: Option<i64>,
    ) -> Result<(), anyhow::Error>;
    fn update_entity_only_new_entries_stick_to_top(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id_in: i64,
        new_entries_stick_to_top: bool,
    ) -> Result<u64, anyhow::Error>;
    fn archive_entity<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        id_in: i64,
    ) -> Result<u64, anyhow::Error>;
    fn unarchive_entity<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        id_in: i64,
    ) -> Result<u64, anyhow::Error>;
    fn set_include_archived_entities(&mut self, value_in: bool);
    fn set_user_preference_entity_id<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        name_in: &str,
        entity_id_in: i64,
    ) -> Result<(), anyhow::Error>;
    fn update_entity_only_public_status(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id_in: i64,
        value: Option<bool>,
    ) -> Result<u64, anyhow::Error>;
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
    ) -> Result<u64, anyhow::Error>;
    fn update_date_attribute(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id_in: i64,
        parent_id_in: i64,
        date_in: i64,
        attr_type_id_in: i64,
    ) -> Result<u64, anyhow::Error>;
    fn update_boolean_attribute(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id_in: i64,
        parent_id_in: i64,
        attr_type_id_in: i64,
        boolean_in: bool,
        valid_on_date_in: Option<i64>,
        observation_date_in: i64,
    ) -> Result<(), anyhow::Error>;
    fn update_boolean_attribute_value(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id_in: i64,
        //(see comment in implementation)
        //parent_id_in: i64,
        boolean_in: bool,
    ) -> Result<(), anyhow::Error>;
    fn update_file_attribute(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id_in: i64,
        parent_id_in: i64,
        attr_type_id_in: i64,
        description_in: String,
    ) -> Result<u64, anyhow::Error>;
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
    ) -> Result<u64, anyhow::Error>;
    fn update_text_attribute(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id_in: i64,
        parent_id_in: i64,
        attr_type_id_in: i64,
        text_in: &str,
        valid_on_date_in: Option<i64>,
        observation_date_in: i64,
    ) -> Result<u64, anyhow::Error>;
    fn update_relation_to_local_entity(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        old_relation_type_id_in: i64,
        entity_id1_in: i64,
        entity_id2_in: i64,
        new_relation_type_id_in: i64,
        valid_on_date_in: Option<i64>,
        observation_date_in: i64,
    ) -> Result<u64, anyhow::Error>;
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
    ) -> Result<u64, anyhow::Error>;
    fn update_group<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        group_id_in: i64,
        name_in: String,
        allow_mixed_classes_in_group_in: bool, /*= false*/
        new_entries_stick_to_top_in: bool,     /*= false*/
    ) -> Result<u64, anyhow::Error>;
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
    ) -> Result<u64, anyhow::Error>;
    fn move_relation_to_local_entity_into_local_entity(
        &self,
        rtle_id_in: i64,
        new_containing_entity_id_in: i64,
        sorting_index_in: i64,
    ) -> Result<(i64, i64), anyhow::Error>;
    fn move_relation_to_remote_entity_to_local_entity(
        &self,
        remote_instance_id_in: &str,
        relation_to_remote_entity_id_in: i64,
        to_containing_entity_id_in: i64,
        sorting_index_in: i64,
    ) -> Result<RelationToRemoteEntity, anyhow::Error>;
    fn create_entity_and_add_has_local_relation_to_it<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        from_entity_id_in: i64,
        new_entity_name_in: &str,
        observation_date_in: i64,
        is_public_in: Option<bool>,
    ) -> Result<(i64, i64, i64), anyhow::Error>;
    fn add_entity_and_relation_to_local_entity<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        rel_type_id_in: i64,
        from_entity_id_in: i64,
        new_entity_name_in: &str,
        valid_on_date_in: Option<i64>,
        observation_date_in: i64,
        is_public_in: Option<bool>,
    ) -> Result<(i64, i64), anyhow::Error>;
    fn move_local_entity_from_local_entity_to_group(
        &self,
        removing_rtle_in: &mut RelationToLocalEntity,
        target_group_id_in: i64,
        sorting_index_in: i64,
    ) -> Result<(), anyhow::Error>;
    fn move_relation_to_group(
        &self,
        relation_to_group_id_in: i64,
        new_containing_entity_id_in: i64,
        sorting_index_in: i64,
    ) -> Result<i64, anyhow::Error>;
    fn move_entity_from_group_to_local_entity(
        &self,
        from_group_id_in: i64,
        to_entity_id_in: i64,
        move_entity_id_in: i64,
        sorting_index_in: i64,
    ) -> Result<(), anyhow::Error>;
    fn move_local_entity_from_group_to_group(
        &self,
        from_group_id_in: i64,
        to_group_id_in: i64,
        move_entity_id_in: i64,
        sorting_index_in: i64,
    ) -> Result<(), anyhow::Error>;
    fn renumber_sorting_indexes<'a, 'b>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'b, Postgres>>>>,
        entity_id_or_group_id_in: i64,
        is_entity_attrs_not_group_entries: bool, /*= true*/
    ) -> Result<(), anyhow::Error>
    where
        'a: 'b;
    fn update_attribute_sorting_index(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        entity_id_in: i64,
        attribute_form_id_in: i64,
        attribute_id_in: i64,
        sorting_index_in: i64,
    ) -> Result<u64, anyhow::Error>;
    fn update_sorting_index_in_a_group(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        group_id_in: i64,
        entity_id_in: i64,
        sorting_index_in: i64,
    ) -> Result<u64, anyhow::Error>;
    fn update_entity_only_name(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id_in: i64,
        name_in: &str,
    ) -> Result<u64, anyhow::Error>;
    fn update_relation_type(
        &self,
        id_in: i64,
        name_in: &str,
        name_in_reverse_direction_in: &str,
        directionality_in: &str,
    ) -> Result<(), anyhow::Error>;
    fn update_class_and_template_entity_name<'a, 'b>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'b, Postgres>>>>,
        class_id_in: i64,
        template_entity_id_in: i64,
        name: &str,
    ) -> Result<(), anyhow::Error>
    where
        'a: 'b;
    fn update_om_instance(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id_in: String,
        address_in: String,
        entity_id_in: Option<i64>,
    ) -> Result<u64, anyhow::Error>;
    fn delete_entity<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        id_in: i64,
    ) -> Result<(), anyhow::Error>;
    fn delete_quantity_attribute<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        id_in: i64,
    ) -> Result<u64, anyhow::Error>;
    fn delete_date_attribute<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        id_in: i64,
    ) -> Result<u64, anyhow::Error>;
    fn delete_boolean_attribute<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        id_in: i64,
    ) -> Result<u64, anyhow::Error>;
    fn delete_file_attribute<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        id_in: i64,
    ) -> Result<u64, anyhow::Error>;
    fn delete_text_attribute<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        id_in: i64,
    ) -> Result<u64, anyhow::Error>;
    fn delete_relation_to_local_entity<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        rel_type_id_in: i64,
        entity_id1_in: i64,
        entity_id2_in: i64,
    ) -> Result<u64, anyhow::Error>;
    fn delete_relation_to_remote_entity<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        rel_type_id_in: i64,
        entity_id1_in: i64,
        remote_instance_id_in: &str,
        entity_id2_in: i64,
    ) -> Result<u64, anyhow::Error>;
    fn delete_relation_to_group<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        entity_id_in: i64,
        rel_type_id_in: i64,
        group_id_in: i64,
    ) -> Result<u64, anyhow::Error>;
    fn delete_group_and_relations_to_it<'a, 'b>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'b, Postgres>>>>,
        id_in: i64,
    ) -> Result<(), anyhow::Error>
    where
        'a: 'b;
    fn delete_relation_type<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        id_in: i64,
    ) -> Result<u64, anyhow::Error>;
    fn delete_class_and_its_template_entity(&self, class_id_in: i64) -> Result<(), anyhow::Error>;
    fn delete_group_relations_to_it_and_its_entries<'a, 'b>(
        &'a self,
        transaction_in: Option<Rc<RefCell<Transaction<'b, Postgres>>>>,
        group_id_in: i64,
    ) -> Result<(), anyhow::Error>
    where 
        'a: 'b
    ;
    fn delete_om_instance<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        id_in: &str,
    ) -> Result<u64, anyhow::Error>;
    fn remove_entity_from_group<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        group_id_in: i64,
        contained_entity_id_in: i64,
    ) -> Result<u64, anyhow::Error>;
    // (See comments above the set of these methods, in RestDatabase.rs:)
    fn get_user_preference_boolean<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        preference_name_in: &str,
        default_value_in: Option<bool>, /* = None*/
    ) -> Result<Option<bool>, anyhow::Error>;
    fn set_user_preference_boolean<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        name_in: &str,
        value_in: bool,
    ) -> Result<(), anyhow::Error>;
    fn get_preferences_container_id(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<i64, anyhow::Error>;
    fn get_user_preference_entity_id<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        preference_name_in: &str,
        default_value_in: Option<i64>, /*= None*/
    ) -> Result<Option<i64>, anyhow::Error>;
    //%% fn get_om_instances(&self, transaction: &Option<&mut Transaction<Postgres>>, localIn: Option<bool> /*= None*/) -> Result<Vec<OmInstance>, anyhow::Error>;
}
