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

import org.onemodel.database.PostgreSQLDatabase
import org.onemodel.model._
import org.onemodel.{Color, OmException, TextUI}

/** Allows sorting of group entries, quick work like for brainstorming.
  */
class QuickGroupMenu(override val ui: TextUI, override val db: PostgreSQLDatabase, val controller: Controller) extends SortableEntriesMenu(ui, db) {
  /** returns None if user wants out. The parameter startingDisplayRowIndexIn refers to the 0-based index among all possible displayable rows (i.e.,
    * if we have displayed
    * 20 objects out of 100, and the user says to go to the next 20, the startingDisplayRowIndexIn would become 21. */
  // The @tailrec is desired when possible,
  // because it seems that otherwise we might try to ESC back to a menu instance which is attempting to view a deleted entity, & crash!  But see the comment
  // mentioning why not to have it, below.  Maybe we need to use a loop around the menu instead of tail recursion in this case, if there is not a
  // way to turn the tail optimization off for a particular line.
  //@tailrec
  //scoping idea: see idea at beginning of EntityMenu.entityMenu
  def quickGroupMenu(groupIn: Group, startingDisplayRowIndexIn: Int, relationToGroupIn: Option[RelationToGroup] = None,
                     highlightedEntityIn: Option[Entity] = None, targetForMovesIn: Option[Entity] = None, callingMenusRtgIn: Option[RelationToGroup] = None,
                     containingEntityIn: Option[Entity] = None): Option[Entity] = {
    try {
      quickGroupMenu_doTheWork(groupIn, startingDisplayRowIndexIn, relationToGroupIn, highlightedEntityIn, targetForMovesIn, callingMenusRtgIn,
                               containingEntityIn)
    } catch {
      case e: Exception =>
        controller.handleException(e)
        val ans = ui.askYesNoQuestion("Go back to what you were doing (vs. going out)?", Some("y"))
        if (ans.isDefined && ans.get) quickGroupMenu(groupIn, startingDisplayRowIndexIn, relationToGroupIn, highlightedEntityIn, targetForMovesIn,
                                                     callingMenusRtgIn, containingEntityIn)
        else None
    }

  }

  def createNewOrFindOneGroupOnEntity(groupIn: Group, targetEntitysRtgCount: Long, targetEntityIn: Entity): (Long, Long, Long) = {
    // if there is 1 (obvious) destination, or no RTG on the selected entity (1 can be created), then add new entry there
    val (targetRtgId: Long, targetRelTypeId: Long, targetGroupId: Long) = {
      if (targetEntitysRtgCount == 0) {
        val name: String = targetEntityIn.getName
        val (newGroup: Group, newRTG: RelationToGroup) = targetEntityIn.createGroupAndAddHASRelationToIt(name, groupIn.getMixedClassesAllowed,
                                                                                                         System.currentTimeMillis)
        (newRTG.getId, newRTG.getAttrTypeId, newGroup.getId)
      } else {
        // given above conditions (w/ moveTargetIndexInObjList, and rtgCount twice), there must be exactly one, or there's a bug:
        val (rtgId, relTypeId, gid, moreAvailable) = db.findRelationToAndGroup_OnEntity(targetEntityIn.getId)
        if (gid.isEmpty || relTypeId.isEmpty || moreAvailable) throw new OmException("Found " + (if (gid.isEmpty) 0 else ">1") + " but by the earlier " +
                                                                                     "checks, " +
                                                                                     "there should be exactly one group in entity " + targetEntityIn.getId
                                                                                     + ": " +
                                                                                     targetEntityIn.getName)
        (rtgId.get, relTypeId.get, gid.get)
      }
    }
    (targetRtgId, targetRelTypeId, targetGroupId)
  }

