/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2013-2017 inclusive, and 2023-2025 inclusive, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
// import java.io.{PrintWriter, StringWriter}
use anyhow::{anyhow, Error, Result};
//use crate::color::Color;
use crate::model::database::{DataType, Database};
use crate::model::id_wrapper::IdWrapper;
use crate::util::Util;
use sqlx::{Postgres, Transaction};
use std::cell::RefCell;
use std::rc::Rc;
use tracing::*;

pub struct EntityClass {
    db: Rc<RefCell<dyn Database>>,
    id: i64,
    already_read_data: bool,                 /*= false*/
    name: String,                            /*= null*/
    template_entity_id: i64,                 /*= 0*/
    create_default_attributes: Option<bool>, /*= None*/
}

impl EntityClass {
    fn name_length() -> u32 {
        Util::class_name_length()
    }

    pub fn is_duplicate(
        db_in: Rc<RefCell<dyn Database>>,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        in_name: &str,
        in_self_id_to_ignore: Option<i64>, /*= None*/
    ) -> Result<bool, Error> {
        db_in.borrow().is_duplicate_class_name(transaction, in_name, in_self_id_to_ignore)
    }

    /// This one is perhaps only called by the database class implementation--so it can return arrays of objects & save more DB hits
    ///  that would have to occur if it only returned arrays of keys. This DOES NOT create a persistent object--but rather should reflect
    /// one that already exists.  It does not confirm that the id exists in the db.
    pub fn new(
        db: Rc<RefCell<dyn Database>>,
        id: i64,
        name_in: &str,
        template_entity_id: i64,
        create_default_attributes: Option<bool>, /*= None*/
    ) -> EntityClass {
        EntityClass {
            db,
            id,
            name: name_in.to_string(),
            template_entity_id,
            create_default_attributes,
            already_read_data: true,
        }
    }

