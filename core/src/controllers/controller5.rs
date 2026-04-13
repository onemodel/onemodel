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
    // Helper method for choose_or_create_object
    pub fn get_lead_text_and_object_list(
        &self,
        //choices_in: &Vec<String>,
        choices_in: Vec<String>,
        object_type_in: &str,
        leading_text_in: &Option<Vec<&str>>,
        num_objects_available: u64,
        starting_display_row_index_in: u64,
        show_only_attribute_types: bool,
        quantity_seeks_unit_not_type_in: bool,
        class_id_in: Option<i64>,
        limit_by_class_in: bool,
        previous_selection_id_in: Option<i64>,
        containing_group_in: Option<i64>,
    ) -> Result<(
        Vec<String>,
        //%%in scala, this type was:
        //java.util.ArrayList[_ >: RelationType with OmInstance with EntityClass <: Object],
        Vec<Rc<RefCell<dyn HasId>>>,
        Vec<String>,
    ), anyhow::Error>  {
        let prefix = match object_type_in {
            Util::ENTITY_TYPE => "ENTITIES: ",
            Util::QUANTITY_TYPE => "QUANTITIES (entities): ",
            Util::DATE_TYPE => "DATE ATTRIBUTES (entities): ",
            Util::BOOLEAN_TYPE => "TRUE/FALSE ATTRIBUTES (entities): ",
            Util::FILE_TYPE => "FILE ATTRIBUTES (entities): ",
            Util::TEXT_TYPE => "TEXT ATTRIBUTES (entities): ",
            Util::RELATION_TYPE_TYPE => "RELATION TYPES: ",
            Util::RELATION_TO_LOCAL_ENTITY_TYPE => "RELATION TYPES: ",
            Util::RELATION_TO_GROUP_TYPE => "RELATION TYPES: ",
            Util::ENTITY_CLASS_TYPE => "CLASSES: ",
            Util::OM_INSTANCE_TYPE => "OneModel INSTANCES: ",
            _ => "",
        };
        let mut leading_text = leading_text_in.as_ref()
            .map(|v| v.iter().map(|s| s.to_string()).collect())
            .unwrap_or_else(|| {
                vec![format!(
                    "{}Pick from menu, or an item by letter; Alt+<letter> \
                    to go to the item & later come back)",
                    prefix
                )]
            });
        let num_displayable_items = self.ui.max_columnar_choices_to_display_after(
            // up to: see more of leading_text below.
            leading_text.len() + 3,
            choices_in.len(),
            Util::max_name_length(),
        )?;
        //%%del: let num_displayable_items: u64 = u64::try_from(num_displayable_items)?;
        let objects_to_display: Vec<Rc<RefCell<dyn HasId>>> = {
            // * KEEP THESE QUERIES AND CONDITIONS IN SYNC W/ THE COROLLARY ONES 1x ELSEWHERE ! (at similar comment):
            if Util::NON_RELATION_ATTR_TYPE_NAMES.contains(&object_type_in) {
                if show_only_attribute_types {
                    //%%cmted code repl w that just below.
                    // self.db
                    //  .borrow()
                    //     .get_entities_used_as_attribute_types(
                    //         self.db.clone(),
                    //         None,
                    //         object_type_in,
                    //         starting_display_row_index_in,
                    //         quantity_seeks_unit_not_type_in,
                    //         Some(num_displayable_items),
                    //     )?
                    //     .into_iter()
                    //     .map(|e| Box::new(e) as Box<&mut dyn HasId>)
                    //     .collect() //%%??
                    let entries = self.db
                        .borrow()
                        .get_entities_used_as_attribute_types(
                            self.db.clone(),
                            None,
                            object_type_in,
                            starting_display_row_index_in,
                            quantity_seeks_unit_not_type_in,
                            Some(num_displayable_items),
                        )?;
                    let mut objects: Vec<Rc<RefCell<dyn HasId>>> = Vec::new();
                    for e in entries {
                        let x = Rc::new(RefCell::new(e));
                        //%% let mut hi = e as dyn HasId;
                        let hi = x as Rc<RefCell<dyn HasId>>;
                        objects.push(hi);
                    };
                    objects
                } else {
                    let entries = self.db
                        .borrow()
                        .get_entities(
                            self.db.clone(),
                            None,
                            starting_display_row_index_in,
                            Some(num_displayable_items),
                        )?;
                    let mut objects: Vec<Rc<RefCell<dyn HasId>>> = Vec::new();
                    for e in entries {
                        let x = Rc::new(RefCell::new(e));
                        let hi = x as Rc<RefCell<dyn HasId>>;
                        objects.push(hi);
                    };
                    objects
                }
            } else if object_type_in == Util::ENTITY_TYPE {
                let entries = self.db
                    .borrow()
                    .get_entities_only(
                        self.db.clone(),
                        None,
                        starting_display_row_index_in,
                        Some(num_displayable_items),
                        class_id_in,
                        limit_by_class_in,
                        previous_selection_id_in,
                        containing_group_in,
                    )?;
                    let mut objects: Vec<Rc<RefCell<dyn HasId>>> = Vec::new();
                    for e in entries {
                        let x = Rc::new(RefCell::new(e));
                        let hi = x as Rc<RefCell<dyn HasId>>;
                        objects.push(hi);
                    };
                    objects
            } else if Util::RELATION_ATTR_TYPE_NAMES.contains(&object_type_in) {
                let entries = self.db
                    .borrow()
                    .get_relation_types(
                        self.db.clone(),
                        None,
                        starting_display_row_index_in,
                        Some(num_displayable_items),
                    )?;
                    let mut objects: Vec<Rc<RefCell<dyn HasId>>> = Vec::new();
                    for e in entries {
                        let x = Rc::new(RefCell::new(e));
                        let hi = x as Rc<RefCell<dyn HasId>>;
                        objects.push(hi);
                    };
                    objects
            } else if object_type_in == Util::ENTITY_CLASS_TYPE {
                let entries = self.db
                    .borrow()
                    .get_classes(
                        self.db.clone(),
                        None,
                        starting_display_row_index_in,
                        Some(num_displayable_items),
                    )?;
                    let mut objects: Vec<Rc<RefCell<dyn HasId>>> = Vec::new();
                    for e in entries {
                        let x = Rc::new(RefCell::new(e));
                        let hi = x as Rc<RefCell<dyn HasId>>;
                        objects.push(hi);
                    };
                    objects
            } else if object_type_in == Util::OM_INSTANCE_TYPE {
                return Err(anyhow!("not yet implemented"));
                //%%
                //self.db
                //    .borrow()
                //    .get_om_instances()
                //    .into_iter()
                //    .map(|i| Box::new(i) as Box<dyn HasId>)
                //    .collect() //%%??
            } else {
                return Err(anyhow!("invalid object_type_in: {}", object_type_in));
            }
        };
        if objects_to_display.is_empty() {
            // IF THIS CHANGES: change the guess at the 1st parameter to max_columnar_choices_to_display_after, JUST ABOVE!
            let txt = format!(
                "\n\n(None of the needed {} have been created in this model, yet.",
                if object_type_in == Util::RELATION_TYPE_TYPE {
                    "relation types"
                } else {
                    "entities"
                }
            );
            leading_text.push(txt);
        }
        /*%%?let choices_with_count = */
        Util::add_remaining_count_to_prompt(
            choices_in,
            u64::try_from(objects_to_display.len())?,
            num_objects_available,
            starting_display_row_index_in,
        )?;
        let mut object_statuses_and_names: Vec<String> = Vec::new();
        //%%why not?: 
        //for obj in objects_to_display {
            //let obj = &obj;
        for i in 0..objects_to_display.len() {
            let mut obj_opt = /*%%&*/objects_to_display.get(i);
            let Some(obj) = obj_opt else {
                return Err(anyhow!("Unexpected value None for object_to_display.get(i) for i={}", i));
            };
            // let obj = obj_opt.unwrap();
            //%%.iter()
            //%%.map(|obj| {
                //%%if let Some(entity) = obj.as_any().downcast_ref::<Entity>() {
                let obj/*%%: &mut dyn HasId*/ = &mut obj.as_ref().borrow_mut();
                let obj_any = obj as &mut dyn Any;
                if object_type_in == Util::ENTITY_TYPE {
                    let entity_opt = obj_any.downcast_ref::<Entity>();
                    if entity_opt.is_none() {
                        return Err(anyhow!("Unexpected None from downcast_ref of entity_opt."));
                    };
                    let mut entity = entity_opt.unwrap();
                    let astat = entity.get_archived_status_display_string(None)?;
                    let s = format!(
                        "{}{}",
                        astat,
                        entity.get_name(None).unwrap_or_default()
                    );
                    object_statuses_and_names.push(s);
                //%%} else if let Some(class) = obj.as_any().downcast_ref::<EntityClass>() {
                } else if object_type_in == Util::ENTITY_CLASS_TYPE {
                    let class_opt = obj_any.downcast_ref::<EntityClass>();
                    if class_opt.is_none() {
                        return Err(anyhow!("Unexpected None from downcast_ref of class_opt."));
                    };
                    let mut class: &EntityClass = class_opt.unwrap();
                    object_statuses_and_names.push(class.get_name(None)?);
                //%%} else if let Some(instance) = obj.as_any().downcast_ref::<OmInstance>() {
                } else if object_type_in == Util::OM_INSTANCE_TYPE {
                    let instance_opt = obj_any.downcast_mut::<OmInstance>();
                    let Some(&mut instance) = instance_opt else {
                        return Err(anyhow!("Unexpected None from downcast_ref of instance_opt."));
                    };
                    object_statuses_and_names.push(instance.get_display_string()?);
                } else {
                    //%%%ret err instd, **AND**? at similar places that panic here:
                    //%%panic!("unexpected class: {}", obj.get_class_name())
                    return Err(anyhow!("unexpected object_type_in: {}", object_type_in));
                }
            } //%%)
            //%%.collect();
        Ok((leading_text, objects_to_display, object_statuses_and_names))
    }

    // Helper method for choose_or_create_object
    pub(crate) fn get_next_starting_object_index(
        &self,
        starting_display_row_index_in: u64,
        previous_list_length_in: usize,
        num_objects_available_in: u64,
    ) -> u64 {
        //%%// (lets just make sure we aren't going to overflow when adding)
        //let starting_display_row_index: u32 = u32::try_from(starting_display_row_index_in).unwrap();
        let previous_list_length: u64 = u64::try_from(previous_list_length_in).unwrap();
        //let x: usize = usize::try_from(starting_display_row_index).unwrap() + usize::try_from(previous_list_length).unwrap();
        let x = starting_display_row_index_in
            .checked_add(previous_list_length)
            .unwrap();
        // Ask Model for list of obj's w/ count desired & starting index (or "first") (in a sorted map, w/
        // id's as key, and names).
        //idea: should this just reuse the "totalExisting" value alr calculated in
        //get_lead_text_and_object_list just above?
        if x >= num_objects_available_in {
            self.ui
                .display_text1("End of list found; starting over from the beginning.");
            0
        } else {
            x
        }
    }

    //%% 
    /*
    // Helper method for choose_or_create_object
    fn handle_link_to_remote_instance(
        &self,
        db_in: Rc<RefCell<dyn Database>>,
    ) -> Result<Option<(IdWrapper, bool, String)>, anyhow::Error> {
        let om_instance_id_option: Option<(_, _, String)> = self.choose_or_create_object(
            db_in.clone(),
            None,
            None,
            None,
            Util::OM_INSTANCE_TYPE,
            0,
            None,
            false,
            None,
            false,
            None,
            false,
        )?;
        let Some(om_instance_id_tuple) = om_instance_id_option else {
            return Ok(None);
        };
        //if om_instance_id_option.is_none() {
        //    return None;
        //}
        //%%next 2 lines in scala were this one:
        //let remoteOmInstance = new OmInstance(db_in, omInstanceIdOption.get._3);
        let instance_id = om_instance_id_tuple.2;
        let remote_om_instance = OmInstance::new2(db_in.clone(), None, instance_id)?;
        let remote_entity_entry_type_answer = self.ui.ask_which(
            Some(vec!["SPECIFY AN ENTITY IN THE REMOTE INSTANCE"]),
            vec![
                "Enter an entity id #",
                "Use the remote site's default entity",
            ],
            Vec::<&str>::new(),
            true,
            None,
            None,
            None,
            None,
        );
        if remote_entity_entry_type_answer.is_none() {
            return Ok(None);
        }
        let rest_db = Database::get_rest_database(&remote_om_instance.get_address());
        let remote_entity_id = match remote_entity_entry_type_answer.unwrap() {
            1 => {
                let remote_entity_answer = self.ui.ask_for_string2(
                    Some(vec![
                        "Enter the remote entity's id # (for example, \"-9223372036854745151\")",
                    ]),
                    Some(Util::is_numeric),
                    None,
                );
                //%%and_then?:
                remote_entity_answer.and_then(|s| {
                    let id = s.trim();
                    if id.is_empty() {
                        None
                    } else {
                        //%%unwrap()?:
                        Some(id.parse::<i64>().unwrap())
                    }
                })
            }
            2 => rest_db.get_default_entity(Some(&self.ui)),
            _ => None,
        };
        if remote_entity_id.is_none() {
            return None;
        }
        let entity_in_json: Option<String> = rest_db
            .get_entity_json_with_optional_err_handling(Some(&self.ui), remote_entity_id.unwrap());
        if entity_in_json.is_none() {
            return None;
        }
        let save_entity_answer = self.ui.ask_yes_no_question(
            &format!(
                "Here is the entity's data: \n======================\n{}\n======================\n\
                So do you want to save a reference to that entity?",
                entity_in_json.unwrap()
            ),
            Some("y"),
            false,
        );
        if save_entity_answer.is_some() && save_entity_answer.unwrap() {
            Some((
                IdWrapper::new(remote_entity_id.unwrap()),
                true,
                remote_om_instance.get_id(),
            ))
        } else {
            None
        }
    }
    */

    // Helper method for choose_or_create_object
    pub(crate) fn show_journal(&self, db_in: Rc<RefCell<dyn Database>>)
    /*(Scala code returned None, so now () or a Result.)*/
    -> Result<(), anyhow::Error> {
        // THIS IS CRUDE RIGHT NOW AND DOESN'T ABSTRACT TEXT SCREEN OUTPUT INTO THE UI CLASS
        // very neatly perhaps, BUT IS HELPFUL ANYWAY:
        // Ideas:
        // - move the lines for this little section, into a separate method, near findExistingObjectByName
        // - do something similar (refactoring findExistingObjectByName?) to show the results in a list,
        //   but make clear on *each line* what kind of result it is.
        // - where going to each letter w/ Alt key does the same: goes 2 that entity so one can see its context, etc.
        // - change the "None" returned to be the selected entity, like the little section above does.
        // - could keep this text output as an option?
        //%%%?: next 2 lines in scala were:
        //let yDate = new java.util.Date(System.currentTimeMillis() - (24 * 60 * 60 * 1000));
        //let yesterday: String = new java.text.SimpleDateFormat("yyyy-MM-dd").format(yDate);
        let yesterday = chrono::Local::now() - chrono::Duration::days(1);
        //%%%?:
        let yesterday_str = yesterday.format("%Y-%m-%d").to_string();
        let begin_date = Util::ask_for_date_generic(
            Some(&format!(
                "BEGINNING date in the time range: {}",
                Util::GENERIC_DATE_PROMPT
            )),
            Some(&yesterday_str),
            &self.ui,
        );
        let Some(begin_date) = begin_date else {
            //user wants out
            return Ok(());
        };
        let end_date: Option<i64> = Util::ask_for_date_generic(
            Some(&format!(
                "ENDING date in the time range: {}",
                Util::GENERIC_DATE_PROMPT
            )),
            None,
            &self.ui,
        );
        let Some(end_date) = end_date else {
            //user wants out
            return Ok(());
        };
        let mut day_currently_showing = String::new();
        let results: Vec<(i64, String, i64)> = db_in
            .borrow()
            .find_journal_entries(None, begin_date, end_date, None)?;
        for (timestamp, name, id) in results {
            //%%?: scala line was:  (ck case of & format string!)
            //let date = new java.text.SimpleDateFormat("yyyy-MM-dd").format(result._1);
            let date = chrono::NaiveDateTime::from_timestamp_opt(timestamp / 1000, 0);
            let Some(date) = date else {
                return Err(anyhow!("Unexpected None from from_timestamp_opt({} / 1000, 0).", timestamp));
            };
            let date: String = date.format("%Y-%m-%d").to_string();
            if day_currently_showing != date {
                println!("\n\nFor: {}------------------", date);
                day_currently_showing = date;
            }
            //%%?: scala line was:  (ck case of & format string!)
            //let time: String = new java.text.SimpleDateFormat("HH:mm:ss").format(result._1);
            let time = chrono::NaiveDateTime::from_timestamp_opt(timestamp / 1000, 0);
            let Some(time) = time else {
                return Err(anyhow!("Unexpected None from from_timestamp_opt({} / 1000, 0).", timestamp));
            };
            let time = time.format("%H:%M:%S").to_string();
            println!("{} {}: {}", time, id, name);
        }
        //%%?: in scala, this was "ui.out.println(....)"
        println!(
            "\n(For other ~'journal' info, could see other things for the day in question, like email, code \
            commits, or entries made on a \
            different day in a specific \"journal\" section of OM.)"
        );
        self.ui
            .display_text1("Scroll back to see more info if needed. Press any key to continue...");
        Ok(())
    }

    pub fn ask_for_name_and_search_for_entity(
        &self,
        db_in: Rc<RefCell<dyn Database>>,
    ) -> Result<Option<IdWrapper>, anyhow::Error> {
        let ans =
            self.ui
                .ask_for_string1(vec![Util::entity_or_group_name_sql_search_prompt(
                    Util::ENTITY_TYPE,
                ).as_str()]);
        let Some(answer) = ans else { return Ok(None) };
        let e: Option<IdWrapper> =
            self.find_existing_object_by_text(db_in, 0, Util::ENTITY_TYPE, None, &answer)?;
        let Some(x) = e else { return Ok(None) };
        return Ok(Some(IdWrapper::new(x.get_id())));
    }

    pub(crate) fn search_by_id(
        &self,
        db_in: Rc<RefCell<dyn Database>>,
        type_name_in: &str,
    ) -> Result<Option<IdWrapper>, anyhow::Error> {
        assert!(type_name_in == Util::ENTITY_TYPE || type_name_in == Util::GROUP_TYPE);
        let ans = self.ui.ask_for_string1(vec![&format!(
            "Enter the {} ID to search for:",
            type_name_in
        ).as_str()]);
        let Some(id_string) = ans else {
            return Ok(None);
        };
        if Util::is_numeric(&id_string).is_err() {
            self.ui.display_text1(&format!(
                "Invalid ID format. An ID is a numeric value from {} to {}",
                db_in.borrow().min_id_value(),
                db_in.borrow().max_id_value()
            ));
            return Ok(None);
        }
        let id = id_string.parse::<i64>().unwrap();
        let exists = if type_name_in == Util::ENTITY_TYPE {
            db_in.borrow().entity_key_exists(None, id, true)?
        } else {
            db_in.borrow().group_key_exists(None, id)?
        };
        if exists {
            Ok(Some(IdWrapper::new(id)))
        } else {
            self.ui.display_text1(&format!(
                "The {} ID {} was not found in the database.",
                type_name_in, id
            ));
            Ok(None)
        }
    }

    //%%verify that such doc comments wk as expected, or?:
    /// Returns None if user wants to cancel. 
    pub fn ask_for_quantity_attribute_number_and_unit(
        &self,
        db_in: Rc<RefCell<dyn Database>>,
        //%%mut dh_in: AttributeDataHolder::QuantityAttributeDH,
        dhv_in: &mut AttributeDataHolder,
        editing_in: bool,
        ui: &TextUI,
    ) -> Result<Option<AttributeDataHolder/*::QuantityAttributeDH*/>, anyhow::Error> {
    // ) -> Result<Option<QuantityAttributeDH>, anyhow::Error> {
        let dh_in: &mut QuantityAttributeDH = match dhv_in {
            AttributeDataHolder::QuantityAttributeDH { qadh } => qadh,
            _ => {
                return Err(anyhow!("Unexpected type for dh_in: {:?}.", dhv_in));
            },
        };
        let leading_text = vec!["SELECT A *UNIT* FOR THIS QUANTITY (i.e., centimeters, or quarts; ESC or blank to cancel):"];
        let previous_selection_desc = if editing_in {
            let mut e = Entity::new2(db_in.clone(), None, dh_in.unit_id)?;
            Some(e.get_name(None)?)
        } else {
            None
        };
        let previous_selection_id = if editing_in {
            Some(dh_in.unit_id)
        } else {
            None
        };
        let unit_selection: Option<(IdWrapper, _, _)> = self.choose_or_create_object(
            db_in.clone(), //%%?:.into(),
            Some(leading_text),
            previous_selection_desc, //%%?.as_deref(),
            previous_selection_id,
            Util::QUANTITY_TYPE,
            0,
            None,
            false,
            None,
            false,
            None,
            true,
        )?;
        if unit_selection.is_none() {
            ui.display_text2(
                "Blank, so assuming you want to cancel; if not come back & add again.",
                false,
            );
            return Ok(None);
        }
        let (id_wrapper, _, _) = unit_selection.unwrap();

        let mut qadh = QuantityAttributeDH {
            observation_date: dh_in.observation_date,
            valid_on_date: dh_in.valid_on_date,
            attr_type_id: dh_in.attr_type_id,
            unit_id: 0,
            number: 0.0,
        };
        qadh.unit_id = id_wrapper.get_id();
        let ans = Util::ask_for_quantity_attribute_number(dh_in.number, ui);
        if ans.is_none() {
            Ok(None)
        } else {
            qadh.number = ans.unwrap();
            let dhv_out = AttributeDataHolder::QuantityAttributeDH { qadh: qadh };
            Ok(Some(dhv_out))
        }
    }

    /// Returns None if user wants to cancel. 
    pub fn ask_for_rel_to_group_info(
        &self,
        db_in: Rc<RefCell<dyn Database>>,
        //%% mut dh_in: RelationToGroupDH,
        dh_in: &mut AttributeDataHolder,
        _editing_in: bool, /*false*/
        ui_in: &TextUI,
    // ) -> Result<Option<RelationToGroupDH>, anyhow::Error> {
    ) -> Result<Option<AttributeDataHolder>, anyhow::Error> {
        let rtgdh_in = match dh_in {
            AttributeDataHolder::RelationToGroupDH { rtgdh } => rtgdh,
            _ => return Err(anyhow!("unexpected enum variant {:?}.", dh_in)),
        };
        let group_selection = self.choose_or_create_group(
            db_in, /*%%?.into()*/
            Some(vec!["SELECT GROUP FOR THIS RELATION".to_string()]),
            0,
            None,
        )?;
        let group_id: Option<i64> = {
            if let Some(group_sel) = group_selection {
                Some(group_sel.get_id())
            } else {
                ui_in.display_text2(
                    "Blank, so assuming you want to cancel; if not come back & add again.",
                    false,
                );
                None
            }
        };
        let Some(gid) = group_id else {
            return Ok(None);
        };
        let rtgdh_out = RelationToGroupDH {
            valid_on_date: rtgdh_in.valid_on_date,
            observation_date: rtgdh_in.observation_date,
            rel_type_id: rtgdh_in.rel_type_id,
            group_id: gid, // the line that matters
            entity_id: rtgdh_in.entity_id,
        };
        let dh_out = AttributeDataHolder::RelationToGroupDH { rtgdh: rtgdh_out };
        Ok(Some(dh_out))
    }

    /// Helper method for choose_or_create_group.
    /// Current_list_length means how many per screenful are shown, out of a total of
    /// total_existing.
    fn get_next_starting_group_index(
        &self,
        current_list_length: u64,
        starting_display_row_index_in: u64,
        total_existing: u64,
    ) -> u64 {
        let x = starting_display_row_index_in + current_list_length;
        if x >= total_existing {
            self.ui
                .display_text1("End of list found; starting over from the beginning.");
            0
        } else {
            x
        }
    }

    /// Returns the id of a Group, or None if user wants out.
    /// The parameter 'containingGroupIn' lets us omit entities that are already in a group,
    /// i.e. omitting them from the list of entities (e.g. to add to the group), that this method returns.
    //%%@tailrec?
    fn choose_or_create_group(
        &self,
        db_in: Rc<RefCell<dyn Database>>,
        mut leading_text_in: Option<Vec<String>>,
        starting_display_row_index_in: u64, /*0*/
        containing_group_in: Option<i64>,   /*None*/
    ) -> Result<Option<IdWrapper>, anyhow::Error> {
        //%%or clone here? revu borrow() vs. clone().
        let total_existing: u64 = db_in.borrow().get_group_count(None)?;
        //%%let mut leading_text = leading_text_in
            //.map(|v| v.iter().map(|s| s.to_string()).collect())
            //.unwrap_or_else(|| vec![Util::PICK_FROM_LIST_PROMPT.to_string()]);
        let mut leading_text = match leading_text_in {
            Some(ref v) => v.clone(),
            None => vec![Util::PICK_FROM_LIST_PROMPT.to_string()],
        };
        let choices_pre_adjustment = vec![
            "List next items".to_string(),
            "Create new group (aka RelationToGroup)".to_string(),
            "Search for existing group by name...".to_string(),
            "Search for existing group by id...".to_string(),
        ];
        let num_displayable_items = self.ui.max_columnar_choices_to_display_after(
            leading_text.len(),
            choices_pre_adjustment.len(),
            Util::max_name_length(),
        )?;
        //%%del: let num_displayable_items: u64 = u64::try_from(num_displayable_items)?;
        //%%clone or borrow, here?:
        let mut objects_to_display: Vec<Group> = db_in.clone().borrow().get_groups(
            db_in.clone(),
            None,
            starting_display_row_index_in,
            Some(num_displayable_items),
            containing_group_in,
        )?;
        if objects_to_display.is_empty() {
            let txt = "\n\n(None of the needed groups have been created in this model, yet.";
            //%%leading_text.push(txt.to_string());
            leading_text.push(txt.to_string());
        }
        let choices = Util::add_remaining_count_to_prompt(
            choices_pre_adjustment,
            u64::try_from(objects_to_display.len())?,
            total_existing,
            starting_display_row_index_in,
        )?;
        //%%replaced due to need to handle Result
        //let object_names: Vec<String> = objects_to_display
        //    .iter()
        //    .map(|group| group.get_name(None)?)
        //    .collect();
        let mut object_names: Vec<String> = Vec::new();
        for g in objects_to_display.iter_mut() {
            object_names.push(g.get_name(None)?);
        }
        let mut choices_strs: Vec<String> = Vec::new();
        for s in choices.iter() {
            choices_strs.push(s.clone());
        }
        let ans = self.ui.ask_which_choice_or_its_alternate(
            Some(leading_text),
            &choices_strs/*%%.clone()*/,
            &object_names,
            true,
            None,
            None,
            None,
            None,
        );
        let Some((answer, user_chose_alternate)) = ans else {
            return Ok(None);
        };
        if answer == 1 && answer <= choices.len() {
            let next_starting_index: u64 =
                self.get_next_starting_group_index(num_displayable_items, starting_display_row_index_in,
                    u64::try_from(objects_to_display.len())?);
            self.choose_or_create_group(
                db_in,
                leading_text_in,
                next_starting_index,
                containing_group_in,
            )
        } else if answer == 2 && answer <= choices.len() {
            let ans = self
                .ui
                .ask_for_string1(vec![Util::RELATION_TO_GROUP_NAME_PROMPT]);
            if ans.is_none()
                || ans.as_ref()
                    .unwrap()
                    .trim()
                    .is_empty()
            {
                return Ok(None);
            }
            let name = ans.clone().unwrap();
            let ans2 = self.ui.ask_yes_no_question(
                "Should this group allow entities with mixed classes? (Usually not desirable: doing so means losing some \
                conveniences such as scripts and assisted data entry.)",
                "n",
                false,
            );
            if ans2.is_none() {
                return Ok(None);
            } else {
                let mixed_classes_allowed = ans2.unwrap();
                let new_group_id =
                    db_in.borrow/*%%_mut*/().create_group(None, &name, mixed_classes_allowed)?;
                Ok(Some(IdWrapper::new(new_group_id)))
            }
        } else if answer == 3 && answer <= choices.len() {
            let ans =
                self.ui
                    .ask_for_string1(vec![Util::entity_or_group_name_sql_search_prompt(
                        Util::GROUP_TYPE,
                    ).as_str()]);
            if let Some(a) = ans {
                let group: Option<IdWrapper> =
                    self.find_existing_object_by_text(db_in, 0, Util::GROUP_TYPE, None, &a)?;
                let Some(g) = group else {
                    return Ok(None);
                };
                Ok(Some(IdWrapper::new(g.get_id())))
            } else {
                Ok(None)
            }
        } else if answer == 4 && answer <= choices.len() {
            Ok(self.search_by_id(db_in, Util::GROUP_TYPE)?)
        } else if answer > choices.len() && answer <= (choices.len() + objects_to_display.len()) {
            // those in that^ condition are 1-based, not 0-based.
            let index = answer - choices.len() - 1;
            //%%use get() instd?
            let group = &objects_to_display[index];
            if user_chose_alternate {
                // for now, picking the first RTG found for this group, until it's clear which of its RTGs to use.
                // (see also the other locations w/ similar comment!)
                let some_relation_to_groups = group.get_containing_relations_to_group(None, 0, Some(1))?;
                //%%:
                return Err(anyhow!("unimplemented"));
                //%%simplify?:
                //GroupMenu::new(self.ui.clone(), Rc::new(self.clone())).group_menu(
                //    //%%use  get instead of [0], x2?
                //    &Group::new2(
                //        db_in.clone(),
                //        None,
                //        some_relation_to_groups[0].get_group_id(),
                //    ),
                //    0,
                //    Some(&some_relation_to_groups[0]),
                //    /*%%?: None,*/
                //);
                //self.choose_or_create_group(
                //    db_in,
                //    leading_text_in,
                //    starting_display_row_index_in,
                //    containing_group_in,
                //)
            } else {
                // user typed a letter to select.. (now 0-based); selected a new object and so we return to
                // the previous menu w/ that one displayed & current
                Ok(Some(IdWrapper::new(group.get_id())))
            }
        } else {
            self.ui
                .display_text1("unknown response in findExistingObjectByText");
            self.choose_or_create_group(
                db_in,
                leading_text_in,
                starting_display_row_index_in,
                containing_group_in,
            )
        }
    }

    /// Returns None if user wants to cancel. 
    pub fn ask_for_relation_entity_id_number2(
        &self,
        db_in: Rc<RefCell<dyn Database>>,
        //%% &mut dh_in: RelationToEntityDH,
        dh_in: &mut AttributeDataHolder,
        editing_in: bool,
        _ui_in: &TextUI,
    ) -> Result<Option<AttributeDataHolder>, anyhow::Error> {
        let rtedh_in: &mut RelationToEntityDH = match dh_in {
            AttributeDataHolder::RelationToEntityDH{ rtedh } => rtedh, //%%{ rel_type_id, valid_on_date, observation_date, entity_id2, is_remote, remote_instance_id }
            _ => return Err(anyhow!("Unexpected data holder enum variant: {:?}", dh_in)),
        };
        let mut e = Entity::new2(db_in.clone() /*%%%?.into()*/, None, rtedh_in.entity_id2)?;
        let s = e.get_name(None)?; /*%%%?:.unwrap_or_default()*/
        let previous_selection_desc: Option<String> = if editing_in {
            Some(s)
        } else {
            None
        };
        let previous_selection_id = if editing_in {
            Some(rtedh_in.entity_id2)
        } else {
            None
        };
        let selection: Option<(IdWrapper, bool, String)> = self.choose_or_create_object(
            db_in, /*%%%?.into()*/
            Some(vec!["SELECT OTHER (RELATED) ENTITY FOR THIS RELATION"]),
            previous_selection_desc, /*%%%.as_deref()*/
            previous_selection_id,
            Util::ENTITY_TYPE,
            0,
            None,
            false,
            None,
            false,
            None,
            false,
        )?;
        if let Some((id_wrapper, is_remote, remote_instance_id)) = selection {
            let rtedh_out = RelationToEntityDH {
                rel_type_id: rtedh_in.rel_type_id,
                valid_on_date: rtedh_in.valid_on_date,
                observation_date: rtedh_in.observation_date,
                entity_id2: id_wrapper.get_id(),
                is_remote: is_remote,
                remote_instance_id: remote_instance_id,
            };
            let adh_out = AttributeDataHolder::RelationToEntityDH { rtedh: rtedh_out };
            Ok(Some(adh_out))
        } else {
            Ok(None)
        }
    }
}
