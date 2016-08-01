/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003-2004 and 2008-2016 inclusive, Luke A. Call; all rights reserved.
    (That copyright statement was previously 2013-2015, until I remembered that much of Controller came from TextUI.scala, and TextUI.java before that.)
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule, guidelines around binary
    distribution, and the GNU Affero General Public License as published by the Free Software Foundation, either version 3
    of the License, or (at your option) any later version.  See the file LICENSE for details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
package org.onemodel.controller

import java.io._
import java.nio.file.{Files, Path}
import java.util
import java.util.Date

import org.apache.commons.io.FilenameUtils
import org.onemodel._
import org.onemodel.database.PostgreSQLDatabase
import org.onemodel.model._
import org.postgresql.util.PSQLException

import scala.annotation.tailrec
import scala.collection.mutable.ArrayBuffer

object Controller {
  // should these be more consistently upper-case? What is the scala style for constants?  similarly in other classes.
  def maxNameLength: Int = math.max(math.max(PostgreSQLDatabase.entityNameLength, PostgreSQLDatabase.relationTypeNameLength),
                                    PostgreSQLDatabase.classNameLength)

  // Might not be the most familiar date form for us Americans, but it seems the most useful in the widest
  // variety of situations, and more readable than with the "T" embedded in place of
  // the 1st space.  So, this approximates iso-9601.
  // these are for input.
  val DATEFORMAT = new java.text.SimpleDateFormat("yyyy-MM-dd HH:mm:ss:SSS zzz")
  val DATEFORMAT2 = new java.text.SimpleDateFormat("yyyy-MM-dd HH:mm:ss zzz")
  val DATEFORMAT3 = new java.text.SimpleDateFormat("yyyy-MM-dd HH:mm zzz")
  val DATEFORMAT_WITH_ERA = new java.text.SimpleDateFormat("GGyyyy-MM-dd HH:mm:ss:SSS zzz")
  val DATEFORMAT2_WITH_ERA = new java.text.SimpleDateFormat("GGyyyy-MM-dd HH:mm:ss zzz")
  val DATEFORMAT3_WITH_ERA = new java.text.SimpleDateFormat("GGyyyy-MM-dd HH:mm zzz")

  //these are here to avoid colliding with use of the same names within other code inside the class.
  // idea: see what scala does with enums and/or constants; update this style?
  val ENTITY_TYPE: String = "entity"
  val QUANTITY_TYPE: String = "quantity"
  val TEXT_TYPE: String = "text"
  val DATE_TYPE: String = "date"
  val BOOLEAN_TYPE: String = "boolean"
  val FILE_TYPE: String = "file"
  //i.e., "relationTypeType", or the thing that we sometimes put in an attribute type parameter, though not exactly an attribute type, which is "RelationType":
  val RELATION_TYPE_TYPE: String = "relationtype"
  val RELATION_TO_ENTITY_TYPE: String = "relation to entity"
  val RELATION_TO_GROUP_TYPE: String = "relation to group"
  val GROUP_TYPE: String = "group"
  val ENTITY_CLASS_TYPE: String = "class"

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

  val HEADER_CONTENT_TAG = "htmlHeaderContent"
  val BODY_CONTENT_TAG = "htmlInitialBodyContent"
  val FOOTER_CONTENT_TAG = "htmlFooterContent"

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

}

/** Improvements to this class should START WITH MAKING IT BETTER TESTED (functional testing? integration? see
  * scalatest docs 4 ideas, & maybe use expect?), delaying side effects more, shorter methods, other better scala style, etc.
  *
  Don't ever instantiate a controller from a *test* without passing in username/password parameters, because it will try to log in to the user's default
  database and run the tests there (ie, they could be destructive):
  */
class Controller(val ui: TextUI, forceUserPassPromptIn: Boolean = false, defaultUsernameIn: Option[String] = None, defaultPasswordIn: Option[String] = None) {
  // ****** MAKE SURE THE NEXT 2 LINES MATCH THE FORMAT of Controller.DATEFORMAT, AND THE USER EXAMPLES IN THIS CLASS' OUTPUT! ******
  // Making this a var so that it can be changed for testing consistency (to use GMT for most tests so hopefully they will pass for developers in
  // another time zone.  idea:  It seems like there's a better way to solve that though, maybe with a subclass of Controller in the test,
  // or of SimpleDateFormat.)
  var timezone: String = new java.text.SimpleDateFormat("zzz").format(System.currentTimeMillis())
  // (This isn't intended to match the date represented by a long value of "0", but is intended to be a usable value to fill in the rest of whatever a user
  // doesn't.  Perhaps assuming that the user will always put in a year if they put in anything (as currently enforced by the code at this time of writing).
  def blankDate = "1970-01-01 00:00:00:000 " + timezone

  val mRelTypeExamples = "i.e., ownership of or \"has\" another entity, family tie, &c"

