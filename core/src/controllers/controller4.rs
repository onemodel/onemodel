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
use crate::model::attribute_with_valid_and_observed_dates::AttributeWithValidAndObservedDates;
use crate::model::boolean_attribute::BooleanAttribute;
use crate::model::date_attribute::DateAttribute;
use crate::model::entity_class::EntityClass;
use crate::model::file_attribute::FileAttribute;
use crate::model::group::Group;
use crate::model::id_wrapper::IdWrapper;
use crate::model::om_instance::OmInstance;
use crate::model::quantity_attribute::QuantityAttribute;
use crate::model::relation_to_group::RelationToGroup;
use crate::model::relation_to_entity::RelationToEntity;
use crate::model::relation_to_local_entity::RelationToLocalEntity;
use crate::model::relation_to_remote_entity::RelationToRemoteEntity;
use crate::model::relation_type::RelationType;
use crate::model::text_attribute::TextAttribute;
use anyhow::anyhow;
//use std::collections::HashMap;
//use std::fs::File;
use std::path::Path;

/// This Controller is for user-interactive things.  The Controller class in the web module
/// is for the REST API.  For shared code that does not fit
/// in those, see struct Util (in util.rs).
///
/// Improvements to this class should START WITH MAKING IT BETTER TESTED (functional testing? integration? see
/// scalatest docs 4 ideas, & maybe use expect or the gnu testing tool that uses expect?), delaying side effects more,
/// shorter methods, other better style?, etc.
///
/// * * * *IMPORTANT * * * * * IMPORTANT* * * * * * *IMPORTANT * * * * * * * IMPORTANT* * * * * * * * *IMPORTANT * * * * * *
/// Don't ever instantiate a Controller from a *test* without passing in username/password
/// parameters, because it will try to log in to the user's
/// default, live Database and run the tests there (ie, they could be destructive)!
/// %%: How make that better/safer/more certain!?--just use the new_* methods below as reminders?
/// * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * *
///
impl Controller {
    /*%%
    // SEE DESCRIPTIVE COMMENT ON ask_for_and_write_class_and_template_entity_name, WHICH APPLIES TO all
    // such METHODS (see this cmt elsewhere).
    /// Return The instance's id, or None if there was a problem or the user wants out.
    fn ask_for_and_write_om_instance_info(
        &self,
        db_in: Rc<RefCell<dyn Database>>,
        old_om_instance_in: Option<OmInstance>, /*= None*/
    ) -> Result<Option<String>, anyhow::Error> {
        let create_not_update = old_om_instance_in.is_none();
        let address_length = OmInstance::address_length();

        let ask_and_save_closure =
            |default_name: Option<String>| -> Result<Option<String>, anyhow::Error> {
                let prompt = vec![format!(
                "Enter the internet address with optional port of a remote OneModel instance (for \
                example, \"om.example.com:2345\", up to {} characters; ESC to cancel; \
                Other examples include (omit commas): localhost, 127.0.0.1:9000, ::1 (is this correct?), \
                my.example.com:80, your.example.com:8080 .): ",
                address_length
            ).as_str()];
                //let address_opt = self.ui.ask_for_string3(Some(prompt), None, default_name.as_deref());
                let address_opt = self.ui.ask_for_string3(prompt, None, default_name.unwrap_or("".to_string()).as_str());
                if address_opt.is_none() {
                    return Ok(None);
                }
                let address = address_opt.unwrap().trim().to_string();
                if address.is_empty() {
                    return Ok(None);
                }
                //let existing_id = old_om_instance_in.as_ref().map(|o| o.get_id());
                let existing_id: Option<String> = match old_om_instance_in {
                    Some(o) => Some(o.get_id()?),
                    None => None,
                };
                if Util::is_duplication_a_problem(
                    OmInstance::is_duplicate(&*db_in.borrow(), None, &address, existing_id)?,
                    false,
                    &self.ui,
                ) {
                    return Ok(None);
                }
                let rest_db = Database::get_rest_database(&address);
                let remote_id = rest_db.get_id_with_optional_err_handling(Some(&self.ui));
                if remote_id.is_none() {
                    return Ok(None);
                }
                let remote_id = remote_id.unwrap();
                if create_not_update {
                    OmInstance::create(db_in.borrow(), None, &remote_id, &address, None)?;
                    Ok(Some(remote_id))
                } else {
                    //let old_instance = old_om_instance_in.as_ref().unwrap(); //from claude
                    let old_instance = old_om_instance_in.unwrap();
                    if old_instance.get_id() == remote_id {
                        old_instance.update(None, &address)?;
                        Ok(Some(old_instance.get_id()))
                    } else {
                        let question = format!(
                        "The IDs of the old and new remote instances don't match (old id/address: {}/{}, \
                        new id/address: {}/{}. Instead of updating the old one, you should create a new \
                        entry for the new remote instance and then optionally delete this old one. \
                        Do you want to create the new entry with this new address, now?",
                        old_instance.get_id(),
                        old_instance.get_address(),
                        remote_id,
                        address
                    );
                        let ans = self.ui.ask_yes_no_question(&question, None, false);
                        if ans.is_some() && ans.unwrap() {
                            let id = OmInstance::create(
                                db_in.borrow(),
                                None,
                                &remote_id,
                                &address,
                                None,
                            )?
                            .get_id();
                            self.ui.display_text(&format!(
                            "Created the new entry for \"{}\". You still have to delete the old one ({}/{}) if \
                            you don't want it to be there.",
                            address,
                            old_instance.get_id(),
                            old_instance.get_address()
                        ));
                            Ok(Some(id))
                        } else {
                            Ok(None)
                        }
                    }
                }
            };
        //let default_address = old_om_instance_in.as_ref().map(|o| o.get_address());
        let default_address = old_om_instance_in.map(|o| o.get_address()?);
        self.try_asking_and_saving(
            db_in,
            &Util::string_too_long_error_message(address_length, ""),
            Self::ask_and_save,
            default_address,
            //type_in,
            //max_name_length,
            //example,
        )
        //%%remove if works w/o
        //tryAskingAndSaving[String](db_in, Util.string_too_long_error_message(address_length()), askAndSave,
        //                           if oldOmInstanceIn.isEmpty) {
        //                             None
        //                           } else {
        //                             Some(oldOmInstanceIn.get.get_address)
        //                           })
    }
    */

    /// This function is separate so it can call itself recursively.
    fn ask_for_info_and_update_attribute(
        &self,
        //%%remove: controller: &Controller,
        db_in: Rc<RefCell<dyn Database>>,
        attribute_in: &mut dyn Attribute,
        dh_in: &mut AttributeDataHolder,
        ask_for_attr_type_id: bool,
        attr_type: &str,
        prompt_for_type_id: &str,
        get_other_info_from_user: fn(
            &Controller,
            Rc<RefCell<dyn Database>>,
            &mut AttributeDataHolder,
            bool,
            &TextUI,
        ) -> Result<Option<AttributeDataHolder>, anyhow::Error>,
        update_typed_attribute: fn(&mut dyn Attribute, AttributeDataHolder) -> Result<(), anyhow::Error>,
    ) -> Result<bool, anyhow::Error>
