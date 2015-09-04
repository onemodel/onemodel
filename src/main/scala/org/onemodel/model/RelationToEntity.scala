/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2004, 2010, 2011, and 2013-2015 inclusive, Luke A. Call; all rights reserved.
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

/**
 * Represents one RelationToEntity object in the system (usually [always, as of 9/2003] used as an attribute on a Entity).
 *
 * This 1st constructor instantiates an existing object from the DB. You can use Entity.addRelationToEntity() to
 * create a new object. Assumes caller just read it from the DB and the info is accurate (i.e., this may only ever need to be called by
 * a PostgreSQLDatabase instance?).
 *
 * Passes 0 as 2nd parameter to AttributeWithValidAndObservedDates, because it's a value that in this case really doesn't make sense for this type.
 * Idea: is that a bad smell? shouldn't inherit, then?
 */
class RelationToEntity(mDB: PostgreSQLDatabase, mRelTypeId: Long, mEntityId1: Long, mEntityId2: Long) extends AttributeWithValidAndObservedDates(mDB, 0) {
  if (mDB.relationToEntityKeyExists(mRelTypeId, mEntityId1, mEntityId2)) {
    // something else might be cleaner, but these are the same thing and we need to make sure the superclass' var doesn't overwrite this w/ 0:
    mAttrTypeId = mRelTypeId
  } else {
    // DON'T CHANGE this msg unless you also change the trap for it, if used, in other code. (should be a constant then, huh? same elsewhere. It's on the list.)
    throw new Exception("Key rel_type_id=" + mRelTypeId + " and entity_id_1=" + mEntityId1 + " and entity_id_2=" + mEntityId2 + " does not exist in " +
                        "database.")
  }

  /**
   * This  one is perhaps only called by the database class implementation--so it can return arrays of objects & save more DB hits
   * that would have to occur if it only returned arrays of keys. This DOES NOT create a persistent object--but rather should reflect
   * one that already exists.
   */
  def this(mDB: PostgreSQLDatabase, inRelTypeId: Long, inEntityId1: Long, inEntityId2: Long, inValidOnDate: Option[Long], inObservationDate: Long) {
    this(mDB, inRelTypeId, inEntityId1, inEntityId2)
    // (The inEntityId1 really doesn't fit here, because it's part of the class' primary key. But passing it here for the convenience of using
    // the class hierarchy which wants it. Improve...?)
    assignCommonVars(inEntityId1, inRelTypeId, inValidOnDate, inObservationDate)
  }

  override def getId: Long = throw new UnsupportedOperationException("getId() operation not applicable to Relation class.")
  override def getParentId: Long = throw new UnsupportedOperationException("getParentId() operation not applicable to Relation class.")
  def getRelatedId1: Long = mEntityId1
  def getRelatedId2: Long = mEntityId2

  /**
   * return something like "son of: Paul" or "owns: Ford truck" or "employed by: hospital". If inLengthLimit is 0 you get the whole thing.
   * The 2nd parameter, inRelatedEntity, is not the entity from whose perspective the result will be returned, e.g.,
   * 'x contains y' or 'y is contained by x': the 2nd parameter should be the *2nd* one in that statement.
   */
  def getDisplayString(lengthLimitIn: Int, inRelatedEntity: Option[Entity], inRT: Option[RelationType], simplify: Boolean = false): String = {
    require(inRelatedEntity.isDefined && inRT.isDefined)
    if (inRT.get.getId != this.getAttrTypeId) {
      throw new Exception("inRT parameter should be the same as the relationType on this relation.")
    }
    val rtName: String =
      if (inRelatedEntity.get.getId == mEntityId2)  inRT.get.getName
      else if (inRelatedEntity.get.getId == mEntityId1) inRT.get.getNameInReverseDirection
      else throw new Exception("Unrelated parent entity parameter?: '" + inRelatedEntity.get.getId + "', '" + inRelatedEntity.get.getName + "'")

    var result: String = if (simplify) {
      if (rtName == PostgreSQLDatabase.theHASrelationTypeName) inRelatedEntity.get.getName
      else rtName + ": " + inRelatedEntity.get.getName
    } else {
      rtName + ": " + Color.blue(inRelatedEntity.get.getName) + "; " + getDatesDescription
    }
    Attribute.limitDescriptionLength(result, lengthLimitIn)
  }

  protected def readDataFromDB() {
    val relationData: Array[Option[Any]] = mDB.getRelationToEntityData(mAttrTypeId, mEntityId1, mEntityId2)
    // No other local variables to assign.  All are either in the superclass or the primary key.
    // (The inEntityId1 really doesn't fit here, because it's part of the class' primary key. But passing it here for the convenience of using
    // the class hierarchy which wants it. Improve...?)
    super.assignCommonVars(mEntityId1, mAttrTypeId,
                           relationData(0).asInstanceOf[Option[Long]],
                           relationData(1).get.asInstanceOf[Long])
  }

  def update(attrTypeIdIn: Option[Long] = None, validOnDateIn:Option[Long], observationDateIn:Option[Long]) {
    mDB.updateRelationToEntity(if (attrTypeIdIn.isEmpty) getAttrTypeId else attrTypeIdIn.get,
                               mEntityId1, mEntityId2,
                               //pass validOnDateIn rather than validOnDateIn.get because validOnDate allows None, unlike others
                               if (validOnDateIn.isEmpty) getValidOnDate else validOnDateIn,
                               if (observationDateIn.isEmpty) getObservationDate else observationDateIn.get)
  }

  /** Removes this object from the system. */
  def delete() = mDB.deleteRelationToEntity(getAttrTypeId, mEntityId1, mEntityId2)

}