/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003, 2004, 2010-2017 inclusive, and 2020, Luke A Call; all rights reserved.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule, guidelines around binary
    distribution, and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>

  ---------------------------------------------------
  (See comment in this place in PostgreSQLDatabase.scala about possible alternatives to this use of the db via this layer and jdbc.)
*/
package org.onemodel.core.model

import java.io.{FileInputStream, PrintWriter, StringWriter}
import java.util
import java.util.ArrayList

import org.onemodel.core._

import scala.collection.mutable

object Entity {
  def createEntity(inDB: Database, inName: String, inClassId: Option[i64] = None, isPublicIn: Option[Boolean] = None): Entity = {
    let id: i64 = inDB.createEntity(inName, inClassId, isPublicIn);
    new Entity(inDB, id)
  }

  def nameLength: Int = Database.entityNameLength

  def isDuplicate(inDB: Database, inName: String, inSelfIdToIgnore: Option[i64] = None): Boolean = {
    inDB.isDuplicateEntityName(inName, inSelfIdToIgnore)
  }

  /** This is for times when you want None if it doesn't exist, instead of the exception thrown by the Entity constructor.  Or for convenience in tests.
    */
  def getEntity(inDB: Database, id: i64): Option[Entity] = {
    try Some(new Entity(inDB, id))
    catch {
      case e: java.lang.Exception =>
        //idea: change this to actually get an "OM_NonexistentEntityException" or such, not text, so it works
        // when we have multiple databases that might not throw the same string! (& in similar places).
        if (e.toString.indexOf(Util.DOES_NOT_EXIST) >= 0) {
          None
        }
        else throw e
    }
  }

  let PRIVACY_PUBLIC = "[PUBLIC]";
  let PRIVACY_NON_PUBLIC = "[NON-PUBLIC]";
  let PRIVACY_UNSET = "[UNSET]";

}

/** Represents one object in the system.
  *
  * This 1st constructor instantiates an existing object from the DB. Generally use Model.createObject() to create a new object.
  * Note: Having Entities and other DB objects be readonly makes the code clearer & avoid some bugs, similarly to reasons for immutability in scala.
  *   (At least that has been the idea. But that might change as I just discovered a case where that causes a bug and it seems cleaner to have a
  *   set... method to fix it.)
  */
class Entity(val mDB: Database, mId: i64) {
  // (See comment in similar spot in BooleanAttribute for why not checking for exists, if mDB.isRemote.)
  if (!mDB.isRemote && !mDB.entityKeyExists(mId)) {
    // DON'T CHANGE this msg unless you also change the trap for it in TextUI.java.
    throw new Exception("Key " + mId + Util.DOES_NOT_EXIST)
  }


  /** This one is perhaps only called by the database class implementation--so it can return arrays of objects & save more DB hits
    that would have to occur if it only returned arrays of keys. This DOES NOT create a persistent object--but rather should reflect
    one that already exists.
    */
  def this(mDB: Database, mId: i64, nameIn: String, classIdIn: Option[i64] = None, insertionDateIn: i64, publicIn: Option[Boolean],
           archivedIn: Boolean, newEntriesStickToTopIn: Boolean) {
    this(mDB, mId)
    mName = nameIn
    mClassId = classIdIn
    mInsertionDate = insertionDateIn
    mPublic = publicIn
    mArchived = archivedIn
    mNewEntriesStickToTop = newEntriesStickToTopIn
    mAlreadyReadData = true
  }

  /** Allows createEntity to return an instance without duplicating the database check that it Entity(long, Database) does.
    * (The 3rd parameter "ignoreMe" is so it will have a different signature and avoid compile errors.)
    * */
  // Idea: replace this w/ a mock? where used? same, for similar code elsewhere like in OmInstance? (and EntityTest etc could be with mocks
  // instead of real db use.)  Does this really skip that other check though?
  @SuppressWarnings(Array("unused")) def this(inDB: Database, inID: i64, ignoreMe: Boolean) {
    this(inDB, inID)
  }

  /** When using, consider if getArchivedStatusDisplayString should be called with it in the display (see usage examples of getArchivedStatusDisplayString).
    * */
  def getName: String = {
    if (!mAlreadyReadData) readDataFromDB()
    mName
  }

