/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003-2004 and 2008-2015 inclusive, Luke A Call; all rights reserved.
    (That copyright statement was previously 2013-2015, until I remembered that much of Controller came from TextUI.scala and TextUI.java before that.)
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule, guidelines around binary
    distribution, and the GNU Affero General Public License as published by the Free Software Foundation, either version 3
    of the License, or (at your option) any later version.  See the file LICENSE for details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
package org.onemodel.controller

import org.onemodel.TextUI
import org.onemodel.model.{IdWrapper, EntityClass, Entity}
import org.onemodel.database.PostgreSQLDatabase

class ClassMenu(override val ui: TextUI, dbInOVERRIDESmDBWhichHasANewDbConnectionTHATWEDONTWANT: PostgreSQLDatabase) extends Controller(ui) {
  override val mDB = dbInOVERRIDESmDBWhichHasANewDbConnectionTHATWEDONTWANT

  /** returns None if user wants out. */
  //@tailrec //see comment re this on EntityMenu
  //scoping idea: see idea at beginning of EntityMenu.entityMenu
  def classMenu(classIn: EntityClass): Option[EntityClass] = {
    try {
      require(classIn != null)
      val leadingText: Array[String] = Array[String]("CLASS: " + classIn.getDisplayString)
      val choices = Array[String]("(stub: classes can be added while creating a new entity)" /*"add"' option, if needed*/ ,
                                  "(stub)" /*"sort" if needed*/ ,
                                  "Edit class name",
                                  "Delete",
                                  "Go to defining entity",
                                  "Search (List all entities in this class)")
      val response = ui.askWhich(Some(leadingText), choices)
      if (response == None) None
      else {
        val answer = response.get
        if (answer == 3) {
          askForAndWriteClassAndDefiningEntityName(Some(classIn.getId), Some(classIn.getName))
          classMenu(new EntityClass(mDB, classIn.getId))
        }
        else if (answer == 4) {
          val entitiesCount: Long = mDB.getEntitiesOnlyCount(Some(classIn.getId), limitByClass = true, Some(classIn.getDefiningEntityId))
          if (entitiesCount > 0) {
            ui.displayText("Can not delete class, because it is the class of " + entitiesCount + " entities.")
          } else {
            val name = classIn.getName
            val definingEntityName: String = new Entity(mDB, classIn.getDefiningEntityId).getName
            val ans = ui.askYesNoQuestion("DELETE CLASS \"" + name + "\" AND its defining ENTITY \"" + definingEntityName + "\" with " +
                                          entityPartsThatCanBeAffected + ".  **ARE YOU REALLY SURE?**")
            if (ans != None && ans.get) {
              classIn.delete()
              ui.displayText("Deleted class \"" + name + "\"" + ".")
              val selection: Option[IdWrapper] = chooseOrCreateObject(None, None, None, Controller.ENTITY_CLASS_TYPE)
              if (selection != None) classMenu(new EntityClass(mDB, selection.get.getId))
            } else {
              ui.displayText("Did not delete class.", waitForKeystroke = false)
            }
          }
          classMenu(classIn)
        } else if (answer == 5) {
          new EntityMenu(ui,mDB).entityMenu(0, new Entity(mDB, classIn.getDefiningEntityId))
          classMenu(new EntityClass(mDB, classIn.getId))
        } else if (answer == 6) {
          val selection: Option[IdWrapper] = chooseOrCreateObject(None, None, Some(classIn.getDefiningEntityId), Controller.ENTITY_TYPE, 0,
                                                                               Some(classIn.getId),
                                                                               limitByClassIn = true)
          if (selection != None) new EntityMenu(ui,mDB).entityMenu(0, new Entity(mDB, selection.get.getId))
          classMenu(new EntityClass(mDB, classIn.getId))
        } else {
          //textui doesn't actually let the code get here, but:
          ui.displayText("invalid response")
          classMenu(classIn)
        }
      }
    }
    catch {
      case e: Exception =>
        showException(e)
        val ans = ui.askYesNoQuestion("Go back to what you were doing (vs. going out)?",Some("y"))
        if (ans != None && ans.get) classMenu(classIn)
        else None
    }
  }

}
