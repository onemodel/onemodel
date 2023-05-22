/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003, 2004, 2010-2017 inclusive, 2020, and 2023, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
use anyhow::{anyhow, Result};
use crate::model::database::{DataType, Database};
use crate::util::Util;
use sqlx::{Postgres, Transaction};

pub struct Entity<'a> {
    m_id: i64,
    m_db: Box<&'a dyn Database>,
    m_already_read_data: bool,        /*%%= false*/
    m_name: String,                   /*%%= _*/
    m_class_id: Option<i64>,          /*%%= None*/
    m_insertion_date: i64,            /*%%= -1*/
    m_public: Option<bool>,           /*%%= None*/
    m_archived: bool,                 /*%%= false*/
    m_new_entries_stick_to_top: bool, /*%%= false*/
}
/*%%
package org.onemodel.core.model
import java.io.{FileInputStream, PrintWriter, StringWriter}
import java.util
import java.util.ArrayList
import org.onemodel.core._
import scala.collection.mutable
*/
impl Entity<'_> {
    /*
        fn create_entity(db_in: Database, inName: String, inClassId: Option<i64> = None, is_public_in: Option<bool> = None) -> Entity {
        let id: i64 = db_in.create_entity(inName, inClassId, is_public_in);
        new Entity(db_in, id)
      }

        fn name_length -> Int {
        Util::entity_name_length()
        }

        fn isDuplicate(db_in: Database, inName: String, inSelfIdToIgnore: Option<i64> = None) -> bool {
        db_in.isDuplicateEntityName(inName, inSelfIdToIgnore)
      }
    */

    /*
        /// This is for times when you want None if it doesn't exist, instead of the exception thrown by the Entity constructor.  Or for convenience in tests.
        fn get_entity(db_in: Box<dyn Database>, id: i64) -> Result<Option<Entity>, String> {
      //%%$%%
          try Some(new Entity(db_in, id))
          catch {
              case e: java.lang.Exception =>
                //idea: change this to actually get an "OM_NonexistentEntityException" or such, not text, so it works
                // when we have multiple databases that might not throw the same string! (& in similar places).
                if e.toString.indexOf(Util::DOES_NOT_EXIST) >= 0) {
                  None
                }
                else throw e
        }
      }

        const PRIVACY_PUBLIC: &'static str = "[PUBLIC]";
        const PRIVACY_NON_PUBLIC: &'static str = "[NON-PUBLIC]";
        const PRIVACY_UNSET: &'static str = "[UNSET]";

    */
    /// Represents one object in the system.
    /// This 1st constructor instantiates an existing object from the DB. Generally use Model.createObject() to create a new object.
    /// Note: Having Entities and other DB objects be readonly makes the code clearer & avoid some bugs, similarly to reasons for immutability in scala.
    /// (At least that has been the idea. But that might change as I just discovered a case where that causes a bug and it seems cleaner to have a
    /// set... method to fix it.)
    pub fn new2<'a>(db: Box<&'a dyn Database>, transaction: &Option<&mut Transaction<Postgres>>, id: i64) -> Result<Entity<'a>, anyhow::Error> {
        // (See comment in similar spot in BooleanAttribute for why not checking for exists, if m_db.is_remote.)
        if !db.is_remote() && !db.entity_key_exists(transaction, id, true)? {
            return Err(anyhow!(format!("Key {}{}", id, Util::DOES_NOT_EXIST)));
        }
        Ok(Entity {
            m_id: id,
            m_db: db,
            m_already_read_data: false,
            m_name: "".to_string(),
            m_class_id: None,
            m_insertion_date: -1,
            m_public: None,
            m_archived: false,
            m_new_entries_stick_to_top: false,
        })
    }
    /*
        /// This one is perhaps only called by the database class implementation--so it can return arrays of objects & save more DB hits
        /// that would have to occur if it only returned arrays of keys. This DOES NOT create a persistent object--but rather should reflect
        /// one that already exists.
        pub fn new8(m_db: Database, m_id: i64, name_in: String, class_id_in: Option<i64> = None, insertion_dateIn: i64, publicIn: Option<bool>,
               archivedIn: bool, newEntriesStickToTopIn: bool) {
            this(m_db, m_id)
            m_name = name_in
            m_class_id = class_id_in
            m_insertion_date = insertion_dateIn
            m_public = publicIn
            m_archived = archivedIn
            m_new_entries_stick_to_top = newEntriesStickToTopIn
            m_already_read_data = true
          }

          /// Allows create_entity to return an instance without duplicating the database check that it Entity(long, Database) does.
          /// (The 3rd parameter "ignoreMe" is so it will have a different signature and avoid compile errors.)
          // Idea: replace this w/ a mock? where used? same, for similar code elsewhere like in OmInstance? (and EntityTest etc could be with mocks
          // instead of real db use.)  Does this really skip that other check though?
          //%%was:  @SuppressWarnings(Array("unused"))
        pub fn new3(db_in: Database, id_in: i64, ignoreMe: bool) {
            this(db_in, id_in)
        }

    */
    /// When using, consider if getArchivedStatusDisplayString should be called with it in the display (see usage examples of getArchivedStatusDisplayString).
    pub fn get_name(&mut self, transaction: &Option<&mut Transaction<Postgres>>) -> Result<&String, anyhow::Error> {
        if !self.m_already_read_data {
            self.read_data_from_db(transaction)?;
        }
        Ok(&self.m_name)
    }

    /*
        fn getClassId -> Option<i64> {
        if !m_already_read_data) read_data_from_db()
        m_class_id
      }

        fn getClassTemplateEntityId -> Option<i64> {
        let class_id = getClassId;
        if class_id.isEmpty) None
        else {
          let templateEntityId: Option<i64> = m_db.getClassData(m_class_id.get)(1).asInstanceOf[Option<i64>];
          templateEntityId
        }
      }

        fn getCreationDate -> i64 {
        if !m_already_read_data) read_data_from_db()
        m_insertion_date
      }

        fn getCreationDateFormatted -> String {
        Util::DATEFORMAT.format(new java.util.Date(getCreationDate))
      }

        fn getPublic -> Option<bool> {
        if !m_already_read_data) read_data_from_db()
        m_public
      }

        fn getPublicStatusDisplayString(blankIfUnset: bool = true) -> String {
        if !m_already_read_data) read_data_from_db()

        if m_public.is_some() && m_public.get) {
          Entity.PRIVACY_PUBLIC
        } else if m_public.is_some() && !m_public.get) {
          Entity.PRIVACY_NON_PUBLIC
        } else if m_public.isEmpty) {
          if blankIfUnset) "" else Entity.PRIVACY_UNSET
        } else throw
          new OmException("how did we get here?")
      }

        fn getPublicStatusDisplayStringWithColor(blankIfUnset: bool = true) -> String {
        //idea: maybe this (logic) knowledge really belongs in the TextUI class. (As some others, probably.)
        let s = this.getPublicStatusDisplayString(blankIfUnset);
        if s == Entity.PRIVACY_PUBLIC) {
          Color.green(s)
        } else if s == Entity.PRIVACY_NON_PUBLIC) {
          Color.yellow(s)
        } else {
          s
        }
      }

        fn getArchivedStatus -> bool {
        if !m_already_read_data) read_data_from_db()
        m_archived
      }

        fn is_archived -> bool {
        if !m_already_read_data) read_data_from_db()
        m_archived
      }

        fn getNewEntriesStickToTop -> bool {
        if !m_already_read_data) read_data_from_db()
        m_new_entries_stick_to_top
      }

        fn getInsertionDate -> i64 {
        if !m_already_read_data) read_data_from_db()
        m_insertion_date
      }

        fn getArchivedStatusDisplayString -> String {
        if !is_archived) {
          ""
        } else {
          if m_db.include_archived_entities) {
            "[ARCHIVED]"
          } else {
            throw new OmException("FYI in case this can be better understood and fixed:  due to an error, the program " +
                                  "got an archived entity to display, but this is probably a bug, " +
                                  "because the db setting to show archived entities is turned off. The entity is " + get_id + " : " + get_name)
          }
        }
      }
    */

    fn read_data_from_db(&mut self, transaction: &Option<&mut Transaction<Postgres>>) -> Result<(), anyhow::Error> {
        let entity_data = self.m_db.get_entity_data(transaction, self.m_id)?;
        if entity_data.len() == 0 {
            return Err(anyhow!(format!(
                "No results returned from data request for: {}",
                self.m_id
            )));
        }
        //idea: surely there is some better way than what I am doing here? See other places similarly.

        // DataType::String(self.m_name) = entity_data[0];
        self.m_name = match &entity_data[0] {
            DataType::String(x) => x.clone(),
            _ => return Err(anyhow!(format!("How did we get here for {:?}?", entity_data[0]))),
        };

        //%%$%FIXME TO USE: entity_data[1]; RELY ON TESTS that I find or uncomment in order, to
        //see what will happen when a null is returned from get_entity_data above, and its dependencies
        // that eventually call pgdb.db_query and see how they all handle a NULL coming back from pg, therefore
        // how to handle that when it gets here.  AND SIMILARLY/SAME do for the fixme just below!
        // DataType::Bigint(self.m_class_id) = None;
        self.m_class_id = None;
        // self.m_class_id = match entity_data[1] {
        //     DataType::Bigint(x) => x,
        //     _ => return Err(anyhow!(format!("How did we get here for {:?}?", entity_data[1]))),
        // };

        self.m_public = None; //%%$%7FIXME TO USE:entity_data[3].asInstanceOf[Option<bool>]
                              // self.m_public = match entity_data[3] {
                              //     DataType::Boolean(x) => x,
                              //     _ => return Err(anyhow!(format!("How did we get here for {:?}?", entity_data[3]))),
                              // };

        // DataType::Bigint(self.m_insertion_date) = entity_data[2];
        self.m_insertion_date = match entity_data[2] {
            DataType::Bigint(x) => x,
            _ => return Err(anyhow!(format!("How did we get here for {:?}?", entity_data[2]))),
        };
        // DataType::Boolean(self.m_archived) = entity_data[4];
        self.m_archived = match entity_data[4] {
            DataType::Boolean(x) => x,
            _ => return Err(anyhow!(format!("How did we get here for {:?}?", entity_data[4]))),
        };
        // DataType::Boolean(self.m_new_entries_stick_to_top) = entity_data[5];
        self.m_new_entries_stick_to_top = match entity_data[5] {
            DataType::Boolean(x) => x,
            _ => return Err(anyhow!(format!("How did we get here for {:?}?", entity_data[5]))),
        };
        self.m_already_read_data = true;
        Ok(())
    }
    /*
     fn get_idWrapper -> IdWrapper() {
     new IdWrapper(m_id)
     }

    */
    pub fn get_id(&self) -> i64 {
        self.m_id
    }
    /*
      /// Intended as a temporarily unique string to distinguish an entity, across OM Instances.  NOT intended as a permanent unique ID (since
      /// the remote address for a given OM instance can change! and the local address is displayed as blank!), see uniqueIdentifier
      /// for that.  This one is like that other in a way, but more for human consumption (eg data export for human reading, not for re-import -- ?).
      lazy let readableIdentifier: String = {;
        let remotePrefix =;
          if m_db.get_remote_address.isEmpty) {
            ""
          } else {
            m_db.get_remote_address.get + "_"
          }
        remotePrefix + get_id.toString
      }

      /** Intended as a unique string to distinguish an entity, even across OM Instances.  Compare to getHumanIdentifier.
        * Idea: would any (future?) use cases be better served by including *both* the human-readable address (as in
        * getHumanIdentifier) and the instance id? Or, just combine the methods into one?
        */
      let uniqueIdentifier: String = {;
        m_db.id + "_" + get_id
      }

        fn get_attribute_count(include_archived_entities_in: bool = m_db.include_archived_entities) -> i64 {
        m_db.get_attribute_count(m_id, include_archived_entities_in)
      }

        fn get_relation_to_group_count -> i64 {
            m_db.get_relation_to_group_count(m_id)
        }

        fn get_display_string_helper(withColor: bool) -> String {
        let mut displayString: String = {;
          if withColor) {
            getPublicStatusDisplayStringWithColor() + getArchivedStatusDisplayString + Color.blue(get_name)
          } else {
            getPublicStatusDisplayString() + getArchivedStatusDisplayString + get_name
          }
        }
        let definerInfo = if m_db.getClassCount(Some(m_id)) > 0) "template (defining entity) for " else "";
        let class_name: Option<String> = if getClassId.is_some()) m_db.getClassName(getClassId.get) else None;
        displayString += (if class_name.is_some()) " (" + definerInfo + "class: " + class_name.get + ")" else "")
        displayString
      }

        fn get_display_string(withColor: bool = false) -> String {
        let mut result = "";
        try {
          result = get_display_string_helper(withColor)
        } catch {
          case e: Exception =>
            result += "Unable to get entity description due to: "
            result += {
              let sw: StringWriter = new StringWriter();
              e.printStackTrace(new PrintWriter(sw))
              sw.toString
            }
        }
        result
      }

      /** Also for convenience */
        fn addQuantityAttribute(inAttrTypeId: i64, inUnitId: i64, inNumber: Float, sorting_index_in: Option<i64>) -> QuantityAttribute {
        addQuantityAttribute(inAttrTypeId, inUnitId, inNumber, sorting_index_in, None, System.currentTimeMillis())
      }

      /** Creates a quantity attribute on this Entity (i.e., "6 inches length"), with default values of "now" for the dates. See "addQuantityAttribute" comment
       in db implementation file,
       for explanation of the parameters. It might also be nice to add the recorder's ID (person or app), but we'd have to do some kind
       of authentication/login 1st? And a GUID for users (as Entities?)?
       See PostgreSQLDatabase.createQuantityAttribute(...) for details.
        */
        fn addQuantityAttribute(inAttrTypeId: i64, inUnitId: i64, inNumber: Float, sorting_index_in: Option<i64> = None,
                               inValidOnDate: Option<i64>, observation_date_in: i64) -> QuantityAttribute {
        // write it to the database table--w/ a record for all these attributes plus a key indicating which Entity
        // it all goes with
        let id = m_db.createQuantityAttribute(m_id, inAttrTypeId, inUnitId, inNumber, inValidOnDate, observation_date_in, sorting_index_in = sorting_index_in);
        new QuantityAttribute(m_db, id)
      }

        fn getQuantityAttribute(inKey: i64) -> QuantityAttribute {
            new
            QuantityAttribute(m_db, inKey)
        }

        fn getTextAttribute(inKey: i64) -> TextAttribute {
            new TextAttribute(m_db, inKey)
        }

        fn getDateAttribute(inKey: i64) -> DateAttribute {
            new_dateAttribute(m_db, inKey)
        }

        fn get_booleanAttribute(inKey: i64) -> BooleanAttribute {
            new BooleanAttribute(m_db, inKey)
        }

        fn getFileAttribute(inKey: i64) -> FileAttribute {
            new FileAttribute(m_db, inKey)
        }

        fn getCountOfContainingGroups -> i64 {
        m_db.getCountOfGroupsContainingEntity(get_id)
      }

        fn getContainingGroupsIds -> ArrayList[i64] {
        m_db.getContainingGroupsIds(get_id)
      }

        fn getContainingRelationsToGroup(startingIndexIn: i64 = 0, maxValsIn: Option<i64> = None) -> java.util.ArrayList[RelationToGroup] {
        m_db.getContainingRelationsToGroup(get_id, startingIndexIn, maxValsIn)
      }

        fn getContainingRelationToGroupDescriptions(limitIn: Option<i64> = None) -> util.ArrayList[String] {
        m_db.getContainingRelationToGroupDescriptions(get_id, limitIn)
      }

        fn findRelationToAndGroup: (Option<i64>, Option<i64>, Option<i64>, Option<String>, bool) {
        m_db.find_relation_to_and_group_on_entity(get_id)
      }

        fn find_contained_local_entity_ids(results_in_out: mutable.TreeSet[i64], search_string_in: String, levels_remainingIn: Int = 20,
                                 stop_after_any_foundIn: bool = true) -> mutable.TreeSet[i64] {
        m_db.find_contained_local_entity_ids(results_in_out, get_id, search_string_in, levels_remainingIn, stop_after_any_foundIn)
      }

        fn getCountOfContainingLocalEntities -> (i64, i64) {
        m_db.getCountOfLocalEntitiesContainingLocalEntity(get_id)
      }

        fn getLocalEntitiesContainingEntity(startingIndexIn: i64 = 0, maxValsIn: Option<i64> = None): java.util.ArrayList[(i64, Entity)] {
        m_db.getLocalEntitiesContainingLocalEntity(get_id, startingIndexIn, maxValsIn)
      }

        fn getAdjacentAttributesSortingIndexes(sorting_index_in: i64, limitIn: Option<i64> = None, forwardNotBackIn: bool = true) -> List[Array[Option[Any]]] {
        m_db.getAdjacentAttributesSortingIndexes(get_id, sorting_index_in, limitIn, forwardNotBackIn = forwardNotBackIn)
      }

        fn getNearestAttributeEntrysSortingIndex(startingPointSortingIndexIn: i64, forwardNotBackIn: bool = true) -> Option<i64> {
        m_db.getNearestAttributeEntrysSortingIndex(get_id, startingPointSortingIndexIn, forwardNotBackIn = forwardNotBackIn)
      }

        fn renumberSortingIndexes(caller_manages_transactions_in: bool = false) /* -> Unit%%*/ {
        m_db.renumberSortingIndexes(get_id, caller_manages_transactions_in, isEntityAttrsNotGroupEntries = true)
      }

        fn updateAttributeSortingIndex(attribute_form_id_in: i64, attribute_id_in: i64, sorting_index_in: i64) /* -> Unit%%*/ {
        m_db.updateAttributeSortingIndex(get_id, attribute_form_id_in, attribute_id_in, sorting_index_in)
      }

        fn getAttributeSortingIndex(attribute_form_id_in: i64, attribute_id_in: i64) -> i64 {
        m_db.get_entityAttributeSortingIndex(get_id, attribute_form_id_in, attribute_id_in)
      }

        fn is_attribute_sorting_index_in_use(sorting_index_in: i64) -> bool {
        m_db.is_attribute_sorting_index_in_use(get_id, sorting_index_in)
      }

        fn find_unused_attribute_sorting_index(starting_with_in: Option<i64> = None) -> i64 {
        m_db.find_unused_attribute_sorting_index(get_id, starting_with_in)
      }

        fn get_relation_to_local_entity_count(include_archived_entities_in: bool = true) -> i64 {
        m_db.get_relation_to_local_entity_count(get_id, include_archived_entities = include_archived_entities_in)
      }

        fn get_relation_to_remote_entity_count -> i64 {
        m_db.get_relation_to_remote_entity_count(get_id)
      }

        fn getTextAttributeByTypeId(typeIdIn: i64, expected_rowsIn: Option[Int] = None) -> ArrayList[TextAttribute] {
        m_db.getTextAttributeByTypeId(get_id, typeIdIn, expected_rowsIn)
      }

        fn addUriEntityWithUriAttribute(new_entity_name_in: String, uriIn: String, observation_date_in: i64, makeThem_publicIn: Option<bool>,
                                       caller_manages_transactions_in: bool, quoteIn: Option<String> = None) -> (Entity, RelationToLocalEntity) {
        m_db.addUriEntityWithUriAttribute(this, new_entity_name_in, uriIn, observation_date_in, makeThem_publicIn, caller_manages_transactions_in, quoteIn)
      }

        fn create_text_attribute(attr_type_id_in: i64, text_in: String, valid_on_date_in: Option<i64> = None,
                              observation_date_in: i64 = System.currentTimeMillis(), caller_manages_transactions_in: bool = false,
                              sorting_index_in: Option<i64> = None) -> /*id*/ i64 {
        m_db.create_text_attribute(get_id, attr_type_id_in, text_in, valid_on_date_in, observation_date_in, caller_manages_transactions_in, sorting_index_in)
      }

        fn updateContainedEntitiesPublicStatus(newValueIn: Option<bool>) -> Int {
        let (attrTuples: Array[(i64, Attribute)], _) = getSortedAttributes(0, 0, onlyPublicEntitiesIn = false);
        let mut count = 0;
        for (attr <- attrTuples) {
          attr._2 match {
            case attribute: RelationToEntity =>
              // Using RelationToEntity here because it actually makes sense. But usually it is best to make sure to use either RelationToLocalEntity
              // or RelationToRemoteEntity, to be clearer about the logic.
              require(attribute.getRelatedId1 == get_id, "Unexpected value: " + attribute.getRelatedId1)
              let e: Entity = new Entity(Database.currentOrRemoteDb(attribute, m_db), attribute.getRelatedId2);
              e.updatePublicStatus(newValueIn)
              count += 1
            case attribute: RelationToGroup =>
              let group_id: i64 = attribute.getGroupId;
              let entries: List[Array[Option[Any]]] = m_db.getGroupEntriesData(group_id, None, include_archived_entities_in = false);
              for (entry <- entries) {
                let entity_id = entry(0).get.asInstanceOf[i64];
                m_db.updateEntityOnlyPublicStatus(entity_id, newValueIn)
                count += 1
              }
            case _ =>
            // do nothing
          }
        }
        count
      }

      /** See addQuantityAttribute(...) methods for comments. */
        fn addTextAttribute(inAttrTypeId: i64, inText: String, sorting_index_in: Option<i64>) -> TextAttribute {
        addTextAttribute(inAttrTypeId, inText, sorting_index_in, None, System.currentTimeMillis)
      }

        fn addTextAttribute(inAttrTypeId: i64, inText: String, sorting_index_in: Option<i64>, inValidOnDate: Option<i64>, observation_date_in: i64,
                           caller_manages_transactions_in: bool = false) -> TextAttribute {
        let id = m_db.create_text_attribute(m_id, inAttrTypeId, inText, inValidOnDate, observation_date_in, caller_manages_transactions_in, sorting_index_in);
        new TextAttribute(m_db, id)
      }

        fn addDateAttribute(inAttrTypeId: i64, inDate: i64, sorting_index_in: Option<i64> = None) -> DateAttribute {
        let id = m_db.createDateAttribute(m_id, inAttrTypeId, inDate, sorting_index_in);
        new DateAttribute(m_db, id)
      }

        fn addBooleanAttribute(inAttrTypeId: i64, inBoolean: bool, sorting_index_in: Option<i64>) -> BooleanAttribute {
        addBooleanAttribute(inAttrTypeId, inBoolean, sorting_index_in, None, System.currentTimeMillis)
      }

        fn addBooleanAttribute(inAttrTypeId: i64, inBoolean: bool, sorting_index_in: Option<i64> = None,
                              inValidOnDate: Option<i64>, observation_date_in: i64) -> BooleanAttribute {
        let id = m_db.create_boolean_attribute(m_id, inAttrTypeId, inBoolean, inValidOnDate, observation_date_in, sorting_index_in);
        new BooleanAttribute(m_db, id)
      }

        fn addFileAttribute(inAttrTypeId: i64, inFile: java.io.File) -> FileAttribute {
        addFileAttribute(inAttrTypeId, inFile.get_name, inFile)
      }

        fn addFileAttribute(inAttrTypeId: i64, descriptionIn: String, inFile: java.io.File, sorting_index_in: Option<i64> = None) -> FileAttribute {
        if !inFile.exists()) {
          throw new Exception("File " + inFile.getCanonicalPath + " doesn't exist.")
        }
        // idea: could be a little faster if the md5Hash method were merged into the database method, so that the file is only traversed once (for both
        // upload and md5 calculation).
        let mut inputStream: java.io.FileInputStream = null;
        try {
          inputStream = new FileInputStream(inFile)
          let id = m_db.createFileAttribute(m_id, inAttrTypeId, descriptionIn, inFile.lastModified, System.currentTimeMillis, inFile.getCanonicalPath,;
                                           inFile.canRead, inFile.canWrite, inFile.canExecute, inFile.length, FileAttribute.md5Hash(inFile), inputStream,
                                           sorting_index_in)
          new FileAttribute(m_db, id)
        }
        finally {
          if inputStream != null) {
            inputStream.close()
          }
        }
      }

        fn addRelationToLocalEntity(inAttrTypeId: i64, inEntityId2: i64, sorting_index_in: Option<i64>,
                              inValidOnDate: Option<i64> = None, observation_date_in: i64 = System.currentTimeMillis) -> RelationToLocalEntity {
        let rte_id = m_db.create_relation_to_local_entity(inAttrTypeId, get_id, inEntityId2, inValidOnDate, observation_date_in, sorting_index_in).get_id;
        new RelationToLocalEntity(m_db, rte_id, inAttrTypeId, get_id, inEntityId2)
      }

        fn addRelationToRemoteEntity(inAttrTypeId: i64, inEntityId2: i64, sorting_index_in: Option<i64>,
                              inValidOnDate: Option<i64> = None, observation_date_in: i64 = System.currentTimeMillis,
                              remote_instance_id_in: String) -> RelationToRemoteEntity {
        let rte_id = m_db.create_relation_to_remote_entity(inAttrTypeId, get_id, inEntityId2, inValidOnDate, observation_date_in,;
                                                     remote_instance_id_in, sorting_index_in).get_id
        new RelationToRemoteEntity(m_db, rte_id, inAttrTypeId, get_id, remote_instance_id_in, inEntityId2)
      }

      /** Creates then adds a particular kind of rtg to this entity.
        * Returns new group's id, and the new RelationToGroup object
        * */
        fn create_groupAndAddHASRelationToIt(new_group_name_in: String, mixed_classes_allowedIn: bool, observation_date_in: i64,
                                           caller_manages_transactions_in: bool = false) -> (Group, RelationToGroup) {
        // the "has" relation type that we want should always be the 1st one, since it is created by in the initial app startup; otherwise it seems we can use it
        // anyway:
        let relationTypeId = m_db.find_relation_type(Database.THE_HAS_RELATION_TYPE_NAME, Some(1)).get(0);
        let (group, rtg) = addGroupAndRelationToGroup(relationTypeId, new_group_name_in, mixed_classes_allowedIn, None, observation_date_in,;
                                                      None, caller_manages_transactions_in)
        (group, rtg)
      }

      /** Like others, returns the new things' IDs. */
        fn addGroupAndRelationToGroup(rel_type_idIn: i64, new_group_name_in: String, allow_mixed_classes_in_group_in: bool = false, valid_on_date_in: Option<i64>,
                                     observation_date_in: i64, sorting_index_in: Option<i64>, caller_manages_transactions_in: bool = false) -> (Group, RelationToGroup) {
        let (group_id: i64, rtg_id: i64) = m_db.create_group_and_relation_to_group(get_id, rel_type_idIn, new_group_name_in, allow_mixed_classes_in_group_in, valid_on_date_in,;
                                                                             observation_date_in, sorting_index_in, caller_manages_transactions_in)
        let group = new Group(m_db, group_id);
        let rtg = new RelationToGroup(m_db, rtg_id, get_id, rel_type_idIn, group_id);
        (group, rtg)
      }

      /**
       * @return the id of the new RTE
       */
        fn addHASRelationToLocalEntity(entity_id_in: i64, valid_on_date_in: Option<i64>, observation_date_in: i64) -> RelationToLocalEntity {
        m_db.addHASRelationToLocalEntity(get_id, entity_id_in, valid_on_date_in, observation_date_in)
      }

      /** Creates new entity then adds it a particular kind of rte to this entity.
        * */
        fn create_entityAndAddHASLocalRelationToIt(new_entity_name_in: String, observation_date_in: i64, is_public_in: Option<bool>,
                                            caller_manages_transactions_in: bool = false) -> (Entity, RelationToLocalEntity) {
        // the "has" relation type that we want should always be the 1st one, since it is created by in the initial app startup; otherwise it seems we can use it
        // anyway:
        let relationTypeId = m_db.find_relation_type(Database.THE_HAS_RELATION_TYPE_NAME, Some(1)).get(0);
        let (entity: Entity, rte: RelationToLocalEntity) = addEntityAndRelationToLocalEntity(relationTypeId, new_entity_name_in, None, observation_date_in,;
                                                                                             is_public_in, caller_manages_transactions_in)
        (entity, rte)
      }

        fn addEntityAndRelationToLocalEntity(rel_type_idIn: i64, new_entity_name_in: String, valid_on_date_in: Option<i64>, observation_date_in: i64,
                                       is_public_in: Option<bool>, caller_manages_transactions_in: bool = false) -> (Entity, RelationToLocalEntity) {
        let (entity_id, rte_id) = m_db.create_entity_and_relation_to_local_entity(get_id, rel_type_idIn, new_entity_name_in, is_public_in, valid_on_date_in, observation_date_in,;
                                                                    caller_manages_transactions_in)
        let entity = new Entity(m_db, entity_id);
        let rte = new RelationToLocalEntity(m_db, rte_id, rel_type_idIn, m_id, entity_id);
        (entity, rte)
      }

      /**
        * @return the new group's id.
        */
        fn addRelationToGroup(rel_type_idIn: i64, group_id_in: i64, sorting_index_in: Option<i64>) -> RelationToGroup {
        addRelationToGroup(rel_type_idIn, group_id_in, sorting_index_in, None, System.currentTimeMillis)
      }

        fn addRelationToGroup(rel_type_idIn: i64, group_id_in: i64, sorting_index_in: Option<i64>,
                             valid_on_date_in: Option<i64>, observation_date_in: i64) -> RelationToGroup {
        let (newRtg_id, sorting_index) = m_db.create_relation_to_group(get_id, rel_type_idIn, group_id_in, valid_on_date_in, observation_date_in, sorting_index_in);
        new RelationToGroup(m_db, newRtg_id, get_id, rel_type_idIn, group_id_in, valid_on_date_in, observation_date_in, sorting_index)
      }

        fn getSortedAttributes(startingObjectIndexIn: Int = 0, maxValsIn: Int = 0, onlyPublicEntitiesIn: bool = true) -> (Array[(i64, Attribute)], Int) {
        m_db.getSortedAttributes(get_id, startingObjectIndexIn, maxValsIn, onlyPublicEntitiesIn = onlyPublicEntitiesIn)
      }

        fn updateClass(class_id_in: Option<i64>) /*%% -> Unit*/ {
        if !m_already_read_data) read_data_from_db()
        if class_id_in != m_class_id) {
          m_db.updateEntitysClass(this.get_id, class_id_in)
          m_class_id = class_id_in
        }
      }

        fn updateNewEntriesStickToTop(b: bool) {
        if !m_already_read_data) read_data_from_db()
        if b != m_new_entries_stick_to_top) {
          m_db.updateEntityOnlyNewEntriesStickToTop(get_id, b)
          m_new_entries_stick_to_top = b
        }
      }

        fn updatePublicStatus(newValueIn: Option<bool>) {
        if !m_already_read_data) read_data_from_db()
        if newValueIn != m_public) {
          // The condition for this (when it was part of EntityMenu) used to include " && !entity_in.isInstanceOf[RelationType]", but maybe it's better w/o that.
          m_db.updateEntityOnlyPublicStatus(get_id, newValueIn)
          m_public = newValueIn
        }
      }

        fn updateName(name_in: String) /*%% -> Unit*/ {
        if !m_already_read_data) read_data_from_db()
        if name_in != m_name) {
          m_db.updateEntityOnlyName(get_id, name_in);
          m_name = name_in
        }
      }

        fn archive() {
        m_db.archiveEntity(m_id);
        m_archived = true
      }

        fn unarchive() {
        m_db.unarchiveEntity(m_id);
        m_archived = false
      }

      /** Removes this object from the system. */
        fn delete() {
          m_db.delete_entity(m_id)
      }

    */
}
