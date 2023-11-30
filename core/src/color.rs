/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2014 and 2016-2017 inclusive, and 2023, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>

*/
use crate::util::Util;

pub struct Color {}

impl Color {
    // This is how we used to do it in scala, and functions below just used the set values like
    // returning
    //   red + s + reset

    // ansi codes (used scalatest source code as a reference; probably doc'd elsewhere also)
    // let (green, cyan, yellow, red, reset) = {
    //   if Util::is_windows() {
    // ("", "", "", "", "")
    // } else {
    //   ("\033[32m",
    //     "\033[36m",
    //     "\033[33m",
    //     "\033[31m",
    //     "\033[0m")
    // }
    // }

    // And probably there is a better way than this, in Rust....

    const ANSI_GREEN: &'static str = "\033[32m";
    const ANSI_CYAN: &'static str = "\033[36m";
    const ANSI_YELLOW: &'static str = "\033[33m";
    const ANSI_RED: &'static str = "\033[31m";
    const ANSI_RESET: &'static str = "\033[0m";

    pub fn red(s: &String) -> String {
        if Util::is_windows() {
            s.clone()
        } else {
            format!("{}{}{}", ANSI_RED, s, ANSI_RESET)
        }
    }

    pub fn cyan(s: &String) -> String {
        if Util::is_windows() {
            s.clone()
        } else {
            format!("{}{}{}", ANSI_CYAN, s, ANSI_RESET)
        }
    }

    pub fn blue(s: &String) -> String {
        cyan(s)
    }

    pub fn green(s: &String) -> String {
        if Util::is_windows() {
            s.clone()
        } else {
            format!("{}{}{}", ANSI_GREEN, s, ANSI_RESET)
        }
    }

    pub fn yellow(s: &String) -> String {
        if Util::is_windows() {
            s.clone()
        } else {
            format!("{}{}{}", ANSI_YELLOW, s, ANSI_RESET)
        }
    }
}
