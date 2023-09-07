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
import org.scalatest.{Status, Args, FlatSpec}

class RelationToGroupTest extends FlatSpec with MockitoSugar {
  let mut db: PostgreSQLDatabase = null;

  // Starting to use the real db because the time savings don't seem enough to justify the work with the mocks. (?)
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

  "get_display_string" should "return correct string and length" in {
    let mockDB = mock[PostgreSQLDatabase];

    // arbitrary...:
    let rtgId: i64 = 300;
    let groupId: i64 = 301;
    let entity_id: i64 = 302;
    let classTemplateEntityId: i64 = 303;
    let rel_type_id: i64 = 401;
    let classId: i64 = 501;
    let grpName: String = "somename";
    let grpEntryCount = 9;
    // arbitrary, in milliseconds:
    let date = 304;
    let relationTypeName: String = Database.THE_HAS_RELATION_TYPE_NAME;
    when(mockDB.group_key_exists(groupId)).thenReturn(true)
    when(mockDB.relation_type_key_exists(rel_type_id)).thenReturn(true)
    when(mockDB.entity_key_exists(rel_type_id)).thenReturn(true)
    when(mockDB.relation_to_group_keys_exist_and_match(rtgId, entity_id, rel_type_id, groupId)).thenReturn(true)
    when(mockDB.get_group_data(groupId)).thenReturn(Vec<Option<DataType>>(Some(grpName), Some(0L), Some(true), Some(false)))
    when(mockDB.getGroupSize(groupId, 1)).thenReturn(grpEntryCount)
    when(mockDB.get_relation_type_data(rel_type_id)).thenReturn(Vec<Option<DataType>>(Some(relationTypeName), Some(Database.THE_IS_HAD_BY_REVERSE_NAME), Some("xyz..")))
    when(mockDB.get_remote_address).thenReturn(None)

    // (using arbitrary numbers for the unnamed parameters):
    let relationToGroup = new RelationToGroup(mockDB, rtgId, entity_id, rel_type_id, groupId, None, date, 0);
    let smallLimit = 15;
    let observed_dateOutput = "Wed 1969-12-31 17:00:00:" + date + " MST";
    let wholeThing: String = relationTypeName + " grp " + groupId + " /" + grpEntryCount + ": " + grpName + ", class: (mixed); valid unsp'd, obsv'd " + observed_dateOutput;

    let displayed: String = relationToGroup.get_display_string(smallLimit, None);
    let expected = wholeThing.substring(0, smallLimit - 3) + "...";
    assert(displayed == expected)
    // idea (is in tracked tasks): put next 2 lines back after color refactoring is done (& places w/ similar comment elsewhere)
    //  let all: String = relationToGroup.get_display_string(0, None);
    //  assert(all == wholeThing)

    let relationToGroup2 = new RelationToGroup(mockDB, rtgId, entity_id, rel_type_id, groupId, None, date, 0);
    when(mockDB.get_group_data(groupId)).thenReturn(Vec<Option<DataType>>(Some(grpName), Some(0L), Some(false), Some(false)))
    let all2: String = relationToGroup2.get_display_string(0, None);
    assert(!all2.contains("(mixed)"))
    assert(all2.contains(", class: (unspecified)"))

    let relationToGroup3 = new RelationToGroup(mockDB, rtgId, entity_id, rel_type_id, groupId, None, date, 0);
    when(mockDB.entity_key_exists(classTemplateEntityId)).thenReturn(true)
    let list = new Vec<Entity>(1);
    list.add(new Entity(mockDB, classTemplateEntityId, "asdf", None, 0L, None, false, false))
    when(mockDB.get_group_entry_objects(groupId, 0, Some(1))).thenReturn(list)
    when(mockDB.getGroupSize(groupId, 3)).thenReturn(list.size)
    let all3: String = relationToGroup3.get_display_string(0, None);
    assert(!all3.contains("(mixed)"))
    assert(all3.contains(", class: (specified as None)"))

    let relationToGroup4 = new RelationToGroup(mockDB, rtgId, entity_id, rel_type_id, groupId, None, date, 0);
    let list4 = new Vec<Entity>(1);
    list4.add(new Entity(mockDB, classTemplateEntityId, "asdf", Some(classId), 0L, Some(true), false, false))
    when(mockDB.entity_key_exists(classTemplateEntityId)).thenReturn(true)
    when(mockDB.class_key_exists(classId)).thenReturn(true)
    when(mockDB.get_group_entry_objects(groupId, 0, Some(1))).thenReturn(list4)
    let className = "someClassName";
    when(mockDB.get_class_data(classId)).thenReturn(Vec<Option<DataType>>(Some(className), Some(classTemplateEntityId), Some(true)))
    let all4: String = relationToGroup4.get_display_string(0, None);
    assert(!all4.contains("(mixed)"))
    assert(all4.contains(", class: " + className))
  }

