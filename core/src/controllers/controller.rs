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
pub struct Controller {
    pub ui: Rc<TextUI>,
    force_user_pass_prompt: bool,
    // NOTE: This should *not* be the definitive DB used for everything, but rather those
    // places in the code that can use a non-local DB should get the DB instance from the
    // entity (or other model object) being processed, to be sure the correct DB instance is used
    // (as there can be Entities etc that are from a different, remote, DB).
    pub/*%%make these all pub(crate)?*/ db: Rc<RefCell<dyn Database>>,
    // putting this in a var instead of recalculating it every time (too frequent) inside find_default_display_entity_id:
    pub(crate) show_public_private_status_preference: Option<bool>,
    default_display_entity_id: Option<i64>,
    move_farther_count: i32,
    move_farthest_count: i32,
}

impl Controller {
    pub fn new_for_non_tests(
        ui: TextUI,
        force_user_pass_prompt: bool,
        default_username: Option<&String>,
        default_password: Option<&String>,
    ) -> Result<Controller, anyhow::Error> {
        let db = Self::try_db_logins(
            force_user_pass_prompt,
            &ui,
            default_username,
            default_password,
        )
        .unwrap_or_else(|e| {
            //idea: should panic instead, at all places like this? to get a stack trace and for style?
            //OR, only if it is truly something unanticipated? Are there not times when returning a failure is expected?
            //%%should eprintln at other places like this also?
            // ui.display_text1(e.to_string().as_str());
            eprintln!("{}", e.to_string().as_str());
            std::process::exit(1);
        });
        let show_public_private_status_preference: Option<bool> = db
            .borrow()
            .get_user_preference_boolean(None, Util::SHOW_PUBLIC_PRIVATE_STATUS_PREFERENCE, None)?;
        let default_display_entity_id: Option<i64> =
             //(see comment in call to expect() in get_default_entity() .)
             db.borrow().get_user_preference_entity_id(None, Util::DEFAULT_ENTITY_PREFERENCE, None)
                 .expect("Faiure in call to get_user_preference_entity_id()");
        Ok(Controller {
            ui: Rc::new(ui),
            force_user_pass_prompt,
            db,
            show_public_private_status_preference,
            default_display_entity_id,
            move_farther_count: 25,
            move_farthest_count: 50,
        })
    }

    /// Returns the id and the entity, if they are available from the preferences lookup (id)
    /// and then finding that in the db (Entity).
    fn get_default_entity(&self) -> Option<(i64, Entity)> {
        match self.default_display_entity_id {
            None => None,
            Some(ddei) => {
                // Calling expect() here because we generally handle Errors in the *Menu
                // struct impls. Errors are not expected to occur here (and didn't, in long use
                // of the version of OM written in Scala).
                let entity: Option<Entity> = Entity::get_entity(self.db.clone(), None, ddei)
                    .expect("Failure in call to Entity::get_entity.");
                match entity {
                    None => None,
                    Some(mut entity) => {
                        if entity
                            .is_archived(None)
                            .expect("Unable to determine if entity.is_archived.")
                        {
                            let msg = format!(
                                "The default entity \n    {}: \"{} \"\n\
                                ... was found but is archived.  You might run into problems \
                                unless you un-archive it, or choose a different entity to make \
                                the default, or display all archived entities then search for \
                                this entity and un-archive it under its Entity Menu options 9, 4.",
                                entity.get_id(),
                                entity
                                    .get_name(None)
                                    .expect("Error running entity.get_name(.")
                            );
                            let ans = self.ui.ask_which(
                                Some(vec![&msg.as_str()]),
                                &vec![
                                    "Un-archive the default entity now".to_string(),
                                    "Display archived entities".to_string(),
                                ],
                                &vec![],
                                true,
                                None,
                                None,
                                None,
                                None,
                            );
                            if ans.is_some() {
                                if ans.unwrap() == 1 {
                                    entity.unarchive(None);
                                } else if ans.unwrap() == 2 {
                                    let db: Rc<RefCell<dyn Database>> = self.db.clone();
                                    let mut db_mut: RefMut<'_, _> = db.borrow_mut();
                                    db_mut.set_include_archived_entities(true);
                                }
                            }
                        }
                        Some((entity.get_id(), entity))
                    }
                }
            }
        }
    }

