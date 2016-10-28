/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003-2004 and 2008-2016 inclusive, Luke A. Call; all rights reserved.
    (That copyright statement was previously 2013-2015, until I remembered that much of Controller came from TextUI.scala, and TextUI.java before that.
    And this file initially came from Controller.scala.)
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule, guidelines around binary
    distribution, and the GNU Affero General Public License as published by the Free Software Foundation, either version 3
    of the License, or (at your option) any later version.  See the file LICENSE for details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
package org.onemodel.core

import org.onemodel.core.database.PostgreSQLDatabase
import org.onemodel.core.model.{Attribute, Entity}

/** This is just a place to put shared code ("Utility") until a grouping or better idea emerges.  Using it also
  * has the benefit of making that file smaller, so it is more quickly processed by code tools (especially the IDE).
 */
object Util {
  // should these be more consistently upper-case? What is the scala style for constants?  similarly in other classes.
  def maxNameLength: Int = math.max(math.max(PostgreSQLDatabase.entityNameLength, PostgreSQLDatabase.relationTypeNameLength),
                                    PostgreSQLDatabase.classNameLength)

  // Might not be the most familiar date form for us Americans, but it seems the most useful in the widest
  // variety of situations, and more readable than with the "T" embedded in place of
  // the 1st space.  So, this approximates iso-8601.
  // these are for input.
  val DATEFORMAT = new java.text.SimpleDateFormat("yyyy-MM-dd HH:mm:ss:SSS zzz")
  val DATEFORMAT2 = new java.text.SimpleDateFormat("yyyy-MM-dd HH:mm:ss zzz")
  val DATEFORMAT3 = new java.text.SimpleDateFormat("yyyy-MM-dd HH:mm zzz")
  val DATEFORMAT_WITH_ERA = new java.text.SimpleDateFormat("GGyyyy-MM-dd HH:mm:ss:SSS zzz")
  val DATEFORMAT2_WITH_ERA = new java.text.SimpleDateFormat("GGyyyy-MM-dd HH:mm:ss zzz")
  val DATEFORMAT3_WITH_ERA = new java.text.SimpleDateFormat("GGyyyy-MM-dd HH:mm zzz")

  //these are here to avoid colliding with use of the same names within other code inside the class.
  // idea: see what scala does with enums and/or constants; update this style?
  val ENTITY_TYPE: String = "Entity"
  val QUANTITY_TYPE: String = "QuantityAttribute"
  val TEXT_TYPE: String = "TextAttribute"
  val DATE_TYPE: String = "DateAttribute"
  val BOOLEAN_TYPE: String = "BooleanAttribute"
  val FILE_TYPE: String = "FileAttribute"
  //i.e., "relationTypeType", or the thing that we sometimes put in an attribute type parameter, though not exactly an attribute type, which is "RelationType":
  val RELATION_TYPE_TYPE: String = "RelationType"
  val RELATION_TO_ENTITY_TYPE: String = "RelationToEntity"
  val RELATION_TO_GROUP_TYPE: String = "RelationToGroup"
  val GROUP_TYPE: String = "Group"
  val ENTITY_CLASS_TYPE: String = "Class"
  val OM_INSTANCE_TYPE: String = "Instance"

  val ORPHANED_GROUP_MESSAGE: String = "There is no entity with a containing relation to the group (orphaned).  You might search for it" +
                                       " (by adding it as an attribute to some entity)," +
                                       " & see if it should be deleted, kept with an entity, or left out there floating." +
                                       "  (While this is not an expected usage, it is allowed and does not imply data corruption.)"

  val unselectMoveTargetPromptText: String = "Unselect current move target (if present; not necessary really)"

  // This says 'same screenful' because it's easier to assume that the returned index refers to the currently available
  // local collections (a subset of all possible entries, for display), than calling chooseOrCreateObject, and sounds as useful:
  val unselectMoveTargetLeadingText: String = "CHOOSE AN ENTRY (that contains only one subgroup) FOR THE TARGET OF MOVES (choose from SAME SCREENFUL as " +
                                              "now;  if the target contains 0 subgroups, or 2 or more subgroups, " +
                                              "use other means to move entities to it until some kind of \"move anywhere\" feature is added):"

