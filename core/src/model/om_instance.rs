/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2016-2017 inclusive, and 2023, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
use anyhow::{anyhow, Error, Result};
use crate::model::database::Database;
use crate::model::database::DataType;
use sqlx::{Postgres, Transaction};
use crate::util::Util;

pub struct OmInstance<'a> {
    id: String,
    db: Box<&'a dyn Database>,
    already_read_data: bool /*= false*/,
    is_local: bool /*= false*/,
    address: String /*= ""*/,
    insertion_date: i64 /*= 0*/,
    entity_id: Option<i64> /*= None*/,
}

impl OmInstance {
    fn address_length() -> i32 {
        Database::om_instance_address_length()
    }

    fn is_duplicate(db_in: &dyn Database, transaction: &Option<&mut Transaction<Postgres>>,
                    address_in: &str, _self_id_to_ignore_in: Option<String> /*= None*/) -> Result<bool, anyhow::Error> {
        db_in.is_duplicate_om_instance_address(transaction, address_in, _self_id_to_ignore_in)
      }

    fn create(db_in: Box<&dyn Database>, transaction: &Option<&mut Transaction<Postgres>>,
              id_in: &str, address_in: &str, entity_id_in: Option<i64> /*= None*/) -> Result<OmInstance, anyhow::Error> {
        // Passing false for is_local_in because the only time that should be true is when it is created at db creation, for this site, and that is done
        // in the db class more directly.
        let insertion_date: i64 = db_in.create_om_instance(transaction, id_in.to_string(), false,
                                                           address_in.to_string(), entity_id_in)?;
        OmInstance::new(db_in, id_in.to_string(), /*is_local_in =*/ false,
                        address_in.to_string(), insertion_date, entity_id_in)?
      }


    /// See table definition in the database class for details.
    /// This 1st constructor instantiates an existing object from the DB. Generally use Model.createObject() to create a new object.
    /// Note: Having Entities and other DB objects be readonly makes the code clearer & avoid some bugs, similarly to reasons for immutability in scala.
    pub fn new2<'a>(db: Box<&'a dyn Database>, transaction: &Option<&mut Transaction<Postgres>>, id: String)
    -> Result<OmInstance<'a>, anyhow::Error> {
      // (See comment in similar spot in BooleanAttribute for why not checking for exists, if db.is_remote.)
      if !db.is_remote && !db.om_instance_key_exists(transaction, id.as_str())? {
          Err(anyhow!("Key {}{}", id, Util::DOES_NOT_EXIST))
      } else {
          Ok(OmInstance {
              id: id.clone(),
              db,
              already_read_data: false,
              is_local: false,
              address: "".to_string(),
              insertion_date: 0,
              entity_id: None,
          })
      }
    }


  /// This one is perhaps only called by the database class implementation--so it can return arrays of objects & save more DB hits
  /// that would have to occur if it only returned arrays of keys. This DOES NOT create a persistent object--but rather should reflect
  /// one that already exists.
    pub fn new<'a>(db: Box<&'a dyn Database>,
            id: String, is_local_in: bool, address_in: String, insertion_date_in: i64,
           entity_id_in: Option<i64> /*= None*/) -> OmInstance<'a> {
      OmInstance {
        id: id.clone(),
        db,
        is_local: is_local_in,
        address: address_in,
        insertion_date: insertion_date_in,
        entity_id: entity_id_in,
        already_read_data: true,
      }
  }

  /// When using, consider if get_archived_status_display_string should be called with it in the display (see usage examples of get_archived_status_display_string).
    fn get_id(&self) -> Result<String, anyhow::Error> {
      // all creation methods ensure id exists, so no need to call read_data_from_db().
      Ok(self.id.clone())
    }

    fn get_local(&mut self) -> Result<bool, anyhow::Error> {
        if !self.already_read_data {
            self.read_data_from_db(&None)?;
        }
        Ok(self.is_local)
    }

    fn get_creation_date(
        &mut self,
        transaction: &Option<&mut Transaction<Postgres>>,
    ) -> Result<i64, anyhow::Error> {
        if !self.already_read_data {
            self.read_data_from_db(transaction)?;
        }
        Ok(self.insertion_date)
    }

    fn get_creation_date_formatted(&mut self, transaction: &Option<&mut Transaction<Postgres>>)
        -> Result<String, anyhow::Error> {
        Ok(Util::useful_date_format(self.get_creation_date(transaction)?))
    }

    fn get_address(&mut self) -> Result<String, anyhow::Error> {
        if !self.already_read_data {
            self.read_data_from_db(&None)?;
        }
        Ok(self.address.clone())
    }

    fn get_entity_id(&mut self,
                     transaction: &Option<&mut Transaction<Postgres>>,
    ) -> Result<Option<i64>, anyhow::Error> {
        if !self.already_read_data {
            self.read_data_from_db(transaction)?;
        }
        Ok(self.entity_id)
    }


    fn read_data_from_db(
        &mut self,
        transaction: &Option<&mut Transaction<Postgres>>,
    ) -> Result<(), anyhow::Error> {
        let data: Vec<Option<DataType>> = self.db.get_om_instance_data(transaction, self.get_id()?)?;
        if data.len() == 0 {
            return Err(anyhow!(
                "No results returned from data request for: {}",
                self.id
            ));
        }
        //see comment at similar place in boolean_attribute.rs
        self.already_read_data = true;
        self.is_local = match data[0] {
            Some(DataType::Boolean(b)) => b,
            _ => return Err(anyhow!("How did we get here for {:?}?", data[0])),
        };
        self.address = match data[1].clone() {
            Some(DataType::String(x)) => x,
            _ => return Err(anyhow!("How did we get here for {:?}?", data[1])),
        };
        self.insertion_date = match data[2] {
            Some(DataType::Bigint(x)) => x,
            _ => return Err(anyhow!("How did we get here for {:?}?", data[2])),
        };
        //%%%%% fix this next part after figuring out about what happens when querying a null back, in pg.db_query etc!
        // valid_on_date: Option<i64> /*%%= None*/,
        /*DataType::Bigint(%%)*/
        self.entity_id = None; //data[3];
        // self.valid_on_date = match data[3] {
        //     DataType::Bigint(x) => x,
        //     _ => return Err(anyhow!("How did we get here for {:?}?", data[3])),
        // };
        // entity_id = omInstanceData(3).asInstanceOf[Option<i64>]

        Ok(())
    }

    fn get_display_string() -> String {
        format!("{}:{}, {}, created on {}", id, (if is_local " (local)" else ""), get_address, get_creation_date_formatted)
    }

    fn update(new_address: String) /*%%-> Unit*/ {
        db.update_om_instance(get_id, new_address, get_entity_id)
      }

    fn delete(&self, transaction: &Option<&mut Transaction<Postgres>>) {
        db.delete_om_instance(transaction, self.get_id())
    }

}