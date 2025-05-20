/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003, 2004, 2010-2017 inclusive, 2020, and 2023-2025 inclusive, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
use crate::model::attribute::Attribute;
use crate::model::boolean_attribute::BooleanAttribute;
use crate::model::database::{DataType, Database};
use crate::model::date_attribute::DateAttribute;
use crate::model::file_attribute::FileAttribute;
use crate::model::group::Group;
use crate::model::id_wrapper::IdWrapper;
use crate::model::relation_to_entity::RelationToEntity;
use crate::model::relation_to_group::RelationToGroup;
use crate::model::relation_to_local_entity::RelationToLocalEntity;
//use crate::model::relation_to_remote_entity::RelationToRemoteEntity;
//use crate::model::postgres::postgresql_database::PostgreSQLDatabase;
use crate::color::Color;
use crate::model::quantity_attribute::QuantityAttribute;
use crate::model::text_attribute::TextAttribute;
use crate::util::Util;
use anyhow::{anyhow, Result};
use chrono::Utc;
use sqlx::{/*Error, */ Postgres, Transaction};
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use tracing::*;

#[derive(Clone)]
pub struct Entity {
    db: Rc<RefCell<dyn Database>>,
    id: i64,
    already_read_data: bool,        /*= false*/
    name: String,                   /*= _*/
    class_id: Option<i64>,          /*= None*/
    insertion_date: i64,            /*= -1*/
    public: Option<bool>,           /*= None*/
    archived: bool,                 /*= false*/
    new_entries_stick_to_top: bool, /*= false*/
}

impl Entity {
    const PRIVACY_PUBLIC: &'static str = "[PUBLIC]";
    const PRIVACY_NON_PUBLIC: &'static str = "[NON-PUBLIC]";
    const PRIVACY_UNSET: &'static str = "[UNSET]";

    /// This one is perhaps only called by the database class implementation--so it can return arrays of objects & save more DB hits
    /// that would have to occur if it only returned arrays of keys. This DOES NOT create a persistent object--but rather should reflect
    /// one that already exists.
    pub fn new(
        db: Rc<RefCell<dyn Database>>,
        id: i64,
        name: String,
        class_id: Option<i64>, /*= None*/
        insertion_date: i64,
        public: Option<bool>,
        archived: bool,
        new_entries_stick_to_top: bool,
    ) -> Entity {
        Entity {
            db,
            id,
            name,
            class_id,
            insertion_date,
            public,
            archived,
            new_entries_stick_to_top,
            already_read_data: true,
        }
    }

    /// Represents one object in the system.
    /// Allows create_entity to return an instance without duplicating the database check that
    /// it Entity(long, Database) does.
    /// This constructor instantiates an existing object from the DB. Generally use Model.createObject()
    /// to create a new object.
    /// Note: Having Entities and other DB objects be readonly makes the code clearer & avoid some bugs,
    /// similarly to reasons for immutability in scala.
    /// (At least that has been the idea. But that might change as I just discovered a case where
    /// that causes a bug and it seems cleaner to have a set... method to fix it.)
    // Idea: replace this w/ a mock? where used? same, for similar code elsewhere like in OmInstance? (and
    // EntityTest etc could be with mocks instead of real db use.)  Does this really skip that other check though?
    pub fn new2(
        db: Rc<RefCell<dyn Database>>,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id: i64,
    ) -> Result<Entity, anyhow::Error> {
        // (See comment in similar spot in BooleanAttribute for why not checking for exists, if db.is_remote.)
        if !db.borrow().is_remote() && !db.borrow().entity_key_exists(transaction, id, true)? {
            return Err(anyhow!("Key {}{}", id, Util::DOES_NOT_EXIST));
        }
        Ok(Entity {
            id,
            db,
            already_read_data: false,
            name: "".to_string(),
            class_id: None,
            insertion_date: -1,
            public: None,
            archived: false,
            new_entries_stick_to_top: false,
        })
    }

    pub fn create_entity(
        db: Rc<RefCell<dyn Database>>,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        in_name: &str,
        in_class_id: Option<i64>,   /*= None*/
        is_public_in: Option<bool>, /*= None*/
    ) -> Result<Entity, anyhow::Error> {
        let id: i64 = db.borrow().create_entity(transaction.clone(), in_name, in_class_id, is_public_in)?;
        //Entity::new2(db as Rc<dyn Database>, transaction.clone(), id)
        Entity::new2(db, transaction.clone(), id)
    }

    fn name_length() -> u32 {
        Util::entity_name_length()
    }

    fn is_duplicate(
        db_in: Rc<RefCell<dyn Database>>,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        in_name: &str,
        in_self_id_to_ignore: Option<i64>, /*= None*/
    ) -> Result<bool, anyhow::Error> {
        db_in.borrow().is_duplicate_entity_name(transaction, in_name, in_self_id_to_ignore)
    }

    /// This is for times when you want None if it doesn't exist, instead of the Error returned by
    /// the Entity constructor.  Or for convenience in tests.
    pub fn get_entity(
        db_in: Rc<RefCell<dyn Database>>,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id: i64,
    ) -> Result<Option<Entity>, String> {
        let e = Entity::new2(db_in, transaction, id);
        match e {
            Ok(entity) => Ok(Some(entity)),
            Err(error) => {
                if error.to_string().contains(Util::DOES_NOT_EXIST) {
                    Ok(None)
                } else {
                    Err(error.to_string())
                }
            }
        }
    }

