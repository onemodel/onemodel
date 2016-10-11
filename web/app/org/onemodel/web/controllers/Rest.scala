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
import org.onemodel.core.database.PostgreSQLDatabase
import org.onemodel.core.model._
import play.api.mvc._
import play.api.http.HttpEntity

class Rest extends play.api.mvc.Controller {
  def id: Action[AnyContent] = Action { implicit request =>
    val systemUserName = System.getProperty("user.name")
    val defaultPassword = "x"
    val db = new PostgreSQLDatabase(systemUserName, defaultPassword)
    val inst: OmInstance = db.getLocalOmInstanceData
    val msg = "Instance " + inst.getDisplayString
    Result(
            header = ResponseHeader(200, Map.empty),
            body = HttpEntity.Strict(ByteString(msg), Some("text/plain"))
          )
  }

  def entity(idIn: Long): Action[AnyContent] = Action { implicit request =>
    val systemUserName = System.getProperty("user.name")
    val defaultPassword = "x"
    val db = new PostgreSQLDatabase(systemUserName, defaultPassword)
    val exists: Boolean = db.entityOnlyKeyExists(idIn)
    if (! exists) {
      val msg: String = "Entity " + idIn + " was not found."
      NotFound(msg)
    } else {
      val entity = new Entity(db, idIn)
      val public: Option[Boolean] = entity.getPublic
      if (public.isDefined && public.get) {
        val msg = "Entity info: " + entity.getDisplayString(withColor = false) +
                  "\n\nGot request [" + request + "]"
        Result(
                header = ResponseHeader(200, Map.empty),
                body = HttpEntity.Strict(ByteString(msg), Some("text/plain"))
              )
      } else {
        val msg: String = "Entity " + idIn + " is not public."
        //idea: look in http response codes: is there one that makes more sense than this, for this use?:
        NotFound(msg)
      }
    }
  }

}
