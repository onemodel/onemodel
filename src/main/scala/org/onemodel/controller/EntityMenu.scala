/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003-2004 and 2008-2016 inclusive, Luke A. Call; all rights reserved.
    (That copyright statement was previously 2013-2015, until I remembered that much of Controller came from TextUI.scala and TextUI.java before that.)
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule, guidelines around binary
    distribution, and the GNU Affero General Public License as published by the Free Software Foundation, either version 3
    of the License, or (at your option) any later version.  See the file LICENSE for details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
package org.onemodel.controller

import java.util

import org.onemodel._
import org.onemodel.database.PostgreSQLDatabase
import org.onemodel.model._

import scala.annotation.tailrec
import scala.collection.mutable

class EntityMenu(override val ui: TextUI, override val db: PostgreSQLDatabase, val controller: Controller) extends SortableEntriesMenu(ui, db) {
  // 2nd return value is whether entityIsDefault (ie whether default object when launching OM is already this entity)
  def getChoices(entityIn: Entity, numAttrsIn: Long): Array[String] = {
    // (idea: might be a little silly to do it this way, once this # gets very big?:)
    var choices = Array[String]("Add entry quickly (creates a \"has\" relation to a new Entity)",
                                if (numAttrsIn > 0) "Move selection (*) up/down" else "(stub)",

                                "[app will fill this one in just a bit later, at \"choices (3) = \" below.  KEEP IT IN THIS RELATIVE POSITION OR CHANGE THE" +
                                " CODE NEAR THE TOP OF entityMenu THAT CHECKS FOR A VALUE IN highlightedAttributeIn]",

                                "Add attribute (add entry with detailed options)",
                                "Go to selected attribute",
                                "Search / List next ...")
    // (the next line's display text is abbreviated to fit in an 80-column terminal window:)
    choices = choices :+ "Select target (entry move destination: gets a '+' marker)"
    // (the next line's display text is abbreviated to fit in an 80-column terminal window:)
    choices = choices :+ (if (numAttrsIn > 0) "Select attribute to highlight (with '*'; type a letter to go to its attr menu)" else "(stub)")
    choices = choices :+ (if (controller.getDefaultEntity._1.isEmpty) "****TRY ME---> " else "") + "Other entity operations..."
    choices
  }

