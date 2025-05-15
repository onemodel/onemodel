/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2013-2017 inclusive, and 2023-2025 inclusive, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>

  ---------------------------------------------------
  If we ever do port to another database, create the Database interface (removed around 2014-1-1 give or take) and see other changes at that time.
*/
// use std::os::unix::process::parent_id;
use crate::model::attribute_with_valid_and_observed_dates::AttributeWithValidAndObservedDates;
use crate::model::database::{DataType, Database};
use crate::util::Util;
use anyhow::{anyhow, Error, Result};
// use sqlx::{PgPool, Postgres, Row, Transaction};
use crate::model::attribute::Attribute;
use crate::model::entity::Entity;
use crate::model::group::Group;
// use crate::model::id_wrapper::IdWrapper;
use crate::model::relation_type::RelationType;
use sqlx::{Postgres, Transaction};
use std::cell::RefCell;
use std::rc::Rc;
//use tracing_subscriber::registry::Data;

// ***NOTE***: Similar/identical code found in *_attribute.rs, relation_to_entity.rs and relation_to_group.rs,
// due to Rust limitations on OO.  Maintain them all similarly.

//#[derive(Clone)]
pub struct RelationToGroup {
    // For descriptions of the meanings of these variables, see the comments
    // on create_quantity_attribute(...) or create_tables() in PostgreSQLDatabase or Database structs,
    // and/or examples in the database testing code.
    db: Rc<RefCell<dyn Database>>,
    id: i64,
    entity_id: i64,
    // Unlike most other things that implement Attribute, rel_type_id takes the place of attr_type_id in this, since
    // unlike in the scala code self does not extend Attribute and inherit attr_type_id.
    rel_type_id: i64,
    group_id: i64,
    already_read_data: bool, /*= false*/
    // %%parent_id: i64,             /*= 0_i64*/
    valid_on_date: Option<i64>, /*= None*/
    observation_date: i64,      /*= 0_i64*/
    sorting_index: i64,         /*= 0_i64*/
}

impl RelationToGroup {
    /// This one is perhaps only called by the database class implementation [AND BELOW]--so it can return arrays of objects & save more DB hits
    /// that would have to occur if it only returned arrays of keys. This DOES NOT create a persistent object--but rather should reflect
    /// one that already exists.  It does not confirm that the id exists in the db.
    /// See comment about these 2 dates in PostgreSQLDatabase.create_tables()
    pub fn new(
        db: Rc<RefCell<dyn Database>>,
        id: i64,
        entity_id: i64,
        rel_type_id: i64,
        group_id: i64,
        valid_on_date: Option<i64>,
        observation_date: i64,
        sorting_index: i64,
    ) -> RelationToGroup {
        RelationToGroup {
            db,
            id,
            entity_id,
            rel_type_id,
            group_id,
            already_read_data: true,
            // %%parent_id: entity_id,
            valid_on_date,
            observation_date,
            sorting_index,
        }
    }

    pub fn new2(
        db: Rc<RefCell<dyn Database>>,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id: i64,
        entity_id: i64,
        rel_type_id: i64,
        group_id: i64,
    ) -> Result<RelationToGroup, anyhow::Error> {
        // (See comment in similar spot in BooleanAttribute for why not checking for exists, if db.is_remote.)
        // if db.is_remote || db.relation_to_group_keys_exist_and_match(transaction, id, entity_id, rel_type_id, group_id) {
        // something else might be cleaner, but these are the same thing and we need to make sure that (what was
        // in the scala code) the superclass' let mut doesn't overwrite this w/ 0:
        // } else {
        if !db.borrow().is_remote()
            && !db.borrow().relation_to_group_keys_exist_and_match(
                transaction,
                id,
                entity_id,
                rel_type_id,
                group_id,
            )?
        {
            Err(anyhow!(
                "Key id={}, {}/{}/{}{}",
                id,
                entity_id,
                rel_type_id,
                group_id,
                Util::DOES_NOT_EXIST
            ))
        } else {
            Ok(RelationToGroup {
                db,
                id,
                entity_id,
                rel_type_id,
                group_id,
                already_read_data: false,
                //%% parent_id: 0,
                valid_on_date: None,
                observation_date: 0,
                sorting_index: 0,
            })
        }
    }

