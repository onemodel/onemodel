%%
/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003, 2004, 2010, 2011, and 2013-2017 inclusive, Luke A. Call; all rights reserved.
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
import org.scalatest.{Args, FlatSpec, Status}

class EntityTest extends FlatSpec with MockitoSugar {
  // ABOUT the last attempt at CHANGING VARS TO VALS: see comment ("NOTE", farther down) that was removed when the last part of this sentence was added.

  let mut mEntity: Entity = null;
  let mut mUnitId: i64 = 0;
  let mut db: PostgreSQLDatabase = null;
  let mut mQuantityAttrTypeId: i64 = 0;
  let mut mTextAttrTypeId: i64 = 0;
  let mut mDateAttrTypeId = 0L;
  let mut m_booleanAttrTypeId = 0L;
  let mut mFileAttrTypeId = 0L;
  let mut mRelationTypeId = 0L;

  override fn runTests(testName: Option<String>, args: Args) -> Status {
    setUp()
    let result: Status = super.runTests(testName, args);
    // (not calling tearDown: see comment inside PostgreSQLDatabaseTest.runTests about "db setup/teardown")
    result
  }

  protected fn setUp() {
    //start fresh
    PostgreSQLDatabaseTest.tearDownTestDB()

    // instantiation does DB setup (creates tables, default data, etc):
    db = new PostgreSQLDatabase(Database.TEST_USER, Database.TEST_PASS)

    mUnitId = db.create_entity("centimeters")
    mQuantityAttrTypeId = db.create_entity("length")
    mTextAttrTypeId = db.create_entity("someName")
    mDateAttrTypeId = db.create_entity("someName")
    m_booleanAttrTypeId = db.create_entity("someName")
    mFileAttrTypeId = db.create_entity("someName")
    mRelationTypeId = db.createRelationType("someRelationType", "reversedName", "NON")
    let id: i64 = db.create_entity("test object");
    mEntity = new Entity(db, id)
  }

  protected fn tearDown() {
    PostgreSQLDatabaseTest.tearDownTestDB()
  }

  "testAddQuantityAttribute" should "work" in {
    db.begin_trans()
    println!("starting testAddQuantityAttribute")
    let id: i64 = mEntity.addQuantityAttribute(mQuantityAttrTypeId, mUnitId, 100, None).get_id;
    let qo: QuantityAttribute = mEntity.getQuantityAttribute(id);
    if qo == null {
      fail("addQuantityAttribute then getQuantityAttribute returned null")
    }
    assert(qo.get_id == id)
    db.rollback_trans()
  }

  "testAddTextAttribute" should "also work" in {
    db.begin_trans()
    println!("starting testAddTextAttribute")
    let id: i64 = mEntity.addTextAttribute(mTextAttrTypeId, "This is someName given to an object", None).get_id;
    let t: TextAttribute = mEntity.get_textAttribute(id);
    if t == null {
      fail("addTextAttribute then get_textAttribute returned null")
    }
    assert(t.get_id == id)
    db.rollback_trans()
  }

  "testAddDateAttribute" should "also work" in {
    db.begin_trans()
    println!("starting testAddDateAttribute")
    let id: i64 = mEntity.addDateAttribute(mDateAttrTypeId, 2).get_id;
    let t: DateAttribute = mEntity.getDateAttribute(id);
    assert(t != null)
    assert(t.get_id == id)
    assert(t.get_attr_type_id() == mDateAttrTypeId)
    assert(t.getDate == 2)
    db.rollback_trans()
  }

  "testAddBooleanAttribute" should "also work" in {
    db.begin_trans()
    println!("starting testAddBooleanAttribute")
    let startTime = System.currentTimeMillis();
    let id: i64 = mEntity.addBooleanAttribute(m_booleanAttrTypeId, inBoolean = true, None).get_id;
    let t: BooleanAttribute = mEntity.get_boolean_attribute(id);
    assert(t != null)
    assert(t.get_id == id)
    assert(t.get_boolean)
    assert(t.get_parent_id() == mEntity.get_id)
    assert(t.get_valid_on_date().isEmpty)
    assert(t.get_observation_date() > (startTime - 1) && t.get_observation_date() < (System.currentTimeMillis() + 1))
    db.rollback_trans()
  }

