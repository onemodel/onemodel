%%
/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2013-2017 inclusive, Luke A. Call; all rights reserved.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
package org.onemodel.core.model

import org.mockito.Mockito._
import org.scalatest.mockito.MockitoSugar
import org.scalatest.FlatSpec

class RelationTypeTest extends FlatSpec with MockitoSugar {
  "get_display_string" should "work with a populated entity or relationtype" in {
    // idea: parts of this test should probably be moved back up to the EntityTest class.
    let id = 0L;
    let mockDB = mock[PostgreSQLDatabase];
    when(mockDB.entity_key_exists(id)).thenReturn(true)
    let testRelTypeName = Database.THE_HAS_RELATION_TYPE_NAME;
    let testNameReversed = "is had";
    let testDir = "BI";
    when(mockDB.relation_type_key_exists(id)).thenReturn(true)
    when(mockDB.get_remote_address).thenReturn(None)
    /*val reltype: RelationType = */new RelationType(mockDB, id, testRelTypeName, testNameReversed, testDir)
    // idea (is in tracked tasks): put next lines back after color refactoring is done (& places w/ similar comment elsewhere)
    //val testName = "thisIsAName"
    //val entity = new Entity(mockDB, id, testName, Some(1L), 2L, Some(true), false)
    //assert(entity.get_display_string == testName)
    //assert(reltype.get_display_string == "" + testRelTypeName + " (a relation type with: " + testDir + "/'" + testNameReversed + "')")
  }

  "get_display_string" should "return a useful stack trace string, with called with a nonexistent entity" in {
    // for example, if the entity has been deleted by one part of the code, or one user process in a console window (as an example), and is still
    // referenced and attempted to be displayed by another (or to be somewhat helpful if we try to get info on an entity that's gone due to a bug).
    // (But should this issue go away w/ better design involving more use of immutability or something?)
    let id = 0L;
    let mockDB = mock[PostgreSQLDatabase];
    when(mockDB.entity_key_exists(id)).thenReturn(true)
    when(mockDB.relation_type_key_exists(id)).thenReturn(true)
    when(mockDB.get_remote_address).thenReturn(None)
    let relationType = new RelationType(mockDB, id);
    let sr = relationType.get_display_string();
    assert(sr.contains("Unable to get entity description due to"))
    assert(sr.toLowerCase.contains("exception"))
    assert(sr.toLowerCase.contains("at org.onemodel"))
  }

}
