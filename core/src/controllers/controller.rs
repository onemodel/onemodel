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

use crate::model::postgresql_database::PostgreSQLDatabase;
// use crate::model::database::Database;
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
    //%%$%
    // default_username: Option<String>,
    // default_password: Option<String>,

    // NOTE: This should *not* be passed around as a parameter to everything, but rather those
    // places in the code should get the DB instance from the
    // entity (or other model object) being processed, to be sure the correct db instance is used.
    db: PostgreSQLDatabase, //%%$%%use "dyn" here & below at warnings?:
    /*%%$%%other qs looking at now:
        consider whether to use "Database" in place of "PostgreSQLDatabase" in places below for correctness? or wait/YAGNI?
        doing what 4docs say re trait...?
        What are the above/below T doing (the err msgs)
        what triggered compiler asking for "dyn"?
        doesnt dyn have to be a Box or reference?
     */
    move_farther_count: i32,
    move_farthest_count: i32,
}

impl Controller {
    pub fn new_for_non_tests(ui: TextUI, force_user_pass_prompt: bool, default_username: Option<&String>, default_password: Option<&String>) -> Controller {
        //%%$%%
        // let db = Self::try_logins_without_username_or_password(force_user_pass_prompt, &ui).unwrap_or_else(|e| {
        let db = Self::try_db_logins(force_user_pass_prompt, &ui, default_username, default_password).unwrap_or_else(|e| {
            //%%should panic instead, at all places like this? to get a stack trace and for style?
            //%%should eprintln at other places like this also?
            // ui.display_text1(e.to_string().as_str());
            eprintln!("{}", e.to_string().as_str());
            std::process::exit(1);
        });
        Controller {
            ui,
            force_user_pass_prompt,
            //%%$%
            // default_username,
            // default_password,
            //%%after the red marks are gone, can ^K on next line, and back?
            //%%$%%
            db,
            move_farther_count: 25,
            move_farthest_count: 50,
        }
    }
    // %%
    /* %%
      /** Returns the id and the entity, if they are available from the preferences lookup (id) and then finding that in the db (Entity). */
        fn getDefaultEntity: Option[(i64, Entity)] {
        if (defaultDisplayEntityId.isEmpty || ! localDb.entityKeyExists(defaultDisplayEntityId.get)) {
          None
        } else {
          let entity: Option[Entity] = Entity.getEntity(localDb, defaultDisplayEntityId.get);
          if (entity.isDefined && entity.get.isArchived) {
            let msg = "The default entity \n" + "    " + entity.get.getId + ": \"" + entity.get.getName + "\"\n" +;
                      "... was found but is archived.  You might run" +
                      " into problems unless you un-archive it, or choose a different entity to make the default, or display all archived" +
                      " entities then search for this entity and un-archive it under its Entity Menu options 9, 4."
            let ans = ui.askWhich(Some(Array(msg)), Array("Un-archive the default entity now", "Display archived entities"));
            if (ans.isDefined) {
              if (ans.get == 1) {
                entity.get.unarchive()
              } else if (ans.get == 2) {
                localDb.setIncludeArchivedEntities(true)
              }
            }
          }
          Some((defaultDisplayEntityId.get, entity.get))
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

        //%%temporary/experiment:
        // loop {
        //     let _x=TextUI::get_user_input_char(None);
        //     // dbg!(_x);
        // }
        /* %%$%
         // Max id used as default here because it seems the least likely # to be used in the system hence the
         // most likely to cause an error as default by being missing, so the system can respond by prompting
         // the user in some other way for a use.
         if (getDefaultEntity.isEmpty) {
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
           new MainMenu(ui, localDb, this).mainMenu(if (getDefaultEntity.isEmpty) None else Some(getDefaultEntity.get._2),
                                               goDirectlyToChoice)
           menuLoop()
         }
         menuLoop(Some(5))
        %%    */
    }

    //%%$%%
    // fn try_logins_without_username_or_password<'a>(force_user_pass_prompt: bool, ui: &'a TextUI) -> Result<PostgreSQLDatabase, &'a str> {
    //     Self::try_logins(force_user_pass_prompt, ui/*%%unused?:, None, None*/)
    // }

    //%%$%%
    /// If the 1st parm is true, the next 2 must be None.
    fn try_db_logins<'a>(force_user_pass_prompt: bool, ui: &'a TextUI, default_username: Option<&String>,
                            default_password: Option<&String>) -> Result<PostgreSQLDatabase, String> {
        if force_user_pass_prompt {
            //%%why had this assertion before?:  delete it now?  (it was a "require" in Controller.scala .)
            // assert!(default_username.is_none() && default_password.is_none());

            ui.display_text1("%%$%put back when ready to implement TextUI.ask_for_string l 240");
            Err("%%$%put back next line when ready to implement TextUI.ask_for_string l 240".to_string())
            // Self::prompt_for_user_pass_and_login(ui)

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
            let db_result: Result<PostgreSQLDatabase, String> = PostgreSQLDatabase::login(user, pass);
            // not attempting to clear that password variable because
            // maybe the default kind is less intended to be secure, anyway?
            db_result
        } else {
            println!("{}","%%$%2: put back next line when ready to implement TextUI.ask_for_string l 240".to_string());
            Self::try_other_logins_or_prompt(ui)
        }
    }

    // //%%$%%
    // fn prompt_for_user_pass_and_login<'a>(ui: &TextUI) -> Result<PostgreSQLDatabase, &'a str> {
    //     loop {
    //         let usr = ui::ask_for_string1(Some(["Username"]));
    //         if usr.isEmpty {
    //             //user probably wants out
    //             std::process::exit(1);
    //         }
    //         let pwd = ui::ask_for_string1(Some(["Password"]), None, None, true);
    //         if pwd.isEmpty {
    //             //user probably wants out.
    //             // %%But what if the pwd is really blank? could happen?
    //             std::process::exit(1);
    //         }
    //         let db: Result<PostgreSQLDatabase, &str> = PostgreSQLDatabase::login(usr.get, pwd.get);
    //         if db.isOk() {
    //             break db;
    //         } else {
    //             continue;
    //         }
    //     }
    // }