    /// See comments on similar methods in group.rs.
    pub fn new2(
        db: Rc<RefCell<dyn Database>>,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id: i64,
    ) -> Result<EntityClass, anyhow::Error> {
        // (See comment in similar spot in BooleanAttribute for why not checking for exists, if db.is_remote.)
        if !db.borrow().is_remote() && !db.borrow().class_key_exists(transaction, id)? {
            Err(anyhow!("Key {}{}", id, Util::DOES_NOT_EXIST))
        } else {
            Ok(EntityClass {
                id,
                db,
                already_read_data: false,
                name: "".to_string(),
                template_entity_id: 0,
                create_default_attributes: None,
            })
        }
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

    //pub fn get_template_entity_id<'a, 'b>(
    //    &'a mut self,
    //    transaction: Option<Rc<RefCell<Transaction<'b, Postgres>>>>,
    pub fn get_template_entity_id(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<i64, anyhow::Error>
//    where 'a: 'b
    {
        if !self.already_read_data {
            self.read_data_from_db(transaction)?
        }
        Ok(self.template_entity_id)
    }

    /// This is an associated function so the database can call it (without having to create a
    /// EntityClass instance, which I couldn't figure out how to do, in the database code, and it
    /// might not be a good idea anyway).
    pub fn get_template_entity_id_2(
        db: &dyn Database,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        entity_class_id: i64,
    ) -> Result<i64, anyhow::Error> {
        let data: Vec<Option<DataType>> = db.get_class_data(transaction, entity_class_id)?;
        if data.len() == 0 {
            return Err(anyhow!(
                "No results returned from data request for: {}",
                entity_class_id
            ));
        }
        let template_entity_id: i64 = match data[1] {
            Some(DataType::Bigint(x)) => x,
            _ => return Err(anyhow!("How did we get here for {:?}?", data[1])),
        };
        Ok(template_entity_id)
    }

    pub fn get_create_default_attributes(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<Option<bool>, Error> {
        if !self.already_read_data {
            self.read_data_from_db(transaction)?
        }
        Ok(self.create_default_attributes)
    }

    fn read_data_from_db(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<(), anyhow::Error> {
        let data: Vec<Option<DataType>> = self.db.borrow().get_class_data(transaction, self.id)?;
        if data.len() == 0 {
            return Err(anyhow!(
                "No results returned from data request for: {}",
                self.id
            ));
        }
        self.name = match data[0].clone() {
            Some(DataType::String(x)) => x,
            _ => return Err(anyhow!("How did we get here for {:?}?", data[0])),
        };
        self.template_entity_id = match data[1] {
            Some(DataType::Bigint(x)) => x,
            _ => return Err(anyhow!("How did we get here for {:?}?", data[1])),
        };
        self.create_default_attributes = match data[2] {
            Some(DataType::Boolean(x)) => Some(x),
            None => None,
            _ => return Err(anyhow!("How did we get here for {:?}?", data[2])),
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

    fn get_display_string_helper(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        // for explanation see comment in entity_class.rs fn get_display_string.
        fail: bool,
    ) -> Result<String, Error> {
        if fail {
            Err(anyhow!(
                "Testing: intentionally generated error in get_display_string_helper"
            ))
        } else {
            self.get_name(transaction)
        }
    }

    fn get_display_string(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        // This parameter is for testing, to avoid using mocking crates that got very many
        // lifetime or other errors from the compiler. (I tried mockall, unimock, faux, and mry.)
        fail: bool,
    ) -> String {
        let result = self.get_display_string_helper(transaction, fail);
        match result {
            Ok(s) => s,
            Err(e) => {
                debug!(
                    "Unable to get class description due to error.\nFull error is: {:?},\nand \
                     just as a short string: {} [end full error debug output]",
                    e,
                    e.to_string()
                );
                format!("{}", e)
            }
        }
    }

    fn update_class_and_template_entity_name<'a, 'b>(
        &'a mut self,
        transaction: Option<Rc<RefCell<Transaction<'b, Postgres>>>>,
        name_in: &str,
    ) -> Result<i64, anyhow::Error>
    where
        'a: 'b,
    {
        let template_entity_id: i64 = self.get_template_entity_id(transaction.clone())?;
        let ref rc_db = &self.db;
        let ref cloned_db = rc_db.clone();
        cloned_db.borrow().update_class_and_template_entity_name(
            transaction.clone(),
            self.get_id(),
            template_entity_id,
            name_in,
        )?;
        self.name = name_in.to_string();
        Ok(template_entity_id)
    }

    fn update_create_default_attributes(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        value_in: Option<bool>,
    ) -> Result<(), Error> {
        self.db.borrow()
            .update_class_create_default_attributes(transaction, self.get_id(), value_in)?;
        self.create_default_attributes = value_in;
        Ok(())
    }

    /** Removes this object etc from the system. */
    fn delete(&self, _transaction: &Option<&mut Transaction<Postgres>>) -> Result<(), Error> {
        self.db.borrow().delete_class_and_its_template_entity(self.id)
    }
}

#[cfg(test)]
mod test {
    use super::EntityClass;
    use crate::model::database::{/*DataType, */Database};
    use crate::model::entity::Entity;
    use crate::util::Util;
    use std::rc::Rc;
    use std::cell::RefCell;
    //use tracing::*;

    /*Idea: Maybe could do something like this again to make the tests run faster. Also in other structs' tests.
          let mut mTemplateEntity: Entity = null;
          let mut mEntityClass: EntityClass = null;
          let mut db: PostgreSQLDatabase = null;
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
            let name = "name of test class and its template entity";
            let (classId, entity_id): (i64, i64) = db.createClassAndItsTemplateEntity(name, name);
            mTemplateEntity = new Entity(db, entity_id)
            mEntityClass = new EntityClass(db, classId)
          }
          protected fn tearDown() {
            PostgreSQLDatabaseTest.tearDownTestDB()
          }
    // */

    #[test]
    fn get_display_string_returns_useful_stack_trace() {
        // For example, if the class has been deleted by one part of the code, or deleted by one user
        // process in a window (as an example), and is still referenced and attempted to
        // be displayed by another. Or to be somewhat helpful
        // if we try to get info on an class that's gone due to a bug.
        // (But should this issue go away w/ better design involving more use of immutability or something?)
        //
        //Was, when using mocks; is an idea for future (but see the "fail" parameter comment in
        //entity_class.rs fn get_display_string .):
        // An ID that will not exist in the test db, as we wouldn't have created enough objects to
        // get there (about a quintillion IIRC):
        //let id = 0;
        //let mock_db = mock[PostgreSQLDatabase];
        //when(mock_db.class_key_exists(id)).thenReturn(true)
        //when(mock_db.get_class_data(id)).thenThrow(new RuntimeException("some exception"))
        //let entityClass = new EntityClass(mock_db, id);

        Util::initialize_tracing();
        let db: Rc<RefCell<dyn Database>> = Rc::new(RefCell::new(Util::initialize_test_db().unwrap()));
        let tx = None;

        let (class_id, _entity_id) = db.borrow()
            .create_class_and_its_template_entity(tx.clone(), "testclass")
            .unwrap();
        let mut entity_class = EntityClass::new2(db.clone(), tx.clone(), class_id).unwrap();
        let ecs: String = entity_class.get_display_string(tx.clone(), true);
        assert!(ecs.contains("intentionally generated error"));
    }

    #[test]
    fn get_display_string_returns_name() {
        //ideas from when using mocks, in case we use them again:
        //let id = 0L;
        //let template_entity_id = 1L;
        //let mock_db = mock[PostgreSQLDatabase];
        //when(mock_db.class_key_exists(id)).thenReturn(true)
        //when(mock_db.get_class_name(id)).thenReturn(Some("class1Name"))
        //when(mock_db.get_class_data(id)).thenReturn(Vec<Option<DataType>>(Some("class1Name"),
        //    Some(template_entity_id), Some(true)))
        //let entityClass = new EntityClass(mock_db, id);

        Util::initialize_tracing();
        let db: Rc<RefCell<dyn Database>> = Rc::new(RefCell::new(Util::initialize_test_db().unwrap()));
        let tx = None;
        let (class_id, _entity_id) = db.borrow()
            .create_class_and_its_template_entity(tx.clone(), "class1Name")
            .unwrap();
        let mut entity_class = EntityClass::new2(db.clone(), tx.clone(), class_id).unwrap();
        let ecs: String = entity_class.get_display_string(tx.clone(), false);
        assert_eq!(ecs, "class1Name");
    }

    #[test]
    fn update_class_and_template_entity_name() {
        Util::initialize_tracing();
        let db: Rc<RefCell<dyn Database>> = Rc::new(RefCell::new(Util::initialize_test_db().unwrap()));
        //about begintrans: see comment farther below.
        //db.begin_trans()
        let tx = None;
        let (class_id, entity_id) = db.borrow()
            .create_class_and_its_template_entity(tx.clone(), "class2Name")
            .unwrap();
        let tmp_name = "garbage-temp";
        let mut ec = EntityClass::new2(db.clone(), None, class_id).unwrap();
        ec.update_class_and_template_entity_name(None, tmp_name)
            .unwrap();
        let name = ec.get_name(tx.clone()).unwrap();
        assert_eq!(name, tmp_name);
        // have to reread to see the change:
        assert_eq!(
            EntityClass::new2(db.clone(), tx.clone(), class_id)
                .unwrap()
                .get_name(tx.clone())
                .unwrap(),
            tmp_name
        );
        assert_eq!(
            Entity::new2(db.clone(), tx.clone(), entity_id)
                .unwrap()
                .get_name(tx.clone())
                .unwrap(),
            format!("{}{}", tmp_name, "-template").as_str()
        );
        // Could add this back & convert it to Rust, but transactions generally are tested in postgresql_database_tests.rs.
        //db.rollback_trans()
        //assert(new EntityClass(db, mEntityClass.get_id).get_name != tmpName)
        //assert(new Entity(db, mTemplateEntity.get_id).get_name != tmpName + "-template")
    }

    #[test]
    fn update_create_default_attributes() {
        Util::initialize_tracing();
        let db: Rc<RefCell<dyn Database>> = Rc::new(RefCell::new(Util::initialize_test_db().unwrap()));
        let tx = None;
        let (class_id, _entity_id) = db.borrow()
            .create_class_and_its_template_entity(tx.clone(), "class3Name")
            .unwrap();
        let mut ec = EntityClass::new2(db.clone(), tx.clone(), class_id).unwrap();

        assert_eq!(ec.get_create_default_attributes(tx.clone()).unwrap(), None);
        ec.update_create_default_attributes(tx.clone(), Some(true))
            .unwrap();
        assert_eq!(
            ec.get_create_default_attributes(tx.clone()).unwrap(),
            Some(true)
        );
        assert_eq!(
            EntityClass::new2(db.clone(), tx.clone(), class_id)
                .unwrap()
                .get_create_default_attributes(tx.clone())
                .unwrap(),
            Some(true)
        );
    }
}
