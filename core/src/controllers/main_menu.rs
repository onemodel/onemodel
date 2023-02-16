/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003-2004 and 2008-2017 inclusive, and 2023, Luke A. Call.
    (That copyright statement was previously 2013-2015, until I remembered that much of Controller came from TextUI.scala and TextUI.java before that.)
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
struct MainMenu {
/*%%
package org.onemodel.core.controllers

import org.onemodel.core._
import org.onemodel.core.model._

class MainMenu(val ui: TextUI, let db: Database, val controller: Controller)  {;
  /** See caller in start() for description of the 2nd parameter. */
  // Removed next line @tailrec because 1) it gets errors about "recursive call not in tail position" (which could be fixed by removing the last call to itself,
  // but for the next reason), and 2) it means the user can't press ESC to go "back" to previously viewed entities.
  // ******* IF THIS IS CHANGED BACK, IN THE FUTURE, and we go back to "@tailrec", then we could use a stack of prior entities to for the user to go "back"
  // to, when hitting ESC from the main menu (like the one removed with the same checkin as this writing, but perhaps simplified).
  // @tailrec
  //scoping idea: see idea at beginning of EntityMenu.entityMenu
    fn mainMenu(entity_in: Option<Entity> = None, goDirectlyToChoice: Option[Int] = None) {
    //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) w/in this method! (if it fits the situation)
    try {
      let numEntities = db.getEntitiesOnlyCount();
      if numEntities == 0 || entity_in.isEmpty) {
        let choices: List[String] = List[String]("Add new entity (such as yourself using your name, to start)",;
                                                 Util.MAIN_SEARCH_PROMPT)
        let response: Option[Int] = ui.ask_which(None, choices.toArray, Vec<String>(), includeEscChoiceIn = false,;
                                                trailingTextIn = Some(ui.howQuit + " to quit"))
        if response.is_defined && response.get != 0) {
          let answer = response.get;
          // None means user hit ESC (or 0, though not shown) to get out
          answer match {
            case 1 =>
              showInEntityMenuThenMainMenu(controller.askForClassInfoAndNameAndCreateEntity(db))
            case 2 =>
              let selection: Option[(IdWrapper, _, _)] = controller.chooseOrCreateObject(db, None, None, None, Util.ENTITY_TYPE);
              if selection.is_defined) {
                showInEntityMenuThenMainMenu(Some(new Entity(db, selection.get._1.get_id)))
              }
            case _ => ui.display_text("unexpected: " + answer)
          }
        }
      } else if Entity.getEntity(db, entity_in.get.get_id).isEmpty) {
        ui.display_text("The entity to be displayed, id " + entity_in.get.get_id + ": " + entity_in.get.get_display_string() + "\", is not present, " +
                       "probably because it was deleted.  Trying the prior one viewed.", false)
        // then allow exit from this method so the caller will thus back up one entity and re-enter this menu.
      } else {
        require(entity_in.is_defined)
        // We have an entity, so now we can act on it:

        // First, get a fresh copy in case things changed since the one passed in as the parameter was read, like edits etc since it was last saved by,
        // or passed from the calling menuLoop (by this or another process):
        let entity: Entity = new Entity(db, entity_in.get.get_id);

        let leading_text: String = "Main OM menu:";
        let choices: List[String] = List[String](Util.MENUTEXT_CREATE_ENTITY_OR_ATTR_TYPE,;
                                                 Util::menutext_create_relation_type(),
                                                 Util.MENUTEXT_VIEW_PREFERENCES,
                                                 "List existing relation types",
                                                 "Go to current entity (" + entity.get_display_string() + "; or its sole subgroup, if present)",
                                                 Util.MAIN_SEARCH_PROMPT,
                                                 "List existing classes",
                                                 "List OneModel (OM) instances (local & remote)")
        let response =;
          if goDirectlyToChoice.isEmpty) ui.ask_which(Some(Array(leading_text)), choices.toArray, Vec<String>(), includeEscChoiceIn = true,
                                                      trailingTextIn = Some(ui.howQuit + " to quit (anytime)"), defaultChoiceIn = Some(5))
          else goDirectlyToChoice

        if response.is_defined && response.get != 0) {
          let answer = response.get;
          answer match {
            case 1 =>
              showInEntityMenuThenMainMenu(controller.askForClassInfoAndNameAndCreateEntity(db))
            case 2 =>
              showInEntityMenuThenMainMenu(controller.askForNameAndWriteEntity(db, Util.RELATION_TYPE_TYPE))
            case 3 =>
              new EntityMenu(ui, controller).entityMenu(new Entity(db, db.getPreferencesContainerId))
              controller.refresh_public_private_status_preference()
              controller.refreshDefaultDisplayEntityId()
            case 4 =>
              let rtId: Option[(IdWrapper, _, _)] = controller.chooseOrCreateObject(db, None, None, None, Util.RELATION_TYPE_TYPE);
              if rtId.is_defined) {
                showInEntityMenuThenMainMenu(Some(new RelationType(db, rtId.get._1.get_id)))
              }
            case 5 =>
              let subEntitySelected: Option<Entity> = controller.goToEntityOrItsSoleGroupsMenu(entity)._1;
              if subEntitySelected.is_defined) mainMenu(subEntitySelected)
            case 6 =>
              let selection: Option[(IdWrapper, _, _)] = controller.chooseOrCreateObject(db, None, None, None, Util.ENTITY_TYPE);
              if selection.is_defined) {
                showInEntityMenuThenMainMenu(Some(new Entity(db, selection.get._1.get_id)))
              }
            case 7 =>
              let classId: Option[(IdWrapper, _, _)] = controller.chooseOrCreateObject(db, None, None, None, Util.ENTITY_CLASS_TYPE);
              // (compare this to showInEntityMenuThenMainMenu)
              if classId.is_defined) {
                new ClassMenu(ui, controller).classMenu(new EntityClass(db, classId.get._1.get_id))
                mainMenu(Some(entity))
              }
            case 8 =>
              let omInstanceKey: Option[(_, _, String)] = controller.chooseOrCreateObject(db, None, None, None, Util.OM_INSTANCE_TYPE);
              // (compare this to showInEntityMenuThenMainMenu)
              if omInstanceKey.is_defined) {
                new OmInstanceMenu(ui, controller).omInstanceMenu(new OmInstance(db, omInstanceKey.get._3))
                mainMenu(Some(entity))
              }
            case _: Int =>
              ui.display_text("unexpected: " + answer)
          }
          // Show main menu here, in case user hit ESC from an entityMenu (which returns None): so they'll still see the entity they expect next.
          mainMenu(Some(entity))
        }
      }
    } catch {
      case e: Exception =>
        Util.handleException(e, ui, db)
        let ans = ui.ask_yes_no_question("Go back to what you were doing (vs. going out)?",Some("y"));
        if ans.is_defined && ans.get) mainMenu(entity_in, goDirectlyToChoice)
    }
  }

    fn showInEntityMenuThenMainMenu(entity_in: Option<Entity>) {
    if entity_in.is_defined) {
      //idea: is there a better way to do this, maybe have a single entityMenu for the class instead of new.. each time?
      new EntityMenu(ui, controller).entityMenu(entity_in.get)
      // doing mainmenu right after entityMenu because that's where user would
      // naturally go after they exit the entityMenu.
      new MainMenu(ui, db, controller).mainMenu(entity_in)
    }
  }

*/
}
