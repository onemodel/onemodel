/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2004, 2010, 2011, and 2013-2016 inclusive, Luke A. Call; all rights reserved.
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

import org.onemodel.core.{OmException, Util, Color}
import org.onemodel.core.database.Database

/**
 * Represents one RelationToEntity object in the system (usually [always, as of 9/2003] used as an attribute on a Entity).
 *
 * This 1st constructor instantiates an existing object from the DB. You can use Entity.addRelationToEntity() to
 * create a new object. Assumes caller just read it from the DB and the info is accurate (i.e., this may only ever need to be called by
 * a PostgreSQLDatabase instance?).
 *
 */
class RelationToEntity(mDB: Database, mId: Long, mRelTypeId: Long, mEntityId1: Long, mEntityId2: Long) extends AttributeWithValidAndObservedDates(mDB, mId) {
  // (See comment at similar location in BooleanAttribute.)

  if (! this.isInstanceOf[RelationToRemoteEntity]) {
    // This is using inheritance as a way to share code, but they do not "inherit" inside the PostgreSQLDatabase:
    if (mDB.isRemote || mDB.relationToEntityKeysExistAndMatch(mId, mRelTypeId, mEntityId1, mEntityId2)) {
      // something else might be cleaner, but these are the same thing and we need to make sure the superclass' var doesn't overwrite this w/ 0:
      mAttrTypeId = mRelTypeId
    } else {
      throw new OmException("Key id=" + mId + ", rel_type_id=" + mRelTypeId + " and entity_id=" + mEntityId1 +
                                " and entity_id_2=" + mEntityId2 + Util.DOES_NOT_EXIST)
    }
  }

  /**
   * This  one is perhaps only called by the database class implementation--so it can return arrays of objects & save more DB hits
   * that would have to occur if it only returned arrays of keys. This DOES NOT create a persistent object--but rather should reflect
   * one that already exists.
   */
  def this(mDB: Database, idIn: Long, relTypeIdIn: Long, entityId1In: Long, entityId2In: Long, validOnDateIn: Option[Long], observationDateIn: Long,
           sortingIndexIn: Long) {
    this(mDB, idIn, relTypeIdIn, entityId1In, entityId2In)
    if (this.isInstanceOf[RelationToRemoteEntity]) {
      // idea: this test & exception feel awkward. What is the better approach?  Maybe using scala's type features?
      throw new OmException("This constructor should not be called by the subclass.")
    }
    // (The inEntityId1 really doesn't fit here, because it's part of the class' primary key. But passing it here for the convenience of using
    // the class hierarchy which wants it. Improve...?)
    assignCommonVars(entityId1In, relTypeIdIn, validOnDateIn, observationDateIn, sortingIndexIn)
  }

  // (the next line used to return "throw new UnsupportedOperationException("getParentId() operation not applicable to Relation class.")", and I'm not
  // sure the reason: if it was just to prevent accidental misuse or confusion, it seems OK to have it be like this instead:
  override def getParentId: Long = getRelatedId1

  def getRelatedId1: Long = mEntityId1
  def getRelatedId2: Long = mEntityId2

  /**
   * @param relatedEntityIn, is not the entity from whose perspective the result will be returned, e.g.,
   * 'x contains y' OR 'y is contained by x': the 2nd parameter should be the *2nd* one in that statement.
   * If left None here, the code will make a guess but might output confusing (backwards) info.
   *
   * @param relationTypeIn can be left None, but will run faster if not.
   *
   * @return something like "son of: Paul" or "owns: Ford truck" or "employed by: hospital". If inLengthLimit is 0 you get the whole thing.
   */
  def getDisplayString(lengthLimitIn: Int, relatedEntityIn: Option[Entity], relationTypeIn: Option[RelationType], simplify: Boolean = false): String = {
    val relType: RelationType = {
      if (relationTypeIn.isDefined) {
        if (relationTypeIn.get.getId != getAttrTypeId) {
          // It can be ignored, but in cases called generically (the same as other Attribute types) it should have the right value or that indicates a
          // misunderstanding in the caller's code. Also, if passed in and this were changed to use it again, it can save processing time re-instantiating one.
          throw new OmException("inRT parameter should be the same as the relationType on this relation.")
        }
        relationTypeIn.get
      } else {
        new RelationType(mDB, getAttrTypeId)
      }
    }

    val rtName: String = {
      if (relatedEntityIn.isDefined) {
        if (relatedEntityIn.get.getId == mEntityId2) {
          relType.getName
        } else if (relatedEntityIn.get.getId == mEntityId1) {
          relType.getNameInReverseDirection
        }
        else throw new OmException("Unrelated parent entity parameter?: '" + relatedEntityIn.get.getId + "', '" + relatedEntityIn.get.getName + "'")
      } else {
        relType.getName
      }
    }

    // (See method comment about the relatedEntityIn param.)
    val relatedEntity = relatedEntityIn.getOrElse(new Entity(mDB, mEntityId2))
    var result: String =
      if (simplify) {
        if (rtName == Database.theHASrelationTypeName) relatedEntity.getName
        else rtName + getRemoteDescription + ": " + relatedEntity.getName
      } else {
        rtName + getRemoteDescription + ": " + Color.blue(relatedEntity.getName) + "; " + getDatesDescription
      }
    if (this.isInstanceOf[RelationToRemoteEntity]) {
      result = "[remote]" + result
    }
    Attribute.limitDescriptionLength(result, lengthLimitIn)
  }

  protected def getRemoteDescription = ""

  protected def readDataFromDB() {
    val relationData: Array[Option[Any]] = mDB.getRelationToEntityData(mAttrTypeId, mEntityId1, mEntityId2)
    if (relationData.length == 0) {
      throw new OmException("No results returned from data request for: " + mAttrTypeId + ", " + mEntityId1 + ", " + mEntityId2)
    }
    // No other local variables to assign.  All are either in the superclass or the primary key.
    // (The inEntityId1 really doesn't fit here, because it's part of the class' primary key. But passing it here for the convenience of using
    // the class hierarchy which wants it. Improve...?)
    super.assignCommonVars(mEntityId1, mAttrTypeId,
                           relationData(1).asInstanceOf[Option[Long]],
                           relationData(2).get.asInstanceOf[Long], relationData(3).get.asInstanceOf[Long])
  }

  def update(oldAttrTypeIdIn: Long, validOnDateIn:Option[Long], observationDateIn:Option[Long], newAttrTypeIdIn: Option[Long] = None) {
    val newAttrTypeId = newAttrTypeIdIn.getOrElse(getAttrTypeId)
    //Using validOnDateIn rather than validOnDateIn.get because validOnDate allows None, unlike others.
    //(Idea/possible bug: the way this is written might mean one can never change vod to None from something else: could ck callers & expectations
    // & how to be most clear (could be the same in RelationToGroup & other Attribute subclasses).)
    val vod = if (validOnDateIn.isDefined) validOnDateIn else getValidOnDate
    val od = if (observationDateIn.isDefined) observationDateIn.get else getObservationDate
    mDB.updateRelationToEntity(oldAttrTypeIdIn, mEntityId1, mEntityId2, newAttrTypeId, vod, od)
    mValidOnDate = vod
    mObservationDate = od
    mAttrTypeId = newAttrTypeId
  }

  /** Removes this object from the system. */
  def delete() = mDB.deleteRelationToEntity(getAttrTypeId, mEntityId1, mEntityId2)

}