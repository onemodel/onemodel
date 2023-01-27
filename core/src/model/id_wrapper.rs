/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2004, 2010, 2013-2014 inclusive, 2017, and 2023, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule, guidelines around
    distribution, and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>

  ---------------------------------------------------
  (See comment in this place in PostgreSQLDatabase.scala about possible alternatives to this use of the db via this layer and jdbc.)
*/
struct IdWrapper {
    /*%%
    package org.onemodel.core.model

    ** Represents the unique ID (key) for an Entity or Attribute object in the system. Benefit is that we can return
      one of these from a method and the signature of the method does not have to specify whether it is
      the ID of a QuantityAttribute, Relation, etc (relation ID has 3 parts, Attribute and Entity ID's for example have one).

      (But: why not just return a i64 or Option[i64]?)
      *
    class IdWrapper(id: i64) {
        fn getId: i64 = {
        id
    }
     */
}