  /** The parameter attributeRowsStartingIndexIn means: of all the sorted attributes of entityIn, which one is to be displayed first (since we can only display
    * so many at a time with finite screen size).
   * Returns None if user wants out (or if entity was deleted so we should exit to containing menu).
   * */
  //@tailrec //removed for now until the compiler can handle it with where the method calls itself.
  //idea on scoping: make this limited like this somehow?:  private[org.onemodel] ... Same for all others like it?
  def entityMenu(entityIn: Entity, attributeRowsStartingIndexIn: Int = 0, highlightedAttributeIn: Option[Attribute] = None,
                 targetForMovesIn: Option[Attribute] = None,
                 containingRelationToEntityIn: Option[RelationToEntity] = None, containingGroupIn: Option[Group] = None): Option[Entity] = try {
    require(entityIn != null)
    if (!db.entityKeyExists(entityIn.getId, includeArchived = false)) {
      ui.displayText("The desired entity, " + entityIn.getId + ", has been deleted or archived, probably while browsing other entities via menu options," +
                     "and so cannot be displayed here.  Exiting to the next menu.")
      return None
    }
    if (containingRelationToEntityIn.isDefined) {
      require(containingGroupIn.isEmpty)
      require(containingRelationToEntityIn.get.getRelatedId2 == entityIn.getId)
    }
    if (containingGroupIn.isDefined) require(containingRelationToEntityIn.isEmpty)
    val numAttrsInEntity: Long = entityIn.getAttrCount
    val classDefiningEntityId: Option[Long] = entityIn.getClassDefiningEntityId
    val leadingText: Array[String] = new Array[String](2)
    val relationSourceEntity: Option[Entity] = {
      // (checking if exists also, because it could have been removed in another menu option)
      if (containingRelationToEntityIn.isEmpty || !db.entityKeyExists(containingRelationToEntityIn.get.getRelatedId1)) {
        None
      } else {
        Some(new Entity(db, containingRelationToEntityIn.get.getParentId))
      }
    }
    val choices: Array[String] = getChoices(entityIn, numAttrsInEntity)
    val numDisplayableAttributes: Int = ui.maxColumnarChoicesToDisplayAfter(leadingText.length, choices.length, Controller.maxNameLength)
    val (attributeTuples: Array[(Long, Attribute)], totalAttrsAvailable: Int) =
      db.getSortedAttributes(entityIn.getId, attributeRowsStartingIndexIn, numDisplayableAttributes)
    if ((numAttrsInEntity > 0 && attributeRowsStartingIndexIn == 0) || attributeTuples.length > 0) {
      require(numAttrsInEntity > 0 && attributeTuples.length > 0)
    }
    controller.addRemainingCountToPrompt(choices, attributeTuples.length, totalAttrsAvailable, attributeRowsStartingIndexIn)
    val leadingTextModified = getLeadingText(leadingText, attributeTuples.length, entityIn, relationSourceEntity, containingRelationToEntityIn, containingGroupIn)
    val (attributeDisplayStrings: Array[String], attributesToDisplay: util.ArrayList[Attribute]) = getItemDisplayStringsAndAttrs(attributeTuples)

    // The variable highlightedIndexInObjList means: of the sorted attributes selected *for display* (potentially fewer than all existing attributes),
    // this is the zero-based index of the one that is marked for possible moving around in the sorted order (in the UI, marked as selected).
    val (highlightedIndexInObjList: Option[Int], highlightedEntry: Option[Attribute], moveTargetIndexInObjList: Option[Int],
    targetForMoves: Option[Attribute]) = {
      if (attributeTuples.length == 0) {
        (None, None, None, None)
      } else {
        var highlightedEntry: Option[Attribute] = Some(highlightedAttributeIn.getOrElse(attributeTuples(0)._2))
        val highlightedObjFormId: Int = highlightedEntry.get.getFormId
        val highlightedObjId: Long = highlightedEntry.get.getId
        var highlightedIndexInObjList: Option[Int] = None
        var moveTargetIndexInObjList: Option[Int] = None
        var targetForMoves: Option[Attribute] = None
        var index = -1
        for (attributeTuple <- attributeTuples) {
          index += 1
          val attribute = attributeTuple._2
          if (attribute.getFormId == highlightedObjFormId && attribute.getId == highlightedObjId) {
            highlightedIndexInObjList = Some(index)
          }
          if (targetForMovesIn.isDefined && attribute.getFormId==targetForMovesIn.get.getFormId && attribute.getId==targetForMovesIn.get.getId) {
            moveTargetIndexInObjList = Some(index)
            targetForMoves = targetForMovesIn
          }
        }
        // if we got to this point, it could simply have been deleted or something (probably) but still passed in the highlightedAttributeIn parm by mistake,
        // so just return something safe (instead of throwing an exception, as in a previous commit):
        if (highlightedIndexInObjList.isEmpty) {
          // maybe the highlightedAttributeIn was defined but not found in the list for some unknown reason, so at least recover nicely:
          highlightedIndexInObjList = Some(0)
          highlightedEntry = Some(attributeTuples(0)._2)
        }
        if (moveTargetIndexInObjList.isDefined && highlightedIndexInObjList.get == moveTargetIndexInObjList.get) {
          // doesn't make sense if they're the same (ie move both, into both?, like if user changed the previous highlight on 1st selection to a move
          // target), so change one:
          if (highlightedIndexInObjList.get == 0 && attributeTuples.length > 1) {
            val indexToUseInstead = 1
            highlightedIndexInObjList = Some(indexToUseInstead)
            highlightedEntry = Some(attributeTuples(indexToUseInstead)._2)
          } else {
            moveTargetIndexInObjList = None
            targetForMoves = None
          }
        }
        (highlightedIndexInObjList, highlightedEntry, moveTargetIndexInObjList, targetForMoves)
      }
    }

    choices(2) =
      // MAKE SURE this condition always matches the one in the edit handler below:
      if (highlightedEntry.isDefined && controller.canEditAttributeOnSingleLine(highlightedEntry.get)) {
        // (the next line's display text is abbreviated to fit in an 80-column terminal window:)
        "Edit the selected attribute's content (single line; go into attr for more)"
      } else "Edit entity name"

    // MAKE SURE this next condition always is the opposite of the one at comment mentioning "choices(4) = ..." below
    if (highlightedIndexInObjList.isEmpty) {
      choices(4) = "(stub)"
    }

    val response = ui.askWhich(Some(leadingTextModified), choices, attributeDisplayStrings, highlightIndexIn = highlightedIndexInObjList,
                               secondaryHighlightIndexIn = moveTargetIndexInObjList)
    if (response.isEmpty) None
    else {
      val answer = response.get
      if (answer == 1) {
        val (newAttributeToHighlight: Option[Attribute], displayStartingRowNumber: Int) = {
          // ask for less info when here, to add entity quickly w/ no fuss, like brainstorming. Like in QuickGroupMenu.  User can always use option 2.
          val newEntity: Option[Entity] = controller.askForNameAndWriteEntity(Controller.ENTITY_TYPE, inLeadingText = Some("NAME THE ENTITY:"))
          if (newEntity.isDefined) {
            val newAttribute: Attribute = entityIn.addHASRelationToEntity(newEntity.get.getId, None, System.currentTimeMillis())
            val displayStartingRowNumber: Int = placeEntryInPosition(entityIn.getId, entityIn.getAttrCount, 0, forwardNotBackIn = true,
                                                                     attributeRowsStartingIndexIn, newAttribute.getId,
                                                                     highlightedIndexInObjList.getOrElse(0),
                                                                     if (highlightedEntry.isDefined) Some(highlightedEntry.get.getId) else None,
                                                                     numDisplayableAttributes, newAttribute.getFormId,
                                                                     if (highlightedEntry.isDefined) Some(highlightedEntry.get.getFormId) else None)
            controller.defaultAttributeCopying(newEntity.get, Some(attributeTuples))
            (Some(newAttribute), displayStartingRowNumber)
          }
          else (highlightedEntry, attributeRowsStartingIndexIn)
        }
        entityMenu(entityIn, displayStartingRowNumber, newAttributeToHighlight, targetForMoves, containingRelationToEntityIn, containingGroupIn)
      } else if (answer == 2 && highlightedEntry.isDefined && highlightedIndexInObjList.isDefined && numAttrsInEntity > 0) {
        val (newStartingDisplayIndex: Int, movedOneOut: Boolean) = moveSelectedEntry(entityIn, attributeRowsStartingIndexIn, totalAttrsAvailable, targetForMoves,
                                                        highlightedIndexInObjList.get, highlightedEntry.get, numDisplayableAttributes,
                                                        relationSourceEntity,
                                                        containingRelationToEntityIn, containingGroupIn)
        val attrToHighlight: Option[Attribute] = Controller.findAttributeToHighlightNext(attributeTuples.length, attributesToDisplay, removedOne = movedOneOut,
                                                                                         highlightedIndexInObjList.get, highlightedEntry.get)
        entityMenu(entityIn, newStartingDisplayIndex, attrToHighlight, targetForMoves, containingRelationToEntityIn, containingGroupIn)
      } else if (answer == 3) {
        // MAKE SURE this next condition always matches the one in "choices(2) = ..." above
        if (highlightedEntry.isDefined && controller.canEditAttributeOnSingleLine(highlightedEntry.get)) {
          controller.editAttributeOnSingleLine(highlightedEntry.get)
          entityMenu(entityIn, attributeRowsStartingIndexIn, highlightedEntry, targetForMoves, containingRelationToEntityIn, containingGroupIn)
        } else {
          val editedEntity: Option[Entity] = controller.editEntityName(entityIn)
          entityMenu(if (editedEntity.isDefined) editedEntity.get else entityIn,
                     attributeRowsStartingIndexIn, highlightedEntry, targetForMoves, containingRelationToEntityIn, containingGroupIn)
        }
      } else if (answer == 4) {
          val newAttribute: Option[Attribute] = addAttribute(entityIn, attributeRowsStartingIndexIn, highlightedEntry, targetForMoves,
                                                             containingRelationToEntityIn, containingGroupIn)
          if (newAttribute.isDefined && highlightedEntry.isDefined) {
            placeEntryInPosition(entityIn.getId, entityIn.getAttrCount, 0, forwardNotBackIn = true, attributeRowsStartingIndexIn, newAttribute.get.getId,
                                 highlightedIndexInObjList.getOrElse(0),
                                 if (highlightedEntry.isDefined) Some(highlightedEntry.get.getId) else None,
                                 numDisplayableAttributes, newAttribute.get.getFormId,
                                 if (highlightedEntry.isDefined) Some(highlightedEntry.get.getFormId) else None)
            entityMenu(entityIn, attributeRowsStartingIndexIn, newAttribute, targetForMoves, containingRelationToEntityIn, containingGroupIn)
          } else {
            entityMenu(entityIn, attributeRowsStartingIndexIn, highlightedEntry, targetForMoves, containingRelationToEntityIn, containingGroupIn)
          }
      } else if (answer == 5) {
        // MAKE SURE this next condition always is the exact opposite of the one in "choices(4) = ..." above (4 vs. 5 because they are 0- vs. 1-based)
        if (highlightedIndexInObjList.isDefined) {
          val listEntryIsGoneNow = goToSelectedAttribute(answer, highlightedIndexInObjList.get, attributeTuples, entityIn)
          if (!db.entityKeyExists(entityIn.getId, includeArchived = false)) {
            // (entity could have been deleted or archived while browsing among containers via submenus)
            None
          } else {
            val defaultEntryToHighlight: Option[Attribute] = highlightedEntry
            // check this, given that while in the goToSelectedAttribute method, the previously highlighted one could have been removed from the list:
            val nextToHighlight: Option[Attribute] = determineNextEntryToHighlight(entityIn, attributeRowsStartingIndexIn, targetForMovesIn,
                                                                                   containingRelationToEntityIn, containingGroupIn, attributesToDisplay,
                                                                                   listEntryIsGoneNow, defaultEntryToHighlight, highlightedIndexInObjList)
            entityMenu(new Entity(db, entityIn.getId), attributeRowsStartingIndexIn, nextToHighlight, targetForMovesIn,
                       containingRelationToEntityIn, containingGroupIn)
          }
        } else {
          ui.displayText("nothing selected")
          entityMenu(entityIn, attributeRowsStartingIndexIn, highlightedEntry, targetForMovesIn, containingRelationToEntityIn, containingGroupIn)
        }
      } else if (answer == 6 && numAttrsInEntity > 0) {
        entitySearchSubmenu(entityIn, attributeRowsStartingIndexIn, containingRelationToEntityIn, containingGroupIn, numAttrsInEntity, attributeTuples,
                            highlightedEntry, targetForMoves, answer)
        entityMenu(entityIn, attributeRowsStartingIndexIn, highlightedEntry, targetForMoves, containingRelationToEntityIn, containingGroupIn)
      } else if (answer == 7) {
        // NOTE: this code is similar (not identical) in EntityMenu as in QuickGroupMenu: if one changes,
        // THE OTHER MIGHT ALSO NEED MAINTENANCE!
        val choices = Array[String](Controller.unselectMoveTargetPromptText)
        val leadingText: Array[String] = Array(Controller.unselectMoveTargetLeadingText)
        controller.addRemainingCountToPrompt(choices, attributeTuples.length, entityIn.getAttrCount, attributeRowsStartingIndexIn)

        val response = ui.askWhich(Some(leadingText), choices, attributeDisplayStrings, highlightIndexIn = highlightedIndexInObjList,
                                   secondaryHighlightIndexIn = moveTargetIndexInObjList)
        val (entryToHighlight, selectedTargetAttribute): (Option[Attribute], Option[Attribute]) = {
          if (response.isEmpty) (highlightedEntry, targetForMoves)
          else {
            val answer = response.get
            if (answer == 1) {
              (highlightedEntry, None)
            } else {
              // those in the condition are 1-based, not 0-based.
              // user typed a letter to select an attribute (now 0-based):
              val selectionIndex: Int = answer - choices.length - 1
              val userSelection: Attribute = attributeTuples(selectionIndex)._2
              if (selectionIndex == highlightedIndexInObjList.get) {
                // chose same entity for the target, as the existing highlighted selection, so make it the target, and no highlighted one.
                (None, Some(userSelection))
              } else {
                (highlightedEntry, Some(userSelection))
              }
            }
          }
        }
        entityMenu(entityIn, attributeRowsStartingIndexIn, entryToHighlight, selectedTargetAttribute, containingRelationToEntityIn, containingGroupIn)
      } else if (answer == 8 && answer <= choices.length && numAttrsInEntity > 0) {
        // lets user select an attribute for further operations like moving, deleting.
        // (we have to have at least one choice or ui.askWhich fails...a require() call there.)
        // NOTE: this code is similar (not identical) in EntityMenu as in QuickGroupMenu: if one changes,
        // THE OTHER MIGHT ALSO NEED MAINTENANCE!
        val choices = Array[String]("keep existing (same as ESC)")
        // says 'same screenful' because (see similar cmt elsewhere).
        val leadingText: Array[String] = Array("CHOOSE an attribute to highlight (*)")
        controller.addRemainingCountToPrompt(choices, attributeTuples.length, entityIn.getAttrCount, attributeRowsStartingIndexIn)
        val response = ui.askWhich(Some(leadingText), choices, attributeDisplayStrings, highlightIndexIn = highlightedIndexInObjList,
                                   secondaryHighlightIndexIn = moveTargetIndexInObjList)
        val entryToHighlight: Option[Attribute] = {
          if (response.isEmpty || response.get == 1) highlightedEntry
          else {
            // those in the condition are 1-based, not 0-based.
            // user typed a letter to select an attribute (now 0-based):
            val choicesIndex = response.get - choices.length - 1
            Some(attributeTuples(choicesIndex)._2)
          }
        }
        entityMenu(entityIn, attributeRowsStartingIndexIn, entryToHighlight, targetForMoves, containingRelationToEntityIn, containingGroupIn)
      } else if (answer == 9 && answer <= choices.length) {
        new OtherEntityMenu(ui, db, controller).otherEntityMenu(entityIn, attributeRowsStartingIndexIn, relationSourceEntity, containingRelationToEntityIn,
                                                                containingGroupIn, classDefiningEntityId, attributeTuples)
        if (! db.entityKeyExists(entityIn.getId, includeArchived = false)) {
          // entity could have been deleted by some operation in OtherEntityMenu
          None
        } else {
          val listEntryIsGoneNow: Boolean = highlightedEntry.isDefined && !db.attributeKeyExists(highlightedEntry.get.getFormId, highlightedEntry.get.getId)
          val defaultEntryToHighlight: Option[Attribute] = highlightedEntry
          val nextToHighlight: Option[Attribute] = determineNextEntryToHighlight(entityIn, attributeRowsStartingIndexIn, targetForMovesIn,
                                                                                 containingRelationToEntityIn, containingGroupIn, attributesToDisplay,
                                                                                 listEntryIsGoneNow, defaultEntryToHighlight, highlightedIndexInObjList)
          entityMenu(new Entity(db, entityIn.getId), attributeRowsStartingIndexIn, nextToHighlight, targetForMovesIn,
                     containingRelationToEntityIn, containingGroupIn)
        }
      } else if (answer > choices.length && answer <= (choices.length + attributeTuples.length)) {
        // checking above for " && answer <= choices.length" because otherwise choosing 'a' returns 8 but if those optional menu choices were not added in,
        // then it is found among the first "choice" answers, instead of being adjusted later ("val attributeChoicesIndex = answer - choices.length - 1")
        // to find it among the "moreChoices" as it should be: would be thrown off by the optional choice numbering.

        // those in the condition are 1-based, not 0-based.
        // lets user go to an entity or group quickly (1 stroke)
        val choicesIndex = answer - choices.length - 1
        val entryIsGoneNow = goToSelectedAttribute(answer, choicesIndex, attributeTuples, entityIn)
        if (!db.entityKeyExists(entityIn.getId, includeArchived = false)) {
          // (entity could have been deleted or archived while browsing among containers via submenus)
          None
        } else {
          val defaultEntryToHighlight: Option[Attribute] = Some(attributeTuples(choicesIndex)._2)
          val nextToHighlight: Option[Attribute] = determineNextEntryToHighlight(entityIn, attributeRowsStartingIndexIn, targetForMovesIn,
                                                                                 containingRelationToEntityIn, containingGroupIn, attributesToDisplay,
                                                                                 entryIsGoneNow, defaultEntryToHighlight, Some(choicesIndex))
          entityMenu(new Entity(db, entityIn.getId), attributeRowsStartingIndexIn, nextToHighlight, targetForMovesIn,
                     containingRelationToEntityIn, containingGroupIn)
        }
      } else {
        ui.displayText("invalid response")
        entityMenu(entityIn, attributeRowsStartingIndexIn, highlightedEntry, targetForMoves, containingRelationToEntityIn, containingGroupIn)
      }
    }
  } catch {
    case e: Throwable =>
      // catching Throwable instead of Exception here, because sometimes depending on how I'm running X etc I might get the InternalError
      // "Can't connect to X11 window server ...", and it's better to recover from that than to abort the app (ie, when eventually calling
      // Controller.getClipboardContent)..
      controller.handleException(e)
      val ans = ui.askYesNoQuestion("Go back to what you were doing (vs. going out)?",Some("y"))
      if (ans.isDefined && ans.get) entityMenu(entityIn, attributeRowsStartingIndexIn, highlightedAttributeIn, targetForMovesIn,
                                               containingRelationToEntityIn, containingGroupIn)
      else None
  }

