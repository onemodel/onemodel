/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2014-2015 inclusive, Luke A. Call; all rights reserved.
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

import scala.annotation.tailrec
import java.io.{File, FileOutputStream}
import org.apache.commons.io.FilenameUtils
import org.onemodel._
import org.onemodel.database.PostgreSQLDatabase

object FileAttribute {
  def md5Hash(fileIn: java.io.File): String = {
    //idea: combine somehow w/ similar logic in PostgreSQLDatabase.verifyFileAttributeContent ?
    var fis: java.io.FileInputStream = null
    val d = java.security.MessageDigest.getInstance("MD5")
    try {
      fis = new java.io.FileInputStream(fileIn)
      val buffer = new Array[Byte](2048)
      var numBytesRead = 0
      @tailrec
      def calculateRest() {
        numBytesRead = fis.read(buffer)
        if (numBytesRead == -1) return
        else {
          d.update(buffer, 0, numBytesRead)
          calculateRest()
        }
      }
      calculateRest()
    }
    finally if (fis != null) fis.close()
    //a handy value for testing code like above, in comparison with the md5sum command on a file containing only "1234" (w/o quotes) and a linefeed (size 5):
    // val ba = Array[Byte]('1', '2', '3', '4', 10)
    //so then in scala REPL (interpreter) you set "val d..." as above, "d.update(ba)", and the below:
    // outputs same as command 'md5sum <file>':
    val md5sum: String = {
      // (the '&' use is an 'advanced feature' style violation but it's the way i found to do it ...)
      d.digest.map(0xFF &).map {"%02x".format(_)}.foldLeft("") {_ + _}
    }
    md5sum
  }

  val filenameFiller = "aaa"

  /** Returns a prefix and suffix (like, "filename" and ".ext") which will not collide with an existing name in System.getProperty("java.io.tmpdir").
    * I.e., for use with java.io.File.createTempFile.  Calling this likely presumes that the caller has already decided not to use the old path,
    * or the old filename in the temp directory.
    */
  def getReplacementFilename(originalFilePathIn: String): (String, String) = {
    val originalName = FilenameUtils.getBaseName(originalFilePathIn)

    // baseName has to be at least 3 chars, for createTempFile:
    val baseName: String = (originalName + FileAttribute.filenameFiller).substring(0, math.max(originalName.length, 3))

    val fullExtension: String = {
      val dotlessExtension = FilenameUtils.getExtension(originalFilePathIn)
      if (dotlessExtension.length > 0) new String("." + dotlessExtension)
      else ""
    }
    //for hidden files to stay that way unless the size is too small:
    if (baseName == FileAttribute.filenameFiller && fullExtension.length >= 3) (fullExtension, "")
    else (baseName, fullExtension)
  }

}

/** See TextAttribute etc for some comments.
  * Also, though this doesn't formally extend Attribute, it still belongs to the same group conceptually (just doesn't have the same date variables so code
  * not shared (idea: model that better, and in DateAttribute). (idea: IN FACT, ALL THE CODE RELATED TO THESE CLASSES COULD PROBABLY HAVE A LOT OF REDUNDANCY
  * REMOVED.)
  */
class FileAttribute(mDB: PostgreSQLDatabase, mId: Long) extends Attribute(mDB, mId) {
  if (!mDB.fileAttributeKeyExists(mId)) {
    // DON'T CHANGE this msg unless you also change the trap for it, if used, in other code.
    throw new Exception("Key " + mId + " does not exist in database.")
  }


  /** This one is perhaps only called by the database class implementation--so it can return arrays of objects & save more DB hits
  that would have to occur if it only returned arrays of keys. This DOES NOT create a persistent object--but rather should reflect
  one that already exists.
    */
  def this(mDB: PostgreSQLDatabase, mId: Long, inParentId: Long, inAttrTypeId: Long, inDescription: String, inOriginalFileDate: Long, inStoredDate: Long,
           inOriginalFilePath: String, readableIn: Boolean, writableIn: Boolean, executableIn: Boolean, inSize: Long, inMd5hash: String) {
    this(mDB, mId)
    mDescription = inDescription
    mOriginalFileDate = inOriginalFileDate
    mStoredDate = inStoredDate
    mOriginalFilePath = inOriginalFilePath
    mReadable = readableIn
    mWritable = writableIn
    mExecutable = executableIn
    mSize = inSize
    mMd5hash = inMd5hash
    assignCommonVars(inParentId, inAttrTypeId)
  }

