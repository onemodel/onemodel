/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2004, 2010, 2011, 2013-2017 inclusive, and 2023-2024 inclusive, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>

    (This was originally cloned from RelationToEntity which has some of the above copyright years for its contents.)
*/
use crate::color::Color;
use crate::model::attribute_with_valid_and_observed_dates::AttributeWithValidAndObservedDates;
use crate::model::database::DataType;
use crate::model::database::Database;
use crate::util::Util;
use anyhow::{anyhow, /*Error, */ Result};
// use sqlx::{PgPool, Postgres, Row, Transaction};
use crate::model::attribute::Attribute;
use crate::model::entity::Entity;
// use crate::model::id_wrapper::IdWrapper;
use crate::model::relation_to_entity::RelationToEntity;
use crate::model::relation_type::RelationType;
use sqlx::{Postgres, Transaction};

// ***NOTE***: Similar/identical code found in *_attribute.rs, relation_to_*entity.rs and relation_to_group.rs,
// due to Rust limitations on OO.  Maintain them all similarly.

/// This class exists, instead of just using RelationToEntity, so that the consuming code can be more clear at any given
/// time as to whether RelationToLocalEntity or RelationToRemoteEntity is being used, to avoid subtle bugs.
pub struct RelationToLocalEntity<'a> {
    // For descriptions of the meanings of these variables, see the comments
    // on create_boolean_attribute(...) or create_tables() in PostgreSQLDatabase or Database structs,
    // and/or examples in the database testing code.
    db: Box<&'a dyn Database>,
    id: i64,
    // Unlike most other things that implement Attribute, rel_type_id takes the place of attr_type_id in this, since
    // unlike in the scala code self does not extend Attribute and inherit attr_type_id.
    rel_type_id: i64,
    entity_id1: i64,
    entity_id2: i64,
    valid_on_date: Option<i64>, /*%%= None*/
    observation_date: i64,      /*%%= 0_i64*/
    sorting_index: i64,         /*%%= 0_i64*/
    already_read_data: bool,    /*%%= false*/
}

