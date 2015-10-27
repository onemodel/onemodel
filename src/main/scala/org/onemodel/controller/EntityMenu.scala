/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003-2004 and 2008-2015 inclusive, Luke A. Call; all rights reserved.
    (That copyright statement was previously 2013-2015, until I remembered that much of Controller came from TextUI.scala and TextUI.java before that.)
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule, guidelines around binary
    distribution, and the GNU Affero General Public License as published by the Free Software Foundation, either version 3
    of the License, or (at your option) any later version.  See the file LICENSE for details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
package org.onemodel.controller

import java.io.File
import java.util

import org.onemodel._
import org.onemodel.database.PostgreSQLDatabase
import org.onemodel.model._

class EntityMenu(override val ui: TextUI, override val db: PostgreSQLDatabase, val controller: Controller) extends SortableEntriesMenu(ui, db) {
  // 2nd return value is whether entityIsDefault (ie whether default object when launching OM is already this entity)
  def getChoices(entityIn: Entity, numAttrsIn: Long, relationSourceEntityIn: Option[Entity] = None,
                 relationIn: Option[RelationToEntity] = None): (Array[String], Boolean) = {
    // (idea: might be a little silly to do it this way, once this # gets very big?:)
    var choices = Array[String]("Add attribute (quantity, true/false, date, text, external file, relation to entity or group: " + controller.mRelTypeExamples + ")...",
                                if (numAttrsIn > 1) "Move selection (*) up/down" else "(stub)",

                                "[app will fill this one in just a bit later, at \"choices (2) = \" below.  KEEP IN THIS RELATIVE POSITION OR CHANGE THE" +
                                " CODE NEAR THE TOP OF entityMenu THAT CHECKS FOR A VALUE IN highlightedAttributeIn]",

                                "Delete or Archive this entity...",
                                "Go to other related entities or groups...",
                                if (numAttrsIn > 0) controller.listNextItemsPrompt else "(stub)")
    if (relationIn.isDefined) {
      // means we got here by selecting a Relation attribute on another entity, so entityIn is the "entityId2" in that relation; so show some options, because
      // we eliminated a separate menu just for the relation and put them here, for UI usage simplicity.
      require(relationIn.get.getRelatedId2 == entityIn.getId && relationSourceEntityIn.isDefined)
    }

    val defaultEntity: Option[Long] = controller.findDefaultDisplayEntity
    //  don't show the "set default" option if it's already been done w/ this same one:
    val entityIsAlreadyTheDefault: Boolean = defaultEntity.isDefined && defaultEntity.get == entityIn.getId
    if (! entityIsAlreadyTheDefault) {
      choices = choices :+ ((if (defaultEntity.isEmpty) "****TRY ME---> " else "") +
                            "Set current entity as default (first to come up when launching this program.)")
    } else choices = choices :+ "(stub)"
    choices = choices :+ (if (numAttrsIn > 0) "Select attribute to highlight (with '*'; typing the letter instead goes to that attribute's menu)" else "(stub)")
    choices = choices :+ "Other entity operations..."
    (choices, entityIsAlreadyTheDefault)
  }

