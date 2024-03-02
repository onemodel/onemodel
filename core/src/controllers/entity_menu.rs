/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003-2004, 2008-2020 inclusive, and 2023, Luke A. Call.
    (That copyright statement was previously 2013-2015, until I remembered that much of Controller came from TextUI.scala and TextUI.java before that.)
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
struct EntityMenu {
/*%%
package org.onemodel.core.controllers

import java.util

import org.onemodel.core._
import org.onemodel.core.model._

import scala.annotation.tailrec
import scala.collection.mutable

class EntityMenu(override let ui: TextUI, val controller: Controller) extends SortableEntriesMenu(ui) {;
  /** The parameter attributeRowsStartingIndexIn means: of all the sorted attributes of entity_in, which one is to be displayed first (since we can only display
    * so many at a time with finite screen size).
    * Returns None if user wants out (or if entity was deleted so we should exit to containing menu).
    * */
  //@tailrec //removed for now until the compiler can handle it with where the method calls itself.
  //idea on scoping: make this limited like this somehow?:  private[org.onemodel] ... Same for all others like it?
  fn entityMenu(entity_in: Entity, attributeRowsStartingIndexIn: Int = 0, highlightedAttributeIn: Option[Attribute] = None,
                 targetForMovesIn: Option[Attribute] = None,
                 //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) w/in this method!
                 containingRelationToEntityIn: Option[AttributeWithValidAndObservedDates] = None,
                 containingGroupIn: Option[Group] = None) -> Option<Entity> {
    require(containingRelationToEntityIn.isEmpty ||
            containingRelationToEntityIn.get.isInstanceOf[RelationToLocalEntity] || containingRelationToEntityIn.get.isInstanceOf[RelationToRemoteEntity])
    require(entity_in != null)
    if !entity_in.db.entity_key_exists(entity_in.get_id, include_archived = entity_in.db.include_archived_entities)) {
      ui.display_text("The desired entity, " + entity_in.get_id + ", has been deleted or archived, probably while browsing other entities via menu options," +
                     "and so cannot be displayed here.  Exiting to the next menu.")
      return None
    }
    let (containingRelationToEntityIn_relatedId1: Option<i64>, containingRelationToEntityIn_relatedId2: Option<i64>) = {;
      if containingRelationToEntityIn.is_defined) {
        //noinspection TypeCheckCanBeMatch
        if containingRelationToEntityIn.get.isInstanceOf[RelationToRemoteEntity]) {
          let rtre = containingRelationToEntityIn.get.asInstanceOf[RelationToRemoteEntity];
          (Some(rtre.get_related_id1), Some(rtre.get_related_id2))
        } else {
          let rtle = containingRelationToEntityIn.get.asInstanceOf[RelationToLocalEntity];
          (Some(rtle.get_related_id1), Some(rtle.get_related_id2))
        }
      } else {
        (None, None)
      }
    }
    if containingRelationToEntityIn.is_defined) {
      // (doesn't make sense to have both at the same time.)
      require(containingGroupIn.isEmpty)
      require(containingRelationToEntityIn_relatedId2.get == entity_in.get_id)
    }
    if containingGroupIn.is_defined) require(containingRelationToEntityIn.isEmpty)
    let numAttrsInEntity: i64 = entity_in.get_attribute_count();
    let leading_text: Vec<String> = new Vec<String>(2);
    let relationSourceEntity: Option<Entity> = {;
      // (checking if exists also, because it could have been removed in another menu option)
      if containingRelationToEntityIn.isEmpty || !containingRelationToEntityIn.get.db.entity_key_exists(containingRelationToEntityIn_relatedId1.get)) {
        None
      } else {
        Some(new Entity(containingRelationToEntityIn.get.db, containingRelationToEntityIn.get.get_parent_id()))
      }
    }
    let choices: Vec<String> = getChoices(entity_in, numAttrsInEntity);
    let numDisplayableAttributes: i32 = ui.maxColumnarChoicesToDisplayAfter(leading_text.length, choices.length, Util.maxNameLength);
    let (attributeTuples: Array[(i64, Attribute)], totalAttrsAvailable: Int) =;
      entity_in.get_sorted_attributes(attributeRowsStartingIndexIn, numDisplayableAttributes, only_public_entities_in = false)
    if (numAttrsInEntity > 0 && attributeRowsStartingIndexIn == 0) || attributeTuples.length > 0) {
      require(numAttrsInEntity > 0 && attributeTuples.length > 0)
    }
    Util.add_remaining_count_to_prompt(choices, attributeTuples.length, totalAttrsAvailable, attributeRowsStartingIndexIn)
    let leading_textModified = getLeadingText(leading_text, attributeTuples.length, entity_in, containingGroupIn);
    let (attributeDisplayStrings: Vec<String>, attributesToDisplay: util.ArrayList[Attribute]) = getItemDisplayStringsAndAttrs(attributeTuples);

    // The variable highlightedIndexInObjList means: of the sorted attributes selected *for display* (potentially fewer than all existing attributes),
    // this is the zero-based index of the one that is marked for possible moving around in the sorted order (in the UI, marked as selected, relative
    // to those displayed, not to all).
    let (highlightedIndexInObjList: Option[Int], highlightedEntry: Option[Attribute], moveTargetIndexInObjList: Option[Int],;
    targetForMoves: Option[Attribute]) = {
      if attributeTuples.length == 0) {
        (None, None, None, None)
      } else {
        let mut highlightedEntry: Option[Attribute] = Some(highlightedAttributeIn.getOrElse(attributeTuples(0)._2));
        let highlightedObjFormId: i32 = highlightedEntry.get.get_form_id;
        let highlightedObjId: i64 = highlightedEntry.get.get_id;
        let mut highlightedIndexInObjList: Option[Int] = None;
        let mut moveTargetIndexInObjList: Option[Int] = None;
        let mut targetForMoves: Option[Attribute] = None;
        let mut index = -1;
        for (attributeTuple <- attributeTuples) {
          index += 1
          let attribute = attributeTuple._2;
          if attribute.get_form_id == highlightedObjFormId && attribute.get_id == highlightedObjId) {
            highlightedIndexInObjList = Some(index)
          }
          if targetForMovesIn.is_defined && attribute.get_form_id == targetForMovesIn.get.get_form_id && attribute.get_id == targetForMovesIn.get.get_id) {
            moveTargetIndexInObjList = Some(index)
            targetForMoves = targetForMovesIn
          }
        }
        // if we got to this point, it could simply have been deleted or something (probably) but still passed in the highlightedAttributeIn parm by mistake,
        // so just return something safe (instead of throwing an exception, as in a previous commit):
        if highlightedIndexInObjList.isEmpty) {
          // maybe the highlightedAttributeIn was defined but not found in the list for some unknown reason, so at least recover nicely:
          highlightedIndexInObjList = Some(0)
          highlightedEntry = Some(attributeTuples(0)._2)
        }
        if moveTargetIndexInObjList.is_defined && highlightedIndexInObjList.get == moveTargetIndexInObjList.get) {
          // doesn't make sense if they're the same (ie move both, into both?, like if user changed the previous highlight on 1st selection to a move
          // target), so change one:
          if highlightedIndexInObjList.get == 0 && attributeTuples.length > 1) {
            let indexToUseInstead = 1;
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
      if highlightedEntry.is_defined && Util.can_edit_attribute_on_single_line(highlightedEntry.get)) {
        // (the next line's display text is abbreviated to fit in an 80-column terminal window:)
        "Edit the selected attribute's content (single line; go into attr for more)"
      } else "Edit entity name"

    // MAKE SURE this next condition always is the opposite of the one at comment mentioning "choices(4) = ..." below
    if highlightedIndexInObjList.isEmpty) {
      choices(4) = "(stub)"
    }

    let response = ui.ask_which(Some(leading_textModified), choices, attributeDisplayStrings, highlightIndexIn = highlightedIndexInObjList,;
                               secondaryHighlightIndexIn = moveTargetIndexInObjList)
    if response.isEmpty) None
    else {
      let answer = response.get;
      if answer == 1) {
        let (newAttributeToHighlight: Option[Attribute], displayStartingRowNumber: Int) = {;
          // ask for less info when here, to add entity quickly w/ no fuss, like brainstorming. Like in QuickGroupMenu.  User can always use option 2.
          let newEntity: Option<Entity> = controller.askForNameAndWriteEntity(entity_in.db, Util.ENTITY_TYPE, leading_text_in = Some("NAME THE ENTITY:"));
          if newEntity.is_defined) {
            let newAttribute: Attribute = entity_in.add_has_RelationToLocalEntity(newEntity.get.get_id, None, System.currentTimeMillis());
            // The next 2 lines are so if adding a new entry on the 1st entry, and if the user so prefers, the new one becomes the
            // first entry (common for logs/jnl w/ latest first), otherwise the new entry is placed after the current entry.
            let goingBackward: bool = highlightedIndexInObjList.getOrElse(0) == 0 && entity_in.get_new_entries_stick_to_top;
            let forward = !goingBackward;
            let displayStartingRowNumber: i32 = placeEntryInPosition(entity_in.db, entity_in.get_id, entity_in.get_attribute_count(), 0, forward_not_back_in = forward,;
                                                                     attributeRowsStartingIndexIn, newAttribute.get_id,
                                                                     highlightedIndexInObjList.getOrElse(0),
                                                                     if highlightedEntry.is_defined) Some(highlightedEntry.get.get_id) else None,
                                                                     numDisplayableAttributes, newAttribute.get_form_id,
                                                                     if highlightedEntry.is_defined) Some(highlightedEntry.get.get_form_id) else None)
            controller.defaultAttributeCopying(newEntity.get, Some(attributeTuples))
            (Some(newAttribute), displayStartingRowNumber)
          }
          else (highlightedEntry, attributeRowsStartingIndexIn)
        }
        entityMenu(entity_in, displayStartingRowNumber, newAttributeToHighlight, targetForMoves, containingRelationToEntityIn, containingGroupIn)
      } else if answer == 2 && highlightedEntry.is_defined && highlightedIndexInObjList.is_defined && numAttrsInEntity > 0) {
        let (newStartingDisplayIndex: Int, movedOneOut: bool) = moveSelectedEntry(entity_in, attributeRowsStartingIndexIn, totalAttrsAvailable,;
                                                                                     targetForMoves,
                                                                                     highlightedIndexInObjList.get, highlightedEntry.get,
                                                                                     numDisplayableAttributes,
                                                                                     relationSourceEntity,
                                                                                     containingRelationToEntityIn, containingGroupIn)
        let attrToHighlight: Option[Attribute] = Util.find_attribute_to_highlight_next(attributeTuples.length, attributesToDisplay, removedOne = movedOneOut,;
                                                                                   highlightedIndexInObjList.get, highlightedEntry.get)
        entityMenu(entity_in, newStartingDisplayIndex, attrToHighlight, targetForMoves, containingRelationToEntityIn, containingGroupIn)
      } else if answer == 3) {
        // MAKE SURE this next condition always matches the one in "choices(2) = ..." above
        if highlightedEntry.is_defined && Util.can_edit_attribute_on_single_line(highlightedEntry.get)) {
          controller.editAttributeOnSingleLine(highlightedEntry.get)
          entityMenu(entity_in, attributeRowsStartingIndexIn, highlightedEntry, targetForMoves, containingRelationToEntityIn, containingGroupIn)
        } else {
          let editedEntity: Option<Entity> = controller.editEntityName(entity_in);
          entityMenu(if editedEntity.is_defined) editedEntity.get else entity_in,
                     attributeRowsStartingIndexIn, highlightedEntry, targetForMoves, containingRelationToEntityIn, containingGroupIn)
        }
      } else if answer == 4) {
        let newAttribute: Option[Attribute] = addAttribute(entity_in, attributeRowsStartingIndexIn, highlightedEntry, targetForMoves, containingGroupIn);
        if newAttribute.is_defined && highlightedEntry.is_defined) {
          // (See comment at similar place in EntityMenu, just before that call to placeEntryInPosition.)
          let goingBackward: bool = highlightedIndexInObjList.getOrElse(0) == 0 && entity_in.get_new_entries_stick_to_top;
          let forward = !goingBackward;
          placeEntryInPosition(entity_in.db, entity_in.get_id, entity_in.get_attribute_count(), 0, forward_not_back_in = forward, attributeRowsStartingIndexIn,
                               newAttribute.get.get_id, highlightedIndexInObjList.getOrElse(0),
                               if highlightedEntry.is_defined) Some(highlightedEntry.get.get_id) else None,
                               numDisplayableAttributes, newAttribute.get.get_form_id,
                               if highlightedEntry.is_defined) Some(highlightedEntry.get.get_form_id) else None)
          entityMenu(entity_in, attributeRowsStartingIndexIn, newAttribute, targetForMoves, containingRelationToEntityIn, containingGroupIn)
        } else {
          entityMenu(entity_in, attributeRowsStartingIndexIn, highlightedEntry, targetForMoves, containingRelationToEntityIn, containingGroupIn)
        }
      } else if answer == 5) {
        // MAKE SURE this next condition always is the exact opposite of the one in "choices(4) = ..." above (4 vs. 5 because they are 0- vs. 1-based)
        if highlightedIndexInObjList.is_defined) {
          goToAttributeThenRedisplayHere(entity_in, attributeRowsStartingIndexIn, targetForMovesIn, containingRelationToEntityIn, containingGroupIn,
                                         attributeTuples, attributesToDisplay, answer, highlightedIndexInObjList.get)
        } else {
          ui.display_text("nothing selected")
          entityMenu(entity_in, attributeRowsStartingIndexIn, highlightedEntry, targetForMovesIn, containingRelationToEntityIn, containingGroupIn)
        }
      } else if answer == 6) {
        entitySearchSubmenu(entity_in, attributeRowsStartingIndexIn, containingRelationToEntityIn, containingGroupIn, numAttrsInEntity, attributeTuples,
                            highlightedEntry, targetForMoves, answer)
        entityMenu(entity_in, attributeRowsStartingIndexIn, highlightedEntry, targetForMoves, containingRelationToEntityIn, containingGroupIn)
      } else if answer == 7) {
        // NOTE: this code is similar (not identical) in EntityMenu as in QuickGroupMenu: if one changes,
        // THE OTHER MIGHT ALSO NEED MAINTENANCE!
        let choices = Vec<String>(Util.UNSELECT_MOVE_TARGET_PROMPT_TEXT);
        let leading_text: Vec<String> = Array(Util.UNSELECT_MOVE_TARGET_LEADING_TEXT);
        Util.add_remaining_count_to_prompt(choices, attributeTuples.length, entity_in.get_attribute_count(), attributeRowsStartingIndexIn)

        let response = ui.ask_which(Some(leading_text), choices, attributeDisplayStrings, highlightIndexIn = highlightedIndexInObjList,;
                                   secondaryHighlightIndexIn = moveTargetIndexInObjList)
        let (entryToHighlight, selectedTargetAttribute): (Option[Attribute], Option[Attribute]) = {;
          if response.isEmpty) (highlightedEntry, targetForMoves)
          else {
            let answer = response.get;
            if answer == 1) {
              (highlightedEntry, None)
            } else {
              // those in the condition are 1-based, not 0-based.
              // user typed a letter to select an attribute (now 0-based):
              let selectionIndex: i32 = answer - choices.length - 1;
              let userSelection: Attribute = attributeTuples(selectionIndex)._2;
              if selectionIndex == highlightedIndexInObjList.get) {
                // chose same entity for the target, as the existing highlighted selection, so make it the target, and no highlighted one.
                (None, Some(userSelection))
              } else {
                (highlightedEntry, Some(userSelection))
              }
            }
          }
        }
        entityMenu(entity_in, attributeRowsStartingIndexIn, entryToHighlight, selectedTargetAttribute, containingRelationToEntityIn, containingGroupIn)
      } else if answer == 8 && answer <= choices.length && numAttrsInEntity > 0) {
        // lets user select an attribute for further operations like moving, deleting.
        // (we have to have at least one choice or ui.ask_which fails...a require() call there.)
        // NOTE: this code is similar (not identical) in EntityMenu as in QuickGroupMenu: if one changes,
        // THE OTHER MIGHT ALSO NEED MAINTENANCE!
        let choices = Vec<String>("keep existing (same as ESC)");
        // says 'same screenful' because (see similar cmt elsewhere).
        let leading_text: Vec<String> = Array("CHOOSE an attribute to highlight (*)");
        Util.add_remaining_count_to_prompt(choices, attributeTuples.length, entity_in.get_attribute_count(), attributeRowsStartingIndexIn)
        let response = ui.ask_which(Some(leading_text), choices, attributeDisplayStrings, highlightIndexIn = highlightedIndexInObjList,;
                                   secondaryHighlightIndexIn = moveTargetIndexInObjList)
        let entryToHighlight: Option[Attribute] = {;
          if response.isEmpty || response.get == 1) highlightedEntry
          else {
            // those in the condition are 1-based, not 0-based.
            // user typed a letter to select an attribute (now 0-based):
            let choices_index = response.get - choices.length - 1;
            Some(attributeTuples(choices_index)._2)
          }
        }
        entityMenu(entity_in, attributeRowsStartingIndexIn, entryToHighlight, targetForMoves, containingRelationToEntityIn, containingGroupIn)
      } else if answer == 9 && answer <= choices.length) {
        new OtherEntityMenu(ui, controller).otherEntityMenu(entity_in, attributeRowsStartingIndexIn, relationSourceEntity, containingRelationToEntityIn,
                                                            containingGroupIn, attributeTuples)
        if !entity_in.db.entity_key_exists(entity_in.get_id, include_archived = false)) {
          // entity could have been deleted by some operation in OtherEntityMenu
          None
        } else {
          let listEntryIsGoneNow: bool = highlightedEntry.is_defined &&;
                                            !highlightedEntry.get.db.attribute_key_exists(highlightedEntry.get.get_form_id, highlightedEntry.get.get_id)
          let defaultEntryToHighlight: Option[Attribute] = highlightedEntry;
          let nextToHighlight: Option[Attribute] = determineNextEntryToHighlight(entity_in, attributesToDisplay,;
                                                                                 listEntryIsGoneNow, defaultEntryToHighlight, highlightedIndexInObjList)
          entityMenu(new Entity(entity_in.db, entity_in.get_id), attributeRowsStartingIndexIn, nextToHighlight, targetForMovesIn,
                     containingRelationToEntityIn, containingGroupIn)
        }
      } else if answer > choices.length && answer <= (choices.length + attributeTuples.length)) {
        // checking above for " && answer <= choices.length" because otherwise choosing 'a' returns 8 but if those optional menu choices were not added in,
        // then it is found among the first "choice" answers, instead of being adjusted later ("val attributeChoicesIndex = answer - choices.length - 1")
        // to find it among the "moreChoices" as it should be: would be thrown off by the optional choice numbering.

        // those in the condition are 1-based, not 0-based.
        // lets user go to an entity or group quickly (1 stroke)
        let choices_index: i32 = answer - choices.length - 1;
        goToAttributeThenRedisplayHere(entity_in, attributeRowsStartingIndexIn, targetForMovesIn, containingRelationToEntityIn, containingGroupIn,
                                       attributeTuples, attributesToDisplay, answer, choices_index)
      } else {
        ui.display_text("invalid response")
        entityMenu(entity_in, attributeRowsStartingIndexIn, highlightedEntry, targetForMoves, containingRelationToEntityIn, containingGroupIn)
      }
    }
  } catch {
    case e: Throwable =>
      // catching Throwable instead of Exception here, because sometimes depending on how I'm running X etc I might get the InternalError
      // "Can't connect to X11 window server ...", and it's better to recover from that than to abort the app (ie, when eventually calling
      // Controller.get_clipboard_content)..
      // Idea: somehow make this handle it right, even if the exception came from a remote db (rest)?
      Util.handleException(e, ui, entity_in.db)
      let ans = ui.ask_yes_no_question("Go back to what you were doing (vs. going out)?", Some("y"));
      if ans.is_defined && ans.get) entityMenu(entity_in, attributeRowsStartingIndexIn, highlightedAttributeIn, targetForMovesIn,
                                               containingRelationToEntityIn, containingGroupIn)
      else None
  }

  // 2nd return value is whether entityIsDefault (ie whether default object when launching OM is already this entity)
    fn getChoices(entity_in: Entity, numAttrsIn: i64) -> Vec<String> {
    // (idea: might be a little silly to do it this way, once this # gets very big?:)
    let mut choices = Vec<String>("Add entry quickly (creates a \"has\" relation to a new Entity)",;
                                if numAttrsIn > 0) "Move selection (*) up/down" else "(stub)",

                                "[app will fill this one in just a bit later, at \"choices (3) = \" below.  KEEP IT IN THIS RELATIVE POSITION OR CHANGE THE" +
                                " CODE NEAR THE TOP OF entityMenu THAT CHECKS FOR A VALUE IN highlightedAttributeIn]",

                                "Add attribute (add entry with detailed options)",
                                "Go to selected attribute",
                                "Search / List next ...")
    // (the next line's display text is abbreviated to fit in an 80-column terminal window:)
    choices = choices :+ "Select target (entry move destination: gets a '+' marker)"
    // (the next line's display text is abbreviated to fit in an 80-column terminal window:)
    choices = choices :+ (if numAttrsIn > 0) "Select attribute to highlight (with '*'; type a letter to go to its attr menu)" else "(stub)")
    choices = choices :+ (if controller.get_default_entity.isEmpty && !entity_in.db.is_remote) "****TRY ME---> " else "") + "Other entity operations..."
    choices
  }

    fn goToAttributeThenRedisplayHere(entity_in: Entity, attributeRowsStartingIndexIn: Int, targetForMovesIn: Option[Attribute],
                                     containingRelationToEntityIn: Option[AttributeWithValidAndObservedDates], containingGroupIn: Option[Group],
                                     attributeTuples: Array[(i64, Attribute)], attributesToDisplay: util.ArrayList[Attribute],
                                     answer: Int, choices_index: Int) -> Option<Entity> {
    require(containingRelationToEntityIn.isEmpty ||
            containingRelationToEntityIn.get.isInstanceOf[RelationToLocalEntity] || containingRelationToEntityIn.get.isInstanceOf[RelationToRemoteEntity])
    let entryIsGoneNow = {;
      // user typed a letter to select an attribute (now 0-based)
      if choices_index >= attributeTuples.length) {
        ui.display_text("The program shouldn't have let us get to this point, but the selection " + answer + " is not in the list.")
        false
      } else {
        let o: Attribute = attributeTuples(choices_index)._2;
        o match {
          //idea: there's probably also some more scala-like cleaner syntax 4 this, as elsewhere:
          case qa: QuantityAttribute => controller.attributeEditMenu(qa)
          case da: DateAttribute => controller.attributeEditMenu(da)
          case ba: BooleanAttribute => controller.attributeEditMenu(ba)
          case fa: FileAttribute => controller.attributeEditMenu(fa)
          case ta: TextAttribute => controller.attributeEditMenu(ta)
          case relToEntity: RelationToLocalEntity =>
            let db = relToEntity.db;
            entityMenu(new Entity(db, relToEntity.get_related_id2), 0, None, None, Some(relToEntity))
            let stillThere: bool = db.entity_key_exists(relToEntity.get_related_id2, include_archived = false) &&;
                                      db.attribute_key_exists(relToEntity.get_form_id, relToEntity.get_id)
            !stillThere
          case relToRemoteEntity: RelationToRemoteEntity =>
            // (An entity can be remote, but referred to by a local RelationToLocalEntity:)
            let remoteDb: Database = relToRemoteEntity.getRemoteDatabase;
            entityMenu(new Entity(remoteDb, relToRemoteEntity.get_related_id2), 0, None, None, Some(relToRemoteEntity))
            let stillThere: bool = remoteDb.entity_key_exists(relToRemoteEntity.get_related_id2, include_archived = false) &&;
                                      remoteDb.attribute_key_exists(relToRemoteEntity.get_form_id, relToRemoteEntity.get_id)
            !stillThere
          case relToGroup: RelationToGroup =>
            new QuickGroupMenu(ui, controller).quickGroupMenu(new Group(relToGroup.db, relToGroup.get_group_id),
                                                              0, Some(relToGroup), containingEntityIn = Some(entity_in))
            if !relToGroup.db.group_key_exists(relToGroup.get_group_id)) true
            else false
          case _ => throw new Exception("Unexpected choice has class " + o.getClass.get_name + "--what should we do here?")
        }
      }
    }

    if !entity_in.db.entity_key_exists(entity_in.get_id, include_archived = false)) {
      // (entity could have been deleted or archived while browsing among containers via submenus)
      None
    } else {
      // check this, given that while in the goToSelectedAttribute method, the previously highlighted one could have been removed from the list:
      let defaultEntryToHighlight: Option[Attribute] = Some(attributeTuples(choices_index)._2);
      let nextToHighlight: Option[Attribute] = determineNextEntryToHighlight(entity_in, attributesToDisplay,;
                                                                             entryIsGoneNow, defaultEntryToHighlight, Some(choices_index))
      entityMenu(new Entity(entity_in.db, entity_in.get_id), attributeRowsStartingIndexIn, nextToHighlight, targetForMovesIn,
                 containingRelationToEntityIn, containingGroupIn)
    }
  }

    fn entitySearchSubmenu(entity_in: Entity, attributeRowsStartingIndexIn: Int, containingRelationToEntityIn: Option[AttributeWithValidAndObservedDates],
                          containingGroupIn: Option[Group], numAttrsInEntity: i64, attributeTuples: Array[(i64, Attribute)],
                          highlightedEntry: Option[Attribute], targetForMoves: Option[Attribute], answer: Int) {
    require(containingRelationToEntityIn.isEmpty ||
            containingRelationToEntityIn.get.isInstanceOf[RelationToLocalEntity] || containingRelationToEntityIn.get.isInstanceOf[RelationToRemoteEntity])
    let searchResponse = ui.ask_which(Some(Array("Choose a search option:")), Array(if numAttrsInEntity > 0) Util.LIST_NEXT_ITEMS_PROMPT else "(stub)",;
                                                                                   if numAttrsInEntity > 0) Util.LIST_PREV_ITEMS_PROMPT else "(stub)",
                                                                                   "Search related entities",
                                                                                   Util.MAIN_SEARCH_PROMPT))
    if searchResponse.is_defined) {
      let searchAnswer = searchResponse.get;
      if searchAnswer == 1) {
        let startingIndex: i32 = getNextStartingRowsIndex(attributeTuples.length, attributeRowsStartingIndexIn, numAttrsInEntity);
        entityMenu(entity_in, startingIndex, highlightedEntry, targetForMoves, containingRelationToEntityIn, containingGroupIn)
      } else if searchAnswer == 2) {
        ui.display_text("(Not yet implemented.)")
      } else if searchAnswer == 3) {
        // Idea: could share some code or ideas between here and Controller.findExistingObjectByText, and perhaps others like them.  For example,
        // this doesn't yet have logic to page down through the results, but maybe for now there won't be many or it can be added later.
        // Idea: maybe we could use an abstraction to make this kind of UI work even simpler, since we do it often.
        // Idea: make the following prompt and its code not be messy, in writing or in where it gets part of the text?

        // NOTE: this prompt should match the logic inside PostgreSQLDatabase.find_contained_local_entity_ids:
        let ans = ui.ask_for_string(Some(Array(Util.entity_or_group_name_sql_search_prompt("Entity name or text attribute content") +;
                                             ", (that is for the textAttribute content, but for the entity names it will do a Matcher.find, " +
                                             "after lowercasing both strings -- regex details at " +
                                             "https://docs.oracle.com/javase/8/docs/api/java/util/regex/Pattern.html.)")))
        if ans.is_defined) {
          let searchString: String = ans.get;
          let levelsAnswer = ui.ask_for_string(Some(Array("Enter the # of levels to search (above 10 can take many hours; currently only searches locally;" +;
                                                        " searching from main/top menu is often faster)")),
                                             Some(Util.is_numeric), Some("5"))
          let levels: i32 = levelsAnswer.getOrElse("4").toInt;
          let entity_idsTreeSet: mutable.TreeSet[i64] = entity_in.find_contained_local_entity_ids(new mutable.TreeSet[i64], searchString, levels,;
                                                                                             stop_after_any_foundIn = false)
          let entity_ids = entity_idsTreeSet.toArray;
          let leading_text2 = Vec<String>(Util.PICK_FROM_LIST_PROMPT);
          // could be like if numAttrsInEntity > 0) Controller.LIST_NEXT_ITEMS_PROMPT else "(stub)" above, if we made the method more sophisticated to do that.
          let choices: Vec<String> = Array("(stub)");
          let entity_idsTruncated: Array[i64] = {;
            //(A temporary workaround for too little info.  Better ideas in my OM todos: search for "show more search results in entitymenu",
            //entry created 2020-12-28.)
            //was:  let numDisplayableAttributes: i32 = ui.maxColumnarChoicesToDisplayAfter(leading_text2.length, choices.length, Util.maxNameLength);
            let numDisplayableAttributes = 84;

            if entity_ids.length <= numDisplayableAttributes) {
              entity_ids
            } else {
              let newarray: Array[i64] = new Array(numDisplayableAttributes);
              entity_ids.copyToArray(newarray, 0, numDisplayableAttributes)
              // (This is to avoid the later "require" error not far from the top of TextUI.ask_whichChoiceOrItsAlternate, if there are too many
              // menu items to display. It could be done better if we implement scrolling among the attrs, similarly to the other use of
              // ui.maxColumnarChoicesToDisplayAfter above, but in a way to avoid re-doing the search each time.)
              ui.display_text("There were " + entity_ids.length + " results, but truncated them to " + numDisplayableAttributes + " for display.  (If" +
                             " desired this can be improved, per the comments in the code.)")
              newarray
            }
          }
          let entityStatusesAndNames: Vec<String> = entity_idsTruncated.map {;
                                                                               id: i64 =>
                                                                                 let entity = new Entity(entity_in.db, id);
                                                                                 entity.get_archived_status_display_string + entity.get_name
                                                                             }
          //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) w/in this method!
          @tailrec fn showSearchResults() {
            let relatedEntitiesResult = ui.ask_which(Some(leading_text2), choices, entityStatusesAndNames);
            if relatedEntitiesResult.is_defined) {
              let relatedEntitiesAnswer = relatedEntitiesResult.get;
              //there might be more than we have room to show here...but...see "idea"s above.
              if relatedEntitiesAnswer == 1 && relatedEntitiesAnswer <= choices.length) {
                // (For reason behind " && answer <= choices.size", see comment where it is used elsewhere in entityMenu.)
                ui.display_text("Nothing implemented here yet.")
              } else if relatedEntitiesAnswer > choices.length && relatedEntitiesAnswer <= (choices.length + entityStatusesAndNames.length)) {
                // those in the condition on the previous line are 1-based, not 0-based.
                let index = relatedEntitiesAnswer - choices.length - 1;
                let id: i64 = entity_ids(index);
                entityMenu(new Entity(entity_in.db, id))
              }
              showSearchResults()
            }
          }
          showSearchResults()
        }
      } else if searchAnswer == 4) {
        let selection: Option[(IdWrapper, _, _)] = controller.chooseOrCreateObject(entity_in.db, None, None, None, Util.ENTITY_TYPE);
        if selection.is_defined) {
          entityMenu(new Entity(entity_in.db, selection.get._1.get_id))
        }
      }
    }
  }

    fn determineNextEntryToHighlight(entity_in: Entity, attributesToDisplay: util.ArrayList[Attribute], entryIsGoneNow: bool,
                                    defaultEntryToHighlight: Option[Attribute], highlightingIndex: Option[Int]) -> Option[Attribute] {
    // The entity or an attribute could have been removed or changed by navigating around various menus, so before trying to view it again,
    // confirm it exists, & (at the call to entityMenu) reread from db to refresh data for display, like public/non-public status:
    if entity_in.db.entity_key_exists(entity_in.get_id, include_archived = false)) {
      if highlightingIndex.is_defined && entryIsGoneNow) {
        Util.find_attribute_to_highlight_next(attributesToDisplay.size, attributesToDisplay, entryIsGoneNow, highlightingIndex.get, defaultEntryToHighlight.get)
      } else {
        defaultEntryToHighlight
      }
    } else {
      None
    }
  }

  /** @return A tuple containing the newStartingDisplayIndex and whether an entry moved from being listed on this entity.
    *         The parm relationSourceEntityIn is derivable from the parm containingRelationToEntityIn, but passing it in saves a db read.
    */
    fn moveSelectedEntry(entity_in: Entity, starting_display_row_index_in: Int, totalAttrsAvailable: Int, targetForMovesIn: Option[Attribute] = None,
                        highlightedIndexInObjListIn: Int, highlightedAttributeIn: Attribute, numObjectsToDisplayIn: Int,
                        relationSourceEntityIn: Option<Entity> = None,
                        containingRelationToEntityIn: Option[AttributeWithValidAndObservedDates] = None,
                        containingGroupIn: Option[Group] = None) -> (Int, Boolean) {
    require(containingRelationToEntityIn.isEmpty ||
            containingRelationToEntityIn.get.isInstanceOf[RelationToLocalEntity] || containingRelationToEntityIn.get.isInstanceOf[RelationToRemoteEntity])

    if relationSourceEntityIn.is_defined || containingRelationToEntityIn.is_defined) {
      require(relationSourceEntityIn.is_defined && containingRelationToEntityIn.is_defined,
              (if relationSourceEntityIn.isEmpty) "relationSourceEntityIn is empty; " else "") +
              (if containingRelationToEntityIn.isEmpty) "containingRelationToEntityIn is empty." else ""))

      require(relationSourceEntityIn.get.get_id == containingRelationToEntityIn.get.get_parent_id(), "relationSourceEntityIn: " + relationSourceEntityIn.get.get_id +
                                                                                                " doesn't match containingRelationToEntityIn.get.get_parent_id():" +
                                                                                                " " + containingRelationToEntityIn.get.get_parent_id() + ".")
    }
    let choices = Vec<String>(// (see comments at similar location in same-named method of QuickGroupMenu.);
                                "Move up " + controller.moveFartherCount,
                                "Move up 5", "Move up 1", "Move down 1", "Move down 5",
                                "Move down " + controller.moveFartherCount,

                                if targetForMovesIn.is_defined) "Move (*) to selected target (+, if any)"
                                else "(stub: have to choose a target before you can move entries into it)",

                                "Move (*) to calling menu (up one)",
                                "Move down " + controller.moveFarthestCount + " but keep data display position ")
    let response = ui.ask_which(None, choices, Vec<String>(), highlightIndexIn = Some(highlightedIndexInObjListIn));
    if response.isEmpty) (starting_display_row_index_in, false)
    else {
      let answer = response.get;
      let mut numRowsToMove = 0;
      let mut forwardNotBack = false;
      if (answer >= 1 && answer <= 6) || answer == 9) {
        if answer == 1) {
          numRowsToMove = controller.moveFartherCount
        } else if answer == 2) {
          numRowsToMove = 5
        } else if answer == 3) {
          numRowsToMove = 1
        } else if answer == 4) {
          numRowsToMove = 1
          forwardNotBack = true
        } else if answer == 5) {
          numRowsToMove = 5
          forwardNotBack = true
        } else if answer == 6) {
          numRowsToMove = controller.moveFartherCount
          forwardNotBack = true
        } else if answer == 9) {
          numRowsToMove = controller.moveFarthestCount
          forwardNotBack = true
        }
        let displayStartingRowNumber: i32 = {;
          let possibleDisplayStartingRowNumber = placeEntryInPosition(entity_in.db, entity_in.get_id, totalAttrsAvailable, numRowsToMove,;
                               forward_not_back_in = forwardNotBack, starting_display_row_index_in, highlightedAttributeIn.get_id,
                               highlightedIndexInObjListIn, Some(highlightedAttributeIn.get_id),
                               numObjectsToDisplayIn, highlightedAttributeIn.get_form_id,
                               Some(highlightedAttributeIn.get_form_id))
          if answer != 9) {
            possibleDisplayStartingRowNumber
          } else {
            // (This will keep the starting index in place, AND the highlight parameter in the menu on the old object,
            // so for now that will make the default, 1st, entry highlighted, but if you page forward in the UI, the
            // previously highlighted just-moved entry, still will be highlighted.  An accidental and awkward but helpful effect.)
            starting_display_row_index_in
          }
        }
        (displayStartingRowNumber, false)
      } else if answer == 7 && targetForMovesIn.is_defined) {
        if !(
             (highlightedAttributeIn.isInstanceOf[RelationToLocalEntity] ||
              highlightedAttributeIn.isInstanceOf[RelationToRemoteEntity] ||
              highlightedAttributeIn.isInstanceOf[RelationToGroup])
             &&
             (targetForMovesIn.get.isInstanceOf[RelationToLocalEntity] ||
              targetForMovesIn.get.isInstanceOf[RelationToRemoteEntity] ||
              targetForMovesIn.get.isInstanceOf[RelationToGroup])
             )) {
          ui.display_text("Currently, you can only move an Entity or a Group, to an Entity or a Group.  Moving thus is not yet implemented for other " +
                         "attribute types, but it shouldn't take much to add that. [1]")
          (starting_display_row_index_in, false)
        } else {
          //noinspection TypeCheckCanBeMatch
          if highlightedAttributeIn.isInstanceOf[RelationToLocalEntity] && targetForMovesIn.get.isInstanceOf[RelationToLocalEntity]) {
            let movingRtle = highlightedAttributeIn.asInstanceOf[RelationToLocalEntity];
            let target_entity_id = targetForMovesIn.get.asInstanceOf[RelationToLocalEntity].get_related_id2;
            require(movingRtle.get_parent_id() == entity_in.get_id)
            movingRtle.move_it(target_entity_id, get_sorting_index(entity_in.db, entity_in.get_id, movingRtle.get_form_id, movingRtle.get_id))
            (starting_display_row_index_in, true)
          } else if highlightedAttributeIn.isInstanceOf[RelationToRemoteEntity] && targetForMovesIn.get.isInstanceOf[RelationToLocalEntity]) {
            let movingRtre: RelationToRemoteEntity = highlightedAttributeIn.asInstanceOf[RelationToRemoteEntity];
            let target_entity_id = targetForMovesIn.get.asInstanceOf[RelationToLocalEntity].get_related_id2;
            require(movingRtre.get_parent_id() == entity_in.get_id)
            movingRtre.move_it(target_entity_id, get_sorting_index(entity_in.db, entity_in.get_id, movingRtre.get_form_id, movingRtre.get_id))
            (starting_display_row_index_in, true)
          } else if highlightedAttributeIn.isInstanceOf[RelationToLocalEntity] && targetForMovesIn.get.isInstanceOf[RelationToGroup]) {
            require(targetForMovesIn.get.get_form_id == Database.get_attribute_form_id(Util.RELATION_TO_GROUP_TYPE))
            let target_group_id = RelationToGroup.create_relation_to_group(targetForMovesIn.get.db, targetForMovesIn.get.get_id).get_group_id;
            let rtle = highlightedAttributeIn.asInstanceOf[RelationToLocalEntity];
            // about the sortingIndex:  see comment on db.move_entity_from_entity_to_group.
            rtle.move_entity_from_entity_to_group(target_group_id, get_sorting_index(entity_in.db, entity_in.get_id, rtle.get_form_id, rtle.get_id))
            (starting_display_row_index_in, true)
          } else if highlightedAttributeIn.isInstanceOf[RelationToGroup] && targetForMovesIn.get.isInstanceOf[RelationToLocalEntity]) {
            let movingRtg = highlightedAttributeIn.asInstanceOf[RelationToGroup];
            let newContainingEntityId = targetForMovesIn.get.asInstanceOf[RelationToLocalEntity].get_related_id2;
            require(movingRtg.get_parent_id() == entity_in.get_id)
            movingRtg.move_it(newContainingEntityId, get_sorting_index(entity_in.db, entity_in.get_id, movingRtg.get_form_id, movingRtg.get_id))
            (starting_display_row_index_in, true)
          } else if highlightedAttributeIn.isInstanceOf[RelationToGroup] && targetForMovesIn.get.isInstanceOf[RelationToGroup]) {
            ui.display_text("Unsupported: groups can't directly contain groups.  But groups can contain entities, and entities can contain groups and" +
                           " other attributes. [1]")
            (starting_display_row_index_in, false)
          } else {
            ui.display_text("Not yet supported.")
            (starting_display_row_index_in, false)
          }
        }
      } else if answer == 8) {
        if !(highlightedAttributeIn.isInstanceOf[RelationToLocalEntity] ||
              highlightedAttributeIn.isInstanceOf[RelationToRemoteEntity] ||
              highlightedAttributeIn.isInstanceOf[RelationToGroup])) {
          ui.display_text("Currently, you can only move an Entity or a Group, *to* an Entity or a Group.  Moving thus is not yet implemented for other " +
                         "attribute types, but it shouldn't take much to add that. [2]")
          (starting_display_row_index_in, false)
        } else {
          if containingRelationToEntityIn.is_defined) {
            require(containingGroupIn.isEmpty)
            let newContainingEntityId = {;
              //noinspection TypeCheckCanBeMatch  // as in some (not all) other places, just a guess as to what is more readable for non-scala-experts.
              if containingRelationToEntityIn.get.isInstanceOf[RelationToLocalEntity]) {
                containingRelationToEntityIn.get.asInstanceOf[RelationToLocalEntity].get_related_id1
              } else if containingRelationToEntityIn.get.isInstanceOf[RelationToRemoteEntity]) {
                containingRelationToEntityIn.get.asInstanceOf[RelationToRemoteEntity].get_related_id1
              } else throw new OmException("unexpected type: " + containingRelationToEntityIn.getClass.getCanonicalName)
            }
            //noinspection TypeCheckCanBeMatch
            if highlightedAttributeIn.isInstanceOf[RelationToLocalEntity]) {
              let movingRtle = highlightedAttributeIn.asInstanceOf[RelationToLocalEntity];
              movingRtle.move_it(newContainingEntityId, get_sorting_index(entity_in.db, entity_in.get_id, movingRtle.get_form_id, movingRtle.get_id))
              (starting_display_row_index_in, true)
            } else if highlightedAttributeIn.isInstanceOf[RelationToRemoteEntity]) {
              let movingRtre = highlightedAttributeIn.asInstanceOf[RelationToRemoteEntity];
              movingRtre.move_it(newContainingEntityId, get_sorting_index(entity_in.db, entity_in.get_id, movingRtre.get_form_id, movingRtre.get_id))
              (starting_display_row_index_in, true)
            } else if highlightedAttributeIn.isInstanceOf[RelationToGroup]) {
              let movingRtg = highlightedAttributeIn.asInstanceOf[RelationToGroup];
              movingRtg.move_it(newContainingEntityId, get_sorting_index(entity_in.db, entity_in.get_id, movingRtg.get_form_id, movingRtg.get_id))
              (starting_display_row_index_in, true)
            } else throw new OmException("Should be impossible to get here: I thought I checked for ok values, above. [1]")
          } else if containingGroupIn.is_defined) {
            require(containingRelationToEntityIn.isEmpty)
            //noinspection TypeCheckCanBeMatch
            if highlightedAttributeIn.isInstanceOf[RelationToLocalEntity]) {
              let target_group_id = containingGroupIn.get.get_id;
              let rtle = highlightedAttributeIn.asInstanceOf[RelationToLocalEntity];
              // about the sortingIndex:  see comment on db.move_entity_from_entity_to_group.
              rtle.move_entity_from_entity_to_group(target_group_id, get_sorting_index(entity_in.db, entity_in.get_id, rtle.get_form_id, rtle.get_id))
              (starting_display_row_index_in, true)
            } else if highlightedAttributeIn.isInstanceOf[RelationToRemoteEntity]) {
              ui.display_text("Unsupported: groups cannot directly contain remote entities.  Only local entities can contain relations" +
                             " to remote entities (currently at least).")
              (starting_display_row_index_in, false)
            } else if highlightedAttributeIn.isInstanceOf[RelationToGroup]) {
              ui.display_text("Unsupported: groups can't directly contain groups or relations to remote entities.  But groups can contain entities, " +
                             "and entities can contain groups and other attributes. [2]")
              (starting_display_row_index_in, false)
            } else throw new OmException("Should be impossible to get here: I thought I checked for ok values, above. [2]")
          } else {
            ui.display_text("One of the container parameters needs to be available, in order to move the highlighted attribute to the containing entity or " +
                           "group (the one from which you navigated here).")
            (starting_display_row_index_in, false)
          }
        }
      } else {
        (starting_display_row_index_in, false)
      }
    }
  }

    fn getLeadingText(leading_text_in: Vec<String>, numAttributes: Int, entity_in: Entity, containingGroupIn: Option[Group] = None) -> Vec<String> {
    leading_text_in(0) = Util.entity_menu_leading_text(entity_in)
    if containingGroupIn.is_defined) {
      leading_text_in(0) += ": found via group: " + containingGroupIn.get.get_name
    }
    //%%%%
    leading_text_in(0) += ": created " + entity_in.get_creation_date_formatted
    leading_text_in(1) = if numAttributes == 0) "No attributes have been assigned to this object, yet."
    else "Attribute list menu: (or choose attribute by letter)"
    leading_text_in
  }

    fn getItemDisplayStringsAndAttrs(attributeTuples: Array[(i64, Attribute)]) -> (Vec<String>, util.ArrayList[Attribute]) {
    let attributes = new util.ArrayList[Attribute];
    let attributeStatusesAndNames: Vec<String> =;
      for (attributeTuple <- attributeTuples) yield {
        let attribute = attributeTuple._2;
        attributes.add(attribute)
        attribute match {
          case relation: RelationToLocalEntity =>
            let toEntity: Entity = new Entity(relation.db, relation.get_related_id2);

            let relationType = new RelationType(relation.db, relation.get_attr_type_id());
            let desc = attribute.get_display_string(Util.maxNameLength, Some(toEntity), Some(relationType), simplify = true);
            let prefix = controller.getEntityContentSizePrefix(toEntity);
            let archivedStatus: String = toEntity.get_archived_status_display_string;
            prefix + archivedStatus + desc + controller.get_public_status_display_string(toEntity)
          case relation: RelationToRemoteEntity =>
            let remoteDb = relation.getRemoteDatabase;
            let toEntity: Entity = new Entity(remoteDb, relation.get_related_id2);

            let relationType = new RelationType(relation.db, relation.get_attr_type_id());
            let desc = attribute.get_display_string(Util.maxNameLength, Some(toEntity), Some(relationType), simplify = true);
            let prefix = controller.getEntityContentSizePrefix(toEntity);
            let archivedStatus: String = toEntity.get_archived_status_display_string;
            prefix + archivedStatus + desc + controller.get_public_status_display_string(toEntity)
          case relation: RelationToGroup =>
            let relationType = new RelationType(relation.db, relation.get_attr_type_id());
            let desc = attribute.get_display_string(Util.maxNameLength, None, Some(relationType), simplify = true);
            let prefix = controller.getGroupContentSizePrefix(relation.db, relation.get_group_id);
            prefix + "group: " + desc
          case _ =>
            attribute.get_display_string(Util.maxNameLength, None, None)
        }
      }
    (attributeStatusesAndNames, attributes)
  }

    fn addAttribute(entity_in: Entity, startingAttributeIndexIn: Int, highlightedAttributeIn: Option[Attribute], targetForMovesIn: Option[Attribute] = None,
                   containingGroupIn: Option[Group] = None) -> Option[Attribute] {
    let whichKindOfAttribute =;
      ui.ask_which(Some(Array("Choose which kind of attribute to add:")),
                  // THESE ARRAY INDICES (after being converted by ask_which to 1-based) MUST MATCH THOSE LISTED IN THE MATCH STATEMENT
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
    if whichKindOfAttribute.is_defined) {
      let attrForm: i32 = whichKindOfAttribute.get match {;
        // This is a bridge between the expected order for convenient UI above, and the parameter value expected by Controller.addAttribute
        // (1-based, not 0-based.)

        // (Using RELATION_TO_LOCAL_ENTITY_TYPE on next line even though it actually will work for either local or remote.  There wasn't room in the menu
        // to list them separately.)
        case 1 => Database.get_attribute_form_id(Util.RELATION_TO_LOCAL_ENTITY_TYPE)
        case 2 => 100
        case 3 => Database.get_attribute_form_id(Util.QUANTITY_TYPE)
        case 4 => Database.get_attribute_form_id(Util.DATE_TYPE)
        case 5 => Database.get_attribute_form_id(Util.BOOLEAN_TYPE)
        case 6 => Database.get_attribute_form_id(Util.FILE_TYPE)
        case 7 => Database.get_attribute_form_id(Util.TEXT_TYPE)
        case 8 => Database.get_attribute_form_id(Util.RELATION_TO_GROUP_TYPE)
        case 9 => 101
        // next one seems to happen if the user just presses Enter:
        case 0 => Database.get_attribute_form_id(Util.RELATION_TO_LOCAL_ENTITY_TYPE)
      }
      controller.addAttribute(entity_in, startingAttributeIndexIn, attrForm, None)
    } else {
      None
    }
  }

    fn getNextStartingRowsIndex(numAttrsToDisplay: Int, startingAttributeRowsIndexIn: Int, numAttrsInEntity: i64) -> Int {
    let startingIndex = {;
      let currentPosition = startingAttributeRowsIndexIn + numAttrsToDisplay;
      if currentPosition >= numAttrsInEntity) {
        ui.display_text("End of attribute list found; restarting from the beginning.")
        0 // start over
      } else currentPosition

    }
    startingIndex
  }

  protected fn getAdjacentEntriesSortingIndexes(db_in: Database, entity_idIn: i64, movingFromPosition_sortingIndexIn: i64, queryLimitIn: Option<i64>,
                                                 forward_not_back_in: bool) -> Vec<Vec<Option<DataType>>> {
    let entity = new Entity(db_in, entity_idIn);
    entity.get_adjacent_attributes_sorting_indexes(movingFromPosition_sortingIndexIn, queryLimitIn, forward_not_back_in)
  }

  protected fn get_sorting_indexOfNearestEntry(db_in: Database, entity_idIn: i64, starting_point_sorting_index_in: i64, forward_not_back_in: bool) -> Option<i64> {
    let entity = new Entity(db_in, entity_idIn);
    entity.get_nearest_attribute_entrys_sorting_index(starting_point_sorting_index_in, forward_not_back_in = forward_not_back_in)
  }

  protected fn renumber_sorting_indexes(db_in: Database, entity_idIn: i64) -> /*Unit%%*/ {
    let entity = new Entity(db_in, entity_idIn);
    entity.renumber_sorting_indexes()
  }

  protected fn updateSortedEntry(db_in: Database, entity_idIn: i64, movingAttributeFormIdIn: Int, movingAttributeIdIn: i64, sortingIndexIn: i64) /*-> Unit%%*/ {
    let entity = new Entity(db_in, entity_idIn);
    entity.update_attribute_sorting_index(movingAttributeFormIdIn, movingAttributeIdIn, sortingIndexIn)
  }

  protected fn get_sorting_index(db_in: Database, entity_idIn: i64, attribute_form_id_in: Int, attribute_id_in: i64) -> i64 {
    let entity = new Entity(db_in, entity_idIn);
    entity.get_attribute_sorting_index(attribute_form_id_in, attribute_id_in)
  }

  protected fn indexIsInUse(db_in: Database, entity_idIn: i64, sortingIndexIn: i64) -> bool {
    let entity = new Entity(db_in, entity_idIn);
    entity.is_attribute_sorting_index_in_use(sortingIndexIn)
  }

  protected fn find_unused_sorting_index(db_in: Database, entity_idIn: i64, startingWithIn: i64) -> i64 {
    let entity = new Entity(db_in, entity_idIn);
    entity.find_unused_attribute_sorting_index(Some(startingWithIn))
  }

*/
}
