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
use crate::model::relation_to_group::RelationToGroup;
use crate::model::relation_to_local_entity::RelationToLocalEntity;
use crate::model::relation_to_remote_entity::RelationToRemoteEntity;
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

#[derive(Clone)]
pub struct Entity<'a> {
    db: &'a dyn Database,
    id: i64,
    already_read_data: bool,        /*= false*/
    name: String,                   /*= _*/
    class_id: Option<i64>,          /*= None*/
    insertion_date: i64,            /*= -1*/
    public: Option<bool>,           /*= None*/
    archived: bool,                 /*= false*/
    new_entries_stick_to_top: bool, /*= false*/
}

impl Entity<'_> {
    const PRIVACY_PUBLIC: &'static str = "[PUBLIC]";
    const PRIVACY_NON_PUBLIC: &'static str = "[NON-PUBLIC]";
    const PRIVACY_UNSET: &'static str = "[UNSET]";

    /// This one is perhaps only called by the database class implementation--so it can return arrays of objects & save more DB hits
    /// that would have to occur if it only returned arrays of keys. This DOES NOT create a persistent object--but rather should reflect
    /// one that already exists.
    pub fn new<'a>(
        db: &'a dyn Database,
        id: i64,
        name: String,
        class_id: Option<i64>, /*= None*/
        insertion_date: i64,
        public: Option<bool>,
        archived: bool,
        new_entries_stick_to_top: bool,
    ) -> Entity<'a> {
        Entity {
            id,
            db,
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
    /// Allows create_entity to return an instance without duplicating the database check that it Entity(long, Database) does.
    /// This constructor instantiates an existing object from the DB. Generally use Model.createObject() to create a new object.
    /// Note: Having Entities and other DB objects be readonly makes the code clearer & avoid some bugs, similarly to reasons for immutability in scala.
    /// (At least that has been the idea. But that might change as I just discovered a case where that causes a bug and it seems cleaner to have a
    /// set... method to fix it.)
    // Idea: replace this w/ a mock? where used? same, for similar code elsewhere like in OmInstance? (and
    // EntityTest etc could be with mocks instead of real db use.)  Does this really skip that other check though?
    pub fn new2<'a>(
        db: &'a dyn Database,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id: i64,
    ) -> Result<Entity<'a>, anyhow::Error> {
        // (See comment in similar spot in BooleanAttribute for why not checking for exists, if db.is_remote.)
        if !db.is_remote() && !db.entity_key_exists(transaction, id, true)? {
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

    pub fn create_entity<'a>(
        db: &'a dyn Database,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        in_name: &'a str,
        in_class_id: Option<i64>,   /*= None*/
        is_public_in: Option<bool>, /*= None*/
    ) -> Result<Entity<'a>, anyhow::Error> {
        let id: i64 = db.create_entity(transaction.clone(), in_name, in_class_id, is_public_in)?;
        Entity::new2(db as &dyn Database, transaction.clone(), id)
    }

    fn name_length() -> u32 {
        Util::entity_name_length()
    }

    fn is_duplicate(
        db_in: &dyn Database,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        in_name: &str,
        in_self_id_to_ignore: Option<i64>, /*= None*/
    ) -> Result<bool, anyhow::Error> {
        db_in.is_duplicate_entity_name(transaction, in_name, in_self_id_to_ignore)
    }

    /// This is for times when you want None if it doesn't exist, instead of the Error returned by
    /// the Entity constructor.  Or for convenience in tests.
    fn get_entity<'a>(
        db_in: &'a dyn Database,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id: i64,
    ) -> Result<Option<Entity<'a>>, String> {
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
                // let template_entity_id: Option<i64> = self.db.get_class_data(transaction, class_id.unwrap()).get(1).asInstanceOf[Option<i64>];
                let row = self.db.get_class_data(transaction.clone(), id)?;
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

    // fn get_archived_status(
    //     &mut self,
    // transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    // ) -> Result<bool, anyhow::Error> {
    //     if !self.already_read_data {
    //         self.read_data_from_db(transaction)?;
    //     }
    //     Ok(archived)
    //   }

    fn is_archived(
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
            if self.db.include_archived_entities() {
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
        let entity_data = self.db.get_entity_data(transaction, self.id)?;
        if entity_data.len() == 0 {
            return Err(anyhow!(
                "No results returned from data request for: {}",
                self.id
            ));
        }
        //idea: surely there is some better way than what I am doing here? See other places similarly.

        // DataType::String(self.name) = entity_data[0];
        self.name = match &entity_data[0] {
            Some(DataType::String(x)) => x.clone(),
            _ => return Err(anyhow!("How did we get here for {:?}?", entity_data[0])),
        };

        //%%%FIXME TO USE: entity_data[1]; RELY ON TESTS that I find or uncomment in order, to
        //see what will happen when a null is returned from get_entity_data above, and its dependencies
        // that eventually call postgresql_databaseN.db_query and see how they all handle a NULL coming back from pg, therefore
        // how to handle that when it gets here.  AND SIMILARLY/SAME do for the fixme just below!
        // DataType::Bigint(self.m_class_id) = None;
        self.class_id = None;
        // self.m_class_id = match entity_data[1] {
        //     DataType::Bigint(x) => x,
        //     _ => return Err(anyhow!(("How did we get here for {:?}?", entity_data[1])),
        // };

        self.public = None; //%%%7FIXME TO USE:entity_data[3].asInstanceOf[Option<bool>]
                            // self.m_public = match entity_data[3] {
                            //     DataType::Boolean(x) => x,
                            //     _ => return Err(anyhow!("How did we get here for {:?}?", entity_data[3])),
                            // };

        // DataType::Bigint(self.insertion_date) = entity_data[2];
        self.insertion_date = match entity_data[2] {
            Some(DataType::Bigint(x)) => x,
            _ => return Err(anyhow!("How did we get here for {:?}?", entity_data[2])),
        };
        // DataType::Boolean(self.m_archived) = entity_data[4];
        self.archived = match entity_data[4] {
            Some(DataType::Boolean(x)) => x,
            _ => return Err(anyhow!("How did we get here for {:?}?", entity_data[4])),
        };
        // DataType::Boolean(self.new_entries_stick_to_top) = entity_data[5];
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
    /// Intended as a temporarily unique string to distinguish an entity, across OM Instances.  NOT intended as a permanent unique ID (since
    /// the remote address for a given OM instance can change! and the local address is displayed as blank!), see get_unique_identifier
    /// for that.  This one is like that other in a way, but more for human consumption (eg data export for human reading, not for re-import -- ?).
    fn get_readable_identifier(&self) -> String {
        let remote_prefix = match self.db.get_remote_address() {
            None => "".to_string(),
            Some(s) => format!("{}_", s),
        };
        format!("{}{}", remote_prefix, self.get_id().to_string())
    }

    /// Intended as a unique string to distinguish an entity, even across OM Instances.  Compare to getHumanIdentifier (get_readable_identifier?)
    /// Idea: would any (future?) use cases be better served by including *both* the human-readable address (as in
    /// getHumanIdentifier) and the instance id? Or, just combine the methods into one?
    fn get_unique_identifier(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<String, anyhow::Error> {
        Ok(format!("{}_{}", self.db.id(transaction)?, self.get_id()))
    }

    fn get_attribute_count(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        include_archived_entities_in: bool, /*= db.include_archived_entities*/
    ) -> Result<u64, anyhow::Error> {
        self.db
            .get_attribute_count(transaction, self.get_id(), include_archived_entities_in)
    }

    fn get_relation_to_group_count(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<u64, anyhow::Error> {
        self.db
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
            .db
            .get_class_count(transaction.clone(), Some(self.get_id()))?;
        let definer_info = if count > 0 {
            "template (defining entity) for "
        } else {
            ""
        };
        let class_name: Option<String> = match self.get_class_id(transaction.clone())? {
            Some(class_id) => self.db.get_class_name(transaction.clone(), class_id)?,
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
        // This was the old way in scala.  Delete comments and this fn, and just use _helper()? Or,
        // why was this needed, or was it really?:
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
    fn add_quantity_attribute<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        in_attr_type_id: i64,
        in_unit_id: i64,
        in_number: f64,
        sorting_index_in: Option<i64>,
    ) -> Result<QuantityAttribute<'a>, anyhow::Error> {
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

    /// Creates a quantity attribute on this Entity (i.e., "6 inches length"), with default values of "now" for the dates. See "add_quantity_attribute" comment
    /// in db implementation file,
    /// for explanation of the parameters. It might also be nice to add the recorder's ID (person or app), but we'd have to do some kind
    /// of authentication/login 1st? And a GUID for users (as Entities?)?
    /// See PostgreSQLDatabase.create_quantity_attribute(...) for details.
    fn add_quantity_attribute2<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        in_attr_type_id: i64,
        in_unit_id: i64,
        in_number: f64,
        sorting_index_in: Option<i64>, /*= None*/
        in_valid_on_date: Option<i64>,
        observation_date_in: i64,
    ) -> Result<QuantityAttribute<'a>, anyhow::Error> {
        // write it to the database table--w/ a record for all these attributes plus a key indicating which Entity
        // it all goes with
        let id = self.db.create_quantity_attribute(
            transaction.clone(),
            self.id,
            in_attr_type_id,
            in_unit_id,
            in_number,
            in_valid_on_date,
            observation_date_in,
            sorting_index_in,
        )?;
        QuantityAttribute::new2(self.db, transaction.clone(), id)
    }

    fn get_quantity_attribute<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        in_key: i64,
    ) -> Result<QuantityAttribute<'a>, anyhow::Error> {
        QuantityAttribute::new2(self.db, transaction, in_key)
    }

    fn get_text_attribute<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        in_key: i64,
    ) -> Result<TextAttribute<'a>, anyhow::Error> {
        TextAttribute::new2(self.db, transaction, in_key)
    }

    fn get_date_attribute<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        in_key: i64,
    ) -> Result<DateAttribute<'a>, anyhow::Error> {
        DateAttribute::new2(self.db, transaction, in_key)
    }

    fn get_boolean_attribute<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        in_key: i64,
    ) -> Result<BooleanAttribute<'a>, anyhow::Error> {
        BooleanAttribute::new2(self.db, transaction, in_key)
    }

    fn get_file_attribute<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        in_key: i64,
    ) -> Result<FileAttribute<'a>, anyhow::Error> {
        FileAttribute::new2(self.db, transaction, in_key)
    }

    fn get_count_of_containing_groups(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<u64, anyhow::Error> {
        self.db
            .get_count_of_groups_containing_entity(transaction, self.get_id())
    }

    fn get_containing_groups_ids(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<Vec<i64>, anyhow::Error> {
        self.db
            .get_containing_groups_ids(transaction, self.get_id())
    }

    fn get_containing_relations_to_group(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        starting_index_in: i64,   /*= 0*/
        max_vals_in: Option<i64>, /*= None*/
    ) -> Result<Vec<RelationToGroup>, anyhow::Error> {
        self.db.get_containing_relations_to_group(
            transaction,
            self.get_id(),
            starting_index_in,
            max_vals_in,
        )
    }

    fn get_containing_relation_to_group_descriptions<'a>(
        &self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        limit_in: Option<i64>, /*= None*/
    ) -> Result<Vec<String>, anyhow::Error> {
        self.db
            .get_containing_relation_to_group_descriptions(transaction, self.get_id(), limit_in)
    }

    fn find_relation_to_and_group(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<(Option<i64>, Option<i64>, Option<i64>, Option<String>, bool), anyhow::Error> {
        self.db
            .find_relation_to_and_group_on_entity(transaction, self.get_id(), None)
    }

    fn find_contained_local_entity_ids<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        results_in_out: &'a mut HashSet<i64>,
        search_string_in: &str,
        levels_remaining_in: i32,      /*= 20*/
        stop_after_any_found_in: bool, /*= true*/
    ) -> Result<&mut HashSet<i64>, anyhow::Error> {
        self.db.find_contained_local_entity_ids(
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
        self.db
            .get_count_of_local_entities_containing_local_entity(transaction, self.get_id())
    }

    fn get_local_entities_containing_entity(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        starting_index_in: i64,   /*= 0*/
        max_vals_in: Option<i64>, /*= None*/
    ) -> Result<Vec<(i64, Entity)>, anyhow::Error> {
        self.db.get_local_entities_containing_local_entity(
            transaction,
            self.get_id(),
            starting_index_in,
            max_vals_in,
        )
    }

    fn get_adjacent_attributes_sorting_indexes(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        sorting_index_in: i64,
        limit_in: Option<i64>,     /*= None*/
        forward_not_back_in: bool, /*= true*/
    ) -> Result<Vec<Vec<Option<DataType>>>, anyhow::Error> {
        self.db.get_adjacent_attributes_sorting_indexes(
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
        self.db.get_nearest_attribute_entrys_sorting_index(
            transaction,
            self.get_id(),
            starting_point_sorting_index_in,
            forward_not_back_in,
        )
    }

    fn renumber_sorting_indexes<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
    ) -> Result<(), anyhow::Error> {
        self.db
            .renumber_sorting_indexes(transaction, self.get_id(), true)
    }

    fn update_attribute_sorting_index(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        attribute_form_id_in: i64,
        attribute_id_in: i64,
        sorting_index_in: i64,
    ) -> Result<u64, anyhow::Error> {
        self.db.update_attribute_sorting_index(
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
        self.db.get_entity_attribute_sorting_index(
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
        self.db
            .is_attribute_sorting_index_in_use(transaction, self.get_id(), sorting_index_in)
    }

    fn find_unused_attribute_sorting_index(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        starting_with_in: Option<i64>, /*= None*/
    ) -> Result<i64, anyhow::Error> {
        self.db
            .find_unused_attribute_sorting_index(transaction, self.get_id(), starting_with_in)
    }

    fn get_relation_to_local_entity_count(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        include_archived_entities_in: bool, /*= true*/
    ) -> Result<u64, anyhow::Error> {
        self.db.get_relation_to_local_entity_count(
            transaction,
            self.get_id(),
            include_archived_entities_in,
        )
    }

    fn get_relation_to_remote_entity_count(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<u64, anyhow::Error> {
        self.db
            .get_relation_to_remote_entity_count(transaction, self.get_id())
    }

    fn get_text_attribute_by_type_id(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        type_id_in: i64,
        expected_rows_in: Option<usize>, /*= None*/
    ) -> Result<Vec<TextAttribute>, anyhow::Error> {
        self.db.get_text_attribute_by_type_id(
            transaction,
            self.get_id(),
            type_id_in,
            expected_rows_in,
        )
    }

    // Depending on future callers, should this return instead an Entity and RTLE,
    // creating them here?
    /// @return the new entity_id and relation_to_local_entity_id that relates to it.
    fn add_uri_entity_with_uri_attribute<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        new_entity_name_in: &str,
        uri_in: &str,
        observation_date_in: i64,
        make_them_public_in: Option<bool>,
        quote_in: Option<&str>, /*= None*/
    ) -> Result<(i64, i64), anyhow::Error> {
        self.db.add_uri_entity_with_uri_attribute(
            transaction,
            self.get_id(),
            new_entity_name_in,
            uri_in,
            observation_date_in,
            make_them_public_in,
            quote_in,
        )
    }

    /*%%%%%
     //%%why do we have both add..() (just below) and create..() here?
      fn create_text_attribute(attr_type_id_in: i64, text_in: String, valid_on_date_in: Option<i64> /*= None*/,
                            observation_date_in: i64 = Utc::now().timestamp_millis(), caller_manages_transactions_in: bool /*= false*/,
                            sorting_index_in: Option<i64> /*= None*/) -> /*id*/ i64 {
      db.create_text_attribute(get_id, attr_type_id_in, text_in, valid_on_date_in, observation_date_in, caller_manages_transactions_in, sorting_index_in)
    }

      fn updateContainedEntitiesPublicStatus(newValueIn: Option<bool>) -> Int {
      let (attrTuples: Array[(i64, Attribute)], _) = get_sorted_attributes(0, 0, only_public_entities_in = false);
      let mut count = 0;
      for (attr <- attrTuples) {
        attr._2 match {
          case attribute: RelationToEntity =>
            // Using RelationToEntity here because it actually makes sense. But usually it is best to make sure to use either RelationToLocalEntity
            // or RelationToRemoteEntity, to be clearer about the logic.
            require(attribute.get_related_id1 == get_id, "Unexpected value: " + attribute.get_related_id1)
            let e: Entity = new Entity(Database.currentOrRemoteDb(attribute, db), attribute.get_related_id2);
            e.updatePublicStatus(newValueIn)
            count += 1
          case attribute: RelationToGroup =>
            let group_id: i64 = attribute.get_group_id;
            let entries: Vec<Vec<Option<DataType>>> = db.get_group_entries_data(group_id, None, include_archived_entities_in = false);
            for (entry <- entries) {
              let entity_id = entry(0).get.asInstanceOf[i64];
              db.update_entity_only_public_status(entity_id, newValueIn)
              count += 1
            }
          case _ =>
          // do nothing
        }
      }
      count
    }
    %%  */

    /// See add_quantity_attribute(...) methods for comments.
    fn add_text_attribute<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        in_attr_type_id: i64,
        in_text: &str,
        sorting_index_in: Option<i64>,
    ) -> Result<TextAttribute<'a>, anyhow::Error> {
        self.add_text_attribute2(
            transaction.clone(),
            in_attr_type_id,
            in_text,
            sorting_index_in,
            None,
            Utc::now().timestamp_millis(),
        )
    }

    pub fn add_text_attribute2<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        in_attr_type_id: i64,
        in_text: &str,
        sorting_index_in: Option<i64>,
        in_valid_on_date: Option<i64>,
        observation_date_in: i64,
    ) -> Result<TextAttribute<'a>, anyhow::Error> {
        let id = self.db.create_text_attribute(
            transaction.clone(),
            self.id,
            in_attr_type_id,
            in_text,
            in_valid_on_date,
            observation_date_in,
            sorting_index_in,
        )?;
        TextAttribute::new2(self.db, transaction, id)
    }

    fn add_date_attribute<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        in_attr_type_id: i64,
        in_date: i64,
        sorting_index_in: Option<i64>, /*= None*/
    ) -> Result<DateAttribute<'a>, anyhow::Error> {
        let id = self.db.create_date_attribute(
            transaction.clone(),
            self.id,
            in_attr_type_id,
            in_date,
            sorting_index_in,
        )?;
        DateAttribute::new2(self.db, transaction, id)
    }

    fn add_boolean_attribute<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        in_attr_type_id: i64,
        in_boolean: bool,
        sorting_index_in: Option<i64>,
    ) -> Result<BooleanAttribute<'a>, anyhow::Error> {
        self.add_boolean_attribute2(
            transaction,
            in_attr_type_id,
            in_boolean,
            sorting_index_in,
            None,
            Utc::now().timestamp_millis(),
        )
    }

    fn add_boolean_attribute2<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        in_attr_type_id: i64,
        in_boolean: bool,
        sorting_index_in: Option<i64>, /*= None*/
        in_valid_on_date: Option<i64>,
        observation_date_in: i64,
    ) -> Result<BooleanAttribute<'a>, anyhow::Error> {
        let id = self.db.create_boolean_attribute(
            transaction.clone(),
            self.id,
            in_attr_type_id,
            in_boolean,
            in_valid_on_date,
            observation_date_in,
            sorting_index_in,
        )?;
        BooleanAttribute::new2(self.db, transaction, id)
    }

    /*%%
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
    fn add_relation_to_local_entity<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        in_attr_type_id: i64,
        in_entity_id2: i64,
        sorting_index_in: Option<i64>,
        in_valid_on_date: Option<i64>, /*= None*/
        in_observation_date: i64,      /*= Utc::now().timestamp_millis()*/
    ) -> Result<RelationToLocalEntity, anyhow::Error> {
        let (rte_id, new_sorting_index) = self.db.create_relation_to_local_entity(
            transaction.clone(),
            in_attr_type_id,
            self.get_id(),
            in_entity_id2,
            in_valid_on_date,
            in_observation_date,
            sorting_index_in,
        )?;
        Ok(RelationToLocalEntity::new(
            self.db,
            rte_id,
            in_attr_type_id,
            self.get_id(),
            in_entity_id2,
            in_valid_on_date,
            in_observation_date,
            new_sorting_index,
        ))
    }

    /*%%put back after converting RelationToRemoteEntity
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
    fn create_group_and_add_a_has_relation_to_it<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        new_group_name_in: &str,
        mixed_classes_allowed_in: bool,
        observation_date_in: i64,
    ) -> Result<(i64, i64), anyhow::Error> {
        // the "has" relation type that we want should always be the 1st one, since it is created by in the initial app startup; otherwise it seems we can use it
        // anyway:
        let relation_type_id = self
            .db
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
    //%%%%%%?
    pub fn add_group_and_relation_to_group<'a, 'b>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'b, Postgres>>>>,
        rel_type_id_in: i64,
        new_group_name_in: &str,
        allow_mixed_classes_in_group_in: bool, /*= false*/
        valid_on_date_in: Option<i64>,
        observation_date_in: i64,
        sorting_index_in: Option<i64>,
    ) -> Result<(i64, i64), anyhow::Error> 
    where
        'a: 'b,
    {
        let (group_id, rtg_id) = self.db.create_group_and_relation_to_group(
            transaction.clone(),
            self.get_id(),
            rel_type_id_in,
            new_group_name_in,
            allow_mixed_classes_in_group_in,
            valid_on_date_in,
            observation_date_in,
            sorting_index_in,
        )?;
        let group: Group = Group::new2(self.db, transaction.clone(), group_id)?;
        let rtg = RelationToGroup::new2(
            self.db,
            transaction.clone(),
            rtg_id,
            self.get_id(),
            rel_type_id_in,
            group_id,
        )?;
        Ok((group.get_id(), rtg.get_id()))
    }

    /// @return the id of the new RTE
    fn add_has_relation_to_local_entity<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        entity_id_in: i64,
        valid_on_date_in: Option<i64>,
        observation_date_in: i64,
    ) -> Result<RelationToLocalEntity<'a>, anyhow::Error> {
        let (rel_id, has_rel_type_id, new_sorting_index) =
            self.db.add_has_relation_to_local_entity(
                transaction,
                self.get_id(),
                entity_id_in,
                valid_on_date_in,
                observation_date_in,
                None,
            )?;
        let rtle: RelationToLocalEntity = RelationToLocalEntity::new(
            self.db,
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
    pub fn create_entity_and_add_has_local_relation_to_it<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        new_entity_name_in: &str,
        observation_date_in: i64,
        is_public_in: Option<bool>,
    ) -> Result<(Entity<'a>, RelationToLocalEntity<'a>), anyhow::Error> {
        let (new_entity_id, rte_id, relation_type_id) =
            self.db.create_entity_and_add_has_local_relation_to_it(
                transaction.clone(),
                self.get_id(),
                new_entity_name_in,
                observation_date_in,
                is_public_in,
            )?;
        //idea: would be faster (no db hit) if we called Entity::new here instead, with the data
        //as needed returned from the fn call just above, instead of Entity::new2. Might have
        //to add some return values to Entity::new.
        let new_entity: Entity = Entity::new2(self.db, transaction.clone(), new_entity_id)?;
        //idea: ditto the comment just above.
        let rte: RelationToLocalEntity = RelationToLocalEntity::new2(
            self.db,
            transaction.clone(),
            rte_id,
            relation_type_id,
            self.get_id(),
            new_entity_id,
        )?;
        Ok((new_entity, rte))
    }

    fn add_entity_and_relation_to_local_entity<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        rel_type_id_in: i64,
        new_entity_name_in: &str,
        valid_on_date_in: Option<i64>,
        observation_date_in: i64,
        is_public_in: Option<bool>,
    ) -> Result<(Entity<'a>, RelationToLocalEntity<'a>), anyhow::Error> {
        let (new_entity_id, new_rte_id) = self.db.create_entity_and_relation_to_local_entity(
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
        let entity = Entity::new2(self.db, transaction.clone(), new_entity_id)?;
        let rte = RelationToLocalEntity::new2(
            self.db,
            transaction,
            new_rte_id,
            rel_type_id_in,
            self.get_id(),
            new_entity_id,
        )?;
        Ok((entity, rte))
    }

    /*
          /**
            * @return the new group's id.
            */
            fn addRelationToGroup(rel_type_id_in: i64, group_id_in: i64, sorting_index_in: Option<i64>) -> RelationToGroup {
            addRelationToGroup(rel_type_id_in, group_id_in, sorting_index_in, None, Utc::now().timestamp_millis())
          }

            fn addRelationToGroup(rel_type_id_in: i64, group_id_in: i64, sorting_index_in: Option<i64>,
                                 valid_on_date_in: Option<i64>, observation_date_in: i64) -> RelationToGroup {
            let (new_rtg_id, sorting_index) = db.create_relation_to_group(get_id, rel_type_id_in, group_id_in, valid_on_date_in, observation_date_in, sorting_index_in);
            new RelationToGroup(db, new_rtg_id, get_id, rel_type_id_in, group_id_in, valid_on_date_in, observation_date_in, sorting_index)
          }

            fn get_sorted_attributes(starting_object_index_in: Int = 0, max_vals_in: Int = 0, only_public_entities_in: bool = true) -> (Array[(i64, Attribute)], Int) {
            db.get_sorted_attributes(get_id, starting_object_index_in, max_vals_in, only_public_entities_in = only_public_entities_in)
          }

            fn updateClass(class_id_in: Option<i64>) /*%% -> Unit*/ {
            if !already_read_data) read_data_from_db()
            if class_id_in != m_class_id) {
              db.update_entitys_class(this.get_id, class_id_in)
              m_class_id = class_id_in
            }
          }

            fn updateNewEntriesStickToTop(b: bool) {
            if !already_read_data) read_data_from_db()
            if b != new_entries_stick_to_top) {
              db.update_entity_only_new_entries_stick_to_top(get_id, b)
              new_entries_stick_to_top = b
            }
          }

            fn updatePublicStatus(newValueIn: Option<bool>) {
            if !already_read_data) read_data_from_db()
            if newValueIn != m_public) {
              // The condition for this (when it was part of EntityMenu) used to include " && !entity_in.isInstanceOf[RelationType]", but maybe it's better w/o that.
              db.update_entity_only_public_status(get_id, newValueIn)
              m_public = newValueIn
            }
          }

            fn updateName(name_in: String) /*%% -> Unit*/ {
            if !already_read_data) read_data_from_db()
            if name_in != name) {
              db.update_entity_only_name(get_id, name_in);
              name = name_in
            }
          }

            fn archive() {
            db.archive_entity(id);
            m_archived = true
          }

            fn unarchive() {
            db.unarchive_entity(id);
            m_archived = false
          }

          /** Removes this object from the system. */
            fn delete() {
              db.delete_entity(id)
          }

    %%*/
}

