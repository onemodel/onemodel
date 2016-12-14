/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2013-2014 inclusive and 2016, Luke A Call; all rights reserved.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule, guidelines around binary
    distribution, and the GNU Affero General Public License as published by the Free Software Foundation, either version 3
    of the License, or (at your option) any later version.  See the file LICENSE for details.
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

  var mEntityClass: EntityClass = null
  var mDB: PostgreSQLDatabase = null

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
    mDB = new PostgreSQLDatabase("testrunner", "testrunner")

    val name = "name of test class and its template entity"
    val (classId, _): (Long, Long) = mDB.createClassAndItsTemplateEntity(name, name)
    mEntityClass = new EntityClass(mDB, classId)
  }

  protected def tearDown() {
    PostgreSQLDatabaseTest.tearDownTestDB()
  }

  "getDisplayString" should "return a useful stack trace string, when called with a nonexistent class" in {
    // for example, if the class has been deleted by one part of the code, or one user process in a console window (as an example), and is still
    // referenced and attempted to be displayed by another (or to be somewhat helpful if we try to get info on an class that's gone due to a bug).
    // (But should this issue go away w/ better design involving more use of immutability or something?)
    val id = 0L
    val mockDB = mock[PostgreSQLDatabase]
    when(mockDB.classKeyExists(id)).thenReturn(true)
    when(mockDB.getClassData(id)).thenThrow(new RuntimeException("some exception"))

    val entityClass = new EntityClass(mockDB, id)
    val ec = entityClass.getDisplayString
    assert(ec.contains("Unable to get class description due to"))
    assert(ec.toLowerCase.contains("exception"))
    assert(ec.toLowerCase.contains("at org.onemodel"))
  }

  "getDisplayString" should "return name" in {
    val id = 0L
    val templateEntityId = 1L
    val mockDB = mock[PostgreSQLDatabase]
    when(mockDB.classKeyExists(id)).thenReturn(true)
    when(mockDB.getClassName(id)).thenReturn(Some("class1Name"))
    when(mockDB.getClassData(id)).thenReturn(Array[Option[Any]](Some("class1Name"), Some(templateEntityId), Some(true)))

    val entityClass = new EntityClass(mockDB, id)
    val ds = entityClass.getDisplayString
    assert(ds == "class1Name")
  }

}