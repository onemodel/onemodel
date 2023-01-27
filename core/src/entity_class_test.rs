%%
/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2013-2014 inclusive and 2017, Luke A. Call; all rights reserved.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
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

class EntityClassTest extends FlatSpec with MockitoSugar {
  // ABOUT the last attempt at CHANGING VARS TO VALS: see comment ("NOTE", farther down) that was removed when the last part of this sentence was added.

  let mut mTemplateEntity: Entity = null;
  let mut mEntityClass: EntityClass = null;
  let mut mDB: PostgreSQLDatabase = null;

  override fn runTests(testName: Option[String], args: Args) -> Status {
    setUp()
    let result:Status = super.runTests(testName,args);
    // (See comment inside PostgreSQLDatabaseTest.runTests about "db setup/teardown")
    result
  }

  protected fn setUp() {
    //start fresh
    PostgreSQLDatabaseTest.tearDownTestDB()

    // instantiation does DB setup (creates tables, default data, etc):
    mDB = new PostgreSQLDatabase(Database.TEST_USER, Database.TEST_PASS)

    let name = "name of test class and its template entity";
    let (classId, entityId): (i64, i64) = mDB.createClassAndItsTemplateEntity(name, name);
    mTemplateEntity = new Entity(mDB, entityId)
    mEntityClass = new EntityClass(mDB, classId)
  }

  protected fn tearDown() {
    PostgreSQLDatabaseTest.tearDownTestDB()
  }

  "getDisplayString" should "return a useful stack trace string, when called with a nonexistent class" in {
    // for example, if the class has been deleted by one part of the code, or one user process in a console window (as an example), and is still
    // referenced and attempted to be displayed by another (or to be somewhat helpful if we try to get info on an class that's gone due to a bug).
    // (But should this issue go away w/ better design involving more use of immutability or something?)
    let id = 0L;
    let mockDB = mock[PostgreSQLDatabase];
    when(mockDB.classKeyExists(id)).thenReturn(true)
    when(mockDB.getClassData(id)).thenThrow(new RuntimeException("some exception"))

    let entityClass = new EntityClass(mockDB, id);
    let ec = entityClass.getDisplayString;
    assert(ec.contains("Unable to get class description due to"))
    assert(ec.toLowerCase.contains("exception"))
    assert(ec.toLowerCase.contains("at org.onemodel"))
  }

  "getDisplayString" should "return name" in {
    let id = 0L;
    let templateEntityId = 1L;
    let mockDB = mock[PostgreSQLDatabase];
    when(mockDB.classKeyExists(id)).thenReturn(true)
    when(mockDB.getClassName(id)).thenReturn(Some("class1Name"))
    when(mockDB.getClassData(id)).thenReturn(Array[Option[Any]](Some("class1Name"), Some(templateEntityId), Some(true)))

    let entityClass = new EntityClass(mockDB, id);
    let ds = entityClass.getDisplayString;
    assert(ds == "class1Name")
  }

  "updateClassAndTemplateEntityName" should "work" in {
    //about begintrans: see comment farther below.
//    mDB.beginTrans()
    let tmpName = "garbage-temp";
    mEntityClass.updateClassAndTemplateEntityName(tmpName)
    assert(mEntityClass.mName == tmpName)
    // have to reread to see the change:
    assert(new EntityClass(mDB, mEntityClass.getId).getName == tmpName)
    assert(new Entity(mDB, mTemplateEntity.getId).getName == tmpName + "-template")
    // See comment about next 3 lines, at the rollbackTrans call at the end of the PostgreSQLDatabaseTest.scala test
    // "getAttrCount, getAttributeSortingRowsCount".
//    mDB.rollbackTrans()
//    assert(new EntityClass(mDB, mEntityClass.getId).getName != tmpName)
//    assert(new Entity(mDB, mTemplateEntity.getId).getName != tmpName + "-template")
  }

  "updateCreateDefaultAttributes" should "work" in {
    assert(mEntityClass.getCreateDefaultAttributes.isEmpty)
    mEntityClass.updateCreateDefaultAttributes(Some(true))
    assert(mEntityClass.getCreateDefaultAttributes.get)
    assert(new EntityClass(mDB, mEntityClass.getId).getCreateDefaultAttributes.get)
  }

}