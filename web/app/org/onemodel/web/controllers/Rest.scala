/*
    This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2016-2016 inclusive, Luke A. Call; all rights reserved.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule, guidelines around binary
    distribution, and the GNU Affero General Public License as published by the Free Software Foundation, either version 3
    of the License, or (at your option) any later version.  See the file LICENSE for details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
package org.onemodel.web.controllers

import java.nio.file.Path
import java.util
import java.util.ArrayList

import org.apache.commons.io.FilenameUtils
import org.onemodel.core._
import org.onemodel.core.database.{Database, PostgreSQLDatabase}
import org.onemodel.core.model._
import play.api.libs.json._
import play.api.mvc._

import scala.collection.JavaConversions._
import scala.collection.mutable

/*: IDEAS: stuff to do then delete these comments, from: http://www.vinaysahni.com/best-practices-for-a-pragmatic-restful-api :

"An API that accepts JSON encoded POST, PUT & PATCH requests should also require the Content-Type header be set to application/json or throw a 415
Unsupported Media Type HTTP status code."

"To prevent abuse, it is standard practice to add some sort of rate limiting to an API. RFC 6585 introduced a HTTP status code 429 Too Many Requests to
accommodate this."

"HTTP defines a bunch of meaningful status codes that can be returned from your API. These can be leveraged to help the API consumers route their responses
accordingly. I've curated a short list of the ones that you definitely should be using:
200 OK - Response to a successful GET, PUT, PATCH or DELETE. Can also be used for a POST that doesn't result in a creation.
201 Created - Response to a POST that results in a creation. Should be combined with a Location header pointing to the location of the new resource
204 No Content - Response to a successful request that won't be returning a body (like a DELETE request)
304 Not Modified - Used when HTTP caching headers are in play
400 Bad Request - The request is malformed, such as if the body does not parse
401 Unauthorized - When no or invalid authentication details are provided. Also useful to trigger an auth popup if the API is used from a browser
403 Forbidden - When authentication succeeded but authenticated user doesn't have access to the resource
404 Not Found - When a non-existent resource is requested
405 Method Not Allowed - When an HTTP method is being requested that isn't allowed for the authenticated user
410 Gone - Indicates that the resource at this end point is no longer available. Useful as a blanket response for old API versions
415 Unsupported Media Type - If incorrect content type was provided as part of the request
422 Un-processable Entity - Used for validation errors
429 Too Many Requests - When a request is rejected due to rate limiting"

*/

class Rest extends play.api.mvc.Controller {
  val (user, pass) = Util.getDefaultUserInfo

  // USE ONLY 1 OF THE NEXT 2 "val db ..." LINES AT A TIME:
  // (Idea, also tracked in tasks: how best to properly automate this, so it works for testing in the test db or manual use as needed?)
  // This one is when testing manually from a client that wants to connect to the main DB via rest:
//  val db = new PostgreSQLDatabase(user, pass)
//   This one is for when running the tests in the core module's RestDatabaseTest:
  val db = new PostgreSQLDatabase(Database.TEST_USER, Database.TEST_USER)

  def id: Action[AnyContent] = Action { implicit request =>
    // This puts quotes around it...
    val localId: String = db.getId
    val msg = new JsString(localId)
    // ...but this one does not. Does it matter? could switch it, then in OM as the rest client try MM 8 then the remote instance check when editing &
    // adding a port # or similar change so it tries the connection to test the difference.
    //    val inst: OmInstance = db.getLocalOmInstanceData
    //    val msg: String = inst.getId
    Ok(msg).as(JSON)
  }

  /** About this construct and its use:
    * https://www.playframework.com/documentation/2.5.x/ScalaJson#Using-Writes-converters
    * ...and it is then used in places like "val json: JsValue = Json.toJson(entity)".
    */
  implicit val entityWrites = new Writes[Entity] {
    def writes(entityIn: Entity) = {
      /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
      val attributeTuples: Array[(Long, Attribute)] = entityIn.getAttributes(onlyPublicEntitiesIn = true)._1
      val attributes: Array[Attribute] = new Array[Attribute](attributeTuples.length)
      var index = 0
      for (attrTuple <- attributeTuples) {
        val sortingIndex: Long = attrTuple._1
        attributes(index) = attrTuple._2
        val attribute = attributes(index)
        require(attribute.getParentId == entityIn.getId,
                "Unexpected: attribute that is supposed to be on entity " + entityIn.getId + " has parentId of " + attribute.getParentId + "?")
        require(attribute.getSortingIndex == sortingIndex,
                "Unexpected: attribute that is supposed to be on entity " + entityIn.getId + " has parentId of " + attribute.getParentId + ", and sorting" +
                "indices don't match: " + sortingIndex + " (from getSortingIndex) and " + attribute.getSortingIndex + "(from attribute object)?")
        index += 1
      }

      Json.obj(
                "id" -> entityIn.getId,
                "name" -> entityIn.getName,
                // This one outputs null (i.e., the json has:  ...,"classId":null,... ) when the value is NULL in the db (as could "public" below, though
                // currently the
                // endpoint intentionally returns an error instead, if the entity has anything but TRUE for public in the db).
                "classId" -> entityIn.getClassId,
                // could instead or in addition use ISO8601 for dates, for benefit of other clients besides OM itself (see everywhere w/ this comment):
                "insertionDate" -> entityIn.getInsertionDate,
                "public" -> entityIn.getPublic,
                "archived" -> entityIn.getArchivedStatus,
                "newEntriesStickToTop" -> entityIn.getNewEntriesStickToTop,
                "attributes" -> Json.arr(
                                          attributes.map { attribute =>
                                            var jsonObject: JsObject = Json.obj(
                                                                                 "sortingIndex" -> attribute.getSortingIndex,
                                                                                 "id" -> attribute.getId,
                                                                                 "formId" -> attribute.getFormId,
                                                                                 "formName" -> Database.getAttributeFormName(attribute.getFormId),
                                                                                 "attrTypeId" -> attribute.getAttrTypeId
                                                                               )
                                            attribute match {
                                              case a: QuantityAttribute =>
                                                // Idea: for questions of possible data loss or improving how we transfer numbers into & out of OM instances,
                                                // see
                                                // http://www.scala-lang.org/api/current/index.html#scala.math.BigDecimal$
                                                // ...but consider which documentation applies for the version of scala in use.
                                                jsonObject = jsonObject + ("unitId" -> JsNumber(a.getUnitId))
                                                jsonObject = jsonObject + ("number" -> JsNumber(BigDecimal.double2bigDecimal(a.getNumber)))
                                              case a: DateAttribute =>
                                                // could instead or in addition use ISO8601 for dates, for benefit of other clients besides OM itself (see
                                                // everywhere w/ this comment):
                                                jsonObject = jsonObject + ("date" -> JsNumber(a.getDate))
                                              case a: BooleanAttribute =>
                                                jsonObject = jsonObject + ("boolean" -> JsBoolean(a.getBoolean))
                                              case a: FileAttribute =>
                                                /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE??
                                                (4now at least) */
                                                jsonObject = jsonObject + ("unstructuredForThisQuickView..." -> JsString(a.getDisplayString(0)))
                                              case a: TextAttribute =>
                                                val text = a.getText
                                                val textShorter = text.substring(0, Math.min(text.length, 40))
                                                val textShorterEscaped = org.apache.commons.lang3.StringEscapeUtils.escapeJson(textShorter)
                                                jsonObject = jsonObject + ("text (a substring)" -> JsString(textShorterEscaped))
                                              case a: RelationToRemoteEntity =>
                                                val relType = new RelationType(db, a.getAttrTypeId)
                                                jsonObject = jsonObject + ("relationTypeName" -> JsString(relType.getName))
                                                jsonObject = jsonObject + ("entity2Id" -> JsNumber(a.getRelatedId2))
                                                val entity2 = new Entity(db, a.getRelatedId2)
                                                jsonObject = jsonObject + ("entity2Name" -> JsString(entity2.getName))
                                                jsonObject = jsonObject + ("remoteInstanceId" -> JsString(a.asInstanceOf[RelationToRemoteEntity].getRemoteInstanceId))
                                                jsonObject = jsonObject + ("remoteInstanceDescription" ->
                                                                           JsString(a.asInstanceOf[RelationToRemoteEntity].getRemoteDescription))
                                              case a: RelationToGroup =>
                                                val relType = new RelationType(db, a.getAttrTypeId)
                                                jsonObject = jsonObject + ("relationTypeName" -> JsString(relType.getName))
                                                jsonObject = jsonObject + ("groupId" -> JsNumber(a.getGroupId))
                                                val group = new Group(db, a.getGroupId)
                                                jsonObject = jsonObject + ("groupName" -> JsString(group.getName))
                                              case a: RelationToEntity =>
                                                val relType = new RelationType(db, a.getAttrTypeId)
                                                jsonObject = jsonObject + ("relationTypeName" -> JsString(relType.getName))
                                                jsonObject = jsonObject + ("entity2Id" -> JsNumber(a.getRelatedId2))
                                                val entity2 = new Entity(db, a.getRelatedId2)
                                                jsonObject = jsonObject + ("entity2Name" -> JsString(entity2.getName))
                                              case _ => throw new OmException("Unexpected type: " + attribute.getClass.getCanonicalName)
                                            }
                                            jsonObject
                                                         }
                                        )
              )
    }
  }

