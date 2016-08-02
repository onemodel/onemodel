/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003, 2004, 2010, 2011, and 2013-2016 inclusive, Luke A. Call; all rights reserved.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule, guidelines around binary
    distribution, and the GNU Affero General Public License as published by the Free Software Foundation, either version 3
    of the License, or (at your option) any later version.  See the file LICENSE for details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
package org.onemodel

import java.io.File

import org.onemodel.controller.{Controller, ImportExport}
import org.onemodel.database.PostgreSQLDatabase
import org.onemodel.model._
import org.scalatest.mock.MockitoSugar
import org.scalatest.{Args, FlatSpec, Status}

import scala.collection.mutable

object PostgreSQLDatabaseTest {
  def tearDownTestDB() {
    // reconnect to the normal production database and tear down the temporary one we used for testing.
    // This is part of the singleton object, in part so that it can be called even before we have a Database object: this is to avoid
    // doing setup (at first db instantiation for a new system), then immediately another teardown/setup for the tests.
    try {
      PostgreSQLDatabase.destroyTables(TEST_USER, TEST_USER, TEST_USER)
    }
    catch {
      case e: java.sql.SQLException =>
        if (e.toString.indexOf("is being accessed by other users") != -1) {
          // why did this happen sometimes?
          // but it can be ignored, as the next test run will also clean this out as it starts.
        }
        else {
          throw e
        }
    }
  }

  val TEST_USER: String = "testrunner"

}

class PostgreSQLDatabaseTest extends FlatSpec with MockitoSugar {
  PostgreSQLDatabaseTest.tearDownTestDB()

  // for a test
  private var mDoDamageBuffer = false

  // instantiation does DB setup (creates tables, default data, etc):
  private val mDB: PostgreSQLDatabase = new PostgreSQLDatabase(PostgreSQLDatabaseTest.TEST_USER, PostgreSQLDatabaseTest.TEST_USER) {
    override def damageBuffer(buffer: Array[Byte]): Unit = {
      if (mDoDamageBuffer) {
        if (buffer.length < 1 || buffer(0) == '0') throw new OmException("Nothing to damage here")
        else {
          if (buffer(0) == '1') buffer(0) = 2.toByte
          else buffer(0) = 1.toByte
          // once is enough until we want to cause another failure
          mDoDamageBuffer = false
        }
      }
    }
  }

  private final val QUANTITY_TYPE_NAME: String = "length"
  private final val RELATION_TYPE_NAME: String = "someRelationToEntityTypeName"

  // connect to existing database first
  private final val RELATED_ENTITY_NAME: String = "someRelatedEntityName"

  override def runTests(testName: Option[String], args: Args): Status = {
    // no longer doing db setup/teardown here, because we need to do teardown as a constructor-like command above,
    // before instantiating the DB (and that instantiation does setup).  Leaving tables in place after to allow adhoc manual test access.
    val result: Status = super.runTests(testName, args)
    result
  }

  "database version table" should "have been created with right data" in {
    val versionTableExists: Boolean = mDB.doesThisExist("select count(1) from pg_class where relname='om_db_version'")
    assert(versionTableExists)
    val results = mDB.dbQueryWrapperForOneRow("select version from om_db_version", "Int")
    assert(results.length == 1)
    val dbVer: Int = results(0).get.asInstanceOf[Int]
    assert(dbVer == PostgreSQLDatabase.CURRENT_DB_VERSION, "dbVer and PostgreSQLDatabase.CURRENT_DB_VERSION are: " +
                                                           dbVer + ", " + PostgreSQLDatabase.CURRENT_DB_VERSION)
  }

  "escapeQuotesEtc" should "allow updating db with single-quotes" in {
    val name: String = "This ' name contains a single-quote."
    mDB.beginTrans()

    //on a create:
    val entityId: Long = mDB.createEntity(name)
    assert(name == mDB.getEntityName(entityId).get)

    //and on an update:
    val textAttributeId: Long = createTestTextAttributeWithOneEntity(entityId)
    val aTextValue = "as'dfjkl"
    val ta = new TextAttribute(mDB, textAttributeId)
    val (pid1, atid1) = (ta.getParentId, ta.getAttrTypeId)
    mDB.updateTextAttribute(textAttributeId, pid1, atid1, aTextValue, Some(123), 456)
    // have to create new instance to re-read the data:
    val ta2 = new TextAttribute(mDB, textAttributeId)
    val txt2 = ta2.getText

    assert(txt2 == aTextValue)
    mDB.rollbackTrans()
  }

  "entity creation/update and transaction rollback" should "create one new entity, work right, then have none" in {
    val name: String = "test: org.onemodel.PSQLDbTest.entitycreation..."
    mDB.beginTrans()

    val entityCountBeforeCreating: Long = mDB.getEntityCount
    val entitiesOnlyFirstCount: Long = mDB.getEntitiesOnlyCount()

    val id: Long = mDB.createEntity(name)
    assert(name == mDB.getEntityName(id).get)
    val entityCountAfter1stCreate: Long = mDB.getEntityCount
    val entitiesOnlyNewCount: Long = mDB.getEntitiesOnlyCount()
    if (entityCountBeforeCreating + 1 != entityCountAfter1stCreate || entitiesOnlyFirstCount + 1 != entitiesOnlyNewCount) {
      fail("getEntityCount after adding doesn't match prior count+1! Before: " + entityCountBeforeCreating + " and " + entitiesOnlyNewCount + ", " +
           "after: " + entityCountAfter1stCreate + " and " + entitiesOnlyNewCount + ".")
    }
    assert(mDB.entityKeyExists(id))

    val newName = "test: ' org.onemodel.PSQLDbTest.entityupdate..."
    mDB.updateEntityOnlyName(id, newName)
    // have to create new instance to re-read the data:
    val updatedEntity = new Entity(mDB, id)
    assert(updatedEntity.getName == newName)

    assert(mDB.entityOnlyKeyExists(id))
    mDB.rollbackTrans()

    // now should not exist
    val entityCountAfterRollback = mDB.getEntityCount
    assert(entityCountAfterRollback == entityCountBeforeCreating)
    assert(!mDB.entityKeyExists(id))
  }

  "findIdWhichIsNotKeyOfAnyEntity" should "find a nonexistent entity key" in {
    assert(!mDB.entityKeyExists(mDB.findIdWhichIsNotKeyOfAnyEntity))
  }

  "entityOnlyKeyExists" should "not find RelationToEntity record" in {
    mDB.beginTrans()
    val tempRelTypeId: Long = mDB.createRelationType(RELATION_TYPE_NAME, "", RelationType.UNIDIRECTIONAL)
    assert(!mDB.entityOnlyKeyExists(tempRelTypeId))
    mDB.deleteRelationType(tempRelTypeId)
    mDB.rollbackTrans()
  }

  "getAttrCount, getAttributeSortingRowsCount" should "work in all circumstances" in {
    mDB.beginTrans()

    val id: Long = mDB.createEntity("test: org.onemodel.PSQLDbTest.getAttrCount...")
    val initialNumSortingRows = mDB.getAttributeSortingRowsCount(Some(id))
    assert(mDB.getAttrCount(id) == 0)
    assert(initialNumSortingRows == 0)

    createTestQuantityAttributeWithTwoEntities(id)
    createTestQuantityAttributeWithTwoEntities(id)
    assert(mDB.getAttrCount(id) == 2)
    assert(mDB.getAttributeSortingRowsCount(Some(id)) == 2)

    createTestTextAttributeWithOneEntity(id)
    assert(mDB.getAttrCount(id) == 3)
    assert(mDB.getAttributeSortingRowsCount(Some(id)) == 3)

    //whatever, just need some relation type to go with:
    val relTypeId: Long = mDB.createRelationType("contains", "", RelationType.UNIDIRECTIONAL)
    createTestRelationToEntity_WithOneEntity(id, relTypeId)
    assert(mDB.getAttrCount(id) == 4)
    assert(mDB.getAttributeSortingRowsCount(Some(id)) == 4)

    createAndAddTestRelationToGroup_ToEntity(id, relTypeId, "somename", Some(12345L))
    assert(mDB.getAttrCount(id) == 5)
    assert(mDB.getAttributeSortingRowsCount(Some(id)) == 5)

    mDB.rollbackTrans()
    //idea: find out: WHY do the next lines fail, because the attrCount(id) is the same (4) after rolling back as before rolling back??
    // Do I not understand rollback?
//    assert(mDB.getAttrCount(id) == 0)
//    assert(mDB.getAttributeSortingRowsCount(Some(id)) == 0)
  }

