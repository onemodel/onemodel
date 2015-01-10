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

import java.util

import org.onemodel._
import org.onemodel.model._
import org.onemodel.database.PostgreSQLDatabase

class GroupMenu(override val ui: TextUI, dbInOVERRIDESmDBWhichHasANewDbConnectionTHATWEDONTWANT: PostgreSQLDatabase) extends Controller(ui) {
  override val mDB = dbInOVERRIDESmDBWhichHasANewDbConnectionTHATWEDONTWANT


  /** Returns None if user wants out. The parameter callingMenusRtgIn exists only to preserve the value as may be used by quickGroupMenu, and passed
    * between it and here.
    */
  // see comment on helper method about tailrec
  //@tailrec
  // idea: There's some better scala idiom for this control logic around recursion and exception handling (& there's similar code in all "*Menu" classes):
  final def groupMenu(displayStartingRowNumberIn: Long, relationToGroupIn: RelationToGroup,
                      callingMenusRtgIn: Option[RelationToGroup] = None): Option[Entity] = {
    try {
      groupMenu_helper(displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn)
    } catch {
      case e: Exception =>
        showException(e)
        val ans = ui.askYesNoQuestion("Go back to what you were doing (vs. going out)?",Some("y"))
        if (ans != None && ans.get) groupMenu(displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn)
        else None
    }
  }

