/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2017-2017 inclusive, Luke A Call; all rights reserved.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule, guidelines around binary
    distribution, and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
package org.onemodel.core.model

import org.scalatest.mockito.MockitoSugar
import org.scalatest.{Args, FlatSpec, Status}

import scala.collection.mutable

class GroupTest extends FlatSpec with MockitoSugar {
  var mDB: PostgreSQLDatabase = null

  // using the real db because it got too complicated with mocks, and the time savings don't seem enough to justify the work with the mocks. (?)
  override def runTests(testName: Option[String], args: Args):Status = {
    setUp()
    val result:Status = super.runTests(testName,args)
    // (See comment inside PostgreSQLDatabaseTest.runTests about "db setup/teardown")
    result
  }

  protected def setUp() {
    //start fresh
    PostgreSQLDatabaseTest.tearDownTestDB()

    // instantiation does DB setup (creates tables, default data, etc):
    mDB = new PostgreSQLDatabase(Database.TEST_USER, Database.TEST_PASS)
  }

  protected def tearDown() {
    PostgreSQLDatabaseTest.tearDownTestDB()
  }

  "moveEntityToDifferentGroup etc" should "work" in {
    val group1 = new Group(mDB, mDB.createGroup("groupName1"))
    val group2 = new Group(mDB, mDB.createGroup("groupName2"))
    val e1 = new Entity(mDB, mDB.createEntity("e1"))
    group1.addEntity(e1.getId)
    assert(group1.isEntityInGroup(e1.getId))
    assert(! group2.isEntityInGroup(e1.getId))
    group1.moveEntityToDifferentGroup(group2.getId, e1.getId, -1)
    assert(! group1.isEntityInGroup(e1.getId))
    assert(group2.isEntityInGroup(e1.getId))

    val index1 = group2.getEntrySortingIndex(e1.getId)
    assert(index1 == -1)
    group2.updateSortingIndex(e1.getId, -2)
    assert(group2.getEntrySortingIndex(e1.getId) == -2)
    group2.renumberSortingIndexes()
    assert(group2.getEntrySortingIndex(e1.getId) != -1)
    assert(group2.getEntrySortingIndex(e1.getId) != -2)
    assert(! group2.isGroupEntrySortingIndexInUse(-1))
    assert(! group2.isGroupEntrySortingIndexInUse(-2))

    val index2: Long = group2.getEntrySortingIndex(e1.getId)
    assert(group2.findUnusedSortingIndex(None) != index2)
    val e3: Entity = new Entity(mDB, mDB.createEntity("e3"))
    group2.addEntity(e3.getId)
    group2.updateSortingIndex(e3.getId, Database.minIdValue)
    // next lines not much of a test but is something:
    val index3: Option[Long] = group2.getNearestGroupEntrysSortingIndex(Database.minIdValue, forwardNotBackIn = true)
    assert(index3.get > Database.minIdValue)
    /*val index4: Long = */group2.getEntrySortingIndex(e1.getId)
    val indexes = group2.getAdjacentGroupEntriesSortingIndexes(Database.minIdValue, Some(0), forwardNotBackIn = true)
    assert(indexes.nonEmpty)

    val e2 = new Entity(mDB, mDB.createEntity("e2"))
    val resultsInOut1: mutable.TreeSet[Long] = e2.findContainedLocalEntityIds(new mutable.TreeSet[Long], "e2")
    assert(resultsInOut1.isEmpty)
    group2.moveEntityFromGroupToLocalEntity(e2.getId, e1.getId, 0)
    assert(! group2.isEntityInGroup(e1.getId))
    val resultsInOut2: mutable.TreeSet[Long] = e2.findContainedLocalEntityIds(new mutable.TreeSet[Long], "e1")
    assert(resultsInOut2.size == 1)
    assert(resultsInOut2.contains(e1.getId))
  }

  "getGroupsContainingEntitysGroupsIds etc" should "work" in {
    val group1 = new Group(mDB, mDB.createGroup("g1"))
    val group2 = new Group(mDB, mDB.createGroup("g2"))
    val group3 = new Group(mDB, mDB.createGroup("g3"))
    val entity1 = new Entity(mDB, mDB.createEntity("e1"))
    val entity2 = new Entity(mDB, mDB.createEntity("e2"))
    group1.addEntity(entity1.getId)
    group2.addEntity(entity2.getId)
    val rt = new RelationType(mDB, mDB.createRelationType("rt", "rtReversed", "BI"))
    entity1.addRelationToGroup(rt.getId, group3.getId, None)
    entity2.addRelationToGroup(rt.getId, group3.getId, None)
    val results = group3.getGroupsContainingEntitysGroupsIds()
    assert(results.size == 2)

    val entities = group3.getEntitiesContainingGroup(0)
    assert(entities.size == 2)
    assert(group3.getCountOfEntitiesContainingGroup._1 == 2)
    assert(group3.getContainingRelationsToGroup(0).size == 2)

    assert(Group.getGroup(mDB, group3.getId).isDefined)
  }

}
