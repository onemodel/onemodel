/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2014-2015 inclusive, Luke A Call; all rights reserved.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule, guidelines around binary
    distribution, and the GNU Affero General Public License as published by the Free Software Foundation, either version 3
    of the License, or (at your option) any later version.  See the file LICENSE for details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
package org.onemodel

import java.io._

import org.onemodel.controller.{Controller, EntityMenu, QuickGroupMenu}
import org.onemodel.database.PostgreSQLDatabase
import org.onemodel.model._

import scala.annotation.tailrec
import scala.collection.mutable

class ImportExport(override val ui: TextUI, dbIn_OVERRIDES_mDB_variable_WhichHasANewDbConnectionTHATWEDONTWANT: PostgreSQLDatabase) extends Controller(ui) {
  override val mDB = dbIn_OVERRIDES_mDB_variable_WhichHasANewDbConnectionTHATWEDONTWANT

  // idea: see commend in EntityMenu about scoping.
  def exportToCollapsibleOutline(entity: Entity) {
    val ans: Option[String] = ui.askForString(Some(Array("Enter number of levels (0 = 'all'); ESC to cancel")), Some(isNumeric), Some("0"))
    if (ans != None) {
      val levelsToDescend: Int = ans.get.toInt

      val ans2: Option[Boolean] = ui.askYesNoQuestion("Include metadata (verbose detail: id's, types...)?")
      if (ans2 != None) {
        val includeMetadata: Boolean = ans2.get

        //idea: make these choice strings into an enum? and/or the answers into an enum? what's the scala idiom? see same issue elsewhere
        val includePublicData: Option[Boolean] = ui.askYesNoQuestion("Include public data?", Some("y"))
        val includeNonPublicData: Option[Boolean] = ui.askYesNoQuestion("Include data marked non-public?")
        val includeUnspecifiedData: Option[Boolean] = ui.askYesNoQuestion("Include data not specified as public or non-public?")

        if (includePublicData != None && includeNonPublicData != None && includeUnspecifiedData != None) {
          require(levelsToDescend >= 0)
          val spacesPerIndentLevel = 2
          val exportedEntities = new scala.collection.mutable.TreeSet[Long]()

          def printSpaces(num: Int, out: PrintWriter) {
            for (i <- 1 to num) {
              out.print(" ")
            }
          }

          //@tailrec  THIS IS NOT TO BE TAIL RECURSIVE UNTIL IT'S KNOWN HOW TO MAKE SOME CALLS BE SO, AND SOME *NOT* TAIL RECURSIVE (because some of them
          //*do* need to return & finish their work, such as when iterating through the entities & subgroups)! (but test it: is it really a problem?)
          def export(entityIn: Entity, levelsRemainingToDescendIn: Int, currentIndentationLevelsIn: Int, out: PrintWriter, includeMetadataIn: Boolean) {
            // useful while debugging:
            //out.flush()

            if (exportedEntities.contains(entityIn.getId)) {
              printSpaces(currentIndentationLevelsIn * spacesPerIndentLevel, out)
              if (includeMetadataIn) out.print("(duplicate: EN --> " + entityIn.getId + ": ")
              out.print(entityIn.getName)
              if (includeMetadataIn) out.print(")")
              out.println()
            } else {
              val entityPublicStatus: Option[Boolean] = entityIn.getPublic
              val allowedToExport = (entityPublicStatus != None && entityPublicStatus.get && includePublicData.get) ||
                                    (entityPublicStatus != None && !entityPublicStatus.get && includeNonPublicData.get) ||
                                    (entityPublicStatus == None && includeUnspecifiedData.get)
              if (allowedToExport) {
                exportedEntities.add(entityIn.getId)

                printSpaces(currentIndentationLevelsIn * spacesPerIndentLevel, out)
                val entityName: String = entityIn.getName
                if (includeMetadataIn) out.println("EN " + entityIn.getId + ": " + entityIn.getDisplayString)
                else out.println(entityName)

                val attributeObjList: java.util.ArrayList[Attribute] = mDB.getSortedAttributes(entityIn.getId, 0, 0)._1
                for (attribute: Attribute <- attributeObjList.toArray(Array[Attribute]())) yield attribute match {
                  case relation: RelationToEntity =>
                    val relationType = new RelationType(mDB, relation.getAttrTypeId)
                    val entity2 = new Entity(mDB, relation.getRelatedId2)
                    if (includeMetadataIn) {
                      printSpaces((currentIndentationLevelsIn + 1) * spacesPerIndentLevel, out)
                      out.println(attribute.getDisplayString(0, Some(entity2), Some(relationType)))
                    }
                    if (levelsRemainingToDescendIn == 0 || levelsRemainingToDescendIn > 1) {
                      export(entity2, if (levelsRemainingToDescendIn == 0) 0 else levelsRemainingToDescendIn - 1, currentIndentationLevelsIn + 1, out,
                             includeMetadataIn)
                    }
                  case relation: RelationToGroup =>
                    val relationType = new RelationType(mDB, relation.getAttrTypeId)
                    val group = new Group(mDB, relation.getGroupId)
                    val grpName = group.getName

                    // if a group name is different from its entity name, indicate the differing group name also, otherwise complete the line just above w/ NL
                    if (entityName != grpName) {
                      printSpaces((currentIndentationLevelsIn + 1) * spacesPerIndentLevel, out)
                      out.println("(group named: " + grpName + ")")
                    }

                    if (includeMetadataIn) {
                      printSpaces(currentIndentationLevelsIn * spacesPerIndentLevel, out)
                      // plus one more level of spaces to make it look better but still ~equivalently/exchangeably importable:
                      printSpaces(spacesPerIndentLevel, out)
                      out.println("(group details: " + attribute.getDisplayString(0, None, Some(relationType)) + ")")
                    }
                    if (levelsRemainingToDescendIn == 0 || levelsRemainingToDescendIn > 1) {
                      for (entityInGroup: Entity <- group.getGroupEntries(0).toArray(Array[Entity]())) {
                        export(entityInGroup, if (levelsRemainingToDescendIn == 0) 0 else levelsRemainingToDescendIn - 1, currentIndentationLevelsIn + 1, out,
                               includeMetadataIn)
                      }
                    }
                  case _ =>
                    printSpaces((currentIndentationLevelsIn + 1) * spacesPerIndentLevel, out)
                    if (includeMetadataIn) {
                      out.println((attribute match {
                        case ba: BooleanAttribute => "BA "
                        case da: DateAttribute => "DA "
                        case fa: FileAttribute => "FA "
                        case qa: QuantityAttribute => "QA "
                        case ta: TextAttribute => "TA "
                      }) + /*attribute.getId +*/ ": " + attribute.getDisplayString(0, None, None))
                    } else out.println(attribute.getDisplayString(0, None, None))
                }
              }
            }
          }

          //idea (also in task list): change this to be a reliable filename (incl no backslashes? limit it to a whitelist of chars? a simple fn for that?
          var fixedEntityName = entity.getName.replace(" ", "")
          fixedEntityName = fixedEntityName.replace("/", "-")
          //fixedEntityName = fixedEntityName.replace("\\","-")

          val prefix: String = "onemodel-export_" + entity.getId + "_" + fixedEntityName + "-"
          val outputFile = File.createTempFile(prefix, ".txt")
          val output: PrintWriter = new PrintWriter(new BufferedWriter(new FileWriter(outputFile)))
          try {
            export(entity, levelsToDescend, 0, output, includeMetadata)
            // flushes before we report 'done' to the user:
            output.close()
            ui.displayText("Exported to file: " + outputFile.getCanonicalPath)
          } finally {
            if (output != null) {
              try output.close()
              catch {
                case e: Exception =>
                // ignore
              }
            }
          }
        }
      }
    }
  }

