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
package org.onemodel.core.controllers

import java.io._
import java.util

import org.onemodel.core._
import org.onemodel.core.database.{Database, RestDatabase, PostgreSQLDatabase}
import org.onemodel.core.model._
import org.postgresql.util.PSQLException

import scala.annotation.tailrec
import scala.collection.mutable.ArrayBuffer

/** This Controller is for user-interactive things.  The Controller class in the web module is for the REST API.  For shared code that does not fit
  * in those, see the org.onemodel.core.Util object (in Util.scala).
  *
  * Improvements to this class should START WITH MAKING IT BETTER TESTED (functional testing? integration? see
  * scalatest docs 4 ideas, & maybe use expect or the gnu testing tool that uses expect?), delaying side effects more,
  * shorter methods, other better scala style, etc.
  *
  * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * *
  Don't ever instantiate a controller from a *test* without passing in username/password parameters, because it will try to log in to the user's default
  database and run the tests there (ie, they could be destructive):
  * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * *
  */
class Controller(val ui: TextUI, forceUserPassPromptIn: Boolean = false, defaultUsernameIn: Option[String] = None, defaultPasswordIn: Option[String] = None) {
  //idea: get more scala familiarity then change this so it has limited visibility/scope: like, protected (subclass instances) + ImportExportTest.
  val db: PostgreSQLDatabase = tryLogins(forceUserPassPromptIn, defaultUsernameIn, defaultPasswordIn)

  /** Returns the id and the entity, if they are available from the preferences lookup (id) and then finding that in the db (Entity). */
  def getDefaultEntity: Option[(Long, Entity)] = {
    if (defaultDisplayEntityId.isEmpty || ! db.entityKeyExists(defaultDisplayEntityId.get)) {
      None
    } else {
      val entity: Option[Entity] = Entity.getEntityById(db, defaultDisplayEntityId.get)
      if (entity.isDefined && entity.get.isArchived) {
        val msg = "The default entity " + TextUI.NEWLN + "    " + entity.get.getId + ": \"" + entity.get.getName + "\"" + TextUI.NEWLN +
                  "... was found but is archived.  You might run" +
                  " into problems unless you un-archive it, or choose a different entity to make the default, or display all archived" +
                  " entities then search for this entity and un-archive it under its Entity Menu options 9, 4."
        val ans = ui.askWhich(Some(Array(msg)), Array("Un-archive the default entity now", "Display archived entities"))
        if (ans.isDefined) {
          if (ans.get == 1) {
            entity.get.unarchive()
          } else if (ans.get == 2) {
            db.setIncludeArchivedEntities(true)
          }
        }
      }
      Some((defaultDisplayEntityId.get, entity.get))
    }
  }

  def start() {
    // idea: wait for keystroke so they do see the copyright each time. (is also tracked):  make it save their answer 'yes/i agree' or such in the DB,
    // and don't make them press the keystroke again (timesaver)!  See code at top of PostgreSQLDatabase that puts things in the db at startup: do similarly?
    ui.displayText(Util.copyright(ui), waitForKeystrokeIn = true, Some("IF YOU DO NOT AGREE TO THOSE TERMS: " + ui.howQuit + " to exit.\n" +
                                                             "If you agree to those terms: "))
    // Max id used as default here because it seems the least likely # to be used in the system hence the
    // most likely to cause an error as default by being missing, so the system can respond by prompting
    // the user in some other way for a use.
    if (getDefaultEntity.isEmpty) {
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
      //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) w/in this method! (should they be, in this case tho'?)
      //re-checking for the default each time because user can change it.
      new MainMenu(ui, db, this).mainMenu(if (getDefaultEntity.isEmpty) None else Some(getDefaultEntity.get._2),
                                          goDirectlyToChoice)
      menuLoop()
    }
    menuLoop(Some(5))
  }

