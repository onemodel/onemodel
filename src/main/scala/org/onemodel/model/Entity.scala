/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003, 2004, and 2010-2016 inclusive, Luke A Call; all rights reserved.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule, guidelines around binary
    distribution, and the GNU Affero General Public License as published by the Free Software Foundation, either version 3
    of the License, or (at your option) any later version.  See the file LICENSE for details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>

  ---------------------------------------------------
  If we ever do port to another database, create the Database interface (removed around 2014-1-1 give or take) and see other changes at that time.
  An alternative method is to use jdbc escapes (but this actually might be even more work?):  http://jdbc.postgresql.org/documentation/head/escapes.html  .
  Another alternative is a layer like JPA, ibatis, hibernate  etc etc.

*/
package org.onemodel.model

import java.io.{FileInputStream, PrintWriter, StringWriter}

import org.onemodel._
import org.onemodel.controller.Controller
import org.onemodel.database.PostgreSQLDatabase

object Entity {
  def nameLength(inDB: PostgreSQLDatabase): Int = PostgreSQLDatabase.entityNameLength

  def isDuplicate(inDB: PostgreSQLDatabase, inName: String, inSelfIdToIgnore: Option[Long] = None): Boolean = inDB.isDuplicateEntity(inName, inSelfIdToIgnore)

  def createEntity(inDB: PostgreSQLDatabase, inName: String, inClassId: Option[Long] = None, isPublicIn: Option[Boolean] = None): Entity = {
    val id: Long = inDB.createEntity(inName, inClassId, isPublicIn)
    new Entity(inDB, id, false)
  }

  def getEntityById(inDB: PostgreSQLDatabase, id: Long): Option[Entity] = {
    try Some(new Entity(inDB, id))
    catch {
      case e: java.lang.Exception =>
        //idea: change this to actually get an "OM_NonexistentEntityException" or such, not text, so it works
        // when we have multiple databases that might not throw the same string!
        if (e.toString.indexOf("does not exist in database.") >= 0) {
          None
        }
        else throw e
    }
  }

  val PRIVACY_PUBLIC = "[PUBLIC]"
  val PRIVACY_NON_PUBLIC = "[NON-PUBLIC]"
  val PRIVACY_UNSET = "[UNSET]"

}

/** Represents one object in the system.
  *
  * This 1st constructor instantiates an existing object from the DB. Generally use Model.createObject() to create a new object.
  * Note: Having Entities and other DB objects be readonly makes the code clearer & avoid some bugs, similarly to reasons for immutability in scala.
  */
class Entity(mDB: PostgreSQLDatabase, mId: Long) {
  if (!mDB.entityKeyExists(mId)) {
    // DON'T CHANGE this msg unless you also change the trap for it in TextUI.java.
    throw new Exception("Key " + mId + " does not exist in database.")
  }


  /** This one is perhaps only called by the database class implementation--so it can return arrays of objects & save more DB hits
    that would have to occur if it only returned arrays of keys. This DOES NOT create a persistent object--but rather should reflect
    one that already exists.
    */
  def this(mDB: PostgreSQLDatabase, mId: Long, nameIn: String, classIdIn: Option[Long] = None, insertionDateIn: Long, publicIn: Option[Boolean]) {
    this(mDB, mId)
    mName = nameIn
    mClassId = classIdIn
    mInsertionDate = insertionDateIn
    mPublic = publicIn
    mAlreadyReadData = true
  }

  /** Allows createEntity to return an instance without duplicating the database check that it Entity(long, PostgreSQLDatabase) does. */
  //idea: replace this w/ a mock? where used? same, for similar code elsewhere? (and EntityTest etc could be with mocks instead of real db use.)
  @SuppressWarnings(Array("unused")) def this(inDB: PostgreSQLDatabase, inID: Long, ignoreMe: Boolean) {
    this(inDB, inID)
  }

  def getName: String = {
    if (!mAlreadyReadData) readDataFromDB()
    mName
  }

  def getClassId: Option[Long] = {
    if (!mAlreadyReadData) readDataFromDB()
    mClassId
  }

  def getClassDefiningEntityId: Option[Long] = {
    val classId = getClassId
    if (classId.isEmpty) None
    else {
      val definingEntityId: Option[Long] = mDB.getClassData(mClassId.get)(1).asInstanceOf[Option[Long]]
      definingEntityId
    }
  }

  def getCreationDate: Long = {
    if (!mAlreadyReadData) readDataFromDB()
    mInsertionDate
  }

  def getCreationDateFormatted: String = {
    Controller.DATEFORMAT.format(new java.util.Date(getCreationDate))
  }

  def getPublic: Option[Boolean] = {
    if (!mAlreadyReadData) readDataFromDB()
    mPublic
  }

