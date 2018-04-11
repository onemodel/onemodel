/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003-2004 and 2008-2017 inclusive, Luke A. Call; all rights reserved.
    (That copyright statement once said 2013-2015, until I remembered that much of Controller came from TextUI.scala, and TextUI.java before that.)
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule, guidelines around binary
    distribution, and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
package org.onemodel.core.controllers

import java.io._
import java.util

import org.onemodel.core._
import org.onemodel.core.model._

import scala.annotation.tailrec
import scala.collection.JavaConversions._
import scala.collection.mutable.ArrayBuffer

/** This Controller is for user-interactive things.  The Controller class in the web module is for the REST API.  For shared code that does not fit
  * in those, see the org.onemodel.core.Util object (in Util.scala).
  *
  * Improvements to this class should START WITH MAKING IT BETTER TESTED (functional testing? integration? see
  * scalatest docs 4 ideas, & maybe use expect or the gnu testing tool that uses expect?), delaying side effects more,
  * shorter methods, other better scala style, etc.
  *
  *
  * * * * *IMPORTANT * * * * * IMPORTANT* * * * * * *IMPORTANT * * * * * * * IMPORTANT* * * * * * * * *IMPORTANT * * * * * *
  Don't ever instantiate a controller from a *test* without passing in username/password parameters, because it will try to log in to the user's default
  database and run the tests there (ie, they could be destructive):
  * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * * *
  */
class Controller(ui: TextUI, forceUserPassPromptIn: Boolean = false, defaultUsernameIn: Option[String] = None, defaultPasswordIn: Option[String] = None) {
  //idea: get more scala familiarity then change this so it has limited visibility/scope: like, protected (subclass instances) + ImportExportTest.
  // This should *not* be passed around as a parameter to everything, but rather those places in the code should get the DB instance from the
  // entity (or other model object) being processed.
  private val localDb: Database = tryLogins(forceUserPassPromptIn, defaultUsernameIn, defaultPasswordIn)
  val moveFartherCount = 25
  val moveFarthestCount = 50

  /** Returns the id and the entity, if they are available from the preferences lookup (id) and then finding that in the db (Entity). */
  def getDefaultEntity: Option[(Long, Entity)] = {
    if (defaultDisplayEntityId.isEmpty || ! localDb.entityKeyExists(defaultDisplayEntityId.get)) {
      None
    } else {
      val entity: Option[Entity] = Entity.getEntity(localDb, defaultDisplayEntityId.get)
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
            localDb.setIncludeArchivedEntities(true)
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
      new MainMenu(ui, localDb, this).mainMenu(if (getDefaultEntity.isEmpty) None else Some(getDefaultEntity.get._2),
                                          goDirectlyToChoice)
      menuLoop()
    }
    menuLoop(Some(5))
  }