    // %%$%%
    /// Tries the system username & default password, & if that doesn't work, prompts user.
    fn try_other_logins_or_prompt(ui: &TextUI) -> Result<PostgreSQLDatabase, String> {
        // (this loop is to simulate recursion, and let the user retry entering username/password)
        loop {
            // try logging in with some obtainable default values first, to save user the trouble, like if pwd is blank
            let (default_username, default_password) = Util::get_default_user_login().unwrap_or_else(|e| {
                eprintln!("Unable to get default username/password.  Trying blank username, and password \"x\" instead.  Underlying error is: \"{}\"", e);
                ("".to_string(), "x")
            });
            let db_with_system_name_blank_pwd = PostgreSQLDatabase::login(default_username.as_str(), default_password);
            if db_with_system_name_blank_pwd.is_ok() {
              ui.display_text2("(Using default user info...)", false);
              break db_with_system_name_blank_pwd;
            } else {
                let usr = ui.ask_for_string3(vec!("Username".to_string()), None, Some(default_username));
                match usr {
                    None => {
                        // seems like the user wants out
                        std::process::exit(1);
                    },
                    Some(username) => {
                        let db_connected_with_default_pwd = PostgreSQLDatabase::login(username.as_str(), default_password);
                        if db_connected_with_default_pwd.is_ok() {
                            break db_connected_with_default_pwd;
                        } else {
                            let pwd = ui.ask_for_string4(vec!("Password".to_string()), None, None, true);
                            match pwd {
                                None => {
                                    // seems like the user wants out
                                    std::process::exit(1);
                                },
                                Some(password) => {
                                    let db_with_user_entered_pwd = PostgreSQLDatabase::login(username.as_str(), password.as_str());
                                    match db_with_user_entered_pwd {
                                        Ok(db) => break Ok(db),
                                        Err(e) => {
                                            let msg = format!("Login failed; retrying ({}) to quit if needed):  {}", ui.how_quit(), e);
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

    /* %%
    // Idea: showPublicPrivateStatusPreference, refreshPublicPrivateStatusPreference, and findDefaultDisplayEntityId, feel awkward.
    // Needs something better, but I'm not sure
    // what, at the moment.  It was created this way as a sort of cache because looking it up every time was costly and made the app slow, like when
    // displaying a list of entities (getting the preference every time, to N levels deep), and especially at startup when checking for the default
    // up to N levels deep, among the preferences that can include entities with deep nesting.  So in a related change I made it also not look N levels
    // deep, for preferences.  If you check other places touched by this commit there may be a "shotgun surgery" bad smell here also.
    //Idea: Maybe these should have their cache expire after a period of time (to help when running multiple clients).
    let mut showPublicPrivateStatusPreference: Option[Boolean] = localDb.getUserPreference_Boolean(Util.SHOW_PUBLIC_PRIVATE_STATUS_PREFERENCE);
    fn refreshPublicPrivateStatusPreference() -> Unit {
      showPublicPrivateStatusPreference = localDb.getUserPreference_Boolean(Util.SHOW_PUBLIC_PRIVATE_STATUS_PREFERENCE)
    }

      // putting this in a var instead of recalculating it every time (too frequent) inside findDefaultDisplayEntityId:;
      let mut defaultDisplayEntityId: Option[i64] = localDb.getUserPreference_EntityId(Util.DEFAULT_ENTITY_PREFERENCE);
        fn refreshDefaultDisplayEntityId() /*-> Unit%%*/  {
        defaultDisplayEntityId = localDb.getUserPreference_EntityId(Util.DEFAULT_ENTITY_PREFERENCE)
      }

    fn askForClass(dbIn: Database) -> Option[i64] {
        let msg = "CHOOSE ENTITY'S CLASS.  (Press ESC if you don't know or care about this.  Detailed explanation on the class feature will be available " +;
                  "at onemodel.org when this feature is documented more (hopefully at the next release), or ask on the email list.)"
        let result: Option[(IdWrapper, Boolean, String)] = chooseOrCreateObject(dbIn, Some(List[String](msg)), None, None, Util.ENTITY_CLASS_TYPE);
        if (result.isEmpty) None
        else Some(result.get._1.getId)
    }

      /** In any given usage, consider whether askForNameAndWriteEntity should be used instead: it is for quick (simpler) creation situations or
        * to just edit the name when the entity already exists, or if the Entity is a RelationType,
        * askForClassInfoAndNameAndCreateEntity (this one) prompts for a class and checks whether it should copy default attributes from the class-defining
        * (template) entity.
        * There is also editEntityName which calls askForNameAndWriteEntity: it checks if the Entity being edited is a RelationType, and if not also checks
        * for whether a group name should be changed at the same time.
        */
        fn askForClassInfoAndNameAndCreateEntity(dbIn: Database, classIdIn: Option[i64] = None) -> Option[Entity] {
        let mut newClass = false;
        let classId: Option[i64] =;
          if (classIdIn.isDefined) classIdIn
          else {
            newClass = true
            askForClass(dbIn)
          }
        let ans: Option[Entity] = askForNameAndWriteEntity(dbIn, Util.ENTITY_TYPE, None, None, None, None, classId,;
                                                           Some(if (newClass) "DEFINE THE ENTITY:" else ""))
        if (ans.isDefined) {
          let entity = ans.get;
          // idea: (is also on fix list): this needs to be removed, after evaluating for other side effects, to fix the bug
          // where creating a new relationship, and creating the entity2 in the process, it puts the wrong info
          // on the header for what is being displayed/edited next!: Needs refactoring anyway: this shouldn't be at
          // a low level.
          ui.display_text("Created " + Util.ENTITY_TYPE + ": " + entity.getName, false);

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
        fn askForNameAndWriteEntity(dbIn: Database, typeIn: String, existingEntityIn: Option[Entity] = None, previousNameIn: Option[String] = None,
                                   previousDirectionalityIn: Option[String] = None,
                                   previousNameInReverseIn: Option[String] = None, classIdIn: Option[i64] = None,
                                   leadingTextIn: Option[String] = None, duplicateNameProbablyOK: Boolean = false) -> Option[Entity] {
        if (classIdIn.isDefined) require(typeIn == Util.ENTITY_TYPE)
        let createNotUpdate: bool = existingEntityIn.isEmpty;
        if (!createNotUpdate && typeIn == Util.RELATION_TYPE_TYPE) require(previousDirectionalityIn.isDefined)
        let maxNameLength = {;
          if (typeIn == Util.RELATION_TYPE_TYPE) model.RelationType.getNameLength
          else if (typeIn == Util.ENTITY_TYPE) model.Entity.nameLength
          else throw new scala.Exception("invalid inType: " + typeIn)
        }
        let example = {;
          if (typeIn == Util.RELATION_TYPE_TYPE) " (use 3rd-person verb like \"owns\"--might make output like sentences more consistent later on)"
          else ""
        }

        /** 2nd i64 in return value is ignored in this particular case.
          */
        def askAndSave(dbIn: Database, defaultNameIn: Option[String] = None): Option[(i64, i64)] = {
          let nameOpt = ui::ask_for_string3(Some(Array[String](leadingTextIn.getOrElse(""),;
                                                           "Enter " + typeIn + " name (up to " + maxNameLength + " characters" + example + "; ESC to cancel)")),
                                        None, defaultNameIn)
          if (nameOpt.isEmpty) None
          else {
            let name = nameOpt.get.trim();
            if (name.length <= 0) None
            else {
              // idea: this size check might be able to account better for the escaping that's done. Or just keep letting the exception handle it as is already
              // done in the caller of this.
              if (name.length > maxNameLength) {
                ui.display_text(Util.stringTooLongErrorMessage(maxNameLength).format(Util.tooLongMessage) + ".")
                askAndSave(dbIn, Some(name))
              } else {
                let selfIdToIgnore: Option[i64] = if (existingEntityIn.isDefined) Some(existingEntityIn.get.getId) else None;
                if (Util.isDuplicationAProblem(model.Entity.isDuplicate(dbIn, name, selfIdToIgnore), duplicateNameProbablyOK, ui)) None
                else {
                  if (typeIn == Util.ENTITY_TYPE) {
                    if (createNotUpdate) {
                      let newId = model.Entity.createEntity(dbIn, name, classIdIn).getId;
                      Some(newId, 0L)
                    } else {
                      existingEntityIn.get.updateName(name)
                      Some(existingEntityIn.get.getId, 0L)
                    }
                  } else if (typeIn == Util.RELATION_TYPE_TYPE) {
                    let ans: Option[String] = Util.askForRelationDirectionality(previousDirectionalityIn, ui);
                    if (ans.isEmpty) None
                    else {
                      let directionalityStr: String = ans.get.trim().toUpperCase;
                      let nameInReverseDirectionStr = Util.askForNameInReverseDirection(directionalityStr, maxNameLength, name, previousNameInReverseIn, ui);
                      if (createNotUpdate) {
                        let newId = new RelationType(dbIn, dbIn.createRelationType(name, nameInReverseDirectionStr, directionalityStr)).getId;
                        Some(newId, 0L)
                      } else {
                        existingEntityIn.get.asInstanceOf[RelationType].update(name, nameInReverseDirectionStr, directionalityStr)
                        Some(existingEntityIn.get.getId, 0L)
                      }
                    }
                  } else throw new scala.Exception("unexpected value: " + typeIn)
                }
              }
            }
          }
        }

        let result = tryAskingAndSaving[(i64, i64)](dbIn, Util.stringTooLongErrorMessage(maxNameLength), askAndSave, previousNameIn);
        if (result.isEmpty) None
        else Some(new Entity(dbIn, result.get._1))
      }

      /** Call a provided function (method?) "askAndSaveIn", which does some work that might throw a specific OmDatabaseException.  If it does throw that,
        * let the user know the problem and call askAndSaveIn again.  I.e., allow retrying if the entered data is bad, instead of crashing the app.
        */
        fn tryAskingAndSaving[T](dbIn: Database,
                                errorMsgIn: String,
                                askAndSaveIn: (Database, Option[String]) => Option[T],
                                defaultNameIn: Option[String] = None) -> Option[T] {
          /*%%for the try/catch, see
             https://doc.rust-lang.org/std/panic/fn.catch_unwind.html
          ....for ideas?  OR JUST USE ERRORS INSTEAD!
     */
        try {
          askAndSaveIn(dbIn, defaultNameIn)
        }
        catch {
          case e: OmDatabaseException =>
            def accumulateMsgs(msgIn: String, t: Throwable): String = {
              if (t.getCause == null) {
                t.toString
              } else {
                msgIn + " (" + accumulateMsgs(t.toString, t.getCause) + ")"
              }
            }
            let cumulativeMsg = accumulateMsgs(e.toString, e.getCause);
            if (cumulativeMsg.contains(Util.tooLongMessage)) {
              ui.display_text(errorMsgIn.format(Util.tooLongMessage) + cumulativeMsg + ".")
              tryAskingAndSaving[T](dbIn, errorMsgIn, askAndSaveIn, defaultNameIn)
            } else throw e
        }
      }

      /**
        * @param classIn (1st parameter) should be None only if the call is intended to create; otherwise it is an edit.
        * @return None if user wants out, otherwise returns the new or updated classId and entityId.
        * */
        fn askForAndWriteClassAndTemplateEntityName(dbIn: Database, classIn: Option[EntityClass] = None) -> Option[(i64, i64)] {
        if (classIn.isDefined) {
          // dbIn is required even if classIn is not provided, but if classIn is provided, make sure things are in order:
          // (Idea:  check: does scala do a deep equals so it is valid?  also tracked in tasks.)
          require(classIn.get.mDB == dbIn)
        }
        let createNotUpdate: bool = classIn.isEmpty;
        let nameLength = model.EntityClass.nameLength(dbIn);
        let oldTemplateNamePrompt = {;
          if (createNotUpdate) ""
          else {
            let entityId = classIn.get.getTemplateEntityId;
            let templateEntityName = new Entity(dbIn, entityId).getName;
            " (which is currently \"" + templateEntityName + "\")"
          }
        }
        def askAndSave(dbIn: Database, defaultNameIn: Option[String]): Option[(i64, i64)] = {
          let nameOpt = ui::ask_for_string3(Some(Array("Enter class name (up to " + nameLength + " characters; will also be used for its template entity name" +;
                                                   oldTemplateNamePrompt + "; ESC to cancel): ")),
                                        None, defaultNameIn)
          if (nameOpt.isEmpty) None
          else {
            let name = nameOpt.get.trim();
            if (name.length() == 0) None
            else {
              if (Util.isDuplicationAProblem(EntityClass.isDuplicate(dbIn, name, if (classIn.isEmpty) None else Some(classIn.get.getId)),
                                             duplicateNameProbablyOK = false, ui)) {
                None
              }
              else {
                if (createNotUpdate) {
                  Some(dbIn.createClassAndItsTemplateEntity(name))
                } else {
                  let entityId: i64 = classIn.get.updateClassAndTemplateEntityName(name);
                  Some(classIn.get.getId, entityId)
                }
              }
            }
          }
        }

        tryAskingAndSaving[(i64, i64)](dbIn, Util.stringTooLongErrorMessage(nameLength), askAndSave, if (classIn.isEmpty) None else Some(classIn.get.getName))
      }

      /** SEE DESCRIPTIVE COMMENT ON askForAndWriteClassAndTemplateEntityName, WHICH APPLIES TO all such METHODS (see this cmt elsewhere).
        * @return The instance's id, or None if there was a problem or the user wants out.
        * */
        fn askForAndWriteOmInstanceInfo(dbIn: Database, oldOmInstanceIn: Option[OmInstance] = None) -> Option[String] {
        let createNotUpdate: bool = oldOmInstanceIn.isEmpty;
        let addressLength = model.OmInstance.addressLength;
        def askAndSave(dbIn: Database, defaultNameIn: Option[String]): Option[String] = {
          let addressOpt = ui::ask_for_string3(Some(Array("Enter the internet address with optional port of a remote OneModel instance (for " +;
                                                      "example, \"om.example.com:9000\", up to " + addressLength + " characters; ESC to cancel;" +
                                                      " Other examples include (omit commas):  localhost,  127.0.0.1:2345,  ::1 (?)," +
                                                      "  my.example.com:80,  your.example.com:8080  .): ")), None, defaultNameIn)
          if (addressOpt.isEmpty) None
          else {
            let address = addressOpt.get.trim();
            if (address.length() == 0) None
            else {
              if (Util.isDuplicationAProblem(OmInstance.isDuplicate(dbIn, address, if (oldOmInstanceIn.isEmpty) None else Some(oldOmInstanceIn.get.getId)),
                                             duplicateNameProbablyOK = false, ui)) {
                None
              } else {
                let restDb = Database.getRestDatabase(address);
                let remoteId: Option[String] = restDb.getIdWithOptionalErrHandling(Some(ui));
                if (remoteId.isEmpty) {
                  None
                } else {
                  if (createNotUpdate) {
                    OmInstance.create(dbIn, remoteId.get, address)
                    remoteId
                  } else {
                    if (oldOmInstanceIn.get.getId == remoteId.get) {
                      oldOmInstanceIn.get.update(address)
                      Some(oldOmInstanceIn.get.getId)
                    } else {
                      let ans: Option[Boolean] = ui.askYesNoQuestion("The IDs of the old and new remote instances don't match (old " +;
                                                                     "id/address: " + oldOmInstanceIn.get.getId + "/" +
                                                                     oldOmInstanceIn.get.getAddress + ", new id/address: " +
                                                                     remoteId.get + "/" + address + ".  Instead of updating the old one, you should create a new" +
                                                                     " entry for the new remote instance and then optionally delete this old one." +
                                                                     "  Do you want to create the new entry with this new address, now?")
                      if (ans.isDefined && ans.get) {
                        let id: String = OmInstance.create(dbIn, remoteId.get, address).getId;
                        ui.display_text("Created the new entry for \"" + address + "\".  You still have to delete the old one (" + oldOmInstanceIn.get.getId + "/" +
                                       oldOmInstanceIn.get.getAddress + ") if you don't want it to be there.")
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

        tryAskingAndSaving[String](dbIn, Util.stringTooLongErrorMessage(addressLength), askAndSave,
                                   if (oldOmInstanceIn.isEmpty) {
                                     None
                                   } else {
                                     Some(oldOmInstanceIn.get.getAddress)
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
        fn askForInfoAndUpdateAttribute[T <: AttributeDataHolder](dbIn: Database, dhIn: T, askForAttrTypeId: Boolean, attrType: String,
                                                                 promptForSelectingTypeId: String,
                                                                 getOtherInfoFromUser: (Database, T, Boolean, TextUI) => Option[T],
                                                                 updateTypedAttribute: (T) => Unit) -> Boolean {
        //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) within this method, below!
        @tailrec def askForInfoAndUpdateAttribute_helper(dhIn: T, attrType: String, promptForTypeId: String): Boolean = {
          let ans: Option[T] = askForAttributeData[T](dbIn, dhIn, askForAttrTypeId, attrType, Some(promptForTypeId),;
                                                      Some(new Entity(dbIn, dhIn.attrTypeId).getName),
                                                      Some(dhIn.attrTypeId), getOtherInfoFromUser, editingIn = true)
          if (ans.isEmpty) {
            false
          } else {
            let dhOut: T = ans.get;
            let ans2: Option[Int] = Util.promptWhetherTo1Add2Correct(attrType, ui);

            if (ans2.isEmpty) {
              false
            } else if (ans2.get == 1) {
              updateTypedAttribute(dhOut)
              true
            }
            else if (ans2.get == 2) askForInfoAndUpdateAttribute_helper(dhOut, attrType, promptForTypeId)
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
      final def attributeEditMenu(attributeIn: Attribute): Boolean = {
        let leadingText: Array[String] = Array("Attribute: " + attributeIn.getDisplayString(0, None, None));
        let mut firstChoices = Array("Edit the attribute type, " +;
                                 (if (Util.canEditAttributeOnSingleLine(attributeIn)) "content (single line)," else "") +
                                 " and valid/observed dates",

                                 if (attributeIn.isInstanceOf[TextAttribute]) "Edit (as multi-line value)" else "(stub)",
                                 if (Util.canEditAttributeOnSingleLine(attributeIn)) "Edit the attribute content (single line)" else "(stub)",
                                 "Delete",
                                 "Go to entity representing the type: " + new Entity(attributeIn.mDB, attributeIn.getAttrTypeId).getName)
        if (attributeIn.isInstanceOf[FileAttribute]) {
          firstChoices = firstChoices ++ Array[String]("Export the file")
        }
        let response = ui.askWhich(Some(leadingText), firstChoices);
        if (response.isEmpty) false
        else {
          let answer: i32 = response.get;
          if (answer == 1) {
            attributeIn match {
              case quantityAttribute: QuantityAttribute =>
                def updateQuantityAttribute(dhInOut: QuantityAttributeDataHolder) {
                  quantityAttribute.update(dhInOut.attrTypeId, dhInOut.unitId, dhInOut.number, dhInOut.validOnDate,
                                           dhInOut.observationDate)
                }
                askForInfoAndUpdateAttribute[QuantityAttributeDataHolder](attributeIn.mDB,
                                                                          new QuantityAttributeDataHolder(quantityAttribute.getAttrTypeId,
                                                                                                          quantityAttribute.getValidOnDate,
                                                                                                          quantityAttribute.getObservationDate,
                                                                                                          quantityAttribute.getNumber, quantityAttribute.getUnitId),
                                                                          askForAttrTypeId = true, Util.QUANTITY_TYPE, Util.quantityTypePrompt,
                                                                          askForQuantityAttributeNumberAndUnit, updateQuantityAttribute)
                //force a reread from the DB so it shows the right info on the repeated menu:
                attributeEditMenu(new QuantityAttribute(attributeIn.mDB, attributeIn.getId))
              case textAttribute: TextAttribute =>
                def updateTextAttribute(dhInOut: TextAttributeDataHolder) {
                  textAttribute.update(dhInOut.attrTypeId, dhInOut.text, dhInOut.validOnDate, dhInOut.observationDate)
                }
                let textAttributeDH: TextAttributeDataHolder = new TextAttributeDataHolder(textAttribute.getAttrTypeId, textAttribute.getValidOnDate,;
                                                                                           textAttribute.getObservationDate, textAttribute.getText)
                askForInfoAndUpdateAttribute[TextAttributeDataHolder](attributeIn.mDB, textAttributeDH, askForAttrTypeId = true, Util.TEXT_TYPE,
                                                                      "CHOOSE TYPE OF " + Util.textDescription + ":",
                                                                      Util.askForTextAttributeText, updateTextAttribute)
                //force a reread from the DB so it shows the right info on the repeated menu:
                attributeEditMenu(new TextAttribute(attributeIn.mDB, attributeIn.getId))
              case dateAttribute: DateAttribute =>
                def updateDateAttribute(dhInOut: DateAttributeDataHolder) {
                  dateAttribute.update(dhInOut.attrTypeId, dhInOut.date)
                }
                let dateAttributeDH: DateAttributeDataHolder = new DateAttributeDataHolder(dateAttribute.getAttrTypeId, dateAttribute.getDate);
                askForInfoAndUpdateAttribute[DateAttributeDataHolder](attributeIn.mDB, dateAttributeDH, askForAttrTypeId = true, Util.DATE_TYPE, "CHOOSE TYPE OF DATE:",
                                                                      Util.askForDateAttributeValue, updateDateAttribute)
                //force a reread from the DB so it shows the right info on the repeated menu:
                attributeEditMenu(new DateAttribute(attributeIn.mDB, attributeIn.getId))
              case booleanAttribute: BooleanAttribute =>
                def updateBooleanAttribute(dhInOut: BooleanAttributeDataHolder) {
                  booleanAttribute.update(dhInOut.attrTypeId, dhInOut.boolean, dhInOut.validOnDate, dhInOut.observationDate)
                }
                let booleanAttributeDH: BooleanAttributeDataHolder = new BooleanAttributeDataHolder(booleanAttribute.getAttrTypeId, booleanAttribute.getValidOnDate,;
                                                                                                    booleanAttribute.getObservationDate,
                                                                                                    booleanAttribute.getBoolean)
                askForInfoAndUpdateAttribute[BooleanAttributeDataHolder](attributeIn.mDB, booleanAttributeDH, askForAttrTypeId = true, Util.BOOLEAN_TYPE,
                                                                         "CHOOSE TYPE OF TRUE/FALSE VALUE:", Util.askForBooleanAttributeValue,
                                                                         updateBooleanAttribute)
                //force a reread from the DB so it shows the right info on the repeated menu:
                attributeEditMenu(new BooleanAttribute(attributeIn.mDB, attributeIn.getId))
              case fa: FileAttribute =>
                def updateFileAttribute(dhInOut: FileAttributeDataHolder) {
                  fa.update(Some(dhInOut.attrTypeId), Some(dhInOut.description))
                }
                let fileAttributeDH: FileAttributeDataHolder = new FileAttributeDataHolder(fa.getAttrTypeId, fa.getDescription, fa.getOriginalFilePath);
                askForInfoAndUpdateAttribute[FileAttributeDataHolder](attributeIn.mDB, fileAttributeDH, askForAttrTypeId = true, Util.FILE_TYPE, "CHOOSE TYPE OF FILE:",
                                                                      Util.askForFileAttributeInfo, updateFileAttribute)
                //force a reread from the DB so it shows the right info on the repeated menu:
                attributeEditMenu(new FileAttribute(attributeIn.mDB, attributeIn.getId))
              case _ => throw new Exception("Unexpected type: " + attributeIn.getClass.getName)
            }
          } else if (answer == 2 && attributeIn.isInstanceOf[TextAttribute]) {
            let ta = attributeIn.asInstanceOf[TextAttribute];
            let newContent: String = Util.editMultilineText(ta.getText, ui);
            ta.update(ta.getAttrTypeId, newContent, ta.getValidOnDate, ta.getObservationDate)
            //then force a reread from the DB so it shows the right info on the repeated menu:
            attributeEditMenu(new TextAttribute(attributeIn.mDB, attributeIn.getId))
          } else if (answer == 3 && Util.canEditAttributeOnSingleLine(attributeIn)) {
            editAttributeOnSingleLine(attributeIn)
            false
          } else if (answer == 4) {
            let ans = ui.askYesNoQuestion("DELETE this attribute: ARE YOU SURE?");
            if (ans.isDefined && ans.get) {
              attributeIn.delete()
              true
            } else {
              ui.display_text("Did not delete attribute.", false);
              attributeEditMenu(attributeIn)
            }
          } else if (answer == 5) {
            new EntityMenu(ui, this).entityMenu(new Entity(attributeIn.mDB, attributeIn.getAttrTypeId))
            attributeEditMenu(attributeIn)
          } else if (answer == 6) {
            if (!attributeIn.isInstanceOf[FileAttribute]) throw new Exception("Menu shouldn't have allowed us to get here w/ a type other than FA (" +
                                                                              attributeIn.getClass.getName + ").")
            let fa: FileAttribute = attributeIn.asInstanceOf[FileAttribute];
            //%%see 1st instance of try {  for rust-specific idea here.
            try {
              // this file should be confirmed by the user as ok to write, even overwriting what is there.
              let file: Option[File] = ui.getExportDestination(fa.getOriginalFilePath, fa.getMd5Hash);
              if (file.isDefined) {
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
        fn editAttributeOnSingleLine(attributeIn: Attribute) -> Boolean {
        require(Util.canEditAttributeOnSingleLine(attributeIn))

        attributeIn match {
          case quantityAttribute: QuantityAttribute =>
            let num: Option[Float] = Util.askForQuantityAttributeNumber(quantityAttribute.getNumber, ui);
            if (num.isDefined) {
              quantityAttribute.update(quantityAttribute.getAttrTypeId, quantityAttribute.getUnitId,
                                       num.get,
                                       quantityAttribute.getValidOnDate, quantityAttribute.getObservationDate)
            }
            num.isEmpty
          case textAttribute: TextAttribute =>
            let textAttributeDH: TextAttributeDataHolder = new TextAttributeDataHolder(textAttribute.getAttrTypeId, textAttribute.getValidOnDate,;
                                                                                       textAttribute.getObservationDate, textAttribute.getText)
            let outDH: Option[TextAttributeDataHolder] = Util.askForTextAttributeText(attributeIn.mDB, textAttributeDH, inEditing = true, ui);
            if (outDH.isDefined) textAttribute.update(outDH.get.attrTypeId, outDH.get.text, outDH.get.validOnDate, outDH.get.observationDate)
            outDH.isEmpty
          case dateAttribute: DateAttribute =>
            let dateAttributeDH: DateAttributeDataHolder = new DateAttributeDataHolder(dateAttribute.getAttrTypeId, dateAttribute.getDate);
            let outDH: Option[DateAttributeDataHolder] = Util.askForDateAttributeValue(attributeIn.mDB, dateAttributeDH, inEditing = true, ui);
            if (outDH.isDefined) dateAttribute.update(outDH.get.attrTypeId, outDH.get.date)
            outDH.isEmpty
          case booleanAttribute: BooleanAttribute =>
            let booleanAttributeDH: BooleanAttributeDataHolder = new BooleanAttributeDataHolder(booleanAttribute.getAttrTypeId, booleanAttribute.getValidOnDate,;
                                                                                                booleanAttribute.getObservationDate,
                                                                                                booleanAttribute.getBoolean)
            let outDH: Option[BooleanAttributeDataHolder] = Util.askForBooleanAttributeValue(booleanAttribute.mDB, booleanAttributeDH, inEditing = true, ui);
            if (outDH.isDefined) booleanAttribute.update(outDH.get.attrTypeId, outDH.get.boolean, outDH.get.validOnDate, outDH.get.observationDate)
            outDH.isEmpty
          case rtle: RelationToLocalEntity =>
            let editedEntity: Option[Entity] = editEntityName(new Entity(rtle.mDB, rtle.getRelatedId2));
            editedEntity.isEmpty
          case rtre: RelationToRemoteEntity =>
            let editedEntity: Option[Entity] = editEntityName(new Entity(rtre.getRemoteDatabase, rtre.getRelatedId2));
            editedEntity.isEmpty
          case rtg: RelationToGroup =>
            let editedGroupName: Option[String] = Util.editGroupName(new Group(rtg.mDB, rtg.getGroupId), ui);
            editedGroupName.isEmpty
          case _ => throw new scala.Exception("Unexpected type: " + attributeIn.getClass.getCanonicalName)
        }
      }

      /**
       * @return (See addAttribute method.)
       */
        fn askForInfoAndAddAttribute[T <: AttributeDataHolder](dbIn: Database, dhIn: T, askForAttrTypeId: Boolean, attrType: String,
                                                              promptForSelectingTypeId: Option[String],
                                                              getOtherInfoFromUser: (Database, T, Boolean, TextUI) => Option[T],
                                                              addTypedAttribute: (T) => Option[Attribute]) -> Option[Attribute] {
        let ans: Option[T] = askForAttributeData[T](dbIn, dhIn, askForAttrTypeId, attrType, promptForSelectingTypeId,;
                                                    None, None, getOtherInfoFromUser, editingIn = false)
        if (ans.isDefined) {
          let dhOut: T = ans.get;
          addTypedAttribute(dhOut)
        } else None
      }

      /**
       * SEE DESCRIPTIVE COMMENT ON askForAndWriteClassAndTemplateEntityName, WHICH APPLIES TO all such METHODS (see this cmt elsewhere).
       *
       * @return None if user wants out.
       */
        fn editEntityName(entityIn: Entity) -> Option[Entity] {
        let editedEntity: Option[Entity] = entityIn match {;
          case relTypeIn: RelationType =>
            let previousNameInReverse: String = relTypeIn.getNameInReverseDirection //idea: check: this edits name w/ prefill also?:;
            askForNameAndWriteEntity(entityIn.mDB, Util.RELATION_TYPE_TYPE, Some(relTypeIn), Some(relTypeIn.getName), Some(relTypeIn.getDirectionality),
                                     if (previousNameInReverse == null || previousNameInReverse.trim().isEmpty) None else Some(previousNameInReverse),
                                     None)
          case entity: Entity =>
            let entityNameBeforeEdit: String = entityIn.getName;
            let editedEntity: Option[Entity] = askForNameAndWriteEntity(entityIn.mDB, Util.ENTITY_TYPE, Some(entity), Some(entity.getName), None, None, None);
            if (editedEntity.isDefined) {
              let entityNameAfterEdit: String = editedEntity.get.getName;
              if (entityNameBeforeEdit != entityNameAfterEdit) {
                let (_, _, groupId, groupName, moreThanOneAvailable) = editedEntity.get.findRelationToAndGroup;
                if (groupId.isDefined && !moreThanOneAvailable) {
                  let attrCount = entityIn.getAttributeCount();
                  // for efficiency, if it's obvious which subgroup's name to change at the same time, offer to do so
                  let defaultAnswer = if (attrCount > 1) Some("n") else Some("y");
                  let ans = ui.askYesNoQuestion("There's a single subgroup named \"" + groupName + "\"" +;
                                                (if (attrCount > 1) " (***AMONG " + (attrCount - 1) + " OTHER ATTRIBUTES***)" else "") +
                                                "; possibly it and this entity were created at the same time.  Also change" +
                                                " the subgroup's name now to be identical?", defaultAnswer)
                  if (ans.isDefined && ans.get) {
                    let group = new Group(entityIn.mDB, groupId.get);
                    group.update(nameIn = Some(entityNameAfterEdit), validOnDateInIGNORED4NOW = None, observationDateInIGNORED4NOW = None)
                  }
                }
              }
            }
            editedEntity
          case _ => throw new Exception("??")
        }
        editedEntity
      }

        fn askForPublicNonpublicStatus(defaultForPrompt: Option[Boolean]) -> Option[Boolean] {
        let valueAfterEdit: Option[Boolean] = ui.askYesNoQuestion("For Public vs. Non-public, enter a yes/no value (or a space" +;
                                                                  " for 'unknown/unspecified'; used e.g. during data export; display preference can be" +
                                                                  " set under main menu / " + Util.menuText_viewPreferences + ")",
                                                                  if (defaultForPrompt.isEmpty) Some("") else if (defaultForPrompt.get) Some("y") else Some("n"),
                                                                  allowBlankAnswer = true)
        valueAfterEdit
      }

      /** Returns data, or None if user wants to cancel/get out.
        * @param attrType Constant referring to Attribute subtype, as used by the inObjectType parameter to the chooseOrCreateObject method
        *                 (ex., Controller.QUANTITY_TYPE).  See comment on that method, for that parm.
        * */
        fn askForAttributeData[T <: AttributeDataHolder](dbIn: Database, inoutDH: T, alsoAskForAttrTypeId: Boolean, attrType: String, attrTypeInputPrompt: Option[String],
                                                        inPreviousSelectionDesc: Option[String], inPreviousSelectionId: Option[i64],
                                                        askForOtherInfo: (Database, T, Boolean, TextUI) => Option[T], editingIn: Boolean) -> Option[T] {
        let (userWantsOut: Boolean, attrTypeId: i64, isRemote, remoteKey) = {
          if (alsoAskForAttrTypeId) {
            require(attrTypeInputPrompt.isDefined)
            let ans: Option[(IdWrapper, Boolean, String)] = chooseOrCreateObject(dbIn, Some(List(attrTypeInputPrompt.get)), inPreviousSelectionDesc,;
                                                                                 inPreviousSelectionId, attrType)
            if (ans.isEmpty) {
              (true, 0L, false, "")
            } else {
              (false, ans.get._1.getId, ans.get._2, ans.get._3)
            }
          } else {
            // maybe not ever reached under current system logic. not certain.
            let (isRemote, remoteKey) = {;
              //noinspection TypeCheckCanBeMatch
              if (inoutDH.isInstanceOf[RelationToEntityDataHolder]) {
                (inoutDH.asInstanceOf[RelationToEntityDataHolder].isRemote, inoutDH.asInstanceOf[RelationToEntityDataHolder].remoteInstanceId)
              } else {
                (false, "")
              }
            }
            (false, inoutDH.attrTypeId, isRemote, remoteKey)
          }
        }

        if (userWantsOut) {
          None
        } else {
          inoutDH.attrTypeId = attrTypeId
          //noinspection TypeCheckCanBeMatch
          if (inoutDH.isInstanceOf[RelationToEntityDataHolder]) {
            inoutDH.asInstanceOf[RelationToEntityDataHolder].isRemote = isRemote
            inoutDH.asInstanceOf[RelationToEntityDataHolder].remoteInstanceId = remoteKey
          }
          let ans2: Option[T] = askForOtherInfo(dbIn, inoutDH, editingIn, ui);
          if (ans2.isEmpty) None
          else {
            let mut userWantsToCancel = false;
            // (the ide/intellij preferred to have it this way instead of 'if')
            inoutDH match {
              case dhWithVOD: AttributeDataHolderWithVODates =>
                let (validOnDate: Option[i64], observationDate: i64, userWantsToCancelInner: Boolean) =;
                  Util.askForAttributeValidAndObservedDates(editingIn, dhWithVOD.validOnDate, dhWithVOD.observationDate, ui)

                if (userWantsToCancelInner) userWantsToCancel = true
                else {
                  dhWithVOD.observationDate = observationDate
                  dhWithVOD.validOnDate = validOnDate
                }
              case _ =>
                //do nothing
            }
            if (userWantsToCancel) None
            else Some(inoutDH)
          }
        }
      }

      /** Searches for a regex, case-insensitively, & returns the id of an Entity, or None if user wants out.  The parameter 'idToOmitIn' lets us omit
        * (or flag?) an entity if it should be for some reason (like it's the caller/container & doesn't make sense to be in the group, or something).
        *
        * Idea: re attrTypeIn parm, enum/improvement: see comment re inAttrType at beginning of chooseOrCreateObject.
        */
      @tailrec final def findExistingObjectByText(dbIn: Database, startingDisplayRowIndexIn: i64 = 0, attrTypeIn: String,
                                                  //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) w/in this method!
                                                  idToOmitIn: Option[i64] = None, regexIn: String): Option[IdWrapper] = {
        let leadingText = List[String]("SEARCH RESULTS: " + Util.pickFromListPrompt);
        let choices: Array[String] = Array(Util.listNextItemsPrompt);
        let numDisplayableItems = ui.maxColumnarChoicesToDisplayAfter(leadingText.size, choices.length, Util.maxNameLength);

        let objectsToDisplay = attrTypeIn match {;
          case Util.ENTITY_TYPE =>
            dbIn.getMatchingEntities(startingDisplayRowIndexIn, Some(numDisplayableItems), idToOmitIn, regexIn)
          case Util.GROUP_TYPE =>
            dbIn.getMatchingGroups(startingDisplayRowIndexIn, Some(numDisplayableItems), idToOmitIn, regexIn)
          case _ =>
            throw new OmException("??")
        }
        if (objectsToDisplay.size == 0) {
          ui.display_text("End of list, or none found; starting over from the beginning...")
          if (startingDisplayRowIndexIn == 0) None
          else findExistingObjectByText(dbIn, 0, attrTypeIn, idToOmitIn, regexIn)
        } else {
          let objectNames: Array[String] = objectsToDisplay.toArray.map {;
                                                                          case entity: Entity =>
                                                                            let numSubgroupsPrefix: String = getEntityContentSizePrefix(entity);
                                                                            numSubgroupsPrefix + entity.getArchivedStatusDisplayString + entity.getName
                                                                          case group: Group =>
                                                                            let numSubgroupsPrefix: String = getGroupContentSizePrefix(group.mDB, group.getId);
                                                                            numSubgroupsPrefix + group.getName
                                                                          case x: Any => throw new Exception("unexpected class: " + x.getClass.getName)
                                                                          case _ => throw new OmException("??")
                                                                        }
          let ans = ui.askWhichChoiceOrItsAlternate(Some(leadingText.toArray), choices, objectNames);
          if (ans.isEmpty) None
          else {
            let (answer, userChoseAlternate: Boolean) = ans.get;
            if (answer == 1 && answer <= choices.length) {
              // (For reason behind " && answer <= choices.size", see comment where it is used in entityMenu.)
              let nextStartingIndex: i64 = startingDisplayRowIndexIn + objectsToDisplay.size;
              findExistingObjectByText(dbIn, nextStartingIndex, attrTypeIn, idToOmitIn, regexIn)
            } else if (answer > choices.length && answer <= (choices.length + objectsToDisplay.size)) {
              // those in the condition on the previous line are 1-based, not 0-based.
              let index = answer - choices.length - 1;
              let o = objectsToDisplay.get(index);
              if (userChoseAlternate) {
                attrTypeIn match {
                  // idea: replace this condition by use of a trait (the type of o, which has getId), or being smarter with scala's type system. attrTypeIn match {
                  case Util.ENTITY_TYPE =>
                    new EntityMenu(ui, this).entityMenu(o.asInstanceOf[Entity])
                  case Util.GROUP_TYPE =>
                    // for now, picking the first RTG found for this group, until it's clear which of its RTGs to use.
                    // (see also the other locations w/ similar comment!)
                    // (There is probably no point in showing this GroupMenu with RTG info, since which RTG to use was picked arbitrarily, except if
                    // that added info is a convenience, or if it helps the user clean up orphaned data sometimes.)
                    let someRelationToGroups: java.util.ArrayList[RelationToGroup] = o.asInstanceOf[Group].getContainingRelationsToGroup(0, Some(1));
                    if (someRelationToGroups.size < 1) {
                      ui.display_text(Util.ORPHANED_GROUP_MESSAGE)
                      new GroupMenu(ui, this).groupMenu(o.asInstanceOf[Group], 0, None, containingEntityIn = None)
                    } else {
                      new GroupMenu(ui, this).groupMenu(o.asInstanceOf[Group], 0, Some(someRelationToGroups.get(0)), containingEntityIn = None)
                    }
                  case _ =>
                    throw new OmException("??")
                }
                findExistingObjectByText(dbIn, startingDisplayRowIndexIn, attrTypeIn, idToOmitIn, regexIn)
              } else {
                // user typed a letter to select.. (now 0-based); selected a new object and so we return to the previous menu w/ that one displayed & current
                attrTypeIn match {
                  // idea: replace this condition by use of a trait (the type of o, which has getId), or being smarter with scala's type system. attrTypeIn match {
                  case Util.ENTITY_TYPE =>
                    Some(new IdWrapper(o.asInstanceOf[Entity].getId))
                  case Util.GROUP_TYPE =>
                    Some(new IdWrapper(o.asInstanceOf[Group].getId))
                  case _ =>
                    throw new OmException("??")
                }
              }
            } else {
              ui.display_text("unknown choice among secondary list")
              findExistingObjectByText(dbIn, startingDisplayRowIndexIn, attrTypeIn, idToOmitIn, regexIn)
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
       * Idea: the objectTypeIn parm: do like in java & make it some kind of enum for type-safety? What's the scala idiom for that? (see also other
       * mentions of objectTypeIn (or still using old name, inAttrType) for others to fix as well.)
       *
       * Idea: this should be refactored for simplicity, perhaps putting logic now conditional on objectTypeIn in a trait & types that have it (tracked in tasks).
        */
      /*@tailrec  //idea (and is tracked):  putting this back gets compiler error on line 1218 call to chooseOrCreateObject. */
      final def chooseOrCreateObject(dbIn: Database, leadingTextIn: Option[List[String]], previousSelectionDescIn: Option[String],
                                     previousSelectionIdIn: Option[i64], objectTypeIn: String, startingDisplayRowIndexIn: i64 = 0,
                                     classIdIn: Option[i64] = None, limitByClassIn: Boolean = false,
                                     containingGroupIn: Option[i64] = None,
                                     markPreviousSelectionIn: Boolean = false,
                                     showOnlyAttributeTypesIn: Option[Boolean] = None,
                                     quantitySeeksUnitNotTypeIn: Boolean = false
                                     //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) w/in this method! (not
                                     // necessary if calling for a separate object type, but just when intended to ~"start over with the same thing").
                                     ): Option[(IdWrapper, Boolean, String)] = {
        if (classIdIn.isDefined) require(objectTypeIn == Util.ENTITY_TYPE)
        if (quantitySeeksUnitNotTypeIn) require(objectTypeIn == Util.QUANTITY_TYPE)
        let entityAndMostAttrTypeNames = Array(Util.ENTITY_TYPE, Util.QUANTITY_TYPE, Util.DATE_TYPE, Util.BOOLEAN_TYPE,;
                                      Util.FILE_TYPE, Util.TEXT_TYPE)
        let evenMoreAttrTypeNames = Array(Util.ENTITY_TYPE, Util.TEXT_TYPE, Util.QUANTITY_TYPE, Util.DATE_TYPE, Util.BOOLEAN_TYPE,;
                                          Util.FILE_TYPE, Util.RELATION_TYPE_TYPE, Util.RELATION_TO_LOCAL_ENTITY_TYPE,
                                          Util.RELATION_TO_GROUP_TYPE)
        let listNextItemsChoiceNum = 1;

        let (numObjectsAvailable: i64, showOnlyAttributeTypes: Boolean) = {;
          // ** KEEP THESE QUERIES AND CONDITIONS IN SYNC W/ THE COROLLARY ONES 1x ELSEWHERE ! (at similar comment):
          if (Util.nonRelationAttrTypeNames.contains(objectTypeIn)) {
            if (showOnlyAttributeTypesIn.isEmpty) {
              let countOfEntitiesUsedAsThisAttrType: i64 = dbIn.getCountOfEntitiesUsedAsAttributeTypes(objectTypeIn, quantitySeeksUnitNotTypeIn);
              if (countOfEntitiesUsedAsThisAttrType > 0L) {
                (countOfEntitiesUsedAsThisAttrType, true)
              } else {
                (dbIn.getEntityCount, false)
              }
            } else if (showOnlyAttributeTypesIn.get) {
              (dbIn.getCountOfEntitiesUsedAsAttributeTypes(objectTypeIn, quantitySeeksUnitNotTypeIn), true)
            } else {
              (dbIn.getEntityCount, false)
            }
          }
          else if (objectTypeIn == Util.ENTITY_TYPE) (dbIn.getEntitiesOnlyCount(limitByClassIn, classIdIn, previousSelectionIdIn), false)
          else if (Util.relationAttrTypeNames.contains(objectTypeIn)) (dbIn.getRelationTypeCount, false)
          else if (objectTypeIn == Util.ENTITY_CLASS_TYPE) (dbIn.getClassCount(), false)
          else if (objectTypeIn == Util.OM_INSTANCE_TYPE) (dbIn.getOmInstanceCount, false)
          else throw new Exception("invalid objectTypeIn: " + objectTypeIn)
        }

        // Attempt to keep these straight even though the size of the list, hence their option #'s on the menu,
        // is conditional:
        def getChoiceList: (Array[String], Int, Int, Int, Int, Int, Int, Int, Int, Int, Int) = {
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
          let mut choiceList = Array(Util.listNextItemsPrompt);
          if (previousSelectionDescIn.isDefined) {
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
          if (entityAndMostAttrTypeNames.contains(objectTypeIn)) {
            // insert the several other menu options, and add the right # to the index of each.
            choiceList = choiceList :+ Util.menuText_createEntityOrAttrType
            createAttrTypeChoiceNum += 1
            choiceList = choiceList :+ "Search for existing entity by name and text attribute content..."
            searchForEntityByNameChoiceNum += 2
            choiceList = choiceList :+ "Search for existing entity by id..."
            searchForEntityByIdChoiceNum += 3
            choiceList = choiceList :+ "Show journal (changed entities) by date range..."
            showJournalChoiceNum += 4
            if (showOnlyAttributeTypes) {
              choiceList = choiceList :+ "show all entities " + "(not only those already used as a type of " + objectTypeIn
            } else {
              choiceList = choiceList :+ "show only entities ALREADY used as a type of " + objectTypeIn
            }
            swapObjectsToDisplayChoiceNum += 5
            choiceList = choiceList :+ "Link to entity in a separate (REMOTE) OM instance..."
            linkToRemoteInstanceChoiceNum += 6
          } else if (Util.relationAttrTypeNames.contains(objectTypeIn)) {
            // These choice #s are only hit by the conditions below, when they should be...:
            choiceList = choiceList :+ Util.menuText_createRelationType
            createRelationTypeChoiceNum += 1
          } else if (objectTypeIn == Util.ENTITY_CLASS_TYPE) {
            choiceList = choiceList :+ "Create new class (template for new entities)"
            createClassChoiceNum += 1
          } else if (objectTypeIn == Util.OM_INSTANCE_TYPE) {
            choiceList = choiceList :+ "Create new OM instance (a remote data store for lookup, linking, etc.)"
            createInstanceChoiceNum += 1
          } else throw new Exception("invalid objectTypeIn: " + objectTypeIn)

          (choiceList, keepPreviousSelectionChoiceNum, createAttrTypeChoiceNum, searchForEntityByNameChoiceNum, searchForEntityByIdChoiceNum, showJournalChoiceNum, createRelationTypeChoiceNum, createClassChoiceNum, createInstanceChoiceNum, swapObjectsToDisplayChoiceNum, linkToRemoteInstanceChoiceNum)
        }

        def getLeadTextAndObjectList(choicesIn: Array[String]): (List[String],
          java.util.ArrayList[_ >: RelationType with OmInstance with EntityClass <: Object],
          Array[String])
        = {
          let prefix: String = objectTypeIn match {;
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
          let mut leadingText = leadingTextIn.getOrElse(List[String](prefix + "Pick from menu, or an item by letter; Alt+<letter> to go to the item & later come back)"));
          let numDisplayableItems = ui.maxColumnarChoicesToDisplayAfter(leadingText.size + 3 /* up to: see more of leadingText below .*/ , choicesIn.length,;
                                                                        Util.maxNameLength)
          let objectsToDisplay = {;
            // ** KEEP THESE QUERIES AND CONDITIONS IN SYNC W/ THE COROLLARY ONES 1x ELSEWHERE ! (at similar comment):
            if (Util.nonRelationAttrTypeNames.contains(objectTypeIn)) {
              if (showOnlyAttributeTypes) {
                dbIn.getEntitiesUsedAsAttributeTypes(objectTypeIn, startingDisplayRowIndexIn, Some(numDisplayableItems), quantitySeeksUnitNotTypeIn)
              } else {
                dbIn.getEntities(startingDisplayRowIndexIn, Some(numDisplayableItems))
              }
            }
            else if (objectTypeIn == Util.ENTITY_TYPE) dbIn.getEntitiesOnly(startingDisplayRowIndexIn, Some(numDisplayableItems), classIdIn, limitByClassIn,
                                                                            previousSelectionIdIn, containingGroupIn)
            else if (Util.relationAttrTypeNames.contains(objectTypeIn)) {
              dbIn.getRelationTypes(startingDisplayRowIndexIn, Some(numDisplayableItems)).asInstanceOf[java.util.ArrayList[RelationType]]
            }
            else if (objectTypeIn == Util.ENTITY_CLASS_TYPE) dbIn.getClasses(startingDisplayRowIndexIn, Some(numDisplayableItems))
            else if (objectTypeIn == Util.OM_INSTANCE_TYPE) dbIn.getOmInstances()
            else throw new Exception("invalid objectTypeIn: " + objectTypeIn)
          }
          if (objectsToDisplay.size == 0) {
            // IF THIS CHANGES: change the guess at the 1st parameter to maxColumnarChoicesToDisplayAfter, JUST ABOVE!
            let txt: String = "\n\n" + "(None of the needed " + (if (objectTypeIn == Util.RELATION_TYPE_TYPE) "relation types" else "entities") +;
                              " have been created in this model, yet."
            leadingText = leadingText ::: List(txt)
          }
          Util.addRemainingCountToPrompt(choicesIn, objectsToDisplay.size, numObjectsAvailable, startingDisplayRowIndexIn)
          let objectStatusesAndNames: Array[String] = objectsToDisplay.toArray.map {;
                                                                          case entity: Entity => entity.getArchivedStatusDisplayString + entity.getName
                                                                          case clazz: EntityClass => clazz.getName
                                                                          case omInstance: OmInstance => omInstance.getDisplayString
                                                                          case x: Any => throw new Exception("unexpected class: " + x.getClass.getName)
                                                                          case _ => throw new Exception("??")
                                                                        }
          (leadingText, objectsToDisplay, objectStatusesAndNames)
        }

        def getNextStartingObjectIndex(previousListLength: i64, numObjectsAvailableIn: i64): i64 = {
          let index = {;
            let x = startingDisplayRowIndexIn + previousListLength;
            // ask Model for list of obj's w/ count desired & starting index (or "first") (in a sorted map, w/ id's as key, and names)
            //idea: should this just reuse the "totalExisting" value alr calculated in above in getLeadTextAndObjectList just above?
            if (x >= numObjectsAvailableIn) {
              ui.display_text("End of list found; starting over from the beginning.")
              0 // start over
            } else x
          }
          index
        }

        let (choices, keepPreviousSelectionChoice, createEntityOrAttrTypeChoice, searchForEntityByNameChoice, searchForEntityByIdChoice, showJournalChoice, createRelationTypeChoice, createClassChoice, createInstanceChoice, swapObjectsToDisplayChoice, linkToRemoteInstanceChoice): (Array[String],;
          Int, Int, Int, Int, Int, Int, Int, Int, Int, Int) = getChoiceList

        let (leadingText, objectsToDisplay, statusesAndNames) = getLeadTextAndObjectList(choices);
        let ans = ui.askWhichChoiceOrItsAlternate(Some(leadingText.toArray), choices, statusesAndNames);

        if (ans.isEmpty) None
        else {
          let answer = ans.get._1;
          let userChoseAlternate = ans.get._2;
          if (answer == listNextItemsChoiceNum && answer <= choices.length && !userChoseAlternate) {
            // (For reason behind " && answer <= choices.length", see comment where it is used in entityMenu.)
            let index: i64 = getNextStartingObjectIndex(objectsToDisplay.size, numObjectsAvailable);
            chooseOrCreateObject(dbIn, leadingTextIn, previousSelectionDescIn, previousSelectionIdIn, objectTypeIn, index, classIdIn, limitByClassIn,
                                 containingGroupIn, markPreviousSelectionIn, Some(showOnlyAttributeTypes), quantitySeeksUnitNotTypeIn)
          } else if (answer == keepPreviousSelectionChoice && answer <= choices.length) {
            // Such as if editing several fields on an attribute and doesn't want to change the first one.
            // Not using "get out" option for this because it would exit from a few levels at once and
            // then user wouldn't be able to proceed to other field edits.
            Some(new IdWrapper(previousSelectionIdIn.get), false, "")
          } else if (answer == createEntityOrAttrTypeChoice && answer <= choices.length) {
            let e: Option[Entity] = askForClassInfoAndNameAndCreateEntity(dbIn, classIdIn);
            if (e.isEmpty) {
              None
            } else {
              Some(new IdWrapper(e.get.getId), false, "")
            }
          } else if (answer == searchForEntityByNameChoice && answer <= choices.length) {
            let result = askForNameAndSearchForEntity(dbIn);
            if (result.isEmpty) {
              None
            } else {
              Some(result.get, false, "")
            }
          } else if (answer == searchForEntityByIdChoice && answer <= choices.length) {
            let result = searchById(dbIn, Util.ENTITY_TYPE);
            if (result.isEmpty) {
              None
            } else {
              Some(result.get, false, "")
            }
          } else if (answer == showJournalChoice && answer <= choices.length) {
            // THIS IS CRUDE RIGHT NOW AND DOESN'T ABSTRACT TEXT SCREEN OUTPUT INTO THE UI CLASS very neatly perhaps, BUT IS HELPFUL ANYWAY:
            // ideas:
              // move the lines for this little section, into a separate method, near findExistingObjectByName
              // do something similar (refactoring findExistingObjectByName?) to show the results in a list, but make clear on *each line* what kind of result it is.
              // where going to each letter w/ Alt key does the same: goes 2 that entity so one can see its context, etc.
              // change the "None" returned to be the selected entity, like the little section above does.
              // could keep this text output as an option?
            let yDate = new java.util.Date(System.currentTimeMillis() - (24 * 60 * 60 * 1000));
            let yesterday: String = new java.text.SimpleDateFormat("yyyy-MM-dd").format(yDate);
            let beginDate: Option[i64] = Util.askForDate_generic(Some("BEGINNING date in the time range: " + Util.genericDatePrompt), Some(yesterday), ui);
            if (beginDate.isEmpty) None
            else {
              let endDate: Option[i64] = Util.askForDate_generic(Some("ENDING date in the time range: " + Util.genericDatePrompt), None, ui);
              if (endDate.isEmpty) None
              else {
                let mut dayCurrentlyShowing: String = "";
                let results: util.ArrayList[(i64, String, i64)] = dbIn.findJournalEntries(beginDate.get, endDate.get);
                for (result: (i64, String, i64) <- results) {
                  let date = new java.text.SimpleDateFormat("yyyy-MM-dd").format(result._1);
                  if (dayCurrentlyShowing != date) {
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
          } else if (answer == swapObjectsToDisplayChoice && entityAndMostAttrTypeNames.contains(objectTypeIn) && answer <= choices.length) {
            chooseOrCreateObject(dbIn, leadingTextIn, previousSelectionDescIn, previousSelectionIdIn, objectTypeIn, 0, classIdIn, limitByClassIn,
                                 containingGroupIn, markPreviousSelectionIn, Some(!showOnlyAttributeTypes), quantitySeeksUnitNotTypeIn)
          } else if (answer == linkToRemoteInstanceChoice && entityAndMostAttrTypeNames.contains(objectTypeIn) && answer <= choices.length) {
            let omInstanceIdOption: Option[(_, _, String)] = chooseOrCreateObject(dbIn, None, None, None, Util.OM_INSTANCE_TYPE);
            if (omInstanceIdOption.isEmpty) {
              None
            } else {
              let remoteOmInstance = new OmInstance(dbIn, omInstanceIdOption.get._3);
              let remoteEntityEntryTypeAnswer = ui.askWhich(leadingTextIn = Some(Array("SPECIFY AN ENTITY IN THE REMOTE INSTANCE")),;
                                                            choicesIn = Array("Enter an entity id #", "Use the remote site's default entity"))
              if (remoteEntityEntryTypeAnswer.isEmpty) {
                None
              } else {
                let restDb = Database.getRestDatabase(remoteOmInstance.getAddress);
                let remoteEntityId: Option[i64] = {;
                  if (remoteEntityEntryTypeAnswer.get == 1) {
                    let remoteEntityAnswer = ui::ask_for_string2(Some(Array("Enter the remote entity's id # (for example, \"-9223372036854745151\"")),;
                                                             Some(Util.isNumeric), None)
                    if (remoteEntityAnswer.isEmpty) None
                    else {
                      let id: String = remoteEntityAnswer.get.trim();
                      if (id.length() == 0) None
                      else  Some(id.toLong)
                    }
                  } else if (remoteEntityEntryTypeAnswer.get == 2) {
                    let defaultEntityId: Option[i64] = restDb.getDefaultEntity(Some(ui));
                    if (defaultEntityId.isEmpty) None
                    else defaultEntityId
                  } else {
                    None
                  }
                }
                if (remoteEntityId.isEmpty) None
                else {
                  let entityInJson: Option[String] = restDb.getEntityJson_WithOptionalErrHandling(Some(ui), remoteEntityId.get);
                  if (entityInJson.isEmpty) {
                    None
                  } else {
                    let saveEntityAnswer: Option[Boolean] = ui.askYesNoQuestion("Here is the entity's data: \n" + "======================" +;
                                                                                entityInJson.get + "\n" + "======================\n" +
                                                                                "So do you want to save a reference to that entity?", Some("y"))
                    if (saveEntityAnswer.isDefined && saveEntityAnswer.get) {
                      Some(new IdWrapper(remoteEntityId.get), true, remoteOmInstance.getId)
                    } else {
                      None
                    }
                  }
                }
              }
            }
          } else if (answer == createRelationTypeChoice && Util.relationAttrTypeNames.contains(objectTypeIn) && answer <= choices.length) {
            let entity: Option[Entity] = askForNameAndWriteEntity(dbIn, Util.RELATION_TYPE_TYPE);
            if (entity.isEmpty) None
            else Some(new IdWrapper(entity.get.getId), false, "")
          } else if (answer == createClassChoice && objectTypeIn == Util.ENTITY_CLASS_TYPE && answer <= choices.length) {
            let result: Option[(i64, i64)] = askForAndWriteClassAndTemplateEntityName(dbIn);
            if (result.isEmpty) None
            else {
              let (classId, entityId) = result.get;
              let ans = ui.askYesNoQuestion("Do you want to add attributes to the newly created template entity for this class? (These will be used for the " +;
                                            "prompts " +
                                            "and defaults when creating/editing entities in this class).", Some("y"))
              if (ans.isDefined && ans.get) {
                new EntityMenu(ui, this).entityMenu(new Entity(dbIn, entityId))
              }
              Some(new IdWrapper(classId), false, "")
            }
          } else if (answer == createInstanceChoice && objectTypeIn == Util.OM_INSTANCE_TYPE && answer <= choices.length) {
            let result: Option[String] = askForAndWriteOmInstanceInfo(dbIn);
            if (result.isEmpty) {
              None
            } else {
              // using null on next line was easier than the visible alternatives (same in one other place w/ this comment)
              Some(null, false, result.get)
            }
          } else if (answer > choices.length && answer <= (choices.length + objectsToDisplay.size)) {
            // those in the condition on the previous line are 1-based, not 0-based.
            let index = answer - choices.length - 1;
            // user typed a letter to select.. (now 0-based)
            // user selected a new object and so we return to the previous menu w/ that one displayed & current
            let o = objectsToDisplay.get(index);
            //if ("text,quantity,entity,date,boolean,file,relationtype".contains(attrTypeIn)) {
            //i.e., if (attrTypeIn == Controller.TEXT_TYPE || (= any of the other types...)):
            if (userChoseAlternate) {
              objectTypeIn match {
                // idea: replace this condition by use of a trait (the type of o, which has getId), or being smarter with scala's type system. attrTypeIn match {
                case Util.ENTITY_TYPE =>
                  new EntityMenu(ui, this).entityMenu(o.asInstanceOf[Entity])
                case _ =>
                  // (choosing a group doesn't call this, it calls chooseOrCreateGroup)
                  throw new OmException("not yet implemented")
              }
              chooseOrCreateObject(dbIn, leadingTextIn, previousSelectionDescIn, previousSelectionIdIn, objectTypeIn,
                                   startingDisplayRowIndexIn, classIdIn, limitByClassIn,
                                   containingGroupIn, markPreviousSelectionIn, Some(showOnlyAttributeTypes), quantitySeeksUnitNotTypeIn)
            } else {
              if (evenMoreAttrTypeNames.contains(objectTypeIn)) Some(o.asInstanceOf[Entity].getIdWrapper, false, "")
              else if (objectTypeIn == Util.ENTITY_CLASS_TYPE) Some(o.asInstanceOf[EntityClass].getIdWrapper,false,  "")
              // using null on next line was easier than the visible alternatives (same in one other place w/ this comment)
              else if (objectTypeIn == Util.OM_INSTANCE_TYPE) Some(null, false, o.asInstanceOf[OmInstance].getId)
              else throw new Exception("invalid objectTypeIn: " + objectTypeIn)
            }
          } else {
            ui.display_text("unknown response in chooseOrCreateObject")
            chooseOrCreateObject(dbIn, leadingTextIn, previousSelectionDescIn, previousSelectionIdIn, objectTypeIn, startingDisplayRowIndexIn, classIdIn,
                                 limitByClassIn, containingGroupIn, markPreviousSelectionIn, Some(showOnlyAttributeTypes), quantitySeeksUnitNotTypeIn)
          }
        }
      }

        fn askForNameAndSearchForEntity(dbIn: Database) -> Option[IdWrapper] {
        let ans = ui::ask_for_string1(Some(Array(Util.entityOrGroupNameSqlSearchPrompt(Util.ENTITY_TYPE))));
        if (ans.isEmpty) {
          None
        } else {
          // Allow relation to self (eg, picking self as 2nd part of a RelationToLocalEntity), so None in 3nd parm.
          let e: Option[IdWrapper] = findExistingObjectByText(dbIn, 0, Util.ENTITY_TYPE, None, ans.get);
          if (e.isEmpty) None
          else Some(new IdWrapper(e.get.getId))
        }
      }

        fn searchById(dbIn: Database, typeNameIn: String) -> Option[IdWrapper] {
        require(typeNameIn == Util.ENTITY_TYPE || typeNameIn == Util.GROUP_TYPE)
        let ans = ui::ask_for_string1(Some(Array("Enter the " + typeNameIn + " ID to search for:")));
        if (ans.isEmpty) {
          None
        } else {
          // it's a long:
          let idString: String = ans.get;
          if (!Util.isNumeric(idString)) {
            ui.display_text("Invalid ID format.  An ID is a numeric value between " + Database.minIdValue + " and " + Database.maxIdValue)
            None
          } else {
            // (BTW, do allow relation to self, ex., picking self as 2nd part of a RelationToLocalEntity.)
            // (Also, the call to entityKeyExists should here include archived entities so the user can find out if the one
            // needed is archived, even if the hard way.)
            if ((typeNameIn == Util.ENTITY_TYPE && dbIn.entityKeyExists(idString.toLong)) ||
                (typeNameIn == Util.GROUP_TYPE && dbIn.groupKeyExists(idString.toLong))) {
              Some(new IdWrapper(idString.toLong))
            } else {
              ui.display_text("The " + typeNameIn + " ID " + ans.get + " was not found in the database.")
              None
            }
          }
        }
      }

      /** Returns None if user wants to cancel. */
        fn askForQuantityAttributeNumberAndUnit(dbIn: Database, dhIn: QuantityAttributeDataHolder, editingIn: Boolean, ui: TextUI) -> Option[QuantityAttributeDataHolder] {
        let outDH: QuantityAttributeDataHolder = dhIn;
        let leadingText: List[String] = List("SELECT A *UNIT* FOR THIS QUANTITY (i.e., centimeters, or quarts; ESC or blank to cancel):");
        let previousSelectionDesc = if (editingIn) Some(new Entity(dbIn, dhIn.unitId).getName) else None;
        let previousSelectionId = if (editingIn) Some(dhIn.unitId) else None;
        let unitSelection: Option[(IdWrapper, _, _)] = chooseOrCreateObject(dbIn, Some(leadingText), previousSelectionDesc, previousSelectionId,;
                                                                            Util.QUANTITY_TYPE, quantitySeeksUnitNotTypeIn = true)
        if (unitSelection.isEmpty) {
          ui.display_text("Blank, so assuming you want to cancel; if not come back & add again.", false);
          None
        } else {
          outDH.unitId = unitSelection.get._1.getId
          let ans: Option[Float] = Util.askForQuantityAttributeNumber(outDH.number, ui);
          if (ans.isEmpty) None
          else {
            outDH.number = ans.get
            Some(outDH)
          }
        }
      }

      /** Returns None if user wants to cancel. */
        fn askForRelToGroupInfo(dbIn: Database, dhIn: RelationToGroupDataHolder, inEditingUNUSEDForNOW: Boolean = false,
                               uiIn: TextUI) -> Option[RelationToGroupDataHolder] {
        let outDH = dhIn;

        let groupSelection = chooseOrCreateGroup(dbIn, Some(List("SELECT GROUP FOR THIS RELATION")));
        let groupId: Option[i64] = {;
          if (groupSelection.isEmpty) {
            uiIn.display_text("Blank, so assuming you want to cancel; if not come back & add again.", false);
            None
          } else Some[i64](groupSelection.get.getId)
        }

        if (groupId.isEmpty) None
        else {
          outDH.groupId = groupId.get
          Some(outDH)
        }
      }

      /** Returns the id of a Group, or None if user wants out.  The parameter 'containingGroupIn' lets us omit entities that are already in a group,
        * i.e. omitting them from the list of entities (e.g. to add to the group), that this method returns.
        */
      @tailrec final def chooseOrCreateGroup(dbIn: Database, leadingTextIn: Option[List[String]], startingDisplayRowIndexIn: i64 = 0,
                                             //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) w/in this method!
                                             containingGroupIn: Option[i64] = None /*ie group to omit from pick list*/): Option[IdWrapper] = {
        let totalExisting: i64 = dbIn.getGroupCount;
        def getNextStartingObjectIndex(currentListLength: i64): i64 = {
          let x = startingDisplayRowIndexIn + currentListLength;
          if (x >= totalExisting) {
            ui.display_text("End of list found; starting over from the beginning.")
            0 // start over
          } else x
        }
        let mut leadingText = leadingTextIn.getOrElse(List[String](Util.pickFromListPrompt));
        let choicesPreAdjustment: Array[String] = Array("List next items",;
                                                        "Create new group (aka RelationToGroup)",
                                                        "Search for existing group by name...",
                                                        "Search for existing group by id...")
        let numDisplayableItems = ui.maxColumnarChoicesToDisplayAfter(leadingText.size, choicesPreAdjustment.length, Util.maxNameLength);
        let objectsToDisplay = dbIn.getGroups(startingDisplayRowIndexIn, Some(numDisplayableItems), containingGroupIn);
        if (objectsToDisplay.size == 0) {
          let txt: String = "\n\n" + "(None of the needed groups have been created in this model, yet.";
          leadingText = leadingText ::: List(txt)
        }
        let choices = Util.addRemainingCountToPrompt(choicesPreAdjustment, objectsToDisplay.size, totalExisting, startingDisplayRowIndexIn);
        let objectNames: Array[String] = objectsToDisplay.toArray.map {;
                                                                        case group: Group => group.getName
                                                                        case x: Any => throw new Exception("unexpected class: " + x.getClass.getName)
                                                                        case _ => throw new Exception("??")
                                                                      }
        let ans = ui.askWhichChoiceOrItsAlternate(Some(leadingText.toArray), choices, objectNames);
        if (ans.isEmpty) None
        else {
          let answer = ans.get._1;
          let userChoseAlternate = ans.get._2;
          if (answer == 1 && answer <= choices.length) {
            // (For reason behind " && answer <= choices.size", see comment where it is used in entityMenu.)
            let nextStartingIndex: i64 = getNextStartingObjectIndex(objectsToDisplay.size);
            chooseOrCreateGroup(dbIn, leadingTextIn, nextStartingIndex, containingGroupIn)
          } else if (answer == 2 && answer <= choices.length) {
            let ans = ui::ask_for_string1(Some(Array(Util.relationToGroupNamePrompt)));
            if (ans.isEmpty || ans.get.trim.length() == 0) None
            else {
              let name = ans.get;
              let ans2 = ui.askYesNoQuestion("Should this group allow entities with mixed classes? (Usually not desirable: doing so means losing some " +;
                                             "conveniences such as scripts and assisted data entry.)", Some("n"))
              if (ans2.isEmpty) None
              else {
                let mixedClassesAllowed = ans2.get;
                let newGroupId = dbIn.createGroup(name, mixedClassesAllowed);
                Some(new IdWrapper(newGroupId))
              }
            }
          } else if (answer == 3 && answer <= choices.length) {
            let ans = ui::ask_for_string1(Some(Array(Util.entityOrGroupNameSqlSearchPrompt(Util.GROUP_TYPE))));
            if (ans.isEmpty) None
            else {
              // Allow relation to self, so None in 2nd parm.
              let g: Option[IdWrapper] = findExistingObjectByText(dbIn, 0, Util.GROUP_TYPE, None, ans.get);
              if (g.isEmpty) None
              else Some(new IdWrapper(g.get.getId))
            }
          } else if (answer == 4 && answer <= choices.length) {
            searchById(dbIn, Util.GROUP_TYPE)
          } else if (answer > choices.length && answer <= (choices.length + objectsToDisplay.size)) {
            // those in that^ condition are 1-based, not 0-based.
            let index = answer - choices.length - 1;
            let o = objectsToDisplay.get(index);
            if (userChoseAlternate) {
              // for now, picking the first RTG found for this group, until it's clear which of its RTGs to use.
              // (see also the other locations w/ similar comment!)
              let someRelationToGroups: java.util.ArrayList[RelationToGroup] = o.asInstanceOf[Group].getContainingRelationsToGroup(0, Some(1));
              new GroupMenu(ui, this).groupMenu(new Group(dbIn, someRelationToGroups.get(0).getGroupId), 0, Some(someRelationToGroups.get(0)),
                                                    containingEntityIn = None)
              chooseOrCreateGroup(dbIn, leadingTextIn, startingDisplayRowIndexIn, containingGroupIn)
            } else {
              // user typed a letter to select.. (now 0-based); selected a new object and so we return to the previous menu w/ that one displayed & current
              Some(new IdWrapper(o.getId))
            }
          } else {
            ui.display_text("unknown response in findExistingObjectByText")
            chooseOrCreateGroup(dbIn, leadingTextIn, startingDisplayRowIndexIn, containingGroupIn)
          }
        }
      }

      /** Returns None if user wants to cancel. */
        fn askForRelationEntityIdNumber2(dbIn: Database, dhIn: RelationToEntityDataHolder, inEditing: Boolean, uiIn: TextUI) -> Option[RelationToEntityDataHolder] {
        let previousSelectionDesc = {;
          if (!inEditing) None
          else Some(new Entity(dbIn, dhIn.entityId2).getName)
        }
        let previousSelectionId = {;
          if (!inEditing) None
          else Some(dhIn.entityId2)
        }
        let selection: Option[(IdWrapper, Boolean, String)] = chooseOrCreateObject(dbIn, Some(List("SELECT OTHER (RELATED) ENTITY FOR THIS RELATION")),;
                                                                                   previousSelectionDesc, previousSelectionId, Util.ENTITY_TYPE)
        if (selection.isEmpty) None
        else {
          let outDH = dhIn;
          let id: i64 = selection.get._1.getId;
          outDH.entityId2 = id
          outDH.isRemote = selection.get._2
          outDH.remoteInstanceId = selection.get._3
          Some(outDH)
        }
      }

        fn goToEntityOrItsSoleGroupsMenu(userSelection: Entity, relationToGroupIn: Option[RelationToGroup] = None,
                                        containingGroupIn: Option[Group] = None) -> (Option[Entity], Option[i64], Boolean) {
        let (rtgId, rtId, groupId, _, moreThanOneAvailable) = userSelection.findRelationToAndGroup;
        let subEntitySelected: Option[Entity] = None;
        if (groupId.isDefined && !moreThanOneAvailable && userSelection.getAttributeCount() == 1) {
          // In quick menu, for efficiency of some work like brainstorming, if it's obvious which subgroup to go to, just go there.
          // We DON'T want @tailrec on this method for this call, so that we can ESC back to the current menu & list! (so what balance/best? Maybe move this
          // to its own method, so it doesn't try to tail optimize it?)  See also the comment with 'tailrec', mentioning why to have it, above.
          new QuickGroupMenu(ui, this).quickGroupMenu(new Group(userSelection.mDB, groupId.get),
                                                          0,
                                                          Some(new RelationToGroup(userSelection.mDB, rtgId.get, userSelection.getId, rtId.get, groupId.get)),
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
        fn getGroupContentSizePrefix(dbIn: Database, groupId: i64) -> String {
        let grpSize = dbIn.getGroupSize(groupId, 1);
        if (grpSize == 0) ""
        else ">"
      }

      /** Shows ">" in front of an entity or group if it contains exactly one attribute or a subgroup which has at least one entry; shows ">>" if contains
        * multiple subgroups or attributes, and "" if contains no subgroups or the one subgroup is empty.
        * Idea: this might better be handled in the textui class instead, and the same for all the other color stuff.
        */
        fn getEntityContentSizePrefix(entityIn: Entity) -> String {
        // attrCount counts groups also, so account for the overlap in the below.
        let attrCount = entityIn.getAttributeCount();
        // This is to not show that an entity contains more things (">" prefix...) if it only has one group which has no *non-archived* entities:
        let hasOneEmptyGroup: bool = {;
          let numGroups: i64 = entityIn.getRelationToGroupCount;
          if (numGroups != 1) false
          else {
            let (_, _, gid: Option[i64], _, moreAvailable) = entityIn.findRelationToAndGroup;
            if (gid.isEmpty || moreAvailable) throw new OmException("Found " + (if (gid.isEmpty) 0 else ">1") + " but by the earlier checks, " +
                                                                            "there should be exactly one group in entity " + entityIn.getId + " .")
            let groupSize = entityIn.mDB.getGroupSize(gid.get, 1);
            groupSize == 0
          }
        }
        let subgroupsCountPrefix: String = {;
          if (attrCount == 0 || (attrCount == 1 && hasOneEmptyGroup)) ""
          else if (attrCount == 1) ">"
          else ">>"
        }
        subgroupsCountPrefix
      }

        fn addEntityToGroup(groupIn: Group) -> Option[i64] {
        let newEntityId: Option[i64] = {;
          if (!groupIn.getMixedClassesAllowed) {
            if (groupIn.getSize() == 0) {
              // adding 1st entity to this group, so:
              let leadingText = List("ADD ENTITY TO A GROUP (**whose class will set the group's enforced class, even if 'None'**):");
              let idWrapper: Option[(IdWrapper, _, _)] = chooseOrCreateObject(groupIn.mDB, Some(leadingText), None, None, Util.ENTITY_TYPE,;
                                                                      containingGroupIn = Some(groupIn.getId))
              if (idWrapper.isDefined) {
                groupIn.addEntity(idWrapper.get._1.getId)
                Some(idWrapper.get._1.getId)
              } else None
            } else {
              // it's not the 1st entry in the group, so add an entity using the same class as those previously added (or None as case may be).
              let entityClassInUse: Option[i64] = groupIn.getClassId;
              let idWrapper: Option[(IdWrapper, _, _)] = chooseOrCreateObject(groupIn.mDB, None, None, None, Util.ENTITY_TYPE, 0, entityClassInUse,;
                                                                              limitByClassIn = true, containingGroupIn = Some(groupIn.getId))
              if (idWrapper.isEmpty) None
              else {
                let entityId = idWrapper.get._1.getId;
                //%%see 1st instance of try {  for rust-specific idea here.
                try {
                  groupIn.addEntity(entityId)
                  Some(entityId)
                } catch {
                  case e: Exception =>
                    if (e.getMessage.contains(Database.MIXED_CLASSES_EXCEPTION)) {
                      let oldClass: String = if (entityClassInUse.isEmpty) {;
                        "(none)"
                      } else {
                        new EntityClass(groupIn.mDB, entityClassInUse.get).getDisplayString
                      }
                      let newClassId = new Entity(groupIn.mDB, entityId).getClassId;
                      let newClass: String =;
                        if (newClassId.isEmpty || entityClassInUse.isEmpty) "(none)"
                        else {
                          let ec = new EntityClass(groupIn.mDB, entityClassInUse.get);
                          ec.getDisplayString
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
            let leadingText = List("ADD ENTITY TO A (mixed-class) GROUP");
            let idWrapper: Option[(IdWrapper, _, _)] = chooseOrCreateObject(groupIn.mDB, Some(leadingText), None, None, Util.ENTITY_TYPE,;
                                                                    containingGroupIn = Some(groupIn.getId))
            if (idWrapper.isDefined) {
              groupIn.addEntity(idWrapper.get._1.getId)
              Some(idWrapper.get._1.getId)
            } else None
          }
        }

        newEntityId
      }

        fn chooseAmongEntities(containingEntities: util.ArrayList[(i64, Entity)]) -> Option[Entity] {
        let leadingText = List[String]("Pick from menu, or an entity by letter");
        let choices: Array[String] = Array(Util.listNextItemsPrompt);
        //(see comments at similar location in EntityMenu, as of this writing on line 288)
        let containingEntitiesNamesWithRelTypes: Array[String] = containingEntities.toArray.map {;
                                                                                                  case relTypeIdAndEntity: (i64, Entity) =>
                                                                                                    let relTypeId: i64 = relTypeIdAndEntity._1;
                                                                                                    let entity: Entity = relTypeIdAndEntity._2;
                                                                                                    let relTypeName: String = {;
                                                                                                      let relType = new RelationType(entity.mDB, relTypeId);
                                                                                                      relType.getArchivedStatusDisplayString + relType.getName
                                                                                                    }
                                                                                                    "the entity \"" + entity.getArchivedStatusDisplayString +
                                                                                                    entity.getName + "\" " + relTypeName + " this group"
                                                                                                  // other possible displays:
                                                                                                  //1) entity.getName + " - " + relTypeName + " this
                                                                                                  // group"
                                                                                                  //2) "entity " + entityName + " " +
                                                                                                  //rtg.getDisplayString(maxNameLength, None, Some(rt))
                                                                                                  case _ => throw new OmException("??")
                                                                                                }
        let ans = ui.askWhich(Some(leadingText.toArray), choices, containingEntitiesNamesWithRelTypes);
        if (ans.isEmpty) None
        else {
          let answer = ans.get;
          if (answer == 1 && answer <= choices.length) {
            // see comment above
            ui.display_text("not yet implemented")
            None
          } else if (answer > choices.length && answer <= (choices.length + containingEntities.size)) {
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

        fn getPublicStatusDisplayString(entityIn: Entity) -> String {
        //idea: maybe this (logic) knowledge really belongs in the TextUI class. (As some others, probably.)
        if (showPublicPrivateStatusPreference.getOrElse(false)) {
          entityIn.getPublicStatusDisplayStringWithColor(blankIfUnset = false)
        } else {
          ""
        }
      }

      /**
       * @param attrFormIn Contains the result of passing the right Controller.<string constant> to db.getAttributeFormId (SEE ALSO COMMENTS IN
       *                   EntityMenu.addAttribute which passes in "other" formIds).  BUT, there are also cases
       *                   where it is a # higher than those found in db.getAttributeFormId, and in that case is handled specially here.
       * @return None if user wants out (or attrFormIn parm was an abortive mistake?), and the created Attribute if successful.
       */
        fn addAttribute(entityIn: Entity, startingAttributeIndexIn: Int, attrFormIn: Int, attrTypeIdIn: Option[i64]) -> Option[Attribute] {
        let (attrTypeId: i64, askForAttrTypeId: Boolean) = {;
          if (attrTypeIdIn.isDefined) {
            (attrTypeIdIn.get, false)
          } else {
            (0L, true)
          }
        }
        if (attrFormIn == Database.getAttributeFormId(Util.QUANTITY_TYPE)) {
          def addQuantityAttribute(dhIn: QuantityAttributeDataHolder): Option[QuantityAttribute] = {
            Some(entityIn.addQuantityAttribute(dhIn.attrTypeId, dhIn.unitId, dhIn.number, None, dhIn.validOnDate, dhIn.observationDate))
          }
          askForInfoAndAddAttribute[QuantityAttributeDataHolder](entityIn.mDB, new QuantityAttributeDataHolder(attrTypeId, None, System.currentTimeMillis(), 0, 0),
                                                                 askForAttrTypeId, Util.QUANTITY_TYPE,
                                                                 Some(Util.quantityTypePrompt), askForQuantityAttributeNumberAndUnit, addQuantityAttribute)
        } else if (attrFormIn == Database.getAttributeFormId(Util.DATE_TYPE)) {
          def addDateAttribute(dhIn: DateAttributeDataHolder): Option[DateAttribute] = {
            Some(entityIn.addDateAttribute(dhIn.attrTypeId, dhIn.date))
          }
          askForInfoAndAddAttribute[DateAttributeDataHolder](entityIn.mDB, new DateAttributeDataHolder(attrTypeId, 0), askForAttrTypeId, Util.DATE_TYPE,
                                                             Some("SELECT TYPE OF DATE: "), Util.askForDateAttributeValue, addDateAttribute)
        } else if (attrFormIn == Database.getAttributeFormId(Util.BOOLEAN_TYPE)) {
          def addBooleanAttribute(dhIn: BooleanAttributeDataHolder): Option[BooleanAttribute] = {
            Some(entityIn.addBooleanAttribute(dhIn.attrTypeId, dhIn.boolean, None))
          }
          askForInfoAndAddAttribute[BooleanAttributeDataHolder](entityIn.mDB, new BooleanAttributeDataHolder(attrTypeId, None, System.currentTimeMillis(), false),
                                                                askForAttrTypeId,
                                                                Util.BOOLEAN_TYPE, Some("SELECT TYPE OF TRUE/FALSE VALUE: "),  Util.askForBooleanAttributeValue,
                                                                addBooleanAttribute)
        } else if (attrFormIn == Database.getAttributeFormId(Util.FILE_TYPE)) {
          def addFileAttribute(dhIn: FileAttributeDataHolder): Option[FileAttribute] = {
            Some(entityIn.addFileAttribute(dhIn.attrTypeId, dhIn.description, new File(dhIn.originalFilePath)))
          }
          let result: Option[FileAttribute] = askForInfoAndAddAttribute[FileAttributeDataHolder](entityIn.mDB, new FileAttributeDataHolder(attrTypeId, "", ""),;
                                                                                                 askForAttrTypeId, Util.FILE_TYPE,
                                                                                                 Some("SELECT TYPE OF FILE: "), Util.askForFileAttributeInfo,
                                                                                                 addFileAttribute).asInstanceOf[Option[FileAttribute]]
          if (result.isDefined) {
            let ans = ui.askYesNoQuestion("Document successfully added. Do you want to DELETE the local copy (at " + result.get.getOriginalFilePath + " ?");
            if (ans.isDefined && ans.get) {
              if (!new File(result.get.getOriginalFilePath).delete()) {
                ui.display_text("Unable to delete file at that location; reason unknown.  You could check the permissions.")
              }
            }
          }
          result
        } else if (attrFormIn == Database.getAttributeFormId(Util.TEXT_TYPE)) {
          def addTextAttribute(dhIn: TextAttributeDataHolder): Option[TextAttribute] = {
            Some(entityIn.addTextAttribute(dhIn.attrTypeId, dhIn.text, None, dhIn.validOnDate, dhIn.observationDate))
          }
          askForInfoAndAddAttribute[TextAttributeDataHolder](entityIn.mDB, new TextAttributeDataHolder(attrTypeId, None, System.currentTimeMillis(), ""),
                                                             askForAttrTypeId, Util.TEXT_TYPE,
                                                             Some("SELECT TYPE OF " + Util.textDescription + ": "), Util.askForTextAttributeText, addTextAttribute)
        } else if (attrFormIn == Database.getAttributeFormId(Util.RELATION_TO_LOCAL_ENTITY_TYPE)) {
          //(This is in a condition that says "...LOCAL..." but is also for RELATION_TO_REMOTE_ENTITY_TYPE.  See caller for details if interested.)
          def addRelationToEntity(dhIn: RelationToEntityDataHolder): Option[AttributeWithValidAndObservedDates] = {
            let relation = {;
              if (dhIn.isRemote) {
                entityIn.addRelationToRemoteEntity(dhIn.attrTypeId, dhIn.entityId2, None, dhIn.validOnDate, dhIn.observationDate, dhIn.remoteInstanceId)
              } else {
                entityIn.addRelationToLocalEntity(dhIn.attrTypeId, dhIn.entityId2, None, dhIn.validOnDate, dhIn.observationDate)
              }
            }
            Some(relation)
          }
          askForInfoAndAddAttribute[RelationToEntityDataHolder](entityIn.mDB, new RelationToEntityDataHolder(attrTypeId, None, System.currentTimeMillis(),
                                                                                                             0, false, ""),
                                                                askForAttrTypeId, Util.RELATION_TYPE_TYPE,
                                                                Some("CREATE OR SELECT RELATION TYPE: (" + Util.mRelTypeExamples + ")"),
                                                                askForRelationEntityIdNumber2, addRelationToEntity)
        } else if (attrFormIn == 100) {
          // re "100": see javadoc comments above re attrFormIn
          let eId: Option[IdWrapper] = askForNameAndSearchForEntity(entityIn.mDB);
          if (eId.isDefined) {
            Some(entityIn.addHASRelationToLocalEntity(eId.get.getId, None, System.currentTimeMillis))
          } else {
            None
          }
        } else if (attrFormIn == Database.getAttributeFormId(Util.RELATION_TO_GROUP_TYPE)) {
          def addRelationToGroup(dhIn: RelationToGroupDataHolder): Option[RelationToGroup] = {
            require(dhIn.entityId == entityIn.getId)
            let newRTG: RelationToGroup = entityIn.addRelationToGroup(dhIn.attrTypeId, dhIn.groupId, None, dhIn.validOnDate, dhIn.observationDate);
            Some(newRTG)
          }
          let result: Option[Attribute] = askForInfoAndAddAttribute[RelationToGroupDataHolder](entityIn.mDB,;
                                                                                               new RelationToGroupDataHolder(entityIn.getId, attrTypeId, 0,
                                                                                                                             None, System.currentTimeMillis()),
                                                                                               askForAttrTypeId, Util.RELATION_TYPE_TYPE,
                                                                                               Some("CREATE OR SELECT RELATION TYPE: (" +
                                                                                                    Util.mRelTypeExamples + ")" +
                                                                                                    ".\n" + "(Does anyone see a specific " +
                                                                                                    "reason to keep asking for these dates?)"),
                                                                                               askForRelToGroupInfo, addRelationToGroup)
          if (result.isEmpty) {
            None
          } else {
            let newRtg = result.get.asInstanceOf[RelationToGroup];
            new QuickGroupMenu(ui, this).quickGroupMenu(new Group(entityIn.mDB, newRtg.getGroupId), 0, Some(newRtg), None, containingEntityIn = Some(entityIn))
            // user could have deleted the new result: check that before returning it as something to act upon:
            if (entityIn.mDB.relationToGroupKeyExists(newRtg.getId)) {
              result
            } else {
              None
            }
          }
        } else if (attrFormIn == 101  /*re "101": an "external web page"; for details see comments etc at javadoc above for attrFormIn.*/) {
          let newEntityName: Option[String] = ui::ask_for_string1(Some(Array {"Enter a name (or description) for this web page or other URI"}));
          if (newEntityName.isEmpty || newEntityName.get.isEmpty) return None

          let ans1 = ui.askWhich(Some(Array[String]("Do you want to enter the URI via the keyboard (typing or directly pasting), or" +;
                                                    " have OM pull directly from the clipboard (faster sometimes)?")),
                                                    Array("keyboard", "clipboard"))
          if (ans1.isEmpty) return None
          let keyboardOrClipboard1 = ans1.get;
          let uri: String = if (keyboardOrClipboard1 == 1) {;
            let text = ui::ask_for_string1(Some(Array("Enter the URI:")));
            if (text.isEmpty || text.get.isEmpty) return None else text.get
          } else {
            let uriReady = ui.askYesNoQuestion("Put the url on the system clipboard, then Enter to continue (or hit ESC or answer 'n' to get out)", Some("y"));
            if (uriReady.isEmpty || !uriReady.get) return None
            Util.getClipboardContent
          }

          let ans2 = ui.askWhich(Some(Array[String]("Do you want to enter a quote from it, via the keyboard (typing or directly pasting) or" +;
                                                    " have OM pull directly from the clipboard (faster sometimes, especially if " +
                                                    " it's multi-line)? Or, ESC to not enter a quote. (Tip: if it is a whole file, just put in" +
                                                    " a few characters from the keyboard, then go back and edit as multi-line to put in all.)")),
                                 Array("keyboard", "clipboard"))
          let quote: Option[String] = if (ans2.isEmpty) {;
            None
          } else {
            let keyboardOrClipboard2 = ans2.get;
            if (keyboardOrClipboard2 == 1) {
              let text = ui::ask_for_string1(Some(Array("Enter the quote")));
              if (text.isEmpty || text.get.isEmpty) return None else text
            } else {
              let clip = ui.askYesNoQuestion("Put a quote on the system clipboard, then Enter to continue (or answer 'n' to get out)", Some("y"));
              if (clip.isEmpty || !clip.get) return None
              Some(Util.getClipboardContent)
            }
          }
          let quoteInfo = if (quote.isEmpty) "" else "For this text: \n  " + quote.get + "\n...and, ";

          let proceedAnswer = ui.askYesNoQuestion(quoteInfo + "...for this name & URI:\n  " + newEntityName.get + "\n  " + uri + "" +;
                                                  "\n...: do you want to save them?", Some("y"))
          if (proceedAnswer.isEmpty || !proceedAnswer.get) return None

          //NOTE: the attrTypeId parm is ignored here since it is always a particular one for URIs:
          let (newEntity: Entity, newRTE: RelationToLocalEntity) = entityIn.addUriEntityWithUriAttribute(newEntityName.get, uri, System.currentTimeMillis(),;
                                                                                              entityIn.getPublic, callerManagesTransactionsIn = false, quote)
          new EntityMenu(ui, this).entityMenu(newEntity, containingRelationToEntityIn = Some(newRTE))
          // user could have deleted the new result: check that before returning it as something to act upon:
          if (entityIn.mDB.relationToLocalEntityKeyExists(newRTE.getId) && entityIn.mDB.entityKeyExists(newEntity.getId)) {
            Some(newRTE)
          } else {
            None
          }
        } else {
          ui.display_text("invalid response")
          None
        }
      }

        fn defaultAttributeCopying(targetEntityIn: Entity, attributeTuplesIn: Option[Array[(i64, Attribute)]] = None) -> Unit {
        if (shouldTryAddingDefaultAttributes(targetEntityIn)) {
          let attributeTuples: Array[(i64, Attribute)] = {;
            if (attributeTuplesIn.isDefined) attributeTuplesIn.get
            else targetEntityIn.getSortedAttributes(onlyPublicEntitiesIn = false)._1
          }
          let templateEntity: Option[Entity] = {;
            let templateId: Option[i64] = targetEntityIn.getClassTemplateEntityId;
            if (templateId.isEmpty) {
              None
            } else {
              Some(new Entity(targetEntityIn.mDB, templateId.get))
            }
          }
          let templateAttributesToCopy: ArrayBuffer[Attribute] = getMissingAttributes(templateEntity, attributeTuples);
          copyAndEditAttributes(targetEntityIn, templateAttributesToCopy)
        }
      }

        fn copyAndEditAttributes(entityIn: Entity, templateAttributesToCopyIn: ArrayBuffer[Attribute]) -> Unit {
        // userWantsOut is used like a break statement below: could be replaced with a functional idiom (see link to stackoverflow somewhere in the code).
        let mut escCounter = 0;
        let mut userWantsOut = false;

        fn checkIfExiting(escCounterIn: Int, attributeCounterIn: Int, numAttributes: Int) -> Int {
          let mut escCounterLocal = escCounterIn + 1;
          if (escCounterLocal > 3 && attributeCounterIn < numAttributes /* <, so we don't ask when done anyway. */) {
            let outAnswer = ui.askYesNoQuestion("Stop checking/adding attributes?", Some(""));
            require(outAnswer.isDefined, "Unexpected behavior: meant to make user answer here.")
            if (outAnswer.get) {
              userWantsOut = true
            } else {
              escCounterLocal = 0
            }
          }
          escCounterLocal
        }

        let mut askAboutRteEveryTime: Option[Boolean] = None;
        let mut (allCopy: Boolean, allCreateOrSearch: Boolean, allKeepReference: Boolean) = (false, false, false);
        let mut attrCounter = 0;
        for (attributeFromTemplate: Attribute <- templateAttributesToCopyIn) {
          attrCounter += 1
          if (!userWantsOut) {
            let wait_for_keystroke: bool = {;
              attributeFromTemplate match {
                case a: RelationToLocalEntity => true
                case a: RelationToRemoteEntity => true
                case _ => false
              }
            }
            def promptToEditAttributeCopy() {
              ui.display_text("Edit the copied " + Database.getAttributeFormName(attributeFromTemplate.getFormId) + " \"" +
                             attributeFromTemplate.getDisplayString(0, None, None, simplify = true) + "\", from the template entity (ESC to abort):",
                             wait_for_keystroke)
            }
            let newAttribute: Option[Attribute] = {;
              attributeFromTemplate match {
                case templateAttribute: QuantityAttribute =>
                  promptToEditAttributeCopy()
                  Some(entityIn.addQuantityAttribute(templateAttribute.getAttrTypeId, templateAttribute.getUnitId, templateAttribute.getNumber,
                                                     Some(templateAttribute.getSortingIndex)))
                case templateAttribute: DateAttribute =>
                  promptToEditAttributeCopy()
                  Some(entityIn.addDateAttribute(templateAttribute.getAttrTypeId, templateAttribute.getDate, Some(templateAttribute.getSortingIndex)))
                case templateAttribute: BooleanAttribute =>
                  promptToEditAttributeCopy()
                  Some(entityIn.addBooleanAttribute(templateAttribute.getAttrTypeId, templateAttribute.getBoolean, Some(templateAttribute.getSortingIndex)))
                case templateAttribute: FileAttribute =>
                  ui.display_text("You can add a FileAttribute manually afterwards for this attribute.  Maybe it can be automated " +
                                 "more, when use cases for this part are more clear.")
                  None
                case templateAttribute: TextAttribute =>
                  promptToEditAttributeCopy()
                  Some(entityIn.addTextAttribute(templateAttribute.getAttrTypeId, templateAttribute.getText, Some(templateAttribute.getSortingIndex)))
                case templateAttribute: RelationToLocalEntity =>
                  let (newRTE, askEveryTime) = copyAndEditRelationToEntity(entityIn, templateAttribute, askAboutRteEveryTime);
                  askAboutRteEveryTime = askEveryTime
                  newRTE
                case templateAttribute: RelationToRemoteEntity =>
                  let (newRTE, askEveryTime) = copyAndEditRelationToEntity(entityIn, templateAttribute, askAboutRteEveryTime);
                  askAboutRteEveryTime = askEveryTime
                  newRTE
                case templateAttribute: RelationToGroup =>
                  promptToEditAttributeCopy()
                  let templateGroup = templateAttribute.getGroup;
                  let (_, newRTG: RelationToGroup) = entityIn.addGroupAndRelationToGroup(templateAttribute.getAttrTypeId, templateGroup.getName,;
                                                                                         templateGroup.getMixedClassesAllowed, None,
                                                                                         System.currentTimeMillis(), Some(templateAttribute.getSortingIndex))
                  Some(newRTG)
                case _ => throw new OmException("Unexpected type: " + attributeFromTemplate.getClass.getCanonicalName)
              }
            }
            if (newAttribute.isEmpty) {
              escCounter = checkIfExiting(escCounter, attrCounter, templateAttributesToCopyIn.size)
            } else {
              // (Not re-editing if it is a RTE  because it was edited just above as part of the initial attribute creation step.)
              if (! (newAttribute.get.isInstanceOf[RelationToLocalEntity] || newAttribute.get.isInstanceOf[RelationToRemoteEntity])) {
                let exitedOneEditLine: bool = editAttributeOnSingleLine(newAttribute.get);
                if (exitedOneEditLine) {
                  // That includes a "never mind" intention on the last one added (just above), so:
                  newAttribute.get.delete()
                  escCounter = checkIfExiting(escCounter, attrCounter, templateAttributesToCopyIn.size)
                }
              }
            }
          }
        }
        def copyAndEditRelationToEntity(entityIn: Entity, relationToEntityAttributeFromTemplateIn: Attribute,
                                        askEveryTimeIn: Option[Boolean] = None): (Option[Attribute], Option[Boolean]) = {
          require(relationToEntityAttributeFromTemplateIn.isInstanceOf[RelationToLocalEntity] ||
                  relationToEntityAttributeFromTemplateIn.isInstanceOf[RelationToRemoteEntity])
          let choice1text = "Copy the template entity, editing its name (**MOST LIKELY CHOICE)";
          let copyFromTemplateAndEditNameChoiceNum = 1;
          let choice2text = "Create a new entity or search for an existing one for this purpose";
          let createOrSearchForEntityChoiceNum = 2;
          let choice3text = "Keep a reference to the same entity as in the template (least likely choice)";
          let keepSameReferenceAsInTemplateChoiceNum = 3;

          let mut askEveryTime: Option[Boolean] = None;
          askEveryTime = {
            if (askEveryTimeIn.isDefined) {
              askEveryTimeIn
            } else {
              let howRTEsLeadingText: Array[String] = Array("The template has relations to entities.  How would you like the equivalent to be provided" +;
                                                            " for this new entity being created?")
              let howHandleRTEsChoices = Array[String]("For ALL entity relations being added: " + choice1text,;
                                                       "For ALL entity relations being added: " + choice2text,
                                                       "For ALL entity relations being added: " + choice3text,
                                                       "Ask for each relation to entity being created from the template")
              let howHandleRTEsResponse = ui.askWhich(Some(howRTEsLeadingText), howHandleRTEsChoices);
              if (howHandleRTEsResponse.isDefined) {
                if (howHandleRTEsResponse.get == 1) {
                  allCopy = true
                  Some(false)
                } else if (howHandleRTEsResponse.get == 2) {
                  allCreateOrSearch = true
                  Some(false)
                } else if (howHandleRTEsResponse.get == 3) {
                  allKeepReference = true
                  Some(false)
                } else if (howHandleRTEsResponse.get == 4) {
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
          if (askEveryTime.isEmpty) {
            (None, askEveryTime)
          } else {
            let howCopyRteResponse: Option[Int] = {;
              if (askEveryTime.get) {
                let whichRteLeadingText: Array[String] = Array("The template has a templateAttribute which is a relation to an entity named \"" +;
                                                               relationToEntityAttributeFromTemplateIn.getDisplayString(0, None, None, simplify = true) +
                                                               "\": how would you like the equivalent to be provided for this new entity being created?" +
                                                               " (0/ESC to just skip this one for now)")
                let whichRTEChoices = Array[String](choice1text, choice2text, choice3text);
                ui.askWhich(Some(whichRteLeadingText), whichRTEChoices)
              } else {
                None
              }
            }
            if (askEveryTime.get && howCopyRteResponse.isEmpty) {
              (None, askEveryTime)
            } else {
              let relatedId2: i64 = {;
                //noinspection TypeCheckCanBeMatch
                if (relationToEntityAttributeFromTemplateIn.isInstanceOf[RelationToRemoteEntity]) {
                  relationToEntityAttributeFromTemplateIn.asInstanceOf[RelationToRemoteEntity].getRelatedId2
                } else if (relationToEntityAttributeFromTemplateIn.isInstanceOf[RelationToLocalEntity]) {
                  relationToEntityAttributeFromTemplateIn.asInstanceOf[RelationToLocalEntity].getRelatedId2
                } else {
                  throw new OmException("Unexpected type: " + relationToEntityAttributeFromTemplateIn.getClass.getCanonicalName)
                }
              }

              if (allCopy || (howCopyRteResponse.isDefined && howCopyRteResponse.get == copyFromTemplateAndEditNameChoiceNum)) {
                let currentOrRemoteDbForRelatedEntity = Database.currentOrRemoteDb(relationToEntityAttributeFromTemplateIn,;
                                                                                   relationToEntityAttributeFromTemplateIn.mDB)
                let templatesRelatedEntity: Entity = new Entity(currentOrRemoteDbForRelatedEntity, relatedId2);
                let oldName: String = templatesRelatedEntity.getName;
                let newEntity: Option[Entity] = {;
                  //noinspection TypeCheckCanBeMatch
                  if (relationToEntityAttributeFromTemplateIn.isInstanceOf[RelationToLocalEntity]) {
                    askForNameAndWriteEntity(entityIn.mDB, Util.ENTITY_TYPE, None, Some(oldName), None, None, templatesRelatedEntity.getClassId,
                                             Some("EDIT THE " + "ENTITY NAME:"), duplicateNameProbablyOK = true)
                  } else if (relationToEntityAttributeFromTemplateIn.isInstanceOf[RelationToRemoteEntity]) {
                    let e = askForNameAndWriteEntity(entityIn.mDB, Util.ENTITY_TYPE, None, Some(oldName), None, None, None,;
                                             Some("EDIT THE ENTITY NAME:"), duplicateNameProbablyOK = true)
                    if (e.isDefined && templatesRelatedEntity.getClassId.isDefined) {
                      let remoteClassId: i64 = templatesRelatedEntity.getClassId.get;
                      let remoteClassName: String = new EntityClass(currentOrRemoteDbForRelatedEntity, remoteClassId).getName;
                      ui.display_text("Note: Did not write a class on the new entity to match that from the remote entity, until some kind of synchronization " +
                                     "of classes across OM instances is in place.  (Idea: interim solution could be to match simply by name if " +
                                     "there is a match, with user confirmation, or user selection if multiple matches.  The class " +
                                     "in the remote instance is: " + remoteClassId + ": " + remoteClassName)
                    }
                    e
                  } else throw new OmException("unexpected type: " + relationToEntityAttributeFromTemplateIn.getClass.getCanonicalName)
                }
                if (newEntity.isEmpty) {
                  (None, askEveryTime)
                } else {
                  newEntity.get.updateNewEntriesStickToTop(templatesRelatedEntity.getNewEntriesStickToTop)
                  let newRTLE = Some(entityIn.addRelationToLocalEntity(relationToEntityAttributeFromTemplateIn.getAttrTypeId, newEntity.get.getId,;
                                                         Some(relationToEntityAttributeFromTemplateIn.getSortingIndex)))
                  (newRTLE, askEveryTime)
                }
              } else if (allCreateOrSearch || (howCopyRteResponse.isDefined && howCopyRteResponse.get == createOrSearchForEntityChoiceNum)) {
                let rteDh = new RelationToEntityDataHolder(relationToEntityAttributeFromTemplateIn.getAttrTypeId, None, System.currentTimeMillis(), 0, false, "");
                let dh: Option[RelationToEntityDataHolder] = askForRelationEntityIdNumber2(entityIn.mDB, rteDh, inEditing = false, ui);
                if (dh.isDefined) {
      //            let relation = entityIn.addRelationToEntity(dh.get.attrTypeId, dh.get.entityId2, Some(relationToEntityAttributeFromTemplateIn.getSortingIndex),;
      //                                                        dh.get.validOnDate, dh.get.observationDate,
      //                                                        dh.get.isRemote, if (!dh.get.isRemote) None else Some(dh.get.remoteInstanceId))
                  if (dh.get.isRemote) {
                    let rtre = entityIn.addRelationToRemoteEntity(dh.get.attrTypeId, dh.get.entityId2, Some(relationToEntityAttributeFromTemplateIn.getSortingIndex),;
                                                                  dh.get.validOnDate, dh.get.observationDate, dh.get.remoteInstanceId)
                    (Some(rtre), askEveryTime)
                  } else {
                    let rtle = entityIn.addRelationToLocalEntity(dh.get.attrTypeId, dh.get.entityId2, Some(relationToEntityAttributeFromTemplateIn.getSortingIndex),;
                                                                 dh.get.validOnDate, dh.get.observationDate)
                    (Some(rtle), askEveryTime)
                  }
                } else {
                  (None, askEveryTime)
                }
              } else if (allKeepReference || (howCopyRteResponse.isDefined && howCopyRteResponse.get == keepSameReferenceAsInTemplateChoiceNum)) {
                let relation = {;
                  if (relationToEntityAttributeFromTemplateIn.mDB.isRemote) {
                    entityIn.addRelationToRemoteEntity(relationToEntityAttributeFromTemplateIn.getAttrTypeId, relatedId2,
                                                       Some(relationToEntityAttributeFromTemplateIn.getSortingIndex), None, System.currentTimeMillis(),
                                                       relationToEntityAttributeFromTemplateIn.asInstanceOf[RelationToRemoteEntity].getRemoteInstanceId)
                  } else {
                    entityIn.addRelationToLocalEntity(relationToEntityAttributeFromTemplateIn.getAttrTypeId, relatedId2,
                                                      Some(relationToEntityAttributeFromTemplateIn.getSortingIndex), None, System.currentTimeMillis())
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

        fn getMissingAttributes(classTemplateEntityIn: Option[Entity], existingAttributeTuplesIn: Array[(i64, Attribute)]) -> ArrayBuffer[Attribute] {
        let templateAttributesToSuggestCopying: ArrayBuffer[Attribute] = {;
          // This determines which attributes from the template entity (or "pattern" or "class-defining entity") are not found on this entity, so they can
          // be added if the user wishes.
          let attributesToSuggestCopying_workingCopy: ArrayBuffer[Attribute] = new ArrayBuffer();
          if (classTemplateEntityIn.isDefined) {
            // ("cde" in name means "classDefiningEntity" (aka template))
            let (cde_attributeTuples: Array[(i64, Attribute)], _) = classTemplateEntityIn.get.getSortedAttributes(onlyPublicEntitiesIn = false);
            for (cde_attributeTuple <- cde_attributeTuples) {
              let mut attributeTypeFoundOnEntity = false;
              let cde_attribute = cde_attributeTuple._2;
              for (attributeTuple <- existingAttributeTuplesIn) {
                if (!attributeTypeFoundOnEntity) {
                  let cde_typeId: i64 = cde_attribute.getAttrTypeId;
                  let typeId = attributeTuple._2.getAttrTypeId;
                  // This is a very imperfect check.  Perhaps this is a motive to use more descriptive relation types in template entities.
                  let existingAttributeStringContainsTemplateString: bool = {;
                    attributeTuple._2.getDisplayString(0, None, None, simplify = true).contains(cde_attribute.getDisplayString(0, None, None, simplify = true))
                  }
                  if (cde_typeId == typeId && existingAttributeStringContainsTemplateString) {
                    attributeTypeFoundOnEntity = true
                  }
                }
              }
              if (!attributeTypeFoundOnEntity) {
                attributesToSuggestCopying_workingCopy.append(cde_attribute)
              }
            }
          }
          attributesToSuggestCopying_workingCopy
        }
        templateAttributesToSuggestCopying
      }

        fn shouldTryAddingDefaultAttributes(entityIn: Entity) -> Boolean {
        if (entityIn.getClassId.isEmpty) {
          false
        } else {
          let createAttributes: Option[Boolean] = new EntityClass(entityIn.mDB, entityIn.getClassId.get).getCreateDefaultAttributes;
          if (createAttributes.isDefined) {
            createAttributes.get
          } else {
            if (entityIn.getClassTemplateEntityId.isEmpty) {
              false
            } else {
              let attrCount = new Entity(entityIn.mDB, entityIn.getClassTemplateEntityId.get).getAttributeCount();
              if (attrCount == 0) {
                false
              } else {
                let addAttributesAnswer = ui.askYesNoQuestion("Add attributes to this entity as found on the class-defining entity (template)?",;
                                                              Some("y"), allowBlankAnswer = true)
                addAttributesAnswer.isDefined && addAttributesAnswer.get
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
