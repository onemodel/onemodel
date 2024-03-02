/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2004, 2010, 2011, 2013-2017 inclusive, and 2023, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
// use std::os::unix::process::parent_id;
//use crate::model::attribute_with_valid_and_observed_dates::AttributeWithValidAndObservedDates;
//use crate::model::database::Database;
//use crate::util::Util;
//use anyhow::{anyhow, Error, Result};
// use sqlx::{PgPool, Postgres, Row, Transaction};
//use crate::model::attribute::Attribute;
use crate::model::entity::Entity;
// use crate::model::id_wrapper::IdWrapper;
use crate::model::relation_type::RelationType;
//use sqlx::{Postgres, Transaction};
//use tracing_subscriber::registry::Data;

// ***NOTE***: Similar/identical code found in *_attribute.rs, relation_to_entity.rs and relation_to_group.rs,
// due to Rust limitations on OO.  Maintain them all similarly.

/// Represents one RelationToEntity object in the system (usually [always, as of 9/2003] used as an attribute on a Entity).
/// You can use Entity.addRelationTo[Local|Remote]Entity() to create a new object.
pub trait RelationToEntity {
    //    //%%not needed right? would be called directly on the subclass rtle or rtre.
    //    fn new<'a>(
    //        db: Box<&'a dyn Database>,
    //        id: i64,
    //        rel_type_id: i64,
    //        entity_id1: i64,
    //        entity_id2: i64,
    //    ) -> RelationToEntity<'a> {
    //        RelationToEntity {
    //            db,
    //            id,
    //            rel_type_id,
    //            entity_id1,
    //            entity_id2,
    //            already_read_data: false,
    //            valid_on_date: None,
    //            observation_date: 0,
    //            sorting_index: 0,
    //        }
    //    }

    fn get_related_id1(&self) -> i64;
    fn get_related_id2(&self) -> i64;

    /// @param related_entity_in, could be either entity_id2 or 1: it is always *not* the entity from whose perspective the result will be returned, ex.,
    /// 'x contains y' OR 'y is contained by x': the 2nd parameter should be the *2nd* one in that statement.
    /// If left None here, the code will make a guess but might output confusing (backwards) info.
    /// @param relation_type_in can be left None, but will run faster if not.
    /// @return something like "son of: Paul" or "owns: Ford truck" or "employed by: hospital". If in_length_limit is 0 you get the whole thing.
    fn get_display_string(
        &self,
        length_limit_in: usize,
        related_entity_in: Option<Entity>,
        relation_type_in: Option<RelationType>,
        simplify: bool, /*= false*/
    ) -> String;

    //%%?: fn get_remote_description -> String

    // If related_entity_in is an RTRE, could be a different db so build accordingly:
    //%%?: fn get_entity_for_entity_id2 -> Entity

    fn get_parent_id(&self) -> i64;
}
