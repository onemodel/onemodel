/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2016-2017 inclusive, 2020, and 2023, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
/* %% package org.onemodel.core.model
import java.util
import org.onemodel.core.{OmDatabaseException, Util}
import scala.collection.mutable
*/

// use std::string::ToString;
// use crate::model::postgresql_database::PostgreSQLDatabase;

// pub struct Database {
// }

pub trait Database {
    const DB_NAME_PREFIX: &'static str = "om_";
    // If next line ever changes, search the code for other places that also have it hard-coded, to change also
    // (ex., INSTALLING, first.exp or its successors, any .psql scripts, ....  "t1/x" is shorter to type
    // during manual testing than "testrunner/testrunner".
    const TEST_USER: &'static str = "t1";
    const TEST_PASS: &'static str = "x";
    /*%%$%WHY cant i see the below constants in other scopes? mbe make things compile w/ lifetimes per beleow then retry? (i nfiles database, main, ~pgdb.rs).
    then cont reading re lifetimes and fix below errs and others in cr or build ^B output
    BETR IDEA: see if any used in stdlib, how referred to?
    ideas: make pub? put in dift/broader scope like main then move here until i see?
    idea: use static instead? read more re static & const?
    */
    const MIXED_CLASSES_EXCEPTION: &'static str = "All the entities in a group should be of the same class.";
    // so named to make it unlikely to collide by name with anything else:
    const SYSTEM_ENTITY_NAME: &'static str = ".system-use-only";
    // aka template entities:
    const CLASS_TEMPLATE_ENTITY_GROUP_NAME: &'static str = "class-defining entities";
    const THE_HAS_RELATION_TYPE_NAME: &'static str = "has";
    const THE_IS_HAD_BY_REVERSE_NAME: &'static str = "is had by";
    const EDITOR_INFO_ENTITY_NAME: &'static str = "editorInfo";
    const TEXT_EDITOR_INFO_ENTITY_NAME: &'static str = "textEditorInfo";
    const TEXT_EDITOR_COMMAND_ATTRIBUTE_TYPE_NAME: &'static str = "textEditorCommand";
    const PREF_TYPE_BOOLEAN: &'static str = "boolean";
    const PREF_TYPE_ENTITY_ID: &'static str = "entityId";
    const TEMPLATE_NAME_SUFFIX: &'static str = "-template";
    const UNUSED_GROUP_ERR1: &'static str = "No available index found which is not already used. How would so many be used?";
    const UNUSED_GROUP_ERR2: &'static str = "Very unexpected, but could it be that you are running out of available sorting indexes!?  Have someone check, before you need to create, for example, a thousand more entities.";
    const GET_CLASS_DATA__RESULT_TYPES: &'static str = "String,i64,Boolean";
    const GET_RELATION_TYPE_DATA__RESULT_TYPES: &'static str = "String,String,String";
    const GET_OM_INSTANCE_DATA__RESULT_TYPES: &'static str = "Boolean,String,i64,i64";
    const GET_QUANTITY_ATTRIBUTE_DATA__RESULT_TYPES: &'static str = "i64,i64,Float,i64,i64,i64,i64";
    const GET_DATE_ATTRIBUTE_DATA__RESULT_TYPES: &'static str = "i64,i64,i64,i64";
    const GET_BOOLEAN_ATTRIBUTE_DATA__RESULT_TYPES: &'static str = "i64,Boolean,i64,i64,i64,i64";
    const GET_FILE_ATTRIBUTE_DATA__RESULT_TYPES: &'static str = "i64,String,i64,i64,i64,String,Boolean,Boolean,Boolean,i64,String,i64";
    const GET_TEXT_ATTRIBUTE_DATA__RESULT_TYPES: &'static str = "i64,String,i64,i64,i64,i64";
    const GET_RELATION_TO_GROUP_DATA_BY_ID__RESULT_TYPES: &'static str = "i64,i64,i64,i64,i64,i64,i64";
    const GET_RELATION_TO_GROUP_DATA_BY_KEYS__RESULT_TYPES: &'static str = "i64,i64,i64,i64,i64,i64,i64";
    const GET_RELATION_TO_LOCAL_ENTITY__RESULT_TYPES: &'static str = "i64,i64,i64,i64";
    const GET_RELATION_TO_REMOTE_ENTITY__RESULT_TYPES: &'static str = "i64,i64,i64,i64";
    const GET_GROUP_DATA__RESULT_TYPES: &'static str = "String,i64,Boolean,Boolean";
    const GET_ENTITY_DATA__RESULT_TYPES: &'static str = "String,i64,i64,Boolean,Boolean,Boolean";
    const GET_GROUP_ENTRIES_DATA__RESULT_TYPES: &'static str = "i64,i64";

