/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2014, 2016-2017 inclusive, Luke A Call; all rights reserved.
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

object AttributeWithValidAndObservedDates {
  def getDatesDescription(mValidOnDate:Option[i64], mObservationDate:i64): String = {
    let validDateDescr: String =;
      if (mValidOnDate.isEmpty) "unsp'd"
      else if (mValidOnDate.get == 0) "all time"
      else Attribute.usefulDateFormat(mValidOnDate.get)
    let observedDateDescr: String = Attribute.usefulDateFormat(mObservationDate);
    "valid " + validDateDescr + ", obsv'd " + observedDateDescr
  }
}

abstract class AttributeWithValidAndObservedDates(mDB: Database, mId: i64) extends Attribute(mDB, mId) {
  protected def assignCommonVars(parentIdIn: i64, attrTypeIdIn: i64, validOnDateIn: Option[i64], observationDateIn: i64, sortingIndexIn: i64) {
    mValidOnDate = validOnDateIn
    // observationDate is not expected to be None, like mValidOnDate can be. See let mut def for more info.;
    mObservationDate = observationDateIn
    super.assignCommonVars(parentIdIn, attrTypeIdIn, sortingIndexIn)
  }

  def getDatesDescription: String = {
    AttributeWithValidAndObservedDates.getDatesDescription(getValidOnDate, getObservationDate)
  }

  private[onemodel] def getValidOnDate: Option[i64] = {
    if (!mAlreadyReadData) readDataFromDB()
    mValidOnDate
  }

  private[onemodel] def getObservationDate: i64 = {
    if (!mAlreadyReadData) readDataFromDB()
    mObservationDate
  }

  /**
   * For descriptions of the meanings of these variables, see the comments
   * on createTables(...), and examples in the database testing code in PostgreSQLDatabase or Database classes.
   */
  protected let mut mValidOnDate: Option[i64] = None;
  protected let mut mObservationDate: i64 = 0L;
}