    /// Just reading from DB, not creating something new here.
    // Old idea?: could change this into a constructor if the fn new's parameters are changed to be only db, transaction, and id, and a new constructor is created
    // to fill in the other fields. But didn't do that because it would require an extra db read with every use, and the ordering of statements in the
    // new constructors just wasn't working out (in scala code when I originally wrote this comment, anyway?).
    ///See comments on fn new, here.
    pub fn new3(
        db: Rc<RefCell<dyn Database>>,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id_in: i64,
    ) -> Result<RelationToGroup, anyhow::Error> {
        let relation_data: Vec<Option<DataType>> =
            db.borrow().get_relation_to_group_data(transaction, id_in)?;
        if relation_data.len() == 0 {
            return Err(anyhow!(
                "No results returned from data request for: {}",
                id_in
            ));
        };
        let DataType::Bigint(entity_id) = relation_data[1].clone().unwrap() else {
            return Err(anyhow!("Unexpected entity_id: {:?}", relation_data[1]));
        };
        let DataType::Bigint(rel_type_id) = relation_data[2].clone().unwrap() else {
            return Err(anyhow!("Unexpected rel_type_id: {:?}", relation_data[2]));
        };
        let DataType::Bigint(group_id) = relation_data[3].clone().unwrap() else {
            return Err(anyhow!("Unexpected group_id: {:?}", relation_data[3]));
        };
        let valid_on_date: Option<i64> = match relation_data[4] {
            None => None,
            Some(DataType::Bigint(vod)) => Some(vod),
            _ => return Err(anyhow!("Unexpected valid_on_date: {:?}", relation_data[4])),
        };
        let DataType::Bigint(observation_date) = relation_data[5].clone().unwrap() else {
            return Err(anyhow!(
                "Unexpected observation_date: {:?}",
                relation_data[5]
            ));
        };
        let DataType::Bigint(sorting_index) = relation_data[6].clone().unwrap() else {
            return Err(anyhow!("Unexpected sorting_index: {:?}", relation_data[6]));
        };
        Ok(RelationToGroup::new(
            db,
            id_in,
            entity_id,
            rel_type_id,
            group_id,
            valid_on_date,
            observation_date,
            sorting_index,
        ))
    }

