/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2004, 2010, 2011, 2013-2017 inclusive, and 2023-2024 inclusive, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
//use crate::color::Color;
//use crate::model::attribute_with_valid_and_observed_dates::AttributeWithValidAndObservedDates;
use crate::model::database::DataType;
use crate::model::database::Database;
use crate::util::Util;
use anyhow::{anyhow, /*Error, */ Result};
// use sqlx::{PgPool, Postgres, Row, Transaction};
//use crate::model::attribute::Attribute;
use crate::model::entity::Entity;
// use crate::model::id_wrapper::IdWrapper;
//use crate::model::relation_to_entity::RelationToEntity;
//use crate::model::relation_type::RelationType;
use sqlx::{Postgres, Transaction};
use std::cell::RefCell;
use std::rc::Rc;

//move this to some *relation* struct like RelationType?
/// See comments on/in (Util or RelationType).ask_for_name_in_reverse_direction() and .ask_for_relation_directionality().
enum RelationDirectionality {
    UNI,
    BI,
    NON,
}

pub struct RelationType<'a> {
    db: &'a dyn Database,
    entity_id: i64,
    name: String,
    /// For descriptions of the meanings of these variables, see the comments
    /// on PostgreSQLDatabase.create_tables(...), and examples in the database testing code.
    name_in_reverse_direction: String,
    directionality: String,
    already_read_data: bool,
}

impl RelationType<'_> {
    // idea: should use these more, elsewhere (replacing hard-coded values! )
    pub const BIDIRECTIONAL: &'static str = "BI";
    pub const UNIDIRECTIONAL: &'static str = "UNI";
    pub const NONDIRECTIONAL: &'static str = "NON";

    /// This one is perhaps only called by the database class implementation--so it can return arrays of objects & save more DB hits
    /// that would have to occur if it only returned arrays of keys. This DOES NOT create a persistent object--but rather should reflect
    /// one that already exists.
    pub fn new<'a>(
        db: &'a dyn Database,
        entity_id: i64,
        name: String,
        name_in_reverse_direction: String,
        directionality: String,
    ) -> RelationType<'a> {
        RelationType {
            db,
            entity_id,
            name,
            name_in_reverse_direction,
            directionality,
            already_read_data: true,
        }
    }

    /// This constructor instantiates an existing object from the DB. You can use Entity.addRelationTypeAttribute() to
    /// create a new object. Assumes caller just read it from the DB and the info is accurate (i.e., this may only
    /// ever need to be called by a Database instance?).
    pub fn new2<'a>(
        db: &'a dyn Database,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id: i64,
    ) -> Result<RelationType<'a>, anyhow::Error> {
        // (see comments at similar location in boolean_attribute.rs.)
        if !db.is_remote() && !db.relation_type_key_exists(transaction, id)? {
            Err(anyhow!("Key {}{}", id, Util::DOES_NOT_EXIST))
        } else {
            Ok(RelationType {
                db,
                entity_id: id,
                name: "".to_string(),
                name_in_reverse_direction: "".to_string(),
                directionality: "".to_string(),
                already_read_data: false,
                //extends Entity(db, id) {
                //                     %%make it work with entity: has-a?
                //                     but: I pass the entity to RelationType.showInEntityMenuThenMainMenu
                //                     but: what makes most sense since later/ideally, I hoped to make this its own table? Better to refer to an entity and get more code reuse that way, and continue having separate tables in the db with ~"homegrown inheritance" as noted there for RelationType?  Just note that here as a possible future?  What about "except quantity units, attr types, & relation types" in Util.rs (other places that use entity) are they just has-a not polymorphic already, so doesn't matter?
                //                     %%(the below fields are from Entity, as ref while building this)
                //                 id,
                //                 name: "".to_string(),
                //                 class_id: None,
                //                 insertion_date: -1,
                //                 public: None,
                //                 archived: false,
                //                 new_entries_stick_to_top: false,
            })
        }
        // (See comment in similar spot in BooleanAttribute for why not checking for exists, if db.is_remote.)
    }

    fn get_name_length(&mut self) -> u32 {
        Util::relation_type_name_length()
    }

    pub fn get_name_in_reverse_direction(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<String, anyhow::Error> {
        if !self.already_read_data {
            self.read_data_from_db(transaction)?
        }
        Ok(self.name_in_reverse_direction.clone())
    }

    fn get_directionality(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<String, anyhow::Error> {
        if !self.already_read_data {
            self.read_data_from_db(transaction)?
        }
        Ok(self.directionality.clone())
    }

    pub fn get_name(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<String, anyhow::Error> {
        if !self.already_read_data {
            self.read_data_from_db(transaction)?
        }
        Ok(self.name.clone())
    }

    fn get_display_string_helper(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        _with_color /*IGNOREDFORNOW*/: bool,
    ) -> Result<String, anyhow::Error> {
        let mut entity: Entity = Entity::new2(self.db, None, self.entity_id)?;
        Ok(format!(
            "{}{} (a relation type with: {}/'{}')",
            entity
                .get_archived_status_display_string(transaction.clone())?
                .clone(),
            self.get_name(transaction.clone())?.clone(),
            self.get_directionality(transaction.clone())?.clone(),
            self.get_name_in_reverse_direction(transaction)?.clone()
        ))
    }

    fn read_data_from_db(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<(), anyhow::Error> {
        let data: Vec<Option<DataType>> = self
            .db
            .get_relation_type_data(transaction, self.entity_id)?;
        if data.len() == 0 {
            return Err(anyhow!(
                "No results returned from data request for: {}",
                self.entity_id
            ));
        }
        //(see similar location in boolean_attribute.rs for comments here.)
        self.already_read_data = true;
        self.name = match data[0].clone() {
            Some(DataType::String(x)) => x,
            _ => return Err(anyhow!("How did we get here for {:?}?", data[0])),
        };
        self.name_in_reverse_direction = match data[1].clone() {
            Some(DataType::String(x)) => x,
            _ => return Err(anyhow!("How did we get here for {:?}?", data[1])),
        };
        self.directionality = match data[2].clone() {
            Some(DataType::String(x)) => x,
            _ => return Err(anyhow!("How did we get here for {:?}?", data[2])),
        };
        Ok(())
    }

    fn update(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        name_in: String,
        name_in_reverse_direction_in: String,
        directionality_in: String,
    ) -> Result<(), anyhow::Error> {
        if !self.already_read_data {
            self.read_data_from_db(transaction)?
        };
        if name_in != self.name
            || name_in_reverse_direction_in != self.name_in_reverse_direction
            || directionality_in != self.directionality
        {
            self.db.update_relation_type(
                self.get_id(),
                name_in.clone(),
                name_in_reverse_direction_in.clone(),
                directionality_in.clone(),
            )?;
            self.name = name_in;
            self.name_in_reverse_direction = name_in_reverse_direction_in;
            self.directionality = directionality_in;
        }
        Ok(())
    }

    /// Removes this object from the system.
    pub fn delete<'a>(
        &'a mut self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
    ) -> Result<u64, anyhow::Error> {
        self.db.delete_relation_type(transaction, self.entity_id)
    }

    pub fn get_id(&self) -> i64 {
        self.entity_id
    }
}