impl RelationToLocalEntity<'_> {
    /// This one is perhaps only called by the database code--so it can return arrays of objects & save more DB hits
    /// that would have to occur if it only returned arrays of keys. This DOES NOT create a persistent object--but rather should reflect
    /// one that already exists.
    fn new<'a>(
        db: &'a dyn Database,
        id: i64,
        rel_type_id: i64,
        entity_id1: i64,
        entity_id2: i64,
        valid_on_date: Option<i64>,
        observation_date: i64,
        sorting_index: i64,
    ) -> RelationToLocalEntity<'a> {
        RelationToLocalEntity {
            db,
            id,
            rel_type_id,
            entity_id1,
            entity_id2,
            valid_on_date,
            observation_date,
            sorting_index,
            already_read_data: true,
        }
        //    if this.isInstanceOf[RelationToRemoteEntity]) {
        //    %%
        //      //idea: this test & exception feel awkward. What is the better approach?  Maybe using scala's type features?
        //      throw new OmException("This constructor should not be called by the subclass.")
        //    }
    }

    /// This constructor instantiates an existing object from the DB and is rarely needed.
    /// You can use Entity.addRelationTo[Local|Remote]Entity() to create a new persistent record.
    pub fn new2<'a>(
        db: &'a dyn Database,
        transaction: &Option<&mut Transaction<Postgres>>,
        id: i64,
        rel_type_id: i64,
        entity_id1: i64,
        entity_id2: i64,
    ) -> Result<RelationToLocalEntity<'a>, anyhow::Error> {
        // Even a RelationToRemoteEntity can have db.is_remote == true, if it
        // is viewing data *in* a remote OM instance looking at RTLEs that are remote to that remote instance.
        // See comment in similar spot in BooleanAttribute for why not checking for exists, if db.is_remote.
        if !db.is_remote
            && !db.relation_to_local_entity_keys_exist_and_match(
                transaction,
                id,
                rel_type_id,
                entity_id1,
                entity_id2,
            )
        {
            Err(anyhow!(
                "Key id={}, rel_type_id={} and entity_id={} and entity_id_2={}",
                id,
                rel_type_id,
                entity_id1,
                entity_id2 + Util::DOES_NOT_EXIST
            ));
        } else {
            //assign_common_vars(entity_id1, rel_type_id_in, valid_on_date_in, observation_date_in, sorting_index_in)
            Ok(RelationToLocalEntity {
                db,
                id,
                rel_type_id,
                entity_id1,
                entity_id2,
                valid_on_date: None,
                observation_date: 0,
                sorting_index: 0,
                already_read_data: false,
            })
        }
    }

    /// This is for times when you want None if it doesn't exist, instead of the Err Result returned
    /// by the Entity constructor.  Or for convenience in tests.
    pub fn get_relation_to_local_entity<'a>(
        db: &'a dyn Database,
        transaction: &'a Option<&'a mut Transaction<'a, Postgres>>,
        id: i64,
    ) -> Result<Option<RelationToLocalEntity<'a>>, anyhow::Error> {
        let result: Vec<Option<DataType>> = db.get_relation_to_local_entity_data_by_id(id);
        let rel_type_id: i64 = result[0].unwrap();
        let eid1: i64 = result[1].unwrap();
        let eid2: i64 = result[2].unwrap();
        let rtle = RelationToLocalEntity::new2(db, transaction, id, rel_type_id, eid1, eid2);
        match rtle {
            Err(e) => {
                if e.to_string().contains(Util::DOES_NOT_EXIST) {
                    Ok(None)
                } else {
                    Err(e)
                }
            }
            Ok(result) => Ok(result),
        };
    }

    fn get_remote_description() -> String {
        //%%have it throw an err instead? what do callers expect.  (The scala version also had "".)
        "".to_string()
    }

    fn get_entity_for_entity_id2<'a>(
        &self,
        transaction: &'a Option<&'a mut Transaction<'a, Postgres>>,
    ) -> Result<Entity<'a>, anyhow::Error> {
        Entity::new2(self.db, self.entity_id2)
    }

    fn read_data_from_db(
        &mut self,
        transaction: &Option<&mut Transaction<Postgres>>,
    ) -> Result<(), anyhow::Error> {
        let data: Vec<Option<DataType>> = self.db.get_relation_to_local_entity_data(
            transaction,
            self.attr_type_id,
            self.entity_id1,
            self.entity_id2,
        );
        if data.len() == 0 {
            return Err(anyhow!(
                "No results returned from data request for: {}, {}, {}",
                self.attr_type_id,
                self.entity_id1,
                self.entity_id2
            ));
        }

        self.already_read_data = true;

        //assign_common_vars(self.entity_id1, self.attr_type_id, relation_data(2).get.asInstanceOf[i64], relation_data(3).get.asInstanceOf[i64])
        //***ONLY ROUGHLY COPIED***:
        //BEGIN COPIED BLOCK descended from Attribute.assign_common_vars (unclear how to do better for now):
        // No other local variables to assign.  All are either in the superclass or the primary key(s?).
        //except omitting this one since the row keys are already filled in.  The above call to a db query returns id,
        //valid_on_date, observation_date, and sorting_index.  Not already filled in by new2() are the last 3 of those.
        // self.parent_id = match data[1] {
        //     Some(DataType::Bigint(x)) => x,
        //     _ => return Err(anyhow!("How did we get here for {:?}?", data[1])),
        // };
        // except(also) omitting this one, since rel_type_id takes the place of attr_type_id in this, since
        // unlike in the scala code self does not extend Attribute and inherit attr_type_id.
        // self.attr_type_id = match data[2] {
        //     Some(DataType::Bigint(x)) => x,
        //     _ => return Err(anyhow!("How did we get here for {:?}?", data[2])),
        // };
        self.sorting_index = match data[3] {
            Some(DataType::Bigint(x)) => x,
            _ => return Err(anyhow!("How did we get here for {:?}?", data[6])),
        };
        //END COPIED BLOCK descended from Attribute.assign_common_vars (might be in comment in boolean_attribute.rs)

        //***ONLY ROUGHLY COPIED***:
        //BEGIN COPIED BLOCK descended from AttributeWithValidAndObservedDates.assign_common_vars (unclear how to do better):
        //%%%%% fix this next part after figuring out about what happens when querying a null back, in pg.db_query etc!
        // valid_on_date: Option<i64> /*%%= None*/,
        /*DataType::Bigint(%%)*/
        self.valid_on_date = None; //data[1];
                                   // self.valid_on_date = match data[4] {
                                   //     DataType::Bigint(x) => x,
                                   //     _ => return Err(anyhow!("How did we get here for {:?}?", data[4])),
                                   // };

        self.observation_date = match data[2] {
            Some(DataType::Bigint(x)) => x,
            _ => return Err(anyhow!("How did we get here for {:?}?", data[5])),
        };
        //END COPIED BLOCK descended from AttributeWithValidAndObservedDates.assign_common_vars.

        Ok(())
    }

    fn move_it(
        &self,
        to_local_containing_entity_id_in: i64,
        sorting_index_in: i64,
    ) -> RelationToLocalEntity {
        self.db.move_relation_to_local_entity_into_local_entity(
            self.get_id(),
            to_local_containing_entity_id_in,
            sorting_index_in,
        )
    }

    fn move_entity_from_entity_to_group(&self, target_group_id_in: i64, sorting_index_in: i64) {
        self.db.move_local_entity_from_local_entity_to_group(
            &self,
            target_group_id_in,
            sorting_index_in,
        )
    }

    fn update(
        &self,
        valid_on_date_in: Option<i64>,
        observation_date_in: Option<i64>,
        new_attr_type_id_in: Option<i64>, /*= None*/
    ) {
        let new_attr_type_id = new_attr_type_id_in.unwrap_or(self.get_attr_type_id());
        //Using valid_on_date_in rather than valid_on_date_in.unwrap(), just below,
        //because valid_on_date allows None, unlike others (od).
        //(Idea/possible bug: the way this is written might mean one can never change vod to None
        //from something else: could ck callers & expectations
        // & how to be most clear (could be the same in RelationToGroup & other (former) Attribute subclasses).)
        let vod = if valid_on_date_in.is_some() {
            valid_on_date_in
        } else {
            self.get_valid_on_date();
        };
        let od = if observation_date_in.is_some() {
            observation_date_in.unwrap()
        } else {
            self.get_observation_date();
        };
        self.db.update_relation_to_local_entity(
            self.attr_type_id,
            self.entity_id1,
            self.entity_id2,
            new_attr_type_id,
            vod,
            od,
        );
        self.valid_on_date = vod;
        self.observation_date = od;
        self.attr_type_id = new_attr_type_id;
    }

    /// Removes this object from the system.
    fn delete(&self, transaction: &Option<&mut Transaction<Postgres>>) {
        self.db.delete_relation_to_local_entity(
            self.get_attr_type_id(),
            self.entity_id1,
            self.entity_id2,
        )
    }
}

