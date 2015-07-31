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
import java.nio.file.{Files, Path}

import org.onemodel.controller.{Controller, EntityMenu, QuickGroupMenu}
import org.onemodel.database.PostgreSQLDatabase
import org.onemodel.model._

import scala.annotation.tailrec
import scala.collection.mutable

object ImportExport {
  val TEXT_EXPORT_TYPE: String = "text"
  val HTML_EXPORT_TYPE: String = "html"
}

class ImportExport(val ui: TextUI, val db: PostgreSQLDatabase, controller: Controller) {
  // idea: see comment in EntityMenu about scoping.
  def exportToFile(entity: Entity, exportTypeIn: String, copyrightYearAndNameIn: Option[String]) {
    val ans: Option[String] = ui.askForString(Some(Array("Enter number of levels to export (including this one; 0 = 'all'); ESC to cancel")), Some(controller.isNumeric), Some("0"))
    if (ans.isDefined) {
      val levelsToExport: Int = ans.get.toInt

      val ans2: Option[Boolean] = ui.askYesNoQuestion("Include metadata (verbose detail: id's, types...)?")
      if (ans2.isDefined) {
        val includeMetadata: Boolean = ans2.get

        //idea: make these choice strings into an enum? and/or the answers into an enum? what's the scala idiom? see same issue elsewhere
        val includePublicData: Option[Boolean] = ui.askYesNoQuestion("Include public data?", Some("y"))
        val includeNonPublicData: Option[Boolean] = ui.askYesNoQuestion("Include data marked non-public?", Some("y"))
        val includeUnspecifiedData: Option[Boolean] = ui.askYesNoQuestion("Include data not specified as public or non-public?", Some("y"))

        if (includePublicData.isDefined && includeNonPublicData.isDefined && includeUnspecifiedData.isDefined) {
          require(levelsToExport >= 0)
          val spacesPerIndentLevel = 2
          val exportedEntities = new scala.collection.mutable.TreeSet[Long]()

          val prefix: String = getExportFileNamePrefix(entity, exportTypeIn)
          val outputDirectory:Path = createOutputDir(prefix)
          if (exportTypeIn == ImportExport.TEXT_EXPORT_TYPE) {
            val (outputFile: File, outputWriter: PrintWriter) = createOutputFile(prefix, exportTypeIn, Some(outputDirectory))
            try {
              doTheExport(entity, levelsToExport == 0, levelsToExport, 0, Some(outputWriter), Some(outputDirectory), includeMetadata,
                          exportTypeIn, exportedEntities,
                          spacesPerIndentLevel, includePublicData, includeNonPublicData, includeUnspecifiedData, copyrightYearAndNameIn)
              // flush before we report 'done' to the user:
              outputWriter.close()
              ui.displayText("Exported to file: " + outputFile.getCanonicalPath)
            } finally {
              if (outputWriter != null) {
                try outputWriter.close()
                catch {
                  case e: Exception =>
                  // ignore
                }
              }
            }
          } else if (exportTypeIn == ImportExport.HTML_EXPORT_TYPE) {
            doTheExport(entity, levelsToExport == 0, levelsToExport, 0, None, Some(outputDirectory), includeMetadata,
                        exportTypeIn, exportedEntities,
                        spacesPerIndentLevel, includePublicData, includeNonPublicData, includeUnspecifiedData, copyrightYearAndNameIn)
            ui.displayText("Exported to directory: " + outputDirectory.toFile.getCanonicalPath)
          }
        }
      }
    }
  }

  val uriLineExample: String = "'nameForTheLink <uri>http://somelink.org/index.html</uri>'"

