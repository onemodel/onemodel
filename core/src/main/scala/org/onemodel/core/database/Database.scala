/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2016-2016 inclusive, Luke A. Call; all rights reserved.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule, guidelines around binary
    distribution, and the GNU Affero General Public License as published by the Free Software Foundation, either version 3
    of the License, or (at your option) any later version.  See the file LICENSE for details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
package org.onemodel.core.database

import org.onemodel.core.model._
import org.onemodel.core.{OmDatabaseException, Util}

import scala.collection.mutable

object Database {
  val dbNamePrefix = "om_"
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
  val getRelationToEntity_resultTypes = "Long,Long,Long,Long"
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
      case Util.RELATION_TO_ENTITY_TYPE => 6
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
      case 6 => Util.RELATION_TO_ENTITY_TYPE
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

}
abstract class Database {
  def isRemote: Boolean
  def beginTrans()
  def rollbackTrans()
  def commitTrans()
  def getId: String

  def createQuantityAttribute(parentIdIn: Long, attrTypeIdIn: Long, unitIdIn: Long, numberIn: Float, validOnDateIn: Option[Long],
                              inObservationDate: Long, callerManagesTransactionsIn: Boolean = false, sortingIndexIn: Option[Long] = None): /*id*/ Long
  def createDateAttribute(parentIdIn: Long, attrTypeIdIn: Long, dateIn: Long, sortingIndexIn: Option[Long] = None): /*id*/ Long
  def createBooleanAttribute(parentIdIn: Long, attrTypeIdIn: Long, booleanIn: Boolean, validOnDateIn: Option[Long], observationDateIn: Long,
                             sortingIndexIn: Option[Long] = None): /*id*/ Long
  def createFileAttribute(parentIdIn: Long, attrTypeIdIn: Long, descriptionIn: String, originalFileDateIn: Long, storedDateIn: Long,
                          originalFilePathIn: String, readableIn: Boolean, writableIn: Boolean, executableIn: Boolean, sizeIn: Long,
                          md5hashIn: String, inputStreamIn: java.io.FileInputStream, sortingIndexIn: Option[Long] = None): /*id*/ Long
  def createTextAttribute(parentIdIn: Long, attrTypeIdIn: Long, textIn: String, validOnDateIn: Option[Long] = None,
                          observationDateIn: Long = System.currentTimeMillis(), callerManagesTransactionsIn: Boolean = false,
                          sortingIndexIn: Option[Long] = None): /*id*/ Long
  def createRelationToEntity(relationTypeIdIn: Long, entityId1In: Long, entityId2In: Long, validOnDateIn: Option[Long], observationDateIn: Long,
                             sortingIndexIn: Option[Long] = None, callerManagesTransactionsIn: Boolean = false): RelationToEntity
  def createRelationToRemoteEntity(relationTypeIdIn: Long, entityId1In: Long, entityId2In: Long, validOnDateIn: Option[Long], observationDateIn: Long,
                                   remoteInstanceIdIn: String, sortingIndexIn: Option[Long] = None,
                                   callerManagesTransactionsIn: Boolean = false): RelationToRemoteEntity
  def createGroupAndRelationToGroup(entityIdIn: Long, relationTypeIdIn: Long, newGroupNameIn: String, allowMixedClassesInGroupIn: Boolean = false,
                                    validOnDateIn: Option[Long], observationDateIn: Long,
                                    sortingIndexIn: Option[Long], callerManagesTransactionsIn: Boolean = false): (Long, Long)
  def createEntity(nameIn: String, classIdIn: Option[Long] = None, isPublicIn: Option[Boolean] = None): /*id*/ Long
  def createEntityAndRelationToEntity(entityIdIn: Long, relationTypeIdIn: Long, newEntityNameIn: String, isPublicIn: Option[Boolean],
                                      validOnDateIn: Option[Long], observationDateIn: Long, callerManagesTransactionsIn: Boolean = false): (Long, Long)
  def createRelationToGroup(entityIdIn: Long, relationTypeIdIn: Long, groupIdIn: Long, validOnDateIn: Option[Long], observationDateIn: Long,
                            sortingIndexIn: Option[Long] = None, callerManagesTransactionsIn: Boolean = false): (Long, Long)
  def addEntityToGroup(groupIdIn: Long, containedEntityIdIn: Long, sortingIndexIn: Option[Long] = None, callerManagesTransactionsIn: Boolean = false)
  def createOmInstance(idIn: String, isLocalIn: Boolean, addressIn: String, entityIdIn: Option[Long] = None, oldTableName: Boolean = false): Long
  def addUriEntityWithUriAttribute(containingEntityIn: Entity, newEntityNameIn: String, uriIn: String, observationDateIn: Long,
                                   makeThemPublicIn: Option[Boolean], callerManagesTransactionsIn: Boolean,
                                   quoteIn: Option[String] = None): (Entity, RelationToEntity)
  def addHASRelationToEntity(fromEntityIdIn: Long, toEntityIdIn: Long, validOnDateIn: Option[Long], observationDateIn: Long,
                             sortingIndexIn: Option[Long] = None): RelationToEntity
  def getOrCreateClassAndTemplateEntityIds(classNameIn: String, callerManagesTransactionsIn: Boolean): (Long, Long)


