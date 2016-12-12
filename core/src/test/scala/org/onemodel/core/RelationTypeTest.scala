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
import org.onemodel.core.model.{Database, PostgreSQLDatabase, RelationType}

class RelationTypeTest extends FlatSpec with MockitoSugar {
  "getDisplayString" should "work with a populated entity or relationtype" in {
    // idea: parts of this test should probably be moved back up to the EntityTest class.
    val id = 0L
    val mockDB = mock[PostgreSQLDatabase]
    when(mockDB.entityKeyExists(id)).thenReturn(true)
    val testRelTypeName = Database.theHASrelationTypeName
    val testNameReversed = "is had"
    val testDir = "BI"
    when(mockDB.relationTypeKeyExists(id)).thenReturn(true)
    val reltype: RelationType = new RelationType(mockDB, id, testRelTypeName, testNameReversed, testDir)
    // idea (is in tracked tasks): put next lines back after color refactoring is done (& places w/ similar comment elsewhere)
    //val testName = "thisIsAName"
    //val entity = new Entity(mockDB, id, testName, Some(1L), 2L, Some(true), false)
    //assert(entity.getDisplayString == testName)
    //assert(reltype.getDisplayString == "" + testRelTypeName + " (a relation type with: " + testDir + "/'" + testNameReversed + "')")
  }

  "getDisplayString" should "return a useful stack trace string, with called with a nonexistent entity" in {
    // for example, if the entity has been deleted by one part of the code, or one user process in a console window (as an example), and is still
    // referenced and attempted to be displayed by another (or to be somewhat helpful if we try to get info on an entity that's gone due to a bug).
    // (But should this issue go away w/ better design involving more use of immutability or something?)
    val id = 0L
    val mockDB = mock[PostgreSQLDatabase]
    when(mockDB.entityKeyExists(id)).thenReturn(true)
    when(mockDB.relationTypeKeyExists(id)).thenReturn(true)
    val relationType = new RelationType(mockDB, id)
    val sr = relationType.getDisplayString()
    assert(sr.contains("Unable to get entity description due to"))
    assert(sr.toLowerCase.contains("exception"))
    assert(sr.toLowerCase.contains("at org.onemodel"))
  }

}
