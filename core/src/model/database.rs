%%
/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2016-2017 inclusive and 2020, Luke A. Call; all rights reserved.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
package org.onemodel.core.model

import java.util
import org.onemodel.core.{OmDatabaseException, Util}
import scala.collection.mutable

object Database {
  let dbNamePrefix = "om_";
  // If next line ever changes, search the code for other places that also have it hard-coded, to change also
  // (ex., INSTALLING, first.exp or its successors, any .psql scripts, ....  "t1/x" is shorter to type
  // during manual testing than "testrunner/testrunner".
  let TEST_USER: String = "t1";
  let TEST_PASS: String = "x";
  let MIXED_CLASSES_EXCEPTION = "All the entities in a group should be of the same class.";
  // so named to make it unlikely to collide by name with anything else:
  let systemEntityName = ".system-use-only";
  // aka template entities:
  let classTemplateEntityGroupName = "class-defining entities";
  let theHASrelationTypeName = "has";
  let theIsHadByReverseName = "is had by";
  let EDITOR_INFO_ENTITY_NAME = "editorInfo";
  let TEXT_EDITOR_INFO_ENTITY_NAME = "textEditorInfo";
  let TEXT_EDITOR_COMMAND_ATTRIBUTE_TYPE_NAME = "textEditorCommand";
  let PREF_TYPE_BOOLEAN = "boolean";
  let PREF_TYPE_ENTITY_ID = "entityId";
  let TEMPLATE_NAME_SUFFIX: String = "-template";
  let UNUSED_GROUP_ERR1 = "No available index found which is not already used. How would so many be used?";
  let UNUSED_GROUP_ERR2 = "Very unexpected, but could it be that you are running out of available sorting indexes!?" +;
                          " Have someone check, before you need to create, for example, a thousand more entities."
  let getClassData_resultTypes = "String,i64,Boolean";
  let getRelationTypeData_resultTypes = "String,String,String";
  let getOmInstanceData_resultTypes = "Boolean,String,i64,i64";
  let getQuantityAttributeData_resultTypes = "i64,i64,Float,i64,i64,i64,i64";
  let getDateAttributeData_resultTypes = "i64,i64,i64,i64";
  let getBooleanAttributeData_resultTypes = "i64,Boolean,i64,i64,i64,i64";
  let getFileAttributeData_resultTypes = "i64,String,i64,i64,i64,String,Boolean,Boolean,Boolean,i64,String,i64";
  let getTextAttributeData_resultTypes = "i64,String,i64,i64,i64,i64";
  let getRelationToGroupDataById_resultTypes = "i64,i64,i64,i64,i64,i64,i64";
  let getRelationToGroupDataByKeys_resultTypes = "i64,i64,i64,i64,i64,i64,i64";
  let getRelationToLocalEntity_resultTypes = "i64,i64,i64,i64";
  let getRelationToRemoteEntity_resultTypes = "i64,i64,i64,i64";
  let getGroupData_resultTypes = "String,i64,Boolean,Boolean";
  let getEntityData_resultTypes = "String,i64,i64,Boolean,Boolean,Boolean";
  let getGroupEntriesData_resultTypes = "i64,i64";

  // where we create the table also calls this.
  // Longer than the old 60 (needed), and a likely familiar length to many people (for ease in knowing when done), seems a decent balance. If any longer
  // is needed, maybe it should be put in a TextAttribute and make those more convenient to use, instead.
  def entityNameLength: Int = 160

  // in postgres, one table "extends" the other (see comments in createTables)
  def relationTypeNameLength: Int = entityNameLength

  def classNameLength: Int = entityNameLength

  // (See usages. The DNS hostname max size seems to be 255 plus 1 null, but the ":<port>" part could add 6 more chars (they seem to go up to :65535).
  // Maybe someday we will have to move to a larger size in case it changes or uses unicode or I don't know what.)
  def omInstanceAddressLength: Int = 262

