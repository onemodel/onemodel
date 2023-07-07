/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2013-2017 inclusive and 2023, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
struct AttributeDataHolder {
/*%%
package org.onemodel.core.model

//idea: replace these w/ tuples?
class AttributeDataHolder(var attr_type_id: i64)

class AttributeDataHolderWithVODates(attr_type_id: i64,
                          let mut valid_on_date: Option<i64>,;
                          let mut observation_date: i64);
  extends AttributeDataHolder(attr_type_id)

class QuantityAttributeDataHolder(attr_type_id_in: i64,
                                  valid_on_date_in: Option<i64>,
                                  observation_date_in: i64,
                                  let mut number: Float,;
                                  let mut unitId: i64);
    extends AttributeDataHolderWithVODates(attr_type_id_in, valid_on_date_in, observation_date_in)

class TextAttributeDataHolder(attr_type_id_in: i64,
                                valid_on_date_in: Option<i64>,
                                observation_date_in: i64,
                                let mut text: String);
    extends AttributeDataHolderWithVODates(attr_type_id_in, valid_on_date_in, observation_date_in)

class RelationToEntityDataHolder(rel_type_id_in: i64,
                                 valid_on_date_in: Option<i64>,
                                 observation_date_in: i64,
                                 let mut entity_id2: i64,;
                                 let mut is_remote: bool,;
                                 let mut remoteInstanceId: String);
  extends AttributeDataHolderWithVODates(rel_type_id_in, valid_on_date_in, observation_date_in)

class GroupDataHolder(var id:i64,
                      let mut name: String,;
                      let mut insertion_dateIn: Option<i64>,;
                      let mut mixed_classes_allowed: bool);

class RelationToGroupDataHolder(var entity_id:i64,
                         rel_type_id_in: i64,
                         let mut group_id: i64,;
                         valid_on_date_in: Option<i64>,
                         observation_date_in: i64)
  extends AttributeDataHolderWithVODates(rel_type_id_in, valid_on_date_in, observation_date_in)

class DateAttributeDataHolder(attr_type_id: i64,
                              let mut date: i64);
  extends AttributeDataHolder(attr_type_id)

class BooleanAttributeDataHolder(attr_type_id_in: i64,
                                 valid_on_date_in: Option<i64>,
                                 observation_date_in: i64,
                              let mut boolean: bool);
  extends AttributeDataHolderWithVODates(attr_type_id_in, valid_on_date_in, observation_date_in)

class FileAttributeDataHolder(attr_type_id: i64,
                              let mut description: String,;
                              let mut original_file_path: String;
                             )
  extends AttributeDataHolder(attr_type_id)
*/
}