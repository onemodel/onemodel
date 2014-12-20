/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2013-2014 inclusive, Luke A Call; all rights reserved.
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
import org.onemodel.model.{RelationType, RelationToGroup, Group, Entity}
import org.onemodel.database.PostgreSQLDatabase

class RelationToGroupTest extends FlatSpec with MockitoSugar {
  "getDisplayString" should "return correct string and length" in {
    val mockDB = mock[PostgreSQLDatabase]
    val mockRelationType = mock[RelationType]

    // arbitrary...:
    val groupId: Long = 301
    val entityId: Long = 302
    val classDefiningEntityId: Long = 303
    val relTypeId: Long = 401
    val classId: Long = 501
    val grpName: String = "somename"
    // arbitrary, in milliseconds:
    val date = 304
    val relationTypeName: String = "has"
    when(mockDB.groupKeyExists(groupId)).thenReturn(true)
    when(mockRelationType.getName).thenReturn(relationTypeName)
    when(mockRelationType.getId).thenReturn(relTypeId)
    when(mockDB.relationTypeKeyExists(relTypeId)).thenReturn(true)
    when(mockDB.entityKeyExists(relTypeId)).thenReturn(true)
    when(mockDB.relationToGroupKeyExists(entityId, relTypeId, groupId)).thenReturn(true)
    when(mockDB.getGroupData(groupId)).thenReturn(Array[Option[Any]](Some(grpName), Some(0L), Some(true)))

    // (using arbitrary numbers for the unnamed parameters):
    val relationToGroup = new RelationToGroup(mockDB, entityId, relTypeId, groupId, None, date)
    val smallLimit = 15
    val observedDateOutput = "Wed 1969-12-31 17:00:00:" + date + " MST"
    val wholeThing: String = relationTypeName + " group/0: " + grpName + ", class: (mixed); valid unsp'd, obsv'd " + observedDateOutput

    val displayed: String = relationToGroup.getDisplayString(smallLimit, None, Some(mockRelationType))
    val expected = wholeThing.substring(0, smallLimit - 3) + "..."
    assert(displayed == expected)
    val all: String = relationToGroup.getDisplayString(0, None, Some(mockRelationType))
    // %%put next line back after color refactoring is done, or after there is a cleaner approach to managing colors, which might change next line's failure.
    // %% see other places w/ similar comments.
    //  assert(all == wholeThing)

    val relationToGroup2 = new RelationToGroup(mockDB, entityId, relTypeId, groupId, None, date)
    when(mockDB.getGroupData(groupId)).thenReturn(Array[Option[Any]](Some(grpName), Some(0L), Some(false)))
    val all2: String = relationToGroup2.getDisplayString(0, None, Some(mockRelationType))
    assert(!all2.contains("(mixed)"))
    assert(all2.contains(", class: (unspecified)"))

    val relationToGroup3 = new RelationToGroup(mockDB, entityId, relTypeId, groupId, None, date)
    when(mockDB.entityKeyExists(classDefiningEntityId)).thenReturn(true)
    val list = new java.util.ArrayList[Entity](1)
    list.add(new Entity(mockDB, classDefiningEntityId, "asdf", None))
    when(mockDB.getGroupEntryObjects(groupId, 0, Some(1))).thenReturn(list)
    when(mockDB.getGroupEntryCount(groupId)).thenReturn(list.size)
    val all3: String = relationToGroup3.getDisplayString(0, None, Some(mockRelationType))
    assert(!all3.contains("(mixed)"))
    assert(all3.contains(", class: (specified as None)"))

    val relationToGroup4 = new RelationToGroup(mockDB, entityId, relTypeId, groupId, None, date)
    val list4 = new java.util.ArrayList[Entity](1)
    list4.add(new Entity(mockDB, classDefiningEntityId, "asdf", Some(true), Some(classId)))
    when(mockDB.entityKeyExists(classDefiningEntityId)).thenReturn(true)
    when(mockDB.classKeyExists(classId)).thenReturn(true)
    when(mockDB.getGroupEntryObjects(groupId, 0, Some(1))).thenReturn(list4)
    val className = "someClassName"
    when(mockDB.getClassData(classId)).thenReturn(Array[Option[Any]](Some(className), Some(classDefiningEntityId)))
    val all4: String = relationToGroup4.getDisplayString(0, None, Some(mockRelationType))
    assert(!all4.contains("(mixed)"))
    assert(all4.contains(", class: " + className))
  }

  "getDefiningEntity" should "work right" in {
    val mockDB = mock[PostgreSQLDatabase]
    val groupId: Long = 301
    //val parentId: Long = 302
    val classDefiningEntityId: Long = 303
    val relTypeId: Long = 401
    val entityId: Long = 402
    val classId: Long = 501
    val className = "someclassname"
    val grpName: String = "somename"
    when(mockDB.relationTypeKeyExists(relTypeId)).thenReturn(true)
    when(mockDB.entityKeyExists(relTypeId)).thenReturn(true)
    when(mockDB.relationToGroupKeyExists(entityId, relTypeId, groupId)).thenReturn(true)
    when(mockDB.groupKeyExists(groupId)).thenReturn(true)

    val group = new Group(mockDB, groupId)
    when(mockDB.groupKeyExists(groupId)).thenReturn(true)
    when(mockDB.entityKeyExists(entityId)).thenReturn(true)
    when(mockDB.entityKeyExists(classDefiningEntityId)).thenReturn(true)
    when(mockDB.classKeyExists(classId)).thenReturn(true)
    when(mockDB.getGroupEntryObjects(groupId, 0, Some(1))).thenReturn(new java.util.ArrayList[Entity](0))
    when(mockDB.getClassData(classId)).thenReturn(Array[Option[Any]](Some(className), Some(classDefiningEntityId)))
    when(mockDB.getGroupData(groupId)).thenReturn(Array[Option[Any]](Some(grpName), Some(0L), Some(false)))
    // should be None because it is not yet specified (no entities added):
    assert(group.getClassDefiningEntity == None)

    val list = new java.util.ArrayList[Entity](1)
    val entity = new Entity(mockDB, entityId, "testentityname", Some(false), Some(classId))
    list.add(entity)
    when(mockDB.getGroupEntryObjects(groupId, 0, Some(1))).thenReturn(list)
    // should be != None because mixed classes are NOT allowed in the group and an entity was added:
    assert(group.getClassDefiningEntity.get.getId == classDefiningEntityId)

    //relationToGroup = new RelationToGroup(mockDB, entityId, relTypeId, groupId, None, date)
    // should be None when mixed classes are allowed in the group:
    when(mockDB.getGroupData(groupId)).thenReturn(Array[Option[Any]](Some(grpName), Some(0L), Some(true)))
    val group2 = new Group(mockDB, groupId)
    assert(group2.getClassDefiningEntity == None)
  }

}