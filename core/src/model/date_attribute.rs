/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2014-2017 inclusive, and 2023, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
struct DateAttribute {
/*%%
package org.onemodel.core.model

import org.onemodel.core.{OmException, Util}

** See TextAttribute etc for some comments.
  * Also, though this doesn't formally extend Attribute, it still belongs to the same group conceptually (just doesn't have the same date variables so code
  * not shared (idea: model that better, and in FileAttribute).
  *
class DateAttribute(m_db: Database, m_id: i64) extends Attribute(m_db, m_id) {
  // (See comment in similar spot in BooleanAttribute for why not checking for exists, if m_db.is_remote.)
  if !m_db.is_remote && !m_db.date_attribute_key_exists(m_id)) {
    throw new Exception("Key " + m_id + Util::DOES_NOT_EXIST)
  }


  // idea: make the parameter order uniform throughout the system
  /** This one is perhaps only called by the database class implementation--so it can return arrays of objects & save more DB hits
    that would have to occur if it only returned arrays of keys. This DOES NOT create a persistent object--but rather should reflect
    one that already exists.
    */
    fn this(m_db: Database, m_id: i64, inParentId: i64, attr_type_id_in: i64, inDate: i64, sorting_index_in: i64) {
    this(m_db, m_id)
    mDate = inDate
    super.assignCommonVars(inParentId, attr_type_id_in, sorting_index_in)
  }

    fn get_display_string(lengthLimitIn: Int, unused: Option<Entity> = None, unused2: Option[RelationType]=None, simplify: bool = false) -> String {
    let typeName: String = m_db.get_entity_name(get_attr_type_id()).get;
    let mut result: String = typeName + ": ";
    result += Attribute.useful_date_format(mDate)
    Attribute.limitDescriptionLength(result, lengthLimitIn)
  }

    fn getDate -> i64 {
    if !m_already_read_data) read_data_from_db()
    mDate
  }

  protected fn read_data_from_db() {
    let daTypeData = m_db.get_date_attribute_data(m_id);
    if daTypeData.length == 0) {
      throw new OmException("No results returned from data request for: " + m_id)
    }
    mDate = daTypeData(1).get.asInstanceOf[i64]
    assignCommonVars(daTypeData(0).get.asInstanceOf[i64], daTypeData(2).get.asInstanceOf[i64], daTypeData(3).get.asInstanceOf[i64])
  }

    fn update(inAttrTypeId: i64, inDate: i64) {
    // write it to the database table--w/ a record for all these attributes plus a key indicating which Entity
    // it all goes with
    m_db.update_date_attribute(m_id, get_parent_id(), inDate, inAttrTypeId)
    mDate = inDate
    m_attr_type_id = inAttrTypeId
  }

  /** Removes this object from the system. */
    fn delete() {
    m_db.delete_date_attribute(m_id)
    }

  /** For descriptions of the meanings of these variables, see the comments
    with create_date_attribute(...) or create_tables() in PostgreSQLDatabase or Database classes
    */
    private let mut mDate: i64 = 0L;
 */
}