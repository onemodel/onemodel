/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003, 2004, 2010, 2011, 2013-2017 inclusive, and 2023-2025 inclusive, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
use crate::model::attribute_with_valid_and_observed_dates::AttributeWithValidAndObservedDates;
use crate::model::database::{DataType, Database};
use crate::util::Util;
use anyhow::{anyhow, Error, Result};
// use sqlx::{PgPool, Postgres, Row, Transaction};
use crate::model::attribute::Attribute;
use crate::model::entity::Entity;
// use crate::model::id_wrapper::IdWrapper;
use crate::model::relation_type::RelationType;
use sqlx::{Postgres, Transaction};
use std::cell::RefCell;
use std::rc::Rc;

// ***NOTE***: Similar/identical code found in *_attribute.rs, relation_to_entity.rs and relation_to_group.rs,
// due to Rust limitations on OO.  Maintain them all similarly.

/// Represents one quantity object in the system (usually [always, as of 9/2002] used as an attribute on a Entity).
#[derive(Debug)]
pub struct QuantityAttribute {
    // For descriptions of the meanings of these variables, see the comments
    // on create_quantity_attribute(...) or create_tables() in PostgreSQLDatabase or Database structs,
    // and/or examples in the database testing code.
    id: i64,
    db: Rc<dyn Database>,
    // **idea: make these members immutable?, by replacing them with something like:
    //           let (unit_id: i64, number: f64) = read_data_from_db();
    // BUT: have to figure out how to work with the
    // assignment from the other constructor, and passing vals to the Trait(scala: was superclass?) to be...immutable.
    // like how additional class vals are set when the other constructor (what's the term again?), is called. How to do the other constructor w/o a db hit.
    unit_id: i64,               /*= 0_i64*/
    number: f64,                /*= .0_f64*/
    already_read_data: bool,    /*= false*/
    parent_id: i64,             /*= 0_i64*/
    attr_type_id: i64,          /*= 0_i64*/
    valid_on_date: Option<i64>, /*= None*/
    observation_date: i64,      /*= 0_i64*/
    sorting_index: i64,         /*= 0_i64*/
}

impl QuantityAttribute {
    /// This one is perhaps only called by the database class implementation--so it can return arrays of objects & save more DB hits
    /// that would have to occur if it only returned arrays of keys. This DOES NOT create a persistent object--but rather should reflect
    /// one that already exists.  It does not confirm that the id exists in the db.
    pub fn new(
        db: Rc<dyn Database>,
        id: i64,
        parent_id: i64,
        attr_type_id: i64,
        unit_id: i64,
        number: f64,
        valid_on_date: Option<i64>,
        observation_date: i64,
        sorting_index: i64,
    ) -> QuantityAttribute {
        QuantityAttribute {
            id,
            db,
            unit_id,
            number,
            already_read_data: true,
            parent_id,
            attr_type_id,
            valid_on_date,
            observation_date,
            sorting_index,
        }
    }

    /// This constructor instantiates an existing object from the DB. You can use Entity.add*Attribute() to
    /// create a new object.
    pub fn new2(
        db: Rc<dyn Database>,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id: i64,
    ) -> Result<QuantityAttribute, anyhow::Error> {
        // (See comment in similar spot in BooleanAttribute for why not checking for exists, if db.is_remote.)
        if !db.is_remote() && !db.quantity_attribute_key_exists(transaction, id)? {
            Err(anyhow!("Key {}{}", id, Util::DOES_NOT_EXIST))
        } else {
            Ok(QuantityAttribute {
                id,
                db,
                unit_id: 0,
                number: 0.0,
                already_read_data: false,
                parent_id: 0,
                attr_type_id: 0,
                valid_on_date: None,
                observation_date: 0,
                sorting_index: 0,
            })
        }
    }

