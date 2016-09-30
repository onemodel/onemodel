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
package org.onemodel.core.controllers

import org.onemodel.core._
import org.onemodel.core.database.PostgreSQLDatabase
import org.onemodel.core.model._

class GroupMenu(val ui: TextUI, val db: PostgreSQLDatabase, val controller: Controller) {

  /** Returns None if user wants out. The parameter callingMenusRtgIn exists only to preserve the value as may be used by quickGroupMenu, and passed
    * between it and here.
    */
  // see comment on helper method about tailrec
  //@tailrec
  // idea: There's some better scala idiom for this control logic around recursion and exception handling (& there's similar code in all "*Menu" classes):
  final def groupMenu(groupIn: Group, displayStartingRowNumberIn: Int, relationToGroupIn: Option[RelationToGroup],
                      //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) w/in this method!
                      callingMenusRtgIn: Option[RelationToGroup] = None, containingEntityIn: Option[Entity]): Option[Entity] = {
    try {
      groupMenu_helper(groupIn, displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn, containingEntityIn)
    } catch {
      case e: Exception =>
        controller.handleException(e)
        val ans = ui.askYesNoQuestion("Go back to what you were doing (vs. going out)?",Some("y"))
        if (ans.isDefined && ans.get) groupMenu(groupIn, displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn, containingEntityIn)
        else None
    }
  }

