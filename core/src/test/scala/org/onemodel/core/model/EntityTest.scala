/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003, 2004, 2010, 2011, and 2013-2017 inclusive, Luke A Call; all rights reserved.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule, guidelines around binary
    distribution, and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>

  ---------------------------------------------------
  (See comment in this place in PostgreSQLDatabase.scala about possible alternatives to this use of the db via this layer and jdbc.)

*/
package org.onemodel.core.model

import org.mockito.Mockito._
import org.scalatest.mockito.MockitoSugar
import org.scalatest.{Args, FlatSpec, Status}

class EntityTest extends FlatSpec with MockitoSugar {
  // ABOUT the last attempt at CHANGING VARS TO VALS: see comment ("NOTE", farther down) that was removed when the last part of this sentence was added.

  var mEntity: Entity = null
  var mUnitId: Long = 0
  var mDB: PostgreSQLDatabase = null
  var mQuantityAttrTypeId: Long = 0
  var mTextAttrTypeId: Long = 0
  var mDateAttrTypeId = 0L
  var mBooleanAttrTypeId = 0L
  var mFileAttrTypeId = 0L
  var mRelationTypeId = 0L

  override def runTests(testName: Option[String], args: Args): Status = {
    setUp()
    val result: Status = super.runTests(testName, args)
    // (not calling tearDown: see comment inside PostgreSQLDatabaseTest.runTests about "db setup/teardown")
    result
  }

  protected def setUp() {
    //start fresh
    PostgreSQLDatabaseTest.tearDownTestDB()

    // instantiation does DB setup (creates tables, default data, etc):
    mDB = new PostgreSQLDatabase(Database.TEST_USER, Database.TEST_PASS)

    mUnitId = mDB.createEntity("centimeters")
    mQuantityAttrTypeId = mDB.createEntity("length")
    mTextAttrTypeId = mDB.createEntity("someName")
    mDateAttrTypeId = mDB.createEntity("someName")
    mBooleanAttrTypeId = mDB.createEntity("someName")
    mFileAttrTypeId = mDB.createEntity("someName")
    mRelationTypeId = mDB.createRelationType("someRelationType", "reversedName", "NON")
    val id: Long = mDB.createEntity("test object")
    mEntity = new Entity(mDB, id)
  }

  protected def tearDown() {
    PostgreSQLDatabaseTest.tearDownTestDB()
  }

  "testAddQuantityAttribute" should "work" in {
    mDB.beginTrans()
    System.out.println("starting testAddQuantityAttribute")
    val id: Long = mEntity.addQuantityAttribute(mQuantityAttrTypeId, mUnitId, 100, None).getId
    val qo: QuantityAttribute = mEntity.getQuantityAttribute(id)
    if (qo == null) {
      fail("addQuantityAttribute then getQuantityAttribute returned null")
    }
    assert(qo.getId == id)
    mDB.rollbackTrans()
  }

  "testAddTextAttribute" should "also work" in {
    mDB.beginTrans()
    System.out.println("starting testAddTextAttribute")
    val id: Long = mEntity.addTextAttribute(mTextAttrTypeId, "This is someName given to an object", None).getId
    val t: TextAttribute = mEntity.getTextAttribute(id)
    if (t == null) {
      fail("addTextAttribute then getTextAttribute returned null")
    }
    assert(t.getId == id)
    mDB.rollbackTrans()
  }

  "testAddDateAttribute" should "also work" in {
    mDB.beginTrans()
    System.out.println("starting testAddDateAttribute")
    val id: Long = mEntity.addDateAttribute(mDateAttrTypeId, 2).getId
    val t: DateAttribute = mEntity.getDateAttribute(id)
    assert(t != null)
    assert(t.getId == id)
    assert(t.getAttrTypeId == mDateAttrTypeId)
    assert(t.getDate == 2)
    mDB.rollbackTrans()
  }

  "testAddBooleanAttribute" should "also work" in {
    mDB.beginTrans()
    System.out.println("starting testAddBooleanAttribute")
    val startTime = System.currentTimeMillis()
    val id: Long = mEntity.addBooleanAttribute(mBooleanAttrTypeId, inBoolean = true, None).getId
    val t: BooleanAttribute = mEntity.getBooleanAttribute(id)
    assert(t != null)
    assert(t.getId == id)
    assert(t.getBoolean)
    assert(t.getParentId == mEntity.getId)
    assert(t.getValidOnDate.isEmpty)
    assert(t.getObservationDate > (startTime - 1) && t.getObservationDate < (System.currentTimeMillis() + 1))
    mDB.rollbackTrans()
  }

