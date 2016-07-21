/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003-2004 and 2008-2016 inclusive, Luke A Call; all rights reserved.
    (That copyright statement earlier omitted 2003-2004, until I remembered that much of Controller came from TextUI.scala and TextUI.java before that.)
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

class ClassMenu(val ui: TextUI, db: PostgreSQLDatabase, controller: Controller) {
  /** returns None if user wants out. */
  //@tailrec //see comment re this on EntityMenu
  //scoping idea: see idea at beginning of EntityMenu.entityMenu
  def classMenu(classIn: EntityClass): Option[EntityClass] = {
    try {
      require(classIn != null)
      val leadingText: Array[String] = Array[String]("CLASS: " + classIn.getDisplayString)
      val choices = Array[String]("(stub: classes can be added while creating a new entity)" /*"add"' option, if needed*/ ,
                                  "(stub)" /*"sort" if needed*/ ,
                                  "Edit...",
                                  "Delete",
                                  "Go to defining entity",
                                  "Search (List all entities in this class)")
      val response = ui.askWhich(Some(leadingText), choices)
      if (response.isEmpty) None
      else {
        val answer = response.get
        if (answer == 3) {
          val currentCreateDefaultAttrValue: Option[Boolean] = classIn.getCreateDefaultAttributes
          val asDisplayed = {
            if (currentCreateDefaultAttrValue.isEmpty) "unset"
            else if (currentCreateDefaultAttrValue.get) "true" else "false"
          }
          val editResponse = ui.askWhich(None, Array[String]("Edit class name",
                                                             "Edit \"Create template attributes by default on new entities\" value (currently " + asDisplayed + ")"))
          if (editResponse.isEmpty) None
          else if (editResponse.get == 1) {
            controller.askForAndWriteClassAndDefiningEntityName(Some(classIn.getId), Some(classIn.getName))
            classMenu(new EntityClass(db, classIn.getId))
          } else if (editResponse.get == 2) {
            val prompt = "Do you want the program to create all the attributes by default, when creating a new entity in this class, using " +
                         "the class defining entity's attributes as a template?  Enter a yes/no value (or a space for 'unknown/unspecified', i.e., to " +
                         "ask every time)"
            val valueBefore: Option[Boolean] = db.getClassCreateDefaultAttributes(classIn.getId)
            val defaultValue: String = valueBefore match {
              case Some(true) => "y"
              case Some(false) => "n"
              case None => " "
            }
            val valueEntered: Option[Boolean] = ui.askYesNoQuestion(prompt, Some(defaultValue), allowBlankAnswer = true)
            if (valueBefore != valueEntered) {
              db.updateClassCreateDefaultAttributes(classIn.getId, valueEntered)
            }
            classMenu(new EntityClass(db, classIn.getId))
          } else {
            //textui doesn't actually let the code get here, but:
            ui.displayText("invalid response")
            None
          }
        }
        else if (answer == 4) {
          val entitiesCount: Long = db.getEntitiesOnlyCount(Some(classIn.getId), limitByClass = true, Some(classIn.getDefiningEntityId))
          if (entitiesCount > 0) {
            ui.displayText("Can not delete class, because it is the class of " + entitiesCount + " entities.")
          } else {
            val name = classIn.getName
            val definingEntityName: String = new Entity(db, classIn.getDefiningEntityId).getName
            val groupCount: Long = db.getCountOfGroupsContainingEntity(classIn.getDefiningEntityId)
            val (entityCountNonArchived, entityCountArchived) = db.getCountOfEntitiesContainingEntity(classIn.getDefiningEntityId)
            val ans = ui.askYesNoQuestion("DELETE CLASS \"" + name + "\" AND its defining ENTITY \"" + definingEntityName + "\" with " +
                                          controller.entityPartsThatCanBeAffected + ".  **ARE YOU REALLY SURE?**  (The defining entity is " +
                                         controller.getContainingEntitiesDescription(entityCountNonArchived, entityCountArchived) + ", and " +
                                          groupCount + " groups.)")
            if (ans.isDefined && ans.get) {
              classIn.delete()
              ui.displayText("Deleted class \"" + name + "\"" + ".")
              val selection: Option[IdWrapper] = controller.chooseOrCreateObject(None, None, None, Controller.ENTITY_CLASS_TYPE)
              if (selection.isDefined) classMenu(new EntityClass(db, selection.get.getId))
            } else {
              ui.displayText("Did not delete class.", waitForKeystroke = false)
            }
          }
          classMenu(classIn)
        } else if (answer == 5) {
          new EntityMenu(ui, db, controller).entityMenu(new Entity(db, classIn.getDefiningEntityId))
          classMenu(new EntityClass(db, classIn.getId))
        } else if (answer == 6) {
          val selection: Option[IdWrapper] = controller.chooseOrCreateObject(None, None, Some(classIn.getDefiningEntityId), Controller.ENTITY_TYPE, 0,
                                                                               Some(classIn.getId),
                                                                               limitByClassIn = true)
          if (selection.isDefined) new EntityMenu(ui, db, controller).entityMenu(new Entity(db, selection.get.getId))
          classMenu(new EntityClass(db, classIn.getId))
        } else {
          //textui doesn't actually let the code get here, but:
          ui.displayText("invalid response")
          classMenu(classIn)
        }
      }
    }
    catch {
      case e: Exception =>
        controller.handleException(e)
        val ans = ui.askYesNoQuestion("Go back to what you were doing (vs. going out)?",Some("y"))
        if (ans.isDefined && ans.get) classMenu(classIn)
        else None
    }
  }

}