    pub fn get_group_id(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<i64, anyhow::Error> {
        if !self.already_read_data {
            self.read_data_from_db(transaction)?;
        }
        Ok(self.group_id)
    }

    fn get_group(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<Group, anyhow::Error> {
        Group::new2(
            self.db.clone(),
            transaction.clone(),
            self.get_group_id(transaction.clone())?,
        )
    }

    fn move_it(
        &self,
        new_containing_entity_id_in: i64,
        sorting_index_in: i64,
    ) -> Result<i64, anyhow::Error> {
        self.db.borrow()
            .move_relation_to_group(self.get_id(), new_containing_entity_id_in, sorting_index_in)
    }

    fn update(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        new_relation_type_id_in: Option<i64>,
        new_group_id_in: Option<i64>,
        valid_on_date_in: Option<i64>,
        observation_date_in: Option<i64>,
    ) -> Result<u64, anyhow::Error> {
        //Idea/possible bug: see comment on similar method in RelationToEntity (or maybe in its subclasses).
        let new_relation_type_id: i64 = match new_relation_type_id_in {
            Some(x) => x,
            None => self.get_attr_type_id(transaction.clone())?,
        };
        let new_group_id: i64 = match new_group_id_in {
            Some(x) => x,
            None => self.get_group_id(transaction.clone())?,
        };
        let vod = match valid_on_date_in {
            //Use valid_on_date_in rather than valid_on_date_in.get because self.valid_on_date allows None, unlike others.
            Some(_x) => valid_on_date_in,
            None => self.get_valid_on_date(transaction.clone())?,
        };
        let od = match observation_date_in {
            Some(x) => x,
            None => self.get_observation_date(transaction.clone())?,
        };
        let rows_affected = self.db.borrow().update_relation_to_group(
            transaction,
            self.entity_id,
            self.rel_type_id,
            new_relation_type_id,
            self.group_id,
            new_group_id,
            vod,
            od,
        )?;
        //%%why weren't next 2 lines found in the scala version of this?
        self.rel_type_id = new_relation_type_id;
        self.group_id = new_group_id;
        self.valid_on_date = vod;
        self.observation_date = od;
        Ok(rows_affected)
    }

    /// Removes this object from the system.
    fn delete_group_and_relations_to_it<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
    ) -> Result<(), anyhow::Error> {
        self.db.borrow()
            .delete_group_and_relations_to_it(transaction, self.group_id)
    }
}

impl Attribute for RelationToGroup {
    fn get_display_string(
        &mut self,
        length_limit_in: usize,
        _unused: Option<Entity>,        /*= None*/
        _unused2: Option<RelationType>, /*=None*/
        simplify: bool,                 /* = false*/
    ) -> Result<String, anyhow::Error> {
        let mut group = Group::new2(self.db.clone(), None, self.get_group_id(None)?)?;
        let rt_name = RelationType::new2(self.db.clone(), None, self.get_attr_type_id(None)?)?
            .get_name(None)?;
        let mut result: String = if simplify && rt_name == Util::THE_HAS_RELATION_TYPE_NAME {
            "".to_string()
        } else {
            format!("{} ", rt_name)
        };
        result = format!("{}{}", result, group.get_display_string(None, 0, simplify)?);
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
        let data: Vec<Option<DataType>> = self.db.borrow().get_relation_to_group_data_by_keys(
            transaction,
            self.entity_id,
            self.rel_type_id,
            self.group_id,
        )?;
        if data.len() == 0 {
            return Err(anyhow!(
                "No results returned from data request for: {}, {}, {}",
                self.entity_id,
                self.rel_type_id,
                self.group_id
            ));
        }

        //***ONLY ROUGHLY COPIED***:
        //BEGIN COPIED BLOCK descended from Attribute.assign_common_vars (unclear how to do better for now):
        //except omitting this one since entity_id takes the place of parent_id here (see below re attr_type_id)
        // self.parent_id = match data[1] {
        //     Some(DataType::Bigint(x)) => x,
        //     _ => return Err(anyhow!("How did we get here for {:?}?", data[1])),
        // };
        // except omitting this one, since rel_type_id takes the place of attr_type_id in this, since
        // unlike in the scala code self does not extend Attribute and inherit attr_type_id.
        // self.attr_type_id = match data[2] {
        //     Some(DataType::Bigint(x)) => x,
        //     _ => return Err(anyhow!("How did we get here for {:?}?", data[2])),
        // };
        self.sorting_index = match data[6] {
            Some(DataType::Bigint(x)) => x,
            _ => return Err(anyhow!("How did we get here for {:?}?", data[6])),
        };
        //END COPIED BLOCK descended from Attribute.assign_common_vars (might be in comment in boolean_attribute.rs)

        //***ONLY ROUGHLY COPIED***:
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

    /// Removes this object from the system.
    fn delete<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
    ) -> Result<u64, anyhow::Error> {
        self.db.borrow().delete_relation_to_group(
            transaction,
            self.entity_id,
            self.rel_type_id,
            self.group_id,
        )
    }

    // This datum is provided upon construction (new2(), at minimum), so can be returned
    // regardless of already_read_data / read_data_from_db().
    fn get_id(&self) -> i64 {
        self.id
    }

    fn get_form_id(&self) -> Result<i32, Error> {
        self.db.borrow().get_attribute_form_id(Util::RELATION_TO_GROUP_TYPE)
    }

    fn get_attr_type_id(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<i64, anyhow::Error> {
        if !self.already_read_data {
            self.read_data_from_db(transaction)?;
        }
        Ok(self.rel_type_id)
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
        Ok(self.entity_id)
    }
}

impl AttributeWithValidAndObservedDates for RelationToGroup {
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
    use super::*;
    use crate::model::{
        entity_class::EntityClass, postgres::postgresql_database::PostgreSQLDatabase,
    };
    use tracing::*;

    // See comments about mockall, in boolean_attribute.rs.

