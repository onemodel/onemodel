/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2015-2020 inclusive, Luke A. Call; all rights reserved.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule, guidelines around binary
    distribution, and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
package org.onemodel.core.controllers

import java.util

import org.onemodel.core._
import org.onemodel.core.{OmException, TextUI}
import org.onemodel.core.model._

import scala.collection.mutable.ArrayBuffer
import scala.collection.JavaConversions._

/** This is simply to hold less-used operations so the main EntityMenu can be the most-used stuff.
  */
class OtherEntityMenu (val ui: TextUI, val controller: Controller) {

  def otherEntityMenu(entityIn: Entity, attributeRowsStartingIndexIn: Int = 0, relationSourceEntityIn: Option[Entity],
                      containingRelationToEntityIn: Option[AttributeWithValidAndObservedDates], containingGroupIn: Option[Group],
                      attributeTuplesIn: Array[(Long, Attribute)]) {
    require(containingRelationToEntityIn.isEmpty ||
            containingRelationToEntityIn.get.isInstanceOf[RelationToLocalEntity] || containingRelationToEntityIn.get.isInstanceOf[RelationToRemoteEntity])
    try {
      require(entityIn != null)
      val leadingText = Array[String]{ Util.entityMenuLeadingText(entityIn) }
      var choices = Array[String]("Edit public/nonpublic status",
                                  "Import/Export...",
                                  "Edit...",
                                  "Delete or " + (if (entityIn.isArchived) "Un-archive" else "Archive") + " this entity (or link)...",
                                  "Go to other related entities or groups...",
                                  "(stub)")
      //  don't show the "set default" option if it's already been done w/ this same one:
      val defaultEntityTuple: Option[(Long, Entity)] = controller.getDefaultEntity
      val defaultEntity: Option[Long] = if (defaultEntityTuple.isEmpty) None else Some(defaultEntityTuple.get._1)
      val entityIsAlreadyTheDefault: Boolean = defaultEntity.isDefined && defaultEntity.get == entityIn.getId
      if (! entityIsAlreadyTheDefault) {
        choices = choices :+ ((if (defaultEntity.isEmpty && !entityIn.mDB.isRemote) "****TRY ME---> " else "") +
                              "Set current entity as default (first to come up when launching this program.)")
      } else choices = choices :+ "(stub)"

      val response = ui.askWhich(Some(leadingText), choices)
      if (response.isDefined) {
        val answer = response.get
        if (answer == 1) {
          val valueBeforeEntry: Option[Boolean] = entityIn.getPublic
          val valueAfterEntry: Option[Boolean] = controller.askForPublicNonpublicStatus(valueBeforeEntry)
          val rteCount: Long = entityIn.getRelationToLocalEntityCount(includeArchivedEntitiesIn = false)
          val rtgCount: Long = entityIn.getRelationToGroupCount
          val whichToUpdateChoices = {
            if (rteCount > 0) {
              Array("...for this entity (\"" + entityIn.getName + "\")",
                    "...for its " + rteCount + " contained entities (one level, local), and all the" +
                    " entities contained in its " + rtgCount + " groups (one level)",
                    "...for both.")
            } else {
              Array("...for this entity only (\"" + entityIn.getName + "\").")
            }
          }
          val publicMenuResponse = ui.askWhich(Some(Array[String]{"Confirm:"}), whichToUpdateChoices)
          if (publicMenuResponse.isDefined) {
            if (publicMenuResponse.get == 1) {
              entityIn.updatePublicStatus(valueAfterEntry)
            } else if (publicMenuResponse.get == 2) {
              val count: Int = entityIn.updateContainedEntitiesPublicStatus(valueAfterEntry)
              ui.displayText("Updated " + count + " contained entities with new status.")
            } else if (publicMenuResponse.get == 3) {
              entityIn.updatePublicStatus(valueAfterEntry)
              val count: Int = entityIn.updateContainedEntitiesPublicStatus(valueAfterEntry)
              ui.displayText("Updated this entity and " + count + " contained entities with new status.")
            } else {
              ui.displayText("invalid response")
            }
          }
        } else if (answer == 2) {
          val importOrExportAnswer = ui.askWhich(Some(Array("NOTE: this is very useful for getting things in & out of OM, but is not" +
                                                            " complete or tested enough" +
                                                            " to use for OM backup/restore.  (That has to be done at the database level.  Try the mailing" +
                                                            " list for help with that.  If it is a hosted OM solution the backups should be done for you.)")),
                                                 Array("Import", "Export to a text file (outline)", "Export to html pages"), Array[String]())
          if (importOrExportAnswer.isDefined) {
            if (importOrExportAnswer.get == 1) {
              new ImportExport(ui, controller).importCollapsibleOutlineAsGroups(entityIn)
            }
            else if (importOrExportAnswer.get == 2) {
              new ImportExport(ui, controller).export(entityIn, ImportExport.TEXT_EXPORT_TYPE, None, None, None)
            }
            else if (importOrExportAnswer.get == 3) {
              val (headerContent: String, beginBodyContent: String, footerContent: Option[String]) = getOptionalContentForExportedPages(entityIn)
              if (footerContent.isDefined && footerContent.get.trim.nonEmpty) {
                new ImportExport(ui, controller).export(entityIn, ImportExport.HTML_EXPORT_TYPE, Some(headerContent), Some(beginBodyContent), footerContent)
              }
            }
          }
          otherEntityMenu(entityIn, attributeRowsStartingIndexIn, relationSourceEntityIn, containingRelationToEntityIn, containingGroupIn,
                          attributeTuplesIn)
        } else if (answer == 3) {
          val templateEntity: Option[Entity] =
            if (entityIn.getClassTemplateEntityId.isEmpty) {
              None
            } else {
              Some(new Entity(entityIn.mDB, entityIn.getClassTemplateEntityId.get))
            }
          val templateAttributesToCopy: ArrayBuffer[Attribute] = controller.getMissingAttributes(templateEntity, attributeTuplesIn)
          val editAnswer = ui.askWhich(Some(Array[String]{Util.entityMenuLeadingText(entityIn)}),
                                       Array("Edit entity name",
                                             "Change its class",
                                             if (templateAttributesToCopy.nonEmpty) "Add/edit missing class-defined fields (in other words, to make this " +
                                                                                    "entity more resemble its class' template)" else "(stub)",
                                             if (entityIn.getNewEntriesStickToTop) {
                                               "Set entity so new items added from the top highlight become the *2nd* entry (CURRENTLY: they stay at the top)."
                                             } else {
                                               "Set entity so new items added from the top highlight become the *top* entry (CURRENTLY: they will be 2nd)."
                                             }))
          if (editAnswer.isDefined) {
            if (editAnswer.get == 1) {
              val editedEntity: Option[Entity] = controller.editEntityName(entityIn)
              otherEntityMenu(if (editedEntity.isDefined) editedEntity.get else entityIn, attributeRowsStartingIndexIn, relationSourceEntityIn,
                              containingRelationToEntityIn, containingGroupIn, attributeTuplesIn)
            } else if (editAnswer.get == 2) {
              val classId: Option[Long] = controller.askForClass(entityIn.mDB)
              if (classId.isDefined) {
                entityIn.updateClass(classId)

                // Idea here: when changing the class of an entity, we *could* controller.defaultAttributeCopying (or prompt as elsewhere) to
                // set up the attributes, but the need is unclear, and user can now do that manually from the menus if needed.  Code in future
                // should also be able to use default values from the template entity, as another fallback.
              }
              otherEntityMenu(entityIn, attributeRowsStartingIndexIn, relationSourceEntityIn, containingRelationToEntityIn,
                              containingGroupIn, attributeTuplesIn)
            } else if (editAnswer.get == 3 && templateAttributesToCopy.nonEmpty) {
              controller.copyAndEditAttributes(entityIn, templateAttributesToCopy)
            } else if (editAnswer.get == 4) {
              entityIn.updateNewEntriesStickToTop(!entityIn.getNewEntriesStickToTop)
            }
          }
        } else if (answer == 4) {
          val (delOrArchiveAnswer, delEntityLink_choiceNumber, delFromContainingGroup_choiceNumber, showAllArchivedEntities_choiceNumber) =
            askWhetherDeleteOrArchiveEtc(entityIn, containingRelationToEntityIn, relationSourceEntityIn, containingGroupIn)

          if (delOrArchiveAnswer.isDefined) {
            val delAnswer = delOrArchiveAnswer.get
            if (delAnswer == 1) {
              deleteEntity(entityIn)
            } else if (delAnswer == 2) {
              if (!entityIn.isArchived) {
                archiveEntity(entityIn)
              } else {
                // ** IF THIS menu OPERATION IS EVER MOVED, UPDATE THE USER MESSAGE ABOUT THE MENU OPTIONS LOCATIONS**, in controller.getDefaultEntity. **
                unarchiveEntity(entityIn)
              }
            } else if (delAnswer == delEntityLink_choiceNumber && containingRelationToEntityIn.isDefined && delAnswer <= choices.length) {
              val ans = ui.askYesNoQuestion("DELETE the relation: ARE YOU SURE?", Some(""))
              if (ans.isDefined && ans.get) {
                containingRelationToEntityIn.get.delete()
              } else {
                ui.displayText("Did not delete relation.", waitForKeystrokeIn = false)
              }
            } else if (delAnswer == delFromContainingGroup_choiceNumber && containingGroupIn.isDefined && delAnswer <= choices.length) {
              removeEntityReferenceFromGroup_Menu(entityIn, containingGroupIn)
            } else if (delAnswer == showAllArchivedEntities_choiceNumber) {
              // ** IF THIS OPERATION IS EVER MOVED, UPDATE THE USER MESSAGE ABOUT THE MENU OPTIONS LOCATIONS**, in controller.getDefaultEntity. **
              entityIn.mDB.setIncludeArchivedEntities(! entityIn.mDB.includeArchivedEntities)
            } else {
              ui.displayText("invalid response")
              otherEntityMenu(new Entity(entityIn.mDB, entityIn.getId), attributeRowsStartingIndexIn, relationSourceEntityIn, containingRelationToEntityIn,
                              containingGroupIn, attributeTuplesIn)
            }
          }
        } else if (answer == 5) {
          val templateEntityId: Option[Long] = entityIn.getClassTemplateEntityId
          goToRelatedPlaces(entityIn, relationSourceEntityIn, containingRelationToEntityIn, templateEntityId)
          //ck 1st if entity exists, if not return None. It could have been deleted while navigating around.
          if (entityIn.mDB.entityKeyExists(entityIn.getId, includeArchived = false)) {
            new EntityMenu(ui, controller).entityMenu(entityIn, attributeRowsStartingIndexIn, None, None, containingRelationToEntityIn, containingGroupIn)
          }
        } else if (answer == 7 && answer <= choices.length && !entityIsAlreadyTheDefault && !entityIn.mDB.isRemote) {
          // updates user preferences such that this obj will be the one displayed by default in future.
          entityIn.mDB.setUserPreference_EntityId(Util.DEFAULT_ENTITY_PREFERENCE, entityIn.getId)
          controller.refreshDefaultDisplayEntityId()
        } else {
          ui.displayText("invalid response")
          otherEntityMenu(entityIn, attributeRowsStartingIndexIn, relationSourceEntityIn, containingRelationToEntityIn, containingGroupIn,
                          attributeTuplesIn)
        }
      }
    } catch {
      case e: Exception =>
        Util.handleException(e, ui, entityIn.mDB)
        val ans = ui.askYesNoQuestion("Go back to what you were doing (vs. going out)?", Some("y"))
        if (ans.isDefined && ans.get) {
          otherEntityMenu(entityIn, attributeRowsStartingIndexIn, relationSourceEntityIn, containingRelationToEntityIn,
                          containingGroupIn, attributeTuplesIn)
        }
    }
  }