    /*
        //%%$% next?: read up on traits and using them. the fns below, to be impld by pgdb .rs file.
        fn isRemote() -> bool;
        fn getRemoteAddress() -> Option[String] = None;
        // let id: String; // %%used for?
        fn includeArchivedEntities() -> bool;
        fn beginTrans();
        fn rollbackTrans();
        fn commitTrans();

        // where we create the table also calls this.
        // Longer than the old 60 (needed), and a likely familiar length to many people (for ease in knowing when done), seems a decent balance. If any longer
        // is needed, maybe it should be put in a TextAttribute and make those more convenient to use, instead.
        pub fn entityNameLength() -> Int { 160 }

        // in postgres, one table "extends" the other (see comments in createTables)
        pub fn relationTypeNameLength() -> Int {
            entityNameLength
        }

        pub fn classNameLength() -> Int {
            entityNameLength
        }

        // (See usages. The DNS hostname max size seems to be 255 plus 1 null, but the ":<port>" part could add 6 more chars (they seem to go up to :65535).
        // Maybe someday we will have to move to a larger size in case it changes or uses unicode or I don't know what.)
        pub fn omInstanceAddressLength() -> Int {
            262
        }

        pub fn getAttributeFormId(key: String) -> Int {
          //MAKE SURE THESE MATCH WITH THOSE IN attributeKeyExists and getAttributeFormName, and the range in the db constraint valid_attribute_form_id ,
          // and in RestDatabase.processArrayOfTuplesAndInt !
          key match {
            case Util.QUANTITY_TYPE => 1
            case Util.DATE_TYPE => 2
            case Util.BOOLEAN_TYPE => 3
            case Util.FILE_TYPE => 4
            case Util.TEXT_TYPE => 5
            case Util.RELATION_TO_LOCAL_ENTITY_TYPE => 6
            case "RelationToLocalEntity" => 6
            case Util.RELATION_TO_GROUP_TYPE => 7
            case Util.RELATION_TO_REMOTE_ENTITY_TYPE => 8
            case _ => throw new OmDatabaseException("unexpected")
          }
        }
        pub fn getAttributeFormName(key: Int) -> String {
          // MAKE SURE THESE MATCH WITH THOSE IN getAttributeFormId !
          //idea: put these values in a structure that is looked up both ways, instead of duplicating them?
          key match {
            case 1 => Util.QUANTITY_TYPE
            case 2 => Util.DATE_TYPE
            case 3 => Util.BOOLEAN_TYPE
            case 4 => Util.FILE_TYPE
            case 5 => Util.TEXT_TYPE
            case 6 => Util.RELATION_TO_LOCAL_ENTITY_TYPE
            case 7 => Util.RELATION_TO_GROUP_TYPE
            case 8 => Util.RELATION_TO_REMOTE_ENTITY_TYPE
            case _ => throw new OmDatabaseException("unexpected")
          }
        }

        pub fn maxIdValue() -> i64 {
            //%%
          // Max size for a Java long type, and for a postgresql 7.2.1 bigint type (which is being used, at the moment, for the id value in Entity table.
          // (these values are from file:///usr/share/doc/postgresql-doc-9.1/html/datatype-numeric.html)
          9223372036854775807L
        }

        pub fn minIdValue() -> i64 {
            //%%
          -9223372036854775808L
        }
    */

