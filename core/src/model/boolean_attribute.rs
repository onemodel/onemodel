/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2014-2017 inclusive, and 2023, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>

  ---------------------------------------------------
  (See comment in this place in PostgreSQLDatabase.scala about possible alternatives to this use of the db via this layer and jdbc.)
*/
struct BooleanAttribute {
/*%%
package org.onemodel.core.model

import org.onemodel.core.{OmException, Util}

** See TextAttribute etc for some comments.
  *
class BooleanAttribute(mDB: Database, mId: i64) extends AttributeWithValidAndObservedDates(mDB, mId) {
  // Not doing these checks if the object is at a remote site because doing it over REST would probably be too slow. Will
  // wait for an error later to see if there is a problem (ie, assuming usually not).
  if (!mDB.isRemote && !mDB.booleanAttributeKeyExists(mId)) {
    throw new Exception("Key " + mId + Util.DOES_NOT_EXIST)
  }


  /** This one is perhaps only called by the database class implementation--so it can return arrays of objects & save more DB hits
    that would have to occur if it only returned arrays of keys. This DOES NOT create a persistent object--but rather should reflect
    one that already exists.
    */
    fn this(mDB: Database, mId: i64, parentIdIn: i64, attrTypeIdIn: i64, booleanIn: Boolean, validOnDate: Option[i64], observationDate: i64,
           sortingIndexIn: i64) {
    this(mDB, mId)
    mBoolean = booleanIn
    assignCommonVars(parentIdIn, attrTypeIdIn, validOnDate, observationDate, sortingIndexIn)
  }

  /** return some string. See comments on QuantityAttribute.getDisplayString regarding the parameters.
    */
    fn getDisplayString(lengthLimitIn: Int, unused: Option[Entity] = None, unused2: Option[RelationType]=None, simplify: Boolean = false) -> String {
    let typeName: String = mDB.getEntityName(getAttrTypeId).get;
    let mut result: String = typeName + ": " + getBoolean + "";
    if (! simplify) result += "; " + getDatesDescription
    Attribute.limitDescriptionLength(result, lengthLimitIn)
  }

    fn getBoolean() -> Boolean {
    if (!mAlreadyReadData) readDataFromDB()
    mBoolean
  }

  protected fn readDataFromDB() {
    let baTypeData = mDB.getBooleanAttributeData(mId);
    if (baTypeData.length == 0) {
      throw new OmException("No results returned from data request for: " + mId)
    }
    mBoolean = baTypeData(1).get.asInstanceOf[Boolean]
    super.assignCommonVars(baTypeData(0).get.asInstanceOf[i64], baTypeData(2).get.asInstanceOf[i64], baTypeData(3).asInstanceOf[Option[i64]],
                           baTypeData(4).get.asInstanceOf[i64], baTypeData(5).get.asInstanceOf[i64])
  }

    fn update(attrTypeIdIn: i64, booleanIn: Boolean, validOnDateIn: Option[i64], observationDateIn: i64) {
    // write it to the database table--w/ a record for all these attributes plus a key indicating which Entity
    // it all goes with
    mDB.updateBooleanAttribute(mId, getParentId, attrTypeIdIn, booleanIn, validOnDateIn, observationDateIn)
    mAttrTypeId = attrTypeIdIn
    mBoolean = booleanIn
    mValidOnDate = validOnDateIn
    mObservationDate = observationDateIn
  }

  /** Removes this object from the system. */
    fn delete() {
    mDB.deleteBooleanAttribute(mId)
    }

  /** For descriptions of the meanings of these variables, see the comments
    on createBooleanAttribute(...) or createTables() in PostgreSQLDatabase or Database classes.
    */
    private let mut mBoolean: bool = false;
 */
}