    /*%%put this back after similar place in boolean_attribute.rs is resolved and this can be similarly:
      let mut db: PostgreSQLDatabase = null;

      // Starting to use the real db because the time savings don't seem enough to justify the work with the mocks. (?)
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

    /*
          "get_display_string" should "return correct string and length" in {
            let mock_db = mock[PostgreSQLDatabase];

            // arbitrary...:
            let rtgId: i64 = 300;
            let groupId: i64 = 301;
            let entity_id: i64 = 302;
            let classTemplateEntityId: i64 = 303;
            let rel_type_id: i64 = 401;
            let classId: i64 = 501;
            let grpName: String = "somename";
            let grpEntryCount = 9;
            // arbitrary, in milliseconds:
            let date = 304;
            let relationTypeName: String = Database.THE_HAS_RELATION_TYPE_NAME;
            when(mock_db.group_key_exists(groupId)).thenReturn(true)
            when(mock_db.relation_type_key_exists(rel_type_id)).thenReturn(true)
            when(mock_db.entity_key_exists(rel_type_id)).thenReturn(true)
            when(mock_db.relation_to_group_keys_exist_and_match(rtgId, entity_id, rel_type_id, groupId)).thenReturn(true)
            when(mock_db.get_group_data(groupId)).thenReturn(Vec<Option<DataType>>(Some(grpName), Some(0L), Some(true), Some(false)))
            when(mock_db.get_group_size(groupId, 1)).thenReturn(grpEntryCount)
            when(mock_db.get_relation_type_data(rel_type_id)).thenReturn(Vec<Option<DataType>>(Some(relationTypeName), Some(Database.THE_IS_HAD_BY_REVERSE_NAME), Some("xyz..")))
            when(mock_db.get_remote_address).thenReturn(None)

            // (using arbitrary numbers for the unnamed parameters):
            let relationToGroup = new RelationToGroup(mock_db, rtgId, entity_id, rel_type_id, groupId, None, date, 0);
            let small_limit = 15;
            let observed_dateOutput = "Wed 1969-12-31 17:00:00:" + date + " MST";
            let whole_thing: String = relationTypeName + " grp " + groupId + " /" + grpEntryCount + ": " + grpName + ", class: (mixed); valid unsp'd, obsv'd " + observed_dateOutput;

            let displayed: String = relationToGroup.get_display_string(small_limit, None);
            let expected = whole_thing.substring(0, small_limit - 3) + "...";
            assert(displayed == expected)
            // idea (is in tracked tasks): put next 2 lines back after color refactoring is done (& places w/ similar comment elsewhere)
            //  let all: String = relationToGroup.get_display_string(0, None);
            //  assert(all == whole_thing)

            let relationToGroup2 = new RelationToGroup(mock_db, rtgId, entity_id, rel_type_id, groupId, None, date, 0);
            when(mock_db.get_group_data(groupId)).thenReturn(Vec<Option<DataType>>(Some(grpName), Some(0L), Some(false), Some(false)))
            let all2: String = relationToGroup2.get_display_string(0, None);
            assert(!all2.contains("(mixed)"))
            assert(all2.contains(", class: (unspecified)"))

            let relationToGroup3 = new RelationToGroup(mock_db, rtgId, entity_id, rel_type_id, groupId, None, date, 0);
            when(mock_db.entity_key_exists(classTemplateEntityId)).thenReturn(true)
            let list = new Vec<Entity>(1);
            list.add(new Entity(mock_db, classTemplateEntityId, "asdf", None, 0L, None, false, false))
            when(mock_db.get_group_entry_ids(groupId, 0, Some(1))).thenReturn(list)
            when(mock_db.get_group_size(groupId, 3)).thenReturn(list.size)
            let all3: String = relationToGroup3.get_display_string(0, None);
            assert(!all3.contains("(mixed)"))
            assert(all3.contains(", class: (specified as None)"))

            let relationToGroup4 = new RelationToGroup(mock_db, rtgId, entity_id, rel_type_id, groupId, None, date, 0);
            let list4 = new Vec<Entity>(1);
            list4.add(new Entity(mock_db, classTemplateEntityId, "asdf", Some(classId), 0L, Some(true), false, false))
            when(mock_db.entity_key_exists(classTemplateEntityId)).thenReturn(true)
            when(mock_db.class_key_exists(classId)).thenReturn(true)
            when(mock_db.get_group_entry_ids(groupId, 0, Some(1))).thenReturn(list4)
            let className = "someClassName";
            when(mock_db.get_class_data(classId)).thenReturn(Vec<Option<DataType>>(Some(className), Some(classTemplateEntityId), Some(true)))
            let all4: String = relationToGroup4.get_display_string(0, None);
            assert(!all4.contains("(mixed)"))
            assert(all4.contains(", class: " + className))
          }
    */

