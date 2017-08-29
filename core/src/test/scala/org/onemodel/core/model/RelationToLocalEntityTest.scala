/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2013-2017 inclusive, Luke A Call; all rights reserved.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule, guidelines around binary
    distribution, and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
package org.onemodel.core.model

import org.scalatest.mockito.MockitoSugar
import org.scalatest.{Status, Args, FlatSpec}

class RelationToLocalEntityTest extends FlatSpec with MockitoSugar {
  var mDB: PostgreSQLDatabase = null

  // using the real db because it got too complicated with mocks, and the time savings don't seem enough to justify the work with the mocks.
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

  "getDisplayString" should "return correct strings and length" in {
    val relationTypeName: String = "is husband to"
    val relationTypeNameInReverseDirection: String = "is wife to"
    val relationTypeId: Long = mDB.createRelationType(relationTypeName, relationTypeNameInReverseDirection, "BI")
    val relationType = new RelationType(mDB, relationTypeId)
    val entity1Name = "husbandName"
    val entity2Name = "wifeName"
    val entity1 = new Entity(mDB, mDB.createEntity(entity1Name))
    val entity2 = new Entity(mDB, mDB.createEntity(entity2Name))
    val date = 304
    val rtle: RelationToLocalEntity = mDB.createRelationToLocalEntity(relationTypeId, entity1.getId, entity2.getId, None, date, Some(0))

    val smallLimit = 15
    val displayed1: String = rtle.getDisplayString(smallLimit, Some(entity2), Some(relationType))
    val expectedDateOutput = "Wed 1969-12-31 17:00:00:"+date+" MST"
    val wholeExpectedThing: String = relationTypeName + ": " + entity2Name + "; valid unsp'd, obsv'd "+expectedDateOutput
    val expected = wholeExpectedThing.substring(0, smallLimit - 3) + "..."
    assert(displayed1 == expected, "unexpected contents: " + displayed1)

    val displayed2: String = rtle.getDisplayString(0, Some(entity1), Some(relationType))
    val expected2:String = relationTypeNameInReverseDirection + ": \033[36m" + entity1Name + "\033[0m; valid unsp'd, obsv'd "+expectedDateOutput
    assert(displayed2 == expected2)

    val displayed3: String = rtle.getDisplayString(smallLimit, Some(entity2), Some(relationType), simplify = true)
    assert(displayed3 == "is husband t...")

  }

  "move and update" should "work" in {
    val entity1 = new Entity(mDB, mDB.createEntity("entity1"))
    val entity2 = new Entity(mDB, mDB.createEntity("entity2"))
    val entity3 = new Entity(mDB, mDB.createEntity("entity3"))
    val relType = new RelationType(mDB, mDB.createRelationType("reltype1", "", "UNI"))
    val rtle: RelationToLocalEntity = mDB.createRelationToLocalEntity(relType.getId, entity1.getId, entity2.getId, Some(0L), 0)
    val firstParent = rtle.getRelatedId1
    assert(firstParent == entity1.getId)
    val newRtle: RelationToLocalEntity = rtle.move(entity3.getId, 0)
    // reread to get new data
    assert(newRtle.getParentId == entity3.getId)
    assert(newRtle.getAttrTypeId == relType.getId)
    assert(newRtle.getRelatedId2 == entity2.getId)

    newRtle.getValidOnDate
    newRtle.getObservationDate
    newRtle.getAttrTypeId
    val newAttrTypeId = mDB.createRelationType("newAttrType", "reversed", "NON")
    val newVod = 345L
    val newOd = 456L
    newRtle.update(Some(newVod), Some(newOd), Some(newAttrTypeId))
    val updatedRtle = new RelationToLocalEntity(mDB, newRtle.getId, newAttrTypeId, newRtle.getRelatedId1, newRtle.getRelatedId2)
    assert(updatedRtle.getValidOnDate.get == newVod)
    assert(updatedRtle.getObservationDate == newOd)

    val groupId = mDB.createGroup("group")
    val group = new Group(mDB, groupId)
    assert(! group.isEntityInGroup(entity2.getId))
    newRtle.moveEntityFromEntityToGroup(groupId, 0)
    assert(! mDB.relationToLocalEntityKeyExists(newRtle.getId))
    assert(group.isEntityInGroup(entity2.getId))
  }

  "delete etc" should "work" in {
    val entity1 = new Entity(mDB, mDB.createEntity("entity1"))
    val entity2 = new Entity(mDB, mDB.createEntity("entity2"))
    val relType = new RelationType(mDB, mDB.createRelationType("reltype1", "", "UNI"))
    val rtle: RelationToLocalEntity = mDB.createRelationToLocalEntity(relType.getId, entity1.getId, entity2.getId, Some(0L), 0)
    assert(mDB.relationToLocalEntityExists(relType.getId, entity1.getId, entity2.getId))
    rtle.delete()
    assert(!mDB.relationToLocalEntityExists(relType.getId, entity1.getId, entity2.getId))

    // throwing in this test for ease & faster running: otherwise should be in RelationTypeTest:
    val newName = "new-reltype-name"
    val newInReverseName = "new-in-reverse"
    relType.update(newName, newInReverseName, "NON")
    val updatedRelationType = new RelationType(mDB, relType.getId)
    assert(updatedRelationType.getName == newName)
    assert(updatedRelationType.getNameInReverseDirection == newInReverseName)
    assert(updatedRelationType.getDirectionality == "NON")
  }
}
