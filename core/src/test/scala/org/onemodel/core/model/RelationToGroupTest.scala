/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2013-2017 inclusive, Luke A. Call; all rights reserved.
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
import org.scalatest.{Status, Args, FlatSpec}

class RelationToGroupTest extends FlatSpec with MockitoSugar {
  var mDB: PostgreSQLDatabase = null

  // Starting to use the real db because the time savings don't seem enough to justify the work with the mocks. (?)
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
    mDB = new PostgreSQLDatabase(Database.TEST_USER, Database.TEST_USER)
  }

  protected def tearDown() {
    PostgreSQLDatabaseTest.tearDownTestDB()
  }

  "getDisplayString" should "return correct string and length" in {
    val mockDB = mock[PostgreSQLDatabase]

    // arbitrary...:
    val rtgId: Long = 300
    val groupId: Long = 301
    val entityId: Long = 302
    val classTemplateEntityId: Long = 303
    val relTypeId: Long = 401
    val classId: Long = 501
    val grpName: String = "somename"
    val grpEntryCount = 9
    // arbitrary, in milliseconds:
    val date = 304
    val relationTypeName: String = Database.theHASrelationTypeName
    when(mockDB.groupKeyExists(groupId)).thenReturn(true)
    when(mockDB.relationTypeKeyExists(relTypeId)).thenReturn(true)
    when(mockDB.entityKeyExists(relTypeId)).thenReturn(true)
    when(mockDB.relationToGroupKeysExistAndMatch(rtgId, entityId, relTypeId, groupId)).thenReturn(true)
    when(mockDB.getGroupData(groupId)).thenReturn(Array[Option[Any]](Some(grpName), Some(0L), Some(true), Some(false)))
    when(mockDB.getGroupSize(groupId, 1)).thenReturn(grpEntryCount)
    when(mockDB.getRelationTypeData(relTypeId)).thenReturn(Array[Option[Any]](Some(relationTypeName), Some(Database.theIsHadByReverseName), Some("xyz..")))
    when(mockDB.getRemoteAddress).thenReturn(None)

    // (using arbitrary numbers for the unnamed parameters):
    val relationToGroup = new RelationToGroup(mockDB, rtgId, entityId, relTypeId, groupId, None, date, 0)
    val smallLimit = 15
    val observedDateOutput = "Wed 1969-12-31 17:00:00:" + date + " MST"
    val wholeThing: String = relationTypeName + " grp " + groupId + " /" + grpEntryCount + ": " + grpName + ", class: (mixed); valid unsp'd, obsv'd " + observedDateOutput

    val displayed: String = relationToGroup.getDisplayString(smallLimit, None)
    val expected = wholeThing.substring(0, smallLimit - 3) + "..."
    assert(displayed == expected)
    // idea (is in tracked tasks): put next 2 lines back after color refactoring is done (& places w/ similar comment elsewhere)
    //  val all: String = relationToGroup.getDisplayString(0, None)
    //  assert(all == wholeThing)

    val relationToGroup2 = new RelationToGroup(mockDB, rtgId, entityId, relTypeId, groupId, None, date, 0)
    when(mockDB.getGroupData(groupId)).thenReturn(Array[Option[Any]](Some(grpName), Some(0L), Some(false), Some(false)))
    val all2: String = relationToGroup2.getDisplayString(0, None)
    assert(!all2.contains("(mixed)"))
    assert(all2.contains(", class: (unspecified)"))

    val relationToGroup3 = new RelationToGroup(mockDB, rtgId, entityId, relTypeId, groupId, None, date, 0)
    when(mockDB.entityKeyExists(classTemplateEntityId)).thenReturn(true)
    val list = new java.util.ArrayList[Entity](1)
    list.add(new Entity(mockDB, classTemplateEntityId, "asdf", None, 0L, None, false, false))
    when(mockDB.getGroupEntryObjects(groupId, 0, Some(1))).thenReturn(list)
    when(mockDB.getGroupSize(groupId, 3)).thenReturn(list.size)
    val all3: String = relationToGroup3.getDisplayString(0, None)
    assert(!all3.contains("(mixed)"))
    assert(all3.contains(", class: (specified as None)"))

    val relationToGroup4 = new RelationToGroup(mockDB, rtgId, entityId, relTypeId, groupId, None, date, 0)
    val list4 = new java.util.ArrayList[Entity](1)
    list4.add(new Entity(mockDB, classTemplateEntityId, "asdf", Some(classId), 0L, Some(true), false, false))
    when(mockDB.entityKeyExists(classTemplateEntityId)).thenReturn(true)
    when(mockDB.classKeyExists(classId)).thenReturn(true)
    when(mockDB.getGroupEntryObjects(groupId, 0, Some(1))).thenReturn(list4)
    val className = "someClassName"
    when(mockDB.getClassData(classId)).thenReturn(Array[Option[Any]](Some(className), Some(classTemplateEntityId), Some(true)))
    val all4: String = relationToGroup4.getDisplayString(0, None)
    assert(!all4.contains("(mixed)"))
    assert(all4.contains(", class: " + className))
  }

  "getTemplateEntity" should "work right" in {
    val mockDB = mock[PostgreSQLDatabase]
    val rtgId: Long = 300
    val groupId: Long = 301
    //val parentId: Long = 302
    val classTemplateEntityId: Long = 303
    val relTypeId: Long = 401
    val entityId: Long = 402
    val classId: Long = 501
    val className = "someclassname"
    val grpName: String = "somename"
    when(mockDB.relationTypeKeyExists(relTypeId)).thenReturn(true)
    when(mockDB.entityKeyExists(relTypeId)).thenReturn(true)
    when(mockDB.relationToGroupKeysExistAndMatch(rtgId, entityId, relTypeId, groupId)).thenReturn(true)
    when(mockDB.groupKeyExists(groupId)).thenReturn(true)

    val group = new Group(mockDB, groupId)
    when(mockDB.groupKeyExists(groupId)).thenReturn(true)
    when(mockDB.entityKeyExists(entityId)).thenReturn(true)
    when(mockDB.entityKeyExists(classTemplateEntityId)).thenReturn(true)
    when(mockDB.classKeyExists(classId)).thenReturn(true)
    when(mockDB.getGroupEntryObjects(groupId, 0, Some(1))).thenReturn(new java.util.ArrayList[Entity](0))
    when(mockDB.getClassData(classId)).thenReturn(Array[Option[Any]](Some(className), Some(classTemplateEntityId), Some(true)))
    when(mockDB.getGroupData(groupId)).thenReturn(Array[Option[Any]](Some(grpName), Some(0L), Some(false), Some(false)))
    when(mockDB.getRemoteAddress).thenReturn(None)
    // should be None because it is not yet specified (no entities added):
    assert(group.getClassTemplateEntity.isEmpty)

    val list = new java.util.ArrayList[Entity](1)
    val entity = new Entity(mockDB, entityId, "testEntityName", Some(classId), 0L, Some(false), false, false)
    list.add(entity)
    when(mockDB.getGroupEntryObjects(groupId, 0, Some(1))).thenReturn(list)
    // should be != None because mixed classes are NOT allowed in the group and an entity was added:
    assert(group.getClassTemplateEntity.get.getId == classTemplateEntityId)

    //relationToGroup = new RelationToGroup(mockDB, entityId, relTypeId, groupId, None, date)
    // should be None when mixed classes are allowed in the group:
    when(mockDB.getGroupData(groupId)).thenReturn(Array[Option[Any]](Some(grpName), Some(0L), Some(true), Some(false)))
    val group2 = new Group(mockDB, groupId)
    assert(group2.getClassTemplateEntity.isEmpty)
  }

  "move and update" should "work" in {
    val entity1 = new Entity(mDB, mDB.createEntity("entityName1"))
    val (_, rtg: RelationToGroup) = entity1.createGroupAndAddHASRelationToIt("groupName", mixedClassesAllowedIn = false, 0)
    val (attributeTuples1: Array[(Long, Attribute)], _) = entity1.getSortedAttributes(0, 0)
    val rtg1 = attributeTuples1(0)._2.asInstanceOf[RelationToGroup]
    assert(rtg1.getParentId == entity1.getId)
    assert(rtg1.getId == rtg.getId)
    val rtg1_gid = rtg1.getGroupId
    val rtg1_rtid = rtg1.getAttrTypeId

    val entity2 = new Entity(mDB, mDB.createEntity("entityName2"))
    rtg.move(entity2.getId, 0)

    val (attributeTuples1a: Array[(Long, Attribute)], _) = entity1.getSortedAttributes(0, 0)
    assert(attributeTuples1a.length == 0)
    val (attributeTuples2: Array[(Long, Attribute)], _) = entity2.getSortedAttributes(0, 0)
    val rtg2 = attributeTuples2(0)._2.asInstanceOf[RelationToGroup]
    val rtg2RelTypeId = rtg2.getAttrTypeId
    val rtg2GroupId = rtg2.getGroupId
    val vod2 = rtg2.getValidOnDate
    val od2 = rtg2.getObservationDate
    assert(rtg2.getParentId == entity2.getId)
    assert(rtg2.getParentId != entity1.getId)
    assert(rtg1_gid == rtg2GroupId)
    assert(rtg1_rtid == rtg2RelTypeId)
    assert(rtg2.getId != rtg.getId)

    val newRelationTypeId = mDB.createRelationType("RTName", "reversed", "BI")
    val newGroupId = mDB.createGroup("newGroup")
    val newVod = Some(4321L)
    val newOd = Some(5432L)
    rtg2.update(Some(newRelationTypeId), Some(newGroupId), newVod, newOd)
    val rtg2a = new RelationToGroup(mDB, rtg2.getId, rtg2.getParentId, newRelationTypeId, newGroupId)
    assert(rtg2a.getValidOnDate != vod2)
    assert(rtg2a.getValidOnDate.get == 4321L)
    assert(rtg2a.getObservationDate != od2)
    assert(rtg2a.getObservationDate == 5432L)
  }

}