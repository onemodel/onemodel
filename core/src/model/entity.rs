/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003, 2004, 2010-2017 inclusive, 2020, and 2023, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
use crate::model::database::Database;
use crate::util::Util;

#[derive(Clone, Copy)]
pub struct Entity {
    /*
    mAlreadyReadData: bool = false;
    mName: String = _;
    mClassId: Option<i64> = None;
    mInsertionDate: i64 = -1;
    mPublic: Option<bool> = None;
    mArchived: bool = false;
    mNewEntriesStickToTop: bool = false;
     */
}
/*%%
package org.onemodel.core.model
import java.io.{FileInputStream, PrintWriter, StringWriter}
import java.util
import java.util.ArrayList
import org.onemodel.core._
import scala.collection.mutable
*/
impl Entity {
  /*
    fn createEntity(in_db: Database, inName: String, inClassId: Option<i64> = None, isPublicIn: Option<bool> = None) -> Entity {
    let id: i64 = in_db.createEntity(inName, inClassId, isPublicIn);
    new Entity(in_db, id)
  }

    fn name_length -> Int {
    Database.entityNameLength
    }

    fn isDuplicate(in_db: Database, inName: String, inSelfIdToIgnore: Option<i64> = None) -> Boolean {
    in_db.isDuplicateEntityName(inName, inSelfIdToIgnore)
  }
*/

/*
    /// This is for times when you want None if it doesn't exist, instead of the exception thrown by the Entity constructor.  Or for convenience in tests.
    fn get_entity(in_db: Box<dyn Database>, id: i64) -> Result<Option<Entity>, String> {
  //%%$%%
      try Some(new Entity(in_db, id))
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

    /// Represents one object in the system.
    /// This 1st constructor instantiates an existing object from the DB. Generally use Model.createObject() to create a new object.
    /// Note: Having Entities and other DB objects be readonly makes the code clearer & avoid some bugs, similarly to reasons for immutability in scala.
    /// (At least that has been the idea. But that might change as I just discovered a case where that causes a bug and it seems cleaner to have a
    /// set... method to fix it.)
    pub fn new2(db: Box< dyn Database>, id: i64) -> Result<Entity, String> {
      // (See comment in similar spot in BooleanAttribute for why not checking for exists, if mDB.is_remote.)
      if ! db.is_remote() && ! db.entity_key_exists(id) {
        // DON'T CHANGE this msg unless you also change the trap for it in TextUI.java.
        throw new Exception("Key " + mId + Util::DOES_NOT_EXIST)
      }
    }

    /// This one is perhaps only called by the database class implementation--so it can return arrays of objects & save more DB hits
    /// that would have to occur if it only returned arrays of keys. This DOES NOT create a persistent object--but rather should reflect
    /// one that already exists.
    pub fn new8(mDB: Database, mId: i64, name_in: String, classIdIn: Option<i64> = None, insertionDateIn: i64, publicIn: Option<bool>,
           archivedIn: Boolean, newEntriesStickToTopIn: Boolean) {
        this(mDB, mId)
        mName = name_in
        mClassId = classIdIn
        mInsertionDate = insertionDateIn
        mPublic = publicIn
        mArchived = archivedIn
        mNewEntriesStickToTop = newEntriesStickToTopIn
        mAlreadyReadData = true
      }

      /// Allows createEntity to return an instance without duplicating the database check that it Entity(long, Database) does.
      /// (The 3rd parameter "ignoreMe" is so it will have a different signature and avoid compile errors.)
      // Idea: replace this w/ a mock? where used? same, for similar code elsewhere like in OmInstance? (and EntityTest etc could be with mocks
      // instead of real db use.)  Does this really skip that other check though?
      //%%was:  @SuppressWarnings(Array("unused"))
    pub fn new3(in_db: Database, inID: i64, ignoreMe: Boolean) {
        this(in_db, inID)
    }

    /// When using, consider if getArchivedStatusDisplayString should be called with it in the display (see usage examples of getArchivedStatusDisplayString).
    fn get_name() -> String {
        if !mAlreadyReadData {
            readDataFromDB();
        }
        mName
    }

    */
  /*
    fn getClassId -> Option<i64> {
    if !mAlreadyReadData) readDataFromDB()
    mClassId
  }

    fn getClassTemplateEntityId -> Option<i64> {
    let classId = getClassId;
    if classId.isEmpty) None
    else {
      let templateEntityId: Option<i64> = mDB.getClassData(mClassId.get)(1).asInstanceOf[Option<i64>];
      templateEntityId
    }
  }

    fn getCreationDate -> i64 {
    if !mAlreadyReadData) readDataFromDB()
    mInsertionDate
  }

    fn getCreationDateFormatted -> String {
    Util.DATEFORMAT.format(new java.util.Date(getCreationDate))
  }

    fn getPublic -> Option<bool> {
    if !mAlreadyReadData) readDataFromDB()
    mPublic
  }

    fn getPublicStatusDisplayString(blankIfUnset: Boolean = true) -> String {
    if !mAlreadyReadData) readDataFromDB()

    if mPublic.is_defined && mPublic.get) {
      Entity.PRIVACY_PUBLIC
    } else if mPublic.is_defined && !mPublic.get) {
      Entity.PRIVACY_NON_PUBLIC
    } else if mPublic.isEmpty) {
      if blankIfUnset) "" else Entity.PRIVACY_UNSET
    } else throw
      new OmException("how did we get here?")
  }

    fn getPublicStatusDisplayStringWithColor(blankIfUnset: Boolean = true) -> String {
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

    fn getArchivedStatus -> Boolean {
    if !mAlreadyReadData) readDataFromDB()
    mArchived
  }

    fn is_archived -> Boolean {
    if !mAlreadyReadData) readDataFromDB()
    mArchived
  }

    fn getNewEntriesStickToTop -> Boolean {
    if !mAlreadyReadData) readDataFromDB()
    mNewEntriesStickToTop
  }

    fn getInsertionDate -> i64 {
    if !mAlreadyReadData) readDataFromDB()
    mInsertionDate
  }

    fn getArchivedStatusDisplayString -> String {
    if !is_archived) {
      ""
    } else {
      if mDB.include_archived_entities) {
        "[ARCHIVED]"
      } else {
        throw new OmException("FYI in case this can be better understood and fixed:  due to an error, the program " +
                              "got an archived entity to display, but this is probably a bug, " +
                              "because the db setting to show archived entities is turned off. The entity is " + get_id + " : " + get_name)
      }
    }
  }

  protected fn readDataFromDB() {
    let entityData = mDB.get_entityData(mId);
    if entityData.length == 0) {
      throw new OmException("No results returned from data request for: " + mId)
    }
    mName = entityData(0).get.asInstanceOf[String]
    mClassId = entityData(1).asInstanceOf[Option<i64>]
    mInsertionDate = entityData(2).get.asInstanceOf[i64]
    mPublic = entityData(3).asInstanceOf[Option<bool>]
    mArchived = entityData(4).get.asInstanceOf[Boolean]
    mNewEntriesStickToTop = entityData(5).get.asInstanceOf[Boolean]
    mAlreadyReadData = true
  }

    fn get_idWrapper -> IdWrapper() {
    new IdWrapper(mId)
    }

    fn get_id -> i64 {
    mId
    }

  /** Intended as a temporarily unique string to distinguish an entity, across OM Instances.  NOT intended as a permanent unique ID (since
    * the remote address for a given OM instance can change! and the local address is displayed as blank!), see uniqueIdentifier
    * for that.  This one is like that other in a way, but more for human consumption (eg data export for human reading, not for re-import -- ?).
    */
  lazy let readableIdentifier: String = {;
    let remotePrefix =;
      if mDB.get_remote_address.isEmpty) {
        ""
      } else {
        mDB.get_remote_address.get + "_"
      }
    remotePrefix + get_id.toString
  }

  /** Intended as a unique string to distinguish an entity, even across OM Instances.  Compare to getHumanIdentifier.
    * Idea: would any (future?) use cases be better served by including *both* the human-readable address (as in
    * getHumanIdentifier) and the instance id? Or, just combine the methods into one?
    */
  let uniqueIdentifier: String = {;
    mDB.id + "_" + get_id
  }

    fn getAttributeCount(include_archived_entitiesIn: Boolean = mDB.include_archived_entities) -> i64 {
    mDB.getAttributeCount(mId, include_archived_entitiesIn)
  }

    fn getRelationToGroupCount -> i64 {
        mDB.getRelationToGroupCount(mId)
    }

    fn get_display_string_helper(withColor: Boolean) -> String {
    let mut displayString: String = {;
      if withColor) {
        getPublicStatusDisplayStringWithColor() + getArchivedStatusDisplayString + Color.blue(get_name)
      } else {
        getPublicStatusDisplayString() + getArchivedStatusDisplayString + get_name
      }
    }
    let definerInfo = if mDB.getClassCount(Some(mId)) > 0) "template (defining entity) for " else "";
    let className: Option<String> = if getClassId.is_defined) mDB.getClassName(getClassId.get) else None;
    displayString += (if className.is_defined) " (" + definerInfo + "class: " + className.get + ")" else "")
    displayString
  }

    fn get_display_string(withColor: Boolean = false) -> String {
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
    fn addQuantityAttribute(inAttrTypeId: i64, inUnitId: i64, inNumber: Float, sortingIndexIn: Option<i64>) -> QuantityAttribute {
    addQuantityAttribute(inAttrTypeId, inUnitId, inNumber, sortingIndexIn, None, System.currentTimeMillis())
  }

  /** Creates a quantity attribute on this Entity (i.e., "6 inches length"), with default values of "now" for the dates. See "addQuantityAttribute" comment
   in db implementation file,
   for explanation of the parameters. It might also be nice to add the recorder's ID (person or app), but we'd have to do some kind
   of authentication/login 1st? And a GUID for users (as Entities?)?
   See PostgreSQLDatabase.createQuantityAttribute(...) for details.
    */
    fn addQuantityAttribute(inAttrTypeId: i64, inUnitId: i64, inNumber: Float, sortingIndexIn: Option<i64> = None,
                           inValidOnDate: Option<i64>, inObservationDate: i64) -> QuantityAttribute {
    // write it to the database table--w/ a record for all these attributes plus a key indicating which Entity
    // it all goes with
    let id = mDB.createQuantityAttribute(mId, inAttrTypeId, inUnitId, inNumber, inValidOnDate, inObservationDate, sortingIndexIn = sortingIndexIn);
    new QuantityAttribute(mDB, id)
  }

    fn getQuantityAttribute(inKey: i64) -> QuantityAttribute {
        new
        QuantityAttribute(mDB, inKey)
    }

    fn getTextAttribute(inKey: i64) -> TextAttribute {
        new TextAttribute(mDB, inKey)
    }

    fn getDateAttribute(inKey: i64) -> DateAttribute {
        new_dateAttribute(mDB, inKey)
    }

    fn getBooleanAttribute(inKey: i64) -> BooleanAttribute {
        new BooleanAttribute(mDB, inKey)
    }

    fn getFileAttribute(inKey: i64) -> FileAttribute {
        new FileAttribute(mDB, inKey)
    }

    fn getCountOfContainingGroups -> i64 {
    mDB.getCountOfGroupsContainingEntity(get_id)
  }

    fn getContainingGroupsIds -> ArrayList[i64] {
    mDB.getContainingGroupsIds(get_id)
  }

    fn getContainingRelationsToGroup(startingIndexIn: i64 = 0, maxValsIn: Option<i64> = None) -> java.util.ArrayList[RelationToGroup] {
    mDB.getContainingRelationsToGroup(get_id, startingIndexIn, maxValsIn)
  }

    fn getContainingRelationToGroupDescriptions(limitIn: Option<i64> = None) -> util.ArrayList[String] {
    mDB.getContainingRelationToGroupDescriptions(get_id, limitIn)
  }

    fn findRelationToAndGroup: (Option<i64>, Option<i64>, Option<i64>, Option<String>, Boolean) {
    mDB.findRelationToAndGroup_OnEntity(get_id)
  }

    fn findContainedLocalEntityIds(resultsInOut: mutable.TreeSet[i64], searchStringIn: String, levelsRemainingIn: Int = 20,
                             stopAfterAnyFoundIn: Boolean = true) -> mutable.TreeSet[i64] {
    mDB.findContainedLocalEntityIds(resultsInOut, get_id, searchStringIn, levelsRemainingIn, stopAfterAnyFoundIn)
  }

    fn getCountOfContainingLocalEntities -> (i64, i64) {
    mDB.getCountOfLocalEntitiesContainingLocalEntity(get_id)
  }

    fn getLocalEntitiesContainingEntity(startingIndexIn: i64 = 0, maxValsIn: Option<i64> = None): java.util.ArrayList[(i64, Entity)] {
    mDB.getLocalEntitiesContainingLocalEntity(get_id, startingIndexIn, maxValsIn)
  }

    fn getAdjacentAttributesSortingIndexes(sortingIndexIn: i64, limitIn: Option<i64> = None, forwardNotBackIn: Boolean = true) -> List[Array[Option[Any]]] {
    mDB.getAdjacentAttributesSortingIndexes(get_id, sortingIndexIn, limitIn, forwardNotBackIn = forwardNotBackIn)
  }

    fn getNearestAttributeEntrysSortingIndex(startingPointSortingIndexIn: i64, forwardNotBackIn: Boolean = true) -> Option<i64> {
    mDB.getNearestAttributeEntrysSortingIndex(get_id, startingPointSortingIndexIn, forwardNotBackIn = forwardNotBackIn)
  }

    fn renumberSortingIndexes(callerManagesTransactionsIn: Boolean = false) /* -> Unit%%*/ {
    mDB.renumberSortingIndexes(get_id, callerManagesTransactionsIn, isEntityAttrsNotGroupEntries = true)
  }

    fn updateAttributeSortingIndex(attributeFormIdIn: i64, attributeIdIn: i64, sortingIndexIn: i64) /* -> Unit%%*/ {
    mDB.updateAttributeSortingIndex(get_id, attributeFormIdIn, attributeIdIn, sortingIndexIn)
  }

    fn getAttributeSortingIndex(attributeFormIdIn: i64, attributeIdIn: i64) -> i64 {
    mDB.get_entityAttributeSortingIndex(get_id, attributeFormIdIn, attributeIdIn)
  }

    fn isAttributeSortingIndexInUse(sortingIndexIn: i64) -> Boolean {
    mDB.isAttributeSortingIndexInUse(get_id, sortingIndexIn)
  }

    fn findUnusedAttributeSortingIndex(startingWithIn: Option<i64> = None) -> i64 {
    mDB.findUnusedAttributeSortingIndex(get_id, startingWithIn)
  }

    fn getRelationToLocalEntityCount(include_archived_entitiesIn: Boolean = true) -> i64 {
    mDB.getRelationToLocalEntityCount(get_id, include_archived_entities = include_archived_entitiesIn)
  }

    fn getRelationToRemoteEntityCount -> i64 {
    mDB.getRelationToRemoteEntityCount(get_id)
  }

    fn getTextAttributeByTypeId(typeIdIn: i64, expectedRowsIn: Option[Int] = None) -> ArrayList[TextAttribute] {
    mDB.getTextAttributeByTypeId(get_id, typeIdIn, expectedRowsIn)
  }

    fn addUriEntityWithUriAttribute(newEntityNameIn: String, uriIn: String, observationDateIn: i64, makeThemPublicIn: Option<bool>,
                                   callerManagesTransactionsIn: Boolean, quoteIn: Option<String> = None) -> (Entity, RelationToLocalEntity) {
    mDB.addUriEntityWithUriAttribute(this, newEntityNameIn, uriIn, observationDateIn, makeThemPublicIn, callerManagesTransactionsIn, quoteIn)
  }

    fn createTextAttribute(attrTypeIdIn: i64, textIn: String, valid_on_date_in: Option<i64> = None,
                          observationDateIn: i64 = System.currentTimeMillis(), callerManagesTransactionsIn: Boolean = false,
                          sortingIndexIn: Option<i64> = None) -> /*id*/ i64 {
    mDB.createTextAttribute(get_id, attrTypeIdIn, textIn, valid_on_date_in, observationDateIn, callerManagesTransactionsIn, sortingIndexIn)
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
          let e: Entity = new Entity(Database.currentOrRemoteDb(attribute, mDB), attribute.getRelatedId2);
          e.updatePublicStatus(newValueIn)
          count += 1
        case attribute: RelationToGroup =>
          let groupId: i64 = attribute.getGroupId;
          let entries: List[Array[Option[Any]]] = mDB.getGroupEntriesData(groupId, None, include_archived_entitiesIn = false);
          for (entry <- entries) {
            let entityId = entry(0).get.asInstanceOf[i64];
            mDB.updateEntityOnlyPublicStatus(entityId, newValueIn)
            count += 1
          }
        case _ =>
        // do nothing
      }
    }
    count
  }

  /** See addQuantityAttribute(...) methods for comments. */
    fn addTextAttribute(inAttrTypeId: i64, inText: String, sortingIndexIn: Option<i64>) -> TextAttribute {
    addTextAttribute(inAttrTypeId, inText, sortingIndexIn, None, System.currentTimeMillis)
  }

    fn addTextAttribute(inAttrTypeId: i64, inText: String, sortingIndexIn: Option<i64>, inValidOnDate: Option<i64>, inObservationDate: i64,
                       callerManagesTransactionsIn: Boolean = false) -> TextAttribute {
    let id = mDB.createTextAttribute(mId, inAttrTypeId, inText, inValidOnDate, inObservationDate, callerManagesTransactionsIn, sortingIndexIn);
    new TextAttribute(mDB, id)
  }

    fn addDateAttribute(inAttrTypeId: i64, inDate: i64, sortingIndexIn: Option<i64> = None) -> DateAttribute {
    let id = mDB.createDateAttribute(mId, inAttrTypeId, inDate, sortingIndexIn);
    new DateAttribute(mDB, id)
  }

    fn addBooleanAttribute(inAttrTypeId: i64, inBoolean: Boolean, sortingIndexIn: Option<i64>) -> BooleanAttribute {
    addBooleanAttribute(inAttrTypeId, inBoolean, sortingIndexIn, None, System.currentTimeMillis)
  }

    fn addBooleanAttribute(inAttrTypeId: i64, inBoolean: Boolean, sortingIndexIn: Option<i64> = None,
                          inValidOnDate: Option<i64>, inObservationDate: i64) -> BooleanAttribute {
    let id = mDB.createBooleanAttribute(mId, inAttrTypeId, inBoolean, inValidOnDate, inObservationDate, sortingIndexIn);
    new BooleanAttribute(mDB, id)
  }

    fn addFileAttribute(inAttrTypeId: i64, inFile: java.io.File) -> FileAttribute {
    addFileAttribute(inAttrTypeId, inFile.get_name, inFile)
  }

    fn addFileAttribute(inAttrTypeId: i64, descriptionIn: String, inFile: java.io.File, sortingIndexIn: Option<i64> = None) -> FileAttribute {
    if !inFile.exists()) {
      throw new Exception("File " + inFile.getCanonicalPath + " doesn't exist.")
    }
    // idea: could be a little faster if the md5Hash method were merged into the database method, so that the file is only traversed once (for both
    // upload and md5 calculation).
    let mut inputStream: java.io.FileInputStream = null;
    try {
      inputStream = new FileInputStream(inFile)
      let id = mDB.createFileAttribute(mId, inAttrTypeId, descriptionIn, inFile.lastModified, System.currentTimeMillis, inFile.getCanonicalPath,;
                                       inFile.canRead, inFile.canWrite, inFile.canExecute, inFile.length, FileAttribute.md5Hash(inFile), inputStream,
                                       sortingIndexIn)
      new FileAttribute(mDB, id)
    }
    finally {
      if inputStream != null) {
        inputStream.close()
      }
    }
  }

    fn addRelationToLocalEntity(inAttrTypeId: i64, inEntityId2: i64, sortingIndexIn: Option<i64>,
                          inValidOnDate: Option<i64> = None, inObservationDate: i64 = System.currentTimeMillis) -> RelationToLocalEntity {
    let rteId = mDB.createRelationToLocalEntity(inAttrTypeId, get_id, inEntityId2, inValidOnDate, inObservationDate, sortingIndexIn).get_id;
    new RelationToLocalEntity(mDB, rteId, inAttrTypeId, get_id, inEntityId2)
  }

    fn addRelationToRemoteEntity(inAttrTypeId: i64, inEntityId2: i64, sortingIndexIn: Option<i64>,
                          inValidOnDate: Option<i64> = None, inObservationDate: i64 = System.currentTimeMillis,
                          remoteInstanceIdIn: String) -> RelationToRemoteEntity {
    let rteId = mDB.createRelationToRemoteEntity(inAttrTypeId, get_id, inEntityId2, inValidOnDate, inObservationDate,;
                                                 remoteInstanceIdIn, sortingIndexIn).get_id
    new RelationToRemoteEntity(mDB, rteId, inAttrTypeId, get_id, remoteInstanceIdIn, inEntityId2)
  }

  /** Creates then adds a particular kind of rtg to this entity.
    * Returns new group's id, and the new RelationToGroup object
    * */
    fn createGroupAndAddHASRelationToIt(newGroupNameIn: String, mixedClassesAllowedIn: Boolean, observationDateIn: i64,
                                       callerManagesTransactionsIn: Boolean = false) -> (Group, RelationToGroup) {
    // the "has" relation type that we want should always be the 1st one, since it is created by in the initial app startup; otherwise it seems we can use it
    // anyway:
    let relationTypeId = mDB.findRelationType(Database.THE_HAS_RELATION_TYPE_NAME, Some(1)).get(0);
    let (group, rtg) = addGroupAndRelationToGroup(relationTypeId, newGroupNameIn, mixedClassesAllowedIn, None, observationDateIn,;
                                                  None, callerManagesTransactionsIn)
    (group, rtg)
  }

  /** Like others, returns the new things' IDs. */
    fn addGroupAndRelationToGroup(relTypeIdIn: i64, newGroupNameIn: String, allowMixedClassesInGroupIn: Boolean = false, valid_on_date_in: Option<i64>,
                                 inObservationDate: i64, sortingIndexIn: Option<i64>, callerManagesTransactionsIn: Boolean = false) -> (Group, RelationToGroup) {
    let (groupId: i64, rtgId: i64) = mDB.createGroupAndRelationToGroup(get_id, relTypeIdIn, newGroupNameIn, allowMixedClassesInGroupIn, valid_on_date_in,;
                                                                         inObservationDate, sortingIndexIn, callerManagesTransactionsIn)
    let group = new Group(mDB, groupId);
    let rtg = new RelationToGroup(mDB, rtgId, get_id, relTypeIdIn, groupId);
    (group, rtg)
  }

  /**
   * @return the id of the new RTE
   */
    fn addHASRelationToLocalEntity(entityIdIn: i64, valid_on_date_in: Option<i64>, observationDateIn: i64) -> RelationToLocalEntity {
    mDB.addHASRelationToLocalEntity(get_id, entityIdIn, valid_on_date_in, observationDateIn)
  }

  /** Creates new entity then adds it a particular kind of rte to this entity.
    * */
    fn createEntityAndAddHASLocalRelationToIt(newEntityNameIn: String, observationDateIn: i64, isPublicIn: Option<bool>,
                                        callerManagesTransactionsIn: Boolean = false) -> (Entity, RelationToLocalEntity) {
    // the "has" relation type that we want should always be the 1st one, since it is created by in the initial app startup; otherwise it seems we can use it
    // anyway:
    let relationTypeId = mDB.findRelationType(Database.THE_HAS_RELATION_TYPE_NAME, Some(1)).get(0);
    let (entity: Entity, rte: RelationToLocalEntity) = addEntityAndRelationToLocalEntity(relationTypeId, newEntityNameIn, None, observationDateIn,;
                                                                                         isPublicIn, callerManagesTransactionsIn)
    (entity, rte)
  }

    fn addEntityAndRelationToLocalEntity(relTypeIdIn: i64, newEntityNameIn: String, valid_on_date_in: Option<i64>, inObservationDate: i64,
                                   isPublicIn: Option<bool>, callerManagesTransactionsIn: Boolean = false) -> (Entity, RelationToLocalEntity) {
    let (entityId, rteId) = mDB.createEntityAndRelationToLocalEntity(get_id, relTypeIdIn, newEntityNameIn, isPublicIn, valid_on_date_in, inObservationDate,;
                                                                callerManagesTransactionsIn)
    let entity = new Entity(mDB, entityId);
    let rte = new RelationToLocalEntity(mDB, rteId, relTypeIdIn, mId, entityId);
    (entity, rte)
  }

  /**
    * @return the new group's id.
    */
    fn addRelationToGroup(relTypeIdIn: i64, groupIdIn: i64, sortingIndexIn: Option<i64>) -> RelationToGroup {
    addRelationToGroup(relTypeIdIn, groupIdIn, sortingIndexIn, None, System.currentTimeMillis)
  }

    fn addRelationToGroup(relTypeIdIn: i64, groupIdIn: i64, sortingIndexIn: Option<i64>,
                         valid_on_date_in: Option<i64>, observationDateIn: i64) -> RelationToGroup {
    let (newRtgId, sortingIndex) = mDB.createRelationToGroup(get_id, relTypeIdIn, groupIdIn, valid_on_date_in, observationDateIn, sortingIndexIn);
    new RelationToGroup(mDB, newRtgId, get_id, relTypeIdIn, groupIdIn, valid_on_date_in, observationDateIn, sortingIndex)
  }

    fn getSortedAttributes(startingObjectIndexIn: Int = 0, maxValsIn: Int = 0, onlyPublicEntitiesIn: Boolean = true) -> (Array[(i64, Attribute)], Int) {
    mDB.getSortedAttributes(get_id, startingObjectIndexIn, maxValsIn, onlyPublicEntitiesIn = onlyPublicEntitiesIn)
  }

    fn updateClass(classIdIn: Option<i64>) /*%% -> Unit*/ {
    if !mAlreadyReadData) readDataFromDB()
    if classIdIn != mClassId) {
      mDB.updateEntitysClass(this.get_id, classIdIn)
      mClassId = classIdIn
    }
  }

    fn updateNewEntriesStickToTop(b: Boolean) {
    if !mAlreadyReadData) readDataFromDB()
    if b != mNewEntriesStickToTop) {
      mDB.updateEntityOnlyNewEntriesStickToTop(get_id, b)
      mNewEntriesStickToTop = b
    }
  }

    fn updatePublicStatus(newValueIn: Option<bool>) {
    if !mAlreadyReadData) readDataFromDB()
    if newValueIn != mPublic) {
      // The condition for this (when it was part of EntityMenu) used to include " && !entity_in.isInstanceOf[RelationType]", but maybe it's better w/o that.
      mDB.updateEntityOnlyPublicStatus(get_id, newValueIn)
      mPublic = newValueIn
    }
  }

    fn updateName(name_in: String) /*%% -> Unit*/ {
    if !mAlreadyReadData) readDataFromDB()
    if name_in != mName) {
      mDB.updateEntityOnlyName(get_id, name_in);
      mName = name_in
    }
  }

    fn archive() {
    mDB.archiveEntity(mId);
    mArchived = true
  }

    fn unarchive() {
    mDB.unarchiveEntity(mId);
    mArchived = false
  }

  /** Removes this object from the system. */
    fn delete() {
      mDB.deleteEntity(mId)
  }

*/
}
