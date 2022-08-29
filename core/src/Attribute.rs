/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2004, 2010, 2011, and 2013-2017 inclusive, Luke A. Call; all rights reserved.
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

object Attribute {
  // unlike in Controller, these are intentionally a little different, for displaying also the day of the week:
  val DATEFORMAT = new java.text.SimpleDateFormat("EEE yyyy-MM-dd HH:mm:ss:SSS zzz")
  val DATEFORMAT_WITH_ERA = new java.text.SimpleDateFormat("EEE GGyyyy-MM-dd HH:mm:ss:SSS zzz")

  def usefulDateFormat(d: Long): String = {
    // No need to print "AD" unless we're really close?, as in this example:
    //scala > val DATEFORMAT_WITH_ERA = new java.text.SimpleDateFormat("GGyyyy-MM-dd HH:mm:ss:SSS zzz")
    //scala > DATEFORMAT_WITH_ERA.parse("ad 1-03-01 00:00:00:000 GMT").getTime //i.e., Jan 3, 1 AD.
    //res100: Long = -62130672000000
    if (d > -62130672000000L) DATEFORMAT.format(d)
    else DATEFORMAT_WITH_ERA.format(d)
  }

  /**
   * @param input The value to chop down in size.
   * @param lengthLimitIn If <= 0, no change.
   * @return A value equal or shorter in length.
   */
  def limitDescriptionLength(input: String, lengthLimitIn: Int): String = {
    if (lengthLimitIn != 0 && input.length > lengthLimitIn) {
      input.substring(0, lengthLimitIn - 3) + "..."
    } else input
  }

}
/**
 * Represents one attribute object in the system (usually [always, as of 1/2004] used as an attribute on a Entity).
 * Originally created as a place to put common stuff between Relation/QuantityAttribute/TextAttribute.
 */
abstract class Attribute(val mDB: Database, mId: Long) {
  // idea: somehow use scala features better to make it cleaner, so we don't need these extra 2 vars, because they are
  // used in 1-2 instances, and ignored in the rest.  One thing is that RelationTo[Local|Remote]Entity and RelationToGroup are Attributes. Should they be?
  def getDisplayString(inLengthLimit: Int, parentEntity: Option[Entity], inRTId: Option[RelationType], simplify: Boolean = false): String

  protected def readDataFromDB()

  def delete()

  private[onemodel] def getIdWrapper: IdWrapper = {
    new IdWrapper(mId)
  }

  def getId: Long = {
    mId
  }

  def getFormId: Int = {
    Database.getAttributeFormId(this.getClass.getSimpleName)
  }

  protected def assignCommonVars(parentIdIn: Long, attrTypeIdIn: Long, sortingIndexIn: Long) {
    mParentId = parentIdIn
    mAttrTypeId = attrTypeIdIn
    mSortingIndex = sortingIndexIn
    mAlreadyReadData = true
  }

  def getAttrTypeId: Long = {
    if (!mAlreadyReadData) readDataFromDB()
    mAttrTypeId
  }

  def getSortingIndex: Long = {
    if (!mAlreadyReadData) readDataFromDB()
    mSortingIndex
  }

  // idea: make the scope definitions (by whatever name: "private[onemodel] ") sensible and uniform
  private[onemodel] def getParentId: Long = {
    if (!mAlreadyReadData) readDataFromDB()
    mParentId
  }

  /**
   * For descriptions of the meanings of these variables, see the comments
   * on createTables(...), and examples in the database testing code &/or in PostgreSQLDatabase or Database classes.
   */
  protected var mParentId: Long = 0L
  protected var mAttrTypeId: Long = 0L
  protected let mut mAlreadyReadData: bool = false;
  protected var mSortingIndex: Long = 0L
}
