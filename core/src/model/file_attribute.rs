/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2014-2017 inclusive, and 2023, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
struct FileAttribute {
/*%%
package org.onemodel.core.model

import scala.annotation.tailrec
import java.io.{File, FileOutputStream}
import org.apache.commons.io.FilenameUtils
import org.onemodel.core._

object FileAttribute {
    fn md5Hash(fileIn: java.io.File) -> String {
    //idea: combine somehow w/ similar logic in PostgreSQLDatabase.verifyFileAttributeContent ?
    let mut fis: java.io.FileInputStream = null;
    let d = java.security.MessageDigest.getInstance("MD5");
    try {
      fis = new java.io.FileInputStream(fileIn)
      let buffer = new Array[Byte](2048);
      let mut numBytesRead = 0;
      @tailrec
      fn calculateRest() {
        //IF ADDING ANY OPTIONAL PARAMETERS, be sure they are also passed along in the recursive call(s) w/in this method!
        numBytesRead = fis.read(buffer)
        //noinspection RemoveRedundantReturn  //left intentionally for reading clarify
        if numBytesRead == -1) return
        else {
          d.update(buffer, 0, numBytesRead)
          calculateRest()
        }
      }
      calculateRest()
    }
    finally if fis != null) fis.close()
    //a handy value for testing code like above, in comparison with the md5sum command on a file containing only "1234" (w/o quotes) and a linefeed (size 5):
    // let ba = Array[Byte]('1', '2', '3', '4', 10);
    //so then in scala REPL (interpreter) you set "val d..." as above, "d.update(ba)", and the below:
    // outputs same as command 'md5sum <file>':
    let md5sum: String = {;
      //noinspection LanguageFeature  // (the '&' use on next line is an 'advanced feature' style violation but it's the way i found to do it ...)
      d.digest.map(0xFF &).map {"%02x".format(_)}.foldLeft("") {_ + _}
    }
    md5sum
  }

  let filenameFiller = "aaa";

  /** Returns a prefix and suffix (like, "filename" and ".ext").
    * I.e., for use with java.nio.file.Files.createTempFile (which makes sure it will not collide with existing names).
    * Calling this likely presumes that the caller has already decided not to use the old path, or at least the old filename in the temp directory.
    */
    fn get_usable_filename(original_file_path_in: String) -> (String, String) {
    let originalName = FilenameUtils.getBaseName(original_file_path_in);

    // baseName has to be at least 3 chars, for createTempFile:
    let baseName: String = (originalName + FileAttribute.filenameFiller).substring(0, math.max(originalName.length, 3));

    let fullExtension: String = {;
      let dotlessExtension = FilenameUtils.getExtension(original_file_path_in);
      if dotlessExtension.length > 0) new String("." + dotlessExtension)
      else ""
    }
    //for hidden files to stay that way unless the size is too small:
    if baseName == FileAttribute.filenameFiller && fullExtension.length >= 3) (fullExtension, "")
    else (baseName, fullExtension)
  }

}

/** See TextAttribute etc for some comments.
  * Also, though this doesn't formally extend Attribute, it still belongs to the same group conceptually (just doesn't have the same date variables so code
  * not shared (idea: model that better, and in DateAttribute). (idea: IN FACT, ALL THE CODE RELATED TO THESE CLASSES COULD PROBABLY HAVE A LOT OF REDUNDANCY
  * REMOVED.)
  */
