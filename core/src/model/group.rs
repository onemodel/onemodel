/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2014-2017 inclusive, and 2023-2025 inclusive, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
use crate::color::Color;
use crate::model::database::{DataType, Database};
use crate::model::entity::Entity;
use crate::model::entity_class::EntityClass;
use crate::model::relation_to_group::RelationToGroup;
use crate::util::Util;
use anyhow::{anyhow, Error, Result};
use sqlx::{Postgres, Transaction};
use std::cell::RefCell;
use std::rc::Rc;
use tracing::*;

pub struct Group {
    id: i64,
    db: Rc<dyn Database>,
    already_read_data: bool,        /*= false*/
    name: String,                   /*= null*/
    insertion_date: i64,            /*= 0L*/
    mixed_classes_allowed: bool,    /*= false*/
    new_entries_stick_to_top: bool, /*= false*/
}

impl Group {
    /// Creates a new group in the database.
    fn create_group(
        db_in: Rc<dyn Database>,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        in_name: &str,
        allow_mixed_classes_in_group_in: bool, /*= false*/
    ) -> Result<Group, Error>
//where
    //    'a: 'b
    {
        let id: i64 = db_in.create_group(
            transaction.clone(),
            in_name,
            allow_mixed_classes_in_group_in,
        )?;
        // Might be obvious but: Calling fn new2, not new, here, because we don't have enough data to
        // call new and so it will load from the db the other values when needed, as saved by the above.
        Group::new2(db_in, transaction, id)
    }

    /// This is for times when you want None if it doesn't exist, instead of the exception thrown by the Entity constructor.  Or for convenience in tests.
    fn get_group(
        db_in: Rc<dyn Database>,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id: i64,
    ) -> Result<Option<Group>, Error> {
        let result: Result<Group, Error> = Group::new2(db_in, transaction, id);
        match result {
            Err(e) => {
                if e.to_string().contains(Util::DOES_NOT_EXIST) {
                    //idea: see comment here in Entity.rc.
                    Ok(None)
                } else {
                    Err(e)
                }
            }
            Ok(group) => Ok(Some(group)),
        }
    }

    /// See comment about these 2(?) dates in Database.create_tables() (?)
    /// [%%Confirm?:] This one is perhaps only called by the database class implementation--so it can return arrays of objects & save more DB hits
    /// that would have to occur if it only returned arrays of keys. This DOES NOT create a persistent object--but rather should reflect
    /// one that already exists.  It does not confirm that the id exists in the db.
    pub fn new(
        db: Rc<dyn Database>,
        id: i64,
        name_in: &str,
        insertion_date: i64,
        mixed_classes_allowed: bool,
        new_entries_stick_to_top: bool,
    ) -> Group {
        Group {
            db,
            id,
            name: name_in.to_string(),
            insertion_date,
            mixed_classes_allowed,
            new_entries_stick_to_top,
            already_read_data: true,
        }
    }

    /// See comments on similar methods in RelationToEntity (or maybe its subclasses%%).
    /// Groups don't contain remote entities (only those at the same DB as the group is), so some logic doesn't have to be written for that.
    pub fn new2(
        db: Rc<dyn Database>,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id: i64,
    ) -> Result<Group, Error>
//where
    //    'a: 'b,
    {
        // (See comment in similar spot in BooleanAttribute for why not checking for exists, if db.is_remote.)
        if !db.is_remote() && !db.group_key_exists(transaction, id)? {
            Err(anyhow!("Key {}{}", id, Util::DOES_NOT_EXIST))
        } else {
            Ok(Group {
                id,
                db,
                already_read_data: false,
                name: "".to_string(),
                insertion_date: 0,
                mixed_classes_allowed: false,
                new_entries_stick_to_top: false,
            })
        }
    }

