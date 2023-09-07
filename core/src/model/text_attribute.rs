/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003, 2004, 2010, 2011, 2013-2017 inclusive, and 2023, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
struct TextAttribute {
/*%%
package org.onemodel.core.model

import org.onemodel.core.{OmException, Util}

* Represents one String object in the system (usually [always, as of 9/2002] used as an attribute on a Entity).

    This constructor instantiates an existing object from the DB. You can use Entity.addTextAttribute() to
    create a new object.
  *
class TextAttribute(db: Database, id: i64) extends AttributeWithValidAndObservedDates(db, id) {
  // (See comment in similar spot in BooleanAttribute for why not checking for exists, if db.is_remote.)
  if !db.is_remote && !db.text_attribute_key_exists(id)) {
    throw new Exception("Key " + id + Util::DOES_NOT_EXIST)
  }


  /** This one is perhaps only called by the database class implementation--so it can return arrays of objects & save more DB hits
    that would have to occur if it only returned arrays of keys. This DOES NOT create a persistent object--but rather should reflect
    one that already exists.
    */
    fn this(db: Database, id: i64, parent_id_in: i64, attr_type_id_in: i64, text_in: String, valid_on_date: Option<i64>, observation_date: i64,
           sorting_index_in: i64) {
    this(db, id)
    assign_common_vars(parent_id_in, attr_type_id_in, valid_on_date, observation_date, sorting_index_in)
    mText = text_in
  }

  /** return some string. See comments on QuantityAttribute.get_display_string regarding the parameters.
    */
    fn get_display_string(length_limit_in: Int, unused: Option<Entity> = None, unused2: Option[RelationType]=None, simplify: bool = false) -> String {
    let type_name: String = db.get_entity_name(get_attr_type_id()).get;
    let mut result: String = {;
      if simplify && (type_name == "paragraph" || type_name == "quote")) get_text
      else type_name + ": \"" + get_text + "\""
    }
    if ! simplify) result += "; " + get_dates_description
    Attribute.limit_attribute_description_length(result, length_limit_in)
  }

    fn get_text -> String {
    if !already_read_data) read_data_from_db()
    mText
  }

  protected fn read_data_from_db() {
    let taTypeData = db.get_text_attribute_data(id);
    if taTypeData.length == 0) {
      throw new OmException("No results returned from data request for: " + id)
    }
    mText = taTypeData(1).get.asInstanceOf[String]
    super.assign_common_vars(taTypeData(0).get.asInstanceOf[i64], taTypeData(2).get.asInstanceOf[i64], taTypeData(3).asInstanceOf[Option<i64>],
                           taTypeData(4).get.asInstanceOf[i64], taTypeData(5).get.asInstanceOf[i64])
  }

    fn update(attr_type_id_in: i64, text_in: String, valid_on_date_in: Option<i64>, observation_date_in: i64) {
    // write it to the database table--w/ a record for all these attributes plus a key indicating which Entity
    // it all goes with
    db.update_text_attribute(id, get_parent_id(), attr_type_id_in, text_in, valid_on_date_in, observation_date_in)
    mText = text_in
    attr_type_id = attr_type_id_in
    valid_on_date = valid_on_date_in
    observation_date = observation_date_in
  }

  /** Removes this object from the system. */
    fn delete() = db.delete_text_attribute(id)

  /** For descriptions of the meanings of these variables, see the comments
    on create_text_attribute(...) or create_tables() in PostgreSQLDatabase or Database classes.
    */
    private let mut mText: String = null;
 */
}