  //idea: get more scala familiarity then change this so it has limited visibility/scope: like, protected (subclass instances) + ImportExportTest.
  val db: PostgreSQLDatabase = tryLogins(forceUserPassPromptIn, defaultUsernameIn, defaultPasswordIn)

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
    "**CURRENT ENTITY " + entityIn.getId + ": " + entityIn.getDisplayString
  }

  val mCopyright: String = {
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
          ui.displayText("The error was: \"" + e.getClass.getName + ": " + e.getMessage + "\".  If you can provide simple instructions to " +
                         "reproduce it consistently, maybe it can be fixed.  " + throwableToString(e))
        }
    }
    all
  }

  /** Returns the id and the entity, if they are available from the preferences lookup (id) and then finding that in the db (Entity). */
  def getDefaultEntity: (Option[Long], Option[Entity]) = {
    if (defaultDisplayEntityId.isEmpty || ! db.entityKeyExists(defaultDisplayEntityId.get)) {
      (None, None)
    } else (defaultDisplayEntityId, Entity.getEntityById(db, defaultDisplayEntityId.get))
  }

  def start() {
    // idea: wait for keystroke so they do see the copyright each time. (is also tracked):  make it save their answer 'yes/i agree' or such in the DB,
    // and don't make them press the keystroke again (timesaver)!  See code at top of PostgreSQLDatabase that puts things in the db at startup: do similarly?
    ui.displayText(mCopyright, waitForKeystroke = true, Some("IF YOU DO NOT AGREE TO THOSE TERMS: " + ui.howQuit + " to exit.\n" +
                                                             "If you agree to those terms: "))
    // Max id used as default here because it seems the least likely # to be used in the system hence the
    // most likely to cause an error as default by being missing, so the system can respond by prompting
    // the user in some other way for a use.
    if (getDefaultEntity._1.isEmpty) {
      ui.displayText("To get started, you probably want to find or create an " +
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
      //re-checking for the default each time because user can change it.
      new MainMenu(ui, db, this).mainMenu(getDefaultEntity._2, goDirectlyToChoice)
      menuLoop()
    }
    menuLoop(Some(5))
  }

  /** If the 1st parm is true, the next 2 must be omitted or None. */
  private def tryLogins(forceUserPassPromptIn: Boolean = false, defaultUsernameIn: Option[String] = None,
                        defaultPasswordIn: Option[String] = None): PostgreSQLDatabase = {

    require(if (forceUserPassPromptIn) defaultUsernameIn.isEmpty && defaultPasswordIn.isEmpty else true)

    // tries the system username, blank password, & if that doesn't work, prompts user.
    @tailrec def tryOtherLoginsOrPrompt(): PostgreSQLDatabase = {
      val db = {
        var pwdOpt: Option[String] = None
        // try logging in with some obtainable default values first, to save user the trouble, like if pwd is blank
        val systemUserName = System.getProperty("user.name")
        val defaultPassword = "x"
        val dbWithSystemNameBlankPwd = login(systemUserName, defaultPassword, showError = false)
        if (dbWithSystemNameBlankPwd.isDefined) dbWithSystemNameBlankPwd
        else {
          val usrOpt = ui.askForString(Some(Array("Username")), None, Some(systemUserName))
          if (usrOpt.isEmpty) System.exit(1)
          val dbConnectedWithBlankPwd = login(usrOpt.get, defaultPassword, showError = false)
          if (dbConnectedWithBlankPwd.isDefined) dbConnectedWithBlankPwd
          else {
            try {
              pwdOpt = ui.askForString(Some(Array("Password")), None, None, isPasswordIn = true)
              if (pwdOpt.isEmpty) System.exit(1)
              val dbWithUserEnteredPwd = login(usrOpt.get, pwdOpt.get, showError = true)
              dbWithUserEnteredPwd
            } finally {
              if (pwdOpt.isDefined) {
                pwdOpt = null
                //garbage collect to keep the memory cleared of passwords. What's a better way? (gc isn't forced to do it all every time IIRC,
                // so poke it--a guess)
                System.gc()
                System.gc()
                System.gc()
              }
            }
          }
        }
      }
      if (db.isEmpty) {
        ui.displayText("Login failed; retrying (" + ui.howQuit + " to quit if needed):",
                       waitForKeystroke = false)
        tryOtherLoginsOrPrompt()
      }
      else db.get
    }

    if (forceUserPassPromptIn) {
      @tailrec def loopPrompting: PostgreSQLDatabase = {
        val usrOpt = ui.askForString(Some(Array("Username")))
        if (usrOpt.isEmpty) System.exit(1)

        val pwdOpt = ui.askForString(Some(Array("Password")), None, None, isPasswordIn = true)
        if (pwdOpt.isEmpty) System.exit(1)

        val dbWithUserEnteredPwd: Option[PostgreSQLDatabase] = login(usrOpt.get, pwdOpt.get, showError = false)
        if (dbWithUserEnteredPwd.isDefined) dbWithUserEnteredPwd.get
        else loopPrompting
      }
      loopPrompting
    } else if (defaultUsernameIn.isDefined && defaultPasswordIn.isDefined) {
      // idea: perhaps this could be enhanced and tested to allow a username parameter, but prompt for a password, if/when need exists.
      val db = login(defaultUsernameIn.get, defaultPasswordIn.get, showError = true)
      if (db.isEmpty) {
        ui.displayText("The program wasn't expected to get to this point in handling it (expected an exception to be thrown previously), " +
                       "but the login with provided credentials failed.")
        System.exit(1)
      }
      db.get
      // not attempting to clear that password variable because (making the parm a var got an err msg and) maybe that kind is less intended
      // to be secure (anyway)?
    } else tryOtherLoginsOrPrompt()
  }

  private def login(username: String, password: String, showError: Boolean): Option[PostgreSQLDatabase] = {
    try new Some(new PostgreSQLDatabase(username, new String(password)))
    catch {
      case ex: PSQLException =>
        // attempt didn't work, but don't throw exc if the program
        // is just trying defaults, for example:
        if (showError) throw ex
        else None
    }
  }

  // Idea: From showPublicPrivateStatusPreference, on down through findDefaultDisplayEntityId, feels awkward.  Needs something better, but I'm not sure
  // what, at the moment.  It was created this way as a sort of cache because looking it up every time was costly and made the app slow, like when
  // displaying a list of entities (getting the preference every time, to N levels deep), and especially at startup when checking for the default
  // up to N levels deep, among the preferences that can include entities with deep nesting.  So in a related change I made it also not look N levels
  // deep, for preferences.  If you check other places touched by this commit there may be a "shotgun surgery" bad smell here also.
  //Idea: Maybe these should have their cache expire after a period of time (to help when running multiple clients).
  var showPublicPrivateStatusPreference: Option[Boolean] = db.getUserPreference_Boolean(Controller.SHOW_PUBLIC_PRIVATE_STATUS_PREFERENCE)
  def refreshPublicPrivateStatusPreference(): Unit = showPublicPrivateStatusPreference = db.getUserPreference_Boolean(Controller.SHOW_PUBLIC_PRIVATE_STATUS_PREFERENCE)
  // putting this in a var instead of recalculating it every time (too frequent) inside findDefaultDisplayEntityId:
  var defaultDisplayEntityId: Option[Long] = db.getUserPreference_EntityId(Controller.DEFAULT_ENTITY_PREFERENCE)
  def refreshDefaultDisplayEntityId(): Unit = defaultDisplayEntityId = db.getUserPreference_EntityId(Controller.DEFAULT_ENTITY_PREFERENCE)

  def askForClass(): Option[Long] = {
    val idWrapper: Option[IdWrapper] = chooseOrCreateObject(Some(List[String]("CHOOSE ENTITY'S CLASS (ESC if you don't know or care about this.  This is a way to associate code or default attributes with groups of entities.  Help me word this better, but:  for example, in the case of an attribute of an entity, it must have a type, which is also an entity; so, a Vehicle Identification Number (VIN) entity represents the *concept* of a VIN, a text attribute on a vehicle entity holds the content of the VIN in a string of characters, and the \"type entity\" (VIN concept entity) can have a class which holds code used to parse VIN strings for additional internal meaning; or, it could serve as a template holding standard fields for entities in the VIN class (such as if the VIN content were written in a multi-field entity rather than in a single text attribute).  The class-defining entity of the VIN class could be the same entity as the type entity for the an attribute, as far as I can see now.  ***Use this feature only if it helps you, otherwise press ESC for None.*** )")), None, None, Controller.ENTITY_CLASS_TYPE)
    if (idWrapper.isEmpty) None
    else Some(idWrapper.get.getId)
  }

  /** In any given usage, consider whether askForNameAndWriteEntity should be used instead: it is for quick (simpler) creation situations or
    * to just edit the name when the entity already exists, or if the Entity is a RelationType,
    * askForClassInfoAndNameAndCreateEntity (this one) prompts for a class and checks whether it should copy default attributes from the class-defining-entity.
    * There is also editEntityName which calls askForNameAndWriteEntity: it checks if the Entity being edited is a RelationType, and if not also checks
    * for whether a group name should be changed at the same time.
    */
  def askForClassInfoAndNameAndCreateEntity(inClassId: Option[Long] = None): Option[Entity] = {
    var newClass = false
    val classId: Option[Long] =
      if (inClassId.isDefined) inClassId
      else {
        newClass = true
        askForClass()
      }
    val ans: Option[Entity] = askForNameAndWriteEntity(Controller.ENTITY_TYPE, None, None, None, None, classId,
                                                       Some(if (newClass) "DEFINE THE ENTITY:" else ""))
    if (ans.isDefined) {
      val entity = ans.get
      // idea: (is also on fix list): this needs to be removed, after evaluating for other side effects, to fix the bug
      // where creating a new relationship, and creating the entity2 in the process, it puts the wrong info
      // on the header for what is being displayed/edited next!: Needs refactoring anyway: this shouldn't be at
      // a low level.
      ui.displayText("Created " + Controller.ENTITY_TYPE + ": " + entity.getName, waitForKeystroke = false)

      defaultAttributeCopying(entity)

      Some(entity)
    } else {
      None
    }
  }

  def showInEntityMenuThenMainMenu(entityIn: Option[Entity]) {
    if (entityIn.isDefined) {
      //idea: is there a better way to do this, maybe have a single entityMenu for the class instead of new.. each time?
      new EntityMenu(ui, db, this).entityMenu(entityIn.get)
      // doing mainmenu right after entityMenu because that's where user would
      // naturally go after they exit the entityMenu.
      new MainMenu(ui, db, this).mainMenu(entityIn)
    }
  }

  /**
   * SEE DESCRIPTIVE COMMENT ON askForClassAndNameAndCreateEntity, WHICH APPLIES TO BOTH METHODS.
    *
    * The "previous..." parameters are for the already-existing data (ie, when editing not creating).
    *
    * @param existingIdIn should be None only if the call is intended to create; otherwise it is an edit.
    * @return None if user wants out.
    */
  def askForNameAndWriteEntity(inType: String, existingIdIn: Option[Long] = None,
                                         previousNameIn: Option[String] = None, previousDirectionalityIn: Option[String] = None,
                                         previousNameInReverseIn: Option[String] = None, inClassId: Option[Long] = None,
                                         inLeadingText: Option[String] = None): Option[Entity] = {
    if (inClassId.isDefined) require(inType == Controller.ENTITY_TYPE)
    val createNotUpdate: Boolean = existingIdIn.isEmpty
    if (!createNotUpdate && inType == Controller.RELATION_TYPE_TYPE) require(previousDirectionalityIn.isDefined)
    val maxNameLength = {
      if (inType == Controller.RELATION_TYPE_TYPE) model.RelationType.getNameLength(db)
      else if (inType == Controller.ENTITY_TYPE) model.Entity.nameLength(db)
      else throw new scala.Exception("invalid inType: " + inType)
    }
    val example = {
      if (inType == Controller.RELATION_TYPE_TYPE) " (use 3rd-person verb like \"owns\"--might make output like sentences more consistent later on)"
      else ""
    }

    /** 2nd Long in return value is ignored in this particular case.
      */
    def askAndSave(defaultNameIn: Option[String] = None): Option[(Long, Long)] = {
      val nameOpt = ui.askForString(Some(Array[String](inLeadingText.getOrElse(""),
                                                       "Enter " + inType + " name (up to " + maxNameLength + " characters" + example + "; ESC to cancel)")),
                                    None, defaultNameIn)
      if (nameOpt.isEmpty) None
      else {
        val name = nameOpt.get.trim()
        if (name.length <= 0) None
        else {
          var duplicate = false
          if (model.Entity.isDuplicate(db, name, existingIdIn)) {
            val answerOpt = ui.askForString(Some(Array("That name is a duplicate--proceed anyway? (y/n)")), None, Some("n"))
            if (answerOpt.isEmpty || (!answerOpt.get.equalsIgnoreCase("y"))) duplicate = true
          }
          // idea: this size check might be able to account better for the escaping that's done. Or just keep letting the exception handle it as is already
          // done in the caller of this.
          if (name.length > maxNameLength) {
            ui.displayText(stringTooLongErrorMessage(maxNameLength).format(tooLongMessage) + ".")
            askAndSave(Some(name))
          } else {
            if (duplicate) None
            else {
              if (inType == Controller.ENTITY_TYPE) {
                if (createNotUpdate) {
                  val newId = model.Entity.createEntity(db, name, inClassId).getId
                  Some(newId, 0L)
                } else {
                  db.updateEntityOnlyName(existingIdIn.get, name)
                  Some(existingIdIn.get, 0L)
                }
              } else if (inType == Controller.RELATION_TYPE_TYPE) {
                val ans: Option[String] = askForRelationDirectionality(previousDirectionalityIn)
                if (ans.isEmpty) None
                else {
                  val directionalityStr: String = ans.get.trim().toUpperCase
                  val nameInReverseDirectionStr = askForNameInReverseDirection(directionalityStr, maxNameLength, name, previousNameInReverseIn)
                  if (createNotUpdate) {
                    val newId = new RelationType(db, db.createRelationType(name, nameInReverseDirectionStr, directionalityStr)).getId
                    Some(newId, 0L)
                  } else {
                    db.updateRelationType(existingIdIn.get, name, nameInReverseDirectionStr, directionalityStr)
                    Some(existingIdIn.get, 0L)
                  }
                }
              } else throw new scala.Exception("unexpected value: " + inType)
            }
          }
        }
      }
    }

    val result = tryAskingAndSaving(stringTooLongErrorMessage(maxNameLength), askAndSave, previousNameIn)
    if (result.isEmpty) None
    else Some(new Entity(db, result.get._1))
  }

  /** Call a provided function (method?) "askAndSaveIn", which does some work that might throw a specific OmDatabaseException.  If it does throw that,
    * let the user know the problem and call askAndSaveIn again.  I.e., allow retrying if the entered data is bad, instead of crashing the app.
    */
  def tryAskingAndSaving(errorMsgIn: String, askAndSaveIn: (Option[String]) => Option[(Long, Long)],
                                   defaultNameIn: Option[String] = None): Option[(Long, Long)] = {
    try {
      askAndSaveIn(defaultNameIn)
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
        val cumulativeMsg = accumulateMsgs(e.toString, e.getCause)
        if (cumulativeMsg.contains(tooLongMessage)) {
          ui.displayText(errorMsgIn.format(tooLongMessage) + cumulativeMsg + ".")
          tryAskingAndSaving(errorMsgIn, askAndSaveIn, defaultNameIn)
        } else throw e
    }
  }

  /** Returns None if user wants out, otherwise returns the new or updated classId and entityId.
    * 1st parameter should be None only if the call is intended to create; otherwise it is an edit.
    * */
  def askForAndWriteClassAndDefiningEntityName(classIdIn: Option[Long] = None,
                                                         previousNameIn: Option[String] = None): Option[(Long, Long)] = {
    val createNotUpdate: Boolean = classIdIn.isEmpty
    val nameLength = model.EntityClass.nameLength(db)
    def askAndSave(defaultNameIn: Option[String]): Option[(Long, Long)] = {
      val nameOpt = ui.askForString(Some(Array("Enter class name (up to " + nameLength + " characters; will also be used for its defining entity name; ESC to" +
                                               " cancel): ")),
                                    None, defaultNameIn)
      if (nameOpt.isEmpty) None
      else {
        val name = nameOpt.get.trim()
        if (name.length() <= 0) None
        else {
          if (duplicationProblem(name, classIdIn, createNotUpdate)) None
          else {
            if (createNotUpdate) Some(db.createClassAndItsDefiningEntity(name))
            else {
              val entityId: Long = db.updateClassAndDefiningEntityName(classIdIn.get, name)
              Some(classIdIn.get, entityId)
            }
          }
        }
      }
    }

    tryAskingAndSaving(stringTooLongErrorMessage(nameLength), askAndSave, previousNameIn)
  }


  def stringTooLongErrorMessage(nameLength: Int): String = {
    // for details, see method PostgreSQLDatabase.escapeQuotesEtc.
    "Got an error.  Please try a shorter (" + nameLength + " chars) entry.  " +
    "(Could be due to escaped, i.e. expanded, characters like \"'\" or \";\".  Details: %s"
  }

  def duplicationProblem(name: String, previousIdIn: Option[Long], createNotUpdate: Boolean): Boolean = {
    var duplicateProblemSoSkip = false
    if (EntityClass.isDuplicate(db, name, previousIdIn)) {
      val answerOpt = ui.askForString(Some(Array("That name is a duplicate--proceed anyway? (y/n)")), None, Some("n"))
      if (answerOpt.isEmpty || (!answerOpt.get.equalsIgnoreCase("y"))) duplicateProblemSoSkip = true
    }
    duplicateProblemSoSkip
  }

  @tailrec final def askForNameInReverseDirection(directionalityStrIn: String, nameLengthIn: Int, nameIn: String,
                                                            previousNameInReverseIn: Option[String] = None): String = {
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
      if (ans.isEmpty || ans.get == 2) askForNameInReverseDirection(directionalityStrIn, nameLengthIn, nameIn, previousNameInReverseIn)
      else nameInReverse
    }
    else throw new Exception("unexpected value for directionality: " + directionalityStrIn)
  }

  def askForRelationDirectionality(previousDirectionalityIn: Option[String] = None): Option[String] = {
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

  val quantityTypePrompt: String = "SELECT TYPE OF QUANTITY (type is like length or volume, but not the measurement unit); ESC or leave both blank to cancel; " +
                                   "cancel if you need to create the needed type before selecting): "
  val textDescription: String = "TEXT (e.g., serial #)"


  /* NOTE: converting the parameters around here from DataHolder to Attribute... means also making the Attribute
  classes writable, and/or
     immutable and recreating them whenever there's a change, but also needing a way to pass around
     partial attribute data in a way that can be shared by code, like return values from the get[AttributeData...]
     methods.
     Need to learn more scala so I can do the equivalent of passing a Tuple without specifying the size in signatures?
   */
  def askForInfoAndUpdateAttribute[T <: AttributeDataHolder](inDH: T, askForAttrTypeId: Boolean, attrType: String, promptForSelectingTypeId: String,
                                                                       getOtherInfoFromUser: (T, Boolean) => Option[T], updateTypedAttribute: (T) => Unit) {
    @tailrec def askForInfoAndUpdateAttribute_helper(dhIn: T, attrType: String, promptForTypeId: String) {
      val ans: Option[T] = askForAttributeData[T](dhIn, askForAttrTypeId, attrType, Some(promptForTypeId), Some(new Entity(db, dhIn.attrTypeId).getName),
                                                  Some(inDH.attrTypeId), getOtherInfoFromUser, inEditing = true)
      if (ans.isDefined) {
        val dhOut: T = ans.get
        val ans2: Option[Int] = promptWhetherTo1Add2Correct(attrType)

        if (ans2.isEmpty) Unit
        else if (ans2.get == 1) {
          updateTypedAttribute(dhOut)
        }
        else if (ans2.get == 2) askForInfoAndUpdateAttribute_helper(dhOut, attrType, promptForTypeId)
        else throw new Exception("unexpected result! should never get here")
      }
    }
    askForInfoAndUpdateAttribute_helper(inDH, attrType, promptForSelectingTypeId)
  }

  /**
   * @return whether the attribute in question was deleted (or archived)
   */
  @tailrec
  final def attributeEditMenu(attributeIn: Attribute): Boolean = {
    val leadingText: Array[String] = Array("Attribute: " + attributeIn.getDisplayString(0, None, None))
    var firstChoices = Array("Edit the attribute type, " +
                             (if (canEditAttributeOnSingleLine(attributeIn)) "content (single line)," else "") +
                             " and valid/observed dates",

                             if (attributeIn.isInstanceOf[TextAttribute]) "Edit (as multiline value)" else "(stub)",
                             if (canEditAttributeOnSingleLine(attributeIn)) "Edit the attribute content (single line)" else "(stub)",
                             "Delete",
                             "Go to entity representing the type: " + new Entity(db, attributeIn.getAttrTypeId).getName)
    if (attributeIn.isInstanceOf[FileAttribute]) {
      firstChoices = firstChoices ++ Array[String]("Export the file")
    }
    val response = ui.askWhich(Some(leadingText), firstChoices)
    if (response.isEmpty) false
    else {
      val answer: Int = response.get
      if (answer == 1) {
        attributeIn match {
          case quantityAttribute: QuantityAttribute =>
            def updateQuantityAttribute(dhInOut: QuantityAttributeDataHolder) {
              quantityAttribute.update(dhInOut.attrTypeId, dhInOut.unitId, dhInOut.number, dhInOut.validOnDate,
                                       dhInOut.observationDate)
            }
            askForInfoAndUpdateAttribute[QuantityAttributeDataHolder](new QuantityAttributeDataHolder(quantityAttribute.getAttrTypeId,
                                                                                                      quantityAttribute.getValidOnDate,
                                                                                                      quantityAttribute.getObservationDate,
                                                                                                      quantityAttribute.getNumber, quantityAttribute.getUnitId),
                                                                      askForAttrTypeId = true, Controller.QUANTITY_TYPE, quantityTypePrompt,
                                                                      askForQuantityAttributeNumberAndUnit, updateQuantityAttribute)
            //force a reread from the DB so it shows the right info on the repeated menu:
            attributeEditMenu(new QuantityAttribute(db, attributeIn.getId))
          case textAttribute: TextAttribute =>
            def updateTextAttribute(dhInOut: TextAttributeDataHolder) {
              textAttribute.update(dhInOut.attrTypeId, dhInOut.text, dhInOut.validOnDate, dhInOut.observationDate)
            }
            val textAttributeDH: TextAttributeDataHolder = new TextAttributeDataHolder(textAttribute.getAttrTypeId, textAttribute.getValidOnDate,
                                                                                       textAttribute.getObservationDate, textAttribute.getText)
            askForInfoAndUpdateAttribute[TextAttributeDataHolder](textAttributeDH, askForAttrTypeId = true, Controller.TEXT_TYPE,
                                                                  "CHOOSE TYPE OF " + textDescription + ":",
                                                                  askForTextAttributeText, updateTextAttribute)
            //force a reread from the DB so it shows the right info on the repeated menu:
            attributeEditMenu(new TextAttribute(db, attributeIn.getId))
          case dateAttribute: DateAttribute =>
            def updateDateAttribute(dhInOut: DateAttributeDataHolder) {
              dateAttribute.update(dhInOut.attrTypeId, dhInOut.date)
            }
            val dateAttributeDH: DateAttributeDataHolder = new DateAttributeDataHolder(dateAttribute.getAttrTypeId, dateAttribute.getDate)
            askForInfoAndUpdateAttribute[DateAttributeDataHolder](dateAttributeDH, askForAttrTypeId = true, Controller.DATE_TYPE, "CHOOSE TYPE OF DATE:",
                                                                  askForDateAttributeValue, updateDateAttribute)
            //force a reread from the DB so it shows the right info on the repeated menu:
            attributeEditMenu(new DateAttribute(db, attributeIn.getId))
          case booleanAttribute: BooleanAttribute =>
            def updateBooleanAttribute(dhInOut: BooleanAttributeDataHolder) {
              booleanAttribute.update(dhInOut.attrTypeId, dhInOut.boolean, dhInOut.validOnDate, dhInOut.observationDate)
            }
            val booleanAttributeDH: BooleanAttributeDataHolder = new BooleanAttributeDataHolder(booleanAttribute.getAttrTypeId, booleanAttribute.getValidOnDate,
                                                                                                booleanAttribute.getObservationDate,
                                                                                                booleanAttribute.getBoolean)
            askForInfoAndUpdateAttribute[BooleanAttributeDataHolder](booleanAttributeDH, askForAttrTypeId = true, Controller.BOOLEAN_TYPE,
                                                                     "CHOOSE TYPE OF TRUE/FALSE VALUE:", askForBooleanAttributeValue, updateBooleanAttribute)
            //force a reread from the DB so it shows the right info on the repeated menu:
            attributeEditMenu(new BooleanAttribute(db, attributeIn.getId))
          case fa: FileAttribute =>
            def updateFileAttribute(dhInOut: FileAttributeDataHolder) {
              fa.update(Some(dhInOut.attrTypeId), Some(dhInOut.description))
            }
            val fileAttributeDH: FileAttributeDataHolder = new FileAttributeDataHolder(fa.getAttrTypeId, fa.getDescription, fa.getOriginalFilePath)
            askForInfoAndUpdateAttribute[FileAttributeDataHolder](fileAttributeDH, askForAttrTypeId = true, Controller.FILE_TYPE, "CHOOSE TYPE OF FILE:",
                                                                  askForFileAttributeInfo, updateFileAttribute)
            //force a reread from the DB so it shows the right info on the repeated menu:
            attributeEditMenu(new FileAttribute(db, attributeIn.getId))
          case _ => throw new Exception("Unexpected type: " + attributeIn.getClass.getName)
        }
      } else if (answer == 2 && attributeIn.isInstanceOf[TextAttribute]) {
        val ta = attributeIn.asInstanceOf[TextAttribute]
        val newContent: String = editMultilineText(ta.getText)
        ta.update(ta.getAttrTypeId, newContent, ta.getValidOnDate, ta.getObservationDate)
        //then force a reread from the DB so it shows the right info on the repeated menu:
        attributeEditMenu(new TextAttribute(db, attributeIn.getId))
      } else if (answer == 3 && canEditAttributeOnSingleLine(attributeIn)) {
        editAttributeOnSingleLine(attributeIn)
        false
      } else if (answer == 4) {
        val ans = ui.askYesNoQuestion("DELETE this attribute: ARE YOU SURE?")
        if (ans.isDefined && ans.get) {
          attributeIn.delete()
          true
        } else {
          ui.displayText("Did not delete attribute.", waitForKeystroke = false)
          attributeEditMenu(attributeIn)
        }
      } else if (answer == 5) {
        new EntityMenu(ui, db, this).entityMenu(new Entity(db, attributeIn.getAttrTypeId))
        attributeEditMenu(attributeIn)
      } else if (answer == 6) {
        if (!attributeIn.isInstanceOf[FileAttribute]) throw new Exception("Menu shouldn't have allowed us to get here w/ a type other than FA (" +
                                                                          attributeIn.getClass.getName + ").")
        val fa: FileAttribute = attributeIn.asInstanceOf[FileAttribute]
        try {
          // this file should be confirmed by the user as ok to write, even overwriting what is there.
          val file: Option[File] = ui.getExportDestinationFile(fa.getOriginalFilePath, fa.getMd5Hash)
          if (file.isDefined) {
            fa.retrieveContent(file.get)
            ui.displayText("File saved at: " + file.get.getCanonicalPath)
          }
        } catch {
          case e: Exception =>
            val msg: String = throwableToString(e)
            ui.displayText("Failed to export file, due to error: " + msg)
        }
        attributeEditMenu(attributeIn)
      } else {
        ui.displayText("invalid response")
        attributeEditMenu(attributeIn)
      }
    }
  }


  def editMultilineText(input: String): String = {
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

  /**
   * @return Whether the user wants just to get out.
   */
  def editAttributeOnSingleLine(attributeIn: Attribute): Boolean = {
    require(canEditAttributeOnSingleLine(attributeIn))

    attributeIn match {
      case quantityAttribute: QuantityAttribute =>
        val num: Option[Float] = askForQuantityAttributeNumber(quantityAttribute.getNumber)
        if (num.isDefined) {
          quantityAttribute.update(quantityAttribute.getAttrTypeId, quantityAttribute.getUnitId,
                                   num.get,
                                   quantityAttribute.getValidOnDate, quantityAttribute.getObservationDate)
        }
        num.isEmpty
      case textAttribute: TextAttribute =>
        val textAttributeDH: TextAttributeDataHolder = new TextAttributeDataHolder(textAttribute.getAttrTypeId, textAttribute.getValidOnDate,
                                                                                   textAttribute.getObservationDate, textAttribute.getText)
        val outDH: Option[TextAttributeDataHolder] = askForTextAttributeText(textAttributeDH, inEditing = true)
        if (outDH.isDefined) textAttribute.update(outDH.get.attrTypeId, outDH.get.text, outDH.get.validOnDate, outDH.get.observationDate)
        outDH.isEmpty
      case dateAttribute: DateAttribute =>
        val dateAttributeDH: DateAttributeDataHolder = new DateAttributeDataHolder(dateAttribute.getAttrTypeId, dateAttribute.getDate)
        val outDH: Option[DateAttributeDataHolder] = askForDateAttributeValue(dateAttributeDH, inEditing = true)
        if (outDH.isDefined) dateAttribute.update(outDH.get.attrTypeId, outDH.get.date)
        outDH.isEmpty
      case booleanAttribute: BooleanAttribute =>
        val booleanAttributeDH: BooleanAttributeDataHolder = new BooleanAttributeDataHolder(booleanAttribute.getAttrTypeId, booleanAttribute.getValidOnDate,
                                                                                            booleanAttribute.getObservationDate,
                                                                                            booleanAttribute.getBoolean)
        val outDH: Option[BooleanAttributeDataHolder] = askForBooleanAttributeValue(booleanAttributeDH, inEditing = true)
        if (outDH.isDefined) booleanAttribute.update(outDH.get.attrTypeId, outDH.get.boolean, outDH.get.validOnDate, outDH.get.observationDate)
        outDH.isEmpty
      case rte: RelationToEntity =>
        val editedEntity: Option[Entity] = editEntityName(new Entity(db, rte.getRelatedId2))
        editedEntity.isEmpty
      case rtg: RelationToGroup =>
        val editedGroupName: Option[String] = editGroupName(new Group(db, rtg.getGroupId))
        editedGroupName.isEmpty
      case _ => throw new scala.Exception("Unexpected type: " + attributeIn.getClass.getName)
    }
  }

  def canEditAttributeOnSingleLine(attributeIn: Attribute): Boolean = {
    ! attributeIn.isInstanceOf[FileAttribute]
  }

  def getReplacementFilename(originalFilePathIn: String): (String, String) = FileAttribute.getReplacementFilename(originalFilePathIn)

  /**
   * @return (See addAttribute method.)
   */
  def askForInfoAndAddAttribute[T <: AttributeDataHolder](inDH: T, askForAttrTypeId: Boolean, attrType: String, promptForSelectingTypeId: Option[String],
                                                                    getOtherInfoFromUser: (T, Boolean) => Option[T],
                                                                    addTypedAttribute: (T) => Option[Attribute]): Option[Attribute] = {
    val ans: Option[T] = askForAttributeData[T](inDH, askForAttrTypeId, attrType, promptForSelectingTypeId, None, None, getOtherInfoFromUser, inEditing = false)
    if (ans.isDefined) {
      val dhOut: T = ans.get
      addTypedAttribute(dhOut)
    } else None
  }

  val entityPartsThatCanBeAffected: String = "ALL its attributes, actions, and relations, but not entities or groups the relations refer to"

  /** Returns whether entity was deleted.
    */
  def deleteOrArchiveEntity(entityIn: Entity, delNotArchive: Boolean): Boolean = {
    val name = entityIn.getName
    val groupCount: Long = db.getCountOfGroupsContainingEntity(entityIn.getId)
    val groupsPrompt = if (groupCount == 0) ""
    else {
      val limit = 10
      val delimiter = ", "
      // (BUG: see comments in psql.java re "OTHER ENTITY NOTED IN A DELETION BUG")
      val descrArray = db.getRelationToGroupDescriptionsContaining(entityIn.getId, Some(limit))
      var descriptions = ""
      var counter = 0
      for (s: String <- descrArray) {
        counter += 1
        descriptions += counter + ") " + s + delimiter
      }
      descriptions = descriptions.substring(0, math.max(0, descriptions.length - delimiter.length)) + ".  "

      //removed next line because it doesn't make sense (& fails): there could be, for example, a single group that contains an
      //entity, but many entities that have a relation to that group:
      //require(descrArray.size == math.min(limit, groupCount))

      "This will ALSO remove it from " + (if (delNotArchive) "" else "visibility in ") + groupCount + " groups, " +
      "including for example these " + descrArray.length + " relations " +
      " that refer to this entity (showing entities & their relations to groups, as \"entity -> group\"): " + descriptions
    }
    // idea: WHEN CONSIDERING MODS TO THIS, ALSO CONSIDER THE Q'S ASKED AT CODE CMT WHERE DELETING A GROUP OF ENTITIES (SEE, for example "recursively").
    val ans = ui.askYesNoQuestion((if (delNotArchive) "DELETE" else "ARCHIVE") + " ENTITY \"" + name + "\" (and " + entityPartsThatCanBeAffected + ").  " +
                                  groupsPrompt + "**ARE YOU REALLY SURE?** (there is not yet an \"undo/un-archive\" feature, but it can be done" +
                                  " manually using SQL commands; feedback welcome):",
                                  if (delNotArchive) Some("n") else Some(""))
    if (ans.isDefined && ans.get) {
      if (delNotArchive) {
        entityIn.delete()
        ui.displayText("Deleted entity \"" + name + "\"" + ".")
      } else {
        entityIn.archive()
      }
      true
    }
    else {
      ui.displayText("Did not " + (if (delNotArchive) "delete" else "archive") + " entity.", waitForKeystroke = false)
      false
    }
  }

  val listNextItemsPrompt = "List next items"
  val listPrevItemsPrompt = "List previous items"
  val relationToGroupNamePrompt = "Type a name for this group (e.g., \"xyz list\"), then press Enter; blank or ESC to cancel"

  def addRemainingCountToPrompt(choicesIn: Array[String], numDisplayedObjects: Long, totalRowsAvailableIn: Long,
                                          startingDisplayRowIndexIn: Long): Array[String] = {
    val numLeft = totalRowsAvailableIn - startingDisplayRowIndexIn - numDisplayedObjects
    val indexOfPrompt = choicesIn.indexOf(listNextItemsPrompt)
    if (numLeft > 0 && indexOfPrompt >= 0) {
      choicesIn(indexOfPrompt) = listNextItemsPrompt + " (of " + numLeft + " more)"
    }
    choicesIn
  }

  /**
   * SEE DESCRIPTIVE COMMENT ON askForClassAndNameAndCreateEntity, WHICH APPLIES TO BOTH METHODS.
   *
   * @return None if user wants out.
   */
  def editEntityName(entityIn: Entity): Option[Entity] = {
    val editedEntity: Option[Entity] = entityIn match {
      case relTypeIn: RelationType =>
        val previousNameInReverse: String = relTypeIn.getNameInReverseDirection //idea: check: this edits name w/ prefill also?:
        askForNameAndWriteEntity(Controller.RELATION_TYPE_TYPE, Some(relTypeIn.getId), Some(relTypeIn.getName), Some(relTypeIn.getDirectionality),
                                 if (previousNameInReverse == null || previousNameInReverse.trim().isEmpty) None else Some(previousNameInReverse),
                                 None)
      case entity: Entity =>
        val entityNameBeforeEdit: String = entityIn.getName
        val editedEntity: Option[Entity] = askForNameAndWriteEntity(Controller.ENTITY_TYPE, Some(entity.getId), Some(entity.getName), None, None, None)
        if (editedEntity.isDefined) {
          val entityNameAfterEdit: String = editedEntity.get.getName
          if (entityNameBeforeEdit != entityNameAfterEdit) {
            val (_, _, groupId, moreThanOneAvailable) = db.findRelationToAndGroup_OnEntity(editedEntity.get.getId)
            if (groupId.isDefined && !moreThanOneAvailable) {
              val attrCount = entityIn.getAttrCount
              // for efficiency, if it's obvious which subgroup's name to change at the same time, offer to do so
              val ans = ui.askYesNoQuestion("There's a single subgroup" + (if (attrCount > 1) " (AMONG " + (attrCount - 1) + " OTHER ATTRIBUTES)" else "") +
                                            "; possibly it and this entity were created at the same time.  Also change" +
                                            " the subgroup's name now to be identical?", Some("y"))
              if (ans.isDefined && ans.get) {
                val group = new Group(db, groupId.get)
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

  def askForPublicNonpublicStatus(defaultForPrompt: Option[Boolean]): Option[Boolean] = {
    val valueAfterEdit: Option[Boolean] = ui.askYesNoQuestion("For Public vs. Non-public, enter a yes/no value (or a space" +
                                                              " for 'unknown/unspecified'; used e.g. during data export; display preference can be" +
                                                              " set under main menu / " + menuText_viewPreferences + ")",
                                                              if (defaultForPrompt.isEmpty) Some("") else if (defaultForPrompt.get) Some("y") else Some("n"),
                                                              allowBlankAnswer = true)
    valueAfterEdit
  }

  def getContainingEntitiesDescription(entityCountNonArchivedIn: Long, entityCountArchivedIn: Long): String = {
    "contained in " + entityCountNonArchivedIn + " entities, and in " + entityCountArchivedIn + " archived entities"
  }

  /**
   * @return None means "get out", or Some(choiceNum) if a choice was made.
   */
  def askWhetherDeleteOrArchiveEtc(entityIn: Entity, relationIn: Option[RelationToEntity], relationSourceEntityIn: Option[Entity],
                                   containingGroupIn: Option[Group]): (Option[Int], Int, Int) = {
    val groupCount: Long = db.getCountOfGroupsContainingEntity(entityIn.getId)
    val (entityCountNonArchived, entityCountArchived) = db.getCountOfEntitiesContainingEntity(entityIn.getId)
    val leadingText = Some(Array("Choose a deletion or archiving option:  (The entity is " +
                                 getContainingEntitiesDescription(entityCountNonArchived, entityCountArchived) + ", and " + groupCount + " groups.)"))
    var choices = Array("Delete this entity",
                        "Archive this entity (remove from visibility but not permanent/total deletion)")
    val delEntityLink_choiceNumber: Int = 3
    var delFromContainingGroup_choiceNumber: Int = 3
    // (check for existence because other things could have been deleted or archived while browsing around different menu options.)
    if (relationIn.isDefined && relationSourceEntityIn.isDefined && db.entityKeyExists(relationSourceEntityIn.get.getId)) {
     // means we got here by selecting a Relation attribute on another entity, so entityIn is the "entityId2" in that relation; so show some options,
      // because
      // we eliminated a separate menu just for the relation and put them here, for UI usage simplicity.
      choices = choices :+ "Delete the link between the linking (or containing) entity: \"" + relationSourceEntityIn.get.getName + "\", " +
                           "and this one: \"" + entityIn.getName + "\""
      delFromContainingGroup_choiceNumber += 1
    }
    if (containingGroupIn.isDefined) {
      choices = choices :+ "Delete the link between the group: \"" + containingGroupIn.get.getName + "\", and this Entity: \"" + entityIn.getName
    }

    val delOrArchiveAnswer: Option[(Int)] = ui.askWhich(leadingText, choices, Array[String]())
    (delOrArchiveAnswer, delEntityLink_choiceNumber, delFromContainingGroup_choiceNumber)
  }

  /** Returns None if user just wants out. */
  def promptWhetherTo1Add2Correct(inAttrTypeDesc: String): Option[Int] = {
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

  /** Returns data, or None if user wants to cancel/get out.
    * @param attrType Constant referring to Attribute subtype, as used by the inObjectType parameter to the chooseOrCreateObject method
    *                 (e.g., Controller.QUANTITY_TYPE).  See comment on that method, for that parm.
    * */
  def askForAttributeData[T <: AttributeDataHolder](inoutDH: T, askForAttrTypeId: Boolean, attrType: String, attrTypeInputPrompt: Option[String],
                                                    inPreviousSelectionDesc: Option[String], inPreviousSelectionId: Option[Long],
                                                    askForOtherInfo: (T, Boolean) => Option[T], inEditing: Boolean): Option[T] = {
    val (userWantsOut: Boolean, attrTypeId: Long) = {
      if (!askForAttrTypeId) {
        (false, inoutDH.attrTypeId)
      } else {
        require(attrTypeInputPrompt.isDefined)
        val ans: Option[Long] = askForAttributeTypeId(attrTypeInputPrompt.get, attrType, inPreviousSelectionDesc, inPreviousSelectionId)
        if (ans.isEmpty) {
          (true, 0)
        } else {
          (false, ans.get)
        }
      }
    }

    if (userWantsOut) {
      None
    } else {
      inoutDH.attrTypeId = attrTypeId
      val ans2: Option[T] = askForOtherInfo(inoutDH, inEditing)
      if (ans2.isEmpty) None
      else {
        var userWantsToCancel = false
        // (the ide/intellij preferred to have it this way instead of 'if')
        inoutDH match {
          case dhWithVOD: AttributeDataHolderWithVODates =>
            val (validOnDate: Option[Long], observationDate: Long, userWantsToCancelInner: Boolean) =
              askForAttributeValidAndObservedDates(inEditing, dhWithVOD.validOnDate, dhWithVOD.observationDate)

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

  def askForAttributeTypeId(prompt: String, attrType: String, inPreviousSelectionDesc: Option[String],
                                      inPreviousSelectionId: Option[Long]): (Option[Long]) = {
    val attrTypeSelection = chooseOrCreateObject(Some(List(prompt)), inPreviousSelectionDesc: Option[String], inPreviousSelectionId: Option[Long], attrType)
    if (attrTypeSelection.isEmpty) {
      ui.displayText("Blank, so assuming you want to cancel; if not come back & add again.", waitForKeystroke = false)
      None
    } else Some[Long](attrTypeSelection.get.getId)
  }

  val pickFromListPrompt: String = "Pick from menu, or an item by letter to select; Alt+<letter> to go to the item then come back here"

  /** Searches for a regex, case-insensitively, & returns the id of an Entity, or None if user wants out.  The parameter 'idToOmitIn' lets us omit
    * (or flag?) an entity if it should be for some reason (like it's the caller/container & doesn't make sense to be in the group, or something).
    *
    * Idea: re attrTypeIn parm, enum/improvement: see comment re inAttrType at beginning of chooseOrCreateObject.
    */
  @tailrec final def findExistingObjectByText(startingDisplayRowIndexIn: Long = 0, attrTypeIn: String,
                                                  idToOmitIn: Option[Long] = None, regexIn: String): Option[IdWrapper] = {
    val leadingText = List[String]("SEARCH RESULTS: " + pickFromListPrompt)
    val choices: Array[String] = Array(listNextItemsPrompt)
    val numDisplayableItems = ui.maxColumnarChoicesToDisplayAfter(leadingText.size, choices.length, Controller.maxNameLength)

    val objectsToDisplay = attrTypeIn match {
      case Controller.ENTITY_TYPE =>
        db.getMatchingEntities(startingDisplayRowIndexIn, Some(numDisplayableItems), idToOmitIn, regexIn)
      case Controller.GROUP_TYPE =>
        db.getMatchingGroups(startingDisplayRowIndexIn, Some(numDisplayableItems), idToOmitIn, regexIn)
      case _ =>
        throw new OmException("??")
    }
    if (objectsToDisplay.size == 0) {
      ui.displayText("End of list, or none found; starting over from the beginning...")
      if (startingDisplayRowIndexIn == 0) None
      else findExistingObjectByText(0, attrTypeIn, idToOmitIn, regexIn)
    } else {
      val objectNames: Array[String] = objectsToDisplay.toArray.map {
                                                                      case entity: Entity =>
                                                                        val numSubgroupsPrefix: String = getEntityContentSizePrefix(entity.getId)
                                                                        numSubgroupsPrefix + entity.getName
                                                                      case group: Group =>
                                                                        val numSubgroupsPrefix: String = getGroupContentSizePrefix(group.getId)
                                                                        numSubgroupsPrefix + group.getName
                                                                      case x: Any => throw new Exception("unexpected class: " + x.getClass.getName)
                                                                      case _ => throw new OmException("??")
                                                                    }
      val ans = ui.askWhichChoiceOrItsAlternate(Some(leadingText.toArray), choices, objectNames)
      if (ans.isEmpty) None
      else {
        val (answer, userChoseAlternate: Boolean) = ans.get
        if (answer == 1 && answer <= choices.length) {
          // (For reason behind " && answer <= choices.size", see comment where it is used in entityMenu.)
          val nextStartingIndex: Long = startingDisplayRowIndexIn + objectsToDisplay.size
          findExistingObjectByText(nextStartingIndex, attrTypeIn, idToOmitIn, regexIn)
        } else if (answer > choices.length && answer <= (choices.length + objectsToDisplay.size)) {
          // those in the condition on the previous line are 1-based, not 0-based.
          val index = answer - choices.length - 1
          val o = objectsToDisplay.get(index)
          if (userChoseAlternate) {
            attrTypeIn match {
              // idea: replace this condition by use of a trait (the type of o, which has getId), or being smarter with scala's type system. attrTypeIn match {
              case Controller.ENTITY_TYPE =>
                new EntityMenu(ui, db, this).entityMenu(o.asInstanceOf[Entity])
              case Controller.GROUP_TYPE =>
                // for now, picking the first RTG found for this group, until it's clear which of its RTGs to use.
                // (see also the other locations w/ similar comment!)
                // (There is probably no point in showing this GroupMenu with RTG info, since which RTG to use was picked arbitrarily, except if
                // that added info is a convenience, or if it helps the user clean up orphaned data sometimes.)
                val someRelationToGroups: java.util.ArrayList[RelationToGroup] = db.getRelationToGroupsByGroup(o.asInstanceOf[Group].getId, 0, Some(1))
                if (someRelationToGroups.size < 1) {
                  ui.displayText(Controller.ORPHANED_GROUP_MESSAGE)
                  new GroupMenu(ui, db, this).groupMenu(o.asInstanceOf[Group], 0, None, containingEntityIn = None)
                } else {
                  new GroupMenu(ui, db, this).groupMenu(o.asInstanceOf[Group], 0, Some(someRelationToGroups.get(0)), containingEntityIn = None)
                }
              case _ =>
                throw new OmException("??")
            }
            findExistingObjectByText(startingDisplayRowIndexIn, attrTypeIn, idToOmitIn, regexIn)
          } else {
            // user typed a letter to select.. (now 0-based); selected a new object and so we return to the previous menu w/ that one displayed & current
            attrTypeIn match {
              // idea: replace this condition by use of a trait (the type of o, which has getId), or being smarter with scala's type system. attrTypeIn match {
              case Controller.ENTITY_TYPE =>
                Some(new IdWrapper(o.asInstanceOf[Entity].getId))
              case Controller.GROUP_TYPE =>
                Some(new IdWrapper(o.asInstanceOf[Group].getId))
              case _ =>
                throw new OmException("??")
            }
          }
        }
        else {
          ui.displayText("unknown response")
          findExistingObjectByText(startingDisplayRowIndexIn, attrTypeIn, idToOmitIn, regexIn)
        }
      }
    }
  }

  def searchPromptPart(typeIn: String): String = "Enter part of the " + typeIn + " name to search for."

  def searchPrompt(typeNameIn: String): String = {
    searchPromptPart(typeNameIn) + "  (For the curious: it will be used in matching as a " +
    "case-insensitive POSIX " +
    "regex; details at  http://www.postgresql.org/docs/9.1/static/functions-matching.html#FUNCTIONS-POSIX-REGEXP .)"
  }

  /** Returns None if user wants out.  The parameter 'containingGroupIn' lets us omit entities that are already in a group,
    * i.e. omitting them from the list of entities (e.g. to add to the group), that this method returns.
    *
    * Idea: the inAttrType parm: do like in java & make it some kind of enum for type-safety? What's the scala idiom for that? (see also other
    * mentions of inAttrType for others to fix as well.)
    */
  @tailrec final def chooseOrCreateObject(inLeadingText: Option[List[String]], inPreviousSelectionDesc: Option[String],
                                                    inPreviousSelectionId: Option[Long], inObjectType: String, startingDisplayRowIndexIn: Long = 0,
                                                    inClassId: Option[Long] = None, limitByClassIn: Boolean = false,
                                                    containingGroupIn: Option[Long] = None,
                                                    markPreviousSelectionIn: Boolean = false): Option[IdWrapper] = {
    if (inClassId.isDefined) require(inObjectType == Controller.ENTITY_TYPE)
    val nonRelationAttrTypeNames = Array(Controller.TEXT_TYPE, Controller.QUANTITY_TYPE, Controller.DATE_TYPE, Controller.BOOLEAN_TYPE, Controller.FILE_TYPE)
    val mostAttrTypeNames = Array(Controller.ENTITY_TYPE, Controller.TEXT_TYPE, Controller.QUANTITY_TYPE, Controller.DATE_TYPE, Controller.BOOLEAN_TYPE,
                                  Controller.FILE_TYPE)
    val relationAttrTypeNames = Array(Controller.RELATION_TYPE_TYPE, Controller.RELATION_TO_ENTITY_TYPE, Controller.RELATION_TO_GROUP_TYPE)
    val evenMoreAttrTypeNames = Array(Controller.ENTITY_TYPE, Controller.TEXT_TYPE, Controller.QUANTITY_TYPE, Controller.DATE_TYPE, Controller.BOOLEAN_TYPE,
                                      Controller.FILE_TYPE, Controller.RELATION_TYPE_TYPE, Controller.RELATION_TO_ENTITY_TYPE,
                                      Controller.RELATION_TO_GROUP_TYPE)
    val listNextItemsChoiceNum = 1

    // attempt to keep these straight even though the size of the list, hence their option #'s on the menu,
    // is conditional:
    def getChoiceList: (Array[String], Int, Int, Int, Int, Int, Int, Int) = {
      var keepPreviousSelectionChoiceNum = 1
      var createAttrTypeChoiceNum = 1
      var searchForEntityByNameChoiceNum = 1
      var searchForEntityByIdChoiceNum = 1
      var showJournalChoiceNum = 1
      var createRelationTypeChoiceNum = 1
      var createClassChoiceNum = 1
      var choiceList = Array(listNextItemsPrompt)
      if (inPreviousSelectionDesc.isDefined) {
        choiceList = choiceList :+ "Keep previous selection (" + inPreviousSelectionDesc.get + ")."
        keepPreviousSelectionChoiceNum += 1
        createAttrTypeChoiceNum += 1
        searchForEntityByNameChoiceNum += 1
        searchForEntityByIdChoiceNum += 1
        showJournalChoiceNum += 1
        createRelationTypeChoiceNum += 1
        createClassChoiceNum += 1
      }
      //idea: use match instead of if: can it do || ?
      if (mostAttrTypeNames.contains(inObjectType)) {
        choiceList = choiceList :+ menuText_createEntityOrAttrType
        createAttrTypeChoiceNum += 1
        choiceList = choiceList :+ "Search for existing entity by name and text attribute content..."
        searchForEntityByNameChoiceNum += 2
        choiceList = choiceList :+ "Search for existing entity by id..."
        searchForEntityByIdChoiceNum += 3
        choiceList = choiceList :+ "Show journal (changed entities) by date range..."
        showJournalChoiceNum += 4
        createRelationTypeChoiceNum += 4
        createClassChoiceNum += 4
      } else if (relationAttrTypeNames.contains(inObjectType)) {
        choiceList = choiceList :+ menuText_createRelationType
        createRelationTypeChoiceNum += 1
        createClassChoiceNum += 1
      } else if (inObjectType == Controller.ENTITY_CLASS_TYPE) {
        choiceList = choiceList :+ "create new class (template for new entities)"
        createClassChoiceNum += 1
      } else throw new Exception("invalid inAttrType: " + inObjectType)

      (choiceList, keepPreviousSelectionChoiceNum, createAttrTypeChoiceNum, searchForEntityByNameChoiceNum, searchForEntityByIdChoiceNum, showJournalChoiceNum, createRelationTypeChoiceNum, createClassChoiceNum)
    }

    def getLeadTextAndObjectList(choicesIn: Array[String]): (List[String], java.util.ArrayList[_ >: RelationType with EntityClass <: Object], Array[String]) = {
      val prefix: String = inObjectType match {
        case Controller.ENTITY_TYPE => "ENTITIES: "
        case Controller.QUANTITY_TYPE => "QUANTITIES (entities): "
        case Controller.TEXT_TYPE => "TEXT ATTRIBUTES (entities): "
        case Controller.DATE_TYPE => "DATE ATTRIBUTES (entities): "
        case Controller.BOOLEAN_TYPE => "TRUE/FALSE ATTRIBUTES (entities): "
        case Controller.FILE_TYPE => "FILE ATTRIBUTES (entities): "
        case Controller.RELATION_TYPE_TYPE => "RELATION TYPES: "
        case Controller.ENTITY_CLASS_TYPE => "CLASSES: "
        case Controller.RELATION_TO_ENTITY_TYPE => "RELATION TYPES: "
        case Controller.RELATION_TO_GROUP_TYPE => "RELATION TYPES: "
        case _ => ""
      }
      var leadingText = inLeadingText.getOrElse(List[String](prefix + "Pick from menu, or an item by letter; Alt+<letter> to go to the item & later come back)"))
      val numDisplayableItems = ui.maxColumnarChoicesToDisplayAfter(leadingText.size + 3 /* up to: see more of leadingText below .*/ , choicesIn.length,
                                                                    Controller.maxNameLength)
      val objectsToDisplay = {
        // ** KEEP THESE QUERIES AND CONDITIONS IN SYNC W/ THE COROLLARY ONES 2x BELOW ! (at similar comment)
        if (nonRelationAttrTypeNames.contains(inObjectType)) db.getEntities(startingDisplayRowIndexIn, Some(numDisplayableItems))
        else if (inObjectType == Controller.ENTITY_TYPE) db.getEntitiesOnly(startingDisplayRowIndexIn, Some(numDisplayableItems), inClassId, limitByClassIn,
                                                                           inPreviousSelectionId,
                                                                           containingGroupIn)
        else if (relationAttrTypeNames.contains(inObjectType)) {
          db.getRelationTypes(startingDisplayRowIndexIn, Some(numDisplayableItems)).asInstanceOf[java.util.ArrayList[RelationType]]
        }
        else if (inObjectType == Controller.ENTITY_CLASS_TYPE) db.getClasses(startingDisplayRowIndexIn, Some(numDisplayableItems))
        else throw new Exception("invalid inAttrType: " + inObjectType)
      }
      if (objectsToDisplay.size == 0) {
        // IF THIS CHANGES: change the guess at the 1st parameter to maxColumnarChoicesToDisplayAfter, JUST ABOVE!
        val txt: String = TextUI.NEWLN + TextUI.NEWLN + "(None of the needed " + (if (inObjectType == "relationtype") "relation types" else "entities") +
                          " have been created in this model, yet."
        leadingText = leadingText ::: List(txt)
      }
      val totalExisting: Long = {
        // ** KEEP THESE QUERIES AND CONDITIONS IN SYNC W/ THE COROLLARY ONES 2x ELSEWHERE ! (at similar comment)
        if (nonRelationAttrTypeNames.contains(inObjectType)) db.getEntitiesOnlyCount(inClassId, limitByClassIn, inPreviousSelectionId)
        else if (inObjectType == Controller.ENTITY_TYPE) db.getEntitiesOnlyCount(inClassId, limitByClassIn, inPreviousSelectionId)
        else if (relationAttrTypeNames.contains(inObjectType)) db.getRelationTypeCount
        else if (inObjectType == Controller.ENTITY_CLASS_TYPE) db.getClassCount()
        else throw new Exception("invalid inAttrType: " + inObjectType)
      }
      addRemainingCountToPrompt(choicesIn, objectsToDisplay.size, totalExisting, startingDisplayRowIndexIn)
      val objectNames: Array[String] = objectsToDisplay.toArray.map {
                                                                      case entity: Entity => entity.getName
                                                                      case clazz: EntityClass => clazz.getName
                                                                      case x: Any => throw new Exception("unexpected class: " + x.getClass.getName)
                                                                      case _ => throw new Exception("??")
                                                                    }
      (leadingText, objectsToDisplay, objectNames)
    }

    def getNextStartingObjectIndex(previousListLength: Long, nonRelationAttrTypeNames: Array[String], relationAttrTypeNames: Array[String]): Long = {
      val index = {
        val x = startingDisplayRowIndexIn + previousListLength
        // ask Model for list of obj's w/ count desired & starting index (or "first") (in a sorted map, w/ id's as key, and names)
        //idea: should this just reuse the "totalExisting" value alr calculated in above in getLeadTextAndObjectList just above?
        val numObjectsInModel =
        // ** KEEP THESE QUERIES AND CONDITIONS IN SYNC W/ THE COROLLARY ONES 2x ABOVE ! (at similar comment)
          if (nonRelationAttrTypeNames.contains(inObjectType))
            db.getEntityCount
          else if (inObjectType == Controller.ENTITY_TYPE) db.getEntitiesOnlyCount(inClassId, limitByClassIn)
          else if (relationAttrTypeNames.contains(inObjectType))
            db.getRelationTypeCount
          else if (inObjectType == Controller.ENTITY_CLASS_TYPE) db.getClassCount()
          else throw new Exception("invalid inAttrType: " + inObjectType)
        if (x >= numObjectsInModel) {
          ui.displayText("End of list found; starting over from the beginning.")
          0 // start over
        } else x
      }
      index
    }

    val (choices, keepPreviousSelectionChoice, createAttrTypeChoice, searchForEntityByNameChoice, searchForEntityByIdChoice, showJournalChoice, createRelationTypeChoice, createClassChoice): (Array[String],
      Int, Int, Int, Int, Int, Int, Int) = getChoiceList

    val (leadingText, objectsToDisplay, names) = getLeadTextAndObjectList(choices)
    val ans = ui.askWhichChoiceOrItsAlternate(Some(leadingText.toArray), choices, names)

    if (ans.isEmpty) None
    else {
      val answer = ans.get._1
      val userChoseAlternate = ans.get._2
      if (answer == listNextItemsChoiceNum && answer <= choices.length) {
        // (For reason behind " && answer <= choices.length", see comment where it is used in entityMenu.)
        val index: Long = getNextStartingObjectIndex(objectsToDisplay.size, nonRelationAttrTypeNames, relationAttrTypeNames)
        chooseOrCreateObject(inLeadingText, inPreviousSelectionDesc, inPreviousSelectionId, inObjectType, index, inClassId, limitByClassIn,
                             containingGroupIn, markPreviousSelectionIn)
      } else if (answer == keepPreviousSelectionChoice && answer <= choices.length) {
        // Such as if editing several fields on an attribute and doesn't want to change the first one.
        // Not using "get out" option for this because it would exit from a few levels at once and
        // then user wouldn't be able to proceed to other field edits.
        Some(new IdWrapper(inPreviousSelectionId.get))
      } else if (answer == createAttrTypeChoice && answer <= choices.length) {
        val e: Option[Entity] = askForClassInfoAndNameAndCreateEntity(inClassId)
        if (e.isEmpty) None
        else Some(new IdWrapper(e.get.getId))
      } else if (answer == searchForEntityByNameChoice && answer <= choices.length) {
        askForNameAndSearchForEntity
      } else if (answer == showJournalChoice && answer <= choices.length) {
        // THIS IS CRUDE RIGHT NOW AND DOESN'T ABSTRACT TEXT SCREEN OUTPUT INTO THE UI CLASS very neatly perhaps, BUT IS HELPFUL ANYWAY:
        // ideas:
          // move the lines for this little section, into a separate method, near findExistingObjectByName
          // do something similar (refactoring findExistingObjectByName?) to show the results in a list, but make clear on *each line* what kind of result it is.
          // where going to each letter w/ Alt key does the same: goes 2 that entity so one can see its context, etc.
          // change the "None" returned to be the selected entity, like the little section above does.
          // could keep this text output as an option?
        val yDate = new java.util.Date(System.currentTimeMillis() - (24 * 60 * 60 * 1000))
        val yesterday: String = new java.text.SimpleDateFormat("yyyy-MM-dd").format(yDate)
        val beginDate: Option[Long] = askForDate_generic(Some("BEGINNING date in the time range: " + genericDatePrompt), Some(yesterday))
        if (beginDate.isEmpty) None
        else {
          val endDate: Option[Long] = askForDate_generic(Some("ENDING date in the time range: " + genericDatePrompt), None)
          if (endDate.isEmpty) None
          else {
            var dayCurrentlyShowing: String = ""
            val results: Array[(Long, String, Long)] = db.findJournalEntries(beginDate.get, endDate.get)
            for (result <- results) {
              val date = new java.text.SimpleDateFormat("yyyy-MM-dd").format(result._1)
              if (dayCurrentlyShowing != date) {
                ui.out.println("\n\nFor: " + date + "------------------")
                dayCurrentlyShowing = date
              }
              val time: String = new java.text.SimpleDateFormat("HH:mm:ss").format(result._1)
              ui.out.println(time + " " + result._3 + ": " + result._2)
            }
            ui.out.println("\n(For other ~'journal' info, could see other things for the day in question, like email, code commits, or entries made on a" +
                               " different day in a specific \"journal\" section of OM.)")
            ui.displayText("Scroll back to see more info if needed.  Press any key to continue...")
            None
          }
        }

      } else if (answer == searchForEntityByIdChoice && answer <= choices.length) {
        searchById(Controller.ENTITY_TYPE)
      } else if (answer == createRelationTypeChoice && relationAttrTypeNames.contains(inObjectType) && answer <= choices.length) {
        val entity: Option[Entity] = askForNameAndWriteEntity(Controller.RELATION_TYPE_TYPE)
        if (entity.isEmpty) None
        else Some(new IdWrapper(entity.get.getId))
      } else if (answer == createClassChoice && inObjectType == Controller.ENTITY_CLASS_TYPE && answer <= choices.length) {
        val result: Option[(Long, Long)] = askForAndWriteClassAndDefiningEntityName()
        if (result.isEmpty) None
        else {
          val (classId, entityId) = result.get
          val ans = ui.askYesNoQuestion("Do you want to add attributes to the newly created defining entity for this class? (These will be used for the " +
                                        "prompts " +
                                        "and defaults when creating/editing entities in this class).", Some("y"))
          if (ans.isDefined && ans.get) {
            new EntityMenu(ui, db, this).entityMenu(new Entity(db, entityId))
          }
          Some(new IdWrapper(classId))
        }
      } else if (answer > choices.length && answer <= (choices.length + objectsToDisplay.size)) {
        // those in the condition on the previous line are 1-based, not 0-based.
        val index = answer - choices.length - 1
        // user typed a letter to select.. (now 0-based)
        // user selected a new object and so we return to the previous menu w/ that one displayed & current
        val o = objectsToDisplay.get(index)
        //if ("text,quantity,entity,date,boolean,file,relationtype".contains(inAttrType)) {
        //i.e., if (inAttrType == Controller.TEXT_TYPE || (= any of the other types...)):
        if (userChoseAlternate) {
          inObjectType match {
            // idea: replace this condition by use of a trait (the type of o, which has getId), or being smarter with scala's type system. attrTypeIn match {
            case Controller.ENTITY_TYPE =>
              new EntityMenu(ui, db, this).entityMenu(o.asInstanceOf[Entity])
            case _ =>
              // (choosing a group doesn't call this, it calls chooseOrCreateGroup)
              throw new OmException("not yet implemented")
          }
          chooseOrCreateObject(inLeadingText, inPreviousSelectionDesc, inPreviousSelectionId, inObjectType, startingDisplayRowIndexIn, inClassId, limitByClassIn,
                               containingGroupIn, markPreviousSelectionIn)
        } else {
          if (evenMoreAttrTypeNames.contains(inObjectType))
            Some(o.asInstanceOf[Entity].getIdWrapper)
          else if (inObjectType == Controller.ENTITY_CLASS_TYPE) Some(o.asInstanceOf[EntityClass].getIdWrapper)
          else throw new Exception("invalid inAttrType: " + inObjectType)
        }
      } else {
        ui.displayText("unknown response")
        chooseOrCreateObject(inLeadingText, inPreviousSelectionDesc, inPreviousSelectionId, inObjectType, startingDisplayRowIndexIn, inClassId,
                             limitByClassIn, containingGroupIn, markPreviousSelectionIn)
      }
    }
  }

  def askForNameAndSearchForEntity: Option[IdWrapper] = {
    val ans = ui.askForString(Some(Array(searchPrompt(Controller.ENTITY_TYPE))))
    if (ans.isEmpty) {
      None
    } else {
      // Allow relation to self (eg, picking self as 2nd part of a RelationToEntity), so None in 3nd parm.
      val e: Option[IdWrapper] = findExistingObjectByText(0, Controller.ENTITY_TYPE, None, ans.get)
      if (e.isEmpty) None
      else Some(new IdWrapper(e.get.getId))
    }
  }

  def searchById(typeNameIn: String): Option[IdWrapper] = {
    require(typeNameIn == Controller.ENTITY_TYPE || typeNameIn == Controller.GROUP_TYPE)
    val ans = ui.askForString(Some(Array("Enter the " + typeNameIn + " ID to search for:")))
    if (ans.isEmpty) {
      None
    } else {
      // it's a long:
      val idString: String = ans.get
      if (!isNumeric(idString)) {
        ui.displayText("Invalid ID format.  An ID is a numeric value between " + db.minIdValue + " and " + db.maxIdValue)
        None
      } else {
        // (BTW, do allow relation to self, e.g., picking self as 2nd part of a RelationToEntity.)
        if ((typeNameIn == Controller.ENTITY_TYPE && db.entityKeyExists(idString.toLong)) ||
            (typeNameIn == Controller.GROUP_TYPE && db.groupKeyExists(idString.toLong))) {
          Some(new IdWrapper(idString.toLong))
        } else {
          ui.displayText("The " + typeNameIn + " ID " + ans.get + " was not found in the database.")
          None
        }
      }
    }
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

  /** Returns None if user wants to cancel. */
  def askForQuantityAttributeNumberAndUnit(inDH: QuantityAttributeDataHolder, inEditing: Boolean): Option[QuantityAttributeDataHolder] = {
    val outDH: QuantityAttributeDataHolder = inDH
    val leadingText: List[String] = List("SELECT A *UNIT* FOR THIS QUANTITY (i.e., centimeters, or quarts; ESC or blank to cancel):")
    val previousSelectionDesc = if (inEditing) Some(new Entity(db, inDH.unitId).getName) else None
    val previousSelectionId = if (inEditing) Some(inDH.unitId) else None
    val unitSelection = chooseOrCreateObject(Some(leadingText), previousSelectionDesc, previousSelectionId, Controller.QUANTITY_TYPE)
    if (unitSelection.isEmpty) {
      ui.displayText("Blank, so assuming you want to cancel; if not come back & add again.", waitForKeystroke = false)
      None
    } else {
      outDH.unitId = unitSelection.get.getId
      val ans: Option[Float] = askForQuantityAttributeNumber(outDH.number)
      if (ans.isEmpty) None
      else {
        outDH.number = ans.get
        Some(outDH)
      }
    }
  }

  def askForQuantityAttributeNumber(previousQuantity: Float): Option[Float] = {
    val leadingText = Array[String]("ENTER THE NUMBER FOR THE QUANTITY (i.e., 5, for 5 centimeters length)")
    val ans = ui.askForString(Some(leadingText), Some(isNumeric), Some(previousQuantity.toString))
    if (ans.isEmpty) None
    else Some(ans.get.toFloat)
  }

  /** Returns None if user wants to cancel. */
  def askForTextAttributeText(inDH: TextAttributeDataHolder, inEditing: Boolean): Option[TextAttributeDataHolder] = {
    val outDH = inDH.asInstanceOf[TextAttributeDataHolder]
    val defaultValue: Option[String] = if (inEditing) Some(inDH.text) else None
    val ans = ui.askForString(Some(Array("Type or paste a single-line attribute value, then press Enter; ESC to cancel.  (If you need to add or edit multiple lines, just " +
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
  def askForDateAttributeValue(inDH: DateAttributeDataHolder, inEditing: Boolean): Option[DateAttributeDataHolder] = {
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
      else Controller.DATEFORMAT.format(System.currentTimeMillis())
    }

    def dateCriteria(date: String): Boolean = {
      !finishAndParseTheDate(date)._2
    }
    val ans = ui.askForString(Some(Array(genericDatePrompt)), Some(dateCriteria), Some(defaultValue))
    if (ans.isEmpty) None
    else {
      val (newDate: Option[Long], retry: Boolean) = finishAndParseTheDate(ans.get)
      if (retry) throw new Exception("Programmer error: date indicated it was parseable, but the same function said afterward it couldn't be parsed.  Why?")
      else if (newDate.isEmpty) throw new Exception("There is a bug: the program shouldn't have got to this point.")
      else {
        outDH.date = newDate.get
        Some(outDH)
      }
    }
  }

  /** Returns None if user wants to cancel. */
  def askForBooleanAttributeValue(inDH: BooleanAttributeDataHolder, inEditing: Boolean): Option[BooleanAttributeDataHolder] = {
    val outDH = inDH.asInstanceOf[BooleanAttributeDataHolder]
    val ans = ui.askYesNoQuestion("Set the new value to true now? ('y' if so, 'n' for false)", if (inEditing && inDH.boolean) Some("y") else Some("n"))
    if (ans.isEmpty) None
    else {
      outDH.boolean = ans.get
      Some(outDH)
    }
  }

  def inputFileValid(path: String): Boolean = {
    val file = new java.io.File(path)
    file.exists && file.canRead
  }

  /** Returns None if user wants to cancel. */
  def askForFileAttributeInfo(inDH: FileAttributeDataHolder, inEditing: Boolean): Option[FileAttributeDataHolder] = {
    val outDH = inDH.asInstanceOf[FileAttributeDataHolder]
    var path: Option[String] = None
    if (!inEditing) {
      // we don't want the original path to be editable after the fact, because that's a historical observation and there is no sense in changing it.
      path = ui.askForString(Some(Array("Enter file path (must exist and be readable), then press Enter; ESC to cancel")), Some(inputFileValid))
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

  /** Returns None if user wants to cancel. */
  def askForRelToGroupInfo(inDH: RelationToGroupDataHolder, inEditingUNUSEDForNOW: Boolean = false): Option[RelationToGroupDataHolder] = {
    val outDH = inDH

    val groupSelection = chooseOrCreateGroup(Some(List("SELECT GROUP FOR THIS RELATION")), 0)
    val groupId: Option[Long] = {
      if (groupSelection.isEmpty) {
        ui.displayText("Blank, so assuming you want to cancel; if not come back & add again.", waitForKeystroke = false)
        None
      } else Some[Long](groupSelection.get.getId)
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
  @tailrec final def chooseOrCreateGroup(inLeadingText: Option[List[String]], startingDisplayRowIndexIn: Long = 0,
                                                   containingGroupIn: Option[Long] = None /*ie group to omit from pick list*/): Option[IdWrapper] = {
    val totalExisting: Long = db.getGroupCount
    def getNextStartingObjectIndex(currentListLength: Long): Long = {
      val x = startingDisplayRowIndexIn + currentListLength
      if (x >= totalExisting) {
        ui.displayText("End of list found; starting over from the beginning.")
        0 // start over
      } else x
    }
    var leadingText = inLeadingText.getOrElse(List[String](pickFromListPrompt))
    val choicesPreAdjustment: Array[String] = Array("List next items",
                                                    "Create new group (aka RelationToGroup)",
                                                    "Search for existing group by name...",
                                                    "Search for existing group by id...")
    val numDisplayableItems = ui.maxColumnarChoicesToDisplayAfter(leadingText.size, choicesPreAdjustment.length, Controller.maxNameLength)
    val objectsToDisplay = db.getGroups(startingDisplayRowIndexIn, Some(numDisplayableItems), containingGroupIn)
    if (objectsToDisplay.size == 0) {
      val txt: String = TextUI.NEWLN + TextUI.NEWLN + "(None of the needed groups have been created in this model, yet."
      leadingText = leadingText ::: List(txt)
    }
    val choices = addRemainingCountToPrompt(choicesPreAdjustment, objectsToDisplay.size, totalExisting, startingDisplayRowIndexIn)
    val objectNames: Array[String] = objectsToDisplay.toArray.map {
                                                                    case group: Group => group.getName
                                                                    case x: Any => throw new Exception("unexpected class: " + x.getClass.getName)
                                                                    case _ => throw new Exception("??")
                                                                  }
    val ans = ui.askWhichChoiceOrItsAlternate(Some(leadingText.toArray), choices, objectNames)
    if (ans.isEmpty) None
    else {
      val answer = ans.get._1
      val userChoseAlternate = ans.get._2
      if (answer == 1 && answer <= choices.length) {
        // (For reason behind " && answer <= choices.size", see comment where it is used in entityMenu.)
        val nextStartingIndex: Long = getNextStartingObjectIndex(objectsToDisplay.size)
        chooseOrCreateGroup(inLeadingText, nextStartingIndex, containingGroupIn)
      } else if (answer == 2 && answer <= choices.length) {
        val ans = ui.askForString(Some(Array(relationToGroupNamePrompt)))
        if (ans.isEmpty || ans.get.trim.length() == 0) None
        else {
          val name = ans.get
          val ans2 = ui.askYesNoQuestion("Should this group allow entities with mixed classes? (Usually not desirable: doing so means losing some " +
                                         "conveniences such as scripts and assisted data entry.)", Some("n"))
          if (ans2.isEmpty) None
          else {
            val mixedClassesAllowed = ans2.get
            val newGroupId = db.createGroup(name, mixedClassesAllowed)
            Some(new IdWrapper(newGroupId))
          }
        }
      } else if (answer == 3 && answer <= choices.length) {
        val ans = ui.askForString(Some(Array(searchPrompt(Controller.GROUP_TYPE))))
        if (ans.isEmpty) None
        else {
          // Allow relation to self, so None in 2nd parm.
          val g: Option[IdWrapper] = findExistingObjectByText(0, Controller.GROUP_TYPE, None, ans.get)
          if (g.isEmpty) None
          else Some(new IdWrapper(g.get.getId))
        }
      } else if (answer == 4 && answer <= choices.length) {
        searchById(Controller.GROUP_TYPE)
      } else if (answer > choices.length && answer <= (choices.length + objectsToDisplay.size)) {
        // those in that^ condition are 1-based, not 0-based.
        val index = answer - choices.length - 1
        val o = objectsToDisplay.get(index)
        if (userChoseAlternate) {
          // for now, picking the first RTG found for this group, until it's clear which of its RTGs to use.
          // (see also the other locations w/ similar comment!)
          val someRelationToGroups: java.util.ArrayList[RelationToGroup] = db.getRelationToGroupsByGroup(o.asInstanceOf[Group].getId, 0, Some(1))
          new GroupMenu(ui, db, this).groupMenu(new Group(db, someRelationToGroups.get(0).getGroupId), 0, Some(someRelationToGroups.get(0)),
                                                containingEntityIn = None)
          chooseOrCreateGroup(inLeadingText, startingDisplayRowIndexIn, containingGroupIn)
        } else {
          // user typed a letter to select.. (now 0-based); selected a new object and so we return to the previous menu w/ that one displayed & current
          Some(new IdWrapper(o.getId))
        }
      }
      else {
        ui.displayText("unknown response")
        chooseOrCreateGroup(inLeadingText, startingDisplayRowIndexIn, containingGroupIn)
      }
    }
  }

  /** Returns None if user wants to cancel. */
  def askForRelationEntityIdNumber2(inDH: RelationToEntityDataHolder, inEditing: Boolean): Option[RelationToEntityDataHolder] = {
    val previousSelectionDesc = if (!inEditing) None
    else Some(new Entity(db, inDH.entityId2).getName)
    val previousSelectionId = if (!inEditing) None
    else Some(inDH.entityId2)
    val (id: Option[Long]) = askForAttributeTypeId("SELECT OTHER (RELATED) ENTITY FOR THIS RELATION", Controller.ENTITY_TYPE, previousSelectionDesc,
                                                   previousSelectionId)
    val outDH = inDH
    if (id.isEmpty) None
    else {
      outDH.entityId2 = id.get
      Some(outDH)
    }
  }

  /** A helper method.  Returns the date as a Long (java-style: ms since 1970 began), and true if there is a problem w/ the string and we need to ask again. */
  def finishAndParseTheDate(dateStrIn: String, blankMeansNOW: Boolean = true): (Option[Long], Boolean) = {
    //to start with, the special forms (be sure to trim the input, otherwise there's no way in the textui to convert from a previously entered (so default)
    //value to "blank/all time"!).
    val dateStrWithOptionalEra =
      if (dateStrIn.equalsIgnoreCase("now") || (blankMeansNOW && dateStrIn.trim.length() == 0)) {
        val currentDateString: String = Controller.DATEFORMAT.format(new java.util.Date(System.currentTimeMillis()))
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
        if (era.isEmpty) Controller.DATEFORMAT.parse(dateStrWithZeros)
        else Controller.DATEFORMAT_WITH_ERA.parse(era + dateStrWithZeros)
      } catch {
        case e: java.text.ParseException =>
          try {
            if (era.isEmpty) Controller.DATEFORMAT2.parse(dateStrWithZeros)
            else Controller.DATEFORMAT2_WITH_ERA.parse(era + dateStrWithZeros)
          } catch {
            case e: java.text.ParseException =>
              if (era.isEmpty) Controller.DATEFORMAT3.parse(dateStrWithZeros)
              else Controller.DATEFORMAT3_WITH_ERA.parse(era + dateStrWithZeros)
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
                                                     oldObservedDateIn: Long): (Option[Long], Long, Boolean) = {

    //idea: make this more generic, passing in prompt strings &c, so it's more cleanly useful for DateAttribute instances. Or not: lacks shared code.
    //idea: separate these into 2 methods, 1 for each time (not much common material of significance).
    // BETTER IDEA: fix the date stuff in the DB first as noted in tasks, so that this part makes more sense (the 0 for all time, etc), and then
    // when at it, recombine the askForDate_Generic method w/ these or so it's all cleaned up.
    /** Helper method made so it can be recursive, it returns the date (w/ meanings as with displayText below, and as in PostgreSQLDatabase.createTables),
      * and true if the user wants to cancel/get out). */
    @tailrec def askForDate(dateTypeIn: String, acceptanceCriteriaIn: (String) => Boolean): (Option[Long], Boolean) = {
      val leadingText: Array[String] = {
        if (dateTypeIn == VALID) {
          Array("\nPlease enter the date when this was first VALID (i.e., true) (Press Enter (blank) for unknown/unspecified, or " +
                "like this, w/ at least the year: \"2002\", \"2000-1-31\", or" +
                " \"2013-01-31 23:59:59:999 MST\"; zeros are " +
                "allowed in all but the yyyy-mm-dd.  Or for current date/time " +
                "enter \"now\".  " +
//                "ESC to exit this.  " + //THIS LINE CAN BE PUT BACK AFTER the bug is fixed so ESC really works.  See similar cmt elsewhere; tracked in tasks.
                "For dates far in the past you can prefix them with \"BC\" (or \"AD\", but either way omit a space " +
                "before the year), like BC3400-01-31 23:59:59:999 GMT, entered at least up through the year, up to ~292000000 years AD or BC.")
          //IDEA: I had thought to say:  "Or for "all time", enter just 0.  ", BUT (while this is probably solved, it's not until the later part of
                // this comment):
//                "There is ambiguity about BC that needs some " +
//                "investigation, because java allows a '0' year (which for now means 'for all time' in just this program), but normal human time doesn't " +
//                "allow a '0' year, so maybe you have to subtract a year from all BC things for them to work right, and enter/read them accordingly, until " +
//                "someone learns for sure, and we decide whether to subtract a year from everything BC for you automatically. Hm. *OR* maybe dates in year " +
//                "zero " +
//                "just don't mean anything so can be ignored by users, and all other dates' entry are just fine, so there's nothing to do but use it as is? " +
//                "But that would have to be kept in mind if doing any relative date calculations in the program, e.g. # of years, spanning 0.)" + TextUI.NEWLN +
//                "Also, real events with more " +
//                "specific time-tracking needs will probably need to model their own time-related entity classes, and establish relations to them, within " +
//                "their use of OM.")
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
            else Some(Controller.DATEFORMAT_WITH_ERA.format(new Date(oldValidOnDateIn.get)))
          }
          else None
        } else if (dateTypeIn == OBSERVED) {
          if (inEditing) {
            Some(Controller.DATEFORMAT_WITH_ERA.format(new Date(oldObservedDateIn)))
          } else {
            Some(Controller.DATEFORMAT_WITH_ERA.format(new Date(System.currentTimeMillis())))
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
          val (newDate: Option[Long], retry: Boolean) = finishAndParseTheDate(dateStr, dateTypeIn == OBSERVED)
          if (retry) askForDate(dateTypeIn, acceptanceCriteriaIn)
          else {
            (newDate, false)
          }
        }
      }
    }

    // The check to see if a long date string is valid comes later.
    // Now that we allow 1-digit dates, there is nothing to ck really.
    def validOnDateCriteria(dateStr: String): Boolean = true
    // Same comments as for observedDateCriteria:
    def observedDateCriteria(dateStr: String): Boolean = true

    // the real action:
    def askForBothDates(): (Option[Long], Long, Boolean) = {
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
          else askForBothDates()
        }
      }
    }
    askForBothDates()
  }

  def goToEntityOrItsSoleGroupsMenu(userSelection: Entity, relationToGroupIn: Option[RelationToGroup] = None,
                                              containingGroupIn: Option[Group] = None): (Option[Entity], Option[Long], Boolean) = {
    val (rtgid, rtid, groupId, moreThanOneAvailable) = db.findRelationToAndGroup_OnEntity(userSelection.getId)
    val subEntitySelected: Option[Entity] = None
    if (groupId.isDefined && !moreThanOneAvailable && db.getAttrCount(userSelection.getId) == 1) {
      // In quick menu, for efficiency of some work like brainstorming, if it's obvious which subgroup to go to, just go there.
      // We DON'T want @tailrec on this method for this call, so that we can ESC back to the current menu & list! (so what balance/best? Maybe move this
      // to its own method, so it doesn't try to tail optimize it?)  See also the comment with 'tailrec', mentioning why to have it, above.
      new QuickGroupMenu(ui, db, this).quickGroupMenu(new Group(db, groupId.get),
                                                      0,
                                                      Some(new RelationToGroup(db, rtgid.get, userSelection.getId, rtid.get, groupId.get)),
                                                      callingMenusRtgIn = relationToGroupIn,
                                                      containingEntityIn = Some(userSelection))
    } else {
      new EntityMenu(ui, db, this).entityMenu(userSelection, containingGroupIn = containingGroupIn)
    }
    (subEntitySelected, groupId, moreThanOneAvailable)
  }

  /** see comments for getContentSizePrefix. */
  def getGroupContentSizePrefix(groupId: Long): String = {
    val grpSize = db.getGroupSize(groupId, Some(false))
    if (grpSize == 0) ""
    else ">"
  }

  /** Shows ">" in front of an entity or group if it contains exactly one attribute or a subgroup which has at least one entry; shows ">>" if contains 
    * multiple subgroups or attributes, and "" if contains no subgroups or the one subgroup is empty.
    * Idea: this might better be handled in the textui class instead, and the same for all the other color stuff.
    */
  def getEntityContentSizePrefix(entityId: Long): String = {
    // attrCount counts groups also, so account for the overlap in the below.
    val attrCount = db.getAttrCount(entityId)
    // removed the below because it was buggy and not worth it at the time, but might be desired again, like, to not show that an entity contains
    // more things (">" prefix...) if it only has one group which has no *non-archived* entities:
//    val (groupsCount: Long, singleGroupEntryCount: Long) = {
//      val rtgCountOnEntity: Long = db.getRelationToGroupCountByEntity(Some(entityId))
//      if (rtgCountOnEntity == 0) {
//        (0L, 0L)
//      } else if (rtgCountOnEntity > 1) {
//        // (For some reason, not having the 'asInstanceOf[Long]' here results in a stack trace on the variable assignment out of this block, with something
//        // about a tuple mismatch?, even tho it is already a Long:)
//        (rtgCountOnEntity.asInstanceOf[Long], 0L)
//      } else {
//        val (_, _, gid: Option[Long], moreAvailable) = db.findRelationToAndGroup_OnEntity(entityId)
//        if (gid.isEmpty || moreAvailable) throw new OmException("Found " + (if (gid.isEmpty) 0 else ">1") + " but by the earlier checks, " +
//                                                                "there should be exactly one group in entity " + entityId + " .")
//        (rtgCountOnEntity, db.getGroupSize(gid.get, Some(false)))
//      }
//    }
    val subgroupsCountPrefix: String = {
      if (attrCount == 0) ""
      else if (attrCount == 1) ">"
      else ">>"
    }
    subgroupsCountPrefix
  }

  /** Returns None if user just wants out; a String (user's answer, not useful outside this method) if update was done..
    */
  def editGroupName(groupIn: Group): Option[String] = {
    // doesn't seem to make sense to ck for duplicate names here: the real identity depends on what it relates to, and dup names may be common.
    val ans = ui.askForString(Some(Array(relationToGroupNamePrompt)), None, Some(groupIn.getName))
    if (ans.isEmpty || ans.get.trim.length() == 0) {
      ui.displayText("Not updated.")
      None
    } else {
      groupIn.update(None, Some(ans.get.trim), None, None, None)
      ans
    }
  }

  def addEntityToGroup(groupIn: Group): Option[Long] = {
    val newEntityId: Option[Long] = {
      if (!groupIn.getMixedClassesAllowed) {
        if (groupIn.getSize == 0) {
          // adding 1st entity to this group, so:
          val leadingText = List("ADD ENTITY TO A GROUP (**whose class will set the group's enforced class, even if 'None'**):")
          val idWrapper: Option[IdWrapper] = chooseOrCreateObject(Some(leadingText), None, None, Controller.ENTITY_TYPE,
                                                                  containingGroupIn = Some(groupIn.getId))
          if (idWrapper.isDefined) {
            db.addEntityToGroup(groupIn.getId, idWrapper.get.getId)
            Some(idWrapper.get.getId)
          } else None
        } else {
          // it's not the 1st entry in the group, so add an entity using the same class as those previously added (or None as case may be).
          val entityClassInUse = groupIn.getClassId
          val idWrapper: Option[IdWrapper] = chooseOrCreateObject(None, None, None, Controller.ENTITY_TYPE, 0, entityClassInUse, limitByClassIn = true,
                                                                  containingGroupIn = Some(groupIn.getId))
          if (idWrapper.isEmpty) None
          else {
            val entityId = idWrapper.get.getId
            try {
              db.addEntityToGroup(groupIn.getId, entityId)
              Some(entityId)
            } catch {
              case e: Exception =>
                if (e.getMessage.contains(PostgreSQLDatabase.MIXED_CLASSES_EXCEPTION)) {
                  val oldClass: String = if (entityClassInUse.isEmpty) "(none)" else new EntityClass(db, entityClassInUse.get).getDisplayString
                  val newClassId = new Entity(db, entityId).getClassId
                  val newClass: String = if (newClassId.isEmpty || entityClassInUse.isEmpty) "(none)"
                  else new EntityClass(db,
                                       entityClassInUse.get).getDisplayString
                  ui.displayText("Adding an entity with class '" + newClass + "' to a group that doesn't allow mixed classes, " +
                                 "and which already has an entity with class '" + oldClass + "' generates an error. The program should have prevented this by" +
                                 " only showing entities with a matching class, but in any case the mismatched entity was not added to the group.")
                  None
                } else throw e
            }
          }
        }
      } else {
        val leadingText = List("ADD ENTITY TO A (mixed-class) GROUP")
        val idWrapper: Option[IdWrapper] = chooseOrCreateObject(Some(leadingText), None, None, Controller.ENTITY_TYPE,
                                                                containingGroupIn = Some(groupIn.getId))
        if (idWrapper.isDefined) {
          db.addEntityToGroup(groupIn.getId, idWrapper.get.getId)
          Some(idWrapper.get.getId)
        } else None
      }
    }

    newEntityId
  }

  def handleException(e: Throwable) {//eliminate this once other users are switched 2 next 1 [what did i mean by that? anything still2do, or elim cmt?]
    if (e.isInstanceOf[org.postgresql.util.PSQLException] || e.isInstanceOf[OmDatabaseException] ||
        throwableToString(e).contains("ERROR: current transaction is aborted, commands ignored until end of transaction block"))
    {
      db.rollbackTrans()
    }
    val ans = ui.askYesNoQuestion("An error occurred: \"" + e.getClass.getName + ": " + e.getMessage + "\".  If you can provide simple instructions to " +
                                  "reproduce it consistently, maybe it can be fixed.  Do you want to see the detailed output?")
    if (ans.isDefined && ans.get) {
      ui.displayText(throwableToString(e))
    }
  }

  def throwableToString(e: Throwable): String = {
    val stringWriter = new StringWriter()
    e.printStackTrace(new PrintWriter(stringWriter))
    stringWriter.toString
  }

  def chooseAmongEntities(containingEntities: util.ArrayList[(Long, Entity)]): Option[Entity] = {
    val leadingText = List[String]("Pick from menu, or an entity by letter")
    val choices: Array[String] = Array(listNextItemsPrompt)
    //(see comments at similar location in EntityMenu, as of this writing on line 288)
    val containingEntitiesNamesWithRelTypes: Array[String] = containingEntities.toArray.map {
                                                                                              case relTypeIdAndEntity: (Long, Entity) =>
                                                                                                val relTypeId: Long = relTypeIdAndEntity._1
                                                                                                val entity: Entity = relTypeIdAndEntity._2
                                                                                                val relTypeName: String = new RelationType(db,
                                                                                                                                           relTypeId).getName
                                                                                                "the entity \"" + entity.getName + "\" " +
                                                                                                relTypeName + " this group"
                                                                                              // other possible displays:
                                                                                              //1) entity.getName + " - " + relTypeName + " this
                                                                                              // group"
                                                                                              //2) "entity " + entityName + " " +
                                                                                              //rtg.getDisplayString(maxNameLength, None, Some(rt))
                                                                                              case _ => throw new OmException("??")
                                                                                            }
    val ans = ui.askWhich(Some(leadingText.toArray), choices, containingEntitiesNamesWithRelTypes)
    if (ans.isEmpty) None
    else {
      val answer = ans.get
      if (answer == 1 && answer <= choices.length) {
        // see comment above
        ui.displayText("not yet implemented")
        None
      } else if (answer > choices.length && answer <= (choices.length + containingEntities.size)) {
        // those in the condition on the previous line are 1-based, not 0-based.
        val index = answer - choices.length - 1
        // user typed a letter to select.. (now 0-based); selected a new object and so we return to the previous menu w/ that one displayed &
        // current
        Some(containingEntities.get(index)._2)
      } else {
        ui.displayText("unknown response")
        None
      }
    }
  }

  /** Cloned from askForDate; see its comments in the code.
    * Idea: consider combining somehow with method askForDateAttributeValue.
    * @return None if user wants out.
    */
  @tailrec final def askForDate_generic(promptTextIn: Option[String] = None, defaultIn: Option[String]): Option[Long] = {
    val leadingText: Array[String] = Array(promptTextIn.getOrElse(genericDatePrompt))
    val default: String = defaultIn.getOrElse(Controller.DATEFORMAT.format(System.currentTimeMillis()))
    val ans = ui.askForString(Some(leadingText), None, Some(default))
    if (ans.isEmpty) None
    else {
      val dateStr = ans.get.trim
      val (newDate: Option[Long], retry: Boolean) = finishAndParseTheDate(dateStr)
      if (retry) askForDate_generic(promptTextIn, defaultIn)
      else newDate
    }
  }

  def removeEntityReferenceFromGroup_Menu(entityIn: Entity, containingGroupIn: Option[Group]): Boolean = {
    val groupCount: Long = db.getCountOfGroupsContainingEntity(entityIn.getId)
    val (entityCountNonArchived, entityCountArchived) = db.getCountOfEntitiesContainingEntity(entityIn.getId)
    val ans = ui.askYesNoQuestion("REMOVE this entity from that group: ARE YOU SURE? (This isn't a deletion: the entity can still be found by searching, and " +
                                  "is " + getContainingEntitiesDescription(entityCountNonArchived, entityCountArchived) +
                                  (if (groupCount > 1) ", and will still be in " + (groupCount - 1) + " group(s).)" else ""),
                                  Some(""))
    if (ans.isDefined && ans.get) {
      containingGroupIn.get.removeEntity(entityIn.getId)
      true

      //is it ever desirable to keep the next line instead of the 'None'? not in most typical usage it seems, but?:
      //entityMenu(startingAttributeIndexIn, entityIn, relationSourceEntityIn, relationIn)
    } else {
      ui.displayText("Did not remove entity from that group.", waitForKeystroke = false)
      false

      //is it ever desirable to keep the next line instead of the 'None'? not in most typical usage it seems, but?:
      //entityMenu(startingAttributeIndexIn, entityIn, relationSourceEntityIn, relationIn, containingGroupIn)
    }
  }

  def getPublicStatusDisplayString(entityIn: Entity): String = {
    //idea: maybe this (logic) knowledge really belongs in the TextUI class. (As some others, probably.)
    if (showPublicPrivateStatusPreference.getOrElse(false)) {
      val s = entityIn.getPublicStatusDisplayString(blankIfUnset = false)
      if (s == Entity.PRIVACY_PUBLIC) {
        Color.green(s)
      } else if (s == Entity.PRIVACY_NON_PUBLIC) {
        Color.yellow(s)
      } else {
        s
      }
    } else {
      ""
    }
  }

  /**
   * @param attrFormIn Contains the result of passing the right Controller.<string constant> to db.getAttributeFormId (SEE ALSO COMMENTS IN
   *                   EntityMenu.addAttribute).  BUT, there are also cases
   *                   where it is a # higher than those found in db.getAttributeFormId, and in that case is handled specially here.
   * @return None if user wants out (or attrFormIn parm was an abortive mistake?), and the created Attribute if successful.
   */
  def addAttribute(entityIn: Entity, startingAttributeIndexIn: Int, attrFormIn: Int, attrTypeIdIn: Option[Long]): Option[Attribute] = {
    val (attrTypeId: Long, askForAttrTypeId: Boolean) = {
      if (attrTypeIdIn.isDefined) {
        (attrTypeIdIn.get, false)
      } else {
        (0L, true)
      }
    }
    if (attrFormIn == PostgreSQLDatabase.getAttributeFormId("quantityattribute")) {
      def addQuantityAttribute(dhIn: QuantityAttributeDataHolder): Option[QuantityAttribute] = {
        Some(entityIn.addQuantityAttribute(dhIn.attrTypeId, dhIn.unitId, dhIn.number, dhIn.validOnDate, dhIn.observationDate))
      }
      askForInfoAndAddAttribute[QuantityAttributeDataHolder](new QuantityAttributeDataHolder(attrTypeId, None, 0, 0, 0), askForAttrTypeId, Controller.QUANTITY_TYPE,
                                                             Some(quantityTypePrompt), askForQuantityAttributeNumberAndUnit, addQuantityAttribute)
    } else if (attrFormIn == PostgreSQLDatabase.getAttributeFormId("dateattribute")) {
      def addDateAttribute(dhIn: DateAttributeDataHolder): Option[DateAttribute] = {
        Some(entityIn.addDateAttribute(dhIn.attrTypeId, dhIn.date))
      }
      askForInfoAndAddAttribute[DateAttributeDataHolder](new DateAttributeDataHolder(attrTypeId, 0), askForAttrTypeId, Controller.DATE_TYPE,
                                                         Some("SELECT TYPE OF DATE: "), askForDateAttributeValue, addDateAttribute)
    } else if (attrFormIn == PostgreSQLDatabase.getAttributeFormId("booleanattribute")) {
      def addBooleanAttribute(dhIn: BooleanAttributeDataHolder): Option[BooleanAttribute] = {
        Some(entityIn.addBooleanAttribute(dhIn.attrTypeId, dhIn.boolean))
      }
      askForInfoAndAddAttribute[BooleanAttributeDataHolder](new BooleanAttributeDataHolder(attrTypeId, Some(0), 0, false), askForAttrTypeId,
                                                            Controller.BOOLEAN_TYPE, Some("SELECT TYPE OF TRUE/FALSE VALUE: "),  askForBooleanAttributeValue,
                                                            addBooleanAttribute)
    } else if (attrFormIn == PostgreSQLDatabase.getAttributeFormId("fileattribute")) {
      def addFileAttribute(dhIn: FileAttributeDataHolder): Option[FileAttribute] = {
        Some(entityIn.addFileAttribute(dhIn.attrTypeId, dhIn.description, new File(dhIn.originalFilePath)))
      }
      val result: Option[FileAttribute] = askForInfoAndAddAttribute[FileAttributeDataHolder](new FileAttributeDataHolder(attrTypeId, "", ""),
                                                                                             askForAttrTypeId, Controller.FILE_TYPE,
                                                                                             Some("SELECT TYPE OF FILE: "), askForFileAttributeInfo,
                                                                                             addFileAttribute).asInstanceOf[Option[FileAttribute]]
      if (result.isDefined) {
        val ans = ui.askYesNoQuestion("Document successfully added. Do you want to DELETE the local copy (at " + result.get.getOriginalFilePath + " ?")
        if (ans.isDefined && ans.get) {
          if (!new File(result.get.getOriginalFilePath).delete()) {
            ui.displayText("Unable to delete file at that location; reason unknown.  You could check the permissions.")
          }
        }
      }
      result
    } else if (attrFormIn == PostgreSQLDatabase.getAttributeFormId("textattribute")) {
      def addTextAttribute(dhIn: TextAttributeDataHolder): Option[TextAttribute] = {
        Some(entityIn.addTextAttribute(dhIn.attrTypeId, dhIn.text, dhIn.validOnDate, dhIn.observationDate))
      }
      askForInfoAndAddAttribute[TextAttributeDataHolder](new TextAttributeDataHolder(attrTypeId, Some(0), 0, ""), askForAttrTypeId, Controller.TEXT_TYPE,
                                                         Some("SELECT TYPE OF " + textDescription + ": "), askForTextAttributeText, addTextAttribute)
    } else if (attrFormIn == PostgreSQLDatabase.getAttributeFormId("relationtoentity")) {
      def addRelationToEntity(dhIn: RelationToEntityDataHolder): Option[RelationToEntity] = {
        Some(entityIn.addRelationToEntity(dhIn.attrTypeId, dhIn.entityId2, dhIn.validOnDate, dhIn.observationDate))
      }
      askForInfoAndAddAttribute[RelationToEntityDataHolder](new RelationToEntityDataHolder(attrTypeId, None, 0, 0), askForAttrTypeId, Controller.RELATION_TYPE_TYPE,
                                                            Some("CREATE OR SELECT RELATION TYPE: (" + mRelTypeExamples + ")"),
                                                            askForRelationEntityIdNumber2, addRelationToEntity)
    } else if (attrFormIn == 100) {
      // re "100": see comments at attrFormIn above
      val eId: Option[IdWrapper] = askForNameAndSearchForEntity
      if (eId.isDefined) {
        Some(entityIn.addHASRelationToEntity(eId.get.getId, None, System.currentTimeMillis))
      } else {
        None
      }
    } else if (attrFormIn == PostgreSQLDatabase.getAttributeFormId("relationtogroup")) {
      def addRelationToGroup(dhIn: RelationToGroupDataHolder): Option[RelationToGroup] = {
        require(dhIn.entityId == entityIn.getId)
        val newRTG: RelationToGroup = entityIn.addRelationToGroup(dhIn.attrTypeId, dhIn.groupId, dhIn.validOnDate, dhIn.observationDate)
        Some(newRTG)
      }
      val result: Option[Attribute] = askForInfoAndAddAttribute[RelationToGroupDataHolder](new RelationToGroupDataHolder(entityIn.getId, attrTypeId, 0,
                                                                                                                         None, System.currentTimeMillis()),
                                                                                           askForAttrTypeId, Controller.RELATION_TYPE_TYPE,
                                                                                           Some("CREATE OR SELECT RELATION TYPE: (" + mRelTypeExamples + ")" +
                                                                                                "." + TextUI.NEWLN + "(Does anyone see a specific " +
                                                                                                "reason to keep asking for these dates?)"),
                                                                                           askForRelToGroupInfo, addRelationToGroup)
      if (result.isEmpty) {
        None
      } else {
        val newRtg = result.get.asInstanceOf[RelationToGroup]
        new GroupMenu(ui, db, this).groupMenu(new Group(db, newRtg.getGroupId), 0, Some(newRtg), None, Some(entityIn))
        result
      }
    } else if (attrFormIn == 101  /*re "101": see comments at attrFormIn above*/) {
      val newEntityName: Option[String] = ui.askForString(Some(Array {"Enter a name (or description) for this web page or other URI"}))
      if (newEntityName.isEmpty || newEntityName.get.isEmpty) return None

      val ans1 = ui.askWhich(Some(Array[String]("Do you want to enter the URI via the keyboard (normal) or the" +
                                                " clipboard (faster sometimes)?")), Array("keyboard", "clipboard"))
      if (ans1.isEmpty) return None
      val keyboardOrClipboard1 = ans1.get
      val uri: String = if (keyboardOrClipboard1 == 1) {
        val text = ui.askForString(Some(Array("Enter the URI:")))
        if (text.isEmpty || text.get.isEmpty) return None else text.get
      } else {
        val uriReady = ui.askYesNoQuestion("Put the url on the system clipboard, then Enter to continue (or hit ESC or answer 'n' to get out)", Some("y"))
        if (uriReady.isEmpty || !uriReady.get) return None
        Controller.getClipboardContent
      }

      val ans2 = ui.askWhich(Some(Array[String]("Do you want to enter a quote from it, via the keyboard (normal) or the" +
                                                " clipboard (faster sometimes, especially if it's multiline)? Or, ESC to not enter a quote.")),
                             Array("keyboard", "clipboard"))
      val quote: Option[String] = if (ans2.isEmpty) {
        None
      } else {
        val keyboardOrClipboard2 = ans2.get
        if (keyboardOrClipboard2 == 1) {
          val text = ui.askForString(Some(Array("Enter the quote")))
          if (text.isEmpty || text.get.isEmpty) return None else text
        } else {
          val clip = ui.askYesNoQuestion("Put a quote on the system clipboard, then Enter to continue (or answer 'n' to get out)", Some("y"))
          if (clip.isEmpty || !clip.get) return None
          Some(Controller.getClipboardContent)
        }
      }
      val quoteInfo = if (quote.isEmpty) "" else "For this text: \n  " + quote.get + "\n...and, "

      val proceedAnswer = ui.askYesNoQuestion(quoteInfo + "...for this name & URI:\n  " + newEntityName.get + "\n  " + uri + "" +
                                              "\n...: do you want to save them?", Some("y"))
      if (proceedAnswer.isEmpty || !proceedAnswer.get) return None

      //NOTE: the attrTypeId parm is ignored here since it is always a particular one for URIs:
      val (newEntity: Entity, newRTE: RelationToEntity) = db.addUriEntityWithUriAttribute(entityIn, newEntityName.get, uri, System.currentTimeMillis(),
                                                                                          entityIn.getPublic, callerManagesTransactionsIn = false, quote)

      new EntityMenu(ui, db, this).entityMenu(newEntity, containingRelationToEntityIn = Some(newRTE))

      Some(newRTE)
    } else {
      ui.displayText("invalid response")
      None
    }
  }

  def defaultAttributeCopying(entityIn: Entity, attributeTuplesIn: Option[Array[(Long, Attribute)]] = None): Unit = {
    if (shouldTryAddingDefaultAttributes(entityIn)) {
      val attributeTuples: Array[(Long, Attribute)] = if (attributeTuplesIn.isDefined) attributeTuplesIn.get else db.getSortedAttributes(entityIn.getId)._1
      val templateAttributesToCopy: ArrayBuffer[Attribute] = getMissingAttributes(entityIn.getClassDefiningEntityId, attributeTuples)
      copyAndEditAttributes(entityIn, templateAttributesToCopy)
    }
  }

  def copyAndEditAttributes(entityIn: Entity, templateAttributesToCopyIn: ArrayBuffer[Attribute]): Unit = {
    // userWantsOut is used like a break statement below: could be replaced with a functional idiom (see link to stackoverflow somewhere in the code).
    var userWantsOut = false
    for (templateAttribute: Attribute <- templateAttributesToCopyIn) {
      if (!userWantsOut) {
        ui.displayText("Edit the copied " + PostgreSQLDatabase.getAttributeFormName(templateAttribute.getFormId) + " \"" +
                       templateAttribute.getDisplayString(0, None, None, simplify = true) + " \", from the archetype entity (ESC to abort):",
                       waitForKeystroke = false)
        val newAttribute: Option[Attribute] = {
          templateAttribute match {
            case a: QuantityAttribute => Some(entityIn.addQuantityAttribute(a.getAttrTypeId, a.getUnitId, a.getNumber))
            case a: DateAttribute => Some(entityIn.addDateAttribute(a.getAttrTypeId, a.getDate))
            case a: BooleanAttribute => Some(entityIn.addBooleanAttribute(a.getAttrTypeId, a.getBoolean))
            case a: FileAttribute =>
              ui.displayText("You can add a FileAttribute manually afterwards for this attribute.  Maybe it can be automated " +
                                                    "more, when use cases for this part are more clear.")
              None
            case a: TextAttribute => Some(entityIn.addTextAttribute(a.getAttrTypeId, a.getText))
            case a: RelationToEntity =>
              val dh: Option[RelationToEntityDataHolder] = askForRelationEntityIdNumber2(new RelationToEntityDataHolder(0, None, 0, 0), inEditing = false)
              if (dh.isDefined) {
                Some(entityIn.addRelationToEntity(a.getAttrTypeId, dh.get.entityId2))
              } else {
                None
              }
            case a: RelationToGroup =>
              val dh: Option[RelationToGroupDataHolder] = askForRelToGroupInfo(new RelationToGroupDataHolder(0, 0, 0, None, 0))
              if (dh.isDefined) {
                Some(entityIn.addRelationToGroup(a.getAttrTypeId, dh.get.groupId))
              } else {
                None
              }
            case _ => throw new OmException("Unexpected type: " + templateAttribute.getClass.getCanonicalName)
          }
        }
        if (newAttribute.isDefined) {
          userWantsOut = editAttributeOnSingleLine(newAttribute.get)
          if (userWantsOut) {
            // That includes a "never mind" intention on the last one added (just above), so:
            newAttribute.get.delete()
          }
        }
      }
    }
  }

  def getMissingAttributes(classDefiningEntityIdIn: Option[Long], attributeTuplesIn: Array[(Long, Attribute)]): ArrayBuffer[Attribute] = {
    val templateAttributesToSuggestCopying: ArrayBuffer[Attribute] = {
      // This determines which attributes from the template entity (or "pattern" or class-defining entity) are not found on this entity, so they can
      // be added if the user wishes.
      val attributesToSuggestCopying_workingCopy: ArrayBuffer[Attribute] = new ArrayBuffer()
      if (classDefiningEntityIdIn.isDefined) {
        // ("cde" in name means "classDefiningEntity")
        val (cde_attributeTuples: Array[(Long, Attribute)], _) = db.getSortedAttributes(classDefiningEntityIdIn.get)
        for (cde_attributeTuple <- cde_attributeTuples) {
          var attributeTypeFoundOnEntity = false
          val cde_attribute = cde_attributeTuple._2
          for (attributeTuple <- attributeTuplesIn) {
            if (!attributeTypeFoundOnEntity) {
              val cde_typeId: Long = cde_attribute.getAttrTypeId
              val typeId = attributeTuple._2.getAttrTypeId
              if (cde_typeId == typeId) {
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

  def shouldTryAddingDefaultAttributes(entityIn: Entity): Boolean = {
    if (entityIn.getClassId.isEmpty) {
      false
    } else {
      val createAttributes: Option[Boolean] = db.getClassCreateDefaultAttributes(entityIn.getClassId.get)
      if (createAttributes.isDefined) {
        createAttributes.get
      } else {
        if (entityIn.getClassDefiningEntityId.isEmpty) {
          false
        } else {
          val attrCount = new Entity(db, entityIn.getClassDefiningEntityId.get).getAttrCount
          if (attrCount == 0) {
            false
          } else {
            val addAttributesAnswer = ui.askYesNoQuestion("Add attributes to this entity as found on the class-defining entity (template)?",
                                                          Some("y"), allowBlankAnswer = true)
            addAttributesAnswer.isDefined && addAttributesAnswer.get
          }
        }
      }
    }
  }

}
