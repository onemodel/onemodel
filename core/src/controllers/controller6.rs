/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003-2004, 2008-2020 inclusive, and 2022-2025 inclusive, Luke A. Call.
    (That copyright statement once said only 2013-2015, until I remembered that much of Controller came from TextUI.scala, and TextUI.java before that.)
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/

// Controller code is split between controller.rs and controller2.rs - controller6.rs, to make
// incremental compilation faster when only one has changed (or editing faster w/ rust-analyzer).

use crate::controllers::main_menu::MainMenu;
use crate::model::database::Database;
use crate::model::entity::Entity;
use crate::model::has_id::HasId;
use crate::model::postgres::postgresql_database::PostgreSQLDatabase;
use crate::model::relation_to_entity::RelationToEntity;
use crate::util::Util;
use crate::TextUI;
use std::cell::{RefCell, RefMut};
//use std::os::openbsd;
use std::rc::Rc;
use std::any::{Any}; //%%, TypeId};

use crate::controllers::entity_menu::EntityMenu;
use crate::controllers::group_menu::GroupMenu;
use crate::controllers::quick_group_menu::QuickGroupMenu;
use crate::model::attribute::Attribute;
use crate::controllers::controller::Controller;
use crate::model::attribute_data_holder::*;
use crate::model::boolean_attribute::BooleanAttribute;
use crate::model::date_attribute::DateAttribute;
use crate::model::entity_class::EntityClass;
use crate::model::file_attribute::FileAttribute;
use crate::model::group::Group;
use crate::model::id_wrapper::IdWrapper;
use crate::model::om_instance::OmInstance;
use crate::model::quantity_attribute::QuantityAttribute;
use crate::model::relation_to_group::RelationToGroup;
use crate::model::relation_to_local_entity::RelationToLocalEntity;
use crate::model::relation_to_remote_entity::RelationToRemoteEntity;
use crate::model::relation_type::RelationType;
use crate::model::text_attribute::TextAttribute;
use anyhow::anyhow;
//use std::collections::HashMap;
//use std::fs::File;
use std::path::Path;

impl Controller {
    pub fn default_attribute_copying(
        &self,
        target_entity_in: &mut Entity,
        attribute_tuples_in: Option</*%%?&*/ Vec<(i64, Rc<RefCell<dyn Attribute>>)>>,
    ) -> Result<(), anyhow::Error> {
        if self.should_try_adding_default_attributes(target_entity_in)? {
            let attribute_tuples: Vec<(i64, Rc<RefCell<dyn Attribute>>)> = if let Some(tuples) = attribute_tuples_in
            {
                tuples.to_vec()
            } else {
                target_entity_in.get_sorted_attributes(None, 0, 0, false)?.0
            };
            let id: Option<i64> = target_entity_in
                .get_class_template_entity_id(None)?;
            let template_entity: Entity = match id {
                // id.map(|id| Entity::new2(target_entity_in.get_db(), None, id)?);
                Some(id) => Entity::new2(target_entity_in.get_db(), None, id)?,
                None => return Err(anyhow!("Unexpected None resulting from entity.get_class_template_entity_id() \
                            where entity id is {}", target_entity_in.get_id())),
            };
            //%%next line in scala was:
            //let templateAttributesToCopy: ArrayBuffer[Attribute] = getMissingAttributes(template_entity, attributeTuples);
            let template_attributes_to_copy: Vec<Rc<RefCell<dyn Attribute>>> = self
                .get_missing_attributes(Some(&template_entity) /*%%?:.as_ref()*/, &attribute_tuples)?;
            self.copy_and_edit_attributes(target_entity_in, template_attributes_to_copy);
        }
        Ok(())
    }

