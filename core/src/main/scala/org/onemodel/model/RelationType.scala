/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2004, 2010, 2011, and 2013-2016 inclusive, Luke A. Call; all rights reserved.
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

import org.onemodel.database.PostgreSQLDatabase

/** Represents one RelationType object in the system.
  */
object RelationType {
  def getNameLength(inDB: PostgreSQLDatabase): Int = {
    PostgreSQLDatabase.relationTypeNameLength
  }

  // idea: should use these more, elsewhere (replacing hard-coded values! )
  val BIDIRECTIONAL: String = "BI"
  val UNIDIRECTIONAL: String = "UNI"
  val NONDIRECTIONAL: String = "NON"
}

/** This constructor instantiates an existing object from the DB. You can use Entity.addRelationTypeAttribute() to
    create a new object. Assumes caller just read it from the DB and the info is accurate (i.e., this may only ever need to be called by
    a Database instance?).
  */
class RelationType(mDB: PostgreSQLDatabase, mId: Long) extends Entity(mDB, mId) {
  if (!mDB.relationTypeKeyExists(mId)) {
    // DON'T CHANGE this msg unless you also change the trap for it, if used, in other code.
    throw new Exception("Key " + mId + " does not exist in database.")
  }


  /** This one is perhaps only called by the database class implementation--so it can return arrays of objects & save more DB hits
    that would have to occur if it only returned arrays of keys. This DOES NOT create a persistent object--but rather should reflect
    one that already exists.
    */
  private[onemodel] def this(inDB: PostgreSQLDatabase, inEntityId: Long, inName: String, inNameInReverseDirection: String,
                             inDirectionality: String) {
    this(inDB, inEntityId)
    mName = inName
    mNameInReverseDirection = inNameInReverseDirection
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
    val relationTypeData: Array[Option[Any]] = mDB.getRelationTypeData(mId)
    mName = relationTypeData(0).get.asInstanceOf[String]
    mNameInReverseDirection = relationTypeData(1).get.asInstanceOf[String]
    mDirectionality = relationTypeData(2).get.asInstanceOf[String].trim
    mAlreadyReadData = true
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