/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2014-2020 inclusive and 2023, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
struct ImportExport {

}
impl ImportExport {
/*%%
  // const TEXT_EXPORT_TYPE: &str = "text";
  // const HTML_EXPORT_TYPE: &str = "html";
package org.onemodel.core.controllers

import java.io._
import java.nio.file.{Files, Path, StandardCopyOption}

import org.onemodel.core._
import org.onemodel.core.model._
import org.onemodel.core.{OmException, TextUI}

import scala.annotation.tailrec
import scala.collection.mutable

object ImportExport {
}
//%%
 * When adding features to this class, any eventual db call that creates a transaction needs to have the info 'caller_manages_transactions_in = true' eventually
 * passed into it, from here, otherwise the rollback feature will fail.
 * /
class ImportExport(val ui: TextUI, controller: Controller) {
  let uriLineExample: String = "'nameForTheLink <uri>http://somelink.org/index.html</uri>'";

   * 1st parameter must be either an Entity or a RelationToGroup (what is the right way to do that, in the signature?).
    fn importCollapsibleOutlineAsGroups(firstContainingEntryIn: AnyRef) {
    //noinspection ComparingUnrelatedTypes
    require(firstContainingEntryIn.isInstanceOf[Entity] || firstContainingEntryIn.isInstanceOf[Group])
    let db: Database = {;
      //noinspection ComparingUnrelatedTypes,TypeCheckCanBeMatch
      if firstContainingEntryIn.isInstanceOf[Entity]) {
        firstContainingEntryIn.asInstanceOf[Entity].m_db
      } else {
        firstContainingEntryIn.asInstanceOf[Group].m_db
      }
    }
    let ans1: Option<String> = ui.ask_for_string(Some(Array("Enter file path (must exist, be readable, AND a text file with lines spaced in the form of a" +;
                                                          " collapsible outline where each level change is marked by 1 tab or 2 spaces; textAttribute content" +
                                                          " can be indicated by surrounding a body of text thus, without quotes: '<ta>text</ta>';" +
                                                          " a URI similarly with a line " + uriLineExample + ")," +
                                                          " then press Enter; ESC to cancel")),
                                               Some(Util::input_file_valid))
    if ans1.is_defined) {
      let path = ans1.get;
      let makeThem_public: Option<bool> = ui.ask_yes_no_question("Do you want the entities imported to be marked as public?  Set it to the value the " +;
                                                      "majority of imported data should have; you can then edit the individual settings afterward as " +
                                                      "needed.  Enter y for public, n for nonpublic, or a space for 'unknown/unspecified', aka decide later.",
                                                      Some(""), allow_blank_answer = true)
      let ans3 = ui.ask_yes_no_question("Keep the filename as the top level of the imported list? (Answering no will put the top level entries from inside" +;
                                     " the file, as entries directly under this entity or group; answering yes will create an entity for the file," +
                                     " and in it a group for the entries.)")
      if ans3.is_defined) {
        let creatingNewStartingGroupFromTheFilename: bool = ans3.get;
        //noinspection ComparingUnrelatedTypes
        let addingToExistingGroup: bool = firstContainingEntryIn.isInstanceOf[Group] && !creatingNewStartingGroupFromTheFilename;

        let putEntriesAtEndOption: Option<bool> = {;
          if addingToExistingGroup) {
            ui.ask_yes_no_question("Put the new entries at the end of the list? (No means put them at the beginning, the default.)")
          } else
            Some(false)
        }

        if putEntriesAtEndOption.is_defined) {
          //@tailrec: would be nice to use, but jvm doesn't support it, or something.
          fn tryIt() {
            //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) w/in this method!
            let mut reader: Reader = null;
            try {
              let putEntriesAtEnd: bool = putEntriesAtEndOption.get;
              let fileToImport = new File(path);
              reader = new FileReader(fileToImport)
              db.begin_trans()

              doTheImport(reader, fileToImport.getCanonicalPath, fileToImport.lastModified(), firstContainingEntryIn, creatingNewStartingGroupFromTheFilename,
                          addingToExistingGroup, putEntriesAtEnd, makeThem_public)

              let keepAnswer: Option<bool> = {;
                //idea: look into how long that time is (see below same cmt):
                let msg: String = "Group imported, but browse around to see if you want to keep it, " +;
                                  "then ESC back here to commit the changes....  (If you wait beyond some amount of time(?) or go beyond just viewing, " +
                                  "it seems that postgres will commit " +
                                  "the change whether you want it or not, even if the message at that time says 'rolled back...')"
                ui.display_text(msg)
                firstContainingEntryIn match {
                  case entity: Entity => new EntityMenu(ui, controller).entityMenu(entity)
                  case group: Group => new QuickGroupMenu(ui, controller).quickGroupMenu(firstContainingEntryIn.asInstanceOf[Group], 0,
                                                                                         containingEntityIn = None)
                  case _ => throw new OmException("??")
                }
                ui.ask_yes_no_question("Do you want to commit the changes as they were made?")
              }
              if keepAnswer.isEmpty || !keepAnswer.get) {
                db.rollback_trans()
                //idea: look into how long that time is (see above same cmt)
                ui.display_text("Rolled back the import: no changes made (unless you browsed farther, into code that had another commit, or " +
                               "waited too long and postgres committed it anyway...?).")
              } else {
                db.commit_trans()
              }
            } catch {
              case e: Exception =>
                db.rollback_trans()
                if reader != null) {
                  try reader.close()
                  catch {
                    case e: Exception =>
                    // ignore
                  }
                }
                let msg: String = {;
                  let stringWriter = new StringWriter();
                  e.printStackTrace(new PrintWriter(stringWriter))
                  stringWriter.toString
                }
                ui.display_text(msg + "\nError while importing; no changes made. ")
                let ans = ui.ask_yes_no_question("For some errors, you can go fix the file then come back here.  Retry now?", Some("y"));
                if ans.is_defined && ans.get) {
                  tryIt()
                }
            } finally {
              if reader != null) {
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
    fn getFirstNonSpaceIndex(line: Array[Byte], index: Int) -> Int {
    //idea: this logic might need to be fixed (9?):
    if line(index) == 9) {
      // could count tab as 1, but not testing with that for now:
      throw new OmException("tab not supported")
    }
    else if index >= line.length || (line(index) != ' ')) {
      index
    } else {
      getFirstNonSpaceIndex(line, index + 1)
    }
  }

    fn createAndAddEntityToGroup(line: String, group: Group, newSortingIndex: i64, isPublicIn: Option<bool>) -> Entity {
    let entityId: i64 = group.m_db.createEntity(line.trim, group.getClassId, isPublicIn);
    group.addEntity(entityId, Some(newSortingIndex), caller_manages_transactions_in = true)
    new Entity(group.m_db, entityId)
  }

  * The parameter lastEntityIdAdded means the one to which a new subgroup will be added, such as in a series of entities
     added to a list and the code needs to know about the most recent one, so if the line is further indented, it knows where to
     create the subgroup.

     We always start from the current container (entity or group) and add the new material to a entry (Entity (+ 1 subgroup if needed)) created there.

     The parameter lastIndentationlevel should be set to zero, from the original caller, and indent from there w/ the recursion.
  *
  @tailrec
  //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) w/in this method!
    fn importRestOfLines(r: LineNumberReader, lastEntityAdded: Option<Entity>, lastIndentationLevel: Int, containerList: List[AnyRef],
                                lastSortingIndexes: List[i64], observation_dateIn: i64, mixedClassesAllowedDefaultIn: bool,
                                makeThem_publicIn: Option<bool>) {
    // (see cmts just above about where we start)
    require(containerList.size == lastIndentationLevel + 1)
    // always should at least have an entry for the entity or group from where the user initiated this import, the base of all the adding.
    require(containerList.nonEmpty)
    // how do this type mgt better, like, in the signature? (also needed elsewhere):
    require(containerList.head.isInstanceOf[Entity] || containerList.head.isInstanceOf[Group])

    let spacesPerIndentLevel = 2;
    let lineUntrimmed: String = r.readLine();
    if lineUntrimmed != null) {
      let lineNumber = r.getLineNumber;

      // these indicate beg/end of TextAttribute content; CODE ASSUMES THEY ARE LOWER-CASE!, so making that explicit, to be sure in case we change them later.
      let beginTaMarker = "<ta>".toLowerCase;
      let endTaMarker = "</ta>".toLowerCase;
      let beginUriMarker = "<uri>".toLowerCase;
      let endUriMarker = "</uri>".toLowerCase;

      if lineUntrimmed.toLowerCase.contains(beginTaMarker)) {
        // we have a section of text marked for importing into a single TextAttribute:
        importTextAttributeContent(lineUntrimmed, r, lastEntityAdded.get, beginTaMarker, endTaMarker)
        importRestOfLines(r, lastEntityAdded, lastIndentationLevel, containerList, lastSortingIndexes, observation_dateIn, mixedClassesAllowedDefaultIn,
                          makeThem_publicIn)
      } else if lineUntrimmed.toLowerCase.contains(beginUriMarker)) {
        // we have a section of text marked for importing into a web link:
        importUriContent(lineUntrimmed, beginUriMarker, endUriMarker, lineNumber, lastEntityAdded.get, observation_dateIn,
                          makeThem_publicIn, caller_manages_transactions_in = true)
        importRestOfLines(r, lastEntityAdded, lastIndentationLevel, containerList, lastSortingIndexes, observation_dateIn, mixedClassesAllowedDefaultIn,
                          makeThem_publicIn)
      } else {
        let line: String = lineUntrimmed.trim;

        if line == "." || line.isEmpty) {
          // nothing to do: that kind of line was just to create whitespace in my outline. So simply go to the next line:
          importRestOfLines(r, lastEntityAdded, lastIndentationLevel, containerList, lastSortingIndexes, observation_dateIn, mixedClassesAllowedDefaultIn,
                            makeThem_publicIn)
        } else {
          if line.length > Util::maxNameLength) throw new OmException("Line " + lineNumber + " is over " + Util::maxNameLength + " characters " +
                                                               " (has " + line.length + "): " + line)
          let indentationSpaceCount: i32 = getFirstNonSpaceIndex(lineUntrimmed.getBytes, 0);
          if indentationSpaceCount % spacesPerIndentLevel != 0) throw new OmException("# of spaces is off, on line " + lineNumber + ": '" + line + "'")
          let newIndentationLevel = indentationSpaceCount / spacesPerIndentLevel;
          if newIndentationLevel == lastIndentationLevel) {
            require(lastIndentationLevel >= 0)
            // same level, so add line to same entity group
            let newSortingIndex = lastSortingIndexes.head + 1;
            let newEntity: Entity = {;
              containerList.head match {
                case entity: Entity =>
                  entity.createEntityAndAddHASLocalRelationToIt(line, observation_dateIn, makeThem_publicIn, caller_manages_transactions_in = true)._1
                case group: Group =>
                  createAndAddEntityToGroup(line, containerList.head.asInstanceOf[Group], newSortingIndex, makeThem_publicIn)
                case _ => throw new OmException("??")
              }
            }

            importRestOfLines(r, Some(newEntity), lastIndentationLevel, containerList, newSortingIndex :: lastSortingIndexes.tail, observation_dateIn,
                              mixedClassesAllowedDefaultIn, makeThem_publicIn)
          } else if newIndentationLevel < lastIndentationLevel) {
            require(lastIndentationLevel >= 0)
            // outdented, so need to go back up to a containing group (list), to add line
            let numLevelsBack = lastIndentationLevel - newIndentationLevel;
            require(numLevelsBack > 0 && lastIndentationLevel - numLevelsBack >= 0)
            let newContainerList = containerList.drop(numLevelsBack);
            let newSortingIndexList = lastSortingIndexes.drop(numLevelsBack);
            let newSortingIndex = newSortingIndexList.head + 1;
            let newEntity: Entity = {;
              newContainerList.head match {
                case entity: Entity =>
                  entity.createEntityAndAddHASLocalRelationToIt(line, observation_dateIn, makeThem_publicIn, caller_manages_transactions_in = true)._1
                case group: Group =>
                  createAndAddEntityToGroup(line, group, newSortingIndex, makeThem_publicIn)
                case _ => throw new OmException("??")
              }
            }
            importRestOfLines(r, Some(newEntity), newIndentationLevel, newContainerList, newSortingIndex :: newSortingIndexList.tail, observation_dateIn,
                              mixedClassesAllowedDefaultIn, makeThem_publicIn)
          } else if newIndentationLevel > lastIndentationLevel) {
            // indented, so create a subgroup & add line there:
            require(newIndentationLevel >= 0)
            // (not None because it will be used now to create a subgroup; when we get here there should always be a value) :
            if lastEntityAdded.isEmpty) {
              throw new OmException("There's an error.  Are you importing a file to a group, but the first line is indented?  If so try fixing that " +
                                    "(un-indent, & fix the rest to match).  Otherwise, there's a bug in the program.")
            }
            let addedLevelsIn = newIndentationLevel - lastIndentationLevel;
            if addedLevelsIn != 1) throw new OmException("Unsupported format: line " + lineNumber + " is indented too far in, " +
                                                          "relative to the line before it: " + line)
            let mixedClassesAllowed: bool = {;
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
            let newGroup: Group = lastEntityAdded.get.create_groupAndAddHASRelationToIt(lastEntityAdded.get.get_name, mixedClassesAllowed,;
                                                                                       observation_dateIn, caller_manages_transactions_in = true)._1
            // since a new grp, start at beginning of sorting indexes
            let newSortingIndex = Database.min_id_value;
            let newSubEntity: Entity = createAndAddEntityToGroup(line, newGroup, newSortingIndex, makeThem_publicIn);
            importRestOfLines(r, Some(newSubEntity), newIndentationLevel, newGroup :: containerList, newSortingIndex :: lastSortingIndexes,
                              observation_dateIn, mixedClassesAllowedDefaultIn, makeThem_publicIn)
          } else throw new OmException("Shouldn't get here!?: " + lastIndentationLevel + ", " + newIndentationLevel)
        }
      }
    }
  }

    fn importTextAttributeContent(lineUntrimmedIn: String, r: LineNumberReader, entity_in: Entity, beginningTagMarker: String, endTaMarker: String) {
    let lineContentBeforeMarker = lineUntrimmedIn.substring(0, lineUntrimmedIn.toLowerCase.indexOf(beginningTagMarker)).trim;
    let restOfLine = lineUntrimmedIn.substring(lineUntrimmedIn.toLowerCase.indexOf(beginningTagMarker) + beginningTagMarker.length).trim;
    if restOfLine.toLowerCase.contains(endTaMarker)) throw new OmException("\"Unsupported format at line " + r.getLineNumber + ": beginning and ending " +
                                                                            "markers must NOT be on the same line.")
    let attr_type_id: i64 = {;
      let idsByName: java.util.ArrayList[i64] = entity_in.m_db.find_all_entity_ids_by_name(lineContentBeforeMarker.trim, case_sensitive = true);
      if idsByName.size == 1)
        idsByName.get(0)
      else {
        // idea: alternatively, could use a generic one in this case?  Optionally?
        let prompt = "A name for the *type* of this text attribute was not provided; it would be the entire line content preceding the \"" +;
                     beginningTagMarker + "\" " +
                     "(it has to match an existing entity, case-sensitively)"
        //IDEA: this used to call Controller.chooseOrCreateObject_OrSaysCancelled instead. Removing it removes a prompt if the user pressed ESC during it,
        //and this lacks a convenient way to test it, and I don't know that anyone uses it right now. So maybe add a test sometime:
        let selection: Option[(IdWrapper, Boolean, String)] = controller.chooseOrCreateObject(entity_in.m_db,;
                                                                                              Some(List(prompt + ", so please choose one or ESC to abort" +
                                                                                                        " this import operation:")),
                                                                                              None, None, Util::TEXT_TYPE)
        if selection.isEmpty) {
          throw new OmException(prompt + " or selected.")
        } else {
          selection.get._1.get_id
        }
      }
    }
    let text: String = restOfLine.trim + "\n" + {;
      fn getRestOfLines(rIn: LineNumberReader, sbIn: mutable.StringBuilder) -> mutable.StringBuilder {
        // Don't trim, because we want to preserve formatting/whitespace here, including blank lines (always? -- yes, editably.).
        let line = rIn.readLine();
        if line == null) {
          sbIn
        } else {
          if line.toLowerCase.contains(endTaMarker.toLowerCase)) {
            let markerStartLocation = line.toLowerCase.indexOf(endTaMarker.toLowerCase);
            let markerEndLocation = markerStartLocation + endTaMarker.length;
            let lineNumber = r.getLineNumber;
            fn rtrim(s: String) -> String {
                s.replaceAll("\\s+$", "")
            }
            let rtrimmedLine = rtrim(line);
            if rtrimmedLine.substring(markerEndLocation).nonEmpty) throw new OmException("\"Unsupported format at line " + lineNumber +
                                                                                  ": A \"" + endTaMarker +
                                                                                  "\" (end text attribute) marker must be the last text on a line.")
            sbIn.append(line.substring(0, markerStartLocation))
          } else {
            sbIn.append(line + "\n")
            getRestOfLines(rIn, sbIn)
          }
        }
      }
      let builder = getRestOfLines(r, new mutable.StringBuilder);
      builder.toString()
    }
    entity_in.create_text_attribute(attr_type_id, text, caller_manages_transactions_in = true)
  }

    fn importUriContent(lineUntrimmedIn: String, beginningTagMarkerIn: String, endMarkerIn: String, lineNumberIn: Int,
                        lastEntityAddedIn: Entity, observation_dateIn: i64, makeThem_publicIn: Option<bool>, caller_manages_transactions_in: bool) {
    //NOTE/idea also in tasks: this all fits better in the class and action *tables*, with this code being stored there
    // also, which implies that the class doesn't need to be created because...it's already there.

    if ! lineUntrimmedIn.toLowerCase.contains(endMarkerIn)) throw new OmException("\"Unsupported format at line " + lineNumberIn + ": beginning and ending " +
                                                                                   "markers MUST be on the same line.")
    let lineContentBeforeMarker = lineUntrimmedIn.substring(0, lineUntrimmedIn.toLowerCase.indexOf(beginningTagMarkerIn)).trim;
    let lineContentFromBeginMarker = lineUntrimmedIn.substring(lineUntrimmedIn.toLowerCase.indexOf(beginningTagMarkerIn)).trim;
    let uriStartLocation: i32 = lineContentFromBeginMarker.toLowerCase.indexOf(beginningTagMarkerIn.toLowerCase) + beginningTagMarkerIn.length;
    let uriEndLocation: i32 = lineContentFromBeginMarker.toLowerCase.indexOf(endMarkerIn.toLowerCase);
    if lineContentFromBeginMarker.substring(uriEndLocation + endMarkerIn.length).trim.nonEmpty) {
      throw new OmException("\"Unsupported format at line " + lineNumberIn + ": A \"" + endMarkerIn + "\" (end URI attribute) marker " +
                            "must be the" + " last text on its line.")
    }
    let name = lineContentBeforeMarker.trim;
    let uri = lineContentFromBeginMarker.substring(uriStartLocation, uriEndLocation).trim;
    if name.isEmpty || uri.isEmpty) throw new OmException("\"Unsupported format at line " + lineNumberIn +
                                                           ": A URI line must be in the format (without quotes): " + uriLineExample)
    // (see note above on this being better in the class and action *tables*, but here for now until those features are ready)
    lastEntityAddedIn.addUriEntityWithUriAttribute(name, uri, observation_dateIn, makeThem_publicIn, caller_manages_transactions_in = true)
  }

  //@tailrec why not? needs that jvm fix first to work for the scala compiler?  see similar comments elsewhere on that? (does java8 provide it now?
  // wait for next debian stable version--jessie?--be4 it's probably worth finding out)
    fn doTheImport(dataSourceIn: Reader, dataSourceFullPath: String, dataSourceLastModifiedDate: i64, firstContainingEntryIn: AnyRef,
                  creatingNewStartingGroupFromTheFilename_in: bool, addingToExistingGroup: bool,
                  //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) w/in this method!
                  putEntriesAtEnd: bool, makeThem_publicIn: Option<bool>, mixedClassesAllowedDefaultIn: bool = false, testing: bool = false) {
    let mut r: LineNumberReader = null;
    r = new LineNumberReader(dataSourceIn)
    let containingEntry: AnyRef = {;
      firstContainingEntryIn match {
        case containingEntity: Entity =>
          if creatingNewStartingGroupFromTheFilename_in) {
            let group: Group = containingEntity.create_groupAndAddHASRelationToIt(dataSourceFullPath,;
                                                                                 mixedClassesAllowedIn = mixedClassesAllowedDefaultIn,
                                                                                 System.currentTimeMillis, caller_manages_transactions_in = true)._1
            group
          } else containingEntity
        case containingGroup: Group =>
          if creatingNewStartingGroupFromTheFilename_in) {
            let name = dataSourceFullPath;
            let newEntity: Entity = createAndAddEntityToGroup(name, containingGroup, containingGroup.findUnusedSortingIndex(), makeThem_publicIn);
            let newGroup: Group = newEntity.create_groupAndAddHASRelationToIt(name, containingGroup.getMixedClassesAllowed, System.currentTimeMillis,;
                                                                             caller_manages_transactions_in = true)._1
            newGroup
          } else {
            assert(addingToExistingGroup)
            // importing the new entries to an existing group
            new Group(containingGroup.m_db, containingGroup.get_id)
          }
        case _ => throw new OmException("??")
      }
    }
    // how manage this (& others like it) better using scala type system?:
    //noinspection ComparingUnrelatedTypes
    require(containingEntry.isInstanceOf[Entity] || containingEntry.isInstanceOf[Group])
    // in order to put the new entries at the end of those already there, find the last used sortingIndex, and use the next one (renumbering
    // if necessary (idea: make this optional: putting them at beginning (w/ m_db.min_id_value) or end (w/ highestCurrentSortingIndex)).
    let startingSortingIndex: i64 = {;
      if addingToExistingGroup && putEntriesAtEnd) {
        let containingGrp = containingEntry.asInstanceOf[Group];
        let nextSortingIndex: i64 = containingGrp.getHighestSortingIndex + 1;
        if nextSortingIndex == Database.min_id_value) {
          // we wrapped from the biggest to lowest i64 value
          containingGrp.renumber_sorting_indexes(caller_manages_transactions_in = true)
          let nextTriedNewSortingIndex: i64 = containingGrp.getHighestSortingIndex + 1;
          if nextSortingIndex == Database.min_id_value) {
            throw new OmException("Huh? How did we get two wraparounds in a row?")
          }
          nextTriedNewSortingIndex
        } else nextSortingIndex
      } else Database.min_id_value
    }

    importRestOfLines(r, None, 0, containingEntry :: Nil, startingSortingIndex :: Nil, dataSourceLastModifiedDate, mixedClassesAllowedDefaultIn,
                      makeThem_publicIn)
  }

  // idea: see comment in EntityMenu about scoping.
  fn export(entity_in: Entity, exportTypeIn: String, headerContentIn: Option<String>, beginBodyContentIn: Option<String>, copyrightYearAndNameIn: Option<String>) {
    fn askForExportChoices: (Boolean, String, Int, Boolean, Boolean, Boolean, Boolean, Boolean, Boolean, Int) {
      let levelsText = "number of levels to export";

      let ans: Option<String> = ui.ask_for_string(Some(Array("Enter " + levelsText + " (including this one; 0 = 'all'); ESC to cancel")),;
                                                Some(Util::is_numeric), Some("0"))
      if ans.isEmpty) return (true, "", 0, false, false, false, false, false, false, 1)
      let levelsToExport: i32 = ans.get.toInt;

      let ans2: Option<bool> = ui.ask_yes_no_question("Include metadata (verbose detail: id's, types...)?");
      if ans2.isEmpty) return (true, "", 0, false, false, false, false, false, false, 1)
      let includeMetadata: bool = ans2.get;

      //idea: make these choice strings into an enum? and/or the answers into an enum? what's the scala idiom? see same issue elsewhere
      let ans3: Option<bool> = ui.ask_yes_no_question("Include public data?  (Note: Whether an entity is public, non-public, or unset can be " +;
                                                                   "marked on each entity's menu, and the preference as to whether to display that status on " +
                                                                   "each entity in a list can be set via the main menu.)", Some("y"), allow_blank_answer = true)
      if ans3.isEmpty) return (true, "", 0, false, false, false, false, false, false, 1)
      let includePublicData: bool = ans3.get;

      let ans4: Option<bool> = ui.ask_yes_no_question("Include data marked non-public?", Some("n"), allow_blank_answer = true);
      if ans4.isEmpty) return (true, "", 0, false, false, false, false, false, false, 1)
      let includeNonPublicData: bool = ans4.get;

      let ans5: Option<bool> = ui.ask_yes_no_question("Include data not specified as public or non-public?", ;
                                                      (if exportTypeIn == ImportExport.TEXT_EXPORT_TYPE)
                                                        Some("y") else Some("n")),
                                                      allow_blank_answer = true)
      if ans5.isEmpty) return (true, "", 0, false, false, false, false, false, false, 1)
      let includeUnspecifiedData: bool = ans5.get;

      let mut numberTheLines: bool = false;
      let mut wrapTheLines: bool = false;
      let mut wrapAtColumn: i32 = 1;
      if exportTypeIn == ImportExport.TEXT_EXPORT_TYPE) {
        let ans6: Option<bool> = ui.ask_yes_no_question("Number the entries in outline form (ex, 3.1.5)?  (Prevents directly re-importing.)", Some("y"), allow_blank_answer = true);
        if ans6.isEmpty) return (true, "", 0, false, false, false, false, false, false, 1)
        numberTheLines = ans6.get

        // (See for more explanation on this prompt, the "adjustedCurrentIndentationLevels" variable used in a different method below.
        let ans7: Option<bool> = ui.ask_yes_no_question("Wrap long lines and add whitespace for readability?  (Prevents directly re-importing; also removes one level of indentation, needless in that case.)", Some("y"), allow_blank_answer = true);
        if ans7.isEmpty) return (true, "", 0, false, false, false, false, false, false, 1)
        wrapTheLines = ans7.get

        wrapAtColumn = {
          fn checkColumn(s: String) -> bool {
            Util::is_numeric(s) && s.toFloat > 0
          }
          let ans8: Option<String> = ui.ask_for_string(Some(Array("Wrap at what column (greater than 0)?")), Some(checkColumn), Some("80"),;
                                                       escKeySkipsCriteriaCheck = true)
          if ans8.isEmpty) return (true, "", 0, false, false, false, false, false, false, 1)
          ans8.get.toInt
        }
      }
      (false, levelsText, levelsToExport, includeMetadata, includePublicData, includeNonPublicData, includeUnspecifiedData, numberTheLines, wrapTheLines,
      wrapAtColumn)
    }


    let (userWantsOut: bool, levelsText: String, levelsToExport: Int, includeMetadata: bool, includePublicData: bool, includeNonPublicData: bool,;
         includeUnspecifiedData: bool, numberTheLines: bool, wrapTheLines: bool, wrapAtColumn: Int) = askForExportChoices

    fn getNumExportableEntries(cachedEntities: mutable.HashMap[String, Entity], cachedAttrs: mutable.HashMap[i64, Array[(i64, Attribute)]]) -> Integer {
      let mut count: Integer = 0;
      let attrTuples: Array[(i64, Attribute)] = getCachedAttributes(entity_in, cachedAttrs);
      for (attributeTuple <- attrTuples) {
        let attribute: Attribute = attributeTuple._2;
        attribute match {
          case relation: RelationToLocalEntity =>
            let e: Entity = getCachedEntity(relation.getRelatedId2, cachedEntities, relation.m_db);
            if levelsRemainAndPublicEnough(e, includePublicData, includeNonPublicData, includeUnspecifiedData,
                                            levelsToExportIsInfiniteIn = false, 1)) {
              count = count + 1
            }
          case relation: RelationToRemoteEntity =>
            // Idea: The next line doesn't currently internally do caching for DBs like we do for entities in getCachedEntity, but that could be added if it is
            // used often enough to be a performance problem (and at similar comment elsewhere in this file)
            // (AND THE SAME AT THE OTHER PLACES W/ SAME COMMENT.)
            let remoteDb = relation.getRemoteDatabase;
            let e: Entity = getCachedEntity(relation.getRelatedId2, cachedEntities, remoteDb);
            if levelsRemainAndPublicEnough(e, includePublicData, includeNonPublicData, includeUnspecifiedData,
                                            levelsToExportIsInfiniteIn = false, 1)) {
              count = count + 1
            }
          case relation: RelationToGroup =>
            // Needed, or is accurate without? (depends on how groups are processed in txt exports: if they count as top-level entities/shown so...
            // probably not so don't increment at all in that case?:
            //                    let entityIds: Array[i64] = getCachedGroupData(relation, cachedGroupInfo);
            //                    for (entityIdInGrp <- entityIds) {
            //                      let entity_in_group: Entity = getCachedEntity(entityIdInGrp, cachedEntities, relation.m_db);
            //                    if levelsRemainAndPublicEnough(e, includePublicData, includeNonPublicData, includeUnspecifiedData,
            //                                                    levelsToExportIsInfiniteIn = false, 1)) {
            count = count + 1
          //                    }
          //                    }
          case _ =>
            // Remove? Put back when (all?) attributes show up in exported text outlines?  And in the meantime, probably need to manually check the count every time this is used?
            count = count + 1
        }
      }
      count
    }

    if !userWantsOut) {
      ui.display_text("Processing...\n" + "(Note: if this takes too long, you can Ctrl+C and start over with a smaller or nonzero " + levelsText + ".)", false)
      require(levelsToExport >= 0)
      let spacesPerIndentLevel = {;
        if wrapTheLines && !numberTheLines) {
          // make it more obvious to readers using variable-width fonts that it is indented (someone might convert to another format,
          // and this might help it stay looking like an outline).
          6
        } else {
          // I would pick 2 as I usually use fixed-width, but readers with variable-width fonts if I send it to them, might still find it harder than 4.
          4
        }
      }

      // To track what's been done so we don't repeat it:  The first part (key) is the Entity.uniqueIdentifier.
      // The value (2nd part) is so we don't redo work if it has already been done.  (Adding this made an export of my web site
      // change from taking more than several days, to under a minute).  It is 0 if "infinite" (or all levels available).
      // (Note: if we later need to look up more than just an Integer, we could try mixing in a MultiMap.)
      // (NOTE: could compare the performance (and exported data!), of HashMap vs. TreeMap, *after* upgrading to a later
      // version of scala that has it?)
      let exportedEntityIds = new mutable.HashMap[String, Integer];

      // The caches are to reduce the expensive repeated queries of attribute lists & entity objects (not all of which are known at the time we write to
      // exportedEntityIds.  Html exports were getting very slow before this caching logic was added.)
      let cachedEntities = new mutable.HashMap[String, Entity];
      // (The key is the entityId, and the value contains the attributes (w/ id & attr) as returned from db.get_sorted_attributes.)
      let cachedAttrs = new mutable.HashMap[i64, Array[(i64, Attribute)]];

      let cachedGroupInfo = new mutable.HashMap[i64, Array[i64]];

      let prefix: String = getExportFileNamePrefix(entity_in, exportTypeIn);
      if exportTypeIn == ImportExport.TEXT_EXPORT_TYPE) {
        let (outputFile: File, outputWriter: PrintWriter) = createOutputFile(prefix, exportTypeIn, None);
        try {
          if wrapTheLines || numberTheLines) {
            let numEntries: Integer = getNumExportableEntries(cachedEntities, cachedAttrs);
            // The next line is debatable, but a point I want to make for now, and a personal convenience.  If you don't like it send a
            // comment on the list, or a patch with it removed, for discussion.
            // Or maybe we just remove the "wrapTheLines" part of the condition so it prints only with the numbered outline format.
            // Done here because the method exportToSingleTextFile is called recursively, and this needs to simply be first.
            // Maybe it (or at least the part after #1) should be replaced with a link to some page ~ "How to do structured skimming to get more out of
            // reading or spend less time".
*Ideas: when I export for others, call the recipient to go over this material verbally (or email?):
   just read #1
   then skip the 1.1, 1.2, etc as details
   then just read #2, skip the rest of 2.n
   then just read #3, skipping details to 4,5,6 to the end.
   ...to get an overview: this is *skimming*.
   then go back to #1. If it is interesting, read its top level (1.1, 1.2, 1.3) skipping details.
   same w/ #2, 3, etc.  pick the interesting ones & do that.
   is that helpful?
   please *please* send cmts to me as to whether it is helpful or not.
 THEN consider each kind of person (age, either some academic or tech bent, experiences, how it is best for each one.
*
            outputWriter.println("(This is an outline, generated from OM data (details at http://onemodel.org), with " + numEntries + " top-level items.  It is meant to be skimmable." +
                                 Util::NEWLN + "Here are some hints for skimming or reading outlines (and other things) efficiently:" +
                                 // WHEN PERSONAL WEB SITE UPDATED W/ THE CONTENTS (already in its todos), simply link here and delete the rest of the output?,
                                 // or keep the tip about just reading the outline, & link to the rest with this & make the overall text work:
                                 //Util::NEWLN + "  <a href=\"http://lukecall.net/e-9223372036854624718.html\">About reading outlines efficiently</a>"
                                 Util::NEWLN + "1) for an outline like this, read only the most out-dented parts," +
                                 " and then the indented parts only if interest in the parent entry justifies it." +
                                 Util::NEWLN + "The rest of this top section is not" +
                                 " for *this* outline, but has general tips on structured skimming that have helped me get more out of reading, in less" +
                                 " time. " +
                                 Util::NEWLN + "2) For essays or academic papers, read the first and" +
                                 " last paragraphs, then if interest remains, just the first sentences of paragraphs, and more only based on the value of" +
                                 " what was read already." +
                                 Util::NEWLN + "3) For news, one  can read just the beginning to get the most important info, and read more only if" +
                                 " you really want the increasing level of detail that comes in later parts of news articles. " +
                                 Util::NEWLN + "For more, see:  https://en.wikipedia.org/wiki/Skimming_(reading)#Skimming_and_scanning  .)" + Util::NEWLN)
          }

          exportToSingleTextFile(entity_in, levelsToExport == 0, levelsToExport, 0, outputWriter, includeMetadata,
                                 exportedEntityIds, cachedEntities, cachedAttrs,
                                 spacesPerIndentLevel, includePublicData, includeNonPublicData, includeUnspecifiedData, wrapTheLines,
                                 wrapAtColumn, numberTheLines)
          // flush before we report 'done' to the user:
          outputWriter.close()
          ui.display_text("Exported to file: " + outputFile.getCanonicalPath)
        } finally {
          if outputWriter != null) {
            try outputWriter.close()
            catch {
              case e: Exception =>
              // ignore
            }
          }
        }
      } else if exportTypeIn == ImportExport.HTML_EXPORT_TYPE) {
        let outputDirectory:Path = createOutputDir(prefix);
        // see note about this usage, in method importUriContent:
        let uriClassId: i64 = entity_in.m_db.get_or_create_class_and_template_entity("URI", caller_manages_transactions_in = true)._1;
        let quoteClassId = entity_in.m_db.get_or_create_class_and_template_entity("quote", caller_manages_transactions_in = true)._1;

        exportHtml(entity_in, levelsToExport == 0, levelsToExport, outputDirectory, exportedEntityIds, cachedEntities, cachedAttrs,
                   cachedGroupInfo, mutable.TreeSet[i64](), uriClassId, quoteClassId,
                   includePublicData, includeNonPublicData, includeUnspecifiedData, headerContentIn, beginBodyContentIn, copyrightYearAndNameIn)
        ui.display_text("Finished export to directory: " + outputDirectory.toFile.getCanonicalPath +
                       " at " + Util::DATEFORMAT2.format(System.currentTimeMillis()))
      } else {
        throw new OmException("unexpected value for exportTypeIn: " + exportTypeIn)
      }
    }
  }

  // This exists for the reasons commented in exportItsChildrenToHtmlFiles, and so that not all callers have to explicitly call both (ie, duplication of code).
    fn exportHtml(entity: Entity, levelsToExportIsInfinite: bool, levelsToExport: Int,
                 outputDirectory: Path, exportedEntityIdsIn: mutable.HashMap[String, Integer], cachedEntitiesIn: mutable.HashMap[String, Entity],
                 cachedAttrsIn: mutable.HashMap[i64, Array[(i64, Attribute)]], cachedGroupInfoIn: mutable.HashMap[i64, Array[i64]],
                 entitiesAlreadyProcessedInThisRefChain: mutable.TreeSet[i64],
                 uriClassId: i64, quoteClassId: i64,
                 includePublicData: bool, includeNonPublicData: bool, includeUnspecifiedData: bool,
                 headerContentIn: Option<String>, beginBodyContentIn: Option<String>, copyrightYearAndNameIn: Option<String>) {
    * The fix [FOR WHAT, AGAIN? See my OM noted todo "make count more accurate at top/header of expo", and other cmts made at same time w/ this commit?]:
        xwhenever adding an entry to that expanded data stru, add an integer saying how many levels are *going* to be done, 0 if all
        when that is checked to decide what to do
          xif not found (or after returned?), add, using the # planned to do or completed *OR 0if infinite), & go ahead.
          If found (alr done)
            if 0 || the # to do is <= than the # done, dont go ahead *even in the children checks*
            if nonzero && the # to do is > than the # done
                go ahead w/ the new #
                and update the # done
        make sure it is commented understandably
        same issue with txt exports?
        same issue with searching relative to a point in the tree (ie, that could be speeded up if there is much duplication w/in the tree)
          just note somewhere?
          same issue with anything that traverses trees?  probably found in some coding algorithms discussions...
          Don't do that for now:
            See/integrate updates w/ existing comments near to of PostgreSQLDatabase.find_contained_local_entity_ids .
        THEN REPEAT THIS FOR TXT EXPORT right?
     *
//    if !shouldExport(entity_in, includePublicDataIn, includeNonPublicDataIn, includeUnspecifiedDataIn, levelsToExportIsInfiniteIn, levels_remainingToExportIn)) {
//      return
//    }
    if !levelsRemainAndPublicEnough(entity, includePublicData, includeNonPublicData, includeUnspecifiedData, levelsToExportIsInfinite, levelsToExport)) {
      return
    }
    // (The next line's "alreadyExportedLevels" is a different concept from the previous line's check:
    // the next line is about *this time* into part of the tree, so we don't traverse the same sub-parts multiple times.
    // The "levelsRemainAndPublicEnough" call is about not ever exceeding the total levels being exported from the top.
    let alreadyExportedLevels: Option[Integer] = exportedEntityIdsIn.get(entity.uniqueIdentifier);
    let entityWasPreviouslyExported = alreadyExportedLevels.is_defined;
    if ! entityWasPreviouslyExported) {
      exportEntityToHtmlFile(entity, levelsToExportIsInfinite, levelsToExport, outputDirectory, exportedEntityIdsIn, cachedEntitiesIn, cachedAttrsIn,
                             uriClassId, quoteClassId, includePublicData, includeNonPublicData, includeUnspecifiedData,
                             headerContentIn, beginBodyContentIn, copyrightYearAndNameIn)

      //add it, so we don't create duplicate files, or loop infinitely while doing sub-entities aka children.
      exportedEntityIdsIn.update(entity.uniqueIdentifier, if levelsToExportIsInfinite) 0 else levelsToExport)

      exportItsChildrenToHtmlFiles(entity, levelsToExportIsInfinite, levelsToExport, outputDirectory, exportedEntityIdsIn, cachedEntitiesIn,
                                   cachedAttrsIn, cachedGroupInfoIn, entitiesAlreadyProcessedInThisRefChain, uriClassId, quoteClassId,
                                   includePublicData, includeNonPublicData, includeUnspecifiedData, headerContentIn, beginBodyContentIn,
                                   copyrightYearAndNameIn)
    } else {
      // No need to recreate this entity's html file since it was already done, but there is a further check before doing
      // children, if we need to go more levels deep now (see comments at or in exportItsChildrenToHtmlFiles, for details. Idea: move those here?).
      if alreadyExportedLevels.get != 0 && levelsToExport > alreadyExportedLevels.get) {
        exportItsChildrenToHtmlFiles(entity, levelsToExportIsInfinite, levelsToExport, outputDirectory, exportedEntityIdsIn, cachedEntitiesIn,
                                     cachedAttrsIn, cachedGroupInfoIn, entitiesAlreadyProcessedInThisRefChain, uriClassId, quoteClassId,
                                     includePublicData, includeNonPublicData, includeUnspecifiedData, headerContentIn, beginBodyContentIn,
                                     copyrightYearAndNameIn)
        exportedEntityIdsIn.update(entity.uniqueIdentifier, if levelsToExportIsInfinite) 0 else levelsToExport)
      } else {
        // don't go ahead with sub-entities: this work has already been done in a previous iteration.
      }
    }
  }

  * This creates a new file for each entity.
    *
    * If levelsToProcessIsInfiniteIn is true, then levels_remainingToProcessIn is irrelevant.
    *
    fn exportEntityToHtmlFile(entity_in: Entity, levelsToExportIsInfiniteIn: bool, levels_remainingToExportIn: Int,
                             outputDirectoryIn: Path, exportedEntityIdsIn: mutable.HashMap[String, Integer], cachedEntitiesIn: mutable.HashMap[String, Entity],
                             cachedAttrsIn: mutable.HashMap[i64, Array[(i64, Attribute)]],
                             uriClassIdIn: i64, quoteClassIdIn: i64,
                             includePublicDataIn: bool, includeNonPublicDataIn: bool, includeUnspecifiedDataIn: bool,
                             headerContentIn: Option<String>, beginBodyContentIn: Option<String>, copyrightYearAndNameIn: Option<String>) {
    // useful while debugging:
    //out.flush()

    let entitysFileNamePrefix: String = getExportFileNamePrefix(entity_in, ImportExport.HTML_EXPORT_TYPE);
    let printWriter = createOutputFile(entitysFileNamePrefix, ImportExport.HTML_EXPORT_TYPE, Some(outputDirectoryIn))._2;
    try {
      printWriter.println("<html><head>")
      printWriter.println("  <title>" + entity_in.get_name + "</title>")
      printWriter.println("  <meta name=\"description\" content=\"" + entity_in.get_name + "\">")
      printWriter.println("  " + headerContentIn.getOrElse(""))

      printWriter.println("</head>")
      printWriter.println()
      printWriter.println("<body>")
      printWriter.println("  " + beginBodyContentIn.getOrElse(""))
      printWriter.println("  <h1>" + htmlEncode(entity_in.get_name) + "</h1>")

      let attrTuples: Array[(i64, Attribute)] = getCachedAttributes(entity_in, cachedAttrsIn);
      printWriter.println("  <ul>")
      for (attrTuple <- attrTuples) {
        let attribute:Attribute = attrTuple._2;
        attribute match {
          case relation: RelationToLocalEntity =>
            let relationType = new RelationType(relation.m_db, relation.get_attr_type_id());
            let entity2 = getCachedEntity(relation.getRelatedId2, cachedEntitiesIn, relation.m_db);
            if levelsRemainAndPublicEnough(entity2, includePublicDataIn, includeNonPublicDataIn, includeUnspecifiedDataIn,
                                            levelsToExportIsInfiniteIn, levels_remainingToExportIn - 1)) {
              if entity2.getClassId.is_defined && entity2.getClassId.get == uriClassIdIn) {
                printListItemForUriEntity(uriClassIdIn, quoteClassIdIn, printWriter, entity2, cachedAttrsIn)
              } else {
                // i.e., don't create this link if it will be a broken link due to not creating the page later; also creating the link could disclose
                // info in the link itself (the entity name) that has been restricted (ex., made nonpublic).
                printListItemForEntity(printWriter, relationType, entity2)
              }
            }
          case relation: RelationToRemoteEntity =>
            let relationType = new RelationType(relation.m_db, relation.get_attr_type_id());
            // Idea: The next line doesn't currently internally do caching for DBs like we do for entities in getCachedEntity, but that could be added if it is
            // used often enough to be a performance problem (and at similar comment elsewhere in this file)
            // (AND THE SAME AT THE OTHER PLACES W/ SAME COMMENT.)
            let remoteDb: Database = relation.getRemoteDatabase;
            let entity2 = getCachedEntity(relation.getRelatedId2, cachedEntitiesIn, remoteDb);
            if levelsRemainAndPublicEnough(entity2, includePublicDataIn, includeNonPublicDataIn, includeUnspecifiedDataIn,
                                            levelsToExportIsInfiniteIn, levels_remainingToExportIn - 1)) {
              // The classId and uriClassIdIn probably won't match because entity2 n all its data comes from a different (remote) db, so not checking that, at
              // least until that sort of cross-db check is supported, so skipping this condition for now (as elsewhere):
//              if entity2.getClassId.is_defined && entity2.getClassId.get == uriClassIdIn) {
//                printListItemForUriEntity(uriClassIdIn, quoteClassIdIn, printWriter, entity2, cachedAttrsIn)
//              } else {
                // i.e., don't create this link if it will be a broken link due to not creating the page later; also creating the link could disclose
                // info in the link itself (the entity name) that has been restricted (ex., made nonpublic).
                printListItemForEntity(printWriter, relationType, entity2)
//              }
            }
          case relation: RelationToGroup =>
            let relationType = new RelationType(relation.m_db, relation.get_attr_type_id());
            let group = new Group(relation.m_db, relation.getGroupId);
            // if a group name is different from its entity name, indicate the differing group name also, otherwise complete the line just above w/ NL
            printWriter.println("    <li>" + htmlEncode(relation.get_display_string(0, None, Some(relationType), simplify = true)) + "</li>")
            printWriter.println("    <ul>")

            // this 'if' check is duplicate with the call just below to isAllowedToExport, but can quickly save the time looping through them all,
            // checking entities, if there's no need:
            if levelsToExportIsInfiniteIn || levels_remainingToExportIn - 1 > 0) {
              for (entity_in_group: Entity <- group.getGroupEntries(0).toArray(Array[Entity]())) {
                // i.e., don't create this link if it will be a broken link due to not creating the page later; also creating the link could disclose
                // info in the link itself (the entity name) that has been restricted (ex., made nonpublic).
                if levelsRemainAndPublicEnough(entity_in_group, includePublicDataIn, includeNonPublicDataIn, includeUnspecifiedDataIn,
                                                levelsToExportIsInfiniteIn, levels_remainingToExportIn - 1)) {
                  if entity_in_group.getClassId.is_defined && entity_in_group.getClassId.get == uriClassIdIn) {
                    printListItemForUriEntity(uriClassIdIn, quoteClassIdIn, printWriter, entity_in_group, cachedAttrsIn)
                  } else{
                    printListItemForEntity(printWriter, relationType, entity_in_group)
                  }
                }
              }
            }
            printWriter.println("    </ul>")
          case textAttr: TextAttribute =>
            let typeName: String = getCachedEntity(textAttr.get_attr_type_id(), cachedEntitiesIn, textAttr.m_db).get_name;
            if typeName==Util::HEADER_CONTENT_TAG || typeName == Util::BODY_CONTENT_TAG || typeName==Util::FOOTER_CONTENT_TAG) {
              //skip it: this is used to create the pages and should not be considered a normal kind of displayable content in them:
            } else {
              printWriter.println("    <li><pre>" + htmlEncode(textAttr.get_display_string(0, None, None, simplify = true)) + "</pre></li>")
            }
          case fileAttr: FileAttribute =>
            let originalPath = fileAttr.getOriginalFilePath;
            let fileName = {;
              if originalPath.indexOf("/") >= 0) originalPath.substring(originalPath.lastIndexOf("/") + 1)
              else if originalPath.indexOf("\\") >= 0) originalPath.substring(originalPath.lastIndexOf("\\") + 1)
              else originalPath
            }
            // (The use of the attribute id prevents problems if the same filename is used more than once on an entity:)
            let file: File = Files.createFile(new File(outputDirectoryIn.toFile, entitysFileNamePrefix + "-" + fileAttr.get_id + "-" + fileName).toPath).toFile;
            fileAttr.retrieveContent(file)
            if originalPath.toLowerCase.endsWith("png") || originalPath.toLowerCase.endsWith("jpg") || originalPath.toLowerCase.endsWith("jpeg") ||
                originalPath.toLowerCase.endsWith("gif")) {
              printWriter.println("    <li><img src=\"" + file.get_name + "\" alt=\"" + htmlEncode(fileAttr.get_display_string(0, None, None, simplify = true)) +
                                  "\"></li>")
            } else {
              printWriter.println("    <li><a href=\"" + file.get_name + "\">" + htmlEncode(fileAttr.get_display_string(0, None, None, simplify = true)) +
                                  "</a></li>")
            }
          case attr: Attribute =>
            printWriter.println("    <li>" + htmlEncode(attr.get_display_string(0, None, None, simplify = true)) + "</li>")
          case unexpected =>
            throw new OmException("How did we get here?: " + unexpected)
        }
      }
      printWriter.println("  </ul>")
      printWriter.println()
      if copyrightYearAndNameIn.is_defined) {
        // (intentionally not doing "htmlEncode(copyrightYearAndNameIn.get)", so that some ~footer-like links can be included in it.
        printWriter.println("  <center><p><small>Copyright " + copyrightYearAndNameIn.get + "</small></p></center>")
      }
      printWriter.println("</body></html>")
      printWriter.close()
    } finally {
      // close each file as we go along.
      if printWriter != null) {
        try printWriter.close()
        catch {
          case e: Exception =>
          // ignore
        }
      }
    }
  }

    fn printListItemForUriEntity(uriClassIdIn: i64, quoteClassIdIn: i64, printWriter: PrintWriter, uriEntity: Entity,
                                cachedAttrsIn: mutable.HashMap[i64, Array[(i64, Attribute)]]) /*-> Unit%%*/ {
    // handle URIs differently than other entities: make it a link as indicated by the URI contents, not to a newly created entity page..
    // (could use a more efficient call in cpu time than get_sorted_attributes, but it's efficient in programmer time:)
    fn findUriAttribute() -> Option[TextAttribute] {
      let attributesOnEntity2: Array[(i64, Attribute)] = getCachedAttributes(uriEntity, cachedAttrsIn);
      let uriTemplateId: i64 = new EntityClass(uriEntity.m_db, uriClassIdIn).get_template_entity_id;
      for (attrTuple <- attributesOnEntity2) {
        let attr2: Attribute = attrTuple._2;
        if attr2.get_attr_type_id() == uriTemplateId && attr2.isInstanceOf[TextAttribute]) {
          return Some(attr2.asInstanceOf[TextAttribute])
        }
      }
      None
    }
    fn findQuoteText() -> Option<String> {
      let attributesOnEntity2: Array[(i64, Attribute)] = getCachedAttributes(uriEntity, cachedAttrsIn);
      let quoteClassTemplateId: i64 = new EntityClass(uriEntity.m_db, quoteClassIdIn).get_template_entity_id;
      for (attrTuple <- attributesOnEntity2) {
        let attr2: Attribute = attrTuple._2;
        if attr2.get_attr_type_id() == quoteClassTemplateId && attr2.isInstanceOf[TextAttribute]) {
          return Some(attr2.asInstanceOf[TextAttribute].get_text)
        }
      }
      None
    }
    let uriAttribute: Option[TextAttribute] = findUriAttribute();
    if uriAttribute.isEmpty) {
      throw new OmException("Unable to find TextAttribute of type URI (classId=" + uriClassIdIn + ") for entity " + uriEntity.get_id)
    }
    // this one can be None and it's no surprise:
    let quoteText: Option<String> = findQuoteText();
    printHtmlListItemWithLink(printWriter, "", uriAttribute.get.get_text, uriEntity.get_name, None, quoteText)
  }

    fn printListItemForEntity(printWriterIn: PrintWriter, relationTypeIn: RelationType, entity_in: Entity) -> /*Unit%%*/ {
    let numSubEntries = getNumSubEntries(entity_in);
    if numSubEntries > 0) {
      let relatedEntitysFileNamePrefix: String = getExportFileNamePrefix(entity_in, ImportExport.HTML_EXPORT_TYPE);
      printHtmlListItemWithLink(printWriterIn,
                                if relationTypeIn.get_name == Database.THE_HAS_RELATION_TYPE_NAME) "" else relationTypeIn.get_name + ": ",
                                relatedEntitysFileNamePrefix + ".html",
                                entity_in.get_name)
                                //removing next line until it matches better with what user can actually see: currently includes non-public stuff, so the #
                                //might confuse a reader, or at least doesn't set fulfillable expectations on how much content there is.
