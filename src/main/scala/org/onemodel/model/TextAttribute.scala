/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003, 2004, 2010, 2011, 2013-2015 inclusive, Luke A. Call; all rights reserved.
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

/** Represents one String object in the system (usually [always, as of 9/2002] used as an attribute on a Entity).

    This constructor instantiates an existing object from the DB. You can use Entity.addTextAttribute() to
    create a new object.
  */
class TextAttribute(mDB: PostgreSQLDatabase, mId: Long) extends AttributeWithValidAndObservedDates(mDB, mId) {
  if (!mDB.textAttributeKeyExists(mId)) {
    // DON'T CHANGE this msg unless you also change the trap for it, if used, in other code.
    throw new Exception("Key " + mId + " does not exist in database.")
  }


  /** This one is perhaps only called by the database class implementation--so it can return arrays of objects & save more DB hits
    that would have to occur if it only returned arrays of keys. This DOES NOT create a persistent object--but rather should reflect
    one that already exists.
    */
  def this(mDB: PostgreSQLDatabase, mId: Long, inParentId: Long, inAttrTypeId: Long, inText: String, validOnDate: Option[Long], observationDate: Long) {
    this(mDB, mId)
    assignCommonVars(inParentId, inAttrTypeId, validOnDate, observationDate)
    mText = inText
  }

  /** return some string. See comments on QuantityAttribute.getDisplayString regarding the parameters.
    */
  def getDisplayString(lengthLimitIn: Int, unused: Option[Entity] = None, unused2: Option[RelationType]=None, simplify: Boolean = false): String = {
    val typeName: String = mDB.getEntityName(getAttrTypeId).get
    var result: String = typeName + ": \"" + getText + "\""
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
    super.assignCommonVars(taTypeData(0).get.asInstanceOf[Long], taTypeData(2).get.asInstanceOf[Long], taTypeData(3).asInstanceOf[Option[Long]], taTypeData(4).get.asInstanceOf[Long])
  }

  def update(inAttrTypeId: Long, inText: String, inValidOnDate: Option[Long], inObservationDate: Long) {
    // write it to the database table--w/ a record for all these attributes plus a key indicating which Entity
    // it all goes with
    mDB.updateTextAttribute(mId, getParentId, inAttrTypeId, inText, inValidOnDate, inObservationDate)
  }

  /** Removes this object from the system. */
  def delete() = mDB.deleteTextAttribute(mId)

  /** For descriptions of the meanings of these variables, see the comments
    on PostgreSQLDatabase.createTextAttribute(...) or createTables().
    */
  private var mText: String = null
}