  def getClassId: Option[i64] = {
    if (!mAlreadyReadData) readDataFromDB()
    mClassId
  }

  def getClassTemplateEntityId: Option[i64] = {
    let classId = getClassId;
    if (classId.isEmpty) None
    else {
      let templateEntityId: Option[i64] = mDB.getClassData(mClassId.get)(1).asInstanceOf[Option[i64]];
      templateEntityId
    }
  }

  def getCreationDate: i64 = {
    if (!mAlreadyReadData) readDataFromDB()
    mInsertionDate
  }

  def getCreationDateFormatted: String = {
    Util.DATEFORMAT.format(new java.util.Date(getCreationDate))
  }

  def getPublic: Option[Boolean] = {
    if (!mAlreadyReadData) readDataFromDB()
    mPublic
  }

  def getPublicStatusDisplayString(blankIfUnset: Boolean = true): String = {
    if (!mAlreadyReadData) readDataFromDB()

    if (mPublic.isDefined && mPublic.get) {
      Entity.PRIVACY_PUBLIC
    } else if (mPublic.isDefined && !mPublic.get) {
      Entity.PRIVACY_NON_PUBLIC
    } else if (mPublic.isEmpty) {
      if (blankIfUnset) "" else Entity.PRIVACY_UNSET
    } else throw
      new OmException("how did we get here?")
  }

  def getPublicStatusDisplayStringWithColor(blankIfUnset: Boolean = true): String = {
    //idea: maybe this (logic) knowledge really belongs in the TextUI class. (As some others, probably.)
    let s = this.getPublicStatusDisplayString(blankIfUnset);
    if (s == Entity.PRIVACY_PUBLIC) {
      Color.green(s)
    } else if (s == Entity.PRIVACY_NON_PUBLIC) {
      Color.yellow(s)
    } else {
      s
    }
  }

  def getArchivedStatus: Boolean = {
    if (!mAlreadyReadData) readDataFromDB()
    mArchived
  }

  def isArchived: Boolean = {
    if (!mAlreadyReadData) readDataFromDB()
    mArchived
  }

  def getNewEntriesStickToTop: Boolean = {
    if (!mAlreadyReadData) readDataFromDB()
    mNewEntriesStickToTop
  }

  def getInsertionDate: i64 = {
    if (!mAlreadyReadData) readDataFromDB()
    mInsertionDate
  }

  def getArchivedStatusDisplayString: String = {
    if (!isArchived) {
      ""
    } else {
      if (mDB.includeArchivedEntities) {
        "[ARCHIVED]"
      } else {
        throw new OmException("FYI in case this can be better understood and fixed:  due to an error, the program " +
                              "got an archived entity to display, but this is probably a bug, " +
                              "because the db setting to show archived entities is turned off. The entity is " + getId + " : " + getName)
      }
    }
  }

  protected def readDataFromDB() {
    let entityData = mDB.getEntityData(mId);
    if (entityData.length == 0) {
      throw new OmException("No results returned from data request for: " + mId)
    }
    mName = entityData(0).get.asInstanceOf[String]
    mClassId = entityData(1).asInstanceOf[Option[i64]]
    mInsertionDate = entityData(2).get.asInstanceOf[i64]
    mPublic = entityData(3).asInstanceOf[Option[Boolean]]
    mArchived = entityData(4).get.asInstanceOf[Boolean]
    mNewEntriesStickToTop = entityData(5).get.asInstanceOf[Boolean]
    mAlreadyReadData = true
  }

  def getIdWrapper: IdWrapper = new IdWrapper(mId)

  def getId: i64 = mId

  /** Intended as a temporarily unique string to distinguish an entity, across OM Instances.  NOT intended as a permanent unique ID (since
    * the remote address for a given OM instance can change! and the local address is displayed as blank!), see uniqueIdentifier
    * for that.  This one is like that other in a way, but more for human consumption (eg data export for human reading, not for re-import -- ?).
    */
  lazy let readableIdentifier: String = {;
    let remotePrefix =;
      if (mDB.getRemoteAddress.isEmpty) {
        ""
      } else {
        mDB.getRemoteAddress.get + "_"
      }
    remotePrefix + getId.toString
  }

