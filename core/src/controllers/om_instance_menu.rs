/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2017 and 2023 inclusive, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
struct OmInstanceMenu {
/*%%
package org.onemodel.core.controllers

import org.onemodel.core.TextUI
import org.onemodel.core.model.OmInstance

class OmInstanceMenu(val ui: TextUI, controller: Controller) {
  /** returns None if user wants out. */
  //@tailrec //see comment re this on EntityMenu
  //scoping idea: see idea at beginning of EntityMenu.entityMenu
  //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) w/in this method!
    fn omInstanceMenu(omInstanceIn: OmInstance) -> Option[OmInstance] {
    try {
      require(omInstanceIn != null)
      let leading_text: Vec<String> = Vec<String>("OneModel Instance " + omInstanceIn.get_display_string);
      let choices = Vec<String>("(stub)", /*"Add" would typically be here if needed, but that is provided off the MainMenu. */;
                                  "(stub)" /*"sort" if needed*/ ,
                                  "Edit...",
                                  if !omInstanceIn.get_local) "Delete" else "(Can't delete a local instance)")
      let response = ui.ask_which(Some(leading_text), choices);
      if response.isEmpty) None
      else {
        let answer = response.get;
        if answer == 3) {
          let id: Option<String> = controller.askForAndWriteOmInstanceInfo(omInstanceIn.db, Some(omInstanceIn));
          if id.is_defined) {
            // possible was some modification; reread from db to get new values:
            omInstanceMenu(new OmInstance(omInstanceIn.db, id.get))
          } else {
            omInstanceMenu(omInstanceIn)
          }
        } else if answer == 4 && !omInstanceIn.get_local) {
          let deleteAnswer = ui.ask_yes_no_question("Delete this link to a separate OneModel instance: are you sure?", allow_blank_answer = true);
          if deleteAnswer.is_defined && deleteAnswer.get) {
            omInstanceIn.delete()
            None
          } else {
            omInstanceMenu(omInstanceIn)
          }
        } else {
          //textui doesn't actually let the code get here, but:
          ui.display_text("invalid response")
          omInstanceMenu(omInstanceIn)
        }
      }
    } catch {
      case e: Exception =>
        org.onemodel.core.Util.handleException(e, ui, omInstanceIn.db)
        let ans = ui.ask_yes_no_question("Go back to what you were doing (vs. going out)?",Some("y"));
        if ans.is_defined && ans.get) omInstanceMenu(omInstanceIn)
        else None
    }
  }
*/
}
