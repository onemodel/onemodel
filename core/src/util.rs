/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003-2004 and 2008-2017 inclusive, 2019, and 2023-2025 inclusive, Luke A. Call.
    (That copyright statement was previously 2013-2015, until I remembered that much of Controller came from TextUI.scala, and TextUI.java before that.
    And this file initially came from Controller.scala.)
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
use crate::model::database::DataType;
//use crate::model::attribute_with_valid_and_observed_dates::AttributeWithValidAndObservedDates;
use crate::model::attribute::Attribute;
use crate::model::attribute_data_holder::*;
use crate::controllers::controller::Controller;
use crate::model::database::Database;
use crate::model::entity::Entity;
use crate::model::group::Group;
use crate::model::postgres::postgresql_database::PostgreSQLDatabase;
use crate::model::relation_to_local_entity::RelationToLocalEntity;
use crate::model::relation_to_remote_entity::RelationToRemoteEntity;
use std::any::{Any}; //%%, TypeId};
// use std::error::Error;
use std::str::FromStr;
// use crate::controllers::controller::Controller;
// use crate::model::relation_type::*;
use crate::text_ui::TextUI;
use anyhow::anyhow;
use chrono::format::ParseResult;
use chrono::LocalResult;
// use chrono::offset::LocalResult;
use chrono::prelude::*;
use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
// use futures::stream_select;
// use sqlx::PgPool;
use std::cell::RefCell;
use std::rc::Rc;
use std::string::ToString;
//ordered by decreasing verbosity:
// use tracing::{trace, debug, info, warn, error, Level};
// for actual use now:
use tracing::debug;
// use tracing_subscriber::FmtSubscriber;

/// This is just a place to put shared code ("Utility") until a grouping for some, or a better idea emerges.  Using it also
/// had (in Scala anyway) the benefit of making the Controller file smaller, so it is more quickly compiled (especially by the IDE).
pub struct Util {}

#[derive(PartialEq, Eq)]
enum DateType {
    VALID,
    OBSERVED,
}

// for explanation and an idea, see fn initialize_test_db() below
static TEST_DB_INIT: std::sync::Once = std::sync::Once::new();
static TEST_TRACING_INIT: std::sync::Once = std::sync::Once::new();
// static mut TEST_DB: Option<PostgreSQLDatabase> = None;

impl Util {
    /// These constants are here because their presence in database.rs prevents Database from being used
    /// as a trait object.  See https://doc.rust-lang.org/reference/items/traits.html#object-safety etc for details.
    /// (Maybe they could go into model/mod.rs or some new struct file instead; haven't tried that.)
    pub const DB_NAME_PREFIX: &'static str = "om_";
    // If next line ever changes, search the code for other places that also have it hard-coded, to change also
    // (ex., INSTALLING, first.exp or its successors, any .psql scripts, ....  "t1/x" is shorter to type
    // during manual testing than "testrunner/testrunner".
    pub const TEST_USER: &'static str = "t1";
    pub const TEST_PASS: &'static str = "x";
    pub const MIXED_CLASSES_EXCEPTION: &'static str =
        "All the entities in a group should be of the same class.";
    // so named to make it unlikely to collide by name with anything else:
    pub const SYSTEM_ENTITY_NAME: &'static str = ".system-use-only";
    // aka template entities:
    pub const CLASS_TEMPLATE_ENTITY_GROUP_NAME: &'static str = "class-defining entities";
    pub const THE_HAS_RELATION_TYPE_NAME: &'static str = "has";
    pub const THE_IS_HAD_BY_REVERSE_NAME: &'static str = "is had by";
    pub const EDITOR_INFO_ENTITY_NAME: &'static str = "editorInfo";
    pub const TEXT_EDITOR_INFO_ENTITY_NAME: &'static str = "textEditorInfo";
    pub const TEXT_EDITOR_COMMAND_ATTRIBUTE_TYPE_NAME: &'static str = "textEditorCommand";
    pub const PREF_TYPE_BOOLEAN: &'static str = "boolean";
    pub const PREF_TYPE_ENTITY_ID: &'static str = "entity_id";
    pub const TEMPLATE_NAME_SUFFIX: &'static str = "-template";
    pub const UNUSED_GROUP_ERR1: &'static str =
        "No available index found which is not already used. How would so many be used?";
    pub const UNUSED_GROUP_ERR2: &'static str = "Very unexpected, but could it be that you are running out of available sorting indexes!?  Have someone check, before you need to create, for example, a thousand more entities.";
    pub const GET_CLASS_DATA__RESULT_TYPES: &'static str = "String,i64,bool";
    pub const GET_RELATION_TYPE_DATA__RESULT_TYPES: &'static str = "String,String,String";
    pub const GET_OM_INSTANCE_DATA__RESULT_TYPES: &'static str = "bool,String,i64,i64";
    pub const GET_QUANTITY_ATTRIBUTE_DATA__RESULT_TYPES: &'static str =
        "i64,i64,i64,i64,i64,i64,Float";
    pub const GET_DATE_ATTRIBUTE_DATA__RESULT_TYPES: &'static str = "i64,i64,i64,i64";
    pub const GET_BOOLEAN_ATTRIBUTE_DATA__RESULT_TYPES: &'static str = "i64,bool,i64,i64,i64,i64";
    pub const GET_FILE_ATTRIBUTE_DATA__RESULT_TYPES: &'static str =
        "i64,String,i64,i64,i64,i64,String,bool,bool,bool,i64,String";
    pub const GET_TEXT_ATTRIBUTE_DATA__RESULT_TYPES: &'static str = "i64,String,i64,i64,i64,i64";
    pub const GET_RELATION_TO_GROUP_DATA_BY_ID__RESULT_TYPES: &'static str =
        "i64,i64,i64,i64,i64,i64,i64";
    pub const GET_RELATION_TO_GROUP_DATA_BY_KEYS__RESULT_TYPES: &'static str =
        "i64,i64,i64,i64,i64,i64,i64";
    pub const GET_RELATION_TO_LOCAL_ENTITY__RESULT_TYPES: &'static str = "i64,i64,i64,i64";
    pub const GET_RELATION_TO_REMOTE_ENTITY__RESULT_TYPES: &'static str = "i64,i64,i64,i64";
    pub const GET_GROUP_DATA__RESULT_TYPES: &'static str = "String,i64,bool,bool";
    pub const GET_ENTITY_DATA__RESULT_TYPES: &'static str = "String,i64,i64,bool,bool,bool";
    pub const GET_GROUP_ENTRIES_DATA__RESULT_TYPES: &'static str = "i64,i64";

    pub fn entity_name_length() -> u16 {
        160
    }

    // in postgres, one table "extends" the other (see comments in create_tables)
    pub fn relation_type_name_length() -> u16 {
        Self::entity_name_length()
    }

    pub fn class_name_length() -> u16 {
        Self::entity_name_length()
    }

    pub fn max_name_length() -> u16 {
        std::cmp::max(
            std::cmp::max(
                Self::entity_name_length(),
                Self::relation_type_name_length(),
            ),
            Self::class_name_length(),
        )
    }

    // %%later: use \n for now but maybe do platform-specifically, per this advice later, per
    // https://stackoverflow.com/questions/47541191/how-to-get-current-platform-end-of-line-character-sequence-in-rust
    // ... (but even so, is "\r" the Mac one, or what?).
    // or just use fns like "is_windows()" below, instead of compilation flags? is either preferred/faster one?
    pub const NEWLN: &'static str = "\n"; //was on JVM: System.getProperty("line.separator");

    // Might not be the most familiar date form for us Americans, but it seems the most useful in the widest
    // variety of situations, and more readable than with the "T" embedded in place of
    // the 1st space.  So, this approximates iso-8601.
    // these are for input.
    //was: new java.text.SimpleDateFormat("yyyy-MM-dd HH:mm:ss:SSS zzz");
    //Note: per chrono API docs (DateTime.format()), in version 4.4 it does not know time zone
    //abbreviations, so it can only print the offset with %Z.
    pub const DATEFORMAT: &'static str = "%Y-%m-%d %H:%M:%S:%3f %Z"; //the %Z output can be > 3 characters.
    pub const DATEFORMAT2: &'static str = "%Y-%m-%d %H:%M:%S %Z";
    pub const DATEFORMAT3: &'static str = "%Y-%m-%d %H:%M %Z";
    pub const DATEFORMAT4: &'static str = "%Y-%m-%d %H:%M";
    pub const DATEFORMAT5: &'static str = "%Y-%m-%d";
    //%%?:
    // the chrono crate does not seem to support the ERA (BC/AD), but instead shows negative years.
    // const DATEFORMAT_WITH_ERA: &'static str = "%Y-%m-%d %H:%M:%S:%3f %Z";
    // const DATEFORMAT_WITH_ERA: &'static str = "GGyyyy-MM-dd HH:mm:ss:SSS zzz";
    // const DATEFORMAT2_WITH_ERA: &'static str = "GGyyyy-MM-dd HH:mm:ss zzz";
    // const DATEFORMAT2_WITH_ERA: &'static str = "%Y-%m-%d %H:%M:%S %Z";
    // const DATEFORMAT3_WITH_ERA: &'static str = "GGyyyy-MM-dd HH:mm zzz";
    // const DATEFORMAT3_WITH_ERA: &'static str = "%Y-%m-%d %H:%M %Z";

    // The next 2 are from Attribute.scala or object Attribute{..}, and were used in
    // method useful_date_format, which has been moved here to satisfy rustc.
    // object Attribute{..} said: "unlike in Controller, these are intentionally a little different, for displaying also the day of the week:"
    // let DATEFORMAT = new java.text.SimpleDateFormat("EEE yyyy-MM-dd HH:mm:ss:SSS zzz");
    // let DATEFORMAT_WITH_ERA = new java.text.SimpleDateFormat("EEE GGyyyy-MM-dd HH:mm:ss:SSS zzz");

    // DON'T CHANGE this msg unless you also change the trap for it in TextUI.java.
    pub const DOES_NOT_EXIST: &'static str = " does not exist in database.";

