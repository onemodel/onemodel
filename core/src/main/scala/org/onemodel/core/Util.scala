/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003-2004 and 2008-2017 inclusive, Luke A. Call; all rights reserved.
    (That copyright statement was previously 2013-2015, until I remembered that much of Controller came from TextUI.scala, and TextUI.java before that.
    And this file initially came from Controller.scala.)
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule, guidelines around binary
    distribution, and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
package org.onemodel.core

import java.io.{BufferedReader, PrintWriter, StringWriter}
import java.nio.file.{Files, Path}
import java.util.Date

import org.apache.commons.io.FilenameUtils
import org.onemodel.core.model._

import scala.annotation.tailrec

/** This is just a place to put shared code ("Utility") until a grouping or better idea emerges.  Using it also
  * has the benefit of making the Controller file smaller, so it is more quickly compiled (especially by the IDE).
 */
object Util {
  // should these be more consistently upper-case? What is the scala style for constants?  similarly in other classes.
  def maxNameLength: Int = math.max(math.max(Database.entityNameLength, Database.relationTypeNameLength),
                                    Database.classNameLength)
  val NEWLN: String = System.getProperty("line.separator")
  // Might not be the most familiar date form for us Americans, but it seems the most useful in the widest
  // variety of situations, and more readable than with the "T" embedded in place of
  // the 1st space.  So, this approximates iso-8601.
  // these are for input.
  val DATEFORMAT = new java.text.SimpleDateFormat("yyyy-MM-dd HH:mm:ss:SSS zzz")
  val DATEFORMAT2 = new java.text.SimpleDateFormat("yyyy-MM-dd HH:mm:ss zzz")
  val DATEFORMAT3 = new java.text.SimpleDateFormat("yyyy-MM-dd HH:mm zzz")
  val DATEFORMAT_WITH_ERA = new java.text.SimpleDateFormat("GGyyyy-MM-dd HH:mm:ss:SSS zzz")
  val DATEFORMAT2_WITH_ERA = new java.text.SimpleDateFormat("GGyyyy-MM-dd HH:mm:ss zzz")
  val DATEFORMAT3_WITH_ERA = new java.text.SimpleDateFormat("GGyyyy-MM-dd HH:mm zzz")

  val DOES_NOT_EXIST = " does not exist in database."

  //these are here to avoid colliding with use of the same names within other code inside the class.
  // idea: see what scala does with enums and/or constants; update this style?
  val ENTITY_TYPE: String = "Entity"
  val QUANTITY_TYPE: String = "QuantityAttribute"
  val TEXT_TYPE: String = "TextAttribute"
  val DATE_TYPE: String = "DateAttribute"
  val BOOLEAN_TYPE: String = "BooleanAttribute"
  val FILE_TYPE: String = "FileAttribute"
  val nonRelationAttrTypeNames = Array(Util.QUANTITY_TYPE, Util.DATE_TYPE, Util.BOOLEAN_TYPE, Util.FILE_TYPE, Util.TEXT_TYPE)
  //i.e., "relationTypeType", or the thing that we sometimes put in an attribute type parameter, though not exactly an attribute type, which is "RelationType":
  val RELATION_TYPE_TYPE: String = "RelationType"
  // IF/WHEN EVER UPDATING THESE TABLE NAMES, also update in cleanTestAccount.psql:
  val RELATION_TO_LOCAL_ENTITY_TYPE: String = "RelationToEntity"
  val RELATION_TO_GROUP_TYPE: String = "RelationToGroup"
  val RELATION_TO_REMOTE_ENTITY_TYPE: String = "RelationToRemoteEntity"
  val relationAttrTypeNames = Array(Util.RELATION_TYPE_TYPE, Util.RELATION_TO_LOCAL_ENTITY_TYPE, Util.RELATION_TO_REMOTE_ENTITY_TYPE,
                                    Util.RELATION_TO_GROUP_TYPE)
  val GROUP_TYPE: String = "Group"
  val ENTITY_CLASS_TYPE: String = "Class"
  val OM_INSTANCE_TYPE: String = "Instance"

  val ORPHANED_GROUP_MESSAGE: String = "There is no entity with a containing relation to the group (orphaned).  You might search for it" +
                                       " (by adding it as an attribute to some entity)," +
                                       " & see if it should be deleted, kept with an entity, or left out there floating." +
                                       "  (While this is not an expected usage, it is allowed and does not imply data corruption.)"

  val unselectMoveTargetPromptText: String = "Unselect current move target (if present; not necessary really)"

