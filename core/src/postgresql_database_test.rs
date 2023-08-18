%%
/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003, 2004, 2010, 2011, 2013-2017 inclusive, and 2023, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
package org.onemodel.core.model

import org.onemodel.core.controllers.{Controller, ImportExport}
import org.scalatest.mockito.MockitoSugar
import java.io.File
import java.util
import org.onemodel.core._
import org.scalatest.{Args, FlatSpec, Status}
import scala.collection.mutable

object PostgreSQLDatabaseTest {
    fn tearDownTestDB() {
      /* %% old ideas to make these work at each startup (but now using Once in util.rs):
           OR INSTEAD: just run a teardown in the build.rs, then have each test run a setup() which has a mutex
              and checks if setup is done, if not makes calls to set up the db (mutex so only one does it at a time),
              and so no subsequent one will do more than a quick check to the db to see if db alr set up.
              OR call the om executable then close it, like expect/integr tests do today, which does the setup,
              though that could later (???) have an out-of-order problem for bootstrapping? If that can work
              it won't be needed to have makeSureDbIsSetup() in every db-using.
              Anyway, doing something like this will make sure the tests don't have to all run serially right?

              AND/OR, the build.rs (only if testing?--ckSomeEnvVar?) just sets a flag in the db saying ~ "starting a build"
              and the above makeSureDbIsClearedAndSetupForTests() gets the mutex, sees the flag (or gets out if set to
              ~"dbReadyForTests"), clears/sets up like now, sets flag as just noted ~dbReadyForTests, and all
              tests check for that using that method, to similarly set it up or skip out?

              Surely there is some simpler way. Reread the above and think of it.

              CHECK THIS?:  Does build.rs run when only doing cargo test (ie no files changed)?  probly not?

              Or just wrap cargo test  in a (ct?) script that clears the db each time, and plan to use it.

              or one of the test frameworks from crates.io helps?  (don't seem to at 2023-02).

              or have a mutex in a fn that cks for a log file that has not grown beyond __ (w/ config.toml determining anything?)?

              or something in doc on build script examples?

              or something that integration tests can do, pre-run? re-ck docs 4 that!

              %%OR BETR:
                ??: create a like or use ct script, moved into OM dirs, to run db cleanup before tests
                  does a ck 1st to see if the place where it needs to run is a subdir of self/where
                then runs cargo test like now
                clears test db after? or later..?
                  or later could call a sept binary that uses a library to do it fr inside om code.
                and doc its use, interlinking notes w/in itself and in README?
                then call a db setup script inside the tests which causes it/them to set up the tables for now (if not alr done)
                and make tests not care if they run on a db where they ran already?
                  & if so test this so it will work if run multi times either way, w/ or w/o ct/cleanup/setup.
                  & doc that also

           cont reading re cargo build.rs &c
           see how make it only do the test setup when rung tests (mbe some env var?)
           is there a way for it to call test/om code? no, not compiled yet! so ..no need hopefully?
              yes, needed. all the db setup stuff right?
              use build scripts' build dependencies? -- can it depend on its own crate/s?
              or just launch/close om! which does the db setup right?
                is there a way to run it only after build, before test?
       */

}

class PostgreSQLDatabaseTest extends FlatSpec with MockitoSugar {
  override fn runTests(testName: Option<String>, args: Args): -> Status {
    // no longer doing db setup/teardown here, because we need to do teardown as a constructor-like command above,
    // before instantiating the DB (and that instantiation does setup).  Leaving tables in place after to allow adhoc manual test access.
    let result: Status = super.runTests(testName, args);
    result
  }


}