    //%%eliminate the _ parameters? who calls it w/ them & why?
    pub fn update(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        _attr_type_id_in: Option<i64>,                 /*= None*/
        name_in: Option<String>,                       /*= None*/
        allow_mixed_classes_in_group_in: Option<bool>, /*= None*/
        new_entries_stick_to_top_in: Option<bool>,     /*= None*/
        _valid_on_date_in: Option<i64>,
        _observation_date_in: Option<i64>,
    ) -> Result<(), Error> {
        // write it to the database table--w/ a record for all these attributes plus a key indicating which Entity
        // it all goes with
        self.db.clone().update_group(
            transaction.clone(),
            self.id,
            match name_in {
                None => self.get_name(transaction.clone())?,
                Some(ref s) => {
                    self.name = s.clone();
                    s.clone()
                }
            },
            match allow_mixed_classes_in_group_in {
                None => self.get_mixed_classes_allowed(transaction.clone())?,
                Some(b) => {
                    self.mixed_classes_allowed = b;
                    b
                }
            },
            match new_entries_stick_to_top_in {
                None => self.get_new_entries_stick_to_top(transaction)?,
                Some(b) => {
                    self.new_entries_stick_to_top = b;
                    b
                }
            },
        )?;
        Ok(())
    }

    pub fn get_display_string(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        length_limit_in: usize, /*= 0*/
        simplify_in: bool,      /* = false*/
    ) -> Result<String, Error> {
        let num_entries = self
            .db
            .get_group_size(transaction.clone(), self.get_id(), 1)?;
        let mut result: String = "".to_string();
        let name = &self.get_name(transaction.clone())?;
        let formatted_name = format!("grp {} /{}: {}", self.id, num_entries, Color::blue(name));
        result.push_str(if simplify_in {
            name.as_str()
        } else {
            formatted_name.as_str()
        });
        if !simplify_in {
            result.push_str(", class: ");
            let class_name = {
                if self.get_mixed_classes_allowed(transaction.clone())? {
                    "(mixed)".to_string()
                } else {
                    let class_name_option = self.get_class_name(transaction.clone())?;
                    match class_name_option {
                        None => "None".to_string(),
                        Some(cn) => {
                            let n = cn.clone();
                            n
                        }
                    }
                }
            };
            result.push_str(class_name.as_str());
        }
        if simplify_in {
            Ok(result)
        } else {
            Ok(Util::limit_attribute_description_length(
                result.as_str(),
                length_limit_in,
            ))
        }
    }

    fn read_data_from_db(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<(), Error> {
        let data: Vec<Option<DataType>> = self.db.get_group_data(transaction, self.id)?;
        if data.len() == 0 {
            return Err(anyhow!(
                "No results returned from data request for: {}",
                self.id
            ));
        }

        self.already_read_data = true;

        self.name = match data[0].clone() {
            Some(DataType::String(x)) => x,
            _ => return Err(anyhow!("How did we get here for {:?}?", data[0])),
        };
        self.insertion_date = match data[1] {
            Some(DataType::Bigint(x)) => x,
            _ => return Err(anyhow!("How did we get here for {:?}?", data[1])),
        };
        self.mixed_classes_allowed = match data[2] {
            Some(DataType::Boolean(b)) => b,
            _ => return Err(anyhow!("How did we get here for {:?}?", data[2])),
        };
        self.new_entries_stick_to_top = match data[3] {
            Some(DataType::Boolean(b)) => b,
            _ => return Err(anyhow!("How did we get here for {:?}?", data[3])),
        };
        Ok(())
    }

    /// Removes this object from the system.
    pub fn delete<'a, 'b>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'b, Postgres>>>>,
    ) -> Result<(), Error>
    where
        'a: 'b,
    {
        self.db
            .delete_group_and_relations_to_it(transaction, self.id)
    }