  // This says 'same screenful' because it's easier to assume that the returned index refers to the currently available
  // local collections (a subset of all possible entries, for display), than calling chooseOrCreateObject, and sounds as useful:
  val unselectMoveTargetLeadingText: String = "CHOOSE AN ENTRY (that contains only one subgroup) FOR THE TARGET OF MOVES (choose from SAME SCREENFUL as " +
                                              "now;  if the target contains 0 subgroups, or 2 or more subgroups, " +
                                              "use other means to move entities to it until some kind of \"move anywhere\" feature is added):"

  val defaultPreferencesDepth = 10
  // Don't change these: they get set and looked up in the data for preferences. Changing it would just require users to reset it though, and would
  // leave the old as clutter in the data.
  val USER_PREFERENCES = "User preferences"
  final val SHOW_PUBLIC_PRIVATE_STATUS_PREFERENCE = "Should entity lists show public/private status for each?"
  final val DEFAULT_ENTITY_PREFERENCE = "Which entity should be displayed as default, when starting the program?"
  // (If change next line, also change the hard-coded use in the file first.exp.)
  val HEADER_CONTENT_TAG = "htmlHeaderContent"
  val BODY_CONTENT_TAG = "htmlInitialBodyContent"
  val FOOTER_CONTENT_TAG = "htmlFooterContent"

  val LOCAL_OM_INSTANCE_DEFAULT_DESCRIPTION = "(local: not for self-connection but to serve id to remotes)"

  def getClipboardContent: String = {
    val clipboard: java.awt.datatransfer.Clipboard = java.awt.Toolkit.getDefaultToolkit.getSystemClipboard
    val contents: String = clipboard.getContents(null).getTransferData(java.awt.datatransfer.DataFlavor.stringFlavor).toString
    contents.trim
    //(example of placing data on the clipboard, for future reference:)
    //val selection = new java.awt.datatransfer.StringSelection("someString")
    //clipboard.setContents(selection, null)
  }

  def isWindows: Boolean = {
    val osName = System.getProperty("os.name").toLowerCase
    osName.contains("win")
  }