  /** If the 1st parm is true, the next 2 must be omitted or None. */
  private def tryLogins(forceUserPassPromptIn: Boolean = false, defaultUsernameIn: Option[String] = None,
                        defaultPasswordIn: Option[String] = None): Database = {

    require(if (forceUserPassPromptIn) defaultUsernameIn.isEmpty && defaultPasswordIn.isEmpty else true)

    // Tries the system username, blank password, & if that doesn't work, prompts user.
    //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) within this method, below!
    @tailrec def tryOtherLoginsOrPrompt(): Database = {
      val db = {
        var pwdOpt: Option[String] = None
        // try logging in with some obtainable default values first, to save user the trouble, like if pwd is blank
        val (defaultUserName, defaultPassword) = Util.getDefaultUserInfo
        val dbWithSystemNameBlankPwd = Database.login(defaultUserName, defaultPassword, showError = false)
        if (dbWithSystemNameBlankPwd.isDefined) {
          ui.displayText("(Using default user info...)", waitForKeystrokeIn = false)
          dbWithSystemNameBlankPwd
        } else {
          val usrOpt = ui.askForString(Some(Array("Username")), None, Some(defaultUserName))
          if (usrOpt.isEmpty) System.exit(1)
          val dbConnectedWithBlankPwd = Database.login(usrOpt.get, defaultPassword, showError = false)
          if (dbConnectedWithBlankPwd.isDefined) dbConnectedWithBlankPwd
          else {
            try {
              pwdOpt = ui.askForString(Some(Array("Password")), None, None, isPasswordIn = true)
              if (pwdOpt.isEmpty) System.exit(1)
              val dbWithUserEnteredPwd = Database.login(usrOpt.get, pwdOpt.get, showError = true)
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
      @tailrec def loopPrompting: Database = {
        val usrOpt = ui.askForString(Some(Array("Username")))
        if (usrOpt.isEmpty) System.exit(1)

        val pwdOpt = ui.askForString(Some(Array("Password")), None, None, isPasswordIn = true)
        if (pwdOpt.isEmpty) System.exit(1)

        val dbWithUserEnteredPwd: Option[Database] = Database.login(usrOpt.get, pwdOpt.get, showError = false)
        if (dbWithUserEnteredPwd.isDefined) dbWithUserEnteredPwd.get
        else loopPrompting
      }
      loopPrompting
    } else if (defaultUsernameIn.isDefined && defaultPasswordIn.isDefined) {
      // idea: perhaps this could be enhanced and tested to allow a username parameter, but prompt for a password, if/when need exists.
      val db = Database.login(defaultUsernameIn.get, defaultPasswordIn.get, showError = true)
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

  // Idea: From showPublicPrivateStatusPreference, on down through findDefaultDisplayEntityId, feels awkward.  Needs something better, but I'm not sure
  // what, at the moment.  It was created this way as a sort of cache because looking it up every time was costly and made the app slow, like when
  // displaying a list of entities (getting the preference every time, to N levels deep), and especially at startup when checking for the default
  // up to N levels deep, among the preferences that can include entities with deep nesting.  So in a related change I made it also not look N levels
  // deep, for preferences.  If you check other places touched by this commit there may be a "shotgun surgery" bad smell here also.
  //Idea: Maybe these should have their cache expire after a period of time (to help when running multiple clients).
  var showPublicPrivateStatusPreference: Option[Boolean] = localDb.getUserPreference_Boolean(Util.SHOW_PUBLIC_PRIVATE_STATUS_PREFERENCE)
  def refreshPublicPrivateStatusPreference(): Unit = {
    showPublicPrivateStatusPreference = localDb.getUserPreference_Boolean(Util.SHOW_PUBLIC_PRIVATE_STATUS_PREFERENCE)
  }
  // putting this in a var instead of recalculating it every time (too frequent) inside findDefaultDisplayEntityId:
  var defaultDisplayEntityId: Option[Long] = localDb.getUserPreference_EntityId(Util.DEFAULT_ENTITY_PREFERENCE)
  def refreshDefaultDisplayEntityId(): Unit = {
    defaultDisplayEntityId = localDb.getUserPreference_EntityId(Util.DEFAULT_ENTITY_PREFERENCE)
  }

  def askForClass(dbIn: Database): Option[Long] = {
    val msg = "CHOOSE ENTITY'S CLASS.  (Press ESC if you don't know or care about this.  Detailed explanation on the class feature will be available " +
              "at onemodel.org when this feature is documented more (hopefully at the next release), or ask on the email list.)"
    val result: Option[(IdWrapper, Boolean, String)] = chooseOrCreateObject(dbIn, Some(List[String](msg)), None, None, Util.ENTITY_CLASS_TYPE)
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
  def askForClassInfoAndNameAndCreateEntity(dbIn: Database, classIdIn: Option[Long] = None): Option[Entity] = {
    var newClass = false
    val classId: Option[Long] =
      if (classIdIn.isDefined) classIdIn
      else {
        newClass = true
        askForClass(dbIn)
      }
    val ans: Option[Entity] = askForNameAndWriteEntity(dbIn, Util.ENTITY_TYPE, None, None, None, None, classId,
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

  /**
   * SEE DESCRIPTIVE COMMENT ON askForAndWriteClassAndTemplateEntityName, WHICH APPLIES TO all such METHODS (see this cmt elsewhere).
    *
    * The "previous..." parameters are for the already-existing data (ie, when editing not creating).
    */
  def askForNameAndWriteEntity(dbIn: Database, typeIn: String, existingEntityIn: Option[Entity] = None, previousNameIn: Option[String] = None,
                               previousDirectionalityIn: Option[String] = None,
                               previousNameInReverseIn: Option[String] = None, classIdIn: Option[Long] = None,
                               leadingTextIn: Option[String] = None, duplicateNameProbablyOK: Boolean = false): Option[Entity] = {
    if (classIdIn.isDefined) require(typeIn == Util.ENTITY_TYPE)
    val createNotUpdate: Boolean = existingEntityIn.isEmpty
    if (!createNotUpdate && typeIn == Util.RELATION_TYPE_TYPE) require(previousDirectionalityIn.isDefined)
    val maxNameLength = {
      if (typeIn == Util.RELATION_TYPE_TYPE) model.RelationType.getNameLength
      else if (typeIn == Util.ENTITY_TYPE) model.Entity.nameLength
      else throw new scala.Exception("invalid inType: " + typeIn)
    }
    val example = {
      if (typeIn == Util.RELATION_TYPE_TYPE) " (use 3rd-person verb like \"owns\"--might make output like sentences more consistent later on)"
      else ""
    }

    /** 2nd Long in return value is ignored in this particular case.
      */
    def askAndSave(dbIn: Database, defaultNameIn: Option[String] = None): Option[(Long, Long)] = {
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
            askAndSave(dbIn, Some(name))
          } else {
            val selfIdToIgnore: Option[Long] = if (existingEntityIn.isDefined) Some(existingEntityIn.get.getId) else None
            if (Util.isDuplicationAProblem(model.Entity.isDuplicate(dbIn, name, selfIdToIgnore), duplicateNameProbablyOK, ui)) None
            else {
              if (typeIn == Util.ENTITY_TYPE) {
                if (createNotUpdate) {
                  val newId = model.Entity.createEntity(dbIn, name, classIdIn).getId
                  Some(newId, 0L)
                } else {
                  existingEntityIn.get.updateName(name)
                  Some(existingEntityIn.get.getId, 0L)
                }
              } else if (typeIn == Util.RELATION_TYPE_TYPE) {
                val ans: Option[String] = Util.askForRelationDirectionality(previousDirectionalityIn, ui)
                if (ans.isEmpty) None
                else {
                  val directionalityStr: String = ans.get.trim().toUpperCase
                  val nameInReverseDirectionStr = Util.askForNameInReverseDirection(directionalityStr, maxNameLength, name, previousNameInReverseIn, ui)
                  if (createNotUpdate) {
                    val newId = new RelationType(dbIn, dbIn.createRelationType(name, nameInReverseDirectionStr, directionalityStr)).getId
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

    val result = tryAskingAndSaving[(Long, Long)](dbIn, Util.stringTooLongErrorMessage(maxNameLength), askAndSave, previousNameIn)
    if (result.isEmpty) None
    else Some(new Entity(dbIn, result.get._1))
  }

  /** Call a provided function (method?) "askAndSaveIn", which does some work that might throw a specific OmDatabaseException.  If it does throw that,
    * let the user know the problem and call askAndSaveIn again.  I.e., allow retrying if the entered data is bad, instead of crashing the app.
    */
  def tryAskingAndSaving[T](dbIn: Database,
                            errorMsgIn: String,
                            askAndSaveIn: (Database, Option[String]) => Option[T],
                            defaultNameIn: Option[String] = None): Option[T] = {
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
        val cumulativeMsg = accumulateMsgs(e.toString, e.getCause)
        if (cumulativeMsg.contains(Util.tooLongMessage)) {
          ui.displayText(errorMsgIn.format(Util.tooLongMessage) + cumulativeMsg + ".")
          tryAskingAndSaving[T](dbIn, errorMsgIn, askAndSaveIn, defaultNameIn)
        } else throw e
    }
  }

  /**
    * @param classIn (1st parameter) should be None only if the call is intended to create; otherwise it is an edit.
    * @return None if user wants out, otherwise returns the new or updated classId and entityId.
    * */
  def askForAndWriteClassAndTemplateEntityName(dbIn: Database, classIn: Option[EntityClass] = None): Option[(Long, Long)] = {
    if (classIn.isDefined) {
      // dbIn is required even if classIn is not provided, but if classIn is provided, make sure things are in order:
      // (Idea:  check: does scala do a deep equals so it is valid?  also tracked in tasks.)
      require(classIn.get.mDB == dbIn)
    }
    val createNotUpdate: Boolean = classIn.isEmpty
    val nameLength = model.EntityClass.nameLength(dbIn)
    val oldTemplateNamePrompt = {
      if (createNotUpdate) ""
      else {
        val entityId = classIn.get.getTemplateEntityId
        val templateEntityName = new Entity(dbIn, entityId).getName
        " (which is currently \"" + templateEntityName + "\")"
      }
    }
    def askAndSave(dbIn: Database, defaultNameIn: Option[String]): Option[(Long, Long)] = {
      val nameOpt = ui.askForString(Some(Array("Enter class name (up to " + nameLength + " characters; will also be used for its template entity name" +
                                               oldTemplateNamePrompt + "; ESC to cancel): ")),
                                    None, defaultNameIn)
      if (nameOpt.isEmpty) None
      else {
        val name = nameOpt.get.trim()
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
              val entityId: Long = classIn.get.updateClassAndTemplateEntityName(name)
              Some(classIn.get.getId, entityId)
            }
          }
        }
      }
    }

    tryAskingAndSaving[(Long, Long)](dbIn, Util.stringTooLongErrorMessage(nameLength), askAndSave, if (classIn.isEmpty) None else Some(classIn.get.getName))
  }

  /** SEE DESCRIPTIVE COMMENT ON askForAndWriteClassAndTemplateEntityName, WHICH APPLIES TO all such METHODS (see this cmt elsewhere).
    * @return The instance's id, or None if there was a problem or the user wants out.
    * */
  def askForAndWriteOmInstanceInfo(dbIn: Database, oldOmInstanceIn: Option[OmInstance] = None): Option[String] = {
    val createNotUpdate: Boolean = oldOmInstanceIn.isEmpty
    val addressLength = model.OmInstance.addressLength
    def askAndSave(dbIn: Database, defaultNameIn: Option[String]): Option[String] = {
      val addressOpt = ui.askForString(Some(Array("Enter the internet address with optional port of a remote OneModel instance (for " +
                                                  "example, \"om.example.com:9000\", up to " + addressLength + " characters; ESC to cancel;" +
                                                  " Other examples include (omit commas):  localhost,  127.0.0.1:2345,  ::1 (?)," +
                                                  "  my.example.com:80,  your.example.com:8080  .): ")), None, defaultNameIn)
      if (addressOpt.isEmpty) None
      else {
        val address = addressOpt.get.trim()
        if (address.length() == 0) None
        else {
          if (Util.isDuplicationAProblem(OmInstance.isDuplicate(dbIn, address, if (oldOmInstanceIn.isEmpty) None else Some(oldOmInstanceIn.get.getId)),
                                         duplicateNameProbablyOK = false, ui)) {
            None
          } else {
            val restDb = Database.getRestDatabase(address)
            val remoteId: Option[String] = restDb.getIdWithOptionalErrHandling(Some(ui))
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
                  val ans: Option[Boolean] = ui.askYesNoQuestion("The IDs of the old and new remote instances don't match (old " +
                                                                 "id/address: " + oldOmInstanceIn.get.getId + "/" +
                                                                 oldOmInstanceIn.get.getAddress + ", new id/address: " +
                                                                 remoteId.get + "/" + address + ".  Instead of updating the old one, you should create a new" +
                                                                 " entry for the new remote instance and then optionally delete this old one." +
                                                                 "  Do you want to create the new entry with this new address, now?")
                  if (ans.isDefined && ans.get) {
                    val id: String = OmInstance.create(dbIn, remoteId.get, address).getId
                    ui.displayText("Created the new entry for \"" + address + "\".  You still have to delete the old one (" + oldOmInstanceIn.get.getId + "/" +
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
  def askForInfoAndUpdateAttribute[T <: AttributeDataHolder](dbIn: Database, dhIn: T, askForAttrTypeId: Boolean, attrType: String,
                                                             promptForSelectingTypeId: String,
                                                             getOtherInfoFromUser: (Database, T, Boolean, TextUI) => Option[T],
                                                             updateTypedAttribute: (T) => Unit): Boolean = {
    //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) within this method, below!
    @tailrec def askForInfoAndUpdateAttribute_helper(dhIn: T, attrType: String, promptForTypeId: String): Boolean = {
      val ans: Option[T] = askForAttributeData[T](dbIn, dhIn, askForAttrTypeId, attrType, Some(promptForTypeId),
                                                  Some(new Entity(dbIn, dhIn.attrTypeId).getName),
                                                  Some(dhIn.attrTypeId), getOtherInfoFromUser, editingIn = true)
      if (ans.isEmpty) {
        false
      } else {
        val dhOut: T = ans.get
        val ans2: Option[Int] = Util.promptWhetherTo1Add2Correct(attrType, ui)

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
    val leadingText: Array[String] = Array("Attribute: " + attributeIn.getDisplayString(0, None, None))
    var firstChoices = Array("Edit the attribute type, " +
                             (if (Util.canEditAttributeOnSingleLine(attributeIn)) "content (single line)," else "") +
                             " and valid/observed dates",

                             if (attributeIn.isInstanceOf[TextAttribute]) "Edit (as multiline value)" else "(stub)",
                             if (Util.canEditAttributeOnSingleLine(attributeIn)) "Edit the attribute content (single line)" else "(stub)",
                             "Delete",
                             "Go to entity representing the type: " + new Entity(attributeIn.mDB, attributeIn.getAttrTypeId).getName)
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
            val textAttributeDH: TextAttributeDataHolder = new TextAttributeDataHolder(textAttribute.getAttrTypeId, textAttribute.getValidOnDate,
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
            val dateAttributeDH: DateAttributeDataHolder = new DateAttributeDataHolder(dateAttribute.getAttrTypeId, dateAttribute.getDate)
            askForInfoAndUpdateAttribute[DateAttributeDataHolder](attributeIn.mDB, dateAttributeDH, askForAttrTypeId = true, Util.DATE_TYPE, "CHOOSE TYPE OF DATE:",
                                                                  Util.askForDateAttributeValue, updateDateAttribute)
            //force a reread from the DB so it shows the right info on the repeated menu:
            attributeEditMenu(new DateAttribute(attributeIn.mDB, attributeIn.getId))
          case booleanAttribute: BooleanAttribute =>
            def updateBooleanAttribute(dhInOut: BooleanAttributeDataHolder) {
              booleanAttribute.update(dhInOut.attrTypeId, dhInOut.boolean, dhInOut.validOnDate, dhInOut.observationDate)
            }
            val booleanAttributeDH: BooleanAttributeDataHolder = new BooleanAttributeDataHolder(booleanAttribute.getAttrTypeId, booleanAttribute.getValidOnDate,
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
            val fileAttributeDH: FileAttributeDataHolder = new FileAttributeDataHolder(fa.getAttrTypeId, fa.getDescription, fa.getOriginalFilePath)
            askForInfoAndUpdateAttribute[FileAttributeDataHolder](attributeIn.mDB, fileAttributeDH, askForAttrTypeId = true, Util.FILE_TYPE, "CHOOSE TYPE OF FILE:",
                                                                  Util.askForFileAttributeInfo, updateFileAttribute)
            //force a reread from the DB so it shows the right info on the repeated menu:
            attributeEditMenu(new FileAttribute(attributeIn.mDB, attributeIn.getId))
          case _ => throw new Exception("Unexpected type: " + attributeIn.getClass.getName)
        }
      } else if (answer == 2 && attributeIn.isInstanceOf[TextAttribute]) {
        val ta = attributeIn.asInstanceOf[TextAttribute]
        val newContent: String = Util.editMultilineText(ta.getText, ui)
        ta.update(ta.getAttrTypeId, newContent, ta.getValidOnDate, ta.getObservationDate)
        //then force a reread from the DB so it shows the right info on the repeated menu:
        attributeEditMenu(new TextAttribute(attributeIn.mDB, attributeIn.getId))
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
        new EntityMenu(ui, this).entityMenu(new Entity(attributeIn.mDB, attributeIn.getAttrTypeId))
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
        val outDH: Option[TextAttributeDataHolder] = Util.askForTextAttributeText(attributeIn.mDB, textAttributeDH, inEditing = true, ui)
        if (outDH.isDefined) textAttribute.update(outDH.get.attrTypeId, outDH.get.text, outDH.get.validOnDate, outDH.get.observationDate)
        outDH.isEmpty
      case dateAttribute: DateAttribute =>
        val dateAttributeDH: DateAttributeDataHolder = new DateAttributeDataHolder(dateAttribute.getAttrTypeId, dateAttribute.getDate)
        val outDH: Option[DateAttributeDataHolder] = Util.askForDateAttributeValue(attributeIn.mDB, dateAttributeDH, inEditing = true, ui)
        if (outDH.isDefined) dateAttribute.update(outDH.get.attrTypeId, outDH.get.date)
        outDH.isEmpty
      case booleanAttribute: BooleanAttribute =>
        val booleanAttributeDH: BooleanAttributeDataHolder = new BooleanAttributeDataHolder(booleanAttribute.getAttrTypeId, booleanAttribute.getValidOnDate,
                                                                                            booleanAttribute.getObservationDate,
                                                                                            booleanAttribute.getBoolean)
        val outDH: Option[BooleanAttributeDataHolder] = Util.askForBooleanAttributeValue(booleanAttribute.mDB, booleanAttributeDH, inEditing = true, ui)
        if (outDH.isDefined) booleanAttribute.update(outDH.get.attrTypeId, outDH.get.boolean, outDH.get.validOnDate, outDH.get.observationDate)
        outDH.isEmpty
      case rtle: RelationToLocalEntity =>
        val editedEntity: Option[Entity] = editEntityName(new Entity(rtle.mDB, rtle.getRelatedId2))
        editedEntity.isEmpty
      case rtre: RelationToRemoteEntity =>
        val editedEntity: Option[Entity] = editEntityName(new Entity(rtre.getRemoteDatabase, rtre.getRelatedId2))
        editedEntity.isEmpty
      case rtg: RelationToGroup =>
        val editedGroupName: Option[String] = Util.editGroupName(new Group(rtg.mDB, rtg.getGroupId), ui)
        editedGroupName.isEmpty
      case _ => throw new scala.Exception("Unexpected type: " + attributeIn.getClass.getCanonicalName)
    }
  }

  /**
   * @return (See addAttribute method.)
   */
  def askForInfoAndAddAttribute[T <: AttributeDataHolder](dbIn: Database, dhIn: T, askForAttrTypeId: Boolean, attrType: String,
                                                          promptForSelectingTypeId: Option[String],
                                                          getOtherInfoFromUser: (Database, T, Boolean, TextUI) => Option[T],
                                                          addTypedAttribute: (T) => Option[Attribute]): Option[Attribute] = {
    val ans: Option[T] = askForAttributeData[T](dbIn, dhIn, askForAttrTypeId, attrType, promptForSelectingTypeId,
                                                None, None, getOtherInfoFromUser, editingIn = false)
    if (ans.isDefined) {
      val dhOut: T = ans.get
      addTypedAttribute(dhOut)
    } else None
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
        askForNameAndWriteEntity(entityIn.mDB, Util.RELATION_TYPE_TYPE, Some(relTypeIn), Some(relTypeIn.getName), Some(relTypeIn.getDirectionality),
                                 if (previousNameInReverse == null || previousNameInReverse.trim().isEmpty) None else Some(previousNameInReverse),
                                 None)
      case entity: Entity =>
        val entityNameBeforeEdit: String = entityIn.getName
        val editedEntity: Option[Entity] = askForNameAndWriteEntity(entityIn.mDB, Util.ENTITY_TYPE, Some(entity), Some(entity.getName), None, None, None)
        if (editedEntity.isDefined) {
          val entityNameAfterEdit: String = editedEntity.get.getName
          if (entityNameBeforeEdit != entityNameAfterEdit) {
            val (_, _, groupId, groupName, moreThanOneAvailable) = editedEntity.get.findRelationToAndGroup
            if (groupId.isDefined && !moreThanOneAvailable) {
              val attrCount = entityIn.getAttributeCount
              // for efficiency, if it's obvious which subgroup's name to change at the same time, offer to do so
              val defaultAnswer = if (attrCount > 1) Some("n") else Some("y")
              val ans = ui.askYesNoQuestion("There's a single subgroup named \"" + groupName + "\"" +
                                            (if (attrCount > 1) " (***AMONG " + (attrCount - 1) + " OTHER ATTRIBUTES***)" else "") +
                                            "; possibly it and this entity were created at the same time.  Also change" +
                                            " the subgroup's name now to be identical?", defaultAnswer)
              if (ans.isDefined && ans.get) {
                val group = new Group(entityIn.mDB, groupId.get)
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

  /** Returns data, or None if user wants to cancel/get out.
    * @param attrType Constant referring to Attribute subtype, as used by the inObjectType parameter to the chooseOrCreateObject method
    *                 (ex., Controller.QUANTITY_TYPE).  See comment on that method, for that parm.
    * */
  def askForAttributeData[T <: AttributeDataHolder](dbIn: Database, inoutDH: T, alsoAskForAttrTypeId: Boolean, attrType: String, attrTypeInputPrompt: Option[String],
                                                    inPreviousSelectionDesc: Option[String], inPreviousSelectionId: Option[Long],
                                                    askForOtherInfo: (Database, T, Boolean, TextUI) => Option[T], editingIn: Boolean): Option[T] = {
    val (userWantsOut: Boolean, attrTypeId: Long, isRemote, remoteKey) = {
      if (alsoAskForAttrTypeId) {
        require(attrTypeInputPrompt.isDefined)
        val ans: Option[(IdWrapper, Boolean, String)] = chooseOrCreateObject(dbIn, Some(List(attrTypeInputPrompt.get)), inPreviousSelectionDesc,
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
      val ans2: Option[T] = askForOtherInfo(dbIn, inoutDH, editingIn, ui)
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
  @tailrec final def findExistingObjectByText(dbIn: Database, startingDisplayRowIndexIn: Long = 0, attrTypeIn: String,
                                              //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) w/in this method!
                                              idToOmitIn: Option[Long] = None, regexIn: String): Option[IdWrapper] = {
    val leadingText = List[String]("SEARCH RESULTS: " + Util.pickFromListPrompt)
    val choices: Array[String] = Array(Util.listNextItemsPrompt)
    val numDisplayableItems = ui.maxColumnarChoicesToDisplayAfter(leadingText.size, choices.length, Util.maxNameLength)

    val objectsToDisplay = attrTypeIn match {
      case Util.ENTITY_TYPE =>
        dbIn.getMatchingEntities(startingDisplayRowIndexIn, Some(numDisplayableItems), idToOmitIn, regexIn)
      case Util.GROUP_TYPE =>
        dbIn.getMatchingGroups(startingDisplayRowIndexIn, Some(numDisplayableItems), idToOmitIn, regexIn)
      case _ =>
        throw new OmException("??")
    }
    if (objectsToDisplay.size == 0) {
      ui.displayText("End of list, or none found; starting over from the beginning...")
      if (startingDisplayRowIndexIn == 0) None
      else findExistingObjectByText(dbIn, 0, attrTypeIn, idToOmitIn, regexIn)
    } else {
      val objectNames: Array[String] = objectsToDisplay.toArray.map {
                                                                      case entity: Entity =>
                                                                        val numSubgroupsPrefix: String = getEntityContentSizePrefix(entity)
                                                                        numSubgroupsPrefix + entity.getArchivedStatusDisplayString + entity.getName
                                                                      case group: Group =>
                                                                        val numSubgroupsPrefix: String = getGroupContentSizePrefix(group.mDB, group.getId)
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
          findExistingObjectByText(dbIn, nextStartingIndex, attrTypeIn, idToOmitIn, regexIn)
        } else if (answer > choices.length && answer <= (choices.length + objectsToDisplay.size)) {
          // those in the condition on the previous line are 1-based, not 0-based.
          val index = answer - choices.length - 1
          val o = objectsToDisplay.get(index)
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
                val someRelationToGroups: java.util.ArrayList[RelationToGroup] = o.asInstanceOf[Group].getContainingRelationsToGroup(0, Some(1))
                if (someRelationToGroups.size < 1) {
                  ui.displayText(Util.ORPHANED_GROUP_MESSAGE)
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
          ui.displayText("unknown choice among secondary list")
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
                                 previousSelectionIdIn: Option[Long], objectTypeIn: String, startingDisplayRowIndexIn: Long = 0,
                                 classIdIn: Option[Long] = None, limitByClassIn: Boolean = false,
                                 containingGroupIn: Option[Long] = None,
                                 markPreviousSelectionIn: Boolean = false,
                                 showOnlyAttributeTypesIn: Option[Boolean] = None,
                                 quantitySeeksUnitNotTypeIn: Boolean = false
                                 //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) w/in this method! (not
                                 // necessary if calling for a separate object type, but just when intended to ~"start over with the same thing").
                                 ): Option[(IdWrapper, Boolean, String)] = {
    if (classIdIn.isDefined) require(objectTypeIn == Util.ENTITY_TYPE)
    if (quantitySeeksUnitNotTypeIn) require(objectTypeIn == Util.QUANTITY_TYPE)
    val entityAndMostAttrTypeNames = Array(Util.ENTITY_TYPE, Util.QUANTITY_TYPE, Util.DATE_TYPE, Util.BOOLEAN_TYPE,
                                  Util.FILE_TYPE, Util.TEXT_TYPE)
    val evenMoreAttrTypeNames = Array(Util.ENTITY_TYPE, Util.TEXT_TYPE, Util.QUANTITY_TYPE, Util.DATE_TYPE, Util.BOOLEAN_TYPE,
                                      Util.FILE_TYPE, Util.RELATION_TYPE_TYPE, Util.RELATION_TO_LOCAL_ENTITY_TYPE,
                                      Util.RELATION_TO_GROUP_TYPE)
    val listNextItemsChoiceNum = 1

    val (numObjectsAvailable: Long, showOnlyAttributeTypes: Boolean) = {
      // ** KEEP THESE QUERIES AND CONDITIONS IN SYNC W/ THE COROLLARY ONES 1x ELSEWHERE ! (at similar comment):
      if (Util.nonRelationAttrTypeNames.contains(objectTypeIn)) {
        if (showOnlyAttributeTypesIn.isEmpty) {
          val countOfEntitiesUsedAsThisAttrType: Long = dbIn.getCountOfEntitiesUsedAsAttributeTypes(objectTypeIn, quantitySeeksUnitNotTypeIn)
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
      var keepPreviousSelectionChoiceNum = 1
      var createAttrTypeChoiceNum = 1
      var searchForEntityByNameChoiceNum = 1
      var searchForEntityByIdChoiceNum = 1
      var showJournalChoiceNum = 1
      var swapObjectsToDisplayChoiceNum = 1
      var linkToRemoteInstanceChoiceNum = 1
      var createRelationTypeChoiceNum = 1
      var createClassChoiceNum = 1
      var createInstanceChoiceNum = 1
      var choiceList = Array(Util.listNextItemsPrompt)
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
      val prefix: String = objectTypeIn match {
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
      var leadingText = leadingTextIn.getOrElse(List[String](prefix + "Pick from menu, or an item by letter; Alt+<letter> to go to the item & later come back)"))
      val numDisplayableItems = ui.maxColumnarChoicesToDisplayAfter(leadingText.size + 3 /* up to: see more of leadingText below .*/ , choicesIn.length,
                                                                    Util.maxNameLength)
      val objectsToDisplay = {
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
        val txt: String = TextUI.NEWLN + TextUI.NEWLN + "(None of the needed " + (if (objectTypeIn == Util.RELATION_TYPE_TYPE) "relation types" else "entities") +
                          " have been created in this model, yet."
        leadingText = leadingText ::: List(txt)
      }
      Util.addRemainingCountToPrompt(choicesIn, objectsToDisplay.size, numObjectsAvailable, startingDisplayRowIndexIn)
      val objectStatusesAndNames: Array[String] = objectsToDisplay.toArray.map {
                                                                      case entity: Entity => entity.getArchivedStatusDisplayString + entity.getName
                                                                      case clazz: EntityClass => clazz.getName
                                                                      case omInstance: OmInstance => omInstance.getDisplayString
                                                                      case x: Any => throw new Exception("unexpected class: " + x.getClass.getName)
                                                                      case _ => throw new Exception("??")
                                                                    }
      (leadingText, objectsToDisplay, objectStatusesAndNames)
    }

    def getNextStartingObjectIndex(previousListLength: Long, numObjectsAvailableIn: Long): Long = {
      val index = {
        val x = startingDisplayRowIndexIn + previousListLength
        // ask Model for list of obj's w/ count desired & starting index (or "first") (in a sorted map, w/ id's as key, and names)
        //idea: should this just reuse the "totalExisting" value alr calculated in above in getLeadTextAndObjectList just above?
        if (x >= numObjectsAvailableIn) {
          ui.displayText("End of list found; starting over from the beginning.")
          0 // start over
        } else x
      }
      index
    }

    val (choices, keepPreviousSelectionChoice, createEntityOrAttrTypeChoice, searchForEntityByNameChoice, searchForEntityByIdChoice, showJournalChoice, createRelationTypeChoice, createClassChoice, createInstanceChoice, swapObjectsToDisplayChoice, linkToRemoteInstanceChoice): (Array[String],
      Int, Int, Int, Int, Int, Int, Int, Int, Int, Int) = getChoiceList

    val (leadingText, objectsToDisplay, statusesAndNames) = getLeadTextAndObjectList(choices)
    val ans = ui.askWhichChoiceOrItsAlternate(Some(leadingText.toArray), choices, statusesAndNames)

    if (ans.isEmpty) None
    else {
      val answer = ans.get._1
      val userChoseAlternate = ans.get._2
      if (answer == listNextItemsChoiceNum && answer <= choices.length && !userChoseAlternate) {
        // (For reason behind " && answer <= choices.length", see comment where it is used in entityMenu.)
        val index: Long = getNextStartingObjectIndex(objectsToDisplay.size, numObjectsAvailable)
        chooseOrCreateObject(dbIn, leadingTextIn, previousSelectionDescIn, previousSelectionIdIn, objectTypeIn, index, classIdIn, limitByClassIn,
                             containingGroupIn, markPreviousSelectionIn, Some(showOnlyAttributeTypes), quantitySeeksUnitNotTypeIn)
      } else if (answer == keepPreviousSelectionChoice && answer <= choices.length) {
        // Such as if editing several fields on an attribute and doesn't want to change the first one.
        // Not using "get out" option for this because it would exit from a few levels at once and
        // then user wouldn't be able to proceed to other field edits.
        Some(new IdWrapper(previousSelectionIdIn.get), false, "")
      } else if (answer == createEntityOrAttrTypeChoice && answer <= choices.length) {
        val e: Option[Entity] = askForClassInfoAndNameAndCreateEntity(dbIn, classIdIn)
        if (e.isEmpty) {
          None
        } else {
          Some(new IdWrapper(e.get.getId), false, "")
        }
      } else if (answer == searchForEntityByNameChoice && answer <= choices.length) {
        val result = askForNameAndSearchForEntity(dbIn)
        if (result.isEmpty) {
          None
        } else {
          Some(result.get, false, "")
        }
      } else if (answer == searchForEntityByIdChoice && answer <= choices.length) {
        val result = searchById(dbIn, Util.ENTITY_TYPE)
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
            val results: util.ArrayList[(Long, String, Long)] = dbIn.findJournalEntries(beginDate.get, endDate.get)
            for (result: (Long, String, Long) <- results) {
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
      } else if (answer == swapObjectsToDisplayChoice && entityAndMostAttrTypeNames.contains(objectTypeIn) && answer <= choices.length) {
        chooseOrCreateObject(dbIn, leadingTextIn, previousSelectionDescIn, previousSelectionIdIn, objectTypeIn, 0, classIdIn, limitByClassIn,
                             containingGroupIn, markPreviousSelectionIn, Some(!showOnlyAttributeTypes), quantitySeeksUnitNotTypeIn)
      } else if (answer == linkToRemoteInstanceChoice && entityAndMostAttrTypeNames.contains(objectTypeIn) && answer <= choices.length) {
        val omInstanceIdOption: Option[(_, _, String)] = chooseOrCreateObject(dbIn, None, None, None, Util.OM_INSTANCE_TYPE)
        if (omInstanceIdOption.isEmpty) {
          None
        } else {
          val remoteOmInstance = new OmInstance(dbIn, omInstanceIdOption.get._3)
          val remoteEntityEntryTypeAnswer = ui.askWhich(leadingTextIn = Some(Array("SPECIFY AN ENTITY IN THE REMOTE INSTANCE")),
                                                        choicesIn = Array("Enter an entity id #", "Use the remote site's default entity"))
          if (remoteEntityEntryTypeAnswer.isEmpty) {
            None
          } else {
            val restDb = Database.getRestDatabase(remoteOmInstance.getAddress)
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
                val defaultEntityId: Option[Long] = restDb.getDefaultEntity(Some(ui))
                if (defaultEntityId.isEmpty) None
                else defaultEntityId
              } else {
                None
              }
            }
            if (remoteEntityId.isEmpty) None
            else {
              val entityInJson: Option[String] = restDb.getEntityJson_WithOptionalErrHandling(Some(ui), remoteEntityId.get)
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
      } else if (answer == createRelationTypeChoice && Util.relationAttrTypeNames.contains(objectTypeIn) && answer <= choices.length) {
        val entity: Option[Entity] = askForNameAndWriteEntity(dbIn, Util.RELATION_TYPE_TYPE)
        if (entity.isEmpty) None
        else Some(new IdWrapper(entity.get.getId), false, "")
      } else if (answer == createClassChoice && objectTypeIn == Util.ENTITY_CLASS_TYPE && answer <= choices.length) {
        val result: Option[(Long, Long)] = askForAndWriteClassAndTemplateEntityName(dbIn)
        if (result.isEmpty) None
        else {
          val (classId, entityId) = result.get
          val ans = ui.askYesNoQuestion("Do you want to add attributes to the newly created template entity for this class? (These will be used for the " +
                                        "prompts " +
                                        "and defaults when creating/editing entities in this class).", Some("y"))
          if (ans.isDefined && ans.get) {
            new EntityMenu(ui, this).entityMenu(new Entity(dbIn, entityId))
          }
          Some(new IdWrapper(classId), false, "")
        }
      } else if (answer == createInstanceChoice && objectTypeIn == Util.OM_INSTANCE_TYPE && answer <= choices.length) {
        val result: Option[String] = askForAndWriteOmInstanceInfo(dbIn)
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
        ui.displayText("unknown response in chooseOrCreateObject")
        chooseOrCreateObject(dbIn, leadingTextIn, previousSelectionDescIn, previousSelectionIdIn, objectTypeIn, startingDisplayRowIndexIn, classIdIn,
                             limitByClassIn, containingGroupIn, markPreviousSelectionIn, Some(showOnlyAttributeTypes), quantitySeeksUnitNotTypeIn)
      }
    }
  }

  def askForNameAndSearchForEntity(dbIn: Database): Option[IdWrapper] = {
    val ans = ui.askForString(Some(Array(Util.searchPrompt(Util.ENTITY_TYPE))))
    if (ans.isEmpty) {
      None
    } else {
      // Allow relation to self (eg, picking self as 2nd part of a RelationToLocalEntity), so None in 3nd parm.
      val e: Option[IdWrapper] = findExistingObjectByText(dbIn, 0, Util.ENTITY_TYPE, None, ans.get)
      if (e.isEmpty) None
      else Some(new IdWrapper(e.get.getId))
    }
  }

  def searchById(dbIn: Database, typeNameIn: String): Option[IdWrapper] = {
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
        // (BTW, do allow relation to self, ex., picking self as 2nd part of a RelationToLocalEntity.)
        // (Also, the call to entityKeyExists should here include archived entities so the user can find out if the one
        // needed is archived, even if the hard way.)
        if ((typeNameIn == Util.ENTITY_TYPE && dbIn.entityKeyExists(idString.toLong)) ||
            (typeNameIn == Util.GROUP_TYPE && dbIn.groupKeyExists(idString.toLong))) {
          Some(new IdWrapper(idString.toLong))
        } else {
          ui.displayText("The " + typeNameIn + " ID " + ans.get + " was not found in the database.")
          None
        }
      }
    }
  }

  /** Returns None if user wants to cancel. */
  def askForQuantityAttributeNumberAndUnit(dbIn: Database, dhIn: QuantityAttributeDataHolder, editingIn: Boolean, ui: TextUI): Option[QuantityAttributeDataHolder] = {
    val outDH: QuantityAttributeDataHolder = dhIn
    val leadingText: List[String] = List("SELECT A *UNIT* FOR THIS QUANTITY (i.e., centimeters, or quarts; ESC or blank to cancel):")
    val previousSelectionDesc = if (editingIn) Some(new Entity(dbIn, dhIn.unitId).getName) else None
    val previousSelectionId = if (editingIn) Some(dhIn.unitId) else None
    val unitSelection: Option[(IdWrapper, _, _)] = chooseOrCreateObject(dbIn, Some(leadingText), previousSelectionDesc, previousSelectionId,
                                                                        Util.QUANTITY_TYPE, quantitySeeksUnitNotTypeIn = true)
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
  def askForRelToGroupInfo(dbIn: Database, dhIn: RelationToGroupDataHolder, inEditingUNUSEDForNOW: Boolean = false,
                           uiIn: TextUI): Option[RelationToGroupDataHolder] = {
    val outDH = dhIn

    val groupSelection = chooseOrCreateGroup(dbIn, Some(List("SELECT GROUP FOR THIS RELATION")), 0)
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
  @tailrec final def chooseOrCreateGroup(dbIn: Database, leadingTextIn: Option[List[String]], startingDisplayRowIndexIn: Long = 0,
                                         //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) w/in this method!
                                         containingGroupIn: Option[Long] = None /*ie group to omit from pick list*/): Option[IdWrapper] = {
    val totalExisting: Long = dbIn.getGroupCount
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
    val objectsToDisplay = dbIn.getGroups(startingDisplayRowIndexIn, Some(numDisplayableItems), containingGroupIn)
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
        chooseOrCreateGroup(dbIn, leadingTextIn, nextStartingIndex, containingGroupIn)
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
            val newGroupId = dbIn.createGroup(name, mixedClassesAllowed)
            Some(new IdWrapper(newGroupId))
          }
        }
      } else if (answer == 3 && answer <= choices.length) {
        val ans = ui.askForString(Some(Array(Util.searchPrompt(Util.GROUP_TYPE))))
        if (ans.isEmpty) None
        else {
          // Allow relation to self, so None in 2nd parm.
          val g: Option[IdWrapper] = findExistingObjectByText(dbIn, 0, Util.GROUP_TYPE, None, ans.get)
          if (g.isEmpty) None
          else Some(new IdWrapper(g.get.getId))
        }
      } else if (answer == 4 && answer <= choices.length) {
        searchById(dbIn, Util.GROUP_TYPE)
      } else if (answer > choices.length && answer <= (choices.length + objectsToDisplay.size)) {
        // those in that^ condition are 1-based, not 0-based.
        val index = answer - choices.length - 1
        val o = objectsToDisplay.get(index)
        if (userChoseAlternate) {
          // for now, picking the first RTG found for this group, until it's clear which of its RTGs to use.
          // (see also the other locations w/ similar comment!)
          val someRelationToGroups: java.util.ArrayList[RelationToGroup] = o.asInstanceOf[Group].getContainingRelationsToGroup(0, Some(1))
          new GroupMenu(ui, this).groupMenu(new Group(dbIn, someRelationToGroups.get(0).getGroupId), 0, Some(someRelationToGroups.get(0)),
                                                containingEntityIn = None)
          chooseOrCreateGroup(dbIn, leadingTextIn, startingDisplayRowIndexIn, containingGroupIn)
        } else {
          // user typed a letter to select.. (now 0-based); selected a new object and so we return to the previous menu w/ that one displayed & current
          Some(new IdWrapper(o.getId))
        }
      } else {
        ui.displayText("unknown response in findExistingObjectByText")
        chooseOrCreateGroup(dbIn, leadingTextIn, startingDisplayRowIndexIn, containingGroupIn)
      }
    }
  }

  /** Returns None if user wants to cancel. */
  def askForRelationEntityIdNumber2(dbIn: Database, dhIn: RelationToEntityDataHolder, inEditing: Boolean, uiIn: TextUI): Option[RelationToEntityDataHolder] = {
    val previousSelectionDesc = {
      if (!inEditing) None
      else Some(new Entity(dbIn, dhIn.entityId2).getName)
    }
    val previousSelectionId = {
      if (!inEditing) None
      else Some(dhIn.entityId2)
    }
    val selection: Option[(IdWrapper, Boolean, String)] = chooseOrCreateObject(dbIn, Some(List("SELECT OTHER (RELATED) ENTITY FOR THIS RELATION")),
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
    val (rtgid, rtid, groupId, _, moreThanOneAvailable) = userSelection.findRelationToAndGroup
    val subEntitySelected: Option[Entity] = None
    if (groupId.isDefined && !moreThanOneAvailable && userSelection.getAttributeCount == 1) {
      // In quick menu, for efficiency of some work like brainstorming, if it's obvious which subgroup to go to, just go there.
      // We DON'T want @tailrec on this method for this call, so that we can ESC back to the current menu & list! (so what balance/best? Maybe move this
      // to its own method, so it doesn't try to tail optimize it?)  See also the comment with 'tailrec', mentioning why to have it, above.
      new QuickGroupMenu(ui, this).quickGroupMenu(new Group(userSelection.mDB, groupId.get),
                                                      0,
                                                      Some(new RelationToGroup(userSelection.mDB, rtgid.get, userSelection.getId, rtid.get, groupId.get)),
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
  def getGroupContentSizePrefix(dbIn: Database, groupId: Long): String = {
    val grpSize = dbIn.getGroupSize(groupId, 1)
    if (grpSize == 0) ""
    else ">"
  }

  /** Shows ">" in front of an entity or group if it contains exactly one attribute or a subgroup which has at least one entry; shows ">>" if contains
    * multiple subgroups or attributes, and "" if contains no subgroups or the one subgroup is empty.
    * Idea: this might better be handled in the textui class instead, and the same for all the other color stuff.
    */
  def getEntityContentSizePrefix(entityIn: Entity): String = {
    // attrCount counts groups also, so account for the overlap in the below.
    val attrCount = entityIn.getAttributeCount
    // This is to not show that an entity contains more things (">" prefix...) if it only has one group which has no *non-archived* entities:
    val hasOneEmptyGroup: Boolean = {
      val numGroups: Long = entityIn.getRelationToGroupCount
      if (numGroups != 1) false
      else {
        val (_, _, gid: Option[Long], _, moreAvailable) = entityIn.findRelationToAndGroup
        if (gid.isEmpty || moreAvailable) throw new OmException("Found " + (if (gid.isEmpty) 0 else ">1") + " but by the earlier checks, " +
                                                                        "there should be exactly one group in entity " + entityIn.getId + " .")
        val groupSize = entityIn.mDB.getGroupSize(gid.get, 1)
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
          val idWrapper: Option[(IdWrapper, _, _)] = chooseOrCreateObject(groupIn.mDB, Some(leadingText), None, None, Util.ENTITY_TYPE,
                                                                  containingGroupIn = Some(groupIn.getId))
          if (idWrapper.isDefined) {
            groupIn.addEntity(idWrapper.get._1.getId)
            Some(idWrapper.get._1.getId)
          } else None
        } else {
          // it's not the 1st entry in the group, so add an entity using the same class as those previously added (or None as case may be).
          val entityClassInUse: Option[Long] = groupIn.getClassId
          val idWrapper: Option[(IdWrapper, _, _)] = chooseOrCreateObject(groupIn.mDB, None, None, None, Util.ENTITY_TYPE, 0, entityClassInUse,
                                                                          limitByClassIn = true, containingGroupIn = Some(groupIn.getId))
          if (idWrapper.isEmpty) None
          else {
            val entityId = idWrapper.get._1.getId
            try {
              groupIn.addEntity(entityId)
              Some(entityId)
            } catch {
              case e: Exception =>
                if (e.getMessage.contains(Database.MIXED_CLASSES_EXCEPTION)) {
                  val oldClass: String = if (entityClassInUse.isEmpty) {
                    "(none)"
                  } else {
                    new EntityClass(groupIn.mDB, entityClassInUse.get).getDisplayString
                  }
                  val newClassId = new Entity(groupIn.mDB, entityId).getClassId
                  val newClass: String =
                    if (newClassId.isEmpty || entityClassInUse.isEmpty) "(none)"
                    else {
                      val ec = new EntityClass(groupIn.mDB, entityClassInUse.get)
                      ec.getDisplayString
                    }
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
        val idWrapper: Option[(IdWrapper, _, _)] = chooseOrCreateObject(groupIn.mDB, Some(leadingText), None, None, Util.ENTITY_TYPE,
                                                                containingGroupIn = Some(groupIn.getId))
        if (idWrapper.isDefined) {
          groupIn.addEntity(idWrapper.get._1.getId)
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
                                                                                                  val reltype = new RelationType(entity.mDB, relTypeId)
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
   *                   EntityMenu.addAttribute which passes in "other" formIds).  BUT, there are also cases
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
      val result: Option[FileAttribute] = askForInfoAndAddAttribute[FileAttributeDataHolder](entityIn.mDB, new FileAttributeDataHolder(attrTypeId, "", ""),
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
      askForInfoAndAddAttribute[TextAttributeDataHolder](entityIn.mDB, new TextAttributeDataHolder(attrTypeId, None, System.currentTimeMillis(), ""),
                                                         askForAttrTypeId, Util.TEXT_TYPE,
                                                         Some("SELECT TYPE OF " + Util.textDescription + ": "), Util.askForTextAttributeText, addTextAttribute)
    } else if (attrFormIn == Database.getAttributeFormId(Util.RELATION_TO_LOCAL_ENTITY_TYPE)) {
      //(This is in a condition that says "...LOCAL..." but is also for RELATION_TO_REMOTE_ENTITY_TYPE.  See caller for details if interested.)
      def addRelationToEntity(dhIn: RelationToEntityDataHolder): Option[AttributeWithValidAndObservedDates] = {
        val relation = {
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
      val eId: Option[IdWrapper] = askForNameAndSearchForEntity(entityIn.mDB)
      if (eId.isDefined) {
        Some(entityIn.addHASRelationToLocalEntity(eId.get.getId, None, System.currentTimeMillis))
      } else {
        None
      }
    } else if (attrFormIn == Database.getAttributeFormId(Util.RELATION_TO_GROUP_TYPE)) {
      def addRelationToGroup(dhIn: RelationToGroupDataHolder): Option[RelationToGroup] = {
        require(dhIn.entityId == entityIn.getId)
        val newRTG: RelationToGroup = entityIn.addRelationToGroup(dhIn.attrTypeId, dhIn.groupId, None, dhIn.validOnDate, dhIn.observationDate)
        Some(newRTG)
      }
      val result: Option[Attribute] = askForInfoAndAddAttribute[RelationToGroupDataHolder](entityIn.mDB,
                                                                                           new RelationToGroupDataHolder(entityIn.getId, attrTypeId, 0,
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
        new QuickGroupMenu(ui, this).quickGroupMenu(new Group(entityIn.mDB, newRtg.getGroupId), 0, Some(newRtg), None, containingEntityIn = Some(entityIn))
        // user could have deleted the new result: check that before returning it as something to act upon:
        if (entityIn.mDB.relationToGroupKeyExists(newRtg.getId)) {
          result
        } else {
          None
        }
      }
    } else if (attrFormIn == 101  /*re "101": an "external web page"; for details see comments etc at javadoc above for attrFormIn.*/) {
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
      val (newEntity: Entity, newRTE: RelationToLocalEntity) = entityIn.addUriEntityWithUriAttribute(newEntityName.get, uri, System.currentTimeMillis(),
                                                                                          entityIn.getPublic, callerManagesTransactionsIn = false, quote)
      new EntityMenu(ui, this).entityMenu(newEntity, containingRelationToEntityIn = Some(newRTE))
      // user could have deleted the new result: check that before returning it as something to act upon:
      if (entityIn.mDB.relationToLocalEntityKeyExists(newRTE.getId) && entityIn.mDB.entityKeyExists(newEntity.getId)) {
        Some(newRTE)
      } else {
        None
      }
    } else {
      ui.displayText("invalid response")
      None
    }
  }

  def defaultAttributeCopying(targetEntityIn: Entity, attributeTuplesIn: Option[Array[(Long, Attribute)]] = None): Unit = {
    if (shouldTryAddingDefaultAttributes(targetEntityIn)) {
      val attributeTuples: Array[(Long, Attribute)] = {
        if (attributeTuplesIn.isDefined) attributeTuplesIn.get
        else targetEntityIn.getSortedAttributes(onlyPublicEntitiesIn = false)._1
      }
      val templateEntity: Option[Entity] = {
        val templateId: Option[Long] = targetEntityIn.getClassTemplateEntityId
        if (templateId.isEmpty) {
          None
        } else {
          Some(new Entity(targetEntityIn.mDB, templateId.get))
        }
      }
      val templateAttributesToCopy: ArrayBuffer[Attribute] = getMissingAttributes(templateEntity, attributeTuples)
      copyAndEditAttributes(targetEntityIn, templateAttributesToCopy)
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

    var askAboutRteEveryTime: Option[Boolean] = None
    var (allCopy: Boolean, allCreateOrSearch: Boolean, allKeepReference: Boolean) = (false, false, false)
    var attrCounter = 0
    for (attributeFromTemplate: Attribute <- templateAttributesToCopyIn) {
      attrCounter += 1
      if (!userWantsOut) {
        val waitForKeystroke: Boolean = {
          attributeFromTemplate match {
            case a: RelationToLocalEntity => true
            case a: RelationToRemoteEntity => true
            case _ => false
          }
        }
        def promptToEditAttributeCopy() {
          ui.displayText("Edit the copied " + Database.getAttributeFormName(attributeFromTemplate.getFormId) + " \"" +
                         attributeFromTemplate.getDisplayString(0, None, None, simplify = true) + "\", from the template entity (ESC to abort):",
                         waitForKeystroke)
        }
        val newAttribute: Option[Attribute] = {
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
              ui.displayText("You can add a FileAttribute manually afterwards for this attribute.  Maybe it can be automated " +
                             "more, when use cases for this part are more clear.")
              None
            case templateAttribute: TextAttribute =>
              promptToEditAttributeCopy()
              Some(entityIn.addTextAttribute(templateAttribute.getAttrTypeId, templateAttribute.getText, Some(templateAttribute.getSortingIndex)))
            case templateAttribute: RelationToLocalEntity =>
              val (newRTE, askEveryTime) = copyAndEditRelationToEntity(entityIn, templateAttribute, askAboutRteEveryTime)
              askAboutRteEveryTime = askEveryTime
              newRTE
            case templateAttribute: RelationToRemoteEntity =>
              val (newRTE, askEveryTime) = copyAndEditRelationToEntity(entityIn, templateAttribute, askAboutRteEveryTime)
              askAboutRteEveryTime = askEveryTime
              newRTE
            case templateAttribute: RelationToGroup =>
              promptToEditAttributeCopy()
              val templateGroup = templateAttribute.getGroup
              val (_, newRTG: RelationToGroup) = entityIn.addGroupAndRelationToGroup(templateAttribute.getAttrTypeId, templateGroup.getName,
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
    def copyAndEditRelationToEntity(entityIn: Entity, relationToEntityAttributeFromTemplateIn: Attribute,
                                    askEveryTimeIn: Option[Boolean] = None): (Option[Attribute], Option[Boolean]) = {
      require(relationToEntityAttributeFromTemplateIn.isInstanceOf[RelationToLocalEntity] ||
              relationToEntityAttributeFromTemplateIn.isInstanceOf[RelationToRemoteEntity])
      val choice1text = "Copy the template entity, editing its name (**MOST LIKELY CHOICE)"
      val copyFromTemplateAndEditNameChoiceNum = 1
      val choice2text = "Create a new entity or search for an existing one for this purpose"
      val createOrSearchForEntityChoiceNum = 2
      val choice3text = "Keep a reference to the same entity as in the template (least likely choice)"
      val keepSameReferenceAsInTemplateChoiceNum = 3

      var askEveryTime: Option[Boolean] = None
      askEveryTime = {
        if (askEveryTimeIn.isDefined) {
          askEveryTimeIn
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
        (None, askEveryTime)
      } else {
        val howCopyRteResponse: Option[Int] = {
          if (askEveryTime.get) {
            val whichRteLeadingText: Array[String] = Array("The template has a templateAttribute which is a relation to an entity named \"" +
                                                           relationToEntityAttributeFromTemplateIn.getDisplayString(0, None, None, simplify = true) +
                                                           "\": how would you like the equivalent to be provided for this new entity being created?" +
                                                           " (0/ESC to just skip this one for now)")
            val whichRTEChoices = Array[String](choice1text, choice2text, choice3text)
            ui.askWhich(Some(whichRteLeadingText), whichRTEChoices)
          } else {
            None
          }
        }
        if (askEveryTime.get && howCopyRteResponse.isEmpty) {
          (None, askEveryTime)
        } else {
          val relatedId2: Long = {
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
            val currentOrRemoteDbForRelatedEntity = Database.currentOrRemoteDb(relationToEntityAttributeFromTemplateIn,
                                                                               relationToEntityAttributeFromTemplateIn.mDB)
            val templatesRelatedEntity: Entity = new Entity(currentOrRemoteDbForRelatedEntity, relatedId2)
            val oldName: String = templatesRelatedEntity.getName
            val newEntity: Option[Entity] = {
              //noinspection TypeCheckCanBeMatch
              if (relationToEntityAttributeFromTemplateIn.isInstanceOf[RelationToLocalEntity]) {
                askForNameAndWriteEntity(entityIn.mDB, Util.ENTITY_TYPE, None, Some(oldName), None, None, templatesRelatedEntity.getClassId,
                                         Some("EDIT THE " + "ENTITY NAME:"), duplicateNameProbablyOK = true)
              } else if (relationToEntityAttributeFromTemplateIn.isInstanceOf[RelationToRemoteEntity]) {
                val e = askForNameAndWriteEntity(entityIn.mDB, Util.ENTITY_TYPE, None, Some(oldName), None, None, None,
                                         Some("EDIT THE ENTITY NAME:"), duplicateNameProbablyOK = true)
                if (e.isDefined && templatesRelatedEntity.getClassId.isDefined) {
                  val remoteClassId: Long = templatesRelatedEntity.getClassId.get
                  val remoteClassName: String = new EntityClass(currentOrRemoteDbForRelatedEntity, remoteClassId).getName
                  ui.displayText("Note: Did not write a class on the new entity to match that from the remote entity, until some kind of synchronization " +
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
              val newRTLE = Some(entityIn.addRelationToLocalEntity(relationToEntityAttributeFromTemplateIn.getAttrTypeId, newEntity.get.getId,
                                                     Some(relationToEntityAttributeFromTemplateIn.getSortingIndex)))
              (newRTLE, askEveryTime)
            }
          } else if (allCreateOrSearch || (howCopyRteResponse.isDefined && howCopyRteResponse.get == createOrSearchForEntityChoiceNum)) {
            val rteDh = new RelationToEntityDataHolder(relationToEntityAttributeFromTemplateIn.getAttrTypeId, None, System.currentTimeMillis(), 0, false, "")
            val dh: Option[RelationToEntityDataHolder] = askForRelationEntityIdNumber2(entityIn.mDB, rteDh, inEditing = false, ui)
            if (dh.isDefined) {
  //            val relation = entityIn.addRelationToEntity(dh.get.attrTypeId, dh.get.entityId2, Some(relationToEntityAttributeFromTemplateIn.getSortingIndex),
  //                                                        dh.get.validOnDate, dh.get.observationDate,
  //                                                        dh.get.isRemote, if (!dh.get.isRemote) None else Some(dh.get.remoteInstanceId))
              if (dh.get.isRemote) {
                val rtre = entityIn.addRelationToRemoteEntity(dh.get.attrTypeId, dh.get.entityId2, Some(relationToEntityAttributeFromTemplateIn.getSortingIndex),
                                                              dh.get.validOnDate, dh.get.observationDate, dh.get.remoteInstanceId)
                (Some(rtre), askEveryTime)
              } else {
                val rtle = entityIn.addRelationToLocalEntity(dh.get.attrTypeId, dh.get.entityId2, Some(relationToEntityAttributeFromTemplateIn.getSortingIndex),
                                                             dh.get.validOnDate, dh.get.observationDate)
                (Some(rtle), askEveryTime)
              }
            } else {
              (None, askEveryTime)
            }
          } else if (allKeepReference || (howCopyRteResponse.isDefined && howCopyRteResponse.get == keepSameReferenceAsInTemplateChoiceNum)) {
            val relation = {
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
            ui.displayText("Unexpected answer: " + allCopy + "/" + allCreateOrSearch + "/" + allKeepReference + "/" + askEveryTime.getOrElse(None) +
                           howCopyRteResponse.getOrElse(None))
            (None, askEveryTime)
          }
        }
      }
    }
  }

  def getMissingAttributes(classTemplateEntityIn: Option[Entity], existingAttributeTuplesIn: Array[(Long, Attribute)]): ArrayBuffer[Attribute] = {
    val templateAttributesToSuggestCopying: ArrayBuffer[Attribute] = {
      // This determines which attributes from the template entity (or "pattern" or "class-defining entity") are not found on this entity, so they can
      // be added if the user wishes.
      val attributesToSuggestCopying_workingCopy: ArrayBuffer[Attribute] = new ArrayBuffer()
      if (classTemplateEntityIn.isDefined) {
        // ("cde" in name means "classDefiningEntity" (aka template))
        val (cde_attributeTuples: Array[(Long, Attribute)], _) = classTemplateEntityIn.get.getSortedAttributes(onlyPublicEntitiesIn = false)
        for (cde_attributeTuple <- cde_attributeTuples) {
          var attributeTypeFoundOnEntity = false
          val cde_attribute = cde_attributeTuple._2
          for (attributeTuple <- existingAttributeTuplesIn) {
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
      val createAttributes: Option[Boolean] = new EntityClass(entityIn.mDB, entityIn.getClassId.get).getCreateDefaultAttributes
      if (createAttributes.isDefined) {
        createAttributes.get
      } else {
        if (entityIn.getClassTemplateEntityId.isEmpty) {
          false
        } else {
          val attrCount = new Entity(entityIn.mDB, entityIn.getClassTemplateEntityId.get).getAttributeCount
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
