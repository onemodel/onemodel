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
use crate::model::database::Database;
//use crate::util::Util;
//use anyhow::{anyhow, Error, Result};
// use sqlx::{PgPool, Postgres, Row, Transaction};
//use crate::model::attribute::Attribute;
//use crate::model::entity::Entity;
// use crate::model::id_wrapper::IdWrapper;
//use crate::model::relation_type::RelationType;
//use sqlx::{Postgres, Transaction};
//use tracing_subscriber::registry::Data;

// ***NOTE***: Similar/identical code found in *_attribute.rs, relation_to_entity.rs and relation_to_group.rs,
// due to Rust limitations on OO.  Maintain them all similarly.

/// Represents one RelationToEntity object in the system (usually [always, as of 9/2003] used as an attribute on a Entity).
/// You can use Entity.addRelationTo[Local|Remote]Entity() to create a new object.
///
/// The intent of this being "abstract protected..." in scala at least,
/// was to help make it so the class is only visible to (or at least only used by) its subclasses, so
/// that everywhere else has to specify whether the usage is a RelationToLocalEntity or RelationToRemoteEntity.
pub struct RelationToEntity<'a> {
    // For descriptions of the meanings of these variables, see the comments
    // on create_quantity_attribute(...) or create_tables() in PostgreSQLDatabase or Database structs,
    // and/or examples in the database testing code.
    db: Box<&'a dyn Database>,
    id: i64,
    // Unlike most other things that implement Attribute, rel_type_id takes the place of attr_type_id in this, since
    // unlike in the scala code self does not extend Attribute and inherit attr_type_id.
    rel_type_id: i64,
    entity_id1: i64,
    entity_id2: i64,
    already_read_data: bool,    /*%%= false*/
    valid_on_date: Option<i64>, /*%%= None*/
    observation_date: i64,      /*%%= 0_i64*/
    sorting_index: i64,         /*%%= 0_i64*/
}

impl RelationToEntity<'_> {
    fn new<'a>(
        db: Box<&'a dyn Database>,
        id: i64,
        rel_type_id: i64,
        entity_id1: i64,
        entity_id2: i64,
    ) -> RelationToEntity<'a> {
        RelationToEntity {
            db,
            id,
            rel_type_id,
            entity_id1,
            entity_id2,
            already_read_data: false,
            valid_on_date: None,
            observation_date: 0,
            sorting_index: 0,
        }
    }

    /*%%%%%%
        fn get_related_id1(&self) -> i64 {
            self.entity_id1
        }
        fn get_related_id2(&self) -> i64 {
            self.entity_id2
        }

      /**
       * @param relatedEntityIn, could be either mEntityId2 or 1: it is always *not* the entity from whose perspective the result will be returned, ex.,
       * 'x contains y' OR 'y is contained by x': the 2nd parameter should be the *2nd* one in that statement.
       * If left None here, the code will make a guess but might output confusing (backwards) info.
       *
       * @param relationTypeIn can be left None, but will run faster if not.
       *
       * @return something like "son of: Paul" or "owns: Ford truck" or "employed by: hospital". If in_length_limit is 0 you get the whole thing.
       */
        fn get_display_string(length_limit_in: Int, relatedEntityIn: Option<Entity>, relationTypeIn: Option[RelationType], simplify: bool = false) -> String {
        let relType: RelationType = {
          if relationTypeIn.is_some()) {
            if relationTypeIn.get.get_id != get_attr_type_id()) {
              // It can be ignored, but in cases called generically (the same as other Attribute types) it should have the right value or that indicates a
              // misunderstanding in the caller's code. Also, if passed in and this were changed to use it again, it can save processing time re-instantiating one.
              throw new OmException("inRT parameter should be the same as the relationType on this relation.")
            }
            relationTypeIn.get
          } else {
            new RelationType(db, get_attr_type_id())
          }
        }
        //   *****  MAKE SURE  ***** that during maintenance, anything that gets data relating to mEntityId2 is using the right (remote) db!:
        let relatedEntity: Entity = {;
          relatedEntityIn.getOrElse(getEntityForEntityId2)
        }
        let rt_name: String = {
          if relatedEntity.get_id == mEntityId2) {
            relType.get_name
          } else if relatedEntity.get_id == mEntityId1) {
            relType.get_name_in_reverse_direction
          } else {
            throw new OmException("Unrelated parent entity parameter?: '" + relatedEntity.get_id + "', '" + relatedEntity.get_name + "'")
          }
        }

        // (See method comment about the relatedEntityIn param.)
        let result: String =;
          if simplify) {
            if rt_name == Database.THE_HAS_RELATION_TYPE_NAME) relatedEntity.get_name
            else rt_name + getRemoteDescription + ": " + relatedEntity.get_name
          } else {
            rt_name + getRemoteDescription + ": " + Color.blue(relatedEntity.get_name) + "; " + get_dates_description
          }

    //    if this.isInstanceOf[RelationToRemoteEntity]) {
    //      result = "[remote]" + result
    //    }
        Attribute.limit_attribute_description_length(result, length_limit_in)
      }

        //%%?: fn getRemoteDescription -> String

      // If relatedEntityIn is an RTRE, could be a different db so build accordingly:
        //%%?: fn getEntityForEntityId2 -> Entity

    // (the next line used to be coded so instead of working it would return an exception, like this:
    //     throw new UnsupportedOperationException("getParentId() operation not applicable to Relation class.")
    // ..., and I'm not sure of the reason: if it was just to prevent accidental misuse or confusion (probably), it seems OK
    // to have it be like this instead, for convenience:
    override fn get_parent_id() -> i64 {
        get_related_id1
    }
    %%%%%%*/
}
