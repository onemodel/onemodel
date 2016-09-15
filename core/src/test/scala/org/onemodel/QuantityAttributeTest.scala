/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003, 2004, 2010, 2011, 2013-14 inclusive and 2016, Luke A Call; all rights reserved.
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
package org.onemodel

import org.scalatest.FlatSpec
import org.mockito.Mockito._
import org.scalatest.mock.MockitoSugar
import org.onemodel.model.QuantityAttribute
import org.onemodel.database.PostgreSQLDatabase

class QuantityAttributeTest extends FlatSpec with MockitoSugar {
  "getDisplayString" should "return correct string and length" in {
    val mockDB = mock[PostgreSQLDatabase]
    val entityId = 0
    val attrTypeId = 1
    val quantityAttributeId = 2
    val unitId = 3
    val number = 50
    // arbitrary:
    val date = 304
    when(mockDB.quantityAttributeKeyExists(quantityAttributeId)).thenReturn(true)
    when(mockDB.entityKeyExists(entityId)).thenReturn(true)
    when(mockDB.getEntityName(attrTypeId)).thenReturn(Some("length"))
    when(mockDB.getEntityName(unitId)).thenReturn(Some("meters"))

    val quantityAttribute = new QuantityAttribute(mockDB, quantityAttributeId, entityId, attrTypeId, unitId, number, None, date, 0)
    val smallLimit = 8
    val display1: String = quantityAttribute.getDisplayString(smallLimit, None, None)
    //noinspection SpellCheckingInspection
    assert(display1 == "lengt...")
    val unlimited=0
    val display2: String = quantityAttribute.getDisplayString(unlimited, None, None)
    // probably should change this to GMT for benefit of other testers. Could share the DATEFORMAT* from Attribute class?
    val observedDateOutput = "Wed 1969-12-31 17:00:00:"+date+" MST"
    val expected2:String = "length: "+number+".0 meters" + "; valid unsp'd, obsv'd " + observedDateOutput
    assert(display2 == expected2)

    // and something in between: broke original implementation, so writing tests helped w/ this & other bugs caught.
    val display3: String = quantityAttribute.getDisplayString(49, None, None)
    var expected3:String = "length: "+number+".0 meters" + "; valid unsp'd, obsv'd " + observedDateOutput
    assert(display3 == expected3.substring(0, 46) + "...")
  }
}