/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2017-2017 inclusive, Luke A. Call; all rights reserved.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule, guidelines around binary
    distribution, and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
package org.onemodel.core.controllers;

import net.sf.expectit.*;
import static net.sf.expectit.matcher.Matchers.*;
import org.onemodel.core.OmException;
import org.onemodel.core.model.Database;
import org.onemodel.core.model.PostgreSQLDatabase;
import org.testng.annotations.*;
import java.lang.Exception;
import java.util.concurrent.TimeUnit;

@Test
/** Integration test ("...IT").
 */
public class EntityMenuIT {
  @BeforeClass
  protected void setUp() {
    // start w/ a very clean environment so can test that scenario also
    PostgreSQLDatabase db = new PostgreSQLDatabase(Database.TEST_USER(), Database.TEST_USER());
    db.destroyTables();
  }
  @AfterClass
  protected void tearDown() {
  }

  public void testOmUiEtc() throws Exception {
    String osName = System.getProperty("os.name");
    if (! osName.equalsIgnoreCase("linux")) {
      throw new OmException("This test isn't yet expected to work on anything but Linux (or maybe other unix), until the om-expect-tests " +
                              "script in the code is adapted to that, and also probably others.");
    }

    // Using expectit here to call *dejagnu* (instead of doing everything directly with expectit), because expectit was less clear how to debug than with
    // dejagnu + expect (maybe just my unfamiliarity), and dejagnu etc seem very mature and documented, and have shorter turnaround time when making
    // mods and retesting from the command-line (no compile step).  To debug the "om-expect-tests" script, run it (maybe via the "oet"
    // convenience script), and see its logs, or see other related documentation (such as mentioned in the core/testsuite/README file).
    // So, expectit is used below to run om-expect-tests, which runs dejagnu, which in turn calls expect.
    // More info about "expectit", is at:  https://github.com/Alexey1Gavrilov/expectit
    Process process = Runtime.getRuntime().exec("om-expect-tests");
    Result result = null;
    try(Expect expect = new ExpectBuilder()
      .withInputs(process.getInputStream())
      .withOutput(process.getOutputStream())
        // For some debugging, can change the the next line.  Details in first.exp under "Useful during testing". Or better yet, debug
        // by calling the om-expect-tests script directly.  Also, this test as of 2017-7-31 takes ~100 seconds on my laptop.
      .withTimeout(5, TimeUnit.MINUTES)
      .withAutoFlushEcho(true)
      .withExceptionOnFailure()
      .withAutoFlushEcho(true)
      .withEchoInput(System.out)
      .withEchoOutput(System.out)
      .build()
    ) {
/*    # Raise this # of expected passes as tests are added--the test output says what the # is if you
      # run "om-expect-tests" manually, to see the number.
      # The check is here because ^C (or other interruptions?) while "mvn verify" (or similar) was running.
      # caused runtest to exit but it returned a 0 (success) code to om-expect-tests, and so
      # a failure could be missed.  This makes sure it runs to completion.
      # (If you don't know what # to use, comment the line out, check the testrun.log that was just
      # updated, and update the #, then uncomment the line. Or just uncomment it, but don't commit that.) */
      // IDEA (also in tracked tasks): check for *the* # of passes here, and a 0 return code.  Having problems with expectit and might try its
      // docs again or a different toolkit for this purpose:
//      result = expect.expect(contains("# of expected passes"));
      result = expect.expect(contains("476"));

      /* ***************************
      (Idea to fix) FOR SOME REASON, when running in the IDE (as opposed to running "om-expect-tests" or "oet" from the command-line), this class test fails
      with the following output, and I'm done spending time on figuring out why since I don't know that running this test in the IDE is a requirement anyway.
      I verified that the data is there, and that removing prior output or checks made no difference (so probably not a full buffer).
      (But note that for testing speed or to check a subset of the expect tests in the IDE, one can use the "testing_newest_code_only" part of first.exp.)
      The output is:
        ==============================================
        To get started, you probably want to find or create an entity (such as with your own name, to track information connected to you, contacts, possessions etc, or with the subject of study), then set that or some entity as your default (using its menu).
        Press any key to continue...
        x






        ==============================================
        1-Add new entity (such as yourself using your name, to start)
        2-Search all / list existing entities (except quantity units, attr types, & relation types)
        Ctrl+C to quit
        2






        ==============================================
        ENTITIES: Pick from menu, or an item by letter; Alt+<letter> to go to the item & later come back)
        1-List next items (of 5 more)
        2-Add new entity (or new type like length, for use with quantity, true/false, date, text, or file attributes)
        3-Search for existing entity by name and text attribute content...
        4-Search for existing entity by id...
        5-Show journal (changed entities) by date range...
        6-Link to entity in a separate (REMOTE) OM instance...
        0/ESC - back/previous menu
         a-.system-use-only                                                              ESC[0m
        ERROR: tcl error sourcing /home/../proj/om/core/bin/../testsuite/om.tests/first.exp.
        ERROR: ERROR: timeout or eof while expecting: "f-User preferences", with timeout set to "3".
            while executing
        "error "ERROR: timeout or eof while expecting: \"$expectation\", with timeout set to \"$timeout\".""
            invoked from within
        "expect {
            "simple instructions to reproduce it consistently, maybe it can be fixed - 1" {
              # in this case OM is offering to provide a stack tr..."
            (procedure "myexpect" line 3)
            invoked from within
        "myexpect "f-User preferences""
            (procedure "initial_main_menu_prefs_add_default" line 15)
            invoked from within
        "initial_main_menu_prefs_add_default $test_user"
            (file "/home/.../proj/om/core/bin/../testsuite/om.tests/first.exp" line 1469)
            invoked from within
        "source /home/.../proj/om/core/bin/../testsuite/om.tests/first.exp"
            ("uplevel" body line 1)
            invoked from within
        "uplevel #0 source /home/.../proj/om/core/bin/../testsuite/om.tests/first.exp"
            invoked from within
        "catch "uplevel #0 source $test_file_name""
        testcase /home/.../proj/om/core/bin/../testsuite/om.tests/first.exp completed in 7 seconds

                        ===  Summary ===

        # of expected passes            6
      ******************** */

      // (This opens then closes the two "read" and "less" commands in the script, since they are
      // probably only needed in manual/interactive calls to the above "om-expect-tests" script:)
      // these also seem not needed (like the waitFor, below):
//      expect.sendLine();
//      expect.send("q");
//      Thread.sleep(250);
//      expect.sendLine();
//      expect.send("q");
      /*
       */


     /* more expectit example code, but see the file core/testsuite/README for better references:
      import static net.sf.expectit.matcher.Matchers.*;
      expect.sendLine("ls -lh");
      // capture the total
      String total = expect.expect(regexp("^total (.*)")).group(1);
      System.out.println("Size: " + total);
      // capture file list
      String list = expect.expect(regexp("\n$")).getBefore();
      // print the result
      System.out.println("List: " + list);
      expect.sendLine("exit");
      // expect the process to finish
      expect.expect(eof());    */

      // seems not needed, causes ide to pause too long under some conditions (like a long timeout for debugging, and an expect (match) failure):
      //process.waitFor();
    } catch (ExpectIOException e) {
      if (result == null) {
        throw e;
      } else {
        System.out.println("Didn't match with: " + result.getBefore());
      }
    }
  }
}