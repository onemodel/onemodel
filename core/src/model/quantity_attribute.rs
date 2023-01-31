/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003, 2004, 2010, 2011, 2013-2017 inclusive, and 2023, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>

  ---------------------------------------------------
  (See comment in this place in PostgreSQLDatabase.scala about possible alternatives to this use of the db via this layer and jdbc.)
*/
struct QuantityAttribute {
/*%%
package org.onemodel.core.model

import org.onemodel.core.{OmException, Util}

** Represents one quantity object in the system (usually [always, as of 9/2002] used as an attribute on a Entity).
  *
  * This constructor instantiates an existing object from the DB. You can use Entity.addQuantityAttribute() to
  * create a new object.
  *
class QuantityAttribute(mDB: Database, mId: i64) extends AttributeWithValidAndObservedDates(mDB, mId) {
  // (See comment in similar spot in BooleanAttribute for why not checking for exists, if mDB.isRemote.)
  if (!mDB.isRemote && !mDB.quantityAttributeKeyExists(mId)) {
    throw new Exception("Key " + mId + Util.DOES_NOT_EXIST)
  }

  /**
   * This one is perhaps only called by the database class implementation--so it can return arrays of objects & save more DB hits
   * that would have to occur if it only returned arrays of keys. This DOES NOT create a persistent object--but rather should reflect
   * one that already exists.
   */
    fn this(db: Database, id: i64, parentIdIn: i64, attrTypeIdIn: i64, unitIdIn: i64, numberIn: Float, validOnDate: Option<i64>,
           observationDate: i64, sortingIndex: i64) {
    this(db, id)
    mUnitId = unitIdIn
    mNumber = numberIn
    assignCommonVars(parentIdIn, attrTypeIdIn, validOnDate, observationDate, sortingIndex)
  }

  /**
   * return something like "volume: 15.1 liters". For full length, pass in 0 for
   * inLengthLimit. The parameter inParentEntity refers to the Entity whose
   * attribute this is. 3rd parameter really only applies in one of the subclasses of Attribute,
   * otherwise can be None.
   */
    fn getDisplayString(lengthLimitIn: Int, unused: Option[Entity]=None, unused2: Option[RelationType]=None, simplify: Boolean = false) -> String {
    let typeName: String = mDB.getEntityName(getAttrTypeId).get;
    let number: Float = getNumber;
    let unitId: i64 = getUnitId;
    let mut result: String = typeName + ": " + number + " " + mDB.getEntityName(unitId).get;
    if (! simplify) result += "; " + getDatesDescription
    Attribute.limitDescriptionLength(result, lengthLimitIn)
  }

  private[onemodel] fn getNumber -> Float {
    if (!mAlreadyReadData) readDataFromDB()
    mNumber
  }

  private[onemodel] fn getUnitId -> i64 {
    if (!mAlreadyReadData) readDataFromDB()
    mUnitId
  }

  protected fn readDataFromDB() {
    let quantityData = mDB.getQuantityAttributeData(mId);
    if (quantityData.length == 0) {
      throw new OmException("No results returned from data request for: " + mId)
    }
    mUnitId = quantityData(1).get.asInstanceOf[i64]
    mNumber = quantityData(2).get.asInstanceOf[Float]
    assignCommonVars(quantityData(0).get.asInstanceOf[i64], quantityData(3).get.asInstanceOf[i64], quantityData(4).asInstanceOf[Option<i64>],
                           quantityData(5).get.asInstanceOf[i64], quantityData(6).get.asInstanceOf[i64])
  }

    fn update(attrTypeIdIn: i64, unitIdIn: i64, numberIn: Float, validOnDateIn: Option<i64>, observationDateIn: i64) {
    // write it to the database table--w/ a record for all these attributes plus a key indicating which Entity
    // it all goes with
    mDB.updateQuantityAttribute(mId, getParentId, attrTypeIdIn, unitIdIn, numberIn, validOnDateIn, observationDateIn)
    mAttrTypeId = attrTypeIdIn
    mUnitId = unitIdIn
    mNumber = numberIn
    mValidOnDate = validOnDateIn
    mObservationDate = observationDateIn
  }

  /** Removes this object from the system. */
    fn delete() {
    mDB.deleteQuantityAttribute(mId)
    }

  // **idea: make these members into vals not vars, by replacing them with the next line.
  //           private let (unitId: i64, number: Float) = readDataFromDB();
  // BUT: have to figure out how to work with the
  // assignment from the other constructor, and passing vals to the superclass to be...vals.  Need to know scala better,
  // like how additional class vals are set when the other constructor (what's the term again?), is called. How to do the other constructor w/o a db hit.
  /**
   * For descriptions of the meanings of these variables, see the comments
   * on createQuantityAttribute(...) or createTables() in PostgreSQLDatabase or Database classes
   */
  private let mut mUnitId: i64 = 0L;
  private let mut mNumber: Float = .0F;
 */
}