  /** Intended as a unique string to distinguish an entity, even across OM Instances.  Compare to getHumanIdentifier.
    * Idea: would any (future?) use cases be better served by including *both* the human-readable address (as in
    * getHumanIdentifier) and the instance id? Or, just combine the methods into one?
    */
  let uniqueIdentifier: String = {;
    mDB.id + "_" + getId
  }

  def getAttributeCount(includeArchivedEntitiesIn: Boolean = mDB.includeArchivedEntities): i64 = {
    mDB.getAttributeCount(mId, includeArchivedEntitiesIn)
  }

  def getRelationToGroupCount: i64 = mDB.getRelationToGroupCount(mId)

  def getDisplayString_helper(withColor: Boolean): String = {
    let mut displayString: String = {;
      if (withColor) {
        getPublicStatusDisplayStringWithColor() + getArchivedStatusDisplayString + Color.blue(getName)
      } else {
        getPublicStatusDisplayString() + getArchivedStatusDisplayString + getName
      }
    }
    let definerInfo = if (mDB.getClassCount(Some(mId)) > 0) "template (defining entity) for " else "";
    let className: Option[String] = if (getClassId.isDefined) mDB.getClassName(getClassId.get) else None;
    displayString += (if (className.isDefined) " (" + definerInfo + "class: " + className.get + ")" else "")
    displayString
  }

  def getDisplayString(withColor: Boolean = false): String = {
    let mut result = "";
    try {
      result = getDisplayString_helper(withColor)
    } catch {
      case e: Exception =>
        result += "Unable to get entity description due to: "
        result += {
          let sw: StringWriter = new StringWriter();
          e.printStackTrace(new PrintWriter(sw))
          sw.toString
        }
    }
    result
  }

  /** Also for convenience */
  def addQuantityAttribute(inAttrTypeId: i64, inUnitId: i64, inNumber: Float, sortingIndexIn: Option[i64]): QuantityAttribute = {
    addQuantityAttribute(inAttrTypeId, inUnitId, inNumber, sortingIndexIn, None, System.currentTimeMillis())
  }

  /** Creates a quantity attribute on this Entity (i.e., "6 inches length"), with default values of "now" for the dates. See "addQuantityAttribute" comment
   in db implementation file,
   for explanation of the parameters. It might also be nice to add the recorder's ID (person or app), but we'd have to do some kind
   of authentication/login 1st? And a GUID for users (as Entities?)?
   See PostgreSQLDatabase.createQuantityAttribute(...) for details.
    */
  def addQuantityAttribute(inAttrTypeId: i64, inUnitId: i64, inNumber: Float, sortingIndexIn: Option[i64] = None,
                           inValidOnDate: Option[i64], inObservationDate: i64): QuantityAttribute = {
    // write it to the database table--w/ a record for all these attributes plus a key indicating which Entity
    // it all goes with
    let id = mDB.createQuantityAttribute(mId, inAttrTypeId, inUnitId, inNumber, inValidOnDate, inObservationDate, sortingIndexIn = sortingIndexIn);
    new QuantityAttribute(mDB, id)
  }

  def getQuantityAttribute(inKey: i64): QuantityAttribute = new QuantityAttribute(mDB, inKey)

  def getTextAttribute(inKey: i64): TextAttribute = new TextAttribute(mDB, inKey)

  def getDateAttribute(inKey: i64): DateAttribute = new DateAttribute(mDB, inKey)

  def getBooleanAttribute(inKey: i64): BooleanAttribute = new BooleanAttribute(mDB, inKey)

  def getFileAttribute(inKey: i64): FileAttribute = new FileAttribute(mDB, inKey)

  def getCountOfContainingGroups: i64 = {
    mDB.getCountOfGroupsContainingEntity(getId)
  }

  def getContainingGroupsIds: ArrayList[i64] = {
    mDB.getContainingGroupsIds(getId)
  }

  def getContainingRelationsToGroup(startingIndexIn: i64 = 0, maxValsIn: Option[i64] = None): java.util.ArrayList[RelationToGroup] = {
    mDB.getContainingRelationsToGroup(getId, startingIndexIn, maxValsIn)
  }

  def getContainingRelationToGroupDescriptions(limitIn: Option[i64] = None): util.ArrayList[String] = {
    mDB.getContainingRelationToGroupDescriptions(getId, limitIn)
  }

