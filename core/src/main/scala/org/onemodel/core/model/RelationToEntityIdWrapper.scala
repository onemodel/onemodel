/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2004, 2010, 2011, 2013-2014 inclusive and 2016, Luke A Call; all rights reserved.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule, guidelines around binary
    distribution, and the GNU Affero General Public License as published by the Free Software Foundation, either version 3
    of the License, or (at your option) any later version.  See the file LICENSE for details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>

  ---------------------------------------------------
  (See comment in this place in PostgreSQLDatabase.scala about possible alternatives to this use of the db via this layer and jdbc.)
*/
package org.onemodel.core.model

/** See comments in class Id. This was created to make it easier for TextUI's displayObjectListMenuAndSelectObject()
  method to return a single type of value, thus can be used to display/choose from dif't types of objects,
  regardless of the type of class the returned key was for.
  */
class RelationToEntityIdWrapper(inRelTypeId: Long, entityId1: Long, entityId2: Long) extends IdWrapper(inRelTypeId) {
  override def getId: Long = throw new Exception("For a RelationId, use getAttrTypeId() instead (i.e., don't confuse the key of a relation w/ the key of another class of object).")

  def getAttrTypeId: Long = inRelTypeId
  def getEntityId1: Long = entityId1
  def getEntityId2: Long = entityId2
}