/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003-2004 and 2008-2017 inclusive, 2019, and 2023, Luke A. Call.
    (That copyright statement was previously 2013-2015, until I remembered that much of Controller came from TextUI.scala, and TextUI.java before that.
    And this file initially came from Controller.scala.)
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
// use crate::controllers::controller::Controller;
// use crate::text_ui::TextUI;

/** This is just a place to put shared code ("Utility") until a grouping for some, or a better idea emerges.  Using it also
 * had (in Scala anyway) the benefit of making the Controller file smaller, so it is more quickly compiled (especially by the IDE).
 */
pub struct Util {}

impl Util {
    /// These constants are%%/were here because their presence in database.rs prevents Database from being used
    /// as a trait object.  See https://doc.rust-lang.org/reference/items/traits.html#object-safety etc for details.
    /// (Maybe they could go into model/mod.rs or some new struct file instead; haven't tried that.)
    fn entity_name_length() -> u32 { 160 }

    // in postgres, one table "extends" the other (see comments in createTables)
    fn relation_type_name_length() -> u32 {
        Self::entity_name_length()
    }

    fn class_name_length() -> u32 {
        Self::entity_name_length()
    }


    pub fn max_name_length() -> u32 {
        std::cmp::max(std::cmp::max(Self::entity_name_length(),
                                    Self::relation_type_name_length()),
                      Self::class_name_length())
        // std::cmp::max(std::cmp::max(crate::model::database::Database::entityNameLength(),
        //                             Database::relationTypeNameLength()),
        //               Database::classNameLength())
    }
    /* %%
      // should these be more consistently upper-case? What is the scala style for constants?  similarly in other classes.
      let NEWLN: String = System.getProperty("line.separator");
      // Might not be the most familiar date form for us Americans, but it seems the most useful in the widest
      // variety of situations, and more readable than with the "T" embedded in place of
      // the 1st space.  So, this approximates iso-8601.
      // these are for input.
      let DATEFORMAT = new java.text.SimpleDateFormat("yyyy-MM-dd HH:mm:ss:SSS zzz");
      let DATEFORMAT2 = new java.text.SimpleDateFormat("yyyy-MM-dd HH:mm:ss zzz");
      let DATEFORMAT3 = new java.text.SimpleDateFormat("yyyy-MM-dd HH:mm zzz");
      let DATEFORMAT_WITH_ERA = new java.text.SimpleDateFormat("GGyyyy-MM-dd HH:mm:ss:SSS zzz");
      let DATEFORMAT2_WITH_ERA = new java.text.SimpleDateFormat("GGyyyy-MM-dd HH:mm:ss zzz");
      let DATEFORMAT3_WITH_ERA = new java.text.SimpleDateFormat("GGyyyy-MM-dd HH:mm zzz");

      let DOES_NOT_EXIST = " does not exist in database.";

      //these are here to avoid colliding with use of the same names within other code inside the class.
      // idea: see what scala does with enums and/or constants; update this style?
      let ENTITY_TYPE: String = "Entity";
      let QUANTITY_TYPE: String = "QuantityAttribute";
      let TEXT_TYPE: String = "TextAttribute";
      let DATE_TYPE: String = "DateAttribute";
      let BOOLEAN_TYPE: String = "boolAttribute";
      let FILE_TYPE: String = "FileAttribute";
      let nonRelationAttrTypeNames = Array(Util.QUANTITY_TYPE, Util.DATE_TYPE, Util.BOOLEAN_TYPE, Util.FILE_TYPE, Util.TEXT_TYPE);
      //i.e., "relationTypeType", or the thing that we sometimes put in an attribute type parameter, though not exactly an attribute type, which is "RelationType":
      let RELATION_TYPE_TYPE: String = "RelationType";
      // IF/WHEN EVER UPDATING THESE TABLE NAMES, also update in cleanTestAccount.psql:
      let RELATION_TO_LOCAL_ENTITY_TYPE: String = "RelationToEntity";
      let RELATION_TO_GROUP_TYPE: String = "RelationToGroup";
      let RELATION_TO_REMOTE_ENTITY_TYPE: String = "RelationToRemoteEntity";
      let relationAttrTypeNames = Array(Util.RELATION_TYPE_TYPE, Util.RELATION_TO_LOCAL_ENTITY_TYPE, Util.RELATION_TO_REMOTE_ENTITY_TYPE,;
                                        Util.RELATION_TO_GROUP_TYPE)
      let GROUP_TYPE: String = "Group";
      let ENTITY_CLASS_TYPE: String = "Class";
      let OM_INSTANCE_TYPE: String = "Instance";

      let ORPHANED_GROUP_MESSAGE: String = "There is no entity with a containing relation to the group (orphaned).  You might search for it" +;
                                           " (by adding it as an attribute to some entity)," +
                                           " & see if it should be deleted, kept with an entity, or left out there floating." +
                                           "  (While this is not an expected usage, it is allowed and does not imply data corruption.)"

      let unselectMoveTargetPromptText: String = "Unselect current move target (if present; not necessary really)";

      // This says 'same screenful' because it's easier to assume that the returned index refers to the currently available
      // local collections (a subset of all possible entries, for display), than calling chooseOrCreateObject, and sounds as useful:
      let unselectMoveTargetLeadingText: String = "CHOOSE AN ENTRY (that contains only one subgroup) FOR THE TARGET OF MOVES (choose from SAME SCREENFUL as " +;
                                                  "now;  if the target contains 0 subgroups, or 2 or more subgroups, " +
                                                  "use other means to move entities to it until some kind of \"move anywhere\" feature is added):"

      let defaultPreferencesDepth = 10;
      // Don't change these: they get set and looked up in the data for preferences. Changing it would just require users to reset it though, and would
      // leave the old as clutter in the data.
      let USER_PREFERENCES = "User preferences";
      final let SHOW_PUBLIC_PRIVATE_STATUS_PREFERENCE = "Should entity lists show public/private status for each?";
      final let DEFAULT_ENTITY_PREFERENCE = "Which entity should be displayed as default, when starting the program?";
      // (If change next line, also change the hard-coded use in the file first.exp.)
      let HEADER_CONTENT_TAG = "htmlHeaderContent";
      let BODY_CONTENT_TAG = "htmlInitialBodyContent";
      let FOOTER_CONTENT_TAG = "htmlFooterContent";

      let LOCAL_OM_INSTANCE_DEFAULT_DESCRIPTION = "(local: not for self-connection but to serve id to remotes)";

        fn getClipboardContent -> String {
        let clipboard: java.awt.datatransfer.Clipboard = java.awt.Toolkit.getDefaultToolkit.getSystemClipboard;
        let contents: String = clipboard.getContents(null).getTransferData(java.awt.datatransfer.DataFlavor.stringFlavor).toString;
        contents.trim
        //(example of placing data on the clipboard, for future reference:)
        //val selection = new java.awt.datatransfer.StringSelection("someString")
        //clipboard.setContents(selection, null)
      }
    */

