/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2004, 2010, 2011, 2013-2017 inclusive, and 2023, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>

    (This was originally cloned from RelationToEntity which has the above copyright years for its contents.)
*/
pub struct RelationToLocalEntity {
/*%%

object RelationToLocalEntity {
  /** This is for times when you want None if it doesn't exist, instead of the exception thrown by the Entity constructor.  Or for convenience in tests.
    */
    fn getRelationToLocalEntity(db_in: Database, id: i64) -> Option[RelationToLocalEntity] {
    let result: Array[Option[Any]] = db_in.getRelationToLocalEntityDataById(id);
    let relTypeId = result(0).get.asInstanceOf[i64];
    let eid1 = result(1).get.asInstanceOf[i64];
    let eid2 = result(2).get.asInstanceOf[i64];
    try Some(new RelationToLocalEntity(db_in, id, relTypeId, eid1, eid2))
    catch {
      case e: java.lang.Exception =>
        //idea: see comment here in Entity.scala.
        if e.toString.indexOf(Util::DOES_NOT_EXIST) >= 0) {
          None
        }
        else throw e
    }
  }
}

/** This class exists, instead of just using RelationToEntity, so that the consuming code can be more clear at any given
  * time as to whether RelationToLocalEntity or RelationToRemoteEntity is being used, to avoid subtle bugs.
  *
  * This 1st constructor instantiates an existing object from the DB and is rarely needed. You can use Entity.addRelationTo[Local|Remote]Entity() to
  * create a new persistent record.
  */
class RelationToLocalEntity(m_db: Database, m_id: i64, mRelTypeId: i64, mEntityId1: i64,
                             mEntityId2: i64) extends RelationToEntity(m_db, m_id, mRelTypeId, mEntityId1, mEntityId2) {
  // This is using inheritance as a way to share code, but they do not "inherit" inside the PostgreSQLDatabase:
  // Even a RelationToRemoteEntity can have m_db.is_remote == true, if it is viewing data *in* a remote OM instance
  // looking at RTLEs that are remote to that remote instance.
  // See comment in similar spot in BooleanAttribute for why not checking for exists, if m_db.is_remote.
  if m_db.is_remote || m_db.relationToLocalEntityKeysExistAndMatch(m_id, mRelTypeId, mEntityId1, mEntityId2)) {
    // something else might be cleaner, but these are the same thing and we need to make sure the superclass' var doesn't overwrite this w/ 0:;
    m_attr_type_id = mRelTypeId
  } else {
    throw new OmException("Key id=" + m_id + ", rel_type_id=" + mRelTypeId + " and entity_id=" + mEntityId1 +
                          " and entity_id_2=" + mEntityId2 + Util::DOES_NOT_EXIST)
  }

  /**
   * This  one is perhaps only called by the database class implementation--so it can return arrays of objects & save more DB hits
   * that would have to occur if it only returned arrays of keys. This DOES NOT create a persistent object--but rather should reflect
   * one that already exists.
   */
    fn this(m_db: Database, id_in: i64, relTypeIdIn: i64, entity_id1_in: i64, entity_id2_in: i64, valid_on_date_in: Option<i64>, observation_date_in: i64,
           sorting_index_in: i64) {
    this(m_db, id_in, relTypeIdIn, entity_id1_in, entity_id2_in)
    //    if this.isInstanceOf[RelationToRemoteEntity]) {
    //      //idea: this test & exception feel awkward. What is the better approach?  Maybe using scala's type features?
    //      throw new OmException("This constructor should not be called by the subclass.")
    //    }

    // (The inEntityId1 really doesn't fit here, because it's part of the class' primary key. But passing it here for the convenience of using
    // the class hierarchy which wants it. Improve...?)
    assignCommonVars(entity_id1_in, relTypeIdIn, valid_on_date_in, observation_date_in, sorting_index_in)
  }

    fn getRemoteDescription() -> String {
    //%%have it throw an err instead or?  what do callers expect?
    ""
    }

    fn getEntityForEntityId2 -> Entity {
    new Entity(m_db, mEntityId2)
  }

  protected fn read_data_from_db() {
    let relationData: Array[Option[Any]] = m_db.getRelationToLocalEntityData(m_attr_type_id, mEntityId1, mEntityId2);
    if relationData.length == 0) {
      throw new OmException("No results returned from data request for: " + m_attr_type_id + ", " + mEntityId1 + ", " + mEntityId2)
    }
    // No other local variables to assign.  All are either in the superclass or the primary key.
    // (The inEntityId1 really doesn't fit here, because it's part of the class' primary key. But passing it here for the convenience of using
    // the class hierarchy which wants it. Improve...?)
    assignCommonVars(mEntityId1, m_attr_type_id,
                     relationData(1).asInstanceOf[Option<i64>],
                     relationData(2).get.asInstanceOf[i64], relationData(3).get.asInstanceOf[i64])
  }

    fn move(toLocalContainingEntityIdIn: i64, sorting_index_in: i64) -> RelationToLocalEntity {
    m_db.moveRelationToLocalEntityToLocalEntity(get_id, toLocalContainingEntityIdIn, sorting_index_in)
  }

    fn moveEntityFromEntityToGroup(targetGroupIdIn: i64, sorting_index_in: i64) {
    m_db.moveLocalEntityFromLocalEntityToGroup(this, targetGroupIdIn, sorting_index_in)
  }

    fn update(valid_on_date_in:Option<i64>, observation_date_in:Option<i64>, newAttrTypeIdIn: Option<i64> = None) {
    let newAttrTypeId = newAttrTypeIdIn.getOrElse(get_attr_type_id());
    //Using valid_on_date_in rather than valid_on_date_in.get because valid_on_date allows None, unlike others.
    //(Idea/possible bug: the way this is written might mean one can never change vod to None from something else: could ck callers & expectations
    // & how to be most clear (could be the same in RelationToGroup & other Attribute subclasses).)
    let vod = if valid_on_date_in.is_some()) valid_on_date_in else get_valid_on_date();
    let od = if observation_date_in.is_some()) observation_date_in.get else get_observation_date();
    m_db.updateRelationToLocalEntity(m_attr_type_id, mEntityId1, mEntityId2, newAttrTypeId, vod, od)
    valid_on_date = vod
    observation_date = od
    m_attr_type_id = newAttrTypeId
  }

  /** Removes this object from the system. */
    fn delete() {
    m_db.deleteRelationToLocalEntity(get_attr_type_id(), mEntityId1, mEntityId2)
    }

*/
}