  "QuantityAttribute creation/update/deletion methods" should "work" in {
    mDB.beginTrans()
    val startingEntityCount = mDB.getEntityCount
    val entityId = mDB.createEntity("test: org.onemodel.PSQLDbTest.quantityAttrs()")
    val initialTotalSortingRowsCount = mDB.getAttributeSortingRowsCount()
    val quantityAttributeId: Long = createTestQuantityAttributeWithTwoEntities(entityId)
    assert(mDB.getAttributeSortingRowsCount() > initialTotalSortingRowsCount)

    val qa = new QuantityAttribute(mDB, quantityAttributeId)
    val (pid1, atid1, uid1) = (qa.getParentId, qa.getAttrTypeId, qa.getUnitId)
    assert(entityId == pid1)
    mDB.updateQuantityAttribute(quantityAttributeId, pid1, atid1, uid1, 4, Some(5), 6)
    // have to create new instance to re-read the data:
    val qa2 = new QuantityAttribute(mDB, quantityAttributeId)
    val (pid2, atid2, uid2, num2, vod2, od2) = (qa2.getParentId, qa2.getAttrTypeId, qa2.getUnitId, qa2.getNumber, qa2.getValidOnDate, qa2.getObservationDate)
    assert(pid2 == pid1)
    assert(atid2 == atid1)
    assert(uid2 == uid1)
    assert(num2 == 4)
    // (the ".contains" suggested by the IDE just caused another problem)
    //noinspection OptionEqualsSome
    assert(vod2 == Some(5L))
    assert(od2 == 6)

    val qAttrCount = mDB.getQuantityAttributeCount(entityId)
    assert(qAttrCount == 1)
    assert(mDB.getAttributeSortingRowsCount(Some(entityId)) == 1)

    //delete the quantity attribute: #'s still right?
    val entityCountBeforeQuantityDeletion: Long = mDB.getEntityCount
    mDB.deleteQuantityAttribute(quantityAttributeId)
    // next 2 lines should work because of the database logic (triggers as of this writing) that removes sorting rows when attrs are removed):
    assert(mDB.getAttributeSortingRowsCount() == initialTotalSortingRowsCount)
    assert(mDB.getAttributeSortingRowsCount(Some(entityId)) == 0)

    val entityCountAfterQuantityDeletion: Long = mDB.getEntityCount
    assert(mDB.getQuantityAttributeCount(entityId) == 0)
    if (entityCountAfterQuantityDeletion != entityCountBeforeQuantityDeletion) {
      fail("Got constraint backwards? Deleting quantity attribute changed Entity count from " + entityCountBeforeQuantityDeletion + " to " +
           entityCountAfterQuantityDeletion)
    }

    mDB.deleteEntity(entityId)
    val endingEntityCount = mDB.getEntityCount
    // 2 more entities came during quantity creation (units & quantity type, is OK to leave in this kind of situation)
    assert(endingEntityCount == startingEntityCount + 2)
    assert(mDB.getQuantityAttributeCount(entityId) == 0)
    mDB.rollbackTrans()
  }

  "Attribute and AttributeSorting row deletion" should "both happen automatically upon entity deletion" in {
    val entityId = mDB.createEntity("test: org.onemodel.PSQLDbTest sorting rows stuff")
    createTestQuantityAttributeWithTwoEntities(entityId)
    assert(mDB.getAttributeSortingRowsCount(Some(entityId)) == 1)
    assert(mDB.getQuantityAttributeCount(entityId) == 1)
    mDB.deleteEntity(entityId)
    assert(mDB.getAttributeSortingRowsCount(Some(entityId)) == 0)
    assert(mDB.getQuantityAttributeCount(entityId) == 0)
  }

  "TextAttribute create/delete/update methods" should "work" in {
    val startingEntityCount = mDB.getEntityCount
    val entityId = mDB.createEntity("test: org.onemodel.PSQLDbTest.testTextAttrs")
    assert(mDB.getAttributeSortingRowsCount(Some(entityId)) == 0)
    val textAttributeId: Long = createTestTextAttributeWithOneEntity(entityId)
    assert(mDB.getAttributeSortingRowsCount(Some(entityId)) == 1)
    val aTextValue = "asdfjkl"

    val ta = new TextAttribute(mDB, textAttributeId)
    val (pid1, atid1) = (ta.getParentId, ta.getAttrTypeId)
    assert(entityId == pid1)
    mDB.updateTextAttribute(textAttributeId, pid1, atid1, aTextValue, Some(123), 456)
    // have to create new instance to re-read the data: immutability makes programs easier to work with
    val ta2 = new TextAttribute(mDB, textAttributeId)
    val (pid2, atid2, txt2, vod2, od2) = (ta2.getParentId, ta2.getAttrTypeId, ta2.getText, ta2.getValidOnDate, ta2.getObservationDate)
    assert(pid2 == pid1)
    assert(atid2 == atid1)
    assert(txt2 == aTextValue)
    // (the ".contains" suggested by the IDE just caused another problem)
    //noinspection OptionEqualsSome
    assert(vod2 == Some(123L))
    assert(od2 == 456)

    assert(mDB.getTextAttributeCount(entityId) == 1)

    val entityCountBeforeTextDeletion: Long = mDB.getEntityCount
    mDB.deleteTextAttribute(textAttributeId)
    assert(mDB.getTextAttributeCount(entityId) == 0)
    // next line should work because of the database logic (triggers as of this writing) that removes sorting rows when attrs are removed):
    assert(mDB.getAttributeSortingRowsCount(Some(entityId)) == 0)
    val entityCountAfterTextDeletion: Long = mDB.getEntityCount
    if (entityCountAfterTextDeletion != entityCountBeforeTextDeletion) {
      fail("Got constraint backwards? Deleting text attribute changed Entity count from " + entityCountBeforeTextDeletion + " to " +
           entityCountAfterTextDeletion)
    }
    // then recreate the text attribute (to verify its auto-deletion when Entity is deleted, below)
    createTestTextAttributeWithOneEntity(entityId)
    mDB.deleteEntity(entityId)
    if (mDB.getTextAttributeCount(entityId) > 0) {
      fail("Deleting the model entity should also have deleted its text attributes; getTextAttributeCount(entityIdInNewTransaction) is " +
           mDB.getTextAttributeCount(entityId) + ".")
    }

    val endingEntityCount = mDB.getEntityCount
    // 2 more entities came during text attribute creation, which we don't care about either way, for this test
    assert(endingEntityCount == startingEntityCount + 2)
  }