    pub fn is_windows() -> bool {
        let os = std::env::consts::OS;
        os.eq("windows")
    }

    /* %%
      // Used for example after one has been deleted, to put the highlight on right next one:
      // idea: This feels overcomplicated.  Make it better?  Fixing bad smells in general (large classes etc etc) is on the task list.
      /**
       * @param objectSetSize # of all the possible entries, not reduced by what fits in the available display space (I think).
       * @param objectsToDisplayIn  Only those that have been chosen to display (ie, smaller list to fit in display size size) (I think).
       * @return
       */
        fn findEntityToHighlightNext(objectSetSize: Int, objectsToDisplayIn: java.util.ArrayList[Entity], removedOneIn: bool,
                                    previouslyHighlightedIndexInObjListIn: Int, previouslyHighlightedEntryIn: Entity) -> Option[Entity] {
        //NOTE: SIMILAR TO findAttributeToHighlightNext: WHEN MAINTAINING ONE, DO SIMILARLY ON THE OTHER, until they are merged maybe by using the scala type
        //system better.

        // here of course, previouslyHighlightedIndexInObjListIn and objIds.size were calculated prior to the deletion.
        if (removedOneIn) {
          let newObjListSize = objectSetSize - 1;
          let newIndexToHighlight = math.min(newObjListSize - 1, previouslyHighlightedIndexInObjListIn);
          if (newIndexToHighlight >= 0) {
            if (newIndexToHighlight != previouslyHighlightedIndexInObjListIn) Some(objectsToDisplayIn.get(newIndexToHighlight))
            else {
              if (newIndexToHighlight + 1 < newObjListSize - 1) Some(objectsToDisplayIn.get(newIndexToHighlight + 1))
              else if (newIndexToHighlight - 1 >= 0) Some(objectsToDisplayIn.get(newIndexToHighlight - 1))
              else None
            }
          } else None
        } else Some(previouslyHighlightedEntryIn)
      }

