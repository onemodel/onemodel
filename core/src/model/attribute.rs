/* . This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2004, 2010, 2011, 2013-2017 inclusive, and 2023, Luke A Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
use anyhow::{anyhow, Result};
// use chrono::LocalResult;
// use chrono::prelude::*;
// use crate::util::Util;
use crate::model::entity::Entity;
use crate::model::id_wrapper::IdWrapper;
use crate::model::relation_type::RelationType;

/// Represents one attribute object in the system (usually [always, as of 1/2004] used as an attribute on a Entity).
/// Originally created as a place to put common stuff between Relation/QuantityAttribute/TextAttribute.
pub trait Attribute {
    // Idea: somehow use language features better to make it cleaner, so we don't need these extra 2 vars, because they are
    // used in 1-2 instances, and ignored in the rest.  One thing is that RelationTo[Local|Remote]Entity and RelationToGroup
    // are Attributes. Should they be?

    fn get_display_string(
        in_length_limit: i32,
        parent_entity: Option<Entity>,
        in_rt_id: Option<RelationType>,
        simplify: bool, /* = false*/
    ) -> String;

    fn read_data_from_db();

    fn delete();

    fn get_id_wrapper() -> IdWrapper;
    // was:
    // fn get_id_wrapper -> IdWrapper {
    //   new IdWrapper(m_id)
    // }

    fn get_id() -> i64;
    // was:
    // fn get_id -> i64 {
    //     m_id
    // }

    fn get_form_id(&self) -> Result<i32, anyhow::Error>;
    // was:
    // fn get_form_id -> Int {
    //     Database.get_attribute_form_id(this.getClass.getSimpleName)
    // }

    fn assign_common_vars(parent_id_in: i64, attr_type_id_in: i64, sorting_index_in: i64);
    //was:
    // protected fn assign_common_vars(parent_id_in: i64, attr_type_id_in: i64, sorting_index_in: i64) {
    //   m_parent_id = parent_id_in
    //   m_attr_type_id = attr_type_id_in
    //   m_sorting_index = sorting_index_in
    //   m_already_read_data = true
    // }

    fn get_attr_type_id() -> i64;
    // was:
    // fn get_attr_type_id() -> i64 {
    //   if !m_already_read_data) read_data_from_db()
    //   m_attr_type_id
    // }

    fn get_sorting_index() -> i64;
    // was:
    //   fn get_sorting_index -> i64 {
    //   if !m_already_read_data) read_data_from_db()
    //   m_sorting_index
    // }

    fn get_parent_id() -> i64;
    // was:
    // fn get_parent_id() -> i64 {
    //   if !m_already_read_data) read_data_from_db()
    //   m_parent_id
    // }

    // For descriptions of the meanings of these variables, see the comments
    // on create_tables(...), and examples in the database testing code &/or in PostgreSQLDatabase or Database classes.
    // %%put these in the structs implementing this trait, along w/ those above methods!
    //m_db: Database;
    //m_id: i64;
    // protected let mut m_parent_id: i64 = 0L;
    // protected let mut m_attr_type_id: i64 = 0L;
    // protected let mut m_already_read_data: bool = false;
    // protected let mut m_sorting_index: i64 = 0L;
}