  def getEntityOverview(idIn: Long): Action[AnyContent] = Action { implicit request =>
    val exists: Boolean = db.entityOnlyKeyExists(idIn)
    if (!exists) {
      val msg: String = "Entity " + idIn + " was not found."
      NotFound(msg)
    } else {
      val entity = new Entity(db, idIn)
      val public: Option[Boolean] = entity.getPublic
      /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
      if (public.isDefined && public.get) {
        // (About this json conversion: see comment on "implicit val entityWrites".)
        val json: JsValue = Json.toJson(entity)
        Ok(Json.prettyPrint(json)).as(JSON)
        //another way, for future convenient reference:
        //        Result(
        //                header = ResponseHeader(200, Map.empty),
        //                body = HttpEntity.Strict(ByteString(msg), Some("text/plain"))
        //              )
      } else {
        /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
        val msg: String = "Entity " + idIn + " is not public."
        Forbidden(msg)
      }
    }
  }

  def defaultEntity = Action { implicit request =>
    val defaultEntityId: Option[Long] = db.getUserPreference_EntityId(Util.DEFAULT_ENTITY_PREFERENCE)
    if (defaultEntityId.isDefined) {
      /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
      val entity = new Entity(db, defaultEntityId.get)
      val json = Json.toJson(entity)
      Ok(Json.prettyPrint(json)).as(JSON)
    } else {
      val msg: String = "A default entity preference was not found."
      NotFound(msg)
    }
  }

  def getAttributeCount(entityIdIn: Long, includeArchivedEntitiesIn: Boolean): Action[AnyContent] = Action { implicit request =>
    val count: Long = db.getAttributeCount(entityIdIn, includeArchivedEntitiesIn = includeArchivedEntitiesIn)
    val msg = new JsNumber(count)
    Ok(msg).as(JSON)
  }

  /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */

  def entityKeyExists(entityIdIn: Long, includeArchivedEntitiesIn: Boolean): Action[AnyContent] = Action { implicit request =>
    val exists: Boolean = db.entityKeyExists(entityIdIn, includeArchived = includeArchivedEntitiesIn)
    val msg = new JsBoolean(exists)
    Ok(msg).as(JSON)
  }

  /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */

  def getGroupSize(groupIdIn: Long, includeWhichEntitiesIn: Int): Action[AnyContent] = Action { implicit request =>
    val size: Long = db.getGroupSize(groupIdIn, includeWhichEntitiesIn)
    val msg = new JsNumber(size)
    Ok(msg).as(JSON)
  }

  /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */

  def findUnusedGroupSortingIndex(groupIdIn: Long): Action[AnyContent] = Action { implicit request =>
    val index: Long = db.findUnusedGroupSortingIndex(groupIdIn, None)
    val msg = new JsNumber(index)
    Ok(msg).as(JSON)
  }

  /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
  def findUnusedGroupSortingIndex2(groupIdIn: Long, startingWithIn: Long): Action[AnyContent] = Action { implicit request =>
    val index: Long = db.findUnusedGroupSortingIndex(groupIdIn, Some(startingWithIn))
    val msg = new JsNumber(index)
    Ok(msg).as(JSON)
  }

  /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */

  def getHighestSortingIndexForGroup(groupIdIn: Long): Action[AnyContent] = Action { implicit request =>
    Ok(new JsNumber(db.getHighestSortingIndexForGroup(groupIdIn))).as(JSON)
  }

  /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */

  def getGroupSortingIndex(groupIdIn: Long, entityIdIn: Long): Action[AnyContent] = Action { implicit request =>
    Ok(new JsNumber(db.getGroupSortingIndex(groupIdIn, entityIdIn))).as(JSON)
  }

  /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */

  def getEntityAttributeSortingIndex(id: Long, attributeFormIdIn: Long, attributeIdIn: Long): Action[AnyContent] = Action { implicit request =>
    Ok(new JsNumber(db.getEntityAttributeSortingIndex(id, attributeFormIdIn, attributeIdIn))).as(JSON)
  }

  def getEntitiesOnlyCount1(limitByClass: Boolean): Action[AnyContent] = Action { implicit request =>
    Ok(new JsNumber(db.getEntitiesOnlyCount(limitByClass, None, None))).as(JSON)
  }

  /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
  def getEntitiesOnlyCount2(limitByClass: Boolean, classIdIn: Long): Action[AnyContent] = Action { implicit request =>
    Ok(new JsNumber(db.getEntitiesOnlyCount(limitByClass, Some(classIdIn), None))).as(JSON)
  }

  def getEntitiesOnlyCount3(limitByClass: Boolean, classIdIn: Long, templateEntityIn: Long): Action[AnyContent] = Action { implicit request =>
    Ok(new JsNumber(db.getEntitiesOnlyCount(limitByClass, Some(classIdIn), Some(templateEntityIn)))).as(JSON)
  }

  /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
  def getCountOfGroupsContainingEntity(entityIdIn: Long): Action[AnyContent] = Action { implicit request =>
    Ok(new JsNumber(db.getCountOfGroupsContainingEntity(entityIdIn))).as(JSON)
  }

