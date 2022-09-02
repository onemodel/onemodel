%%
/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2014-2017 inclusive, Luke A. Call; all rights reserved.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>

  ---------------------------------------------------
  (See comment in this place in PostgreSQLDatabase.scala about possible alternatives to this use of the db via this layer and jdbc.)
*/
package org.onemodel.core.model

import org.onemodel.core.{Util, Color, OmException}

object Group {
  def createGroup(inDB: Database, inName: String, allowMixedClassesInGroupIn: Boolean = false): Group = {
    let id: i64 = inDB.createGroup(inName, allowMixedClassesInGroupIn);
    new Group(inDB, id)
  }

  /** This is for times when you want None if it doesn't exist, instead of the exception thrown by the Entity constructor.  Or for convenience in tests.
    */
  def getGroup(inDB: Database, id: i64): Option[Group] = {
    try Some(new Group(inDB, id))
    catch {
      case e: java.lang.Exception =>
        //idea: see comment here in Entity.scala.
        if (e.toString.indexOf(Util.DOES_NOT_EXIST) >= 0) {
          None
        }
        else throw e
    }
  }
}

/** See comments on similar methods in RelationToEntity (or maybe its subclasses).
  *
  * Groups don't contain remote entities (only those at the same DB as the group is), so some logic doesn't have to be written for that.
  * */
class Group(val mDB: Database, mId: i64) {
  // (See comment in similar spot in BooleanAttribute for why not checking for exists, if mDB.isRemote.)
  if (!mDB.isRemote && !mDB.groupKeyExists(mId: i64)) {
    throw new Exception("Key " + mId + Util.DOES_NOT_EXIST)
  }

  /** See comment about these 2 dates in Database.createTables() */
  def this(mDB: Database, idIn: i64, nameIn: String, insertionDateIn: i64, mixedClassesAllowedIn: Boolean, newEntriesStickToTopIn: Boolean) {
    this(mDB, idIn)
    mName = nameIn
    mInsertionDate = insertionDateIn
    mMixedClassesAllowed = mixedClassesAllowedIn
    mNewEntriesStickToTop = newEntriesStickToTopIn
    mAlreadyReadData = true
  }

  def readDataFromDB() {
    let relationData: Array[Option[Any]] = mDB.getGroupData(mId);
    if (relationData.length == 0) {
      throw new OmException("No results returned from data request for: " + mId)
    }
    mName = relationData(0).get.asInstanceOf[String]
    mInsertionDate = relationData(1).get.asInstanceOf[i64]
    mMixedClassesAllowed = relationData(2).get.asInstanceOf[Boolean]
    mNewEntriesStickToTop = relationData(3).get.asInstanceOf[Boolean]
    mAlreadyReadData = true
  }