  def moveSelectedEntry(groupIn: Group, startingDisplayRowIndexIn: Int, relationToGroupIn: Option[RelationToGroup], targetForMovesIn: Option[Entity],
                        highlightedIndexInObjListIn: Int, moveTargetIndexInObjList: Option[Int], highlightedEntry: Entity,
                        highlightedObjId: Long, objIds: Array[Long], objectsToDisplay: java.util.ArrayList[Entity],
                        callingMenusRtgIn: Option[RelationToGroup] = None, containingEntityIn: Option[Entity] = None): Option[Entity] = {
    val choices = Array[String](// these are ordered for convenience in doing them w/ the left hand: by frequency of use, and what seems easiest to remember
                                // for common operations with the 4 fingers sitting on the '1234' keys.  Using LH more in this because my RH gets tired more,
                                // and it seems like often people have their RH on the mouse.
                                "Move up 5", "Move up 1", "Move down 1", "Move down 5",

                                if (targetForMovesIn.isDefined) "Move (*) to (sole, if present) subgroup of selected target (+, if any)"
                                else "(stub: have to choose a target before you can move entries to it)",

                                "Move (*) to calling menu (up one)",
                                "Move up 25",
                                "Move down 25"
                                // idea: make an option #9 here which is a "quick archive"? (for removing completed tasks: maybe only after showing
                                // archived things and "undo" works well, or use 9 for the 'cut' part of a logical 'cut/paste' operation to move something?)
                               )
    val response = ui.askWhich(None, choices, Array[String](), highlightIndexIn = Some(highlightedIndexInObjListIn),
                               secondaryHighlightIndexIn = moveTargetIndexInObjList)
    if (response.isEmpty) quickGroupMenu(groupIn, highlightedIndexInObjListIn, relationToGroupIn, Some(highlightedEntry), targetForMovesIn, callingMenusRtgIn,
                                         containingEntityIn)
    else {
      val answer = response.get
      var numRowsToMove = 0
      var forwardNotBack = false

      if ((answer >= 1 && answer <= 4) || (answer >= 7 && answer <= 8)) {
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
        val displayStartingRowNumber: Int = placeEntryInPosition(groupIn.getId, groupIn.getSize, numRowsToMove, forwardNotBack, startingDisplayRowIndexIn, highlightedObjId,

                                                                 highlightedIndexInObjListIn, Some(highlightedObjId), objectsToDisplay.size, -1, Some(-1))
        quickGroupMenu(groupIn, displayStartingRowNumber, relationToGroupIn, Some(highlightedEntry), targetForMovesIn, callingMenusRtgIn, containingEntityIn)
      } else if (answer == 5 && targetForMovesIn.isDefined) {
        val targetRtgCount: Long = db.getRelationToGroupCountByEntity(Some(targetForMovesIn.get.getId))
        if (moveTargetIndexInObjList.isEmpty || targetRtgCount > 1) {
          // IDEA: could guess & move it in even if >1 subgroup present, by seeing which subgroup has the same class, if only 1 like that? Or if same-named?
          ui.displayText("Target must be selected (shows '+'), and must have exactly one subgroup (a single '>'), or none.")
          quickGroupMenu(groupIn, startingDisplayRowIndexIn, relationToGroupIn, Some(highlightedEntry), targetForMovesIn, callingMenusRtgIn, containingEntityIn)
        } else {
          // if there is 1 (obvious) destination, or no RTG on the selected entity (1 can be created), then move it there
          val (_, _, targetGroupId) = createNewOrFindOneGroupOnEntity(groupIn, targetRtgCount, targetForMovesIn.get)
          // about the sortingIndex:  see comment on db.moveEntityToNewGroup.
          db.moveEntityToNewGroup(targetGroupId, groupIn.getId, highlightedObjId, getSortingIndex(groupIn.getId, -1, highlightedObjId))
          val entityToHighlight: Option[Entity] = Controller.findEntityToHighlightNext(objIds.length, objectsToDisplay, removedOne = true,
                                                                                      highlightedIndexInObjListIn, highlightedEntry)
          quickGroupMenu(groupIn, startingDisplayRowIndexIn, relationToGroupIn, entityToHighlight, targetForMovesIn, callingMenusRtgIn, containingEntityIn)
        }
      } else if (answer == 6) {
        // if there is 1 (provided or obvious) destination), then move it there
        val targetGroupId: Option[Long] = {
          if (callingMenusRtgIn.isDefined) {
            Some(callingMenusRtgIn.get.getGroupId)
          } else {
            // none provided, so see if it's guessable
            val containingGroups: List[Array[Option[Any]]] = db.getGroupsContainingEntitysGroupsIds(groupIn.getId)
            if (containingGroups.isEmpty) {
              ui.displayText("Unable to find any containing groups, for the group \"" + groupIn.getName + "\" (ie, nowhere \"up\" found, to move it to).")
              None
            } else if (containingGroups.size == 1) {
              Some(containingGroups.head(0).get.asInstanceOf[Long])
            } else {
              ui.displayText("There are more than one containing groups, for the group \"" + groupIn.getName + "\".  You could, from an Entity Menu, " +
                             "choose the option to 'Go to...' and explore what contains it, to see if you want to make changes to the organization.  Might " +
                             "need a feature to choose which containing group to which to move an entity...?")
              None
            }
          }
        }
        if (targetGroupId.isEmpty) quickGroupMenu(groupIn, startingDisplayRowIndexIn, relationToGroupIn, Some(highlightedEntry), targetForMovesIn, callingMenusRtgIn,
                                                  containingEntityIn)
        else {
          db.moveEntityToNewGroup(targetGroupId.get, groupIn.getId, highlightedObjId, getSortingIndex(groupIn.getId, -1, highlightedObjId))
          val entityToHighlight: Option[Entity] = Controller.findEntityToHighlightNext(objIds.length, objectsToDisplay, removedOne = true,
                                                                                              highlightedIndexInObjListIn, highlightedEntry)
          quickGroupMenu(groupIn, startingDisplayRowIndexIn, relationToGroupIn, entityToHighlight, targetForMovesIn, callingMenusRtgIn, containingEntityIn)
        }
      } else {
        quickGroupMenu(groupIn, startingDisplayRowIndexIn, relationToGroupIn, Some(highlightedEntry), targetForMovesIn, callingMenusRtgIn, containingEntityIn)
      }
    }
  }

