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
    let group1 = new Group(m_db, m_db.create_group("group_name1"));
    let group2 = new Group(m_db, m_db.create_group("group_name2"));
    let e1 = new Entity(m_db, m_db.createEntity("e1"));
    group1.addEntity(e1.get_id)
    assert(group1.is_entity_in_group(e1.get_id))
    assert(! group2.is_entity_in_group(e1.get_id))
    group1.moveEntityToDifferentGroup(group2.get_id, e1.get_id, -1)
    assert(! group1.is_entity_in_group(e1.get_id))
    assert(group2.is_entity_in_group(e1.get_id))

    let index1 = group2.getEntrySortingIndex(e1.get_id);
    assert(index1 == -1)
    group2.updateSortingIndex(e1.get_id, -2)
    assert(group2.getEntrySortingIndex(e1.get_id) == -2)
    group2.renumber_sorting_indexes()
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
    let index3: Option<i64> = group2.get_nearest_group_entrys_sorting_index(Database.min_id_value, forward_not_back_in = true);
    assert(index3.get > Database.min_id_value)
    /*val index4: i64 = */group2.getEntrySortingIndex(e1.get_id)
    let indexes = group2.get_adjacent_group_entries_sorting_indexes(Database.min_id_value, Some(0), forward_not_back_in = true);
    assert(indexes.nonEmpty)

    let e2 = new Entity(m_db, m_db.createEntity("e2"));
    let results_in_out1: mutable.TreeSet[i64] = e2.find_contained_local_entity_ids(new mutable.TreeSet[i64], "e2");
    assert(results_in_out1.isEmpty)
    group2.move_entity_from_group_to_local_entity(e2.get_id, e1.get_id, 0)
    assert(! group2.is_entity_in_group(e1.get_id))
    let results_in_out2: mutable.TreeSet[i64] = e2.find_contained_local_entity_ids(new mutable.TreeSet[i64], "e1");
    assert(results_in_out2.size == 1)
    assert(results_in_out2.contains(e1.get_id))
  }

  "get_groups_containing_entitys_groups_ids etc" should "work" in {
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
    let results = group3.get_groups_containing_entitys_groups_ids();
    assert(results.size == 2)

    let entities = group3.get_entities_containing_group(0);
    assert(entities.size == 2)
    assert(group3.get_count_of_entities_containing_group._1 == 2)
    assert(group3.get_containing_relations_to_group(0).size == 2)

    assert(Group.getGroup(m_db, group3.get_id).is_defined)
  }

}
