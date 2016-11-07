/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2014-2016 inclusive, Luke A. Call; all rights reserved.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule, guidelines around binary
    distribution, and the GNU Affero General Public License as published by the Free Software Foundation, either version 3
    of the License, or (at your option) any later version.  See the file LICENSE for details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
package org.onemodel.core.controllers

import java.io._
import java.nio.file.{Files, Path, StandardCopyOption}

import org.onemodel.core._
import org.onemodel.core.model._
import org.onemodel.core.database.PostgreSQLDatabase
import org.onemodel.core.{OmException, TextUI}

import scala.annotation.tailrec
import scala.collection.mutable

object ImportExport {
  val TEXT_EXPORT_TYPE: String = "text"
  val HTML_EXPORT_TYPE: String = "html"
}

/**
 * When adding features to this class, any eventual db call that creates a transaction needs to have the info 'callerManagesTransactionsIn = true' eventually
 * passed into it, from here, otherwise the rollback feature will fail.
 */
class ImportExport(val ui: TextUI, val db: PostgreSQLDatabase, controller: Controller) {
  val uriLineExample: String = "'nameForTheLink <uri>http://somelink.org/index.html</uri>'"

  /**
   * 1st parameter must be either an Entity or a RelationToGroup (what is the right way to do that, in the signature?).
   */
  def importCollapsibleOutlineAsGroups(firstContainingEntryIn: AnyRef) {
    //noinspection ComparingUnrelatedTypes
    require(firstContainingEntryIn.isInstanceOf[Entity] || firstContainingEntryIn.isInstanceOf[Group])
    val ans1: Option[String] = ui.askForString(Some(Array("Enter file path (must exist, be readable, AND a text file with lines spaced in the form of a" +
                                                          " collapsible outline where each level change is marked by 1 tab or 2 spaces; textAttribute content" +
                                                          " can be indicated by surrounding a body of text thus, without quotes: '<ta>text</ta>';" +
                                                          " a URI similarly with a line " + uriLineExample + ")," +
                                                          " then press Enter; ESC to cancel")),
                                               Some(Util.inputFileValid))
    if (ans1.isDefined) {
      val path = ans1.get
      val makeThemPublic: Option[Boolean] = ui.askYesNoQuestion("Do you want the entities imported to be marked as public?  Set it to the value the " +
                                                      "majority of imported data should have; you can then edit the individual exceptions afterward as " +
                                                      "needed.  Enter y for public, n for nonpublic, or a space for 'unknown/unspecified', aka decide later.",
                                                      Some(""), allowBlankAnswer = true)
      val ans3 = ui.askYesNoQuestion("Keep the filename as the top level of the imported list? (Answering no will put the top level entries from inside" +
                                     " the file, as entries directly under this entity or group; answering yes will create an entity for the file," +
                                     " and in it a group for the entries.)")
      if (ans3.isDefined) {
        val creatingNewStartingGroupFromTheFilename: Boolean = ans3.get
        //noinspection ComparingUnrelatedTypes
        val addingToExistingGroup: Boolean = firstContainingEntryIn.isInstanceOf[Group] && !creatingNewStartingGroupFromTheFilename

        val putEntriesAtEndOption: Option[Boolean] = {
          if (addingToExistingGroup) {
            ui.askYesNoQuestion("Put the new entries at the end of the list? (No means put them at the beginning, the default.)")
          } else
            Some(false)
        }

        if (putEntriesAtEndOption.isDefined) {
          //@tailrec: would be nice to use, but jvm doesn't support it, or something.
          def tryIt() {
            //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) w/in this method!
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
                  case entity: Entity => new EntityMenu(ui, db, controller).entityMenu(entity)
                  case group: Group => new QuickGroupMenu(ui, db, controller).quickGroupMenu(firstContainingEntryIn.asInstanceOf[Group], 0,
                                                                                             containingEntityIn = None)
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
  //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) w/in this method!
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
  //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) w/in this method!
  private def importRestOfLines(r: LineNumberReader, lastEntityAdded: Option[Entity], lastIndentationLevel: Int, containerList: List[AnyRef],
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
        importRestOfLines(r, lastEntityAdded, lastIndentationLevel, containerList, lastSortingIndexes, observationDateIn, mixedClassesAllowedDefaultIn,
                          makeThemPublicIn)
      } else if (lineUntrimmed.toLowerCase.contains(beginUriMarker)) {
        // we have a section of text marked for importing into a web link:
        importUriContent(lineUntrimmed, beginUriMarker, endUriMarker, lineNumber, lastEntityAdded.get, observationDateIn,
                          makeThemPublicIn, callerManagesTransactionsIn = true)
        importRestOfLines(r, lastEntityAdded, lastIndentationLevel, containerList, lastSortingIndexes, observationDateIn, mixedClassesAllowedDefaultIn,
                          makeThemPublicIn)
      } else {
        val line: String = lineUntrimmed.trim

        if (line == "." || line.isEmpty) {
          // nothing to do: that kind of line was just to create whitespace in my outline. So simply go to the next line:
          importRestOfLines(r, lastEntityAdded, lastIndentationLevel, containerList, lastSortingIndexes, observationDateIn, mixedClassesAllowedDefaultIn,
                            makeThemPublicIn)
        } else {
          if (line.length > Util.maxNameLength) throw new OmException("Line " + lineNumber + " is over " + Util.maxNameLength + " characters " +
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

            importRestOfLines(r, Some(newEntity), lastIndentationLevel, containerList, newSortingIndex :: lastSortingIndexes.tail, observationDateIn,
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
            importRestOfLines(r, Some(newEntity), newIndentationLevel, newContainerList, newSortingIndex :: newSortingIndexList.tail, observationDateIn,
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
            importRestOfLines(r, Some(newSubEntity), newIndentationLevel, newGroup :: containerList, newSortingIndex :: lastSortingIndexes,
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
        val typeId = controller.chooseOrCreateObject_OrSaysCancelled(prompt + ", so please choose one or ESC to abort this import operation:", Util.TEXT_TYPE, None, None)
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
    db.createTextAttribute(entityId, attrTypeId, text, callerManagesTransactionsIn = true)
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
    db.addUriEntityWithUriAttribute(lastEntityAddedIn, name, uri, observationDateIn, makeThemPublicIn, callerManagesTransactionsIn = true)
  }

  //@tailrec why not? needs that jvm fix first to work for the scala compiler?  see similar comments elsewhere on that? (does java8 provide it now?
  // wait for next debian stable version--jessie?--be4 it's probably worth finding out)
  def doTheImport(dataSourceIn: Reader, dataSourceFullPath: String, dataSourceLastModifiedDate: Long, firstContainingEntryIn: AnyRef,
                  creatingNewStartingGroupFromTheFilenameIn: Boolean, addingToExistingGroup: Boolean,
                  //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) w/in this method!
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
        case containingGroup: Group =>
          if (creatingNewStartingGroupFromTheFilenameIn) {
            val name = dataSourceFullPath
            val newEntity: Entity = createAndAddEntityToGroup(name, containingGroup, db.findUnusedGroupSortingIndex(containingGroup.getId), makeThemPublicIn)
            val newGroup: Group = newEntity.createGroupAndAddHASRelationToIt(name, containingGroup.getMixedClassesAllowed, System.currentTimeMillis,
                                                                             callerManagesTransactionsIn = true)._1
            newGroup
          } else {
            assert(addingToExistingGroup)
            // importing the new entries to an existing group
            new Group(db, containingGroup.getId)
          }
        case _ => throw new OmException("??")
      }
    }
    // how manage this (& others like it) better using scala type system?:
    //noinspection ComparingUnrelatedTypes
    require(containingEntry.isInstanceOf[Entity] || containingEntry.isInstanceOf[Group])
    // in order to put the new entries at the end of those already there, find the last used sortingIndex, and use the next one (renumbering
    // if necessary (idea: make this optional: putting them at beginning (w/ mDB.minIdValue) or end (w/ highestCurrentSortingIndex)).
    val startingSortingIndex: Long = {
      if (addingToExistingGroup && putEntriesAtEnd) {
        val containingGrp = containingEntry.asInstanceOf[Group]
        val nextSortingIndex: Long = containingGrp.getHighestSortingIndex + 1
        if (nextSortingIndex == db.minIdValue) {
          // we wrapped from the biggest to lowest Long value
          db.renumberSortingIndexes(containingGrp.getId, callerManagesTransactionsIn = true, isEntityAttrsNotGroupEntries = false)
          val nextTriedNewSortingIndex: Long = containingGrp.getHighestSortingIndex + 1
          if (nextSortingIndex == db.minIdValue) {
            throw new OmException("Huh? How did we get two wraparounds in a row?")
          }
          nextTriedNewSortingIndex
        } else nextSortingIndex
      } else db.minIdValue
    }

    importRestOfLines(r, None, 0, containingEntry :: Nil, startingSortingIndex :: Nil, dataSourceLastModifiedDate, mixedClassesAllowedDefaultIn,
                      makeThemPublicIn)
  }

  // idea: see comment in EntityMenu about scoping.
  def export(entity: Entity, exportTypeIn: String, headerContentIn: Option[String], beginBodyContentIn: Option[String], copyrightYearAndNameIn: Option[String]) {
    val ans: Option[String] = ui.askForString(Some(Array("Enter number of levels to export (including this one; 0 = 'all'); ESC to cancel")),
                                              Some(Util.isNumeric), Some("0"))
    if (ans.isEmpty) return
    val levelsToExport: Int = ans.get.toInt
    val ans2: Option[Boolean] = ui.askYesNoQuestion("Include metadata (verbose detail: id's, types...)?")
    if (ans2.isEmpty) return
    val includeMetadata: Boolean = ans2.get
    //idea: make these choice strings into an enum? and/or the answers into an enum? what's the scala idiom? see same issue elsewhere
    val includePublicData: Option[Boolean] = ui.askYesNoQuestion("Include public data?  (Note: Whether an entity is public, non-public, or unset can be " +
                                                                 "marked on each entity's menu, and the preference as to whether to display that status on " +
                                                                 "each entity in a list can be set via the main menu.)", Some("y"))
    val includeNonPublicData: Option[Boolean] = ui.askYesNoQuestion("Include data marked non-public?", Some("n"))
    val includeUnspecifiedData: Option[Boolean] = ui.askYesNoQuestion("Include data not specified as public or non-public?", Some("n"))

    if (includePublicData.isDefined && includeNonPublicData.isDefined && includeUnspecifiedData.isDefined &&
        (includePublicData.get || includePublicData.get || includeUnspecifiedData.get)) {
      require(levelsToExport >= 0)
      val spacesPerIndentLevel = 2

      // To track what's been done so we don't repeat it:
      val exportedEntityIds = new mutable.TreeSet[Long]

      // The caches are to reduce the expensive repeated queries of attribute lists & entity objects (not all of which are known at the time we write to
      // exportedEntityIds).
      // Html exports were getting very slow before this caching logic was added.)
      val cachedEntities = new mutable.HashMap[Long, Entity]
      // (The key is the entityId, and the value contains the attributes (w/ id & attr) as returned from db.getSortedAttributes.)
      val cachedAttrs = new mutable.HashMap[Long, Array[(Long, Attribute)]]
      val cachedGroupInfo = new mutable.HashMap[Long, Array[Long]]

      val prefix: String = getExportFileNamePrefix(entity, exportTypeIn)
      if (exportTypeIn == ImportExport.TEXT_EXPORT_TYPE) {
        val (outputFile: File, outputWriter: PrintWriter) = createOutputFile(prefix, exportTypeIn, None)
        try {
          exportToSingleTxtFile(entity, levelsToExport == 0, levelsToExport, 0, outputWriter, includeMetadata, exportedEntityIds, cachedEntities, cachedAttrs,
                                spacesPerIndentLevel, includePublicData.get, includeNonPublicData.get, includeUnspecifiedData.get)
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
        val outputDirectory:Path = createOutputDir(prefix)
        // see note about this usage, in method importUriContent:
        val uriClassId: Long = db.getOrCreateClassAndTemplateEntityIds("URI", callerManagesTransactionsIn = true)._1
        val quoteClassId = db.getOrCreateClassAndTemplateEntityIds("quote", callerManagesTransactionsIn = true)._1

        exportHtml(entity, levelsToExport == 0, levelsToExport, outputDirectory, exportedEntityIds, cachedEntities, cachedAttrs,
                   cachedGroupInfo, mutable.TreeSet[Long](), uriClassId, quoteClassId,
                   includePublicData.get, includeNonPublicData.get, includeUnspecifiedData.get, headerContentIn, beginBodyContentIn, copyrightYearAndNameIn)
        ui.displayText("Exported to directory: " + outputDirectory.toFile.getCanonicalPath)
      } else {
        throw new OmException("unexpected value for exportTypeIn: " + exportTypeIn)
      }
    }
  }

  // This exists for the reasons commented in exportItsChildrenToHtmlFiles, and so that not all callers have to explicitly call both (ie, duplication of code).
  def exportHtml(entity: Entity, levelsToExportIsInfinite: Boolean, levelsToExport: Int,
                 outputDirectory: Path, exportedEntityIdsIn: mutable.TreeSet[Long], cachedEntitiesIn: mutable.HashMap[Long, Entity],
                 cachedAttrsIn: mutable.HashMap[Long, Array[(Long, Attribute)]], cachedGroupInfoIn: mutable.HashMap[Long, Array[Long]],
                 entitiesAlreadyProcessedInThisRefChain: mutable.TreeSet[Long],
                 uriClassId: Long, quoteClassId: Long,
                 includePublicData: Boolean, includeNonPublicData: Boolean, includeUnspecifiedData: Boolean,
                 headerContentIn: Option[String], beginBodyContentIn: Option[String], copyrightYearAndNameIn: Option[String]) {
    exportEntityToHtmlFile(entity, levelsToExportIsInfinite, levelsToExport, outputDirectory, exportedEntityIdsIn, cachedEntitiesIn, cachedAttrsIn,
                           uriClassId, quoteClassId, includePublicData, includeNonPublicData, includeUnspecifiedData,
                           headerContentIn, beginBodyContentIn, copyrightYearAndNameIn)
    exportItsChildrenToHtmlFiles(entity, levelsToExportIsInfinite, levelsToExport, outputDirectory, exportedEntityIdsIn, cachedEntitiesIn,
                                 cachedAttrsIn, cachedGroupInfoIn, entitiesAlreadyProcessedInThisRefChain, uriClassId, quoteClassId,
                                 includePublicData, includeNonPublicData, includeUnspecifiedData, headerContentIn, beginBodyContentIn, copyrightYearAndNameIn)
  }

  /** This creates a new file for each entity.
    *
    * If levelsToProcessIsInfiniteIn is true, then levelsRemainingToProcessIn is irrelevant.
    *
    */
  def exportEntityToHtmlFile(entityIn: Entity, levelsToExportIsInfiniteIn: Boolean, levelsRemainingToExportIn: Int,
                             outputDirectoryIn: Path, exportedEntityIdsIn: mutable.TreeSet[Long], cachedEntitiesIn: mutable.HashMap[Long, Entity],
                             cachedAttrsIn: mutable.HashMap[Long, Array[(Long, Attribute)]],
                             uriClassIdIn: Long, quoteClassIdIn: Long,
                             includePublicDataIn: Boolean, includeNonPublicDataIn: Boolean, includeUnspecifiedDataIn: Boolean,
                             headerContentIn: Option[String], beginBodyContentIn: Option[String], copyrightYearAndNameIn: Option[String]) {
    // useful while debugging:
    //out.flush()

    if (! isAllowedToExport(entityIn, includePublicDataIn, includeNonPublicDataIn, includeUnspecifiedDataIn,
                           levelsToExportIsInfiniteIn, levelsRemainingToExportIn)) {
      return
    }
    if (exportedEntityIdsIn.contains(entityIn.getId)) {
      // no need to recreate this entity's html file, so return
      return
    }

    val entitysFileNamePrefix: String = getExportFileNamePrefix(entityIn, ImportExport.HTML_EXPORT_TYPE)
    val printWriter = createOutputFile(entitysFileNamePrefix, ImportExport.HTML_EXPORT_TYPE, Some(outputDirectoryIn))._2
    //record, so we don't create duplicate files:
    exportedEntityIdsIn.add(entityIn.getId)
    try {
      printWriter.println("<html><head>")
      printWriter.println("  <title>" + entityIn.getName + "</title>")
      printWriter.println("  <meta name=\"description\" content=\"" + entityIn.getName + "\">")
      printWriter.println("  " + headerContentIn.getOrElse(""))

      printWriter.println("</head>")
      printWriter.println()
      printWriter.println("<body>")
      printWriter.println("  " + beginBodyContentIn.getOrElse(""))
      printWriter.println("  <h1>" + htmlEncode(entityIn.getName) + "</h1>")

      val attrTuples: Array[(Long, Attribute)] = getCachedAttributes(entityIn.getId, cachedAttrsIn)
      printWriter.println("  <ul>")
      for (attrTuple <- attrTuples) {
        val attribute:Attribute = attrTuple._2
        attribute match {
          case relation: RelationToEntity =>
            val relationType = new RelationType(db, relation.getAttrTypeId)
            val entity2 = getCachedEntity(relation.getRelatedId2, cachedEntitiesIn)
            if (isAllowedToExport(entity2, includePublicDataIn, includeNonPublicDataIn, includeUnspecifiedDataIn,
                                  levelsToExportIsInfiniteIn, levelsRemainingToExportIn - 1)) {
              if (entity2.getClassId.isDefined && entity2.getClassId.get == uriClassIdIn) {
                printListItemForUriEntity(uriClassIdIn, quoteClassIdIn, printWriter, entity2, cachedAttrsIn)
              } else {
                // i.e., don't create this link if it will be a broken link due to not creating the page later; also creating the link could disclose
                // info in the link itself (the entity name) that has been restricted (e.g., made nonpublic).
                printListItemForEntity(printWriter, relationType, entity2)
              }
            }
          case relation: RelationToGroup =>
            val relationType = new RelationType(db, relation.getAttrTypeId)
            val group = new Group(db, relation.getGroupId)
            // if a group name is different from its entity name, indicate the differing group name also, otherwise complete the line just above w/ NL
            printWriter.println("    <li>" + htmlEncode(relation.getDisplayString(0, None, Some(relationType), simplify = true)) + "</li>")
            printWriter.println("    <ul>")

            // this 'if' check is duplicate with the call just below to isAllowedToExport, but can quickly save the time looping through them all,
            // checking entities, if there's no need:
            if (levelsToExportIsInfiniteIn || levelsRemainingToExportIn - 1 > 0) {
              for (entityInGrp: Entity <- group.getGroupEntries(0).toArray(Array[Entity]())) {
                // i.e., don't create this link if it will be a broken link due to not creating the page later; also creating the link could disclose
                // info in the link itself (the entity name) that has been restricted (e.g., made nonpublic).
                if (isAllowedToExport(entityInGrp, includePublicDataIn, includeNonPublicDataIn, includeUnspecifiedDataIn,
                                      levelsToExportIsInfiniteIn, levelsRemainingToExportIn - 1)) {
                  if (entityInGrp.getClassId.isDefined && entityInGrp.getClassId.get == uriClassIdIn) {
                    printListItemForUriEntity(uriClassIdIn, quoteClassIdIn, printWriter, entityInGrp, cachedAttrsIn)
                  } else{
                    printListItemForEntity(printWriter, relationType, entityInGrp)
                  }
                }
              }
            }
            printWriter.println("    </ul>")
          case textAttr: TextAttribute =>
            val typeName: String = getCachedEntity(textAttr.getAttrTypeId, cachedEntitiesIn).getName
            if (typeName==Util.HEADER_CONTENT_TAG || typeName == Util.BODY_CONTENT_TAG || typeName==Util.FOOTER_CONTENT_TAG) {
              //skip it: this is used to create the pages and should not be considered a normal kind of displayable content in them:
            } else {
              printWriter.println("    <li><pre>" + htmlEncode(textAttr.getDisplayString(0, None, None, simplify = true)) + "</pre></li>")
            }
          case fileAttr: FileAttribute =>
            val originalPath = fileAttr.getOriginalFilePath
            val fileName = {
              if (originalPath.indexOf("/") >= 0) originalPath.substring(originalPath.lastIndexOf("/") + 1)
              else if (originalPath.indexOf("\\") >= 0) originalPath.substring(originalPath.lastIndexOf("\\") + 1)
              else originalPath
            }
            // (The use of the attribute id prevents problems if the same filename is used more than once on an entity:)
            val file: File = Files.createFile(new File(outputDirectoryIn.toFile, entitysFileNamePrefix + "-" + fileAttr.getId + "-" + fileName).toPath).toFile
            fileAttr.retrieveContent(file)
            if (originalPath.toLowerCase.endsWith("png") || originalPath.toLowerCase.endsWith("jpg") || originalPath.toLowerCase.endsWith("jpeg") ||
                originalPath.toLowerCase.endsWith("gif")) {
              printWriter.println("    <li><img src=\"" + file.getName + "\" alt=\"" + htmlEncode(fileAttr.getDisplayString(0, None, None, simplify = true)) +
                                  "\"></li>")
            } else {
              printWriter.println("    <li><a href=\"" + file.getName + "\">" + htmlEncode(fileAttr.getDisplayString(0, None, None, simplify = true)) +
                                  "</a></li>")
            }
          case attr: Attribute =>
            printWriter.println("    <li>" + htmlEncode(attr.getDisplayString(0, None, None, simplify = true)) + "</li>")
          case unexpected =>
            throw new OmException("How did we get here?: " + unexpected)
        }
      }
      printWriter.println("  </ul>")
      printWriter.println()
      if (copyrightYearAndNameIn.isDefined) {
        // (intentionally not doing "htmlEncode(copyrightYearAndNameIn.get)", so that some ~footer-like links can be included in it.
        printWriter.println("  <center><p><small>Copyright " + copyrightYearAndNameIn.get + "</small></p></center>")
      }
      printWriter.println("</body></html>")
      printWriter.close()
    } finally {
      // close each file as we go along.
      if (printWriter != null) {
        try printWriter.close()
        catch {
          case e: Exception =>
          // ignore
        }
      }
    }
  }

  def printListItemForUriEntity(uriClassIdIn: Long, quoteClassIdIn: Long, printWriter: PrintWriter, entity2: Entity,
                                cachedAttrsIn: mutable.HashMap[Long, Array[(Long, Attribute)]]): Unit = {
    // handle URIs differently than other entities: make it a link as indicated by the URI contents, not to a newly created entity page..
    // (could use a more efficient call in cpu time than getSortedAttributes, but it's efficient in programmer time:)
    def findUriAttribute(): Option[TextAttribute] = {
      val attributesOnEntity2: Array[(Long, Attribute)] = getCachedAttributes(entity2.getId, cachedAttrsIn)
      val uriTemplateId: Long = new EntityClass(db, uriClassIdIn).getTemplateEntityId
      for (attrTuple <- attributesOnEntity2) {
        val attr2: Attribute = attrTuple._2
        if (attr2.getAttrTypeId == uriTemplateId && attr2.isInstanceOf[TextAttribute]) {
          return Some(attr2.asInstanceOf[TextAttribute])
        }
      }
      None
    }
    def findQuoteText(): Option[String] = {
      val attributesOnEntity2: Array[(Long, Attribute)] = getCachedAttributes(entity2.getId, cachedAttrsIn)
      val quoteClassTemplateId: Long = new EntityClass(db, quoteClassIdIn).getTemplateEntityId
      for (attrTuple <- attributesOnEntity2) {
        val attr2: Attribute = attrTuple._2
        if (attr2.getAttrTypeId == quoteClassTemplateId && attr2.isInstanceOf[TextAttribute]) {
          return Some(attr2.asInstanceOf[TextAttribute].getText)
        }
      }
      None
    }
    val uriAttribute: Option[TextAttribute] = findUriAttribute()
    if (uriAttribute.isEmpty) {
      throw new OmException("Unable to find TextAttribute of type URI (classId=" + uriClassIdIn + ") for entity " + entity2.getId)
    }
    // this one can be None and it's no surprise:
    val quoteText: Option[String] = findQuoteText()
    printHtmlListItemWithLink(printWriter, "", uriAttribute.get.getText, entity2.getName, None, quoteText)
  }

  def printListItemForEntity(printWriterIn: PrintWriter, relationTypeIn: RelationType, entityIn: Entity): Unit = {
    val numSubEntries = getNumSubEntries(entityIn)
    if (numSubEntries > 0) {
      val relatedEntitysFileNamePrefix: String = getExportFileNamePrefix(entityIn, ImportExport.HTML_EXPORT_TYPE)
      printHtmlListItemWithLink(printWriterIn,
                                if (relationTypeIn.getName == PostgreSQLDatabase.theHASrelationTypeName) "" else relationTypeIn.getName + ": ",
                                relatedEntitysFileNamePrefix + ".html",
                                entityIn.getName)
                                //removing next line until it matches better with what user can actually see: currently includes non-public stuff, so the #
                                //might confuse a reader, or at least doesn't set fulfillable expectations on how much content there is.
//                                Some("(" + numSubEntries + ")"))
    } else {
      val line = (if (relationTypeIn.getName == PostgreSQLDatabase.theHASrelationTypeName) "" else relationTypeIn.getName + ": ") +
                 entityIn.getName
      printWriterIn.println("<li>" + htmlEncode(line) + "</li>")
    }
  }

  /** This method exists (as opposed to including the logic inside exportToHtmlFile) because there was a bug.  Here I try explaining:
    *   - the parm levelsRemainingToExportIn limits how far in the hierarchy (distance from the root entity of the export) the export will include (or descend).
    *   - at some "deep" point in the hierarchy, an entity X might be exported, but not its children, because X was at the depth limit.
    *   - X might also be found elsewhere, "shallower" in the hierarchy, but having been exported before (at the deep point), it is not now exported again.
    *   - Therefore X's children should have been exported from the "shallow" point, because they are now less than levelsRemainingToExportIn levels deep, but
    *     were not exported because X was skipped (having been already been done).
    *   - Therefore separating the logic for the children allows them to be exported anyway, which fixes the bug.
    * Still, within this method it is also necessary to void infinitely looping around entities who contain references to (eventually) themselves, which
    * is the purpose of the variable "entitiesAlreadyProcessedInThisRefChain".
    *
    * If parameter levelsToProcessIsInfiniteIn is true, then levelsRemainingToProcessIn is irrelevant.
    */
  def exportItsChildrenToHtmlFiles(entityIn: Entity, levelsToExportIsInfiniteIn: Boolean, levelsRemainingToExportIn: Int,
                                   outputDirectoryIn: Path, exportedEntityIdsIn: mutable.TreeSet[Long], cachedEntitiesIn: mutable.HashMap[Long, Entity],
                                   cachedAttrsIn: mutable.HashMap[Long, Array[(Long, Attribute)]], cachedGroupInfoIn: mutable.HashMap[Long, Array[Long]],
                                   entitiesAlreadyProcessedInThisRefChainIn: mutable.TreeSet[Long], uriClassIdIn: Long, quoteClassId: Long,
                                   includePublicDataIn: Boolean, includeNonPublicDataIn: Boolean, includeUnspecifiedDataIn: Boolean,
                                   headerContentIn: Option[String], beginBodyContentIn: Option[String], copyrightYearAndNameIn: Option[String]) {
    if (! isAllowedToExport(entityIn, includePublicDataIn, includeNonPublicDataIn, includeUnspecifiedDataIn,
                            levelsToExportIsInfiniteIn, levelsRemainingToExportIn)) {
      return
    }
    if (entitiesAlreadyProcessedInThisRefChainIn.contains(entityIn.getId)) {
      return
    }

    entitiesAlreadyProcessedInThisRefChainIn.add(entityIn.getId)
    val attrTuples: Array[(Long, Attribute)] = getCachedAttributes(entityIn.getId, cachedAttrsIn)
    for (attributeTuple <- attrTuples) {
      val attribute: Attribute = attributeTuple._2
      attribute match {
        case relation: RelationToEntity =>
          val entity2: Entity = getCachedEntity(relation.getRelatedId2, cachedEntitiesIn)
          if (entity2.getClassId.isEmpty || entity2.getClassId.get != uriClassIdIn) {
            // that means it's not a URI but an actual traversable thing to follow when exporting children:
            exportHtml(entity2, levelsToExportIsInfiniteIn, levelsRemainingToExportIn - 1,
                       outputDirectoryIn, exportedEntityIdsIn, cachedEntitiesIn,
                       cachedAttrsIn, cachedGroupInfoIn, entitiesAlreadyProcessedInThisRefChainIn, uriClassIdIn, quoteClassId,
                       includePublicDataIn, includeNonPublicDataIn, includeUnspecifiedDataIn,
                       headerContentIn, beginBodyContentIn, copyrightYearAndNameIn)
          }
        case relation: RelationToGroup =>
          val entityIds: Array[Long] = getCachedGroupData(relation.getGroupId, cachedGroupInfoIn)
          for (entityIdInGrp <- entityIds) {
            val entityInGrp: Entity = getCachedEntity(entityIdInGrp, cachedEntitiesIn)
            exportHtml(entityInGrp, levelsToExportIsInfiniteIn, levelsRemainingToExportIn - 1,
                       outputDirectoryIn, exportedEntityIdsIn, cachedEntitiesIn,
                       cachedAttrsIn, cachedGroupInfoIn, entitiesAlreadyProcessedInThisRefChainIn, uriClassIdIn, quoteClassId,
                       includePublicDataIn, includeNonPublicDataIn, includeUnspecifiedDataIn,
                       headerContentIn, beginBodyContentIn, copyrightYearAndNameIn)
          }
        case _ =>
          // nothing intended here
      }
    }
    // remove the entityId we've just processed, in order to allow traversing through it again later on a different ref chain if needed.  See
    // comments on this method, above, for more explanation.
    entitiesAlreadyProcessedInThisRefChainIn.remove(entityIn.getId)
  }

  def getCachedGroupData(groupIdIn: Long, cachedGroupInfoIn: mutable.HashMap[Long, Array[Long]]): Array[Long] = {
    val cachedIds: Option[Array[Long]] = cachedGroupInfoIn.get(groupIdIn)
    if (cachedIds.isDefined) {
      cachedIds.get
    } else {
      val group = new Group(db, groupIdIn)
      val data: List[Array[Option[Any]]] = db.getGroupEntriesData(group.getId, None, includeArchivedEntitiesIn = false)
      val entityIds = new Array[Long](data.size)
      var count = 0
      for (entry <- data) {
        val entityIdInGroup: Long = entry(0).get.asInstanceOf[Long]
        entityIds(count) = entityIdInGroup
        count += 1
      }
      cachedGroupInfoIn.put(groupIdIn, entityIds)
      entityIds
    }
  }

  def getCachedAttributes(entityIdIn: Long, cachedAttrsIn: mutable.HashMap[Long, Array[(Long, Attribute)]]): Array[(Long, Attribute)] = {
    val cachedInfo: Option[Array[(Long, Attribute)]] = cachedAttrsIn.get(entityIdIn)
    if (cachedInfo.isDefined) {
      cachedInfo.get
    } else {
      val attrTuples = db.getSortedAttributes(entityIdIn, 0, 0)._1
      // record, so we don't create files more than once, calculate attributes more than once, etc.
      cachedAttrsIn.put(entityIdIn, attrTuples)
      attrTuples
    }
  }

  def getCachedEntity(entityIdIn: Long, cachedEntitiesIn: mutable.HashMap[Long, Entity]): Entity = {
    val cachedInfo: Option[Entity] = cachedEntitiesIn.get(entityIdIn)
    if (cachedInfo.isDefined) {
      cachedInfo.get
    } else {
      val entity = new Entity(db, entityIdIn)
      cachedEntitiesIn.put(entityIdIn, entity)
      entity
    }
  }

  /** Very basic for now. Noted in task list to do more, under i18n and under "do a better job of encoding"
    */
  def htmlEncode(in: String): String = {
    var out = in.replace("&", "&amp;")
    out = out.replace(">", "&gt;")
    out = out.replace("<", "&lt;")
    out = out.replace("\"", "&quot;")
    out
  }

  //@tailrec  THIS IS NOT TO BE TAIL RECURSIVE UNTIL IT'S KNOWN HOW TO MAKE SOME CALLS to it BE recursive, AND SOME *NOT* TAIL RECURSIVE (because some of them
  //*do* need to return & finish their work, such as when iterating through the entities & subgroups)! (but test it: is it really a problem?)
  // (Idea: See note at the top of Controller.chooseOrCreateObject re inAttrType about similarly making exportTypeIn an enum.)
  /**
    * If levelsToProcessIsInfiniteIn is true, then levelsRemainingToProcessIn is irrelevant.
    */
  def exportToSingleTxtFile(entityIn: Entity, levelsToExportIsInfiniteIn: Boolean, levelsRemainingToExportIn: Int, currentIndentationLevelsIn: Int,
                            printWriterIn: PrintWriter,
                            includeMetadataIn: Boolean, exportedEntityIdsIn: mutable.TreeSet[Long], cachedEntitiesIn: mutable.HashMap[Long, Entity],
                            cachedAttrsIn: mutable.HashMap[Long, Array[(Long, Attribute)]], spacesPerIndentLevelIn: Int,
                            //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) w/in this method!
                            includePublicDataIn: Boolean, includeNonPublicDataIn: Boolean, includeUnspecifiedDataIn: Boolean) {
    // useful while debugging:
    //out.flush()

    if (exportedEntityIdsIn.contains(entityIn.getId)) {
      printSpaces(currentIndentationLevelsIn * spacesPerIndentLevelIn, printWriterIn)
      if (includeMetadataIn) printWriterIn.print("(duplicate: EN --> " + entityIn.getId + ": ")
      printWriterIn.print(entityIn.getName)
      if (includeMetadataIn) printWriterIn.print(")")
      printWriterIn.println()
    } else {
      val allowedToExport: Boolean = isAllowedToExport(entityIn, includePublicDataIn, includeNonPublicDataIn, includeUnspecifiedDataIn,
                                                       levelsToExportIsInfiniteIn, levelsRemainingToExportIn)
      if (allowedToExport) {
        //record, so we don't create duplicate files:
        exportedEntityIdsIn.add(entityIn.getId)

        val entityName: String = entityIn.getName
        printSpaces(currentIndentationLevelsIn * spacesPerIndentLevelIn, printWriterIn)

        if (includeMetadataIn) printWriterIn.println("EN " + entityIn.getId + ": " + entityIn.getDisplayString())
        else printWriterIn.println(entityName)

        val attrTuples: Array[(Long, Attribute)] = getCachedAttributes(entityIn.getId, cachedAttrsIn)
        for (attributeTuple <- attrTuples) {
          val attribute:Attribute = attributeTuple._2
          attribute match {
            case relation: RelationToEntity =>
              val relationType = new RelationType(db, relation.getAttrTypeId)
              val entity2 = new Entity(db, relation.getRelatedId2)
              if (includeMetadataIn) {
                printSpaces((currentIndentationLevelsIn + 1) * spacesPerIndentLevelIn, printWriterIn)
                printWriterIn.println(attribute.getDisplayString(0, Some(entity2), Some(relationType)))
              }
              exportToSingleTxtFile(entity2, levelsToExportIsInfiniteIn, levelsRemainingToExportIn - 1, currentIndentationLevelsIn + 1, printWriterIn,
                                    includeMetadataIn, exportedEntityIdsIn, cachedEntitiesIn, cachedAttrsIn, spacesPerIndentLevelIn,
                                    includePublicDataIn, includeNonPublicDataIn, includeUnspecifiedDataIn)
            case relation: RelationToGroup =>
              val relationType = new RelationType(db, relation.getAttrTypeId)
              val group = new Group(db, relation.getGroupId)
              val grpName = group.getName
              // if a group name is different from its entity name, indicate the differing group name also, otherwise complete the line just above w/ NL
              if (entityName != grpName) {
                printSpaces((currentIndentationLevelsIn + 1) * spacesPerIndentLevelIn, printWriterIn)
                printWriterIn.println("(" + relationType.getName + " group named: " + grpName + ")")
              }
              if (includeMetadataIn) {
                printSpaces(currentIndentationLevelsIn * spacesPerIndentLevelIn, printWriterIn)
                // plus one more level of spaces to make it look better but still ~equivalently/exchangeably importable:
                printSpaces(spacesPerIndentLevelIn, printWriterIn)
                printWriterIn.println("(group details: " + attribute.getDisplayString(0, None, Some(relationType)) + ")")
              }
              for (entityInGrp: Entity <- group.getGroupEntries(0).toArray(Array[Entity]())) {
                exportToSingleTxtFile(entityInGrp, levelsToExportIsInfiniteIn, levelsRemainingToExportIn - 1, currentIndentationLevelsIn + 1, printWriterIn,
                                      includeMetadataIn, exportedEntityIdsIn, cachedEntitiesIn, cachedAttrsIn, spacesPerIndentLevelIn,
                                      includePublicDataIn, includeNonPublicDataIn, includeUnspecifiedDataIn)
              }
            case _ =>
              printSpaces((currentIndentationLevelsIn + 1) * spacesPerIndentLevelIn, printWriterIn)
              if (includeMetadataIn) {
                printWriterIn.println((attribute match {
                  case ba: BooleanAttribute => "BA "
                  case da: DateAttribute => "DA "
                  case fa: FileAttribute => "FA "
                  case qa: QuantityAttribute => "QA "
                  case ta: TextAttribute => "TA "
                }) + /*attribute.getId +*/ ": " + attribute.getDisplayString(0, None, None))
              } else {
                printWriterIn.println(attribute.getDisplayString(0, None, None, simplify = true))
              }
          }
        }
      }
    }
  }

  def isAllowedToExport(entityIn: Entity, includePublicDataIn: Boolean, includeNonPublicDataIn: Boolean,
                        includeUnspecifiedDataIn: Boolean, levelsToExportIsInfiniteIn: Boolean, levelsRemainingToExportIn: Int): Boolean = {
    val entityPublicStatus: Option[Boolean] = entityIn.getPublic
    val publicEnoughToExport = (entityPublicStatus.isDefined && entityPublicStatus.get && includePublicDataIn) ||
                          (entityPublicStatus.isDefined && !entityPublicStatus.get && includeNonPublicDataIn) ||
                          (entityPublicStatus.isEmpty && includeUnspecifiedDataIn)

    publicEnoughToExport && (levelsToExportIsInfiniteIn || levelsRemainingToExportIn > 0)
  }

  def printHtmlListItemWithLink(printWriterIn: PrintWriter, preLabel: String, uri: String, linkDisplayText: String, suffix: Option[String] = None,
                                textOnNextLineButSameHtmlListItem: Option[String] = None): Unit = {
    printWriterIn.print("<li>")
    printWriterIn.print(htmlEncode(preLabel) + "<a href=\"" + uri + "\">" + htmlEncode(linkDisplayText) + "</a>" + " " + htmlEncode(suffix.getOrElse("")))
    if (textOnNextLineButSameHtmlListItem.isDefined) printWriterIn.print("<br><pre>\"" + htmlEncode(textOnNextLineButSameHtmlListItem.get) + "\"</pre>")
    printWriterIn.println("</li>")
  }

  def getNumSubEntries(entityIn: Entity): Long = {
    val numSubEntries = {
      val numAttrs = db.getAttrCount(entityIn.getId)
      if (numAttrs == 1) {
        val (_, _, groupId, moreThanOneAvailable) = db.findRelationToAndGroup_OnEntity(entityIn.getId)
        if (groupId.isDefined && !moreThanOneAvailable) {
          db.getGroupSize(groupId.get, 4)
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
        Files.createTempFile(prefix, extension).toFile
      }
      else throw new OmException("unexpected exportTypeIn: " + exportTypeIn)

    val output: PrintWriter = new PrintWriter(new BufferedWriter(new FileWriter(outputFile)))
    (outputFile, output)
  }

  // these methods are in this class so it can be found by both PostgreSQLDatabaseTest and ImportExportTest (not sure why it couldn't be found
  // by PostgreSQLDatabaseTest when it was in ImportExportTest).
  def tryImporting_FOR_TESTS(filenameIn: String, entityIn: Entity): File = {
    //PROBLEM: these 2 lines make it so it's hard to test in the IDE without first building a .jar since it finds the file in the jar. How fix?
    val stream = this.getClass.getClassLoader.getResourceAsStream(filenameIn)
    val reader: java.io.Reader = new java.io.InputStreamReader(stream)

    // manual testing alternative to the above 2 lines, such as for use w/ interactive scala (REPL):
    //val path = "PUT-Full-path-to-some-text-file-here"
    //val fileToImport = new File(path)
    //val reader = new FileReader(fileToImport)

    doTheImport(reader, "name", 0L, entityIn, creatingNewStartingGroupFromTheFilenameIn = false, addingToExistingGroup = false,
                putEntriesAtEnd = true, mixedClassesAllowedDefaultIn = true, testing = true, makeThemPublicIn = Some(false))

    // write it out for later comparison:
    val stream2 = this.getClass.getClassLoader.getResourceAsStream(filenameIn)
    val tmpCopy: Path = Files.createTempFile(null, null)
    Files.copy(stream2, tmpCopy, StandardCopyOption.REPLACE_EXISTING)
    tmpCopy.toFile
  }
  // (see cmt on tryImporting method)
  def tryExportingTxt_FOR_TESTS(ids: Option[List[Long]], dbIn: PostgreSQLDatabase): (String, File) = {
    assert(ids.get.nonEmpty)
    val entityId: Long = ids.get.head
    val startingEntity: Entity = new Entity(dbIn, entityId)

    // see comments in ImportExport.export() method for explanation of these 3
    val exportedEntityIds = new mutable.TreeSet[Long]
    val cachedEntities = new mutable.HashMap[Long, Entity]
    val cachedAttrs = new mutable.HashMap[Long, Array[(Long, Attribute)]]

    val prefix: String = getExportFileNamePrefix(startingEntity, ImportExport.TEXT_EXPORT_TYPE)
    val (outputFile: File, outputWriter: PrintWriter) = createOutputFile(prefix, ImportExport.TEXT_EXPORT_TYPE, None)
    exportToSingleTxtFile(startingEntity, levelsToExportIsInfiniteIn = true, 0, 0, outputWriter, includeMetadataIn = false, exportedEntityIds, cachedEntities,
                          cachedAttrs, 2, includePublicDataIn = true, includeNonPublicDataIn = true, includeUnspecifiedDataIn = true)
    assert(outputFile.exists)
    outputWriter.close()
    val firstNewFileContents: String = new Predef.String(Files.readAllBytes(outputFile.toPath))
    (firstNewFileContents, outputFile)
  }

}