  def entitySearchSubmenu(entityIn: Entity, attributeRowsStartingIndexIn: Int, containingRelationToEntityIn: Option[RelationToEntity], containingGroupIn:
  Option[Group], numAttrsInEntity: Long, attributeTuples: Array[(Long, Attribute)], highlightedEntry: Option[Attribute], targetForMoves: Option[Attribute],
                          answer: Int) {
    val searchResponse = ui.askWhich(Some(Array("Choose a search option:")), Array(if (numAttrsInEntity > 0) controller.listNextItemsPrompt else "(stub)",
                                                                                   if (numAttrsInEntity > 0) controller.listPrevItemsPrompt else "(stub)",
                                                                                   "Search related entities",
                                                                                   controller.mainSearchPrompt))
    if (searchResponse.isDefined) {
      val searchAnswer = searchResponse.get
      if (searchAnswer == 1) {
        val startingIndex: Int = getNextStartingRowsIndex(attributeTuples.length, attributeRowsStartingIndexIn, numAttrsInEntity)
        entityMenu(entityIn, startingIndex, highlightedEntry, targetForMoves, containingRelationToEntityIn, containingGroupIn)
      } else if (searchAnswer == 2) {
        ui.displayText("(Not yet implemented.)")
      } else if (searchAnswer == 3) {
        // Idea: could share some code or ideas between here and Controller.findExistingObjectByText, and perhaps others like them.  For example,
        // this doesn't yet have logic to page down through the results, but maybe for now there won't be many or it can be added later.
        // Idea: maybe we could use an abstraction to make this kind of UI work even simpler, since we do it often.
        val ans = ui.askForString(Some(Array(controller.searchPromptPart(Controller.ENTITY_TYPE))))
        if (ans.isDefined) {
          val searchString: String = ans.get
          val levelsAnswer = ui.askForString(Some(Array("Enter the # of levels to search (above 10 can take many hours)")), Some(controller.isNumeric),
                                             Some("4"))
          val levels: Int = levelsAnswer.getOrElse("4").toInt
          val entityIds: Array[Long] = db.findContainedEntityIds(new mutable.TreeSet[Long], entityIn.getId, searchString, levels,
                                                                 stopAfterAnyFound = false).toArray
          val leadingText2 = Array[String]("SEARCH RESULTS: Pick an item by letter to select:")
          // could be like if (numAttrsInEntity > 0) controller.listNextItemsPrompt else "(stub)" above, if we made the method more sophisticated to do that.
          val choices: Array[String] = Array("(stub)")
          val entityIdsTruncated: Array[Long] = {
            val numDisplayableAttributes: Int = ui.maxColumnarChoicesToDisplayAfter(leadingText2.length, choices.length, Controller.maxNameLength)
            if (entityIds.length <= numDisplayableAttributes) {
              entityIds
            } else {
              val newarray: Array[Long] = new Array(numDisplayableAttributes)
              entityIds.copyToArray(newarray, 0, numDisplayableAttributes)
              // (This is to avoid the later "require" error not far from the top of TextUI.askWhichChoiceOrItsAlternate, if there are too many
              // menu items to display. It could be done better if we implement scrolling among the attrs, similarly to the other use of
              // ui.maxColumnarChoicesToDisplayAfter above, but in a way to avoid re-doing the search each time.)
              ui.displayText("There were " + entityIds.length + " results, but truncated them to " + numDisplayableAttributes + " for display.  (If" +
                             " desired this can be improved, per the comments in the code.)")
              newarray
            }
          }
          val entityNames: Array[String] = entityIdsTruncated.toArray.map {
                                                                   case id: Long => new Entity(db, id).getName
                                                                 }
          @tailrec def showSearchResults() {
            val relatedEntitiesResult = ui.askWhich(Some(leadingText2), choices, entityNames)
            if (relatedEntitiesResult.isDefined) {
              val relatedEntitiesAnswer = relatedEntitiesResult.get
              //there might be more than we have room to show here...but...see "idea"s above.
              if (relatedEntitiesAnswer == 1 && relatedEntitiesAnswer <= choices.length) {
                // (For reason behind " && answer <= choices.size", see comment where it is used elsewhere in entityMenu.)
                ui.displayText("Nothing implemented here yet.")
              } else if (relatedEntitiesAnswer > choices.length && relatedEntitiesAnswer <= (choices.length + entityNames.length)) {
                // those in the condition on the previous line are 1-based, not 0-based.
                val index = relatedEntitiesAnswer - choices.length - 1
                val id: Long = entityIds(index)
                entityMenu(new Entity(db, id))
              }
              showSearchResults()
            }
          }
          showSearchResults()
        }
      } else if (searchAnswer == 4) {
        val selection: Option[IdWrapper] = controller.chooseOrCreateObject(None, None, None, Controller.ENTITY_TYPE)
        if (selection.isDefined) {
          entityMenu(new Entity(db, selection.get.getId))
        }
      }
    }
  }

