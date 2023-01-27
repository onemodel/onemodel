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
    fn mainMenu(entityIn: Option[Entity] = None, goDirectlyToChoice: Option[Int] = None) {
    //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) w/in this method! (if it fits the situation)
    try {
      let numEntities = db.getEntitiesOnlyCount();
      if (numEntities == 0 || entityIn.isEmpty) {
        let choices: List[String] = List[String]("Add new entity (such as yourself using your name, to start)",;
                                                 Util.mainSearchPrompt)
        let response: Option[Int] = ui.askWhich(None, choices.toArray, Array[String](), includeEscChoiceIn = false,;
                                                trailingTextIn = Some(ui.howQuit + " to quit"))
        if (response.isDefined && response.get != 0) {
          let answer = response.get;
          // None means user hit ESC (or 0, though not shown) to get out
          answer match {
            case 1 =>
              showInEntityMenuThenMainMenu(controller.askForClassInfoAndNameAndCreateEntity(db))
            case 2 =>
              let selection: Option[(IdWrapper, _, _)] = controller.chooseOrCreateObject(db, None, None, None, Util.ENTITY_TYPE);
              if (selection.isDefined) {
                showInEntityMenuThenMainMenu(Some(new Entity(db, selection.get._1.getId)))
              }
            case _ => ui.display_text("unexpected: " + answer)
          }
        }
      } else if (Entity.getEntity(db, entityIn.get.getId).isEmpty) {
        ui.display_text("The entity to be displayed, id " + entityIn.get.getId + ": " + entityIn.get.getDisplayString() + "\", is not present, " +
                       "probably because it was deleted.  Trying the prior one viewed.", false)
        // then allow exit from this method so the caller will thus back up one entity and re-enter this menu.
      } else {
        require(entityIn.isDefined)
        // We have an entity, so now we can act on it:

        // First, get a fresh copy in case things changed since the one passed in as the parameter was read, like edits etc since it was last saved by,
        // or passed from the calling menuLoop (by this or another process):
        let entity: Entity = new Entity(db, entityIn.get.getId);

        let leadingText: String = "Main OM menu:";
        let choices: List[String] = List[String](Util.menuText_createEntityOrAttrType,;
                                                 Util.menuText_createRelationType,
                                                 Util.menuText_viewPreferences,
                                                 "List existing relation types",
                                                 "Go to current entity (" + entity.getDisplayString() + "; or its sole subgroup, if present)",
                                                 Util.mainSearchPrompt,
                                                 "List existing classes",
                                                 "List OneModel (OM) instances (local & remote)")
        let response =;
          if (goDirectlyToChoice.isEmpty) ui.askWhich(Some(Array(leadingText)), choices.toArray, Array[String](), includeEscChoiceIn = true,
                                                      trailingTextIn = Some(ui.howQuit + " to quit (anytime)"), defaultChoiceIn = Some(5))
          else goDirectlyToChoice

        if (response.isDefined && response.get != 0) {
          let answer = response.get;
          answer match {
            case 1 =>
              showInEntityMenuThenMainMenu(controller.askForClassInfoAndNameAndCreateEntity(db))
            case 2 =>
              showInEntityMenuThenMainMenu(controller.askForNameAndWriteEntity(db, Util.RELATION_TYPE_TYPE))
            case 3 =>
              new EntityMenu(ui, controller).entityMenu(new Entity(db, db.getPreferencesContainerId))
              controller.refreshPublicPrivateStatusPreference()
              controller.refreshDefaultDisplayEntityId()
            case 4 =>
              let rtId: Option[(IdWrapper, _, _)] = controller.chooseOrCreateObject(db, None, None, None, Util.RELATION_TYPE_TYPE);
              if (rtId.isDefined) {
                showInEntityMenuThenMainMenu(Some(new RelationType(db, rtId.get._1.getId)))
              }
            case 5 =>
              let subEntitySelected: Option[Entity] = controller.goToEntityOrItsSoleGroupsMenu(entity)._1;
              if (subEntitySelected.isDefined) mainMenu(subEntitySelected)
            case 6 =>
              let selection: Option[(IdWrapper, _, _)] = controller.chooseOrCreateObject(db, None, None, None, Util.ENTITY_TYPE);
              if (selection.isDefined) {
                showInEntityMenuThenMainMenu(Some(new Entity(db, selection.get._1.getId)))
              }
            case 7 =>
              let classId: Option[(IdWrapper, _, _)] = controller.chooseOrCreateObject(db, None, None, None, Util.ENTITY_CLASS_TYPE);
              // (compare this to showInEntityMenuThenMainMenu)
              if (classId.isDefined) {
                new ClassMenu(ui, controller).classMenu(new EntityClass(db, classId.get._1.getId))
                mainMenu(Some(entity))
              }
            case 8 =>
              let omInstanceKey: Option[(_, _, String)] = controller.chooseOrCreateObject(db, None, None, None, Util.OM_INSTANCE_TYPE);
              // (compare this to showInEntityMenuThenMainMenu)
              if (omInstanceKey.isDefined) {
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
        let ans = ui.askYesNoQuestion("Go back to what you were doing (vs. going out)?",Some("y"));
        if (ans.isDefined && ans.get) mainMenu(entityIn, goDirectlyToChoice)
    }
  }

    fn showInEntityMenuThenMainMenu(entityIn: Option[Entity]) {
    if (entityIn.isDefined) {
      //idea: is there a better way to do this, maybe have a single entityMenu for the class instead of new.. each time?
      new EntityMenu(ui, controller).entityMenu(entityIn.get)
      // doing mainmenu right after entityMenu because that's where user would
      // naturally go after they exit the entityMenu.
      new MainMenu(ui, db, controller).mainMenu(entityIn)
    }
  }

*/
}
