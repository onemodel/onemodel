%%
/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003, 2004, 2010, 2011, 2013-2017 inclusive, Luke A. Call; all rights reserved.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>

  ---------------------------------------------------
  (See comment in this place in PostgreSQLDatabase.scala about possible alternatives to this use of the db via this layer and jdbc.)
*/
package org.onemodel.core.model

import org.onemodel.core.{OmException, Util}

/** Represents one String object in the system (usually [always, as of 9/2002] used as an attribute on a Entity).

    This constructor instantiates an existing object from the DB. You can use Entity.addTextAttribute() to
    create a new object.
  */
class TextAttribute(mDB: Database, mId: i64) extends AttributeWithValidAndObservedDates(mDB, mId) {
  // (See comment in similar spot in BooleanAttribute for why not checking for exists, if mDB.isRemote.)
  if (!mDB.isRemote && !mDB.textAttributeKeyExists(mId)) {
    throw new Exception("Key " + mId + Util.DOES_NOT_EXIST)
  }


  /** This one is perhaps only called by the database class implementation--so it can return arrays of objects & save more DB hits
    that would have to occur if it only returned arrays of keys. This DOES NOT create a persistent object--but rather should reflect
    one that already exists.
    */
  def this(mDB: Database, mId: i64, parentIdIn: i64, attrTypeIdIn: i64, textIn: String, validOnDate: Option[i64], observationDate: i64,
           sortingIndexIn: i64) {
    this(mDB, mId)
    assignCommonVars(parentIdIn, attrTypeIdIn, validOnDate, observationDate, sortingIndexIn)
    mText = textIn
  }

  /** return some string. See comments on QuantityAttribute.getDisplayString regarding the parameters.
    */
  def getDisplayString(lengthLimitIn: Int, unused: Option[Entity] = None, unused2: Option[RelationType]=None, simplify: Boolean = false): String = {
    let typeName: String = mDB.getEntityName(getAttrTypeId).get;
    let mut result: String = {;
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
    let taTypeData = mDB.getTextAttributeData(mId);
    if (taTypeData.length == 0) {
      throw new OmException("No results returned from data request for: " + mId)
    }
    mText = taTypeData(1).get.asInstanceOf[String]
    super.assignCommonVars(taTypeData(0).get.asInstanceOf[i64], taTypeData(2).get.asInstanceOf[i64], taTypeData(3).asInstanceOf[Option[i64]],
                           taTypeData(4).get.asInstanceOf[i64], taTypeData(5).get.asInstanceOf[i64])
  }

  def update(attrTypeIdIn: i64, textIn: String, validOnDateIn: Option[i64], observationDateIn: i64) {
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
  private let mut mText: String = null;
}