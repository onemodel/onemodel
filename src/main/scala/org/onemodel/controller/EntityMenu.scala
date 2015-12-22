/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003-2004 and 2008-2015 inclusive, Luke A. Call; all rights reserved.
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
import org.onemodel.database.PostgreSQLDatabase
import org.onemodel.model._

class EntityMenu(override val ui: TextUI, override val db: PostgreSQLDatabase, val controller: Controller) extends SortableEntriesMenu(ui, db) {
  // 2nd return value is whether entityIsDefault (ie whether default object when launching OM is already this entity)
  def getChoices(entityIn: Entity, numAttrsIn: Long, relationSourceEntityIn: Option[Entity] = None,
                 relationIn: Option[RelationToEntity] = None): Array[String] = {
    // (idea: might be a little silly to do it this way, once this # gets very big?:)
    var choices = Array[String]("Add entry quickly (creates a \"has\" relation to a new Entity)",

                                if (numAttrsIn > 1) "Move selection (*) up/down" else "(stub)",

                                "[app will fill this one in just a bit later, at \"choices (3) = \" below.  KEEP IN THIS RELATIVE POSITION OR CHANGE THE" +
                                " CODE NEAR THE TOP OF entityMenu THAT CHECKS FOR A VALUE IN highlightedAttributeIn]",

                                "Add attribute (add entry with detailed options)",

                                "Go to selected attribute",
                                if (numAttrsIn > 0) controller.listNextItemsPrompt else "(stub)")
    if (relationIn.isDefined) {
      // means we got here by selecting a Relation attribute on another entity, so entityIn is the "entityId2" in that relation; so show some options, because
      // we eliminated a separate menu just for the relation and put them here, for UI usage simplicity.
      require(relationIn.get.getRelatedId2 == entityIn.getId && relationSourceEntityIn.isDefined)
    }
    choices = choices :+ "(stub)"
    choices = choices :+ (if (numAttrsIn > 0) "Select attribute to highlight (with '*'; typing the letter instead goes to that attribute's menu)" else "(stub)")
    choices = choices :+ (if (controller.findDefaultDisplayEntity.isEmpty) "****TRY ME---> " else "") + "Other entity operations..."
    choices
  }