  "DateAttribute create/delete/update methods" should "work" in {
    val startingEntityCount = mDB.getEntityCount
    val entityId = mDB.createEntity("test: org.onemodel.PSQLDbTest.testDateAttrs")
    assert(mDB.getAttributeSortingRowsCount(Some(entityId)) == 0)
    val dateAttributeId: Long = createTestDateAttributeWithOneEntity(entityId)
    assert(mDB.getAttributeSortingRowsCount(Some(entityId)) == 1)
    val da = new DateAttribute(mDB, dateAttributeId)
    val (pid1, atid1) = (da.getParentId, da.getAttrTypeId)
    assert(entityId == pid1)
    val date = System.currentTimeMillis
    mDB.updateDateAttribute(dateAttributeId, pid1, date, atid1)
    // Have to create new instance to re-read the data: immutability makes the program easier to debug/reason about.
    val da2 = new DateAttribute(mDB, dateAttributeId)
    val (pid2, atid2, date2) = (da2.getParentId, da2.getAttrTypeId, da2.getDate)
    assert(pid2 == pid1)
    assert(atid2 == atid1)
    assert(date2 == date)
    // Also test the other constructor.
    val da3 = new DateAttribute(mDB, dateAttributeId, pid1, atid1, date, 0)
    val (pid3, atid3, date3) = (da3.getParentId, da3.getAttrTypeId, da3.getDate)
    assert(pid3 == pid1)
    assert(atid3 == atid1)
    assert(date3 == date)
    assert(mDB.getDateAttributeCount(entityId) == 1)

    val entityCountBeforeDateDeletion: Long = mDB.getEntityCount
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
    val startingEntityCount = mDB.getEntityCount
    val entityId = mDB.createEntity("test: org.onemodel.PSQLDbTest.testBooleanAttrs")
    val val1 = true
    val observationDate: Long = System.currentTimeMillis
    val validOnDate: Option[Long] = Some(1234L)
    assert(mDB.getAttributeSortingRowsCount(Some(entityId)) == 0)
    val booleanAttributeId: Long = createTestBooleanAttributeWithOneEntity(entityId, val1, validOnDate, observationDate)
    assert(mDB.getAttributeSortingRowsCount(Some(entityId)) == 1)

    val ba = new BooleanAttribute(mDB, booleanAttributeId)
    val (pid1, atid1) = (ba.getParentId, ba.getAttrTypeId)
    assert(entityId == pid1)

    val val2 = false
    mDB.updateBooleanAttribute(booleanAttributeId, pid1, atid1, val2, Some(123), 456)
    // have to create new instance to re-read the data:
    val ba2 = new BooleanAttribute(mDB, booleanAttributeId)
    val (pid2, atid2, bool2, vod2, od2) = (ba2.getParentId, ba2.getAttrTypeId, ba2.getBoolean, ba2.getValidOnDate, ba2.getObservationDate)
    assert(pid2 == pid1)
    assert(atid2 == atid1)
    assert(bool2 == val2)
    // (the ".contains" suggested by the IDE just caused another problem)
    //noinspection OptionEqualsSome
    assert(vod2 == Some(123L))
    assert(od2 == 456)

    assert(mDB.getBooleanAttributeCount(entityId) == 1)

    val entityCountBeforeAttrDeletion: Long = mDB.getEntityCount
    mDB.deleteBooleanAttribute(booleanAttributeId)
    assert(mDB.getBooleanAttributeCount(entityId) == 0)
    // next line should work because of the database logic (triggers as of this writing) that removes sorting rows when attrs are removed):
    assert(mDB.getAttributeSortingRowsCount(Some(entityId)) == 0)
    val entityCountAfterAttrDeletion: Long = mDB.getEntityCount
    if (entityCountAfterAttrDeletion != entityCountBeforeAttrDeletion) {
      fail("Got constraint backwards? Deleting boolean attribute changed Entity count from " + entityCountBeforeAttrDeletion + " to " +
           entityCountAfterAttrDeletion)
    }

    // then recreate the attribute (to verify its auto-deletion when Entity is deleted, below; and to verify behavior with other values)
    val testval2: Boolean = true
    val validOnDate2: Option[Long] = None
    val boolAttributeId2: Long = mDB.createBooleanAttribute(pid1, atid1, testval2, validOnDate2, observationDate)
    val ba3: BooleanAttribute = new BooleanAttribute(mDB, boolAttributeId2)
    assert(ba3.getBoolean == testval2)
    assert(ba3.getValidOnDate.isEmpty)
    mDB.deleteEntity(entityId)
    assert(mDB.getBooleanAttributeCount(entityId) == 0)

    val endingEntityCount = mDB.getEntityCount
    // 2 more entities came during attribute creation, but we deleted one and (unlike similar tests) didn't recreate it.
    assert(endingEntityCount == startingEntityCount + 1)
  }