  def getRelationToEntityCount(entityIdIn: Long, includeArchivedEntitiesIn: Boolean): Action[AnyContent] = Action { implicit request =>
    Ok(new JsNumber(db.getRelationToEntityCount(entityIdIn, includeArchivedEntitiesIn))).as(JSON)
  }

  def getRelationToGroupCount(entityIdIn: Long): Action[AnyContent] = Action { implicit request =>
    Ok(new JsNumber(db.getRelationToGroupCount(entityIdIn))).as(JSON)
  }

  def getClassCount: Action[AnyContent] = Action { implicit request =>
    Ok(new JsNumber(db.getClassCount(None))).as(JSON)
  }

  /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
  def getClassCount2(templateEntityIdIn: Long): Action[AnyContent] = Action { implicit request =>
    Ok(new JsNumber(db.getClassCount(Some(templateEntityIdIn)))).as(JSON)
  }

  def findUnusedAttributeSortingIndex(entityIdIn: Long): Action[AnyContent] = Action { implicit request =>
    Ok(new JsNumber(db.getClassCount(None))).as(JSON)
  }

  def findUnusedAttributeSortingIndex2(entityIdIn: Long, startingWithIn: Long): Action[AnyContent] = Action { implicit request =>
    Ok(new JsNumber(db.getClassCount(Some(startingWithIn)))).as(JSON)
  }

  /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
  def isAttributeSortingIndexInUse(entityIdIn: Long, sortingIndexIn: Long): Action[AnyContent] = Action { implicit request =>
    val msg = new JsBoolean(db.isAttributeSortingIndexInUse(entityIdIn, sortingIndexIn))
    Ok(msg).as(JSON)
  }

  /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
  def isGroupEntrySortingIndexInUse(groupIdIn: Long, sortingIndexIn: Long): Action[AnyContent] = Action { implicit request =>
    val msg = new JsBoolean(db.isGroupEntrySortingIndexInUse(groupIdIn, sortingIndexIn))
    Ok(msg).as(JSON)
  }

  def relationTypeKeyExists(idIn: Long): Action[AnyContent] = Action { implicit request =>
    Ok(new JsBoolean(db.relationTypeKeyExists(idIn))).as(JSON)
  }

  def quantityAttributeKeyExists(idIn: Long): Action[AnyContent] = Action { implicit request =>
    Ok(new JsBoolean(db.quantityAttributeKeyExists(idIn))).as(JSON)
  }

  /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
  def dateAttributeKeyExists(idIn: Long): Action[AnyContent] = Action { implicit request =>
    Ok(new JsBoolean(db.dateAttributeKeyExists(idIn))).as(JSON)
  }

  def booleanAttributeKeyExists(idIn: Long): Action[AnyContent] = Action { implicit request =>
    Ok(new JsBoolean(db.booleanAttributeKeyExists(idIn))).as(JSON)
  }

  def fileAttributeKeyExists(idIn: Long): Action[AnyContent] = Action { implicit request =>
    Ok(new JsBoolean(db.fileAttributeKeyExists(idIn))).as(JSON)
  }

  def textAttributeKeyExists(idIn: Long): Action[AnyContent] = Action { implicit request =>
    Ok(new JsBoolean(db.textAttributeKeyExists(idIn))).as(JSON)
  }

  /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
  def isEntityInGroup(groupIdIn: Long, entityIdIn: Long): Action[AnyContent] = Action { implicit request =>
    Ok(new JsBoolean(db.isEntityInGroup(groupIdIn, entityIdIn))).as(JSON)
  }

  def relationToGroupKeysExistAndMatch(id: Long, entityId: Long, relationTypeId: Long, groupId: Long): Action[AnyContent] = Action { implicit request =>
    Ok(new JsBoolean(db.relationToGroupKeysExistAndMatch(id, entityId, relationTypeId, groupId))).as(JSON)
  }

  def isDuplicateEntityName(name: String): Action[AnyContent] = Action { implicit request =>
    Ok(new JsBoolean(db.isDuplicateEntityName(name))).as(JSON)
  }

  /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
  def isDuplicateEntityName2(name: String, idToIgnore: Long): Action[AnyContent] = Action { implicit request =>
    Ok(new JsBoolean(db.isDuplicateEntityName(name, Some(idToIgnore)))).as(JSON)
  }

  def classKeyExists(id: Long): Action[AnyContent] = Action { implicit request =>
    Ok(new JsBoolean(db.classKeyExists(id))).as(JSON)
  }

  def omInstanceKeyExists(id: String): Action[AnyContent] = Action { implicit request =>
    Ok(new JsBoolean(db.omInstanceKeyExists(id))).as(JSON)
  }

  def groupKeyExists(id: Long): Action[AnyContent] = Action { implicit request =>
    Ok(new JsBoolean(db.groupKeyExists(id))).as(JSON)
  }

  /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
  def includeArchivedEntities: Action[AnyContent] = Action { implicit request =>
    Ok(new JsBoolean(db.includeArchivedEntities)).as(JSON)
  }

  def isDuplicateOmInstanceAddress(address: String): Action[AnyContent] = Action { implicit request =>
    Ok(new JsBoolean(db.isDuplicateOmInstanceAddress(address))).as(JSON)
  }

  def isDuplicateOmInstanceAddress2(address: String, selfIdToIgnore: String): Action[AnyContent] = Action { implicit request =>
    Ok(new JsBoolean(db.isDuplicateOmInstanceAddress(address, Some(selfIdToIgnore)))).as(JSON)
  }

  /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
  def attributeKeyExists(formIdIn: Long, idIn: Long): Action[AnyContent] = Action { implicit request =>
    Ok(new JsBoolean(db.attributeKeyExists(formIdIn, idIn))).as(JSON)
  }

  def relationToEntityKeyExists(idIn: Long): Action[AnyContent] = Action { implicit request =>
    Ok(new JsBoolean(db.relationToEntityKeyExists(idIn))).as(JSON)
  }

  def relationToEntityKeysExistAndMatch(id: Long, relationTypeId: Long, entityId1: Long, entityId2: Long): Action[AnyContent] = Action { implicit request =>
    Ok(new JsBoolean(db.relationToEntityKeysExistAndMatch(id, relationTypeId, entityId1, entityId2))).as(JSON)
  }

  def relationToRemoteEntityKeyExists(id: Long): Action[AnyContent] = Action { implicit request =>
    Ok(new JsBoolean(db.relationToRemoteEntityKeyExists(id: Long))).as(JSON)
  }

  def relationToRemoteEntityKeysExistAndMatch(idIn: Long, relationTypeIdIn: Long, entityId1In: Long, remoteInstanceIdIn: String, entityId2In: Long):
  Action[AnyContent] = Action { implicit request =>
    Ok(new JsBoolean(db.relationToRemoteEntityKeysExistAndMatch(idIn, relationTypeIdIn, entityId1In, remoteInstanceIdIn, entityId2In))).as(JSON)
  }