    fn copy_and_edit_attributes(
        &self,
        entity_in: &Entity,
        template_attributes_to_copy_in: Vec<Rc<RefCell<dyn Attribute>>>,
    ) -> Result<(), anyhow::Error> {
        let mut esc_counter = 0;
        let mut user_wants_out = false;

        let check_if_exiting = |esc_counter_in: i32,
                                attribute_counter_in: usize,
                                num_attributes: usize|
         -> (i32, bool) {
            let mut esc_counter_local = esc_counter_in + 1;
            let mut wants_out = false;
            // <, so we don't ask when done anyway:
            if esc_counter_local > 3 && attribute_counter_in < num_attributes {
                let out_answer = self.ui.ask_yes_no_question(
                    "Stop checking/adding attributes?",
                    "",
                    false,
                );
                assert!(
                    out_answer.is_some(),
                    "Unexpected behavior: meant to make user answer here."
                );
                if out_answer.unwrap() {
                    wants_out = true;
                } else {
                    esc_counter_local = 0;
                }
            }
            (esc_counter_local, wants_out)
        };

        let mut ask_about_rte_every_time: Option<bool> = None;
        let mut all_copy = false;
        let mut all_create_or_search = false;
        let mut all_keep_reference = false;
        let mut attr_counter = 0;
        for mut attribute_from_template in template_attributes_to_copy_in.clone() {
            attr_counter += 1;
            if user_wants_out {
                break;
            }
            // let wait_for_keystroke: bool = match attribute_from_template.get_form_id() {
            let form_id = attribute_from_template.borrow().get_form_id()?;
            let wait_for_keystroke: bool = match attribute_from_template.borrow().get_db().borrow().get_attribute_form_name(form_id)? {
                //%% Util::RELATION_TO_LOCAL_ENTITY_TYPE_FORM_ID => true,
                Util::RELATION_TO_LOCAL_ENTITY_TYPE => true,
                Util::RELATION_TO_REMOTE_ENTITY_TYPE => true,
                _ => false,
            };

            let db = self.db.borrow();
            let form_name = db.get_attribute_form_name(attribute_from_template.borrow().get_form_id()?)?;
            let descr = attribute_from_template.borrow_mut().get_display_string(0, None, None, true)?;
            let prompt_to_edit_attribute_copy = || {
                self.ui.display_text2(
                    &format!(
                        "Edit the copied {} \"{}\" from the template entity (ESC to abort):",
                        form_name,
                        descr,
                    ),
                    wait_for_keystroke,
                );
            };

            let form_id = attribute_from_template.borrow().get_form_id()?;
            let new_attribute: Option<Box<dyn Attribute>> = match form_id
            {
                form_id if self.db.borrow().get_attribute_form_name(form_id)? == Util::QUANTITY_TYPE => {
                    prompt_to_edit_attribute_copy();
                    let template_attr = (&mut attribute_from_template as &mut dyn Any)
                        .downcast_mut::<QuantityAttribute>()
                        .unwrap();
                    let x = entity_in.add_quantity_attribute(
                        None,
                        template_attr.get_attr_type_id(None)?,
                        template_attr.get_unit_id(None)?,
                        template_attr.get_number(None)?,
                        Some(template_attr.get_sorting_index(None)?),
                    )?;
                    //%%?was: ).ok().map(|attr| Box::new(attr) as Box<dyn Attribute>)
                    Some(Box::new(x))
                }
                form_id if self.db.borrow().get_attribute_form_name(form_id)? == Util::DATE_TYPE => {
                    prompt_to_edit_attribute_copy();
                    //%%? and the one like it just above:
                    let template_attr = (&mut attribute_from_template as &mut dyn Any)
                        .downcast_mut::<DateAttribute>()
                        .unwrap();
                    let x = entity_in.add_date_attribute(
                        None,
                        template_attr.get_attr_type_id(None)?,
                        template_attr.get_date(None)?,
                        Some(template_attr.get_sorting_index(None)?),
                    )?;
                    //%%%??: ).ok().map(|attr| Box::new(attr) as Box<dyn Attribute>)
                    Some(Box::new(x))
                }
                form_id if self.db.borrow().get_attribute_form_name(form_id)? == Util::BOOLEAN_TYPE => {
                    prompt_to_edit_attribute_copy();
                    //%%%?:
                    let template_attr = (&mut attribute_from_template as &mut dyn Any)
                        .downcast_mut::<BooleanAttribute>()
                        .unwrap();
                    let x = entity_in.add_boolean_attribute(
                        None,
                        template_attr.get_attr_type_id(None)?,
                        template_attr.get_boolean(None)?,
                        Some(template_attr.get_sorting_index(None)?),
                    )?;
                    //%%?: ).ok().map(|attr| Box::new(attr) as Box<dyn Attribute>)
                    Some(Box::new(x))
                }
                form_id if self.db.borrow().get_attribute_form_name(form_id)? == Util::FILE_TYPE => {
                    self.ui.display_text1(
                        "You can add a FileAttribute manually afterwards for this attribute. Maybe it can be automated \
                        more, when use cases for this part are more clear."
                    );
                    None
                    //%%%did the formatter put a comma on next line, or remove it from others?:
                }
                form_id if self.db.borrow().get_attribute_form_name(form_id)? == Util::TEXT_TYPE => {
                    prompt_to_edit_attribute_copy();
                    //%%%?:
                    let template_attr = (&mut attribute_from_template as &mut dyn Any)
                        .downcast_mut::<TextAttribute>()
                        .unwrap();
                    let x = entity_in.add_text_attribute(
                        None,
                        template_attr.get_attr_type_id(None)?,
                        &template_attr.get_text(None)?,
                        Some(template_attr.get_sorting_index(None)?),
                    )?;
                    //%%%?: ).ok().map(|attr| Box::new(attr) as Box<dyn Attribute>)
                    Some(Box::new(x))
                }
                form_id
                    if self.db.borrow().get_attribute_form_name(form_id)? == Util::RELATION_TO_LOCAL_ENTITY_TYPE
                        || self.db.borrow().get_attribute_form_name(form_id)? == Util::RELATION_TO_REMOTE_ENTITY_TYPE =>
                {
                    let (new_rte, ask_every_time) = self.copy_and_edit_relation_to_entity(
                        entity_in,
                        attribute_from_template.clone(), /*%%?.as_ref()*/
                        ask_about_rte_every_time,
                        // %%%%need mut on lines below? and are the parms right?
                        &mut all_copy,
                        &mut all_create_or_search,
                        &mut all_keep_reference,
                    )?;
                    ask_about_rte_every_time = ask_every_time;
                    new_rte
                }
                form_id if self.db.borrow().get_attribute_form_name(form_id)? == Util::RELATION_TO_GROUP_TYPE => {
                    prompt_to_edit_attribute_copy();
                    //%%?:
                    let template_attr_opt: Option<&mut RelationToGroup> = (&mut attribute_from_template as &mut dyn Any)
                        .downcast_mut::<RelationToGroup>();
                    let Some(template_attr) = template_attr_opt else {
                        return Err(anyhow!("Unexpected None found in template_attr_opt."));
                    };
                    let mut template_group = template_attr.get_group(None)?;
                    let (_, new_rtg_id): (_, i64) = entity_in
                        .add_group_and_relation_to_group(
                            None,
                            template_attr.get_attr_type_id(None)?,
                            &template_group.get_name(None)?,
                            template_group.get_mixed_classes_allowed(None)?,
                            None,
                            //%%%%?:
                            chrono::Local::now().timestamp_millis(),
                            Some(template_attr.get_sorting_index(None)?),
                        )?; //%%?: .unwrap();
                    let new_rtg: RelationToGroup = RelationToGroup::new3(self.db.clone(), None, new_rtg_id)?;
                    Some(Box::new(new_rtg) as Box<dyn Attribute>)
                }
                //%%next line in scala was. See similar places elsewhere, or just type something in?:
                //case _ => throw new OmException("Unexpected type: " + attributeFromTemplate.getClass.getCanonicalName)
                _ => {
                    return Err(anyhow!("Unexpected type: {:?}", attribute_from_template))
                }
            };

            if new_attribute.is_none() {
                let (new_counter, wants_out) = check_if_exiting(
                    esc_counter,
                    attr_counter,
                    template_attributes_to_copy_in.len(),
                );
                esc_counter = new_counter;
                user_wants_out = wants_out;
            } else if self.db.borrow().get_attribute_form_name(new_attribute.as_ref().unwrap().get_form_id()?)?
                == Util::RELATION_TO_LOCAL_ENTITY_TYPE
                || self.db.borrow().get_attribute_form_name(new_attribute.as_ref().unwrap().get_form_id()?)?
                    == Util::RELATION_TO_REMOTE_ENTITY_TYPE
            {
                // (Not re-editing if it is a RTE because it was edited just above as part of the initial
                // attribute creation step.)
                let exited_one_edit_line: bool = self.edit_attribute_on_single_line(
                    new_attribute.as_ref().unwrap().as_ref()
                )?;
                if exited_one_edit_line {
                    new_attribute.as_ref().unwrap().delete(None)?;
                    let (new_counter, wants_out) = check_if_exiting(
                        esc_counter,
                        attr_counter,
                        template_attributes_to_copy_in.len(),
                    );
                    esc_counter = new_counter;
                    user_wants_out = wants_out;
                }
            }
        }
        Ok(())
    }

