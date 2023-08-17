/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2013-2017 inclusive, and 2023, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>

    (Some code was moved to here from PostgreSQLDatabaseTest.scala, but the above year range for copyrights derived therefrom is a wild guess.)
*/
struct DatabaseTestUtils {
/*%%
package org.onemodel.core.model

object DatabaseTestUtils {
  /** Returns the group_id, and the RTG.
    * This file is in the core package (not in the test directory), so that by being included in the .jar,
    * it is available for use by the integration module (in RestDatabaseTest.scala).
    */
    fn createAndAddTestRelationToGroup_ToEntity(dbIn: Database, in_parent_id: i64, in_rel_type_id: i64, inGroupName: String = "something",
                                               in_valid_on_date: Option<i64> = None, allowMixedClassesIn: bool = true) -> (i64, RelationToGroup) {
    let valid_on_date: Option<i64> = if in_valid_on_date.isEmpty) None else in_valid_on_date;
    let observation_date: i64 = System.currentTimeMillis;
    let (group:Group, rtg: RelationToGroup) = new Entity(dbIn, in_parent_id).;
                                              addGroupAndRelationToGroup(in_rel_type_id, inGroupName, allowMixedClassesIn, valid_on_date, observation_date, None)

    // and verify it:
    if in_valid_on_date.isEmpty) {
      assert(rtg.get_valid_on_date().isEmpty)
    } else {
      let inDt: i64 = in_valid_on_date.get;
      let gotDt: i64 = rtg.get_valid_on_date().get;
      assert(inDt == gotDt)
    }
    assert(group.getMixedClassesAllowed == allowMixedClassesIn)
    assert(group.get_name == inGroupName)
    assert(rtg.get_observation_date() == observation_date)
    (group.get_id, rtg)
  }

*/
}