    //%%$%%
    // mbe moving to be inside pgsql .rs instead..?
    // fn login(username: &str, password: &str) -> Result<Database, &'static str> {
    //     PostgreSQLDatabase::new(username, password)
    // }
/*
    pub fn getRestDatabase(remoteAddressIn: String) -> RestDatabase {
      new RestDatabase(remoteAddressIn)
    }

    pub fn currentOrRemoteDb(relationToEntityIn: Attribute, currentDb: Database) -> Database {
      require(relationToEntityIn.isInstanceOf[RelationToLocalEntity] || relationToEntityIn.isInstanceOf[RelationToRemoteEntity])

      // Can't use ".isRemote" here because a RelationToRemoteEntity is stored locally (so would say false),
      // but refers to an entity which is remote (so we want the next line to be true in that case):
      //noinspection TypeCheckCanBeMatch
      if (relationToEntityIn.isInstanceOf[RelationToRemoteEntity]) {
        relationToEntityIn.asInstanceOf[RelationToRemoteEntity].getRemoteDatabase
      } else if (relationToEntityIn.isInstanceOf[RelationToLocalEntity]) {
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
    fn createQuantityAttribute(parentIdIn: i64, attrTypeIdIn: i64, unitIdIn: i64, numberIn: Float, validOnDateIn: Option[i64],
                                inObservationDate: i64, callerManagesTransactionsIn: Boolean = false, sortingIndexIn: Option[i64] = None) -> /*id*/ i64;
    fn createDateAttribute(parentIdIn: i64, attrTypeIdIn: i64, dateIn: i64, sortingIndexIn: Option[i64] = None) -> /*id*/ i64;
    fn createBooleanAttribute(parentIdIn: i64, attrTypeIdIn: i64, booleanIn: Boolean, validOnDateIn: Option[i64], observationDateIn: i64,
                               sortingIndexIn: Option[i64] = None) -> /*id*/ i64;
    fn createFileAttribute(parentIdIn: i64, attrTypeIdIn: i64, descriptionIn: String, originalFileDateIn: i64, storedDateIn: i64,
                            originalFilePathIn: String, readableIn: Boolean, writableIn: Boolean, executableIn: Boolean, sizeIn: i64,
                            md5hashIn: String, inputStreamIn: java.io.FileInputStream, sortingIndexIn: Option[i64] = None) -> /*id*/ i64;
    fn createTextAttribute(parentIdIn: i64, attrTypeIdIn: i64, textIn: String, validOnDateIn: Option[i64] = None,
                            observationDateIn: i64 = System.currentTimeMillis(), callerManagesTransactionsIn: Boolean = false,
                            sortingIndexIn: Option[i64] = None) -> /*id*/ i64;
    fn createRelationToLocalEntity(relationTypeIdIn: i64, entityId1In: i64, entityId2In: i64, validOnDateIn: Option[i64], observationDateIn: i64,
                               sortingIndexIn: Option[i64] = None, callerManagesTransactionsIn: Boolean = false) -> RelationToLocalEntity;
    fn createRelationToRemoteEntity(relationTypeIdIn: i64, entityId1In: i64, entityId2In: i64, validOnDateIn: Option[i64], observationDateIn: i64,
                                     remoteInstanceIdIn: String, sortingIndexIn: Option[i64] = None,
                                     callerManagesTransactionsIn: Boolean = false) -> RelationToRemoteEntity;
    fn createGroupAndRelationToGroup(entityIdIn: i64, relationTypeIdIn: i64, newGroupNameIn: String, allowMixedClassesInGroupIn: Boolean = false,
                                      validOnDateIn: Option[i64], observationDateIn: i64,
                                      sortingIndexIn: Option[i64], callerManagesTransactionsIn: Boolean = false) -> (i64, i64);
    pub fn createEntity(nameIn: String, classIdIn: Option[i64] = None, isPublicIn: Option[Boolean] = None) -> /*id*/ i64;
    fn createEntityAndRelationToLocalEntity(entityIdIn: i64, relationTypeIdIn: i64, newEntityNameIn: String, isPublicIn: Option[Boolean],
                                        validOnDateIn: Option[i64], observationDateIn: i64, callerManagesTransactionsIn: Boolean = false) -> (i64, i64);
    fn createRelationToGroup(entityIdIn: i64, relationTypeIdIn: i64, groupIdIn: i64, validOnDateIn: Option[i64], observationDateIn: i64,
                              sortingIndexIn: Option[i64] = None, callerManagesTransactionsIn: Boolean = false) -> (i64, i64);
    fn addEntityToGroup(groupIdIn: i64, containedEntityIdIn: i64, sortingIndexIn: Option[i64] = None, callerManagesTransactionsIn: Boolean = false);
    pub fn createOmInstance(idIn: String, isLocalIn: Boolean, addressIn: String, entityIdIn: Option[i64] = None, oldTableName: Boolean = false) -> i64;
    fn addHASRelationToLocalEntity(fromEntityIdIn: i64, toEntityIdIn: i64, validOnDateIn: Option[i64], observationDateIn: i64,
                               sortingIndexIn: Option[i64] = None) -> RelationToLocalEntity;
    pub fn getOrCreateClassAndTemplateEntity(classNameIn: String, callerManagesTransactionsIn: Boolean) -> (i64, i64);
    pub fn createRelationType(nameIn: String, nameInReverseDirectionIn: String, directionalityIn: String) -> /*id*/ i64;
    pub fn createClassAndItsTemplateEntity(classNameIn: String) -> (i64, i64);
    fn addUriEntityWithUriAttribute(containingEntityIn: Entity, newEntityNameIn: String, uriIn: String, observationDateIn: i64,
                                     makeThemPublicIn: Option[Boolean], callerManagesTransactionsIn: Boolean,
                                     quoteIn: Option[String] = None) -> (Entity, RelationToLocalEntity);