      /** SEE COMMENTS FOR findEntityToHighlightNext. */
        fn findAttributeToHighlightNext(objectSetSize: Int, objectsToDisplayIn: java.util.ArrayList[Attribute], removedOne: bool,
                                       previouslyHighlightedIndexInObjListIn: Int, previouslyHighlightedEntryIn: Attribute) -> Option[Attribute] {
        //NOTE: SIMILAR TO findEntityToHighlightNext: WHEN MAINTAINING ONE, DO SIMILARLY ON THE OTHER, until they are merged maybe by using the scala type
        //system better.
        if (removedOne) {
          let newObjListSize = objectSetSize - 1;
          let newIndexToHighlight = math.min(newObjListSize - 1, previouslyHighlightedIndexInObjListIn);
          if (newIndexToHighlight >= 0) {
            if (newIndexToHighlight != previouslyHighlightedIndexInObjListIn) {
              Some(objectsToDisplayIn.get(newIndexToHighlight))
            } else {
              if (newIndexToHighlight + 1 < newObjListSize - 1) Some(objectsToDisplayIn.get(newIndexToHighlight + 1))
              else if (newIndexToHighlight - 1 >= 0) Some(objectsToDisplayIn.get(newIndexToHighlight - 1))
              else None
            }
          } else None
        } else Some(previouslyHighlightedEntryIn)
      }
*/
      pub fn get_default_user_login() -> Result<(String, &'static str), String> {
          //%%how do platform-independently? some crate? std doesn't seem to have a clear answer.
          //was in scala: (System.getProperty("user.name"), "x")
          match std::env::var("USER") {
              Ok(val) => Ok((val, "x")),
              Err(e) => {
                  let msg = e.to_string();
                  Err(msg)
              },
          }
      }
/*
      // ****** MAKE SURE THE NEXT 2 LINES MATCH THE FORMAT of Controller.DATEFORMAT, AND THE USER EXAMPLES IN THIS CLASS' OUTPUT! ******
      // Making this a var so that it can be changed for testing consistency (to use GMT for most tests so hopefully they will pass for developers in;
      // another time zone.  idea:  It seems like there's a better way to solve that though, maybe with a subclass of Controller in the test,
      // or of SimpleDateFormat.)
      let mut timezone: String = new java.text.SimpleDateFormat("zzz").format(System.currentTimeMillis());
      // (This isn't intended to match the date represented by a long value of "0", but is intended to be a usable value to fill in the rest of whatever a user
      // doesn't.  Perhaps assuming that the user will always put in a year if they put in anything (as currently enforced by the code at this time of writing).
        fn blankDate -> String
        {
        "1970-01-01 00:00:00:000 " + timezone
        }

      let mRelTypeExamples = "i.e., ownership of or \"has\" another entity, family tie, &c";

      // (the startup message already suggests that they create it with their own name, no need to repeat that here:    )
      let menuText_createEntityOrAttrType: String = "Add new entity (or new type like length, for use with quantity, true/false, date, text, or file attributes)";
      let menuText_createRelationType: String = "Add new relation type (" + mRelTypeExamples + ")";
      let mainSearchPrompt = "Search all / list existing entities (except quantity units, attr types, & relation types)";
      let menuText_viewPreferences: String = "View preferences";


      // date stuff
      let VALID = "valid";
      let OBSERVED = "observed";
      let genericDatePrompt: String = "Please enter the date like this, w/ at least the year, and other parts as desired: \"2013-01-31 23:59:59:999 MDT\"; zeros are " +;
                                      "allowed in all but the yyyy-mm-dd)." +
                                      //THIS LINE CAN BE PUT BACK AFTER the bug is fixed so ESC really works.  See similar cmt elsewhere; tracked in tasks:
                                      //"  Or ESC to exit.  " +
                                      "\"BC\" or \"AD\" prefix allowed (before the year, with no space)."
      let tooLongMessage = "value too long for type";

        fn entityMenuLeadingText(entityIn: Entity) {
        "**CURRENT ENTITY " + entityIn.getId + ": " + entityIn.getDisplayString(withColor = true)
      }

        fn groupMenuLeadingText(groupIn: Group) {
        "**CURRENT GROUP " + groupIn.getId + ": " + groupIn.getDisplayString()
      }

      let quantityTypePrompt: String = "SELECT TYPE OF QUANTITY (type is like length or volume, but not the measurement unit); ESC or leave both blank to cancel; " +;
                                       "cancel if you need to create the needed type before selecting): "
      let textDescription: String = "TEXT (ex., serial #)";

        fn canEditAttributeOnSingleLine(attributeIn: Attribute) -> bool {
        ! attributeIn.isInstanceOf[FileAttribute]
      }

        fn getUsableFilename(originalFilePathIn: String): (String, String) {
        FileAttribute.getUsableFilename(originalFilePathIn)
        }

      let entityPartsThatCanBeAffected: String = "ALL its attributes, actions, and relations, but not entities or groups the relations refer to";

      let listNextItemsPrompt = "List next items";
      let listPrevItemsPrompt = "List previous items";
      let relationToGroupNamePrompt = "Type a name for this group (ex., \"xyz list\"), then press Enter; blank or ESC to cancel";

        fn addRemainingCountToPrompt(choicesIn: Array[String], numDisplayedObjects: i64, totalRowsAvailableIn: i64,
                                    startingDisplayRowIndexIn: i64) -> Array[String] {
        let numLeft = totalRowsAvailableIn - startingDisplayRowIndexIn - numDisplayedObjects;
        let indexOfPrompt = choicesIn.indexOf(listNextItemsPrompt);
        if (numLeft > 0 && indexOfPrompt >= 0) {
          choicesIn(indexOfPrompt) = listNextItemsPrompt + " (of " + numLeft + " more)"
        }
        choicesIn
      }

        fn getContainingEntitiesDescription(entityCountNonArchivedIn: i64, entityCountArchivedIn: i64) -> String {
        "contained in " + entityCountNonArchivedIn + " entities, and in " + entityCountArchivedIn + " archived entities"
      }

      let pickFromListPrompt: String = "Pick from menu, or an item by letter to select; Alt+<letter> to go to the item then come back here";

        fn searchPromptPart(typeIn: String) -> String {
         "Enter part of the " + typeIn + " name to search for."
         }

        fn entityOrGroupNameSqlSearchPrompt(typeNameIn: String) -> String {
        searchPromptPart(typeNameIn) + "  (For the curious: it will be used in matching as a " +
        "case-insensitive POSIX " +
        "regex; details at  http://www.postgresql.org/docs/current/static/functions-matching.html#FUNCTIONS-POSIX-REGEXP .)"
      }

        fn isNumeric(input: String) -> bool {
        // simplicity over performance in this case:
        try {
          // throws an exception if not numeric:
          input.toFloat
          true
        } catch {
          case e: NumberFormatException => false
        }
      }

        fn inputFileValid(path: String) -> bool {
        let file = new java.io.File(path);
        file.exists && file.canRead
      }

      // The check to see if a long date string is valid comes later.
      // Now that we allow 1-digit dates, there is nothing to ck really.
        fn validOnDateCriteria(dateStr: String) -> bool { true }
      // Same comments as for observedDateCriteria:
        fn observedDateCriteria(dateStr: String) -> bool { true }

        fn throwableToString(e: Throwable) -> String {
        let stringWriter = new StringWriter();
        e.printStackTrace(new PrintWriter(stringWriter))
        stringWriter.toString
      }

        fn handleException(e: Throwable, ui: TextUI, db: Database) {
        if (e.isInstanceOf[org.postgresql.util.PSQLException] || e.isInstanceOf[OmDatabaseException] ||
            throwableToString(e).contains("ERROR: current transaction is aborted, commands ignored until end of transaction block"))
        {
          db.rollbackTrans()
        }
        // If changing this string (" - 1"), also change in first.exp that looks for it (distinguished from " - 2" elsewhere).
        let ans = ui.askYesNoQuestion("An error occurred: \"" + e.getClass.getName + ": " + e.getMessage + "\".  If you can provide simple instructions to " +;
                                      "reproduce it consistently, maybe it can be fixed - 1.  Do you want to see the detailed output?")
        if (ans.isDefined && ans.get) {
          ui.display_text(throwableToString(e))
        }
      }

      /** A helper method.  Returns the date as a i64 (java-style: ms since 1970 began), and true if there is a problem w/ the string and we need to ask again. */
        fn finishAndParseTheDate(dateStrIn: String, blankMeansNOW: bool = true, ui: TextUI) -> (Option<i64>, bool) {
        //to start with, the special forms (be sure to trim the input, otherwise there's no way in the textui to convert from a previously entered (so default)
        //value to "blank/all time"!).
        let dateStrWithOptionalEra =;
          if (dateStrIn.equalsIgnoreCase("now") || (blankMeansNOW && dateStrIn.trim.length() == 0)) {
            let currentDateString: String = Util.DATEFORMAT.format(new java.util.Date(System.currentTimeMillis()));
            currentDateString
          }
          else dateStrIn.trim

        // chop off the era before doing some of the other logic
        let (era: String, dateStr) =;
          if (dateStrWithOptionalEra.toUpperCase.startsWith("AD") || dateStrWithOptionalEra.toUpperCase.startsWith("BC")) {
            (dateStrWithOptionalEra.substring(0, 2), dateStrWithOptionalEra.substring(2))
          } else ("", dateStrWithOptionalEra)

        // help user if they put in something like 2013-1-1 instead of 2013-01-01, so the parsed date isn't messed up. See test.
        // (The year could be other than 4 digits, so check for the actual location of the 1st hyphen):
        let firstHyphenPosition = if (dateStr.indexOf('-') != -1) dateStr.indexOf('-') else dateStr.length;
        //but only if the string format looks somewhat expected; otherwise let later parsing handle it all.
        let filledInDateStr =;
          if (dateStr.length > firstHyphenPosition + 1 && dateStr.length < firstHyphenPosition + 6
              && dateStr.indexOf('-') == firstHyphenPosition && dateStr.indexOf('-', firstHyphenPosition + 1) >= 0) {
            let secondHyphenPosition = dateStr.indexOf('-', firstHyphenPosition + 1);
            if (secondHyphenPosition == firstHyphenPosition + 2 || secondHyphenPosition == firstHyphenPosition + 3) {
              if (dateStr.length == secondHyphenPosition + 2 || dateStr.length == secondHyphenPosition + 3) {
                let year = dateStr.substring(0, firstHyphenPosition);
                let mo = dateStr.substring(firstHyphenPosition + 1, secondHyphenPosition);
                let dy = dateStr.substring(secondHyphenPosition + 1);
                year + '-' + (if (mo.length == 1) "0" + mo else mo) + '-' + (if (dy.length == 1) "0" + dy else dy)
              }
              else dateStr
            }
            else dateStr
          } else if (dateStr.length == firstHyphenPosition + 2) {
            // also handle format like 2013-1
            let year = dateStr.substring(0, firstHyphenPosition);
            let mo = dateStr.substring(firstHyphenPosition + 1);
            year + '-' + "0" + mo
          }
          else dateStr


        // Fill in the date w/ "blank" information for whatever detail the user didn't provide:
        let filledInDateStrWithoutYear = if (firstHyphenPosition < filledInDateStr.length) filledInDateStr.substring(firstHyphenPosition + 1) else "";
        let year = filledInDateStr.substring(0, firstHyphenPosition);

        let blankDateWithoutYear = blankDate.substring(5);

        let dateStrWithZeros =;
          if (filledInDateStrWithoutYear.length() < blankDateWithoutYear.length) {
            year + '-' + filledInDateStrWithoutYear + blankDateWithoutYear.substring(filledInDateStrWithoutYear.length())
          }
          else filledInDateStr
        // then parse it:
        try {
          let d: java.util.Date =;
            try {
              if (era.isEmpty) Util.DATEFORMAT.parse(dateStrWithZeros)
              else Util.DATEFORMAT_WITH_ERA.parse(era + dateStrWithZeros)
            } catch {
              case e: java.text.ParseException =>
                try {
                  if (era.isEmpty) Util.DATEFORMAT2.parse(dateStrWithZeros)
                  else Util.DATEFORMAT2_WITH_ERA.parse(era + dateStrWithZeros)
                } catch {
                  case e: java.text.ParseException =>
                    if (era.isEmpty) Util.DATEFORMAT3.parse(dateStrWithZeros)
                    else Util.DATEFORMAT3_WITH_ERA.parse(era + dateStrWithZeros)
                }
            }
          (Some(d.getTime), false)
        } catch {
          case e: java.text.ParseException =>
            ui.display_text("Invalid date format. Try something like \"2003\", or \"2003-01-31\", or \"2003-01-31 22:15\" for 10:15pm, or if you need a timezone, " +
                           "all of \"yyyy-MM-dd HH:mm:ss:SSS zzz\", like for just before midnight: \"2013-01-31 //23:59:59:999 MST\".")
            (None, true)
        }
      }

