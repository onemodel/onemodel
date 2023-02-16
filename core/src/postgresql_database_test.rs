%%
/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003, 2004, 2010, 2011, 2013-2017 inclusive, and 2023, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
package org.onemodel.core.model

import org.onemodel.core.controllers.{Controller, ImportExport}
import org.scalatest.mockito.MockitoSugar
import java.io.File
import java.util
import org.onemodel.core._
import org.scalatest.{Args, FlatSpec, Status}
import scala.collection.mutable

object PostgreSQLDatabaseTest {
    fn tearDownTestDB() {
    // reconnect to the normal production database and tear down the temporary one we used for testing.
    // This is part of the singleton object, in part so that it can be called even before we have a Database object: this is to avoid
    // doing setup (at first db instantiation for a new system), then immediately another teardown/setup for the tests.
    try {
      PostgreSQLDatabase.destroyTables(Database.TEST_USER, Database.TEST_USER, Database.TEST_PASS)
    }
    catch {
      case e: java.sql.SQLException =>
        if e.toString.indexOf("is being accessed by other users") != -1 {
          // why did this happen sometimes?
          // but it can be ignored, as the next test run will also clean this out as it starts.
        }
        else {
          throw e
        }
    }
  }

}

class PostgreSQLDatabaseTest extends FlatSpec with MockitoSugar {
  PostgreSQLDatabaseTest.tearDownTestDB()

  // for a test
  private let mut mDoDamageBuffer = false;

  // instantiation does DB setup (creates tables, default data, etc):
  private let mDB: PostgreSQLDatabase = new PostgreSQLDatabase(Database.TEST_USER, Database.TEST_PASS) {;
    override fn damageBuffer(buffer: Array[Byte]) /*%%-> Unit*/ {
      if mDoDamageBuffer {
        if buffer.length < 1 || buffer(0) == '0' { throw new OmException("Nothing to damage here") }
        else {
          if buffer(0) == '1' { buffer(0) = 2.toByte }
          else { buffer(0) = 1.toByte }
          // once is enough until we want to cause another failure
          mDoDamageBuffer = false
        }
      }
    }
  }

  private final let QUANTITY_TYPE_NAME: String = "length";
  private final let RELATION_TYPE_NAME: String = "someRelationToEntityTypeName";

  // connect to existing database first
  private final let RELATED_ENTITY_NAME: String = "someRelatedEntityName";

  override fn runTests(testName: Option<String>, args: Args): -> Status {
    // no longer doing db setup/teardown here, because we need to do teardown as a constructor-like command above,
    // before instantiating the DB (and that instantiation does setup).  Leaving tables in place after to allow adhoc manual test access.
    let result: Status = super.runTests(testName, args);
    result
  }

  "database version table" should "have been created with right data" in {
    let versionTableExists: bool = mDB.does_this_exist("select count(1) from pg_class where relname='om_db_version'");
    assert(versionTableExists)
    let results = mDB.db_query_wrapper_for_one_row("select version from om_db_version", "Int");
    assert(results.length == 1)
    let dbVer: i32 = results(0).get.asInstanceOf[Int];
    assert(dbVer == PostgreSQLDatabase.SCHEMA_VERSION, "dbVer and PostgreSQLDatabase.SCHEMA_VERSION are: " +
                                                           dbVer + ", " + PostgreSQLDatabase.SCHEMA_VERSION)
  }

  "getLocalOmInstanceData and friends" should "work" in {
    let oi: OmInstance = mDB.getLocalOmInstanceData;
    let uuid: String = oi.get_id;
    assert(oi.getLocal)
    assert(mDB.omInstanceKeyExists(uuid))
    let startingOmiCount = mDB.getOmInstanceCount;
    assert(startingOmiCount > 0)
    let oiAgainAddress = mDB.getOmInstanceData(uuid)(1).get.asInstanceOf[String];
    assert(oiAgainAddress == Util.LOCAL_OM_INSTANCE_DEFAULT_DESCRIPTION)
    let omInstances: util.ArrayList[OmInstance] = mDB.getOmInstances();
    assert(omInstances.size == startingOmiCount)
    let sizeNowTrue = mDB.getOmInstances(Some(true)).size;
    assert(sizeNowTrue > 0)
    // Idea: fix: Next line fails at times, maybe due to code running in parallel between this and RestDatabaseTest, creating/deleting rows.  Only seems to happen
    // when all tests are run, never when the test classes are run separately.
    //    let sizeNowFalse = mDB.getOmInstances(Some(false)).size;
    //assert(sizeNowFalse < sizeNowTrue)
    assert(! mDB.omInstanceKeyExists(java.util.UUID.randomUUID().toString))
    assert(new OmInstance(mDB, uuid).getAddress == Util.LOCAL_OM_INSTANCE_DEFAULT_DESCRIPTION)

    let uuid2 = java.util.UUID.randomUUID().toString;
    mDB.createOmInstance(uuid2, isLocalIn = false, "om.example.com", Some(mDB.getSystemEntityId))
    // should have the local one created at db creation, and now the one for this test:
    assert(mDB.getOmInstanceCount == startingOmiCount + 1)
    let mut i2: OmInstance = new OmInstance(mDB, uuid2);
    assert(i2.getAddress == "om.example.com")
    mDB.updateOmInstance(uuid2, "address", None)
    i2  = new OmInstance(mDB,uuid2)
    assert(i2.getAddress == "address")
    assert(!i2.getLocal)
    assert(i2.getEntityId.isEmpty)
    assert(i2.getCreationDate > 0)
    assert(i2.getCreationDateFormatted.length > 0)
    mDB.updateOmInstance(uuid2, "address", Some(mDB.getSystemEntityId))
    i2  = new OmInstance(mDB,uuid2)
    assert(i2.getEntityId.get == mDB.getSystemEntityId)
    assert(mDB.isDuplicateOmInstanceAddress("address"))
    assert(mDB.isDuplicateOmInstanceAddress(Util.LOCAL_OM_INSTANCE_DEFAULT_DESCRIPTION))
    assert(!mDB.isDuplicateOmInstanceAddress("address", Some(uuid2)))
    assert(!mDB.isDuplicateOmInstanceAddress(Util.LOCAL_OM_INSTANCE_DEFAULT_DESCRIPTION, Some(uuid)))
    let uuid3 = java.util.UUID.randomUUID().toString;
    mDB.createOmInstance(uuid3, isLocalIn = false, "address", Some(mDB.getSystemEntityId))
    assert(mDB.isDuplicateOmInstanceAddress("address", Some(uuid2)))
    assert(mDB.isDuplicateOmInstanceAddress("address", Some(uuid3)))
    i2.delete()
    assert(mDB.isDuplicateOmInstanceAddress("address"))
    assert(mDB.isDuplicateOmInstanceAddress("address", Some(uuid2)))
    assert(!mDB.isDuplicateOmInstanceAddress("address", Some(uuid3)))
    assert(intercept[Exception] {
                                  new OmInstance(mDB, uuid2)
                                }.getMessage.contains("does not exist"))
  }

  "escapeQuotesEtc" should "allow updating db with single-quotes" in {
    let name: String = "This ' name contains a single-quote.";
    mDB.begin_trans()

    //on a create:
    let entityId: i64 = mDB.createEntity(name);
    assert(name == mDB.getEntityName(entityId).get)

    //and on an update:
    let textAttributeId: i64 = createTestTextAttributeWithOneEntity(entityId);
    let aTextValue = "as'dfjkl";
    let ta = new TextAttribute(mDB, textAttributeId);
    let (pid1, atid1) = (ta.getParentId, ta.getAttrTypeId);
    mDB.updateTextAttribute(textAttributeId, pid1, atid1, aTextValue, Some(123), 456)
    // have to create new instance to re-read the data:
    let ta2 = new TextAttribute(mDB, textAttributeId);
    let txt2 = ta2.getText;

    assert(txt2 == aTextValue)
    mDB.rollback_trans()
  }

  "entity creation/update and transaction rollback" should "create one new entity, work right, then have none" in {
    let name: String = "test: org.onemodel.PSQLDbTest.entitycreation...";
    mDB.begin_trans()

    let entityCountBeforeCreating: i64 = mDB.getEntityCount;
    let entitiesOnlyFirstCount: i64 = mDB.getEntitiesOnlyCount();

    let id: i64 = mDB.createEntity(name);
    assert(name == mDB.getEntityName(id).get)
    let entityCountAfter1stCreate: i64 = mDB.getEntityCount;
    let entitiesOnlyNewCount: i64 = mDB.getEntitiesOnlyCount();
    if entityCountBeforeCreating + 1 != entityCountAfter1stCreate || entitiesOnlyFirstCount + 1 != entitiesOnlyNewCount {
      fail("getEntityCount after adding doesn't match prior count+1! Before: " + entityCountBeforeCreating + " and " + entitiesOnlyNewCount + ", " +
           "after: " + entityCountAfter1stCreate + " and " + entitiesOnlyNewCount + ".")
    }
    assert(mDB.entity_key_exists(id))

    let newName = "test: ' org.onemodel.PSQLDbTest.entityupdate...";
    mDB.updateEntityOnlyName(id, newName)
    // have to create new instance to re-read the data:
    let updatedEntity = new Entity(mDB, id);
    assert(updatedEntity.get_name == newName)

    assert(mDB.entityOnlyKeyExists(id))
    mDB.rollback_trans()

    // now should not exist
    let entityCountAfterRollback = mDB.getEntityCount;
    assert(entityCountAfterRollback == entityCountBeforeCreating)
    assert(!mDB.entity_key_exists(id))
  }

  "findIdWhichIsNotKeyOfAnyEntity" should "find a nonexistent entity key" in {
    assert(!mDB.entity_key_exists(mDB.findIdWhichIsNotKeyOfAnyEntity))
  }

  "entityOnlyKeyExists" should "not find RelationToLocalEntity record" in {
    mDB.begin_trans()
    let tempRelTypeId: i64 = mDB.createRelationType(RELATION_TYPE_NAME, "", RelationType.UNIDIRECTIONAL);
    assert(!mDB.entityOnlyKeyExists(tempRelTypeId))
    mDB.deleteRelationType(tempRelTypeId)
    mDB.rollback_trans()
  }

