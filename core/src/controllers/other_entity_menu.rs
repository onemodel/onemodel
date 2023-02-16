/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2015-2020 inclusive and 2023, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
struct OtherEntityMenu {
    /*%%
  package org.onemodel.core.controllers

    import java.util

    import org.onemodel.core._
    import org.onemodel.core.{OmException, TextUI}
    import org.onemodel.core.model._

    import scala.collection.mutable.ArrayBuffer
    import scala.collection.JavaConversions._

    * This is simply to hold less-used operations so the main EntityMenu can be the most-used stuff.
      *
    class OtherEntityMenu (val ui: TextUI, let controller: Controller) {;

        fn otherEntityMenu(entity_in: Entity, attributeRowsStartingIndexIn: Int = 0, relationSourceEntityIn: Option<Entity>,
                          containingRelationToEntityIn: Option[AttributeWithValidAndObservedDates], containingGroupIn: Option[Group],
                          attributeTuplesIn: Array[(i64, Attribute)]) {
        require(containingRelationToEntityIn.isEmpty ||
                containingRelationToEntityIn.get.isInstanceOf[RelationToLocalEntity] || containingRelationToEntityIn.get.isInstanceOf[RelationToRemoteEntity])
        try {
          require(entity_in != null)
          let leading_text = Vec<String>{ Util.entity_menu_leading_text(entity_in) };
          let mut choices = Vec<String>("Edit public/nonpublic status",;
                                      "Import/Export...",
                                      "Edit...",
                                      "Delete or " + (if entity_in.is_archived) "Un-archive" else "Archive") + " this entity (or link)...",
                                      "Go to other related entities or groups...",
                                      "(stub)")
          //  don't show the "set default" option if it's already been done w/ this same one:
          let defaultEntityTuple: Option[(i64, Entity)] = controller.get_default_entity;
          let defaultEntity: Option<i64> = if defaultEntityTuple.isEmpty) None else Some(defaultEntityTuple.get._1);
          let entityIsAlreadyTheDefault: bool = defaultEntity.is_defined && defaultEntity.get == entity_in.get_id;
          if ! entityIsAlreadyTheDefault) {
            choices = choices :+ ((if defaultEntity.isEmpty && !entity_in.mDB.is_remote) "****TRY ME---> " else "") +
                                  "Set current entity as default (first to come up when launching this program.)")
          } else choices = choices :+ "(stub)"

          let response = ui.ask_which(Some(leading_text), choices);
          if response.is_defined) {
            let answer = response.get;
            if answer == 1) {
              let valueBeforeEntry: Option<bool> = entity_in.getPublic;
              let valueAfterEntry: Option<bool> = controller.askForPublicNonpublicStatus(valueBeforeEntry);
              let rteCount: i64 = entity_in.getRelationToLocalEntityCount(include_archived_entitiesIn = false);
              let rtgCount: i64 = entity_in.getRelationToGroupCount;
              let whichToUpdateChoices = {;
                if rteCount > 0) {
                  Array("...for this entity (\"" + entity_in.get_name + "\")",
                        "...for its " + rteCount + " contained entities (one level, local), and all the" +
                        " entities contained in its " + rtgCount + " groups (one level)",
                        "...for both.")
                } else {
                  Array("...for this entity only (\"" + entity_in.get_name + "\").")
                }
              }
              let publicMenuResponse = ui.ask_which(Some(Vec<String>{"Confirm:"}), whichToUpdateChoices);
              if publicMenuResponse.is_defined) {
                if publicMenuResponse.get == 1) {
                  entity_in.updatePublicStatus(valueAfterEntry)
                } else if publicMenuResponse.get == 2) {
                  let count: i32 = entity_in.updateContainedEntitiesPublicStatus(valueAfterEntry);
                  ui.display_text("Updated " + count + " contained entities with new status.")
                } else if publicMenuResponse.get == 3) {
                  entity_in.updatePublicStatus(valueAfterEntry)
                  let count: i32 = entity_in.updateContainedEntitiesPublicStatus(valueAfterEntry);
                  ui.display_text("Updated this entity and " + count + " contained entities with new status.")
                } else {
                  ui.display_text("invalid response")
                }
              }
            } else if answer == 2) {
              let importOrExportAnswer = ui.ask_which(Some(Array("NOTE: this is very useful for getting things in & out of OM, but is not" +;
                                                                " complete or tested enough" +
                                                                " to use for OM backup/restore.  (That has to be done at the database level.  Try the mailing" +
                                                                " list for help with that.  If it is a hosted OM solution the backups should be done for you.)")),
                                                     Array("Import", "Export to a text file (outline)", "Export to html pages"), Vec<String>())
              if importOrExportAnswer.is_defined) {
                if importOrExportAnswer.get == 1) {
                  new ImportExport(ui, controller).importCollapsibleOutlineAsGroups(entity_in)
                }
                else if importOrExportAnswer.get == 2) {
                  new ImportExport(ui, controller).export(entity_in, ImportExport.TEXT_EXPORT_TYPE, None, None, None)
                }
                else if importOrExportAnswer.get == 3) {
                  let (headerContent: String, beginBodyContent: String, footerContent: Option<String>) = getOptionalContentForExportedPages(entity_in);
                  if footerContent.is_defined && footerContent.get.trim.nonEmpty) {
                    new ImportExport(ui, controller).export(entity_in, ImportExport.HTML_EXPORT_TYPE, Some(headerContent), Some(beginBodyContent), footerContent)
                  }
                }
              }
              otherEntityMenu(entity_in, attributeRowsStartingIndexIn, relationSourceEntityIn, containingRelationToEntityIn, containingGroupIn,
                              attributeTuplesIn)
            } else if answer == 3) {
              let templateEntity: Option<Entity> =;
                if entity_in.getClassTemplateEntityId.isEmpty) {
                  None
                } else {
                  Some(new Entity(entity_in.mDB, entity_in.getClassTemplateEntityId.get))
                }
              let templateAttributesToCopy: ArrayBuffer[Attribute] = controller.getMissingAttributes(templateEntity, attributeTuplesIn);
              let editAnswer = ui.ask_which(Some(Vec<String>{Util.entity_menu_leading_text(entity_in)}),;
                                           Array("Edit entity name",
                                                 "Change its class",
                                                 if templateAttributesToCopy.nonEmpty) "Add/edit missing class-defined fields (in other words, to make this " +
                                                                                        "entity more resemble its class' template)" else "(stub)",
                                                 if entity_in.getNewEntriesStickToTop) {
                                                   "Set entity so new items added from the top highlight become the *2nd* entry (CURRENTLY: they stay at the top)."
                                                 } else {
                                                   "Set entity so new items added from the top highlight become the *top* entry (CURRENTLY: they will be 2nd)."
                                                 }))
              if editAnswer.is_defined) {
                if editAnswer.get == 1) {
                  let editedEntity: Option<Entity> = controller.editEntityName(entity_in);
                  otherEntityMenu(if editedEntity.is_defined) editedEntity.get else entity_in, attributeRowsStartingIndexIn, relationSourceEntityIn,
                                  containingRelationToEntityIn, containingGroupIn, attributeTuplesIn)
                } else if editAnswer.get == 2) {
                  let classId: Option<i64> = controller.askForClass(entity_in.mDB);
                  if classId.is_defined) {
                    entity_in.updateClass(classId)

                    // Idea here: when changing the class of an entity, we *could* Controller.defaultAttributeCopying (or prompt as elsewhere) to
                    // set up the attributes, but the need is unclear, and user can now do that manually from the menus if needed.  Code in future
                    // should also be able to use default values from the template entity, as another fallback.
                  }
                  otherEntityMenu(entity_in, attributeRowsStartingIndexIn, relationSourceEntityIn, containingRelationToEntityIn,
                                  containingGroupIn, attributeTuplesIn)
                } else if editAnswer.get == 3 && templateAttributesToCopy.nonEmpty) {
                  controller.copyAndEditAttributes(entity_in, templateAttributesToCopy)
                } else if editAnswer.get == 4) {
                  entity_in.updateNewEntriesStickToTop(!entity_in.getNewEntriesStickToTop)
                }
              }
            } else if answer == 4) {
              let (delOrArchiveAnswer, delEntityLink_choiceNumber, delFromContainingGroup_choiceNumber, showAllArchivedEntities_choiceNumber) =;
                askWhetherDeleteOrArchiveEtc(entity_in, containingRelationToEntityIn, relationSourceEntityIn, containingGroupIn)

              if delOrArchiveAnswer.is_defined) {
                let delAnswer = delOrArchiveAnswer.get;
                if delAnswer == 1) {
                  deleteEntity(entity_in)
                } else if delAnswer == 2) {
                  if !entity_in.is_archived) {
                    archiveEntity(entity_in)
                  } else {
                    // ** IF THIS menu OPERATION IS EVER MOVED, UPDATE THE USER MESSAGE ABOUT THE MENU OPTIONS LOCATIONS**, in Controller.get_default_entity. **
                    unarchiveEntity(entity_in)
                  }
                } else if delAnswer == delEntityLink_choiceNumber && containingRelationToEntityIn.is_defined && delAnswer <= choices.length) {
                  let ans = ui.ask_yes_no_question("DELETE the relation: ARE YOU SURE?", Some(""));
                  if ans.is_defined && ans.get) {
                    containingRelationToEntityIn.get.delete()
                  } else {
                    ui.display_text("Did not delete relation.", false);
                  }
                } else if delAnswer == delFromContainingGroup_choiceNumber && containingGroupIn.is_defined && delAnswer <= choices.length) {
                  removeEntityReferenceFromGroup_Menu(entity_in, containingGroupIn)
                } else if delAnswer == showAllArchivedEntities_choiceNumber) {
                  // ** IF THIS OPERATION IS EVER MOVED, UPDATE THE USER MESSAGE ABOUT THE MENU OPTIONS LOCATIONS**, in Controller.get_default_entity. **
                  entity_in.mDB.set_include_archived_entities(! entity_in.mDB.include_archived_entities)
                } else {
                  ui.display_text("invalid response")
                  otherEntityMenu(new Entity(entity_in.mDB, entity_in.get_id), attributeRowsStartingIndexIn, relationSourceEntityIn, containingRelationToEntityIn,
                                  containingGroupIn, attributeTuplesIn)
                }
              }
            } else if answer == 5) {
              let templateEntityId: Option<i64> = entity_in.getClassTemplateEntityId;
              goToRelatedPlaces(entity_in, relationSourceEntityIn, containingRelationToEntityIn, templateEntityId)
              //ck 1st if entity exists, if not return None. It could have been deleted while navigating around.
              if entity_in.mDB.entity_key_exists(entity_in.get_id, include_archived = false)) {
                new EntityMenu(ui, controller).entityMenu(entity_in, attributeRowsStartingIndexIn, None, None, containingRelationToEntityIn, containingGroupIn)
              }
            } else if answer == 7 && answer <= choices.length && !entityIsAlreadyTheDefault && !entity_in.mDB.is_remote) {
              // updates user preferences such that this obj will be the one displayed by default in future.
              entity_in.mDB.setUserPreference_EntityId(Util.DEFAULT_ENTITY_PREFERENCE, entity_in.get_id)
              controller.refreshDefaultDisplayEntityId()
            } else {
              ui.display_text("invalid response")
              otherEntityMenu(entity_in, attributeRowsStartingIndexIn, relationSourceEntityIn, containingRelationToEntityIn, containingGroupIn,
                              attributeTuplesIn)
            }
          }
        } catch {
          case e: Exception =>
            Util.handleException(e, ui, entity_in.mDB)
            let ans = ui.ask_yes_no_question("Go back to what you were doing (vs. going out)?", Some("y"));
            if ans.is_defined && ans.get) {
              otherEntityMenu(entity_in, attributeRowsStartingIndexIn, relationSourceEntityIn, containingRelationToEntityIn,
                              containingGroupIn, attributeTuplesIn)
            }
        }
      }

        fn removeEntityReferenceFromGroup_Menu(entity_in: Entity, containingGroupIn: Option[Group]) -> Boolean {
        let groupCount: i64 = entity_in.getCountOfContainingGroups;
        let (entityCountNonArchived, entityCountArchived) = entity_in.getCountOfContainingLocalEntities;
        let ans = ui.ask_yes_no_question("REMOVE this entity from that group: ARE YOU SURE? (This isn't a deletion: the entity can still be found by searching, and " +;
                                      "is " + Util.get_containing_entities_description(entityCountNonArchived, entityCountArchived) +
                                      (if groupCount > 1) ", and will still be in " + (groupCount - 1) + " group(s).)" else ""),
                                      Some(""))
        if ans.is_defined && ans.get) {
          containingGroupIn.get.removeEntity(entity_in.get_id)
          true

          //is it ever desirable to keep the next line instead of the 'None'? not in most typical usage it seems, but?:
          //entityMenu(startingAttributeIndexIn, entity_in, relationSourceEntityIn, relationIn)
        } else {
          ui.display_text("Did not remove entity from that group.", false);
          false

          //is it ever desirable to keep the next line instead of the 'None'? not in most typical usage it seems, but?:
          //entityMenu(startingAttributeIndexIn, entity_in, relationSourceEntityIn, relationIn, containingGroupIn)
        }
      }

      * @return whether entity was deleted.
        *
        fn deleteEntity(entity_in: Entity) -> Boolean {
        //IDEA: could combine this method with the following two. The only differences as of now are 3 strings and a method call, easily parameterized. Not
        //doing it immediately in case they diverge again soon.
        let name = entity_in.get_name;
        let groupCount: i64 = entity_in.getCountOfContainingGroups;
        let affectedExamples = getExampleAffectedGroupsDescriptions(groupCount, entity_in);
        let effectMsg =  "This will ALSO remove it from " + groupCount + " groups, including for example these relations" +;
                         " that refer to this entity (showing entities & their relations to groups, as \"entity -> group\"): " + affectedExamples
        // idea: WHEN CONSIDERING MODS TO THIS, ALSO CONSIDER THE Q'S ASKED AT CODE CMT WHERE DELETING A GROUP OF ENTITIES (SEE, for example "recursively").
        // (and in the other 2 methods just like this)
        let warningMsg = "DELETE ENTITY \"" + name + "\" (and " + Util.ENTITY_PARTS_THAT_CAN_BE_AFFECTED + ").  " + effectMsg + "\n**ARE YOU REALLY SURE?**";
        let ans = ui.ask_yes_no_question(warningMsg, Some("n"));
        if ans.is_defined && ans.get) {
          entity_in.delete()
          ui.display_text("Deleted entity \"" + name + "\"" + ".")
          true
        } else {
          ui.display_text("Did not delete entity.", false);
          false
        }
      }

      * @return whether entity was archived.
        *
        fn archiveEntity(entity_in: Entity) -> Boolean {
        let name = entity_in.get_name;
        let groupCount: i64 = entity_in.getCountOfContainingGroups;
        let affectedExamples = getExampleAffectedGroupsDescriptions(groupCount, entity_in);
        let effectMsg = "This will affect affect its visibility in " + groupCount + " groups, including for example these relations" +;
                        " that refer to this entity (showing entities & their relations to groups, as \"entity -> group\"): " + affectedExamples
        // idea: WHEN CONSIDERING MODS TO THIS, ALSO CONSIDER THE Q'S ASKED AT CODE CMT WHERE DELETING A GROUP OF ENTITIES (SEE, for example "recursively").
        // (and in the other 2 methods just like this)
        let warningMsg = "ARCHIVE ENTITY \"" + name + "\" (and " + Util.ENTITY_PARTS_THAT_CAN_BE_AFFECTED + ").  " + effectMsg + "\n**ARE YOU REALLY SURE?**";
        let ans = ui.ask_yes_no_question(warningMsg, Some(""));
        if ans.is_defined && ans.get) {
          entity_in.archive()
          ui.display_text("Archived entity \"" + name + "\"" + ".", false);
          true
        } else {
          ui.display_text("Did not archive entity.", false);
          false
        }
      }

      * @return whether entity was un-archived.
        *
        fn unarchiveEntity(entity_in: Entity) -> Boolean {
        let name = entity_in.get_name;
        let groupCount: i64 = entity_in.getCountOfContainingGroups;
        let affectedExamples = getExampleAffectedGroupsDescriptions(groupCount, entity_in);
        let effectMsg = "This will affect affect its visibility in " + groupCount + " groups, including for example these relations" +;
                        " that refer to this entity (showing entities & their relations to groups, as \"entity -> group\"): " + affectedExamples
        // idea: WHEN CONSIDERING MODS TO THIS, ALSO CONSIDER THE Q'S ASKED AT CODE CMT WHERE DELETING A GROUP OF ENTITIES (SEE, for example "recursively").
        // (and in the other 2 methods just like this)
        let warningMsg = "un-archive entity \"" + name + "\" (and " + Util.ENTITY_PARTS_THAT_CAN_BE_AFFECTED + ").  " + effectMsg + "**ARE YOU REALLY SURE?**";
        let ans = ui.ask_yes_no_question(warningMsg, Some(""));
        if ans.is_defined && ans.get) {
          entity_in.unarchive()
          ui.display_text("Un-archived entity \"" + name + "\"" + ".", false);
          true
        } else {
          ui.display_text("Did not un-archive entity.", false);
          false
        }
      }

        fn getExampleAffectedGroupsDescriptions(groupCount: i64, entity_in: Entity) -> (String) {
        if groupCount == 0) {
          ""
        } else {
          let limit = 10;
          let delimiter = ", ";
          // (BUG: see comments in psql.java re "OTHER ENTITY NOTED IN A DELETION BUG")
          let descrArray = entity_in.getContainingRelationToGroupDescriptions(Some(limit));
          let mut descriptions = "";
          let mut counter = 0;
          for (s: String <- descrArray) {
            counter += 1
            descriptions += counter + ") " + s + delimiter
          }
          descriptions.substring(0, math.max(0, descriptions.length - delimiter.length)) + ".  "
        }
      }

        fn getOptionalContentForExportedPages(entity_in: Entity) -> (String, String, Option<String>) {
        let prompt1 = "Enter lines containing the ";
        let prompt2 = " (if any).  ";
        let prompt3 = "  (NOTE: to simplify this step in the future, you can add to this entity a single text attribute whose type is an entity named ";
        // (Wrote "lines" plural, to clarify when this is presented with the "SINGLE LINE" copyright prompt below.)
        let prompt4 = ", and put the relevant lines of html (or nothing) in the value for that attribute.  Or just press Enter to skip through this each time.)";

        let headerTypeIds: java.util.ArrayList[i64] = entity_in.mDB.findAllEntityIdsByName(Util.HEADER_CONTENT_TAG, caseSensitive = true);
        let bodyContentTypeIds: java.util.ArrayList[i64] = entity_in.mDB.findAllEntityIdsByName(Util.BODY_CONTENT_TAG, caseSensitive = true);
        let footerTypeIds: java.util.ArrayList[i64] = entity_in.mDB.findAllEntityIdsByName(Util.FOOTER_CONTENT_TAG, caseSensitive = true);
        if (headerTypeIds.size > 1) || (bodyContentTypeIds.size > 1) || (footerTypeIds.size > 1)) {
          throw new OmException("Expected at most one entity (as typeId) each, with the names " + Util.HEADER_CONTENT_TAG + ", " +
                                Util.BODY_CONTENT_TAG + ", or " + Util.FOOTER_CONTENT_TAG + ", but found respectively " +
                                headerTypeIds.size + ", " + bodyContentTypeIds.size + ", and " + headerTypeIds.size + ".  Could change" +
                                " the app to just take the first one found perhaps.... Anyway you'll need to fix in the data, that before proceeding " +
                                "with the export.")

        }

        fn getAttrText(entity_in: Entity, typeIdIn: i64) -> Option<String> {
          let attrs: java.util.ArrayList[TextAttribute] = entity_in.getTextAttributeByTypeId(typeIdIn);
          if attrs.size == 0) None
          else if attrs.size > 1) throw new OmException("The program doesn't know what to do with > 1 textAttributes with this type on the same " +
                                                           "entity, for entity " + entity_in.get_id + ", and typeId " + typeIdIn)
          else Some(attrs.get(0).getText)
        }

        // (Idea: combine the next 3 let definitions' code into one method with the "else" part as a parameter, but it should still be clear to most beginner;
        // scala programmers.)
        let headerContent: String = {;
          let savedAttrText: Option<String> = {;
            if headerTypeIds.size > 0) {
              getAttrText(entity_in, headerTypeIds.get(0))
            } else {
              None
            }
          }
          savedAttrText.getOrElse( {
            ui.display_text(prompt1 + "html page \"<head>\" section contents" + prompt2 +
                           " (Title & 'meta name=\"description\"' tags are automatically filled in from the entity's name.)" +
                           prompt3 + "\"" + Util.HEADER_CONTENT_TAG + "\"" + prompt4, false)
            let s: String = Util.edit_multiline_text("", ui);
            s
          })
        }
        let beginBodyContent: String = {;
          let savedAttrText: Option<String> = {;
            if bodyContentTypeIds.size > 0) {
              getAttrText(entity_in, bodyContentTypeIds.get(0))
            } else {
              None
            }
          }
          savedAttrText.getOrElse({
            ui.display_text(prompt1 + "initial *body* content (like a common banner or header)" + prompt2 +
                           prompt3 + "\"" + Util.BODY_CONTENT_TAG + "\"" + prompt4, false)
            let beginBodyContentIn: String = Util.edit_multiline_text("", ui);
            beginBodyContentIn
          })
        }
        // (This value is an Option so that if None, it tells the program that the user wants out. The others haven't been set up that way (yet?).)
        let footerContent: Option<String> = {;
          let savedAttrText: Option<String> = {;
            if footerTypeIds.size > 0) {
              getAttrText(entity_in, footerTypeIds.get(0))
            } else {
              None
            }
          }
          if savedAttrText.isEmpty) {
            // idea (in task list):  have the date default to the entity creation date, then later add/replace that (w/ range or what for ranges?)
            // with the last edit date, when that feature exists.
            let copyrightYearAndName = ui.ask_for_string(Some(Array("On a SINGLE LINE, enter copyright year(s) and holder's name, i.e., the \"2015 John Doe\" part " +;
                                                                  "of \"Copyright 2015 John Doe\" (This accepts HTML so can also be used for a " +
                                                                  "page footer, for example.)" +
                                                                  prompt3 + "\"" + Util.FOOTER_CONTENT_TAG + "\"" + prompt4)))
            copyrightYearAndName
          } else {
            savedAttrText
          }
        }
        (headerContent, beginBodyContent, footerContent)
      }

      *
       * @param relationIn  (See comment on "@param relationIn" on method askWhetherDeleteOrArchiveEtc. )
       *
        fn goToRelatedPlaces(entity_in: Entity, relationSourceEntityIn: Option<Entity> = None,
                            relationIn: Option[AttributeWithValidAndObservedDates] = None, templateEntityId: Option<i64>) {
        //idea: make this and similar locations share code? What other places could?? There is plenty of duplicated code here!
        require(relationIn.isEmpty || relationIn.get.isInstanceOf[RelationToLocalEntity] || relationIn.get.isInstanceOf[RelationToRemoteEntity])
        let leading_text = Some(Array("Go to..."));
        let seeContainingEntities_choiceNumber: i32 = 1;
        let seeContainingGroups_choiceNumber: i32 = 2;
        let goToRelation_choiceNumber: i32 = 3;
        let goToRelationType_choiceNumber: i32 = 4;
        // The next 2 values are 3 & 4 in case the previous 2 are unused.  If the previous 2 are used, the next 2 will be += 2, below.
        let mut goToTemplateEntity_choiceNumber: i32 = 3;
        let mut goToClass_choiceNumber: i32 = 4;
        let numContainingEntities: i64 = {;
          let (nonArchived, archived) = entity_in.getCountOfContainingLocalEntities;
          if entity_in.mDB.include_archived_entities)  nonArchived + archived
          else nonArchived
        }
        // (idea: make this next call efficient: now it builds them all when we just want a count; but is infrequent & likely small numbers)
        let numContainingGroups = entity_in.getCountOfContainingGroups;
        let mut containingGroup: Option[Group] = None;
        let mut containingRtg: Option[RelationToGroup] = None;
        if numContainingGroups == 1) {
          let containingGroupsIds: java.util.ArrayList[i64] = entity_in.getContainingGroupsIds;
          // (Next line is just confirming the consistency of logic that got us here: see 'if' just above.)
          require(containingGroupsIds.size == 1)
          containingGroup = Some(new Group(entity_in.mDB, containingGroupsIds.get(0)))

          let containingRtgList: util.ArrayList[RelationToGroup] = entity_in.getContainingRelationsToGroup(0, Some(1));
          if containingRtgList.size < 1) {
            ui.display_text("There is a group containing the entity (" + entity_in.get_name + "), but:  " + Util.ORPHANED_GROUP_MESSAGE)
          } else {
            containingRtg = Some(containingRtgList.get(0))
          }
        }

        let mut choices = Vec<String>("See entities that directly relate to this entity (" + numContainingEntities + ")",;
                                    if numContainingGroups == 1) {
                                      "Go to group containing this entity: " + containingGroup.get.get_name
                                    } else {
                                      "See groups containing this entity (" + numContainingGroups + ")"
                                    })
        // (check for existence because other things could have been deleted or archived while browsing around different menu options.)
        if relationIn.is_defined && relationSourceEntityIn.is_defined && relationSourceEntityIn.get.mDB.entity_key_exists(relationSourceEntityIn.get.get_id)) {
          choices = choices :+ "Go edit the relation to entity that led here: " +
                               relationIn.get.get_display_string(15, Some(entity_in), Some(new RelationType(relationIn.get.mDB, relationIn.get.getAttrTypeId)))
          choices = choices :+ "Go to the type, for the relation that led here: " + new Entity(relationIn.get.mDB, relationIn.get.getAttrTypeId).get_name
          goToTemplateEntity_choiceNumber += 2
          goToClass_choiceNumber += 2
        }
        if templateEntityId.is_defined) {
          choices = choices ++ Vec<String>("Go to template entity")
          choices = choices ++ Vec<String>("Go to class")
        }
        // (Here for reference, for now. See cmt re one possible usage below. But if ever used, specify local vs. remote?)
        //var relationToEntity: Option[RelationToEntity] = relationIn

        let response = ui.ask_which(leading_text, choices, Vec<String>());
        if response.is_defined) {
          let goWhereAnswer = response.get;
          if goWhereAnswer == seeContainingEntities_choiceNumber && goWhereAnswer <= choices.length) {
            let leading_text = List[String]("Pick from menu, or an entity by letter");
            let choices: Vec<String> = Array(Util.LIST_NEXT_ITEMS_PROMPT);
            let numDisplayableItems: i64 = ui.maxColumnarChoicesToDisplayAfter(leading_text.size, choices.length, Util.maxNameLength);
            // This is partly set up so it could handle multiple screensful, but would need to be broken into a recursive method that
            // can specify dif't values on each call, for the startingIndexIn parm of getRelatingEntities.  I.e., could make it look more like
            // searchForExistingObject or such ? IF needed.  But to be needed means the user is putting the same object related by multiple
            // entities: enough to fill > 1 screen when listed.
            let containingEntities: util.ArrayList[(i64, Entity)] = entity_in.getLocalEntitiesContainingEntity(0, Some(numDisplayableItems));
            let containingEntitiesStatusAndNames: Vec<String> = containingEntities.toArray.map {;
                                                                                          case relTypeIdAndEntity: (i64, Entity) =>
                                                                                            let entity: Entity = relTypeIdAndEntity._2;
                                                                                            entity.getArchivedStatusDisplayString + entity.get_name
                                                                                          case _ => throw new OmException("??")
                                                                                        }
            let ans = ui.ask_which(Some(leading_text.toArray), choices, containingEntitiesStatusAndNames);
            if ans.is_defined) {
              let answer = ans.get;
              if answer == 1 && answer <= choices.length) {
                // see comment above
                ui.display_text("not yet implemented")
              } else if answer > choices.length && answer <= (choices.length + containingEntities.size)) {
                // those in the condition on the previous line are 1-based, not 0-based.
                let index = answer - choices.length - 1;
                // user typed a letter to select.. (now 0-based); selected a new object and so we return to the previous menu w/ that one displayed & current
                let entity: Entity = containingEntities.get(index)._2;
                new EntityMenu(ui, controller).entityMenu(entity)
              } else {
                ui.display_text("unknown response")
              }
            }
          } else if goWhereAnswer == seeContainingGroups_choiceNumber && goWhereAnswer <= choices.length) {
            if numContainingGroups == 1) {
              require(containingGroup.is_defined)
              new QuickGroupMenu(ui, controller).quickGroupMenu(containingGroup.get, 0, containingRtg, containingEntityIn = None)
            } else {
              viewContainingGroups(entity_in)
            }
          } else if goWhereAnswer == goToRelation_choiceNumber && relationIn.is_defined && goWhereAnswer <= choices.length) {
            fn dummyMethod(inDb: Database, dh_in: RelationToEntityDataHolder, editing_in: Boolean, ui: TextUI) -> Option[RelationToEntityDataHolder] {
              Some(dh_in)
            }
            fn updateRelationToEntity(dhInOut: RelationToEntityDataHolder) {
              // This "if" exists only to get things to compile while limiting visibility of "RelationToEntity" (per comments in that class).
              //noinspection TypeCheckCanBeMatch
              if relationIn.get.isInstanceOf[RelationToLocalEntity]) {
                relationIn.get.asInstanceOf[RelationToLocalEntity].update(dhInOut.valid_on_date, Some(dhInOut.observationDate), Some(dhInOut.attrTypeId))
              } else if relationIn.get.isInstanceOf[RelationToRemoteEntity]) {
                relationIn.get.asInstanceOf[RelationToRemoteEntity].update(dhInOut.valid_on_date, Some(dhInOut.observationDate), Some(dhInOut.attrTypeId))
              } else {
                throw new OmException("unexpected type: " + relationIn.getClass.getCanonicalName)
              }
            }
            let relatedId2 = {;
              // This "if" exists only to get things to compile while limiting visibility of "RelationToEntity" (per comments on that class).
              //noinspection TypeCheckCanBeMatch
              if relationIn.get.isInstanceOf[RelationToLocalEntity]) {
                relationIn.get.asInstanceOf[RelationToLocalEntity].getRelatedId2
              } else if relationIn.get.isInstanceOf[RelationToRemoteEntity]) {
                relationIn.get.asInstanceOf[RelationToRemoteEntity].getRelatedId2
              } else {
                throw new OmException("unexpected type: " + relationIn.getClass.getCanonicalName)
              }
            }
            let relationToEntityDH: RelationToEntityDataHolder = new RelationToEntityDataHolder(relationIn.get.getAttrTypeId, relationIn.get.getValidOnDate,;
                                                                                                relationIn.get.getObservationDate, relatedId2,
                                                                                                relationIn.get.mDB.is_remote, relationIn.get.mDB.id)
            controller.askForInfoAndUpdateAttribute[RelationToEntityDataHolder](relationIn.get.mDB, relationToEntityDH, askForAttrTypeId = true,
                                                                                Util.RELATION_TO_LOCAL_ENTITY_TYPE,
                                                                                "CHOOSE TYPE OF Relation to Entity:", dummyMethod, updateRelationToEntity)
            // Force a reread from the DB so it shows the right info SO THIS IS NOT FORGOTTEN, IN CASE we add later a call a menu which
            // needs it as a parameter.  But if ever used, specify local vs. remote.
            //relationToEntity = Some(new RelationToEntity(db, relationIn.get.get_id, relationIn.get.getAttrTypeId, relationIn.get.getRelatedId1,
            //                                             relationIn.get.getRelatedId2))
          } else if goWhereAnswer == goToRelationType_choiceNumber && relationIn.is_defined && goWhereAnswer <= choices.length) {
            new EntityMenu(ui, controller).entityMenu(new Entity(relationIn.get.mDB, relationIn.get.getAttrTypeId))
          } else if goWhereAnswer == goToTemplateEntity_choiceNumber && templateEntityId.is_defined && goWhereAnswer <= choices.length) {
            new EntityMenu(ui, controller).entityMenu(new Entity(entity_in.mDB, templateEntityId.get))
          } else if goWhereAnswer == goToClass_choiceNumber && templateEntityId.is_defined && goWhereAnswer <= choices.length) {
            let classId: Option<i64> = entity_in.getClassId;
            if classId.isEmpty) {
              throw new OmException("Unexpectedly, this entity doesn't seem to have a class id.  That is probably a bug.")
            } else {
              new ClassMenu(ui, controller).classMenu(new EntityClass(entity_in.mDB, classId.get))
            }
          } else {
            ui.display_text("invalid response")
          }
        }
      }

        fn viewContainingGroups(entity_in: Entity) -> Option<Entity> {
        let leading_text = List[String]("Pick from menu, or a letter to (go to if one or) see the entities containing that group, or Alt+<letter> for the actual " +;
                                       "*group* by letter")
        let choices: Vec<String> = Array(Util.LIST_NEXT_ITEMS_PROMPT);
        let numDisplayableItems = ui.maxColumnarChoicesToDisplayAfter(leading_text.size, choices.length, Util.maxNameLength);
        // (see comment in similar location just above where this is called, near "val containingEntities: util.ArrayList"...)
        let containingRelationToGroups: util.ArrayList[RelationToGroup] = entity_in.getContainingRelationsToGroup(0, Some(numDisplayableItems));
        let containingRtgDescriptions: Vec<String> = containingRelationToGroups.toArray.map {;
                                                                                                case rtg: (RelationToGroup) =>
                                                                                                  let entityName: String = new Entity(rtg.mDB,;
                                                                                                                                      rtg.getParentId).get_name
                                                                                                  let rt: RelationType = new RelationType(rtg.mDB,;
                                                                                                                                          rtg.getAttrTypeId)
                                                                                                  "entity " + entityName + " " +
                                                                                                  rtg.get_display_string(Util.maxNameLength, None, Some(rt))
                                                                                                case _ => throw new OmException("??")
                                                                                              }
        let ans = ui.ask_whichChoiceOrItsAlternate(Some(leading_text.toArray), choices, containingRtgDescriptions);
        if ans.isEmpty) None
        else {
          let (answer, userPressedAltKey: Boolean) = ans.get;
          // those in the condition on the previous line are 1-based, not 0-based.
          let index = answer - choices.length - 1;
          if answer == 1 && answer <= choices.length) {
            // see comment above
            ui.display_text("not yet implemented")
            None
          } else if answer > choices.length && answer <= (choices.length + containingRelationToGroups.size) && !userPressedAltKey) {
            // This displays (or allows to choose) the entity that contains the group, rather than the chosen group itself.  Probably did it that way originally
            // because I thought it made more sense to show a group in context than by itself.
            let containingRelationToGroup = containingRelationToGroups.get(index);
            let containingEntities = containingRelationToGroup.mDB.getEntitiesContainingGroup(containingRelationToGroup.getGroupId, 0);
            let numContainingEntities = containingEntities.size;
            if numContainingEntities == 1) {
              let containingEntity: Entity = containingEntities.get(0)._2;
              new EntityMenu(ui, controller).entityMenu(containingEntity, containingGroupIn = Some(new Group(containingRelationToGroup.mDB,
                                                                                                             containingRelationToGroup.getGroupId)))
            } else {
              controller.chooseAmongEntities(containingEntities)
            }
          } else if answer > choices.length && answer <= (choices.length + containingRelationToGroups.size) && userPressedAltKey) {
            // user typed a letter to select.. (now 0-based); selected a new object and so we return to the previous menu w/ that one displayed & current
            let id: i64 = containingRelationToGroups.get(index).get_id;
            let entityId: i64 = containingRelationToGroups.get(index).getParentId;
            let groupId: i64 = containingRelationToGroups.get(index).getGroupId;
            let relTypeId: i64 = containingRelationToGroups.get(index).getAttrTypeId;
            new QuickGroupMenu(ui, controller).quickGroupMenu(new Group(entity_in.mDB, groupId), 0,
                                                              Some(new RelationToGroup(entity_in.mDB, id, entityId, relTypeId, groupId)),
                                                              Some(entity_in), containingEntityIn = None)
          } else {
            ui.display_text("unknown response")
            None
          }
        }
      }

      *
       * @param relationIn Is of type Option[AttributeWithValidAndObservedDates], because it needs to be a RelationToLocalEntity (RTLE) *or* a
       *                   RelationToRemoteEntity, and I don't know how to specify that except by the lowest available parent and the
       *                   following require statement.  I don't want to use RelationToEntity, because specifying local vs. remote helps keep the code unambiguous
       *                   as to whether things need special remote logic, ie if the code is forced to choose either RTLE or RTRE.  Same reasons elsewhere.
       * @return None means "get out", or Some(choiceNum) if a choice was made.
       *
        fn askWhetherDeleteOrArchiveEtc(entity_in: Entity, relationIn: Option[AttributeWithValidAndObservedDates], relationSourceEntityIn: Option<Entity>,
                                       containingGroupIn: Option[Group]) -> (Option[Int], Int, Int, Int) {
        require(relationIn.isEmpty || relationIn.get.isInstanceOf[RelationToLocalEntity] || relationIn.get.isInstanceOf[RelationToRemoteEntity])

        let groupCount: i64 = entity_in.getCountOfContainingGroups;
        let (entityCountNonArchived, entityCountArchived) = entity_in.getCountOfContainingLocalEntities;
        let relToGroupCnt = entity_in.getRelationToGroupCount;
        let relToLocalEntityCnt = entity_in.getRelationToLocalEntityCount(true);
        let relToLocalEntityCntNotArchived = entity_in.getRelationToLocalEntityCount(false);
        let relToLocalEntityCntArchived = relToLocalEntityCnt - relToLocalEntityCntNotArchived ;
        let relToRemoteEntityCnt = entity_in.getRelationToRemoteEntityCount;
        let totalNumOfAttributes = entity_in.getAttributeCount(true);
        let adjNumOfAttributes = (totalNumOfAttributes - relToGroupCnt) - relToLocalEntityCnt;
        //(Idea: the next line/block could use thorough tests, incl of the "remote" part)
        let leading_text = Some(Array(("Choose a deletion or archiving option:  " + Util.NEWLN +;
          (if entityCountNonArchived != 0 || entityCountArchived != 0) {
            "  The entity is " + Util.get_containing_entities_description(entityCountNonArchived, entityCountArchived) + "." + Util.NEWLN
          } else "")
          +
          (if groupCount != 0) {
            "  The entity is contained in " + groupCount + " group(s)." + Util.NEWLN
          } else "")
          +
          (if relToLocalEntityCnt != 0 || relToLocalEntityCntArchived != 0 || relToRemoteEntityCnt != 0
            || relToGroupCnt != 0 || adjNumOfAttributes != 0)
            {
              let mut directContains = "The entity directly contains: " + Util.NEWLN +;
              (if relToLocalEntityCnt != 0) {
                "    " + relToLocalEntityCnt + " local entity(ies)" +
                (if relToLocalEntityCntArchived != 0) {
                  " (" + relToLocalEntityCntArchived + " of them archived)"
                } else "") + Util.NEWLN
              } else "") +
              (if relToRemoteEntityCnt != 0) {
                //(Idea: similar places might also mention remote entities..?)
                relToRemoteEntityCnt + "    remote entity(ies) (incl. archived), " + Util.NEWLN
              } else "") +
              (if relToGroupCnt != 0) {
                "    " + relToGroupCnt + " group(s)" + Util.NEWLN
              } else "") +
              (if adjNumOfAttributes != 0) {
                "    " + adjNumOfAttributes + " other attribute(s)" + Util.NEWLN
              } else "")

              "  " + directContains.trim + "." + Util.NEWLN
              //directContains
            } else "")
          ).trim
        ))

        let mut choices = Array("Delete this entity",;
                            if entity_in.is_archived) {
                              "Un-archive this entity"
                            } else {
                              "Archive this entity (remove from visibility but not permanent/total deletion)"
                            })
        let delEntityLink_choiceNumber: i32 = 3;
        let mut delFromContainingGroup_choiceNumber: i32 = 3;
        let mut showAllArchivedEntities_choiceNumber: i32 = 3;
        // (check for existence because other things could have been deleted or archived while browsing around different menu options.)
        if relationIn.is_defined && relationSourceEntityIn.is_defined && relationSourceEntityIn.get.mDB.entity_key_exists(relationSourceEntityIn.get.get_id)) {
          // means we got here by selecting a Relation attribute on another entity, so entity_in is the "entityId2" in that relation; so show some options,
          // because
          // we eliminated a separate menu just for the relation and put them here, for UI usage simplicity.
          choices = choices :+ "Delete the link from the linking (or containing) entity:" + Util.NEWLN +
                               "    \"" + relationSourceEntityIn.get.get_name + "\", " + Util.NEWLN +
                               "  ...to this one:" + Util.NEWLN +
                               "    \"" + entity_in.get_name + "\""
          delFromContainingGroup_choiceNumber += 1
          showAllArchivedEntities_choiceNumber += 1
        }
        if containingGroupIn.is_defined) {
          choices = choices :+ "Delete the link from the containing group:" + Util.NEWLN +
                               "    \"" + containingGroupIn.get.get_name + "\"," + Util.NEWLN +
                               "  ...to this Entity:" + Util.NEWLN +
                               "    \"" + entity_in.get_name + "\""
          showAllArchivedEntities_choiceNumber += 1
        }
        choices = choices :+ (if !entity_in.mDB.include_archived_entities) "Show archived entities" else "Do not show archived entities")

        let delOrArchiveAnswer: Option[(Int)] = ui.ask_which(leading_text, choices, Vec<String>());
        (delOrArchiveAnswer, delEntityLink_choiceNumber, delFromContainingGroup_choiceNumber, showAllArchivedEntities_choiceNumber)
      }

    */
}
