/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2013-2017 inclusive, Luke A. Call; all rights reserved.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule, guidelines around binary
    distribution, and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>

  ---------------------------------------------------
  If we ever do port to another database, create the Database interface (removed around 2014-1-1 give or take) and see other changes at that time.
  An alternative method is to use jdbc escapes (but this actually might be even more work?):  http://jdbc.postgresql.org/documentation/head/escapes.html  .
  Another alternative is a layer like JPA, ibatis, hibernate  etc etc.

*/
package org.onemodel.core.model

import org.onemodel.core.{OmException, Util}

object RelationToGroup {
  // Old idea: could change this into a constructor if the "class" line's parameters are changed to be only mDB and mId, and a new constructor is created
  // to fill in the other fields. But didn't do that because it would require an extra db read with every use, and the ordering of statements in the
  // new constructors just wasn't working out.
  // Idea: rename this to instantiateRelationToGroup, since create sounds like inserting a new row in the db. Not sure if there's a convention for that case.
  def createRelationToGroup(mDB: Database, idIn: Long): RelationToGroup = {
    val relationData: Array[Option[Any]] = mDB.getRelationToGroupData(idIn)
    if (relationData.length == 0) {
      throw new OmException("No results returned from data request for: " + idIn)
    }
    new RelationToGroup(mDB, idIn, relationData(1).get.asInstanceOf[Long], relationData(2).get.asInstanceOf[Long], relationData(3).get.asInstanceOf[Long],
                     relationData(4).asInstanceOf[Option[Long]], relationData(5).get.asInstanceOf[Long], relationData(6).get.asInstanceOf[Long])
  }
}

/** See comments on similar methods in RelationToEntity (or maybe its subclasses). */
class RelationToGroup(mDB: Database, mId: Long, mEntityId:Long, mRelTypeId: Long, mGroupId: Long) extends AttributeWithValidAndObservedDates(mDB, mId) {
  // (See comment in similar spot in BooleanAttribute for why not checking for exists, if mDB.isRemote.)
  if (mDB.isRemote || mDB.relationToGroupKeysExistAndMatch(mId, mEntityId, mRelTypeId, mGroupId)) {
    // something else might be cleaner, but these are the same thing and we need to make sure the superclass' var doesn't overwrite this w/ 0:
    mAttrTypeId = mRelTypeId
  } else {
    throw new Exception("Key id=" + mId + ", " + mEntityId + "/" + mRelTypeId + "/" + mGroupId + Util.DOES_NOT_EXIST)
  }

  /** See comment about these 2 dates in PostgreSQLDatabase.createTables() */
  def this(mDB: Database, idIn: Long, entityIdIn: Long, relTypeIdIn: Long, groupIdIn: Long, validOnDateIn: Option[Long], observationDateIn: Long,
           sortingIndexIn: Long) {
    this(mDB, idIn, entityIdIn, relTypeIdIn, groupIdIn)
    assignCommonVars(entityIdIn, relTypeIdIn, validOnDateIn, observationDateIn, sortingIndexIn)
  }

  def getGroupId: Long = mGroupId

  def getGroup: Group = {
    new Group(mDB, getGroupId)
  }

  def getDisplayString(lengthLimitIn: Int, unused: Option[Entity] = None, ignoredParameter: Option[RelationType] = None, simplify: Boolean = false): String = {
    val group = new Group(mDB, mGroupId)
    val rtName = new RelationType(mDB, this.getAttrTypeId).getName
    var result: String = if (simplify && rtName == Database.theHASrelationTypeName) "" else rtName + " "
    result += group.getDisplayString(0, simplify)
    if (! simplify) result += "; " + getDatesDescription
    Attribute.limitDescriptionLength(result, lengthLimitIn)
  }

  protected def readDataFromDB() {
    val relationData: Array[Option[Any]] = mDB.getRelationToGroupDataByKeys(mEntityId, mRelTypeId, mGroupId)
    if (relationData.length == 0) {
      throw new OmException("No results returned from data request for: " + mEntityId + ", " + mRelTypeId + ", " + mGroupId)
    }
    super.assignCommonVars(mEntityId, mRelTypeId,
                           relationData(4).asInstanceOf[Option[Long]],
                           relationData(5).get.asInstanceOf[Long], relationData(6).get.asInstanceOf[Long])
  }

  def move(newContainingEntityIdIn: Long, sortingIndexIn: Long): Long = {
    mDB.moveRelationToGroup(getId, newContainingEntityIdIn, sortingIndexIn)
  }

  def update(newRelationTypeIdIn: Option[Long], newGroupIdIn: Option[Long], validOnDateIn:Option[Long], observationDateIn:Option[Long]) {
    //use validOnDateIn rather than validOnDateIn.get because validOnDate allows None, unlike others
    //Idea/possible bug: see comment on similar method in RelationToEntity (or maybe in its subclasses).
    val newRelationTypeId: Long = if (newRelationTypeIdIn.isDefined) newRelationTypeIdIn.get else getAttrTypeId
    val newGroupId: Long = if (newGroupIdIn.isDefined) newGroupIdIn.get else getGroupId
    val vod = if (validOnDateIn.isDefined) validOnDateIn else getValidOnDate
    val od = if (observationDateIn.isDefined) observationDateIn.get else getObservationDate
    mDB.updateRelationToGroup(mEntityId, mRelTypeId, newRelationTypeId, mGroupId, newGroupId, vod, od)
    mValidOnDate = vod
    mObservationDate = od
  }

  /** Removes this object from the system. */
  def delete() = mDB.deleteRelationToGroup(mEntityId, mRelTypeId, mGroupId)

  /** Removes this object from the system. */
  def deleteGroupAndRelationsToIt() = mDB.deleteGroupAndRelationsToIt(mGroupId)
}
