/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2015-2016 inclusive, Luke A. Call; all rights reserved.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule, guidelines around binary
    distribution, and the GNU Affero General Public License as published by the Free Software Foundation, either version 3
    of the License, or (at your option) any later version.  See the file LICENSE for details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
package org.onemodel.core.controllers

import java.util

import org.onemodel.core._
import org.onemodel.core.{OmException, TextUI}
import org.onemodel.core.database.Database
import org.onemodel.core.model._

import scala.collection.mutable.ArrayBuffer

/** This is simply to hold less-used operations so the main EntityMenu can be the most-used stuff.
  */
class OtherEntityMenu (val ui: TextUI, val db: Database, val controller: Controller) {

  def otherEntityMenu(entityIn: Entity, attributeRowsStartingIndexIn: Int = 0, relationSourceEntityIn: Option[Entity],
                      containingRelationToEntityIn: Option[RelationToEntity], containingGroupIn: Option[Group],
                      attributeTuplesIn: Array[(Long, Attribute)]) {
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
        choices = choices :+ ((if (defaultEntity.isEmpty) "****TRY ME---> " else "") +
                              "Set current entity as default (first to come up when launching this program.)")
      } else choices = choices :+ "(stub)"

      val response = ui.askWhich(Some(leadingText), choices)
      if (response.isDefined) {
        val answer = response.get
        if (answer == 1) {
          val valueBeforeEntry: Option[Boolean] = entityIn.getPublic
          val valueAfterEntry: Option[Boolean] = controller.askForPublicNonpublicStatus(valueBeforeEntry)
          val rteCount: Long = db.getRelationToEntityCount(entityIn.getId, includeArchivedEntities = false)
          val rtgCount: Long = db.getRelationToGroupCount(entityIn.getId)
          val publicMenuResponse = ui.askWhich(None, Array("...for this entity (\"" + entityIn.getName + "\")",
                                                           "...for its " + rteCount + " contained entities (one level), and all the" +
                                                           " entities contained in its " + rtgCount + " groups (one level)",
                                                           "...for both."))
          if (publicMenuResponse.isDefined) {
            if (publicMenuResponse.get == 1) {
              updatePublicStatus(entityIn.getId, valueBeforeEntry, valueAfterEntry)
            } else if (publicMenuResponse.get == 2) {
              val count: Int = updateContainedEntitiesPublicStatus(entityIn.getId, valueAfterEntry)
              ui.displayText("Updated " + count + " contained entities with new status.")
            } else if (publicMenuResponse.get == 3) {
              updatePublicStatus(entityIn.getId, valueBeforeEntry, valueAfterEntry)
              val count: Int = updateContainedEntitiesPublicStatus(entityIn.getId, valueAfterEntry)
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
            if (importOrExportAnswer.get == 1) new ImportExport(ui, db, controller).importCollapsibleOutlineAsGroups(entityIn)
            else if (importOrExportAnswer.get == 2) new ImportExport(ui, db, controller).export(entityIn, ImportExport.TEXT_EXPORT_TYPE, None, None, None)
            else if (importOrExportAnswer.get == 3) {
              val (headerContent: String, beginBodyContent: String, footerContent: Option[String]) = getOptionalContentForExportedPages(entityIn)
              if (footerContent.isDefined && footerContent.get.trim.nonEmpty) {
                new ImportExport(ui, db, controller).export(entityIn, ImportExport.HTML_EXPORT_TYPE, Some(headerContent), Some(beginBodyContent), footerContent)
              }
            }
          }
          otherEntityMenu(entityIn, attributeRowsStartingIndexIn, relationSourceEntityIn, containingRelationToEntityIn, containingGroupIn,
                          attributeTuplesIn)
        } else if (answer == 3) {
          val templateEntityId: Option[Long] = entityIn.getClassTemplateEntityId
          val templateAttributesToCopy: ArrayBuffer[Attribute] = controller.getMissingAttributes(templateEntityId, attributeTuplesIn)
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
              val classId: Option[Long] = controller.askForClass()
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
            controller.askWhetherDeleteOrArchiveEtc(entityIn, containingRelationToEntityIn, relationSourceEntityIn, containingGroupIn)

          if (delOrArchiveAnswer.isDefined) {
            val delAnswer = delOrArchiveAnswer.get
            if (delAnswer == 1) {
              controller.deleteEntity(entityIn)
            } else if (delAnswer == 2) {
              if (!entityIn.isArchived) {
                controller.archiveEntity(entityIn)
              } else {
                // ** IF THIS OPERATION IS EVER MOVED, UPDATE THE USER MESSAGE ABOUT THE MENU OPTIONS LOCATIONS**, in controller.getDefaultEntity. **
                controller.unarchiveEntity(entityIn)
              }
            } else if (delAnswer == delEntityLink_choiceNumber && containingRelationToEntityIn.isDefined && delAnswer <= choices.length) {
              val ans = ui.askYesNoQuestion("DELETE the relation: ARE YOU SURE?", Some(""))
              if (ans.isDefined && ans.get) {
                containingRelationToEntityIn.get.delete()
              } else {
                ui.displayText("Did not delete relation.", waitForKeystrokeIn = false)
              }
            } else if (delAnswer == delFromContainingGroup_choiceNumber && containingGroupIn.isDefined && delAnswer <= choices.length) {
              controller.removeEntityReferenceFromGroup_Menu(entityIn, containingGroupIn)
            } else if (delAnswer == showAllArchivedEntities_choiceNumber) {
              // ** IF THIS OPERATION IS EVER MOVED, UPDATE THE USER MESSAGE ABOUT THE MENU OPTIONS LOCATIONS**, in controller.getDefaultEntity. **
              db.setIncludeArchivedEntities(! db.includeArchivedEntities)
            } else {
              ui.displayText("invalid response")
              otherEntityMenu(new Entity(db, entityIn.getId), attributeRowsStartingIndexIn, relationSourceEntityIn, containingRelationToEntityIn,
                              containingGroupIn, attributeTuplesIn)
            }
          }
        } else if (answer == 5) {
          val templateEntityId: Option[Long] = entityIn.getClassTemplateEntityId
          goToRelatedPlaces(entityIn, relationSourceEntityIn, containingRelationToEntityIn, templateEntityId)
          //ck 1st if entity exists, if not return None. It could have been deleted while navigating around.
          if (db.entityKeyExists(entityIn.getId, includeArchived = false)) {
            new EntityMenu(ui, db, controller).entityMenu(entityIn, attributeRowsStartingIndexIn, None, None, containingRelationToEntityIn, containingGroupIn)
          }
        } else if (answer == 7 && answer <= choices.length && !entityIsAlreadyTheDefault) {
          // updates user preferences such that this obj will be the one displayed by default in future.
          db.setUserPreference_EntityId(Util.DEFAULT_ENTITY_PREFERENCE, entityIn.getId)
          controller.refreshDefaultDisplayEntityId()
        } else {
          ui.displayText("invalid response")
          otherEntityMenu(entityIn, attributeRowsStartingIndexIn, relationSourceEntityIn, containingRelationToEntityIn, containingGroupIn,
                          attributeTuplesIn)
        }
      }
    } catch {
      case e: Exception =>
        Util.handleException(e, controller.ui, controller.db)
        val ans = ui.askYesNoQuestion("Go back to what you were doing (vs. going out)?", Some("y"))
        if (ans.isDefined && ans.get) {
          otherEntityMenu(entityIn, attributeRowsStartingIndexIn, relationSourceEntityIn, containingRelationToEntityIn,
                          containingGroupIn, attributeTuplesIn)
        }
    }
  }

  def updatePublicStatus(entityIdIn: Long, oldValueIn: Option[Boolean], newValueIn: Option[Boolean]): Unit = {
    // The condition for this (when it was part of EntityMenu) used to include " && !entityIn.isInstanceOf[RelationType]", but maybe it's better
    // w/o that.
    if (newValueIn != oldValueIn) {
      db.updateEntityOnlyPublicStatus(entityIdIn, newValueIn)
    }
  }

  def updateContainedEntitiesPublicStatus(entityIdIn: Long, newValueIn: Option[Boolean]): Int = {
    val (attrTuples: Array[(Long, Attribute)], _) = db.getSortedAttributes(entityIdIn, 0, 0, onlyPublicEntitiesIn = false)
    var count = 0
    for (attr <- attrTuples) {
      attr._2 match {
        case attribute: RelationToEntity =>
          require(attribute.getRelatedId1 == entityIdIn, "Unexpected value: " + attribute.getRelatedId1)
          val e: Entity = new Entity(Util.currentOrRemoteDb(attribute, db), attribute.getRelatedId2)
          e.updatePublicStatus(newValueIn)
          count += 1
        case attribute: RelationToGroup =>
          val groupId: Long = attribute.getGroupId
          val entries: List[Array[Option[Any]]] = db.getGroupEntriesData(groupId, None, includeArchivedEntitiesIn = false)
          for (entry <- entries) {
            val entityId = entry(0).get.asInstanceOf[Long]
            db.updateEntityOnlyPublicStatus(entityId, newValueIn)
            count += 1
          }
        case _ =>
        // do nothing
      }
    }
    count
  }

  def getOptionalContentForExportedPages(entityIn: Entity): (String, String, Option[String]) = {
    val prompt1 = "Enter lines containing the "
    val prompt2 = " (if any).  "
    val prompt3 = "  (NOTE: to simplify this step in the future, you can add to this entity a single text attribute whose type is an entity named "
    // (Wrote "lines" plural, to clarify when this is presented with the "SINGLE LINE" copyright prompt below.)
    val prompt4 = ", and put the relevant lines of html (or nothing) in the value for that attribute.  Or just press Enter to skip through this each time.)"

    val headerTypeIds: java.util.ArrayList[Long] = db.findAllEntityIdsByName(Util.HEADER_CONTENT_TAG, caseSensitive = true)
    val bodyContentTypeIds: java.util.ArrayList[Long] = db.findAllEntityIdsByName(Util.BODY_CONTENT_TAG, caseSensitive = true)
    val footerTypeIds: java.util.ArrayList[Long] = db.findAllEntityIdsByName(Util.FOOTER_CONTENT_TAG, caseSensitive = true)
    if ((headerTypeIds.size > 1) || (bodyContentTypeIds.size > 1) || (footerTypeIds.size > 1)) {
      throw new OmException("Expected at most one entity (as typeId) each, with the names " + Util.HEADER_CONTENT_TAG + ", " +
                            Util.BODY_CONTENT_TAG + ", or " + Util.FOOTER_CONTENT_TAG + ", but found respectively " +
                            headerTypeIds.size + ", " + bodyContentTypeIds.size + ", and " + headerTypeIds.size + ".  Could change" +
                            " the app to just take the first one found perhaps.... Anyway you'll need to fix in the data, that before proceeding " +
                            "with the export.")

    }

    def getAttrText(entityIdIn: Long, typeIdIn: Long): Option[String] = {
      val attrs: java.util.ArrayList[TextAttribute] = db.getTextAttributeByTypeId(entityIdIn, typeIdIn)
      if (attrs.size == 0) None
      else if (attrs.size > 1) throw new OmException("The program doesn't know what to do with > 1 textAttributes with this type on the same " +
                                                       "entity, for entity " + entityIdIn + ", and typeId " + typeIdIn)
      else Some(attrs.get(0).getText)
    }

    // (Idea: combine the next 3 val definitions' code into one method with the "else" part as a parameter, but it should still be clear to most beginner
    // scala programmers.)
    val headerContent: String = {
      val savedAttrText: Option[String] = {
        if (headerTypeIds.size > 0) {
          getAttrText(entityIn.getId, headerTypeIds.get(0))
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
          getAttrText(entityIn.getId, bodyContentTypeIds.get(0))
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
        if (headerTypeIds.size > 0) {
          getAttrText(entityIn.getId, footerTypeIds.get(0))
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

  def goToRelatedPlaces(entityIn: Entity, relationSourceEntityIn: Option[Entity] = None,
                        relationIn: Option[RelationToEntity] = None, templateEntityId: Option[Long]) {
    //idea: make this and similar locations share code? What other places could?? There is plenty of duplicated code here!
    val leadingText = Some(Array("Go to..."))
    val seeContainingEntities_choiceNumber: Int = 1
    val seeContainingGroups_choiceNumber: Int = 2
    val goToRelation_choiceNumber: Int = 3
    val goToRelationType_choiceNumber: Int = 4
    // The next 2 values are 3 & 4 in case the previous 2 are unused.  If the previous 2 are used, the next 2 will be += 2, below.
    var goToTemplateEntity_choiceNumber: Int = 3
    var goToClass_choiceNumber: Int = 4
    val numContainingEntities = db.getEntitiesContainingEntity(entityIn.getId, 0).size
    // (idea: make this next call efficient: now it builds them all when we just want a count; but is infrequent & likely small numbers)
    val numContainingGroups = db.getCountOfGroupsContainingEntity(entityIn.getId)
    var containingGroup: Option[Group] = None
    var containingRtg: Option[RelationToGroup] = None
    if (numContainingGroups == 1) {
      val containingGroupsIds: java.util.ArrayList[Long] = db.getContainingGroupsIds(entityIn.getId)
      // (Next line is just confirming the consistency of logic that got us here: see 'if' just above.)
      require(containingGroupsIds.size == 1)
      containingGroup = Some(new Group(db, containingGroupsIds.get(0)))

      val containingRtgList: util.ArrayList[RelationToGroup] = db.getContainingRelationToGroups(entityIn.getId, 0, Some(1))
      if (containingRtgList.size < 1) {
        ui.displayText("There is a group containing the entity (" + entityIn.getName + "), but:  " + Util.ORPHANED_GROUP_MESSAGE)
      } else {
        containingRtg = Some(containingRtgList.get(0))
      }
    }

    var choices = Array[String]("See entities that directly relate to this entity ( " + numContainingEntities + ")",
                                if (numContainingGroups == 1) {
                                  "Go to group containing this entity: " + containingGroup.get.getName
                                } else {
                                  "See groups containing this entity (" + numContainingGroups + ")"
                                })
    // (check for existence because other things could have been deleted or archived while browsing around different menu options.)
    if (relationIn.isDefined && relationSourceEntityIn.isDefined && db.entityKeyExists(relationSourceEntityIn.get.getId)) {
      choices = choices :+ "Go edit the relation to entity that led here: " +
                           relationIn.get.getDisplayString(15, Some(entityIn), Some(new RelationType(db, relationIn.get.getAttrTypeId)))
      choices = choices :+ "Go to the type, for the relation that led here: " + new Entity(db, relationIn.get.getAttrTypeId).getName
      goToTemplateEntity_choiceNumber += 2
      goToClass_choiceNumber += 2
    }
    if (templateEntityId.isDefined) {
      choices = choices ++ Array[String]("Go to template entity")
      choices = choices ++ Array[String]("Go to class")
    }
    // (Here for reference, for now. See cmt re one possible usage below.)
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
        val containingEntities: util.ArrayList[(Long, Entity)] = db.getEntitiesContainingEntity(entityIn.getId, 0, Some(numDisplayableItems))
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
            new EntityMenu(ui, db, controller).entityMenu(entity)
          } else {
            ui.displayText("unknown response")
          }
        }
      } else if (goWhereAnswer == seeContainingGroups_choiceNumber && goWhereAnswer <= choices.length) {
        if (numContainingGroups == 1) {
          require(containingGroup.isDefined)
          new QuickGroupMenu(ui, db, controller).quickGroupMenu(containingGroup.get, 0, containingRtg, containingEntityIn = None)
        } else {
          viewContainingGroups(entityIn)
        }
      } else if (goWhereAnswer == goToRelation_choiceNumber && relationIn.isDefined && goWhereAnswer <= choices.length) {
        def dummyMethod(inDH: RelationToEntityDataHolder, inEditing: Boolean, ui: TextUI): Option[RelationToEntityDataHolder] = {
          Some(inDH)
        }
        def updateRelationToEntity(dhInOut: RelationToEntityDataHolder) {
          relationIn.get.update(relationIn.get.getAttrTypeId, dhInOut.validOnDate, Some(dhInOut.observationDate), Some(dhInOut.attrTypeId))
        }
        val relationToEntityDH: RelationToEntityDataHolder = new RelationToEntityDataHolder(relationIn.get.getAttrTypeId, relationIn.get.getValidOnDate,
                                                                                            relationIn.get.getObservationDate, relationIn.get.getRelatedId2,
                                                                                            relationIn.get.isRemote, relationIn.get.omInstanceKey)
        controller.askForInfoAndUpdateAttribute[RelationToEntityDataHolder](relationToEntityDH, askForAttrTypeId = true, Util.RELATION_TO_ENTITY_TYPE,
                                                                            "CHOOSE TYPE OF Relation to Entity:", dummyMethod, updateRelationToEntity)
        // Force a reread from the DB so it shows the right info SO THIS IS NOT FORGOTTEN, IN CASE we add later a call a menu which
        // needs it as a parameter.
        //relationToEntity = Some(new RelationToEntity(db, relationIn.get.getId, relationIn.get.getAttrTypeId, relationIn.get.getRelatedId1,
        //                                             relationIn.get.getRelatedId2))
      } else if (goWhereAnswer == goToRelationType_choiceNumber && relationIn.isDefined && goWhereAnswer <= choices.length) {
        new EntityMenu(ui, db, controller).entityMenu(new Entity(db, relationIn.get.getAttrTypeId))
      } else if (goWhereAnswer == goToTemplateEntity_choiceNumber && templateEntityId.isDefined && goWhereAnswer <= choices.length) {
        new EntityMenu(ui, db, controller).entityMenu(new Entity(db, templateEntityId.get))
      } else if (goWhereAnswer == goToClass_choiceNumber && templateEntityId.isDefined && goWhereAnswer <= choices.length) {
        val classId: Option[Long] = entityIn.getClassId
        if (classId.isEmpty) {
          throw new OmException("Unexpectedly, this entity doesn't seem to have a class id.  That is probably a bug.")
        } else {
          new ClassMenu(ui, db, controller).classMenu(new EntityClass(db, classId.get))
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
    val containingRelationToGroups: util.ArrayList[RelationToGroup] = db.getContainingRelationToGroups(entityIn.getId, 0, Some(numDisplayableItems))
    val containingRtgDescriptions: Array[String] = containingRelationToGroups.toArray.map {
                                                                                            case rtg: (RelationToGroup) =>
                                                                                              val entityName: String = new Entity(db, rtg.getParentId).getName
                                                                                              val rt: RelationType = new RelationType(db, rtg.getAttrTypeId)
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
        val containingEntities = db.getEntitiesContainingGroup(containingRelationToGroup.getGroupId, 0)
        val numContainingEntities = containingEntities.size
        if (numContainingEntities == 1) {
          val containingEntity: Entity = containingEntities.get(0)._2
          new EntityMenu(ui, db, controller).entityMenu(containingEntity, containingGroupIn = Some(new Group(db, containingRelationToGroup.getGroupId)))
        } else {
          controller.chooseAmongEntities(containingEntities)
        }
      } else if (answer > choices.length && answer <= (choices.length + containingRelationToGroups.size) && userPressedAltKey) {
        // user typed a letter to select.. (now 0-based); selected a new object and so we return to the previous menu w/ that one displayed & current
        val id: Long = containingRelationToGroups.get(index).getId
        val entityId: Long = containingRelationToGroups.get(index).getParentId
        val groupId: Long = containingRelationToGroups.get(index).getGroupId
        val relTypeId: Long = containingRelationToGroups.get(index).getAttrTypeId
        new QuickGroupMenu(ui, db, controller).quickGroupMenu(new Group(db, groupId), 0, Some(new RelationToGroup(db, id, entityId, relTypeId, groupId)),
                                                              Some(entityIn), containingEntityIn = None)
      } else {
        ui.displayText("unknown response")
        None
      }
    }
  }

}