  /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
  def getNearestGroupEntrysSortingIndex(groupId: Long, startingPointSortingIndex: Long, forwardNotBack: Boolean): Action[AnyContent] = Action { implicit
                                                                                                                                                request =>
    val index: Option[Long] = db.getNearestGroupEntrysSortingIndex(groupId, startingPointSortingIndex, forwardNotBack)
    if (index.isDefined) {
      Ok(new JsNumber(index.get)).as(JSON)
    } else {
      Ok(JsNull).as(JSON)
    }
  }

  /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
  def getAdjacentGroupEntriesSortingIndexes(groupId: Long, startingPointSortingIndex: Long, forwardNotBack: Boolean,
                                            limit: Option[Long]): Action[AnyContent] = Action { implicit request =>
    val results: List[Array[Option[Any]]] = db.getAdjacentGroupEntriesSortingIndexes(groupId, startingPointSortingIndex, limit, forwardNotBack)
    if (results.isEmpty) {
      Ok(JsNull).as(JSON)
    } else {
      var json = Json.arr()
      for (result <- results) {
        json = json.append(Json.obj("sortingIndex" -> result(0).get.asInstanceOf[Long]))
      }
      Ok(Json.prettyPrint(json)).as(JSON)
    }
  }

  /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
  def getGroupsContainingEntitysGroupsIds(groupId: Long, limit: Option[Long]): Action[AnyContent] = Action { implicit request =>
    val results: List[Array[Option[Any]]] = db.getGroupsContainingEntitysGroupsIds(groupId, limit)
    if (results.isEmpty) {
      Ok(JsNull).as(JSON)
    } else {
      var json = Json.arr()
      for (result <- results) {
        // Idea: is there any point in saying "groupId", or should it just be an array of longs? similarly elsewhere. Or maybe this is good
        // in case other values are added later and clarity/distinction is needed between them?
        json = json.append(Json.obj("groupId" -> result(0).get.asInstanceOf[Long]))
      }
      Ok(Json.prettyPrint(json)).as(JSON)
    }
  }

  /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
  def getAdjacentAttributesSortingIndexes(entityId: Long, sortingIndexIn: Long, forwardNotBackIn: Boolean,
                                          limit: Option[Long]): Action[AnyContent] = Action { implicit request =>
    val results: List[Array[Option[Any]]] = db.getAdjacentAttributesSortingIndexes(entityId, sortingIndexIn, limit, forwardNotBackIn)
    if (results.isEmpty) {
      Ok(JsNull).as(JSON)
    } else {
      var json = Json.arr()
      for (result <- results) {
        // Idea: is there any point in saying "groupId", or should it just be an array of longs? similarly elsewhere. Or maybe this is good
        // in case other values are added later and clarity/distinction is needed between them?
        json = json.append(Json.obj("sortingIndex" -> result(0).get.asInstanceOf[Long]))
      }
      Ok(Json.prettyPrint(json)).as(JSON)
    }
  }

  /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
  def getGroupEntriesData(groupId: Long, includeArchivedEntities: Boolean, limit: Option[Long]): Action[AnyContent] = Action { implicit request =>
    val results: List[Array[Option[Any]]] = db.getGroupEntriesData(groupId, limit, includeArchivedEntities)
    if (results.isEmpty) {
      Ok(JsNull).as(JSON)
    } else {
      var json = Json.arr()
      for (result <- results) {
        json = json.append(Json.obj("entityId" -> result(0).get.asInstanceOf[Long],
                                    "sortingIndex" -> result(1).get.asInstanceOf[Long]))
      }
      Ok(Json.prettyPrint(json)).as(JSON)
    }
  }

  /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
  def getContainingRelationToGroups(entityId: Long, startingIndexIn: Long, limit: Option[Long]): Action[AnyContent] = Action { implicit request =>
    val results: ArrayList[RelationToGroup] = db.getContainingRelationToGroups(entityId, startingIndexIn, limit)
    if (results.isEmpty) {
      Ok(JsNull).as(JSON)
    } else {
      var json = Json.arr()
      // (The "import scala.collection.JavaConversions._" is so this iterator works (and others like it).  an alternative might
      // be: "new scala.collection.jcl.ArrayList(results).toList" or just results.toArray.toList or such.)
      for (result: RelationToGroup <- results) {
        json = json.append(Json.obj("id" -> result.getId,
                                    "entityId" -> result.getParentId,
                                    "relationTypeId" -> result.getAttrTypeId,
                                    "groupId" -> result.getGroupId,
                                    "validOnDate" -> result.getValidOnDate,
                                    "observationDate" -> result.getObservationDate,
                                    "sortingIndex" -> result.getSortingIndex))
      }
      Ok(Json.prettyPrint(json)).as(JSON)
    }
  }

  /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
  def getTextAttributeByTypeId(entityId: Long, typeId: Long, expectedRows: Option[Int]): Action[AnyContent] = Action { implicit request =>
    val results: ArrayList[TextAttribute] = db.getTextAttributeByTypeId(entityId, typeId, expectedRows)
    if (results.isEmpty) {
      Ok(JsNull).as(JSON)
    } else {
      var json = Json.arr()
      for (result: TextAttribute <- results) {
        json = json.append(Json.obj("id" -> result.getId,
                                    "entityId" -> result.getParentId,
                                    "relationTypeId" -> result.getAttrTypeId,
                                    "groupId" -> result.getText,
                                    "validOnDate" -> result.getValidOnDate,
                                    "observationDate" -> result.getObservationDate,
                                    "sortingIndex" -> result.getSortingIndex))
      }
      Ok(Json.prettyPrint(json)).as(JSON)
    }
  }

  /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
  def findContainedEntityIds(fromEntityIdIn: Long, searchStringIn: String, levelsRemaining: Int,
                             stopAfterAnyFound: Boolean): Action[AnyContent] = Action { implicit request =>
    val results: mutable.TreeSet[Long] = db.findContainedEntityIds(new mutable.TreeSet[Long](), fromEntityIdIn, searchStringIn,
                                                                   levelsRemaining, stopAfterAnyFound)
    if (results.isEmpty) {
      Ok(JsNull).as(JSON)
    } else {
      var json = Json.arr()
      for (result: Long <- results) {
        json = json.append(Json.obj("id" -> result))
      }
      Ok(Json.prettyPrint(json)).as(JSON)
    }
  }

  /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
  def findAllEntityIdsByName(nameIn: String, caseSensitive: Boolean): Action[AnyContent] = Action { implicit request =>
    val results: util.ArrayList[Long] = db.findAllEntityIdsByName(nameIn, caseSensitive)
    getResultingIdsAsJson(results)
  }

  def getResultingIdsAsJson(results: util.ArrayList[Long]): Result = {
    if (results.isEmpty) {
      Ok(JsNull).as(JSON)
    } else {
      var json = Json.arr()
      for (result: Long <- results) {
        json = json.append(Json.obj("id" -> result))
      }
      Ok(Json.prettyPrint(json)).as(JSON)
    }
  }

  /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
  def getNearestAttributeEntrysSortingIndex(entityId: Long, startingPointSortingIndex: Long, forwardNotBack: Boolean): Action[AnyContent] = Action { implicit
                                                                                                                                                     request =>
    val index: Option[Long] = db.getNearestAttributeEntrysSortingIndex(entityId, startingPointSortingIndex, forwardNotBack)
    if (index.isDefined) {
      Ok(new JsNumber(index.get)).as(JSON)
    } else {
      Ok(JsNull).as(JSON)
    }
  }

