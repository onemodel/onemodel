/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2014-2015 inclusive, Luke A. Call; all rights reserved.
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

import org.onemodel.{Color, OmException}
import org.onemodel.database.PostgreSQLDatabase

/** See comments on similar methods in RelationToEntity. */
class Group(mDB: PostgreSQLDatabase, mId: Long) {
  if (!mDB.groupKeyExists(mId: Long)) {
    // DON'T CHANGE this msg unless you also change the trap for it, if used, in other code. (should be a constant then, huh? same elsewhere. It's on the list.)
    throw new Exception("Key " + mId + " does not exist in database.")
  }

  /** See comment about these 2 dates in PostgreSQLDatabase.createTables() */
  def this(mDB: PostgreSQLDatabase, idIn: Long, nameIn: String, insertionDateIn: Long, mixedClassesAllowedIn: Boolean) {
    this(mDB, idIn)
    mName = nameIn
    mInsertionDate = insertionDateIn
    mMixedClassesAllowed = mixedClassesAllowedIn
  }

  def readDataFromDB() {
    val relationData: Array[Option[Any]] = mDB.getGroupData(mId)
    mName = relationData(0).get.asInstanceOf[String]
    mInsertionDate = relationData(1).get.asInstanceOf[Long]
    mMixedClassesAllowed = relationData(2).get.asInstanceOf[Boolean]
    mAlreadyReadData = true
  }

  def update(attrTypeIdIn: Option[Long] = None, nameIn: Option[String] = None, allowMixedClassesInGroupIn: Option[Boolean] = None,
             validOnDateInIGNORED4NOW: Option[Long], observationDateInIGNORED4NOW: Option[Long]) {
    mDB.updateGroup(mId,

                    if (nameIn.isEmpty) getName
                    else nameIn.get,

                    if (allowMixedClassesInGroupIn.isEmpty) getMixedClassesAllowed
                    else allowMixedClassesInGroupIn.get)
  }

  /** Removes this object from the system. */
  def delete() = mDB.deleteGroupAndRelationsToIt(mId)

  /** Removes an entity from this group. */
  def removeEntity(entityId: Long) = mDB.removeEntityFromGroup(mId, entityId)

  def deleteWithEntities() = mDB.deleteGroupRelationsToItAndItsEntries(mId)

  // idea: cache this?  when doing any other query also?  Is that safer because we really don't edit these in place (ie, immutability, or vals not vars)?
  def getSize: Long = mDB.getGroupSize(mId)

  def getDisplayString(lengthLimitIn: Int, simplifyIn: Boolean = false): String = {
    val numEntries = mDB.getGroupSize(getId, Some(false))
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

  def getInsertionDate: Long = {
    if (!mAlreadyReadData) readDataFromDB()
    mInsertionDate
  }

  def getClassName: Option[String] = {
    if (getMixedClassesAllowed)
      None
    else {
      val classId: Option[Long] = getClassId
      if (classId.isEmpty && getSize == 0) {
        // display should indicate that we know mixed are not allowed, so a class could be specified, but none has.
        Some("(unspecified)")
      }
      // means the group requires uniform classes, but the enforced uniform class is None:
      else if (classId.isEmpty)
             Some("(specified as None)")
      else {
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
    mDB.getHighestSortingIndex(getId)
  }

  private var mAlreadyReadData: Boolean = false
  private var mName: String = null
  private var mInsertionDate: Long = 0L
  private var mMixedClassesAllowed: Boolean = false
}
