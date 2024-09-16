/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2014-2017 inclusive, and 2023, Luke A. Call.
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

pub struct Group<'a> {
    id: i64,
    db: &'a dyn Database,
    already_read_data: bool,        /*= false*/
    name: String,                   /*= null*/
    insertion_date: i64,            /*= 0L*/
    mixed_classes_allowed: bool,    /*= false*/
    new_entries_stick_to_top: bool, /*= false*/
}

impl Group<'_> {
    /// Creates a new group in the database.
    fn create_group<'a>(
        db_in: &'a dyn Database,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        in_name: &str,
        allow_mixed_classes_in_group_in: bool, /*= false*/
    ) -> Result<Group<'a>, Error> {
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
    fn get_group<'a>(
        db_in: &'a dyn Database,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id: i64,
    ) -> Result<Option<Group<'a>>, Error> {
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
    fn new<'a>(
        db: &'a dyn Database,
        id: i64,
        name_in: &str,
        insertion_date: i64,
        mixed_classes_allowed: bool,
        new_entries_stick_to_top: bool,
    ) -> Group<'a> {
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
    pub fn new2<'a>(
        db: &'a dyn Database,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id: i64,
    ) -> Result<Group<'a>, Error> {
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
    fn update<'a>(
        &'a mut self,
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
        self.db.update_group(
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
        let name = &self.get_name(None)?;
        let formatted_name = format!("grp {} /{}: {}", self.id, num_entries, Color::blue(name));
        result.push_str(if simplify_in {
            name.as_str()
        } else {
            formatted_name.as_str()
        });
        if !simplify_in {
            result.push_str(", class: ");
            let class_name = {
                if self.get_mixed_classes_allowed(transaction)? {
                    "(mixed)".to_string()
                } else {
                    let class_name_option = self.get_class_name(None)?;
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
    fn delete<'a>(
        &'a self,
        ////_transaction: &Option<&mut Transaction<'a, Postgres>>,
        //_transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        //_id_in: i64,
    ) -> Result<(), Error> {
        self.db.delete_group_and_relations_to_it(self.id)
    }

    /// Removes an entity from this group.
    fn remove_entity<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        entity_id: i64,
    ) -> Result<u64, Error> {
        self.db
            .remove_entity_from_group(transaction, self.id, entity_id, false)
    }

    fn delete_with_entities(&self) -> Result<(), Error> {
        self.db
            .delete_group_relations_to_it_and_its_entries(self.id)
    }

    // idea: cache this?  when doing any other query also?  Is that safer because we really don't edit these in place (ie, immutability)?
    fn get_size(&self, include_which_entities: i32 /*= 3*/) -> Result<u64, Error> {
        self.db
            .get_group_size(None, self.id, include_which_entities)
    }

    fn get_group_entries(
        &self,
        starting_index_in: i64,
        max_vals_in: Option<i64>, /*= None*/
    ) -> Result<Vec<Entity>, Error> {
        self.db
            .get_group_entry_objects(None, self.id, starting_index_in, max_vals_in)
    }

    fn add_entity(
        &self,
        in_entity_id: i64,
        sorting_index_in: Option<i64>,        /*= None*/
        caller_manages_transactions_in: bool, /*= false*/
    ) -> Result<(), Error> {
        self.db.add_entity_to_group(
            None,
            self.get_id(),
            in_entity_id,
            sorting_index_in,
            caller_manages_transactions_in,
        )
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
                    if self.get_size(3)? == 0 {
                        // display should indicate that we know mixed are not allowed, so a class could be specified, but none has.
                        Ok(Some("(unspecified)".to_string()))
                    } else {
                        // means the group requires uniform classes, but the enforced uniform class is None, i.e., to not have a class:
                        Ok(Some("(specified as None)".to_string()))
                    }
                }
                Some(cid) => {
                    let mut example_entitys_class =
                        EntityClass::new2(self.db, transaction.clone(), cid)?;
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
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<Option<i64>, Error> {
        if self.mixed_classes_allowed {
            Ok(None)
        } else {
            let mut entries: Vec<Entity> =
                self.db
                    .get_group_entry_objects(transaction.clone(), self.get_id(), 0, Some(1))?;
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

    fn get_class_template_entity(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<Option<Entity>, Error> {
        let class_id: Option<i64> = self.get_class_id(transaction.clone())?;
        match class_id {
            None if self.get_mixed_classes_allowed(transaction.clone())? => Ok(None),
            Some(id) => {
                let mut ec = EntityClass::new2(self.db, transaction.clone(), id)?;
                let template_entity_id = ec.get_template_entity_id(transaction.clone())?;
                //Ok(Some(Entity::new2(self.db, transaction.clone(), template_entity_id)?))
                //if let Ok(db) = self.db.downcast::<&dyn Database>() {
                //if let db = *self.db {
                //    Ok(Some(Entity::new2(Box::new(db), transaction.clone(), template_entity_id)?))
                //} else {
                //    Err(anyhow!("Unexpected result from dereference, in group.get_class_template_entity?"))
                // }

                let db: &dyn Database = self.db;
                Ok(Some(Entity::new2(db, transaction, template_entity_id)?))
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
        self.db.get_relations_to_group_containing_this_group(
            transaction,
            self.get_id(),
            starting_index_in,
            max_vals_in,
        )
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
        self.db.get_entities_containing_group(
            transaction,
            self.get_id(),
            starting_index_in,
            max_vals_in,
        )
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
        caller_manages_transactions_in: bool, /*= false*/
    ) -> Result<(), Error> {
        self.db.renumber_sorting_indexes(
            transaction,
            self.get_id(),
            caller_manages_transactions_in,
            false,
        )
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

      "move_entity_to_different_group etc" should "work" in {
        let group1 = new Group(db, db.create_group("group_name1"));
        let group2 = new Group(db, db.create_group("group_name2"));
        let e1 = new Entity(db, db.create_entity("e1"));
        group1.add_entity(e1.get_id)
        assert(group1.is_entity_in_group(e1.get_id))
        assert(! group2.is_entity_in_group(e1.get_id))
        group1.move_entity_to_different_group(group2.get_id, e1.get_id, -1)
        assert(! group1.is_entity_in_group(e1.get_id))
        assert(group2.is_entity_in_group(e1.get_id))

        let index1 = group2.get_entry_sorting_index(e1.get_id);
        assert(index1 == -1)
        group2.update_sorting_index(e1.get_id, -2)
        assert(group2.get_entry_sorting_index(e1.get_id) == -2)
        group2.renumber_sorting_indexes()
        assert(group2.get_entry_sorting_index(e1.get_id) != -1)
        assert(group2.get_entry_sorting_index(e1.get_id) != -2)
        assert(! group2.isGroupEntrySortingIndexInUse(-1))
        assert(! group2.isGroupEntrySortingIndexInUse(-2))

        let index2: i64 = group2.get_entry_sorting_index(e1.get_id);
        assert(group2.find_unused_sorting_index(None) != index2)
        let e3: Entity = new Entity(db, db.create_entity("e3"));
        group2.add_entity(e3.get_id)
        group2.update_sorting_index(e3.get_id, Database.min_id_value)
        // next lines not much of a test but is something:
        let index3: Option<i64> = group2.get_nearest_group_entrys_sorting_index(Database.min_id_value, forward_not_back_in = true);
        assert(index3.get > Database.min_id_value)
        /*val index4: i64 = */group2.get_entry_sorting_index(e1.get_id)
        let indexes = group2.get_adjacent_group_entries_sorting_indexes(Database.min_id_value, Some(0), forward_not_back_in = true);
        assert(indexes.nonEmpty)

        let e2 = new Entity(db, db.create_entity("e2"));
        let results_in_out1: mutable.TreeSet[i64] = e2.find_contained_local_entity_ids(new mutable.TreeSet[i64], "e2");
        assert(results_in_out1.isEmpty)
        group2.move_entity_from_group_to_local_entity(e2.get_id, e1.get_id, 0)
        assert(! group2.is_entity_in_group(e1.get_id))
        let results_in_out2: mutable.TreeSet[i64] = e2.find_contained_local_entity_ids(new mutable.TreeSet[i64], "e1");
        assert(results_in_out2.size == 1)
        assert(results_in_out2.contains(e1.get_id))
      }

      "get_groups_containing_entitys_groups_ids etc" should "work" in {
        let group1 = new Group(db, db.create_group("g1"));
        let group2 = new Group(db, db.create_group("g2"));
        let group3 = new Group(db, db.create_group("g3"));
        let entity1 = new Entity(db, db.create_entity("e1"));
        let entity2 = new Entity(db, db.create_entity("e2"));
        group1.add_entity(entity1.get_id)
        group2.add_entity(entity2.get_id)
        let rt = new RelationType(db, db.create_relation_type("rt", "rtReversed", "BI"));
        entity1.addRelationToGroup(rt.get_id, group3.get_id, None)
        entity2.addRelationToGroup(rt.get_id, group3.get_id, None)
        let results = group3.get_groups_containing_entitys_groups_ids();
        assert(results.size == 2)

        let entities = group3.get_entities_containing_group(0);
        assert(entities.size == 2)
        assert(group3.get_count_of_entities_containing_group._1 == 2)
        assert(group3.get_containing_relations_to_group(0).size == 2)

        assert(Group.get_group(db, group3.get_id).is_defined)
      }

    */
}

// %%%%%