  /** The parameter attributeRowsStartingIndexIn means: of all the sorted attributes of entityIn, which one is to be displayed first (since we can only display
    * so many at a time with finite screen size).
   * Returns None if user wants out.
   * */
  //@tailrec //removed for now until the compiler can handle it with where the method calls itself.
  //idea on scoping: make this limited like this somehow?:  private[org.onemodel] ... Same for all others like it?
  def entityMenu(entityIn: Entity, attributeRowsStartingIndexIn: Int = 0, highlightedAttributeIn: Option[Attribute] = None,
                 relationSourceEntityIn: Option[Entity] = None,
                 relationIn: Option[RelationToEntity] = None, containingGroupIn: Option[Group] = None): Option[Entity] = try {
    require(entityIn != null)
    val numAttrsInEntity: Long = entityIn.getAttrCount
    val classDefiningEntityId: Option[Long] = entityIn.getClassDefiningEntityId
    val leadingText: Array[String] = new Array[String](2)
    val numAttrs = db.getAttrCount(entityIn.getId)
    val (choices: Array[String], entityIsAlreadyTheDefault: Boolean) = getChoices(entityIn, numAttrs, relationSourceEntityIn, relationIn)
    val numDisplayableAttributes: Int = ui.maxColumnarChoicesToDisplayAfter(leadingText.length, choices.length, controller.maxNameLength)
    val (attributeTuples: Array[(Long, Attribute)], totalAttrsAvailable: Int) =
      db.getSortedAttributes(entityIn.getId, attributeRowsStartingIndexIn, numDisplayableAttributes)
    if (numAttrs > 0 || attributeTuples.length > 0) require(numAttrs > 0 && attributeTuples.length > 0)
    require(totalAttrsAvailable == numAttrs)
    val choicesModified = controller.addRemainingCountToPrompt(choices, attributeTuples.length, totalAttrsAvailable, attributeRowsStartingIndexIn)
    val leadingTextModified = getLeadingText(leadingText, attributeTuples.length, entityIn, relationSourceEntityIn, relationIn, containingGroupIn)
    val attributeDisplayStrings: Array[String] = getItemDisplayStrings(attributeTuples)

    val highlightedEntry: Option[Attribute] = if (attributeTuples.length == 0) None else Some(highlightedAttributeIn.getOrElse(attributeTuples(0)._2))
    choices(2) =
      // MAKE SURE this condition always matches the one in the edit handler below:
      if (highlightedEntry.isDefined && controller.canEditAttributeOnSingleLine(highlightedEntry.get)) {
        "Edit the selected attribute's content (single line; go into the attribute's menu for more options)"
      } else "Edit entity name"



    // The variable highlightedIndexInObjList means: of the sorted attributes selected *for display* (potentially fewer than all existing attributes),
    // this is the zero-based index of the one that is marked for possible moving around in the sorted order (in the UI, marked as selected).
    def getHighlightedIndexInAttrList: Option[Int] = {
      if (highlightedEntry.isEmpty) None
      else {
        val highlightedObjFormId: Int = highlightedEntry.get.getFormId
        val highlightedObjId: Long = highlightedEntry.get.getId
        var index = -1
        for (attributeTuple <- attributeTuples) {
          index += 1
          val attribute = attributeTuple._2
          if (attribute.getFormId == highlightedObjFormId && attribute.getId == highlightedObjId) {
            return Some(index)
          }
        }
        // if we got to this point, it could simply have been deleted or something (probably), so just return something safe (instead of throwing an
        // exception, as in a previous commit):
        None
      }
    }
    val highlightedIndexInObjList: Option[Int] = getHighlightedIndexInAttrList


    val response = ui.askWhich(Some(leadingTextModified), choicesModified, attributeDisplayStrings, highlightIndexIn = getHighlightedIndexInAttrList)
    if (response.isEmpty) None
    else {
      val answer = response.get
      if (answer == 1) {
        val newAttribute: Option[Attribute] = addAttribute(entityIn, attributeRowsStartingIndexIn, highlightedEntry, relationSourceEntityIn,
                                                           relationIn, containingGroupIn)
        if (newAttribute.isDefined && highlightedEntry.isDefined) {
          placeEntryInPosition(entityIn.getId, entityIn.getAttrCount, 0, forwardNotBackIn = true,
                                                                   attributeRowsStartingIndexIn, newAttribute.get.getId, highlightedIndexInObjList.get,
                                                                   highlightedEntry.get.getId, numDisplayableAttributes, newAttribute.get.getFormId,
                                                                   highlightedEntry.get.getFormId)
        }
        entityMenu(entityIn, attributeRowsStartingIndexIn, newAttribute, relationSourceEntityIn, relationIn, containingGroupIn)
      } else if (answer == 2 && highlightedEntry.isDefined && highlightedIndexInObjList.isDefined && numAttrs > 1) {
        val newStartingDisplayIndex = moveSelectedEntry(entityIn, attributeRowsStartingIndexIn, totalAttrsAvailable, highlightedIndexInObjList.get,
                          highlightedEntry.get, numDisplayableAttributes, relationSourceEntityIn, relationIn, containingGroupIn)
        entityMenu(entityIn, newStartingDisplayIndex, highlightedEntry, relationSourceEntityIn, relationIn, containingGroupIn)
      } else if (answer == 3) {
        // MAKE SURE this next condition always matches the one in "choices(2) = ..." above
        if (highlightedEntry.isDefined && controller.canEditAttributeOnSingleLine(highlightedEntry.get)) {
          controller.editAttributeOnSingleLine(highlightedEntry.get)
          entityMenu(entityIn, attributeRowsStartingIndexIn, highlightedEntry, relationSourceEntityIn, relationIn, containingGroupIn)
        } else {
          val editedEntity: Option[Entity] = controller.editEntityName(entityIn)
          entityMenu(if (editedEntity.isDefined) editedEntity.get else entityIn,
                     attributeRowsStartingIndexIn, highlightedEntry, relationSourceEntityIn, relationIn, containingGroupIn)
        }
      } else if (answer == 4) {
        val (delOrArchiveAnswer, delEntityLink_choiceNumber, delFromContainingGroup_choiceNumber) =
          controller.askWhetherDeleteOrArchiveEtc(entityIn, relationIn, relationSourceEntityIn, containingGroupIn)

        if (delOrArchiveAnswer.isDefined) {
          val answer = delOrArchiveAnswer.get
          if (answer == 1 || answer == 2) {
            val thisEntityWasDeletedOrArchived = controller.deleteOrArchiveEntity(entityIn, answer == 1)
            if (thisEntityWasDeletedOrArchived) None
            else entityMenu(entityIn, attributeRowsStartingIndexIn, highlightedEntry, relationSourceEntityIn, relationIn, containingGroupIn)
          } else if (answer == delEntityLink_choiceNumber && relationIn.isDefined && answer <= choices.length) {
            val ans = ui.askYesNoQuestion("DELETE the relation: ARE YOU SURE?")
            if (ans.isDefined && ans.get) {
              relationIn.get.delete()
              None
            } else {
              ui.displayText("Did not delete relation.", waitForKeystroke = false)
              entityMenu(entityIn, attributeRowsStartingIndexIn, highlightedEntry, relationSourceEntityIn, relationIn, containingGroupIn)
            }
          } else if (answer == delFromContainingGroup_choiceNumber && containingGroupIn.isDefined && answer <= choices.length) {
            if (removeEntityReferenceFromGroup_Menu(entityIn, containingGroupIn))
              None
            else
              entityMenu(entityIn, attributeRowsStartingIndexIn, highlightedEntry, relationSourceEntityIn, relationIn, containingGroupIn)
          } else {
            ui.displayText("invalid response")
            entityMenu(entityIn, attributeRowsStartingIndexIn, highlightedEntry, relationSourceEntityIn, relationIn, containingGroupIn)
          }
        } else {
          entityMenu(entityIn, attributeRowsStartingIndexIn, highlightedEntry, relationSourceEntityIn, relationIn, containingGroupIn)
        }
      } else if (answer == 5) {
        goToRelatedPlaces(attributeRowsStartingIndexIn, entityIn, relationSourceEntityIn, relationIn, containingGroupIn, classDefiningEntityId)
      } else if (answer == 6 && numAttrs > 0) {
        val startingIndex: Int = getNextStartingRowsIndex(attributeTuples.length, attributeRowsStartingIndexIn, numAttrsInEntity)
        entityMenu(entityIn, startingIndex, highlightedEntry, relationSourceEntityIn, relationIn, containingGroupIn)
      } else if (answer == 7 && answer <= choices.length && !entityIsAlreadyTheDefault) {
        // updates user preferences such that this obj will be the one displayed by default in future.
        controller.mPrefs.putLong("first_display_entity", entityIn.getId)
        entityMenu(entityIn, attributeRowsStartingIndexIn, highlightedEntry, relationSourceEntityIn, relationIn, containingGroupIn)
      } else if (answer == 8 && answer <= choices.length && highlightedEntry.isDefined && highlightedIndexInObjList.isDefined) {
        // lets user select an attribute for further operations like moving, deleting.
        // (we have to have at least one choice or ui.askWhich fails...a require() call there.)
        // NOTE: this code is similar (not identical) in EntityMenu as in QuickGroupMenu: if one changes, the other might also need maintenance.
        val choices = Array[String]("keep existing (same as ESC)")
        // says 'same screenful' because (see similar cmt elsewhere).
        val leadingText: Array[String] = Array("CHOOSE an attribute to highlight (*)")
        controller.addRemainingCountToPrompt(choices, attributeTuples.length, db.getAttrCount(entityIn.getId), attributeRowsStartingIndexIn)
        val response = ui.askWhich(Some(leadingText), choices, attributeDisplayStrings, highlightIndexIn = highlightedIndexInObjList)
        val entryToHighlight: Option[Attribute] = {
          if (response.isEmpty || response.get == 1) highlightedEntry
          else {
            // those in the condition are 1-based, not 0-based.
            // user typed a letter to select an attribute (now 0-based):
            val choicesIndex = response.get - choices.length - 1
            Some(attributeTuples(choicesIndex)._2)
          }
        }
        entityMenu(entityIn, attributeRowsStartingIndexIn, entryToHighlight, relationSourceEntityIn, relationIn, containingGroupIn)
      } else if (answer == 9 && answer <= choices.length) {
        new OtherEntityMenu(ui, db, controller).otherEntityMenu(entityIn)
        // reread from db to refresh data for display, like public/non-public status:
        entityMenu(new Entity(db, entityIn.getId), attributeRowsStartingIndexIn, highlightedEntry, relationSourceEntityIn, relationIn, containingGroupIn)
      } else if (answer > choices.length && answer <= (choices.length + attributeTuples.length)) {
        // those in the condition are 1-based, not 0-based.
        // lets user go to an entity or group quickly (1 stroke)
        val choicesIndex = answer - choices.length - 1
        // checking also for " && answer <= choices.length" because otherwise choosing 'a' returns 8 but if those optional menu choices were not added in,
        // then it is found among the first "choice" answers, instead of being adjusted later ("val attributeChoicesIndex = answer - choices.length - 1")
        // to find it among the "moreChoices" as it should be: would be thrown off by the optional choice numbering.
        goToSelectedAttribute(answer, choicesIndex, attributeTuples, entityIn)
        entityMenu(entityIn, attributeRowsStartingIndexIn, Some(attributeTuples(choicesIndex)._2), relationSourceEntityIn, relationIn, containingGroupIn)
      } else {
        ui.displayText("invalid response")
        entityMenu(entityIn, attributeRowsStartingIndexIn, highlightedEntry, relationSourceEntityIn, relationIn, containingGroupIn)
      }
    }
  } catch {
    case e: Throwable =>
      // catching Throwable instead of Exception here, because sometimes depending on how I'm running X etc I might get the InternalError
      // "Can't connect to X11 window server ...", and it's better to recover from that than to abort the app (ie, when eventually calling
      // Controller.getClipboardContent)..
      controller.handleException(e)
      val ans = ui.askYesNoQuestion("Go back to what you were doing (vs. going out)?",Some("y"))
      if (ans.isDefined && ans.get) entityMenu(entityIn, attributeRowsStartingIndexIn, highlightedAttributeIn, relationSourceEntityIn, relationIn, containingGroupIn)
      else None
  }