  /** The parameter relationToGroupIn is nice when available, optional otherwise, and represents the relation via which we got to this group.
    *
    * */
  def quickGroupMenu_doTheWork(groupIn: Group, startingDisplayRowIndexIn: Int, relationToGroupIn: Option[RelationToGroup], highlightedEntityIn: Option[Entity] = None,
                               targetForMovesIn: Option[Entity] = None, callingMenusRtgIn: Option[RelationToGroup] = None,
                               containingEntityIn: Option[Entity] = None): Option[Entity] = {
    require(groupIn != null)
    val choices = Array[String]("Create new entry quickly",
                                "Move selection (*) up/down, in, out...",
                                "Edit the selected entry's name",
                                "Create new entry...",
                                "Go to selected entity (not the subgroup)",
                                "List next items...",
                                "Select target (entry move destination: gets a '+')",
                                "Select entry to highlight (with '*'; typing the letter instead goes to the subgroup if any, else to that entity)",
                                "Other (slower actions, more complete menu)")
    val displayDescription = if (relationToGroupIn.isDefined) relationToGroupIn.get.getDisplayString(0) else groupIn.getDisplayString(0)
    // (idea: maybe this use of color on next line could be removed, if people don't rely on the color change.  I originally added it as a visual
    // cue to aid my transition to using entities more & groups less.  Same thing is done in GroupMenu.)
    // (Idea: this color thing should probably be handled in the textui class instead, especially if there were multiple kinds of UI.)
    val leadingText: Array[String] = Array(Color.yellow("ENTITY GROUP") + " (quick menu: acts on (w/ #'s) OR selects (w/ letters...) an entity): "
                                           + displayDescription)
    val numDisplayableItems = ui.maxColumnarChoicesToDisplayAfter(leadingText.length, choices.length, controller.maxNameLength)
    val objectsToDisplay: java.util.ArrayList[Entity] = groupIn.getGroupEntries(startingDisplayRowIndexIn, Some(numDisplayableItems))
    val objIds = for (entity: Entity <- objectsToDisplay.toArray(Array[Entity]())) yield {
      entity.getId
    }
    controller.addRemainingCountToPrompt(choices, objectsToDisplay.size, groupIn.getSize, startingDisplayRowIndexIn)
    val names: Array[String] = for (entity: Entity <- objectsToDisplay.toArray(Array[Entity]())) yield {
      val numSubgroupsPrefix: String = controller.getEntityContentSizePrefix(entity.getId)
      numSubgroupsPrefix + entity.getName + " " + entity.getPublicStatusString()
    }
    if (objIds.length == 0) {
      val response = ui.askWhich(Some(leadingText), Array[String]("Add entry", "Other (slower, more complete menu)"), Array[String](),
                                 highlightIndexIn = None)
      if (response.isEmpty) None
      else {
        val answer = response.get
        if (answer == 1) {
          controller.addEntityToGroup(groupIn)
          quickGroupMenu(groupIn, 0, relationToGroupIn, callingMenusRtgIn = callingMenusRtgIn, containingEntityIn = containingEntityIn)
        } else if (answer == 2 && answer <= choices.length) {
          new GroupMenu(ui, db, controller).groupMenu(groupIn, startingDisplayRowIndexIn, relationToGroupIn, callingMenusRtgIn, containingEntityIn)
        } else if (answer == 0) None
        else {
          // expected to be unreachable based on askWhich behavior (doesn't allow answers beyond the list of choices available), but for the compiler:
          None
        }
      }
    } else {
      // Idea: improve wherever needed, to remove bad smells, especially the named types used here & in related code.
      val (highlightedIndexInObjList: Int, highlightedObjId: Long, highlightedEntry: Entity, moveTargetIndexInObjList: Option[Int],
      targetForMoves: Option[Entity]) = {
        // Be sure the code is OK even if the highlightedEntityIn isn't really in the list due to caller logic error, etc.
        var highlightedObjId: Long = if (highlightedEntityIn.isEmpty) objIds(0) else highlightedEntityIn.get.getId
        var highlightedIndexInObjList: Int = {
          val index = objIds.indexOf(highlightedObjId)
          // if index == -1 then there could be a logic error where an entity not in the list was passed in, or an entry was moved and we're not displaying
          // the portion of the list containing that entry.  Regardless, don't fail (ie, don't throw AIOOBE) due to the -1 later, just make it None.
          if (index < 0) {
            highlightedObjId = objIds(0)
            0
          }
          else index
        }
        var moveTargetIndexInObjList: Option[Int] = if (targetForMovesIn.isEmpty) None
                                                    else {
                                                      val index = objIds.indexOf(targetForMovesIn.get.getId)
                                                      // same as just above: don't bomb w/ a -1
                                                      if (index < 0) None
                                                      else Some(index)
                                                    }
        if (moveTargetIndexInObjList.isDefined && highlightedIndexInObjList == moveTargetIndexInObjList.get) {
          // doesn't make sense if they're equal (ie move both, into both?, like if user changed the previous highlight on 1st selection to a move
          // target), so change one:
          if (highlightedIndexInObjList == 0 && objIds.length > 1) {
            highlightedIndexInObjList = 1
          } else {
            moveTargetIndexInObjList = None
          }
        }
        assert(highlightedIndexInObjList >= 0)
        val highlightedEntry: Entity = objectsToDisplay.get(highlightedIndexInObjList)
        highlightedObjId = highlightedEntry.getId
        val targetForMoves: Option[Entity] = if (moveTargetIndexInObjList.isEmpty) None
                                             else Some(objectsToDisplay.get(moveTargetIndexInObjList.get))
        (highlightedIndexInObjList, highlightedObjId, highlightedEntry, moveTargetIndexInObjList, targetForMoves)
      }

      if (highlightedIndexInObjList == moveTargetIndexInObjList.getOrElse(None)) {
        throw new OmException("We have wound up with the same entry for targetForMoves and highlightedEntry: that will be a problem: aborting before we put" +
                              " an entity in its own group and lose track of it or something.")
      }


      val response = ui.askWhich(Some(leadingText), choices, names, highlightIndexIn = Some(highlightedIndexInObjList),
                                 secondaryHighlightIndexIn = moveTargetIndexInObjList)
      if (response.isEmpty) None
      else {
        val answer = response.get
        if (answer == 1) {
          val (entryToHighlight:Option[Entity], displayStartingRowNumber: Int) = {
            // ask for less info when here in the quick menu, where want to add entity quickly w/ no fuss, like brainstorming.  User can always use long menu.
            val ans: Option[Entity] = controller.askForNameAndWriteEntity(Controller.ENTITY_TYPE, inLeadingText = Some("NAME THE ENTITY:"),
                                                               inClassId = groupIn.getClassId)
            if (ans.isDefined) {
              val newEntityId: Long = ans.get.getId
              db.addEntityToGroup(groupIn.getId, newEntityId)
              val displayStartingRowNumber: Int = placeEntryInPosition(groupIn.getId, groupIn.getSize, 0, forwardNotBackIn = true,
                                                                       startingDisplayRowIndexIn, newEntityId,
                                                                       highlightedIndexInObjList, Some(highlightedObjId), objectsToDisplay.size, -1, Some(-1))
              (Some(new Entity(db, newEntityId)), displayStartingRowNumber)
            }
            else (Some(highlightedEntry), startingDisplayRowIndexIn)
          }
          quickGroupMenu(groupIn, displayStartingRowNumber, relationToGroupIn, entryToHighlight, targetForMoves, callingMenusRtgIn, containingEntityIn)
        } else if (answer == 2) {
          moveSelectedEntry(groupIn, startingDisplayRowIndexIn, relationToGroupIn, targetForMoves, highlightedIndexInObjList, moveTargetIndexInObjList,
                            highlightedEntry, highlightedObjId, objIds, objectsToDisplay)
        } else if (answer == 3) {
          val editedEntity: Option[Entity] = controller.editEntityName(highlightedEntry)
          if (editedEntity.isEmpty)
            quickGroupMenu(groupIn, startingDisplayRowIndexIn, relationToGroupIn, Some(highlightedEntry), targetForMoves, callingMenusRtgIn, containingEntityIn)
          else {
            quickGroupMenu(groupIn, startingDisplayRowIndexIn, relationToGroupIn, editedEntity, targetForMoves, callingMenusRtgIn, containingEntityIn)
          }
        } else if (answer == 4) {
          //(the first is the same as if user goes to the selection & presses '1', but is here so there can
          // be a similar #2 for consistency/memorability with the EntityMenu.)
          val choices = Array[String]("Create new entry INSIDE selected entry",
                                      "Add entry from existing (quick search by name; uses \"has\" relation)")
          val response = ui.askWhich(None, choices, new Array[String](0))
          if (response.isDefined) {
            val addEntryAnswer = response.get
            if (addEntryAnswer == 1) {
              val targetRtgCount: Long = db.getRelationToGroupCountByEntity(Some(highlightedEntry.getId))
              val defaultToUsingSubgroup = {
                // (This question is experimental, to see if my usage of OM can move away from mostly using groups, to using just entities and attributes,
                // but still as efficiently. I.e., leaning toward 'modeling' data rather than just collapsible outlines.)  If it's a nuisance (ie, if most users
                // want OM to just create the subgroup without asking, never just adding an entity to an entity in a group) the question can be removed
                // and logic treated as if it were always Some(true). Or, logic could be treated as if it were always Some(false), so that subgroups can be
                // added only when that is what the user really wants (by going to the entity and adding the RelationToGroup attribute).  It depends on
                // which behavior should be encouraged.
                if (targetRtgCount == 0) ui.askYesNoQuestion("Create a new subgroup on this entity first, putting the new entity in the new subgroup? "
                                                             + "(answering \"n\" means just add it as a relationToEntity inside the selected entry", Some("n"))
                else Some(true)
              }
              if (defaultToUsingSubgroup.isDefined) {
                if (defaultToUsingSubgroup.get) {
                  if (targetRtgCount > 1) {
                    // IDEA: (see idea at similar logic above where entry is moved into a targeted group, about guessing which one)
                    ui.displayText("For this operation, the selection must have exactly one subgroup (a single '>'), or none.")
                    quickGroupMenu(groupIn, startingDisplayRowIndexIn, relationToGroupIn, Some(highlightedEntry), targetForMovesIn, callingMenusRtgIn,
                                   containingEntityIn)
                  } else {
                    val (rtgId: Long, relTypeId: Long, targetGroupId: Long) = createNewOrFindOneGroupOnEntity(groupIn, targetRtgCount, highlightedEntry)
                    // about the sortingIndex:  see comment on db.moveEntityToNewGroup.
                    val ans: Option[Entity] = controller.askForNameAndWriteEntity(Controller.ENTITY_TYPE, inLeadingText = Some("NAME THE ENTITY:"),
                                                                                  inClassId = groupIn.getClassId)
                    if (ans.isDefined) {
                      val newEntityId: Long = ans.get.getId
                      db.addEntityToGroup(targetGroupId, newEntityId)
                      val newRtg: RelationToGroup = new RelationToGroup(db, rtgId, highlightedEntry.getId, relTypeId, targetGroupId)
                      quickGroupMenu(new Group(db, targetGroupId), 0, Some(newRtg), None, None)
                    }
                    quickGroupMenu(groupIn, startingDisplayRowIndexIn, relationToGroupIn, Some(highlightedEntry), targetForMoves, callingMenusRtgIn, containingEntityIn)
                  }
                } else {
                  val newEntity: Option[Entity] = controller.askForNameAndWriteEntity(Controller.ENTITY_TYPE, inLeadingText = Some("NAME THE ENTITY:"),
                                                                                      inClassId = groupIn.getClassId)
                  if (newEntity.isDefined) {
                    val newEntityId: Long = newEntity.get.getId
                    val newRte: RelationToEntity = highlightedEntry.addHASRelationToEntity(newEntityId, None, System.currentTimeMillis())
                    new EntityMenu(ui, db, controller).entityMenu(newEntity.get, 0, None, Some(highlightedEntry), Some(newRte))
                  }
                  quickGroupMenu(groupIn, startingDisplayRowIndexIn, relationToGroupIn, newEntity, targetForMoves, callingMenusRtgIn, containingEntityIn)
                }
              } else {
                quickGroupMenu(groupIn, startingDisplayRowIndexIn, relationToGroupIn, Some(highlightedEntry), targetForMoves, callingMenusRtgIn, containingEntityIn)
              }
            } else if (addEntryAnswer == 2) {
              val entityChosen: Option[IdWrapper] = controller.askForNameAndSearchForEntity
              val (entryToHighlight:Option[Entity], displayStartingRowNumber: Int) = {
                if (entityChosen.isDefined) {
                  val entityChosenId: Long = entityChosen.get.getId
                  db.addEntityToGroup(groupIn.getId, entityChosenId)
                  val newDisplayStartingRowNumber: Int = placeEntryInPosition(groupIn.getId, groupIn.getSize, 0, forwardNotBackIn = true,
                                                                              startingDisplayRowIndexIn, entityChosenId, highlightedIndexInObjList,
                                                                              Some(highlightedObjId), objectsToDisplay.size, -1, Some(-1))
                  (Some(new Entity(db, entityChosenId)), newDisplayStartingRowNumber)
                } else (Some(highlightedEntry), startingDisplayRowIndexIn)
              }
              quickGroupMenu(groupIn, displayStartingRowNumber, relationToGroupIn, entryToHighlight, targetForMoves, callingMenusRtgIn, containingEntityIn)
            } else {
              ui.displayText("unexpected selection")
              quickGroupMenu(groupIn, startingDisplayRowIndexIn, relationToGroupIn, Some(highlightedEntry), targetForMoves, callingMenusRtgIn, containingEntityIn)
            }
          } else {
            quickGroupMenu(groupIn, startingDisplayRowIndexIn, relationToGroupIn, Some(highlightedEntry), targetForMoves, callingMenusRtgIn, containingEntityIn)
          }
        } else if (answer == 5) {
          new EntityMenu(ui, db, controller).entityMenu(highlightedEntry, 0, None, None, None, Some(groupIn))
          // deal with entityMenu possibly having deleted the entity:
          val removedOne: Boolean = !db.isEntityInGroup(groupIn.getId, highlightedEntry.getId)
          val entityToHighlightNext: Option[Entity] = Controller.findEntityToHighlightNext(objIds.length, objectsToDisplay, removedOne, highlightedIndexInObjList,
                                                                               highlightedEntry)
          quickGroupMenu(groupIn, startingDisplayRowIndexIn, relationToGroupIn, entityToHighlightNext, targetForMoves, callingMenusRtgIn, containingEntityIn)
        } else if (answer == 6) {
          val (entryToHighlight: Option[Entity], displayStartingRowNumber: Int) = {
            val nextStartPosition = startingDisplayRowIndexIn + objectsToDisplay.size
            if (nextStartPosition >= groupIn.getSize) {
              ui.displayText("End of attribute list found; restarting from the beginning.")
              (None, 0) // start over
            } else (highlightedEntityIn, nextStartPosition)
          }
          quickGroupMenu(groupIn, displayStartingRowNumber, relationToGroupIn, entryToHighlight, targetForMoves, callingMenusRtgIn, containingEntityIn)
        } else if (answer == 7) {
          val choices = Array[String]("Unselect current move target (if present; not necessary really)")
          // says 'same screenful' because it's easier to assume that the returned index refers to the currently available
          // local collections (a subset of all possible entries, for display), than calling chooseOrCreateObject and sounds as useful:
          val leadingText: Array[String] = Array("CHOOSE AN ENTRY (that contains only one subgroup) FOR THE TARGET OF MOVES (choose from SAME SCREENFUL as " +
                                                 "now;  if the target contains 0 subgroups, or 2 or more subgroups, " +
                                                 "use other means to move entities to it until some kind of \"move anywhere\" feature is added):")
          controller.addRemainingCountToPrompt(choices, objectsToDisplay.size, groupIn.getSize, startingDisplayRowIndexIn)

          val response = ui.askWhich(Some(leadingText), choices, names, highlightIndexIn = Some(highlightedIndexInObjList),
                                     secondaryHighlightIndexIn = moveTargetIndexInObjList)
          val (entryToHighlight, selectedTargetEntity): (Option[Entity], Option[Entity]) =
            if (response.isEmpty) (Some(highlightedEntry), targetForMoves)
            else {
              val answer = response.get
              if (answer == 1) {
                (Some(highlightedEntry), None)
              } else {
                // those in the condition are 1-based, not 0-based.
                // user typed a letter to select an attribute (now 0-based):
                val choicesIndex = answer - choices.length - 1
                val userSelection: Entity = objectsToDisplay.get(choicesIndex)
                if (choicesIndex == highlightedIndexInObjList) {
                  // chose same entity for the target, as the existing highlighted selection, so make it the target, and no highlighted one.
                  (None, Some(userSelection))
                } else {
                  (Some(highlightedEntry), Some(userSelection))
                }
              }
            }
          //  and do same on selected, make sure that's not same as secondary (target): if so fail & don't select  (OR: just pass None so other one is
          // un-selected!)
          quickGroupMenu(groupIn, startingDisplayRowIndexIn, relationToGroupIn, entryToHighlight, selectedTargetEntity, callingMenusRtgIn, containingEntityIn)
        } else if (answer == 8) {
          // lets user select a new entity or group for further operations like moving, deleting.
          // (we have to have at least one choice or ui.askWhich fails...a require() call there.)
          // NOTE: this code is similar (not identical) in EntityMenu as in QuickGroupMenu: if one changes, the other might also need maintenance.
          val choices = Array[String]("keep existing (same as ESC)")
          // says 'same screenful' because (see similar cmt elsewhere).
          val leadingText: Array[String] = Array("CHOOSE AN ENTRY to highlight (*)")
          controller.addRemainingCountToPrompt(choices, objectsToDisplay.size, groupIn.getSize, startingDisplayRowIndexIn)
          val response = ui.askWhich(Some(leadingText), choices, names, highlightIndexIn = Some(highlightedIndexInObjList),
                                     secondaryHighlightIndexIn = moveTargetIndexInObjList)
          val (entityToHighlight, selectedTargetEntity): (Option[Entity], Option[Entity]) =
            if (response.isEmpty || response.get == 1) (Some(highlightedEntry), targetForMoves)
            else {
              // those in the condition are 1-based, not 0-based.
              // user typed a letter to select an attribute (now 0-based):
              val choicesIndex = response.get - choices.length - 1
              val userSelection: Entity = objectsToDisplay.get(choicesIndex)
              if (choicesIndex == moveTargetIndexInObjList.getOrElse(None)) {
                // chose same entity for the target, as the existing highlighted selection, so make it the target, and no highlighted one.
                (Some(userSelection), None)
              } else {
                (Some(userSelection), targetForMoves)
              }
            }
          quickGroupMenu(groupIn, startingDisplayRowIndexIn, relationToGroupIn, entityToHighlight, selectedTargetEntity, callingMenusRtgIn, containingEntityIn)
        } else if (answer == 9 && answer <= choices.length) {
          new GroupMenu(ui, db, controller).groupMenu(groupIn, startingDisplayRowIndexIn, relationToGroupIn, callingMenusRtgIn, containingEntityIn)
        } else if (false /*can this be changed so that if they hit Enter it makes it to here ?*/ ) {
          // do something with enter: do a quick text edit & update the dates. Or quickAddEntry ?
          ui.displayText("not yet implemented")
          quickGroupMenu(groupIn, startingDisplayRowIndexIn, relationToGroupIn, Some(highlightedEntry), targetForMoves, callingMenusRtgIn, containingEntityIn)
        } else if (answer == 0) None
        else if (answer > choices.length && answer <= (choices.length + objectsToDisplay.size)) {
          // those in the condition are 1-based, not 0-based.
          // lets user go to an entity or group quickly (1 stroke)
          val choicesIndex = answer - choices.length - 1
          // user typed a letter to select an attribute (now 0-based)
          if (choicesIndex >= objectsToDisplay.size()) {
            ui.displayText("The program shouldn't have let us get to this point, but the selection " + answer + " is not in the list.")
            quickGroupMenu(groupIn, startingDisplayRowIndexIn, relationToGroupIn, Some(highlightedEntry), targetForMoves, callingMenusRtgIn, containingEntityIn)
          } else {
            val userSelection: Entity = objectsToDisplay.get(choicesIndex)

            val (_ /*subEntitySelected:Option[Entity]*/ , groupId: Option[Long], moreThanOneGroupAvailable) =
              controller.goToEntityOrItsSoleGroupsMenu(userSelection, relationToGroupIn, Some(groupIn))

            val removedOne: Boolean = !db.isEntityInGroup(groupIn.getId, userSelection.getId)
            var entityToHighlightNext: Option[Entity] = Some(userSelection)
            if (groupId.isDefined && !moreThanOneGroupAvailable) {
              //idea: do something w/ this unused variable? Like, if the userSelection was deleted, then use this in its place in parms to
              // qGM just below? or what was it for originally?  Or, del this var around here?
              entityToHighlightNext = Controller.findEntityToHighlightNext(objIds.length, objectsToDisplay, removedOne, highlightedIndexInObjList, highlightedEntry)
            }

            //ck 1st if it exists, if not return None. It could have been deleted while navigating around.
            if (db.groupKeyExists(groupIn.getId)) {
              if (choicesIndex == moveTargetIndexInObjList.getOrElse(None)) {
                quickGroupMenu(groupIn, startingDisplayRowIndexIn, relationToGroupIn, Some(userSelection), None, callingMenusRtgIn, containingEntityIn)
              } else {
                quickGroupMenu(groupIn, startingDisplayRowIndexIn, relationToGroupIn, Some(userSelection), targetForMoves, callingMenusRtgIn, containingEntityIn)
              }
            } else None
          }
        } else {
          ui.displayText("invalid selection")
          quickGroupMenu(groupIn, startingDisplayRowIndexIn, relationToGroupIn, Some(highlightedEntry), targetForMoves, callingMenusRtgIn, containingEntityIn)
        }
      }
    }
  }

