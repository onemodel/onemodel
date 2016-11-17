/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2014-2016 inclusive, Luke A. Call; all rights reserved.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule, guidelines around binary
    distribution, and the GNU Affero General Public License as published by the Free Software Foundation, either version 3
    of the License, or (at your option) any later version.  See the file LICENSE for details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>

  ---------------------------------------------------
  (See comment in this place in PostgreSQLDatabase.scala about possible alternatives to this use of the db via this layer and jdbc.)
*/
package org.onemodel.core.model

import org.onemodel.core.{Util, Color, OmException}
import org.onemodel.core.database.Database

/** See comments on similar methods in RelationToEntity. */
class Group(mDB: Database, mId: Long) {
  // (See comment at similar location in BooleanAttribute.)
  if (!mDB.isRemote && !mDB.groupKeyExists(mId: Long)) {
    throw new Exception("Key " + mId + Util.DOES_NOT_EXIST)
  }

  /** See comment about these 2 dates in Database.createTables() */
  def this(mDB: Database, idIn: Long, nameIn: String, insertionDateIn: Long, mixedClassesAllowedIn: Boolean, newEntriesStickToTopIn: Boolean) {
    this(mDB, idIn)
    mName = nameIn
    mInsertionDate = insertionDateIn
    mMixedClassesAllowed = mixedClassesAllowedIn
    mNewEntriesStickToTop = newEntriesStickToTopIn
  }

  def readDataFromDB() {
    val relationData: Array[Option[Any]] = mDB.getGroupData(mId)
    mName = relationData(0).get.asInstanceOf[String]
    mInsertionDate = relationData(1).get.asInstanceOf[Long]
    mMixedClassesAllowed = relationData(2).get.asInstanceOf[Boolean]
    mNewEntriesStickToTop = relationData(3).get.asInstanceOf[Boolean]
    mAlreadyReadData = true
  }

  def update(attrTypeIdInIGNOREDFORSOMEREASON: Option[Long] = None, nameIn: Option[String] = None, allowMixedClassesInGroupIn: Option[Boolean] = None,
             newEntriesStickToTopIn: Option[Boolean] = None,
             validOnDateInIGNORED4NOW: Option[Long], observationDateInIGNORED4NOW: Option[Long]) {

    mDB.updateGroup(mId,
                    if (nameIn.isEmpty) getName else nameIn.get,
                    if (allowMixedClassesInGroupIn.isEmpty) getMixedClassesAllowed else allowMixedClassesInGroupIn.get,
                    if (newEntriesStickToTopIn.isEmpty) getNewEntriesStickToTop else newEntriesStickToTopIn.get)

    if (nameIn.isDefined) mName = nameIn.get
    if (allowMixedClassesInGroupIn.isDefined) mMixedClassesAllowed = allowMixedClassesInGroupIn.get
    if (newEntriesStickToTopIn.isDefined) mNewEntriesStickToTop = newEntriesStickToTopIn.get
  }

  /** Removes this object from the system. */
  def delete() = mDB.deleteGroupAndRelationsToIt(mId)

  /** Removes an entity from this group. */
  def removeEntity(entityId: Long) = mDB.removeEntityFromGroup(mId, entityId)

  def deleteWithEntities() = mDB.deleteGroupRelationsToItAndItsEntries(mId)

  // idea: cache this?  when doing any other query also?  Is that safer because we really don't edit these in place (ie, immutability, or vals not vars)?
  def getSize(includeWhichEntities: Int = 3): Long = {
    mDB.getGroupSize(mId, includeWhichEntities)
  }

  def getDisplayString(lengthLimitIn: Int = 0, simplifyIn: Boolean = false): String = {
    val numEntries = mDB.getGroupSize(getId, 1)
    var result: String =  ""
    result += {
      if (simplifyIn) getName
      else "grp " + mId + " /" + numEntries + ": " + Color.blue(getName)
    }
    if (!simplifyIn) {
      result += ", class: "
      val className =
        if (getMixedClassesAllowed)
          "(mixed)"
        else {
          val classNameOption = getClassName
          if (classNameOption.isEmpty) "None"
          else classNameOption.get
        }
      result += className
    }
    if (simplifyIn) result
    else Attribute.limitDescriptionLength(result, lengthLimitIn)
  }

  def getGroupEntries(startingIndexIn: Long, maxValsIn: Option[Long] = None): java.util.ArrayList[Entity] = {
    mDB.getGroupEntryObjects(mId, startingIndexIn, maxValsIn)
  }

  def addEntity(inEntityId: Long) {
    mDB.addEntityToGroup(mId, inEntityId)
  }

  def getId: Long = mId

  def getName: String = {
    if (!mAlreadyReadData) readDataFromDB()
    mName
  }

  def getMixedClassesAllowed: Boolean = {
    if (!mAlreadyReadData) readDataFromDB()
    mMixedClassesAllowed
  }

  def getNewEntriesStickToTop: Boolean = {
    if (!mAlreadyReadData) readDataFromDB()
    mNewEntriesStickToTop
  }

  def getInsertionDate: Long = {
    if (!mAlreadyReadData) readDataFromDB()
    mInsertionDate
  }

  def getClassName: Option[String] = {
    if (getMixedClassesAllowed)
      None
    else {
      val classId: Option[Long] = getClassId
      if (classId.isEmpty && getSize() == 0) {
        // display should indicate that we know mixed are not allowed, so a class could be specified, but none has.
        Some("(unspecified)")
      } else if (classId.isEmpty) {
        // means the group requires uniform classes, but the enforced uniform class is None, i.e., to not have a class:
        Some("(specified as None)")
      } else {
        val exampleEntitysClass = new EntityClass(mDB, classId.get)
        Some(exampleEntitysClass.getName)
      }
    }
  }

  def getClassId: Option[Long] = {
    if (getMixedClassesAllowed)
      None
    else {
      val entries = mDB.getGroupEntryObjects(getId, 0, Some(1))
      val specified: Boolean = entries.size() > 0
      if (!specified)
        None
      else {
        // idea: eliminate/simplify most of this part, since groups can't have subgroups only entities in them now?
        def findAnEntity(nextIndex: Int): Option[Entity] = {
          // We will have to change this (and probably other things) to traverse "subgroups" (groups in the entities in this group) also,
          // if we decide that disallowing mixed classes also means class uniformity across all subgroups.
          if (nextIndex == entries.size)
            None
          else entries.get(nextIndex) match {
            case entity: Entity =>
              Some(entity)
            case _ =>
              val className = entries.get(nextIndex).getClass.getName
              throw new OmException(s"a group contained an entry that's not an entity?  Thought had eliminated use of 'subgroups' except via entities. It's " +
                                    s"of type: $className")
          }
        }
        val entity: Option[Entity] = findAnEntity(0)
        if (entity.isDefined)
          entity.get.getClassId
        else
          None
      }
    }
  }

  def getClassTemplateEntity: (Option[Entity]) = {
    val classId: Option[Long] = getClassId
    if (getMixedClassesAllowed || classId.isEmpty)
      None
    else {
      val templateEntityId = new EntityClass(mDB, classId.get).getTemplateEntityId
      Some(new Entity(mDB, templateEntityId))
    }
  }

  def getHighestSortingIndex: Long = {
    mDB.getHighestSortingIndexForGroup(getId)
  }

  private var mAlreadyReadData: Boolean = false
  private var mName: String = null
  private var mInsertionDate: Long = 0L
  private var mMixedClassesAllowed: Boolean = false
  private var mNewEntriesStickToTop: Boolean = false
}