  "FileAttribute create/delete/update methods" should "work" in {
    val startingEntityCount = mDB.getEntityCount
    val entityId = mDB.createEntity("test: org.onemodel.PSQLDbTest.testFileAttrs")
    val descr = "somedescr"
    assert(mDB.getAttributeSortingRowsCount(Some(entityId)) == 0)
    val fa: FileAttribute = createTestFileAttributeAndOneEntity(new Entity(mDB, entityId), descr, 1)
    assert(mDB.getAttributeSortingRowsCount(Some(entityId)) == 1)
    val fileAttributeId = fa.getId
    val (pid1, atid1, desc1) = (fa.getParentId, fa.getAttrTypeId, fa.getDescription)
    assert(desc1 == descr)
    val descNew = "otherdescription"
    val originalFileDateNew = 1
    val storedDateNew = 2
    val pathNew = "/a/b/cd.efg"
    val sizeNew = 1234
    val hashNew = "hashchars..."
    val b11 = false
    val b12 = true
    val b13 = false
    mDB.updateFileAttribute(fa.getId, pid1, atid1, descNew, originalFileDateNew, storedDateNew, pathNew, b11, b12, b13, sizeNew, hashNew)
    // have to create new instance to re-read the data:
    val fa2 = new FileAttribute(mDB, fa.getId)
    val (pid2, atid2, desc2, ofd2, sd2, ofp2, b21, b22, b23, size2, hash2) = (fa2.getParentId, fa2.getAttrTypeId, fa2.getDescription, fa2.getOriginalFileDate,
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

    val someRelTypeId = mDB.createRelationType("test: org.onemodel.PSQLDbTest.testFileAttrs-reltyp", "reversed", "BI")
    val descNewer = "other-newer"
    new FileAttribute(mDB, fa.getId).update(Some(someRelTypeId), Some(descNewer))

    // have to create new instance to re-read the data:
    val fa3 = new FileAttribute(mDB, fileAttributeId)
    val (pid3, atid3, desc3, ofd3, sd3, ofp3, b31, b32, b33, size3, hash3) = (fa3.getParentId, fa3.getAttrTypeId, fa3.getDescription, fa3.getOriginalFileDate,
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

    val fileAttribute4 = new FileAttribute(mDB, fileAttributeId)
    fileAttribute4.update()
    // have to create new instance to re-read the data:
    val fa4 = new FileAttribute(mDB, fileAttributeId)
    val (atid4, d4, ofd4, sd4, ofp4, b41) =
      (fa4.getAttrTypeId, fa4.getDescription, fa4.getOriginalFileDate, fa4.getStoredDate, fa4.getOriginalFilePath, fa4.getReadable)
    // these 2 are the key ones for this section: make sure they didn't change since we passed None to the update:
    assert(atid4 == atid3)
    assert(d4 == desc3)
    //throw in a few more
    assert(ofd4 == originalFileDateNew)
    assert(sd4 == storedDateNew)
    assert(ofp4 == pathNew)
    assert(b41 == b11)

    val entityCountBeforeFileAttrDeletion: Long = mDB.getEntityCount
    mDB.deleteFileAttribute(fileAttributeId)
    assert(mDB.getFileAttributeCount(entityId) == 0)
    // next line should work because of the database logic (triggers as of this writing) that removes sorting rows when attrs are removed):
    assert(mDB.getAttributeSortingRowsCount(Some(entityId)) == 0)
    val entityCountAfterFileAttrDeletion: Long = mDB.getEntityCount
    if (entityCountAfterFileAttrDeletion != entityCountBeforeFileAttrDeletion) {
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
    val entityId: Long = mDB.createEntity("someent")
    val attrTypeId: Long = mDB.createEntity("fileAttributeType")
    val uploadSourceFile: java.io.File = java.io.File.createTempFile("om-test-iofailures-", null)
    var writer: java.io.FileWriter = null
    var inputStream: java.io.FileInputStream = null
    val downloadTargetFile = File.createTempFile("om-testing-file-retrieval-", null)
    try {
      writer = new java.io.FileWriter(uploadSourceFile)
      writer.write("<1 kB file from: " + uploadSourceFile.getCanonicalPath + ", created " + new java.util.Date())
      writer.close()
      try {
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
        val faId: Long = mDB.createFileAttribute(entityId, attrTypeId, "xyz", 0, 0, "/doesntmatter", readableIn = true, writableIn = true, executableIn = false,
                                                 uploadSourceFile.length(), FileAttribute.md5Hash(uploadSourceFile), inputStream, None)

        val fa: FileAttribute = new FileAttribute(mDB, faId)
        mDoDamageBuffer = true
        intercept[OmFileTransferException] {
                                              fa.retrieveContent(downloadTargetFile)
                                            }
        mDoDamageBuffer = false
        //so it should work now
        fa.retrieveContent(downloadTargetFile)
      }
    } finally {
      mDoDamageBuffer=false
      if (inputStream != null) inputStream.close()
      if (writer != null) writer.close()
      if (downloadTargetFile != null) {
        downloadTargetFile.delete()
      }
    }
  }

  "relation to entity methods and relation type methods" should "work" in {
    val startingEntityOnlyCount = mDB.getEntitiesOnlyCount()
    val startingRelationTypeCount = mDB.getRelationTypeCount
    val entityId = mDB.createEntity("test: org.onemodel.PSQLDbTest.testRelsNRelTypes()")
    val startingRelCount = mDB.getRelationTypes(0, Some(25)).size
    val relTypeId: Long = mDB.createRelationType("contains", "", RelationType.UNIDIRECTIONAL)

    //verify a bugfix from 2013-10-31 or 2013-11-4 in how SELECT is written.
    assert(mDB.getRelationTypes(0, Some(25)).size == startingRelCount + 1)
    assert(mDB.getEntitiesOnlyCount() == startingEntityOnlyCount + 1)

    assert(mDB.getAttributeSortingRowsCount(Some(entityId)) == 0)
    val relatedEntityId: Long = createTestRelationToEntity_WithOneEntity(entityId, relTypeId)
    assert(mDB.getAttributeSortingRowsCount(Some(entityId)) == 1)
    val checkRelation = mDB.getRelationToEntityData(relTypeId, entityId, relatedEntityId)
    val checkValidOnDate = checkRelation(1)
    assert(checkValidOnDate.isEmpty) // should get back None when created with None: see description for table's field in createTables method.

    assert(mDB.getRelationToEntityCount(entityId) == 1)

    val newName = "test: org.onemodel.PSQLDbTest.relationupdate..."
    val nameInReverse = "nameinreverse;!@#$%^&*()-_=+{}[]:\"'<>?,./`~" //and verify can handle some variety of chars
    mDB.updateRelationType(relTypeId, newName, nameInReverse, RelationType.BIDIRECTIONAL)
    // have to create new instance to re-read the data:
    val updatedRelationType = new RelationType(mDB, relTypeId)
    assert(updatedRelationType.getName == newName)
    assert(updatedRelationType.getNameInReverseDirection == nameInReverse)
    assert(updatedRelationType.getDirectionality == RelationType.BIDIRECTIONAL)

    mDB.deleteRelationToEntity(relTypeId, entityId, relatedEntityId)
    assert(mDB.getRelationToEntityCount(entityId) == 0)
    // next line should work because of the database logic (triggers as of this writing) that removes sorting rows when attrs are removed):
    assert(mDB.getAttributeSortingRowsCount(Some(entityId)) == 0)

    val entityOnlyCountBeforeRelationTypeDeletion: Long = mDB.getEntitiesOnlyCount()
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
    val entityId1 = mDB.createEntity("test-getContainingGroupsIds-entity1")
    val relTypeId: Long = mDB.createRelationType("test-getContainingGroupsIds-reltype1", "", RelationType.UNIDIRECTIONAL)
    val (groupId1, _) = createAndAddTestRelationToGroup_ToEntity(entityId1, relTypeId, "test-getContainingGroupsIds-group1")
    val group1 = new Group(mDB,groupId1)
    val entityId2 = mDB.createEntity("test-getContainingGroupsIds-entity2")
    group1.addEntity(entityId2)
    val (groupId2, _) = createAndAddTestRelationToGroup_ToEntity(entityId2, relTypeId, "test-getContainingGroupsIds-group1")
    val group2 = new Group(mDB, groupId2)

    val containingGroups:List[Array[Option[Any]]] = mDB.getGroupsContainingEntitysGroupsIds(group2.getId)
    assert(containingGroups.size == 1)
    assert(containingGroups.head(0).get.asInstanceOf[Long] == groupId1)

    val entityId3 = mDB.createEntity("test-getContainingGroupsIds-entity3")
    val (groupId3, _) = createAndAddTestRelationToGroup_ToEntity(entityId3, relTypeId, "test-getContainingGroupsIds-group1")
    val group3 = new Group(mDB, groupId3)
    group3.addEntity(entityId2)

    val containingGroups2:List[Array[Option[Any]]] = mDB.getGroupsContainingEntitysGroupsIds(group2.getId)
    assert(containingGroups2.size == 2)
    assert(containingGroups2.head(0).get.asInstanceOf[Long] == groupId1)
    assert(containingGroups2.tail.head(0).get.asInstanceOf[Long] == groupId3)
  }

  "relation to group and group methods" should "work" in {
    val relToGroupName = "test: PSQLDbTest.testRelsNRelTypes()"
    val entityName = relToGroupName + "--theEntity"
    val entityId = mDB.createEntity(entityName)
    val relTypeId: Long = mDB.createRelationType("contains", "", RelationType.UNIDIRECTIONAL)
    val validOnDate = 12345L
    assert(mDB.getAttributeSortingRowsCount(Some(entityId)) == 0)
    val (groupId:Long, createdRtg:RelationToGroup) = createAndAddTestRelationToGroup_ToEntity(entityId, relTypeId, relToGroupName, Some(validOnDate),
                                                                                              allowMixedClassesIn = true)
    assert(mDB.getAttributeSortingRowsCount(Some(entityId)) == 1)

    val rtg: RelationToGroup = new RelationToGroup(mDB, createdRtg.getId, createdRtg.getParentId, createdRtg.getAttrTypeId, createdRtg.getGroupId)
    val group: Group = new Group(mDB, groupId)
    assert(group.getMixedClassesAllowed)
    assert(group.getName == relToGroupName)

    val checkRelation = mDB.getRelationToGroupData(rtg.getParentId, rtg.getAttrTypeId, rtg.getGroupId)
    assert(checkRelation(0).get.asInstanceOf[Long] == rtg.getId)
    assert(checkRelation(0).get.asInstanceOf[Long] == createdRtg.getId)
    assert(checkRelation(1).get.asInstanceOf[Long] == entityId)
    assert(checkRelation(2).get.asInstanceOf[Long] == relTypeId)
    assert(checkRelation(3).get.asInstanceOf[Long] == groupId)
    assert(checkRelation(4).get.asInstanceOf[Long] == validOnDate)

    assert(group.getSize == 0)
    val entityId2 = mDB.createEntity(entityName + 2)
    group.addEntity(entityId2)
    assert(group.getSize == 1)
    group.deleteWithEntities()
    assert(intercept[Exception] {
                                  new RelationToGroup(mDB, rtg.getId, rtg.getParentId, rtg.getAttrTypeId, rtg.getGroupId )
                                }.getMessage.contains("do not exist"))
    assert(intercept[Exception] {
                                  new Entity(mDB, entityId2)
                                }.getMessage.contains("does not exist"))
    assert(group.getSize == 0)
    // next line should work because of the database logic (triggers as of this writing) that removes sorting rows when attrs are removed):
    assert(mDB.getAttributeSortingRowsCount(Some(entityId)) == 0)

    val (groupId2, _) = createAndAddTestRelationToGroup_ToEntity(entityId, relTypeId, "somename", None)

    val group2: Group = new Group(mDB, groupId2)
    assert(group2.getSize == 0)

    val entityId3 = mDB.createEntity(entityName + 3)
    group2.addEntity(entityId3)
    assert(group2.getSize == 1)

    val entityId4 = mDB.createEntity(entityName + 4)
    group2.addEntity(entityId4)
    val entityId5 = mDB.createEntity(entityName + 5)
    group2.addEntity(entityId5)
    assert(group2.getSize == 3)
    assert(mDB.getGroupEntryObjects(group2.getId, 0).size() == 3)

    group2.removeEntity(entityId5)
    assert(mDB.getGroupEntryObjects(group2.getId, 0).size() == 2)

    group2.delete()
    assert(intercept[Exception] {
                                  new Group(mDB, groupId)
                                }.getMessage.contains("does not exist"))
    assert(group2.getSize == 0)
    // ensure the other entity still exists: not deleted by that delete command
    new Entity(mDB, entityId3)

    // probably revise this later for use when adding that update method:
    //val newName = "test: org.onemodel.PSQLDbTest.relationupdate..."
    //mDB.updateRelationType(relTypeId, newName, nameInReverse, RelationType.BIDIRECTIONAL)
    //// have to create new instance to re-read the data:
    //val updatedRelationType = new RelationType(mDB, relTypeId)
    //assert(updatedRelationType.getName == newName)
    //assert(updatedRelationType.getNameInReverseDirection == nameInReverse)
    //assert(updatedRelationType.getDirectionality == RelationType.BIDIRECTIONAL)

    //mDB.deleteRelationToGroup(relToGroupId)
    //assert(mDB.getRelationToGroupCount(entityId) == 0)
  }

  "deleting entity" should "work even if entity is in a relationtogroup" in {
    val startingEntityCount = mDB.getEntitiesOnlyCount()
    val relToGroupName = "test:PSQLDbTest.testDelEntity_InGroup"
    val entityName = relToGroupName + "--theEntity"
    val entityId = mDB.createEntity(entityName)
    val relTypeId: Long = mDB.createRelationType("contains", "", RelationType.UNIDIRECTIONAL)
    val validOnDate = 12345L
    val groupId = createAndAddTestRelationToGroup_ToEntity(entityId, relTypeId, relToGroupName, Some(validOnDate))._1
    //val rtg: RelationToGroup = new RelationToGroup
    val group:Group = new Group(mDB, groupId)
    group.addEntity(mDB.createEntity(entityName + 1))
    assert(mDB.getEntitiesOnlyCount() == startingEntityCount + 2)
    assert(mDB.getGroupSize(groupId) == 1)

    val entityId2 = mDB.createEntity(entityName + 2)
    assert(mDB.getEntitiesOnlyCount() == startingEntityCount + 3)
    assert(mDB.getCountOfGroupsContainingEntity(entityId2) == 0)
    group.addEntity(entityId2)
    assert(mDB.getGroupSize(groupId) == 2)
    assert(mDB.getCountOfGroupsContainingEntity(entityId2) == 1)
    val descriptions = mDB.getRelationToGroupDescriptionsContaining(entityId2, Some(9999))
    assert(descriptions.length == 1)
    assert(descriptions(0) == entityName + "->" + relToGroupName)

    //doesn't get an error:
    mDB.deleteEntity(entityId2)

    val descriptions2 = mDB.getRelationToGroupDescriptionsContaining(entityId2, Some(9999))
    assert(descriptions2.length == 0)
    assert(mDB.getCountOfGroupsContainingEntity(entityId2) == 0)
    assert(mDB.getEntitiesOnlyCount() == startingEntityCount + 2)
    assert(intercept[Exception] {
                                  new Entity(mDB, entityId2)
                                }.getMessage.contains("does not exist"))

    assert(mDB.getGroupSize(groupId) == 1)

    val list = mDB.getGroupEntryObjects(groupId, 0)
    assert(list.size == 1)
    val remainingContainedEntityId = list.get(0).getId

    // ensure the first entities still exist: not deleted by that delete command
    new Entity(mDB, entityId)
    new Entity(mDB, remainingContainedEntityId)
  }

  "getSortedAttributes" should "return them all and correctly" in {
    val entityId = mDB.createEntity("test: org.onemodel.PSQLDbTest.testRelsNRelTypes()")
    createTestTextAttributeWithOneEntity(entityId)
    createTestQuantityAttributeWithTwoEntities(entityId)
    val relTypeId: Long = mDB.createRelationType("contains", "", RelationType.UNIDIRECTIONAL)
    createTestRelationToEntity_WithOneEntity(entityId, relTypeId)
    createAndAddTestRelationToGroup_ToEntity(entityId, relTypeId)
    createTestDateAttributeWithOneEntity(entityId)
    createTestBooleanAttributeWithOneEntity(entityId, valIn = false, None, 0)
    createTestFileAttributeAndOneEntity(new Entity(mDB, entityId), "desc", 2, verifyIn = false)

    val (attrTuples: Array[(Long, Attribute)], totalAttrsAvailable) = mDB.getSortedAttributes(entityId, 0, 999)
    val counter: Long = attrTuples.length
    // should be the same since we didn't create enough to span screens (requested them all):
    assert(counter == totalAttrsAvailable)
    if (counter != 7) {
      fail("We added attributes (RelationToEntity, quantity & text, date,bool,file,RTG), but getAttributeIdsAndAttributeTypeIds() returned " + counter + "?")
    }

    var (foundQA, foundTA, foundRTE, foundRTG, foundDA, foundBA, foundFA) = (false, false, false, false, false, false, false)
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
        case attribute: RelationToEntity =>
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

  "entity deletion" should "also delete RelationToEntity attributes" in {
    val entityId = mDB.createEntity("test: org.onemodel.PSQLDbTest.testRelsNRelTypes()")
    val relTypeId: Long = mDB.createRelationType("is sitting next to", "", RelationType.UNIDIRECTIONAL)
    val relatedEntityId: Long = createTestRelationToEntity_WithOneEntity(entityId, relTypeId)
    assert(mDB.getRelationToEntityCount(entityId) == 1)
    mDB.deleteEntity(entityId)
    if (mDB.getRelationToEntityCount(entityId) != 0) {
      fail("Deleting the model entity should also have deleted its RelationToEntity objects. getRelationToEntityCount(entityIdInNewTransaction) is " + mDB
                                                                                                                                                       .getRelationToEntityCount(entityId) + ".")
    }
    assert(intercept[Exception] {
                                  mDB.getRelationToEntityData(relTypeId, entityId, relatedEntityId)
                                }.getMessage.contains("Got 0 instead of 1 result"))

    mDB.deleteRelationType(relTypeId)
  }

  "attributes" should "handle validOnDates properly in & out of db" in {
    val entityId = mDB.createEntity("test: org.onemodel.PSQLDbTest.attributes...")
    val relTypeId = mDB.createRelationType(RELATION_TYPE_NAME, "", RelationType.UNIDIRECTIONAL)
    // create attributes & read back / other values (None alr done above) as entered (confirms read back correctly)
    // (these methods do the checks, internally)
    createTestRelationToEntity_WithOneEntity(entityId, relTypeId, Some(0L))
    createTestRelationToEntity_WithOneEntity(entityId, relTypeId, Some(System.currentTimeMillis()))
    createTestQuantityAttributeWithTwoEntities(entityId)
    createTestQuantityAttributeWithTwoEntities(entityId, Some(0))
    createTestTextAttributeWithOneEntity(entityId)
    createTestTextAttributeWithOneEntity(entityId, Some(0))
  }

  "testAddQuantityAttributeWithBadParentID" should "not work" in {
    System.out.println("starting testAddQuantityAttributeWithBadParentID")
    val badParentId: Long = mDB.findIdWhichIsNotKeyOfAnyEntity

    // Database should not allow adding quantity with a bad parent (Entity) ID!
    // idea: make it a more specific exception type, so we catch only the error we want...
    intercept[Exception] {
                           createTestQuantityAttributeWithTwoEntities(badParentId)
                         }

  }

  private def createTestQuantityAttributeWithTwoEntities(inParentId: Long, inValidOnDate: Option[Long] = None): Long = {
    val unitId: Long = mDB.createEntity("centimeters")
    val attrTypeId: Long = mDB.createEntity(QUANTITY_TYPE_NAME)
    val defaultDate: Long = System.currentTimeMillis
    val validOnDate: Option[Long] = inValidOnDate
    val observationDate: Long = defaultDate
    val number: Float = 50
    val quantityId: Long = mDB.createQuantityAttribute(inParentId, attrTypeId, unitId, number, validOnDate, observationDate)

    // and verify it:
    val qa: QuantityAttribute = new QuantityAttribute(mDB, quantityId)
    assert(qa.getParentId == inParentId)
    assert(qa.getUnitId == unitId)
    assert(qa.getNumber == number)
    assert(qa.getAttrTypeId == attrTypeId)
    if (inValidOnDate.isEmpty) {
      assert(qa.getValidOnDate.isEmpty)
    } else {
      val inDate: Long = inValidOnDate.get
      val gotDate: Long = qa.getValidOnDate.get
      assert(inDate == gotDate)
    }
    assert(qa.getObservationDate == observationDate)
    quantityId
  }

  private def createTestTextAttributeWithOneEntity(inParentId: Long, inValidOnDate: Option[Long] = None): Long = {
    val attrTypeId: Long = mDB.createEntity("textAttributeTypeLikeSsn")
    val defaultDate: Long = System.currentTimeMillis
    val validOnDate: Option[Long] = inValidOnDate
    val observationDate: Long = defaultDate
    val text: String = "some test text"
    val textAttributeId: Long = mDB.createTextAttribute(inParentId, attrTypeId, text, validOnDate, observationDate)

    // and verify it:
    val ta: TextAttribute = new TextAttribute(mDB, textAttributeId)
    assert(ta.getParentId == inParentId)
    assert(ta.getText == text)
    assert(ta.getAttrTypeId == attrTypeId)
    if (inValidOnDate.isEmpty) {
      assert(ta.getValidOnDate.isEmpty)
    } else {
      assert(ta.getValidOnDate.get == inValidOnDate.get)
    }
    assert(ta.getObservationDate == observationDate)

    textAttributeId
  }

  private def createTestDateAttributeWithOneEntity(inParentId: Long): Long = {
    val attrTypeId: Long = mDB.createEntity("dateAttributeType--likeDueOn")
    val date: Long = System.currentTimeMillis
    val dateAttributeId: Long = mDB.createDateAttribute(inParentId, attrTypeId, date)
    val ba: DateAttribute = new DateAttribute(mDB, dateAttributeId)
    assert(ba.getParentId == inParentId)
    assert(ba.getDate == date)
    assert(ba.getAttrTypeId == attrTypeId)
    dateAttributeId
  }

  private def createTestBooleanAttributeWithOneEntity(inParentId: Long, valIn: Boolean, inValidOnDate: Option[Long] = None, inObservationDate: Long): Long = {
    val attrTypeId: Long = mDB.createEntity("boolAttributeType-like-isDone")
    val booleanAttributeId: Long = mDB.createBooleanAttribute(inParentId, attrTypeId, valIn, inValidOnDate, inObservationDate)
    val ba = new BooleanAttribute(mDB, booleanAttributeId)
    assert(ba.getAttrTypeId == attrTypeId)
    assert(ba.getBoolean == valIn)
    assert(ba.getValidOnDate == inValidOnDate)
    assert(ba.getParentId == inParentId)
    assert(ba.getObservationDate == inObservationDate)
    booleanAttributeId
  }

  private def createTestFileAttributeAndOneEntity(inParentEntity: Entity, inDescr: String, addedKiloBytesIn: Int, verifyIn: Boolean = true): FileAttribute = {
    val attrTypeId: Long = mDB.createEntity("fileAttributeType")
    val file: java.io.File = java.io.File.createTempFile("om-test-file-attr-", null)
    var writer: java.io.FileWriter = null
    var verificationFile: java.io.File = null
    try {
      writer = new java.io.FileWriter(file)
      writer.write(addedKiloBytesIn + "+ kB file from: " + file.getCanonicalPath + ", created " + new java.util.Date())
      var nextInteger: Long = 1
      for (i: Int <- 1 to (1000 * addedKiloBytesIn)) {
        // there's a bug here: files aren't the right size (not single digits being added) but oh well it's just to make some file.
        writer.write(nextInteger.toString)
        if (i % 1000 == 0) nextInteger += 1
      }
      writer.close()

      // sleep is so we can see a difference between the 2 dates to be saved, in later assertion.
      val sleepPeriod = 5
      Thread.sleep(sleepPeriod)
      val size = file.length()
      var inputStream: java.io.FileInputStream = null
      var fa: FileAttribute = null
      try {
        inputStream = new java.io.FileInputStream(file)
        fa = inParentEntity.addFileAttribute(attrTypeId, inDescr, file)
      } finally {
        if (inputStream != null) inputStream.close()
      }

      if (verifyIn) {
        // this first part is just testing DB consistency from add to retrieval, not the actual file:
        assert(fa.getParentId == inParentEntity.getId)
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
      if (verificationFile != null) verificationFile.delete()
      if (writer != null) writer.close()
      if (file != null) file.delete()
    }
  }

  private def createTestRelationToEntity_WithOneEntity(inEntityId: Long, inRelTypeId: Long, inValidOnDate: Option[Long] = None): Long = {
    // idea: could use here instead: db.createEntityAndRelationToEntity
    val relatedEntityId: Long = mDB.createEntity(RELATED_ENTITY_NAME)
    val validOnDate: Option[Long] = if (inValidOnDate.isEmpty) None else inValidOnDate
    val observationDate: Long = System.currentTimeMillis
    val id = mDB.createRelationToEntity(inRelTypeId, inEntityId, relatedEntityId, validOnDate, observationDate).getId

    // and verify it:
    val rel: RelationToEntity = new RelationToEntity(mDB, id, inRelTypeId, inEntityId, relatedEntityId)
    if (inValidOnDate.isEmpty) {
      assert(rel.getValidOnDate.isEmpty)
    } else {
      val inDt: Long = inValidOnDate.get
      val gotDt: Long = rel.getValidOnDate.get
      assert(inDt == gotDt)
    }
    assert(rel.getObservationDate == observationDate)
    relatedEntityId
  }

  /** Returns the groupId, and the RTG.
    */
  private def createAndAddTestRelationToGroup_ToEntity(inParentId: Long, inRelTypeId: Long, inGroupName: String = "something",
                                                       inValidOnDate: Option[Long] = None, allowMixedClassesIn: Boolean = true): (Long, RelationToGroup) = {
    val validOnDate: Option[Long] = if (inValidOnDate.isEmpty) None else inValidOnDate
    val observationDate: Long = System.currentTimeMillis
    val (group:Group, rtg: RelationToGroup) = new Entity(mDB, inParentId).addGroupAndRelationToGroup(inRelTypeId, inGroupName, allowMixedClassesIn, validOnDate, observationDate, None)

    // and verify it:
    if (inValidOnDate.isEmpty) {
      assert(rtg.getValidOnDate.isEmpty)
    } else {
      val inDt: Long = inValidOnDate.get
      val gotDt: Long = rtg.getValidOnDate.get
      assert(inDt == gotDt)
    }
    assert(group.getMixedClassesAllowed == allowMixedClassesIn)
    assert(group.getName == inGroupName)
    assert(rtg.getObservationDate == observationDate)
    (group.getId, rtg)
  }

  "rollbackWithCatch" should "catch and return chained exception showing failed rollback" in {
    val db = new PostgreSQLDatabase("abc", "defg") {
      override def connect(inDbName: String, username: String, password: String) {
        // leave it null so calling it will fail as desired below.
        mConn = null
      }
      override def createExpectedData(): Unit = {
        // Overriding because it is not needed for this test, and normally uses mConn, which by being set to null just above, breaks the method.
        // (intentional style violation for readability)
        //noinspection ScalaUselessExpression
        None
      }
      override def modelTablesExist: Boolean = true
      //noinspection ScalaUselessExpression  (intentional style violation, for readability)
      override def doDatabaseUpgradesIfNeeded() = Unit
    }
    var found = false
    val originalErrMsg: String = "testing123"
    try {
      try throw new Exception(originalErrMsg)
      catch {
        case e: Exception => throw db.rollbackWithCatch(e)
      }
    } catch {
      case t: Throwable =>
        found = true
        val sw = new java.io.StringWriter()
        t.printStackTrace(new java.io.PrintWriter(sw))
        val s = sw.toString
        assert(s.contains(originalErrMsg))
        assert(s.contains("See the chained messages for ALL: the cause of rollback failure, AND"))
        assert(s.contains("at org.onemodel.database.PostgreSQLDatabase.rollbackTrans"))
    }
    assert(found)
  }

  "createDefaultData, findEntityOnlyIdsByName, createClassDefiningEntity, findContainedEntries, and findRelationToGroup_OnEntity" should
  "have worked right in earlier db setup and now" in {
    val PERSON_TEMPLATE: String = "person-template"
    val systemEntityId = mDB.getSystemEntityId
    val groupIdOfClassTemplates = mDB.findRelationToAndGroup_OnEntity(systemEntityId, Some(PostgreSQLDatabase.classDefiningEntityGroupName))._3

    // (Should be some value, but the activity on the test DB wouldn't have ids incremented to 0 yet,so that one would be invalid. Could use the
    // other method to find an unused id, instead of 0.)
    assert(groupIdOfClassTemplates.isDefined && groupIdOfClassTemplates.get != 0)
    assert(new Group(mDB, groupIdOfClassTemplates.get).getMixedClassesAllowed)

    val personTemplateEntityId: Long = mDB.findEntityOnlyIdsByName(PERSON_TEMPLATE).get.head
    // idea: make this next part more scala-like (but only if still very simple to read for programmers who are used to other languages):
    var found = false
    val entitiesInGroup: java.util.ArrayList[Entity] = mDB.getGroupEntryObjects(groupIdOfClassTemplates.get, 0)
    for (entity <- entitiesInGroup.toArray) {
      if (entity.asInstanceOf[Entity].getId == personTemplateEntityId) {
        found = true
      }
    }
    assert(found)

    // make sure the other approach also works, even with deeply nested data:
    val relTypeId: Long = mDB.createRelationType("contains", "", RelationType.UNIDIRECTIONAL)
    val te1 = createTestRelationToEntity_WithOneEntity(personTemplateEntityId, relTypeId)
    val te2 = createTestRelationToEntity_WithOneEntity(te1, relTypeId)
    val te3 = createTestRelationToEntity_WithOneEntity(te2, relTypeId)
    val te4 = createTestRelationToEntity_WithOneEntity(te3, relTypeId)
    val foundIds: mutable.TreeSet[Long] = mDB.findContainedEntityIds(new mutable.TreeSet[Long](), systemEntityId, PERSON_TEMPLATE, 4,
                                                                     stopAfterAnyFound = false)
    assert(foundIds.contains(personTemplateEntityId), "Value not found in query: " + personTemplateEntityId)
    val allContainedWithName: mutable.TreeSet[Long] = mDB.findContainedEntityIds(new mutable.TreeSet[Long](), systemEntityId, RELATED_ENTITY_NAME, 4,
                                                                                 stopAfterAnyFound = false)
    // (see idea above about making more scala-like)
    var allContainedIds = ""
    for (id: Long <- allContainedWithName) {
      allContainedIds += id + ", "
    }
    assert(allContainedWithName.size == 3, "Returned set had wrong count (" + allContainedWithName.size + "): " + allContainedIds)
    val te4Entity: Entity = new Entity(mDB, te4)
    te4Entity.addTextAttribute(te1/*not really but whatever*/, RELATED_ENTITY_NAME, None, None, 0)
    val allContainedWithName2: mutable.TreeSet[Long] = mDB.findContainedEntityIds(new mutable.TreeSet[Long](), systemEntityId, RELATED_ENTITY_NAME, 4,
                                                                                  stopAfterAnyFound = false)
    // should be no change yet (added it outside the # of levels to check):
    assert(allContainedWithName2.size == 3, "Returned set had wrong count (" + allContainedWithName.size + "): " + allContainedIds)
    val te2Entity: Entity = new Entity(mDB, te2)
    te2Entity.addTextAttribute(te1/*not really but whatever*/, RELATED_ENTITY_NAME, None, None, 0)
    val allContainedWithName3: mutable.TreeSet[Long] = mDB.findContainedEntityIds(new mutable.TreeSet[Long](), systemEntityId, RELATED_ENTITY_NAME, 4,
                                                                                  stopAfterAnyFound = false)
    // should be no change yet (the entity was already in the return set, so the TA addition didn't add anything)
    assert(allContainedWithName3.size == 3, "Returned set had wrong count (" + allContainedWithName.size + "): " + allContainedIds)
    te2Entity.addTextAttribute(te1/*not really but whatever*/, "otherText", None, None, 0)
    val allContainedWithName4: mutable.TreeSet[Long] = mDB.findContainedEntityIds(new mutable.TreeSet[Long](), systemEntityId, "otherText", 4,
                                                                                  stopAfterAnyFound = false)
    // now there should be a change:
    assert(allContainedWithName4.size == 1, "Returned set had wrong count (" + allContainedWithName.size + "): " + allContainedIds)

    val editorCmd = mDB.getTextEditorCommand
    if (Controller.isWindows) assert(editorCmd.contains("notepad"))
    else assert(editorCmd == "vi")
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
    val name: String = "testing isDuplicateEntity"
    val entityId: Long = mDB.createEntity(name)
    assert(mDB.isDuplicateEntity(name))
    assert(!mDB.isDuplicateEntity(name, Some(entityId)))

    val entityWithSpaceInNameId: Long = mDB.createEntity(name + " ")
    assert(!mDB.isDuplicateEntity(name + " ", Some(entityWithSpaceInNameId)))

    val entityIdWithLowercaseName: Long = mDB.createEntity(name.toLowerCase)
    assert(mDB.isDuplicateEntity(name, Some(entityIdWithLowercaseName)))

    mDB.updateEntityOnlyName(entityId, name.toLowerCase)
    assert(mDB.isDuplicateEntity(name, Some(entityIdWithLowercaseName)))
    assert(mDB.isDuplicateEntity(name, Some(entityId)))

    mDB.deleteEntity(entityIdWithLowercaseName)
    assert(!mDB.isDuplicateEntity(name, Some(entityId)))

    // intentionally put some uppercase letters for later comparison w/ lowercase.
    val relTypeName = name + "-RelationType"
    val relTypeId: Long = mDB.createRelationType("testingOnly", relTypeName, RelationType.UNIDIRECTIONAL)
    assert(mDB.isDuplicateEntity(relTypeName))
    assert(!mDB.isDuplicateEntity(relTypeName, Some(relTypeId)))

    mDB.beginTrans()
    mDB.updateEntityOnlyName(entityId, relTypeName.toLowerCase)
    assert(mDB.isDuplicateEntity(relTypeName, Some(entityId)))
    assert(mDB.isDuplicateEntity(relTypeName, Some(relTypeId)))
    // because setting an entity name to relTypeName doesn't really make sense, was just for that part of the test.
    mDB.rollbackTrans()
  }

  "isDuplicateEntityClass and class update/deletion" should "work" in {
    val name: String = "testing isDuplicateEntityClass"
    val (classId, entityId) = mDB.createClassAndItsDefiningEntity(name)
    assert(EntityClass.isDuplicate(mDB, name))
    assert(!EntityClass.isDuplicate(mDB, name, Some(classId)))

    mDB.updateClassName(classId, name.toLowerCase)
    assert(!EntityClass.isDuplicate(mDB, name, Some(classId)))
    assert(EntityClass.isDuplicate(mDB, name.toLowerCase))
    assert(!EntityClass.isDuplicate(mDB, name.toLowerCase, Some(classId)))
    mDB.updateClassName(classId, name)

    mDB.updateEntitysClass(entityId, None)
    mDB.deleteClassAndItsDefiningEntity(classId)
    assert(!EntityClass.isDuplicate(mDB, name, Some(classId)))
    assert(!EntityClass.isDuplicate(mDB, name))
  }

  "EntitiesInAGroup and getclasses/classcount methods" should "work, and should enforce class_id uniformity within a group of entities" in {
    // ...for now anyway. See comments at this table in psqld.createTables and/or hasMixedClasses.

    // This also tests db.createEntity and db.updateEntityOnlyClass.

    val entityName = "test: PSQLDbTest.testgroup-class-uniqueness" + "--theEntity"
    val (classId, entityId) = mDB.createClassAndItsDefiningEntity(entityName)
    val (classId2, entityId2) = mDB.createClassAndItsDefiningEntity(entityName + 2)
    val classCount = mDB.getClassCount()
    val classes = mDB.getClasses(0)
    assert(classCount == classes.size)
    val classCountLimited = mDB.getClassCount(Some(entityId2))
    assert(classCountLimited == 1)

    //whatever, just need some relation type to go with:
    val relTypeId: Long = mDB.createRelationType("contains", "", RelationType.UNIDIRECTIONAL)
    val groupId = createAndAddTestRelationToGroup_ToEntity(entityId, relTypeId, "test: PSQLDbTest.testgroup-class-uniqueness", Some(12345L),
                                                                allowMixedClassesIn = false)._1
    val group: Group = new Group(mDB, groupId)
    assert(! mDB.isEntityInGroup(groupId, entityId))
    assert(! mDB.isEntityInGroup(groupId, entityId))
    group.addEntity(entityId)
    assert(mDB.isEntityInGroup(groupId, entityId))
    assert(! mDB.isEntityInGroup(groupId, entityId2))

    //should fail due to mismatched classId (a long):
    assert(intercept[Exception] {
                                  group.addEntity(entityId2)
                                }.getMessage.contains(PostgreSQLDatabase.MIXED_CLASSES_EXCEPTION))
    // should succeed (same class now):
    mDB.updateEntitysClass(entityId2, Some(classId))
    group.addEntity(entityId2)
    // ...and for convenience while here, make sure we can't make mixed classes with changing the *entity* either:
    assert(intercept[Exception] {
                                  mDB.updateEntitysClass(entityId2, Some(classId2))
                                }.getMessage.contains(PostgreSQLDatabase.MIXED_CLASSES_EXCEPTION))
    assert(intercept[Exception] {
                                  mDB.updateEntitysClass(entityId2, None)
                                }.getMessage.contains(PostgreSQLDatabase.MIXED_CLASSES_EXCEPTION))

    //should fail due to mismatched classId (NULL):
    val entityId3 = mDB.createEntity(entityName + 3)
    assert(intercept[Exception] {
                                  group.addEntity(entityId3)
                                }.getMessage.contains(PostgreSQLDatabase.MIXED_CLASSES_EXCEPTION))

    assert(!mDB.areMixedClassesAllowed(groupId))


    val systemEntityId = mDB.getSystemEntityId
    // idea: (noted at other use of this method)
    val classGroupId = mDB.findRelationToAndGroup_OnEntity(systemEntityId, Some(PostgreSQLDatabase.classDefiningEntityGroupName))._3
    assert(mDB.areMixedClassesAllowed(classGroupId.get))

    val groupSizeBeforeRemoval = mDB.getGroupSize(groupId)

    assert(mDB.getGroupSize(groupId, Some(true)) == 0)
    assert(mDB.getGroupSize(groupId, Some(false)) == groupSizeBeforeRemoval)
    assert(mDB.getGroupSize(groupId, None) == groupSizeBeforeRemoval)
    mDB.archiveEntity(entityId2)
    assert(mDB.getGroupSize(groupId, Some(true)) == 1)
    assert(mDB.getGroupSize(groupId, Some(false)) == groupSizeBeforeRemoval - 1)
    assert(mDB.getGroupSize(groupId, None) == groupSizeBeforeRemoval)

    mDB.removeEntityFromGroup(groupId, entityId2)
    val groupSizeAfterRemoval = mDB.getGroupSize(groupId)
    assert(groupSizeAfterRemoval < groupSizeBeforeRemoval)

    assert(mDB.getGroupSize(groupId, Some(true)) == 0)
    assert(mDB.getGroupSize(groupId, Some(false)) == groupSizeBeforeRemoval - 1)
    assert(mDB.getGroupSize(groupId, None) == groupSizeBeforeRemoval - 1)
  }

  "getEntitiesOnly and ...Count" should "allow limiting results by classId and/or group containment" in {
    // idea: this could be rewritten to not depend on pre-existing data to fail when it's supposed to fail.
    val startingEntityCount = mDB.getEntitiesOnlyCount()
    val someClassId: Long = mDB.dbQueryWrapperForOneRow("select id from class limit 1", "Long")(0).get.asInstanceOf[Long]
    val numEntitiesInClass = mDB.extractRowCountFromCountQuery("select count(1) from entity where class_id=" + someClassId)
    assert(startingEntityCount > numEntitiesInClass)
    val allEntitiesInClass = mDB.getEntitiesOnly(0, None, Some(someClassId), limitByClass = true)
    val allEntitiesInClassCount1 = mDB.getEntitiesOnlyCount(Some(someClassId), limitByClass = true)
    val allEntitiesInClassCount2 = mDB.getEntitiesOnlyCount(Some(someClassId), limitByClass = true, None)
    assert(allEntitiesInClassCount1 == allEntitiesInClassCount2)
    val definingClassId: Long = new EntityClass(mDB, someClassId).getDefiningEntityId
    val allEntitiesInClassCountWoClass = mDB.getEntitiesOnlyCount(Some(someClassId), limitByClass = true, Some(definingClassId))
    assert(allEntitiesInClassCountWoClass == allEntitiesInClassCount1 - 1)
    assert(allEntitiesInClass.size == allEntitiesInClassCount1)
    assert(allEntitiesInClass.size < mDB.getEntitiesOnly(0, None, Some(someClassId), limitByClass = false).size)
    assert(allEntitiesInClass.size == numEntitiesInClass)
    val e: Entity = allEntitiesInClass.get(0)
    assert(e.getClassId.get == someClassId)

    // part 2:
    // some setup, confirm good
    val startingEntityCount2 = mDB.getEntitiesOnlyCount()
    val relTypeId: Long = mDB.createRelationType("contains", "", RelationType.UNIDIRECTIONAL)
    val id1: Long = mDB.createEntity("name1")
    val group: Group = new Entity(mDB, id1).addGroupAndRelationToGroup(relTypeId, "someRelToGroupName", allowMixedClassesInGroupIn = false, None, 1234L,
                                                                       None, callerManagesTransactionsIn = false)._1
    val id2: Long = mDB.createEntity("name2")
    group.addEntity(id2)
    val entityCountAfterCreating = mDB.getEntitiesOnlyCount()
    assert(entityCountAfterCreating == startingEntityCount2 + 2)
    val resultSize = mDB.getEntitiesOnly(0).size()
    assert(entityCountAfterCreating == resultSize)
    val resultSizeWithNoneParameter = mDB.getEntitiesOnly(0, None, groupToOmitIdIn = None).size()
    assert(entityCountAfterCreating == resultSizeWithNoneParameter)

    // the real part 2 test
    val resultSizeWithGroupOmission = mDB.getEntitiesOnly(0, None, groupToOmitIdIn = Some(group.getId)).size()
    assert(entityCountAfterCreating - 1 == resultSizeWithGroupOmission)
  }

  "EntitiesInAGroup table (or methods? ick)" should "allow all a group's entities to have no class" in {
    // ...for now anyway.  See comments at this table in psqld.createTables and/or hasMixedClasses.

    val entityName = "test: PSQLDbTest.testgroup-class-allowsAllNulls" + "--theEntity"
    val (classId, entityId) = mDB.createClassAndItsDefiningEntity(entityName)
    val relTypeId: Long = mDB.createRelationType("contains", "", RelationType.UNIDIRECTIONAL)
    val groupId = createAndAddTestRelationToGroup_ToEntity(entityId, relTypeId, "test: PSQLDbTest.testgroup-class-allowsAllNulls", Some(12345L),
                                                                allowMixedClassesIn = false)._1
    val group: Group = new Group(mDB, groupId)
    // 1st one has a NULL class_id
    val entityId3 = mDB.createEntity(entityName + 3)
    group.addEntity(entityId3)
    // ...so it works to add another one that's NULL
    val entityId4 = mDB.createEntity(entityName + 4)
    group.addEntity(entityId4)
    // but adding one with a class_id fails w/ mismatch:
    val entityId5 = mDB.createEntity(entityName + 5, Some(classId))
    assert(intercept[Exception] {
                                  group.addEntity(entityId5)
                                }.getMessage.contains(PostgreSQLDatabase.MIXED_CLASSES_EXCEPTION))
  }

  "getEntitiesOnlyCount" should "not count entities used as relation types or attribute types" in {
    val entityId = mDB.createEntity("test: org.onemodel.PSQLDbTest.getEntitiesOnlyCount")
    val c1 = mDB.getEntitiesOnlyCount()
    assert(mDB.getEntitiesOnlyCount() == c1)
    val relTypeId: Long = mDB.createRelationType("contains", "", RelationType.UNIDIRECTIONAL)
    assert(mDB.getEntitiesOnlyCount() == c1)
    createTestRelationToEntity_WithOneEntity(entityId, relTypeId)
    val c2 = c1 + 1
    assert(mDB.getEntitiesOnlyCount() == c2)

    // this kind shouldn't matter--confirming:
    val relTypeId2: Long = mDB.createRelationType("contains2", "", RelationType.UNIDIRECTIONAL)
    createAndAddTestRelationToGroup_ToEntity(entityId, relTypeId2)
    assert(mDB.getEntitiesOnlyCount() == c2)

    createTestDateAttributeWithOneEntity(entityId)
    assert(mDB.getEntitiesOnlyCount() == c2)

    createTestBooleanAttributeWithOneEntity(entityId, valIn = false, None, 0)
    assert(mDB.getEntitiesOnlyCount() == c2)

    createTestFileAttributeAndOneEntity(new Entity(mDB, entityId), "desc", 2, verifyIn = false)
    assert(mDB.getEntitiesOnlyCount() == c2)

  }

  "getMatchingEntities" should "work" in {
    val entityId1 = mDB.createEntity("test: org.onemodel.PSQLDbTest.getMatchingEntities1--abc")
    val entityId2 = mDB.createEntity("test: org.onemodel.PSQLDbTest.getMatchingEntities2")
    mDB.createTextAttribute(entityId1, entityId2, "defg", None, 0)
    val entities1 = mDB.getMatchingEntities(0, None, None, "abc")
    assert(entities1.size == 1)
    mDB.createTextAttribute(entityId2, entityId1, "abc", None, 0)
    val entities2 = mDB.getMatchingEntities(0, None, None, "abc")
    assert(entities2.size == 2)
  }

  //idea: should this be moved to ImportExportTest? why did i put it here originally?
  "getJournal" should "show activity during a date range" in {
    val startDataSetupTime = System.currentTimeMillis()
    val entityId: Long = mDB.createEntity("test object")
    val entity: Entity = new Entity(mDB, entityId)
    // (idea: next line should be fixed: see cmt at similar usage in ImportExportTest.scala:)
    val importExport = new ImportExport(null, mDB, new Controller(null, false, Some(PostgreSQLDatabaseTest.TEST_USER), Some(PostgreSQLDatabaseTest.TEST_USER)))
    val importFile: File = importExport.tryImporting_FOR_TESTS("testImportFile0.txt", entity)
    val ids: Option[List[Long]] = mDB.findAllEntityIdsByName("vsgeer-testing-getJournal-in-db")
    val (fileContents: String, outputFile: File) = importExport.tryExportingTxt_FOR_TESTS(ids, mDB)
    // (next 3 lines are redundant w/ a similar test in ImportExportTest, but are here to make sure the data
    // is as expected before proceeding with the actual purpose of this test:)
    assert(fileContents.contains("vsgeer"), "unexpected file contents:  " + fileContents)
    assert(fileContents.contains("record/report/review"), "unexpected file contents:  " + fileContents)
    assert(outputFile.length == importFile.length)

    mDB.archiveEntity(entityId)
    val endDataSetupTime = System.currentTimeMillis()

    val results: Array[(Long, String, Long)] = mDB.findJournalEntries(startDataSetupTime, endDataSetupTime)
    assert(results.length > 0)
  }

  "getTextAttributeByNameForEntity" should "fail when no rows found" in {
    intercept[OmDatabaseException] {
                                     val systemEntityId = mDB.getSystemEntityId
                                     mDB.getTextAttributeByTypeId(systemEntityId, 1L, Some(1))
                                   }
  }

}