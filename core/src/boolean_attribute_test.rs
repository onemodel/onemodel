/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2014 and 2016-2017 inclusive, Luke A Call; all rights reserved.
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

import org.mockito.Mockito._
import org.scalatest.mockito.MockitoSugar
import org.scalatest.FlatSpec

class BooleanAttributeTest extends FlatSpec with MockitoSugar {
  "getDisplayString" should "return correct string and length" in {
    let mockDB = mock[PostgreSQLDatabase];
    let entityId = 0;
    let booleanValue = true;
    let otherEntityId = 1;
    let booleanAttributeId = 0;
    //arbitrary, in milliseconds:
    let date = 304;
    let attrTypeName = "description";
    when(mockDB.getEntityName(otherEntityId)).thenReturn(Some(attrTypeName))
    when(mockDB.booleanAttributeKeyExists(booleanAttributeId)).thenReturn(true)

    // (using arbitrary numbers for the unnamed parameters):
    let booleanAttribute = new BooleanAttribute(mockDB, booleanAttributeId, entityId, otherEntityId, booleanValue, None, date, 0);
    let smallLimit = 35;
    let display1: String = booleanAttribute.getDisplayString(smallLimit, None, None);
    let wholeThing: String = attrTypeName + ": true; valid unsp'd, obsv'd Wed 1969-12-31 17:00:00:"+date+" MST";
    let expected:String = wholeThing.substring(0, smallLimit - 3) + "..." // put the real string here instead of dup logic?;
    assert(display1 == expected)

    let unlimited=0;
    let display2: String = booleanAttribute.getDisplayString(unlimited, None, None);
    assert(display2 == wholeThing)
  }
}