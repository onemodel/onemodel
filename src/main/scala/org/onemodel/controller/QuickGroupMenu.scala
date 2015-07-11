/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003-2004 and 2008-2015 inclusive, Luke A Call; all rights reserved.
    (That copyright statement was previously 2013-2015, until I remembered that much of Controller came from TextUI.scala and TextUI.java before that.)
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule, guidelines around binary
    distribution, and the GNU Affero General Public License as published by the Free Software Foundation, either version 3
    of the License, or (at your option) any later version.  See the file LICENSE for details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
package org.onemodel.controller

import org.onemodel.model._
import org.onemodel.{TextUI, OmException}
import org.onemodel.database.PostgreSQLDatabase

class QuickGroupMenu(val ui: TextUI, val db: PostgreSQLDatabase, val controller: Controller) {
  /** returns None if user wants out. The parameter startingDisplayRowIndexIn refers to the 0-based index among all possible displayable rows (i.e.,
    * if we have displayed
    * 20 objects out of 100, and the user says to go to the next 20, the startingDisplayRowIndexIn would become 21. */
  // The @tailrec is desired when possible,
  // because it seems that otherwise we might try to ESC back to a menu instance which is attempting to view a deleted entity, & crash!  But see the comment
  // mentioning why not to have it, below.  Maybe we need to use a loop around the menu instead of tail recursion in this case, if there is not a
  // way to turn the tail optimization off for a particular line.
  //@tailrec
  //scoping idea: see idea at beginning of EntityMenu.entityMenu
  def quickGroupMenu(startingDisplayRowIndexIn: Long, relationToGroupIn: RelationToGroup, highlightedEntityIn: Option[Entity] = None,
                     targetForMovesIn: Option[Entity] = None, callingMenusRtgIn: Option[RelationToGroup] = None): Option[Entity] = {
    val group = new Group(db, relationToGroupIn.getGroupId)

    try {
      quickGroupMenu_doTheWork(group, startingDisplayRowIndexIn, relationToGroupIn, highlightedEntityIn, targetForMovesIn, callingMenusRtgIn)
    } catch {
      case e: Exception =>
        controller.handleException(e)
        val ans = ui.askYesNoQuestion("Go back to what you were doing (vs. going out)?", Some("y"))
        if (ans != None && ans.get) quickGroupMenu(startingDisplayRowIndexIn, relationToGroupIn, highlightedEntityIn, targetForMovesIn, callingMenusRtgIn)
        else None
    }

  }

  def findNewNeighbors(groupIn: Group, movingDistanceIn: Int, forwardNotBackIn: Boolean, movingFromPosition_sortingIndex: Long): (Long, Option[Long],
    Option[Long]) = {

    // (idea: this could probably be made more efficient by combining the 2nd part of the (fixed) algorithm (the call to mDB.getNearestGroupEntryInRange)
    // with the first part.  I.e., maybe we don't need to calculate the farNewNeighborSortingIndex at first, since we're just going to soon replace
    // it with the "next one after the nearNewNeighbor" anyway.  But first it should have some good tests around it: coverage.)

    // get enough data to represent the new location in the sort order: movingDistanceIn entries away, and one beyond, and place this entity between them:
    val queryLimit = movingDistanceIn + 1

    val results: Array[Array[Option[Any]]] = db.getAdjacentGroupEntries(groupIn.getId, movingFromPosition_sortingIndex, Some(queryLimit),
                                                                         forwardNotBackIn = forwardNotBackIn).toArray
    require(results.size <= queryLimit)
    // (get the last result's sortingIndex, if possible; 0-based of course; i.e., that of the first entry beyond where we're moving to):
    val farNewNeighborSortingIndex: Option[Long] =
      if (results.size > 0 && results.size == queryLimit) results(results.size - 1)(1).asInstanceOf[Option[Long]]
      else None
    val (nearNewNeighborSortingIndex: Option[Long], byHowManyEntriesMoving: Int) = {
      if (results.size == 0) {
        // there's nowhere to move to, so just get out of here (shortly, as noted in the caller)
        (None, 0)
      } else if (results.size == queryLimit) {
        if (queryLimit == 1) (Some(movingFromPosition_sortingIndex), 1)
        else {
          // get the next-to-last result's sortingIndex
          (results(queryLimit - 2)(1).asInstanceOf[Option[Long]], results.size - 1)
        }
      } else {
        // given the 'require' statement above, results.size now has to be between 0 and queryLimit, so use the last result as the "near new neighbor", and
        // move just beyond that
        (results(results.size - 1)(1).asInstanceOf[Option[Long]], results.size)
      }
    }

    //(idea: make this comment shorter/clearer, still complete)
    /** HERE in these methods, we need to do the counting of how far to move (eg., how many entries down to go...) etc (ie in method findNewNeighbors)
      based on what is *not* archived (in order to move it the same # of entries as the user expects from seeing
      the UI visually displaying those that are not archived), but adjust the farNewNeighbor, so that we can calculate the new sorting_index based on what *is*
      archived! (unless we're *displaying* archived things: IDEA:  find how to make it work well and *simply*, both ways??)
      So, now account for the fact that there could be archived entities between the 2 new neighbors, previously ignored, but at this point we will
      recalculate the farNewNeighbor, so that the later calculation of the sorting_index doesn't collide with an existing, but archived, entity:
      */
    val adjustedFarNewNeighborSortingIndex:Option[Long] = {
      if (nearNewNeighborSortingIndex == None || farNewNeighborSortingIndex == None)
        None
      else db.getNearestGroupEntry(groupIn.getId, nearNewNeighborSortingIndex.get, forwardNotBackIn = forwardNotBackIn)
    }

    (byHowManyEntriesMoving, nearNewNeighborSortingIndex, adjustedFarNewNeighborSortingIndex)
  }