  // put @tailrec back when tail recursion works better on the JVM & don't get that err "...not in tail position" (unless we want to have all the calls
  // preserved, so that each previously seen individual menu is displayed when ESCaping back out of the stack of calls?).
  //@tailrec
  //
  def groupMenu_helper(displayStartingRowNumberIn: Long, relationToGroupIn: RelationToGroup, callingMenusRtgIn: Option[RelationToGroup] = None):
  Option[Entity] = {
    val group = new Group(mDB, relationToGroupIn.getGroupId)
    require(relationToGroupIn != null)

    val definingEntity = group.getClassDefiningEntity
    val choices = Array[String]("Add entity to group (if you add an existing entity with a relationship to one group, that is effectively adding that group " +
                                "as a subgroup to this one)",

                                "Import/Export...",
                                "Edit group name",
                                "Delete...",
                                "Go to...",
                                listNextItemsPrompt,
                                "Filter (limit which are shown; unimplemented)",
                                "(stub)" /*sort?*/ ,
                                "Quick group menu")
    val attrType = Some(new RelationType(mDB, relationToGroupIn.getAttrTypeId))
    val leadingText: Array[String] = Array("ENTITY GROUP (regular menu: more complete, so slower for some things): " +
                                           relationToGroupIn.getDisplayString(0, None, attrType))
    val numDisplayableItems = ui.maxColumnarChoicesToDisplayAfter(leadingText.length, choices.size, maxNameLength)
    val objectsToDisplay: java.util.ArrayList[Entity] = group.getGroupEntries(displayStartingRowNumberIn, Some(numDisplayableItems))
    addRemainingCountToPrompt(choices, objectsToDisplay.size, group.groupSize, displayStartingRowNumberIn)
    val names: Array[String] = for (entity: Entity <- objectsToDisplay.toArray(Array[Entity]())) yield {
      val numSubgroupsPrefix: String = getNumSubgroupsPrefix(entity.getId)
      numSubgroupsPrefix + entity.getName + " " + entity.getPublicStatusString()
    }
    val numEntitiesInGroup: Long = group.groupSize


    val response = ui.askWhich(Some(leadingText), choices, names)
    if (response == None) None
    else {
      val answer = response.get
      if (answer == 1) {
        addEntityToGroup(group)
        groupMenu(displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn = callingMenusRtgIn)
      } else if (answer == 2) {
        val importOrExport = ui.askWhich(None, Array("Import", "Export"), Array[String]())
        if (importOrExport != None) {
          if (importOrExport.get == 1) new ImportExport(ui,mDB).importCollapsibleOutlineAsGroups(relationToGroupIn)
          else if (importOrExport.get == 2) {
            ui.displayText("not yet implemented: try it from an entity rather than a group where it is supported, for now.")
            //exportToCollapsibleOutline(entityIn)
          }
        }
        groupMenu(displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn = callingMenusRtgIn)
      } else if (answer == 3) {
        val ans = editGroupName(group)
        if (ans == None) {
          groupMenu(displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn = callingMenusRtgIn)
        } else {
          // reread the RTG to get the updated info:
          groupMenu(displayStartingRowNumberIn, new RelationToGroup(mDB, relationToGroupIn.getParentId, relationToGroupIn.getAttrTypeId,
                                                                    relationToGroupIn.getGroupId), callingMenusRtgIn = callingMenusRtgIn)
        }
      } else if (answer == 4) {
        val response = ui.askWhich(None, Array("Delete group definition & remove from all relationships where it is found",
                                               "Delete group definition & remove from all relationships where it is found, AND delete all entities in it?"),
                                   Array[String]())
        if (response == None) groupMenu(displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn = callingMenusRtgIn)
        else {
          val answer = response.get
          if (answer == 1) {
            val ans = ui.askYesNoQuestion("DELETE this group definition AND remove from all entities that link to it (but not entities it contains): **ARE " +
                                          "YOU REALLY SURE?**")
            if (ans != None && ans.get) {
              val desc: String = relationToGroupIn.getDisplayString(0, None, attrType)
              relationToGroupIn.deleteGroupAndRelationsToIt()
              ui.displayText("Deleted group definition: \"" + desc + "\"" + ".")
              None
            } else {
              ui.displayText("Did not delete group definition.", waitForKeystroke = false)
              groupMenu(displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn = callingMenusRtgIn)
            }
          } else if (answer == 2) {
            // if calculating the total to be deleted for this prompt or anything else recursive, we have to deal with looping data & not duplicate it in
            // counting.
            // IDEA:  ******ALSO WHEN UPDATING THIS TO BE RECURSIVE, OR CONSIDERING SUCH, CONSIDER ALSO HOW TO ADDRESS ARCHIVED ENTITIES: SUCH AS IF ALL QUERIES
            // USED IN THIS WILL ALSO CK FOR ARCHIVED ENTITIES, AND ANYTHING ELSE?  And show the # of archived entities to the user or suggest that they view
            // those
            // also be4 deleting everything?
            val ans = ui.askYesNoQuestion("DELETE this group definition from *all* relationships where it is found, *AND* its entities, " +
                                          "with *ALL* entities and their \"subgroups\" that they eventually " +
                                          "refer" +
                                          " to, recursively (actually, the recursion is not finished and will probably fail if you have nesting): *******ARE " +
                                          "YOU " +
                                          "REALLY SURE?******")
            if (ans != None && ans.get) {
              val ans = ui.askYesNoQuestion("Um, this seems unusual.  Really _really_ sure?  I certainly hope you make regular backups of the data AND TEST" +
                                            " RESTORES.  (Note: the deletion does(n't yet do) recursion but doesn't yet properly handle groups that " +
                                            "loop--that eventually contain themselves.)  Proceed to delete it all?:")
              if (ans != None && ans.get) {
                val name: String = relationToGroupIn.getDisplayString(0, None, attrType)
                group.deleteWithEntities()
                ui.displayText("Deleted relation to group\"" + name + "\", along with the " + numEntitiesInGroup + " entities: " + ".")
                None
              } else None
            } else {
              ui.displayText("Did not delete group.", waitForKeystroke = false)
              groupMenu(displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn = callingMenusRtgIn)
            }
          } else {
            ui.displayText("invalid response")
            groupMenu(displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn = callingMenusRtgIn)
          }
        }
      } else if (answer == 5 && answer <= choices.size) {
        val containingEntities = mDB.getContainingEntities2(relationToGroupIn, 0)
        val numContainingEntities = containingEntities.size
        // (idea: make this next call efficient: now it builds them all when we just want a count; but is infrequent & likely small numbers)
        val choices = Array("Go edit the relation to group that led us here :" + relationToGroupIn.getDisplayString(15, None, attrType),
                            if (numContainingEntities == 1) "Go to entity containing this group: " + containingEntities.get(0)._2.getName
                                                            else "See entities that contain this group ( " + numContainingEntities + ")",
                            if (definingEntity != None) "Go to class-defining entity" else "(stub: no class-defining entity to go to)")
        //idea: consider: do we want this?:
        //(see similar comment in postgresqldatabase)
        //"See groups containing this group (" + numContainingGroups + ")")
        //val numContainingGroups = mDB.getContainingRelationToGroups(relationToGroupIn, 0).size

        val response = ui.askWhich(None, choices, Array[String]())
        if (response == None) None
        else {
          val answer = response.get
          if (answer == 1) {
            def updateRelationToGroup(dhInOut: RelationToGroupDataHolder) {
              //idea: does this make sense, to only update the dates when we prompt for everything on initial add? change(or note2later) update everything?
              relationToGroupIn.update(dhInOut.validOnDate, Some(dhInOut.observationDate))
            }
            val relationToGroupDH: RelationToGroupDataHolder = new RelationToGroupDataHolder(relationToGroupIn.getParentId, relationToGroupIn.getAttrTypeId,
                                                                                             relationToGroupIn.getGroupId, relationToGroupIn.getValidOnDate,
                                                                                             relationToGroupIn.getObservationDate)
            askForInfoAndUpdateAttribute[RelationToGroupDataHolder](relationToGroupDH, Controller.RELATION_TO_GROUP_TYPE,
                                                                    "CHOOSE TYPE OF [correct me: or edit existing?] Relation to Entity:",
                                                                    askForRelToGroupInfo, updateRelationToGroup)
            //force a reread from the DB so it shows the right info on the repeated menu:
            groupMenu(displayStartingRowNumberIn, new RelationToGroup(mDB, relationToGroupDH.entityId, relationToGroupDH.attrTypeId,
                                                                      relationToGroupDH.groupId), callingMenusRtgIn = callingMenusRtgIn)
          } else if (answer == 2 && answer <= choices.size) {
            val entity: Option[Entity] =
              if (numContainingEntities == 1) {
                Some(containingEntities.get(0)._2)
              } else {
                chooseAmongEntities(containingEntities)
              }

            if (entity != None)
              new EntityMenu(ui, mDB).entityMenu(0, entity.get)

            groupMenu(displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn = callingMenusRtgIn)
          } else if (answer == 3 && definingEntity != None && answer <= choices.size) {
            new EntityMenu(ui, mDB).entityMenu(0, definingEntity.get)
            groupMenu(displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn = callingMenusRtgIn)
          } else {
            ui.displayText("invalid response")
            groupMenu(displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn = callingMenusRtgIn)
          }
        }
      } else if (answer == 6) {
        val displayRowsStartingWithCounter: Long = {
          val currentPosition = displayStartingRowNumberIn + objectsToDisplay.size
          if (currentPosition >= numEntitiesInGroup) {
            ui.displayText("End of attribute list found; restarting from the beginning.")
            0 // start over
          } else currentPosition
        }
        groupMenu(displayRowsStartingWithCounter, relationToGroupIn, callingMenusRtgIn = callingMenusRtgIn)
      } else if (answer == 7) {
        ui.displayText("not yet implemented")
        groupMenu(displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn = callingMenusRtgIn)
      } else if (answer == 8) {
        ui.displayText("placeholder: nothing implemented here yet")
        groupMenu(displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn = callingMenusRtgIn)
      } else if (answer == 9 && answer <= choices.size) {
        new QuickGroupMenu(ui,mDB).quickGroupMenu(displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn = callingMenusRtgIn)
      } else if (answer == 0) None
      else if (answer > choices.length && answer <= (choices.length + objectsToDisplay.size)) {
        // those in the condition are 1-based, not 0-based.
        // lets user select a new entity and return to the main menu w/ that one displayed & current
        val choicesIndex = answer - choices.length - 1
        // user typed a letter to select an attribute (now 0-based)
        if (choicesIndex >= objectsToDisplay.size()) {
          ui.displayText("The program shouldn't have let us get to this point, but the selection " + answer + " is not in the list.")
          groupMenu(displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn = callingMenusRtgIn)
        } else {
          val entry = objectsToDisplay.get(choicesIndex)
          new EntityMenu(ui, mDB).entityMenu(0, entry.asInstanceOf[Entity], None, None, Some(group))
          groupMenu(0, relationToGroupIn, callingMenusRtgIn = callingMenusRtgIn)
        }
      } else {
        ui.displayText("invalid response")
        groupMenu(displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn = callingMenusRtgIn)
      }
    }
  }

}
