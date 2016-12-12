/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2013-2016 inclusive, Luke A Call; all rights reserved.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule, guidelines around binary
    distribution, and the GNU Affero General Public License as published by the Free Software Foundation, either version 3
    of the License, or (at your option) any later version.  See the file LICENSE for details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>

  ---------------------------------------------------
  (See comment in this place in PostgreSQLDatabase.scala about possible alternatives to this use of the db via this layer and jdbc.)
*/
package org.onemodel.core.model

import java.io.{PrintWriter, StringWriter}
import org.onemodel.core.{OmException, Util}
import org.onemodel.core.database.Database

object EntityClass {
  def nameLength(inDB: Database): Int = Database.classNameLength

  def isDuplicate(inDB: PostgreSQLDatabase, inName: String, inSelfIdToIgnore: Option[Long] = None): Boolean = inDB.isDuplicateClass(inName, inSelfIdToIgnore)
}

class EntityClass(mDB: Database, mId: Long) {
  // (See comment at similar location in BooleanAttribute.)
  if (!mDB.isRemote && !mDB.classKeyExists(mId)) {
    throw new Exception("Key " + mId + Util.DOES_NOT_EXIST)
  }

  /** This one is perhaps only called by the database class implementation--so it can return arrays of objects & save more DB hits
    that would have to occur if it only returned arrays of keys. This DOES NOT create a persistent object--but rather should reflect
    one that already exists.
    */
  def this(mDB: Database, mId: Long, inName: String, inTemplateEntityId: Long, createDefaultAttributesIn: Option[Boolean] = None) {
    this(mDB, mId)
    mName = inName
    mTemplateEntityId = inTemplateEntityId
    mCreateDefaultAttributes = createDefaultAttributesIn
    mAlreadyReadData = true
  }

  def getName: String = {
    if (!mAlreadyReadData) readDataFromDB()
    mName
  }

  def getTemplateEntityId: Long = {
    if (!mAlreadyReadData) readDataFromDB()
    mTemplateEntityId
  }

  def getCreateDefaultAttributes: Option[Boolean] = {
    if (!mAlreadyReadData) readDataFromDB()
    mCreateDefaultAttributes
  }

  protected def readDataFromDB() {
    val classData: Array[Option[Any]] = mDB.getClassData(mId)
    if (classData.length == 0) {
      throw new OmException("No results returned from data request for: " + mId)
    }
    mName = classData(0).get.asInstanceOf[String]
    mTemplateEntityId = classData(1).get.asInstanceOf[Long]
    mCreateDefaultAttributes = classData(2).asInstanceOf[Option[Boolean]]
    mAlreadyReadData = true
  }

  def getIdWrapper: IdWrapper = new IdWrapper(mId)

  def getId: Long = mId

  def getDisplayString_helper: String = {
    getName
  }

  def getDisplayString: String = {
    var result = ""
    try {
      result = getDisplayString_helper
    } catch {
      case e: Exception =>
        result += "Unable to get class description due to: "
        result += {
          val sw: StringWriter = new StringWriter()
          e.printStackTrace(new PrintWriter(sw))
          sw.toString
        }
    }
    result
  }

  /** Removes this object etc from the system. */
  def delete() = mDB.deleteClassAndItsTemplateEntity(mId)

  var mAlreadyReadData: Boolean = false
  var mName: String = null
  var mTemplateEntityId: Long = 0
  var mCreateDefaultAttributes: Option[Boolean] = None
}