  /** The parameter attributeRowsStartingIndexIn means: of all the sorted attributes of entityIn, which one is to be displayed first (since we can only display
    * so many at a time with finite screen size).
   * Returns None if user wants out.
   * */
  //@tailrec //removed for now until the compiler can handle it with where the method calls itself.
  //idea on scoping: make this limited like this somehow?:  private[org.onemodel] ... Same for all others like it?
  def entityMenu(entityIn: Entity, attributeRowsStartingIndexIn: Int = 0, highlightedAttributeIn: Option[Attribute] = None,
                 relationSourceEntityIn: Option[Entity] = None,
                 relationIn: Option[RelationToEntity] = None, containingGroupIn: Option[Group] = None): Option[Entity] = try {
    require(entityIn != null)
    val numAttrsInEntity: Long = entityIn.getAttrCount
    val classDefiningEntityId: Option[Long] = entityIn.getClassDefiningEntityId
    val leadingText: Array[String] = new Array[String](2)
    val choices: Array[String] = getChoices(entityIn, numAttrsInEntity, relationSourceEntityIn, relationIn)
    val numDisplayableAttributes: Int = ui.maxColumnarChoicesToDisplayAfter(leadingText.length, choices.length, controller.maxNameLength)
    val (attributeTuples: Array[(Long, Attribute)], totalAttrsAvailable: Int) =
      db.getSortedAttributes(entityIn.getId, attributeRowsStartingIndexIn, numDisplayableAttributes)
    if (numAttrsInEntity > 0 || attributeTuples.length > 0) require(numAttrsInEntity > 0 && attributeTuples.length > 0)
    require(totalAttrsAvailable == numAttrsInEntity)
    val choicesModified = controller.addRemainingCountToPrompt(choices, attributeTuples.length, totalAttrsAvailable, attributeRowsStartingIndexIn)
    val leadingTextModified = getLeadingText(leadingText, attributeTuples.length, entityIn, relationSourceEntityIn, relationIn, containingGroupIn)
    val attributeDisplayStrings: Array[String] = getItemDisplayStrings(attributeTuples)

    var highlightedEntry: Option[Attribute] = if (attributeTuples.length == 0) None else Some(highlightedAttributeIn.getOrElse(attributeTuples(0)._2))
    // The variable highlightedIndexInObjList means: of the sorted attributes selected *for display* (potentially fewer than all existing attributes),
    // this is the zero-based index of the one that is marked for possible moving around in the sorted order (in the UI, marked as selected).
    def getHighlightedIndexInAttrList: Option[Int] = {
      if (highlightedEntry.isEmpty) None
      else {
        val highlightedObjFormId: Int = highlightedEntry.get.getFormId
        val highlightedObjId: Long = highlightedEntry.get.getId
        var index = -1
        for (attributeTuple <- attributeTuples) {
          index += 1
          val attribute = attributeTuple._2
          if (attribute.getFormId == highlightedObjFormId && attribute.getId == highlightedObjId) {
            return Some(index)
          }
        }
        // if we got to this point, it could simply have been deleted or something (probably), so just return something safe (instead of throwing an
        // exception, as in a previous commit):
        highlightedEntry = None
        None
      }
    }
    val highlightedIndexInObjList: Option[Int] = getHighlightedIndexInAttrList

    choices(2) =
      // MAKE SURE this condition always matches the one in the edit handler below:
      if (highlightedEntry.isDefined && controller.canEditAttributeOnSingleLine(highlightedEntry.get)) {
        "Edit the selected attribute's content (single line; go into the attribute's menu for more options)"
      } else "Edit entity name"

    if (highlightedIndexInObjList.isEmpty) {
      choices(4) = "(stub)"
    }

    val response = ui.askWhich(Some(leadingTextModified), choicesModified, attributeDisplayStrings, highlightIndexIn = getHighlightedIndexInAttrList)
    if (response.isEmpty) None
    else {
      val answer = response.get
      if (answer == 1) {
        val (newAttributeToHighlight: Option[Attribute], displayStartingRowNumber: Int) = {
          // ask for less info when here, to add entity quickly w/ no fuss, like brainstorming. Like in QuickGroupMenu.  User can always use option 2.
          val newEntity: Option[Entity] = controller.askForNameAndWriteEntity(Controller.ENTITY_TYPE, inLeadingText = Some("NAME THE ENTITY:"))
          if (newEntity.isDefined) {
            val newAttribute: Attribute = entityIn.addHASRelationToEntity(newEntity.get.getId, None, System.currentTimeMillis())
            val displayStartingRowNumber: Int = placeEntryInPosition(entityIn.getId, entityIn.getAttrCount, 0, forwardNotBackIn = true,
                                                                     attributeRowsStartingIndexIn, newAttribute.getId,
                                                                     highlightedIndexInObjList.getOrElse(0),
                                                                     if (highlightedEntry.isDefined) Some(highlightedEntry.get.getId) else None,
                                                                     numDisplayableAttributes, newAttribute.getFormId,
                                                                     if (highlightedEntry.isDefined) Some(highlightedEntry.get.getFormId) else None)
            (Some(newAttribute), displayStartingRowNumber)
          }
          else (highlightedEntry, attributeRowsStartingIndexIn)
        }
        entityMenu(entityIn, displayStartingRowNumber, newAttributeToHighlight, relationSourceEntityIn, relationIn, containingGroupIn)
      } else if (answer == 2 && highlightedEntry.isDefined && highlightedIndexInObjList.isDefined && numAttrsInEntity > 1) {
        val newStartingDisplayIndex = moveSelectedEntry(entityIn, attributeRowsStartingIndexIn, totalAttrsAvailable, highlightedIndexInObjList.get,
                                                        highlightedEntry.get, numDisplayableAttributes, relationSourceEntityIn, relationIn, containingGroupIn)
        entityMenu(entityIn, newStartingDisplayIndex, highlightedEntry, relationSourceEntityIn, relationIn, containingGroupIn)
      } else if (answer == 3) {
        // MAKE SURE this next condition always matches the one in "choices(2) = ..." above
        if (highlightedEntry.isDefined && controller.canEditAttributeOnSingleLine(highlightedEntry.get)) {
          controller.editAttributeOnSingleLine(highlightedEntry.get)
          entityMenu(entityIn, attributeRowsStartingIndexIn, highlightedEntry, relationSourceEntityIn, relationIn, containingGroupIn)
        } else {
          val editedEntity: Option[Entity] = controller.editEntityName(entityIn)
          entityMenu(if (editedEntity.isDefined) editedEntity.get else entityIn,
                     attributeRowsStartingIndexIn, highlightedEntry, relationSourceEntityIn, relationIn, containingGroupIn)
        }
      } else if (answer == 4) {
          val newAttribute: Option[Attribute] = addAttribute(entityIn, attributeRowsStartingIndexIn, highlightedEntry, relationSourceEntityIn,
                                                             relationIn, containingGroupIn)
          if (newAttribute.isDefined && highlightedEntry.isDefined) {
            placeEntryInPosition(entityIn.getId, entityIn.getAttrCount, 0, forwardNotBackIn = true, attributeRowsStartingIndexIn, newAttribute.get.getId,
                                 highlightedIndexInObjList.getOrElse(0),
                                 if (highlightedEntry.isDefined) Some(highlightedEntry.get.getId) else None,
                                 numDisplayableAttributes, newAttribute.get.getFormId,
                                 if (highlightedEntry.isDefined) Some(highlightedEntry.get.getFormId) else None)
            entityMenu(entityIn, attributeRowsStartingIndexIn, newAttribute, relationSourceEntityIn, relationIn, containingGroupIn)
          } else {
            entityMenu(entityIn, attributeRowsStartingIndexIn, highlightedEntry, relationSourceEntityIn, relationIn, containingGroupIn)
          }
      } else if (answer == 5) {
        // MAKE SURE this next condition always matches the one in "choices(4) = ..." above
        if (highlightedIndexInObjList.isDefined) {
          goToSelectedAttribute(answer, highlightedIndexInObjList.get, attributeTuples, entityIn)
        } else {
          ui.displayText("nothing selected")
        }
        entityMenu(entityIn, attributeRowsStartingIndexIn, highlightedEntry, relationSourceEntityIn, relationIn, containingGroupIn)
      } else if (answer == 6 && numAttrsInEntity > 0) {
        val startingIndex: Int = getNextStartingRowsIndex(attributeTuples.length, attributeRowsStartingIndexIn, numAttrsInEntity)
        entityMenu(entityIn, startingIndex, highlightedEntry, relationSourceEntityIn, relationIn, containingGroupIn)
      } else if (answer == 8 && answer <= choices.length && numAttrsInEntity > 0) {
        // lets user select an attribute for further operations like moving, deleting.
        // (we have to have at least one choice or ui.askWhich fails...a require() call there.)
        // NOTE: this code is similar (not identical) in EntityMenu as in QuickGroupMenu: if one changes, the other might also need maintenance.
        val choices = Array[String]("keep existing (same as ESC)")
        // says 'same screenful' because (see similar cmt elsewhere).
        val leadingText: Array[String] = Array("CHOOSE an attribute to highlight (*)")
        controller.addRemainingCountToPrompt(choices, attributeTuples.length, db.getAttrCount(entityIn.getId), attributeRowsStartingIndexIn)
        val response = ui.askWhich(Some(leadingText), choices, attributeDisplayStrings, highlightIndexIn = highlightedIndexInObjList)
        val entryToHighlight: Option[Attribute] = {
          if (response.isEmpty || response.get == 1) highlightedEntry
          else {
            // those in the condition are 1-based, not 0-based.
            // user typed a letter to select an attribute (now 0-based):
            val choicesIndex = response.get - choices.length - 1
            Some(attributeTuples(choicesIndex)._2)
          }
        }
        entityMenu(entityIn, attributeRowsStartingIndexIn, entryToHighlight, relationSourceEntityIn, relationIn, containingGroupIn)
      } else if (answer == 9 && answer <= choices.length) {
        new OtherEntityMenu(ui, db, controller).otherEntityMenu(entityIn, attributeRowsStartingIndexIn, relationSourceEntityIn, relationIn,
                                                                containingGroupIn, classDefiningEntityId)
        // Entity could have been removed or changed, so before trying to view it, confirm it exists, & reread from db to refresh data for
        // display, like public/non-public status:
        if (db.entityOnlyKeyExists(entityIn.getId)) {
          entityMenu(new Entity(db, entityIn.getId), attributeRowsStartingIndexIn, highlightedEntry, relationSourceEntityIn, relationIn, containingGroupIn)
        } else None
      } else if (answer > choices.length && answer <= (choices.length + attributeTuples.length)) {
        // those in the condition are 1-based, not 0-based.
        // lets user go to an entity or group quickly (1 stroke)
        val choicesIndex = answer - choices.length - 1
        // checking also for " && answer <= choices.length" because otherwise choosing 'a' returns 8 but if those optional menu choices were not added in,
        // then it is found among the first "choice" answers, instead of being adjusted later ("val attributeChoicesIndex = answer - choices.length - 1")
        // to find it among the "moreChoices" as it should be: would be thrown off by the optional choice numbering.
        goToSelectedAttribute(answer, choicesIndex, attributeTuples, entityIn)
        entityMenu(entityIn, attributeRowsStartingIndexIn, Some(attributeTuples(choicesIndex)._2), relationSourceEntityIn, relationIn, containingGroupIn)
      } else {
        ui.displayText("invalid response")
        entityMenu(entityIn, attributeRowsStartingIndexIn, highlightedEntry, relationSourceEntityIn, relationIn, containingGroupIn)
      }
    }
  } catch {
    case e: Throwable =>
      // catching Throwable instead of Exception here, because sometimes depending on how I'm running X etc I might get the InternalError
      // "Can't connect to X11 window server ...", and it's better to recover from that than to abort the app (ie, when eventually calling
      // Controller.getClipboardContent)..
      controller.handleException(e)
      val ans = ui.askYesNoQuestion("Go back to what you were doing (vs. going out)?",Some("y"))
      if (ans.isDefined && ans.get) entityMenu(entityIn, attributeRowsStartingIndexIn, highlightedAttributeIn, relationSourceEntityIn, relationIn, containingGroupIn)
      else None
  }