  // put @tailrec back when tail recursion works better on the JVM & don't get that err "...not in tail position" (unless we want to have all the calls
  // preserved, so that each previously seen individual menu is displayed when ESCaping back out of the stack of calls?).
  // BUT: does it still work when this recursive method calls other methods who then call this method? (I.e., can we avoid 'long method' smell, or does
  // any code wanting to be inside the tail recursion and make tail recursive calls, have to be directly inside the method?)
  //@tailrec
  //
  def groupMenu_helper(groupIn: Group, displayStartingRowNumberIn: Int, relationToGroupIn: Option[RelationToGroup],
                       //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) w/in this method!
                       callingMenusRtgIn: Option[RelationToGroup] = None, containingEntityIn: Option[Entity]): Option[Entity] = {
    require(relationToGroupIn != null)

    val templateEntity = groupIn.getClassTemplateEntity
    val choices = Array[String]("Add entity to group (if you add an existing entity with a relationship to one group, that is effectively adding that group " +
                                "as a subgroup to this one)",

                                "Import/Export...",
                                "Edit ...",
                                "Delete...",
                                "Go to...",
                                controller.listNextItemsPrompt,
                                "Filter (limit which are shown; unimplemented)",
                                "(stub)" /*sort?*/ ,
                                "Quick group menu")
    val displayDescription = if (relationToGroupIn.isDefined) relationToGroupIn.get.getDisplayString(0) else groupIn.getDisplayString(0)
    // (idea: maybe this use of color on next line could be removed, if people don't rely on the color change.  I originally added it as a visual
    // cue to aid my transition to using entities more & groups less. Same thing is done in QuickGroupMenu.)
    val leadingText: Array[String] = Array(Color.yellow("ENTITY GROUP ") + "(regular menu: more complete, so slower for some things): " + displayDescription)
    val numDisplayableItems = ui.maxColumnarChoicesToDisplayAfter(leadingText.length, choices.length, Controller.maxNameLength)
    val objectsToDisplay: java.util.ArrayList[Entity] = groupIn.getGroupEntries(displayStartingRowNumberIn, Some(numDisplayableItems))
    controller.addRemainingCountToPrompt(choices, objectsToDisplay.size, groupIn.getSize(4), displayStartingRowNumberIn)
    val statusesAndNames: Array[String] = for (entity: Entity <- objectsToDisplay.toArray(Array[Entity]())) yield {
      val numSubgroupsPrefix: String = controller.getEntityContentSizePrefix(entity.getId)
      val archivedStatus = entity.getArchivedStatusDisplayString
      numSubgroupsPrefix + archivedStatus + entity.getName + " " + controller.getPublicStatusDisplayString(entity)
    }


    val response = ui.askWhich(Some(leadingText), choices, statusesAndNames)
    if (response.isEmpty) None
    else {
      val answer = response.get
      if (answer == 1) {
        controller.addEntityToGroup(groupIn)
        groupMenu(groupIn, displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn, containingEntityIn)
      } else if (answer == 2) {
        val importOrExport = ui.askWhich(None, Array("Import", "Export"), Array[String]())
        if (importOrExport.isDefined) {
          if (importOrExport.get == 1) new ImportExport(ui, db, controller).importCollapsibleOutlineAsGroups(groupIn)
          else if (importOrExport.get == 2) {
            ui.displayText("not yet implemented: try it from an entity rather than a group where it is supported, for now.")
            //exportToCollapsibleOutline(entityIn)
          }
        }
        groupMenu(groupIn, displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn, containingEntityIn)
      } else if (answer == 3) {
        val editAnswer = ui.askWhich(Some(Array[String]{controller.groupMenuLeadingText(groupIn)}),
                                     Array("Edit group name",

                                           if (groupIn.getNewEntriesStickToTop) {
                                             "Set group so new items added from the top highlight become the *2nd* entry (CURRENTLY: they stay at the top)."
                                           } else {
                                             "Set group so new items added from the top highlight become the *top* entry (CURRENTLY: they will be 2nd)."
                                           }))
        if (editAnswer.isDefined) {
          if (editAnswer.get == 1) {
            val ans = controller.editGroupName(groupIn)
            if (ans.isDefined) {
              // reread the RTG to get the updated info:
              groupMenu(groupIn, displayStartingRowNumberIn,
                        if (relationToGroupIn.isDefined) {
                          Some(new RelationToGroup(db, relationToGroupIn.get.getId, relationToGroupIn.get.getParentId, relationToGroupIn.get.getAttrTypeId,
                                                   relationToGroupIn.get.getGroupId))
                        } else None,
                        callingMenusRtgIn,
                        containingEntityIn)
            }
          } else if (editAnswer.get == 2) {
            groupIn.update(None, None, None, Some(!groupIn.getNewEntriesStickToTop), None, None)
          }
        }
        groupMenu(groupIn, displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn, containingEntityIn)
      } else if (answer == 4) {
        confirmAndDoDeletionOrRemoval(displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn, containingEntityIn, groupIn, displayDescription,
                                      response)
      } else if (answer == 5 && answer <= choices.length) {
        val containingEntities = db.getEntitiesContainingGroup(groupIn.getId, 0)
        val numContainingEntities = containingEntities.size
        // (idea: make this next call efficient: now it builds them all when we just want a count; but is infrequent & likely small numbers)
        val choices = Array(if (relationToGroupIn.isDefined) "Go edit the relation to group that led us here :" + displayDescription
                            else "(stub)",
                            if (numContainingEntities == 1) {
                              val entity = containingEntities.get(0)._2
                              val entityStatusAndName = entity.getArchivedStatusDisplayString + entity.getName
                              "Go to entity containing this group: " + entityStatusAndName
                            } else {
                              "See entities that contain this group ( " + numContainingEntities + ")"
                            },
                            if (templateEntity.isDefined) "Go to template entity" else "(stub: no template entity to go to)")
        //idea: consider: do we want this?:
        //(see similar comment in postgresqldatabase)
        //"See groups containing this group (" + numContainingGroups + ")")
        //val numContainingGroups = mDB.getContainingRelationToGroups(relationToGroupIn, 0).size

        val response = ui.askWhich(None, choices, Array[String]())
        if (response.isEmpty) None
        else {
          val ans = response.get
          if (ans == 1 && relationToGroupIn.isDefined) {
            def updateRelationToGroup(dhInOut: RelationToGroupDataHolder) {
              //idea: does this make sense, to only update the dates when we prompt for everything on initial add? change(or note2later) update everything?
              relationToGroupIn.get.update(dhInOut.validOnDate, Some(dhInOut.observationDate))
            }
            val relationToGroupDH: RelationToGroupDataHolder = new RelationToGroupDataHolder(relationToGroupIn.get.getParentId,
                                                                                             relationToGroupIn.get.getAttrTypeId,
                                                                                             relationToGroupIn.get.getGroupId,
                                                                                             relationToGroupIn.get.getValidOnDate,
                                                                                             relationToGroupIn.get.getObservationDate)
            controller.askForInfoAndUpdateAttribute[RelationToGroupDataHolder](relationToGroupDH, askForAttrTypeId = true, Controller.RELATION_TO_GROUP_TYPE,
                                                                    "CHOOSE TYPE OF [correct me: or edit existing?] Relation to Entity:",
                                                                    controller.askForRelToGroupInfo, updateRelationToGroup)
            //force a reread from the DB so it shows the right info on the repeated menu:
            groupMenu(groupIn, displayStartingRowNumberIn,
                      Some(new RelationToGroup(db, relationToGroupIn.get.getId, relationToGroupDH.entityId, relationToGroupDH.attrTypeId, relationToGroupDH.groupId)),
                      callingMenusRtgIn, containingEntityIn)
          } else if (ans == 2 && ans <= choices.length) {
            val entity: Option[Entity] =
              if (numContainingEntities == 1) {
                Some(containingEntities.get(0)._2)
              } else {
                controller.chooseAmongEntities(containingEntities)
              }

            if (entity.isDefined)
              new EntityMenu(ui, db, controller).entityMenu(entity.get)

            //ck 1st if it exists, if not return None. It could have been deleted while navigating around.
            if (db.groupKeyExists(groupIn.getId)) groupMenu(groupIn, displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn, containingEntityIn)
            else None
          } else if (ans == 3 && templateEntity.isDefined && ans <= choices.length) {
            new EntityMenu(ui, db, controller).entityMenu(templateEntity.get)
            //ck 1st if it exists, if not return None. It could have been deleted while navigating around.
            if (db.groupKeyExists(groupIn.getId)) groupMenu(groupIn, displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn, containingEntityIn)
            else None
          } else {
            ui.displayText("invalid response")
            groupMenu(groupIn, displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn, containingEntityIn)
          }
        }
      } else if (answer == 6) {
        val displayRowsStartingWithCounter: Int = {
          val currentPosition = displayStartingRowNumberIn + objectsToDisplay.size
          if (currentPosition >= groupIn.getSize(4)) {
            ui.displayText("End of attribute list found; restarting from the beginning.")
            0 // start over
          } else currentPosition
        }
        groupMenu(groupIn, displayRowsStartingWithCounter, relationToGroupIn, callingMenusRtgIn, containingEntityIn)
      } else if (answer == 7) {
        ui.displayText("not yet implemented")
        groupMenu(groupIn, displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn, containingEntityIn)
      } else if (answer == 8) {
        ui.displayText("placeholder: nothing implemented here yet")
        groupMenu(groupIn, displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn, containingEntityIn)
      } else if (answer == 9 && answer <= choices.length) {
        new QuickGroupMenu(ui,db, controller).quickGroupMenu(groupIn, displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn = callingMenusRtgIn,
                                                             containingEntityIn = containingEntityIn)
      } else if (answer == 0) None
      else if (answer > choices.length && answer <= (choices.length + objectsToDisplay.size)) {
        // those in the condition are 1-based, not 0-based.
        // lets user select a new entity and return to the main menu w/ that one displayed & current
        val choicesIndex = answer - choices.length - 1
        // user typed a letter to select an attribute (now 0-based)
        if (choicesIndex >= objectsToDisplay.size()) {
          ui.displayText("The program shouldn't have let us get to this point, but the selection " + answer + " is not in the list.")
          groupMenu(groupIn, displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn, containingEntityIn)
        } else {
          val entry = objectsToDisplay.get(choicesIndex)
          new EntityMenu(ui, db, controller).entityMenu(entry.asInstanceOf[Entity], containingGroupIn = Some(groupIn))
          groupMenu(groupIn, 0, relationToGroupIn, callingMenusRtgIn, containingEntityIn)
        }
      } else {
        ui.displayText("invalid response")
        groupMenu(groupIn, displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn, containingEntityIn)
      }
    }
  }

