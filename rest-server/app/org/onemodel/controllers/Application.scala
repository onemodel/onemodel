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
package org.onemodel.controllers

import javax.inject._
import akka.util.ByteString
import play.api.mvc._
import play.api.http.HttpEntity

import org.onemodel.controller._

class Application @Inject extends play.api.mvc.Controller {

  def index: Action[AnyContent] = Action { implicit request =>
    Result(
      header = ResponseHeader(200, Map.empty),
      body = HttpEntity.Strict(ByteString("Got request [" + request + "]?: " + org.onemodel.controller.Controller.isWindows), Some("text/plain"))
    )
  }
}
