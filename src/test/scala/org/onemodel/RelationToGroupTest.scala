/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2013-2016 inclusive, Luke A. Call; all rights reserved.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule, guidelines around binary
    distribution, and the GNU Affero General Public License as published by the Free Software Foundation, either version 3
    of the License, or (at your option) any later version.  See the file LICENSE for details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>

  ---------------------------------------------------
  If we ever do port to another database, create the Database interface (removed around 2014-1-1 give or take) and see other changes at that time.
  An alternative method is to use jdbc escapes (but this actually might be even more work?):  http://jdbc.postgresql.org/documentation/head/escapes.html  .
  Another alternative is a layer like JPA, ibatis, hibernate  etc etc.

*/
package org.onemodel

import org.scalatest.FlatSpec
import org.mockito.Mockito._
import org.scalatest.mock.MockitoSugar
import org.onemodel.model.{RelationToGroup, Group, Entity}
import org.onemodel.database.PostgreSQLDatabase

class RelationToGroupTest extends FlatSpec with MockitoSugar {
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
    val relationTypeName: String = PostgreSQLDatabase.theHASrelationTypeName
    when(mockDB.groupKeyExists(groupId)).thenReturn(true)
    when(mockDB.relationTypeKeyExists(relTypeId)).thenReturn(true)
    when(mockDB.entityKeyExists(relTypeId)).thenReturn(true)
    when(mockDB.relationToGroupKeysExistAndMatch(rtgId, entityId, relTypeId, groupId)).thenReturn(true)
    when(mockDB.getGroupData(groupId)).thenReturn(Array[Option[Any]](Some(grpName), Some(0L), Some(true)))
    when(mockDB.getGroupSize(groupId, 1)).thenReturn(grpEntryCount)
    when(mockDB.getRelationTypeData(relTypeId)).thenReturn(Array[Option[Any]](Some(relationTypeName), Some(PostgreSQLDatabase.theIsHadByReverseName),
                                                                              Some("xyz..")))
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
    when(mockDB.getGroupData(groupId)).thenReturn(Array[Option[Any]](Some(grpName), Some(0L), Some(false)))
    val all2: String = relationToGroup2.getDisplayString(0, None)
    assert(!all2.contains("(mixed)"))
    assert(all2.contains(", class: (unspecified)"))

    val relationToGroup3 = new RelationToGroup(mockDB, rtgId, entityId, relTypeId, groupId, None, date, 0)
    when(mockDB.entityKeyExists(classTemplateEntityId)).thenReturn(true)
    val list = new java.util.ArrayList[Entity](1)
    list.add(new Entity(mockDB, classTemplateEntityId, "asdf", None, 0L, None, false))
    when(mockDB.getGroupEntryObjects(groupId, 0, Some(1))).thenReturn(list)
    when(mockDB.getGroupSize(groupId, 3)).thenReturn(list.size)
    val all3: String = relationToGroup3.getDisplayString(0, None)
    assert(!all3.contains("(mixed)"))
    assert(all3.contains(", class: (specified as None)"))

    val relationToGroup4 = new RelationToGroup(mockDB, rtgId, entityId, relTypeId, groupId, None, date, 0)
    val list4 = new java.util.ArrayList[Entity](1)
    list4.add(new Entity(mockDB, classTemplateEntityId, "asdf", Some(classId), 0L, Some(true), false))
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
    when(mockDB.getGroupData(groupId)).thenReturn(Array[Option[Any]](Some(grpName), Some(0L), Some(false)))
    // should be None because it is not yet specified (no entities added):
    assert(group.getClassTemplateEntity.isEmpty)

    val list = new java.util.ArrayList[Entity](1)
    val entity = new Entity(mockDB, entityId, "testEntityName", Some(classId), 0L, Some(false), false)
    list.add(entity)
    when(mockDB.getGroupEntryObjects(groupId, 0, Some(1))).thenReturn(list)
    // should be != None because mixed classes are NOT allowed in the group and an entity was added:
    assert(group.getClassTemplateEntity.get.getId == classTemplateEntityId)

    //relationToGroup = new RelationToGroup(mockDB, entityId, relTypeId, groupId, None, date)
    // should be None when mixed classes are allowed in the group:
    when(mockDB.getGroupData(groupId)).thenReturn(Array[Option[Any]](Some(grpName), Some(0L), Some(true)))
    val group2 = new Group(mockDB, groupId)
    assert(group2.getClassTemplateEntity.isEmpty)
  }

}