/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003, 2004, and 2010-2016 inclusive, Luke A Call; all rights reserved.
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
package org.onemodel.core.model

import org.onemodel.core._
import org.onemodel.core.database.PostgreSQLDatabase

object OmInstance {
  def addressLength: Int = PostgreSQLDatabase.omInstanceAddressLength

  def isDuplicate(dbIn: PostgreSQLDatabase, addressIn: String, selfIdToIgnoreIn: Option[String] = None): Boolean = {
    dbIn.isDuplicateOmInstance(addressIn, selfIdToIgnoreIn)
  }

  def create(inDB: PostgreSQLDatabase, idIn: String, addressIn: String, entityIdIn: Option[Long] = None): OmInstance = {
    // Passing false for isLocalIn because the only time that should be true is when it is created at db creation, for this site, and that is done
    // in the db class more directly.
    val insertionDate: Long = inDB.createOmInstance(idIn, isLocalIn = false, addressIn, entityIdIn)
    new OmInstance(inDB, idIn, isLocalIn = false, addressIn = addressIn, insertionDateIn = insertionDate, entityIdIn = entityIdIn)
  }
}

/** See table definition in the database class for details.
  *
  * This 1st constructor instantiates an existing object from the DB. Generally use Model.createObject() to create a new object.
  * Note: Having Entities and other DB objects be readonly makes the code clearer & avoid some bugs, similarly to reasons for immutability in scala.
  */
class OmInstance(mDB: PostgreSQLDatabase, mId: String) {
  if (!mDB.omInstanceKeyExists(mId)) {
    // DON'T CHANGE this msg unless you also change the trap for it in TextUI.java.
    throw new OmException("Key " + mId + " does not exist in database.")
  }


  /** This one is perhaps only called by the database class implementation--so it can return arrays of objects & save more DB hits
    that would have to occur if it only returned arrays of keys. This DOES NOT create a persistent object--but rather should reflect
    one that already exists.
    */
  def this(mDB: PostgreSQLDatabase, mId: String, isLocalIn: Boolean, addressIn: String, insertionDateIn: Long, entityIdIn: Option[Long] = None) {
    this(mDB, mId)
    mLocal = isLocalIn
    mAddress = addressIn
    mInsertionDate = insertionDateIn
    mEntityId = entityIdIn
    mAlreadyReadData = true
  }

  /** When using, consider if getArchivedStatusDisplayString should be called with it in the display (see usage examples of getArchivedStatusDisplayString).
    * */
  def getId: String = {
    if (!mAlreadyReadData) readDataFromDB()
    mId
  }

  def getLocal: Boolean = {
    if (!mAlreadyReadData) readDataFromDB()
    mLocal
  }

  def getCreationDate: Long = {
    if (!mAlreadyReadData) readDataFromDB()
    mInsertionDate
  }

  def getCreationDateFormatted: String = {
    Util.DATEFORMAT.format(new java.util.Date(getCreationDate))
  }

  def getAddress: String = {
    if (!mAlreadyReadData) readDataFromDB()
    mAddress
  }

  def getEntityId: Option[Long] = {
    if (!mAlreadyReadData) readDataFromDB()
    mEntityId
  }

  protected def readDataFromDB() {
    val omInstanceData = mDB.getOmInstanceData(mId)
    mLocal = omInstanceData(0).get.asInstanceOf[Boolean]
    mAddress = omInstanceData(1).get.asInstanceOf[String]
    mInsertionDate = omInstanceData(2).get.asInstanceOf[Long]
    mEntityId = omInstanceData(3).asInstanceOf[Option[Long]]
    mAlreadyReadData = true
  }

  def getDisplayString: String = {
    val result: String = mId + ":" + (if (mLocal) " (local)" else "") + " " + getAddress + ", created on " + getCreationDateFormatted
    result
  }

  def delete() = mDB.deleteOmInstance(mId)

  var mAlreadyReadData: Boolean = false
  var mLocal: Boolean = false
  var mAddress: String = ""
  var mInsertionDate: Long = 0
  var mEntityId: Option[Long] = None
}