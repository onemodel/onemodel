/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2016-2017 inclusive, 2020, and 2023, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
use crate::model::relation_to_local_entity::RelationToLocalEntity;
use crate::model::relation_to_remote_entity::RelationToRemoteEntity;
use crate::util::Util;
use sqlx::{Postgres, Transaction};
use std::collections::HashSet;
// use std::string::ToString;
// use crate::model::postgresql_database::PostgreSQLDatabase;

#[derive(Debug, Clone)]
pub enum DataType {
    Float(f64),
    String(String),
    Bigint(i64),
    Boolean(bool),
    Smallint(i16),
}

pub trait Database {
    fn is_remote(&self) -> bool;
    // fn setup_db(&self) -> Result<(), String>;

    fn get_remote_address(&self) -> Option<String> {
        None
    }
    fn include_archived_entities(&self) -> bool;
    fn begin_trans(&self) -> Result<Transaction<Postgres>, sqlx::Error>;
    fn begin_trans_test(&self) -> Result<i32 /*Transaction<Postgres>*/, sqlx::Error>;
    // fn begin_trans_test(&self) -> Result<Transaction<Postgres>, sqlx::Error>;
    fn rollback_trans(&self, tx: Transaction<Postgres>) -> Result<(), sqlx::Error>;
    fn commit_trans(&self, tx: Transaction<Postgres>) -> Result<(), sqlx::Error>;

    // where we create the table also calls this.
    // Longer than the old 60 (needed), and a likely familiar length to many people (for ease in knowing when done), seems a decent balance. If any longer
    // is needed, maybe it should be put in a TextAttribute and make those more convenient to use, instead.
    // (See usages. The DNS hostname max size seems to be 255 plus 1 null, but the ":<port>" part could add 6 more chars (they seem to go up to :65535).
    // Maybe someday we will have to move to a larger size in case it changes or uses unicode or I don't know what.)
    fn om_instance_address_length(&self) -> i32 {
        262
    }

    /// This has &self as a parameter to avoid a compiler error about Database not being able to
    /// be made into an object, unless it is there.
    fn get_attribute_form_id(&self, key: &str) -> Option<i32> {
        //MAKE SURE THESE MATCH WITH THOSE IN attribute_key_exists and get_attribute_form_name, and the range in the db constraint valid_attribute_form_id ,
        // and in RestDatabase.process_array_of_tuples_and_int !
        match key {
            Util::QUANTITY_TYPE => Some(1),
            Util::DATE_TYPE => Some(2),
            Util::BOOLEAN_TYPE => Some(3),
            Util::FILE_TYPE => Some(4),
            Util::TEXT_TYPE => Some(5),
            Util::RELATION_TO_LOCAL_ENTITY_TYPE => Some(6),
            "RelationToLocalEntity" => Some(6),
            Util::RELATION_TO_GROUP_TYPE => Some(7),
            Util::RELATION_TO_REMOTE_ENTITY_TYPE => Some(8),
            _ => None,
        }
    }
    fn get_attribute_form_name(&self, key: i32) -> Option<&str> {
        // MAKE SURE THESE MATCH WITH THOSE IN get_attribute_form_id !
        //idea: put these values in a structure that is looked up both ways, instead of duplicating them?
        match key {
            1 => Some(Util::QUANTITY_TYPE),
            2 => Some(Util::DATE_TYPE),
            3 => Some(Util::BOOLEAN_TYPE),
            4 => Some(Util::FILE_TYPE),
            5 => Some(Util::TEXT_TYPE),
            6 => Some(Util::RELATION_TO_LOCAL_ENTITY_TYPE),
            7 => Some(Util::RELATION_TO_GROUP_TYPE),
            8 => Some(Util::RELATION_TO_REMOTE_ENTITY_TYPE),
            _ => None,
        }
    }

