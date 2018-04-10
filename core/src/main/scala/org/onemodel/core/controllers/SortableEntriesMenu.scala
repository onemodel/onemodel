/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2014-2017 inclusive, Luke A. Call; all rights reserved.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule, guidelines around binary
    distribution, and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
package org.onemodel.core.controllers

import org.onemodel.core.model.Database
import org.onemodel.core.{OmException, TextUI}

abstract class SortableEntriesMenu(val ui: TextUI) {

  /** Returns the starting row number (in case the view window was adjusted to show other entries around the moved entity).
    *
    * The dbIn should represent the *same* database as where containingObjectIdIn is stored!  (See details in comments at similar location
    * about containingObjectIdIn.)
    *
    * Note that if the goal
    * is to place a newly created object in the right spot in the list, then the parameters movingObjIdIn doesn't have to refer to the same object as
    * (moveFromIndexInObjListIn and objectIdAtThatIndex which are the same)!  But if it is to move an existing object, they should all be the same.
    *
    * For the parameters  movingObjsAttributeFormIdIn: it is (eventually) ignored when this is called from the QuickGroupMenu,
    * but used when called from *EntityMenu. In
    * that case, the values for movingObjsAttributeFormIdIn and objectAtThatIndexFormIdIn would NOT be the same IN THE CASE WHERE (if, someday we have the
    * feature such that) the user inserts a new attribute after an existing one (ie specifying its position immediately instead of just moving it later) and
    * therefore the attribute already in that position, and the  one added at that position are different.
    */
  protected def placeEntryInPosition(dbIn: Database, containingObjectIdIn: Long, groupSizeOrNumAttributes_ToCalcNewDisplayStartingIndex_In: Long,
                                     numRowsToMoveIfThereAreThatManyIn: Int, forwardNotBackIn: Boolean,
                                     startingDisplayRowIndexIn: Int, movingObjIdIn: Long, moveFromIndexInObjListIn: Int, objectAtThatIndexIdIn: Option[Long],
                                     numDisplayLinesIn: Int, movingObjsAttributeFormIdIn: Int, objectAtThatIndexFormIdIn: Option[Int]): Int = {

    require(if (objectAtThatIndexIdIn.isDefined || objectAtThatIndexFormIdIn.isDefined) {
                objectAtThatIndexIdIn.isDefined && objectAtThatIndexFormIdIn.isDefined
            } else true )

    val movingFromPosition_sortingIndex: Long = {
      if (objectAtThatIndexIdIn.isDefined) {
        getSortingIndex(dbIn, containingObjectIdIn, objectAtThatIndexFormIdIn.get, objectAtThatIndexIdIn.get)
      } else {
        // could happen if it's the first entry (first attribute) in an entity, or if the caller (due to whatever reason including possibly a bug) did not
        // know what objectAtThatIndexIdIn value to use, so passed None: attempting to be resilient to that here.
        Database.minIdValue + 990
      }
    }

    val (byHowManyEntriesActuallyMoving: Int, nearNewNeighborSortingIndex: Option[Long], farNewNeighborSortingIndex: Option[Long]) =
      findNewNeighbors(dbIn, containingObjectIdIn, numRowsToMoveIfThereAreThatManyIn, forwardNotBackIn, movingFromPosition_sortingIndex)

    var displayStartingRowNumber = startingDisplayRowIndexIn

    if (nearNewNeighborSortingIndex.isEmpty) {
      ui.displayText("Nowhere to move it to, so doing nothing.")
    } else {
      val (newSortingIndex: Long, trouble: Boolean) = {
        var (newSortingIndex: Long, trouble: Boolean, newStartingRowNum: Int) = {
          getNewSortingIndex(dbIn, containingObjectIdIn, groupSizeOrNumAttributes_ToCalcNewDisplayStartingIndex_In, startingDisplayRowIndexIn,
                             nearNewNeighborSortingIndex, farNewNeighborSortingIndex, forwardNotBackIn,
                             byHowManyEntriesActuallyMoving, movingFromPosition_sortingIndex, moveFromIndexInObjListIn, numDisplayLinesIn)
        }
        displayStartingRowNumber = newStartingRowNum
        if (trouble) {
          renumberSortingIndexes(dbIn, containingObjectIdIn)

          // Get the sortingIndex of the entry right before the one being placed, increment (since just renumbered; or not?), then use that as the "old
          // position" moving from.  (Getting a new value because the old movingFromPosition_sortingIndex value is now invalid, since we just renumbered above.)
          val movingFromPosition_sortingIndex2: Long = {
            if (objectAtThatIndexIdIn.isDefined) {
              getSortingIndex(dbIn, containingObjectIdIn, objectAtThatIndexFormIdIn.get, objectAtThatIndexIdIn.get)
            } else {
              // (reason for next line is in related comments above at "val movingFromPosition_sortingIndex: Long =".)
              Database.minIdValue + 990
            }
          }
          val (byHowManyEntriesMoving2: Int, nearNewNeighborSortingIndex2: Option[Long], farNewNeighborSortingIndex2: Option[Long]) =
            findNewNeighbors(dbIn, containingObjectIdIn, numRowsToMoveIfThereAreThatManyIn, forwardNotBackIn, movingFromPosition_sortingIndex2)
          // (for some reason, can't reassign the results directly to the vars like this "(newSortingIndex, trouble, newStartingRowNum) = ..."?
          val (a: Long, b: Boolean, c: Int) = getNewSortingIndex(dbIn, containingObjectIdIn, groupSizeOrNumAttributes_ToCalcNewDisplayStartingIndex_In,
                                                                 startingDisplayRowIndexIn, nearNewNeighborSortingIndex2,
                                                                 farNewNeighborSortingIndex2, forwardNotBackIn,
                                                                 byHowManyEntriesMoving2, movingFromPosition_sortingIndex2, moveFromIndexInObjListIn,
                                                                 numDisplayLinesIn)
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
        updateSortedEntry(dbIn, containingObjectIdIn, movingObjsAttributeFormIdIn, movingObjIdIn, newSortingIndex)
      }
    }
    displayStartingRowNumber
  }

  protected def getSortingIndex(dbIn: Database, containingObjectIdIn: Long, objectAtThatIndexFormIdIn: Int, objectAtThatIndexIdIn: Long): Long

  /** The dbIn should represent the *same* database as where containingObjectIdIn is stored!  (Idea: enforce that by passing in a containingObject instead
    * of a containingObjectIdIn (ie an Entity or Group, using scala's type system), or a boolean saying which it is, then get the db from it instead of
    * passing it as a parm?  Same in location(s) w/ similar comment about containingObjectIdIn.)
    */
  protected def getNewSortingIndex(dbIn: Database, containingObjectIdIn: Long, groupSizeOrNumAttributes_ToCalcNewDisplayStartingIndex_In: Long, startingDisplayRowIndexIn: Int,
                                   nearNewNeighborSortingIndex: Option[Long], farNewNeighborSortingIndex: Option[Long], forwardNotBack: Boolean,
                                   byHowManyEntriesMoving: Int, movingFromPosition_sortingIndex: Long, moveFromRelativeIndexInObjListIn: Int,
                                   numDisplayLines: Int): (Long, Boolean, Int) = {
    if (nearNewNeighborSortingIndex.isEmpty) {
      throw new OmException("never should have got here: should have been the logic of ~nowhere to go so doing nothing")
    }

    def ensureNonDuplicate(groupOrEntityIdIn: Long, newIndexIn: Long): Option[Long] = {
      // At this point we might have as newIndexIn, the dup of an archived entity's sorting index, since archived entities are ignored the in
      // logic that calculated our *NewNeighborSortingIndex variable
      // values.  If so, find another candidate (feels like a kludge and knowledge scattered across code, but not sure of a better algorithm right now).
      if (indexIsInUse(dbIn, groupOrEntityIdIn, newIndexIn)) {
        try {
            Some(findUnusedSortingIndex(dbIn, containingObjectIdIn, newIndexIn))
        } catch {
          case e: Exception =>
            if (e.getMessage == Database.UNUSED_GROUP_ERR1 || e.getMessage == Database.UNUSED_GROUP_ERR2) None
            else throw e
        }
      } else
        Some(newIndexIn)
    }


    val (newIndex: Long, trouble: Boolean) = {
      if (farNewNeighborSortingIndex.isEmpty) {
        //halfway between min value of a long (or max, depending on direction of the move), and whatever highlightIndexIn's long (sorting_index) is now
        if (forwardNotBack) {
          // do calculation as float or it wraps & gets wrong result, with inputs like this (idea: unit tests....)
          //     scala> -3074457345618258604L + ((9223372036854775807L - -3074457345618258604L) / 2)
          //     res2: Long = -6148914691236517206
          val newIndex = (nearNewNeighborSortingIndex.get + ((Database.maxIdValue.asInstanceOf[Float] - nearNewNeighborSortingIndex.get) / 2)).asInstanceOf[Long]
          val nonDuplicatedNewIndex: Option[Long] = ensureNonDuplicate(containingObjectIdIn, newIndex)
          // leaving it to communicate intent, but won't be '>' because a Long would just wrap, so...
          val trouble: Boolean = nonDuplicatedNewIndex.isEmpty || nonDuplicatedNewIndex.get > Database.maxIdValue ||
                                 nonDuplicatedNewIndex.get <= movingFromPosition_sortingIndex || nonDuplicatedNewIndex.get <= nearNewNeighborSortingIndex.get
          (nonDuplicatedNewIndex.getOrElse(0L), trouble)
        } else {
          // Leaving it to communicate intent, but won't be '<' because a Long would just wrap, so...
          val newIndex = nearNewNeighborSortingIndex.get - math.abs((math.abs(Database.minIdValue) - math.abs(nearNewNeighborSortingIndex.get)) / 2)
          val nonDuplicatedNewIndex: Option[Long] = ensureNonDuplicate(containingObjectIdIn, newIndex)
          val trouble: Boolean = nonDuplicatedNewIndex.isEmpty || nonDuplicatedNewIndex.get < Database.minIdValue ||
                                 nonDuplicatedNewIndex.get >= movingFromPosition_sortingIndex ||
                                 nonDuplicatedNewIndex.get >= nearNewNeighborSortingIndex.get
          (nonDuplicatedNewIndex.getOrElse(0L), trouble)
        }
      } else {
        val halfDistance: Long = math.abs(farNewNeighborSortingIndex.get - nearNewNeighborSortingIndex.get) / 2
        val newIndex: Long = {
                               // a Float so it won't wrap around:
                               if (forwardNotBack) nearNewNeighborSortingIndex.get.asInstanceOf[Float] + halfDistance
                               else nearNewNeighborSortingIndex.get - halfDistance
                             }.asInstanceOf[Long]
        val nonDuplicatedNewIndex = ensureNonDuplicate(containingObjectIdIn, newIndex)
        // leaving this comment to communicate intent, but won't be '<' or '>' because a Long would just wrap, so...
        val trouble: Boolean =
          if (forwardNotBack) {
            nonDuplicatedNewIndex.isEmpty || nonDuplicatedNewIndex.get <= movingFromPosition_sortingIndex ||
            nonDuplicatedNewIndex.get >= farNewNeighborSortingIndex.get || nonDuplicatedNewIndex.get <= nearNewNeighborSortingIndex.get
          } else {
            nonDuplicatedNewIndex.isEmpty || nonDuplicatedNewIndex.get >= movingFromPosition_sortingIndex ||
            nonDuplicatedNewIndex.get <= farNewNeighborSortingIndex.get || nonDuplicatedNewIndex.get >= nearNewNeighborSortingIndex.get
          }
        (nonDuplicatedNewIndex.getOrElse(0L), trouble)
      }
    }

    val newDisplayRowsStartingWithCounter: Int = {
      if (forwardNotBack) {
        if ((moveFromRelativeIndexInObjListIn + byHowManyEntriesMoving) >= numDisplayLines) {
          // if the object will move too far to be seen in this screenful, adjust the screenful to redisplay, with some margin
          // ("- 1" on next line because the indexes are zero-based)
          val lastScreenfulStartingIndex: Long = groupSizeOrNumAttributes_ToCalcNewDisplayStartingIndex_In - numDisplayLines - 1
          //(was: "(numDisplayLines / 4)", but center it better in the screen):
          // Another name for next var might be  like "display index at new entry but going back to show enough contextual data on screen".
          val numLinesInHalfTheScreen = numDisplayLines / 2
          val movedEntrysNewAbsoluteIndexMinusHalfScreenful: Double = startingDisplayRowIndexIn + moveFromRelativeIndexInObjListIn +
                                                                      byHowManyEntriesMoving - numLinesInHalfTheScreen
          val min: Int = math.min(lastScreenfulStartingIndex, movedEntrysNewAbsoluteIndexMinusHalfScreenful).asInstanceOf[Int]
          math.max(0, min)
        } else startingDisplayRowIndexIn
      } else {
        if ((moveFromRelativeIndexInObjListIn - byHowManyEntriesMoving) < 0) {
          val movedEntrysNewAbsoluteIndexMinusHalfScreenful: Int = startingDisplayRowIndexIn + moveFromRelativeIndexInObjListIn -
                                                                   byHowManyEntriesMoving - (numDisplayLines / 2)
          math.max(0, movedEntrysNewAbsoluteIndexMinusHalfScreenful)
        } else startingDisplayRowIndexIn
      }
    }

    (newIndex, trouble, newDisplayRowsStartingWithCounter)
  }

  /** The dbIn should represent the *same* database as where groupOrEntityIdIn is stored!  (See details in comments at similar location
    * about containingObjectIdIn.)
    */
  protected def findNewNeighbors(dbIn: Database, groupOrEntityIdIn: Long, movingDistanceIn: Int, forwardNotBackIn: Boolean,
                                 movingFromPosition_sortingIndex: Long): (Int, Option[Long], Option[Long]) = {

    // (idea: this could probably be made more efficient by combining the 2nd part of the (fixed) algorithm (the call to mDB.getNearestEntry)
    // with the first part.  I.e., maybe we don't need to calculate the farNewNeighborSortingIndex at first, since we're just going to soon replace
    // it with the "next one after the nearNewNeighbor" anyway.  But first it should have some good tests around it: coverage.)

    // get enough data to represent the new location in the sort order: movingDistanceIn entries away, and one beyond, and place this entity between them:
    val queryLimit = movingDistanceIn + 1

    val results: Array[Array[Option[Any]]] = getAdjacentEntriesSortingIndexes(dbIn, groupOrEntityIdIn, movingFromPosition_sortingIndex, Some(queryLimit),
                                                                     forwardNotBackIn = forwardNotBackIn).toArray
    require(results.length <= queryLimit)
    // (get the last result's sortingIndex, if possible; 0-based of course; i.e., that of the first entry beyond where we're moving to):
    val farNewNeighborSortingIndex: Option[Long] =
      if (results.length > 0 && results.length == queryLimit) results(results.length - 1)(0).asInstanceOf[Option[Long]]
      else None
    val (nearNewNeighborSortingIndex: Option[Long], byHowManyEntriesMoving: Int) = {
      if (results.length == 0) {
        // It could be a new entry trying to be moved to the a first or last position, or a mistake with the current entity. Both seem OK if we
        // just say we need to move from a slightly incremented/decremented position.  Maybe the increment/decrement isn't even needed, but harmless & cheap.
        val newNearIndex = {
          if (forwardNotBackIn) movingFromPosition_sortingIndex + 1
          else movingFromPosition_sortingIndex - 1
        }
        (Some(newNearIndex), 1)
      } else if (results.length == queryLimit) {
        if (queryLimit == 1) (Some(movingFromPosition_sortingIndex), 1)
        else {
          // get the next-to-last result's sortingIndex
          (results(queryLimit - 2)(0).asInstanceOf[Option[Long]], results.length - 1)
        }
      } else {
        // given the 'require' statement above, results.size now has to be between 0 and queryLimit, so use the last result as the "near new neighbor", and
        // move just beyond that
        (results(results.length - 1)(0).asInstanceOf[Option[Long]], results.length)
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
      if (nearNewNeighborSortingIndex.isEmpty || farNewNeighborSortingIndex.isEmpty)
        None
      else
        getNearestEntrysSortingIndex(dbIn, groupOrEntityIdIn, nearNewNeighborSortingIndex.get, forwardNotBackIn = forwardNotBackIn)
    }

    (byHowManyEntriesMoving, nearNewNeighborSortingIndex, adjustedFarNewNeighborSortingIndex)
  }

  protected def getAdjacentEntriesSortingIndexes(dbIn: Database, groupOrEntityIdIn: Long, movingFromPosition_sortingIndexIn: Long, queryLimitIn: Option[Long],
                                                 forwardNotBackIn: Boolean): List[Array[Option[Any]]]

  protected def getNearestEntrysSortingIndex(dbIn: Database, containingIdIn: Long, startingPointSortingIndexIn: Long, forwardNotBackIn: Boolean): Option[Long]

  protected def renumberSortingIndexes(dbIn: Database, containingObjectIdIn: Long)

  protected def updateSortedEntry(dbIn: Database, containingObjectIdIn: Long, movingObjsAttributeFormIdIn: Int, movingObjIdIn: Long, sortingIndexIn: Long)

  protected def indexIsInUse(dbIn: Database, groupOrEntityIdIn: Long, sortingIndexIn: Long): Boolean

  protected def findUnusedSortingIndex(dbIn: Database, groupOrEntityIdIn: Long, startingWithIn: Long): Long

}