//BEGIN SIMILAR CODE: MAINTAIN THIS LIKE CODE FOUND IN relation_to_remote_entity.rs ! --------------------
impl RelationToEntity for RelationToLocalEntity<'_> {
    fn get_related_id1(&self) -> i64 {
        self.entity_id1
    }
    fn get_related_id2(&self) -> i64 {
        self.entity_id2
    }

    /// See comments on namesake in the trait.
    fn get_display_string(
        &self,
        length_limit_in: usize,
        related_entity_in: Option<Entity>,
        relation_type_in: Option<RelationType>,
        simplify: bool, /*= false*/
    ) -> Result<String, anyhow::Error> {
        let rel_type: RelationType = {
            match relation_type_in {
                Some(rt) => {
                    if rt.get_id() != self.get_attr_type_id() {
                        // It can be ignored, but in cases called generically (the same as other Attribute types) it should have the right value or that indicates a
                        // misunderstanding in the caller's code. Also, if passed in and this were changed to use it again, it can save processing time re-instantiating one.
                        return Err(anyhow!("inRT parameter should be the same as the relationType on this relation."));
                    }
                    rt
                }
                _ => RelationType::new2(self.db, self.get_attr_type_id()),
            }
        };
        //   *****  MAKE SURE  ***** that during maintenance, anything that gets data relating to entity_id2 is using the right (local or remote) db!:
        let related_entity: Entity =
            { related_entity_in.unwrap_or_else(|| self.get_entity_for_entity_id2()) };
        let rt_name: String = {
            if related_entity.get_id() == self.entity_id2 {
                rel_type.get_name()
            } else if related_entity.get_id() == self.entity_id1 {
                rel_type.get_name_in_reverse_direction
            } else {
                return Err(anyhow!(
                    "Unrelated parent entity parameter?: '{}', '{}'",
                    related_entity.get_id(),
                    related_entity.get_name()
                ));
            }
        };
        // (See method comment about the related_entity_in param.)
        let result: String = if simplify {
            if rt_name == Util::THE_HAS_RELATION_TYPE_NAME {
                related_entity.get_name()
            } else {
                format!(
                    "{}{}: {}",
                    rt_name,
                    self.get_remote_description(),
                    related_entity.get_name()
                )
            }
        } else {
            format!(
                "{}{}: {}; {}",
                rt_name,
                self.get_remote_description(),
                Color::blue(related_entity.get_name()),
                self.get_dates_description()
            )
        };

        //    if this.isInstanceOf[RelationToRemoteEntity]) {
        //      result = "[remote]" + result
        //    }
        Ok(Util::limit_attribute_description_length(
            result,
            length_limit_in,
        ))
    }

    //%%?: fn get_remote_description -> String

    // If related_entity_in is an RTRE, could be a different db so build accordingly:
    //%%?: fn get_entity_for_entity_id2 -> Entity

    // (the next line used to be coded so instead of working it would return an exception, like this:
    //     throw new UnsupportedOperationException("getParentId() operation not applicable to Relation class.")
    // ..., and I'm not sure of the reason: if it was just to prevent accidental misuse or confusion (probably), it seems OK
    // to have it be like this instead, for convenience:
    fn get_parent_id(&self) -> i64 {
        self.get_related_id1()
    }
}
//END SIMILAR CODE--------------------

