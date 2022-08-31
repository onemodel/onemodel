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
  let mut mDB: PostgreSQLDatabase = null;

  // using the real db because it got too complicated with mocks, and the time savings don't seem enough to justify the work with the mocks.
  override def runTests(testName: Option[String], args: Args):Status = {
    setUp()
    let result:Status = super.runTests(testName,args);
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
    let relationTypeName: String = "is husband to";
    let relationTypeNameInReverseDirection: String = "is wife to";
    let relationTypeId: i64 = mDB.createRelationType(relationTypeName, relationTypeNameInReverseDirection, "BI");
    let relationType = new RelationType(mDB, relationTypeId);
    let entity1Name = "husbandName";
    let entity2Name = "wifeName";
    let entity1 = new Entity(mDB, mDB.createEntity(entity1Name));
    let entity2 = new Entity(mDB, mDB.createEntity(entity2Name));
    let date = 304;
    let rtle: RelationToLocalEntity = mDB.createRelationToLocalEntity(relationTypeId, entity1.getId, entity2.getId, None, date, Some(0));

    let smallLimit = 15;
    let displayed1: String = rtle.getDisplayString(smallLimit, Some(entity2), Some(relationType));
    let expectedDateOutput = "Wed 1969-12-31 17:00:00:"+date+" MST";
    let wholeExpectedThing: String = relationTypeName + ": " + entity2Name + "; valid unsp'd, obsv'd "+expectedDateOutput;
    let expected = wholeExpectedThing.substring(0, smallLimit - 3) + "...";
    assert(displayed1 == expected, "unexpected contents: " + displayed1)

    let displayed2: String = rtle.getDisplayString(0, Some(entity1), Some(relationType));
    let expected2:String = relationTypeNameInReverseDirection + ": \033[36m" + entity1Name + "\033[0m; valid unsp'd, obsv'd "+expectedDateOutput;
    assert(displayed2 == expected2)

    let displayed3: String = rtle.getDisplayString(smallLimit, Some(entity2), Some(relationType), simplify = true);
    assert(displayed3 == "is husband t...")

  }

  "move and update" should "work" in {
    let entity1 = new Entity(mDB, mDB.createEntity("entity1"));
    let entity2 = new Entity(mDB, mDB.createEntity("entity2"));
    let entity3 = new Entity(mDB, mDB.createEntity("entity3"));
    let relType = new RelationType(mDB, mDB.createRelationType("reltype1", "", "UNI"));
    let rtle: RelationToLocalEntity = mDB.createRelationToLocalEntity(relType.getId, entity1.getId, entity2.getId, Some(0L), 0);
    let firstParent = rtle.getRelatedId1;
    assert(firstParent == entity1.getId)
    let newRtle: RelationToLocalEntity = rtle.move(entity3.getId, 0);
    // reread to get new data
    assert(newRtle.getParentId == entity3.getId)
    assert(newRtle.getAttrTypeId == relType.getId)
    assert(newRtle.getRelatedId2 == entity2.getId)

    newRtle.getValidOnDate
    newRtle.getObservationDate
    newRtle.getAttrTypeId
    let newAttrTypeId = mDB.createRelationType("newAttrType", "reversed", "NON");
    let newVod = 345L;
    let newOd = 456L;
    newRtle.update(Some(newVod), Some(newOd), Some(newAttrTypeId))
    let updatedRtle = new RelationToLocalEntity(mDB, newRtle.getId, newAttrTypeId, newRtle.getRelatedId1, newRtle.getRelatedId2);
    assert(updatedRtle.getValidOnDate.get == newVod)
    assert(updatedRtle.getObservationDate == newOd)

    let groupId = mDB.createGroup("group");
    let group = new Group(mDB, groupId);
    assert(! group.isEntityInGroup(entity2.getId))
    newRtle.moveEntityFromEntityToGroup(groupId, 0)
    assert(! mDB.relationToLocalEntityKeyExists(newRtle.getId))
    assert(group.isEntityInGroup(entity2.getId))
  }

  "delete etc" should "work" in {
    let entity1 = new Entity(mDB, mDB.createEntity("entity1"));
    let entity2 = new Entity(mDB, mDB.createEntity("entity2"));
    let relType = new RelationType(mDB, mDB.createRelationType("reltype1", "", "UNI"));
    let rtle: RelationToLocalEntity = mDB.createRelationToLocalEntity(relType.getId, entity1.getId, entity2.getId, Some(0L), 0);
    assert(mDB.relationToLocalEntityExists(relType.getId, entity1.getId, entity2.getId))
    rtle.delete()
    assert(!mDB.relationToLocalEntityExists(relType.getId, entity1.getId, entity2.getId))

    // throwing in this test for ease & faster running: otherwise should be in RelationTypeTest:
    let newName = "new-reltype-name";
    let newInReverseName = "new-in-reverse";
    relType.update(newName, newInReverseName, "NON")
    let updatedRelationType = new RelationType(mDB, relType.getId);
    assert(updatedRelationType.getName == newName)
    assert(updatedRelationType.getNameInReverseDirection == newInReverseName)
    assert(updatedRelationType.getDirectionality == "NON")
  }
}
