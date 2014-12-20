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
import org.onemodel._
import org.onemodel.model._
import org.onemodel.database.PostgreSQLDatabase

class EntityMenu(override val ui: TextUI, dbInOVERRIDESmDBWhichHasANewDbConnectionTHATWEDONTWANT: PostgreSQLDatabase) extends Controller(ui) {
  override val mDB = dbInOVERRIDESmDBWhichHasANewDbConnectionTHATWEDONTWANT

  /** returns None if user wants out. */
  //@tailrec //removed for now until the compiler can handle it with where the method calls itself.
  //idea: make this limited like this somehow?:  private[org.onemodel] ... Same for all others like it?
  def entityMenu(startingAttributeIndexIn: Long, entityIn: Entity, relationSourceEntityIn: Option[Entity] = None,
                           relationIn: Option[RelationToEntity] = None, containingGroupIn: Option[Group] = None): Option[Entity] = try {
    require(entityIn != null)
    val numAttrsInEntity: Long = entityIn.getAttrCount
    val classDefiningEntityId: Option[Long] = entityIn.getClassDefiningEntityId

    def getLeadingText(leadingTextIn: Array[String], attributeObjListIn: java.util.ArrayList[Attribute]): Array[String] = {
      leadingTextIn(0) = "**CURRENT ENTITY:" + entityIn.getDisplayString
      if (relationIn != None) {
        leadingTextIn(0) += ": found via relation: " + relationSourceEntityIn.get.getName + " " +
                            relationIn.get.getDisplayString(0, Some(new Entity(mDB, relationIn.get.getRelatedId2)),
                                                            Some(new RelationType(mDB, relationIn.get.getAttrTypeId)))
      }
      if (containingGroupIn != None) {
        leadingTextIn(0) += ": found via group: " + containingGroupIn.get.getName
      }
      leadingTextIn(1) = if (attributeObjListIn.size == 0) "No attributes have been assigned to this object, yet."
                         else "Attribute list menu: (or choose attribute by letter)"
      leadingTextIn
    }

    def getItemDisplayStrings(attributeObjListIn: java.util.ArrayList[Attribute]) = {
      val attributeNames: Array[String] = for (attribute: Attribute <- attributeObjListIn.toArray(Array[Attribute]())) yield attribute match {
        case relation: RelationToEntity =>
          val relationType = new RelationType(mDB, relation.getAttrTypeId)
          attribute.getDisplayString(maxNameLength, Some(new Entity(mDB, relation.getRelatedId2)), Some(relationType))
        case relation: RelationToGroup =>
          val relationType = new RelationType(mDB, relation.getAttrTypeId)
          attribute.getDisplayString(maxNameLength, None, Some(relationType))
        case _ => attribute.getDisplayString(maxNameLength, None, None)
      }
      attributeNames
    }

    def addAttribute() {
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
      if (whichKindOfAttribute != None) {
        val whichKindAnswer = whichKindOfAttribute.get
        if (whichKindAnswer == 1) {
          def addQuantityAttribute(dhIn: QuantityAttributeDataHolder): Option[QuantityAttribute] = {
            Some(entityIn.addQuantityAttribute(dhIn.attrTypeId, dhIn.unitId, dhIn.number, dhIn.validOnDate, dhIn.observationDate))
          }
          askForInfoAndAddAttribute[QuantityAttributeDataHolder](new QuantityAttributeDataHolder(0, None, 0, 0, 0), Controller.QUANTITY_TYPE,
                                                                 quantityDescription,
                                                                 askForQuantityAttributeNumberAndUnit, addQuantityAttribute)
        } else if (whichKindAnswer == 2) {
          def addBooleanAttribute(dhIn: BooleanAttributeDataHolder): Option[BooleanAttribute] = {
            Some(entityIn.addBooleanAttribute(dhIn.attrTypeId, dhIn.boolean))
          }
          askForInfoAndAddAttribute[BooleanAttributeDataHolder](new BooleanAttributeDataHolder(0, Some(0), 0, false), Controller.BOOLEAN_TYPE,
                                                                "SELECT TYPE OF TRUE/FALSE VALUE: ", askForBooleanAttributeValue, addBooleanAttribute)
        } else if (whichKindAnswer == 3) {
          def addDateAttribute(dhIn: DateAttributeDataHolder): Option[DateAttribute] = {
            Some(entityIn.addDateAttribute(dhIn.attrTypeId, dhIn.date))
          }
          askForInfoAndAddAttribute[DateAttributeDataHolder](new DateAttributeDataHolder(0, 0), Controller.DATE_TYPE,
                                                             "SELECT TYPE OF DATE: ", askForDateAttributeValue, addDateAttribute)
        } else if (whichKindAnswer == 4) {
          def addTextAttribute(dhIn: TextAttributeDataHolder): Option[TextAttribute] = {
            Some(entityIn.addTextAttribute(dhIn.attrTypeId, dhIn.text, dhIn.validOnDate, dhIn.observationDate))
          }
          askForInfoAndAddAttribute[TextAttributeDataHolder](new TextAttributeDataHolder(0, Some(0), 0, ""), Controller.TEXT_TYPE,
                                                             "SELECT TYPE OF " + textDescription + ": ", askForTextAttributeText, addTextAttribute)
        } else if (whichKindAnswer == 5) {
          def addRelationToEntity(dhIn: RelationToEntityDataHolder): Option[RelationToEntity] = {
            Some(entityIn.addRelationToEntity(dhIn.attrTypeId, dhIn.entityId1, dhIn.entityId2, dhIn.validOnDate, dhIn.observationDate))
          }
          askForInfoAndAddAttribute[RelationToEntityDataHolder](new RelationToEntityDataHolder(0, None, 0, entityIn.getId, 0), Controller.RELATION_TYPE_TYPE,
                                                                "CREATE OR SELECT RELATION TYPE: (" + mRelTypeExamples + ")",
                                                                askForRelationEntityIdNumber2, addRelationToEntity)
        } else if (whichKindAnswer == 6) {
          def addRelationToGroup(dhIn: RelationToGroupDataHolder): Option[RelationToGroup] = {
            entityIn.addRelationToGroup(dhIn.attrTypeId, dhIn.groupId, dhIn.validOnDate, dhIn.observationDate)
            Some(new RelationToGroup(mDB, dhIn.entityId, dhIn.attrTypeId, dhIn.groupId))
          }
          val result: Option[Attribute] = askForInfoAndAddAttribute[RelationToGroupDataHolder](new RelationToGroupDataHolder(entityIn.getId, 0, 0, None,
                                                                                                                             System.currentTimeMillis()),
                                                                                               Controller.RELATION_TYPE_TYPE,
                                                                                               "CREATE OR SELECT RELATION TYPE: (" + mRelTypeExamples + ")" +
                                                                                               "." + TextUI.NEWLN + "(Does anyone see a specific " +
                                                                                               "reason to keep asking for these dates?)",
                                                                                               askForRelToGroupInfo, addRelationToGroup)
          if (result == None) entityMenu(startingAttributeIndexIn, entityIn, relationSourceEntityIn, relationIn, containingGroupIn)
          else new GroupMenu(ui, mDB).groupMenu(0, result.get.asInstanceOf[RelationToGroup])
        } else if (whichKindAnswer == 7) {
          def addFileAttribute(dhIn: FileAttributeDataHolder): Option[FileAttribute] = {
            Some(entityIn.addFileAttribute(dhIn.attrTypeId, dhIn.description, new java.io.File(dhIn.originalFilePath)))
          }
          val result: Option[FileAttribute] = askForInfoAndAddAttribute[FileAttributeDataHolder](new FileAttributeDataHolder(0, "", ""), Controller.FILE_TYPE,
                                                                                                 "SELECT TYPE OF FILE: ", askForFileAttributeInfo,
                                                                                                 addFileAttribute).asInstanceOf[Option[FileAttribute]]
          if (result != None) {
            val ans = ui.askYesNoQuestion("Document successfully added. Do you want to DELETE the local copy (at " + result.get.getOriginalFilePath + " ?")
            if (ans != None && ans.get) {
              new File(result.get.getOriginalFilePath).delete()
            }
          }
        } else {
          ui.displayText("invalid response")
        }
      }
    }

    def listNextAttributes(attributeObjList: java.util.ArrayList[Attribute]): Long = {
      val startingIndex = {
        val currentPosition = startingAttributeIndexIn + attributeObjList.size
        if (currentPosition >= numAttrsInEntity) {
          ui.displayText("End of attribute list found; restarting from the beginning.")
          0 // start over
        } else currentPosition

      }
      startingIndex
    }

    def goToSelectedAttribute(answer: Int, choicesIn: Array[String], attributeObjListIn: java.util.ArrayList[Attribute]) {
      // attributeChoicesIndexIn is 1-based, not 0-based.
      val attributeChoicesIndex = answer - choicesIn.length - 1
      // user typed a letter to select an attribute (now 0-based)
      if (attributeChoicesIndex >= attributeObjListIn.size()) {
        ui.displayText("The program shouldn't have let us get to this point, but the selection " + answer + " is not in the list.")
      } else {
        val o: Attribute = attributeObjListIn.get(attributeChoicesIndex)
        o match {
          //idea: there's probably also some more scala-like cleaner syntax 4 this, as elsewhere:
          case qa: QuantityAttribute => attributeEditMenu(qa)
          case ta: TextAttribute => attributeEditMenu(ta)
          case relToEntity: RelationToEntity => entityMenu(0, new Entity(mDB, relToEntity.getRelatedId2), Some(entityIn), Some(relToEntity))
          case relToGroup: RelationToGroup => new QuickGroupMenu(ui,mDB).quickGroupMenu(0, relToGroup)
          case da: DateAttribute => attributeEditMenu(da)
          case ba: BooleanAttribute => attributeEditMenu(ba)
          case fa: FileAttribute => attributeEditMenu(fa)
          case _ => throw new Exception("Unexpected choice has class " + o.getClass.getName + "--what should we do here?")
        }
      }
    }

    // 2nd return value is whether entityIsDefault (ie whether default object when launching OM is already this entity)
    def getChoices: (Array[String], Boolean) = {
      // (idea: might be a little silly to do it this way, once this # gets very big?:)
      var choices = Array[String]("Add attribute (quantity, true/false, date, text, external file, relation to entity or group: " + mRelTypeExamples + ")...",
                                  "Import/Export...",
                                  "Edit name",
                                  "Delete or Archive...",
                                  "Go to...",
                                  listNextItemsPrompt)
      if (relationIn != None) {
        // means we got here by selecting a Relation attribute on another entity, so entityIn is the "entityId2" in that relation; so show some options, because
        // we eliminated a separate menu just for the relation and put them here, for UI usage simplicity.
        require(relationIn.get.getRelatedId2 == entityIn.getId && relationSourceEntityIn != None)
      }

      val defaultEntity: Option[Long] = findDefaultDisplayEntity
      //  don't show the "set default" option if it's already been done w/ this same one:
      val entityIsAlreadyTheDefault: Boolean = defaultEntity != None && defaultEntity.get == entityIn.getId
      if (! entityIsAlreadyTheDefault) {
        choices = choices :+ ((if (defaultEntity == None) "****TRY ME---> " else "") +
                              "Set current entity (" + entityIn.getDisplayString + ") as default (first to come up when launching this program.)")
      } else choices = choices :+ "(stub)"
      choices = choices :+ "Edit public/nonpublic status"
      (choices, entityIsAlreadyTheDefault)
    }


    val leadingText: Array[String] = new Array[String](2)
    val (choices: Array[String], entityIsAlreadyTheDefault: Boolean) = getChoices
    val numDisplayableAttributes = ui.maxColumnarChoicesToDisplayAfter(leadingText.length, choices.size, maxNameLength)
    val (attributeObjList: java.util.ArrayList[Attribute], totalRowsAvailable) =
      mDB.getSortedAttributes(entityIn.getId, startingAttributeIndexIn, numDisplayableAttributes)
    val choicesModified = addRemainingCountToPrompt(choices, attributeObjList.size, totalRowsAvailable, startingAttributeIndexIn)
    val leadingTextModified = getLeadingText(leadingText, attributeObjList)
    val attributeDisplayStrings: Array[String] = getItemDisplayStrings(attributeObjList)


    val response = ui.askWhich(Some(leadingTextModified), choicesModified, attributeDisplayStrings)
    if (response == None) None
    else {
      val answer = response.get
      if (answer == 1) {
        addAttribute()
        entityMenu(startingAttributeIndexIn, entityIn, relationSourceEntityIn, relationIn, containingGroupIn)
      }
      if (answer == 2) {
        val importOrExport = ui.askWhich(None, Array("Import", "Export"), Array[String]())
        if (importOrExport != None) {
          if (importOrExport.get == 1) new ImportExport(ui, mDB).importCollapsibleOutlineAsGroups(entityIn)
          else if (importOrExport.get == 2) new ImportExport(ui, mDB).exportToCollapsibleOutline(entityIn)
        }
        entityMenu(startingAttributeIndexIn, entityIn, relationSourceEntityIn, relationIn, containingGroupIn)
      }
      else if (answer == 3) {
        val editedEntity: Option[Entity] = editEntityName(entityIn)
        entityMenu(startingAttributeIndexIn, if (editedEntity != None) editedEntity.get else entityIn, relationSourceEntityIn, relationIn, containingGroupIn)
      }
      else if (answer == 4) {
        val (delOrArchiveAnswer, delLinkingRelation_choiceNumber, delFromContainingGroup_choiceNumber) =
          askWhetherDeleteOrArchiveEtc(entityIn, relationIn, relationSourceEntityIn, containingGroupIn)

        if (delOrArchiveAnswer != None) {
          val answer = delOrArchiveAnswer.get
          if (answer == 1 || answer == 2) {
            val thisEntityWasDeletedOrArchived = deleteOrArchiveEntity(entityIn, answer == 1)
            if (thisEntityWasDeletedOrArchived) None
            else entityMenu(startingAttributeIndexIn, entityIn, relationSourceEntityIn, relationIn, containingGroupIn)
          } else if (answer == delLinkingRelation_choiceNumber && relationIn != None && answer <= choices.size) {
            val ans = ui.askYesNoQuestion("DELETE the relation: ARE YOU SURE?")
            if (ans != None && ans.get) {
              relationIn.get.delete()
              None
            } else {
              ui.displayText("Did not delete relation.", waitForKeystroke = false)
              entityMenu(startingAttributeIndexIn, entityIn, relationSourceEntityIn, relationIn, containingGroupIn)
            }
          } else if (answer == delFromContainingGroup_choiceNumber && containingGroupIn != None && answer <= choices.size) {
            val ans = ui.askYesNoQuestion("REMOVE this entity from that group: ARE YOU SURE?")
            if (ans != None && ans.get) {
              containingGroupIn.get.removeEntity(entityIn.getId)
              entityMenu(startingAttributeIndexIn, entityIn, relationSourceEntityIn, relationIn)
            } else {
              ui.displayText("Did not remove entity from that group.", waitForKeystroke = false)
              entityMenu(startingAttributeIndexIn, entityIn, relationSourceEntityIn, relationIn, containingGroupIn)
            }
          } else {
            ui.displayText("invalid response")
            entityMenu(startingAttributeIndexIn, entityIn, relationSourceEntityIn, relationIn, containingGroupIn)
          }
        } else {
          entityMenu(startingAttributeIndexIn, entityIn, relationSourceEntityIn, relationIn, containingGroupIn)
        }
      } else if (answer == 5) {
        //%%:
        //idea: make this and similar locations share code? What other places could?? There is plenty of duplicated code here!
        val leadingText = Some(Array("Go to..."))
        val seeContainingEntities_choiceNumber: Int = 1
        val seeContainingGroups_choiceNumber: Int = 2
        val goToRelation_choiceNumber: Int = 3
        val goToRelationType_choiceNumber: Int = 4
        var goToClassDefiningEntity_choiceNumber: Int = 3
        val numContainingEntities = mDB.getContainingEntities1(entityIn, 0).size
        // (idea: make this next call efficient: now it builds them all when we just want a count; but is infrequent & likely small numbers)
        val numContainingGroups = mDB.getCountOfGroupsContainingEntity(entityIn.getId)
        var containingGroup: Option[Group] = None
        var containingRtg: Option[RelationToGroup] = None
        if (numContainingGroups == 1) {
          containingRtg = Some(mDB.getContainingRelationToGroups(entityIn, 0, Some(1)).get(0))
          containingGroup = Some(new Group(mDB, containingRtg.get.getGroupId))
        }

        var choices = Array[String]("See entities that directly relate to this entity ( " + numContainingEntities + ")",
                                    if (numContainingGroups == 1) {
                                      "Go to group containing this entity: " + containingGroup.get.getName
                                    } else {
                                      "See groups containing this entity (" + numContainingGroups + ")"
                                    })

        if (relationIn != None) {
          choices = choices :+ "Go edit the relation to entity that that led here: " +
                               relationIn.get.getDisplayString(15, relationSourceEntityIn, Some(new RelationType(mDB, relationIn.get.getAttrTypeId)))
          choices = choices :+ "Go to the type, for the relation that that led here: " + new Entity(mDB, relationIn.get.getAttrTypeId).getName
          goToClassDefiningEntity_choiceNumber += 2
        }
        if (classDefiningEntityId != None) {
          choices = choices ++ Array[String]("Go to class-defining entity")
        }
        val goToWhereAnswer = ui.askWhich(leadingText, choices, Array[String]())
        if (goToWhereAnswer != None) {
          val answer = goToWhereAnswer.get
          if (answer == seeContainingEntities_choiceNumber && answer <= choices.size) {
            val leadingText = List[String]("Pick from menu, or an entity by letter")
            val choices: Array[String] = Array(listNextItemsPrompt)
            val numDisplayableItems: Long = ui.maxColumnarChoicesToDisplayAfter(leadingText.size, choices.size, maxNameLength)
            // This is partly set up so it could handle multiple screensful, but would need to be broken into a recursive method that
            // can specify dif't values on each call, for the startingIndexIn parm of getRelatingEntities.  I.e., could make it look more like
            // searchForExistingObject or such ? IF needed.  But to be needed means the user is putting the same object related by multiple
            // entities: enough to fill > 1 screen when listed.
            val containingEntities: java.util.ArrayList[(Long, Entity)] = mDB.getContainingEntities1(entityIn, 0, Some(numDisplayableItems))
            val containingEntitiesNames: Array[String] = containingEntities.toArray.map {
                                                                                          case relTypeIdAndEntity: (Long, Entity) =>
                                                                                            val entity: Entity = relTypeIdAndEntity._2
                                                                                            entity.getName
                                                                                          case _ => throw new OmException("??")
                                                                                        }
            val ans = ui.askWhich(Some(leadingText.toArray), choices, containingEntitiesNames)
            if (ans == None) None
            else {
              val answer = ans.get
              if (answer == 1 && answer <= choices.size) {
                // see comment above
                ui.displayText("not yet implemented") //%%
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
          } else if (answer == seeContainingGroups_choiceNumber && answer <= choices.size) {
            if (numContainingGroups == 1) {
              new QuickGroupMenu(ui,mDB).quickGroupMenu(0, containingRtg.get)
            } else {
              val leadingText = List[String]("Pick from menu, or a letter to (go to if one or) see the entities containing that group, or Alt+<letter> for the actual *group* by letter")
              val choices: Array[String] = Array(listNextItemsPrompt)
              val numDisplayableItems = ui.maxColumnarChoicesToDisplayAfter(leadingText.size, choices.size, maxNameLength)
              // (see comment in similar location just above)
              val containingRelationToGroups: java.util.ArrayList[RelationToGroup] = mDB.getContainingRelationToGroups(entityIn, 0,
                                                                                                                       Some(numDisplayableItems))
              val containingRtgDescriptions: Array[String] = containingRelationToGroups.toArray.map {
                                                                                                      case rtg: (RelationToGroup) =>
                                                                                                        val entityName: String = new Entity(mDB,
                                                                                                                                            rtg.getParentId)
                                                                                                                                 .getName
                                                                                                        val rt: RelationType = new RelationType(mDB,
                                                                                                                                                rtg.getAttrTypeId)
                                                                                                        "entity " + entityName + " " +
                                                                                                        rtg.getDisplayString(maxNameLength, None, Some(rt))
                                                                                                      case _ => throw new OmException("??")
                                                                                                    }

              val ans = ui.askWhichChoiceOrItsAlternate(Some(leadingText.toArray), choices, containingRtgDescriptions)
              if (ans == None) None
              else {
                val (answer, userChoseAlternate: Boolean) = ans.get
                // those in the condition on the previous line are 1-based, not 0-based.
                val index = answer - choices.length - 1
                if (answer == 1 && answer <= choices.size) {
                  // see comment above
                  ui.displayText("not yet implemented") //%%
                } else if (answer > choices.length && answer <= (choices.length + containingRelationToGroups.size) && !userChoseAlternate) {
                  val containingRelationToGroup = containingRelationToGroups.get(index)
                  val containingEntities = mDB.getContainingEntities2(containingRelationToGroup, 0)
                  val numContainingEntities = containingEntities.size
                  if (numContainingEntities == 1) {
                    val containingEntity = containingEntities.get(0)._2
                    entityMenu(0, containingEntity, None, None, Some(new Group(mDB, containingRelationToGroup.getGroupId)))
                  } else {
                    chooseAmongEntities(containingEntities)
                  }
                } else if (answer > choices.length && answer <= (choices.length + containingRelationToGroups.size) && userChoseAlternate) {
                  // user typed a letter to select.. (now 0-based); selected a new object and so we return to the previous menu w/ that one displayed & current
                  val entityId: Long = containingRelationToGroups.get(index).getParentId
                  val groupId: Long = containingRelationToGroups.get(index).getGroupId
                  val relTypeId: Long = containingRelationToGroups.get(index).getAttrTypeId
                  new QuickGroupMenu(ui,mDB).quickGroupMenu(0, new RelationToGroup(mDB, entityId, relTypeId, groupId), Some(entityIn))
                } else {
                  ui.displayText("unknown response")
                }
              }
            }
            entityMenu(startingAttributeIndexIn, entityIn, relationSourceEntityIn, relationIn, containingGroupIn)
          } else if (answer == goToRelation_choiceNumber && relationIn != None && answer <= choices.size) {
            def dummyMethod(inDH: RelationToEntityDataHolder, inEditing: Boolean): Option[RelationToEntityDataHolder] = {
              Some(inDH)
            }
            def updateRelationToEntity(dhInOut: RelationToEntityDataHolder) {
              relationIn.get.update(Some(dhInOut.attrTypeId), dhInOut.validOnDate, Some(dhInOut.observationDate))
            }
            val relationToEntityDH: RelationToEntityDataHolder = new RelationToEntityDataHolder(relationIn.get.getAttrTypeId, relationIn.get.getValidOnDate,
                                                                                                relationIn.get.getObservationDate, relationIn.get.getRelatedId1,
                                                                                                relationIn.get.getRelatedId2)
            askForInfoAndUpdateAttribute[RelationToEntityDataHolder](relationToEntityDH, Controller.RELATION_TO_ENTITY_TYPE,
                                                                     "CHOOSE TYPE OF Relation to Entity:",
                                                                     dummyMethod, updateRelationToEntity)
            //force a reread from the DB so it shows the right info on the repeated menu:
            entityMenu(startingAttributeIndexIn, entityIn, relationSourceEntityIn, Some(new RelationToEntity(mDB, relationIn.get.getAttrTypeId,
                                                                                                             relationIn.get.getRelatedId1,
                                                                                                             relationIn.get.getRelatedId2)), containingGroupIn)
          }
          else if (answer == goToRelationType_choiceNumber && relationIn != None && answer <= choices.size) {
            entityMenu(0, new Entity(mDB, relationIn.get.getAttrTypeId))
            entityMenu(startingAttributeIndexIn, entityIn, relationSourceEntityIn, relationIn, containingGroupIn)
          }
          else if (answer == goToClassDefiningEntity_choiceNumber && classDefiningEntityId != None && answer <= choices.size) {
            entityMenu(0, new Entity(mDB, classDefiningEntityId.get))
            entityMenu(startingAttributeIndexIn, entityIn, relationSourceEntityIn, relationIn, containingGroupIn)
          } else {
            ui.displayText("invalid response")
            entityMenu(startingAttributeIndexIn, entityIn, relationSourceEntityIn, relationIn, containingGroupIn)
          }
        } else {
          entityMenu(startingAttributeIndexIn, entityIn, relationSourceEntityIn, relationIn, containingGroupIn)
        }
      }
      else if (answer == 6) {
        val startingIndex: Long = listNextAttributes(attributeObjList)
        entityMenu(startingIndex, entityIn, relationSourceEntityIn, relationIn, containingGroupIn)
      }
      else if (answer == 7 && answer <= choices.size && !entityIsAlreadyTheDefault) {
        // updates user preferences such that this obj will be the one displayed by default in future.
        mPrefs.putLong("first_display_entity", entityIn.getId)
        entityMenu(startingAttributeIndexIn, entityIn, relationSourceEntityIn, relationIn, containingGroupIn)
      }
      else if (answer == 8 && answer <= choices.size && !entityIn.isInstanceOf[RelationType]) {
        val editedEntity: Option[Entity] = editEntityPublicStatus(entityIn)
        entityMenu(startingAttributeIndexIn, editedEntity.get, relationSourceEntityIn, relationIn, containingGroupIn)
      }
      else if (answer > choices.length && answer <= (choices.length + attributeObjList.size)) {
        // checking also for " && answer <= choices.length" because otherwise choosing 'a' returns 8 but if those optional menu choices were not added in,
        // then it is found among the first "choice" answers, instead of being adjusted later ("val attributeChoicesIndex = answer - choices.length - 1")
        // to find it among the "moreChoices" as it should be: would be thrown off by the optional choice numbering.
        goToSelectedAttribute(answer, choicesModified, attributeObjList)
        entityMenu(startingAttributeIndexIn, entityIn, relationSourceEntityIn, relationIn, containingGroupIn)
      } else {
        ui.displayText("invalid response")
        entityMenu(startingAttributeIndexIn, entityIn, relationSourceEntityIn, relationIn, containingGroupIn)
      }
    }

  }
  catch {
    case e: Exception =>
      showException(e)
      val ans = ui.askYesNoQuestion("Go back to what you were doing (vs. going out)?",Some("y"))
      if (ans != None && ans.get) entityMenu(startingAttributeIndexIn, entityIn, relationSourceEntityIn, relationIn, containingGroupIn)
      else None
  }
}