  def getDisplayString(lengthLimitIn: Int, unused: Option[Entity] = None, unused2: Option[RelationType] = None, simplify: Boolean = false): String = {
    val typeName: String = mDB.getEntityName(getAttrTypeId).get
    var result: String = getDescription + " (" + typeName + "); " + getFileSizeDescription
    if (! simplify) result = result + " " + getPermissionsDescription + " from " +
                             getOriginalFilePath + ", " + getDatesDescription + "; md5 " + getMd5Hash + "."
    Attribute.limitDescriptionLength(result, lengthLimitIn)
  }

  def getPermissionsDescription: String = {
    //ex: rwx or rw-, like "ls -l" does
    (if (getReadable) "r" else "-") +
    (if (getWritable) "w" else "-") +
    (if (getExecutable) "x" else "-")
  }

  def getFileSizeDescription: String = {
    // note: it seems that (as per SI? IEC?), 1024 bytes is now 1 "binary kilobyte" aka kibibyte or KiB, etc.
    val decimalFormat = new java.text.DecimalFormat("0")
    if (getSize < math.pow(10, 3)) "" + getSize + " bytes"
    else if (getSize < math.pow(10, 6)) "" + decimalFormat.format(getSize / math.pow(10, 3)) + "kB (" + getSize + ")"
    else if (getSize < math.pow(10, 9)) "" + decimalFormat.format(getSize / math.pow(10, 6)) + "MB (" + getSize + ")"
    else "" + decimalFormat.format(getSize / math.pow(10, 9)) + "GB (" + getSize + ")"
  }

  protected def readDataFromDB() {
    val faTypeData = mDB.getFileAttributeData(mId)
    mDescription = faTypeData(1).get.asInstanceOf[String]
    mOriginalFileDate = faTypeData(3).get.asInstanceOf[Long]
    mStoredDate = faTypeData(4).get.asInstanceOf[Long]
    mOriginalFilePath = faTypeData(5).get.asInstanceOf[String]
    mReadable = faTypeData(6).get.asInstanceOf[Boolean]
    mWritable = faTypeData(7).get.asInstanceOf[Boolean]
    mExecutable = faTypeData(8).get.asInstanceOf[Boolean]
    mSize = faTypeData(9).get.asInstanceOf[Long]
    mMd5hash = faTypeData(10).get.asInstanceOf[String]
    assignCommonVars(faTypeData(0).get.asInstanceOf[Long], faTypeData(2).get.asInstanceOf[Long])
  }


  // We don't update the dates, path, size, hash because we set those based on the file's own timestamp, path current date,
  // & contents when it is *first* written. So the only point to having an update method might be the attribute type & description.
  // AND note that: The dates for a fileAttribute shouldn't ever be None/NULL like with other Attributes, because it is the file date in the filesystem
  // before it was
  // read into OM, and the current date; so they should be known whenever adding a document.
  def update(inAttrTypeId: Option[Long] = None, inDescription: Option[String] = None) {
    // write it to the database table--w/ a record for all these attributes plus a key indicating which Entity
    // it all goes with
    val descr = if (inDescription.isEmpty) getDescription else inDescription.get
    val attrTypeId = if (inAttrTypeId.isEmpty) getAttrTypeId else inAttrTypeId.get
    mDB.updateFileAttribute(getId, getParentId,
                            attrTypeId,
                            descr)
    mDescription = descr
    mAttrTypeId = attrTypeId
  }

  ///** Using Options for the parameters so caller can pass in only those desired (named), and other members will stay the same.
  //  */
  //def update(inAttrTypeId: Option[Long] = None, inDescription: Option[String] = None, originalFileDateIn: Option[Long] = None,
  //           storedDateIn: Option[Long] = None, originalFilePathIn: Option[String] = None, sizeIn: Option[Long] = None, md5hashIn: Option[String] = None) {
  //  // write it to the database table--w/ a record for all these attributes plus a key indicating which Entity
  //  // it all goes with
  //  //********IF THIS METHOD IS EVER UNCOMMENTED: BE SURE TO TEST THAT the values (like size, hash, original date,
  // stored date!) are untouched if unchanged if
  //  // not passed in!! And probably need to add the 3 boolean fields to it & test.
  //  mDB.updateFileAttribute(mId, mParentId,
  //                          if (inAttrTypeId == None) getAttrTypeId else inAttrTypeId.get,
  //                          if (inDescription == None) getDescription else inDescription.get,
  //                          if (originalFileDateIn == None) getOriginalFileDate else originalFileDateIn.get,
  //                          if (storedDateIn == None) getStoredDate else storedDateIn.get,
  //                          if (originalFilePathIn == None) getOriginalFilePath else originalFilePathIn.get,
  //                          if (sizeIn == None) getSize else sizeIn.get,
  //                          if (md5hashIn == None) getMd5hash else md5hashIn.get)
  //}

