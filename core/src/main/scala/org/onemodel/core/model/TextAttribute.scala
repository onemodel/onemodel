/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003, 2004, 2010, 2011, 2013-2016 inclusive, Luke A. Call; all rights reserved.
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

/** Represents one String object in the system (usually [always, as of 9/2002] used as an attribute on a Entity).

    This constructor instantiates an existing object from the DB. You can use Entity.addTextAttribute() to
    create a new object.
  */
class TextAttribute(mDB: Database, mId: Long) extends AttributeWithValidAndObservedDates(mDB, mId) {
  // (See comment at similar location in BooleanAttribute.)
  if (!mDB.isRemote && !mDB.textAttributeKeyExists(mId)) {
    throw new Exception("Key " + mId + Util.DOES_NOT_EXIST)
  }


  /** This one is perhaps only called by the database class implementation--so it can return arrays of objects & save more DB hits
    that would have to occur if it only returned arrays of keys. This DOES NOT create a persistent object--but rather should reflect
    one that already exists.
    */
  def this(mDB: Database, mId: Long, parentIdIn: Long, attrTypeIdIn: Long, textIn: String, validOnDate: Option[Long], observationDate: Long,
           sortingIndexIn: Long) {
    this(mDB, mId)
    assignCommonVars(parentIdIn, attrTypeIdIn, validOnDate, observationDate, sortingIndexIn)
    mText = textIn
  }

  /** return some string. See comments on QuantityAttribute.getDisplayString regarding the parameters.
    */
  def getDisplayString(lengthLimitIn: Int, unused: Option[Entity] = None, unused2: Option[RelationType]=None, simplify: Boolean = false): String = {
    val typeName: String = mDB.getEntityName(getAttrTypeId).get
    var result: String = {
      if (simplify && (typeName == "paragraph" || typeName == "quote")) getText
      else typeName + ": \"" + getText + "\""
    }
    if (! simplify) result += "; " + getDatesDescription
    Attribute.limitDescriptionLength(result, lengthLimitIn)
  }

  def getText: String = {
    if (!mAlreadyReadData) readDataFromDB()
    mText
  }

  protected def readDataFromDB() {
    val taTypeData = mDB.getTextAttributeData(mId)
    mText = taTypeData(1).get.asInstanceOf[String]
    super.assignCommonVars(taTypeData(0).get.asInstanceOf[Long], taTypeData(2).get.asInstanceOf[Long], taTypeData(3).asInstanceOf[Option[Long]], taTypeData(4).get.asInstanceOf[Long], taTypeData(5).get.asInstanceOf[Long])
  }

  def update(attrTypeIdIn: Long, textIn: String, validOnDateIn: Option[Long], observationDateIn: Long) {
    // write it to the database table--w/ a record for all these attributes plus a key indicating which Entity
    // it all goes with
    mDB.updateTextAttribute(mId, getParentId, attrTypeIdIn, textIn, validOnDateIn, observationDateIn)
    mText = textIn
    mAttrTypeId = attrTypeIdIn
    mValidOnDate = validOnDateIn
    mObservationDate = observationDateIn
  }

  /** Removes this object from the system. */
  def delete() = mDB.deleteTextAttribute(mId)

  /** For descriptions of the meanings of these variables, see the comments
    on createTextAttribute(...) or createTables() in PostgreSQLDatabase or Database classes.
    */
  private var mText: String = null
}