  // Used for example after one has been deleted, to put the highlight on right next one:
  // idea: This feels overcomplicated.  Make it better?  Fixing bad smells in general (large classes etc etc) is on the task list.
  /**
   * @param objectSetSize # of all the possible entries, not reduced by what fits in the available display space (I think).
   * @param objectsToDisplayIn  Only those that have been chosen to display (ie, smaller list to fit in display size size) (I think).
   * @return
   */
  def findEntityToHighlightNext(objectSetSize: Int, objectsToDisplayIn: java.util.ArrayList[Entity], removedOneIn: Boolean,
                                previouslyHighlightedIndexInObjListIn: Int, previouslyHighlightedEntryIn: Entity): Option[Entity] = {
    //NOTE: SIMILAR TO findAttributeToHighlightNext: WHEN MAINTAINING ONE, DO SIMILARLY ON THE OTHER, until they are merged maybe by using the scala type
    //system better.

    // here of course, previouslyHighlightedIndexInObjListIn and objIds.size were calculated prior to the deletion.
    if (removedOneIn) {
      val newObjListSize = objectSetSize - 1
      val newIndexToHighlight = math.min(newObjListSize - 1, previouslyHighlightedIndexInObjListIn)
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
  def findAttributeToHighlightNext(objectSetSize: Int, objectsToDisplayIn: java.util.ArrayList[Attribute], removedOne: Boolean,
                                   previouslyHighlightedIndexInObjListIn: Int, previouslyHighlightedEntryIn: Attribute): Option[Attribute] = {
    //NOTE: SIMILAR TO findEntityToHighlightNext: WHEN MAINTAINING ONE, DO SIMILARLY ON THE OTHER, until they are merged maybe by using the scala type
    //system better.
    if (removedOne) {
      val newObjListSize = objectSetSize - 1
      val newIndexToHighlight = math.min(newObjListSize - 1, previouslyHighlightedIndexInObjListIn)
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

  def getDefaultUserInfo: (String, String) = {
    (System.getProperty("user.name"), "x")
  }

  // ****** MAKE SURE THE NEXT 2 LINES MATCH THE FORMAT of Controller.DATEFORMAT, AND THE USER EXAMPLES IN THIS CLASS' OUTPUT! ******
  // Making this a var so that it can be changed for testing consistency (to use GMT for most tests so hopefully they will pass for developers in
  // another time zone.  idea:  It seems like there's a better way to solve that though, maybe with a subclass of Controller in the test,
  // or of SimpleDateFormat.)
  var timezone: String = new java.text.SimpleDateFormat("zzz").format(System.currentTimeMillis())
  // (This isn't intended to match the date represented by a long value of "0", but is intended to be a usable value to fill in the rest of whatever a user
  // doesn't.  Perhaps assuming that the user will always put in a year if they put in anything (as currently enforced by the code at this time of writing).
  def blankDate = "1970-01-01 00:00:00:000 " + timezone

  val mRelTypeExamples = "i.e., ownership of or \"has\" another entity, family tie, &c"

  // (the startup message already suggests that they create it with their own name, no need to repeat that here:    )
  val menuText_createEntityOrAttrType: String = "Add new entity (or new type like length, for use with quantity, true/false, date, text, or file attributes)"
  val menuText_createRelationType: String = "Add new relation type (" + mRelTypeExamples + ")"
  val mainSearchPrompt = "Search all / list existing entities (except quantity units, attr types, & relation types)"
  val menuText_viewPreferences: String = "View preferences"


  // date stuff
  val VALID = "valid"
  val OBSERVED = "observed"
  val genericDatePrompt: String = "Please enter the date like this, w/ at least the year, and other parts as desired: \"2013-01-31 23:59:59:999 MDT\"; zeros are " +
                                  "allowed in all but the yyyy-mm-dd)." +
                                  //THIS LINE CAN BE PUT BACK AFTER the bug is fixed so ESC really works.  See similar cmt elsewhere; tracked in tasks:
                                  //"  Or ESC to exit.  " +
                                  "\"BC\" or \"AD\" prefix allowed (before the year, with no space)."
  val tooLongMessage = "value too long for type"

  def entityMenuLeadingText(entityIn: Entity) = {
    "**CURRENT ENTITY " + entityIn.getId + ": " + entityIn.getDisplayString(withColor = true)
  }

  def groupMenuLeadingText(groupIn: Group) = {
    "**CURRENT GROUP " + groupIn.getId + ": " + groupIn.getDisplayString()
  }

  val quantityTypePrompt: String = "SELECT TYPE OF QUANTITY (type is like length or volume, but not the measurement unit); ESC or leave both blank to cancel; " +
                                   "cancel if you need to create the needed type before selecting): "
  val textDescription: String = "TEXT (ex., serial #)"

  def canEditAttributeOnSingleLine(attributeIn: Attribute): Boolean = {
    ! attributeIn.isInstanceOf[FileAttribute]
  }

  def getUsableFilename(originalFilePathIn: String): (String, String) = FileAttribute.getUsableFilename(originalFilePathIn)

  val entityPartsThatCanBeAffected: String = "ALL its attributes, actions, and relations, but not entities or groups the relations refer to"

  val listNextItemsPrompt = "List next items"
  val listPrevItemsPrompt = "List previous items"
  val relationToGroupNamePrompt = "Type a name for this group (ex., \"xyz list\"), then press Enter; blank or ESC to cancel"

  def addRemainingCountToPrompt(choicesIn: Array[String], numDisplayedObjects: Long, totalRowsAvailableIn: Long,
                                startingDisplayRowIndexIn: Long): Array[String] = {
    val numLeft = totalRowsAvailableIn - startingDisplayRowIndexIn - numDisplayedObjects
    val indexOfPrompt = choicesIn.indexOf(listNextItemsPrompt)
    if (numLeft > 0 && indexOfPrompt >= 0) {
      choicesIn(indexOfPrompt) = listNextItemsPrompt + " (of " + numLeft + " more)"
    }
    choicesIn
  }

  def getContainingEntitiesDescription(entityCountNonArchivedIn: Long, entityCountArchivedIn: Long): String = {
    "contained in " + entityCountNonArchivedIn + " entities, and in " + entityCountArchivedIn + " archived entities"
  }

  val pickFromListPrompt: String = "Pick from menu, or an item by letter to select; Alt+<letter> to go to the item then come back here"

  def searchPromptPart(typeIn: String): String = "Enter part of the " + typeIn + " name to search for."

  def searchPrompt(typeNameIn: String): String = {
    searchPromptPart(typeNameIn) + "  (For the curious: it will be used in matching as a " +
    "case-insensitive POSIX " +
    "regex; details at  http://www.postgresql.org/docs/9.1/static/functions-matching.html#FUNCTIONS-POSIX-REGEXP .)"
  }

  def isNumeric(input: String): Boolean = {
    // simplicity over performance in this case:
    try {
      // throws an exception if not numeric:
      input.toFloat
      true
    } catch {
      case e: NumberFormatException => false
    }
  }

  def inputFileValid(path: String): Boolean = {
    val file = new java.io.File(path)
    file.exists && file.canRead
  }

  // The check to see if a long date string is valid comes later.
  // Now that we allow 1-digit dates, there is nothing to ck really.
  def validOnDateCriteria(dateStr: String): Boolean = true
  // Same comments as for observedDateCriteria:
  def observedDateCriteria(dateStr: String): Boolean = true

  def throwableToString(e: Throwable): String = {
    val stringWriter = new StringWriter()
    e.printStackTrace(new PrintWriter(stringWriter))
    stringWriter.toString
  }

  def handleException(e: Throwable, ui: TextUI, db: Database) {
    if (e.isInstanceOf[org.postgresql.util.PSQLException] || e.isInstanceOf[OmDatabaseException] ||
        throwableToString(e).contains("ERROR: current transaction is aborted, commands ignored until end of transaction block"))
    {
      db.rollbackTrans()
    }
    // If changing this string (" - 1"), also change in first.exp that looks for it (distinguished from " - 2" elsewhere).
    val ans = ui.askYesNoQuestion("An error occurred: \"" + e.getClass.getName + ": " + e.getMessage + "\".  If you can provide simple instructions to " +
                                  "reproduce it consistently, maybe it can be fixed - 1.  Do you want to see the detailed output?")
    if (ans.isDefined && ans.get) {
      ui.displayText(throwableToString(e))
    }
  }

  /** A helper method.  Returns the date as a Long (java-style: ms since 1970 began), and true if there is a problem w/ the string and we need to ask again. */
  def finishAndParseTheDate(dateStrIn: String, blankMeansNOW: Boolean = true, ui: TextUI): (Option[Long], Boolean) = {
    //to start with, the special forms (be sure to trim the input, otherwise there's no way in the textui to convert from a previously entered (so default)
    //value to "blank/all time"!).
    val dateStrWithOptionalEra =
      if (dateStrIn.equalsIgnoreCase("now") || (blankMeansNOW && dateStrIn.trim.length() == 0)) {
        val currentDateString: String = Util.DATEFORMAT.format(new java.util.Date(System.currentTimeMillis()))
        currentDateString
      }
      else dateStrIn.trim

    // chop off the era before doing some of the other logic
    val (era: String, dateStr) =
      if (dateStrWithOptionalEra.toUpperCase.startsWith("AD") || dateStrWithOptionalEra.toUpperCase.startsWith("BC")) {
        (dateStrWithOptionalEra.substring(0, 2), dateStrWithOptionalEra.substring(2))
      } else ("", dateStrWithOptionalEra)

    // help user if they put in something like 2013-1-1 instead of 2013-01-01, so the parsed date isn't messed up. See test.
    // (The year could be other than 4 digits, so check for the actual location of the 1st hyphen):
    val firstHyphenPosition = if (dateStr.indexOf('-') != -1) dateStr.indexOf('-') else dateStr.length
    //but only if the string format looks somewhat expected; otherwise let later parsing handle it all.
    val filledInDateStr =
      if (dateStr.length > firstHyphenPosition + 1 && dateStr.length < firstHyphenPosition + 6
          && dateStr.indexOf('-') == firstHyphenPosition && dateStr.indexOf('-', firstHyphenPosition + 1) >= 0) {
        val secondHyphenPosition = dateStr.indexOf('-', firstHyphenPosition + 1)
        if (secondHyphenPosition == firstHyphenPosition + 2 || secondHyphenPosition == firstHyphenPosition + 3) {
          if (dateStr.length == secondHyphenPosition + 2 || dateStr.length == secondHyphenPosition + 3) {
            val year = dateStr.substring(0, firstHyphenPosition)
            val mo = dateStr.substring(firstHyphenPosition + 1, secondHyphenPosition)
            val dy = dateStr.substring(secondHyphenPosition + 1)
            year + '-' + (if (mo.length == 1) "0" + mo else mo) + '-' + (if (dy.length == 1) "0" + dy else dy)
          }
          else dateStr
        }
        else dateStr
      } else if (dateStr.length == firstHyphenPosition + 2) {
        // also handle format like 2013-1
        val year = dateStr.substring(0, firstHyphenPosition)
        val mo = dateStr.substring(firstHyphenPosition + 1)
        year + '-' + "0" + mo
      }
      else dateStr


    // Fill in the date w/ "blank" information for whatever detail the user didn't provide:
    val filledInDateStrWithoutYear = if (firstHyphenPosition < filledInDateStr.length) filledInDateStr.substring(firstHyphenPosition + 1) else ""
    val year = filledInDateStr.substring(0, firstHyphenPosition)

    val blankDateWithoutYear = blankDate.substring(5)

    val dateStrWithZeros =
      if (filledInDateStrWithoutYear.length() < blankDateWithoutYear.length) {
        year + '-' + filledInDateStrWithoutYear + blankDateWithoutYear.substring(filledInDateStrWithoutYear.length())
      }
      else filledInDateStr
    // then parse it:
    try {
      val d: java.util.Date =
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
        ui.displayText("Invalid date format. Try something like \"2003\", or \"2003-01-31\", or \"2003-01-31 22:15\" for 10:15pm, or if you need a timezone, " +
                       "all of \"yyyy-MM-dd HH:mm:ss:SSS zzz\", like for just before midnight: \"2013-01-31 //23:59:59:999 MST\".")
        (None, true)
    }
  }

  /** Returns (validOnDate, observationDate, userWantsToCancel) */
  def askForAttributeValidAndObservedDates(inEditing: Boolean,
                                           oldValidOnDateIn: Option[Long],
                                           oldObservedDateIn: Long,
                                           ui: TextUI): (Option[Long], Long, Boolean) = {
    //idea: make this more generic, passing in prompt strings &c, so it's more cleanly useful for DateAttribute instances. Or not: lacks shared code.
    //idea: separate these into 2 methods, 1 for each time (not much common material of significance).
    // BETTER IDEA: fix the date stuff in the DB first as noted in tasks, so that this part makes more sense (the 0 for all time, etc), and then
    // when at it, recombine the askForDate_Generic method w/ these or so it's all cleaned up.
    /** Helper method made so it can be recursive, it returns the date (w/ meanings as with displayText below, and as in PostgreSQLDatabase.createTables),
      * and true if the user wants to cancel/get out). */
    //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) w/in this method!
    @tailrec def askForDate(dateTypeIn: String, acceptanceCriteriaIn: (String) => Boolean): (Option[Long], Boolean) = {
      val leadingText: Array[String] = {
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
          //    "But that would have to be kept in mind if doing any relative date calculations in the program, e.g. # of years, spanning 0.)" + TextUI.NEWLN +
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

      val defaultValue: Option[String] = {
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

      val ans = ui.askForString(Some(leadingText), None, defaultValue)

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
        val dateStr = ans.get.trim
        if (dateTypeIn == VALID && dateStr.trim.length == 0) (None, false)
        else if (dateTypeIn == VALID && dateStr.trim == "0") (Some(0), false)
        else if (!acceptanceCriteriaIn(dateStr)) askForDate(dateTypeIn, acceptanceCriteriaIn)
        else {
          // (special values like "0" or blank are already handled above)
          val (newDate: Option[Long], retry: Boolean) = finishAndParseTheDate(dateStr, dateTypeIn == OBSERVED, ui)
          if (retry) askForDate(dateTypeIn, acceptanceCriteriaIn)
          else {
            (newDate, false)
          }
        }
      }
    }

    // the real action:
    def askForBothDates(ui: TextUI): (Option[Long], Long, Boolean) = {
      val (validOnDate, userCancelled) = askForDate(VALID, validOnDateCriteria)
      if (userCancelled) (None, 0, userCancelled)
      else {
        val (observedDate, userCancelled) = askForDate(OBSERVED, observedDateCriteria)
        if (userCancelled) (Some(0), 0, userCancelled)
        else {
          // (for why validOnDate is sometimes allowed to be None, but not observedDate: see val validOnPrompt.)
          require(observedDate.isDefined)
          val ans = ui.askYesNoQuestion("Dates are: " + AttributeWithValidAndObservedDates.getDatesDescription(validOnDate,
                                                                                                               observedDate.get) + ": right?", Some("y"))
          if (ans.isDefined && ans.get) (validOnDate, observedDate.get, userCancelled)
          else askForBothDates(ui)
        }
      }
    }
    askForBothDates(ui)
  }

  /** Cloned from controller.askForDate; see its comments in the code.
    * Idea: consider combining somehow with method askForDateAttributeValue.
    * @return None if user wants out.
    */
  //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) w/in this method!
  @tailrec final def askForDate_generic(promptTextIn: Option[String] = None, defaultIn: Option[String], ui: TextUI): Option[Long] = {
    val leadingText: Array[String] = Array(promptTextIn.getOrElse(genericDatePrompt))
    val default: String = defaultIn.getOrElse(Util.DATEFORMAT.format(System.currentTimeMillis()))
    val ans = ui.askForString(Some(leadingText), None, Some(default))
    if (ans.isEmpty) None
    else {
      val dateStr = ans.get.trim
      val (newDate: Option[Long], retry: Boolean) = finishAndParseTheDate(dateStr, ui = ui)
      if (retry) askForDate_generic(promptTextIn, defaultIn, ui)
      else newDate
    }
  }

  def copyright(ui: TextUI): String = {
    var all = ""
    try {
      val reader: BufferedReader = {
        // first try to get it from the jar being run by the user:
        val stream = this.getClass.getClassLoader.getResourceAsStream("LICENSE")
        if (stream != null) {
          new BufferedReader(new java.io.InputStreamReader(stream))
        } else {
          // failing that, check the filesystem, i.e., checking how it looks during development when the jar isn't built (or at least for consistent behavior
          // during development)
          new BufferedReader(scala.io.Source.fromFile("LICENSE").reader())
        }
      }
      var append = false
      var beforeAnyDashes = true
      // idea: do this in a most scala-like way, like w/ immutable "line", recursion instead of a while loop, and can its libraries simplify this?:
      var line: String = reader.readLine()
      while (line != null) {
        if ((!append) && line.startsWith("-----") && beforeAnyDashes) {
          append = true
          beforeAnyDashes = false
        } else if (append && line.contains("(see below). If not, see")) {
          all = all + line.replace("(see below). If not, see", "(see the file LICENSE). If not, see") + TextUI.NEWLN
          append = false
        } else if (append)
          all = all + line + TextUI.NEWLN
        else if (!append) {
          // do nothing
        }
        line = reader.readLine()
      }
    }
    catch {
      case e: Exception =>
        val ans = ui.askYesNoQuestion(TextUI.NEWLN + TextUI.NEWLN + "The file LICENSE is missing from the distribution of this program or for " +
                                      "some other reason can't be displayed normally; please let us know to " +
                                      " correct that, and please be aware of the license.  You can go to this URL to see it:" + TextUI.NEWLN +
                                      "    http://onemodel.org/download/OM-LICENSE " + TextUI.NEWLN +
                                      ".  (Do you want to see the detailed error output?)")
        if (ans.isDefined && ans.get) {
          // (" - 2" is to distinguished from " - 1" because some code looks for " - 1".)
          ui.displayText("The error was: \"" + e.getClass.getName + ": " + e.getMessage + "\".  If you can provide simple instructions to " +
                         "reproduce it consistently, maybe it can be fixed - 2.  " + Util.throwableToString(e))
        }
    }
    all
  }

  def stringTooLongErrorMessage(nameLength: Int): String = {
    // for details, see method PostgreSQLDatabase.escapeQuotesEtc.
    "Got an error.  Please try a shorter (" + nameLength + " chars) entry.  " +
    "(Could be due to escaped, i.e. expanded, characters like ' or \";\".  Details: %s"
  }

  def isDuplicationAProblem(isDuplicateIn: Boolean, duplicateNameProbablyOK: Boolean, ui: TextUI): Boolean = {
    var duplicateProblemSoSkip = false
    if (isDuplicateIn) {
      if (!duplicateNameProbablyOK) {
        val answerOpt = ui.askForString(Some(Array("That name is a duplicate--proceed anyway? (y/n)")), None, Some("n"))
        if (answerOpt.isEmpty || (!answerOpt.get.equalsIgnoreCase("y"))) duplicateProblemSoSkip = true
      }
    }
    duplicateProblemSoSkip
  }

  //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) within this method, below!
  @tailrec final def askForNameInReverseDirection(directionalityStrIn: String, nameLengthIn: Int, nameIn: String,
                                                  previousNameInReverseIn: Option[String] = None, ui: TextUI): String = {
    // see createTables (or UI prompts) for meanings of bi/uni/non...
    if (directionalityStrIn == "UNI") ""
    else if (directionalityStrIn == "NON") nameIn
    else if (directionalityStrIn == "BI") {
      // see createTables (or UI prompts) for meanings...
      val msg = Array("Enter relation name when direction is reversed (i.e., 'is husband to' becomes 'is wife to', 'employs' becomes 'is employed by' " +
                      "by; up to " + nameLengthIn + " characters (ESC to cancel): ")
      val nameInReverse = {
        val ans: Option[String] = ui.askForString(Some(msg), None, previousNameInReverseIn)
        if (ans.isEmpty) return ""
        ans.get.trim() //see above comment about trim
      }
      val ans = ui.askWhich(Some(Array("Is this the correct name for the relationship in reverse direction?: ")), Array("Yes", "No"))
      if (ans.isEmpty || ans.get == 2) askForNameInReverseDirection(directionalityStrIn, nameLengthIn, nameIn, previousNameInReverseIn, ui)
      else nameInReverse
    }
    else throw new Exception("unexpected value for directionality: " + directionalityStrIn)
  }

  def askForRelationDirectionality(previousDirectionalityIn: Option[String] = None, ui: TextUI): Option[String] = {
    val msg = Array("Enter directionality (\"bi\", \"uni\", or \"non\"; examples: \"is parent of\"/\"is child of\" is bidirectional, " +
                    "since it differs substantially by the direction but goes both ways; unidirectional might be like 'lists': the thing listed doesn't know " +
                    "it; \"is acquaintanted with\" could be nondirectional if it is an identical relationship either way  (ESC to cancel): ")
    def criteria(entryIn: String): Boolean = {
      val entry = entryIn.trim().toUpperCase
      entry == "BI" || entry == "UNI" || entry == "NON"
    }

    val directionality = ui.askForString(Some(msg), Some(criteria(_: String)), previousDirectionalityIn)
    if (directionality.isEmpty) None
    else Some(directionality.get.toUpperCase)
  }

  def editMultilineText(input: String, ui: TextUI): String = {
    //idea: allow user to change the edit command setting (ie which editor to use) from here?

    //idea: allow user to prevent this message in future. Could be by using ui.askYesNoQuestion instead, adding to the  prompt "(ask this again?)", with
    // 'y' as default, and storing the answer in the db.systemEntityName somewhere perhaps.
    //PUT THIS BACK (& review/test it) after taking the time to read the Process package's classes or something like
    // apache commons has, and learn to launch vi workably, from scala. And will the terminal settings changes by OM have to be undone/redone for it?:
    //        val command: String = db.getTextEditorCommand
    //        ui.displayText("Using " + command + " as the text editor, but you can change that by navigating to the Main OM menu with ESC, search for
    // existing " +
    //                       "entities, choose the first one (called " + PostgreSQLDatabase.systemEntityName + "), choose " +
    //                       PostgreSQLDatabase.EDITOR_INFO_ENTITY_NAME + ", choose " +
    //                       "" + PostgreSQLDatabase.TEXT_EDITOR_INFO_ENTITY_NAME + ", then choose the " +
    //                       PostgreSQLDatabase.TEXT_EDITOR_COMMAND_ATTRIBUTE_TYPE_NAME + " and edit it with option 3.")

    val path: Path = Files.createTempFile("om-edit-", ".txt")
    Files.write(path, input.getBytes)
    ui.displayText("Until we improve this, you can now go edit the content in this temporary file, & save it:\n" +
                   path.toFile.getCanonicalPath + "\n...then come back here when ready to import that text.")
    val newContent: String = new Predef.String(Files.readAllBytes(path))
    path.toFile.delete()
    newContent
  }

  /** Returns None if user just wants out. */
  def promptWhetherTo1Add2Correct(inAttrTypeDesc: String, ui: TextUI): Option[Int] = {
    //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) within this method, below!
    @tailrec def ask: Option[Int] = {
      val ans = ui.askWhich(None, Array("1-Save this " + inAttrTypeDesc + " attribute?", "2-Correct it?"))
      if (ans.isEmpty) return None
      val answer = ans.get
      if (answer < 1 || answer > 2) {
        ui.displayText("invalid response")
        ask
      } else Some(answer)
    }
    ask
  }

  def askForQuantityAttributeNumber(previousQuantity: Float, ui: TextUI): Option[Float] = {
    val leadingText = Array[String]("ENTER THE NUMBER FOR THE QUANTITY (i.e., 5, for 5 centimeters length)")
    val ans = ui.askForString(Some(leadingText), Some(Util.isNumeric), Some(previousQuantity.toString))
    if (ans.isEmpty) None
    else Some(ans.get.toFloat)
  }

  /** Returns None if user wants to cancel. */
  def askForTextAttributeText(ignore: Database, inDH: TextAttributeDataHolder, inEditing: Boolean, ui: TextUI): Option[TextAttributeDataHolder] = {
    val outDH = inDH.asInstanceOf[TextAttributeDataHolder]
    val defaultValue: Option[String] = if (inEditing) Some(inDH.text) else None
    val ans = ui.askForString(Some(Array("Type or paste a single-line attribute value, then press Enter; ESC to cancel." +
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
  def askForDateAttributeValue(ignore: Database, inDH: DateAttributeDataHolder, inEditing: Boolean, ui: TextUI): Option[DateAttributeDataHolder] = {
    val outDH = inDH.asInstanceOf[DateAttributeDataHolder]

    // make the DateFormat omit trailing zeros, for editing convenience (to not have to backspace thru the irrelevant parts if not specified):
    var dateFormatString = "yyyy-MM-dd"
    val milliseconds: String = new java.text.SimpleDateFormat("SSS").format(new java.util.Date(inDH.date))
    val seconds: String = new java.text.SimpleDateFormat("ss").format(new java.util.Date(inDH.date))
    val minutes: String = new java.text.SimpleDateFormat("mm").format(new java.util.Date(inDH.date))
    val hours: String = new java.text.SimpleDateFormat("HH").format(new java.util.Date(inDH.date))
    if (milliseconds != "000") {
      dateFormatString = dateFormatString + " HH:mm:ss:SSS zzz"
    } else if (seconds != "00") {
      dateFormatString = dateFormatString + " HH:mm:ss zzz"
    } else if (minutes != "00" || hours != "00") {
      dateFormatString = dateFormatString + " HH:mm zzz"
    }
    val dateFormat = new java.text.SimpleDateFormat(dateFormatString)
    val defaultValue: String = {
      if (inEditing) dateFormat.format(new Date(inDH.date))
      else Util.DATEFORMAT.format(System.currentTimeMillis())
    }

    def dateCriteria(date: String): Boolean = {
      !Util.finishAndParseTheDate(date, ui = ui)._2
    }
    val ans = ui.askForString(Some(Array(Util.genericDatePrompt)), Some(dateCriteria), Some(defaultValue))
    if (ans.isEmpty) None
    else {
      val (newDate: Option[Long], retry: Boolean) = Util.finishAndParseTheDate(ans.get, ui = ui)
      if (retry) throw new Exception("Programmer error: date indicated it was parseable, but the same function said afterward it couldn't be parsed.  Why?")
      else if (newDate.isEmpty) throw new Exception("There is a bug: the program shouldn't have got to this point.")
      else {
        outDH.date = newDate.get
        Some(outDH)
      }
    }
  }

  /** Returns None if user wants to cancel. */
  def askForBooleanAttributeValue(ignore: Database, inDH: BooleanAttributeDataHolder, inEditing: Boolean, ui: TextUI): Option[BooleanAttributeDataHolder] = {
    val outDH = inDH.asInstanceOf[BooleanAttributeDataHolder]
    val ans = ui.askYesNoQuestion("Set the new value to true now? ('y' if so, 'n' for false)", if (inEditing && inDH.boolean) Some("y") else Some("n"))
    if (ans.isEmpty) None
    else {
      outDH.boolean = ans.get
      Some(outDH)
    }
  }

  /** Returns None if user wants to cancel. */
  def askForFileAttributeInfo(ignore: Database, inDH: FileAttributeDataHolder, inEditing: Boolean, ui: TextUI): Option[FileAttributeDataHolder] = {
    val outDH = inDH.asInstanceOf[FileAttributeDataHolder]
    var path: Option[String] = None
    if (!inEditing) {
      // we don't want the original path to be editable after the fact, because that's a historical observation and there is no sense in changing it.
      path = ui.askForString(Some(Array("Enter file path (must exist and be readable), then press Enter; ESC to cancel")), Some(Util.inputFileValid))
    }
    if (!inEditing && path.isEmpty) None
    else {
      // if we can't fill in the path variables by now, there is a bug:
      if (!inEditing) outDH.originalFilePath = path.get
      else path = Some(outDH.originalFilePath)

      val defaultValue: Option[String] = if (inEditing) Some(inDH.description) else Some(FilenameUtils.getBaseName(path.get))
      val ans = ui.askForString(Some(Array("Type file description, then press Enter; ESC to cancel")), None, defaultValue)
      if (ans.isEmpty) None
      else {
        outDH.description = ans.get
        Some(outDH)
      }
    }
  }

  /** Returns None if user just wants out; a String (user's answer, not useful outside this method) if update was done..
    */
  def editGroupName(groupIn: Group, ui: TextUI): Option[String] = {
    // doesn't seem to make sense to ck for duplicate names here: the real identity depends on what it relates to, and dup names may be common.
    val ans = ui.askForString(Some(Array(Util.relationToGroupNamePrompt)), None, Some(groupIn.getName))
    if (ans.isEmpty || ans.get.trim.length() == 0) {
      None
    } else {
      groupIn.update(None, Some(ans.get.trim), None, None, None, None)
      ans
    }
  }

}
