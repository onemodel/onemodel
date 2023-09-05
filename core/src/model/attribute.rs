/* . This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2004, 2010, 2011, 2013-2017 inclusive, and 2023, Luke A Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
use crate::util::Util;
use chrono::prelude::*;
use chrono::LocalResult;

//%%$%%%%IDEAS: mbe do polymorphism by 1) seeing how postgresql obj does it as a trait obj,
//2) considering that ":" to have one trait have the methods of another, 3) include BA has-a AttributeData to hold the data parts,
// and Attr holds the methods?
// Reread the Book ch/s on traits, applicable sections, & see if there is some betr way? Such as in ch 17 section 17.3
// "requesting a review of the post changes its state" and teh following code w/ an example.

pub struct Attribute {}

impl Attribute {
    /*%%
    package org.onemodel.core.model

    object Attribute {
      // unlike in Controller, these are intentionally a little different, for displaying also the day of the week:
      //%%see if below uses are tested/working/done or if these are needed for anything
      let DATEFORMAT = new java.text.SimpleDateFormat("EEE yyyy-MM-dd HH:mm:ss:SSS zzz");
      let DATEFORMAT_WITH_ERA = new java.text.SimpleDateFormat("EEE GGyyyy-MM-dd HH:mm:ss:SSS zzz");
    */
    pub fn useful_date_format(d: i64) -> String {
        // No need to print "AD" unless we're really close?, as in this example:
        //scala > let DATEFORMAT_WITH_ERA = new java.text.SimpleDateFormat("GGyyyy-MM-dd HH:mm:ss:SSS zzz");
        //scala > DATEFORMAT_WITH_ERA.parse("ad 1-03-01 00:00:00:000 GMT").getTime //i.e., Jan 3, 1 AD.
        //res100: i64 = -62130672000000
        // see Util::DATEFORMAT* for comment about ERA (BC/AD).
        // if d > -62130672000000_i64 {
        //     DATEFORMAT.format(d)
        // %%need to test this date thing also to confirm works as expected/same as scala OM.
        //See also uses of this, in case need to borrow one or update both, in util.rs .
        let date: LocalResult<DateTime<Utc>> = Utc.timestamp_opt(d, 0);
        match date {
            LocalResult::None => {
                "Error(1) trying to format {} as a date/time; probably a bug.".to_string()
            }
            LocalResult::Single(dt) => {
                let typed_dt: DateTime<Utc> = dt;
                typed_dt.format(Util::DATEFORMAT).to_string()
            }
            _ => "Error(2) trying to format {} as a date/time; probably a bug.".to_string(),
        }

        // } else {
        //     DATEFORMAT_WITH_ERA.format(d)
        // }
    }

    /*
      /// @param input The value to chop down in size.
      /// @param lengthLimitIn If <= 0, no change.
      /// @return A value equal or shorter in length.
      fn limitDescriptionLength(input: String, lengthLimitIn: Int) -> String {
        if lengthLimitIn != 0 && input.length > lengthLimitIn) {
          input.substring(0, lengthLimitIn - 3) + "..."
        } else input
      }

    }
    /**
     * Represents one attribute object in the system (usually [always, as of 1/2004] used as an attribute on a Entity).
     * Originally created as a place to put common stuff between Relation/QuantityAttribute/TextAttribute.
     */
    abstract class Attribute(val m_db: Database, m_id: i64) {
      // idea: somehow use scala features better to make it cleaner, so we don't need these extra 2 vars, because they are
      // used in 1-2 instances, and ignored in the rest.  One thing is that RelationTo[Local|Remote]Entity and RelationToGroup are Attributes. Should they be?
        fn get_display_string(inLengthLimit: Int, parentEntity: Option<Entity>, inRTId: Option[RelationType], simplify: bool = false) -> String;

      protected fn read_data_from_db();

        fn delete();

      private[onemodel] fn get_idWrapper -> IdWrapper {
        new IdWrapper(m_id)
      }

        fn get_id -> i64 {
        m_id
      }

        fn get_form_id -> Int {
        Database.get_attribute_form_id(this.getClass.getSimpleName)
      }

      protected fn assignCommonVars(parent_id_in: i64, attr_type_id_in: i64, sorting_index_in: i64) {
        m_parent_id = parent_id_in
        m_attr_type_id = attr_type_id_in
        m_sorting_index = sorting_index_in
        m_already_read_data = true
      }

        fn get_attr_type_id() -> i64 {
        if !m_already_read_data) read_data_from_db()
        m_attr_type_id
      }

        fn getSortingIndex -> i64 {
        if !m_already_read_data) read_data_from_db()
        m_sorting_index
      }

    //(already implemented in boolean_attribute.  should move here?? Look how traits have traits!: "rust super trait"?)
      fn get_parent_id() -> i64 {
        if !m_already_read_data) read_data_from_db()
        m_parent_id
      }

      /**
       * For descriptions of the meanings of these variables, see the comments
       * on create_tables(...), and examples in the database testing code &/or in PostgreSQLDatabase or Database classes.
       */
      protected let mut m_parent_id: i64 = 0L;
      protected let mut m_attr_type_id: i64 = 0L;
      protected let mut m_already_read_data: bool = false;
      protected let mut m_sorting_index: i64 = 0L;
    */
}