  /**
   * 1st parameter must be either an Entity or a RelationToGroup (what is the right way to do that, in the signature?).
   */
  def importCollapsibleOutlineAsGroups(firstContainingEntryIn: AnyRef) {
    require(firstContainingEntryIn.isInstanceOf[Entity] || firstContainingEntryIn.isInstanceOf[RelationToGroup])
    val ans1: Option[String] = ui.askForString(Some(Array("Enter file path (must exist, be readable, AND a text file with lines spaced in the form of a" +
                                                          " collapsible outline where each level change is marked by 1 tab or 2 spaces; textAttribute content" +
                                                          " can be indicated by surrounding a body of text thus: <ta>text</ta> )," +
                                                          " then press Enter; ESC to cancel")),
                                               Some(inputFileValid))
    if (ans1 != None) {
      val path = ans1.get
      val makeThemPublic: Option[Boolean] = ui.askYesNoQuestion("Do you want the entities imported to be marked as public?  Set it to the value the majority of " +
                                                      "imported data should have; you can then edit the individual exceptions afterward as needed.  " +
                                                      "Enter y for public, n for nonpublic, or a space for 'unknown/unspecified', aka decide later.",
                                                      Some(""), allowBlankAnswer = true)
      val ans3 = ui.askYesNoQuestion("Keep the filename as the top level of the imported list? (Answering no will put the top level entries from inside" +
                                     " the " +
                                     "file, as entries directly under this entity or group.)")
      if (ans3 != None) {
        val creatingNewStartingGroupFromTheFilename: Boolean = ans3.get
        val addingToExistingGroup: Boolean = firstContainingEntryIn.isInstanceOf[RelationToGroup] && !creatingNewStartingGroupFromTheFilename

        val putEntriesAtEndOption: Option[Boolean] = {
          if (addingToExistingGroup) {
            ui.askYesNoQuestion("Put the new entries at the end of the list? (No means put them at the beginning, the default.)")
          } else
            Some(false)
        }

        if (putEntriesAtEndOption != None) {
          //@tailrec: would be nice to use, but jvm doesn't support it, or something.
          def tryIt() {
            var reader: Reader = null
            try {
              val putEntriesAtEnd: Boolean = putEntriesAtEndOption.get
              val fileToImport = new File(path)
              reader = new FileReader(fileToImport)
              mDB.beginTrans()

              doTheImport(reader, fileToImport.getCanonicalPath, fileToImport.lastModified(), firstContainingEntryIn, creatingNewStartingGroupFromTheFilename,
                          addingToExistingGroup, putEntriesAtEnd, makeThemPublic)

              val keepAnswer: Option[Boolean] = {
                //idea: look into how long that time is (see below same cmt):
                val msg: String = "Group imported, but browse around to see if you want to keep it, " +
                                  "then ESC back here to commit the changes....  (If you wait beyond some amount of time(?), it seems that postgres will commit " +
                                  "the change whether you want it or not, even if the message at that time says 'rolled back...')"
                ui.displayText(msg)
                firstContainingEntryIn match {
                  case entity: Entity => new EntityMenu(ui, mDB).entityMenu(0, entity)
                  case rtg: RelationToGroup => new QuickGroupMenu(ui, mDB).quickGroupMenu(0, firstContainingEntryIn
                                                                                             .asInstanceOf[RelationToGroup])
                  case _ => throw new OmException("??")
                }
                ui.askYesNoQuestion("Do you want to commit the changes as they were made?")
              }
              if (keepAnswer == None || !keepAnswer.get) {
                mDB.rollbackTrans()
                //idea: look into how long that time is (see above same cmt)
                ui.displayText("Rolled back the import: no changes made (unless you waited too long and postgres committed it anyway...?).")
              } else {
                mDB.commitTrans()
              }
            } catch {
              case e: Exception =>
                mDB.rollbackTrans()
                if (reader != null) {
                  try reader.close()
                  catch {
                    case e: Exception =>
                    // ignore
                  }
                }
                val msg: String = {
                  val stringWriter = new StringWriter()
                  e.printStackTrace(new PrintWriter(stringWriter))
                  stringWriter.toString
                }
                ui.displayText(msg + TextUI.NEWLN + "Error while importing; no changes made. ")
                val ans = ui.askYesNoQuestion("For some errors, you can go fix the file then come back here.  Retry now?", Some("y"))
                if (ans != None && ans.get) {
                  tryIt()
                }
            } finally {
              if (reader != null) {
                try reader.close()
                catch {
                  case e: Exception =>
                  // ignore
                }
              }
            }
          }

          tryIt()
        }
      }
    }
  }

