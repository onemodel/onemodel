/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003-2004 and 2008-2017 inclusive, and 2023-2025 inclusive, Luke A. Call.
    (That copyright statement was previously 2013-2015, until I remembered that much of Controller
    came from TextUI.scala and TextUI.java before that.)
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/

use crate::model::database::Database;
//use crate::model::relation_type::RelationType;
use crate::controllers::controller::Controller;
use crate::model::entity::Entity;
use crate::model::entity_class::EntityClass;
use crate::model::om_instance::OmInstance;
use crate::util::Util;
use crate::TextUI;
use std::cell::RefCell;
use std::rc::Rc;
use tracing::*;

pub struct MainMenu {
    ui: Rc<TextUI>,
    db: Rc<RefCell<dyn Database>>,
    controller: Rc<Controller>,
}

impl MainMenu {
    pub fn new(
        ui: Rc<TextUI>,
        db: Rc<RefCell<dyn Database>>,
        controller: Rc<Controller>,
    ) -> MainMenu {
        MainMenu { ui, db, controller }
    }

    pub fn main_menu(
        &self,
        entity_in: Option<Entity>,            /*= None*/
        go_directly_to_choice: Option<usize>, /*= None*/
    ) {
        loop {
            let result = self.main_menu_helper(&entity_in, go_directly_to_choice);
            // We check for it in a loop just in case user wants to keep editing the same entity,
            // but the default entity (which the caller in controller would display) might have
            // changed.
            match result {
                Ok(()) => break,
                Err(e) => {
                    Util::handle_error(
                        e,
                        self.ui.clone(),
                        format!("{}:{}:{}", file!(), line!(), column!()).as_str(),
                    );
                    let ans = self.ui.ask_yes_no_question(
                        "Go back to what you were doing (vs. going out)?",
                        "y",
                        true,
                    );
                    if ans.is_some() && ans.unwrap() {
                        //mainMenu(entity_in, go_directly_to_choice)
                        continue;
                    } else {
                        break;
                    }
                }
            };
        }
    }