    /// When using, consider if get_archived_status_display_string should be called with it in the
    /// display (see usage examples of get_archived_status_display_string).
    pub fn get_name(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<&String, anyhow::Error> {
        if !self.already_read_data {
            self.read_data_from_db(transaction)?;
        }
        Ok(&self.name)
    }

    pub fn get_class_id(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<Option<i64>, anyhow::Error> {
        if !self.already_read_data {
            self.read_data_from_db(transaction)?;
        }
        Ok(self.class_id)
    }

    fn get_class_template_entity_id(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<Option<i64>, anyhow::Error> {
        let class_id: Option<i64> = self.get_class_id(transaction.clone())?;
        match class_id {
            None => Ok(None),
            Some(id) => {
                // let template_entity_id: Option<i64> = self.db.get_class_data(transaction, class_id.unwrap())
                // .get(1).asInstanceOf[Option<i64>];
                let row = self.db.borrow().get_class_data(transaction.clone(), id)?;
                let template_entity_id: Result<Option<i64>> = match row.get(1) {
                    None => Err(anyhow!("In get_class_template_entity_id: How got not enough values in the row for id {} ?", id)),
                    Some(Some(DataType::Bigint(i))) => Ok(Some(*i)),
                    _ => Err(anyhow!("In get_class_template_entity_id: How got a blank id or something odd in the row.get(1) for id {} ?", id)),
                };
                template_entity_id
            }
        }
    }

    fn get_creation_date(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<i64, anyhow::Error> {
        if !self.already_read_data {
            self.read_data_from_db(transaction)?;
        }
        Ok(self.insertion_date)
    }

    fn get_creation_date_formatted(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<String, anyhow::Error> {
        // Util::DATEFORMAT.format(new java.util.Date(get_creation_date))
        Ok(Util::useful_date_format(
            self.get_creation_date(transaction)?,
        ))
    }

    fn get_public(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<Option<bool>, anyhow::Error> {
        if !self.already_read_data {
            self.read_data_from_db(transaction)?;
        }
        Ok(self.public)
    }

    fn get_public_status_display_string(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        blank_if_unset: bool, /*= true*/
    ) -> Result<String, anyhow::Error> {
        if !self.already_read_data {
            self.read_data_from_db(transaction)?;
        }
        let result = match self.public {
            Some(p) => {
                if p {
                    Entity::PRIVACY_PUBLIC
                } else {
                    Entity::PRIVACY_NON_PUBLIC
                }
            }
            None => {
                if blank_if_unset {
                    ""
                } else {
                    Entity::PRIVACY_UNSET
                }
            }
        };
        Ok(result.to_string())
    }

    fn get_public_status_display_string_with_color(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        blank_if_unset: bool, /*= true*/
    ) -> Result<String, anyhow::Error> {
        // idea: maybe this (logic) knowledge really belongs in the TextUI class. (As some others, probably.)
        let s = self.get_public_status_display_string(transaction, blank_if_unset)?;
        if s == Entity::PRIVACY_PUBLIC {
            Ok(Color::green(&s))
        } else if s == Entity::PRIVACY_NON_PUBLIC {
            Ok(Color::yellow(&s))
        } else {
            Ok(s)
        }
    }

    fn get_archived_status(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<bool, anyhow::Error> {
        if !self.already_read_data {
            self.read_data_from_db(transaction)?;
        }
        Ok(self.archived)
    }

    pub fn is_archived(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<bool, anyhow::Error> {
        if !self.already_read_data {
            self.read_data_from_db(transaction)?;
        }
        Ok(self.archived)
    }

    fn get_new_entries_stick_to_top(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<bool, anyhow::Error> {
        if !self.already_read_data {
            self.read_data_from_db(transaction)?;
        }
        Ok(self.new_entries_stick_to_top)
    }

    fn get_insertion_date(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<i64, anyhow::Error> {
        if !self.already_read_data {
            self.read_data_from_db(transaction)?;
        }
        Ok(self.insertion_date)
    }

    pub fn get_archived_status_display_string(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<String, anyhow::Error> {
        if !self.already_read_data {
            self.read_data_from_db(transaction.clone())?;
        }
        let result = if !self.is_archived(transaction.clone())? {
            ""
        } else {
            if self.db.borrow().include_archived_entities() {
                "[ARCHIVED]"
            } else {
                return Err(anyhow!("FYI in case this can be better understood and fixed:  due to an error, the program \
                      got an archived entity to display, but this is probably a bug, \
                      because the db setting to show archived entities is turned off. The entity is {} : {}", self.get_id(), self.get_name(transaction.clone())?));
            }
        };
        Ok(result.to_string())
    }

    fn read_data_from_db(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<(), anyhow::Error> {
        let entity_data = self.db.borrow().get_entity_data(transaction, self.id)?;
        if entity_data.len() == 0 {
            return Err(anyhow!(
                "No results returned from data request for: {}",
                self.id
            ));
        }
        //idea: surely there is some better way than what I am doing here? See other places similarly.

        self.name = match &entity_data[0] {
            Some(DataType::String(x)) => x.clone(),
            _ => return Err(anyhow!("How did we get here for {:?}?", entity_data[0])),
        };

        self.class_id = match entity_data[1] {
            Some(DataType::Bigint(x)) => Some(x),
            None => None,
            _ => return Err(anyhow!("How did we get here for {:?}?", entity_data[1])),
        };
        self.public = match entity_data[3] {
            Some(DataType::Boolean(x)) => Some(x),
            None => None,
            _ => return Err(anyhow!("How did we get here for {:?}?", entity_data[3])),
        };

        self.insertion_date = match entity_data[2] {
            Some(DataType::Bigint(x)) => x,
            _ => return Err(anyhow!("How did we get here for {:?}?", entity_data[2])),
        };
        self.archived = match entity_data[4] {
            Some(DataType::Boolean(x)) => x,
            _ => return Err(anyhow!("How did we get here for {:?}?", entity_data[4])),
        };
        self.new_entries_stick_to_top = match entity_data[5] {
            Some(DataType::Boolean(x)) => x,
            _ => return Err(anyhow!("How did we get here for {:?}?", entity_data[5])),
        };
        self.already_read_data = true;
        Ok(())
    }

    fn get_id_wrapper(&self) -> IdWrapper {
        IdWrapper::new(self.id)
    }

    pub fn get_id(&self) -> i64 {
        self.id
    }

    // idea: change this back to a lazy value as it was in scala? To be slightly more efficient?
    /// Intended as a temporarily unique string to distinguish an entity, across OM Instances.  
    /// NOT intended as a permanent unique ID (since the remote address for a given OM instance
    /// can change! and the local address is displayed as blank!), see get_unique_identifier
    /// for that.  This one is like that other in a way, but more for human consumption (eg data
    /// export for human reading, not for re-import -- ?).
    fn get_readable_identifier(&self) -> String {
        let remote_prefix = match self.db.borrow().get_remote_address() {
            None => "".to_string(),
            Some(s) => format!("{}_", s),
        };
        format!("{}{}", remote_prefix, self.get_id().to_string())
    }

    /// Intended as a unique string to distinguish an entity, even across OM Instances.  
    /// Compare to getHumanIdentifier (get_readable_identifier?)
    /// Idea: would any (future?) use cases be better served by including *both* the human-readable address (as in
    /// getHumanIdentifier) and the instance id? Or, just combine the methods into one?
    fn get_unique_identifier(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<String, anyhow::Error> {
        Ok(format!("{}_{}", self.db.borrow().id(transaction)?, self.get_id()))
    }

    fn get_attribute_count(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        include_archived_entities_in: bool, /*= db.include_archived_entities*/
    ) -> Result<u64, anyhow::Error> {
        self.db.borrow()
            .get_attribute_count(transaction, self.get_id(), include_archived_entities_in)
    }

    fn get_relation_to_group_count(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<u64, anyhow::Error> {
        self.db.borrow()
            .get_relation_to_group_count(transaction, self.get_id())
    }

    fn get_display_string_helper(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        with_color: bool,
    ) -> Result<String, anyhow::Error> {
        let mut display_string: String = {
            if with_color {
                format!(
                    "{}{}{}",
                    self.get_public_status_display_string_with_color(transaction.clone(), true)?,
                    self.get_archived_status_display_string(transaction.clone())?,
                    Color::blue(self.get_name(transaction.clone())?)
                )
            } else {
                format!(
                    "{}{}{}",
                    self.get_public_status_display_string(transaction.clone(), true)?,
                    self.get_archived_status_display_string(transaction.clone())?,
                    self.get_name(transaction.clone())?
                )
            }
        };
        let count = self
            .db.borrow()
            .get_class_count(transaction.clone(), Some(self.get_id()))?;
        let definer_info = if count > 0 {
            "template (defining entity) for "
        } else {
            ""
        };
        let class_name: Option<String> = match self.get_class_id(transaction.clone())? {
            Some(class_id) => self.db.borrow().get_class_name(transaction.clone(), class_id)?,
            None => None,
        };
        let sometext = match class_name {
            Some(name) => format!(" ({}class: {})", definer_info, name),
            _ => "".to_string(),
        };
        display_string = format!("{}{}", display_string, sometext);
        Ok(display_string)
    }

    fn get_display_string(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        with_color: bool, /*= false*/
    ) -> Result<String, anyhow::Error> {
        // let mut result = "".to_string();
        // try {
        let result = self.get_display_string_helper(transaction, with_color);
        // } catch {
        // This was the old way in scala. Put back if we want to provide the error message
        // instead of the display string, at some future point instead?
        // If it is put back for that reason, see the test code deleted in the commit on or just after 2025-03-28,
        // labeled:
        //     "get_display_string" should "return a useful stack trace string, when called with a nonexistent entity" in
        //
        //   case e: Exception =>
        //     result += "Unable to get entity description due to: "
        //     result += {
        //       let sw: StringWriter = new StringWriter();
        //       e.printStackTrace(new PrintWriter(sw))
        //       sw.toString
        //     }
        // }
        result
    }

    /// Also for convenience
    fn add_quantity_attribute<'a, 'b>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'b, Postgres>>>>,
        in_attr_type_id: i64,
        in_unit_id: i64,
        in_number: f64,
        sorting_index_in: Option<i64>,
    ) -> Result<QuantityAttribute, anyhow::Error>
    where
        'a: 'b,
    {
        self.add_quantity_attribute2(
            transaction,
            in_attr_type_id,
            in_unit_id,
            in_number,
            sorting_index_in,
            None,
            Utc::now().timestamp_millis(),
        )
    }

    /// Creates a quantity attribute on this Entity (i.e., "6 inches length"), with default values
    /// of "now" for the dates. See "add_quantity_attribute" comment in db implementation file,
    /// for explanation of the parameters.
    /// It might also be nice to add the recorder's ID (person
    /// or app), but we'd have to do some kind of authentication/login 1st? And a GUID for users (as Entities?)?
    /// See PostgreSQLDatabase.create_quantity_attribute(...) for details.
    fn add_quantity_attribute2(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        in_attr_type_id: i64,
        in_unit_id: i64,
        in_number: f64,
        sorting_index_in: Option<i64>, /*= None*/
        in_valid_on_date: Option<i64>,
        observation_date_in: i64,
    ) -> Result<QuantityAttribute, anyhow::Error>
    {
        // write it to the database table--w/ a record for all these attributes plus a key indicating which Entity
        // it all goes with
        let db = self.db.borrow();
        let id: i64 = db.create_quantity_attribute(
            transaction.clone(),
            self.id,
            in_attr_type_id,
            in_unit_id,
            in_number,
            in_valid_on_date,
            observation_date_in,
            sorting_index_in,
        )?;
        return QuantityAttribute::new2(self.db.clone(), transaction.clone(), id);
    }

    fn get_quantity_attribute(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        in_key: i64,
    ) -> Result<QuantityAttribute, anyhow::Error> {
        QuantityAttribute::new2(self.db.clone(), transaction, in_key)
    }

    fn get_text_attribute(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        in_key: i64,
    ) -> Result<TextAttribute, anyhow::Error> {
        TextAttribute::new2(self.db.clone(), transaction, in_key)
    }

    fn get_date_attribute(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        in_key: i64,
    ) -> Result<DateAttribute, anyhow::Error> {
        DateAttribute::new2(self.db.clone(), transaction, in_key)
    }

    fn get_boolean_attribute(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        in_key: i64,
    ) -> Result<BooleanAttribute, anyhow::Error> {
        BooleanAttribute::new2(self.db.clone(), transaction, in_key)
    }

    fn get_file_attribute(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        in_key: i64,
    ) -> Result<FileAttribute, anyhow::Error> {
        FileAttribute::new2(self.db.clone(), transaction, in_key)
    }

    fn get_count_of_containing_groups(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<u64, anyhow::Error> {
        self.db.borrow()
            .get_count_of_groups_containing_entity(transaction, self.get_id())
    }

    fn get_containing_groups_ids(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<Vec<i64>, anyhow::Error> {
        self.db.borrow()
            .get_containing_groups_ids(transaction, self.get_id())
    }

    fn get_containing_relations_to_group(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        starting_index_in: i64,   /*= 0*/
        max_vals_in: Option<i64>, /*= None*/
    ) -> Result<Vec<RelationToGroup>, anyhow::Error> {
        let rtgs_data: Vec<(i64, i64, i64, i64, Option<i64>, i64, i64)> =
            self.db.borrow().get_containing_relations_to_group(
                transaction,
                self.get_id(),
                starting_index_in,
                max_vals_in,
            )?;
        let mut containing_relations_to_group: Vec<RelationToGroup> = Vec::new();
        for rtg_data in rtgs_data {
            let rtg = RelationToGroup::new(
                self.db.clone(),
                rtg_data.0,
                rtg_data.1,
                rtg_data.2,
                rtg_data.3,
                rtg_data.4,
                rtg_data.5,
                rtg_data.6,
            );
            containing_relations_to_group.push(rtg);
        }
        Ok(containing_relations_to_group)
    }

    fn get_containing_relation_to_group_descriptions(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        limit_in: Option<i64>, /*= None*/
    ) -> Result<Vec<String>, anyhow::Error> {
        self.db.borrow()
            .get_containing_relation_to_group_descriptions(transaction, self.get_id(), limit_in)
    }

    fn find_relation_to_and_group(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<(Option<i64>, Option<i64>, Option<i64>, Option<String>, bool), anyhow::Error> {
        self.db.borrow()
            .find_relation_to_and_group_on_entity(transaction, self.get_id(), None)
    }

    //pub fn find_contained_local_entity_ids<'a, 'c>(
    //    &'a self,
    pub fn find_contained_local_entity_ids(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        results_in_out: &mut HashSet<i64>,
        search_string_in: &str,
        levels_remaining_in: i32,      /*= 20*/
        stop_after_any_found_in: bool, /*= true*/
    //) -> Result<&'c mut HashSet<i64>, anyhow::Error> {
    ) -> Result<(), anyhow::Error> {
        self.db.borrow().find_contained_local_entity_ids(
            transaction,
            results_in_out,
            self.get_id(),
            search_string_in,
            levels_remaining_in,
            stop_after_any_found_in,
        )
    }

    fn get_count_of_containing_local_entities(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<(u64, u64), anyhow::Error> {
        self.db.borrow()
            .get_count_of_local_entities_containing_local_entity(transaction, self.get_id())
    }

    fn get_local_entities_containing_entity(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        starting_index_in: i64,   /*= 0*/
        max_vals_in: Option<i64>, /*= None*/
    ) -> Result<Vec<(i64, Entity)>, anyhow::Error> {
        let list: Vec<(i64, i64)> = self.db.borrow().get_local_entities_containing_local_entity(
            transaction.clone(),
            self.get_id(),
            starting_index_in,
            max_vals_in,
        )?;
        let mut result: Vec<(i64, Entity)> = Vec::new();
        for (rel_type_id, entity_id) in list.iter() {
            let entity = Entity::new2(self.db.clone(), transaction.clone(), *entity_id)?;
            result.push((*rel_type_id, entity));
        }
        Ok(result)
    }

    fn get_adjacent_attributes_sorting_indexes(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        sorting_index_in: i64,
        limit_in: Option<i64>,     /*= None*/
        forward_not_back_in: bool, /*= true*/
    ) -> Result<Vec<Vec<Option<DataType>>>, anyhow::Error> {
        self.db.borrow().get_adjacent_attributes_sorting_indexes(
            transaction,
            self.get_id(),
            sorting_index_in,
            limit_in,
            forward_not_back_in,
        )
    }

    fn get_nearest_attribute_entrys_sorting_index(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        starting_point_sorting_index_in: i64,
        forward_not_back_in: bool, /*= true*/
    ) -> Result<Option<i64>, anyhow::Error> {
        self.db.borrow().get_nearest_attribute_entrys_sorting_index(
            transaction,
            self.get_id(),
            starting_point_sorting_index_in,
            forward_not_back_in,
        )
    }

    fn renumber_sorting_indexes(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<(), anyhow::Error> {
        let ref rc_db = &self.db;
        let ref cloned = rc_db.clone();
        let tx = transaction.clone();
        let id = self.get_id();
        cloned.borrow().renumber_sorting_indexes(tx, id, true)
    }

    fn update_attribute_sorting_index(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        attribute_form_id_in: i64,
        attribute_id_in: i64,
        sorting_index_in: i64,
    ) -> Result<u64, anyhow::Error> {
        self.db.borrow().update_attribute_sorting_index(
            transaction,
            self.get_id(),
            attribute_form_id_in,
            attribute_id_in,
            sorting_index_in,
        )
    }

    fn get_attribute_sorting_index(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        attribute_form_id_in: i64,
        attribute_id_in: i64,
    ) -> Result<i64, anyhow::Error> {
        self.db.borrow().get_entity_attribute_sorting_index(
            transaction,
            self.get_id(),
            attribute_form_id_in,
            attribute_id_in,
        )
    }

    fn is_attribute_sorting_index_in_use(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        sorting_index_in: i64,
    ) -> Result<bool, anyhow::Error> {
        self.db.borrow()
            .is_attribute_sorting_index_in_use(transaction, self.get_id(), sorting_index_in)
    }

    fn find_unused_attribute_sorting_index(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        starting_with_in: Option<i64>, /*= None*/
    ) -> Result<i64, anyhow::Error> {
        self.db.borrow()
            .find_unused_attribute_sorting_index(transaction, self.get_id(), starting_with_in)
    }

    fn get_relation_to_local_entity_count(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        include_archived_entities_in: bool, /*= true*/
    ) -> Result<u64, anyhow::Error> {
        self.db.borrow().get_relation_to_local_entity_count(
            transaction,
            self.get_id(),
            include_archived_entities_in,
        )
    }

    fn get_relation_to_remote_entity_count(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<u64, anyhow::Error> {
        self.db.borrow()
            .get_relation_to_remote_entity_count(transaction, self.get_id())
    }

    fn get_text_attribute_by_type_id(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        type_id_in: i64,
        expected_rows_in: Option<usize>, /*= None*/
    ) -> Result<Vec<TextAttribute>, anyhow::Error> {
        let query_results: Vec<(i64, i64, i64, String, Option<i64>, i64, i64)> =
            self.db.borrow().get_text_attribute_by_type_id(
                transaction,
                self.get_id(),
                type_id_in,
                expected_rows_in,
            )?;
        let mut results: Vec<TextAttribute> = Vec::with_capacity(query_results.len());
        for qr in query_results {
            let (
                text_attribute_id,
                parent_entity_id,
                attr_type_id,
                textvalue,
                valid_on_date,
                observation_date,
                sorting_index,
            ) = qr;
            let ta = TextAttribute::new(
                self.db.clone(),
                text_attribute_id,
                parent_entity_id,
                attr_type_id,
                textvalue.as_str(),
                valid_on_date,
                observation_date,
                sorting_index,
            );
            results.push(ta);
        }
        Ok(results)
    }

    // Depending on future callers, should this return instead an Entity and RTLE,
    // creating them here?
    /// @return the new entity_id and relation_to_local_entity_id that relates to it.
    fn add_uri_entity_with_uri_attribute<'a, 'b>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'b, Postgres>>>>,
        new_entity_name_in: &str,
        uri_in: &str,
        observation_date_in: i64,
        make_them_public_in: Option<bool>,
        quote_in: Option<&str>, /*= None*/
    ) -> Result<(i64, i64), anyhow::Error>
    where
        'a: 'b,
    {
        let ref rc_db = &self.db;
        let ref cloned = rc_db.clone();
        let tx = transaction.clone();
        cloned.borrow().add_uri_entity_with_uri_attribute(
            tx,
            self.get_id(),
            new_entity_name_in,
            uri_in,
            observation_date_in,
            make_them_public_in,
            quote_in,
        )
    }

    /// Returns the id of the newly created attribute.
    //%%why do we have both add..() (just below) and create..() here? If import_export.rs (as
    //noted there) can use add_text_attribute or 2 (below) instead, then can delete this.
    fn create_text_attribute<'a, 'b>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'b, Postgres>>>>,
        attr_type_id_in: i64,
        text_in: &str,
        valid_on_date_in: Option<i64>, /*= None*/
        observation_date_in: i64,      /*= Utc::now().timestamp_millis()*/
        sorting_index_in: Option<i64>, /*= None*/
    ) -> Result<i64, anyhow::Error>
    where
        'a: 'b,
    {
        self.db.borrow().create_text_attribute(
            transaction,
            self.get_id(),
            attr_type_id_in,
            text_in,
            valid_on_date_in,
            observation_date_in,
            sorting_index_in,
        )
    }

    /// Returns the count of entities updated.
    pub fn update_contained_entities_public_status(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        new_value_in: Option<bool>,
    ) -> Result<i32, anyhow::Error> {
        let (attr_tuples, _) = self.get_sorted_attributes(transaction.clone(), 0, 0, false)?;
        let attr_tuples_len = attr_tuples.len();
        let mut count = 0;
        for attr in attr_tuples {
            match attr.1 {
                attribute
                    if attribute.get_form_id()?
                        == self
                            .db.borrow()
                            .get_attribute_form_id(Util::RELATION_TO_LOCAL_ENTITY_TYPE)?
                        || attribute.get_form_id()?
                            == self
                                .db.borrow()
                                .get_attribute_form_id(Util::RELATION_TO_REMOTE_ENTITY_TYPE)? =>
                {
                    debug!("gh1 in update_contained_entities_public_status");
                    // (Was) using RelationToEntity here because it actually makes sense. But usually it is
                    // best to make sure to use either RelationToLocalEntity or RelationToRemoteEntity, to
                    // be clearer about the logic.
                    let related_id2 = if attribute.get_form_id()?
                        == self
                            .db.borrow()
                            .get_attribute_form_id(Util::RELATION_TO_LOCAL_ENTITY_TYPE)?
                    {
                        debug!("gh2 in update_contained_entities_public_status");
                        //let rtle = attribute.downcast_ref::<RelationToLocalEntity>()
                        //    .ok_or_else(|| anyhow!("Downcast failed for RelationToLocalEntity"))?;
                        let rtle: Option<RelationToLocalEntity> = RelationToLocalEntity::new3(
                            self.db.clone(),
                            transaction.clone(),
                            attribute.get_id(),
                        )?;
                        match rtle {
                            Some(mut r) => {
                                debug!("gh3 in update_contained_entities_public_status");
                                let related_id1 = r.get_parent_id(transaction.clone())?;
                                if related_id1 != self.get_id() {
                                    return Err(anyhow!("Unexpected value: {}", related_id1));
                                }
                                r.get_related_id2()
                            }
                            None => {
                                return Err(anyhow!(
                                    "No RTLE returned for id {}?",
                                    attribute.get_id()
                                ))
                            }
                        }
                    } else {
                        // %%For remote entities, we'd need to get the ID differently
                        // This would depend on how RelationToRemoteEntity is implemented
                        // For now, let's skip this part until we have RelationToRemoteEntity fully implemented
                        unimplemented!();
                    };
                    let mut e = Entity::new2(self.db.clone(), transaction.clone(), related_id2)?;
                    e.update_public_status(transaction.clone(), new_value_in)?;
                    debug!("gh4 in update_contained_entities_public_status");
                    count += 1;
                }
                attribute
                    if attribute.get_form_id()?
                        == self
                            .db.borrow()
                            .get_attribute_form_id(Util::RELATION_TO_GROUP_TYPE)? =>
                {
                    debug!("gh5 in update_contained_entities_public_status");
                    //let rtg = attribute.downcast_ref::<RelationToGroup>()
                    //    .ok_or_else(|| anyhow!("Downcast failed for RelationToGroup"))?;
                    let mut rtg = RelationToGroup::new3(
                        self.db.clone(),
                        transaction.clone(),
                        attribute.get_id(),
                    )?;
                    let group_id = rtg.get_group_id(transaction.clone())?;
                    let entries: Vec<Vec<Option<DataType>>> = self.db.borrow().get_group_entries_data(
                        transaction.clone(),
                        group_id,
                        None,
                        false,
                    )?;
                    for entry in entries {
                        debug!("gh6 in update_contained_entities_public_status");
                        if let Some(DataType::Bigint(entity_id)) = entry[0] {
                            debug!("gh7 in update_contained_entities_public_status");
                            self.db.borrow().update_entity_only_public_status(
                                transaction.clone(),
                                entity_id,
                                new_value_in,
                            )?;
                            count += 1;
                        } else {
                            return Err(anyhow!("unexpected value: {:?}", entry));
                        }
                    }
                }
                _ => {
                    debug!("Value for attr.1 is: {:?}", attr.1);
                    // do nothing
                }
            }
        }
        debug!("gh8 in update_contained_entities_public_status; count is {:?}, attr_tuples.len() is {:?}", 
            count, attr_tuples_len);
        Ok(count)
    }

    /// See add_quantity_attribute(...) methods for comments.
    pub fn add_text_attribute<'a, 'b>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'b, Postgres>>>>,
        in_attr_type_id: i64,
        in_text: &str,
        sorting_index_in: Option<i64>,
    ) -> Result<TextAttribute, anyhow::Error>
    where
        'a: 'b,
    {
        self.add_text_attribute2(
            transaction.clone(),
            in_attr_type_id,
            in_text,
            sorting_index_in,
            None,
            Utc::now().timestamp_millis(),
        )
    }

    pub fn add_text_attribute2<'a, 'b>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'b, Postgres>>>>,
        in_attr_type_id: i64,
        in_text: &str,
        sorting_index_in: Option<i64>,
        in_valid_on_date: Option<i64>,
        observation_date_in: i64,
    ) -> Result<TextAttribute, anyhow::Error>
    where
        'a: 'b,
    {
        let ref rc_db = &self.db;
        let ref cloned = rc_db.clone();
        let id = cloned.borrow().create_text_attribute(
            transaction.clone(),
            self.id,
            in_attr_type_id,
            in_text,
            in_valid_on_date,
            observation_date_in,
            sorting_index_in,
        )?;
        TextAttribute::new2(self.db.clone(), transaction, id)
    }

    fn add_date_attribute<'a, 'b>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'b, Postgres>>>>,
        in_attr_type_id: i64,
        in_date: i64,
        sorting_index_in: Option<i64>, /*= None*/
    ) -> Result<DateAttribute, anyhow::Error>
    where
        'a: 'b,
    {
        let id = self.db.borrow().create_date_attribute(
            transaction.clone(),
            self.id,
            in_attr_type_id,
            in_date,
            sorting_index_in,
        )?;
        DateAttribute::new2(self.db.clone(), transaction, id)
    }

    fn add_boolean_attribute<'a, 'b>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'b, Postgres>>>>,
        in_attr_type_id: i64,
        in_boolean: bool,
        sorting_index_in: Option<i64>,
    ) -> Result<BooleanAttribute, anyhow::Error>
    where
        'a: 'b,
    {
        self.add_boolean_attribute2(
            transaction,
            in_attr_type_id,
            in_boolean,
            sorting_index_in,
            None,
            Utc::now().timestamp_millis(),
        )
    }

    fn add_boolean_attribute2<'a, 'b>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'b, Postgres>>>>,
        in_attr_type_id: i64,
        in_boolean: bool,
        sorting_index_in: Option<i64>, /*= None*/
        in_valid_on_date: Option<i64>,
        observation_date_in: i64,
    ) -> Result<BooleanAttribute, anyhow::Error>
    where
        'a: 'b,
    {
        let id = self.db.borrow().create_boolean_attribute(
            transaction.clone(),
            self.id,
            in_attr_type_id,
            in_boolean,
            in_valid_on_date,
            observation_date_in,
            sorting_index_in,
        )?;
        BooleanAttribute::new2(self.db.clone(), transaction, id)
    }

    /*%%file_attr later when impld:
                    fn add_file_attribute(in_attr_type_id: i64, inFile: java.io.File) -> FileAttribute {
                    add_file_attribute(in_attr_type_id, inFile.get_name, inFile)
                  }

                    fn add_file_attribute(in_attr_type_id: i64, description_in: String, inFile: java.io.File, sorting_index_in: Option<i64> = None) -> FileAttribute {
                    if !inFile.exists() {
                      throw new Exception("File " + inFile.getCanonicalPath + " doesn't exist.")
                    }
                    // idea: could be a little faster if the md5_hash method were merged into the database method, so that the file is only traversed once (for both
                    // upload and md5 calculation).
                    let mut inputStream: java.io.FileInputStream = null;
                    try {
                      inputStream = new FileInputStream(inFile)
                      let id = db.create_file_attribute(id, in_attr_type_id, description_in, inFile.lastModified, Utc::now().timestamp_millis(), inFile.getCanonicalPath,;
                                                       inFile.canRead, inFile.canWrite, inFile.canExecute, inFile.length, FileAttribute::md5_hash(inFile), inputStream,
                                                       sorting_index_in)
                      FileAttribute::new(db, id)
                    }
                    finally {
                      if inputStream != null) {
                        inputStream.close()
                      }
                    }
                  }
    */
    fn add_relation_to_local_entity<'a, 'b>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'b, Postgres>>>>,
        in_attr_type_id: i64,
        in_entity_id2: i64,
        sorting_index_in: Option<i64>,
        in_valid_on_date: Option<i64>, /*= None*/
        in_observation_date: i64,      /*= Utc::now().timestamp_millis()*/
    ) -> Result<RelationToLocalEntity, anyhow::Error>
    where
        'a: 'b,
    {
        let (rte_id, new_sorting_index) = self.db.borrow().create_relation_to_local_entity(
            transaction.clone(),
            in_attr_type_id,
            self.get_id(),
            in_entity_id2,
            in_valid_on_date,
            in_observation_date,
            sorting_index_in,
        )?;
        Ok(RelationToLocalEntity::new(
            self.db.clone(),
            rte_id,
            in_attr_type_id,
            self.get_id(),
            in_entity_id2,
            in_valid_on_date,
            in_observation_date,
            new_sorting_index,
        ))
    }

    /*%%put back after converting RelationToRemoteEntity to Rust
    fn add_relation_to_remote_entity(&self,
                                        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
                                         in_attr_type_id: i64, in_entity_id2: i64, sorting_index_in: Option<i64>,
                              in_valid_on_date: Option<i64> /*= None*/, observation_date_in: i64 /*= Utc::now().timestamp_millis()*/,
                              remote_instance_id_in: String)
    -> Result<RelationToRemoteEntity, anyhow::Error> {
        let rte_id = self.db.create_relation_to_remote_entity(transaction.clone(), in_attr_type_id, self.get_id(), in_entity_id2, in_valid_on_date, observation_date_in, remote_instance_id_in, sorting_index_in, false);
        RelationToRemoteEntity::new2(self.db, rte_id, in_attr_type_id, self.get_id(), remote_instance_id_in, in_entity_id2)
      }
                              */

    /// Creates then adds a particular kind of rtg to this entity.
    /// Returns new group's id, and the new RelationToGroup object
    fn create_group_and_add_a_has_relation_to_it<'a, 'b>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'b, Postgres>>>>,
        new_group_name_in: &str,
        mixed_classes_allowed_in: bool,
        observation_date_in: i64,
    ) -> Result<(i64, i64), anyhow::Error>
    where
        'a: 'b,
    {
        // the "has" relation type that we want should always be the 1st one, since it is created by in the initial app startup; otherwise it seems we can use it
        // anyway:
        let relation_type_id = self
            .db.borrow()
            .find_relation_type(transaction.clone(), Util::THE_HAS_RELATION_TYPE_NAME)?; //, Some(1)).get(0);
        let (group_id, rtg_id) = self.add_group_and_relation_to_group(
            transaction.clone(),
            relation_type_id,
            new_group_name_in,
            mixed_classes_allowed_in,
            None,
            observation_date_in,
            None,
        )?;
        Ok((group_id, rtg_id))
    }

    /// Like others, returns the new things' IDs. */
    pub fn add_group_and_relation_to_group(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        rel_type_id_in: i64,
        new_group_name_in: &str,
        allow_mixed_classes_in_group_in: bool, /*= false*/
        valid_on_date_in: Option<i64>,
        observation_date_in: i64,
        sorting_index_in: Option<i64>,
    ) -> Result<(i64, i64), anyhow::Error> {
        let ref rc_db = self.db;
        let ref cloned = rc_db.clone();
        let tx = transaction.clone();
        let id = self.get_id();
        let (group_id, rtg_id) = cloned.borrow().create_group_and_relation_to_group(
            tx,
            id,
            rel_type_id_in,
            new_group_name_in,
            allow_mixed_classes_in_group_in,
            valid_on_date_in,
            observation_date_in,
            sorting_index_in,
        )?;
        let group: Group = Group::new2(self.db.clone(), transaction.clone(), group_id)?;
        let rtg = RelationToGroup::new2(
            self.db.clone(),
            transaction.clone(),
            rtg_id,
            self.get_id(),
            rel_type_id_in,
            group_id,
        )?;
        Ok((group.get_id(), rtg.get_id()))
    }

    /// @return the id of the new RTE
    fn add_has_relation_to_local_entity(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        entity_id_in: i64,
        valid_on_date_in: Option<i64>,
        observation_date_in: i64,
    ) -> Result<RelationToLocalEntity, anyhow::Error> {
        let ref rc_db = self.db;
        let ref cloned = rc_db.clone();
        let (rel_id, has_rel_type_id, new_sorting_index) = cloned.borrow()
            .add_has_relation_to_local_entity(
                transaction,
                self.get_id(),
                entity_id_in,
                valid_on_date_in,
                observation_date_in,
                None,
            )?;
        let rtle: RelationToLocalEntity = RelationToLocalEntity::new(
            self.db.clone(),
            rel_id,
            has_rel_type_id,
            self.get_id(),
            entity_id_in,
            valid_on_date_in,
            observation_date_in,
            new_sorting_index,
        );
        Ok(rtle)
    }

    /// Creates new entity then adds it a particular kind of rte to this entity.
    pub fn create_entity_and_add_has_local_relation_to_it<'a, 'b>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'b, Postgres>>>>,
        new_entity_name_in: &str,
        observation_date_in: i64,
        is_public_in: Option<bool>,
    ) -> Result<(Entity, RelationToLocalEntity), anyhow::Error>
    where
        'a: 'b,
    {
        let (new_entity_id, rte_id, relation_type_id) =
            self.db.borrow().create_entity_and_add_has_local_relation_to_it(
                transaction.clone(),
                self.get_id(),
                new_entity_name_in,
                observation_date_in,
                is_public_in,
            )?;
        //idea: would be faster (no db hit) if we called Entity::new here instead, with the data
        //as needed returned from the fn call just above, instead of Entity::new2. Might have
        //to add some return values to Entity::new.
        let new_entity: Entity = Entity::new2(self.db.clone(), transaction.clone(), new_entity_id)?;
        //idea: ditto the comment just above.
        let rte: RelationToLocalEntity = RelationToLocalEntity::new2(
            self.db.clone(),
            transaction.clone(),
            rte_id,
            relation_type_id,
            self.get_id(),
            new_entity_id,
        )?;
        Ok((new_entity, rte))
    }

    fn add_entity_and_relation_to_local_entity<'a, 'b>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'b, Postgres>>>>,
        rel_type_id_in: i64,
        new_entity_name_in: &str,
        valid_on_date_in: Option<i64>,
        observation_date_in: i64,
        is_public_in: Option<bool>,
    ) -> Result<(Entity, RelationToLocalEntity), anyhow::Error>
    where
        'a: 'b,
    {
        let (new_entity_id, new_rte_id) = self.db.borrow().create_entity_and_relation_to_local_entity(
            transaction.clone(),
            self.get_id(),
            rel_type_id_in,
            new_entity_name_in,
            is_public_in,
            valid_on_date_in,
            observation_date_in,
        )?;
        //idea: for speed of next 2 lines, see comment at equivalent point
        //in create_entity_and_add_has_local_relation_to_it().
        let entity = Entity::new2(self.db.clone(), transaction.clone(), new_entity_id)?;
        let rte = RelationToLocalEntity::new2(
            self.db.clone(),
            transaction,
            new_rte_id,
            rel_type_id_in,
            self.get_id(),
            new_entity_id,
        )?;
        Ok((entity, rte))
    }

    /// @return the new group's id.
    pub fn add_relation_to_group<'a, 'b>(
        &'a self,
        tx: Option<Rc<RefCell<Transaction<'b, Postgres>>>>,
        rel_type_id_in: i64,
        group_id_in: i64,
        sorting_index_in: Option<i64>,
    ) -> Result<RelationToGroup, anyhow::Error>
    where
        'a: 'b,
    {
        self.add_relation_to_group2(
            tx.clone(),
            rel_type_id_in,
            group_id_in,
            sorting_index_in,
            None,
            Utc::now().timestamp_millis(),
        )
    }

    fn add_relation_to_group2<'a, 'b>(
        &'a self,
        tx: Option<Rc<RefCell<Transaction<'b, Postgres>>>>,
        rel_type_id_in: i64,
        group_id_in: i64,
        sorting_index_in: Option<i64>,
        valid_on_date_in: Option<i64>,
        observation_date_in: i64,
    ) -> Result<RelationToGroup, anyhow::Error>
    where
        'a: 'b,
    {
        let ref rc_db = &self.db;
        let ref cloned = rc_db.clone();
        let local_tx = tx.clone();
        let (new_rtg_id, sorting_index) = cloned.borrow().create_relation_to_group(
            local_tx,
            self.get_id(),
            rel_type_id_in,
            group_id_in,
            valid_on_date_in,
            observation_date_in,
            sorting_index_in,
        )?;
        Ok(RelationToGroup::new(
            self.db.clone(),
            new_rtg_id,
            self.get_id(),
            rel_type_id_in,
            group_id_in,
            valid_on_date_in,
            observation_date_in,
            sorting_index,
        ))
    }

    pub fn get_sorted_attributes(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        starting_object_index_in: usize, /*= 0*/
        max_vals_in: usize,              /*= 0*/
        only_public_entities_in: bool,   /*= true*/
    ) -> Result<(Vec<(i64, Rc<dyn Attribute>)>, usize), anyhow::Error> {
        self.db.borrow().get_sorted_attributes(
            self.db.clone(),
            transaction,
            self.get_id(),
            starting_object_index_in,
            max_vals_in,
            only_public_entities_in,
        )
    }

    pub fn update_class<'a, 'b>(
        &'a mut self,
        transaction: Option<Rc<RefCell<Transaction<'b, Postgres>>>>,
        class_id_in: Option<i64>,
    ) -> Result<(), anyhow::Error>
    where
        'a: 'b,
    {
        if !self.already_read_data {
            self.read_data_from_db(transaction.clone())?;
        }
        if class_id_in != self.class_id {
            self.db.borrow()
                .update_entitys_class(transaction, self.get_id(), class_id_in)?;
            self.class_id = class_id_in;
        }
        Ok(())
    }

    pub fn update_new_entries_stick_to_top(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        b: bool,
    ) -> Result<(), anyhow::Error> {
        if !self.already_read_data {
            self.read_data_from_db(transaction.clone())?;
        }
        if b != self.new_entries_stick_to_top {
            self.db.borrow()
                .update_entity_only_new_entries_stick_to_top(transaction, self.get_id(), b)?;
            self.new_entries_stick_to_top = b;
        }
        Ok(())
    }

    pub fn update_public_status(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        new_value_in: Option<bool>,
    ) -> Result<(), anyhow::Error> {
        if !self.already_read_data {
            self.read_data_from_db(transaction.clone())?;
        }
        if new_value_in != self.public {
            // The condition for this (when it was part of EntityMenu) used to include
            // " && !entity_in.isInstanceOf[RelationType]", but maybe it's better w/o that.
            self.db.borrow()
                .update_entity_only_public_status(transaction, self.get_id(), new_value_in)?;
            self.public = new_value_in;
        }
        Ok(())
    }

    pub fn update_name(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        name_in: &str,
    ) -> Result<(), anyhow::Error> {
        if !self.already_read_data {
            self.read_data_from_db(transaction.clone())?;
        }
        if name_in != self.name {
            self.db.borrow()
                .update_entity_only_name(transaction, self.get_id(), name_in)?;
            self.name = name_in.to_string();
        }
        Ok(())
    }

    pub fn archive(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<(), anyhow::Error> {
        self.db.borrow().archive_entity(transaction, self.get_id())?;
        self.archived = true;
        Ok(())
    }

    pub fn unarchive(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<(), anyhow::Error> {
        self.db.borrow().unarchive_entity(transaction, self.get_id())?;
        self.archived = false;
        Ok(())
    }

    /// Removes this object from the system.
    pub fn delete<'a, 'b>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'b, Postgres>>>>,
    ) -> Result<(), anyhow::Error>
    where
        'a: 'b,
    {
        self.db.borrow().delete_entity(transaction, self.get_id())
    }
}

#[cfg(test)]
mod test {
    use super::Entity;
    use crate::color::Color;
    use crate::model::attribute::Attribute;
    use crate::model::attribute_with_valid_and_observed_dates::AttributeWithValidAndObservedDates;
    use crate::model::boolean_attribute::BooleanAttribute;
    use crate::model::database::{DataType, Database};
    use crate::model::date_attribute::DateAttribute;
    use crate::model::file_attribute::FileAttribute;
    use crate::model::group::Group;
    use crate::model::id_wrapper::IdWrapper;
    use crate::model::postgres::postgresql_database::PostgreSQLDatabase;
    use crate::model::quantity_attribute::QuantityAttribute;
    use crate::model::relation_to_group::RelationToGroup;
    use crate::model::relation_to_local_entity::RelationToLocalEntity;
    use crate::model::text_attribute::TextAttribute;
    use crate::util::Util;
    use anyhow::{anyhow, Result};
    use chrono::Utc;
    use sqlx::{/*Error, */ Postgres, Transaction};
    use std::cell::RefCell;
    use std::collections::HashSet;
    use std::rc::Rc;
    use tracing::*;

    #[test]
    fn test_add_quantity_attribute() {
        Util::initialize_tracing();
        let db: Rc<RefCell<PostgreSQLDatabase> >= Rc::new(RefCell::new(Util::initialize_test_db().unwrap()));
        let e = Entity::create_entity(db.clone(), None, "testEntityName", None, None).unwrap();
        //Using None instead of tx here for simplicity, but might have to change if
        //running tests in parallel.
        //let tx = db.begin_trans().unwrap();
        //let tx = Some(Rc::new(RefCell::new(tx)));
        let tx = None;

        //%%latertests Maybe move things like this into a setup method that does "once" like util.rs does, equiv to what
        //I used to do in Scala? Or one that returns the various values? or just fill in & keep copy/pasting as needed?
        //Or look up the once_cell crate -- would it help fill in vars w/ boxes? at what level? *singletons*?
        //Same for other tests below?
        let quantity_attr_type_id =
            Entity::create_entity(db.clone(), tx.clone(), "length", None, None)
                .unwrap()
                .get_id();
        let unit_id = Entity::create_entity(db.clone(), tx.clone(), "centimeters", None, None)
            .unwrap()
            .get_id();
        let qa: QuantityAttribute = e
            .add_quantity_attribute(tx.clone(), quantity_attr_type_id, unit_id, 100.0, None)
            .unwrap();
        let qa_retrieved: QuantityAttribute =
            e.get_quantity_attribute(tx.clone(), qa.get_id()).unwrap();
        assert_eq!(qa_retrieved.get_id(), qa.get_id());
    }

    #[test]
    fn test_add_text_attribute() {
        Util::initialize_tracing();
        let db: Rc<RefCell<PostgreSQLDatabase> >= Rc::new(RefCell::new(Util::initialize_test_db().unwrap()));
        let entity = Entity::create_entity(db.clone(), None, "testEntityName", None, None).unwrap();
        //Using None instead of tx here for simplicity, but might have to change if
        //running tests in parallel.
        //let tx = db.begin_trans().unwrap();
        //let tx = Some(Rc::new(RefCell::new(tx)));
        let tx = None;
        println!("starting testAddTextAttribute");
        let text_attr_type: Entity =
            Entity::create_entity(db.clone(), tx.clone(), "description", None, None).unwrap();
        let text_attr_type_id: i64 = text_attr_type.get_id();
        let text = "This is some text given to an object";
        let ta = entity
            .add_text_attribute(tx.clone(), text_attr_type_id, text, None)
            .unwrap();
        let ta_retrieved = entity.get_text_attribute(tx.clone(), ta.get_id()).unwrap();
        assert_eq!(ta_retrieved.get_id(), ta.get_id());
    }

    #[test]
    fn test_add_date_attribute() {
        Util::initialize_tracing();
        let db: Rc<RefCell<PostgreSQLDatabase>> = Rc::new(RefCell::new(Util::initialize_test_db().unwrap()));
        let entity = Entity::create_entity(db.clone(), None, "testEntityName", None, None).unwrap();
        //Using None instead of tx here for simplicity, but might have to change if
        //running tests in parallel.
        //let tx = db.begin_trans().unwrap();
        //let tx = Some(Rc::new(RefCell::new(tx)));
        let tx = None;
        println!("starting testAddDateAttribute");
        let date_attr_type_id =
            Entity::create_entity(db.clone(), tx.clone(), "birthdate", None, None)
                .unwrap()
                .get_id();
        let date_value = 2; // Using simple value for test
        let da = entity
            .add_date_attribute(tx.clone(), date_attr_type_id, date_value, None)
            .unwrap();
        let mut da_retrieved = entity.get_date_attribute(tx.clone(), da.get_id()).unwrap();
        assert_eq!(da_retrieved.get_id(), da.get_id());
        assert_eq!(
            da_retrieved.get_attr_type_id(tx.clone()).unwrap(),
            date_attr_type_id
        );
        assert_eq!(da_retrieved.get_date(tx.clone()).unwrap(), date_value);
    }

    #[test]
    fn test_add_boolean_attribute() {
        Util::initialize_tracing();
        let db: Rc<RefCell<PostgreSQLDatabase>> = Rc::new(RefCell::new(Util::initialize_test_db().unwrap()));
        let entity = Entity::create_entity(db.clone(), None, "testEntityName", None, None).unwrap();
        //Using None instead of tx here for simplicity, but might have to change if
        //running tests in parallel.
        //let tx = db.begin_trans().unwrap();
        //let tx = Some(Rc::new(RefCell::new(tx)));
        let tx = None;
        println!("starting testAddBooleanAttribute");
        let start_time = Utc::now().timestamp_millis();
        let boolean_attr_type_id: i64 =
            Entity::create_entity(db.clone(), tx.clone(), "isComplete", None, None)
                .unwrap()
                .get_id();
        let ba = entity
            .add_boolean_attribute(tx.clone(), boolean_attr_type_id, true, None)
            .unwrap();
        let mut ba_retrieved: BooleanAttribute = entity
            .get_boolean_attribute(tx.clone(), ba.get_id())
            .unwrap();
        assert_eq!(ba_retrieved.get_id(), ba.get_id());
        assert_eq!(ba_retrieved.get_boolean(tx.clone()).unwrap(), true);
        assert_eq!(
            ba_retrieved.get_parent_id(tx.clone()).unwrap(),
            entity.get_id()
        );
        assert!(ba_retrieved
            .get_valid_on_date(tx.clone())
            .unwrap()
            .is_none());
        let observation_date = ba_retrieved.get_observation_date(tx.clone()).unwrap();
        assert!(
            observation_date > (start_time - 1)
                && observation_date < (Utc::now().timestamp_millis() + 1)
        );
    }

    /* %%file_attr: put back next test after file_attribute things are more implemented
          "testAddFileAttribute" should "also work" in {
            db.begin_trans()
            let mut file: java.io.File = null;
            let mut fw: java.io.FileWriter = null;
            println!("starting testAddFileAttribute")
            try {
              file = java.io.File.createTempFile("om-test-file-attr-", null)
              fw = new java.io.FileWriter(file)
              fw.write("1234" + new String("\n"))
              fw.close()
              assert(FileAttribute::md5_hash(file) == "e7df7cd2ca07f4f1ab415d457a6e1c13")
              let path = file.getCanonicalPath;
              let id0: i64 = mEntity.add_file_attribute(mFileAttrTypeId, file).get_id;
              let t0: FileAttribute = mEntity.get_file_attribute(id0);
              assert(t0 != null)
              assert(t0.get_id == id0)
              assert(t0.get_description() == file.get_name)

              let id: i64 = mEntity.add_file_attribute(mFileAttrTypeId, "file desc here, long or short", file).get_id;
              let t: FileAttribute = mEntity.get_file_attribute(id);
              assert(t.get_parent_id() == mEntity.get_id)
              assert(t.get_attr_type_id() == mFileAttrTypeId)
              assert(t.get_description() == "file desc here, long or short")
              assert(t.get_original_file_date() > 1389461364000L)
              let now = Utc::now().timestamp_millis();
              assert(t.get_stored_date() < now && t.get_stored_date() > now - (5 * 1000 * 60))
              assert(t.get_original_file_path() == path)
              assert(t.self.get_readable())
              assert(t.get_writeable())
              assert(!t.get_executable())
              assert(t.get_size == 5)
            }
            finally {
              if fw != null { fw.close() }
              if file != null { file.delete() }
            }
            db.rollback_trans()
          }
    */

    #[test]
    fn test_display_string() {
        Util::initialize_tracing();
        let db: Rc<RefCell<dyn Database>> = Rc::new(RefCell::new(Util::initialize_test_db().unwrap()));
        //Using None instead of tx here for simplicity, but might have to change if
        //running tests in parallel.
        //let tx = db.begin_trans().unwrap();
        //let tx = Some(Rc::new(RefCell::new(tx)));
        let tx = None;

        let class_name = "class1Name";
        let (class_id, template_entity_id) = db.borrow()
            .create_class_and_its_template_entity(tx.clone(), class_name)
            .unwrap();
        let mut entity =
            Entity::create_entity(db.clone(), tx.clone(), "entity1Name", Some(class_id), None)
                .unwrap();
        let display_string = entity.get_display_string(tx.clone(), false).unwrap();
        // Inconvenient to test exact string due to color formatting, so check for important parts
        assert!(display_string.contains("entity1Name"));
        assert!(display_string.contains(&format!("class: {}", class_name)));
        // Second test: Entity without class
        let mut entity2 =
            Entity::create_entity(db.clone(), tx.clone(), "entity2Name", None, None).unwrap();
        let display_string2 = entity2.get_display_string(tx.clone(), false).unwrap();
        assert_eq!(display_string2, "entity2Name");
        assert!(!display_string2.contains("class:"));
        // Third test: Entity as template
        // Create an entity that is a template for a class
        let display_string3 = Entity::new2(db.clone(), tx.clone(), template_entity_id)
            .unwrap()
            .get_display_string(tx.clone(), false)
            .unwrap();
        debug!("display_string3 is: {}", display_string3);
        assert!(display_string3.contains("(template (defining entity) for class: "));
        assert!(display_string3.contains(class_name));
        //assert(entity2.get_display_string == name2 + " (template entity (template) for class: " + "class2Name)")
    }

    #[test]
    fn test_get_class_template_entity_id() {
        Util::initialize_tracing();
        let db: Rc<RefCell<PostgreSQLDatabase>> = Rc::new(RefCell::new(Util::initialize_test_db().unwrap()));
        //Using None instead of tx here for simplicity, but might have to change if
        //running tests in parallel.
        //let tx = db.begin_trans().unwrap();
        //let tx = Some(Rc::new(RefCell::new(tx)));
        let tx = None;
        let class_name = "classname";
        let (class_id, template_entity_id) = db.borrow()
            .create_class_and_its_template_entity(tx.clone(), class_name)
            .unwrap();
        // Entity without class
        let mut entity =
            Entity::create_entity(db.clone(), tx.clone(), "entityname", None, None).unwrap();
        assert!(entity
            .get_class_template_entity_id(tx.clone())
            .unwrap()
            .is_none());
        // Entity with class
        let mut entity2 =
            Entity::create_entity(db.clone(), tx.clone(), "entityname", Some(class_id), None)
                .unwrap();
        assert_eq!(
            entity2
                .get_class_template_entity_id(tx.clone())
                .unwrap()
                .unwrap(),
            template_entity_id
        );
    }

    #[test]
    fn test_update_contained_entities_public_status() {
        Util::initialize_tracing();
        let db: Rc<RefCell<PostgreSQLDatabase>> = Rc::new(RefCell::new(Util::initialize_test_db().unwrap()));
        let container = Entity::create_entity(db.clone(), None, "container", None, None).unwrap();
        let mut entity1 =
            Entity::create_entity(db.clone(), None, "test object1", None, None).unwrap();
        let mut entity2 =
            Entity::create_entity(db.clone(), None, "test object2", None, None).unwrap();
        debug!(
            "In test_update_contained_entities_public_status, 3 entities' ids: {}, {}, {}.",
            container.get_id(),
            entity1.get_id(),
            entity2.get_id()
        );
        let rel_type_id = db.borrow()
            .find_relation_type(None, Util::THE_HAS_RELATION_TYPE_NAME)
            .unwrap();
        let (group_id, _) = container
            .add_group_and_relation_to_group(
                None, //tx.clone(),
                rel_type_id,
                "grpName",
                true,
                None,
                Utc::now().timestamp_millis(),
                None,
            )
            .unwrap();
        let group = Group::new2(db.clone(), None, group_id).unwrap();
        //Using None instead of tx here for simplicity, but might have to change if
        //running tests in parallel.
        //let tx = db.begin_trans().unwrap();
        //let tx = Some(Rc::new(RefCell::new(tx)));
        let tx = None;

        // Create container entity
        // Create entity (above) and add has relation to container
        container
            .add_has_relation_to_local_entity(
                tx.clone(),
                entity1.get_id(),
                None,
                Utc::now().timestamp_millis(),
            )
            .unwrap();
        // Create group (above) and add relation to container (above, for transaction lifetime issues)
        // Create entity2 (above) and add to group
        group
            .add_entity(tx.clone(), entity2.get_id(), None)
            .unwrap();
        // Check initial status
        //let mut e1 = Entity::new2(db.clone(), tx.clone(), entity1.get_id()).unwrap();
        //let mut e2 = Entity::new2(db.clone(), tx.clone(), entity2.get_id()).unwrap();
        assert!(entity1.get_public(tx.clone()).unwrap().is_none());
        assert!(entity2.get_public(tx.clone()).unwrap().is_none());
        // Update status
        let count = container
            .update_contained_entities_public_status(tx.clone(), Some(true))
            .unwrap();
        assert_eq!(count, 2);
        // Check updated status
        let mut e1_reread = Entity::new2(db.clone(), tx.clone(), entity1.get_id()).unwrap();
        let mut e2_reread = Entity::new2(db.clone(), tx.clone(), entity2.get_id()).unwrap();
        assert_eq!(e1_reread.get_public(tx.clone()).unwrap(), Some(true));
        assert_eq!(e2_reread.get_public(tx.clone()).unwrap(), Some(true));
    }

    /*
      "get_count_of_containing_local_entities etc" should "work" in {
        let e1 = Entity.create_entity(db, "e1");
        let (e2id: i64, rteId: i64) = db.create_entityAndRelationToLocalEntity(e1.get_id, mRelationTypeId, "e2", None, None, 0L);
        let e2: Option<Entity> = Entity.getEntity(db, e2id);
        assert(e2.get.get_count_of_containing_local_entities._1 == 1)
        assert(e2.get.get_local_entities_containing_entity().size == 1)
        /*val (e3id: i64, rte2id: i64) = */db.create_entityAndRelationToLocalEntity(e1.get_id, mRelationTypeId, "e3", None, None, 0L)
        assert(e1.get_adjacent_attributes_sorting_indexes(Database::min_id_value).nonEmpty)
        let nearestSortingIndex = e1.get_nearest_attribute_entrys_sorting_index(Database::min_id_value).get;
        assert(nearestSortingIndex > Database::min_id_value)
        e1.renumber_sorting_indexes()
        let nearestSortingIndex2 = e1.get_nearest_attribute_entrys_sorting_index(Database::min_id_value).get;
        assert(nearestSortingIndex2 > nearestSortingIndex)

        //let rte = RelationToLocalEntity.get_relation_to_local_entity(db, rteId).get;
        let rte = RelationToLocalEntity::new3(db, rteId).get;
        assert(! e1.is_attribute_sorting_index_in_use(Database::max_id_value))
        e1.update_attribute_sorting_index(rte.get_form_id, rte.get_id, Database::max_id_value)
        assert(e1.get_attribute_sorting_index(rte.get_form_id, rte.get_id) == Database::max_id_value)
        assert(e1.is_attribute_sorting_index_in_use(Database::max_id_value))
        assert(e1.find_unused_attribute_sorting_index() != Database::max_id_value)
        assert(e1.get_relation_to_local_entity_count() == 2)
        e2.get.archive()
        assert(e1.get_relation_to_local_entity_count(include_archived_entities_in = false) == 1)
        assert(e1.get_relation_to_local_entity_count(include_archived_entities_in = true) == 2)
        assert(e1.get_text_attribute_by_type_id(mRelationTypeId).size == 0)
        e1.add_text_attribute(mRelationTypeId, "abc", None)
        assert(e1.get_text_attribute_by_type_id(mRelationTypeId).size == 1)

        assert(Entity.getEntity(db, e1.get_id).get.get_name != "updated")
        e1.updateName("updated")
        assert(Entity.getEntity(db, e1.get_id).get.get_name == "updated")
        assert(Entity.is_duplicate(db, "updated"))
        assert(! Entity.is_duplicate(db, "xyzNOTANAMEupdated"))

        let g1 = Group.create_group(db, "g1");
        g1.add_entity(e1.get_id)
        assert(e1.get_containing_groups_ids.size == 1)
        assert(e1.get_count_of_containing_groups == 1)
        e2.get.add_relation_to_group(mRelationTypeId, g1.get_id, None)
        assert(e1.get_containing_relations_to_group().size == 1)
        assert(e1.get_containing_relation_to_group_descriptions().size == 0)
        e2.get.unarchive()
        assert(e1.get_containing_relation_to_group_descriptions().size == 1)
      }
    */

    fn test_get_count_of_containing_local_entities_etc() {
        Util::initialize_tracing();
        let db: Rc<RefCell<PostgreSQLDatabase> >= Rc::new(RefCell::new(Util::initialize_test_db().unwrap()));
        let e1 = Entity::create_entity(db.clone(), None, "e1", None, None).unwrap();
        let rel_type_id = db.borrow()
            .find_relation_type(None, Util::THE_HAS_RELATION_TYPE_NAME)
            .unwrap();
        let (e2_id, rte_id) = db.borrow()
            .create_entity_and_relation_to_local_entity(
                None,
                e1.get_id(),
                rel_type_id,
                "e2",
                None,
                None,
                Utc::now().timestamp_millis(),
            )
            .unwrap();
        let e2 = Entity::new2(db.clone(), None, e2_id).unwrap();
        let g1_id = db.borrow().create_group(None, "g1", false).unwrap();
        let g1 = Group::new2(db.clone(), None, g1_id).unwrap();
        //Using None instead of tx here for simplicity, but might have to change if
        //running tests in parallel.
        //let tx = db.begin_trans().unwrap();
        //let tx = Some(Rc::new(RefCell::new(tx)));
        let tx = None;

        // e2 should be contained in one entity (e1)
        let (count, _) = e2
            .get_count_of_containing_local_entities(tx.clone())
            .unwrap();
        assert_eq!(count, 1);
        let entities = e2
            .get_local_entities_containing_entity(tx.clone(), 0, None)
            .unwrap();
        assert_eq!(entities.len(), 1);
        // Create e3 with relation from e1
        db.borrow().create_entity_and_relation_to_local_entity(
            tx.clone(),
            e1.get_id(),
            rel_type_id,
            "e3",
            None,
            None,
            Utc::now().timestamp_millis(),
        )
        .unwrap();
        // Test adjacent attributes sorting indexes
        let indexes: Vec<Vec<Option<DataType>>> = e1
            .get_adjacent_attributes_sorting_indexes(tx.clone(), i64::MIN, None, true)
            .unwrap();
        assert!(!indexes.is_empty());
        let nearest_index = e1
            .get_nearest_attribute_entrys_sorting_index(tx.clone(), i64::MIN, true)
            .unwrap();
        assert!(nearest_index.unwrap() > i64::MIN);
        // Test renumbering
        e1.renumber_sorting_indexes(tx.clone()).unwrap();
        let nearest_index2 = e1
            .get_nearest_attribute_entrys_sorting_index(tx.clone(), i64::MIN, true)
            .unwrap();
        assert!(nearest_index2.unwrap() > nearest_index.unwrap());

        // Test attribute sorting index operations
        let rte = RelationToLocalEntity::new3(db.clone(), tx.clone(), rte_id)
            .unwrap()
            .unwrap(); //, rel_type_id, e1.get_id(), e2_id).unwrap();
        let form_id = rte.get_form_id().unwrap();
        assert!(!e1
            .is_attribute_sorting_index_in_use(tx.clone(), i64::MAX)
            .unwrap());
        e1.update_attribute_sorting_index(tx.clone(), form_id.into(), rte_id, i64::MAX)
            .unwrap();
        assert_eq!(
            e1.get_attribute_sorting_index(tx.clone(), form_id.into(), rte_id)
                .unwrap(),
            i64::MAX
        );
        assert!(e1
            .is_attribute_sorting_index_in_use(tx.clone(), i64::MAX)
            .unwrap());
        let unused_index = e1
            .find_unused_attribute_sorting_index(tx.clone(), None)
            .unwrap();
        assert_ne!(unused_index, i64::MAX);

        // Test relation to entity counts
        assert_eq!(
            e1.get_relation_to_local_entity_count(tx.clone(), true)
                .unwrap(),
            2
        );
        // Test entity archiving and counting
        {
            //let mut e2 = Entity::new2(db.clone(), tx.clone(), e2_id).unwrap();
            //e2.archive(tx.clone()).unwrap();
            let mut e2 = Entity::new2(db.clone(), None, e2_id).unwrap();
            e2.archive(None).unwrap();
        }
        assert_eq!(
            e1.get_relation_to_local_entity_count(tx.clone(), false)
                .unwrap(),
            1
        );
        assert_eq!(
            e1.get_relation_to_local_entity_count(tx.clone(), true)
                .unwrap(),
            2
        );

        // Test text attribute operations
        let text_attr_results = e1
            .get_text_attribute_by_type_id(tx.clone(), rel_type_id, None)
            .unwrap();
        assert_eq!(text_attr_results.len(), 0);
        let text = "test text for relation type";
        e1.add_text_attribute(tx.clone(), rel_type_id, text, None)
            .unwrap();
        let text_attr_results2 = e1
            .get_text_attribute_by_type_id(tx.clone(), rel_type_id, None)
            .unwrap();
        assert_eq!(text_attr_results2.len(), 1);

        // Test entity name update
        let mut e1 = Entity::new2(db.clone(), None, e1.get_id()).unwrap();
        assert_ne!(e1.get_name(tx.clone()).unwrap(), "updated");
        e1.update_name(tx.clone(), "updated").unwrap();
        let mut updated_e1 = Entity::new2(db.clone(), tx.clone(), e1.get_id()).unwrap();
        assert_eq!(updated_e1.get_name(tx.clone()).unwrap(), "updated");
        // Test name duplication check
        assert!(Entity::is_duplicate(db.clone(), tx.clone(), "updated", None).unwrap());
        assert!(!Entity::is_duplicate(db.clone(), tx.clone(), "xyzNOTANAMEupdated", None).unwrap());

        // Test group containment operations
        g1.add_entity(tx.clone(), e1.get_id(), None).unwrap();
        let containing_groups = e1.get_containing_groups_ids(tx.clone()).unwrap();
        assert_eq!(containing_groups.len(), 1);
        assert_eq!(e1.get_count_of_containing_groups(tx.clone()).unwrap(), 1);

        let e2 = e2.clone();
        //e2.add_relation_to_group(tx.clone(), rel_type_id, g1_id, None).unwrap();
        e2.add_relation_to_group(None, rel_type_id, g1_id, None)
            .unwrap();
        let rtgs = e1
            .get_containing_relations_to_group(tx.clone(), 0, None)
            .unwrap();
        assert_eq!(rtgs.len(), 1);

        let descriptions = e1
            .get_containing_relation_to_group_descriptions(tx.clone(), None)
            .unwrap();
        assert!(descriptions.is_empty());
        let mut e2 = e2.clone();
        //e2.unarchive(tx.clone()).unwrap();
        e2.unarchive(None).unwrap();
        let descriptions = e1
            .get_containing_relation_to_group_descriptions(tx.clone(), None)
            .unwrap();
        assert!(descriptions.len() == 1);
    }
}
