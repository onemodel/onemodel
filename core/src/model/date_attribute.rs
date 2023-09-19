/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2014-2017 inclusive, and 2023, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
use crate::model::attribute_with_valid_and_observed_dates::AttributeWithValidAndObservedDates;
use crate::model::database::DataType;
use crate::model::database::Database;
use crate::util::Util;
use anyhow::{anyhow, Error, Result};
// use sqlx::{PgPool, Postgres, Row, Transaction};
use crate::model::attribute::Attribute;
use crate::model::entity::Entity;
use crate::model::id_wrapper::IdWrapper;
use crate::model::relation_type::RelationType;
use sqlx::{Postgres, Transaction};

/// See TextAttribute etc code, for some comments.
/// Also, though this doesn't formally extend Attribute, it still belongs to the same group conceptually (just doesn't have the same date variables so code
/// not shared (idea: model that better, and in FileAttribute).
pub struct DateAttribute {
    /*
    // For descriptions of the meanings of these variables, see the comments
    // with create_date_attribute(...) or create_tables() in PostgreSQLDatabase or Database classes
    id: i64,
    db: Box<&'a dyn Database>,
    date_value: i64 /*= 0L*/,
    already_read_data: bool,
    /*%%= false*/
    parent_id: i64,
    /*%%= 0L*/
    attr_type_id: i64,
    /*%%= 0L*/
    sorting_index: i64,
    /*%%= 0L*/
}

impl DateAttribute<'_> {
  // (See comment in similar spot in BooleanAttribute for why not checking for exists, if db.is_remote.)
  if !db.is_remote && !db.date_attribute_key_exists(id)) {
    throw new Exception("Key " + id + Util::DOES_NOT_EXIST)
  }


  // idea: make the parameter order uniform throughout the system
  /** This one is perhaps only called by the database class implementation--so it can return arrays of objects & save more DB hits
    that would have to occur if it only returned arrays of keys. This DOES NOT create a persistent object--but rather should reflect
    one that already exists.
    */
    fn this(db: Database, id: i64, in_parent_id: i64, attr_type_id_in: i64, inDate: i64, sorting_index_in: i64) {
    this(db, id)
    mDate = inDate
    super.assign_common_vars(in_parent_id, attr_type_id_in, sorting_index_in)
  }

    fn get_display_string(length_limit_in: Int, unused: Option<Entity> = None, unused2: Option[RelationType]=None, simplify: bool = false) -> String {
    let type_name: String = db.get_entity_name(get_attr_type_id()).get;
    let mut result: String = type_name + ": ";
    result += Attribute.useful_date_format(mDate)
    Attribute.limit_attribute_description_length(result, length_limit_in)
  }

    fn getDate -> i64 {
    if !already_read_data) read_data_from_db()
    mDate
  }

  protected fn read_data_from_db() {
    let daTypeData = db.get_date_attribute_data(id);
    if daTypeData.length == 0) {
      throw new OmException("No results returned from data request for: " + id)
    }
    mDate = daTypeData(1).get.asInstanceOf[i64]
    assign_common_vars(daTypeData(0).get.asInstanceOf[i64], daTypeData(2).get.asInstanceOf[i64], daTypeData(3).get.asInstanceOf[i64])
  }

    fn update(inAttrTypeId: i64, inDate: i64) {
    // write it to the database table--w/ a record for all these attributes plus a key indicating which Entity
    // it all goes with
    db.update_date_attribute(id, get_parent_id(), inDate, inAttrTypeId)
    mDate = inDate
    attr_type_id = inAttrTypeId
  }

  /** Removes this object from the system. */
    fn delete() {
    db.delete_date_attribute(id)
    }
*/
}