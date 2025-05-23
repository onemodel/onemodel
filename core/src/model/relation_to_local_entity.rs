/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2004, 2010, 2011, 2013-2017 inclusive, and 2023-2025 inclusive, Luke A. Call.
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
use std::cell::RefCell;
use std::rc::Rc;

// ***NOTE***: Similar/identical code found in *_attribute.rs, relation_to_*entity.rs and relation_to_group.rs,
// due to Rust limitations on OO.  Maintain them all similarly.

/// This class exists, instead of just using RelationToEntity, so that the consuming code can be more clear at any given
/// time as to whether RelationToLocalEntity or RelationToRemoteEntity is being used, to avoid subtle bugs.
pub struct RelationToLocalEntity {
    // For descriptions of the meanings of these variables, see the comments
    // on create_boolean_attribute(...) or create_tables() in PostgreSQLDatabase or Database structs,
    // and/or examples in the database testing code.
    db: Rc<RefCell<dyn Database>>,
    id: i64,
    // Unlike most other things that implement Attribute, rel_type_id takes the place of attr_type_id in this, since
    // unlike in the scala code self does not extend Attribute and inherit attr_type_id.
    rel_type_id: i64,
    entity_id1: i64,
    entity_id2: i64,
    valid_on_date: Option<i64>, /*= None*/
    observation_date: i64,      /*= 0_i64*/
    sorting_index: i64,         /*= 0_i64*/
    already_read_data: bool,    /*= false*/
}

impl RelationToLocalEntity {
    /// This one is perhaps only called by the database code [or code that just hit the db]--so it can return
    /// arrays of objects & save more DB hits
    /// that would have to occur if it only returned arrays of keys. This DOES NOT create a persistent
    /// object--but rather should reflect one that already exists.
    pub fn new(
        db: Rc<RefCell<dyn Database>>,
        id: i64,
        rel_type_id: i64,
        entity_id1: i64,
        entity_id2: i64,
        valid_on_date: Option<i64>,
        observation_date: i64,
        sorting_index: i64,
    ) -> RelationToLocalEntity {
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
        //    %%latercheck
        //      //idea: this test & exception feel awkward. What is the better approach?  Maybe using Rust's type features?
        //      throw new OmException("This constructor should not be called by the subclass.")
        //    }
    }

