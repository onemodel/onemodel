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

import java.io.File
import java.util
import org.onemodel._
import org.onemodel.model._
import org.onemodel.database.PostgreSQLDatabase

class EntityMenu(val ui: TextUI, val db: PostgreSQLDatabase, val controller: Controller) {
  // 2nd return value is whether entityIsDefault (ie whether default object when launching OM is already this entity)
  def getChoices(entityIn: Entity, relationSourceEntityIn: Option[Entity] = None, relationIn: Option[RelationToEntity] = None): (Array[String], Boolean) = {
    // (idea: might be a little silly to do it this way, once this # gets very big?:)
    var choices = Array[String]("Add attribute (quantity, true/false, date, text, external file, relation to entity or group: " + controller.mRelTypeExamples + ")...",
                                "Import/Export...",
                                "Edit name",
                                "Delete or Archive...",
                                "Go to...",
                                controller.listNextItemsPrompt)
    if (relationIn.isDefined) {
      // means we got here by selecting a Relation attribute on another entity, so entityIn is the "entityId2" in that relation; so show some options, because
      // we eliminated a separate menu just for the relation and put them here, for UI usage simplicity.
      require(relationIn.get.getRelatedId2 == entityIn.getId && relationSourceEntityIn.isDefined)
    }

    val defaultEntity: Option[Long] = controller.findDefaultDisplayEntity
    //  don't show the "set default" option if it's already been done w/ this same one:
    val entityIsAlreadyTheDefault: Boolean = defaultEntity.isDefined && defaultEntity.get == entityIn.getId
    if (! entityIsAlreadyTheDefault) {
      choices = choices :+ ((if (defaultEntity.isEmpty) "****TRY ME---> " else "") +
                            "Set current entity as default (first to come up when launching this program.)")
    } else choices = choices :+ "(stub)"
    choices = choices :+ "Edit public/nonpublic status"
    (choices, entityIsAlreadyTheDefault)
  }

