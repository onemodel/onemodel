/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2014-2015 inclusive, Luke A. Call; all rights reserved.
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

import org.onemodel.database.PostgreSQLDatabase

/** See TextAttribute etc for some comments.
  */
class BooleanAttribute(mDB: PostgreSQLDatabase, mId: Long) extends AttributeWithValidAndObservedDates(mDB, mId) {
  if (!mDB.booleanAttributeKeyExists(mId)) {
    // DON'T CHANGE this msg unless you also change the trap for it, if used, in other code.
    throw new Exception("Key " + mId + " does not exist in database.")
  }


  /** This one is perhaps only called by the database class implementation--so it can return arrays of objects & save more DB hits
    that would have to occur if it only returned arrays of keys. This DOES NOT create a persistent object--but rather should reflect
    one that already exists.
    */
  def this(mDB: PostgreSQLDatabase, mId: Long, inParentId: Long, inAttrTypeId: Long, inBoolean: Boolean, validOnDate: Option[Long], observationDate: Long) {
    this(mDB, mId)
    mBoolean = inBoolean
    assignCommonVars(inParentId, inAttrTypeId, validOnDate, observationDate)
  }

  /** return some string. See comments on QuantityAttribute.getDisplayString regarding the parameters.
    */
  def getDisplayString(lengthLimitIn: Int, unused: Option[Entity] = None, unused2: Option[RelationType]=None, simplify: Boolean = false): String = {
    val typeName: String = mDB.getEntityName(getAttrTypeId).get
    var result: String = typeName + ": " + getBoolean + ""
    if (! simplify) result += "; " + getDatesDescription
    Attribute.limitDescriptionLength(result, lengthLimitIn)
  }

  def getBoolean: Boolean = {
    if (!mAlreadyReadData) readDataFromDB()
    mBoolean
  }

  protected def readDataFromDB() {
    val baTypeData = mDB.getBooleanAttributeData(mId)
    mBoolean = baTypeData(1).get.asInstanceOf[Boolean]
    super.assignCommonVars(baTypeData(0).get.asInstanceOf[Long], baTypeData(2).get.asInstanceOf[Long], baTypeData(3).asInstanceOf[Option[Long]], baTypeData(4).get.asInstanceOf[Long])
  }

  def update(inAttrTypeId: Long, inBoolean: Boolean, inValidOnDate: Option[Long], inObservationDate: Long) {
    // write it to the database table--w/ a record for all these attributes plus a key indicating which Entity
    // it all goes with
    mDB.updateBooleanAttribute(mId, getParentId, inAttrTypeId, inBoolean, inValidOnDate, inObservationDate)
  }

  /** Removes this object from the system. */
  def delete() = mDB.deleteBooleanAttribute(mId)

  /** For descriptions of the meanings of these variables, see the comments
    on PostgreSQLDatabase.createBooleanAttribute(...) or createTables().
    */
  private var mBoolean: Boolean = false
}