    pub fn attributeKeyExists(formIdIn: i64, idIn: i64) -> Boolean;
    fn findContainedLocalEntityIds(resultsInOut: mutable.TreeSet[i64], fromEntityIdIn: i64, searchStringIn: String,
                               levelsRemaining: Int = 20, stopAfterAnyFound: Boolean = true) -> mutable.TreeSet[i64];
    pub fn entityKeyExists(idIn: i64, includeArchived: Boolean = true) -> Boolean;
    fn relationTypeKeyExists(idIn: i64) -> Boolean;
    fn quantityAttributeKeyExists(idIn: i64) -> Boolean;
    fn dateAttributeKeyExists(idIn: i64) -> Boolean;
    fn booleanAttributeKeyExists(idIn: i64) -> Boolean;
    fn fileAttributeKeyExists(idIn: i64) -> Boolean;
    fn textAttributeKeyExists(idIn: i64) -> Boolean;
    pub fn relationToLocalEntityKeyExists(idIn: i64) -> Boolean;
    pub fn groupKeyExists(idIn: i64) -> Boolean;
    fn relationToGroupKeysExistAndMatch(id: i64, entityId: i64, relTypeId: i64, groupId: i64) -> Boolean;
    fn classKeyExists(idIn: i64) -> Boolean;
    fn omInstanceKeyExists(idIn: String) -> Boolean;
    fn getEntityData(idIn: i64) -> Array[Option[Any]];
    fn getEntityName(idIn: i64) -> Option[String];
    fn isDuplicateEntityName(nameIn: String, selfIdToIgnoreIn: Option[i64] = None) -> Boolean;
    fn getSortedAttributes(entityIdIn: i64, startingObjectIndexIn: Int = 0, maxValsIn: Int = 0,
                            onlyPublicEntitiesIn: Boolean = true) -> (Array[(i64, Attribute)], Int);
    pub fn findRelationType(typeNameIn: String, expectedRows: Option[Int] = Some(1)) -> java.util.ArrayList[i64];
    fn getRelationTypeData(idIn: i64) -> Array[Option[Any]];
    fn getQuantityAttributeData(idIn: i64) -> Array[Option[Any]];
    fn getDateAttributeData(idIn: i64) -> Array[Option[Any]];
    fn getBooleanAttributeData(idIn: i64) -> Array[Option[Any]];
    fn getFileAttributeData(idIn: i64) -> Array[Option[Any]];
    fn getFileAttributeContent(fileAttributeIdIn: i64, outputStreamIn: java.io.OutputStream) -> (i64, String);
    fn getTextAttributeData(idIn: i64) -> Array[Option[Any]];
    fn relationToLocalEntityKeysExistAndMatch(idIn: i64, relTypeIdIn: i64, entityId1In: i64, entityId2In: i64) -> Boolean;
    fn relationToRemoteEntityKeyExists(idIn: i64) -> Boolean;
    fn relationToRemoteEntityKeysExistAndMatch(idIn: i64, relTypeIdIn: i64, entityId1In: i64, remoteInstanceIdIn: String, entityId2In: i64) -> Boolean;
    fn getRelationToLocalEntityData(relTypeIdIn: i64, entityId1In: i64, entityId2In: i64) -> Array[Option[Any]];
    fn getRelationToLocalEntityDataById(idIn: i64) -> Array[Option[Any]];
    fn getRelationToRemoteEntityData(relTypeIdIn: i64, entityId1In: i64, remoteInstanceIdIn: String, entityId2In: i64) -> Array[Option[Any]];
    fn getGroupData(idIn: i64) -> Array[Option[Any]];
    fn getGroupEntryObjects(groupIdIn: i64, startingObjectIndexIn: i64, maxValsIn: Option[i64] = None) -> java.util.ArrayList[Entity];
    pub fn getGroupSize(groupIdIn: i64, includeWhichEntitiesIn: Int = 3) -> i64;
    fn getHighestSortingIndexForGroup(groupIdIn: i64) -> i64;
    fn getRelationToGroupDataByKeys(entityId: i64, relTypeId: i64, groupId: i64) -> Array[Option[Any]];
    fn getRelationToGroupData(idIn: i64) -> Array[Option[Any]];
    pub fn getGroupEntriesData(groupIdIn: i64, limitIn: Option[i64] = None, includeArchivedEntitiesIn: Boolean = true) -> List[Array[Option[Any]]];
    fn findRelationToAndGroup_OnEntity(entityIdIn: i64,
                                                         groupNameIn: Option[String] = None) -> (Option[i64], Option[i64], Option[i64], Option[String], Boolean);
    pub fn getEntitiesContainingGroup(groupIdIn: i64, startingIndexIn: i64, maxValsIn: Option[i64] = None) -> java.util.ArrayList[(i64, Entity)];
    fn getCountOfEntitiesContainingGroup(groupIdIn: i64) -> (i64, i64);
    fn getClassData(idIn: i64) -> Array[Option[Any]];
    fn getAttributeCount(entityIdIn: i64, includeArchivedEntitiesIn: Boolean = false) -> i64;
    fn getRelationToLocalEntityCount(entityIdIn: i64, includeArchivedEntities: Boolean = false) -> i64;
    fn getRelationToRemoteEntityCount(entityIdIn: i64) -> i64;
    fn getRelationToGroupCount(entityIdIn: i64) -> i64;
    pub fn getClassCount(entityIdIn: Option[i64] = None) -> i64;
    fn getClassName(idIn: i64) -> Option[String];
    fn getOmInstanceData(idIn: String) -> Array[Option[Any]];
    fn isDuplicateOmInstanceAddress(addressIn: String, selfIdToIgnoreIn: Option[String] = None) -> Boolean;
    fn  getGroupsContainingEntitysGroupsIds(groupIdIn: i64, limitIn: Option[i64] = Some(5)) -> List[Array[Option[Any]]];
    fn isEntityInGroup(groupIdIn: i64, entityIdIn: i64) -> Boolean;
    fn getAdjacentGroupEntriesSortingIndexes(groupIdIn: i64, sortingIndexIn: i64, limitIn: Option[i64] = None,
                                              forwardNotBackIn: Boolean) -> List[Array[Option[Any]]];
    fn getNearestGroupEntrysSortingIndex(groupIdIn: i64, startingPointSortingIndexIn: i64, forwardNotBackIn: Boolean) -> Option[i64];
    fn getAdjacentAttributesSortingIndexes(entityIdIn: i64, sortingIndexIn: i64, limitIn: Option[i64], forwardNotBackIn: Boolean) -> List[Array[Option[Any]]];
    fn getNearestAttributeEntrysSortingIndex(entityIdIn: i64, startingPointSortingIndexIn: i64, forwardNotBackIn: Boolean) -> Option[i64];
    fn getEntityAttributeSortingIndex(entityIdIn: i64, attributeFormIdIn: i64, attributeIdIn: i64) -> i64;
    fn getGroupEntrySortingIndex(groupIdIn: i64, entityIdIn: i64) -> i64;
    fn isGroupEntrySortingIndexInUse(groupIdIn: i64, sortingIndexIn: i64) -> Boolean;
    fn isAttributeSortingIndexInUse(entityIdIn: i64, sortingIndexIn: i64) -> Boolean;
    fn findUnusedAttributeSortingIndex(entityIdIn: i64, startingWithIn: Option[i64] = None) -> i64;
    pub fn findAllEntityIdsByName(nameIn: String, caseSensitive: Boolean = false) -> java.util.ArrayList[i64];
    fn findUnusedGroupSortingIndex(groupIdIn: i64, startingWithIn: Option[i64] = None) -> i64;
    fn getTextAttributeByTypeId(parentEntityIdIn: i64, typeIdIn: i64, expectedRows: Option[Int] = None) -> java.util.ArrayList[TextAttribute];
    fn getLocalEntitiesContainingLocalEntity(entityIdIn: i64, startingIndexIn: i64, maxValsIn: Option[i64] = None) -> java.util.ArrayList[(i64, Entity)];
    fn getCountOfGroupsContainingEntity(entityIdIn: i64) -> i64;
    fn getContainingGroupsIds(entityIdIn: i64) -> java.util.ArrayList[i64];
    fn getContainingRelationsToGroup(entityIdIn: i64, startingIndexIn: i64,
                                                       maxValsIn: Option[i64] = None) -> java.util.ArrayList[RelationToGroup];
  //  fn getShouldCreateDefaultAttributes(classIdIn: i64) -> Option[Boolean];
    fn updateClassCreateDefaultAttributes(classIdIn: i64, value -> Option[Boolean]);
    pub fn getEntitiesOnlyCount(limitByClass: Boolean = false, classIdIn: Option[i64] = None, templateEntity: Option[i64] = None) -> i64;
    fn getCountOfLocalEntitiesContainingLocalEntity(entityIdIn: i64) -> (i64, i64);
    //idea (tracked): make "*duplicate*" methods just be ... called "search"? combine w/ search, or rename? makes sense for callers?
    pub fn isDuplicateClassName(nameIn: String, selfIdToIgnoreIn: Option[i64] = None) -> Boolean;
    fn getContainingRelationToGroupDescriptions(entityIdIn: i64, limitIn: Option[i64] = None) -> util.ArrayList[String];
    pub fn getMatchingEntities(startingObjectIndexIn: i64, maxValsIn: Option[i64] = None, omitEntityIdIn: Option[i64],
                            nameRegexIn: String) -> java.util.ArrayList[Entity];
    pub fn getMatchingGroups(startingObjectIndexIn: i64, maxValsIn: Option[i64] = None, omitGroupIdIn: Option[i64],
                          nameRegexIn: String) -> java.util.ArrayList[Group];
    fn getRelationsToGroupContainingThisGroup(groupIdIn: i64, startingIndexIn: i64,
                                                                maxValsIn: Option[i64] = None) -> java.util.ArrayList[RelationToGroup];
    pub fn getEntities(startingObjectIndexIn: i64, maxValsIn: Option[i64] = None) -> java.util.ArrayList[Entity];
    pub fn getEntitiesOnly(startingObjectIndexIn: i64, maxValsIn: Option[i64] = None, classIdIn: Option[i64] = None,
                        limitByClass: Boolean = false, templateEntity: Option[i64] = None,
                        groupToOmitIdIn: Option[i64] = None) -> java.util.ArrayList[Entity];
    pub fn getCountOfEntitiesUsedAsAttributeTypes(objectTypeIn: String, quantitySeeksUnitNotTypeIn: Boolean) -> i64;
    pub fn getEntitiesUsedAsAttributeTypes(objectTypeIn: String, startingObjectIndexIn: i64, maxValsIn: Option[i64] = None,
                                        quantitySeeksUnitNotTypeIn: Boolean) -> java.util.ArrayList[Entity];
    pub fn getRelationTypes(startingObjectIndexIn: i64, maxValsIn: Option[i64] = None) -> java.util.ArrayList[Entity];
    pub fn getClasses(startingObjectIndexIn: i64, maxValsIn: Option[i64] = None) -> java.util.ArrayList[EntityClass];
    pub fn getRelationTypeCount -> i64;
    pub fn getOmInstanceCount -> i64;
    pub fn getEntityCount -> i64;
    pub fn findJournalEntries(startTimeIn: i64, endTimeIn: i64, limitIn: Option[i64] = None) -> util.ArrayList[(i64, String, i64)];
    pub fn getGroupCount -> i64;
    pub fn getGroups(startingObjectIndexIn: i64, maxValsIn: Option[i64] = None, groupToOmitIdIn: Option[i64] = None) -> java.util.ArrayList[Group];
    pub fn createGroup(nameIn: String, allowMixedClassesInGroupIn: Boolean = false) -> i64;
    pub fn relationToGroupKeyExists(idIn: i64) -> Boolean;


