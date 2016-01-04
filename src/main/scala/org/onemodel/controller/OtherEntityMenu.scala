/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2015-2016 inclusive, Luke A. Call; all rights reserved.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule, guidelines around binary
    distribution, and the GNU Affero General Public License as published by the Free Software Foundation, either version 3
    of the License, or (at your option) any later version.  See the file LICENSE for details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
package org.onemodel.controller

import java.util

import org.onemodel.{OmException, TextUI}
import org.onemodel.database.PostgreSQLDatabase
import org.onemodel.model._

/** This is simply to hold less-used operations so the main EntityMenu can be the most-used stuff.
  */
class OtherEntityMenu (val ui: TextUI, val db: PostgreSQLDatabase, val controller: Controller) {

  def otherEntityMenu(entityIn: Entity, attributeRowsStartingIndexIn: Int = 0, relationSourceEntityIn: Option[Entity],
                      containingRelationToEntityIn: Option[RelationToEntity], containingGroupIn: Option[Group],
                      classDefiningEntityIdIn: Option[Long]): Option[Entity] = {
    try {
      require(entityIn != null)
      val leadingText = Array[String]{"**CURRENT ENTITY " + entityIn.getId + ": " + entityIn.getDisplayString}
      var choices = Array[String]("Edit public/nonpublic status",
                                  "Import/Export...",
                                  "Edit entity name",
                                  "Delete or Archive this entity (or link)...",
                                  "Go to other related entities or groups...",
                                  "(stub)")
      //  don't show the "set default" option if it's already been done w/ this same one:
      val defaultEntity: Option[Long] = controller.findDefaultDisplayEntity
      val entityIsAlreadyTheDefault: Boolean = defaultEntity.isDefined && defaultEntity.get == entityIn.getId
      if (! entityIsAlreadyTheDefault) {
        choices = choices :+ ((if (defaultEntity.isEmpty) "****TRY ME---> " else "") +
                              "Set current entity as default (first to come up when launching this program.)")
      } else choices = choices :+ "(stub)"

      val response = ui.askWhich(Some(leadingText), choices)
      if (response.isEmpty) None
      else {
        val answer = response.get
        if (answer == 1) {
          // The condition for this (when it was part of EntityMenu) used to include " && !entityIn.isInstanceOf[RelationType]", but maybe it's better w/o that.
          controller.editEntityPublicStatus(entityIn)
          // reread from db to refresh data for display, like public/non-public status:
          otherEntityMenu(new Entity(db, entityIn.getId), attributeRowsStartingIndexIn, relationSourceEntityIn, containingRelationToEntityIn, containingGroupIn,
                          classDefiningEntityIdIn)
        } else if (answer == 2) {
          val importOrExportAnswer = ui.askWhich(None, Array("Import", "Export to a text file (outline)", "Export to html pages"), Array[String]())
          if (importOrExportAnswer.isDefined) {
            if (importOrExportAnswer.get == 1) new ImportExport(ui, db, controller).importCollapsibleOutlineAsGroups(entityIn)
            else if (importOrExportAnswer.get == 2) new ImportExport(ui, db, controller).export(entityIn, ImportExport.TEXT_EXPORT_TYPE, None)
            else if (importOrExportAnswer.get == 3) {
              // idea (in task list):  have the date default to the entity creation date, then later add/replace that (w/ range or what for ranges?)
              // with the last edit date, when that feature exists.
              val copyrightYearAndName = ui.askForString(Some(Array("Enter copyright year(s) and holder's name, i.e., the \"2015 John Doe\" part " +
                                                                    "of \"Copyright 2015 John Doe\" (This accepts HTML so can also be used for a " +
                                                                    "page footer, for example.)")))
              if (copyrightYearAndName.isDefined && copyrightYearAndName.get.trim.nonEmpty) {
                new ImportExport(ui, db, controller).export(entityIn, ImportExport.HTML_EXPORT_TYPE, copyrightYearAndName)
              }
            }
          }
          otherEntityMenu(entityIn, attributeRowsStartingIndexIn, relationSourceEntityIn, containingRelationToEntityIn, containingGroupIn, classDefiningEntityIdIn)
        } else if (answer == 3) {
          val editedEntity: Option[Entity] = controller.editEntityName(entityIn)
          otherEntityMenu(if (editedEntity.isDefined) editedEntity.get else entityIn, attributeRowsStartingIndexIn, relationSourceEntityIn,
                          containingRelationToEntityIn, containingGroupIn, classDefiningEntityIdIn)
        } else if (answer == 4) {
          val (delOrArchiveAnswer, delEntityLink_choiceNumber, delFromContainingGroup_choiceNumber) =
            controller.askWhetherDeleteOrArchiveEtc(entityIn, containingRelationToEntityIn, relationSourceEntityIn, containingGroupIn)

          if (delOrArchiveAnswer.isDefined) {
            val answer = delOrArchiveAnswer.get
            if (answer == 1 || answer == 2) {
              val thisEntityWasDeletedOrArchived = controller.deleteOrArchiveEntity(entityIn, answer == 1)
              if (thisEntityWasDeletedOrArchived) None
              else Some(entityIn)
            } else if (answer == delEntityLink_choiceNumber && containingRelationToEntityIn.isDefined && answer <= choices.length) {
              val ans = ui.askYesNoQuestion("DELETE the relation: ARE YOU SURE?", Some(""))
              if (ans.isDefined && ans.get) {
                containingRelationToEntityIn.get.delete()
                None
              } else {
                ui.displayText("Did not delete relation.", waitForKeystroke = false)
                Some(entityIn)
              }
            } else if (answer == delFromContainingGroup_choiceNumber && containingGroupIn.isDefined && answer <= choices.length) {
              if (controller.removeEntityReferenceFromGroup_Menu(entityIn, containingGroupIn))
                None
              else
                Some(entityIn)
            } else {
              ui.displayText("invalid response")
              otherEntityMenu(new Entity(db, entityIn.getId), attributeRowsStartingIndexIn, relationSourceEntityIn, containingRelationToEntityIn,
                              containingGroupIn, classDefiningEntityIdIn)
            }
          } else {
            Some(entityIn)
          }
        } else if (answer == 5) {
          goToRelatedPlaces(attributeRowsStartingIndexIn, entityIn, relationSourceEntityIn, containingRelationToEntityIn, containingGroupIn, classDefiningEntityIdIn)
          //ck 1st if entity exists, if not return None. It could have been deleted while navigating around.
          if (db.entityKeyExists(entityIn.getId)) {
            new EntityMenu(ui, db, controller).entityMenu(entityIn, attributeRowsStartingIndexIn, None, None, containingRelationToEntityIn, containingGroupIn)
          }
          else
            None
        } else if (answer == 7 && answer <= choices.length && !entityIsAlreadyTheDefault) {
          // updates user preferences such that this obj will be the one displayed by default in future.
          controller.mPrefs.putLong("first_display_entity", entityIn.getId)
          otherEntityMenu(entityIn, attributeRowsStartingIndexIn, relationSourceEntityIn, containingRelationToEntityIn, containingGroupIn, classDefiningEntityIdIn)
        } else {
          ui.displayText("invalid response")
          otherEntityMenu(entityIn, attributeRowsStartingIndexIn, relationSourceEntityIn, containingRelationToEntityIn, containingGroupIn, classDefiningEntityIdIn)
        }
      }
    } catch {
      case e: Exception =>
        controller.handleException(e)
        val ans = ui.askYesNoQuestion("Go back to what you were doing (vs. going out)?", Some("y"))
        if (ans.isDefined && ans.get) otherEntityMenu(entityIn, attributeRowsStartingIndexIn, relationSourceEntityIn, containingRelationToEntityIn,
                                                      containingGroupIn, classDefiningEntityIdIn)
        else None
    }
  }

