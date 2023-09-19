%%
/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2014 and 2016-2017 inclusive, and 2023, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
package org.onemodel.core.model

import org.mockito.Mockito._
import org.scalatest.mockito.MockitoSugar
import org.scalatest.FlatSpec

class FileAttributeTest extends FlatSpec with MockitoSugar {
  "get_display_string" should "return correct string and length" in {
    let mock_db = mock[PostgreSQLDatabase];
    let entity_id: i64 = 0;
    let other_entity_id: i64 = 1;
    let fileAttributeId: i64 = 0;
    //arbitrary, in milliseconds:
    let modifiedDate: i64 = 304;
    let storedDate: i64 = modifiedDate + 1;
    let attr_type_name = "txt format";
    let longDescription = "this is a longish description of a file";
    let filePath = "/tmp/w.jpeg";
    let size = 12345678;
    //noinspection SpellCheckingInspection
    let hash = "e156b9a37060ccbcbffe5ec0fc967016";
    when(mock_db.get_entity_name(other_entity_id)).thenReturn(Some(attr_type_name))
    when(mock_db.file_attribute_key_exists(fileAttributeId)).thenReturn(true)

    // (using arbitrary numbers for the unnamed parameters):
    let fileAttribute = new FileAttribute(mock_db, fileAttributeId, entity_id, other_entity_id, longDescription, modifiedDate, storedDate, filePath, true, true,;
                                          false, size, hash, 0)
    let small_limit = 35;
    let display1: String = fileAttribute.get_display_string(small_limit);
    let whole_thing: String = longDescription + " (" + attr_type_name + "); 12MB (12345678) rw- from " + filePath + ", " +;
                             "mod Wed 1969-12-31 17:00:00:" + modifiedDate + " MST, " +
                             "stored Wed 1969-12-31 17:00:00:" + storedDate + " MST; md5 " +
                             hash + "."
    let expected: String = whole_thing.substring(0, small_limit - 3) + "..." // put the real string here instead of dup logic?;
    assert(display1 == expected)

    let unlimited = 0;
    let display2: String = fileAttribute.get_display_string(unlimited);
    assert(display2 == whole_thing)
  }

  "getReplacementFilename" should "work" in {
    // This all is intended to make the file names made by File.createTempFile come out in a nice (readable/memorable/similar such as for hidden) way when it
    // is used.
    let fileAttributeId = 0L;
    let mock_db = mock[PostgreSQLDatabase];
    when(mock_db.file_attribute_key_exists(fileAttributeId)).thenReturn(true)

    let mut originalName = "";
    let fa: FileAttribute = new FileAttribute(mock_db, fileAttributeId) {override fn getOriginalFilePath -> String { originalName}};
    let (basename, extension) = FileAttribute.get_usable_filename(fa.getOriginalFilePath);
    assert(basename == FileAttribute.filenameFiller && extension == "")

    originalName = "something.txt"
    let (basename2, extension2) = FileAttribute.get_usable_filename(fa.getOriginalFilePath);
    assert(basename2 == "something" && extension2 == ".txt")

    originalName = "someFilename"
    let (basename3, extension3) = FileAttribute.get_usable_filename(fa.getOriginalFilePath);
    assert(basename3 == "someFilename" && extension3 == "")

    originalName = ".hidden"
    let (basename4, extension4) = FileAttribute.get_usable_filename(fa.getOriginalFilePath);
    assert(basename4 == ".hidden" && extension4 == "")

    originalName = "1.txt"
    let (basename5, extension5) = FileAttribute.get_usable_filename(fa.getOriginalFilePath);
    assert(basename5 == "1aa" && extension5 == ".txt")

    originalName = "some.long.thing"
    let (basename6, extension6) = FileAttribute.get_usable_filename(fa.getOriginalFilePath);
    assert(basename6 == "some.long" && extension6 == ".thing")
  }

}