    fn updateEntitysClass(entityId: i64, classId: Option[i64], callerManagesTransactions -> Boolean = false);
    fn updateEntityOnlyNewEntriesStickToTop(idIn: i64, newEntriesStickToTop -> Boolean);
    fn archiveEntity(idIn: i64, callerManagesTransactionsIn -> Boolean = false);
    fn unarchiveEntity(idIn: i64, callerManagesTransactionsIn -> Boolean = false);
    pub fn setIncludeArchivedEntities(in: Boolean) -> Unit;
    pub fn setUserPreference_EntityId(nameIn: String, entityIdIn -> i64);
    fn updateEntityOnlyPublicStatus(idIn: i64, value -> Option[Boolean]);
    fn updateQuantityAttribute(idIn: i64, parentIdIn: i64, attrTypeIdIn: i64, unitIdIn: i64, numberIn: Float, validOnDateIn: Option[i64],
                                inObservationDate -> i64);
    fn updateDateAttribute(idIn: i64, parentIdIn: i64, dateIn: i64, attrTypeIdIn -> i64);
    fn updateBooleanAttribute(idIn: i64, parentIdIn: i64, attrTypeIdIn: i64, booleanIn: Boolean, validOnDateIn: Option[i64],
                               inObservationDate -> i64);
    fn updateFileAttribute(idIn: i64, parentIdIn: i64, attrTypeIdIn: i64, descriptionIn -> String);
    fn updateFileAttribute(idIn: i64, parentIdIn: i64, attrTypeIdIn: i64, descriptionIn: String,
                                             originalFileDateIn: i64, storedDateIn: i64,
                            originalFilePathIn: String, readableIn: Boolean, writableIn: Boolean, executableIn: Boolean, sizeIn: i64, md5hashIn: String);
    fn updateTextAttribute(idIn: i64, parentIdIn: i64, attrTypeIdIn: i64, textIn: String, validOnDateIn: Option[i64],
                                             observationDateIn -> i64);
    fn updateRelationToLocalEntity(oldRelationTypeIdIn: i64, entityId1In: i64, entityId2In: i64,
                               newRelationTypeIdIn: i64, validOnDateIn: Option[i64], observationDateIn -> i64);
    fn updateRelationToRemoteEntity(oldRelationTypeIdIn: i64, entityId1In: i64, remoteInstanceIdIn: String, entityId2In: i64,
                                     newRelationTypeIdIn: i64, validOnDateIn: Option[i64], observationDateIn -> i64);
    fn updateGroup(groupIdIn: i64, nameIn: String, allowMixedClassesInGroupIn: Boolean = false, newEntriesStickToTopIn -> Boolean = false);
    fn updateRelationToGroup(entityIdIn: i64, oldRelationTypeIdIn: i64, newRelationTypeIdIn: i64, oldGroupIdIn: i64, newGroupIdIn: i64,
                              validOnDateIn: Option[i64], observationDateIn -> i64);
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
    fn updateEntityOnlyName(idIn: i64, nameIn: String);
    fn updateRelationType(idIn: i64, nameIn: String, nameInReverseDirectionIn: String, directionalityIn: String);
    fn updateClassAndTemplateEntityName(classIdIn: i64, name: String) -> i64;
    fn updateOmInstance(idIn: String, addressIn: String, entityIdIn: Option[i64]);

