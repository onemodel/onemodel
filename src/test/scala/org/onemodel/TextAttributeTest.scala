/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2013-2014 inclusive, Luke A Call; all rights reserved.
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
import org.onemodel.model.TextAttribute
import org.onemodel.database.PostgreSQLDatabase

class TextAttributeTest extends FlatSpec with MockitoSugar {
  "getDisplayString" should "return correct string and length" in {
    val mockDB = mock[PostgreSQLDatabase]
    val entityId = 0
    val otherEntityId = 1
    val textAttributeId = 0
    //arbitrary, in milliseconds:
    val date = 304
    val attrTypeName = "description"
    val longDescription = "this is a long description of a thing which may or may not really have a description but here's some text"
    when(mockDB.getEntityName(otherEntityId)).thenReturn(Some(attrTypeName))
    when(mockDB.textAttributeKeyExists(textAttributeId)).thenReturn(true)

    // (using arbitrary numbers for the unnamed parameters):
    val textAttribute = new TextAttribute(mockDB, textAttributeId, entityId, otherEntityId, longDescription, None, date)
    val smallLimit = 35
    val display1: String = textAttribute.getDisplayString(smallLimit, None, None)
    val wholeThing: String = attrTypeName + ": \"" + longDescription + "\"; valid unsp'd, obsv'd Wed 1969-12-31 17:00:00:"+date+" MST"
    val expected:String = wholeThing.substring(0, smallLimit - 3) + "..." // put the real string here instead of dup logic?
    assert(display1 == expected)

    val unlimited=0
    val display2: String = textAttribute.getDisplayString(unlimited, None, None)
    assert(display2 == wholeThing)
  }
}