  /** Removes this object from the system. */
  def delete() = mDB.deleteFileAttribute(mId)

  def getDatesDescription: String = "mod " + Attribute.usefulDateFormat(getOriginalFileDate) + ", stored " + Attribute.usefulDateFormat(getStoredDate)

  private[onemodel] def getOriginalFileDate: Long = {
    if (!mAlreadyReadData) readDataFromDB()
    mOriginalFileDate
  }

  private[onemodel] def getStoredDate: Long = {
    if (!mAlreadyReadData) readDataFromDB()
    mStoredDate
  }

  private[onemodel] def getDescription: String = {
    if (!mAlreadyReadData) readDataFromDB()
    mDescription
  }

  private[onemodel] def getOriginalFilePath: String = {
    if (!mAlreadyReadData) readDataFromDB()
    mOriginalFilePath
  }

  private[onemodel] def getSize: Long = {
    if (!mAlreadyReadData) readDataFromDB()
    mSize
  }

  private[onemodel] def getMd5Hash: String = {
    if (!mAlreadyReadData) readDataFromDB()
    mMd5hash
  }

  def getReadable: Boolean = {
    if (!mAlreadyReadData) readDataFromDB()
    mReadable
  }

  def getWritable: Boolean = {
    if (!mAlreadyReadData) readDataFromDB()
    mWritable
  }

  def getExecutable: Boolean = {
    if (!mAlreadyReadData) readDataFromDB()
    mExecutable
  }

  /** just calling the File.getUsableSpace function on a nonexistent file yields 0, so come up with something better. -1 if it just can't figure it out.
    */
  def getUsableSpace(fileIn: File): Long = {
    try {
      if (fileIn.exists) fileIn.getUsableSpace
      else if (fileIn.getParentFile == null) -1
      else getUsableSpace(fileIn.getParentFile)
    } catch {
      // oh well, give up.
      case e: Exception => -1
    }
  }

  // idea: how make hte 2nd parameter an option with None as default, instead of null as default?
  def retrieveContent(fileIn: File, damageFileForTesting: (File) => Unit = null) {
    var outputStream: FileOutputStream = null
    try {
      if ((!fileIn.exists()) || fileIn.length() < this.getSize) {
        val space = getUsableSpace(fileIn)
        if (space > -1 && space < this.getSize) throw new OmException("Not enough space on disk to retrieve file of size " + this.getFileSizeDescription + ".")
      }
      outputStream = new FileOutputStream(fileIn)
      //idea: if the file exists, copy out to a temp name, then after retrieval delete it & rename the new one to it? (uses more space)
      val (sizeStoredInDb, hashStoredInDb) = mDB.getFileAttributeContent(getId, outputStream)
      // idea: this could be made more efficient if we checked the hash during streaming it to the local disk (in mDB.getFileAttributeContent)
      // (as does mDB.verifyFileAttributeContent).

      // this is a hook so tests can verify that we do fail if the file isn't intact
      // (huh? This next line does nothing. Noted in tasks to see what is meant & make it do that. or at least more clear.)
      if (damageFileForTesting != null) damageFileForTesting

      val downloadedFilesHash = FileAttribute.md5Hash(fileIn)
      if (fileIn.length != sizeStoredInDb) throw new OmFileTransferException("File sizes differ!: stored/downloaded: " + sizeStoredInDb + " / " + fileIn
                                                                                                                                                  .length())
      if (downloadedFilesHash != hashStoredInDb) throw new OmFileTransferException("The md5sum hashes differ!: stored/downloaded: " + hashStoredInDb + " / "
                                                                                   + downloadedFilesHash)
      fileIn.setReadable(getReadable)
      fileIn.setWritable(getWritable)
      fileIn.setExecutable(getExecutable)
    } finally {
      if (outputStream != null) outputStream.close()
    }
  }

  /**
   * For descriptions of the meanings of these variables, see the comments
   * on PostgreSQLDatabase.createTables(...), and examples in the database testing code.
   */
  private var mDescription: String = null
  private var mOriginalFileDate: Long = 0
  private var mStoredDate: Long = 0
  private var mOriginalFilePath: String = null
  private var mReadable: Boolean = false
  private var mWritable: Boolean = false
  private var mExecutable: Boolean = false
  private var mSize: Long = 0
  private var mMd5hash: String = null
}