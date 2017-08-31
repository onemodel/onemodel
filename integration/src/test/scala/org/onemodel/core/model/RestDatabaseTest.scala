/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2016-2017 inclusive, Luke A. Call; all rights reserved.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule, guidelines around binary
    distribution, and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>

    This file is in the integration module because it exercises code that spans other modules (currently the core module as exposed via the web module,
    and confirming some operations by hitting the core module directly.).
*/
package org.onemodel.core.model

import java.io.{File, FileOutputStream}
import java.util

import org.onemodel.core._
import org.scalatest.mockito.MockitoSugar
import org.scalatest.{Args, FlatSpec, Status}
import scala.collection._
import scala.collection.JavaConversions._

class RestDatabaseTest extends FlatSpec with MockitoSugar {
//  Comment out the next line after this comment (i.e., put "//" in front of the "/*") to make these tests run, but don't commit it
//  that way (yet).  They are not currently run automatically
//  because nothing is in place to start the required web server automatically.  To start it manually:
//    - install sbt (search the web for how to do it; as of 2017-7-29 I am using version 0.13.11.)
//    - at the command-line, cd to the "core" module, and run "mvn clean install" (to make the latest core code changes available to the web server).
//    - cd into the "web" module and run "sbt"
//    - inside sbt type the command "~ run"
//    - then this test class can run.
//
//  For details, see the URLs in RestDatabase.scala which mention playframework.com, and see the below
//  call to "new RestDatabase".  Before the web module can build, "mvn install" has to be run in the core module.
//  NOTE: WHEN MAKING THESE AUTOMATICALLY RUN AS PART OF "mvn install" OR "mvn verify", BE SURE TO UPDATE THE "INSTALLING" DOCUMENT(s),
//  AND when updating those docs, keep in mind what EntityMenuIT.java says about the testsuite/README file, so that one coming to the
//  project for the first time has an overall guide for a good dev on-boarding experience.
/*

  private val mPG: PostgreSQLDatabase = new PostgreSQLDatabase(Database.TEST_USER, Database.TEST_PASS)
  // mRD will access mPG via REST, in the tests, so this tests both web module code and core code.
  private val mRD: RestDatabase = new RestDatabase("localhost:9000")

  override def runTests(testName: Option[String], args: Args): Status = {
    val result: Status = super.runTests(testName, args)
    result
  }

  "start" should "work" in {
    assert(mRD.isRemote)
    assert(mRD.id.length > 30)

    mPG.setUserPreference_EntityId(Util.DEFAULT_ENTITY_PREFERENCE, mPG.getSystemEntityId)
    val defaultEntityId = mRD.getDefaultEntityId
    assert(defaultEntityId < 0)
  }

  "getGroupSize etc" should "work" in {
    val entityId0 = mPG.createEntity("test: org.onemodel.RestDatabaseTest.getGroupSize-e0")
    val entity0 = new Entity(mPG, entityId0)
    val relationTypeNameBase = "contains" + Math.random()
    val relationTypeName = relationTypeNameBase.substring(0, Math.min(Database.relationTypeNameLength, relationTypeNameBase.length))
    val relTypeId: Long = mPG.createRelationType(relationTypeName, "", RelationType.UNIDIRECTIONAL)
    assert(mRD.relationTypeKeyExists(relTypeId))
    val relationTypes = mRD.findRelationType(relationTypeName, None)
    assert(relationTypes.size == 1)
    assert(relationTypes.get(0) == relTypeId)
    assert(mRD.findRelationType(relationTypeName, Some(1)).size == 1)

    val grpCount = mRD.getGroupCount
    val grpName = "getGroupSize-testGrp"
    val (groupId, relationToGroup) = DatabaseTestUtils.createAndAddTestRelationToGroup_ToEntity(mPG, entityId0, relTypeId, grpName)
    val grpCount2 = mRD.getGroupCount
    assert(grpCount2 == grpCount + 1)
    val rtgCount = mRD.getRelationToGroupCount(entityId0)
    assert(rtgCount == 1)
    assert(mRD.getGroupSize(groupId) == 0)
    val group = new Group(mPG, groupId)
    group.addEntity(entityId0)
    assert(mRD.isEntityInGroup(groupId, entityId0))
    entity0.archive()
    val entityId1 = mPG.createEntity("test: getGroupSize-e1")
    val entity1 = new Entity(mPG, entityId1)
    group.addEntity(entityId1)
    assert(mRD.getGroupSize(groupId) == 2)
    assert(mRD.getGroupSize(groupId, 1) == 1)
    assert(mRD.getGroupSize(groupId, 2) == 1)
    assert(mRD.getGroupSize(groupId, 3) == 2)
    entity0.unarchive()
    assert(mRD.getGroupSize(groupId, 1) == 2)
    assert(mRD.getGroupSize(groupId, 2) == 0)
    assert(mRD.getGroupSize(groupId, 3) == 2)

    val entitiesContainingGroup: util.ArrayList[(Long, Entity)] = mRD.getEntitiesContainingGroup(groupId, 0)
    assert(entitiesContainingGroup.size == 1)
    assert(entitiesContainingGroup.get(0)._1 == relTypeId)
    assert(entitiesContainingGroup.get(0)._2.getId == entityId0)

    val count: (Long, Long) = mRD.getCountOfEntitiesContainingGroup(groupId)
    assert(count._1 == 1)
    entity1.addRelationToGroup(relationToGroup.getAttrTypeId, groupId, sortingIndexIn = None)
    val count2 = mRD.getCountOfEntitiesContainingGroup(groupId)
    assert(count2._1 == 2)

    entity0.addHASRelationToLocalEntity(entityId1, None, 0)
    val count3: (Long, Long) = mRD.getCountOfLocalEntitiesContainingLocalEntity(entityId1)
    assert(count3._1 == 1 && count3._2 == 0)

    def foundInResults(resultsIn: util.ArrayList[Group], idIn: Long): Boolean = {
      var found = false
      for (group: Group <- resultsIn) {
        if (group.getId == idIn) {
          found = true
        }
      }
      found
    }
    val (_, _ /*groupId2, relationToGroup2*/) = DatabaseTestUtils.createAndAddTestRelationToGroup_ToEntity(mPG, entityId0, relTypeId, grpName + "2")
    val (_, _ /*groupId3, relationToGroup3*/) = DatabaseTestUtils.createAndAddTestRelationToGroup_ToEntity(mPG, entityId0, relTypeId, grpName + "3")
    val groupsMatching: util.ArrayList[Group] = mRD.getMatchingGroups(0, None, None, grpName)
    assert(groupsMatching.size >= 3)
    assert(foundInResults(groupsMatching, groupId))
    val groupsMatching2 = mRD.getMatchingGroups(0, Some(2), None, grpName)
    assert(groupsMatching2.size == 2)
    val groupsMatching3 = mRD.getMatchingGroups(0, None, Some(groupId), grpName)
    assert(! foundInResults(groupsMatching3, groupId))
    assert(groupsMatching3.size == groupsMatching.size - 1)

    val groups = mRD.getGroups(0, None)
    assert(groups.size > 0)
    val groups2 = mRD.getGroups(0, None, Some(groupId))
    assert(groups2.size == groups.size - 1)
    val groups3 = mRD.getGroups(0, Some(1))
    assert(groups3.size == 1)
  }

  "SortingIndex and counting methods etc" should "work" in {
    val namePrefix = "test: findUnusedGroupSortingIndex"
    val entityOnlyCount = mRD.getEntitiesOnlyCount()
    assert(mRD.getEntitiesOnly(0).size == entityOnlyCount)
    val entityCount = mRD.getEntityCount
    val entityId0 = mPG.createEntity(namePrefix + "-e0")
    assert(mRD.getEntityCount == entityCount + 1)
    val relTypeId: Long = mPG.createRelationType("contains", "", RelationType.BIDIRECTIONAL)
    val (groupId, relationToGroup) = DatabaseTestUtils.createAndAddTestRelationToGroup_ToEntity(mPG, entityId0, relTypeId,
                                                                                                "findUnusedGroupSortingIndex-testGrp")
    assert(mRD.relationToGroupKeysExistAndMatch(relationToGroup.getId, entityId0, relTypeId, groupId))
    assert(mRD.groupKeyExists(groupId))
    val group = new Group(mPG, groupId)
    val entityId1 = mPG.createEntity(namePrefix + "-e1")
    group.addEntity(entityId1)
    val highestUsedIndex = mRD.getHighestSortingIndexForGroup(groupId)
    val onlyUsedIndexInGroup = mRD.getGroupEntrySortingIndex(groupId, entityId1)
    assert(onlyUsedIndexInGroup == highestUsedIndex)
    val groupUnusedIndex = mRD.findUnusedGroupSortingIndex(groupId, Some(-33))
    assert(groupUnusedIndex != highestUsedIndex)
    assert(mRD.isGroupEntrySortingIndexInUse(groupId, onlyUsedIndexInGroup))
    assert(!mRD.isGroupEntrySortingIndexInUse(groupId, groupUnusedIndex))

    val nearestEntrysIndex: Option[Long] = mRD.getNearestGroupEntrysSortingIndex(groupId, onlyUsedIndexInGroup, forwardNotBackIn = true)
    assert(nearestEntrysIndex.isEmpty)
    val entityCountAfter = mRD.getEntitiesOnlyCount()
    assert(entityCountAfter == (entityOnlyCount + 2))

    val oneUsedIndexInGroup: Long = onlyUsedIndexInGroup
    //noinspection SpellCheckingInspection
    val name2base = "FUGSI-e2" + Math.random()
    val testName2 = name2base.substring(0, Math.min(Database.entityNameLength, name2base.length))
    val entityId2 = mPG.createEntity(testName2)
    val entityId3 = mPG.createEntity(namePrefix + "-e3")
    val entityId4 = mPG.createEntity(namePrefix + "-e4")
    group.addEntity(entityId2)
    val aNewlyUsedIndexInGroup = mRD.getHighestSortingIndexForGroup(groupId)
    group.addEntity(entityId3)
    group.addEntity(entityId4)
    val adjacentIndexes1: List[Array[Option[Any]]] = mRD.getAdjacentGroupEntriesSortingIndexes(groupId, oneUsedIndexInGroup, forwardNotBackIn = true)
    assert(aNewlyUsedIndexInGroup == adjacentIndexes1.head(0).get.asInstanceOf[Long])
    assert(adjacentIndexes1.size == 3)
    val adjacentIndexes2: List[Array[Option[Any]]] = mRD.getAdjacentGroupEntriesSortingIndexes(groupId, oneUsedIndexInGroup, Some(1), forwardNotBackIn = true)
    assert(adjacentIndexes2.size == 1)
    assert(adjacentIndexes2.head(0).get.asInstanceOf[Long] == aNewlyUsedIndexInGroup)

    val countOfGroupsContaining: Long = mRD.getCountOfGroupsContainingEntity(entityId1)
    assert(countOfGroupsContaining == 1)
    assert(mRD.getContainingRelationsToGroup(entityId1, 0).size == 1)
    val containingGroupsIds: util.ArrayList[Long] = mRD.getContainingGroupsIds(entityId1)
    assert(containingGroupsIds.size == 1)
    assert(containingGroupsIds.get(0) == groupId)

    val unusedAttributeSortingIndex = mRD.findUnusedAttributeSortingIndex(entityId0)
    val unusedAttributeSortingIndex2 = mRD.findUnusedAttributeSortingIndex(entityId0, Some(0))

    val formId = Database.getAttributeFormId(Util.RELATION_TO_GROUP_TYPE)
    val attrIndex = mRD.getEntityAttributeSortingIndex(entityId0, formId, relationToGroup.getId)
    assert(mRD.isAttributeSortingIndexInUse(entityId0, attrIndex))
    assert(!mRD.isAttributeSortingIndexInUse(entityId0, unusedAttributeSortingIndex))
    assert(!mRD.isAttributeSortingIndexInUse(entityId0, unusedAttributeSortingIndex2))
    assert(mRD.attributeKeyExists(formId, relationToGroup.getId))

    val nearestAttributesIndex = mRD.getNearestAttributeEntrysSortingIndex(entityId0, attrIndex, forwardNotBackIn = true)
    assert(nearestAttributesIndex.isEmpty)

    val containedEntityIds = mRD.findContainedLocalEntityIds(new mutable.TreeSet[Long](), entityId0, namePrefix)
    assert(containedEntityIds.size == 3)

    val foundEntityIdsByName = mRD.findAllEntityIdsByName(testName2.toLowerCase)
    assert(foundEntityIdsByName.size == 1)
    val foundEntityIdsByName2 = mRD.findAllEntityIdsByName(testName2.toLowerCase, caseSensitive = true)
    assert(foundEntityIdsByName2.size == 0)

    val entries: util.ArrayList[Entity] = mRD.getGroupEntryObjects(groupId, 0)
    assert(entries.size == 4)
  }

  "getAttributeCount etc" should "work" in {
    val name = "test: getAttributeCount-e0-" + Math.random()
    val entityId0 = mPG.createEntity(name)
    assert(mRD.isDuplicateEntityName(name))
    assert(!mRD.isDuplicateEntityName(name, Some(entityId0)))
    val entity0 = new Entity(mPG, entityId0)
    val (e1, _): (Entity, RelationToLocalEntity) = entity0.createEntityAndAddHASLocalRelationToIt("test: getAttributeCount-e1", 0, None)
    entity0.createEntityAndAddHASLocalRelationToIt("test: getAttributeCount-e2", 0, None)

    assert(mRD.getRelationToLocalEntityCount(entityId0, includeArchivedEntitiesIn = false) == 2)
    assert(mRD.getRelationToLocalEntityCount(entityId0, includeArchivedEntitiesIn = true) == 2)
    e1.archive()
    assert(mRD.getRelationToLocalEntityCount(entityId0, includeArchivedEntitiesIn = false) == 1)
    assert(mRD.getRelationToLocalEntityCount(entityId0, includeArchivedEntitiesIn = true) == 2)

    val countWithArchived = mRD.getAttributeCount(entityId0, includeArchivedEntitiesIn = true)
    val countWithoutArchived = mRD.getAttributeCount(entityId0, includeArchivedEntitiesIn = false)
    assert(countWithArchived == (countWithoutArchived + 1))
  }

  "classes" should "work" in {
    val classCount1 = mRD.getClassCount()
    val className = "test classes in RDT-" + Math.random()
    val (classId1, entityId1): (Long, Long) = mPG.createClassAndItsTemplateEntity(className)
    assert(mRD.isDuplicateClassName(className))
    assert(!mRD.isDuplicateClassName(className + Math.random()))
    assert(mRD.getClassName(classId1).get == className)
    mPG.updateClassName(classId1, "")
    val blankName = mRD.getClassName(classId1)
    assert(blankName.isEmpty || blankName.get.isEmpty, "Unexpected nonempty value: " + blankName)
    mPG.updateClassName(classId1, className)
    val classCount2 = mRD.getClassCount()
    assert(classCount2 == (classCount1 + 1))
    val classCount2a = mRD.getClassCount(Some(entityId1))
    assert(classCount2a == 1)

    val entityId2 = mPG.createEntity("test: classes-e0")
    val classCount3 = mRD.getClassCount(Some(entityId2))
    assert(classCount3 == 0)
    assert(mRD.classKeyExists(classId1))

    mPG.updateClassCreateDefaultAttributes(classId1, Some(false))
    val should1: Option[Boolean] = new EntityClass(mRD, classId1).getCreateDefaultAttributes
    assert(!should1.get)
    mPG.updateClassCreateDefaultAttributes(classId1, None)
    val should2: Option[Boolean] = new EntityClass(mRD, classId1).getCreateDefaultAttributes
    assert(should2.isEmpty)
    mPG.updateClassCreateDefaultAttributes(classId1, Some(true))
    val should3: Option[Boolean] = new EntityClass(mRD, classId1).getCreateDefaultAttributes
    assert(should3.get)

    val classData: Array[Option[Any]] = mRD.getClassData(classId1)
    // (see  Database.getClassData_resultTypes)
    assert(classData(0).get.asInstanceOf[String] == className)
    assert(classData(1).get.asInstanceOf[Long] == entityId1)
    assert(classData(2).get.asInstanceOf[Boolean])
    mPG.updateClassCreateDefaultAttributes(classId1, None)
    val classData2: Array[Option[Any]] = mRD.getClassData(classId1)
    assert(classData2(2).isEmpty)

    // (hopefully this id is unused:)
    val nonexistentId: Long = Database.maxIdValue
    require(!mPG.classKeyExists(nonexistentId))
    val classDataForBadId: Array[Option[Any]] = mRD.getClassData(nonexistentId)
    assert(classDataForBadId.length == 0)

    val classes = mRD.getClasses(0, None)
    assert(classes.size > 0)
    val classes2 = mRD.getClasses(0, Some(1))
    assert(classes2.size == 1)
  }

  "other exists tests" should "work" in {
    val e1Name = "test: others-e0-" + Math.random()
    val entityId0 = mPG.createEntity(e1Name)
    assert(mRD.getEntityName(entityId0).get == e1Name)
    val entity0 = new Entity(mPG, entityId0)
    mPG.updateEntityOnlyName(entityId0, "")
    val blankName = mRD.getEntityName(entityId0)
    assert(blankName.isEmpty || blankName.get.isEmpty, "Unexpected nonempty value: " + blankName)
    mPG.createEntity("test: others-e1-" + Math.random())
    assert(mRD.entityKeyExists(entityId0))
    assert(mRD.entityKeyExists(entityId0, includeArchived = false))
    entity0.archive()
    assert(!mRD.entityKeyExists(entityId0, includeArchived = false))
  }

  //  "setIncludeArchivedEntities and check" should "work" in {
  // idea: can put these back when (some decisions and) code are in place for *write* access (at least for this non-persistent variable).
  // Maybe it should be stateless, ie, a parm on every request, instead, or just not provide this via REST?:
  //    mRD.setIncludeArchivedEntities(in = true)
  //    assert(mRD.includeArchivedEntities)
  //    mRD.setIncludeArchivedEntities(in = false)
  //    assert(!mRD.includeArchivedEntities)
  //    mRD.setIncludeArchivedEntities(in = true)
  //    assert(mRD.includeArchivedEntities)
  //  }

  "getRelationTypeData and similar" should "work" in {
    val rtCount = mRD.getRelationTypeCount
    val relTypeId: Long = mPG.createRelationType("contains", "", RelationType.UNIDIRECTIONAL)
    assert(mRD.getRelationTypeCount == rtCount + 1)
    val relationType = new RelationType(mPG, relTypeId)
    val relationTypeData: Array[Option[Any]] = mRD.getRelationTypeData(relTypeId)
    // (see  Database.getRelationTypeData_resultTypes)
    assert(relationTypeData(0).get.asInstanceOf[String] == relationType.getName)
    assert(relationTypeData(1).get.asInstanceOf[String] == relationType.getNameInReverseDirection)
    assert(relationTypeData(2).get.asInstanceOf[String] == relationType.getDirectionality)
    assert(relationTypeData.length == 3)

    val relTypes = mRD.getRelationTypes(0, None)
    assert(relTypes.size > 0)
    val relTypes2 = mRD.getRelationTypes(0, Some(1))
    assert(relTypes2.size == 1)
  }

  "getOmInstanceData" should "work" in {
    val omiCount = mRD.getOmInstanceCount
    val uuid = java.util.UUID.randomUUID().toString
    val omInstance: OmInstance = OmInstance.create(mPG, uuid, "test: getRelationTypeData-" + uuid)
    assert(mRD.getOmInstanceCount == omiCount + 1)
    val omInstanceData = mRD.getOmInstanceData(omInstance.getId)
    assert(mRD.omInstanceKeyExists(omInstance.getId))
    assert(mRD.isDuplicateOmInstanceAddress(omInstance.getAddress))
    assert(!mRD.isDuplicateOmInstanceAddress(omInstance.getAddress, Some(omInstance.getId)))

    // (see Database.getOmInstanceData_resultTypes)
    assert(omInstanceData(0).get.asInstanceOf[Boolean] == omInstance.getLocal)
    assert(omInstanceData(1).get.asInstanceOf[String] == omInstance.getAddress)
    assert(omInstanceData(2).get.asInstanceOf[Long] == omInstance.getCreationDate)
    assert(omInstanceData(3) == omInstance.getEntityId)
    assert(omInstanceData.length == 4)
  }

  "file stuff" should "work" in {
    val testEntityId1: Long = mPG.createEntity("test entity for multiple tests1")
    val testEntity1: Entity = new Entity(mPG, testEntityId1)
    val testEntityId2: Long = mPG.createEntity("test entity for multiple tests2")
    val (f: File, fa: FileAttribute) = createFileAttribute(testEntity1, testEntityId2)
    assert(mRD.fileAttributeKeyExists(fa.getId))

    val outputStream: FileOutputStream = new FileOutputStream(f)
    mRD.getFileAttributeContent(fa.getId, outputStream)
    outputStream.close()
    val contentRetrievedViaRest_md5hash = FileAttribute.md5Hash(f)
    val localDbStoredMd5Hash = fa.getMd5Hash
    assert(contentRetrievedViaRest_md5hash == localDbStoredMd5Hash)

    // (see  Database.getFileAttributeData_resultTypes)
    val faData = mRD.getFileAttributeData(fa.getId)
    assert(faData(0).get.asInstanceOf[Long] == fa.getParentId)
    assert(faData(1).get.asInstanceOf[String] == fa.getDescription)
    assert(faData(2).get.asInstanceOf[Long] == fa.getAttrTypeId)
    assert(faData(3).get.asInstanceOf[Long] == fa.getOriginalFileDate)
    assert(faData(4).get.asInstanceOf[Long] == fa.getStoredDate)
    assert(faData(5).get.asInstanceOf[String] == fa.getOriginalFilePath)
    assert(faData(6).get.asInstanceOf[Boolean] == fa.getReadable)
    assert(faData(7).get.asInstanceOf[Boolean] == fa.getWritable)
    assert(faData(8).get.asInstanceOf[Boolean] == fa.getExecutable)
    assert(faData(9).get.asInstanceOf[Long] == fa.getSize)
    assert(faData(10).get.asInstanceOf[String] == fa.getMd5Hash)
    assert(faData(11).get.asInstanceOf[Long] == fa.getSortingIndex)
    assert(faData.length == 12)
  }

  def createFileAttribute(onEntity: Entity, attributeTypeId: Long): (File, FileAttribute) = {
    val f = java.io.File.createTempFile("/tmp/some-" + Math.random(), ".txt")
    f.deleteOnExit()
    val path = f.toPath
    java.nio.file.Files.write(path, Array[Byte]('x', 'y'))
    val fa: FileAttribute = onEntity.addFileAttribute(attributeTypeId, "deletable in a second", f)
    (f, fa)
  }

  "quantity stuff" should "work" in {
    val testEntityId1: Long = mPG.createEntity("test entity for multiple tests1")
    val testEntity1: Entity = new Entity(mPG, testEntityId1)
    val testEntityId2: Long = mPG.createEntity("test entity for multiple tests2")
    val qa: QuantityAttribute = testEntity1.addQuantityAttribute(testEntityId2, testEntityId2, 0, None)
    assert(mRD.quantityAttributeKeyExists(qa.getId))

    // (see Database.getQuantityAttributeData_resultTypes)
    val qaData = mRD.getQuantityAttributeData(qa.getId)
    assert(qaData(0).get.asInstanceOf[Long] == qa.getParentId)
    assert(qaData(1).get.asInstanceOf[Long] == qa.getUnitId)
    assert(qaData(2).get.asInstanceOf[Float] == qa.getNumber)
    assert(qaData(3).get.asInstanceOf[Long] == qa.getAttrTypeId)
    assert(qaData(4) == qa.getValidOnDate)
    assert(qaData(5).get.asInstanceOf[Long] == qa.getObservationDate)
    assert(qaData(6).get.asInstanceOf[Long] == qa.getSortingIndex)
    assert(qaData.length == 7)
  }

  "date stuff" should "work" in {
    val testEntityId1: Long = mPG.createEntity("test entity for multiple tests1")
    val testEntity1: Entity = new Entity(mPG, testEntityId1)
    val testEntityId2: Long = mPG.createEntity("test entity for multiple tests2")
    val da = testEntity1.addDateAttribute(testEntityId2, 0)
    assert(mRD.dateAttributeKeyExists(da.getId))

    // (see postgresqldatabase.getDateAttributeData & caller for type info)
    val daData = mRD.getDateAttributeData(da.getId)
    assert(daData(0).get.asInstanceOf[Long] == da.getParentId)
    assert(daData(1).get.asInstanceOf[Long] == da.getDate)
    assert(daData(2).get.asInstanceOf[Long] == da.getAttrTypeId)
    assert(daData(3).get.asInstanceOf[Long] == da.getSortingIndex)
    assert(daData.length == 4)
  }

  "boolean stuff" should "work" in {
    val testEntityId1: Long = mPG.createEntity("test entity for multiple tests1")
    val testEntity1: Entity = new Entity(mPG, testEntityId1)
    val testEntityId2: Long = mPG.createEntity("test entity for multiple tests2")
    val ba = testEntity1.addBooleanAttribute(testEntityId2, inBoolean = false, None)
    assert(mRD.booleanAttributeKeyExists(ba.getId))

    // see postgresqldatabase.getBooleanAttributeData & caller for type info)
    val baData = mRD.getBooleanAttributeData(ba.getId)
    assert(baData(0).get.asInstanceOf[Long] == ba.getParentId)
    assert(baData(1).get.asInstanceOf[Boolean] == ba.getBoolean)
    assert(baData(2).get.asInstanceOf[Long] == ba.getAttrTypeId)
    assert(baData(3) == ba.getValidOnDate)
    assert(baData(4).get.asInstanceOf[Long] == ba.getObservationDate)
    assert(baData(5).get.asInstanceOf[Long] == ba.getSortingIndex)
    assert(baData.length == 6)
  }

  "text stuff" should "work" in {
    val testEntityId1: Long = mPG.createEntity("test entity for multiple tests1")
    val testEntity1: Entity = new Entity(mPG, testEntityId1)
    val testEntityId2: Long = mPG.createEntity("test entity for multiple tests2")
    val attrText = "asdf"
    val ta = testEntity1.addTextAttribute(testEntityId2, attrText, None)
    assert(mRD.textAttributeKeyExists(ta.getId))

    val taData = mRD.getTextAttributeData(ta.getId)
    assert(taData(0).get.asInstanceOf[Long] == ta.getParentId)
    assert(taData(1).get.asInstanceOf[String] == ta.getText)
    assert(taData(2).get.asInstanceOf[Long] == ta.getAttrTypeId)
    assert(taData(3) == ta.getValidOnDate)
    assert(taData(4).get.asInstanceOf[Long] == ta.getObservationDate)
    assert(taData(5).get.asInstanceOf[Long] == ta.getSortingIndex)
    assert(taData.length == 6)

    val textAttrsByTypeId: java.util.ArrayList[TextAttribute] = mRD.getTextAttributeByTypeId(testEntityId1, testEntityId2)
    assert(textAttrsByTypeId.size == 1)
    assert(textAttrsByTypeId.get(0).getText == attrText)
  }

  "relation stuff etc" should "work" in {
    val testEntityId1: Long = mPG.createEntity("test entity for multiple tests1")
    val testEntity1: Entity = new Entity(mPG, testEntityId1)
    mPG.createEntity("test entity for multiple tests2")
    val relTypeId: Long = mPG.createRelationType("contains", "", RelationType.UNIDIRECTIONAL)
    val rte = testEntity1.addRelationToLocalEntity(relTypeId, testEntityId1, None)
    assert(mRD.relationToLocalEntityKeyExists(rte.getId))
    assert(mRD.relationToLocalEntityKeysExistAndMatch(rte.getId, rte.getAttrTypeId, testEntityId1, testEntityId1))
    val rteData = mRD.getRelationToLocalEntityData(relTypeId, testEntityId1, testEntityId1)
    assert(rteData(0).get.asInstanceOf[Long] == rte.getId)
    assert(rteData(1) == rte.getValidOnDate)
    assert(rteData(2).get.asInstanceOf[Long] == rte.getObservationDate)
    assert(rteData(3).get.asInstanceOf[Long] == rte.getSortingIndex)
    assert(rteData.length == 4)

    val entitiesContainingEntity: util.ArrayList[(Long, Entity)] = mRD.getLocalEntitiesContainingLocalEntity(testEntityId1, 0)
    assert(entitiesContainingEntity.size == 1)
    assert(entitiesContainingEntity.get(0)._1 == relTypeId)
    assert(entitiesContainingEntity.get(0)._2.getId == testEntityId1)

    val uuid = java.util.UUID.randomUUID().toString
    val omInstance: OmInstance = OmInstance.create(mPG, uuid, "test: relation stuff-" + uuid)
    val rtre = testEntity1.addRelationToRemoteEntity(relTypeId, 0, None, remoteInstanceIdIn = omInstance.getId)
    assert(mRD.relationToRemoteEntityKeyExists(rtre.getId))
    assert(mPG.relationToRemoteEntityKeyExists(rtre.getId))
    assert(mPG.attributeKeyExists(rtre.getFormId, rtre.getId))
    assert(mPG.relationToRemoteEntityExists(rtre.getAttrTypeId, rtre.getRelatedId1, rtre.getRemoteInstanceId, rtre.getRelatedId2))
    assert(mRD.relationToRemoteEntityKeysExistAndMatch(rtre.getId, rtre.getAttrTypeId, rtre.getRelatedId1, omInstance.getId, rtre.getRelatedId2))
    val rtreData = mRD.getRelationToRemoteEntityData(relTypeId, testEntityId1, omInstance.getId, 0)
    assert(rtreData(0).get.asInstanceOf[Long] == rtre.getId)
    assert(rtreData(1) == rtre.getValidOnDate)
    assert(rtreData(2).get.asInstanceOf[Long] == rtre.getObservationDate)
    assert(rtreData(3).get.asInstanceOf[Long] == rtre.getSortingIndex)
    assert(rtreData.length == 4)
    rtre.update(validOnDateIn = Some(9999), observationDateIn = Some(9998))
    val rtreData2 = mRD.getRelationToRemoteEntityData(relTypeId, testEntityId1, omInstance.getId, rtre.getRelatedId2)
    assert(rtreData2(1).get == 9999)
    assert(rtreData2(2).get == 9998)

    // as a little elsewhere, this tests the local db rather than the remote, but better than not doing that anywhere:
    val rtreDesc = rtre.getRemoteDescription
    assert(rtreDesc.indexOf("at") > -1)
    /*val idOfLocalReferenceToRemote = */ rtre.getId
    rtre.delete()
    assert(intercept[Exception] {
                                  new RelationToRemoteEntity(mPG, rtreData(0).get.asInstanceOf[Long], relTypeId, testEntityId1, omInstance.getId, 0)
                                }.getMessage.contains("does not exist"))


    val (groupId, rtgId) = mPG.createGroupAndRelationToGroup(testEntityId1, relTypeId, "test relation to group stuff", allowMixedClassesInGroupIn = true,
                                                             Some(System.currentTimeMillis()), 12345L, None)
    val rtg = new RelationToGroup(mPG, rtgId, testEntityId1, relTypeId, groupId)
    assert(mRD.relationToGroupKeyExists(rtgId))
    assert(!mRD.relationToGroupKeyExists(123456789))
    val rtgData: Array[Option[Any]] = mRD.getRelationToGroupData(rtgId)
    assert(rtgData(0).get.asInstanceOf[Long] == rtg.getId)
    assert(rtgData(1).get.asInstanceOf[Long] == rtg.getParentId)
    assert(rtgData(2).get.asInstanceOf[Long] == rtg.getAttrTypeId)
    assert(rtgData(3).get.asInstanceOf[Long] == rtg.getGroupId)
    assert(rtgData(4) == rtg.getValidOnDate)
    assert(rtgData(5).get.asInstanceOf[Long] == rtg.getObservationDate)
    assert(rtgData(6).get.asInstanceOf[Long] == rtg.getSortingIndex)
    assert(rtgData.length == 7)

    val rtgDataByKeys = mRD.getRelationToGroupDataByKeys(testEntityId1, relTypeId, groupId)
    assert(rtgDataByKeys(0).get.asInstanceOf[Long] == rtg.getId)
    assert(rtgDataByKeys(1).get.asInstanceOf[Long] == rtg.getParentId)
    assert(rtgDataByKeys(2).get.asInstanceOf[Long] == rtg.getAttrTypeId)
    assert(rtgDataByKeys(3).get.asInstanceOf[Long] == rtg.getGroupId)
    assert(rtgDataByKeys(4) == rtg.getValidOnDate)
    assert(rtgDataByKeys(5).get.asInstanceOf[Long] == rtg.getObservationDate)
    assert(rtgDataByKeys(6).get.asInstanceOf[Long] == rtg.getSortingIndex)
    assert(rtgDataByKeys.length == 7)

    // This was tested successfully with mocks when it was in RelationToEntityTest, but then I got rid of the mocks because of a bug.
    // So to continue auto-testing RelationToRemoteEntity.getDisplayString (but here now), it could be good to get the ability
    // to *create* a remote entity (without mocks), then put this back and get it to work right:
//    entity1.addRelationToRemoteEntity(relationTypeId, 4321L)
//    val rtre = new RelationToRemoteEntity(mDB, rteId, relTypeId, entity1Id, remoteInstanceId, entity2Id, None, date, 0)
//    val displayString: String = rtre.getDisplayString(0, Some(mockEntity2), Some(mockRelationType), simplify = false)
//    val expectedObservedDateOutput2 = "Wed 1969-12-31 17:00:00:"+date+" MST"
//    val wholeExpectedThing2: String = relationTypeName + " (at " + remoteAddress + "): \033[36m" + entity2Name +
//                                      "\033[0m; valid unsp'd, obsv'd "+expectedDateOutput
//    assert(displayString.contains(" (at "), "unexpected contents: " + displayString)
//    assert(displayString == wholeExpectedThing2, "unexpected contents: " + displayString)
  }

  "getGroupData etc" should "work" in {
    val groupName = "test getGroupData stuff"
    val groupId: Long = mPG.createGroup(groupName, allowMixedClassesInGroupIn = true)
    val group = new Group(mPG, groupId)
    val groupData = mRD.getGroupData(groupId)
    assert(groupData(0).get.asInstanceOf[String] == group.getName)
    assert(groupData(1).get.asInstanceOf[Long] == group.getInsertionDate)
    assert(groupData(2).get.asInstanceOf[Boolean] == group.getMixedClassesAllowed)
    assert(groupData(3).get.asInstanceOf[Boolean] == group.getNewEntriesStickToTop)
    assert(groupData.length == 4)

    val entity0Name = "test: getGroupDataEtc-0"
    val entityId0 = mPG.createEntity(entity0Name)
    val entityId1 = mPG.createEntity("test: getGroupDataEtc-1")
    val entity0 = new Entity(mPG, entityId0)
    val relationTypeId = mPG.findRelationType(Database.theHASrelationTypeName, Some(1)).get(0)
    val rtg: RelationToGroup = entity0.addRelationToGroup(relationTypeId, groupId, None)
    val groupId2: Long = mPG.createGroup("test getGroupData stuff 2", allowMixedClassesInGroupIn = true)
    val group2 = new Group(mPG, groupId2)
    group2.addEntity(entityId0)
    group2.addEntity(entityId1)
    val groups: List[Array[Option[Any]]] = mRD.getGroupsContainingEntitysGroupsIds(groupId)
    assert(groups.nonEmpty)
    val groups2: List[Array[Option[Any]]] = mRD.getGroupsContainingEntitysGroupsIds(groupId, Some(1))
    assert(groups2.size == 1)
    val entriesData: List[Array[Option[Any]]] = mRD.getGroupEntriesData(groupId2)
    assert(entriesData(0)(0).get.asInstanceOf[Long] == entityId0)
    assert(entriesData(1)(0).get.asInstanceOf[Long] == entityId1)
    assert(entriesData.size == 2)

    val (relationToGroupId3, relationTypeId3, groupId3, _, moreRowsAvailable3) = mRD.findRelationToAndGroup_OnEntity(entityId0, Some(groupName))
    assert(relationToGroupId3.get == rtg.getId)
    assert(relationTypeId3.get == relationTypeId)
    assert(groupId3.get == groupId)
    assert(!moreRowsAvailable3)

    val (relationToGroupId4, _, _, _, moreRowsAvailable4) = mRD.findRelationToAndGroup_OnEntity(entityId0, Some(Math.random().toString))
    assert(relationToGroupId4.isEmpty)
    assert(!moreRowsAvailable4)

    group.addEntity(entityId1)
    val descriptions: util.ArrayList[String] = mRD.getContainingRelationToGroupDescriptions(entityId1)
    assert(descriptions.size == 1)
    assert(descriptions.get(0).contains(entity0Name))
    assert(descriptions.get(0).contains(groupName))

    val rtgs: util.ArrayList[RelationToGroup] = mRD.getRelationsToGroupContainingThisGroup(groupId, 0, None)
    assert(rtgs.size == 1)
    assert(rtgs.get(0).getId == rtg.getId)
  }

  "entity stuff" should "work" in {
    val startTime = System.currentTimeMillis()
    val part = "test entity for multiple tests"
    val entityName1 = part + "1"
    val testEntityId1: Long = mPG.createEntity(entityName1)
    val remoteEntity = new Entity(mRD, testEntityId1)
    assert(intercept[NotImplementedError] {
                                  remoteEntity.updatePublicStatus(Some(false))
                                }.getMessage.contains("implementation is missing"))
    val testEntity1: Entity = new Entity(mPG, testEntityId1)
    val entityName2 = part + "2"
    val testEntityId2: Long = mPG.createEntity(entityName2)
    val endTime = System.currentTimeMillis()
    val journalEntries = mRD.findJournalEntries(startTime, endTime)
    assert(journalEntries.size >= 2)
    val qa: QuantityAttribute = testEntity1.addQuantityAttribute(testEntityId2, testEntityId2, 0, None)
    testEntity1.addDateAttribute(testEntityId2, 0)
    testEntity1.addBooleanAttribute(testEntityId2, inBoolean = false, None)
    val ta = testEntity1.addTextAttribute(testEntityId2, "asdf", None)

    assert(intercept[Exception] {
                                  mRD.getEntityJson_WithOptionalErrHandling(None, testEntityId1)
                                }.getMessage.contains("is not public"))
    mPG.updateEntityOnlyPublicStatus(testEntityId1, Some(true))
    val entityOverview = mRD.getEntityJson_WithOptionalErrHandling(None, testEntityId1)
    assert(entityOverview.get.contains("insertionDate"))
    assert(entityOverview.get.contains("boolean"))
    assert(entityOverview.get.contains("unitId"))
    assert(entityOverview.get.contains("text"))

    val entityData = mRD.getEntityData(testEntityId1)
    assert(entityData(0).get.asInstanceOf[String] == testEntity1.getName)
    assert(entityData(1) == testEntity1.getClassId)
    assert(entityData(2).get.asInstanceOf[Long] == testEntity1.getInsertionDate)
    assert(entityData(3) == testEntity1.getPublic)
    assert(entityData(4).get.asInstanceOf[Boolean] == testEntity1.isArchived)
    assert(entityData(5).get.asInstanceOf[Boolean] == testEntity1.getNewEntriesStickToTop)
    assert(entityData.length == 6)

    val adjacentAttributesSortingIndexes: List[Array[Option[Any]]] = mRD.getAdjacentAttributesSortingIndexes(testEntityId1, qa.getSortingIndex,
                                                                                                             None, forwardNotBackIn = true)
    assert(adjacentAttributesSortingIndexes.size == 3)
    val adjacentAttributesSortingIndexes2 = mRD.getAdjacentAttributesSortingIndexes(testEntityId1, ta.getSortingIndex, Some(1), forwardNotBackIn = false)
    assert(adjacentAttributesSortingIndexes2.size == 1)

    val entities = mRD.getEntities(0, None)
    // 2 entities were created in this test, and at least the system entity always created in a new db:
    assert(entities.size >= 3)
    val entities2 = mRD.getEntities(0, Some(1))
    assert(entities2.size == 1)

    def foundInResults(resultsIn: util.ArrayList[Entity], idIn: Long): Boolean = {
      var found = false
      for (entity: Entity <- resultsIn) {
        if (entity.getId == idIn) {
          found = true
        }
      }
      found
    }
    val entitiesMatching: util.ArrayList[Entity] = mRD.getMatchingEntities(0, None, None, part)
    assert(entitiesMatching.size >= 2)
    assert(foundInResults(entitiesMatching, testEntityId1))
    val entitiesMatching2 = mRD.getMatchingEntities(0, Some(2), None, part)
    assert(entitiesMatching2.size == 2)
    val entitiesMatching3 = mRD.getMatchingEntities(0, None, Some(testEntityId1), part)
    assert(! foundInResults(entitiesMatching3, testEntityId1))

  }

  "getSortedAttributes etc" should "work" in {
    val testEntityId1: Long = mPG.createEntity("test entity for multiple tests1")
    val testEntity1: Entity = new Entity(mPG, testEntityId1)
    val attributeTypeId: Long = mPG.createRelationType("contains", "", RelationType.UNIDIRECTIONAL)
    val qa: QuantityAttribute = testEntity1.addQuantityAttribute(attributeTypeId, attributeTypeId, 0, None)
    val da = testEntity1.addDateAttribute(attributeTypeId, 0)
    val ba = testEntity1.addBooleanAttribute(attributeTypeId, inBoolean = false, None)
    val x: (File, FileAttribute) = createFileAttribute(testEntity1, attributeTypeId)
    val fa = x._2
    val attrText = "asdfjkl;"
    val ta = testEntity1.addTextAttribute(attributeTypeId, attrText, None)
    val rte = testEntity1.addRelationToLocalEntity(attributeTypeId, testEntityId1, None)
    val uuid = java.util.UUID.randomUUID().toString
    val omInstance: OmInstance = OmInstance.create(mPG, uuid, "test: relation stuff-" + uuid)
    val rtre = testEntity1.addRelationToRemoteEntity(attributeTypeId, 0, None, remoteInstanceIdIn = omInstance.getId)

    val (groupId, rtgId) = mPG.createGroupAndRelationToGroup(testEntityId1, attributeTypeId, "test relation to group stuff",
                                                             allowMixedClassesInGroupIn = true, Some(System.currentTimeMillis()), 12345L, None)
    val rtg = new RelationToGroup(mPG, rtgId, testEntityId1, attributeTypeId, groupId)

    val attributes: (Array[(Long, Attribute)], Int) = mRD.getSortedAttributes(testEntityId1, 0, 0, onlyPublicEntitiesIn = false)
    assert(attributes._1.length == 8)
    assert(attributes._2 == 8)
    mPG.updateEntityOnlyPublicStatus(testEntityId1, Some(false))

    for (tuple <- attributes._1) {
      val attribute = tuple._2
      assert(testEntityId1 == attribute.getParentId)
      assert(attributeTypeId == attribute.getAttrTypeId)
      attribute match {
        case a: QuantityAttribute =>
          assert(qa.getId == a.getId)
          assert(qa.getFormId == a.getFormId)
          assert(qa.getSortingIndex == a.getSortingIndex)
          assert(qa.getValidOnDate == a.getValidOnDate)
          assert(qa.getObservationDate == a.getObservationDate)
          assert(qa.getUnitId == a.getUnitId)
          assert(qa.getNumber == a.getNumber)
        case a: DateAttribute =>
          assert(da.getId == a.getId)
          assert(da.getFormId == a.getFormId)
          assert(da.getSortingIndex == a.getSortingIndex)
          assert(da.getDate == a.getDate)
        case a: BooleanAttribute =>
          assert(ba.getId == a.getId)
          assert(ba.getFormId == a.getFormId)
          assert(ba.getSortingIndex == a.getSortingIndex)
          assert(ba.getValidOnDate == a.getValidOnDate)
          assert(ba.getObservationDate == a.getObservationDate)
          assert(ba.getBoolean == a.getBoolean)
        case a: FileAttribute =>
          assert(fa.getId == a.getId)
          assert(fa.getFormId == a.getFormId)
          assert(fa.getSortingIndex == a.getSortingIndex)
          assert(fa.getDescription == a.getDescription)
          assert(fa.getOriginalFileDate == a.getOriginalFileDate)
          assert(fa.getStoredDate == a.getStoredDate)
          assert(fa.getOriginalFilePath == a.getOriginalFilePath)
          assert(fa.getReadable == a.getReadable)
          assert(fa.getWritable == a.getWritable)
          assert(fa.getExecutable == a.getExecutable)
          assert(fa.getSize == a.getSize)
          assert(fa.getMd5Hash == a.getMd5Hash)
        case a: TextAttribute =>
          assert(ta.getId == a.getId)
          assert(ta.getFormId == a.getFormId)
          assert(ta.getSortingIndex == a.getSortingIndex)
          assert(ta.getValidOnDate == a.getValidOnDate)
          assert(ta.getObservationDate == a.getObservationDate)
          assert(ta.getValidOnDate == a.getValidOnDate)
          assert(ta.getObservationDate == a.getObservationDate)
          assert(ta.getText == a.getText)
        case a: RelationToRemoteEntity =>
          assert(rtre.getId == a.getId)
          assert(rtre.getFormId == a.getFormId)
          assert(rtre.getSortingIndex == a.getSortingIndex)
          assert(rtre.getValidOnDate == a.getValidOnDate)
          assert(rtre.getObservationDate == a.getObservationDate)
          assert(rtre.getRelatedId1 == a.getRelatedId1)
          assert(rtre.getRelatedId2 == a.getRelatedId2)
          assert(rtre.asInstanceOf[RelationToRemoteEntity].getRemoteInstanceId == a.asInstanceOf[RelationToRemoteEntity].getRemoteInstanceId)
        case a: RelationToLocalEntity =>
          assert(rte.getId == a.getId)
          assert(rte.getFormId == a.getFormId)
          assert(rte.getSortingIndex == a.getSortingIndex)
          assert(rte.getValidOnDate == a.getValidOnDate)
          assert(rte.getObservationDate == a.getObservationDate)
          assert(rte.getRelatedId1 == a.getRelatedId1)
          assert(rte.getRelatedId2 == a.getRelatedId2)
        case a: RelationToGroup =>
          assert(rtg.getId == a.getId)
          assert(rtg.getFormId == a.getFormId)
          assert(rtg.getSortingIndex == a.getSortingIndex)
          assert(rtg.getValidOnDate == a.getValidOnDate)
          assert(rtg.getObservationDate == a.getObservationDate)
          assert(rtg.getParentId == a.getParentId)
          assert(rtg.getGroupId == a.getGroupId)
        case _ => throw new org.onemodel.core.OmException("Unexpected type: " + attribute.getClass.getCanonicalName)
      }
    }

    val attributes2: (Array[(Long, Attribute)], Int) = mRD.getSortedAttributes(testEntityId1, 0, 0, onlyPublicEntitiesIn = true)
    assert(attributes2._1.length == 7)
  }

// (Keep next line to match the comment-able one at the top, right after the package statement.)
// */
}
