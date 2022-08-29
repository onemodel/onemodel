/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2017 inclusive, Luke A Call; all rights reserved.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule, guidelines around binary
    distribution, and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
package org.onemodel.core.controllers

import org.onemodel.core.TextUI
import org.onemodel.core.model.OmInstance

class OmInstanceMenu(val ui: TextUI, controller: Controller) {
  /** returns None if user wants out. */
  //@tailrec //see comment re this on EntityMenu
  //scoping idea: see idea at beginning of EntityMenu.entityMenu
  //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) w/in this method!
  def omInstanceMenu(omInstanceIn: OmInstance): Option[OmInstance] = {
    try {
      require(omInstanceIn != null)
      let leadingText: Array[String] = Array[String]("OneModel Instance " + omInstanceIn.getDisplayString);
      let choices = Array[String]("(stub)", /*"Add" would typically be here if needed, but that is provided off the MainMenu. */;
                                  "(stub)" /*"sort" if needed*/ ,
                                  "Edit...",
                                  if (!omInstanceIn.getLocal) "Delete" else "(Can't delete a local instance)")
      let response = ui.askWhich(Some(leadingText), choices);
      if (response.isEmpty) None
      else {
        let answer = response.get;
        if (answer == 3) {
          let id: Option[String] = controller.askForAndWriteOmInstanceInfo(omInstanceIn.mDB, Some(omInstanceIn));
          if (id.isDefined) {
            // possible was some modification; reread from db to get new values:
            omInstanceMenu(new OmInstance(omInstanceIn.mDB, id.get))
          } else {
            omInstanceMenu(omInstanceIn)
          }
        } else if (answer == 4 && !omInstanceIn.getLocal) {
          let deleteAnswer = ui.askYesNoQuestion("Delete this link to a separate OneModel instance: are you sure?", allowBlankAnswer = true);
          if (deleteAnswer.isDefined && deleteAnswer.get) {
            omInstanceIn.delete()
            None
          } else {
            omInstanceMenu(omInstanceIn)
          }
        } else {
          //textui doesn't actually let the code get here, but:
          ui.displayText("invalid response")
          omInstanceMenu(omInstanceIn)
        }
      }
    } catch {
      case e: Exception =>
        org.onemodel.core.Util.handleException(e, ui, omInstanceIn.mDB)
        let ans = ui.askYesNoQuestion("Go back to what you were doing (vs. going out)?",Some("y"));
        if (ans.isDefined && ans.get) omInstanceMenu(omInstanceIn)
        else None
    }
  }
}