  protected def getAdjacentEntriesSortingIndexes(groupIdIn: Long, movingFromPosition_sortingIndexIn: Long, queryLimitIn: Option[Long],
                                        forwardNotBackIn: Boolean): List[Array[Option[Any]]] = {
    db.getAdjacentGroupEntriesSortingIndexes(groupIdIn, movingFromPosition_sortingIndexIn, queryLimitIn, forwardNotBackIn)
  }

  protected def getNearestEntrysSortingIndex(groupIdIn: Long, startingPointSortingIndexIn: Long, forwardNotBackIn: Boolean): Option[Long] = {
    db.getNearestGroupEntrysSortingIndex(groupIdIn, startingPointSortingIndexIn, forwardNotBackIn = forwardNotBackIn)
  }

  protected def renumberSortingIndexes(groupIdIn: Long): Unit = {
    db.renumberSortingIndexes(groupIdIn, isEntityAttrsNotGroupEntries = false)
  }

  protected def updateSortedEntry(groupIdIn: Long, ignoredParameter: Int, movingEntityIdIn: Long, sortingIndexIn: Long): Unit = {
    db.updateEntityInAGroup(groupIdIn, movingEntityIdIn, sortingIndexIn)
  }

  protected def getSortingIndex(groupIdIn: Long, ignoredParameter: Int, entityIdIn: Long): Long = {
    db.getGroupSortingIndex(groupIdIn, entityIdIn)
  }

  protected def indexIsInUse(groupIdIn: Long, sortingIndexIn: Long): Boolean = {
    db.groupEntrySortingIndexInUse(groupIdIn, sortingIndexIn)
  }

  protected def findUnusedSortingIndex(groupIdIn: Long, startingWithIn: Long): Long = {
    db.findUnusedGroupSortingIndex(groupIdIn, Some(startingWithIn))
  }

}
