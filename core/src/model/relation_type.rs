/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2004, 2010, 2011, 2013-2017 inclusive, and 2023, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
struct RelationType {
/*%%
package org.onemodel.core.model

import org.onemodel.core.{OmException, Util}

/* Represents one RelationType object in the system.
  *
object RelationType {
    fn getNameLength() -> Int {
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
class RelationType(mDB: Database, mId: i64) extends Entity(mDB, mId) {
  // (See comment in similar spot in BooleanAttribute for why not checking for exists, if mDB.is_remote.)
  if (!mDB.is_remote && !mDB.relationTypeKeyExists(mId)) {
    throw new Exception("Key " + mId + Util.DOES_NOT_EXIST)
  }


  /** This one is perhaps only called by the database class implementation--so it can return arrays of objects & save more DB hits
    that would have to occur if it only returned arrays of keys. This DOES NOT create a persistent object--but rather should reflect
    one that already exists.
    */
  private[onemodel] fn this(dbIn: Database, entityIdIn: i64, nameIn: String, nameInReverseDirectionIn: String,
                             inDirectionality: String) {
    this(dbIn, entityIdIn)
    mName = nameIn
    mNameInReverseDirection = nameInReverseDirectionIn
    mDirectionality = inDirectionality
    mAlreadyReadData = true
  }

  private[onemodel] fn getNameInReverseDirection() -> String {
    if (!mAlreadyReadData) {
      readDataFromDB()
    }
    mNameInReverseDirection
  }

  private[onemodel] fn getDirectionality() -> String {
    if (!mAlreadyReadData) {
      readDataFromDB()
    }
    mDirectionality
  }

  override fn getName() -> String {
    if (!mAlreadyReadData) {
      readDataFromDB()
    }
    mName
  }

  override fn getDisplayString_helper(withColorIGNOREDFORNOW: Boolean)() -> String {
    getArchivedStatusDisplayString + getName + " (a relation type with: " + getDirectionality + "/'" + getNameInReverseDirection + "')"
  }

  protected override fn readDataFromDB() {
    let relationTypeData: Array[Option[Any]] = mDB.getRelationTypeData(mId);
    if (relationTypeData.length == 0) {
      throw new OmException("No results returned from data request for: " + mId)
    }
    mName = relationTypeData(0).get.asInstanceOf[String]
    mNameInReverseDirection = relationTypeData(1).get.asInstanceOf[String]
    mDirectionality = relationTypeData(2).get.asInstanceOf[String].trim
    mAlreadyReadData = true
  }

    fn update(nameIn: String, nameInReverseDirectionIn: String, directionalityIn: String) -> /*%% -> Unit*/ {
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
  override fn delete() {
    mDB.deleteRelationType(mId)
  }

  /** For descriptions of the meanings of these variables, see the comments
    on PostgreSQLDatabase.createTables(...), and examples in the database testing code.
    */
  private let mut mNameInReverseDirection: String = null;
  private let mut mDirectionality: String = null;
 */
 */
}