  def getAttributeFormId(key: String): Int = {
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
  def getAttributeFormName(key: Int): String = {
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

  def maxIdValue: i64 = {
    // Max size for a Java long type, and for a postgresql 7.2.1 bigint type (which is being used, at the moment, for the id value in Entity table.
    // (these values are from file:///usr/share/doc/postgresql-doc-9.1/html/datatype-numeric.html)
    9223372036854775807L
  }

  def minIdValue: i64 = {
    -9223372036854775808L
  }

  def login(username: String, password: String, showError: Boolean): Option[Database] = {
    try Some(new PostgreSQLDatabase(username, new String(password)))
    catch {
      case ex: org.postgresql.util.PSQLException =>
        // attempt didn't work, but don't throw exc if the program
        // is just trying defaults, for example:
        if (showError) throw ex
        else None
    }
  }

  def getRestDatabase(remoteAddressIn: String): RestDatabase = {
    new RestDatabase(remoteAddressIn)
  }

  def currentOrRemoteDb(relationToEntityIn: Attribute, currentDb: Database): Database = {
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
}

abstract class Database {
  def isRemote: Boolean
  def getRemoteAddress: Option[String] = None
  let id: String;
  def includeArchivedEntities: Boolean
  def beginTrans()
  def rollbackTrans()
  def commitTrans()

  /* Many of these methods are marked "protected[model]" for 2 reasons:
       1) to minimize the risk of calling db.<method> on the wrong db, when the full model object (like Entity) would contain the right db for itself
          (ie, what if one called db.delete and the same entity id # exists in both databases), and
       2) to generally manage the coupling between the Controller and model package, since it seems cleaner to go through model objects when then can
          call the db for themselves, rather than everything touching the db entrails directly.
     ...but should be avoided when going through the model object (like Entity) causes enough more db hits to not be worth it (performance vs.
     clarity & ease of maintenance).
  * */
  protected[model] def createQuantityAttribute(parentIdIn: i64, attrTypeIdIn: i64, unitIdIn: i64, numberIn: Float, validOnDateIn: Option[i64],
                              inObservationDate: i64, callerManagesTransactionsIn: Boolean = false, sortingIndexIn: Option[i64] = None): /*id*/ i64
  protected[model] def createDateAttribute(parentIdIn: i64, attrTypeIdIn: i64, dateIn: i64, sortingIndexIn: Option[i64] = None): /*id*/ i64
  protected[model] def createBooleanAttribute(parentIdIn: i64, attrTypeIdIn: i64, booleanIn: Boolean, validOnDateIn: Option[i64], observationDateIn: i64,
                             sortingIndexIn: Option[i64] = None): /*id*/ i64
  protected[model] def createFileAttribute(parentIdIn: i64, attrTypeIdIn: i64, descriptionIn: String, originalFileDateIn: i64, storedDateIn: i64,
                          originalFilePathIn: String, readableIn: Boolean, writableIn: Boolean, executableIn: Boolean, sizeIn: i64,
                          md5hashIn: String, inputStreamIn: java.io.FileInputStream, sortingIndexIn: Option[i64] = None): /*id*/ i64
  protected[model] def createTextAttribute(parentIdIn: i64, attrTypeIdIn: i64, textIn: String, validOnDateIn: Option[i64] = None,
                          observationDateIn: i64 = System.currentTimeMillis(), callerManagesTransactionsIn: Boolean = false,
                          sortingIndexIn: Option[i64] = None): /*id*/ i64
  protected[model] def createRelationToLocalEntity(relationTypeIdIn: i64, entityId1In: i64, entityId2In: i64, validOnDateIn: Option[i64], observationDateIn: i64,
                             sortingIndexIn: Option[i64] = None, callerManagesTransactionsIn: Boolean = false): RelationToLocalEntity
  protected[model] def createRelationToRemoteEntity(relationTypeIdIn: i64, entityId1In: i64, entityId2In: i64, validOnDateIn: Option[i64], observationDateIn: i64,
                                   remoteInstanceIdIn: String, sortingIndexIn: Option[i64] = None,
                                   callerManagesTransactionsIn: Boolean = false): RelationToRemoteEntity
  protected[model] def createGroupAndRelationToGroup(entityIdIn: i64, relationTypeIdIn: i64, newGroupNameIn: String, allowMixedClassesInGroupIn: Boolean = false,
                                    validOnDateIn: Option[i64], observationDateIn: i64,
                                    sortingIndexIn: Option[i64], callerManagesTransactionsIn: Boolean = false): (i64, i64)
  def createEntity(nameIn: String, classIdIn: Option[i64] = None, isPublicIn: Option[Boolean] = None): /*id*/ i64
  protected[model] def createEntityAndRelationToLocalEntity(entityIdIn: i64, relationTypeIdIn: i64, newEntityNameIn: String, isPublicIn: Option[Boolean],
                                      validOnDateIn: Option[i64], observationDateIn: i64, callerManagesTransactionsIn: Boolean = false): (i64, i64)
  protected[model] def createRelationToGroup(entityIdIn: i64, relationTypeIdIn: i64, groupIdIn: i64, validOnDateIn: Option[i64], observationDateIn: i64,
                            sortingIndexIn: Option[i64] = None, callerManagesTransactionsIn: Boolean = false): (i64, i64)
  protected[model] def addEntityToGroup(groupIdIn: i64, containedEntityIdIn: i64, sortingIndexIn: Option[i64] = None, callerManagesTransactionsIn: Boolean = false)
  def createOmInstance(idIn: String, isLocalIn: Boolean, addressIn: String, entityIdIn: Option[i64] = None, oldTableName: Boolean = false): i64
  protected[model] def addHASRelationToLocalEntity(fromEntityIdIn: i64, toEntityIdIn: i64, validOnDateIn: Option[i64], observationDateIn: i64,
                             sortingIndexIn: Option[i64] = None): RelationToLocalEntity
  def getOrCreateClassAndTemplateEntity(classNameIn: String, callerManagesTransactionsIn: Boolean): (i64, i64)
  def createRelationType(nameIn: String, nameInReverseDirectionIn: String, directionalityIn: String): /*id*/ i64
  def createClassAndItsTemplateEntity(classNameIn: String): (i64, i64)
  protected[model] def addUriEntityWithUriAttribute(containingEntityIn: Entity, newEntityNameIn: String, uriIn: String, observationDateIn: i64,
                                   makeThemPublicIn: Option[Boolean], callerManagesTransactionsIn: Boolean,
                                   quoteIn: Option[String] = None): (Entity, RelationToLocalEntity)


  def attributeKeyExists(formIdIn: i64, idIn: i64): Boolean
  protected[model] def findContainedLocalEntityIds(resultsInOut: mutable.TreeSet[i64], fromEntityIdIn: i64, searchStringIn: String,
                             levelsRemaining: Int = 20, stopAfterAnyFound: Boolean = true): mutable.TreeSet[i64]
  def entityKeyExists(idIn: i64, includeArchived: Boolean = true): Boolean
  protected[model] def relationTypeKeyExists(idIn: i64): Boolean
  protected[model] def quantityAttributeKeyExists(idIn: i64): Boolean
  protected[model] def dateAttributeKeyExists(idIn: i64): Boolean
  protected[model] def booleanAttributeKeyExists(idIn: i64): Boolean
  protected[model] def fileAttributeKeyExists(idIn: i64): Boolean
  protected[model] def textAttributeKeyExists(idIn: i64): Boolean
  def relationToLocalEntityKeyExists(idIn: i64): Boolean
  def groupKeyExists(idIn: i64): Boolean
  protected[model] def relationToGroupKeysExistAndMatch(id: i64, entityId: i64, relTypeId: i64, groupId: i64): Boolean
  protected[model] def classKeyExists(idIn: i64): Boolean
  protected[model] def omInstanceKeyExists(idIn: String): Boolean
  protected[model] def getEntityData(idIn: i64): Array[Option[Any]]
  protected[model] def getEntityName(idIn: i64): Option[String]
  protected[model] def isDuplicateEntityName(nameIn: String, selfIdToIgnoreIn: Option[i64] = None): Boolean
  protected[model] def getSortedAttributes(entityIdIn: i64, startingObjectIndexIn: Int = 0, maxValsIn: Int = 0,
                          onlyPublicEntitiesIn: Boolean = true): (Array[(i64, Attribute)], Int)
  def findRelationType(typeNameIn: String, expectedRows: Option[Int] = Some(1)): java.util.ArrayList[i64]
  protected[model] def getRelationTypeData(idIn: i64): Array[Option[Any]]
  protected[model] def getQuantityAttributeData(idIn: i64): Array[Option[Any]]
  protected[model] def getDateAttributeData(idIn: i64): Array[Option[Any]]
  protected[model] def getBooleanAttributeData(idIn: i64): Array[Option[Any]]
  protected[model] def getFileAttributeData(idIn: i64): Array[Option[Any]]
  protected[model] def getFileAttributeContent(fileAttributeIdIn: i64, outputStreamIn: java.io.OutputStream): (i64, String)
  protected[model] def getTextAttributeData(idIn: i64): Array[Option[Any]]
  protected[model] def relationToLocalEntityKeysExistAndMatch(idIn: i64, relTypeIdIn: i64, entityId1In: i64, entityId2In: i64): Boolean
  protected[model] def relationToRemoteEntityKeyExists(idIn: i64): Boolean
  protected[model] def relationToRemoteEntityKeysExistAndMatch(idIn: i64, relTypeIdIn: i64, entityId1In: i64, remoteInstanceIdIn: String, entityId2In: i64): Boolean
  protected[model] def getRelationToLocalEntityData(relTypeIdIn: i64, entityId1In: i64, entityId2In: i64): Array[Option[Any]]
  protected[model] def getRelationToLocalEntityDataById(idIn: i64): Array[Option[Any]]
  protected[model] def getRelationToRemoteEntityData(relTypeIdIn: i64, entityId1In: i64, remoteInstanceIdIn: String, entityId2In: i64): Array[Option[Any]]
  protected[model] def getGroupData(idIn: i64): Array[Option[Any]]
  protected[model] def getGroupEntryObjects(groupIdIn: i64, startingObjectIndexIn: i64, maxValsIn: Option[i64] = None): java.util.ArrayList[Entity]
  def getGroupSize(groupIdIn: i64, includeWhichEntitiesIn: Int = 3): i64
  protected[model] def getHighestSortingIndexForGroup(groupIdIn: i64): i64
  protected[model] def getRelationToGroupDataByKeys(entityId: i64, relTypeId: i64, groupId: i64): Array[Option[Any]]
  protected[model] def getRelationToGroupData(idIn: i64): Array[Option[Any]]
  def getGroupEntriesData(groupIdIn: i64, limitIn: Option[i64] = None, includeArchivedEntitiesIn: Boolean = true): List[Array[Option[Any]]]
  protected[model] def findRelationToAndGroup_OnEntity(entityIdIn: i64,
                                                       groupNameIn: Option[String] = None): (Option[i64], Option[i64], Option[i64], Option[String], Boolean)
  def getEntitiesContainingGroup(groupIdIn: i64, startingIndexIn: i64, maxValsIn: Option[i64] = None): java.util.ArrayList[(i64, Entity)]
  protected[model] def getCountOfEntitiesContainingGroup(groupIdIn: i64): (i64, i64)
  protected[model] def getClassData(idIn: i64): Array[Option[Any]]
  protected[model] def getAttributeCount(entityIdIn: i64, includeArchivedEntitiesIn: Boolean = false): i64
  protected[model] def getRelationToLocalEntityCount(entityIdIn: i64, includeArchivedEntities: Boolean = false): i64
  protected[model] def getRelationToRemoteEntityCount(entityIdIn: i64): i64
  protected[model] def getRelationToGroupCount(entityIdIn: i64): i64
  def getClassCount(entityIdIn: Option[i64] = None): i64
  protected[model] def getClassName(idIn: i64): Option[String]
  protected[model] def getOmInstanceData(idIn: String): Array[Option[Any]]
  protected[model] def isDuplicateOmInstanceAddress(addressIn: String, selfIdToIgnoreIn: Option[String] = None): Boolean
  protected[model] def  getGroupsContainingEntitysGroupsIds(groupIdIn: i64, limitIn: Option[i64] = Some(5)): List[Array[Option[Any]]]
  protected[model] def isEntityInGroup(groupIdIn: i64, entityIdIn: i64): Boolean
  protected[model] def getAdjacentGroupEntriesSortingIndexes(groupIdIn: i64, sortingIndexIn: i64, limitIn: Option[i64] = None,
                                            forwardNotBackIn: Boolean): List[Array[Option[Any]]]
  protected[model] def getNearestGroupEntrysSortingIndex(groupIdIn: i64, startingPointSortingIndexIn: i64, forwardNotBackIn: Boolean): Option[i64]
  protected[model] def getAdjacentAttributesSortingIndexes(entityIdIn: i64, sortingIndexIn: i64, limitIn: Option[i64], forwardNotBackIn: Boolean): List[Array[Option[Any]]]
  protected[model] def getNearestAttributeEntrysSortingIndex(entityIdIn: i64, startingPointSortingIndexIn: i64, forwardNotBackIn: Boolean): Option[i64]
  protected[model] def getEntityAttributeSortingIndex(entityIdIn: i64, attributeFormIdIn: i64, attributeIdIn: i64): i64
  protected[model] def getGroupEntrySortingIndex(groupIdIn: i64, entityIdIn: i64): i64
  protected[model] def isGroupEntrySortingIndexInUse(groupIdIn: i64, sortingIndexIn: i64): Boolean
  protected[model] def isAttributeSortingIndexInUse(entityIdIn: i64, sortingIndexIn: i64): Boolean
  protected[model] def findUnusedAttributeSortingIndex(entityIdIn: i64, startingWithIn: Option[i64] = None): i64
  def findAllEntityIdsByName(nameIn: String, caseSensitive: Boolean = false): java.util.ArrayList[i64]
  protected[model] def findUnusedGroupSortingIndex(groupIdIn: i64, startingWithIn: Option[i64] = None): i64
  protected[model] def getTextAttributeByTypeId(parentEntityIdIn: i64, typeIdIn: i64, expectedRows: Option[Int] = None): java.util.ArrayList[TextAttribute]
  protected[model] def getLocalEntitiesContainingLocalEntity(entityIdIn: i64, startingIndexIn: i64, maxValsIn: Option[i64] = None): java.util.ArrayList[(i64, Entity)]
  protected[model] def getCountOfGroupsContainingEntity(entityIdIn: i64): i64
  protected[model] def getContainingGroupsIds(entityIdIn: i64): java.util.ArrayList[i64]
  protected[model] def getContainingRelationsToGroup(entityIdIn: i64, startingIndexIn: i64,
                                                     maxValsIn: Option[i64] = None): java.util.ArrayList[RelationToGroup]
//  protected[model] def getShouldCreateDefaultAttributes(classIdIn: i64): Option[Boolean]
  protected[model] def updateClassCreateDefaultAttributes(classIdIn: i64, value: Option[Boolean])
  def getEntitiesOnlyCount(limitByClass: Boolean = false, classIdIn: Option[i64] = None, templateEntity: Option[i64] = None): i64
  protected[model] def getCountOfLocalEntitiesContainingLocalEntity(entityIdIn: i64): (i64, i64)
  //idea (tracked): make "*duplicate*" methods just be ... called "search"? combine w/ search, or rename? makes sense for callers?
  def isDuplicateClassName(nameIn: String, selfIdToIgnoreIn: Option[i64] = None): Boolean
  protected[model] def getContainingRelationToGroupDescriptions(entityIdIn: i64, limitIn: Option[i64] = None): util.ArrayList[String]
  def getMatchingEntities(startingObjectIndexIn: i64, maxValsIn: Option[i64] = None, omitEntityIdIn: Option[i64],
                          nameRegexIn: String): java.util.ArrayList[Entity]
  def getMatchingGroups(startingObjectIndexIn: i64, maxValsIn: Option[i64] = None, omitGroupIdIn: Option[i64],
                        nameRegexIn: String): java.util.ArrayList[Group]
  protected[model] def getRelationsToGroupContainingThisGroup(groupIdIn: i64, startingIndexIn: i64,
                                                              maxValsIn: Option[i64] = None): java.util.ArrayList[RelationToGroup]
  def getEntities(startingObjectIndexIn: i64, maxValsIn: Option[i64] = None): java.util.ArrayList[Entity]
  def getEntitiesOnly(startingObjectIndexIn: i64, maxValsIn: Option[i64] = None, classIdIn: Option[i64] = None,
                      limitByClass: Boolean = false, templateEntity: Option[i64] = None,
                      groupToOmitIdIn: Option[i64] = None): java.util.ArrayList[Entity]
  def getCountOfEntitiesUsedAsAttributeTypes(objectTypeIn: String, quantitySeeksUnitNotTypeIn: Boolean): i64
  def getEntitiesUsedAsAttributeTypes(objectTypeIn: String, startingObjectIndexIn: i64, maxValsIn: Option[i64] = None,
                                      quantitySeeksUnitNotTypeIn: Boolean): java.util.ArrayList[Entity]
  def getRelationTypes(startingObjectIndexIn: i64, maxValsIn: Option[i64] = None): java.util.ArrayList[Entity]
  def getClasses(startingObjectIndexIn: i64, maxValsIn: Option[i64] = None): java.util.ArrayList[EntityClass]
  def getRelationTypeCount: i64
  def getOmInstanceCount: i64
  def getEntityCount: i64
  def findJournalEntries(startTimeIn: i64, endTimeIn: i64, limitIn: Option[i64] = None): util.ArrayList[(i64, String, i64)]
  def getGroupCount: i64
  def getGroups(startingObjectIndexIn: i64, maxValsIn: Option[i64] = None, groupToOmitIdIn: Option[i64] = None): java.util.ArrayList[Group]
  def createGroup(nameIn: String, allowMixedClassesInGroupIn: Boolean = false): i64
  def relationToGroupKeyExists(idIn: i64): Boolean


  protected[model] def updateEntitysClass(entityId: i64, classId: Option[i64], callerManagesTransactions: Boolean = false)
  protected[model] def updateEntityOnlyNewEntriesStickToTop(idIn: i64, newEntriesStickToTop: Boolean)
  protected[model] def archiveEntity(idIn: i64, callerManagesTransactionsIn: Boolean = false)
  protected[model] def unarchiveEntity(idIn: i64, callerManagesTransactionsIn: Boolean = false)
  def setIncludeArchivedEntities(in: Boolean): Unit
  def setUserPreference_EntityId(nameIn: String, entityIdIn: i64)
  protected[model] def updateEntityOnlyPublicStatus(idIn: i64, value: Option[Boolean])
  protected[model] def updateQuantityAttribute(idIn: i64, parentIdIn: i64, attrTypeIdIn: i64, unitIdIn: i64, numberIn: Float, validOnDateIn: Option[i64],
                              inObservationDate: i64)
  protected[model] def updateDateAttribute(idIn: i64, parentIdIn: i64, dateIn: i64, attrTypeIdIn: i64)
  protected[model] def updateBooleanAttribute(idIn: i64, parentIdIn: i64, attrTypeIdIn: i64, booleanIn: Boolean, validOnDateIn: Option[i64],
                             inObservationDate: i64)
  protected[model] def updateFileAttribute(idIn: i64, parentIdIn: i64, attrTypeIdIn: i64, descriptionIn: String)
  protected[model] def updateFileAttribute(idIn: i64, parentIdIn: i64, attrTypeIdIn: i64, descriptionIn: String,
                                           originalFileDateIn: i64, storedDateIn: i64,
                          originalFilePathIn: String, readableIn: Boolean, writableIn: Boolean, executableIn: Boolean, sizeIn: i64, md5hashIn: String)
  protected[model] def updateTextAttribute(idIn: i64, parentIdIn: i64, attrTypeIdIn: i64, textIn: String, validOnDateIn: Option[i64],
                                           observationDateIn: i64)
  protected[model] def updateRelationToLocalEntity(oldRelationTypeIdIn: i64, entityId1In: i64, entityId2In: i64,
                             newRelationTypeIdIn: i64, validOnDateIn: Option[i64], observationDateIn: i64)
  protected[model] def updateRelationToRemoteEntity(oldRelationTypeIdIn: i64, entityId1In: i64, remoteInstanceIdIn: String, entityId2In: i64,
                                   newRelationTypeIdIn: i64, validOnDateIn: Option[i64], observationDateIn: i64)
  protected[model] def updateGroup(groupIdIn: i64, nameIn: String, allowMixedClassesInGroupIn: Boolean = false, newEntriesStickToTopIn: Boolean = false)
  protected[model] def updateRelationToGroup(entityIdIn: i64, oldRelationTypeIdIn: i64, newRelationTypeIdIn: i64, oldGroupIdIn: i64, newGroupIdIn: i64,
                            validOnDateIn: Option[i64], observationDateIn: i64)
  protected[model] def moveRelationToLocalEntityToLocalEntity(rtleIdIn: i64, newContainingEntityIdIn: i64,
                                                              sortingIndexIn: i64): RelationToLocalEntity
  protected[model] def moveRelationToRemoteEntityToLocalEntity(remoteInstanceIdIn: String, relationToRemoteEntityIdIn: i64, toContainingEntityIdIn: i64,
                                                               sortingIndexIn: i64): RelationToRemoteEntity
  protected[model] def moveLocalEntityFromLocalEntityToGroup(removingRtleIn: RelationToLocalEntity, targetGroupIdIn: i64, sortingIndexIn: i64)
  protected[model] def moveRelationToGroup(relationToGroupIdIn: i64, newContainingEntityIdIn: i64, sortingIndexIn: i64): i64
  protected[model] def moveEntityFromGroupToLocalEntity(fromGroupIdIn: i64, toEntityIdIn: i64, moveEntityIdIn: i64, sortingIndexIn: i64)
  protected[model] def moveLocalEntityFromGroupToGroup(fromGroupIdIn: i64, toGroupIdIn: i64, moveEntityIdIn: i64, sortingIndexIn: i64)
  protected[model] def renumberSortingIndexes(entityIdOrGroupIdIn: i64, callerManagesTransactionsIn: Boolean = false,
                                              isEntityAttrsNotGroupEntries: Boolean = true)
  protected[model] def updateAttributeSortingIndex(entityIdIn: i64, attributeFormIdIn: i64, attributeIdIn: i64, sortingIndexIn: i64)
  protected[model] def updateSortingIndexInAGroup(groupIdIn: i64, entityIdIn: i64, sortingIndexIn: i64)
  protected[model] def updateEntityOnlyName(idIn: i64, nameIn: String)
  protected[model] def updateRelationType(idIn: i64, nameIn: String, nameInReverseDirectionIn: String, directionalityIn: String)
  protected[model] def updateClassAndTemplateEntityName(classIdIn: i64, name: String): i64
  protected[model] def updateOmInstance(idIn: String, addressIn: String, entityIdIn: Option[i64])

  protected[model] def deleteEntity(idIn: i64, callerManagesTransactionsIn: Boolean = false)
  protected[model] def deleteQuantityAttribute(idIn: i64)
  protected[model] def deleteDateAttribute(idIn: i64)
  protected[model] def deleteBooleanAttribute(idIn: i64)
  protected[model] def deleteFileAttribute(idIn: i64)
  protected[model] def deleteTextAttribute(idIn: i64)
  protected[model] def deleteRelationToLocalEntity(relTypeIdIn: i64, entityId1In: i64, entityId2In: i64)
  protected[model] def deleteRelationToRemoteEntity(relTypeIdIn: i64, entityId1In: i64, remoteInstanceIdIn: String, entityId2In: i64)
  protected[model] def deleteRelationToGroup(entityIdIn: i64, relTypeIdIn: i64, groupIdIn: i64)
  protected[model] def deleteGroupAndRelationsToIt(idIn: i64)
  protected[model] def deleteRelationType(idIn: i64)
  protected[model] def deleteClassAndItsTemplateEntity(classIdIn: i64)
  protected[model] def deleteGroupRelationsToItAndItsEntries(groupidIn: i64)
  protected[model] def deleteOmInstance(idIn: String): Unit
  protected[model] def removeEntityFromGroup(groupIdIn: i64, containedEntityIdIn: i64, callerManagesTransactionsIn: Boolean = false)


  // (See comments above the set of these methods, in RestDatabase.scala:)
  def getUserPreference_Boolean(preferenceNameIn: String, defaultValueIn: Option[Boolean] = None): Option[Boolean]
  def getPreferencesContainerId: i64
  def getUserPreference_EntityId(preferenceNameIn: String, defaultValueIn: Option[i64] = None): Option[i64]
  def getOmInstances(localIn: Option[Boolean] = None): java.util.ArrayList[OmInstance]
}
