/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2013-2014 inclusive, Luke A Call; all rights reserved.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule, guidelines around binary
    distribution, and the GNU Affero General Public License as published by the Free Software Foundation, either version 3
    of the License, or (at your option) any later version.  See the file LICENSE for details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>

  ---------------------------------------------------
  If we ever do port to another database, create the Database interface (removed around 2014-1-1 give or take) and see other changes at that time.
  An alternative method is to use jdbc escapes (but this actually might be even more work?):  http://jdbc.postgresql.org/documentation/head/escapes.html  .
  Another alternative is a layer like JPA, ibatis, hibernate  etc etc.

*/
package org.onemodel.model

//idea: replace these w/ tuples?
class AttributeDataHolder(var attrTypeId: Long)

class AttributeDataHolderWithVODates(attrTypeId: Long,
                          var validOnDate: Option[Long],
                          var observationDate: Long)
  extends AttributeDataHolder(attrTypeId)

class QuantityAttributeDataHolder(attrTypeIdIn: Long,
                                  validOnDateIn: Option[Long],
                                  observationDateIn: Long,
                                  var number: Float,
                                  var unitId: Long)
    extends AttributeDataHolderWithVODates(attrTypeIdIn, validOnDateIn, observationDateIn)

class TextAttributeDataHolder(attrTypeIdIn: Long,
                                validOnDateIn: Option[Long],
                                observationDateIn: Long,
                                var text: String)
    extends AttributeDataHolderWithVODates(attrTypeIdIn, validOnDateIn, observationDateIn)

class RelationToEntityDataHolder(relTypeIdIn: Long,
                         validOnDateIn: Option[Long],
                         observationDateIn: Long,
                         var entityId2: Long)
    extends AttributeDataHolderWithVODates(relTypeIdIn, validOnDateIn, observationDateIn)

class GroupDataHolder(var id:Long,
                      var name: String,
                      var insertionDateIn: Option[Long],
                      var mixedClassesAllowed: Boolean)

class RelationToGroupDataHolder(var entityId:Long,
                         relTypeIdIn: Long,
                         var groupId: Long,
                         validOnDateIn: Option[Long],
                         observationDateIn: Long)
  extends AttributeDataHolderWithVODates(relTypeIdIn, validOnDateIn, observationDateIn)

class DateAttributeDataHolder(attrTypeId: Long,
                              var date: Long)
  extends AttributeDataHolder(attrTypeId)

class BooleanAttributeDataHolder(attrTypeIdIn: Long,
                                 validOnDateIn: Option[Long],
                                 observationDateIn: Long,
                              var boolean: Boolean)
  extends AttributeDataHolderWithVODates(attrTypeIdIn, validOnDateIn, observationDateIn)

class FileAttributeDataHolder(attrTypeId: Long,
                              var description: String,
                              var originalFilePath: String
                             )
  extends AttributeDataHolder(attrTypeId)