  @tailrec
  private def getFirstNonSpaceIndex(line: Array[Byte], index: Int): Int = {
    //idea: this logic might need to be fixed (9?):
    if (line(index) == 9) {
      // could count tab as 1, but not testing with that for now:
      throw new OmException("tab not supported")
    }
    else if (index >= line.size || (line(index) != ' ')) {
      index
    } else {
      getFirstNonSpaceIndex(line, index + 1)
    }
  }

  def createAndAddEntityToGroup(line: String, group: Group, newSortingIndex: Long, isPublicIn: Option[Boolean]): Entity = {
    val entityId: Long = mDB.createEntity(line.trim, group.getClassId, isPublicIn)
    mDB.addEntityToGroup(group.getId, entityId, Some(newSortingIndex), callerManagesTransactionsIn = true)
    new Entity(mDB, entityId)
  }

  /* The parameter lastEntityIdAdded means the one to which a new subgroup will be added, such as in a series of entities
     added to a list and the code needs to know about the most recent one, so if the line is further indented, it knows where to
     create the subgroup.

     We always start from the current container (entity or group) and add the new material to a entry (Entity (+ 1 subgroup if needed)) created there.

     The parameter lastIndentationlevel should be set to zero, from the original caller, and indent from there w/ the recursion.
  */
  @tailrec
  private def handleRestOfLines(r: LineNumberReader, lastEntityAdded: Option[Entity], lastIndentationLevel: Int, containerList: List[AnyRef],
                                lastSortingIndexes: List[Long], observationDateIn: Long, mixedClassesAllowedDefaultIn: Boolean,
                                makeThemPublicIn: Option[Boolean]) {
    // (see cmts just above about where we start)
    require(containerList.size == lastIndentationLevel + 1)
    // always should at least have an entry for the entity or group from where the user initiated this import, the base of all the adding.
    require(containerList.size > 0)
    // how do this type mgt better, like, in the signature? (also needed elsewhere):
    require(containerList.head.isInstanceOf[Entity] || containerList.head.isInstanceOf[Group])

    val spacesPerIndentLevel = 2
    val lineUntrimmed: String = r.readLine()
    if (lineUntrimmed != null) {
      val lineNumber = r.getLineNumber

      // these indicate beg/end of TextAttribute content; CODE ASSUMES THEY ARE LOWER-CASE!, so making that explicit, to be sure in case we change them later.
      val beginTaMarker = "<ta>".toLowerCase
      val endTaMarker = "</ta>".toLowerCase

      if (lineUntrimmed.toLowerCase.contains(beginTaMarker)) {
        // we have a section of text marked for importing into a single TextAttribute:
        val lineContentBeforeMarker = lineUntrimmed.substring(0, lineUntrimmed.toLowerCase.indexOf(beginTaMarker)).trim
        val restOfLine = lineUntrimmed.substring(lineUntrimmed.toLowerCase.indexOf(beginTaMarker) + beginTaMarker.length).trim
        if (restOfLine.toLowerCase.contains(endTaMarker)) throw new OmException("\"Unsupported format at line " + lineNumber + ": beginning and ending " +
                                                                                "markers must not be on the same line.")
        processTextAttributeContent(lineContentBeforeMarker, restOfLine, r, lastEntityAdded.get.getId, beginTaMarker, endTaMarker)
        handleRestOfLines(r, lastEntityAdded, lastIndentationLevel, containerList, lastSortingIndexes, observationDateIn, mixedClassesAllowedDefaultIn,
                          makeThemPublicIn)
      } else {
        val line: String = lineUntrimmed.trim

        if (line == "." || line.isEmpty) {
          // nothing to do: that kind of line was just to create whitespace in my outline. So simply go to the next line:
          handleRestOfLines(r, lastEntityAdded, lastIndentationLevel, containerList, lastSortingIndexes, observationDateIn, mixedClassesAllowedDefaultIn,
                            makeThemPublicIn)
        } else {
          if (line.size > maxNameLength) throw new OmException("Line " + lineNumber + " is over " + maxNameLength + " characters " +
                                                               " (has " + line.size + "): " + line)
          val indentationSpaceCount: Int = getFirstNonSpaceIndex(lineUntrimmed.getBytes, 0)
          if (indentationSpaceCount % spacesPerIndentLevel != 0) throw new OmException("# of spaces is off, on line " + lineNumber + ": '" + line + "'")
          val newIndentationLevel = indentationSpaceCount / spacesPerIndentLevel
          if (newIndentationLevel == lastIndentationLevel) {
            require(lastIndentationLevel >= 0)
            // same level, so add line to same entity group
            val newSortingIndex = lastSortingIndexes.head + 1
            val newEntity: Entity = {
              containerList.head match {
                case entity: Entity =>
                  entity.createEntityAndAddHASRelationToIt(line, observationDateIn, makeThemPublicIn, callerManagesTransactionsIn = true)._1
                case group: Group =>
                  createAndAddEntityToGroup(line, containerList.head.asInstanceOf[Group], newSortingIndex, makeThemPublicIn)
                case _ => throw new OmException("??")
              }
            }

            handleRestOfLines(r, Some(newEntity), lastIndentationLevel, containerList, newSortingIndex :: lastSortingIndexes.tail, observationDateIn,
                              mixedClassesAllowedDefaultIn, makeThemPublicIn)
          } else if (newIndentationLevel < lastIndentationLevel) {
            require(lastIndentationLevel >= 0)
            // outdented, so need to go back up to a containing group (list), to add line
            val numLevelsBack = lastIndentationLevel - newIndentationLevel
            require(numLevelsBack > 0 && lastIndentationLevel - numLevelsBack >= 0)
            val newContainerList = containerList.drop(numLevelsBack)
            val newSortingIndexList = lastSortingIndexes.drop(numLevelsBack)
            val newSortingIndex = newSortingIndexList.head + 1
            val newEntity: Entity = {
              newContainerList.head match {
                case entity: Entity =>
                  entity.createEntityAndAddHASRelationToIt(line, observationDateIn, makeThemPublicIn, callerManagesTransactionsIn = true)._1
                case group: Group =>
                  createAndAddEntityToGroup(line, group, newSortingIndex, makeThemPublicIn)
                case _ => throw new OmException("??")
              }
            }
            handleRestOfLines(r, Some(newEntity), newIndentationLevel, newContainerList, newSortingIndex :: newSortingIndexList.tail, observationDateIn,
                              mixedClassesAllowedDefaultIn, makeThemPublicIn)
          } else if (newIndentationLevel > lastIndentationLevel) {
            // indented, so create a subgroup & add line there:
            require(newIndentationLevel >= 0)
            // (not None because it will be used now to create a subgroup; when we get here there should always be a value) :
            if (lastEntityAdded == None) {
              throw new OmException("There's an error.  Are you importing a file to a group, but the first line is indented?  If so try fixing that (un-indent, & fix the rest to match).  Otherwise, there's a bug in the program.");
            }
            val addedLevelsIn = newIndentationLevel - lastIndentationLevel
            if (addedLevelsIn != 1) throw new OmException("Unsupported format: line " + lineNumber + " is indented too far in, " +
                                                          "relative to the line before it: " + line)
            val mixedClassesAllowed: Boolean = {
              containerList.head match {
                case group: Group =>
                  //untested, could be useful in dif't need:
                  group.getMixedClassesAllowed
                //throw new OmException("how did we get here, if indenting should always be from a prior-created entity")
                case entity: Entity =>
                  mixedClassesAllowedDefaultIn
                case _ => throw new OmException("??")
              }
            }
            // E.g., if "3" is the last entity created in the series of lines '1', '2', and '3' (which has indented under it '4'), and so '4' is the
            // current line, create a subgroup on '3' called '3' (the subgroup that entity sort of represents), and it becomes the new container. If the
            // user preferred this to be a relation to entity instead of to group to contain the sub-things,
            // oh well they can add it to the entity as such,
            // for now at least.
            val newGroup: Group = lastEntityAdded.get.createGroupAndAddHASRelationToIt(lastEntityAdded.get.getName, mixedClassesAllowed,
                                                                                       observationDateIn, callerManagesTransactionsIn = true)._1
            // since a new grp, start at beginning of sorting indexes
            val newSortingIndex = mDB.minIdValue
            val newSubEntity: Entity = createAndAddEntityToGroup(line, newGroup, newSortingIndex, makeThemPublicIn)
            handleRestOfLines(r, Some(newSubEntity), newIndentationLevel, newGroup :: containerList, newSortingIndex :: lastSortingIndexes,
                              observationDateIn, mixedClassesAllowedDefaultIn, makeThemPublicIn)
          } else throw new OmException("Shouldn't get here!?: " + lastIndentationLevel + ", " + newIndentationLevel)
        }
      }
    }
  }