  def determineNextEntryToHighlight(entityIn: Entity, attributeRowsStartingIndexIn: Int, targetForMovesIn: Option[Attribute],
                                       containingRelationToEntityIn: Option[RelationToEntity], containingGroupIn: Option[Group],
                                       attributesToDisplay: util.ArrayList[Attribute], entryIsGoneNow: Boolean,
                                       defaultEntryToHighlight: Option[Attribute], highlightingIndex: Option[Int]): Option[Attribute] = {
    // The entity or an attribute could have been removed or changed by navigating around various menus, so before trying to view it again,
    // confirm it exists, & (at the call to entityMenu) reread from db to refresh data for display, like public/non-public status:
    if (db.entityKeyExists(entityIn.getId, includeArchived = false)) {
      if (highlightingIndex.isDefined && entryIsGoneNow) {
        Controller.findAttributeToHighlightNext(attributesToDisplay.size, attributesToDisplay, entryIsGoneNow, highlightingIndex.get, defaultEntryToHighlight.get)
      } else {
        defaultEntryToHighlight
      }
    } else {
      None
    }
  }

  /** @return A tuple containing the newStartingDisplayIndex and whether an entry moved from being listed on this entity.
    * The parm relationSourceEntityIn is derivable from the parm containingRelationToEntityIn, but passing it in saves a db read.
    */
  def moveSelectedEntry(entityIn: Entity, startingDisplayRowIndexIn: Int, totalAttrsAvailable: Int, targetForMovesIn: Option[Attribute] = None,
                        highlightedIndexInObjListIn: Int, highlightedAttributeIn: Attribute, numObjectsToDisplayIn: Int,
                        relationSourceEntityIn: Option[Entity] = None,
                        containingRelationToEntityIn: Option[RelationToEntity] = None, containingGroupIn: Option[Group] = None): (Int, Boolean) = {
    if (relationSourceEntityIn.isDefined || containingRelationToEntityIn.isDefined) {
      require(relationSourceEntityIn.isDefined && containingRelationToEntityIn.isDefined,
              (if (relationSourceEntityIn.isEmpty) "relationSourceEntityIn is empty; " else "") +
              (if (containingRelationToEntityIn.isEmpty) "containingRelationToEntityIn is empty." else ""))
      require(relationSourceEntityIn.get.getId == containingRelationToEntityIn.get.getParentId, "relationSourceEntityIn: " + relationSourceEntityIn.get.getId +
             " doesn't match containingRelationToEntityIn.get.getParentId: " + containingRelationToEntityIn.get.getParentId + ".")
    }
    val choices = Array[String](// (see comments at similar location in same-named method of QuickGroupMenu.)
                                "Move up 25",
                                "Move up 5", "Move up 1", "Move down 1", "Move down 5",
                                "Move down 25",

                                if (targetForMovesIn.isDefined) "Move (*) to selected target (+, if any)"
                                else "(stub: have to choose a target before you can move entries into it)",

                                "Move (*) to calling menu (up one)")
    val response = ui.askWhich(None, choices, Array[String](), highlightIndexIn = Some(highlightedIndexInObjListIn))
    if (response.isEmpty) (startingDisplayRowIndexIn, false)
    else {
      val answer = response.get
      var numRowsToMove = 0
      var forwardNotBack = false
      if (answer >= 1 && answer <= 6) {
        if (answer == 1) {
          numRowsToMove = 20
        } else if (answer == 2) {
          numRowsToMove = 5
        } else if (answer == 3) {
          numRowsToMove = 1
        } else if (answer == 4) {
          numRowsToMove = 1
          forwardNotBack = true
        } else if (answer == 5) {
          numRowsToMove = 5
          forwardNotBack = true
        } else if (answer == 6) {
          numRowsToMove = 20
          forwardNotBack = true
        }
        val displayStartingRowNumber: Int = placeEntryInPosition(entityIn.getId, totalAttrsAvailable, numRowsToMove, forwardNotBackIn = forwardNotBack,
                                                                 startingDisplayRowIndexIn, highlightedAttributeIn.getId,
                                                                 highlightedIndexInObjListIn,
                                                                 Some(highlightedAttributeIn.getId),
                                                                 numObjectsToDisplayIn, highlightedAttributeIn.getFormId,
                                                                 Some(highlightedAttributeIn.getFormId))
        (displayStartingRowNumber, false)
      } else if (answer == 7 && targetForMovesIn.isDefined) {
        if (! ((highlightedAttributeIn.isInstanceOf[RelationToEntity] || highlightedAttributeIn.isInstanceOf[RelationToGroup]) &&
                (targetForMovesIn.get.isInstanceOf[RelationToEntity] || targetForMovesIn.get.isInstanceOf[RelationToGroup]))) {
          ui.displayText("Currently, you can only move an Entity or a Group, to an Entity or a Group.  Moving thus is not yet implemented for other " +
                         "attribute types, but it shouldn't take much to add that. [1]")
          (startingDisplayRowIndexIn, false)
        } else {
          //noinspection TypeCheckCanBeMatch
          if (highlightedAttributeIn.isInstanceOf[RelationToEntity] && targetForMovesIn.get.isInstanceOf[RelationToEntity]) {
            val movingRte = highlightedAttributeIn.asInstanceOf[RelationToEntity]
            val targetContainingEntityId = targetForMovesIn.get.asInstanceOf[RelationToEntity].getRelatedId2
            require(movingRte.getParentId == entityIn.getId)
            db.moveRelationToEntity(movingRte.getId, targetContainingEntityId, getSortingIndex(entityIn.getId, movingRte.getFormId, movingRte.getId))
            (startingDisplayRowIndexIn, true)
          } else if (highlightedAttributeIn.isInstanceOf[RelationToEntity] && targetForMovesIn.get.isInstanceOf[RelationToGroup]) {
            require(targetForMovesIn.get.getFormId == PostgreSQLDatabase.getAttributeFormId(Controller.RELATION_TO_GROUP_TYPE))
            val targetGroupId = RelationToGroup.createRelationToGroup(db, targetForMovesIn.get.getId).getGroupId
            val rte = highlightedAttributeIn.asInstanceOf[RelationToEntity]
            // about the sortingIndex:  see comment on db.moveEntityFromEntityToGroup.
            db.moveEntityFromEntityToGroup(rte, targetGroupId, getSortingIndex(entityIn.getId, rte.getFormId, rte.getId))
            (startingDisplayRowIndexIn, true)
          } else if (highlightedAttributeIn.isInstanceOf[RelationToGroup] && targetForMovesIn.get.isInstanceOf[RelationToEntity]) {
            val movingRtg = highlightedAttributeIn.asInstanceOf[RelationToGroup]
            val newContainingEntityId = targetForMovesIn.get.asInstanceOf[RelationToEntity].getRelatedId2
            require(movingRtg.getParentId == entityIn.getId)
            db.moveRelationToGroup(movingRtg.getId, newContainingEntityId, getSortingIndex(entityIn.getId, movingRtg.getFormId, movingRtg.getId))
            (startingDisplayRowIndexIn, true)
          } else if (highlightedAttributeIn.isInstanceOf[RelationToGroup] && targetForMovesIn.get.isInstanceOf[RelationToGroup]) {
            ui.displayText("Can't do this: groups can't directly contain groups.  But groups can contain entities, and entities can contain groups and" +
                           " other attributes. [1]")
            (startingDisplayRowIndexIn, false)
          } else {
            (startingDisplayRowIndexIn, false)
          }
        }
      } else if (answer == 8) {
        if (! (highlightedAttributeIn.isInstanceOf[RelationToEntity] || highlightedAttributeIn.isInstanceOf[RelationToGroup])) {
          ui.displayText("Currently, you can only move an Entity or a Group, *to* an Entity or a Group.  Moving thus is not yet implemented for other " +
                         "attribute types, but it shouldn't take much to add that. [2]")
          (startingDisplayRowIndexIn, false)
        } else {
          if (containingRelationToEntityIn.isDefined) {
            require(containingGroupIn.isEmpty)
            val newContainingEntityId = containingRelationToEntityIn.get.getRelatedId1
            //noinspection TypeCheckCanBeMatch
            if (highlightedAttributeIn.isInstanceOf[RelationToEntity]) {
              val movingRte = highlightedAttributeIn.asInstanceOf[RelationToEntity]
              db.moveRelationToEntity(movingRte.getId, newContainingEntityId, getSortingIndex(entityIn.getId, movingRte.getFormId, movingRte.getId))
              (startingDisplayRowIndexIn, true)
            } else if (highlightedAttributeIn.isInstanceOf[RelationToGroup]) {
              val movingRtg = highlightedAttributeIn.asInstanceOf[RelationToGroup]
              db.moveRelationToGroup(movingRtg.getId, newContainingEntityId, getSortingIndex(entityIn.getId, movingRtg.getFormId, movingRtg.getId))
              (startingDisplayRowIndexIn, true)
            } else throw new OmException("Should be impossible to get here: I thought I checked for ok values, above. [1]")
          } else if (containingGroupIn.isDefined) {
            require(containingRelationToEntityIn.isEmpty)
            //noinspection TypeCheckCanBeMatch
            if (highlightedAttributeIn.isInstanceOf[RelationToEntity]) {
              val targetGroupId = containingGroupIn.get.getId
              val rte = highlightedAttributeIn.asInstanceOf[RelationToEntity]
              // about the sortingIndex:  see comment on db.moveEntityFromEntityToGroup.
              db.moveEntityFromEntityToGroup(rte, targetGroupId, getSortingIndex(entityIn.getId, rte.getFormId, rte.getId))
              (startingDisplayRowIndexIn, true)
            } else if (highlightedAttributeIn.isInstanceOf[RelationToGroup]) {
              ui.displayText("Can't do this: groups can't directly contain groups.  But groups can contain entities, and entities can contain groups and" +
                             " other attributes. [2]")
              (startingDisplayRowIndexIn, false)
            } else throw new OmException("Should be impossible to get here: I thought I checked for ok values, above. [2]")
          } else {
            ui.displayText("One of the container parameters needs to be available, in order to move the highlighted attribute to the containing entity or group" +
                           " (the one from which you navigated here).")
            (startingDisplayRowIndexIn, false)
          }
        }
      } else {
        (startingDisplayRowIndexIn, false)
      }
    }
  }

