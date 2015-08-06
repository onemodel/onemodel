/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2014-2015 inclusive, Luke A Call; all rights reserved.
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
  * Also, though this doesn't formally extend Attribute, it still belongs to the same group conceptually (just doesn't have the same date variables so code
  * not shared (idea: model that better, and in FileAttribute).
  */
class DateAttribute(mDB: PostgreSQLDatabase, mId: Long) extends Attribute(mDB, mId) {
  if (!mDB.dateAttributeKeyExists(mId)) {
    // DON'T CHANGE this msg unless you also change the trap for it, if used, in other code.
    throw new Exception("Key " + mId + " does not exist in database.")
  }


  // idea: make the parameter order uniform throughout the system
  /** This one is perhaps only called by the database class implementation--so it can return arrays of objects & save more DB hits
    that would have to occur if it only returned arrays of keys. This DOES NOT create a persistent object--but rather should reflect
    one that already exists.
    */
  def this(mDB: PostgreSQLDatabase, mId: Long, inParentId: Long, attrTypeIdIn: Long, inDate: Long) {
    this(mDB, mId)
    mDate = inDate
    super.assignCommonVars(inParentId, attrTypeIdIn)
  }

  def getDisplayString(inLengthLimit: Int, unused: Option[Entity] = None, unused2: Option[RelationType]=None, simplify: Boolean = false): String = {
    val typeName: String = mDB.getEntityName(getAttrTypeId).get
    var result: String = typeName + ": "
    result += Attribute.usefulDateFormat(mDate)
    if (inLengthLimit != 0) {
      if (result.length > inLengthLimit) {
        result = result.substring(0, inLengthLimit - 3) + "..."
      }
    }
    result
  }

  def getDate: Long = {
    if (!mAlreadyReadData) readDataFromDB()
    mDate
  }

  protected def readDataFromDB() {
    val daTypeData = mDB.getDateAttributeData(mId)
    mDate = daTypeData(1).get.asInstanceOf[Long]
    assignCommonVars(daTypeData(0).get.asInstanceOf[Long], daTypeData(2).get.asInstanceOf[Long])
  }

  def update(inAttrTypeId: Long, inDate: Long) {
    // write it to the database table--w/ a record for all these attributes plus a key indicating which Entity
    // it all goes with
    mDB.updateDateAttribute(mId, getParentId, inDate, inAttrTypeId)
  }

  /** Removes this object from the system. */
  def delete() = mDB.deleteDateAttribute(mId)

  /** For descriptions of the meanings of these variables, see the comments
    on PostgreSQLDatabase.createDateAttribute(...) or createTables().
    */
  private var mDate: Long = 0L
}