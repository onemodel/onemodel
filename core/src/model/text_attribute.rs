/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003, 2004, 2010, 2011, 2013-2017 inclusive, and 2023, Luke A. Call.
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

// Similar/identical code found in *_attribute.rs due to Rust limitations on OO.  Maintain them all similarly.
/// Represents one String object in the system (usually [always, as of 9/2002] used as an attribute on a Entity).
pub struct TextAttribute<'a> {
    // For descriptions of the meanings of these variables, see the comments
    // on create_text_attribute(...) or create_tables() in PostgreSQLDatabase or Database structs,
    // and/or examples in the database testing code.
    id: i64,
    db: Box<&'a dyn Database>,
    text: String,        /*%=null in scala.*/
    already_read_data: bool,    /*%%= false*/
    parent_id: i64,             /*%%= 0_i64*/
    attr_type_id: i64,          /*%%= 0_i64*/
    valid_on_date: Option<i64>, /*%%= None*/
    observation_date: i64,      /*%%= 0_i64*/
    sorting_index: i64,         /*%%= 0_i64*/
}

impl TextAttribute<'_> {
    /// This one is perhaps only called by the database class implementation (and a test)--so it
    /// can return arrays of objects & save more DB hits
    /// that would have to occur if it only returned arrays of keys. This DOES NOT create a persistent object--but rather should reflect
    /// one that already exists.  It does not confirm that the id exists in the db.
    fn new<'a>(
        db: Box<&'a dyn Database>,
        id: i64,
        parent_id: i64,
        attr_type_id: i64,
        text_in: &str,
        valid_on_date: Option<i64>,
        observation_date: i64,
        sorting_index: i64,
    ) -> TextAttribute<'a> {
        // idea: make the parameter order uniform throughout the system?
        TextAttribute {
            id,
            db,
            text: text_in.to_string(),
            already_read_data: true,
            parent_id,
            attr_type_id,
            valid_on_date,
            observation_date,
            sorting_index,
        }
    }

    /// This constructor instantiates an existing object from the DB. You can use Entity.addTextAttribute() to
    /// create a new object.
    /// This constructor instantiates an existing object from the DB. You can use Entity.add*Attribute() to
    /// create a new object.
    pub fn new2<'a>(
        db: Box<&'a dyn Database>,
        transaction: &Option<&mut Transaction<Postgres>>,
        id: i64,
    ) -> Result<TextAttribute<'a>, anyhow::Error> {
        // (See comment in similar spot in BooleanAttribute for why not checking for exists, if db.is_remote.)
        if !db.is_remote() && !db.text_attribute_key_exists(transaction, id)? {
            Err(anyhow!("Key {}{}", id, Util::DOES_NOT_EXIST))
        } else {
            Ok(TextAttribute {
                id,
                db,
                text: "".to_string(),
                already_read_data: false,
                parent_id: 0,
                attr_type_id: 0,
                valid_on_date: None,
                observation_date: 0,
                sorting_index: 0,
            })
        }
    }

    pub fn get_text(
        &mut self,
        transaction: &Option<&mut Transaction<Postgres>>,
    ) -> Result<&str, anyhow::Error> {
        if !self.already_read_data {
            self.read_data_from_db(transaction)?;
        }
        Ok(self.text.as_str())
    }

    fn update(
        &mut self,
        transaction: &Option<&mut Transaction<Postgres>>,
        attr_type_id_in: i64,
        text_in: &str,
        valid_on_date_in: Option<i64>,
        observation_date_in: i64,
    ) -> Result<(), anyhow::Error> {
        // write it to the database table--w/ a record for all these attributes plus a key indicating which Entity
        // it all goes with
        self.db.update_text_attribute(
            transaction,
            self.id,
            self.get_parent_id(transaction)?,
            attr_type_id_in,
            text_in,
            valid_on_date_in,
            observation_date_in,
        )?;
        self.text = text_in.to_string();
        // (next line is already set by just-above call to get_parent_id().)
        // self.already_read_data = true;
        self.attr_type_id = attr_type_id_in;
        self.valid_on_date = valid_on_date_in;
        self.observation_date = observation_date_in;
        Ok(())
    }

}

