/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2014-2018 inclusive, Luke A. Call; all rights reserved.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule, guidelines around binary
    distribution, and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
package org.onemodel.core.controllers

import java.io._
import java.nio.file.{Files, Path, StandardCopyOption}

import org.onemodel.core._
import org.onemodel.core.model._
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
class ImportExport(val ui: TextUI, controller: Controller) {
  val uriLineExample: String = "'nameForTheLink <uri>http://somelink.org/index.html</uri>'"

  /**
   * 1st parameter must be either an Entity or a RelationToGroup (what is the right way to do that, in the signature?).
   */
  def importCollapsibleOutlineAsGroups(firstContainingEntryIn: AnyRef) {
    //noinspection ComparingUnrelatedTypes
    require(firstContainingEntryIn.isInstanceOf[Entity] || firstContainingEntryIn.isInstanceOf[Group])
    val db: Database = {
      //noinspection ComparingUnrelatedTypes,TypeCheckCanBeMatch
      if (firstContainingEntryIn.isInstanceOf[Entity]) {
        firstContainingEntryIn.asInstanceOf[Entity].mDB
      } else {
        firstContainingEntryIn.asInstanceOf[Group].mDB
      }
    }
    val ans1: Option[String] = ui.askForString(Some(Array("Enter file path (must exist, be readable, AND a text file with lines spaced in the form of a" +
                                                          " collapsible outline where each level change is marked by 1 tab or 2 spaces; textAttribute content" +
                                                          " can be indicated by surrounding a body of text thus, without quotes: '<ta>text</ta>';" +
                                                          " a URI similarly with a line " + uriLineExample + ")," +
                                                          " then press Enter; ESC to cancel")),
                                               Some(Util.inputFileValid))
    if (ans1.isDefined) {
      val path = ans1.get
      val makeThemPublic: Option[Boolean] = ui.askYesNoQuestion("Do you want the entities imported to be marked as public?  Set it to the value the " +
                                                      "majority of imported data should have; you can then edit the individual settings afterward as " +
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
                                  "then ESC back here to commit the changes....  (If you wait beyond some amount of time(?) or go beyond just viewing, " +
                                  "it seems that postgres will commit " +
                                  "the change whether you want it or not, even if the message at that time says 'rolled back...')"
                ui.displayText(msg)
                firstContainingEntryIn match {
                  case entity: Entity => new EntityMenu(ui, controller).entityMenu(entity)
                  case group: Group => new QuickGroupMenu(ui, controller).quickGroupMenu(firstContainingEntryIn.asInstanceOf[Group], 0,
                                                                                         containingEntityIn = None)
                  case _ => throw new OmException("??")
                }
                ui.askYesNoQuestion("Do you want to commit the changes as they were made?")
              }
              if (keepAnswer.isEmpty || !keepAnswer.get) {
                db.rollbackTrans()
                //idea: look into how long that time is (see above same cmt)
                ui.displayText("Rolled back the import: no changes made (unless you browsed farther, into code that had another commit, or " +
                               "waited too long and postgres committed it anyway...?).")
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
    val entityId: Long = group.mDB.createEntity(line.trim, group.getClassId, isPublicIn)
    group.addEntity(entityId, Some(newSortingIndex), callerManagesTransactionsIn = true)
    new Entity(group.mDB, entityId)
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
        importTextAttributeContent(lineUntrimmed, r, lastEntityAdded.get, beginTaMarker, endTaMarker)
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
                  entity.createEntityAndAddHASLocalRelationToIt(line, observationDateIn, makeThemPublicIn, callerManagesTransactionsIn = true)._1
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
                  entity.createEntityAndAddHASLocalRelationToIt(line, observationDateIn, makeThemPublicIn, callerManagesTransactionsIn = true)._1
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
            // Ex., if "3" is the last entity created in the series of lines '1', '2', and '3' (which has indented under it '4'), and so '4' is the
            // current line, create a subgroup on '3' called '3' (the subgroup that entity sort of represents), and it becomes the new container. If the
            // user preferred this to be a relation to entity instead of to group to contain the sub-things,
            // oh well they can add it to the entity as such,
            // for now at least.
            val newGroup: Group = lastEntityAdded.get.createGroupAndAddHASRelationToIt(lastEntityAdded.get.getName, mixedClassesAllowed,
                                                                                       observationDateIn, callerManagesTransactionsIn = true)._1
            // since a new grp, start at beginning of sorting indexes
            val newSortingIndex = Database.minIdValue
            val newSubEntity: Entity = createAndAddEntityToGroup(line, newGroup, newSortingIndex, makeThemPublicIn)
            importRestOfLines(r, Some(newSubEntity), newIndentationLevel, newGroup :: containerList, newSortingIndex :: lastSortingIndexes,
                              observationDateIn, mixedClassesAllowedDefaultIn, makeThemPublicIn)
          } else throw new OmException("Shouldn't get here!?: " + lastIndentationLevel + ", " + newIndentationLevel)
        }
      }
    }
  }

  def importTextAttributeContent(lineUntrimmedIn: String, r: LineNumberReader, entityIn: Entity, beginningTagMarker: String, endTaMarker: String) {
    val lineContentBeforeMarker = lineUntrimmedIn.substring(0, lineUntrimmedIn.toLowerCase.indexOf(beginningTagMarker)).trim
    val restOfLine = lineUntrimmedIn.substring(lineUntrimmedIn.toLowerCase.indexOf(beginningTagMarker) + beginningTagMarker.length).trim
    if (restOfLine.toLowerCase.contains(endTaMarker)) throw new OmException("\"Unsupported format at line " + r.getLineNumber + ": beginning and ending " +
                                                                            "markers must NOT be on the same line.")
    val attrTypeId: Long = {
      val idsByName: java.util.ArrayList[Long] = entityIn.mDB.findAllEntityIdsByName(lineContentBeforeMarker.trim, caseSensitive = true)
      if (idsByName.size == 1)
        idsByName.get(0)
      else {
        // idea: alternatively, could use a generic one in this case?  Optionally?
        val prompt = "A name for the *type* of this text attribute was not provided; it would be the entire line content preceding the \"" +
                     beginningTagMarker + "\" " +
                     "(it has to match an existing entity, case-sensitively)"
        //IDEA: this used to call controller.chooseOrCreateObject_OrSaysCancelled instead. Removing it removes a prompt if the user pressed ESC during it,
        //and this lacks a convenient way to test it, and I don't know that anyone uses it right now. So maybe add a test sometime:
        val selection: Option[(IdWrapper, Boolean, String)] = controller.chooseOrCreateObject(entityIn.mDB,
                                                                                              Some(List(prompt + ", so please choose one or ESC to abort" +
                                                                                                        " this import operation:")),
                                                                                              None, None, Util.TEXT_TYPE)
        if (selection.isEmpty) {
          throw new OmException(prompt + " or selected.")
        } else {
          selection.get._1.getId
        }
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
    entityIn.createTextAttribute(attrTypeId, text, callerManagesTransactionsIn = true)
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
    lastEntityAddedIn.addUriEntityWithUriAttribute(name, uri, observationDateIn, makeThemPublicIn, callerManagesTransactionsIn = true)
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
            val newEntity: Entity = createAndAddEntityToGroup(name, containingGroup, containingGroup.findUnusedSortingIndex(), makeThemPublicIn)
            val newGroup: Group = newEntity.createGroupAndAddHASRelationToIt(name, containingGroup.getMixedClassesAllowed, System.currentTimeMillis,
                                                                             callerManagesTransactionsIn = true)._1
            newGroup
          } else {
            assert(addingToExistingGroup)
            // importing the new entries to an existing group
            new Group(containingGroup.mDB, containingGroup.getId)
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
        if (nextSortingIndex == Database.minIdValue) {
          // we wrapped from the biggest to lowest Long value
          containingGrp.renumberSortingIndexes(callerManagesTransactionsIn = true)
          val nextTriedNewSortingIndex: Long = containingGrp.getHighestSortingIndex + 1
          if (nextSortingIndex == Database.minIdValue) {
            throw new OmException("Huh? How did we get two wraparounds in a row?")
          }
          nextTriedNewSortingIndex
        } else nextSortingIndex
      } else Database.minIdValue
    }

    importRestOfLines(r, None, 0, containingEntry :: Nil, startingSortingIndex :: Nil, dataSourceLastModifiedDate, mixedClassesAllowedDefaultIn,
                      makeThemPublicIn)
  }

  // idea: see comment in EntityMenu about scoping.
  def export(entityIn: Entity, exportTypeIn: String, headerContentIn: Option[String], beginBodyContentIn: Option[String], copyrightYearAndNameIn: Option[String]) {
    def askForExportChoices: (Boolean, String, Int, Boolean, Boolean, Boolean, Boolean, Boolean, Boolean, Int) = {
      val levelsText = "number of levels to export"

      val ans: Option[String] = ui.askForString(Some(Array("Enter " + levelsText + " (including this one; 0 = 'all'); ESC to cancel")),
                                                Some(Util.isNumeric), Some("0"))
      if (ans.isEmpty) return (true, "", 0, false, false, false, false, false, false, 1)
      val levelsToExport: Int = ans.get.toInt

      val ans2: Option[Boolean] = ui.askYesNoQuestion("Include metadata (verbose detail: id's, types...)?")
      if (ans2.isEmpty) return (true, "", 0, false, false, false, false, false, false, 1)
      val includeMetadata: Boolean = ans2.get

      //idea: make these choice strings into an enum? and/or the answers into an enum? what's the scala idiom? see same issue elsewhere
      val ans3: Option[Boolean] = ui.askYesNoQuestion("Include public data?  (Note: Whether an entity is public, non-public, or unset can be " +
                                                                   "marked on each entity's menu, and the preference as to whether to display that status on " +
                                                                   "each entity in a list can be set via the main menu.)", Some("y"), allowBlankAnswer = true)
      if (ans3.isEmpty) return (true, "", 0, false, false, false, false, false, false, 1)
      val includePublicData: Boolean = ans3.get

      val ans4: Option[Boolean] = ui.askYesNoQuestion("Include data marked non-public?", Some("n"), allowBlankAnswer = true)
      if (ans4.isEmpty) return (true, "", 0, false, false, false, false, false, false, 1)
      val includeNonPublicData: Boolean = ans4.get

      val ans5: Option[Boolean] = ui.askYesNoQuestion("Include data not specified as public or non-public?", Some("y"),
                                                      allowBlankAnswer = true)
      if (ans5.isEmpty) return (true, "", 0, false, false, false, false, false, false, 1)
      val includeUnspecifiedData: Boolean = ans5.get

      var numberTheLines: Boolean = false
      var wrapTheLines: Boolean = false
      var wrapAtColumn: Int = 1
      if (exportTypeIn == ImportExport.TEXT_EXPORT_TYPE) {
        val ans6: Option[Boolean] = ui.askYesNoQuestion("Number the entries in outline form (ex, 3.1.5)?  (Prevents directly re-importing.)", Some("n"), allowBlankAnswer = true)
        if (ans6.isEmpty) return (true, "", 0, false, false, false, false, false, false, 1)
        numberTheLines = ans6.get

        // (See for more explanation on this prompt, the "adjustedCurrentIndentationLevels" variable used in a different method below.
        val ans7: Option[Boolean] = ui.askYesNoQuestion("Wrap long lines and add whitespace for readability?  (Prevents directly re-importing; also removes one level of indentation, needless in that case.)", Some("y"), allowBlankAnswer = true)
        if (ans7.isEmpty) return (true, "", 0, false, false, false, false, false, false, 1)
        wrapTheLines = ans7.get

        wrapAtColumn = {
          def checkColumn(s: String): Boolean = {
            Util.isNumeric(s) && s.toFloat > 0
          }
          val ans8: Option[String] = ui.askForString(Some(Array("Wrap at what column (greater than 0)?")), Some(checkColumn), Some("80"),
                                                       escKeySkipsCriteriaCheck = true)
          if (ans8.isEmpty) return (true, "", 0, false, false, false, false, false, false, 1)
          ans8.get.toInt
        }
      }
      (false, levelsText, levelsToExport, includeMetadata, includePublicData, includeNonPublicData, includeUnspecifiedData, numberTheLines, wrapTheLines,
      wrapAtColumn)
    }


    val (userWantsOut: Boolean, levelsText: String, levelsToExport: Int, includeMetadata: Boolean, includePublicData: Boolean, includeNonPublicData: Boolean,
         includeUnspecifiedData: Boolean, numberTheLines: Boolean, wrapTheLines: Boolean, wrapAtColumn: Int) = askForExportChoices
    if (! userWantsOut) {
      ui.displayText("Processing..." + Util.NEWLN +
                     "(Note: if this takes too long, you can Ctrl+C and start over with a smaller or nonzero " + levelsText + ".)", waitForKeystrokeIn = false)
      require(levelsToExport >= 0)
      val spacesPerIndentLevel = 2

      // To track what's been done so we don't repeat it:
      val exportedEntityIds = new mutable.TreeSet[String]

      // The caches are to reduce the expensive repeated queries of attribute lists & entity objects (not all of which are known at the time we write to
      // exportedEntityIds).
      // Html exports were getting very slow before this caching logic was added.)
      val cachedEntities = new mutable.HashMap[String, Entity]
      // (The key is the entityId, and the value contains the attributes (w/ id & attr) as returned from db.getSortedAttributes.)
      val cachedAttrs = new mutable.HashMap[Long, Array[(Long, Attribute)]]
      val cachedGroupInfo = new mutable.HashMap[Long, Array[Long]]

      val prefix: String = getExportFileNamePrefix(entityIn, exportTypeIn)
      if (exportTypeIn == ImportExport.TEXT_EXPORT_TYPE) {
        val (outputFile: File, outputWriter: PrintWriter) = createOutputFile(prefix, exportTypeIn, None)
        try {
          if (wrapTheLines || numberTheLines) {
            // The next line is debatable, but a point I want to make for now, and a personal convenience.  If you don't like it send a
            // comment on the list, or a patch with it removed, for discussion.
            // Or maybe we just remove the "wrapTheLines" part of the condition so it prints only with the numbered outline format.
            // Done here because the method exportToSingleTextFile is called recursively, and this needs to simply be first.
            // Maybe it (or at least the part after #1) should be replaced with a link to some page ~ "How to do structured skimming to get more out of
            // reading or spend less time".
            outputWriter.println("(This is an outline, meant to be skimmable.  That means: 1) for an outline like this, read only the most out-dented parts," +
                                 " and then the indented parts only if interest in the parent entry justifies it;" +
                                 Util.NEWLN + "and (the rest of this paragraph is not" +
                                 " for this content, but has general tips on structured skimming that have helped me get more out of reading, in less" +
                                 " time), " +
                                 Util.NEWLN + "2) for essays or papers, read the first and" +
                                 " last paragraphs, then if interest remains, just the first sentences of paragraphs, and more only based on the value of" +
                                 " what was read already; and , " +
                                 Util.NEWLN + "3) for news, just the beginning to get the most important info, and read more rest only if" +
                                 " you really want the increasing level of detail that comes in later parts of news articles.), " +
                                 Util.NEWLN + "For more, see:  https://en.wikipedia.org/wiki/Skimming_(reading)#Skimming_and_scanning" + Util.NEWLN)
          }

          exportToSingleTextFile(entityIn, levelsToExport == 0, levelsToExport, 0, outputWriter, includeMetadata, exportedEntityIds, cachedEntities, cachedAttrs,
                                spacesPerIndentLevel, includePublicData, includeNonPublicData, includeUnspecifiedData, wrapTheLines,
                                wrapAtColumn, numberTheLines)
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
        val uriClassId: Long = entityIn.mDB.getOrCreateClassAndTemplateEntity("URI", callerManagesTransactionsIn = true)._1
        val quoteClassId = entityIn.mDB.getOrCreateClassAndTemplateEntity("quote", callerManagesTransactionsIn = true)._1

        exportHtml(entityIn, levelsToExport == 0, levelsToExport, outputDirectory, exportedEntityIds, cachedEntities, cachedAttrs,
                   cachedGroupInfo, mutable.TreeSet[Long](), uriClassId, quoteClassId,
                   includePublicData, includeNonPublicData, includeUnspecifiedData, headerContentIn, beginBodyContentIn, copyrightYearAndNameIn)
        ui.displayText("Exported to directory: " + outputDirectory.toFile.getCanonicalPath)
      } else {
        throw new OmException("unexpected value for exportTypeIn: " + exportTypeIn)
      }
    }
  }

  // This exists for the reasons commented in exportItsChildrenToHtmlFiles, and so that not all callers have to explicitly call both (ie, duplication of code).
  def exportHtml(entity: Entity, levelsToExportIsInfinite: Boolean, levelsToExport: Int,
                 outputDirectory: Path, exportedEntityIdsIn: mutable.TreeSet[String], cachedEntitiesIn: mutable.HashMap[String, Entity],
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
                             outputDirectoryIn: Path, exportedEntityIdsIn: mutable.TreeSet[String], cachedEntitiesIn: mutable.HashMap[String, Entity],
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
    if (exportedEntityIdsIn.contains(entityIn.uniqueIdentifier)) {
      // no need to recreate this entity's html file, so return
      return
    }

    val entitysFileNamePrefix: String = getExportFileNamePrefix(entityIn, ImportExport.HTML_EXPORT_TYPE)
    val printWriter = createOutputFile(entitysFileNamePrefix, ImportExport.HTML_EXPORT_TYPE, Some(outputDirectoryIn))._2
    //record, so we don't create duplicate files:
    exportedEntityIdsIn.add(entityIn.uniqueIdentifier)
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

      val attrTuples: Array[(Long, Attribute)] = getCachedAttributes(entityIn, cachedAttrsIn)
      printWriter.println("  <ul>")
      for (attrTuple <- attrTuples) {
        val attribute:Attribute = attrTuple._2
        attribute match {
          case relation: RelationToLocalEntity =>
            val relationType = new RelationType(relation.mDB, relation.getAttrTypeId)
            val entity2 = getCachedEntity(relation.getRelatedId2, cachedEntitiesIn, relation.mDB)
            if (isAllowedToExport(entity2, includePublicDataIn, includeNonPublicDataIn, includeUnspecifiedDataIn,
                                  levelsToExportIsInfiniteIn, levelsRemainingToExportIn - 1)) {
              if (entity2.getClassId.isDefined && entity2.getClassId.get == uriClassIdIn) {
                printListItemForUriEntity(uriClassIdIn, quoteClassIdIn, printWriter, entity2, cachedAttrsIn)
              } else {
                // i.e., don't create this link if it will be a broken link due to not creating the page later; also creating the link could disclose
                // info in the link itself (the entity name) that has been restricted (ex., made nonpublic).
                printListItemForEntity(printWriter, relationType, entity2)
              }
            }
          case relation: RelationToRemoteEntity =>
            val relationType = new RelationType(relation.mDB, relation.getAttrTypeId)
            // Idea: The next line doesn't currently internally do caching for DBs like we do for entities in getCachedEntity, but that could be added if it is
            // used often enough to be a performance problem (and at similar comment elsewhere in this file)
            val remoteDb: Database = relation.getRemoteDatabase
            val entity2 = getCachedEntity(relation.getRelatedId2, cachedEntitiesIn, remoteDb)
            if (isAllowedToExport(entity2, includePublicDataIn, includeNonPublicDataIn, includeUnspecifiedDataIn,
                                  levelsToExportIsInfiniteIn, levelsRemainingToExportIn - 1)) {
              // The classId and uriClassIdIn probably won't match because entity2 n all its data comes from a different (remote) db, so not checking that, at
              // least until that sort of cross-db check is supported, so skipping this condition for now (as elsewhere):
//              if (entity2.getClassId.isDefined && entity2.getClassId.get == uriClassIdIn) {
//                printListItemForUriEntity(uriClassIdIn, quoteClassIdIn, printWriter, entity2, cachedAttrsIn)
//              } else {
                // i.e., don't create this link if it will be a broken link due to not creating the page later; also creating the link could disclose
                // info in the link itself (the entity name) that has been restricted (ex., made nonpublic).
                printListItemForEntity(printWriter, relationType, entity2)
//              }
            }
          case relation: RelationToGroup =>
            val relationType = new RelationType(relation.mDB, relation.getAttrTypeId)
            val group = new Group(relation.mDB, relation.getGroupId)
            // if a group name is different from its entity name, indicate the differing group name also, otherwise complete the line just above w/ NL
            printWriter.println("    <li>" + htmlEncode(relation.getDisplayString(0, None, Some(relationType), simplify = true)) + "</li>")
            printWriter.println("    <ul>")

            // this 'if' check is duplicate with the call just below to isAllowedToExport, but can quickly save the time looping through them all,
            // checking entities, if there's no need:
            if (levelsToExportIsInfiniteIn || levelsRemainingToExportIn - 1 > 0) {
              for (entityInGrp: Entity <- group.getGroupEntries(0).toArray(Array[Entity]())) {
                // i.e., don't create this link if it will be a broken link due to not creating the page later; also creating the link could disclose
                // info in the link itself (the entity name) that has been restricted (ex., made nonpublic).
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
            val typeName: String = getCachedEntity(textAttr.getAttrTypeId, cachedEntitiesIn, textAttr.mDB).getName
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

  def printListItemForUriEntity(uriClassIdIn: Long, quoteClassIdIn: Long, printWriter: PrintWriter, uriEntity: Entity,
                                cachedAttrsIn: mutable.HashMap[Long, Array[(Long, Attribute)]]): Unit = {
    // handle URIs differently than other entities: make it a link as indicated by the URI contents, not to a newly created entity page..
    // (could use a more efficient call in cpu time than getSortedAttributes, but it's efficient in programmer time:)
    def findUriAttribute(): Option[TextAttribute] = {
      val attributesOnEntity2: Array[(Long, Attribute)] = getCachedAttributes(uriEntity, cachedAttrsIn)
      val uriTemplateId: Long = new EntityClass(uriEntity.mDB, uriClassIdIn).getTemplateEntityId
      for (attrTuple <- attributesOnEntity2) {
        val attr2: Attribute = attrTuple._2
        if (attr2.getAttrTypeId == uriTemplateId && attr2.isInstanceOf[TextAttribute]) {
          return Some(attr2.asInstanceOf[TextAttribute])
        }
      }
      None
    }
    def findQuoteText(): Option[String] = {
      val attributesOnEntity2: Array[(Long, Attribute)] = getCachedAttributes(uriEntity, cachedAttrsIn)
      val quoteClassTemplateId: Long = new EntityClass(uriEntity.mDB, quoteClassIdIn).getTemplateEntityId
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
      throw new OmException("Unable to find TextAttribute of type URI (classId=" + uriClassIdIn + ") for entity " + uriEntity.getId)
    }
    // this one can be None and it's no surprise:
    val quoteText: Option[String] = findQuoteText()
    printHtmlListItemWithLink(printWriter, "", uriAttribute.get.getText, uriEntity.getName, None, quoteText)
  }

  def printListItemForEntity(printWriterIn: PrintWriter, relationTypeIn: RelationType, entityIn: Entity): Unit = {
    val numSubEntries = getNumSubEntries(entityIn)
    if (numSubEntries > 0) {
      val relatedEntitysFileNamePrefix: String = getExportFileNamePrefix(entityIn, ImportExport.HTML_EXPORT_TYPE)
      printHtmlListItemWithLink(printWriterIn,
                                if (relationTypeIn.getName == Database.theHASrelationTypeName) "" else relationTypeIn.getName + ": ",
                                relatedEntitysFileNamePrefix + ".html",
                                entityIn.getName)
                                //removing next line until it matches better with what user can actually see: currently includes non-public stuff, so the #
                                //might confuse a reader, or at least doesn't set fulfillable expectations on how much content there is.
//                                Some("(" + numSubEntries + ")"))
    } else {
      val line = (if (relationTypeIn.getName == Database.theHASrelationTypeName) "" else relationTypeIn.getName + ": ") +
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
                                   outputDirectoryIn: Path, exportedEntityIdsIn: mutable.TreeSet[String], cachedEntitiesIn: mutable.HashMap[String, Entity],
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
    val attrTuples: Array[(Long, Attribute)] = getCachedAttributes(entityIn, cachedAttrsIn)
    for (attributeTuple <- attrTuples) {
      val attribute: Attribute = attributeTuple._2
      attribute match {
        case relation: RelationToLocalEntity =>
          val entity2: Entity = getCachedEntity(relation.getRelatedId2, cachedEntitiesIn, relation.mDB)
          if (entity2.getClassId.isEmpty || entity2.getClassId.get != uriClassIdIn) {
            // that means it's not a URI but an actual traversable thing to follow when exporting children:
            exportHtml(entity2, levelsToExportIsInfiniteIn, levelsRemainingToExportIn - 1,
                       outputDirectoryIn, exportedEntityIdsIn, cachedEntitiesIn,
                       cachedAttrsIn, cachedGroupInfoIn, entitiesAlreadyProcessedInThisRefChainIn, uriClassIdIn, quoteClassId,
                       includePublicDataIn, includeNonPublicDataIn, includeUnspecifiedDataIn,
                       headerContentIn, beginBodyContentIn, copyrightYearAndNameIn)
          }
        case relation: RelationToRemoteEntity =>
          // Idea: The next line doesn't currently internally do caching for DBs like we do for entities in getCachedEntity, but that could be added if it is
          // used often enough to be a performance problem (and at similar comment elsewhere in this file)
          val remoteDb = relation.getRemoteDatabase
          val entity2: Entity = getCachedEntity(relation.getRelatedId2, cachedEntitiesIn, remoteDb)
          // The classId and uriClassIdIn probably won't match because entity2 n all its data comes from a different (remote) db, so not checking that, at
          // least until that sort of cross-db check is supported, so skipping this condition for now (as elsewhere):
//          if (entity2.getClassId.isEmpty || entity2.getClassId.get != uriClassIdIn) {
//            // that means it's not a URI but an actual traversable thing to follow when exporting children:
            exportHtml(entity2, levelsToExportIsInfiniteIn, levelsRemainingToExportIn - 1,
                       outputDirectoryIn, exportedEntityIdsIn, cachedEntitiesIn,
                       cachedAttrsIn, cachedGroupInfoIn, entitiesAlreadyProcessedInThisRefChainIn, uriClassIdIn, quoteClassId,
                       includePublicDataIn, includeNonPublicDataIn, includeUnspecifiedDataIn,
                       headerContentIn, beginBodyContentIn, copyrightYearAndNameIn)
//          }
        case relation: RelationToGroup =>
          val entityIds: Array[Long] = getCachedGroupData(relation, cachedGroupInfoIn)
          for (entityIdInGrp <- entityIds) {
            val entityInGrp: Entity = getCachedEntity(entityIdInGrp, cachedEntitiesIn, relation.mDB)
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

  def getCachedGroupData(rtg: RelationToGroup, cachedGroupInfoIn: mutable.HashMap[Long, Array[Long]]): Array[Long] = {
    val cachedIds: Option[Array[Long]] = cachedGroupInfoIn.get(rtg.getGroupId)
    if (cachedIds.isDefined) {
      cachedIds.get
    } else {
      val data: List[Array[Option[Any]]] = rtg.mDB.getGroupEntriesData(rtg.getGroupId, None, includeArchivedEntitiesIn = false)
      val entityIds = new Array[Long](data.size)
      var count = 0
      for (entry <- data) {
        val entityIdInGroup: Long = entry(0).get.asInstanceOf[Long]
        entityIds(count) = entityIdInGroup
        count += 1
      }
      cachedGroupInfoIn.put(rtg.getGroupId, entityIds)
      entityIds
    }
  }

  def getCachedAttributes(entityIn: Entity, cachedAttrsIn: mutable.HashMap[Long, Array[(Long, Attribute)]]): Array[(Long, Attribute)] = {
    val cachedInfo: Option[Array[(Long, Attribute)]] = cachedAttrsIn.get(entityIn.getId)
    if (cachedInfo.isDefined) {
      cachedInfo.get
    } else {
      val attrTuples = entityIn.getSortedAttributes(0, 0, onlyPublicEntitiesIn = false)._1
      // record, so we don't create files more than once, calculate attributes more than once, etc.
      cachedAttrsIn.put(entityIn.getId, attrTuples)
      attrTuples
    }
  }

  def getCachedEntity(entityIdIn: Long, cachedEntitiesIn: mutable.HashMap[String, Entity], dbIn: Database): Entity = {
    val key: String = dbIn.id + entityIdIn.toString
    val cachedInfo: Option[Entity] = cachedEntitiesIn.get(key)
    if (cachedInfo.isDefined) {
      cachedInfo.get
    } else {
      val entity = new Entity(dbIn, entityIdIn)
      cachedEntitiesIn.put(key, entity)
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
    *
    * @return  Whether lines were wrapped--so a later call to it can decide whether to print a leading blank line.
    */
  def exportToSingleTextFile(entityIn: Entity, levelsToExportIsInfiniteIn: Boolean, levelsRemainingToExportIn: Int, currentIndentationLevelsIn: Int,
                             printWriterIn: PrintWriter,
                             includeMetadataIn: Boolean, exportedEntityIdsIn: mutable.TreeSet[String], cachedEntitiesIn: mutable.HashMap[String, Entity],
                             cachedAttrsIn: mutable.HashMap[Long, Array[(Long, Attribute)]], spacesPerIndentLevelIn: Int,
                             //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) w/in this method!
                             includePublicDataIn: Boolean, includeNonPublicDataIn: Boolean, includeUnspecifiedDataIn: Boolean,
                             wrapLongLinesIn: Boolean = false, wrapColumnIn: Int = 80, includeOutlineNumberingIn: Boolean = true,
                             outlineNumbersTrackingInOut: java.util.ArrayList[Int] = new java.util.ArrayList[Int],
                             previousEntityWasWrappedIn: Boolean = false): Boolean = {
    // useful while debugging, but maybe can also put that in the expression evaluator (^U)
    //printWriterIn.flush()

    var previousEntityWasWrapped = previousEntityWasWrappedIn
    val isFirstEntryOfAll: Boolean = outlineNumbersTrackingInOut.size == 0

    def incrementOutlineNumbering(): Unit = {
      // Don't do on the first entry: because that is just the header and
      // shouldn't have a number, and the outlineNumbersTrackingInOut info
      // isn't there to increment so it would fail anyway:
      if (!isFirstEntryOfAll) {
        val lastIndex = outlineNumbersTrackingInOut.size() - 1
        val incrementedLastNumber = outlineNumbersTrackingInOut.get(lastIndex) + 1
        outlineNumbersTrackingInOut.set(lastIndex, incrementedLastNumber)
      }
    }

    def getLineNumbers(includeOutlineNumbering: Boolean = true, nextKnownOutlineNumbers: java.util.ArrayList[Int]): String = {
      // (just a check, to learn. Maybe there is a better spot for it)
      require(currentIndentationLevelsIn == nextKnownOutlineNumbers.size)

      val s = new StringBuffer
      if (includeOutlineNumbering && nextKnownOutlineNumbers.size > 0) {
        // (if nextKnownOutlineNumbersIn.size == 0, it is the first line/entity in the exported file, ie, just the
        // containing entity or heading for the rest, so nothing to do.
        for (i <- 0 until nextKnownOutlineNumbers.size) {
          s.append(nextKnownOutlineNumbers.get(i))
          if (nextKnownOutlineNumbers.size() - 1 > i) s.append(".")
        }
      }
      s.toString
    }

    /** Does optional line wrapping and spacing for readability.
      * @param printWriterIn  The destination to print to.
      * @param entryText  The text to print, like an entity name.
      * @return  Whether lines were wrapped--so a later call to it can decide whether to print a leading blank line.
      */
    def printEntry(printWriterIn: PrintWriter, entryText: String): Boolean = {
      // (Idea:  this method feels overcomplicated.  Maybe some sub-methods could be broken out or the logic made
      // consistent but simpler.  I do use the features though, for how outlines are spaced etc., and it has been well-tested.)
      val indentingSpaces: String = {
        val adjustedCurrentIndentationLevelsIn = {
          if (wrapLongLinesIn) {
            // As also mentioned where we prompt the user in method "askForExportChoices" above, the one extra (initial) indent does not
            // seem helpful for readability, and can sometimes hinder it, such as if the exported content is going to become a
            // document or email message.
            Math.max(0, currentIndentationLevelsIn - 1)
          } else {
            currentIndentationLevelsIn
          }
        }
        getSpaces(adjustedCurrentIndentationLevelsIn * spacesPerIndentLevelIn)
      }
      incrementOutlineNumbering()
      val lineNumbers: String = getLineNumbers(includeOutlineNumberingIn, outlineNumbersTrackingInOut)
      var numCharactersBeforeActualContent = indentingSpaces.length + lineNumbers.length
      var stillToPrint: String = indentingSpaces + lineNumbers
      if (lineNumbers.length > 0 ) {
        stillToPrint = stillToPrint + " "
        numCharactersBeforeActualContent += 1
      }
      val wrappingThisEntrysLines: Boolean = wrapLongLinesIn && (stillToPrint.length + entryText.length) > wrapColumnIn


      if (includeOutlineNumberingIn) {
        // Just do the more complicated/optimized whitespace additions if adding outline numbers,
        // because only there is it trying to conserve vertical space (for now), with the numbers
        // helping readability to compensate for less vertical whitespace in some places.  This might let
        // exported content print on fewer sheets and require less page-turning.
        if (wrappingThisEntrysLines && !previousEntityWasWrapped) {
          // In this case we just had a single-line entry (which don't always have a blank line after),
          // now being followed by a wrapped (multi-line) one,
          // and it makes it easier to read if there is also a preceding blank line *before* a wrapped block.
          stillToPrint = Util.NEWLN + stillToPrint + entryText
        } else {
          stillToPrint = stillToPrint + entryText
        }
      } else {
        stillToPrint = stillToPrint + entryText
      }

      if (! wrappingThisEntrysLines) {
        // print the one line, no need to wrap.
        // (No extra trailing NEWLN needed for readability if printing unwrapped lines, for example,
        // if (includeOutlineNumberingIn == true), or if doing just a basic export without readability
        // enhancements (because of tests' assumptions about size, and no need.)
        printWriterIn.println(stillToPrint)
      } else {
        while (stillToPrint.length > 0) {
          // figure out how much to print, out of a long line
          //("wrapColumnIn - 1", is there to still respect the limit (wrapColumnIn) given that we do
          // + 1 afterward to include the trailing space.)
          var lastSpaceIndex = stillToPrint.lastIndexOf(" ", wrapColumnIn - 1)
          val endLineIndex =
            if (lastSpaceIndex > numCharactersBeforeActualContent && stillToPrint.length > wrapColumnIn) {
              // + 1 to include the space on the end of this line, instead of leaving it at the beginning of the
              // next one as excess initial whitespace.
              lastSpaceIndex + 1
            } else {
              // prevent endless loop of printing prefixes and adding more prefixes to print:
              Math.max(numCharactersBeforeActualContent + 1, Math.min(wrapColumnIn, stillToPrint.length))
            }

          // print the part of the line that fits
          printWriterIn.println(stillToPrint.substring(0, endLineIndex))

          // (fix for the next loop through, so it won't include the outline number now (if any))
          numCharactersBeforeActualContent = indentingSpaces.length
          if (stillToPrint.substring(endLineIndex).length > 0) {
            stillToPrint = indentingSpaces + stillToPrint.substring(endLineIndex)
          } else {
            stillToPrint = stillToPrint.substring(endLineIndex)
            // in other words, done with the content:
            assert(stillToPrint.length == 0)
           }
        }
      }
      if (isFirstEntryOfAll && wrapLongLinesIn) {
        // Just a readability convenience: underline the very top entry (since its children
        // are not indented under it--to set it off visually as something like a "title".
        var length = Math.min(wrapColumnIn, entryText.length)
        // (Compare use of "entryText.lastIndexOf(..."  with the "val lastSpaceIndex = " line elsewhere.
        val underline: StringBuffer = new StringBuffer(wrapColumnIn)
        for (_ <- 1 to length) {
          underline.append("-")
        }
        printWriterIn.println(underline)
      }
      if (wrappingThisEntrysLines || (wrapLongLinesIn && !includeOutlineNumberingIn)) {
        // whitespace for readability
        printWriterIn.println()
      }
      // Return whether we *did* wrap lines:
      wrappingThisEntrysLines
    }



    val entityName = entityIn.getName
    if (exportedEntityIdsIn.contains(entityIn.uniqueIdentifier)) {
      // it is a duplicate of something already exported, so just print a stub.
      val infoToPrint = if (includeMetadataIn) {
        "(duplicate: EN --> " + entityIn.getId + ": " + entityName + ")"
      } else {
        entityName
      }
      previousEntityWasWrapped = printEntry(printWriterIn, infoToPrint)
    } else {
      val allowedToExport: Boolean = isAllowedToExport(entityIn, includePublicDataIn, includeNonPublicDataIn, includeUnspecifiedDataIn,
                                                       levelsToExportIsInfiniteIn, levelsRemainingToExportIn)
      if (allowedToExport) {
        //record it, so we don't create duplicate entries:
        exportedEntityIdsIn.add(entityIn.uniqueIdentifier)

        val infoToPrint = if (includeMetadataIn) {
          "EN " + entityIn.getId + ": " + entityIn.getDisplayString()
        } else {
          entityName
        }

        previousEntityWasWrapped =
          printEntry(printWriterIn, infoToPrint)

        val attrTuples: Array[(Long, Attribute)] = getCachedAttributes(entityIn, cachedAttrsIn)
        outlineNumbersTrackingInOut.add(0)
        for (attributeTuple <- attrTuples) {
          val attribute:Attribute = attributeTuple._2
          attribute match {
            case relation: RelationToLocalEntity =>
              val relationType = new RelationType(relation.mDB, relation.getAttrTypeId)
              val entity2 = new Entity(relation.mDB, relation.getRelatedId2)
              if (includeMetadataIn) {
                printWriterIn.print(getSpaces((currentIndentationLevelsIn + 1) * spacesPerIndentLevelIn))
                printWriterIn.println(attribute.getDisplayString(0, Some(entity2), Some(relationType)))
              }
              // Idea: write tests to confirm that printing metadata as just above and the entity as just below, will all
              // work together with features such as wrapping, entities containing entities directly rather than via groups,
              // duplicate entities tracked via exportedEntityIdsIn, and all other attr types & variations on the parameters
              // to exportToSingleTxtFile.  Or wait for a need.
              previousEntityWasWrapped = exportToSingleTextFile(entity2, levelsToExportIsInfiniteIn, levelsRemainingToExportIn - 1,
                                                                currentIndentationLevelsIn + 1, printWriterIn,
                                                                includeMetadataIn, exportedEntityIdsIn, cachedEntitiesIn, cachedAttrsIn, spacesPerIndentLevelIn,
                                                                includePublicDataIn, includeNonPublicDataIn, includeUnspecifiedDataIn,
                                                                wrapLongLinesIn, wrapColumnIn, includeOutlineNumberingIn, outlineNumbersTrackingInOut,
                                                                previousEntityWasWrapped)
            case relation: RelationToRemoteEntity =>
              val relationType = new RelationType(relation.mDB, relation.getAttrTypeId)
              val remoteDb: Database = relation.getRemoteDatabase
              val entity2 = new Entity(remoteDb, relation.getRelatedId2)
              if (includeMetadataIn) {
                printWriterIn.print(getSpaces((currentIndentationLevelsIn + 1) * spacesPerIndentLevelIn))
                printWriterIn.println(attribute.getDisplayString(0, Some(entity2), Some(relationType)))
              }
              previousEntityWasWrapped = exportToSingleTextFile(entity2, levelsToExportIsInfiniteIn, levelsRemainingToExportIn - 1,
                                                                currentIndentationLevelsIn + 1, printWriterIn,
                                                                includeMetadataIn, exportedEntityIdsIn, cachedEntitiesIn, cachedAttrsIn, spacesPerIndentLevelIn,
                                                                includePublicDataIn, includeNonPublicDataIn, includeUnspecifiedDataIn,
                                                                wrapLongLinesIn, wrapColumnIn, includeOutlineNumberingIn, outlineNumbersTrackingInOut,
                                                                previousEntityWasWrapped)
            case relation: RelationToGroup =>
              val relationType = new RelationType(relation.mDB, relation.getAttrTypeId)
              val group = new Group(relation.mDB, relation.getGroupId)
              val grpName = group.getName
              // if a group name is different from its entity name, indicate the differing group name also, otherwise complete the line just above w/ NL
              if (entityName != grpName) {
                printWriterIn.print(getSpaces((currentIndentationLevelsIn + 1) * spacesPerIndentLevelIn))
                printWriterIn.println("(" + relationType.getName + " group named: " + grpName + ")")
              }
              if (includeMetadataIn) {
                printWriterIn.print(getSpaces(currentIndentationLevelsIn * spacesPerIndentLevelIn))
                // plus one more level of spaces to make it look better but still ~equivalently/exchangeably importable:
                printWriterIn.print(getSpaces(spacesPerIndentLevelIn))
                printWriterIn.println("(group details: " + attribute.getDisplayString(0, None, Some(relationType)) + ")")
              }
              for (entityInGrp: Entity <- group.getGroupEntries(0).toArray(Array[Entity]())) {
                previousEntityWasWrapped = exportToSingleTextFile(entityInGrp, levelsToExportIsInfiniteIn, levelsRemainingToExportIn - 1,
                                                                  currentIndentationLevelsIn + 1, printWriterIn, includeMetadataIn,
                                                                  exportedEntityIdsIn, cachedEntitiesIn, cachedAttrsIn, spacesPerIndentLevelIn,
                                                                  includePublicDataIn, includeNonPublicDataIn, includeUnspecifiedDataIn,
                                                                  wrapLongLinesIn, wrapColumnIn, includeOutlineNumberingIn, outlineNumbersTrackingInOut,
                                                                  previousEntityWasWrapped)
              }
            case _ =>
              incrementOutlineNumbering()
              printWriterIn.print(getSpaces((currentIndentationLevelsIn + 1) * spacesPerIndentLevelIn))
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
        outlineNumbersTrackingInOut.remove(outlineNumbersTrackingInOut.size() - 1)
      }
    }
    return previousEntityWasWrapped
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
      val numAttrs = entityIn.getAttributeCount
      if (numAttrs == 1) {
        val (_, _, groupId, _, moreThanOneAvailable) = entityIn.findRelationToAndGroup
        if (groupId.isDefined && !moreThanOneAvailable) {
          entityIn.mDB.getGroupSize(groupId.get, 4)
        } else numAttrs
      } else numAttrs
    }
    numSubEntries
  }

  def getSpaces(num: Int): String = {
    val s: StringBuffer = new StringBuffer
    for (i <- 1 to num) {
      s.append(" ")
    }
    s.toString
  }

  def getExportFileNamePrefix(entity: Entity, exportTypeIn: String): String = {
    val entityIdentifier: String = {
      if (entity.mDB.isRemote) {
        require(entity.mDB.getRemoteAddress.isDefined)
        "remote-" + entity.readableIdentifier
      } else {
        entity.getId.toString
      }
    }
    if (exportTypeIn == ImportExport.HTML_EXPORT_TYPE) {
      // (The 'e' is for "entity"; for explanation see cmts in methods createOutputDir and createOutputFile.)
      "e" + entityIdentifier
    } else {
      //idea (also in task list): change this to be a reliable filename (incl no backslashes? limit it to a whitelist of chars? a simple fn for that?
      var fixedEntityName = entity.getName.replace(" ", "")
      fixedEntityName = fixedEntityName.replace("/", "-")
      //fixedEntityName = fixedEntityName.replace("\\","-")
      "onemodel-export_" + entityIdentifier + "_" + fixedEntityName + "-"
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
  def tryExportingTxt_FOR_TESTS(ids: java.util.ArrayList[Long], dbIn: Database, wrapLongLinesIn: Boolean = false,
                                wrapColumnIn: Int = 80, includeOutlineNumberingIn: Boolean = false): (String, File) = {
    assert(ids.size > 0)
    val entityId: Long = ids.get(0)
    val startingEntity: Entity = new Entity(dbIn, entityId)

    // see comments in ImportExport.export() method for explanation of these 3
    val exportedEntityIds = new mutable.TreeSet[String]
    val cachedEntities = new mutable.HashMap[String, Entity]
    val cachedAttrs = new mutable.HashMap[Long, Array[(Long, Attribute)]]

    val prefix: String = getExportFileNamePrefix(startingEntity, ImportExport.TEXT_EXPORT_TYPE)
    val (outputFile: File, outputWriter: PrintWriter) = createOutputFile(prefix, ImportExport.TEXT_EXPORT_TYPE, None)
    exportToSingleTextFile(startingEntity, levelsToExportIsInfiniteIn = true, 0, 0, outputWriter, includeMetadataIn = false, exportedEntityIds, cachedEntities,
                          cachedAttrs, 2, includePublicDataIn = true, includeNonPublicDataIn = true, includeUnspecifiedDataIn = true,
                          wrapLongLinesIn, wrapColumnIn, includeOutlineNumberingIn)
    assert(outputFile.exists)
    outputWriter.close()
    val firstNewFileContents: String = new Predef.String(Files.readAllBytes(outputFile.toPath))
    (firstNewFileContents, outputFile)
  }

}
