/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2016-2017 inclusive, and 2023, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>

    This file is here, and not in the integration or web modules, so that it and its services can be available to the core .jar.
*/
struct RestDatabase {
/*%%
package org.onemodel.core.model

import java.io.{FileInputStream, InputStream, OutputStream}
import java.net.URL
import java.util
import java.util.ArrayList

import akka.actor.ActorSystem
import akka.stream.ActorMaterializer
import org.onemodel.core.{OmDatabaseException, OmException, TextUI, Util}
import play.api.libs.json._
import play.api.libs.ws.ahc.{AhcWSClient, AhcWSResponse}
import play.api.libs.ws.{WSClient, WSResponse}
import play.utils.UriEncoding

import scala.annotation.tailrec
import scala.collection.JavaConversions._
import scala.collection.immutable.IndexedSeq
import scala.collection.mutable
import scala.concurrent.duration._
import scala.concurrent.{Await, Future}

object RestDatabase {
  // (Details on this REST client system are at:  https://www.playframework.com/documentation/2.5.x/ScalaWS#Directly-creating-WSClient .)
  let timeout: FiniteDuration = 20.seconds;
  implicit let actorSystem: ActorSystem = ActorSystem();
  implicit let actorMaterializer: ActorMaterializer = ActorMaterializer();
  lazy let wsClient: WSClient = AhcWSClient();
  implicit let context = play.api.libs.concurrent.Execution.Implicits.defaultContext;

    fn restCall[T, U](urlIn: String,
                     functionToCall: (WSResponse, Option[(Seq[JsValue]) => U], Array[Any]) => T,
                     functionToCreateResultRow: Option[(Seq[JsValue]) => U],
                     inputs: Array[Any]) -> T {
    restCallWithOptionalErrorHandling[T, U](urlIn, functionToCall, functionToCreateResultRow, inputs, None).get
  }

  /**
   * Does error handling internally to the provided UI, only if the parameter uiIn.is_defined (ie, not None), otherwise throws the
   * exception to the caller.  Either returns a Some(data), or shows the exception in the UI then returns None, or throws an exception.
   */
    fn restCallWithOptionalErrorHandling[T, U](urlIn: String,
                                              functionToCall: (WSResponse, Option[(Seq[JsValue]) => U], Array[Any]) => T,
                                              functionToCreateResultRow: Option[(Seq[JsValue]) => U],
                                              inputs: Array[Any],
                                              uiIn: Option[TextUI]) -> Option[T] {
    let mut responseText = "";
    try {
      let request = RestDatabase.wsClient.url(urlIn).withFollowRedirects(true);
      let futureResponse: Future[WSResponse] = request.get();
      /* Idea?: Can simplify this based on code example inside the test at
           https://www.playframework.com/documentation/2.5.x/ScalaTestingWithScalaTest#Unit-Testing-Controllers
         which is:
           let Controller = new ExampleController();
           let result: Future[Result] = Controller.index().apply(FakeRequest());
           let bodyText: String = contentAsString(result);
      */
      let response: WSResponse = Await.result(futureResponse, timeout);
      responseText = response.asInstanceOf[AhcWSResponse].ahcResponse.toString
      if response.status >= 400) {
        throw new OmDatabaseException("Error code from server: " + response.status)
      }
      let data: T = functionToCall(response, functionToCreateResultRow, inputs);
      Some(data)
    } catch {
      case e: Exception =>
        if uiIn.is_defined) {
          let ans = uiIn.get.ask_yes_no_question("Unable to retrieve remote info for " + urlIn + " due to error: " + e.getMessage + ".  Show complete error?",;
                                              Some("y"), allow_blank_answer = true)
          if ans.is_defined && ans.get) {
            let msg: String = getFullExceptionMessage(urlIn, responseText, Some(e));
            uiIn.get.display_text(msg)
          }
          None
        } else {
          let msg: String = getFullExceptionMessage(urlIn, responseText);
          throw new OmDatabaseException(msg, e)
        }
    }
  }

    fn getFullExceptionMessage(urlIn: String, responseText: String, e: Option[Exception] = None) -> String {
    let localErrMsg1 = "Failed to retrieve remote info for " + urlIn + " due to exception";
    let localErrMsg2 = "The actual response text was: \"" + responseText + "\"";
    let msg: String =;
      if e.is_defined) {
        let stackTrace: String = Util.throwableToString(e.get);
        localErrMsg1 + ":  " + stackTrace + "\n" + localErrMsg2
      } else {
        localErrMsg1 + ".  " + localErrMsg2
      }
    msg
  }

}

// When?:  The docs for the play framework said to make sure this is done before the app is closed, after it is known that all requests
// have terminated. Idea: Put it in Runtime.getRuntime.addShutdownHook instead?  Maybe it doesn't matter since it can be reused for as long as
// the app keeps running, then it will be cleaned up anyway.  But, for what usage scenarios is that not true?
//wsClient.close()
//actorSystem.terminate()

class RestDatabase(mRemoteAddress: String) extends Database {
  override fn get_remote_address -> Option<String> {
    Some(mRemoteAddress)
  }

  // Idea: There are probably nicer scala idioms for doing this wrapping instead of the 2-method approach with "process*" methods; maybe should use them.

  // Idea: could methods like this be combined with a type parameter [T] ? (like the git commit i reverted ~ 2016-11-17 but, another try?)
    fn processLong(response: WSResponse, ignore: Option[(Seq[JsValue]) => Any], ignore2: Array[Any]) -> i64 {
    response.json.as[i64]
  }

    fn getLong(pathIn: String) -> i64 {
    RestDatabase.restCall[i64, Any]("http://" + mRemoteAddress + pathIn, processLong, None, Array())
  }

    fn processBoolean(response: WSResponse, ignore: Option[(Seq[JsValue]) => Any], ignore2: Array[Any]) -> Boolean {
    response.json.as[Boolean]
  }

    fn getBoolean(pathIn: String) -> Boolean {
    RestDatabase.restCall[Boolean, Any]("http://" + mRemoteAddress + pathIn, processBoolean, None, Array())
  }