impl Attribute for RelationToLocalEntity<'_> {
    //%% see ex in rtg
    /// Removes this object from the system.
%%
    // This datum is provided upon construction (new2(), at minimum), so can be returned
    // regardless of already_read_data / read_data_from_db().
    fn get_id(&self) -> i64 {
        self.id
    }

    fn get_form_id(&self) -> Result<i32, Error> {
        self.db.get_attribute_form_id(Util::RELATION_TO_LOCAL_ENTITY_TYPE)
    }

    fn get_attr_type_id(
        &mut self,
        transaction: &Option<&mut Transaction<Postgres>>,
    ) -> Result<i64, anyhow::Error> {
        if !self.already_read_data {
            self.read_data_from_db(transaction)?;
        }
        Ok(self.rel_type_id)
    }

    fn get_sorting_index(
        &mut self,
        transaction: &Option<&mut Transaction<Postgres>>,
    ) -> Result<i64, anyhow::Error> {
        if !self.already_read_data {
            self.read_data_from_db(transaction)?;
        }
        Ok(self.sorting_index)
    }
%%
    fn get_parent_id(
        &mut self,
        transaction: &Option<&mut Transaction<Postgres>>,
    ) -> Result<i64, anyhow::Error> {
        if !self.already_read_data {
            self.read_data_from_db(transaction)?;
        }
        Ok(self.entity_id)
    }
}

impl AttributeWithValidAndObservedDates for RelationToLocalEntity<'_> {
    //%% see ex in rtg
}

#[cfg(test)]
mod test {
    //%%
}
