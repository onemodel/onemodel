/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2013-2016 inclusive, Luke A. Call; all rights reserved.
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

import org.onemodel.database.PostgreSQLDatabase

object RelationToGroup {
  // Old idea: could change this into a constructor if the "class" line's parameters are changed to be only mDB and mId, and a new constructor is created
  // to fill in the other fields. But didn't do that because it would require an extra db read with every use, and the ordering of statements in the
  // new constructors just wasn't working out.
  // Idea: rename this to instantiateRelationToGroup, since create sounds like inserting a new row in the db. Not sure if there's a convention for that case.
  def createRelationToGroup(mDB: PostgreSQLDatabase, idIn: Long): RelationToGroup = {
    val relationData: Array[Option[Any]] = mDB.getRelationToGroupDataById(idIn)
    new RelationToGroup(mDB, idIn, relationData(1).get.asInstanceOf[Long], relationData(2).get.asInstanceOf[Long], relationData(3).get.asInstanceOf[Long],
                     relationData(4).asInstanceOf[Option[Long]], relationData(5).get.asInstanceOf[Long], relationData(6).get.asInstanceOf[Long])
  }
}

/** See comments on similar methods in RelationToEntity. */
class RelationToGroup(mDB: PostgreSQLDatabase, mId: Long, mEntityId:Long, mRelTypeId: Long, mGroupId:Long) extends AttributeWithValidAndObservedDates(mDB, mId) {
  if (mDB.relationToGroupKeysExistAndMatch(mId, mEntityId, mRelTypeId, mGroupId)) {
    // something else might be cleaner, but these are the same thing and we need to make sure the superclass' var doesn't overwrite this w/ 0:
    mAttrTypeId = mRelTypeId
  } else {
    // DON'T CHANGE this msg unless you also change the trap for it, if used, in other code. (should be a constant then, huh? same elsewhere. It's on the list.)
    throw new Exception("Keys id=" + mId + ", with multi-column key composed of:  " + mEntityId + "/" + mRelTypeId + "/" + mGroupId + " do not exist in database.")
  }

  /** See comment about these 2 dates in PostgreSQLDatabase.createTables() */
  def this(mDB: PostgreSQLDatabase, idIn: Long, entityIdIn: Long, relTypeIdIn: Long, groupIdIn: Long, inValidOnDate: Option[Long], inObservationDate: Long,
           sortingIndexIn: Long) {
    this(mDB, idIn, entityIdIn, relTypeIdIn, groupIdIn)
    assignCommonVars(entityIdIn, relTypeIdIn, inValidOnDate, inObservationDate, sortingIndexIn)
  }

  def getGroupId: Long = mGroupId

  def getDisplayString(lengthLimitIn: Int, unused: Option[Entity] = None, ignoredParameter: Option[RelationType] = None, simplify: Boolean = false): String = {
    val group = new Group(mDB, mGroupId)
    val rtName = new RelationType(mDB, this.getAttrTypeId).getName
    var result: String = if (simplify && rtName == PostgreSQLDatabase.theHASrelationTypeName) "" else rtName + " "
    result += group.getDisplayString(0, simplify)
    if (! simplify) result += "; " + getDatesDescription
    Attribute.limitDescriptionLength(result, lengthLimitIn)
  }

  protected def readDataFromDB() {
    val relationData: Array[Option[Any]] = mDB.getRelationToGroupData(mEntityId, mRelTypeId, mGroupId)
    super.assignCommonVars(mEntityId, mRelTypeId,
                           relationData(4).asInstanceOf[Option[Long]],
                           relationData(5).get.asInstanceOf[Long], relationData(6).get.asInstanceOf[Long])
  }

  def update(validOnDateIn:Option[Long], observationDateIn:Option[Long]) {
    //use validOnDateIn rather than validOnDateIn.get because validOnDate allows None, unlike others
    //Idea/possible bug: see comment on similar method in RelationToEntity.
    val vod = if (validOnDateIn.isDefined) validOnDateIn else getValidOnDate
    val od = if (observationDateIn.isDefined) observationDateIn.get else getObservationDate
    mDB.updateRelationToGroup(mEntityId, mRelTypeId, mGroupId, vod, od)
    mValidOnDate = vod
    mObservationDate = od
  }

  /** Removes this object from the system. */
  def delete() = mDB.deleteRelationToGroup(mEntityId, mRelTypeId, mGroupId)

  /** Removes this object from the system. */
  def deleteGroupAndRelationsToIt() = mDB.deleteGroupAndRelationsToIt(mGroupId)
}