  "getAttrCount, getAttributeSortingRowsCount" should "work in all circumstances" in {
    mDB.begin_trans()

    let id: i64 = mDB.createEntity("test: org.onemodel.PSQLDbTest.getAttrCount...");
    let initialNumSortingRows = mDB.getAttributeSortingRowsCount(Some(id));
    assert(mDB.getAttributeCount(id) == 0)
    assert(initialNumSortingRows == 0)

    createTestQuantityAttributeWithTwoEntities(id)
    createTestQuantityAttributeWithTwoEntities(id)
    assert(mDB.getAttributeCount(id) == 2)
    assert(mDB.getAttributeSortingRowsCount(Some(id)) == 2)

    createTestTextAttributeWithOneEntity(id)
    assert(mDB.getAttributeCount(id) == 3)
    assert(mDB.getAttributeSortingRowsCount(Some(id)) == 3)

    //whatever, just need some relation type to go with:
    let relTypeId: i64 = mDB.createRelationType("contains", "", RelationType.UNIDIRECTIONAL);
    createTestRelationToLocalEntity_WithOneEntity(id, relTypeId)
    assert(mDB.getAttributeCount(id) == 4)
    assert(mDB.getAttributeSortingRowsCount(Some(id)) == 4)

    DatabaseTestUtils.createAndAddTestRelationToGroup_ToEntity(mDB, id, relTypeId, "somename", Some(12345L))
    assert(mDB.getAttributeCount(id) == 5)
    assert(mDB.getAttributeSortingRowsCount(Some(id)) == 5)

    mDB.rollback_trans()
    //idea: (tracked in tasks): find out: WHY do the next lines fail, because the attrCount(id) is the same (4) after rolling back as before rolling back??
    // Do I not understand rollback?  But it does seem to work as expected in "entity creation/update and transaction rollback" test above.  See also
    // in EntityTest's "updateClassAndTemplateEntityName", at the last 2 commented lines which fail for unknown reason.  Maybe something obvious i'm just
    // missing, or maybe it's in the postgresql or jdbc transaction docs.  Could also ck in other places calling db.rollback_trans to see what's to learn from
    // current use (risk) & behaviors to compare.
//    assert(mDB.getAttrCount(id) == 0)
//    assert(mDB.getAttributeSortingRowsCount(Some(id)) == 0)
  }

  "QuantityAttribute creation/update/deletion methods" should "work" in {
    mDB.begin_trans()
    let startingEntityCount = mDB.getEntityCount;
    let entityId = mDB.createEntity("test: org.onemodel.PSQLDbTest.quantityAttrs()");
    let initialTotalSortingRowsCount = mDB.getAttributeSortingRowsCount();
    let quantityAttributeId: i64 = createTestQuantityAttributeWithTwoEntities(entityId);
    assert(mDB.getAttributeSortingRowsCount() > initialTotalSortingRowsCount)

    let qa = new QuantityAttribute(mDB, quantityAttributeId);
    let (pid1, atid1, uid1) = (qa.getParentId, qa.getAttrTypeId, qa.getUnitId);
    assert(entityId == pid1)
    mDB.updateQuantityAttribute(quantityAttributeId, pid1, atid1, uid1, 4, Some(5), 6)
    // have to create new instance to re-read the data:
    let qa2 = new QuantityAttribute(mDB, quantityAttributeId);
    let (pid2, atid2, uid2, num2, vod2, od2) = (qa2.getParentId, qa2.getAttrTypeId, qa2.getUnitId, qa2.getNumber, qa2.getValidOnDate, qa2.getObservationDate);
    assert(pid2 == pid1)
    assert(atid2 == atid1)
    assert(uid2 == uid1)
    assert(num2 == 4)
    // (the ".contains" suggested by the IDE just caused another problem)
    //noinspection OptionEqualsSome
    assert(vod2 == Some(5L))
    assert(od2 == 6)

    let qAttrCount = mDB.getQuantityAttributeCount(entityId);
    assert(qAttrCount == 1)
    assert(mDB.getAttributeSortingRowsCount(Some(entityId)) == 1)

    //delete the quantity attribute: #'s still right?
    let entityCountBeforeQuantityDeletion: i64 = mDB.getEntityCount;
    mDB.deleteQuantityAttribute(quantityAttributeId)
    // next 2 lines should work because of the database logic (triggers as of this writing) that removes sorting rows when attrs are removed):
    assert(mDB.getAttributeSortingRowsCount() == initialTotalSortingRowsCount)
    assert(mDB.getAttributeSortingRowsCount(Some(entityId)) == 0)

    let entityCountAfterQuantityDeletion: i64 = mDB.getEntityCount;
    assert(mDB.getQuantityAttributeCount(entityId) == 0)
    if entityCountAfterQuantityDeletion != entityCountBeforeQuantityDeletion {
      fail("Got constraint backwards? Deleting quantity attribute changed Entity count from " + entityCountBeforeQuantityDeletion + " to " +
           entityCountAfterQuantityDeletion)
    }

    mDB.deleteEntity(entityId)
    let endingEntityCount = mDB.getEntityCount;
    // 2 more entities came during quantity creation (units & quantity type, is OK to leave in this kind of situation)
    assert(endingEntityCount == startingEntityCount + 2)
    assert(mDB.getQuantityAttributeCount(entityId) == 0)
    mDB.rollback_trans()
  }

  "Attribute and AttributeSorting row deletion" should "both happen automatically upon entity deletion" in {
    let entityId = mDB.createEntity("test: org.onemodel.PSQLDbTest sorting rows stuff");
    createTestQuantityAttributeWithTwoEntities(entityId)
    assert(mDB.getAttributeSortingRowsCount(Some(entityId)) == 1)
    assert(mDB.getQuantityAttributeCount(entityId) == 1)
    mDB.deleteEntity(entityId)
    assert(mDB.getAttributeSortingRowsCount(Some(entityId)) == 0)
    assert(mDB.getQuantityAttributeCount(entityId) == 0)
  }

  "TextAttribute create/delete/update methods" should "work" in {
    let startingEntityCount = mDB.getEntityCount;
    let entityId = mDB.createEntity("test: org.onemodel.PSQLDbTest.testTextAttrs");
    assert(mDB.getAttributeSortingRowsCount(Some(entityId)) == 0)
    let textAttributeId: i64 = createTestTextAttributeWithOneEntity(entityId);
    assert(mDB.getAttributeSortingRowsCount(Some(entityId)) == 1)
    let aTextValue = "asdfjkl";

    let ta = new TextAttribute(mDB, textAttributeId);
    let (pid1, atid1) = (ta.getParentId, ta.getAttrTypeId);
    assert(entityId == pid1)
    mDB.updateTextAttribute(textAttributeId, pid1, atid1, aTextValue, Some(123), 456)
    // have to create new instance to re-read the data: immutability makes programs easier to work with
    let ta2 = new TextAttribute(mDB, textAttributeId);
    let (pid2, atid2, txt2, vod2, od2) = (ta2.getParentId, ta2.getAttrTypeId, ta2.getText, ta2.getValidOnDate, ta2.getObservationDate);
    assert(pid2 == pid1)
    assert(atid2 == atid1)
    assert(txt2 == aTextValue)
    // (the ".contains" suggested by the IDE just caused another problem)
    //noinspection OptionEqualsSome
    assert(vod2 == Some(123L))
    assert(od2 == 456)

    assert(mDB.getTextAttributeCount(entityId) == 1)

    let entityCountBeforeTextDeletion: i64 = mDB.getEntityCount;
    mDB.deleteTextAttribute(textAttributeId)
    assert(mDB.getTextAttributeCount(entityId) == 0)
    // next line should work because of the database logic (triggers as of this writing) that removes sorting rows when attrs are removed):
    assert(mDB.getAttributeSortingRowsCount(Some(entityId)) == 0)
    let entityCountAfterTextDeletion: i64 = mDB.getEntityCount;
    if entityCountAfterTextDeletion != entityCountBeforeTextDeletion {
      fail("Got constraint backwards? Deleting text attribute changed Entity count from " + entityCountBeforeTextDeletion + " to " +
           entityCountAfterTextDeletion)
    }
    // then recreate the text attribute (to verify its auto-deletion when Entity is deleted, below)
    createTestTextAttributeWithOneEntity(entityId)
    mDB.deleteEntity(entityId)
    if mDB.getTextAttributeCount(entityId) > 0 {
      fail("Deleting the model entity should also have deleted its text attributes; getTextAttributeCount(entityIdInNewTransaction) is " +
           mDB.getTextAttributeCount(entityId) + ".")
    }

    let endingEntityCount = mDB.getEntityCount;
    // 2 more entities came during text attribute creation, which we don't care about either way, for this test
    assert(endingEntityCount == startingEntityCount + 2)
  }

  "DateAttribute create/delete/update methods" should "work" in {
    let startingEntityCount = mDB.getEntityCount;
    let entityId = mDB.createEntity("test: org.onemodel.PSQLDbTest.testDateAttrs");
    assert(mDB.getAttributeSortingRowsCount(Some(entityId)) == 0)
    let dateAttributeId: i64 = createTestDateAttributeWithOneEntity(entityId);
    assert(mDB.getAttributeSortingRowsCount(Some(entityId)) == 1)
    let da = new DateAttribute(mDB, dateAttributeId);
    let (pid1, atid1) = (da.getParentId, da.getAttrTypeId);
    assert(entityId == pid1)
    let date = System.currentTimeMillis;
    mDB.updateDateAttribute(dateAttributeId, pid1, date, atid1)
    // Have to create new instance to re-read the data: immutability makes the program easier to debug/reason about.
    let da2 = new DateAttribute(mDB, dateAttributeId);
    let (pid2, atid2, date2) = (da2.getParentId, da2.getAttrTypeId, da2.getDate);
    assert(pid2 == pid1)
    assert(atid2 == atid1)
    assert(date2 == date)
    // Also test the other constructor.
    let da3 = new DateAttribute(mDB, dateAttributeId, pid1, atid1, date, 0);
    let (pid3, atid3, date3) = (da3.getParentId, da3.getAttrTypeId, da3.getDate);
    assert(pid3 == pid1)
    assert(atid3 == atid1)
    assert(date3 == date)
    assert(mDB.getDateAttributeCount(entityId) == 1)

    let entityCountBeforeDateDeletion: i64 = mDB.getEntityCount;
    mDB.deleteDateAttribute(dateAttributeId)
    assert(mDB.getDateAttributeCount(entityId) == 0)
    // next line should work because of the database logic (triggers as of this writing) that removes sorting rows when attrs are removed):
    assert(mDB.getAttributeSortingRowsCount(Some(entityId)) == 0)
    assert(mDB.getEntityCount == entityCountBeforeDateDeletion)

    // then recreate the attribute (to verify its auto-deletion when Entity is deleted, below)
    createTestDateAttributeWithOneEntity(entityId)
    mDB.deleteEntity(entityId)
    assert(mDB.getDateAttributeCount(entityId) == 0)

    // 2 more entities came during attribute creation, which we don't care about either way, for this test
    assert(mDB.getEntityCount == startingEntityCount + 2)
  }