    fn copy_and_edit_relation_to_entity(
        &self,
        entity_in: &Entity,
        relation_to_entity_attribute_from_template_in: Rc<RefCell<dyn Attribute>>,
        ask_every_time_in: Option<bool>, /*=None*/
        all_copy: &mut bool,
        all_create_or_search: &mut bool,
        all_keep_reference: &mut bool,
    ) -> Result<(Option<Box<dyn Attribute>>, Option<bool>), anyhow::Error> {
        let rte_attr_from_template_form_id: i32 = relation_to_entity_attribute_from_template_in.borrow().get_form_id()?;
        let db_borrow = self.db.borrow();
        let form_name = db_borrow.get_attribute_form_name(rte_attr_from_template_form_id)?;
        // assert!(matches!(
        //     rte_attr_from_template_form_id,
        //     form_id if form_name == Util::RELATION_TO_LOCAL_ENTITY_TYPE || form_name == Util::RELATION_TO_REMOTE_ENTITY_TYPE
        // ));
        assert!(form_name == Util::RELATION_TO_LOCAL_ENTITY_TYPE || form_name == Util::RELATION_TO_REMOTE_ENTITY_TYPE,
            "unexpected form_name: {}", form_name);
        let choice1_text = "Copy the template entity, editing its name (**MOST LIKELY CHOICE)".to_string();
        let copy_from_template_and_edit_name_choice_num = 1;
        let choice2_text = "Create a new entity or search for an existing one for this purpose".to_string();
        let create_or_search_for_entity_choice_num = 2;
        let choice3_text =
            "Keep a reference to the same entity as in the template (least likely choice)".to_string();
        let keep_same_reference_as_in_template_choice_num = 3;
        let mut ask_every_time: Option<bool> = ask_every_time_in;
        if ask_every_time.is_none() {
            let how_rtes_leading_text = vec![
                "The template has relations to entities. How would you like the equivalent to be provided \
                for this new entity being created?".to_string()
            ];
            let how_handle_rtes_choices = vec![
                format!("For ALL entity relations being added: {}", choice1_text),
                format!("For ALL entity relations being added: {}", choice2_text),
                format!("For ALL entity relations being added: {}", choice3_text),
                "Ask for each relation to entity being created from the template".to_string(),
            ];
            let how_handle_rtes_response = self.ui.ask_which(
                Some(how_rtes_leading_text),
                &how_handle_rtes_choices,
                &Vec::<String>::new(),
                true,
                None,
                None,
                None,
                None,
            );
            if let Some(response) = how_handle_rtes_response {
                match response {
                    1 => {
                        *all_copy = true;
                        ask_every_time = Some(false);
                    }
                    2 => {
                        *all_create_or_search = true;
                        ask_every_time = Some(false);
                    }
                    3 => {
                        *all_keep_reference = true;
                        ask_every_time = Some(false);
                    }
                    4 => {
                        ask_every_time = Some(true);
                    }
                    _ => {
                        self.ui
                            .display_text1(&format!("Unexpected answer: {}", response).as_str());
                        ask_every_time = None;
                    }
                }
            }
        }
        if ask_every_time.is_none() {
            return Ok((None, ask_every_time));
        }
        let how_copy_rte_response: Option<i32> = if ask_every_time.unwrap() {
            let which_rte_leading_text = vec![format!(
                "The template has a templateAttribute which is a relation to an entity named \"{}\": \
                how would you like the equivalent to be provided for this new entity being created? \
                (0/ESC to just skip this one for now)",
                relation_to_entity_attribute_from_template_in.borrow_mut().get_display_string(0, None, None, /*simplify=*/ true)?
            )];
            let which_rte_choices = vec![choice1_text, choice2_text, choice3_text];
            let ans = self.ui.ask_which(
                Some(which_rte_leading_text),
                &which_rte_choices,
                &Vec::<String>::new(),
                true,
                None,
                None,
                None,
                None,
            );
            match ans {
                Some(ans) => {
                    let ans: i32 = ans.try_into()?;
                    Some(ans)
                }
                _ => None
            }
        } else {
            None
        };
        if ask_every_time.unwrap() && how_copy_rte_response.is_none() {
            return Ok((None, ask_every_time));
        }
        let related_id2 = if form_name //%%relation_to_entity_attribute_from_template_in.borrow().get_form_id()
            == Util::RELATION_TO_REMOTE_ENTITY_TYPE //%%_FORM_ID
        {
            //%%:
            return Err(anyhow!("not yet implemented"));
            // (relation_to_entity_attribute_from_template_in.as_ref() as &dyn Any)
            //     .downcast_ref::<RelationToRemoteEntity>()
            //     .unwrap()
            //     .get_related_id2()
        } else {
            // guaranteed by condition at top of method, to be Util::RELATION_TO_REMOTE_ENTITY_TYPE, now:
            let rteafti = relation_to_entity_attribute_from_template_in.clone();
            let rteaftia = &rteafti as &dyn Any;
            rteaftia.downcast_ref::<RelationToLocalEntity>()
                .unwrap()
                .get_related_id2()
        };
        if *all_copy
            || (how_copy_rte_response.is_some()
                && how_copy_rte_response.unwrap() == copy_from_template_and_edit_name_choice_num)
        {
            let current_or_remote_db_for_related_entity = Util::current_or_remote_db(
                relation_to_entity_attribute_from_template_in.clone(),
                relation_to_entity_attribute_from_template_in.borrow().get_db(),
            )?;
            let mut templates_related_entity =
                Entity::new2(current_or_remote_db_for_related_entity.clone(), None, related_id2)?;
            let old_name: String = templates_related_entity.get_name(None)?; //%%was from claude: .unwrap_or_default();
            let form_id = relation_to_entity_attribute_from_template_in.borrow_mut().get_form_id()?;
            let db_borrow2 = self.db.borrow();
            let form_name = db_borrow2.get_attribute_form_name(form_id)?;
            let new_entity: Option<Entity> = if form_name == Util::RELATION_TO_LOCAL_ENTITY_TYPE
            {
                self.ask_for_name_and_write_entity(
                    entity_in.get_db(),
                    Util::ENTITY_TYPE,
                    Rc::new(RefCell::new(None)),
                    Some(old_name.clone()),
                    None,
                    None,
                    templates_related_entity.get_class_id(None)?,
                    Some("EDIT THE ENTITY NAME:"),
                    /*%%%duplicate_name_probably_ok = */ true,
                )?
            } else {
                let Some(e) = self.ask_for_name_and_write_entity(
                    entity_in.get_db(),
                    Util::ENTITY_TYPE,
                    Rc::new(RefCell::new(None)),
                    Some(old_name),
                    None,
                    None,
                    None,
                    Some("EDIT THE ENTITY NAME:"),
                    /*%%%duplicate_name_probably_ok=*/ true,
                )? else {
                    return Err(anyhow!("unexpected result: e should not be None"));
                };
                let Some(remote_class_id) = templates_related_entity.get_class_id(None)? else {
                    return Err(anyhow!("unexpected result: class_id should not be None"));
                };
                let remote_class_name: String = EntityClass::new2(
                    current_or_remote_db_for_related_entity.clone(),
                    None,
                    remote_class_id,
                )?
                .get_name(None)?;
                self.ui.display_text1(&format!(
                    "Note: Did not write a class on the new entity to match that from the \
                    remote entity, until some kind of synchronization \
                    of classes across OM instances is in place. (Idea: interim solution \
                    could be to match simply by name if there is a match, with user confirmation, \
                    or user selection if multiple matches. The class in the remote instance is: {}: {}",
                    remote_class_id, remote_class_name
                ));
                Some(e)
            };
            if let Some(mut entity) = new_entity {
                entity.update_new_entries_stick_to_top(
                    None,
                    templates_related_entity.get_new_entries_stick_to_top(None)?,
                )?;
                let new_rtle = entity_in.add_relation_to_local_entity(
                    None,
                    relation_to_entity_attribute_from_template_in.borrow_mut().get_attr_type_id(None)?,
                    entity.get_id(),
                    Some(relation_to_entity_attribute_from_template_in.borrow_mut().get_sorting_index(None)?),
                    //None,
                    None,
                    //%% Utc::now().timestamp_millis(),
                    chrono::Local::now().timestamp_millis(),
                )?;
                Ok((
                    Some(new_rtle as Box<dyn Attribute>),
                    ask_every_time,
                ))
            } else {
                Ok((None, ask_every_time))
            }
        } else if *all_create_or_search
            || (how_copy_rte_response.is_some()
                && how_copy_rte_response.unwrap() == create_or_search_for_entity_choice_num)
        {
            let rte_dh = RelationToEntityDH {
                rel_type_id: relation_to_entity_attribute_from_template_in.borrow_mut().get_attr_type_id(None)?,
                valid_on_date: None,
                observation_date: chrono::Local::now().timestamp_millis(),
                entity_id2: 0,
                is_remote: false,
                remote_instance_id: String::new(),
            };
            let mut rte_dhv = AttributeDataHolder::RelationToEntityDH { rtedh: rte_dh };
            let adh: Option<AttributeDataHolder> = self.ask_for_relation_entity_id_number2(
                entity_in.get_db(), /*%%?:.as_ref()*/
                &mut rte_dhv,
                /*%%editing_in=*/ false,
                &self.ui,
            )?;
            let Some(adh) = adh else {
                return Ok((None, ask_every_time));
            };
            let AttributeDataHolder::RelationToEntityDH { rtedh: dh } = adh else {
                return Ok((None, ask_every_time));
            };
            if dh.is_remote {
                return Err(anyhow!("%%Not yet implemented"));
                // let rtre = entity_in.add_relation_to_remote_entity(
                //     dh.attr_type_id,
                //     dh.entity_id2,
                //     Some(relation_to_entity_attribute_from_template_in.borrow().get_sorting_index(None)),
                //     dh.valid_on_date,
                //     dh.observation_date,
                //     &dh.remote_instance_id,
                // )?; //%%?.unwrap();
                // Ok((Some(Box::new(rtre) as Box<dyn Attribute>), ask_every_time))
            } else {
                let rtle = entity_in.add_relation_to_local_entity(
                    None,
                    dh.rel_type_id,
                    dh.entity_id2,
                    Some(relation_to_entity_attribute_from_template_in.borrow_mut().get_sorting_index(None)?),
                    dh.valid_on_date,
                    dh.observation_date,
                )?; //%%?:.unwrap();
                Ok((Some(rtle as Box<dyn Attribute>), ask_every_time))
            }
        } else if *all_keep_reference
            || (how_copy_rte_response.is_some()
                && how_copy_rte_response.unwrap() == keep_same_reference_as_in_template_choice_num)
        {
            let relation = if relation_to_entity_attribute_from_template_in
                .borrow()
                .get_db()
                .borrow()
                .is_remote()
            {
                return Err(anyhow!("%%unimplemented"));
                // let rtre_template = (relation_to_entity_attribute_from_template_in as &dyn Any)
                //     .downcast_ref::<RelationToRemoteEntity>()
                //     .unwrap();
                // entity_in.add_relation_to_remote_entity(
                //     relation_to_entity_attribute_from_template_in.borrow().get_attr_type_id(None),
                //     related_id2,
                //     Some(relation_to_entity_attribute_from_template_in.borrow().get_sorting_index(None)),
                //     None,
                //     //%%%%:
                //     chrono::Local::now().timestamp_millis(),
                //     &rtre_template.get_remote_instance_id(),
                // )? //%%?: .unwrap()
            } else {
                entity_in
                    .add_relation_to_local_entity(
                        None,
                        relation_to_entity_attribute_from_template_in.borrow_mut().get_attr_type_id(None)?,
                        related_id2,
                        Some(relation_to_entity_attribute_from_template_in.borrow_mut().get_sorting_index(None)?),
                        None,
                        //%%%%:
                        chrono::Local::now().timestamp_millis(),
                    )
                    .unwrap()
            };
            Ok((
                Some(relation as Box<dyn Attribute>),
                ask_every_time,
            ))
        } else {
            self.ui.display_text1(&format!(
                "Unexpected answer: {}/{}/{}/{:?}/{:?}",
                *all_copy,
                *all_create_or_search,
                *all_keep_reference,
                ask_every_time,
                how_copy_rte_response
            ));
            Ok((None, ask_every_time))
        }
    }

