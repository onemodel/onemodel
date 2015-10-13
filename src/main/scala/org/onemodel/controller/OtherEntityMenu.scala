/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2015-2015 inclusive, Luke A. Call; all rights reserved.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule, guidelines around binary
    distribution, and the GNU Affero General Public License as published by the Free Software Foundation, either version 3
    of the License, or (at your option) any later version.  See the file LICENSE for details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
package org.onemodel.controller

import org.onemodel.TextUI
import org.onemodel.database.PostgreSQLDatabase
import org.onemodel.model._

/** This is simply to hold less-used operations so the main EntityMenu can be the most-used stuff.
  */
class OtherEntityMenu (val ui: TextUI, val db: PostgreSQLDatabase, val controller: Controller) {

  def otherEntityMenu(entityIn: Entity): Option[Entity] = {
    try {
      require(entityIn != null)
      val leadingText = Array[String]{"**CURRENT ENTITY " + entityIn.getId + ": " + entityIn.getDisplayString}
      val choices = Array[String]("Edit public/nonpublic status",
                                  "Import/Export...")
      val response = ui.askWhich(Some(leadingText), choices)
      if (response.isEmpty) None
      else {
        val answer = response.get
        if (answer == 1) {
          // The condition for this (when it was part of EntityMenu) used to include " && !entityIn.isInstanceOf[RelationType]", but maybe it's better w/o that.
          controller.editEntityPublicStatus(entityIn)
          // reread from db to refresh data for display, like public/non-public status:
          otherEntityMenu(new Entity(db, entityIn.getId))
        }
        else if (answer == 2) {
          val importOrExportAnswer = ui.askWhich(None, Array("Import", "Export to a text file (outline)", "Export to html pages"), Array[String]())
          if (importOrExportAnswer.isDefined) {
            if (importOrExportAnswer.get == 1) new ImportExport(ui, db, controller).importCollapsibleOutlineAsGroups(entityIn)
            else if (importOrExportAnswer.get == 2) new ImportExport(ui, db, controller).export(entityIn, ImportExport.TEXT_EXPORT_TYPE, None)
            else if (importOrExportAnswer.get == 3) {
              // idea (in task list):  have the date default to the entity creation date, then later add/replace that (w/ range or what for ranges?)
              // with the last edit date, when that feature exists.
              val copyrightYearAndName = ui.askForString(Some(Array("Enter copyright year(s) and holder's name, i.e., the \"2015 John Doe\" part " +
                                                                    "of \"Copyright 2015 John Doe\" (This accepts HTML so can also be used for a " +
                                                                    "page footer, for example.)")))
              if (copyrightYearAndName.isDefined && copyrightYearAndName.get.trim.nonEmpty) {
                new ImportExport(ui, db, controller).export(entityIn, ImportExport.HTML_EXPORT_TYPE, copyrightYearAndName)
              }
            }
          }
          otherEntityMenu(entityIn)
        } else {
          ui.displayText("invalid response")
          otherEntityMenu(entityIn)
        }
      }
    } catch {
      case e: Exception =>
        controller.handleException(e)
        val ans = ui.askYesNoQuestion("Go back to what you were doing (vs. going out)?", Some("y"))
        if (ans.isDefined && ans.get) otherEntityMenu(entityIn)
        else None
    }
  }

}
