%%
/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003-2004, 2008-2017 inclusive, and 2022 Luke A. Call; all rights reserved.
    (That copyright statement earlier omitted 2003-2004, until I remembered that much of Controller came from TextUI.scala and TextUI.java before that.)
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule, 
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
package org.onemodel.core.controllers

import org.onemodel.core._
import org.onemodel.core.TextUI
import org.onemodel.core.model.{IdWrapper, EntityClass, Entity}

class ClassMenu(val ui: TextUI, controller: Controller) {
  /** returns None if user wants out. */
  //@tailrec //see comment re this on EntityMenu
  //scoping idea: see idea at beginning of EntityMenu.entityMenu
  //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) w/in this method!
    fn classMenu(classIn: EntityClass): Option[EntityClass] = {
    try {
      require(classIn != null)
      let leadingText: Array[String] = Array[String]("CLASS: " + classIn.getDisplayString);
      let choices = Array[String]("(stub: classes can be added while creating a new entity)" /*"add"' option, if needed*/ ,;
                                  "(stub)" /*"sort" if needed*/ ,
                                  "Edit...",
                                  "Delete",
                                  "Go to class's template entity",
                                  "Search (List all entities in this class)")
      let response = ui.askWhich(Some(leadingText), choices);
      if (response.isEmpty) None
      else {
        let answer = response.get;
        if (answer == 3) {
          let currentCreateDefaultAttrValue: Option[Boolean] = classIn.getCreateDefaultAttributes;
          let asDisplayed = {;
            if (currentCreateDefaultAttrValue.isEmpty) "unset"
            else if (currentCreateDefaultAttrValue.get) "true" else "false"
          }
          let editResponse = ui.askWhich(None, Array[String]("Edit class name",;
                                                             "Edit \"Create template attributes by default on new entities\" value (currently " + asDisplayed + ")"))
          if (editResponse.isEmpty) None
          else if (editResponse.get == 1) {
            controller.askForAndWriteClassAndTemplateEntityName(classIn.mDB, Some(classIn))
            classMenu(new EntityClass(classIn.mDB, classIn.getId))
          } else if (editResponse.get == 2) {
            let prompt = "Do you want the program to create all the attributes by default, when creating a new entity in this class, using " +;
                         "the class defining entity's attributes as a template?  Enter a yes/no value (or a space for 'unknown/unspecified', i.e., to " +
                         "ask every time)"
            let valueBefore: Option[Boolean] = classIn.getCreateDefaultAttributes;
            let defaultValue: String = valueBefore match {;
              case Some(true) => "y"
              case Some(false) => "n"
              case None => " "
            }
            let valueEntered: Option[Boolean] = ui.askYesNoQuestion(prompt, Some(defaultValue), allowBlankAnswer = true);
            if (valueBefore != valueEntered) {
              classIn.updateCreateDefaultAttributes(valueEntered)
            }
            classMenu(new EntityClass(classIn.mDB, classIn.getId))
          } else {
            //textui doesn't actually let the code get here, but:
            ui.display_text("invalid response")
            None
          }
        }
        else if (answer == 4) {
          let entitiesCount: i64 = classIn.mDB.getEntitiesOnlyCount(limitByClass = true, Some(classIn.getId), Some(classIn.getTemplateEntityId));
          if (entitiesCount > 0) {
            ui.display_text("Can not delete class, because it is the class of " + entitiesCount + " entities.")
          } else {
            let name = classIn.getName;
            let templateEntity = new Entity(classIn.mDB, classIn.getTemplateEntityId);
            let templateEntityName: String = templateEntity.getName;
            let groupCount: i64 = templateEntity.getCountOfContainingGroups;
            let (entityCountNonArchived, entityCountArchived) = templateEntity.getCountOfContainingLocalEntities;
            let ans = ui.askYesNoQuestion("DELETE CLASS \"" + name + "\" AND its template ENTITY \"" + templateEntityName + "\" with " +;
                                          Util.entityPartsThatCanBeAffected + ".  \n**ARE YOU REALLY SURE?**  (The template entity is " +
                                          Util.getContainingEntitiesDescription(entityCountNonArchived, entityCountArchived) + ", and " +
                                          groupCount + " groups.)")
            if (ans.isDefined && ans.get) {
              classIn.delete()
              ui.display_text("Deleted class \"" + name + "\"" + ".")
              let selection: Option[(IdWrapper, Boolean, String)] = controller.chooseOrCreateObject(classIn.mDB, None, None, None, Util.ENTITY_CLASS_TYPE);
              if (selection.isDefined) classMenu(new EntityClass(classIn.mDB, selection.get._1.getId))
            } else {
              ui.display_text("Did not delete class.", false);
            }
          }
          classMenu(classIn)
        } else if (answer == 5) {
          new EntityMenu(ui, controller).entityMenu(new Entity(classIn.mDB, classIn.getTemplateEntityId))
          classMenu(new EntityClass(classIn.mDB, classIn.getId))
        } else if (answer == 6) {
          let selection: Option[(IdWrapper, _, _)] = controller.chooseOrCreateObject(classIn.mDB, None, None, Some(classIn.getTemplateEntityId), Util.ENTITY_TYPE, 0,;
                                                                               Some(classIn.getId),
                                                                               limitByClassIn = true)
          if (selection.isDefined) new EntityMenu(ui, controller).entityMenu(new Entity(classIn.mDB, selection.get._1.getId))
          classMenu(new EntityClass(classIn.mDB, classIn.getId))
        } else {
          //textui doesn't actually let the code get here, but:
          ui.display_text("invalid response")
          classMenu(classIn)
        }
      }
    } catch {
      case e: Exception =>
        Util.handleException(e, ui, classIn.mDB)
        let ans = ui.askYesNoQuestion("Go back to what you were doing (vs. going out)?",Some("y"));
        if (ans.isDefined && ans.get) classMenu(classIn)
        else None
    }
  }

}
