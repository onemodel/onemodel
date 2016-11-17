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

import org.onemodel.core.model.{RelationToRemoteEntity, Entity, Attribute, RelationToEntity}
import org.onemodel.core.{OmDatabaseException, Util}

object Database {
  val dbNamePrefix = "om_"
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
    //MAKE SURE THESE MATCH WITH THOSE IN attributeKeyExists and getAttributeFormName, and the range in the db constraint valid_attribute_form_id !
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


}
abstract class Database {
  def isRemote: Boolean
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
  def isDuplicateEntity(nameIn: String, selfIdToIgnoreIn: Option[Long] = None): Boolean
  def getSortedAttributes(entityIdIn: Long, startingObjectIndexIn: Int = 0, maxValsIn: Int = 0,
                          onlyPublicEntitiesIn: Boolean = true): (Array[(Long, Attribute)], Int)
  def findRelationType(typeNameIn: String, expectedRows: Option[Int] = Some(1)): Array[Long]
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
  def getRelationToGroupData(entityId: Long, relTypeId: Long, groupId: Long): Array[Option[Any]]
  def getRelationToGroupDataById(idIn: Long): Array[Option[Any]]
  def getClassData(idIn: Long): Array[Option[Any]]
  def getAttrCount(entityIdIn: Long, includeArchivedEntitiesIn: Boolean = false): Long
  def getClassCount(entityIdIn: Option[Long] = None): Long
  def getClassName(idIn: Long): Option[String]
  def getOmInstanceData(idIn: String): Array[Option[Any]]
  def isDuplicateOmInstance(addressIn: String, selfIdToIgnoreIn: Option[String] = None): Boolean


  def updateEntitysClass(entityId: Long, classId: Option[Long], callerManagesTransactions: Boolean = false)
  def updateEntityOnlyNewEntriesStickToTop(idIn: Long, newEntriesStickToTop: Boolean)
  def archiveEntity(idIn: Long, callerManagesTransactionsIn: Boolean = false)
  def unarchiveEntity(idIn: Long, callerManagesTransactionsIn: Boolean = false)
  def addHASRelationToEntity(fromEntityIdIn: Long, toEntityIdIn: Long, validOnDateIn: Option[Long], observationDateIn: Long,
                             sortingIndexIn: Option[Long] = None): RelationToEntity
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