  def getLeadingText(leadingTextIn: Array[String], numAttributes: Int,
                     entityIn: Entity, relationSourceEntityIn: Option[Entity] = None,
                     relationIn: Option[RelationToEntity] = None, containingGroupIn: Option[Group] = None): Array[String] = {
    leadingTextIn(0) = controller.entityMenuLeadingText(entityIn)
    if (containingGroupIn.isDefined) {
      leadingTextIn(0) += ": found via group: " + containingGroupIn.get.getName
    }
    leadingTextIn(0) += ": created " + entityIn.getCreationDateFormatted
    leadingTextIn(1) = if (numAttributes == 0) "No attributes have been assigned to this object, yet."
    else "Attribute list menu: (or choose attribute by letter)"
    leadingTextIn
  }

  def getItemDisplayStringsAndAttrs(attributeTuples: Array[(Long, Attribute)]): (Array[String], util.ArrayList[Attribute]) = {
    val attributes = new util.ArrayList[Attribute]
    val attributeNames: Array[String] =
      for (attributeTuple <- attributeTuples) yield {
        val attribute = attributeTuple._2
        attributes.add(attribute)
        attribute match {
        case relation: RelationToEntity =>
          val toEntity: Entity = new Entity(db, relation.getRelatedId2)
          val relationType = new RelationType(db, relation.getAttrTypeId)
          val desc = attribute.getDisplayString(Controller.maxNameLength, Some(toEntity), Some(relationType), simplify = true)
          val prefix = controller.getEntityContentSizePrefix(relation.getRelatedId2)
          prefix + desc + controller.getPublicStatusDisplayString(toEntity)
        case relation: RelationToGroup =>
          val relationType = new RelationType(db, relation.getAttrTypeId)
          val desc = attribute.getDisplayString(Controller.maxNameLength, None, Some(relationType), simplify = true)
          val prefix = controller.getGroupContentSizePrefix(relation.getGroupId)
          prefix + "group: " + desc
        case _ =>
          attribute.getDisplayString(Controller.maxNameLength, None, None)
      }
    }
    (attributeNames, attributes)
  }