    pub fn get_number(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<f64, anyhow::Error> {
        if !self.already_read_data {
            self.read_data_from_db(transaction)?;
        }
        Ok(self.number)
    }

    pub fn get_unit_id(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<i64, anyhow::Error> {
        if !self.already_read_data {
            self.read_data_from_db(transaction)?;
        }
        Ok(self.unit_id)
    }

    fn update(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        attr_type_id_in: i64,
        unit_id_in: i64,
        number_in: f64,
        valid_on_date_in: Option<i64>,
        observation_date_in: i64,
    ) -> Result<(), anyhow::Error> {
        // write it to the database table--w/ a record for all these attributes plus a key indicating which Entity
        // it all goes with
        self.db.clone().update_quantity_attribute(
            transaction.clone(),
            self.id,
            self.get_parent_id(transaction)?,
            attr_type_id_in,
            unit_id_in,
            number_in,
            valid_on_date_in,
            observation_date_in,
        )?;
        self.unit_id = unit_id_in;
        self.number = number_in;
        // (next line is already set by just-above call to get_parent_id().)
        // self.already_read_data = true;
        self.attr_type_id = attr_type_id_in;
        self.valid_on_date = valid_on_date_in;
        self.observation_date = observation_date_in;
        Ok(())
    }
}

impl Attribute for QuantityAttribute {
    /// Return something like "volume: 15.1 liters". For full length, pass in 0 for
    /// in_length_limit. The parameter in_parent_entity refers to the Entity whose
    /// attribute this is. 3rd parameter really only applies in one of the subclasses of Attribute,
    /// otherwise can be None.
    fn get_display_string(
        &mut self,
        length_limit_in: usize,
        _unused: Option<Entity>,        /*= None*/
        _unused2: Option<RelationType>, /*=None*/
        simplify: bool,                 /* = false*/
    ) -> Result<String, anyhow::Error> {
        let attr_type_id = self.get_attr_type_id(None)?;
        let type_name: String = match self.db.get_entity_name(None, attr_type_id)? {
            None => "(None)".to_string(),
            Some(x) => x,
        };
        let entity_name: String = match self.db.clone().get_entity_name(None, self.get_unit_id(None)?)? {
            None => "(None)".to_string(),
            Some(s) => s,
        };
        let mut result: String =
            format!("{}: {} {}", type_name, self.get_number(None)?, entity_name);
        if !simplify {
            result = format!(
                "{}; {}",
                result,
                Util::get_dates_description(self.get_valid_on_date(None)?, self.get_observation_date(None)?)
            );
        }
        Ok(Util::limit_attribute_description_length(
            result.as_str(),
            length_limit_in,
        ))
    }

    fn read_data_from_db(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<(), anyhow::Error> {
        let data: Vec<Option<DataType>> =
            self.db.get_quantity_attribute_data(transaction, self.id)?;
        if data.len() == 0 {
            return Err(anyhow!(
                "No results returned from data request for: {}",
                self.id
            ));
        }

        self.already_read_data = true;
        self.unit_id = match data[1] {
            Some(DataType::Bigint(x)) => x,
            _ => return Err(anyhow!("How did we get here for {:?}?", data[1])),
        };
        self.number = match data[6] {
            Some(DataType::Float(x)) => x,
            _ => return Err(anyhow!("How did we get here for {:?}?", data[6])),
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
        self.sorting_index = match data[3] {
            Some(DataType::Bigint(x)) => x,
            _ => return Err(anyhow!("How did we get here for {:?}?", data[3])),
        };
        //END COPIED BLOCK descended from Attribute.assign_common_vars (might be in comment in boolean_attribute.rs)

        //BEGIN COPIED BLOCK descended from AttributeWithValidAndObservedDates.assign_common_vars (unclear how to do better):
        self.valid_on_date = match data[4] {
            Some(DataType::Bigint(x)) => Some(x),
            None => None,
            _ => return Err(anyhow!("How did we get here for {:?}?", data[4])),
        };
        self.observation_date = match data[5] {
            Some(DataType::Bigint(x)) => x,
            _ => return Err(anyhow!("How did we get here for {:?}?", data[5])),
        };
        //END COPIED BLOCK descended from AttributeWithValidAndObservedDates.assign_common_vars.

        Ok(())
    }

    //%%why is an id passed as a parm, vs. using the one in the struct??  ck scala original, callers.
    fn delete<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
    ) -> Result<u64, anyhow::Error> 
    {
        self.db.delete_quantity_attribute(transaction, self.id)
    }

    // This datum is provided upon construction (new2(), at minimum), so can be returned
    // regardless of already_read_data / read_data_from_db().
    fn get_id(&self) -> i64 {
        self.id
    }

    fn get_form_id(&self) -> Result<i32, Error> {
        self.db.get_attribute_form_id(Util::QUANTITY_TYPE)
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

impl AttributeWithValidAndObservedDates for QuantityAttribute {
    fn get_valid_on_date(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<Option<i64>, anyhow::Error> {
        if !self.already_read_data {
            self.read_data_from_db(transaction)?;
        }
        Ok(self.valid_on_date)
    }
    fn get_observation_date(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<i64, anyhow::Error> {
        if !self.already_read_data {
            self.read_data_from_db(transaction)?;
        }
        Ok(self.observation_date)
    }
}

#[cfg(test)]
mod test {
    /*%%put this back after similar place in boolean_attribute.rs is resolved and this can be similarly:
       "get_display_string" should "return correct string and length" in {
           let mock_db = mock[PostgreSQLDatabase];
           let entity_id = 0;
           let attr_type_id = 1;
           let quantityAttributeId = 2;
           let unit_id = 3;
           let number = 50;
           // arbitrary:
           let date = 304;
           when(mock_db.relation_type_key_exists(quantityAttributeId)).thenReturn(true)
           when(mock_db.entity_key_exists(entity_id)).thenReturn(true)
           when(mock_db.get_entity_name(attr_type_id)).thenReturn(Some("length"))
           when(mock_db.get_entity_name(unit_id)).thenReturn(Some("meters"))

           let quantityAttribute = new QuantityAttribute(mock_db, quantityAttributeId, entity_id, attr_type_id, unit_id, number, None, date, 0);
           let small_limit = 8;
           let display1: String = quantityAttribute.get_display_string(small_limit, None, None);
           //noinspection SpellCheckingInspection
           assert(display1 == "lengt...")
           let unlimited=0;
           let display2: String = quantityAttribute.get_display_string(unlimited, None, None);
           // probably should change this to GMT for benefit of other testers. Could share the DATEFORMAT* from Attribute class?
           let observed_dateOutput = "Wed 1969-12-31 17:00:00:"+date+" MST";
           let expected2:String = "length: "+number+".0 meters" + "; valid unsp'd, obsv'd " + observed_dateOutput;
           assert(display2 == expected2)

           // and something in between: broke original implementation, so writing tests helped w/ this & other bugs caught.
           let display3: String = quantityAttribute.get_display_string(49, None, None);
           let expected3: String = "length: " + number + ".0 meters" + "; valid unsp'd, obsv'd " + observed_dateOutput;
           assert(display3 == expected3.substring(0, 46) + "...")
       }
    */
}
