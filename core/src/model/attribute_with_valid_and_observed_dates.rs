/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2014, 2016-2017 inclusive and 2023, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
use crate::model::attribute::Attribute;
// use crate::util::Util;
use sqlx::{Postgres, Transaction};
use std::cell::{RefCell};
use std::rc::Rc;

// (For more info see "supertraits" in The Book (or in anki notes).)
pub trait AttributeWithValidAndObservedDates: Attribute {
    //%%
    //was:
    // fn assign_common_vars(
    //     parent_id_in: i64,
    //     attr_type_id_in: i64,
    //     valid_on_date_in: Option<i64>,
    //     observation_date_in: i64,
    //     sorting_index_in: i64,
    // );

    // was:
    //  fn assign_common_vars(parent_id_in: i64, attr_type_id_in: i64, valid_on_date_in: Option<i64>, observation_date_in: i64, sorting_index_in: i64) {
    //    valid_on_date = valid_on_date_in
    //    // observation_date is not expected to be None, like valid_on_date can be. See let mut def for more info.;
    //    observation_date = observation_date_in
    //    super.assign_common_vars(parent_id_in, attr_type_id_in, sorting_index_in)
    //  }

    // just call Util directly from callers, instead?:  Except when called for FA??--dift.
    // fn get_dates_description(valid_on_date: Option<i64>, observation_date: i64) -> String {
    //     Util::get_dates_description(valid_on_date, observation_date)
    //   }

    fn get_valid_on_date(&mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<Option<i64>, anyhow::Error>;

    fn get_observation_date(
        &mut self,
        transaction: Option<Rc<RefCell<Transaction<Postgres>>>>,
    ) -> Result<i64, anyhow::Error>;

    // For descriptions of the meanings of these variables, see the comments
    // on create_tables(...), and examples in the database testing code in PostgreSQLDatabase or Database classes.
    // %%put these in the structs implementing this trait, along w/ those above methods!
    // protected let mut valid_on_date: Option<i64> = None;
    // protected let mut observation_date: i64 = 0L;
}