//%%remove if works w/o:
    //where
    //    T: AttributeDataHolder + Clone,
    {
        let attr_type_id: i64 = match dh_in/*%%.attr_type_id*/ {
            AttributeDataHolder::QuantityAttributeDH{qadh} => qadh.attr_type_id,
            AttributeDataHolder::TextAttributeDH{tadh} => tadh.attr_type_id,
            AttributeDataHolder::DateAttributeDH{dadh} => dadh.attr_type_id,
            AttributeDataHolder::BooleanAttributeDH{badh} => badh.attr_type_id,
            AttributeDataHolder::FileAttributeDH{fadh} => fadh.attr_type_id,
            _ => {
                //%%make this do a better error message experience?--show the form name if avail?
                //I.e, see form_name in db code (findgrepfiles) or some attr code?
                //(same at other place w/ similar cmt)
                self.ui
                    .display_text1(format!("Unexpected type (attr_type, ...): {}, {:?}", attr_type, dh_in).as_str());
                return Ok(false);
            },
        };
        let attr_type_name = Entity::new2(db_in.clone(), None, attr_type_id)?.get_name(None)?;
        //%%?:let ans = controller.ask_for_attribute_data(
        let ans = self.ask_for_attribute_data(
            db_in.clone(),
            dh_in,
            ask_for_attr_type_id,
            attr_type,
            Some(prompt_for_type_id),
            Some(attr_type_name),
            Some(attr_type_id),
            get_other_info_from_user,
            true,
        )?;
        let Some(mut dh_out) = ans else {
            return Ok(false);
        };
        let ans2 = Util::prompt_whether_to_1add_or_2correct(attr_type, &self.ui)?;
        match ans2 {
            None => Ok(false),
            Some(1) => {
                update_typed_attribute(attribute_in, dh_out).unwrap();
                Ok(true)
            }
            Some(2) => {
                let return_value = self.ask_for_info_and_update_attribute(
                    //%%%controller,
                    db_in,
                    attribute_in,
                    &mut dh_out,
                    ask_for_attr_type_id,
                    attr_type,
                    prompt_for_type_id,
                    get_other_info_from_user,
                    update_typed_attribute,
                );
                return_value
            }
            _ => {
                self.ui
                    .display_text1("unexpected result! should never get here");
                Ok(false)
            }
        }
    }

    /// @return whether the attribute in question was deleted (or archived)
    //%%?:@tailrec
    //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive
    //call(s) within this method, below!
    fn attribute_edit_menu(&self, attribute_in: &mut dyn Attribute) -> Result<bool, anyhow::Error> {
        let leading_text = vec![format!(
            "Attribute: {}",
            attribute_in.get_display_string(0, None, None, false)?
        )];
        let mut e = Entity::new2(
            attribute_in.get_db(),
            None,
            attribute_in.get_attr_type_id(None)?
        )?;
        let mut first_choices = vec![
            format!(
                "Edit the attribute type, {}and valid/observed dates",
                if Util::can_edit_attribute_on_single_line(self.db.clone(), attribute_in)? {
                    "content (single line), "
                } else {
                    ""
                }
            ),
            if self.db.borrow().get_attribute_form_name(attribute_in.get_form_id()?)? == Util::TEXT_TYPE {
            //%% if Database::get_attribute_form_name(&self.db, attribute_in.get_form_id()?)?
                // == Util::TEXT_TYPE
            // {
                "Edit (as multi-line value)".to_string()
            } else {
                "(stub)".to_string()
            },
            if Util::can_edit_attribute_on_single_line(self.db.clone(), attribute_in)? {
                "Edit the attribute content (single line)".to_string()
            } else {
                "(stub)".to_string()
            },
            "Delete".to_string(),
            format!(
                "Go to entity representing the type: {}",
               e.get_name(None)?
            ),
        ];
        if self
            .db
            .borrow().get_attribute_form_name(attribute_in.get_form_id()?)?
            == Util::FILE_TYPE
        {
            first_choices.push("Export the file".to_string());
        }
        let response = self.ui.ask_which(
            Some(leading_text),
            &first_choices,
            &Vec::new(),
            true,
            None,
            None,
            None,
            None,
        );
        let Some(answer) = response else {
            return Ok(false);
        };
        let form_id = attribute_in.get_form_id()?;
        if answer == 1 {
            // Edit attribute details based on type
            if self.db.borrow().get_attribute_form_name(form_id)? == Util::QUANTITY_TYPE {
                //%%Is there a better way to do this? next line? this works for now. maybe do like
                //in attribute_data_holder.rs , making an Attribute enum and having structs in the
                //variants?
                //claude said: let quantity_attr = attribute_in.as_quantity_attribute();
                //Could do it like the match just below?
                let mut quantity_attr: QuantityAttribute =
                    QuantityAttribute::new2(attribute_in.get_db(), None, attribute_in.get_id())?;
                let qadh = QuantityAttributeDH {
                    attr_type_id: quantity_attr.get_attr_type_id(None)?,
                    valid_on_date: quantity_attr.get_valid_on_date(None)?,
                    observation_date: quantity_attr.get_observation_date(None)?,
                    number: quantity_attr.get_number(None)?,
                    unit_id: quantity_attr.get_unit_id(None)?,
                };
                let mut dh = AttributeDataHolder::QuantityAttributeDH { qadh };
                //See %%%%%%%s in Util.java for ideas re this and similar places below?
                let update_fn = |mut quantity_attr_in: &mut dyn Attribute, dh: AttributeDataHolder| -> Result<(), anyhow::Error> {
                    let Some(qa) = quantity_attr_in.as_any_mut().downcast_mut::<QuantityAttribute>() else {
                        return Err(anyhow!("unexpected attribute type: {:?}", quantity_attr_in));
                    };
                    match dh {
                        AttributeDataHolder::QuantityAttributeDH { qadh } => qa.update(
                            None,
                            qadh.attr_type_id,
                            qadh.unit_id,
                            qadh.number,
                            qadh.valid_on_date,
                            qadh.observation_date,
                        ),
                        _ => {
                            return Err(anyhow!("Unexpected attribute type: {:?}", dh));
                            //unreachable!("Unexpected attribute type. Can't get here right?");
                        }
                    }
                };
                let fn_ask_for_info: fn(&Controller, Rc<RefCell<dyn Database>>, &mut AttributeDataHolder, bool, &TextUI) -> Result<Option<AttributeDataHolder>, anyhow::Error> = Self::ask_for_quantity_attribute_number_and_unit;
                self.ask_for_info_and_update_attribute(
                    attribute_in.get_db(),
                    // Box::new(quantity_attr),
                    &mut quantity_attr as &mut dyn Attribute,
                    &mut dh,
                    true,
                    Util::QUANTITY_TYPE,
                    Util::QUANTITY_TYPE_PROMPT,
                    // Self::ask_for_quantity_attribute_number_and_unit,
                    fn_ask_for_info,
                    update_fn,
                );
                //force a reread from the DB so it shows the right info on the repeated menu:
                let mut x = QuantityAttribute::new2( attribute_in.get_db(), None, attribute_in.get_id())?;
                self.attribute_edit_menu(&mut x)
            } else if self.db.borrow().get_attribute_form_name(form_id)? == Util::TEXT_TYPE {
                //%%%%%see similar notes above for quantity
                //let text_attr = attribute_in.as_text_attribute();
                let mut text_attr: TextAttribute =
                    TextAttribute::new2(attribute_in.get_db(), None, attribute_in.get_id())?;
                let tadh = TextAttributeDH {
                    attr_type_id: text_attr.get_attr_type_id(None)?,
                    valid_on_date: text_attr.get_valid_on_date(None)?,
                    observation_date: text_attr.get_observation_date(None)?,
                    text: text_attr.get_text(None)?,
                };
                let mut dh = AttributeDataHolder::TextAttributeDH { tadh };
                let update_fn =
                    //%%%is this fixed in similar code in ler3.rs? (sch for "|dh")
                    |mut text_attr_in: &mut dyn Attribute, dh_in: AttributeDataHolder/*::TextAttributeDH*/| -> Result<(), anyhow::Error> {
                    let Some(ta) = text_attr_in.as_any_mut().downcast_mut::<TextAttribute>() else {
                        return Err(anyhow!("unexpected attribute type: {:?}", text_attr_in));
                    };
                        match dh_in {
                            AttributeDataHolder::TextAttributeDH{ tadh } => {
                                ta.update(
                                    None,
                                    tadh.attr_type_id,
                                    &tadh.text,
                                    tadh.valid_on_date,
                                    tadh.observation_date,
                                )
                            },
                            _ => {
                                Err(anyhow!("unexpected variant of attributeDataHolder: {:?}", dh_in))
                            },
                        }
                    };
                self.ask_for_info_and_update_attribute(
                    attribute_in.get_db(),
                    attribute_in,
                    &mut dh,
                    true,
                    Util::TEXT_TYPE,
                    &format!("CHOOSE TYPE OF {}:", Util::TEXT_DESCRIPTION),
                    Util::ask_for_text_attribute_text,
                    update_fn,
                );
                //force a reread from the DB so it shows the right info on the repeated menu:
                let mut ta = TextAttribute::new2(
                    attribute_in.get_db(),
                    None,
                    attribute_in.get_id(),
                )?;
                self.attribute_edit_menu(&mut ta)
            //%%} else if form_id == Util::DATE_TYPE_FORM_ID {
            } else if self.db.borrow().get_attribute_form_name(form_id)? == Util::DATE_TYPE {
                //%%%%%re next line, see similar plc above with QuantityAttribute
                //let date_attr: DateAttribute = attribute_in.as_date_attribute();
                let mut date_attr: DateAttribute =
                    DateAttribute::new2(attribute_in.get_db(), None, attribute_in.get_id())?;
                let dadh = DateAttributeDH {
                    attr_type_id: date_attr.get_attr_type_id(None)?,
                    date: date_attr.get_date(None)?,
                };
                let mut dh = AttributeDataHolder::DateAttributeDH { dadh };
                let update_fn = |mut date_attribute_in: &mut dyn Attribute, dh_in: AttributeDataHolder/*%%%%%%%::DateAttributeDH*/| -> Result<(), anyhow::Error> {
                    let Some(da) = date_attribute_in.as_any_mut().downcast_mut::<DateAttribute>() else {
                        return Err(anyhow!("unexpected attribute type: {:?}", date_attribute_in));
                    };
                    match dh_in {
                        AttributeDataHolder::DateAttributeDH{ dadh } => {
                            da.update(None, dadh.attr_type_id, dadh.date)
                        },
                        _ => {
                            Err(anyhow!("unexpected variant of attributeDataHolder: {:?}", dh_in))
                        },
                    }
                };
                self.ask_for_info_and_update_attribute(
                    attribute_in.get_db(),
                    attribute_in,
                    &mut dh,
                    true,
                    Util::DATE_TYPE,
                    "CHOOSE TYPE OF DATE:",
                    Util::ask_for_date_attribute_value,
                    update_fn,
                );
                //force a reread from the DB so it shows the right info on the repeated menu:
                let mut da = DateAttribute::new2(
                    attribute_in.get_db(),
                    None,
                    attribute_in.get_id(),
                )?;
                self.attribute_edit_menu(&mut da)
            //%%} else if form_id == Util::BOOLEAN_TYPE_FORM_ID {
            } else if self.db.borrow().get_attribute_form_name(form_id)? == Util::BOOLEAN_TYPE {
                //%%%%%see similar places above
                //let boolean_attr = attribute_in.as_boolean_attribute().unwrap();
                let mut boolean_attr: BooleanAttribute =
                    BooleanAttribute::new2(attribute_in.get_db(), None, attribute_in.get_id())?;
                let badh = BooleanAttributeDH {
                    attr_type_id: boolean_attr.get_attr_type_id(None)?,
                    valid_on_date: boolean_attr.get_valid_on_date(None)?,
                    observation_date: boolean_attr.get_observation_date(None)?,
                    boolean: boolean_attr.get_boolean(None)?,
                };
                let mut dh = AttributeDataHolder::BooleanAttributeDH { badh };
                //%%can just pass dh on this & similar lines? or any other way to simplify? 
                let update_fn = |mut boolean_attribute_in: &mut dyn Attribute, dh_in: AttributeDataHolder/*%%::BooleanAttributeDH*/| -> Result<(), anyhow::Error> {
                    let Some(ba) = boolean_attribute_in.as_any_mut().downcast_mut::<BooleanAttribute>() else {
                        return Err(anyhow!("unexpected attribute type: {:?}", boolean_attribute_in));
                    };
                    match dh_in {
                        AttributeDataHolder::BooleanAttributeDH{ badh } => {
                            ba.update(
                                None,
                                badh.attr_type_id,
                                badh.boolean,
                                badh.valid_on_date,
                                badh.observation_date,
                            )
                        },
                        _ => {
                            Err(anyhow!("unexpected variant of attributeDataHolder: {:?}", dh_in))
                        },
                    }
                };
                self.ask_for_info_and_update_attribute(
                    attribute_in.get_db(),
                    attribute_in,
                    &mut dh,
                    true,
                    Util::BOOLEAN_TYPE,
                    "CHOOSE TYPE OF TRUE/FALSE VALUE:",
                    Util::ask_for_boolean_attribute_value,
                    update_fn,
                );
                //force a reread from the DB so it shows the right info on the repeated menu:
                let mut ba = BooleanAttribute::new2(
                    attribute_in.get_db(),
                    None,
                    attribute_in.get_id(),
                )?;
                self.attribute_edit_menu(&mut ba)
            //} else if form_id == Util::FILE_TYPE_FORM_ID {
            } else if self.db.borrow().get_attribute_form_name(form_id)? == Util::FILE_TYPE {
                return Err(anyhow!("file stuff not supported yet"));
                /*%%
                //%%%%%see similar places just above for possible improvement to next line:
                //let file_attr = attribute_in.as_file_attribute();
                let file_attr: FileAttribute =
                    FileAttribute::new2(attribute_in.get_db(), None, attribute_in.get_id())?;
                let mut dh = AttributeDataHolder::FileAttributeDH {
                    attr_type_id: file_attr.get_attr_type_id(None)?,
                    description: file_attr.get_description(None)?,
                    original_file_path: file_attr.get_original_file_path(None)?,
                };
                let update_fn = |dh: AttributeDataHolder/*%%%%%%%::FileAttributeDH*/| -> Result<(), anyhow::Error> {
                    file_attr.update(Some(dh.attr_type_id), Some(&dh.description))
                };
                self.ask_for_info_and_update_attribute(
                    attribute_in.get_db(),
                    attribute_in,
                    &mut dh,
                    true,
                    Util::FILE_TYPE,
                    "CHOOSE TYPE OF FILE:",
                    Util::ask_for_file_attribute_info,
                    update_fn,
                );
                //force a reread from the DB so it shows the right info on the repeated menu:
                self.attribute_edit_menu(&FileAttribute::new2(
                    attribute_in.get_db(),
                    None,
                    attribute_in.get_id(),
                ))
                    */
            } else {
                //%%make this do a better error message experience?--show the form name if avail?
                //I.e, see form_name in db code (findgrepfiles) or some attr code?
                //(same at other place w/ similar cmt)
                self.ui
                    .display_text1(format!("Unexpected type (form_id): {}", form_id).as_str());
                Ok(false)
            }
        //} else if answer == 2 && attribute_in.get_form_id() == Util::TEXT_TYPE_FORM_ID {
        } else if answer == 2 && self.db.borrow().get_attribute_form_name(form_id)? == Util::TEXT_TYPE {
            //%%%%%see similar lines above
            //let ta = attribute_in.as_text_attribute();
            let mut ta: TextAttribute =
                TextAttribute::new2(attribute_in.get_db(), None, attribute_in.get_id())?;
            let new_content = Util::edit_multiline_text(&ta.get_text(None)?, &self.ui)?;
            let attr_type_id = ta.get_attr_type_id(None)?;
            let vod = ta.get_valid_on_date(None)?;
            let od = ta.get_observation_date(None)?;
            ta.update(
                None,
                attr_type_id,
                &new_content,
                vod,
                od,
            )?;
            //then force a reread from the DB so it shows the right info on the repeated menu:
            let mut ta = TextAttribute::new2(
                attribute_in.get_db(),
                None,
                attribute_in.get_id(),
            )?;
            self.attribute_edit_menu(&mut ta)
        } else if answer == 3 && Util::can_edit_attribute_on_single_line(self.db.clone(), attribute_in)? {
            self.edit_attribute_on_single_line(attribute_in);
            Ok(false)
        } else if answer == 4 {
            let ans =
                self.ui
                    .ask_yes_no_question("DELETE this attribute: ARE YOU SURE?", "", false);
            if ans.is_some() && ans.unwrap() {
                attribute_in.delete(None);
                Ok(true)
            } else {
                self.ui.display_text2("Did not delete attribute.", false);
                self.attribute_edit_menu(attribute_in)
            }
        } else if answer == 5 {
            return Err(anyhow!("EntityMenu stuff not supported yet"));
            /*%%
            EntityMenu::new(self.ui.clone(), Rc::new(self.clone())).entity_menu(
                &Entity::new2(
                    attribute_in.get_db(),
                    None,
                    attribute_in.get_attr_type_id(None),
                ),
                None,
            );
            self.attribute_edit_menu(attribute_in)
            */
        //} else if answer == 6 && attribute_in.get_form_id() == Util::FILE_TYPE_FORM_ID {
        } else if answer == 6 && self.db.borrow().get_attribute_form_name(form_id)? == Util::FILE_TYPE {
            //if !attributeIn.isInstanceOf[FileAttribute]) throw new Exception("Menu shouldn't have
            //allowed us to get here w/ a type other than FA (" + attributeIn.getClass.get_name + ").")
            //%%%%%see similar lines above
            //let fa = attribute_in.as_file_attribute().unwrap();
            let fa: FileAttribute =
                FileAttribute::new2(attribute_in.get_db(), None, attribute_in.get_id())?;
            // this file should be confirmed by the user as ok to write, even overwriting what is there.
            return Err(anyhow!("file stuff not supported yet"));
            /*%%
            match self
                .ui
                .get_export_destination(&fa.get_original_file_path(), &fa.get_md5hash())
            {
                Ok(Some(file_path)) => match fa.retrieve_content(&file_path) {
                    Ok(_) => self
                        .ui
                        .display_text(&format!("File saved at: {}", file_path.display())),
                    Err(e) => self
                        .ui
                        .display_text(&format!("Failed to export file, due to error: {}", e)),
                },
                //%%should next line give some message? When can it occur? Scala code didn't give a msg either.
                Ok(None) => {}
                Err(e) => self
                    .ui
                    .display_text(&format!("Failed to export file, due to error: {}", e)),
            }
            self.attribute_edit_menu(attribute_in)
            */
            //%%old/scala version. Remove after the above is tested well enuf?
            //} else if answer == 6) {
            //  //%%see 1st instance of try {  for rust-specific idea here.
            //  try {
            //    let file: Option[File] = ui.getExportDestination(fa.get_original_file_path(), fa.get_md5hash());
            //    if file.is_defined) {
            //      fa.retrieveContent(file.get)
            //      ui.display_text("File saved at: " + file.get.getCanonicalPath)
            //    }
            //  } catch {
            //    case e: Exception =>
            //      let msg: String = Util.throwableToString(e);
            //      ui.display_text("Failed to export file, due to error: " + msg)
            //  }
            //  attributeEditMenu(attributeIn)
        } else {
            self.ui.display_text1("invalid response");
            self.attribute_edit_menu(attribute_in)
        }
    }

    /// Returns whether the user wants just to get out.
    //%%should test these manually? 
    pub fn edit_attribute_on_single_line(
        &self,
        //attribute_in: Box<&dyn Attribute>,
        attribute_in: &dyn Attribute,
    ) -> Result<bool, anyhow::Error> {
        if ! Util::can_edit_attribute_on_single_line(self.db.clone(), attribute_in)? {
            return Err(anyhow!("Failed expectation: can_edit_attribute_on_single_line should be true"));
        }
        let form_id = attribute_in.get_form_id()?;
        if self.db.borrow().get_attribute_form_name(form_id)? == Util::QUANTITY_TYPE {
            let mut quantity_attr: QuantityAttribute =
                QuantityAttribute::new2(attribute_in.get_db(), None, attribute_in.get_id())?;
            let num =
                Util::ask_for_quantity_attribute_number(quantity_attr.get_number(None)?, &self.ui);
            let Some(number) = num else {
                return Ok(true);
            };
            let attr_type_id = quantity_attr.get_attr_type_id(None)?;
            let unit_id = quantity_attr.get_unit_id(None)?;
            let vod = quantity_attr.get_valid_on_date(None)?;
            let od = quantity_attr.get_observation_date(None)?;
            quantity_attr.update(
                None,
                attr_type_id,
                unit_id,
                number,
                vod,
                od,
            )?;
            Ok(false)
        } else if self.db.borrow().get_attribute_form_name(form_id)? == Util::TEXT_TYPE {
            let mut text_attr: TextAttribute =
                TextAttribute::new2(attribute_in.get_db(), None, attribute_in.get_id())?;
            let tadh_in: TextAttributeDH = TextAttributeDH {
                attr_type_id: text_attr.get_attr_type_id(None)?,
                valid_on_date: text_attr.get_valid_on_date(None)?,
                observation_date: text_attr.get_observation_date(None)?,
                text: text_attr.get_text(None)?.clone(),
            };
            let mut dhv_in = AttributeDataHolder::TextAttributeDH { tadh: tadh_in };
            let dhv_out_opt =
                Util::ask_for_text_attribute_text(&self, attribute_in.get_db(), &mut dhv_in, true, &self.ui)?;
            if let Some(dhv_out) = dhv_out_opt {
                let AttributeDataHolder::TextAttributeDH { tadh: tadh_out } = dhv_out else {
                    return Err(anyhow!("Unexpected type for AttributeDataHolder: {:?}", dhv_out));
                };
                // %%%%%%%%%%FIX THIS and places like it to be clearer? it reads like this is just saving what got passed in as parms. It might be an in/out var, so OK, but is not clear. Find out and make it clear then do others similarly.
                text_attr.update(
                    None,
                    tadh_out.attr_type_id,
                    tadh_out.text.as_str(),
                    tadh_out.valid_on_date,
                    tadh_out.observation_date,
                )?;
                Ok(false)
            } else {
                Ok(true)
            }
        } else if self.db.borrow().get_attribute_form_name(form_id)? == Util::DATE_TYPE {
            let mut date_attr: DateAttribute =
                DateAttribute::new2(attribute_in.get_db(), None, attribute_in.get_id())?;
            let dh = DateAttributeDH { 
                attr_type_id: date_attr.get_attr_type_id(None)?,
                date: date_attr.get_date(None)?,
            };
            let mut dhv = AttributeDataHolder::DateAttributeDH { dadh: dh };
            let dhv_out =
                Util::ask_for_date_attribute_value(&self, attribute_in.get_db(), &mut dhv, true, &self.ui)?;
            let Some(AttributeDataHolder::DateAttributeDH { dadh: dadh_out }) = dhv_out else {
                // return Err(anyhow!("Unexpected type AttributeDataHolder: {:?}", dhv_out));
            //%% } else {
                return Ok(true);
            //%% }
            };
            // if let Some(dhv_out) = dhv_out {
                date_attr.update(None, dadh_out.attr_type_id, dadh_out.date)?;
                Ok(false)
            // } else {
            //     Ok(true)
            // }
        } else if self.db.borrow().get_attribute_form_name(form_id)? == Util::BOOLEAN_TYPE {
            //%%see similars elsewhere
            //let boolean_attr = attribute_in.as_boolean_attribute().unwrap();
            let mut boolean_attr: BooleanAttribute =
                BooleanAttribute::new2(attribute_in.get_db(), None, attribute_in.get_id())?;
            let badh = BooleanAttributeDH {
                attr_type_id: boolean_attr.get_attr_type_id(None)?,
                valid_on_date: boolean_attr.get_valid_on_date(None)?,
                observation_date: boolean_attr.get_observation_date(None)?,
                boolean: boolean_attr.get_boolean(None)?,
            };
            let mut dh_in = AttributeDataHolder::BooleanAttributeDH { badh };
            let dh_out =
                Util::ask_for_boolean_attribute_value(&self, attribute_in.get_db(), &mut dh_in, true, &self.ui)?;
            if let Some(AttributeDataHolder::BooleanAttributeDH { badh }) = dh_out {
                boolean_attr
                    .update(
                        None,
                        badh.attr_type_id,
                        badh.boolean,
                        badh.valid_on_date,
                        badh.observation_date,
                    )
                    .unwrap();
                Ok(false)
            } else {
                Ok(true)
            }
        } else if self.db.borrow().get_attribute_form_name(form_id)? == Util::RELATION_TO_LOCAL_ENTITY_TYPE {
            let rtle: RelationToLocalEntity = 
                match RelationToLocalEntity::new3(attribute_in.get_db(), None, attribute_in.get_id())? {
                    Some(x) => x,
                    None => {
                        return Err(anyhow!("RelationToLocalEntity {} not found.", attribute_in.get_id()));
                    },
                };
            let mut e: Entity = Entity::new2(rtle.get_db(), None, rtle.get_related_id2())?;
            let edited_entity =
                self.edit_entity_name(&mut e)?;
            Ok(edited_entity.is_none())
        } else if self.db.borrow().get_attribute_form_name(form_id)? == Util::RELATION_TO_REMOTE_ENTITY_TYPE {
            return Err(anyhow!("Not yet implemented."));
            /*%%
            //%%see similars above
            //let rtre = attribute_in.as_relation_to_remote_entity()?;
            let rtre: RelationToRemoteEntity =
                RelationToRemoteEntity::new3(attribute_in.get_db(), None, attribute_in.get_id())?;
            let edited_entity = self.edit_entity_name(&Entity::new2(
                rtre.get_remote_database(),
                None,
                rtre.get_related_id2(),
            ));
            Ok(edited_entity.is_none())
            */
        } else if self.db.borrow().get_attribute_form_name(form_id)? == Util::RELATION_TO_GROUP_TYPE {
            let mut rtg: RelationToGroup =
                RelationToGroup::new3(attribute_in.get_db(), None, attribute_in.get_id())?;
            let edited_group_name: Option<String> = Util::edit_group_name(
                &mut Group::new2(rtg.get_db(), None, rtg.get_group_id(None)?)?,
                &self.ui,
            )?;
            Ok(edited_group_name.is_none())
        } else {
            //%%Is there some kind of method to get type name? see some other note like this or
            //similar place above? like where I mention "form_name" above or such!
            //do a findgrep....
            self.ui
                .display_text1(format!("Unexpected type: {:?}", attribute_in/*%%.get_class_name()*/).as_str());
            Ok(true)
        }
    }

    /// For return info, see add_attribute method (or add_typed_attribute?).
    pub fn ask_for_info_and_add_attribute(
        &self,
        db_in: Rc<RefCell<dyn Database>>,
        dh_in: &mut AttributeDataHolder,
        ask_for_attr_type_id: bool,
        attr_type: &str,
        prompt_for_selecting_type_id: Option<&str>,
        entity_in: &Entity,
        get_other_info_from_user: fn(
            &Controller,
            Rc<RefCell<dyn Database>>,
            &mut AttributeDataHolder,
            bool,
            &TextUI,
        ) -> Result<Option<AttributeDataHolder>, anyhow::Error>,
        add_typed_attribute: fn(&mut AttributeDataHolder, &Entity) -> Result<Option<Box<dyn Attribute>>, anyhow::Error>,
    ) -> Result<Option<Box<dyn Attribute>>, anyhow::Error>
