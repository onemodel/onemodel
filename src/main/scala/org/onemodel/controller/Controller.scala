/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003-2004 and 2008-2015 inclusive, Luke A Call; all rights reserved.
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
import java.util

import org.apache.commons.io.FilenameUtils
import org.onemodel._
import org.onemodel.database.PostgreSQLDatabase
import org.onemodel.model._
import org.postgresql.util.PSQLException

import scala.annotation.tailrec

object Controller {
  // Might not be the most familiar date form for us Americans, but it seems the most useful in the widest
  // variety of situations, and more readable than with the "T" embedded in place of
  // the 1st space.  So, this approximates iso-9601.
  // these are for input.
  val DATEFORMAT = new java.text.SimpleDateFormat("yyyy-MM-dd HH:mm:ss:SSS zzz")
  val DATEFORMAT_WITH_ERA = new java.text.SimpleDateFormat("GGyyyy-MM-dd HH:mm:ss:SSS zzz")

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
}

/** Improvements to this class should START WITH MAKING IT BETTER TESTED (functional testing? integration? see
  * scalatest docs 4 ideas, & maybe use expect?), delaying side effects more, shorter methods, other better scala style, etc.
  */
class Controller(val ui: TextUI, forceUserPassPromptIn: Boolean = false, defaultUsernameIn: Option[String] = None, defaultPasswordIn: Option[String] = None) {
  protected val mRelTypeExamples = "i.e., ownership of or \"has\" another entity, family ties, etc"

  //%%
  //idea: get more scala familiarity then change this so it has limited visibility/scope: like, protected (subclass instances) + ImportExportTest.
  val mDB: PostgreSQLDatabase = tryLogins(forceUserPassPromptIn, defaultUsernameIn, defaultPasswordIn)

  // get default entity ID from user preferences; try to use i:
  protected val mPrefs = java.util.prefs.Preferences.userNodeForPackage(this.getClass)
  // (the startup message already suggests that they create it with their own name, no need to repeat that here:    )
  protected val menuText_createEntityOrAttrType: String = "Add new entity (or new quantity, true/false, date, text or file type to use as an attribute)"
  protected val menuText_CreateRelationType: String = "Add new relation type (" + mRelTypeExamples + ")"

  // date stuff
  val VALID = "valid"
  val OBSERVED = "observed"
  val genericDatePrompt: String = "Please enter the date (like this, w/ at least the year \"2013-01-31 23:59:59:999 MDT\"; zeros are " +
                                  "allowed in all but the yyyy-mm-dd).  Or press Enter (blank) for \"now\"; ESC to exit.  \"BC\" or \"AD\" prefix allowed " +
                                  "(before the year, with no space)."
  val tooLongMessage = "value too long for type"

  // ****** MAKE SURE THE NEXT 2 LINES MATCH THE FORMAT of the STRING ABOVE, AND THE USER EXAMPLES IN THIS CLASS' OUTPUT! ******
  // (i.e. + cur't time zone, checked at each call.)
  var timezone: String = new java.text.SimpleDateFormat("zzz").format(0)

  // (This isn't intended to match the date represented by a long value of "0", but is intended to be a usable value to fill in the rest of whatever a user
  // doesn't.  Perhaps assuming that the user will always put in a year if they put in anything (as currently enforced by the code at this time of writing).
  // Also, making this a var so that it can be changed for testing consistency (to use GMT for most tests so hopefully they will pass for developers in
  // another time zone.  idea:  It seems like there's a better way to solve that though, maybe with a subclass of Controller in the test,
  // or of SimpleDateFormat.)
  def blankDate = "1970-01-01 00:00:00:000 " + timezone

