/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003-2004 and 2008-2018 inclusive, Luke A. Call; all rights reserved.
    (That copyright statement was previously 2013-2015, until I remembered that much of Controller came from TextUI.scala and TextUI.java before that.)
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule, guidelines around binary
    distribution, and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
package org.onemodel.core.controllers

import org.onemodel.core._
import org.onemodel.core.model._
import org.onemodel.core.{Color, OmException, TextUI}

/** Allows sorting of group entries, quick work like for brainstorming.
  */
class QuickGroupMenu(override val ui: TextUI, val controller: Controller) extends SortableEntriesMenu(ui) {
  // The @tailrec is desired when possible,
  // because it seems that otherwise we might try to ESC back to a menu instance which is attempting to view a deleted entity, & crash!  But see the comment
  // mentioning why not to have it, below.  Maybe we need to use a loop around the menu instead of tail recursion in this case, if there is not a
  // way to turn the tail optimization off for a particular line.
  //@tailrec
  //scoping idea: see idea at beginning of EntityMenu.entityMenu
  /**
   * @param groupIn The id of the group whose info is being displayed.
   * @param startingDisplayRowIndexIn refers to the 0-based index among all possible displayable rows (i.e., if we have displayed
   *                                  20 objects out of 100, and the user says to go to the next 20, the startingDisplayRowIndexIn would become 21.
   * @param relationToGroupIn Matches the groupIn parameter, but with more info when available, about the RelationToGroup that this group menu display shows,
   *                          like if we came here via an entity (usually), the RelationToGroup that linked to the group,
   * @param highlightedEntityIn
   * @param targetForMovesIn
   * @param callingMenusRtgIn Only applies if a we were at a QuickGroupMenu and went directly to an entity's sole subgroup: this parm holds
   *                          the RelationToGroup for the group that was being displayed by that prior menu.
   * @param containingEntityIn Since every group was once contained by an entity, this can usually be filled in, but would not if we were viewing an orphaned
   *                           group (ex., if its containing entity was deleted?, or cases where we came to the group some other way, not via the entity.)
   * @return None if user wants out.
   */
  //noinspection ScalaDocMissingParameterDescription ...since i like the auto-generation or having all the parms here, but not having to fill them all in.
  def quickGroupMenu(groupIn: Group, startingDisplayRowIndexIn: Int, relationToGroupIn: Option[RelationToGroup] = None,
                     highlightedEntityIn: Option[Entity] = None, targetForMovesIn: Option[Entity] = None,
                     //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) w/in this method!
                     callingMenusRtgIn: Option[RelationToGroup] = None, containingEntityIn: Option[Entity]): Option[Entity] = {
    try {
      quickGroupMenu_doTheWork(groupIn, startingDisplayRowIndexIn, relationToGroupIn, highlightedEntityIn, targetForMovesIn, callingMenusRtgIn,
                               containingEntityIn)
    } catch {
      case e: Exception =>
        Util.handleException(e, ui, groupIn.mDB)
        val ans = ui.askYesNoQuestion("Go back to what you were doing (vs. going out)?", Some("y"))
        if (ans.isDefined && ans.get) quickGroupMenu(groupIn, startingDisplayRowIndexIn, relationToGroupIn, highlightedEntityIn, targetForMovesIn,
                                                     callingMenusRtgIn, containingEntityIn)
        else None
    }

  }

  /** Should be called only if the targetEntityIn has 0 or 1 RelationToGroups (no more).
    */
  def createNewOrFindOneGroupOnEntity(groupIn: Group, targetEntitysRtgCount: Long, targetEntityIn: Entity): (Long, Long, Long) = {
    // if there is 1 (obvious) destination, or no RTG on the selected entity (1 can be created), then add new entry there
    val (targetRelationToGroupId: Long, targetRelationTypeId: Long, targetGroupId: Long) = {
      if (targetEntitysRtgCount == 0) {
        val name: String = targetEntityIn.getName
        val (newGroup: Group, newRTG: RelationToGroup) = targetEntityIn.createGroupAndAddHASRelationToIt(name, groupIn.getMixedClassesAllowed,
                                                                                                         System.currentTimeMillis)
        (newRTG.getId, newRTG.getAttrTypeId, newGroup.getId)
      } else {
        // given above conditions (w/ moveTargetIndexInObjList, and rtgCount twice), there must be exactly one, or there's a bug:
        val (rtgId, relTypeId, gid, _, moreAvailable): (Option[Long], Option[Long],
          Option[Long], Option[String], Boolean) = targetEntityIn.findRelationToAndGroup

        if (gid.isEmpty || relTypeId.isEmpty || moreAvailable) throw new OmException("Found " + (if (gid.isEmpty) 0 else ">1") + " but by the earlier " +
                                                                                     "checks, " +
                                                                                     "there should be exactly one group in entity " + targetEntityIn.getId
                                                                                     + ": " +
                                                                                     targetEntityIn.getName)
        (rtgId.get, relTypeId.get, gid.get)
      }
    }
    (targetRelationToGroupId, targetRelationTypeId, targetGroupId)
  }

