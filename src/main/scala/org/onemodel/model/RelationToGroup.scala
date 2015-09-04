/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2013-2015 inclusive, Luke A. Call; all rights reserved.
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

import org.onemodel.Color
import org.onemodel.database.PostgreSQLDatabase

/** See comments on similar methods in RelationToEntity. */
class RelationToGroup(mDB: PostgreSQLDatabase, mEntityId:Long, mRelTypeId: Long, mGroupId:Long) extends AttributeWithValidAndObservedDates(mDB, 0) {
  if (mDB.relationToGroupKeyExists(mEntityId, mRelTypeId, mGroupId)) {
    // something else might be cleaner, but these are the same thing and we need to make sure the superclass' var doesn't overwrite this w/ 0:
    mAttrTypeId = mRelTypeId
  } else {
    // DON'T CHANGE this msg unless you also change the trap for it, if used, in other code. (should be a constant then, huh? same elsewhere. It's on the list.)
    throw new Exception("Key " + mEntityId + "/" + mRelTypeId + "/" + mGroupId + " does not exist in database.")
  }

  /** See comment about these 2 dates in PostgreSQLDatabase.createTables() */
  def this(mDB: PostgreSQLDatabase, entityIdIn: Long, relTypeIdIn: Long, groupIdIn: Long, inValidOnDate: Option[Long], inObservationDate: Long) {
    this(mDB, entityIdIn, relTypeIdIn, groupIdIn)
    assignCommonVars(entityIdIn, relTypeIdIn, inValidOnDate, inObservationDate)
  }

  override def getId: Long = throw new UnsupportedOperationException("getId() operation not applicable to RelationToGroup class.")

  def getGroupId: Long = mGroupId

  def getDisplayString(lengthLimitIn: Int, unused: Option[Entity] = None, ignoredParameter: Option[RelationType] = None, simplify: Boolean = false): String = {
    val group = new Group(mDB, mGroupId)
    val rtName = new RelationType(mDB, this.getAttrTypeId).getName
    var result: String = if (simplify && rtName == PostgreSQLDatabase.theHASrelationTypeName) "" else rtName + " "
    result += group.getDisplayString(0)
    if (! simplify) result += "; " + getDatesDescription
    Attribute.limitDescriptionLength(result, lengthLimitIn)
  }

  protected def readDataFromDB() {
    val relationData: Array[Option[Any]] = mDB.getRelationToGroupData(mEntityId, mRelTypeId, mGroupId)
    super.assignCommonVars(relationData(0).get.asInstanceOf[Long], relationData(1).get.asInstanceOf[Long],
                           relationData(3).asInstanceOf[Option[Long]],
                           relationData(4).get.asInstanceOf[Long])
  }

  def update(validOnDateIn:Option[Long], observationDateIn:Option[Long]) {
    mDB.updateRelationToGroup(mEntityId, mRelTypeId, mGroupId,
                              //pass validOnDateIn rather than validOnDateIn.get because validOnDate allows None, unlike others
                              if (validOnDateIn.isEmpty) getValidOnDate else validOnDateIn,
                              if (observationDateIn.isEmpty) getObservationDate else observationDateIn.get)
  }

  /** Removes this object from the system. */
  def delete() = mDB.deleteRelationToGroup(mEntityId, mRelTypeId, mGroupId)

  /** Removes this object from the system. */
  def deleteGroupAndRelationsToIt() = mDB.deleteGroupAndRelationsToIt(mGroupId)
}