  /**
   * 1st parameter must be either an Entity or a RelationToGroup (what is the right way to do that, in the signature?).
   */
  def importCollapsibleOutlineAsGroups(firstContainingEntryIn: AnyRef) {
    require(firstContainingEntryIn.isInstanceOf[Entity] || firstContainingEntryIn.isInstanceOf[RelationToGroup])
    val ans1: Option[String] = ui.askForString(Some(Array("Enter file path (must exist, be readable, AND a text file with lines spaced in the form of a" +
                                                          " collapsible outline where each level change is marked by 1 tab or 2 spaces; textAttribute content" +
                                                          " can be indicated by surrounding a body of text thus, without quotes: '<ta>text</ta>';" +
                                                          " a URI similarly with a line " + uriLineExample + ")," +
                                                          " then press Enter; ESC to cancel")),
                                               Some(controller.inputFileValid))
    if (ans1.isDefined) {
      val path = ans1.get
      val makeThemPublic: Option[Boolean] = ui.askYesNoQuestion("Do you want the entities imported to be marked as public?  Set it to the value the " +
                                                      "majority of imported data should have; you can then edit the individual exceptions afterward as " +
                                                      "needed.  Enter y for public, n for nonpublic, or a space for 'unknown/unspecified', aka decide later.",
                                                      Some(""), allowBlankAnswer = true)
      val ans3 = ui.askYesNoQuestion("Keep the filename as the top level of the imported list? (Answering no will put the top level entries from inside" +
                                     " the " +
                                     "file, as entries directly under this entity or group.)")
      if (ans3.isDefined) {
        val creatingNewStartingGroupFromTheFilename: Boolean = ans3.get
        val addingToExistingGroup: Boolean = firstContainingEntryIn.isInstanceOf[RelationToGroup] && !creatingNewStartingGroupFromTheFilename

        val putEntriesAtEndOption: Option[Boolean] = {
          if (addingToExistingGroup) {
            ui.askYesNoQuestion("Put the new entries at the end of the list? (No means put them at the beginning, the default.)")
          } else
            Some(false)
        }

        if (putEntriesAtEndOption.isDefined) {
          //@tailrec: would be nice to use, but jvm doesn't support it, or something.
          def tryIt() {
            var reader: Reader = null
            try {
              val putEntriesAtEnd: Boolean = putEntriesAtEndOption.get
              val fileToImport = new File(path)
              reader = new FileReader(fileToImport)
              db.beginTrans()

              doTheImport(reader, fileToImport.getCanonicalPath, fileToImport.lastModified(), firstContainingEntryIn, creatingNewStartingGroupFromTheFilename,
                          addingToExistingGroup, putEntriesAtEnd, makeThemPublic)

              val keepAnswer: Option[Boolean] = {
                //idea: look into how long that time is (see below same cmt):
                val msg: String = "Group imported, but browse around to see if you want to keep it, " +
                                  "then ESC back here to commit the changes....  (If you wait beyond some amount of time(?), " +
                                  "it seems that postgres will commit " +
                                  "the change whether you want it or not, even if the message at that time says 'rolled back...')"
                ui.displayText(msg)
                firstContainingEntryIn match {
                  case entity: Entity => new EntityMenu(ui, db, controller).entityMenu(0, entity)
                  case rtg: RelationToGroup => new QuickGroupMenu(ui, db, controller).quickGroupMenu(0, firstContainingEntryIn
                                                                                             .asInstanceOf[RelationToGroup])
                  case _ => throw new OmException("??")
                }
                ui.askYesNoQuestion("Do you want to commit the changes as they were made?")
              }
              if (keepAnswer.isEmpty || !keepAnswer.get) {
                db.rollbackTrans()
                //idea: look into how long that time is (see above same cmt)
                ui.displayText("Rolled back the import: no changes made (unless you waited too long and postgres committed it anyway...?).")
              } else {
                db.commitTrans()
              }
            } catch {
              case e: Exception =>
                db.rollbackTrans()
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
                if (ans.isDefined && ans.get) {
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
    else if (index >= line.length || (line(index) != ' ')) {
      index
    } else {
      getFirstNonSpaceIndex(line, index + 1)
    }
  }

  def createAndAddEntityToGroup(line: String, group: Group, newSortingIndex: Long, isPublicIn: Option[Boolean]): Entity = {
    val entityId: Long = db.createEntity(line.trim, group.getClassId, isPublicIn)
    db.addEntityToGroup(group.getId, entityId, Some(newSortingIndex), callerManagesTransactionsIn = true)
    new Entity(db, entityId)
  }

  /* The parameter lastEntityIdAdded means the one to which a new subgroup will be added, such as in a series of entities
     added to a list and the code needs to know about the most recent one, so if the line is further indented, it knows where to
     create the subgroup.

     We always start from the current container (entity or group) and add the new material to a entry (Entity (+ 1 subgroup if needed)) created there.

     The parameter lastIndentationlevel should be set to zero, from the original caller, and indent from there w/ the recursion.
  */
  @tailrec
  private def processRestOfLines(r: LineNumberReader, lastEntityAdded: Option[Entity], lastIndentationLevel: Int, containerList: List[AnyRef],
                                lastSortingIndexes: List[Long], observationDateIn: Long, mixedClassesAllowedDefaultIn: Boolean,
                                makeThemPublicIn: Option[Boolean]) {
    // (see cmts just above about where we start)
    require(containerList.size == lastIndentationLevel + 1)
    // always should at least have an entry for the entity or group from where the user initiated this import, the base of all the adding.
    require(containerList.nonEmpty)
    // how do this type mgt better, like, in the signature? (also needed elsewhere):
    require(containerList.head.isInstanceOf[Entity] || containerList.head.isInstanceOf[Group])

    val spacesPerIndentLevel = 2
    val lineUntrimmed: String = r.readLine()
    if (lineUntrimmed != null) {
      val lineNumber = r.getLineNumber

      // these indicate beg/end of TextAttribute content; CODE ASSUMES THEY ARE LOWER-CASE!, so making that explicit, to be sure in case we change them later.
      val beginTaMarker = "<ta>".toLowerCase
      val endTaMarker = "</ta>".toLowerCase
      val beginUriMarker = "<uri>".toLowerCase
      val endUriMarker = "</uri>".toLowerCase

      if (lineUntrimmed.toLowerCase.contains(beginTaMarker)) {
        // we have a section of text marked for importing into a single TextAttribute:
        importTextAttributeContent(lineUntrimmed, r, lastEntityAdded.get.getId, beginTaMarker, endTaMarker)
        processRestOfLines(r, lastEntityAdded, lastIndentationLevel, containerList, lastSortingIndexes, observationDateIn, mixedClassesAllowedDefaultIn,
                          makeThemPublicIn)
      } else if (lineUntrimmed.toLowerCase.contains(beginUriMarker)) {
        // we have a section of text marked for importing into a web link:
        importUriContent(lineUntrimmed, beginUriMarker, endUriMarker, lineNumber, lastEntityAdded.get, observationDateIn,
                          makeThemPublicIn, callerManagesTransactionsIn = true)
        processRestOfLines(r, lastEntityAdded, lastIndentationLevel, containerList, lastSortingIndexes, observationDateIn, mixedClassesAllowedDefaultIn,
                          makeThemPublicIn)
      } else {
        val line: String = lineUntrimmed.trim

        if (line == "." || line.isEmpty) {
          // nothing to do: that kind of line was just to create whitespace in my outline. So simply go to the next line:
          processRestOfLines(r, lastEntityAdded, lastIndentationLevel, containerList, lastSortingIndexes, observationDateIn, mixedClassesAllowedDefaultIn,
                            makeThemPublicIn)
        } else {
          if (line.length > controller.maxNameLength) throw new OmException("Line " + lineNumber + " is over " + controller.maxNameLength + " characters " +
                                                               " (has " + line.length + "): " + line)
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

            processRestOfLines(r, Some(newEntity), lastIndentationLevel, containerList, newSortingIndex :: lastSortingIndexes.tail, observationDateIn,
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
            processRestOfLines(r, Some(newEntity), newIndentationLevel, newContainerList, newSortingIndex :: newSortingIndexList.tail, observationDateIn,
                              mixedClassesAllowedDefaultIn, makeThemPublicIn)
          } else if (newIndentationLevel > lastIndentationLevel) {
            // indented, so create a subgroup & add line there:
            require(newIndentationLevel >= 0)
            // (not None because it will be used now to create a subgroup; when we get here there should always be a value) :
            if (lastEntityAdded.isEmpty) {
              throw new OmException("There's an error.  Are you importing a file to a group, but the first line is indented?  If so try fixing that " +
                                    "(un-indent, & fix the rest to match).  Otherwise, there's a bug in the program.")
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
            val newSortingIndex = db.minIdValue
            val newSubEntity: Entity = createAndAddEntityToGroup(line, newGroup, newSortingIndex, makeThemPublicIn)
            processRestOfLines(r, Some(newSubEntity), newIndentationLevel, newGroup :: containerList, newSortingIndex :: lastSortingIndexes,
                              observationDateIn, mixedClassesAllowedDefaultIn, makeThemPublicIn)
          } else throw new OmException("Shouldn't get here!?: " + lastIndentationLevel + ", " + newIndentationLevel)
        }
      }
    }
  }

  def importTextAttributeContent(lineUntrimmedIn: String, r: LineNumberReader, entityId: Long, beginningTagMarker: String, endTaMarker: String) {
    val lineContentBeforeMarker = lineUntrimmedIn.substring(0, lineUntrimmedIn.toLowerCase.indexOf(beginningTagMarker)).trim
    val restOfLine = lineUntrimmedIn.substring(lineUntrimmedIn.toLowerCase.indexOf(beginningTagMarker) + beginningTagMarker.length).trim
    if (restOfLine.toLowerCase.contains(endTaMarker)) throw new OmException("\"Unsupported format at line " + r.getLineNumber + ": beginning and ending " +
                                                                            "markers must NOT be on the same line.")
    val attrTypeId: Long = {
      val idsByName: Option[List[Long]] = db.findAllEntityIdsByName(lineContentBeforeMarker.trim, caseSensitive = true)
      if (idsByName.isDefined && idsByName.get.size == 1)
        idsByName.get.head
      else {
        // idea: alternatively, could use a generic one in this case?  Optionally?
        val prompt = "A name for the *type* of this text attribute was not provided; it would be the entire line content preceding the \"" +
                     beginningTagMarker + "\" " +
                     "(it has to match an existing entity, case-sensitively)"
        val typeId = controller.askForAttributeTypeId(prompt + ", so please choose one or ESC to abort this import operation:", Controller.TEXT_TYPE, None, None)
        if (typeId.isEmpty)
          throw new OmException(prompt + " or selected.")
        else
          typeId.get
      }
    }
    val text: String = restOfLine.trim + TextUI.NEWLN + {
      def getRestOfLines(rIn: LineNumberReader, sbIn: mutable.StringBuilder): mutable.StringBuilder = {
        // Don't trim, because we want to preserve formatting/whitespace here, including blank lines (always? -- yes, editably.).
        val line = rIn.readLine()
        if (line == null) {
          sbIn
        } else {
          if (line.toLowerCase.contains(endTaMarker.toLowerCase)) {
            val markerStartLocation = line.toLowerCase.indexOf(endTaMarker.toLowerCase)
            val markerEndLocation = markerStartLocation + endTaMarker.length
            val lineNumber = r.getLineNumber
            def rtrim(s: String): String = s.replaceAll("\\s+$", "")
            val rtrimmedLine = rtrim(line)
            if (rtrimmedLine.substring(markerEndLocation).nonEmpty) throw new OmException("\"Unsupported format at line " + lineNumber +
                                                                                  ": A \"" + endTaMarker +
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
    db.createTextAttribute(entityId, attrTypeId, text)
  }

  def importUriContent(lineUntrimmedIn: String, beginningTagMarkerIn: String, endMarkerIn: String, lineNumberIn: Int,
                        lastEntityAddedIn: Entity, observationDateIn: Long, makeThemPublicIn: Option[Boolean], callerManagesTransactionsIn: Boolean) {
    //NOTE/idea also in tasks: this all fits better in the class and action *tables*, with this code being stored there
    // also, which implies that the class doesn't need to be created because...it's already there.

    if (! lineUntrimmedIn.toLowerCase.contains(endMarkerIn)) throw new OmException("\"Unsupported format at line " + lineNumberIn + ": beginning and ending " +
                                                                                   "markers MUST be on the same line.")
    val lineContentBeforeMarker = lineUntrimmedIn.substring(0, lineUntrimmedIn.toLowerCase.indexOf(beginningTagMarkerIn)).trim
    val lineContentFromBeginMarker = lineUntrimmedIn.substring(lineUntrimmedIn.toLowerCase.indexOf(beginningTagMarkerIn)).trim
    val uriStartLocation: Int = lineContentFromBeginMarker.toLowerCase.indexOf(beginningTagMarkerIn.toLowerCase) + beginningTagMarkerIn.length
    val uriEndLocation: Int = lineContentFromBeginMarker.toLowerCase.indexOf(endMarkerIn.toLowerCase)
    if (lineContentFromBeginMarker.substring(uriEndLocation + endMarkerIn.length).trim.nonEmpty) {
      throw new OmException("\"Unsupported format at line " + lineNumberIn + ": A \"" + endMarkerIn + "\" (end URI attribute) marker " +
                            "must be the" + " last text on its line.")
    }
    val name = lineContentBeforeMarker.trim
    val uri = lineContentFromBeginMarker.substring(uriStartLocation, uriEndLocation).trim
    if (name.isEmpty || uri.isEmpty) throw new OmException("\"Unsupported format at line " + lineNumberIn +
                                                           ": A URI line must be in the format (without quotes): " + uriLineExample)
    // (see note above on this being better in the class and action *tables*, but here for now until those features are ready)
    var uriClass: Option[Long]  = db.findFIRSTClassIdByName("URI", caseSensitive = true)
    if (uriClass.isEmpty) {
      val (classId, _) = db.createClassAndItsDefiningEntity("URI")
      uriClass = Some(classId)
    }
    val newEntity: Entity = lastEntityAddedIn.createEntityAndAddHASRelationToIt(name, observationDateIn, makeThemPublicIn,
                                                                                callerManagesTransactionsIn = true)._1
    db.updateEntityOnlyClass(newEntity.getId, Some(uriClass.get), callerManagesTransactionsIn)
    newEntity.addTextAttribute(uriClass.get, uri, None, observationDateIn)
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
            val containingGroup: Group = new Group(db, containingRtg.getGroupId)
            val name = dataSourceFullPath
            val newEntity: Entity = createAndAddEntityToGroup(name, containingGroup, db.findUnusedSortingIndex(containingGroup.getId), makeThemPublicIn)
            val newGroup: Group = newEntity.createGroupAndAddHASRelationToIt(name, containingGroup.getMixedClassesAllowed, System.currentTimeMillis,
                                                                             callerManagesTransactionsIn = true)._1
            newGroup
          } else {
            assert(addingToExistingGroup)
            // importing the new entries to an existing group
            new Group(db, containingRtg.getGroupId)
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
        if (nextSortingIndex == db.minIdValue) {
          // we wrapped from the biggest to lowest Long value
          db.renumberGroupSortingIndexes(containingGrp.getId)
          val nextTriedNewSortingIndex: Long = containingGrp.getHighestSortingIndex + 1
          if (nextSortingIndex == db.minIdValue) {
            throw new OmException("Huh? How did we get two wraparounds in a row?")
          }
          nextTriedNewSortingIndex
        } else nextSortingIndex
      } else db.minIdValue
    }

    processRestOfLines(r, None, 0, containingEntry :: Nil, startingSortingIndex :: Nil, dataSourceLastModifiedDate, mixedClassesAllowedDefaultIn,
                      makeThemPublicIn)
  }

  //@tailrec  THIS IS NOT TO BE TAIL RECURSIVE UNTIL IT'S KNOWN HOW TO MAKE SOME CALLS to it BE recursive, AND SOME *NOT* TAIL RECURSIVE (because some of them
  //*do* need to return & finish their work, such as when iterating through the entities & subgroups)! (but test it: is it really a problem?)
  // (Idea: See note at the top of Controller.chooseOrCreateObject re inAttrType about similarly making exportTypeIn an enum.)
  /** If exporting html, this creates a new file for each entity; if exporting txt format, it all goes in a single file together (a
    * collapsible outline).
    *
    * The parm outputWriterOptionIn is required for either txt (created once at the beginning) or html ( a new file for each entity: we wait to create each
    * .html file until we are ready to write to it; otherwise it was creating duplicates by doing the file creation before the duplicate check.
    *
    * If levelsToProcessIsInfiniteIn is true, then levelsRemainingToProcessIn is irrelevant.
    *
    * Returns whether the entity in question was exported, so that the caller can know whether to include a link to that exported information (such as
    * to an html page).
    */
  def doTheExport(entityIn: Entity, levelsToExportIsInfiniteIn: Boolean, levelsRemainingToExportIn: Int, currentIndentationLevelsIn: Int, outputWriterOptionIn: Option[PrintWriter],
                  outputDirectoryIn: Option[Path],
                  includeMetadataIn: Boolean, exportTypeIn: String, exportedEntitiesIn: scala.collection.mutable.TreeSet[Long], spacesPerIndentLevelIn: Int,
                  includePublicDataIn: Option[Boolean], includeNonPublicDataIn: Option[Boolean], includeUnspecifiedDataIn: Option[Boolean],
                  copyrightYearAndNameIn: Option[String]): Boolean = {
    // useful while debugging:
    //out.flush()

    // see note about this usage, in method importUriContent.
    val uriClassId: Option[Long] = db.findFIRSTClassIdByName("URI", caseSensitive = true)

    if (exportTypeIn == ImportExport.TEXT_EXPORT_TYPE) {
      require(outputDirectoryIn.isEmpty, "No html directory needed, but text file is needed, if exporting text format.")
      require(outputWriterOptionIn.isDefined, "Text output requires a valid output file definition, up front.")
    } else if (exportTypeIn == ImportExport.HTML_EXPORT_TYPE) {
      require(outputDirectoryIn.isDefined, "For each entity there should be an output file & a directory, if exporting html.")
      require(outputWriterOptionIn.isEmpty, "Html output requires NO valid output file definition, up front; we will create it as we go, for each entity.")
    } else {
      throw new OmException("unexpected value for exportTypeIn: " + exportTypeIn)
    }

    if (includeMetadataIn)  require(exportTypeIn == ImportExport.TEXT_EXPORT_TYPE, "Metadata is currently only set up to be print with text format output.")

    if (exportedEntitiesIn.contains(entityIn.getId)) {
      if (exportTypeIn == ImportExport.TEXT_EXPORT_TYPE) {
        printSpaces(currentIndentationLevelsIn * spacesPerIndentLevelIn, outputWriterOptionIn.get)
        if (includeMetadataIn) outputWriterOptionIn.get.print("(duplicate: EN --> " + entityIn.getId + ": ")
        outputWriterOptionIn.get.print(entityIn.getName)
        if (includeMetadataIn) outputWriterOptionIn.get.print(")")
        outputWriterOptionIn.get.println()
      }
      true
    } else {
      val entityPublicStatus: Option[Boolean] = entityIn.getPublic
      val allowedToExport = (entityPublicStatus.isDefined && entityPublicStatus.get && includePublicDataIn.get) ||
                            (entityPublicStatus.isDefined && !entityPublicStatus.get && includeNonPublicDataIn.get) ||
                            (entityPublicStatus.isEmpty && includeUnspecifiedDataIn.get)
      if (allowedToExport && (levelsToExportIsInfiniteIn || levelsRemainingToExportIn > 0)) {
        val entitysFileNamePrefix: String = getExportFileNamePrefix(entityIn, exportTypeIn)
        val printWriter = {
          if (exportTypeIn == ImportExport.TEXT_EXPORT_TYPE) outputWriterOptionIn.get
          else if (exportTypeIn == ImportExport.HTML_EXPORT_TYPE) createOutputFile(entitysFileNamePrefix, exportTypeIn, outputDirectoryIn)._2
          else throw new OmException("unexpected value for exportTypeIn: " + exportTypeIn)
        }
        try {
          // record, so we don't create duplicate files, etc:
          exportedEntitiesIn.add(entityIn.getId)

          if (exportTypeIn == ImportExport.HTML_EXPORT_TYPE) printWriter.println("<html><body>")

          val entityName: String = entityIn.getName
          if (exportTypeIn == ImportExport.TEXT_EXPORT_TYPE) {
            printSpaces(currentIndentationLevelsIn * spacesPerIndentLevelIn, printWriter)
            if (includeMetadataIn) printWriter.println("EN " + entityIn.getId + ": " + entityIn.getDisplayString)
            else printWriter.println(entityName)
          } else if (exportTypeIn == ImportExport.HTML_EXPORT_TYPE) {
            printWriter.println("<h1>" + entityIn.getName + "</h1>")
          }

          val attributeObjList: java.util.ArrayList[Attribute] = db.getSortedAttributes(entityIn.getId, 0, 0)._1
          if (exportTypeIn == ImportExport.HTML_EXPORT_TYPE) printWriter.println("<ul>")
          for (attribute: Attribute <- attributeObjList.toArray(Array[Attribute]())) yield attribute match {
            case relation: RelationToEntity =>
              val relationType = new RelationType(db, relation.getAttrTypeId)
              val entity2 = new Entity(db, relation.getRelatedId2)
              if (uriClassId.isDefined && exportTypeIn == ImportExport.HTML_EXPORT_TYPE && entity2.getClassId == uriClassId) {
                // handle URIs differently than other entities: make it a link as indicated by the URI contents, not to a newly created entity page..
                // (could use a more efficient call in cpu time than getSortedAttributes, but it's efficient in programmer time:)
                def findUriAttribute(): Option[TextAttribute] = {
                  val (attributesOnEntity2: java.util.ArrayList[Attribute], _) = db.getSortedAttributes(entity2.getId, 0, 0)
                  for (attr2: Attribute <- attributesOnEntity2.toArray(Array[Attribute]())) {
                    if (attr2.getAttrTypeId == uriClassId.get && attr2.isInstanceOf[TextAttribute]) {
                      return Some(attr2.asInstanceOf[TextAttribute])
                    }
                  }
                  None
                }
                val uriAttribute: Option[TextAttribute] = findUriAttribute()
                if (uriAttribute.isEmpty) {
                  throw new OmException("Unable to find TextAttribute of type URI (classId=" + uriClassId.get + ") for entity " + entity2.getId)
                }
                printHtmlLink(printWriter, "", uriAttribute.get.getText, entity2.getName)
              }
              else {
                if (includeMetadataIn) {
                  printSpaces((currentIndentationLevelsIn + 1) * spacesPerIndentLevelIn, printWriter)
                  printWriter.println(attribute.getDisplayString(0, Some(entity2), Some(relationType)))
                }
                val relatedEntitysFileNamePrefix: String = getExportFileNamePrefix(entity2, exportTypeIn)
                val exported: Boolean = doTheExport(entity2, levelsToExportIsInfiniteIn, levelsRemainingToExportIn - 1,
                                                    currentIndentationLevelsIn + 1, 
                                                    {
                                                      if (exportTypeIn == ImportExport.TEXT_EXPORT_TYPE) Some(printWriter)
                                                      else if (exportTypeIn == ImportExport.HTML_EXPORT_TYPE) None
                                                      else throw new OmException("unexpected value for exportTypeIn: " + exportTypeIn)
                                                    },
                                                    outputDirectoryIn, includeMetadataIn, exportTypeIn, exportedEntitiesIn, spacesPerIndentLevelIn,
                                                    includePublicDataIn, includeNonPublicDataIn, includeUnspecifiedDataIn, copyrightYearAndNameIn)
                if (exported && exportTypeIn == ImportExport.HTML_EXPORT_TYPE) {
                  // meaning that we just created an html file, so now let's create a link to it:
                  // (Create this link, even if the # of subentries is 0, because there might be other useful info on the page (sometime).)
                  printHtmlLink(printWriter, relationType.getName + ": ", relatedEntitysFileNamePrefix + ".html", entity2.getName, Some("(" +
                                                                                                                                        getNumSubEntries
                                                                                                                                        (entity2) + ")"))
                }
              }
            case relation: RelationToGroup =>
              val relationType = new RelationType(db, relation.getAttrTypeId)
              val group = new Group(db, relation.getGroupId)
              val grpName = group.getName

              // if a group name is different from its entity name, indicate the differing group name also, otherwise complete the line just above w/ NL
              if (entityName != grpName) {
                printSpaces((currentIndentationLevelsIn + 1) * spacesPerIndentLevelIn, printWriter)
                printWriter.println("(group named: " + grpName + ")")
              }
              if (exportTypeIn == ImportExport.HTML_EXPORT_TYPE) printWriter.println("<ul>")

              if (includeMetadataIn) {
                printSpaces(currentIndentationLevelsIn * spacesPerIndentLevelIn, printWriter)
                // plus one more level of spaces to make it look better but still ~equivalently/exchangeably importable:
                printSpaces(spacesPerIndentLevelIn, printWriter)
                printWriter.println("(group details: " + attribute.getDisplayString(0, None, Some(relationType)) + ")")
              }
              for (entityInGrp: Entity <- group.getGroupEntries(0).toArray(Array[Entity]())) {
                val relatedGroupsEntitysFileNamePrefix: String = getExportFileNamePrefix(entityInGrp, exportTypeIn)
                val exported: Boolean = doTheExport(entityInGrp, levelsToExportIsInfiniteIn, levelsRemainingToExportIn - 1,
                                                    currentIndentationLevelsIn + 1,
                                                    {
                                                      if (exportTypeIn == ImportExport.TEXT_EXPORT_TYPE) Some(printWriter)
                                                      else if (exportTypeIn == ImportExport.HTML_EXPORT_TYPE) None
                                                      else throw new OmException("unexpected value for exportTypeIn: " + exportTypeIn)
                                                    },
                                                    outputDirectoryIn, includeMetadataIn, exportTypeIn, exportedEntitiesIn, spacesPerIndentLevelIn,
                                                    includePublicDataIn, includeNonPublicDataIn, includeUnspecifiedDataIn, copyrightYearAndNameIn)
                if (exported && exportTypeIn == ImportExport.HTML_EXPORT_TYPE) {
                  // meaning that we just created an html file, so now let's create a link to it:
                  // (Create this link, even if the # of subentries is 0, because there might be other useful info on the page (sometime).)
                  printHtmlLink(printWriter, relationType.getName + ": ", relatedGroupsEntitysFileNamePrefix + ".html", entityInGrp.getName, Some("(" +
                                                                                                                                                  getNumSubEntries(entityInGrp) + ")"))
                }
              }
              if (exportTypeIn == ImportExport.HTML_EXPORT_TYPE) printWriter.println("</ul>")
            case _ =>
              printSpaces((currentIndentationLevelsIn + 1) * spacesPerIndentLevelIn, printWriter)
              if (includeMetadataIn) {
                printWriter.println((attribute match {
                  case ba: BooleanAttribute => "BA "
                  case da: DateAttribute => "DA "
                  case fa: FileAttribute => "FA "
                  case qa: QuantityAttribute => "QA "
                  case ta: TextAttribute => "TA "
                }) + /*attribute.getId +*/ ": " + attribute.getDisplayString(0, None, None))
              } else printWriter.println(attribute.getDisplayString(0, None, None))
          }

          if (exportTypeIn == ImportExport.HTML_EXPORT_TYPE) printWriter.println("</ul>")

          if (exportTypeIn == ImportExport.HTML_EXPORT_TYPE) {
            if (copyrightYearAndNameIn.isDefined) {
              printWriter.println("<center><small>Copyright " + copyrightYearAndNameIn.get + "; all rights reserved.</small></center>")
            }
            printWriter.println("</html></body>")
            printWriter.close()
          }
        } finally {
          if (exportTypeIn == ImportExport.HTML_EXPORT_TYPE) {
            // close each html file as we go along.
            if (printWriter != null) {
              try printWriter.close()
              catch {
                case e: Exception =>
                // ignore
              }
            }
          }
        }
      }
      true
    }
  }

  def printHtmlLink(printWriterIn: PrintWriter, preLabel: String, uri: String, linkDisplayText: String, suffix: Option[String] = None): Unit = {
    printWriterIn.print("<li>")
    printWriterIn.print(preLabel + "<a href=\"" + uri + "\">" + linkDisplayText + "</a>" + " " + suffix.getOrElse(""))
    printWriterIn.println("</li>")
  }

  def getNumSubEntries(entityIn: Entity): Long = {
    val numSubEntries = {
      val numAttrs = db.getAttrCount(entityIn.getId)
      if (numAttrs == 1) {
        val (_, groupId, moreThanOneAvailable) = db.findRelationToAndGroup_OnEntity(entityIn.getId)
        if (groupId.isDefined && !moreThanOneAvailable) {
          db.getGroupEntryCount(groupId.get, Some(false))
        } else numAttrs
      } else numAttrs
    }
    numSubEntries
  }

  def printSpaces(num: Int, out: PrintWriter) {
    for (i <- 1 to num) {
      out.print(" ")
    }
  }

  def getExportFileNamePrefix(entity: Entity, exportTypeIn: String): String = {
    if (exportTypeIn == ImportExport.HTML_EXPORT_TYPE) {
      // (The 'e' is for "entity"; for explanation see cmts in methods createOutputDir and createOutputFile.)
      "e" + entity.getId.toString
    } else {
      //idea (also in task list): change this to be a reliable filename (incl no backslashes? limit it to a whitelist of chars? a simple fn for that?
      var fixedEntityName = entity.getName.replace(" ", "")
      fixedEntityName = fixedEntityName.replace("/", "-")
      //fixedEntityName = fixedEntityName.replace("\\","-")
      "onemodel-export_" + entity.getId + "_" + fixedEntityName + "-"
    }
  }

  def createOutputDir(prefix: String): Path = {
    // even though entityIds start with a '-', it's a problem if a filename does (eg, "ls" cmd thinks it is an option, not a name):
    // (there's a similar line elsewhere)
    require(!prefix.startsWith("-"))
    // hyphen after the prefix is in case one wants to see where the id ends & the temporary/generated name begins, for understanding/diagnosing things:
    Files.createTempDirectory(prefix + "-")
  }

  def createOutputFile(prefix:String, exportTypeIn: String, exportDirectory: Option[Path]): (File, PrintWriter) = {
    // even though entityIds start with a '-', it's a problem if a filename does (eg, "ls" cmd thinks it is an option, not a name):
    // (there's a similar line elsewhere)
    require(!prefix.startsWith("-"))

    // make sure we have a place to put all the html files, together:
    if (exportTypeIn == ImportExport.HTML_EXPORT_TYPE) require(exportDirectory.isDefined && exportDirectory.get.toFile.isDirectory)

    val extension: String = {
      if (exportTypeIn == ImportExport.TEXT_EXPORT_TYPE) ".txt"
      else if (exportTypeIn == ImportExport.HTML_EXPORT_TYPE) ".html"
      else throw new OmException("unexpected exportTypeIn: " + exportTypeIn)
    }

    val outputFile: File =
      if (exportTypeIn == ImportExport.HTML_EXPORT_TYPE ) {
        Files.createFile(new File(exportDirectory.get.toFile, prefix + extension).toPath).toFile
      } else if (exportTypeIn == ImportExport.TEXT_EXPORT_TYPE) {
        Files.createTempFile(exportDirectory.get, prefix, extension).toFile
      }
      else throw new OmException("unexpected exportTypeIn: " + exportTypeIn)

    val output: PrintWriter = new PrintWriter(new BufferedWriter(new FileWriter(outputFile)))
    (outputFile, output)
  }

}