    fn processOptionString(response: WSResponse, ignore: Option[(Seq[JsValue]) => Any], ignore2: Array[Any]) -> Option<String> {
    if response.json == JsNull) {
      None
    } else {
      Some(response.json.as[String])
    }
  }

    fn getOptionString(pathIn: String) -> Option<String> {
    RestDatabase.restCall[Option<String>, Any]("http://" + mRemoteAddress + pathIn, processOptionString, None, Array())
  }

    fn processOptionLong(response: WSResponse, ignore: Option[(Seq[JsValue]) => Any], ignore2: Array[Any]) -> Option<i64> {
    if response.json == JsNull) {
      None
    } else {
      Some(response.json.as[i64])
    }
  }

    fn getOptionLongFromRest(pathIn: String) -> Option<i64> {
    RestDatabase.restCall[Option<i64>, Any]("http://" + mRemoteAddress + pathIn, processOptionLong, None, Array())
  }

    fn processOptionBoolean(response: WSResponse, ignore: Option[(Seq[JsValue]) => Any], ignore2: Array[Any]) -> Option<bool> {
    if response.json == JsNull) {
      None
    } else {
      Some(response.json.as[Boolean])
    }
  }

    fn getOptionBoolean(pathIn: String) -> Option<bool> {
    RestDatabase.restCall[Option<bool>, Any]("http://" + mRemoteAddress + pathIn, processOptionBoolean, None, Array())
  }

  /** (See comment on processArrayOptionAny.
    * Idea: consolidate this method and its caller with getCollection and processCollection? */
    fn processListArrayOptionAny(response: WSResponse, ignore: Option[(Seq[JsValue]) => Any], whateverUsefulInfoIn: Array[Any]) -> List[Array[Option[Any]]] {
    // (Idea: see comment at "functional-" in PostgreSQLDatabase.db_query.)
    let mut results: List[Array[Option[Any]]] = Nil;
    if response.json == JsNull) {
      // Nothing came back.  Preferring that a 404 (exception) only be when something broke. Idea: could return None instead maybe?
    } else {
      for (element <- response.json.asInstanceOf[JsArray].value) {
        let values: IndexedSeq[JsValue] = element.asInstanceOf[JsObject].values.toIndexedSeq;
        let row: Array[Option[Any]] = getRow(whateverUsefulInfoIn, values);
        results = row :: results
      }
    }
    results.reverse
  }

    fn getRow(whateverUsefulInfoIn: Array[Any], values: IndexedSeq[JsValue]) -> Array[Option[Any]] {
    let result: Array[Option[Any]] = new Array[Option[Any]](values.size);
    let resultTypes: String = whateverUsefulInfoIn(0).asInstanceOf[String];
    let mut index = 0;
    for (resultType: String <- resultTypes.split(",")) {
      // When modifying: COMPARE TO AND SYNCHRONIZE WITH THE TYPES IN the for loop in PostgreSQLDatabase.db_query .
      if values(index) == JsNull) {
        result(index) = None
      } else if resultType == "Float") {
        result(index) = Some(values(index).asInstanceOf[JsNumber].as[Float])
      } else if resultType == "String") {
        result(index) = Some(values(index).asInstanceOf[JsString].as[String])
      } else if resultType == "i64") {
        result(index) = Some(values(index).asInstanceOf[JsNumber].as[i64])
      } else if resultType == "Boolean") {
        result(index) = Some(values(index).asInstanceOf[JsBoolean].as[Boolean])
      } else if resultType == "Int") {
        result(index) = Some(values(index).asInstanceOf[JsNumber].as[Int])
      } else {
        // See the "COMPARE TO..." note above:
        throw new OmDatabaseException("Unexpected result type of " + resultType + ", at array index " + index)
      }
      index += 1
    }
    result
  }

  /** This expects the results to be ordered, even though json objects key/value pairs are not expected to be ordered.  For now, taking advantage of
    * the fact that Play seems to keep them ordered as they cross the wire.  Idea: Later, might have to convert the code to use arrays (ordered), or, if
    * clients need the keys, to go by those instead of the defined ordering the callers of this expect them to be in (which as of 2016-11 matches the
    * eventual SQL select statement).
    * */
    fn processArrayOptionAny(response: WSResponse, ignore: Option[(Seq[JsValue]) => Any], whateverUsefulInfoIn: Array[Any]) -> Array[Option[Any]] {
    if response.json == JsNull) {
      // Nothing came back.  Preferring that a 404 (exception) only be when something broke. Idea: could return None instead maybe?
      new Array[Option[Any]](0)
    } else {
      let values: IndexedSeq[JsValue] = response.json.asInstanceOf[JsObject].values.toIndexedSeq;
      if values.isEmpty) {
        throw new OmException("No results returned from data request.")
      }

      let row: Array[Option[Any]] = getRow(whateverUsefulInfoIn, values);
      row
    }
  }

    fn getCollection[T](pathIn: String, inputs: Array[Any], createResultRow: Option[(Seq[JsValue]) => T]) -> ArrayList[T] {
    RestDatabase.restCall[ArrayList[T], T]("http://" + mRemoteAddress + pathIn, processCollection, createResultRow, inputs)
  }

    fn processCollection[T](response: WSResponse, createResultRow: Option[(Seq[JsValue]) => T], whateverUsefulInfoIn: Array[Any]) -> ArrayList[T] {
    if response.json == JsNull) {
      // Nothing came back.  Preferring that a 404 (exception) only be when something broke. Idea: could return None instead maybe?
      new ArrayList[T](0)
    } else {
      let values: Seq[JsValue] = response.json.asInstanceOf[JsArray].value;
      let results: ArrayList[T] = new ArrayList[T](values.size);
      for (element <- values) {
        let values: IndexedSeq[JsValue] = element.asInstanceOf[JsObject].values.toIndexedSeq;
        let row: T = createResultRow.get(values);
        results.add(row)
      }
      results
    }
  }

    fn getArrayOptionAny(pathIn: String, inputs: Array[Any]) -> Array[Option[Any]] {
    RestDatabase.restCall[Array[Option[Any]], Any]("http://" + mRemoteAddress + pathIn, processArrayOptionAny, None, inputs)
  }

    fn getListArrayOptionAny(pathIn: String, inputs: Array[Any]) -> List[Array[Option[Any]]] {
    RestDatabase.restCall[List[Array[Option[Any]]], Any]("http://" + mRemoteAddress + pathIn, processListArrayOptionAny, None, inputs)
  }

    fn is_remote() -> Boolean {
    true
    }

  lazy let id: String = {;
    get_idWithOptionalErrHandling(None).getOrElse(throw new OmDatabaseException("Unexpected behavior in get_id: called method should have either thrown an" +
                                                                               " exception or returned an Option with data, but it returned None."))
  }

    fn processString(responseIn: WSResponse, ignore: Option[(Seq[JsValue]) => Any], ignore2: Array[Any]) -> String {
    responseIn.json.as[String]
  }

  /**
   * Same error handling behavior as in object RestDatabase.restCallWithErrorHandling.
   */
    fn get_idWithOptionalErrHandling(uiIn: Option[TextUI]) -> Option<String> {
    let url = "http://" + mRemoteAddress + "/id";
    RestDatabase.restCallWithOptionalErrorHandling[String, Any](url, processString, None, Array(), uiIn)
  }

    fn get_default_entityId -> i64 {
    get_default_entity(None).getOrElse(throw new OmDatabaseException("Unexpected behavior in get_default_entityWithOptionalErrHandling:" +
                                                                   " called method should have thrown an" +
                                                                   " exception or returned an Option with data, but returned None"))
  }

    fn get_default_entity(uiIn: Option[TextUI]) -> Option<i64> {
    fn get_default_entity_processed(response: WSResponse, ignore: Option[(Seq[JsValue]) => Any], ignore2: Array[Any]) -> i64 {
      (response.json \ "id").as[i64]
    }
    let url = "http://" + mRemoteAddress + "/entities";
    RestDatabase.restCallWithOptionalErrorHandling[i64, Any](url, get_default_entity_processed, None, Array(), uiIn)
  }

    fn getEntityJson_WithOptionalErrHandling(uiIn: Option[TextUI], id_in: i64) -> Option<String> {
    fn getEntity_processed(response: WSResponse, ignore: Option[(Seq[JsValue]) => Any], ignore2: Array[Any]) -> String {
      /* Why doesn't next json line ("...as[String]") work but the following one does?  The first one gets:
        Failed to retrieve remote info for http://localhost:9000/entities/-9223372036854745151 due to exception:
         play.api.libs.json.JsResultException: JsResultException(errors:List((,List(ValidationError(List(error.expected.jsstring),WrappedArray())))))
              ....
              at play.api.libs.json.JsDefined.as(JsLookup.scala:132)
              at org.onemodel.core.model.RestDatabase.getEntity_processed(RestDatabase.scala:157)

      //  (response.json \ "id").as[String]
      //  (response.json \ "id").get.toString
      // But, didn't want to get just the id, anyway.
      */
      response.json.toString()
    }
    let url = "http://" + mRemoteAddress + "/entities/" + id_in + "/overview";
    RestDatabase.restCallWithOptionalErrorHandling[String, Any](url, getEntity_processed, None, Array(), uiIn)
  }

  override fn getGroupSize(groupIdIn: i64, includeWhichEntitiesIn: Int = 3) -> i64 {
    getLong("/groups/" + groupIdIn + "/size/" + includeWhichEntitiesIn)
  }

  override fn findUnusedGroupSortingIndex(groupIdIn: i64, startingWithIn: Option<i64>) -> i64 {
    getLong("/groups/" + groupIdIn + "/unusedSortingIndex/" + startingWithIn.getOrElse(""))
  }

  override fn getHighestSortingIndexForGroup(groupIdIn: i64) -> i64 {
    getLong("/groups/" + groupIdIn + "/highestSortingIndex")
  }

  override fn getGroupEntrySortingIndex(groupIdIn: i64, entityIdIn: i64) -> i64 {
    getLong("/groups/" + groupIdIn + "/sortingIndex/" + entityIdIn)
  }

  override fn getEntityAttributeSortingIndex(entityIdIn: i64, attributeFormIdIn: i64, attributeIdIn: i64) -> i64 {
    getLong("/entities/" + entityIdIn + "/sortingIndex/" + attributeFormIdIn + "/" + attributeIdIn)
  }

  override fn getEntitiesOnlyCount(limitByClass: Boolean, classIdIn: Option<i64>, templateEntity: Option<i64>) -> i64 {
    getLong("/entities/entitiesOnlyCount/" + limitByClass +
            (if classIdIn.isEmpty) ""
            else {
              "/" + classIdIn.get + {
                if templateEntity.isEmpty) ""
                else {
                  "/" + templateEntity.get
                }
              }
            }))
  }

  override fn getAttributeCount(entityIdIn: i64, include_archived_entitiesIn: Boolean = false) -> i64 {
    getLong("/entities/" + entityIdIn + "/attributeCount/" + include_archived_entitiesIn)
  }

  override fn getCountOfGroupsContainingEntity(entityIdIn: i64) -> i64 {
    getLong("/entities/" + entityIdIn + "/countOfGroupsContaining")
  }

  override fn getRelationToLocalEntityCount(entityIdIn: i64, include_archived_entitiesIn: Boolean) -> i64 {
    getLong("/entities/" + entityIdIn + "/countOfRelationsToEntity/" + include_archived_entitiesIn)
  }

  override fn getRelationToRemoteEntityCount(entityIdIn: i64) -> i64 {
    getLong("/entities/" + entityIdIn + "/countOfRelationsToRemoteEntity/")
  }

  override fn getRelationToGroupCount(entityIdIn: i64) -> i64 {
    getLong("/entities/" + entityIdIn + "/countOfRelationsToGroup")
  }

  override fn getClassCount(templateEntityIdIn: Option<i64>) -> i64 {
    getLong("/classes/count/" + templateEntityIdIn.getOrElse(""))
  }

  override fn findUnusedAttributeSortingIndex(entityIdIn: i64, startingWithIn: Option<i64>) -> i64 {
    getLong("/entities/" + entityIdIn + "/unusedAttributeSortingIndex/" + startingWithIn.getOrElse(""))
  }

  override fn getGroupCount -> i64 {
    getLong("/groups/count")
  }

  override fn getOmInstanceCount -> i64 {
    getLong("/omInstances/count")
  }

  override fn getRelationTypeCount -> i64 {
    getLong("/relationTypes/count")
  }

  override fn getEntityCount -> i64 {
    getLong("/entities/count")
  }

  override fn isDuplicateClassName(name_in: String, selfIdToIgnoreIn: Option<i64>) -> Boolean {
    let name = UriEncoding.encodePathSegment(name_in, "UTF-8");
    getBoolean("/classes/isDuplicate/" + name + "/" + selfIdToIgnoreIn.getOrElse(""))
  }

  override fn relationToGroupKeyExists(id_in: i64) -> Boolean {
    getBoolean("/relationsToGroup/" + id_in + "/exists")
  }

  override fn isAttributeSortingIndexInUse(entityIdIn: i64, sortingIndexIn: i64) -> Boolean {
    getBoolean("/entities/" + entityIdIn + "/isAttributeSortingIndexInUse/" + sortingIndexIn)
  }

  override fn isGroupEntrySortingIndexInUse(groupIdIn: i64, sortingIndexIn: i64) -> Boolean {
    getBoolean("/groups/" + groupIdIn + "/isEntrySortingIndexInUse/" + sortingIndexIn)
  }

  override fn entity_key_exists(id_in: i64, include_archived: Boolean) -> Boolean {
    getBoolean("/entities/" + id_in + "/exists/" + include_archived)
  }

  override fn  relationTypeKeyExists(id_in: i64) -> Boolean {
    getBoolean("/relationTypes/" + id_in + "/exists")
  }

  override fn omInstanceKeyExists(id_in: String) -> Boolean {
    getBoolean("/omInstances/" + UriEncoding.encodePathSegment(id_in, "UTF-8") + "/exists")
  }

  override fn classKeyExists(id_in: i64) -> Boolean {
    getBoolean("/classes/" + id_in + "/exists")
  }

  override fn attribute_key_exists(formIdIn: i64, id_in: i64) -> Boolean {
    getBoolean("/attributes/" + formIdIn + "/" + id_in + "/exists")
  }

  override fn quantityAttributeKeyExists(id_in: i64) -> Boolean {
    getBoolean("/quantityAttributes/" + id_in + "/exists")
  }

  override fn dateAttributeKeyExists(id_in: i64) -> Boolean {
    getBoolean("/dateAttributes/" + id_in + "/exists")
  }

  override fn booleanAttributeKeyExists(id_in: i64) -> Boolean {
    getBoolean("/booleanAttributes/" + id_in + "/exists")
  }

  override fn fileAttributeKeyExists(id_in: i64) -> Boolean {
    getBoolean("/fileAttributes/" + id_in + "/exists")
  }

  override fn textAttributeKeyExists(id_in: i64) -> Boolean {
    getBoolean("/textAttributes/" + id_in + "/exists")
  }

  override fn relationToLocalEntityKeysExistAndMatch(id_in: i64, relationTypeIdIn: i64, entityId1In: i64, entityId2In: i64) -> Boolean {
    getBoolean("/relationsToEntity/" + id_in + "/existsWith/" + relationTypeIdIn + "/" + entityId1In + "/" + entityId2In)
  }

  override fn  relationToLocalentity_key_exists(id_in: i64) -> Boolean {
    getBoolean("/relationsToEntity/" + id_in + "/exists")
  }

  override fn  relationToRemoteentity_key_exists(id_in: i64) -> Boolean {
    getBoolean("/relationsToRemoteEntity/" + id_in + "/exists")
  }

  override fn  relationToRemoteEntityKeysExistAndMatch(id_in: i64, relationTypeIdIn: i64, entityId1In: i64, remoteInstanceIdIn: String, entityId2In: i64) -> Boolean {
    getBoolean("/relationsToRemoteEntity/" + id_in + "/existsWith/" + relationTypeIdIn + "/" + entityId1In + "/" +
               UriEncoding.encodePathSegment(remoteInstanceIdIn, "UTF-8") + "/" + entityId2In)
  }

  override fn relationToGroupKeysExistAndMatch(id: i64, entityId: i64, relationTypeId: i64, groupId: i64) -> Boolean {
    getBoolean("/relationsToGroup/" + id + "/existsWith/" + entityId + "/" + relationTypeId + "/" + groupId)
  }

  override  fn groupKeyExists(id_in: i64) -> Boolean {
    getBoolean("/groups/" + id_in + "/exists")
  }

  override fn isDuplicateEntityName(name_in: String, selfIdToIgnoreIn: Option<i64>) -> Boolean {
    //If we need to change the 2nd parameter from UTF-8 to something else below, see javadocs for a class about encode/encoding, IIRC.
    let name = UriEncoding.encodePathSegment(name_in, "UTF-8");
    getBoolean("/entities/isDuplicate/" + name + "/" + selfIdToIgnoreIn.getOrElse(""))
  }

  override fn isDuplicateOmInstanceAddress(addressIn: String, selfIdToIgnoreIn: Option<String>) -> Boolean {
    getBoolean("/omInstances/isDuplicate/" + UriEncoding.encodePathSegment(addressIn, "UTF-8") + "/" +
               UriEncoding.encodePathSegment(selfIdToIgnoreIn.getOrElse(""), "UTF-8"))
  }

  override fn isEntityInGroup(groupIdIn: i64, entityIdIn: i64) -> Boolean {
    getBoolean("/groups/" + groupIdIn + "/containsEntity/" + entityIdIn)
  }

  override fn include_archived_entities -> Boolean {
    getBoolean("/entities/include_archived")
  }

  override fn getClassName(id_in: i64) -> Option<String> {
    getOptionString("/classes/" + id_in + "/name")
  }

  override fn  getEntityName(id_in: i64) -> Option<String> {
    getOptionString("/entities/" + id_in + "/name")
  }

  override fn getNearestGroupEntrysSortingIndex(groupIdIn: i64, startingPointSortingIndexIn: i64, forwardNotBackIn: Boolean) -> Option<i64> {
    getOptionLongFromRest("/groups/" + groupIdIn + "/nearestEntrysSortingIndex/" + startingPointSortingIndexIn + "/" + forwardNotBackIn)
  }

  override fn getNearestAttributeEntrysSortingIndex(entityIdIn: i64, startingPointSortingIndexIn: i64, forwardNotBackIn: Boolean) -> Option<i64> {
    getOptionLongFromRest("/entities/" + entityIdIn + "/nearestAttributeSortingIndex/" + startingPointSortingIndexIn + "/" + forwardNotBackIn)
  }

  override fn getClassData(id_in: i64) -> Array[Option[Any]] {
    getArrayOptionAny("/classes/" + id_in, Array(Database.GET_CLASS_DATA__RESULT_TYPES))
  }

  override fn  getRelationTypeData(id_in: i64) -> Array[Option[Any]] {
    getArrayOptionAny("/relationTypes/" + id_in, Array(Database.GET_RELATION_TYPE_DATA__RESULT_TYPES))
  }

  override fn getOmInstanceData(id_in: String) -> Array[Option[Any]] {
    let id = UriEncoding.encodePathSegment(id_in, "UTF-8");
    getArrayOptionAny("/omInstances/" + id, Array(Database.GET_OM_INSTANCE_DATA__RESULT_TYPES))
  }

  override fn  getFileAttributeData(id_in: i64) -> Array[Option[Any]] {
    getArrayOptionAny("/fileAttributes/" + id_in, Array(Database.GET_FILE_ATTRIBUTE_DATA__RESULT_TYPES))
  }

  override fn  getTextAttributeData(id_in: i64) -> Array[Option[Any]] {
    getArrayOptionAny("/textAttributes/" + id_in, Array(Database.GET_TEXT_ATTRIBUTE_DATA__RESULT_TYPES))
  }

  override fn  getQuantityAttributeData(id_in: i64) -> Array[Option[Any]] {
    getArrayOptionAny("/quantityAttributes/" + id_in, Array(Database.GET_QUANTITY_ATTRIBUTE_DATA__RESULT_TYPES))
  }

  override fn  getRelationToGroupData(id_in: i64) -> Array[Option[Any]] {
    getArrayOptionAny("/relationsToGroup/" + id_in, Array(Database.GET_RELATION_TO_GROUP_DATA_BY_ID__RESULT_TYPES))
  }

  override fn getRelationToGroupDataByKeys(entityId: i64, relationTypeId: i64, groupId: i64) -> Array[Option[Any]] {
    getArrayOptionAny("/relationsToGroup/byKeys/" + entityId + "/" + relationTypeId + "/" + groupId, Array(Database.GET_RELATION_TO_GROUP_DATA_BY_KEYS__RESULT_TYPES))
  }

  override fn  getGroupData(id_in: i64) -> Array[Option[Any]] {
    getArrayOptionAny("/groups/" + id_in, Array(Database.GET_GROUP_DATA__RESULT_TYPES))
  }

  override fn getDateAttributeData(id_in: i64) -> Array[Option[Any]] {
    getArrayOptionAny("/dateAttributes/" + id_in, Array(Database.GET_DATE_ATTRIBUTE_DATA__RESULT_TYPES))
  }

  override  fn getBooleanAttributeData(id_in: i64) -> Array[Option[Any]] {
    getArrayOptionAny("/booleanAttributes/" + id_in, Array(Database.GET_BOOLEAN_ATTRIBUTE_DATA__RESULT_TYPES))
  }

  override fn  getRelationToLocalEntityData(relationTypeIdIn: i64, entityId1In: i64, entityId2In: i64) -> Array[Option[Any]] {
    getArrayOptionAny("/relationsToEntity/" + relationTypeIdIn + "/" + entityId1In + "/" + entityId2In, Array(Database.GET_RELATION_TO_LOCAL_ENTITY__RESULT_TYPES))
  }

  override fn  getRelationToRemoteEntityData(relationTypeIdIn: i64, entityId1In: i64, remoteInstanceIdIn: String, entityId2In: i64) -> Array[Option[Any]] {
    getArrayOptionAny("/relationsToRemoteEntity/" + relationTypeIdIn + "/" + entityId1In + "/" +
                      UriEncoding.encodePathSegment(remoteInstanceIdIn, "UTF-8") + "/" + entityId2In,
                      Array(Database.GET_RELATION_TO_REMOTE_ENTITY__RESULT_TYPES))
  }

  override fn  getEntityData(id_in: i64) -> Array[Option[Any]] {
    getArrayOptionAny("/entities/" + id_in, Array(Database.GET_ENTITY_DATA__RESULT_TYPES))
  }

  override fn  getAdjacentGroupEntriesSortingIndexes(groupIdIn: i64, adjacentToEntrySortingIndexIn: i64, limitIn: Option<i64>,
                                                     forwardNotBackIn: Boolean) -> List[Array[Option[Any]]] {
    getListArrayOptionAny("/groups/" + groupIdIn + "/adjacentEntriesSortingIndexes/" + adjacentToEntrySortingIndexIn + "/" + forwardNotBackIn +
                          (if limitIn.isEmpty) "" else "?limit=" + limitIn.get),
                          Array("i64"))
  }

  //Idea: simplify return type of things like this so it is more consumer-friendly, unless it is more friendly to be like the other code already is (ie,
  // like now). Some
  //of the other methods return less generic structures and they are more work to consume in this class because they are different/nonstandard so more
  //methods needed to handle each kind.
  override fn getGroupsContainingEntitysGroupsIds(groupIdIn: i64, limitIn: Option<i64>) -> List[Array[Option[Any]]] {
    getListArrayOptionAny("/groups/" + groupIdIn + "/containingEntitysGroupsIds" + (if limitIn.isEmpty) "" else "?limit=" + limitIn.get), Array("i64"))
  }

  override fn getGroupEntriesData(groupIdIn: i64, limitIn: Option<i64>, include_archived_entitiesIn: Boolean) -> List[Array[Option[Any]]] {
    getListArrayOptionAny("/groups/" + groupIdIn + "/entriesData/" + include_archived_entitiesIn + (if limitIn.isEmpty) "" else "?limit=" + limitIn.get),
                          Array(Database.GET_GROUP_ENTRIES_DATA__RESULT_TYPES))
  }

  override fn getAdjacentAttributesSortingIndexes(entityIdIn: i64, sortingIndexIn: i64, limitIn: Option<i64>,
                                                   forwardNotBackIn: Boolean) -> List[Array[Option[Any]]] {
    getListArrayOptionAny("/entities/" + entityIdIn + "/adjacentAttributesSortingIndexes/" + sortingIndexIn + "/" + forwardNotBackIn +
                          (if limitIn.isEmpty) "" else "?limit=" + limitIn.get),
                          Array("i64"))
  }

    fn createTextAttributeRow(values: Seq[JsValue]) -> TextAttribute {
    new TextAttribute(this, values(0).asInstanceOf[JsNumber].as[i64], values(1).asInstanceOf[JsNumber].as[i64],
                      values(2).asInstanceOf[JsNumber].as[i64],
                      values(3).asInstanceOf[JsString].as[String],
                      if values(4) == JsNull) None else Some(values(4).asInstanceOf[JsNumber].as[i64]),
                      values(5).asInstanceOf[JsNumber].as[i64],
                      values(6).asInstanceOf[JsNumber].as[i64])
  }

  override fn getTextAttributeByTypeId(parentEntityIdIn: i64, typeIdIn: i64, expectedRows: Option[Int]) -> java.util.ArrayList[TextAttribute] {
    getCollection[TextAttribute]("/entities/" + parentEntityIdIn + "/textAttributeByTypeId/" + typeIdIn +
                                 (if expectedRows.isEmpty) "" else "?expectedRows=" + expectedRows.get),
                                 Array(), Some(createTextAttributeRow))
  }

    fn createLongValueRow(values: Seq[JsValue]) -> i64 {
    values(0).asInstanceOf[JsNumber].as[i64]
  }

    fn createStringValueRow(values: Seq[JsValue]) -> String {
    values(0).asInstanceOf[JsString].as[String]
  }

    fn createLongStringLongRow(values: Seq[JsValue]) -> (i64, String, i64) {
    (values(0).asInstanceOf[JsNumber].as[i64], values(1).asInstanceOf[JsString].as[String], values(2).asInstanceOf[JsNumber].as[i64])
  }

  override fn findContainedLocalEntityIds(resultsInOut: mutable.TreeSet[i64], fromEntityIdIn: i64, searchStringIn: String, levelsRemaining: Int,
                                           stopAfterAnyFound: Boolean) -> mutable.TreeSet[i64] {
    let searchString = UriEncoding.encodePathSegment(searchStringIn, "UTF-8");
    let results: util.ArrayList[i64] = getCollection[i64]("/entities/" + fromEntityIdIn + "/findContainedIds/" + searchString +;
                                                            "/" + levelsRemaining + "/" + stopAfterAnyFound, Array(), Some(createLongValueRow))
    // then convert to the needed type:
    let treeSetResults: mutable.TreeSet[i64] = mutable.TreeSet[i64]();
    for (result: i64 <- results) {
      treeSetResults.add(result)
    }
    treeSetResults
  }

  override fn findAllEntityIdsByName(name_in: String, caseSensitive: Boolean) -> java.util.ArrayList[i64] {
    let name = UriEncoding.encodePathSegment(name_in, "UTF-8");
    getCollection[i64]("/entities/findAllByName/" + name + "/" + caseSensitive, Array(), Some(createLongValueRow))
  }

  override fn getContainingGroupsIds(entityIdIn: i64) -> java.util.ArrayList[i64] {
    getCollection[i64]("/entities/" + entityIdIn + "/containingGroupsIds", Array(), Some(createLongValueRow))
  }

  override fn getContainingRelationToGroupDescriptions(entityIdIn: i64, limitIn: Option<i64>) -> ArrayList[String] {
    getCollection[String]("/entities/" + entityIdIn + "/containingRelationsToGroupDescriptions" +
                          (if limitIn.isEmpty) "" else "?limit=" + limitIn.get),
                          Array(), Some(createStringValueRow))
  }

    fn createRelationToGroupRow(values: Seq[JsValue]) -> RelationToGroup {
    new RelationToGroup(this, values(0).asInstanceOf[JsNumber].as[i64], values(1).asInstanceOf[JsNumber].as[i64],
                        values(2).asInstanceOf[JsNumber].as[i64],
                        values(3).asInstanceOf[JsNumber].as[i64],
                        if values(4) == JsNull) None else Some(values(4).asInstanceOf[JsNumber].as[i64]),
                        values(5).asInstanceOf[JsNumber].as[i64],
                        values(6).asInstanceOf[JsNumber].as[i64])
  }

  override fn getContainingRelationsToGroup(entityIdIn: i64, startingIndexIn: i64, limitIn: Option<i64>) -> ArrayList[RelationToGroup] {
    // (The 2nd parameter has to match the types in the 2nd (1st alternate) constructor for RelationToGroup.  Consider putting it in a constant like
    // Database.GET_CLASS_DATA__RESULT_TYPES etc.)
    getCollection[RelationToGroup]("/entities/" + entityIdIn + "/containingRelationsToGroup/" + startingIndexIn +
                                   (if limitIn.isEmpty) "" else "?limit=" + limitIn.get),
                                   Array(),
                                   Some(createRelationToGroupRow))
  }

  override fn getRelationsToGroupContainingThisGroup(groupIdIn: i64, startingIndexIn: i64, maxValsIn: Option<i64>) -> ArrayList[RelationToGroup] {
    getCollection[RelationToGroup]("/groups/" + groupIdIn + "/relationsToGroupContainingThisGroup/" + startingIndexIn +
                                   (if maxValsIn.isEmpty) "" else "?maxVals=" + maxValsIn.get),
                                   Array(),
                                   Some(createRelationToGroupRow))
  }

  override fn findJournalEntries(startTimeIn: i64, endTimeIn: i64, limitIn: Option<i64>) -> ArrayList[(i64, String, i64)] {
    getCollection[(i64, String, i64)]("/entities/addedAndArchivedByDate/" + startTimeIn + "/" + endTimeIn +
                                        (if limitIn.isEmpty) "" else "?limit=" + limitIn.get),
                                        Array(),
                                        Some(createLongStringLongRow))
  }

  override fn findRelationType(type_name_in: String, expectedRows: Option[Int]) -> ArrayList[i64] {
    getCollection[i64]("/relationTypes/find/" + UriEncoding.encodePathSegment(type_name_in, "UTF-8") +
                        (if expectedRows.isEmpty) "" else "?expectedRows=" + expectedRows.get),
                        Array(), Some(createLongValueRow))
  }

  // idea: make private all methods used for the same purpose like this one:
    fn createEntityRow(values: Seq[JsValue]) -> Entity {
    new Entity(this, values(0).asInstanceOf[JsNumber].as[i64],
               values(1).asInstanceOf[JsString].as[String],
               if values(2) == JsNull) None else Some(values(2).asInstanceOf[JsNumber].as[i64]),
               values(3).asInstanceOf[JsNumber].as[i64],
               if values(4) == JsNull) None else Some(values(4).asInstanceOf[JsBoolean].as[Boolean]),
               values(5).asInstanceOf[JsBoolean].as[Boolean],
               values(6).asInstanceOf[JsBoolean].as[Boolean])
  }

    fn createGroupRow(values: Seq[JsValue]) -> Group {
    new Group(this, values(0).asInstanceOf[JsNumber].as[i64],
              values(1).asInstanceOf[JsString].as[String],
              values(2).asInstanceOf[JsNumber].as[i64],
              values(3).asInstanceOf[JsBoolean].as[Boolean],
              values(4).asInstanceOf[JsBoolean].as[Boolean])
  }

    fn createEntityClassRow(values: Seq[JsValue]) -> EntityClass {
    new EntityClass(this, values(0).asInstanceOf[JsNumber].as[i64],
                    values(1).asInstanceOf[JsString].as[String],
                    values(2).asInstanceOf[JsNumber].as[i64],
                    if values(3) == JsNull) None else Some(values(3).asInstanceOf[JsBoolean].as[Boolean]))
  }

  override fn getGroupEntryObjects(groupIdIn: i64, startingObjectIndexIn: i64, maxValsIn: Option<i64>) -> ArrayList[Entity] {
    getCollection[Entity]("/groups/" + groupIdIn + "/entries/" + startingObjectIndexIn +
                          (if maxValsIn.isEmpty) "" else "?maxVals=" + maxValsIn.get),
                          Array(), Some(createEntityRow))
  }

  override fn getEntitiesOnly(startingObjectIndexIn: i64, maxValsIn: Option<i64>, classIdIn: Option<i64>,
                               limitByClass: Boolean, templateEntityIn: Option<i64>, groupToOmitIdIn: Option<i64>) -> util.ArrayList[Entity] {
    let url = "/entities/" + startingObjectIndexIn + "/" + limitByClass +;
              (if maxValsIn.is_defined || classIdIn.is_defined || templateEntityIn.is_defined || groupToOmitIdIn.is_defined) "?" else "") +
              (if maxValsIn.isEmpty) "" else "maxVals=" + maxValsIn.get + "&") +
              (if classIdIn.isEmpty) "" else "classId=" + classIdIn.get + "&") +
              (if templateEntityIn.isEmpty) "" else "templateEntity=" + templateEntityIn.get + "&") +
              (if groupToOmitIdIn.isEmpty) "" else "groupToOmitId=" + groupToOmitIdIn.get + "&")
    getCollection[Entity](url, Array(), Some(createEntityRow))
  }

  override fn getEntities(startingObjectIndexIn: i64, maxValsIn: Option<i64>) -> util.ArrayList[Entity] {
    let url: String = "/entities/all/" + startingObjectIndexIn +;
                      (if maxValsIn.isEmpty) "" else "?maxVals=" + maxValsIn.get)
    getCollection[Entity](url, Array(), Some(createEntityRow))
  }

  override fn getMatchingEntities(startingObjectIndexIn: i64, maxValsIn: Option<i64>, omitEntityIdIn: Option<i64>,
                                   nameRegexIn: String) -> util.ArrayList[Entity] {
    let nameRegex = UriEncoding.encodePathSegment(nameRegexIn, "UTF-8");
    let url: String = "/entities/search/" + nameRegex + "/" + startingObjectIndexIn +;
                      (if maxValsIn.is_defined || omitEntityIdIn.is_defined) "?" else "") +
                      (if maxValsIn.isEmpty) "" else "maxVals=" + maxValsIn.get + "&") +
                      (if omitEntityIdIn.isEmpty) "" else "omitEntityId=" + omitEntityIdIn.get + "&")
    getCollection[Entity](url, Array(), Some(createEntityRow))
  }

  override fn getMatchingGroups(startingObjectIndexIn: i64, maxValsIn: Option<i64>, omitGroupIdIn: Option<i64>,
                                 nameRegexIn: String) -> util.ArrayList[Group] {
    getCollection[Group]("/groups/search/" + UriEncoding.encodePathSegment(nameRegexIn, "UTF-8") + "/" + startingObjectIndexIn +
                         (if maxValsIn.is_defined || omitGroupIdIn.is_defined) "?" else "") +
                         (if maxValsIn.isEmpty) "" else "maxVals=" + maxValsIn.get + "&") +
                         (if omitGroupIdIn.isEmpty) "" else "omitGroupId=" + omitGroupIdIn.get + "&"),
                         Array(), Some(createGroupRow))
  }

  override fn getRelationTypes(startingObjectIndexIn: i64, maxValsIn: Option<i64>) -> util.ArrayList[Entity] {
    let url = "/relationTypes/all/" + startingObjectIndexIn +;
              (if maxValsIn.isEmpty) "" else "?maxVals=" + maxValsIn.get)
    getCollection[RelationType](url, Array(), Some(createRelationTypeRow)).asInstanceOf[util.ArrayList[Entity]]
  }

  override fn getClasses(startingObjectIndexIn: i64, maxValsIn: Option<i64>) -> util.ArrayList[EntityClass] {
    let url = "/classes/all/" + startingObjectIndexIn +;
              (if maxValsIn.isEmpty) "" else "?maxVals=" + maxValsIn.get)
    getCollection[EntityClass](url, Array(), Some(createEntityClassRow))
  }

  override fn getGroups(startingObjectIndexIn: i64, maxValsIn: Option<i64>, groupToOmitIdIn: Option<i64>) -> util.ArrayList[Group] {
    getCollection[Group]("/groups/all/" + startingObjectIndexIn +
                         (if maxValsIn.is_defined || groupToOmitIdIn.is_defined) "?" else "") +
                         (if maxValsIn.isEmpty) "" else "maxVals=" + maxValsIn.get + "&") +
                         (if groupToOmitIdIn.isEmpty) "" else "groupToOmitId=" + groupToOmitIdIn.get + "&"),
                         Array(), Some(createGroupRow))
  }

    fn createRelationTypeIdAndEntityRow(values: Seq[JsValue]) -> (i64, Entity) {
    let entity: Entity = createEntityRow(values);
    let relationTypeId: i64 = values(7).asInstanceOf[JsNumber].as[i64];
    (relationTypeId, entity)
  }

    fn createRelationTypeRow(values: Seq[JsValue]) -> RelationType {
    new RelationType(this, values(0).asInstanceOf[JsNumber].as[i64],
                     values(1).asInstanceOf[JsString].as[String],
                     values(7).asInstanceOf[JsString].as[String],
                     values(8).asInstanceOf[JsString].as[String])
  }

  override fn getEntitiesContainingGroup(groupIdIn: i64, startingIndexIn: i64, maxValsIn: Option<i64>) -> ArrayList[(i64, Entity)] {
    getCollection[(i64, Entity)]("/groups/" + groupIdIn + "/containingEntities/" + startingIndexIn +
                                  (if maxValsIn.isEmpty) "" else "?maxVals=" + maxValsIn.get),
                                  Array(), Some(createRelationTypeIdAndEntityRow))
  }

  override fn getLocalEntitiesContainingLocalEntity(entityIdIn: i64, startingIndexIn: i64, maxValsIn: Option<i64>) -> ArrayList[(i64, Entity)] {
    getCollection[(i64, Entity)]("/entities/" + entityIdIn + "/containingEntities/" + startingIndexIn +
                                  (if maxValsIn.isEmpty) "" else "?maxVals=" + maxValsIn.get),
                                  Array(), Some(createRelationTypeIdAndEntityRow))
  }

    fn process2Longs(response: WSResponse, ignore: Option[(Seq[JsValue]) => Any], ignore2: Array[Any]) -> (i64, i64) {
    if response.json == JsNull) {
      throw new OmDatabaseException("Unexpected: null result in the REST response (basically the remote side saying \"found nothing\".")
    } else {
      let values: IndexedSeq[JsValue] = response.json.asInstanceOf[JsObject].values.toIndexedSeq;
      let first: i64 = values(0).asInstanceOf[JsNumber].as[i64];
      let second: i64 = values(1).asInstanceOf[JsNumber].as[i64];
      (first, second)
    }
  }

    fn get2Longs(pathIn: String) -> (i64, i64) {
    RestDatabase.restCall[(i64, i64), Any]("http://" + mRemoteAddress + pathIn, process2Longs, None, Array())
  }

  override fn getCountOfEntitiesContainingGroup(groupIdIn: i64) -> (i64, i64) {
    get2Longs("/groups/" + groupIdIn + "/countOfContainingEntities")
  }

  override fn getCountOfLocalEntitiesContainingLocalEntity(entityIdIn: i64) -> (i64, i64) {
    get2Longs("/entities/" + entityIdIn + "/countOfContainingEntities")
  }

  override fn getFileAttributeContent(fileAttributeIdIn: i64, outputStreamIn: OutputStream) -> (i64, String) {
    // (Idea: should this (and others) instead just call something that returns a complete FileAttribute, so that multiple places in the code do
    // not all have to know the indexes for each datum?:)
    let faData = getFileAttributeData(fileAttributeIdIn);
    let fileSize = faData(9).get.asInstanceOf[i64];
    let md5hash = faData(10).get.asInstanceOf[String];
    let url = new URL("http://" + mRemoteAddress + "/fileAttributes/" + fileAttributeIdIn + "/content");
    let mut input: InputStream = null;
    try {
      input = url.openStream()
      // see mention of 4096 elsewhere for why that # was chosen
      let b = new Array[Byte](4096);
      @tailrec fn stream() {
        //Idea, also tracked in tasks: put at least next line or surrounding, on a separate thread or w/ a future, so it can use a timeout & not block forever:
        let length = input.read(b);
        if length != -1) {
          outputStreamIn.write(b, 0, length)
          stream()
        }
      }
      stream()
    } finally {
      if input != null) input.close()
    }
    (fileSize, md5hash)
  }

    fn processOptionLongsStringBoolean(response: WSResponse, ignore: Option[(Seq[JsValue]) => Any],
                                      ignore2: Array[Any]): (Option<i64>, Option<i64>, Option<i64>, Option<String>, Boolean) {
    if response.json == JsNull) {
      throw new OmDatabaseException("Unexpected: null result in the REST response (basically the remote side saying \"found nothing\".")
    } else {
      let values: IndexedSeq[JsValue] = response.json.asInstanceOf[JsObject].values.toIndexedSeq;
      let first: Option<i64> = getOptionLongFromJson(values, 0);
      let second: Option<i64> = getOptionLongFromJson(values, 1);
      let third: Option<i64> = getOptionLongFromJson(values, 2);
      let fourth: Option<String> = getOptionStringFromJson(values, 3);
      let last: bool = values(4).asInstanceOf[JsBoolean].as[Boolean];
      (first, second, third, fourth, last)
    }
  }

    fn getOptionLongsStringBoolean(pathIn: String): (Option<i64>, Option<i64>, Option<i64>, Option<String>, Boolean) {
    RestDatabase.restCall[(Option<i64>, Option<i64>, Option<i64>, Option<String>, Boolean), Any]("http://" + mRemoteAddress + pathIn,
                                                                                                    processOptionLongsStringBoolean, None, Array())
  }

  override fn findRelationToAndGroup_OnEntity(entityIdIn: i64,
                                               groupNameIn: Option<String>): (Option<i64>, Option<i64>, Option<i64>, Option<String>, Boolean) {
    getOptionLongsStringBoolean("/entities/" + entityIdIn + "/findRelationToAndGroup" +
                                (if groupNameIn.isEmpty) "" else "?groupName=" + java.net.URLEncoder.encode(groupNameIn.get, "UTF-8")))
    // Note: using a different kind of encoder/encoding for a query part of a URI (vs. the path, as elsewhere), per info at:
    //   https://www.playframework.com/documentation/2.5.x/api/scala/index.html#play.utils.UriEncoding$
    // ...which says:
    /*"Encode a string so that it can be used safely in the "path segment" part of a URI. A path segment is defined in RFC 3986. In a URI such as http://www
    .example.com/abc/def?a=1&b=2 both abc and def are path segments.
    Path segment encoding differs from encoding for other parts of a URI. For example, the "&" character is permitted in a path segment, but has special
    meaning in query parameters. On the other hand, the "/" character cannot appear in a path segment, as it is the path delimiter, so it must be encoded as
    "%2F". These are just two examples of the differences between path segment and query string encoding; there are other differences too.
    When encoding path segments the encodePathSegment method should always be used in preference to the java.net.URLEncoder.encode method. URLEncoder.encode,
     despite its name, actually provides encoding in the application/x-www-form-urlencoded MIME format which is the encoding used for form data in HTTP GET
     and POST requests. This encoding is suitable for inclusion in the query part of a URI. But URLEncoder.encode should not be used for path segment
     encoding. (Also note that URLEncoder.encode is not quite spec compliant. For example, it percent-encodes the ~ character when really it should leave it
     as unencoded.)"
    */
  }

    fn getOptionLongFromJson(values: IndexedSeq[JsValue], index: Int) -> Option<i64> {
    if values(index) == JsNull) None
    else {
      Some(values(index).asInstanceOf[JsNumber].as[i64])
      // Idea: learn why in some places this needed instead: is there a difference in the way it is sent from the web module? or do both work?:
      // Some(response.json.as[i64])
    }
  }

    fn getOptionStringFromJson(values: IndexedSeq[JsValue], index: Int) -> Option<String> {
    if values(index) == JsNull) None
    else {
      Some(values(index).asInstanceOf[JsString].as[String])
      // Idea: learn why in some places this needed instead: is there a difference in the way it is sent from the web module? or do both work?:
      // Some(response.json.as[i64])
    }
  }

    fn processSortedAttributes(response: WSResponse, ignore: Option[(Seq[JsValue]) => Any], ignore2: Array[Any]) -> (Array[(i64, Attribute)], Int) {
    if response.json == JsNull) {
      throw new OmDatabaseException("Unexpected: null result in the REST response (basically the remote side saying \"found nothing\".")
    } else {
      let arrayAndInt = response.json.asInstanceOf[JsObject].values.toIndexedSeq;
      let totalAttributesAvailable: i32 = arrayAndInt(0).asInstanceOf[JsNumber].as[Int];
      let attributesRetrieved: JsArray = arrayAndInt(1).asInstanceOf[JsArray];
      let resultsAccumulator = new ArrayList[(i64, Attribute)](totalAttributesAvailable);
      for (attributeJson <- attributesRetrieved.value) {
        let values: IndexedSeq[JsValue] = attributeJson.asInstanceOf[JsObject].values.toIndexedSeq;
        let id: i64 = values(0).asInstanceOf[JsNumber].as[i64];
        let formId: i64 = values(1).asInstanceOf[JsNumber].as[i64];
        let parentId: i64 = values(2).asInstanceOf[JsNumber].as[i64];
        let attributeTypeId: i64 = values(3).asInstanceOf[JsNumber].as[i64];
        let sortingIndex: i64 = values(4).asInstanceOf[JsNumber].as[i64];
        let attribute: Attribute = formId match {;
          case 1 =>
            let valid_on_date = getOptionLongFromJson(values, 5);
            let observationDate: i64 = values(6).asInstanceOf[JsNumber].as[i64];
            let unitId: i64 = values(7).asInstanceOf[JsNumber].as[i64];
            let number: Float = values(8).asInstanceOf[JsNumber].as[Float];
            new QuantityAttribute(this, id, parentId, attributeTypeId, unitId, number, valid_on_date, observationDate, sortingIndex)
          case 2 =>
            let date: i64 = values(5).asInstanceOf[JsNumber].as[i64];
            new DateAttribute(this, id, parentId, attributeTypeId, date, sortingIndex)
          case 3 =>
            let valid_on_date = getOptionLongFromJson(values, 5);
            let observationDate: i64 = values(6).asInstanceOf[JsNumber].as[i64];
            let bool: bool = values(7).asInstanceOf[JsBoolean].as[Boolean];
            new BooleanAttribute(this, id, parentId, attributeTypeId, bool, valid_on_date, observationDate, sortingIndex)
          case 4 =>
            let description = values(5).asInstanceOf[JsString].as[String];
            let originalFileDate = values(6).asInstanceOf[JsNumber].as[i64];
            let storedDate = values(7).asInstanceOf[JsNumber].as[i64];
            let original_file_path = values(8).asInstanceOf[JsString].as[String];
            let readable: bool = values(9).asInstanceOf[JsBoolean].as[Boolean];
            let writable: bool = values(10).asInstanceOf[JsBoolean].as[Boolean];
            let executable: bool = values(11).asInstanceOf[JsBoolean].as[Boolean];
            let size = values(12).asInstanceOf[JsNumber].as[i64];
            let md5hash = values(13).asInstanceOf[JsString].as[String];
            new FileAttribute(this, id, parentId, attributeTypeId, description, originalFileDate, storedDate, original_file_path, readable, writable,
                              executable, size, md5hash, sortingIndex)
          case 5 =>
            let valid_on_date = getOptionLongFromJson(values, 5);
            let observationDate: i64 = values(6).asInstanceOf[JsNumber].as[i64];
            let textEscaped = values(7).asInstanceOf[JsString].as[String];
            let text = org.apache.commons.lang3.StringEscapeUtils.unescapeJson(textEscaped);
            new TextAttribute(this, id, parentId, attributeTypeId, text, valid_on_date, observationDate, sortingIndex)
          case 6 =>
            let valid_on_date = getOptionLongFromJson(values, 5);
            let observationDate: i64 = values(6).asInstanceOf[JsNumber].as[i64];
            let entityId1: i64 = values(7).asInstanceOf[JsNumber].as[i64];
            let entityId2: i64 = values(8).asInstanceOf[JsNumber].as[i64];
            new RelationToLocalEntity(this, id, attributeTypeId, entityId1, entityId2, valid_on_date, observationDate, sortingIndex)
          case 7 =>
            let valid_on_date = getOptionLongFromJson(values, 5);
            let observationDate: i64 = values(6).asInstanceOf[JsNumber].as[i64];
            let entityId: i64 = values(7).asInstanceOf[JsNumber].as[i64];
            let groupId: i64 = values(8).asInstanceOf[JsNumber].as[i64];
            new RelationToGroup(this, id, entityId, attributeTypeId, groupId, valid_on_date, observationDate, sortingIndex)
          case 8 =>
            let valid_on_date = getOptionLongFromJson(values, 5);
            let observationDate: i64 = values(6).asInstanceOf[JsNumber].as[i64];
            let entityId1: i64 = values(7).asInstanceOf[JsNumber].as[i64];
            let remoteInstanceId = values(8).asInstanceOf[JsString].as[String];
            let entityId2: i64 = values(9).asInstanceOf[JsNumber].as[i64];
            new RelationToRemoteEntity(this, id, attributeTypeId, entityId1, remoteInstanceId, entityId2, valid_on_date, observationDate, sortingIndex)
          case _ => throw new OmDatabaseException("unexpected formId: " + formId)
        }
        resultsAccumulator.add((sortingIndex, attribute))
      }
      (resultsAccumulator.toArray(new Array[(i64, Attribute)](0)), totalAttributesAvailable)
    }
  }

  override fn getSortedAttributes(entityIdIn: i64, startingObjectIndexIn: Int, maxValsIn: Int,
                                   onlyPublicEntitiesIn: Boolean) -> (Array[(i64, Attribute)], Int) {
    let path: String = "/entities/" + entityIdIn + "/sortedAttributes/" + startingObjectIndexIn + "/" + maxValsIn + "/" + onlyPublicEntitiesIn;
    RestDatabase.restCall[(Array[(i64, Attribute)], Int), Any]("http://" + mRemoteAddress + path, processSortedAttributes, None, Array())
  }

    //%%???:
    fn getCountOfEntitiesUsedAsAttributeTypes(objectTypeIn: String, quantitySeeksUnitNotTypeIn: Boolean): i64 = ???
    fn getEntitiesUsedAsAttributeTypes(objectTypeIn: String, startingObjectIndexIn: i64, maxValsIn: Option<i64> = None,
                                      quantitySeeksUnitNotTypeIn: Boolean): Vec<Entity> = ???


  // Below are methods that WRITE to the DATABASE.
  //
  // Things were generated with "override" by the IDE, but after some reading, it seems not worth the bother to always type it.
  //
  // When implementing later, REMEMBER TO MAKE READONLY OR SECURE (only showing public or allowed data),
  // OR HANDLE THEIR LACK, IN THE UI IN A FRIENDLY WAY.

  //idea: when implementing these, first sort by CRUD groupings, and/or by return type, to group similar things for ease?:
  override fn begin_trans(): Unit = ???

  override fn rollback_trans(): Unit = ???

  override fn commit_trans(): Unit = ???

  override fn moveRelationToGroup(relationToGroupIdIn: i64, newContainingEntityIdIn: i64, sortingIndexIn: i64): i64 = ???

  override fn updateRelationToRemoteEntity(oldRelationTypeIdIn: i64, entityId1In: i64, remoteInstanceIdIn: String, entityId2In: i64,
                                            newRelationTypeIdIn: i64, valid_on_date_in: Option<i64>, observationDateIn: i64): Unit = ???

  override fn unarchiveEntity(id_in: i64, callerManagesTransactionsIn: Boolean): Unit = ???

  override fn  set_include_archived_entities(in: Boolean): Unit = ???

  override fn  createOmInstance(id_in: String, isLocalIn: Boolean, addressIn: String, entityIdIn: Option<i64>, oldTableName: Boolean): i64 = ???

  override fn  deleteOmInstance(id_in: String): Unit = ???

  override fn  deleteDateAttribute(id_in: i64): Unit = ???

  override fn  updateDateAttribute(id_in: i64, parentIdIn: i64, date_in: i64, attrTypeIdIn: i64): Unit = ???

  override fn updateRelationToGroup(entityIdIn: i64, oldRelationTypeIdIn: i64, newRelationTypeIdIn: i64, oldGroupIdIn: i64, newGroupIdIn: i64,
                                     valid_on_date_in: Option<i64>, observationDateIn: i64): Unit = ???

  override  fn archiveEntity(id_in: i64, callerManagesTransactionsIn: Boolean): Unit = ???

  override fn  moveLocalEntityFromGroupToGroup(fromGroupIdIn: i64, toGroupIdIn: i64, moveEntityIdIn: i64, sortingIndexIn: i64): Unit = ???

  override fn  deleteClassAndItsTemplateEntity(classIdIn: i64): Unit = ???

  override fn  createRelationToLocalEntity(relationTypeIdIn: i64, entityId1In: i64, entityId2In: i64, valid_on_date_in: Option<i64>, observationDateIn: i64,
                                           sortingIndexIn: Option<i64>, callerManagesTransactionsIn: Boolean): RelationToLocalEntity = ???

  override fn deleteRelationToGroup(entityIdIn: i64, relationTypeIdIn: i64, groupIdIn: i64): Unit = ???

  override fn deleteQuantityAttribute(id_in: i64): Unit = ???

  override fn removeEntityFromGroup(groupIdIn: i64, containedEntityIdIn: i64, callerManagesTransactionsIn: Boolean): Unit = ???

  override fn addEntityToGroup(groupIdIn: i64, containedEntityIdIn: i64, sortingIndexIn: Option<i64>, callerManagesTransactionsIn: Boolean): Unit = ???

  override fn deleteRelationToRemoteEntity(relationTypeIdIn: i64, entityId1In: i64, remoteInstanceIdIn: String, entityId2In: i64): Unit = ???

  override fn deleteFileAttribute(id_in: i64): Unit = ???

  override fn  updateFileAttribute(id_in: i64, parentIdIn: i64, attrTypeIdIn: i64, descriptionIn: String): Unit = ???

  override fn  updateFileAttribute(id_in: i64, parentIdIn: i64, attrTypeIdIn: i64, descriptionIn: String, originalFileDateIn: i64, storedDateIn: i64,
                                   original_file_path_in: String, readableIn: Boolean, writableIn: Boolean, executableIn: Boolean, sizeIn: i64,
                                   md5hashIn: String): Unit = ???

  override fn  updateQuantityAttribute(id_in: i64, parentIdIn: i64, attrTypeIdIn: i64, unitIdIn: i64, numberIn: Float, valid_on_date_in: Option<i64>,
                                       inObservationDate: i64): Unit = ???

  override fn  deleteGroupRelationsToItAndItsEntries(groupid_in: i64): Unit = ???

  override fn  updateEntitysClass(entityId: i64, classId: Option<i64>, callerManagesTransactions: Boolean): Unit = ???

  override fn  deleteBooleanAttribute(id_in: i64): Unit = ???

  override fn  moveLocalEntityFromLocalEntityToGroup(removingRtleIn: RelationToLocalEntity, targetGroupIdIn: i64, sortingIndexIn: i64): Unit = ???

  override fn  renumberSortingIndexes(entityIdOrGroupIdIn: i64, callerManagesTransactionsIn: Boolean, isEntityAttrsNotGroupEntries: Boolean): Unit = ???

  override fn  updateEntityOnlyNewEntriesStickToTop(id_in: i64, newEntriesStickToTop: Boolean): Unit = ???

  override fn  createDateAttribute(parentIdIn: i64, attrTypeIdIn: i64, date_in: i64, sortingIndexIn: Option<i64>): i64 = ???

  override fn  createGroupAndRelationToGroup(entityIdIn: i64, relationTypeIdIn: i64, newGroupNameIn: String, allowMixedClassesInGroupIn: Boolean,
                                             valid_on_date_in: Option<i64>, observationDateIn: i64, sortingIndexIn: Option<i64>,
                                             callerManagesTransactionsIn: Boolean): (i64, i64) = ???

  override fn  addHASRelationToLocalEntity(fromEntityIdIn: i64, toEntityIdIn: i64, valid_on_date_in: Option<i64>, observationDateIn: i64,
                                           sortingIndexIn: Option<i64>): RelationToLocalEntity = ???

  override fn  updateRelationToLocalEntity(oldRelationTypeIdIn: i64, entityId1In: i64, entityId2In: i64, newRelationTypeIdIn: i64,
                                           valid_on_date_in: Option<i64>, observationDateIn: i64): Unit = ???

  override fn  updateSortingIndexInAGroup(groupIdIn: i64, entityIdIn: i64, sortingIndexIn: i64): Unit = ???

  override fn  updateAttributeSortingIndex(entityIdIn: i64, attributeFormIdIn: i64, attributeIdIn: i64, sortingIndexIn: i64): Unit = ???

  override fn  updateGroup(groupIdIn: i64, name_in: String, allowMixedClassesInGroupIn: Boolean, newEntriesStickToTopIn: Boolean): Unit = ???

  override fn setUserPreference_EntityId(name_in: String, entityIdIn: i64): Unit = ???

  override fn deleteRelationType(id_in: i64): Unit = ???

  override fn  deleteGroupAndRelationsToIt(id_in: i64): Unit = ???

  override fn  deleteEntity(id_in: i64, callerManagesTransactionsIn: Boolean): Unit = ???

  override fn  moveRelationToLocalEntityToLocalEntity(rtleIdIn: i64, newContainingEntityIdIn: i64,
                                                      sortingIndexIn: i64): RelationToLocalEntity = ???

  //NOTE: when implementing the below method (ie, so there is more supporting code then), also create a test (locally though...?) for RTRE.move.
  // (And while at it, also for RTRE.getEntityForEntityId2 and RTLE.getEntityForEntityId2 ?  Do they get called?)
  override fn  moveRelationToRemoteEntityToLocalEntity(remoteInstanceIdIn: String, relationToRemoteEntityIdIn: i64, toContainingEntityIdIn: i64,
                                                       sortingIndexIn: i64): RelationToRemoteEntity = ???

  override  fn createFileAttribute(parentIdIn: i64, attrTypeIdIn: i64, descriptionIn: String, originalFileDateIn: i64, storedDateIn: i64,
                                   original_file_path_in: String, readableIn: Boolean, writableIn: Boolean, executableIn: Boolean, sizeIn: i64,
                                   md5hashIn: String, inputStreamIn: FileInputStream, sortingIndexIn: Option<i64>): i64 = ???

  override fn deleteTextAttribute(id_in: i64): Unit = ???

  override fn createEntityAndRelationToLocalEntity(entityIdIn: i64, relationTypeIdIn: i64, newEntityNameIn: String, isPublicIn: Option<bool>,
                                                    valid_on_date_in: Option<i64>, observationDateIn: i64,
                                                    callerManagesTransactionsIn: Boolean): (i64, i64) = ???

  override fn moveEntityFromGroupToLocalEntity(fromGroupIdIn: i64, toEntityIdIn: i64, moveEntityIdIn: i64, sortingIndexIn: i64): Unit = ???

  override fn  updateTextAttribute(id_in: i64, parentIdIn: i64, attrTypeIdIn: i64, textIn: String, valid_on_date_in: Option<i64>,
                                   observationDateIn: i64): Unit = ???

  override fn  getOrCreateClassAndTemplateEntity(classNameIn: String, callerManagesTransactionsIn: Boolean): (i64, i64) = ???

  override fn  addUriEntityWithUriAttribute(containingEntityIn: Entity, newEntityNameIn: String, uriIn: String, observationDateIn: i64,
                                            makeThemPublicIn: Option<bool>, callerManagesTransactionsIn: Boolean,
                                            quoteIn: Option<String> = None): (Entity, RelationToLocalEntity) = ???

  override fn  updateEntityOnlyPublicStatus(id_in: i64, value: Option<bool>): Unit = ???

  override fn createRelationToRemoteEntity(relationTypeIdIn: i64, entityId1In: i64, entityId2In: i64, valid_on_date_in: Option<i64>,
                                            observationDateIn: i64, remoteInstanceIdIn: String, sortingIndexIn: Option<i64>,
                                            callerManagesTransactionsIn: Boolean): RelationToRemoteEntity = ???

  override  fn createRelationToGroup(entityIdIn: i64, relationTypeIdIn: i64, groupIdIn: i64, valid_on_date_in: Option<i64>, observationDateIn: i64,
                                     sortingIndexIn: Option<i64>, callerManagesTransactionsIn: Boolean): (i64, i64) = ???

  override  fn  createBooleanAttribute(parentIdIn: i64, attrTypeIdIn: i64, booleanIn: Boolean, valid_on_date_in: Option<i64>, observationDateIn: i64,
                                      sortingIndexIn: Option<i64>): i64 = ???

  override fn createEntity(name_in: String, classIdIn: Option<i64>, isPublicIn: Option<bool>): i64 = ???

  override  fn deleteRelationToLocalEntity(relationTypeIdIn: i64, entityId1In: i64, entityId2In: i64): Unit = ???

  override  fn  updateClassCreateDefaultAttributes(classIdIn: i64, value: Option<bool>) = ???

  override fn updateBooleanAttribute(id_in: i64, parentIdIn: i64, attrTypeIdIn: i64, booleanIn: Boolean,
                                      valid_on_date_in: Option<i64>, inObservationDate: i64): Unit = ???

  override fn createQuantityAttribute(parentIdIn: i64, attrTypeIdIn: i64, unitIdIn: i64, numberIn: Float, valid_on_date_in: Option<i64>,
                                       inObservationDate: i64, callerManagesTransactionsIn: Boolean = false, sortingIndexIn: Option<i64> = None): /*id*/
  i64 = ???

  override  fn  createTextAttribute(parentIdIn: i64, attrTypeIdIn: i64, textIn: String, valid_on_date_in: Option<i64>,
                                   observationDateIn: i64, callerManagesTransactionsIn: Boolean, sortingIndexIn: Option<i64>): i64 = ???

  override fn  createRelationType(name_in: String, name_in_reverseDirectionIn: String, directionalityIn: String): i64 = ???

  override fn  updateRelationType(id_in: i64, name_in: String, name_in_reverseDirectionIn: String, directionalityIn: String): Unit = ???

  override  fn createClassAndItsTemplateEntity(classNameIn: String): (i64, i64) = ???

  override  fn createGroup(name_in: String, allowMixedClassesInGroupIn: Boolean): i64 = ???

  override fn updateEntityOnlyName(id_in: i64, name_in: String): Unit = ???

  override fn updateClassAndTemplateEntityName(classIdIn: i64, name: String): i64 = ???

  override fn updateOmInstance(id_in: String, addressIn: String, entityIdIn: Option<i64>): Unit = ???


  // NOTE: those below, like getUserPreference_Boolean or getPreferencesContainerId, are intentionally unimplemented, not because they are
  // writable as those above, but because there is no known reason to implement them in this class (they are not known to be
  // called when the DB is this type). They are only here to allow OM to
  // compile even though things like Controller (which starts with a local database even though the compiler doesn't enforce that) can have a member "db"
  // which is an abstract Database instead of a specific local database class, which is to help modify the code so that it can refer to either a
  // local or remote database, and in some cases to make it so
  // some methods are not to be called directly against the DB, but via the model package classes, which will themselves decide *which* DB should
  // be accessed (the default local DB, or a determined remote per the model object), so that for example we properly handle the distinction between
  // RelationToLocalEntity vs RelationToRemoteEntity, etc.
  // Idea: improve & shorten that rambling explanation.
  override fn getUserPreference_Boolean(preferenceNameIn: String, default_value_in: Option<bool>): Option<bool> = ???

  override fn getPreferencesContainerId: i64 = ???

  override fn getUserPreference_EntityId(preferenceNameIn: String, default_value_in: Option<i64>): Option<i64> = ???

  override fn getOmInstances(localIn: Option<bool>): util.ArrayList[OmInstance] = ???

    fn getRelationToLocalEntityDataById(id_in: i64): Array[Option[Any]] = ???
*/
}