  def processTextAttributeContent(attrTypeName: String, firstLineContent: String, r: LineNumberReader, entityId: Long, beginningTagMarker: String,
                                  endTaMarker: String) {
    val attrTypeId: Long = {
      val idsByName: Option[List[Long]] = mDB.findAllEntityIdsByName(attrTypeName.trim, caseSensitive = true)
      if (idsByName != None && idsByName.get.size == 1)
        idsByName.get.head
      else {
        // idea: alternatively, could use a generic one in this case?  Optionally?
        val prompt = "A name for the *type* of this text attribute was not provided; it would be the entire line content preceding the \"" +
                     beginningTagMarker + "\" " +
                     "(it has to match an existing entity, case-sensitively)"
        val typeId = askForAttributeTypeId(prompt + ", so please choose one or ESC to abort this import operation:", Controller.TEXT_TYPE, None, None)
        if (typeId == None)
          throw new OmException(prompt + " or selected.")
        else
          typeId.get
      }
    }
    val text: String = firstLineContent.trim + TextUI.NEWLN + {
      def getRestOfLines(rIn: LineNumberReader, sbIn: mutable.StringBuilder): mutable.StringBuilder = {
        // Don't trim, because we want to preserve formatting/whitespace here, including blank lines (always? -- yes, editably.).
        val line = rIn.readLine()
        if (line == null) {
          sbIn
        } else {
          if (line.toLowerCase.contains(endTaMarker.toLowerCase)) {
            val markerStartLocation = line.toLowerCase.indexOf(endTaMarker.toLowerCase)
            val markerEndLocation = markerStartLocation + endTaMarker.size
            val lineNumber = r.getLineNumber
            def rtrim(s: String): String = s.replaceAll("\\s+$", "")
            val rtrimmedLine = rtrim(line)
            if (rtrimmedLine.substring(markerEndLocation).size > 0) throw new OmException("\"Unsupported format at line " + lineNumber + ": A \"" + endTaMarker +
                                                                                  "\" (end text attribute) marker must be the last text on a line.")
            sbIn.append(line.substring(0, markerStartLocation))
          } else {
            sbIn.append(line + TextUI.NEWLN)
            getRestOfLines(rIn, sbIn)
          }
        }
      }
      val builder = getRestOfLines(r, new mutable.StringBuilder)
      builder.toString()
    }
    mDB.createTextAttribute(entityId, attrTypeId, text)
  }