  def getClassName(classId: Long): Action[AnyContent] = Action { implicit request =>
    val name: Option[String] = db.getClassName(classId: Long)
    if (name.isDefined) {
      Ok(new JsString(name.get)).as(JSON)
    } else {
      Ok(JsNull).as(JSON)
    }
  }

  /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
  def getEntityName(entityId: Long): Action[AnyContent] = Action { implicit request =>
    val name: Option[String] = db.getEntityName(entityId: Long)
    if (name.isDefined) {
      Ok(new JsString(name.get)).as(JSON)
    } else {
      Ok(JsNull).as(JSON)
    }
  }

  def getShouldCreateDefaultAttributes(classIdIn: Long): Action[AnyContent] = Action { implicit request =>
    val shouldCreate: Option[Boolean] = db.getShouldCreateDefaultAttributes(classIdIn: Long)
    if (shouldCreate.isDefined) {
      Ok(new JsBoolean(shouldCreate.get)).as(JSON)
    } else {
      Ok(JsNull).as(JSON)
    }
  }

  // (About this json conversion: see comment on "implicit val entityWrites".)
  implicit val classWrites = new Writes[EntityClass] {
    def writes(classIn: EntityClass) = {
      /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
      Json.obj(
                // Omitting next line because nothing currently uses the data, and changing it requires changing the consumers
                // of org.onemodel.core.database.Database.getClassData_resultTypes and their callers.
                //"id" -> classIn.getId,
                "name" -> classIn.getName,
                "templateEntityId" -> classIn.getTemplateEntityId,
                //idea: add these when/if needed:
                //                "public" -> classIn.getPublic,
                //                "createDefaultAttributes" -> (if (classIn.getCreateDefaultAttributes.isEmpty) JsNull else classIn
                // .getCreateDefaultAttributes.get)
                "createDefaultAttributes" -> classIn.getCreateDefaultAttributes
              )
    }
  }

  /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
  def getClassData(classIdIn: Long): Action[AnyContent] = Action { implicit request =>
    val exists: Boolean = db.classKeyExists(classIdIn)
    if (!exists) {
      Ok(JsNull).as(JSON)
    } else {
      val clazz = new EntityClass(db, classIdIn)
      /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
      //      val public: Option[Boolean] = entity.getPublic
      //      if (public.isDefined && public.get) {
      //        /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
      //        val msg: String = "Entity " + classIdIn + " is not public."
      //        Forbidden(msg)
      //      }

      // (About this json conversion: see comment on "implicit val entityWrites".)
      val json: JsValue = Json.toJson(clazz)
      Ok(Json.prettyPrint(json)).as(JSON)
    }
  }

  /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
  def getRelationTypeData(idIn: Long): Action[AnyContent] = Action { implicit request =>
    val exists: Boolean = db.relationTypeKeyExists(idIn)
    if (!exists) {
      Ok(JsNull).as(JSON)
    } else {
      val relationType = new RelationType(db, idIn)
      /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */

      // (Didn't use next line as in other cases, because of type issues in attempting to create the required "... new Writes[<: Relationtype]" method or
      // something along those lines. And this way probably makes for less total code anyway.
      //val json1: JsValue = Json.toJson(relationType)

      val json: JsValue = Json.obj("name" -> relationType.getName,
                                   "nameInReverseDirection" -> relationType.getNameInReverseDirection,
                                   "directionality" -> relationType.getDirectionality
                                  )
      Ok(Json.prettyPrint(json)).as(JSON)
    }
  }

  def getQuantityAttributeData(idIn: Long): Action[AnyContent] = Action { implicit request =>
    val exists: Boolean = db.quantityAttributeKeyExists(idIn)
    /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
    if (!exists) {
      Ok(JsNull).as(JSON)
    } else {
      val attribute = new QuantityAttribute(db, idIn)
      //(About converting to json: see comment at similar line in method getRelationTypeData.)
      val json: JsValue = Json.obj("parentId" -> attribute.getParentId,
                                   "unitId" -> attribute.getUnitId,
                                   "number" -> attribute.getNumber,
                                   "type" -> attribute.getAttrTypeId,
                                   "validOnDate" -> attribute.getValidOnDate,
                                   "observationDate" -> attribute.getObservationDate,
                                   "sortingIndex" -> attribute.getSortingIndex
                                  )
      Ok(Json.prettyPrint(json)).as(JSON)
    }
  }

  def getDateAttributeData(idIn: Long): Action[AnyContent] = Action { implicit request =>
    val exists: Boolean = db.dateAttributeKeyExists(idIn)
    if (!exists) {
      Ok(JsNull).as(JSON)
    } else {
      val attribute = new DateAttribute(db, idIn)
      //(About converting to json: see comment at similar line in method getRelationTypeData.)
      val json: JsValue = Json.obj("parentId" -> attribute.getParentId,
                                   "date" -> attribute.getDate,
                                   "type" -> attribute.getAttrTypeId,
                                   "sortingIndex" -> attribute.getSortingIndex
                                  )
      Ok(Json.prettyPrint(json)).as(JSON)
    }
  }

  def getBooleanAttributeData(idIn: Long): Action[AnyContent] = Action { implicit request =>
    val exists: Boolean = db.booleanAttributeKeyExists(idIn)
    /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
    if (!exists) {
      Ok(JsNull).as(JSON)
    } else {
      val attribute = new BooleanAttribute(db, idIn)
      //(About converting to json: see comment at similar line in method getRelationTypeData.)
      val json: JsValue = Json.obj("parentId" -> attribute.getParentId,
                                   "boolean" -> attribute.getBoolean,
                                   "type" -> attribute.getAttrTypeId,
                                   "validOnDate" -> attribute.getValidOnDate,
                                   "observationDate" -> attribute.getObservationDate,
                                   "sortingIndex" -> attribute.getSortingIndex
                                  )
      Ok(Json.prettyPrint(json)).as(JSON)
    }
  }

  def getTextAttributeData(idIn: Long): Action[AnyContent] = Action { implicit request =>
    val exists: Boolean = db.textAttributeKeyExists(idIn)
    /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
    if (!exists) {
      Ok(JsNull).as(JSON)
    } else {
      val attribute = new TextAttribute(db, idIn)
      //(About converting to json: see comment at similar line in method getRelationTypeData.)
      val json: JsValue = Json.obj("parentId" -> attribute.getParentId,
                                   "text" -> attribute.getText,
                                   "type" -> attribute.getAttrTypeId,
                                   "validOnDate" -> attribute.getValidOnDate,
                                   "observationDate" -> attribute.getObservationDate,
                                   "sortingIndex" -> attribute.getSortingIndex
                                  )
      Ok(Json.prettyPrint(json)).as(JSON)
    }
  }