  def getPublicStatusString(blankIfUnset: Boolean = true): String = {
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

  protected def readDataFromDB() {
    val entityData = mDB.getEntityData(mId)
    mName = entityData(0).get.asInstanceOf[String]
    mClassId = entityData(1).asInstanceOf[Option[Long]]
    mInsertionDate = entityData(2).get.asInstanceOf[Long]
    mPublic = entityData(3).asInstanceOf[Option[Boolean]]
    mAlreadyReadData = true
  }

  def getIdWrapper: IdWrapper = new IdWrapper(mId)

  def getId: Long = mId

  def getAttrCount: Long = mDB.getAttrCount(mId)

  def getDisplayString_helper: String = {
    var displayString: String = getPublicStatusString()
    displayString += Color.blue(getName)
    val definerInfo = if (mDB.getClassCount(Some(mId)) > 0) "defining entity (template) for " else ""
    val className: Option[String] = if (getClassId.isDefined) mDB.getClassName(getClassId.get) else None
    displayString += (if (className.isDefined) " (" + definerInfo + "class: " + className.get + ")" else "")
    displayString
  }

  def getDisplayString: String = {
    var result = ""
    try {
      result = getDisplayString_helper
    } catch {
      case e: Exception =>
        result += "Unable to get entity description due to: "
        result += {
          val sw: StringWriter = new StringWriter()
          e.printStackTrace(new PrintWriter(sw))
          sw.toString
        }
    }
    result
  }

  /** Removes this object from the system. */
  def delete() = mDB.deleteEntity(mId)

  def archive() = mDB.archiveEntity(mId)

  /** Also for convenience */
  def addQuantityAttribute(inAttrTypeId: Long, inUnitId: Long, inNumber: Float): QuantityAttribute = {
    addQuantityAttribute(inAttrTypeId, inUnitId, inNumber, None, System.currentTimeMillis())
  }

  /** Creates a quantity attribute on this Entity (i.e., "6 inches length"), with default values of "now" for the dates. See "addQuantityAttribute" comment
    * in db implementation file
   for explanation of the parameters. It might also be nice to add the recorder's ID (person or app), but we'd have to do some kind
   of authentication/login 1st? And a GUID for users (as Entities?)?
   See PostgreSQLDatabase.createQuantityAttribute(...) for details.
    */
  def addQuantityAttribute(inAttrTypeId: Long, inUnitId: Long, inNumber: Float, inValidOnDate: Option[Long], inObservationDate: Long): QuantityAttribute = {
    // write it to the database table--w/ a record for all these attributes plus a key indicating which Entity
    // it all goes with
    val id = mDB.createQuantityAttribute(mId, inAttrTypeId, inUnitId, inNumber, inValidOnDate, inObservationDate)
    new QuantityAttribute(mDB, id)
  }

  def getQuantityAttribute(inKey: Long): QuantityAttribute = new QuantityAttribute(mDB, inKey)

  def getTextAttribute(inKey: Long): TextAttribute = new TextAttribute(mDB, inKey)

  def getDateAttribute(inKey: Long): DateAttribute = new DateAttribute(mDB, inKey)

  def getBooleanAttribute(inKey: Long): BooleanAttribute = new BooleanAttribute(mDB, inKey)

  def getFileAttribute(inKey: Long): FileAttribute = new FileAttribute(mDB, inKey)

  /** See addQuantityAttribute(...) methods for comments. */
  def addTextAttribute(inAttrTypeId: Long, inText: String): TextAttribute = addTextAttribute(inAttrTypeId, inText, None, System.currentTimeMillis)

  def addTextAttribute(inAttrTypeId: Long, inText: String, inValidOnDate: Option[Long], inObservationDate: Long,
                       callerManagesTransactionsIn: Boolean = false): TextAttribute = {
    val id = mDB.createTextAttribute(mId, inAttrTypeId, inText, inValidOnDate, inObservationDate, callerManagesTransactionsIn)
    new TextAttribute(mDB, id)
  }

  def addDateAttribute(inAttrTypeId: Long, inDate: Long): DateAttribute = {
    val id = mDB.createDateAttribute(mId, inAttrTypeId, inDate)
    new DateAttribute(mDB, id)
  }

  def addBooleanAttribute(inAttrTypeId: Long, inBoolean: Boolean): BooleanAttribute = addBooleanAttribute(inAttrTypeId, inBoolean, None,
                                                                                                          System.currentTimeMillis)

  def addBooleanAttribute(inAttrTypeId: Long, inBoolean: Boolean, inValidOnDate: Option[Long], inObservationDate: Long): BooleanAttribute = {
    val id = mDB.createBooleanAttribute(mId, inAttrTypeId, inBoolean, inValidOnDate, inObservationDate)
    new BooleanAttribute(mDB, id)
  }

  def addFileAttribute(inAttrTypeId: Long, inFile: java.io.File): FileAttribute = addFileAttribute(inAttrTypeId, inFile.getName, inFile)

  def addFileAttribute(inAttrTypeId: Long, descriptionIn: String, inFile: java.io.File): FileAttribute = {
    if (!inFile.exists()) {
      throw new Exception("File " + inFile.getCanonicalPath + " doesn't exist.")
    }
    // idea: could be a little faster if the md5Hash method were merged into the database method, so that the file is only traversed once (for both
    // upload and md5 calculation).
    var inputStream: java.io.FileInputStream = null
    try {
      inputStream = new FileInputStream(inFile)
      val id = mDB.createFileAttribute(mId, inAttrTypeId, descriptionIn, inFile.lastModified, System.currentTimeMillis, inFile.getCanonicalPath,
                                       inFile.canRead, inFile.canWrite, inFile.canExecute, inFile.length, FileAttribute.md5Hash(inFile), inputStream)
      new FileAttribute(mDB, id)
    }
    finally {
      if (inputStream != null) {
        inputStream.close()
      }
    }
  }

  def addRelationToEntity(inAttrTypeId: Long, inEntityId2: Long, inValidOnDate: Option[Long], inObservationDate: Long): RelationToEntity = {
    val rteId = mDB.createRelationToEntity(inAttrTypeId, getId, inEntityId2, inValidOnDate, inObservationDate).getId
    new RelationToEntity(mDB, rteId, inAttrTypeId, getId, inEntityId2)
  }

  /** Creates then adds a particular kind of rtg to this entity.
    * Returns new group's id, and the new RelationToGroup object
    * */
  def createGroupAndAddHASRelationToIt(newGroupNameIn: String, mixedClassesAllowedIn: Boolean, observationDateIn: Long,
                                       callerManagesTransactionsIn: Boolean = false): (Group, RelationToGroup) = {
    // the "has" relation type that we want should always be the 1st one, since it is created by in the initial app startup; otherwise it seems we can use it
    // anyway:
    val relationTypeId = mDB.findRelationType(PostgreSQLDatabase.theHASrelationTypeName, Some(1))(0)
    val (group, rtg) = addGroupAndRelationToGroup(relationTypeId, newGroupNameIn, mixedClassesAllowedIn, None, observationDateIn,
                                                  callerManagesTransactionsIn)
    (group, rtg)
  }

  /** Like others, returns the new things' IDs. */
  def addGroupAndRelationToGroup(relTypeIdIn: Long, newGroupNameIn: String, allowMixedClassesInGroupIn: Boolean = false, validOnDateIn: Option[Long],
                                 inObservationDate: Long, callerManagesTransactionsIn: Boolean = false): (Group, RelationToGroup) = {
    val (groupId: Long, rtgId: Long) = mDB.createGroupAndRelationToGroup(getId, relTypeIdIn, newGroupNameIn, allowMixedClassesInGroupIn, validOnDateIn,
                                                                         inObservationDate, callerManagesTransactionsIn)
    val group = new Group(mDB, groupId)
    val rtg = new RelationToGroup(mDB, rtgId, getId, relTypeIdIn, groupId)
    (group, rtg)
  }

  /**
   * @return the id of the new RTE
   */
  def addHASRelationToEntity(entityIdIn: Long, validOnDateIn: Option[Long], observationDateIn: Long): RelationToEntity = {
    mDB.addHASRelationToEntity(getId, entityIdIn, validOnDateIn, observationDateIn)
  }

  /** Creates new entity then adds it a particular kind of rte to this entity.
    * Returns new entity's id, and the new RelationToEntity object
    * */
  def createEntityAndAddHASRelationToIt(newEntityNameIn: String, observationDateIn: Long, isPublicIn: Option[Boolean],
                                        callerManagesTransactionsIn: Boolean = false): (Entity, RelationToEntity) = {
    // the "has" relation type that we want should always be the 1st one, since it is created by in the initial app startup; otherwise it seems we can use it
    // anyway:
    val relationTypeId = mDB.findRelationType(PostgreSQLDatabase.theHASrelationTypeName, Some(1))(0)
    val (entity, rte) = addEntityAndRelationToEntity(relationTypeId, newEntityNameIn, None, observationDateIn, isPublicIn,
                                                     callerManagesTransactionsIn)
    (entity, rte)
  }

  /** Like others, returns the new things' IDs. */
  def addEntityAndRelationToEntity(relTypeIdIn: Long, newEntityNameIn: String, validOnDateIn: Option[Long], inObservationDate: Long,
                                   isPublicIn: Option[Boolean], callerManagesTransactionsIn: Boolean = false): (Entity, RelationToEntity) = {
    val (entityId, rteId) = mDB.createEntityAndRelationToEntity(getId, relTypeIdIn, newEntityNameIn, isPublicIn, validOnDateIn, inObservationDate,
                                                                callerManagesTransactionsIn)
    val entity = new Entity(mDB, entityId)
    val rte = new RelationToEntity(mDB, rteId, relTypeIdIn, mId, entityId)
    (entity, rte)
  }

  /**
    * @return the new group's id.
    */
  def addRelationToGroup(relTypeIdIn: Long, groupIdIn: Long, validOnDateIn: Option[Long], observationDateIn: Long): RelationToGroup = {
    val newRtgId = mDB.createRelationToGroup(getId, relTypeIdIn, groupIdIn, validOnDateIn, observationDateIn)
    new RelationToGroup(mDB, newRtgId, getId, relTypeIdIn, groupIdIn, validOnDateIn, observationDateIn)
  }

  var mAlreadyReadData: Boolean = false
  var mName: String = null
  var mClassId: Option[Long] = None
  var mInsertionDate: Long = -1
  var mPublic: Option[Boolean] = None
}