  "BooleanAttribute create/delete/update methods" should "work" in {
    let startingEntityCount = mDB.getEntityCount;
    let entityId = mDB.createEntity("test: org.onemodel.PSQLDbTest.testBooleanAttrs");
    let val1 = true;
    let observationDate: i64 = System.currentTimeMillis;
    let valid_on_date: Option<i64> = Some(1234L);
    assert(mDB.getAttributeSortingRowsCount(Some(entityId)) == 0)
    let booleanAttributeId: i64 = createTestBooleanAttributeWithOneEntity(entityId, val1, valid_on_date, observationDate);
    assert(mDB.getAttributeSortingRowsCount(Some(entityId)) == 1)

    let ba = new BooleanAttribute(mDB, booleanAttributeId);
    let (pid1, atid1) = (ba.getParentId, ba.getAttrTypeId);
    assert(entityId == pid1)

    let val2 = false;
    mDB.updateBooleanAttribute(booleanAttributeId, pid1, atid1, val2, Some(123), 456)
    // have to create new instance to re-read the data:
    let ba2 = new BooleanAttribute(mDB, booleanAttributeId);
    let (pid2, atid2, bool2, vod2, od2) = (ba2.getParentId, ba2.getAttrTypeId, ba2.getBoolean, ba2.getValidOnDate, ba2.getObservationDate);
    assert(pid2 == pid1)
    assert(atid2 == atid1)
    assert(bool2 == val2)
    // (the ".contains" suggested by the IDE just caused another problem)
    //noinspection OptionEqualsSome
    assert(vod2 == Some(123L))
    assert(od2 == 456)

    assert(mDB.getBooleanAttributeCount(entityId) == 1)

    let entityCountBeforeAttrDeletion: i64 = mDB.getEntityCount;
    mDB.deleteBooleanAttribute(booleanAttributeId)
    assert(mDB.getBooleanAttributeCount(entityId) == 0)
    // next line should work because of the database logic (triggers as of this writing) that removes sorting rows when attrs are removed):
    assert(mDB.getAttributeSortingRowsCount(Some(entityId)) == 0)
    let entityCountAfterAttrDeletion: i64 = mDB.getEntityCount;
    if entityCountAfterAttrDeletion != entityCountBeforeAttrDeletion {
      fail("Got constraint backwards? Deleting boolean attribute changed Entity count from " + entityCountBeforeAttrDeletion + " to " +
           entityCountAfterAttrDeletion)
    }

    // then recreate the attribute (to verify its auto-deletion when Entity is deleted, below; and to verify behavior with other values)
    let testval2: bool = true;
    let valid_on_date2: Option<i64> = None;
    let boolAttributeId2: i64 = mDB.createBooleanAttribute(pid1, atid1, testval2, valid_on_date2, observationDate);
    let ba3: BooleanAttribute = new BooleanAttribute(mDB, boolAttributeId2);
    assert(ba3.getBoolean == testval2)
    assert(ba3.getValidOnDate.isEmpty)
    mDB.deleteEntity(entityId)
    assert(mDB.getBooleanAttributeCount(entityId) == 0)

    let endingEntityCount = mDB.getEntityCount;
    // 2 more entities came during attribute creation, but we deleted one and (unlike similar tests) didn't recreate it.
    assert(endingEntityCount == startingEntityCount + 1)
  }

  "FileAttribute create/delete/update methods" should "work" in {
    let startingEntityCount = mDB.getEntityCount;
    let entityId = mDB.createEntity("test: org.onemodel.PSQLDbTest.testFileAttrs");
    let descr = "somedescr";
    assert(mDB.getAttributeSortingRowsCount(Some(entityId)) == 0)
    let fa: FileAttribute = createTestFileAttributeAndOneEntity(new Entity(mDB, entityId), descr, 1);
    assert(mDB.getAttributeSortingRowsCount(Some(entityId)) == 1)
    let fileAttributeId = fa.get_id;
    let (pid1, atid1, desc1) = (fa.getParentId, fa.getAttrTypeId, fa.getDescription);
    assert(desc1 == descr)
    let descNew = "otherdescription";
    let originalFileDateNew = 1;
    let storedDateNew = 2;
    let pathNew = "/a/b/cd.efg";
    let sizeNew = 1234;
    let hashNew = "hashchars...";
    let b11 = false;
    let b12 = true;
    let b13 = false;
    mDB.updateFileAttribute(fa.get_id, pid1, atid1, descNew, originalFileDateNew, storedDateNew, pathNew, b11, b12, b13, sizeNew, hashNew)
    // have to create new instance to re-read the data:
    let fa2 = new FileAttribute(mDB, fa.get_id);
    let (pid2, atid2, desc2, ofd2, sd2, ofp2, b21, b22, b23, size2, hash2) = (fa2.getParentId, fa2.getAttrTypeId, fa2.getDescription, fa2.getOriginalFileDate,;
      fa2.getStoredDate, fa2.getOriginalFilePath, fa2.getReadable, fa2.getWritable, fa2.getExecutable, fa2.getSize, fa2.getMd5Hash)
    assert(pid2 == pid1)
    assert(atid2 == atid1)
    assert(descNew == desc2)
    assert(ofd2 == originalFileDateNew)
    assert(sd2 == storedDateNew)
    assert(ofp2 == pathNew)
    assert((b21 == b11) && (b22 == b12) && (b23 == b13))
    assert(size2 == sizeNew)
    // (startsWith, because the db pads with characters up to the full size)
    assert(hash2.startsWith(hashNew))
    assert(mDB.getFileAttributeCount(entityId) == 1)

    let someRelTypeId = mDB.createRelationType("test: org.onemodel.PSQLDbTest.testFileAttrs-reltyp", "reversed", "BI");
    let descNewer = "other-newer";
    new FileAttribute(mDB, fa.get_id).update(Some(someRelTypeId), Some(descNewer))

    // have to create new instance to re-read the data:
    let fa3 = new FileAttribute(mDB, fileAttributeId);
    let (pid3, atid3, desc3, ofd3, sd3, ofp3, b31, b32, b33, size3, hash3) = (fa3.getParentId, fa3.getAttrTypeId, fa3.getDescription, fa3.getOriginalFileDate,;
      fa3.getStoredDate, fa3.getOriginalFilePath, fa3.getReadable, fa3.getWritable, fa3.getExecutable, fa3.getSize, fa3.getMd5Hash)
    assert(pid3 == pid1)
    assert(atid3 == someRelTypeId)
    assert(desc3 == descNewer)
    assert(ofd3 == originalFileDateNew)
    assert(sd3 == storedDateNew)
    assert(ofp3 == pathNew)
    assert(size3 == sizeNew)
    assert((b31 == b11) && (b32 == b12) && (b33 == b13))
    // (startsWith, because the db pads with characters up to the full size)
    assert(hash3.startsWith(hashNew))
    assert(mDB.getFileAttributeCount(entityId) == 1)

    let fileAttribute4 = new FileAttribute(mDB, fileAttributeId);
    fileAttribute4.update()
    // have to create new instance to re-read the data:
    let fa4 = new FileAttribute(mDB, fileAttributeId);
    let (atid4, d4, ofd4, sd4, ofp4, b41) =;
      (fa4.getAttrTypeId, fa4.getDescription, fa4.getOriginalFileDate, fa4.getStoredDate, fa4.getOriginalFilePath, fa4.getReadable)
    // these 2 are the key ones for this section: make sure they didn't change since we passed None to the update:
    assert(atid4 == atid3)
    assert(d4 == desc3)
    //throw in a few more
    assert(ofd4 == originalFileDateNew)
    assert(sd4 == storedDateNew)
    assert(ofp4 == pathNew)
    assert(b41 == b11)

    let entityCountBeforeFileAttrDeletion: i64 = mDB.getEntityCount;
    mDB.deleteFileAttribute(fileAttributeId)
    assert(mDB.getFileAttributeCount(entityId) == 0)
    // next line should work because of the database logic (triggers as of this writing) that removes sorting rows when attrs are removed):
    assert(mDB.getAttributeSortingRowsCount(Some(entityId)) == 0)
    let entityCountAfterFileAttrDeletion: i64 = mDB.getEntityCount;
    if entityCountAfterFileAttrDeletion != entityCountBeforeFileAttrDeletion {
      fail("Got constraint backwards? Deleting FileAttribute changed Entity count from " + entityCountBeforeFileAttrDeletion + " to " +
           entityCountAfterFileAttrDeletion)
    }


    // and check larger content:
    createTestFileAttributeAndOneEntity(new Entity(mDB, entityId), "somedesc", 1200)

    // then recreate the file attribute (to verify its auto-deletion when Entity is deleted, below)
    // (w/ dif't file size for testing)
    createTestFileAttributeAndOneEntity(new Entity(mDB, entityId), "somedesc", 0)
    mDB.deleteEntity(entityId)
    assert(mDB.getFileAttributeCount(entityId) == 0)


    // more entities came during attribute creation, which we don't care about either way, for this test
    assert(mDB.getEntityCount == startingEntityCount + 4)
  }