  def getRelationToEntityData(relationTypeIdIn: Long, entityId1In: Long, entityId2In: Long): Action[AnyContent] = Action { implicit request =>
    val exists: Boolean = db.relationToEntityExists(relationTypeIdIn, entityId1In, entityId2In)
    /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
    if (!exists) {
      Ok(JsNull).as(JSON)
    } else {
      val rteData = db.getRelationToEntityData(relationTypeIdIn, entityId1In, entityId2In)
      val json: JsValue = Json.obj("id" -> rteData(0).get.asInstanceOf[Long],
                                   "validOnDate" -> rteData(1).asInstanceOf[Option[Long]],
                                   "observationDate" -> rteData(2).get.asInstanceOf[Long],
                                   "sortingIndex" -> rteData(3).get.asInstanceOf[Long]
                                  )
      Ok(Json.prettyPrint(json)).as(JSON)
    }
  }

  def getRelationToRemoteEntityData(relationTypeIdIn: Long, entityId1In: Long, remoteInstanceIdIn: String,
                                    entityId2In: Long): Action[AnyContent] = Action { implicit request =>
    val exists: Boolean = db.relationToRemoteEntityExists(relationTypeIdIn, entityId1In, remoteInstanceIdIn, entityId2In)
    /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
    if (!exists) {
      Ok(JsNull).as(JSON)
    } else {
      val rteData = db.getRelationToRemoteEntityData(relationTypeIdIn, entityId1In, remoteInstanceIdIn, entityId2In)
      val json: JsValue = Json.obj("id" -> rteData(0).get.asInstanceOf[Long],
                                   "validOnDate" -> rteData(1).asInstanceOf[Option[Long]],
                                   "observationDate" -> rteData(2).get.asInstanceOf[Long],
                                   "sortingIndex" -> rteData(3).get.asInstanceOf[Long]
                                  )
      Ok(Json.prettyPrint(json)).as(JSON)
    }
  }

  def getRelationToGroupData(id: Long): Action[AnyContent] = Action { implicit request =>
    val exists: Boolean = db.relationToGroupKeyExists(id)
    /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
    if (!exists) {
      Ok(JsNull).as(JSON)
    } else {
      val rtgData = db.getRelationToGroupData(id)
      val json: JsValue = Json.obj("id" -> rtgData(0).get.asInstanceOf[Long],
                                   "entityId" -> rtgData(1).get.asInstanceOf[Long],
                                   "relationTypeId" -> rtgData(2).get.asInstanceOf[Long],
                                   "groupId" -> rtgData(3).get.asInstanceOf[Long],
                                   "validOnDate" -> rtgData(4).asInstanceOf[Option[Long]],
                                   "observationDate" -> rtgData(5).get.asInstanceOf[Long],
                                   "sortingIndex" -> rtgData(6).get.asInstanceOf[Long]
                                  )
      Ok(Json.prettyPrint(json)).as(JSON)
    }
  }

  def getRelationToGroupDataByKeys(entityId: Long, relationTypeId: Long, groupId: Long): Action[AnyContent] = Action { implicit request =>
    val exists: Boolean = db.relationToGroupKeysExist(entityId, relationTypeId, groupId)
    /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
    if (!exists) {
      Ok(JsNull).as(JSON)
    } else {
      val rtgData = db.getRelationToGroupDataByKeys(entityId, relationTypeId, groupId)
      val json: JsValue = Json.obj("id" -> rtgData(0).get.asInstanceOf[Long],
                                   "entityId" -> rtgData(1).get.asInstanceOf[Long],
                                   "relationTypeId" -> rtgData(2).get.asInstanceOf[Long],
                                   "groupId" -> rtgData(3).get.asInstanceOf[Long],
                                   "validOnDate" -> rtgData(4).asInstanceOf[Option[Long]],
                                   "observationDate" -> rtgData(5).get.asInstanceOf[Long],
                                   "sortingIndex" -> rtgData(6).get.asInstanceOf[Long]
                                  )
      Ok(Json.prettyPrint(json)).as(JSON)
    }
  }

  implicit val omInstanceWrites = new Writes[OmInstance] {
    def writes(omInstanceIn: OmInstance) = {
      Json.obj(
                "local" -> omInstanceIn.getLocal,
                "address" -> omInstanceIn.getAddress,
                "insertionDate" -> omInstanceIn.getCreationDate,
                // FYI about the next line & similar ones: a manual test indicated that calling a method that returns Option[someValue] is the same
                // as calling "if (omInstanceIn.getEntityId.isEmpty) JsNull else Some(omInstanceIn.getEntityId)) ...", in
                // how they will pass JsNull in the end if it is None.
                "entityId" -> omInstanceIn.getEntityId
              )
    }
  }

  /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
  def getOmInstanceData(idIn: String): Action[AnyContent] = Action { implicit request =>
    val exists: Boolean = db.omInstanceKeyExists(idIn)
    if (!exists) {
      Ok(JsNull).as(JSON)
    } else {
      val omi = new OmInstance(db, idIn)
      /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
      val json: JsValue = Json.toJson(omi)
      Ok(Json.prettyPrint(json)).as(JSON)
    }
  }

  // (About this json conversion: see comment on "implicit val entityWrites".)
  implicit val fileAttributeWrites = new Writes[FileAttribute] {
    def writes(fileAttributeIn: FileAttribute) = {
      Json.obj(
                "entityId" -> fileAttributeIn.getParentId,
                "description" -> fileAttributeIn.getDescription,
                "attributeTypeId" -> fileAttributeIn.getAttrTypeId,
                "originalFileDate" -> fileAttributeIn.getOriginalFileDate,
                "storedDate" -> fileAttributeIn.getStoredDate,
                "originalFilePath" -> fileAttributeIn.getOriginalFilePath,
                "readable" -> fileAttributeIn.getReadable,
                "writable" -> fileAttributeIn.getWritable,
                "executable" -> fileAttributeIn.getExecutable,
                "size" -> fileAttributeIn.getSize,
                "md5Hash" -> fileAttributeIn.getMd5Hash,
                "sortingIndex" -> fileAttributeIn.getSortingIndex
              )
    }
  }

  /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
  def getFileAttributeData(idIn: Long): Action[AnyContent] = Action { implicit request =>
    val exists: Boolean = db.fileAttributeKeyExists(idIn)
    if (!exists) {
      Ok(JsNull).as(JSON)
    } else {
      val fa = new FileAttribute(db, idIn)
      /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
      val json: JsValue = Json.toJson(fa)
      Ok(Json.prettyPrint(json)).as(JSON)
    }
  }

  /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
  def getGroupData(idIn: Long): Action[AnyContent] = Action { implicit request =>
    val exists: Boolean = db.groupKeyExists(idIn)
    if (!exists) {
      Ok(JsNull).as(JSON)
    } else {
      val group = new Group(db, idIn)
      /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
      val json = Json.obj("name" -> group.getName,
                          "insertionDate" -> group.getInsertionDate,
                          "mixedClassesAllowed" -> group.getMixedClassesAllowed,
                          "newEntriesStickToTop" -> group.getNewEntriesStickToTop
                         )
      Ok(Json.prettyPrint(json)).as(JSON)
    }
  }

  /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
  def getEntityData(idIn: Long): Action[AnyContent] = Action { implicit request =>
    val exists: Boolean = db.entityKeyExists(idIn)
    if (!exists) {
      Ok(JsNull).as(JSON)
    } else {
      val entity = new Entity(db, idIn)
      /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
      val json = Json.obj("name" -> entity.getName,
                          "classId" -> entity.getClassId,
                          "insertionDate" -> entity.getInsertionDate,
                          "isPublic" -> entity.getPublic,
                          "isArchived" -> entity.getArchivedStatus,
                          "newEntriesStickToTop" -> entity.getNewEntriesStickToTop
                         )
      Ok(Json.prettyPrint(json)).as(JSON)
    }
  }