  def moveSelectedEntry(entityIn: Entity, startingDisplayRowIndexIn: Int, totalAttrsAvailable: Int,
                        highlightedIndexInObjListIn: Int, highlightedAttributeIn: Attribute, numObjectsToDisplayIn: Int,
                        relationSourceEntityIn: Option[Entity] = None, relationIn: Option[RelationToEntity] = None,
                        containingGroupIn: Option[Group] = None): Int = {
    val choices = Array[String](// (see comments at similar location in same-named method of QuickGroupMenu.)
                                "Move up 5", "Move up 1", "Move down 1", "Move down 5",
                                "(stub)", "(stub)",
                                "Move up 25", "Move down 25")
    val response = ui.askWhich(None, choices, Array[String](), highlightIndexIn = Some(highlightedIndexInObjListIn))
    if (response.isEmpty) startingDisplayRowIndexIn
    else {
      val answer = response.get
      var numRowsToMove = 0
      var forwardNotBack = false
      if (answer == 1) {
        numRowsToMove = 5
      } else if (answer == 2) {
        numRowsToMove = 1
      } else if (answer == 3) {
        numRowsToMove = 1
        forwardNotBack = true
      } else if (answer == 4) {
        numRowsToMove = 5
        forwardNotBack = true
      } else if (answer == 7) {
        numRowsToMove = 20
      } else if (answer == 8) {
        numRowsToMove = 20
        forwardNotBack = true
      }
      if ((answer >= 1 && answer <= 4) || (answer >= 7 && answer <= 8)) {
        val displayStartingRowNumber: Int = placeEntryInPosition(entityIn.getId, totalAttrsAvailable, numRowsToMove, forwardNotBackIn = forwardNotBack,
                                                                 startingDisplayRowIndexIn, highlightedAttributeIn.getId,
                                                                 highlightedIndexInObjListIn, highlightedAttributeIn.getId, numObjectsToDisplayIn,
                                                                 highlightedAttributeIn.getFormId, highlightedAttributeIn.getFormId)
        displayStartingRowNumber
      }
      else {
        startingDisplayRowIndexIn
      }
    }
  }

