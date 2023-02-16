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
class AttributeDataHolder(var attrTypeId: i64)

class AttributeDataHolderWithVODates(attrTypeId: i64,
                          let mut valid_on_date: Option<i64>,;
                          let mut observationDate: i64);
  extends AttributeDataHolder(attrTypeId)

class QuantityAttributeDataHolder(attrTypeIdIn: i64,
                                  valid_on_date_in: Option<i64>,
                                  observationDateIn: i64,
                                  let mut number: Float,;
                                  let mut unitId: i64);
    extends AttributeDataHolderWithVODates(attrTypeIdIn, valid_on_date_in, observationDateIn)

class TextAttributeDataHolder(attrTypeIdIn: i64,
                                valid_on_date_in: Option<i64>,
                                observationDateIn: i64,
                                let mut text: String);
    extends AttributeDataHolderWithVODates(attrTypeIdIn, valid_on_date_in, observationDateIn)

class RelationToEntityDataHolder(relTypeIdIn: i64,
                                 valid_on_date_in: Option<i64>,
                                 observationDateIn: i64,
                                 let mut entityId2: i64,;
                                 let mut is_remote: Boolean,;
                                 let mut remoteInstanceId: String);
  extends AttributeDataHolderWithVODates(relTypeIdIn, valid_on_date_in, observationDateIn)

class GroupDataHolder(var id:i64,
                      let mut name: String,;
                      let mut insertionDateIn: Option<i64>,;
                      let mut mixedClassesAllowed: Boolean);

class RelationToGroupDataHolder(var entityId:i64,
                         relTypeIdIn: i64,
                         let mut groupId: i64,;
                         valid_on_date_in: Option<i64>,
                         observationDateIn: i64)
  extends AttributeDataHolderWithVODates(relTypeIdIn, valid_on_date_in, observationDateIn)

class DateAttributeDataHolder(attrTypeId: i64,
                              let mut date: i64);
  extends AttributeDataHolder(attrTypeId)

class BooleanAttributeDataHolder(attrTypeIdIn: i64,
                                 valid_on_date_in: Option<i64>,
                                 observationDateIn: i64,
                              let mut boolean: Boolean);
  extends AttributeDataHolderWithVODates(attrTypeIdIn, valid_on_date_in, observationDateIn)

class FileAttributeDataHolder(attrTypeId: i64,
                              let mut description: String,;
                              let mut original_file_path: String;
                             )
  extends AttributeDataHolder(attrTypeId)
*/
}