    pub fn start(self: Rc<Controller>) {
        // idea: wait for keystroke so they do see the copyright each time. (is also tracked):
        // make it save their answer 'yes/i agree' or such in the DB, and don't make them press
        // the keystroke again (time-saver)!  See code at top of postgresql_database.rs that
        // puts things in the db at startup: do similarly?
        self.ui.display_text3(
            Util::license().as_str(),
            true,
            Some(
                String::from("IF YOU DO NOT AGREE TO THOSE TERMS: ")
                    + self.ui.how_quit()
                    + " to exit.\n"
                    + "If you agree to those terms: ",
            ),
        );

        // (2025: Not sure I understand this comment any more, even after checking the old scala code.)
        // Max id used as default here because it seems the least likely # to be used in the system hence the
        // most likely to cause an error as default by being missing, so the system can respond by prompting
        // the user in some other way for a use.
        //let default_entity_info = self.get_default_entity();
        let mut default_entity: Option<Entity> = match self.get_default_entity() {
            Some((_, default_entity)) => Some(default_entity),
            None => {
                self.ui.display_text1("To get started, you probably want to find or create an \
                          entity (such as with your own name, to track information \
                          connected to you, contacts, possessions etc, or with the subject \
                          of study), then set that or some entity as your default (using its menu).");
                None
            }
        };

        // Explicitly *not* properly tail-recursive, so user can go "back" to previously viewed entities. See
        // comments below at "fn main_menu" for more on the feature of the user going back.
        // (But: this one currently only ever passes defaultEntity as a parameter, so there
        // is no "back", except what is handled within mainmenu calling itself.  It seems like if
        // we need to be any more clever we're going to want that stack back....see those same comments below.)
        //
        // The 1st parameter to mainMenu might be a kludge. But it lets us, at startup, go straight to
        // the attributeMenu of the default Entity.  When instead we simply called
        // entityMenu(0,defaultEntity.get) before going into menuLoop, it didn't have the usual
        // context for normal behavior, and caused odd things for the user, like choosing a related
        // entity to view its entity menu showed the default object's entity menu instead, until going
        // into the usual loop and choosing it again. Now we do it w/ the same code path, thus the
        // same behavior, as normally expected.
        let default_choice = Some(5);
        //let self_rc = Rc::new(self);
        //self.menu_loop(default_choice)
        loop {
            //%%%%remove if works w/o:
            //let entity = match default_entity_info {
            //        None => None,
            //        Some(t) => {
            //            let entity = t.1;
            //            Some(entity)
            //        },
            //};
            //MainMenu::new(self.ui, self.db.clone(), self_rc.clone()).main_menu(
            let mm = MainMenu::new(self.ui.clone(), self.db.clone(), self.clone());
            mm.main_menu(default_entity, default_choice);

            //%%%%just while debugging/testing:
            println!("any key2cont.../testing...");
            TextUI::wait_for_user_input_key();

            //Re-checking for the default each time because user can change it in this or another window.
            default_entity = match self.get_default_entity() {
                Some((_, default_entity)) => Some(default_entity),
                None => None,
            };
        }
        //%% */
    }