  def getNewSortingIndex(groupIn: Group, startingDisplayRowIndexIn: Long, nearNewNeighborSortingIndex: Option[Long],
                         farNewNeighborSortingIndex: Option[Long], forwardNotBack: Boolean,
                         byHowManyEntriesMoving: Long, movingFromPosition_sortingIndex: Long, moveFromIndexInObjListIn: Long,
                         numDisplayLines: Int): (Long, Boolean, Long) = {
    if (nearNewNeighborSortingIndex == None) {
      throw new OmException("never should have got here: should have been the logic of ~nowhere to go so doing nothing")
    }

    val (newIndex: Long, trouble: Boolean) = {
      if (farNewNeighborSortingIndex == None) {
        //halfway between min value of a long (or max, depending on direction of the move), and whatever highlightIndexIn's long (sorting_index) is now
        if (forwardNotBack) {
          // do calculation as float or it wraps & gets wrong result, with inputs like this (idea: unit tests....)
          //     scala> -3074457345618258604L + ((9223372036854775807L - -3074457345618258604L) / 2)
          //     res2: Long = -6148914691236517206
          val newIndex = (nearNewNeighborSortingIndex.get + ((db.maxIdValue.asInstanceOf[Float] - nearNewNeighborSortingIndex.get) / 2)).asInstanceOf[Long]
          // leaving it to communicate intent, but won't be '>' because a Long would just wrap, so...
          val trouble: Boolean = newIndex > db.maxIdValue || newIndex <= movingFromPosition_sortingIndex || newIndex <= nearNewNeighborSortingIndex.get
          (newIndex, trouble)
        } else {
          val newIndex = nearNewNeighborSortingIndex.get - math.abs((math.abs(db.minIdValue) - math.abs(nearNewNeighborSortingIndex.get)) / 2)
          // leaving it to communicate intent, but won't be '<' because a Long would just wrap, so...
          val trouble: Boolean = newIndex < db.minIdValue || newIndex >= movingFromPosition_sortingIndex || newIndex >= nearNewNeighborSortingIndex.get
          (newIndex, trouble)
        }
      } else {
        val halfDistance: Long = math.abs(farNewNeighborSortingIndex.get - nearNewNeighborSortingIndex.get) / 2
        val newIndex: Long = {
                               // a Float so it won't wrap around:
                               if (forwardNotBack) nearNewNeighborSortingIndex.get.asInstanceOf[Float] + halfDistance
                               else nearNewNeighborSortingIndex.get - halfDistance
                             }.asInstanceOf[Long]
        // leaving this comment to communicate intent, but won't be '<' or '>' because a Long would just wrap, so...
        val trouble: Boolean =
          if (forwardNotBack) {
            newIndex <= movingFromPosition_sortingIndex || newIndex >= farNewNeighborSortingIndex.get || newIndex <= nearNewNeighborSortingIndex.get
          } else newIndex >= movingFromPosition_sortingIndex || newIndex <= farNewNeighborSortingIndex.get || newIndex >= nearNewNeighborSortingIndex.get
        (newIndex, trouble)
      }
    }

    val newDisplayRowsStartingWithCounter: Long = {
      if (forwardNotBack) {
        if ((moveFromIndexInObjListIn + byHowManyEntriesMoving) > numDisplayLines) {
          // if the object will move too far to be seen in this screenful, adjust the screenful to redisplay, with some margin
          math.min(groupIn.groupSize - numDisplayLines,
                   startingDisplayRowIndexIn + numDisplayLines + byHowManyEntriesMoving -
                   //(was: "/ 4", but center it better in the screen):
                   (numDisplayLines / 2))
        } else startingDisplayRowIndexIn
      } else {
        if ((moveFromIndexInObjListIn - byHowManyEntriesMoving) < 0) {
          // if the object will move too far to be seen in this screenful, adjust the screenful to redisplay, with some margin
          // (was: "/ 4", but center it better in the screen):
          math.max(0, startingDisplayRowIndexIn - byHowManyEntriesMoving - (numDisplayLines / 2))
        } else startingDisplayRowIndexIn
      }
    }

    (newIndex, trouble, newDisplayRowsStartingWithCounter)
  }