    /// Removes an entity from this group.
    pub fn remove_entity<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        entity_id: i64,
    ) -> Result<u64, Error> {
        self.db
            .remove_entity_from_group(transaction, self.id, entity_id)
    }

    pub fn delete_with_entities<'a, 'b>(
        &'a self,
        // purpose: see comment in delete_objects
        transaction_in: Option<Rc<RefCell<Transaction<'b, Postgres>>>>,
    ) -> Result<(), Error>
    where
        'a: 'b,
    {
        self.db
            .delete_group_relations_to_it_and_its_entries(transaction_in.clone(), self.id)
    }

    // idea: cache this?  when doing any other query also?  Is that safer because we really don't edit these in place (ie, immutability)?
    pub fn get_size(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        include_which_entities: i32, /*= 3*/
    ) -> Result<u64, Error> {
        self.db
            .get_group_size(transaction, self.id, include_which_entities)
    }

    fn get_group_entries(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        starting_index_in: i64,
        max_vals_in: Option<i64>, /*= None*/
    ) -> Result<Vec<Group>, Error> {
        let ids = self.db.clone().get_group_entry_ids(
            transaction.clone(),
            self.id,
            starting_index_in,
            max_vals_in,
        )?;
        let mut results: Vec<Group> = Vec::new();
        for id in ids {
            let grp = Group::new2(self.db.clone(), transaction.clone(), id)?;
            results.push(grp);
        }
        Ok(results)
    }

    pub fn add_entity<'a, 'b>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'b, Postgres>>>>,
        in_entity_id: i64,
        sorting_index_in: Option<i64>, /*= None*/
    ) -> Result<(), Error>
    where
        'a: 'b,
    {
        let ref rc_db = &self.db;
        let ref cloned = rc_db.clone();
        let tx = transaction.clone();
        let id = self.get_id();
        cloned.add_entity_to_group(tx, id, in_entity_id, sorting_index_in)
    }

    pub fn get_id(&self) -> i64 {
        self.id
    }

    pub fn get_name(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<String, Error> {
        if !self.already_read_data {
            self.read_data_from_db(transaction)?
        }
        Ok(self.name.clone())
    }

    pub fn get_mixed_classes_allowed(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<bool, Error> {
        if !self.already_read_data {
            self.read_data_from_db(transaction)?
        }
        Ok(self.mixed_classes_allowed)
    }

    pub fn get_new_entries_stick_to_top(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<bool, Error> {
        if !self.already_read_data {
            self.read_data_from_db(transaction)?
        }
        Ok(self.new_entries_stick_to_top)
    }

    fn get_insertion_date(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<i64, Error> {
        if !self.already_read_data {
            self.read_data_from_db(transaction)?
        }
        Ok(self.insertion_date)
    }

    fn get_class_name(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<Option<String>, Error> {
        if self.get_mixed_classes_allowed(transaction.clone())? {
            Ok(None)
        } else {
            let class_id: Option<i64> = self.get_class_id(transaction.clone())?;
            match class_id {
                None => {
                    if self.get_size(transaction.clone(), 3)? == 0 {
                        // display should indicate that we know mixed are not allowed, so a class could be specified, but none has.
                        Ok(Some("(unspecified)".to_string()))
                    } else {
                        // means the group requires uniform classes, but the enforced uniform class is None, i.e., to not have a class:
                        Ok(Some("(specified as None)".to_string()))
                    }
                }
                Some(cid) => {
                    let mut example_entitys_class =
                        EntityClass::new2(self.db.clone(), transaction.clone(), cid)?;
                    Ok(Some(example_entitys_class.get_name(transaction)?))
                }
            }
        }
    }

    // idea: eliminate/simplify most of this part, since groups can't have subgroups only entities in them now?
    // fn find_an_entity(next_index: usize, entries: Vec<Entity>) -> Result<Option<Entity>, Error> {
    // We will have to change this (and probably other things) to traverse "subgroups" (groups in the entities in this group) also,
    // if we decide that disallowing mixed classes also means class uniformity across all subgroups.
    // if next_index >= entries.size {
    //   None
    // } else {

    // match entries.get(next_index) {
    //   case entity: Entity =>
    //     Some(entity)
    //   case _ =>
    //   let class_name = entries.get(next_index).getClass.get_name;
    //   throw new OmException(s"a group contained an entry that's not an entity?  Thought had eliminated use of 'subgroups' except via entities. It's " +
    //       s"of type: $class_name")
    // }
    // }
    // -or-
    //   match entries.get(next_index) {
    //     None => Ok(None),
    //     Some(e: Entity) => Ok(Some(e)),
    //     _ => return Err(anyhow!("unexpected result?: {:?}", entries.get(next_index)))
    //   }
    // }
    fn get_class_id(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<Option<i64>, Error> {
        if self.get_mixed_classes_allowed(transaction.clone())? {
            Ok(None)
        } else {
            let ids: Vec<i64> =
                self.db
                    .get_group_entry_ids(transaction.clone(), self.get_id(), 0, Some(1))?;
            let mut entries: Vec<Entity> = Vec::new();
            for id in ids {
                let entity: Entity = Entity::new2(self.db.clone(), transaction.clone(), id)?;
                entries.push(entity);
            }
            let specified: bool = entries.len() > 0;
            if !specified {
                Ok(None)
            } else {
                let next_index = 0;
                let next: Option<&mut Entity> = entries.get_mut(next_index);
                if next.is_none() {
                    return Ok(None);
                } else {
                    //let e: &mut Entity = next.unwrap();
                    let e: &mut Entity = next.unwrap();

                    let id: Option<i64> = e.get_class_id(transaction)?;
                    return Ok(id);
                }
                // /*let entity: Option<Entity> = */match next {
                //  Some(e) => match e.get_class_id(transaction)? {
                //      Some(id) => Ok(Some(id)),
                //      None => Ok(None),
                //  },
                //  None => Ok(None),
                //  //_ => return Err(anyhow!("unexpected result?: entity with id {}", entries.get(next_index).get_id()))
                // }
                //
                // match entity {
                //   Some(e) => e.get_class_id(transaction),
                //   _ => None,
                // }
            }
        }
    }

    //fn get_class_template_entity<'a, 'b>(
    //    &'a mut self,
    //    transaction: Option<Rc<RefCell<Transaction<'b, Postgres>>>>,
    fn get_class_template_entity(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<Option<Entity>, Error> {
        let class_id: Option<i64> = self.get_class_id(transaction.clone())?;
        match class_id {
            None if self.get_mixed_classes_allowed(transaction.clone())? => Ok(None),
            Some(id) => {
                let mut ec = EntityClass::new2(self.db.clone(), transaction.clone(), id)?;
                let template_entity_id: i64 = ec.get_template_entity_id(transaction.clone())?;

                let db: Rc<dyn Database> = self.db.clone();
                let e = Entity::new2(db.clone(), transaction.clone(), template_entity_id)?;
                Ok(Some(e))
            }
            _ => Err(anyhow!("Unexpected value for {:?}", class_id)),
        }
    }

    fn get_highest_sorting_index(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<i64, Error> {
        self.db
            .get_highest_sorting_index_for_group(transaction, self.get_id())
    }

    fn get_containing_relations_to_group(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        starting_index_in: i64,
        max_vals_in: Option<u64>, /*= None*/
    ) -> Result<Vec<RelationToGroup>, Error> {
        let rtgs_data: Vec<(i64, i64, i64, i64, Option<i64>, i64, i64)> =
            self.db.get_relations_to_group_containing_this_group(
                transaction,
                self.get_id(),
                starting_index_in,
                max_vals_in,
            )?;
        let mut rtgs: Vec<RelationToGroup> = Vec::new();
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
            rtgs.push(rtg);
        }
        Ok(rtgs)
    }

    fn get_count_of_entities_containing_group(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<(u64, u64), Error> {
        self.db
            .get_count_of_entities_containing_group(transaction, self.get_id())
    }

    fn get_entities_containing_group(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        starting_index_in: i64,
        max_vals_in: Option<i64>, /*= None*/
    ) -> Result<Vec<(i64, Entity)>, Error> {
        let ids = self.db.get_entities_containing_group(
            transaction.clone(),
            self.get_id(),
            starting_index_in,
            max_vals_in,
        )?;
        let mut results: Vec<(i64, Entity)> = Vec::new();
        for (rel_type_id, entity_id) in ids {
            let entity = Entity::new2(self.db.clone(), transaction.clone(), entity_id)?;
            results.push((rel_type_id, entity));
        }
        Ok(results)
    }

    fn find_unused_sorting_index(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        starting_with_in: Option<i64>, /*= None*/
    ) -> Result<i64, Error> {
        self.db
            .find_unused_group_sorting_index(transaction, self.get_id(), starting_with_in)
    }

    fn get_groups_containing_entitys_groups_ids(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        limit_in: Option<i64>, /*= Some(5)*/
    ) -> Result<Vec<Vec<Option<DataType>>>, Error> {
        self.db
            .get_groups_containing_entitys_groups_ids(transaction, self.get_id(), limit_in)
    }

    fn is_entity_in_group(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        entity_id_in: i64,
    ) -> Result<bool, Error> {
        self.db
            .is_entity_in_group(transaction, self.get_id(), entity_id_in)
    }

    fn get_adjacent_group_entries_sorting_indexes(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        sorting_index_in: i64,
        limit_in: Option<i64>, /*= None*/
        forward_not_back_in: bool,
    ) -> Result<Vec<Vec<Option<DataType>>>, Error> {
        self.db.get_adjacent_group_entries_sorting_indexes(
            transaction,
            self.get_id(),
            sorting_index_in,
            limit_in,
            forward_not_back_in,
        )
    }

    fn get_nearest_group_entrys_sorting_index(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        starting_point_sorting_index_in: i64,
        forward_not_back_in: bool,
    ) -> Result<Option<i64>, Error> {
        self.db.get_nearest_group_entrys_sorting_index(
            transaction,
            self.get_id(),
            starting_point_sorting_index_in,
            forward_not_back_in,
        )
    }

    fn get_entry_sorting_index(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        entity_id_in: i64,
    ) -> Result<i64, Error> {
        self.db
            .get_group_entry_sorting_index(transaction, self.get_id(), entity_id_in)
    }

    fn is_group_entry_sorting_index_in_use(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        sorting_index_in: i64,
    ) -> Result<bool, Error> {
        self.db
            .is_group_entry_sorting_index_in_use(transaction, self.get_id(), sorting_index_in)
    }

    fn update_sorting_index(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        entity_id_in: i64,
        sorting_index_in: i64,
    ) -> Result<u64, Error> {
        self.db.update_sorting_index_in_a_group(
            transaction,
            self.get_id(),
            entity_id_in,
            sorting_index_in,
        )
    }

    fn renumber_sorting_indexes<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
    ) -> Result<(), Error> {
        self.db
            .renumber_sorting_indexes(transaction, self.get_id(), false)
    }

    fn move_entity_from_group_to_local_entity(
        &self,
        to_entity_id_in: i64,
        move_entity_id_in: i64,
        sorting_index_in: i64,
    ) -> Result<(), Error> {
        self.db.move_entity_from_group_to_local_entity(
            self.get_id(),
            to_entity_id_in,
            move_entity_id_in,
            sorting_index_in,
        )
    }

    fn move_entity_to_different_group(
        &self,
        to_group_id_in: i64,
        move_entity_id_in: i64,
        sorting_index_in: i64,
    ) -> Result<(), Error> {
        self.db.move_local_entity_from_group_to_group(
            self.get_id(),
            to_group_id_in,
            move_entity_id_in,
            sorting_index_in,
        )
    }
}

#[cfg(test)]
mod test {
    use crate::model::database::Database;
    use crate::model::entity::Entity;
    use crate::model::group::Group;
    use crate::model::postgres::postgresql_database::PostgreSQLDatabase;
    use crate::model::relation_type::RelationType;
    use crate::util::Util;
    use sqlx::{Postgres, Transaction};
    use std::cell::RefCell;
    use std::collections::HashSet;
    use std::rc::Rc;
    /*
     let mut db: PostgreSQLDatabase = null;

     // using the real db because it got too complicated with mocks, and the time savings don't seem enough to justify the work with the mocks. (?)
     override fn runTests(testName: Option<String>, args: Args) -> Status {
       setUp()
       let result:Status = super.runTests(testName,args);
       // (See comment inside PostgreSQLDatabaseTest.runTests about "db setup/teardown")
       result
     }

     protected fn setUp() {
       //start fresh
       PostgreSQLDatabaseTest.tearDownTestDB()

       // instantiation does DB setup (creates tables, default data, etc):
       db = new PostgreSQLDatabase(Database.TEST_USER, Database.TEST_PASS)
     }

     protected fn tearDown() {
       PostgreSQLDatabaseTest.tearDownTestDB()
     }
    */
 //%%

    #[test]
    fn move_entity_to_different_group_etc() {
        Util::initialize_tracing();
        let db: Rc<PostgreSQLDatabase> = Rc::new(Util::initialize_test_db().unwrap());
        let gid1 = db.create_group(None, "group_name1", false).unwrap();
        let gid2 = db.create_group(None, "group_name2", false).unwrap();
        let group1 = Group::new2(db.clone(), None, gid1).unwrap();
        let group2 = Group::new2(db.clone(), None, gid2).unwrap();

        //let tx = db.begin_trans().unwrap();
        //let tx: Option<Rc<RefCell<Transaction<Postgres>>>> = Some(Rc::new(RefCell::new(tx)));
        //See below 2 calls that do not take a "transaction" (noted there). Would it fail if using the above 2
        //lines instead of the "let tx = None;" just below?
        let tx = None;

        let eid1 = db.create_entity(tx.clone(), "e1", None, None).unwrap();
        let e1 = Entity::new2(db.clone(), tx.clone(), eid1).unwrap();

        // Add the entity to group1 and verify its presence.
        group1.add_entity(tx.clone(), e1.get_id(), None).unwrap();
        assert!(group1.is_entity_in_group(tx.clone(), e1.get_id()).unwrap());
        assert!(!group2.is_entity_in_group(tx.clone(), e1.get_id()).unwrap());

        // Move the entity from group1 to group2 and verify the change.
        // why doesnt this call take a transaction? See note at top of this test.
        group1
            .move_entity_to_different_group(group2.get_id(), e1.get_id(), -1)
            .unwrap();
        assert!(!group1.is_entity_in_group(tx.clone(), e1.get_id()).unwrap());
        assert!(group2.is_entity_in_group(tx.clone(), e1.get_id()).unwrap());

        // Verify and update the sorting index.
        let index1 = group2
            .get_entry_sorting_index(tx.clone(), e1.get_id())
            .unwrap();
        assert_eq!(index1, -1);
        group2
            .update_sorting_index(tx.clone(), e1.get_id(), -2)
            .unwrap();
        assert_eq!(
            group2
                .get_entry_sorting_index(tx.clone(), e1.get_id())
                .unwrap(),
            -2
        );

        // Renumber sorting indexes and verify changes.
        group2.renumber_sorting_indexes(tx.clone()).unwrap();
        assert_ne!(
            group2
                .get_entry_sorting_index(tx.clone(), e1.get_id())
                .unwrap(),
            -1
        );
        assert_ne!(
            group2
                .get_entry_sorting_index(tx.clone(), e1.get_id())
                .unwrap(),
            -2
        );
        assert!(!group2
            .is_group_entry_sorting_index_in_use(tx.clone(), -1)
            .unwrap());
        assert!(!group2
            .is_group_entry_sorting_index_in_use(tx.clone(), -2)
            .unwrap());

        let index2: i64 = group2
            .get_entry_sorting_index(tx.clone(), e1.get_id())
            .unwrap();
        assert_ne!(
            group2.find_unused_sorting_index(tx.clone(), None).unwrap(),
            index2
        );

        // Add another entity to group2 and update its sorting index.
        let e3 = Entity::new2(
            db.clone(),
            tx.clone(),
            db.create_entity(tx.clone(), "e3", None, None).unwrap(),
        )
        .unwrap();
        group2.add_entity(tx.clone(), e3.get_id(), None).unwrap();
        group2
            .update_sorting_index(tx.clone(), e3.get_id(), db.min_id_value())
            .unwrap();

        // next lines not much of a test but is something:
        let index3: Option<i64> = group2
            .get_nearest_group_entrys_sorting_index(tx.clone(), db.min_id_value(), true)
            .unwrap();
        assert!(index3.unwrap() > db.min_id_value());
        /*val index4: i64 = */
        group2
            .get_entry_sorting_index(tx.clone(), e1.get_id())
            .unwrap();
        let indexes = group2
            .get_adjacent_group_entries_sorting_indexes(
                tx.clone(),
                db.min_id_value(),
                Some(0),
                true,
            )
            .unwrap();
        assert!(!indexes.is_empty());

        // Move entity from group to local entity.
        let e2 = Entity::new2(
            db.clone(),
            tx.clone(),
            db.create_entity(tx.clone(), "e2", None, None).unwrap(),
        )
        .unwrap();
        let mut hs: HashSet<i64> = HashSet::new();
        let results_in_out1: &HashSet<i64> = e2
            .find_contained_local_entity_ids(tx.clone(), &mut hs, "e2", 20, true)
            .unwrap();
        assert!(results_in_out1.is_empty());
        // why doesnt this call take a transaction? See note at top of this test.
        group2
            .move_entity_from_group_to_local_entity(e2.get_id(), e1.get_id(), 0)
            .unwrap();
        assert!(!group2.is_entity_in_group(tx.clone(), e1.get_id()).unwrap());

        let mut hs2: HashSet<i64> = HashSet::new();
        let results_in_out2: &HashSet<i64> = e2
            .find_contained_local_entity_ids(tx.clone(), &mut hs2, "e1", 20, true)
            .unwrap();
        assert_eq!(results_in_out2.len(), 1);
        assert!(results_in_out2.contains(&e1.get_id()));
    }

    #[test]
    fn get_groups_containing_entitys_groups_ids_etc_should_work() {
        Util::initialize_tracing();
        let db: Rc<PostgreSQLDatabase> = Rc::new(Util::initialize_test_db().unwrap());

        let g1 = Group::new2(
            db.clone(),
            None,
            db.create_group(None, "g1", false).unwrap(),
        )
        .unwrap();
        let g2 = Group::new2(
            db.clone(),
            None,
            db.create_group(None, "g2", false).unwrap(),
        )
        .unwrap();
        let g3 = Group::new2(
            db.clone(),
            None,
            db.create_group(None, "g3", false).unwrap(),
        )
        .unwrap();
        let e1 = Entity::new2(
            db.clone(),
            None,
            db.create_entity(None, "e1", None, None).unwrap(),
        )
        .unwrap();
        let e2 = Entity::new2(
            db.clone(),
            None,
            db.create_entity(None, "e2", None, None).unwrap(),
        )
        .unwrap();
        let rt_id = db
            .clone()
            .create_relation_type(None, "rt", "rtReversed", "BI")
            .unwrap();
        let rt = RelationType::new2(db.clone(), None, rt_id).unwrap();

        //let tx = db.begin_trans().unwrap();
        //let tx: Option<Rc<RefCell<Transaction<Postgres>>>> = Some(Rc::new(RefCell::new(tx)));
        let tx = None;

        g1.add_entity(tx.clone(), e1.get_id(), None).unwrap();
        g2.add_entity(tx.clone(), e2.get_id(), None).unwrap();
        e1.add_relation_to_group(tx.clone(), rt.get_id(), g3.get_id(), None)
            .unwrap();
        e2.add_relation_to_group(tx.clone(), rt.get_id(), g3.get_id(), None)
            .unwrap();
        let results = g3
            .get_groups_containing_entitys_groups_ids(tx.clone(), Some(5))
            .unwrap();
        assert_eq!(results.len(), 2);
        let entities = g3
            .get_entities_containing_group(tx.clone(), 0, None)
            .unwrap();
        assert_eq!(entities.len(), 2);
        assert_eq!(
            g3.get_count_of_entities_containing_group(tx.clone())
                .unwrap()
                .0,
            2
        );
        assert_eq!(
            g3.get_containing_relations_to_group(tx.clone(), 0, None)
                .unwrap()
                .len(),
            2
        );
        assert!(Group::get_group(db, tx.clone(), g3.get_id())
            .unwrap()
            .is_some());
    }
}