#[cfg(test)]
mod test {
    use super::Entity;
    use crate::model::boolean_attribute::BooleanAttribute;
    //use crate::model::database::{DataType, Database};
    use crate::model::date_attribute::DateAttribute;
    use crate::model::file_attribute::FileAttribute;
    use crate::model::id_wrapper::IdWrapper;
    use crate::model::postgres::postgresql_database::PostgreSQLDatabase;
    use crate::model::relation_to_group::RelationToGroup;
    use crate::model::relation_to_local_entity::RelationToLocalEntity;
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

    /*%%latertests
          let mut mUnitId: i64 = 0;
          let mut quantity_attr_type_id: i64 = 0;
          let mut mTextAttrTypeId: i64 = 0;
          let mut mDateAttrTypeId = 0L;
          let mut m_booleanAttrTypeId = 0L;
          let mut mFileAttrTypeId = 0L;
          let mut mRelationTypeId = 0L;

          override fn runTests(testName: Option<String>, args: Args) -> Status {
            setUp()
            let result: Status = super.runTests(testName, args);
            // (not calling tearDown: see comment inside PostgreSQLDatabaseTest.runTests about "db setup/teardown")
            result
          }

          protected fn setUp() {
            //start fresh
            PostgreSQLDatabaseTest.tearDownTestDB()

            // instantiation does DB setup (creates tables, default data, etc):
            db = new PostgreSQLDatabase(Database::TEST_USER, Database::TEST_PASS)

            mUnitId = db.create_entity("centimeters")
            mTextAttrTypeId = db.create_entity("someName")
            mDateAttrTypeId = db.create_entity("someName")
            m_booleanAttrTypeId = db.create_entity("someName")
            mFileAttrTypeId = db.create_entity("someName")
            mRelationTypeId = db.create_relation_type("someRelationType", "reversedName", "NON")
            let id: i64 = db.create_entity("test object");
            mEntity = new Entity(db, id)
          }

          protected fn tearDown() {
            PostgreSQLDatabaseTest.tearDownTestDB()
          }
    */
    /* %%latertests
    #[test]
    fn test_add_quantity_attribute() {
        Util::initialize_tracing();
        let db: PostgreSQLDatabase = Util::initialize_test_db().unwrap();
        let tx = db.begin_trans().unwrap();
        let tx = Some(Rc::new(RefCell::new(tx)));
        //%%latertests Maybe move things like this into a setup method that does "once" like util.rs does, equiv to the above?
        //Or one that returns the various values? or just fill in & keep copy/pasting as needed?
        //%%latertests Or look up the once_cell crate -- would it help fill in vars w/ boxes? at what level? *singletons*?
        let quantity_attr_type_id = Entity::create_entity(&DB, tx, "length", None, None).unwrap();
        let e: Entity = Entity::create_entity(&DB, tx, "testEntityName", None, None).unwrap();
        let id: i64 = e.add_quantity_attribute(quantity_attr_type_id, mUnitId, 100, None).get_id();
        let qo: QuantityAttribute = mEntity.get_quantity_attribute(id);
        if qo == null {
          fail("add_quantity_attribute then get_quantity_attribute returned null")
        }
        assert(qo.get_id == id)
        db.rollback_trans()
      }

      "testAddTextAttribute" should "also work" in {
        db.begin_trans()
        println!("starting testAddTextAttribute")
        let id: i64 = mEntity.add_text_attribute(mTextAttrTypeId, "This is someName given to an object", None).get_id;
        let t: TextAttribute = mEntity.get_textAttribute(id);
        if t == null {
          fail("add_text_attribute then get_textAttribute returned null")
        }
        assert(t.get_id == id)
        db.rollback_trans()
      }

      "testAddDateAttribute" should "also work" in {
        db.begin_trans()
        println!("starting testAddDateAttribute")
        let id: i64 = mEntity.add_date_attribute(mDateAttrTypeId, 2).get_id;
        let t: DateAttribute = mEntity.get_date_Attribute(id);
        assert(t != null)
        assert(t.get_id == id)
        assert(t.get_attr_type_id() == mDateAttrTypeId)
        assert(t.get_date == 2)
        db.rollback_trans()
      }

      "testAddBooleanAttribute" should "also work" in {
        db.begin_trans()
        println!("starting testAddBooleanAttribute")
        let startTime = Utc::now().timestamp_millis();
        let id: i64 = mEntity.add_boolean_attribute(m_booleanAttrTypeId, in_boolean = true, None).get_id;
        let t: BooleanAttribute = mEntity.get_boolean_attribute(id);
        assert(t != null)
        assert(t.get_id == id)
        assert(t.get_boolean)
        assert(t.get_parent_id() == mEntity.get_id)
        assert(t.get_valid_on_date().isEmpty)
        assert(t.get_observation_date() > (startTime - 1) && t.get_observation_date() < (Utc::now().timestamp_millis() + 1))
        db.rollback_trans()
      }

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

      "get_display_string" should "return a useful stack trace string, when called with a nonexistent entity" in {
        // for example, if the entity has been deleted by one part of the code, or one user process in a console window (as an example), and is still
        // referenced and attempted to be displayed by another (or to be somewhat helpful if we try to get info on an entity that's gone due to a bug).
        // (But should this issue go away w/ better design involving more use of immutability or something?)
        let id = 0L;
        let mock_db = mock[PostgreSQLDatabase];
        when(mock_db.entity_key_exists(id)).thenReturn(true)
        when(mock_db.get_entity_data(id)).thenThrow(new RuntimeException("some exception"))
        when(mock_db.get_remote_address).thenReturn(None)
        let entity = new Entity(mock_db, id);
        let se = entity.get_display_string();
        assert(se.contains("Unable to get entity description due to"))
        assert(se.toLowerCase.contains("exception"))
        assert(se.toLowerCase.contains("at org.onemodel"))
      }

      "get_display_string" should "return name & class info" in {
        let id = 0L;
        let classId = 1L;
        let mock_db = mock[PostgreSQLDatabase];
        when(mock_db.entity_key_exists(id)).thenReturn(true)
        when(mock_db.get_class_name(classId)).thenReturn(Some("class1Name"))
        when(mock_db.get_entity_data(id)).thenReturn(Vec<Option<DataType>>(Some("entity1Name"), Some(classId)))
        // idea (is in tracked tasks): put next 3 lines back after color refactoring is done (& places w/ similar comment elsewhere)
        //val entity = new Entity(mock_db, id)
        //val ds = entity.get_display_string
        //assert(ds == "entity1Name (class: class1Name)")

        let id2 = 2L;
        let classId2 = 4L;
        let name2 = "entity2Name";
        let mock_db2 = mock[PostgreSQLDatabase];
        when(mock_db2.entity_key_exists(id2)).thenReturn(true)
        when(mock_db2.get_entity_data(id2)).thenReturn(Vec<Option<DataType>>(Some(name2), None))
        when(mock_db2.get_class_name(classId2)).thenReturn(None)
        // idea (is in tracked tasks): put next lines back after color refactoring is done (& places w/ similar comment elsewhere)
        //val entity2 = new Entity(mock_db2, id2, name2, Some(false), Some(classId2))
        //val ds2 = entity2.get_display_string
        //assert(ds2 == name2)

        when(mock_db2.get_class_name(classId2)).thenReturn(Some("class2Name"))
        when(mock_db2.get_class_count(Some(id2))).thenReturn(1)
        when(mock_db2.get_entity_data(id2)).thenReturn(Vec<Option<DataType>>(Some(name2), Some(classId2)))
        // idea (is in tracked tasks): put next line back after color refactoring is done (& places w/ similar comment elsewhere)
        //assert(entity2.get_display_string == name2 + " (template entity (template) for class: " + "class2Name)")
      }

      "get_class_template_entity_id" should "work right" in {
        let mock_db = mock[PostgreSQLDatabase];
        let id = 1L;
        let classId = 2L;
        let className = "classname";
        let template_entity_id = 3L;
        when(mock_db.entity_key_exists(id)).thenReturn(true)
        let e = new Entity(mock_db, id, "entityname", None, 0L, Some(true), false, false);
        assert(e.get_class_template_entity_id.isEmpty)

        let e2 = new Entity(mock_db, id, "entityname", Option(classId), 0L, Some(false), false, false);
        when(mock_db.class_key_exists(classId)).thenReturn(true)
        when(mock_db.get_class_data(classId)).thenReturn(Vec<Option<DataType>>(Some(className), Some(template_entity_id)))
        assert(e2.get_class_template_entity_id.get == template_entity_id)
      }

      "updateContainedEntitiesPublicStatus" should "work" in {
        let e1Id: i64 = db.create_entity("test object1");
        let e1 = new Entity(db, e1Id);
        mEntity.add_has_relation_to_local_entity(e1.get_id, Some(0), 0)
        let (group: Group, _/*rtg: RelationToGroup*/) = mEntity.add_group_and_relation_to_group(mRelationTypeId, "grpName",;
                                                                                        allow_mixed_classes_inGroupIn = true, Some(0), 0, None)
        let e2Id: i64 = db.create_entity("test object2");
        let e2 = new Entity(db, e1Id);
        group.add_entity(e2Id)

        assert(e1.get_public.isEmpty)
        assert(e2.get_public.isEmpty)
        mEntity.updateContainedEntitiesPublicStatus(Some(true))
        let e1reRead = new Entity(db, e1Id);
        let e2reRead = new Entity(db, e2Id);
        assert(e1reRead.get_public.get)
        assert(e2reRead.get_public.get)
      }

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

        let rte = RelationToLocalEntity.get_relation_to_local_entity(db, rteId).get;
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
        e2.get.addRelationToGroup(mRelationTypeId, g1.get_id, None)
        assert(e1.get_containing_relations_to_group().size == 1)
        assert(e1.get_containing_relation_to_group_descriptions().size == 0)
        e2.get.unarchive()
        assert(e1.get_containing_relation_to_group_descriptions().size == 1)
      }
    */
}
