/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2014, 2016-2016 inclusive, Luke A Call; all rights reserved.
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

import org.onemodel.core.model.Database

object AttributeWithValidAndObservedDates {
  def getDatesDescription(mValidOnDate:Option[Long], mObservationDate:Long): String = {
    val validDateDescr: String =
      if (mValidOnDate.isEmpty) "unsp'd"
      else if (mValidOnDate.get == 0) "all time"
      else Attribute.usefulDateFormat(mValidOnDate.get)
    val observedDateDescr: String = Attribute.usefulDateFormat(mObservationDate)
    "valid " + validDateDescr + ", obsv'd " + observedDateDescr
  }
}

abstract class AttributeWithValidAndObservedDates(mDB: Database, mId: Long) extends Attribute(mDB, mId) {
  protected def assignCommonVars(parentIdIn: Long, attrTypeIdIn: Long, validOnDateIn: Option[Long], observationDateIn: Long, sortingIndexIn: Long) {
    mValidOnDate = validOnDateIn
    // observationDate is not expected to be None, like mValidOnDate can be. See var def for more info.
    mObservationDate = observationDateIn
    super.assignCommonVars(parentIdIn, attrTypeIdIn, sortingIndexIn)
  }

  def getDatesDescription: String = {
    AttributeWithValidAndObservedDates.getDatesDescription(getValidOnDate, getObservationDate)
  }

  private[onemodel] def getValidOnDate: Option[Long] = {
    if (!mAlreadyReadData) readDataFromDB()
    mValidOnDate
  }

  private[onemodel] def getObservationDate: Long = {
    if (!mAlreadyReadData) readDataFromDB()
    mObservationDate
  }

  /**
   * For descriptions of the meanings of these variables, see the comments
   * on createTables(...), and examples in the database testing code in PostgreSQLDatabase or Database classes.
   */
  protected var mValidOnDate: Option[Long] = None
  protected var mObservationDate: Long = 0L
}