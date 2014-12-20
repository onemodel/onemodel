/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2004, 2010, 2011, and 2013-2014 inclusive, Luke A Call; all rights reserved.
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