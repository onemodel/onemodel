/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2013-2017 inclusive, and 2023-2024 inclusive, Luke A. Call.
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
//use crate::model::entity::Entity;
use crate::model::id_wrapper::IdWrapper;
use crate::util::Util;
use sqlx::{Postgres, Transaction};
use std::cell::{RefCell};
use std::rc::Rc;

pub struct EntityClass<'a> {
    id: i64,
    db: &'a dyn Database,
    already_read_data: bool,                 /*= false*/
    name: String,                            /*= null*/
    template_entity_id: i64,                 /*= 0*/
    create_default_attributes: Option<bool>, /*= None*/
}

impl EntityClass<'_> {
    fn name_length() -> u32 {
        Util::class_name_length()
    }

    fn is_duplicate<'a>(
        db_in: &'a dyn Database,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        in_name: &str,
        in_self_id_to_ignore: Option<i64>, /*= None*/
    ) -> Result<bool, Error> {
        db_in.is_duplicate_class_name(transaction, in_name, in_self_id_to_ignore)
    }

    /// This one is perhaps only called by the database class implementation--so it can return arrays of objects & save more DB hits
    ///  that would have to occur if it only returned arrays of keys. This DOES NOT create a persistent object--but rather should reflect
    /// one that already exists.  It does not confirm that the id exists in the db.
    pub fn new<'a>(
        db: &'a dyn Database,
        id: i64,
        name_in: &str,
        template_entity_id: i64,
        create_default_attributes: Option<bool>, /*= None*/
    ) -> EntityClass<'a> {
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
    pub fn new2<'a>(
        db: &'a dyn Database,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id: i64,
    ) -> Result<EntityClass<'a>, anyhow::Error> {
        // (See comment in similar spot in BooleanAttribute for why not checking for exists, if db.is_remote.)
        if !db.is_remote() && !db.class_key_exists(transaction, id)? {
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

    pub fn get_template_entity_id(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<i64, anyhow::Error> {
        if !self.already_read_data {
            self.read_data_from_db(transaction)?
        }
        Ok(self.template_entity_id)
    }

    fn get_create_default_attributes(
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
        let data: Vec<Option<DataType>> = self.db.get_class_data(transaction, self.id)?;
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
        self.template_entity_id = match data[1] {
            Some(DataType::Bigint(x)) => x,
            _ => return Err(anyhow!("How did we get here for {:?}?", data[1])),
        };

        //%%%%% fix this next part after figuring out about what happens when querying a null back, in pg.db_query etc!
        //(like similar place in BooleanAttribute)
        // self.create_default_attributes = match data[2] {
        //     Some(DataType::Boolean(b)) => b,
        //     _ => return Err(anyhow!("How did we get here for {:?}?", data[2])),
        // };
        self.create_default_attributes = None;

        // create_default_attributes = classData(2).asInstanceOf[Option<bool>]
        self.already_read_data = true;
        Ok(())
    }

    fn get_id_wrapper(&self) -> IdWrapper {
        IdWrapper::new(self.id)
    }

    pub fn get_id(&self) -> i64 {
        self.id
    }

    fn get_display_string(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<String, Error> {
        self.get_name(transaction)
    }

    // fn get_display_string -> String {
    // let mut result = "";
    // try {
    //   result = get_display_string_helper
    // } catch {
    //   case e: Exception =>
    //     result += "Unable to get class description due to: "
    //     result += {
    //       let sw: StringWriter = new StringWriter();
    //       e.printStackTrace(new PrintWriter(sw))
    //       sw.toString
    //     }
    // }
    // result
    // }

    fn update_class_and_template_entity_name<'a>(
        &'a mut self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
        name_in: &str,
    ) -> Result<i64, anyhow::Error> {
        let template_entity_id: i64 = self.db.update_class_and_template_entity_name(
            transaction.clone(),
            self.get_id(),
            name_in,
        )?;
        self.name = name_in.to_string();
        let read_template_entity_id = self.get_template_entity_id(transaction)?;
        if self.template_entity_id != read_template_entity_id {
            return Err(anyhow!(
                "Template entity IDs do not match: {}, {}",
                self.template_entity_id,
                read_template_entity_id
            ));
        }
        Ok(template_entity_id)
    }

    fn update_create_default_attributes(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        value_in: Option<bool>,
    ) -> Result<(), Error> {
        self.db
            .update_class_create_default_attributes(transaction, self.get_id(), value_in)?;
        self.create_default_attributes = value_in;
        Ok(())
    }

    /** Removes this object etc from the system. */
    fn delete(&self, _transaction: &Option<&mut Transaction<Postgres>>) -> Result<(), Error> {
        self.db.delete_class_and_its_template_entity(self.id)
    }
}

#[cfg(test)]
mod test {
    /*
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

      "get_display_string" should "return a useful stack trace string, when called with a nonexistent class" in {
        // for example, if the class has been deleted by one part of the code, or one user process in a console window (as an example), and is still
        // referenced and attempted to be displayed by another (or to be somewhat helpful if we try to get info on an class that's gone due to a bug).
        // (But should this issue go away w/ better design involving more use of immutability or something?)
        let id = 0L;
        let mock_db = mock[PostgreSQLDatabase];
        when(mock_db.class_key_exists(id)).thenReturn(true)
        when(mock_db.get_class_data(id)).thenThrow(new RuntimeException("some exception"))

        let entityClass = new EntityClass(mock_db, id);
        let ec = entityClass.get_display_string;
        assert(ec.contains("Unable to get class description due to"))
        assert(ec.toLowerCase.contains("exception"))
        assert(ec.toLowerCase.contains("at org.onemodel"))
      }

      "get_display_string" should "return name" in {
        let id = 0L;
        let template_entity_id = 1L;
        let mock_db = mock[PostgreSQLDatabase];
        when(mock_db.class_key_exists(id)).thenReturn(true)
        when(mock_db.get_class_name(id)).thenReturn(Some("class1Name"))
        when(mock_db.get_class_data(id)).thenReturn(Vec<Option<DataType>>(Some("class1Name"), Some(template_entity_id), Some(true)))

        let entityClass = new EntityClass(mock_db, id);
        let ds = entityClass.get_display_string;
        assert(ds == "class1Name")
      }

      "update_class_and_template_entity_name" should "work" in {
        //about begintrans: see comment farther below.
    //    db.begin_trans()
        let tmpName = "garbage-temp";
        mEntityClass.update_class_and_template_entity_name(tmpName)
        assert(mEntityClass.name == tmpName)
        // have to reread to see the change:
        assert(new EntityClass(db, mEntityClass.get_id).get_name == tmpName)
        assert(new Entity(db, mTemplateEntity.get_id).get_name == tmpName + "-template")
        // See comment about next 3 lines, at the rollback_trans call at the end of the PostgreSQLDatabaseTest.scala test
        // "getAttrCount, get_attribute_sorting_rows_count".
    //    db.rollback_trans()
    //    assert(new EntityClass(db, mEntityClass.get_id).get_name != tmpName)
    //    assert(new Entity(db, mTemplateEntity.get_id).get_name != tmpName + "-template")
      }

      "update_create_default_attributes" should "work" in {
        assert(mEntityClass.get_create_default_attributes.isEmpty)
        mEntityClass.update_create_default_attributes(Some(true))
        assert(mEntityClass.get_create_default_attributes.get)
        assert(new EntityClass(db, mEntityClass.get_id).get_create_default_attributes.get)
      }
    */
}
// %%%%%