  def confirmAndDoDeletionOrRemoval(displayStartingRowNumberIn: Int, relationToGroupIn: Option[RelationToGroup], callingMenusRtgIn: Option[RelationToGroup],
                                    containingEntityIn: Option[Entity], groupIn: Group, groupDescrIn: String,
                                    response: Option[Int]): Option[Entity] = {
    require(groupIn.getId == relationToGroupIn.get.getGroupId)
    val totalInGroup = groupIn.getSize(3)
    val numNonArchivedEntitiesInGroup: Long = groupIn.getSize(1)
    val numArchivedInGroup = totalInGroup - numNonArchivedEntitiesInGroup
    require(numArchivedInGroup == groupIn.getSize(2))
    val (nonArchivedContainingCount, archivedContainingCount) = db.getCountOfEntitiesContainingGroup(groupIn.getId)
    var choices: Array[String] = Array("Delete group definition & remove from all relationships where it is found?",
                                       "Delete group definition & remove from all relationships where it is found, AND delete all entities in it?")
    if (containingEntityIn.isDefined && relationToGroupIn.isDefined) {
      choices = choices :+ "Delete the link between the containing entity \"" + containingEntityIn.get.getName + "\", and this group?: " +
                           groupDescrIn
    }
    val response = ui.askWhich(Some(Array("DELETION:  (This group contains " + totalInGroup + " entities, including " + numArchivedInGroup + " archived, and is " +
                                          controller.getContainingEntitiesDescription(nonArchivedContainingCount, archivedContainingCount) + ")")),
                               choices, Array[String]())
    if (response.isEmpty) groupMenu(groupIn, displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn, containingEntityIn)
    else {
      val ans = response.get
      if (ans == 1) {
        val ans = ui.askYesNoQuestion("DELETE this group definition AND remove from all entities that link to it (but not entities it contains): **ARE " +
                                      "YOU REALLY SURE?**")
        if (ans.isDefined && ans.get) {
          groupIn.delete()
          ui.displayText("Deleted group definition: \"" + groupDescrIn + "\"" + ".")
          None
        } else {
          ui.displayText("Did not delete group definition.", waitForKeystrokeIn = false)
          groupMenu(groupIn, displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn, containingEntityIn)
        }
      } else if (ans == 2) {
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
                                      "YOU REALLY SURE?******")
        if (ans.isDefined && ans.get) {
          val ans = ui.askYesNoQuestion("Um, this seems unusual; note that this will also delete archived (~invisible) entities with the group!.  " +
                                        "Really _really_ sure?  " +
                                        "I certainly hope you make regular backups of the data AND TEST " +
                                        " RESTORES.  (Note: the deletion does(n't yet do) recursion but doesn't yet properly handle groups that " +
                                        "loop--that eventually contain themselves.)  Proceed to delete it all?:")
          if (ans.isDefined && ans.get) {
            //idea: could put a ck here to see if entities are members of some other group also, and give user a helpful message instead of just
            //hitting the constraint & throwing exception when the deletion is attempted.
            groupIn.deleteWithEntities()
            ui.displayText("Deleted relation to group\"" + groupDescrIn + "\", along with the " + totalInGroup + " entities: " + ".")
            None
          } else None
        } else {
          ui.displayText("Did not delete group.", waitForKeystrokeIn = false)
          groupMenu(groupIn, displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn, containingEntityIn)
        }
      } else if (ans == 3 && relationToGroupIn.isDefined) {
        if (removingGroupReferenceFromEntity_Menu(relationToGroupIn.get, containingEntityIn.get))
          None
        else
          groupMenu(groupIn, displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn, containingEntityIn)
      } else {
        ui.displayText("invalid response")
        groupMenu(groupIn, displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn, containingEntityIn)
      }
    }
  }

  /**
   * @return If it was deleted.
   */
  def removingGroupReferenceFromEntity_Menu(relationToGroupIn: RelationToGroup, containingEntityIn: Entity): Boolean = {
    val (nonArchivedCount, archivedCount) = db.getCountOfEntitiesContainingGroup(relationToGroupIn.getGroupId)
    val ans = ui.askYesNoQuestion("REMOVE this group from being an attribute of the entity \'" + containingEntityIn.getName + "\": ARE YOU SURE? (This isn't " +
                                  "a deletion. It can still be found by searching, and is " +
                                  controller.getContainingEntitiesDescription(nonArchivedCount, archivedCount) + ").", Some(""))
    if (ans.isDefined && ans.get) {
      relationToGroupIn.delete()
      true

      //is it ever desirable to keep the next line instead of the 'None'? not in most typical usage it seems, but?:
      //entityMenu(startingAttributeIndexIn, entityIn, relationSourceEntityIn, relationIn)
    } else {
      ui.displayText("Did not remove group from the entity.", waitForKeystrokeIn = false)
      false

      //is it ever desirable to keep the next line instead of the 'None'? not in most typical usage it seems, but?:
      //entityMenu(startingAttributeIndexIn, entityIn, relationSourceEntityIn, relationIn, containingGroupIn)
    }
  }

}