  val defaultPreferencesDepth = 10
  // Don't change these: they get set and looked up in the data for preferences. Changing it would just require users to reset it though, and would
  // leave the old as clutter in the data.
  val USER_PREFERENCES = "User preferences"
  final val SHOW_PUBLIC_PRIVATE_STATUS_PREFERENCE = "Should entity lists show public/private status for each?"
  final val DEFAULT_ENTITY_PREFERENCE = "Which entity should be displayed as default, when starting the program?"

  val HEADER_CONTENT_TAG = "htmlHeaderContent"
  val BODY_CONTENT_TAG = "htmlInitialBodyContent"
  val FOOTER_CONTENT_TAG = "htmlFooterContent"

  def getClipboardContent: String = {
    val clipboard: java.awt.datatransfer.Clipboard = java.awt.Toolkit.getDefaultToolkit.getSystemClipboard
    val contents: String = clipboard.getContents(null).getTransferData(java.awt.datatransfer.DataFlavor.stringFlavor).toString
    contents.trim
    //(example of placing data on the clipboard, for future reference:)
    //val selection = new java.awt.datatransfer.StringSelection("someString")
    //clipboard.setContents(selection, null)
  }

  def isWindows: Boolean = {
    val osName = System.getProperty("os.name").toLowerCase
    osName.contains("win")
  }

  // Used for example after one has been deleted, to put the highlight on right next one:
  // idea: This feels overcomplicated.  Make it better?  Fixing bad smells in general (large classes etc etc) is on the task list.
  /**
   * @param objectSetSize # of all the possible entries, not reduced by what fits in the available display space (I think).
   * @param objectsToDisplayIn  Only those that have been chosen to display (ie, smaller list to fit in display size size) (I think).
   * @return
   */
  def findEntityToHighlightNext(objectSetSize: Int, objectsToDisplayIn: java.util.ArrayList[Entity], removedOneIn: Boolean,
                                previouslyHighlightedIndexInObjListIn: Int, previouslyHighlightedEntryIn: Entity): Option[Entity] = {
    //NOTE: SIMILAR TO findAttributeToHighlightNext: WHEN MAINTAINING ONE, DO SIMILARLY ON THE OTHER, until they are merged maybe by using the scala type
    //system better.

    // here of course, previouslyHighlightedIndexInObjListIn and objIds.size were calculated prior to the deletion.
    if (removedOneIn) {
      val newObjListSize = objectSetSize - 1
      val newIndexToHighlight = math.min(newObjListSize - 1, previouslyHighlightedIndexInObjListIn)
      if (newIndexToHighlight >= 0) {
        if (newIndexToHighlight != previouslyHighlightedIndexInObjListIn) Some(objectsToDisplayIn.get(newIndexToHighlight))
        else {
          if (newIndexToHighlight + 1 < newObjListSize - 1) Some(objectsToDisplayIn.get(newIndexToHighlight + 1))
          else if (newIndexToHighlight - 1 >= 0) Some(objectsToDisplayIn.get(newIndexToHighlight - 1))
          else None
        }
      } else None
    } else Some(previouslyHighlightedEntryIn)
  }

  /** SEE COMMENTS FOR findEntityToHighlightNext. */
  def findAttributeToHighlightNext(objectSetSize: Int, objectsToDisplayIn: java.util.ArrayList[Attribute], removedOne: Boolean,
                                   previouslyHighlightedIndexInObjListIn: Int, previouslyHighlightedEntryIn: Attribute): Option[Attribute] = {
    //NOTE: SIMILAR TO findEntityToHighlightNext: WHEN MAINTAINING ONE, DO SIMILARLY ON THE OTHER, until they are merged maybe by using the scala type
    //system better.
    if (removedOne) {
      val newObjListSize = objectSetSize - 1
      val newIndexToHighlight = math.min(newObjListSize - 1, previouslyHighlightedIndexInObjListIn)
      if (newIndexToHighlight >= 0) {
        if (newIndexToHighlight != previouslyHighlightedIndexInObjListIn) {
          Some(objectsToDisplayIn.get(newIndexToHighlight))
        } else {
          if (newIndexToHighlight + 1 < newObjListSize - 1) Some(objectsToDisplayIn.get(newIndexToHighlight + 1))
          else if (newIndexToHighlight - 1 >= 0) Some(objectsToDisplayIn.get(newIndexToHighlight - 1))
          else None
        }
      } else None
    } else Some(previouslyHighlightedEntryIn)
  }

  def getDefaultUserInfo: (String, String) = {
    (System.getProperty("user.name"), "x")
  }

}