  def createNewOrFindOneGroupOnEntity(groupIn: Group, targetEntitysRtgCount: Long, targetEntityIn: Entity): (Long, Long) = {
    // if there is 1 (obvious) destination, or no RTG on the selected entity (1 can be created), then add new entry there
    val (targetRelTypeId: Long, targetGroupId: Long) = {
      if (targetEntitysRtgCount == 0) {
        val name: String = targetEntityIn.getName
        val (newGroup: Group, newRTG: RelationToGroup) = targetEntityIn.createGroupAndAddHASRelationToIt(name, groupIn.getMixedClassesAllowed,
                                                                                                         System.currentTimeMillis)
        (newRTG.getAttrTypeId, newGroup.getId)
      } else {
        // given above conditions (w/ moveTargetIndexInObjList, and rtgCount twice), there must be exactly one, or there's a bug:
        val (relTypeId, gid, moreAvailable) = db.findRelationToAndGroup_OnEntity(targetEntityIn.getId)
        if (gid == None || relTypeId == None || moreAvailable) throw new OmException("Found " + (if (gid == None) 0 else ">1") + " but by the earlier " +
                                                                                     "checks, " +
                                                                                     "there should be exactly one group in entity " + targetEntityIn.getId
                                                                                     + ": " +
                                                                                     targetEntityIn.getName)
        (relTypeId.get, gid.get)
      }
    }
    (targetRelTypeId, targetGroupId)
  }

