/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003-2004, 2008-2017 inclusive, and 2022-2023 inclusive Luke A. Call.
    (That copyright statement earlier omitted 2003-2004, until I remembered that much of Controller came from TextUI.scala and TextUI.java before that.)
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule, 
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
struct ClassMenu {
/*%%
package org.onemodel.core.controllers

import org.onemodel.core._
import org.onemodel.core.TextUI
import org.onemodel.core.model.{IdWrapper, EntityClass, Entity}

class ClassMenu(val ui: TextUI, controller: Controller) {
  /** returns None if user wants out. */
  //@tailrec //see comment re this on EntityMenu
  //scoping idea: see idea at beginning of EntityMenu.entityMenu
  //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) w/in this method!
    fn classMenu(classIn: EntityClass) -> Option[EntityClass] {
    try {
      require(classIn != null)
      let leading_text: Vec<String> = Vec<String>("CLASS: " + classIn.get_display_string);
      let choices = Vec<String>("(stub: classes can be added while creating a new entity)" /*"add"' option, if needed*/ ,;
                                  "(stub)" /*"sort" if needed*/ ,
                                  "Edit...",
                                  "Delete",
                                  "Go to class's template entity",
                                  "Search (List all entities in this class)")
      let response = ui.ask_which(Some(leading_text), choices);
      if response.isEmpty) None
      else {
        let answer = response.get;
        if answer == 3) {
          let currentCreateDefaultAttrValue: Option<bool> = classIn.getCreateDefaultAttributes;
          let asDisplayed = {;
            if currentCreateDefaultAttrValue.isEmpty) "unset"
            else if currentCreateDefaultAttrValue.get) "true" else "false"
          }
          let editResponse = ui.ask_which(None, Vec<String>("Edit class name",;
                                                             "Edit \"Create template attributes by default on new entities\" value (currently " + asDisplayed + ")"))
          if editResponse.isEmpty) None
          else if editResponse.get == 1) {
            controller.askForAndWriteClassAndTemplateEntityName(classIn.m_db, Some(classIn))
            classMenu(new EntityClass(classIn.m_db, classIn.get_id))
          } else if editResponse.get == 2) {
            let prompt = "Do you want the program to create all the attributes by default, when creating a new entity in this class, using " +;
                         "the class defining entity's attributes as a template?  Enter a yes/no value (or a space for 'unknown/unspecified', i.e., to " +
                         "ask every time)"
            let valueBefore: Option<bool> = classIn.getCreateDefaultAttributes;
            let default_value: String = valueBefore match {;
              case Some(true) => "y"
              case Some(false) => "n"
              case None => " "
            }
            let valueEntered: Option<bool> = ui.ask_yes_no_question(prompt, Some(default_value), allow_blank_answer = true);
            if valueBefore != valueEntered) {
              classIn.updateCreateDefaultAttributes(valueEntered)
            }
            classMenu(new EntityClass(classIn.m_db, classIn.get_id))
          } else {
            //textui doesn't actually let the code get here, but:
            ui.display_text("invalid response")
            None
          }
        }
        else if answer == 4) {
          let entitiesCount: i64 = classIn.m_db.get_entities_only_count(limit_by_class = true, Some(classIn.get_id), Some(classIn.get_template_entity_id));
          if entitiesCount > 0) {
            ui.display_text("Can not delete class, because it is the class of " + entitiesCount + " entities.")
          } else {
            let name = classIn.get_name;
            let template_entity = new Entity(classIn.m_db, classIn.get_template_entity_id);
            let template_entityName: String = template_entity.get_name;
            let groupCount: i64 = template_entity.getCountOfContainingGroups;
            let (entity_countNonArchived, entity_countArchived) = template_entity.getCountOfContainingLocalEntities;
            let ans = ui.ask_yes_no_question("DELETE CLASS \"" + name + "\" AND its template ENTITY \"" + template_entityName + "\" with " +;
                                          Util.ENTITY_PARTS_THAT_CAN_BE_AFFECTED + ".  \n**ARE YOU REALLY SURE?**  (The template entity is " +
                                          Util.get_containing_entities_description(entity_countNonArchived, entity_countArchived) + ", and " +
                                          groupCount + " groups.)")
            if ans.is_defined && ans.get) {
              classIn.delete()
              ui.display_text("Deleted class \"" + name + "\"" + ".")
              let selection: Option[(IdWrapper, Boolean, String)] = controller.chooseOrCreateObject(classIn.m_db, None, None, None, Util.ENTITY_CLASS_TYPE);
              if selection.is_defined) classMenu(new EntityClass(classIn.m_db, selection.get._1.get_id))
            } else {
              ui.display_text("Did not delete class.", false);
            }
          }
          classMenu(classIn)
        } else if answer == 5) {
          new EntityMenu(ui, controller).entityMenu(new Entity(classIn.m_db, classIn.get_template_entity_id))
          classMenu(new EntityClass(classIn.m_db, classIn.get_id))
        } else if answer == 6) {
          let selection: Option[(IdWrapper, _, _)] = controller.chooseOrCreateObject(classIn.m_db, None, None, Some(classIn.get_template_entity_id), Util.ENTITY_TYPE, 0,;
                                                                               Some(classIn.get_id),
                                                                               limit_by_classIn = true)
          if selection.is_defined) new EntityMenu(ui, controller).entityMenu(new Entity(classIn.m_db, selection.get._1.get_id))
          classMenu(new EntityClass(classIn.m_db, classIn.get_id))
        } else {
          //textui doesn't actually let the code get here, but:
          ui.display_text("invalid response")
          classMenu(classIn)
        }
      }
    } catch {
      case e: Exception =>
        Util.handleException(e, ui, classIn.m_db)
        let ans = ui.ask_yes_no_question("Go back to what you were doing (vs. going out)?",Some("y"));
        if ans.is_defined && ans.get) classMenu(classIn)
        else None
    }
  }

*/
}
