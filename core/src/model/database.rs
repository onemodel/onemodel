/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2016-2017 inclusive, 2020, and 2023, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
use crate::util::Util;
use sqlx::{Postgres, Transaction};
/* %% package org.onemodel.core.model
import java.util
import org.onemodel.core.{OmDatabaseException, Util}
import scala.collection.mutable
*/

// use std::string::ToString;
// use crate::model::postgresql_database::PostgreSQLDatabase;

pub trait Database {
    fn is_remote(&self) -> bool;

    fn get_remote_address(&self) -> Option<String> {
        None
    }
    fn include_archived_entities(&self) -> bool;
    fn begin_trans(&self) -> Result<Transaction<Postgres>, sqlx::Error>;
    fn rollback_trans(&self, tx: sqlx::Transaction<Postgres>) -> Result<(), sqlx::Error>;
    fn commit_trans(&self, tx: sqlx::Transaction<Postgres>) -> Result<(), sqlx::Error>;

    // where we create the table also calls this.
    // Longer than the old 60 (needed), and a likely familiar length to many people (for ease in knowing when done), seems a decent balance. If any longer
    // is needed, maybe it should be put in a TextAttribute and make those more convenient to use, instead.
    // (See usages. The DNS hostname max size seems to be 255 plus 1 null, but the ":<port>" part could add 6 more chars (they seem to go up to :65535).
    // Maybe someday we will have to move to a larger size in case it changes or uses unicode or I don't know what.)
    fn om_instance_address_length(&self) -> i32 {
        262
    }

