/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2016 inclusive, Luke A Call; all rights reserved.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule, guidelines around binary
    distribution, and the GNU Affero General Public License as published by the Free Software Foundation, either version 3
    of the License, or (at your option) any later version.  See the file LICENSE for details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
package org.onemodel.core.controllers

import org.onemodel.core.TextUI
import org.onemodel.core.model.{Database, OmInstance}

class OmInstanceMenu(val ui: TextUI, db: Database, controller: Controller) {
  /** returns None if user wants out. */
  //@tailrec //see comment re this on EntityMenu
  //scoping idea: see idea at beginning of EntityMenu.entityMenu
  //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) w/in this method!
  def omInstanceMenu(omInstanceIn: OmInstance): Option[OmInstance] = {
    try {
      require(omInstanceIn != null)
      val leadingText: Array[String] = Array[String]("OneModel Instance " + omInstanceIn.getDisplayString)
      val choices = Array[String]("(stub)", /*"Add" would typically be here if needed, but that is provided off the MainMenu. */
                                  "(stub)" /*"sort" if needed*/ ,
                                  "Edit...",
                                  if (!omInstanceIn.getLocal) "Delete" else "(Can't delete a local instance)")
      val response = ui.askWhich(Some(leadingText), choices)
      if (response.isEmpty) None
      else {
        val answer = response.get
        if (answer == 3) {
          val id: Option[String] = controller.askForAndWriteOmInstanceInfo(Some(omInstanceIn.getId), Some(omInstanceIn.getAddress))
          if (id.isDefined) {
            omInstanceMenu(new OmInstance(db, id.get))
          } else {
            omInstanceMenu(omInstanceIn)
          }
        } else if (answer == 4 && !omInstanceIn.getLocal) {
          val deleteAnswer = ui.askYesNoQuestion("Delete this link to a separate OneModel instance: are you sure?", allowBlankAnswer = true)
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
        org.onemodel.core.Util.handleException(e, controller.ui, controller.db)
        val ans = ui.askYesNoQuestion("Go back to what you were doing (vs. going out)?",Some("y"))
        if (ans.isDefined && ans.get) omInstanceMenu(omInstanceIn)
        else None
    }
  }
}