  def attributeKeyExists(formIdIn: Long, idIn: Long): Boolean
  def findContainedEntityIds(resultsInOut: mutable.TreeSet[Long], fromEntityIdIn: Long, searchStringIn: String,
                             levelsRemaining: Int = 20, stopAfterAnyFound: Boolean = true): mutable.TreeSet[Long]
  def entityKeyExists(idIn: Long, includeArchived: Boolean = true): Boolean
  def relationTypeKeyExists(idIn: Long): Boolean
  def quantityAttributeKeyExists(idIn: Long): Boolean
  def dateAttributeKeyExists(idIn: Long): Boolean
  def booleanAttributeKeyExists(idIn: Long): Boolean
  def fileAttributeKeyExists(idIn: Long): Boolean
  def textAttributeKeyExists(idIn: Long): Boolean
  def relationToEntityKeyExists(idIn: Long): Boolean
  def groupKeyExists(idIn: Long): Boolean
  def relationToGroupKeysExistAndMatch(id: Long, entityId: Long, relTypeId: Long, groupId: Long): Boolean
  def classKeyExists(idIn: Long): Boolean
  def omInstanceKeyExists(idIn: String): Boolean
  def getEntityData(idIn: Long): Array[Option[Any]]
  def includeArchivedEntities: Boolean
  def getEntityName(idIn: Long): Option[String]
  def isDuplicateEntityName(nameIn: String, selfIdToIgnoreIn: Option[Long] = None): Boolean
  def getSortedAttributes(entityIdIn: Long, startingObjectIndexIn: Int = 0, maxValsIn: Int = 0,
                          onlyPublicEntitiesIn: Boolean = true): (Array[(Long, Attribute)], Int)
  def findRelationType(typeNameIn: String, expectedRows: Option[Int] = Some(1)): java.util.ArrayList[Long]
  def getRelationTypeData(idIn: Long): Array[Option[Any]]
  def getQuantityAttributeData(idIn: Long): Array[Option[Any]]
  def getDateAttributeData(idIn: Long): Array[Option[Any]]
  def getBooleanAttributeData(idIn: Long): Array[Option[Any]]
  def getFileAttributeData(idIn: Long): Array[Option[Any]]
  def getFileAttributeContent(fileAttributeIdIn: Long, outputStreamIn: java.io.OutputStream): (Long, String)
  def getTextAttributeData(idIn: Long): Array[Option[Any]]
  def relationToEntityKeysExistAndMatch(idIn: Long, relTypeIdIn: Long, entityId1In: Long, entityId2In: Long): Boolean
  def relationToRemoteEntityKeyExists(idIn: Long): Boolean
  def relationToRemoteEntityKeysExistAndMatch(idIn: Long, relTypeIdIn: Long, entityId1In: Long, remoteInstanceIdIn: String, entityId2In: Long): Boolean
  def getRelationToEntityData(relTypeIdIn: Long, entityId1In: Long, entityId2In: Long): Array[Option[Any]]
  def getRelationToRemoteEntityData(relTypeIdIn: Long, entityId1In: Long, remoteInstanceIdIn: String, entityId2In: Long): Array[Option[Any]]
  def getGroupData(idIn: Long): Array[Option[Any]]
  def getGroupEntryObjects(groupIdIn: Long, startingObjectIndexIn: Long, maxValsIn: Option[Long] = None): java.util.ArrayList[Entity]
  def getGroupSize(groupIdIn: Long, includeWhichEntitiesIn: Int = 3): Long