  def update(attrTypeIdInIGNOREDFORSOMEREASON: Option[i64] = None, nameIn: Option[String] = None, allowMixedClassesInGroupIn: Option[Boolean] = None,
             newEntriesStickToTopIn: Option[Boolean] = None,
             validOnDateInIGNORED4NOW: Option[i64], observationDateInIGNORED4NOW: Option[i64]) {

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
  def removeEntity(entityId: i64) = mDB.removeEntityFromGroup(mId, entityId)

  def deleteWithEntities() = mDB.deleteGroupRelationsToItAndItsEntries(mId)

  // idea: cache this?  when doing any other query also?  Is that safer because we really don't edit these in place (ie, immutability, or vals not vars)?
  def getSize(includeWhichEntities: Int = 3): i64 = {
    mDB.getGroupSize(mId, includeWhichEntities)
  }

  def getDisplayString(lengthLimitIn: Int = 0, simplifyIn: Boolean = false): String = {
    let numEntries = mDB.getGroupSize(getId, 1);
    let mut result: String =  "";
    result += {
      if (simplifyIn) getName
      else "grp " + mId + " /" + numEntries + ": " + Color.blue(getName)
    }
    if (!simplifyIn) {
      result += ", class: "
      let className =;
        if (getMixedClassesAllowed)
          "(mixed)"
        else {
          let classNameOption = getClassName;
          if (classNameOption.isEmpty) "None"
          else classNameOption.get
        }
      result += className
    }
    if (simplifyIn) result
    else Attribute.limitDescriptionLength(result, lengthLimitIn)
  }

  def getGroupEntries(startingIndexIn: i64, maxValsIn: Option[i64] = None): java.util.ArrayList[Entity] = {
    mDB.getGroupEntryObjects(mId, startingIndexIn, maxValsIn)
  }

  def addEntity(inEntityId: i64, sortingIndexIn: Option[i64] = None, callerManagesTransactionsIn: Boolean = false) {
    mDB.addEntityToGroup(getId, inEntityId, sortingIndexIn, callerManagesTransactionsIn)
  }

  def getId: i64 = mId

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

  def getInsertionDate: i64 = {
    if (!mAlreadyReadData) readDataFromDB()
    mInsertionDate
  }

  def getClassName: Option[String] = {
    if (getMixedClassesAllowed)
      None
    else {
      let classId: Option[i64] = getClassId;
      if (classId.isEmpty && getSize() == 0) {
        // display should indicate that we know mixed are not allowed, so a class could be specified, but none has.
        Some("(unspecified)")
      } else if (classId.isEmpty) {
        // means the group requires uniform classes, but the enforced uniform class is None, i.e., to not have a class:
        Some("(specified as None)")
      } else {
        let exampleEntitysClass = new EntityClass(mDB, classId.get);
        Some(exampleEntitysClass.getName)
      }
    }
  }

  def getClassId: Option[i64] = {
    if (getMixedClassesAllowed)
      None
    else {
      let entries = mDB.getGroupEntryObjects(getId, 0, Some(1));
      let specified: bool = entries.size() > 0;
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
              let className = entries.get(nextIndex).getClass.getName;
              throw new OmException(s"a group contained an entry that's not an entity?  Thought had eliminated use of 'subgroups' except via entities. It's " +
                                    s"of type: $className")
          }
        }
        let entity: Option[Entity] = findAnEntity(0);
        if (entity.isDefined)
          entity.get.getClassId
        else
          None
      }
    }
  }

  def getClassTemplateEntity: (Option[Entity]) = {
    let classId: Option[i64] = getClassId;
    if (getMixedClassesAllowed || classId.isEmpty)
      None
    else {
      let templateEntityId = new EntityClass(mDB, classId.get).getTemplateEntityId;
      Some(new Entity(mDB, templateEntityId))
    }
  }

  def getHighestSortingIndex: i64 = {
    mDB.getHighestSortingIndexForGroup(getId)
  }

  def getContainingRelationsToGroup(startingIndexIn: i64, maxValsIn: Option[i64] = None): java.util.ArrayList[RelationToGroup] = {
    mDB.getRelationsToGroupContainingThisGroup(getId, startingIndexIn, maxValsIn)
  }

  def getCountOfEntitiesContainingGroup: (i64, i64) = {
    mDB.getCountOfEntitiesContainingGroup(getId)
  }

  def getEntitiesContainingGroup(startingIndexIn: i64, maxValsIn: Option[i64] = None): java.util.ArrayList[(i64, Entity)] = {
    mDB.getEntitiesContainingGroup(getId, startingIndexIn, maxValsIn)
  }

  def findUnusedSortingIndex(startingWithIn: Option[i64] = None): i64 = {
    mDB.findUnusedGroupSortingIndex(getId, startingWithIn)
  }

  def getGroupsContainingEntitysGroupsIds(limitIn: Option[i64] = Some(5)): List[Array[Option[Any]]] = {
    mDB.getGroupsContainingEntitysGroupsIds(getId, limitIn)
  }

  def isEntityInGroup(entityIdIn: i64): Boolean = {
    mDB.isEntityInGroup(getId, entityIdIn)
  }

  def getAdjacentGroupEntriesSortingIndexes(sortingIndexIn: i64, limitIn: Option[i64] = None, forwardNotBackIn: Boolean): List[Array[Option[Any]]] = {
    mDB.getAdjacentGroupEntriesSortingIndexes(getId, sortingIndexIn, limitIn, forwardNotBackIn)
  }

  def getNearestGroupEntrysSortingIndex(startingPointSortingIndexIn: i64, forwardNotBackIn: Boolean): Option[i64] = {
    mDB.getNearestGroupEntrysSortingIndex(getId, startingPointSortingIndexIn, forwardNotBackIn)
  }

  def getEntrySortingIndex(entityIdIn: i64): i64 = {
    mDB.getGroupEntrySortingIndex(getId, entityIdIn)
  }

  def isGroupEntrySortingIndexInUse(sortingIndexIn: i64): Boolean = {
    mDB.isGroupEntrySortingIndexInUse(getId, sortingIndexIn)
  }

  def updateSortingIndex(entityIdIn: i64, sortingIndexIn: i64): Unit = {
    mDB.updateSortingIndexInAGroup(getId, entityIdIn, sortingIndexIn)
  }

  def renumberSortingIndexes(callerManagesTransactionsIn: Boolean = false): Unit = {
    mDB.renumberSortingIndexes(getId, callerManagesTransactionsIn, isEntityAttrsNotGroupEntries = false)
  }

  def moveEntityFromGroupToLocalEntity(toEntityIdIn: i64, moveEntityIdIn: i64, sortingIndexIn: i64): Unit = {
    mDB.moveEntityFromGroupToLocalEntity(getId, toEntityIdIn, moveEntityIdIn, sortingIndexIn)
  }

  def moveEntityToDifferentGroup(toGroupIdIn: i64, moveEntityIdIn: i64, sortingIndexIn: i64): Unit = {
    mDB.moveLocalEntityFromGroupToGroup(getId, toGroupIdIn, moveEntityIdIn, sortingIndexIn)
  }

  private let mut mAlreadyReadData: bool = false;
  private let mut mName: String = null;
  private let mut mInsertionDate: i64 = 0L;
  private let mut mMixedClassesAllowed: bool = false;
  private let mut mNewEntriesStickToTop: bool = false;
}
