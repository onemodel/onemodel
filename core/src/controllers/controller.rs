/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003-2004, 2008-2020 inclusive, and 2022-2023 inclusive, Luke A. Call.
    (That copyright statement once said only 2013-2015, until I remembered that much of Controller came from TextUI.scala, and TextUI.java before that.)
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/

use crate::model::database::Database;
use crate::model::postgres::postgresql_database::PostgreSQLDatabase;
use crate::util::Util;
use crate::TextUI;

/// This Controller is for user-interactive things.  The Controller class in the web module is for the REST API.  For shared code that does not fit
/// in those, see struct Util (in util.rs).
///
/// Improvements to this class should START WITH MAKING IT BETTER TESTED (functional testing? integration? see
/// scalatest docs 4 ideas, & maybe use expect or the gnu testing tool that uses expect?), delaying side effects more,
/// shorter methods, other better style?, etc.
///
/// * * * *IMPORTANT * * * * * IMPORTANT* * * * * * *IMPORTANT * * * * * * * IMPORTANT* * * * * * * * *IMPORTANT * * * * * *
/// Don't ever instantiate a Controller from a *test* without passing in username/password parameters, because it will try to log in to the user's
/// default, live Database and run the tests there (ie, they could be destructive)!:
/// %%: How make that better/safer!?--just use the new_* methods below as reminders?
/// * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * *
///
pub struct Controller {
    ui: TextUI,
    force_user_pass_prompt: bool,

