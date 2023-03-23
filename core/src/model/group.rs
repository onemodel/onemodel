/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2014-2017 inclusive, and 2023, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
struct Group {
/*%%
package org.onemodel.core.model

import org.onemodel.core.{Util, Color, OmException}

object Group {
    fn create_group(db_in: Database, inName: String, allow_mixed_classes_in_group_in: bool = false) -> Group {
    let id: i64 = db_in.create_group(inName, allow_mixed_classes_in_group_in);
    new Group(db_in, id)
  }

  /** This is for times when you want None if it doesn't exist, instead of the exception thrown by the Entity constructor.  Or for convenience in tests.
    */
    fn getGroup(db_in: Database, id: i64) -> Option[Group] {
    try Some(new Group(db_in, id))
    catch {
      case e: java.lang.Exception =>
        //idea: see comment here in Entity.scala.
        if e.toString.indexOf(Util::DOES_NOT_EXIST) >= 0) {
          None
        }
        else throw e
    }
  }
}

/** See comments on similar methods in RelationToEntity (or maybe its subclasses).
  *
  * Groups don't contain remote entities (only those at the same DB as the group is), so some logic doesn't have to be written for that.
  * */
class Group(val m_db: Database, m_id: i64) {
  // (See comment in similar spot in BooleanAttribute for why not checking for exists, if m_db.is_remote.)
  if !m_db.is_remote && !m_db.groupKeyExists(m_id: i64)) {
    throw new Exception("Key " + m_id + Util::DOES_NOT_EXIST)
  }

  /** See comment about these 2 dates in Database.create_tables() */
    fn this(m_db: Database, id_in: i64, name_in: String, insertion_dateIn: i64, mixed_classes_allowedIn: bool, newEntriesStickToTopIn: bool) {
    this(m_db, id_in)
    m_name = name_in
    m_insertion_date = insertion_dateIn
    mMixedClassesAllowed = mixed_classes_allowedIn
    m_new_entries_stick_to_top = newEntriesStickToTopIn
    m_already_read_data = true
  }

    fn read_data_from_db() {
    let relationData: Array[Option[Any]] = m_db.getGroupData(m_id);
    if relationData.length == 0) {
      throw new OmException("No results returned from data request for: " + m_id)
    }
    m_name = relationData(0).get.asInstanceOf[String]
    m_insertion_date = relationData(1).get.asInstanceOf[i64]
    mMixedClassesAllowed = relationData(2).get.asInstanceOf[Boolean]
    m_new_entries_stick_to_top = relationData(3).get.asInstanceOf[Boolean]
    m_already_read_data = true
  }

    fn update(attr_type_id_inIGNOREDFORSOMEREASON: Option<i64> = None, name_in: Option<String> = None, allow_mixed_classes_in_group_in: Option<bool> = None,
             newEntriesStickToTopIn: Option<bool> = None,
             valid_on_date_inIGNORED4NOW: Option<i64>, observation_date_inIGNORED4NOW: Option<i64>) {

    m_db.updateGroup(m_id,
                    if name_in.isEmpty) get_name else name_in.get,
                    if allow_mixed_classes_in_group_in.isEmpty) getMixedClassesAllowed else allow_mixed_classes_in_group_in.get,
                    if newEntriesStickToTopIn.isEmpty) getNewEntriesStickToTop else newEntriesStickToTopIn.get)

    if name_in.is_some()) m_name = name_in.get
    if allow_mixed_classes_in_group_in.is_some()) mMixedClassesAllowed = allow_mixed_classes_in_group_in.get
    if newEntriesStickToTopIn.is_some()) m_new_entries_stick_to_top = newEntriesStickToTopIn.get
  }

  /** Removes this object from the system. */
    fn delete() {
    m_db.deleteGroupAndRelationsToIt(m_id)
    }

  /** Removes an entity from this group. */
    fn removeEntity(entity_id: i64) {
    m_db.removeEntityFromGroup(m_id, entity_id)
    }

    fn deleteWithEntities() {
    m_db.deleteGroupRelationsToItAndItsEntries(m_id)
    }

  // idea: cache this?  when doing any other query also?  Is that safer because we really don't edit these in place (ie, immutability, or vals not vars)?
    fn getSize(includeWhichEntities: Int = 3) -> i64 {
    m_db.get_group_size(m_id, includeWhichEntities)
  }

    fn get_display_string(lengthLimitIn: Int = 0, simplifyIn: bool = false) -> String {
    let numEntries = m_db.get_group_size(get_id, 1);
    let mut result: String =  "";
    result += {
      if simplifyIn) get_name
      else "grp " + m_id + " /" + numEntries + ": " + Color.blue(get_name)
    }
    if !simplifyIn) {
      result += ", class: "
      let class_name =;
        if getMixedClassesAllowed)
          "(mixed)"
        else {
          let class_nameOption = getClassName;
          if class_nameOption.isEmpty) "None"
          else class_nameOption.get
        }
      result += class_name
    }
    if simplifyIn) result
    else Attribute.limitDescriptionLength(result, lengthLimitIn)
  }

    fn getGroupEntries(startingIndexIn: i64, maxValsIn: Option<i64> = None) -> Vec<Entity> {
    m_db.getGroupEntryObjects(m_id, startingIndexIn, maxValsIn)
  }

    fn addEntity(inEntityId: i64, sorting_index_in: Option<i64> = None, caller_manages_transactions_in: bool = false) {
    m_db.add_entity_to_group(get_id, inEntityId, sorting_index_in, caller_manages_transactions_in)
  }

    fn get_id() -> i64 {
    m_id
    }

    fn get_name -> String {
    if !m_already_read_data) read_data_from_db()
    m_name
  }

    fn getMixedClassesAllowed -> bool {
    if !m_already_read_data) read_data_from_db()
    mMixedClassesAllowed
  }

    fn getNewEntriesStickToTop -> bool {
    if !m_already_read_data) read_data_from_db()
    m_new_entries_stick_to_top
  }

    fn getInsertionDate -> i64 {
    if !m_already_read_data) read_data_from_db()
    m_insertion_date
  }

    fn getClassName -> Option<String> {
    if getMixedClassesAllowed)
      None
    else {
      let class_id: Option<i64> = getClassId;
      if class_id.isEmpty && getSize() == 0) {
        // display should indicate that we know mixed are not allowed, so a class could be specified, but none has.
        Some("(unspecified)")
      } else if class_id.isEmpty) {
        // means the group requires uniform classes, but the enforced uniform class is None, i.e., to not have a class:
        Some("(specified as None)")
      } else {
        let exampleEntitysClass = new EntityClass(m_db, class_id.get);
        Some(exampleEntitysClass.get_name)
      }
    }
  }

    fn getClassId -> Option<i64> {
    if getMixedClassesAllowed)
      None
    else {
      let entries = m_db.getGroupEntryObjects(get_id, 0, Some(1));
      let specified: bool = entries.size() > 0;
      if !specified)
        None
      else {
        // idea: eliminate/simplify most of this part, since groups can't have subgroups only entities in them now?
        fn findAnEntity(nextIndex: Int) -> Option<Entity> {
          // We will have to change this (and probably other things) to traverse "subgroups" (groups in the entities in this group) also,
          // if we decide that disallowing mixed classes also means class uniformity across all subgroups.
          if nextIndex == entries.size)
            None
          else entries.get(nextIndex) match {
            case entity: Entity =>
              Some(entity)
            case _ =>
              let class_name = entries.get(nextIndex).getClass.get_name;
              throw new OmException(s"a group contained an entry that's not an entity?  Thought had eliminated use of 'subgroups' except via entities. It's " +
                                    s"of type: $class_name")
          }
        }
        let entity: Option<Entity> = findAnEntity(0);
        if entity.is_some())
          entity.get.getClassId
        else
          None
      }
    }
  }

    fn getClassTemplateEntity -> (Option<Entity>) {
    let class_id: Option<i64> = getClassId;
    if getMixedClassesAllowed || class_id.isEmpty)
      None
    else {
      let templateEntityId = new EntityClass(m_db, class_id.get).getTemplateEntityId;
      Some(new Entity(m_db, templateEntityId))
    }
  }

    fn getHighestSortingIndex -> i64 {
    m_db.getHighestSortingIndexForGroup(get_id)
  }

    fn getContainingRelationsToGroup(startingIndexIn: i64, maxValsIn: Option<i64> = None) -> java.util.ArrayList[RelationToGroup] {
    m_db.getRelationsToGroupContainingThisGroup(get_id, startingIndexIn, maxValsIn)
  }

    fn getCountOfEntitiesContainingGroup -> (i64, i64) {
    m_db.getCountOfEntitiesContainingGroup(get_id)
  }

    fn getEntitiesContainingGroup(startingIndexIn: i64, maxValsIn: Option<i64> = None) -> java.util.ArrayList[(i64, Entity)] {
    m_db.getEntitiesContainingGroup(get_id, startingIndexIn, maxValsIn)
  }

    fn findUnusedSortingIndex(starting_with_in: Option<i64> = None) -> i64 {
    m_db.find_unused_group_sorting_index(get_id, starting_with_in)
  }

    fn getGroupsContainingEntitysGroupsIds(limitIn: Option<i64> = Some(5)) -> List[Array[Option[Any]]] {
    m_db.getGroupsContainingEntitysGroupsIds(get_id, limitIn)
  }

    fn isEntityInGroup(entity_id_in: i64) -> bool {
    m_db.isEntityInGroup(get_id, entity_id_in)
  }

    fn getAdjacentGroupEntriesSortingIndexes(sorting_index_in: i64, limitIn: Option<i64> = None, forwardNotBackIn: bool) -> List[Array[Option[Any]]] {
    m_db.getAdjacentGroupEntriesSortingIndexes(get_id, sorting_index_in, limitIn, forwardNotBackIn)
  }

    fn getNearestGroupEntrysSortingIndex(startingPointSortingIndexIn: i64, forwardNotBackIn: bool) -> Option<i64> {
    m_db.getNearestGroupEntrysSortingIndex(get_id, startingPointSortingIndexIn, forwardNotBackIn)
  }

    fn getEntrySortingIndex(entity_id_in: i64) -> i64 {
    m_db.getGroupEntrySortingIndex(get_id, entity_id_in)
  }

    fn is_group_entry_sorting_index_in_use(sorting_index_in: i64) -> bool {
    m_db.is_group_entry_sorting_index_in_use(get_id, sorting_index_in)
  }

    fn updateSortingIndex(entity_id_in: i64, sorting_index_in: i64) /*-> Unit%%*/ {
    m_db.updateSortingIndexInAGroup(get_id, entity_id_in, sorting_index_in)
  }

    fn renumberSortingIndexes(caller_manages_transactions_in: bool = false) /*%%-> Unit*/ {
    m_db.renumberSortingIndexes(get_id, caller_manages_transactions_in, isEntityAttrsNotGroupEntries = false)
  }

    fn moveEntityFromGroupToLocalEntity(toEntityIdIn: i64, moveEntityIdIn: i64, sorting_index_in: i64) /*%%-> Unit*/ {
    m_db.moveEntityFromGroupToLocalEntity(get_id, toEntityIdIn, moveEntityIdIn, sorting_index_in)
  }

    fn moveEntityToDifferentGroup(toGroupIdIn: i64, moveEntityIdIn: i64, sorting_index_in: i64) /*%%-> Unit*/ {
    m_db.moveLocalEntityFromGroupToGroup(get_id, toGroupIdIn, moveEntityIdIn, sorting_index_in)
  }

  private let mut m_already_read_data: bool = false;
  private let mut m_name: String = null;
  private let mut m_insertion_date: i64 = 0L;
  private let mut mMixedClassesAllowed: bool = false;
  private let mut m_new_entries_stick_to_top: bool = false;
*/
}