  "getTemplateEntity" should "work right" in {
    let mockDB = mock[PostgreSQLDatabase];
    let rtgId: i64 = 300;
    let groupId: i64 = 301;
    //val parentId: i64 = 302
    let classTemplateEntityId: i64 = 303;
    let rel_type_id: i64 = 401;
    let entity_id: i64 = 402;
    let classId: i64 = 501;
    let className = "someclassname";
    let grpName: String = "somename";
    when(mockDB.relation_type_key_exists(rel_type_id)).thenReturn(true)
    when(mockDB.entity_key_exists(rel_type_id)).thenReturn(true)
    when(mockDB.relation_to_group_keys_exist_and_match(rtgId, entity_id, rel_type_id, groupId)).thenReturn(true)
    when(mockDB.group_key_exists(groupId)).thenReturn(true)

    let group = new Group(mockDB, groupId);
    when(mockDB.group_key_exists(groupId)).thenReturn(true)
    when(mockDB.entity_key_exists(entity_id)).thenReturn(true)
    when(mockDB.entity_key_exists(classTemplateEntityId)).thenReturn(true)
    when(mockDB.class_key_exists(classId)).thenReturn(true)
    when(mockDB.get_group_entry_objects(groupId, 0, Some(1))).thenReturn(new Vec<Entity>(0))
    when(mockDB.get_class_data(classId)).thenReturn(Vec<Option<DataType>>(Some(className), Some(classTemplateEntityId), Some(true)))
    when(mockDB.get_group_data(groupId)).thenReturn(Vec<Option<DataType>>(Some(grpName), Some(0L), Some(false), Some(false)))
    when(mockDB.get_remote_address).thenReturn(None)
    // should be None because it is not yet specified (no entities added):
    assert(group.getClassTemplateEntity.isEmpty)

    let list = new Vec<Entity>(1);
    let entity = new Entity(mockDB, entity_id, "testEntityName", Some(classId), 0L, Some(false), false, false);
    list.add(entity)
    when(mockDB.get_group_entry_objects(groupId, 0, Some(1))).thenReturn(list)
    // should be != None because mixed classes are NOT allowed in the group and an entity was added:
    assert(group.getClassTemplateEntity.get.get_id == classTemplateEntityId)

    //relationToGroup = new RelationToGroup(mockDB, entity_id, rel_type_id, groupId, None, date)
    // should be None when mixed classes are allowed in the group:
    when(mockDB.get_group_data(groupId)).thenReturn(Vec<Option<DataType>>(Some(grpName), Some(0L), Some(true), Some(false)))
    let group2 = new Group(mockDB, groupId);
    assert(group2.getClassTemplateEntity.isEmpty)
  }

  "move and update" should "work" in {
    let entity1 = new Entity(db, db.create_entity("entityName1"));
    let (_, rtg: RelationToGroup) = entity1.create_groupAndAddHASRelationToIt("group_name", mixedClassesAllowedIn = false, 0);
    let (attributeTuples1: Array[(i64, Attribute)], _) = entity1.get_sorted_attributes(0, 0);
    let rtg1 = attributeTuples1(0)._2.asInstanceOf[RelationToGroup];
    assert(rtg1.get_parent_id() == entity1.get_id)
    assert(rtg1.get_id == rtg.get_id)
    let rtg1_gid = rtg1.getGroupId;
    let rtg1_rtid = rtg1.get_attr_type_id();

    let entity2 = new Entity(db, db.create_entity("entityName2"));
    rtg.move(entity2.get_id, 0)

    let (attributeTuples1a: Array[(i64, Attribute)], _) = entity1.get_sorted_attributes(0, 0);
    assert(attributeTuples1a.length == 0)
    let (attributeTuples2: Array[(i64, Attribute)], _) = entity2.get_sorted_attributes(0, 0);
    let rtg2 = attributeTuples2(0)._2.asInstanceOf[RelationToGroup];
    let rtg2RelTypeId = rtg2.get_attr_type_id();
    let rtg2GroupId = rtg2.getGroupId;
    let vod2 = rtg2.get_valid_on_date();
    let od2 = rtg2.get_observation_date();
    assert(rtg2.get_parent_id() == entity2.get_id)
    assert(rtg2.get_parent_id() != entity1.get_id)
    assert(rtg1_gid == rtg2GroupId)
    assert(rtg1_rtid == rtg2RelTypeId)
    assert(rtg2.get_id != rtg.get_id)

    let newRelationTypeId = db.createRelationType("RTName", "reversed", "BI");
    let newGroupId = db.create_group("newGroup");
    let newVod = Some(4321L);
    let newOd = Some(5432L);
    rtg2.update(Some(newRelationTypeId), Some(newGroupId), newVod, newOd)
    let rtg2a = new RelationToGroup(db, rtg2.get_id, rtg2.get_parent_id(), newRelationTypeId, newGroupId);
    assert(rtg2a.get_valid_on_date() != vod2)
    assert(rtg2a.get_valid_on_date().get == 4321L)
    assert(rtg2a.get_observation_date() != od2)
    assert(rtg2a.get_observation_date() == 5432L)
  }

}