  def moveSelectedEntry(entityIn: Entity, startingDisplayRowIndexIn: Int, totalAttrsAvailable: Int,
                        highlightedIndexInObjListIn: Int, highlightedAttributeIn: Attribute, numObjectsToDisplayIn: Int,
                        relationSourceEntityIn: Option[Entity] = None, relationIn: Option[RelationToEntity] = None,
                        containingGroupIn: Option[Group] = None): Int = {
    val choices = Array[String](// (see comments at similar location in same-named method of QuickGroupMenu.)
                                "Move up 5", "Move up 1", "Move down 1", "Move down 5",
                                "(stub)", "(stub)",
                                "Move up 25", "Move down 25")
    val response = ui.askWhich(None, choices, Array[String](), highlightIndexIn = Some(highlightedIndexInObjListIn))
    if (response.isEmpty) startingDisplayRowIndexIn
    else {
      val answer = response.get
      var numRowsToMove = 0
      var forwardNotBack = false
      if ((answer >= 1 && answer <= 4) || (answer >= 7 && answer <= 8)) {
        if (answer == 1) {
          numRowsToMove = 5
        } else if (answer == 2) {
          numRowsToMove = 1
        } else if (answer == 3) {
          numRowsToMove = 1
          forwardNotBack = true
        } else if (answer == 4) {
          numRowsToMove = 5
          forwardNotBack = true
        } else if (answer == 7) {
          numRowsToMove = 20
        } else if (answer == 8) {
          numRowsToMove = 20
          forwardNotBack = true
        }
        val displayStartingRowNumber: Int = placeEntryInPosition(entityIn.getId, totalAttrsAvailable, numRowsToMove, forwardNotBackIn = forwardNotBack,
                                                                 startingDisplayRowIndexIn, highlightedAttributeIn.getId,
                                                                 highlightedIndexInObjListIn,
                                                                 Some(highlightedAttributeIn.getId),
                                                                 numObjectsToDisplayIn, highlightedAttributeIn.getFormId,
                                                                 Some(highlightedAttributeIn.getFormId))
        displayStartingRowNumber
      }
      else {
        startingDisplayRowIndexIn
      }
    }
  }