    //idea: fix bad smells: long method, with logical but bad-habit-forming unwrap()s?
    pub fn main_menu_helper(
        &self,
        entity_in: &Option<Entity>,           /*= None*/
        go_directly_to_choice: Option<usize>, /*= None*/
    ) -> Result<(), anyhow::Error> {
        let dbb = self.db.borrow();
        let num_entities = dbb.get_entities_only_count(None, false, None, None)?;
        if num_entities == 0 || entity_in.is_none() {
            let choices = vec![
                "Add new entity (such as yourself using your name, to start)".to_string(),
                Util::MAIN_SEARCH_PROMPT.to_string(),
            ];
            let response = self.ui.ask_which(
                None,
                &choices,
                &Vec::new(),
                false,
                Some(format!("{}{}", self.ui.how_quit(), " to quit").as_str()),
                None,
                None,
                None,
            );
            if response.is_some() && response.unwrap() != 0 {
                let answer = response.unwrap();
                // None means user hit ESC (or 0, though not shown) to get out
                match answer {
                    1 => {
                        println!(
                            "%%self.show_in_entity_menu_then_main_menu(
                            self.controller.ask_for_class_info_and_name_and_create_entity(
                                self.db.clone(),
                                None,
                            ),
                        );"
                        );
                        Ok(())
                    }
                    2 => {
                        println!("%%chooseorcreateobject");
                        //let selection = self.controller.choose_or_create_object();
                        //    self.db.clone(),
                        //    None,
                        //    None,
                        //    None,
                        //    Util::ENTITY_TYPE,
                        //    0,
                        //    None,
                        //    false,
                        //    None,
                        //    false,
                        //    None,
                        //    false,
                        //);
                        ////%%rid of 2nd unwrap here? (repl w/ ?;)
                        //if selection.is_some() {
                        //    let entity = Entity::new2(
                        //        self.db.clone(),
                        //        None,
                        //        selection.unwrap().0.get_id(),
                        //    )
                        //    .unwrap();
                        //    self.show_in_entity_menu_then_main_menu(Some(entity));
                        //}
                        Ok(())
                    }
                    _ => {
                        self.ui
                            .display_text1(format!("unexpected: {}", answer).as_str());
                        Ok(())
                    }
                }
            } else {
                Ok(())
            }
        } else if Entity::get_entity(self.db.clone(), None, entity_in.clone().unwrap().get_id())?
            //.unwrap()
            .is_none()
        {
            // This unwrap() and the one in the "else" condition just above are guaranteed safe by the original
            // above "if num_entities == 0 || entity_in.is_none()" condition.
            let mut entity = entity_in.clone().unwrap();
            self.ui.display_text2(
                &format!(
                    "The entity to be displayed, id {}: \"{}\", is not present, \
                     probably because it was deleted.  Trying the prior one viewed.",
                    entity.get_id(),
                    entity.get_display_string(None, false).unwrap()
                ),
                false,
            );
            Ok(())
            // then allow exit from this method so the caller will thus back up one entity and re-enter this menu.
        } else {
            assert!(entity_in.is_some());
            // We have an entity, so now we can act on it:

            // First, get a fresh copy in case things changed since the one passed in as the parameter
            // was read, like edits etc since it was last saved by,
            // or passed from the calling menuLoop (by this or another process):
            let mut entity =
                Entity::new2(self.db.clone(), None, entity_in.clone().unwrap().get_id())?;
            let leading_text = "Main OM menu:";
            let menutext_create_relation_type = Util::menutext_create_relation_type();
            let entity_descr = entity.get_display_string(None, false)?;
            let go_to_current_entity = format!(
                "Go to current entity ({}; or its sole subgroup, if present)",
                entity_descr
            );
            let choices = vec![
                Util::MENUTEXT_CREATE_ENTITY_OR_ATTR_TYPE.to_string(),
                menutext_create_relation_type.as_str().to_string(),
                Util::MENUTEXT_VIEW_PREFERENCES.to_string(),
                "List existing relation types".to_string(),
                go_to_current_entity.as_str().to_string(),
                Util::MAIN_SEARCH_PROMPT.to_string(),
                "List existing classes".to_string(),
                "List OneModel (OM) instances (local & remote)".to_string(),
            ];
            let response = if go_directly_to_choice.is_none() {
                let ans = self.ui.ask_which(
                    Some(vec![leading_text.to_string()]),
                    &choices,
                    &Vec::new(),
                    true,
                    Some(format!("{} to quit (anytime)", self.ui.how_quit()).as_str()),
                    None,
                    None,
                    Some(5),
                );
                ans
            } else {
                go_directly_to_choice
            };
            if response.is_some() && response.unwrap() != 0 {
                let answer = response.unwrap();
                match answer {
                    1 => {
                        println!(
                            "%%self.show_in_entity_menu_then_main_menu(
                            self.controller.ask_for_class_info_and_name_and_create_entity(
                                self.db.clone(),
                                None,
                            ),
                        );"
                        );
                    }
                    2 => {
                        println!(
                            "%%self.show_in_entity_menu_then_main_menu(
                            self.controller.ask_for_name_and_write_entity(
                                self.db.clone(),
                                Util::RELATION_TYPE_TYPE,
                                None,
                                None,
                                None,
                                None,
                                None,
                                None,
                                false,
                            ),
                        );"
                        );
                    }
                    3 => {
                        println!(
                            "%%
                        let preferences_container_id = self.db.get_preferences_container_id(None)?;
                        let entity = Entity::new2(self.db.clone(), None, preferences_container_id)?;
                        EntityMenu::new(self.ui.clone(), self.controller.clone())
                            .entity_menu(entity, None, None, None);
                        self.controller.refresh_public_private_status_preference();
                        self.controller.refresh_default_display_entity_id();
                        "
                        );
                    }
                    4 => {
                        println!("%%chooseobj");
                        //let rt_id = self.controller.choose_or_create_object(
                        //    self.db.clone(),
                        //    None,
                        //    None,
                        //    None,
                        //    Util::RELATION_TYPE_TYPE,
                        //    0,
                        //    None,
                        //    false,
                        //    None,
                        //    false,
                        //    None,
                        //    false,
                        //);
                        //if rt_id.is_some() {
                        //    //can just pass in the the RT's has-a entity here?
                        //    //%%hopefully so. test it.
                        //    let entity_id = rt_id.unwrap().0.get_id();
                        //    //was: let rt = RelationType::new2(
                        //    //    self.db.clone(),
                        //    //    None,
                        //    //    entity_id,
                        //    //)?;
                        //    let entity: Entity = Entity::new2(self.db, None, entity_id)?;
                        //    //self.show_in_entity_menu_then_main_menu(Some(rt));
                        //    self.show_in_entity_menu_then_main_menu(Some(entity));
                        //}
                    }
                    5 => {
                        let (sub_entity_selected, _, _) =
                            self.controller.go_to_entity_or_its_sole_groups_menu(
                                &entity,
                                None,
                                None,
                            )?;
                        if sub_entity_selected.is_some() {
                            self.main_menu(sub_entity_selected, None);
                        }
                    }
                    6 => {
                        println!("%%%%coco");
                        //let selection = self.controller.choose_or_create_object(
                        //    self.db.clone(),
                        //    None,
                        //    None,
                        //    None,
                        //    Util::ENTITY_TYPE,
                        //    0,
                        //    None,
                        //    false,
                        //    None,
                        //    false,
                        //    None,
                        //    false,
                        //);
                        //if selection.is_some() {
                        //    let entity = Entity::new2(
                        //        self.db.clone(),
                        //        None,
                        //        selection.unwrap().0.get_id(),
                        //    )?;
                        //    self.show_in_entity_menu_then_main_menu(Some(entity));
                        //}
                    }
                    7 => {
                        println!("%%coco");
                        //let class_id = self.controller.choose_or_create_object(
                        //    self.db.clone(),
                        //    None,
                        //    None,
                        //    None,
                        //    Util::ENTITY_CLASS_TYPE,
                        //    0,
                        //    None,
                        //    false,
                        //    None,
                        //    false,
                        //    None,
                        //    false,
                        //);
                        //// (compare this to show_in_entity_menu_then_main_menu)
                        //if class_id.is_some() {
                        //    let entity_class = EntityClass::new2(
                        //        self.db.clone(),
                        //        None,
                        //        class_id.unwrap().0.get_id(),
                        //    )?;
                        //    ClassMenu::new(self.ui.clone(), self.controller.clone())
                        //        .class_menu(entity_class);
                        //    self.main_menu(Some(entity), None);
                        //}
                    }
                    8 => {
                        println!("%%coco");
                        //let om_instance_key = self.controller.choose_or_create_object(
                        //    self.db.clone(),
                        //    None,
                        //    None,
                        //    None,
                        //    Util::OM_INSTANCE_TYPE,
                        //    0,
                        //    None,
                        //    false,
                        //    None,
                        //    false,
                        //    None,
                        //    false,
                        //);
                        //// (compare this to show_in_entity_menu_then_main_menu)
                        //if om_instance_key.is_some() {
                        //    //%%is ui.clone needed here & in similar places?
                        //    let om_instance = OmInstance::new2(
                        //        self.db.clone(),
                        //        None,
                        //        om_instance_key.unwrap().2,
                        //    )?;
                        //    OmInstanceMenu::new(self.ui.clone(), self.controller.clone())
                        //        .om_instance_menu(om_instance);
                        //    self.main_menu(Some(entity), None);
                        //}
                    }
                    _ => {
                        self.ui.display_text1(&format!("unexpected: {}", answer));
                    }
                }
                // Show main menu here, in case user hit ESC from an entityMenu (which returns
                // None): so they'll still see the entity they expect next.
                self.main_menu(Some(entity), None);
                Ok(())
            } else {
                Ok(())
            }
        }
    }

    fn show_in_entity_menu_then_main_menu(&self, entity_in: Option<Entity>) {
        println!("%%siemtmm");
        //if entity_in.is_some() {
        //    //idea: is there a better way to do this, maybe have a single entityMenu for the
        //    //class instead of new.. each time?
        //
        //    EntityMenu::new(self.ui.clone(), self.controller.clone())
        //        .entity_menu(entity_in.unwrap().clone(), None, None, None);
        //    // doing mainmenu right after entityMenu because that's where user would
        //    // naturally go after they exit the entityMenu.
        //    MainMenu::new(self.ui.clone(), self.db.clone(), self.controller.clone())
        //        .main_menu(entity_in, None);
        //}
    }
}