  "testAddFileAttribute" should "also work" in {
    mDB.beginTrans()
    var file: java.io.File = null
    var fw: java.io.FileWriter = null
    System.out.println("starting testAddFileAttribute")
    try {
      file = java.io.File.createTempFile("om-test-file-attr-", null)
      fw = new java.io.FileWriter(file)
      fw.write("1234" + new String("\n"))
      fw.close()
      assert(FileAttribute.md5Hash(file) == "e7df7cd2ca07f4f1ab415d457a6e1c13")
      val path = file.getCanonicalPath
      val id0: Long = mEntity.addFileAttribute(mFileAttrTypeId, file).getId
      val t0: FileAttribute = mEntity.getFileAttribute(id0)
      assert(t0 != null)
      assert(t0.getId == id0)
      assert(t0.getDescription == file.getName)

      val id: Long = mEntity.addFileAttribute(mFileAttrTypeId, "file desc here, long or short", file).getId
      val t: FileAttribute = mEntity.getFileAttribute(id)
      assert(t.getParentId == mEntity.getId)
      assert(t.getAttrTypeId == mFileAttrTypeId)
      assert(t.getDescription == "file desc here, long or short")
      assert(t.getOriginalFileDate > 1389461364000L)
      val now = System.currentTimeMillis()
      assert(t.getStoredDate < now && t.getStoredDate > now - (5 * 1000 * 60))
      assert(t.getOriginalFilePath == path)
      assert(t.getReadable)
      assert(t.getWritable)
      assert(!t.getExecutable)
      assert(t.getSize == 5)
    }
    finally {
      if (fw != null) fw.close()
      if (file != null) file.delete()
    }
    mDB.rollbackTrans()
  }

  "getDisplayString" should "return a useful stack trace string, when called with a nonexistent entity" in {
    // for example, if the entity has been deleted by one part of the code, or one user process in a console window (as an example), and is still
    // referenced and attempted to be displayed by another (or to be somewhat helpful if we try to get info on an entity that's gone due to a bug).
    // (But should this issue go away w/ better design involving more use of immutability or something?)
    val id = 0L
    val mockDB = mock[PostgreSQLDatabase]
    when(mockDB.entityKeyExists(id)).thenReturn(true)
    when(mockDB.getEntityData(id)).thenThrow(new RuntimeException("some exception"))
    when(mockDB.getRemoteAddress).thenReturn(None)
    val entity = new Entity(mockDB, id)
    val se = entity.getDisplayString()
    assert(se.contains("Unable to get entity description due to"))
    assert(se.toLowerCase.contains("exception"))
    assert(se.toLowerCase.contains("at org.onemodel"))
  }

  "getDisplayString" should "return name & class info" in {
    val id = 0L
    val classId = 1L
    val mockDB = mock[PostgreSQLDatabase]
    when(mockDB.entityKeyExists(id)).thenReturn(true)
    when(mockDB.getClassName(classId)).thenReturn(Some("class1Name"))
    when(mockDB.getEntityData(id)).thenReturn(Array[Option[Any]](Some("entity1Name"), Some(classId)))
    // idea (is in tracked tasks): put next 3 lines back after color refactoring is done (& places w/ similar comment elsewhere)
    //val entity = new Entity(mockDB, id)
    //val ds = entity.getDisplayString
    //assert(ds == "entity1Name (class: class1Name)")

    val id2 = 2L
    val classId2 = 4L
    val name2 = "entity2Name"
    val mockDB2 = mock[PostgreSQLDatabase]
    when(mockDB2.entityKeyExists(id2)).thenReturn(true)
    when(mockDB2.getEntityData(id2)).thenReturn(Array[Option[Any]](Some(name2), None))
    when(mockDB2.getClassName(classId2)).thenReturn(None)
    // idea (is in tracked tasks): put next lines back after color refactoring is done (& places w/ similar comment elsewhere)
    //val entity2 = new Entity(mockDB2, id2, name2, Some(false), Some(classId2))
    //val ds2 = entity2.getDisplayString
    //assert(ds2 == name2)

    when(mockDB2.getClassName(classId2)).thenReturn(Some("class2Name"))
    when(mockDB2.getClassCount(Some(id2))).thenReturn(1)
    when(mockDB2.getEntityData(id2)).thenReturn(Array[Option[Any]](Some(name2), Some(classId2)))
    // idea (is in tracked tasks): put next line back after color refactoring is done (& places w/ similar comment elsewhere)
    //assert(entity2.getDisplayString == name2 + " (template entity (template) for class: " + "class2Name)")
  }