  def getLeadingText(leadingTextIn: Array[String], numAttributes: Int,
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
    leadingTextIn(0) += ": created " + entityIn.getCreationDateFormatted
    leadingTextIn(1) = if (numAttributes == 0) "No attributes have been assigned to this object, yet."
    else "Attribute list menu: (or choose attribute by letter)"
    leadingTextIn
  }

  def getItemDisplayStrings(attributeTuples: Array[(Long, Attribute)]) = {
    val attributeNames: Array[String] =
      for (attributeTuple <- attributeTuples) yield {
        val attribute = attributeTuple._2
        attribute match {
        case relation: RelationToEntity =>
          val relationType = new RelationType(db, relation.getAttrTypeId)
          val desc = attribute.getDisplayString(controller.maxNameLength, Some(new Entity(db, relation.getRelatedId2)), Some(relationType), simplify = true)
          val prefix = controller.getEntityContentSizePrefix(relation.getRelatedId2)
          prefix + desc
        case relation: RelationToGroup =>
          val relationType = new RelationType(db, relation.getAttrTypeId)
          val desc = attribute.getDisplayString(controller.maxNameLength, None, Some(relationType), simplify = true)
          val prefix = controller.getGroupContentSizePrefix(relation.getGroupId)
          prefix + "group: " + desc
        case _ =>
          attribute.getDisplayString(controller.maxNameLength, None, None)
      }
    }
    attributeNames
  }

