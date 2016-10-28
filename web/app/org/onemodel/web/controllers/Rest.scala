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
