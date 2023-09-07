/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2013-2017 inclusive, and 2023, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
struct EntityClass {
/*%%
package org.onemodel.core.model

import java.io.{PrintWriter, StringWriter}
import org.onemodel.core.{OmException, Util}

object EntityClass {
    fn name_length(db_in: Database) -> Int {
     Database.class_name_length
     }

    fn isDuplicate(db_in: Database, inName: String, inSelfIdToIgnore: Option<i64> = None) -> bool {
    db_in.is_duplicate_class_name(inName, inSelfIdToIgnore)
    }
}

class EntityClass(val db: Database, id: i64) {
  // (See comment in similar spot in BooleanAttribute for why not checking for exists, if db.is_remote.)
  if !db.is_remote && !db.class_key_exists(id)) {
    throw new Exception("Key " + id + Util::DOES_NOT_EXIST)
  }

  /** This one is perhaps only called by the database class implementation--so it can return arrays of objects & save more DB hits
    that would have to occur if it only returned arrays of keys. This DOES NOT create a persistent object--but rather should reflect
    one that already exists.
    */
    fn this(db: Database, id: i64, inName: String, inTemplateEntityId: i64, createDefaultAttributesIn: Option<bool> = None) {
    this(db, id)
    m_name = inName
    mTemplateEntityId = inTemplateEntityId
    mCreateDefaultAttributes = createDefaultAttributesIn
    already_read_data = true
  }

    fn get_name -> String {
    if !already_read_data) read_data_from_db()
    m_name
  }

    fn get_template_entity_id -> i64 {
    if !already_read_data) read_data_from_db()
    mTemplateEntityId
  }


    fn getCreateDefaultAttributes -> Option<bool> {
    if !already_read_data) read_data_from_db()
    mCreateDefaultAttributes
  }

  protected fn read_data_from_db() {
    let classData: Vec<Option<DataType>> = db.get_class_data(id);
    if classData.length == 0) {
      throw new OmException("No results returned from data request for: " + id)
    }
    m_name = classData(0).get.asInstanceOf[String]
    mTemplateEntityId = classData(1).get.asInstanceOf[i64]
    mCreateDefaultAttributes = classData(2).asInstanceOf[Option<bool>]
    already_read_data = true
  }

    fn get_id_wrapper -> IdWrapper {
     new IdWrapper(id)
     }

    fn get_id -> i64 {
    id
    }

    fn get_display_string_helper -> String {
    get_name
  }

    fn get_display_string -> String {
    let mut result = "";
    try {
      result = get_display_string_helper
    } catch {
      case e: Exception =>
        result += "Unable to get class description due to: "
        result += {
          let sw: StringWriter = new StringWriter();
          e.printStackTrace(new PrintWriter(sw))
          sw.toString
        }
    }
    result
  }

    fn update_class_and_template_entity_name(name_in: String) -> i64 {
    let template_entity_id = db.update_class_and_template_entity_name(this.get_id, name_in);
    m_name = name_in
    require(template_entity_id == get_template_entity_id)
    template_entity_id
  }

    fn updateCreateDefaultAttributes(value_in: Option<bool>) /*%%-> Unit*/ {
    db.update_class_create_default_attributes(get_id, value_in)
    mCreateDefaultAttributes = value_in
  }

  /** Removes this object etc from the system. */
    fn delete() {
    db.delete_class_and_its_template_entity(id)
    }

  let mut already_read_data: bool = false;
  let mut m_name: String = null;
  let mut mTemplateEntityId: i64 = 0;
  let mut mCreateDefaultAttributes: Option<bool> = None;
*/
}
