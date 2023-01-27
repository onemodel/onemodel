/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2004, 2010, 2011, 2013-2017 inclusive, and 2023, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>

    (This was originally cloned from RelationToEntity which has the above copyright years for its contents.)

  ---------------------------------------------------
  (See comment in this place in PostgreSQLDatabase.scala about possible alternatives to this use of the db via this layer and jdbc.)
*/
struct RelationToRemoteEntity {
/*%%
package org.onemodel.core.model

import org.onemodel.core.{OmException, Util}

**
 * Represents one RelationToRemoteEntity object in the system, used as an attribute on a Entity.
 *
 * RelationToLocalEntity and RelationToRemoteEntity in the db are separate due to needing a different column and different indexes.  And they are
 * also separate in the code to make it as clear as possible which logic is needed at a given time.
 *
 * This 1st constructor instantiates an existing object from the DB and is rarely needed. You can use Entity.addRelationTo[Local|Remote]Entity() to
 * create a new persistent record.
 *
   **NOTE**: it *yes does* make sense to instantiate a RelationToRemoteEntity with a db parameter being for a *local* (ex., postgresql) database,
   because the local db contains references to remote ones.  Then, when creating for example an Entity for a record at a remote site, that
   would have a db parameter which is remote (i.e., an instance of RestDatabase; see Entity.addRelationToRemoteEntity).

   *****  MAKE SURE  ***** that during maintenance, anything that gets data relating to mEntityId2 is using the right (remote) db!:
 *
class RelationToRemoteEntity(mDB: Database, mId: i64, mRelTypeId: i64, mEntityId1: i64, mRemoteInstanceId: String,
                       mEntityId2: i64) extends RelationToEntity(mDB, mId, mRelTypeId, mEntityId1, mEntityId2) {
  // This is using inheritance as a way to share code, but they do not "inherit" inside the PostgreSQLDatabase:
  // (See comment in similar spot in BooleanAttribute for why not checking for exists, if mDB.isRemote.)
  if (mDB.isRemote || mDB.relationToRemoteEntityKeysExistAndMatch(mId, mRelTypeId, mEntityId1, mRemoteInstanceId, mEntityId2)) {
    // something else might be cleaner, but these are the same thing and we need to make sure an eventual superclass' var doesn't overwrite this w/ 0:;
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
    fn this(mDB: Database, idIn: i64, relTypeIdIn: i64, entityId1In: i64, remoteInstanceIdIn: String, entityId2In: i64,
           validOnDateIn: Option[i64], observationDateIn: i64, sortingIndexIn: i64) {
    this(mDB, idIn, relTypeIdIn, entityId1In, remoteInstanceIdIn, entityId2In)
    // (The inEntityId1 really doesn't fit here, because it's part of the class' primary key. But passing it here for the convenience of using
    // the class hierarchy which wants it. Improve...?)
    assignCommonVars(entityId1In, relTypeIdIn, validOnDateIn, observationDateIn, sortingIndexIn)
  }

    fn getRemoteInstanceId -> String {
    mRemoteInstanceId
    }

  protected override fn readDataFromDB() {
    let relationData: Array[Option[Any]] = mDB.getRelationToRemoteEntityData(mAttrTypeId, mEntityId1, mRemoteInstanceId, mEntityId2);
    // No other local variables to assign.  All are either in the superclass or the primary key.
    // (The inEntityId1 really doesn't fit here, because it's part of the class' primary key. But passing it here for the convenience of using
    // the class hierarchy which wants it. Improve...?)
    if (relationData.length == 0) {
      throw new OmException("No results returned from data request for: " + mAttrTypeId + ", " + mEntityId1 + ", " + mRemoteInstanceId + ", " + mEntityId2)
    }
    assignCommonVars(mEntityId1, mAttrTypeId, relationData(1).asInstanceOf[Option[i64]],
                     relationData(2).get.asInstanceOf[i64], relationData(3).get.asInstanceOf[i64])
  }

    fn move(toContainingEntityIdIn: i64, sortingIndexIn: i64) -> RelationToRemoteEntity {
    mDB.moveRelationToRemoteEntityToLocalEntity(getRemoteInstanceId, getId, toContainingEntityIdIn, sortingIndexIn)
  }

  override fn getRemoteDescription() {
    let remoteOmInstance = new OmInstance(mDB, this.asInstanceOf[RelationToRemoteEntity].getRemoteInstanceId);
    " (at " + remoteOmInstance.getAddress + ")"
  }

    fn getEntityForEntityId2() -> Entity {
    new Entity(getRemoteDatabase, mEntityId2)
  }

    fn getRemoteDatabase() -> Database {
    Database.getRestDatabase(mRemoteAddress)
  }

    fn update(validOnDateIn:Option[i64], observationDateIn:Option[i64], newAttrTypeIdIn: Option[i64] = None) {
    let newAttrTypeId = newAttrTypeIdIn.getOrElse(getAttrTypeId);
    //Using validOnDateIn rather than validOnDateIn.get because validOnDate allows None, unlike others.
    //(Idea/possible bug: the way this is written might mean one can never change vod to None from something else: could ck callers & expectations
    // & how to be most clear (could be the same in RelationToGroup & other Attribute subclasses).)
    let vod = if (validOnDateIn.isDefined) validOnDateIn else getValidOnDate;
    let od = if (observationDateIn.isDefined) observationDateIn.get else getObservationDate;
    mDB.updateRelationToRemoteEntity(mAttrTypeId, mEntityId1, getRemoteInstanceId, mEntityId2, newAttrTypeId, vod, od)
    mValidOnDate = vod
    mObservationDate = od
    mAttrTypeId = newAttrTypeId
  }

  /** Removes this object from the system. */
    fn delete() {
    mDB.deleteRelationToRemoteEntity(getAttrTypeId, getRelatedId1, mRemoteInstanceId, getRelatedId2)
    }

  lazy private let mRemoteAddress = new OmInstance(mDB, getRemoteInstanceId).getAddress;

*/
}