  /** If the 1st parm is true, the next 2 must be omitted or None. */
  private def tryLogins(forceUserPassPromptIn: Boolean = false, defaultUsernameIn: Option[String] = None,
                        defaultPasswordIn: Option[String] = None): PostgreSQLDatabase = {

    require(if (forceUserPassPromptIn) defaultUsernameIn.isEmpty && defaultPasswordIn.isEmpty else true)

    // Tries the system username, blank password, & if that doesn't work, prompts user.
    //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) within this method, below!
    @tailrec def tryOtherLoginsOrPrompt(): PostgreSQLDatabase = {
      val db = {
        var pwdOpt: Option[String] = None
        // try logging in with some obtainable default values first, to save user the trouble, like if pwd is blank
        val (defaultUserName, defaultPassword) = Util.getDefaultUserInfo
        val dbWithSystemNameBlankPwd = login(defaultUserName, defaultPassword, showError = false)
        if (dbWithSystemNameBlankPwd.isDefined) dbWithSystemNameBlankPwd
        else {
          val usrOpt = ui.askForString(Some(Array("Username")), None, Some(defaultUserName))
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
                       waitForKeystrokeIn = false)
        tryOtherLoginsOrPrompt()
      }
      else db.get
    }

    if (forceUserPassPromptIn) {
      //IF ADDING ANY optional PARAMETERS, be sure they are also passed along in the recursive call(s) within this method, below!
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
    try Some(new PostgreSQLDatabase(username, new String(password)))
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
  var showPublicPrivateStatusPreference: Option[Boolean] = db.getUserPreference_Boolean(Util.SHOW_PUBLIC_PRIVATE_STATUS_PREFERENCE)
  def refreshPublicPrivateStatusPreference(): Unit = showPublicPrivateStatusPreference = db.getUserPreference_Boolean(Util.SHOW_PUBLIC_PRIVATE_STATUS_PREFERENCE)
  // putting this in a var instead of recalculating it every time (too frequent) inside findDefaultDisplayEntityId:
  var defaultDisplayEntityId: Option[Long] = db.getUserPreference_EntityId(Util.DEFAULT_ENTITY_PREFERENCE)
  def refreshDefaultDisplayEntityId(): Unit = defaultDisplayEntityId = db.getUserPreference_EntityId(Util.DEFAULT_ENTITY_PREFERENCE)

  def askForClass(): Option[Long] = {
    val msg = "CHOOSE ENTITY'S CLASS.  (Press ESC if you don't know or care about this.  Detailed explanation on the class feature will be available " +
              "at onemodel.org when this feature is documented more (hopefully at the next release), or ask on the email list.)"
    val result: Option[(IdWrapper, Boolean, String)] = chooseOrCreateObject(Some(List[String](msg)), None, None, Util.ENTITY_CLASS_TYPE)
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
  def askForClassInfoAndNameAndCreateEntity(classIdIn: Option[Long] = None): Option[Entity] = {
    var newClass = false
    val classId: Option[Long] =
      if (classIdIn.isDefined) classIdIn
      else {
        newClass = true
        askForClass()
      }
    val ans: Option[Entity] = askForNameAndWriteEntity(Util.ENTITY_TYPE, None, None, None, None, classId,
                                                       Some(if (newClass) "DEFINE THE ENTITY:" else ""))
    if (ans.isDefined) {
      val entity = ans.get
      // idea: (is also on fix list): this needs to be removed, after evaluating for other side effects, to fix the bug
      // where creating a new relationship, and creating the entity2 in the process, it puts the wrong info
      // on the header for what is being displayed/edited next!: Needs refactoring anyway: this shouldn't be at
      // a low level.
      ui.displayText("Created " + Util.ENTITY_TYPE + ": " + entity.getName, waitForKeystrokeIn = false)

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
   * SEE DESCRIPTIVE COMMENT ON askForAndWriteClassAndTemplateEntityName, WHICH APPLIES TO all such METHODS (see this cmt elsewhere).
    *
    * The "previous..." parameters are for the already-existing data (ie, when editing not creating).
    */
  def askForNameAndWriteEntity(typeIn: String, existingIdIn: Option[Long] = None,
                                         previousNameIn: Option[String] = None, previousDirectionalityIn: Option[String] = None,
                                         previousNameInReverseIn: Option[String] = None, classIdIn: Option[Long] = None,
                                         leadingTextIn: Option[String] = None, duplicateNameProbablyOK: Boolean = false): Option[Entity] = {
    if (classIdIn.isDefined) require(typeIn == Util.ENTITY_TYPE)
    val createNotUpdate: Boolean = existingIdIn.isEmpty
    if (!createNotUpdate && typeIn == Util.RELATION_TYPE_TYPE) require(previousDirectionalityIn.isDefined)
    val maxNameLength = {
      if (typeIn == Util.RELATION_TYPE_TYPE) model.RelationType.getNameLength(db)
      else if (typeIn == Util.ENTITY_TYPE) model.Entity.nameLength(db)
      else throw new scala.Exception("invalid inType: " + typeIn)
    }
    val example = {
      if (typeIn == Util.RELATION_TYPE_TYPE) " (use 3rd-person verb like \"owns\"--might make output like sentences more consistent later on)"
      else ""
    }

    /** 2nd Long in return value is ignored in this particular case.
      */
    def askAndSave(defaultNameIn: Option[String] = None): Option[(Long, Long)] = {
      val nameOpt = ui.askForString(Some(Array[String](leadingTextIn.getOrElse(""),
                                                       "Enter " + typeIn + " name (up to " + maxNameLength + " characters" + example + "; ESC to cancel)")),
                                    None, defaultNameIn)
      if (nameOpt.isEmpty) None
      else {
        val name = nameOpt.get.trim()
        if (name.length <= 0) None
        else {
          // idea: this size check might be able to account better for the escaping that's done. Or just keep letting the exception handle it as is already
          // done in the caller of this.
          if (name.length > maxNameLength) {
            ui.displayText(Util.stringTooLongErrorMessage(maxNameLength).format(Util.tooLongMessage) + ".")
            askAndSave(Some(name))
          } else {
            if (Util.isDuplicationAProblem(model.Entity.isDuplicate(db, name, existingIdIn), duplicateNameProbablyOK, ui)) None
            else {
              if (typeIn == Util.ENTITY_TYPE) {
                if (createNotUpdate) {
                  val newId = model.Entity.createEntity(db, name, classIdIn).getId
                  Some(newId, 0L)
                } else {
                  db.updateEntityOnlyName(existingIdIn.get, name)
                  Some(existingIdIn.get, 0L)
                }
              } else if (typeIn == Util.RELATION_TYPE_TYPE) {
                val ans: Option[String] = Util.askForRelationDirectionality(previousDirectionalityIn, ui)
                if (ans.isEmpty) None
                else {
                  val directionalityStr: String = ans.get.trim().toUpperCase
                  val nameInReverseDirectionStr = Util.askForNameInReverseDirection(directionalityStr, maxNameLength, name, previousNameInReverseIn, ui)
                  if (createNotUpdate) {
                    val newId = new RelationType(db, db.createRelationType(name, nameInReverseDirectionStr, directionalityStr)).getId
                    Some(newId, 0L)
                  } else {
                    db.updateRelationType(existingIdIn.get, name, nameInReverseDirectionStr, directionalityStr)
                    Some(existingIdIn.get, 0L)
                  }
                }
              } else throw new scala.Exception("unexpected value: " + typeIn)
            }
          }
        }
      }
    }

    val result = tryAskingAndSaving[(Long, Long)](Util.stringTooLongErrorMessage(maxNameLength), askAndSave, previousNameIn)
    if (result.isEmpty) None
    else Some(new Entity(db, result.get._1))
  }

  /** Call a provided function (method?) "askAndSaveIn", which does some work that might throw a specific OmDatabaseException.  If it does throw that,
    * let the user know the problem and call askAndSaveIn again.  I.e., allow retrying if the entered data is bad, instead of crashing the app.
    */
  def tryAskingAndSaving[T](errorMsgIn: String, askAndSaveIn: (Option[String]) => Option[T],
                                   defaultNameIn: Option[String] = None): Option[T] = {
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
        if (cumulativeMsg.contains(Util.tooLongMessage)) {
          ui.displayText(errorMsgIn.format(Util.tooLongMessage) + cumulativeMsg + ".")
          tryAskingAndSaving[T](errorMsgIn, askAndSaveIn, defaultNameIn)
        } else throw e
    }
  }

  /**
    * @param classIdIn (1st parameter) should be None only if the call is intended to create; otherwise it is an edit.
    * @return None if user wants out, otherwise returns the new or updated classId and entityId.
    * */
  def askForAndWriteClassAndTemplateEntityName(classIdIn: Option[Long] = None, previousNameIn: Option[String] = None): Option[(Long, Long)] = {
    val createNotUpdate: Boolean = classIdIn.isEmpty
    val nameLength = model.EntityClass.nameLength(db)
    val oldTemplateNamePrompt = {
      if (createNotUpdate) ""
      else {
        val entityId = new EntityClass(db, classIdIn.get).getTemplateEntityId
        val templateEntityName = new Entity(db, entityId).getName
        " (which is currently \"" + templateEntityName + "\")"
      }
    }
    def askAndSave(defaultNameIn: Option[String]): Option[(Long, Long)] = {
      val nameOpt = ui.askForString(Some(Array("Enter class name (up to " + nameLength + " characters; will also be used for its template entity name" +
                                               oldTemplateNamePrompt + "; ESC to cancel): ")),
                                    None, defaultNameIn)
      if (nameOpt.isEmpty) None
      else {
        val name = nameOpt.get.trim()
        if (name.length() == 0) None
        else {
          if (Util.isDuplicationAProblem(EntityClass.isDuplicate(db, name, classIdIn), duplicateNameProbablyOK = false, ui)) None
          else {
            if (createNotUpdate) Some(db.createClassAndItsTemplateEntity(name))
            else {
              val entityId: Long = db.updateClassAndTemplateEntityName(classIdIn.get, name)
              Some(classIdIn.get, entityId)
            }
          }
        }
      }
    }

    tryAskingAndSaving[(Long, Long)](Util.stringTooLongErrorMessage(nameLength), askAndSave, previousNameIn)
  }

  /** SEE DESCRIPTIVE COMMENT ON askForAndWriteClassAndTemplateEntityName, WHICH APPLIES TO all such METHODS (see this cmt elsewhere).
    * @return The instance's id, or None if there was a problem or the user wants out.
    * */
  def askForAndWriteOmInstanceInfo(idIn: Option[String] = None, previousAddressIn: Option[String] = None): Option[String] = {
    val createNotUpdate: Boolean = idIn.isEmpty
    val addressLength = model.OmInstance.addressLength
    def askAndSave(defaultNameIn: Option[String]): Option[String] = {
      val addressOpt = ui.askForString(Some(Array("Enter the internet address with optional port of a remote OneModel instance (for " +
                                                  "example, \"om.example.com:9000\", up to " + addressLength + " characters; ESC to cancel;" +
                                                  " Other examples include (omit commas):  localhost,  127.0.0.1:2345,  ::1 (?)," +
                                                  "  my.example.com:80,  your.example.com:8080  .): ")), None, defaultNameIn)
      if (addressOpt.isEmpty) None
      else {
        val address = addressOpt.get.trim()
        if (address.length() == 0) None
        else {
          if (Util.isDuplicationAProblem(OmInstance.isDuplicate(db, address, idIn), duplicateNameProbablyOK = false, ui)) None
          else {
            val restDb = new RestDatabase(address)
            val remoteId: Option[String] = restDb.getIdWithOptionalErrHandling(Some(ui))
            if (remoteId.isEmpty) {
              None
            } else {
              if (createNotUpdate) {
                OmInstance.create(db, remoteId.get, address)
                remoteId
              } else {
                val oldOmInstance: OmInstance = new OmInstance(db, idIn.get)
                if (idIn.get == remoteId.get) {
                  db.updateOmInstance(idIn.get, address, oldOmInstance.getEntityId)
                  idIn
                } else {
                  val ans: Option[Boolean] = ui.askYesNoQuestion("The IDs of the old and new remote instances don't match (old " +
                                                                 "id/address: " + idIn.get + "/" + oldOmInstance.getAddress + ", new id/address: " +
                                                                 remoteId.get + "/" + address + ".  Instead of updating the old one, you should create a new" +
                                                                 " entry for the new remote instance and then optionally delete this old one." +
                                                                 "  Do you want to create the new entry with this new address, now?")
                  if (ans.isDefined && ans.get) {
                    val id: String = OmInstance.create(db, remoteId.get, address).getId
                    ui.displayText("Created the new entry for \"" + address + "\".  You still have to delete the old one (" + idIn.get + "/" +
                                   oldOmInstance.getAddress + ") if you don't want it to be there.")
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

    tryAskingAndSaving[String](Util.stringTooLongErrorMessage(addressLength), askAndSave, previousAddressIn)
  }

  /* NOTE: converting the parameters around here from DataHolder to Attribute... means also making the Attribute
  classes writable, and/or
     immutable and recreating them whenever there's a change, but also needing a way to pass around
     partial attribute data in a way that can be shared by code, like return values from the get[AttributeData...]
     methods.
     Need to learn more scala so I can do the equivalent of passing a Tuple without specifying the size in signatures?
   */
  def askForInfoAndUpdateAttribute[T <: AttributeDataHolder](dhIn: T, askForAttrTypeId: Boolean, attrType: String, promptForSelectingTypeId: String,
                                                             getOtherInfoFromUser: (T, Boolean, TextUI) => Option[T],
                                                             updateTypedAttribute: (T) => Unit) {
    //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) within this method, below!
    @tailrec def askForInfoAndUpdateAttribute_helper(dhIn: T, attrType: String, promptForTypeId: String) {
      val ans: Option[T] = askForAttributeData[T](dhIn, askForAttrTypeId, attrType, Some(promptForTypeId), Some(new Entity(db, dhIn.attrTypeId).getName),
                                                  Some(dhIn.attrTypeId), getOtherInfoFromUser, editingIn = true)
      if (ans.isDefined) {
        val dhOut: T = ans.get
        val ans2: Option[Int] = Util.promptWhetherTo1Add2Correct(attrType, ui)

        if (ans2.isEmpty) Unit
        else if (ans2.get == 1) {
          updateTypedAttribute(dhOut)
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
    val leadingText: Array[String] = Array("Attribute: " + attributeIn.getDisplayString(0, None, None))
    var firstChoices = Array("Edit the attribute type, " +
                             (if (Util.canEditAttributeOnSingleLine(attributeIn)) "content (single line)," else "") +
                             " and valid/observed dates",

                             if (attributeIn.isInstanceOf[TextAttribute]) "Edit (as multiline value)" else "(stub)",
                             if (Util.canEditAttributeOnSingleLine(attributeIn)) "Edit the attribute content (single line)" else "(stub)",
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
                                                                      askForAttrTypeId = true, Util.QUANTITY_TYPE, Util.quantityTypePrompt,
                                                                      askForQuantityAttributeNumberAndUnit, updateQuantityAttribute)
            //force a reread from the DB so it shows the right info on the repeated menu:
            attributeEditMenu(new QuantityAttribute(db, attributeIn.getId))
          case textAttribute: TextAttribute =>
            def updateTextAttribute(dhInOut: TextAttributeDataHolder) {
              textAttribute.update(dhInOut.attrTypeId, dhInOut.text, dhInOut.validOnDate, dhInOut.observationDate)
            }
            val textAttributeDH: TextAttributeDataHolder = new TextAttributeDataHolder(textAttribute.getAttrTypeId, textAttribute.getValidOnDate,
                                                                                       textAttribute.getObservationDate, textAttribute.getText)
            askForInfoAndUpdateAttribute[TextAttributeDataHolder](textAttributeDH, askForAttrTypeId = true, Util.TEXT_TYPE,
                                                                  "CHOOSE TYPE OF " + Util.textDescription + ":",
                                                                  Util.askForTextAttributeText, updateTextAttribute)
            //force a reread from the DB so it shows the right info on the repeated menu:
            attributeEditMenu(new TextAttribute(db, attributeIn.getId))
          case dateAttribute: DateAttribute =>
            def updateDateAttribute(dhInOut: DateAttributeDataHolder) {
              dateAttribute.update(dhInOut.attrTypeId, dhInOut.date)
            }
            val dateAttributeDH: DateAttributeDataHolder = new DateAttributeDataHolder(dateAttribute.getAttrTypeId, dateAttribute.getDate)
            askForInfoAndUpdateAttribute[DateAttributeDataHolder](dateAttributeDH, askForAttrTypeId = true, Util.DATE_TYPE, "CHOOSE TYPE OF DATE:",
                                                                  Util.askForDateAttributeValue, updateDateAttribute)
            //force a reread from the DB so it shows the right info on the repeated menu:
            attributeEditMenu(new DateAttribute(db, attributeIn.getId))
          case booleanAttribute: BooleanAttribute =>
            def updateBooleanAttribute(dhInOut: BooleanAttributeDataHolder) {
              booleanAttribute.update(dhInOut.attrTypeId, dhInOut.boolean, dhInOut.validOnDate, dhInOut.observationDate)
            }
            val booleanAttributeDH: BooleanAttributeDataHolder = new BooleanAttributeDataHolder(booleanAttribute.getAttrTypeId, booleanAttribute.getValidOnDate,
                                                                                                booleanAttribute.getObservationDate,
                                                                                                booleanAttribute.getBoolean)
            askForInfoAndUpdateAttribute[BooleanAttributeDataHolder](booleanAttributeDH, askForAttrTypeId = true, Util.BOOLEAN_TYPE,
                                                                     "CHOOSE TYPE OF TRUE/FALSE VALUE:", Util.askForBooleanAttributeValue,
                                                                     updateBooleanAttribute)
            //force a reread from the DB so it shows the right info on the repeated menu:
            attributeEditMenu(new BooleanAttribute(db, attributeIn.getId))
          case fa: FileAttribute =>
            def updateFileAttribute(dhInOut: FileAttributeDataHolder) {
              fa.update(Some(dhInOut.attrTypeId), Some(dhInOut.description))
            }
            val fileAttributeDH: FileAttributeDataHolder = new FileAttributeDataHolder(fa.getAttrTypeId, fa.getDescription, fa.getOriginalFilePath)
            askForInfoAndUpdateAttribute[FileAttributeDataHolder](fileAttributeDH, askForAttrTypeId = true, Util.FILE_TYPE, "CHOOSE TYPE OF FILE:",
                                                                  Util.askForFileAttributeInfo, updateFileAttribute)
            //force a reread from the DB so it shows the right info on the repeated menu:
            attributeEditMenu(new FileAttribute(db, attributeIn.getId))
          case _ => throw new Exception("Unexpected type: " + attributeIn.getClass.getName)
        }
      } else if (answer == 2 && attributeIn.isInstanceOf[TextAttribute]) {
        val ta = attributeIn.asInstanceOf[TextAttribute]
        val newContent: String = Util.editMultilineText(ta.getText, ui)
        ta.update(ta.getAttrTypeId, newContent, ta.getValidOnDate, ta.getObservationDate)
        //then force a reread from the DB so it shows the right info on the repeated menu:
        attributeEditMenu(new TextAttribute(db, attributeIn.getId))
      } else if (answer == 3 && Util.canEditAttributeOnSingleLine(attributeIn)) {
        editAttributeOnSingleLine(attributeIn)
        false
      } else if (answer == 4) {
        val ans = ui.askYesNoQuestion("DELETE this attribute: ARE YOU SURE?")
        if (ans.isDefined && ans.get) {
          attributeIn.delete()
          true
        } else {
          ui.displayText("Did not delete attribute.", waitForKeystrokeIn = false)
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
          val file: Option[File] = ui.getExportDestination(fa.getOriginalFilePath, fa.getMd5Hash)
          if (file.isDefined) {
            fa.retrieveContent(file.get)
            ui.displayText("File saved at: " + file.get.getCanonicalPath)
          }
        } catch {
          case e: Exception =>
            val msg: String = Util.throwableToString(e)
            ui.displayText("Failed to export file, due to error: " + msg)
        }
        attributeEditMenu(attributeIn)
      } else {
        ui.displayText("invalid response")
        attributeEditMenu(attributeIn)
      }
    }
  }

  /**
   * @return Whether the user wants just to get out.
   */
  def editAttributeOnSingleLine(attributeIn: Attribute): Boolean = {
    require(Util.canEditAttributeOnSingleLine(attributeIn))

    attributeIn match {
      case quantityAttribute: QuantityAttribute =>
        val num: Option[Float] = Util.askForQuantityAttributeNumber(quantityAttribute.getNumber, ui)
        if (num.isDefined) {
          quantityAttribute.update(quantityAttribute.getAttrTypeId, quantityAttribute.getUnitId,
                                   num.get,
                                   quantityAttribute.getValidOnDate, quantityAttribute.getObservationDate)
        }
        num.isEmpty
      case textAttribute: TextAttribute =>
        val textAttributeDH: TextAttributeDataHolder = new TextAttributeDataHolder(textAttribute.getAttrTypeId, textAttribute.getValidOnDate,
                                                                                   textAttribute.getObservationDate, textAttribute.getText)
        val outDH: Option[TextAttributeDataHolder] = Util.askForTextAttributeText(textAttributeDH, inEditing = true, ui)
        if (outDH.isDefined) textAttribute.update(outDH.get.attrTypeId, outDH.get.text, outDH.get.validOnDate, outDH.get.observationDate)
        outDH.isEmpty
      case dateAttribute: DateAttribute =>
        val dateAttributeDH: DateAttributeDataHolder = new DateAttributeDataHolder(dateAttribute.getAttrTypeId, dateAttribute.getDate)
        val outDH: Option[DateAttributeDataHolder] = Util.askForDateAttributeValue(dateAttributeDH, inEditing = true, ui)
        if (outDH.isDefined) dateAttribute.update(outDH.get.attrTypeId, outDH.get.date)
        outDH.isEmpty
      case booleanAttribute: BooleanAttribute =>
        val booleanAttributeDH: BooleanAttributeDataHolder = new BooleanAttributeDataHolder(booleanAttribute.getAttrTypeId, booleanAttribute.getValidOnDate,
                                                                                            booleanAttribute.getObservationDate,
                                                                                            booleanAttribute.getBoolean)
        val outDH: Option[BooleanAttributeDataHolder] = Util.askForBooleanAttributeValue(booleanAttributeDH, inEditing = true, ui)
        if (outDH.isDefined) booleanAttribute.update(outDH.get.attrTypeId, outDH.get.boolean, outDH.get.validOnDate, outDH.get.observationDate)
        outDH.isEmpty
      case rte: RelationToEntity =>
        val editedEntity: Option[Entity] = editEntityName(new Entity(db, rte.getRelatedId2))
        editedEntity.isEmpty
      case rtg: RelationToGroup =>
        val editedGroupName: Option[String] = Util.editGroupName(new Group(db, rtg.getGroupId), ui)
        editedGroupName.isEmpty
      case _ => throw new scala.Exception("Unexpected type: " + attributeIn.getClass.getName)
    }
  }

  /**
   * @return (See addAttribute method.)
   */
  def askForInfoAndAddAttribute[T <: AttributeDataHolder](dhIn: T, askForAttrTypeId: Boolean, attrType: String, promptForSelectingTypeId: Option[String],
                                                          getOtherInfoFromUser: (T, Boolean, TextUI) => Option[T],
                                                          addTypedAttribute: (T) => Option[Attribute]): Option[Attribute] = {
    val ans: Option[T] = askForAttributeData[T](dhIn, askForAttrTypeId, attrType, promptForSelectingTypeId, None, None, getOtherInfoFromUser, editingIn = false)
    if (ans.isDefined) {
      val dhOut: T = ans.get
      addTypedAttribute(dhOut)
    } else None
  }

  def getExampleAffectedGroupsDescriptions(groupCount: Long, entityId: Long): (String) = {
    if (groupCount == 0) {
      ""
    } else {
      val limit = 10
      val delimiter = ", "
      // (BUG: see comments in psql.java re "OTHER ENTITY NOTED IN A DELETION BUG")
      val descrArray = db.getContainingRelationToGroupDescriptions(entityId, Some(limit))
      var descriptions = ""
      var counter = 0
      for (s: String <- descrArray) {
        counter += 1
        descriptions += counter + ") " + s + delimiter
      }
      descriptions.substring(0, math.max(0, descriptions.length - delimiter.length)) + ".  "
    }
  }

  /** @return whether entity was deleted.
    */
  def deleteEntity(entityIn: Entity): Boolean = {
    //IDEA: could combine this method with the following two. The only differences as of now are 3 strings and a method call, easily parameterized. Not
    //doing it immediately in case they diverge again soon.
    val name = entityIn.getName
    val groupCount: Long = db.getCountOfGroupsContainingEntity(entityIn.getId)
    val affectedExamples = getExampleAffectedGroupsDescriptions(groupCount, entityIn.getId)
    val effectMsg =  "This will ALSO remove it from " + groupCount + " groups, including for example these relations " +
      " that refer to this entity (showing entities & their relations to groups, as \"entity -> group\"): " + affectedExamples
    // idea: WHEN CONSIDERING MODS TO THIS, ALSO CONSIDER THE Q'S ASKED AT CODE CMT WHERE DELETING A GROUP OF ENTITIES (SEE, for example "recursively").
    // (and in the other 2 methods just like this)
    val warningMsg = "DELETE ENTITY \"" + name + "\" (and " + Util.entityPartsThatCanBeAffected + ").  " + effectMsg + "**ARE YOU REALLY SURE?**"
    val ans = ui.askYesNoQuestion(warningMsg, Some("n"))
    if (ans.isDefined && ans.get) {
      entityIn.delete()
      ui.displayText("Deleted entity \"" + name + "\"" + ".")
      true
    } else {
      ui.displayText("Did not delete entity.", waitForKeystrokeIn = false)
      false
    }
  }

  /** @return whether entity was archived.
    */
  def archiveEntity(entityIn: Entity): Boolean = {
    val name = entityIn.getName
    val groupCount: Long = db.getCountOfGroupsContainingEntity(entityIn.getId)
    val affectedExamples = getExampleAffectedGroupsDescriptions(groupCount, entityIn.getId)
    val effectMsg = "This will affect affect its visibility in " + groupCount + " groups, including for example these relations " +
                    " that refer to this entity (showing entities & their relations to groups, as \"entity -> group\"): " + affectedExamples
    // idea: WHEN CONSIDERING MODS TO THIS, ALSO CONSIDER THE Q'S ASKED AT CODE CMT WHERE DELETING A GROUP OF ENTITIES (SEE, for example "recursively").
    // (and in the other 2 methods just like this)
    val warningMsg = "ARCHIVE ENTITY \"" + name + "\" (and " + Util.entityPartsThatCanBeAffected + ").  " + effectMsg + "**ARE YOU REALLY SURE?**"
    val ans = ui.askYesNoQuestion(warningMsg, Some(""))
    if (ans.isDefined && ans.get) {
      entityIn.archive()
      ui.displayText("Archived entity \"" + name + "\"" + ".", waitForKeystrokeIn = false)
      true
    } else {
      ui.displayText("Did not archive entity.", waitForKeystrokeIn = false)
      false
    }
  }

  /** @return whether entity was un-archived.
    */
  def unarchiveEntity(entityIn: Entity): Boolean = {
    val name = entityIn.getName
    val groupCount: Long = db.getCountOfGroupsContainingEntity(entityIn.getId)
    val affectedExamples = getExampleAffectedGroupsDescriptions(groupCount, entityIn.getId)
    val effectMsg = "This will affect affect its visibility in " + groupCount + " groups, including for example these relations " +
      " that refer to this entity (showing entities & their relations to groups, as \"entity -> group\"): " + affectedExamples
    // idea: WHEN CONSIDERING MODS TO THIS, ALSO CONSIDER THE Q'S ASKED AT CODE CMT WHERE DELETING A GROUP OF ENTITIES (SEE, for example "recursively").
    // (and in the other 2 methods just like this)
    val warningMsg = "un-archive entity \"" + name + "\" (and " + Util.entityPartsThatCanBeAffected + ").  " + effectMsg + "**ARE YOU REALLY SURE?**"
    val ans = ui.askYesNoQuestion(warningMsg, Some(""))
    if (ans.isDefined && ans.get) {
      entityIn.unarchive()
      ui.displayText("Un-archived entity \"" + name + "\"" + ".", waitForKeystrokeIn = false)
      true
    } else {
      ui.displayText("Did not un-archive entity.", waitForKeystrokeIn = false)
      false
    }
  }

  /**
   * SEE DESCRIPTIVE COMMENT ON askForAndWriteClassAndTemplateEntityName, WHICH APPLIES TO all such METHODS (see this cmt elsewhere).
   *
   * @return None if user wants out.
   */
  def editEntityName(entityIn: Entity): Option[Entity] = {
    val editedEntity: Option[Entity] = entityIn match {
      case relTypeIn: RelationType =>
        val previousNameInReverse: String = relTypeIn.getNameInReverseDirection //idea: check: this edits name w/ prefill also?:
        askForNameAndWriteEntity(Util.RELATION_TYPE_TYPE, Some(relTypeIn.getId), Some(relTypeIn.getName), Some(relTypeIn.getDirectionality),
                                 if (previousNameInReverse == null || previousNameInReverse.trim().isEmpty) None else Some(previousNameInReverse),
                                 None)
      case entity: Entity =>
        val entityNameBeforeEdit: String = entityIn.getName
        val editedEntity: Option[Entity] = askForNameAndWriteEntity(Util.ENTITY_TYPE, Some(entity.getId), Some(entity.getName), None, None, None)
        if (editedEntity.isDefined) {
          val entityNameAfterEdit: String = editedEntity.get.getName
          if (entityNameBeforeEdit != entityNameAfterEdit) {
            val (_, _, groupId, moreThanOneAvailable) = db.findRelationToAndGroup_OnEntity(editedEntity.get.getId)
            if (groupId.isDefined && !moreThanOneAvailable) {
              val attrCount = entityIn.getAttrCount
              // for efficiency, if it's obvious which subgroup's name to change at the same time, offer to do so
              val defaultAnswer = if (attrCount > 1) Some("n") else Some("y")
              val ans = ui.askYesNoQuestion("There's a single subgroup" +
                                            (if (attrCount > 1) " (***AMONG " + (attrCount - 1) + " OTHER ATTRIBUTES***)" else "") +
                                            "; possibly it and this entity were created at the same time.  Also change" +
                                            " the subgroup's name now to be identical?", defaultAnswer)
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
                                                              " set under main menu / " + Util.menuText_viewPreferences + ")",
                                                              if (defaultForPrompt.isEmpty) Some("") else if (defaultForPrompt.get) Some("y") else Some("n"),
                                                              allowBlankAnswer = true)
    valueAfterEdit
  }

  /**
   * @return None means "get out", or Some(choiceNum) if a choice was made.
   */
  def askWhetherDeleteOrArchiveEtc(entityIn: Entity, relationIn: Option[RelationToEntity], relationSourceEntityIn: Option[Entity],
                                   containingGroupIn: Option[Group]): (Option[Int], Int, Int, Int) = {
    val groupCount: Long = db.getCountOfGroupsContainingEntity(entityIn.getId)
    val (entityCountNonArchived, entityCountArchived) = db.getCountOfEntitiesContainingEntity(entityIn.getId)
    val leadingText = Some(Array("Choose a deletion or archiving option:  (The entity is " +
                                 Util.getContainingEntitiesDescription(entityCountNonArchived, entityCountArchived) + ", and " + groupCount + " groups.)"))
    var choices = Array("Delete this entity",
                        if (entityIn.isArchived) {
                          "Un-archive this entity"
                        } else {
                          "Archive this entity (remove from visibility but not permanent/total deletion)"
                        })
    val delEntityLink_choiceNumber: Int = 3
    var delFromContainingGroup_choiceNumber: Int = 3
    var showAllArchivedEntities_choiceNumber: Int = 3
    // (check for existence because other things could have been deleted or archived while browsing around different menu options.)
    if (relationIn.isDefined && relationSourceEntityIn.isDefined && db.entityKeyExists(relationSourceEntityIn.get.getId)) {
     // means we got here by selecting a Relation attribute on another entity, so entityIn is the "entityId2" in that relation; so show some options,
      // because
      // we eliminated a separate menu just for the relation and put them here, for UI usage simplicity.
      choices = choices :+ "Delete the link between the linking (or containing) entity: \"" + relationSourceEntityIn.get.getName + "\", " +
                           "and this one: \"" + entityIn.getName + "\""
      delFromContainingGroup_choiceNumber += 1
      showAllArchivedEntities_choiceNumber += 1
    }
    if (containingGroupIn.isDefined) {
      choices = choices :+ "Delete the link between the group: \"" + containingGroupIn.get.getName + "\", and this Entity: \"" + entityIn.getName
      showAllArchivedEntities_choiceNumber += 1
    }
    choices = choices :+ (if (!db.includeArchivedEntities) "Show archived entities" else "Do not show archived entities")

    val delOrArchiveAnswer: Option[(Int)] = ui.askWhich(leadingText, choices, Array[String]())
    (delOrArchiveAnswer, delEntityLink_choiceNumber, delFromContainingGroup_choiceNumber, showAllArchivedEntities_choiceNumber)
  }

  /** Returns data, or None if user wants to cancel/get out.
    * @param attrType Constant referring to Attribute subtype, as used by the inObjectType parameter to the chooseOrCreateObject method
    *                 (e.g., Controller.QUANTITY_TYPE).  See comment on that method, for that parm.
    * */
  def askForAttributeData[T <: AttributeDataHolder](inoutDH: T, alsoAskForAttrTypeId: Boolean, attrType: String, attrTypeInputPrompt: Option[String],
                                                    inPreviousSelectionDesc: Option[String], inPreviousSelectionId: Option[Long],
                                                    askForOtherInfo: (T, Boolean, TextUI) => Option[T], editingIn: Boolean): Option[T] = {
    val (userWantsOut: Boolean, attrTypeId: Long, isRemote, remoteKey) = {
      if (alsoAskForAttrTypeId) {
        require(attrTypeInputPrompt.isDefined)
        val ans: Option[(IdWrapper, Boolean, String)] = chooseOrCreateObject(Some(List(attrTypeInputPrompt.get)), inPreviousSelectionDesc,
                                                                             inPreviousSelectionId, attrType)
        if (ans.isEmpty) {
          (true, 0L, false, "")
        } else {
          (false, ans.get._1.getId, ans.get._2, ans.get._3)
        }
      } else {
        // maybe not ever reached under current system logic. not certain.
        val (isRemote, remoteKey) = {
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
      val ans2: Option[T] = askForOtherInfo(inoutDH, editingIn, ui)
      if (ans2.isEmpty) None
      else {
        var userWantsToCancel = false
        // (the ide/intellij preferred to have it this way instead of 'if')
        inoutDH match {
          case dhWithVOD: AttributeDataHolderWithVODates =>
            val (validOnDate: Option[Long], observationDate: Long, userWantsToCancelInner: Boolean) =
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
  @tailrec final def findExistingObjectByText(startingDisplayRowIndexIn: Long = 0, attrTypeIn: String,
                                              //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) w/in this method!
                                              idToOmitIn: Option[Long] = None, regexIn: String): Option[IdWrapper] = {
    val leadingText = List[String]("SEARCH RESULTS: " + Util.pickFromListPrompt)
    val choices: Array[String] = Array(Util.listNextItemsPrompt)
    val numDisplayableItems = ui.maxColumnarChoicesToDisplayAfter(leadingText.size, choices.length, Util.maxNameLength)

    val objectsToDisplay = attrTypeIn match {
      case Util.ENTITY_TYPE =>
        db.getMatchingEntities(startingDisplayRowIndexIn, Some(numDisplayableItems), idToOmitIn, regexIn)
      case Util.GROUP_TYPE =>
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
                                                                        numSubgroupsPrefix + entity.getArchivedStatusDisplayString + entity.getName
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
              case Util.ENTITY_TYPE =>
                new EntityMenu(ui, db, this).entityMenu(o.asInstanceOf[Entity])
              case Util.GROUP_TYPE =>
                // for now, picking the first RTG found for this group, until it's clear which of its RTGs to use.
                // (see also the other locations w/ similar comment!)
                // (There is probably no point in showing this GroupMenu with RTG info, since which RTG to use was picked arbitrarily, except if
                // that added info is a convenience, or if it helps the user clean up orphaned data sometimes.)
                val someRelationToGroups: java.util.ArrayList[RelationToGroup] = db.getRelationToGroupsByGroup(o.asInstanceOf[Group].getId, 0, Some(1))
                if (someRelationToGroups.size < 1) {
                  ui.displayText(Util.ORPHANED_GROUP_MESSAGE)
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
              case Util.ENTITY_TYPE =>
                Some(new IdWrapper(o.asInstanceOf[Entity].getId))
              case Util.GROUP_TYPE =>
                Some(new IdWrapper(o.asInstanceOf[Group].getId))
              case _ =>
                throw new OmException("??")
            }
          }
        } else {
          ui.displayText("unknown choice among secondary list")
          findExistingObjectByText(startingDisplayRowIndexIn, attrTypeIn, idToOmitIn, regexIn)
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
    * Idea: the inAttrType parm: do like in java & make it some kind of enum for type-safety? What's the scala idiom for that? (see also other
    * mentions of inAttrType for others to fix as well.)
    */
  /*@tailrec  //idea (and is tracked):  putting this back gets compiler error on line 1218 call to chooseOrCreateObject. */
  final def chooseOrCreateObject(leadingTextIn: Option[List[String]], previousSelectionDescIn: Option[String],
                                          previousSelectionIdIn: Option[Long], objectTypeIn: String, startingDisplayRowIndexIn: Long = 0,
                                          classIdIn: Option[Long] = None, limitByClassIn: Boolean = false,
                                          containingGroupIn: Option[Long] = None,
                                          //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) w/in this method!
                                          markPreviousSelectionIn: Boolean = false): Option[(IdWrapper, Boolean, String)] = {
    if (classIdIn.isDefined) require(objectTypeIn == Util.ENTITY_TYPE)
    val nonRelationAttrTypeNames = Array(Util.TEXT_TYPE, Util.QUANTITY_TYPE, Util.DATE_TYPE, Util.BOOLEAN_TYPE, Util.FILE_TYPE)
    val mostAttrTypeNames = Array(Util.ENTITY_TYPE, Util.TEXT_TYPE, Util.QUANTITY_TYPE, Util.DATE_TYPE, Util.BOOLEAN_TYPE,
                                  Util.FILE_TYPE)
    val relationAttrTypeNames = Array(Util.RELATION_TYPE_TYPE, Util.RELATION_TO_ENTITY_TYPE, Util.RELATION_TO_GROUP_TYPE)
    val evenMoreAttrTypeNames = Array(Util.ENTITY_TYPE, Util.TEXT_TYPE, Util.QUANTITY_TYPE, Util.DATE_TYPE, Util.BOOLEAN_TYPE,
                                      Util.FILE_TYPE, Util.RELATION_TYPE_TYPE, Util.RELATION_TO_ENTITY_TYPE,
                                      Util.RELATION_TO_GROUP_TYPE)
    val listNextItemsChoiceNum = 1

    // Attempt to keep these straight even though the size of the list, hence their option #'s on the menu,
    // is conditional:
    def getChoiceList: (Array[String], Int, Int, Int, Int, Int, Int, Int, Int, Int) = {
      var keepPreviousSelectionChoiceNum = 1
      var createAttrTypeChoiceNum = 1
      var searchForEntityByNameChoiceNum = 1
      var searchForEntityByIdChoiceNum = 1
      var showJournalChoiceNum = 1
      var linkToRemoteInstanceChoiceNum = 1
      var createRelationTypeChoiceNum = 1
      var createClassChoiceNum = 1
      var createInstanceChoiceNum = 1
      var choiceList = Array(Util.listNextItemsPrompt)
      if (previousSelectionDescIn.isDefined) {
        choiceList = choiceList :+ "Keep previous selection (" + previousSelectionDescIn.get + ")."
        keepPreviousSelectionChoiceNum += 1
        createAttrTypeChoiceNum += 1
        searchForEntityByNameChoiceNum += 1
        searchForEntityByIdChoiceNum += 1
        showJournalChoiceNum += 1
        createRelationTypeChoiceNum += 1
        createClassChoiceNum += 1
        createInstanceChoiceNum += 1
      }
      //idea: use match instead of if: can it do || ?
      if (mostAttrTypeNames.contains(objectTypeIn)) {
        choiceList = choiceList :+ Util.menuText_createEntityOrAttrType
        createAttrTypeChoiceNum += 1
        choiceList = choiceList :+ "Search for existing entity by name and text attribute content..."
        searchForEntityByNameChoiceNum += 2
        choiceList = choiceList :+ "Search for existing entity by id..."
        searchForEntityByIdChoiceNum += 3
        choiceList = choiceList :+ "Show journal (changed entities) by date range..."
        showJournalChoiceNum += 4
        choiceList = choiceList :+ "Link to entity in a separate (REMOTE) OM instance..."
        linkToRemoteInstanceChoiceNum += 5
        createRelationTypeChoiceNum += 5
        createClassChoiceNum += 5
        createInstanceChoiceNum += 5
      } else if (relationAttrTypeNames.contains(objectTypeIn)) {
        choiceList = choiceList :+ Util.menuText_createRelationType
        //idea: consider how to clarify how these next "...choiceNum"s work, & maybe see how to refactor this so is cleaner. Maybe fix per
        // Fowler's Refactoring book?
        // Note that these 3 are managed together and it works because they have different other criteria below for choosing a code block based on them.
        createRelationTypeChoiceNum += 1
      } else if (objectTypeIn == Util.ENTITY_CLASS_TYPE) {
        choiceList = choiceList :+ "Create new class (template for new entities)"
        createClassChoiceNum += 1
      } else if (objectTypeIn == Util.OM_INSTANCE_TYPE) {
        choiceList = choiceList :+ "Create new OM instance (a remote data store for lookup, linking, etc.)"
        createInstanceChoiceNum += 1
      } else throw new Exception("invalid inAttrType: " + objectTypeIn)

      (choiceList, keepPreviousSelectionChoiceNum, createAttrTypeChoiceNum, searchForEntityByNameChoiceNum, searchForEntityByIdChoiceNum, showJournalChoiceNum, createRelationTypeChoiceNum, createClassChoiceNum, createInstanceChoiceNum, linkToRemoteInstanceChoiceNum)
    }

    def getLeadTextAndObjectList(choicesIn: Array[String]): (List[String],
      java.util.ArrayList[_ >: RelationType with OmInstance with EntityClass <: Object],
      Array[String])
    = {
      val prefix: String = objectTypeIn match {
        case Util.ENTITY_TYPE => "ENTITIES: "
        case Util.QUANTITY_TYPE => "QUANTITIES (entities): "
        case Util.TEXT_TYPE => "TEXT ATTRIBUTES (entities): "
        case Util.DATE_TYPE => "DATE ATTRIBUTES (entities): "
        case Util.BOOLEAN_TYPE => "TRUE/FALSE ATTRIBUTES (entities): "
        case Util.FILE_TYPE => "FILE ATTRIBUTES (entities): "
        case Util.RELATION_TYPE_TYPE => "RELATION TYPES: "
        case Util.ENTITY_CLASS_TYPE => "CLASSES: "
        case Util.OM_INSTANCE_TYPE => "OneModel INSTANCES: "
        case Util.RELATION_TO_ENTITY_TYPE => "RELATION TYPES: "
        case Util.RELATION_TO_GROUP_TYPE => "RELATION TYPES: "
        case _ => ""
      }
      var leadingText = leadingTextIn.getOrElse(List[String](prefix + "Pick from menu, or an item by letter; Alt+<letter> to go to the item & later come back)"))
      val numDisplayableItems = ui.maxColumnarChoicesToDisplayAfter(leadingText.size + 3 /* up to: see more of leadingText below .*/ , choicesIn.length,
                                                                    Util.maxNameLength)
      val objectsToDisplay = {
        // ** KEEP THESE QUERIES AND CONDITIONS IN SYNC W/ THE COROLLARY ONES 2x BELOW ! (at similar comment)
        if (nonRelationAttrTypeNames.contains(objectTypeIn)) db.getEntities(startingDisplayRowIndexIn, Some(numDisplayableItems))
        else if (objectTypeIn == Util.ENTITY_TYPE) db.getEntitiesOnly(startingDisplayRowIndexIn, Some(numDisplayableItems), classIdIn, limitByClassIn,
                                                                           previousSelectionIdIn,
                                                                           containingGroupIn)
        else if (relationAttrTypeNames.contains(objectTypeIn)) {
          db.getRelationTypes(startingDisplayRowIndexIn, Some(numDisplayableItems)).asInstanceOf[java.util.ArrayList[RelationType]]
        }
        else if (objectTypeIn == Util.ENTITY_CLASS_TYPE) db.getClasses(startingDisplayRowIndexIn, Some(numDisplayableItems))
        else if (objectTypeIn == Util.OM_INSTANCE_TYPE) db.getOmInstances()
        else throw new Exception("invalid inAttrType: " + objectTypeIn)
      }
      if (objectsToDisplay.size == 0) {
        // IF THIS CHANGES: change the guess at the 1st parameter to maxColumnarChoicesToDisplayAfter, JUST ABOVE!
        val txt: String = TextUI.NEWLN + TextUI.NEWLN + "(None of the needed " + (if (objectTypeIn == Util.RELATION_TYPE_TYPE) "relation types" else "entities") +
                          " have been created in this model, yet."
        leadingText = leadingText ::: List(txt)
      }
      val totalExisting: Long = {
        // ** KEEP THESE QUERIES AND CONDITIONS IN SYNC W/ THE COROLLARY ONES 2x ELSEWHERE ! (at similar comment)
        if (nonRelationAttrTypeNames.contains(objectTypeIn)) db.getEntitiesOnlyCount(classIdIn, limitByClassIn, previousSelectionIdIn)
        else if (objectTypeIn == Util.ENTITY_TYPE) db.getEntitiesOnlyCount(classIdIn, limitByClassIn, previousSelectionIdIn)
        else if (relationAttrTypeNames.contains(objectTypeIn)) db.getRelationTypeCount
        else if (objectTypeIn == Util.ENTITY_CLASS_TYPE) db.getClassCount()
        else if (objectTypeIn == Util.OM_INSTANCE_TYPE) db.getOmInstanceCount
        else throw new Exception("invalid inAttrType: " + objectTypeIn)
      }
      Util.addRemainingCountToPrompt(choicesIn, objectsToDisplay.size, totalExisting, startingDisplayRowIndexIn)
      val objectStatusesAndNames: Array[String] = objectsToDisplay.toArray.map {
                                                                      case entity: Entity => entity.getArchivedStatusDisplayString + entity.getName
                                                                      case clazz: EntityClass => clazz.getName
                                                                      case omInstance: OmInstance => omInstance.getDisplayString
                                                                      case x: Any => throw new Exception("unexpected class: " + x.getClass.getName)
                                                                      case _ => throw new Exception("??")
                                                                    }
      (leadingText, objectsToDisplay, objectStatusesAndNames)
    }

    def getNextStartingObjectIndex(previousListLength: Long, nonRelationAttrTypeNames: Array[String], relationAttrTypeNames: Array[String]): Long = {
      val index = {
        val x = startingDisplayRowIndexIn + previousListLength
        // ask Model for list of obj's w/ count desired & starting index (or "first") (in a sorted map, w/ id's as key, and names)
        //idea: should this just reuse the "totalExisting" value alr calculated in above in getLeadTextAndObjectList just above?
        val numObjectsInModel =
        // ** KEEP THESE QUERIES AND CONDITIONS IN SYNC W/ THE COROLLARY ONES 2x ABOVE ! (at similar comment)
          if (nonRelationAttrTypeNames.contains(objectTypeIn))
            db.getEntityCount
          else if (objectTypeIn == Util.ENTITY_TYPE) db.getEntitiesOnlyCount(classIdIn, limitByClassIn)
          else if (relationAttrTypeNames.contains(objectTypeIn))
            db.getRelationTypeCount
          else if (objectTypeIn == Util.ENTITY_CLASS_TYPE) db.getClassCount()
          else if (objectTypeIn == Util.OM_INSTANCE_TYPE) db.getOmInstanceCount
          else throw new Exception("invalid inAttrType: " + objectTypeIn)
        if (x >= numObjectsInModel) {
          ui.displayText("End of list found; starting over from the beginning.")
          0 // start over
        } else x
      }
      index
    }

    val (choices, keepPreviousSelectionChoice, createEntityOrAttrTypeChoice, searchForEntityByNameChoice, searchForEntityByIdChoice, showJournalChoice, createRelationTypeChoice, createClassChoice, createInstanceChoice, linkToRemoteInstanceChoice): (Array[String],
      Int, Int, Int, Int, Int, Int, Int, Int, Int) = getChoiceList

    val (leadingText, objectsToDisplay, statusesAndNames) = getLeadTextAndObjectList(choices)
    val ans = ui.askWhichChoiceOrItsAlternate(Some(leadingText.toArray), choices, statusesAndNames)

    if (ans.isEmpty) None
    else {
      val answer = ans.get._1
      val userChoseAlternate = ans.get._2
      if (answer == listNextItemsChoiceNum && answer <= choices.length) {
        // (For reason behind " && answer <= choices.length", see comment where it is used in entityMenu.)
        val index: Long = getNextStartingObjectIndex(objectsToDisplay.size, nonRelationAttrTypeNames, relationAttrTypeNames)
        chooseOrCreateObject(leadingTextIn, previousSelectionDescIn, previousSelectionIdIn, objectTypeIn, index, classIdIn, limitByClassIn,
                             containingGroupIn, markPreviousSelectionIn)
      } else if (answer == keepPreviousSelectionChoice && answer <= choices.length) {
        // Such as if editing several fields on an attribute and doesn't want to change the first one.
        // Not using "get out" option for this because it would exit from a few levels at once and
        // then user wouldn't be able to proceed to other field edits.
        Some(new IdWrapper(previousSelectionIdIn.get), false, "")
      } else if (answer == createEntityOrAttrTypeChoice && answer <= choices.length) {
        val e: Option[Entity] = askForClassInfoAndNameAndCreateEntity(classIdIn)
        if (e.isEmpty) {
          None
        } else {
          Some(new IdWrapper(e.get.getId), false, "")
        }
      } else if (answer == searchForEntityByNameChoice && answer <= choices.length) {
        val result = askForNameAndSearchForEntity
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
        val yDate = new java.util.Date(System.currentTimeMillis() - (24 * 60 * 60 * 1000))
        val yesterday: String = new java.text.SimpleDateFormat("yyyy-MM-dd").format(yDate)
        val beginDate: Option[Long] = Util.askForDate_generic(Some("BEGINNING date in the time range: " + Util.genericDatePrompt), Some(yesterday), ui)
        if (beginDate.isEmpty) None
        else {
          val endDate: Option[Long] = Util.askForDate_generic(Some("ENDING date in the time range: " + Util.genericDatePrompt), None, ui)
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
        val result = searchById(Util.ENTITY_TYPE)
        if (result.isEmpty) {
          None
        } else {
          Some(result.get, false, "")
        }
      } else if (answer == linkToRemoteInstanceChoice && mostAttrTypeNames.contains(objectTypeIn) && answer <= choices.length) {
        val omInstanceIdOption: Option[(_, _, String)] = chooseOrCreateObject(None, None, None, Util.OM_INSTANCE_TYPE)
        if (omInstanceIdOption.isEmpty) {
          None
        } else {
          val remoteOmInstance = new OmInstance(db, omInstanceIdOption.get._3)
          val remoteEntityEntryTypeAnswer = ui.askWhich(leadingTextIn = Some(Array("SPECIFY AN ENTITY IN THE REMOTE INSTANCE")),
                                                        choicesIn = Array("Enter an entity id #", "Use the remote site's default entity"))
          if (remoteEntityEntryTypeAnswer.isEmpty) {
            None
          } else {
            val restDb = new RestDatabase(remoteOmInstance.getAddress)
            val remoteEntityId: Option[Long] = {
              if (remoteEntityEntryTypeAnswer.get == 1) {
                val remoteEntityAnswer = ui.askForString(Some(Array("Enter the remote entity's id # (for example, \"-9223372036854745151\"")),
                                                         Some(Util.isNumeric))
                if (remoteEntityAnswer.isEmpty) None
                else {
                  val id: String = remoteEntityAnswer.get.trim()
                  if (id.length() == 0) None
                  else  Some(id.toLong)
                }
              } else if (remoteEntityEntryTypeAnswer.get == 2) {
                val defaultEntityId: Option[Long] = restDb.getDefaultEntityWithOptionalErrHandling(Some(ui))
                if (defaultEntityId.isEmpty) None
                else defaultEntityId
              } else {
                None
              }
            }
            if (remoteEntityId.isEmpty) None
            else {
              val entityInJson: Option[String] = restDb.getEntityWithOptionalErrHandling(Some(ui), remoteEntityId.get)
              if (entityInJson.isEmpty) {
                None
              } else {
                val saveEntityAnswer: Option[Boolean] = ui.askYesNoQuestion("Here is the entity's data: " + TextUI.NEWLN + "======================" +
                                                                            entityInJson.get + TextUI.NEWLN + "======================" + TextUI.NEWLN +
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
      } else if (answer == createRelationTypeChoice && relationAttrTypeNames.contains(objectTypeIn) && answer <= choices.length) {
        val entity: Option[Entity] = askForNameAndWriteEntity(Util.RELATION_TYPE_TYPE)
        if (entity.isEmpty) None
        else Some(new IdWrapper(entity.get.getId), false, "")
      } else if (answer == createClassChoice && objectTypeIn == Util.ENTITY_CLASS_TYPE && answer <= choices.length) {
        val result: Option[(Long, Long)] = askForAndWriteClassAndTemplateEntityName()
        if (result.isEmpty) None
        else {
          val (classId, entityId) = result.get
          val ans = ui.askYesNoQuestion("Do you want to add attributes to the newly created template entity for this class? (These will be used for the " +
                                        "prompts " +
                                        "and defaults when creating/editing entities in this class).", Some("y"))
          if (ans.isDefined && ans.get) {
            new EntityMenu(ui, db, this).entityMenu(new Entity(db, entityId))
          }
          Some(new IdWrapper(classId), false, "")
        }
      } else if (answer == createInstanceChoice && objectTypeIn == Util.OM_INSTANCE_TYPE && answer <= choices.length) {
        val result: Option[String] = askForAndWriteOmInstanceInfo()
        if (result.isEmpty) {
          None
        } else {
          // using null on next line was easier than the visible alternatives (same in one other place w/ this comment)
          Some(null, false, result.get)
        }
      } else if (answer > choices.length && answer <= (choices.length + objectsToDisplay.size)) {
        // those in the condition on the previous line are 1-based, not 0-based.
        val index = answer - choices.length - 1
        // user typed a letter to select.. (now 0-based)
        // user selected a new object and so we return to the previous menu w/ that one displayed & current
        val o = objectsToDisplay.get(index)
        //if ("text,quantity,entity,date,boolean,file,relationtype".contains(attrTypeIn)) {
        //i.e., if (attrTypeIn == Controller.TEXT_TYPE || (= any of the other types...)):
        if (userChoseAlternate) {
          objectTypeIn match {
            // idea: replace this condition by use of a trait (the type of o, which has getId), or being smarter with scala's type system. attrTypeIn match {
            case Util.ENTITY_TYPE =>
              new EntityMenu(ui, db, this).entityMenu(o.asInstanceOf[Entity])
            case _ =>
              // (choosing a group doesn't call this, it calls chooseOrCreateGroup)
              throw new OmException("not yet implemented")
          }
          chooseOrCreateObject(leadingTextIn, previousSelectionDescIn, previousSelectionIdIn, objectTypeIn, startingDisplayRowIndexIn, classIdIn, limitByClassIn,
                               containingGroupIn, markPreviousSelectionIn)
        } else {
          if (evenMoreAttrTypeNames.contains(objectTypeIn)) Some(o.asInstanceOf[Entity].getIdWrapper, false, "")
          else if (objectTypeIn == Util.ENTITY_CLASS_TYPE) Some(o.asInstanceOf[EntityClass].getIdWrapper,false,  "")
          // using null on next line was easier than the visible alternatives (same in one other place w/ this comment)
          else if (objectTypeIn == Util.OM_INSTANCE_TYPE) Some(null, false, o.asInstanceOf[OmInstance].getId)
          else throw new Exception("invalid inAttrType: " + objectTypeIn)
        }
      } else {
        ui.displayText("unknown response in chooseOrCreateObject")
        chooseOrCreateObject(leadingTextIn, previousSelectionDescIn, previousSelectionIdIn, objectTypeIn, startingDisplayRowIndexIn, classIdIn,
                             limitByClassIn, containingGroupIn, markPreviousSelectionIn)
      }
    }
  }

  def askForNameAndSearchForEntity: Option[IdWrapper] = {
    val ans = ui.askForString(Some(Array(Util.searchPrompt(Util.ENTITY_TYPE))))
    if (ans.isEmpty) {
      None
    } else {
      // Allow relation to self (eg, picking self as 2nd part of a RelationToEntity), so None in 3nd parm.
      val e: Option[IdWrapper] = findExistingObjectByText(0, Util.ENTITY_TYPE, None, ans.get)
      if (e.isEmpty) None
      else Some(new IdWrapper(e.get.getId))
    }
  }

  def searchById(typeNameIn: String): Option[IdWrapper] = {
    require(typeNameIn == Util.ENTITY_TYPE || typeNameIn == Util.GROUP_TYPE)
    val ans = ui.askForString(Some(Array("Enter the " + typeNameIn + " ID to search for:")))
    if (ans.isEmpty) {
      None
    } else {
      // it's a long:
      val idString: String = ans.get
      if (!Util.isNumeric(idString)) {
        ui.displayText("Invalid ID format.  An ID is a numeric value between " + Database.minIdValue + " and " + Database.maxIdValue)
        None
      } else {
        // (BTW, do allow relation to self, e.g., picking self as 2nd part of a RelationToEntity.)
        // (Also, the call to entityKeyExists should here include archived entities so the user can find out if the one
        // needed is archived, even if the hard way.)
        if ((typeNameIn == Util.ENTITY_TYPE && db.entityKeyExists(idString.toLong)) ||
            (typeNameIn == Util.GROUP_TYPE && db.groupKeyExists(idString.toLong))) {
          Some(new IdWrapper(idString.toLong))
        } else {
          ui.displayText("The " + typeNameIn + " ID " + ans.get + " was not found in the database.")
          None
        }
      }
    }
  }

  /** Returns None if user wants to cancel. */
  def askForQuantityAttributeNumberAndUnit(dhIn: QuantityAttributeDataHolder, editingIn: Boolean, ui: TextUI): Option[QuantityAttributeDataHolder] = {
    val outDH: QuantityAttributeDataHolder = dhIn
    val leadingText: List[String] = List("SELECT A *UNIT* FOR THIS QUANTITY (i.e., centimeters, or quarts; ESC or blank to cancel):")
    val previousSelectionDesc = if (editingIn) Some(new Entity(db, dhIn.unitId).getName) else None
    val previousSelectionId = if (editingIn) Some(dhIn.unitId) else None
    val unitSelection: Option[(IdWrapper, _, _)] = chooseOrCreateObject(Some(leadingText), previousSelectionDesc, previousSelectionId, Util.QUANTITY_TYPE)
    if (unitSelection.isEmpty) {
      ui.displayText("Blank, so assuming you want to cancel; if not come back & add again.", waitForKeystrokeIn = false)
      None
    } else {
      outDH.unitId = unitSelection.get._1.getId
      val ans: Option[Float] = Util.askForQuantityAttributeNumber(outDH.number, ui)
      if (ans.isEmpty) None
      else {
        outDH.number = ans.get
        Some(outDH)
      }
    }
  }

  /** Returns None if user wants to cancel. */
  def askForRelToGroupInfo(dhIn: RelationToGroupDataHolder, inEditingUNUSEDForNOW: Boolean = false, uiIn: TextUI): Option[RelationToGroupDataHolder] = {
    val outDH = dhIn

    val groupSelection = chooseOrCreateGroup(Some(List("SELECT GROUP FOR THIS RELATION")), 0)
    val groupId: Option[Long] = {
      if (groupSelection.isEmpty) {
        uiIn.displayText("Blank, so assuming you want to cancel; if not come back & add again.", waitForKeystrokeIn = false)
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
  @tailrec final def chooseOrCreateGroup(leadingTextIn: Option[List[String]], startingDisplayRowIndexIn: Long = 0,
                                         //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) w/in this method!
                                         containingGroupIn: Option[Long] = None /*ie group to omit from pick list*/): Option[IdWrapper] = {
    val totalExisting: Long = db.getGroupCount
    def getNextStartingObjectIndex(currentListLength: Long): Long = {
      val x = startingDisplayRowIndexIn + currentListLength
      if (x >= totalExisting) {
        ui.displayText("End of list found; starting over from the beginning.")
        0 // start over
      } else x
    }
    var leadingText = leadingTextIn.getOrElse(List[String](Util.pickFromListPrompt))
    val choicesPreAdjustment: Array[String] = Array("List next items",
                                                    "Create new group (aka RelationToGroup)",
                                                    "Search for existing group by name...",
                                                    "Search for existing group by id...")
    val numDisplayableItems = ui.maxColumnarChoicesToDisplayAfter(leadingText.size, choicesPreAdjustment.length, Util.maxNameLength)
    val objectsToDisplay = db.getGroups(startingDisplayRowIndexIn, Some(numDisplayableItems), containingGroupIn)
    if (objectsToDisplay.size == 0) {
      val txt: String = TextUI.NEWLN + TextUI.NEWLN + "(None of the needed groups have been created in this model, yet."
      leadingText = leadingText ::: List(txt)
    }
    val choices = Util.addRemainingCountToPrompt(choicesPreAdjustment, objectsToDisplay.size, totalExisting, startingDisplayRowIndexIn)
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
        chooseOrCreateGroup(leadingTextIn, nextStartingIndex, containingGroupIn)
      } else if (answer == 2 && answer <= choices.length) {
        val ans = ui.askForString(Some(Array(Util.relationToGroupNamePrompt)))
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
        val ans = ui.askForString(Some(Array(Util.searchPrompt(Util.GROUP_TYPE))))
        if (ans.isEmpty) None
        else {
          // Allow relation to self, so None in 2nd parm.
          val g: Option[IdWrapper] = findExistingObjectByText(0, Util.GROUP_TYPE, None, ans.get)
          if (g.isEmpty) None
          else Some(new IdWrapper(g.get.getId))
        }
      } else if (answer == 4 && answer <= choices.length) {
        searchById(Util.GROUP_TYPE)
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
          chooseOrCreateGroup(leadingTextIn, startingDisplayRowIndexIn, containingGroupIn)
        } else {
          // user typed a letter to select.. (now 0-based); selected a new object and so we return to the previous menu w/ that one displayed & current
          Some(new IdWrapper(o.getId))
        }
      } else {
        ui.displayText("unknown response in findExistingObjectByText")
        chooseOrCreateGroup(leadingTextIn, startingDisplayRowIndexIn, containingGroupIn)
      }
    }
  }

  /** Returns None if user wants to cancel. */
  def askForRelationEntityIdNumber2(dhIn: RelationToEntityDataHolder, inEditing: Boolean, uiIn: TextUI): Option[RelationToEntityDataHolder] = {
    val previousSelectionDesc = {
      if (!inEditing) None
      else Some(new Entity(db, dhIn.entityId2).getName)
    }
    val previousSelectionId = {
      if (!inEditing) None
      else Some(dhIn.entityId2)
    }
    val selection: Option[(IdWrapper, Boolean, String)] = chooseOrCreateObject(Some(List("SELECT OTHER (RELATED) ENTITY FOR THIS RELATION")),
                                                       previousSelectionDesc, previousSelectionId, Util.ENTITY_TYPE)
    if (selection.isEmpty) None
    else {
      val outDH = dhIn
      val id: Long = selection.get._1.getId
      outDH.entityId2 = id
      outDH.isRemote = selection.get._2
      outDH.remoteInstanceId = selection.get._3
      Some(outDH)
    }
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
                                                      //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s)
                                                      // w/in this method!
                                                      containingEntityIn = Some(userSelection))
    } else {
      new EntityMenu(ui, db, this).entityMenu(userSelection, containingGroupIn = containingGroupIn)
    }
    (subEntitySelected, groupId, moreThanOneAvailable)
  }

  /** see comments for getContentSizePrefix. */
  def getGroupContentSizePrefix(groupId: Long): String = {
    val grpSize = db.getGroupSize(groupId, 1)
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
    // This is to not show that an entity contains more things (">" prefix...) if it only has one group which has no *non-archived* entities:
    val hasOneEmptyGroup: Boolean = {
      val numGroups: Long = db.getRelationToGroupCountByEntity(Some(entityId))
      if (numGroups != 1) false
      else {
        val (_, _, gid: Option[Long], moreAvailable) = db.findRelationToAndGroup_OnEntity(entityId)
        if (gid.isEmpty || moreAvailable) throw new OmException("Found " + (if (gid.isEmpty) 0 else ">1") + " but by the earlier checks, " +
                                                                        "there should be exactly one group in entity " + entityId + " .")
        val groupSize = db.getGroupSize(gid.get, 1)
        groupSize == 0
      }
    }
    val subgroupsCountPrefix: String = {
      if (attrCount == 0 || (attrCount == 1 && hasOneEmptyGroup)) ""
      else if (attrCount == 1) ">"
      else ">>"
    }
    subgroupsCountPrefix
  }

  def addEntityToGroup(groupIn: Group): Option[Long] = {
    val newEntityId: Option[Long] = {
      if (!groupIn.getMixedClassesAllowed) {
        if (groupIn.getSize() == 0) {
          // adding 1st entity to this group, so:
          val leadingText = List("ADD ENTITY TO A GROUP (**whose class will set the group's enforced class, even if 'None'**):")
          val idWrapper: Option[(IdWrapper, _, _)] = chooseOrCreateObject(Some(leadingText), None, None, Util.ENTITY_TYPE,
                                                                  containingGroupIn = Some(groupIn.getId))
          if (idWrapper.isDefined) {
            db.addEntityToGroup(groupIn.getId, idWrapper.get._1.getId)
            Some(idWrapper.get._1.getId)
          } else None
        } else {
          // it's not the 1st entry in the group, so add an entity using the same class as those previously added (or None as case may be).
          val entityClassInUse = groupIn.getClassId
          val idWrapper: Option[(IdWrapper, _, _)] = chooseOrCreateObject(None, None, None, Util.ENTITY_TYPE, 0, entityClassInUse, limitByClassIn = true,
                                                                            containingGroupIn = Some(groupIn.getId))
          if (idWrapper.isEmpty) None
          else {
            val entityId = idWrapper.get._1.getId
            try {
              db.addEntityToGroup(groupIn.getId, entityId)
              Some(entityId)
            } catch {
              case e: Exception =>
                if (e.getMessage.contains(Database.MIXED_CLASSES_EXCEPTION)) {
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
        val idWrapper: Option[(IdWrapper, _, _)] = chooseOrCreateObject(Some(leadingText), None, None, Util.ENTITY_TYPE,
                                                                containingGroupIn = Some(groupIn.getId))
        if (idWrapper.isDefined) {
          db.addEntityToGroup(groupIn.getId, idWrapper.get._1.getId)
          Some(idWrapper.get._1.getId)
        } else None
      }
    }

    newEntityId
  }

  def chooseAmongEntities(containingEntities: util.ArrayList[(Long, Entity)]): Option[Entity] = {
    val leadingText = List[String]("Pick from menu, or an entity by letter")
    val choices: Array[String] = Array(Util.listNextItemsPrompt)
    //(see comments at similar location in EntityMenu, as of this writing on line 288)
    val containingEntitiesNamesWithRelTypes: Array[String] = containingEntities.toArray.map {
                                                                                              case relTypeIdAndEntity: (Long, Entity) =>
                                                                                                val relTypeId: Long = relTypeIdAndEntity._1
                                                                                                val entity: Entity = relTypeIdAndEntity._2
                                                                                                val relTypeName: String = {
                                                                                                  val reltype = new RelationType(db, relTypeId)
                                                                                                  reltype.getArchivedStatusDisplayString + reltype.getName
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

  def removeEntityReferenceFromGroup_Menu(entityIn: Entity, containingGroupIn: Option[Group]): Boolean = {
    val groupCount: Long = db.getCountOfGroupsContainingEntity(entityIn.getId)
    val (entityCountNonArchived, entityCountArchived) = db.getCountOfEntitiesContainingEntity(entityIn.getId)
    val ans = ui.askYesNoQuestion("REMOVE this entity from that group: ARE YOU SURE? (This isn't a deletion: the entity can still be found by searching, and " +
                                  "is " + Util.getContainingEntitiesDescription(entityCountNonArchived, entityCountArchived) +
                                  (if (groupCount > 1) ", and will still be in " + (groupCount - 1) + " group(s).)" else ""),
                                  Some(""))
    if (ans.isDefined && ans.get) {
      containingGroupIn.get.removeEntity(entityIn.getId)
      true

      //is it ever desirable to keep the next line instead of the 'None'? not in most typical usage it seems, but?:
      //entityMenu(startingAttributeIndexIn, entityIn, relationSourceEntityIn, relationIn)
    } else {
      ui.displayText("Did not remove entity from that group.", waitForKeystrokeIn = false)
      false

      //is it ever desirable to keep the next line instead of the 'None'? not in most typical usage it seems, but?:
      //entityMenu(startingAttributeIndexIn, entityIn, relationSourceEntityIn, relationIn, containingGroupIn)
    }
  }

  def getPublicStatusDisplayString(entityIn: Entity): String = {
    //idea: maybe this (logic) knowledge really belongs in the TextUI class. (As some others, probably.)
    if (showPublicPrivateStatusPreference.getOrElse(false)) {
      entityIn.getPublicStatusDisplayStringWithColor(blankIfUnset = false)
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
    if (attrFormIn == Database.getAttributeFormId(Util.QUANTITY_TYPE)) {
      def addQuantityAttribute(dhIn: QuantityAttributeDataHolder): Option[QuantityAttribute] = {
        Some(entityIn.addQuantityAttribute(dhIn.attrTypeId, dhIn.unitId, dhIn.number, None, dhIn.validOnDate, dhIn.observationDate))
      }
      askForInfoAndAddAttribute[QuantityAttributeDataHolder](new QuantityAttributeDataHolder(attrTypeId, None, System.currentTimeMillis(), 0, 0),
                                                             askForAttrTypeId, Util.QUANTITY_TYPE,
                                                             Some(Util.quantityTypePrompt), askForQuantityAttributeNumberAndUnit, addQuantityAttribute)
    } else if (attrFormIn == Database.getAttributeFormId(Util.DATE_TYPE)) {
      def addDateAttribute(dhIn: DateAttributeDataHolder): Option[DateAttribute] = {
        Some(entityIn.addDateAttribute(dhIn.attrTypeId, dhIn.date))
      }
      askForInfoAndAddAttribute[DateAttributeDataHolder](new DateAttributeDataHolder(attrTypeId, 0), askForAttrTypeId, Util.DATE_TYPE,
                                                         Some("SELECT TYPE OF DATE: "), Util.askForDateAttributeValue, addDateAttribute)
    } else if (attrFormIn == Database.getAttributeFormId(Util.BOOLEAN_TYPE)) {
      def addBooleanAttribute(dhIn: BooleanAttributeDataHolder): Option[BooleanAttribute] = {
        Some(entityIn.addBooleanAttribute(dhIn.attrTypeId, dhIn.boolean, None))
      }
      askForInfoAndAddAttribute[BooleanAttributeDataHolder](new BooleanAttributeDataHolder(attrTypeId, None, System.currentTimeMillis(), false),
                                                            askForAttrTypeId,
                                                            Util.BOOLEAN_TYPE, Some("SELECT TYPE OF TRUE/FALSE VALUE: "),  Util.askForBooleanAttributeValue,
                                                            addBooleanAttribute)
    } else if (attrFormIn == Database.getAttributeFormId(Util.FILE_TYPE)) {
      def addFileAttribute(dhIn: FileAttributeDataHolder): Option[FileAttribute] = {
        Some(entityIn.addFileAttribute(dhIn.attrTypeId, dhIn.description, new File(dhIn.originalFilePath)))
      }
      val result: Option[FileAttribute] = askForInfoAndAddAttribute[FileAttributeDataHolder](new FileAttributeDataHolder(attrTypeId, "", ""),
                                                                                             askForAttrTypeId, Util.FILE_TYPE,
                                                                                             Some("SELECT TYPE OF FILE: "), Util.askForFileAttributeInfo,
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
    } else if (attrFormIn == Database.getAttributeFormId(Util.TEXT_TYPE)) {
      def addTextAttribute(dhIn: TextAttributeDataHolder): Option[TextAttribute] = {
        Some(entityIn.addTextAttribute(dhIn.attrTypeId, dhIn.text, None, dhIn.validOnDate, dhIn.observationDate))
      }
      askForInfoAndAddAttribute[TextAttributeDataHolder](new TextAttributeDataHolder(attrTypeId, None, System.currentTimeMillis(), ""),
                                                         askForAttrTypeId, Util.TEXT_TYPE,
                                                         Some("SELECT TYPE OF " + Util.textDescription + ": "), Util.askForTextAttributeText, addTextAttribute)
    } else if (attrFormIn == Database.getAttributeFormId(Util.RELATION_TO_ENTITY_TYPE)) {
      // Not trapping here for RELATION_TO_REMOTE_ENTITY_TYPE because this one covers both.  The menu before this lets you pick RTE, and
      // the one it calls lets you specify if it should be a remote kind.  The distinction is more in the database than in the UI.
      def addRelationToEntity(dhIn: RelationToEntityDataHolder): Option[AttributeWithValidAndObservedDates] = {
        if (dhIn.isRemote) {
          val relation = entityIn.addRelationToEntity(dhIn.attrTypeId, dhIn.entityId2, None, dhIn.validOnDate, dhIn.observationDate,
                                                      remoteIn = true, Some(dhIn.remoteInstanceId))
          Some(relation)
        } else {
          val relation = entityIn.addRelationToEntity(dhIn.attrTypeId, dhIn.entityId2, None, dhIn.validOnDate, dhIn.observationDate)
          Some(relation)
        }
      }
      askForInfoAndAddAttribute[RelationToEntityDataHolder](new RelationToEntityDataHolder(attrTypeId, None, System.currentTimeMillis(), 0, false, ""),
                                                            askForAttrTypeId, Util.RELATION_TYPE_TYPE,
                                                            Some("CREATE OR SELECT RELATION TYPE: (" + Util.mRelTypeExamples + ")"),
                                                            askForRelationEntityIdNumber2, addRelationToEntity)
    } else if (attrFormIn == 100) {
      // re "100": see comments at attrFormIn above
      val eId: Option[IdWrapper] = askForNameAndSearchForEntity
      if (eId.isDefined) {
        Some(entityIn.addHASRelationToEntity(eId.get.getId, None, System.currentTimeMillis))
      } else {
        None
      }
    } else if (attrFormIn == Database.getAttributeFormId(Util.RELATION_TO_GROUP_TYPE)) {
      def addRelationToGroup(dhIn: RelationToGroupDataHolder): Option[RelationToGroup] = {
        require(dhIn.entityId == entityIn.getId)
        val newRTG: RelationToGroup = entityIn.addRelationToGroup(dhIn.attrTypeId, dhIn.groupId, None, dhIn.validOnDate, dhIn.observationDate)
        Some(newRTG)
      }
      val result: Option[Attribute] = askForInfoAndAddAttribute[RelationToGroupDataHolder](new RelationToGroupDataHolder(entityIn.getId, attrTypeId, 0,
                                                                                                                         None, System.currentTimeMillis()),
                                                                                           askForAttrTypeId, Util.RELATION_TYPE_TYPE,
                                                                                           Some("CREATE OR SELECT RELATION TYPE: (" +
                                                                                                Util.mRelTypeExamples + ")" +
                                                                                                "." + TextUI.NEWLN + "(Does anyone see a specific " +
                                                                                                "reason to keep asking for these dates?)"),
                                                                                           askForRelToGroupInfo, addRelationToGroup)
      if (result.isEmpty) {
        None
      } else {
        val newRtg = result.get.asInstanceOf[RelationToGroup]
        new QuickGroupMenu(ui, db, this).quickGroupMenu(new Group(db, newRtg.getGroupId), 0, Some(newRtg), None, containingEntityIn = Some(entityIn))
        // user could have deleted the new result: check that before returning it as something to act upon:
        if (db.relationToGroupKeyExists(newRtg.getId)) {
          result
        } else {
          None
        }
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
        Util.getClipboardContent
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
          Some(Util.getClipboardContent)
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
      // user could have deleted the new result: check that before returning it as something to act upon:
      if (db.relationToEntityKeyExists(newRTE.getId) && db.entityKeyExists(newEntity.getId)) {
        Some(newRTE)
      } else {
        None
      }
    } else {
      ui.displayText("invalid response")
      None
    }
  }

  def defaultAttributeCopying(entityIn: Entity, attributeTuplesIn: Option[Array[(Long, Attribute)]] = None): Unit = {
    if (shouldTryAddingDefaultAttributes(entityIn)) {
      val attributeTuples: Array[(Long, Attribute)] = {
        if (attributeTuplesIn.isDefined) attributeTuplesIn.get
        else db.getSortedAttributes(entityIn.getId, onlyPublicEntitiesIn = false)._1
      }
      val templateAttributesToCopy: ArrayBuffer[Attribute] = getMissingAttributes(entityIn.getClassTemplateEntityId, attributeTuples)
      copyAndEditAttributes(entityIn, templateAttributesToCopy)
    }
  }

  def copyAndEditAttributes(entityIn: Entity, templateAttributesToCopyIn: ArrayBuffer[Attribute]): Unit = {
    // userWantsOut is used like a break statement below: could be replaced with a functional idiom (see link to stackoverflow somewhere in the code).
    var escCounter = 0
    var userWantsOut = false

    def checkIfExiting(escCounterIn: Int, attributeCounterIn: Int, numAttributes: Int): Int = {
      var escCounterLocal = escCounterIn + 1
      if (escCounterLocal > 3 && attributeCounterIn < numAttributes /* <, so we don't ask when done anyway. */) {
        val outAnswer = ui.askYesNoQuestion("Stop checking/adding attributes?", Some(""))
        require(outAnswer.isDefined, "Unexpected behavior: meant to make user answer here.")
        if (outAnswer.get) {
          userWantsOut = true
        } else {
          escCounterLocal = 0
        }
      }
      escCounterLocal
    }

    var (allCopy: Boolean, allCreateOrSearch: Boolean, allKeepReference: Boolean) = (false, false, false)
    val choice1text = "Copy the template entity, editing its name (**MOST LIKELY CHOICE)"
    val copyFromTemplateAndEditNameChoiceNum = 1
    val choice2text = "Create a new entity or search for an existing one for this purpose"
    val createOrSearchForEntityChoiceNum = 2
    val choice3text = "Keep a reference to the same entity as in the template (least likely choice)"
    val keepSameReferenceAsInTemplateChoiceNum = 3
    var askEveryTime: Option[Boolean] = None
    var attrCounter = 0
    for (templateAttribute: Attribute <- templateAttributesToCopyIn) {
      attrCounter += 1
      if (!userWantsOut) {
        val waitForKeystroke = {
          templateAttribute match {
            case a: RelationToEntity => true
            case _ => false
          }
        }
        def promptToEditAttributeCopy() {
          ui.displayText("Edit the copied " + Database.getAttributeFormName(templateAttribute.getFormId) + " \"" +
                         templateAttribute.getDisplayString(0, None, None, simplify = true) + "\", from the template entity (ESC to abort):",
                         waitForKeystroke)
        }
        val newAttribute: Option[Attribute] = {
          templateAttribute match {
            case a: QuantityAttribute =>
              promptToEditAttributeCopy()
              Some(entityIn.addQuantityAttribute(a.getAttrTypeId, a.getUnitId, a.getNumber, Some(a.getSortingIndex)))
            case a: DateAttribute =>
              promptToEditAttributeCopy()
              Some(entityIn.addDateAttribute(a.getAttrTypeId, a.getDate, Some(a.getSortingIndex)))
            case a: BooleanAttribute =>
              promptToEditAttributeCopy()
              Some(entityIn.addBooleanAttribute(a.getAttrTypeId, a.getBoolean, Some(a.getSortingIndex)))
            case a: FileAttribute =>
              ui.displayText("You can add a FileAttribute manually afterwards for this attribute.  Maybe it can be automated " +
                             "more, when use cases for this part are more clear.")
              None
            case a: TextAttribute =>
              promptToEditAttributeCopy()
              Some(entityIn.addTextAttribute(a.getAttrTypeId, a.getText, Some(a.getSortingIndex)))
            case a: RelationToEntity =>
              askEveryTime = {
                if (askEveryTime.isDefined) {
                  askEveryTime
                } else {
                  val howRTEsLeadingText: Array[String] = Array("The template has relations to entities.  How would you like the equivalent to be provided" +
                                                                " for this new entity being created?")
                  val howHandleRTEsChoices = Array[String]("For ALL entity relations being added: " + choice1text,
                                                           "For ALL entity relations being added: " + choice2text,
                                                           "For ALL entity relations being added: " + choice3text,
                                                           "Ask for each relation to entity being created from the template")
                  val howHandleRTEsResponse = ui.askWhich(Some(howRTEsLeadingText), howHandleRTEsChoices)
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
                      ui.displayText("Unexpected answer: " + howHandleRTEsResponse.get)
                      None
                    }
                  } else {
                    None
                  }
                }
              }
              if (askEveryTime.isEmpty) {
                None
              } else {
                var whichRTEResponse: Option[Int] = None
                if (askEveryTime.get) {
                  val whichRteLeadingText: Array[String] = Array("The template has a relation to an entity named \"" +
                                                                 templateAttribute.getDisplayString(0, None, None, simplify = true) +
                                                                 "\": how would you like the equivalent to be provided for this new entity being created?" +
                                                                 " (0/ESC to just skip this one for now)")
                  val whichRTEChoices = Array[String](choice1text, choice2text, choice3text)
                  whichRTEResponse = ui.askWhich(Some(whichRteLeadingText), whichRTEChoices)
                }
                if (askEveryTime.get && whichRTEResponse.isEmpty) {
                  None
                } else {
                  if (allCopy || (whichRTEResponse.isDefined && whichRTEResponse.get == copyFromTemplateAndEditNameChoiceNum)) {
                    val templatesRelatedEntity: Entity = new Entity(db, a.getRelatedId2)
                    val oldName: String = templatesRelatedEntity.getName
                    val newEntity = askForNameAndWriteEntity(Util.ENTITY_TYPE, None, Some(oldName), None, None, templatesRelatedEntity.getClassId,
                                                             Some("EDIT THE ENTITY NAME:"), duplicateNameProbablyOK = true)
                    if (newEntity.isEmpty) {
                      None
                    } else {
                      newEntity.get.updateNewEntriesStickToTop(templatesRelatedEntity.getNewEntriesStickToTop)
                      Some(entityIn.addRelationToEntity(a.getAttrTypeId, newEntity.get.getId, Some(a.getSortingIndex)))
                    }
                  } else if (allCreateOrSearch || (whichRTEResponse.isDefined && whichRTEResponse.get == createOrSearchForEntityChoiceNum)) {
                    val dh: Option[RelationToEntityDataHolder] = askForRelationEntityIdNumber2(new RelationToEntityDataHolder(a.getAttrTypeId, None,
                                                                                                                              System.currentTimeMillis(), 0,
                                                                                                                              false, ""),
                                                                                               inEditing = false, ui)
                    if (dh.isDefined) {
                      if (dh.get.isRemote) {
                        val relation = entityIn.addRelationToEntity(dh.get.attrTypeId, dh.get.entityId2, None, dh.get.validOnDate, dh.get.observationDate,
                                                                    dh.get.isRemote, Some(dh.get.remoteInstanceId))
                        Some(relation.asInstanceOf[RelationToRemoteEntity])
                      } else {
                        val relation: RelationToEntity = entityIn.addRelationToEntity(a.getAttrTypeId, dh.get.entityId2, Some(a.getSortingIndex))
                        Some(relation)
                      }
                    } else {
                      None
                    }
                  } else if (allKeepReference || (whichRTEResponse.isDefined && whichRTEResponse.get == keepSameReferenceAsInTemplateChoiceNum)) {
                    val relation = entityIn.addRelationToEntity(a.getAttrTypeId, a.getRelatedId2, Some(a.getSortingIndex), None, System.currentTimeMillis(),
                                                                a.isRemote,
                                                                if (a.isRemote) Some(a.asInstanceOf[RelationToRemoteEntity].getRemoteInstanceIdIn) else None)
                    Some(relation)
                  } else {
                    ui.displayText("Unexpected answer: " + whichRTEResponse.get)
                    None
                  }
                }
              }
            case a: RelationToGroup =>
              promptToEditAttributeCopy()
              val templateGroup = a.getGroup
              val (_, newRTG: RelationToGroup) = entityIn.addGroupAndRelationToGroup(a.getAttrTypeId, templateGroup.getName,
                                                                                     templateGroup.getMixedClassesAllowed, None,
                                                                                     System.currentTimeMillis(), Some(a.getSortingIndex))
              Some(newRTG)
            case _ => throw new OmException("Unexpected type: " + templateAttribute.getClass.getCanonicalName)
          }
        }
        if (newAttribute.isEmpty) {
          escCounter = checkIfExiting(escCounter, attrCounter, templateAttributesToCopyIn.size)
        } else {
          // (Not re-editing if it is a RTE  because it was edited just above as part of the initial attribute creation step.)
          if (!newAttribute.get.isInstanceOf[RelationToEntity]) {
            val exitedOneEditLine: Boolean = editAttributeOnSingleLine(newAttribute.get)
            if (exitedOneEditLine) {
              // That includes a "never mind" intention on the last one added (just above), so:
              newAttribute.get.delete()
              escCounter = checkIfExiting(escCounter, attrCounter, templateAttributesToCopyIn.size)
            }
          }
        }
      }
    }
  }

  def getMissingAttributes(classTemplateEntityIdIn: Option[Long], attributeTuplesIn: Array[(Long, Attribute)]): ArrayBuffer[Attribute] = {
    val templateAttributesToSuggestCopying: ArrayBuffer[Attribute] = {
      // This determines which attributes from the template entity (or "pattern" or "class-defining entity") are not found on this entity, so they can
      // be added if the user wishes.
      val attributesToSuggestCopying_workingCopy: ArrayBuffer[Attribute] = new ArrayBuffer()
      if (classTemplateEntityIdIn.isDefined) {
        // ("cde" in name means "classDefiningEntity" (aka template))
        val (cde_attributeTuples: Array[(Long, Attribute)], _) = db.getSortedAttributes(classTemplateEntityIdIn.get, onlyPublicEntitiesIn = false)
        for (cde_attributeTuple <- cde_attributeTuples) {
          var attributeTypeFoundOnEntity = false
          val cde_attribute = cde_attributeTuple._2
          for (attributeTuple <- attributeTuplesIn) {
            if (!attributeTypeFoundOnEntity) {
              val cde_typeId: Long = cde_attribute.getAttrTypeId
              val typeId = attributeTuple._2.getAttrTypeId
              // This is a very imperfect check.  Perhaps this is a motive to use more descriptive relation types in template entities.
              val existingAttributeStringContainsTemplateString: Boolean = {
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

  def shouldTryAddingDefaultAttributes(entityIn: Entity): Boolean = {
    if (entityIn.getClassId.isEmpty) {
      false
    } else {
      val createAttributes: Option[Boolean] = db.getClassCreateDefaultAttributes(entityIn.getClassId.get)
      if (createAttributes.isDefined) {
        createAttributes.get
      } else {
        if (entityIn.getClassTemplateEntityId.isEmpty) {
          false
        } else {
          val attrCount = new Entity(db, entityIn.getClassTemplateEntityId.get).getAttrCount
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
