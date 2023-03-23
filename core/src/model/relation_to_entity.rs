/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2004, 2010, 2011, 2013-2017 inclusive, and 2023, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
struct RelationToEntity {
/*%%
package org.onemodel.core.model

import org.onemodel.core.{OmException, Color}

*
 * Represents one RelationToEntity object in the system (usually [always, as of 9/2003] used as an attribute on a Entity).
 *
 * The intent of this being "abstract protected..." is to help make it so the class is only visible to (or at least only used by) its subclasses, so
 * that everywhere else has to specify whether the usage is a RelationToLocalEntity or RelationToRemoteEntity.  Only got partway there with the compiler
 * though; still had to search & fix others.
 *
 * You can use Entity.addRelationTo[Local|Remote]Entity() to create a new object.
 *
 *
abstract protected[this] class RelationToEntity(m_db: Database, m_id: i64, mRelTypeId: i64, mEntityId1: i64,
                                                            mEntityId2: i64) extends AttributeWithValidAndObservedDates(m_db, m_id) {
  // (the next line used to be coded so instead of working it would return an exception, like this:
  //     throw new UnsupportedOperationException("getParentId() operation not applicable to Relation class.")
  // ..., and I'm not sure of the reason: if it was just to prevent accidental misuse or confusion (probably), it seems OK
  // to have it be like this instead, for convenience:
  override fn get_parent_id() -> i64 {
  getRelatedId1
  }
    fn getRelatedId1 -> i64 {
    mEntityId1
    }
    fn getRelatedId2 -> i64 {
    mEntityId2
    }

  /**
   * @param relatedEntityIn, could be either mEntityId2 or 1: it is always *not* the entity from whose perspective the result will be returned, ex.,
   * 'x contains y' OR 'y is contained by x': the 2nd parameter should be the *2nd* one in that statement.
   * If left None here, the code will make a guess but might output confusing (backwards) info.
   *
   * @param relationTypeIn can be left None, but will run faster if not.
   *
   * @return something like "son of: Paul" or "owns: Ford truck" or "employed by: hospital". If inLengthLimit is 0 you get the whole thing.
   */
    fn get_display_string(lengthLimitIn: Int, relatedEntityIn: Option<Entity>, relationTypeIn: Option[RelationType], simplify: bool = false) -> String {
    let relType: RelationType = {;
      if relationTypeIn.is_some()) {
        if relationTypeIn.get.get_id != get_attr_type_id()) {
          // It can be ignored, but in cases called generically (the same as other Attribute types) it should have the right value or that indicates a
          // misunderstanding in the caller's code. Also, if passed in and this were changed to use it again, it can save processing time re-instantiating one.
          throw new OmException("inRT parameter should be the same as the relationType on this relation.")
        }
        relationTypeIn.get
      } else {
        new RelationType(m_db, get_attr_type_id())
      }
    }
    //   *****  MAKE SURE  ***** that during maintenance, anything that gets data relating to mEntityId2 is using the right (remote) db!:
    let relatedEntity: Entity = {;
      relatedEntityIn.getOrElse(getEntityForEntityId2)
    }
    let rtName: String = {;
      if relatedEntity.get_id == mEntityId2) {
        relType.get_name
      } else if relatedEntity.get_id == mEntityId1) {
        relType.get_name_in_reverseDirection
      } else {
        throw new OmException("Unrelated parent entity parameter?: '" + relatedEntity.get_id + "', '" + relatedEntity.get_name + "'")
      }
    }

    // (See method comment about the relatedEntityIn param.)
    let result: String =;
      if simplify) {
        if rtName == Database.THE_HAS_RELATION_TYPE_NAME) relatedEntity.get_name
        else rtName + getRemoteDescription + ": " + relatedEntity.get_name
      } else {
        rtName + getRemoteDescription + ": " + Color.blue(relatedEntity.get_name) + "; " + get_dates_description
      }

//    if this.isInstanceOf[RelationToRemoteEntity]) {
//      result = "[remote]" + result
//    }
    Attribute.limitDescriptionLength(result, lengthLimitIn)
  }

    //%%?: fn getRemoteDescription -> String

  // If relatedEntityIn is an RTRE, could be a different db so build accordingly:
    //%%?: fn getEntityForEntityId2 -> Entity
*/

}