impl Attribute for TextAttribute<'_> {
    /// Return some string. See comments on QuantityAttribute.get_display_string regarding the parameters.
    fn get_display_string(
        &mut self,
        length_limit_in: usize,
        _unused: Option<Entity>,        /*= None*/
        _unused2: Option<RelationType>, /*=None*/
        simplify: bool,                 /* = false*/
    ) -> Result<String, anyhow::Error> {
        let attr_type_id = self.get_attr_type_id(&None)?;
        let type_name: String = match self.db.get_entity_name(&None, attr_type_id)? {
            None => "(None)".to_string(),
            Some(x) => x,
        };
        let mut result: String = if simplify && (type_name == "paragraph" || type_name == "quote") {
            self.get_text(&None)?.to_string()
        } else {
            let result = format!("{}: \"{}\"", type_name, self.get_text(&None)?);
            format!(
                "{}; {}",
                result,
                Util::get_dates_description(self.valid_on_date, self.observation_date)
            )
        };
        if !simplify {
            result = format!("{}; {}", result, Util::get_dates_description(self.valid_on_date, self.observation_date));
        }
        Ok(Util::limit_attribute_description_length(
            result.as_str(),
            length_limit_in,
        ))
    }
    // fn get_display_string(length_limit_in: Int, unused: Option<Entity> = None, unused2: Option[RelationType]=None, simplify: bool = false) -> String {
    //     let mut result: String = {
    //       if simplify && (type_name == "paragraph" || type_name == "quote")) get_text
    //       else type_name + ": \"" + get_text + "\""
    //     }
    //     if ! simplify) result += "; " + get_dates_description
    //     Attribute.limit_attribute_description_length(result, length_limit_in)
    // }

    fn read_data_from_db(
        &mut self,
        transaction: &Option<&mut Transaction<Postgres>>,
    ) -> Result<(), anyhow::Error> {
        let data: Vec<Option<DataType>> =
            self.db.get_text_attribute_data(transaction, self.id)?;
        if data.len() == 0 {
            return Err(anyhow!(
                "No results returned from data request for: {}",
                self.id
            ));
        }

        self.already_read_data = true;
        self.text = match data[1].clone() {
            Some(DataType::String(x)) => x,
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
        self.sorting_index = match data[3] {
            Some(DataType::Bigint(x)) => x,
            _ => return Err(anyhow!("How did we get here for {:?}?", data[3])),
        };
        //END COPIED BLOCK descended from Attribute.assign_common_vars (might be in comment in boolean_attribute.rs)

        //BEGIN COPIED BLOCK descended from AttributeWithValidAndObservedDates.assign_common_vars (unclear how to do better):
        //%%$%%% fix this next part after figuring out about what happens when querying a null back, in pg.db_query etc!
        // valid_on_date: Option<i64> /*%%= None*/,
        /*DataType::Bigint(%%)*/
        self.valid_on_date = None; //data[4];
        // self.valid_on_date = match data[4] {
        //     DataType::Bigint(x) => x,
        //     _ => return Err(anyhow!("How did we get here for {:?}?", data[4])),
        // };

        // DataType::Bigint(self.observation_date) = data[4];
        self.observation_date = match data[5] {
            Some(DataType::Bigint(x)) => x,
            _ => return Err(anyhow!("How did we get here for {:?}?", data[5])),
        };
        //END COPIED BLOCK descended from AttributeWithValidAndObservedDates.assign_common_vars.

        Ok(())
    }

  /// Removes this object from the system.
  fn delete<'a>(
      &'a self,
      transaction: &Option<&mut Transaction<'a, Postgres>>,
      id_in: i64,
  ) -> Result<u64, anyhow::Error> {
      self.db.delete_text_attribute(transaction, id_in)
  }

    // This datum is provided upon construction (new2(), at minimum), so can be returned
    // regardless of already_read_data / read_data_from_db().
    fn get_id(&self) -> i64 {
        self.id
    }

    fn get_form_id(&self) -> Result<i32, Error> {
        // self.db.get_attribute_form_id(was in scala:  this.getClass.getSimpleName)
        //%% Since not using the reflection(?) from the line above, why not just return a constant
        //here?  What other places call the below method and its reverse? Do they matter now?
        self.db.get_attribute_form_id(Util::BOOLEAN_TYPE)
    }

    fn get_attr_type_id(
        &mut self,
        transaction: &Option<&mut Transaction<Postgres>>,
    ) -> Result<i64, anyhow::Error> {
        if !self.already_read_data {
            self.read_data_from_db(transaction)?;
        }
        Ok(self.attr_type_id)
    }

    fn get_sorting_index(
        &mut self,
        transaction: &Option<&mut Transaction<Postgres>>,
    ) -> Result<i64, anyhow::Error> {
        if !self.already_read_data {
            self.read_data_from_db(transaction);
        }
        Ok(self.sorting_index)
    }

    fn get_parent_id(
        &mut self,
        transaction: &Option<&mut Transaction<Postgres>>,
    ) -> Result<i64, anyhow::Error> {
        if !self.already_read_data {
            self.read_data_from_db(transaction)?;
        }
        Ok(self.parent_id)
    }
}

impl AttributeWithValidAndObservedDates for TextAttribute<'_> {
    fn get_valid_on_date(
        &mut self,
        transaction: &Option<&mut Transaction<Postgres>>,
    ) -> Result<Option<i64>, anyhow::Error> {
        if !self.already_read_data {
            self.read_data_from_db(transaction)?;
        }
        Ok(self.valid_on_date)
    }
    fn get_observation_date(
        &mut self,
        transaction: &Option<&mut Transaction<Postgres>>,
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
        let other_entity_id = 1;
        let text_attribute_id = 0;
        //arbitrary, in milliseconds:
        let date = 304;
        let attr_type_name = "description";
        let longDescription = "this is a long description of a thing which may or may not really have a description but here's some text";
        when(mock_db.get_entity_name(other_entity_id)).thenReturn(Some(attr_type_name))
        when(mock_db.text_attribute_key_exists(text_attribute_id)).thenReturn(true)

        // (using arbitrary numbers for the unnamed parameters):
        let textAttribute = new TextAttribute(mock_db, text_attribute_id, entity_id, other_entity_id, longDescription, None, date, 0);
        let small_limit = 35;
        let display1: String = textAttribute.get_display_string(small_limit, None, None);
        let whole_thing: String = attr_type_name + ": \"" + longDescription + "\"; valid unsp'd, obsv'd Wed 1969-12-31 17:00:00:"+date+" MST";
        let expected:String = whole_thing.substring(0, small_limit - 3) + "..." // put the real string here instead of dup logic?;
        assert(display1 == expected)

        let unlimited=0;
        let display2: String = textAttribute.get_display_string(unlimited, None, None);
        assert(display2 == whole_thing)
    }
    */
}