  def addAttribute(entityIn: Entity, startingAttributeIndexIn: Int, highlightedAttributeIn: Option[Attribute], targetForMovesIn: Option[Attribute] = None,
                   containingRelationToEntityIn: Option[RelationToEntity] = None, containingGroupIn: Option[Group] = None): Option[Attribute] = {
    val whichKindOfAttribute =
      ui.askWhich(Some(Array("Choose which kind of attribute to add:")),
                  // THESE ARRAY INDICES (after being converted by askWhich to 1-based) MUST MATCH THOSE LISTED IN THE MATCH STATEMENT
                  // JUST BELOW. See the comment there.
                  Array("Relation to entity (i.e., \"is near\" a microphone, complete menu)",
                        "Relation to existing entity: quick search by name (uses \"has\" relation)",
                        "quantity attribute (example: a numeric value like \"length\"",
                        "date",
                        "true/false value",

                        "external file (to be captured in OM; BUT CONSIDER FIRST ADDING AN ENTITY SPECIFICALLY FOR THE DOCUMENT SO IT CAN HAVE A DATE, " +
                        "OTHER ATTRS ETC.; AND ADDING THE DOCUMENT TO THAT ENTITY, SO IT CAN ALSO BE ASSOCIATED WITH OTHER ENTITIES EASILY!; also, " +
                        "given the concept behind OM, it's probably best" +
                        " to use this only for historical artifacts, or when you really can't fully model the data right now)",

                        "text attribute (rare: usually prefer relations; but for example: a serial number, which is not subject to arithmetic, or a quote)",
                        "Relation to group (i.e., \"has\" a list/group)",
                        "external web page (or other URI, to refer to external information and optionally quote it)")
                 )
    if (whichKindOfAttribute.isDefined) {
      val attrForm: Int = whichKindOfAttribute.get match {
        // This is a bridge between the expected order for convenient UI above, and the parameter value expected by controller.addAttribute
        // (1-based, not 0-based.)
        case 1 => PostgreSQLDatabase.getAttributeFormId(Controller.RELATION_TO_ENTITY_TYPE)
        case 2 => 100
        case 3 => PostgreSQLDatabase.getAttributeFormId(Controller.QUANTITY_TYPE)
        case 4 => PostgreSQLDatabase.getAttributeFormId(Controller.DATE_TYPE)
        case 5 => PostgreSQLDatabase.getAttributeFormId(Controller.BOOLEAN_TYPE)
        case 6 => PostgreSQLDatabase.getAttributeFormId(Controller.FILE_TYPE)
        case 7 => PostgreSQLDatabase.getAttributeFormId(Controller.TEXT_TYPE)
        case 8 => PostgreSQLDatabase.getAttributeFormId(Controller.RELATION_TO_GROUP_TYPE)
        case 9 => 101
      }
      controller.addAttribute(entityIn, startingAttributeIndexIn, attrForm, None)
    } else {
      None
    }
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

  /**
   * @return whether the entry in question was deleted (or archived)
   */
  def goToSelectedAttribute(answer: Int, choiceIndex: Int, attributeTuples: Array[(Long, Attribute)], entityIn: Entity): Boolean = {
    // user typed a letter to select an attribute (now 0-based)
    if (choiceIndex >= attributeTuples.length) {
      ui.displayText("The program shouldn't have let us get to this point, but the selection " + answer + " is not in the list.")
      false
    } else {
      val o: Attribute = attributeTuples(choiceIndex)._2
      o match {
        //idea: there's probably also some more scala-like cleaner syntax 4 this, as elsewhere:
        case qa: QuantityAttribute => controller.attributeEditMenu(qa)
        case da: DateAttribute => controller.attributeEditMenu(da)
        case ba: BooleanAttribute => controller.attributeEditMenu(ba)
        case fa: FileAttribute => controller.attributeEditMenu(fa)
        case ta: TextAttribute => controller.attributeEditMenu(ta)
        case relToEntity: RelationToEntity =>
          entityMenu(new Entity(db, relToEntity.getRelatedId2), 0, None, None, Some(relToEntity))
          val wasRemoved: Boolean = !( db.entityKeyExists(relToEntity.getRelatedId2, includeArchived = false)
                                       && db.attributeKeyExists(relToEntity.getFormId, relToEntity.getId) )
          wasRemoved
        case relToGroup: RelationToGroup =>
          new QuickGroupMenu(ui, db, controller).quickGroupMenu(new Group(db, relToGroup.getGroupId), 0, Some(relToGroup), containingEntityIn = Some(entityIn))
          if (!db.groupKeyExists(relToGroup.getGroupId)) true
          else false
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
    db.renumberSortingIndexes(entityIdIn)
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
