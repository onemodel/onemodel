/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2013-2017 inclusive, Luke A. Call; all rights reserved.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule, guidelines around binary
    distribution, and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>

  ---------------------------------------------------
  (See comment in this place in PostgreSQLDatabase.scala about possible alternatives to this use of the db via this layer and jdbc.)
*/
package org.onemodel.core.model

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
                                 var entityId2: Long,
                                 var isRemote: Boolean,
                                 var remoteInstanceId: String)
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
