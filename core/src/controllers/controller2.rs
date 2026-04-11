/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003-2004, 2008-2020 inclusive, and 2022-2025 inclusive, Luke A. Call.
    (That copyright statement once said only 2013-2015, until I remembered that much of Controller came from TextUI.scala,
    and TextUI.java before that.)
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/

// Controller code is split between controller.rs and controller2.rs - controller6.rs, to make
// incremental compilation faster when only one has changed (or editing faster w/ rust-analyzer).

use crate::controllers::controller::Controller;
use crate::controllers::main_menu::MainMenu;
use crate::model::database::Database;
use crate::model::entity::Entity;
use crate::model::has_id::HasId;
use crate::model::postgres::postgresql_database::PostgreSQLDatabase;
use crate::util::Util;
use crate::TextUI;
use std::any::{Any, TypeId};
use std::cell::{RefCell, RefMut};
//use std::os::openbsd;
use std::rc::Rc;

use crate::controllers::entity_menu::EntityMenu;
use crate::controllers::group_menu::GroupMenu;
use crate::controllers::quick_group_menu::QuickGroupMenu;
use crate::model::attribute::Attribute;
use crate::model::attribute_data_holder::*;
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
    /*
    ///need2test this: is in tests, or manually?
    ///Not sure if this will even be called, but it is here to preserve the functionality that was
    ///in edit_entity_name, in the Scala version of the code, which was removed (separated here).
    fn edit_relation_type_name(&self, rel_type: &RelationType) -> Option<RelationType> {
        let previous_name_in_reverse = rel_type.get_name_in_reverse_direction(None);
        let edited_rt: Option<RelationType> = self.ask_for_name_and_write_rel_type(
            rel_type.get_db(),
            Util::RELATION_TYPE_TYPE,
            Some(rel_type), //%%?: .clone()),
            Some(rel_type.get_name(None)),
            Some(rel_type.get_directionality(None)),
            if previous_name_in_reverse.is_none()
                || previous_name_in_reverse.unwrap().trim().is_empty()
            {
                None
            } else {
                Some(previous_name_in_reverse)
            },
            None,
            None,
            false,
        );
        edited_rt
    }
    */

    fn ask_for_public_nonpublic_status(&self, default_for_prompt: Option<bool>) -> Option<bool> {
        let default = match default_for_prompt {
            None => "",
            Some(true) => "y",
            Some(false) => "n",
        };
        self.ui.ask_yes_no_question(
            format!(
                "For Public vs. Non-public, enter a yes/no value (or a space \
                for 'unknown/unspecified'; used e.g. during data export; display preference can be \
                set under main menu / {})",
                Util::MENUTEXT_VIEW_PREFERENCES
            )
            .as_str(),
            default,
            true,
        )
    }

    /// Returns data, or None if user wants to cancel/get out.
    /// The parameter attr_type is a constant referring to Attribute subtype, as used by the in_object_type
    /// parameter to the choose_or_create_object method
    ///                 (ex., Controller.QUANTITY_TYPE).  See comment on that method, for that parm.
    /// The editingIn parameter (I think) being true means we are editing data, not adding new data.
    pub fn ask_for_attribute_data/*%%<T>*/(
        &self,
        db_in: Rc<RefCell<dyn Database>>,
        inout_dh: &mut AttributeDataHolder,
        also_ask_for_attr_type_id: bool,
        attr_type: &str,
        attr_type_input_prompt: Option<&str>,
        in_previous_selection_desc: Option<String>,
        in_previous_selection_id: Option<i64>,
        ask_for_other_info: fn(
            &Controller,
            Rc<RefCell<dyn Database>>,
            &mut AttributeDataHolder,
            bool,
            &TextUI,
        ) -> Result<Option<AttributeDataHolder>, anyhow::Error>,
        editing_in: bool,
    ) -> Result<Option<AttributeDataHolder>, anyhow::Error>
