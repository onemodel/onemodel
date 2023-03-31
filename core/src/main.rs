/*  This file is part of OneModel, a program to manage knowledge.
    Copyright in each year of 2003-2004, 2008-2019 inclusive, and 2022-2023 inclusive, Luke A. Call.
    OneModel is free software, distributed under a license that includes honesty, the Golden Rule,
    and the GNU Affero General Public License as published by the Free Software Foundation;
    see the file LICENSE for license version and details.
    OneModel is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero General Public License for more details.
    You should have received a copy of the GNU Affero General Public License along with OneModel.  If not, see <http://www.gnu.org/licenses/>
*/
pub mod controllers;
pub mod model;
pub mod color;
pub mod om_exception;
pub mod text_ui;
pub mod util;
use std::env;
use crate::controllers::controller::Controller;
// use crate::util::Util;
use crate::text_ui::TextUI;

/// Provides a text-based interface for efficiency, or for people who like that,
/// The first OM user interface, it is intended to demonstrate basic concepts until we (or someone?) can make something more friendly,
/// or a library and/or good REST api for such.
fn main() {
    /*%%$%%next tasks?:
        fix more/warnings? formatting? (sep't ckin)
        do all util, db, pg, & their tests at once: style and compile and test.
        MAKE TESTS for code be4 ckin! see them each fail then pass.
        Debug/breakpoints...? ??? (esp in pg and util?)
        make code compile that i have now?
        AND rustfmt  (separate commit tho?)
        In OM,  using #[derive(Debug)] on a all? And fmt::Display (vs fmt::Debug) on all public types.
        other %%$%s, %%s &c
     */
    //%%pledge/unveil here?  examples in crates.io? or sch for openbsd or libc?

    let args: Vec<String> = env::args().collect();
    // dbg!(args.as_slice());
    //%%see std::env::args() docs: next 2 args dift on windows, might be 0 & 1 not 1 & 2? If a change,
    // see next cmt also about args.len() and adjust if needed, for windows.
    let default_username: Option<&String> = args.get(1);
    let default_password: Option<&String> = args.get(2);
    // If user provides a single command-line argument to the app, consider that a request to
    // prompt for username & password. (The first argument, in my environment, is always the name
    // of the app, so when the user provides another that makes 2.  That was not true under
    // java, only user-provided arguments being included in args, so the length check then was
    // just 1 to set forceUsernamePasswordPrompt to true.)
    let force_user_pass_prompt: bool = if args.len() == 2 { true } else { false };
    println!("args.len: {}", args.len());//%%
    println!( //%%
        "forceUsernamePasswordPrompt: {}",
        force_user_pass_prompt
    );

    let ui = TextUI{
        testing: false,
    };

    let controller = Controller::new_for_non_tests(
        ui,
        force_user_pass_prompt,
        default_username,
        default_password,
    );
    controller.start();

}
