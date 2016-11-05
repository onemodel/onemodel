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

import akka.util.ByteString
import play.api.mvc._
import play.api.http.HttpEntity
import play.api.libs.json._

import org.onemodel.core._
import org.onemodel.core.database.PostgreSQLDatabase
import org.onemodel.core.model._

/*: IDEAS: stuff to do then delete these comments, from: http://www.vinaysahni.com/best-practices-for-a-pragmatic-restful-api :
"Errors: Just like an HTML error page shows a useful error message to a visitor, an API should provide a useful error message in a known consumable format. The representation of an error should be no different than the representation of any resource, just with its own set of fields. "...with consumable JSON error representation.... A JSON error body should provide a few things for the developer - a useful error message, a unique error code (that can be looked up for more details in the docs) and possibly a detailed description. JSON output representation for something like this would look like:
{
  "code" : 1234,
  "message" : "Something bad happened :(",
  "description" : "More details about the error here"
}
Validation errors for PUT, PATCH and POST requests will need a field breakdown. This is best modeled by using a fixed top-level error code for validation failures and providing the detailed errors in an additional errors field, like so:
{
  "code" : 1024,
  "message" : "Validation Failed",
  "errors" : [
    {
      "code" : 5432,
      "field" : "first_name",
      "message" : "First name cannot have fancy characters"
    },
    {
       "code" : 5622,
       "field" : "password",
       "message" : "Password cannot be blank"
    }
  ]
}

"An API that accepts JSON encoded POST, PUT & PATCH requests should also require the Content-Type header be set to application/json or throw a 415 Unsupported Media Type HTTP status code."

"To prevent abuse, it is standard practice to add some sort of rate limiting to an API. RFC 6585 introduced a HTTP status code 429 Too Many Requests to accommodate this."

"HTTP defines a bunch of meaningful status codes that can be returned from your API. These can be leveraged to help the API consumers route their responses accordingly. I've curated a short list of the ones that you definitely should be using:
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
422 Unprocessable Entity - Used for validation errors
429 Too Many Requests - When a request is rejected due to rate limiting"

*/

class Rest extends play.api.mvc.Controller {
  val (user, pass) = Util.getDefaultUserInfo
  val db = new PostgreSQLDatabase(user, pass)

  def id: Action[AnyContent] = Action { implicit request =>
    val inst: OmInstance = db.getLocalOmInstanceData
    val msg = new JsString(inst.getId)
    Ok(msg)
  }

  implicit val entityWrites = new Writes[Entity] {
    def writes(entityIn: Entity) = Json.obj(
      "id" -> entityIn.getId,
      "name" -> entityIn.getName,
      // This one says null (json has:  ...,"classId":null,... ) when the value is NULL in the db (as could "public" below, though currently the
      // endpoint returns an error instead, if the entity has anything but TRUE for public in the db).
      "classId" -> entityIn.getClassId,
      // could add one that uses ISO8601 for other clients besides OM itself:
      "insertionDate" -> entityIn.getInsertionDate,
      "public" -> entityIn.getPublic,
      "archived" -> entityIn.getArchivedStatus,
      "newEntriesStickToTop" -> entityIn.getNewEntriesStickToTop
    )
  }

  def entities(idIn: Long): Action[AnyContent] = Action { implicit request =>
    val exists: Boolean = db.entityOnlyKeyExists(idIn)
    if (!exists) {
      val msg: String = "Entity " + idIn + " was not found."
      NotFound(msg)
    } else {
      val entity = new Entity(db, idIn)
      val public: Option[Boolean] = entity.getPublic
      if (public.isDefined && public.get) {
        val json = Json.toJson(entity)
        // the ".as(JSON)" seems optional, but for reference:
        Ok(json).as(JSON)
//        Result(
//                header = ResponseHeader(200, Map.empty),
//                body = HttpEntity.Strict(ByteString(msg), Some("text/plain"))
//              )
      } else {
        val msg: String = "Entity " + idIn + " is not public."
        Forbidden(msg)
      }
    }
  }

  def defaultEntity = Action { implicit request =>
    var defaultEntityId: Option[Long] = db.getUserPreference_EntityId(Util.DEFAULT_ENTITY_PREFERENCE)
    if (defaultEntityId.isDefined) {
      val entity = new Entity(db, defaultEntityId.get)
      val json = Json.toJson(entity)
      Ok(json).as(JSON)
    } else {
      val msg: String = "A default entity preference was not found."
      NotFound(msg)
    }
  }

}
