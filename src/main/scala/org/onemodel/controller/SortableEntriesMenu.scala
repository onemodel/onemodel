/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2014-2015 inclusive, Luke A. Call; all rights reserved.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule, guidelines around binary
    distribution, and the GNU Affero General Public License as published by the Free Software Foundation, either version 3
    of the License, or (at your option) any later version.  See the file LICENSE for details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
package org.onemodel.controller

import org.onemodel.database.PostgreSQLDatabase
import org.onemodel.{TextUI, OmException}

abstract class SortableEntriesMenu(val ui: TextUI, val db: PostgreSQLDatabase) {

  /** Returns the starting row number (in case the view window was adjusted to show other entries around the moved entity).
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
  protected def placeEntryInPosition(containingObjectIdIn: Long, groupSizeOrNumAttributesIn: Long, numRowsToMoveIfThereAreThatManyIn: Int, forwardNotBackIn: Boolean,
                           startingDisplayRowIndexIn: Int, movingObjIdIn: Long, moveFromIndexInObjListIn: Int, objectAtThatIndexIdIn: Long,
                           numDisplayLinesIn: Int, movingObjsAttributeFormIdIn: Int, objectAtThatIndexFormIdIn: Int): Int = {
    //val movingFromPosition_sortingIndex = mDB.getEntityInAGroupData(groupIn.getId, objectIdAtThatIndex)(0).get.asInstanceOf[Long]
    val movingFromPosition_sortingIndex = getSortingIndex(containingObjectIdIn, objectAtThatIndexFormIdIn, objectAtThatIndexIdIn)

    val (byHowManyEntriesActuallyMoving: Int, nearNewNeighborSortingIndex: Option[Long], farNewNeighborSortingIndex: Option[Long]) =
      findNewNeighbors(containingObjectIdIn, numRowsToMoveIfThereAreThatManyIn, forwardNotBackIn, movingFromPosition_sortingIndex)

    var displayStartingRowNumber = startingDisplayRowIndexIn

    if (nearNewNeighborSortingIndex.isEmpty) {
      ui.displayText("Nowhere to move it to, so doing nothing.")
    } else {
      val (newSortingIndex: Long, trouble: Boolean) = {
        var (newSortingIndex: Long, trouble: Boolean, newStartingRowNum: Int) = {
          getNewSortingIndex(groupSizeOrNumAttributesIn, startingDisplayRowIndexIn, nearNewNeighborSortingIndex, farNewNeighborSortingIndex, forwardNotBackIn,
                             byHowManyEntriesActuallyMoving, movingFromPosition_sortingIndex, moveFromIndexInObjListIn, numDisplayLinesIn)
        }
        displayStartingRowNumber = newStartingRowNum
        if (trouble) {
          renumberSortingIndexes(containingObjectIdIn)

          // Get the sortingIndex of the one it was right after, increment (since just renumbered; or not?), then use that as the "old position" moving from.
          // (This is because the old movingFromPosition_sortingIndex value is now invalid, since we just renumbered above.)
          val movingFromPosition_sortingIndex2: Long = getSortingIndex(containingObjectIdIn, objectAtThatIndexFormIdIn, objectAtThatIndexIdIn)
          val (byHowManyEntriesMoving2: Int, nearNewNeighborSortingIndex2: Option[Long], farNewNeighborSortingIndex2: Option[Long]) =
            findNewNeighbors(containingObjectIdIn, numRowsToMoveIfThereAreThatManyIn, forwardNotBackIn, movingFromPosition_sortingIndex2)
          // (for some reason, can't reassign the results directly to the vars like this "(newSortingIndex, trouble, newStartingRowNum) = ..."?
          val (a: Long, b: Boolean, c: Int) = getNewSortingIndex(groupSizeOrNumAttributesIn, startingDisplayRowIndexIn, nearNewNeighborSortingIndex2,
                                             farNewNeighborSortingIndex2, forwardNotBackIn,
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
        updateSortedEntry(containingObjectIdIn, movingObjsAttributeFormIdIn, movingObjIdIn, newSortingIndex)
      }
    }
    displayStartingRowNumber
  }

  protected def getSortingIndex(containingObjectIdIn: Long, objectAtThatIndexFormIdIn: Int, objectAtThatIndexIdIn: Long): Long

  protected def getNewSortingIndex(groupSizeOrNumAttributesIn: Long, startingDisplayRowIndexIn: Int, nearNewNeighborSortingIndex: Option[Long],
                         farNewNeighborSortingIndex: Option[Long], forwardNotBack: Boolean,
                         byHowManyEntriesMoving: Int, movingFromPosition_sortingIndex: Long, moveFromIndexInObjListIn: Int,
                         numDisplayLines: Int): (Long, Boolean, Int) = {
    if (nearNewNeighborSortingIndex.isEmpty) {
      throw new OmException("never should have got here: should have been the logic of ~nowhere to go so doing nothing")
    }

    val (newIndex: Long, trouble: Boolean) = {
      if (farNewNeighborSortingIndex.isEmpty) {
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

    val newDisplayRowsStartingWithCounter: Int = {
      if (forwardNotBack) {
        if ((moveFromIndexInObjListIn + byHowManyEntriesMoving) > numDisplayLines) {
          // if the object will move too far to be seen in this screenful, adjust the screenful to redisplay, with some margin
          val x: Long = groupSizeOrNumAttributesIn - numDisplayLines
          //(was: "(numDisplayLines / 4)", but center it better in the screen):
          val y: Int = startingDisplayRowIndexIn + numDisplayLines + byHowManyEntriesMoving - (numDisplayLines / 2)
          val min: Int = math.min(x,y).asInstanceOf[Int]
          min
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

  protected def findNewNeighbors(groupOrEntityIdIn: Long, movingDistanceIn: Int, forwardNotBackIn: Boolean,
                                 movingFromPosition_sortingIndex: Long): (Int, Option[Long], Option[Long]) = {

    // (idea: this could probably be made more efficient by combining the 2nd part of the (fixed) algorithm (the call to mDB.getNearestEntry)
    // with the first part.  I.e., maybe we don't need to calculate the farNewNeighborSortingIndex at first, since we're just going to soon replace
    // it with the "next one after the nearNewNeighbor" anyway.  But first it should have some good tests around it: coverage.)

    // get enough data to represent the new location in the sort order: movingDistanceIn entries away, and one beyond, and place this entity between them:
    val queryLimit = movingDistanceIn + 1

    val results: Array[Array[Option[Any]]] = getAdjacentEntriesSortingIndexes(groupOrEntityIdIn, movingFromPosition_sortingIndex, Some(queryLimit),
                                                                     forwardNotBackIn = forwardNotBackIn).toArray
    require(results.length <= queryLimit)
    // (get the last result's sortingIndex, if possible; 0-based of course; i.e., that of the first entry beyond where we're moving to):
    val farNewNeighborSortingIndex: Option[Long] =
      if (results.length > 0 && results.length == queryLimit) results(results.length - 1)(0).asInstanceOf[Option[Long]]
      else None
    val (nearNewNeighborSortingIndex: Option[Long], byHowManyEntriesMoving: Int) = {
      if (results.length == 0) {
        // there's nowhere to move to, so just get out of here (shortly, as noted in the caller)
        (None, 0)
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
        getNearestEntrysSortingIndex(groupOrEntityIdIn, nearNewNeighborSortingIndex.get, forwardNotBackIn = forwardNotBackIn)
    }

    (byHowManyEntriesMoving, nearNewNeighborSortingIndex, adjustedFarNewNeighborSortingIndex)
  }

  protected def getAdjacentEntriesSortingIndexes(groupOrEntityIdIn: Long, movingFromPosition_sortingIndexIn: Long, queryLimitIn: Option[Long],
                                                 forwardNotBackIn: Boolean): List[Array[Option[Any]]]

  protected def getNearestEntrysSortingIndex(containingIdIn: Long, startingPointSortingIndexIn: Long, forwardNotBackIn: Boolean): Option[Long]

  protected def renumberSortingIndexes(containingObjectIdIn: Long)

  protected def updateSortedEntry(containingObjectIdIn: Long, movingObjsAttributeFormIdIn: Int, movingObjIdIn: Long, sortingIndexIn: Long)

}