  /** returns None if user wants out. */
  //@tailrec //removed for now until the compiler can handle it with where the method calls itself.
  //idea on scoping: make this limited like this somehow?:  private[org.onemodel] ... Same for all others like it?
  def entityMenu(startingAttributeIndexIn: Long, entityIn: Entity, relationSourceEntityIn: Option[Entity] = None,
                           relationIn: Option[RelationToEntity] = None, containingGroupIn: Option[Group] = None): Option[Entity] = try {
    require(entityIn != null)
    val numAttrsInEntity: Long = entityIn.getAttrCount
    val classDefiningEntityId: Option[Long] = entityIn.getClassDefiningEntityId
    val leadingText: Array[String] = new Array[String](2)
    val (choices: Array[String], entityIsAlreadyTheDefault: Boolean) = getChoices(entityIn, relationSourceEntityIn, relationIn)
    val numDisplayableAttributes = ui.maxColumnarChoicesToDisplayAfter(leadingText.length, choices.length, controller.maxNameLength)
    val (attributeObjList: java.util.ArrayList[Attribute], totalRowsAvailable) =
      db.getSortedAttributes(entityIn.getId, startingAttributeIndexIn, numDisplayableAttributes)
    val choicesModified = controller.addRemainingCountToPrompt(choices, attributeObjList.size, totalRowsAvailable, startingAttributeIndexIn)
    val leadingTextModified = getLeadingText(leadingText, attributeObjList, entityIn, relationSourceEntityIn, relationIn, containingGroupIn)
    val attributeDisplayStrings: Array[String] = getItemDisplayStrings(attributeObjList)

    val response = ui.askWhich(Some(leadingTextModified), choicesModified, attributeDisplayStrings)
    if (response.isEmpty) None
    else {
      val answer = response.get
      if (answer == 1) {
        addAttribute(startingAttributeIndexIn, entityIn, relationSourceEntityIn, relationIn, containingGroupIn)
        entityMenu(startingAttributeIndexIn, entityIn, relationSourceEntityIn, relationIn, containingGroupIn)
      }
      if (answer == 2) {
        val importOrExport = ui.askWhich(None, Array("Import", "Export to a text file (outline)", "Export to html pages"), Array[String]())
        if (importOrExport.isDefined) {
          if (importOrExport.get == 1) new ImportExport(ui, db, controller).importCollapsibleOutlineAsGroups(entityIn)
          else if (importOrExport.get == 2) new ImportExport(ui, db, controller).export(entityIn, ImportExport.TEXT_EXPORT_TYPE, None)
          else if (importOrExport.get == 3) {
            // idea (in task list):  have the date default to the entity creation date, then later add/replace that (w/ range or what for ranges?)
            // with the last edit date, when that feature exists.
            val copyrightYearAndName = ui.askForString(Some(Array("Enter copyright year(s) and holder's name, i.e., the \"2015 John Doe\" part " +
                                                                  "of \"Copyright 2015 John Doe\"")))
            new ImportExport(ui, db, controller).export(entityIn, ImportExport.HTML_EXPORT_TYPE, copyrightYearAndName)
          }
        }
        entityMenu(startingAttributeIndexIn, entityIn, relationSourceEntityIn, relationIn, containingGroupIn)
      }
      else if (answer == 3) {
        val editedEntity: Option[Entity] = controller.editEntityName(entityIn)
        entityMenu(startingAttributeIndexIn, if (editedEntity.isDefined) editedEntity.get else entityIn, relationSourceEntityIn, relationIn, containingGroupIn)
      } else if (answer == 4) {
        val (delOrArchiveAnswer, delEntityLink_choiceNumber, delFromContainingGroup_choiceNumber) =
          controller.askWhetherDeleteOrArchiveEtc(entityIn, relationIn, relationSourceEntityIn, containingGroupIn)

        if (delOrArchiveAnswer.isDefined) {
          val answer = delOrArchiveAnswer.get
          if (answer == 1 || answer == 2) {
            val thisEntityWasDeletedOrArchived = controller.deleteOrArchiveEntity(entityIn, answer == 1)
            if (thisEntityWasDeletedOrArchived) None
            else entityMenu(startingAttributeIndexIn, entityIn, relationSourceEntityIn, relationIn, containingGroupIn)
          } else if (answer == delEntityLink_choiceNumber && relationIn.isDefined && answer <= choices.length) {
            val ans = ui.askYesNoQuestion("DELETE the relation: ARE YOU SURE?")
            if (ans.isDefined && ans.get) {
              relationIn.get.delete()
              None
            } else {
              ui.displayText("Did not delete relation.", waitForKeystroke = false)
              entityMenu(startingAttributeIndexIn, entityIn, relationSourceEntityIn, relationIn, containingGroupIn)
            }
          } else if (answer == delFromContainingGroup_choiceNumber && containingGroupIn.isDefined && answer <= choices.length) {
            if (removeEntityReferenceFromGroup_Menu(entityIn, containingGroupIn))
              None
            else
              entityMenu(startingAttributeIndexIn, entityIn, relationSourceEntityIn, relationIn, containingGroupIn)
          } else {
            ui.displayText("invalid response")
            entityMenu(startingAttributeIndexIn, entityIn, relationSourceEntityIn, relationIn, containingGroupIn)
          }
        } else {
          entityMenu(startingAttributeIndexIn, entityIn, relationSourceEntityIn, relationIn, containingGroupIn)
        }
      } else if (answer == 5) {
        goToRelatedPlaces(startingAttributeIndexIn, entityIn, relationSourceEntityIn, relationIn, containingGroupIn, classDefiningEntityId)
      }
      else if (answer == 6) {
        val startingIndex: Long = listNextAttributes(attributeObjList, numAttrsInEntity, numAttrsInEntity)
        entityMenu(startingIndex, entityIn, relationSourceEntityIn, relationIn, containingGroupIn)
      }
      else if (answer == 7 && answer <= choices.length && !entityIsAlreadyTheDefault) {
        // updates user preferences such that this obj will be the one displayed by default in future.
        controller.mPrefs.putLong("first_display_entity", entityIn.getId)
        entityMenu(startingAttributeIndexIn, entityIn, relationSourceEntityIn, relationIn, containingGroupIn)
      }
      else if (answer == 8 && answer <= choices.length && !entityIn.isInstanceOf[RelationType]) {
        val editedEntity: Option[Entity] = controller.editEntityPublicStatus(entityIn)
        entityMenu(startingAttributeIndexIn, editedEntity.get, relationSourceEntityIn, relationIn, containingGroupIn)
      }
      else if (answer > choices.length && answer <= (choices.length + attributeObjList.size)) {
        // checking also for " && answer <= choices.length" because otherwise choosing 'a' returns 8 but if those optional menu choices were not added in,
        // then it is found among the first "choice" answers, instead of being adjusted later ("val attributeChoicesIndex = answer - choices.length - 1")
        // to find it among the "moreChoices" as it should be: would be thrown off by the optional choice numbering.
        goToSelectedAttribute(answer, choicesModified, attributeObjList, entityIn)
        entityMenu(startingAttributeIndexIn, entityIn, relationSourceEntityIn, relationIn, containingGroupIn)
      } else {
        ui.displayText("invalid response")
        entityMenu(startingAttributeIndexIn, entityIn, relationSourceEntityIn, relationIn, containingGroupIn)
      }
    }

  }
  catch {
    case e: Exception =>
      controller.handleException(e)
      val ans = ui.askYesNoQuestion("Go back to what you were doing (vs. going out)?",Some("y"))
      if (ans.isDefined && ans.get) entityMenu(startingAttributeIndexIn, entityIn, relationSourceEntityIn, relationIn, containingGroupIn)
      else None
  }

