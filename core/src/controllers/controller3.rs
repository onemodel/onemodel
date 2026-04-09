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
    pub fn go_to_entity_or_its_sole_groups_menu(
        &self,
        user_selection: &Entity,
        relation_to_group_in: Option<&RelationToGroup>, /*None*/
        containing_group_in: Option<&Group>,            /*None*/
    ) -> Result<(Option<Entity>, Option<i64>, bool), anyhow::Error> {
        let (rtg_id, rt_id, group_id, _, more_than_one_available): (
            Option<i64>,
            Option<i64>,
            Option<i64>,
            Option<String>,
            bool,
        ) = user_selection.find_relation_to_and_group(None)?;
        let sub_entity_selected: Option<Entity> = None;
        if group_id.is_some()
            && !more_than_one_available
            && user_selection.get_attribute_count(None, self.db.borrow().include_archived_entities())? == 1
        {
            // In quick menu, for efficiency of some work like brainstorming, if it's obvious which
            // subgroup to go to, just go there.
            // We DON'T want @tailrec on this method for this call, so that we can ESC back to the current
            // menu & list! (so what balance/best? Maybe move this
            // to its own method, so it doesn't try to tail optimize it?)  See also the comment with 'tailrec',
            // mentioning why to have it, above.
            //Was, in scala: IF ADDING ANY OPTIONAL PARAMETERS, be sure they
            //are also passed along in the recursive call(s)
            // w/in this method!
            self.ui.display_text1("not yet implemented");
            //%%
            //QuickGroupMenu::new(self.ui.clone(), Rc::new(self.clone())).quick_group_menu(
            //    &Group::new2(user_selection.get_db(), None, group_id.unwrap()),
            //    0,
            //    Some(&RelationToGroup::new2(
            //        user_selection.get_db(),
            //        None,
            //        rtg_id.unwrap(),
            //        user_selection.get_id(),
            //        rt_id.unwrap(),
            //        group_id.unwrap(),
            //    )),
            //    relation_to_group_in,
            //    Some(user_selection),
            //);
        } else {
            self.ui.display_text1("not yet implemented");
            //%%
            //EntityMenu::new(self.ui.clone(), Rc::new(self.clone()))
            //    .entity_menu(user_selection, containing_group_in);
        }
        Ok((sub_entity_selected, group_id, more_than_one_available))
    }

    /// see comments for Entity.getContentSizePrefix. 
    pub fn get_group_content_size_prefix(
        &self,
        db_in: Rc<RefCell<dyn Database>>,
        group_id: i64,
    ) -> Result<String, anyhow::Error> {
        let grp_size = db_in.borrow().get_group_size(None, group_id, 1)?;
        if grp_size == 0 {
            Ok(String::new())
        } else {
            Ok(">".to_string())
        }
    }

    /// Shows ">" in front of an entity or group if it contains exactly one attribute or a subgroup
    /// which has at least one entry; shows ">>" if contains
    /// multiple subgroups or attributes, and "" if contains no subgroups or the one subgroup is empty.
    /// Idea: this might better be handled in the textui class instead, and the same for all the other color stuff.
    pub fn get_entity_content_size_prefix(&self, entity_in: &Entity) -> Result<String, anyhow::Error> {
        // attr_count counts groups also, so account for the overlap in the below.
        let attr_count = entity_in.get_attribute_count(None, self.db.borrow().include_archived_entities())?;
        // This is to not show that an entity contains more things (">" prefix...) if it only has
        // one group which has no *non-archived* entities:
        let has_one_empty_group: bool = {
            let num_groups = entity_in.get_relation_to_group_count(None)?;
            if num_groups != 1 {
                false
            } else {
                let (_, _, gid, _, more_available) = entity_in.find_relation_to_and_group(None)?;
                if gid.is_none() || more_available {
                    //%%%return err instead
                    return Err(anyhow!(
                        "Found {} but by the earlier checks, there should be exactly one group in entity {}.",
                        if gid.is_none() { "0" } else { ">1" },
                        entity_in.get_id()
                    ));
                }
                let group_size = entity_in.get_db().borrow().get_group_size(None, gid.unwrap(), 1)?;
                group_size == 0
            }
        };
        let subgroups_count_prefix: String = {
            if attr_count == 0 || (attr_count == 1 && has_one_empty_group) {
                String::new()
            } else if attr_count == 1 {
                ">".to_string()
            } else {
                ">>".to_string()
            }
        };
        Ok(subgroups_count_prefix)
    }

    fn add_entity_to_group(&self, group_in: &mut Group) -> Result<Option<i64>, anyhow::Error> {
        let new_entity_id: Option<i64> = if !group_in.get_mixed_classes_allowed(None)? {
            if group_in.get_size(None, 3)? == 0 {
                // adding 1st entity to this group, so:
                let leading_text = vec![
                    "ADD ENTITY TO A GROUP (**whose class will set the group's enforced \
                                   class, even if 'None'**):",
                ];
                let id_wrapper: Option<(IdWrapper, _, _)> = self.choose_or_create_object(
                    group_in.get_db(),
                    Some(leading_text),
                    None,
                    None,
                    Util::ENTITY_TYPE,
                    0,
                    None,
                    false,
                    Some(group_in.get_id()),
                    false,
                    None,
                    false,
                )?;
                if let Some((wrapper, _, _)) = id_wrapper {
                    group_in.add_entity(None, wrapper.get_id(), None); /*%%.ok();?*/
                    Some(wrapper.get_id())
                } else {
                    None
                }
            } else {
                // It's not the 1st entry in the group, so add an entity using the same class as those
                // previously added (or None as the case may be).
                let entity_class_in_use: Option<i64> = group_in.get_class_id(None)?;
                let id_wrapper: Option<(IdWrapper, _, _)> = self.choose_or_create_object(
                    group_in.get_db(),
                    None,
                    None,
                    None,
                    Util::ENTITY_TYPE,
                    0,
                    entity_class_in_use,
                    true,
                    Some(group_in.get_id()),
                    false,
                    None,
                    false,
                )?;
                if let Some((wrapper, _, _)) = id_wrapper {
                    let entity_id = wrapper.get_id();
                    match group_in.add_entity(None, entity_id, None) {
                        Ok(_) => Some(entity_id),
                        Err(e) => {
                            if e.to_string().contains(Util::MIXED_CLASSES_EXCEPTION) {
                                let old_class = if let Some(class_id) = entity_class_in_use {
                                    EntityClass::new2(group_in.get_db(), None, class_id)?
                                        .get_display_string(None, false)
                                } else {
                                    "(none)".to_string()
                                };
                                let new_class_id =
                                    Entity::new2(group_in.get_db(), None, entity_id)?.get_class_id(None)?;
                                let new_class: String =
                                    if new_class_id.is_none() || entity_class_in_use.is_none() {
                                        "(none)".to_string()
                                    } else {
                                        let class_id = entity_class_in_use.unwrap();
                                        EntityClass::new2(group_in.get_db(), None, class_id)?
                                            .get_display_string(None, false)
                                    };
                                self.ui.display_text1(&format!(
                                    "Adding an entity with class '{}' to a group that doesn't allow mixed classes, \
                                    and which already has an entity with class '{}' generates an error. \
                                    The program should have prevented this by \
                                    only showing entities with a matching class, but in any case the mismatched \
                                    entity was not added to the group.",
                                    new_class, old_class
                                ));
                                None
                            } else {
                                return Err(anyhow!("{}", e));
                            }
                        }
                    }
                } else {
                    None
                }
            }
        } else {
            let leading_text = vec!["ADD ENTITY TO A (mixed-class) GROUP"];
            let id_wrapper: Option<(IdWrapper, _, _)> = self.choose_or_create_object(
                group_in.get_db(),
                Some(leading_text),
                None,
                None,
                Util::ENTITY_TYPE,
                0,
                None,
                false,
                Some(group_in.get_id()),
                false,
                None,
                false,
            )?;
            if let Some((wrapper, _, _)) = id_wrapper {
                group_in.add_entity(None, wrapper.get_id(), None); /*%%?:.ok();*/
                Some(wrapper.get_id())
            } else {
                None
            }
        };
        Ok(new_entity_id)
    }

    fn choose_among_entities(&self, containing_entities: &mut Vec<(i64, Entity)>) -> Result<Option<Entity>, anyhow::Error> {
        let leading_text = vec!["Pick from menu, or an entity by letter"];
        let choices = vec![Util::LIST_NEXT_ITEMS_PROMPT.to_string()];
        //(see comments at similar location in EntityMenu, as of this writing [in scala] on line 288)
        let mut containing_entities_names_with_rel_types: Vec<String> = Vec::new();
        for (mut rel_type_id, mut entity) in containing_entities.iter() {
            //%% .map(|(rel_type_id, entity)| {
            let mut rel_type = RelationType::new2(entity.get_db(), None, rel_type_id)?;
            let rel_type_name: String = format!(
                "{}{}",
                //%%%%%%%is this called? does this make sense, to use the same entity as passed
                //in?
                entity/*%%rel_type*/.get_archived_status_display_string(None)?,
                rel_type.get_name(None)?
            );
            //%%is this called? does this "this group" mention make sense? seems to have been
            //in below scala code also.
            containing_entities_names_with_rel_types.push(format!(
                "the entity \"{}{}\" {} this group",
                entity.get_archived_status_display_string(None)?,
                entity.get_name(None)?,
                rel_type_name
            ));
            // other possible displays:
            //1) entity.get_name + " - " + relTypeName + " this
            // group"
            //2) "entity " + entityName + " " +
            //rtg.get_display_string(maxNameLength, None, Some(rt))
        };//%%).collect();
        /*%% Scala code for above "let..." was:
        let containingEntitiesNamesWithRelTypes: Vec<String> = containingEntities.toArray.map {
                                      case rel_type_idAndEntity: (i64, Entity) =>
                                        let rel_type_id: i64 = rel_type_idAndEntity._1;
                                        let entity: Entity = rel_type_idAndEntity._2;
                                        let relTypeName: String = {
                                          let relType = new RelationType(entity.db, rel_type_id);
                                          relType.get_archived_status_display_string + relType.get_name
                                        }
                                        "the entity \"" + entity.get_archived_status_display_string +
                                        entity.get_name + "\" " + relTypeName + " this group"
                                      case _ => throw new OmException("??")
                                    }
        */
        let mut names_as_strs: Vec<String> = Vec::new();
        for s in containing_entities_names_with_rel_types {
            names_as_strs.push(s.clone());
        }
        let ans = self.ui.ask_which(
            Some(leading_text),
            &choices, /*%%?:.clone()*/
            // containing_entities_names_with_rel_types.iter(),
            &names_as_strs,
            true,
            None,
            None,
            None,
            None,
        );
        if ans.is_none() {
            return Ok(None);
        }
        let answer = ans.unwrap();
        if answer == 1 && answer <= choices.len() {
            // see comment above
            self.ui.display_text1("not yet implemented");
            Ok(None)
        } else if answer > choices.len() && answer <= (choices.len() + containing_entities.len()) {
            // those in the condition on the previous line are 1-based, not 0-based.
            let index = answer - choices.len() - 1;
            // user typed a letter to select.. (now 0-based); selected a new object and so we return
            // to the previous menu w/ that one displayed & current
            Ok(Some(containing_entities[index].1.clone() /*%%?:.clone()*/))
        } else {
            self.ui.display_text1("unknown response");
            Ok(None)
        }
    }

    fn get_public_status_display_string(
        &self,
        entity_in: &mut Entity,
    ) -> Result<String, anyhow::Error> {
        //idea: maybe this (logic) knowledge really belongs in the TextUI class. (As some others, probably.)
        if self.show_public_private_status_preference.unwrap_or(false) {
            entity_in.get_public_status_display_string_with_color(None, false)
        } else {
            Ok(String::new())
        }
    }

    /// The parameter attr_form_in Contains the result of passing the right Util::<string constant> to
    /// db.get_attribute_form_id (SEE ALSO COMMENTS IN entity_menu.add_attribute which passes in "other"
    /// form_ids).  BUT, there are also cases where it is a # higher than those found in db.get_attribute_form_id,
    /// and in that case is handled specially here.
    /// Returns None if user wants out (or attr_form_in parm was an abortive mistake?); returns the created Attribute
    /// if successful.
    fn add_attribute(
        &self,
        entity_in: &mut Entity,
        _starting_attribute_index_in: i32,
        attr_form_in: i32,
        attr_type_id_in: Option<i64>,
    ) -> Result<Option<Box<dyn Attribute>>, anyhow::Error> {
        let (attr_type_id, ask_for_attr_type_id): (i64, bool) = if let Some(id) = attr_type_id_in {
            (id, false)
        } else {
            (0, true)
        };
        if attr_form_in == self.db.borrow().get_attribute_form_id(Util::QUANTITY_TYPE)? {
            let add_quantity_attribute = |dh: &mut AttributeDataHolder/*::QuantityAttributeDH*/, entity_in: &Entity| 
                -> Result<Option<Box<dyn Attribute>/*%%? QuantityAttribute*/>, anyhow::Error> {
                match dh {
                    AttributeDataHolder::QuantityAttributeDH{ qadh } => {
                        let qa = entity_in.add_quantity_attribute(
                            None,
                            qadh.attr_type_id,
                            qadh.unit_id,
                            qadh.number,
                            None,
                            //dh.valid_on_date,
                            //dh.observation_date,
                            //%%%was, in scala: ).ok().map(|attr| Box::new(attr) as Box<dyn Attribute>)
                        )?;
                        Ok(Some(Box::new(qa)))
                    },
                    _ => {
                        Err(anyhow!("Unexpected variant of attributeDataHolder: {:?}", dh))
                    }
                }
            };
            let qadh = QuantityAttributeDH {
                attr_type_id,
                valid_on_date: None,
                //%%is same now as used elsewhere like in util?:
                observation_date: chrono::Local::now().timestamp_millis(),
                number: 0.0,
                unit_id: 0,
            };
            let mut qadh_variant = AttributeDataHolder::QuantityAttributeDH { qadh: qadh };
            let fn_afqanau: fn(&Controller, Rc<RefCell<dyn Database>>, &mut AttributeDataHolder, bool, &TextUI) -> Result<Option<AttributeDataHolder>, anyhow::Error> = Self::ask_for_quantity_attribute_number_and_unit;
            let a: Option<Box<dyn Attribute>> = self.ask_for_info_and_add_attribute(
                entity_in.get_db(),
                &mut qadh_variant,
                ask_for_attr_type_id,
                Util::QUANTITY_TYPE,
                Some(Util::QUANTITY_TYPE_PROMPT),
                entity_in,
                //%% Self::ask_for_quantity_attribute_number_and_unit,
                fn_afqanau,
                add_quantity_attribute,
            )?;
            Ok(a)
        } else if attr_form_in == self.db.borrow().get_attribute_form_id(Util::DATE_TYPE)? {
            let add_date_attribute = |dh: &mut AttributeDataHolder/*::DateAttributeDH*/, entity_in: &Entity| 
                -> Result<Option</*%%DateAttribute*/Box<dyn Attribute>>, anyhow::Error> {
                match dh {
                    AttributeDataHolder::DateAttributeDH{ dadh } => {
                        let da = entity_in.add_date_attribute(None, dadh.attr_type_id, dadh.date, None)?;
                        //%%%? was, from claude: .ok().map(|attr| Box::new(attr) as Box<dyn Attribute>)
                        Ok(Some(Box::new(da)))
                    },
                    _ => {
                        Err(anyhow!("Unexpected variant of attributeDataHolder: {:?}", dh))
                    }
                }
            };
            //%%Maybe I need2test all these conditions well in this method.
            //
            //%%%let ask_for_date_attribute_value = Util::ask_for_date_attribute_value;
            let dadh: DateAttributeDH  = DateAttributeDH {
                attr_type_id,
                date: 0,
            };
            let mut dadh_variant = AttributeDataHolder::DateAttributeDH { dadh: dadh };
            let fn_afdav: fn(&Controller, Rc<RefCell<dyn Database>>, &mut AttributeDataHolder, bool, &TextUI) -> Result<Option<AttributeDataHolder>, anyhow::Error> = Util::ask_for_date_attribute_value;
            let a = self.ask_for_info_and_add_attribute(
                entity_in.get_db(),
                &mut dadh_variant,
                ask_for_attr_type_id,
                Util::DATE_TYPE,
                Some("SELECT TYPE OF DATE: "),
                entity_in,
                //%% Util::ask_for_date_attribute_value, //%%%ask_for_date_attribute_value,
                fn_afdav,
                add_date_attribute,
            );
            a
        } else if attr_form_in == self.db.borrow().get_attribute_form_id(Util::BOOLEAN_TYPE)? {
            let add_boolean_attribute =
                |dh: &mut AttributeDataHolder/*::BooleanAttributeDH*/, entity_in: &Entity| 
            -> Result<Option<Box<dyn Attribute>>, anyhow::Error> {
                match dh {
                    AttributeDataHolder::BooleanAttributeDH { badh } => {
                        let ba =
                            entity_in.add_boolean_attribute(None, badh.attr_type_id, badh.boolean, None)?;
                        //%%%claude had, 4 rust: .ok().map(|attr| Box::new(attr) as Box<dyn Attribute>)
                        Ok(Some(Box::new(ba)))
                    },
                    _ => {
                        Err(anyhow!("Unexpected variant of attributeDataHolder: {:?}", dh))
                    }
                }
            };
            let badh = BooleanAttributeDH {
                attr_type_id,
                valid_on_date: None,
                //%%%?:
                observation_date: chrono::Local::now().timestamp_millis(),
                boolean: false,
            };
            let mut badh_variant = AttributeDataHolder::BooleanAttributeDH { badh };
            let fn_afbav: fn(&Controller, Rc<RefCell<dyn Database>>, &mut AttributeDataHolder, bool, &TextUI) -> Result<Option<AttributeDataHolder>, anyhow::Error> = Util::ask_for_boolean_attribute_value;
            let a = self.ask_for_info_and_add_attribute(
                entity_in.get_db(),
                &mut badh_variant,
                ask_for_attr_type_id,
                Util::BOOLEAN_TYPE,
                Some("SELECT TYPE OF TRUE/FALSE VALUE: "),
                entity_in,
                //%% Util::ask_for_boolean_attribute_value,
                fn_afbav,
                add_boolean_attribute,
            );
            a
        } else if attr_form_in == self.db.borrow().get_attribute_form_id(Util::FILE_TYPE)? {
            Err(anyhow!("%%Not yet implemented."))
            // let add_file_attribute = |dh: AttributeDataHolder/*::FileAttributeDH*/| 
            // -> Result<Option<Box<dyn Attribute>>, anyhow::Error> 
            // {
            //     //%%%path? scala code was:
            //     //Some(entity_in.add_file_attribute(dhIn.attr_type_id, dhIn.description, new File(dhIn.original_file_path)))
            //     match dh {
            //         AttributeDataHolder::FileAttributeDH { fadh } => {
            //             let fa = entity_in.add_file_attribute(
            //                 dh.attr_type_id,
            //                 &dh.description,
            //                 &Path::new(&dh.original_file_path),
            //             )?;
            //             //%%%claude code was:  .ok().map(|attr| Box::new(attr) as Box<dyn Attribute>)
            //             Ok(Some(fa))
            //         },
            //         _ => {
            //             Err(anyhow!("Unexpected variant of attributeDataHolder: {:?}", dh))
            //         }
            //     }
            // };
            // let fadhv = FileAttributeDH {
            //     attr_type_id,
            //     description: String::new(),
            //     original_file_path: String::new(),
            // };
            // let result: Option<FileAttribute> = self.ask_for_info_and_add_attribute(
            //     entity_in.get_db(),
            //     &mut fadhv,
            //     ask_for_attr_type_id,
            //     Util::FILE_TYPE,
            //     Some("SELECT TYPE OF FILE: "),
            //     Util::ask_for_file_attribute_info,
            //     add_file_attribute,
            // );
            // if let Some(ref attr) = result {
            //     //%%%?:
            //     let fa = attr.as_any().downcast_ref::<FileAttribute>().unwrap();
            //     let ans = self.ui.ask_yes_no_question(
            //         &format!("Document successfully added. Do you want to DELETE the local copy (at {})?", 
            //             fa.get_original_file_path()),
            //         None,
            //         false,
            //     );
            //     if ans.is_some() && ans.unwrap() {
            //         //%%%%%?:
            //         if !std::fs::remove_file(&fa.get_original_file_path()).is_ok() {
            //             //%%%%reason unknown, give nresult from above line?:
            //             self.ui.display_text(
            //                 "Unable to delete file at that location; reason unknown. You could \
            //                 check the permissions.",
            //             );
            //         }
            //     }
            // }
            // result
        } else if attr_form_in == self.db.borrow().get_attribute_form_id(Util::TEXT_TYPE)? {
            let add_text_attribute = |dh: &mut AttributeDataHolder/*::TextAttributeDH*/, entity_in: &Entity| -> Result<Option<Box<dyn Attribute>>, anyhow::Error> {
                match dh {
                    AttributeDataHolder::TextAttributeDH { tadh } => {
                        let ta = entity_in.add_text_attribute(None, tadh.attr_type_id, &tadh.text, None)?;
                        //%%%?:  claude said: .ok().map(|attr| Box::new(attr) as Box<dyn Attribute>)
                        Ok(Some(Box::new(ta)))
                    },
                    _ => {
                        Err(anyhow!("Unexpected variant of attributeDataHolder: {:?}", dh))
                    }
                }
            };
            let tadh = TextAttributeDH {
                attr_type_id,
                valid_on_date: None,
                //%%%%%?:
                observation_date: chrono::Local::now().timestamp_millis(),
                text: String::new(),
            };
            let mut tadh_variant = AttributeDataHolder::TextAttributeDH { tadh };
            let fn_aftat: fn(&Controller, Rc<RefCell<dyn Database>>, &mut AttributeDataHolder, bool, &TextUI) -> Result<Option<AttributeDataHolder>, anyhow::Error> = Util::ask_for_text_attribute_text;
            let ta = self.ask_for_info_and_add_attribute(
                entity_in.get_db(),
                &mut tadh_variant,
                ask_for_attr_type_id,
                Util::TEXT_TYPE,
                Some(&format!("SELECT TYPE OF {}: ", Util::TEXT_DESCRIPTION).as_str()),
                entity_in,
                //%% Util::ask_for_text_attribute_text,
                fn_aftat,
                add_text_attribute,
            )?;
            Ok(ta)
        } else if attr_form_in
            == self.db.borrow().get_attribute_form_id(Util::RELATION_TO_LOCAL_ENTITY_TYPE)?
        {
            //(This is in a condition that says "...LOCAL..." but is also for
            //RELATION_TO_REMOTE_ENTITY_TYPE.  See caller for details if interested.)
            let add_relation_to_entity =
                |dh: &mut AttributeDataHolder, entity_in: &Entity| -> Result<Option<Box<dyn Attribute>>, anyhow::Error> {
                    match dh {
                        AttributeDataHolder::RelationToEntityDH { rtedh } => {
                            let relation: Box<dyn Attribute> = if rtedh.is_remote {
                                return Err(anyhow!("%%Not yet implemented (see entity.rs ~1230 fn add_relation_to_remote_entity)."))
                                // entity_in.add_relation_to_remote_entity(
                                //     rtedh.rel_type_id,
                                //     rtedh.entity_id2,
                                //     None,
                                //     rtedh.valid_on_date,
                                //     rtedh.observation_date,
                                //     &rtedh.remote_instance_id,
                                //     //%%%? claude said:
                                //     //).ok().map(|attr| Box::new(attr) as Box<dyn Attribute>)
                                // )?
                            } else {
                                entity_in.add_relation_to_local_entity(
                                    None,
                                    rtedh.rel_type_id,
                                    rtedh.entity_id2,
                                    None,
                                    rtedh.valid_on_date,
                                    rtedh.observation_date,
                                    //%%%? claude said:
                                    //).ok'.().map(|attr| Box::new(attr) as Box<dyn Attribute>)
                                )
                            }?;
                            Ok(Some(relation))
                        },
                        _ => {
                            Err(anyhow!("Unexpected variant of attributeDataHolder: {:?}", dh))
                        }
                    }
                };
            let rtedh = RelationToEntityDH {
                rel_type_id: attr_type_id,
                valid_on_date: None,
                //%%%?:
                observation_date: chrono::Local::now().timestamp_millis(),
                entity_id2: 0,
                is_remote: false,
                remote_instance_id: String::new(),
            };
            let mut rtedh_variant = AttributeDataHolder::RelationToEntityDH { rtedh };
            let fn_afreid2: fn(&Controller, Rc<RefCell<dyn Database>>, &mut AttributeDataHolder, bool, &TextUI) -> Result<Option<AttributeDataHolder>, anyhow::Error> = Self::ask_for_relation_entity_id_number2;
            let a = self.ask_for_info_and_add_attribute(
                entity_in.get_db(),
                &mut rtedh_variant,
                ask_for_attr_type_id,
                Util::RELATION_TYPE_TYPE,
                Some(&format!(
                    "CREATE OR SELECT RELATION TYPE: ({})",
                    Util::REL_TYPE_EXAMPLES
                )),
                entity_in,
                Self::ask_for_relation_entity_id_number2,
                add_relation_to_entity,
            )?;
            Ok(a)
        } else if attr_form_in == 100 {
            // re "100": see doc comments above re attr_form_in
            let e_id: Option<IdWrapper> =
                self.ask_for_name_and_search_for_entity(entity_in.get_db())?;
            if let Some(id) = e_id {
                //%%%%right time thing?:
                let rtle = entity_in.add_has_relation_to_local_entity(
                    None,
                    id.get_id(),
                    None,
                    chrono::Local::now().timestamp_millis(),
                )?;
                //%%%%.ok().map(|attr| Box::new(attr) as Box<dyn Attribute>)
                Ok(Some(Box::new(rtle)))
            } else {
                Ok(None)
            }
        } else if attr_form_in == self.db.borrow().get_attribute_form_id(Util::RELATION_TO_GROUP_TYPE)? {
            let add_relation_to_group =
                |dh: &mut AttributeDataHolder, entity_in: &Entity| -> Result<Option<Box<dyn Attribute>>, anyhow::Error> {
                    match dh {
                        AttributeDataHolder::RelationToGroupDH { rtgdh } => {
                            assert_eq!(rtgdh.entity_id, entity_in.get_id());
                            let new_rtg: RelationToGroup = entity_in.add_relation_to_group(
                                None,
                                rtgdh.rel_type_id,
                                rtgdh.group_id,
                                None,
                            )?;
                            //%%%%%??: .ok().map(|attr| Box::new(attr) as Box<dyn Attribute>)
                            Ok(Some(Box::new(new_rtg)))
                        },
                        _ => {
                            Err(anyhow!("Unexpected variant of attributeDataHolder: {:?}", dh))
                        }
                    }
            };
            let rtgdh = RelationToGroupDH {
                    entity_id: entity_in.get_id(),
                    rel_type_id: attr_type_id,
                    group_id: 0,
                    valid_on_date: None,
                    //%%%%%?:
                    observation_date: chrono::Local::now().timestamp_millis(),
            };
            let mut rtgdh_variant = AttributeDataHolder::RelationToGroupDH { rtgdh };
            let fn_afrtgi: fn(&Controller, Rc<RefCell<dyn Database>>, &mut AttributeDataHolder, bool, &TextUI) -> Result<Option<AttributeDataHolder>, anyhow::Error> = Self::ask_for_rel_to_group_info;
            let result: Option<Box<dyn Attribute>> = self.ask_for_info_and_add_attribute(
                entity_in.get_db(),
                &mut rtgdh_variant,
                ask_for_attr_type_id,
                Util::RELATION_TYPE_TYPE,
                Some(&format!(
                    "CREATE OR SELECT RELATION TYPE: ({}).\n(Does anyone see a specific reason to keep asking for these dates?)",
                    Util::REL_TYPE_EXAMPLES
                )),
                entity_in,
                //%% Self::ask_for_rel_to_group_info,
                fn_afrtgi,
                add_relation_to_group,
            )?;
            if let Some(ref attr) = result {
                //%%%?: [keep the part of the cmt so I can search for "as_any" in future to find this.
                // let new_rtg = attr.as_any().downcast_ref::<RelationToGroup>().unwrap();
                let new_rtg_option = (attr as &dyn Any).downcast_ref::<RelationToGroup>();
                let Some(new_rtg) = new_rtg_option else {
                    return Err(anyhow!("unexpected result from downcast_ref: {:?}", new_rtg_option));
                };
                return Err(anyhow!("unimplemented")); //%%
                // QuickGroupMenu::new(self.ui.clone(), Rc::new(self.clone())).quick_group_menu(
                //     &Group::new2(entity_in.get_db(), None, new_rtg.get_group_id()),
                //     0,
                //     Some(new_rtg),
                //     None,
                //     None,
                //     None,
                //     Some(entity_in), /*%%%%containing_entity_in parm*/
                // );
                // // user could have deleted the new result: check that before returning it as something to act upon:
                // if entity_in
                //     .get_db()
                //     .borrow()
                //     .relation_to_group_key_exists(None, new_rtg.get_id())?
                // {
                //     Ok(result)
                // } else {
                //     Ok(None)
                // }
            } else {
                Ok(None)
            }
        } else if attr_form_in == 101 {
            //re "101": an "external web page"; for details see comments etc at javadoc above for attr_form_in.)
            let new_entity_name: Option<String> = self.ui.ask_for_string1(vec![
                "Enter a name (or description) for this web page or other URI",
            ]);

            let Some(new_entity_name) = new_entity_name else {
                return Ok(None);
            };
            //%% if new_entity_name.is_none()
            //     || new_entity_name /*.as_ref()*/
            //         .unwrap()
            //         .is_empty()
            // {
            //     return Ok(None);
            // }
            
            let ans1 = self.ui.ask_which(
                Some(vec![
                    "Do you want to enter the URI via the keyboard (typing or directly pasting), or \
                    have OM pull directly from the clipboard (faster sometimes)?"
                ]),
                &vec!["keyboard".to_string(), "clipboard".to_string()],
                &Vec::<String>::new(),
                true,
                None,
                None,
                None,
                None
            );
            if ans1.is_none() {
                return Ok(None);
            }
            let keyboard_or_clipboard1 = ans1.unwrap();
            let uri: String = if keyboard_or_clipboard1 == 1 {
                let text = self.ui.ask_for_string1(vec!["Enter the URI:"]);
                match text {
                    None => {
                        return Ok(None);
                    },
                    Some(t) => {
                        if t.is_empty() {
                            return Ok(None);
                        } else {
                           t
                        }
                    },
                }
                //%%delete--above should be same but compile.
                // if text.is_none()
                //     || text /*.as_ref()*/
                //         .unwrap()
                //         .is_empty()
                // {
                //     return Ok(None);
                // } else {
                //     text.unwrap()
                // }
            } else {
                let uri_ready = self.ui.ask_yes_no_question(
                    "Put the url on the system clipboard, then Enter to continue (or hit ESC or answer 'n' to get out)",
                    "y",
                    false,
                );
                if uri_ready.is_none() || !uri_ready.unwrap() {
                    return Ok(None);
                }
                Util::get_clipboard_content()
            };
            let ans2 = self.ui.ask_which(
                Some(vec![
                    "Do you want to enter a quote from it, via the keyboard (typing or directly pasting) or \
                    have OM pull directly from the clipboard (faster sometimes, especially if \
                    it's multi-line)? Or, ESC to not enter a quote. (Tip: if it is a whole file, just put in \
                    a few characters from the keyboard, then go back and edit as multi-line to put in all.)"
                ]),
                &vec!["keyboard".to_string(), "clipboard".to_string()],
                &Vec::<String>::new(),
                true,
                None,
                None,
                None,
                None
            );
            let quote = if ans2.is_none() {
                None
            } else {
                let keyboard_or_clipboard2 = ans2.unwrap();
                if keyboard_or_clipboard2 == 1 {
                    let text = self.ui.ask_for_string1(vec!["Enter the quote"]);
                    if text.is_none()
                        || text.as_ref() /*%%& ~2 abovesimilar?.as_ref()*/
                            .unwrap()
                            .is_empty()
                    {
                        return Ok(None);
                    }
                    text
                } else {
                    let clip = self.ui.ask_yes_no_question(
                        "Put a quote on the system clipboard, then Enter to continue (or answer 'n' to get out)",
                        "y",
                        false,
                    );
                    if clip.is_none() || !clip.unwrap() {
                        return Ok(None);
                    }
                    Some(Util::get_clipboard_content())
                }
            };
            let quote_info = if let Some(ref q) = quote {
                format!("For this text: \n  {}\n...and, ", q)
            } else {
                String::new()
            };
            let proceed_answer = self.ui.ask_yes_no_question(
                &format!(
                    "{}...for this name & URI:\n  {}\n  {}\n...: do you want to save them?",
                    quote_info,
                    new_entity_name,
                    uri
                ),
                "y",
                false,
            );
            if proceed_answer.is_none() || !proceed_answer.unwrap() {
                return Ok(None);
            }
            let is_public = entity_in.get_public(None)?;
            //NOTE: the attr_type_id parm is ignored here since it is always a particular one for URIs:
            let (new_entity, new_rte): (Entity, RelationToLocalEntity) = entity_in
                .add_uri_entity_with_uri_attribute(
                    None,
                    new_entity_name,
                    &uri,
                    //%%?:
                    chrono::Local::now().timestamp_millis(),
                    is_public,
                    quote.as_deref()
                )?; /*%%?.unwrap();*/

            //%%:
            // EntityMenu::new(self.ui.clone(), Rc::new(self.clone()))
            //     .entity_menu(&new_entity, /*containingRteIn=*/ Some(&new_rte));
            self.ui.display_text2("EntityMenu not yet converted to Rust.", true);


            // user could have deleted the new result: check that before returning it as something to act upon:
            if entity_in
                .get_db()
                .borrow()
                .relation_to_local_entity_key_exists(None, new_rte.get_id())?
                && entity_in
                    .get_db()
                    .borrow()
                    .entity_key_exists(None, new_entity.get_id(), true)?
            {
                //%%%?:
                Ok(Some(Box::new(new_rte) as Box<dyn Attribute>))
            } else {
                Ok(None)
            }
        } else {
            self.ui.display_text1("invalid response");
            Ok(None)
        }
    }
}