  def removeEntityReferenceFromGroup_Menu(entityIn: Entity, containingGroupIn: Option[Group]): Boolean = {
    val groupCount: Long = entityIn.getCountOfContainingGroups
    val (entityCountNonArchived, entityCountArchived) = entityIn.getCountOfContainingLocalEntities
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

  /** @return whether entity was deleted.
    */
  def deleteEntity(entityIn: Entity): Boolean = {
    //IDEA: could combine this method with the following two. The only differences as of now are 3 strings and a method call, easily parameterized. Not
    //doing it immediately in case they diverge again soon.
    val name = entityIn.getName
    val groupCount: Long = entityIn.getCountOfContainingGroups
    val affectedExamples = getExampleAffectedGroupsDescriptions(groupCount, entityIn)
    val effectMsg =  "This will ALSO remove it from " + groupCount + " groups, including for example these relations" +
                     " that refer to this entity (showing entities & their relations to groups, as \"entity -> group\"): " + affectedExamples
    // idea: WHEN CONSIDERING MODS TO THIS, ALSO CONSIDER THE Q'S ASKED AT CODE CMT WHERE DELETING A GROUP OF ENTITIES (SEE, for example "recursively").
    // (and in the other 2 methods just like this)
    val warningMsg = "DELETE ENTITY \"" + name + "\" (and " + Util.entityPartsThatCanBeAffected + ").  " + effectMsg + "\n**ARE YOU REALLY SURE?**"
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
    val groupCount: Long = entityIn.getCountOfContainingGroups
    val affectedExamples = getExampleAffectedGroupsDescriptions(groupCount, entityIn)
    val effectMsg = "This will affect affect its visibility in " + groupCount + " groups, including for example these relations" +
                    " that refer to this entity (showing entities & their relations to groups, as \"entity -> group\"): " + affectedExamples
    // idea: WHEN CONSIDERING MODS TO THIS, ALSO CONSIDER THE Q'S ASKED AT CODE CMT WHERE DELETING A GROUP OF ENTITIES (SEE, for example "recursively").
    // (and in the other 2 methods just like this)
    val warningMsg = "ARCHIVE ENTITY \"" + name + "\" (and " + Util.entityPartsThatCanBeAffected + ").  " + effectMsg + "\n**ARE YOU REALLY SURE?**"
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
    val groupCount: Long = entityIn.getCountOfContainingGroups
    val affectedExamples = getExampleAffectedGroupsDescriptions(groupCount, entityIn)
    val effectMsg = "This will affect affect its visibility in " + groupCount + " groups, including for example these relations" +
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

  def getExampleAffectedGroupsDescriptions(groupCount: Long, entityIn: Entity): (String) = {
    if (groupCount == 0) {
      ""
    } else {
      val limit = 10
      val delimiter = ", "
      // (BUG: see comments in psql.java re "OTHER ENTITY NOTED IN A DELETION BUG")
      val descrArray = entityIn.getContainingRelationToGroupDescriptions(Some(limit))
      var descriptions = ""
      var counter = 0
      for (s: String <- descrArray) {
        counter += 1
        descriptions += counter + ") " + s + delimiter
      }
      descriptions.substring(0, math.max(0, descriptions.length - delimiter.length)) + ".  "
    }
  }

  def getOptionalContentForExportedPages(entityIn: Entity): (String, String, Option[String]) = {
    val prompt1 = "Enter lines containing the "
    val prompt2 = " (if any).  "
    val prompt3 = "  (NOTE: to simplify this step in the future, you can add to this entity a single text attribute whose type is an entity named "
    // (Wrote "lines" plural, to clarify when this is presented with the "SINGLE LINE" copyright prompt below.)
    val prompt4 = ", and put the relevant lines of html (or nothing) in the value for that attribute.  Or just press Enter to skip through this each time.)"

    val headerTypeIds: java.util.ArrayList[Long] = entityIn.mDB.findAllEntityIdsByName(Util.HEADER_CONTENT_TAG, caseSensitive = true)
    val bodyContentTypeIds: java.util.ArrayList[Long] = entityIn.mDB.findAllEntityIdsByName(Util.BODY_CONTENT_TAG, caseSensitive = true)
    val footerTypeIds: java.util.ArrayList[Long] = entityIn.mDB.findAllEntityIdsByName(Util.FOOTER_CONTENT_TAG, caseSensitive = true)
    if ((headerTypeIds.size > 1) || (bodyContentTypeIds.size > 1) || (footerTypeIds.size > 1)) {
      throw new OmException("Expected at most one entity (as typeId) each, with the names " + Util.HEADER_CONTENT_TAG + ", " +
                            Util.BODY_CONTENT_TAG + ", or " + Util.FOOTER_CONTENT_TAG + ", but found respectively " +
                            headerTypeIds.size + ", " + bodyContentTypeIds.size + ", and " + headerTypeIds.size + ".  Could change" +
                            " the app to just take the first one found perhaps.... Anyway you'll need to fix in the data, that before proceeding " +
                            "with the export.")

    }

    def getAttrText(entityIn: Entity, typeIdIn: Long): Option[String] = {
      val attrs: java.util.ArrayList[TextAttribute] = entityIn.getTextAttributeByTypeId(typeIdIn)
      if (attrs.size == 0) None
      else if (attrs.size > 1) throw new OmException("The program doesn't know what to do with > 1 textAttributes with this type on the same " +
                                                       "entity, for entity " + entityIn.getId + ", and typeId " + typeIdIn)
      else Some(attrs.get(0).getText)
    }

    // (Idea: combine the next 3 val definitions' code into one method with the "else" part as a parameter, but it should still be clear to most beginner
    // scala programmers.)
    val headerContent: String = {
      val savedAttrText: Option[String] = {
        if (headerTypeIds.size > 0) {
          getAttrText(entityIn, headerTypeIds.get(0))
        } else {
          None
        }
      }
      savedAttrText.getOrElse( {
        ui.displayText(prompt1 + "html page \"<head>\" section contents" + prompt2 +
                       " (Title & 'meta name=\"description\"' tags are automatically filled in from the entity's name.)" +
                       prompt3 + "\"" + Util.HEADER_CONTENT_TAG + "\"" + prompt4, waitForKeystrokeIn = false)
        val s: String = Util.editMultilineText("", ui)
        s
      })
    }
    val beginBodyContent: String = {
      val savedAttrText: Option[String] = {
        if (bodyContentTypeIds.size > 0) {
          getAttrText(entityIn, bodyContentTypeIds.get(0))
        } else {
          None
        }
      }
      savedAttrText.getOrElse({
        ui.displayText(prompt1 + "initial *body* content (like a common banner or header)" + prompt2 +
                       prompt3 + "\"" + Util.BODY_CONTENT_TAG + "\"" + prompt4, waitForKeystrokeIn = false)
        val beginBodyContentIn: String = Util.editMultilineText("", ui)
        beginBodyContentIn
      })
    }
    // (This value is an Option so that if None, it tells the program that the user wants out. The others haven't been set up that way (yet?).)
    val footerContent: Option[String] = {
      val savedAttrText: Option[String] = {
        if (footerTypeIds.size > 0) {
          getAttrText(entityIn, footerTypeIds.get(0))
        } else {
          None
        }
      }
      if (savedAttrText.isEmpty) {
        // idea (in task list):  have the date default to the entity creation date, then later add/replace that (w/ range or what for ranges?)
        // with the last edit date, when that feature exists.
        val copyrightYearAndName = ui.askForString(Some(Array("On a SINGLE LINE, enter copyright year(s) and holder's name, i.e., the \"2015 John Doe\" part " +
                                                              "of \"Copyright 2015 John Doe\" (This accepts HTML so can also be used for a " +
                                                              "page footer, for example.)" +
                                                              prompt3 + "\"" + Util.FOOTER_CONTENT_TAG + "\"" + prompt4)))
        copyrightYearAndName
      } else {
        savedAttrText
      }
    }
    (headerContent, beginBodyContent, footerContent)
  }

  /**
   * @param relationIn  (See comment on "@param relationIn" on method askWhetherDeleteOrArchiveEtc. )
   */
  def goToRelatedPlaces(entityIn: Entity, relationSourceEntityIn: Option[Entity] = None,
                        relationIn: Option[AttributeWithValidAndObservedDates] = None, templateEntityId: Option[Long]) {
    //idea: make this and similar locations share code? What other places could?? There is plenty of duplicated code here!
    require(relationIn.isEmpty || relationIn.get.isInstanceOf[RelationToLocalEntity] || relationIn.get.isInstanceOf[RelationToRemoteEntity])
    val leadingText = Some(Array("Go to..."))
    val seeContainingEntities_choiceNumber: Int = 1
    val seeContainingGroups_choiceNumber: Int = 2
    val goToRelation_choiceNumber: Int = 3
    val goToRelationType_choiceNumber: Int = 4
    // The next 2 values are 3 & 4 in case the previous 2 are unused.  If the previous 2 are used, the next 2 will be += 2, below.
    var goToTemplateEntity_choiceNumber: Int = 3
    var goToClass_choiceNumber: Int = 4
    val numContainingEntities: Long = {
      val (nonArchived, archived) = entityIn.getCountOfContainingLocalEntities
      if (entityIn.mDB.includeArchivedEntities)  nonArchived + archived
      else nonArchived
    }
    // (idea: make this next call efficient: now it builds them all when we just want a count; but is infrequent & likely small numbers)
    val numContainingGroups = entityIn.getCountOfContainingGroups
    var containingGroup: Option[Group] = None
    var containingRtg: Option[RelationToGroup] = None
    if (numContainingGroups == 1) {
      val containingGroupsIds: java.util.ArrayList[Long] = entityIn.getContainingGroupsIds
      // (Next line is just confirming the consistency of logic that got us here: see 'if' just above.)
      require(containingGroupsIds.size == 1)
      containingGroup = Some(new Group(entityIn.mDB, containingGroupsIds.get(0)))

      val containingRtgList: util.ArrayList[RelationToGroup] = entityIn.getContainingRelationsToGroup(0, Some(1))
      if (containingRtgList.size < 1) {
        ui.displayText("There is a group containing the entity (" + entityIn.getName + "), but:  " + Util.ORPHANED_GROUP_MESSAGE)
      } else {
        containingRtg = Some(containingRtgList.get(0))
      }
    }

    var choices = Array[String]("See entities that directly relate to this entity (" + numContainingEntities + ")",
                                if (numContainingGroups == 1) {
                                  "Go to group containing this entity: " + containingGroup.get.getName
                                } else {
                                  "See groups containing this entity (" + numContainingGroups + ")"
                                })
    // (check for existence because other things could have been deleted or archived while browsing around different menu options.)
    if (relationIn.isDefined && relationSourceEntityIn.isDefined && relationSourceEntityIn.get.mDB.entityKeyExists(relationSourceEntityIn.get.getId)) {
      choices = choices :+ "Go edit the relation to entity that led here: " +
                           relationIn.get.getDisplayString(15, Some(entityIn), Some(new RelationType(relationIn.get.mDB, relationIn.get.getAttrTypeId)))
      choices = choices :+ "Go to the type, for the relation that led here: " + new Entity(relationIn.get.mDB, relationIn.get.getAttrTypeId).getName
      goToTemplateEntity_choiceNumber += 2
      goToClass_choiceNumber += 2
    }
    if (templateEntityId.isDefined) {
      choices = choices ++ Array[String]("Go to template entity")
      choices = choices ++ Array[String]("Go to class")
    }
    // (Here for reference, for now. See cmt re one possible usage below. But if ever used, specify local vs. remote?)
    //var relationToEntity: Option[RelationToEntity] = relationIn

    val response = ui.askWhich(leadingText, choices, Array[String]())
    if (response.isDefined) {
      val goWhereAnswer = response.get
      if (goWhereAnswer == seeContainingEntities_choiceNumber && goWhereAnswer <= choices.length) {
        val leadingText = List[String]("Pick from menu, or an entity by letter")
        val choices: Array[String] = Array(Util.listNextItemsPrompt)
        val numDisplayableItems: Long = ui.maxColumnarChoicesToDisplayAfter(leadingText.size, choices.length, Util.maxNameLength)
        // This is partly set up so it could handle multiple screensful, but would need to be broken into a recursive method that
        // can specify dif't values on each call, for the startingIndexIn parm of getRelatingEntities.  I.e., could make it look more like
        // searchForExistingObject or such ? IF needed.  But to be needed means the user is putting the same object related by multiple
        // entities: enough to fill > 1 screen when listed.
        val containingEntities: util.ArrayList[(Long, Entity)] = entityIn.getLocalEntitiesContainingEntity(0, Some(numDisplayableItems))
        val containingEntitiesStatusAndNames: Array[String] = containingEntities.toArray.map {
                                                                                      case relTypeIdAndEntity: (Long, Entity) =>
                                                                                        val entity: Entity = relTypeIdAndEntity._2
                                                                                        entity.getArchivedStatusDisplayString + entity.getName
                                                                                      case _ => throw new OmException("??")
                                                                                    }
        val ans = ui.askWhich(Some(leadingText.toArray), choices, containingEntitiesStatusAndNames)
        if (ans.isDefined) {
          val answer = ans.get
          if (answer == 1 && answer <= choices.length) {
            // see comment above
            ui.displayText("not yet implemented")
          } else if (answer > choices.length && answer <= (choices.length + containingEntities.size)) {
            // those in the condition on the previous line are 1-based, not 0-based.
            val index = answer - choices.length - 1
            // user typed a letter to select.. (now 0-based); selected a new object and so we return to the previous menu w/ that one displayed & current
            val entity: Entity = containingEntities.get(index)._2
            new EntityMenu(ui, controller).entityMenu(entity)
          } else {
            ui.displayText("unknown response")
          }
        }
      } else if (goWhereAnswer == seeContainingGroups_choiceNumber && goWhereAnswer <= choices.length) {
        if (numContainingGroups == 1) {
          require(containingGroup.isDefined)
          new QuickGroupMenu(ui, controller).quickGroupMenu(containingGroup.get, 0, containingRtg, containingEntityIn = None)
        } else {
          viewContainingGroups(entityIn)
        }
      } else if (goWhereAnswer == goToRelation_choiceNumber && relationIn.isDefined && goWhereAnswer <= choices.length) {
        def dummyMethod(inDb: Database, inDH: RelationToEntityDataHolder, inEditing: Boolean, ui: TextUI): Option[RelationToEntityDataHolder] = {
          Some(inDH)
        }
        def updateRelationToEntity(dhInOut: RelationToEntityDataHolder) {
          // This "if" exists only to get things to compile while limiting visibility of "RelationToEntity" (per comments in that class).
          //noinspection TypeCheckCanBeMatch
          if (relationIn.get.isInstanceOf[RelationToLocalEntity]) {
            relationIn.get.asInstanceOf[RelationToLocalEntity].update(dhInOut.validOnDate, Some(dhInOut.observationDate), Some(dhInOut.attrTypeId))
          } else if (relationIn.get.isInstanceOf[RelationToRemoteEntity]) {
            relationIn.get.asInstanceOf[RelationToRemoteEntity].update(dhInOut.validOnDate, Some(dhInOut.observationDate), Some(dhInOut.attrTypeId))
          } else {
            throw new OmException("unexpected type: " + relationIn.getClass.getCanonicalName)
          }
        }
        val relatedId2 = {
          // This "if" exists only to get things to compile while limiting visibility of "RelationToEntity" (per comments on that class).
          //noinspection TypeCheckCanBeMatch
          if (relationIn.get.isInstanceOf[RelationToLocalEntity]) {
            relationIn.get.asInstanceOf[RelationToLocalEntity].getRelatedId2
          } else if (relationIn.get.isInstanceOf[RelationToRemoteEntity]) {
            relationIn.get.asInstanceOf[RelationToRemoteEntity].getRelatedId2
          } else {
            throw new OmException("unexpected type: " + relationIn.getClass.getCanonicalName)
          }
        }
        val relationToEntityDH: RelationToEntityDataHolder = new RelationToEntityDataHolder(relationIn.get.getAttrTypeId, relationIn.get.getValidOnDate,
                                                                                            relationIn.get.getObservationDate, relatedId2,
                                                                                            relationIn.get.mDB.isRemote, relationIn.get.mDB.id)
        controller.askForInfoAndUpdateAttribute[RelationToEntityDataHolder](relationIn.get.mDB, relationToEntityDH, askForAttrTypeId = true,
                                                                            Util.RELATION_TO_LOCAL_ENTITY_TYPE,
                                                                            "CHOOSE TYPE OF Relation to Entity:", dummyMethod, updateRelationToEntity)
        // Force a reread from the DB so it shows the right info SO THIS IS NOT FORGOTTEN, IN CASE we add later a call a menu which
        // needs it as a parameter.  But if ever used, specify local vs. remote.
        //relationToEntity = Some(new RelationToEntity(db, relationIn.get.getId, relationIn.get.getAttrTypeId, relationIn.get.getRelatedId1,
        //                                             relationIn.get.getRelatedId2))
      } else if (goWhereAnswer == goToRelationType_choiceNumber && relationIn.isDefined && goWhereAnswer <= choices.length) {
        new EntityMenu(ui, controller).entityMenu(new Entity(relationIn.get.mDB, relationIn.get.getAttrTypeId))
      } else if (goWhereAnswer == goToTemplateEntity_choiceNumber && templateEntityId.isDefined && goWhereAnswer <= choices.length) {
        new EntityMenu(ui, controller).entityMenu(new Entity(entityIn.mDB, templateEntityId.get))
      } else if (goWhereAnswer == goToClass_choiceNumber && templateEntityId.isDefined && goWhereAnswer <= choices.length) {
        val classId: Option[Long] = entityIn.getClassId
        if (classId.isEmpty) {
          throw new OmException("Unexpectedly, this entity doesn't seem to have a class id.  That is probably a bug.")
        } else {
          new ClassMenu(ui, controller).classMenu(new EntityClass(entityIn.mDB, classId.get))
        }
      } else {
        ui.displayText("invalid response")
      }
    }
  }

  def viewContainingGroups(entityIn: Entity): Option[Entity] = {
    val leadingText = List[String]("Pick from menu, or a letter to (go to if one or) see the entities containing that group, or Alt+<letter> for the actual " +
                                   "*group* by letter")
    val choices: Array[String] = Array(Util.listNextItemsPrompt)
    val numDisplayableItems = ui.maxColumnarChoicesToDisplayAfter(leadingText.size, choices.length, Util.maxNameLength)
    // (see comment in similar location just above where this is called, near "val containingEntities: util.ArrayList"...)
    val containingRelationToGroups: util.ArrayList[RelationToGroup] = entityIn.getContainingRelationsToGroup(0, Some(numDisplayableItems))
    val containingRtgDescriptions: Array[String] = containingRelationToGroups.toArray.map {
                                                                                            case rtg: (RelationToGroup) =>
                                                                                              val entityName: String = new Entity(rtg.mDB,
                                                                                                                                  rtg.getParentId).getName
                                                                                              val rt: RelationType = new RelationType(rtg.mDB,
                                                                                                                                      rtg.getAttrTypeId)
                                                                                              "entity " + entityName + " " +
                                                                                              rtg.getDisplayString(Util.maxNameLength, None, Some(rt))
                                                                                            case _ => throw new OmException("??")
                                                                                          }
    val ans = ui.askWhichChoiceOrItsAlternate(Some(leadingText.toArray), choices, containingRtgDescriptions)
    if (ans.isEmpty) None
    else {
      val (answer, userPressedAltKey: Boolean) = ans.get
      // those in the condition on the previous line are 1-based, not 0-based.
      val index = answer - choices.length - 1
      if (answer == 1 && answer <= choices.length) {
        // see comment above
        ui.displayText("not yet implemented")
        None
      } else if (answer > choices.length && answer <= (choices.length + containingRelationToGroups.size) && !userPressedAltKey) {
        // This displays (or allows to choose) the entity that contains the group, rather than the chosen group itself.  Probably did it that way originally
        // because I thought it made more sense to show a group in context than by itself.
        val containingRelationToGroup = containingRelationToGroups.get(index)
        val containingEntities = containingRelationToGroup.mDB.getEntitiesContainingGroup(containingRelationToGroup.getGroupId, 0)
        val numContainingEntities = containingEntities.size
        if (numContainingEntities == 1) {
          val containingEntity: Entity = containingEntities.get(0)._2
          new EntityMenu(ui, controller).entityMenu(containingEntity, containingGroupIn = Some(new Group(containingRelationToGroup.mDB,
                                                                                                         containingRelationToGroup.getGroupId)))
        } else {
          controller.chooseAmongEntities(containingEntities)
        }
      } else if (answer > choices.length && answer <= (choices.length + containingRelationToGroups.size) && userPressedAltKey) {
        // user typed a letter to select.. (now 0-based); selected a new object and so we return to the previous menu w/ that one displayed & current
        val id: Long = containingRelationToGroups.get(index).getId
        val entityId: Long = containingRelationToGroups.get(index).getParentId
        val groupId: Long = containingRelationToGroups.get(index).getGroupId
        val relTypeId: Long = containingRelationToGroups.get(index).getAttrTypeId
        new QuickGroupMenu(ui, controller).quickGroupMenu(new Group(entityIn.mDB, groupId), 0,
                                                          Some(new RelationToGroup(entityIn.mDB, id, entityId, relTypeId, groupId)),
                                                          Some(entityIn), containingEntityIn = None)
      } else {
        ui.displayText("unknown response")
        None
      }
    }
  }

  /**
   * @param relationIn Is of type Option[AttributeWithValidAndObservedDates], because it needs to be a RelationToLocalEntity (RTLE) *or* a
   *                   RelationToRemoteEntity, and I don't know how to specify that except by the lowest available parent and the
   *                   following require statement.  I don't want to use RelationToEntity, because specifying local vs. remote helps keep the code unambiguous
   *                   as to whether things need special remote logic, ie if the code is forced to choose either RTLE or RTRE.  Same reasons elsewhere.
   * @return None means "get out", or Some(choiceNum) if a choice was made.
   */
  def askWhetherDeleteOrArchiveEtc(entityIn: Entity, relationIn: Option[AttributeWithValidAndObservedDates], relationSourceEntityIn: Option[Entity],
                                   containingGroupIn: Option[Group]): (Option[Int], Int, Int, Int) = {
    require(relationIn.isEmpty || relationIn.get.isInstanceOf[RelationToLocalEntity] || relationIn.get.isInstanceOf[RelationToRemoteEntity])

    val groupCount: Long = entityIn.getCountOfContainingGroups
    val (entityCountNonArchived, entityCountArchived) = entityIn.getCountOfContainingLocalEntities
    val relToGroupCnt = entityIn.getRelationToGroupCount
    val relToLocalEntityCnt = entityIn.getRelationToLocalEntityCount(true)
    val relToLocalEntityCntNotArchived = entityIn.getRelationToLocalEntityCount(false)
    val relToLocalEntityCntArchived = relToLocalEntityCnt - relToLocalEntityCntNotArchived 
    val relToRemoteEntityCnt = entityIn.getRelationToRemoteEntityCount
    val totalNumOfAttributes = entityIn.getAttributeCount(true)
    val adjNumOfAttributes = (totalNumOfAttributes - relToGroupCnt) - relToLocalEntityCnt
    //(Idea: the next line/block could use thorough tests, incl of the "remote" part)
    val leadingText = Some(Array(("Choose a deletion or archiving option:  " + Util.NEWLN +
      (if (entityCountNonArchived != 0 || entityCountArchived != 0) {
        "  The entity is " + Util.getContainingEntitiesDescription(entityCountNonArchived, entityCountArchived) + "." + Util.NEWLN
      } else "") 
      + 
      (if (groupCount != 0) {
        "  The entity is contained in " + groupCount + " group(s)." + Util.NEWLN
      } else "")
      +
      (if (relToLocalEntityCnt != 0 || relToLocalEntityCntArchived != 0 || relToRemoteEntityCnt != 0 
        || relToGroupCnt != 0 || adjNumOfAttributes != 0) 
        {
          var directContains = "The entity directly contains: " + Util.NEWLN +
          (if (relToLocalEntityCnt != 0) {
            "    " + relToLocalEntityCnt + " local entity(ies)" +
            (if (relToLocalEntityCntArchived != 0) {
              " (" + relToLocalEntityCntArchived + " of them archived)"
            } else "") + Util.NEWLN
          } else "") +
          (if (relToRemoteEntityCnt != 0) {
            //(Idea: similar places might also mention remote entities..?)
            relToRemoteEntityCnt + "    remote entity(ies) (incl. archived), " + Util.NEWLN
          } else "") +
          (if (relToGroupCnt != 0) {
            "    " + relToGroupCnt + " group(s)" + Util.NEWLN
          } else "") + 
          (if (adjNumOfAttributes != 0) {
            "    " + adjNumOfAttributes + " other attribute(s)" + Util.NEWLN
          } else "")

          "  " + directContains.trim + "." + Util.NEWLN
          //directContains
        } else "")
      ).trim 
    ))

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
    if (relationIn.isDefined && relationSourceEntityIn.isDefined && relationSourceEntityIn.get.mDB.entityKeyExists(relationSourceEntityIn.get.getId)) {
      // means we got here by selecting a Relation attribute on another entity, so entityIn is the "entityId2" in that relation; so show some options,
      // because
      // we eliminated a separate menu just for the relation and put them here, for UI usage simplicity.
      choices = choices :+ "Delete the link from the linking (or containing) entity:" + Util.NEWLN +
                           "    \"" + relationSourceEntityIn.get.getName + "\", " + Util.NEWLN +
                           "  ...to this one:" + Util.NEWLN +
                           "    \"" + entityIn.getName + "\""
      delFromContainingGroup_choiceNumber += 1
      showAllArchivedEntities_choiceNumber += 1
    }
    if (containingGroupIn.isDefined) {
      choices = choices :+ "Delete the link from the containing group:" + Util.NEWLN +
                           "    \"" + containingGroupIn.get.getName + "\"," + Util.NEWLN +
                           "  ...to this Entity:" + Util.NEWLN +
                           "    \"" + entityIn.getName + "\""
      showAllArchivedEntities_choiceNumber += 1
    }
    choices = choices :+ (if (!entityIn.mDB.includeArchivedEntities) "Show archived entities" else "Do not show archived entities")

    val delOrArchiveAnswer: Option[(Int)] = ui.askWhich(leadingText, choices, Array[String]())
    (delOrArchiveAnswer, delEntityLink_choiceNumber, delFromContainingGroup_choiceNumber, showAllArchivedEntities_choiceNumber)
  }

}
