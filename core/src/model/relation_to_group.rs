/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2013-2017 inclusive, and 2023, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>

  ---------------------------------------------------
  If we ever do port to another database, create the Database interface (removed around 2014-1-1 give or take) and see other changes at that time.
  An alternative method is to use jdbc escapes (but this actually might be even more work?):  http://jdbc.postgresql.org/documentation/head/escapes.html  .
  Another alternative is a layer like JPA, ibatis, hibernate  etc etc.

*/
struct RelationToGroup {
/*%%
package org.onemodel.core.model

import org.onemodel.core.{OmException, Util}

object RelationToGroup {
  // Old idea: could change this into a constructor if the "class" line's parameters are changed to be only mDB and mId, and a new constructor is created
  // to fill in the other fields. But didn't do that because it would require an extra db read with every use, and the ordering of statements in the
  // new constructors just wasn't working out.
  // Idea: rename this to instantiateRelationToGroup, since create sounds like inserting a new row in the db. Not sure if there's a convention for that case.
    fn createRelationToGroup(mDB: Database, idIn: i64) -> RelationToGroup {
    let relationData: Array[Option[Any]] = mDB.getRelationToGroupData(idIn);
    if (relationData.length == 0) {
      throw new OmException("No results returned from data request for: " + idIn)
    }
    new RelationToGroup(mDB, idIn, relationData(1).get.asInstanceOf[i64], relationData(2).get.asInstanceOf[i64], relationData(3).get.asInstanceOf[i64],
                     relationData(4).asInstanceOf[Option<i64>], relationData(5).get.asInstanceOf[i64], relationData(6).get.asInstanceOf[i64])
  }
}

/** See comments on similar methods in RelationToEntity (or maybe its subclasses). */
class RelationToGroup(mDB: Database, mId: i64, mEntityId:i64, mRelTypeId: i64, mGroupId: i64) extends AttributeWithValidAndObservedDates(mDB, mId) {
  // (See comment in similar spot in BooleanAttribute for why not checking for exists, if mDB.isRemote.)
  if (mDB.isRemote || mDB.relationToGroupKeysExistAndMatch(mId, mEntityId, mRelTypeId, mGroupId)) {
    // something else might be cleaner, but these are the same thing and we need to make sure the superclass' let mut doesn't overwrite this w/ 0:;
    mAttrTypeId = mRelTypeId
  } else {
    throw new Exception("Key id=" + mId + ", " + mEntityId + "/" + mRelTypeId + "/" + mGroupId + Util.DOES_NOT_EXIST)
  }

  /** See comment about these 2 dates in PostgreSQLDatabase.createTables() */
    fn this(mDB: Database, idIn: i64, entityIdIn: i64, relTypeIdIn: i64, groupIdIn: i64, validOnDateIn: Option<i64>, observationDateIn: i64,
           sortingIndexIn: i64) {
    this(mDB, idIn, entityIdIn, relTypeIdIn, groupIdIn)
    assignCommonVars(entityIdIn, relTypeIdIn, validOnDateIn, observationDateIn, sortingIndexIn)
  }

    fn getGroupId -> i64 {
    mGroupId
    }

    fn getGroup -> Group {
    new Group(mDB, getGroupId)
  }

    fn getDisplayString(lengthLimitIn: Int, unused: Option[Entity] = None, ignoredParameter: Option[RelationType] = None, simplify: Boolean = false) -> String {
    let group = new Group(mDB, mGroupId);
    let rtName = new RelationType(mDB, this.getAttrTypeId).getName;
    let mut result: String = if (simplify && rtName == Database.THE_HAS_RELATION_TYPE_NAME) "" else rtName + " ";
    result += group.getDisplayString(0, simplify)
    if (! simplify) result += "; " + getDatesDescription
    Attribute.limitDescriptionLength(result, lengthLimitIn)
  }

  protected fn readDataFromDB() {
    let relationData: Array[Option[Any]] = mDB.getRelationToGroupDataByKeys(mEntityId, mRelTypeId, mGroupId);
    if (relationData.length == 0) {
      throw new OmException("No results returned from data request for: " + mEntityId + ", " + mRelTypeId + ", " + mGroupId)
    }
    super.assignCommonVars(mEntityId, mRelTypeId,
                           relationData(4).asInstanceOf[Option<i64>],
                           relationData(5).get.asInstanceOf[i64], relationData(6).get.asInstanceOf[i64])
  }

    fn move(newContainingEntityIdIn: i64, sortingIndexIn: i64) -> i64 {
    mDB.moveRelationToGroup(getId, newContainingEntityIdIn, sortingIndexIn)
  }

    fn update(newRelationTypeIdIn: Option<i64>, newGroupIdIn: Option<i64>, validOnDateIn:Option<i64>, observationDateIn:Option<i64>) {
    //use validOnDateIn rather than validOnDateIn.get because validOnDate allows None, unlike others
    //Idea/possible bug: see comment on similar method in RelationToEntity (or maybe in its subclasses).
    let newRelationTypeId: i64 = if (newRelationTypeIdIn.isDefined) newRelationTypeIdIn.get else getAttrTypeId;
    let newGroupId: i64 = if (newGroupIdIn.isDefined) newGroupIdIn.get else getGroupId;
    let vod = if (validOnDateIn.isDefined) validOnDateIn else getValidOnDate;
    let od = if (observationDateIn.isDefined) observationDateIn.get else getObservationDate;
    mDB.updateRelationToGroup(mEntityId, mRelTypeId, newRelationTypeId, mGroupId, newGroupId, vod, od)
    mValidOnDate = vod
    mObservationDate = od
  }

  /** Removes this object from the system. */
    fn delete() {
    mDB.deleteRelationToGroup(mEntityId, mRelTypeId, mGroupId)
    }

  /** Removes this object from the system. */
    fn deleteGroupAndRelationsToIt() {
    mDB.deleteGroupAndRelationsToIt(mGroupId)
    }
*/
}