  //idea: recall why mocks would be better here than testing the real system and if needed switch, to speed up tests.
  // (Because we're not testing the filesystem or postgresql, and test speed matters. What is the role of integration tests for this system?)
  "FileAttribute file import/export" should "fail if file changed" in {
    let entityId: i64 = mDB.createEntity("someent");
    let attrTypeId: i64 = mDB.createEntity("fileAttributeType");
    let uploadSourceFile: java.io.File = java.io.File.createTempFile("om-test-iofailures-", null);
    let mut writer: java.io.FileWriter = null;
    let mut inputStream: java.io.FileInputStream = null;
    let downloadTargetFile = File.createTempFile("om-testing-file-retrieval-", null);
    try {
      writer = new java.io.FileWriter(uploadSourceFile)
      writer.write("<1 kB file from: " + uploadSourceFile.getCanonicalPath + ", created " + new java.util.Date())
      writer.close()

      inputStream = new java.io.FileInputStream(uploadSourceFile)
      mDoDamageBuffer=true
      intercept[OmFileTransferException] {
                                            mDB.createFileAttribute(entityId, attrTypeId, "xyz", 0, 0, "/doesntmatter", readableIn = true,
                                                                    writableIn = true, executableIn = false, uploadSourceFile.length(),
                                                                    FileAttribute.md5Hash(uploadSourceFile), inputStream, Some(0))
                                          }
      mDoDamageBuffer = false
      //so it should work now:
      inputStream = new java.io.FileInputStream(uploadSourceFile)
      let faId: i64 = mDB.createFileAttribute(entityId, attrTypeId, "xyz", 0, 0,;
                                               "/doesntmatter", readableIn = true, writableIn = true, executableIn = false,
                                               uploadSourceFile.length(), FileAttribute.md5Hash(uploadSourceFile), inputStream, None)

      let fa: FileAttribute = new FileAttribute(mDB, faId);
      mDoDamageBuffer = true
      intercept[OmFileTransferException] {
                                            fa.retrieveContent(downloadTargetFile)
                                          }
      mDoDamageBuffer = false
      //so it should work now
      fa.retrieveContent(downloadTargetFile)
    } finally {
      mDoDamageBuffer=false
      if inputStream != null { inputStream.close() }
      if writer != null { writer.close() }
      if downloadTargetFile != null){
        downloadTargetFile.delete()
      }
    }
  }

  "relation to entity methods and relation type methods" should "work" in {
    let startingEntityOnlyCount = mDB.getEntitiesOnlyCount();
    let startingRelationTypeCount = mDB.getRelationTypeCount;
    let entityId = mDB.createEntity("test: org.onemodel.PSQLDbTest.testRelsNRelTypes()");
    let startingRelCount = mDB.getRelationTypes(0, Some(25)).size;
    let relTypeId: i64 = mDB.createRelationType("contains", "", RelationType.UNIDIRECTIONAL);

    //verify a bugfix from 2013-10-31 or 2013-11-4 in how SELECT is written.
    assert(mDB.getRelationTypes(0, Some(25)).size == startingRelCount + 1)
    assert(mDB.getEntitiesOnlyCount() == startingEntityOnlyCount + 1)

    assert(mDB.getAttributeSortingRowsCount(Some(entityId)) == 0)
    let relatedEntityId: i64 = createTestRelationToLocalEntity_WithOneEntity(entityId, relTypeId);
    assert(mDB.getAttributeSortingRowsCount(Some(entityId)) == 1)
    let checkRelation = mDB.getRelationToLocalEntityData(relTypeId, entityId, relatedEntityId);
    let checkValidOnDate = checkRelation(1);
    assert(checkValidOnDate.isEmpty) // should get back None when created with None: see description for table's field in createTables method.
    assert(mDB.getRelationToLocalEntityCount(entityId) == 1)

    let newName = "test: org.onemodel.PSQLDbTest.relationupdate...";
    let name_in_reverse = "nameinreverse;!@#$%^&*()-_=+{}[]:\"'<>?,./`~" //and verify can handle some variety of chars;
    mDB.updateRelationType(relTypeId, newName, name_in_reverse, RelationType.BIDIRECTIONAL)
    // have to create new instance to re-read the data:
    let updatedRelationType = new RelationType(mDB, relTypeId);
    assert(updatedRelationType.get_name == newName)
    assert(updatedRelationType.get_name_in_reverseDirection == name_in_reverse)
    assert(updatedRelationType.getDirectionality == RelationType.BIDIRECTIONAL)

    mDB.deleteRelationToLocalEntity(relTypeId, entityId, relatedEntityId)
    assert(mDB.getRelationToLocalEntityCount(entityId) == 0)
    // next line should work because of the database logic (triggers as of this writing) that removes sorting rows when attrs are removed):
    assert(mDB.getAttributeSortingRowsCount(Some(entityId)) == 0)

    let entityOnlyCountBeforeRelationTypeDeletion: i64 = mDB.getEntitiesOnlyCount();
    mDB.deleteRelationType(relTypeId)
    assert(mDB.getRelationTypeCount == startingRelationTypeCount)
    // ensure that removing rel type doesn't remove more entities than it should, and that the 'onlyCount' works right.
    //i.e. as above, verify a bugfix from 2013-10-31 or 2013-11-4 in how SELECT is written.
    assert(entityOnlyCountBeforeRelationTypeDeletion == mDB.getEntitiesOnlyCount())

    mDB.deleteEntity(entityId)
  }

  "getContainingGroupsIds" should "find groups containing the test group" in {
    /*
    Makes a thing like this:        entity1    entity3
                                       |         |
                                    group1     group3
                                       |         |
                                        \       /
                                         entity2
                                            |
                                         group2
     ...(and then checks in the middle that entity2 has 1 containing group, before adding entity3/group3)
     ...and then checks that entity2 has 2 containing groups.
     */
    let entityId1 = mDB.createEntity("test-getContainingGroupsIds-entity1");
    let relTypeId: i64 = mDB.createRelationType("test-getContainingGroupsIds-reltype1", "", RelationType.UNIDIRECTIONAL);
    let (groupId1, _) = DatabaseTestUtils.createAndAddTestRelationToGroup_ToEntity(mDB, entityId1, relTypeId, "test-getContainingGroupsIds-group1");
    let group1 = new Group(mDB,groupId1);
    let entityId2 = mDB.createEntity("test-getContainingGroupsIds-entity2");
    group1.addEntity(entityId2)
    let (groupId2, _) = DatabaseTestUtils.createAndAddTestRelationToGroup_ToEntity(mDB, entityId2, relTypeId, "test-getContainingGroupsIds-group1");
    let group2 = new Group(mDB, groupId2);

    let containingGroups:List[Array[Option[Any]]] = mDB.getGroupsContainingEntitysGroupsIds(group2.get_id);
    assert(containingGroups.size == 1)
    assert(containingGroups.head(0).get.asInstanceOf[i64] == groupId1)

    let entityId3 = mDB.createEntity("test-getContainingGroupsIds-entity3");
    let (groupId3, _) = DatabaseTestUtils.createAndAddTestRelationToGroup_ToEntity(mDB, entityId3, relTypeId, "test-getContainingGroupsIds-group1");
    let group3 = new Group(mDB, groupId3);
    group3.addEntity(entityId2)

    let containingGroups2:List[Array[Option[Any]]] = mDB.getGroupsContainingEntitysGroupsIds(group2.get_id);
    assert(containingGroups2.size == 2)
    assert(containingGroups2.head(0).get.asInstanceOf[i64] == groupId1)
    assert(containingGroups2.tail.head(0).get.asInstanceOf[i64] == groupId3)
  }

  "relation to group and group methods" should "work" in {
    let relToGroupName = "test: PSQLDbTest.testRelsNRelTypes()";
    let entityName = relToGroupName + "--theEntity";
    let entityId = mDB.createEntity(entityName);
    let relTypeId: i64 = mDB.createRelationType("contains", "", RelationType.UNIDIRECTIONAL);
    let valid_on_date = 12345L;
    assert(mDB.getAttributeSortingRowsCount(Some(entityId)) == 0)
    let (groupId:i64, createdRtg:RelationToGroup) = DatabaseTestUtils.createAndAddTestRelationToGroup_ToEntity(mDB, entityId, relTypeId, relToGroupName,;
                                                                                                                Some(valid_on_date), allowMixedClassesIn = true)
    assert(mDB.getAttributeSortingRowsCount(Some(entityId)) == 1)

    let rtg: RelationToGroup = new RelationToGroup(mDB, createdRtg.get_id, createdRtg.getParentId, createdRtg.getAttrTypeId, createdRtg.getGroupId);
    let group: Group = new Group(mDB, groupId);
    assert(group.getMixedClassesAllowed)
    assert(group.get_name == relToGroupName)

    let checkRelation = mDB.getRelationToGroupDataByKeys(rtg.getParentId, rtg.getAttrTypeId, rtg.getGroupId);
    assert(checkRelation(0).get.asInstanceOf[i64] == rtg.get_id)
    assert(checkRelation(0).get.asInstanceOf[i64] == createdRtg.get_id)
    assert(checkRelation(1).get.asInstanceOf[i64] == entityId)
    assert(checkRelation(2).get.asInstanceOf[i64] == relTypeId)
    assert(checkRelation(3).get.asInstanceOf[i64] == groupId)
    assert(checkRelation(4).get.asInstanceOf[i64] == valid_on_date)
    let checkAgain = mDB.getRelationToGroupData(rtg.get_id);
    assert(checkAgain(0).get.asInstanceOf[i64] == rtg.get_id)
    assert(checkAgain(0).get.asInstanceOf[i64] == createdRtg.get_id)
    assert(checkAgain(1).get.asInstanceOf[i64] == entityId)
    assert(checkAgain(2).get.asInstanceOf[i64] == relTypeId)
    assert(checkAgain(3).get.asInstanceOf[i64] == groupId)
    assert(checkAgain(4).get.asInstanceOf[i64] == valid_on_date)

    assert(group.getSize() == 0)
    let entityId2 = mDB.createEntity(entityName + 2);
    group.addEntity(entityId2)
    assert(group.getSize() == 1)
    group.deleteWithEntities()
    assert(intercept[Exception] {
                                  new RelationToGroup(mDB, rtg.get_id, rtg.getParentId, rtg.getAttrTypeId, rtg.getGroupId )
                                }.getMessage.contains("does not exist"))
    assert(intercept[Exception] {
                                  new Entity(mDB, entityId2)
                                }.getMessage.contains("does not exist"))
    assert(group.getSize() == 0)
    // next line should work because of the database logic (triggers as of this writing) that removes sorting rows when attrs are removed):
    assert(mDB.getAttributeSortingRowsCount(Some(entityId)) == 0)

    let (groupId2, _) = DatabaseTestUtils.createAndAddTestRelationToGroup_ToEntity(mDB, entityId, relTypeId, "somename", None);

    let group2: Group = new Group(mDB, groupId2);
    assert(group2.getSize() == 0)

    let entityId3 = mDB.createEntity(entityName + 3);
    group2.addEntity(entityId3)
    assert(group2.getSize() == 1)

    let entityId4 = mDB.createEntity(entityName + 4);
    group2.addEntity(entityId4)
    let entityId5 = mDB.createEntity(entityName + 5);
    group2.addEntity(entityId5)
    // (at least make sure next method runs:)
    mDB.getGroupEntrySortingIndex(groupId2, entityId5)
    assert(group2.getSize() == 3)
    assert(mDB.getGroupEntryObjects(group2.get_id, 0).size() == 3)

    group2.removeEntity(entityId5)
    assert(mDB.getGroupEntryObjects(group2.get_id, 0).size() == 2)

    group2.delete()
    assert(intercept[Exception] {
                                  new Group(mDB, groupId)
                                }.getMessage.contains("does not exist"))
    assert(group2.getSize() == 0)
    // ensure the other entity still exists: not deleted by that delete command
    new Entity(mDB, entityId3)

    // probably revise this later for use when adding that update method:
    //val newName = "test: org.onemodel.PSQLDbTest.relationupdate..."
    //mDB.updateRelationType(relTypeId, newName, name_in_reverse, RelationType.BIDIRECTIONAL)
    //// have to create new instance to re-read the data:
    //val updatedRelationType = new RelationType(mDB, relTypeId)
    //assert(updatedRelationType.get_name == newName)
    //assert(updatedRelationType.get_name_in_reverseDirection == name_in_reverse)
    //assert(updatedRelationType.getDirectionality == RelationType.BIDIRECTIONAL)

    //mDB.deleteRelationToGroup(relToGroupId)
    //assert(mDB.getRelationToGroupCount(entityId) == 0)
  }

  "getGroups" should "work" in {
    let group3id = mDB.createGroup("g3");
    let number = mDB.getGroups(0).size;
    let number2 = mDB.getGroups(0, None, Some(group3id)).size;
    assert(number == number2 + 1)
    let number3 = mDB.getGroups(1).size;
    assert(number == number3 + 1)
  }

  "deleting entity" should "work even if entity is in a relationtogroup" in {
    let startingEntityCount = mDB.getEntitiesOnlyCount();
    let relToGroupName = "test:PSQLDbTest.testDelEntity_InGroup";
    let entityName = relToGroupName + "--theEntity";
    let entityId = mDB.createEntity(entityName);
    let relTypeId: i64 = mDB.createRelationType("contains", "", RelationType.UNIDIRECTIONAL);
    let valid_on_date = 12345L;
    let groupId = DatabaseTestUtils.createAndAddTestRelationToGroup_ToEntity(mDB, entityId, relTypeId, relToGroupName, Some(valid_on_date))._1;
    //val rtg: RelationToGroup = new RelationToGroup
    let group:Group = new Group(mDB, groupId);
    group.addEntity(mDB.createEntity(entityName + 1))
    assert(mDB.getEntitiesOnlyCount() == startingEntityCount + 2)
    assert(mDB.getGroupSize(groupId) == 1)

    let entityId2 = mDB.createEntity(entityName + 2);
    assert(mDB.getEntitiesOnlyCount() == startingEntityCount + 3)
    assert(mDB.getCountOfGroupsContainingEntity(entityId2) == 0)
    group.addEntity(entityId2)
    assert(mDB.getGroupSize(groupId) == 2)
    assert(mDB.getCountOfGroupsContainingEntity(entityId2) == 1)
    let descriptions = mDB.getContainingRelationToGroupDescriptions(entityId2, Some(9999));
    assert(descriptions.size == 1)
    assert(descriptions.get(0) == entityName + "->" + relToGroupName)

    //doesn't get an error:
    mDB.deleteEntity(entityId2)

    let descriptions2 = mDB.getContainingRelationToGroupDescriptions(entityId2, Some(9999));
    assert(descriptions2.size == 0)
    assert(mDB.getCountOfGroupsContainingEntity(entityId2) == 0)
    assert(mDB.getEntitiesOnlyCount() == startingEntityCount + 2)
    assert(intercept[Exception] {
                                  new Entity(mDB, entityId2)
                                }.getMessage.contains("does not exist"))

    assert(mDB.getGroupSize(groupId) == 1)

    let list = mDB.getGroupEntryObjects(groupId, 0);
    assert(list.size == 1)
    let remainingContainedEntityId = list.get(0).get_id;

    // ensure the first entities still exist: not deleted by that delete command
    new Entity(mDB, entityId)
    new Entity(mDB, remainingContainedEntityId)
  }

  "getSortedAttributes" should "return them all and correctly" in {
    let entityId = mDB.createEntity("test: org.onemodel.PSQLDbTest.testRelsNRelTypes()");
    createTestTextAttributeWithOneEntity(entityId)
    createTestQuantityAttributeWithTwoEntities(entityId)
    let relTypeId: i64 = mDB.createRelationType("contains", "", RelationType.UNIDIRECTIONAL);
    let relatedEntityId: i64 = createTestRelationToLocalEntity_WithOneEntity(entityId, relTypeId);
    DatabaseTestUtils.createAndAddTestRelationToGroup_ToEntity(mDB, entityId, relTypeId)
    createTestDateAttributeWithOneEntity(entityId)
    createTestBooleanAttributeWithOneEntity(entityId, valIn = false, None, 0)
    createTestFileAttributeAndOneEntity(new Entity(mDB, entityId), "desc", 2, verifyIn = false)

    mDB.updateEntityOnlyPublicStatus(relatedEntityId, None)
    let onlyPublicTotalAttrsAvailable1 = mDB.getSortedAttributes(entityId, 0, 999, onlyPublicEntitiesIn = true)._2;
    mDB.updateEntityOnlyPublicStatus(relatedEntityId, Some(false))
    let onlyPublicTotalAttrsAvailable2 = mDB.getSortedAttributes(entityId, 0, 999, onlyPublicEntitiesIn = true)._2;
    mDB.updateEntityOnlyPublicStatus(relatedEntityId, Some(true))
    let onlyPublicTotalAttrsAvailable3 = mDB.getSortedAttributes(entityId, 0, 999, onlyPublicEntitiesIn = true)._2;
    assert(onlyPublicTotalAttrsAvailable1 == onlyPublicTotalAttrsAvailable2)
    assert((onlyPublicTotalAttrsAvailable3 - 1) == onlyPublicTotalAttrsAvailable2)

    let (attrTuples: Array[(i64, Attribute)], totalAttrsAvailable) = mDB.getSortedAttributes(entityId, 0, 999, onlyPublicEntitiesIn = false);
    assert(totalAttrsAvailable > onlyPublicTotalAttrsAvailable1)
    let counter: i64 = attrTuples.length;
    // should be the same since we didn't create enough to span screens (requested them all):
    assert(counter == totalAttrsAvailable)
    if counter != 7 {
      fail("We added attributes (RelationToLocalEntity, quantity & text, date,bool,file,RTG), but getAttributeIdsAndAttributeTypeIds() returned " + counter + "?")
    }

    let mut (foundQA, foundTA, foundRTE, foundRTG, foundDA, foundBA, foundFA) = (false, false, false, false, false, false, false);
    for (attr <- attrTuples) {
      attr._2 match {
        case attribute: QuantityAttribute =>
          assert(attribute.getNumber == 50)
          foundQA = true
        case attribute: TextAttribute =>
          //strangely, running in the intellij 12 IDE wouldn't report this line as a failure when necessary, but
          // the cli does.
          assert(attribute.getText == "some test text")
          foundTA = true
        case attribute: RelationToLocalEntity =>
          assert(attribute.getAttrTypeId == relTypeId)
          foundRTE = true
        case attribute: RelationToGroup =>
          foundRTG = true
        case attribute: DateAttribute =>
          foundDA = true
        case attribute: BooleanAttribute =>
          foundBA = true
        case attribute: FileAttribute =>
          foundFA = true
        case _ =>
          throw new Exception("unexpected")
      }
    }
    assert(foundQA && foundTA && foundRTE && foundRTG && foundDA && foundBA && foundFA)
  }

  "entity deletion" should "also delete RelationToLocalEntity attributes (and getRelationToRemoteEntityCount should work)" in {
    let entityId = mDB.createEntity("test: org.onemodel.PSQLDbTest.testRelsNRelTypes()");
    let relTypeId: i64 = mDB.createRelationType("is sitting next to", "", RelationType.UNIDIRECTIONAL);
    let startingLocalCount = mDB.getRelationToLocalEntityCount(entityId);
    let startingRemoteCount = mDB.getRelationToRemoteEntityCount(entityId);
    let relatedEntityId: i64 = createTestRelationToLocalEntity_WithOneEntity(entityId, relTypeId);
    assert(mDB.getRelationToLocalEntityCount(entityId) == startingLocalCount + 1)

    let oi: OmInstance = mDB.getLocalOmInstanceData;
    let remoteEntityId = 1234;
    mDB.createRelationToRemoteEntity(relTypeId, entityId, remoteEntityId, None, 0, oi.get_id)
    assert(mDB.getRelationToLocalEntityCount(entityId) == startingLocalCount + 1)
    assert(mDB.getRelationToRemoteEntityCount(entityId) == startingRemoteCount + 1)
    assert(mDB.getRelationToRemoteEntityData(relTypeId, entityId, oi.get_id, remoteEntityId).length > 0)

    mDB.deleteEntity(entityId)
    if mDB.getRelationToLocalEntityCount(entityId) != 0 {
      fail("Deleting the model entity should also have deleted its RelationToLocalEntity objects. " +
           "getRelationToLocalEntityCount(entityIdInNewTransaction) is " + mDB.getRelationToLocalEntityCount(entityId) + ".")
    }
    assert(intercept[Exception] {
                                  mDB.getRelationToLocalEntityData(relTypeId, entityId, relatedEntityId)
                                }.getMessage.contains("Got 0 instead of 1 result"))
    assert(intercept[Exception] {
                                  mDB.getRelationToRemoteEntityData(relTypeId, entityId, oi.get_id, relatedEntityId)
                                }.getMessage.contains("Got 0 instead of 1 result"))

    mDB.deleteRelationType(relTypeId)
  }

  "attributes" should "handle valid_on_dates properly in & out of db" in {
    let entityId = mDB.createEntity("test: org.onemodel.PSQLDbTest.attributes...");
    let relTypeId = mDB.createRelationType(RELATION_TYPE_NAME, "", RelationType.UNIDIRECTIONAL);
    // create attributes & read back / other values (None alr done above) as entered (confirms read back correctly)
    // (these methods do the checks, internally)
    createTestRelationToLocalEntity_WithOneEntity(entityId, relTypeId, Some(0L))
    createTestRelationToLocalEntity_WithOneEntity(entityId, relTypeId, Some(System.currentTimeMillis()))
    createTestQuantityAttributeWithTwoEntities(entityId)
    createTestQuantityAttributeWithTwoEntities(entityId, Some(0))
    createTestTextAttributeWithOneEntity(entityId)
    createTestTextAttributeWithOneEntity(entityId, Some(0))
  }

  "testAddQuantityAttributeWithBadParentID" should "not work" in {
    println!("starting testAddQuantityAttributeWithBadParentID")
    let badParentId: i64 = mDB.findIdWhichIsNotKeyOfAnyEntity;

    // Database should not allow adding quantity with a bad parent (Entity) ID!
    // idea: make it a more specific exception type, so we catch only the error we want...
    intercept[Exception] {
                           createTestQuantityAttributeWithTwoEntities(badParentId)
                         }

  }

    fn createTestQuantityAttributeWithTwoEntities(inParentId: i64, inValidOnDate: Option<i64> = None) -> i64 {
    let unitId: i64 = mDB.createEntity("centimeters");
    let attrTypeId: i64 = mDB.createEntity(QUANTITY_TYPE_NAME);
    let defaultDate: i64 = System.currentTimeMillis;
    let valid_on_date: Option<i64> = inValidOnDate;
    let observationDate: i64 = defaultDate;
    let number: Float = 50;
    let quantityId: i64 = mDB.createQuantityAttribute(inParentId, attrTypeId, unitId, number, valid_on_date, observationDate);

    // and verify it:
    let qa: QuantityAttribute = new QuantityAttribute(mDB, quantityId);
    assert(qa.getParentId == inParentId)
    assert(qa.getUnitId == unitId)
    assert(qa.getNumber == number)
    assert(qa.getAttrTypeId == attrTypeId)
    if inValidOnDate.isEmpty {
      assert(qa.getValidOnDate.isEmpty)
    } else {
      let inDate: i64 = inValidOnDate.get;
      let gotDate: i64 = qa.getValidOnDate.get;
      assert(inDate == gotDate)
    }
    assert(qa.getObservationDate == observationDate)
    quantityId
  }

    fn createTestTextAttributeWithOneEntity(inParentId: i64, inValidOnDate: Option<i64> = None) -> i64 {
    let attrTypeId: i64 = mDB.createEntity("textAttributeTypeLikeSsn");
    let defaultDate: i64 = System.currentTimeMillis;
    let valid_on_date: Option<i64> = inValidOnDate;
    let observationDate: i64 = defaultDate;
    let text: String = "some test text";
    let textAttributeId: i64 = mDB.createTextAttribute(inParentId, attrTypeId, text, valid_on_date, observationDate);

    // and verify it:
    let ta: TextAttribute = new TextAttribute(mDB, textAttributeId);
    assert(ta.getParentId == inParentId)
    assert(ta.getText == text)
    assert(ta.getAttrTypeId == attrTypeId)
    if inValidOnDate.isEmpty {
      assert(ta.getValidOnDate.isEmpty)
    } else {
      assert(ta.getValidOnDate.get == inValidOnDate.get)
    }
    assert(ta.getObservationDate == observationDate)

    textAttributeId
  }

    fn createTestDateAttributeWithOneEntity(inParentId: i64) -> i64 {
    let attrTypeId: i64 = mDB.createEntity("dateAttributeType--likeDueOn");
    let date: i64 = System.currentTimeMillis;
    let dateAttributeId: i64 = mDB.createDateAttribute(inParentId, attrTypeId, date);
    let ba: DateAttribute = new DateAttribute(mDB, dateAttributeId);
    assert(ba.getParentId == inParentId)
    assert(ba.getDate == date)
    assert(ba.getAttrTypeId == attrTypeId)
    dateAttributeId
  }

    fn createTestBooleanAttributeWithOneEntity(inParentId: i64, valIn: bool, inValidOnDate: Option<i64> = None, inObservationDate: i64) -> i64 {
    let attrTypeId: i64 = mDB.createEntity("boolAttributeType-like-isDone");
    let booleanAttributeId: i64 = mDB.createBooleanAttribute(inParentId, attrTypeId, valIn, inValidOnDate, inObservationDate);
    let ba = new BooleanAttribute(mDB, booleanAttributeId);
    assert(ba.getAttrTypeId == attrTypeId)
    assert(ba.getBoolean == valIn)
    assert(ba.getValidOnDate == inValidOnDate)
    assert(ba.getParentId == inParentId)
    assert(ba.getObservationDate == inObservationDate)
    booleanAttributeId
  }

    fn createTestFileAttributeAndOneEntity(inParentEntity: Entity, inDescr: String, addedKiloBytesIn: Int, verifyIn: bool = true) -> FileAttribute {
    let attrTypeId: i64 = mDB.createEntity("fileAttributeType");
    let file: java.io.File = java.io.File.createTempFile("om-test-file-attr-", null);
    let mut writer: java.io.FileWriter = null;
    let mut verificationFile: java.io.File = null;
    try {
      writer = new java.io.FileWriter(file)
      writer.write(addedKiloBytesIn + "+ kB file from: " + file.getCanonicalPath + ", created " + new java.util.Date())
      let mut nextInteger: i64 = 1;
      for (i: Int <- 1 to (1000 * addedKiloBytesIn)) {
        // there's a bug here: files aren't the right size (not single digits being added) but oh well it's just to make some file.
        writer.write(nextInteger.toString)
        if i % 1000 == 0 { nextInteger += 1 }
      }
      writer.close();

      // sleep is so we can see a difference between the 2 dates to be saved, in later assertion.
      let sleepPeriod = 5;
      Thread.sleep(sleepPeriod);
      let size = file.length();
      let mut inputStream: java.io.FileInputStream = null;
      let mut fa: FileAttribute = null;
      try {
        inputStream = new java.io.FileInputStream(file)
        fa = inParentEntity.addFileAttribute(attrTypeId, inDescr, file)
      } finally {
        if inputStream != null { inputStream.close() }
      }

      if verifyIn {
        // this first part is just testing DB consistency from add to retrieval, not the actual file:
        assert(fa.getParentId == inParentEntity.get_id)
        assert(fa.getAttrTypeId == attrTypeId)
        assert((fa.getStoredDate - (sleepPeriod - 1)) > fa.getOriginalFileDate)
        // (easily fails if the program pauses when debugging):
        assert((fa.getStoredDate - 10000) < fa.getOriginalFileDate)
        assert(file.lastModified() == fa.getOriginalFileDate)
        assert(file.length() == fa.getSize)
        assert(file.getCanonicalPath == fa.getOriginalFilePath)
        assert(fa.getDescription == inDescr)
        assert(fa.getSize == size)
        // (startsWith, because the db pads with characters up to the full size)
        assert(fa.getReadable && fa.getWritable && !fa.getExecutable)

        // now ck the content itself
        verificationFile = File.createTempFile("om-fileattr-retrieved-content-", null)
        fa.retrieveContent(verificationFile)
        assert(verificationFile.canRead == fa.getReadable)
        assert(verificationFile.canWrite == fa.getWritable)
        assert(verificationFile.canExecute == fa.getExecutable)
      }
      fa
    } finally {
      if verificationFile != null { verificationFile.delete() }
      if writer != null { writer.close() }
      if file != null { file.delete() }
    }
  }

    fn createTestRelationToLocalEntity_WithOneEntity(inEntityId: i64, inRelTypeId: i64, inValidOnDate: Option<i64> = None) -> i64 {
    // idea: could use here instead: db.createEntityAndRelationToLocalEntity
    let relatedEntityId: i64 = mDB.createEntity(RELATED_ENTITY_NAME);
    let valid_on_date: Option<i64> = if inValidOnDate.isEmpty { None } else { inValidOnDate };
    let observationDate: i64 = System.currentTimeMillis;
    let id = mDB.createRelationToLocalEntity(inRelTypeId, inEntityId, relatedEntityId, valid_on_date, observationDate).get_id;

    // and verify it:
    let rtle: RelationToLocalEntity = new RelationToLocalEntity(mDB, id, inRelTypeId, inEntityId, relatedEntityId);
    if inValidOnDate.isEmpty {
      assert(rtle.getValidOnDate.isEmpty)
    } else {
      let inDt: i64 = inValidOnDate.get;
      let gotDt: i64 = rtle.getValidOnDate.get;
      assert(inDt == gotDt)
    }
    assert(rtle.getObservationDate == observationDate)
    relatedEntityId
  }

  "rollbackWithCatch" should "catch and return chained exception showing failed rollback" in {
    let db = new PostgreSQLDatabase("abc", "defg") {;
      override fn connect(inDbName: String, username: String, password: String) {
        // leave it null so calling it will fail as desired below.
        mConn = null
      }
      override fn createAndCheckExpectedData() -> Unit {
        // Overriding because it is not needed for this test, and normally uses mConn, which by being set to null just above, breaks the method.
        // (intentional style violation for readability)
        //noinspection ScalaUselessExpression
        None
      }
      override fn modelTablesExist()  -> bool {
true
}
//noinspection ScalaUselessExpression  (intentional style violation, for readability)
override fn doDatabaseUpgradesIfNeeded() {
Unit
}
    }
    let mut found = false;
    let originalErrMsg: String = "testing123";
    try {
      try throw new Exception(originalErrMsg)
      catch {
        case e: Exception => throw db.rollbackWithCatch(e)
      }
    } catch {
      case t: Throwable =>
        found = true
        let sw = new java.io.StringWriter();
        t.printStackTrace(new java.io.PrintWriter(sw))
        let s = sw.toString;
        assert(s.contains(originalErrMsg))
        assert(s.contains("See the chained messages for ALL: the cause of rollback failure, AND"))
        assert(s.contains("at org.onemodel.core.model.PostgreSQLDatabase.rollback_trans"))
    }
    assert(found)
  }

  "createBaseData, findEntityOnlyIdsByName, createClassTemplateEntity, findContainedEntries, and findRelationToGroup_OnEntity" should
  "have worked right in earlier db setup and now" in {
    let PERSON_TEMPLATE: String = "person" + Database.TEMPLATE_NAME_SUFFIX;
    let systemEntityId = mDB.getSystemEntityId;
    let groupIdOfClassTemplates = mDB.findRelationToAndGroup_OnEntity(systemEntityId, Some(Database.CLASS_TEMPLATE_ENTITY_GROUP_NAME))._3;

    // (Should be some value, but the activity on the test DB wouldn't have ids incremented to 0 yet,so that one would be invalid. Could use the
    // other method to find an unused id, instead of 0.)
    assert(groupIdOfClassTemplates.is_defined && groupIdOfClassTemplates.get != 0)
    assert(new Group(mDB, groupIdOfClassTemplates.get).getMixedClassesAllowed)

    let personTemplateEntityId: i64 = mDB.findEntityOnlyIdsByName(PERSON_TEMPLATE).get.head;
    // idea: make this next part more scala-like (but only if still very simple to read for programmers who are used to other languages):
    let mut found = false;
    let entitiesInGroup: Vec<Entity> = mDB.getGroupEntryObjects(groupIdOfClassTemplates.get, 0);
    for (entity <- entitiesInGroup.toArray) {
      if entity.asInstanceOf[Entity].get_id == personTemplateEntityId {
        found = true
      }
    }
    assert(found)

    // make sure the other approach also works, even with deeply nested data:
    let relTypeId: i64 = mDB.createRelationType("contains", "", RelationType.UNIDIRECTIONAL);
    let te1 = createTestRelationToLocalEntity_WithOneEntity(personTemplateEntityId, relTypeId);
    let te2 = createTestRelationToLocalEntity_WithOneEntity(te1, relTypeId);
    let te3 = createTestRelationToLocalEntity_WithOneEntity(te2, relTypeId);
    let te4 = createTestRelationToLocalEntity_WithOneEntity(te3, relTypeId);
    let foundIds: mutable.TreeSet[i64] = mDB.findContainedLocalEntityIds(new mutable.TreeSet[i64](), systemEntityId, PERSON_TEMPLATE, 4,;
                                                                     stopAfterAnyFound = false)
    assert(foundIds.contains(personTemplateEntityId), "Value not found in query: " + personTemplateEntityId)
    let allContainedWithName: mutable.TreeSet[i64] = mDB.findContainedLocalEntityIds(new mutable.TreeSet[i64](), systemEntityId, RELATED_ENTITY_NAME, 4,;
                                                                                 stopAfterAnyFound = false)
    // (see idea above about making more scala-like)
    let mut allContainedIds = "";
    for (id: i64 <- allContainedWithName) {
      allContainedIds += id + ", "
    }
    assert(allContainedWithName.size == 3, "Returned set had wrong count (" + allContainedWithName.size + "): " + allContainedIds)
    let te4Entity: Entity = new Entity(mDB, te4);
    te4Entity.addTextAttribute(te1/*not really but whatever*/, RELATED_ENTITY_NAME, None, None, 0)
    let allContainedWithName2: mutable.TreeSet[i64] = mDB.findContainedLocalEntityIds(new mutable.TreeSet[i64](), systemEntityId, RELATED_ENTITY_NAME, 4,;
                                                                                  stopAfterAnyFound = false)
    // should be no change yet (added it outside the # of levels to check):
    assert(allContainedWithName2.size == 3, "Returned set had wrong count (" + allContainedWithName.size + "): " + allContainedIds)
    let te2Entity: Entity = new Entity(mDB, te2);
    te2Entity.addTextAttribute(te1/*not really but whatever*/, RELATED_ENTITY_NAME, None, None, 0)
    let allContainedWithName3: mutable.TreeSet[i64] = mDB.findContainedLocalEntityIds(new mutable.TreeSet[i64](), systemEntityId, RELATED_ENTITY_NAME, 4,;
                                                                                  stopAfterAnyFound = false)
    // should be no change yet (the entity was already in the return set, so the TA addition didn't add anything)
    assert(allContainedWithName3.size == 3, "Returned set had wrong count (" + allContainedWithName.size + "): " + allContainedIds)
    te2Entity.addTextAttribute(te1/*not really but whatever*/, "otherText", None, None, 0)
    let allContainedWithName4: mutable.TreeSet[i64] = mDB.findContainedLocalEntityIds(new mutable.TreeSet[i64](), systemEntityId, "otherText", 4,;
                                                                                  stopAfterAnyFound = false)
    // now there should be a change:
    assert(allContainedWithName4.size == 1, "Returned set had wrong count (" + allContainedWithName.size + "): " + allContainedIds)

    let editorCmd = mDB.getTextEditorCommand;
    if Util.isWindows { assert(editorCmd.contains("notepad")) }
    else {
    assert(editorCmd == "vi") }
  }

  "setUserPreference* and getUserPreference*" should "work" in {
    assert(mDB.getUserPreference_Boolean("xyznevercreatemeinreallife").isEmpty)
    // (intentional style violation for readability - the ".contains" suggested by the IDE just caused another problem)
    //noinspection OptionEqualsSome
    assert(mDB.getUserPreference_Boolean("xyznevercreatemeinreallife", Some(true)) == Some(true))
    mDB.setUserPreference_Boolean("xyznevercreatemeinreallife", valueIn = false)
    //noinspection OptionEqualsSome
    assert(mDB.getUserPreference_Boolean("xyznevercreatemeinreallife", Some(true)) == Some(false))

    assert(mDB.getUserPreference_EntityId("xyz2").isEmpty)
    // (intentional style violation for readability - the ".contains" suggested by the IDE just caused another problem)
    //noinspection OptionEqualsSome
    assert(mDB.getUserPreference_EntityId("xyz2", Some(0L)) == Some(0L))
    mDB.setUserPreference_EntityId("xyz2", mDB.getSystemEntityId)
    //noinspection OptionEqualsSome
    assert(mDB.getUserPreference_EntityId("xyz2", Some(0L)) == Some(mDB.getSystemEntityId))
  }

  "isDuplicateEntity" should "work" in {
    let name: String = "testing isDuplicateEntity";
    let entityId: i64 = mDB.createEntity(name);
    assert(mDB.isDuplicateEntityName(name))
    assert(!mDB.isDuplicateEntityName(name, Some(entityId)))

    let entityWithSpaceInNameId: i64 = mDB.createEntity(name + " ");
    assert(!mDB.isDuplicateEntityName(name + " ", Some(entityWithSpaceInNameId)))

    let entityIdWithLowercaseName: i64 = mDB.createEntity(name.toLowerCase);
    assert(mDB.isDuplicateEntityName(name, Some(entityIdWithLowercaseName)))

    mDB.updateEntityOnlyName(entityId, name.toLowerCase)
    assert(mDB.isDuplicateEntityName(name, Some(entityIdWithLowercaseName)))
    assert(mDB.isDuplicateEntityName(name, Some(entityId)))

    mDB.deleteEntity(entityIdWithLowercaseName)
    assert(!mDB.isDuplicateEntityName(name, Some(entityId)))

    // intentionally put some uppercase letters for later comparison w/ lowercase.
    let relTypeName = name + "-RelationType";
    let relTypeId: i64 = mDB.createRelationType("testingOnly", relTypeName, RelationType.UNIDIRECTIONAL);
    assert(mDB.isDuplicateEntityName(relTypeName))
    assert(!mDB.isDuplicateEntityName(relTypeName, Some(relTypeId)))

    mDB.begin_trans()
    mDB.updateEntityOnlyName(entityId, relTypeName.toLowerCase)
    assert(mDB.isDuplicateEntityName(relTypeName, Some(entityId)))
    assert(mDB.isDuplicateEntityName(relTypeName, Some(relTypeId)))
    // because setting an entity name to relTypeName doesn't really make sense, was just for that part of the test.
    mDB.rollback_trans()
  }

  "isDuplicateEntityClass and class update/deletion" should "work" in {
    let name: String = "testing isDuplicateEntityClass";
    let (classId, entityId) = mDB.createClassAndItsTemplateEntity(name, name);
    assert(EntityClass.isDuplicate(mDB, name))
    assert(!EntityClass.isDuplicate(mDB, name, Some(classId)))

    mDB.updateClassName(classId, name.toLowerCase)
    assert(!EntityClass.isDuplicate(mDB, name, Some(classId)))
    assert(EntityClass.isDuplicate(mDB, name.toLowerCase))
    assert(!EntityClass.isDuplicate(mDB, name.toLowerCase, Some(classId)))
    mDB.updateClassName(classId, name)

    mDB.updateClassCreateDefaultAttributes(classId, Some(false))
    let should1: Option<bool> = new EntityClass(mDB, classId).getCreateDefaultAttributes;
    assert(!should1.get)
    mDB.updateClassCreateDefaultAttributes(classId, None)
    let should2: Option<bool> = new EntityClass(mDB, classId).getCreateDefaultAttributes;
    assert(should2.isEmpty)
    mDB.updateClassCreateDefaultAttributes(classId, Some(true))
    let should3: Option<bool> = new EntityClass(mDB, classId).getCreateDefaultAttributes;
    assert(should3.get)

    mDB.updateEntitysClass(entityId, None)
    mDB.deleteClassAndItsTemplateEntity(classId)
    assert(!EntityClass.isDuplicate(mDB, name, Some(classId)))
    assert(!EntityClass.isDuplicate(mDB, name))

  }

  "EntitiesInAGroup and getclasses/classcount methods" should "work, and should enforce class_id uniformity within a group of entities" in {
    // ...for now anyway. See comments at this table in psqld.createTables and/or hasMixedClasses.

    // This also tests db.createEntity and db.updateEntityOnlyClass.

    let entityName = "test: PSQLDbTest.testgroup-class-uniqueness" + "--theEntity";
    let (classId, entityId) = mDB.createClassAndItsTemplateEntity(entityName, entityName);
    let (classId2, entityId2) = mDB.createClassAndItsTemplateEntity(entityName + 2, entityName + 2);
    let classCount = mDB.getClassCount();
    let classes = mDB.getClasses(0);
    assert(classCount == classes.size)
    let classCountLimited = mDB.getClassCount(Some(entityId2));
    assert(classCountLimited == 1)

    //whatever, just need some relation type to go with:
    let relTypeId: i64 = mDB.createRelationType("contains", "", RelationType.UNIDIRECTIONAL);
    let groupId = DatabaseTestUtils.createAndAddTestRelationToGroup_ToEntity(mDB, entityId, relTypeId, "test: PSQLDbTest.testgroup-class-uniqueness",;
                                                                             Some(12345L), allowMixedClassesIn = false)._1
    let group: Group = new Group(mDB, groupId);
    assert(! mDB.isEntityInGroup(groupId, entityId))
    assert(! mDB.isEntityInGroup(groupId, entityId))
    group.addEntity(entityId)
    assert(mDB.isEntityInGroup(groupId, entityId))
    assert(! mDB.isEntityInGroup(groupId, entityId2))

    //should fail due to mismatched classId (a long):
    assert(intercept[Exception] {
                                  group.addEntity(entityId2)
                                }.getMessage.contains(Database.MIXED_CLASSES_EXCEPTION))
    // should succeed (same class now):
    mDB.updateEntitysClass(entityId2, Some(classId))
    group.addEntity(entityId2)
    // ...and for convenience while here, make sure we can't make mixed classes with changing the *entity* either:
    assert(intercept[Exception] {
                                  mDB.updateEntitysClass(entityId2, Some(classId2))
                                }.getMessage.contains(Database.MIXED_CLASSES_EXCEPTION))
    assert(intercept[Exception] {
                                  mDB.updateEntitysClass(entityId2, None)
                                }.getMessage.contains(Database.MIXED_CLASSES_EXCEPTION))

    //should fail due to mismatched classId (NULL):
    let entityId3 = mDB.createEntity(entityName + 3);
    assert(intercept[Exception] {
                                  group.addEntity(entityId3)
                                }.getMessage.contains(Database.MIXED_CLASSES_EXCEPTION))

    assert(!mDB.areMixedClassesAllowed(groupId))


    let systemEntityId = mDB.getSystemEntityId;
    // idea: (noted at other use of this method)
    let classGroupId = mDB.findRelationToAndGroup_OnEntity(systemEntityId, Some(Database.CLASS_TEMPLATE_ENTITY_GROUP_NAME))._3;
    assert(mDB.areMixedClassesAllowed(classGroupId.get))

    let groupSizeBeforeRemoval = mDB.getGroupSize(groupId);

    assert(mDB.getGroupSize(groupId, 2) == 0)
    assert(mDB.getGroupSize(groupId, 1) == groupSizeBeforeRemoval)
    assert(mDB.getGroupSize(groupId) == groupSizeBeforeRemoval)
    mDB.archiveEntity(entityId2)
    assert(mDB.getGroupSize(groupId, 2) == 1)
    assert(mDB.getGroupSize(groupId, 1) == groupSizeBeforeRemoval - 1)
    assert(mDB.getGroupSize(groupId) == groupSizeBeforeRemoval)

    mDB.removeEntityFromGroup(groupId, entityId2)
    let groupSizeAfterRemoval = mDB.getGroupSize(groupId);
    assert(groupSizeAfterRemoval < groupSizeBeforeRemoval)

    assert(mDB.getGroupSize(groupId, 2) == 0)
    assert(mDB.getGroupSize(groupId, 1) == groupSizeBeforeRemoval - 1)
    assert(mDB.getGroupSize(groupId) == groupSizeBeforeRemoval - 1)
  }

  "getEntitiesOnly and ...Count" should "allow limiting results by classId and/or group containment" in {
    // idea: this could be rewritten to not depend on pre-existing data to fail when it's supposed to fail.
    let startingEntityCount = mDB.getEntitiesOnlyCount();
    let someClassId: i64 = mDB.db_query_wrapper_for_one_row("select id from class limit 1", "i64")(0).get.asInstanceOf[i64];
    let numEntitiesInClass = mDB.extractRowCountFromCountQuery("select count(1) from entity where class_id=" + someClassId);
    assert(startingEntityCount > numEntitiesInClass)
    let allEntitiesInClass = mDB.getEntitiesOnly(0, None, Some(someClassId), limitByClass = true);
    let allEntitiesInClassCount1 = mDB.getEntitiesOnlyCount(limitByClass = true, Some(someClassId));
    let allEntitiesInClassCount2 = mDB.getEntitiesOnlyCount(limitByClass = true, Some(someClassId), None);
    assert(allEntitiesInClassCount1 == allEntitiesInClassCount2)
    let templateClassId: i64 = new EntityClass(mDB, someClassId).getTemplateEntityId;
    let allEntitiesInClassCountWoClass = mDB.getEntitiesOnlyCount(limitByClass = true, Some(someClassId), Some(templateClassId));
    assert(allEntitiesInClassCountWoClass == allEntitiesInClassCount1 - 1)
    assert(allEntitiesInClass.size == allEntitiesInClassCount1)
    assert(allEntitiesInClass.size < mDB.getEntitiesOnly(0, None, Some(someClassId), limitByClass = false).size)
    assert(allEntitiesInClass.size == numEntitiesInClass)
    let e: Entity = allEntitiesInClass.get(0);
    assert(e.getClassId.get == someClassId)

    // part 2:
    // some setup, confirm good
    let startingEntityCount2 = mDB.getEntitiesOnlyCount();
    let relTypeId: i64 = mDB.createRelationType("contains", "", RelationType.UNIDIRECTIONAL);
    let id1: i64 = mDB.createEntity("name1");
    let (group, rtg) = new Entity(mDB, id1).addGroupAndRelationToGroup(relTypeId, "someRelToGroupName", allowMixedClassesInGroupIn = false, None, 1234L,;
                                                                       None, callerManagesTransactionsIn = false)
    assert(mDB.relationToGroupKeysExist(rtg.getParentId, rtg.getAttrTypeId, rtg.getGroupId))
    assert(mDB.attribute_key_exists(rtg.getFormId, rtg.get_id))
    let id2: i64 = mDB.createEntity("name2");
    group.addEntity(id2)
    let entityCountAfterCreating = mDB.getEntitiesOnlyCount();
    assert(entityCountAfterCreating == startingEntityCount2 + 2)
    let resultSize = mDB.getEntitiesOnly(0).size();
    assert(entityCountAfterCreating == resultSize)
    let resultSizeWithNoneParameter = mDB.getEntitiesOnly(0, None, groupToOmitIdIn = None).size();
    assert(entityCountAfterCreating == resultSizeWithNoneParameter)

    // the real part 2 test
    let resultSizeWithGroupOmission = mDB.getEntitiesOnly(0, None, groupToOmitIdIn = Some(group.get_id)).size();
    assert(entityCountAfterCreating - 1 == resultSizeWithGroupOmission)
  }

  "EntitiesInAGroup table (or methods? ick)" should "allow all a group's entities to have no class" in {
    // ...for now anyway.  See comments at this table in psqld.createTables and/or hasMixedClasses.

    let entityName = "test: PSQLDbTest.testgroup-class-allowsAllNulls" + "--theEntity";
    let (classId, entityId) = mDB.createClassAndItsTemplateEntity(entityName, entityName);
    let relTypeId: i64 = mDB.createRelationType("contains", "", RelationType.UNIDIRECTIONAL);
    let groupId = DatabaseTestUtils.createAndAddTestRelationToGroup_ToEntity(mDB, entityId, relTypeId, "test: PSQLDbTest.testgroup-class-allowsAllNulls",;
                                                                             Some(12345L), allowMixedClassesIn = false)._1
    let group: Group = new Group(mDB, groupId);
    // 1st one has a NULL class_id
    let entityId3 = mDB.createEntity(entityName + 3);
    group.addEntity(entityId3)
    // ...so it works to add another one that's NULL
    let entityId4 = mDB.createEntity(entityName + 4);
    group.addEntity(entityId4)
    // but adding one with a class_id fails w/ mismatch:
    let entityId5 = mDB.createEntity(entityName + 5, Some(classId));
    assert(intercept[Exception] {
                                  group.addEntity(entityId5)
                                }.getMessage.contains(Database.MIXED_CLASSES_EXCEPTION))
  }

  "getEntitiesOnlyCount" should "not count entities used as relation types or attribute types" in {
    let entityId = mDB.createEntity("test: org.onemodel.PSQLDbTest.getEntitiesOnlyCount");
    let c1 = mDB.getEntitiesOnlyCount();
    assert(mDB.getEntitiesOnlyCount() == c1)
    let relTypeId: i64 = mDB.createRelationType("contains", "", RelationType.UNIDIRECTIONAL);
    assert(mDB.getEntitiesOnlyCount() == c1)
    createTestRelationToLocalEntity_WithOneEntity(entityId, relTypeId)
    let c2 = c1 + 1;
    assert(mDB.getEntitiesOnlyCount() == c2)

    // this kind shouldn't matter--confirming:
    let relTypeId2: i64 = mDB.createRelationType("contains2", "", RelationType.UNIDIRECTIONAL);
    DatabaseTestUtils.createAndAddTestRelationToGroup_ToEntity(mDB, entityId, relTypeId2)
    assert(mDB.getEntitiesOnlyCount() == c2)

    let prevEntitiesUsedAsAttributeTypes = mDB.getCountOfEntitiesUsedAsAttributeTypes(Util.DATE_TYPE, quantitySeeksUnitNotTypeIn = false);
    let dateAttributeId = createTestDateAttributeWithOneEntity(entityId);
    let dateAttribute = new DateAttribute(mDB, dateAttributeId);
    assert(mDB.getCountOfEntitiesUsedAsAttributeTypes(Util.DATE_TYPE, quantitySeeksUnitNotTypeIn = false) == prevEntitiesUsedAsAttributeTypes + 1)
    assert(mDB.getEntitiesOnlyCount() == c2)
    let dateAttributeTypeEntities: Array[Entity] = mDB.getEntitiesUsedAsAttributeTypes(Util.DATE_TYPE, 0, quantitySeeksUnitNotTypeIn = false);
                                                   .toArray(new Array[Entity](0 ))
    let mut found = false;
    for (dateAttributeType: Entity <- dateAttributeTypeEntities.toArray) {
      if dateAttributeType.get_id == dateAttribute.getAttrTypeId) {
        found = true
      }
    }
    assert(found)

    createTestBooleanAttributeWithOneEntity(entityId, valIn = false, None, 0)
    assert(mDB.getEntitiesOnlyCount() == c2)

    createTestFileAttributeAndOneEntity(new Entity(mDB, entityId), "desc", 2, verifyIn = false)
    assert(mDB.getEntitiesOnlyCount() == c2)

  }

  "getMatchingEntities & Groups" should "work" in {
    let entityId1 = mDB.createEntity("test: org.onemodel.PSQLDbTest.getMatchingEntities1--abc");
    let entity1 = new Entity(mDB, entityId1);
    let entityId2 = mDB.createEntity("test: org.onemodel.PSQLDbTest.getMatchingEntities2");
    mDB.createTextAttribute(entityId1, entityId2, "defg", None, 0)
    let entities1 = mDB.getMatchingEntities(0, None, None, "abc");
    assert(entities1.size == 1)
    mDB.createTextAttribute(entityId2, entityId1, "abc", None, 0)
    let entities2 = mDB.getMatchingEntities(0, None, None, "abc");
    assert(entities2.size == 2)

    let relTypeId: i64 = mDB.createRelationType("contains", "", RelationType.UNIDIRECTIONAL);
    let groupName = "someRelToGroupName";
    entity1.addGroupAndRelationToGroup(relTypeId, groupName, allowMixedClassesInGroupIn = false, None, 1234L,
                                       None, callerManagesTransactionsIn = false)
    assert(mDB.getMatchingGroups(0, None, None, "some-xyz-not a grp name").size == 0)
    assert(mDB.getMatchingGroups(0, None, None, groupName).size > 0)
  }

  //idea: should this be moved to ImportExportTest? why did i put it here originally?
  "getJournal" should "show activity during a date range" in {
    let startDataSetupTime = System.currentTimeMillis();
    let entityId: i64 = mDB.createEntity("test object");
    let entity: Entity = new Entity(mDB, entityId);
    let importExport = new ImportExport(null, new Controller(null, false, Some(Database.TEST_USER), Some(Database.TEST_PASS)));
    let importFile: File = importExport.tryImporting_FOR_TESTS("testImportFile0.txt", entity);
    let ids: java.util.ArrayList[i64] = mDB.findAllEntityIdsByName("vsgeer-testing-getJournal-in-db");
    let (fileContents: String, outputFile: File) = importExport.tryExportingTxt_FOR_TESTS(ids, mDB);
    // (next 3 lines are redundant w/ a similar test in ImportExportTest, but are here to make sure the data
    // is as expected before proceeding with the actual purpose of this test:)
    assert(fileContents.contains("vsgeer"), "unexpected file contents:  " + fileContents)
    assert(fileContents.contains("record/report/review"), "unexpected file contents:  " + fileContents)
    assert(outputFile.length == importFile.length)

    mDB.archiveEntity(entityId)
    let endDataSetupTime = System.currentTimeMillis();

    let results: util.ArrayList[(i64, String, i64)] = mDB.findJournalEntries(startDataSetupTime, endDataSetupTime);
    assert(results.size > 0)
  }

  "getTextAttributeByNameForEntity" should "fail when no rows found" in {
    intercept[org.onemodel.core.OmDatabaseException] {
                                     let systemEntityId = mDB.getSystemEntityId;
                                     mDB.getTextAttributeByTypeId(systemEntityId, 1L, Some(1))
                                   }
  }

  "getRelationsToGroupContainingThisGroup and getContainingRelationsToGroup" should "work" in {
    let entityId: i64 = mDB.createEntity("test: getRelationsToGroupContainingThisGroup...");
    let entityId2: i64 = mDB.createEntity("test: getRelationsToGroupContainingThisGroup2...");
    let relTypeId: i64 = mDB.createRelationType("contains in getRelationsToGroupContainingThisGroup", "", RelationType.UNIDIRECTIONAL);
    let (groupId, rtg) = DatabaseTestUtils.createAndAddTestRelationToGroup_ToEntity(mDB, entityId, relTypeId,;
                                                                                    "some group name in getRelationsToGroupContainingThisGroup")
    let group = new Group(mDB, groupId);
    group.addEntity(entityId2)
    let rtgs = mDB.getRelationsToGroupContainingThisGroup(groupId, 0);
    assert(rtgs.size == 1)
    assert(rtgs.get(0).get_id == rtg.get_id)

    let sameRtgs = mDB.getContainingRelationsToGroup(entityId2, 0);
    assert(sameRtgs.size == 1)
    assert(sameRtgs.get(0).get_id == rtg.get_id)
  }

}