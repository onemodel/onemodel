/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2004, 2010, 2011, and 2013-2016 inclusive, Luke A. Call; all rights reserved.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule, guidelines around binary
    distribution, and the GNU Affero General Public License as published by the Free Software Foundation, either version 3
    of the License, or (at your option) any later version.  See the file LICENSE for details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>

    (This was originally cloned from RelationToEntity which has the above copyright years for its contents.)

  ---------------------------------------------------
  (See comment in this place in PostgreSQLDatabase.scala about possible alternatives to this use of the db via this layer and jdbc.)
*/
package org.onemodel.core.model

import org.onemodel.core.model.Database
import org.onemodel.core.{OmException, Util}

/**
 * Represents one RelationToRemoteEntity object in the system, used as an attribute on a Entity.
 *
 * RelationToEntity and RelationToRemoteEntity in the db are separate due to needing a different column and different indexes.  But they are
 * managed together in some other parts of the code to try to make the experience smooth and similar for the user, for whichever kind is in use.
 *
 * This 1st constructor instantiates an existing object from the DB. You can use Entity.addRelationToEntity() to
 * create a new object. Assumes caller just read it from the DB and the info is accurate (i.e., this may only ever need to be called by
 * a PostgreSQLDatabase instance?).
 *
   **NOTE**: it does make sense to instantiate a RelationToRemoteEntity with a db parameter being for a *local* (e.g., postgresql) database,
   because the local db contains references to remote ones.  Then, when creating for example an Entity for one at the remote site, that
   would have a db parameter which is remote (i.e., an instance of RestDatabase).

 */
class RelationToRemoteEntity(mDB: Database, mId: Long, mRelTypeId: Long, mEntityId1: Long, mRemoteInstanceId: String,
                       mEntityId2: Long) extends RelationToEntity(mDB, mId, mRelTypeId, mEntityId1, mEntityId2) {
  // (See comment at similar location in BooleanAttribute.)

  // This is using inheritance as a way to share code, but they do not "inherit" inside the PostgreSQLDatabase:
  if (mDB.isRemote || mDB.relationToRemoteEntityKeysExistAndMatch(mId, mRelTypeId, mEntityId1, mRemoteInstanceId, mEntityId2)) {
    // something else might be cleaner, but these are the same thing and we need to make sure an eventual superclass' var doesn't overwrite this w/ 0:
    mAttrTypeId = mRelTypeId
  } else {
    throw new scala.Exception("Key id=" + mId + ", rel_type_id=" + mRelTypeId + " and entity_id=" + mEntityId1 +
                              " and entity_id_2=" + mEntityId2 + " and remote_instance_id='" + mRemoteInstanceId + Util.DOES_NOT_EXIST)
  }


  /**
   * This  one is perhaps only called by the database class implementation--so it can return arrays of objects & save more DB hits
   * that would have to occur if it only returned arrays of keys. This DOES NOT create a persistent object--but rather should reflect
   * one that already exists.
   */
  def this(mDB: Database, idIn: Long, relTypeIdIn: Long, entityId1In: Long, remoteInstanceIdIn: String, entityId2In: Long,
           validOnDateIn: Option[Long], observationDateIn: Long, sortingIndexIn: Long) {
    this(mDB, idIn, relTypeIdIn, entityId1In, remoteInstanceIdIn, entityId2In)
    // (The inEntityId1 really doesn't fit here, because it's part of the class' primary key. But passing it here for the convenience of using
    // the class hierarchy which wants it. Improve...?)
    assignCommonVars(entityId1In, relTypeIdIn, validOnDateIn, observationDateIn, sortingIndexIn)
  }

  def getRemoteInstanceId: String = mRemoteInstanceId

  protected override def readDataFromDB() {
    val relationData: Array[Option[Any]] = mDB.getRelationToRemoteEntityData(mAttrTypeId, mEntityId1, mRemoteInstanceId, mEntityId2)
    // No other local variables to assign.  All are either in the superclass or the primary key.
    // (The inEntityId1 really doesn't fit here, because it's part of the class' primary key. But passing it here for the convenience of using
    // the class hierarchy which wants it. Improve...?)
    if (relationData.length == 0) {
      throw new OmException("No results returned from data request for: " + mAttrTypeId + ", " + mEntityId1 + ", " + mRemoteInstanceId + ", " + mEntityId2)
    }
    super.assignCommonVars(mEntityId1, mAttrTypeId,
                           relationData(1).asInstanceOf[Option[Long]],
                           relationData(2).get.asInstanceOf[Long], relationData(3).get.asInstanceOf[Long])
  }

  override def update(oldAttrTypeIdIn: Long, validOnDateIn:Option[Long], observationDateIn:Option[Long], newAttrTypeIdIn: Option[Long] = None) {
    val newAttrTypeId = newAttrTypeIdIn.getOrElse(getAttrTypeId)
    //Using validOnDateIn rather than validOnDateIn.get because validOnDate allows None, unlike others.
    //(Idea/possible bug: the way this is written might mean one can never change vod to None from something else: could ck callers & expectations
    // & how to be most clear (could be the same in RelationToGroup & other Attribute subclasses).)
    val vod = if (validOnDateIn.isDefined) validOnDateIn else getValidOnDate
    val od = if (observationDateIn.isDefined) observationDateIn.get else getObservationDate
    mDB.updateRelationToRemoteEntity(oldAttrTypeIdIn, mEntityId1, mRemoteInstanceId, mEntityId2, newAttrTypeId, vod, od)
    mValidOnDate = vod
    mObservationDate = od
    mAttrTypeId = newAttrTypeId
  }

  /** Removes this object from the system. */
  override def delete() = mDB.deleteRelationToRemoteEntity(getAttrTypeId, getRelatedId1, mRemoteInstanceId, getRelatedId2)

  override def getRemoteDescription = {
    val remoteOmInstance = new OmInstance(mDB, this.asInstanceOf[RelationToRemoteEntity].getRemoteInstanceId)
    " (at " + remoteOmInstance.getAddress + ")"
  }

  def getRemoteAddress: String = {
    mRemoteAddress
  }

  lazy private val mRemoteAddress = new OmInstance(mDB, getRemoteInstanceId).getAddress

}
