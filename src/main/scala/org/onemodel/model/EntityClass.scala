/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2013-2014 inclusive, Luke A Call; all rights reserved.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule, guidelines around binary
    distribution, and the GNU Affero General Public License as published by the Free Software Foundation, either version 3
    of the License, or (at your option) any later version.  See the file LICENSE for details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>

  ---------------------------------------------------
  If we ever do port to another database, create the Database interface (removed around 2014-1-1 give or take) and see other changes at that time.
  An alternative method is to use jdbc escapes (but this actually might be even more work?):  http://jdbc.postgresql.org/documentation/head/escapes.html  .
  Another alternative is a layer like JPA, ibatis, hibernate  etc etc.

*/
package org.onemodel.model

import java.io.{PrintWriter, StringWriter}
import scala.Predef._
import org.onemodel.database.PostgreSQLDatabase

object EntityClass {
  def nameLength(inDB: PostgreSQLDatabase): Int = inDB.classNameLength

  def isDuplicate(inDB: PostgreSQLDatabase, inName: String, inSelfIdToIgnore: Option[Long] = None): Boolean = inDB.isDuplicateClass(inName, inSelfIdToIgnore)
}

class EntityClass(mDB: PostgreSQLDatabase, mId: Long) {
  if (!mDB.classKeyExists(mId)) {
    // DON'T CHANGE this msg unless you also change the trap for it in TextUI.java.
    throw new Exception("Key " + mId + " does not exist in database.")
  }

  /** This one is perhaps only called by the database class implementation--so it can return arrays of objects & save more DB hits
    that would have to occur if it only returned arrays of keys. This DOES NOT create a persistent object--but rather should reflect
    one that already exists.
    */
  def this(mDB: PostgreSQLDatabase, mId: Long, inName: String, inDefiningEntityId: Long) {
    this(mDB, mId)
    mName = inName
    mDefiningEntityId = inDefiningEntityId
    mAlreadyReadData = true
  }

  def getName: String = {
    if (!mAlreadyReadData) readDataFromDB()
    mName
  }

  def getDefiningEntityId: Long = {
    if (!mAlreadyReadData) readDataFromDB()
    mDefiningEntityId
  }

  protected def readDataFromDB() {
    val classData = mDB.getClassData(mId)
    mName = classData(0).get.asInstanceOf[String]
    mDefiningEntityId = classData(1).get.asInstanceOf[Long]
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
  def delete() = mDB.deleteClassAndItsDefiningEntity(mId)

  var mAlreadyReadData: Boolean = false
  var mName: String = null
  var mDefiningEntityId: Long = 0
}
