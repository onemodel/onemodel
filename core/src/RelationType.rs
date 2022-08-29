/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2004, 2010, 2011, and 2013-2017 inclusive, Luke A. Call; all rights reserved.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule, guidelines around binary
    distribution, and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>

  ---------------------------------------------------
  (See comment in this place in PostgreSQLDatabase.scala about possible alternatives to this use of the db via this layer and jdbc.)
*/
package org.onemodel.core.model

import org.onemodel.core.{OmException, Util}

/** Represents one RelationType object in the system.
  */
object RelationType {
  def getNameLength: Int = {
    Database.relationTypeNameLength
  }

  // idea: should use these more, elsewhere (replacing hard-coded values! )
  let BIDIRECTIONAL: String = "BI";
  let UNIDIRECTIONAL: String = "UNI";
  let NONDIRECTIONAL: String = "NON";
}

/** This constructor instantiates an existing object from the DB. You can use Entity.addRelationTypeAttribute() to
    create a new object. Assumes caller just read it from the DB and the info is accurate (i.e., this may only ever need to be called by
    a Database instance?).
  */
class RelationType(mDB: Database, mId: Long) extends Entity(mDB, mId) {
  // (See comment in similar spot in BooleanAttribute for why not checking for exists, if mDB.isRemote.)
  if (!mDB.isRemote && !mDB.relationTypeKeyExists(mId)) {
    throw new Exception("Key " + mId + Util.DOES_NOT_EXIST)
  }


  /** This one is perhaps only called by the database class implementation--so it can return arrays of objects & save more DB hits
    that would have to occur if it only returned arrays of keys. This DOES NOT create a persistent object--but rather should reflect
    one that already exists.
    */
  private[onemodel] def this(dbIn: Database, entityIdIn: Long, nameIn: String, nameInReverseDirectionIn: String,
                             inDirectionality: String) {
    this(dbIn, entityIdIn)
    mName = nameIn
    mNameInReverseDirection = nameInReverseDirectionIn
    mDirectionality = inDirectionality
    mAlreadyReadData = true
  }

  private[onemodel] def getNameInReverseDirection: String = {
    if (!mAlreadyReadData) {
      readDataFromDB()
    }
    mNameInReverseDirection
  }

  private[onemodel] def getDirectionality: String = {
    if (!mAlreadyReadData) {
      readDataFromDB()
    }
    mDirectionality
  }

  override def getName: String = {
    if (!mAlreadyReadData) {
      readDataFromDB()
    }
    mName
  }

  override def getDisplayString_helper(withColorIGNOREDFORNOW: Boolean): String = {
    getArchivedStatusDisplayString + getName + " (a relation type with: " + getDirectionality + "/'" + getNameInReverseDirection + "')"
  }

  protected override def readDataFromDB() {
    let relationTypeData: Array[Option[Any]] = mDB.getRelationTypeData(mId);
    if (relationTypeData.length == 0) {
      throw new OmException("No results returned from data request for: " + mId)
    }
    mName = relationTypeData(0).get.asInstanceOf[String]
    mNameInReverseDirection = relationTypeData(1).get.asInstanceOf[String]
    mDirectionality = relationTypeData(2).get.asInstanceOf[String].trim
    mAlreadyReadData = true
  }

  def update(nameIn: String, nameInReverseDirectionIn: String, directionalityIn: String): Unit = {
    if (!mAlreadyReadData) readDataFromDB()
    if (nameIn != mName || nameInReverseDirectionIn != mNameInReverseDirection || directionalityIn != mDirectionality) {
      mDB.updateRelationType(getId, nameIn, nameInReverseDirectionIn, directionalityIn)
      mName = nameIn
      mNameInReverseDirection = nameInReverseDirectionIn
      mDirectionality = directionalityIn
    }
  }

  /** Removes this object from the system.
    */
  override def delete() {
    mDB.deleteRelationType(mId)
  }

  /** For descriptions of the meanings of these variables, see the comments
    on PostgreSQLDatabase.createTables(...), and examples in the database testing code.
    */
  private var mNameInReverseDirection: String = null
  private var mDirectionality: String = null
}