    /// This constructor instantiates an existing object from the DB and is rarely needed.
    /// You can use Entity.addRelationTo[Local|Remote]Entity() to create a new persistent record.
    pub fn new2(
        db: Rc<RefCell<dyn Database>>,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        id: i64,
        rel_type_id: i64,
        entity_id1: i64,
        entity_id2: i64,
    ) -> Result<RelationToLocalEntity, anyhow::Error> {
        // Even a RelationToRemoteEntity can have db.is_remote == true, if it
        // is viewing data *in* a remote OM instance looking at RTLEs that are remote to that remote instance.
        // See comment in similar spot in BooleanAttribute for why not checking for exists, if db.is_remote.
        if !db.borrow().is_remote()
            && !db.borrow().relation_to_local_entity_keys_exist_and_match(
                transaction,
                id,
                rel_type_id,
                entity_id1,
                entity_id2,
            )?
        {
            Err(anyhow!(
                "Key id={}, rel_type_id={} and entity_id1={} and entity_id_2={} {}",
                id,
                rel_type_id,
                entity_id1,
                entity_id2,
                Util::DOES_NOT_EXIST
            ))
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
    /// Was called "get_relation_to_local_entity" but that was harder to remember.
    pub fn new3<'a, 'b>(
        db: Rc<RefCell<dyn Database>>,
        transaction: Option<Rc<RefCell<Transaction<'b, Postgres>>>>,
        id: i64,
    ) -> Result<Option<RelationToLocalEntity>, anyhow::Error> 
    where
        'a: 'b
    {
        let result: Vec<Option<DataType>> =
            db.borrow().get_relation_to_local_entity_data_by_id(transaction.clone(), id)?;
        let Some(DataType::Bigint(rel_type_id)) = result[0] else {
            return Err(anyhow!("Unexpected result: {:?}", result));
        };
        let Some(DataType::Bigint(eid1)) = result[1] else {
            return Err(anyhow!("Unexpected result: {:?}", result));
        };
        let Some(DataType::Bigint(eid2)) = result[2] else {
            return Err(anyhow!("Unexpected result: {:?}", result));
        };
        let rtle = RelationToLocalEntity::new2(db, transaction, id, rel_type_id, eid1, eid2);
        match rtle {
            Err(e) => {
                if e.to_string().contains(Util::DOES_NOT_EXIST) {
                    Ok(None)
                } else {
                    Err(e)
                }
            }
            Ok(r) => Ok(Some(r)),
        }
    }

    /// @return the id and sorting_index of the newly moved RTLE.
    fn move_it(
        &self,
        to_local_containing_entity_id_in: i64,
        sorting_index_in: i64,
    ) -> Result<(i64, i64), anyhow::Error> {
        self.db.borrow().move_relation_to_local_entity_into_local_entity(
            self.get_id(),
            to_local_containing_entity_id_in,
            sorting_index_in,
        )
    }

    fn move_entity_from_entity_to_group(
        &mut self,
        target_group_id_in: i64,
        sorting_index_in: i64,
    ) -> Result<(), anyhow::Error> {
        self.db.clone().borrow().move_local_entity_from_local_entity_to_group(
            self,
            target_group_id_in,
            sorting_index_in,
        )
    }

    fn update(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
        valid_on_date_in: Option<i64>,
        observation_date_in: Option<i64>,
        new_attr_type_id_in: Option<i64>, /*= None*/
    ) -> Result<(), anyhow::Error> {
        let new_attr_type_id = new_attr_type_id_in.unwrap_or(self.get_attr_type_id(None)?);
        //Using valid_on_date_in rather than valid_on_date_in.unwrap(), just below,
        //because valid_on_date allows None, unlike others (od).
        //(Idea/possible bug: the way this is written might mean one can never change vod to None
        //from something else: could ck callers & expectations
        // & how to be most clear (could be the same in RelationToGroup & other (former) Attribute subclasses).)
        let vod = if valid_on_date_in.is_some() {
            valid_on_date_in
        } else {
            self.get_valid_on_date(transaction.clone())?
        };
        let od = if observation_date_in.is_some() {
            observation_date_in.unwrap()
        } else {
            self.get_observation_date(transaction.clone())?
        };
        self.db.borrow().update_relation_to_local_entity(
            transaction,
            self.rel_type_id,
            self.entity_id1,
            self.entity_id2,
            new_attr_type_id,
            vod,
            od,
        )?;
        self.valid_on_date = vod;
        self.observation_date = od;
        self.rel_type_id = new_attr_type_id;
        Ok(())
    }
}

//BEGIN SIMILAR CODE: MAINTAIN THIS LIKE CODE FOUND IN relation_to_remote_entity.rs ! --------------------
impl RelationToEntity for RelationToLocalEntity {
    fn get_related_id1(&self) -> i64 {
        self.entity_id1
    }
    fn get_related_id2(&self) -> i64 {
        self.entity_id2
    }

    // If related_entity_in is an RTRE, could be a different db so build accordingly:
    fn get_remote_description(&self) -> String {
        //%%later: have it throw an err instead? what do callers expect.  (The scala version also had "".)
        "".to_string()
    }

    fn get_entity_for_entity_id2(
        &self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<Entity, anyhow::Error> 
    {
        Entity::new2(self.db.clone(), transaction, self.entity_id2)
    }
}
//END SIMILAR CODE--------------------

impl Attribute for RelationToLocalEntity {
    // (The next line used to be coded so instead of working it would return an exception, like this:
    //     throw new UnsupportedOperationException("getParentId() operation not applicable to Relation class.")
    // ..., and I'm not sure of the reason: if it was just to prevent accidental misuse or confusion (probably), it seems OK
    // to have it be like this instead, for convenience:
    fn get_parent_id(
        &mut self,
        _transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<i64, anyhow::Error> {
        Ok(self.get_related_id1())
    }

    /// Removes this object from the system.
    fn delete<'a>(
        &'a self,
        transaction: Option<Rc<RefCell<Transaction<'a, Postgres>>>>,
    ) -> Result<u64, anyhow::Error> {
        self.db.borrow().delete_relation_to_local_entity(
            transaction,
            self.rel_type_id,
            self.entity_id1,
            self.entity_id2,
        )
    }

    fn read_data_from_db(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<(), anyhow::Error> {
        let data: Vec<Option<DataType>> = self.db.borrow().get_relation_to_local_entity_data(
            transaction,
            self.rel_type_id,
            self.entity_id1,
            self.entity_id2,
        )?;
        if data.len() == 0 {
            return Err(anyhow!(
                "No results returned from data request for: {}, {}, {}",
                self.rel_type_id,
                self.entity_id1,
                self.entity_id2
            ));
        }

        self.id = match data[0] {
            Some(DataType::Bigint(x)) => x,
            _ => return Err(anyhow!("How did we get here for {:?}?", data[0])),
        };
        //assign_common_vars(self.entity_id1, self.attr_type_id, relation_data(2).get.asInstanceOf[i64],
        //relation_data(3).get.asInstanceOf[i64])
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
            _ => return Err(anyhow!("How did we get here for {:?}?", data[3])),
        };
        //END COPIED BLOCK descended from Attribute.assign_common_vars (might be in comment in boolean_attribute.rs)

        //***ONLY ROUGHLY COPIED***:
        //BEGIN COPIED BLOCK descended from AttributeWithValidAndObservedDates.assign_common_vars (unclear how to do better):
        self.valid_on_date = match data[1] {
            Some(DataType::Bigint(x)) => Some(x),
            None => None,
            _ => return Err(anyhow!("How did we get here for {:?}?", data[1])),
        };
        self.observation_date = match data[2] {
            Some(DataType::Bigint(x)) => x,
            _ => return Err(anyhow!("How did we get here for {:?}?", data[2])),
        };
        //END COPIED BLOCK descended from AttributeWithValidAndObservedDates.assign_common_vars.

        self.already_read_data = true;
        Ok(())
    }

    /// @param related_entity_in, could be either entity_id2 or 1: it is always *not* the entity
    /// from whose perspective the result will be returned, ex.,
    /// 'x contains y' OR 'y is contained by x': the 2nd parameter should be the *2nd* one in that statement.
    /// If left None here, the code will make a guess but might output confusing (backwards) info.
    /// @param relation_type_in can be left None, but will run faster if not.
    /// @return something like "son of: Paul" or "owns: Ford truck" or "employed by: hospital".
    /// If in_length_limit is 0 you get the whole thing.
    fn get_display_string(
        &mut self,
        length_limit_in: usize,
        related_entity_in: Option<Entity>,
        relation_type_in: Option<RelationType>,
        simplify: bool, /*= false*/
    ) -> Result<String, anyhow::Error> {
        let mut rel_type: RelationType = {
            match relation_type_in {
                Some(rt) => {
                    if rt.get_id() != self.get_attr_type_id(None)? {
                        // It can be ignored, but in cases called generically (the same as other Attribute types)
                        // it should have the right value or that indicates a
                        // misunderstanding in the caller's code. Also, if passed in and this were changed to use
                        // it again, it can save processing time re-instantiating one.
                        return Err(anyhow!("inRT parameter should be the same as the relationType on this relation."));
                    }
                    rt
                }
                _ => RelationType::new2(self.db.clone(), None, self.get_attr_type_id(None)?)?,
            }
        };
        //   *****MAKE SURE***** that during maintenance, anything that gets data relating to entity_id2
        //   is using the right (local or remote) db!:
        let mut related_entity: Entity = match related_entity_in {
            Some(e) => e,
            None => self.get_entity_for_entity_id2(None)?,
        };
        let rt_name: String = {
            if related_entity.get_id() == self.entity_id2 {
                rel_type.get_name(None)?
            } else if related_entity.get_id() == self.entity_id1 {
                rel_type.get_name_in_reverse_direction(None)?
            } else {
                return Err(anyhow!(
                    "Unrelated parent entity parameter?: '{}', '{}'",
                    related_entity.get_id(),
                    related_entity.get_name(None)?
                ));
            }
        };
        // (See method comment about the related_entity_in param.)
        let result: String = if simplify {
            if rt_name == Util::THE_HAS_RELATION_TYPE_NAME {
                related_entity.get_name(None)?.clone()
            } else {
                format!(
                    "{}{}: {}",
                    rt_name,
                    self.get_remote_description(),
                    related_entity.get_name(None)?
                )
            }
        } else {
            format!(
                "{}{}: {}; {}",
                rt_name,
                self.get_remote_description(),
                Color::blue(related_entity.get_name(None)?),
                Util::get_dates_description(self.get_valid_on_date(None)?, self.get_observation_date(None)?)
            )
        };

        //    if this.isInstanceOf[RelationToRemoteEntity]) {
        //      result = "[remote]" + result
        //    }
        Ok(Util::limit_attribute_description_length(
            result.as_str(),
            length_limit_in,
        ))
    }

    // This datum is provided upon construction (new2(), at minimum), so can be returned
    // regardless of already_read_data / read_data_from_db().
    fn get_id(&self) -> i64 {
        self.id
    }

    fn get_form_id(&self) -> Result<i32, anyhow::Error> {
        self.db.borrow()
            .get_attribute_form_id(Util::RELATION_TO_LOCAL_ENTITY_TYPE)
    }

    fn get_attr_type_id(
        &mut self,
        _transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<i64, anyhow::Error> {
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
}

impl AttributeWithValidAndObservedDates for RelationToLocalEntity {
    //%%later: Can these be impl in the trait only, instead of here/all children?
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
    //%%latertests
}
