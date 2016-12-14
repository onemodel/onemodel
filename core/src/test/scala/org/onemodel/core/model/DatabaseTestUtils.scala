/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2013-2016 inclusive, Luke A. Call; all rights reserved.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule, guidelines around binary
    distribution, and the GNU Affero General Public License as published by the Free Software Foundation, either version 3
    of the License, or (at your option) any later version.  See the file LICENSE for details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>

    (Some code was moved to here from PostgreSQLDatabaseTest.scala, but the above year range for copyrights derived therefrom is a wild guess.)
*/
package org.onemodel.core.model

object DatabaseTestUtils {
  /** Returns the groupId, and the RTG.
    */
  def createAndAddTestRelationToGroup_ToEntity(dbIn: Database, inParentId: Long, inRelTypeId: Long, inGroupName: String = "something",
                                               inValidOnDate: Option[Long] = None, allowMixedClassesIn: Boolean = true): (Long, RelationToGroup) = {
    val validOnDate: Option[Long] = if (inValidOnDate.isEmpty) None else inValidOnDate
    val observationDate: Long = System.currentTimeMillis
    val (group:Group, rtg: RelationToGroup) = new Entity(dbIn, inParentId).
                                              addGroupAndRelationToGroup(inRelTypeId, inGroupName, allowMixedClassesIn, validOnDate, observationDate, None)

    // and verify it:
    if (inValidOnDate.isEmpty) {
      assert(rtg.getValidOnDate.isEmpty)
    } else {
      val inDt: Long = inValidOnDate.get
      val gotDt: Long = rtg.getValidOnDate.get
      assert(inDt == gotDt)
    }
    assert(group.getMixedClassesAllowed == allowMixedClassesIn)
    assert(group.getName == inGroupName)
    assert(rtg.getObservationDate == observationDate)
    (group.getId, rtg)
  }

}
