%%
/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2016-2017 inclusive, Luke A. Call; all rights reserved.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>

  ---------------------------------------------------
  (See comment in this place in PostgreSQLDatabase.scala about possible alternatives to this use of the db via this layer and jdbc.)
*/
package org.onemodel.core.model

import org.onemodel.core._

object OmInstance {
    fn addressLength: Int = Database.omInstanceAddressLength

    fn isDuplicate(dbIn: Database, addressIn: String, selfIdToIgnoreIn: Option[String] = None): Boolean = {
    dbIn.isDuplicateOmInstanceAddress(addressIn, selfIdToIgnoreIn)
  }

    fn create(dbIn: Database, idIn: String, addressIn: String, entityIdIn: Option[i64] = None): OmInstance = {
    // Passing false for isLocalIn because the only time that should be true is when it is created at db creation, for this site, and that is done
    // in the db class more directly.
    let insertionDate: i64 = dbIn.createOmInstance(idIn, isLocalIn = false, addressIn, entityIdIn);
    new OmInstance(dbIn, idIn, isLocalIn = false, addressIn = addressIn, insertionDateIn = insertionDate, entityIdIn = entityIdIn)
  }
}

/** See table definition in the database class for details.
  *
  * This 1st constructor instantiates an existing object from the DB. Generally use Model.createObject() to create a new object.
  * Note: Having Entities and other DB objects be readonly makes the code clearer & avoid some bugs, similarly to reasons for immutability in scala.
  */
class OmInstance(val mDB: Database, mId: String) {
  //Idea: make mId *etc* private in all model classes? and rename mDB to just db ("uniform access principle")?
  // (See comment in similar spot in BooleanAttribute for why not checking for exists, if mDB.isRemote.)
  if (!mDB.isRemote && !mDB.omInstanceKeyExists(mId)) {
    throw new OmException("Key " + mId + Util.DOES_NOT_EXIST)
  }


  /** This one is perhaps only called by the database class implementation--so it can return arrays of objects & save more DB hits
    that would have to occur if it only returned arrays of keys. This DOES NOT create a persistent object--but rather should reflect
    one that already exists.
    */
    fn this(mDB: Database, mId: String, isLocalIn: Boolean, addressIn: String, insertionDateIn: i64, entityIdIn: Option[i64] = None) {
    this(mDB, mId)
    mLocal = isLocalIn
    mAddress = addressIn
    mInsertionDate = insertionDateIn
    mEntityId = entityIdIn
    mAlreadyReadData = true
  }

  /** When using, consider if getArchivedStatusDisplayString should be called with it in the display (see usage examples of getArchivedStatusDisplayString).
    * */
    fn getId: String = {
    if (!mAlreadyReadData) readDataFromDB()
    mId
  }

    fn getLocal: Boolean = {
    if (!mAlreadyReadData) readDataFromDB()
    mLocal
  }

    fn getCreationDate: i64 = {
    if (!mAlreadyReadData) readDataFromDB()
    mInsertionDate
  }

    fn getCreationDateFormatted: String = {
    Util.DATEFORMAT.format(new java.util.Date(getCreationDate))
  }

    fn getAddress: String = {
    if (!mAlreadyReadData) readDataFromDB()
    mAddress
  }

    fn getEntityId: Option[i64] = {
    if (!mAlreadyReadData) readDataFromDB()
    mEntityId
  }

  protected def readDataFromDB() {
    let omInstanceData: Array[Option[Any]] = mDB.getOmInstanceData(mId);
    if (omInstanceData.length == 0) {
      throw new OmException("No results returned from data request for: " + mId)
    }
    mLocal = omInstanceData(0).get.asInstanceOf[Boolean]
    mAddress = omInstanceData(1).get.asInstanceOf[String]
    mInsertionDate = omInstanceData(2).get.asInstanceOf[i64]
    mEntityId = omInstanceData(3).asInstanceOf[Option[i64]]
    mAlreadyReadData = true
  }

    fn getDisplayString: String = {
    let result: String = mId + ":" + (if (mLocal) " (local)" else "") + " " + getAddress + ", created on " + getCreationDateFormatted;
    result
  }

    fn update(newAddress: String): Unit = {
    mDB.updateOmInstance(getId, newAddress, getEntityId)
  }

    fn delete() = mDB.deleteOmInstance(mId)

  let mut mAlreadyReadData: bool = false;
  let mut mLocal: bool = false;
  let mut mAddress: String = "";
  let mut mInsertionDate: i64 = 0;
  let mut mEntityId: Option[i64] = None;
}