  val mCopyright = {
    var all = ""
    var append = false;
    var beforeAnyDashes = true;
    try {
      for (line <- scala.io.Source.fromFile("LICENSE").getLines) {
        //println(line)
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
          Unit
        }
      }
    }
    catch {
      case e: java.io.FileNotFoundException =>
        ui.displayText("The file LICENSE is missing from the distribution of this program; please correct that and be aware of the license.")
    }
    all
  }

  /** Returns the id and the entity, if they are available from the preferences lookup (id) and then finding that in the db (Entity). */
  def getDefaultEntity: (Option[Long], Option[Entity]) = {
    val defaultEntityId = findDefaultDisplayEntity
    if (defaultEntityId == None) (None, None)
    else (defaultEntityId, Entity.getEntityById(mDB, defaultEntityId.get))
  }

  def start() {
    // wait for keystroke so they do see the copyright each time.  Idea (is also tracked):  make it save their answer 'yes/i agree' or such in the DB,
    // and don't make them press the keystroke again (timesaver)!
    ui.displayText(mCopyright, waitForKeystroke = true, Some("If you do not agree to those terms: Press Ctrl-C or close the window to exit.\n" +
                                                             "If you agree to those terms: "))
    // Max id used as default here because it seems the least likely # to be used in the system hence the
    // most likely to cause an error as default by being missing, so the system can respond by prompting
    // the user in some other way for a use.
    if (getDefaultEntity._1 == None) {
      ui.displayText("Unable to find user's preference for first entity to display, or that entity is gone.  You probably will want to find or create an " +
                     "entity (such as with your own name to track information connected to you, contacts, possessions etc, " +
                     "or with the subject of study) then set that or some entity as your default, using its menu.")
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
      new MainMenu(ui, mDB).mainMenu(getDefaultEntity._2, goDirectlyToChoice)
      menuLoop()
    }
    menuLoop(Some(5))
  }

  /** If the 1st parm is true, the next 2 must be omitted or None. */
  private def tryLogins(forceUserPassPromptIn: Boolean = false, defaultUsernameIn: Option[String] = None,
                        defaultPasswordIn: Option[String] = None): PostgreSQLDatabase = {

    require(if (forceUserPassPromptIn) defaultUsernameIn == None && defaultPasswordIn == None else true)

    // tries the system username, blank password, & if that doesn't work, prompts user.
    @tailrec def tryOtherLoginsOrPrompt(): PostgreSQLDatabase = {
      val db = {
        var pwdOpt: Option[String] = None
        // try logging in with some obtainable default values first, to save user the trouble, like if pwd is blank
        val systemUserName = System.getProperty("user.name")
        val dbWithSystemNameBlankPwd = login(systemUserName, "", showError = false)
        if (None != dbWithSystemNameBlankPwd) dbWithSystemNameBlankPwd
        else {
          val usrOpt = ui.askForString(Some(Array("Username")), None, Some(systemUserName))
          if (None == usrOpt) System.exit(1)
          val dbConnectedWithBlankPwd = login(usrOpt.get, "", showError = false)
          if (dbConnectedWithBlankPwd != None) dbConnectedWithBlankPwd
          else {
            try {
              pwdOpt = ui.askForString(Some(Array("Password")), None, None, inIsPassword = true)
              if (pwdOpt == None) System.exit(1)
              val dbWithUserEnteredPwd = login(usrOpt.get, pwdOpt.get, showError = true)
              dbWithUserEnteredPwd
            } finally {
              if (pwdOpt != None) {
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
      if (None == db) {
        ui.displayText("Login failed; retrying (^C to quit):", waitForKeystroke = false)
        tryOtherLoginsOrPrompt()
      }
      else db.get
    }

    if (forceUserPassPromptIn) {
      @tailrec def loopPrompting: PostgreSQLDatabase = {
        val usrOpt = ui.askForString(Some(Array("Username")))
        if (None == usrOpt) System.exit(1)

        val pwdOpt = ui.askForString(Some(Array("Password")), None, None, inIsPassword = true)
        if (pwdOpt == None) System.exit(1)

        val dbWithUserEnteredPwd: Option[PostgreSQLDatabase] = login(usrOpt.get, pwdOpt.get, showError = false)
        if (dbWithUserEnteredPwd != None) dbWithUserEnteredPwd.get
        else loopPrompting
      }
      loopPrompting
    } else if (defaultUsernameIn != None && defaultPasswordIn != None) {
      // idea: perhaps this could be enhanced and tested to allow a username parameter, but prompt for a password, if/when need exists.
      val db = login(defaultUsernameIn.get, defaultPasswordIn.get, showError = true)
      if (db == None) {
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

  protected def findDefaultDisplayEntity: Option[Long] = {
    //idea: move this from prefs into an auto-created (always there) first/"System" object (predictable location: lowest id?),
    //which has a certain "defaultEntity" attribute always, which is not null if user has set a pref. (and, allow user to delete the pref?);
    //that also sounds more easily testable using current strategies.
    val first = mPrefs.get("first_display_entity", null)
    if (first == null) None
    else {
      try {
        if (!mDB.entityKeyExists(first.toLong)) None
        else Some(first.toLong)
      } catch {
        case e: java.lang.NumberFormatException =>
          ui.displayText("There is non-numeric value (" + first + ") in a file located somewhere like ~/.java/.userPrefs/org/onemodel/prefs.xml. You might " +
                         "want to fix that, delete it, or re-save your default entity using the entity menu. Proceeding without it.")
          None
      }
    }
  }

  protected def askForInfoAndCreateEntity(inClassId: Option[Long] = None): Option[Entity] = {
    var newClass = false
    val classId: Option[Long] =
      if (inClassId != None) inClassId
      else {
        val idWrapper: Option[IdWrapper] = chooseOrCreateObject(Some(List[String]("CHOOSE ENTITY'S CLASS (entity template; ESC for None)")), None, None,
                                                                Controller.ENTITY_CLASS_TYPE)
        newClass = true
        if (idWrapper == None) None
        else Some(idWrapper.get.getId)
      }
    val ans: Option[Entity] = askForNameAndWriteEntity(Controller.ENTITY_TYPE, None, None, None, None, classId,
                                                       Some(if (newClass) "DEFINE THE ENTITY:" else ""))
    if (ans != None) {
      val entity = ans.get
      // idea: (is also on fix list): this needs to be removed, after evaluating for other side effects, to fix the bug
      // where creating a new relationship, and creating the entity2 in the process, it puts the wrong info
      // on the header for what is being displayed/edited next!: Needs refactoring anyway: this shouldn't be at
      // a low level.
      ui.displayText("Created " + Controller.ENTITY_TYPE + ": " + entity.getName, waitForKeystroke = false)
      Some(entity)
    } else {
      None
    }
  }

  protected def showInEntityMenuThenMainMenu(entityIn: Option[Entity]) {
    if (entityIn != None) {
      //idea: is there a better way to do this, maybe have a single entityMenu for the class instead of new.. each time?
      new EntityMenu(ui, mDB).entityMenu(0, entityIn.get)
      // doing mainmenu right after entityMenu because that's where user would
      // naturally go after they exit the entityMenu.
      new MainMenu(ui, mDB).mainMenu(entityIn)
    }
  }

  /** Returns None if user wants out.
    * 2nd parameter should be None only if the call is intended to create; otherwise it is an edit.
    * The "previous..." parameters are for the already-existing data (ie, when editing not creating).
    */
  protected def askForNameAndWriteEntity(inType: String, existingIdIn: Option[Long] = None,
                                         previousNameIn: Option[String] = None, previousDirectionalityIn: Option[String] = None,
                                         previousNameInReverseIn: Option[String] = None, inClassId: Option[Long] = None,
                                         inLeadingText: Option[String] = None): Option[Entity] = {
    if (inClassId != None) require(inType == Controller.ENTITY_TYPE)
    val createNotUpdate: Boolean = existingIdIn == None
    if (!createNotUpdate && inType == Controller.RELATION_TYPE_TYPE) require(previousDirectionalityIn != None)
    val maxNameLength = {
      if (inType == Controller.RELATION_TYPE_TYPE) model.RelationType.getNameLength(mDB)
      else if (inType == Controller.ENTITY_TYPE) model.Entity.nameLength(mDB)
      else throw new Exception("invalid inType: " + inType)
    }
    val example = {
      if (inType == Controller.RELATION_TYPE_TYPE) " (use 3rd-person verb like \"owns\"--might make output like sentences more consistent later on)"
      else ""
    }

    val potentialErrorMsg = "Got an error: %s.  Please try a shorter (" + maxNameLength + " chars) entry.  Details: "

    /** 2nd Long in return value is ignored in this particular case.
      */
    def askAndSave(defaultNameIn: Option[String] = None): Option[(Long, Long)] = {
      val nameOpt = ui.askForString(Some(Array[String](inLeadingText.getOrElse(""),
                                                       "Enter " + inType + " name (up to " + maxNameLength + " characters" + example + "; ESC to cancel)")),
                                    None, defaultNameIn)
      if (nameOpt == None) None
      else {
        val name = nameOpt.get.trim()
        if (name.size <= 0) None
        else {
          var duplicate = false
          if (model.Entity.isDuplicate(mDB, name, existingIdIn)) {
            val answerOpt = ui.askForString(Some(Array("That name is a duplicate--proceed anyway? (y/n)")), None, Some("n"))
            if (answerOpt == None || (!answerOpt.get.equalsIgnoreCase("y"))) duplicate = true
          }
          // idea: this size check might be able to account better for the escaping that's done. Or just keep letting the exception handle it as is already
          // done in the caller of this.
          if (name.size > maxNameLength) {
            ui.displayText(potentialErrorMsg.format(tooLongMessage) + ".")
            askAndSave(Some(name))
          } else {
            if (duplicate) None
            else {
              if (inType == Controller.ENTITY_TYPE) {
                if (createNotUpdate) {
                  val newId = model.Entity.createEntity(mDB, name, inClassId).getId
                  Some(newId, 0L)
                } else {
                  mDB.updateEntityOnlyName(existingIdIn.get, name)
                  Some(existingIdIn.get, 0L)
                }
              } else if (inType == Controller.RELATION_TYPE_TYPE) {
                val ans: Option[String] = askForRelationDirectionality(previousDirectionalityIn)
                if (ans == None) None
                else {
                  val directionalityStr: String = ans.get.trim().toUpperCase
                  val nameInReverseDirectionStr = askForNameInReverseDirection(directionalityStr, maxNameLength, name, previousNameInReverseIn)
                  if (createNotUpdate) {
                    val newId = new RelationType(mDB, mDB.createRelationType(name, nameInReverseDirectionStr, directionalityStr)).getId
                    Some(newId, 0L)
                  } else {
                    mDB.updateRelationType(existingIdIn.get, name, nameInReverseDirectionStr, directionalityStr)
                    Some(existingIdIn.get, 0L)
                  }
                }
              } else throw new Exception("unexpected value: " + inType)
            }
          }
        }
      }
    }

    val result = tryAskingAndSaving(potentialErrorMsg, askAndSave, previousNameIn)
    if (result == None) None
    else Some(new Entity(mDB, result.get._1))
  }

  /** Call a provided function (method?) "askAndSaveIn", which does some work that might throw a specific OmDatabaseException.  If it does throw that,
    * let the user know the problem and call askAndSaveIn again.  I.e., allow retrying if the entered data is bad, instead of crashing the app.
    */
  protected def tryAskingAndSaving(errorMsgIn: String, askAndSaveIn: (Option[String]) => Option[(Long, Long)],
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
  protected def askForAndWriteClassAndDefiningEntityName(classIdIn: Option[Long] = None,
                                                         previousNameIn: Option[String] = None): Option[(Long, Long)] = {
    val createNotUpdate: Boolean = classIdIn == None
    val nameLength = model.EntityClass.nameLength(mDB)
    val potentialErrorMsg = "Got an error: %s.  Please try a shorter (" + nameLength + " chars) entry.  Details: "

    def askAndSave(defaultNameIn: Option[String]): Option[(Long, Long)] = {
      val nameOpt = ui.askForString(Some(Array("Enter class name (up to " + nameLength + " characters; will also be used for its defining entity name; ESC to" +
                                               " cancel): ")),
                                    None, defaultNameIn)
      if (nameOpt == None) None
      else {
        val name = nameOpt.get.trim()
        if (name.length() <= 0) None
        else {
          if (duplicationProblem(name, classIdIn, createNotUpdate)) None
          else {
            if (createNotUpdate) Some(mDB.createClassAndItsDefiningEntity(name))
            else {
              val entityId: Long = mDB.updateClassAndDefiningEntityName(classIdIn.get, name)
              Some(classIdIn.get, entityId)
            }
          }
        }
      }
    }

    tryAskingAndSaving(potentialErrorMsg, askAndSave)
  }


  def duplicationProblem(name: String, previousIdIn: Option[Long], createNotUpdate: Boolean): Boolean = {
    var duplicateProblemSoSkip = false
    if (EntityClass.isDuplicate(mDB, name, previousIdIn)) {
      val answerOpt = ui.askForString(Some(Array("That name is a duplicate--proceed anyway? (y/n)")), None, Some("n"))
      if (answerOpt == None || (!answerOpt.get.equalsIgnoreCase("y"))) duplicateProblemSoSkip = true
    }
    duplicateProblemSoSkip
  }

  @tailrec protected final def askForNameInReverseDirection(directionalityStrIn: String, nameLengthIn: Int, nameIn: String,
                                                            previousNameInReverseIn: Option[String] = None): String = {
    // see createTables (or UI prompts) for meanings of bi/uni/non...
    if (directionalityStrIn == "UNI") ""
    else if (directionalityStrIn == "NON") nameIn
    else if (directionalityStrIn == "BI") {
      // see createTables (or UI prompts) for meanings...
      val msg = Array("Enter relation name when direction is reversed (i.e., 'is husband to' becomes 'is wife to', 'employs' becomes 'is employed by' " +
                      "by; up to " + nameLengthIn + " characters (ESC to cancel): ")
      val nameInReverseOpt = {
        val ans = ui.askForString(Some(msg), None, previousNameInReverseIn)
        if (ans == None) None
        ans.get.trim() //see above comment about trim
      }
      val ans = ui.askWhich(Some(Array("Is this the correct name for the relationship in reverse direction?: ")), Array("Yes", "No"))
      if (ans == None || ans.get == 2) askForNameInReverseDirection(directionalityStrIn, nameLengthIn, nameIn, previousNameInReverseIn)
      else nameInReverseOpt
    }
    else throw new Exception("unexpected value for directionality: " + directionalityStrIn)
  }

  protected def askForRelationDirectionality(previousDirectionalityIn: Option[String] = None): Option[String] = {
    val msg = Array("Enter directionality (\"bi\", \"uni\", or \"non\"; examples: \"is parent of\"/\"is child of\" is bidirectional, " +
                    "since it differs substantially by the direction but goes both ways; unidirectional might be like 'lists': the thing listed doesn't know " +
                    "it; \"is acquaintanted with\" could be nondirectional if it is an identical relationship either way  (ESC to cancel): ")
    def criteria(entryIn: String): Boolean = {
      val entry = entryIn.trim().toUpperCase
      entry == "BI" || entry == "UNI" || entry == "NON"
    }

    val directionality = ui.askForString(Some(msg), Some(criteria(_: String)), previousDirectionalityIn)
    if (directionality == None) None
    else Some(directionality.get.toUpperCase)
  }

  val quantityDescription: String = "SELECT TYPE OF QUANTITY (type is length or volume, but not the measurement unit); ESC or leave both blank to cancel; " +
                                    "cancel if you need to create the needed type before selecting): "
  val textDescription: String = "TEXT (e.g., serial #)"


  /* NOTE: converting the parameters around here from DataHolder to Attribute... means also making the Attribute
  classes writable, and/or
     immutable and recreating them whenever there's a change, but also needing a way to pass around
     partial attribute data in a way that can be shared by code, like return values from the get[AttributeData...]
     methods.
     Need to learn more scala so I can do the equivalent of passing a Tuple without specifying the size in signatures?
   */
  protected def askForInfoAndUpdateAttribute[T <: AttributeDataHolder](inDH: T, attrType: String, promptForSelectingTypeId: String,
                                                                       getOtherInfoFromUser: (T, Boolean) => Option[T], updateTypedAttribute: (T) => Unit) {
    @tailrec def askForInfoAndUpdateAttribute_helper(dhIn: T, attrType: String, promptForTypeId: String) {
      val ans: Option[T] = askForAttributeData[T](dhIn, promptForTypeId, attrType, Some(new Entity(mDB, dhIn.attrTypeId).getName), Some(inDH.attrTypeId),
                                                  getOtherInfoFromUser, inEditing = true)
      if (ans != None) {
        val dhOut: T = ans.get
        val ans2: Option[Int] = promptWhetherTo1Add2Correct(attrType)

        if (ans2 == None) Unit
        else if (ans2.get == 1) {
          updateTypedAttribute(dhOut)
        }
        else if (ans2.get == 2) askForInfoAndUpdateAttribute_helper(dhOut, attrType, promptForTypeId)
        else throw new Exception("unexpected result! should never get here")
      }
    }
    askForInfoAndUpdateAttribute_helper(inDH, attrType, promptForSelectingTypeId)
  }

  @tailrec
  protected final def attributeEditMenu(attributeIn: Attribute): Option[Entity] = {
    val leadingText: Array[String] = Array("Attribute: " + attributeIn.getDisplayString(0, None, None))
    var firstChoices = Array("(stub, to make others consistent)",
                             "(stub)",
                             "Edit",
                             "Delete",
                             "Go to entity representing the type: " + new Entity(mDB, attributeIn.getAttrTypeId).getName)
    if (attributeIn.isInstanceOf[FileAttribute]) {
      firstChoices = firstChoices ++ Array[String]("Export the file")
    }
    val response = ui.askWhich(Some(leadingText), firstChoices)
    if (response == None) None
    else {
      val answer: Int = response.get
      if (answer == 3) {
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
                                                                      Controller.QUANTITY_TYPE, quantityDescription,
                                                                      askForQuantityAttributeNumberAndUnit,
                                                                      updateQuantityAttribute)
            //force a reread from the DB so it shows the right info on the repeated menu:
            attributeEditMenu(new QuantityAttribute(mDB, attributeIn.getId))
          case textAttribute: TextAttribute =>
            def updateTextAttribute(dhInOut: TextAttributeDataHolder) {
              textAttribute.update(dhInOut.attrTypeId, dhInOut.text, dhInOut.validOnDate, dhInOut.observationDate)
            }
            val textAttributeDH: TextAttributeDataHolder = new TextAttributeDataHolder(textAttribute.getAttrTypeId, textAttribute.getValidOnDate,
                                                                                       textAttribute.getObservationDate, textAttribute.getText)
            askForInfoAndUpdateAttribute[TextAttributeDataHolder](textAttributeDH, Controller.TEXT_TYPE,
                                                                  "CHOOSE TYPE OF " + textDescription + ":",
                                                                  askForTextAttributeText, updateTextAttribute)
            //force a reread from the DB so it shows the right info on the repeated menu:
            attributeEditMenu(new TextAttribute(mDB, attributeIn.getId))
          case dateAttribute: DateAttribute =>
            def updateDateAttribute(dhInOut: DateAttributeDataHolder) {
              dateAttribute.update(dhInOut.attrTypeId, dhInOut.date)
            }
            val dateAttributeDH: DateAttributeDataHolder = new DateAttributeDataHolder(dateAttribute.getAttrTypeId, dateAttribute.getDate)
            askForInfoAndUpdateAttribute[DateAttributeDataHolder](dateAttributeDH, Controller.DATE_TYPE, "CHOOSE TYPE OF DATE:",
                                                                  askForDateAttributeValue, updateDateAttribute)
            //force a reread from the DB so it shows the right info on the repeated menu:
            attributeEditMenu(new DateAttribute(mDB, attributeIn.getId))
          case booleanAttribute: BooleanAttribute =>
            def updateBooleanAttribute(dhInOut: BooleanAttributeDataHolder) {
              booleanAttribute.update(dhInOut.attrTypeId, dhInOut.boolean, dhInOut.validOnDate, dhInOut.observationDate)
            }
            val booleanAttributeDH: BooleanAttributeDataHolder = new BooleanAttributeDataHolder(booleanAttribute.getAttrTypeId, booleanAttribute.getValidOnDate,
                                                                                                booleanAttribute.getObservationDate,
                                                                                                booleanAttribute.getBoolean)
            askForInfoAndUpdateAttribute[BooleanAttributeDataHolder](booleanAttributeDH, Controller.BOOLEAN_TYPE,
                                                                     "CHOOSE TYPE OF TRUE/FALSE VALUE:",
                                                                     askForBooleanAttributeValue, updateBooleanAttribute)
            //force a reread from the DB so it shows the right info on the repeated menu:
            attributeEditMenu(new BooleanAttribute(mDB, attributeIn.getId))
          case fa: FileAttribute =>
            def updateFileAttribute(dhInOut: FileAttributeDataHolder) {
              fa.update(Some(dhInOut.attrTypeId), Some(dhInOut.description))
            }
            val fileAttributeDH: FileAttributeDataHolder = new FileAttributeDataHolder(fa.getAttrTypeId, fa.getDescription, fa.getOriginalFilePath)
            askForInfoAndUpdateAttribute[FileAttributeDataHolder](fileAttributeDH, Controller.FILE_TYPE, "CHOOSE TYPE OF FILE:",
                                                                  askForFileAttributeInfo, updateFileAttribute)
            //force a reread from the DB so it shows the right info on the repeated menu:
            attributeEditMenu(new FileAttribute(mDB, attributeIn.getId))
          case _ => throw new Exception("Unexpected type: " + attributeIn.getClass.getName)
        }
      }
      else if (answer == 4) {
        val ans = ui.askYesNoQuestion("DELETE this attribute: ARE YOU SURE?")
        if (ans != None && ans.get) {
          attributeIn.delete()
          // return None so it doesn't show again the menu of the object just deleted
          None
        } else {
          ui.displayText("Did not delete attribute.", waitForKeystroke = false)
        }
        attributeEditMenu(attributeIn)
      }
      else if (answer == 5) {
        new EntityMenu(ui, mDB).entityMenu(0, new Entity(mDB, attributeIn.getAttrTypeId))
        attributeEditMenu(attributeIn)
      }
      else if (answer == 6) {
        if (!attributeIn.isInstanceOf[FileAttribute]) throw new Exception("Menu shouldn't have allowed us to get here w/ a type other than FA (" +
                                                                          attributeIn.getClass.getName + ").")
        val fa = attributeIn.asInstanceOf[FileAttribute]
        try {
          // this file should be confirmed by the user as ok to write, even overwriting what is there.
          val file: Option[File] = ui.getExportDestinationFile(fa.getOriginalFilePath, fa.getMd5Hash)
          if (file != None) {
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


  def getReplacementFilename(originalFilePathIn: String): (String, String) = FileAttribute.getReplacementFilename(originalFilePathIn)

  protected def askForInfoAndAddAttribute[T <: AttributeDataHolder](inDH: T, attrType: String, promptForSelectingTypeId: String,
                                                                    getOtherInfoFromUser: (T, Boolean) => Option[T],
                                                                    addTypedAttribute: (T) => Option[Attribute]): Option[Attribute] = {
    val ans: Option[T] = askForAttributeData[T](inDH, promptForSelectingTypeId, attrType, None, None, getOtherInfoFromUser, inEditing = false)
    if (ans != None) {
      val dhOut: T = ans.get
      addTypedAttribute(dhOut)
    } else None
  }

  val entityPartsThatCanBeAffected: String = "ALL its attributes, actions, and relations, but not entities or groups the relations refer to"

  /** Returns whether entity was deleted.
    */
  def deleteOrArchiveEntity(entityIn: Entity, delNotArchive: Boolean): Boolean = {
    val name = entityIn.getName
    val groupCount: Long = mDB.getCountOfGroupsContainingEntity(entityIn.getId)
    val groupsPrompt = if (groupCount == 0) ""
    else {
      val limit = 10
      val delimiter = ", "
      // (%%BUG: see comments in psql.java re "OTHER ENTITY NOTED IN A DELETION BUG")
      val descrArray = mDB.getRelationToGroupDescriptionsContaining(entityIn.getId, Some(limit))
      var descriptions = ""
      var counter = 0
      for (s: String <- descrArray) {
        counter += 1
        descriptions += counter + ") " + s + delimiter
      }
      descriptions = descriptions.substring(0, descriptions.length - delimiter.length) + ".  "

      //removed next line because it doesn't make sense (& fails): there could be, for example, a single group that contains an
      //entity, but many entities that have a relation to that group:
      //require(descrArray.size == math.min(limit, groupCount))

      "This will ALSO remove it from " + (if (delNotArchive) "" else "visibility in ") + groupCount + " groups, " +
      "including for example these " + descrArray.size + " relations " +
      " that refer to this entity (showing entities & their relations to groups, as \"entity -> group\"): " + descriptions
    }
    // idea: WHEN CONSIDERING MODS TO THIS, ALSO CONSIDER THE Q'S ASKED AT CODE CMT WHERE DELETING A GROUP OF ENTITIES (SEE, for example "recursively").
    val ans = ui.askYesNoQuestion((if (delNotArchive) "DELETE" else "ARCHIVE") + " ENTITY \"" + name + "\" (and " + entityPartsThatCanBeAffected + ").  " +
                                  groupsPrompt +
                                  "**ARE YOU REALLY SURE?**")
    if (ans != None && ans.get) {
      if (delNotArchive) {
        entityIn.delete()
        ui.displayText("Deleted entity \"" + name + "\"" + ".")
      } else {
        entityIn.archive()
        ui.displayText("Archived entity \"" + name + "\"" + ".")
      }
      true
    }
    else {
      ui.displayText("Did not " + (if (delNotArchive) "delete" else "archive") + " entity.", waitForKeystroke = false)
      false
    }
  }

  val listNextItemsPrompt = "List next items"
  val relationToGroupNamePrompt = "Type a name for this group (e.g., \"xyz list\"), then press Enter; blank or ESC to cancel"

  protected def addRemainingCountToPrompt(choicesIn: Array[String], numDisplayedObjects: Long, totalRowsAvailableIn: Long,
                                          startingDisplayRowIndexIn: Long): Array[String] = {
    val numLeft = totalRowsAvailableIn - startingDisplayRowIndexIn - numDisplayedObjects
    val indexOfPrompt = choicesIn.indexOf(listNextItemsPrompt)
    if (numLeft > 0 && indexOfPrompt >= 0) {
      choicesIn(indexOfPrompt) = listNextItemsPrompt + " (of " + numLeft + " more)"
    }
    choicesIn
  }

  def editEntityName(entityIn: Entity): Option[Entity] = {
    val editedEntity: Option[Entity] = entityIn match {
      case relTypeIn: RelationType =>
        val previousNameInReverse: String = relTypeIn.getNameInReverseDirection //%%: this edits name w/ prefill also?:
        askForNameAndWriteEntity(Controller.RELATION_TYPE_TYPE, Some(relTypeIn.getId), Some(relTypeIn.getName), Some(relTypeIn.getDirectionality),
                                 if (previousNameInReverse == null || previousNameInReverse.trim().isEmpty) None else Some(previousNameInReverse),
                                 None)
      case entity: Entity =>
        val entityNameBeforeEdit: String = entityIn.getName
        val editedEntity: Option[Entity] = askForNameAndWriteEntity(Controller.ENTITY_TYPE, Some(entity.getId), Some(entity.getName), None, None, None)
        if (editedEntity != None) {
          val entityNameAfterEdit: String = editedEntity.get.getName
          if (entityNameBeforeEdit != entityNameAfterEdit) {
            val (_, groupId, moreThanOneAvailable) = mDB.findRelationToAndGroup_OnEntity(editedEntity.get.getId)
            if (groupId != None && !moreThanOneAvailable) {
              // for efficiency, if it's obvious which subgroup's name to change at the same time, offer to do so
              val ans = ui.askYesNoQuestion("There's a single subgroup with the same old name; probably it and this entity were created at the same time, " +
                                            "for the subgroup.  Change" +
                                            " the subgroup's name at the same time to be identical?", Some("y"))
              if (ans != None && ans.get) {
                val group = new Group(mDB, groupId.get)
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

  def editEntityPublicStatus(entityIn: Entity): Option[Entity] = {
    val editedEntity: Option[Entity] = entityIn match {
      //this doesn't seem to be working, doesn't get the exception. Maybe it's better without it, anyway.  Wouldn't matter when those 2 tables are broken apart.
      case relTypeIn: RelationType =>
        throw new OmException("shouldn't have got here: doesn't make sense to edit the public/nonpublic status of a RelationType (& those tables should be " +
                              "separated)")
      case entity: Entity =>
        val valueBeforeEdit: Option[Boolean] = entityIn.getPublic
        val valueAfterEdit: Option[Boolean] = ui.askYesNoQuestion("Enter yes/no value (or a space for 'unknown/unspecified'",
                                                                  if (valueBeforeEdit == None) Some("") else if (valueBeforeEdit.get) Some("y") else Some("n"),
                                                                  allowBlankAnswer = true)
        if (valueAfterEdit != valueBeforeEdit) {
          mDB.updateEntityOnlyPublicStatus(entity.getId, valueAfterEdit)
          // reload to reflect the latest data (immutability of entity objects & all; see comment near top of db.createTables)
          Some(new Entity(mDB, entity.getId))
        }
        else
          Some(entity)
      case _ => throw new Exception("??")
    }
    editedEntity
  }

  /**
   * @return None means "get out", or Some(choiceNum) if a choice was made.
   */
  def askWhetherDeleteOrArchiveEtc(entityIn: Entity, relationIn: Option[RelationToEntity], relationSourceEntityIn: Option[Entity],
                                   containingGroupIn: Option[Group]): (Option[Int], Int, Int) = {
    val leadingText = Some(Array("Choose a deletion or archiving option:"))
    var choices = Array("Delete this entity",
                        "Archive this entity (remove from visibility but not permanent/total deletion)")
    val delLinkingRelation_choiceNumber: Int = 3
    var delFromContainingGroup_choiceNumber: Int = 3
    if (relationIn != None) {
      // means we got here by selecting a Relation attribute on another entity, so entityIn is the "entityId2" in that relation; so show some options,
      // because
      // we eliminated a separate menu just for the relation and put them here, for UI usage simplicity.
      choices = choices :+ "Delete the linking relation between the linker: \"" + relationSourceEntityIn.get.getName + "\", " +
                           "and this Entity: \"" + entityIn.getName + "\""
      delFromContainingGroup_choiceNumber += 1
    }
    if (containingGroupIn != None) {
      choices = choices :+ "Delete the link between the group: \"" + containingGroupIn.get.getName + "\", and this Entity: \"" + entityIn.getName
    }

    val delOrArchiveAnswer: Option[(Int)] = ui.askWhich(leadingText, choices, Array[String]())
    (delOrArchiveAnswer, delLinkingRelation_choiceNumber, delFromContainingGroup_choiceNumber)
  }

  /** Returns None if user just wants out. */
  protected def promptWhetherTo1Add2Correct(inAttrTypeDesc: String): Option[Int] = {
    @tailrec def ask: Option[Int] = {
      val ans = ui.askWhich(None, Array("1-Save this " + inAttrTypeDesc + " attribute?", "2-Correct it?"))
      if (ans == None) return None
      val answer = ans.get
      if (answer < 1 || answer > 2) {
        ui.displayText("invalid response")
        ask
      } else Some(answer)
    }
    ask
  }

  /** Returns data, or None if user wants to cancel/get out. */
  protected def askForAttributeData[T <: AttributeDataHolder](inoutDH: T, prompt: String, attrType: String, inPreviousSelectionDesc: Option[String],
                                                              inPreviousSelectionId: Option[Long], askForOtherInfo: (T, Boolean) => Option[T],
                                                              inEditing: Boolean): Option[T] = {
    val ans: Option[Long] = askForAttributeTypeId(prompt, attrType, inPreviousSelectionDesc, inPreviousSelectionId)
    if (ans == None) None
    else {
      inoutDH.attrTypeId = ans.get
      val ans2: Option[T] = askForOtherInfo(inoutDH, inEditing)
      if (ans2 == None) None
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

  protected def askForAttributeTypeId(prompt: String, attrType: String, inPreviousSelectionDesc: Option[String],
                                      inPreviousSelectionId: Option[Long]): (Option[Long]) = {
    val attrTypeSelection = chooseOrCreateObject(Some(List(prompt)), inPreviousSelectionDesc: Option[String], inPreviousSelectionId: Option[Long], attrType)
    if (attrTypeSelection == None) {
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
  @tailrec protected final def findExistingObject(startingDisplayRowIndexIn: Long = 0, attrTypeIn: String,
                                                  idToOmitIn: Option[Long] = None, nameRegexIn: String): Option[IdWrapper] = {
    val leadingText = List[String]("SEARCH RESULTS: " + pickFromListPrompt)
    val choices: Array[String] = Array(listNextItemsPrompt)
    val numDisplayableItems = ui.maxColumnarChoicesToDisplayAfter(leadingText.size, choices.size, maxNameLength)

    val objectsToDisplay = attrTypeIn match {
      case Controller.ENTITY_TYPE =>
        mDB.getMatchingEntities(startingDisplayRowIndexIn, Some(numDisplayableItems), idToOmitIn, nameRegexIn)
      case Controller.GROUP_TYPE =>
        mDB.getMatchingGroups(startingDisplayRowIndexIn, Some(numDisplayableItems), idToOmitIn, nameRegexIn)
      case _ =>
        throw new OmException("??")
    }
    if (objectsToDisplay.size == 0) {
      ui.displayText("End of list, or none found; starting over from the beginning...")
      if (startingDisplayRowIndexIn == 0) None
      else findExistingObject(0, attrTypeIn, idToOmitIn, nameRegexIn)
    } else {
      val objectNames: Array[String] = objectsToDisplay.toArray.map {
                                                                      case entity: Entity =>
                                                                        val numSubgroupsPrefix: String = getNumSubgroupsPrefix(entity.getId)
                                                                        numSubgroupsPrefix + entity.getName
                                                                      case group: Group =>
                                                                        val numSubgroupsPrefix: String = getNumSubgroupsPrefix(group.getId)
                                                                        numSubgroupsPrefix + group.getName
                                                                      case x: Any => throw new Exception("unexpected class: " + x.getClass.getName)
                                                                      case _ => throw new OmException("??")
                                                                    }
      val ans = ui.askWhichChoiceOrItsAlternate(Some(leadingText.toArray), choices, objectNames)
      if (ans == None) None
      else {
        val (answer, userChoseAlternate: Boolean) = ans.get
        if (answer == 1 && answer <= choices.size) {
          // (For reason behind " && answer <= choices.size", see comment where it is used in entityMenu.)
          val nextStartingIndex: Long = startingDisplayRowIndexIn + objectsToDisplay.size
          findExistingObject(nextStartingIndex, attrTypeIn, idToOmitIn, nameRegexIn)
        } else if (answer > choices.length && answer <= (choices.length + objectsToDisplay.size)) {
          // those in the condition on the previous line are 1-based, not 0-based.
          var index = answer - choices.length - 1
          val o = objectsToDisplay.get(index)
          if (userChoseAlternate) {
            attrTypeIn match {
              // idea: replace this condition by use of a trait (the type of o, which has getId), or being smarter with scala's type system. attrTypeIn match {
              case Controller.ENTITY_TYPE =>
                new EntityMenu(ui, mDB).entityMenu(0, o.asInstanceOf[Entity])
              case Controller.GROUP_TYPE =>
                // for now, picking the first RTG found for this group, until it's clear which of its RTGs to use.
                // (see also the other locations w/ similar comment!)
                val someRelationToGroups: java.util.ArrayList[RelationToGroup] = mDB.getRelationToGroupsByGroup(o.asInstanceOf[Group].getId, 0, Some(1))
                new GroupMenu(ui, mDB).groupMenu(0, someRelationToGroups.get(0))
              case _ =>
                throw new OmException("??")
            }
            findExistingObject(startingDisplayRowIndexIn, attrTypeIn, idToOmitIn, nameRegexIn)
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
          findExistingObject(startingDisplayRowIndexIn, attrTypeIn, idToOmitIn, nameRegexIn)
        }
      }
    }
  }

  def searchPrompt(typeNameIn: String): String = {
    "Enter part of the " + typeNameIn + " name to search for.  (For the curious: it will be used in matching as a " +
    "case-insensitive POSIX " +
    "regex; details at  http://www.postgresql.org/docs/9.1/static/functions-matching.html#FUNCTIONS-POSIX-REGEXP .)"
  }

  /** Returns None if user wants out.  The parameter 'containingGroupIn' lets us omit entities that are already in a group,
    * i.e. omitting them from the list of entities (e.g. to add to the group), that this method returns.
    *
    * Idea: the inAttrType parm: do like in java & make it some kind of enum for type-safety? What's the scala idiom for that?
    */
  @tailrec protected final def chooseOrCreateObject(inLeadingText: Option[List[String]], inPreviousSelectionDesc: Option[String],
                                                    inPreviousSelectionId: Option[Long], inAttrType: String, startingDisplayRowIndexIn: Long = 0,
                                                    inClassId: Option[Long] = None, limitByClassIn: Boolean = false,
                                                    containingGroupIn: Option[Long] = None,
                                                    markPreviousSelectionIn: Boolean = false): Option[IdWrapper] = {
    if (inClassId != None) require(inAttrType == Controller.ENTITY_TYPE)
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
    def getChoiceList: (Array[String], Int, Int, Int, Int, Int) = {
      var keepPreviousSelectionChoiceNum = 1
      var createAttrTypeChoiceNum = 1
      var searchForEntityChoiceNum = 1
      var createRelationTypeChoiceNum = 1
      var createClassChoiceNum = 1
      var choiceList = Array(listNextItemsPrompt)
      if (inPreviousSelectionDesc != None) {
        keepPreviousSelectionChoiceNum += 1
        createAttrTypeChoiceNum += 1
        searchForEntityChoiceNum += 1
        createRelationTypeChoiceNum += 1
        createClassChoiceNum += 1
        choiceList = choiceList :+ "Keep previous selection (" + inPreviousSelectionDesc.get + ")."
      }
      //idea: use match instead of if: can it do || ?
      if (mostAttrTypeNames.contains(inAttrType)) {
        createAttrTypeChoiceNum += 1
        choiceList = choiceList :+ menuText_createEntityOrAttrType
        choiceList = choiceList :+ "Search for existing entity..."
        searchForEntityChoiceNum += 2
        createRelationTypeChoiceNum += 2
        createClassChoiceNum += 2
      } else if (relationAttrTypeNames.contains(inAttrType)) {
        choiceList = choiceList :+ menuText_CreateRelationType
        createRelationTypeChoiceNum += 1
        createClassChoiceNum += 1
      } else if (inAttrType == Controller.ENTITY_CLASS_TYPE) {
        choiceList = choiceList :+ "create new class (template for new entities)"
        createClassChoiceNum += 1
      } else throw new Exception("invalid inAttrType: " + inAttrType)

      (choiceList, keepPreviousSelectionChoiceNum, createAttrTypeChoiceNum, searchForEntityChoiceNum, createRelationTypeChoiceNum, createClassChoiceNum)
    }

    def getLeadTextAndObjectList(choicesIn: Array[String]): (List[String], java.util.ArrayList[_ >: RelationType with EntityClass <: Object], Array[String]) = {
      val prefix: String = inAttrType match {
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
      val numDisplayableItems = ui.maxColumnarChoicesToDisplayAfter(leadingText.size + 3 /* up to: see more of leadingText below .*/ , choicesIn.size,
                                                                    maxNameLength)
      val objectsToDisplay = {
        // ** KEEP THESE QUERIES AND CONDITIONS IN SYNC W/ THE COROLLARY ONES 2x BELOW ! (at similar comment)
        if (nonRelationAttrTypeNames.contains(inAttrType)) mDB.getEntities(startingDisplayRowIndexIn, Some(numDisplayableItems))
        else if (inAttrType == Controller.ENTITY_TYPE) mDB.getEntitiesOnly(startingDisplayRowIndexIn, Some(numDisplayableItems), inClassId, limitByClassIn,
                                                                           inPreviousSelectionId,
                                                                           containingGroupIn)
        else if (relationAttrTypeNames.contains(inAttrType)) {
          mDB.getRelationTypes(startingDisplayRowIndexIn, Some(numDisplayableItems)).asInstanceOf[java.util.ArrayList[RelationType]]
        }
        else if (inAttrType == Controller.ENTITY_CLASS_TYPE) mDB.getClasses(startingDisplayRowIndexIn, Some(numDisplayableItems))
        else throw new Exception("invalid inAttrType: " + inAttrType)
      }
      if (objectsToDisplay.size == 0) {
        // IF THIS CHANGES: change the guess at the 1st parameter to maxColumnarChoicesToDisplayAfter, JUST ABOVE!
        val txt: String = TextUI.NEWLN + TextUI.NEWLN + "(None of the needed " + (if (inAttrType == "relationtype") "relation types" else "entities") +
                          " have been created in this model, yet."
        leadingText = leadingText ::: List(txt)
      }
      val totalExisting: Long = {
        // ** KEEP THESE QUERIES AND CONDITIONS IN SYNC W/ THE COROLLARY ONES 2x ELSEWHERE ! (at similar comment)
        if (nonRelationAttrTypeNames.contains(inAttrType)) mDB.getEntitiesOnlyCount(inClassId, limitByClassIn, inPreviousSelectionId)
        else if (inAttrType == Controller.ENTITY_TYPE) mDB.getEntitiesOnlyCount(inClassId, limitByClassIn, inPreviousSelectionId)
        else if (relationAttrTypeNames.contains(inAttrType)) mDB.getRelationTypeCount
        else if (inAttrType == Controller.ENTITY_CLASS_TYPE) mDB.getClassCount()
        else throw new Exception("invalid inAttrType: " + inAttrType)
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
          if (nonRelationAttrTypeNames.contains(inAttrType))
            mDB.getEntityCount
          else if (inAttrType == Controller.ENTITY_TYPE) mDB.getEntitiesOnlyCount(inClassId, limitByClassIn)
          else if (relationAttrTypeNames.contains(inAttrType))
            mDB.getRelationTypeCount
          else if (inAttrType == Controller.ENTITY_CLASS_TYPE) mDB.getClassCount()
          else throw new Exception("invalid inAttrType: " + inAttrType)
        if (x >= numObjectsInModel) {
          ui.displayText("End of list found; starting over from the beginning.")
          0 // start over
        } else x
      }
      index
    }

    val (choices, keepPreviousSelectionChoice, createAttrTypeChoice, searchForEntityChoice, createRelationTypeChoice, createClassChoice): (Array[String],
      Int, Int, Int, Int, Int) = getChoiceList

    val (leadingText, objectsToDisplay, names) = getLeadTextAndObjectList(choices)
    val ans = ui.askWhichChoiceOrItsAlternate(Some(leadingText.toArray), choices, names)

    if (ans == None) None
    else {
      val answer = ans.get._1
      val userChoseAlternate = ans.get._2
      if (answer == listNextItemsChoiceNum && answer <= choices.size) {
        // (For reason behind " && answer <= choices.size", see comment where it is used in entityMenu.)
        val index: Long = getNextStartingObjectIndex(objectsToDisplay.size, nonRelationAttrTypeNames, relationAttrTypeNames)
        chooseOrCreateObject(inLeadingText, inPreviousSelectionDesc, inPreviousSelectionId, inAttrType, index, inClassId, limitByClassIn,
                             containingGroupIn, markPreviousSelectionIn)
      }
      else if (answer == keepPreviousSelectionChoice && answer <= choices.size) {
        // Such as if editing several fields on an attribute and doesn't want to change the first one.
        // Not using "get out" option for this because it would exit from a few levels at once and
        // then user wouldn't be able to proceed to other field edits.
        Some(new IdWrapper(inPreviousSelectionId.get))
      }
      else if (answer == createAttrTypeChoice && answer <= choices.size) {
        val e: Option[Entity] = askForInfoAndCreateEntity(inClassId)
        if (e == None) None
        else Some(new IdWrapper(e.get.getId))
      }
      else if (answer == searchForEntityChoice && answer <= choices.size) {
        val ans = ui.askForString(Some(Array(searchPrompt(Controller.ENTITY_TYPE))))
        if (ans == None)
          None
        else {
          // Allow relation to self (eg, picking self as 2nd part of a RelationToEntity), so None in 2nd parm.
          val e: Option[IdWrapper] = findExistingObject(0, Controller.ENTITY_TYPE, None, ans.get)
          if (e == None) None
          else Some(new IdWrapper(e.get.getId))
        }
      }
      else if (answer == createRelationTypeChoice && relationAttrTypeNames.contains(inAttrType) && answer <= choices.size) {
        val entity: Option[Entity] = askForNameAndWriteEntity(Controller.RELATION_TYPE_TYPE)
        if (entity == None) None
        else Some(new IdWrapper(entity.get.getId))
      }
      else if (answer == createClassChoice && inAttrType == Controller.ENTITY_CLASS_TYPE && answer <= choices.size) {
        val result: Option[(Long, Long)] = askForAndWriteClassAndDefiningEntityName()
        if (result == None) None
        else {
          val (classId, entityId) = result.get
          val ans = ui.askYesNoQuestion("Do you want to add attributes to the newly created defining entity for this class? (These will be used for the " +
                                        "prompts " +
                                        "and defaults when creating/editing entities in this class).", Some("y"))
          if (ans != None && ans.get) {
            new EntityMenu(ui, mDB).entityMenu(0, new Entity(mDB, entityId))
          }
          Some(new IdWrapper(classId))
        }
      }
      else if (answer > choices.length && answer <= (choices.length + objectsToDisplay.size)) {
        // those in the condition on the previous line are 1-based, not 0-based.
        val index = answer - choices.length - 1
        // user typed a letter to select.. (now 0-based)
        // user selected a new object and so we return to the previous menu w/ that one displayed & current
        val o = objectsToDisplay.get(index)
        //if ("text,quantity,entity,date,boolean,file,relationtype".contains(inAttrType)) {
        //i.e., if (inAttrType == Controller.TEXT_TYPE || (= any of the other types...)):
        if (userChoseAlternate) {
          inAttrType match {
            // idea: replace this condition by use of a trait (the type of o, which has getId), or being smarter with scala's type system. attrTypeIn match {
            case Controller.ENTITY_TYPE =>
              new EntityMenu(ui, mDB).entityMenu(0, o.asInstanceOf[Entity])
            case _ =>
              // (choosing a group doesn't call this, it calls chooseOrCreateGroup)
              throw new OmException("not yet implemented")
          }
          chooseOrCreateObject(inLeadingText, inPreviousSelectionDesc, inPreviousSelectionId, inAttrType, startingDisplayRowIndexIn, inClassId, limitByClassIn,
                               containingGroupIn, markPreviousSelectionIn)
        } else {
          if (evenMoreAttrTypeNames.contains(inAttrType))
            Some(o.asInstanceOf[Entity].getIdWrapper)
          else if (inAttrType == Controller.ENTITY_CLASS_TYPE) Some(o.asInstanceOf[EntityClass].getIdWrapper)
          else throw new Exception("invalid inAttrType: " + inAttrType)
        }
      }
      else {
        ui.displayText("unknown response")
        chooseOrCreateObject(inLeadingText, inPreviousSelectionDesc, inPreviousSelectionId, inAttrType, startingDisplayRowIndexIn, inClassId,
                             limitByClassIn, containingGroupIn, markPreviousSelectionIn)
      }
    }
  }

  def maxNameLength: Int = math.max(math.max(mDB.entityNameLength, mDB.relationTypeNameLength), mDB.classNameLength)

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
  protected def askForQuantityAttributeNumberAndUnit(inDH: QuantityAttributeDataHolder, inEditing: Boolean): Option[QuantityAttributeDataHolder] = {
    val outDH: QuantityAttributeDataHolder = inDH
    val leadingText: List[String] = List("SELECT A UNIT FOR THIS QUANTITY (i.e., centimeters, or quarts; ESC or blank to cancel):")
    val previousSelectionDesc = if (inEditing) Some(new Entity(mDB, inDH.unitId).getName) else None
    val previousSelectionId = if (inEditing) Some(inDH.unitId) else None
    val unitSelection = chooseOrCreateObject(Some(leadingText), previousSelectionDesc, previousSelectionId, Controller.QUANTITY_TYPE)
    if (unitSelection == None) {
      ui.displayText("Blank, so assuming you want to cancel; if not come back & add again.", waitForKeystroke = false)
      None
    } else {
      outDH.unitId = unitSelection.get.getId
      val leadingText = Array[String]("ENTER THE NUMBER FOR THE QUANTITY (i.e., 5, for 5 centimeters length)")
      val previousQuantity: String = outDH.number.toString
      val ans = ui.askForString(Some(leadingText), Some(isNumeric), Some(previousQuantity))
      if (ans == None) None
      else {
        val numStr = ans.get
        outDH.number = numStr.toFloat
        Some(outDH)
      }
    }
  }

  /** Returns None if user wants to cancel. */
  protected def askForTextAttributeText(inDH: TextAttributeDataHolder, inEditing: Boolean): Option[TextAttributeDataHolder] = {
    val outDH = inDH.asInstanceOf[TextAttributeDataHolder]
    val defaultValue: Option[String] = if (inEditing) Some(inDH.text) else None
    val ans = ui.askForString(Some(Array("Type attribute value, then press Enter; ESC to cancel")), None, defaultValue)
    if (ans == None) None
    else {
      outDH.text = ans.get
      Some(outDH)
    }
  }

  /** Returns None if user wants to cancel. */
  protected def askForDateAttributeValue(inDH: DateAttributeDataHolder, inEditing: Boolean): Option[DateAttributeDataHolder] = {
    val outDH = inDH.asInstanceOf[DateAttributeDataHolder]
    val defaultValue: Option[String] = if (!inEditing) None else Some(Controller.DATEFORMAT.format(new java.util.Date(inDH.date)))
    def dateCriteria(date: String): Boolean = {
      !finishAndParseTheDate(date)._2
    }
    val ans = ui.askForString(Some(Array(genericDatePrompt)), Some(dateCriteria), defaultValue)
    if (ans == None) None
    else {
      val (newDate: Option[Long], retry: Boolean) = finishAndParseTheDate(ans.get)
      if (retry) throw new Exception("Programmer error: date indicated it was parseable, but the same function said afterward it couldn't be parsed.  Why?")
      else if (newDate == None) throw new Exception("There is a bug: the program shouldn't have got to this point.")
      else {
        outDH.date = newDate.get
        Some(outDH)
      }
    }
  }

  /** Returns None if user wants to cancel. */
  protected def askForBooleanAttributeValue(inDH: BooleanAttributeDataHolder, inEditing: Boolean): Option[BooleanAttributeDataHolder] = {
    val outDH = inDH.asInstanceOf[BooleanAttributeDataHolder]
    val ans = ui.askYesNoQuestion("Is the new true/false value true now?", if (inEditing && inDH.boolean) Some("y") else Some("n"))
    if (ans == None) None
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
  protected def askForFileAttributeInfo(inDH: FileAttributeDataHolder, inEditing: Boolean): Option[FileAttributeDataHolder] = {
    val outDH = inDH.asInstanceOf[FileAttributeDataHolder]
    var path: Option[String] = None
    if (!inEditing) {
      // we don't want the original path to be editable after the fact, because that's a historical observation and there is no sense in changing it.
      path = ui.askForString(Some(Array("Enter file path (must exist and be readable), then press Enter; ESC to cancel")), Some(inputFileValid))
    }
    if (!inEditing && path == None) None
    else {
      // if we can't fill in the path variables by now, there is a bug:
      if (!inEditing) outDH.originalFilePath = path.get
      else path = Some(outDH.originalFilePath)

      val defaultValue: Option[String] = if (inEditing) Some(inDH.description) else Some(FilenameUtils.getBaseName(path.get))
      val ans = ui.askForString(Some(Array("Type file description, then press Enter; ESC to cancel")), None, defaultValue)
      if (ans == None) None
      else {
        outDH.description = ans.get
        Some(outDH)
      }
    }
  }

  /** Returns None if user wants to cancel. */
  protected def askForRelToGroupInfo(inDH: RelationToGroupDataHolder, inEditingUNUSEDForNOW: Boolean = false): Option[RelationToGroupDataHolder] = {
    val outDH = inDH

    val groupSelection = chooseOrCreateGroup(Some(List("SELECT GROUP FOR THIS RELATION")), 0)
    val groupId: Option[Long] = {
      if (groupSelection == None) {
        ui.displayText("Blank, so assuming you want to cancel; if not come back & add again.", waitForKeystroke = false)
        None
      } else Some[Long](groupSelection.get.getId)
    }

    if (groupId == None) None
    else {
      outDH.groupId = groupId.get
      Some(outDH)
    }
  }

  /** Returns the id of a Group, or None if user wants out.  The parameter 'containingGroupIn' lets us omit entities that are already in a group,
    * i.e. omitting them from the list of entities (e.g. to add to the group), that this method returns.
    */
  @tailrec protected final def chooseOrCreateGroup(inLeadingText: Option[List[String]], startingDisplayRowIndexIn: Long = 0,
                                                   containingGroupIn: Option[Long] = None /*ie group to omit from pick list*/
                                                    ): Option[IdWrapper] = {
    val totalExisting: Long = mDB.getGroupCount
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
                                                    "Search for existing group...")
    val numDisplayableItems = ui.maxColumnarChoicesToDisplayAfter(leadingText.size, choicesPreAdjustment.size, maxNameLength)
    val objectsToDisplay = mDB.getGroups(startingDisplayRowIndexIn, Some(numDisplayableItems), containingGroupIn)
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
    if (ans == None) None
    else {
      val answer = ans.get._1
      val userChoseAlternate = ans.get._2
      if (answer == 1 && answer <= choices.size) {
        // (For reason behind " && answer <= choices.size", see comment where it is used in entityMenu.)
        val nextStartingIndex: Long = getNextStartingObjectIndex(objectsToDisplay.size)
        chooseOrCreateGroup(inLeadingText, nextStartingIndex, containingGroupIn)
      } else if (answer == 2 && answer <= choices.size) {
        val ans = ui.askForString(Some(Array(relationToGroupNamePrompt)))
        if (ans == None || ans.get.trim.length() == 0) None
        else {
          val name = ans.get
          val ans2 = ui.askYesNoQuestion("Should this group allow entities with mixed classes? (Usually not desirable: doing so means losing some " +
                                         "conveniences such as scripts and assisted data entry.)", Some("y"))
          if (ans2 == None) None
          else {
            val mixedClassesAllowed = ans2.get
            val newGroupId = mDB.createGroup(name, mixedClassesAllowed)
            Some(new IdWrapper(newGroupId))
          }
        }
      } else if (answer == 3 && answer <= choices.size) {
        val ans = ui.askForString(Some(Array(searchPrompt(Controller.GROUP_TYPE))))
        if (ans == None) None
        else {
          // Allow relation to self, so None in 2nd parm.
          val g: Option[IdWrapper] = findExistingObject(0, Controller.GROUP_TYPE, None, ans.get)
          if (g == None) None
          else Some(new IdWrapper(g.get.getId))
        }
      } else if (answer > choices.length && answer <= (choices.length + objectsToDisplay.size)) {
        // those in that^ condition are 1-based, not 0-based.
        val index = answer - choices.length - 1
        val o = objectsToDisplay.get(index)
        if (userChoseAlternate) {
          // for now, picking the first RTG found for this group, until it's clear which of its RTGs to use.
          // (see also the other locations w/ similar comment!)
          val someRelationToGroups: java.util.ArrayList[RelationToGroup] = mDB.getRelationToGroupsByGroup(o.asInstanceOf[Group].getId, 0, Some(1))
          new GroupMenu(ui, mDB).groupMenu(0, someRelationToGroups.get(0))
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
  protected def askForRelationEntityIdNumber2(inDH: RelationToEntityDataHolder, inEditing: Boolean): Option[RelationToEntityDataHolder] = {
    val previousSelectionDesc = if (!inEditing) None
    else Some(new Entity(mDB, inDH.entityId2).getName)
    val previousSelectionId = if (!inEditing) None
    else Some(inDH.entityId2)
    val (id: Option[Long]) = askForAttributeTypeId("SELECT OTHER (RELATED) ENTITY FOR THIS RELATION", Controller.ENTITY_TYPE, previousSelectionDesc,
                                                   previousSelectionId)
    val outDH = inDH
    if (id == None) None
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
    val firstHyphenPosition = if (dateStr.indexOf('-') != -1) dateStr.indexOf('-') else dateStr.size
    //but only if the string format looks somewhat expected; otherwise let later parsing handle it all.
    val filledInDateStr =
      if (dateStr.size > firstHyphenPosition + 1 && dateStr.size < firstHyphenPosition + 6
          && dateStr.indexOf('-') == firstHyphenPosition && dateStr.indexOf('-', firstHyphenPosition + 1) >= 0) {
        val secondHyphenPosition = dateStr.indexOf('-', firstHyphenPosition + 1)
        if (secondHyphenPosition == firstHyphenPosition + 2 || secondHyphenPosition == firstHyphenPosition + 3) {
          if (dateStr.size == secondHyphenPosition + 2 || dateStr.size == secondHyphenPosition + 3) {
            val year = dateStr.substring(0, firstHyphenPosition)
            val mo = dateStr.substring(firstHyphenPosition + 1, secondHyphenPosition)
            val dy = dateStr.substring(secondHyphenPosition + 1)
            year + '-' + (if (mo.size == 1) "0" + mo else mo) + '-' + (if (dy.size == 1) "0" + dy else dy)
          }
          else dateStr
        }
        else dateStr
      } else if (dateStr.size == firstHyphenPosition + 2) {
        // also handle format like 2013-1
        val year = dateStr.substring(0, firstHyphenPosition)
        val mo = dateStr.substring(firstHyphenPosition + 1)
        year + '-' + "0" + mo
      }
      else dateStr


    // Fill in the date w/ "blank" information for whatever detail the user didn't provide:
    val filledInDateStrWithoutYear = if (firstHyphenPosition < filledInDateStr.size) filledInDateStr.substring(firstHyphenPosition + 1) else ""
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
        if (era.isEmpty) Controller.DATEFORMAT.parse(dateStrWithZeros)
        else Controller.DATEFORMAT_WITH_ERA.parse(era + dateStrWithZeros)
      (Some(d.getTime), false)
    } catch {
      case e: java.text.ParseException =>
        ui.displayText("Invalid date format. Try something like \"2003\", or \"2003-01-31\", or if you need a timezone, " +
                       "all of \"yyyy-MM-dd HH:mm:ss:SSS zzz\", like for just before midnight: \"2013-01-31 //23:59:59:999 MST\".")
        (None, true)
    }
  }

  /** Returns (validOnDate, observationDate, userWantsToCancel) */
  protected def askForAttributeValidAndObservedDates(inEditing: Boolean,
                                                     oldValidOnDateIn: Option[Long],
                                                     oldObservedDateIn: Long): (Option[Long], Long, Boolean) = {

    //idea: make this more generic, passing in prompt strings &c, so it's more cleanly useful for DateAttribute instances. Or not: lacks shared code.
    //idea: separate these into 2 methods, 1 for each time (not much common material of significance).
    /** Helper method made so it can be recursive, it returns the date (w/ meanings as with displayText below, and as in PostgreSQLDatabase.createTables),
      * and true if the user wants to cancel/get out). */
    @tailrec def askForDate(dateTypeIn: String, acceptanceCriteriaIn: (String) => Boolean): (Option[Long], Boolean) = {
      val leadingText: Array[String] =
        if (dateTypeIn == VALID) {
          Array("Please enter the date when this was first VALID (true) (like this, w/ at least the year: \"2013-01-31 23:59:59:999 MST\"; zeros are " +
                "allowed in all but the yyyy-mm-dd):  (Or for \"all time\", enter just 0.  Or for unknown/unspecified leave blank.  Or for current date/time " +
                "enter \"now\".  ESC to exit this.  For dates far in the past you can prefix them with \"BC \" (or \"AD \", but either way omit a space " +
                "before the year), like BC3400-01-31 23:59:59:999 GMT, entered at least up through the year, up to ~292000000 years AD or BC.  " +
                "There is ambiguity about BC that needs some " +
                "investigation, because java allows a '0' year (which for now means 'for all time' in just this program), but normal human time doesn't " +
                "allow a '0' year, so maybe you have to subtract a year from all BC things for them to work right, and enter/read them accordingly, until " +
                "someone learns for sure, and we decide whether to subtract a year from everything BC for you automatically. Hm. *OR* maybe dates in year " +
                "zero " +
                "just don't mean anything so can be ignored by users, and all other dates' entry are just fine, so there's nothing to do but use it as is? " +
                "But that would have to be kept in mind if doing any relative date calculations in the program, e.g. # of years, spanning 0.)" + TextUI.NEWLN +
                "Also, real events with more " +
                "specific time-tracking needs will probably need to model their own time-related entity classes, and establish relations to them, within " +
                "their use of OM.")
          //ABOUT THAT LAST COMMENT: WHY DOES JAVA ALLOW A 0 YEAR, UNLESS ONLY BECAUSE IT USES long #'S? SEE E.G.
          // http://www.msevans.com/calendar/daysbetweendatesapplet.php
          //which says: "...[java?] uses a year 0, which is really 1 B.C. For B.C. dates, you have to remember that the years are off by one--10 B.C.
          // to [java?] is really 11 B.C.", but this really needs more investigation on what is the Right Thing to do.
          // Or, just let the dates go in & out of the data, interpreted just as they are now, but the savvy users will recognize that dates in year zero just
          // don't mean anything, thus the long values in that range don't mean anything so can be disregarded (is that how it really works in java??), (or if
          // so we could inform users when such a date is present, that it's bogus and to use 1 instead)?
        } else if (dateTypeIn == OBSERVED) {
          Array("WHEN OBSERVED?: " + genericDatePrompt + " (\"All time\" and \"unknown\" not" + " allowed here.) ")
        } else throw new Exception("unexpected type: " + dateTypeIn)
      val ans = ui.askForString(Some(leadingText), None,
                                inDefaultValue =
                                  if (dateTypeIn == VALID) {
                                    if (inEditing && oldValidOnDateIn != None) {
                                      if (oldValidOnDateIn.get == 0) Some("0")
                                      else Some(Controller.DATEFORMAT_WITH_ERA.format(new java.util.Date(oldValidOnDateIn.get)))
                                    }
                                    else None
                                  } else if (dateTypeIn == OBSERVED) {
                                    if (inEditing) Some(Controller.DATEFORMAT_WITH_ERA.format(new java.util.Date(oldObservedDateIn))) else None
                                  } else throw new Exception("unexpected type: " + dateTypeIn)
                               )
      if (ans == None) {
        if (dateTypeIn == VALID) (None, true)
        else if (dateTypeIn == OBSERVED) {
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
          require(observedDate != None)
          val ans = ui.askYesNoQuestion("Dates are: " + AttributeWithValidAndObservedDates.getDatesDescription(validOnDate,
                                                                                                               observedDate.get) + ": right?", Some("y"))
          if (ans != None && ans.get) (validOnDate, observedDate.get, userCancelled)
          else askForBothDates()
        }
      }
    }
    askForBothDates()
  }

  // Used for example after one has been deleted, to put the highlight on right next one:
  // idea: This feels overcomplicated.  Make it better?  Fixing bad smells in general (large classes etc etc) is on the task list.
  def findEntryToHighlightNext(objIdsIn: Array[Long], objectsToDisplayIn: java.util.ArrayList[Entity], deletedOrArchivedOneIn: Boolean,
                               previouslyHighlightedIndexInObjListIn: Int, previouslyHighlightedEntryIn: Entity): Option[Entity] = {
    // here of course, previouslyHighlightedIndexInObjListIn and objIds.size were calculated prior to the deletion.
    if (deletedOrArchivedOneIn) {
      val newObjListSize = objIdsIn.size - 1
      val newIndexToHighlight = math.min(newObjListSize - 1, previouslyHighlightedIndexInObjListIn)
      if (newIndexToHighlight >= 0) {
        if (newIndexToHighlight != previouslyHighlightedIndexInObjListIn) Some(objectsToDisplayIn.get(newIndexToHighlight))
        else {
          if (newIndexToHighlight + 1 < newObjListSize - 1) Some(objectsToDisplayIn.get(newIndexToHighlight + 1))
          else if (newIndexToHighlight - 1 >= 0) Some(objectsToDisplayIn.get(newIndexToHighlight - 1))
          else None
        }
      }
      else None
    } else {
      Some(previouslyHighlightedEntryIn)
    }
  }

  protected def goToEntityOrItsSoleGroupsMenu(userSelection: Entity, relationToGroupIn: Option[RelationToGroup] = None,
                                              containingGroupIn: Option[Group] = None): (Option[Entity], Option[Long], Boolean) = {
    val (rtid, groupId, moreThanOneAvailable) = mDB.findRelationToAndGroup_OnEntity(userSelection.getId)
    val subEntitySelected: Option[Entity] = None
    if (groupId != None && !moreThanOneAvailable) {
      // In quick menu, for efficiency of some work like brainstorming, if it's obvious which subgroup to go to, just go there.
      // We DON'T want @tailrec on this method for this call, so that we can ESC back to the current menu & list! (so what balance/best? Maybe move this
      // to its own method, so it doesn't try to tail optimize it?)  See also the comment with 'tailrec', mentioning why to have it, above.
      new QuickGroupMenu(ui, mDB).quickGroupMenu(0, new RelationToGroup(mDB, userSelection.getId, rtid.get, groupId.get), callingMenusRtgIn = relationToGroupIn)
    } else {
      new EntityMenu(ui, mDB).entityMenu(0, userSelection, None, None, containingGroupIn)
      // deal with entityMenu possibly having deleted the entity:
    }
    (subEntitySelected, groupId, moreThanOneAvailable)
  }

  /** Shows ">" in front of a group if it contains exactly one subgroup which has at least one entry; shows ">>" if contains multiple subgroups,
    * and "" if contains no subgroups or the one subgroup is empty.
    */
  def getNumSubgroupsPrefix(entityId: Long): String = {
    val (groupsCount: Long, singleGroupEntryCount: Long) = {
      val rtgCountOnEntity: Long = mDB.getRelationToGroupCountByEntity(Some(entityId))
      if (rtgCountOnEntity == 0) {
        (0L, 0L)
      }
      else {
        if (rtgCountOnEntity > 1) {
          // (For some reason, not having the 'asInstanceOf[Long]' here results in a stack trace on the variable assignment out of this block, with something
          // about a tuple mismatch?, even tho it is already a Long:)
          (rtgCountOnEntity.asInstanceOf[Long], 0L)
        } else {
          val (_, gid: Option[Long], moreAvailable) = mDB.findRelationToAndGroup_OnEntity(entityId)
          if (gid == None || moreAvailable) throw new OmException("Found " + (if (gid == None) 0 else ">1") + " but by the earlier checks, " +
                                                                  "there should be exactly one group in entity " + entityId + " .")
          (rtgCountOnEntity, mDB.getGroupEntryCount(gid.get, Some(false)))
        }
      }
    }
    val subgroupsCountPrefix: String =
      if (groupsCount == 0 || (groupsCount == 1 && singleGroupEntryCount == 0)) {
        ""
      } else if (groupsCount == 1 && singleGroupEntryCount > 0) {
        ">"
      } else {
        ">>"
      }
    subgroupsCountPrefix
  }

  /** Returns None if user just wants out; a String (user's answer, not useful outside this method) if update was done..
    */
  def editGroupName(groupIn: Group): Option[String] = {
    // doesn't seem to make sense to ck for duplicate names here: the real identity depends on what it relates to, and dup names may be common.
    val ans = ui.askForString(Some(Array(relationToGroupNamePrompt)), None, Some(groupIn.getName))
    if (ans == None || ans.get.trim.length() == 0) {
      ui.displayText("Not updated.")
      None
    } else {
      groupIn.update(None, Some(ans.get.trim), None, None, None)
      ans
    }
  }

  protected def addEntityToGroup(groupIn: Group): Option[Long] = {
    if (!groupIn.getMixedClassesAllowed) {
      if (groupIn.groupSize == 0) {
        // adding 1st entity to this group, so:
        val leadingText = List("ADD ENTITY TO A GROUP (**whose class will set the group's enforced class, even if 'None'**):")
        val idWrapper: Option[IdWrapper] = chooseOrCreateObject(Some(leadingText), None, None, Controller.ENTITY_TYPE,
                                                                containingGroupIn = Some(groupIn.getId))
        if (idWrapper != None) {
          mDB.addEntityToGroup(groupIn.getId, idWrapper.get.getId)
          Some(idWrapper.get.getId)
        } else None
      } else {
        // it's not the 1st entry in the group, so add an entity using the same class as those previously added (or None as case may be).
        val entityClassInUse = groupIn.getClassId
        val idWrapper: Option[IdWrapper] = chooseOrCreateObject(None, None, None, Controller.ENTITY_TYPE, 0, entityClassInUse, limitByClassIn = true,
                                                                containingGroupIn = Some(groupIn.getId))
        if (idWrapper == None) None
        else {
          val entityId = idWrapper.get.getId
          try {
            mDB.addEntityToGroup(groupIn.getId, entityId)
            Some(entityId)
          }
          catch {
            case e: Exception =>
              if (e.getMessage.contains(PostgreSQLDatabase.MIXED_CLASSES_EXCEPTION)) {
                val oldClass: String = if (entityClassInUse == None) "(none)" else new EntityClass(mDB, entityClassInUse.get).getDisplayString
                val newClassId = new Entity(mDB, entityId).getClassId
                val newClass: String = if (newClassId == None || entityClassInUse == None) "(none)"
                else new EntityClass(mDB,
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
      if (idWrapper != None) {
        mDB.addEntityToGroup(groupIn.getId, idWrapper.get.getId)
        Some(idWrapper.get.getId)
      } else None
    }
  }

  def showException(e: Exception) {
    if (e.isInstanceOf[org.postgresql.util.PSQLException] || e.isInstanceOf[OmDatabaseException] || throwableToString(e).contains("ERROR: current transaction" +
                                                                                                                                  " is aborted, " +
                                                                                                                                  "commands ignored until end" +
                                                                                                                                  " of transaction block")) {
      mDB.rollbackTrans()
    }
    val ans = ui.askYesNoQuestion("An error occurred: \"" + e.getClass.getName + ": " + e.getMessage + "\".  If you can provide simple instructions to " +
                                  "reproduce it consistently, " +
                                  "maybe it can be fixed.  Do you want to see the detailed output?")
    if (ans != None && ans.get) {
      ui.displayText(throwableToString(e))
    }
  }

  def throwableToString(e: Exception): String = {
    val stringWriter = new StringWriter()
    e.printStackTrace(new PrintWriter(stringWriter))
    stringWriter.toString
  }

  def chooseAmongEntities(containingEntities: util.ArrayList[(Long, Entity)]): Option[Entity] = {
    val leadingText = List[String]("Pick from menu, or an entity by letter")
    val choices: Array[String] = Array(listNextItemsPrompt)
    val numDisplayableItems: Long = ui.maxColumnarChoicesToDisplayAfter(leadingText.size, choices.size, maxNameLength)
    //(see comments at similar location in EntityMenu, as of this writing on line 288)
    val containingEntitiesNamesWithRelTypes: Array[String] = containingEntities.toArray.map {
                                                                                              case relTypeIdAndEntity: (Long, Entity) =>
                                                                                                val relTypeId: Long = relTypeIdAndEntity._1
                                                                                                val entity: Entity = relTypeIdAndEntity._2
                                                                                                val relTypeName: String = new RelationType(mDB,
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
    if (ans == None) None
    else {
      val answer = ans.get
      if (answer == 1 && answer <= choices.size) {
        // see comment above
        ui.displayText("not yet implemented") //%%
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

}