//where
    //    T: AttributeDataHolder + Clone,
    {
        let ans: Option<AttributeDataHolder> = self.ask_for_attribute_data(
            db_in,
            dh_in,
            ask_for_attr_type_id,
            attr_type,
            prompt_for_selecting_type_id,
            None,
            None,
            get_other_info_from_user,
            false,
        )?;
        // ans.and_then(|dh_out, &entity| add_typed_attribute(dh_out, entity))
        let Some(mut dh_out) = ans else {
            return Err(anyhow!("Unexpected answer: {:?}.", ans));
        };
        add_typed_attribute(&mut dh_out, entity_in)
    }

    //%%fix the other method name in comment here, after confirming it newly exists?
    /// SEE DESCRIPTIVE COMMENT ON askForAndWriteClassAndTemplateEntityName, WHICH APPLIES TO
    /// all such METHODS (see this cmt elsewhere).
    /// Returns None if user wants out.
    fn edit_entity_name(&self, entity_in: &mut Entity) -> Result<Option<Entity>, anyhow::Error> {
        let entity_name_before_edit: String = entity_in.get_name(None)?;//%%.unwrap_or_default(); //%%%%%elim unwrap? default??
        //let previous_name/*%%: &str*/ = entity_in.get_name(None)?;
            //.unwrap_or("".to_string().as_ref())
            //.to_str();
        let mut edited_entity: Option<Entity> = self.ask_for_name_and_write_entity(
            entity_in.get_db(),
            Util::ENTITY_TYPE,
            Rc::new(RefCell::new(Some(entity_in))), //%%.clone()),
            //%%%%%unwrap? default? use value obtained above? Just pass None if MT (is defa in old method)?
            //Some(&previous_name.as_str()),
            Some(entity_name_before_edit.clone()),
            None,
            None,
            None,
            None,
            false,
        )?;
        //%%if let Some(ref entity) = edited_entity {
        let Some(mut edited_entity) = edited_entity else {
            return Ok(None);
        };
        let entity_name_after_edit: String = edited_entity.get_name(None)?;//%%.unwrap_or_default(); 
        if entity_name_before_edit != entity_name_after_edit {
            let (_, _, group_id, group_name, more_than_one_available) =
                edited_entity.find_relation_to_and_group(None)?;
            if let (Some(gid), Some(gn)) = (group_id, group_name) {
                if !more_than_one_available {
                    let attr_count = entity_in
                        .get_attribute_count(None, self.db.borrow().include_archived_entities())?;
                    // for efficiency, if it's obvious which subgroup's name to change at the same time, offer to do so
                    let default_answer = if attr_count > 1 { "n" } else { "y" };
                    let comment = if attr_count > 1 {
                        format!(" (***AMONG {} OTHER ATTRIBUTES***)", attr_count - 1)
                    } else {
                        String::new()
                    };
                    let question = format!(
                        "There's a single subgroup named \"{}\"{}; possibly it and this entity were \
                        created at the same time. Also change \
                        the subgroup's name now to be identical?",
                        gn,
                        comment,
                    );
                    let ans = self
                        .ui
                        .ask_yes_no_question(&question, default_answer, false);
                    if ans.is_some() && ans.unwrap() {
                        let mut group = Group::new2(entity_in.get_db(), None, gid)?;
                        group.update(None, None, Some(&entity_name_after_edit), None, None, None, None)?;
                    }
                }
            }
        }
        Ok(Some(edited_entity))
    }
}
