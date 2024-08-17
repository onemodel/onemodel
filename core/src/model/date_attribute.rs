/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2014-2017 inclusive, and 2023, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
use anyhow::{anyhow, Error, Result};
use sqlx::{Postgres, Transaction};

// use sqlx::{PgPool, Postgres, Row, Transaction};
use crate::model::attribute::Attribute;
// use crate::model::attribute_with_valid_and_observed_dates::AttributeWithValidAndObservedDates;
use crate::model::database::DataType;
use crate::model::database::Database;
use crate::model::entity::Entity;
// use crate::model::id_wrapper::IdWrapper;
use crate::model::relation_type::RelationType;
use crate::util::Util;
use std::cell::{RefCell};
use std::rc::Rc;

// ***NOTE***: Similar/identical code found in *_attribute.rs, relation_to_entity.rs and relation_to_group.rs,
// due to Rust limitations on OO.  Maintain them all similarly.

/// See TextAttribute etc code, for some comments.
/// Also, though this doesn't formally extend Attribute, it still belongs to the same group conceptually (just doesn't have the same date variables so code
/// not shared (idea: model that better, and in FileAttribute).
pub struct DateAttribute<'a> {
    // For descriptions of the meanings of these variables, see the comments
    // with create_date_attribute(...) or create_tables() in PostgreSQLDatabase or Database classes
    id: i64,
    db: &'a dyn Database,
    date_value: i64,         /*= 0_i64*/
    already_read_data: bool, /*%%= false*/
    parent_id: i64,          /*%%= 0_i64*/
    attr_type_id: i64,       /*%%= 0_i64*/
    sorting_index: i64,      /*%%= 0_i64*/
}

impl DateAttribute<'_> {
    /// This one is perhaps only called by the database class implementation (and a test)--so it
    /// can return arrays of objects & save more DB hits
    /// that would have to occur if it only returned arrays of keys. This DOES NOT create a persistent object--but rather should reflect
    /// one that already exists.  It does not confirm that the id exists in the db.
    fn new<'a>(
        db: &'a dyn Database,
        id: i64,
        parent_id: i64,
        attr_type_id: i64,
        date_value: i64,
        sorting_index: i64,
    ) -> DateAttribute<'a> {
        DateAttribute {
            id,
            db,
            date_value,
            already_read_data: true,
            parent_id,
            attr_type_id,
            sorting_index,
        }
    }

    /// This constructor instantiates an existing object from the DB. You can use Entity.add*Attribute() to
    /// create a new object.
    pub fn new2<'a>(
        db: &'a dyn Database,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id: i64,
    ) -> Result<DateAttribute<'a>, anyhow::Error> {
        // (See comment in similar spot in BooleanAttribute for why not checking for exists, if db.is_remote.)
        if !db.is_remote() && !db.date_attribute_key_exists(transaction, id)? {
            Err(anyhow!("Key {}{}", id, Util::DOES_NOT_EXIST))
        } else {
            Ok(DateAttribute {
                id,
                db,
                date_value: 0_i64,
                already_read_data: false,
                parent_id: 0,
                attr_type_id: 0,
                sorting_index: 0,
            })
        }
    }

    pub fn get_date(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<i64, anyhow::Error> {
        if !self.already_read_data {
            self.read_data_from_db(transaction)?;
        }
        Ok(self.date_value)
    }

    fn update(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        attr_type_id_in: i64,
        date_in: i64,
    ) -> Result<(), anyhow::Error> {
        // write it to the database table--w/ a record for all these attributes plus a key indicating which Entity
        // it all goes with
        self.db.update_date_attribute(
            transaction.clone(),
            self.id,
            self.get_parent_id(transaction.clone())?,
            date_in,
            attr_type_id_in,
        )?;
        self.date_value = date_in;
        // (next line is already set by just-above call to get_parent_id().)
        // self.already_read_data = true;
        self.attr_type_id = attr_type_id_in;
        Ok(())
    }
}