  "getClassTemplateEntityId" should "work right" in {
    val mockDB = mock[PostgreSQLDatabase]
    val id = 1L
    val classId = 2L
    val className = "classname"
    val templateEntityId = 3L
    when(mockDB.entityKeyExists(id)).thenReturn(true)
    val e = new Entity(mockDB, id, "entityname", None, 0L, Some(true), false, false)
    assert(e.getClassTemplateEntityId.isEmpty)

    val e2 = new Entity(mockDB, id, "entityname", Option(classId), 0L, Some(false), false, false)
    when(mockDB.classKeyExists(classId)).thenReturn(true)
    when(mockDB.getClassData(classId)).thenReturn(Array[Option[Any]](Some(className), Some(templateEntityId)))
    assert(e2.getClassTemplateEntityId.get == templateEntityId)
  }

  "updateContainedEntitiesPublicStatus" should "work" in {
    val e1Id: Long = mDB.createEntity("test object1")
    val e1 = new Entity(mDB, e1Id)
    mEntity.addHASRelationToLocalEntity(e1.getId, Some(0), 0)
    val (group: Group, _/*rtg: RelationToGroup*/) = mEntity.addGroupAndRelationToGroup(mRelationTypeId, "grpName",
                                                                                    allowMixedClassesInGroupIn = true, Some(0), 0, None)
    val e2Id: Long = mDB.createEntity("test object2")
    val e2 = new Entity(mDB, e1Id)
    group.addEntity(e2Id)

    assert(e1.getPublic.isEmpty)
    assert(e2.getPublic.isEmpty)
    mEntity.updateContainedEntitiesPublicStatus(Some(true))
    val e1reRead = new Entity(mDB, e1Id)
    val e2reRead = new Entity(mDB, e2Id)
    assert(e1reRead.getPublic.get)
    assert(e2reRead.getPublic.get)
  }

  "getCountOfContainingLocalEntities etc" should "work" in {
    val e1 = Entity.createEntity(mDB, "e1")
    val (e2id: Long, rteId: Long) = mDB.createEntityAndRelationToLocalEntity(e1.getId, mRelationTypeId, "e2", None, None, 0L)
    val e2: Option[Entity] = Entity.getEntity(mDB, e2id)
    assert(e2.get.getCountOfContainingLocalEntities._1 == 1)
    assert(e2.get.getLocalEntitiesContainingEntity().size == 1)
    /*val (e3id: Long, rte2id: Long) = */mDB.createEntityAndRelationToLocalEntity(e1.getId, mRelationTypeId, "e3", None, None, 0L)
    assert(e1.getAdjacentAttributesSortingIndexes(Database.minIdValue).nonEmpty)
    val nearestSortingIndex = e1.getNearestAttributeEntrysSortingIndex(Database.minIdValue).get
    assert(nearestSortingIndex > Database.minIdValue)
    e1.renumberSortingIndexes()
    val nearestSortingIndex2 = e1.getNearestAttributeEntrysSortingIndex(Database.minIdValue).get
    assert(nearestSortingIndex2 > nearestSortingIndex)

    val rte = RelationToLocalEntity.getRelationToLocalEntity(mDB, rteId).get
    assert(! e1.isAttributeSortingIndexInUse(Database.maxIdValue))
    e1.updateAttributeSortingIndex(rte.getFormId, rte.getId, Database.maxIdValue)
    assert(e1.getAttributeSortingIndex(rte.getFormId, rte.getId) == Database.maxIdValue)
    assert(e1.isAttributeSortingIndexInUse(Database.maxIdValue))
    assert(e1.findUnusedAttributeSortingIndex() != Database.maxIdValue)
    assert(e1.getRelationToLocalEntityCount() == 2)
    e2.get.archive()
    assert(e1.getRelationToLocalEntityCount(includeArchivedEntitiesIn = false) == 1)
    assert(e1.getRelationToLocalEntityCount(includeArchivedEntitiesIn = true) == 2)
    assert(e1.getTextAttributeByTypeId(mRelationTypeId).size == 0)
    e1.addTextAttribute(mRelationTypeId, "abc", None)
    assert(e1.getTextAttributeByTypeId(mRelationTypeId).size == 1)

    assert(Entity.getEntity(mDB, e1.getId).get.getName != "updated")
    e1.updateName("updated")
    assert(Entity.getEntity(mDB, e1.getId).get.getName == "updated")
    assert(Entity.isDuplicate(mDB, "updated"))
    assert(! Entity.isDuplicate(mDB, "xyzNOTANAMEupdated"))

    val g1 = Group.createGroup(mDB, "g1")
    g1.addEntity(e1.getId)
    assert(e1.getContainingGroupsIds.size == 1)
    assert(e1.getCountOfContainingGroups == 1)
    e2.get.addRelationToGroup(mRelationTypeId, g1.getId, None)
    assert(e1.getContainingRelationsToGroup().size == 1)
    assert(e1.getContainingRelationToGroupDescriptions().size == 0)
    e2.get.unarchive()
    assert(e1.getContainingRelationToGroupDescriptions().size == 1)
  }



}