  def viewContainingGroups(entityIn: Entity): Option[Entity] = {
    val leadingText = List[String]("Pick from menu, or a letter to (go to if one or) see the entities containing that group, or Alt+<letter> for the actual " +
                                   "*group* by letter")
    val choices: Array[String] = Array(controller.listNextItemsPrompt)
    val numDisplayableItems = ui.maxColumnarChoicesToDisplayAfter(leadingText.size, choices.length, controller.maxNameLength)
    // (see comment in similar location just above where this is called)
    val containingRelationToGroups: util.ArrayList[RelationToGroup] = db.getContainingRelationToGroups(entityIn, 0,
                                                                                                       Some(numDisplayableItems))
    val containingRtgDescriptions: Array[String] = containingRelationToGroups.toArray.map {
                                                                                            case rtg: (RelationToGroup) =>
                                                                                              val entityName: String = new Entity(db,
                                                                                                                                  rtg.getParentId)
                                                                                                                       .getName
                                                                                              val rt: RelationType = new RelationType(db,
                                                                                                                                      rtg.getAttrTypeId)
                                                                                              "entity " + entityName + " " +
                                                                                              rtg.getDisplayString(controller.maxNameLength, None, Some(rt))
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
          entityMenu(containingEntity, 0, None, None, None, Some(new Group(db, containingRelationToGroup.getGroupId)))
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
                                                              Some(entityIn))
      } else {
        ui.displayText("unknown response")
        None
      }
    }
  }

  def goToRelatedPlaces(startingAttributeRowsIndexIn: Int, entityIn: Entity, relationSourceEntityIn: Option[Entity] = None,
                        relationIn: Option[RelationToEntity] = None, containingGroupIn: Option[Group] = None,
                        classDefiningEntityId: Option[Long]): Option[Entity] = {
    //idea: make this and similar locations share code? What other places could?? There is plenty of duplicated code here!
    val leadingText = Some(Array("Go to..."))
    val seeContainingEntities_choiceNumber: Int = 1
    val seeContainingGroups_choiceNumber: Int = 2
    val goToRelation_choiceNumber: Int = 3
    val goToRelationType_choiceNumber: Int = 4
    var goToClassDefiningEntity_choiceNumber: Int = 3
    val numContainingEntities = db.getEntitiesContainingEntity(entityIn, 0).size
    // (idea: make this next call efficient: now it builds them all when we just want a count; but is infrequent & likely small numbers)
    val numContainingGroups = db.getCountOfGroupsContainingEntity(entityIn.getId)
    var containingGroup: Option[Group] = None
    var containingRtg: Option[RelationToGroup] = None
    if (numContainingGroups == 1) {
      val containingGroupsIds: List[Long] = db.getContainingGroupsIds(entityIn.getId)
      // (Next line is just confirming the consistency of logic that got us here: see 'if' just above.)
      require(containingGroupsIds.size == 1)
      containingGroup = Some(new Group(db, containingGroupsIds.head))

      val containingRtgList: util.ArrayList[RelationToGroup] = db.getContainingRelationToGroups(entityIn, 0, Some(1))
      if (containingRtgList.size < 1) {
        ui.displayText("There is a group containing the entity (" + entityIn.getName + "), but:  " + Controller.ORPHANED_GROUP_MESSAGE)
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
    if (relationIn.isDefined) {
      choices = choices :+ "Go edit the relation to entity that that led here: " +
                           relationIn.get.getDisplayString(15, relationSourceEntityIn, Some(new RelationType(db, relationIn.get.getAttrTypeId)))
      choices = choices :+ "Go to the type, for the relation that that led here: " + new Entity(db, relationIn.get.getAttrTypeId).getName
      goToClassDefiningEntity_choiceNumber += 2
    }
    if (classDefiningEntityId.isDefined) {
      choices = choices ++ Array[String]("Go to class-defining entity")
    }
    var relationToEntity: Option[RelationToEntity] = relationIn

    val response = ui.askWhich(leadingText, choices, Array[String]())
    if (response.isDefined) {
      val goWhereAnswer = response.get
      if (goWhereAnswer == seeContainingEntities_choiceNumber && goWhereAnswer <= choices.length) {
        val leadingText = List[String]("Pick from menu, or an entity by letter")
        val choices: Array[String] = Array(controller.listNextItemsPrompt)
        val numDisplayableItems: Long = ui.maxColumnarChoicesToDisplayAfter(leadingText.size, choices.length, controller.maxNameLength)
        // This is partly set up so it could handle multiple screensful, but would need to be broken into a recursive method that
        // can specify dif't values on each call, for the startingIndexIn parm of getRelatingEntities.  I.e., could make it look more like
        // searchForExistingObject or such ? IF needed.  But to be needed means the user is putting the same object related by multiple
        // entities: enough to fill > 1 screen when listed.
        val containingEntities: util.ArrayList[(Long, Entity)] = db.getEntitiesContainingEntity(entityIn, 0, Some(numDisplayableItems))
        val containingEntitiesNames: Array[String] = containingEntities.toArray.map {
                                                                                      case relTypeIdAndEntity: (Long, Entity) =>
                                                                                        val entity: Entity = relTypeIdAndEntity._2
                                                                                        entity.getName
                                                                                      case _ => throw new OmException("??")
                                                                                    }
        val ans = ui.askWhich(Some(leadingText.toArray), choices, containingEntitiesNames)
        if (ans.isEmpty) return None
        else {
          val answer = ans.get
          if (answer == 1 && answer <= choices.length) {
            // see comment above
            ui.displayText("not yet implemented")
          } else if (answer > choices.length && answer <= (choices.length + containingEntities.size)) {
            // those in the condition on the previous line are 1-based, not 0-based.
            val index = answer - choices.length - 1
            // user typed a letter to select.. (now 0-based); selected a new object and so we return to the previous menu w/ that one displayed & current
            val entity: Entity = containingEntities.get(index)._2
            entityMenu(entity, 0, None)
          } else {
            ui.displayText("unknown response")
          }
        }
      } else if (goWhereAnswer == seeContainingGroups_choiceNumber && goWhereAnswer <= choices.length) {
        if (numContainingGroups == 1) {
          require(containingGroup.isDefined)
          new QuickGroupMenu(ui, db, controller).quickGroupMenu(containingGroup.get, 0, containingRtg)
        } else {
          viewContainingGroups(entityIn)
        }
      } else if (goWhereAnswer == goToRelation_choiceNumber && relationIn.isDefined && goWhereAnswer <= choices.length) {
        def dummyMethod(inDH: RelationToEntityDataHolder, inEditing: Boolean): Option[RelationToEntityDataHolder] = {
          Some(inDH)
        }
        def updateRelationToEntity(dhInOut: RelationToEntityDataHolder) {
          relationIn.get.update(Some(dhInOut.attrTypeId), dhInOut.validOnDate, Some(dhInOut.observationDate))
        }
        val relationToEntityDH: RelationToEntityDataHolder = new RelationToEntityDataHolder(relationIn.get.getAttrTypeId, relationIn.get.getValidOnDate,
                                                                                            relationIn.get.getObservationDate, relationIn.get
                                                                                                                               .getRelatedId1,
                                                                                            relationIn.get.getRelatedId2)
        controller.askForInfoAndUpdateAttribute[RelationToEntityDataHolder](relationToEntityDH, Controller.RELATION_TO_ENTITY_TYPE,
                                                                            "CHOOSE TYPE OF Relation to Entity:", dummyMethod, updateRelationToEntity)
        // force a reread from the DB so it shows the right info on the repeated menu (below):
        relationToEntity = Some(new RelationToEntity(db, relationIn.get.getId, relationIn.get.getAttrTypeId, relationIn.get.getRelatedId1,
                                                     relationIn.get.getRelatedId2))
      }
      else if (goWhereAnswer == goToRelationType_choiceNumber && relationIn.isDefined && goWhereAnswer <= choices.length) {
        entityMenu(new Entity(db, relationIn.get.getAttrTypeId), 0, None)
      }
      else if (goWhereAnswer == goToClassDefiningEntity_choiceNumber && classDefiningEntityId.isDefined && goWhereAnswer <= choices.length) {
        entityMenu(new Entity(db, classDefiningEntityId.get), 0, None)
      } else {
        ui.displayText("invalid response")
      }
    }
    //ck 1st if entity exists, if not return None. It could have been deleted while navigating around.
    if (db.entityKeyExists(entityIn.getId)) entityMenu(entityIn, startingAttributeRowsIndexIn, None, relationSourceEntityIn, relationToEntity, containingGroupIn)
    else None
  }

  def removeEntityReferenceFromGroup_Menu(entityIn: Entity, containingGroupIn: Option[Group]): Boolean = {
    val groupCount: Long = db.getCountOfGroupsContainingEntity(entityIn.getId)
    val (entityCountNonArchived, entityCountArchived) = db.getCountOfEntitiesContainingEntity(entityIn.getId)
    val ans = ui.askYesNoQuestion("REMOVE this entity from that group: ARE YOU SURE? (This isn't a deletion. It can still be found by searching, and in " + 
                                  (groupCount - 1) + " group(s), and associated directly with " +
                                  entityCountNonArchived + " other entity(ies) (and " + entityCountArchived + " archived entities)..")
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

  def getLeadingText(leadingTextIn: Array[String], numAttributes: Int,
                     entityIn: Entity, relationSourceEntityIn: Option[Entity] = None,
                     relationIn: Option[RelationToEntity] = None, containingGroupIn: Option[Group] = None): Array[String] = {
    leadingTextIn(0) = "**CURRENT ENTITY " + entityIn.getId + ": " + entityIn.getDisplayString
    if (relationIn.isDefined) {
      leadingTextIn(0) += ": found via relation: " + relationSourceEntityIn.get.getName + " " +
                          relationIn.get.getDisplayString(0, Some(new Entity(db, relationIn.get.getRelatedId2)),
                                                          Some(new RelationType(db, relationIn.get.getAttrTypeId)))
    }
    if (containingGroupIn.isDefined) {
      leadingTextIn(0) += ": found via group: " + containingGroupIn.get.getName
    }
    leadingTextIn(0) += ": created " + entityIn.getCreationDateFormatted
    leadingTextIn(1) = if (numAttributes == 0) "No attributes have been assigned to this object, yet."
    else "Attribute list menu: (or choose attribute by letter)"
    leadingTextIn
  }

  def getItemDisplayStrings(attributeTuples: Array[(Long, Attribute)]) = {
    val attributeNames: Array[String] =
      for (attributeTuple <- attributeTuples) yield {
        val attribute = attributeTuple._2
        attribute match {
        case relation: RelationToEntity =>
          val relationType = new RelationType(db, relation.getAttrTypeId)
          attribute.getDisplayString(controller.maxNameLength, Some(new Entity(db, relation.getRelatedId2)), Some(relationType))
        case relation: RelationToGroup =>
          val relationType = new RelationType(db, relation.getAttrTypeId)
          attribute.getDisplayString(controller.maxNameLength, None, Some(relationType))
        case _ =>
          attribute.getDisplayString(controller.maxNameLength, None, None)
      }
    }
    attributeNames
  }

  def addAttribute(entityIn: Entity, startingAttributeIndexIn: Int, highlightedAttributeIn: Option[Attribute], relationSourceEntityIn: Option[Entity] = None,
                   relationIn: Option[RelationToEntity] = None, containingGroupIn: Option[Group] = None): Option[Attribute] = {
    val whichKindOfAttribute =
      ui.askWhich(Some(Array("Choose which kind of attribute to add:")),
                  Array("quantity attribute (example: a numeric value like \"length\"",
                        "text attribute (rare: usually prefer relations; but for example: a serial number, which is not subject to arithmetic)",
                        "date",
                        "true/false value",

                        "external file (BUT CONSIDER FIRST ADDING AN ENTITY SPECIFICALLY FOR THE DOCUMENT SO IT CAN HAVE A DATE, OTHER ATTRS ETC.; " +
                        "AND ADDING THE DOCUMENT TO THAT ENTITY, SO IT CAN ALSO BE ASSOCIATED WITH OTHER ENTITIES EASILY!; also, " +
                        "given the concept behind OM, it's probably best" +
                        " to use this only for historical artifacts, or when you really can't fully model the data right now",

                        "Relation to entity (i.e., \"is near\" a microphone)",
                        "Relation to group (i.e., \"has\" a list/group)",
                        "external web page (or other URI, to refer to external information and optionally quote it)")
                 )
    if (whichKindOfAttribute.isDefined) {
      val whichKindAnswer = whichKindOfAttribute.get
      if (whichKindAnswer == 1) {
        def addQuantityAttribute(dhIn: QuantityAttributeDataHolder): Option[QuantityAttribute] = {
          Some(entityIn.addQuantityAttribute(dhIn.attrTypeId, dhIn.unitId, dhIn.number, dhIn.validOnDate, dhIn.observationDate))
        }
        controller.askForInfoAndAddAttribute[QuantityAttributeDataHolder](new QuantityAttributeDataHolder(0, None, 0, 0, 0), Controller.QUANTITY_TYPE,
                                                                          controller.quantityDescription,
                                                                          controller.askForQuantityAttributeNumberAndUnit, addQuantityAttribute)
      } else if (whichKindAnswer == 2) {
        def addTextAttribute(dhIn: TextAttributeDataHolder): Option[TextAttribute] = {
          Some(entityIn.addTextAttribute(dhIn.attrTypeId, dhIn.text, dhIn.validOnDate, dhIn.observationDate))
        }
        controller.askForInfoAndAddAttribute[TextAttributeDataHolder](new TextAttributeDataHolder(0, Some(0), 0, ""), Controller.TEXT_TYPE,
                                                                      "SELECT TYPE OF " + controller.textDescription + ": ", controller
                                                                                                                             .askForTextAttributeText,
                                                                      addTextAttribute)
      } else if (whichKindAnswer == 3) {
        def addDateAttribute(dhIn: DateAttributeDataHolder): Option[DateAttribute] = {
          Some(entityIn.addDateAttribute(dhIn.attrTypeId, dhIn.date))
        }
        controller.askForInfoAndAddAttribute[DateAttributeDataHolder](new DateAttributeDataHolder(0, 0), Controller.DATE_TYPE,
                                                                      "SELECT TYPE OF DATE: ", controller.askForDateAttributeValue, addDateAttribute)
      } else if (whichKindAnswer == 4) {
        def addBooleanAttribute(dhIn: BooleanAttributeDataHolder): Option[BooleanAttribute] = {
          Some(entityIn.addBooleanAttribute(dhIn.attrTypeId, dhIn.boolean))
        }
        controller.askForInfoAndAddAttribute[BooleanAttributeDataHolder](new BooleanAttributeDataHolder(0, Some(0), 0, false), Controller.BOOLEAN_TYPE,
                                                                         "SELECT TYPE OF TRUE/FALSE VALUE: ", controller.askForBooleanAttributeValue,
                                                                         addBooleanAttribute)
      } else if (whichKindAnswer == 5) {
        def addFileAttribute(dhIn: FileAttributeDataHolder): Option[FileAttribute] = {
          Some(entityIn.addFileAttribute(dhIn.attrTypeId, dhIn.description, new java.io.File(dhIn.originalFilePath)))
        }
        val result: Option[FileAttribute] = controller.askForInfoAndAddAttribute[FileAttributeDataHolder](new FileAttributeDataHolder(0, "", ""), Controller.FILE_TYPE,
                                                                                                          "SELECT TYPE OF FILE: ", controller.askForFileAttributeInfo,
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
      } else if (whichKindAnswer == 6) {
        def addRelationToEntity(dhIn: RelationToEntityDataHolder): Option[RelationToEntity] = {
          Some(entityIn.addRelationToEntity(dhIn.attrTypeId, dhIn.entityId1, dhIn.entityId2, dhIn.validOnDate, dhIn.observationDate))
        }
        controller.askForInfoAndAddAttribute[RelationToEntityDataHolder](new RelationToEntityDataHolder(0, None, 0, entityIn.getId, 0), Controller .RELATION_TYPE_TYPE,
                                                                         "CREATE OR SELECT RELATION TYPE: (" + controller.mRelTypeExamples + ")",
                                                                         controller.askForRelationEntityIdNumber2, addRelationToEntity)
      } else if (whichKindAnswer == 7) {
        def addRelationToGroup(dhIn: RelationToGroupDataHolder): Option[RelationToGroup] = {
          val rtgId: Long = entityIn.addRelationToGroup(dhIn.attrTypeId, dhIn.groupId, dhIn.validOnDate, dhIn.observationDate)
          Some(new RelationToGroup(db, rtgId, dhIn.entityId, dhIn.attrTypeId, dhIn.groupId))
        }
        val result: Option[Attribute] = controller.askForInfoAndAddAttribute[RelationToGroupDataHolder](new RelationToGroupDataHolder(entityIn.getId, 0, 0,
                                                                                                                                      None,
                                                                                                                                      System
                                                                                                                                      .currentTimeMillis()),
                                                                                                        Controller.RELATION_TYPE_TYPE,
                                                                                                        "CREATE OR SELECT RELATION TYPE: (" + controller
                                                                                                                                              .mRelTypeExamples + ")" +
                                                                                                        "." + TextUI.NEWLN + "(Does anyone see a specific " +
                                                                                                        "reason to keep asking for these dates?)",
                                                                                                        controller.askForRelToGroupInfo, addRelationToGroup)
        if (result.isEmpty) {
          entityMenu(entityIn, startingAttributeIndexIn, highlightedAttributeIn, relationSourceEntityIn, relationIn, containingGroupIn)
          None
        } else {
          val newRtg = result.get.asInstanceOf[RelationToGroup]
          new GroupMenu(ui, db, controller).groupMenu(new Group(db, newRtg.getGroupId), 0, Some(newRtg), None, Some(entityIn))
          result
        }
      } else if (whichKindAnswer == 8) {
        val newEntityName: Option[String] = ui.askForString(Some(Array{"Enter a name (or description) for this web page or other URI"}))
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
        val quote:Option[String] = if (ans2.isEmpty) {
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

        val (newEntity: Entity, newRTE: RelationToEntity) = db.addUriEntityWithUriAttribute(entityIn, newEntityName.get, uri, System.currentTimeMillis(),
                                                                entityIn.getPublic, callerManagesTransactionsIn = false, quote)
        entityMenu(newEntity, relationSourceEntityIn = Some(entityIn))
        Some(newRTE)
      } else {
        ui.displayText("invalid response")
        None
      }
    } else None
  }

  def getNextStartingRowsIndex(numAttrsToDisplay: Int, startingAttributeRowsIndexIn: Int, numAttrsInEntity: Long): Int = {
    val startingIndex = {
      val currentPosition = startingAttributeRowsIndexIn + numAttrsToDisplay
      if (currentPosition >= numAttrsInEntity) {
        ui.displayText("End of attribute list found; restarting from the beginning.")
        0 // start over
      } else currentPosition

    }
    startingIndex
  }

  def goToSelectedAttribute(answer: Int, choiceIndex: Int, attributeTuples: Array[(Long, Attribute)], entityIn: Entity) {
    // user typed a letter to select an attribute (now 0-based)
    if (choiceIndex >= attributeTuples.length) {
      ui.displayText("The program shouldn't have let us get to this point, but the selection " + answer + " is not in the list.")
    } else {
      val o: Attribute = attributeTuples(choiceIndex)._2
      o match {
        //idea: there's probably also some more scala-like cleaner syntax 4 this, as elsewhere:
        case qa: QuantityAttribute => controller.attributeEditMenu(qa)
        case da: DateAttribute => controller.attributeEditMenu(da)
        case ba: BooleanAttribute => controller.attributeEditMenu(ba)
        case fa: FileAttribute => controller.attributeEditMenu(fa)
        case ta: TextAttribute => controller.attributeEditMenu(ta)
        case relToEntity: RelationToEntity => entityMenu(new Entity(db, relToEntity.getRelatedId2), 0, None, Some(entityIn), Some(relToEntity))
        case relToGroup: RelationToGroup => new QuickGroupMenu(ui, db, controller).quickGroupMenu(new Group(db, relToGroup.getGroupId), 0, Some(relToGroup),
                                                                                                  containingEntityIn = Some(entityIn))
        case _ => throw new Exception("Unexpected choice has class " + o.getClass.getName + "--what should we do here?")
      }
    }
  }

  protected def getAdjacentEntriesSortingIndexes(entityIdIn: Long, movingFromPosition_sortingIndexIn: Long, queryLimitIn: Option[Long],
                                   forwardNotBackIn: Boolean): List[Array[Option[Any]]] = {
    db.getAdjacentAttributesSortingIndexes(entityIdIn, movingFromPosition_sortingIndexIn, queryLimitIn, forwardNotBackIn)
  }

  protected def getNearestEntrysSortingIndex(entityIdIn: Long, startingPointSortingIndexIn: Long, forwardNotBackIn: Boolean): Option[Long] = {
    db.getNearestAttributeEntrysSortingIndex(entityIdIn, startingPointSortingIndexIn, forwardNotBackIn = forwardNotBackIn)
  }

  protected def renumberSortingIndexes(entityIdIn: Long): Unit = {
    db.renumberAttributeSortingIndexes(entityIdIn)
  }

  protected def updateSortedEntry(entityIdIn: Long, movingAttributeFormIdIn: Int, movingAttributeIdIn: Long, sortingIndexIn: Long): Unit = {
    db.updateAttributeSorting(entityIdIn, movingAttributeFormIdIn, movingAttributeIdIn, sortingIndexIn)
  }

  protected def getSortingIndex(entityIdIn: Long, attributeFormIdIn: Int, attributeIdIn: Long): Long = {
    db.getEntityAttributeSortingIndex(entityIdIn, attributeFormIdIn, attributeIdIn)
  }

  protected def indexIsInUse(entityIdIn: Long, sortingIndexIn: Long): Boolean = {
    db.attributeSortingIndexInUse(entityIdIn, sortingIndexIn)
  }

  protected def findUnusedSortingIndex(entityIdIn: Long, startingWithIn: Long): Long = {
    db.findUnusedAttributeSortingIndex(entityIdIn, Some(startingWithIn))
  }

}
