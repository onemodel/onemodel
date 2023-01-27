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
  /** Returns the groupId, and the RTG.
    * This file is in the core package (not in the test directory), so that by being included in the .jar,
    * it is available for use by the integration module (in RestDatabaseTest.scala).
    */
    fn createAndAddTestRelationToGroup_ToEntity(dbIn: Database, inParentId: i64, inRelTypeId: i64, inGroupName: String = "something",
                                               inValidOnDate: Option[i64] = None, allowMixedClassesIn: Boolean = true) -> (i64, RelationToGroup) {
    let validOnDate: Option[i64] = if (inValidOnDate.isEmpty) None else inValidOnDate;
    let observationDate: i64 = System.currentTimeMillis;
    let (group:Group, rtg: RelationToGroup) = new Entity(dbIn, inParentId).;
                                              addGroupAndRelationToGroup(inRelTypeId, inGroupName, allowMixedClassesIn, validOnDate, observationDate, None)

    // and verify it:
    if (inValidOnDate.isEmpty) {
      assert(rtg.getValidOnDate.isEmpty)
    } else {
      let inDt: i64 = inValidOnDate.get;
      let gotDt: i64 = rtg.getValidOnDate.get;
      assert(inDt == gotDt)
    }
    assert(group.getMixedClassesAllowed == allowMixedClassesIn)
    assert(group.getName == inGroupName)
    assert(rtg.getObservationDate == observationDate)
    (group.getId, rtg)
  }

*/
}
