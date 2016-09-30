/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2013-2016 inclusive, Luke A Call; all rights reserved.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule, guidelines around binary
    distribution, and the GNU Affero General Public License as published by the Free Software Foundation, either version 3
    of the License, or (at your option) any later version.  See the file LICENSE for details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
package org.onemodel.core

import org.scalatest.FlatSpec
import org.mockito.Mockito._
import org.scalatest.mock.MockitoSugar
import org.onemodel.core.model.{RelationType, RelationToEntity, Entity}
import org.onemodel.core.database.PostgreSQLDatabase

class RelationToEntityTest extends FlatSpec with MockitoSugar {
  "getDisplayString" should "return correct string and length" in {
    val mockDB = mock[PostgreSQLDatabase]
    val mockRelationType = mock[RelationType]
    val mockEntity1 = mock[Entity]
    val mockEntity2 = mock[Entity]

    val rteId: Long = 101
    val entity1Id: Long = 102
    val entity2Id: Long = 103
    val relTypeId: Long = 401
    //arbitrary, in milliseconds:
    val date = 304
    val entity1Name = "husbandName"
    val entity2Name = "wifeName"
    when(mockEntity2.getId).thenReturn(entity2Id)
    val relationTypeName: String = "is husband to"
    when(mockRelationType.getName).thenReturn(relationTypeName)
    when(mockRelationType.getId).thenReturn(relTypeId)
    when(mockEntity1.getId).thenReturn(entity1Id)
    when(mockEntity1.getName).thenReturn(entity1Name)
    when(mockEntity2.getName).thenReturn(entity2Name)
    when(mockDB.relationToEntityKeysExistAndMatch(rteId, relTypeId, entity1Id, entity2Id)).thenReturn(true)

    // (using arbitrary numbers for the unnamed parameters):
    val relation = new RelationToEntity(mockDB, rteId, relTypeId, entity1Id, entity2Id, None, date, 0)
    val smallLimit = 15
    val displayed1: String = relation.getDisplayString(smallLimit, Some(mockEntity2), Some(mockRelationType))
    val observedDateOutput = "Wed 1969-12-31 17:00:00:"+date+" MST"
    val wholeThing: String = relationTypeName + ": " + entity2Name + "; valid unsp'd, obsv'd "+observedDateOutput
    val expected = wholeThing.substring(0, smallLimit - 3) + "..."
    assert(displayed1 == expected)

    // the next part passes in intellij 12, but not from the cli as "mvn test". Maybe due to some ignorance
    // about mockito.  It gets this NPE, but unclear why it calls the method given the mock. Here's the failure description
            /*should return correct string and length *** FAILED ***
              java.lang.NullPointerException:
              at org.onemodel.RelationType.readDataFromDB(RelationType.java:63)
              at org.onemodel.RelationTest$$anonfun$1.apply$mcV$sp(RelationTest.scala:41)
              at org.onemodel.RelationTest$$anonfun$1.apply(RelationTest.scala:12)
              at org.onemodel.RelationTest$$anonfun$1.apply(RelationTest.scala:12)
            */
    //val relationTypeNameInReverseDirection: String = "is wife to"
    //when(mockRelationType.getNameInReverseDirection).thenReturn(relationTypeNameInReverseDirection)
    ////when(mockRelationType.readDataFromDB()).thenReturn()
    //val displayed2: String = relation.getDisplayString(0, Some(mockEntity1), Some(mockRelationType))
    //val expected2:String = relationTypeNameInReverseDirection + ": " + entity1Name + "; valid unsp'd, obsv'd "+observedDateOutput
    //assert(displayed2 == expected2)
  }
}
