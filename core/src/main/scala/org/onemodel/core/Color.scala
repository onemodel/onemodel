/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2014 and 2016-2016 inclusive, Luke A Call; all rights reserved.
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
package org.onemodel.core

object Color {
  // ansi codes (used scalatest source code as a reference; probably doc'd variously elsewhere also)
  val (green, cyan, yellow, red, reset) = {
    if (Util.isWindows) {
      ("", "", "", "", "")
    } else {
      ("\033[32m",
        "\033[36m",
        "\033[33m",
        "\033[31m",
        "\033[0m")
    }
  }

  def red(s:String): String = {
    red + s + reset
  }

  def cyan(s:String): String = {
    cyan + s + reset
  }

  def blue(s:String) : String = cyan(s)

  def green(s:String): String = {
    green + s + reset
  }

  def yellow(s:String): String = {
    yellow + s + reset
  }

}
