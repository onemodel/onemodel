/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003-2004 and 2008-2017 inclusive, 2019, and 2023, Luke A. Call.
    (That copyright statement was previously 2013-2015, until I remembered that much of Controller came from TextUI.scala and TextUI.java
    s free software, distributed under a license that includes honesty, the Golden Rule, guidelines around binary
    distribution, and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
struct GroupMenu {
/*%%
package org.onemodel.core.controllers

import org.onemodel.core._
import org.onemodel.core.model._

class GroupMenu(val ui: TextUI, let controller: Controller) {;

  /** Returns None if user wants out. The parameter callingMenusRtgIn exists only to preserve the value as may be used by quickGroupMenu, and passed
    * between it and here.
    */
  // see comment on helper method about tailrec
  //@tailrec
  // idea: There's some better scala idiom for this control logic around recursion and exception handling (& there's similar code in all "*Menu" classes):
  final fn groupMenu(group_in: Group, displayStartingRowNumberIn: Int, relationToGroupIn: Option[RelationToGroup],
                      //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) w/in this method!
                      callingMenusRtgIn: Option[RelationToGroup] = None, containingEntityIn -> Option<Entity>): Option<Entity> {
    try {
      groupMenu_helper(group_in, displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn, containingEntityIn)
    } catch {
      case e: Exception =>
        Util::handleException(e, ui, group_in.db)
        let ans = ui.ask_yes_no_question("Go back to what you were doing (vs. going out)?",Some("y"));
        if ans.is_defined && ans.get) groupMenu(group_in, displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn, containingEntityIn)
        else None
    }
  }

  // put @tailrec back when tail recursion works better on the JVM & don't get that err "...not in tail position" (unless we want to have all the calls
  // preserved, so that each previously seen individual menu is displayed when ESCaping back out of the stack of calls?).
  // BUT: does it still work when this recursive method calls other methods who then call this method? (I.e., can we avoid 'long method' smell, or does
  // any code wanting to be inside the tail recursion and make tail recursive calls, have to be directly inside the method?)
  //@tailrec
  //
    fn groupMenu_helper(group_in: Group, displayStartingRowNumberIn: Int, relationToGroupIn: Option[RelationToGroup],
                       //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) w/in this method!
                       callingMenusRtgIn: Option[RelationToGroup] = None, containingEntityIn: Option<Entity>) -> Option<Entity> {
    require(relationToGroupIn != null)

    let template_entity = group_in.get_class_template_entity;
    let choices = Vec<String>("Add entity to group (if you add an existing entity with a relationship to one group, that is effectively adding that group " +;
                                "as a subgroup to this one)",

                                "Import/Export...",
                                "Edit ...",
                                "Delete...",
                                "Go to...",
                                Util::LIST_NEXT_ITEMS_PROMPT,
                                "Filter (limit which are shown; unimplemented)",
                                "(stub)" /*sort?*/ ,
                                "Quick group menu")
    let displayDescription = if relationToGroupIn.is_defined) relationToGroupIn.get.get_display_string(0) else group_in.get_display_string(0);
    // (idea: maybe this use of color on next line could be removed, if people don't rely on the color change.  I originally added it as a visual
    // cue to aid my transition to using entities more & groups less. Same thing is done in QuickGroupMenu.)
    let leading_text: Vec<String> = Array(Color.yellow("ENTITY GROUP ") + "(regular menu: more complete, so slower for some things): " + displayDescription);
    let numDisplayableItems = ui.maxColumnarChoicesToDisplayAfter(leading_text.length, choices.length, Util::maxNameLength);
    let objectsToDisplay: Vec<Entity> = group_in.get_group_entries(displayStartingRowNumberIn, Some(numDisplayableItems));
    Util::add_remaining_count_to_prompt(choices, objectsToDisplay.size, group_in.get_size(4), displayStartingRowNumberIn)
    let statusesAndNames: Vec<String> = for (entity: Entity <- objectsToDisplay.toArray(Array[Entity]())) yield {;
      let numSubgroupsPrefix: String = controller.getEntityContentSizePrefix(entity);
      let archivedStatus = entity.get_archived_status_display_string;
      numSubgroupsPrefix + archivedStatus + entity.get_name + " " + controller.get_public_status_display_string(entity)
    }


    let response = ui.ask_which(Some(leading_text), choices, statusesAndNames);
    if response.isEmpty) None
    else {
      let answer = response.get;
      if answer == 1) {
        controller.add_entityToGroup(group_in)
        groupMenu(group_in, displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn, containingEntityIn)
      } else if answer == 2) {
        let importOrExport = ui.ask_which(None, Array("Import", "Export"), Vec<String>());
        if importOrExport.is_defined) {
          if importOrExport.get == 1) new ImportExport(ui, controller).importCollapsibleOutlineAsGroups(group_in)
          else if importOrExport.get == 2) {
            ui.display_text("not yet implemented: try it from an entity rather than a group where it is supported, for now.")
            //exportToCollapsibleOutline(entity_in)
          }
        }
        groupMenu(group_in, displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn, containingEntityIn)
      } else if answer == 3) {
        let editAnswer = ui.ask_which(Some(Vec<String>{Util::group_menu_leading_text(group_in)}),;
                                     Array("Edit group name",

                                           if group_in.get_new_entries_stick_to_top) {
                                             "Set group so new items added from the top highlight become the *2nd* entry (CURRENTLY: they stay at the top)."
                                           } else {
                                             "Set group so new items added from the top highlight become the *top* entry (CURRENTLY: they will be 2nd)."
                                           }))
        if editAnswer.is_defined) {
          if editAnswer.get == 1) {
            let ans = Util::edit_group_name(group_in, ui);
            if ans.is_defined) {
              // reread the RTG to get the updated info:
              groupMenu(group_in, displayStartingRowNumberIn,
                        if relationToGroupIn.is_defined) {
                          Some(new RelationToGroup(relationToGroupIn.get.db, relationToGroupIn.get.get_id, relationToGroupIn.get.get_parent_id(),
                                                   relationToGroupIn.get.get_attr_type_id(), relationToGroupIn.get.get_group_id))
                        } else None,
                        callingMenusRtgIn,
                        containingEntityIn)
            }
          } else if editAnswer.get == 2) {
            group_in.update(None, None, None, Some(!group_in.get_new_entries_stick_to_top), None, None)
          }
        }
        groupMenu(group_in, displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn, containingEntityIn)
      } else if answer == 4) {
        confirmAndDoDeletionOrRemoval(displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn, containingEntityIn, group_in, displayDescription,
                                      response)
      } else if answer == 5 && answer <= choices.length) {
        let containingEntities = group_in.get_entities_containing_group(0);
        let numContainingEntities = containingEntities.size;
        // (idea: make this next call efficient: now it builds them all when we just want a count; but is infrequent & likely small numbers)
        let choices = Array(if relationToGroupIn.is_defined) "Go edit the relation to group that led us here :" + displayDescription;
                            else "(stub)",
                            if numContainingEntities == 1) {
                              let entity = containingEntities.get(0)._2;
                              let entityStatusAndName = entity.get_archived_status_display_string + entity.get_name;
                              "Go to entity containing this group: " + entityStatusAndName
                            } else {
                              "See entities that contain this group ( " + numContainingEntities + ")"
                            },
                            if template_entity.is_defined) "Go to template entity" else "(stub: no template entity to go to)")
        //idea: consider: do we want this?:
        //(see similar comment in postgresqldatabase)
        //"See groups containing this group (" + numContainingGroups + ")")
        //val numContainingGroups = db.getContainingRelationToGroups(relationToGroupIn, 0).size

        let response = ui.ask_which(None, choices, Vec<String>());
        if response.isEmpty) None
        else {
          let ans = response.get;
          if ans == 1 && relationToGroupIn.is_defined) {
            fn update_relation_to_group(dhInOut: RelationToGroupDataHolder) {
              //idea: does this make sense, to only update the dates when we prompt for everything on initial add? change(or note2later) update everything?
              relationToGroupIn.get.update(Some(dhInOut.attr_type_id), Some(dhInOut.groupId), dhInOut.valid_on_date, Some(dhInOut.observation_date))
            }
            let relationToGroupDH: RelationToGroupDataHolder = new RelationToGroupDataHolder(relationToGroupIn.get.get_parent_id(),;
                                                                                             relationToGroupIn.get.get_attr_type_id(),
                                                                                             relationToGroupIn.get.get_group_id,
                                                                                             relationToGroupIn.get.get_valid_on_date(),
                                                                                             relationToGroupIn.get.get_observation_date())
            let (newRelationToGroup: Option[RelationToGroup], newGroup: Group) = {;
              if controller.askForInfoAndUpdateAttribute[RelationToGroupDataHolder](relationToGroupIn.get.db, relationToGroupDH, askForAttrTypeId = true,
                                                                                     Util::RELATION_TO_GROUP_TYPE,
                                                                                     "CHOOSE TYPE OF Relation to Entity:",
                                                                                     controller.askForRelToGroupInfo, update_relation_to_group)) {
                //force a reread from the DB so it shows the right info on the repeated menu, for these things which could have been changed:
                (Some(new RelationToGroup(relationToGroupIn.get.db, relationToGroupIn.get.get_id, relationToGroupDH.entity_id,
                                         relationToGroupDH.attr_type_id, relationToGroupDH.groupId)),
                  new Group(group_in.db, relationToGroupDH.groupId))
              } else {
                (relationToGroupIn, group_in)
              }
            }
            groupMenu(newGroup, displayStartingRowNumberIn, newRelationToGroup, callingMenusRtgIn, containingEntityIn)
          } else if ans == 2 && ans <= choices.length) {
            let entity: Option<Entity> =;
              if numContainingEntities == 1) {
                Some(containingEntities.get(0)._2)
              } else {
                controller.chooseAmongEntities(containingEntities)
              }

            if entity.is_defined) {
              new EntityMenu(ui, controller).entityMenu(entity.get)
            }
            //ck 1st if it exists, if not return None. It could have been deleted while navigating around.
            if group_in.db.group_key_exists(group_in.get_id)) groupMenu(group_in, displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn, containingEntityIn)
            else None
          } else if ans == 3 && template_entity.is_defined && ans <= choices.length) {
            new EntityMenu(ui, controller).entityMenu(template_entity.get)
            //ck 1st if it exists, if not return None. It could have been deleted while navigating around.
            if group_in.db.group_key_exists(group_in.get_id)) groupMenu(group_in, displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn, containingEntityIn)
            else None
          } else {
            ui.display_text("invalid response")
            groupMenu(group_in, displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn, containingEntityIn)
          }
        }
      } else if answer == 6) {
        let displayRowsStartingWithCounter: i32 = {;
          let currentPosition = displayStartingRowNumberIn + objectsToDisplay.size;
          if currentPosition >= group_in.get_size(4)) {
            ui.display_text("End of attribute list found; restarting from the beginning.")
            0 // start over
          } else currentPosition
        }
        groupMenu(group_in, displayRowsStartingWithCounter, relationToGroupIn, callingMenusRtgIn, containingEntityIn)
      } else if answer == 7) {
        ui.display_text("not yet implemented")
        groupMenu(group_in, displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn, containingEntityIn)
      } else if answer == 8) {
        ui.display_text("placeholder: nothing implemented here yet")
        groupMenu(group_in, displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn, containingEntityIn)
      } else if answer == 9 && answer <= choices.length) {
        new QuickGroupMenu(ui, controller).quickGroupMenu(group_in, displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn = callingMenusRtgIn,
                                                             containingEntityIn = containingEntityIn)
      } else if answer == 0) None
      else if answer > choices.length && answer <= (choices.length + objectsToDisplay.size)) {
        // those in the condition are 1-based, not 0-based.
        // lets user select a new entity and return to the main menu w/ that one displayed & current
        let choices_index = answer - choices.length - 1;
        // user typed a letter to select an attribute (now 0-based)
        if choices_index >= objectsToDisplay.size()) {
          ui.display_text("The program shouldn't have let us get to this point, but the selection " + answer + " is not in the list.")
          groupMenu(group_in, displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn, containingEntityIn)
        } else {
          let entry = objectsToDisplay.get(choices_index);
          new EntityMenu(ui, controller).entityMenu(entry.asInstanceOf[Entity], containingGroupIn = Some(group_in))
          groupMenu(group_in, 0, relationToGroupIn, callingMenusRtgIn, containingEntityIn)
        }
      } else {
        ui.display_text("invalid response")
        groupMenu(group_in, displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn, containingEntityIn)
      }
    }
  }

    fn confirmAndDoDeletionOrRemoval(displayStartingRowNumberIn: Int, relationToGroupIn: Option[RelationToGroup], callingMenusRtgIn: Option[RelationToGroup],
                                    containingEntityIn: Option<Entity>, group_in: Group, groupDescrIn: String,
                                    response: Option[Int]) -> Option<Entity> {
    require(group_in.get_id == relationToGroupIn.get.get_group_id)
    let totalInGroup = group_in.get_size(3);
    let numNonArchivedEntitiesInGroup: i64 = group_in.get_size(1);
    let num_archived_inGroup = totalInGroup - numNonArchivedEntitiesInGroup;
    require(num_archived_inGroup == group_in.get_size(2))
    let (non_archivedContainingCount, archivedContainingCount) = group_in.get_count_of_entities_containing_group;
    let mut choices: Vec<String> = Array("Delete group definition & remove from all relationships where it is found?",;
                                       "Delete group definition & remove from all relationships where it is found, AND delete all entities in it?")
    if containingEntityIn.is_defined && relationToGroupIn.is_defined) {
      choices = choices :+ "Delete the link from the containing entity:" + Util::NEWLN +
                           "    \"" + containingEntityIn.get.get_name + "\"," + Util::NEWLN +
                           "  ...to this Group?:" + Util::NEWLN +
                           "    \"" + groupDescrIn + "\""
    }
    let response = ui.ask_which(Some(Array("DELETION:  (This group contains " + totalInGroup + " entities, including " + num_archived_inGroup + " archived, and is " +;
                                          Util::get_containing_entities_description(non_archivedContainingCount, archivedContainingCount) + ")")),
                               choices, Vec<String>())
    if response.isEmpty) groupMenu(group_in, displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn, containingEntityIn)
    else {
      let ans = response.get;
      if ans == 1) {
        let ans = ui.ask_yes_no_question("DELETE this group definition AND remove from all entities that link to it (but not entities it contains): **ARE " +;
                                      "YOU REALLY SURE?**")
        if ans.is_defined && ans.get) {
          group_in.delete()
          ui.display_text("Deleted group definition: \"" + groupDescrIn + "\"" + ".")
          None
        } else {
          ui.display_text("Did not delete group definition.", false);
          groupMenu(group_in, displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn, containingEntityIn)
        }
      } else if ans == 2) {
        // if calculating the total to be deleted for this prompt or anything else recursive, we have to deal with looping data & not duplicate it in
        // counting.
        // IDEA:  ******ALSO WHEN UPDATING THIS TO BE RECURSIVE, OR CONSIDERING SUCH, CONSIDER ALSO HOW TO ADDRESS ARCHIVED ENTITIES: SUCH AS IF ALL QUERIES
        // USED IN THIS WILL ALSO CK FOR ARCHIVED ENTITIES, AND ANYTHING ELSE?  And show the # of archived entities to the user or suggest that they view
        // those
        // also be4 deleting everything?
        let ans = ui.ask_yes_no_question("DELETE this group definition from *all* relationships where it is found, *AND* its entities, " +;
                                      "with *ALL* entities and their \"subgroups\" that they eventually " +
                                      "refer" +
                                      " to, recursively (actually, the recursion is not finished and will probably fail if you have nesting): *******ARE " +
                                      "YOU REALLY SURE?******")
        if ans.is_defined && ans.get) {
          let ans = ui.ask_yes_no_question("Um, this seems unusual; note that this will also delete archived (~invisible) entities with the group!.  " +;
                                        "Really _really_ sure?  " +
                                        "I certainly hope you make regular backups of the data AND TEST " +
                                        " RESTORES.  (Note: the deletion does(n't yet do) recursion but doesn't yet properly handle groups that " +
                                        "loop--that eventually contain themselves.)  Proceed to delete it all?:")
          if ans.is_defined && ans.get) {
            //idea: could put a ck here to see if entities are members of some other group also, and give user a helpful message instead of just
            //hitting the constraint & throwing exception when the deletion is attempted.
            group_in.delete_with_entities()
            ui.display_text("Deleted relation to group\"" + groupDescrIn + "\", along with the " + totalInGroup + " entities: " + ".")
            None
          } else None
        } else {
          ui.display_text("Did not delete group.", false);
          groupMenu(group_in, displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn, containingEntityIn)
        }
      } else if ans == 3 && relationToGroupIn.is_defined) {
        if removingGroupReferenceFromEntity_Menu(relationToGroupIn.get, group_in, containingEntityIn.get))
          None
        else
          groupMenu(group_in, displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn, containingEntityIn)
      } else {
        ui.display_text("invalid response")
        groupMenu(group_in, displayStartingRowNumberIn, relationToGroupIn, callingMenusRtgIn, containingEntityIn)
      }
    }
  }

  /**
   * @return If it was deleted.
   */
    fn removingGroupReferenceFromEntity_Menu(relationToGroupIn: RelationToGroup, group_in: Group, containingEntityIn: Entity) -> bool {
    let (non_archivedCount, archivedCount) = group_in.get_count_of_entities_containing_group;
    let ans = ui.ask_yes_no_question("REMOVE this group from being an attribute of the entity \'" + containingEntityIn.get_name + "\": ARE YOU SURE? (This isn't " +;
                                  "a deletion. It can still be found by searching, and is " +
                                  Util::get_containing_entities_description(non_archivedCount, archivedCount) + ").", Some(""))
    if ans.is_defined && ans.get) {
      relationToGroupIn.delete()
      true

      //is it ever desirable to keep the next line instead of the 'None'? not in most typical usage it seems, but?:
      //entityMenu(startingAttributeIndexIn, entity_in, relationSourceEntityIn, relationIn)
    } else {
      ui.display_text("Did not remove group from the entity.", false);
      false

      //is it ever desirable to keep the next line instead of the 'None'? not in most typical usage it seems, but?:
      //entityMenu(startingAttributeIndexIn, entity_in, relationSourceEntityIn, relationIn, containingGroupIn)
    }
  }

*/
}