//%%where
    //    T: AttributeDataHolder + Clone,
    {
        let (user_wants_out, attr_type_id, is_remote, remote_key): (bool, i64, _, _) =
            if also_ask_for_attr_type_id {
                assert!(attr_type_input_prompt.is_some());
                let ans: Option<(IdWrapper, bool, String)> = self.choose_or_create_object(
                    db_in.clone(),
                    //the assert!() above guarantees a safe unwrap()
                    Some(vec![attr_type_input_prompt.unwrap()]),
                    in_previous_selection_desc,
                    in_previous_selection_id,
                    attr_type,
                    0,
                    None,
                    false,
                    None,
                    false,
                    None,
                    false,
                )?;
                match ans {
                    None => (true, 0, false, String::new()),
                    Some((id_wrapper, is_remote, remote_key)) => {
                        //%%let answer = ans.unwrap();
                        //(
                        //    false,
                        //    answer.0.get_id(),
                        //    answer.1, //is_remote,
                        //    answer.2, //remote_key,
                        //)
                        (false, id_wrapper.get_id(), is_remote, remote_key)
                    }
                }
            } else {
                // maybe not ever reached under current system logic. not certain.
                //%%%%%make inout_dh.call be checking for the enum type:
                let (is_remote, remote_key) =
                    //%%was:
                    //if let Some(holder) = inout_dh.as_relation_to_entity_data_holder() {
                    //    (holder.is_remote, holder.remote_instance_id.clone())
                    //} else {
                    //    (false, String::new())
                    //};
                    match inout_dh {
                        AttributeDataHolder::RelationToEntityDH{ rtedh } => (rtedh.is_remote, rtedh.remote_instance_id.clone()),
                        _ => (false, String::new()),
                    };
                (false, inout_dh.get_attr_type_id()?, is_remote, remote_key)
            };
        if user_wants_out {
            return Ok(None);
        }
        inout_dh.set_attr_type_id(attr_type_id);
        //%%%%%see just above re this: ck for enum type
        if let Some(holder) = inout_dh.as_relation_to_entity_data_holder_mut() {
            holder.is_remote = is_remote;
            holder.remote_instance_id = remote_key;
        }
        let ans2 = ask_for_other_info(&self, db_in, inout_dh, editing_in, &self.ui)?;
        let Some(mut inout_dh) = ans2 else {
            return Ok(None);
        };
        let mut user_wants_to_cancel = false;
        //%%%%%use some ck of the enum instead of inout_dh.this:
        //case dhWithVOD: AttributeDataHolderWithVODates =>
        if let Some(holder) = inout_dh.as_valid_and_observation_dates_data_holder_mut() {
            let (valid_on_date, observation_date, user_wants_to_cancel_inner): (
                Option<i64>,
                i64,
                bool,
            ) = Util::ask_for_attribute_valid_and_observed_dates(
                holder.get_valid_on_date(),
                holder.get_observation_date(),
                &self.ui,
                false,
            );
            if user_wants_to_cancel_inner {
                user_wants_to_cancel = true;
            } else {
                holder.set_valid_on_date(valid_on_date);
                holder.set_observation_date(observation_date);
            }
        }
        if user_wants_to_cancel {
            Ok(None)
        } else {
            Ok(Some(inout_dh))
        }
    }

    /// Searches for a regex, case-insensitively, & returns the id of an Entity, or None if
    /// user wants out.  The parameter 'idToOmitIn' lets us omit
    /// (or flag?) an entity if it should be for some reason (like it's the caller/container &
    /// doesn't make sense to be in the group, or something).
    /// Idea: re attrTypeIn parm, enum/improvement: see comment re inAttrType at beginning of chooseOrCreateObject.
    //%%?: @tailrec
    pub fn find_existing_object_by_text(
        &self,
        db_in: Rc<RefCell<dyn Database>>,
        starting_display_row_index_in: u64, /* = 0*/
        attr_type_in: &str,
        id_to_omit_in: Option<i64>, /*= None*/
        regex_in: &str,
    ) -> Result<Option<IdWrapper>, anyhow::Error> {
        let x = format!("SEARCH RESULTS: {}", Util::PICK_FROM_LIST_PROMPT);
        let leading_text = vec![x];
        let choices = vec![Util::LIST_NEXT_ITEMS_PROMPT.to_string()];
        let num_displayable_items = self.ui.max_columnar_choices_to_display_after(
            leading_text.len(),
            choices.len(),
            u16::try_from(Util::max_name_length())?,
        )?;
        //%%OR NOT?:
        //one of these 2 will be used depending on the attr_type_in:
        let mut entities_to_display: Vec<Entity> = Vec::new();
        let mut groups_to_display: Vec<Group> = Vec::new();
        /*%%let objects_to_display: Vec<Box<dyn HasId>> = */
        match attr_type_in {
            Util::ENTITY_TYPE => {
                entities_to_display = db_in.borrow().get_matching_entities(
                    db_in.clone(),
                    None,
                    starting_display_row_index_in,
                    Some(num_displayable_items),
                    id_to_omit_in,
                    regex_in.to_string(),
                )? //%%?: .into_iter().map(|e| Box::new(e) as Box<dyn HasId>).collect()
            }
            Util::GROUP_TYPE => {
                groups_to_display = db_in.borrow().get_matching_groups(
                    db_in.clone(),
                    None,
                    starting_display_row_index_in,
                    Some(num_displayable_items),
                    id_to_omit_in,
                    regex_in.to_string(),
                )? //%%?: .into_iter().map(|g| Box::new(g) as Box<dyn HasId>).collect()
            }
            _ => return Err(anyhow!("??")),
        };
        //%%if objects_to_display.is_empty() {
        if (attr_type_in == Util::ENTITY_TYPE && entities_to_display.is_empty())
            || (attr_type_in == Util::GROUP_TYPE && groups_to_display.is_empty())
        {
            self.ui
                .display_text1("End of list, or none found; starting over from the beginning...");
            if starting_display_row_index_in == 0 {
                Ok(None)
            } else {
                self.find_existing_object_by_text(db_in, 0, attr_type_in, id_to_omit_in, regex_in)
            }
        } else {
            let mut object_names: Vec<String> = Vec::new();
            match attr_type_in {
                Util::ENTITY_TYPE => {
                    for entity in entities_to_display.iter_mut() {
                    //entities_to_display.iter().map(|mut entity| {
                        //%%?:
                        //let entity = obj.as_any().downcast_ref::<Entity>().unwrap();
                        //let entity: &mut Entity = obj;
                        let num_subgroups_prefix: String = self.get_entity_content_size_prefix(&entity)?;
                        let s = format!(
                            "{}{}{}",
                            num_subgroups_prefix,
                            entity.get_archived_status_display_string(None)?,
                            entity.get_name(None)?
                        );
                        object_names.push(s.clone());
                    }//%%).collect()
                }
                Util::GROUP_TYPE => {
                    for group in groups_to_display.iter_mut() {
                    //groups_to_display.iter().map(|obj| {
                        //%%?:
                        //let group = obj.as_any().downcast_ref::<Group>().unwrap();
                        let num_subgroups_prefix: String =
                            self.get_group_content_size_prefix(group.get_db(), group.get_id())?;
                        let s = format!("{}{}", num_subgroups_prefix, group.get_name(None)?);
                        object_names.push(s);
                    }//%%).collect()
                }
                _ => return Err(anyhow!("??")),
            };
            let ans = self.ui.ask_which_choice_or_its_alternate(
                Some(leading_text),
                &choices,
                &object_names,
                true,
                None,
                None,
                None,
                None,
            );
            let Some((answer, user_chose_alternate)) = ans else {
                //%%make sure this works both ways (let and else)
                return Ok(None);
            };
            if answer == 1 && answer <= choices.len() {
                // (For reason behind " && answer <= choices.len()", see comment where it is used in entityMenu.)
                //%%fix "as" here:
                let next_starting_index =
                    starting_display_row_index_in + u64::try_from(object_names.len())?;
                self.find_existing_object_by_text(
                    db_in,
                    next_starting_index,
                    attr_type_in,
                    id_to_omit_in,
                    regex_in,
                )
            } else if answer > choices.len() && answer <= (choices.len() + object_names.len()) {
                let index = answer - choices.len() - 1;
                //let obj = &objects_to_display[index];
                if user_chose_alternate {
                    match attr_type_in {
                        // idea (or was one, in scala; reconsider?): replace this condition by use of a trait (the
                        // type of o, which has get_id), or being smarter with scala's type system. attrTypeIn match
                        Util::ENTITY_TYPE => {
                            //%%?:
                            //let entity = obj.as_any().downcast_ref::<Entity>().unwrap();
                            let Some(entity) = entities_to_display.get(index) else {
                                return Err(anyhow!("unable to find entity for index {}", index));
                            };
                            self.ui.display_text1("%%need2impl EntityMenu");
                            //EntityMenu::new(self.ui.clone(), Rc::new(self.clone()))
                            //    //%%?:
                            //    .entity_menu(obj as Entity, None);
                            //    .entity_menu(entity, None);
                        }
                        Util::GROUP_TYPE => {
                            // for now, picking the first RTG found for this group, until it's clear which of its RTGs to use.
                            // (see also the other locations w/ similar comment!)
                            // (There is probably no point in showing this GroupMenu with RTG info, since which RTG
                            // to use was picked arbitrarily, except if
                            // that added info is a convenience, or if it helps the user clean up orphaned data sometimes.)
                            // //%%?:
                            //let group = obj.as_any().downcast_ref::<Group>().unwrap();
                            let Some(group) = groups_to_display.get(index) else {
                                return Err(anyhow!("unable to find group for index {}", index));
                            };
                            let some_relation_to_groups: Vec<RelationToGroup> =
                                group.get_containing_relations_to_group(None, 0, Some(1))?;
                            if some_relation_to_groups.is_empty() {
                                self.ui.display_text1(Util::ORPHANED_GROUP_MESSAGE);
                                //%%Rc? clones?:
                                self.ui.display_text1("%%need2implement GroupMenu...");
                                //GroupMenu::new(self.ui.clone(), Rc::new(self.clone()))
                                //    .group_menu(group, 0, None, None);
                            } else {
                                self.ui.display_text1("%%need2implement GroupMenu...");
                                //%%Rc? clones? [0] vs get?:
                                //%%
                                //GroupMenu::new(self.ui.clone(), Rc::new(self.clone())).group_menu(
                                //    group,
                                //    0,
                                //    Some(&some_relation_to_groups[0]),
                                //    None,
                                //);
                            }
                        }
                        _ => return Err(anyhow!("??")),
                    }
                    self.find_existing_object_by_text(
                        db_in,
                        starting_display_row_index_in,
                        attr_type_in,
                        id_to_omit_in,
                        regex_in,
                    )
                } else {
                    // user typed a letter to select.. (now 0-based); selected a new object and so we return to
                    // the previous menu w/ that one displayed & current
                    match attr_type_in {
                        Util::ENTITY_TYPE => {
                            let Some(entity) = entities_to_display.get(index) else {
                                return Err(anyhow!("err 2: unable to find entity for index {}", index));
                            };
                            Ok(Some(IdWrapper::new(entity.get_id())))
                        }
                        Util::GROUP_TYPE => {
                            let Some(group) = groups_to_display.get(index) else {
                                return Err(anyhow!("err 2: unable to find group for index {}", index));
                            };
                            Ok(Some(IdWrapper::new(group.get_id())))
                        }
                        _ => Err(anyhow!("Unexpected attr_type_in: {}", attr_type_in))
                    }
                }
            } else {
                self.ui.display_text1("unknown choice among secondary list");
                self.find_existing_object_by_text(
                    db_in,
                    starting_display_row_index_in,
                    attr_type_in,
                    id_to_omit_in,
                    regex_in,
                )
            }
        }
    }

    /// The param containing_group_in lets us omit entities that are already in a group,
    /// i.e. omitting them from the list of entities (e.g. to add to the group), that this method returns.
    /// Returns: None if user wants out, otherwise: a relevant id, a Boolean indicating if the id is for an object in
    /// a remote OM instance, and if the object selected represents the key of a remote instance, that key as a String.
    /// Idea: the object_type_in parm: do like in java & make it some kind of enum for type-safety? What's the scala
    /// idiom for that? (see also other
    /// mentions of object_type_in (or still using old name, inAttrType) for others to fix as well.)
    /// Idea: this should be refactored for simplicity, perhaps putting logic now conditional on object_type_in in
    /// a trait & types that have it (tracked in tasks).
    //%%: @tailrec? Idea (and is tracked): putting this back got Scala compiler error on line 1218 call to chooseOrCreateObject.
    pub fn choose_or_create_object(
        &self,
        db_in: Rc<RefCell<dyn Database>>,
        leading_text_in: Option<Vec<&str>>,
        previous_selection_desc_in: Option<String>,
        previous_selection_id_in: Option<i64>,
        object_type_in: &str,
        starting_display_row_index_in: u64,         /* 0 */
        class_id_in: Option<i64>,                   /* None */
        limit_by_class_in: bool,                    /*false*/
        containing_group_in: Option<i64>,           /*None*/
        mark_previous_selection_in: bool,           /*false*/
        show_only_attribute_types_in: Option<bool>, /*None*/
        quantity_seeks_unit_not_type_in: bool,      /*false*/
    ) -> Result<Option<(IdWrapper, bool, String)>, anyhow::Error> {
        if class_id_in.is_some() {
            //%%return err instd?
            assert_eq!(object_type_in, Util::ENTITY_TYPE);
        }
        if quantity_seeks_unit_not_type_in {
            //%%return err instd?
            assert_eq!(object_type_in, Util::QUANTITY_TYPE);
        }
        let entity_and_most_attr_type_names = vec![
            Util::ENTITY_TYPE,
            Util::QUANTITY_TYPE,
            Util::DATE_TYPE,
            Util::BOOLEAN_TYPE,
            Util::FILE_TYPE,
            Util::TEXT_TYPE,
        ];
        let even_more_attr_type_names = vec![
            Util::ENTITY_TYPE,
            Util::TEXT_TYPE,
            Util::QUANTITY_TYPE,
            Util::DATE_TYPE,
            Util::BOOLEAN_TYPE,
            Util::FILE_TYPE,
            Util::RELATION_TYPE_TYPE,
            Util::RELATION_TO_LOCAL_ENTITY_TYPE,
            Util::RELATION_TO_GROUP_TYPE,
        ];
        let list_next_items_choice_num = 1;
        let (num_objects_available, show_only_attribute_types) = {
            if Util::NON_RELATION_ATTR_TYPE_NAMES.contains(&object_type_in) {
                // ** KEEP THESE QUERIES AND CONDITIONS IN SYNC W/ THE COROLLARY ONES 1x ELSEWHERE ! (at similar comment):
                if show_only_attribute_types_in.is_none() {
                    let count_of_entities_used_as_this_attr_type = db_in
                        .borrow()
                        .get_count_of_entities_used_as_attribute_types(
                            None,
                            object_type_in,
                            quantity_seeks_unit_not_type_in,
                        )?;
                    if count_of_entities_used_as_this_attr_type > 0 {
                        (count_of_entities_used_as_this_attr_type, true)
                    } else {
                        (db_in.borrow().get_entity_count(None)?, false)
                    }
                } else if show_only_attribute_types_in.unwrap() {
                    //above unwrap() guaranteed safe by condition higher up.
                    (
                        db_in
                            .borrow()
                            .get_count_of_entities_used_as_attribute_types(
                                None,
                                object_type_in,
                                quantity_seeks_unit_not_type_in,
                            )?,
                        true,
                    )
                } else {
                    (db_in.borrow().get_entity_count(None)?, false)
                }
            } else if object_type_in == Util::ENTITY_TYPE {
                (
                    db_in.borrow().get_entities_only_count(
                        None,
                        limit_by_class_in,
                        class_id_in,
                        previous_selection_id_in,
                    )?,
                    false
                )
            } else if Util::RELATION_ATTR_TYPE_NAMES.contains(&object_type_in) {
                (db_in.borrow().get_relation_type_count(None)?, false)
            } else if object_type_in == Util::ENTITY_CLASS_TYPE {
                (db_in.borrow().get_class_count(None, None)?, false)
            } else if object_type_in == Util::OM_INSTANCE_TYPE {
                (db_in.borrow().get_om_instance_count(None)?, false)
            } else {
                //%%return err instead
                return Err(anyhow!("invalid object_type_in: {}", object_type_in))
            }
        };
        // Build choice list
        let (
            choices,
            keep_previous_selection_choice,
            create_entity_or_attr_type_choice,
            search_for_entity_by_name_choice,
            search_for_entity_by_id_choice,
            show_journal_choice,
            create_relation_type_choice,
            create_class_choice,
            create_instance_choice,
            swap_objects_to_display_choice,
            link_to_remote_instance_choice,
        ): (
            Vec<String>,
            u16,
            u16,
            u16,
            u16,
            u16,
            u16,
            u16,
            u16,
            u16,
            u16,
        ) = self.get_choice_list(
            object_type_in,
            previous_selection_desc_in.as_deref(),
            &entity_and_most_attr_type_names,
            show_only_attribute_types,
        );
        let mut choices_strs: Vec<String> = Vec::new();
        for s in choices.iter() {
            choices_strs.push(s.clone());
        };
        let (leading_text, objects_to_display, statuses_and_names) = self
            .get_lead_text_and_object_list(
                choices_strs,
                object_type_in,
                &leading_text_in,
                num_objects_available,
                starting_display_row_index_in,
                show_only_attribute_types,
                quantity_seeks_unit_not_type_in,
                class_id_in,
                limit_by_class_in,
                previous_selection_id_in,
                containing_group_in,
            )?;
        let leading_text_as_strs: Vec<String> = leading_text.iter().map(|s| s).collect();
        let choices_as_strs: Vec<String> = choices.iter().map(|s| s.clone()).collect();
        let ans = self.ui.ask_which_choice_or_its_alternate(
            Some(leading_text_as_strs),
            &choices_as_strs,
            //statuses_and_names.iter().map(|s| s.as_str()).collect(),
            &statuses_and_names,
            true,
            None,
            None,
            None,
            None,
        );
        let Some((answer, user_chose_alternate)) = ans else {
            return Ok(None);
        };
        // Handle menu choices
        if answer == list_next_items_choice_num && answer <= choices.len() && !user_chose_alternate
        {
            let index = self.get_next_starting_object_index(
                starting_display_row_index_in,
                objects_to_display.len(), /*%%as i64*/
                num_objects_available,
            );
            self.choose_or_create_object(
                db_in,
                leading_text_in,
                previous_selection_desc_in,
                previous_selection_id_in,
                object_type_in,
                index,
                class_id_in,
                limit_by_class_in,
                containing_group_in,
                mark_previous_selection_in,
                Some(show_only_attribute_types),
                quantity_seeks_unit_not_type_in,
            )
        } else if answer == usize::from(keep_previous_selection_choice) && answer <= choices.len() {
            // Such as if editing several fields on an attribute and doesn't want to change the first one.
            // Not using "get out" option for this because it would exit from a few levels at once and
            // then user wouldn't be able to proceed to other field edits.
            //%%remove unwrap?
            Ok(Some((
                IdWrapper::new(previous_selection_id_in.unwrap()),
                false,
                String::new(),
            )))
        } else if answer == usize::from(create_entity_or_attr_type_choice) && answer <= choices.len() {
            let e: Option<Entity> =
                self.ask_for_class_info_and_name_and_create_entity(db_in, class_id_in)?;
            Ok(e.map(|entity| (IdWrapper::new(entity.get_id()), false, String::new())))
        } else if answer == usize::from(search_for_entity_by_name_choice) && answer <= choices.len() {
            let idw = self.ask_for_name_and_search_for_entity(db_in)?;
            //%%Ok(result.map(|id| (id, false, String::new())))
            match idw {
                Some(w) => Ok(Some((w, false, String::new()))),
                None => Ok(None),
            }
            //%%Ok(Some((x, false, String::new())))
        } else if answer == usize::from(search_for_entity_by_id_choice) && answer <= choices.len() {
            let idw = self.search_by_id(db_in, Util::ENTITY_TYPE)?;
            //%%Ok(result.map(|id| (id, false, String::new())))
            match idw {
                Some(w) => Ok(Some((w, false, String::new()))),
                None => Ok(None),
            }
            //%%Ok((idw, false, String::new()))
        } else if answer == usize::from(show_journal_choice) && answer <= choices.len() {
            self.show_journal(db_in);
            Ok(None)
        } else if answer == usize::from(swap_objects_to_display_choice)
            && entity_and_most_attr_type_names.contains(&object_type_in)
            && answer <= choices.len()
        {
            self.choose_or_create_object(
                db_in,
                leading_text_in,
                previous_selection_desc_in,
                previous_selection_id_in,
                object_type_in,
                0,
                class_id_in,
                limit_by_class_in,
                containing_group_in,
                mark_previous_selection_in,
                Some(!show_only_attribute_types),
                quantity_seeks_unit_not_type_in,
            )
        } else if answer == usize::from(link_to_remote_instance_choice)
            && entity_and_most_attr_type_names.contains(&object_type_in)
            && answer <= choices.len()
        {
            return Err(anyhow!("not yet implemented"));
            //Ok(self.handle_link_to_remote_instance(db_in)?)
        } else if answer == usize::from(create_relation_type_choice)
            && Util::RELATION_ATTR_TYPE_NAMES.contains(&object_type_in)
            && answer <= choices.len()
        {
            //%%replace all these constants (which, all?) with enums? Do they ever really have to
            //be strings for something, or can the enum impl Display easily & well? Ck usages.
            let entity: Option<Entity> = self.ask_for_name_and_write_entity(
                db_in,
                Util::RELATION_TYPE_TYPE,
                Rc::new(RefCell::new(None)),
                None,
                None,
                None,
                None,
                None,
                false,
            )?;
            Ok(entity.map(|e| (IdWrapper::new(e.get_id()), false, String::new())))
        } else if answer == usize::from(create_class_choice)
            && object_type_in == Util::ENTITY_CLASS_TYPE
            && answer <= choices.len()
        {
            let entity = Rc::new(RefCell::new(None));
            let result: Option<i64> =
                //%%test this call: had to add parameters, guessing:
                self.ask_for_and_write_class_and_template_entity_name(db_in, None, entity, &mut None, Util::ENTITY_CLASS_TYPE, 
                    Entity::name_length(), "", false, None, None, None)?;
            if let Some(class_id) = result {
                let ans = self.ui.ask_yes_no_question(
                    "Do you want to add attributes to the newly created template entity for this \
                    class? (These will be used for the \
                    prompts and defaults when creating/editing entities in this class).",
                    "y",
                    false,
                );
                if ans.is_some() && ans.unwrap() {
                    //%%need ui.clone? Rc? self.clone? What is the None for and is it correct?
                    //%%fill in:
                    //EntityMenu::new(self.ui.clone(), Rc::new(self.clone()))
                    //    .entity_menu(&Entity::new2(db_in.clone(), None, entity_id), None);
                }
                Ok(Some((IdWrapper::new(class_id), false, String::new())))
            } else {
                Ok(None)
            }
        } else if answer == usize::from(create_instance_choice)
            && object_type_in == Util::OM_INSTANCE_TYPE
            && answer <= choices.len()
        {
            return Err(anyhow!("ask_for_and_write_om_instance_info not impl yet"));
            //%%
            //let result: Option<String> = self.ask_for_and_write_om_instance_info(db_in, None);
            ////%%%is the new(0) going to have the same effect as null? is correct??
            //// In scala at least, using null on next line was easier than the visible alternatives (same
            //// in one other place w/ this comment)
            ////%%In scala, next line was:  Some(null, false, result.get)
            //result.map(|id| (IdWrapper::new(0), false, id))
        } else if answer > choices.len() && answer <= (choices.len() + objects_to_display.len()) {
            // those in the condition on the previous line are 1-based, not 0-based.
            let index = answer - choices.len() - 1;
            // user typed a letter to select.. (now 0-based)
            // user selected a new object and so we return to the previous menu w/ that one displayed & current
            //%%use .get instead? scala code line was:
            //let o = objectsToDisplay.get(index);
            let obj = &objects_to_display[index];
            //if "text,quantity,entity,date,boolean,file,relationtype".contains(attrTypeIn)) {
            //i.e., if attrTypeIn == Controller.TEXT_TYPE || (= any of the other types...)):
            if user_chose_alternate {
                match object_type_in {
                    Util::ENTITY_TYPE => {
                        // idea: replace this condition by use of a trait (the type of o, which has get_id),
                        // or being smarter with scala's type system. attrTypeIn match {
                        //%%?:
                        //%%%%%%%%%SEE Anki notes about "type as trait" and try that here? Since they seem to be Any already?:
                        //let entity = obj.as_any().downcast_ref::<Entity>().unwrap();
                        //let entity = obj.downcast_ref::<Entity>().unwrap();
                        //(Note: there is also an as_any crate that could be useful, or search ddg for "rust as_any".)
                        let entity_option = (obj as &dyn Any).downcast_ref::<Entity>();
                        let Some(entity) = entity_option else {
                            return Err(anyhow!("unexpected result from downcast_ref: {:?}", entity_option));
                        };
                        //let entity = (**obj as dyn std::any::Any).downcast_ref::<Entity>().unwrap();
                        //let entity = (*obj as dyn HasId).downcast_ref::<Entity>().unwrap();
                        //%%needed?: clone, rc, clone, None:
                        //%%:
                        let obj_any = obj as &dyn Any;
                        self.ui.display_text1(format!("obj type_id info right?: {:?}", obj_any.type_id()).as_str());
                        let entity_any = entity as &dyn Any;
                        self.ui.display_text1(format!("entity_any type info right?: {:?}", entity_any.type_id()).as_str());
                        let it_is = TypeId::of::<Entity>() == entity_any.type_id();
                        self.ui.display_text1(format!("entity type info right?: {:?}", it_is).as_str());
                        return Err(anyhow!("EntityMenu not yet impl--WHEN FIXING PUT BACK %%%%% code just below!"));
                        //EntityMenu::new(self.ui.clone(), Rc::new(self.clone()))
                            //.entity_menu(entity, None);
                    }
                    // (choosing a group doesn't call this, it calls choose_or_create_group)
                    _ => {
                        return Err(anyhow!("not yet implemented"));
                    }
                }
                    /*%%%%%PUT BACK WHEN implementing entityMenu just above!
                self.choose_or_create_object(
                    db_in,
                    leading_text_in,
                    previous_selection_desc_in,
                    previous_selection_id_in,
                    object_type_in,
                    starting_display_row_index_in,
                    class_id_in,
                    limit_by_class_in,
                    containing_group_in,
                    mark_previous_selection_in,
                    Some(show_only_attribute_types),
                    quantity_seeks_unit_not_type_in,
                )
                    */
            } else {
                let obj_any = obj as &dyn Any;
                if even_more_attr_type_names.contains(&object_type_in) {
                    let Some(entity) = obj_any.downcast_ref::<Entity>() else {
                        return Err(anyhow!("unexpected inability to downcast_ref to Entity"));
                    };
                    Ok(Some((entity.get_id_wrapper(), false, String::new())))
                } else if object_type_in == Util::ENTITY_CLASS_TYPE {
                    let Some(class) = obj_any.downcast_ref::<EntityClass>() else {
                        return Err(anyhow!("unexpected inability to downcast_ref to EntityClass"));
                    };
                    Ok(Some((class.get_id_wrapper(), false, String::new())))
                } else if object_type_in == Util::OM_INSTANCE_TYPE {
                    let Some(instance) = obj_any.downcast_ref::<OmInstance>() else {
                        return Err(anyhow!("unexpected inability to downcast_ref to OmInstance"));
                    };
                    // In scala at least, using null on next line was easier than the visible alternatives (same
                    // in one other place w/ this comment)
                    //%%%is this new(0) the same as "null" in scala? same effect?? see similar place
                    //above!
                    Ok(Some((IdWrapper::new(0), false, instance.get_id())))
                } else {
                    return Err(anyhow!("invalid object_type_in: {:?}", object_type_in));
                }
            }
        } else {
            self.ui
                .display_text1("unknown response in chooseOrCreateObject");
            self.choose_or_create_object(
                db_in,
                leading_text_in,
                previous_selection_desc_in,
                previous_selection_id_in,
                object_type_in,
                starting_display_row_index_in,
                class_id_in,
                limit_by_class_in,
                containing_group_in,
                mark_previous_selection_in,
                Some(show_only_attribute_types),
                quantity_seeks_unit_not_type_in,
            )
        }
    }

    // Helper method for choose_or_create_object
    fn get_choice_list(
        &self,
        object_type_in: &str,
        previous_selection_desc_in: Option<&str>,
        entity_and_most_attr_type_names: &[&str],
        show_only_attribute_types: bool,
    ) -> (
        Vec<String>,
        u16,
        u16,
        u16,
        u16,
        u16,
        u16,
        u16,
        u16,
        u16,
        u16,
    ) {
        // Attempt to keep these straight even though the size of the list, hence their option #'s on the menu,
        // is conditional:
        let mut keep_previous_selection_choice_num = 1;
        let mut create_attr_type_choice_num = 1;
        let mut search_for_entity_by_name_choice_num = 1;
        let mut search_for_entity_by_id_choice_num = 1;
        let mut show_journal_choice_num = 1;
        let mut swap_objects_to_display_choice_num = 1;
        let mut link_to_remote_instance_choice_num = 1;
        let mut create_relation_type_choice_num = 1;
        let mut create_class_choice_num = 1;
        let mut create_instance_choice_num = 1;
        let mut choice_list = vec![Util::LIST_NEXT_ITEMS_PROMPT.to_string()];
        if previous_selection_desc_in.is_some() {
            choice_list.push(format!(
                "Keep previous selection ({}).",
                previous_selection_desc_in.unwrap()
            ));
            keep_previous_selection_choice_num += 1;
            // inserted a menu option, so add 1 to all the others' indexes.
            create_attr_type_choice_num += 1;
            search_for_entity_by_name_choice_num += 1;
            search_for_entity_by_id_choice_num += 1;
            show_journal_choice_num += 1;
            swap_objects_to_display_choice_num += 1;
            link_to_remote_instance_choice_num += 1;
            create_relation_type_choice_num += 1;
            create_class_choice_num += 1;
            create_instance_choice_num += 1;
        }
        //idea: use match instead of if: can it do || ?
        if entity_and_most_attr_type_names.contains(&object_type_in) {
            // insert the several other menu options, and add the right # to the index of each.
            choice_list.push(Util::MENUTEXT_CREATE_ENTITY_OR_ATTR_TYPE.to_string());
            create_attr_type_choice_num += 1;
            choice_list.push(
                "Search for existing entity by name and text attribute content...".to_string(),
            );
            search_for_entity_by_name_choice_num += 2;
            choice_list.push("Search for existing entity by id...".to_string());
            search_for_entity_by_id_choice_num += 3;
            choice_list.push("Show journal (changed entities) by date range...".to_string());
            show_journal_choice_num += 4;
            if show_only_attribute_types {
                choice_list.push(format!(
                    "show all entities (not only those already used as a type of {})",
                    object_type_in
                ));
            } else {
                choice_list.push(format!(
                    "show only entities ALREADY used as a type of {}",
                    object_type_in
                ));
            }
            swap_objects_to_display_choice_num += 5;
            choice_list.push("Link to entity in a separate (REMOTE) OM instance...".to_string());
            link_to_remote_instance_choice_num += 6;
        } else if Util::RELATION_ATTR_TYPE_NAMES.contains(&object_type_in) {
            // These choice #s are only hit by the conditions below, when they should be...:
            choice_list.push(Util::menutext_create_relation_type());
            create_relation_type_choice_num += 1;
        } else if object_type_in == Util::ENTITY_CLASS_TYPE {
            choice_list.push("Create new class (template for new entities)".to_string());
            create_class_choice_num += 1;
        } else if object_type_in == Util::OM_INSTANCE_TYPE {
            choice_list.push(
                "Create new OM instance (a remote data store for lookup, linking, etc.)"
                    .to_string(),
            );
            create_instance_choice_num += 1;
        } else {
            //%%ret err instd? scala threw exc.
            panic!("invalid object_type_in: {}", object_type_in);
        }
        (
            choice_list,
            keep_previous_selection_choice_num,
            create_attr_type_choice_num,
            search_for_entity_by_name_choice_num,
            search_for_entity_by_id_choice_num,
            show_journal_choice_num,
            create_relation_type_choice_num,
            create_class_choice_num,
            create_instance_choice_num,
            swap_objects_to_display_choice_num,
            link_to_remote_instance_choice_num,
        )
    }
}
