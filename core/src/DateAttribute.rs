/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2014-2017 inclusive, Luke A. Call; all rights reserved.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule, guidelines around binary
    distribution, and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>

  ---------------------------------------------------
  (See comment in this place in PostgreSQLDatabase.scala about possible alternatives to this use of the db via this layer and jdbc.)
*/
package org.onemodel.core.model

import org.onemodel.core.{OmException, Util}

/** See TextAttribute etc for some comments.
  * Also, though this doesn't formally extend Attribute, it still belongs to the same group conceptually (just doesn't have the same date variables so code
  * not shared (idea: model that better, and in FileAttribute).
  */
class DateAttribute(mDB: Database, mId: Long) extends Attribute(mDB, mId) {
  // (See comment in similar spot in BooleanAttribute for why not checking for exists, if mDB.isRemote.)
  if (!mDB.isRemote && !mDB.dateAttributeKeyExists(mId)) {
    throw new Exception("Key " + mId + Util.DOES_NOT_EXIST)
  }


  // idea: make the parameter order uniform throughout the system
  /** This one is perhaps only called by the database class implementation--so it can return arrays of objects & save more DB hits
    that would have to occur if it only returned arrays of keys. This DOES NOT create a persistent object--but rather should reflect
    one that already exists.
    */
  def this(mDB: Database, mId: Long, inParentId: Long, attrTypeIdIn: Long, inDate: Long, sortingIndexIn: Long) {
    this(mDB, mId)
    mDate = inDate
    super.assignCommonVars(inParentId, attrTypeIdIn, sortingIndexIn)
  }

  def getDisplayString(lengthLimitIn: Int, unused: Option[Entity] = None, unused2: Option[RelationType]=None, simplify: Boolean = false): String = {
    val typeName: String = mDB.getEntityName(getAttrTypeId).get
    var result: String = typeName + ": "
    result += Attribute.usefulDateFormat(mDate)
    Attribute.limitDescriptionLength(result, lengthLimitIn)
  }

  def getDate: Long = {
    if (!mAlreadyReadData) readDataFromDB()
    mDate
  }

  protected def readDataFromDB() {
    val daTypeData = mDB.getDateAttributeData(mId)
    if (daTypeData.length == 0) {
      throw new OmException("No results returned from data request for: " + mId)
    }
    mDate = daTypeData(1).get.asInstanceOf[Long]
    assignCommonVars(daTypeData(0).get.asInstanceOf[Long], daTypeData(2).get.asInstanceOf[Long], daTypeData(3).get.asInstanceOf[Long])
  }

  def update(inAttrTypeId: Long, inDate: Long) {
    // write it to the database table--w/ a record for all these attributes plus a key indicating which Entity
    // it all goes with
    mDB.updateDateAttribute(mId, getParentId, inDate, inAttrTypeId)
    mDate = inDate
    mAttrTypeId = inAttrTypeId
  }

  /** Removes this object from the system. */
  def delete() = mDB.deleteDateAttribute(mId)

  /** For descriptions of the meanings of these variables, see the comments
    on createDateAttribute(...) or createTables() in PostgreSQLDatabase or Database classes
    */
  private var mDate: Long = 0L
}