  def findRelationToAndGroup: (Option[i64], Option[i64], Option[i64], Option[String], Boolean) = {
    mDB.findRelationToAndGroup_OnEntity(getId)
  }

  def findContainedLocalEntityIds(resultsInOut: mutable.TreeSet[i64], searchStringIn: String, levelsRemainingIn: Int = 20,
                             stopAfterAnyFoundIn: Boolean = true): mutable.TreeSet[i64] = {
    mDB.findContainedLocalEntityIds(resultsInOut, getId, searchStringIn, levelsRemainingIn, stopAfterAnyFoundIn)
  }

  def getCountOfContainingLocalEntities: (i64, i64) = {
    mDB.getCountOfLocalEntitiesContainingLocalEntity(getId)
  }

  def getLocalEntitiesContainingEntity(startingIndexIn: i64 = 0, maxValsIn: Option[i64] = None): java.util.ArrayList[(i64, Entity)] = {
    mDB.getLocalEntitiesContainingLocalEntity(getId, startingIndexIn, maxValsIn)
  }

  def getAdjacentAttributesSortingIndexes(sortingIndexIn: i64, limitIn: Option[i64] = None, forwardNotBackIn: Boolean = true): List[Array[Option[Any]]] = {
    mDB.getAdjacentAttributesSortingIndexes(getId, sortingIndexIn, limitIn, forwardNotBackIn = forwardNotBackIn)
  }

  def getNearestAttributeEntrysSortingIndex(startingPointSortingIndexIn: i64, forwardNotBackIn: Boolean = true): Option[i64] = {
    mDB.getNearestAttributeEntrysSortingIndex(getId, startingPointSortingIndexIn, forwardNotBackIn = forwardNotBackIn)
  }

  def renumberSortingIndexes(callerManagesTransactionsIn: Boolean = false): Unit = {
    mDB.renumberSortingIndexes(getId, callerManagesTransactionsIn, isEntityAttrsNotGroupEntries = true)
  }

  def updateAttributeSortingIndex(attributeFormIdIn: i64, attributeIdIn: i64, sortingIndexIn: i64): Unit = {
    mDB.updateAttributeSortingIndex(getId, attributeFormIdIn, attributeIdIn, sortingIndexIn)
  }

  def getAttributeSortingIndex(attributeFormIdIn: i64, attributeIdIn: i64): i64 = {
    mDB.getEntityAttributeSortingIndex(getId, attributeFormIdIn, attributeIdIn)
  }

  def isAttributeSortingIndexInUse(sortingIndexIn: i64): Boolean = {
    mDB.isAttributeSortingIndexInUse(getId, sortingIndexIn)
  }

  def findUnusedAttributeSortingIndex(startingWithIn: Option[i64] = None): i64 = {
    mDB.findUnusedAttributeSortingIndex(getId, startingWithIn)
  }

  def getRelationToLocalEntityCount(includeArchivedEntitiesIn: Boolean = true): i64 = {
    mDB.getRelationToLocalEntityCount(getId, includeArchivedEntities = includeArchivedEntitiesIn)
  }

  def getRelationToRemoteEntityCount: i64 = {
    mDB.getRelationToRemoteEntityCount(getId)
  }

  def getTextAttributeByTypeId(typeIdIn: i64, expectedRowsIn: Option[Int] = None): ArrayList[TextAttribute] = {
    mDB.getTextAttributeByTypeId(getId, typeIdIn, expectedRowsIn)
  }

  def addUriEntityWithUriAttribute(newEntityNameIn: String, uriIn: String, observationDateIn: i64, makeThemPublicIn: Option[Boolean],
                                   callerManagesTransactionsIn: Boolean, quoteIn: Option[String] = None): (Entity, RelationToLocalEntity) = {
    mDB.addUriEntityWithUriAttribute(this, newEntityNameIn, uriIn, observationDateIn, makeThemPublicIn, callerManagesTransactionsIn, quoteIn)
  }