  /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
  def getContainingGroupsIds(idIn: Long): Action[AnyContent] = Action { implicit request =>
    val results: util.ArrayList[Long] = db.getContainingGroupsIds(idIn)
    getResultingIdsAsJson(results)
  }

  /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
  def findRelationType(nameIn: String, expectedRowsIn: Option[Int]): Action[AnyContent] = Action { implicit request =>
    val results: util.ArrayList[Long] = db.findRelationType(nameIn, expectedRowsIn)
    getResultingIdsAsJson(results)
  }

  /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
  def getGroupEntryObjects(groupIdIn: Long, startingObjectIndexIn: Long, maxValsIn: Option[Long]): Action[AnyContent] = Action { implicit request =>
    val results: ArrayList[Entity] = db.getGroupEntryObjects(groupIdIn, startingObjectIndexIn, maxValsIn)
    if (results.isEmpty) {
      Ok(JsNull).as(JSON)
    } else {
      var json = Json.arr()
      for (result: Entity <- results) {
        json = json.append(Json.obj("entityId" -> result.getId,
                                    "name" -> result.getName,
                                    "classId" -> result.getClassId,
                                    "insertionDate" -> result.getInsertionDate,
                                    "public" -> result.getPublic,
                                    "archived" -> result.isArchived,
                                    "newEntriesStickToTop" -> result.getNewEntriesStickToTop))
      }
      Ok(Json.prettyPrint(json)).as(JSON)
    }
  }
  def getResultingRelationTypeIdAndEntityAsJson(results: util.ArrayList[(Long, Entity)]): Result = {
    if (results.isEmpty) {
      Ok(JsNull).as(JSON)
    } else {
      var json = Json.arr()
      for (result: (Long, Entity) <- results) {
        json = json.append(Json.obj("entityId" -> result._2.getId,
                                    "name" -> result._2.getName,
                                    "classId" -> result._2.getClassId,
                                    "insertionDate" -> result._2.getInsertionDate,
                                    "public" -> result._2.getPublic,
                                    "archived" -> result._2.isArchived,
                                    "newEntriesStickToTop" -> result._2.getNewEntriesStickToTop,
                                    "relationTypeId" -> result._1))
      }
      Ok(Json.prettyPrint(json)).as(JSON)
    }
  }

  /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
  def getEntitiesContainingGroup(groupIdIn: Long, startingIndexIn: Long, maxValsIn: Option[Long]): Action[AnyContent] = Action { implicit request =>
    val results: ArrayList[(Long, Entity)] = db.getEntitiesContainingGroup(groupIdIn, startingIndexIn, maxValsIn)
    getResultingRelationTypeIdAndEntityAsJson(results)
  }

  /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
  def getEntitiesContainingEntity(entityIdIn: Long, startingIndexIn: Long, maxValsIn: Option[Long]): Action[AnyContent] = Action { implicit request =>
    val results: util.ArrayList[(Long, Entity)] = db.getEntitiesContainingEntity(entityIdIn, startingIndexIn, maxValsIn)
    getResultingRelationTypeIdAndEntityAsJson(results)
  }

  /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
  def getCountOfEntitiesContainingGroup(groupIdIn: Long): Action[AnyContent] = Action { implicit request =>
    val results: (Long, Long) = db.getCountOfEntitiesContainingGroup(groupIdIn)
    val json = Json.obj("nonArchived" -> results._1, "archived" -> results._2)
    Ok(Json.prettyPrint(json)).as(JSON)
  }

  /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
  def getCountOfEntitiesContainingEntity(idIn: Long): Action[AnyContent] = Action { implicit request =>
    val results: (Long, Long) = db.getCountOfEntitiesContainingEntity(idIn)
    val json = Json.obj("nonArchived" -> results._1, "archived" -> results._2)
    Ok(Json.prettyPrint(json)).as(JSON)
  }

  /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
  def getFileAttributeContent(fileAttributeIdIn: Long): Action[AnyContent] = Action { implicit request =>
    // References:
    //   https://www.playframework.com/documentation/2.5.x/ScalaStream#Serving-files
    //   or search for something like "play REST file download" etc.

    //Note: This approach (per the docs at:
    //   https://www.playframework.com/documentation/2.5.x/ScalaStream#Serving-files
    // ...might be best for very large files, and seems handy for doing this from a stream directly from postgresql, instead of having to save
    // the file on the local filesystem first), was getting errors in the client like "java.io.IOException: Premature EOF" or
    // "ERR_INCOMPLETE_CHUNKED_ENCODING" (in chromium).
    // Idea: maybe a way to see if the problem is in core module code or here, would be to stream a test file out here directly, instead of calling
    // db.getFileAttributeContent(fileAttributeIdIn, output); then (if that works better) possibly change its streaming code in some way to further isolate it.
    // Or, maybe I need to learn/understand much better the APIs around "StreamConverters.fromInputStream(getDataStream)" etc where called.
     /*
    val output = new java.io.PipedOutputStream
    val input = new java.io.PipedInputStream(output)
    var fileSize: Long = 0
    var md5hash = ""
    val writingContentThread = new Thread {
      override def run(): Unit = {
        val (fs, md5) = db.getFileAttributeContent(fileAttributeIdIn, output)
        fileSize = fs
        md5hash = md5
      }
    }
    writingContentThread.start()
    //    val CHUNK_SIZE = 100
    def getDataStream(): InputStream = {
      input
    }
    val dataContent: Source[ByteString, Future[IOResult]] = StreamConverters.fromInputStream(getDataStream)
    Ok.chunked(dataContent)
    // Note: if the above is ever used, it still needs a check on the content-type header, the filename passed to client, & testing;
    // and, if needed?, have a Thread.`yield`() inside db.getFileAttributeContent's "action" method (if doesn't works well without, under load)?--it might
    // not be needed since javadoc for Thread says it is "rarely appropriate". Hm.
// */

    // So because of the problems per the above comments, doing it this way instead:
    val fa = new FileAttribute(db, fileAttributeIdIn)
    val name = FilenameUtils.getBaseName(fa.getOriginalFilePath)
    val (prefix, suffix): (String, String) = Util.getUsableFilename(name)
    val path: Path = java.nio.file.Files.createTempFile(prefix, suffix)
    val onClose = () => {
      java.nio.file.Files.delete(path)
    }
    fa.retrieveContent(path.toFile)
    Ok.sendPath(path, inline = true, _ => prefix + suffix, onClose)
  }

  /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
  def findRelationToAndGroup_OnEntity(entityIdIn: Long, groupNameIn: Option[String]): Action[AnyContent] = Action { implicit request =>
    val result: (Option[Long], Option[Long], Option[Long], Boolean) = db.findRelationToAndGroup_OnEntity(entityIdIn, groupNameIn)
    val json: JsValue = Json.obj("relationToGroupId" -> result._1,
                                 "relationTypeId" -> result._2,
                                 "groupId" -> result._3,
                                 "moreRowsAvailable" -> result._4.asInstanceOf[Boolean]
                                )
    Ok(Json.prettyPrint(json)).as(JSON)
  }

