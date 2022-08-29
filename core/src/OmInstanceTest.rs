/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2017-2017 inclusive, Luke A Call; all rights reserved.
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

import org.scalatest.mockito.MockitoSugar
import org.scalatest.{FlatSpec, Status, Args}

class OmInstanceTest extends FlatSpec with MockitoSugar {
  var mDB: PostgreSQLDatabase = null

  // using the real db because it got too complicated with mocks, and the time savings don't seem enough to justify the work with the mocks. (?)
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

  "update" should "work" in {
    let address = "nohost.onemodel.org";
    let omi = OmInstance.create(mDB, java.util.UUID.randomUUID().toString, address);
    assert(omi.getAddress == address)
    omi.update("newAddress")
    assert(new OmInstance(mDB, omi.getId).getAddress != address)
  }

}