      /** Returns (validOnDate, observationDate, userWantsToCancel) */
        fn askForAttributeValidAndObservedDates(inEditing: bool,
                                               oldValidOnDateIn: Option<i64>,
                                               oldObservedDateIn: i64,
                                               ui: TextUI) -> (Option<i64>, i64, bool) {
        //idea: make this more generic, passing in prompt strings &c, so it's more cleanly useful for DateAttribute instances. Or not: lacks shared code.
        //idea: separate these into 2 methods, 1 for each time (not much common material of significance).
        // BETTER IDEA: fix the date stuff in the DB first as noted in tasks, so that this part makes more sense (the 0 for all time, etc), and then
        // when at it, recombine the askForDate_Generic method w/ these or so it's all cleaned up.
        /** Helper method made so it can be recursive, it returns the date (w/ meanings as with display_text below, and as in PostgreSQLDatabase.createTables),
          * and true if the user wants to cancel/get out). */
        //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) w/in this method!
        @tailrec fn askForDate(dateTypeIn: String, acceptanceCriteriaIn: (String) => bool) -> (Option<i64>, bool) {
          let leadingText: Array[String] = {;
            if (dateTypeIn == VALID) {
              Array("\nPlease enter the date when this was first VALID (i.e., true) (Press Enter (blank) for unknown/unspecified, or " +
                    "like this, w/ at least the year: \"2002\", \"2000-1-31\", or" +
                    " \"2013-01-31 23:59:59:999 MST\"; zeros are " +
                    "allowed in all but the yyyy-mm-dd.  Or for current date/time " +
                    "enter \"now\".  ESC to exit this.  " +
                    "For dates far in the past you can prefix them with \"BC\" (or \"AD\", but either way omit a space " +
                    "before the year), like BC3400-01-31 23:59:59:999 GMT, entered at least up through the year, up to ~292000000 years AD or BC.")
              //IDEA: I had thought to say:  "Or for "all time", enter just 0.  ", BUT (while this is probably solved, it's not until the later part of
              // this comment):
              //    "There is ambiguity about BC that needs some " +
              //    "investigation, because java allows a '0' year (which for now means 'for all time' in just this program), but normal human time doesn't " +
              //    "allow a '0' year, so maybe you have to subtract a year from all BC things for them to work right, and enter/read them accordingly, until " +
              //    "someone learns for sure, and we decide whether to subtract a year from everything BC for you automatically. Hm. *OR* maybe dates in year " +
              //    "zero " +
              //    "just don't mean anything so can be ignored by users, and all other dates' entry are just fine, so there's nothing to do but use it as is? " +
              //    "But that would have to be kept in mind if doing any relative date calculations in the program, e.g. # of years, spanning 0.)\n" +
              //    "Also, real events with more " +
              //    "specific time-tracking needs will probably need to model their own time-related entity classes, and establish relations to them, within " +
              //    "their use of OM.")
              //ABOUT THAT LAST COMMENT: WHY DOES JAVA ALLOW A 0 YEAR, UNLESS ONLY BECAUSE IT USES long #'S? SEE E.G.
              // http://www.msevans.com/calendar/daysbetweendatesapplet.php
              //which says: "...[java?] uses a year 0, which is really 1 B.C. For B.C. dates, you have to remember that the years are off by one--10 B.C.
              // to [java?] is really 11 B.C.", but this really needs more investigation on what is the Right Thing to do.
              // Or, just let the dates go in & out of the data, interpreted just as they are now, but the savvy users will recognize that dates in year zero just
              // don't mean anything, thus the long values in that range don't mean anything so can be disregarded (is that how it really works in java??), (or if
              // so we could inform users when such a date is present, that it's bogus and to use 1 instead)?
              // **SO:** it is already in the task list to have a separate flag in the database for "all time".
            } else if (dateTypeIn == OBSERVED) {
              Array("\nWHEN OBSERVED?: " + genericDatePrompt + /*" (\"All time\" and*/ " \"unknown\" not" + " allowed here.) ")
            } else throw new scala.Exception("unexpected type: " + dateTypeIn)
          }

          let defaultValue: Option[String] = {;
            if (dateTypeIn == VALID) {
              if (inEditing && oldValidOnDateIn.isDefined) {
                if (oldValidOnDateIn.get == 0) Some("0")
                else Some(Util.DATEFORMAT_WITH_ERA.format(new Date(oldValidOnDateIn.get)))
              }
              else None
            } else if (dateTypeIn == OBSERVED) {
              if (inEditing) {
                Some(Util.DATEFORMAT_WITH_ERA.format(new Date(oldObservedDateIn)))
              } else {
                Some(Util.DATEFORMAT_WITH_ERA.format(new Date(System.currentTimeMillis())))
              }
            } else throw new scala.Exception("unexpected type: " + dateTypeIn)
          }

          let ans = ui.askForString(Some(leadingText), None, defaultValue);

          if (ans.isEmpty) {
            if (dateTypeIn == VALID) {
              // don't let user cancel from valid date: blank there means "unknown" (but user can ESC again from observed date. Making these
              // consistent probably means figuring out how to modify jline2 (again, now) so that it will distinguish between user pressing ESC and user
              // pressing Enter with a blank line: now IIRC it just returns a blank line for both.  Or something.
              (None, false)
            } else if (dateTypeIn == OBSERVED) {
              //getting out, but observed date not allowed to be None (see caller for details)
              (Some(0), true)
            }
            else throw new Exception("unexpected type: " + dateTypeIn)
          } else {
            let dateStr = ans.get.trim;
            if (dateTypeIn == VALID && dateStr.trim.length == 0) (None, false)
            else if (dateTypeIn == VALID && dateStr.trim == "0") (Some(0), false)
            else if (!acceptanceCriteriaIn(dateStr)) askForDate(dateTypeIn, acceptanceCriteriaIn)
            else {
              // (special values like "0" or blank are already handled above)
              let (newDate: Option<i64>, retry: bool) = finishAndParseTheDate(dateStr, dateTypeIn == OBSERVED, ui);
              if (retry) askForDate(dateTypeIn, acceptanceCriteriaIn)
              else {
                (newDate, false)
              }
            }
          }
        }

        // the real action:
        fn askForBothDates(ui: TextUI) -> (Option<i64>, i64, bool) {
          let (validOnDate, userCancelled) = askForDate(VALID, validOnDateCriteria);
          if (userCancelled) (None, 0, userCancelled)
          else {
            let (observedDate, userCancelled) = askForDate(OBSERVED, observedDateCriteria);
            if (userCancelled) (Some(0), 0, userCancelled)
            else {
              // (for why validOnDate is sometimes allowed to be None, but not observedDate: see let validOnPrompt.);
              require(observedDate.isDefined)
              let ans = ui.askYesNoQuestion("Dates are: " + AttributeWithValidAndObservedDates.getDatesDescription(validOnDate,;
                                                                                                                   observedDate.get) + ": right?", Some("y"))
              if (ans.isDefined && ans.get) (validOnDate, observedDate.get, userCancelled)
              else askForBothDates(ui)
            }
          }
        }
        askForBothDates(ui)
      }