    /// This has &self as a parameter to avoid a compiler error about Database not being able to
    /// be made into an object, unless it is there.
    fn max_id_value(&self) -> i64 {
        //%%
        // Max size for a Java long type, a Rust i64, and for a postgresql 7.2.1 bigint type (which is being used, at the moment, for the id value in Entity table.
        // (these values are from file:///usr/share/doc/postgresql-doc-9.1/html/datatype-numeric.html)
        // 9223372036854775807L
        i64::MAX
    }

    /// This has &self as a parameter to avoid a compiler error about Database not being able to
    /// be made into an object, unless it is there.
    fn min_id_value(&self) -> i64 {
        //%% -9223372036854775808L
        i64::MIN
    }
    // mbe moving to be inside pgsql .rs instead..?
    // fn login(username: &str, password: &str) -> Result<Database, &'static str> {
    //     PostgreSQLDatabase::new(username, password)
    // }

    fn create_boolean_attribute(
        &self,
        parent_id_in: i64,
        attr_type_id_in: i64,
        boolean_in: bool,
        valid_on_date_in: Option<i64>,
        observation_date_in: i64,
        sorting_index_in: Option<i64>, /*%%= None*/
    ) -> Result<i64, String>;
    fn create_text_attribute<'a>(
        &'a self,
        transaction: &Option<&mut Transaction<'a, Postgres>>,
        parent_id_in: i64,
        attr_type_id_in: i64,
        text_in: &str,
        valid_on_date_in: Option<i64>,        /*%%= None*/
        observation_date_in: i64,             /*%%= System.currentTimeMillis()*/
        caller_manages_transactions_in: bool, /*%%= false*/
        sorting_index_in: Option<i64>,        /*%%= None*/
    ) -> Result<i64, String>;
    fn create_relation_to_local_entity<'a>(
        &'a self,
        transaction: &Option<&mut Transaction<'a, Postgres>>,
        relation_type_id_in: i64,
        entity_id1_in: i64,
        entity_id2_in: i64,
        valid_on_date_in: Option<i64>,
        observation_date_in: i64,
        sorting_index_in: Option<i64>,        /*%% = None*/
        caller_manages_transactions_in: bool, /*%%= false*/
    ) -> Result<RelationToLocalEntity, String>;
    fn create_relation_to_remote_entity<'a>(
        &'a self,
        transaction: &Option<&mut Transaction<'a, Postgres>>,
        relation_type_id_in: i64,
        entity_id1_in: i64,
        entity_id2_in: i64,
        valid_on_date_in: Option<i64>,
        observation_date_in: i64,
        remote_instance_id_in: String,
        sorting_index_in: Option<i64>,        /*%% = None*/
        caller_manages_transactions_in: bool, /*%% = false*/
    ) -> Result<RelationToRemoteEntity, String>;
    fn create_group_and_relation_to_group<'a>(
        &'a self,
        transaction: &Option<&mut Transaction<'a, Postgres>>,
        entity_id_in: i64,
        relation_type_id_in: i64,
        new_group_name_in: &str,
        allow_mixed_classes_in_group_in: bool, /*%%= false*/
        valid_on_date_in: Option<i64>,
        observation_date_in: i64,
        sorting_index_in: Option<i64>,
        caller_manages_transactions_in: bool, /*%%= false*/
    ) -> Result<(i64, i64), String>;

    fn create_entity<'a>(
        &'a self,
        transaction: &Option<&mut Transaction<'a, Postgres>>,
        name_in: &str,
        class_id_in: Option<i64>,   /*%%= None*/
        is_public_in: Option<bool>, /*%% = None*/
    ) -> Result<i64, String>;
    fn create_entity_and_relation_to_local_entity<'a>(
        &'a self,
        transaction: &Option<&mut Transaction<'a, Postgres>>,
        entity_id_in: i64,
        relation_type_id_in: i64,
        new_entity_name_in: &str,
        is_public_in: Option<bool>,
        valid_on_date_in: Option<i64>,
        observation_date_in: i64,
        caller_manages_transactions_in: bool, /*%%= false*/
    ) -> Result<(i64, i64), String>;
    fn create_relation_to_group<'a>(
        &'a self,
        transaction: &Option<&mut Transaction<'a, Postgres>>,
        entity_id_in: i64,
        relation_type_id_in: i64,
        group_id_in: i64,
        valid_on_date_in: Option<i64>,
        observation_date_in: i64,
        sorting_index_in: Option<i64>,        /*%%= None*/
        caller_manages_transactions_in: bool, /*%%= false*/
    ) -> Result<(i64, i64), String>;
    fn add_entity_to_group<'a>(
        &'a self,
        transaction: &Option<&mut Transaction<'a, Postgres>>,
        group_id_in: i64,
        contained_entity_id_in: i64,
        sorting_index_in: Option<i64>,        /*%%= None*/
        caller_manages_transactions_in: bool, /*%% = false*/
    ) -> Result<(), String>;
    fn create_om_instance<'a>(
        &'a self,
        transaction: &Option<&mut Transaction<'a, Postgres>>,
        id_in: String,
        is_local_in: bool,
        address_in: String,
        entity_id_in: Option<i64>, /*%%= None*/
        old_table_name: bool,      /*%% = false*/
    ) -> Result<i64, String>;
    fn create_relation_type<'a>(
        &'a self,
        caller_manages_transactions_in: bool,
        transaction: &Option<&mut Transaction<'a, Postgres>>,
        name_in: &str,
        name_in_reverse_direction_in: &str,
        directionality_in: &str,
    ) -> Result<i64, String>;
    fn create_class_and_its_template_entity(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        class_name_in: String,
    ) -> Result<(i64, i64), String>;
    fn find_contained_local_entity_ids<'a>(
        &'a self,
        transaction: &Option<&mut Transaction<Postgres>>,
        results_in_out: &'a mut HashSet<i64>,
        from_entity_id_in: i64,
        search_string_in: &str,
        levels_remaining: i32,      /* = 20%%*/
        stop_after_any_found: bool, /*%% = true*/
    ) -> Result<&mut HashSet<i64>, String>;

    fn entity_key_exists(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        id_in: i64,
        include_archived: bool, /*= true%%*/
    ) -> Result<bool, String>;
    fn boolean_attribute_key_exists(&self, transaction: &Option<&mut Transaction<Postgres>>, id_in: i64) -> Result<bool, String>;
    fn get_entity_data(&self, transaction: &Option<&mut Transaction<Postgres>>, id_in: i64) -> Result<Vec<DataType>, String>;
    fn get_entity_name(&self, transaction: &Option<&mut Transaction<Postgres>>, id_in: i64) -> Result<Option<String>, String>;
    fn find_relation_type(&self, transaction: &Option<&mut Transaction<Postgres>>, type_name_in: String) -> Result<i64, String>;
    fn get_boolean_attribute_data(&self, transaction: &Option<&mut Transaction<Postgres>>, id_in: i64) -> Result<Vec<DataType>, String>;
    fn get_group_size(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        group_id_in: i64,
        include_which_entities_in: i32, /*%% = 3*/
    ) -> Result<i64, String>;

    /*
      pub fn getRestDatabase(remoteAddressIn: String) -> RestDatabase {
        new RestDatabase(remoteAddressIn)
      }

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

      /* %%Many of these methods were marked "protected[model]" in scala, for 2 reasons:
           1) to minimize the risk of calling db.<method> on the wrong db, when the full model object (like Entity) would contain the right db for itself
              (ie, what if one called db.delete and the same entity id # exists in both databases), and
           2) to generally manage the coupling between the Controller and model package, since it seems cleaner to go through model objects when they can
              call the db for themselves, rather than everything touching the db entrails directly.
         ...but should be avoided when going through the model object (like Entity) causes enough more db hits to not be worth it (performance vs.
         clarity & ease of maintenance).
      * */
      fn createQuantityAttribute(parent_id_in: i64, attr_type_id_in: i64, unitIdIn: i64, numberIn: Float, valid_on_date_in: Option<i64>,
                                  observation_date_in: i64, caller_manages_transactions_in: bool = false, sorting_index_in: Option<i64> = None) -> /*id*/ i64;
      fn createDateAttribute(parent_id_in: i64, attr_type_id_in: i64, date_in: i64, sorting_index_in: Option<i64> = None) -> /*id*/ i64;
      fn createFileAttribute(parent_id_in: i64, attr_type_id_in: i64, descriptionIn: String, originalFileDateIn: i64, storedDateIn: i64,
                              original_file_path_in: String, readableIn: bool, writableIn: bool, executableIn: bool, sizeIn: i64,
                              md5hashIn: String, inputStreamIn: java.io.FileInputStream, sorting_index_in: Option<i64> = None) -> /*id*/ i64;
      fn addHASRelationToLocalEntity(from_entity_id_in: i64, toEntityIdIn: i64, valid_on_date_in: Option<i64>, observation_date_in: i64,
                                 sorting_index_in: Option<i64> = None) -> RelationToLocalEntity;
      pub fn getOrCreateClassAndTemplateEntity(class_name_in: String, caller_manages_transactions_in: bool) -> (i64, i64);
      fn addUriEntityWithUriAttribute(containingEntityIn: Entity, new_entity_name_in: String, uriIn: String, observation_date_in: i64,
                                       makeThem_publicIn: Option<bool>, caller_manages_transactions_in: bool,
                                       quoteIn: Option<String> = None) -> (Entity, RelationToLocalEntity);


      pub fn attribute_key_exists(form_id_in: i64, id_in: i64) -> bool;
    fn relationTypeKeyExists(id_in: i64) -> bool;
    fn quantityAttributeKeyExists(id_in: i64) -> bool;
    fn dateAttributeKeyExists(id_in: i64) -> bool;
          fn fileAttributeKeyExists(id_in: i64) -> bool;
          fn textAttributeKeyExists(id_in: i64) -> bool;
          pub fn relationToLocal_entity_key_exists(id_in: i64) -> bool;
          pub fn groupKeyExists(id_in: i64) -> bool;
          fn relationToGroupKeysExistAndMatch(id: i64, entity_id: i64, rel_type_id: i64, group_id: i64) -> bool;
          fn classKeyExists(id_in: i64) -> bool;
          fn omInstanceKeyExists(id_in: String) -> bool;
           fn isDuplicateEntityName(name_in: String, selfIdToIgnoreIn: Option<i64> = None) -> bool;
           fn getSortedAttributes(entity_id_in: i64, startingObjectIndexIn: Int = 0, maxValsIn: Int = 0,
                                   onlyPublicEntitiesIn: bool = true) -> (Array[(i64, Attribute)], Int);
    fn getRelationTypeData(id_in: i64) -> Array[Option[Any]];
    fn getQuantityAttributeData(id_in: i64) -> Array[Option[Any]];
    fn getDateAttributeData(id_in: i64) -> Array[Option[Any]];
      fn getFileAttributeData(id_in: i64) -> Array[Option[Any]];
      fn getFileAttributeContent(fileAttributeIdIn: i64, outputStreamIn: java.io.OutputStream) -> (i64, String);
      fn getTextAttributeData(id_in: i64) -> Array[Option[Any]];
      fn relationToLocalEntityKeysExistAndMatch(id_in: i64, rel_type_idIn: i64, entity_id1_in: i64, entity_id2_in: i64) -> bool;
      fn relationToRemote_entity_key_exists(id_in: i64) -> bool;
      fn relationToRemoteEntityKeysExistAndMatch(id_in: i64, rel_type_idIn: i64, entity_id1_in: i64, remote_instance_id_in: String, entity_id2_in: i64) -> bool;
      fn getRelationToLocalEntityData(rel_type_idIn: i64, entity_id1_in: i64, entity_id2_in: i64) -> Array[Option[Any]];
      fn getRelationToLocalEntityDataById(id_in: i64) -> Array[Option[Any]];
      fn getRelationToRemoteEntityData(rel_type_idIn: i64, entity_id1_in: i64, remote_instance_id_in: String, entity_id2_in: i64) -> Array[Option[Any]];
      fn getGroupData(id_in: i64) -> Array[Option[Any]];
      fn getGroupEntryObjects(group_id_in: i64, startingObjectIndexIn: i64, maxValsIn: Option<i64> = None) -> Vec<Entity>;
      fn getHighestSortingIndexForGroup(group_id_in: i64) -> i64;
      fn getRelationToGroupDataByKeys(entity_id: i64, rel_type_id: i64, group_id: i64) -> Array[Option[Any]];
      fn getRelationToGroupData(id_in: i64) -> Array[Option[Any]];
      pub fn getGroupEntriesData(group_id_in: i64, limitIn: Option<i64> = None, include_archived_entities_in: bool = true) -> List[Array[Option[Any]]];

    */
    fn find_relation_to_and_group_on_entity(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        entity_id_in: i64,
        group_name_in: Option<String>, /*%% = None*/
    ) -> Result<(Option<i64>, Option<i64>, Option<i64>, Option<String>, bool), String>;
    /*
    pub fn getEntitiesContainingGroup(group_id_in: i64, startingIndexIn: i64, maxValsIn: Option<i64> = None) -> java.util.ArrayList[(i64, Entity)];
    fn getCountOfEntitiesContainingGroup(group_id_in: i64) -> (i64, i64);
    fn getClassData(id_in: i64) -> Array[Option[Any]];
    */
    fn get_attribute_count(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        entity_id_in: i64,
        include_archived_entities_in: bool, /*%%= false*/
    ) -> Result<i64, String>;
    fn get_relation_to_local_entity_count(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        entity_id_in: i64,
        include_archived_entities: bool, /*%%= false*/
    ) -> Result<i64, String>;
    fn get_relation_to_remote_entity_count(&self, transaction: &Option<&mut Transaction<Postgres>>,
                                           entity_id_in: i64) -> Result<i64, String>;
    fn get_relation_to_group_count(&self, transaction: &Option<&mut Transaction<Postgres>>, entity_id_in: i64) -> Result<i64, String>;
    /*
    pub fn getClassCount(entity_id_in: Option<i64> = None) -> i64;
    fn getClassName(id_in: i64) -> Option<String>;
    fn getOmInstanceData(id_in: String) -> Array[Option[Any]];
    fn isDuplicateOmInstanceAddress(address_in: String, selfIdToIgnoreIn: Option<String> = None) -> bool;
    fn  getGroupsContainingEntitysGroupsIds(group_id_in: i64, limitIn: Option<i64> = Some(5)) -> List[Array[Option[Any]]];
    fn isEntityInGroup(group_id_in: i64, entity_id_in: i64) -> bool;
    fn getAdjacentGroupEntriesSortingIndexes(group_id_in: i64, sorting_index_in: i64, limitIn: Option<i64> = None,
                                              forwardNotBackIn: bool) -> List[Array[Option[Any]]];
    fn getNearestGroupEntrysSortingIndex(group_id_in: i64, startingPointSortingIndexIn: i64, forwardNotBackIn: bool) -> Option<i64>;
    fn getAdjacentAttributesSortingIndexes(entity_id_in: i64, sorting_index_in: i64, limitIn: Option<i64>, forwardNotBackIn: bool) -> List[Array[Option[Any]]];
    fn getNearestAttributeEntrysSortingIndex(entity_id_in: i64, startingPointSortingIndexIn: i64, forwardNotBackIn: bool) -> Option<i64>;
    fn getEntityAttributeSortingIndex(entity_id_in: i64, attribute_form_id_in: i64, attribute_id_in: i64) -> i64;
    fn getGroupEntrySortingIndex(group_id_in: i64, entity_id_in: i64) -> i64;
    */
    fn is_group_entry_sorting_index_in_use(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        group_id_in: i64,
        sorting_index_in: i64,
    ) -> Result<bool, String>;
    /*
    fn is_attribute_sorting_index_in_use(entity_id_in: i64, sorting_index_in: i64) -> bool;
    */
    fn find_unused_attribute_sorting_index(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        entity_id_in: i64,
        starting_with_in: Option<i64>, /*%%= None*/
    ) -> Result<i64, String>;
    /*
    pub fn findAllEntityIdsByName(name_in: String, caseSensitive: bool = false) -> java.util.ArrayList[i64];
    */
    fn find_unused_group_sorting_index(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        group_id_in: i64,
        starting_with_in: Option<i64>, /*%% = None*/
    ) -> Result<i64, String>;
    /*
      fn getTextAttributeByTypeId(parentEntityIdIn: i64, typeIdIn: i64, expected_rows: Option[Int] = None) -> java.util.ArrayList[TextAttribute];
      fn getLocalEntitiesContainingLocalEntity(entity_id_in: i64, startingIndexIn: i64, maxValsIn: Option<i64> = None) -> java.util.ArrayList[(i64, Entity)];
      fn getCountOfGroupsContainingEntity(entity_id_in: i64) -> i64;
      fn getContainingGroupsIds(entity_id_in: i64) -> java.util.ArrayList[i64];
      fn getContainingRelationsToGroup(entity_id_in: i64, startingIndexIn: i64,
                                                         maxValsIn: Option<i64> = None) -> java.util.ArrayList[RelationToGroup];
    //  fn getShouldCreateDefaultAttributes(class_id_in: i64) -> Option<bool>;
      fn updateClassCreateDefaultAttributes(class_id_in: i64, value -> Option<bool>);
      pub fn getEntitiesOnlyCount(limitByClass: bool = false, class_id_in: Option<i64> = None, templateEntity: Option<i64> = None) -> i64;
      fn getCountOfLocalEntitiesContainingLocalEntity(entity_id_in: i64) -> (i64, i64);
      //idea (tracked): make "*duplicate*" methods just be ... called "search"? combine w/ search, or rename? makes sense for callers?
      pub fn isDuplicateClassName(name_in: String, selfIdToIgnoreIn: Option<i64> = None) -> bool;
      fn getContainingRelationToGroupDescriptions(entity_id_in: i64, limitIn: Option<i64> = None) -> util.ArrayList[String];
      pub fn getMatchingEntities(startingObjectIndexIn: i64, maxValsIn: Option<i64> = None, omitEntityIdIn: Option<i64>,
                              nameRegexIn: String) -> Vec<Entity>;
      pub fn getMatchingGroups(startingObjectIndexIn: i64, maxValsIn: Option<i64> = None, omitGroupIdIn: Option<i64>,
                            nameRegexIn: String) -> java.util.ArrayList[Group];
      fn getRelationsToGroupContainingThisGroup(group_id_in: i64, startingIndexIn: i64,
                                                                  maxValsIn: Option<i64> = None) -> java.util.ArrayList[RelationToGroup];
      pub fn getEntities(startingObjectIndexIn: i64, maxValsIn: Option<i64> = None) -> Vec<Entity>;
      pub fn getEntitiesOnly(startingObjectIndexIn: i64, maxValsIn: Option<i64> = None, class_id_in: Option<i64> = None,
                          limitByClass: bool = false, templateEntity: Option<i64> = None,
                          groupToOmitIdIn: Option<i64> = None) -> Vec<Entity>;
      pub fn getCountOfEntitiesUsedAsAttributeTypes(objectTypeIn: String, quantitySeeksUnitNotTypeIn: bool) -> i64;
      pub fn getEntitiesUsedAsAttributeTypes(objectTypeIn: String, startingObjectIndexIn: i64, maxValsIn: Option<i64> = None,
                                          quantitySeeksUnitNotTypeIn: bool) -> Vec<Entity>;
      pub fn getRelationTypes(startingObjectIndexIn: i64, maxValsIn: Option<i64> = None) -> Vec<Entity>;
      pub fn getClasses(startingObjectIndexIn: i64, maxValsIn: Option<i64> = None) -> java.util.ArrayList[EntityClass];
      pub fn getRelationTypeCount -> i64;
      pub fn getOmInstanceCount -> i64;
      pub fn getEntityCount -> i64;
      pub fn findJournalEntries(startTimeIn: i64, endTimeIn: i64, limitIn: Option<i64> = None) -> util.ArrayList[(i64, String, i64)];
      pub fn getGroupCount -> i64;
      pub fn getGroups(startingObjectIndexIn: i64, maxValsIn: Option<i64> = None, groupToOmitIdIn: Option<i64> = None) -> java.util.ArrayList[Group];
      */
    fn create_group(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        name_in: &str,
        allow_mixed_classes_in_group_in: bool, /*%%= false*/
    ) -> Result<i64, String>;
    /*
    pub fn relationToGroupKeyExists(id_in: i64) -> bool;


    fn updateEntitysClass(entity_id: i64, class_id: Option<i64>, caller_manages_transactions_in -> bool = false);
    fn updateEntityOnlyNewEntriesStickToTop(id_in: i64, newEntriesStickToTop -> bool);
    fn archiveEntity(id_in: i64, caller_manages_transactions_in -> bool = false);
    fn unarchiveEntity(id_in: i64, caller_manages_transactions_in -> bool = false);
    pub fn set_include_archived_entities(in: bool) -> Unit;
    pub fn setUserPreference_EntityId(name_in: String, entity_id_in -> i64);
    fn updateEntityOnlyPublicStatus(id_in: i64, value -> Option<bool>);
    fn updateQuantityAttribute(id_in: i64, parent_id_in: i64, attr_type_id_in: i64, unitIdIn: i64, numberIn: Float, valid_on_date_in: Option<i64>,
                                observation_date_in -> i64);
    fn updateDateAttribute(id_in: i64, parent_id_in: i64, date_in: i64, attr_type_id_in -> i64);
    */
    fn update_boolean_attribute(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        id_in: i64,
        parent_id_in: i64,
        attr_type_id_in: i64,
        boolean_in: bool,
        valid_on_date_in: Option<i64>,
        observation_date_in: i64,
    ) -> Result<(), String>;
    /*
           fn updateFileAttribute(id_in: i64, parent_id_in: i64, attr_type_id_in: i64, descriptionIn -> String);
           fn updateFileAttribute(id_in: i64, parent_id_in: i64, attr_type_id_in: i64, descriptionIn: String,
                                                    originalFileDateIn: i64, storedDateIn: i64,
                                   original_file_path_in: String, readableIn: bool, writableIn: bool, executableIn: bool, sizeIn: i64, md5hashIn: String);
           fn updateTextAttribute(id_in: i64, parent_id_in: i64, attr_type_id_in: i64, text_in: String, valid_on_date_in: Option<i64>,
                                                    observation_date_in -> i64);
           fn updateRelationToLocalEntity(oldRelationTypeIdIn: i64, entity_id1_in: i64, entity_id2_in: i64,
                                      newRelationTypeIdIn: i64, valid_on_date_in: Option<i64>, observation_date_in -> i64);
           fn updateRelationToRemoteEntity(oldRelationTypeIdIn: i64, entity_id1_in: i64, remote_instance_id_in: String, entity_id2_in: i64,
                                            newRelationTypeIdIn: i64, valid_on_date_in: Option<i64>, observation_date_in -> i64);
           fn updateGroup(group_id_in: i64, name_in: String, allow_mixed_classes_in_group_in: bool = false, newEntriesStickToTopIn -> bool = false);
           fn updateRelationToGroup(entity_id_in: i64, oldRelationTypeIdIn: i64, newRelationTypeIdIn: i64, oldGroupIdIn: i64, newGroupIdIn: i64,
                                     valid_on_date_in: Option<i64>, observation_date_in -> i64);
           fn moveRelationToLocalEntityToLocalEntity(rtleIdIn: i64, newContainingEntityIdIn: i64,
                                                                       sorting_index_in: i64) -> RelationToLocalEntity;
           fn moveRelationToRemoteEntityToLocalEntity(remote_instance_id_in: String, relationToRemoteEntityIdIn: i64, toContainingEntityIdIn: i64,
                                                                        sorting_index_in: i64) -> RelationToRemoteEntity;
           fn moveLocalEntityFromLocalEntityToGroup(removingRtleIn: RelationToLocalEntity, targetGroupIdIn: i64, sorting_index_in: i64);
           fn moveRelationToGroup(relationToGroupIdIn: i64, newContainingEntityIdIn: i64, sorting_index_in: i64) -> i64;
           fn moveEntityFromGroupToLocalEntity(fromGroupIdIn: i64, toEntityIdIn: i64, moveEntityIdIn: i64, sorting_index_in: i64);
           fn moveLocalEntityFromGroupToGroup(fromGroupIdIn: i64, toGroupIdIn: i64, moveEntityIdIn: i64, sorting_index_in: i64);
           fn renumberSortingIndexes(entity_idOrGroupIdIn: i64, caller_manages_transactions_in: bool = false,
                                                       isEntityAttrsNotGroupEntries: bool = true);
           fn updateAttributeSortingIndex(entity_id_in: i64, attribute_form_id_in: i64, attribute_id_in: i64, sorting_index_in: i64);
           fn updateSortingIndexInAGroup(group_id_in: i64, entity_id_in: i64, sorting_index_in: i64);
           fn updateEntityOnlyName(id_in: i64, name_in: String);
           fn updateRelationType(id_in: i64, name_in: String, name_in_reverse_direction_in: String, directionality_in: String);
           fn updateClassAndTemplateEntityName(class_id_in: i64, name: String) -> i64;
           fn updateOmInstance(id_in: String, address_in: String, entity_id_in: Option<i64>);
    */
    fn delete_entity<'a>(
        &'a self,
        transaction: &Option<&mut Transaction<'a, Postgres>>,
        id_in: i64,
        caller_manages_transactions_in: bool, /*%%= false*/
    ) -> Result<(), String>;
    /*
       fn deleteQuantityAttribute(id_in: i64);
       fn deleteDateAttribute(id_in: i64);
       fn deleteBooleanAttribute(id_in: i6;4)
       fn deleteFileAttribute(id_in: i64);
       fn deleteTextAttribute(id_in: i64);
       fn deleteRelationToLocalEntity(rel_type_idIn: i64, entity_id1_in: i64, entity_id2_in: i64);
       fn deleteRelationToRemoteEntity(rel_type_idIn: i64, entity_id1_in: i64, remote_instance_id_in: String, entity_id2_in: i64);
       fn deleteRelationToGroup(entity_id_in: i64, rel_type_idIn: i64, group_id_in: i64);
       fn deleteGroupAndRelationsToIt(id_in: i64);
       fn deleteRelationType(id_in: i64);
       fn deleteClassAndItsTemplateEntity(class_id_in: i64);
       fn deleteGroupRelationsToItAndItsEntries(group_id_in: i64);
       fn deleteOmInstance(id_in: String) -> Unit;
       fn removeEntityFromGroup(group_id_in: i64, contained_entity_id_in: i64, caller_manages_transactions_in: bool = false);
    */

    // (See comments above the set of these methods, in RestDatabase.rs:)
    fn get_user_preference_boolean(
        &self,
        transaction: &Option<&mut Transaction<Postgres>>,
        preference_name_in: &str,
        default_value_in: Option<bool>, /*%% = None*/
    ) -> Option<bool>;
    fn set_user_preference_boolean<'a>(&'a self, transaction: &Option<&mut Transaction<'a, Postgres>>,
                                   name_in: &str, value_in: bool) -> Result<(), String>;
    fn get_preferences_container_id(&self, transaction: &Option<&mut Transaction<Postgres>>) -> Result<i64, String>;
    //%%$%%next:
    // fn getUserPreference_EntityId(&self, preference_name_in: String, default_value_in: Option<i64> = None) -> Option<i64>;
    // fn getOmInstances(&self, localIn: Option<bool> = None) -> java.util.ArrayList[OmInstance];
}
