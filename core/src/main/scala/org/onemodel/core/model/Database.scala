/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2016-2017 inclusive, Luke A. Call; all rights reserved.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule, guidelines around binary
    distribution, and the GNU Affero General Public License as published by the Free Software Foundation;
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
  val dbNamePrefix = "om_"
  // If next line ever changes, search the code for other places that also have it hard-coded, to change also
  // (e.g., INSTALLING, first.exp or its successors, any .psql scripts, ....
  val TEST_USER: String = "testrunner"
  val MIXED_CLASSES_EXCEPTION = "All the entities in a group should be of the same class."
  // so named to make it unlikely to collide by name with anything else:
  val systemEntityName = ".system-use-only"
  // aka template entities:
  val classTemplateEntityGroupName = "class-defining entities"
  val theHASrelationTypeName = "has"
  val theIsHadByReverseName = "is had by"
  val EDITOR_INFO_ENTITY_NAME = "editorInfo"
  val TEXT_EDITOR_INFO_ENTITY_NAME = "textEditorInfo"
  val TEXT_EDITOR_COMMAND_ATTRIBUTE_TYPE_NAME = "textEditorCommand"
  val PREF_TYPE_BOOLEAN = "boolean"
  val PREF_TYPE_ENTITY_ID = "entityId"
  val TEMPLATE_NAME_SUFFIX: String = "-template"
  val UNUSED_GROUP_ERR1 = "No available index found which is not already used. How would so many be used?"
  val UNUSED_GROUP_ERR2 = "Very unexpected, but could it be that you are running out of available sorting indexes!?" +
                          " Have someone check, before you need to create, for example, a thousand more entities."
  val getClassData_resultTypes = "String,Long,Boolean"
  val getRelationTypeData_resultTypes = "String,String,String"
  val getOmInstanceData_resultTypes = "Boolean,String,Long,Long"
  val getQuantityAttributeData_resultTypes = "Long,Long,Float,Long,Long,Long,Long"
  val getDateAttributeData_resultTypes = "Long,Long,Long,Long"
  val getBooleanAttributeData_resultTypes = "Long,Boolean,Long,Long,Long,Long"
  val getFileAttributeData_resultTypes = "Long,String,Long,Long,Long,String,Boolean,Boolean,Boolean,Long,String,Long"
  val getTextAttributeData_resultTypes = "Long,String,Long,Long,Long,Long"
  val getRelationToGroupDataById_resultTypes = "Long,Long,Long,Long,Long,Long,Long"
  val getRelationToGroupDataByKeys_resultTypes = "Long,Long,Long,Long,Long,Long,Long"
  val getRelationToLocalEntity_resultTypes = "Long,Long,Long,Long"
  val getRelationToRemoteEntity_resultTypes = "Long,Long,Long,Long"
  val getGroupData_resultTypes = "String,Long,Boolean,Boolean"
  val getEntityData_resultTypes = "String,Long,Long,Boolean,Boolean,Boolean"
  val getGroupEntriesData_resultTypes = "Long,Long"

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

  def maxIdValue: Long = {
    // Max size for a Java long type, and for a postgresql 7.2.1 bigint type (which is being used, at the moment, for the id value in Entity table.
    // (these values are from file:///usr/share/doc/postgresql-doc-9.1/html/datatype-numeric.html)
    9223372036854775807L
  }

  def minIdValue: Long = {
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
  val id: String
  def includeArchivedEntities: Boolean
  def beginTrans()
  def rollbackTrans()
  def commitTrans()

  /* Many of these methods are marked "protected[model]" for 2 reasons:
       1) to minimize the risk of calling db.<method> on the wrong db, when the full model object (like Entity) would contain the right db for itself
          (ie, what if one called db.delete and the same entity id # exists in both databases), and
       2) to generally manage the coupling between the controller and model package, since it seems cleaner to go through model objects when then can
          call the db for themselves, rather than everything touching the db entrails directly.
     ...but should be avoided when going through the model object (like Entity) causes enough more db hits to not be worth it (performance vs.
     clarity & ease of maintenance).
  * */
  protected[model] def createQuantityAttribute(parentIdIn: Long, attrTypeIdIn: Long, unitIdIn: Long, numberIn: Float, validOnDateIn: Option[Long],
                              inObservationDate: Long, callerManagesTransactionsIn: Boolean = false, sortingIndexIn: Option[Long] = None): /*id*/ Long
  protected[model] def createDateAttribute(parentIdIn: Long, attrTypeIdIn: Long, dateIn: Long, sortingIndexIn: Option[Long] = None): /*id*/ Long
  protected[model] def createBooleanAttribute(parentIdIn: Long, attrTypeIdIn: Long, booleanIn: Boolean, validOnDateIn: Option[Long], observationDateIn: Long,
                             sortingIndexIn: Option[Long] = None): /*id*/ Long
  protected[model] def createFileAttribute(parentIdIn: Long, attrTypeIdIn: Long, descriptionIn: String, originalFileDateIn: Long, storedDateIn: Long,
                          originalFilePathIn: String, readableIn: Boolean, writableIn: Boolean, executableIn: Boolean, sizeIn: Long,
                          md5hashIn: String, inputStreamIn: java.io.FileInputStream, sortingIndexIn: Option[Long] = None): /*id*/ Long
  protected[model] def createTextAttribute(parentIdIn: Long, attrTypeIdIn: Long, textIn: String, validOnDateIn: Option[Long] = None,
                          observationDateIn: Long = System.currentTimeMillis(), callerManagesTransactionsIn: Boolean = false,
                          sortingIndexIn: Option[Long] = None): /*id*/ Long
  protected[model] def createRelationToLocalEntity(relationTypeIdIn: Long, entityId1In: Long, entityId2In: Long, validOnDateIn: Option[Long], observationDateIn: Long,
                             sortingIndexIn: Option[Long] = None, callerManagesTransactionsIn: Boolean = false): RelationToLocalEntity
  protected[model] def createRelationToRemoteEntity(relationTypeIdIn: Long, entityId1In: Long, entityId2In: Long, validOnDateIn: Option[Long], observationDateIn: Long,
                                   remoteInstanceIdIn: String, sortingIndexIn: Option[Long] = None,
                                   callerManagesTransactionsIn: Boolean = false): RelationToRemoteEntity
  protected[model] def createGroupAndRelationToGroup(entityIdIn: Long, relationTypeIdIn: Long, newGroupNameIn: String, allowMixedClassesInGroupIn: Boolean = false,
                                    validOnDateIn: Option[Long], observationDateIn: Long,
                                    sortingIndexIn: Option[Long], callerManagesTransactionsIn: Boolean = false): (Long, Long)
  def createEntity(nameIn: String, classIdIn: Option[Long] = None, isPublicIn: Option[Boolean] = None): /*id*/ Long
  protected[model] def createEntityAndRelationToLocalEntity(entityIdIn: Long, relationTypeIdIn: Long, newEntityNameIn: String, isPublicIn: Option[Boolean],
                                      validOnDateIn: Option[Long], observationDateIn: Long, callerManagesTransactionsIn: Boolean = false): (Long, Long)
  protected[model] def createRelationToGroup(entityIdIn: Long, relationTypeIdIn: Long, groupIdIn: Long, validOnDateIn: Option[Long], observationDateIn: Long,
                            sortingIndexIn: Option[Long] = None, callerManagesTransactionsIn: Boolean = false): (Long, Long)
  protected[model] def addEntityToGroup(groupIdIn: Long, containedEntityIdIn: Long, sortingIndexIn: Option[Long] = None, callerManagesTransactionsIn: Boolean = false)
  def createOmInstance(idIn: String, isLocalIn: Boolean, addressIn: String, entityIdIn: Option[Long] = None, oldTableName: Boolean = false): Long
  protected[model] def addHASRelationToLocalEntity(fromEntityIdIn: Long, toEntityIdIn: Long, validOnDateIn: Option[Long], observationDateIn: Long,
                             sortingIndexIn: Option[Long] = None): RelationToLocalEntity
  def getOrCreateClassAndTemplateEntity(classNameIn: String, callerManagesTransactionsIn: Boolean): (Long, Long)
  def createRelationType(nameIn: String, nameInReverseDirectionIn: String, directionalityIn: String): /*id*/ Long
  def createClassAndItsTemplateEntity(classNameIn: String): (Long, Long)
  protected[model] def addUriEntityWithUriAttribute(containingEntityIn: Entity, newEntityNameIn: String, uriIn: String, observationDateIn: Long,
                                   makeThemPublicIn: Option[Boolean], callerManagesTransactionsIn: Boolean,
                                   quoteIn: Option[String] = None): (Entity, RelationToLocalEntity)


  def attributeKeyExists(formIdIn: Long, idIn: Long): Boolean
  protected[model] def findContainedLocalEntityIds(resultsInOut: mutable.TreeSet[Long], fromEntityIdIn: Long, searchStringIn: String,
                             levelsRemaining: Int = 20, stopAfterAnyFound: Boolean = true): mutable.TreeSet[Long]
  def entityKeyExists(idIn: Long, includeArchived: Boolean = true): Boolean
  protected[model] def relationTypeKeyExists(idIn: Long): Boolean
  protected[model] def quantityAttributeKeyExists(idIn: Long): Boolean
  protected[model] def dateAttributeKeyExists(idIn: Long): Boolean
  protected[model] def booleanAttributeKeyExists(idIn: Long): Boolean
  protected[model] def fileAttributeKeyExists(idIn: Long): Boolean
  protected[model] def textAttributeKeyExists(idIn: Long): Boolean
  def relationToLocalEntityKeyExists(idIn: Long): Boolean
  def groupKeyExists(idIn: Long): Boolean
  protected[model] def relationToGroupKeysExistAndMatch(id: Long, entityId: Long, relTypeId: Long, groupId: Long): Boolean
  protected[model] def classKeyExists(idIn: Long): Boolean
  protected[model] def omInstanceKeyExists(idIn: String): Boolean
  protected[model] def getEntityData(idIn: Long): Array[Option[Any]]
  protected[model] def getEntityName(idIn: Long): Option[String]
  protected[model] def isDuplicateEntityName(nameIn: String, selfIdToIgnoreIn: Option[Long] = None): Boolean
  protected[model] def getSortedAttributes(entityIdIn: Long, startingObjectIndexIn: Int = 0, maxValsIn: Int = 0,
                          onlyPublicEntitiesIn: Boolean = true): (Array[(Long, Attribute)], Int)
  def findRelationType(typeNameIn: String, expectedRows: Option[Int] = Some(1)): java.util.ArrayList[Long]
  protected[model] def getRelationTypeData(idIn: Long): Array[Option[Any]]
  protected[model] def getQuantityAttributeData(idIn: Long): Array[Option[Any]]
  protected[model] def getDateAttributeData(idIn: Long): Array[Option[Any]]
  protected[model] def getBooleanAttributeData(idIn: Long): Array[Option[Any]]
  protected[model] def getFileAttributeData(idIn: Long): Array[Option[Any]]
  protected[model] def getFileAttributeContent(fileAttributeIdIn: Long, outputStreamIn: java.io.OutputStream): (Long, String)
  protected[model] def getTextAttributeData(idIn: Long): Array[Option[Any]]
  protected[model] def relationToLocalEntityKeysExistAndMatch(idIn: Long, relTypeIdIn: Long, entityId1In: Long, entityId2In: Long): Boolean
  protected[model] def relationToRemoteEntityKeyExists(idIn: Long): Boolean
  protected[model] def relationToRemoteEntityKeysExistAndMatch(idIn: Long, relTypeIdIn: Long, entityId1In: Long, remoteInstanceIdIn: String, entityId2In: Long): Boolean
  protected[model] def getRelationToLocalEntityData(relTypeIdIn: Long, entityId1In: Long, entityId2In: Long): Array[Option[Any]]
  protected[model] def getRelationToLocalEntityDataById(idIn: Long): Array[Option[Any]]
  protected[model] def getRelationToRemoteEntityData(relTypeIdIn: Long, entityId1In: Long, remoteInstanceIdIn: String, entityId2In: Long): Array[Option[Any]]
  protected[model] def getGroupData(idIn: Long): Array[Option[Any]]
  protected[model] def getGroupEntryObjects(groupIdIn: Long, startingObjectIndexIn: Long, maxValsIn: Option[Long] = None): java.util.ArrayList[Entity]
  def getGroupSize(groupIdIn: Long, includeWhichEntitiesIn: Int = 3): Long
  protected[model] def getHighestSortingIndexForGroup(groupIdIn: Long): Long
  protected[model] def getRelationToGroupDataByKeys(entityId: Long, relTypeId: Long, groupId: Long): Array[Option[Any]]
  protected[model] def getRelationToGroupData(idIn: Long): Array[Option[Any]]
  def getGroupEntriesData(groupIdIn: Long, limitIn: Option[Long] = None, includeArchivedEntitiesIn: Boolean = true): List[Array[Option[Any]]]
  protected[model] def findRelationToAndGroup_OnEntity(entityIdIn: Long,
                                                       groupNameIn: Option[String] = None): (Option[Long], Option[Long], Option[Long], Option[String], Boolean)
  def getEntitiesContainingGroup(groupIdIn: Long, startingIndexIn: Long, maxValsIn: Option[Long] = None): java.util.ArrayList[(Long, Entity)]
  protected[model] def getCountOfEntitiesContainingGroup(groupIdIn: Long): (Long, Long)
  protected[model] def getClassData(idIn: Long): Array[Option[Any]]
  protected[model] def getAttributeCount(entityIdIn: Long, includeArchivedEntitiesIn: Boolean = false): Long
  protected[model] def getRelationToLocalEntityCount(entityIdIn: Long, includeArchivedEntities: Boolean = false): Long
  protected[model] def getRelationToGroupCount(entityIdIn: Long): Long
  def getClassCount(entityIdIn: Option[Long] = None): Long
  protected[model] def getClassName(idIn: Long): Option[String]
  protected[model] def getOmInstanceData(idIn: String): Array[Option[Any]]
  protected[model] def isDuplicateOmInstanceAddress(addressIn: String, selfIdToIgnoreIn: Option[String] = None): Boolean
  protected[model] def  getGroupsContainingEntitysGroupsIds(groupIdIn: Long, limitIn: Option[Long] = Some(5)): List[Array[Option[Any]]]
  protected[model] def isEntityInGroup(groupIdIn: Long, entityIdIn: Long): Boolean
  protected[model] def getAdjacentGroupEntriesSortingIndexes(groupIdIn: Long, sortingIndexIn: Long, limitIn: Option[Long] = None,
                                            forwardNotBackIn: Boolean): List[Array[Option[Any]]]
  protected[model] def getNearestGroupEntrysSortingIndex(groupIdIn: Long, startingPointSortingIndexIn: Long, forwardNotBackIn: Boolean): Option[Long]
  protected[model] def getAdjacentAttributesSortingIndexes(entityIdIn: Long, sortingIndexIn: Long, limitIn: Option[Long], forwardNotBackIn: Boolean): List[Array[Option[Any]]]
  protected[model] def getNearestAttributeEntrysSortingIndex(entityIdIn: Long, startingPointSortingIndexIn: Long, forwardNotBackIn: Boolean): Option[Long]
  protected[model] def getEntityAttributeSortingIndex(entityIdIn: Long, attributeFormIdIn: Long, attributeIdIn: Long): Long
  protected[model] def getGroupEntrySortingIndex(groupIdIn: Long, entityIdIn: Long): Long
  protected[model] def isGroupEntrySortingIndexInUse(groupIdIn: Long, sortingIndexIn: Long): Boolean
  protected[model] def isAttributeSortingIndexInUse(entityIdIn: Long, sortingIndexIn: Long): Boolean
  protected[model] def findUnusedAttributeSortingIndex(entityIdIn: Long, startingWithIn: Option[Long] = None): Long
  def findAllEntityIdsByName(nameIn: String, caseSensitive: Boolean = false): java.util.ArrayList[Long]
  protected[model] def findUnusedGroupSortingIndex(groupIdIn: Long, startingWithIn: Option[Long] = None): Long
  protected[model] def getTextAttributeByTypeId(parentEntityIdIn: Long, typeIdIn: Long, expectedRows: Option[Int] = None): java.util.ArrayList[TextAttribute]
  protected[model] def getLocalEntitiesContainingLocalEntity(entityIdIn: Long, startingIndexIn: Long, maxValsIn: Option[Long] = None): java.util.ArrayList[(Long, Entity)]
  protected[model] def getCountOfGroupsContainingEntity(entityIdIn: Long): Long
  protected[model] def getContainingGroupsIds(entityIdIn: Long): java.util.ArrayList[Long]
  protected[model] def getContainingRelationsToGroup(entityIdIn: Long, startingIndexIn: Long,
                                                     maxValsIn: Option[Long] = None): java.util.ArrayList[RelationToGroup]
//  protected[model] def getShouldCreateDefaultAttributes(classIdIn: Long): Option[Boolean]
  protected[model] def updateClassCreateDefaultAttributes(classIdIn: Long, value: Option[Boolean])
  def getEntitiesOnlyCount(limitByClass: Boolean = false, classIdIn: Option[Long] = None, templateEntity: Option[Long] = None): Long
  protected[model] def getCountOfLocalEntitiesContainingLocalEntity(entityIdIn: Long): (Long, Long)
  //idea (tracked): make "*duplicate*" methods just be ... called "search"? combine w/ search, or rename? makes sense for callers?
  def isDuplicateClassName(nameIn: String, selfIdToIgnoreIn: Option[Long] = None): Boolean
  protected[model] def getContainingRelationToGroupDescriptions(entityIdIn: Long, limitIn: Option[Long] = None): util.ArrayList[String]
  def getMatchingEntities(startingObjectIndexIn: Long, maxValsIn: Option[Long] = None, omitEntityIdIn: Option[Long],
                          nameRegexIn: String): java.util.ArrayList[Entity]
  def getMatchingGroups(startingObjectIndexIn: Long, maxValsIn: Option[Long] = None, omitGroupIdIn: Option[Long],
                        nameRegexIn: String): java.util.ArrayList[Group]
  protected[model] def getRelationsToGroupContainingThisGroup(groupIdIn: Long, startingIndexIn: Long,
                                                              maxValsIn: Option[Long] = None): java.util.ArrayList[RelationToGroup]
  def getEntities(startingObjectIndexIn: Long, maxValsIn: Option[Long] = None): java.util.ArrayList[Entity]
  def getEntitiesOnly(startingObjectIndexIn: Long, maxValsIn: Option[Long] = None, classIdIn: Option[Long] = None,
                      limitByClass: Boolean = false, templateEntity: Option[Long] = None,
                      groupToOmitIdIn: Option[Long] = None): java.util.ArrayList[Entity]
  def getRelationTypes(startingObjectIndexIn: Long, maxValsIn: Option[Long] = None): java.util.ArrayList[Entity]
  def getClasses(startingObjectIndexIn: Long, maxValsIn: Option[Long] = None): java.util.ArrayList[EntityClass]
  def getRelationTypeCount: Long
  def getOmInstanceCount: Long
  def getEntityCount: Long
  def findJournalEntries(startTimeIn: Long, endTimeIn: Long, limitIn: Option[Long] = None): util.ArrayList[(Long, String, Long)]
  def getGroupCount: Long
  def getGroups(startingObjectIndexIn: Long, maxValsIn: Option[Long] = None, groupToOmitIdIn: Option[Long] = None): java.util.ArrayList[Group]
  def createGroup(nameIn: String, allowMixedClassesInGroupIn: Boolean = false): Long
  def relationToGroupKeyExists(idIn: Long): Boolean


  protected[model] def updateEntitysClass(entityId: Long, classId: Option[Long], callerManagesTransactions: Boolean = false)
  protected[model] def updateEntityOnlyNewEntriesStickToTop(idIn: Long, newEntriesStickToTop: Boolean)
  protected[model] def archiveEntity(idIn: Long, callerManagesTransactionsIn: Boolean = false)
  protected[model] def unarchiveEntity(idIn: Long, callerManagesTransactionsIn: Boolean = false)
  def setIncludeArchivedEntities(in: Boolean): Unit
  def setUserPreference_EntityId(nameIn: String, entityIdIn: Long)
  protected[model] def updateEntityOnlyPublicStatus(idIn: Long, value: Option[Boolean])
  protected[model] def updateQuantityAttribute(idIn: Long, parentIdIn: Long, attrTypeIdIn: Long, unitIdIn: Long, numberIn: Float, validOnDateIn: Option[Long],
                              inObservationDate: Long)
  protected[model] def updateDateAttribute(idIn: Long, parentIdIn: Long, dateIn: Long, attrTypeIdIn: Long)
  protected[model] def updateBooleanAttribute(idIn: Long, parentIdIn: Long, attrTypeIdIn: Long, booleanIn: Boolean, validOnDateIn: Option[Long],
                             inObservationDate: Long)
  protected[model] def updateFileAttribute(idIn: Long, parentIdIn: Long, attrTypeIdIn: Long, descriptionIn: String)
  protected[model] def updateFileAttribute(idIn: Long, parentIdIn: Long, attrTypeIdIn: Long, descriptionIn: String,
                                           originalFileDateIn: Long, storedDateIn: Long,
                          originalFilePathIn: String, readableIn: Boolean, writableIn: Boolean, executableIn: Boolean, sizeIn: Long, md5hashIn: String)
  protected[model] def updateTextAttribute(idIn: Long, parentIdIn: Long, attrTypeIdIn: Long, textIn: String, validOnDateIn: Option[Long],
                                           observationDateIn: Long)
  protected[model] def updateRelationToLocalEntity(oldRelationTypeIdIn: Long, entityId1In: Long, entityId2In: Long,
                             newRelationTypeIdIn: Long, validOnDateIn: Option[Long], observationDateIn: Long)
  protected[model] def updateRelationToRemoteEntity(oldRelationTypeIdIn: Long, entityId1In: Long, remoteInstanceIdIn: String, entityId2In: Long,
                                   newRelationTypeIdIn: Long, validOnDateIn: Option[Long], observationDateIn: Long)
  protected[model] def updateGroup(groupIdIn: Long, nameIn: String, allowMixedClassesInGroupIn: Boolean = false, newEntriesStickToTopIn: Boolean = false)
  protected[model] def updateRelationToGroup(entityIdIn: Long, oldRelationTypeIdIn: Long, newRelationTypeIdIn: Long, oldGroupIdIn: Long, newGroupIdIn: Long,
                            validOnDateIn: Option[Long], observationDateIn: Long)
  protected[model] def moveRelationToLocalEntityToLocalEntity(rtleIdIn: Long, newContainingEntityIdIn: Long,
                                                              sortingIndexIn: Long): RelationToLocalEntity
  protected[model] def moveRelationToRemoteEntityToLocalEntity(remoteInstanceIdIn: String, relationToRemoteEntityIdIn: Long, toContainingEntityIdIn: Long,
                                                               sortingIndexIn: Long): RelationToRemoteEntity
  protected[model] def moveLocalEntityFromLocalEntityToGroup(removingRtleIn: RelationToLocalEntity, targetGroupIdIn: Long, sortingIndexIn: Long)
  protected[model] def moveRelationToGroup(relationToGroupIdIn: Long, newContainingEntityIdIn: Long, sortingIndexIn: Long): Long
  protected[model] def moveEntityFromGroupToLocalEntity(fromGroupIdIn: Long, toEntityIdIn: Long, moveEntityIdIn: Long, sortingIndexIn: Long)
  protected[model] def moveLocalEntityFromGroupToGroup(fromGroupIdIn: Long, toGroupIdIn: Long, moveEntityIdIn: Long, sortingIndexIn: Long)
  protected[model] def renumberSortingIndexes(entityIdOrGroupIdIn: Long, callerManagesTransactionsIn: Boolean = false,
                                              isEntityAttrsNotGroupEntries: Boolean = true)
  protected[model] def updateAttributeSortingIndex(entityIdIn: Long, attributeFormIdIn: Long, attributeIdIn: Long, sortingIndexIn: Long)
  protected[model] def updateSortingIndexInAGroup(groupIdIn: Long, entityIdIn: Long, sortingIndexIn: Long)
  protected[model] def updateEntityOnlyName(idIn: Long, nameIn: String)
  protected[model] def updateRelationType(idIn: Long, nameIn: String, nameInReverseDirectionIn: String, directionalityIn: String)
  protected[model] def updateClassAndTemplateEntityName(classIdIn: Long, name: String): Long
  protected[model] def updateOmInstance(idIn: String, addressIn: String, entityIdIn: Option[Long])

  protected[model] def deleteEntity(idIn: Long, callerManagesTransactionsIn: Boolean = false)
  protected[model] def deleteQuantityAttribute(idIn: Long)
  protected[model] def deleteDateAttribute(idIn: Long)
  protected[model] def deleteBooleanAttribute(idIn: Long)
  protected[model] def deleteFileAttribute(idIn: Long)
  protected[model] def deleteTextAttribute(idIn: Long)
  protected[model] def deleteRelationToLocalEntity(relTypeIdIn: Long, entityId1In: Long, entityId2In: Long)
  protected[model] def deleteRelationToRemoteEntity(relTypeIdIn: Long, entityId1In: Long, remoteInstanceIdIn: String, entityId2In: Long)
  protected[model] def deleteRelationToGroup(entityIdIn: Long, relTypeIdIn: Long, groupIdIn: Long)
  protected[model] def deleteGroupAndRelationsToIt(idIn: Long)
  protected[model] def deleteRelationType(idIn: Long)
  protected[model] def deleteClassAndItsTemplateEntity(classIdIn: Long)
  protected[model] def deleteGroupRelationsToItAndItsEntries(groupidIn: Long)
  protected[model] def deleteOmInstance(idIn: String): Unit
  protected[model] def removeEntityFromGroup(groupIdIn: Long, containedEntityIdIn: Long, callerManagesTransactionsIn: Boolean = false)


  // (See comments above the set of these methods, in RestDatabase.scala:)
  def getUserPreference_Boolean(preferenceNameIn: String, defaultValueIn: Option[Boolean] = None): Option[Boolean]
  def getPreferencesContainerId: Long
  def getUserPreference_EntityId(preferenceNameIn: String, defaultValueIn: Option[Long] = None): Option[Long]
  def getOmInstances(localIn: Option[Boolean] = None): java.util.ArrayList[OmInstance]
}
