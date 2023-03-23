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

// pub trait AttributeWithValidAndObservedDates {
pub struct AttributeWithValidAndObservedDates {}

// }

impl AttributeWithValidAndObservedDates {
  pub fn get_dates_description(valid_on_date:Option<i64>, observation_date:i64) -> String {
    let valid_date_descr: String = {
      match valid_on_date {
        None => "unsp'd".to_string(),
        Some(date) if date == 0 => "all time".to_string(),
        Some(date) => Attribute::useful_date_format(date),
      }
    };
    let observed_date_descr: String = Attribute::useful_date_format(observation_date);
    format!("valid {}, obsv'd {}", valid_date_descr, observed_date_descr)
  }

/*%%
abstract class AttributeWithValidAndObservedDates(m_db: Database, m_id: i64) extends Attribute(m_db, m_id) {
  protected fn assignCommonVars(parent_id_in: i64, attr_type_id_in: i64, valid_on_date_in: Option<i64>, observation_date_in: i64, sorting_index_in: i64) {
    valid_on_date = valid_on_date_in
    // observationDate is not expected to be None, like valid_on_date can be. See let mut def for more info.;
    observation_date = observation_date_in
    super.assignCommonVars(parent_id_in, attr_type_id_in, sorting_index_in)
  }

    fn get_dates_description -> String {
    AttributeWithValidAndObservedDates.get_dates_description(get_valid_on_date(), get_observation_date())
  }

  private[onemodel] fn get_valid_on_date() -> Option<i64> {
    if !m_already_read_data) read_data_from_db()
    valid_on_date
  }

  private[onemodel] fn get_observation_date() -> i64 {
    if !m_already_read_data) read_data_from_db()
    observation_date
  }

  /**
   * For descriptions of the meanings of these variables, see the comments
   * on create_tables(...), and examples in the database testing code in PostgreSQLDatabase or Database classes.
   */
  protected let mut valid_on_date: Option<i64> = None;
  protected let mut observation_date: i64 = 0L;
 */
}