    // NOTE: This should *not* be passed around as a parameter to everything, but rather those
    // places in the code should get the DB instance from the
    // entity (or other model object) being processed, to be sure the correct db instance is used.
    db: Box<dyn Database>,
    // putting this in a var instead of recalculating it every time (too frequent) inside find_default_display_entity_id:
    show_public_private_status_preference: Option<bool>,
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
            //%%should panic instead, at all places like this? to get a stack trace and for style?
            //%%OR, only if it is truly something unanticipated? Are there not times when returning a failure is expected?
            //%%should eprintln at other places like this also?
            // ui.display_text1(e.to_string().as_str());
            eprintln!("{}", e.to_string().as_str());
            std::process::exit(1);
        });
        //
        let show_public_private_status_preference: Option<bool> = db.get_user_preference_boolean(
            &None,
            Util::SHOW_PUBLIC_PRIVATE_STATUS_PREFERENCE,
            None,
        )?;
        //%%%temp values:
        // let show_public_private_status_preference: Option<bool> = Some(true);
        // let default_display_entity_id: Option<i64> = localDb.get_user_preference_entity_id(Util::DEFAULT_ENTITY_PREFERENCE);
        let default_display_entity_id: Option<i64> = Some(-9223372036854745151);
        Ok(Controller {
            ui,
            force_user_pass_prompt,
            db,
            show_public_private_status_preference,
            default_display_entity_id,
            move_farther_count: 25,
            move_farthest_count: 50,
        })
    }

    /*
        /** Returns the id and the entity, if they are available from the preferences lookup (id) and then finding that in the db (Entity). */
        fn get_default_entity(&self) -> Option<(i64, Entity)> {
            match self.default_display_entity_id {
                None => None,
                Some(ddei) => {
                    //%%%%
                    let entity: Option<Entity> = Entity::get_entity(&self.db, ddei);
                    match entity {
                        None => None,
                        Some(entity) => {
                            if entity.is_archived() {
                                let msg = format!("The default entity \n    {}: \"{} + "
                                    \"\n" +
                                    "... was found but is archived.  You might run" +
                                    " into problems unless you un-archive it, or choose a different entity to make the default, or display all archived" +
                                    " entities then search for this entity and un-archive it under its Entity Menu options 9, 4.",
                                    entity.get_id(), entity.get_name());
                                let ans = ui.ask_which(Some(vec!(msg)), vec!("Un-archive the default entity now", "Display archived entities"));
                                if ans.is_defined() {
                                    if ans.get == 1 {
                                        entity.unarchive();
                                    } else if ans.get == 2 {
                                        localDb.set_include_archived_entities(true);
                                    }
                                }
                            }
                            Some((entity.get_id(), entity))
                        }
                    }
                }
            }
        }
    */

    pub fn start(&self) {
        // idea: wait for keystroke so they do see the copyright each time. (is also tracked):  make it save their answer 'yes/i agree' or such in the DB,
        // and don't make them press the keystroke again (time-saver)!  See code at top of postgresql_database.rs that puts things in the db at startup: do similarly?
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

        /* %%%%
         // Max id used as default here because it seems the least likely # to be used in the system hence the
         // most likely to cause an error as default by being missing, so the system can respond by prompting
         // the user in some other way for a use.
         if get_default_entity.isEmpty {
           ui.display_text("To get started, you probably want to find or create an " +
                          "entity (such as with your own name, to track information connected to you, contacts, possessions etc, " +
                          "or with the subject of study), then set that or some entity as your default (using its menu).")
         }

         // Explicitly *not* "@tailrec" so user can go "back" to previously viewed entities. See comments below at "def mainMenu" for more on the feature of the
         // user going back. (But: this one currently only ever passes defaultEntity as a parameter, so there is no "back", except what is handled withing
         // mainmenu calling itself.  It seems like if we need to be any more clever we're going to want that stack back....see those same comments below.)
         //
         // The 1st parameter to mainMenu might be a kludge. But it lets us, at startup, go straight to the attributeMenu of the default Entity.  When instead we
         // simply called
         // entityMenu(0,defaultEntity.get) before going into menuLoop, it didn't have the usual context for normal behavior, and caused odd things for the user,
         // like choosing a related entity to view its entity menu showed the default object's entity menu instead, until going into the usual loop and
         // choosing it again. Now we do it w/ the same code path, thus the same behavior, as normally expected.
         def menuLoop(goDirectlyToChoice: Option[Int] = None) {
           //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) w/in this method! (should they be, in this case tho'?)
           //re-checking for the default each time because user can change it.
           new MainMenu(ui, localDb, this).mainMenu(if get_default_entity.isEmpty { None } else { Some(get_default_entity.get._2) },
                                               goDirectlyToChoice)
           menuLoop()
         }
         menuLoop(Some(5))
        %%    */
    }

    /// If the 1st parm is true, the next 2 must be None.
    fn try_db_logins<'a>(
        force_user_pass_prompt: bool,
        ui: &'a TextUI,
        default_username: Option<&String>,
        default_password: Option<&String>,
    ) -> Result<Box<dyn Database>, anyhow::Error> {
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

    fn prompt_for_user_pass_and_login<'a>(ui: &TextUI) -> Result<Box<dyn Database>, anyhow::Error> {
        loop {
            let usr = ui.ask_for_string1(vec!["Username"]);
            match usr {
                None => {
                    //user probably wants out
                    std::process::exit(1);
                }
                Some(username) => {
                    let pwd = ui.ask_for_string4(vec!["Password"], None, "", true);
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
    fn try_other_logins_or_prompt(ui: &TextUI) -> Result<Box<dyn Database>, anyhow::Error> {
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
                let usr = ui.ask_for_string3(vec!["Username"], None, default_username.as_str());
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
                            let pwd = ui.ask_for_string4(vec!["Password"], None, "", true);
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
                                            let msg = format!("Login failed; retrying ({}) to quit if needed):  {}", ui.how_quit(), e.to_string());
                                            ui.display_text2(msg.as_str(), false)
                                        }
                                    }
                                    //%%AND: IN RUST instead of setting to null & doing gc(), could
                                    // look into the "zeroize" and "secrecy" crates for that, per an article
                                    // i just (20221201) read in "this week in Rust" rss feed, "Rust Foundation - Secure App Development with rust's Memory Model", at
                                    //  https://foundation.rust-lang.org/news/secure-app-development-with-rust-s-memory-model/  .
                                    // OR: no need because Rust will clean up anyway? will it be reused or could hang around to be exploited?
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Idea: show_public_private_status_preference, refresh_public_private_status_preference, and find_default_display_entity_id(), feel awkward.
    // Needs something better, but I'm not sure
    // what, at the moment.  It was created this way as a sort of cache because looking it up every time was costly and made the app slow, like when
    // displaying a list of entities (getting the preference every time, to N levels deep), and especially at startup when checking for the default
    // up to N levels deep, among the preferences that can include entities with deep nesting.  So in a related change I made it also not look N levels
    // deep, for preferences.  If you check other places touched by this commit there may be a "shotgun surgery" bad smell here also.
    //Idea: Maybe these should have their cache expire after a period of time (to help when running multiple clients).
    /* %%%%
    // fn refresh_public_private_status_preference() -> Unit {
    //     show_public_private_status_preference = localDb.get_user_preference_boolean(Util::SHOW_PUBLIC_PRIVATE_STATUS_PREFERENCE)
    // }
    //
    // //%%never called? should be? remove or leave4now?
    // fn refresh_default_display_entity_id() /*-> Unit%%*/  {
    //     default_display_entity_id = localDb.get_user_preference_entity_id(Util::DEFAULT_ENTITY_PREFERENCE)
    // }

    fn askForClass(db_in: Database) -> Option<i64> {
        let msg = "CHOOSE ENTITY'S CLASS.  (Press ESC if you don't know or care about this.  Detailed explanation on the class feature will be available " +;
                  "at onemodel.org when this feature is documented more (hopefully at the next release), or ask on the email list.)"
        let result: Option[(IdWrapper, bool, String)] = chooseOrCreateObject(db_in, Some(List[String](msg)), None, None, Util::ENTITY_CLASS_TYPE);
        if result.isEmpty None
        else Some(result.get._1.get_id)
    }

      /** In any given usage, consider whether askForNameAndWriteEntity should be used instead: it is for quick (simpler) creation situations or
        * to just edit the name when the entity already exists, or if the Entity is a RelationType,
        * askForClassInfoAndNameAndCreateEntity (this one) prompts for a class and checks whether it should copy default attributes from the class-defining
        * (template) entity.
        * There is also editEntityName which calls askForNameAndWriteEntity: it checks if the Entity being edited is a RelationType, and if not also checks
        * for whether a group name should be changed at the same time.
        */
        fn askForClassInfoAndNameAndCreateEntity(db_in: Database, classIdIn: Option<i64> = None) -> Option<Entity> {
        let mut newClass = false;
        let classId: Option<i64> =;
          if classIdIn.is_defined {
           classIdIn
          } else {
            newClass = true
            askForClass(db_in)
          }
        let ans: Option<Entity> = askForNameAndWriteEntity(db_in, Util::ENTITY_TYPE, None, None, None, None, classId,;
                                                           Some(if newClass { "DEFINE THE ENTITY:" } else { "" }))
        if ans.is_defined {
          let entity = ans.get;
          // idea: (is also on fix list): this needs to be removed, after evaluating for other side effects, to fix the bug
          // where creating a new relationship, and creating the entity2 in the process, it puts the wrong info
          // on the header for what is being displayed/edited next!: Needs refactoring anyway: this shouldn't be at
          // a low level.
          ui.display_text("Created " + Util::ENTITY_TYPE + ": " + entity.get_name, false);

          defaultAttributeCopying(entity)

          Some(entity)
        } else {
          None
        }
      }

      /**
       * SEE DESCRIPTIVE COMMENT ON askForAndWriteClassAndTemplateEntityName, WHICH APPLIES TO all such METHODS (see this cmt elsewhere).
        *
        * The "previous..." parameters are for the already-existing data (ie, when editing not creating).
        */
        fn askForNameAndWriteEntity(db_in: Database, type_in: String, existingEntityIn: Option<Entity> = None, previousNameIn: Option<String> = None,
                                   previous_directionality_in: Option<String> = None,
                                   previous_name_in_reverse_in: Option<String> = None, classIdIn: Option<i64> = None,
                                   leading_text_in: Option<String> = None, duplicate_name_probably_ok: bool = false) -> Option<Entity> {
        if classIdIn.is_defined { require(type_in == Util::ENTITY_TYPE) }
        let createNotUpdate: bool = existingEntityIn.isEmpty;
        if !createNotUpdate && type_in == Util::RELATION_TYPE_TYPE { require(previous_directionality_in.is_defined) }
        let maxNameLength = {;
          if type_in == Util::RELATION_TYPE_TYPE { model.RelationType.get_name_length
          } else if type_in == Util.ENTITY_TYPE { model.Entity.name_length
          } else { throw new scala.Exception("invalid inType: " + type_in) }
        }
        let example = {;
          if type_in == Util.RELATION_TYPE_TYPE) " (use 3rd-person verb like \"owns\"--might make output like sentences more consistent later on)"
          else ""
        }

        /** 2nd i64 in return value is ignored in this particular case.
          */
        def askAndSave(db_in: Database, defaultNameIn: Option<String> = None): Option[(i64, i64)] = {
          let nameOpt = ui::ask_for_string3(Some(Vec<String>(leading_text_in.getOrElse(""),;
                                                           "Enter " + type_in + " name (up to " + maxNameLength + " characters" + example + "; ESC to cancel)")),
                                        None, defaultNameIn)
          if nameOpt.isEmpty) None
          else {
            let name = nameOpt.get.trim();
            if name.length <= 0) None
            else {
              // idea: this size check might be able to account better for the escaping that's done. Or just keep letting the exception handle it as is already
              // done in the caller of this.
              if name.length > maxNameLength) {
                ui.display_text(Util.string_too_long_error_message(maxNameLength).format(Util.TOO_LONG_MESSAGE) + ".")
                askAndSave(db_in, Some(name))
              } else {
                let selfIdToIgnore: Option<i64> = if existingEntityIn.is_defined) Some(existingEntityIn.get.get_id) else None;
                if Util.is_duplication_a_problem(model.Entity.is_duplicate(db_in, name, selfIdToIgnore), duplicate_name_probably_ok, ui)) None
                else {
                  if type_in == Util.ENTITY_TYPE) {
                    if createNotUpdate) {
                      let newId = model.Entity.create_entity(db_in, name, classIdIn).get_id;
                      Some(newId, 0L)
                    } else {
                      existingEntityIn.get.updateName(name)
                      Some(existingEntityIn.get.get_id, 0L)
                    }
                  } else if type_in == Util.RELATION_TYPE_TYPE) {
                    let ans: Option<String> = Util.ask_for_relation_directionality(previous_directionality_in, ui);
                    if ans.isEmpty) None
                    else {
                      let directionalityStr: String = ans.get.trim().toUpperCase;
                      let name_in_reverse_directionStr = Util.ask_for_name_in_reverse_direction(directionalityStr, maxNameLength, name, previous_name_in_reverse_in, ui);
                      if createNotUpdate) {
                        let newId = new RelationType(db_in, db_in.createRelationType(name, name_in_reverse_directionStr, directionalityStr)).get_id;
                        Some(newId, 0L)
                      } else {
                        existingEntityIn.get.asInstanceOf[RelationType].update(name, name_in_reverse_directionStr, directionalityStr)
                        Some(existingEntityIn.get.get_id, 0L)
                      }
                    }
                  } else throw new scala.Exception("unexpected value: " + type_in)
                }
              }
            }
          }
        }

        let result = tryAskingAndSaving[(i64, i64)](db_in, Util.string_too_long_error_message(maxNameLength), askAndSave, previousNameIn);
        if result.isEmpty) None
        else Some(new Entity(db_in, result.get._1))
      }

      /** Call a provided function (method?) "askAndSaveIn", which does some work that might throw a specific OmDatabaseException.  If it does throw that,
        * let the user know the problem and call askAndSaveIn again.  I.e., allow retrying if the entered data is bad, instead of crashing the app.
        */
        fn tryAskingAndSaving[T](db_in: Database,
                                errorMsgIn: String,
                                askAndSaveIn: (Database, Option<String>) => Option[T],
                                defaultNameIn: Option<String> = None) -> Option[T] {
          /*%%for the try/catch, see
             https://doc.rust-lang.org/std/panic/fn.catch_unwind.html
          ....for ideas?  OR JUST USE ERRORS INSTEAD!
     */
        try {
          askAndSaveIn(db_in, defaultNameIn)
        }
        catch {
          case e: OmDatabaseException =>
            def accumulateMsgs(msgIn: String, t: Throwable): String = {
              if t.getCause == null) {
                t.toString
              } else {
                msgIn + " (" + accumulateMsgs(t.toString, t.getCause) + ")"
              }
            }
            let cumulativeMsg = accumulateMsgs(e.toString, e.getCause);
            if cumulativeMsg.contains(Util.TOO_LONG_MESSAGE)) {
              ui.display_text(errorMsgIn.format(Util.TOO_LONG_MESSAGE) + cumulativeMsg + ".")
              tryAskingAndSaving[T](db_in, errorMsgIn, askAndSaveIn, defaultNameIn)
            } else throw e
        }
      }

      /**
        * @param classIn (1st parameter) should be None only if the call is intended to create; otherwise it is an edit.
        * @return None if user wants out, otherwise returns the new or updated classId and entity_id.
        * */
        fn askForAndWriteClassAndTemplateEntityName(db_in: Database, classIn: Option[EntityClass] = None) -> Option[(i64, i64)] {
        if classIn.is_defined) {
          // db_in is required even if classIn is not provided, but if classIn is provided, make sure things are in order:
          // (Idea:  check: does scala do a deep equals so it is valid?  also tracked in tasks.)
          require(classIn.get.db == db_in)
        }
        let createNotUpdate: bool = classIn.isEmpty;
        let name_length = model.EntityClass.name_length(db_in);
        let oldTemplateNamePrompt = {;
          if createNotUpdate) ""
          else {
            let entity_id = classIn.get.get_template_entity_id;
            let template_entityName = new Entity(db_in, entity_id).get_name;
            " (which is currently \"" + template_entityName + "\")"
          }
        }
        def askAndSave(db_in: Database, defaultNameIn: Option<String>): Option[(i64, i64)] = {
          let nameOpt = ui::ask_for_string3(Some(Array("Enter class name (up to " + name_length + " characters; will also be used for its template entity name" +;
                                                   oldTemplateNamePrompt + "; ESC to cancel): ")),
                                        None, defaultNameIn)
          if nameOpt.isEmpty) None
          else {
            let name = nameOpt.get.trim();
            if name.length() == 0) None
            else {
              if Util.is_duplication_a_problem(EntityClass.is_duplicate(db_in, name, if classIn.isEmpty) None else Some(classIn.get.get_id)),
                                             duplicate_name_probably_ok = false, ui)) {
                None
              }
              else {
                if createNotUpdate) {
                  Some(db_in.createClassAndItsTemplateEntity(name))
                } else {
                  let entity_id: i64 = classIn.get.update_class_and_template_entity_name(name);
                  Some(classIn.get.get_id, entity_id)
                }
              }
            }
          }
        }

        tryAskingAndSaving[(i64, i64)](db_in, Util.string_too_long_error_message(name_length), askAndSave, if classIn.isEmpty) None else Some(classIn.get.get_name))
      }

      /** SEE DESCRIPTIVE COMMENT ON askForAndWriteClassAndTemplateEntityName, WHICH APPLIES TO all such METHODS (see this cmt elsewhere).
        * @return The instance's id, or None if there was a problem or the user wants out.
        * */
        fn askForAndWriteOmInstanceInfo(db_in: Database, oldOmInstanceIn: Option[OmInstance] = None) -> Option<String> {
        let createNotUpdate: bool = oldOmInstanceIn.isEmpty;
        let address_length() = model.OmInstance.address_length();
        def askAndSave(db_in: Database, defaultNameIn: Option<String>): Option<String> = {
          let addressOpt = ui::ask_for_string3(Some(Array("Enter the internet address with optional port of a remote OneModel instance (for " +;
                                                      "example, \"om.example.com:9000\", up to " + address_length() + " characters; ESC to cancel;" +
                                                      " Other examples include (omit commas):  localhost,  127.0.0.1:2345,  ::1 (?)," +
                                                      "  my.example.com:80,  your.example.com:8080  .): ")), None, defaultNameIn)
          if addressOpt.isEmpty) None
          else {
            let address = addressOpt.get.trim();
            if address.length() == 0) None
            else {
              if Util.is_duplication_a_problem(OmInstance.is_duplicate(db_in, address, if oldOmInstanceIn.isEmpty) None else Some(oldOmInstanceIn.get.get_id)),
                                             duplicate_name_probably_ok = false, ui)) {
                None
              } else {
                let restDb = Database.getRestDatabase(address);
                let remoteId: Option<String> = restDb.get_idWithOptionalErrHandling(Some(ui));
                if remoteId.isEmpty) {
                  None
                } else {
                  if createNotUpdate) {
                    OmInstance.create(db_in, remoteId.get, address)
                    remoteId
                  } else {
                    if oldOmInstanceIn.get.get_id == remoteId.get) {
                      oldOmInstanceIn.get.update(address)
                      Some(oldOmInstanceIn.get.get_id)
                    } else {
                      let ans: Option<bool> = ui.ask_yes_no_question("The IDs of the old and new remote instances don't match (old " +;
                                                                     "id/address: " + oldOmInstanceIn.get.get_id + "/" +
                                                                     oldOmInstanceIn.get.get_address + ", new id/address: " +
                                                                     remoteId.get + "/" + address + ".  Instead of updating the old one, you should create a new" +
                                                                     " entry for the new remote instance and then optionally delete this old one." +
                                                                     "  Do you want to create the new entry with this new address, now?")
                      if ans.is_defined && ans.get) {
                        let id: String = OmInstance.create(db_in, remoteId.get, address).get_id;
                        ui.display_text("Created the new entry for \"" + address + "\".  You still have to delete the old one (" + oldOmInstanceIn.get.get_id + "/" +
                                       oldOmInstanceIn.get.get_address + ") if you don't want it to be there.")
                        Some(id)
                      } else {
                        None
                      }
                    }
                  }
                }
              }
            }
          }
        }

        tryAskingAndSaving[String](db_in, Util.string_too_long_error_message(address_length()), askAndSave,
                                   if oldOmInstanceIn.isEmpty) {
                                     None
                                   } else {
                                     Some(oldOmInstanceIn.get.get_address)
                                   })
      }

      /* NOTE: converting the parameters around here from DataHolder to Attribute... means also making the Attribute
      classes writable, and/or
         immutable and recreating them whenever there's a change, but also needing a way to pass around
         partial attribute data in a way that can be shared by code, like return values from the get[AttributeData...]
         methods.
         Need to learn more scala so I can do the equivalent of passing a Tuple without specifying the size in signatures?
       */
      /**
       * @return true if the user made a desired change, false if they just want out.
       */
        fn askForInfoAndUpdateAttribute[T <: AttributeDataHolder](db_in: Database, dhIn: T, askForAttrTypeId: bool, attrType: String,
                                                                 promptForSelectingTypeId: String,
                                                                 getOtherInfoFromUser: (Database, T, Boolean, TextUI) => Option[T],
                                                                 updateTypedAttribute: (T) => Unit) -> bool {
        //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) within this method, below!
        @tailrec def askForInfoAndUpdateAttribute_helper(dhIn: T, attrType: String, promptForTypeId: String): bool = {
          let ans: Option[T] = askForAttributeData[T](db_in, dhIn, askForAttrTypeId, attrType, Some(promptForTypeId),;
                                                      Some(new Entity(db_in, dhIn.attr_type_id).get_name),
                                                      Some(dhIn.attr_type_id), getOtherInfoFromUser, editingIn = true)
          if ans.isEmpty) {
            false
          } else {
            let dhOut: T = ans.get;
            let ans2: Option[Int] = Util.prompt_whether_to_1add_2correct(attrType, ui);

            if ans2.isEmpty) {
              false
            } else if ans2.get == 1) {
              updateTypedAttribute(dhOut)
              true
            }
            else if ans2.get == 2) askForInfoAndUpdateAttribute_helper(dhOut, attrType, promptForTypeId)
            else throw new Exception("unexpected result! should never get here")
          }
        }
        askForInfoAndUpdateAttribute_helper(dhIn, attrType, promptForSelectingTypeId)
      }

      /**
       * @return whether the attribute in question was deleted (or archived)
       */
      @tailrec
      //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) within this method, below!
      final def attributeEditMenu(attributeIn: Attribute): bool = {
        let leading_text: Vec<String> = Array("Attribute: " + attributeIn.get_display_string(0, None, None));
        let mut firstChoices = Array("Edit the attribute type, " +;
                                 (if Util.can_edit_attribute_on_single_line(attributeIn)) "content (single line)," else "") +
                                 " and valid/observed dates",
                                    // for rust equiv of isInstanceOf, mbe see Any.is() or print out the type_id per my ex in ~/proj/learnrust/my_cargo_app/src/main.rx ?
                                 if attributeIn.isInstanceOf[TextAttribute]) "Edit (as multi-line value)" else "(stub)",
                                 if Util.can_edit_attribute_on_single_line(attributeIn)) "Edit the attribute content (single line)" else "(stub)",
                                 "Delete",
                                 "Go to entity representing the type: " + new Entity(attributeIn.db, attributeIn.get_attr_type_id()).get_name)
        if attributeIn.isInstanceOf[FileAttribute]) {
          firstChoices = firstChoices ++ Vec<String>("Export the file")
        }
        let response = ui.ask_which(Some(leading_text), firstChoices);
        if response.isEmpty) false
        else {
          let answer: i32 = response.get;
          if answer == 1) {
            attributeIn match {
              case quantityAttribute: QuantityAttribute =>
                def update_quantity_attribute(dhInOut: QuantityAttributeDataHolder) {
                  quantityAttribute.update(dhInOut.attr_type_id, dhInOut.unitId, dhInOut.number, dhInOut.valid_on_date,
                                           dhInOut.observation_date)
                }
                askForInfoAndUpdateAttribute[QuantityAttributeDataHolder](attributeIn.db,
                                                                          new QuantityAttributeDataHolder(quantityAttribute.get_attr_type_id(),
                                                                                                          quantityAttribute.get_valid_on_date(),
                                                                                                          quantityAttribute.get_observation_date(),
                                                                                                          quantityAttribute.getNumber, quantityAttribute.getUnitId),
                                                                          askForAttrTypeId = true, Util.QUANTITY_TYPE, Util.QUANTITY_TYPE_PROMPT,
                                                                          ask_for_quantity_attribute_numberAndUnit, update_quantity_attribute)
                //force a reread from the DB so it shows the right info on the repeated menu:
                attributeEditMenu(new QuantityAttribute(attributeIn.db, attributeIn.get_id))
              case textAttribute: TextAttribute =>
                def update_text_attribute(dhInOut: TextAttributeDataHolder) {
                  textAttribute.update(dhInOut.attr_type_id, dhInOut.text, dhInOut.valid_on_date, dhInOut.observation_date)
                }
                let textAttributeDH: TextAttributeDataHolder = new TextAttributeDataHolder(textAttribute.get_attr_type_id(), textAttribute.get_valid_on_date(),;
                                                                                           textAttribute.get_observation_date(), textAttribute.get_text)
                askForInfoAndUpdateAttribute[TextAttributeDataHolder](attributeIn.db, textAttributeDH, askForAttrTypeId = true, Util.TEXT_TYPE,
                                                                      "CHOOSE TYPE OF " + Util.TEXT_DESCRIPTION + ":",
                                                                      Util.ask_for_text_attribute_text, update_text_attribute)
                //force a reread from the DB so it shows the right info on the repeated menu:
                attributeEditMenu(new TextAttribute(attributeIn.db, attributeIn.get_id))
              case dateAttribute: DateAttribute =>
                def update_date_attribute(dhInOut: DateAttributeDataHolder) {
                  dateAttribute.update(dhInOut.attr_type_id, dhInOut.date)
                }
                let dateAttributeDH: DateAttributeDataHolder = new DateAttributeDataHolder(dateAttribute.get_attr_type_id(), dateAttribute.get_date);
                askForInfoAndUpdateAttribute[DateAttributeDataHolder](attributeIn.db, dateAttributeDH, askForAttrTypeId = true, Util.DATE_TYPE, "CHOOSE TYPE OF DATE:",
                                                                      Util.ask_for_date_attribute_value, update_date_attribute)
                //force a reread from the DB so it shows the right info on the repeated menu:
                attributeEditMenu(new DateAttribute(attributeIn.db, attributeIn.get_id))
              case boolean_attribute: BooleanAttribute =>
                def update_boolean_attribute(dhInOut: BooleanAttributeDataHolder) {
                  boolean_attribute.update(dhInOut.attr_type_id, dhInOut.boolean, dhInOut.valid_on_date, dhInOut.observation_date)
                }
                let boolean_attributeDH: BooleanAttributeDataHolder = new BooleanAttributeDataHolder(boolean_attribute.get_attr_type_id(), boolean_attribute.get_valid_on_date(),;
                                                                                                    boolean_attribute.get_observation_date(),
                                                                                                    boolean_attribute.get_boolean)
                askForInfoAndUpdateAttribute[BooleanAttributeDataHolder](attributeIn.db, boolean_attributeDH, askForAttrTypeId = true, Util.BOOLEAN_TYPE,
                                                                         "CHOOSE TYPE OF TRUE/FALSE VALUE:", Util.askForBooleanAttributeValue,
                                                                         update_boolean_attribute)
                //force a reread from the DB so it shows the right info on the repeated menu:
                attributeEditMenu(new BooleanAttribute(attributeIn.db, attributeIn.get_id))
              case fa: FileAttribute =>
                def update_file_attribute(dhInOut: FileAttributeDataHolder) {
                  fa.update(Some(dhInOut.attr_type_id), Some(dhInOut.description))
                }
                let fileAttributeDH: FileAttributeDataHolder = new FileAttributeDataHolder(fa.get_attr_type_id(), fa.get_description(), fa.get_original_file_path());
                askForInfoAndUpdateAttribute[FileAttributeDataHolder](attributeIn.db, fileAttributeDH, askForAttrTypeId = true, Util.FILE_TYPE, "CHOOSE TYPE OF FILE:",
                                                                      Util.ask_for_file_attribute_info, update_file_attribute)
                //force a reread from the DB so it shows the right info on the repeated menu:
                attributeEditMenu(new FileAttribute(attributeIn.db, attributeIn.get_id))
              case _ => throw new Exception("Unexpected type: " + attributeIn.getClass.get_name)
            }
          } else if answer == 2 && attributeIn.isInstanceOf[TextAttribute]) {
            let ta = attributeIn.asInstanceOf[TextAttribute];
            let new_content: String = Util.edit_multiline_text(ta.get_text, ui);
            ta.update(ta.get_attr_type_id(), new_content, ta.get_valid_on_date(), ta.get_observation_date())
            //then force a reread from the DB so it shows the right info on the repeated menu:
            attributeEditMenu(new TextAttribute(attributeIn.db, attributeIn.get_id))
          } else if answer == 3 && Util.can_edit_attribute_on_single_line(attributeIn)) {
            editAttributeOnSingleLine(attributeIn)
            false
          } else if answer == 4) {
            let ans = ui.ask_yes_no_question("DELETE this attribute: ARE YOU SURE?");
            if ans.is_defined && ans.get) {
              attributeIn.delete()
              true
            } else {
              ui.display_text("Did not delete attribute.", false);
              attributeEditMenu(attributeIn)
            }
          } else if answer == 5) {
            new EntityMenu(ui, this).entityMenu(new Entity(attributeIn.db, attributeIn.get_attr_type_id()))
            attributeEditMenu(attributeIn)
          } else if answer == 6) {
            if !attributeIn.isInstanceOf[FileAttribute]) throw new Exception("Menu shouldn't have allowed us to get here w/ a type other than FA (" +
                                                                              attributeIn.getClass.get_name + ").")
            let fa: FileAttribute = attributeIn.asInstanceOf[FileAttribute];
            //%%see 1st instance of try {  for rust-specific idea here.
            try {
              // this file should be confirmed by the user as ok to write, even overwriting what is there.
              let file: Option[File] = ui.getExportDestination(fa.get_original_file_path(), fa.get_md5hash());
              if file.is_defined) {
                fa.retrieveContent(file.get)
                ui.display_text("File saved at: " + file.get.getCanonicalPath)
              }
            } catch {
              case e: Exception =>
                let msg: String = Util.throwableToString(e);
                ui.display_text("Failed to export file, due to error: " + msg)
            }
            attributeEditMenu(attributeIn)
          } else {
            ui.display_text("invalid response")
            attributeEditMenu(attributeIn)
          }
        }
      }

      /**
       * @return Whether the user wants just to get out.
       */
        fn editAttributeOnSingleLine(attributeIn: Attribute) -> bool {
        require(Util.can_edit_attribute_on_single_line(attributeIn))

        attributeIn match {
          case quantityAttribute: QuantityAttribute =>
            let num: Option[Float] = Util.ask_for_quantity_attribute_number(quantityAttribute.getNumber, ui);
            if num.is_defined) {
              quantityAttribute.update(quantityAttribute.get_attr_type_id(), quantityAttribute.getUnitId,
                                       num.get,
                                       quantityAttribute.get_valid_on_date(), quantityAttribute.get_observation_date())
            }
            num.isEmpty
          case textAttribute: TextAttribute =>
            let textAttributeDH: TextAttributeDataHolder = new TextAttributeDataHolder(textAttribute.get_attr_type_id(), textAttribute.get_valid_on_date(),;
                                                                                       textAttribute.get_observation_date(), textAttribute.get_text)
            let outDH: Option[TextAttributeDataHolder] = Util.ask_for_text_attribute_text(attributeIn.db, textAttributeDH, editing_in = true, ui);
            if outDH.is_defined) textAttribute.update(outDH.get.attr_type_id, outDH.get.text, outDH.get.valid_on_date, outDH.get.observation_date)
            outDH.isEmpty
          case dateAttribute: DateAttribute =>
            let dateAttributeDH: DateAttributeDataHolder = new DateAttributeDataHolder(dateAttribute.get_attr_type_id(), dateAttribute.get_date);
            let outDH: Option[DateAttributeDataHolder] = Util.ask_for_date_attribute_value(attributeIn.db, dateAttributeDH, editing_in = true, ui);
            if outDH.is_defined) dateAttribute.update(outDH.get.attr_type_id, outDH.get.date)
            outDH.isEmpty
          case boolean_attribute: BooleanAttribute =>
            let boolean_attributeDH: BooleanAttributeDataHolder = new BooleanAttributeDataHolder(boolean_attribute.get_attr_type_id(), boolean_attribute.get_valid_on_date(),;
                                                                                                boolean_attribute.get_observation_date(),
                                                                                                boolean_attribute.get_boolean)
            let outDH: Option[BooleanAttributeDataHolder] = Util.askForBooleanAttributeValue(boolean_attribute.db, boolean_attributeDH, editing_in = true, ui);
            if outDH.is_defined) boolean_attribute.update(outDH.get.attr_type_id, outDH.get.boolean, outDH.get.valid_on_date, outDH.get.observation_date)
            outDH.isEmpty
          case rtle: RelationToLocalEntity =>
            let editedEntity: Option<Entity> = editEntityName(new Entity(rtle.db, rtle.get_related_id2));
            editedEntity.isEmpty
          case rtre: RelationToRemoteEntity =>
            let editedEntity: Option<Entity> = editEntityName(new Entity(rtre.getRemoteDatabase, rtre.get_related_id2));
            editedEntity.isEmpty
          case rtg: RelationToGroup =>
            let editedGroupName: Option<String> = Util::edit_group_name(new Group(rtg.db, rtg.get_group_id), ui);
            editedGroupName.isEmpty
          case _ => throw new scala.Exception("Unexpected type: " + attributeIn.getClass.getCanonicalName)
        }
      }

      /**
       * @return (See addAttribute method.)
       */
        fn askForInfoAndAddAttribute[T <: AttributeDataHolder](db_in: Database, dhIn: T, askForAttrTypeId: bool, attrType: String,
                                                              promptForSelectingTypeId: Option<String>,
                                                              getOtherInfoFromUser: (Database, T, Boolean, TextUI) => Option[T],
                                                              addTypedAttribute: (T) => Option[Attribute]) -> Option[Attribute] {
        let ans: Option[T] = askForAttributeData[T](db_in, dhIn, askForAttrTypeId, attrType, promptForSelectingTypeId,;
                                                    None, None, getOtherInfoFromUser, editingIn = false)
        if ans.is_defined) {
          let dhOut: T = ans.get;
          addTypedAttribute(dhOut)
        } else None
      }

      /**
       * SEE DESCRIPTIVE COMMENT ON askForAndWriteClassAndTemplateEntityName, WHICH APPLIES TO all such METHODS (see this cmt elsewhere).
       *
       * @return None if user wants out.
       */
        fn editEntityName(entity_in: Entity) -> Option<Entity> {
        let editedEntity: Option<Entity> = entity_in match {;
          case relTypeIn: RelationType =>
            let previousNameInReverse: String = relTypeIn.get_name_in_reverse_direction //idea: check: this edits name w/ prefill also?:;
            askForNameAndWriteEntity(entity_in.db, Util.RELATION_TYPE_TYPE, Some(relTypeIn), Some(relTypeIn.get_name), Some(relTypeIn.get_directionality),
                                     if previousNameInReverse == null || previousNameInReverse.trim().isEmpty) None else Some(previousNameInReverse),
                                     None)
          case entity: Entity =>
            let entityNameBeforeEdit: String = entity_in.get_name;
            let editedEntity: Option<Entity> = askForNameAndWriteEntity(entity_in.db, Util.ENTITY_TYPE, Some(entity), Some(entity.get_name), None, None, None);
            if editedEntity.is_defined) {
              let entityNameAfterEdit: String = editedEntity.get.get_name;
              if entityNameBeforeEdit != entityNameAfterEdit) {
                let (_, _, groupId, group_name, moreThanOneAvailable) = editedEntity.get.find_relation_to_and_group;
                if groupId.is_defined && !moreThanOneAvailable) {
                  let attrCount = entity_in.get_attribute_count();
                  // for efficiency, if it's obvious which subgroup's name to change at the same time, offer to do so
                  let defaultAnswer = if attrCount > 1) Some("n") else Some("y");
                  let ans = ui.ask_yes_no_question("There's a single subgroup named \"" + group_name + "\"" +;
                                                (if attrCount > 1) " (***AMONG " + (attrCount - 1) + " OTHER ATTRIBUTES***)" else "") +
                                                "; possibly it and this entity were created at the same time.  Also change" +
                                                " the subgroup's name now to be identical?", defaultAnswer)
                  if ans.is_defined && ans.get) {
                    let group = new Group(entity_in.db, groupId.get);
                    group.update(name_in = Some(entityNameAfterEdit), valid_on_date_inIGNORED4NOW = None, observation_dateInIGNORED4NOW = None)
                  }
                }
              }
            }
            editedEntity
          case _ => throw new Exception("??")
        }
        editedEntity
      }

        fn askForPublicNonpublicStatus(defaultForPrompt: Option<bool>) -> Option<bool> {
        let valueAfterEdit: Option<bool> = ui.ask_yes_no_question("For Public vs. Non-public, enter a yes/no value (or a space" +;
                                                                  " for 'unknown/unspecified'; used e.g. during data export; display preference can be" +
                                                                  " set under main menu / " + Util.MENUTEXT_VIEW_PREFERENCES + ")",
                                                                  if defaultForPrompt.isEmpty) Some("") else if defaultForPrompt.get) Some("y") else Some("n"),
                                                                  allow_blank_answer = true)
        valueAfterEdit
      }

      /// Returns data, or None if user wants to cancel/get out.
      /// @param attrType Constant referring to Attribute subtype, as used by the inObjectType parameter to the chooseOrCreateObject method
      ///                 (ex., Controller.QUANTITY_TYPE).  See comment on that method, for that parm.
      /// The editingIn parameter (I think) being true means we are editing data, not adding new data.
        fn askForAttributeData[T <: AttributeDataHolder](db_in: Database, inoutDH: T, alsoAskForAttrTypeId: bool, attrType: String, attrTypeInputPrompt: Option<String>,
                                                        inPreviousSelectionDesc: Option<String>, inPreviousSelectionId: Option<i64>,
                                                        askForOtherInfo: (Database, T, Boolean, TextUI) => Option[T], editingIn: bool) -> Option[T] {
        let (userWantsOut: bool, attr_type_id: i64, is_remote, remoteKey) = {
          if alsoAskForAttrTypeId) {
            require(attrTypeInputPrompt.is_defined)
            let ans: Option[(IdWrapper, Boolean, String)] = chooseOrCreateObject(db_in, Some(List(attrTypeInputPrompt.get)), inPreviousSelectionDesc,;
                                                                                 inPreviousSelectionId, attrType)
            if ans.isEmpty) {
              (true, 0L, false, "")
            } else {
              (false, ans.get._1.get_id, ans.get._2, ans.get._3)
            }
          } else {
            // maybe not ever reached under current system logic. not certain.
            let (is_remote, remoteKey) = {;
              //noinspection TypeCheckCanBeMatch
              if inoutDH.isInstanceOf[RelationToEntityDataHolder]) {
                (inoutDH.asInstanceOf[RelationToEntityDataHolder].is_remote, inoutDH.asInstanceOf[RelationToEntityDataHolder].remoteInstanceId)
              } else {
                (false, "")
              }
            }
            (false, inoutDH.attr_type_id, is_remote, remoteKey)
          }
        }

        if userWantsOut) {
          None
        } else {
          inoutDH.attr_type_id = attr_type_id
          //noinspection TypeCheckCanBeMatch
          if inoutDH.isInstanceOf[RelationToEntityDataHolder]) {
            inoutDH.asInstanceOf[RelationToEntityDataHolder].is_remote = is_remote
            inoutDH.asInstanceOf[RelationToEntityDataHolder].remoteInstanceId = remoteKey
          }
          let ans2: Option[T] = askForOtherInfo(db_in, inoutDH, editingIn, ui);
          if ans2.isEmpty) None
          else {
            let mut userWantsToCancel = false;
            // (the ide/intellij preferred to have it this way instead of 'if')
            inoutDH match {
              case dhWithVOD: AttributeDataHolderWithVODates =>
                let (valid_on_date: Option<i64>, observation_date: i64, userWantsToCancelInner: bool) =;
                  Util.ask_for_attribute_valid_and_observed_dates(dhWithVOD.valid_on_date, dhWithVOD.observation_date, ui)

                if userWantsToCancelInner) userWantsToCancel = true
                else {
                  dhWithVOD.observation_date = observation_date
                  dhWithVOD.valid_on_date = valid_on_date
                }
              case _ =>
                //do nothing
            }
            if userWantsToCancel) None
            else Some(inoutDH)
          }
        }
      }

      /** Searches for a regex, case-insensitively, & returns the id of an Entity, or None if user wants out.  The parameter 'idToOmitIn' lets us omit
        * (or flag?) an entity if it should be for some reason (like it's the caller/container & doesn't make sense to be in the group, or something).
        *
        * Idea: re attrTypeIn parm, enum/improvement: see comment re inAttrType at beginning of chooseOrCreateObject.
        */
      @tailrec final def findExistingObjectByText(db_in: Database, starting_display_row_index_in: i64 = 0, attrTypeIn: String,
                                                  //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) w/in this method!
                                                  idToOmitIn: Option<i64> = None, regexIn: String): Option[IdWrapper] = {
        let leading_text = List[String]("SEARCH RESULTS: " + Util.PICK_FROM_LIST_PROMPT);
        let choices: Vec<String> = Array(Util.LIST_NEXT_ITEMS_PROMPT);
        let numDisplayableItems = ui.maxColumnarChoicesToDisplayAfter(leading_text.size, choices.length, Util.maxNameLength);

        let objectsToDisplay = attrTypeIn match {;
          case Util.ENTITY_TYPE =>
            db_in.get_matching_entities(starting_display_row_index_in, Some(numDisplayableItems), idToOmitIn, regexIn)
          case Util.GROUP_TYPE =>
            db_in.get_matching_groups(starting_display_row_index_in, Some(numDisplayableItems), idToOmitIn, regexIn)
          case _ =>
            throw new OmException("??")
        }
        if objectsToDisplay.size == 0) {
          ui.display_text("End of list, or none found; starting over from the beginning...")
          if starting_display_row_index_in == 0) None
          else findExistingObjectByText(db_in, 0, attrTypeIn, idToOmitIn, regexIn)
        } else {
          let objectNames: Vec<String> = objectsToDisplay.toArray.map {;
                                                                          case entity: Entity =>
                                                                            let numSubgroupsPrefix: String = getEntityContentSizePrefix(entity);
                                                                            numSubgroupsPrefix + entity.get_archived_status_display_string + entity.get_name
                                                                          case group: Group =>
                                                                            let numSubgroupsPrefix: String = getGroupContentSizePrefix(group.db, group.get_id);
                                                                            numSubgroupsPrefix + group.get_name
                                                                          case x: Any => throw new Exception("unexpected class: " + x.getClass.get_name)
                                                                          case _ => throw new OmException("??")
                                                                        }
          let ans = ui.ask_whichChoiceOrItsAlternate(Some(leading_text.toArray), choices, objectNames);
          if ans.isEmpty) None
          else {
            let (answer, userChoseAlternate: bool) = ans.get;
            if answer == 1 && answer <= choices.length) {
              // (For reason behind " && answer <= choices.size", see comment where it is used in entityMenu.)
              let nextStartingIndex: i64 = starting_display_row_index_in + objectsToDisplay.size;
              findExistingObjectByText(db_in, nextStartingIndex, attrTypeIn, idToOmitIn, regexIn)
            } else if answer > choices.length && answer <= (choices.length + objectsToDisplay.size)) {
              // those in the condition on the previous line are 1-based, not 0-based.
              let index = answer - choices.length - 1;
              let o = objectsToDisplay.get(index);
              if userChoseAlternate) {
                attrTypeIn match {
                  // idea: replace this condition by use of a trait (the type of o, which has get_id), or being smarter with scala's type system. attrTypeIn match {
                  case Util.ENTITY_TYPE =>
                    new EntityMenu(ui, this).entityMenu(o.asInstanceOf[Entity])
                  case Util.GROUP_TYPE =>
                    // for now, picking the first RTG found for this group, until it's clear which of its RTGs to use.
                    // (see also the other locations w/ similar comment!)
                    // (There is probably no point in showing this GroupMenu with RTG info, since which RTG to use was picked arbitrarily, except if
                    // that added info is a convenience, or if it helps the user clean up orphaned data sometimes.)
                    let someRelationToGroups: java.util.ArrayList[RelationToGroup] = o.asInstanceOf[Group].get_containing_relations_to_group(0, Some(1));
                    if someRelationToGroups.size < 1) {
                      ui.display_text(Util.ORPHANED_GROUP_MESSAGE)
                      new GroupMenu(ui, this).groupMenu(o.asInstanceOf[Group], 0, None, containingEntityIn = None)
                    } else {
                      new GroupMenu(ui, this).groupMenu(o.asInstanceOf[Group], 0, Some(someRelationToGroups.get(0)), containingEntityIn = None)
                    }
                  case _ =>
                    throw new OmException("??")
                }
                findExistingObjectByText(db_in, starting_display_row_index_in, attrTypeIn, idToOmitIn, regexIn)
              } else {
                // user typed a letter to select.. (now 0-based); selected a new object and so we return to the previous menu w/ that one displayed & current
                attrTypeIn match {
                  // idea: replace this condition by use of a trait (the type of o, which has get_id), or being smarter with scala's type system. attrTypeIn match {
                  case Util.ENTITY_TYPE =>
                    Some(new IdWrapper(o.asInstanceOf[Entity].get_id))
                  case Util.GROUP_TYPE =>
                    Some(new IdWrapper(o.asInstanceOf[Group].get_id))
                  case _ =>
                    throw new OmException("??")
                }
              }
            } else {
              ui.display_text("unknown choice among secondary list")
              findExistingObjectByText(db_in, starting_display_row_index_in, attrTypeIn, idToOmitIn, regexIn)
            }
          }
        }
      }

      /**
       * @param containingGroupIn lets us omit entities that are already in a group,
       *        i.e. omitting them from the list of entities (e.g. to add to the group), that this method returns.
       *
       * @return None if user wants out, otherwise: a relevant id, a Boolean indicating if the id is for an object in a remote OM instance,
       *         and if the object selected represents the key of a remote instance, that key as a String.
       *
       * Idea: the object_type_in parm: do like in java & make it some kind of enum for type-safety? What's the scala idiom for that? (see also other
       * mentions of object_type_in (or still using old name, inAttrType) for others to fix as well.)
       *
       * Idea: this should be refactored for simplicity, perhaps putting logic now conditional on object_type_in in a trait & types that have it (tracked in tasks).
        */
      /*@tailrec  //idea (and is tracked):  putting this back gets compiler error on line 1218 call to chooseOrCreateObject. */
      final def chooseOrCreateObject(db_in: Database, leading_text_in: Option[List[String]], previousSelectionDescIn: Option<String>,
                                     previousSelectionIdIn: Option<i64>, object_type_in: String, starting_display_row_index_in: i64 = 0,
                                     classIdIn: Option<i64> = None, limit_by_classIn: bool = false,
                                     containingGroupIn: Option<i64> = None,
                                     markPreviousSelectionIn: bool = false,
                                     showOnlyAttributeTypesIn: Option<bool> = None,
                                     quantity_seeks_unit_not_type_in: bool = false
                                     //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) w/in this method! (not
                                     // necessary if calling for a separate object type, but just when intended to ~"start over with the same thing").
                                     ): Option[(IdWrapper, Boolean, String)] = {
        if classIdIn.is_defined) require(object_type_in == Util.ENTITY_TYPE)
        if quantity_seeks_unit_not_type_in) require(object_type_in == Util.QUANTITY_TYPE)
        let entityAndMostAttrTypeNames = Array(Util.ENTITY_TYPE, Util.QUANTITY_TYPE, Util.DATE_TYPE, Util.BOOLEAN_TYPE,;
                                      Util.FILE_TYPE, Util.TEXT_TYPE)
        let evenMoreAttrTypeNames = Array(Util.ENTITY_TYPE, Util.TEXT_TYPE, Util.QUANTITY_TYPE, Util.DATE_TYPE, Util.BOOLEAN_TYPE,;
                                          Util.FILE_TYPE, Util.RELATION_TYPE_TYPE, Util.RELATION_TO_LOCAL_ENTITY_TYPE,
                                          Util.RELATION_TO_GROUP_TYPE)
        let listNextItemsChoiceNum = 1;

        let (numObjectsAvailable: i64, showOnlyAttributeTypes: bool) = {;
          // ** KEEP THESE QUERIES AND CONDITIONS IN SYNC W/ THE COROLLARY ONES 1x ELSEWHERE ! (at similar comment):
          if Util.NON_RELATION_ATTR_TYPE_NAMES.contains(object_type_in)) {
            if showOnlyAttributeTypesIn.isEmpty) {
              let countOfEntitiesUsedAsThisAttrType: i64 = db_in.get_count_of_entities_used_as_attribute_types(object_type_in, quantity_seeks_unit_not_type_in);
              if countOfEntitiesUsedAsThisAttrType > 0L) {
                (countOfEntitiesUsedAsThisAttrType, true)
              } else {
                (db_in.get_entity_count(), false)
              }
            } else if showOnlyAttributeTypesIn.get) {
              (db_in.get_count_of_entities_used_as_attribute_types(object_type_in, quantity_seeks_unit_not_type_in), true)
            } else {
              (db_in.get_entity_count(), false)
            }
          }
          else if object_type_in == Util.ENTITY_TYPE) (db_in.get_entities_only_count(limit_by_classIn, classIdIn, previousSelectionIdIn), false)
          else if Util.RELATION_ATTR_TYPE_NAMES.contains(object_type_in)) (db_in.get_relation_type_count(), false)
          else if object_type_in == Util.ENTITY_CLASS_TYPE) (db_in.get_class_count(), false)
          else if object_type_in == Util.OM_INSTANCE_TYPE) (db_in.get_om_instance_count(), false)
          else throw new Exception("invalid object_type_in: " + object_type_in)
        }

        // Attempt to keep these straight even though the size of the list, hence their option #'s on the menu,
        // is conditional:
        def getChoiceList: (Vec<String>, Int, Int, Int, Int, Int, Int, Int, Int, Int, Int) = {
          let mut keepPreviousSelectionChoiceNum = 1;
          let mut createAttrTypeChoiceNum = 1;
          let mut searchForEntityByNameChoiceNum = 1;
          let mut searchForEntityByIdChoiceNum = 1;
          let mut showJournalChoiceNum = 1;
          let mut swapObjectsToDisplayChoiceNum = 1;
          let mut linkToRemoteInstanceChoiceNum = 1;
          let mut createRelationTypeChoiceNum = 1;
          let mut createClassChoiceNum = 1;
          let mut createInstanceChoiceNum = 1;
          let mut choiceList = Array(Util.LIST_NEXT_ITEMS_PROMPT);
          if previousSelectionDescIn.is_defined) {
            choiceList = choiceList :+ "Keep previous selection (" + previousSelectionDescIn.get + ")."
            keepPreviousSelectionChoiceNum += 1
            // inserted a menu option, so add 1 to all the others' indexes.
            createAttrTypeChoiceNum += 1
            searchForEntityByNameChoiceNum += 1
            searchForEntityByIdChoiceNum += 1
            showJournalChoiceNum += 1
            swapObjectsToDisplayChoiceNum += 1
            linkToRemoteInstanceChoiceNum += 1
            createRelationTypeChoiceNum += 1
            createClassChoiceNum += 1
            createInstanceChoiceNum += 1
          }
          //idea: use match instead of if: can it do || ?
          if entityAndMostAttrTypeNames.contains(object_type_in)) {
            // insert the several other menu options, and add the right # to the index of each.
            choiceList = choiceList :+ Util.MENUTEXT_CREATE_ENTITY_OR_ATTR_TYPE
            createAttrTypeChoiceNum += 1
            choiceList = choiceList :+ "Search for existing entity by name and text attribute content..."
            searchForEntityByNameChoiceNum += 2
            choiceList = choiceList :+ "Search for existing entity by id..."
            searchForEntityByIdChoiceNum += 3
            choiceList = choiceList :+ "Show journal (changed entities) by date range..."
            showJournalChoiceNum += 4
            if showOnlyAttributeTypes) {
              choiceList = choiceList :+ "show all entities " + "(not only those already used as a type of " + object_type_in
            } else {
              choiceList = choiceList :+ "show only entities ALREADY used as a type of " + object_type_in
            }
            swapObjectsToDisplayChoiceNum += 5
            choiceList = choiceList :+ "Link to entity in a separate (REMOTE) OM instance..."
            linkToRemoteInstanceChoiceNum += 6
          } else if Util.RELATION_ATTR_TYPE_NAMES.contains(object_type_in)) {
            // These choice #s are only hit by the conditions below, when they should be...:
            choiceList = choiceList :+ Util::menutext_create_relation_type()
            createRelationTypeChoiceNum += 1
          } else if object_type_in == Util.ENTITY_CLASS_TYPE) {
            choiceList = choiceList :+ "Create new class (template for new entities)"
            createClassChoiceNum += 1
          } else if object_type_in == Util.OM_INSTANCE_TYPE) {
            choiceList = choiceList :+ "Create new OM instance (a remote data store for lookup, linking, etc.)"
            createInstanceChoiceNum += 1
          } else throw new Exception("invalid object_type_in: " + object_type_in)

          (choiceList, keepPreviousSelectionChoiceNum, createAttrTypeChoiceNum, searchForEntityByNameChoiceNum, searchForEntityByIdChoiceNum, showJournalChoiceNum, createRelationTypeChoiceNum, createClassChoiceNum, createInstanceChoiceNum, swapObjectsToDisplayChoiceNum, linkToRemoteInstanceChoiceNum)
        }

        def getLeadTextAndObjectList(choices_in: Vec<String>): (List[String],
          java.util.ArrayList[_ >: RelationType with OmInstance with EntityClass <: Object],
          Vec<String>)
        = {
          let prefix: String = object_type_in match {;
            case Util.ENTITY_TYPE => "ENTITIES: "
            case Util.QUANTITY_TYPE => "QUANTITIES (entities): "
            case Util.DATE_TYPE => "DATE ATTRIBUTES (entities): "
            case Util.BOOLEAN_TYPE => "TRUE/FALSE ATTRIBUTES (entities): "
            case Util.FILE_TYPE => "FILE ATTRIBUTES (entities): "
            case Util.TEXT_TYPE => "TEXT ATTRIBUTES (entities): "
            case Util.RELATION_TYPE_TYPE => "RELATION TYPES: "
            case Util.RELATION_TO_LOCAL_ENTITY_TYPE => "RELATION TYPES: "
            case Util.RELATION_TO_GROUP_TYPE => "RELATION TYPES: "
            case Util.ENTITY_CLASS_TYPE => "CLASSES: "
            case Util.OM_INSTANCE_TYPE => "OneModel INSTANCES: "
            case _ => ""
          }
          let mut leading_text = leading_text_in.getOrElse(List[String](prefix + "Pick from menu, or an item by letter; Alt+<letter> to go to the item & later come back)"));
          let numDisplayableItems = ui.maxColumnarChoicesToDisplayAfter(leading_text.size + 3 /* up to: see more of leading_text below .*/ , choices_in.length,;
                                                                        Util.maxNameLength)
          let objectsToDisplay = {;
            // ** KEEP THESE QUERIES AND CONDITIONS IN SYNC W/ THE COROLLARY ONES 1x ELSEWHERE ! (at similar comment):
            if Util.NON_RELATION_ATTR_TYPE_NAMES.contains(object_type_in)) {
              if showOnlyAttributeTypes) {
                db_in.get_entities_used_as_attribute_types(object_type_in, starting_display_row_index_in, Some(numDisplayableItems), quantity_seeks_unit_not_type_in)
              } else {
                db_in.get_entities(starting_display_row_index_in, Some(numDisplayableItems))
              }
            }
            else if object_type_in == Util.ENTITY_TYPE) db_in.get_entities_only(starting_display_row_index_in, Some(numDisplayableItems), classIdIn, limit_by_classIn,
                                                                            previousSelectionIdIn, containingGroupIn)
            else if Util.RELATION_ATTR_TYPE_NAMES.contains(object_type_in)) {
              db_in.get_relation_types(starting_display_row_index_in, Some(numDisplayableItems)).asInstanceOf[java.util.ArrayList[RelationType]]
            }
            else if object_type_in == Util.ENTITY_CLASS_TYPE) db_in.get_classes(starting_display_row_index_in, Some(numDisplayableItems))
            else if object_type_in == Util.OM_INSTANCE_TYPE) db_in.get_om_instances()
            else throw new Exception("invalid object_type_in: " + object_type_in)
          }
          if objectsToDisplay.size == 0) {
            // IF THIS CHANGES: change the guess at the 1st parameter to maxColumnarChoicesToDisplayAfter, JUST ABOVE!
            let txt: String = "\n\n" + "(None of the needed " + (if object_type_in == Util.RELATION_TYPE_TYPE) "relation types" else "entities") +;
                              " have been created in this model, yet."
            leading_text = leading_text ::: List(txt)
          }
          Util.add_remaining_count_to_prompt(choices_in, objectsToDisplay.size, numObjectsAvailable, starting_display_row_index_in)
          let objectStatusesAndNames: Vec<String> = objectsToDisplay.toArray.map {;
                                                                          case entity: Entity => entity.get_archived_status_display_string + entity.get_name
                                                                          case clazz: EntityClass => clazz.get_name
                                                                          case omInstance: OmInstance => omInstance.get_display_string
                                                                          case x: Any => throw new Exception("unexpected class: " + x.getClass.get_name)
                                                                          case _ => throw new Exception("??")
                                                                        }
          (leading_text, objectsToDisplay, objectStatusesAndNames)
        }

        def getNextStartingObjectIndex(previousListLength: i64, numObjectsAvailableIn: i64): i64 = {
          let index = {;
            let x = starting_display_row_index_in + previousListLength;
            // ask Model for list of obj's w/ count desired & starting index (or "first") (in a sorted map, w/ id's as key, and names)
            //idea: should this just reuse the "totalExisting" value alr calculated in above in getLeadTextAndObjectList just above?
            if x >= numObjectsAvailableIn) {
              ui.display_text("End of list found; starting over from the beginning.")
              0 // start over
            } else x
          }
          index
        }

        let (choices, keepPreviousSelectionChoice, create_entityOrAttrTypeChoice, searchForEntityByNameChoice, searchForEntityByIdChoice, showJournalChoice, createRelationTypeChoice, createClassChoice, createInstanceChoice, swapObjectsToDisplayChoice, linkToRemoteInstanceChoice): (Vec<String>,;
          Int, Int, Int, Int, Int, Int, Int, Int, Int, Int) = getChoiceList

        let (leading_text, objectsToDisplay, statusesAndNames) = getLeadTextAndObjectList(choices);
        let ans = ui.ask_whichChoiceOrItsAlternate(Some(leading_text.toArray), choices, statusesAndNames);

        if ans.isEmpty) None
        else {
          let answer = ans.get._1;
          let userChoseAlternate = ans.get._2;
          if answer == listNextItemsChoiceNum && answer <= choices.length && !userChoseAlternate) {
            // (For reason behind " && answer <= choices.length", see comment where it is used in entityMenu.)
            let index: i64 = getNextStartingObjectIndex(objectsToDisplay.size, numObjectsAvailable);
            chooseOrCreateObject(db_in, leading_text_in, previousSelectionDescIn, previousSelectionIdIn, object_type_in, index, classIdIn, limit_by_classIn,
                                 containingGroupIn, markPreviousSelectionIn, Some(showOnlyAttributeTypes), quantity_seeks_unit_not_type_in)
          } else if answer == keepPreviousSelectionChoice && answer <= choices.length) {
            // Such as if editing several fields on an attribute and doesn't want to change the first one.
            // Not using "get out" option for this because it would exit from a few levels at once and
            // then user wouldn't be able to proceed to other field edits.
            Some(new IdWrapper(previousSelectionIdIn.get), false, "")
          } else if answer == create_entityOrAttrTypeChoice && answer <= choices.length) {
            let e: Option<Entity> = askForClassInfoAndNameAndCreateEntity(db_in, classIdIn);
            if e.isEmpty) {
              None
            } else {
              Some(new IdWrapper(e.get.get_id), false, "")
            }
          } else if answer == searchForEntityByNameChoice && answer <= choices.length) {
            let result = askForNameAndSearchForEntity(db_in);
            if result.isEmpty) {
              None
            } else {
              Some(result.get, false, "")
            }
          } else if answer == searchForEntityByIdChoice && answer <= choices.length) {
            let result = searchById(db_in, Util.ENTITY_TYPE);
            if result.isEmpty) {
              None
            } else {
              Some(result.get, false, "")
            }
          } else if answer == showJournalChoice && answer <= choices.length) {
            // THIS IS CRUDE RIGHT NOW AND DOESN'T ABSTRACT TEXT SCREEN OUTPUT INTO THE UI CLASS very neatly perhaps, BUT IS HELPFUL ANYWAY:
            // ideas:
              // move the lines for this little section, into a separate method, near findExistingObjectByName
              // do something similar (refactoring findExistingObjectByName?) to show the results in a list, but make clear on *each line* what kind of result it is.
              // where going to each letter w/ Alt key does the same: goes 2 that entity so one can see its context, etc.
              // change the "None" returned to be the selected entity, like the little section above does.
              // could keep this text output as an option?
            let yDate = new java.util.Date(System.currentTimeMillis() - (24 * 60 * 60 * 1000));
            let yesterday: String = new java.text.SimpleDateFormat("yyyy-MM-dd").format(yDate);
            let beginDate: Option<i64> = Util.ask_for_date_generic(Some("BEGINNING date in the time range: " + Util.GENERIC_DATE_PROMPT), Some(yesterday), ui);
            if beginDate.isEmpty) None
            else {
              let endDate: Option<i64> = Util.ask_for_date_generic(Some("ENDING date in the time range: " + Util.GENERIC_DATE_PROMPT), None, ui);
              if endDate.isEmpty) None
              else {
                let mut dayCurrentlyShowing: String = "";
                let results: util.ArrayList[(i64, String, i64)] = db_in.find_journal_entries(beginDate.get, endDate.get);
                for (result: (i64, String, i64) <- results) {
                  let date = new java.text.SimpleDateFormat("yyyy-MM-dd").format(result._1);
                  if dayCurrentlyShowing != date) {
                    ui.out.println("\n\nFor: " + date + "------------------")
                    dayCurrentlyShowing = date
                  }
                  let time: String = new java.text.SimpleDateFormat("HH:mm:ss").format(result._1);
                  ui.out.println(time + " " + result._3 + ": " + result._2)
                }
                ui.out.println("\n(For other ~'journal' info, could see other things for the day in question, like email, code commits, or entries made on a" +
                                   " different day in a specific \"journal\" section of OM.)")
                ui.display_text("Scroll back to see more info if needed.  Press any key to continue...")
                None
              }
            }
          } else if answer == swapObjectsToDisplayChoice && entityAndMostAttrTypeNames.contains(object_type_in) && answer <= choices.length) {
            chooseOrCreateObject(db_in, leading_text_in, previousSelectionDescIn, previousSelectionIdIn, object_type_in, 0, classIdIn, limit_by_classIn,
                                 containingGroupIn, markPreviousSelectionIn, Some(!showOnlyAttributeTypes), quantity_seeks_unit_not_type_in)
          } else if answer == linkToRemoteInstanceChoice && entityAndMostAttrTypeNames.contains(object_type_in) && answer <= choices.length) {
            let omInstanceIdOption: Option[(_, _, String)] = chooseOrCreateObject(db_in, None, None, None, Util.OM_INSTANCE_TYPE);
            if omInstanceIdOption.isEmpty) {
              None
            } else {
              let remoteOmInstance = new OmInstance(db_in, omInstanceIdOption.get._3);
              let remoteEntityEntryTypeAnswer = ui.ask_which(leading_text_in = Some(Array("SPECIFY AN ENTITY IN THE REMOTE INSTANCE")),;
                                                            choices_in = Array("Enter an entity id #", "Use the remote site's default entity"))
              if remoteEntityEntryTypeAnswer.isEmpty) {
                None
              } else {
                let restDb = Database.getRestDatabase(remoteOmInstance.get_address);
                let remoteEntityId: Option<i64> = {;
                  if remoteEntityEntryTypeAnswer.get == 1) {
                    let remoteEntityAnswer = ui::ask_for_string2(Some(Array("Enter the remote entity's id # (for example, \"-9223372036854745151\"")),;
                                                             Some(Util.is_numeric), None)
                    if remoteEntityAnswer.isEmpty) None
                    else {
                      let id: String = remoteEntityAnswer.get.trim();
                      if id.length() == 0) None
                      else  Some(id.toLong)
                    }
                  } else if remoteEntityEntryTypeAnswer.get == 2) {
                    let defaultEntityId: Option<i64> = restDb.get_default_entity(Some(ui));
                    if defaultEntityId.isEmpty) None
                    else defaultEntityId
                  } else {
                    None
                  }
                }
                if remoteEntityId.isEmpty) None
                else {
                  let entity_in_json: Option<String> = restDb.getEntityJson_WithOptionalErrHandling(Some(ui), remoteEntityId.get);
                  if entity_in_json.isEmpty) {
                    None
                  } else {
                    let saveEntityAnswer: Option<bool> = ui.ask_yes_no_question("Here is the entity's data: \n" + "======================" +;
                                                                                entity_in_json.get + "\n" + "======================\n" +
                                                                                "So do you want to save a reference to that entity?", Some("y"))
                    if saveEntityAnswer.is_defined && saveEntityAnswer.get) {
                      Some(new IdWrapper(remoteEntityId.get), true, remoteOmInstance.get_id)
                    } else {
                      None
                    }
                  }
                }
              }
            }
          } else if answer == createRelationTypeChoice && Util.RELATION_ATTR_TYPE_NAMES.contains(object_type_in) && answer <= choices.length) {
            let entity: Option<Entity> = askForNameAndWriteEntity(db_in, Util.RELATION_TYPE_TYPE);
            if entity.isEmpty) None
            else Some(new IdWrapper(entity.get.get_id), false, "")
          } else if answer == createClassChoice && object_type_in == Util.ENTITY_CLASS_TYPE && answer <= choices.length) {
            let result: Option[(i64, i64)] = askForAndWriteClassAndTemplateEntityName(db_in);
            if result.isEmpty) None
            else {
              let (classId, entity_id) = result.get;
              let ans = ui.ask_yes_no_question("Do you want to add attributes to the newly created template entity for this class? (These will be used for the " +;
                                            "prompts " +
                                            "and defaults when creating/editing entities in this class).", Some("y"))
              if ans.is_defined && ans.get) {
                new EntityMenu(ui, this).entityMenu(new Entity(db_in, entity_id))
              }
              Some(new IdWrapper(classId), false, "")
            }
          } else if answer == createInstanceChoice && object_type_in == Util.OM_INSTANCE_TYPE && answer <= choices.length) {
            let result: Option<String> = askForAndWriteOmInstanceInfo(db_in);
            if result.isEmpty) {
              None
            } else {
              // using null on next line was easier than the visible alternatives (same in one other place w/ this comment)
              Some(null, false, result.get)
            }
          } else if answer > choices.length && answer <= (choices.length + objectsToDisplay.size)) {
            // those in the condition on the previous line are 1-based, not 0-based.
            let index = answer - choices.length - 1;
            // user typed a letter to select.. (now 0-based)
            // user selected a new object and so we return to the previous menu w/ that one displayed & current
            let o = objectsToDisplay.get(index);
            //if "text,quantity,entity,date,boolean,file,relationtype".contains(attrTypeIn)) {
            //i.e., if attrTypeIn == Controller.TEXT_TYPE || (= any of the other types...)):
            if userChoseAlternate) {
              object_type_in match {
                // idea: replace this condition by use of a trait (the type of o, which has get_id), or being smarter with scala's type system. attrTypeIn match {
                case Util.ENTITY_TYPE =>
                  new EntityMenu(ui, this).entityMenu(o.asInstanceOf[Entity])
                case _ =>
                  // (choosing a group doesn't call this, it calls chooseOrCreateGroup)
                  throw new OmException("not yet implemented")
              }
              chooseOrCreateObject(db_in, leading_text_in, previousSelectionDescIn, previousSelectionIdIn, object_type_in,
                                   starting_display_row_index_in, classIdIn, limit_by_classIn,
                                   containingGroupIn, markPreviousSelectionIn, Some(showOnlyAttributeTypes), quantity_seeks_unit_not_type_in)
            } else {
              if evenMoreAttrTypeNames.contains(object_type_in)) Some(o.asInstanceOf[Entity].get_id_wrapper, false, "")
              else if object_type_in == Util.ENTITY_CLASS_TYPE) Some(o.asInstanceOf[EntityClass].get_id_wrapper,false,  "")
              // using null on next line was easier than the visible alternatives (same in one other place w/ this comment)
              else if object_type_in == Util.OM_INSTANCE_TYPE) Some(null, false, o.asInstanceOf[OmInstance].get_id)
              else throw new Exception("invalid object_type_in: " + object_type_in)
            }
          } else {
            ui.display_text("unknown response in chooseOrCreateObject")
            chooseOrCreateObject(db_in, leading_text_in, previousSelectionDescIn, previousSelectionIdIn, object_type_in, starting_display_row_index_in, classIdIn,
                                 limit_by_classIn, containingGroupIn, markPreviousSelectionIn, Some(showOnlyAttributeTypes), quantity_seeks_unit_not_type_in)
          }
        }
      }

        fn askForNameAndSearchForEntity(db_in: Database) -> Option[IdWrapper] {
        let ans = ui::ask_for_string1(Some(Array(Util.entity_or_group_name_sql_search_prompt(Util.ENTITY_TYPE))));
        if ans.isEmpty) {
          None
        } else {
          // Allow relation to self (eg, picking self as 2nd part of a RelationToLocalEntity), so None in 3nd parm.
          let e: Option[IdWrapper] = findExistingObjectByText(db_in, 0, Util.ENTITY_TYPE, None, ans.get);
          if e.isEmpty) None
          else Some(new IdWrapper(e.get.get_id))
        }
      }

        fn searchById(db_in: Database, type_name_in: String) -> Option[IdWrapper] {
        require(type_name_in == Util.ENTITY_TYPE || type_name_in == Util.GROUP_TYPE)
        let ans = ui::ask_for_string1(Some(Array("Enter the " + type_name_in + " ID to search for:")));
        if ans.isEmpty) {
          None
        } else {
          // it's a long:
          let idString: String = ans.get;
          if !Util.is_numeric(idString)) {
            ui.display_text("Invalid ID format.  An ID is a numeric value between " + Database.min_id_value + " and " + Database.max_id_value)
            None
          } else {
            // (BTW, do allow relation to self, ex., picking self as 2nd part of a RelationToLocalEntity.)
            // (Also, the call to entity_key_exists should here include archived entities so the user can find out if the one
            // needed is archived, even if the hard way.)
            if (type_name_in == Util.ENTITY_TYPE && db_in.entity_key_exists(idString.toLong)) ||
                (type_name_in == Util.GROUP_TYPE && db_in.group_key_exists(idString.toLong))) {
              Some(new IdWrapper(idString.toLong))
            } else {
              ui.display_text("The " + type_name_in + " ID " + ans.get + " was not found in the database.")
              None
            }
          }
        }
      }

      /** Returns None if user wants to cancel. */
        fn ask_for_quantity_attribute_numberAndUnit(db_in: Database, dhIn: QuantityAttributeDataHolder, editingIn: bool, ui: TextUI) -> Option[QuantityAttributeDataHolder] {
        let outDH: QuantityAttributeDataHolder = dhIn;
        let leading_text: List[String] = List("SELECT A *UNIT* FOR THIS QUANTITY (i.e., centimeters, or quarts; ESC or blank to cancel):");
        let previousSelectionDesc = if editingIn) Some(new Entity(db_in, dhIn.unitId).get_name) else None;
        let previousSelectionId = if editingIn) Some(dhIn.unitId) else None;
        let unitSelection: Option[(IdWrapper, _, _)] = chooseOrCreateObject(db_in, Some(leading_text), previousSelectionDesc, previousSelectionId,;
                                                                            Util.QUANTITY_TYPE, quantity_seeks_unit_not_type_in = true)
        if unitSelection.isEmpty) {
          ui.display_text("Blank, so assuming you want to cancel; if not come back & add again.", false);
          None
        } else {
          outDH.unitId = unitSelection.get._1.get_id
          let ans: Option[Float] = Util.ask_for_quantity_attribute_number(outDH.number, ui);
          if ans.isEmpty) None
          else {
            outDH.number = ans.get
            Some(outDH)
          }
        }
      }

      /** Returns None if user wants to cancel. */
        fn askForRelToGroupInfo(db_in: Database, dhIn: RelationToGroupDataHolder, editing_inUNUSEDForNOW: bool = false,
                               uiIn: TextUI) -> Option[RelationToGroupDataHolder] {
        let outDH = dhIn;

        let groupSelection = chooseOrCreateGroup(db_in, Some(List("SELECT GROUP FOR THIS RELATION")));
        let groupId: Option<i64> = {;
          if groupSelection.isEmpty) {
            uiIn.display_text("Blank, so assuming you want to cancel; if not come back & add again.", false);
            None
          } else Some[i64](groupSelection.get.get_id)
        }

        if groupId.isEmpty) None
        else {
          outDH.groupId = groupId.get
          Some(outDH)
        }
      }

      /** Returns the id of a Group, or None if user wants out.  The parameter 'containingGroupIn' lets us omit entities that are already in a group,
        * i.e. omitting them from the list of entities (e.g. to add to the group), that this method returns.
        */
      @tailrec final def chooseOrCreateGroup(db_in: Database, leading_text_in: Option[List[String]], starting_display_row_index_in: i64 = 0,
                                             //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) w/in this method!
                                             containingGroupIn: Option<i64> = None /*ie group to omit from pick list*/): Option[IdWrapper] = {
        let totalExisting: i64 = db_in.get_group_count();
        def getNextStartingObjectIndex(currentListLength: i64): i64 = {
          let x = starting_display_row_index_in + currentListLength;
          if x >= totalExisting) {
            ui.display_text("End of list found; starting over from the beginning.")
            0 // start over
          } else x
        }
        let mut leading_text = leading_text_in.getOrElse(List[String](Util.PICK_FROM_LIST_PROMPT));
        let choicesPreAdjustment: Vec<String> = Array("List next items",;
                                                        "Create new group (aka RelationToGroup)",
                                                        "Search for existing group by name...",
                                                        "Search for existing group by id...")
        let numDisplayableItems = ui.maxColumnarChoicesToDisplayAfter(leading_text.size, choicesPreAdjustment.length, Util.maxNameLength);
        let objectsToDisplay = db_in.get_groups(starting_display_row_index_in, Some(numDisplayableItems), containingGroupIn);
        if objectsToDisplay.size == 0) {
          let txt: String = "\n\n" + "(None of the needed groups have been created in this model, yet.";
          leading_text = leading_text ::: List(txt)
        }
        let choices = Util.add_remaining_count_to_prompt(choicesPreAdjustment, objectsToDisplay.size, totalExisting, starting_display_row_index_in);
        let objectNames: Vec<String> = objectsToDisplay.toArray.map {;
                                                                        case group: Group => group.get_name
                                                                        case x: Any => throw new Exception("unexpected class: " + x.getClass.get_name)
                                                                        case _ => throw new Exception("??")
                                                                      }
        let ans = ui.ask_whichChoiceOrItsAlternate(Some(leading_text.toArray), choices, objectNames);
        if ans.isEmpty) None
        else {
          let answer = ans.get._1;
          let userChoseAlternate = ans.get._2;
          if answer == 1 && answer <= choices.length) {
            // (For reason behind " && answer <= choices.size", see comment where it is used in entityMenu.)
            let nextStartingIndex: i64 = getNextStartingObjectIndex(objectsToDisplay.size);
            chooseOrCreateGroup(db_in, leading_text_in, nextStartingIndex, containingGroupIn)
          } else if answer == 2 && answer <= choices.length) {
            let ans = ui::ask_for_string1(Some(Array(Util.RELATION_TO_GROUP_NAME_PROMPT)));
            if ans.isEmpty || ans.get.trim.length() == 0) None
            else {
              let name = ans.get;
              let ans2 = ui.ask_yes_no_question("Should this group allow entities with mixed classes? (Usually not desirable: doing so means losing some " +;
                                             "conveniences such as scripts and assisted data entry.)", Some("n"))
              if ans2.isEmpty) None
              else {
                let mixedClassesAllowed = ans2.get;
                let new_group_id = db_in.create_group(name, mixedClassesAllowed);
                Some(new IdWrapper(new_group_id))
              }
            }
          } else if answer == 3 && answer <= choices.length) {
            let ans = ui::ask_for_string1(Some(Array(Util.entity_or_group_name_sql_search_prompt(Util.GROUP_TYPE))));
            if ans.isEmpty) None
            else {
              // Allow relation to self, so None in 2nd parm.
              let g: Option[IdWrapper] = findExistingObjectByText(db_in, 0, Util.GROUP_TYPE, None, ans.get);
              if g.isEmpty) None
              else Some(new IdWrapper(g.get.get_id))
            }
          } else if answer == 4 && answer <= choices.length) {
            searchById(db_in, Util.GROUP_TYPE)
          } else if answer > choices.length && answer <= (choices.length + objectsToDisplay.size)) {
            // those in that^ condition are 1-based, not 0-based.
            let index = answer - choices.length - 1;
            let o = objectsToDisplay.get(index);
            if userChoseAlternate) {
              // for now, picking the first RTG found for this group, until it's clear which of its RTGs to use.
              // (see also the other locations w/ similar comment!)
              let someRelationToGroups: java.util.ArrayList[RelationToGroup] = o.asInstanceOf[Group].get_containing_relations_to_group(0, Some(1));
              new GroupMenu(ui, this).groupMenu(new Group(db_in, someRelationToGroups.get(0).get_group_id), 0, Some(someRelationToGroups.get(0)),
                                                    containingEntityIn = None)
              chooseOrCreateGroup(db_in, leading_text_in, starting_display_row_index_in, containingGroupIn)
            } else {
              // user typed a letter to select.. (now 0-based); selected a new object and so we return to the previous menu w/ that one displayed & current
              Some(new IdWrapper(o.get_id))
            }
          } else {
            ui.display_text("unknown response in findExistingObjectByText")
            chooseOrCreateGroup(db_in, leading_text_in, starting_display_row_index_in, containingGroupIn)
          }
        }
      }

      /** Returns None if user wants to cancel. */
        fn askForRelationEntityIdNumber2(db_in: Database, dhIn: RelationToEntityDataHolder, editing_in: bool, uiIn: TextUI) -> Option[RelationToEntityDataHolder] {
        let previousSelectionDesc = {;
          if !editing_in) None
          else Some(new Entity(db_in, dhIn.entity_id2).get_name)
        }
        let previousSelectionId = {;
          if !editing_in) None
          else Some(dhIn.entity_id2)
        }
        let selection: Option[(IdWrapper, Boolean, String)] = chooseOrCreateObject(db_in, Some(List("SELECT OTHER (RELATED) ENTITY FOR THIS RELATION")),;
                                                                                   previousSelectionDesc, previousSelectionId, Util.ENTITY_TYPE)
        if selection.isEmpty) None
        else {
          let outDH = dhIn;
          let id: i64 = selection.get._1.get_id;
          outDH.entity_id2 = id
          outDH.is_remote = selection.get._2
          outDH.remoteInstanceId = selection.get._3
          Some(outDH)
        }
      }

        fn goToEntityOrItsSoleGroupsMenu(userSelection: Entity, relationToGroupIn: Option[RelationToGroup] = None,
                                        containingGroupIn: Option[Group] = None) -> (Option<Entity>, Option<i64>, Boolean) {
        let (rtgId, rtId, groupId, _, moreThanOneAvailable) = userSelection.find_relation_to_and_group;
        let subEntitySelected: Option<Entity> = None;
        if groupId.is_defined && !moreThanOneAvailable && userSelection.get_attribute_count() == 1) {
          // In quick menu, for efficiency of some work like brainstorming, if it's obvious which subgroup to go to, just go there.
          // We DON'T want @tailrec on this method for this call, so that we can ESC back to the current menu & list! (so what balance/best? Maybe move this
          // to its own method, so it doesn't try to tail optimize it?)  See also the comment with 'tailrec', mentioning why to have it, above.
          new QuickGroupMenu(ui, this).quickGroupMenu(new Group(userSelection.db, groupId.get),
                                                          0,
                                                          Some(new RelationToGroup(userSelection.db, rtgId.get, userSelection.get_id, rtId.get, groupId.get)),
                                                          callingMenusRtgIn = relationToGroupIn,
                                                          //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s)
                                                          // w/in this method!
                                                          containingEntityIn = Some(userSelection))
        } else {
          new EntityMenu(ui, this).entityMenu(userSelection, containingGroupIn = containingGroupIn)
        }
        (subEntitySelected, groupId, moreThanOneAvailable)
      }

      /** see comments for Entity.getContentSizePrefix. */
        fn getGroupContentSizePrefix(db_in: Database, groupId: i64) -> String {
        let grpSize = db_in.get_group_size(groupId, 1);
        if grpSize == 0) ""
        else ">"
      }

      /** Shows ">" in front of an entity or group if it contains exactly one attribute or a subgroup which has at least one entry; shows ">>" if contains
        * multiple subgroups or attributes, and "" if contains no subgroups or the one subgroup is empty.
        * Idea: this might better be handled in the textui class instead, and the same for all the other color stuff.
        */
        fn getEntityContentSizePrefix(entity_in: Entity) -> String {
        // attrCount counts groups also, so account for the overlap in the below.
        let attrCount = entity_in.get_attribute_count();
        // This is to not show that an entity contains more things (">" prefix...) if it only has one group which has no *non-archived* entities:
        let hasOneEmptyGroup: bool = {;
          let num_groups: i64 = entity_in.get_relation_to_group_count;
          if num_groups != 1) false
          else {
            let (_, _, gid: Option<i64>, _, moreAvailable) = entity_in.find_relation_to_and_group;
            if gid.isEmpty || moreAvailable) throw new OmException("Found " + (if gid.isEmpty) 0 else ">1") + " but by the earlier checks, " +
                                                                            "there should be exactly one group in entity " + entity_in.get_id + " .")
            let groupSize = entity_in.db.get_group_size(gid.get, 1);
            groupSize == 0
          }
        }
        let subgroupsCountPrefix: String = {;
          if attrCount == 0 || (attrCount == 1 && hasOneEmptyGroup)) ""
          else if attrCount == 1) ">"
          else ">>"
        }
        subgroupsCountPrefix
      }

        fn add_entityToGroup(group_in: Group) -> Option<i64> {
        let new_entity_id: Option<i64> = {;
          if !group_in.get_mixed_classes_allowed) {
            if group_in.get_size() == 0) {
              // adding 1st entity to this group, so:
              let leading_text = List("ADD ENTITY TO A GROUP (**whose class will set the group's enforced class, even if 'None'**):");
              let id_wrapper: Option[(IdWrapper, _, _)] = chooseOrCreateObject(group_in.db, Some(leading_text), None, None, Util.ENTITY_TYPE,;
                                                                      containingGroupIn = Some(group_in.get_id))
              if id_wrapper.is_defined) {
                group_in.add_entity(id_wrapper.get._1.get_id)
                Some(id_wrapper.get._1.get_id)
              } else None
            } else {
              // it's not the 1st entry in the group, so add an entity using the same class as those previously added (or None as case may be).
              let entityClassInUse: Option<i64> = group_in.get_class_id;
              let id_wrapper: Option[(IdWrapper, _, _)] = chooseOrCreateObject(group_in.db, None, None, None, Util.ENTITY_TYPE, 0, entityClassInUse,;
                                                                              limit_by_classIn = true, containingGroupIn = Some(group_in.get_id))
              if id_wrapper.isEmpty) None
              else {
                let entity_id = id_wrapper.get._1.get_id;
                //%%see 1st instance of try {  for rust-specific idea here.
                try {
                  group_in.add_entity(entity_id)
                  Some(entity_id)
                } catch {
                  case e: Exception =>
                    if e.getMessage.contains(Database.MIXED_CLASSES_EXCEPTION)) {
                      let oldClass: String = if entityClassInUse.isEmpty) {;
                        "(none)"
                      } else {
                        new EntityClass(group_in.db, entityClassInUse.get).get_display_string
                      }
                      let newClassId = new Entity(group_in.db, entity_id).get_class_id;
                      let newClass: String =;
                        if newClassId.isEmpty || entityClassInUse.isEmpty) "(none)"
                        else {
                          let ec = new EntityClass(group_in.db, entityClassInUse.get);
                          ec.get_display_string
                        }
                      ui.display_text("Adding an entity with class '" + newClass + "' to a group that doesn't allow mixed classes, " +
                                     "and which already has an entity with class '" + oldClass + "' generates an error. The program should have prevented this by" +
                                     " only showing entities with a matching class, but in any case the mismatched entity was not added to the group.")
                      None
                    } else throw e
                }
              }
            }
          } else {
            let leading_text = List("ADD ENTITY TO A (mixed-class) GROUP");
            let id_wrapper: Option[(IdWrapper, _, _)] = chooseOrCreateObject(group_in.db, Some(leading_text), None, None, Util.ENTITY_TYPE,;
                                                                    containingGroupIn = Some(group_in.get_id))
            if id_wrapper.is_defined) {
              group_in.add_entity(id_wrapper.get._1.get_id)
              Some(id_wrapper.get._1.get_id)
            } else None
          }
        }

        new_entity_id
      }

        fn chooseAmongEntities(containingEntities: util.ArrayList[(i64, Entity)]) -> Option<Entity> {
        let leading_text = List[String]("Pick from menu, or an entity by letter");
        let choices: Vec<String> = Array(Util.LIST_NEXT_ITEMS_PROMPT);
        //(see comments at similar location in EntityMenu, as of this writing on line 288)
        let containingEntitiesNamesWithRelTypes: Vec<String> = containingEntities.toArray.map {;
                                                                                                  case rel_type_idAndEntity: (i64, Entity) =>
                                                                                                    let rel_type_id: i64 = rel_type_idAndEntity._1;
                                                                                                    let entity: Entity = rel_type_idAndEntity._2;
                                                                                                    let relTypeName: String = {;
                                                                                                      let relType = new RelationType(entity.db, rel_type_id);
                                                                                                      relType.get_archived_status_display_string + relType.get_name
                                                                                                    }
                                                                                                    "the entity \"" + entity.get_archived_status_display_string +
                                                                                                    entity.get_name + "\" " + relTypeName + " this group"
                                                                                                  // other possible displays:
                                                                                                  //1) entity.get_name + " - " + relTypeName + " this
                                                                                                  // group"
                                                                                                  //2) "entity " + entityName + " " +
                                                                                                  //rtg.get_display_string(maxNameLength, None, Some(rt))
                                                                                                  case _ => throw new OmException("??")
                                                                                                }
        let ans = ui.ask_which(Some(leading_text.toArray), choices, containingEntitiesNamesWithRelTypes);
        if ans.isEmpty) None
        else {
          let answer = ans.get;
          if answer == 1 && answer <= choices.length) {
            // see comment above
            ui.display_text("not yet implemented")
            None
          } else if answer > choices.length && answer <= (choices.length + containingEntities.size)) {
            // those in the condition on the previous line are 1-based, not 0-based.
            let index = answer - choices.length - 1;
            // user typed a letter to select.. (now 0-based); selected a new object and so we return to the previous menu w/ that one displayed &
            // current
            Some(containingEntities.get(index)._2)
          } else {
            ui.display_text("unknown response")
            None
          }
        }
      }

        fn get_public_status_display_string(entity_in: Entity) -> String {
        //idea: maybe this (logic) knowledge really belongs in the TextUI class. (As some others, probably.)
        if show_public_private_status_preference.getOrElse(false)) {
          entity_in.get_public_status_display_string_with_color(blank_if_unset = false)
        } else {
          ""
        }
      }

      /**
       * @param attrFormIn Contains the result of passing the right Controller.<string constant> to db.get_attribute_form_id (SEE ALSO COMMENTS IN
       *                   EntityMenu.addAttribute which passes in "other" form_ids).  BUT, there are also cases
       *                   where it is a # higher than those found in db.get_attribute_form_id, and in that case is handled specially here.
       * @return None if user wants out (or attrFormIn parm was an abortive mistake?), and the created Attribute if successful.
       */
        fn addAttribute(entity_in: Entity, startingAttributeIndexIn: Int, attrFormIn: Int, attr_type_id_in: Option<i64>) -> Option[Attribute] {
        let (attr_type_id: i64, askForAttrTypeId: bool) = {;
          if attr_type_id_in.is_defined) {
            (attr_type_id_in.get, false)
          } else {
            (0L, true)
          }
        }
        if attrFormIn == Database.get_attribute_form_id(Util.QUANTITY_TYPE)) {
          def add_quantity_attribute(dhIn: QuantityAttributeDataHolder): Option[QuantityAttribute] = {
            Some(entity_in.add_quantity_attribute(dhIn.attr_type_id, dhIn.unitId, dhIn.number, None, dhIn.valid_on_date, dhIn.observation_date))
          }
          askForInfoAndAddAttribute[QuantityAttributeDataHolder](entity_in.db, new QuantityAttributeDataHolder(attr_type_id, None, System.currentTimeMillis(), 0, 0),
                                                                 askForAttrTypeId, Util.QUANTITY_TYPE,
                                                                 Some(Util.QUANTITY_TYPE_PROMPT), ask_for_quantity_attribute_numberAndUnit, add_quantity_attribute)
        } else if attrFormIn == Database.get_attribute_form_id(Util.DATE_TYPE)) {
          def add_date_attribute(dhIn: DateAttributeDataHolder): Option[DateAttribute] = {
            Some(entity_in.add_date_attribute(dhIn.attr_type_id, dhIn.date))
          }
          askForInfoAndAddAttribute[DateAttributeDataHolder](entity_in.db, new DateAttributeDataHolder(attr_type_id, 0), askForAttrTypeId, Util.DATE_TYPE,
                                                             Some("SELECT TYPE OF DATE: "), Util.ask_for_date_attribute_value, add_date_attribute)
        } else if attrFormIn == Database.get_attribute_form_id(Util.BOOLEAN_TYPE)) {
          def add_boolean_attribute(dhIn: BooleanAttributeDataHolder): Option[BooleanAttribute] = {
            Some(entity_in.add_boolean_attribute(dhIn.attr_type_id, dhIn.boolean, None))
          }
          askForInfoAndAddAttribute[BooleanAttributeDataHolder](entity_in.db, new BooleanAttributeDataHolder(attr_type_id, None, System.currentTimeMillis(), false),
                                                                askForAttrTypeId,
                                                                Util.BOOLEAN_TYPE, Some("SELECT TYPE OF TRUE/FALSE VALUE: "),  Util.askForBooleanAttributeValue,
                                                                add_boolean_attribute)
        } else if attrFormIn == Database.get_attribute_form_id(Util.FILE_TYPE)) {
          def add_file_attribute(dhIn: FileAttributeDataHolder): Option[FileAttribute] = {
            Some(entity_in.add_file_attribute(dhIn.attr_type_id, dhIn.description, new File(dhIn.original_file_path)))
          }
          let result: Option[FileAttribute] = askForInfoAndAddAttribute[FileAttributeDataHolder](entity_in.db, new FileAttributeDataHolder(attr_type_id, "", ""),;
                                                                                                 askForAttrTypeId, Util.FILE_TYPE,
                                                                                                 Some("SELECT TYPE OF FILE: "), Util.ask_for_file_attribute_info,
                                                                                                 add_file_attribute).asInstanceOf[Option[FileAttribute]]
          if result.is_defined) {
            let ans = ui.ask_yes_no_question("Document successfully added. Do you want to DELETE the local copy (at " + result.get.get_original_file_path() + " ?");
            if ans.is_defined && ans.get) {
              if !new File(result.get.get_original_file_path()).delete()) {
                ui.display_text("Unable to delete file at that location; reason unknown.  You could check the permissions.")
              }
            }
          }
          result
        } else if attrFormIn == Database.get_attribute_form_id(Util.TEXT_TYPE)) {
          def add_text_attribute(dhIn: TextAttributeDataHolder): Option[TextAttribute] = {
            Some(entity_in.add_text_attribute(dhIn.attr_type_id, dhIn.text, None, dhIn.valid_on_date, dhIn.observation_date))
          }
          askForInfoAndAddAttribute[TextAttributeDataHolder](entity_in.db, new TextAttributeDataHolder(attr_type_id, None, System.currentTimeMillis(), ""),
                                                             askForAttrTypeId, Util.TEXT_TYPE,
                                                             Some("SELECT TYPE OF " + Util.TEXT_DESCRIPTION + ": "), Util.ask_for_text_attribute_text, add_text_attribute)
        } else if attrFormIn == Database.get_attribute_form_id(Util.RELATION_TO_LOCAL_ENTITY_TYPE)) {
          //(This is in a condition that says "...LOCAL..." but is also for RELATION_TO_REMOTE_ENTITY_TYPE.  See caller for details if interested.)
          def addRelationToEntity(dhIn: RelationToEntityDataHolder): Option[AttributeWithValidAndObservedDates] = {
            let relation = {;
              if dhIn.is_remote) {
                entity_in.addRelationToRemoteEntity(dhIn.attr_type_id, dhIn.entity_id2, None, dhIn.valid_on_date, dhIn.observation_date, dhIn.remoteInstanceId)
              } else {
                entity_in.addRelationToLocalEntity(dhIn.attr_type_id, dhIn.entity_id2, None, dhIn.valid_on_date, dhIn.observation_date)
              }
            }
            Some(relation)
          }
          askForInfoAndAddAttribute[RelationToEntityDataHolder](entity_in.db, new RelationToEntityDataHolder(attr_type_id, None, System.currentTimeMillis(),
                                                                                                             0, false, ""),
                                                                askForAttrTypeId, Util.RELATION_TYPE_TYPE,
                                                                Some("CREATE OR SELECT RELATION TYPE: (" + Util.REL_TYPE_EXAMPLES + ")"),
                                                                askForRelationEntityIdNumber2, addRelationToEntity)
        } else if attrFormIn == 100) {
          // re "100": see javadoc comments above re attrFormIn
          let eId: Option[IdWrapper] = askForNameAndSearchForEntity(entity_in.db);
          if eId.is_defined) {
            Some(entity_in.add_has_RelationToLocalEntity(eId.get.get_id, None, System.currentTimeMillis))
          } else {
            None
          }
        } else if attrFormIn == Database.get_attribute_form_id(Util.RELATION_TO_GROUP_TYPE)) {
          def addRelationToGroup(dhIn: RelationToGroupDataHolder): Option[RelationToGroup] = {
            require(dhIn.entity_id == entity_in.get_id)
            let newRTG: RelationToGroup = entity_in.addRelationToGroup(dhIn.attr_type_id, dhIn.groupId, None, dhIn.valid_on_date, dhIn.observation_date);
            Some(newRTG)
          }
          let result: Option[Attribute] = askForInfoAndAddAttribute[RelationToGroupDataHolder](entity_in.db,;
                                                                                               new RelationToGroupDataHolder(entity_in.get_id, attr_type_id, 0,
                                                                                                                             None, System.currentTimeMillis()),
                                                                                               askForAttrTypeId, Util.RELATION_TYPE_TYPE,
                                                                                               Some("CREATE OR SELECT RELATION TYPE: (" +
                                                                                                    Util.REL_TYPE_EXAMPLES + ")" +
                                                                                                    ".\n" + "(Does anyone see a specific " +
                                                                                                    "reason to keep asking for these dates?)"),
                                                                                               askForRelToGroupInfo, addRelationToGroup)
          if result.isEmpty) {
            None
          } else {
            let newRtg = result.get.asInstanceOf[RelationToGroup];
            new QuickGroupMenu(ui, this).quickGroupMenu(new Group(entity_in.db, newRtg.get_group_id), 0, Some(newRtg), None, containingEntityIn = Some(entity_in))
            // user could have deleted the new result: check that before returning it as something to act upon:
            if entity_in.db.relation_to_group_key_exists(newRtg.get_id)) {
              result
            } else {
              None
            }
          }
        } else if attrFormIn == 101  /*re "101": an "external web page"; for details see comments etc at javadoc above for attrFormIn.*/) {
          let newEntityName: Option<String> = ui::ask_for_string1(Some(Array {"Enter a name (or description) for this web page or other URI"}));
          if newEntityName.isEmpty || newEntityName.get.isEmpty) return None

          let ans1 = ui.ask_which(Some(Vec<String>("Do you want to enter the URI via the keyboard (typing or directly pasting), or" +;
                                                    " have OM pull directly from the clipboard (faster sometimes)?")),
                                                    Array("keyboard", "clipboard"))
          if ans1.isEmpty) return None
          let keyboardOrClipboard1 = ans1.get;
          let uri: String = if keyboardOrClipboard1 == 1) {;
            let text = ui::ask_for_string1(Some(Array("Enter the URI:")));
            if text.isEmpty || text.get.isEmpty) return None else text.get
          } else {
            let uriReady = ui.ask_yes_no_question("Put the url on the system clipboard, then Enter to continue (or hit ESC or answer 'n' to get out)", Some("y"));
            if uriReady.isEmpty || !uriReady.get) return None
            Util.get_clipboard_content
          }

          let ans2 = ui.ask_which(Some(Vec<String>("Do you want to enter a quote from it, via the keyboard (typing or directly pasting) or" +;
                                                    " have OM pull directly from the clipboard (faster sometimes, especially if " +
                                                    " it's multi-line)? Or, ESC to not enter a quote. (Tip: if it is a whole file, just put in" +
                                                    " a few characters from the keyboard, then go back and edit as multi-line to put in all.)")),
                                 Array("keyboard", "clipboard"))
          let quote: Option<String> = if ans2.isEmpty) {;
            None
          } else {
            let keyboardOrClipboard2 = ans2.get;
            if keyboardOrClipboard2 == 1) {
              let text = ui::ask_for_string1(Some(Array("Enter the quote")));
              if text.isEmpty || text.get.isEmpty) return None else text
            } else {
              let clip = ui.ask_yes_no_question("Put a quote on the system clipboard, then Enter to continue (or answer 'n' to get out)", Some("y"));
              if clip.isEmpty || !clip.get) return None
              Some(Util.get_clipboard_content)
            }
          }
          let quote_info = if quote.isEmpty) "" else "For this text: \n  " + quote.get + "\n...and, ";

          let proceedAnswer = ui.ask_yes_no_question(quote_info + "...for this name & URI:\n  " + newEntityName.get + "\n  " + uri + "" +;
                                                  "\n...: do you want to save them?", Some("y"))
          if proceedAnswer.isEmpty || !proceedAnswer.get) return None

          //NOTE: the attr_type_id parm is ignored here since it is always a particular one for URIs:
          let (newEntity: Entity, new_rte: RelationToLocalEntity) = entity_in.add_uri_entity_with_uri_attribute(newEntityName.get, uri, System.currentTimeMillis(),;
                                                                                              entity_in.get_public, caller_manages_transactions_in = false, quote)
          new EntityMenu(ui, this).entityMenu(newEntity, containingRelationToEntityIn = Some(new_rte))
          // user could have deleted the new result: check that before returning it as something to act upon:
          if entity_in.db.relationToLocalentity_key_exists(new_rte.get_id) && entity_in.db.entity_key_exists(newEntity.get_id)) {
            Some(new_rte)
          } else {
            None
          }
        } else {
          ui.display_text("invalid response")
          None
        }
      }

        fn defaultAttributeCopying(targetEntityIn: Entity, attributeTuplesIn: Option[Array[(i64, Attribute)]] = None) -> Unit {
        if shouldTryAddingDefaultAttributes(targetEntityIn)) {
          let attributeTuples: Array[(i64, Attribute)] = {;
            if attributeTuplesIn.is_defined) attributeTuplesIn.get
            else targetEntityIn.get_sorted_attributes(only_public_entities_in = false)._1
          }
          let template_entity: Option<Entity> = {;
            let templateId: Option<i64> = targetEntityIn.get_class_template_entity_id;
            if templateId.isEmpty) {
              None
            } else {
              Some(new Entity(targetEntityIn.db, templateId.get))
            }
          }
          let templateAttributesToCopy: ArrayBuffer[Attribute] = getMissingAttributes(template_entity, attributeTuples);
          copyAndEditAttributes(targetEntityIn, templateAttributesToCopy)
        }
      }

        fn copyAndEditAttributes(entity_in: Entity, templateAttributesToCopyIn: ArrayBuffer[Attribute]) -> Unit {
        // userWantsOut is used like a break statement below: could be replaced with a functional idiom (see link to stackoverflow somewhere in the code).
        let mut escCounter = 0;
        let mut userWantsOut = false;

        fn checkIfExiting(escCounterIn: Int, attributeCounterIn: Int, numAttributes: Int) -> Int {
          let mut escCounterLocal = escCounterIn + 1;
          if escCounterLocal > 3 && attributeCounterIn < numAttributes /* <, so we don't ask when done anyway. */) {
            let outAnswer = ui.ask_yes_no_question("Stop checking/adding attributes?", Some(""));
            require(outAnswer.is_defined, "Unexpected behavior: meant to make user answer here.")
            if outAnswer.get) {
              userWantsOut = true
            } else {
              escCounterLocal = 0
            }
          }
          escCounterLocal
        }

        let mut askAboutRteEveryTime: Option<bool> = None;
        let mut (allCopy: bool, allCreateOrSearch: bool, allKeepReference: bool) = (false, false, false);
        let mut attrCounter = 0;
        for (attributeFromTemplate: Attribute <- templateAttributesToCopyIn) {
          attrCounter += 1
          if !userWantsOut) {
            let wait_for_keystroke: bool = {;
              attributeFromTemplate match {
                case a: RelationToLocalEntity => true
                case a: RelationToRemoteEntity => true
                case _ => false
              }
            }
            def promptToEditAttributeCopy() {
              ui.display_text("Edit the copied " + Database.get_attribute_form_name(attributeFromTemplate.get_form_id) + " \"" +
                             attributeFromTemplate.get_display_string(0, None, None, simplify = true) + "\", from the template entity (ESC to abort):",
                             wait_for_keystroke)
            }
            let newAttribute: Option[Attribute] = {;
              attributeFromTemplate match {
                case templateAttribute: QuantityAttribute =>
                  promptToEditAttributeCopy()
                  Some(entity_in.add_quantity_attribute(templateAttribute.get_attr_type_id(), templateAttribute.getUnitId, templateAttribute.getNumber,
                                                     Some(templateAttribute.get_sorting_index)))
                case templateAttribute: DateAttribute =>
                  promptToEditAttributeCopy()
                  Some(entity_in.add_date_attribute(templateAttribute.get_attr_type_id(), templateAttribute.get_date, Some(templateAttribute.get_sorting_index)))
                case templateAttribute: BooleanAttribute =>
                  promptToEditAttributeCopy()
                  Some(entity_in.add_boolean_attribute(templateAttribute.get_attr_type_id(), templateAttribute.get_boolean, Some(templateAttribute.get_sorting_index)))
                case templateAttribute: FileAttribute =>
                  ui.display_text("You can add a FileAttribute manually afterwards for this attribute.  Maybe it can be automated " +
                                 "more, when use cases for this part are more clear.")
                  None
                case templateAttribute: TextAttribute =>
                  promptToEditAttributeCopy()
                  Some(entity_in.add_text_attribute(templateAttribute.get_attr_type_id(), templateAttribute.get_text, Some(templateAttribute.get_sorting_index)))
                case templateAttribute: RelationToLocalEntity =>
                  let (new_rte, askEveryTime) = copyAndEditRelationToEntity(entity_in, templateAttribute, askAboutRteEveryTime);
                  askAboutRteEveryTime = askEveryTime
                  new_rte
                case templateAttribute: RelationToRemoteEntity =>
                  let (new_rte, askEveryTime) = copyAndEditRelationToEntity(entity_in, templateAttribute, askAboutRteEveryTime);
                  askAboutRteEveryTime = askEveryTime
                  new_rte
                case templateAttribute: RelationToGroup =>
                  promptToEditAttributeCopy()
                  let templateGroup = templateAttribute.getGroup;
                  let (_, newRTG: RelationToGroup) = entity_in.addGroupAndRelationToGroup(templateAttribute.get_attr_type_id(), templateGroup.get_name,;
                                                                                         templateGroup.get_mixed_classes_allowed, None,
                                                                                         System.currentTimeMillis(), Some(templateAttribute.get_sorting_index))
                  Some(newRTG)
                case _ => throw new OmException("Unexpected type: " + attributeFromTemplate.getClass.getCanonicalName)
              }
            }
            if newAttribute.isEmpty) {
              escCounter = checkIfExiting(escCounter, attrCounter, templateAttributesToCopyIn.size)
            } else {
              // (Not re-editing if it is a RTE  because it was edited just above as part of the initial attribute creation step.)
              if ! (newAttribute.get.isInstanceOf[RelationToLocalEntity] || newAttribute.get.isInstanceOf[RelationToRemoteEntity])) {
                let exitedOneEditLine: bool = editAttributeOnSingleLine(newAttribute.get);
                if exitedOneEditLine) {
                  // That includes a "never mind" intention on the last one added (just above), so:
                  newAttribute.get.delete()
                  escCounter = checkIfExiting(escCounter, attrCounter, templateAttributesToCopyIn.size)
                }
              }
            }
          }
        }
        def copyAndEditRelationToEntity(entity_in: Entity, relationToEntityAttributeFromTemplateIn: Attribute,
                                        askEveryTimeIn: Option<bool> = None): (Option[Attribute], Option<bool>) = {
          require(relationToEntityAttributeFromTemplateIn.isInstanceOf[RelationToLocalEntity] ||
                  relationToEntityAttributeFromTemplateIn.isInstanceOf[RelationToRemoteEntity])
          let choice1text = "Copy the template entity, editing its name (**MOST LIKELY CHOICE)";
          let copyFromTemplateAndEditNameChoiceNum = 1;
          let choice2text = "Create a new entity or search for an existing one for this purpose";
          let createOrSearchForEntityChoiceNum = 2;
          let choice3text = "Keep a reference to the same entity as in the template (least likely choice)";
          let keepSameReferenceAsInTemplateChoiceNum = 3;

          let mut askEveryTime: Option<bool> = None;
          askEveryTime = {
            if askEveryTimeIn.is_defined) {
              askEveryTimeIn
            } else {
              let howRTEsLeadingText: Vec<String> = Array("The template has relations to entities.  How would you like the equivalent to be provided" +;
                                                            " for this new entity being created?")
              let howHandleRTEsChoices = Vec<String>("For ALL entity relations being added: " + choice1text,;
                                                       "For ALL entity relations being added: " + choice2text,
                                                       "For ALL entity relations being added: " + choice3text,
                                                       "Ask for each relation to entity being created from the template")
              let howHandleRTEsResponse = ui.ask_which(Some(howRTEsLeadingText), howHandleRTEsChoices);
              if howHandleRTEsResponse.is_defined) {
                if howHandleRTEsResponse.get == 1) {
                  allCopy = true
                  Some(false)
                } else if howHandleRTEsResponse.get == 2) {
                  allCreateOrSearch = true
                  Some(false)
                } else if howHandleRTEsResponse.get == 3) {
                  allKeepReference = true
                  Some(false)
                } else if howHandleRTEsResponse.get == 4) {
                  Some(true)
                } else {
                  ui.display_text("Unexpected answer: " + howHandleRTEsResponse.get)
                  None
                }
              } else {
                None
              }
            }
          }
          if askEveryTime.isEmpty) {
            (None, askEveryTime)
          } else {
            let howCopyRteResponse: Option[Int] = {;
              if askEveryTime.get) {
                let whichRteLeadingText: Vec<String> = Array("The template has a templateAttribute which is a relation to an entity named \"" +;
                                                               relationToEntityAttributeFromTemplateIn.get_display_string(0, None, None, simplify = true) +
                                                               "\": how would you like the equivalent to be provided for this new entity being created?" +
                                                               " (0/ESC to just skip this one for now)")
                let whichRTEChoices = Vec<String>(choice1text, choice2text, choice3text);
                ui.ask_which(Some(whichRteLeadingText), whichRTEChoices)
              } else {
                None
              }
            }
            if askEveryTime.get && howCopyRteResponse.isEmpty) {
              (None, askEveryTime)
            } else {
              let relatedId2: i64 = {;
                //noinspection TypeCheckCanBeMatch
                if relationToEntityAttributeFromTemplateIn.isInstanceOf[RelationToRemoteEntity]) {
                  relationToEntityAttributeFromTemplateIn.asInstanceOf[RelationToRemoteEntity].get_related_id2
                } else if relationToEntityAttributeFromTemplateIn.isInstanceOf[RelationToLocalEntity]) {
                  relationToEntityAttributeFromTemplateIn.asInstanceOf[RelationToLocalEntity].get_related_id2
                } else {
                  throw new OmException("Unexpected type: " + relationToEntityAttributeFromTemplateIn.getClass.getCanonicalName)
                }
              }

              if allCopy || (howCopyRteResponse.is_defined && howCopyRteResponse.get == copyFromTemplateAndEditNameChoiceNum)) {
                let currentOrRemoteDbForRelatedEntity = Database.currentOrRemoteDb(relationToEntityAttributeFromTemplateIn,;
                                                                                   relationToEntityAttributeFromTemplateIn.db)
                let templatesRelatedEntity: Entity = new Entity(currentOrRemoteDbForRelatedEntity, relatedId2);
                let oldName: String = templatesRelatedEntity.get_name;
                let newEntity: Option<Entity> = {;
                  //noinspection TypeCheckCanBeMatch
                  if relationToEntityAttributeFromTemplateIn.isInstanceOf[RelationToLocalEntity]) {
                    askForNameAndWriteEntity(entity_in.db, Util.ENTITY_TYPE, None, Some(oldName), None, None, templatesRelatedEntity.get_class_id,
                                             Some("EDIT THE " + "ENTITY NAME:"), duplicate_name_probably_ok = true)
                  } else if relationToEntityAttributeFromTemplateIn.isInstanceOf[RelationToRemoteEntity]) {
                    let e = askForNameAndWriteEntity(entity_in.db, Util.ENTITY_TYPE, None, Some(oldName), None, None, None,;
                                             Some("EDIT THE ENTITY NAME:"), duplicate_name_probably_ok = true)
                    if e.is_defined && templatesRelatedEntity.get_class_id.is_defined) {
                      let remoteClassId: i64 = templatesRelatedEntity.get_class_id.get;
                      let remoteClassName: String = new EntityClass(currentOrRemoteDbForRelatedEntity, remoteClassId).get_name;
                      ui.display_text("Note: Did not write a class on the new entity to match that from the remote entity, until some kind of synchronization " +
                                     "of classes across OM instances is in place.  (Idea: interim solution could be to match simply by name if " +
                                     "there is a match, with user confirmation, or user selection if multiple matches.  The class " +
                                     "in the remote instance is: " + remoteClassId + ": " + remoteClassName)
                    }
                    e
                  } else throw new OmException("unexpected type: " + relationToEntityAttributeFromTemplateIn.getClass.getCanonicalName)
                }
                if newEntity.isEmpty) {
                  (None, askEveryTime)
                } else {
                  newEntity.get.updateNewEntriesStickToTop(templatesRelatedEntity.get_new_entries_stick_to_top)
                  let newRTLE = Some(entity_in.addRelationToLocalEntity(relationToEntityAttributeFromTemplateIn.get_attr_type_id(), newEntity.get.get_id,;
                                                         Some(relationToEntityAttributeFromTemplateIn.get_sorting_index)))
                  (newRTLE, askEveryTime)
                }
              } else if allCreateOrSearch || (howCopyRteResponse.is_defined && howCopyRteResponse.get == createOrSearchForEntityChoiceNum)) {
                let rteDh = new RelationToEntityDataHolder(relationToEntityAttributeFromTemplateIn.get_attr_type_id(), None, System.currentTimeMillis(), 0, false, "");
                let dh: Option[RelationToEntityDataHolder] = askForRelationEntityIdNumber2(entity_in.db, rteDh, editing_in = false, ui);
                if dh.is_defined) {
      //            let relation = entity_in.addRelationToEntity(dh.get.attr_type_id, dh.get.entity_id2, Some(relationToEntityAttributeFromTemplateIn.get_sorting_index),;
      //                                                        dh.get.valid_on_date, dh.get.observation_date,
      //                                                        dh.get.is_remote, if !dh.get.is_remote) None else Some(dh.get.remoteInstanceId))
                  if dh.get.is_remote) {
                    let rtre = entity_in.addRelationToRemoteEntity(dh.get.attr_type_id, dh.get.entity_id2, Some(relationToEntityAttributeFromTemplateIn.get_sorting_index),;
                                                                  dh.get.valid_on_date, dh.get.observation_date, dh.get.remoteInstanceId)
                    (Some(rtre), askEveryTime)
                  } else {
                    let rtle = entity_in.addRelationToLocalEntity(dh.get.attr_type_id, dh.get.entity_id2, Some(relationToEntityAttributeFromTemplateIn.get_sorting_index),;
                                                                 dh.get.valid_on_date, dh.get.observation_date)
                    (Some(rtle), askEveryTime)
                  }
                } else {
                  (None, askEveryTime)
                }
              } else if allKeepReference || (howCopyRteResponse.is_defined && howCopyRteResponse.get == keepSameReferenceAsInTemplateChoiceNum)) {
                let relation = {;
                  if relationToEntityAttributeFromTemplateIn.db.is_remote) {
                    entity_in.addRelationToRemoteEntity(relationToEntityAttributeFromTemplateIn.get_attr_type_id(), relatedId2,
                                                       Some(relationToEntityAttributeFromTemplateIn.get_sorting_index), None, System.currentTimeMillis(),
                                                       relationToEntityAttributeFromTemplateIn.asInstanceOf[RelationToRemoteEntity].getRemoteInstanceId)
                  } else {
                    entity_in.addRelationToLocalEntity(relationToEntityAttributeFromTemplateIn.get_attr_type_id(), relatedId2,
                                                      Some(relationToEntityAttributeFromTemplateIn.get_sorting_index), None, System.currentTimeMillis())
                  }
                }
                (Some(relation), askEveryTime)
              } else {
                ui.display_text("Unexpected answer: " + allCopy + "/" + allCreateOrSearch + "/" + allKeepReference + "/" + askEveryTime.getOrElse(None) +
                               howCopyRteResponse.getOrElse(None))
                (None, askEveryTime)
              }
            }
          }
        }
      }

        fn getMissingAttributes(classTemplateEntityIn: Option<Entity>, existingAttributeTuplesIn: Array[(i64, Attribute)]) -> ArrayBuffer[Attribute] {
        let templateAttributesToSuggestCopying: ArrayBuffer[Attribute] = {;
          // This determines which attributes from the template entity (or "pattern" or "class-defining entity") are not found on this entity, so they can
          // be added if the user wishes.
          let attributesToSuggestCopying_workingCopy: ArrayBuffer[Attribute] = new ArrayBuffer();
          if classTemplateEntityIn.is_defined) {
            // ("cde" in name means "classDefiningEntity" (aka template))
            let (cde_attributeTuples: Array[(i64, Attribute)], _) = classTemplateEntityIn.get.get_sorted_attributes(only_public_entities_in = false);
            for (cde_attributeTuple <- cde_attributeTuples) {
              let mut attributeTypeFoundOnEntity = false;
              let cde_attribute = cde_attributeTuple._2;
              for (attributeTuple <- existingAttributeTuplesIn) {
                if !attributeTypeFoundOnEntity) {
                  let cde_typeId: i64 = cde_attribute.get_attr_type_id();
                  let typeId = attributeTuple._2.get_attr_type_id();
                  // This is a very imperfect check.  Perhaps this is a motive to use more descriptive relation types in template entities.
                  let existingAttributeStringContainsTemplateString: bool = {;
                    attributeTuple._2.get_display_string(0, None, None, simplify = true).contains(cde_attribute.get_display_string(0, None, None, simplify = true))
                  }
                  if cde_typeId == typeId && existingAttributeStringContainsTemplateString) {
                    attributeTypeFoundOnEntity = true
                  }
                }
              }
              if !attributeTypeFoundOnEntity) {
                attributesToSuggestCopying_workingCopy.append(cde_attribute)
              }
            }
          }
          attributesToSuggestCopying_workingCopy
        }
        templateAttributesToSuggestCopying
      }

        fn shouldTryAddingDefaultAttributes(entity_in: Entity) -> bool {
        if entity_in.get_class_id.isEmpty) {
          false
        } else {
          let createAttributes: Option<bool> = new EntityClass(entity_in.db, entity_in.get_class_id.get).get_create_default_attributes;
          if createAttributes.is_defined) {
            createAttributes.get
          } else {
            if entity_in.get_class_template_entity_id.isEmpty) {
              false
            } else {
              let attrCount = new Entity(entity_in.db, entity_in.get_class_template_entity_id.get).get_attribute_count();
              if attrCount == 0) {
                false
              } else {
                let addAttributesAnswer = ui.ask_yes_no_question("Add attributes to this entity as found on the class-defining entity (template)?",;
                                                              Some("y"), allow_blank_answer = true)
                addAttributesAnswer.is_defined && addAttributesAnswer.get
              }
            }
          }
        }
      }
    */
}

/*  %%
package org.onemodel.core.controllers

import java.io._
import java.util

import org.onemodel.core._
import org.onemodel.core.model._

import scala.annotation.tailrec
import scala.collection.JavaConversions._
import scala.collection.mutable.ArrayBuffer

*/
