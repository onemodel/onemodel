/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2014-2017 inclusive, and 2023-2025 inclusive, Luke A. Call.
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
// use crate::model::id_wrapper::IdWrapper;
use crate::model::relation_type::RelationType;
use sqlx::{Postgres, Transaction};
use std::cell::RefCell;
use std::rc::Rc;

// ***NOTE***: Similar/identical code found in *_attribute.rs, relation_to_*entity.rs and relation_to_group.rs,
// due to Rust limitations on OO.  Maintain them all similarly.

pub struct BooleanAttribute {
    // For descriptions of the meanings of these variables, see the comments
    // on create_boolean_attribute(...) or create_tables() in PostgreSQLDatabase or Database structs,
    // and/or examples in the database testing code.
    db: Rc<dyn Database>,
    id: i64,
    parent_id: i64,             /*= 0_i64*/
    attr_type_id: i64,          /*= 0_i64*/
    boolean_value: bool,        /*false*/
    valid_on_date: Option<i64>, /*= None*/
    observation_date: i64,      /*= 0_i64*/
    sorting_index: i64,         /*= 0_i64*/
    already_read_data: bool,    /*= false*/
}

impl BooleanAttribute {
    /// This one is perhaps only called by the database class implementation (and a test)--so it
    /// can return arrays of objects & save more DB hits
    /// that would have to occur if it only returned arrays of keys. This DOES NOT create a persistent object--but rather should reflect
    /// one that already exists.  It does not confirm that the id exists in the db.
    pub fn new(
        db: Rc<dyn Database>,
        id: i64,
        parent_id: i64,
        attr_type_id: i64,
        boolean_value: bool,
        valid_on_date: Option<i64>,
        observation_date: i64,
        sorting_index: i64,
    ) -> BooleanAttribute {
        BooleanAttribute {
            db,
            id,
            parent_id,
            attr_type_id,
            boolean_value,
            valid_on_date,
            observation_date,
            sorting_index,
            already_read_data: true,
        }
        // assign_common_vars(parent_id_in, attr_type_id_in, valid_on_date, observation_date, sorting_index_in)
    }

    /// This constructor instantiates an existing object from the DB (or rather, creates it minimally, and then
    /// when data is read, it reads the whole object from the DB). You can use Entity.add*Attribute() to
    /// create a new object.
    pub fn new2(
        db: Rc<dyn Database>,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id: i64,
    ) -> Result<BooleanAttribute, anyhow::Error> {
        // Not doing these checks if the object is at a remote site because doing it over REST would probably be too slow. Will
        // wait for an error later to see if there is a problem (ie, assuming usually not).
        // idea: And today having doubts about that.
        if !db.is_remote() && !db.boolean_attribute_key_exists(transaction, id)? {
            Err(anyhow!("Key {}{}", id, Util::DOES_NOT_EXIST))
        } else {
            Ok(BooleanAttribute {
                id,
                db,
                boolean_value: false,
                already_read_data: false,
                parent_id: 0,
                attr_type_id: 0,
                valid_on_date: None,
                observation_date: 0,
                sorting_index: 0,
            })
        }
    }

    pub fn get_boolean(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<bool, anyhow::Error> {
        if !self.already_read_data {
            self.read_data_from_db(transaction)?;
        }
        Ok(self.boolean_value)
    }

    fn update(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        attr_type_id_in: i64,
        boolean_in: bool,
        valid_on_date_in: Option<i64>,
        observation_date_in: i64,
    ) -> Result<(), anyhow::Error> {
        // write it to the database table--w/ a record for all these attributes plus a key indicating which Entity
        // it all goes with
        self.db.clone().update_boolean_attribute(
            transaction.clone(),
            self.id,
            self.get_parent_id(transaction.clone())?,
            attr_type_id_in,
            boolean_in,
            valid_on_date_in,
            observation_date_in,
        )?;
        self.boolean_value = boolean_in;
        // (next line is already set by just-above call to get_parent_id().)
        // self.already_read_data = true;
        self.attr_type_id = attr_type_id_in;
        self.valid_on_date = valid_on_date_in;
        self.observation_date = observation_date_in;
        Ok(())
    }
}

impl Attribute for BooleanAttribute {
    /// Return some string. See comments on QuantityAttribute.get_display_string regarding the parameters.
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
        let mut result: String = format!("{}: {}", type_name, self.get_boolean(None)?);
        if !simplify {
            result = format!(
                "{}; {}",
                result,
                Util::get_dates_description(
                    self.get_valid_on_date(None)?,
                    self.get_observation_date(None)?
                )
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
            self.db.get_boolean_attribute_data(transaction, self.id)?;
        if data.len() == 0 {
            return Err(anyhow!(
                "No results returned from data request for: {}",
                self.id
            ));
        }

        //%%later: what do about making this into shared code? duplicate it or can work from the Trait/s? see in anki re : to get fns from a trait (search
        // rustlang deck re trait, is near end of a note), or
        // the newtype pattern?
        //idea: surely there is some better way than what I am doing here? See other places similarly.  Maybe implement DataType.clone() ?

        // super.assign_common_vars(data(0).get.asInstanceOf[i64], data(2).get.asInstanceOf[i64], data(3).asInstanceOf[Option<i64>],
        //                        data(4).get.asInstanceOf[i64], data(5).get.asInstanceOf[i64])
        // DataType::Boolean(self.boolean_value) = data[1];
        self.boolean_value = match data[1] {
            Some(DataType::Boolean(b)) => b,
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

        self.already_read_data = true;
        Ok(())
    }

    fn delete<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
    ) -> Result<u64, anyhow::Error> {
        self.db.delete_boolean_attribute(transaction, self.id)
    }

    // (Considered moving this to the Attribute trait with this signature:
    // fn get_id_wrapper(&self, id: i64 /*= self.id*/) -> IdWrapper {
    // ...but that would require external callers to be able to access the id themselves anyway.
    //looks unused:
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
        //%%later: Since not using the reflection(?) from the line above, why not just return a constant
        //here?  What other places call the below method and its reverse? Do they matter now?
        self.db.get_attribute_form_id(Util::BOOLEAN_TYPE)
    }