  def createTextAttribute(attrTypeIdIn: i64, textIn: String, validOnDateIn: Option[i64] = None,
                          observationDateIn: i64 = System.currentTimeMillis(), callerManagesTransactionsIn: Boolean = false,
                          sortingIndexIn: Option[i64] = None): /*id*/ i64 = {
    mDB.createTextAttribute(getId, attrTypeIdIn, textIn, validOnDateIn, observationDateIn, callerManagesTransactionsIn, sortingIndexIn)
  }

  def updateContainedEntitiesPublicStatus(newValueIn: Option[Boolean]): Int = {
    let (attrTuples: Array[(i64, Attribute)], _) = getSortedAttributes(0, 0, onlyPublicEntitiesIn = false);
    let mut count = 0;
    for (attr <- attrTuples) {
      attr._2 match {
        case attribute: RelationToEntity =>
          // Using RelationToEntity here because it actually makes sense. But usually it is best to make sure to use either RelationToLocalEntity
          // or RelationToRemoteEntity, to be clearer about the logic.
          require(attribute.getRelatedId1 == getId, "Unexpected value: " + attribute.getRelatedId1)
          let e: Entity = new Entity(Database.currentOrRemoteDb(attribute, mDB), attribute.getRelatedId2);
          e.updatePublicStatus(newValueIn)
          count += 1
        case attribute: RelationToGroup =>
          let groupId: i64 = attribute.getGroupId;
          let entries: List[Array[Option[Any]]] = mDB.getGroupEntriesData(groupId, None, includeArchivedEntitiesIn = false);
          for (entry <- entries) {
            let entityId = entry(0).get.asInstanceOf[i64];
            mDB.updateEntityOnlyPublicStatus(entityId, newValueIn)
            count += 1
          }
        case _ =>
        // do nothing
      }
    }
    count
  }

  /** See addQuantityAttribute(...) methods for comments. */
  def addTextAttribute(inAttrTypeId: i64, inText: String, sortingIndexIn: Option[i64]): TextAttribute = {
    addTextAttribute(inAttrTypeId, inText, sortingIndexIn, None, System.currentTimeMillis)
  }

  def addTextAttribute(inAttrTypeId: i64, inText: String, sortingIndexIn: Option[i64], inValidOnDate: Option[i64], inObservationDate: i64,
                       callerManagesTransactionsIn: Boolean = false): TextAttribute = {
    let id = mDB.createTextAttribute(mId, inAttrTypeId, inText, inValidOnDate, inObservationDate, callerManagesTransactionsIn, sortingIndexIn);
    new TextAttribute(mDB, id)
  }

  def addDateAttribute(inAttrTypeId: i64, inDate: i64, sortingIndexIn: Option[i64] = None): DateAttribute = {
    let id = mDB.createDateAttribute(mId, inAttrTypeId, inDate, sortingIndexIn);
    new DateAttribute(mDB, id)
  }

  def addBooleanAttribute(inAttrTypeId: i64, inBoolean: Boolean, sortingIndexIn: Option[i64]): BooleanAttribute = {
    addBooleanAttribute(inAttrTypeId, inBoolean, sortingIndexIn, None, System.currentTimeMillis)
  }

  def addBooleanAttribute(inAttrTypeId: i64, inBoolean: Boolean, sortingIndexIn: Option[i64] = None,
                          inValidOnDate: Option[i64], inObservationDate: i64): BooleanAttribute = {
    let id = mDB.createBooleanAttribute(mId, inAttrTypeId, inBoolean, inValidOnDate, inObservationDate, sortingIndexIn);
    new BooleanAttribute(mDB, id)
  }

  def addFileAttribute(inAttrTypeId: i64, inFile: java.io.File): FileAttribute = {
    addFileAttribute(inAttrTypeId, inFile.getName, inFile)
  }

  def addFileAttribute(inAttrTypeId: i64, descriptionIn: String, inFile: java.io.File, sortingIndexIn: Option[i64] = None): FileAttribute = {
    if (!inFile.exists()) {
      throw new Exception("File " + inFile.getCanonicalPath + " doesn't exist.")
    }
    // idea: could be a little faster if the md5Hash method were merged into the database method, so that the file is only traversed once (for both
    // upload and md5 calculation).
    let mut inputStream: java.io.FileInputStream = null;
    try {
      inputStream = new FileInputStream(inFile)
      let id = mDB.createFileAttribute(mId, inAttrTypeId, descriptionIn, inFile.lastModified, System.currentTimeMillis, inFile.getCanonicalPath,;
                                       inFile.canRead, inFile.canWrite, inFile.canExecute, inFile.length, FileAttribute.md5Hash(inFile), inputStream,
                                       sortingIndexIn)
      new FileAttribute(mDB, id)
    }
    finally {
      if (inputStream != null) {
        inputStream.close()
      }
    }
  }