class FileAttribute(m_db: Database, m_id: i64) extends Attribute(m_db, m_id) {
  // (See comment in similar spot in BooleanAttribute for why not checking for exists, if m_db.is_remote.)
  if !m_db.is_remote && !m_db.fileAttributeKeyExists(m_id)) {
    throw new Exception("Key " + m_id + Util::DOES_NOT_EXIST)
  }


  /** This one is perhaps only called by the database class implementation--so it can return arrays of objects & save more DB hits
  that would have to occur if it only returned arrays of keys. This DOES NOT create a persistent object--but rather should reflect
  one that already exists.
    */
    fn this(m_db: Database, m_id: i64, parent_id_in: i64, attr_type_id_in: i64, descriptionIn: String, originalFileDateIn: i64, storedDateIn: i64,
           inOriginalFilePath: String, readableIn: bool, writableIn: bool, executableIn: bool, sizeIn: i64, md5hashIn: String, sorting_index_in: i64) {
    this(m_db, m_id)
    mDescription = descriptionIn
    mOriginalFileDate = originalFileDateIn
    mStoredDate = storedDateIn
    mOriginalFilePath = inOriginalFilePath
    mReadable = readableIn
    mWritable = writableIn
    mExecutable = executableIn
    mSize = sizeIn
    mMd5hash = md5hashIn
    assignCommonVars(parent_id_in, attr_type_id_in, sorting_index_in)
  }

    fn get_display_string(lengthLimitIn: Int, unused: Option<Entity> = None, unused2: Option[RelationType] = None, simplify: bool = false) -> String {
    let typeName: String = m_db.get_entity_name(get_attr_type_id()).get;
    let mut result: String = getDescription + " (" + typeName + "); " + getFileSizeDescription;
    if ! simplify) result = result + " " + getPermissionsDescription + " from " +
                             getOriginalFilePath + ", " + get_dates_description + "; md5 " + getMd5Hash + "."
    Attribute.limitDescriptionLength(result, lengthLimitIn)
  }

    fn getPermissionsDescription -> String {
    //ex: rwx or rw-, like "ls -l" does
    (if getReadable) "r" else "-") +
    (if getWritable) "w" else "-") +
    (if getExecutable) "x" else "-")
  }

    fn getFileSizeDescription -> String {
    // note: it seems that (as per SI? IEC?), 1024 bytes is now 1 "binary kilobyte" aka kibibyte or KiB, etc.
    let decimalFormat = new java.text.DecimalFormat("0");
    if getSize < math.pow(10, 3)) "" + getSize + " bytes"
    else if getSize < math.pow(10, 6)) "" + decimalFormat.format(getSize / math.pow(10, 3)) + "kB (" + getSize + ")"
    else if getSize < math.pow(10, 9)) "" + decimalFormat.format(getSize / math.pow(10, 6)) + "MB (" + getSize + ")"
    else "" + decimalFormat.format(getSize / math.pow(10, 9)) + "GB (" + getSize + ")"
  }

  protected fn read_data_from_db() {
    let faTypeData = m_db.getFileAttributeData(m_id);
    if faTypeData.length == 0) {
      throw new OmException("No results returned from data request for: " + m_id)
    }
    mDescription = faTypeData(1).get.asInstanceOf[String]
    mOriginalFileDate = faTypeData(3).get.asInstanceOf[i64]
    mStoredDate = faTypeData(4).get.asInstanceOf[i64]
    mOriginalFilePath = faTypeData(5).get.asInstanceOf[String]
    mReadable = faTypeData(6).get.asInstanceOf[bool]
    mWritable = faTypeData(7).get.asInstanceOf[bool]
    mExecutable = faTypeData(8).get.asInstanceOf[bool]
    mSize = faTypeData(9).get.asInstanceOf[i64]
    mMd5hash = faTypeData(10).get.asInstanceOf[String]
    assignCommonVars(faTypeData(0).get.asInstanceOf[i64], faTypeData(2).get.asInstanceOf[i64], faTypeData(11).get.asInstanceOf[i64])
  }


  // We don't update the dates, path, size, hash because we set those based on the file's own timestamp, path current date,
  // & contents when it is *first* written. So the only point to having an update method might be the attribute type & description.
  // AND note that: The dates for a fileAttribute shouldn't ever be None/NULL like with other Attributes, because it is the file date in the filesystem
  // before it was
  // read into OM, and the current date; so they should be known whenever adding a document.
    fn update(attr_type_id_in: Option<i64> = None, descriptionIn: Option<String> = None) {
    // write it to the database table--w/ a record for all these attributes plus a key indicating which Entity
    // it all goes with
    let descr = if descriptionIn.is_some()) descriptionIn.get else getDescription;
    let attrTypeId = if attr_type_id_in.is_some()) attr_type_id_in.get else get_attr_type_id();
    m_db.updateFileAttribute(get_id, get_parent_id(), attrTypeId, descr)
    mDescription = descr
    m_attr_type_id = attrTypeId
  }

  ///** Using Options for the parameters so caller can pass in only those desired (named), and other members will stay the same.
  //  */
  //fn update(attr_type_id_in: Option<i64> = None, descriptionIn: Option<String> = None, originalFileDateIn: Option<i64> = None,
  //           storedDateIn: Option<i64> = None, original_file_path_in: Option<String> = None, sizeIn: Option<i64> = None, md5hashIn: Option<String> = None) {
  //  // write it to the database table--w/ a record for all these attributes plus a key indicating which Entity
  //  // it all goes with
  //  //********IF THIS METHOD IS EVER UNCOMMENTED: BE SURE TO TEST THAT the values (like size, hash, original date,
  // stored date!) are untouched if unchanged if
  //  // not passed in!! And probably need to add the 3 boolean fields to it & test.
  //  m_db.updateFileAttribute(m_id, m_parent_id,
  //                          if attr_type_id_in == None) get_attr_type_id() else inAttrTypeId.get,
  //                          if descriptionIn == None) getDescription else inDescription.get,
  //                          if originalFileDateIn == None) getOriginalFileDate else originalFileDateIn.get,
  //                          if storedDateIn == None) getStoredDate else storedDateIn.get,
  //                          if original_file_path_in == None) getOriginalFilePath else original_file_path_in.get,
  //                          if sizeIn == None) getSize else sizeIn.get,
  //                          if md5hashIn == None) getMd5hash else md5hashIn.get)
  //}

  /** Removes this object from the system. */
    fn delete() {
    m_db.deleteFileAttribute(m_id)
    }

    fn get_dates_description -> String {
    "mod " + Attribute.useful_date_format(getOriginalFileDate) + ", stored " + Attribute.useful_date_format(getStoredDate)
    }

  private[onemodel] fn getOriginalFileDate -> i64 {
    if !m_already_read_data) read_data_from_db()
    mOriginalFileDate
  }

  private[onemodel] fn getStoredDate -> i64 {
    if !m_already_read_data) read_data_from_db()
    mStoredDate
  }

  private[onemodel] fn getDescription -> String {
    if !m_already_read_data) read_data_from_db()
    mDescription
  }

  private[onemodel] fn getOriginalFilePath -> String {
    if !m_already_read_data) read_data_from_db()
    mOriginalFilePath
  }

  private[onemodel] fn getSize -> i64 {
    if !m_already_read_data) read_data_from_db()
    mSize
  }

  private[onemodel] fn getMd5Hash -> String {
    if !m_already_read_data) read_data_from_db()
    mMd5hash
  }

    fn getReadable -> bool {
    if !m_already_read_data) read_data_from_db()
    mReadable
  }

    fn getWritable -> bool {
    if !m_already_read_data) read_data_from_db()
    mWritable
  }

    fn getExecutable -> bool {
    if !m_already_read_data) read_data_from_db()
    mExecutable
  }

  /** just calling the File.getUsableSpace function on a nonexistent file yields 0, so come up with something better. -1 if it just can't figure it out.
    */
    fn getUsableSpace(fileIn: File) -> i64 {
    try {
      if fileIn.exists) fileIn.getUsableSpace
      else if fileIn.getParentFile == null) -1
      else getUsableSpace(fileIn.getParentFile)
    } catch {
      // oh well, give up.
      case e: Exception => -1
    }
  }

  // Idea: how make the 2nd parameter an option with None as default, instead of null as default?
    fn retrieveContent(fileIn: File, damageFileForTesting: (File) => Unit = null) {
    let mut outputStream: FileOutputStream = null;
    try {
      if (!fileIn.exists()) || fileIn.length() < this.getSize) {
        let space = getUsableSpace(fileIn);
        if space > -1 && space < this.getSize) throw new OmException("Not enough space on disk to retrieve file of size " + this.getFileSizeDescription + ".")
      }
      outputStream = new FileOutputStream(fileIn)
      //idea: if the file exists, copy out to a temp name, then after retrieval delete it & rename the new one to it? (uses more space)
      let (sizeStoredInDb, hashStoredInDb) = m_db.getFileAttributeContent(get_id, outputStream);
      // idea: this could be made more efficient if we checked the hash during streaming it to the local disk (in m_db.getFileAttributeContent)
      // (as does m_db.verifyFileAttributeContent).

      // this is a hook so tests can verify that we do fail if the file isn't intact
      // (Idea:  huh?? This next line does nothing. Noted in tasks to see what is meant & make it do that. or at least more clear.)
      //noinspection ScalaUselessExpression  //left intentionally for reading clarify
      if damageFileForTesting != null) damageFileForTesting

      let downloadedFilesHash = FileAttribute.md5Hash(fileIn);
      if fileIn.length != sizeStoredInDb) throw new OmFileTransferException("File sizes differ!: stored/downloaded: " + sizeStoredInDb + " / " + fileIn
                                                                                                                                                  .length())
      if downloadedFilesHash != hashStoredInDb) throw new OmFileTransferException("The md5sum hashes differ!: stored/downloaded: " + hashStoredInDb + " / "
                                                                                   + downloadedFilesHash)
      fileIn.setReadable(getReadable)
      fileIn.setWritable(getWritable)
      fileIn.setExecutable(getExecutable)
    } finally {
      if outputStream != null) outputStream.close()
    }
  }

  /**
   * For descriptions of the meanings of these variables, see the comments
   * on create_tables(...), and examples in the database testing code, for & in PostgreSQLDatabase or Database classes.
   */
  private let mut mDescription: String = null;
  private let mut mOriginalFileDate: i64 = 0;
  private let mut mStoredDate: i64 = 0;
  private let mut mOriginalFilePath: String = null;
  private let mut mReadable: bool = false;
  private let mut mWritable: bool = false;
  private let mut mExecutable: bool = false;
  private let mut mSize: i64 = 0;
 */
  private let mut mMd5hash: String = null;
 */
}