  def getHighestSortingIndexForGroup(groupIdIn: Long): Long
  def getRelationToGroupDataByKeys(entityId: Long, relTypeId: Long, groupId: Long): Array[Option[Any]]
  def getRelationToGroupData(idIn: Long): Array[Option[Any]]
  def getGroupEntriesData(groupIdIn: Long, limitIn: Option[Long] = None, includeArchivedEntitiesIn: Boolean = true): List[Array[Option[Any]]]
  def findRelationToAndGroup_OnEntity(entityIdIn: Long, groupNameIn: Option[String] = None): (Option[Long], Option[Long], Option[Long], Boolean)
  def getEntitiesContainingGroup(groupIdIn: Long, startingIndexIn: Long, maxValsIn: Option[Long] = None): java.util.ArrayList[(Long, Entity)]
  def getCountOfEntitiesContainingGroup(groupIdIn: Long): (Long, Long)
  def getClassData(idIn: Long): Array[Option[Any]]
  def getAttributeCount(entityIdIn: Long, includeArchivedEntitiesIn: Boolean = false): Long
  def getRelationToEntityCount(entityIdIn: Long, includeArchivedEntities: Boolean = false): Long
  def getRelationToGroupCount(entityIdIn: Long): Long
  def getClassCount(entityIdIn: Option[Long] = None): Long
  def getClassName(idIn: Long): Option[String]
  def getOmInstanceData(idIn: String): Array[Option[Any]]
  def isDuplicateOmInstanceAddress(addressIn: String, selfIdToIgnoreIn: Option[String] = None): Boolean
  def getGroupsContainingEntitysGroupsIds(groupIdIn: Long, limitIn: Option[Long] = Some(5)): List[Array[Option[Any]]]
  def isEntityInGroup(groupIdIn: Long, entityIdIn: Long): Boolean
  def getAdjacentGroupEntriesSortingIndexes(groupIdIn: Long, sortingIndexIn: Long, limitIn: Option[Long] = None,
                                            forwardNotBackIn: Boolean): List[Array[Option[Any]]]
  def getNearestGroupEntrysSortingIndex(groupIdIn: Long, startingPointSortingIndexIn: Long, forwardNotBackIn: Boolean): Option[Long]
  def getAdjacentAttributesSortingIndexes(entityIdIn: Long, sortingIndexIn: Long, limitIn: Option[Long], forwardNotBackIn: Boolean): List[Array[Option[Any]]]
  def getNearestAttributeEntrysSortingIndex(entityIdIn: Long, startingPointSortingIndexIn: Long, forwardNotBackIn: Boolean): Option[Long]
  def getEntityAttributeSortingIndex(entityIdIn: Long, attributeFormIdIn: Long, attributeIdIn: Long): Long
  def getGroupSortingIndex(groupIdIn: Long, entityIdIn: Long): Long
  def isGroupEntrySortingIndexInUse(groupIdIn: Long, sortingIndexIn: Long): Boolean
  def isAttributeSortingIndexInUse(entityIdIn: Long, sortingIndexIn: Long): Boolean
  def findUnusedAttributeSortingIndex(entityIdIn: Long, startingWithIn: Option[Long] = None): Long
  def findAllEntityIdsByName(nameIn: String, caseSensitive: Boolean = false): java.util.ArrayList[Long]
  def findUnusedGroupSortingIndex(groupIdIn: Long, startingWithIn: Option[Long] = None): Long
  def getTextAttributeByTypeId(parentEntityIdIn: Long, typeIdIn: Long, expectedRows: Option[Int] = None): java.util.ArrayList[TextAttribute]
  def getEntitiesContainingEntity(entityIdIn: Long, startingIndexIn: Long, maxValsIn: Option[Long] = None): java.util.ArrayList[(Long, Entity)]
  def getCountOfGroupsContainingEntity(entityIdIn: Long): Long
  def getContainingGroupsIds(entityIdIn: Long): java.util.ArrayList[Long]
  def getContainingRelationToGroups(entityIdIn: Long, startingIndexIn: Long, maxValsIn: Option[Long] = None): java.util.ArrayList[RelationToGroup]
  def getShouldCreateDefaultAttributes(classIdIn: Long): Option[Boolean]
  def updateClassCreateDefaultAttributes(classIdIn: Long, value: Option[Boolean])
  def getEntitiesOnlyCount(limitByClass: Boolean = false, classIdIn: Option[Long] = None, templateEntity: Option[Long] = None): Long
  def getCountOfEntitiesContainingEntity(entityIdIn: Long): (Long, Long)


