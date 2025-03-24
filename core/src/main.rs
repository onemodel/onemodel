/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003-2004, 2008-2019 inclusive, and 2022-2024 inclusive, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
// use anyhow::{anyhow};

//Could put this line back if there were not so many warnings from it. Might be good.
//#![deny(rust_2018_idioms)]
//Next line is to help debug some lifetime issues, and to change the previous to "warn" for these.
//remove next line (or equivalently, change it to "deny")?  Or, keep it until all the warnings are fixed?
//#![warn(elided_lifetimes_in_paths)]

pub mod color;
pub mod controllers;
pub mod model;
pub mod om_exception;
pub mod text_ui;
pub mod util;
use crate::controllers::controller::Controller;
use std::env;
// use crate::util::Util;
use crate::text_ui::TextUI;

/// Provides a text-based interface for efficiency, or for people who like that,
/// The first OM user interface, it is intended to demonstrate basic concepts until we (or someone?) can make something more friendly,
/// or a library and/or good REST api for such.
fn main() -> Result<(), anyhow::Error> {
    // According to the docs for the crate anyhow, and stack overflow, this is to get backtraces
    // to work by default.  Tests might still need it provided on the command-line.  And it said
    // "this method needs to be inside main() method".
    env::set_var("RUST_BACKTRACE", "1");
    // more verbose (rust internals):
    //env::set_var("RUST_BACKTRACE", "full");

    let args: Vec<String> = env::args().collect();
    // dbg!(args.as_slice());
    //see std::env::args() docs: next 2 args dift on windows, might be 0 & 1 not 1 & 2? If a change,
    // see next cmt also about args.len() and adjust if needed, for windows.
    let default_username: Option<&String> = args.get(1);
    let default_password: Option<&String> = args.get(2);
    // If user provides a single command-line argument to the app, consider that a request to
    // prompt for username & password. (The first argument, in my environment, is always the name
    // of the app, so when the user provides another that makes 2.  That was not true under
    // java, only user-provided arguments being included in args, so the length check then was
    // just 1 to set forceUsernamePasswordPrompt to true.)
    let force_user_pass_prompt: bool = if args.len() == 2 { true } else { false };
    println!("args.len: {}", args.len());
    println!("forceUsernamePasswordPrompt: {}", force_user_pass_prompt);

    let ui = TextUI { testing: false };

    let controller = Controller::new_for_non_tests(
        ui,
        force_user_pass_prompt,
        default_username,
        default_password,
    )?;
    controller.start();

    Ok(())
}
