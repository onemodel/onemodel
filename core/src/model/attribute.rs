/* . This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2004, 2010, 2011, 2013-2017 inclusive, and 2023, Luke A Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
// use anyhow::{anyhow, Result};
// use chrono::LocalResult;
// use chrono::prelude::*;
// use crate::util::Util;
use crate::model::entity::Entity;
// use crate::model::id_wrapper::IdWrapper;
use crate::model::relation_type::RelationType;
use sqlx::{Postgres, Transaction};
use std::cell::{RefCell};
use std::rc::Rc;

/// Represents one attribute object in the system (usually [always, as of 1/2004] used as an attribute on a Entity).
/// Originally created as a place to put common stuff between Relation/QuantityAttribute/TextAttribute.
pub trait Attribute {
    //%%?:
    // Idea: somehow use language features better to make it cleaner, so we don't need these extra 2 vars, because they are
    // used in 1-2 instances, and ignored in the rest.  One thing is that RelationTo[Local|Remote]Entity and RelationToGroup
    // are Attributes. Should they be?

    fn get_display_string(
        &mut self,
        in_length_limit: usize,
        parent_entity: Option<Entity>,
        in_rt_id: Option<RelationType>,
        simplify: bool, /* = false*/
    ) -> Result<String, anyhow::Error>;

    fn read_data_from_db(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<(), anyhow::Error>;

    fn delete<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
    ) -> Result<u64, anyhow::Error>;

    //looks unused except for Entity and EntityClass
    // fn get_id_wrapper(&self) -> IdWrapper;

    fn get_id(&self) -> i64;

    fn get_form_id(&self) -> Result<i32, anyhow::Error>;

    // fn assign_common_vars(parent_id_in: i64, attr_type_id_in: i64, sorting_index_in: i64);

    fn get_attr_type_id(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<i64, anyhow::Error>;

    fn get_sorting_index(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<i64, anyhow::Error>;

    fn get_parent_id(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<i64, anyhow::Error>;


    // For descriptions of the meanings of these variables, see the comments
    // on create_tables(...), and examples in the database testing code &/or in PostgreSQLDatabase or Database classes.
    // %%put these in the structs implementing this trait, along w/ those above methods!
    //db: Database;
    //id: i64;
    // protected let mut parent_id: i64 = 0L;
    // protected let mut attr_type_id: i64 = 0L;
    // protected let mut already_read_data: bool = false;
    // protected let mut sorting_index: i64 = 0L;
}
