/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2016-2017 inclusive, and 2023, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
struct OmInstance  {
/*%%
package org.onemodel.core.model

import org.onemodel.core._

object OmInstance {
    fn addressLength -> Int {
    Database.om_instance_address_length
    }

    fn isDuplicate(dbIn: Database, address_in: String, selfIdToIgnoreIn: Option<String> = None) -> bool {
    dbIn.isDuplicateOmInstanceAddress(address_in, selfIdToIgnoreIn)
  }

    fn create(dbIn: Database, id_in: String, address_in: String, entity_id_in: Option<i64> = None) -> OmInstance {
    // Passing false for is_local_in because the only time that should be true is when it is created at db creation, for this site, and that is done
    // in the db class more directly.
    let insertion_date: i64 = dbIn.create_om_instance(id_in, is_local_in = false, address_in, entity_id_in);
    new OmInstance(dbIn, id_in, is_local_in = false, address_in = address_in, insertion_dateIn = insertion_date, entity_id_in = entity_id_in)
  }
}

/** See table definition in the database class for details.
  *
  * This 1st constructor instantiates an existing object from the DB. Generally use Model.createObject() to create a new object.
  * Note: Having Entities and other DB objects be readonly makes the code clearer & avoid some bugs, similarly to reasons for immutability in scala.
  */
class OmInstance(val m_db: Database, m_id: String) {
  //Idea: make m_id *etc* private in all model classes? and rename m_db to just db ("uniform access principle")?
  // (See comment in similar spot in BooleanAttribute for why not checking for exists, if m_db.is_remote.)
  if !m_db.is_remote && !m_db.omInstanceKeyExists(m_id)) {
    throw new OmException("Key " + m_id + Util::DOES_NOT_EXIST)
  }


  /** This one is perhaps only called by the database class implementation--so it can return arrays of objects & save more DB hits
    that would have to occur if it only returned arrays of keys. This DOES NOT create a persistent object--but rather should reflect
    one that already exists.
    */
    fn this(m_db: Database, m_id: String, is_local_in: bool, address_in: String, insertion_dateIn: i64, entity_id_in: Option<i64> = None) {
    this(m_db, m_id)
    mLocal = is_local_in
    mAddress = address_in
    m_insertion_date = insertion_dateIn
    mEntityId = entity_id_in
    m_already_read_data = true
  }

  /** When using, consider if getArchivedStatusDisplayString should be called with it in the display (see usage examples of getArchivedStatusDisplayString).
    * */
    fn get_id() -> String {
    if !m_already_read_data) read_data_from_db()
    m_id
  }

    fn getLocal() -> bool {
    if !m_already_read_data) read_data_from_db()
    mLocal
  }

    fn getCreationDate() -> i64 {
    if !m_already_read_data) read_data_from_db()
    m_insertion_date
  }

    fn getCreationDateFormatted() -> String {
    Util::DATEFORMAT.format(new java.util.Date(getCreationDate))
  }

    fn getAddress() -> String {
    if !m_already_read_data) read_data_from_db()
    mAddress
  }

    fn getEntityId() -> Option<i64> {
    if !m_already_read_data) read_data_from_db()
    mEntityId
  }

  protected fn read_data_from_db() {
    let omInstanceData: Array[Option[Any]] = m_db.getOmInstanceData(m_id);
    if omInstanceData.length == 0) {
      throw new OmException("No results returned from data request for: " + m_id)
    }
    mLocal = omInstanceData(0).get.asInstanceOf[bool]
    mAddress = omInstanceData(1).get.asInstanceOf[String]
    m_insertion_date = omInstanceData(2).get.asInstanceOf[i64]
    mEntityId = omInstanceData(3).asInstanceOf[Option<i64>]
    m_already_read_data = true
  }

    fn get_display_string() -> String {
    let result: String = m_id + ":" + (if mLocal) " (local)" else "") + " " + getAddress + ", created on " + getCreationDateFormatted;
    result
  }

    fn update(newAddress: String) /*%%-> Unit*/ {
    m_db.updateOmInstance(get_id, newAddress, getEntityId)
  }

    fn delete() {
    m_db.deleteOmInstance(m_id)
    }

  let mut m_already_read_data: bool = false;
  let mut mLocal: bool = false;
  let mut mAddress: String = "";
  let mut m_insertion_date: i64 = 0;
    let mut mEntityId: Option<i64> = None;
 */
}