/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2014-2017 inclusive, and 2023, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
use anyhow::{anyhow, Result};
use crate::model::database::DataType;
use crate::model::database::Database;
use crate::util::Util;
use sqlx::{PgPool, Postgres, Row, Transaction};

pub struct BooleanAttribute<'a> {
    m_id: i64,
    m_db: Box<&'a dyn Database>,
    // For descriptions of the meanings of these variables, see the comments
    // on create_tables(...), and examples in the database testing code &/or in PostgreSQLDatabase or Database classes.
    m_boolean: bool,              /*%%false*/
    m_already_read_data: bool,    /*%%= false*/
    m_parent_id: i64,             /*%%= 0L*/
    m_attr_type_id: i64,          /*%%= 0L*/
    m_valid_on_date: Option<i64>, /*%%= None*/
    m_observation_date: i64,      /*%%= 0L*/
    m_sorting_index: i64,         /*%%= 0L*/
}

impl BooleanAttribute<'_> {
    pub fn new2<'a>(db: Box<&'a dyn Database>, transaction: &Option<&mut Transaction<Postgres>>, id: i64) -> Result<BooleanAttribute<'a>, anyhow::Error> {
        // Not doing these checks if the object is at a remote site because doing it over REST would probably be too slow. Will
        // wait for an error later to see if there is a problem (ie, assuming usually not).
        // idea: And today having doubts about that.
        if !db.is_remote() && !db.boolean_attribute_key_exists(transaction, id)? {
            Err(anyhow!(format!("Key {}{}", id, Util::DOES_NOT_EXIST)))
        } else {
            Ok(BooleanAttribute {
                m_id: id,
                m_db: db,
                m_boolean: false,
                m_already_read_data: false,
                m_parent_id: 0,
                m_attr_type_id: 0,
                m_valid_on_date: None,
                m_observation_date: 0,
                m_sorting_index: 0,
            })
        }
    }

    fn get_boolean(&mut self, transaction: &Option<&mut Transaction<Postgres>>) -> Result<bool, anyhow::Error> {
        if !self.m_already_read_data {
            self.read_data_from_db(transaction)?;
        }
        Ok(self.m_boolean)
    }

    fn read_data_from_db(&mut self, transaction: &Option<&mut Transaction<Postgres>>) -> Result<(), anyhow::Error> {
        let ba_type_data: Vec<DataType> = self.m_db.get_boolean_attribute_data(transaction, self.m_id)?;
        if ba_type_data.len() == 0 {
            return Err(anyhow!(format!(
                "No results returned from data request for: {}",
                self.m_id
            )));
        }
        // DataType::Boolean(self.m_boolean) = ba_type_data[1];
        self.m_boolean = match ba_type_data[1] {
            DataType::Boolean(b) => b,
            _ => return Err(anyhow!(format!("How did we get here for {:?}?", ba_type_data[1]))),
        };

        //%%$%%%%what do about making this into shared code? duplicate it or can work from the Trait/s?
        //idea: surely there is some better way than what I am doing here? See other places similarly.  Maybe implement DataType.clone() ?

        // super.assignCommonVars(ba_type_data(0).get.asInstanceOf[i64], ba_type_data(2).get.asInstanceOf[i64], ba_type_data(3).asInstanceOf[Option<i64>],
        //                        ba_type_data(4).get.asInstanceOf[i64], ba_type_data(5).get.asInstanceOf[i64])
        self.m_already_read_data = true;
        // DataType::Bigint(self.m_parent_id) = ba_type_data[0];
        self.m_parent_id = match ba_type_data[0] {
            DataType::Bigint(x) => x,
            _ => return Err(anyhow!(format!("How did we get here for {:?}?", ba_type_data[0]))),
        };
        // DataType::Bigint(self.m_attr_type_id) = ba_type_data[2];
        self.m_attr_type_id = match ba_type_data[2] {
            DataType::Bigint(x) => x,
            _ => return Err(anyhow!(format!("How did we get here for {:?}?", ba_type_data[2]))),
        };

        //%%$%%%% fix this next part after figuring out about what happens when querying a null back, in pg.db_query etc!
        // valid_on_date: Option<i64> /*%%= None*/,
        /*DataType::Bigint(%%)*/
        self.m_valid_on_date = None; //ba_type_data[3];
                                     // self.m_valid_on_date = match ba_type_data[3] {
                                     //     DataType::Bigint(x) => x,
                                     //     _ => return Err(anyhow!(format!("How did we get here for {:?}?", ba_type_data[3]))),
                                     // };

        // DataType::Bigint(self.m_observation_date) = ba_type_data[4];
        self.m_observation_date = match ba_type_data[4] {
            DataType::Bigint(x) => x,
            _ => return Err(anyhow!(format!("How did we get here for {:?}?", ba_type_data[4]))),
        };
        // DataType::Bigint(self.m_sorting_index) = ba_type_data[5];
        self.m_sorting_index = match ba_type_data[4] {
            DataType::Bigint(x) => x,
            _ => return Err(anyhow!(format!("How did we get here for {:?}?", ba_type_data[5]))),
        };
        Ok(())
    }

    pub fn get_parent_id(&mut self, transaction: &Option<&mut Transaction<Postgres>>) -> Result<i64, anyhow::Error> {
        if !self.m_already_read_data {
            self.read_data_from_db(transaction)?;
        }
        Ok(self.m_parent_id)
    }
    pub fn get_id(&self) -> i64 {
        // This datum is provided upon construction (new2(), at minimum), so can be returned
        // regardless of m_already_read_data / read_data_from_db().
        self.m_id
    }
    pub fn get_attr_type_id(&mut self, transaction: &Option<&mut Transaction<Postgres>>) -> Result<i64, anyhow::Error> {
        if !self.m_already_read_data {
            self.read_data_from_db(transaction)?;
        }
        Ok(self.m_attr_type_id)
    }
    pub fn get_valid_on_date(&mut self, transaction: &Option<&mut Transaction<Postgres>>) -> Result<Option<i64>, anyhow::Error> {
        if !self.m_already_read_data {
            self.read_data_from_db(transaction)?;
        }
        Ok(self.m_valid_on_date)
    }
    pub fn get_observation_date(&mut self, transaction: &Option<&mut Transaction<Postgres>>) -> Result<i64, anyhow::Error> {
        if !self.m_already_read_data {
            self.read_data_from_db(transaction)?;
        }
        Ok(self.m_observation_date)
    }
    // }

    /// See TextAttribute etc for some comments.
    // impl AttributeWithValidAndObservedDates for BooleanAttribute {

    /*%%


      /** This one is perhaps only called by the database class implementation--so it can return arrays of objects & save more DB hits
        that would have to occur if it only returned arrays of keys. This DOES NOT create a persistent object--but rather should reflect
        one that already exists.
        */
        fn this(m_db: Database, m_id: i64, parent_id_in: i64, attr_type_id_in: i64, boolean_in: bool, valid_on_date: Option<i64>, observationDate: i64,
               sorting_index_in: i64) {
        this(m_db, m_id)
        m_boolean = boolean_in
        assignCommonVars(parent_id_in, attr_type_id_in, valid_on_date, observationDate, sorting_index_in)
      }

      /** return some string. See comments on QuantityAttribute.get_display_string regarding the parameters.
        */
        fn get_display_string(lengthLimitIn: Int, unused: Option<Entity> = None, unused2: Option[RelationType]=None, simplify: bool = false) -> String {
        let typeName: String = m_db.get_entity_name(get_attr_type_id()).get;
        let mut result: String = typeName + ": " + get_boolean + "";
        if ! simplify) result += "; " + get_dates_description
        Attribute.limitDescriptionLength(result, lengthLimitIn)
      }
    */
    fn update(
        &mut self,
        transaction: &Option<&mut Transaction<Postgres>>,
        attr_type_id_in: i64,
        boolean_in: bool,
        valid_on_date_in: Option<i64>,
        observation_date_in: i64,
    ) -> Result<(), anyhow::Error> {
        // write it to the database table--w/ a record for all these attributes plus a key indicating which Entity
        // it all goes with
        self.m_db.update_boolean_attribute(
            transaction,
            self.m_id,
            self.get_parent_id(transaction)?,
            attr_type_id_in,
            boolean_in,
            valid_on_date_in,
            observation_date_in,
        )?;
        self.m_boolean = boolean_in;
        // (next line is already set by just-above call to get_parent_id().)
        // self.m_already_read_data = true;
        self.m_attr_type_id = attr_type_id_in;
        self.m_valid_on_date = valid_on_date_in;
        self.m_observation_date = observation_date_in;
        Ok(())
    }
    /*
     /** Removes this object from the system. */
       fn delete() {
       m_db.deleteBooleanAttribute(m_id)
       }

     /** For descriptions of the meanings of these variables, see the comments
       on create_boolean_attribute(...) or create_tables() in PostgreSQLDatabase or Database classes.
       */
       private let mut m_boolean: bool = false;
    */
}