  def moveSelectedEntry(groupIn: Group, startingDisplayRowIndexIn: Int, relationToGroupIn: Option[RelationToGroup], targetForMovesIn: Option[Entity],
                        highlightedIndexInObjListIn: Int, moveTargetIndexInObjList: Option[Int], highlightedEntry: Entity,
                        highlightedObjId: Long, objIds: Array[Long], objectsToDisplay: java.util.ArrayList[Entity],
                        callingMenusRtgIn: Option[RelationToGroup] = None, containingEntityIn: Option[Entity] = None): Option[Entity] = {
    val choices = Array[String](// these are ordered for convenience in doing them w/ the left hand: by frequency of use, and what seems easiest to remember
                                // for common operations with the 4 fingers sitting on the '1234' keys.  Using LH more in this because my RH gets tired more,
                                // and it seems like often people have their RH on the mouse.
                                "Move up " + controller.moveFartherCount,
                                "Move up 5", "Move up 1", "Move down 1", "Move down 5",
                                "Move down " + controller.moveFartherCount,

                                if (targetForMovesIn.isDefined) "Move (*) to selected target (+, if any)"
                                else "(stub: have to choose a target before you can move entries into it)",

                                "Move (*) to calling menu (up one)",
                                "Move down " + controller.moveFarthestCount + " but keep data display position "
                                // idea: make an option here which is a "quick archive"? (for removing completed tasks: maybe only after showing
                                // archived things and "undo" works well, or use 9 for the 'cut' part of a logical 'cut/paste' operation to move something?)
                                // But, would have to start using alt keys to distinguish between options chosen when there are that many?
                               )
    val response = ui.askWhich(None, choices, Array[String](), highlightIndexIn = Some(highlightedIndexInObjListIn),
                               secondaryHighlightIndexIn = moveTargetIndexInObjList)
    if (response.isEmpty) quickGroupMenu(groupIn, highlightedIndexInObjListIn, relationToGroupIn, Some(highlightedEntry), targetForMovesIn, callingMenusRtgIn,
                                         containingEntityIn)
    else {
      val answer = response.get
      var numRowsToMove = 0
      var forwardNotBack = false

      if ((answer >= 1 && answer <= 6) || answer == 9) {
        if (answer == 1) {
          numRowsToMove = controller.moveFartherCount
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
          numRowsToMove = controller.moveFartherCount
          forwardNotBack = true
        } else if (answer == 9) {
          numRowsToMove = controller.moveFarthestCount
          forwardNotBack = true
        }
        val displayStartingRowNumber: Int = {
          val possibleDisplayStartingRowNumber = placeEntryInPosition(groupIn.mDB, groupIn.getId, groupIn.getSize(4), numRowsToMove, forwardNotBack,
                               startingDisplayRowIndexIn, highlightedObjId, highlightedIndexInObjListIn,
                               Some(highlightedObjId), objectsToDisplay.size, -1, Some(-1))
          if (answer != 9) {
            possibleDisplayStartingRowNumber
          } else {
            // (see note at same place in EntityMenu, re the position and the highlight)
            startingDisplayRowIndexIn
          }
        }
        quickGroupMenu(groupIn, displayStartingRowNumber, relationToGroupIn, Some(highlightedEntry), targetForMovesIn, callingMenusRtgIn, containingEntityIn)
      } else if (answer == 7 && targetForMovesIn.isDefined) {
        val targetRtgCount: Long = targetForMovesIn.get.getRelationToGroupCount
        if (moveTargetIndexInObjList.isEmpty) {
          ui.displayText("Target must be selected (shows '+').")
          quickGroupMenu(groupIn, startingDisplayRowIndexIn, relationToGroupIn, Some(highlightedEntry), targetForMovesIn, callingMenusRtgIn, containingEntityIn)
        } else {
          if (targetRtgCount > 1) {
            // can't guess which subgroup so just move it to the entity (idea: could ask whether to do that or go to which subgroup, perhaps...)
            groupIn.moveEntityFromGroupToLocalEntity(targetForMovesIn.get.getId, highlightedObjId, getSortingIndex(groupIn.mDB, groupIn.getId, -1, highlightedObjId))
            val entityToHighlight: Option[Entity] = Util.findEntityToHighlightNext(objIds.length, objectsToDisplay, removedOneIn = true,
                                                                                         highlightedIndexInObjListIn, highlightedEntry)
            quickGroupMenu(groupIn, startingDisplayRowIndexIn, relationToGroupIn, entityToHighlight, targetForMovesIn, callingMenusRtgIn, containingEntityIn)
          } else {
            val (_, defaultToUsingSubgroup: Option[Boolean]) = useSubgroup(targetForMovesIn.get)
            if (defaultToUsingSubgroup.isEmpty) {
              // user just wanted out of the question (whether the code can get here depends on parms passed to the ui question in above call to useSubgroup)
              quickGroupMenu(groupIn, startingDisplayRowIndexIn, relationToGroupIn, Some(highlightedEntry), targetForMovesIn, callingMenusRtgIn,
                             containingEntityIn)
            } else {
              if (defaultToUsingSubgroup.get) {
                // if there is 1 (obvious) destination, or no RTG on the selected entity (1 can be created), then move it there
                val (_, _, targetGroupId) = createNewOrFindOneGroupOnEntity(groupIn, targetRtgCount, targetForMovesIn.get)
                // about the sortingIndex:  see comment on db.moveEntityToNewGroup.
                groupIn.moveEntityToDifferentGroup(targetGroupId, highlightedObjId, getSortingIndex(groupIn.mDB, groupIn.getId, -1, highlightedObjId))
              } else {
                // getting here means to just create a RelationToLocalEntity on the entity, not a subgroup:
                groupIn.moveEntityFromGroupToLocalEntity(targetForMovesIn.get.getId, highlightedObjId, getSortingIndex(groupIn.mDB, groupIn.getId,
                                                                                                                  -1, highlightedObjId))
              }
              val entityToHighlight: Option[Entity] = Util.findEntityToHighlightNext(objIds.length, objectsToDisplay, removedOneIn = true,
                                                                                           highlightedIndexInObjListIn, highlightedEntry)
              quickGroupMenu(groupIn, startingDisplayRowIndexIn, relationToGroupIn, entityToHighlight, targetForMovesIn, callingMenusRtgIn, containingEntityIn)
            }
          }
        }
      } else if (answer == 8) {
        // if there is 1 (provided or guessable) destination), then move it there
        val (targetGroupId: Option[Long], targetEntity: Option[Entity]) = {
          // see whether will be moving it to an entity or a group, if anywhere.
          // ONLY RETURN ONE OF THE TWO, since the below relies on that to know which to do.
          if (callingMenusRtgIn.isDefined && containingEntityIn.isDefined) {
            val answer = ui.askWhich(None,
                        Array("Move it to the containing entity: " + containingEntityIn.get.getName,
                              "Move it to the containing group: " + new Group(callingMenusRtgIn.get.mDB, callingMenusRtgIn.get.getGroupId).getName))
            if (answer.isEmpty) {
              (None, None)
            } else if (answer.get == 1) {
              (None, containingEntityIn)
            } else if (answer.get == 2) {
              (Some(callingMenusRtgIn.get.getGroupId), None)
            }
          } else if (callingMenusRtgIn.isDefined) {
            (Some(callingMenusRtgIn.get.getGroupId), None)
          } else if (containingEntityIn.isDefined) {
            (None, containingEntityIn)
          } else {
            // none provided, so see if it's guessable
            // (Idea: if useful, could also try guessing the entity if there's just one, then if both are there let user choose which as just above.
            // And if > 1 of either or both groups/entities, ask to which of them to move it?)
            val containingGroupsIds: List[Array[Option[Any]]] = groupIn.getGroupsContainingEntitysGroupsIds()
            if (containingGroupsIds.isEmpty) {
              ui.displayText("Unable to find any containing groups, for the group \"" + groupIn.getName + "\" (ie, nowhere \"up\" found, to move it to).")
              (None, None)
            } else if (containingGroupsIds.size == 1) {
              (Some(containingGroupsIds.head(0).get.asInstanceOf[Long]), None)
            } else {
              ui.displayText("There are more than one containing groups, for the group \"" + groupIn.getName + "\".  You could, from an Entity Menu, " +
                             "choose the option to 'Go to...' and explore what contains it, to see if you want to make changes to the organization.  Might " +
                             "need a feature to choose a containing group as the target for moving an entity...?")
              (None, None)
            }
          }
        }
        if (targetEntity.isDefined) {
          require(targetGroupId.isEmpty)
          groupIn.moveEntityFromGroupToLocalEntity(containingEntityIn.get.getId, highlightedObjId, getSortingIndex(groupIn.mDB, groupIn.getId, -1, highlightedObjId))
          val entityToHighlight: Option[Entity] = Util.findEntityToHighlightNext(objIds.length, objectsToDisplay, removedOneIn = true,
                                                                                       highlightedIndexInObjListIn, highlightedEntry)
          quickGroupMenu(groupIn, startingDisplayRowIndexIn, relationToGroupIn, entityToHighlight, targetForMovesIn, callingMenusRtgIn, containingEntityIn)
        } else if (targetGroupId.isDefined) {
          require(targetEntity.isEmpty)
          groupIn.moveEntityToDifferentGroup(targetGroupId.get, highlightedObjId, getSortingIndex(groupIn.mDB, groupIn.getId, -1, highlightedObjId))
          val entityToHighlight: Option[Entity] = Util.findEntityToHighlightNext(objIds.length, objectsToDisplay, removedOneIn = true,
                                                                                      highlightedIndexInObjListIn, highlightedEntry)
          quickGroupMenu(groupIn, startingDisplayRowIndexIn, relationToGroupIn, entityToHighlight, targetForMovesIn, callingMenusRtgIn, containingEntityIn)
        } else {
          quickGroupMenu(groupIn, startingDisplayRowIndexIn, relationToGroupIn, Some(highlightedEntry), targetForMovesIn, callingMenusRtgIn, containingEntityIn)
        }
      } else {
        quickGroupMenu(groupIn, startingDisplayRowIndexIn, relationToGroupIn, Some(highlightedEntry), targetForMovesIn, callingMenusRtgIn, containingEntityIn)
      }
    }
  }