      /** Cloned from Controller.askForDate; see its comments in the code.
        * Idea: consider combining somehow with method askForDateAttributeValue.
        * @return None if user wants out.
        */
      //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) w/in this method!
      @tailrec final fn askForDate_generic(promptTextIn: Option[String] = None, defaultIn: Option[String], ui: TextUI) -> Option<i64> {
        let leadingText: Array[String] = Array(promptTextIn.getOrElse(genericDatePrompt));
        let default: String = defaultIn.getOrElse(Util.DATEFORMAT.format(System.currentTimeMillis()));
        let ans = ui.askForString(Some(leadingText), None, Some(default));
        if (ans.isEmpty) None
        else {
          let dateStr = ans.get.trim;
          let (newDate: Option<i64>, retry: bool) = finishAndParseTheDate(dateStr, ui = ui);
          if (retry) askForDate_generic(promptTextIn, defaultIn, ui)
          else newDate
        }
      }
    */
    /** This gets the an abbreviated part of the copyright text to be used by the UI.  It is
    customized to the actual content of the LICENSE file, to extract & modify suitably for UI display.
    */
    pub fn license() -> String {
        let mut text_to_show = String::new();
        // Note: Before the next line was added, the binary
        // debug size was 4,884,336 bytes.
        // After the next line was added (with some other changes in the last commit), the binary
        // debug size was 5,066,280 (difference of 181,944), with a LICENSE file size of 38,816 bytes.
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
            if (!append) && line.starts_with("-----") && before_any_dashes {
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

        // String::from("[stub copyright]")
    } //%% just to end the fn temporarily w smthg until i convert it to rust
      /*  %%
          try {
            let reader: BufferedReader = {;
              // first try to get it from the jar being run by the user:
              let stream = this.getClass.getClassLoader.getResourceAsStream("LICENSE");
              if (stream != null) {
                new BufferedReader(new java.io.InputStreamReader(stream))
              } else {
                // failing that, check the filesystem, i.e., checking how it looks during development when the jar isn't built (or at least for consistent behavior
                // during development)
                new BufferedReader(scala.io.Source.fromFile("LICENSE").reader())
              }
            }
            // idea: do this in a most scala-like way, like w/ immutable "line", recursion instead of a while loop, and can its libraries simplify this?:
            ...
          }
          catch {
            case e: Exception =>
              let ans = ui.askYesNoQuestion("\n\nThe file LICENSE is missing from the distribution of this program or for " +;
                                            "some other reason can't be displayed normally; please let us know to " +
                                            " correct that, and please be aware of the license.  You can go to this URL to see it:\n" +
                                            "    http://onemodel.org/download/OM-LICENSE \n" +
                                            ".  (Do you want to see the detailed error output?)")
              if (ans.isDefined && ans.get) {
                // (" - 2" is to distinguished from " - 1" because some code looks for " - 1".)
                ui.display_text("The error was: \"" + e.getClass.getName + ": " + e.getMessage + "\".  If you can provide simple instructions to " +
                               "reproduce it consistently, maybe it can be fixed - 2.  " + Util.throwableToString(e))
              }
          }
          all
        }

          fn stringTooLongErrorMessage(nameLength: Int) -> String {
          // for details, see method PostgreSQLDatabase.escapeQuotesEtc.
          "Got an error.  Please try a shorter (" + nameLength + " chars) entry.  " +
          "(Could be due to escaped, i.e. expanded, characters like ' or \";\".  Details: %s"
        }

          fn isDuplicationAProblem(isDuplicateIn: bool, duplicateNameProbablyOK: bool, ui: TextUI) -> bool {
          let mut duplicateProblemSoSkip = false;
          if (isDuplicateIn) {
            if (!duplicateNameProbablyOK) {
              let answerOpt = ui.askForString(Some(Array("That name is a duplicate--proceed anyway? (y/n)")), None, Some("n"));
              if (answerOpt.isEmpty || (!answerOpt.get.equalsIgnoreCase("y"))) duplicateProblemSoSkip = true
            }
          }
          duplicateProblemSoSkip
        }

        //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) within this method, below!
        @tailrec final fn askForNameInReverseDirection(directionalityStrIn: String, nameLengthIn: Int, nameIn: String,
                                                        previousNameInReverseIn: Option[String] = None, ui: TextUI) -> String {
          // see createTables (or UI prompts) for meanings of bi/uni/non...
          if (directionalityStrIn == "UNI") ""
          else if (directionalityStrIn == "NON") nameIn
          else if (directionalityStrIn == "BI") {
            // see createTables (or UI prompts) for meanings...
            let msg = Array("Enter relation name when direction is reversed (i.e., 'is husband to' becomes 'is wife to', 'employs' becomes 'is employed by' " +;
                            "by; up to " + nameLengthIn + " characters (ESC to cancel): ")
            let nameInReverse = {;
              let ans: Option[String] = ui.askForString(Some(msg), None, previousNameInReverseIn);
              if (ans.isEmpty) return ""
              ans.get.trim() //see above comment about trim
            }
            let ans = ui.askWhich(Some(Array("Is this the correct name for the relationship in reverse direction?: ")), Array("Yes", "No"));
            if (ans.isEmpty || ans.get == 2) askForNameInReverseDirection(directionalityStrIn, nameLengthIn, nameIn, previousNameInReverseIn, ui)
            else nameInReverse
          }
          else throw new Exception("unexpected value for directionality: " + directionalityStrIn)
        }

          fn askForRelationDirectionality(previousDirectionalityIn: Option[String] = None, ui: TextUI) -> Option[String] {
          let msg = Array("Enter directionality (\"bi\", \"uni\", or \"non\"; examples: \"is parent of\"/\"is child of\" is bidirectional, " +;
                          "since it differs substantially by the direction but goes both ways; unidirectional might be like 'lists': the thing listed doesn't know " +
                          "it; \"is acquaintanted with\" could be nondirectional if it is an identical relationship either way  (ESC to cancel): ")
          fn criteria(entryIn: String) -> bool {
            let entry = entryIn.trim().toUpperCase;
            entry == "BI" || entry == "UNI" || entry == "NON"
          }

          let directionality = ui.askForString(Some(msg), Some(criteria(_: String)), previousDirectionalityIn);
          if (directionality.isEmpty) None
          else Some(directionality.get.toUpperCase)
        }

          fn editMultilineText(input: String, ui: TextUI) -> String {
          //idea: allow user to change the edit command setting (ie which editor to use) from here?

          //idea: allow user to prevent this message in future. Could be by using ui.askYesNoQuestion instead, adding to the  prompt "(ask this again?)", with
          // 'y' as default, and storing the answer in the db.SYSTEM_ENTITY_NAME somewhere perhaps.
          //PUT THIS BACK (& review/test it) after taking the time to read the Process package's classes or something like
          // apache commons has, and learn to launch vi workably, from scala. And will the terminal settings changes by OM have to be undone/redone for it?:
          //        let command: String = db.getTextEditorCommand;
          //        ui.display_text("Using " + command + " as the text editor, but you can change that by navigating to the Main OM menu with ESC, search for
          // existing " +
          //                       "entities, choose the first one (called " + PostgreSQLDatabase.SYSTEM_ENTITY_NAME + "), choose " +
          //                       PostgreSQLDatabase.EDITOR_INFO_ENTITY_NAME + ", choose " +
          //                       "" + PostgreSQLDatabase.TEXT_EDITOR_INFO_ENTITY_NAME + ", then choose the " +
          //                       PostgreSQLDatabase.TEXT_EDITOR_COMMAND_ATTRIBUTE_TYPE_NAME + " and edit it with option 3.")

          let path: Path = Files.createTempFile("om-edit-", ".txt");
          Files.write(path, input.getBytes)
          ui.display_text("Until we improve this, you can now go edit the content in this temporary file, & save it:\n" +
                         path.toFile.getCanonicalPath + "\n...then come back here when ready to import that text.")
          let newContent: String = new Predef.String(Files.readAllBytes(path));
          path.toFile.delete()
          newContent
        }

        /** Returns None if user just wants out. */
          fn promptWhetherTo1Add2Correct(inAttrTypeDesc: String, ui: TextUI) -> Option[Int] {
          //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) within this method, below!
          @tailrec fn ask -> Option[Int] {
            let ans = ui.askWhich(None, Array("1-Save this " + inAttrTypeDesc + " attribute?", "2-Correct it?"));
            if (ans.isEmpty) return None
            let answer = ans.get;
            if (answer < 1 || answer > 2) {
              ui.display_text("invalid response")
              ask
            } else Some(answer)
          }
          ask
        }

          fn askForQuantityAttributeNumber(previousQuantity: Float, ui: TextUI) -> Option[Float] {
          let leadingText = Array[String]("ENTER THE NUMBER FOR THE QUANTITY (i.e., 5, for 5 centimeters length)");
          let ans = ui.askForString(Some(leadingText), Some(Util.isNumeric), Some(previousQuantity.toString));
          if (ans.isEmpty) None
          else Some(ans.get.toFloat)
        }

        /** Returns None if user wants to cancel. */
          fn askForTextAttributeText(ignore: Database, inDH: TextAttributeDataHolder, inEditing: bool, ui: TextUI) -> Option[TextAttributeDataHolder] {
          let outDH = inDH.asInstanceOf[TextAttributeDataHolder];
          let defaultValue: Option[String] = if (inEditing) Some(inDH.text) else None;
          let ans = ui.askForString(Some(Array("Type or paste a single-line attribute value, then press Enter; ESC to cancel." +;
                                               "  (If you need to add or edit multiple lines, just " +
                                               "put in a single line or letter for now (or leave the multiple lines if already in place), then you can edit " +
                                               "it afterward to add the full text.  But consider if a 'file' attribute " +
                                               "or some other way of modeling the info " +
                                               "would be better at representing what it really *is*.  Legitimate use cases for a text attribute might include a " +
                                               "quote or a stack trace.)")), None, defaultValue)
          if (ans.isEmpty) None
          else {
            outDH.text = ans.get
            Some(outDH)
          }
        }

        /** Returns None if user wants to cancel.
          * Idea: consider combining somehow with method askForDate_generic or note here why not, perhaps.
          */
          fn askForDateAttributeValue(ignore: Database, inDH: DateAttributeDataHolder, inEditing: bool, ui: TextUI) -> Option[DateAttributeDataHolder] {
          let outDH = inDH.asInstanceOf[DateAttributeDataHolder];

          // make the DateFormat omit trailing zeros, for editing convenience (to not have to backspace thru the irrelevant parts if not specified):
          let mut dateFormatString = "yyyy-MM-dd";
          let milliseconds: String = new java.text.SimpleDateFormat("SSS").format(new java.util.Date(inDH.date));
          let seconds: String = new java.text.SimpleDateFormat("ss").format(new java.util.Date(inDH.date));
          let minutes: String = new java.text.SimpleDateFormat("mm").format(new java.util.Date(inDH.date));
          let hours: String = new java.text.SimpleDateFormat("HH").format(new java.util.Date(inDH.date));
          if (milliseconds != "000") {
            dateFormatString = dateFormatString + " HH:mm:ss:SSS zzz"
          } else if (seconds != "00") {
            dateFormatString = dateFormatString + " HH:mm:ss zzz"
          } else if (minutes != "00" || hours != "00") {
            dateFormatString = dateFormatString + " HH:mm zzz"
          }
          let dateFormat = new java.text.SimpleDateFormat(dateFormatString);
          let defaultValue: String = {;
            if (inEditing) dateFormat.format(new Date(inDH.date))
            else Util.DATEFORMAT.format(System.currentTimeMillis())
          }

          fn dateCriteria(date: String) -> bool {
            !Util.finishAndParseTheDate(date, ui = ui)._2
          }
          let ans = ui.askForString(Some(Array(Util.genericDatePrompt)), Some(dateCriteria), Some(defaultValue));
          if (ans.isEmpty) None
          else {
            let (newDate: Option<i64>, retry: bool) = Util.finishAndParseTheDate(ans.get, ui = ui);
            if (retry) throw new Exception("Programmer error: date indicated it was parseable, but the same function said afterward it couldn't be parsed.  Why?")
            else if (newDate.isEmpty) throw new Exception("There is a bug: the program shouldn't have got to this point.")
            else {
              outDH.date = newDate.get
              Some(outDH)
            }
          }
        }

        /** Returns None if user wants to cancel. */
          fn askForboolAttributeValue(ignore: Database, inDH: BooleanAttributeDataHolder, inEditing: bool, ui: TextUI) -> Option[BooleanAttributeDataHolder] {
          let outDH = inDH.asInstanceOf[BooleanAttributeDataHolder];
          let ans = ui.askYesNoQuestion("Set the new value to true now? ('y' if so, 'n' for false)", if (inEditing && inDH.boolean) Some("y") else Some("n"));
          if (ans.isEmpty) None
          else {
            outDH.boolean = ans.get
            Some(outDH)
          }
        }

        /** Returns None if user wants to cancel. */
          fn askForFileAttributeInfo(ignore: Database, inDH: FileAttributeDataHolder, inEditing: Boolean, ui: TextUI) -> Option[FileAttributeDataHolder] {
          let outDH = inDH.asInstanceOf[FileAttributeDataHolder];
          let mut path: Option[String] = None;
          if (!inEditing) {
            // we don't want the original path to be editable after the fact, because that's a historical observation and there is no sense in changing it.
            path = ui.askForString(Some(Array("Enter file path (must exist and be readable), then press Enter; ESC to cancel")), Some(Util.inputFileValid))
          }
          if (!inEditing && path.isEmpty) None
          else {
            // if we can't fill in the path variables by now, there is a bug:
            if (!inEditing) outDH.originalFilePath = path.get
            else path = Some(outDH.originalFilePath)

            let defaultValue: Option[String] = if (inEditing) Some(inDH.description) else Some(FilenameUtils.getBaseName(path.get));
            let ans = ui.askForString(Some(Array("Type file description, then press Enter; ESC to cancel")), None, defaultValue);
            if (ans.isEmpty) None
            else {
              outDH.description = ans.get
              Some(outDH)
            }
          }
        }

        /** Returns None if user just wants out; a String (user's answer, not useful outside this method) if update was done..
          */
          fn editGroupName(groupIn: Group, ui: TextUI) -> Option[String] {
          // doesn't seem to make sense to ck for duplicate names here: the real identity depends on what it relates to, and dup names may be common.
          let ans = ui.askForString(Some(Array(Util.relationToGroupNamePrompt)), None, Some(groupIn.getName));
          if (ans.isEmpty || ans.get.trim.length() == 0) {
            None
          } else {
            groupIn.update(None, Some(ans.get.trim), None, None, None, None)
            ans
          }
        }

      package org.onemodel.core

      import java.io.{BufferedReader, PrintWriter, StringWriter}
      import java.nio.file.{Files, Path}
      import java.util.Date

      import org.apache.commons.io.FilenameUtils
      import org.onemodel.core.model._

      import scala.annotation.tailrec
      */
}