//                                Some("(" + numSubEntries + ")"))
    } else {
      let line = (if relationTypeIn.get_name == Database.THE_HAS_RELATION_TYPE_NAME) "" else relationTypeIn.get_name + ": ") +;
                 entity_in.get_name
      printWriterIn.println("<li>" + htmlEncode(line) + "</li>")
    }
  }

  * This method exists (as opposed to including the logic inside exportToHtmlFile) because there was a bug.  Here I try explaining:
    *   - the parm levels_remainingToExportIn limits how far in the hierarchy (distance from the root entity of the export) the export will include (or descend).
    *   - at some "deep" point in the hierarchy, an entity X might be exported, but not its children, because X was at the depth limit.
    *   - X might also be found elsewhere, "shallower" in the hierarchy, but having been exported before (at the deep point), it is not now exported again.
    *   - Therefore X's children should have been exported from the "shallow" point, because they are now less than levels_remainingToExportIn levels deep, but
    *     were not exported because X was skipped (having been already been done).
    *   - Therefore separating the logic for the children allows them to be exported anyway, which fixes the bug.
    *   - Idea: does this same issue happen with exporting as text?  Does it need to be fixed there too?  See other cmts made in same commit..?
    *
    * Still, within this method it is also necessary to avoid infinitely looping around entities who contain references to (eventually) themselves, which
    * is the purpose of the variable "entitiesAlreadyProcessedInThisRefChain".
    *
    * If parameter levelsToProcessIsInfiniteIn is true, then levels_remainingToProcessIn is irrelevant.
    *
    fn exportItsChildrenToHtmlFiles(entity_in: Entity, levelsToExportIsInfiniteIn: bool, levels_remainingToExportIn: Int,
                                   outputDirectoryIn: Path,
                                   //in this method, next parm is only used to pass along in calls to exportHtml
                                   //(idea: check: true? See also usage of entitiesAlreadyProcessedInThisRefChainIn just below, as part of ck?)
                                   exportedEntityIdsIn: mutable.HashMap[String, Integer],
                                   cachedEntitiesIn: mutable.HashMap[String, Entity],
                                   cachedAttrsIn: mutable.HashMap[i64, Array[(i64, Attribute)]], cachedGroupInfoIn: mutable.HashMap[i64, Array[i64]],
                                   entitiesAlreadyProcessedInThisRefChainIn: mutable.TreeSet[i64], uriClassIdIn: i64, quoteClassId: i64,
                                   includePublicDataIn: bool, includeNonPublicDataIn: bool, includeUnspecifiedDataIn: bool,
                                   headerContentIn: Option<String>, beginBodyContentIn: Option<String>, copyrightYearAndNameIn: Option<String>) {
    if !levelsRemainAndPublicEnough(entity_in, includePublicDataIn, includeNonPublicDataIn, includeUnspecifiedDataIn,
                                     levelsToExportIsInfiniteIn, levels_remainingToExportIn)) {
      return
    }
    // (See comment at similar location in exportEntityToHtmlFile about the use of the next line, compared to the check a couple of lines above.)
    if entitiesAlreadyProcessedInThisRefChainIn.contains(entity_in.get_id)) {
      // (Breakpoints do hit this line when I export my personal site (at least with 40 levels and including entries marked neither public nor non-public).)
      return
    }

    entitiesAlreadyProcessedInThisRefChainIn.add(entity_in.get_id)
    let attrTuples: Array[(i64, Attribute)] = getCachedAttributes(entity_in, cachedAttrsIn);
    for (attributeTuple <- attrTuples) {
      let attribute: Attribute = attributeTuple._2;
      attribute match {
        case relation: RelationToLocalEntity =>
          let entity2: Entity = getCachedEntity(relation.getRelatedId2, cachedEntitiesIn, relation.m_db);
          if entity2.getClassId.isEmpty || entity2.getClassId.get != uriClassIdIn) {
            // that means it's not a URI but an actual traversable thing to follow when exporting children:
            exportHtml(entity2, levelsToExportIsInfiniteIn, levels_remainingToExportIn - 1,
                       outputDirectoryIn, exportedEntityIdsIn, cachedEntitiesIn,
                       cachedAttrsIn, cachedGroupInfoIn, entitiesAlreadyProcessedInThisRefChainIn, uriClassIdIn, quoteClassId,
                       includePublicDataIn, includeNonPublicDataIn, includeUnspecifiedDataIn,
                       headerContentIn, beginBodyContentIn, copyrightYearAndNameIn)
          }
        case relation: RelationToRemoteEntity =>
          // Idea: The next line doesn't currently internally do caching for DBs like we do for entities in getCachedEntity, but that could be added if it is
          // used often enough to be a performance problem (and at similar comment elsewhere in this file)
          // (AND THE SAME AT THE OTHER PLACES W/ SAME COMMENT.)
          let remoteDb = relation.getRemoteDatabase;
          let entity2: Entity = getCachedEntity(relation.getRelatedId2, cachedEntitiesIn, remoteDb);
          // The classId and uriClassIdIn probably won't match because entity2 n all its data comes from a different (remote) db, so not checking that, at
          // least until that sort of cross-db check is supported, so skipping this condition for now (as elsewhere):
//          if entity2.getClassId.isEmpty || entity2.getClassId.get != uriClassIdIn) {
//            // that means it's not a URI but an actual traversable thing to follow when exporting children:
            exportHtml(entity2, levelsToExportIsInfiniteIn, levels_remainingToExportIn - 1,
                       outputDirectoryIn, exportedEntityIdsIn, cachedEntitiesIn,
                       cachedAttrsIn, cachedGroupInfoIn, entitiesAlreadyProcessedInThisRefChainIn, uriClassIdIn, quoteClassId,
                       includePublicDataIn, includeNonPublicDataIn, includeUnspecifiedDataIn,
                       headerContentIn, beginBodyContentIn, copyrightYearAndNameIn)
//          }
        case relation: RelationToGroup =>
          let entityIds: Array[i64] = getCachedGroupData(relation, cachedGroupInfoIn);
          for (entityIdInGrp <- entityIds) {
            let entity_in_group: Entity = getCachedEntity(entityIdInGrp, cachedEntitiesIn, relation.m_db);
            exportHtml(entity_in_group, levelsToExportIsInfiniteIn, levels_remainingToExportIn - 1,
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
    entitiesAlreadyProcessedInThisRefChainIn.remove(entity_in.get_id)
  }

    fn getCachedGroupData(rtg: RelationToGroup, cachedGroupInfoIn: mutable.HashMap[i64, Array[i64]]) -> Array[i64] {
    let cachedIds: Option[Array[i64]] = cachedGroupInfoIn.get(rtg.getGroupId);
    if cachedIds.is_defined) {
      cachedIds.get
    } else {
      let data: Vec<Vec<Option<DataType>>> = rtg.m_db.get_group_entries_data(rtg.getGroupId, None, include_archived_entities_in = false);
      let entityIds = new Array[i64](data.size);
      let mut count = 0;
      for (entry <- data) {
        let entityIdInGroup: i64 = entry(0).get.asInstanceOf[i64];
        entityIds(count) = entityIdInGroup
        count += 1
      }
      cachedGroupInfoIn.put(rtg.getGroupId, entityIds)
      entityIds
    }
  }

    fn getCachedAttributes(entity_in: Entity, cachedAttrsIn: mutable.HashMap[i64, Array[(i64, Attribute)]]) -> Array[(i64, Attribute)] {
    let cachedInfo: Option[Array[(i64, Attribute)]] = cachedAttrsIn.get(entity_in.get_id);
    if cachedInfo.is_defined) {
      cachedInfo.get
    } else {
      let attrTuples = entity_in.get_sorted_attributes(0, 0, only_public_entities_in = false)._1;
      // record, so we don't create files more than once, calculate attributes more than once, etc.
      cachedAttrsIn.put(entity_in.get_id, attrTuples)
      attrTuples
    }
  }

    fn getCachedEntity(entityIdIn: i64, cachedEntitiesIn: mutable.HashMap[String, Entity], dbIn: Database) -> Entity = {
    let key: String = dbIn.id + entityIdIn.toString;
    let cachedInfo: Option<Entity> = cachedEntitiesIn.get(key);
    if cachedInfo.is_defined) {
      cachedInfo.get
    } else {
      let entity = new Entity(dbIn, entityIdIn);
      cachedEntitiesIn.put(key, entity)
      entity
    }
  }

  * Very basic for now. Noted in task list to do more, under i18n and under "do a better job of encoding"
    *
    fn htmlEncode(in: String) -> String {
    let mut out = in.replace("&", "&amp;");
    out = out.replace(">", "&gt;")
    out = out.replace("<", "&lt;")
    out = out.replace("\"", "&quot;")
    out
  }

    fn getLineNumbers(includeOutlineNumbering: bool = true, currentIndentationLevels: Int, nextKnownOutlineNumbers: java.util.ArrayList[Int]) -> String {
    // (just a check, to learn. Maybe there is a better spot for it)
    //test fails with it, as does item noted in my om todos??:
    // require(currentIndentationLevels == nextKnownOutlineNumbers.size)

    let s = new StringBuffer;
    if includeOutlineNumbering && nextKnownOutlineNumbers.size > 0) {
      // (if nextKnownOutlineNumbersIn.size == 0, it is the first line/entity in the exported file, ie, just the
      // containing entity or heading for the rest, so nothing to do.
      for (i <- 0 until nextKnownOutlineNumbers.size) {
        s.append(nextKnownOutlineNumbers.get(i))
        if nextKnownOutlineNumbers.size() - 1 > i) s.append(".")
      }
    }
    s.toString
  }

  //@tailrec  THIS IS NOT TO BE TAIL RECURSIVE UNTIL IT'S KNOWN HOW TO MAKE SOME CALLS to it BE recursive, AND SOME *NOT* TAIL RECURSIVE (because some of them
  // *do* need to return & finish their work, such as when iterating through the entities & subgroups)! (but test it: is it really a problem?)
  // (Idea: See note at the top of Controller.chooseOrCreateObject re inAttrType about similarly making exportTypeIn an enum.)
  *
    * If levelsToProcessIsInfiniteIn is true, then levels_remainingToProcessIn is irrelevant.
    *
    * @return  Whether lines were wrapped--so a later call to it can decide whether to print a leading blank line.
    *
    fn exportToSingleTextFile(entity_in: Entity, levelsToExportIsInfiniteIn: bool, levels_remainingToExportIn: Int, currentIndentationLevelsIn: Int,
                             printWriterIn: PrintWriter,
                             includeMetadataIn: bool, exportedEntityIdsIn: mutable.HashMap[String, Integer],
                             cachedEntitiesIn: mutable.HashMap[String, Entity],
                             cachedAttrsIn: mutable.HashMap[i64, Array[(i64, Attribute)]], spacesPerIndentLevelIn: Int,
                             //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) w/in this method!
                             includePublicDataIn: bool, includeNonPublicDataIn: bool, includeUnspecifiedDataIn: bool,
                             wrapLongLinesIn: bool = false, wrapColumnIn: Int = 80, includeOutlineNumberingIn: bool = true,
                             outlineNumbersTrackingInOut: java.util.ArrayList[Int] = new java.util.ArrayList[Int],
                             previousEntityWasWrappedIn: bool = false) -> bool {
    // useful while debugging, but maybe can also put that in the expression evaluator (^U)
    //printWriterIn.flush()

    let mut previousEntityWasWrapped = previousEntityWasWrappedIn;
    let isFirstEntryOfAll: bool = outlineNumbersTrackingInOut.size == 0;

    fn incrementOutlineNumbering() /*-> Unit%%*/ {
      // Don't do on the first entry: because that is just the header and
      // shouldn't have a number, and the outlineNumbersTrackingInOut info
      // isn't there to increment so it would fail anyway:
      if !isFirstEntryOfAll) {
        let lastIndex = outlineNumbersTrackingInOut.size() - 1;
        let incrementedLastNumber = outlineNumbersTrackingInOut.get(lastIndex) + 1;
        outlineNumbersTrackingInOut.set(lastIndex, incrementedLastNumber)
      }
    }

    * Does optional line wrapping and spacing for readability.
      * @param printWriterIn  The destination to print to.
      * @param entryText  The text to print, like an entity name.
      * @return  Whether lines were wrapped--so a later call to it can decide whether to print a leading blank line.
      *
    fn printEntry(printWriterIn: PrintWriter, entryText: String) -> bool {
      // (Idea:  this method feels overcomplicated.  Maybe some sub-methods could be broken out or the logic made
      // consistent but simpler.  I do use the features though, for how outlines are spaced etc., and it has been well-tested.)
      let indentingSpaces: String = {;
        let adjustedCurrentIndentationLevelsIn = {;
          if wrapLongLinesIn) {
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
      let lineNumbers: String = getLineNumbers(includeOutlineNumberingIn, currentIndentationLevelsIn, outlineNumbersTrackingInOut);
      let mut numCharactersBeforeActualContent = indentingSpaces.length + lineNumbers.length;
      let mut stillToPrint: String = indentingSpaces + lineNumbers;
      if lineNumbers.length > 0 ) {
        stillToPrint = stillToPrint + " "
        numCharactersBeforeActualContent += 1
      }
      let wrappingThisEntrysLines: bool = wrapLongLinesIn && (stillToPrint.length + entryText.length) > wrapColumnIn;


      if includeOutlineNumberingIn) {
        // Just do the more complicated/optimized whitespace additions if adding outline numbers,
        // because only there is it trying to conserve vertical space (for now), with the numbers
        // helping readability to compensate for less vertical whitespace in some places.  This might let
        // exported content print on fewer sheets and require less page-turning.
        if wrappingThisEntrysLines && !previousEntityWasWrapped) {
          // In this case we just had a single-line entry (which don't always have a blank line after),
          // now being followed by a wrapped (multi-line) one,
          // and it makes it easier to read if there is also a preceding blank line *before* a wrapped block.
          stillToPrint = Util::NEWLN + stillToPrint + entryText
        } else {
          stillToPrint = stillToPrint + entryText
        }
      } else {
        stillToPrint = stillToPrint + entryText
      }

      if ! wrappingThisEntrysLines) {
        // print the one line, no need to wrap.
        // (No extra trailing NEWLN needed for readability if printing unwrapped lines, for example,
        // if includeOutlineNumberingIn == true), or if doing just a basic export without readability
        // enhancements (because of tests' assumptions about size, and no need.)
        printWriterIn.println(stillToPrint)
      } else {
        while (stillToPrint.length > 0) {
          // figure out how much to print, out of a long line
          //("wrapColumnIn - 1", is there to still respect the limit (wrapColumnIn) given that we do
          // + 1 afterward to include the trailing space.)
          let lastSpaceIndex = stillToPrint.lastIndexOf(" ", wrapColumnIn - 1);
          let endLineIndex =;
            if lastSpaceIndex > numCharactersBeforeActualContent && stillToPrint.length > wrapColumnIn) {
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
          if stillToPrint.substring(endLineIndex).length > 0) {
            stillToPrint = indentingSpaces + stillToPrint.substring(endLineIndex)
          } else {
            stillToPrint = stillToPrint.substring(endLineIndex)
            // in other words, done with the content:
            assert(stillToPrint.length == 0)
           }
        }
      }
      if isFirstEntryOfAll && wrapLongLinesIn) {
        // Just a readability convenience: underline the very top entry (since its children
        // are not indented under it--to set it off visually as something like a "title").
        let length = Math.min(wrapColumnIn, entryText.length);
        // (Compare use of "entryText.lastIndexOf(..."  with the "val lastSpaceIndex = " line elsewhere.
        let underline: StringBuffer = new StringBuffer(wrapColumnIn);
        for (_ <- 1 to length) {
          underline.append("-")
        }
        printWriterIn.println(underline)
      }
      if wrappingThisEntrysLines || (wrapLongLinesIn && !includeOutlineNumberingIn)) {
        // *WHEN MAINTAINING HERE, MAINTAIN SIMILARLY BOTH PLACES THAT SAY "whitespace for readability" in comment.*
        // whitespace for readability
        printWriterIn.println()
      }
      // Return whether we *did* wrap lines:
      wrappingThisEntrysLines
    }



    let entityName = entity_in.get_name;
    if exportedEntityIdsIn.contains(entity_in.uniqueIdentifier)) {
      // it is a duplicate of something already exported, so just print a stub.
      let infoToPrint = if includeMetadataIn) {;
        "(duplicate: EN --> " + entity_in.get_id + ": " + entityName + ")"
      } else {
        entityName
      }
      previousEntityWasWrapped = printEntry(printWriterIn, infoToPrint)
    } else {
      if levelsRemainAndPublicEnough(entity_in, includePublicDataIn, includeNonPublicDataIn, includeUnspecifiedDataIn,
                                      levelsToExportIsInfiniteIn, levels_remainingToExportIn)) {
        //add it, so we don't create duplicate entries:
        // (NOTE: the -1 is not being used for now, in text file exports)
        exportedEntityIdsIn.update(entity_in.uniqueIdentifier, -1)

        let infoToPrint = if includeMetadataIn) {;
          "EN " + entity_in.get_id + ": " + entity_in.get_display_string()
        } else {
          entityName
        }

        previousEntityWasWrapped =
          printEntry(printWriterIn, infoToPrint)

        let attrTuples: Array[(i64, Attribute)] = getCachedAttributes(entity_in, cachedAttrsIn);
        outlineNumbersTrackingInOut.add(0)
        for (attributeTuple <- attrTuples) {
          let attribute:Attribute = attributeTuple._2;
          attribute match {
            case relation: RelationToLocalEntity =>
              let relationType = new RelationType(relation.m_db, relation.get_attr_type_id());
              let entity2 = new Entity(relation.m_db, relation.getRelatedId2);
              if includeMetadataIn) {
                printWriterIn.print(getSpaces((currentIndentationLevelsIn + 1) * spacesPerIndentLevelIn))
                printWriterIn.println(attribute.get_display_string(0, Some(entity2), Some(relationType)))
              }
              // Idea: write tests to confirm that printing metadata as just above and the entity as just below, will all
              // work together with features such as wrapping, entities containing entities directly rather than via groups,
              // duplicate entities tracked via exportedEntityIdsIn, and all other attr types & variations on the parameters
              // to exportToSingleTxtFile.  Or wait for a need.
              previousEntityWasWrapped = exportToSingleTextFile(entity2, levelsToExportIsInfiniteIn, levels_remainingToExportIn - 1,
                                                                currentIndentationLevelsIn + 1, printWriterIn,
                                                                includeMetadataIn, exportedEntityIdsIn, cachedEntitiesIn, cachedAttrsIn, spacesPerIndentLevelIn,
                                                                includePublicDataIn, includeNonPublicDataIn, includeUnspecifiedDataIn,
                                                                wrapLongLinesIn, wrapColumnIn, includeOutlineNumberingIn, outlineNumbersTrackingInOut,
                                                                previousEntityWasWrapped)
            case relation: RelationToRemoteEntity =>
              let relationType = new RelationType(relation.m_db, relation.get_attr_type_id());
              let remoteDb: Database = relation.getRemoteDatabase;
              let entity2 = new Entity(remoteDb, relation.getRelatedId2);
              if includeMetadataIn) {
                printWriterIn.print(getSpaces((currentIndentationLevelsIn + 1) * spacesPerIndentLevelIn))
                printWriterIn.println(attribute.get_display_string(0, Some(entity2), Some(relationType)))
              }
              previousEntityWasWrapped = exportToSingleTextFile(entity2, levelsToExportIsInfiniteIn, levels_remainingToExportIn - 1,
                                                                currentIndentationLevelsIn + 1, printWriterIn,
                                                                includeMetadataIn, exportedEntityIdsIn, cachedEntitiesIn, cachedAttrsIn, spacesPerIndentLevelIn,
                                                                includePublicDataIn, includeNonPublicDataIn, includeUnspecifiedDataIn,
                                                                wrapLongLinesIn, wrapColumnIn, includeOutlineNumberingIn, outlineNumbersTrackingInOut,
                                                                previousEntityWasWrapped)
            case relation: RelationToGroup =>
              let relationType = new RelationType(relation.m_db, relation.get_attr_type_id());
              let group = new Group(relation.m_db, relation.getGroupId);
              let grpName = group.get_name;
              // if a group name is different from its entity name, indicate the differing group name also, otherwise complete the line just above w/ NL
              if entityName != grpName) {
                printWriterIn.print(getSpaces((currentIndentationLevelsIn + 1) * spacesPerIndentLevelIn))
                printWriterIn.println("(" + relationType.get_name + " group named: " + grpName + ")")
              }
              if includeMetadataIn) {
                printWriterIn.print(getSpaces(currentIndentationLevelsIn * spacesPerIndentLevelIn))
                // plus one more level of spaces to make it look better but still ~equivalently/exchangeably importable:
                printWriterIn.print(getSpaces(spacesPerIndentLevelIn))
                printWriterIn.println("(group details: " + attribute.get_display_string(0, None, Some(relationType)) + ")")
              }
              for (entity_in_group: Entity <- group.getGroupEntries(0).toArray(Array[Entity]())) {
                previousEntityWasWrapped = exportToSingleTextFile(entity_in_group, levelsToExportIsInfiniteIn, levels_remainingToExportIn - 1,
                                                                  currentIndentationLevelsIn + 1, printWriterIn, includeMetadataIn,
                                                                  exportedEntityIdsIn, cachedEntitiesIn, cachedAttrsIn, spacesPerIndentLevelIn,
                                                                  includePublicDataIn, includeNonPublicDataIn, includeUnspecifiedDataIn,
                                                                  wrapLongLinesIn, wrapColumnIn, includeOutlineNumberingIn, outlineNumbersTrackingInOut,
                                                                  previousEntityWasWrapped)
              }
            case _ =>
              incrementOutlineNumbering()
              //idea?: print as a simple prefix the getLineNumbers content as done elsewhere in this file.  How does it look? Then stop enhancing until used?
              printWriterIn.print(getLineNumbers(includeOutlineNumberingIn, currentIndentationLevelsIn, outlineNumbersTrackingInOut))
              printWriterIn.print(getSpaces((currentIndentationLevelsIn + 1) * spacesPerIndentLevelIn))
              if includeMetadataIn) {
                printWriterIn.println((attribute match {
                  case ba: BooleanAttribute => "BA "
                  case da: DateAttribute => "DA "
                  case fa: FileAttribute => "FA "
                  case qa: QuantityAttribute => "QA "
                  case ta: TextAttribute => "TA "
                }) + ": " + attribute.get_display_string(0, None, None))
              } else {
                printWriterIn.println(attribute.get_display_string(0, None, None, simplify = true))
              }
              if wrapLongLinesIn && !includeOutlineNumberingIn) {
                // *WHEN MAINTAINING HERE, MAINTAIN SIMILARLY BOTH PLACES THAT SAY "whitespace for readability" in comment.*
                // whitespace for readability, similarly to what is done in printEntry
                printWriterIn.println()
              }
          }
        }
        outlineNumbersTrackingInOut.remove(outlineNumbersTrackingInOut.size() - 1)
      }
    }
    return previousEntityWasWrapped
  }

    fn levelsRemainAndPublicEnough(entity_in: Entity, includePublicDataIn: bool, includeNonPublicDataIn: bool,
                                  includeUnspecifiedDataIn: bool, levelsToExportIsInfiniteIn: bool, levels_remainingToExportIn: Int) -> bool {
    if !levelsToExportIsInfiniteIn && levels_remainingToExportIn == 0) {
      return false
    }
    let entityPublicStatus: Option<bool> = entity_in.getPublic;
    let publicEnoughToExport = (entityPublicStatus.is_defined && entityPublicStatus.get && includePublicDataIn) ||;
                          (entityPublicStatus.is_defined && !entityPublicStatus.get && includeNonPublicDataIn) ||
                          (entityPublicStatus.isEmpty && includeUnspecifiedDataIn)
    publicEnoughToExport
  }

    fn printHtmlListItemWithLink(printWriterIn: PrintWriter, preLabel: String, uri: String, linkDisplayText: String, suffix: Option<String> = None,
                                textOnNextLineButSameHtmlListItem: Option<String> = None) /* -> Unit%%*/ {
    printWriterIn.print("<li>")
    printWriterIn.print(htmlEncode(preLabel) + "<a href=\"" + uri + "\">" + htmlEncode(linkDisplayText) + "</a>" + " " + htmlEncode(suffix.getOrElse("")))
    if textOnNextLineButSameHtmlListItem.is_defined) printWriterIn.print("<br><pre>\"" + htmlEncode(textOnNextLineButSameHtmlListItem.get) + "\"</pre>")
    printWriterIn.println("</li>")
  }

    fn getNumSubEntries(entity_in: Entity) -> i64 {
    let numSubEntries = {;
      let numAttrs = entity_in.get_attribute_count();
      if numAttrs == 1) {
        let (_, _, groupId, _, moreThanOneAvailable) = entity_in.findRelationToAndGroup;
        if groupId.is_defined && !moreThanOneAvailable) {
          entity_in.m_db.getGroupSize(groupId.get, 4)
        } else numAttrs
      } else numAttrs
    }
    numSubEntries
  }

    fn getSpaces(num: Int) -> String {
    let s: StringBuffer = new StringBuffer;
    for (i <- 1 to num) {
      s.append(" ")
    }
    s.toString
  }

    fn getExportFileNamePrefix(entity: Entity, exportTypeIn: String) -> String {
    let entityIdentifier: String = {;
      if entity.m_db.is_remote) {
        require(entity.m_db.get_remote_address.is_defined)
        "remote-" + entity.readableIdentifier
      } else {
        entity.get_id.toString
      }
    }
    if exportTypeIn == ImportExport.HTML_EXPORT_TYPE) {
      // (The 'e' is for "entity"; for explanation see cmts in methods createOutputDir and createOutputFile.)
      "e" + entityIdentifier
    } else {
      //idea (also in task list): change this to be a reliable filename (incl no backslashes? limit it to a whitelist of chars? a simple fn for that?
      let mut fixedEntityName = entity.get_name.replace(" ", "");
      fixedEntityName = fixedEntityName.replace("/", "-")
      //fixedEntityName = fixedEntityName.replace("\\","-")
      "onemodel-export_" + entityIdentifier + "_" + fixedEntityName + "-"
    }
  }

    fn createOutputDir(prefix: String) -> Path {
    // even though entityIds start with a '-', it's a problem if a filename does (eg, "ls" cmd thinks it is an option, not a name):
    // (there's a similar line elsewhere)
    require(!prefix.startsWith("-"))
    // hyphen after the prefix is in case one wants to see where the id ends & the temporary/generated name begins, for understanding/diagnosing things:
    Files.createTempDirectory(prefix + "-")
  }

    fn createOutputFile(prefix:String, exportTypeIn: String, exportDirectory: Option[Path]) -> (File, PrintWriter) {
    // even though entityIds start with a '-', it's a problem if a filename does (eg, "ls" cmd thinks it is an option, not a name):
    // (there's a similar line elsewhere)
    require(!prefix.startsWith("-"));

    // make sure we have a place to put all the html files, together:
    if exportTypeIn == ImportExport.HTML_EXPORT_TYPE) require(exportDirectory.is_defined && exportDirectory.get.toFile.isDirectory);

    let extension: String = {;
      if exportTypeIn == ImportExport.TEXT_EXPORT_TYPE) ".txt"
      else if exportTypeIn == ImportExport.HTML_EXPORT_TYPE) ".html"
      else throw new OmException("unexpected exportTypeIn: " + exportTypeIn)
    }

    let outputFile: File =;
      if exportTypeIn == ImportExport.HTML_EXPORT_TYPE ) {
        Files.createFile(new File(exportDirectory.get.toFile, prefix + extension).toPath).toFile
      } else if exportTypeIn == ImportExport.TEXT_EXPORT_TYPE) {
        Files.createTempFile(prefix, extension).toFile
      }
      else throw new OmException("unexpected exportTypeIn: " + exportTypeIn)

    let output: PrintWriter = new PrintWriter(new BufferedWriter(new FileWriter(outputFile)));
    (outputFile, output)
  }

  // these methods are in this class so it can be found by both PostgreSQLDatabaseTest and ImportExportTest (not sure why it couldn't be found
  // by PostgreSQLDatabaseTest when it was in ImportExportTest).
    fn tryImporting_FOR_TESTS(filename_in: String, entity_in: Entity) -> File {
    //PROBLEM: these 2 lines make it so it's hard to test in the IDE without first building a .jar since it finds the file in the jar. How fix?
    let stream = this.getClass.getClassLoader.getResourceAsStream(filename_in);
    let reader: java.io.Reader = new java.io.InputStreamReader(stream);

    // manual testing alternative to the above 2 lines, such as for use w/ interactive scala (REPL):
    //val path = "PUT-Full-path-to-some-text-file-here"
    //val fileToImport = new File(path)
    //val reader = new FileReader(fileToImport)

    doTheImport(reader, "name", 0L, entity_in, creatingNewStartingGroupFromTheFilename_in = false, addingToExistingGroup = false,
                putEntriesAtEnd = true, mixedClassesAllowedDefaultIn = true, testing = true, makeThem_publicIn = Some(false))

    // write it out for later comparison:
    let stream2 = this.getClass.getClassLoader.getResourceAsStream(filename_in);
    let tmpCopy: Path = Files.createTempFile(null, null);
    Files.copy(stream2, tmpCopy, StandardCopyOption.REPLACE_EXISTING)
    tmpCopy.toFile
  }
  // (see cmt on tryImporting method)
    fn tryExportingTxt_FOR_TESTS(ids: java.util.ArrayList[i64], dbIn: Database, wrapLongLinesIn: bool = false,
                                wrapColumnIn: Int = 80, includeOutlineNumberingIn: bool = false) -> (String, File) {
    assert(ids.size > 0)
    let entityId: i64 = ids.get(0);
    let startingEntity: Entity = new Entity(dbIn, entityId);

    // see comments in ImportExport.export() method for explanation of these 3
    let exportedEntityIds = new mutable.HashMap[String, Integer];
    let cachedEntities = new mutable.HashMap[String, Entity];
    let cachedAttrs = new mutable.HashMap[i64, Array[(i64, Attribute)]];

    let prefix: String = getExportFileNamePrefix(startingEntity, ImportExport.TEXT_EXPORT_TYPE);
    let (outputFile: File, outputWriter: PrintWriter) = createOutputFile(prefix, ImportExport.TEXT_EXPORT_TYPE, None);
    exportToSingleTextFile(startingEntity, levelsToExportIsInfiniteIn = true, 0, 0, outputWriter, includeMetadataIn = false, exportedEntityIds, cachedEntities,
                          cachedAttrs, 2, includePublicDataIn = true, includeNonPublicDataIn = true, includeUnspecifiedDataIn = true,
                          wrapLongLinesIn, wrapColumnIn, includeOutlineNumberingIn)
    assert(outputFile.exists)
    outputWriter.close()
    let firstNewFileContents: String = new Predef.String(Files.readAllBytes(outputFile.toPath));
    (firstNewFileContents, outputFile)
  }

*/
}
