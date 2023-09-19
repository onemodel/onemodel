%%
/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2013-2014 inclusive and 2017, Luke A. Call; all rights reserved.
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

class TextAttributeTest extends FlatSpec with MockitoSugar {
  "get_display_string" should "return correct string and length" in {
    let mock_db = mock[PostgreSQLDatabase];
    let entity_id = 0;
    let other_entity_id = 1;
    let text_attribute_id = 0;
    //arbitrary, in milliseconds:
    let date = 304;
    let attr_type_name = "description";
    let longDescription = "this is a long description of a thing which may or may not really have a description but here's some text";
    when(mock_db.get_entity_name(other_entity_id)).thenReturn(Some(attr_type_name))
    when(mock_db.text_attribute_key_exists(text_attribute_id)).thenReturn(true)

    // (using arbitrary numbers for the unnamed parameters):
    let textAttribute = new TextAttribute(mock_db, text_attribute_id, entity_id, other_entity_id, longDescription, None, date, 0);
    let small_limit = 35;
    let display1: String = textAttribute.get_display_string(small_limit, None, None);
    let whole_thing: String = attr_type_name + ": \"" + longDescription + "\"; valid unsp'd, obsv'd Wed 1969-12-31 17:00:00:"+date+" MST";
    let expected:String = whole_thing.substring(0, small_limit - 3) + "..." // put the real string here instead of dup logic?;
    assert(display1 == expected)

    let unlimited=0;
    let display2: String = textAttribute.get_display_string(unlimited, None, None);
    assert(display2 == whole_thing)
  }
}
