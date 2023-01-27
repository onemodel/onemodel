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
#[tokio::main] //%%%$%where put this thing fr sqlx pg example? what means/does?
async fn main() {
    //%%pledge/unveil here?  examples in crates.io? or sch for openbsd or libc?

    let args: Vec<String> = env::args().collect();
    // dbg!(args.as_slice());
    //%%see std::env::args() docs: next 2 args dift on windows, might be 0 & 1 not 1 & 2?
    let default_username: Option<&String> = args.get(1);
    let default_password: Option<&String> = args.get(2);
    //%%let (default_username: Option<String>, default_password: Option<&String>) = (args.get(1), args.get(2));
    dbg!(default_username, default_password); //%%
    let force_user_pass_prompt: bool = if args.len() == 1 { true } else { false };
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
    //%%:
    controller.start();

    /*%%
let mut rl = Editor::<()>::new()?;

let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                println!("Line: {}", line);
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }

        let mTerminal: jline.Terminal = initializeTerminal();
        let jlineReader: ConsoleReader = initializeReader();

        // used to coordinate the mTerminal initialization (problems still happened when it wasn't lazy), and the cleanup thread, so that
        // the cleanup actually happens.
        private let mCleanupStarted: bool = false;

        private let mut mJlineTerminalInitFinished: bool = false;
        private let mut mJlineReaderInitFinished: bool = false;
        %%
    */
}