impl Attribute for DateAttribute<'_> {
    /// Return some string. See comments on QuantityAttribute.get_display_string regarding the parameters.
    fn get_display_string(
        &mut self,
        length_limit_in: usize,
        _parent_entity: Option<Entity>,  /*= None*/
        _in_rt_id: Option<RelationType>, /*=None*/
        _simplify: bool,                 /* = false*/
    ) -> Result<String, anyhow::Error> {
        let attr_type_id = self.get_attr_type_id(None)?;
        let type_name: String = match self.db.get_entity_name(None, attr_type_id)? {
            None => "(None)".to_string(),
            Some(x) => x,
        };
        let result: String = format!(
            "{}: {}",
            type_name,
            Util::useful_date_format(self.get_date(None)?)
        );
        Ok(Util::limit_attribute_description_length(
            result.as_str(),
            length_limit_in,
        ))
    }

    fn read_data_from_db(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<(), anyhow::Error> {
        let data: Vec<Option<DataType>> = self.db.get_date_attribute_data(transaction, self.id)?;
        if data.len() == 0 {
            return Err(anyhow!(
                "No results returned from data request for: {}",
                self.id
            ));
        }
        //see comment at similar place in boolean_attribute.rs
        self.already_read_data = true;
        self.date_value = match data[1] {
            Some(DataType::Bigint(d)) => d,
            _ => return Err(anyhow!("How did we get here for {:?}?", data[1])),
        };

        //BEGIN COPIED BLOCK descended from Attribute.assign_common_vars (unclear how to do better for now):
        self.parent_id = match data[0] {
            Some(DataType::Bigint(x)) => x,
            _ => return Err(anyhow!("How did we get here for {:?}?", data[0])),
        };
        self.attr_type_id = match data[2] {
            Some(DataType::Bigint(x)) => x,
            _ => return Err(anyhow!("How did we get here for {:?}?", data[2])),
        };
        // DataType::Bigint(self.sorting_index) = data[5];
        self.sorting_index = match data[3] {
            Some(DataType::Bigint(x)) => x,
            _ => return Err(anyhow!("How did we get here for {:?}?", data[3])),
        };
        //END COPIED BLOCK descended from Attribute.assign_common_vars (might be in comment in boolean_attribute.rs)

        // assign_common_vars(daTypeData(0).get.asInstanceOf[i64], daTypeData(2).get.asInstanceOf[i64], daTypeData(3).get.asInstanceOf[i64])

        Ok(())
    }

    fn delete<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        //id_in: i64,
    ) -> Result<u64, anyhow::Error> {
        self.db.delete_date_attribute(transaction, self.id)
    }

    //looks unused
    // fn get_id_wrapper(&self) -> IdWrapper {
    //     IdWrapper::new(self.id)
    // }

    // This datum is provided upon construction (new2(), at minimum), so can be returned
    // regardless of already_read_data / read_data_from_db().
    fn get_id(&self) -> i64 {
        self.id
    }

    fn get_form_id(&self) -> Result<i32, Error> {
        // self.db.get_attribute_form_id(was in scala:  this.getClass.getSimpleName)
        //%% Since not using the reflection(?) from the line above, why not just return a constant
        //here?  What other places call the below method and its reverse? Do they matter now?
        self.db.get_attribute_form_id(Util::DATE_TYPE)
    }

    fn get_attr_type_id(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<i64, anyhow::Error> {
        if !self.already_read_data {
            self.read_data_from_db(transaction)?;
        }
        Ok(self.attr_type_id)
    }

    fn get_sorting_index(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<i64, anyhow::Error> {
        if !self.already_read_data {
            self.read_data_from_db(transaction)?;
        }
        Ok(self.sorting_index)
    }

    fn get_parent_id(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<i64, anyhow::Error> {
        if !self.already_read_data {
            self.read_data_from_db(transaction)?;
        }
        Ok(self.parent_id)
    }
}

#[cfg(test)]
mod test {
    /*%%put this back after similar place in boolean_attribute.rs is resolved and this can be similarly:
    "get_display_string" should "return correct string and length" in {
        let mock_db = mock[PostgreSQLDatabase];
        let entity_id = 0;
        let other_entity_id = 1;
        let date_attribute_id = 0;
        //arbitrary, in milliseconds:
        let date = 304;
        let attr_type_name = "aDateTypeName";
        when(mock_db.get_entity_name(other_entity_id)).thenReturn(Some(attr_type_name))
        when(mock_db.date_attribute_key_exists(date_attribute_id)).thenReturn(true)

        // (using arbitrary numbers for the unnamed parameters):
        let dateAttribute = new DateAttribute(mock_db, date_attribute_id, entity_id, other_entity_id, date, 0);
        let small_limit = 35;
        let display1: String = dateAttribute.get_display_string(small_limit);
        let whole_thing: String = attr_type_name + ": Wed 1969-12-31 17:00:00:"+date+" MST";
        let expected:String = whole_thing.substring(0, small_limit - 3) + "..." // put the real string here instead of dup logic?;
        assert(display1 == expected)

        let unlimited=0;
        let display2: String = dateAttribute.get_display_string(unlimited);
        assert(display2 == whole_thing)
    }
    */
}