  def viewContainingGroups(entityIn: Entity): Option[Entity] = {
    val leadingText = List[String]("Pick from menu, or a letter to (go to if one or) see the entities containing that group, or Alt+<letter> for the actual " +
                                   "*group* by letter")
    val choices: Array[String] = Array(controller.listNextItemsPrompt)
    val numDisplayableItems = ui.maxColumnarChoicesToDisplayAfter(leadingText.size, choices.length, controller.maxNameLength)
    // (see comment in similar location just above where this is called)
    val containingRelationToGroups: util.ArrayList[RelationToGroup] = db.getContainingRelationToGroups(entityIn, 0,
                                                                                                       Some(numDisplayableItems))
    val containingRtgDescriptions: Array[String] = containingRelationToGroups.toArray.map {
                                                                                            case rtg: (RelationToGroup) =>
                                                                                              val entityName: String = new Entity(db,
                                                                                                                                  rtg.getParentId)
                                                                                                                       .getName
                                                                                              val rt: RelationType = new RelationType(db,
                                                                                                                                      rtg.getAttrTypeId)
                                                                                              "entity " + entityName + " " +
                                                                                              rtg.getDisplayString(controller.maxNameLength, None, Some(rt))
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
          entityMenu(0, containingEntity, None, None, Some(new Group(db, containingRelationToGroup.getGroupId)))
        } else {
          controller.chooseAmongEntities(containingEntities)
        }
      } else if (answer > choices.length && answer <= (choices.length + containingRelationToGroups.size) && userPressedAltKey) {
        // user typed a letter to select.. (now 0-based); selected a new object and so we return to the previous menu w/ that one displayed & current
        val entityId: Long = containingRelationToGroups.get(index).getParentId
        val groupId: Long = containingRelationToGroups.get(index).getGroupId
        val relTypeId: Long = containingRelationToGroups.get(index).getAttrTypeId
        new QuickGroupMenu(ui, db, controller).quickGroupMenu(0, new RelationToGroup(db, entityId, relTypeId, groupId), Some(entityIn))
      } else {
        ui.displayText("unknown response")
        None
      }
    }
  }

  def goToRelatedPlaces(startingAttributeIndexIn: Long, entityIn: Entity, relationSourceEntityIn: Option[Entity] = None,
                        relationIn: Option[RelationToEntity] = None, containingGroupIn: Option[Group] = None, classDefiningEntityId: Option[Long]): Option[Entity] = {
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
      containingRtg = Some(db.getContainingRelationToGroups(entityIn, 0, Some(1)).get(0))
      containingGroup = Some(new Group(db, containingRtg.get.getGroupId))
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


    val response = ui.askWhich(leadingText, choices, Array[String]())
    if (response.isDefined) {
      val goWhereAnswer = response.get
      if (goWhereAnswer == seeContainingEntities_choiceNumber && goWhereAnswer <= choices.length) {
        val leadingText = List[String]("Pick from menu, or an entity by letter")
        val choices: Array[String] = Array(controller.listNextItemsPrompt)
        val numDisplayableItems: Long = ui.maxColumnarChoicesToDisplayAfter(leadingText.size, choices.length, controller.maxNameLength)
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
        if (ans.isEmpty) None
        else {
          val answer = ans.get
          if (answer == 1 && answer <= choices.length) {
            // see comment above
            ui.displayText("not yet implemented")
          } else if (answer > choices.length && answer <= (choices.length + containingEntities.size)) {
            // those in the condition on the previous line are 1-based, not 0-based.
            val index = answer - choices.length - 1
            // user typed a letter to select.. (now 0-based); selected a new object and so we return to the previous menu w/ that one displayed & current
            val entity: Entity = containingEntities.get(index)._2
            entityMenu(0, entity)
          } else {
            ui.displayText("unknown response")
          }
          entityMenu(startingAttributeIndexIn, entityIn, relationSourceEntityIn, relationIn, containingGroupIn)
        }
      } else if (goWhereAnswer == seeContainingGroups_choiceNumber && goWhereAnswer <= choices.length) {
        if (numContainingGroups == 1) {
          new QuickGroupMenu(ui, db, controller).quickGroupMenu(0, containingRtg.get)
        } else {
          viewContainingGroups(entityIn)
        }
        entityMenu(startingAttributeIndexIn, entityIn, relationSourceEntityIn, relationIn, containingGroupIn)
      } else if (goWhereAnswer == goToRelation_choiceNumber && relationIn.isDefined && goWhereAnswer <= choices.length) {
        def dummyMethod(inDH: RelationToEntityDataHolder, inEditing: Boolean): Option[RelationToEntityDataHolder] = {
          Some(inDH)
        }
        def updateRelationToEntity(dhInOut: RelationToEntityDataHolder) {
          relationIn.get.update(Some(dhInOut.attrTypeId), dhInOut.validOnDate, Some(dhInOut.observationDate))
        }
        val relationToEntityDH: RelationToEntityDataHolder = new RelationToEntityDataHolder(relationIn.get.getAttrTypeId, relationIn.get.getValidOnDate,
                                                                                            relationIn.get.getObservationDate, relationIn.get
                                                                                                                               .getRelatedId1,
                                                                                            relationIn.get.getRelatedId2)
        controller.askForInfoAndUpdateAttribute[RelationToEntityDataHolder](relationToEntityDH, Controller.RELATION_TO_ENTITY_TYPE,
                                                                            "CHOOSE TYPE OF Relation to Entity:", dummyMethod, updateRelationToEntity)
        // force a reread from the DB so it shows the right info on the repeated menu:
        entityMenu(startingAttributeIndexIn, entityIn, relationSourceEntityIn, Some(new RelationToEntity(db, relationIn.get.getAttrTypeId,
                                                                                                         relationIn.get.getRelatedId1,
                                                                                                         relationIn.get.getRelatedId2)),
                   containingGroupIn)
      }
      else if (goWhereAnswer == goToRelationType_choiceNumber && relationIn.isDefined && goWhereAnswer <= choices.length) {
        entityMenu(0, new Entity(db, relationIn.get.getAttrTypeId))
        entityMenu(startingAttributeIndexIn, entityIn, relationSourceEntityIn, relationIn, containingGroupIn)
      }
      else if (goWhereAnswer == goToClassDefiningEntity_choiceNumber && classDefiningEntityId.isDefined && goWhereAnswer <= choices.length) {
        entityMenu(0, new Entity(db, classDefiningEntityId.get))
        entityMenu(startingAttributeIndexIn, entityIn, relationSourceEntityIn, relationIn, containingGroupIn)
      } else {
        ui.displayText("invalid response")
        entityMenu(startingAttributeIndexIn, entityIn, relationSourceEntityIn, relationIn, containingGroupIn)
      }
    } else {
      entityMenu(startingAttributeIndexIn, entityIn, relationSourceEntityIn, relationIn, containingGroupIn)
    }
  }

  def removeEntityReferenceFromGroup_Menu(entityIn: Entity, containingGroupIn: Option[Group]): Boolean = {
    val groupCount: Long = db.getCountOfGroupsContainingEntity(entityIn.getId)
    val (entityCountNonArchived, entityCountArchived) = db.getCountOfEntitiesContainingEntity(entityIn.getId)
    val ans = ui.askYesNoQuestion("REMOVE this entity from that group: ARE YOU SURE? (This isn't a deletion. It can still be found by searching, and in " + 
                                  (groupCount - 1) + " group(s), and associated directly with " +
                                  entityCountNonArchived + " other entity(ies) (and " + entityCountArchived + " archived entities)..")
    if (ans.isDefined && ans.get) {
      containingGroupIn.get.removeEntity(entityIn.getId)
      true

      //is it ever desirable to keep the next line instead of the 'None'? not in most typical usage it seems, but?:
      //entityMenu(startingAttributeIndexIn, entityIn, relationSourceEntityIn, relationIn)
    } else {
      ui.displayText("Did not remove entity from that group.", waitForKeystroke = false)
      false

      //is it ever desirable to keep the next line instead of the 'None'? not in most typical usage it seems, but?:
      //entityMenu(startingAttributeIndexIn, entityIn, relationSourceEntityIn, relationIn, containingGroupIn)
    }
  }

  def getLeadingText(leadingTextIn: Array[String], attributeObjListIn: java.util.ArrayList[Attribute],
                     entityIn: Entity, relationSourceEntityIn: Option[Entity] = None,
                     relationIn: Option[RelationToEntity] = None, containingGroupIn: Option[Group] = None): Array[String] = {
    leadingTextIn(0) = "**CURRENT ENTITY " + entityIn.getId + ": " + entityIn.getDisplayString
    if (relationIn.isDefined) {
      leadingTextIn(0) += ": found via relation: " + relationSourceEntityIn.get.getName + " " +
                          relationIn.get.getDisplayString(0, Some(new Entity(db, relationIn.get.getRelatedId2)),
                                                          Some(new RelationType(db, relationIn.get.getAttrTypeId)))
    }
    if (containingGroupIn.isDefined) {
      leadingTextIn(0) += ": found via group: " + containingGroupIn.get.getName
    }
    leadingTextIn(1) = if (attributeObjListIn.size == 0) "No attributes have been assigned to this object, yet."
    else "Attribute list menu: (or choose attribute by letter)"
    leadingTextIn
  }

  def getItemDisplayStrings(attributeObjListIn: java.util.ArrayList[Attribute]) = {
    val attributeNames: Array[String] = for (attribute: Attribute <- attributeObjListIn.toArray(Array[Attribute]())) yield attribute match {
      case relation: RelationToEntity =>
        val relationType = new RelationType(db, relation.getAttrTypeId)
        attribute.getDisplayString(controller.maxNameLength, Some(new Entity(db, relation.getRelatedId2)), Some(relationType))
      case relation: RelationToGroup =>
        val relationType = new RelationType(db, relation.getAttrTypeId)
        attribute.getDisplayString(controller.maxNameLength, None, Some(relationType))
      case _ => attribute.getDisplayString(controller.maxNameLength, None, None)
    }
    attributeNames
  }

  def addAttribute(startingAttributeIndexIn: Long, entityIn: Entity, relationSourceEntityIn: Option[Entity] = None,
                   relationIn: Option[RelationToEntity] = None, containingGroupIn: Option[Group] = None) {
    val whichKindOfAttribute =
      ui.askWhich(Some(Array("Choose which kind of attribute to add:")),
                  Array("quantity attribute (example: a numeric value like \"length\"",
                        "true/false value",
                        "date",
                        "text attribute (rare: usually prefer relations; but for example: a serial number, which is not subject to arithmetic)",
                        "Relation to entity (i.e., \"is near\" a microphone)",
                        "Relation to group (i.e., \"has\" a list/group)",

                        "external file (BUT CONSIDER FIRST ADDING AN ENTITY SPECIFICALLY FOR THE DOCUMENT SO IT CAN HAVE A DATE, OTHER ATTRS ETC.; " +
                        "AND ADDING THE DOCUMENT TO THAT ENTITY, SO IT CAN ALSO BE ASSOCIATED WITH OTHER ENTITIES EASILY!; also, " +
                        "given the concept behind OM, it's probably best" +
                        " to use this only for historical artifacts, or when you really can't fully model the data right now")
                 )
    if (whichKindOfAttribute.isDefined) {
      val whichKindAnswer = whichKindOfAttribute.get
      if (whichKindAnswer == 1) {
        def addQuantityAttribute(dhIn: QuantityAttributeDataHolder): Option[QuantityAttribute] = {
          Some(entityIn.addQuantityAttribute(dhIn.attrTypeId, dhIn.unitId, dhIn.number, dhIn.validOnDate, dhIn.observationDate))
        }
        controller.askForInfoAndAddAttribute[QuantityAttributeDataHolder](new QuantityAttributeDataHolder(0, None, 0, 0, 0), Controller.QUANTITY_TYPE,
                                                                          controller.quantityDescription,
                                                                          controller.askForQuantityAttributeNumberAndUnit, addQuantityAttribute)
      } else if (whichKindAnswer == 2) {
        def addBooleanAttribute(dhIn: BooleanAttributeDataHolder): Option[BooleanAttribute] = {
          Some(entityIn.addBooleanAttribute(dhIn.attrTypeId, dhIn.boolean))
        }
        controller.askForInfoAndAddAttribute[BooleanAttributeDataHolder](new BooleanAttributeDataHolder(0, Some(0), 0, false), Controller.BOOLEAN_TYPE,
                                                                         "SELECT TYPE OF TRUE/FALSE VALUE: ", controller.askForBooleanAttributeValue, addBooleanAttribute)
      } else if (whichKindAnswer == 3) {
        def addDateAttribute(dhIn: DateAttributeDataHolder): Option[DateAttribute] = {
          Some(entityIn.addDateAttribute(dhIn.attrTypeId, dhIn.date))
        }
        controller.askForInfoAndAddAttribute[DateAttributeDataHolder](new DateAttributeDataHolder(0, 0), Controller.DATE_TYPE,
                                                                      "SELECT TYPE OF DATE: ", controller.askForDateAttributeValue, addDateAttribute)
      } else if (whichKindAnswer == 4) {
        def addTextAttribute(dhIn: TextAttributeDataHolder): Option[TextAttribute] = {
          Some(entityIn.addTextAttribute(dhIn.attrTypeId, dhIn.text, dhIn.validOnDate, dhIn.observationDate))
        }
        controller.askForInfoAndAddAttribute[TextAttributeDataHolder](new TextAttributeDataHolder(0, Some(0), 0, ""), Controller.TEXT_TYPE,
                                                                      "SELECT TYPE OF " + controller.textDescription + ": ", controller.askForTextAttributeText, addTextAttribute)
      } else if (whichKindAnswer == 5) {
        def addRelationToEntity(dhIn: RelationToEntityDataHolder): Option[RelationToEntity] = {
          Some(entityIn.addRelationToEntity(dhIn.attrTypeId, dhIn.entityId1, dhIn.entityId2, dhIn.validOnDate, dhIn.observationDate))
        }
        controller.askForInfoAndAddAttribute[RelationToEntityDataHolder](new RelationToEntityDataHolder(0, None, 0, entityIn.getId, 0), Controller.RELATION_TYPE_TYPE,
                                                                         "CREATE OR SELECT RELATION TYPE: (" + controller.mRelTypeExamples + ")",
                                                                         controller.askForRelationEntityIdNumber2, addRelationToEntity)
      } else if (whichKindAnswer == 6) {
        def addRelationToGroup(dhIn: RelationToGroupDataHolder): Option[RelationToGroup] = {
          entityIn.addRelationToGroup(dhIn.attrTypeId, dhIn.groupId, dhIn.validOnDate, dhIn.observationDate)
          Some(new RelationToGroup(db, dhIn.entityId, dhIn.attrTypeId, dhIn.groupId))
        }
        val result: Option[Attribute] = controller.askForInfoAndAddAttribute[RelationToGroupDataHolder](new RelationToGroupDataHolder(entityIn.getId, 0, 0, None,
                                                                                                                                      System.currentTimeMillis()),
                                                                                                        Controller.RELATION_TYPE_TYPE,
                                                                                                        "CREATE OR SELECT RELATION TYPE: (" + controller.mRelTypeExamples + ")" +
                                                                                                        "." + TextUI.NEWLN + "(Does anyone see a specific " +
                                                                                                        "reason to keep asking for these dates?)",
                                                                                                        controller.askForRelToGroupInfo, addRelationToGroup)
        if (result.isEmpty) entityMenu(startingAttributeIndexIn, entityIn, relationSourceEntityIn, relationIn, containingGroupIn)
        else new GroupMenu(ui, db, controller).groupMenu(0, result.get.asInstanceOf[RelationToGroup], None, Some(entityIn))
      } else if (whichKindAnswer == 7) {
        def addFileAttribute(dhIn: FileAttributeDataHolder): Option[FileAttribute] = {
          Some(entityIn.addFileAttribute(dhIn.attrTypeId, dhIn.description, new java.io.File(dhIn.originalFilePath)))
        }
        val result: Option[FileAttribute] = controller.askForInfoAndAddAttribute[FileAttributeDataHolder](new FileAttributeDataHolder(0, "", ""), Controller.FILE_TYPE,
                                                                                                          "SELECT TYPE OF FILE: ", controller.askForFileAttributeInfo,
                                                                                                          addFileAttribute).asInstanceOf[Option[FileAttribute]]
        if (result.isDefined) {
          val ans = ui.askYesNoQuestion("Document successfully added. Do you want to DELETE the local copy (at " + result.get.getOriginalFilePath + " ?")
          if (ans.isDefined && ans.get) {
            if (! new File(result.get.getOriginalFilePath).delete()) {
              ui.displayText("Unable to delete file at that location; reason unknown.  You could check the permissions.")
            }
          }
        }
      } else {
        ui.displayText("invalid response")
      }
    }
  }

  def listNextAttributes(attributeObjList: java.util.ArrayList[Attribute],
                         startingAttributeIndexIn: Long, numAttrsInEntity: Long): Long = {
    val startingIndex = {
      val currentPosition = startingAttributeIndexIn + attributeObjList.size
      if (currentPosition >= numAttrsInEntity) {
        ui.displayText("End of attribute list found; restarting from the beginning.")
        0 // start over
      } else currentPosition

    }
    startingIndex
  }

  def goToSelectedAttribute(answer: Int, choicesIn: Array[String], attributeObjListIn: java.util.ArrayList[Attribute], entityIn: Entity) {
    // attributeChoicesIndexIn is 1-based, not 0-based.
    val attributeChoicesIndex = answer - choicesIn.length - 1
    // user typed a letter to select an attribute (now 0-based)
    if (attributeChoicesIndex >= attributeObjListIn.size()) {
      ui.displayText("The program shouldn't have let us get to this point, but the selection " + answer + " is not in the list.")
    } else {
      val o: Attribute = attributeObjListIn.get(attributeChoicesIndex)
      o match {
        //idea: there's probably also some more scala-like cleaner syntax 4 this, as elsewhere:
        case qa: QuantityAttribute => controller.attributeEditMenu(qa)
        case ta: TextAttribute => controller.attributeEditMenu(ta)
        case relToEntity: RelationToEntity => entityMenu(0, new Entity(db, relToEntity.getRelatedId2), Some(entityIn), Some(relToEntity))
        case relToGroup: RelationToGroup => new QuickGroupMenu(ui, db, controller).quickGroupMenu(0, relToGroup, containingEntityIn = Some(entityIn))
        case da: DateAttribute => controller.attributeEditMenu(da)
        case ba: BooleanAttribute => controller.attributeEditMenu(ba)
        case fa: FileAttribute => controller.attributeEditMenu(fa)
        case _ => throw new Exception("Unexpected choice has class " + o.getClass.getName + "--what should we do here?")
      }
    }
  }

}