    // fn assign_common_vars(parent_id_in: i64, attr_type_id_in: i64, sorting_index_in: i64) {
    //   parent_id = parent_id_in
    //   attr_type_id = attr_type_id_in
    //   sorting_index = sorting_index_in
    //   already_read_data = true
    // }

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

impl AttributeWithValidAndObservedDates for BooleanAttribute {
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
    /*%%latertests: Revisit mbe when it is time to learn from mockall docs, since putting "#[automock]" at
        the top of Database gets ~500 errors, and automock in docs is not supported. See:
        https://docs.rs/mockall/latest/
        ...and search for "mock!", click/open that/use it, and then mbe cont reading at "Static return values".
        //use mockall::{automock, mock, predicate::*};
    */
    use super::*;
    use crate::model::postgres::postgresql_database::PostgreSQLDatabase;
    use tracing::*;

    /// BA should "return correct string and length"
    #[test]
    fn test_get_display_string() {
        Util::initialize_tracing();
        // let mock_db = mock[PostgreSQLDatabase];
        //let mut mock_db = MockDatabase::new();
        let db: Rc<PostgreSQLDatabase> = Rc::new(Util::initialize_test_db().unwrap());
        //let tx = db.begin_trans().unwrap();
        //let tx = Some(Rc::new(RefCell::new(tx)));
        //let entity_id = 0;
        //If using mocks I wouldn't have to actually create the data in the db.
        let attr_type_name = "description";
        let entity_id: i64 = Entity::create_entity(db.clone(), None, attr_type_name, None, None)
            .unwrap()
            .get_id();
        //let boolean_value = true;
        //let other_entity_id = 1;
        //let boolean_attribute_id = 0;
        //arbitrary, in milliseconds:
        let date = 304;
        //// when(mock_db.get_entity_name(other_entity_id)).thenReturn(Some(attr_type_name))
        //mock_db.expect_get_entity_name()
        //    .with(predicate::eq(other_entity_id))
        //    .times(1)
        //    .returning(|| "description");
        //// when(mock_db.boolean_attribute_key_exists(boolean_attribute_id)).thenReturn(true)

        // (using arbitrary numbers for the unnamed parameters):
        //let mut boolean_attribute = BooleanAttribute::new(mock_db, boolean_attribute_id,
        //let mut boolean_attribute = BooleanAttribute::new(&db, boolean_attribute_id, entity_id,
        //                                                  other_entity_id, boolean_value, None, date, 0,
        //);
        let bid = db
            .create_boolean_attribute(
                None, entity_id, //boolean_attribute_id,
                entity_id, true, None, date, None,
            )
            .unwrap();
        let mut boolean_attribute: BooleanAttribute =
            BooleanAttribute::new2(db, None, bid).unwrap();
        let small_limit = 35;
        let display1: String = boolean_attribute
            .get_display_string(small_limit, None, None, false)
            .unwrap();
        let whole_thing: String = format!(
            //%%later: make this not depend on my local time zone being MST!? or what? See 2nd assert below.
            // And the same thing in other attributes.
            // See comment about (lack of) TZ abbreviations, at Util.DATEFORMAT.
            //"{}: true; valid unsp'd, obsv'd 1969-12-31 17:00:00:{} MST",
            //(Also note that it is MST, not MDT even in April, I'm guessing because 12-31 is
            //in the Standard Time period.)
            "{}: true; valid unsp'd, obsv'd 1969-12-31 17:00:00:{} -07:00",
            attr_type_name, date
        );
        // idea: put the real string here instead of dup logic?;
        // let expected: String = whole_thing.substring(0, small_limit - 3) + "..." ;
        let expected: String =
            Util::substring_from_start(whole_thing.as_str(), small_limit - 3) + "...";
        debug!("display1 = \"{}\", expected = \"{}\"", display1, expected);
        assert!(display1 == expected);

        let unlimited = 0;
        let display2: String = boolean_attribute
            .get_display_string(unlimited, None, None, false)
            .unwrap();
        debug!(
            "display2 = \"{}\", whole_thing = \"{}\"",
            display2, whole_thing
        );
        assert!(display2 == whole_thing);
    }
}
