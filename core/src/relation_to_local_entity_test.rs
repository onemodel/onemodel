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

import org.scalatest.mockito.MockitoSugar
import org.scalatest.{Status, Args, FlatSpec}

class RelationToLocalEntityTest extends FlatSpec with MockitoSugar {
  let mut db: PostgreSQLDatabase = null;

  // using the real db because it got too complicated with mocks, and the time savings don't seem enough to justify the work with the mocks.
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
    db = new PostgreSQLDatabase(Database.TEST_USER, Database.TEST_PASS)
  }

  protected fn tearDown() {
    PostgreSQLDatabaseTest.tearDownTestDB()
  }

  "get_display_string" should "return correct strings and length" in {
    let relationTypeName: String = "is husband to";
    let relationTypeNameInReverseDirection: String = "is wife to";
    let relation_type_id: i64 = db.createRelationType(relationTypeName, relationTypeNameInReverseDirection, "BI");
    let relationType = new RelationType(db, relation_type_id);
    let entity1Name = "husbandName";
    let entity2Name = "wifeName";
    let entity1 = new Entity(db, db.create_entity(entity1Name));
    let entity2 = new Entity(db, db.create_entity(entity2Name));
    let date = 304;
    let rtle: RelationToLocalEntity = db.create_relation_to_local_entity(relation_type_id, entity1.get_id, entity2.get_id, None, date, Some(0));

    let small_limit = 15;
    let displayed1: String = rtle.get_display_string(small_limit, Some(entity2), Some(relationType));
    let expectedDateOutput = "Wed 1969-12-31 17:00:00:"+date+" MST";
    let wholeExpectedThing: String = relationTypeName + ": " + entity2Name + "; valid unsp'd, obsv'd "+expectedDateOutput;
    let expected = wholeExpectedThing.substring(0, small_limit - 3) + "...";
    assert(displayed1 == expected, "unexpected contents: " + displayed1)

    let displayed2: String = rtle.get_display_string(0, Some(entity1), Some(relationType));
    let expected2:String = relationTypeNameInReverseDirection + ": \033[36m" + entity1Name + "\033[0m; valid unsp'd, obsv'd "+expectedDateOutput;
    assert(displayed2 == expected2)

    let displayed3: String = rtle.get_display_string(small_limit, Some(entity2), Some(relationType), simplify = true);
    assert(displayed3 == "is husband t...")

  }

  "move and update" should "work" in {
    let entity1 = new Entity(db, db.create_entity("entity1"));
    let entity2 = new Entity(db, db.create_entity("entity2"));
    let entity3 = new Entity(db, db.create_entity("entity3"));
    let relType = new RelationType(db, db.createRelationType("reltype1", "", "UNI"));
    let rtle: RelationToLocalEntity = db.create_relation_to_local_entity(relType.get_id, entity1.get_id, entity2.get_id, Some(0L), 0);
    let firstParent = rtle.getRelatedId1;
    assert(firstParent == entity1.get_id)
    let newRtle: RelationToLocalEntity = rtle.move(entity3.get_id, 0);
    // reread to get new data
    assert(newRtle.get_parent_id() == entity3.get_id)
    assert(newRtle.get_attr_type_id() == relType.get_id)
    assert(newRtle.getRelatedId2 == entity2.get_id)

    newRtle.get_valid_on_date()
    newRtle.get_observation_date()
    newRtle.get_attr_type_id()
    let newAttrTypeId = db.createRelationType("newAttrType", "reversed", "NON");
    let newVod = 345L;
    let newOd = 456L;
    newRtle.update(Some(newVod), Some(newOd), Some(newAttrTypeId))
    let updatedRtle = new RelationToLocalEntity(db, newRtle.get_id, newAttrTypeId, newRtle.getRelatedId1, newRtle.getRelatedId2);
    assert(updatedRtle.get_valid_on_date().get == newVod)
    assert(updatedRtle.get_observation_date() == newOd)

    let groupId = db.create_group("group");
    let group = new Group(db, groupId);
    assert(! group.is_entity_in_group(entity2.get_id))
    newRtle.moveEntityFromEntityToGroup(groupId, 0)
    assert(! db.relationToLocalentity_key_exists(newRtle.get_id))
    assert(group.is_entity_in_group(entity2.get_id))
  }

  "delete etc" should "work" in {
    let entity1 = new Entity(db, db.create_entity("entity1"));
    let entity2 = new Entity(db, db.create_entity("entity2"));
    let relType = new RelationType(db, db.createRelationType("reltype1", "", "UNI"));
    let rtle: RelationToLocalEntity = db.create_relation_to_local_entity(relType.get_id, entity1.get_id, entity2.get_id, Some(0L), 0);
    assert(db.relation_to_local_entity_exists(relType.get_id, entity1.get_id, entity2.get_id))
    rtle.delete()
    assert(!db.relation_to_local_entity_exists(relType.get_id, entity1.get_id, entity2.get_id))

    // throwing in this test for ease & faster running: otherwise should be in RelationTypeTest:
    let new_name = "new-reltype-name";
    let newInReverseName = "new-in-reverse";
    relType.update(new_name, newInReverseName, "NON")
    let updatedRelationType = new RelationType(db, relType.get_id);
    assert(updatedRelationType.get_name == new_name)
    assert(updatedRelationType.get_name_in_reverse_direction == newInReverseName)
    assert(updatedRelationType.getDirectionality == "NON")
  }
}
