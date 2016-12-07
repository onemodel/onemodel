/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2016-2016 inclusive, Luke A. Call; all rights reserved.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule, guidelines around binary
    distribution, and the GNU Affero General Public License as published by the Free Software Foundation, either version 3
    of the License, or (at your option) any later version.  See the file LICENSE for details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
package org.onemodel.core.database

import java.io.{InputStream, FileInputStream, OutputStream}
import java.net.URL
import java.util
import java.util.ArrayList

import akka.actor.ActorSystem
import akka.stream.ActorMaterializer
import org.onemodel.core.model._
import org.onemodel.core.{OmException, OmDatabaseException, TextUI, Util}
import play.api.libs.json._
import play.api.libs.ws.{WSResponse, WSClient}
import play.api.libs.ws.ahc.{AhcWSResponse, AhcWSClient}
import play.utils.UriEncoding
import scala.annotation.tailrec
import scala.collection.immutable.IndexedSeq
import scala.collection.JavaConversions._
import scala.collection.mutable
import scala.concurrent.duration._

import scala.concurrent.{Await, Future}

object RestDatabase {
  // (Details on this REST client system are at:  https://www.playframework.com/documentation/2.5.x/ScalaWS#Directly-creating-WSClient .)
  val timeout: FiniteDuration = 20.seconds
  implicit val actorSystem: ActorSystem = ActorSystem()
  implicit val actorMaterializer: ActorMaterializer = ActorMaterializer()
  lazy val wsClient: WSClient = AhcWSClient()
  implicit val context = play.api.libs.concurrent.Execution.Implicits.defaultContext

  def restCall[T, U](urlIn: String,
                  functionToCall: (WSResponse, Option[(Seq[JsValue]) => U], Array[Any]) => T,
                  functionToCreateResultRow: Option[(Seq[JsValue]) => U],
                  inputs: Array[Any]): T = {
    restCallWithOptionalErrorHandling[T, U](urlIn, functionToCall, functionToCreateResultRow, inputs, None).get
  }

  /**
   * Does error handling internally to the provided UI, only if the parameter uiIn.isDefined (ie, not None), otherwise throws the
   * exception to the caller.  Either returns a Some(data), or shows the exception in the UI then returns None, or throws an exception.
   */
  def restCallWithOptionalErrorHandling[T, U](urlIn: String,
                                           functionToCall: (WSResponse, Option[(Seq[JsValue]) => U], Array[Any]) => T,
                                           functionToCreateResultRow: Option[(Seq[JsValue]) => U],
                                           inputs: Array[Any],
                                           uiIn: Option[TextUI]): Option[T] = {
    var responseText = ""
    try {
      val request = RestDatabase.wsClient.url(urlIn).withFollowRedirects(true)
      val futureResponse: Future[WSResponse] = request.get()
      /* Idea?: Can simplify this based on code example inside the test at
           https://www.playframework.com/documentation/2.5.x/ScalaTestingWithScalaTest#Unit-Testing-Controllers
         which is:
           val controller = new ExampleController()
           val result: Future[Result] = controller.index().apply(FakeRequest())
           val bodyText: String = contentAsString(result)
      */
      val response: WSResponse = Await.result(futureResponse, timeout)
      responseText = response.asInstanceOf[AhcWSResponse].ahcResponse.toString
      if (response.status >= 400) {
        throw new OmDatabaseException("Error code from server: " + response.status)
      }
      val data: T = functionToCall(response, functionToCreateResultRow, inputs)
      Some(data)
    } catch {
      case e: Exception =>
        if (uiIn.isDefined) {
          val ans = uiIn.get.askYesNoQuestion("Unable to retrieve remote info for " + urlIn + " due to error: " + e.getMessage + ".  Show complete error?",
                                              Some("y"), allowBlankAnswer = true)
          if (ans.isDefined && ans.get) {
            val msg: String = getFullExceptionMessage(urlIn, responseText, Some(e))
            uiIn.get.displayText(msg)
          }
          None
        } else {
          val msg: String = getFullExceptionMessage(urlIn, responseText)
          throw new OmDatabaseException(msg, e)
        }
    }
  }