    fn deleteEntity(idIn: i64, callerManagesTransactionsIn: Boolean = false);
    fn deleteQuantityAttribute(idIn: i64);
    fn deleteDateAttribute(idIn: i64);
    fn deleteBooleanAttribute(idIn: i6;4)
    fn deleteFileAttribute(idIn: i64);
    fn deleteTextAttribute(idIn: i64);
    fn deleteRelationToLocalEntity(relTypeIdIn: i64, entityId1In: i64, entityId2In: i64);
    fn deleteRelationToRemoteEntity(relTypeIdIn: i64, entityId1In: i64, remoteInstanceIdIn: String, entityId2In: i64);
    fn deleteRelationToGroup(entityIdIn: i64, relTypeIdIn: i64, groupIdIn: i64);
    fn deleteGroupAndRelationsToIt(idIn: i64);
    fn deleteRelationType(idIn: i64);
    fn deleteClassAndItsTemplateEntity(classIdIn: i64);
    fn deleteGroupRelationsToItAndItsEntries(groupidIn: i64);
    fn deleteOmInstance(idIn: String) -> Unit;
    fn removeEntityFromGroup(groupIdIn: i64, containedEntityIdIn: i64, callerManagesTransactionsIn: Boolean = false);


    // (See comments above the set of these methods, in RestDatabase.scala:)
    pub fn getUserPreference_Boolean(preferenceNameIn: String, defaultValueIn: Option[Boolean] = None) -> Option[Boolean];
    pub fn getPreferencesContainerId() -> i64;
    pub fn getUserPreference_EntityId(preferenceNameIn: String, defaultValueIn: Option[i64] = None) -> Option[i64];
    pub fn getOmInstances(localIn: Option[Boolean] = None) -> java.util.ArrayList[OmInstance];

 */
}
