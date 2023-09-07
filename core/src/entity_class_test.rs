%%
/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2013-2014 inclusive and 2017, Luke A. Call; all rights reserved.
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

class EntityClassTest extends FlatSpec with MockitoSugar {
  // ABOUT the last attempt at CHANGING VARS TO VALS: see comment ("NOTE", farther down) that was removed when the last part of this sentence was added.

  let mut mTemplateEntity: Entity = null;
  let mut mEntityClass: EntityClass = null;
  let mut db: PostgreSQLDatabase = null;

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

    let name = "name of test class and its template entity";
    let (classId, entity_id): (i64, i64) = db.createClassAndItsTemplateEntity(name, name);
    mTemplateEntity = new Entity(db, entity_id)
    mEntityClass = new EntityClass(db, classId)
  }

  protected fn tearDown() {
    PostgreSQLDatabaseTest.tearDownTestDB()
  }

  "get_display_string" should "return a useful stack trace string, when called with a nonexistent class" in {
    // for example, if the class has been deleted by one part of the code, or one user process in a console window (as an example), and is still
    // referenced and attempted to be displayed by another (or to be somewhat helpful if we try to get info on an class that's gone due to a bug).
    // (But should this issue go away w/ better design involving more use of immutability or something?)
    let id = 0L;
    let mockDB = mock[PostgreSQLDatabase];
    when(mockDB.class_key_exists(id)).thenReturn(true)
    when(mockDB.get_class_data(id)).thenThrow(new RuntimeException("some exception"))

    let entityClass = new EntityClass(mockDB, id);
    let ec = entityClass.get_display_string;
    assert(ec.contains("Unable to get class description due to"))
    assert(ec.toLowerCase.contains("exception"))
    assert(ec.toLowerCase.contains("at org.onemodel"))
  }

  "get_display_string" should "return name" in {
    let id = 0L;
    let template_entity_id = 1L;
    let mockDB = mock[PostgreSQLDatabase];
    when(mockDB.class_key_exists(id)).thenReturn(true)
    when(mockDB.get_class_name(id)).thenReturn(Some("class1Name"))
    when(mockDB.get_class_data(id)).thenReturn(Vec<Option<DataType>>(Some("class1Name"), Some(template_entity_id), Some(true)))

    let entityClass = new EntityClass(mockDB, id);
    let ds = entityClass.get_display_string;
    assert(ds == "class1Name")
  }

  "update_class_and_template_entity_name" should "work" in {
    //about begintrans: see comment farther below.
//    db.begin_trans()
    let tmpName = "garbage-temp";
    mEntityClass.update_class_and_template_entity_name(tmpName)
    assert(mEntityClass.m_name == tmpName)
    // have to reread to see the change:
    assert(new EntityClass(db, mEntityClass.get_id).get_name == tmpName)
    assert(new Entity(db, mTemplateEntity.get_id).get_name == tmpName + "-template")
    // See comment about next 3 lines, at the rollback_trans call at the end of the PostgreSQLDatabaseTest.scala test
    // "getAttrCount, get_attribute_sorting_rows_count".
//    db.rollback_trans()
//    assert(new EntityClass(db, mEntityClass.get_id).get_name != tmpName)
//    assert(new Entity(db, mTemplateEntity.get_id).get_name != tmpName + "-template")
  }

  "updateCreateDefaultAttributes" should "work" in {
    assert(mEntityClass.getCreateDefaultAttributes.isEmpty)
    mEntityClass.updateCreateDefaultAttributes(Some(true))
    assert(mEntityClass.getCreateDefaultAttributes.get)
    assert(new EntityClass(db, mEntityClass.get_id).getCreateDefaultAttributes.get)
  }

}