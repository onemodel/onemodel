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

import java.io.{FileInputStream, OutputStream}
import java.util

import akka.actor.ActorSystem
import akka.stream.ActorMaterializer
import org.onemodel.core.model.{Entity, RelationToEntity, Attribute, RelationToRemoteEntity}
import org.onemodel.core.{OmDatabaseException, TextUI, Util}
import play.api.libs.ws.{WSResponse, WSClient}
import play.api.libs.ws.ahc.{AhcWSResponse, AhcWSClient}
import scala.concurrent.duration._

import scala.concurrent.{Await, Future}

object RestDatabase {
  // (Details on this REST client system are at:  https://www.playframework.com/documentation/2.5.x/ScalaWS#Directly-creating-WSClient .)
  val timeout: FiniteDuration = 20.seconds
  implicit val actorSystem: ActorSystem = ActorSystem()
  implicit val actorMaterializer: ActorMaterializer = ActorMaterializer()
  lazy val wsClient: WSClient = AhcWSClient()
  implicit val context = play.api.libs.concurrent.Execution.Implicits.defaultContext

  def restCall[T](urlIn: String,
                  functionToCall: (WSResponse, Array[AnyVal]) => T,
                  inputs: Array[AnyVal]): T = {
    restCallWithOptionalErrorHandling(urlIn, functionToCall, inputs, None).get
  }