  def goToRelatedPlaces(startingAttributeRowsIndexIn: Int, entityIn: Entity, relationSourceEntityIn: Option[Entity] = None,
                        relationIn: Option[RelationToEntity] = None, containingGroupIn: Option[Group] = None,
                        classDefiningEntityId: Option[Long]) {
    //idea: make this and similar locations share code? What other places could?? There is plenty of duplicated code here!
    val leadingText = Some(Array("Go to..."))
    val seeContainingEntities_choiceNumber: Int = 1
    val seeContainingGroups_choiceNumber: Int = 2
    val goToRelation_choiceNumber: Int = 3
    val goToRelationType_choiceNumber: Int = 4
    var goToClassDefiningEntity_choiceNumber: Int = 3
    val numContainingEntities = db.getEntitiesContainingEntity(entityIn, 0).size
    // (idea: make this next call efficient: now it builds them all when we just want a count; but is infrequent & likely small numbers)
    val numContainingGroups = db.getCountOfGroupsContainingEntity(entityIn.getId)
    var containingGroup: Option[Group] = None
    var containingRtg: Option[RelationToGroup] = None
    if (numContainingGroups == 1) {
      val containingGroupsIds: List[Long] = db.getContainingGroupsIds(entityIn.getId)
      // (Next line is just confirming the consistency of logic that got us here: see 'if' just above.)
      require(containingGroupsIds.size == 1)
      containingGroup = Some(new Group(db, containingGroupsIds.head))

      val containingRtgList: util.ArrayList[RelationToGroup] = db.getContainingRelationToGroups(entityIn, 0, Some(1))
      if (containingRtgList.size < 1) {
        ui.displayText("There is a group containing the entity (" + entityIn.getName + "), but:  " + Controller.ORPHANED_GROUP_MESSAGE)
      } else {
        containingRtg = Some(containingRtgList.get(0))
      }
    }

    var choices = Array[String]("See entities that directly relate to this entity ( " + numContainingEntities + ")",
                                if (numContainingGroups == 1) {
                                  "Go to group containing this entity: " + containingGroup.get.getName
                                } else {
                                  "See groups containing this entity (" + numContainingGroups + ")"
                                })
    if (relationIn.isDefined) {
      choices = choices :+ "Go edit the relation to entity that that led here: " +
                           relationIn.get.getDisplayString(15, relationSourceEntityIn, Some(new RelationType(db, relationIn.get.getAttrTypeId)))
      choices = choices :+ "Go to the type, for the relation that that led here: " + new Entity(db, relationIn.get.getAttrTypeId).getName
      goToClassDefiningEntity_choiceNumber += 2
    }
    if (classDefiningEntityId.isDefined) {
      choices = choices ++ Array[String]("Go to class-defining entity")
    }
    var relationToEntity: Option[RelationToEntity] = relationIn

    val response = ui.askWhich(leadingText, choices, Array[String]())
    if (response.isDefined) {
      val goWhereAnswer = response.get
      if (goWhereAnswer == seeContainingEntities_choiceNumber && goWhereAnswer <= choices.length) {
        val leadingText = List[String]("Pick from menu, or an entity by letter")
        val choices: Array[String] = Array(controller.listNextItemsPrompt)
        val numDisplayableItems: Long = ui.maxColumnarChoicesToDisplayAfter(leadingText.size, choices.length, Controller.maxNameLength)
        // This is partly set up so it could handle multiple screensful, but would need to be broken into a recursive method that
        // can specify dif't values on each call, for the startingIndexIn parm of getRelatingEntities.  I.e., could make it look more like
        // searchForExistingObject or such ? IF needed.  But to be needed means the user is putting the same object related by multiple
        // entities: enough to fill > 1 screen when listed.
        val containingEntities: util.ArrayList[(Long, Entity)] = db.getEntitiesContainingEntity(entityIn, 0, Some(numDisplayableItems))
        val containingEntitiesNames: Array[String] = containingEntities.toArray.map {
                                                                                      case relTypeIdAndEntity: (Long, Entity) =>
                                                                                        val entity: Entity = relTypeIdAndEntity._2
                                                                                        entity.getName
                                                                                      case _ => throw new OmException("??")
                                                                                    }
        val ans = ui.askWhich(Some(leadingText.toArray), choices, containingEntitiesNames)
        if (ans.isDefined) {
          val answer = ans.get
          if (answer == 1 && answer <= choices.length) {
            // see comment above
            ui.displayText("not yet implemented")
          } else if (answer > choices.length && answer <= (choices.length + containingEntities.size)) {
            // those in the condition on the previous line are 1-based, not 0-based.
            val index = answer - choices.length - 1
            // user typed a letter to select.. (now 0-based); selected a new object and so we return to the previous menu w/ that one displayed & current
            val entity: Entity = containingEntities.get(index)._2
            new EntityMenu(ui, db, controller).entityMenu(entity)
          } else {
            ui.displayText("unknown response")
          }
        }
      } else if (goWhereAnswer == seeContainingGroups_choiceNumber && goWhereAnswer <= choices.length) {
        if (numContainingGroups == 1) {
          require(containingGroup.isDefined)
          new QuickGroupMenu(ui, db, controller).quickGroupMenu(containingGroup.get, 0, containingRtg, containingEntityIn = None)
        } else {
          viewContainingGroups(entityIn)
        }
      } else if (goWhereAnswer == goToRelation_choiceNumber && relationIn.isDefined && goWhereAnswer <= choices.length) {
        def dummyMethod(inDH: RelationToEntityDataHolder, inEditing: Boolean): Option[RelationToEntityDataHolder] = {
          Some(inDH)
        }
        def updateRelationToEntity(dhInOut: RelationToEntityDataHolder) {
          relationIn.get.update(Some(dhInOut.attrTypeId), dhInOut.validOnDate, Some(dhInOut.observationDate))
        }
        val relationToEntityDH: RelationToEntityDataHolder = new RelationToEntityDataHolder(relationIn.get.getAttrTypeId, relationIn.get.getValidOnDate,
                                                                                            relationIn.get.getObservationDate, relationIn.get.getRelatedId2)
        controller.askForInfoAndUpdateAttribute[RelationToEntityDataHolder](relationToEntityDH, Controller.RELATION_TO_ENTITY_TYPE,
                                                                            "CHOOSE TYPE OF Relation to Entity:", dummyMethod, updateRelationToEntity)
        // force a reread from the DB so it shows the right info on the repeated menu (below):
        relationToEntity = Some(new RelationToEntity(db, relationIn.get.getId, relationIn.get.getAttrTypeId, relationIn.get.getRelatedId1,
                                                     relationIn.get.getRelatedId2))
      } else if (goWhereAnswer == goToRelationType_choiceNumber && relationIn.isDefined && goWhereAnswer <= choices.length) {
        new EntityMenu(ui, db, controller).entityMenu(new Entity(db, relationIn.get.getAttrTypeId))
      } else if (goWhereAnswer == goToClassDefiningEntity_choiceNumber && classDefiningEntityId.isDefined && goWhereAnswer <= choices.length) {
        new EntityMenu(ui, db, controller).entityMenu(new Entity(db, classDefiningEntityId.get))
      } else {
        ui.displayText("invalid response")
      }
    }
  }