  "testAddFileAttribute" should "also work" in {
    db.begin_trans()
    let mut file: java.io.File = null;
    let mut fw: java.io.FileWriter = null;
    println!("starting testAddFileAttribute")
    try {
      file = java.io.File.createTempFile("om-test-file-attr-", null)
      fw = new java.io.FileWriter(file)
      fw.write("1234" + new String("\n"))
      fw.close()
      assert(FileAttribute::md5_hash(file) == "e7df7cd2ca07f4f1ab415d457a6e1c13")
      let path = file.getCanonicalPath;
      let id0: i64 = mEntity.addFileAttribute(mFileAttrTypeId, file).get_id;
      let t0: FileAttribute = mEntity.getFileAttribute(id0);
      assert(t0 != null)
      assert(t0.get_id == id0)
      assert(t0.get_description() == file.get_name)

      let id: i64 = mEntity.addFileAttribute(mFileAttrTypeId, "file desc here, long or short", file).get_id;
      let t: FileAttribute = mEntity.getFileAttribute(id);
      assert(t.get_parent_id() == mEntity.get_id)
      assert(t.get_attr_type_id() == mFileAttrTypeId)
      assert(t.get_description() == "file desc here, long or short")
      assert(t.get_original_file_date() > 1389461364000L)
      let now = System.currentTimeMillis();
      assert(t.get_stored_date() < now && t.get_stored_date() > now - (5 * 1000 * 60))
      assert(t.get_original_file_path() == path)
      assert(t.self.get_readable())
      assert(t.get_writeable())
      assert(!t.get_executable())
      assert(t.get_size == 5)
    }
    finally {
      if fw != null { fw.close() }
      if file != null { file.delete() }
    }
    db.rollback_trans()
  }

  "get_display_string" should "return a useful stack trace string, when called with a nonexistent entity" in {
    // for example, if the entity has been deleted by one part of the code, or one user process in a console window (as an example), and is still
    // referenced and attempted to be displayed by another (or to be somewhat helpful if we try to get info on an entity that's gone due to a bug).
    // (But should this issue go away w/ better design involving more use of immutability or something?)
    let id = 0L;
    let mock_db = mock[PostgreSQLDatabase];
    when(mock_db.entity_key_exists(id)).thenReturn(true)
    when(mock_db.get_entity_data(id)).thenThrow(new RuntimeException("some exception"))
    when(mock_db.get_remote_address).thenReturn(None)
    let entity = new Entity(mock_db, id);
    let se = entity.get_display_string();
    assert(se.contains("Unable to get entity description due to"))
    assert(se.toLowerCase.contains("exception"))
    assert(se.toLowerCase.contains("at org.onemodel"))
  }

  "get_display_string" should "return name & class info" in {
    let id = 0L;
    let classId = 1L;
    let mock_db = mock[PostgreSQLDatabase];
    when(mock_db.entity_key_exists(id)).thenReturn(true)
    when(mock_db.get_class_name(classId)).thenReturn(Some("class1Name"))
    when(mock_db.get_entity_data(id)).thenReturn(Vec<Option<DataType>>(Some("entity1Name"), Some(classId)))
    // idea (is in tracked tasks): put next 3 lines back after color refactoring is done (& places w/ similar comment elsewhere)
    //val entity = new Entity(mock_db, id)
    //val ds = entity.get_display_string
    //assert(ds == "entity1Name (class: class1Name)")

    let id2 = 2L;
    let classId2 = 4L;
    let name2 = "entity2Name";
    let mock_db2 = mock[PostgreSQLDatabase];
    when(mock_db2.entity_key_exists(id2)).thenReturn(true)
    when(mock_db2.get_entity_data(id2)).thenReturn(Vec<Option<DataType>>(Some(name2), None))
    when(mock_db2.get_class_name(classId2)).thenReturn(None)
    // idea (is in tracked tasks): put next lines back after color refactoring is done (& places w/ similar comment elsewhere)
    //val entity2 = new Entity(mock_db2, id2, name2, Some(false), Some(classId2))
    //val ds2 = entity2.get_display_string
    //assert(ds2 == name2)

    when(mock_db2.get_class_name(classId2)).thenReturn(Some("class2Name"))
    when(mock_db2.get_class_count(Some(id2))).thenReturn(1)
    when(mock_db2.get_entity_data(id2)).thenReturn(Vec<Option<DataType>>(Some(name2), Some(classId2)))
    // idea (is in tracked tasks): put next line back after color refactoring is done (& places w/ similar comment elsewhere)
    //assert(entity2.get_display_string == name2 + " (template entity (template) for class: " + "class2Name)")
  }

