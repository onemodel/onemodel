/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003-2004 and 2008-2019 inclusive, and 2023, Luke A. Call.
    (That copyright statement was previously 2013-2015, until I remembered that much of Controller came from TextUI.scala and TextUI.java before that.)
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
struct QuickGroupMenu {
/*%%
package org.onemodel.core.controllers

import org.onemodel.core._
import org.onemodel.core.model._
import org.onemodel.core.{Color, OmException, TextUI}

* Allows sorting of group entries, quick work like for brainstorming.
  *
class QuickGroupMenu(override let ui: TextUI, val controller: Controller) extends SortableEntriesMenu(ui) {;
  // The @tailrec is desired when possible,
  // because it seems that otherwise we might try to ESC back to a menu instance which is attempting to view a deleted entity, & crash!  But see the comment
  // mentioning why not to have it, below.  Maybe we need to use a loop around the menu instead of tail recursion in this case, if there is not a
  // way to turn the tail optimization off for a particular line.
  //@tailrec
  //scoping idea: see idea at beginning of EntityMenu.entityMenu
  /**
   * @param group_in The id of the group whose info is being displayed.
   * @param starting_display_row_index_in refers to the 0-based index among all possible displayable rows (i.e., if we have displayed
   *                                  20 objects out of 100, and the user says to go to the next 20, the starting_display_row_index_in would become 21.
   * @param relationToGroupIn Matches the group_in parameter, but with more info when available, about the RelationToGroup that this group menu display shows,
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
    fn quickGroupMenu(group_in: Group, starting_display_row_index_in: Int, relationToGroupIn: Option[RelationToGroup] = None,
                     highlightedEntityIn: Option<Entity> = None, targetForMovesIn: Option<Entity> = None,
                     //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) w/in this method!
                     callingMenusRtgIn: Option[RelationToGroup] = None, containingEntityIn: Option<Entity>) -> Option<Entity> {
    try {
      quickGroupMenu_doTheWork(group_in, starting_display_row_index_in, relationToGroupIn, highlightedEntityIn, targetForMovesIn, callingMenusRtgIn,
                               containingEntityIn)
    } catch {
      case e: Exception =>
        Util.handleException(e, ui, group_in.db)
        let ans = ui.ask_yes_no_question("Go back to what you were doing (vs. going out)?", Some("y"));
        if ans.is_defined && ans.get) quickGroupMenu(group_in, starting_display_row_index_in, relationToGroupIn, highlightedEntityIn, targetForMovesIn,
                                                     callingMenusRtgIn, containingEntityIn)
        else None
    }

  }

  /** Should be called only if the targetEntityIn has 0 or 1 RelationToGroups (no more).
    */
    fn createNewOrFindOneGroupOnEntity(group_in: Group, targetEntitysRtgCount: i64, targetEntityIn: Entity) -> (i64, i64, i64) {
    // if there is 1 (obvious) destination, or no RTG on the selected entity (1 can be created), then add new entry there
    let (targetRelationToGroupId: i64, targetRelationTypeId: i64, targetGroupId: i64) = {;
      if targetEntitysRtgCount == 0) {
        let name: String = targetEntityIn.get_name;
        let (newGroup: Group, newRTG: RelationToGroup) = targetEntityIn.create_groupAndAddHASRelationToIt(name, group_in.getMixedClassesAllowed,;
                                                                                                         System.currentTimeMillis)
        (newRTG.get_id, newRTG.get_attr_type_id(), newGroup.get_id)
      } else {
        // given above conditions (w/ moveTargetIndexInObjList, and rtgCount twice), there must be exactly one, or there's a bug:
        let (rtgId, rel_type_id, gid, _, moreAvailable): (Option<i64>, Option<i64>,;
          Option<i64>, Option<String>, Boolean) = targetEntityIn.findRelationToAndGroup

        if gid.isEmpty || rel_type_id.isEmpty || moreAvailable) throw new OmException("Found " + (if gid.isEmpty) 0 else ">1") + " but by the earlier " +
                                                                                     "checks, " +
                                                                                     "there should be exactly one group in entity " + targetEntityIn.get_id
                                                                                     + ": " +
                                                                                     targetEntityIn.get_name)
        (rtgId.get, rel_type_id.get, gid.get)
      }
    }
    (targetRelationToGroupId, targetRelationTypeId, targetGroupId)
  }

    fn moveSelectedEntry(group_in: Group, starting_display_row_index_in: Int, relationToGroupIn: Option[RelationToGroup], targetForMovesIn: Option<Entity>,
                        highlightedIndexInObjListIn: Int, moveTargetIndexInObjList: Option[Int], highlightedEntry: Entity,
                        highlightedObjId: i64, obj_ids: Array[i64], objectsToDisplay: Vec<Entity>,
                        callingMenusRtgIn: Option[RelationToGroup] = None, containingEntityIn: Option<Entity> = None) -> Option<Entity> {
    let choices = Vec<String>(// these are ordered for convenience in doing them w/ the left hand: by frequency of use, and what seems easiest to remember;
                                // for common operations with the 4 fingers sitting on the '1234' keys.  Using LH more in this because my RH gets tired more,
                                // and it seems like often people have their RH on the mouse.
                                "Move up " + controller.moveFartherCount,
                                "Move up 5", "Move up 1", "Move down 1", "Move down 5",
                                "Move down " + controller.moveFartherCount,

                                if targetForMovesIn.is_defined) "Move (*) to selected target (+, if any)"
                                else "(stub: have to choose a target before you can move entries into it)",

                                "Move (*) to calling menu (up one)",
                                "Move down " + controller.moveFarthestCount + " but keep data display position "
                                // idea: make an option here which is a "quick archive"? (for removing completed tasks: maybe only after showing
                                // archived things and "undo" works well, or use 9 for the 'cut' part of a logical 'cut/paste' operation to move something?)
                                // But, would have to start using alt keys to distinguish between options chosen when there are that many?
                               )
    let response = ui.ask_which(None, choices, Vec<String>(), highlightIndexIn = Some(highlightedIndexInObjListIn),;
                               secondaryHighlightIndexIn = moveTargetIndexInObjList)
    if response.isEmpty) quickGroupMenu(group_in, highlightedIndexInObjListIn, relationToGroupIn, Some(highlightedEntry), targetForMovesIn, callingMenusRtgIn,
                                         containingEntityIn)
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
          let possibleDisplayStartingRowNumber = placeEntryInPosition(group_in.db, group_in.get_id, group_in.get_size(4), numRowsToMove, forwardNotBack,;
                               starting_display_row_index_in, highlightedObjId, highlightedIndexInObjListIn,
                               Some(highlightedObjId), objectsToDisplay.size, -1, Some(-1))
          if answer != 9) {
            possibleDisplayStartingRowNumber
          } else {
            // (see note at same place in EntityMenu, re the position and the highlight)
            starting_display_row_index_in
          }
        }
        quickGroupMenu(group_in, displayStartingRowNumber, relationToGroupIn, Some(highlightedEntry), targetForMovesIn, callingMenusRtgIn, containingEntityIn)
      } else if answer == 7 && targetForMovesIn.is_defined) {
        let targetRtgCount: i64 = targetForMovesIn.get.get_relation_to_group_count;
        if moveTargetIndexInObjList.isEmpty) {
          ui.display_text("Target must be selected (shows '+').")
          quickGroupMenu(group_in, starting_display_row_index_in, relationToGroupIn, Some(highlightedEntry), targetForMovesIn, callingMenusRtgIn, containingEntityIn)
        } else {
          if targetRtgCount > 1) {
            // can't guess which subgroup so just move it to the entity (idea: could ask whether to do that or go to which subgroup, perhaps...)
            group_in.move_entity_from_group_to_local_entity(targetForMovesIn.get.get_id, highlightedObjId, get_sorting_index(group_in.db, group_in.get_id, -1, highlightedObjId))
            let entityToHighlight: Option<Entity> = Util.find_entity_to_highlight_next(obj_ids.length, objectsToDisplay, removedOneIn = true,;
                                                                                         highlightedIndexInObjListIn, highlightedEntry)
            quickGroupMenu(group_in, starting_display_row_index_in, relationToGroupIn, entityToHighlight, targetForMovesIn, callingMenusRtgIn, containingEntityIn)
          } else {
            let (_, defaultToUsingSubgroup: Option<bool>) = useSubgroup(targetForMovesIn.get);
            if defaultToUsingSubgroup.isEmpty) {
              // user just wanted out of the question (whether the code can get here depends on parms passed to the ui question in above call to useSubgroup)
              quickGroupMenu(group_in, starting_display_row_index_in, relationToGroupIn, Some(highlightedEntry), targetForMovesIn, callingMenusRtgIn,
                             containingEntityIn)
            } else {
              if defaultToUsingSubgroup.get) {
                // if there is 1 (obvious) destination, or no RTG on the selected entity (1 can be created), then move it there
                let (_, _, targetGroupId) = createNewOrFindOneGroupOnEntity(group_in, targetRtgCount, targetForMovesIn.get);
                // about the sortingIndex:  see comment on db.moveEntityToNewGroup.
                group_in.moveEntityToDifferentGroup(targetGroupId, highlightedObjId, get_sorting_index(group_in.db, group_in.get_id, -1, highlightedObjId))
              } else {
                // getting here means to just create a RelationToLocalEntity on the entity, not a subgroup:
                group_in.move_entity_from_group_to_local_entity(targetForMovesIn.get.get_id, highlightedObjId, get_sorting_index(group_in.db, group_in.get_id,
                                                                                                                  -1, highlightedObjId))
              }
              let entityToHighlight: Option<Entity> = Util.find_entity_to_highlight_next(obj_ids.length, objectsToDisplay, removedOneIn = true,;
                                                                                           highlightedIndexInObjListIn, highlightedEntry)
              quickGroupMenu(group_in, starting_display_row_index_in, relationToGroupIn, entityToHighlight, targetForMovesIn, callingMenusRtgIn, containingEntityIn)
            }
          }
        }
      } else if answer == 8) {
        // if there is 1 (provided or guessable) destination), then move it there
        let (targetGroupId: Option<i64>, targetEntity: Option<Entity>) = {;
          // see whether will be moving it to an entity or a group, if anywhere.
          // ONLY RETURN ONE OF THE TWO, since the below relies on that to know which to do.
          if callingMenusRtgIn.is_defined && containingEntityIn.is_defined) {
            let answer = ui.ask_which(None,;
                        Array("Move it to the containing entity: " + containingEntityIn.get.get_name,
                              "Move it to the containing group: " + new Group(callingMenusRtgIn.get.db, callingMenusRtgIn.get.getGroupId).get_name))
            if answer.isEmpty) {
              (None, None)
            } else if answer.get == 1) {
              (None, containingEntityIn)
            } else if answer.get == 2) {
              (Some(callingMenusRtgIn.get.getGroupId), None)
            }
          } else if callingMenusRtgIn.is_defined) {
            (Some(callingMenusRtgIn.get.getGroupId), None)
          } else if containingEntityIn.is_defined) {
            (None, containingEntityIn)
          } else {
            // none provided, so see if it's guessable
            // (Idea: if useful, could also try guessing the entity if there's just one, then if both are there let user choose which as just above.
            // And if > 1 of either or both groups/entities, ask to which of them to move it?)
            let containingGroupsIds: Vec<Vec<Option<DataType>>> = group_in.get_groups_containing_entitys_groups_ids();
            if containingGroupsIds.isEmpty) {
              ui.display_text("Unable to find any containing groups, for the group \"" + group_in.get_name + "\" (ie, nowhere \"up\" found, to move it to).")
              (None, None)
            } else if containingGroupsIds.size == 1) {
              (Some(containingGroupsIds.head(0).get.asInstanceOf[i64]), None)
            } else {
              ui.display_text("There are more than one containing groups, for the group \"" + group_in.get_name + "\".  You could, from an Entity Menu, " +
                             "choose the option to 'Go to...' and explore what contains it, to see if you want to make changes to the organization.  Might " +
                             "need a feature to choose a containing group as the target for moving an entity...?")
              (None, None)
            }
          }
        }
        if targetEntity.is_defined) {
          require(targetGroupId.isEmpty)
          group_in.move_entity_from_group_to_local_entity(containingEntityIn.get.get_id, highlightedObjId, get_sorting_index(group_in.db, group_in.get_id, -1, highlightedObjId))
          let entityToHighlight: Option<Entity> = Util.find_entity_to_highlight_next(obj_ids.length, objectsToDisplay, removedOneIn = true,;
                                                                                       highlightedIndexInObjListIn, highlightedEntry)
          quickGroupMenu(group_in, starting_display_row_index_in, relationToGroupIn, entityToHighlight, targetForMovesIn, callingMenusRtgIn, containingEntityIn)
        } else if targetGroupId.is_defined) {
          require(targetEntity.isEmpty)
          group_in.moveEntityToDifferentGroup(targetGroupId.get, highlightedObjId, get_sorting_index(group_in.db, group_in.get_id, -1, highlightedObjId))
          let entityToHighlight: Option<Entity> = Util.find_entity_to_highlight_next(obj_ids.length, objectsToDisplay, removedOneIn = true,;
                                                                                      highlightedIndexInObjListIn, highlightedEntry)
          quickGroupMenu(group_in, starting_display_row_index_in, relationToGroupIn, entityToHighlight, targetForMovesIn, callingMenusRtgIn, containingEntityIn)
        } else {
          quickGroupMenu(group_in, starting_display_row_index_in, relationToGroupIn, Some(highlightedEntry), targetForMovesIn, callingMenusRtgIn, containingEntityIn)
        }
      } else {
        quickGroupMenu(group_in, starting_display_row_index_in, relationToGroupIn, Some(highlightedEntry), targetForMovesIn, callingMenusRtgIn, containingEntityIn)
      }
    }
  }

  /** The parameter relationToGroupIn is nice when available, optional otherwise, and represents the relation via which we got to this group.
    *
    * */
    fn quickGroupMenu_doTheWork(group_in: Group, starting_display_row_index_in: Int, relationToGroupIn: Option[RelationToGroup],
                               highlightedEntityIn: Option<Entity> = None, targetForMovesIn: Option<Entity> = None,
                               callingMenusRtgIn: Option[RelationToGroup] = None, containingEntityIn: Option<Entity>) -> Option<Entity> {
    require(group_in != null)
    let choices = Vec<String>("Create new entry quickly",;
                                "Move selection (*) up/down, in, out... (choose this then ESC to re-center from current selection, maybe)",
                                "Edit the selected entry's name",
                                "Create new entry...",
                                "Go to selected entity (not the subgroup)",
                                "List next items...",
                                "Select target (entry move destination: gets a '+')",
                                "Select entry to highlight (with '*'; typing the letter instead goes to the subgroup if any, else to that entity)",
                                "Other (slower actions, more complete menu)")
    let displayDescription = if relationToGroupIn.is_defined) relationToGroupIn.get.get_display_string(0) else group_in.get_display_string();
    // (idea: maybe this use of color on next line could be removed, if people don't rely on the color change.  I originally added it as a visual
    // cue to aid my transition to using entities more & groups less.  Same thing is done in GroupMenu.)
    // (Idea: this color thing should probably be handled in the textui class instead, especially if there were multiple kinds of UI.)
    let leading_text: Vec<String> = Array(Color.yellow("ENTITY GROUP") + " (quick menu: acts on (w/ #'s) OR selects (w/ letters...) an entity): ";
                                           + displayDescription)
    let numDisplayableItems = ui.maxColumnarChoicesToDisplayAfter(leading_text.length, choices.length, Util.maxNameLength);
    let objectsToDisplay: Vec<Entity> = group_in.getGroupEntries(starting_display_row_index_in, Some(numDisplayableItems));
    let obj_ids = for (entity: Entity <- objectsToDisplay.toArray(Array[Entity]())) yield {;
      entity.get_id
    }
    Util.add_remaining_count_to_prompt(choices, objectsToDisplay.size, group_in.get_size(4), starting_display_row_index_in)
    let statusesAndNames: Vec<String> = for (entity: Entity <- objectsToDisplay.toArray(Array[Entity]())) yield {;
      let numSubgroupsPrefix: String = controller.getEntityContentSizePrefix(entity);
      let archivedStatus = entity.getArchivedStatusDisplayString;
      archivedStatus + numSubgroupsPrefix + entity.get_name + " " + controller.getPublicStatusDisplayString(entity)
    }
    if obj_ids.length == 0) {
      let response = ui.ask_which(Some(leading_text), Vec<String>("Add entry", "Other (slower, more complete menu)"), Vec<String>(),;
                                 highlightIndexIn = None)
      if response.isEmpty) None
      else {
        let answer = response.get;
        if answer == 1) {
          controller.addEntityToGroup(group_in)
          quickGroupMenu(group_in, 0, relationToGroupIn, callingMenusRtgIn = callingMenusRtgIn, containingEntityIn = containingEntityIn)
        } else if answer == 2 && answer <= choices.length) {
          new GroupMenu(ui, controller).groupMenu(group_in, starting_display_row_index_in, relationToGroupIn, callingMenusRtgIn, containingEntityIn)
        } else if answer == 0) None
        else {
          // expected to be unreachable based on ask_which behavior (doesn't allow answers beyond the list of choices available), but for the compiler:
          None
        }
      }
    } else {
      // Idea: improve wherever needed, to remove bad smells, especially the named types used here & in related code.
      let (highlightedIndexInObjList: Int, highlightedObjId: i64, highlightedEntry: Entity, moveTargetIndexInObjList: Option[Int],;
      targetForMoves: Option<Entity>) = {
        // Be sure the code is OK even if the highlightedEntityIn isn't really in the list due to caller logic error, etc.
        let mut highlightedObjId: i64 = if highlightedEntityIn.isEmpty) obj_ids(0) else highlightedEntityIn.get.get_id;
        let mut highlightedIndexInObjList: i32 = {;
          let index = obj_ids.indexOf(highlightedObjId);
          // if index == -1 then there could be a logic error where an entity not in the list was passed in, or an entry was moved and we're not displaying
          // the portion of the list containing that entry.  Regardless, don't fail (ie, don't throw AIOOBE) due to the -1 later, just make it None.
          if index < 0) {
            highlightedObjId = obj_ids(0)
            0
          }
          else index
        }
        let mut moveTargetIndexInObjList: Option[Int] = if targetForMovesIn.isEmpty) None;
                                                    else {
                                                      let index = obj_ids.indexOf(targetForMovesIn.get.get_id);
                                                      // same as just above: don't bomb w/ a -1
                                                      if index < 0) None
                                                      else Some(index)
                                                    }
        if moveTargetIndexInObjList.is_defined && highlightedIndexInObjList == moveTargetIndexInObjList.get) {
          // doesn't make sense if they're equal (ie move both, into both?, like if user changed the previous highlight on 1st selection to a move
          // target), so change one:
          if highlightedIndexInObjList == 0 && obj_ids.length > 1) {
            highlightedIndexInObjList = 1
          } else {
            moveTargetIndexInObjList = None
          }
        }
        assert(highlightedIndexInObjList >= 0)
        let highlightedEntry: Entity = objectsToDisplay.get(highlightedIndexInObjList);
        highlightedObjId = highlightedEntry.get_id
        let targetForMoves: Option<Entity> = if moveTargetIndexInObjList.isEmpty) None;
                                             else Some(objectsToDisplay.get(moveTargetIndexInObjList.get))
        (highlightedIndexInObjList, highlightedObjId, highlightedEntry, moveTargetIndexInObjList, targetForMoves)
      }

      if highlightedIndexInObjList == moveTargetIndexInObjList.getOrElse(None)) {
        throw new OmException("We have wound up with the same entry for targetForMoves and highlightedEntry: that will be a problem: aborting before we put" +
                              " an entity in its own group and lose track of it or something.")
      }


      let response = ui.ask_which(Some(leading_text), choices, statusesAndNames, highlightIndexIn = Some(highlightedIndexInObjList),;
                                 secondaryHighlightIndexIn = moveTargetIndexInObjList)
      if response.isEmpty) None
      else {
        let answer = response.get;
        if answer == 1) {
          let (entryToHighlight:Option<Entity>, displayStartingRowNumber: Int) = {;
            // ask for less info when here in the quick menu, where want to add entity quickly w/ no fuss, like brainstorming.  User can always use long menu.
            let ans: Option<Entity> = controller.askForNameAndWriteEntity(group_in.db, Util.ENTITY_TYPE, leading_text_in = Some("NAME THE ENTITY:"),;
                                                               classIdIn = group_in.getClassId)
            if ans.is_defined) {
              let newEntity = ans.get;
              let new_entity_id: i64 = newEntity.get_id;
              group_in.addEntity(new_entity_id)
              // (See comment at similar place in EntityMenu, just before that call to placeEntryInPosition.)
              let goingBackward: bool = highlightedIndexInObjList == 0 && group_in.getNewEntriesStickToTop;
              let forwardNotBack = !goingBackward;
              let displayStartingRowNumber: i32 = placeEntryInPosition(group_in.db, group_in.get_id, group_in.get_size(4), 0,;
                                                                       forward_not_back_in = forwardNotBack, starting_display_row_index_in, new_entity_id,
                                                                       highlightedIndexInObjList, Some(highlightedObjId), objectsToDisplay.size, -1, Some(-1))
              controller.defaultAttributeCopying(newEntity)
              (Some(new Entity(group_in.db, new_entity_id)), displayStartingRowNumber)
            }
            else (Some(highlightedEntry), starting_display_row_index_in)
          }
          quickGroupMenu(group_in, displayStartingRowNumber, relationToGroupIn, entryToHighlight, targetForMoves, callingMenusRtgIn, containingEntityIn)
        } else if answer == 2) {
          moveSelectedEntry(group_in, starting_display_row_index_in, relationToGroupIn, targetForMoves, highlightedIndexInObjList, moveTargetIndexInObjList,
                            highlightedEntry, highlightedObjId, obj_ids, objectsToDisplay, callingMenusRtgIn, containingEntityIn)
        } else if answer == 3) {
          let editedEntity: Option<Entity> = controller.editEntityName(highlightedEntry);
          if editedEntity.isEmpty)
            quickGroupMenu(group_in, starting_display_row_index_in, relationToGroupIn, Some(highlightedEntry), targetForMoves, callingMenusRtgIn, containingEntityIn)
          else {
            quickGroupMenu(group_in, starting_display_row_index_in, relationToGroupIn, editedEntity, targetForMoves, callingMenusRtgIn, containingEntityIn)
          }
        } else if answer == 4) {
          //(the first is the same as if user goes to the selection & presses '1', but is here so there can
          // be a similar #2 for consistency/memorability with the EntityMenu.)
          let choices = Vec<String>("Create new entry INSIDE selected entry",;
                                      "Add entry from existing (quick search by name; uses \"has\" relation)")
          let response = ui.ask_which(None, choices, new Vec<String>(0));
          if response.is_defined) {
            let addEntryAnswer = response.get;
            if addEntryAnswer == 1) {
              let (targetRtgCount: i64, defaultToUsingSubgroup: Option<bool>) = useSubgroup(highlightedEntry);
              if defaultToUsingSubgroup.is_defined) {
                if defaultToUsingSubgroup.get) {
                  if targetRtgCount > 1) {
                    // IDEA: (see idea at similar logic above where entry is moved into a targeted group, about guessing which one)
                    ui.display_text("For this operation, the selection must have exactly one subgroup (a single '>'), or none.")
                    quickGroupMenu(group_in, starting_display_row_index_in, relationToGroupIn, Some(highlightedEntry), targetForMovesIn, callingMenusRtgIn,
                                   containingEntityIn)
                  } else {
                    let (rtgId: i64, rel_type_id: i64, targetGroupId: i64) = createNewOrFindOneGroupOnEntity(group_in, targetRtgCount, highlightedEntry);
                    // about the sortingIndex:  see comment on db.moveEntityToNewGroup.
                    let ans: Option<Entity> = controller.askForNameAndWriteEntity(group_in.db, Util.ENTITY_TYPE, leading_text_in = Some("NAME THE ENTITY:"),;
                                                                                  classIdIn = group_in.getClassId)
                    if ans.is_defined) {
                      let new_entity_id: i64 = ans.get.get_id;
                      let newEntity: Entity = ans.get;
                      let targetGroup = new Group(group_in.db, targetGroupId);
                      targetGroup.addEntity(new_entity_id)

                      controller.defaultAttributeCopying(newEntity)

                      let newRtg: RelationToGroup = new RelationToGroup(group_in.db, rtgId, highlightedEntry.get_id, rel_type_id, targetGroup.get_id);
                      quickGroupMenu(new Group(targetGroup.db, targetGroup.get_id), 0, Some(newRtg), None, None, containingEntityIn = Some(highlightedEntry))
                    }
                    quickGroupMenu(group_in, starting_display_row_index_in, relationToGroupIn, Some(highlightedEntry), targetForMoves, callingMenusRtgIn, containingEntityIn)
                  }
                } else {
                  let newEntity: Option<Entity> = controller.askForNameAndWriteEntity(group_in.db, Util.ENTITY_TYPE, leading_text_in = Some("NAME THE ENTITY:"),;
                                                                                      classIdIn = group_in.getClassId)
                  if newEntity.is_defined) {
                    let new_entity_id: i64 = newEntity.get.get_id;
                    let new_rte: RelationToLocalEntity = highlightedEntry.add_has_relation_to_local_entity(new_entity_id, None, System.currentTimeMillis());
                    require(new_rte.get_parent_id() == highlightedEntry.get_id)
                    new EntityMenu(ui, controller).entityMenu(newEntity.get, containingRelationToEntityIn = Some(new_rte))
                  }
                  quickGroupMenu(group_in, starting_display_row_index_in, relationToGroupIn, newEntity, targetForMoves, callingMenusRtgIn, containingEntityIn)
                }
              } else {
                quickGroupMenu(group_in, starting_display_row_index_in, relationToGroupIn, Some(highlightedEntry), targetForMoves, callingMenusRtgIn, containingEntityIn)
              }
            } else if addEntryAnswer == 2) {
              let entityChosen: Option[IdWrapper] = controller.askForNameAndSearchForEntity(group_in.db);
              let (entryToHighlight:Option<Entity>, displayStartingRowNumber: Int) = {;
                if entityChosen.is_defined) {
                  let entityChosenId: i64 = entityChosen.get.get_id;
                  group_in.addEntity(entityChosenId)
                  // (See comment at similar place in EntityMenu, just before that call to placeEntryInPosition.)
                  let goingBackward: bool = highlightedIndexInObjList == 0 && group_in.getNewEntriesStickToTop;
                  let forward = !goingBackward;
                  let newDisplayStartingRowNumber: i32 = placeEntryInPosition(group_in.db, group_in.get_id, group_in.get_size(4), 0, forward_not_back_in = forward,;
                                                                              starting_display_row_index_in, entityChosenId, highlightedIndexInObjList,
                                                                              Some(highlightedObjId), objectsToDisplay.size, -1, Some(-1))
                  (Some(new Entity(group_in.db, entityChosenId)), newDisplayStartingRowNumber)
                } else (Some(highlightedEntry), starting_display_row_index_in)
              }
              quickGroupMenu(group_in, displayStartingRowNumber, relationToGroupIn, entryToHighlight, targetForMoves, callingMenusRtgIn, containingEntityIn)
            } else {
              ui.display_text("unexpected selection")
              quickGroupMenu(group_in, starting_display_row_index_in, relationToGroupIn, Some(highlightedEntry), targetForMoves, callingMenusRtgIn, containingEntityIn)
            }
          } else {
            quickGroupMenu(group_in, starting_display_row_index_in, relationToGroupIn, Some(highlightedEntry), targetForMoves, callingMenusRtgIn, containingEntityIn)
          }
        } else if answer == 5) {
          new EntityMenu(ui, controller).entityMenu(highlightedEntry, containingGroupIn = Some(group_in))
          // deal with entityMenu possibly having deleted the entity:
          let removedOne: bool = !group_in.is_entity_in_group(highlightedEntry.get_id);
          let entityToHighlightNext: Option<Entity> = Util.find_entity_to_highlight_next(obj_ids.length, objectsToDisplay, removedOne, highlightedIndexInObjList,;
                                                                               highlightedEntry)
          quickGroupMenu(group_in, starting_display_row_index_in, relationToGroupIn, entityToHighlightNext, targetForMoves, callingMenusRtgIn, containingEntityIn)
        } else if answer == 6) {
          let (entryToHighlight: Option<Entity>, displayStartingRowNumber: Int) = {;
            let nextStartPosition = starting_display_row_index_in + objectsToDisplay.size;
            if nextStartPosition >= group_in.get_size(4)) {
              ui.display_text("End of attribute list found; restarting from the beginning.")
              (None, 0) // start over
            } else (highlightedEntityIn, nextStartPosition)
          }
          quickGroupMenu(group_in, displayStartingRowNumber, relationToGroupIn, entryToHighlight, targetForMoves, callingMenusRtgIn, containingEntityIn)
        } else if answer == 7) {
          // NOTE: this code is similar (not identical) in EntityMenu as in QuickGroupMenu: if one changes,
          // THE OTHER MIGHT ALSO NEED MAINTENANCE!
          let choices = Vec<String>(Util.UNSELECT_MOVE_TARGET_PROMPT_TEXT);
          let leading_text: Vec<String> = Array(Util.UNSELECT_MOVE_TARGET_LEADING_TEXT);
          Util.add_remaining_count_to_prompt(choices, objectsToDisplay.size, group_in.get_size(4), starting_display_row_index_in)

          let response = ui.ask_which(Some(leading_text), choices, statusesAndNames, highlightIndexIn = Some(highlightedIndexInObjList),;
                                     secondaryHighlightIndexIn = moveTargetIndexInObjList)
          let (entryToHighlight, selectedTargetEntity): (Option<Entity>, Option<Entity>) =;
            if response.isEmpty) (Some(highlightedEntry), targetForMoves)
            else {
              let answer = response.get;
              if answer == 1) {
                (Some(highlightedEntry), None)
              } else {
                // those in the condition are 1-based, not 0-based.
                // user typed a letter to select an attribute (now 0-based):
                let choices_index = answer - choices.length - 1;
                let userSelection: Entity = objectsToDisplay.get(choices_index);
                if choices_index == highlightedIndexInObjList) {
                  // chose same entity for the target, as the existing highlighted selection, so make it the target, and no highlighted one.
                  (None, Some(userSelection))
                } else {
                  (Some(highlightedEntry), Some(userSelection))
                }
              }
            }
          quickGroupMenu(group_in, starting_display_row_index_in, relationToGroupIn, entryToHighlight, selectedTargetEntity, callingMenusRtgIn, containingEntityIn)
        } else if answer == 8) {
          // lets user select a new entity or group for further operations like moving, deleting.
          // (we have to have at least one choice or ui.ask_which fails...a require() call there.)
          // NOTE: this code is similar (not identical) in EntityMenu as in QuickGroupMenu: if one changes,
          // THE OTHER MIGHT ALSO NEED MAINTENANCE!
          let choices = Vec<String>("keep existing (same as ESC)");
          // says 'same screenful' because (see similar cmt elsewhere).
          let leading_text: Vec<String> = Array("CHOOSE AN ENTRY to highlight (*)");
          Util.add_remaining_count_to_prompt(choices, objectsToDisplay.size, group_in.get_size(4), starting_display_row_index_in)
          let response = ui.ask_which(Some(leading_text), choices, statusesAndNames, highlightIndexIn = Some(highlightedIndexInObjList),;
                                     secondaryHighlightIndexIn = moveTargetIndexInObjList)
          let (entityToHighlight, selectedTargetEntity): (Option<Entity>, Option<Entity>) =;
            if response.isEmpty || response.get == 1) (Some(highlightedEntry), targetForMoves)
            else {
              // those in the condition are 1-based, not 0-based.
              // user typed a letter to select an attribute (now 0-based):
              let choices_index = response.get - choices.length - 1;
              let userSelection: Entity = objectsToDisplay.get(choices_index);
              if choices_index == moveTargetIndexInObjList.getOrElse(None)) {
                // chose same entity for the target, as the existing highlighted selection, so make it the target, and no highlighted one.
                (Some(userSelection), None)
              } else {
                (Some(userSelection), targetForMoves)
              }
            }
          quickGroupMenu(group_in, starting_display_row_index_in, relationToGroupIn, entityToHighlight, selectedTargetEntity, callingMenusRtgIn, containingEntityIn)
        } else if answer == 9 && answer <= choices.length) {
          new GroupMenu(ui, controller).groupMenu(group_in, starting_display_row_index_in, relationToGroupIn, callingMenusRtgIn, containingEntityIn)
        } else if false /*can this be changed so that if they hit Enter it makes it to here ?*/ ) {
          // do something with enter: do a quick text edit & update the dates. Or quickAddEntry ?
          ui.display_text("not yet implemented")
          quickGroupMenu(group_in, starting_display_row_index_in, relationToGroupIn, Some(highlightedEntry), targetForMoves, callingMenusRtgIn, containingEntityIn)
        } else if answer == 0) None
        else if answer > choices.length && answer <= (choices.length + objectsToDisplay.size)) {
          // those in the condition are 1-based, not 0-based.
          // lets user go to an entity or group quickly (1 stroke)
          let choices_index = answer - choices.length - 1;
          // user typed a letter to select an attribute (now 0-based)
          if choices_index >= objectsToDisplay.size()) {
            ui.display_text("The program shouldn't have let us get to this point, but the selection " + answer + " is not in the list.")
            quickGroupMenu(group_in, starting_display_row_index_in, relationToGroupIn, Some(highlightedEntry), targetForMoves, callingMenusRtgIn, containingEntityIn)
          } else {
            let userSelection: Entity = objectsToDisplay.get(choices_index);

            let (_ /*subEntitySelected:Option<Entity>*/ , groupId: Option<i64>, moreThanOneGroupAvailable) =;
              controller.goToEntityOrItsSoleGroupsMenu(userSelection, relationToGroupIn, Some(group_in))

            let removedOne: bool = !group_in.is_entity_in_group(userSelection.get_id);
            let mut entityToHighlightNext: Option<Entity> = Some(userSelection);
            if groupId.is_defined && !moreThanOneGroupAvailable) {
              //idea: do something w/ this unused variable? Like, if the userSelection was deleted, then use this in its place in parms to
              // qGM just below? or what was it for originally?  Or, del this let mut around here?;
              entityToHighlightNext = Util.find_entity_to_highlight_next(obj_ids.length, objectsToDisplay, removedOne, highlightedIndexInObjList, highlightedEntry)
            }

            //ck 1st if it exists, if not return None. It could have been deleted while navigating around.
            if group_in.db.group_key_exists(group_in.get_id)) {
              if choices_index == moveTargetIndexInObjList.getOrElse(None)) {
                quickGroupMenu(group_in, starting_display_row_index_in, relationToGroupIn, Some(userSelection), None, callingMenusRtgIn, containingEntityIn)
              } else {
                quickGroupMenu(group_in, starting_display_row_index_in, relationToGroupIn, Some(userSelection), targetForMoves, callingMenusRtgIn, containingEntityIn)
              }
            } else None
          }
        } else {
          ui.display_text("invalid selection")
          quickGroupMenu(group_in, starting_display_row_index_in, relationToGroupIn, Some(highlightedEntry), targetForMoves, callingMenusRtgIn, containingEntityIn)
        }
      }
    }
  }

    fn useSubgroup(targetEntry: Entity) -> (i64, Option<bool>) {
    let targetRtgCount: i64 = targetEntry.get_relation_to_group_count;
    let defaultToUsingSubgroup: Option<bool> = {;
      if targetRtgCount == 0) {
        Some(false)
      } else if targetRtgCount == 1) {
        Some(true)
      } else {
        ui.display_text("There are multiple subgroups on this entity, so for now OM will just move the one entry to be contained by the" +
                       " other entity, to which you can then manually go and move it further to a subgroup as needed.")
        Some(false)
      }
    }
    (targetRtgCount, defaultToUsingSubgroup)
  }

  protected fn getAdjacentEntriesSortingIndexes(dbIn: Database, groupIdIn: i64, movingFromPosition_sortingIndexIn: i64, queryLimitIn: Option<i64>,
                                        forward_not_back_in: bool) -> Vec<Vec<Option<DataType>>> {
    let group = new Group(dbIn, groupIdIn);
    group.get_adjacent_group_entries_sorting_indexes(movingFromPosition_sortingIndexIn, queryLimitIn, forward_not_back_in)
  }

  protected fn get_sorting_indexOfNearestEntry(dbIn: Database, groupIdIn: i64, starting_point_sorting_index_in: i64, forward_not_back_in: bool) -> Option<i64> {
    let group = new Group(dbIn, groupIdIn);
    group.get_nearest_group_entrys_sorting_index(starting_point_sorting_index_in, forward_not_back_in = forward_not_back_in)
  }

  protected fn renumber_sorting_indexes(dbIn: Database, groupIdIn: i64) /* -> Unit%%*/ {
    let group = new Group(dbIn, groupIdIn);
    group.renumber_sorting_indexes()
  }

  protected fn updateSortedEntry(dbIn: Database, groupIdIn: i64, ignoredParameter: Int, movingEntityIdIn: i64, sortingIndexIn: i64) /* -> Unit%%*/ {
    let group = new Group(dbIn, groupIdIn);
    group.updateSortingIndex(movingEntityIdIn, sortingIndexIn)
  }

  protected fn get_sorting_index(dbIn: Database, groupIdIn: i64, ignoredParameter: Int, entity_idIn: i64) -> i64 {
    let group = new Group(dbIn, groupIdIn);
    group.getEntrySortingIndex(entity_idIn)
  }

  protected fn indexIsInUse(dbIn: Database, groupIdIn: i64, sortingIndexIn: i64) -> bool {
    let group = new Group(dbIn, groupIdIn);
    group.isGroupEntrySortingIndexInUse(sortingIndexIn)
  }

  protected fn findUnusedSortingIndex(dbIn: Database, groupIdIn: i64, startingWithIn: i64) -> i64 {
    let group = new Group(dbIn, groupIdIn);
    group.findUnusedSortingIndex(Some(startingWithIn))
  }

*/
}