  /** The parameter relationToGroupIn is nice when available, optional otherwise, and represents the relation via which we got to this group.
    *
    * */
  def quickGroupMenu_doTheWork(groupIn: Group, startingDisplayRowIndexIn: Int, relationToGroupIn: Option[RelationToGroup],
                               highlightedEntityIn: Option[Entity] = None, targetForMovesIn: Option[Entity] = None,
                               callingMenusRtgIn: Option[RelationToGroup] = None, containingEntityIn: Option[Entity]): Option[Entity] = {
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
    val numDisplayableItems = ui.maxColumnarChoicesToDisplayAfter(leadingText.length, choices.length, Util.maxNameLength)
    val objectsToDisplay: java.util.ArrayList[Entity] = groupIn.getGroupEntries(startingDisplayRowIndexIn, Some(numDisplayableItems))
    val objIds = for (entity: Entity <- objectsToDisplay.toArray(Array[Entity]())) yield {
      entity.getId
    }
    Util.addRemainingCountToPrompt(choices, objectsToDisplay.size, groupIn.getSize(4), startingDisplayRowIndexIn)
    val statusesAndNames: Array[String] = for (entity: Entity <- objectsToDisplay.toArray(Array[Entity]())) yield {
      val numSubgroupsPrefix: String = controller.getEntityContentSizePrefix(entity)
      val archivedStatus = entity.getArchivedStatusDisplayString
      archivedStatus + numSubgroupsPrefix + entity.getName + " " + controller.getPublicStatusDisplayString(entity)
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
          new GroupMenu(ui, controller).groupMenu(groupIn, startingDisplayRowIndexIn, relationToGroupIn, callingMenusRtgIn, containingEntityIn)
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


      val response = ui.askWhich(Some(leadingText), choices, statusesAndNames, highlightIndexIn = Some(highlightedIndexInObjList),
                                 secondaryHighlightIndexIn = moveTargetIndexInObjList)
      if (response.isEmpty) None
      else {
        val answer = response.get
        if (answer == 1) {
          val (entryToHighlight:Option[Entity], displayStartingRowNumber: Int) = {
            // ask for less info when here in the quick menu, where want to add entity quickly w/ no fuss, like brainstorming.  User can always use long menu.
            val ans: Option[Entity] = controller.askForNameAndWriteEntity(groupIn.mDB, Util.ENTITY_TYPE, leadingTextIn = Some("NAME THE ENTITY:"),
                                                               classIdIn = groupIn.getClassId)
            if (ans.isDefined) {
              val newEntity = ans.get
              val newEntityId: Long = newEntity.getId
              groupIn.addEntity(newEntityId)
              // (See comment at similar place in EntityMenu, just before that call to placeEntryInPosition.)
              val goingBackward: Boolean = highlightedIndexInObjList == 0 && groupIn.getNewEntriesStickToTop
              val forwardNotBack = !goingBackward
              val displayStartingRowNumber: Int = placeEntryInPosition(groupIn.mDB, groupIn.getId, groupIn.getSize(4), 0,
                                                                       forwardNotBackIn = forwardNotBack, startingDisplayRowIndexIn, newEntityId,
                                                                       highlightedIndexInObjList, Some(highlightedObjId), objectsToDisplay.size, -1, Some(-1))
              controller.defaultAttributeCopying(newEntity)
              (Some(new Entity(groupIn.mDB, newEntityId)), displayStartingRowNumber)
            }
            else (Some(highlightedEntry), startingDisplayRowIndexIn)
          }
          quickGroupMenu(groupIn, displayStartingRowNumber, relationToGroupIn, entryToHighlight, targetForMoves, callingMenusRtgIn, containingEntityIn)
        } else if (answer == 2) {
          moveSelectedEntry(groupIn, startingDisplayRowIndexIn, relationToGroupIn, targetForMoves, highlightedIndexInObjList, moveTargetIndexInObjList,
                            highlightedEntry, highlightedObjId, objIds, objectsToDisplay, callingMenusRtgIn, containingEntityIn)
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
              val (targetRtgCount: Long, defaultToUsingSubgroup: Option[Boolean]) = useSubgroup(highlightedEntry)
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
                    val ans: Option[Entity] = controller.askForNameAndWriteEntity(groupIn.mDB, Util.ENTITY_TYPE, leadingTextIn = Some("NAME THE ENTITY:"),
                                                                                  classIdIn = groupIn.getClassId)
                    if (ans.isDefined) {
                      val newEntityId: Long = ans.get.getId
                      val newEntity: Entity = ans.get
                      val targetGroup = new Group(groupIn.mDB, targetGroupId)
                      targetGroup.addEntity(newEntityId)

                      controller.defaultAttributeCopying(newEntity)

                      val newRtg: RelationToGroup = new RelationToGroup(groupIn.mDB, rtgId, highlightedEntry.getId, relTypeId, targetGroup.getId)
                      quickGroupMenu(new Group(targetGroup.mDB, targetGroup.getId), 0, Some(newRtg), None, None, containingEntityIn = Some(highlightedEntry))
                    }
                    quickGroupMenu(groupIn, startingDisplayRowIndexIn, relationToGroupIn, Some(highlightedEntry), targetForMoves, callingMenusRtgIn, containingEntityIn)
                  }
                } else {
                  val newEntity: Option[Entity] = controller.askForNameAndWriteEntity(groupIn.mDB, Util.ENTITY_TYPE, leadingTextIn = Some("NAME THE ENTITY:"),
                                                                                      classIdIn = groupIn.getClassId)
                  if (newEntity.isDefined) {
                    val newEntityId: Long = newEntity.get.getId
                    val newRte: RelationToLocalEntity = highlightedEntry.addHASRelationToLocalEntity(newEntityId, None, System.currentTimeMillis())
                    require(newRte.getParentId == highlightedEntry.getId)
                    new EntityMenu(ui, controller).entityMenu(newEntity.get, containingRelationToEntityIn = Some(newRte))
                  }
                  quickGroupMenu(groupIn, startingDisplayRowIndexIn, relationToGroupIn, newEntity, targetForMoves, callingMenusRtgIn, containingEntityIn)
                }
              } else {
                quickGroupMenu(groupIn, startingDisplayRowIndexIn, relationToGroupIn, Some(highlightedEntry), targetForMoves, callingMenusRtgIn, containingEntityIn)
              }
            } else if (addEntryAnswer == 2) {
              val entityChosen: Option[IdWrapper] = controller.askForNameAndSearchForEntity(groupIn.mDB)
              val (entryToHighlight:Option[Entity], displayStartingRowNumber: Int) = {
                if (entityChosen.isDefined) {
                  val entityChosenId: Long = entityChosen.get.getId
                  groupIn.addEntity(entityChosenId)
                  // (See comment at similar place in EntityMenu, just before that call to placeEntryInPosition.)
                  val goingBackward: Boolean = highlightedIndexInObjList == 0 && groupIn.getNewEntriesStickToTop
                  val forward = !goingBackward
                  val newDisplayStartingRowNumber: Int = placeEntryInPosition(groupIn.mDB, groupIn.getId, groupIn.getSize(4), 0, forwardNotBackIn = forward,
                                                                              startingDisplayRowIndexIn, entityChosenId, highlightedIndexInObjList,
                                                                              Some(highlightedObjId), objectsToDisplay.size, -1, Some(-1))
                  (Some(new Entity(groupIn.mDB, entityChosenId)), newDisplayStartingRowNumber)
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
          new EntityMenu(ui, controller).entityMenu(highlightedEntry, containingGroupIn = Some(groupIn))
          // deal with entityMenu possibly having deleted the entity:
          val removedOne: Boolean = !groupIn.isEntityInGroup(highlightedEntry.getId)
          val entityToHighlightNext: Option[Entity] = Util.findEntityToHighlightNext(objIds.length, objectsToDisplay, removedOne, highlightedIndexInObjList,
                                                                               highlightedEntry)
          quickGroupMenu(groupIn, startingDisplayRowIndexIn, relationToGroupIn, entityToHighlightNext, targetForMoves, callingMenusRtgIn, containingEntityIn)
        } else if (answer == 6) {
          val (entryToHighlight: Option[Entity], displayStartingRowNumber: Int) = {
            val nextStartPosition = startingDisplayRowIndexIn + objectsToDisplay.size
            if (nextStartPosition >= groupIn.getSize(4)) {
              ui.displayText("End of attribute list found; restarting from the beginning.")
              (None, 0) // start over
            } else (highlightedEntityIn, nextStartPosition)
          }
          quickGroupMenu(groupIn, displayStartingRowNumber, relationToGroupIn, entryToHighlight, targetForMoves, callingMenusRtgIn, containingEntityIn)
        } else if (answer == 7) {
          // NOTE: this code is similar (not identical) in EntityMenu as in QuickGroupMenu: if one changes,
          // THE OTHER MIGHT ALSO NEED MAINTENANCE!
          val choices = Array[String](Util.unselectMoveTargetPromptText)
          val leadingText: Array[String] = Array(Util.unselectMoveTargetLeadingText)
          Util.addRemainingCountToPrompt(choices, objectsToDisplay.size, groupIn.getSize(4), startingDisplayRowIndexIn)

          val response = ui.askWhich(Some(leadingText), choices, statusesAndNames, highlightIndexIn = Some(highlightedIndexInObjList),
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
          quickGroupMenu(groupIn, startingDisplayRowIndexIn, relationToGroupIn, entryToHighlight, selectedTargetEntity, callingMenusRtgIn, containingEntityIn)
        } else if (answer == 8) {
          // lets user select a new entity or group for further operations like moving, deleting.
          // (we have to have at least one choice or ui.askWhich fails...a require() call there.)
          // NOTE: this code is similar (not identical) in EntityMenu as in QuickGroupMenu: if one changes,
          // THE OTHER MIGHT ALSO NEED MAINTENANCE!
          val choices = Array[String]("keep existing (same as ESC)")
          // says 'same screenful' because (see similar cmt elsewhere).
          val leadingText: Array[String] = Array("CHOOSE AN ENTRY to highlight (*)")
          Util.addRemainingCountToPrompt(choices, objectsToDisplay.size, groupIn.getSize(4), startingDisplayRowIndexIn)
          val response = ui.askWhich(Some(leadingText), choices, statusesAndNames, highlightIndexIn = Some(highlightedIndexInObjList),
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
          new GroupMenu(ui, controller).groupMenu(groupIn, startingDisplayRowIndexIn, relationToGroupIn, callingMenusRtgIn, containingEntityIn)
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

            val removedOne: Boolean = !groupIn.isEntityInGroup(userSelection.getId)
            var entityToHighlightNext: Option[Entity] = Some(userSelection)
            if (groupId.isDefined && !moreThanOneGroupAvailable) {
              //idea: do something w/ this unused variable? Like, if the userSelection was deleted, then use this in its place in parms to
              // qGM just below? or what was it for originally?  Or, del this var around here?
              entityToHighlightNext = Util.findEntityToHighlightNext(objIds.length, objectsToDisplay, removedOne, highlightedIndexInObjList, highlightedEntry)
            }

            //ck 1st if it exists, if not return None. It could have been deleted while navigating around.
            if (groupIn.mDB.groupKeyExists(groupIn.getId)) {
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

  private def useSubgroup(targetEntry: Entity): (Long, Option[Boolean]) = {
    val targetRtgCount: Long = targetEntry.getRelationToGroupCount
    val defaultToUsingSubgroup: Option[Boolean] = {
      if (targetRtgCount == 0) {
        Some(false)
      } else if (targetRtgCount == 1) {
        Some(true)
      } else {
        ui.displayText("There are multiple subgroups on this entity, so for now OM will just move the one entry to be contained by the" +
                       " other entity, to which you can then manually go and move it further to a subgroup as needed.")
        Some(false)
      }
    }
    (targetRtgCount, defaultToUsingSubgroup)
  }

  protected def getAdjacentEntriesSortingIndexes(dbIn: Database, groupIdIn: Long, movingFromPosition_sortingIndexIn: Long, queryLimitIn: Option[Long],
                                        forwardNotBackIn: Boolean): List[Array[Option[Any]]] = {
    val group = new Group(dbIn, groupIdIn)
    group.getAdjacentGroupEntriesSortingIndexes(movingFromPosition_sortingIndexIn, queryLimitIn, forwardNotBackIn)
  }

  protected def getNearestEntrysSortingIndex(dbIn: Database, groupIdIn: Long, startingPointSortingIndexIn: Long, forwardNotBackIn: Boolean): Option[Long] = {
    val group = new Group(dbIn, groupIdIn)
    group.getNearestGroupEntrysSortingIndex(startingPointSortingIndexIn, forwardNotBackIn = forwardNotBackIn)
  }

  protected def renumberSortingIndexes(dbIn: Database, groupIdIn: Long): Unit = {
    val group = new Group(dbIn, groupIdIn)
    group.renumberSortingIndexes()
  }

  protected def updateSortedEntry(dbIn: Database, groupIdIn: Long, ignoredParameter: Int, movingEntityIdIn: Long, sortingIndexIn: Long): Unit = {
    val group = new Group(dbIn, groupIdIn)
    group.updateSortingIndex(movingEntityIdIn, sortingIndexIn)
  }

  protected def getSortingIndex(dbIn: Database, groupIdIn: Long, ignoredParameter: Int, entityIdIn: Long): Long = {
    val group = new Group(dbIn, groupIdIn)
    group.getEntrySortingIndex(entityIdIn)
  }

  protected def indexIsInUse(dbIn: Database, groupIdIn: Long, sortingIndexIn: Long): Boolean = {
    val group = new Group(dbIn, groupIdIn)
    group.isGroupEntrySortingIndexInUse(sortingIndexIn)
  }

  protected def findUnusedSortingIndex(dbIn: Database, groupIdIn: Long, startingWithIn: Long): Long = {
    val group = new Group(dbIn, groupIdIn)
    group.findUnusedSortingIndex(Some(startingWithIn))
  }

}
