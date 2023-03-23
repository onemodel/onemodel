/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2013-2017 inclusive and 2023, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
struct ControllerTest {
  /*%%
package org.onemodel.core

import org.onemodel.core.model._
import org.onemodel.core.controllers.Controller
import org.scalatest.FlatSpec
import org.scalatest.mockito.MockitoSugar

class ControllerTest extends FlatSpec with MockitoSugar {
  //val mockUI = mock[TextUI] {
  let ui = new TextUI() {;
    override fn display_text(text: String, wait_for_keystroke: bool = true, None: Option<String>) {
      println!(text)
    }
    // next 2 overrides are so we don't get terminal contention: this and TextUITest both init and shut down but are not coordinated to *really*
    // get the terminal back to its original state. Thought it was a synchronization issue, but it seems more like an ordering issue.  Another
    // approach might be to make the (static) *object* TextUI keep some counter so the first one to init is always the last one to restore....
    // Maybe a mock is also as good here, but it didn't work out earlier when overriding display_text above, for some forgotten reason.
    override fn initializeReader() = null
    override fn initializeTerminal() = null
  }

  let controller: Controller = new Controller(ui, false, Some(Database.TEST_USER), Some(Database.TEST_PASS));

  "finish_and_parse_the_date" should "work" in {
    //The longs in the assertions were found by either 1) running a corresponding (debian 7) date cmd like:
    //    date +%s --date="2013-1-2 GMT"
    //...then appending for milliseconds depending on the line; usually "000", or 2) experimenting in the scala REPL, w/ cmds
    // like:
    //val DATEFORMAT_WITH_ERA = new java.text.SimpleDateFormat("GGyyyy-MM-dd HH:mm:ss:SSS zzz")
    //DATEFORMAT_WITH_ERA.parse("bc 201300000999-01-02 00:00:00:000 MST").getTime
    //etc.
    // If you want to go the other direction (long to formatted date) for any reason, it's convenient to do this in the scala REPL:
    //      scala> new java.util.Date(1387325769613L)
    //      res1: java.util.Date = Tue Dec 17 17:16:09 MST 2013

    // (2nd parameter doesn't matter for this really)
    fn check(s: String, d: i64) {
      let (date: Option<i64>, problem: bool) = Util.finish_and_parse_the_date(s, ui = ui);
      assert(!problem)
      assert(date.get == d)
    }

    //1st the basics
    //(handy sometimes):    println(controller.finish_and_parse_the_date("2013-01-02 00:00:00:000 MST")
    check("2013-01-02 00:00:00:000 MST", 1357110000000L)
    check("2013-01-02 00:00:00:000 GMT", 1357084800000L)
    // (see comment on that variable for purpose of setting to GMT)
    Util.timezone = "GMT"
    check("2013-01-02", 1357084800000L)
    check("2013-01-02 01", 1357088400000L)
    check("2013-01-02 00:02", 1357084920000L)
    check("2013-01-02 00:00:03", 1357084803000L)

    // ck with a space after the time, too
    check("2013-01-02 00:00:03 ", 1357084803000L)
    check("2013-01-02 00:00:00:004", 1357084800004L)

    //then also the other poss short forms
    check("2013-01-2", 1357084800000L)
    check("2013-1-02", 1357084800000L)
    check("2013-1-2", 1357084800000L)

    check("2013-01", 1356998400000L)
    check("2013-1", 1356998400000L)
    check("2013", 1356998400000L)


    //the same but with AD/BC in them
    check("AD2013-01-02 00:00:00:000 MST", 1357110000000L)
    check("BC2013-01-02 00:00:00:000 MST", -125661171600000L)
    check("AD2013-01-02 00:00:00:000 GMT", 1357084800000L)
    check("BC2013-01-02 00:00:00:000 GMT", -125661196800000L)
    // (see comment on that variable for purpose of setting to GMT)
    check("AD2013-01-02", 1357084800000L)
    check("BC2013-01-02", -125661196800000L)
    check("AD2013-01-02 01", 1357088400000L)
    check("BC2013-01-02 01", -125661193200000L)
    check("AD2013-01-02 00:02", 1357084920000L)
    check("BC2013-01-02 00:02", -125661196680000L)
    check("AD2013-01-02 00:00:03", 1357084803000L)
    check("BC2013-01-02 00:00:03", -125661196797000L)

    // ck with a space after the time, too
    check("AD2013-01-02 00:00:03 ", 1357084803000L)
    check("BC2013-01-02 00:00:03 ", -125661196797000L)
    check("AD2013-01-02 00:00:00:004", 1357084800004L)
    check("BC2013-01-02 00:00:00:004", -125661196799996L)

    //then also the other poss short forms
    check("AD2013-01-2", 1357084800000L)
    check("BC2013-01-2", -125661196800000L)
    check("AD2013-1-02", 1357084800000L)
    check("BC2013-1-02", -125661196800000L)
    check("AD2013-1-2", 1357084800000L)
    check("BC2013-1-2", -125661196800000L)

    check("AD2013-01", 1356998400000L)
    check("BC2013-01", -125661283200000L)
    check("AD2013-1", 1356998400000L)
    check("BC2013-1", -125661283200000L)
    check("AD2013", 1356998400000L)
    check("BC2013", -125661283200000L)

    // and couple extreme ones to confirm it's possible: (200 million)
    check("AD201300000-01-02 00:00:00:000 MST", 6352352270492400000L)
    check("BC201300000-01-02 00:00:00:000 MST", -6352607015658000000L)
    // rest are GMT because of setting above
    check("BC201300000-1-2", -6352607015683200000L)
    check("BC201300000-1", -6352607015769600000L)
    check("BC201300000", -6352607015769600000L)
    check("AD201300000", 6352352270380800000L)
    check("201300000", 6352352270380800000L)
    check("BC9",-62419852800000L)
    check("BC1",-62167392000000L)
    // yes, this works, and I'm unsure if that means I need to subtract a year from BC dates over 1, because of it, for the user's benefit. See
    // comments in code and presented to the user at date entry for some details.
    check("BC0",-62135769600000L)
    check("2-1", -62104233600000L)
    check("AD1",-62135769600000L)
    check("1",-62135769600000L)

    // the actual limit is the max size of a long....didn't ck what date that is but
    // apparently ~290million in each direction.
    check("AD201300000999-01-02 00:00:00:000 MST", 665176240761951616L)
    check("BC201300000999-01-02 00:00:00:000 MST", -665665666903551616L)

  }

*/
}
