/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003, 2004, 2010, 2011, and 2013-2016 inclusive, Luke A. Call; all rights reserved.
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

import org.onemodel.core.Util
import org.onemodel.core.database.Database

/** Represents one quantity object in the system (usually [always, as of 9/2002] used as an attribute on a Entity).
  *
  * This constructor instantiates an existing object from the DB. You can use Entity.addQuantityAttribute() to
  * create a new object.
  */
class QuantityAttribute(mDB: Database, mId: Long) extends AttributeWithValidAndObservedDates(mDB, mId) {
  // (See comment at similar location in BooleanAttribute.)
  if (!mDB.isRemote && !mDB.quantityAttributeKeyExists(mId)) {
    throw new Exception("Key " + mId + Util.DOES_NOT_EXIST)
  }

  /**
   * This one is perhaps only called by the database class implementation--so it can return arrays of objects & save more DB hits
   * that would have to occur if it only returned arrays of keys. This DOES NOT create a persistent object--but rather should reflect
   * one that already exists.
   */
  def this(db: Database, id: Long, parentIdIn: Long, attrTypeIdIn: Long, unitIdIn: Long, numberIn: Float, validOnDate: Option[Long],
           observationDate: Long, sortingIndex: Long) {
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
  def getDisplayString(lengthLimitIn: Int, unused: Option[Entity]=None, unused2: Option[RelationType]=None, simplify: Boolean = false): String = {
    val typeName: String = mDB.getEntityName(getAttrTypeId).get
    val number: Float = getNumber
    val unitId: Long = getUnitId
    var result: String = typeName + ": " + number + " " + mDB.getEntityName(unitId).get
    if (! simplify) result += "; " + getDatesDescription
    Attribute.limitDescriptionLength(result, lengthLimitIn)
  }

  private[onemodel] def getNumber: Float = {
    if (!mAlreadyReadData) readDataFromDB()
    mNumber
  }

  private[onemodel] def getUnitId: Long = {
    if (!mAlreadyReadData) readDataFromDB()
    mUnitId
  }

  protected def readDataFromDB() {
    val quantityData = mDB.getQuantityAttributeData(mId)
    mUnitId = quantityData(1).get.asInstanceOf[Long]
    mNumber = quantityData(2).get.asInstanceOf[Float]
    assignCommonVars(quantityData(0).get.asInstanceOf[Long], quantityData(3).get.asInstanceOf[Long], quantityData(4).asInstanceOf[Option[Long]],
                           quantityData(5).get.asInstanceOf[Long], quantityData(6).get.asInstanceOf[Long])
  }

  def update(attrTypeIdIn: Long, unitIdIn: Long, numberIn: Float, validOnDateIn: Option[Long], observationDateIn: Long) {
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
  def delete() = mDB.deleteQuantityAttribute(mId)

  // **idea: make these members into vals not vars, by replacing them with the next line.
  //           private val (unitId: Long, number: Float) = readDataFromDB()
  // BUT: have to figure out how to work with the
  // assignment from the other constructor, and passing vals to the superclass to be...vals.  Need to know scala better,
  // like how additional class vals are set when the other constructor (what's the term again?), is called. How to do the other constructor w/o a db hit.
  /**
   * For descriptions of the meanings of these variables, see the comments
   * on createQuantityAttribute(...) or createTables() in PostgreSQLDatabase or Database classes
   */
  private var mUnitId: Long = 0L
  private var mNumber: Float = .0F
}