  def moveSelectedEntry(groupIn: Group, startingDisplayRowIndexIn: Long, relationToGroupIn: RelationToGroup, targetForMovesIn: Option[Entity],
                        highlightedIndexInObjListIn: Int, moveTargetIndexInObjList: Option[Int], highlightedEntry: Entity,
                        highlightedObjId: Long, objIds: Array[Long], objectsToDisplay: java.util.ArrayList[Entity],
                        callingMenusRtgIn: Option[RelationToGroup] = None): Option[Entity] = {
    val choices = Array[String](// these are ordered for convenience in doing them w/ the left hand: by frequency of use, and what seems easiest to remember
                                // for common operations with the 4 fingers sitting on the '1234' keys.  Using LH more in this because my RH gets tired more,
                                // and it seems like often people have their RH on the mouse.
                                "Move up 5", "Move up 1", "Move down 1", "Move down 5",

                                if (targetForMovesIn != None) "Move (*) to (sole, if present) subgroup of selected target (+, if any)"
                                else "(stub: have to choose a target before you can move entries to it)",

                                "Move (*) to calling menu (up one)",
                                "Move up 25",
                                "Move down 25"
                                // idea: make an option #9 here which is a "quick archive"? (for removing completed tasks: maybe only after showing archived things works
                                // well)
                               )
    val response = ui.askWhich(None, choices, Array[String](), highlightIndexIn = Some(highlightedIndexInObjListIn),
                               secondaryHighlightIndexIn = moveTargetIndexInObjList)
    if (response == None) quickGroupMenu(highlightedIndexInObjListIn, relationToGroupIn, Some(highlightedEntry), targetForMovesIn, callingMenusRtgIn)
    else {
      val answer = response.get
      if (answer == 1) {
        val displayStartingRowNumber: Long = placeEntryInPosition(groupIn, 5, forwardNotBackIn = false, startingDisplayRowIndexIn, highlightedObjId,
                                                                  highlightedIndexInObjListIn, highlightedObjId, objectsToDisplay.size)
        quickGroupMenu(displayStartingRowNumber, relationToGroupIn, Some(highlightedEntry), targetForMovesIn, callingMenusRtgIn)
      } else if (answer == 2) {
        val displayStartingRowNumber: Long = placeEntryInPosition(groupIn, 1, forwardNotBackIn = false, startingDisplayRowIndexIn, highlightedObjId,
                                                                  highlightedIndexInObjListIn, highlightedObjId, objectsToDisplay.size)
        quickGroupMenu(displayStartingRowNumber, relationToGroupIn, Some(highlightedEntry), targetForMovesIn, callingMenusRtgIn)
      } else if (answer == 3) {
        val displayStartingRowNumber: Long = placeEntryInPosition(groupIn, 1, forwardNotBackIn = true, startingDisplayRowIndexIn, highlightedObjId,
                                                                  highlightedIndexInObjListIn, highlightedObjId, objectsToDisplay.size)
        quickGroupMenu(displayStartingRowNumber, relationToGroupIn, Some(highlightedEntry), targetForMovesIn, callingMenusRtgIn)
      } else if (answer == 4) {
        val displayStartingRowNumber: Long = placeEntryInPosition(groupIn, 5, forwardNotBackIn = true, startingDisplayRowIndexIn, highlightedObjId,
                                                                  highlightedIndexInObjListIn, highlightedObjId, objectsToDisplay.size)
        quickGroupMenu(displayStartingRowNumber, relationToGroupIn, Some(highlightedEntry), targetForMovesIn, callingMenusRtgIn)
      } else if (answer == 5 && targetForMovesIn != None) {
        val targetRtgCount: Long = db.getRelationToGroupCountByEntity(Some(targetForMovesIn.get.getId))
        if (moveTargetIndexInObjList == None || targetRtgCount > 1) {
          // IDEA: could guess & move it in even if >1 subgroup present, by seeing which subgroup has the same class, if only 1 like that? Or if same-named?
          ui.displayText("Target must be selected (shows '+'), and must have exactly one subgroup (a single '>'), or none.")
          quickGroupMenu(startingDisplayRowIndexIn, relationToGroupIn, Some(highlightedEntry), targetForMovesIn, callingMenusRtgIn)
        } else {
          // if there is 1 (obvious) destination, or no RTG on the selected entity (1 can be created), then move it there
          val (_, targetGroupId) = createNewOrFindOneGroupOnEntity(groupIn, targetRtgCount, targetForMovesIn.get)
          // about the sortingIndex:  see comment on db.moveEntityToNewGroup.
          db.moveEntityToNewGroup(targetGroupId, groupIn.getId, highlightedObjId, db.getSortingIndex(groupIn.getId, highlightedObjId))
          val entityToHighlight: Option[Entity] = controller.findEntryToHighlightNext(objIds, objectsToDisplay, deletedOrArchivedOneIn = true,
                                                                           highlightedIndexInObjListIn, highlightedEntry)
          quickGroupMenu(startingDisplayRowIndexIn, relationToGroupIn, entityToHighlight, targetForMovesIn, callingMenusRtgIn)
        }
      } else if (answer == 6) {
        // if there is 1 (provided or obvious) destination), then move it there
        val targetGroupId: Option[Long] = {
          if (callingMenusRtgIn != None) {
            Some(callingMenusRtgIn.get.getGroupId)
          } else {
            // none provided, so see if it's guessable
            val containingGroups: List[Array[Option[Any]]] = db.getContainingGroupsIds(groupIn.getId)
            if (containingGroups.size == 0) {
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
        if (targetGroupId == None) quickGroupMenu(startingDisplayRowIndexIn, relationToGroupIn, Some(highlightedEntry), targetForMovesIn, callingMenusRtgIn)
        else {
          db.moveEntityToNewGroup(targetGroupId.get, groupIn.getId, highlightedObjId, db.getSortingIndex(groupIn.getId,
                                                                                                           highlightedObjId))
          val entityToHighlight: Option[Entity] = controller.findEntryToHighlightNext(objIds, objectsToDisplay, deletedOrArchivedOneIn = true,
                                                                           highlightedIndexInObjListIn, highlightedEntry)
          quickGroupMenu(startingDisplayRowIndexIn, relationToGroupIn, entityToHighlight, targetForMovesIn, callingMenusRtgIn)
        }
      } else if (answer == 7) {
        val displayStartingRowNumber: Long = placeEntryInPosition(groupIn, 20, forwardNotBackIn = false, startingDisplayRowIndexIn, highlightedObjId,
                                                                  highlightedIndexInObjListIn, highlightedObjId, objectsToDisplay.size)
        quickGroupMenu(displayStartingRowNumber, relationToGroupIn, Some(highlightedEntry), targetForMovesIn, callingMenusRtgIn)
      } else if (answer == 8) {
        val displayStartingRowNumber: Long = placeEntryInPosition(groupIn, 20, forwardNotBackIn = true, startingDisplayRowIndexIn, highlightedObjId,
                                                                  highlightedIndexInObjListIn, highlightedObjId, objectsToDisplay.size)
        quickGroupMenu(displayStartingRowNumber, relationToGroupIn, Some(highlightedEntry), targetForMovesIn, callingMenusRtgIn)
      } else {
        quickGroupMenu(startingDisplayRowIndexIn, relationToGroupIn, Some(highlightedEntry), targetForMovesIn, callingMenusRtgIn)
      }
    }
  }

  def quickGroupMenu_doTheWork(groupIn: Group, startingDisplayRowIndexIn: Long, relationToGroupIn: RelationToGroup, highlightedEntityIn: Option[Entity] = None,
                               targetForMovesIn: Option[Entity] = None, callingMenusRtgIn: Option[RelationToGroup] = None): Option[Entity] = {
    require(groupIn != null)
    val choices = Array[String]("Create new entry",
                                "Move selection (*) up/down, in, out...",
                                "Edit the selected entry's name",
                                "Create new entry INSIDE or UNDER selected entry",
                                //"Delete or Archive the selection (*)...",
                                "Go to selected entity (not the subgroup)",
                                "Find existing entry to add / list next items...",
                                "Select target (entry move destination: gets a '+')",
                                "Select entry to highlight (with '*'; typing the letter instead goes to the subgroup if any, else to that entity)",
                                "Other (slower actions, more complete menu)")
    val attrType = Some(new RelationType(db, relationToGroupIn.getAttrTypeId))
    val leadingText: Array[String] = Array("ENTITY GROUP (quick menu: acts on (w/ #'s) OR selects (w/ letters...) an entity): " +
                                           relationToGroupIn.getDisplayString(0, None, attrType))
    val numDisplayableItems = ui.maxColumnarChoicesToDisplayAfter(leadingText.length, choices.size, controller.maxNameLength)
    val objectsToDisplay: java.util.ArrayList[Entity] = groupIn.getGroupEntries(startingDisplayRowIndexIn, Some(numDisplayableItems))
    val objIds = for (entity: Entity <- objectsToDisplay.toArray(Array[Entity]())) yield {
      entity.getId
    }
    controller.addRemainingCountToPrompt(choices, objectsToDisplay.size, groupIn.groupSize, startingDisplayRowIndexIn)
    val names: Array[String] = for (entity: Entity <- objectsToDisplay.toArray(Array[Entity]())) yield {
      val numSubgroupsPrefix: String = controller.getNumSubgroupsPrefix(entity.getId)
      numSubgroupsPrefix + entity.getName + " " + entity.getPublicStatusString()
    }
    if (objIds.length == 0) {
      val response = ui.askWhich(Some(leadingText), Array[String]("Add entry", "Other (slower, more complete menu)"), Array[String](),
                                 highlightIndexIn = None)
      if (response == None) None
      else {
        val answer = response.get
        if (answer == 1) {
          controller.addEntityToGroup(groupIn)
          quickGroupMenu(0, relationToGroupIn, callingMenusRtgIn = callingMenusRtgIn)
        } else if (answer == 2 && answer <= choices.size) {
          new GroupMenu(ui, db, controller).groupMenu(startingDisplayRowIndexIn, relationToGroupIn, callingMenusRtgIn = callingMenusRtgIn)
        } else if (answer == 0) None
        else {
          // expected to be unreachable based on askWhich behavior (doesn't allow answers beyond the list of choices available), but for the compiler:
          None
        }
      }
    } else {
      // Some of this stuff is a kludge:  I'm tired and need to get something usable more than I need elegance now. Idea: improve it,
      // especially the types used here & in related code.
      // Be sure the code is OK even if the highlightedEntityIn isn't really in the list due to caller logic error, etc.
      val (highlightedIndexInObjList: Int, highlightedObjId: Long, highlightedEntry: Entity, moveTargetIndexInObjList: Option[Int],
      targetForMoves: Option[Entity]) = {
        var highlightedObjId: Long = if (highlightedEntityIn == None) objIds(0) else highlightedEntityIn.get.getId
        var highlightedIndexInObjList: Int = {
          val index = objIds.indexOf(highlightedObjId)
          // if index == -1 then there could be a logic error where an entity not in the list was passed in, or an entry was moved and we're not displaying
          // the portion of the list containing that entry.  Regardless, don't bomb with AIOOBE due to the -1 later, just make it None.
          if (index < 0) {
            highlightedObjId = objIds(0)
            0
          }
          else index
        }
        var moveTargetIndexInObjList: Option[Int] = if (targetForMovesIn == None) None
                                                    else {
                                                      val index = objIds.indexOf(targetForMovesIn.get.getId)
                                                      // same as just above: don't bomb w/ a -1
                                                      if (index < 0) None
                                                      else Some(index)
                                                    }
        if (moveTargetIndexInObjList != None && highlightedIndexInObjList == moveTargetIndexInObjList.get) {
          // doesn't make sense if they're equal (ie move both, into both?, like if user changed the previous highlight on 1st selection to a move
          // target), so change one:
          if (highlightedIndexInObjList == 0 && objIds.size > 1) {
            highlightedIndexInObjList = 1
          } else {
            moveTargetIndexInObjList = None
          }
        }
        assert(highlightedIndexInObjList >= 0)
        val highlightedEntry: Entity = objectsToDisplay.get(highlightedIndexInObjList)
        highlightedObjId = highlightedEntry.getId
        val targetForMoves: Option[Entity] = if (moveTargetIndexInObjList == None) None
                                             else Some(objectsToDisplay.get(moveTargetIndexInObjList.get))
        (highlightedIndexInObjList, highlightedObjId, highlightedEntry, moveTargetIndexInObjList, targetForMoves)
      }
      if (highlightedIndexInObjList == moveTargetIndexInObjList.getOrElse(None)) {
        throw new OmException("We have wound up with the same entry for targetForMoves and highlightedEntry: that will be a problem: aborting before we put" +
                              " an entity in its own group and lose track of it or something.")
      }


      val response = ui.askWhich(Some(leadingText), choices, names, highlightIndexIn = Some(highlightedIndexInObjList),
                                 secondaryHighlightIndexIn = moveTargetIndexInObjList)
      if (response == None) None
      else {
        val answer = response.get
        if (answer == 1) {
          val (entryToHighlight:Option[Entity], displayStartingRowNumber:Long) = {
            // ask for less info when here in the quick menu, where want to add entity quickly w/ no fuss, like brainstorming.  User can always use long menu.
            val ans: Option[Entity] = controller.askForNameAndWriteEntity(Controller.ENTITY_TYPE, inLeadingText = Some("NAME THE ENTITY:"),
                                                               inClassId = groupIn.getClassId)
            if (ans != None) {
              val newEntityId: Long = ans.get.getId
              db.addEntityToGroup(groupIn.getId, newEntityId)
              val displayStartingRowNumber: Long = placeEntryInPosition(groupIn, 0, forwardNotBackIn = true, startingDisplayRowIndexIn, newEntityId,
                                                                        highlightedIndexInObjList, highlightedObjId, objectsToDisplay.size)
              (Some(new Entity(db, newEntityId)), displayStartingRowNumber)
            }
            else
              (highlightedEntityIn, startingDisplayRowIndexIn)
          }
          quickGroupMenu(displayStartingRowNumber, relationToGroupIn, entryToHighlight, targetForMoves, callingMenusRtgIn = callingMenusRtgIn)
        } else if (answer == 2) {
          moveSelectedEntry(groupIn, startingDisplayRowIndexIn, relationToGroupIn, targetForMoves, highlightedIndexInObjList, moveTargetIndexInObjList,
                            highlightedEntry, highlightedObjId, objIds, objectsToDisplay)
        } else if (answer == 3) {
          val editedEntity: Option[Entity] = controller.editEntityName(highlightedEntry)
          if (editedEntity == None)
            quickGroupMenu(startingDisplayRowIndexIn, relationToGroupIn, Some(highlightedEntry), targetForMoves, callingMenusRtgIn)
          else {
            quickGroupMenu(startingDisplayRowIndexIn, relationToGroupIn, editedEntity, targetForMoves, callingMenusRtgIn)
          }
        } else if (answer == 4) {
          //feature idea: askWhich(create it INSIDE or UNDER selected entity "(creating a subgroup on entity if needed)")
          val targetRtgCount: Long = db.getRelationToGroupCountByEntity(Some(highlightedEntry.getId))
          if (targetRtgCount > 1) {
            // IDEA: (see idea at similar logic above where entry is moved into a targeted group, about guessing which one)
            ui.displayText("For this operation, the selection must have exactly one subgroup (a single '>'), or none.")
            quickGroupMenu(startingDisplayRowIndexIn, relationToGroupIn, Some(highlightedEntry), targetForMovesIn, callingMenusRtgIn)
          } else {
            val (relTypeId: Long, targetGroupId: Long) = createNewOrFindOneGroupOnEntity(groupIn, targetRtgCount, highlightedEntry)
            // about the sortingIndex:  see comment on db.moveEntityToNewGroup.
            val ans: Option[Entity] = controller.askForNameAndWriteEntity(Controller.ENTITY_TYPE, inLeadingText = Some("NAME THE ENTITY:"),
                                                               inClassId = groupIn.getClassId)
            if (ans != None) {
              val newEntityId: Long = ans.get.getId
              //val newEntity = new Entity(mDB, newEntityId)
              db.addEntityToGroup(targetGroupId, newEntityId)
              val newRtg: RelationToGroup = new RelationToGroup(db, highlightedEntry.getId, relTypeId, targetGroupId)
              quickGroupMenu(0, newRtg, None, None)
            }
            quickGroupMenu(startingDisplayRowIndexIn, relationToGroupIn, Some(highlightedEntry), targetForMoves, callingMenusRtgIn)
          }
        } else if (answer == 5) {
          new EntityMenu(ui, db, controller).entityMenu(0, highlightedEntry, None, None, Some(groupIn))
          // deal with entityMenu possibly having deleted the entity:
          val deletedOrArchivedOne: Boolean = !db.isEntityInGroup(groupIn.getId, highlightedEntry.getId)
          val entityToHighlightNext: Option[Entity] = controller.findEntryToHighlightNext(objIds, objectsToDisplay, deletedOrArchivedOne, highlightedIndexInObjList,
                                                                               highlightedEntry)
          quickGroupMenu(startingDisplayRowIndexIn, relationToGroupIn, entityToHighlightNext, targetForMoves, callingMenusRtgIn = callingMenusRtgIn)
        } else if (answer == 6) {
          val choices = Array[String]("List next items...", "Search for *existing* entry, to insert after the selected one...")
          val response = ui.askWhich(None, choices, new Array[String](0))
          val (entryToHighlight: Option[Entity], displayStartingRowNumber: Long) = {
            if (response == None) {
              (highlightedEntityIn, startingDisplayRowIndexIn)
            } else {
              val answer = response.get
              if (answer == 1) {
                val nextStartPosition = startingDisplayRowIndexIn + objectsToDisplay.size
                if (nextStartPosition >= groupIn.groupSize) {
                  ui.displayText("End of attribute list found; restarting from the beginning.")
                  (None, 0L) // start over
                } else (highlightedEntityIn, nextStartPosition)
              } else if (answer == 2) {
                val entityChosen: Option[IdWrapper] = controller.chooseOrCreateObject(None, None, None, Controller.ENTITY_TYPE, 0, groupIn.getClassId,
                                                                           !groupIn.getMixedClassesAllowed, Some(groupIn.getId))
                if (entityChosen != None) {
                  val entityChosenId: Long = entityChosen.get.getId
                  db.addEntityToGroup(groupIn.getId, entityChosenId)
                  val newDisplayStartingRowNumber: Long = placeEntryInPosition(groupIn, 0, forwardNotBackIn = true, startingDisplayRowIndexIn,
                                                                            entityChosenId, highlightedIndexInObjList, highlightedObjId,
                                                                            objectsToDisplay.size)
                  (Some(new Entity(db, entityChosenId)), newDisplayStartingRowNumber)
                } else {
                  (highlightedEntityIn, startingDisplayRowIndexIn)
                }
              } else {
                ui.displayText("unexpected selection")
                (highlightedEntityIn, startingDisplayRowIndexIn)
              }
            }
          }
          quickGroupMenu(displayStartingRowNumber, relationToGroupIn, entryToHighlight, targetForMoves, callingMenusRtgIn = callingMenusRtgIn)
        } else if (answer == 7) {
          val choices = Array[String]("Unselect current move target (if present; not necessary really)")
          // says 'same screenful' because it's easier to assume that the returned index refers to the currently available
          // local collections (a subset of all possible entries, for display), than calling chooseOrCreateObject and sounds as useful:
          val leadingText: Array[String] = Array("CHOOSE AN ENTRY (that contains only one subgroup) FOR THE TARGET OF MOVES (choose from SAME SCREENFUL as " +
                                                 "now;  if the target contains 0 subgroups, or 2 or more subgroups, " +
                                                 "use other means to move entities to it until some kind of \"move anywhere\" feature is added):")
          controller.addRemainingCountToPrompt(choices, objectsToDisplay.size, groupIn.groupSize, startingDisplayRowIndexIn)

          val response = ui.askWhich(Some(leadingText), choices, names, highlightIndexIn = Some(highlightedIndexInObjList),
                                     secondaryHighlightIndexIn = moveTargetIndexInObjList)
          val (entityToHighlight, selectedTargetEntity): (Option[Entity], Option[Entity]) =
            if (response == None) (Some(highlightedEntry), targetForMoves)
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
          quickGroupMenu(startingDisplayRowIndexIn, relationToGroupIn, entityToHighlight, selectedTargetEntity, callingMenusRtgIn = callingMenusRtgIn)
        } else if (answer == 8) {
          // lets user select a new entity or group for further operations like moving, deleting.
          // (we have to have at least one choice or ui.askWhich fails...a require() call there.)
          val choices = Array[String]("keep existing (same as ESC)")
          // says 'same screenful' because (see similar cmt elsewhere).
          val leadingText: Array[String] = Array("CHOOSE AN ENTRY to highlight (*)")
          controller.addRemainingCountToPrompt(choices, objectsToDisplay.size, groupIn.groupSize, startingDisplayRowIndexIn)

          val response = ui.askWhich(Some(leadingText), choices, names, highlightIndexIn = Some(highlightedIndexInObjList),
                                     secondaryHighlightIndexIn = moveTargetIndexInObjList)
          val (entityToHighlight, selectedTargetEntity): (Option[Entity], Option[Entity]) =
            if (response == None) (Some(highlightedEntry), targetForMoves)
            else {
              val answer = response.get
              if (answer == 1) {
                (Some(highlightedEntry), targetForMoves)
              } else {
                // those in the condition are 1-based, not 0-based.
                // user typed a letter to select an attribute (now 0-based):
                val choicesIndex = answer - choices.length - 1
                val userSelection: Entity = objectsToDisplay.get(choicesIndex)
                if (choicesIndex == moveTargetIndexInObjList.getOrElse(None)) {
                  // chose same entity for the target, as the existing highlighted selection, so make it the target, and no highlighted one.
                  (Some(userSelection), None)
                } else {
                  (Some(userSelection), targetForMoves)
                }
              }
            }
          quickGroupMenu(startingDisplayRowIndexIn, relationToGroupIn, entityToHighlight, selectedTargetEntity, callingMenusRtgIn = callingMenusRtgIn)
        } else if (answer == 9 && answer <= choices.size) {
          new GroupMenu(ui, db, controller).groupMenu(startingDisplayRowIndexIn, relationToGroupIn, callingMenusRtgIn = callingMenusRtgIn)
        } else if (false /*can this be changed so that if they hit Enter it makes it to here ?*/ ) {
          // do something with enter: do a quick text edit & update the dates. Or quickAddEntry ?
          ui.displayText("not yet implemented")
          quickGroupMenu(startingDisplayRowIndexIn, relationToGroupIn, Some(highlightedEntry), targetForMoves, callingMenusRtgIn = callingMenusRtgIn)
        } else if (answer == 0) None
        else if (answer > choices.length && answer <= (choices.length + objectsToDisplay.size)) {
          // those in the condition are 1-based, not 0-based.
          // lets user go to an entity or group quickly (1 stroke)
          val choicesIndex = answer - choices.length - 1
          // user typed a letter to select an attribute (now 0-based)
          if (choicesIndex >= objectsToDisplay.size()) {
            ui.displayText("The program shouldn't have let us get to this point, but the selection " + answer + " is not in the list.")
            quickGroupMenu(startingDisplayRowIndexIn, relationToGroupIn, Some(highlightedEntry), targetForMoves, callingMenusRtgIn = callingMenusRtgIn)
          } else {
            val userSelection: Entity = objectsToDisplay.get(choicesIndex)

            val (_ /*subEntitySelected:Option[Entity]*/ , groupId: Option[Long], moreThanOneGroupAvailable) =
              controller.goToEntityOrItsSoleGroupsMenu(userSelection, Some(relationToGroupIn), Some(groupIn))

            val deletedOrArchivedOne: Boolean = !db.isEntityInGroup(groupIn.getId, userSelection.getId)
            var entityToHighlightNext: Option[Entity] = Some(userSelection)
            if (groupId != None && !moreThanOneGroupAvailable) {
              entityToHighlightNext = controller.findEntryToHighlightNext(objIds, objectsToDisplay, deletedOrArchivedOne, highlightedIndexInObjList, highlightedEntry)
              //idea: do something w/ this? Like, if the userSelection was deleted, then use this in its place in parms to qGM just below? or what was it for
              // originally?  Or, del this var around here?
            }

            if (choicesIndex == moveTargetIndexInObjList.getOrElse(None)) {
              quickGroupMenu(startingDisplayRowIndexIn, relationToGroupIn, Some(userSelection), None, callingMenusRtgIn = callingMenusRtgIn)
            } else {
              quickGroupMenu(startingDisplayRowIndexIn, relationToGroupIn, Some(userSelection), targetForMoves, callingMenusRtgIn = callingMenusRtgIn)
            }
          }
        } else {
          ui.displayText("invalid selection")
          quickGroupMenu(startingDisplayRowIndexIn, relationToGroupIn, Some(highlightedEntry), targetForMoves, callingMenusRtgIn)
        }
      }
    }
  }

  /** Returns the starting row number (in case the view window was adjusted to show other entries around the moved entity).  Note that if the goal
    * is to place a newly created object in the right spot in the list, then the parameters movingObjIdIn doesn't have to refer to the same object as
    * (moveFromIndexInObjListIn and objectIdAtThatIndex which are the same)!  But if it is to move an existing object, they should all be the same.
    */
  def placeEntryInPosition(groupIn: Group, numRowsToMoveIfThereAreThatManyIn: Int, forwardNotBackIn: Boolean, startingDisplayRowIndexIn: Long,
                           movingObjIdIn: Long, moveFromIndexInObjListIn: Long, objectIdAtThatIndex: Long, numDisplayLinesIn: Int): Long = {

    //val movingFromPosition_sortingIndex = mDB.getEntityInAGroupData(groupIn.getId, objectIdAtThatIndex)(0).get.asInstanceOf[Long]
    val movingFromPosition_sortingIndex = db.getSortingIndex(groupIn.getId, objectIdAtThatIndex)

    val (byHowManyEntriesActuallyMoving: Long, nearNewNeighborSortingIndex: Option[Long], farNewNeighborSortingIndex: Option[Long]) =
      findNewNeighbors(groupIn, numRowsToMoveIfThereAreThatManyIn, forwardNotBackIn, movingFromPosition_sortingIndex)

    var displayStartingRowNumber = startingDisplayRowIndexIn

    if (nearNewNeighborSortingIndex == None) {
      ui.displayText("Nowhere to move it to, so doing nothing.")
    } else {
      val (newSortingIndex: Long, trouble: Boolean) = {
        var (newSortingIndex: Long, trouble: Boolean, newStartingRowNum: Long) = {
          getNewSortingIndex(groupIn, startingDisplayRowIndexIn, nearNewNeighborSortingIndex, farNewNeighborSortingIndex, forwardNotBackIn,
                             byHowManyEntriesActuallyMoving, movingFromPosition_sortingIndex, moveFromIndexInObjListIn, numDisplayLinesIn)
        }
        displayStartingRowNumber = newStartingRowNum
        if (trouble) {
          db.renumberGroupSortingIndexes(groupIn.getId)

          // Get the sortingIndex of the one it was right after, increment (since just renumbered; or not?), then use that as the "old position" moving from.
          // (This is because the old movingFromPosition_sortingIndex value is now invalid, since we just renumbered above.)
          val movingFromPosition_sortingIndex2: Long = db.getSortingIndex(groupIn.getId, objectIdAtThatIndex)
          val (byHowManyEntriesMoving2: Long, nearNewNeighborSortingIndex2: Option[Long], farNewNeighborSortingIndex2: Option[Long]) =
            findNewNeighbors(groupIn, numRowsToMoveIfThereAreThatManyIn, forwardNotBackIn, movingFromPosition_sortingIndex2)
          // (for some reason, can't reassign the results directly to the vars like this "(newSortingIndex, trouble, newStartingRowNum) = ..."?
          val (a, b, c) = getNewSortingIndex(groupIn, startingDisplayRowIndexIn, nearNewNeighborSortingIndex2, farNewNeighborSortingIndex2, forwardNotBackIn,
                                             byHowManyEntriesMoving2, movingFromPosition_sortingIndex2, moveFromIndexInObjListIn, numDisplayLinesIn)
          newSortingIndex = a
          trouble = b
          newStartingRowNum = c
          displayStartingRowNumber = newStartingRowNum
        }
        (newSortingIndex, trouble)
      }

      if (trouble) {
        throw new OmException("Unable to determine a useful new sorting index. Renumbered, then came up with " + newSortingIndex + " but that " +
                              "still conflicts with something.")
      }
      else {
        db.updateEntityInAGroup(groupIn.getId, movingObjIdIn, newSortingIndex)
      }
    }
    displayStartingRowNumber
  }

}