    #[test]
    fn rtg_get_display_string_returns_correct_string_and_length() {
        Util::initialize_tracing();
        let db: Rc<RefCell<PostgreSQLDatabase>>= Rc::new(RefCell::new(Util::initialize_test_db().unwrap()));
        //using None instead of transactions to avoid some complexities not important for a test.
        //let tx = db.begin_trans().unwrap();
        //let tx: Option<Rc<RefCell<Transaction<Postgres>>>> = Some(Rc::new(RefCell::new(tx)));

        // See comment about mockall in boolean_attribute.rs. These comments are here for reference
        // in case I use mocks later.
        // let mock_db = mock[PostgreSQLDatabase];
        // arbitrary...:
        //let entity_id: i64 = 302;
        //let rel_type_id: i64 = 401;
        //let group_id: i64 = 301;
        //let rtg_id: i64 = 300;
        //let class_template_entity_id: i64 = 303;
        //let class_id: i64 = 501;
        let grp_name = "somename";
        let grp_entry_count = 2;
        //// arbitrary, in milliseconds:
        let date = 304;
        let relation_type_name: String = Util::THE_HAS_RELATION_TYPE_NAME.to_string();
        // when(mock_db.group_key_exists(group_id)).thenReturn(true)
        // when(mock_db.relation_type_key_exists(rel_type_id)).thenReturn(true)
        // when(mock_db.entity_key_exists(rel_type_id)).thenReturn(true)
        // when(mock_db.relation_to_group_keys_exist_and_match(rtg_id, entity_id, rel_type_id, group_id)).thenReturn(true)
        // when(mock_db.get_group_data(group_id)).thenReturn(Vec<Option<DataType>>(Some(grp_name), Some(0L),
        // Some(true), Some(false)))
        // when(mock_db.get_group_size(group_id, 1)).thenReturn(grp_entry_count)
        // when(mock_db.get_relation_type_data(rel_type_id)).thenReturn(Vec<Option<DataType>>(Some(relation_type_name),
        // Some(Database.THE_IS_HAD_BY_REVERSE_NAME), Some("xyz..")))
        // when(mock_db.get_remote_address).thenReturn(None)

        // Now, work around the lack of above mocks:
        let entity = Entity::create_entity(db.clone(), None, "entity1", None, None).unwrap();
        let entity2 = Entity::create_entity(db.clone(), None, "entity2", None, None).unwrap();
        let entity3 = Entity::create_entity(db.clone(), None, "entity3", None, None).unwrap();
        let rel_type_id = db.borrow()
            .create_relation_type(
                None,
                &relation_type_name,
                "name_in_reverse_direction_in",
                RelationType::UNIDIRECTIONAL,
            )
            .unwrap();
        let group_id = db.borrow().create_group(None, &grp_name, true).unwrap();
        let (rtg_id, _sorting_index) = db.borrow()
            .create_relation_to_group(
                None,
                entity.get_id(),
                rel_type_id,
                group_id,
                None,
                date,
                None,
            )
            .unwrap();
        let mut group: Group = Group::new2(db.clone(), None, group_id).unwrap();
        group.add_entity(None, entity2.get_id(), None).unwrap();
        group.add_entity(None, entity3.get_id(), None).unwrap();

        // (using arbitrary numbers for the unnamed parameters):
        let mut relation_to_group = RelationToGroup::new2(
            db.clone(),
            None,
            rtg_id,
            entity.get_id(),
            rel_type_id,
            group_id,
        )
        .unwrap();
        let small_limit = 15;
        // See comment about (lack of) TZ abbreviations, at Util.DATEFORMAT.
        //let expected_date_output = format!("Wed 1969-12-31 17:00:00:{} MST", date);
        let expected_date_output = format!("Wed 1969-12-31 17:00:00:{} -07:00", date);
        let expected_longer: String = format!(
            "{} grp {} /{}: {}, class: (mixed); valid unsp'd, obsv'd {}",
            relation_type_name, group_id, grp_entry_count, grp_name, expected_date_output
        );
        let displayed: String = relation_to_group
            .get_display_string(small_limit, None, None, false)
            .unwrap();
        let expected = format!("{}...", &expected_longer[0..small_limit - 3]);

        //let displayed_longer: String = relation_to_group.get_display_string(0, None, None, false).unwrap(); //%%
        //debug!("displayed: {}", displayed);
        //debug!("longer: {}", displayed_longer);
        //debug!("expected: {}", expected);
        //debug!("whole thing: {}", expected_longer);

        assert_eq!(displayed, expected);

        // idea (is in tracked tasks): put next 2 lines back after color refactoring is
        // done (& places w/ similar comment elsewhere)
        // let all: String = relation_to_group.get_display_string(tx.clone(), 0, None).unwrap();
        // assert_eq!(all, whole_thing);

        let mut relation_to_group2 = RelationToGroup::new2(
            db.clone(),
            None,
            rtg_id,
            entity.get_id(),
            rel_type_id,
            group_id,
        )
        .unwrap();
        // when(mock_db.get_group_data(group_id)).thenReturn(Vec<Option<DataType>>(Some(grp_name), Some(0L), Some(false), Some(false)))
        group
            .update(None, None, None, Some(false), None, None, None)
            .unwrap();
        let all2: String = relation_to_group2
            .get_display_string(0, None, None, false)
            .unwrap();
        assert!(!all2.contains("(mixed)"));
        assert!(all2.contains(", class: (specified as None)"));

        group.remove_entity(None, entity2.get_id()).unwrap();
        group.remove_entity(None, entity3.get_id()).unwrap();
        let mut relation_to_group3 = RelationToGroup::new2(
            db.clone(),
            None,
            rtg_id,
            entity.get_id(),
            rel_type_id,
            group_id,
        )
        .unwrap();
        // when(mock_db.entity_key_exists(class_template_entity_id)).thenReturn(true)
        // let list = new Vec<Entity>(1);
        // list.add(new Entity(mock_db, class_template_entity_id, "asdf", None, 0L, None, false, false))
        // when(mock_db.get_group_entry_ids(group_id, 0, Some(1))).thenReturn(list)
        // when(mock_db.get_group_size(group_id, 3)).thenReturn(list.size)
        let all3: String = relation_to_group3
            .get_display_string(0, None, None, false)
            .unwrap();
        assert!(!all3.contains("(mixed)"));
        assert!(all3.contains(", class: (unspecified)"));

        let mut relation_to_group4 = RelationToGroup::new2(
            db.clone(),
            None,
            rtg_id,
            entity.get_id(),
            rel_type_id,
            group_id,
        )
        .unwrap();
        // let list4 = new Vec<Entity>(1);
        // list4.add(new Entity(mock_db, class_template_entity_id, "asdf", Some(class_id), 0L, Some(true), false, false))
        // when(mock_db.entity_key_exists(class_template_entity_id)).thenReturn(true)
        // when(mock_db.class_key_exists(class_id)).thenReturn(true)
        // when(mock_db.get_group_entry_ids(group_id, 0, Some(1))).thenReturn(list4)

        //group.add_entity(None, entity2.get_id(), None).unwrap();
        let class_name = "someClassName";
        let (class_id, _class_template_entity_id) = db.borrow()
            .create_class_and_its_template_entity(None, class_name)
            .unwrap();
        let entity4 =
            Entity::create_entity(db.clone(), None, "entity4", Some(class_id), None).unwrap();
        group.add_entity(None, entity4.get_id(), None).unwrap();
        // when(mock_db.get_class_data(class_id)).thenReturn(Vec<Option<DataType>>(Some(class_name), Some(class_template_entity_id), Some(true)))
        let all4: String = relation_to_group4
            .get_display_string(0, None, None, false)
            .unwrap();
        debug!("all4 is: {}", all4);
        assert!(!all4.contains("(mixed)"));
        let s = format!(", class: {}", class_name);
        assert!(all4.contains(&s));

        // no need to db.rollback_trans(), because that is automatic when tx goes out of scope, per sqlx docs.
    }
    // %% */
    /*
      "getTemplateEntity" should "work right" in {
        let mock_db = mock[PostgreSQLDatabase];
        let rtgId: i64 = 300;
        let groupId: i64 = 301;
        //val parentId: i64 = 302
        let classTemplateEntityId: i64 = 303;
        let rel_type_id: i64 = 401;
        let entity_id: i64 = 402;
        let classId: i64 = 501;
        let className = "someclassname";
        let grpName: String = "somename";
        when(mock_db.relation_type_key_exists(rel_type_id)).thenReturn(true)
        when(mock_db.entity_key_exists(rel_type_id)).thenReturn(true)
        when(mock_db.relation_to_group_keys_exist_and_match(rtgId, entity_id, rel_type_id, groupId)).thenReturn(true)
        when(mock_db.group_key_exists(groupId)).thenReturn(true)

        let group = new Group(mock_db, groupId);
        when(mock_db.group_key_exists(groupId)).thenReturn(true)
        when(mock_db.entity_key_exists(entity_id)).thenReturn(true)
        when(mock_db.entity_key_exists(classTemplateEntityId)).thenReturn(true)
        when(mock_db.class_key_exists(classId)).thenReturn(true)
        when(mock_db.get_group_entry_ids(groupId, 0, Some(1))).thenReturn(new Vec<Entity>(0))
        when(mock_db.get_class_data(classId)).thenReturn(Vec<Option<DataType>>(Some(className), Some(classTemplateEntityId), Some(true)))
        when(mock_db.get_group_data(groupId)).thenReturn(Vec<Option<DataType>>(Some(grpName), Some(0L), Some(false), Some(false)))
        when(mock_db.get_remote_address).thenReturn(None)
        // should be None because it is not yet specified (no entities added):
        assert(group.get_class_template_entity.isEmpty)

        let list = new Vec<Entity>(1);
        let entity = new Entity(mock_db, entity_id, "testEntityName", Some(classId), 0L, Some(false), false, false);
        list.add(entity)
        when(mock_db.get_group_entry_ids(groupId, 0, Some(1))).thenReturn(list)
        // should be != None because mixed classes are NOT allowed in the group and an entity was added:
        assert(group.get_class_template_entity.get.get_id == classTemplateEntityId)

        //relationToGroup = new RelationToGroup(mock_db, entity_id, rel_type_id, groupId, None, date)
        // should be None when mixed classes are allowed in the group:
        when(mock_db.get_group_data(groupId)).thenReturn(Vec<Option<DataType>>(Some(grpName), Some(0L), Some(true), Some(false)))
        let group2 = new Group(mock_db, groupId);
        assert(group2.get_class_template_entity.isEmpty)
      }

      "move and update" should "work" in {
        let entity1 = new Entity(db, db.create_entity("entityName1"));
        let (_, rtg: RelationToGroup) = entity1.create_group_and_add_a_has_relation_to_it("group_name", mixedClassesAllowedIn = false, 0);
        let (attributeTuples1: Array[(i64, Attribute)], _) = entity1.get_sorted_attributes(0, 0);
        let rtg1 = attributeTuples1(0)._2.asInstanceOf[RelationToGroup];
        assert(rtg1.get_parent_id() == entity1.get_id)
        assert(rtg1.get_id == rtg.get_id)
        let rtg1_gid = rtg1.get_group_id;
        let rtg1_rtid = rtg1.get_attr_type_id();

        let entity2 = new Entity(db, db.create_entity("entityName2"));
        rtg.move_it(entity2.get_id, 0)

        let (attributeTuples1a: Array[(i64, Attribute)], _) = entity1.get_sorted_attributes(0, 0);
        assert(attributeTuples1a.length == 0)
        let (attributeTuples2: Array[(i64, Attribute)], _) = entity2.get_sorted_attributes(0, 0);
        let rtg2 = attributeTuples2(0)._2.asInstanceOf[RelationToGroup];
        let rtg2RelTypeId = rtg2.get_attr_type_id();
        let rtg2GroupId = rtg2.get_group_id;
        let vod2 = rtg2.get_valid_on_date();
        let od2 = rtg2.get_observation_date();
        assert(rtg2.get_parent_id() == entity2.get_id)
        assert(rtg2.get_parent_id() != entity1.get_id)
        assert(rtg1_gid == rtg2GroupId)
        assert(rtg1_rtid == rtg2RelTypeId)
        assert(rtg2.get_id != rtg.get_id)

        let new_relation_type_id = db.create_relation_type("RTName", "reversed", "BI");
        let new_group_id = db.create_group("newGroup");
        let newVod = Some(4321L);
        let newOd = Some(5432L);
        rtg2.update(Some(new_relation_type_id), Some(new_group_id), newVod, newOd)
        let rtg2a = new RelationToGroup(db, rtg2.get_id, rtg2.get_parent_id(), new_relation_type_id, new_group_id);
        assert(rtg2a.get_valid_on_date() != vod2)
        assert(rtg2a.get_valid_on_date().get == 4321L)
        assert(rtg2a.get_observation_date() != od2)
        assert(rtg2a.get_observation_date() == 5432L)
      }

    */
}