    /// If the 1st parm is true, the next 2 must be None.
    fn try_db_logins<'a>(
        force_user_pass_prompt: bool,
        ui: &'a TextUI,
        default_username: Option<&String>,
        default_password: Option<&String>,
    ) -> Result<Rc<RefCell<dyn Database>>, anyhow::Error> {
        if force_user_pass_prompt {
            //%%why had this assertion before?:  delete it now?  (it was a "require" in Controller.scala .)
            // assert!(default_username.is_none() && default_password.is_none());

            Self::prompt_for_user_pass_and_login(ui)
        } else if default_username.is_some() && default_password.is_some() {
            // idea: perhaps this could be enhanced and tested to allow a username parameter, but prompt for a password, if/when need exists.
            let user = default_username.unwrap_or_else(|| {
                ui.display_text1("How could username be absent? Just checked and it was there.");
                std::process::exit(1);
            });
            let pass = default_password.unwrap_or_else(|| {
                ui.display_text1("How could password be absent? Just checked and it was there.");
                std::process::exit(1);
            });
            let db_result = PostgreSQLDatabase::new(user, pass);
            // not attempting to clear that password variable because
            // maybe the default kind is less intended to be secure, anyway?
            db_result
        } else {
            Self::try_other_logins_or_prompt(ui)
        }
    }

    fn prompt_for_user_pass_and_login<'a>(
        ui: &TextUI,
    ) -> Result<Rc<RefCell<dyn Database>>, anyhow::Error> {
        loop {
            let usr = ui.ask_for_string1(vec!["Username"]);
            match usr {
                None => {
                    //user probably wants out
                    std::process::exit(1);
                }
                Some(username) => {
                    let pwd = ui.ask_for_string4(vec!["Password"], None, "".to_string(), true);
                    match pwd {
                        None => {
                            //user probably wants out
                            std::process::exit(1);
                        }
                        Some(password) => {
                            let db = PostgreSQLDatabase::new(username.as_str(), password.as_str());
                            if db.is_ok() {
                                break db;
                            } else {
                                // bad username/password combo? Let user retry.
                                continue;
                            }
                        }
                    }
                }
            }
        }
    }

    /// Tries the system username & default password, & if that doesn't work, prompts user.
    fn try_other_logins_or_prompt(ui: &TextUI) -> Result<Rc<RefCell<dyn Database>>, anyhow::Error> {
        // (this loop is to simulate recursion, and let the user retry entering username/password)
        loop {
            // try logging in with some obtainable default values first, to save user the trouble, like if pwd is blank
            let (default_username, default_password) = Util::get_default_user_login().unwrap_or_else(|e| {
                eprintln!("Unable to get default username/password.  Trying blank username, and password \"x\" instead.  Underlying error is: \"{}\"", e);
                ("".to_string(), "x")
            });
            let db_with_system_name_blank_pwd =
                PostgreSQLDatabase::new(default_username.as_str(), default_password);
            if db_with_system_name_blank_pwd.is_ok() {
                ui.display_text2("(Using default user info...)", false);
                break db_with_system_name_blank_pwd;
            } else {
                let usr = ui.ask_for_string3(vec!["Username"], None, default_username);
                match usr {
                    None => {
                        // seems like the user wants out
                        std::process::exit(1);
                    }
                    Some(username) => {
                        let db_connected_with_default_pwd =
                            PostgreSQLDatabase::new(username.as_str(), default_password);
                        if db_connected_with_default_pwd.is_ok() {
                            break db_connected_with_default_pwd;
                        } else {
                            let pwd = ui.ask_for_string4(vec!["Password"], None, "".to_string(), true);
                            match pwd {
                                None => {
                                    // seems like the user wants out
                                    std::process::exit(1);
                                }
                                Some(password) => {
                                    let db_with_user_entered_pwd = PostgreSQLDatabase::new(
                                        username.as_str(),
                                        password.as_str(),
                                    );
                                    match db_with_user_entered_pwd {
                                        Ok(db) => break Ok(db),
                                        Err(e) => {
                                            let msg = format!("Login failed; retrying ({} to quit if needed):  {}", ui.how_quit(), e.to_string());
                                            ui.display_text2(msg.as_str(), false)
                                        }
                                    }
                                    //%%AND: IN RUST instead of setting to null & doing gc(), could
                                    // look into the "zeroize" and "secrecy" crates for that, per an article
                                    // i just (20221201) read in "this week in Rust" rss feed,
                                    // "Rust Foundation - Secure App Development with rust's Memory Model", at
                                    //  https://foundation.rust-lang.org/news/secure-app-development-with-rust-s-memory-model/
                                    // OR: no need because Rust will clean up anyway? will it be reused
                                    // or could hang around to be exploited?
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Idea: show_public_private_status_preference, refresh_public_private_status_preference, and
    // find_default_display_entity_id(), feel awkward. Needs something better, but I'm not sure
    // what, at the moment.  It was created this way as a sort of cache because looking it up every
    // time was costly and made the app slow, like when displaying a list of entities (getting
    // the preference every time, to N levels deep), and especially at startup when checking for the default
    // up to N levels deep, among the preferences that can include entities with deep nesting.  So
    // in a related change I made it also not look N levels deep, for preferences.  If you check
    // other places touched by this commit there may be a "shotgun surgery" bad smell here also.
    //Idea: Maybe these should have their cache expire after a period of time (to help when running multiple clients).
    pub fn refresh_public_private_status_preference(&mut self) -> Result<(), anyhow::Error> {
        self.show_public_private_status_preference = self.db.borrow().get_user_preference_boolean(
            None,
            Util::SHOW_PUBLIC_PRIVATE_STATUS_PREFERENCE,
            None,
        )?;
        Ok(())
    }

    //%%never called? should be, like in loops in main_menu.rs or where we call main_menu here? remove or leave4now?
    pub fn refresh_default_display_entity_id(&mut self) -> Result<(), anyhow::Error> {
        self.default_display_entity_id = self.db.borrow().get_user_preference_entity_id(
            None,
            Util::DEFAULT_ENTITY_PREFERENCE,
            None,
        )?;
        Ok(())
    }

    fn ask_for_class(&self, db_in: Rc<RefCell<dyn Database>>) -> Result<Option<i64>, anyhow::Error> {
        let msg = "CHOOSE ENTITY'S CLASS. (Press ESC if you don't know or care about this. \
                  Detailed explanation on the class feature will be available \
                  at onemodel.org when this feature is documented more (hopefully at the next \
                  release), or ask on the email list.)";
        let result = self.choose_or_create_object(
            db_in,
            Some(vec![msg]),
            None,
            None,
            Util::ENTITY_CLASS_TYPE,
            0,
            None,
            false,
            None,
            false,
            None,
            false,
        )?;
        match result {
            None => Ok(None),
            Some((id_wrapper, _, _)) => Ok(Some(id_wrapper.get_id())),
        }
    }

    /// In any given usage, consider whether ask_for_name_and_write_entity should be used instead:
    /// it is for quick (simpler) creation situations or
    /// to just edit the name when the entity already exists, or if the Entity is a RelationType,
    /// ask_for_class_info_and_name_and_create_entity (this one) prompts for a class and checks whether it
    /// should copy default attributes from the class-defining
    /// (template) entity.
    /// There is also edit_entity_name which calls ask_for_name_and_write_entity: it checks if the Entity being
    /// edited is a RelationType, and if not also checks
    /// for whether a group name should be changed at the same time.
    /// And there is (or was? unused) edit_relation_type_name.
    pub fn ask_for_class_info_and_name_and_create_entity(
        &self,
        db_in: Rc<RefCell<dyn Database>>,
        class_id_in: Option<i64>, /*None*/
    ) -> Result<Option<Entity>, anyhow::Error> {
        let mut new_class = false;
        let class_id: Option<i64> = if class_id_in.is_some() {
            class_id_in
        } else {
            new_class = true;
            self.ask_for_class(db_in.clone())?
        };
        let leading_text = if new_class { "DEFINE THE ENTITY:" } else { "" };
        let entity_option = self.ask_for_name_and_write_entity(
            db_in.clone(),
            Util::ENTITY_TYPE,
            Rc::new(RefCell::new(None)),
            None,
            None,
            None,
            class_id,
            Some(leading_text),
            false,
        )?;
        if entity_option.is_some() {
            let mut entity = entity_option.unwrap();
            // idea: (is also on fix list): this needs to be removed, after evaluating for
            // other side effects, to fix the bug
            // where creating a new relationship, and creating the entity2 in the process, it puts the wrong info
            // on the header for what is being displayed/edited next!: Needs refactoring anyway: this shouldn't be at
            // a low level.
            self.ui.display_text2(
                &format!("Created {}: {}", Util::ENTITY_TYPE, entity.get_name(None)?),
                false,
            );
            self.default_attribute_copying(&mut entity, None);
            Ok(Some(entity))
        } else {
            Ok(None)
        }
    }

    /// Returns the id of the new entry created.
    fn ask_and_save(
        &self,
        //%%%%%should be here? is needed when doing multi dbs? ck callers.  other, similar places?
        db_in: Rc<RefCell<dyn Database>>,
        default_name: Option<String>,        /*= None*/
        leading_text_in: Option<&str>,       /*= None*/
        existing_entity_in: Rc<RefCell<Option<&mut Entity>>>, /*= None*/
        create_not_update: bool,
        type_in: &str,
        max_name_length: u16,
        example: &str,
        duplicate_name_probably_ok: bool,
        previous_name_in_reverse_in: Option<&str>,
        previous_directionality_in: Option<&str>,
        class_id_in: Option<i64>,
        class_in: &mut Option<EntityClass>,
        //%%) -> Result<Option<(i64, i64)>, anyhow::Error> {
    ) -> Result<Option<i64>, anyhow::Error> {
        assert!(create_not_update == existing_entity_in.borrow().is_none());
        let x = format!(
                "Enter {} name (up to {} characters{}; ESC to cancel)",
                type_in, max_name_length, example
            );
        let y = x.as_str();
        let prompt = vec![
            leading_text_in.unwrap_or(""),
            y,
        ];
        let x3: String = default_name.unwrap_or("").to_string();
        // let x4: &str = x3.as_str();
        let name_opt: Option<String> = self.ui.ask_for_string3(prompt, None, x3); //from claude:.as_deref());
        let Some(name) = name_opt else {
            return Ok(None);
        };
        let name = name.trim().to_string();
        if name.is_empty() {
            return Ok(None);
        }
        // idea: this size check might be able to account better for the escaping
        // that's done. Or just keep letting the exception handle it as is already
        // done in the caller of this.
        if name.len() > usize::from(max_name_length) {
            self.ui.display_text1(
                format!(
                    "{}.",
                    Util::string_too_long_error_message(max_name_length, Util::TOO_LONG_MESSAGE)
                )
                .as_str(),
            );
            return self.ask_and_save(
                db_in,
                Some(name.as_str()),
                leading_text_in,
                existing_entity_in,
                create_not_update,
                type_in,
                max_name_length,
                example,
                duplicate_name_probably_ok,
                previous_name_in_reverse_in,
                previous_directionality_in,
                class_id_in,
                class_in,
            );
        }
        //claude had as_ref: let self_id_to_ignore = existing_entity_in.as_ref().map(|e| e.get_id());
        let self_id_to_ignore: Option<i64> = match *existing_entity_in.borrow() {
            Some(ref eei) => Some(eei.get_id()),
            None => None,
        };
        if Util::is_duplication_a_problem(
            Entity::is_duplicate(db_in.clone(), None, name.as_str(), self_id_to_ignore)?,
            duplicate_name_probably_ok,
            &self.ui,
        ) {
            return Ok(None);
        }
        if type_in == Util::ENTITY_TYPE {
            if create_not_update {
                let new_id =
                    Entity::create_entity(db_in.clone(), None, &name, class_id_in, None)?.get_id();
                //%%Ok(Some((new_id, 0)))
                Ok(Some(new_id))
            } else {
                if existing_entity_in.borrow().is_none() {
                    return Err(anyhow!("Unexpected None for existing_entity_in??"));
                } else {
                    //then the assertion at start of method guarantees this unwrap() is safe.
                    //%%make sure next line is safe (tested), given borrow() just above--will this panic? or others like it?
                    //%%surely there is some better way than the next few lines?
                    let mut x = existing_entity_in.borrow_mut();
                    let y = x.as_mut();
                    let e:&mut Entity = y.unwrap();
                    e.update_name(None, &name.as_str())?;
                    //%%Ok(Some((existing_entity_in.unwrap().get_id(), 0)))
                    Ok(Some(e.get_id()))
                }
                // let Some(mut e) = existing_entity_in else {
                //     return Err(anyhow!("Unexpected None for existing_entity_in??"));
                // };
                //// let e: &mut Entity = 
            }
        } else if type_in == Util::RELATION_TYPE_TYPE {
            let directionality_answer = Util::ask_for_relation_directionality(
                previous_directionality_in.unwrap_or(""),
                &self.ui,
            );
            //%%Util::ask_for_relation_directionality(previous_directionality_in.as_deref(), &self.ui);
            if directionality_answer.is_none() {
                return Ok(None);
            }
            let x = directionality_answer.unwrap();
            let directionality_str = x.trim().to_uppercase().clone();
            let name_in_reverse_direction_str = Util::ask_for_name_in_reverse_direction(
                directionality_str.clone(),
                max_name_length,
                name.clone(),
                //previous_name_in_reverse_in.as_deref(),
                previous_name_in_reverse_in,
                &self.ui,
            );
            if create_not_update {
                let new_rt: RelationType = RelationType::new2(
                    //%%%%%%%after controller compiles otherwise?: fix this for RelationType maybe
                    //by making the db a &, not owned, in the RT? Can manage the resulting lifetime
                    //issues? OR, how make it owned? how have multiple owned instances of the db,
                    //in general?? Maybe that is what refs are? Or, could have the rc/stuff solve
                    //it, and they are all owned none borrowed...??? (ie, change the db_in parm to
                    //not have a & ?
                    db_in.clone(),
                    None,
                    db_in.borrow_mut().create_relation_type(
                        None,
                        &name,
                        &name_in_reverse_direction_str,
                        &directionality_str,
                    )?,
                )?;
                //%%Ok(Some((new_rt.get_id(), 0)))
                Ok(Some(new_rt.get_id()))
            } else {
                //assertion at top of this method guarantees that the next unwrap() is safe.
                //let rt = existing_entity_in.as_ref().unwrap().as_relation_type().unwrap();
                //%%(test this fix) what is the as_relation_type().unwrap() doing? is good or fix/cmt?
                //was: existingEntityIn.get.asInstanceOf[RelationType].update(name, name_in_reverse_directionStr, directionalityStr)
                //let rt = existing_entity_in.unwrap().as_relation_type().unwrap();
                //There has to be a better way than these next few lines. What am I missing?
                let x = existing_entity_in.borrow();
                let y = (*x).as_ref();
                let z = *y.as_ref().unwrap();
                let entity_id = z.get_id();
                let mut rt = RelationType::new2(db_in.clone(), None, entity_id)?;
                rt.update(
                    None,
                    &name,
                    &name_in_reverse_direction_str,
                    &directionality_str,
                )?;
                //%%Ok(Some((rt.get_id(), 0)))
                Ok(Some(rt.get_id()))
            }
        } else {
            //%%return Errors instead of calling the panic&assert each, in this method? or
            //just such things note4future when there are more users?
            panic!("unexpected value: {}", type_in);
        }
    }

    ///SEE DESCRIPTIVE COMMENT ON askForAndWriteClassAndTemplateEntityName, WHICH APPLIES TO all
    ///such METHODS (see this cmt elsewhere).
    ///The "previous..." parameters are for the already-existing data (ie, when editing not creating).
    pub fn ask_for_name_and_write_entity(
        &self,
        //%%%%%should be here? is needed when doing multi dbs? ck callers.  other, similar places?
        db_in: Rc<RefCell<dyn Database>>,
        type_in: &str,
        existing_entity_in: Rc<RefCell<Option<&mut Entity>>>,       /*= None*/
        previous_name_in: Option<String>,            /*None*/
        previous_directionality_in: Option<&str>,  /*None*/
        previous_name_in_reverse_in: Option<&str>, /*None*/
        class_id_in: Option<i64>,                  /*None*/
        leading_text_in: Option<&str>,             /*None*/
        duplicate_name_probably_ok: bool,          /*false*/
    ) -> Result<Option<Entity>, anyhow::Error> {
        if class_id_in.is_some() {
            assert_eq!(type_in, Util::ENTITY_TYPE);
        }
        let create_not_update = existing_entity_in.borrow().is_none();
        if !create_not_update && type_in == Util::RELATION_TYPE_TYPE {
            if ! previous_directionality_in.is_some() {
                return Err(anyhow!("Unexpected value None for previous_directionality_in."));
            }
        }
        let max_name_length = if type_in == Util::RELATION_TYPE_TYPE {
            RelationType::get_name_length()
        } else if type_in == Util::ENTITY_TYPE {
            Entity::name_length()
        } else {
            return Err(anyhow!("invalid type_in: {}", type_in));
        };
        let example = if type_in == Util::RELATION_TYPE_TYPE {
            " (use 3rd-person verb like \"owns\"--might make output like sentences more consistent later on)"
        } else {
            ""
        };
        //%%%%works?: was: let result = tryAskingAndSaving[(i64, i64)](db_in, Util.string_too_long_error_message(maxNameLength), askAndSave, previousNameIn);
        //%%let result = self.try_asking_and_saving<(i64, i64)>(
        let result = self.try_asking_and_saving(
            db_in.clone(),
            &Util::string_too_long_error_message(max_name_length, Util::TOO_LONG_MESSAGE),
            &Self::ask_and_save,
            previous_name_in,
            type_in,
            max_name_length,
            example,
            duplicate_name_probably_ok,
            leading_text_in,
            existing_entity_in,
            create_not_update,
            previous_name_in_reverse_in,
            previous_directionality_in,
            class_id_in,
        )?;
        //%%result.map(|(id, _)| Entity::new(db_in, id))
        if result.is_none() {
            Ok(None)
        } else {
            //else Some(new Entity(db_in, result.get._1))
            //%%Ok(Some(Entity::new2(db_in, None, result.unwrap().get_id())))
            Ok(Some(Entity::new2(db_in, None, result.unwrap())?))
        }
    }

    /// Call a provided function "ask_and_save_in", which does some work that might return
    /// a specific error (in Scala it was: OmDatabaseException).  If it does that,
    /// let the user know the problem and call askAndSaveIn again.  I.e., allow retrying
    /// if the entered data is bad, instead of crashing the app.
    //%%test this
    fn try_asking_and_saving<F>(
        &self,
        db_in: Rc<RefCell<dyn Database>>,
        error_msg_in: &str,
        ask_and_save_in: &F,
        default_name_in: Option<String>, /*= None*/
        //%%%%%%%%
        type_in: &str,
        max_name_length: u16,
        example: &str,
        duplicate_name_probably_ok: bool,
        leading_text_in: Option<&str>,
        existing_entity_in: Rc<RefCell<Option<&mut Entity>>>,
        create_not_update: bool,
        previous_name_in_reverse_in: Option<&str>,
        previous_directionality_in: Option<&str>,
        class_id_in: Option<i64>,
    ) -> Result<Option<i64>, anyhow::Error>
        //%%%%%%%%
    where
        F: Fn(
            &Self,
            Rc<RefCell<dyn Database>>,
            Option<String>,
            Option<&str>,
            Rc<RefCell<Option<&mut Entity>>>,
            bool,
            &str,
            u16,
            &str,
            bool,
            Option<&str>,
            Option<&str>,
            Option<i64>,
            &mut Option<EntityClass>,
        ) -> Result<Option<i64>, anyhow::Error>,
    {
        let mut entity_class: Option<EntityClass> = match class_id_in {
            None => None,
            Some(class_id) => Some(EntityClass::new2(db_in.clone(), None, class_id)?),
        };
        let intermediate_result: Result<Option<i64>, anyhow::Error> = ask_and_save_in(
            &self,
            db_in.clone(),
            default_name_in.clone(),
        //%%%%%%%%
            leading_text_in,
            existing_entity_in.clone(),
            create_not_update,
            type_in,
            max_name_length,
            example,
            duplicate_name_probably_ok,
            previous_name_in_reverse_in,
            previous_directionality_in,
            class_id_in,
            &mut entity_class,
        );
        match intermediate_result {
            //%%%?:, type_in, max_name_length, example) {
            Ok(result) => Ok(result),
            Err(e) => {
                //In Scala, this used to call ".getCause" on the Throwable until the whole set of
                //causes was included. Seems not needed in Rust?:
                //let cumulative_msg = self.accumulate_msgs(&e.to_string(), &e);
                //if cumulative_msg.contains(Util::TOO_LONG_MESSAGE) {
                if e.to_string().contains(Util::TOO_LONG_MESSAGE) {
                    //%%self.ui.display_text(&format!("{}{}", error_msg_in)); //, cumulative_msg));
                    self.ui.display_text1(&format!("{}", error_msg_in)); //, cumulative_msg));
                    self.try_asking_and_saving(
                        db_in,
                        error_msg_in,
                        &ask_and_save_in,
                        default_name_in,
        //%%%%%%%%
                        type_in,
                        max_name_length,
                        example,
                        duplicate_name_probably_ok,
                        leading_text_in,
                        existing_entity_in,
                        create_not_update,
                        previous_name_in_reverse_in,
                        previous_directionality_in,
                        class_id_in,
                    )
                } else {
                    //Err(anyhow!(e))
                    Err(e)
                }
            }
        }
    }

    /// The parameter class_in should be None only if the call is intended to create;
    /// otherwise it is an edit.
    /// Returns None if user wants out, otherwise returns the new or updated classId and entity_id.
    pub fn ask_for_and_write_class_and_template_entity_name(
        &self,
        db_in: Rc<RefCell<dyn Database>>,
        //%%%%%%%%
        leading_text_in: Option<&str>,
        existing_entity_in: Rc<RefCell<Option<&mut Entity>>>,
        class_in: &mut Option<EntityClass>, /*= None*/
        type_in: &str,
        max_name_length: u16,
        example: &str,
        duplicate_name_probably_ok: bool,
        previous_name_in_reverse_in: Option<&str>,
        previous_directionality_in: Option<&str>,
        class_id_in: Option<i64>,
    ) -> Result<Option<i64>, anyhow::Error> {
        //%%later: convert this to better rust, also looking up how to implement equals for the db.
        //in scala:
        //if classIn.is_defined) {
        //  require(classIn.get.db == db_in)
        //}
        //in rust per claude, needs work:
        //if let Some(ref class) = class_in {
        //  // db_in is required even if classIn is not provided, but if classIn is provided, make sure things are in order:
        //  // (Idea:  check: does scala do a deep equals so it is valid?  also tracked in tasks.)
        //    assert!(Rc::ptr_eq(&class.get_db(), &db_in));
        //}

        let create_not_update = class_in.is_none();
        let name_length = EntityClass::name_length();
        let old_template_name_prompt = if create_not_update {
            String::new()
        } else {
            //let entity_id = class_in.as_ref().unwrap().get_template_entity_id();
            
            //This unwrap() is safe because of the create_not_update check just above.
            let c = class_in.as_mut().unwrap();
            let entity_id = c.get_template_entity_id(None)?;

            let template_entity_name =
                Entity::new2(db_in.clone(), None, entity_id)?.get_name(None)?;
            format!(" (which is currently \"{}\")", template_entity_name)
        };

        //def askAndSave(db_in: Database, defaultNameIn: Option<String>): Option[(i64, i64)] = {
        let ask_and_save_closure = |controller: &Controller,
                                    db_in: Rc<RefCell<dyn Database>>,
                                    default_name: Option<String>,
        //%%%%%%%%
                                    leading_text_in: Option<&str>,
                                    existing_entity_in: Rc<RefCell<Option<&mut Entity>>>,
                                    create_not_update: bool,
                                    type_in: &str,
                                    max_name_length: u16,
                                    example: &str,
                                    duplicate_name_probably_ok: bool,
                                    previous_name_in_reverse_in: Option<&str>,
                                    previous_directionality_in: Option<&str>,
                                    class_id_in: Option<i64>,
                                    class_in: &mut Option<EntityClass>,
                                        |
         -> Result<Option<i64>, anyhow::Error> {
            let prompt_string = format!(
                "Enter class name (up to {} characters; will also be used for its template entity name{}; ESC to cancel): ",
                name_length, old_template_name_prompt);
            let prompt_str = prompt_string.as_str();
            let prompts = vec![prompt_str];
            let name_opt = self
                .ui
                .ask_for_string3(prompts, None, default_name.unwrap_or("".to_string()));
            if name_opt.is_none() {
                return Ok(None);
            }
            let name = name_opt.unwrap().trim().to_string();
            if name.is_empty() {
                return Ok(None);
            }
            let existing_id = class_in.as_ref().map(|c| c.get_id());
            if Util::is_duplication_a_problem(
                //EntityClass::is_duplicate(&*db_in.borrow(), &name, existing_id)?,
                EntityClass::is_duplicate(db_in.clone(), None, &name, existing_id)?,
                false,
                &self.ui,
            ) {
                return Ok(None);
            }
            if create_not_update {
                Ok(Some(
                    db_in
                        .borrow_mut()
                        .create_class_and_its_template_entity(None, &name)?.0,
                ))
            } else {
                //%%?: let entity_id = class_in.as_ref().unwrap().update_class_and_template_entity_name(&name)?;
                //These unwrap()s are safe because of the create_not_update test above.
                let entity_id = class_in.as_mut()
                    .unwrap()
                    .update_class_and_template_entity_name(None, &name)?;
                //%%?: Ok(Some((class_in.as_ref().unwrap().get_id(), entity_id)))
                Ok(Some(class_in.as_ref().unwrap().get_id()/*%%, entity_id)*/))
            }
        };

        //let default_name = class_in.as_ref().map(|c| c.get_name());
        let default_name: Option<String> = match class_in {
            Some(ref mut c) => {
                let name = c.get_name(None)?;
                Some(name.clone())
            },
            None => None,
        };
        //tryAskingAndSaving[(i64, i64)](db_in, Util.string_too_long_error_message(name_length), askAndSave,
        //    if classIn.isEmpty) None else Some(classIn.get.get_name))
        self.try_asking_and_saving(
            db_in,
            &Util::string_too_long_error_message(name_length, ""),
            &ask_and_save_closure,
            default_name,
        //%%%%%%%%
            type_in,
            max_name_length,
            example,
            duplicate_name_probably_ok,
            leading_text_in,
            existing_entity_in,
            create_not_update,
            previous_name_in_reverse_in,
            previous_directionality_in,
            class_id_in,
        )
    }

}
