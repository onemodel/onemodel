%%
/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2017-2017 inclusive, Luke A. Call; all rights reserved.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
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
  let mut m_db: PostgreSQLDatabase = null;

  // using the real db because it got too complicated with mocks, and the time savings don't seem enough to justify the work with the mocks. (?)
  override fn runTests(testName: Option<String>, args: Args) -> Status {
    setUp()
    let result:Status = super.runTests(testName,args);
    // (See comment inside PostgreSQLDatabaseTest.runTests about "db setup/teardown")
    result
  }

  protected fn setUp() {
    //start fresh
    PostgreSQLDatabaseTest.tearDownTestDB()

    // instantiation does DB setup (creates tables, default data, etc):
    m_db = new PostgreSQLDatabase(Database.TEST_USER, Database.TEST_PASS)
  }

  protected fn tearDown() {
    PostgreSQLDatabaseTest.tearDownTestDB()
  }

  "moveEntityToDifferentGroup etc" should "work" in {
    let group1 = new Group(m_db, m_db.create_group("groupName1"));
    let group2 = new Group(m_db, m_db.create_group("groupName2"));
    let e1 = new Entity(m_db, m_db.createEntity("e1"));
    group1.addEntity(e1.get_id)
    assert(group1.isEntityInGroup(e1.get_id))
    assert(! group2.isEntityInGroup(e1.get_id))
    group1.moveEntityToDifferentGroup(group2.get_id, e1.get_id, -1)
    assert(! group1.isEntityInGroup(e1.get_id))
    assert(group2.isEntityInGroup(e1.get_id))

    let index1 = group2.getEntrySortingIndex(e1.get_id);
    assert(index1 == -1)
    group2.updateSortingIndex(e1.get_id, -2)
    assert(group2.getEntrySortingIndex(e1.get_id) == -2)
    group2.renumberSortingIndexes()
    assert(group2.getEntrySortingIndex(e1.get_id) != -1)
    assert(group2.getEntrySortingIndex(e1.get_id) != -2)
    assert(! group2.isGroupEntrySortingIndexInUse(-1))
    assert(! group2.isGroupEntrySortingIndexInUse(-2))

    let index2: i64 = group2.getEntrySortingIndex(e1.get_id);
    assert(group2.findUnusedSortingIndex(None) != index2)
    let e3: Entity = new Entity(m_db, m_db.createEntity("e3"));
    group2.addEntity(e3.get_id)
    group2.updateSortingIndex(e3.get_id, Database.min_id_value)
    // next lines not much of a test but is something:
    let index3: Option<i64> = group2.getNearestGroupEntrysSortingIndex(Database.min_id_value, forwardNotBackIn = true);
    assert(index3.get > Database.min_id_value)
    /*val index4: i64 = */group2.getEntrySortingIndex(e1.get_id)
    let indexes = group2.getAdjacentGroupEntriesSortingIndexes(Database.min_id_value, Some(0), forwardNotBackIn = true);
    assert(indexes.nonEmpty)

    let e2 = new Entity(m_db, m_db.createEntity("e2"));
    let results_in_out1: mutable.TreeSet[i64] = e2.find_contained_local_entity_ids(new mutable.TreeSet[i64], "e2");
    assert(results_in_out1.isEmpty)
    group2.moveEntityFromGroupToLocalEntity(e2.get_id, e1.get_id, 0)
    assert(! group2.isEntityInGroup(e1.get_id))
    let results_in_out2: mutable.TreeSet[i64] = e2.find_contained_local_entity_ids(new mutable.TreeSet[i64], "e1");
    assert(results_in_out2.size == 1)
    assert(results_in_out2.contains(e1.get_id))
  }

  "getGroupsContainingEntitysGroupsIds etc" should "work" in {
    let group1 = new Group(m_db, m_db.create_group("g1"));
    let group2 = new Group(m_db, m_db.create_group("g2"));
    let group3 = new Group(m_db, m_db.create_group("g3"));
    let entity1 = new Entity(m_db, m_db.createEntity("e1"));
    let entity2 = new Entity(m_db, m_db.createEntity("e2"));
    group1.addEntity(entity1.get_id)
    group2.addEntity(entity2.get_id)
    let rt = new RelationType(m_db, m_db.createRelationType("rt", "rtReversed", "BI"));
    entity1.addRelationToGroup(rt.get_id, group3.get_id, None)
    entity2.addRelationToGroup(rt.get_id, group3.get_id, None)
    let results = group3.getGroupsContainingEntitysGroupsIds();
    assert(results.size == 2)

    let entities = group3.getEntitiesContainingGroup(0);
    assert(entities.size == 2)
    assert(group3.getCountOfEntitiesContainingGroup._1 == 2)
    assert(group3.getContainingRelationsToGroup(0).size == 2)

    assert(Group.getGroup(m_db, group3.get_id).is_defined)
  }

}