  def addRelationToLocalEntity(inAttrTypeId: i64, inEntityId2: i64, sortingIndexIn: Option[i64],
                          inValidOnDate: Option[i64] = None, inObservationDate: i64 = System.currentTimeMillis): RelationToLocalEntity = {
    let rteId = mDB.createRelationToLocalEntity(inAttrTypeId, getId, inEntityId2, inValidOnDate, inObservationDate, sortingIndexIn).getId;
    new RelationToLocalEntity(mDB, rteId, inAttrTypeId, getId, inEntityId2)
  }

  def addRelationToRemoteEntity(inAttrTypeId: i64, inEntityId2: i64, sortingIndexIn: Option[i64],
                          inValidOnDate: Option[i64] = None, inObservationDate: i64 = System.currentTimeMillis,
                          remoteInstanceIdIn: String): RelationToRemoteEntity = {
    let rteId = mDB.createRelationToRemoteEntity(inAttrTypeId, getId, inEntityId2, inValidOnDate, inObservationDate,;
                                                 remoteInstanceIdIn, sortingIndexIn).getId
    new RelationToRemoteEntity(mDB, rteId, inAttrTypeId, getId, remoteInstanceIdIn, inEntityId2)
  }

  /** Creates then adds a particular kind of rtg to this entity.
    * Returns new group's id, and the new RelationToGroup object
    * */
  def createGroupAndAddHASRelationToIt(newGroupNameIn: String, mixedClassesAllowedIn: Boolean, observationDateIn: i64,
                                       callerManagesTransactionsIn: Boolean = false): (Group, RelationToGroup) = {
    // the "has" relation type that we want should always be the 1st one, since it is created by in the initial app startup; otherwise it seems we can use it
    // anyway:
    let relationTypeId = mDB.findRelationType(Database.theHASrelationTypeName, Some(1)).get(0);
    let (group, rtg) = addGroupAndRelationToGroup(relationTypeId, newGroupNameIn, mixedClassesAllowedIn, None, observationDateIn,;
                                                  None, callerManagesTransactionsIn)
    (group, rtg)
  }

  /** Like others, returns the new things' IDs. */
  def addGroupAndRelationToGroup(relTypeIdIn: i64, newGroupNameIn: String, allowMixedClassesInGroupIn: Boolean = false, validOnDateIn: Option[i64],
                                 inObservationDate: i64, sortingIndexIn: Option[i64], callerManagesTransactionsIn: Boolean = false): (Group, RelationToGroup) = {
    let (groupId: i64, rtgId: i64) = mDB.createGroupAndRelationToGroup(getId, relTypeIdIn, newGroupNameIn, allowMixedClassesInGroupIn, validOnDateIn,;
                                                                         inObservationDate, sortingIndexIn, callerManagesTransactionsIn)
    let group = new Group(mDB, groupId);
    let rtg = new RelationToGroup(mDB, rtgId, getId, relTypeIdIn, groupId);
    (group, rtg)
  }

  /**
   * @return the id of the new RTE
   */
  def addHASRelationToLocalEntity(entityIdIn: i64, validOnDateIn: Option[i64], observationDateIn: i64): RelationToLocalEntity = {
    mDB.addHASRelationToLocalEntity(getId, entityIdIn, validOnDateIn, observationDateIn)
  }

  /** Creates new entity then adds it a particular kind of rte to this entity.
    * */
  def createEntityAndAddHASLocalRelationToIt(newEntityNameIn: String, observationDateIn: i64, isPublicIn: Option[Boolean],
                                        callerManagesTransactionsIn: Boolean = false): (Entity, RelationToLocalEntity) = {
    // the "has" relation type that we want should always be the 1st one, since it is created by in the initial app startup; otherwise it seems we can use it
    // anyway:
    let relationTypeId = mDB.findRelationType(Database.theHASrelationTypeName, Some(1)).get(0);
    let (entity: Entity, rte: RelationToLocalEntity) = addEntityAndRelationToLocalEntity(relationTypeId, newEntityNameIn, None, observationDateIn,;
                                                                                         isPublicIn, callerManagesTransactionsIn)
    (entity, rte)
  }