  //@tailrec why not? needs that jvm fix first to work for the scala compiler?  see similar comments elsewhere on that? (does java8 provide it now?
  // wait for next debian stable version--jessie?--be4 it's probably worth finding out)
  def doTheImport(dataSourceIn: Reader, dataSourceFullPath: String, dataSourceLastModifiedDate: Long, firstContainingEntryIn: AnyRef,
                  creatingNewStartingGroupFromTheFilenameIn: Boolean, addingToExistingGroup: Boolean,
                  putEntriesAtEnd: Boolean, makeThemPublicIn: Option[Boolean], mixedClassesAllowedDefaultIn: Boolean = false, testing: Boolean = false) {
    var r: LineNumberReader = null
    r = new LineNumberReader(dataSourceIn)
    val containingEntry: AnyRef = {
      firstContainingEntryIn match {
        case containingEntity: Entity =>
          if (creatingNewStartingGroupFromTheFilenameIn) {
            val group: Group = containingEntity.createGroupAndAddHASRelationToIt(dataSourceFullPath,
                                                                                 mixedClassesAllowedIn = mixedClassesAllowedDefaultIn,
                                                                                 System.currentTimeMillis, callerManagesTransactionsIn = true)._1
            group
          } else containingEntity
        case containingRtg: RelationToGroup =>
          if (creatingNewStartingGroupFromTheFilenameIn) {
            val containingGroup: Group = new Group(mDB, containingRtg.getGroupId)
            val name = dataSourceFullPath
            val newEntity: Entity = createAndAddEntityToGroup(name, containingGroup, mDB.findUnusedSortingIndex(containingGroup.getId), makeThemPublicIn)
            val newGroup: Group = newEntity.createGroupAndAddHASRelationToIt(name, containingGroup.getMixedClassesAllowed, System.currentTimeMillis,
                                                                             callerManagesTransactionsIn = true)._1
            newGroup
          } else {
            assert(addingToExistingGroup)
            // importing the new entries to an existing group
            new Group(mDB, containingRtg.getGroupId)
          }
        case _ => throw new OmException("??")
      }
    }
    // how manage this (& others like it) better using scala type system?:
    require(containingEntry.isInstanceOf[Entity] || containingEntry.isInstanceOf[Group])
    // in order to put the new entries at the end of those already there, find the last used sortingIndex, and use the next one (renumbering
    // if necessary (idea: make this optional: putting them at beginning (w/ mDB.minIdValue) or end (w/ highestCurrentSortingIndex)).
    val startingSortingIndex: Long = {
      if (addingToExistingGroup && putEntriesAtEnd) {
        val containingGrp = containingEntry.asInstanceOf[Group]
        val nextSortingIndex: Long = containingGrp.getHighestSortingIndex + 1
        if (nextSortingIndex == mDB.minIdValue) {
          // we wrapped from the biggest to lowest Long value
          mDB.renumberGroupSortingIndexes(containingGrp.getId)
          val nextTriedNewSortingIndex: Long = containingGrp.getHighestSortingIndex + 1
          if (nextSortingIndex == mDB.minIdValue) {
            throw new OmException("Huh? How did we get two wraparounds in a row?")
          }
          nextTriedNewSortingIndex
        } else nextSortingIndex
      } else mDB.minIdValue
    }

    handleRestOfLines(r, None, 0, containingEntry :: Nil, startingSortingIndex :: Nil, dataSourceLastModifiedDate, mixedClassesAllowedDefaultIn,
                      makeThemPublicIn)
  }

}