  "getClassTemplateEntityId" should "work right" in {
    let mock_db = mock[PostgreSQLDatabase];
    let id = 1L;
    let classId = 2L;
    let className = "classname";
    let template_entity_id = 3L;
    when(mock_db.entity_key_exists(id)).thenReturn(true)
    let e = new Entity(mock_db, id, "entityname", None, 0L, Some(true), false, false);
    assert(e.getClassTemplateEntityId.isEmpty)

    let e2 = new Entity(mock_db, id, "entityname", Option(classId), 0L, Some(false), false, false);
    when(mock_db.class_key_exists(classId)).thenReturn(true)
    when(mock_db.get_class_data(classId)).thenReturn(Vec<Option<DataType>>(Some(className), Some(template_entity_id)))
    assert(e2.getClassTemplateEntityId.get == template_entity_id)
  }

  "updateContainedEntitiesPublicStatus" should "work" in {
    let e1Id: i64 = db.create_entity("test object1");
    let e1 = new Entity(db, e1Id);
    mEntity.add_has_relation_to_local_entity(e1.get_id, Some(0), 0)
    let (group: Group, _/*rtg: RelationToGroup*/) = mEntity.addGroupAndRelationToGroup(mRelationTypeId, "grpName",;
                                                                                    allowMixedClassesInGroupIn = true, Some(0), 0, None)
    let e2Id: i64 = db.create_entity("test object2");
    let e2 = new Entity(db, e1Id);
    group.addEntity(e2Id)

    assert(e1.getPublic.isEmpty)
    assert(e2.getPublic.isEmpty)
    mEntity.updateContainedEntitiesPublicStatus(Some(true))
    let e1reRead = new Entity(db, e1Id);
    let e2reRead = new Entity(db, e2Id);
    assert(e1reRead.getPublic.get)
    assert(e2reRead.getPublic.get)
  }

  "getCountOfContainingLocalEntities etc" should "work" in {
    let e1 = Entity.create_entity(db, "e1");
    let (e2id: i64, rteId: i64) = db.create_entityAndRelationToLocalEntity(e1.get_id, mRelationTypeId, "e2", None, None, 0L);
    let e2: Option<Entity> = Entity.getEntity(db, e2id);
    assert(e2.get.getCountOfContainingLocalEntities._1 == 1)
    assert(e2.get.getLocalEntitiesContainingEntity().size == 1)
    /*val (e3id: i64, rte2id: i64) = */db.create_entityAndRelationToLocalEntity(e1.get_id, mRelationTypeId, "e3", None, None, 0L)
    assert(e1.get_adjacent_attributes_sorting_indexes(Database.min_id_value).nonEmpty)
    let nearestSortingIndex = e1.get_nearest_attribute_entrys_sorting_index(Database.min_id_value).get;
    assert(nearestSortingIndex > Database.min_id_value)
    e1.renumber_sorting_indexes()
    let nearestSortingIndex2 = e1.get_nearest_attribute_entrys_sorting_index(Database.min_id_value).get;
    assert(nearestSortingIndex2 > nearestSortingIndex)

    let rte = RelationToLocalEntity.getRelationToLocalEntity(db, rteId).get;
    assert(! e1.is_attribute_sorting_index_in_use(Database.max_id_value))
    e1.update_attribute_sorting_index(rte.get_form_id, rte.get_id, Database.max_id_value)
    assert(e1.getAttributeSortingIndex(rte.get_form_id, rte.get_id) == Database.max_id_value)
    assert(e1.is_attribute_sorting_index_in_use(Database.max_id_value))
    assert(e1.find_unused_attribute_sorting_index() != Database.max_id_value)
    assert(e1.get_relation_to_local_entity_count() == 2)
    e2.get.archive()
    assert(e1.get_relation_to_local_entity_count(include_archived_entities_in = false) == 1)
    assert(e1.get_relation_to_local_entity_count(include_archived_entities_in = true) == 2)
    assert(e1.get_text_attribute_by_type_id(mRelationTypeId).size == 0)
    e1.addTextAttribute(mRelationTypeId, "abc", None)
    assert(e1.get_text_attribute_by_type_id(mRelationTypeId).size == 1)

    assert(Entity.getEntity(db, e1.get_id).get.get_name != "updated")
    e1.updateName("updated")
    assert(Entity.getEntity(db, e1.get_id).get.get_name == "updated")
    assert(Entity.isDuplicate(db, "updated"))
    assert(! Entity.isDuplicate(db, "xyzNOTANAMEupdated"))

    let g1 = Group.create_group(db, "g1");
    g1.addEntity(e1.get_id)
    assert(e1.get_containing_groups_ids.size == 1)
    assert(e1.getCountOfContainingGroups == 1)
    e2.get.addRelationToGroup(mRelationTypeId, g1.get_id, None)
    assert(e1.get_containing_relations_to_group().size == 1)
    assert(e1.get_containing_relation_to_group_descriptions().size == 0)
    e2.get.unarchive()
    assert(e1.get_containing_relation_to_group_descriptions().size == 1)
  }



}