    //these are here to avoid colliding with use of the same names within other code inside the class.
    // idea: see re enums and/or constants; update this style?
    pub const ENTITY_TYPE: &'static str = "Entity";
    pub const QUANTITY_TYPE: &'static str = "QuantityAttribute";
    pub const TEXT_TYPE: &'static str = "TextAttribute";
    pub const DATE_TYPE: &'static str = "DateAttribute";
    pub const BOOLEAN_TYPE: &'static str = "BooleanAttribute";
    pub const FILE_TYPE: &'static str = "FileAttribute";
    pub const NON_RELATION_ATTR_TYPE_NAMES: [&'static str; 5] = [
        Util::QUANTITY_TYPE,
        Util::DATE_TYPE,
        Util::BOOLEAN_TYPE,
        Util::FILE_TYPE,
        Util::TEXT_TYPE,
    ];
    //i.e., "relationTypeType", or the thing that we sometimes put in an attribute type parameter, though not exactly an attribute type, which is "RelationType":
    pub const RELATION_TYPE_TYPE: &'static str = "RelationType";
    // IF/WHEN EVER UPDATING THESE TABLE NAMES, also update in cleanTestAccount.psql:
    pub const RELATION_TO_LOCAL_ENTITY_TYPE: &'static str = "RelationToEntity";
    pub const RELATION_TO_GROUP_TYPE: &'static str = "RelationToGroup";
    pub const RELATION_TO_REMOTE_ENTITY_TYPE: &'static str = "RelationToRemoteEntity";
    //%%change this to an enum? and similar things used the same way at places where it is used?
    //%%move this to the attribute.rs file or its mod.rs maybe?:
    pub const RELATION_ATTR_TYPE_NAMES: [&'static str; 4] = [
        Util::RELATION_TYPE_TYPE,
        Util::RELATION_TO_LOCAL_ENTITY_TYPE,
        Util::RELATION_TO_REMOTE_ENTITY_TYPE,
        Util::RELATION_TO_GROUP_TYPE,
    ];
    pub const GROUP_TYPE: &'static str = "Group";
    pub const ENTITY_CLASS_TYPE: &'static str = "Class";
    pub const OM_INSTANCE_TYPE: &'static str = "Instance";
    pub const ORPHANED_GROUP_MESSAGE: &'static str = "There is no entity with a containing relation to the group (orphaned).  You might search for it \
                                           (by adding it as an attribute to some entity), \
                                           & see if it should be deleted, kept with an entity, or left out there floating.  \
                                           (While this is not an expected usage, it is allowed and does not imply data corruption.)";
    const UNSELECT_MOVE_TARGET_PROMPT_TEXT: &'static str =
        "Unselect current move target (if present; not necessary really)";
    // This says 'same screenful' because it's easier to assume that the returned index refers to the currently available
    // local collections (a subset of all possible entries, for display), than calling chooseOrCreateObject, and sounds as useful:
    const UNSELECT_MOVE_TARGET_LEADING_TEXT: &'static str = "CHOOSE AN ENTRY (that contains only one subgroup) FOR THE TARGET OF MOVES (choose from SAME SCREENFUL as \
                                                  now;  if the target contains 0 subgroups, or 2 or more subgroups, \
                                                  use other means to move entities to it until some kind of \"move anywhere\" feature is added):";
    // unused?:
    const DEFAULT_PREFERENCES_DEPTH: i32 = 10;
    // Don't change these: they get set and looked up in the data for preferences. Changing it would just require users to reset it though, and would
    // leave the old as clutter in the data.
    pub const USER_PREFERENCES: &'static str = "User preferences";
    pub const SHOW_PUBLIC_PRIVATE_STATUS_PREFERENCE: &'static str =
        "Should entity lists show public/private status for each?";
    pub const DEFAULT_ENTITY_PREFERENCE: &'static str =
        "Which entity should be displayed as default, when starting the program?";
    // (If change next line, also change the hard-coded use in the file first.exp.)
    const HEADER_CONTENT_TAG: &'static str = "htmlHeaderContent";
    const BODY_CONTENT_TAG: &'static str = "htmlInitialBodyContent";
    const FOOTER_CONTENT_TAG: &'static str = "htmlFooterContent";
    pub const LOCAL_OM_INSTANCE_DEFAULT_DESCRIPTION: &'static str =
        "(local: not for self-connection but to serve id to remotes)";
    pub const SELECT_ENTITY_START: &'static str =
        "SELECT e.id, e.name, e.class_id, e.insertion_date, e.public, e.archived, e.new_entries_stick_to_top ";

    pub fn get_clipboard_content() -> String {
        // let clipboard: java.awt.datatransfer.Clipboard = java.awt.Toolkit.getDefaultToolkit.getSystemClipboard;
        // let contents: String = clipboard.getContents(null).getTransferData(java.awt.datatransfer.DataFlavor.stringFlavor).toString;
        // contents.trim
        // //(example of placing data on the clipboard, for future reference:)
        // //val selection = new java.awt.datatransfer.StringSelection("someString")
        // //clipboard.setContents(selection, null)

        //%%implement above
        "not yet implemented".to_string()
    }

    pub fn is_windows() -> bool {
        let os = std::env::consts::OS;
        os.to_lowercase().eq("windows")
    }

    //%%
    /// SEE COMMENTS FOR find_entity_to_highlight_next.
    //%%AND SEE ITS RECENT MODS, to match here, to deal w/ usize issue, can't be negative, - 1 logic....  Not critical though.
    // fn find_attribute_to_highlight_next(object_set_size: Int, objects_to_display_in: Vec<Attribute>, removedOne: bool,
    //                                  previously_highlighted_index_in_obj_list_in: Int, previously_highlighted_entry_in: Attribute) -> Option[Attribute] {
    //   //NOTE: SIMILAR TO find_entity_to_highlight_next: WHEN MAINTAINING ONE, DO SIMILARLY ON THE OTHER, until they are merged maybe by using the scala type
    //   //system better.
    //   if removedOne {
    //     let new_obj_list_size = object_set_size - 1;
    //     let new_index_to_highlight = math.min(new_obj_list_size - 1, previously_highlighted_index_in_obj_list_in);
    //     if new_index_to_highlight >= 0 {
    //       if new_index_to_highlight != previously_highlighted_index_in_obj_list_in {
    //         Some(objects_to_display_in.get(new_index_to_highlight))
    //       } else {
    //         if new_index_to_highlight + 1 < new_obj_list_size - 1 { Some(objects_to_display_in.get(new_index_to_highlight + 1)) }
    //         } else if new_index_to_highlight - 1 >= 0 { Some(objects_to_display_in.get(new_index_to_highlight - 1)) }
    //         } else { None }
    //       }
    //     } else { None }
    //   } else { Some(previously_highlighted_entry_in) }
    // }

    pub fn get_default_user_login() -> Result<(String, &'static str), String> {
        //%%how do this on other platforms? windows at least? some crate? std doesn't seem to have a clear answer.
        //was in scala: (System.getProperty("user.name"), "x")
        match std::env::var("USER") {
            Ok(val) => Ok((val, "x")),
            Err(e) => {
                let msg = e.to_string();
                Err(msg)
            }
        }
    }

    // ****** MAKE SURE THE NEXT 2 LINES MATCH THE FORMAT of Controller.DATEFORMAT, AND THE USER EXAMPLES IN THIS CLASS' OUTPUT! ******
    // Making this mutable so that it can be changed for testing consistency (to use GMT for most tests so hopefully they will pass for developers in;
    // another time zone.  idea:  It seems like there's a better way to solve that though, maybe with a subclass of Controller in the test,
    // or of SimpleDateFormat.)
    //%%how handle it if the system time zone changes?
    //%%is there another way, to not need this? see where it is used.
    // let mut timezone: String = new java.text.SimpleDateFormat("zzz").format(System.currentTimeMillis());
    // // (This isn't intended to match the date represented by a long value of "0", but is intended to be a usable value to fill in the rest of whatever a user
    // // doesn't.  Perhaps assuming that the user will always put in a year if they put in anything (as currently enforced by the code at this time of writing).
    // fn blankDate -> String
    // {
    //     "1970-01-01 00:00:00:000 " + timezone
    // }

    pub const REL_TYPE_EXAMPLES: &'static str =
        "i.e., ownership of or \"has\" another entity, family tie, &c";

    // (the startup message already suggests that they create it with their own name, no need to repeat that here:    )
    pub const MENUTEXT_CREATE_ENTITY_OR_ATTR_TYPE: &'static str = "Add new entity (or new type like length, for use with quantity, true/false, date, text, or file attributes)";

    pub fn menutext_create_relation_type() -> String {
        format!("Add new relation type ({})", Util::REL_TYPE_EXAMPLES)
    }

    pub const MAIN_SEARCH_PROMPT: &'static str =
        "Search all / list existing entities (except quantity units, attr types, & relation types)";
    pub const MENUTEXT_VIEW_PREFERENCES: &'static str = "View preferences";

    pub const GENERIC_DATE_PROMPT: &'static str =
        "Please enter the date like this, w/ at least the year, \
    and other parts as desired: \"2013-01-31 23:59:59:999 MDT\"; zeros are allowed in all but the \
    yyyy-mm-dd).  \"BC\" or \"AD\" prefix allowed (before the year, with no space).";
    //%%THIS LINE CAN BE PUT BACK AFTER the bug is fixed so ESC really works.  See similar cmt elsewhere; tracked in tasks:
    //(in the above, after the "yyyy-mm-dd)."
    //"  Or ESC to exit.  " +
    pub const TOO_LONG_MESSAGE: &'static str = "value too long for type";