  def getFullExceptionMessage(urlIn: String, responseText: String, e: Option[Exception] = None): String = {
    val localErrMsg1 = "Failed to retrieve remote info for " + urlIn + " due to exception"
    val localErrMsg2 = "The actual response text was: \"" + responseText + "\""
    val msg: String =
      if (e.isDefined) {
        val stackTrace: String = Util.throwableToString(e.get)
        localErrMsg1 + ":  " + stackTrace + TextUI.NEWLN + localErrMsg2
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
  // Idea: There are probably nicer scala idioms for doing this wrapping instead of the 2-method approach with "process*" methods; maybe should use them.

  // Idea: could methods like this be combined with a type parameter [T] ? (like the git commit i reverted ~ 2016-11-17 but, another try?)
  def processLong(response: WSResponse, ignore: Option[(Seq[JsValue]) => Any], ignore2: Array[Any]): Long = {
    response.json.as[Long]
  }
  def getLong(pathIn: String): Long = {
    RestDatabase.restCall[Long, Any]("http://" + mRemoteAddress + pathIn, processLong, None, Array())
  }
  def processBoolean(response: WSResponse, ignore: Option[(Seq[JsValue]) => Any], ignore2: Array[Any]): Boolean = {
    response.json.as[Boolean]
  }
  def getBoolean(pathIn: String): Boolean = {
    RestDatabase.restCall[Boolean, Any]("http://" + mRemoteAddress + pathIn, processBoolean, None, Array())
  }
  def processOptionString(response: WSResponse, ignore: Option[(Seq[JsValue]) => Any], ignore2: Array[Any]): Option[String] = {
    if (response.json == JsNull) {
      None
    } else {
      Some(response.json.as[String])
    }
  }
  def getOptionString(pathIn: String): Option[String] = {
    RestDatabase.restCall[Option[String], Any]("http://" + mRemoteAddress + pathIn, processOptionString, None, Array())
  }
  def processOptionLong(response: WSResponse, ignore: Option[(Seq[JsValue]) => Any], ignore2: Array[Any]): Option[Long] = {
    if (response.json == JsNull) {
      None
    } else {
      Some(response.json.as[Long])
    }
  }
  def getOptionLongFromRest(pathIn: String): Option[Long] = {
    RestDatabase.restCall[Option[Long], Any]("http://" + mRemoteAddress + pathIn, processOptionLong, None, Array())
  }
  def processOptionBoolean(response: WSResponse, ignore: Option[(Seq[JsValue]) => Any], ignore2: Array[Any]): Option[Boolean] = {
    if (response.json == JsNull) {
      None
    } else {
      Some(response.json.as[Boolean])
    }
    /*For future reference, here is a possible alternative approach to determining if something in JSON is null and responding accordingly:
    try {
      // next line gets an exception if it is not null?:
      response.json.as[Null]
      None
    } catch {
      case e: Exception => Some(response.json.as[Boolean])
    } */
  }
  def getOptionBoolean(pathIn: String): Option[Boolean] = {
    RestDatabase.restCall[Option[Boolean], Any]("http://" + mRemoteAddress + pathIn, processOptionBoolean, None, Array())
  }

  /** (See comment on processArrayOptionAny.
    * Idea: consolidate this method and its caller with getCollection and processCollection? */
  def processListArrayOptionAny(response: WSResponse, ignore: Option[(Seq[JsValue]) => Any], whateverUsefulInfoIn: Array[Any]): List[Array[Option[Any]]] = {
    // (Idea: see comment at "functional-" in PostgreSQLDatabase.dbQuery.)
    var results: List[Array[Option[Any]]] = Nil
    if (response.json == JsNull) {
      // Nothing came back.  Preferring that a 404 (exception) only be when something broke. Idea: could return None instead maybe?
    } else {
      for (element <- response.json.asInstanceOf[JsArray].value) {
        val values: IndexedSeq[JsValue] = element.asInstanceOf[JsObject].values.toIndexedSeq
        val row: Array[Option[Any]] = getRow(whateverUsefulInfoIn, values)
        results = row :: results
      }
    }
    results.reverse
  }
  def getRow(whateverUsefulInfoIn: Array[Any], values: IndexedSeq[JsValue]): Array[Option[Any]] = {
    val result: Array[Option[Any]] = new Array[Option[Any]](values.size)
    val resultTypes: String = whateverUsefulInfoIn(0).asInstanceOf[String]
    var index = 0
    for (resultType: String <- resultTypes.split(",")) {
      // When modifying: COMPARE TO AND SYNCHRONIZE WITH THE TYPES IN the for loop in PostgreSQLDatabase.dbQuery .
      if (values(index) == JsNull) {
        result(index) = None
      } else if (resultType == "Float") {
        result(index) = Some(values(index).asInstanceOf[JsNumber].as[Float])
      } else if (resultType == "String") {
        result(index) = Some(values(index).asInstanceOf[JsString].as[String])
      } else if (resultType == "Long") {
        result(index) = Some(values(index).asInstanceOf[JsNumber].as[Long])
      } else if (resultType == "Boolean") {
        result(index) = Some(values(index).asInstanceOf[JsBoolean].as[Boolean])
      } else if (resultType == "Int") {
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
  def processArrayOptionAny(response: WSResponse, ignore: Option[(Seq[JsValue]) => Any], whateverUsefulInfoIn: Array[Any]): Array[Option[Any]] = {
    if (response.json == JsNull) {
      // Nothing came back.  Preferring that a 404 (exception) only be when something broke. Idea: could return None instead maybe?
      new Array[Option[Any]](0)
    } else {
      val values: IndexedSeq[JsValue] = response.json.asInstanceOf[JsObject].values.toIndexedSeq
      if (values.isEmpty) {
        throw new OmException("No results returned from data request.")
      }

      val row: Array[Option[Any]] = getRow(whateverUsefulInfoIn, values)
      row
    }
  }
  def getCollection[T](pathIn: String, inputs: Array[Any], createResultRow: Option[(Seq[JsValue]) => T]): ArrayList[T] = {
    RestDatabase.restCall[ArrayList[T], T]("http://" + mRemoteAddress + pathIn, processCollection, createResultRow, inputs)
  }
  def processCollection[T](response: WSResponse, createResultRow: Option[(Seq[JsValue]) => T], whateverUsefulInfoIn: Array[Any]): ArrayList[T] = {
    if (response.json == JsNull) {
      // Nothing came back.  Preferring that a 404 (exception) only be when something broke. Idea: could return None instead maybe?
      new ArrayList[T](0)
    } else {
      val values: Seq[JsValue] = response.json.asInstanceOf[JsArray].value
      val results: ArrayList[T] = new ArrayList[T](values.size)
      for (element <- values) {
        val values: IndexedSeq[JsValue] = element.asInstanceOf[JsObject].values.toIndexedSeq
        val row: T = createResultRow.get(values)
        results.add(row)
      }
      results
    }
  }
  def getArrayOptionAny(pathIn: String, inputs: Array[Any]): Array[Option[Any]] = {
    RestDatabase.restCall[Array[Option[Any]], Any]("http://" + mRemoteAddress + pathIn, processArrayOptionAny, None, inputs)
  }
  def getListArrayOptionAny(pathIn: String, inputs: Array[Any]): List[Array[Option[Any]]] = {
    RestDatabase.restCall[List[Array[Option[Any]]], Any]("http://" + mRemoteAddress + pathIn, processListArrayOptionAny, None, inputs)
  }

  @Override
  def isRemote: Boolean = true

  def getId: String = {
    getIdWithOptionalErrHandling(None).getOrElse(throw new OmDatabaseException("Unexpected behavior in getId: called method should have either thrown an" +
                                                                               " exception or returned an Option with data, but it returned None."))
  }
  def processString(responseIn: WSResponse, ignore: Option[(Seq[JsValue]) => Any], ignore2: Array[Any]): String = {
    responseIn.json.as[String]
  }
  /**
   * Same error handling behavior as in object RestDatabase.restCallWithErrorHandling.
   */
  def getIdWithOptionalErrHandling(uiIn: Option[TextUI]): Option[String] = {
    val url = "http://" + mRemoteAddress + "/id"
    RestDatabase.restCallWithOptionalErrorHandling[String, Any](url, processString, None, Array(), uiIn)
  }

  def getDefaultEntityId: Long = {
    getDefaultEntity(None).getOrElse(throw new OmDatabaseException("Unexpected behavior in getDefaultEntityWithOptionalErrHandling:" +
                                                                             " called method should have thrown an" +
                                                                             " exception or returned an Option with data, but returned None"))
  }
  def getDefaultEntity(uiIn: Option[TextUI]): Option[Long] = {
    def getDefaultEntity_processed(response: WSResponse, ignore: Option[(Seq[JsValue]) => Any], ignore2: Array[Any]): Long = {
      (response.json \ "id").as[Long]
    }
    val url = "http://" + mRemoteAddress + "/entities"
    RestDatabase.restCallWithOptionalErrorHandling[Long, Any](url, getDefaultEntity_processed, None, Array(), uiIn)
  }

  def getEntityJson_WithOptionalErrHandling(uiIn: Option[TextUI], idIn: Long): Option[String] = {
    def getEntity_processed(response: WSResponse, ignore: Option[(Seq[JsValue]) => Any], ignore2: Array[Any]): String = {
      /* Why doesn't next json line ("...as[String]") work but the following one does?  The first one gets:
        Failed to retrieve remote info for http://localhost:9000/entities/-9223372036854745151 due to exception:
         play.api.libs.json.JsResultException: JsResultException(errors:List((,List(ValidationError(List(error.expected.jsstring),WrappedArray())))))
              ....
              at play.api.libs.json.JsDefined.as(JsLookup.scala:132)
              at org.onemodel.core.database.RestDatabase.getEntity_processed(RestDatabase.scala:157)

      //  (response.json \ "id").as[String]
      //  (response.json \ "id").get.toString
      // But, didn't want to get just the id, anyway.
      */
      response.json.toString()
    }
    val url = "http://" + mRemoteAddress + "/entities/" + idIn + "/overview"
    RestDatabase.restCallWithOptionalErrorHandling[String, Any](url, getEntity_processed, None, Array(), uiIn)
  }

  override def getGroupSize(groupIdIn: Long, includeWhichEntitiesIn: Int = 3): Long = {
    getLong("/groups/" + groupIdIn + "/size/" + includeWhichEntitiesIn)
  }
  override def findUnusedGroupSortingIndex(groupIdIn: Long, startingWithIn: Option[Long]): Long = {
    getLong("/groups/" + groupIdIn + "/unusedSortingIndex/" + startingWithIn.getOrElse(""))
  }
  override def getHighestSortingIndexForGroup(groupIdIn: Long): Long = {
    getLong("/groups/" + groupIdIn + "/highestSortingIndex")
  }
  override def getGroupSortingIndex(groupIdIn: Long, entityIdIn: Long): Long = {
    getLong("/groups/" + groupIdIn + "/sortingIndex/" + entityIdIn)
  }
  override def getEntityAttributeSortingIndex(entityIdIn: Long, attributeFormIdIn: Long, attributeIdIn: Long): Long = {
    getLong("/entities/" + entityIdIn + "/sortingIndex/" + attributeFormIdIn + "/" + attributeIdIn)
  }

  override def getEntitiesOnlyCount(limitByClass: Boolean, classIdIn: Option[Long], templateEntity: Option[Long]): Long = {
    getLong("/entities/entitiesOnlyCount/" + limitByClass +
            (if (classIdIn.isEmpty) ""
            else {
              "/" + classIdIn.get + {
                if (templateEntity.isEmpty) ""
                else {
                  "/"  + templateEntity.get
                }
              }
            }))
  }
  override def getAttributeCount(entityIdIn: Long, includeArchivedEntitiesIn: Boolean = false): Long = {
    getLong("/entities/" + entityIdIn + "/attributeCount/" + includeArchivedEntitiesIn)
  }
  override def getCountOfGroupsContainingEntity(entityIdIn: Long): Long = {
    getLong("/entities/" + entityIdIn + "/countOfGroupsContaining")
  }
  override def getRelationToEntityCount(entityIdIn: Long, includeArchivedEntitiesIn: Boolean): Long = {
    getLong("/entities/" + entityIdIn + "/countOfRelationsToEntity/" + includeArchivedEntitiesIn)
  }
  override def getRelationToGroupCount(entityIdIn: Long): Long = {
    getLong("/entities/" + entityIdIn + "/countOfRelationsToGroup")
  }
  override def getClassCount(templateEntityIdIn: Option[Long]): Long = {
    getLong("/classes/count/" + templateEntityIdIn.getOrElse(""))
  }

  override def findUnusedAttributeSortingIndex(entityIdIn: Long, startingWithIn: Option[Long]): Long = {
    getLong("/entities/" + entityIdIn + "/unusedAttributeSortingIndex/" + startingWithIn.getOrElse(""))
  }
  override def isAttributeSortingIndexInUse(entityIdIn: Long, sortingIndexIn: Long): Boolean = {
    getBoolean("/entities/" + entityIdIn + "/isAttributeSortingIndexInUse/" + sortingIndexIn)
  }
  override def isGroupEntrySortingIndexInUse(groupIdIn: Long, sortingIndexIn: Long): Boolean = {
    getBoolean("/groups/" + groupIdIn + "/isEntrySortingIndexInUse/" + sortingIndexIn)
  }

  override def entityKeyExists(idIn: Long, includeArchived: Boolean): Boolean = {
    getBoolean("/entities/" + idIn + "/exists/" + includeArchived)
  }
  override def relationTypeKeyExists(idIn: Long): Boolean = {
    getBoolean("/relationTypes/" + idIn + "/exists")
  }
  override def omInstanceKeyExists(idIn: String): Boolean = {
    getBoolean("/omInstances/" + UriEncoding.encodePathSegment(idIn, "UTF-8") + "/exists")
  }
  override def classKeyExists(idIn: Long): Boolean = {
    getBoolean("/classes/" + idIn + "/exists")
  }
  override def attributeKeyExists(formIdIn: Long, idIn: Long): Boolean = {
    getBoolean("/attributes/" + formIdIn + "/" + idIn + "/exists")
  }
  override def quantityAttributeKeyExists(idIn: Long): Boolean = {
    getBoolean("/quantityAttributes/" + idIn + "/exists")
  }
  override def dateAttributeKeyExists(idIn: Long): Boolean = {
    getBoolean("/dateAttributes/" + idIn + "/exists")
  }
  override def booleanAttributeKeyExists(idIn: Long): Boolean = {
    getBoolean("/booleanAttributes/" + idIn + "/exists")
  }
  override def fileAttributeKeyExists(idIn: Long): Boolean = {
    getBoolean("/fileAttributes/" + idIn + "/exists")
  }
  override def textAttributeKeyExists(idIn: Long): Boolean = {
    getBoolean("/textAttributes/" + idIn + "/exists")
  }
  override def relationToEntityKeysExistAndMatch(idIn: Long, relationTypeIdIn: Long, entityId1In: Long, entityId2In: Long): Boolean = {
    getBoolean("/relationsToEntity/" + idIn + "/existsWith/" + relationTypeIdIn + "/" + entityId1In + "/" + entityId2In)
  }
  override def relationToEntityKeyExists(idIn: Long): Boolean = {
    getBoolean("/relationsToEntity/" + idIn + "/exists")
  }
  override def relationToRemoteEntityKeyExists(idIn: Long): Boolean = {
    getBoolean("/relationsToRemoteEntity/" + idIn + "/exists")
  }
  override def relationToRemoteEntityKeysExistAndMatch(idIn: Long, relationTypeIdIn: Long, entityId1In: Long, remoteInstanceIdIn: String, entityId2In: Long): Boolean = {
    getBoolean("/relationsToRemoteEntity/" + idIn + "/existsWith/" + relationTypeIdIn + "/" + entityId1In + "/" +
               UriEncoding.encodePathSegment(remoteInstanceIdIn, "UTF-8") + "/" + entityId2In)
  }
  override def relationToGroupKeysExistAndMatch(id: Long, entityId: Long, relationTypeId: Long, groupId: Long): Boolean = {
    getBoolean("/relationsToGroup/" + id + "/existsWith/" + entityId + "/" + relationTypeId + "/" + groupId)
  }
  override def groupKeyExists(idIn: Long): Boolean = {
    getBoolean("/groups/" + idIn + "/exists")
  }
  override def isDuplicateEntityName(nameIn: String, selfIdToIgnoreIn: Option[Long]): Boolean = {
    //If we need to change the 2nd parameter from UTF-8 to something else below, see javadocs for a class about encode/encoding, IIRC.
    val name = UriEncoding.encodePathSegment(nameIn, "UTF-8")
    getBoolean("/entities/isDuplicate/" + name + "/" + selfIdToIgnoreIn.getOrElse(""))
  }
  override def isDuplicateOmInstanceAddress(addressIn: String, selfIdToIgnoreIn: Option[String]): Boolean = {
    getBoolean("/omInstances/isDuplicate/" + UriEncoding.encodePathSegment(addressIn, "UTF-8") + "/" +
               UriEncoding.encodePathSegment(selfIdToIgnoreIn.getOrElse(""), "UTF-8"))
  }
  override def isEntityInGroup(groupIdIn: Long, entityIdIn: Long): Boolean = {
    getBoolean("/groups/" + groupIdIn + "/containsEntity/" + entityIdIn)
  }
  override def includeArchivedEntities: Boolean = {
    getBoolean("/entities/includeArchived")
  }

  override def getClassName(idIn: Long): Option[String] = {
    getOptionString("/classes/" + idIn + "/name")
  }
  override def getEntityName(idIn: Long): Option[String] = {
    getOptionString("/entities/" + idIn + "/name")
  }
  override def getNearestGroupEntrysSortingIndex(groupIdIn: Long, startingPointSortingIndexIn: Long, forwardNotBackIn: Boolean): Option[Long] = {
    getOptionLongFromRest("/groups/" + groupIdIn + "/nearestEntrysSortingIndex/" + startingPointSortingIndexIn + "/" + forwardNotBackIn)
  }
  override def getNearestAttributeEntrysSortingIndex(entityIdIn: Long, startingPointSortingIndexIn: Long, forwardNotBackIn: Boolean): Option[Long] = {
    getOptionLongFromRest("/entities/" + entityIdIn + "/nearestAttributeSortingIndex/" + startingPointSortingIndexIn + "/" + forwardNotBackIn)
  }
  override def getShouldCreateDefaultAttributes(classIdIn: Long): Option[Boolean] = {
    getOptionBoolean("/classes/" + classIdIn + "/shouldCreateDefaultAttributes")
  }
  override def getClassData(idIn: Long): Array[Option[Any]] = {
    getArrayOptionAny("/classes/" + idIn, Array(Database.getClassData_resultTypes))
  }
  override def getRelationTypeData(idIn: Long): Array[Option[Any]] = {
    getArrayOptionAny("/relationTypes/" + idIn, Array(Database.getRelationTypeData_resultTypes))
  }
  override def getOmInstanceData(idIn: String): Array[Option[Any]] = {
    val id = UriEncoding.encodePathSegment(idIn, "UTF-8")
    getArrayOptionAny("/omInstances/" + id, Array(Database.getOmInstanceData_resultTypes))
  }
  override def getFileAttributeData(idIn: Long): Array[Option[Any]] = {
    getArrayOptionAny("/fileAttributes/" + idIn, Array(Database.getFileAttributeData_resultTypes))
  }
  override def getTextAttributeData(idIn: Long): Array[Option[Any]] = {
    getArrayOptionAny("/textAttributes/" + idIn, Array(Database.getTextAttributeData_resultTypes))
  }
  override def getQuantityAttributeData(idIn: Long): Array[Option[Any]] = {
    getArrayOptionAny("/quantityAttributes/" + idIn, Array(Database.getQuantityAttributeData_resultTypes))
  }
  override def getRelationToGroupData(idIn: Long): Array[Option[Any]] = {
    getArrayOptionAny("/relationsToGroup/" + idIn, Array(Database.getRelationToGroupDataById_resultTypes))
  }
  override def getRelationToGroupDataByKeys(entityId: Long, relationTypeId: Long, groupId: Long): Array[Option[Any]] = {
    getArrayOptionAny("/relationsToGroup/byKeys/" + entityId + "/" + relationTypeId + "/" + groupId, Array(Database.getRelationToGroupDataByKeys_resultTypes))
  }
  override def getGroupData(idIn: Long): Array[Option[Any]] = {
    getArrayOptionAny("/groups/" + idIn, Array(Database.getGroupData_resultTypes))
  }
  override def getDateAttributeData(idIn: Long): Array[Option[Any]] = {
    getArrayOptionAny("/dateAttributes/" + idIn, Array(Database.getDateAttributeData_resultTypes))
  }
  override def getBooleanAttributeData(idIn: Long): Array[Option[Any]] = {
    getArrayOptionAny("/booleanAttributes/" + idIn, Array(Database.getBooleanAttributeData_resultTypes))
  }
  override def getRelationToEntityData(relationTypeIdIn: Long, entityId1In: Long, entityId2In: Long): Array[Option[Any]] = {
    getArrayOptionAny("/relationsToEntity/" + relationTypeIdIn + "/" + entityId1In + "/" + entityId2In, Array(Database.getRelationToEntity_resultTypes))
  }
  override def getRelationToRemoteEntityData(relationTypeIdIn: Long, entityId1In: Long, remoteInstanceIdIn: String, entityId2In: Long): Array[Option[Any]] = {
    getArrayOptionAny("/relationsToRemoteEntity/" + relationTypeIdIn + "/" + entityId1In + "/" +
                      UriEncoding.encodePathSegment(remoteInstanceIdIn, "UTF-8") + "/" + entityId2In,
                      Array(Database.getRelationToRemoteEntity_resultTypes))
  }
  override def getEntityData(idIn: Long): Array[Option[Any]] = {
    getArrayOptionAny("/entities/" + idIn, Array(Database.getEntityData_resultTypes))
  }
  override def getAdjacentGroupEntriesSortingIndexes(groupIdIn: Long, adjacentToEntrySortingIndexIn: Long, limitIn: Option[Long],
                                                     forwardNotBackIn: Boolean): List[Array[Option[Any]]] = {
    getListArrayOptionAny("/groups/" + groupIdIn + "/adjacentEntriesSortingIndexes/" + adjacentToEntrySortingIndexIn + "/" + forwardNotBackIn +
                          (if (limitIn.isEmpty) "" else "?limit=" + limitIn.get),
                          Array("Long"))
  }
  //Idea: simplify return type of things like this so it is more consumer-friendly, unless it is more friendly to be like the other code already is (ie,
  // like now). Some
  //of the other methods return less generic structures and they are more work to consume in this class because they are different/nonstandard so more
  //methods needed to handle each kind.
  override def getGroupsContainingEntitysGroupsIds(groupIdIn: Long, limitIn: Option[Long]): List[Array[Option[Any]]] = {
    getListArrayOptionAny("/groups/" + groupIdIn + "/containingEntitysGroupsIds" + (if (limitIn.isEmpty) "" else "?limit=" + limitIn.get), Array("Long"))
  }
  override def getGroupEntriesData(groupIdIn: Long, limitIn: Option[Long], includeArchivedEntitiesIn: Boolean): List[Array[Option[Any]]] = {
    getListArrayOptionAny("/groups/" + groupIdIn + "/entriesData/" + includeArchivedEntitiesIn + (if (limitIn.isEmpty) "" else "?limit=" + limitIn.get),
                          Array(Database.getGroupEntriesData_resultTypes))
  }
  override def getAdjacentAttributesSortingIndexes(entityIdIn: Long, sortingIndexIn: Long, limitIn: Option[Long],
                                                   forwardNotBackIn: Boolean): List[Array[Option[Any]]] = {
    getListArrayOptionAny("/entities/" + entityIdIn + "/adjacentAttributesSortingIndexes/" + sortingIndexIn + "/" + forwardNotBackIn +
                          (if (limitIn.isEmpty) "" else "?limit=" + limitIn.get),
                          Array("Long"))
  }
  def createTextAttributeRow(values: Seq[JsValue]): TextAttribute = {
    new TextAttribute(this, values(0).asInstanceOf[JsNumber].as[Long], values(1).asInstanceOf[JsNumber].as[Long],
                        values(2).asInstanceOf[JsNumber].as[Long],
                        values(3).asInstanceOf[JsString].as[String],
                        if (values(4) == JsNull) None else Some(values(4).asInstanceOf[JsNumber].as[Long]),
                        values(5).asInstanceOf[JsNumber].as[Long],
                        values(6).asInstanceOf[JsNumber].as[Long])
  }
  override def getTextAttributeByTypeId(parentEntityIdIn: Long, typeIdIn: Long, expectedRows: Option[Int]): java.util.ArrayList[TextAttribute] = {
    getCollection[TextAttribute]("/entities/" + parentEntityIdIn + "/textAttributeByTypeId/" + typeIdIn +
                                 (if (expectedRows.isEmpty) "" else "?expectedRows=" + expectedRows.get),
                                 Array(), Some(createTextAttributeRow))
  }
  def createLongValueRow(values: Seq[JsValue]): Long = {
    values(0).asInstanceOf[JsNumber].as[Long]
  }
  override def findContainedEntityIds(resultsInOut: mutable.TreeSet[Long], fromEntityIdIn: Long, searchStringIn: String, levelsRemaining: Int,
                                      stopAfterAnyFound: Boolean): mutable.TreeSet[Long] = {
    val searchString = UriEncoding.encodePathSegment(searchStringIn, "UTF-8")
    val results: util.ArrayList[Long] = getCollection[Long]("/entities/" + fromEntityIdIn + "/findContainedIds/" + searchString +
                                                            "/" + levelsRemaining + "/" + stopAfterAnyFound, Array(), Some(createLongValueRow))
    // then convert to the needed type:
    val treeSetResults: mutable.TreeSet[Long] = mutable.TreeSet[Long]()
    for (result: Long <- results) {
      treeSetResults.add(result)
    }
    treeSetResults
  }
  override def findAllEntityIdsByName(nameIn: String, caseSensitive: Boolean): java.util.ArrayList[Long] = {
    val name = UriEncoding.encodePathSegment(nameIn, "UTF-8")
    getCollection[Long]("/entities/findAllByName/" + name + "/" + caseSensitive, Array(), Some(createLongValueRow))
  }
  override def getContainingGroupsIds(entityIdIn: Long): java.util.ArrayList[Long] = {
    getCollection[Long]("/entities/" + entityIdIn + "/containingGroupsIds", Array(), Some(createLongValueRow))
  }
  def createRelationToGroupRow(values: Seq[JsValue]): RelationToGroup = {
    new RelationToGroup(this, values(0).asInstanceOf[JsNumber].as[Long], values(1).asInstanceOf[JsNumber].as[Long],
                        values(2).asInstanceOf[JsNumber].as[Long],
                        values(3).asInstanceOf[JsNumber].as[Long],
                        if (values(4) == JsNull) None else Some(values(4).asInstanceOf[JsNumber].as[Long]),
                        values(5).asInstanceOf[JsNumber].as[Long],
                        values(6).asInstanceOf[JsNumber].as[Long])
  }
  override def getContainingRelationToGroups(entityIdIn: Long, startingIndexIn: Long, limitIn: Option[Long]): ArrayList[RelationToGroup] = {
    // (The 2nd parameter has to match the types in the 2nd (1st alternate) constructor for RelationToGroup.  Consider putting it in a constant like
    // Database.getClassData_resultTypes etc.)
    getCollection[RelationToGroup]("/entities/" + entityIdIn + "/containingRelationToGroups/" + startingIndexIn +
                                   (if (limitIn.isEmpty) "" else "?limit=" + limitIn.get),
                                   Array(),
                                   Some(createRelationToGroupRow))
  }
  override def findRelationType(typeNameIn: String, expectedRows: Option[Int]): ArrayList[Long] = {
    getCollection[Long]("/relationTypes/find/" + UriEncoding.encodePathSegment(typeNameIn, "UTF-8") +
                        (if (expectedRows.isEmpty)  "" else "?expectedRows=" + expectedRows.get),
                        Array(), Some(createLongValueRow))
  }
  // idea: make private all methods used for the same purpose like this one:
  def createEntityRow(values: Seq[JsValue]): Entity = {
    new Entity(this, values(0).asInstanceOf[JsNumber].as[Long],
               values(1).asInstanceOf[JsString].as[String],
               if (values(2) == JsNull) None else Some(values(2).asInstanceOf[JsNumber].as[Long]),
               values(3).asInstanceOf[JsNumber].as[Long],
               if (values(4) == JsNull) None else Some(values(4).asInstanceOf[JsBoolean].as[Boolean]),
               values(5).asInstanceOf[JsBoolean ].as[Boolean],
               values(6).asInstanceOf[JsBoolean ].as[Boolean])
  }
  override def getGroupEntryObjects(groupIdIn: Long, startingObjectIndexIn: Long, maxValsIn: Option[Long]): ArrayList[Entity] = {
    getCollection[Entity]("/groups/" + groupIdIn + "/entries/" + startingObjectIndexIn +
                          (if (maxValsIn.isEmpty)  "" else "?maxValsIn=" + maxValsIn.get),
                          Array(), Some(createEntityRow))
  }
  def createRelationTypeIdAndEntityRow(values: Seq[JsValue]): (Long, Entity) = {
    val entity: Entity = createEntityRow(values)
    val relationTypeId: Long = values(7).asInstanceOf[JsNumber].as[Long]
    (relationTypeId, entity)
  }
  override def getEntitiesContainingGroup(groupIdIn: Long, startingIndexIn: Long, maxValsIn: Option[Long]): ArrayList[(Long, Entity)] = {
    getCollection[(Long,Entity)]("/groups/" + groupIdIn + "/containingEntities/" + startingIndexIn +
                                 (if (maxValsIn.isEmpty)  "" else "?maxValsIn=" + maxValsIn.get),
                                 Array(), Some(createRelationTypeIdAndEntityRow))
  }
  override def getEntitiesContainingEntity(entityIdIn: Long, startingIndexIn: Long, maxValsIn: Option[Long]): ArrayList[(Long, Entity)] = {
    getCollection[(Long,Entity)]("/entities/" + entityIdIn + "/containingEntities/" + startingIndexIn +
                                 (if (maxValsIn.isEmpty)  "" else "?maxValsIn=" + maxValsIn.get),
                                 Array(), Some(createRelationTypeIdAndEntityRow))
  }
  def process2Longs(response: WSResponse, ignore: Option[(Seq[JsValue]) => Any], ignore2: Array[Any]): (Long, Long) = {
    if (response.json == JsNull) {
      throw new OmDatabaseException("Unexpected: null result in the REST response (basically the remote side saying \"found nothing\".")
    } else {
      val values: IndexedSeq[JsValue] = response.json.asInstanceOf[JsObject].values.toIndexedSeq
      val first: Long = values(0).asInstanceOf[JsNumber].as[Long]
      val second: Long = values(1).asInstanceOf[JsNumber].as[Long]
      (first, second)
    }
  }
  def get2Longs(pathIn: String): (Long, Long) = {
    RestDatabase.restCall[(Long, Long), Any]("http://" + mRemoteAddress + pathIn, process2Longs, None, Array())
  }
  override def getCountOfEntitiesContainingGroup(groupIdIn: Long): (Long, Long) = {
    get2Longs("/groups/" + groupIdIn + "/countOfContainingEntities")
  }
  override def getCountOfEntitiesContainingEntity(entityIdIn: Long): (Long, Long) = {
    get2Longs("/entities/" + entityIdIn + "/countOfContainingEntities")
  }
  override def getFileAttributeContent(fileAttributeIdIn: Long, outputStreamIn: OutputStream): (Long, String) = {
    // (Idea: should this (and others) instead just call something that returns a complete FileAttribute, so that multiple places in the code do
    // not all have to know the indexes for each datum?:)
    val faData = getFileAttributeData(fileAttributeIdIn)
    val fileSize = faData(9).get.asInstanceOf[Long]
    val md5hash = faData(10).get.asInstanceOf[String]
    val url = new URL("http://" + mRemoteAddress + "/fileAttributes/" + fileAttributeIdIn + "/content")
    var input: InputStream = null
    try {
      input = url.openStream()
      // see mention of 4096 elsewhere for why that # was chosen
      val b = new Array[Byte](4096)
      @tailrec def stream() {
        //Idea, also tracked in tasks: put at least next line or surrounding, on a separate thread or w/ a future, so it can use a timeout & not block forever:
        val length = input.read(b)
        if (length != -1) {
          outputStreamIn.write(b, 0, length)
          stream()
        }
      }
      stream()
    } finally {
      if (input != null) input.close()
    }
    (fileSize, md5hash)
  }
  def processOptionLongsBoolean(response: WSResponse, ignore: Option[(Seq[JsValue]) => Any],
                                ignore2: Array[Any]): (Option[Long], Option[Long], Option[Long], Boolean) = {
    if (response.json == JsNull) {
      throw new OmDatabaseException("Unexpected: null result in the REST response (basically the remote side saying \"found nothing\".")
    } else {
      val values: IndexedSeq[JsValue] = response.json.asInstanceOf[JsObject].values.toIndexedSeq
      val first: Option[Long] = getOptionLongFromJson(values, 0)
      val second: Option[Long] = getOptionLongFromJson(values, 1)
      val third: Option[Long] = getOptionLongFromJson(values, 2)
      val last: Boolean = values(3).asInstanceOf[JsBoolean].as[Boolean]
      (first, second, third, last)
    }
  }
  def getOptionLongsBoolean(pathIn: String): (Option[Long], Option[Long], Option[Long], Boolean) = {
    RestDatabase.restCall[(Option[Long], Option[Long], Option[Long], Boolean), Any]("http://" + mRemoteAddress + pathIn,
                                                                                    processOptionLongsBoolean, None, Array())
  }
  override def findRelationToAndGroup_OnEntity(entityIdIn: Long, groupNameIn: Option[String]): (Option[Long], Option[Long], Option[Long], Boolean) = {
    getOptionLongsBoolean("/entities/" + entityIdIn + "/findRelationToAndGroup" +
                          (if (groupNameIn.isEmpty)  "" else "?groupName=" + java.net.URLEncoder.encode(groupNameIn.get, "UTF-8")))
                          // Note: using a different kind of encoder/encoding for a query part of a URI, per info at:
                          //   https://www.playframework.com/documentation/2.5.x/api/scala/index.html#play.utils.UriEncoding$
                          // ...which says:
    /*"Encode a string so that it can be used safely in the "path segment" part of a URI. A path segment is defined in RFC 3986. In a URI such as http://www.example.com/abc/def?a=1&b=2 both abc and def are path segments.
    Path segment encoding differs from encoding for other parts of a URI. For example, the "&" character is permitted in a path segment, but has special meaning in query parameters. On the other hand, the "/" character cannot appear in a path segment, as it is the path delimiter, so it must be encoded as "%2F". These are just two examples of the differences between path segment and query string encoding; there are other differences too.
    When encoding path segments the encodePathSegment method should always be used in preference to the java.net.URLEncoder.encode method. URLEncoder.encode, despite its name, actually provides encoding in the application/x-www-form-urlencoded MIME format which is the encoding used for form data in HTTP GET and POST requests. This encoding is suitable for inclusion in the query part of a URI. But URLEncoder.encode should not be used for path segment encoding. (Also note that URLEncoder.encode is not quite spec compliant. For example, it percent-encodes the ~ character when really it should leave it as unencoded.)"
    */
  }
  def getOptionLongFromJson(values: IndexedSeq[JsValue], index: Int): Option[Long] = {
    if (values(index) == JsNull) None
    else {
      Some(values(index).asInstanceOf[JsNumber].as[Long])
      // Idea: learn why in some places this needed instead: is there a difference in the way it is sent from the web module? or do both work?:
      // Some(response.json.as[Long])
    }
  }
  def processSortedAttributes(response: WSResponse, ignore: Option[(Seq[JsValue]) => Any], ignore2: Array[Any]): (Array[(Long, Attribute)], Int) = {
    if (response.json == JsNull) {
      throw new OmDatabaseException("Unexpected: null result in the REST response (basically the remote side saying \"found nothing\".")
    } else {
      val arrayAndInt = response.json.asInstanceOf[JsObject].values.toIndexedSeq
      val totalAttributesAvailable: Int = arrayAndInt(0).asInstanceOf[JsNumber].as[Int]
      val attributesRetrieved: JsArray = arrayAndInt(1).asInstanceOf[JsArray]
      val resultsAccumulator = new ArrayList[(Long, Attribute)](totalAttributesAvailable)
      for (attributeJson <- attributesRetrieved.value) {
        val values: IndexedSeq[JsValue] = attributeJson.asInstanceOf[JsObject].values.toIndexedSeq
        val id: Long = values(0).asInstanceOf[JsNumber].as[Long]
        val formId: Long = values(1).asInstanceOf[JsNumber].as[Long]
        val parentId: Long = values(2).asInstanceOf[JsNumber].as[Long]
        val attributeTypeId: Long = values(3).asInstanceOf[JsNumber].as[Long]
        val sortingIndex: Long = values(4).asInstanceOf[JsNumber].as[Long]
        val attribute: Attribute = formId match {
          case 1 =>
            val validOnDate = getOptionLongFromJson(values, 5)
            val observationDate: Long = values(6).asInstanceOf[JsNumber].as[Long]
            val unitId: Long = values(7).asInstanceOf[JsNumber].as[Long]
            val number: Float = values(8).asInstanceOf[JsNumber].as[Float]
            new QuantityAttribute(this, id, parentId, attributeTypeId, unitId, number, validOnDate, observationDate, sortingIndex)
          case 2 =>
            val date: Long = values(5).asInstanceOf[JsNumber].as[Long]
            new DateAttribute(this, id, parentId, attributeTypeId, date, sortingIndex)
          case 3 =>
            val validOnDate = getOptionLongFromJson(values, 5)
            val observationDate: Long = values(6).asInstanceOf[JsNumber].as[Long]
            val bool: Boolean = values(7).asInstanceOf[JsBoolean].as[Boolean]
            new BooleanAttribute(this, id, parentId, attributeTypeId, bool, validOnDate, observationDate, sortingIndex)
          case 4 =>
            val description = values(5).asInstanceOf[JsString].as[String]
            val originalFileDate = values(6).asInstanceOf[JsNumber].as[Long]
            val storedDate = values(7).asInstanceOf[JsNumber].as[Long]
            val originalFilePath = values(8).asInstanceOf[JsString].as[String]
            val readable: Boolean = values(9).asInstanceOf[JsBoolean].as[Boolean]
            val writable: Boolean = values(10).asInstanceOf[JsBoolean].as[Boolean]
            val executable: Boolean = values(11).asInstanceOf[JsBoolean].as[Boolean]
            val size = values(12).asInstanceOf[JsNumber].as[Long]
            val md5hash = values(13).asInstanceOf[JsString].as[String]
            new FileAttribute(this, id, parentId, attributeTypeId, description, originalFileDate, storedDate, originalFilePath, readable, writable,
                              executable, size, md5hash, sortingIndex)
          case 5 =>
            val validOnDate = getOptionLongFromJson(values, 5)
            val observationDate: Long = values(6).asInstanceOf[JsNumber].as[Long]
            val textEscaped = values(7).asInstanceOf[JsString].as[String]
            val text = org.apache.commons.lang3.StringEscapeUtils.unescapeJson(textEscaped)
            new TextAttribute(this, id, parentId, attributeTypeId, text, validOnDate, observationDate, sortingIndex)
          case 6 =>
            val validOnDate = getOptionLongFromJson(values, 5)
            val observationDate: Long = values(6).asInstanceOf[JsNumber].as[Long]
            val entityId1: Long = values(7).asInstanceOf[JsNumber].as[Long]
            val entityId2: Long = values(8).asInstanceOf[JsNumber].as[Long]
            new RelationToEntity(this, id, attributeTypeId, entityId1, entityId2, validOnDate, observationDate, sortingIndex)
          case 7 =>
            val validOnDate = getOptionLongFromJson(values, 5)
            val observationDate: Long = values(6).asInstanceOf[JsNumber].as[Long]
            val entityId: Long = values(7).asInstanceOf[JsNumber].as[Long]
            val groupId: Long = values(8).asInstanceOf[JsNumber].as[Long]
            new RelationToGroup(this, id, entityId, attributeTypeId, groupId, validOnDate, observationDate, sortingIndex)
          case 8 =>
            val validOnDate = getOptionLongFromJson(values, 5)
            val observationDate: Long = values(6).asInstanceOf[JsNumber].as[Long]
            val entityId1: Long = values(7).asInstanceOf[JsNumber].as[Long]
            val remoteInstanceId = values(8).asInstanceOf[JsString].as[String]
            val entityId2: Long = values(9).asInstanceOf[JsNumber].as[Long]
            new RelationToRemoteEntity(this, id, attributeTypeId, entityId1, remoteInstanceId, entityId2, validOnDate, observationDate, sortingIndex)
          case _ => throw new OmDatabaseException("unexpected formId: " + formId)
        }
        resultsAccumulator.add((sortingIndex, attribute))
      }
      (resultsAccumulator.toArray(new Array[(Long, Attribute)](0)), totalAttributesAvailable)
    }
  }
  override def getSortedAttributes(entityIdIn: Long, startingObjectIndexIn: Int, maxValsIn: Int, onlyPublicEntitiesIn: Boolean): (Array[(Long, Attribute)], Int) = {
    val path: String = "/entities/" + entityIdIn + "/sortedAttributes/" + startingObjectIndexIn + "/" + maxValsIn + "/" + onlyPublicEntitiesIn
    RestDatabase.restCall[(Array[(Long, Attribute)], Int), Any]("http://" + mRemoteAddress + path, processSortedAttributes, None, Array())
  }


  // Methods that WRITE to the DATABASE: when implementing later, REMEMBER TO MAKE READONLY OR SECURE (only showing public or allowed data),
  // OR HANDLE THEIR LACK, IN THE UI IN A FRIENDLY WAY.
  //idea: when implementing these, first sort by CRUD groupings, and/or by return type, to group similar things for ease?:
  override def beginTrans(): Unit = ???

  override def rollbackTrans(): Unit = ???

  override def commitTrans(): Unit = ???

  override def moveRelationToGroup(relationToGroupIdIn: Long, newContainingEntityIdIn: Long, sortingIndexIn: Long): Long = ???

  override def updateRelationToRemoteEntity(oldRelationTypeIdIn: Long, entityId1In: Long, remoteInstanceIdIn: String, entityId2In: Long, newRelationTypeIdIn: Long, validOnDateIn: Option[Long], observationDateIn: Long): Unit = ???

  override def unarchiveEntity(idIn: Long, callerManagesTransactionsIn: Boolean): Unit = ???

  override def setIncludeArchivedEntities(in: Boolean): Unit = ???

  override def createOmInstance(idIn: String, isLocalIn: Boolean, addressIn: String, entityIdIn: Option[Long], oldTableName: Boolean): Long = ???

  override def deleteOmInstance(idIn: String): Unit = ???

  override def deleteDateAttribute(idIn: Long): Unit = ???

  override def updateDateAttribute(idIn: Long, parentIdIn: Long, dateIn: Long, attrTypeIdIn: Long): Unit = ???

  override def updateRelationToGroup(entityIdIn: Long, oldRelationTypeIdIn: Long, newRelationTypeIdIn: Long, oldGroupIdIn: Long, newGroupIdIn: Long,
                                     validOnDateIn: Option[Long], observationDateIn: Long): Unit = ???
  override def archiveEntity(idIn: Long, callerManagesTransactionsIn: Boolean): Unit = ???

  override def moveEntityFromGroupToGroup(fromGroupIdIn: Long, toGroupIdIn: Long, moveEntityIdIn: Long, sortingIndexIn: Long): Unit = ???

  override def deleteClassAndItsTemplateEntity(classIdIn: Long): Unit = ???

  override def createRelationToEntity(relationTypeIdIn: Long, entityId1In: Long, entityId2In: Long, validOnDateIn: Option[Long], observationDateIn: Long,
                                      sortingIndexIn: Option[Long], callerManagesTransactionsIn: Boolean): RelationToEntity = ???

  override def deleteRelationToGroup(entityIdIn: Long, relationTypeIdIn: Long, groupIdIn: Long): Unit = ???

  override def deleteQuantityAttribute(idIn: Long): Unit = ???

  override def removeEntityFromGroup(groupIdIn: Long, containedEntityIdIn: Long, callerManagesTransactionsIn: Boolean): Unit = ???

  override def addEntityToGroup(groupIdIn: Long, containedEntityIdIn: Long, sortingIndexIn: Option[Long], callerManagesTransactionsIn: Boolean): Unit = ???

  override def deleteRelationToRemoteEntity(relationTypeIdIn: Long, entityId1In: Long, remoteInstanceIdIn: String, entityId2In: Long): Unit = ???

  override def deleteFileAttribute(idIn: Long): Unit = ???

  override def updateFileAttribute(idIn: Long, parentIdIn: Long, attrTypeIdIn: Long, descriptionIn: String): Unit = ???

  override def updateFileAttribute(idIn: Long, parentIdIn: Long, attrTypeIdIn: Long, descriptionIn: String, originalFileDateIn: Long, storedDateIn: Long, originalFilePathIn: String, readableIn: Boolean, writableIn: Boolean, executableIn: Boolean, sizeIn: Long, md5hashIn: String): Unit = ???

  override def updateQuantityAttribute(idIn: Long, parentIdIn: Long, attrTypeIdIn: Long, unitIdIn: Long, numberIn: Float, validOnDateIn: Option[Long], inObservationDate: Long): Unit = ???

  override def deleteGroupRelationsToItAndItsEntries(groupidIn: Long): Unit = ???

  override def updateEntitysClass(entityId: Long, classId: Option[Long], callerManagesTransactions: Boolean): Unit = ???

  override def deleteBooleanAttribute(idIn: Long): Unit = ???

  override def moveEntityFromEntityToGroup(removingRelationToEntityIn: RelationToEntity, targetGroupIdIn: Long, sortingIndexIn: Long): Unit = ???

  override def renumberSortingIndexes(entityIdOrGroupIdIn: Long, callerManagesTransactionsIn: Boolean, isEntityAttrsNotGroupEntries: Boolean): Unit = ???

  override def updateEntityOnlyNewEntriesStickToTop(idIn: Long, newEntriesStickToTop: Boolean): Unit = ???

  override def createDateAttribute(parentIdIn: Long, attrTypeIdIn: Long, dateIn: Long, sortingIndexIn: Option[Long]): Long = ???

  override def createGroupAndRelationToGroup(entityIdIn: Long, relationTypeIdIn: Long, newGroupNameIn: String, allowMixedClassesInGroupIn: Boolean, validOnDateIn: Option[Long], observationDateIn: Long, sortingIndexIn: Option[Long], callerManagesTransactionsIn: Boolean): (Long, Long) = ???

  override def addHASRelationToEntity(fromEntityIdIn: Long, toEntityIdIn: Long, validOnDateIn: Option[Long], observationDateIn: Long, sortingIndexIn: Option[Long]): RelationToEntity = ???

  override def updateRelationToEntity(oldRelationTypeIdIn: Long, entityId1In: Long, entityId2In: Long, newRelationTypeIdIn: Long, validOnDateIn:
  Option[Long], observationDateIn: Long): Unit = ???

  override def updateEntityInAGroup(groupIdIn: Long, entityIdIn: Long, sortingIndexIn: Long): Unit = ???

  override def updateAttributeSorting(entityIdIn: Long, attributeFormIdIn: Long, attributeIdIn: Long, sortingIndexIn: Long): Unit = ???

  override def updateGroup(groupIdIn: Long, nameIn: String, allowMixedClassesInGroupIn: Boolean, newEntriesStickToTopIn: Boolean): Unit = ???

  override def setUserPreference_EntityId(nameIn: String, entityIdIn: Long): Unit = ???

  override def deleteRelationType(idIn: Long): Unit = ???

  override def deleteGroupAndRelationsToIt(idIn: Long): Unit = ???

  override def deleteEntity(idIn: Long, callerManagesTransactionsIn: Boolean): Unit = ???

  override def moveRelationToEntity(relationToEntityIdIn: Long, newContainingEntityIdIn: Long, sortingIndexIn: Long): RelationToEntity = ???

  override def createFileAttribute(parentIdIn: Long, attrTypeIdIn: Long, descriptionIn: String, originalFileDateIn: Long, storedDateIn: Long, originalFilePathIn: String, readableIn: Boolean, writableIn: Boolean, executableIn: Boolean, sizeIn: Long, md5hashIn: String, inputStreamIn: FileInputStream, sortingIndexIn: Option[Long]): Long = ???

  override def deleteTextAttribute(idIn: Long): Unit = ???

  override def createEntityAndRelationToEntity(entityIdIn: Long, relationTypeIdIn: Long, newEntityNameIn: String, isPublicIn: Option[Boolean], validOnDateIn:
  Option[Long], observationDateIn: Long, callerManagesTransactionsIn: Boolean): (Long, Long) = ???

  override def moveEntityFromGroupToEntity(fromGroupIdIn: Long, toEntityIdIn: Long, moveEntityIdIn: Long, sortingIndexIn: Long): Unit = ???

  override def updateTextAttribute(idIn: Long, parentIdIn: Long, attrTypeIdIn: Long, textIn: String, validOnDateIn: Option[Long], observationDateIn: Long): Unit = ???

  override def getOrCreateClassAndTemplateEntityIds(classNameIn: String, callerManagesTransactionsIn: Boolean): (Long, Long) = ???
  override def addUriEntityWithUriAttribute(containingEntityIn: Entity, newEntityNameIn: String, uriIn: String, observationDateIn: Long, makeThemPublicIn:
  Option[Boolean], callerManagesTransactionsIn: Boolean, quoteIn: Option[String]): (Entity, RelationToEntity) = ???

  override def updateEntityOnlyPublicStatus(idIn: Long, value: Option[Boolean]): Unit = ???

  override def createRelationToRemoteEntity(relationTypeIdIn: Long, entityId1In: Long, entityId2In: Long, validOnDateIn: Option[Long], observationDateIn:
  Long, remoteInstanceIdIn: String, sortingIndexIn: Option[Long], callerManagesTransactionsIn: Boolean): RelationToRemoteEntity = ???

  override def createRelationToGroup(entityIdIn: Long, relationTypeIdIn: Long, groupIdIn: Long, validOnDateIn: Option[Long], observationDateIn: Long,
                                     sortingIndexIn: Option[Long], callerManagesTransactionsIn: Boolean): (Long, Long) = ???

  override def createBooleanAttribute(parentIdIn: Long, attrTypeIdIn: Long, booleanIn: Boolean, validOnDateIn: Option[Long], observationDateIn: Long,
                                      sortingIndexIn: Option[Long]): Long = ???

  override def createEntity(nameIn: String, classIdIn: Option[Long], isPublicIn: Option[Boolean]): Long = ???

  override def deleteRelationToEntity(relationTypeIdIn: Long, entityId1In: Long, entityId2In: Long): Unit = ???

  override def updateClassCreateDefaultAttributes(classIdIn: Long, value: Option[Boolean]) = ???

  override def updateBooleanAttribute(idIn: Long, parentIdIn: Long, attrTypeIdIn: Long, booleanIn: Boolean, validOnDateIn: Option[Long], inObservationDate: Long): Unit = ???

  override def createQuantityAttribute(parentIdIn: Long, attrTypeIdIn: Long, unitIdIn: Long, numberIn: Float, validOnDateIn: Option[Long],
                              inObservationDate: Long, callerManagesTransactionsIn: Boolean = false, sortingIndexIn: Option[Long] = None): /*id*/ Long = ???

  override def createTextAttribute(parentIdIn: Long, attrTypeIdIn: Long, textIn: String, validOnDateIn: Option[Long], observationDateIn: Long, callerManagesTransactionsIn: Boolean, sortingIndexIn: Option[Long]): Long = ???

}