  /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
  def getSortedAttributes(entityIdIn: Long, startingObjectIndexIn: Int, maxValsIn: Int,
                          onlyPublicEntitiesIn: Boolean): Action[AnyContent] = Action { implicit request =>
    val sortedAttributeInfo: (Array[(Long, Attribute)], Int) = db.getSortedAttributes(entityIdIn, startingObjectIndexIn, maxValsIn, onlyPublicEntitiesIn)
    val attributeTuples = sortedAttributeInfo._1
    val attributes: Array[Attribute] = new Array[Attribute](attributeTuples.length)
    var index = 0
    for (attrTuple <- attributeTuples) {
      val sortingIndex: Long = attrTuple._1
      attributes(index) = attrTuple._2
      val attribute = attributes(index)
      require(attribute.getParentId == entityIdIn,
              "Unexpected: attribute that is supposed to be on entity " + entityIdIn + " has parentId of " + attribute.getParentId + "?")
      require(attribute.getSortingIndex == sortingIndex,
              "Unexpected: attribute that is supposed to be on entity " + entityIdIn + " has parentId of " + attribute.getParentId + ", and sorting" +
              "indices don't match: " + sortingIndex + " (from getSortingIndex) and " + attribute.getSortingIndex + "(from attribute object)?")
      index += 1
    }
    val json =
      Json.obj("totalAttributesAvailable" -> sortedAttributeInfo._2,
               "attributes" -> attributes.map { attribute =>
                 var jsonObject: JsObject =
                   Json.obj("id" -> attribute.getId,
                            "formId" -> attribute.getFormId,
                            "parentId" -> attribute.getParentId,
                            "attrTypeId" -> attribute.getAttrTypeId,
                            "sortingIndex" -> JsNumber(attribute.getSortingIndex)
                            )

                 attribute match {
                   case a: QuantityAttribute =>
                     // Idea: for questions of possible data loss or improving how we transfer numbers into & out of OM instances, see
                     // http://www.scala-lang.org/api/current/index.html#scala.math.BigDecimal$
                     // ...but consider which documentation applies for the version of scala in use.
                     jsonObject = jsonObject + ("validOnDate" -> (if (a.getValidOnDate.isEmpty) JsNull else JsNumber(a.getValidOnDate.get)))
                     jsonObject = jsonObject + ("observationDate" -> JsNumber(a.getObservationDate))
                     jsonObject = jsonObject + ("unitId" -> JsNumber(a.getUnitId))
                     jsonObject = jsonObject + ("number" -> JsNumber(BigDecimal.double2bigDecimal(a.getNumber)))
                   case a: DateAttribute =>
                     // could instead or in addition use ISO8601 for dates, for benefit of other clients besides OM itself (see
                     // everywhere w/ this comment):
                     jsonObject = jsonObject + ("date" -> JsNumber(a.getDate))
                   case a: BooleanAttribute =>
                     jsonObject = jsonObject + ("validOnDate" -> (if (a.getValidOnDate.isEmpty) JsNull else JsNumber(a.getValidOnDate.get)))
                     jsonObject = jsonObject + ("observationDate" -> JsNumber(a.getObservationDate))
                     jsonObject = jsonObject + ("boolean" -> JsBoolean(a.getBoolean))
                   case a: FileAttribute =>
                     jsonObject = jsonObject + ("description" -> JsString(a.getDescription))
                     jsonObject = jsonObject + ("originalFileDate" -> JsNumber(a.getOriginalFileDate))
                     jsonObject = jsonObject + ("storedDate" -> JsNumber(a.getStoredDate))
                     jsonObject = jsonObject + ("originalFilePath" -> JsString(a.getOriginalFilePath))
                     jsonObject = jsonObject + ("readable" -> JsBoolean(a.getReadable))
                     jsonObject = jsonObject + ("writable" -> JsBoolean(a.getWritable))
                     jsonObject = jsonObject + ("executable" -> JsBoolean(a.getExecutable))
                     jsonObject = jsonObject + ("sizeInBytes" -> JsNumber(a.getSize))
                     jsonObject = jsonObject + ("md5hash" -> JsString(a.getMd5Hash))
                   /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */
                   case a: TextAttribute =>
                     jsonObject = jsonObject + ("validOnDate" -> (if (a.getValidOnDate.isEmpty) JsNull else JsNumber(a.getValidOnDate.get)))
                     jsonObject = jsonObject + ("observationDate" -> JsNumber(a.getObservationDate))
                     jsonObject = jsonObject + ("text" -> JsString(org.apache.commons.lang3.StringEscapeUtils.escapeJson(a.getText)))
                   case a: RelationToRemoteEntity =>
                     jsonObject = jsonObject + ("validOnDate" -> (if (a.getValidOnDate.isEmpty) JsNull else JsNumber(a.getValidOnDate.get)))
                     jsonObject = jsonObject + ("observationDate" -> JsNumber(a.getObservationDate))
                     jsonObject = jsonObject + ("entity1Id" -> JsNumber(a.getRelatedId1))
                     require(a.getRelatedId1 == entityIdIn)
                     jsonObject = jsonObject + ("remoteInstanceId" -> JsString(a.asInstanceOf[RelationToRemoteEntity].getRemoteInstanceId))
                     jsonObject = jsonObject + ("entity2Id" -> JsNumber(a.getRelatedId2))
                   case a: RelationToEntity =>
                     // NOTE: this case should come *after* that for RelationToRemoteEntity above, because RelationToRemoteEntity is a subtype of RTE and we don't want
                     // to skip either one.
                     jsonObject = jsonObject + ("validOnDate" -> (if (a.getValidOnDate.isEmpty) JsNull else JsNumber(a.getValidOnDate.get)))
                     jsonObject = jsonObject + ("observationDate" -> JsNumber(a.getObservationDate))
                     jsonObject = jsonObject + ("entity1Id" -> JsNumber(a.getRelatedId1))
                     require(a.getRelatedId1 == entityIdIn)
                     jsonObject = jsonObject + ("entity2Id" -> JsNumber(a.getRelatedId2))
                   case a: RelationToGroup =>
                     jsonObject = jsonObject + ("validOnDate" -> (if (a.getValidOnDate.isEmpty) JsNull else JsNumber(a.getValidOnDate.get)))
                     jsonObject = jsonObject + ("observationDate" -> JsNumber(a.getObservationDate))
                     jsonObject = jsonObject + ("entityId" -> JsNumber(a.getParentId))
                     require(a.getParentId == entityIdIn)
                     jsonObject = jsonObject + ("groupId" -> JsNumber(a.getGroupId))
                   case _ => throw new OmException("Unexpected type: " + attribute.getClass.getCanonicalName)
                 }
                 jsonObject
               })
    Ok(Json.prettyPrint(json)).as(JSON)
  }

  //Keep cmt scattered thru all future code here to make sure & remember. Is there a better way to be sure/remember?:
  /* * * * **ONLY PROVIDE PUBLIC INFO (SAME COMMENT IN BOTH MODULES, EVERYWHERE!): HOW TO REMEMBER/BE SURE?? (4now at least) */

}