  def updateEntitysClass(entityId: Long, classId: Option[Long], callerManagesTransactions: Boolean = false)
  def updateEntityOnlyNewEntriesStickToTop(idIn: Long, newEntriesStickToTop: Boolean)
  def archiveEntity(idIn: Long, callerManagesTransactionsIn: Boolean = false)
  def unarchiveEntity(idIn: Long, callerManagesTransactionsIn: Boolean = false)
  def setIncludeArchivedEntities(in: Boolean): Unit
  def setUserPreference_EntityId(nameIn: String, entityIdIn: Long)
  def updateEntityOnlyPublicStatus(idIn: Long, value: Option[Boolean])
  def updateQuantityAttribute(idIn: Long, parentIdIn: Long, attrTypeIdIn: Long, unitIdIn: Long, numberIn: Float, validOnDateIn: Option[Long],
                              inObservationDate: Long)
  def updateDateAttribute(idIn: Long, parentIdIn: Long, dateIn: Long, attrTypeIdIn: Long)
  def updateBooleanAttribute(idIn: Long, parentIdIn: Long, attrTypeIdIn: Long, booleanIn: Boolean, validOnDateIn: Option[Long],
                             inObservationDate: Long)
  def updateFileAttribute(idIn: Long, parentIdIn: Long, attrTypeIdIn: Long, descriptionIn: String)
  def updateFileAttribute(idIn: Long, parentIdIn: Long, attrTypeIdIn: Long, descriptionIn: String, originalFileDateIn: Long, storedDateIn: Long,
                          originalFilePathIn: String, readableIn: Boolean, writableIn: Boolean, executableIn: Boolean, sizeIn: Long, md5hashIn: String)
  def updateTextAttribute(idIn: Long, parentIdIn: Long, attrTypeIdIn: Long, textIn: String, validOnDateIn: Option[Long], observationDateIn: Long)
  def updateRelationToEntity(oldRelationTypeIdIn: Long, entityId1In: Long, entityId2In: Long,
                             newRelationTypeIdIn: Long, validOnDateIn: Option[Long], observationDateIn: Long)
  def updateRelationToRemoteEntity(oldRelationTypeIdIn: Long, entityId1In: Long, remoteInstanceIdIn: String, entityId2In: Long,
                                   newRelationTypeIdIn: Long, validOnDateIn: Option[Long], observationDateIn: Long)
  def updateGroup(groupIdIn: Long, nameIn: String, allowMixedClassesInGroupIn: Boolean = false, newEntriesStickToTopIn: Boolean = false)
  def updateRelationToGroup(entityIdIn: Long, oldRelationTypeIdIn: Long, newRelationTypeIdIn: Long, oldGroupIdIn: Long, newGroupIdIn: Long,
                            validOnDateIn: Option[Long], observationDateIn: Long)
  def moveRelationToEntity(relationToEntityIdIn: Long, newContainingEntityIdIn: Long, sortingIndexIn: Long): RelationToEntity
  def moveEntityFromEntityToGroup(removingRelationToEntityIn: RelationToEntity, targetGroupIdIn: Long, sortingIndexIn: Long)
  def moveRelationToGroup(relationToGroupIdIn: Long, newContainingEntityIdIn: Long, sortingIndexIn: Long): Long
  def moveEntityFromGroupToEntity(fromGroupIdIn: Long, toEntityIdIn: Long, moveEntityIdIn: Long, sortingIndexIn: Long)
  def moveEntityFromGroupToGroup(fromGroupIdIn: Long, toGroupIdIn: Long, moveEntityIdIn: Long, sortingIndexIn: Long)
  def renumberSortingIndexes(entityIdOrGroupIdIn: Long, callerManagesTransactionsIn: Boolean = false, isEntityAttrsNotGroupEntries: Boolean = true)
  def updateAttributeSorting(entityIdIn: Long, attributeFormIdIn: Long, attributeIdIn: Long, sortingIndexIn: Long)
  def updateEntityInAGroup(groupIdIn: Long, entityIdIn: Long, sortingIndexIn: Long)

  def deleteEntity(idIn: Long, callerManagesTransactionsIn: Boolean = false)
  def deleteQuantityAttribute(idIn: Long)
  def deleteDateAttribute(idIn: Long)
  def deleteBooleanAttribute(idIn: Long)
  def deleteFileAttribute(idIn: Long)
  def deleteTextAttribute(idIn: Long)
  def deleteRelationToEntity(relTypeIdIn: Long, entityId1In: Long, entityId2In: Long)
  def deleteRelationToRemoteEntity(relTypeIdIn: Long, entityId1In: Long, remoteInstanceIdIn: String, entityId2In: Long)
  def deleteRelationToGroup(entityIdIn: Long, relTypeIdIn: Long, groupIdIn: Long)
  def deleteGroupAndRelationsToIt(idIn: Long)
  def deleteRelationType(idIn: Long)
  def deleteClassAndItsTemplateEntity(classIdIn: Long)
  def deleteGroupRelationsToItAndItsEntries(groupidIn: Long)
  def deleteOmInstance(idIn: String): Unit
  def removeEntityFromGroup(groupIdIn: Long, containedEntityIdIn: Long, callerManagesTransactionsIn: Boolean = false)

}
