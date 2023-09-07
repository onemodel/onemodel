/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003, 2004, 2010, 2011, 2013-2017 inclusive, and 2023, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
struct QuantityAttribute {
/*%%
package org.onemodel.core.model

import org.onemodel.core.{OmException, Util}

** Represents one quantity object in the system (usually [always, as of 9/2002] used as an attribute on a Entity).
  *
  * This constructor instantiates an existing object from the DB. You can use Entity.addQuantityAttribute() to
  * create a new object.
  *
class QuantityAttribute(m_db: Database, m_id: i64) extends AttributeWithValidAndObservedDates(m_db, m_id) {
  // (See comment in similar spot in BooleanAttribute for why not checking for exists, if m_db.is_remote.)
  if !m_db.is_remote && !m_db.relation_type_key_exists(m_id)) {
    throw new Exception("Key " + m_id + Util::DOES_NOT_EXIST)
  }

  /**
   * This one is perhaps only called by the database class implementation--so it can return arrays of objects & save more DB hits
   * that would have to occur if it only returned arrays of keys. This DOES NOT create a persistent object--but rather should reflect
   * one that already exists.
   */
    fn this(db: Database, id: i64, parent_id_in: i64, attr_type_id_in: i64, unit_id_in: i64, number_in: Float, valid_on_date: Option<i64>,
           observation_date: i64, sorting_index: i64) {
    this(db, id)
    mUnitId = unit_id_in
    mNumber = number_in
    assign_common_vars(parent_id_in, attr_type_id_in, valid_on_date, observation_date, sorting_index)
  }

  /**
   * return something like "volume: 15.1 liters". For full length, pass in 0 for
   * in_length_limit. The parameter inParentEntity refers to the Entity whose
   * attribute this is. 3rd parameter really only applies in one of the subclasses of Attribute,
   * otherwise can be None.
   */
    fn get_display_string(length_limit_in: Int, unused: Option<Entity>=None, unused2: Option[RelationType]=None, simplify: bool = false) -> String {
    let type_name: String = m_db.get_entity_name(get_attr_type_id()).get;
    let number: Float = getNumber;
    let unitId: i64 = getUnitId;
    let mut result: String = type_name + ": " + number + " " + m_db.get_entity_name(unitId).get;
    if ! simplify) result += "; " + get_dates_description
    Attribute.limit_attribute_description_length(result, length_limit_in)
  }

  private[onemodel] fn getNumber -> Float {
    if !m_already_read_data) read_data_from_db()
    mNumber
  }

  private[onemodel] fn getUnitId -> i64 {
    if !m_already_read_data) read_data_from_db()
    mUnitId
  }

  protected fn read_data_from_db() {
    let quantityData = m_db.get_quantity_attribute_data(m_id);
    if quantityData.length == 0) {
      throw new OmException("No results returned from data request for: " + m_id)
    }
    mUnitId = quantityData(1).get.asInstanceOf[i64]
    mNumber = quantityData(2).get.asInstanceOf[Float]
    assign_common_vars(quantityData(0).get.asInstanceOf[i64], quantityData(3).get.asInstanceOf[i64], quantityData(4).asInstanceOf[Option<i64>],
                           quantityData(5).get.asInstanceOf[i64], quantityData(6).get.asInstanceOf[i64])
  }

    fn update(attr_type_id_in: i64, unit_id_in: i64, number_in: Float, valid_on_date_in: Option<i64>, observation_date_in: i64) {
    // write it to the database table--w/ a record for all these attributes plus a key indicating which Entity
    // it all goes with
    m_db.update_quantity_attribute(m_id, get_parent_id(), attr_type_id_in, unit_id_in, number_in, valid_on_date_in, observation_date_in)
    m_attr_type_id = attr_type_id_in
    mUnitId = unit_id_in
    mNumber = number_in
    valid_on_date = valid_on_date_in
    observation_date = observation_date_in
  }

  /** Removes this object from the system. */
    fn delete() {
    m_db.delete_quantity_attribute(m_id)
    }

  // **idea: make these members into vals not vars, by replacing them with the next line.
  //           private let (unitId: i64, number: Float) = read_data_from_db();
  // BUT: have to figure out how to work with the
  // assignment from the other constructor, and passing vals to the superclass to be...vals.  Need to know scala better,
  // like how additional class vals are set when the other constructor (what's the term again?), is called. How to do the other constructor w/o a db hit.
  /**
   * For descriptions of the meanings of these variables, see the comments
   * on create_quantity_attribute(...) or create_tables() in PostgreSQLDatabase or Database classes
   */
  private let mut mUnitId: i64 = 0L;
  private let mut mNumber: Float = .0F;
 */
}