  def viewContainingGroups(entityIn: Entity): Option[Entity] = {
    val leadingText = List[String]("Pick from menu, or a letter to (go to if one or) see the entities containing that group, or Alt+<letter> for the actual " +
                                   "*group* by letter")
    val choices: Array[String] = Array(controller.listNextItemsPrompt)
    val numDisplayableItems = ui.maxColumnarChoicesToDisplayAfter(leadingText.size, choices.length, Controller.maxNameLength)
    // (see comment in similar location just above where this is called, near "val containingEntities: util.ArrayList"...)
    val containingRelationToGroups: util.ArrayList[RelationToGroup] = db.getContainingRelationToGroups(entityIn, 0, Some(numDisplayableItems))
    val containingRtgDescriptions: Array[String] = containingRelationToGroups.toArray.map {
                                                                                            case rtg: (RelationToGroup) =>
                                                                                              val entityName: String = new Entity(db, rtg.getParentId).getName
                                                                                              val rt: RelationType = new RelationType(db, rtg.getAttrTypeId)
                                                                                              "entity " + entityName + " " +
                                                                                              rtg.getDisplayString(Controller.maxNameLength, None, Some(rt))
                                                                                            case _ => throw new OmException("??")
                                                                                          }
    val ans = ui.askWhichChoiceOrItsAlternate(Some(leadingText.toArray), choices, containingRtgDescriptions)
    if (ans.isEmpty) None
    else {
      val (answer, userPressedAltKey: Boolean) = ans.get
      // those in the condition on the previous line are 1-based, not 0-based.
      val index = answer - choices.length - 1
      if (answer == 1 && answer <= choices.length) {
        // see comment above
        ui.displayText("not yet implemented")
        None
      } else if (answer > choices.length && answer <= (choices.length + containingRelationToGroups.size) && !userPressedAltKey) {
        // This displays (or allows to choose) the entity that contains the group, rather than the chosen group itself.  Probably did it that way originally
        // because I thought it made more sense to show a group in context than by itself.
        val containingRelationToGroup = containingRelationToGroups.get(index)
        val containingEntities = db.getEntitiesContainingGroup(containingRelationToGroup.getGroupId, 0)
        val numContainingEntities = containingEntities.size
        if (numContainingEntities == 1) {
          val containingEntity: Entity = containingEntities.get(0)._2
          new EntityMenu(ui, db, controller).entityMenu(containingEntity, containingGroupIn = Some(new Group(db, containingRelationToGroup.getGroupId)))
        } else {
          controller.chooseAmongEntities(containingEntities)
        }
      } else if (answer > choices.length && answer <= (choices.length + containingRelationToGroups.size) && userPressedAltKey) {
        // user typed a letter to select.. (now 0-based); selected a new object and so we return to the previous menu w/ that one displayed & current
        val id: Long = containingRelationToGroups.get(index).getId
        val entityId: Long = containingRelationToGroups.get(index).getParentId
        val groupId: Long = containingRelationToGroups.get(index).getGroupId
        val relTypeId: Long = containingRelationToGroups.get(index).getAttrTypeId
        new QuickGroupMenu(ui, db, controller).quickGroupMenu(new Group(db, groupId), 0, Some(new RelationToGroup(db, id, entityId, relTypeId, groupId)),
                                                              Some(entityIn), containingEntityIn = None)
      } else {
        ui.displayText("unknown response")
        None
      }
    }
  }

}
