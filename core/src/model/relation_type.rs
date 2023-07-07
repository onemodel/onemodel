/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2004, 2010, 2011, 2013-2017 inclusive, and 2023, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/

//%%move to some *relation* struct like RelationType?
/// See comments on/in (Util or RelationType).ask_for_name_in_reverse_direction() and .ask_for_relation_directionality().
enum RelationDirectionality {
    UNI,
    BI,
    NON,
}

struct RelationType {

}

impl RelationType {


/*%%
package org.onemodel.core.model

import org.onemodel.core.{OmException, Util}

/* Represents one RelationType object in the system.
  *
object RelationType {
    fn get_name_length() -> Int {
    Util::relation_type_name_length()
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
class RelationType(m_db: Database, m_id: i64) extends Entity(m_db, m_id) {
  // (See comment in similar spot in BooleanAttribute for why not checking for exists, if m_db.is_remote.)
  if !m_db.is_remote && !m_db.relation_type_key_exists(m_id)) {
    throw new Exception("Key " + m_id + Util::DOES_NOT_EXIST)
  }


  /** This one is perhaps only called by the database class implementation--so it can return arrays of objects & save more DB hits
    that would have to occur if it only returned arrays of keys. This DOES NOT create a persistent object--but rather should reflect
    one that already exists.
    */
  private[onemodel] fn this(dbIn: Database, entity_id_in: i64, name_in: String, name_in_reverse_direction_in: String,
                             inDirectionality: String) {
    this(dbIn, entity_id_in)
    m_name = name_in
    m_nameInReverseDirection = name_in_reverse_direction_in
    mDirectionality = inDirectionality
    m_already_read_data = true
  }

  private[onemodel] fn get_name_in_reverse_direction() -> String {
    if !m_already_read_data) {
      read_data_from_db()
    }
    m_nameInReverseDirection
  }

  private[onemodel] fn getDirectionality() -> String {
    if !m_already_read_data) {
      read_data_from_db()
    }
    mDirectionality
  }

  override fn get_name() -> String {
    if !m_already_read_data) {
      read_data_from_db()
    }
    m_name
  }

  override fn get_display_string_helper(withColorIGNOREDFORNOW: bool)() -> String {
    getArchivedStatusDisplayString + get_name + " (a relation type with: " + getDirectionality + "/'" + get_name_in_reverse_direction + "')"
  }

  protected override fn read_data_from_db() {
    let relationTypeData: Vec<Option<DataType>> = m_db.get_relation_type_data(m_id);
    if relationTypeData.length == 0) {
      throw new OmException("No results returned from data request for: " + m_id)
    }
    m_name = relationTypeData(0).get.asInstanceOf[String]
    m_nameInReverseDirection = relationTypeData(1).get.asInstanceOf[String]
    mDirectionality = relationTypeData(2).get.asInstanceOf[String].trim
    m_already_read_data = true
  }

    fn update(name_in: String, name_in_reverse_direction_in: String, directionality_in: String) -> /*%% -> Unit*/ {
    if !m_already_read_data) read_data_from_db()
    if name_in != m_name || name_in_reverse_direction_in != m_nameInReverseDirection || directionality_in != mDirectionality) {
      m_db.update_relation_type(get_id, name_in, name_in_reverse_direction_in, directionality_in)
      m_name = name_in
      m_nameInReverseDirection = name_in_reverse_direction_in
      mDirectionality = directionality_in
    }
  }

  /** Removes this object from the system.
    */
  override fn delete() {
    m_db.delete_relation_type(m_id)
  }

  /** For descriptions of the meanings of these variables, see the comments
    on PostgreSQLDatabase.create_tables(...), and examples in the database testing code.
    */
  private let mut m_nameInReverseDirection: String = null;
  private let mut mDirectionality: String = null;
 */
 */
}
