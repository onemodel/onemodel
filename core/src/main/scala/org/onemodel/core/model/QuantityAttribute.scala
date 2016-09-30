/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003, 2004, 2010, 2011, and 2013-2016 inclusive, Luke A. Call; all rights reserved.
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
package org.onemodel.core.model

import org.onemodel.core.database.PostgreSQLDatabase

/** Represents one quantity object in the system (usually [always, as of 9/2002] used as an attribute on a Entity).
  *
  * This constructor instantiates an existing object from the DB. You can use Entity.addQuantityAttribute() to
  * create a new object.
  */
class QuantityAttribute(mDB: PostgreSQLDatabase, mId: Long) extends AttributeWithValidAndObservedDates(mDB, mId) {
  if (!mDB.quantityAttributeKeyExists(mId)) {
    // DON'T CHANGE this msg unless you also change the trap for it, if used, in other code.
    throw new Exception("Key " + mId + " does not exist in database.")
  }

  /**
   * This one is perhaps only called by the database class implementation--so it can return arrays of objects & save more DB hits
   * that would have to occur if it only returned arrays of keys. This DOES NOT create a persistent object--but rather should reflect
   * one that already exists.
   */
  def this(db: PostgreSQLDatabase, id: Long, inParentId: Long, inAttrTypeId: Long, inUnitId: Long, inNumber: Float, validOnDate: Option[Long],
           observationDate: Long, sortingIndex: Long) {
    this(db, id)
    mUnitId = inUnitId
    mNumber = inNumber
    assignCommonVars(inParentId, inAttrTypeId, validOnDate, observationDate, sortingIndex)
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

  def update(inAttrTypeId: Long, inUnitId: Long, inNumber: Float, inValidOnDate: Option[Long], inObservationDate: Long) {
    // write it to the database table--w/ a record for all these attributes plus a key indicating which Entity
    // it all goes with
    mDB.updateQuantityAttribute(mId, getParentId, inAttrTypeId, inUnitId, inNumber, inValidOnDate, inObservationDate)
    mAttrTypeId = inAttrTypeId
    mUnitId = inUnitId
    mNumber = inNumber
    mValidOnDate = inValidOnDate
    mObservationDate = inObservationDate
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
   * on PostgreSQLDatabase.createQuantityAttribute(...) or createTables().
   */
  private var mUnitId: Long = 0L
  private var mNumber: Float = .0F
}