  def addAttribute(entityIn: Entity, startingAttributeIndexIn: Int, highlightedAttributeIn: Option[Attribute], relationSourceEntityIn: Option[Entity] = None,
                   relationIn: Option[RelationToEntity] = None, containingGroupIn: Option[Group] = None): Option[Attribute] = {
    val whichKindOfAttribute =
      ui.askWhich(Some(Array("Choose which kind of attribute to add:")),
                  Array("Relation to entity (i.e., \"is near\" a microphone, complete menu)",
                        "Relation to existing entity: quick search by name (uses \"has\" relation)",
                        "quantity attribute (example: a numeric value like \"length\"",
                        "text attribute (rare: usually prefer relations; but for example: a serial number, which is not subject to arithmetic)",
                        "date",
                        "true/false value",

                        "external file (BUT CONSIDER FIRST ADDING AN ENTITY SPECIFICALLY FOR THE DOCUMENT SO IT CAN HAVE A DATE, OTHER ATTRS ETC.; " +
                        "AND ADDING THE DOCUMENT TO THAT ENTITY, SO IT CAN ALSO BE ASSOCIATED WITH OTHER ENTITIES EASILY!; also, " +
                        "given the concept behind OM, it's probably best" +
                        " to use this only for historical artifacts, or when you really can't fully model the data right now",

                        "Relation to group (i.e., \"has\" a list/group)",
                        "external web page (or other URI, to refer to external information and optionally quote it)")
                 )
    if (whichKindOfAttribute.isDefined) {
      val whichKindAnswer = whichKindOfAttribute.get
      if (whichKindAnswer == 1) {
        def addRelationToEntity(dhIn: RelationToEntityDataHolder): Option[RelationToEntity] = {
          Some(entityIn.addRelationToEntity(dhIn.attrTypeId, dhIn.entityId2, dhIn.validOnDate, dhIn.observationDate))
        }
        controller.askForInfoAndAddAttribute[RelationToEntityDataHolder](new RelationToEntityDataHolder(0, None, 0, 0), Controller .RELATION_TYPE_TYPE,
                                                                         "CREATE OR SELECT RELATION TYPE: (" + controller.mRelTypeExamples + ")",
                                                                         controller.askForRelationEntityIdNumber2, addRelationToEntity)
      } else if (whichKindAnswer == 2) {
        val eId: Option[IdWrapper] = controller.askForNameAndSearchForEntity
        if (eId.isDefined) {
          Some(entityIn.addHASRelationToEntity(eId.get.getId, None, System.currentTimeMillis))
        } else None
      } else if (whichKindAnswer == 3) {
        def addQuantityAttribute(dhIn: QuantityAttributeDataHolder): Option[QuantityAttribute] = {
          Some(entityIn.addQuantityAttribute(dhIn.attrTypeId, dhIn.unitId, dhIn.number, dhIn.validOnDate, dhIn.observationDate))
        }
        controller.askForInfoAndAddAttribute[QuantityAttributeDataHolder](new QuantityAttributeDataHolder(0, None, 0, 0, 0), Controller.QUANTITY_TYPE,
                                                                          controller.quantityDescription,
                                                                          controller.askForQuantityAttributeNumberAndUnit, addQuantityAttribute)
      } else if (whichKindAnswer == 4) {
        def addTextAttribute(dhIn: TextAttributeDataHolder): Option[TextAttribute] = {
          Some(entityIn.addTextAttribute(dhIn.attrTypeId, dhIn.text, dhIn.validOnDate, dhIn.observationDate))
        }
        controller.askForInfoAndAddAttribute[TextAttributeDataHolder](new TextAttributeDataHolder(0, Some(0), 0, ""), Controller.TEXT_TYPE,
                                                                      "SELECT TYPE OF " + controller.textDescription + ": ", controller
                                                                                                                             .askForTextAttributeText,
                                                                      addTextAttribute)
      } else if (whichKindAnswer == 5) {
        def addDateAttribute(dhIn: DateAttributeDataHolder): Option[DateAttribute] = {
          Some(entityIn.addDateAttribute(dhIn.attrTypeId, dhIn.date))
        }
        controller.askForInfoAndAddAttribute[DateAttributeDataHolder](new DateAttributeDataHolder(0, 0), Controller.DATE_TYPE,
                                                                      "SELECT TYPE OF DATE: ", controller.askForDateAttributeValue, addDateAttribute)
      } else if (whichKindAnswer == 6) {
        def addBooleanAttribute(dhIn: BooleanAttributeDataHolder): Option[BooleanAttribute] = {
          Some(entityIn.addBooleanAttribute(dhIn.attrTypeId, dhIn.boolean))
        }
        controller.askForInfoAndAddAttribute[BooleanAttributeDataHolder](new BooleanAttributeDataHolder(0, Some(0), 0, false), Controller.BOOLEAN_TYPE,
                                                                         "SELECT TYPE OF TRUE/FALSE VALUE: ", controller.askForBooleanAttributeValue,
                                                                         addBooleanAttribute)
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
            if (!new File(result.get.getOriginalFilePath).delete()) {
              ui.displayText("Unable to delete file at that location; reason unknown.  You could check the permissions.")
            }
          }
        }
        result
      } else if (whichKindAnswer == 8) {
        def addRelationToGroup(dhIn: RelationToGroupDataHolder): Option[RelationToGroup] = {
          val rtgId: Long = entityIn.addRelationToGroup(dhIn.attrTypeId, dhIn.groupId, dhIn.validOnDate, dhIn.observationDate)
          Some(new RelationToGroup(db, rtgId, dhIn.entityId, dhIn.attrTypeId, dhIn.groupId))
        }
        val result: Option[Attribute] = controller.askForInfoAndAddAttribute[RelationToGroupDataHolder](new RelationToGroupDataHolder(entityIn.getId, 0, 0,
                                                                                                                                      None,
                                                                                                                                      System
                                                                                                                                      .currentTimeMillis()),
                                                                                                        Controller.RELATION_TYPE_TYPE,
                                                                                                        "CREATE OR SELECT RELATION TYPE: (" + controller
                                                                                                                                              .mRelTypeExamples + ")" +
                                                                                                        "." + TextUI.NEWLN + "(Does anyone see a specific " +
                                                                                                        "reason to keep asking for these dates?)",
                                                                                                        controller.askForRelToGroupInfo, addRelationToGroup)
        if (result.isEmpty) {
          entityMenu(entityIn, startingAttributeIndexIn, highlightedAttributeIn, relationSourceEntityIn, relationIn, containingGroupIn)
          None
        } else {
          val newRtg = result.get.asInstanceOf[RelationToGroup]
          new GroupMenu(ui, db, controller).groupMenu(new Group(db, newRtg.getGroupId), 0, Some(newRtg), None, Some(entityIn))
          result
        }
      } else if (whichKindAnswer == 9) {
        val newEntityName: Option[String] = ui.askForString(Some(Array{"Enter a name (or description) for this web page or other URI"}))
        if (newEntityName.isEmpty || newEntityName.get.isEmpty) return None

        val ans1 = ui.askWhich(Some(Array[String]("Do you want to enter the URI via the keyboard (normal) or the" +
                                                  " clipboard (faster sometimes)?")), Array("keyboard", "clipboard"))
        if (ans1.isEmpty) return None
        val keyboardOrClipboard1 = ans1.get
        val uri: String = if (keyboardOrClipboard1 == 1) {
          val text = ui.askForString(Some(Array("Enter the URI:")))
          if (text.isEmpty || text.get.isEmpty) return None else text.get
        } else {
          val uriReady = ui.askYesNoQuestion("Put the url on the system clipboard, then Enter to continue (or hit ESC or answer 'n' to get out)", Some("y"))
          if (uriReady.isEmpty || !uriReady.get) return None
          Controller.getClipboardContent
        }

        val ans2 = ui.askWhich(Some(Array[String]("Do you want to enter a quote from it, via the keyboard (normal) or the" +
                                                  " clipboard (faster sometimes, especially if it's multiline)? Or, ESC to not enter a quote.")),
                               Array("keyboard", "clipboard"))
        val quote:Option[String] = if (ans2.isEmpty) {
          None
        } else {
          val keyboardOrClipboard2 = ans2.get
          if (keyboardOrClipboard2 == 1) {
            val text = ui.askForString(Some(Array("Enter the quote")))
            if (text.isEmpty || text.get.isEmpty) return None else text
          } else {
            val clip = ui.askYesNoQuestion("Put a quote on the system clipboard, then Enter to continue (or answer 'n' to get out)", Some("y"))
            if (clip.isEmpty || !clip.get) return None
            Some(Controller.getClipboardContent)
          }
        }
        val quoteInfo = if (quote.isEmpty) "" else "For this text: \n  " + quote.get + "\n...and, "

        val proceedAnswer = ui.askYesNoQuestion(quoteInfo + "...for this name & URI:\n  " + newEntityName.get + "\n  " + uri + "" +
                                                "\n...: do you want to save them?", Some("y"))
        if (proceedAnswer.isEmpty || !proceedAnswer.get) return None

        val (newEntity: Entity, newRTE: RelationToEntity) = db.addUriEntityWithUriAttribute(entityIn, newEntityName.get, uri, System.currentTimeMillis(),
                                                                entityIn.getPublic, callerManagesTransactionsIn = false, quote)
        entityMenu(newEntity, relationSourceEntityIn = Some(entityIn))
        Some(newRTE)
      } else {
        ui.displayText("invalid response")
        None
      }
    } else None
  }

  def getNextStartingRowsIndex(numAttrsToDisplay: Int, startingAttributeRowsIndexIn: Int, numAttrsInEntity: Long): Int = {
    val startingIndex = {
      val currentPosition = startingAttributeRowsIndexIn + numAttrsToDisplay
      if (currentPosition >= numAttrsInEntity) {
        ui.displayText("End of attribute list found; restarting from the beginning.")
        0 // start over
      } else currentPosition

    }
    startingIndex
  }

  def goToSelectedAttribute(answer: Int, choiceIndex: Int, attributeTuples: Array[(Long, Attribute)], entityIn: Entity) {
    // user typed a letter to select an attribute (now 0-based)
    if (choiceIndex >= attributeTuples.length) {
      ui.displayText("The program shouldn't have let us get to this point, but the selection " + answer + " is not in the list.")
    } else {
      val o: Attribute = attributeTuples(choiceIndex)._2
      o match {
        //idea: there's probably also some more scala-like cleaner syntax 4 this, as elsewhere:
        case qa: QuantityAttribute => controller.attributeEditMenu(qa)
        case da: DateAttribute => controller.attributeEditMenu(da)
        case ba: BooleanAttribute => controller.attributeEditMenu(ba)
        case fa: FileAttribute => controller.attributeEditMenu(fa)
        case ta: TextAttribute => controller.attributeEditMenu(ta)
        case relToEntity: RelationToEntity => entityMenu(new Entity(db, relToEntity.getRelatedId2), 0, None, Some(entityIn), Some(relToEntity))
        case relToGroup: RelationToGroup => new QuickGroupMenu(ui, db, controller).quickGroupMenu(new Group(db, relToGroup.getGroupId), 0, Some(relToGroup),
                                                                                                  containingEntityIn = Some(entityIn))
        case _ => throw new Exception("Unexpected choice has class " + o.getClass.getName + "--what should we do here?")
      }
    }
  }

  protected def getAdjacentEntriesSortingIndexes(entityIdIn: Long, movingFromPosition_sortingIndexIn: Long, queryLimitIn: Option[Long],
                                   forwardNotBackIn: Boolean): List[Array[Option[Any]]] = {
    db.getAdjacentAttributesSortingIndexes(entityIdIn, movingFromPosition_sortingIndexIn, queryLimitIn, forwardNotBackIn)
  }

  protected def getNearestEntrysSortingIndex(entityIdIn: Long, startingPointSortingIndexIn: Long, forwardNotBackIn: Boolean): Option[Long] = {
    db.getNearestAttributeEntrysSortingIndex(entityIdIn, startingPointSortingIndexIn, forwardNotBackIn = forwardNotBackIn)
  }

  protected def renumberSortingIndexes(entityIdIn: Long): Unit = {
    db.renumberSortingIndexes(entityIdIn)
  }

  protected def updateSortedEntry(entityIdIn: Long, movingAttributeFormIdIn: Int, movingAttributeIdIn: Long, sortingIndexIn: Long): Unit = {
    db.updateAttributeSorting(entityIdIn, movingAttributeFormIdIn, movingAttributeIdIn, sortingIndexIn)
  }

  protected def getSortingIndex(entityIdIn: Long, attributeFormIdIn: Int, attributeIdIn: Long): Long = {
    db.getEntityAttributeSortingIndex(entityIdIn, attributeFormIdIn, attributeIdIn)
  }

  protected def indexIsInUse(entityIdIn: Long, sortingIndexIn: Long): Boolean = {
    db.attributeSortingIndexInUse(entityIdIn, sortingIndexIn)
  }

  protected def findUnusedSortingIndex(entityIdIn: Long, startingWithIn: Long): Long = {
    db.findUnusedAttributeSortingIndex(entityIdIn, Some(startingWithIn))
  }

}