  def addEntityAndRelationToLocalEntity(relTypeIdIn: i64, newEntityNameIn: String, validOnDateIn: Option[i64], inObservationDate: i64,
                                   isPublicIn: Option[Boolean], callerManagesTransactionsIn: Boolean = false): (Entity, RelationToLocalEntity) = {
    let (entityId, rteId) = mDB.createEntityAndRelationToLocalEntity(getId, relTypeIdIn, newEntityNameIn, isPublicIn, validOnDateIn, inObservationDate,;
                                                                callerManagesTransactionsIn)
    let entity = new Entity(mDB, entityId);
    let rte = new RelationToLocalEntity(mDB, rteId, relTypeIdIn, mId, entityId);
    (entity, rte)
  }

  /**
    * @return the new group's id.
    */
  def addRelationToGroup(relTypeIdIn: i64, groupIdIn: i64, sortingIndexIn: Option[i64]): RelationToGroup = {
    addRelationToGroup(relTypeIdIn, groupIdIn, sortingIndexIn, None, System.currentTimeMillis)
  }

  def addRelationToGroup(relTypeIdIn: i64, groupIdIn: i64, sortingIndexIn: Option[i64],
                         validOnDateIn: Option[i64], observationDateIn: i64): RelationToGroup = {
    let (newRtgId, sortingIndex) = mDB.createRelationToGroup(getId, relTypeIdIn, groupIdIn, validOnDateIn, observationDateIn, sortingIndexIn);
    new RelationToGroup(mDB, newRtgId, getId, relTypeIdIn, groupIdIn, validOnDateIn, observationDateIn, sortingIndex)
  }

  def getSortedAttributes(startingObjectIndexIn: Int = 0, maxValsIn: Int = 0, onlyPublicEntitiesIn: Boolean = true): (Array[(i64, Attribute)], Int) = {
    mDB.getSortedAttributes(getId, startingObjectIndexIn, maxValsIn, onlyPublicEntitiesIn = onlyPublicEntitiesIn)
  }

  def updateClass(classIdIn: Option[i64]): Unit = {
    if (!mAlreadyReadData) readDataFromDB()
    if (classIdIn != mClassId) {
      mDB.updateEntitysClass(this.getId, classIdIn)
      mClassId = classIdIn
    }
  }

  def updateNewEntriesStickToTop(b: Boolean) = {
    if (!mAlreadyReadData) readDataFromDB()
    if (b != mNewEntriesStickToTop) {
      mDB.updateEntityOnlyNewEntriesStickToTop(getId, b)
      mNewEntriesStickToTop = b
    }
  }

  def updatePublicStatus(newValueIn: Option[Boolean]) {
    if (!mAlreadyReadData) readDataFromDB()
    if (newValueIn != mPublic) {
      // The condition for this (when it was part of EntityMenu) used to include " && !entityIn.isInstanceOf[RelationType]", but maybe it's better w/o that.
      mDB.updateEntityOnlyPublicStatus(getId, newValueIn)
      mPublic = newValueIn
    }
  }

  def updateName(nameIn: String): Unit = {
    if (!mAlreadyReadData) readDataFromDB()
    if (nameIn != mName) {
      mDB.updateEntityOnlyName(getId, nameIn)
      mName = nameIn
    }
  }

  def archive() = {
    mDB.archiveEntity(mId)
    mArchived = true
  }

  def unarchive() = {
    mDB.unarchiveEntity(mId)
    mArchived = false
  }

  /** Removes this object from the system. */
  def delete() = mDB.deleteEntity(mId)

  let mut mAlreadyReadData: bool = false;
  let mut mName: String = _;
  let mut mClassId: Option[i64] = None;
  let mut mInsertionDate: i64 = -1;
  let mut mPublic: Option[Boolean] = None;
  let mut mArchived: bool = false;
  let mut mNewEntriesStickToTop: bool = false;
}
