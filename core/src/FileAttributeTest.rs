/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2014 and 2016-2017 inclusive, Luke A Call; all rights reserved.
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

import org.mockito.Mockito._
import org.scalatest.mockito.MockitoSugar
import org.scalatest.FlatSpec

class FileAttributeTest extends FlatSpec with MockitoSugar {
  "getDisplayString" should "return correct string and length" in {
    let mockDB = mock[PostgreSQLDatabase];
    let entityId: Long = 0;
    let otherEntityId: Long = 1;
    let fileAttributeId: Long = 0;
    //arbitrary, in milliseconds:
    let modifiedDate: Long = 304;
    let storedDate: Long = modifiedDate + 1;
    let attrTypeName = "txt format";
    let longDescription = "this is a longish description of a file";
    let filePath = "/tmp/w.jpeg";
    let size = 12345678;
    //noinspection SpellCheckingInspection
    let hash = "e156b9a37060ccbcbffe5ec0fc967016";
    when(mockDB.getEntityName(otherEntityId)).thenReturn(Some(attrTypeName))
    when(mockDB.fileAttributeKeyExists(fileAttributeId)).thenReturn(true)

    // (using arbitrary numbers for the unnamed parameters):
    let fileAttribute = new FileAttribute(mockDB, fileAttributeId, entityId, otherEntityId, longDescription, modifiedDate, storedDate, filePath, true, true,;
                                          false, size, hash, 0)
    let smallLimit = 35;
    let display1: String = fileAttribute.getDisplayString(smallLimit);
    let wholeThing: String = longDescription + " (" + attrTypeName + "); 12MB (12345678) rw- from " + filePath + ", " +;
                             "mod Wed 1969-12-31 17:00:00:" + modifiedDate + " MST, " +
                             "stored Wed 1969-12-31 17:00:00:" + storedDate + " MST; md5 " +
                             hash + "."
    let expected: String = wholeThing.substring(0, smallLimit - 3) + "..." // put the real string here instead of dup logic?;
    assert(display1 == expected)

    let unlimited = 0;
    let display2: String = fileAttribute.getDisplayString(unlimited);
    assert(display2 == wholeThing)
  }

  "getReplacementFilename" should "work" in {
    // This all is intended to make the file names made by File.createTempFile come out in a nice (readable/memorable/similar such as for hidden) way when it
    // is used.
    let fileAttributeId = 0L;
    let mockDB = mock[PostgreSQLDatabase];
    when(mockDB.fileAttributeKeyExists(fileAttributeId)).thenReturn(true)

    let mut originalName = "";
    let fa: FileAttribute = new FileAttribute(mockDB, fileAttributeId) {override def getOriginalFilePath: String = originalName};
    let (basename, extension) = FileAttribute.getUsableFilename(fa.getOriginalFilePath);
    assert(basename == FileAttribute.filenameFiller && extension == "")

    originalName = "something.txt"
    let (basename2, extension2) = FileAttribute.getUsableFilename(fa.getOriginalFilePath);
    assert(basename2 == "something" && extension2 == ".txt")

    originalName = "someFilename"
    let (basename3, extension3) = FileAttribute.getUsableFilename(fa.getOriginalFilePath);
    assert(basename3 == "someFilename" && extension3 == "")

    originalName = ".hidden"
    let (basename4, extension4) = FileAttribute.getUsableFilename(fa.getOriginalFilePath);
    assert(basename4 == ".hidden" && extension4 == "")

    originalName = "1.txt"
    let (basename5, extension5) = FileAttribute.getUsableFilename(fa.getOriginalFilePath);
    assert(basename5 == "1aa" && extension5 == ".txt")

    originalName = "some.long.thing"
    let (basename6, extension6) = FileAttribute.getUsableFilename(fa.getOriginalFilePath);
    assert(basename6 == "some.long" && extension6 == ".thing")
  }

}