  /**
   * Does error handling internally to the provided UI, only if the parameter uiIn.isDefined (ie, not None), otherwise throws the
   * exception to the caller.  Either returns a Some(data), or shows the exception in the UI then returns None, or throws an exception.
   */
  def restCallWithOptionalErrorHandling[T](urlIn: String,
                                           functionToCall: (WSResponse, Array[AnyVal]) => T,
                                           inputs: Array[AnyVal],
                                           uiIn: Option[TextUI]): Option[T] = {
    var responseText = ""
    try {
      val request = RestDatabase.wsClient.url(urlIn).withFollowRedirects(true)
      val futureResponse: Future[WSResponse] = request.get()
      val response: WSResponse = Await.result(futureResponse, timeout)
      responseText = response.asInstanceOf[AhcWSResponse].ahcResponse.toString
      if (response.status >= 400) {
        throw new OmDatabaseException("Error code from server: " + response.status)
      }
      val data: T = functionToCall(response, inputs)
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
  // Idea: There are probably nicer scala idioms for doing this wrapping instead of the 2-method approach with "_asRightType" methods; maybe should use them.

  @Override
  def isRemote: Boolean = true

  def getId: String = {
    getIdWithOptionalErrHandling(None).getOrElse(throw new OmDatabaseException("Unexpected behavior in getId: called method should have either thrown an" +
                                                                               " exception or returned an Option with data, but it returned None."))
  }
  /**
   * Same error handling behavior as in object RestDatabase.restCallWithErrorHandling.
   */
  def getIdWithOptionalErrHandling(uiIn: Option[TextUI]): Option[String] = {
    val url = "http://" + mRemoteAddress + "/id"
    RestDatabase.restCallWithOptionalErrorHandling[String](url, getId_asRightType, Array(), uiIn)
  }
  def getId_asRightType(responseIn: WSResponse, ignore: Array[AnyVal]): String = {
    responseIn.json.as[String]
  }

  def getDefaultEntity: Long = {
    getDefaultEntityWithOptionalErrHandling(None).getOrElse(throw new OmDatabaseException("Unexpected behavior in getDefaultEntityWithOptionalErrHandling:" +
                                                                                          " called method should have thrown an" +
                                                                                          " exception or returned an Option with data, but returned None"))
  }
  def getDefaultEntityWithOptionalErrHandling(uiIn: Option[TextUI]): Option[Long] = {
    val url = "http://" + mRemoteAddress + "/entities"
    RestDatabase.restCallWithOptionalErrorHandling[Long](url, getDefaultEntity_asRightType, Array(), uiIn)
  }
  def getDefaultEntity_asRightType(response: WSResponse, ignore: Array[AnyVal]): Long = {
    (response.json \ "id").as[Long]
  }

  def getEntity(id: Long): String = {
    getEntityWithOptionalErrHandling(None, id).getOrElse(throw new OmDatabaseException("Unexpected behavior in getEntityWithOptionalErrHandling:" +
                                                                                       " called method should have thrown an" +
                                                                                       " exception or returned an Option with data, but returned None"))
  }
  def getEntityWithOptionalErrHandling(uiIn: Option[TextUI], idIn: Long): Option[String] = {
    val url = "http://" + mRemoteAddress + "/entities/" + idIn
    RestDatabase.restCallWithOptionalErrorHandling[String](url, getEntity_asRightType, Array(), uiIn)
  }
  def getEntity_asRightType(response: WSResponse, ignore: Array[AnyVal]): String = {
    /* Why doesn't next json line ("...as[String]") work but the following one does?  The first one gets:
      Failed to retrieve remote info for http://localhost:9000/entities/-9223372036854745151 due to exception:
       play.api.libs.json.JsResultException: JsResultException(errors:List((,List(ValidationError(List(error.expected.jsstring),WrappedArray())))))
            ....
            at play.api.libs.json.JsDefined.as(JsLookup.scala:132)
            at org.onemodel.core.database.RestDatabase.getEntity_asRightType(RestDatabase.scala:157)

    //  (response.json \ "id").as[String]
    //  (response.json \ "id").get.toString
    // But, didn't want to get just the id, anyway.
    */
    response.json.toString()
  }

  override def createQuantityAttribute(parentIdIn: Long, attrTypeIdIn: Long, unitIdIn: Long, numberIn: Float, validOnDateIn: Option[Long],
                                       inObservationDate: Long, callerManagesTransactionsIn: Boolean, sortingIndexIn: Option[Long]): Long = ???

  override def updateRelationToRemoteEntity(oldRelationTypeIdIn: Long, entityId1In: Long, remoteInstanceIdIn: String, entityId2In: Long,
                                            newRelationTypeIdIn: Long, validOnDateIn: Option[Long], observationDateIn: Long): Unit = ???

  override def getDateAttributeData(idIn: Long): Array[Option[Any]] = ???

  override def unarchiveEntity(idIn: Long, callerManagesTransactionsIn: Boolean): Unit = ???

  override def getGroupSize(groupIdIn: Long, includeWhichEntitiesIn: Int): Long = ???

  override def relationToEntityKeysExistAndMatch(idIn: Long, relTypeIdIn: Long, entityId1In: Long, entityId2In: Long): Boolean = ???

  override def createOmInstance(idIn: String, isLocalIn: Boolean, addressIn: String, entityIdIn: Option[Long], oldTableName: Boolean): Long = ???

  override def deleteOmInstance(idIn: String): Unit = ???

  override def getRelationToRemoteEntityData(relTypeIdIn: Long, entityId1In: Long, remoteInstanceIdIn: String, entityId2In: Long): Array[Option[Any]] = ???

  override def deleteDateAttribute(idIn: Long): Unit = ???

  override def getEntityData(idIn: Long): Array[Option[Any]] = ???

  override def updateDateAttribute(idIn: Long, parentIdIn: Long, dateIn: Long, attrTypeIdIn: Long): Unit = ???

  override def updateRelationToGroup(entityIdIn: Long, oldRelationTypeIdIn: Long, newRelationTypeIdIn: Long, oldGroupIdIn: Long, newGroupIdIn: Long,
                                     validOnDateIn: Option[Long], observationDateIn: Long): Unit = ???

  override def relationToEntityKeyExists(idIn: Long): Boolean = ???

  override def findRelationType(typeNameIn: String, expectedRows: Option[Int]): Array[Long] = ???

  override def archiveEntity(idIn: Long, callerManagesTransactionsIn: Boolean): Unit = ???

  override def classKeyExists(idIn: Long): Boolean = ???

  override def deleteClassAndItsTemplateEntity(classIdIn: Long): Unit = ???

  override def booleanAttributeKeyExists(idIn: Long): Boolean = ???

  override def deleteQuantityAttribute(idIn: Long): Unit = ???

  override def relationToRemoteEntityKeysExistAndMatch(idIn: Long, relTypeIdIn: Long, entityId1In: Long, remoteInstanceIdIn: String,
                                                       entityId2In: Long): Boolean = ???

  override def getBooleanAttributeData(idIn: Long): Array[Option[Any]] = ???

  override def removeEntityFromGroup(groupIdIn: Long, containedEntityIdIn: Long, callerManagesTransactionsIn: Boolean): Unit = ???

  override def addEntityToGroup(groupIdIn: Long, containedEntityIdIn: Long, sortingIndexIn: Option[Long], callerManagesTransactionsIn: Boolean): Unit = ???

  override def getRelationToEntityData(relTypeIdIn: Long, entityId1In: Long, entityId2In: Long): Array[Option[Any]] = ???

  override def dateAttributeKeyExists(idIn: Long): Boolean = ???

  override def createRelationToEntity(relationTypeIdIn: Long, entityId1In: Long, entityId2In: Long, validOnDateIn: Option[Long],
                                      observationDateIn: Long, sortingIndexIn: Option[Long], callerManagesTransactionsIn: Boolean): RelationToEntity = ???

  override def deleteRelationToGroup(entityIdIn: Long, relTypeIdIn: Long, groupIdIn: Long): Unit = ???

  override def isDuplicateOmInstance(addressIn: String, selfIdToIgnoreIn: Option[String]): Boolean = ???

  override def getGroupEntryObjects(groupIdIn: Long, startingObjectIndexIn: Long, maxValsIn: Option[Long]): util.ArrayList[Entity] = ???

  override def includeArchivedEntities: Boolean = ???

  override def deleteFileAttribute(idIn: Long): Unit = ???

  override def updateFileAttribute(idIn: Long, parentIdIn: Long, attrTypeIdIn: Long, descriptionIn: String): Unit = ???

  override def updateFileAttribute(idIn: Long, parentIdIn: Long, attrTypeIdIn: Long, descriptionIn: String, originalFileDateIn: Long, storedDateIn: Long,
                                   originalFilePathIn: String, readableIn: Boolean, writableIn: Boolean, executableIn: Boolean, sizeIn: Long,
                                   md5hashIn: String): Unit = ???

  override def relationTypeKeyExists(idIn: Long): Boolean = ???

  override def updateQuantityAttribute(idIn: Long, parentIdIn: Long, attrTypeIdIn: Long, unitIdIn: Long, numberIn: Float, validOnDateIn: Option[Long],
                                       inObservationDate: Long): Unit = ???

  override def getClassData(idIn: Long): Array[Option[Any]] = ???

  override def getEntityName(idIn: Long): Option[String] = ???

  override def deleteRelationToRemoteEntity(relTypeIdIn: Long, entityId1In: Long, remoteInstanceIdIn: String, entityId2In: Long): Unit = ???

  override def relationToGroupKeysExistAndMatch(id: Long, entityId: Long, relTypeId: Long, groupId: Long): Boolean = ???

  override def entityKeyExists(idIn: Long, includeArchived: Boolean): Boolean = ???

  override def createDateAttribute(parentIdIn: Long, attrTypeIdIn: Long, dateIn: Long, sortingIndexIn: Option[Long]): Long = ???

  override def getRelationTypeData(idIn: Long): Array[Option[Any]] = ???

  override def textAttributeKeyExists(idIn: Long): Boolean = ???

  override def getClassName(idIn: Long): Option[String] = ???

  override def createGroupAndRelationToGroup(entityIdIn: Long, relationTypeIdIn: Long, newGroupNameIn: String, allowMixedClassesInGroupIn: Boolean,
                                             validOnDateIn: Option[Long], observationDateIn: Long, sortingIndexIn: Option[Long],
                                             callerManagesTransactionsIn: Boolean): (Long, Long) = ???

  override def addHASRelationToEntity(fromEntityIdIn: Long, toEntityIdIn: Long, validOnDateIn: Option[Long], observationDateIn: Long,
                                      sortingIndexIn: Option[Long]): RelationToEntity = ???

  override def updateRelationToEntity(oldRelationTypeIdIn: Long, entityId1In: Long, entityId2In: Long, newRelationTypeIdIn: Long,
                                      validOnDateIn: Option[Long], observationDateIn: Long): Unit = ???

  override def quantityAttributeKeyExists(idIn: Long): Boolean = ???

  override def updateEntityOnlyNewEntriesStickToTop(idIn: Long, newEntriesStickToTop: Boolean): Unit = ???

  override def getOmInstanceData(idIn: String): Array[Option[Any]] = ???

  override def getSortedAttributes(entityIdIn: Long, startingObjectIndexIn: Int, maxValsIn: Int,
                                   onlyPublicEntitiesIn: Boolean): (Array[(Long, Attribute)], Int) = ???

  override def deleteBooleanAttribute(idIn: Long): Unit = ???

  override def getFileAttributeData(idIn: Long): Array[Option[Any]] = ???

  override def deleteGroupRelationsToItAndItsEntries(groupidIn: Long): Unit = ???

  override def updateEntitysClass(entityId: Long, classId: Option[Long], callerManagesTransactions: Boolean): Unit = ???

  override def getHighestSortingIndexForGroup(groupIdIn: Long): Long = ???

  override def createFileAttribute(parentIdIn: Long, attrTypeIdIn: Long, descriptionIn: String, originalFileDateIn: Long, storedDateIn: Long,
                                   originalFilePathIn: String, readableIn: Boolean, writableIn: Boolean, executableIn: Boolean, sizeIn: Long,
                                   md5hashIn: String, inputStreamIn: FileInputStream, sortingIndexIn: Option[Long]): Long = ???

  override def getTextAttributeData(idIn: Long): Array[Option[Any]] = ???

  override def getClassCount(entityIdIn: Option[Long]): Long = ???

  override def isDuplicateEntity(nameIn: String, selfIdToIgnoreIn: Option[Long]): Boolean = ???

  override def deleteGroupAndRelationsToIt(idIn: Long): Unit = ???

  override def deleteEntity(idIn: Long, callerManagesTransactionsIn: Boolean): Unit = ???

  override def getFileAttributeContent(fileAttributeIdIn: Long, outputStreamIn: OutputStream): (Long, String) = ???

  override def deleteRelationType(idIn: Long): Unit = ???

  override def updateGroup(groupIdIn: Long, nameIn: String, allowMixedClassesInGroupIn: Boolean, newEntriesStickToTopIn: Boolean): Unit = ???

  override def getQuantityAttributeData(idIn: Long): Array[Option[Any]] = ???

  override def getRelationToGroupDataById(idIn: Long): Array[Option[Any]] = ???

  override def groupKeyExists(idIn: Long): Boolean = ???

  override def createBooleanAttribute(parentIdIn: Long, attrTypeIdIn: Long, booleanIn: Boolean, validOnDateIn: Option[Long], observationDateIn: Long,
                                      sortingIndexIn: Option[Long]): Long = ???

  override def createEntity(nameIn: String, classIdIn: Option[Long], isPublicIn: Option[Boolean]): Long = ???

  override def deleteRelationToEntity(relTypeIdIn: Long, entityId1In: Long, entityId2In: Long): Unit = ???

  override def createRelationToRemoteEntity(relationTypeIdIn: Long, entityId1In: Long, entityId2In: Long, validOnDateIn: Option[Long],
                                            observationDateIn: Long, remoteInstanceIdIn: String, sortingIndexIn: Option[Long],
                                            callerManagesTransactionsIn: Boolean): RelationToRemoteEntity = ???

  override def createRelationToGroup(entityIdIn: Long, relationTypeIdIn: Long, groupIdIn: Long, validOnDateIn: Option[Long], observationDateIn: Long,
                                     sortingIndexIn: Option[Long], callerManagesTransactionsIn: Boolean): (Long, Long) = ???

  override def getRelationToGroupData(entityId: Long, relTypeId: Long, groupId: Long): Array[Option[Any]] = ???

  override def getAttrCount(entityIdIn: Long, includeArchivedEntitiesIn: Boolean): Long = ???

  override def relationToRemoteEntityKeyExists(idIn: Long): Boolean = ???

  override def updateTextAttribute(idIn: Long, parentIdIn: Long, attrTypeIdIn: Long, textIn: String, validOnDateIn: Option[Long],
                                   observationDateIn: Long): Unit = ???

  override def fileAttributeKeyExists(idIn: Long): Boolean = ???

  override def createEntityAndRelationToEntity(entityIdIn: Long, relationTypeIdIn: Long, newEntityNameIn: String, isPublicIn: Option[Boolean],
                                               validOnDateIn: Option[Long], observationDateIn: Long,
                                               callerManagesTransactionsIn: Boolean): (Long, Long) = ???

  override def deleteTextAttribute(idIn: Long): Unit = ???

  override def getGroupData(idIn: Long): Array[Option[Any]] = ???

  override def createTextAttribute(parentIdIn: Long, attrTypeIdIn: Long, textIn: String, validOnDateIn: Option[Long], observationDateIn: Long,
                                   callerManagesTransactionsIn: Boolean, sortingIndexIn: Option[Long]): Long = ???

  override def omInstanceKeyExists(idIn: String): Boolean = ???

  override def updateBooleanAttribute(idIn: Long, parentIdIn: Long, attrTypeIdIn: Long, booleanIn: Boolean, validOnDateIn: Option[Long],
                                      inObservationDate: Long): Unit = ???
}