    /*%%
       fn entity_menu_leading_text(entity_in: Entity) {
       "**CURRENT ENTITY " + entity_in.get_id() + ": " + entity_in.get_display_string(/*%%with_color = */true)
     }
    */

    /*
            fn group_menu_leading_text(group_in: Group) {
            "**CURRENT GROUP " + group_in.get_id + ": " + group_in.get_display_string()
          }
    */

    pub const QUANTITY_TYPE_PROMPT: &'static str =
        "SELECT TYPE OF QUANTITY (type is like length or \
      volume, but not the measurement unit); ESC or leave both blank to cancel; \
      cancel if you need to create the needed type before selecting): ";

    pub const TEXT_DESCRIPTION: &'static str = "TEXT (ex., serial #)";

    pub fn can_edit_attribute_on_single_line(
        db_in: Rc<RefCell<dyn Database>>,
        attribute_in: &dyn Attribute,
    ) -> Result<bool, anyhow::Error> {
        //%%was:  ! attributeIn.isInstanceOf[FileAttribute]
        //%%ex: if self.db.get_attribute_form_name(attribute_in.get_form_id()?)? == Util::TEXT_TYPE {
        //%%above syas: pub const FILE_TYPE: &'static str = "FileAttribute";
        let type_name = db_in.borrow().get_attribute_form_name(attribute_in.get_form_id()?)?;
        let matches = type_name == Util::FILE_TYPE;
        Ok(matches)
        //Ok(!attribute_in.get_form_id()? == Util::FILE_TYPE)
    }

    //%%
    // fn get_usable_filename(original_file_path_in: &str) -> (String, String) {
    // FileAttribute.get_usable_filename(original_file_path_in)
    // }

    const ENTITY_PARTS_THAT_CAN_BE_AFFECTED: &'static str = "ALL its attributes, actions, and relations, but not entities or groups the relations refer to";

    pub const LIST_NEXT_ITEMS_PROMPT: &'static str = "List next items";
    const LIST_PREV_ITEMS_PROMPT: &'static str = "List previous items";
    pub const RELATION_TO_GROUP_NAME_PROMPT: &'static str =
        "Type a name for this group (ex., \"xyz list\"), then press Enter; blank or ESC to cancel";

    pub fn add_remaining_count_to_prompt(
        mut choices_in: Vec<String>,
        num_displayed_objects: u64,
        total_rows_available_in: u64,
        starting_display_row_index_in: u64,
    ) -> Result<Vec<String>, anyhow::Error> {
        let num_left =
            total_rows_available_in - starting_display_row_index_in - num_displayed_objects;
        //%%how know if the next blocks (position() search, mutation in "if") work right, when can't 
        //put breakpoints everywhere & step thru..? Can, w/ gdb or such? 
        //%% Same w/ all other code changed lately?
        let mut found = true;
        let index_of_prompt: usize =
            match choices_in.iter().position(|&x| x == Self::LIST_NEXT_ITEMS_PROMPT.to_string()) {
                Some(i) => {
                    //%%i64::try_from(i)?
                    i
                },
                None => {
                    found = false;
                    0
                },
            };
        if num_left > 0 && found {
            match choices_in.get(index_of_prompt) {
                None => {
                    // do nothing due to very unexpected error of not finding it, after finding it? Or, how show the 
                    // err?--pop it up and move on as we do elsewhere? what best fr user standpoint? fixing?
                }
                Some(found_entry) => {
                    *found_entry = format!("{} (of {} more)", Util::LIST_NEXT_ITEMS_PROMPT, num_left);
                }
            }
        }
        Ok(choices_in)
    }

    fn get_containing_entities_description(
        entity_count_non_archived_in: i64,
        entity_count_archived_in: i64,
    ) -> String {
        format!(
            "contained in {} entities, and in {} archived entities",
            entity_count_non_archived_in, entity_count_archived_in
        )
    }

    pub const PICK_FROM_LIST_PROMPT: &'static str = "Pick from menu, or an item by letter to select; Alt+<letter> to go to the item then come back here";

    fn search_prompt_part(type_in: &str) -> String {
        format!("Enter part of the {} name to search for.", type_in)
    }

    pub fn entity_or_group_name_sql_search_prompt(type_name_in: &str) -> String {
        let part = Self::search_prompt_part(type_name_in);
        format!("{}  (For the curious: it will be used in matching as a case-insensitive POSIX regex; details at  http://www.postgresql.org/docs/current/static/functions-matching.html#FUNCTIONS-POSIX-REGEXP .)",
                    part)
    }

    pub fn is_numeric(input: &str) -> Result<(), anyhow::Error> {
        match f64::from_str(input) {
            Err(e) => Err(anyhow!("{}", e.to_string())),
            Ok(num) => Ok(()),
        }
    }

    /// this makes sure it exists and can open readonly w/o errors (so, :exists & readable).
    fn input_file_valid(path: &str) -> Result<(), anyhow::Error> {
        let file = std::fs::OpenOptions::new().read(true).open(path);
        match file {
            Err(e) => Err(anyhow!("invalid input file: {}", e.to_string())),
            Ok(_) => Ok(()),
        }
    }

    /// The check to see if a long date string is valid comes later.
    /// Now that we allow 1-digit dates, there is nothing to ck really.
    // %%get rid of this and observed_date_criteria--unused or needed really?
    fn valid_on_date_criteria(_: &str) -> bool {
        true
    }

    /// Same comments as for valid_on_date_criteria:
    fn observed_date_criteria(_: &str) -> bool {
        true
    }

    // //%%used from places that we will keep, and which still need this?:
    //   fn throwableToString(e: Throwable) -> String {
    //     let stringWriter = new StringWriter();
    //     e.printStackTrace(new PrintWriter(stringWriter))
    //     stringWriter.toString
    //   }

    //name in Scala: fn handleException
    //fn handle_error(e: anyhow::Error, ui: TextUI, db: Rc<RefCell<dyn Database>>) {
    pub fn handle_error(e: anyhow::Error, ui: Rc<TextUI>, caller_location: &str) {
        //if e.isInstanceOf[org.postgresql.util.PSQLException] || e.isInstanceOf[OmDatabaseException] ||
        //    throwableToString(e).contains("ERROR: current transaction is aborted, \
        //    commands ignored until end of transaction block")) {
        //
        //    //Rollback I think happens automatically now with rust/sqlx, when the transaction variable
        //    //is dropped, without committing first. Or something like that. See comments
        //    elsewhere (search for rollback, probably).
        //  db.rollback_trans()
        //}

        // If changing this string (" - 1"), also change in first.exp that looks for
        // it (distinguished from " - 2" elsewhere).
        let ans = ui.ask_yes_no_question(
            format!(
                "An error occurred: \"{}: {}\". If you \
                can provide simple instructions to reproduce it consistently, maybe it \
                can be fixed - 1.  Do you want to see the detailed output?",
                //e.getClass.get_name, e.getMessage),
                caller_location,
                e.to_string()
            )
            .as_str(),
            "n",
            false,
        );
        if ans.is_some() && ans.unwrap() {
            //ui.display_text(throwableToString(e))
            //ui.display_text(e.to_string());
            ui.display_text1(Util::get_backtrace().as_str());
        }
    }

    // // %%maybe replace this w/ just the parse command from rust.  It is complicated, and unclear if necessary (now?).
    // For now, in the code that would call this, just force a specific format until the code is otherwise working. Then come back and:
    //  THEN?:, maybe just take the string, try parsing it w/ various formats, and if none of them work, give a msg & loop or let user get out.
    //   /// A helper method.  Returns the date as a i64 (java-style: ms since 1970 began, UTC), and true if there is a problem w/ the string and we need to ask again.
    // fn finish_and_parse_the_date(date_in: &str, blank_means_now: bool, ui: TextUI) -> (Option<i64>, bool) {
    //       //%%review/update this to accomodate a minus sign on the year. Right now it assumes not. Or, rethink/simplify???
    //
    //     //to start with, the special forms (be sure to trim the input, otherwise there's no way in the textui to convert from a previously entered (so default)
    //     //value to "blank/all time"!).
    //     //%% let dateWithOptionalEra: String = {
    //     let date: String = {
    //         //%%need to ck how best test string equality? is by val or ref? need to change recent == on that below if is not by == ?
    //         if date_in.eq_ignore_ascii_case("now") || (blank_means_now && date_in.trim().len() == 0) {
    //             Utc.now().format(Util::DATEFORMAT).to_string()
    //         } else {
    //             date_in.trim().to_string()
    //         }
    //     };
    //     // %%just del?:
    //     // chop off the era before doing some of the other logic
    //     // let (era: String, date) =;
    //     //   if dateWithOptionalEra.toUpperCase.startsWith("AD") || dateWithOptionalEra.toUpperCase.startsWith("BC") {
    //     //     (dateWithOptionalEra.substring(0, 2), dateWithOptionalEra.substring(2))
    //     //   } else ("", dateWithOptionalEra)
    //
    //     // help user if they put in something like 2013-1-1 instead of 2013-01-01, so the parsed date isn't messed up. See test.
    //     // (The year could be other than 4 digits, so check for the actual location of the 1st hyphen):
    //       let hyphen_index = date.find('-');
    //       let firstHyphenPosition = match hyphen_index {
    //           Some(i) if i >= 0 => hyphen_index,
    //           _ => date.len(),
    //       };
    //     //but only if the string format looks somewhat expected; otherwise let later parsing handle it all.
    //     let filledInDateStr =
    //       if date.len() > firstHyphenPosition + 1 && date.len() < firstHyphenPosition + 6
    //           && date.find('-') == firstHyphenPosition && date.indexOf('-', firstHyphenPosition + 1) >= 0 {
    //         let secondHyphenPosition = date.indexOf('-', firstHyphenPosition + 1);
    //         if secondHyphenPosition == firstHyphenPosition + 2 || secondHyphenPosition == firstHyphenPosition + 3 {
    //           if date.length == secondHyphenPosition + 2 || date.length == secondHyphenPosition + 3 {
    //             let year = date.substring(0, firstHyphenPosition);
    //             let mo = date.substring(firstHyphenPosition + 1, secondHyphenPosition);
    //             let dy = date.substring(secondHyphenPosition + 1);
    //             year + '-' + (if mo.length == 1) "0" + mo else mo) + '-' + (if dy.length == 1) "0" + dy else dy)
    //           }
    //           else { date }
    //         }
    //         else { date }
    //       } else if date.length == firstHyphenPosition + 2 {
    //         // also handle format like 2013-1
    //         let year = date.substring(0, firstHyphenPosition);
    //         let mo = date.substring(firstHyphenPosition + 1);
    //         year + '-' + "0" + mo
    //       }
    //       else { date }
    //
    //
    //     // Fill in the date w/ "blank" information for whatever detail the user didn't provide:
    //     let filledInDateStrWithoutYear = if firstHyphenPosition < filledInDateStr.length { filledInDateStr.substring(firstHyphenPosition + 1) } else { "" };
    //     let year = filledInDateStr.substring(0, firstHyphenPosition);
    //
    //     let blankDateWithoutYear = blankDate.substring(5);
    //
    //     let dateWithZeros =
    //       if filledInDateStrWithoutYear.length() < blankDateWithoutYear.length {
    //         year + '-' + filledInDateStrWithoutYear + blankDateWithoutYear.substring(filledInDateStrWithoutYear.length())
    //       }
    //       else { filledInDateStr }
    //     // then parse it:
    //     try {
    //       let d: java.util.Date =;
    //         try {
    //           if era.isEmpty { Util::DATEFORMAT.parse(dateWithZeros) }
    //           else  { Util::DATEFORMAT_WITH_ERA.parse(era + dateWithZeros) }
    //         } catch {
    //           case e: java.text.ParseException =>
    //             try {
    //               if era.isEmpty { Util::DATEFORMAT2.parse(dateWithZeros) }
    //               else { Util::DATEFORMAT2_WITH_ERA.parse(era + dateWithZeros) }
    //             } catch {
    //               case e: java.text.ParseException =>
    //                 if era.isEmpty { Util::DATEFORMAT3.parse(dateWithZeros) }
    //                 else { Util::DATEFORMAT3_WITH_ERA.parse(era + dateWithZeros) }
    //             }
    //         }
    //       (Some(d.getTime), false)
    //     } catch {
    //       case e: java.text.ParseException =>
    //         ui.display_text("Invalid date format. Try something like \"2003\", or \"2003-01-31\", or \"2003-01-31 22:15\" for 10:15pm, or if you need a timezone, " +
    //                        "all of \"yyyy-MM-dd HH:mm:ss:SSS zzz\", like for just before midnight: \"2013-01-31 //23:59:59:999 MST\".")
    //         (None, true)
    //     }
    //   }

    /// Returns (valid_on_date, observation_date, userWantsToCancel)
    /// The editing_in parameter (I think) being true means we are editing data, not adding new data.
    pub fn ask_for_attribute_valid_and_observed_dates(
        old_valid_on_date_in: Option<i64>,
        old_observed_date_in: i64,
        ui: &TextUI,
        editing_in: bool,
    ) -> (Option<i64>, i64, bool) {
        loop {
            //%% was: fn askForBothDates(ui: TextUI) -> (Option<i64>, i64, bool) {
            let (valid_on_date, user_cancelled) = Self::ask_for_date(
                DateType::VALID,
                Self::valid_on_date_criteria,
                old_valid_on_date_in,
                &ui,
                editing_in,
            );
            if user_cancelled {
                break (None, 0, user_cancelled);
            } else {
                let (observed_date, user_cancelled) = Self::ask_for_date(
                    DateType::OBSERVED,
                    Self::observed_date_criteria,
                    Some(old_observed_date_in),
                    &ui,
                    editing_in,
                );
                if user_cancelled {
                    break (Some(0), 0, user_cancelled);
                } else {
                    // (for why valid_on_date is sometimes allowed to be None, but not observed_date: see let validOnPrompt.);
                    match observed_date {
                        None => {
                            // There is probably a smoother Rust way for this; this is what the scala code did.
                            assert!(observed_date.is_some());
                        }
                        Some(od) => {
                            let dates_descr: String =
                                Util::get_dates_description(valid_on_date, od);
                            let prompt = format!("Dates are: {}: right?", dates_descr).as_str();
                            let answer = ui.ask_yes_no_question(prompt, "y", false);
                            match answer {
                                Some(ans) if ans => {
                                    break (valid_on_date, od, user_cancelled);
                                }
                                _ => continue,
                            }
                        }
                    }
                }
            }
        }
        // }%%
    }

    // (This was probably originally in AttributeWithValidAndObservedDates.scala, moved here so can be called w/o rustc complaining.)
    pub fn get_dates_description(valid_on_date: Option<i64>, observation_date: i64) -> String {
        let valid_date_descr: String = {
            match valid_on_date {
                None => "unsp'd".to_string(),
                Some(date) if date == 0 => "all time".to_string(),
                Some(date) => Util::useful_date_format(date),
            }
        };
        let observed_date_descr: String = Util::useful_date_format(observation_date);
        format!("valid {}, obsv'd {}", valid_date_descr, observed_date_descr)
    }

    // This came from Attribute.rs, to to be able to call it w/o  errors from rustc.
    /// d is in milliseconds since 1970. For more on dates, see postgresql_database.rs
    /// under "create table RelationToEntity" for comments about dates' meanings.
    pub fn useful_date_format(d: i64) -> String {
        // No need to print "AD" unless we're really close?, as in this example:
        //scala > let DATEFORMAT_WITH_ERA = new java.text.SimpleDateFormat("GGyyyy-MM-dd HH:mm:ss:SSS zzz");
        //scala > DATEFORMAT_WITH_ERA.parse("ad 1-03-01 00:00:00:000 GMT").getTime //i.e., Jan 3, 1 AD.
        //res100: i64 = -62130672000000
        // see Util::DATEFORMAT* for comment about ERA (BC/AD).
        // if d > -62130672000000_i64 {
        //     DATEFORMAT.format(d)
        // %%need to test this date thing also to confirm works as expected/same as scala OM.
        //See also uses of this, in case need to borrow one or update both, in util.rs .
        let seconds: i64 = d / 1000;
        let milliseconds: i64 = match d.checked_rem(1000) {
            None => return format!("Unable to calculate ms for value d={}", d),
            Some(s) => s,
        };
        let utc_date: LocalResult<DateTime<Utc>> =
            Utc.timestamp_opt(seconds, (milliseconds * 1_000_000) as u32);
        match utc_date {
            LocalResult::None => {
                format!(
                    "Error(1) trying to format {:?} as a date/time; probably a bug.",
                    utc_date
                )
            }
            LocalResult::Single(dt) => {
                let typed_dt: DateTime<Utc> = dt;
                let naive_utc_dt: NaiveDateTime = typed_dt.naive_utc();
                let local_now: DateTime<Local> = Local::now();
                let local_time_zone: Local = local_now.timezone();
                //debug!("local_time_zone is: {:?}", local_time_zone);
                let local_dt: DateTime<Local> = local_time_zone.from_utc_datetime(&naive_utc_dt);
                //alternative? In ISO-8601 format:
                //debug!("local_dt.to_rfc3339() is: {:?}",local_dt.to_rfc3339());
                local_dt.format(Util::DATEFORMAT).to_string()
            }
            _ => format!(
                "Error(2) trying to format {:?} as a date/time; probably a bug.",
                utc_date
            ),
        }

        //%%see above comment at ..."from Attribute.scala".
        // } else {
        //     DATEFORMAT_WITH_ERA.format(d)
        // }
        // I.e., The full code in Attribute.scala for this was this: need to mimic any further?:
        // def usefulDateFormat(d: Long): String = {
        //     // No need to print "AD" unless we're really close?, as in this example:
        //     //scala > val DATEFORMAT_WITH_ERA = new java.text.SimpleDateFormat("GGyyyy-MM-dd HH:mm:ss:SSS zzz")
        //     //scala > DATEFORMAT_WITH_ERA.parse("ad 1-03-01 00:00:00:000 GMT").getTime //i.e., Jan 3, 1 AD.
        //     //res100: Long = -62130672000000
        //     if (d > -62130672000000L) DATEFORMAT.format(d)
        //     else DATEFORMAT_WITH_ERA.format(d)
        // }
    }

    /// @param input The value to chop down in size.
    /// @param length_limit_in If <= 0, no change.
    /// @return A value equal or shorter in length.
    // This came from Attribute.rs, to be able to call it w/o errors from rustc.
    pub fn limit_attribute_description_length(input: &str, length_limit_in: usize) -> String {
        //%%does this method have any purpose now that should not be handled instead inside the
        // fn substring* below?
        if length_limit_in != 0 && input.len() > length_limit_in {
            let substr = Util::substring_from_start(input, length_limit_in - 3);
            format!("{}...", substr)
        } else {
            input.to_string()
        }
    }

    pub fn substring_from_start(s: &str, length: usize) -> String {
        // This is a convenience, and maybe good enough for roughly displaying text when only
        // the last letter(s) could be miscalculated anyway (right?).

        // In scala this (from fn limit_attribute_description_length) was like:
        //    input.substring(0, length_limit_in - 3) + "..."

        // Instead, should I be using something like the unicode-segmentation crate, then
        // UnicodeSegmentation::graphemes() ?  Or, could search for substring on crates.io.

        // This is per https://users.rust-lang.org/t/how-to-get-a-substring-of-a-string/1351 .
        // (The next 2 lines are another suggestion from that same above URL:)
        //let mut end : usize = 0;
        //s.chars().into_iter().take(6).for_each(|x| end += x.len_utf8());

        //%%what if length is too long? test4that or just look up what take does...?
        let end = s.chars().map(|c| c.len_utf8()).take(length).sum();
        format!("{}", &s[..end])
    }

    //idea: make this more generic, passing in prompt strings &c, so it's more cleanly useful for DateAttribute instances. Or not: lacks shared code.
    //idea: separate these into 2 methods, 1 for each time (not much common material of significance).
    // BETTER IDEA: fix the date stuff in the DB first as noted in tasks (and comments below?), so that this part makes more sense (the 0 for all time, etc), and then
    // when at it, recombine the ask_for_date_generic method w/ these or so it's all cleaned up.
    /// Returns the date (w/ meanings as with display_text below, and as in PostgreSQLDatabase.create_tables),
    /// and true if the user wants to cancel/get out).
    /// The editing_in parameter (I think) being true means we are editing data, not adding new data.
    fn ask_for_date(
        date_type_in: DateType,
        acceptance_criteria_in: fn(&str) -> bool,
        old_date_in: Option<i64>,
        ui: &TextUI,
        editing_in: bool,
    ) -> (Option<i64>, bool) {
        let leading_text: String = match date_type_in {
            DateType::VALID => {
                format!("\nPlease enter the date when this was first VALID (i.e., true) (Press Enter (blank) for unknown/unspecified, or {}{}{}{}{}",
                    //%%put back when allowing more formats:
                    // "like this, w/ at least the year: \"2002\", \"2000-1-31\", or " +
                    // "like \"2013-01-31 23:59:59:999 MST\"; zeros are " +
                    "like \"2013-01-31 23:59\" or maybe w/o the time part; zeros are ",
                    "allowed in all but the yyyy-mm-dd.  Or for current date/time ",
                    "enter \"now\".  ESC to exit this.  ",
                    "For dates in the BC era you can prefix them with a minus sign: -3400 for example, but either way omit a space ",
                    "before the year), like -3400-01-31 23:59:59:999 GMT, entered at least up through the year, up to ~262000 years AD or BC.")
                //IDEA: I had thought to say:  "Or for "all time", enter just 0.  ", BUT (while this is probably
                //solved, it's not until the later part of this comment):
                //    "There is ambiguity about BC that needs some " +
                //    "investigation, because java allows a '0' year (which for now means 'for all \
                //    time' in just this program), but normal human time doesn't " +
                //    "allow a '0' year, so maybe you have to subtract a year from all BC things \
                //    for them to work right, and enter/read them accordingly, until " +
                //    "someone learns for sure, and we decide whether to subtract a year from everything \
                //    BC for you automatically. Hm. *OR* maybe dates in year " +
                //    "zero just don't mean anything so can be ignored by users, and all other dates' \
                //    entry are just fine, so there's nothing to do but use it as is? " +
                //    "But that would have to be kept in mind if doing any relative date calculations in \
                //    the program, e.g. # of years, spanning 0.)\n" +
                //    "Also, real events with more " +
                //    "specific time-tracking needs will probably need to model their own time-related \
                //    entity classes, and establish relations to them, within their use of OM.")
                //ABOUT THAT LAST COMMENT: WHY DOES JAVA ALLOW A 0 YEAR, UNLESS ONLY BECAUSE IT USES long #'S? SEE E.G.
                // http://www.msevans.com/calendar/daysbetweendatesapplet.php
                // which says: "...[java?] uses a year 0, which is really 1 B.C. For B.C. dates, you have
                // to remember that the years are off by one--10 B.C.
                // to [java?] is really 11 B.C.", but this really needs more investigation on what is
                // the Right Thing to do.
                // Or, just let the dates go in & out of the data, interpreted just as they are now, but
                // the savvy users will recognize that dates in year zero just
                // don't mean anything, thus the long values in that range don't mean anything so can
                // be disregarded (is that how it really works in java??), (or if
                // so we could inform users when such a date is present, that it's bogus and to use 1 instead)?
                // **SO:** it is already in the task list to have a separate flag in the database for "all time".
                // AND: how does Rust (chrono crate?) address all that?  Are we storing dates in UTC or
                // what? sch code for GMT, MST, UTC.
                // and how does om know to show mst vs. mdt in the output of entities created in summer vs. winter?
            }
            DateType::OBSERVED => {
                // vec![format!("\nWHEN OBSERVED?: {} (\"unknown\" not allowed here.) ", Self::GENERIC_DATE_PROMPT).as_str()]
                format!(
                    "\nWHEN OBSERVED?: {} (\"unknown\" not allowed here.) ",
                    Self::GENERIC_DATE_PROMPT
                )
            }
        };

        let default_value: String = match date_type_in {
            DateType::VALID => {
                if editing_in {
                    match old_date_in {
                        Some(0) => "0".to_string(),
                        Some(old_date) => {
                            //%% was: Some(Util::DATEFORMAT_WITH_ERA.format(new Date(old_date)))
                            //%%NEED TO TEST all THIS DATE STUFF EXPLICITLY, and the file _?_ and attribute.rs .
                            //See also uses of this, in case need to borrow one or update both, below & in attribute.rs .
                            let ndt_option = NaiveDateTime::from_timestamp_opt(old_date, 0);
                            match ndt_option {
                                None => format!("Unable to build a date string from {} seconds--why? Probably a bug to report and fix.", old_date),
                                Some(ndt) => {
                                    let default = DateTime::<Utc>::from_utc(ndt, Utc).to_string();
                                    default
                                }
                            }
                        }
                        _ => "".to_string(),
                    }
                } else {
                    "".to_string()
                }
            }
            DateType::OBSERVED => {
                match old_date_in {
                    Some(old_date) if editing_in => {
                        // was: Some(Util::DATEFORMAT_WITH_ERA.format(new Date(old_observed_date_in)))
                        //%%NEED TO TEST all THIS DATE STUFF EXPLICITLY, and the file _?_ and attribute.rs .
                        //See also similar/dift use, in case need to borrow one or update both, above & in attribute.rs .
                        let ndt_option = NaiveDateTime::from_timestamp_opt(old_date, 0);
                        match ndt_option {
                            None => format!("Unable to build a date string from {} seconds--why? Probably a bug to report and fix(2).", old_date),
                            Some(ndt) => {
                                let default = DateTime::<Utc>::from_utc(ndt, Utc).to_string();
                                default
                            }
                        }
                    }
                    _ => {
                        // was: Some(Util::DATEFORMAT_WITH_ERA.format(new Date(System.currentTimeMillis())))
                        let default = Utc::now().to_string();
                        default
                    }
                }
            }
        };

        // let default: Option<&str> = match default_value {
        //     None => None,
        //     Some(s) => Some(s.as_str())
        // };
        loop {
            let ans = ui.ask_for_string3(vec![leading_text.as_str()], None, default_value.as_str());
            match ans {
                None => {
                    match date_type_in {
                        DateType::VALID => {
                            // don't let user cancel from the "valid on" date: blank there means "unknown"
                            // (but user can ESC again from observed date. Making these
                            // consistent probably meant figuring out how to modify jline2 (again, now)
                            // so that it will distinguish between user pressing ESC and user
                            // pressing Enter with a blank line: now IIRC it just returns a blank line for both.
                            // Or something, now that using a readline library in Rust.
                            break (None, false);
                        }
                        DateType::OBSERVED => {
                            //getting out, but observed date not allowed to be None (see caller for details)
                            break (Some(0), true);
                        }
                    }
                }
                Some(answer) => {
                    let date: &str = answer.as_str().trim();
                    if date_type_in == DateType::VALID && date.len() == 0 {
                        break (None, false);
                    } else if date_type_in == DateType::VALID && date == "0" {
                        break (Some(0), false);
                    } else if !acceptance_criteria_in(date) {
                        continue;
                    } else {
                        // (special values like "0" or blank are already handled above)
                        //%% let (new_date, retry): (Option<i64>, bool) = finish_and_parse_the_date(date, date_type_in == DateType::OBSERVED, ui);
                        let new_date: ParseResult<DateTime<FixedOffset>> =
                            DateTime::parse_from_str(date, Util::DATEFORMAT4);
                        match new_date {
                            Ok(dt) => break (Some(dt.timestamp()), false),
                            Err(e) => {
                                let text: String = format!("Unable to parse provided date, with error: \"{}\"  Please retry.", e.to_string());
                                ui.display_text2(text.as_str(), true);
                                continue;
                            }
                        }
                    }
                }
            }
        }
    }

    /// Cloned from Controller.ask_for_date; see its comments in the code.
    /// Idea: consider combining somehow with method ask_for_date_attribute_value.
    /// Return milliseconds UTC (unix time in millis), None if user wants out.
    pub fn ask_for_date_generic(
        prompt_text_in: Option<&str>, /*%% = None*/
        default_in: Option<&str>,
        ui: &TextUI,
    //) -> Option<DateTime<FixedOffset>> {
    ) -> Option<i64> {
        loop {
            let leading_text: Vec<&str> = vec![prompt_text_in.unwrap_or(Self::GENERIC_DATE_PROMPT)];
            let df: String = Utc::now().format(Util::DATEFORMAT4).to_string();
            let default: &str = default_in.unwrap_or(df.as_str());
            let ans = ui.ask_for_string3(leading_text, None, default);
            match ans {
                None => return None,
                Some(answer) => {
                    let date = answer.trim();
                    // let (new_date: Option<i64>, retry: bool) = finish_and_parse_the_date(date, true, ui);
                    let new_date: ParseResult<DateTime<FixedOffset>> =
                        DateTime::parse_from_str(date, Util::DATEFORMAT4);
                    match new_date {
                        Ok(dt) => return Some(dt.timestamp_millis()), //%%type? & at similar plc/es after finish_and_parse_the_date ?
                        Err(e) => {
                            let text: String = format!(
                                "Unable to parse provided date, with error: \"{}\"  Please retry.",
                                e.to_string()
                            );
                            ui.display_text2(text.as_str(), true);
                            //%% but how does the get out w/o entry, from this & similar places after finish_and_parse_the_date or any other loop?
                            //%% test/try it?
                            continue;
                        }
                    }
                }
            }
        }
    }

    /// This gets the an abbreviated part of the copyright text to be used by the UI.  It is
    /// customized to the actual content of the LICENSE file, to extract & modify suitably for UI display.
    pub fn license() -> String {
        let mut text_to_show = String::new();
        // Note: Before the next line was added, the binary
        // debug size was 4,884,336 bytes.
        // After the next line was added (with some other changes in the last commit), the binary
        // debug size was 5,066,280 (difference of 181,944), with a LICENSE file size of 38,816 bytes.
        // But maybe the release version would have a smaller size difference from adding this.
        // Idea: could make the binary smaller by including only the part of the LICENSE file that
        // is used by this fn.  Maybe have similar logic against a temp file and just include that.
        // (See related notes in Cargo.toml.)
        let text = include_str!("../../LICENSE");
        let mut append = false;
        let mut before_any_dashes = true;
        let mut lines = text.lines();
        let mut line_opt = lines.next();
        while line_opt.is_some() {
            let line = line_opt.unwrap_or("ERROR: How did we get a None from a line in the LICENSE file, after checking line_opt.is_some()?");
            if !append && line.starts_with("-----") && before_any_dashes {
                append = true;
                before_any_dashes = false;
            } else if append && line.contains("(see below). If not, see") {
                text_to_show = text_to_show
                    + &line.replace(
                        "(see below). If not, see",
                        "(see the file LICENSE). If not, see",
                    )
                    + "\n";
                // append = false;  // commented out because never read; comment here for reading clarity.
                // Stop doing the extra checks in vain now, since no more appending done. It cut the
                // the ~0.5 second startup to ~0.25 sec. Oh well, it was fun to test the difference.
                // Just moving to rust from scala, made the startup go from ~2-4 sec to ~0.5 sec.
                break;
            } else if append {
                text_to_show = text_to_show + line + "\n";
            } else if !append {
                // do nothing.
            }
            line_opt = lines.next();
        }
        text_to_show
        /*idea: do this again somehow, or drop the idea?  There is the issue of providing the AGPL (etc/mine?)
          with the app, and maybe it could also address that:
                    case e: Exception =>
                      let ans = ui.ask_yes_no_question("\n\nThe file LICENSE is missing from the distribution \
                      of this program or for " +;
                        "some other reason can't be displayed normally; } please let us know to " +
                        " correct that, and please be aware of the license.  You can go to this URL to see it:\n" +
                        "    http://onemodel.org/download/OM-LICENSE \n" +
                        ".  (Do you want to see the detailed error output?)")
        */
    }

    pub fn string_too_long_error_message(name_length: u16, details_in: &str) -> String /*= ""*/ {
        let details = if details_in.is_empty() {
            ""
        } else {
            format!(" Details: {}", details_in).as_str()
        };
        // for details, see method PostgreSQLDatabase.escape_quotes_etc. (??)
        format!(
            "Got an error.  Please try a shorter ({}) chars) entry.  \
          (Could be due to escaped, i.e. expanded, characters like ' or \";\".{}",
            name_length, details
        )
    }

    pub fn is_duplication_a_problem(
        is_duplicate_in: bool,
        duplicate_name_probably_ok: bool,
        ui: &TextUI,
    ) -> bool {
        let mut duplicate_problem_so_skip = false;
        if is_duplicate_in {
            if !duplicate_name_probably_ok {
                let answer_opt = ui.ask_for_string3(
                    vec!["That name is a duplicate--proceed anyway? (y/n)"],
                    None,
                    "n",
                );
                match answer_opt {
                    None => duplicate_problem_so_skip = true,
                    Some(ans) => {
                        if !ans.eq_ignore_ascii_case("y") {
                            duplicate_problem_so_skip = true
                        }
                    }
                }
            }
        }
        duplicate_problem_so_skip
    }

    //%%move to some *relation* struct like RelationType? same w/ others below? Or keep together for maintenance?
    //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) within this method, below!
    pub fn ask_for_name_in_reverse_direction(
        _directionality_in: String,
        _name_length_in: u16,
        _name_in: String,
        _previous_name_in_reverse_in: Option<&str>, /*%%= None*/
        _ui: &TextUI,
    ) -> String {
        // see create_tables (or UI prompts) for meanings of bi/uni/non...
        // %%?:
        //   match directionality_in {
        //       RelationType::RelationDirectionality::UNI => "".to_string(),
        //       RelationType::RelationDirectionality::NON => name_in,
        //       RelationType::RelationDirectionality::BI => {
        //           loop {
        //               // see create_tables (or UI prompts) for meanings...
        //               let msg = vec![format!("Enter relation name when direction is reversed (i.e., 'is husband to' becomes 'is wife to', 'employs' becomes 'is employed by' by; up to {name_length_in} characters (ESC to cancel): ").as_str()];
        //               let name_in_reverse: String = {
        //                   let answer: Option<String> = ui.ask_for_string3(msg, None, previous_name_in_reverse_in);
        //                   match answer {
        //                       None => "".to_string(),
        //                       Some(ans) => ans.get.trim() //see above comment about trim (??)
        //                   }
        //               };
        //               let answer = ui.ask_which(Some(vec!["Is this the correct name for the relationship in reverse direction?: "]), vec!["Yes", "No"]);
        //               match answer {
        //                   None => continue,
        //                   Some(ans) if ans == 2 => continue,
        //                   _ => name_in_reverse
        //               }
        //           }
        //       }
        //   }
        //%%temp, to compile:
        "".to_string()
    }

    pub fn ask_for_relation_directionality(
        previous_directionality_in: &str, /*%%= "" (was: None)*/
        ui: &TextUI,
    ) -> Option<String> {
        let msg = vec!["Enter directionality (\"bi\", \"uni\", or \"non\"; examples: \"is parent of\"/\"is child of\" is bidirectional, \
                          since it differs substantially by the direction but goes both ways; unidirectional might be like 'lists': the thing listed doesn't know \
                          it; \"is acquaintanted with\" could be nondirectional if it is an identical relationship either way  (ESC to cancel): "];

        //%%does this work? Make mental note 4 future, if so.
        fn criteria_for_ask_for_relation_directionality(entry_in: &str) -> Result<(), anyhow::Error> {
            let entry = entry_in.trim().to_uppercase();
            if entry == "BI" || entry == "UNI" || entry == "NON" {
                Ok(())
            } else {
                Err(anyhow!("Unexpected directionality: {} (should be \"BI\", \"UNI\", or \"NON\").", entry_in))
            }
        }

        let criteria_fn: fn(&str) -> Result<(), anyhow::Error> = criteria_for_ask_for_relation_directionality;
        let directionality = ui.ask_for_string3(
            msg,
            Some(criteria_fn),
            previous_directionality_in,
        );
        match directionality {
            None => None,
            Some(d) => Some(d.to_uppercase()),
        }
    }

    // pub fn edit_multiline_text(input: &String, ui: &TextUI) -> Result<String, String> {
    pub fn edit_multiline_text(input: &String, ui: &TextUI) -> Result<String, anyhow::Error> {
        //idea: allow user to change the edit command setting (ie which editor to use) from here?
        //idea: allow user to prevent this message in future. Could be by using ui.ask_yes_no_question
        //instead, adding to the  prompt "(ask this again?)", with
        //'y' as default, and storing the answer in the db.SYSTEM_ENTITY_NAME somewhere perhaps.
        // %%?:
        //PUT THIS BACK (& review/test it) after taking the time to read the (Rust equivalent of the)
        //Process package's classes or something like
        // apache commons has, and learn to launch vi workably, from scala. And will the terminal
        // settings changes by OM have to be undone/redone for it?:
        //        let command: String = db.get_text_editor_command;
        //        ui.display_text("Using " + command + " as the text editor, but you can change that by
        //        navigating to the Main OM menu with ESC, search for
        // existing " +
        //                       "entities, choose the first one (called " +
        //                       PostgreSQLDatabase.SYSTEM_ENTITY_NAME + "), choose " +
        //                       PostgreSQLDatabase.EDITOR_INFO_ENTITY_NAME + ", choose " +
        //                       "" + PostgreSQLDatabase.TEXT_EDITOR_INFO_ENTITY_NAME + ", then choose the " +
        //                       PostgreSQLDatabase.TEXT_EDITOR_COMMAND_ATTRIBUTE_TYPE_NAME + " and edit it with option 3.")

        //%%test (all paths?) & clean up:
        //   let path: Path = Files.createTempFile("om-edit-", ".txt");
        let mut rng = randlib::Rand::new();
        let rand_num: u64 = rng.rand_u64();
        let filename = format!("om-edit-{}.txt", rand_num.to_string());
        let msg_possibly = format!(
            "Unable to convert OS temp path, and filename {} to a UTF8 string.",
            filename
        );
        let path_buf: std::path::PathBuf = std::env::temp_dir().with_file_name(filename);
        // Files.write(path, input.getBytes)
        let (write_result, full_path) = {
            // let psr = path.as_str();
            let psr = path_buf.to_str();
            match psr {
                // Err(e) => {
                None => {
                    // let msg = format!("Unable to convert temp path and filename {} to a UTF8 string: {}", filename, e.to_string());
                    return Err(anyhow!(msg_possibly));
                }
                // Ok(ps) => {
                Some(ps) => (std::fs::write(ps, input), ps),
            }
        };
        //                   std::fs::write(path.to_str(), input)?;
        match write_result {
            Err(e) => {
                //%%?:
                // ui.display_text1(format!("Unable to write temporary file for editing: {}", e.to_string()).as_str());
                // e
                // Instead of using "?" to return the error, creating a new one so I know what type to return from the function.
                // And similarly just below.
                let msg = format!(
                    "Unable to write temporary file for editing: {}",
                    e.to_string()
                );
                Err(anyhow!(msg))
            }
            Ok(_) => {
                ui.display_text1(format!("Until we improve this, you can now go edit the content in this temporary file, & save it:\n{}\n...then come back here when ready to import that text.",
                                             full_path).as_str());
                // let new_content: String = new Predef.String(Files.readAllBytes(path));
                let result2 = std::fs::read(&path_buf);
                // let new_content: Vec<u8> = std::fs::read(path)?;
                match result2 {
                    Err(e) => {
                        // ui.display_text1(format!("Unable to read temporary file from editing session: {}", e.to_string()).as_str());
                        let msg = format!("Unable to read temporary file from editing session: {} .  File not deleted and edits not saved to OneModel.", e.to_string());
                        Err(anyhow!(msg))
                    }
                    Ok(new_content) => {
                        match String::from_utf8(new_content) {
                            Err(e) => {
                                //%%test. Is this a problem when edited with LO or other/whatever tools?  Find
                                // another way so can just slurp in whatever, if not UTF8???!!
                                // ui.display_text1(format!("Unable to convert the content to a UTF8 string.  Will leave the temporary file in place for you to view, but edits have not been saved back to OneModel:  {}", e.to_string()).as_str());
                                let msg = format!("Unable to convert the content to a UTF8 string.  Will leave the temporary file in place for you to view, but edits have not been saved back to OneModel:  {}", e.to_string());
                                Err(anyhow!(msg))
                            }
                            Ok(new_content_checked) => {
                                //%%ask whether to delete the temp copy? leave as an idea? Next 2 ops seem out of order, no? But what best instead?
                                //(Maybe just deleting poses no greater risk than removing contents of a file and saving it? But
                                // transactionality/safety-- should save the new content to DB
                                // (in caller?) before deleting the temp file?? Like, what if we
                                // delete the file and then due to some hw or sw error, can't save it?)
                                let result3 = std::fs::remove_file(&path_buf);
                                match result3 {
                                    Err(e) => {
                                        ui.display_text1(format!("Unable to delete the temporary file with the edited info; \
                                            no harm done except clutter until the OS cleans it up, but probably want to check on why: \
                                            {}", e.to_string()).as_str());
                                    }
                                    _ => {}
                                }
                                Ok(new_content_checked)
                            }
                        }
                    }
                }
            }
        }
    }

    /// Returns None if user wants to cancel.
    //%%move to TextUI?
    pub fn prompt_whether_to_1add_or_2correct(attr_type_desc_in: &str, ui: &TextUI) -> Result<Option<i32>, anyhow::Error> {
        loop {
            let answer = ui.ask_which(
                None,
                &vec![
                    format!("1-Save this {} attribute?", attr_type_desc_in).as_str(),
                    "2-Correct it?",
                ],
                &vec![],
                true,
                None,
                None,
                None,
                None,
            );
            match answer {
                None => {
                    return Ok(None);
                },
                Some(ans) => {
                    if ans < 1 || ans > 2 {
                        ui.display_text1("invalid response");
                        continue;
                    } else {
                        return Ok(Some(i32::try_from(answer.unwrap())?));
                    }
                }
            }
        }
    }

    //%%move to quantityattr struct or ui? same w/ others below? Or keep together for maintenance?
    /// Returns None if user wants to cancel.
    pub fn ask_for_quantity_attribute_number(previous_quantity: f64, ui: &TextUI) -> Option<f64> {
        loop {
            let leading_text =
                vec!["ENTER THE NUMBER FOR THE QUANTITY (i.e., 5, for 5 centimeters length)"];
            let answer = ui.ask_for_string3(
                leading_text,
                Some(Util::is_numeric),
                format!("{}", previous_quantity).as_str(),
            );
            match answer {
                None => return None,
                Some(ans) => {
                    let result = f64::from_str(ans.as_str());
                    match result {
                        Err(e) => {
                            ui.display_text2(format!("Not a valid number. Please retry. (Developer: is_numeric should have already checked this. How did we get here?.  Error message: \"{}\")", e.to_string()).as_str(), true);
                            continue;
                        }
                        Ok(f) => {
                            return Some(f);
                        }
                    }
                }
            }
        }
    }

    //%%shouldn't this be in the postgresql_database.rs file, and make its struct fields not pub? or cmt why it isn't?
    pub fn initialize_test_db() -> Result<PostgreSQLDatabase, &'static str> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        // It seems like it would be faster to put the next two statements inside the ".call_once()"
        // below, but then returning the db or assigning it to a static mut TEST_DB for others to
        // access was initially problematic and I didn't see an obvious solution.
        let pool =
            PostgreSQLDatabase::connect(&rt, Util::TEST_USER, Util::TEST_USER, Util::TEST_PASS)
                .unwrap();
        let db: PostgreSQLDatabase = PostgreSQLDatabase {
            rt,
            pool,
            include_archived_entities: false,
        };
        //Idea: another way to do this might be the crate once_cell. (Or does std::sync::Once
        //superceded that?)
        TEST_DB_INIT.call_once(|| {
            // for some explanation, see:
            //   https://doc.rust-lang.org/std/sync/struct.Once.html
            //   https://stackoverflow.com/questions/58006033/how-to-run-setup-code-before-any-tests-run-in-rust/58006287#58006287

            //%%log instead?
            debug!("starting db call_once");
            //mbe not needed?: just return the db?
            // for why this is safe, see explanation & examples in above link to doc.rust-lang.org .
            // unsafe {
            //     //%% NOTE!!!: If I uncomment & use this, sch for "static mutable singleton rust"
            //     to find a safe pattern for that.
            //     TEST_DB = Some(db);
            // }

            // no point in a transaction to destroy tables, it seems.
            db.destroy_tables().unwrap();
            //%%?:
            // let mut tx = db
            //     .begin_trans()
            //     .expect("Failure to begin transaction before creating test data.");
            // db.create_tables(&Some(&mut tx)).unwrap();
            db.setup_db().unwrap();
            // db.commit_trans(tx)
            //     .expect("Failure to commit transaction after creating test data.");

            debug!("finishing db call_once");
        });
        Ok(db)
    }

    pub fn initialize_tracing() {
        TEST_TRACING_INIT.call_once(|| {
            let subscriber = tracing_subscriber::FmtSubscriber::builder()
                .with_max_level(tracing::Level::TRACE).finish();
            tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
            debug!("testing tracing at debug level (they go by verbosity--high level = higher verbosity).");
        });
    }

    /// Used for example after one has been deleted, to put the highlight on right next one:
    /// idea: This feels overcomplicated.  Make it better?  Fixing bad smells in general (large classes etc etc) is on the task list.
    /**%%fix doc formatting:
     * @param object_set_size  # of all the possible entries, not reduced by what fits in the available display space (I think).
     * @param objects_to_display_in  Only those that have been chosen to display (ie, smaller list to fit in display size size) (I think).
     * @return
     */
    //%%do any callers of this have a transaction? If so, does it make sense to pass that into here so
    //it can pass it into the below call to "let new_same_entity = match Entity::new2(...)"?
    fn find_entity_to_highlight_next(
        db: Rc<RefCell<dyn Database>>,
        object_set_size: usize,
        objects_to_display_in: Vec<Entity>,
        removed_one_in: bool,
        previously_highlighted_index_in_obj_list_in: usize,
        previously_highlighted_entry_in: Entity,
    ) -> Result<Option<Entity>, String> {
        //NOTE: SIMILAR TO find_attribute_to_highlight_next: WHEN MAINTAINING ONE, DO SIMILARLY ON THE OTHER, until they are merged maybe by using the type
        //system better.

        // Here of course, previously_highlighted_index_in_obj_list_in and obj_ids.size were calculated prior to the deletion.

        if removed_one_in {
            if object_set_size <= 1 {
                return Ok(None);
            }
            let new_obj_list_size: usize = object_set_size - 1;
            if new_obj_list_size == 0 {
                //(redundant with above test/None, but for clarity in reading)
                Ok(None)
            } else {
                let mut new_index_to_highlight = std::cmp::min(
                    new_obj_list_size - 1,
                    previously_highlighted_index_in_obj_list_in,
                );
                //IF CODE WORKS OK w/ the below this comment block, it can be deleted. Try deleting an entry at the beginning,
                //one at the end, one in the middle, and none? adding? write tests for it or skip?
                // if new_index_to_highlight != previously_highlighted_index_in_obj_list_in {
                //     // %%why doesn't Rust know the element is an Entity, vs. <Unknown>? why can't just return
                //     // objects_to_display_in.get(new_index_to_highlight)? Maybe rustc would do OK but the
                //     // IDE doesn't? try changing at first 1 of the
                //     // 3 below places back, and see if rustc gets it right? or am I mistaken?
                //     match objects_to_display_in.get(new_index_to_highlight) {
                //         None => Ok(None),
                //         //does the next line actually work?? ie, unknown how clone would work w/ its db.
                //         //If not, remove derive clone fr entity?
                //         //might have to create a new instance of the entity, instead, with new2()?
                //         // Some(&e) => Some(e.to_owned()),
                //         Some(&e) => {
                //             // create a new instance of this entity, to avoid compiler errors
                //             let new_same_entity = match Entity::new2(self, e.get_id()) {
                //                 Err(e) => return Err(e.to_string()),
                //                 Ok(entity) => entity,
                //             };
                //             Ok(Some(new_same_entity))
                //         },
                //     }
                // } else {
                //     if new_index_to_highlight + 1 < new_obj_list_size - 1 {
                //         match objects_to_display_in.get(new_index_to_highlight + 1) {
                //             None => Ok(None),
                //             Some(&e) => Some(e),
                //         }
                //     } else if new_index_to_highlight >= 1 {
                //         match objects_to_display_in.get(new_index_to_highlight - 1) {
                //             None => None,
                //             Some(&e) => Some(e),
                //         }
                //     } else {
                //         None
                //     }
                // }
                //%%replace/del cmted part above w/ below?
                new_index_to_highlight =
                    if new_index_to_highlight != previously_highlighted_index_in_obj_list_in {
                        new_index_to_highlight
                    } else {
                        if new_index_to_highlight + 1 < new_obj_list_size - 1 {
                            new_index_to_highlight + 1
                            // None => Ok(None),
                            // Some(&e) => Some(e),
                        } else if new_index_to_highlight >= 1 {
                            new_index_to_highlight - 1
                        } else {
                            return Ok(None);
                        }
                    };
                match objects_to_display_in.get(new_index_to_highlight) {
                    None => Ok(None),
                    Some(e) => {
                        // create a new instance of this entity, to avoid compiler errors
                        let new_same_entity = match Entity::new2(db, None, e.get_id()) {
                            Err(e) => return Err(e.to_string()),
                            Ok(entity) => entity,
                        };
                        Ok(Some(new_same_entity))
                    }
                }
                // }
            }
        } else {
            Ok(Some(previously_highlighted_entry_in))
        }
    }

    // compare to fn print_backtrace.
    // %%test this somehow? is it used? Was thinking of using it in entity_class
    // get_display_string.
    pub fn get_backtrace() -> String {
        let bt = std::backtrace::Backtrace::capture();
        let mut s = "".to_string();
        s.push_str(format!("Backtrace.status()={:?}", bt.status()).as_str());
        s.push_str(format!("Backtrace.status()={}", bt).as_str());
        s
    }

    ///Print a backtrace (stack trace) if the RUST_BACKTRACE environment variable is nonzero and
    ///the platform supports it: (and if in debug mode, at least for now)
    pub fn print_backtrace() {
        let bt = std::backtrace::Backtrace::capture();
        debug!("Backtrace.status()={:?}", bt.status());
        debug!("Backtrace.status()={}", bt);
    }

    /// Returns None if user wants to cancel.
    pub fn ask_for_text_attribute_text(
        _: &Controller,
        _: Rc<RefCell<dyn Database>>,
        dh_in: &mut AttributeDataHolder, /*::TextAttributeDH*/
        // i.e., vs. adding a new one, right?:
        editing_in: bool,
        ui: &TextUI,
    ) -> Result<Option<AttributeDataHolder /*::TextAttributeDH*/>, anyhow::Error> {
        // This is overcomplicated for what it really does, just because the signature needs to 
        // match with other functions that can be passed in where this one is used.
        // let default_text_value: String;
        // let mut tadh_out: TextAttributeDH;
        /*%%let mut dh_out: TextAttributeDH =*/ match dh_in {
            AttributeDataHolder::TextAttributeDH { tadh } => {
                let mut tadh_out = TextAttributeDH {
                    attr_type_id: tadh.attr_type_id,
                    valid_on_date: tadh.valid_on_date,
                    observation_date: tadh.observation_date,
                    text: "".to_string(),
                };
                let default_text_value: &str = if editing_in {
                    tadh.text.as_str()
                } else {
                    ""
                };
                let leading_text = vec!["Type or paste a single-line attribute value, then press Enter; ESC to cancel.  \
                                     (If you need to add or edit multiple lines, just put in a single line or letter for \
                                     now (or leave the multiple lines if already in place), then you can edit it afterward \
                                     to add the full text.  But consider if a 'file' attribute or some other way of \
                                     modeling the info would be better at representing what it really *is*.  Legitimate \
                                     use cases for a text attribute might include a quote or a stack trace.)"];
                let answer = ui.ask_for_string3(leading_text, None, default_text_value);
                match answer {
                    None => Ok(None),
                    Some(ans) => {
                        tadh_out.text = ans;
                        Ok(Some(AttributeDataHolder::TextAttributeDH { tadh: tadh_out }))
                    }
                }
            },
            _ => {
                return Err(anyhow!("unexpected enum variant: {:?}", dh_in));
            }
        }
    }

    /// Returns None if user wants to cancel.
    // Idea: consider combining somehow with method ask_for_date_generic or note here why not, perhaps.
    pub fn ask_for_date_attribute_value(
        _: &Controller,
        _ignore: Rc<RefCell<dyn Database>>,
        dh_in: &mut AttributeDataHolder, /*%%::DateAttributeDH*/
        _editing_in: bool,
        ui: &TextUI,
    ) -> Result<Option<AttributeDataHolder>, anyhow::Error> {
        // let outDH = dh_in.asInstanceOf[DateAttributeDataHolder];
        let mut dh_out = match dh_in {
            AttributeDataHolder::DateAttributeDH{ dadh } => DateAttributeDH {
                attr_type_id: dadh.attr_type_id,
                date: dadh.date,
            },
            _ => {
                return Err(anyhow!("unexpected enum variant: {:?}", dh_in));
            }
        };

        //%% skipping this date processing for now, but make it convenient again later for omitting parts, or specifying detail.
        // // make the DateFormat omit trailing zeros, for editing convenience (to not have to backspace thru the irrelevant parts if not specified):
        // let mut dateFormatString = "yyyy-MM-dd";
        // let milliseconds: String = new java.text.SimpleDateFormat("SSS").format(new java.util.Date(dh_in.date));
        // let seconds: String = new java.text.SimpleDateFormat("ss").format(new java.util.Date(dh_in.date));
        // let minutes: String = new java.text.SimpleDateFormat("mm").format(new java.util.Date(dh_in.date));
        // let hours: String = new java.text.SimpleDateFormat("HH").format(new java.util.Date(dh_in.date));
        // if milliseconds != "000") {
        //   dateFormatString = dateFormatString + " HH:mm:ss:SSS zzz"
        // } else if seconds != "00") {
        //   dateFormatString = dateFormatString + " HH:mm:ss zzz"
        // } else if minutes != "00" || hours != "00") {
        //   dateFormatString = dateFormatString + " HH:mm zzz"
        // }
        // let dateFormat = new java.text.SimpleDateFormat(dateFormatString);
        // let default_value: String = {
        //   if editing_in dateFormat.format(new Date(dh_in.date))
        //   else Util::DATEFORMAT.format(System.currentTimeMillis())
        // }

        let date_criteria = |date: &str/*, ui: &TextUI%%*/| -> Result<(), anyhow::Error> {
            //%% !Util::finish_and_parse_the_date(date, true, ui)._2
            let new_date: ParseResult<DateTime<FixedOffset>> =
                DateTime::parse_from_str(date, Util::DATEFORMAT4);
            match new_date {
                Err(e) => {
                    //%%now handled by passing in the err and the ask_for_string3 shows the
                    //message. See note re testing it, in ui.ask_for_string5 (called below).
                    // ui.display_text1(
                    //     format!("Could not recognize date format: {}", e.to_string()).as_str()
                    // );
                    Err(anyhow!(e.to_string()))
                },
                Ok(_) => Ok(()),
            }
        };

        let answer = ui.ask_for_string5(
            vec![Util::GENERIC_DATE_PROMPT],
            Some(date_criteria),
            "", /*%%default_value.as_str()*/
            false,
            true, 
            Some("Could not recognize date format"),
        );
        match answer {
            None => Ok(None),
            Some(s) => {
                // let (new_date: Option<i64>, retry: bool) = Util::finish_and_parse_the_date(ans.get, true, ui);
                let new_date: ParseResult<DateTime<FixedOffset>> =
                    DateTime::parse_from_str(s.as_str(), Util::DATEFORMAT4);
                match new_date {
                    Err(e) => Err(anyhow!(e.to_string())),
                    Ok(dt) => {
                        dh_out.date = dt.timestamp();
                        Ok(Some(AttributeDataHolder::DateAttributeDH { dadh: dh_out }))
                    }
                }
            }
        }
    }

    /// Returns None if user wants to cancel.
    pub fn ask_for_boolean_attribute_value(
        _: &Controller,
        _: Rc<RefCell<dyn Database>>,
        dh_in: &mut AttributeDataHolder, /*BooleanAttributeDH,%%*/
        editing_in: bool,
        ui: &TextUI,
    ) -> Result<Option<AttributeDataHolder>, anyhow::Error> {
        let badh = match dh_in {
            AttributeDataHolder::BooleanAttributeDH { badh } => badh,
            _ => return Err(anyhow!("Unexpected AttributeDataHolder type: {:?}", dh_in)),
        };
        let bool_in = badh.boolean;
        let default_value = 
            {
                if editing_in && bool_in {
                    "y"
                } else {
                    "n"
                }
            };
        let answer = ui.ask_yes_no_question(
            "Set the value to true now? ('y' if so, 'n' for false)",
            default_value,
            false,
        );
        match answer {
            None => Ok(None),
            Some(ans) => {
                let dhv = AttributeDataHolder::BooleanAttributeDH { 
                    badh: BooleanAttributeDH { attr_type_id: badh.attr_type_id, 
                        valid_on_date: badh.valid_on_date, 
                        observation_date: badh.observation_date, 
                        boolean: ans 
                    }
                };
                Ok(Some(dhv))
            }
        }
    }

    /*%% just to quick ref/view code copied from above:
               let path: std::path::PathBuf = std::env::temp_dir().with_file_name(filename);
               let file = std::fs::OpenOptions::new().read(true).open(path);
               match file {
                   Err(_) => false,
                   Ok(_) => true,
               std::fs::write(path.to_str(), input)?;
    */

    /// Returns None if user wants to cancel/get out.
    //%%NEED TO TEST THIS EXPLICITLY
    pub fn ask_for_file_attribute_info(
        _: &dyn Database,
        dh_in: AttributeDataHolder, /*%%%%%%%::FileAttributeDH*/
        editing_in: bool,
        ui: &TextUI,
    ) -> Result<Option<AttributeDataHolder /*%%%%%%%FileAttributeDH*/>, anyhow::Error> {
        //Doing these things with DataHolders might be excessive in Rust, where it made more sense
        //in Scala. But these ask_for* methods or the like are passed in to various places so they
        //need the same signature, and doing this seems expedient for now. Maybe could be
        //re-evaluated and simplified.
        let dh_in = match dh_in {
            AttributeDataHolder::FileAttributeDH { fadh } => fadh,
            _ => return Err(anyhow!("Unexpected AttributeDataHolder variant: {:?}", dh_in)),
        };
        let mut path: Option<String> = None;
        if !editing_in {
            // I.e., not editing an existing fileAttribute, but adding a new fileAttribute (%%right??).
            // We don't want the original path to be editable after the fact, because that's a
            // historical observation and there is no sense in changing it.
            let leading_text = vec![
                "Enter file path (must exist and be readable), then press Enter; ESC to cancel",
            ];
            path = ui.ask_for_string3(leading_text, Some(Util::input_file_valid), "");
        }
        //%%deletable attempt at new logic w/ match, but too confusing to maintain old ideas
        // match path {
        //     None => {
        //         if !editing_in {
        //             None
        //         } else {
        //             %%?
        //         }
        //     },
        //     Some(p) => {
        //         if !editing_in {
        //             dh.original_file_path = p;
        //         } else {
        //             %%?
        //         }
        //         dh
        //     }
        // }

        // %%new logic, trying to emulate old in rust.  (Idea: Would this method's intent and
        // logic be clearer if we refactor the method to have one branch for editing and another
        // for adding (!editing_in)?)
        if !editing_in && path.is_none() {
            Ok(None)
        } else {
            let mut fadh_out = FileAttributeDH { 
                attr_type_id: dh_in.attr_type_id, 
                description: dh_in.description.clone(), 
                original_file_path: dh_in.original_file_path.clone() 
            };
            // if we can't fill in the path variables by now, there is a bug:
            if !editing_in {
                // unwrap guaranteed to work here due to "if" condition just above
                let s = path.as_ref().unwrap();
                fadh_out.original_file_path = s.clone();
            } else {
                // The default value for fadh_out.original_file_path is stored just above 
                // when creating fadh_out.
                path = Some(dh_in.original_file_path.clone());
            }
            let default_description_value: String = if editing_in {
                let s = dh_in.description.clone();
                s
            } else {
                // unwrap guaranteed to work due to above conditional logic setting path.
                let p = path.as_ref().unwrap().clone();
                let file_stem = std::path::Path::new(&p).file_stem();
                match file_stem {
                    None => "".to_string(), //"some_file_name".to_string(),
                    Some(os_string) => match os_string.to_str() {
                        None => {
                            "(file_stem_is_not_a_valid_unicode_string--\
                                  delete_this_and_enter_your_own_description)".to_string()
                        }
                        Some(s) => s.to_string(),
                    },
                }
            };
            let leading_text = vec!["Type file description, then press Enter; ESC to cancel"];
            let answer = ui.ask_for_string3(leading_text, None, default_description_value.as_str());
            match answer {
                None => Ok(None),
                Some(ans) => {
                    fadh_out.description = ans;
                    let dh_out = AttributeDataHolder::FileAttributeDH { fadh: fadh_out };
                    Ok(Some(dh_out))
                }
            }
        }

        //%%old scala logic, partly rustified but same logic for temp reference
        // if !editing_in && path.isEmpty {
        //     None
        // } else {
        //   // if we can't fill in the path variables by now, there is a bug:
        //   if !editing_in dh.original_file_path = path.get
        //   else path = Some(dh.original_file_path)
        //
        //   let default_value: Option<String> = if editing_in Some(dh_in.description) else Some(FilenameUtils.getBaseName(path.get));
        //   let ans = ui.ask_for_string(Some(Array("Type file description, then press Enter; ESC to cancel")), None, default_value);
        //   if ans.isEmpty None
        //   else {
        //     outDH.description = ans.get
        //     Some(outDH)
        //   }
        // }
    }

    /// Returns None if user just wants out; a String (user's answer, not useful outside this method) if
    /// update was done.
    /// This is here instead of on the struct Group, to avoid model objects (like Group) depending on UI code,
    /// which seems wrong.
    pub fn edit_group_name(group_in: &mut Group, ui: &TextUI) -> Result<Option<String>, anyhow::Error> {
        // doesn't seem to make sense to ck for duplicate names here: the real identity depends on what it relates to, and dup names may be common.
        let answer = ui.ask_for_string3(
            vec![Util::RELATION_TO_GROUP_NAME_PROMPT],
            None,
            group_in.get_name(None)?.as_str(),
        );
        match answer {
            None => Ok(None),
            Some(ref ans) => {
                if ans.trim().len() == 0 {
                    Ok(None)
                } else {
                    group_in.update(None, None, Some(&ans.trim().to_string()), None, None, None, None);
                    Ok(answer)
                }
            }
        }
    }

    //Some convenience functions.
    //%%Now that they exist, should probably use them in more of the
    //model classes like those with read_data_from_db, etc?
    //Is there an even better way to do this?
    pub fn get_value_bigint(description: &str, x: &Option<DataType>) -> Result<i64, anyhow::Error> {
        match x {
            Some(DataType::Bigint(x)) => Ok(*x),
            _ => return Err(anyhow!("Unexpected value for {}: {:?}", description, x)),
        }
    }
    pub fn get_value_bigint_option(
        description: &str,
        x: &Option<DataType>,
    ) -> Result<Option<i64>, anyhow::Error> {
        match x {
            Some(DataType::Bigint(x)) => Ok(Some(*x)),
            None => Ok(None),
            _ => return Err(anyhow!("Unexpected value for {}: {:?}", description, x)),
        }
    }
    pub fn get_value_float(description: &str, x: &Option<DataType>) -> Result<f64, anyhow::Error> {
        match x {
            Some(DataType::Float(x)) => Ok(*x),
            _ => return Err(anyhow!("Unexpected value for {}: {:?}", description, x)),
        }
    }
    pub fn get_value_bool(description: &str, x: &Option<DataType>) -> Result<bool, anyhow::Error> {
        match x {
            Some(DataType::Boolean(x)) => Ok(*x),
            _ => return Err(anyhow!("Unexpected value for {}: {:?}", description, x)),
        }
    }
    pub fn get_value_bool_option(
        description: &str,
        x: &Option<DataType>,
    ) -> Result<Option<bool>, anyhow::Error> {
        match x {
            Some(DataType::Boolean(x)) => Ok(Some(*x)),
            None => Ok(None),
            _ => return Err(anyhow!("Unexpected value for {}: {:?}", description, x)),
        }
    }
    pub fn get_value_string(
        description: &str,
        x: &Option<DataType>,
    ) -> Result<String, anyhow::Error> {
        match x {
            Some(DataType::String(x)) => Ok(x.to_string()),
            _ => return Err(anyhow!("Unexpected value for {}: {:?}", description, x)),
        }
    }
    pub fn get_value_smallint(
        description: &str,
        x: &Option<DataType>,
    ) -> Result<i32, anyhow::Error> {
        match x {
            Some(DataType::Smallint(x)) => Ok(*x),
            _ => return Err(anyhow!("Unexpected value for {}: {:?}", description, x)),
        }
    }
    /*
    package org.onemodel.core

    import java.io.{BufferedReader, PrintWriter, StringWriter}
    import java.nio.file.{Files, Path}
    import java.util.Date

    import org.apache.commons.io.FilenameUtils
    import org.onemodel.core.model._

    import scala.annotation.tailrec
    %%*/

    pub fn current_or_remote_db(relation_to_entity_in: Rc<RefCell<dyn Attribute>>, current_db: Rc<RefCell<dyn Database>>) 
        -> Result<Rc<RefCell<dyn Database>>, anyhow::Error> 
    {
        //%%delme--done in if below when it throws exception: 
        //require(relationToEntityIn.isInstanceOf[RelationToLocalEntity] || relationToEntityIn.isInstanceOf[RelationToRemoteEntity])
        
        // Can't use ".is_remote" here because a RelationToRemoteEntity is stored locally (so would say false),
        // but refers to an entity which is remote (so we want the next line to be true in that case):
        // if relation_to_entity_in.isInstanceOf[RelationToRemoteEntity]) {
        let rte = relation_to_entity_in.clone();
        let rte = &rte as &dyn Any;
        let rtreo = rte.downcast_ref::<RelationToRemoteEntity>();
        // if let Some(rtre) = rtreo {
        if rtreo.is_some() {
            //%% Ok(rtre.get_remote_database())
            Err(anyhow!("unimplemented%%"))
        } else {
            let rtleo = rte.downcast_ref::<RelationToLocalEntity>();
            if rtleo.is_some() { 
                //%% if let Some(_rtle) = rtleo {
                Ok(current_db)
            } else {
                Err(anyhow!("Unexpected type: {:?}", relation_to_entity_in))
            }
        }
    }

}