    /// This determines which attributes from the template entity (or "pattern" or "class-defining entity")
    /// are not found on this entity, so they can be added if the user wishes.
    fn get_missing_attributes(
        &self,
        class_template_entity_in: Option<&Entity>,
        // existing_attribute_tuples_in: Vec<(i64, Box<dyn Attribute>)>,
        existing_attribute_tuples_in: &Vec<(i64, Rc<RefCell<dyn Attribute>>)>,
    ) -> Result<Vec<Rc<RefCell<dyn Attribute>>>, anyhow::Error> {
        let mut attributes_to_suggest_copying = Vec::<Rc<RefCell<dyn Attribute>>>::new();
        if let Some(template_entity) = class_template_entity_in {
            let (cde_attribute_tuples, _) =
                template_entity.get_sorted_attributes(None, 0, 0, false)?;
            for (_, cde_attribute) in cde_attribute_tuples {
                let mut attribute_type_found_on_entity = false;
                //%%% next line in scala was:
                //let cde_attribute = cde_attributeTuple._2;
                let cde_type_id = cde_attribute.borrow_mut().get_attr_type_id(None)?;
                //%%%in scala: for (attributeTuple <- existingAttributeTuplesIn) {
                for (_, attribute) in existing_attribute_tuples_in {
                    if !attribute_type_found_on_entity {
                        let type_id = attribute.borrow_mut().get_attr_type_id(None)?;
                        // This is a very imperfect check.  Perhaps this is a motive to use more descriptive
                        // relation types in template entities.
                        let existing_attribute_string_contains_template_string: bool = attribute
                            .borrow_mut()
                            .get_display_string(0, None, None, /*%%%simplify=*/ true)?
                            .contains(
                                &cde_attribute
                                    .borrow_mut()
                                    .get_display_string(0, None, None, /*%%%simplify=*/ true)?,
                            );
                        if cde_type_id == type_id
                            && existing_attribute_string_contains_template_string
                        {
                            attribute_type_found_on_entity = true;
                        }
                    }
                }
                if !attribute_type_found_on_entity {
                    attributes_to_suggest_copying.push(cde_attribute /*%%%?: .clone()*/);
                }
            }
        }
        Ok(attributes_to_suggest_copying)
    }

    fn should_try_adding_default_attributes(
        &self,
        entity_in: &mut Entity,
    ) -> Result<bool, anyhow::Error> {
        let Some(class_id) = entity_in.get_class_id(None)? else {
            return Ok(false);
        };
        let mut entity_class =
            EntityClass::new2(entity_in.get_db(), None, class_id)?; //%%?:.unwrap());
        let create_attributes: Option<bool> = entity_class.get_create_default_attributes(None)?;
        if let Some(create) = create_attributes {
            Ok(create)
        } else if let Some(template_id) = entity_in.get_class_template_entity_id(None)? {
            let template_entity: Entity = Entity::new2(entity_in.get_db(), None, template_id)?;
            let attr_count =
                template_entity.get_attribute_count(None, self.db.borrow().include_archived_entities())?;
            if attr_count == 0 {
                Ok(false)
            } else {
                let add_attributes_answer = self.ui.ask_yes_no_question(
                    "Add attributes to this entity as found on the class-defining entity (template)?",
                    "y",
                    true
                );
                Ok(add_attributes_answer.is_some() && add_attributes_answer.unwrap())
            }
        } else {
            Ok(false)
        }
    }
}