    fn get_attribute_form_id(&self, key: String) -> Option<i32> {
        //MAKE SURE THESE MATCH WITH THOSE IN attribute_key_exists and get_attribute_form_name, and the range in the db constraint valid_attribute_form_id ,
        // and in RestDatabase.process_array_of_tuples_and_int !
        match key.as_str() {
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

    fn max_id_value(&self) -> i64 {
        //%%
        // Max size for a Java long type, a Rust i64, and for a postgresql 7.2.1 bigint type (which is being used, at the moment, for the id value in Entity table.
        // (these values are from file:///usr/share/doc/postgresql-doc-9.1/html/datatype-numeric.html)
        // 9223372036854775807L
        i64::MAX
    }

    fn min_id_value(&self) -> i64 {
        //%% -9223372036854775808L
        i64::MIN
    }

    // mbe moving to be inside pgsql .rs instead..?
    // fn login(username: &str, password: &str) -> Result<Database, &'static str> {
    //     PostgreSQLDatabase::new(username, password)
    // }

    /*  %%$%%
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
       fn createQuantityAttribute(parentIdIn: i64, attrTypeIdIn: i64, unitIdIn: i64, numberIn: Float, valid_on_date_in: Option<i64>,
                                   inObservationDate: i64, callerManagesTransactionsIn: Boolean = false, sortingIndexIn: Option<i64> = None) -> /*id*/ i64;
       fn createDateAttribute(parentIdIn: i64, attrTypeIdIn: i64, date_in: i64, sortingIndexIn: Option<i64> = None) -> /*id*/ i64;
       fn createBooleanAttribute(parentIdIn: i64, attrTypeIdIn: i64, booleanIn: Boolean, valid_on_date_in: Option<i64>, observationDateIn: i64,
                                  sortingIndexIn: Option<i64> = None) -> /*id*/ i64;
       fn createFileAttribute(parentIdIn: i64, attrTypeIdIn: i64, descriptionIn: String, originalFileDateIn: i64, storedDateIn: i64,
                               original_file_path_in: String, readableIn: Boolean, writableIn: Boolean, executableIn: Boolean, sizeIn: i64,
                               md5hashIn: String, inputStreamIn: java.io.FileInputStream, sortingIndexIn: Option<i64> = None) -> /*id*/ i64;
       fn createTextAttribute(parentIdIn: i64, attrTypeIdIn: i64, textIn: String, valid_on_date_in: Option<i64> = None,
                               observationDateIn: i64 = System.currentTimeMillis(), callerManagesTransactionsIn: Boolean = false,
                               sortingIndexIn: Option<i64> = None) -> /*id*/ i64;
       fn createRelationToLocalEntity(relationTypeIdIn: i64, entityId1In: i64, entityId2In: i64, valid_on_date_in: Option<i64>, observationDateIn: i64,
                                  sortingIndexIn: Option<i64> = None, callerManagesTransactionsIn: Boolean = false) -> RelationToLocalEntity;
       fn createRelationToRemoteEntity(relationTypeIdIn: i64, entityId1In: i64, entityId2In: i64, valid_on_date_in: Option<i64>, observationDateIn: i64,
                                        remoteInstanceIdIn: String, sortingIndexIn: Option<i64> = None,
                                        callerManagesTransactionsIn: Boolean = false) -> RelationToRemoteEntity;
       fn createGroupAndRelationToGroup(entityIdIn: i64, relationTypeIdIn: i64, newGroupNameIn: String, allowMixedClassesInGroupIn: Boolean = false,
                                         valid_on_date_in: Option<i64>, observationDateIn: i64,
                                         sortingIndexIn: Option<i64>, callerManagesTransactionsIn: Boolean = false) -> (i64, i64);
       pub fn createEntity(name_in: String, classIdIn: Option<i64> = None, isPublicIn: Option<bool> = None) -> /*id*/ i64;
       fn createEntityAndRelationToLocalEntity(entityIdIn: i64, relationTypeIdIn: i64, newEntityNameIn: String, isPublicIn: Option<bool>,
                                           valid_on_date_in: Option<i64>, observationDateIn: i64, callerManagesTransactionsIn: Boolean = false) -> (i64, i64);
       fn createRelationToGroup(entityIdIn: i64, relationTypeIdIn: i64, groupIdIn: i64, valid_on_date_in: Option<i64>, observationDateIn: i64,
                                 sortingIndexIn: Option<i64> = None, callerManagesTransactionsIn: Boolean = false) -> (i64, i64);
       fn addEntityToGroup(groupIdIn: i64, containedEntityIdIn: i64, sortingIndexIn: Option<i64> = None, callerManagesTransactionsIn: Boolean = false);
       pub fn createOmInstance(id_in: String, isLocalIn: Boolean, addressIn: String, entityIdIn: Option<i64> = None, oldTableName: Boolean = false) -> i64;
       fn addHASRelationToLocalEntity(fromEntityIdIn: i64, toEntityIdIn: i64, valid_on_date_in: Option<i64>, observationDateIn: i64,
                                  sortingIndexIn: Option<i64> = None) -> RelationToLocalEntity;
       pub fn getOrCreateClassAndTemplateEntity(classNameIn: String, callerManagesTransactionsIn: Boolean) -> (i64, i64);
       pub fn createRelationType(name_in: String, name_in_reverseDirectionIn: String, directionalityIn: String) -> /*id*/ i64;
       pub fn createClassAndItsTemplateEntity(classNameIn: String) -> (i64, i64);
       fn addUriEntityWithUriAttribute(containingEntityIn: Entity, newEntityNameIn: String, uriIn: String, observationDateIn: i64,
                                        makeThemPublicIn: Option<bool>, callerManagesTransactionsIn: Boolean,
                                        quoteIn: Option<String> = None) -> (Entity, RelationToLocalEntity);


       pub fn attribute_key_exists(formIdIn: i64, id_in: i64) -> Boolean;
       fn findContainedLocalEntityIds(resultsInOut: mutable.TreeSet[i64], fromEntityIdIn: i64, searchStringIn: String,
                                  levelsRemaining: Int = 20, stopAfterAnyFound: Boolean = true) -> mutable.TreeSet[i64];

       fn entity_key_exists(&self, id_in: i64, include_archived: Boolean /*= true%%*/) -> bool;
       fn relationTypeKeyExists(id_in: i64) -> Boolean;
       fn quantityAttributeKeyExists(id_in: i64) -> Boolean;
       fn dateAttributeKeyExists(id_in: i64) -> Boolean;
       fn booleanAttributeKeyExists(id_in: i64) -> Boolean;
       fn fileAttributeKeyExists(id_in: i64) -> Boolean;
       fn textAttributeKeyExists(id_in: i64) -> Boolean;
       pub fn relationToLocal_entity_key_exists(id_in: i64) -> Boolean;
       pub fn groupKeyExists(id_in: i64) -> Boolean;
       fn relationToGroupKeysExistAndMatch(id: i64, entityId: i64, relTypeId: i64, groupId: i64) -> Boolean;
       fn classKeyExists(id_in: i64) -> Boolean;
       fn omInstanceKeyExists(id_in: String) -> Boolean;
       fn getEntityData(id_in: i64) -> Array[Option[Any]];
       fn getEntityName(id_in: i64) -> Option<String>;
       fn isDuplicateEntityName(name_in: String, selfIdToIgnoreIn: Option<i64> = None) -> Boolean;
       fn getSortedAttributes(entityIdIn: i64, startingObjectIndexIn: Int = 0, maxValsIn: Int = 0,
                               onlyPublicEntitiesIn: Boolean = true) -> (Array[(i64, Attribute)], Int);
       pub fn findRelationType(type_name_in: String, expectedRows: Option[Int] = Some(1)) -> java.util.ArrayList[i64];
       fn getRelationTypeData(id_in: i64) -> Array[Option[Any]];
       fn getQuantityAttributeData(id_in: i64) -> Array[Option[Any]];
       fn getDateAttributeData(id_in: i64) -> Array[Option[Any]];
       fn getBooleanAttributeData(id_in: i64) -> Array[Option[Any]];
       fn getFileAttributeData(id_in: i64) -> Array[Option[Any]];
       fn getFileAttributeContent(fileAttributeIdIn: i64, outputStreamIn: java.io.OutputStream) -> (i64, String);
       fn getTextAttributeData(id_in: i64) -> Array[Option[Any]];
       fn relationToLocalEntityKeysExistAndMatch(id_in: i64, relTypeIdIn: i64, entityId1In: i64, entityId2In: i64) -> Boolean;
       fn relationToRemote_entity_key_exists(id_in: i64) -> Boolean;
       fn relationToRemoteEntityKeysExistAndMatch(id_in: i64, relTypeIdIn: i64, entityId1In: i64, remoteInstanceIdIn: String, entityId2In: i64) -> Boolean;
       fn getRelationToLocalEntityData(relTypeIdIn: i64, entityId1In: i64, entityId2In: i64) -> Array[Option[Any]];
       fn getRelationToLocalEntityDataById(id_in: i64) -> Array[Option[Any]];
       fn getRelationToRemoteEntityData(relTypeIdIn: i64, entityId1In: i64, remoteInstanceIdIn: String, entityId2In: i64) -> Array[Option[Any]];
       fn getGroupData(id_in: i64) -> Array[Option[Any]];
       fn getGroupEntryObjects(groupIdIn: i64, startingObjectIndexIn: i64, maxValsIn: Option<i64> = None) -> Vec<Entity>;
       pub fn getGroupSize(groupIdIn: i64, includeWhichEntitiesIn: Int = 3) -> i64;
       fn getHighestSortingIndexForGroup(groupIdIn: i64) -> i64;
       fn getRelationToGroupDataByKeys(entityId: i64, relTypeId: i64, groupId: i64) -> Array[Option[Any]];
       fn getRelationToGroupData(id_in: i64) -> Array[Option[Any]];
       pub fn getGroupEntriesData(groupIdIn: i64, limitIn: Option<i64> = None, include_archived_entitiesIn: Boolean = true) -> List[Array[Option[Any]]];
       fn findRelationToAndGroup_OnEntity(entityIdIn: i64,
                                                            groupNameIn: Option<String> = None) -> (Option<i64>, Option<i64>, Option<i64>, Option<String>, Boolean);
       pub fn getEntitiesContainingGroup(groupIdIn: i64, startingIndexIn: i64, maxValsIn: Option<i64> = None) -> java.util.ArrayList[(i64, Entity)];
       fn getCountOfEntitiesContainingGroup(groupIdIn: i64) -> (i64, i64);
       fn getClassData(id_in: i64) -> Array[Option[Any]];
       fn getAttributeCount(entityIdIn: i64, include_archived_entitiesIn: Boolean = false) -> i64;
       fn getRelationToLocalEntityCount(entityIdIn: i64, include_archived_entities: Boolean = false) -> i64;
       fn getRelationToRemoteEntityCount(entityIdIn: i64) -> i64;
       fn getRelationToGroupCount(entityIdIn: i64) -> i64;
       pub fn getClassCount(entityIdIn: Option<i64> = None) -> i64;
       fn getClassName(id_in: i64) -> Option<String>;
       fn getOmInstanceData(id_in: String) -> Array[Option[Any]];
       fn isDuplicateOmInstanceAddress(addressIn: String, selfIdToIgnoreIn: Option<String> = None) -> Boolean;
       fn  getGroupsContainingEntitysGroupsIds(groupIdIn: i64, limitIn: Option<i64> = Some(5)) -> List[Array[Option[Any]]];
       fn isEntityInGroup(groupIdIn: i64, entityIdIn: i64) -> Boolean;
       fn getAdjacentGroupEntriesSortingIndexes(groupIdIn: i64, sortingIndexIn: i64, limitIn: Option<i64> = None,
                                                 forwardNotBackIn: Boolean) -> List[Array[Option[Any]]];
       fn getNearestGroupEntrysSortingIndex(groupIdIn: i64, startingPointSortingIndexIn: i64, forwardNotBackIn: Boolean) -> Option<i64>;
       fn getAdjacentAttributesSortingIndexes(entityIdIn: i64, sortingIndexIn: i64, limitIn: Option<i64>, forwardNotBackIn: Boolean) -> List[Array[Option[Any]]];
       fn getNearestAttributeEntrysSortingIndex(entityIdIn: i64, startingPointSortingIndexIn: i64, forwardNotBackIn: Boolean) -> Option<i64>;
       fn getEntityAttributeSortingIndex(entityIdIn: i64, attributeFormIdIn: i64, attributeIdIn: i64) -> i64;
       fn getGroupEntrySortingIndex(groupIdIn: i64, entityIdIn: i64) -> i64;
       fn isGroupEntrySortingIndexInUse(groupIdIn: i64, sortingIndexIn: i64) -> Boolean;
       fn isAttributeSortingIndexInUse(entityIdIn: i64, sortingIndexIn: i64) -> Boolean;
       fn findUnusedAttributeSortingIndex(entityIdIn: i64, startingWithIn: Option<i64> = None) -> i64;
       pub fn findAllEntityIdsByName(name_in: String, caseSensitive: Boolean = false) -> java.util.ArrayList[i64];
       fn findUnusedGroupSortingIndex(groupIdIn: i64, startingWithIn: Option<i64> = None) -> i64;
       fn getTextAttributeByTypeId(parentEntityIdIn: i64, typeIdIn: i64, expectedRows: Option[Int] = None) -> java.util.ArrayList[TextAttribute];
       fn getLocalEntitiesContainingLocalEntity(entityIdIn: i64, startingIndexIn: i64, maxValsIn: Option<i64> = None) -> java.util.ArrayList[(i64, Entity)];
       fn getCountOfGroupsContainingEntity(entityIdIn: i64) -> i64;
       fn getContainingGroupsIds(entityIdIn: i64) -> java.util.ArrayList[i64];
       fn getContainingRelationsToGroup(entityIdIn: i64, startingIndexIn: i64,
                                                          maxValsIn: Option<i64> = None) -> java.util.ArrayList[RelationToGroup];
     //  fn getShouldCreateDefaultAttributes(classIdIn: i64) -> Option<bool>;
       fn updateClassCreateDefaultAttributes(classIdIn: i64, value -> Option<bool>);
       pub fn getEntitiesOnlyCount(limitByClass: Boolean = false, classIdIn: Option<i64> = None, templateEntity: Option<i64> = None) -> i64;
       fn getCountOfLocalEntitiesContainingLocalEntity(entityIdIn: i64) -> (i64, i64);
       //idea (tracked): make "*duplicate*" methods just be ... called "search"? combine w/ search, or rename? makes sense for callers?
       pub fn isDuplicateClassName(name_in: String, selfIdToIgnoreIn: Option<i64> = None) -> Boolean;
       fn getContainingRelationToGroupDescriptions(entityIdIn: i64, limitIn: Option<i64> = None) -> util.ArrayList[String];
       pub fn getMatchingEntities(startingObjectIndexIn: i64, maxValsIn: Option<i64> = None, omitEntityIdIn: Option<i64>,
                               nameRegexIn: String) -> Vec<Entity>;
       pub fn getMatchingGroups(startingObjectIndexIn: i64, maxValsIn: Option<i64> = None, omitGroupIdIn: Option<i64>,
                             nameRegexIn: String) -> java.util.ArrayList[Group];
       fn getRelationsToGroupContainingThisGroup(groupIdIn: i64, startingIndexIn: i64,
                                                                   maxValsIn: Option<i64> = None) -> java.util.ArrayList[RelationToGroup];
       pub fn getEntities(startingObjectIndexIn: i64, maxValsIn: Option<i64> = None) -> Vec<Entity>;
       pub fn getEntitiesOnly(startingObjectIndexIn: i64, maxValsIn: Option<i64> = None, classIdIn: Option<i64> = None,
                           limitByClass: Boolean = false, templateEntity: Option<i64> = None,
                           groupToOmitIdIn: Option<i64> = None) -> Vec<Entity>;
       pub fn getCountOfEntitiesUsedAsAttributeTypes(objectTypeIn: String, quantitySeeksUnitNotTypeIn: Boolean) -> i64;
       pub fn getEntitiesUsedAsAttributeTypes(objectTypeIn: String, startingObjectIndexIn: i64, maxValsIn: Option<i64> = None,
                                           quantitySeeksUnitNotTypeIn: Boolean) -> Vec<Entity>;
       pub fn getRelationTypes(startingObjectIndexIn: i64, maxValsIn: Option<i64> = None) -> Vec<Entity>;
       pub fn getClasses(startingObjectIndexIn: i64, maxValsIn: Option<i64> = None) -> java.util.ArrayList[EntityClass];
       pub fn getRelationTypeCount -> i64;
       pub fn getOmInstanceCount -> i64;
       pub fn getEntityCount -> i64;
       pub fn findJournalEntries(startTimeIn: i64, endTimeIn: i64, limitIn: Option<i64> = None) -> util.ArrayList[(i64, String, i64)];
       pub fn getGroupCount -> i64;
       pub fn getGroups(startingObjectIndexIn: i64, maxValsIn: Option<i64> = None, groupToOmitIdIn: Option<i64> = None) -> java.util.ArrayList[Group];
       pub fn createGroup(name_in: String, allowMixedClassesInGroupIn: Boolean = false) -> i64;
       pub fn relationToGroupKeyExists(id_in: i64) -> Boolean;


       fn updateEntitysClass(entityId: i64, classId: Option<i64>, callerManagesTransactions -> Boolean = false);
       fn updateEntityOnlyNewEntriesStickToTop(id_in: i64, newEntriesStickToTop -> Boolean);
       fn archiveEntity(id_in: i64, callerManagesTransactionsIn -> Boolean = false);
       fn unarchiveEntity(id_in: i64, callerManagesTransactionsIn -> Boolean = false);
       pub fn set_include_archived_entities(in: Boolean) -> Unit;
       pub fn setUserPreference_EntityId(name_in: String, entityIdIn -> i64);
       fn updateEntityOnlyPublicStatus(id_in: i64, value -> Option<bool>);
       fn updateQuantityAttribute(id_in: i64, parentIdIn: i64, attrTypeIdIn: i64, unitIdIn: i64, numberIn: Float, valid_on_date_in: Option<i64>,
                                   inObservationDate -> i64);
       fn updateDateAttribute(id_in: i64, parentIdIn: i64, date_in: i64, attrTypeIdIn -> i64);
       fn updateBooleanAttribute(id_in: i64, parentIdIn: i64, attrTypeIdIn: i64, booleanIn: Boolean, valid_on_date_in: Option<i64>,
                                  inObservationDate -> i64);
       fn updateFileAttribute(id_in: i64, parentIdIn: i64, attrTypeIdIn: i64, descriptionIn -> String);
       fn updateFileAttribute(id_in: i64, parentIdIn: i64, attrTypeIdIn: i64, descriptionIn: String,
                                                originalFileDateIn: i64, storedDateIn: i64,
                               original_file_path_in: String, readableIn: Boolean, writableIn: Boolean, executableIn: Boolean, sizeIn: i64, md5hashIn: String);
       fn updateTextAttribute(id_in: i64, parentIdIn: i64, attrTypeIdIn: i64, textIn: String, valid_on_date_in: Option<i64>,
                                                observationDateIn -> i64);
       fn updateRelationToLocalEntity(oldRelationTypeIdIn: i64, entityId1In: i64, entityId2In: i64,
                                  newRelationTypeIdIn: i64, valid_on_date_in: Option<i64>, observationDateIn -> i64);
       fn updateRelationToRemoteEntity(oldRelationTypeIdIn: i64, entityId1In: i64, remoteInstanceIdIn: String, entityId2In: i64,
                                        newRelationTypeIdIn: i64, valid_on_date_in: Option<i64>, observationDateIn -> i64);
       fn updateGroup(groupIdIn: i64, name_in: String, allowMixedClassesInGroupIn: Boolean = false, newEntriesStickToTopIn -> Boolean = false);
       fn updateRelationToGroup(entityIdIn: i64, oldRelationTypeIdIn: i64, newRelationTypeIdIn: i64, oldGroupIdIn: i64, newGroupIdIn: i64,
                                 valid_on_date_in: Option<i64>, observationDateIn -> i64);
       fn moveRelationToLocalEntityToLocalEntity(rtleIdIn: i64, newContainingEntityIdIn: i64,
                                                                   sortingIndexIn: i64) -> RelationToLocalEntity;
       fn moveRelationToRemoteEntityToLocalEntity(remoteInstanceIdIn: String, relationToRemoteEntityIdIn: i64, toContainingEntityIdIn: i64,
                                                                    sortingIndexIn: i64) -> RelationToRemoteEntity;
       fn moveLocalEntityFromLocalEntityToGroup(removingRtleIn: RelationToLocalEntity, targetGroupIdIn: i64, sortingIndexIn: i64);
       fn moveRelationToGroup(relationToGroupIdIn: i64, newContainingEntityIdIn: i64, sortingIndexIn: i64) -> i64;
       fn moveEntityFromGroupToLocalEntity(fromGroupIdIn: i64, toEntityIdIn: i64, moveEntityIdIn: i64, sortingIndexIn: i64);
       fn moveLocalEntityFromGroupToGroup(fromGroupIdIn: i64, toGroupIdIn: i64, moveEntityIdIn: i64, sortingIndexIn: i64);
       fn renumberSortingIndexes(entityIdOrGroupIdIn: i64, callerManagesTransactionsIn: Boolean = false,
                                                   isEntityAttrsNotGroupEntries: Boolean = true);
       fn updateAttributeSortingIndex(entityIdIn: i64, attributeFormIdIn: i64, attributeIdIn: i64, sortingIndexIn: i64);
       fn updateSortingIndexInAGroup(groupIdIn: i64, entityIdIn: i64, sortingIndexIn: i64);
       fn updateEntityOnlyName(id_in: i64, name_in: String);
       fn updateRelationType(id_in: i64, name_in: String, name_in_reverseDirectionIn: String, directionalityIn: String);
       fn updateClassAndTemplateEntityName(classIdIn: i64, name: String) -> i64;
       fn updateOmInstance(id_in: String, addressIn: String, entityIdIn: Option<i64>);

       fn deleteEntity(id_in: i64, callerManagesTransactionsIn: Boolean = false);
       fn deleteQuantityAttribute(id_in: i64);
       fn deleteDateAttribute(id_in: i64);
       fn deleteBooleanAttribute(id_in: i6;4)
       fn deleteFileAttribute(id_in: i64);
       fn deleteTextAttribute(id_in: i64);
       fn deleteRelationToLocalEntity(relTypeIdIn: i64, entityId1In: i64, entityId2In: i64);
       fn deleteRelationToRemoteEntity(relTypeIdIn: i64, entityId1In: i64, remoteInstanceIdIn: String, entityId2In: i64);
       fn deleteRelationToGroup(entityIdIn: i64, relTypeIdIn: i64, groupIdIn: i64);
       fn deleteGroupAndRelationsToIt(id_in: i64);
       fn deleteRelationType(id_in: i64);
       fn deleteClassAndItsTemplateEntity(classIdIn: i64);
       fn deleteGroupRelationsToItAndItsEntries(groupIdIn: i64);
       fn deleteOmInstance(id_in: String) -> Unit;
       fn removeEntityFromGroup(groupIdIn: i64, containedEntityIdIn: i64, callerManagesTransactionsIn: Boolean = false);


       // (See comments above the set of these methods, in RestDatabase.scala:)
       pub fn getUserPreference_Boolean(preferenceNameIn: String, default_value_in: Option<bool> = None) -> Option<bool>;
       pub fn getPreferencesContainerId() -> i64;
       pub fn getUserPreference_EntityId(preferenceNameIn: String, default_value_in: Option<i64> = None) -> Option<i64>;
       pub fn getOmInstances(localIn: Option<bool> = None) -> java.util.ArrayList[OmInstance];

    */
}
